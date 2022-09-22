use colored::Colorize;
use std::process::{Command, ExitStatus, Stdio};

use crate::DynError;

pub fn ci() -> Result<(), DynError> {
    let tasks = vec![
        ("cargo check on code", vec!["check"]),
        ("cargo check on examples", vec!["check", "--examples"]),
        ("cargo clippy", vec!["clippy", "--", "-D", "warnings"]),
        ("cargo build", vec!["build"]),
        ("cargo build on examples", vec!["build", "--examples"]),
        ("cargo nextest", vec!["nextest", "run"]),
        ("cargo test", vec!["test", "--doc"]),
        // (
        //     "cargo nextest on examples",
        //     vec!["nextest", "run", "--examples"],
        // ),
        ("cargo test on examples", vec!["test", "--examples"]),
        ("cargo audit", vec!["audit"]),
        ("cargo fmt", vec!["fmt"]),
    ];

    for (name, args) in tasks {
        let mut cmd = cargo_command(args);
        println!(
            "{}{}{}",
            "Running ".truecolor(255, 165, 0),
            name.truecolor(255, 165, 0),
            "...".truecolor(255, 165, 0)
        );
        let status = cmd.status()?;
        print_error_with_status_code(name, status);
        if !status.success() {
            break;
        }
    }

    Ok(())
}

fn print_error_with_status_code(task: &str, status: ExitStatus) {
    let code = match status.code() {
        Some(x) => x.to_string(),
        None => "<< no status code >>".to_string(),
    };
    if !status.success() {
        println!(
            "{} `{}` finished with a non-zero status code: {}",
            "Error:".to_string().red(),
            task.blue(),
            code
        );
    }
}

fn cargo_command(args: Vec<&str>) -> Command {
    let mut cmd = Command::new("cargo");
    cmd.args(args).stdout(Stdio::inherit());
    cmd
}
