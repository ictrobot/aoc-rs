use std::env;
use std::error::Error;
use utils::date::{Day, Year};

pub mod cmd;
pub mod common;

fn main() {
    let mut args = env::args().skip(1);
    let subcommand = args.next().expect("expected subcommand");
    if let Err(e) = match subcommand.as_str() {
        "input" => cmd::input::main(args),
        "new" => cmd::new::main(args),
        "update" => cmd::update::main(args),
        "wait" => cmd::wait::main(args),
        _ => panic!("unknown subcommand"),
    } {
        eprintln!("error: {e}");
        std::process::exit(1);
    }
}

pub(crate) fn year_arg(args: &mut impl Iterator<Item = String>) -> Result<Year, Box<dyn Error>> {
    Ok(args
        .next()
        .ok_or_else(|| Box::<dyn Error>::from("expected year argument"))?
        .parse::<Year>()?)
}

pub(crate) fn day_arg(args: &mut impl Iterator<Item = String>) -> Result<Day, Box<dyn Error>> {
    Ok(args
        .next()
        .ok_or_else(|| Box::<dyn Error>::from("expected day argument"))?
        .parse::<Day>()?)
}

pub(crate) fn ensure_no_args(mut args: impl Iterator<Item = String>) -> Result<(), Box<dyn Error>> {
    match args.next() {
        None => Ok(()),
        Some(_) => Err("unexpected extra arguments".into()),
    }
}
