use crate::cli::mode::test::thread::AutoJoinHandle;
use crate::cli::mode::test::{mpmc, oneshot};
use std::fmt::Debug;
use std::io;
use std::num::NonZeroUsize;
use std::process::{Child, ChildStderr, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread::{Builder, sleep};
use std::time::{Duration, Instant};

#[derive(Debug)]
pub struct ProcessPool<S, T> {
    // Field order controls drop order. First close the channel for new jobs, then join the threads
    // and finally close the result channel.
    job_sender: Option<mpmc::Sender<(S, T, ProcessJob)>>,
    managers: Vec<ProcessManager>,
    event_receiver: mpsc::Receiver<ProcessEvent<S, T>>,
    pending_results: usize,
}

pub enum ProcessEvent<S, T> {
    Started(S),
    Finished(T, io::Result<ProcessResult>),
}

#[derive(Debug)]
pub struct ProcessResult {
    pub killed: bool,
    pub exit_status: ExitStatus,
    pub stdin: Option<io::Error>,
    pub stdout: io::Result<String>,
    pub stderr: io::Result<String>,
}

impl<S: Debug + Send + 'static, T: Debug + Send + 'static> ProcessPool<S, T> {
    pub fn new(max_processes: NonZeroUsize) -> io::Result<Self> {
        let (job_sender, job_receiver) = mpmc::channel();
        let (event_sender, event_receiver) = mpsc::channel();

        Ok(ProcessPool {
            job_sender: Some(job_sender),
            managers: (0..max_processes.get())
                .map(|num| ProcessManager::new(job_receiver.clone(), event_sender.clone(), num))
                .collect::<io::Result<_>>()?,
            event_receiver,
            pending_results: 0,
        })
    }

    pub fn enqueue(
        &mut self,
        cmd: Command,
        stdin: String,
        timeout: Duration,
        started_id: S,
        id: T,
    ) {
        let Some(sender) = &self.job_sender else {
            panic!("pool has been closed to new jobs");
        };
        self.pending_results += 1;

        sender
            .send((
                started_id,
                id,
                ProcessJob {
                    cmd,
                    stdin,
                    timeout,
                },
            ))
            .expect("failed to send job");
    }

    pub fn close(&mut self) {
        self.job_sender.take();
    }

    pub fn recv_timeout(&mut self, timeout: Duration) -> Option<ProcessEvent<S, T>> {
        let event = self.event_receiver.recv_timeout(timeout).ok();
        if matches!(event, Some(ProcessEvent::Finished(_, _))) {
            self.pending_results -= 1;
        }
        event
    }

    pub fn max_processes(&self) -> usize {
        self.managers.len()
    }

    pub fn pending_results(&self) -> usize {
        self.pending_results
    }
}

#[derive(Debug)]
struct ProcessManager {
    // Field order controls drop order. Drop the manager thread first as it should be the first to
    // exit, after the job sender is dropped, which will then close the senders for the remaining
    // threads, causing them to exit.
    _manager: AutoJoinHandle<()>,
    _stdin: AutoJoinHandle<()>,
    _stdout: AutoJoinHandle<()>,
    _stderr: AutoJoinHandle<()>,
}

impl ProcessManager {
    fn new<S: Send + 'static, T: Send + 'static>(
        job_receiver: mpmc::Receiver<(S, T, ProcessJob)>,
        event_sender: mpsc::Sender<ProcessEvent<S, T>>,
        num: usize,
    ) -> io::Result<Self> {
        let (stdin_sender, stdin_receiver) = mpsc::sync_channel(0);
        let (stdout_sender, stdout_receiver) = mpsc::sync_channel(0);
        let (stderr_sender, stderr_receiver) = mpsc::sync_channel(0);

        // AutoJoinHandle should automatically join any existing threads if spawning a thread fails
        Ok(Self {
            _manager: Builder::new()
                .name(format!("{num}-manager"))
                .spawn(ProcessJob::worker(
                    job_receiver,
                    event_sender,
                    stdin_sender,
                    stdout_sender,
                    stderr_sender,
                ))?
                .into(),
            _stdin: Builder::new()
                .name(format!("{num}-stdin"))
                .spawn(WriteJob::worker(stdin_receiver))?
                .into(),
            _stdout: Builder::new()
                .name(format!("{num}-stdout"))
                .spawn(ReadJob::worker(stdout_receiver))?
                .into(),
            _stderr: Builder::new()
                .name(format!("{num}-stderr"))
                .spawn(ReadJob::worker(stderr_receiver))?
                .into(),
        })
    }
}

#[derive(Debug)]
struct ProcessJob {
    cmd: Command,
    stdin: String,
    timeout: Duration,
}

impl ProcessJob {
    fn worker<S: Send + 'static, T: Send + 'static>(
        job_receiver: mpmc::Receiver<(S, T, Self)>,
        event_sender: mpsc::Sender<ProcessEvent<S, T>>,
        stdin_job_sender: mpsc::SyncSender<WriteJob<ChildStdin>>,
        stdout_job_sender: mpsc::SyncSender<ReadJob<ChildStdout>>,
        stderr_job_sender: mpsc::SyncSender<ReadJob<ChildStderr>>,
    ) -> impl FnOnce() + Send + 'static {
        move || {
            while let Some((start_id, id, job)) = job_receiver.recv() {
                if event_sender.send(ProcessEvent::Started(start_id)).is_err() {
                    return; // The receiver has gone away
                }

                let result = job
                    .spawn(&stdin_job_sender, &stdout_job_sender, &stderr_job_sender)
                    .and_then(Process::wait);

                if event_sender
                    .send(ProcessEvent::Finished(id, result))
                    .is_err()
                {
                    return;
                }
            }
        }
    }

    fn spawn(
        mut self,
        stdin_job_sender: &mpsc::SyncSender<WriteJob<ChildStdin>>,
        stdout_job_sender: &mpsc::SyncSender<ReadJob<ChildStdout>>,
        stderr_job_sender: &mpsc::SyncSender<ReadJob<ChildStderr>>,
    ) -> io::Result<Process> {
        self.cmd.stdin(Stdio::piped());
        self.cmd.stdout(Stdio::piped());
        self.cmd.stderr(Stdio::piped());

        let start = Instant::now();
        let mut child = self.cmd.spawn()?;

        let (stdin_sender, stdin_receiver) = oneshot::channel();
        stdin_job_sender
            .send(WriteJob {
                writer: child.stdin.take().expect("failed to take stdin"),
                data: self.stdin,
                result_sender: stdin_sender,
            })
            .expect("failed to send stdin job");

        let (stdout_sender, stdout_receiver) = oneshot::channel();
        stdout_job_sender
            .send(ReadJob {
                reader: child.stdout.take().expect("failed to take stdout"),
                result_sender: stdout_sender,
            })
            .expect("failed to send stdout job");

        let (stderr_sender, stderr_receiver) = oneshot::channel();
        stderr_job_sender
            .send(ReadJob {
                reader: child.stderr.take().expect("failed to take stderr"),
                result_sender: stderr_sender,
            })
            .expect("failed to send stderr job");

        Ok(Process {
            child,
            stdin_receiver,
            stdout_receiver,
            stderr_receiver,
            start,
            deadline: start + self.timeout,
        })
    }
}

struct Process {
    child: Child,
    stdin_receiver: oneshot::Receiver<Option<io::Error>>,
    stdout_receiver: oneshot::Receiver<io::Result<String>>,
    stderr_receiver: oneshot::Receiver<io::Result<String>>,
    start: Instant,
    deadline: Instant,
}

impl Process {
    // Many solutions finish very quickly, within ~5ms, so attempt to poll every 0.25 ms for the
    // first 5 ms (actual sleep durations likely to be slightly longer). After that, slowly increase
    // the interval by adding 1/12th of the interval each time. By ~50ms the interval should be
    // ~4 ms, and by 100 ms it should be ~8 ms. The interval is capped at 50 ms which should be
    // reached after ~650 ms.
    const POLL_INITIAL_INTERVAL: Duration = Duration::from_micros(250);
    const POLL_BACKOFF_AFTER: Duration = Duration::from_millis(5);
    const BACKOFF_DIVISOR: u32 = 12;
    const POLL_MAX_INTERVAL: Duration = Duration::from_millis(50);

    fn wait(mut self) -> io::Result<ProcessResult> {
        let mut now = Instant::now();
        let mut next_poll = self.start + Self::POLL_INITIAL_INTERVAL;
        let mut interval = Self::POLL_INITIAL_INTERVAL;

        let (exit_status, killed) = loop {
            if next_poll > now {
                sleep(next_poll - now);
                now = Instant::now();
            }

            if let Some(status) = self.child.try_wait()? {
                // Process finished
                break (status, false);
            }
            if now > self.deadline {
                // Process exceeded deadline
                self.child.kill()?;
                break (self.child.wait()?, true);
            }

            next_poll = (now + interval).min(self.deadline);
            if next_poll - self.start > Self::POLL_BACKOFF_AFTER {
                interval = interval
                    .checked_add(interval / Self::BACKOFF_DIVISOR)
                    .unwrap_or(Self::POLL_MAX_INTERVAL)
                    .min(Self::POLL_MAX_INTERVAL);
            }
            now = Instant::now();
        };

        Ok(ProcessResult {
            killed,
            exit_status,
            stdin: self.stdin_receiver.recv().expect("failed to receive stdin"),
            stdout: self
                .stdout_receiver
                .recv()
                .expect("failed to receive stdout"),
            stderr: self
                .stderr_receiver
                .recv()
                .expect("failed to receive stderr"),
        })
    }
}

#[derive(Debug)]
struct WriteJob<T> {
    writer: T,
    data: String,
    result_sender: oneshot::Sender<Option<io::Error>>,
}

impl<T: io::Write + Send + 'static> WriteJob<T> {
    fn worker(queue: mpsc::Receiver<Self>) -> impl FnOnce() + Send + 'static {
        move || {
            while let Ok(mut job) = queue.recv() {
                let result = job.writer.write_all(job.data.as_bytes()).err();
                // Ignore errors from the receiver being closed
                let _ = job.result_sender.send(result);
            }
        }
    }
}

#[derive(Debug)]
struct ReadJob<T> {
    reader: T,
    result_sender: oneshot::Sender<io::Result<String>>,
}

impl<T: io::Read + Send + 'static> ReadJob<T> {
    fn worker(queue: mpsc::Receiver<Self>) -> impl FnOnce() + Send + 'static {
        move || {
            while let Ok(mut job) = queue.recv() {
                let mut buf = String::new();
                let result = job.reader.read_to_string(&mut buf).map(|_| buf);
                // Ignore errors from the receiver being closed
                let _ = job.result_sender.send(result);
            }
        }
    }
}
