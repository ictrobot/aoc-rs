use std::thread::JoinHandle;

#[derive(Debug)]
pub struct AutoJoinHandle<T>(Option<JoinHandle<T>>);

impl<T> From<JoinHandle<T>> for AutoJoinHandle<T> {
    fn from(value: JoinHandle<T>) -> Self {
        AutoJoinHandle(Some(value))
    }
}

impl<T> Drop for AutoJoinHandle<T> {
    fn drop(&mut self) {
        if let Some(handle) = self.0.take() {
            handle.join().unwrap();
        }
    }
}
