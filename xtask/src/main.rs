use std::env;

use xtask::{
    tasks::{ci::ci, distribute::dist},
    DynError,
};

fn main() {
    if let Err(e) = try_main() {
        eprintln!("{}", e);
        std::process::exit(-1);
    }
}

fn try_main() -> Result<(), DynError> {
    let task = env::args().nth(1);
    match task.as_deref() {
        Some("dist") => dist()?,
        Some("ci") => ci()?,
        _ => print_help(),
    }
    Ok(())
}

fn print_help() {
    eprintln!(
        "
Usage: cargo xtask <task>

Tasks:
  ci              runs all necessary checks to avoid CI errors when git pushed
  dist            builds application and man pages
"
    )
}
