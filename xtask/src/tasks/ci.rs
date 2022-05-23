use colored::Colorize;
use std::process::{Command, ExitStatus, Stdio};

use crate::DynError;

pub fn ci() -> Result<(), DynError> {
    println!("Running `cargo check`...");
    let cargo_check_status = Command::new("cargo")
        .args(["check", "-p", "ravendb-client"])
        .stdout(Stdio::inherit())
        .status()?;

    println!("Running `cargo clippy`...");
    let cargo_clippy_status = Command::new("cargo")
        .args(["clippy", "-p", "ravendb-client"])
        .stdout(Stdio::inherit())
        .status()?;

    println!("Running `cargo build`...");
    let cargo_build_status = Command::new("cargo")
        .args(["build", "-p", "ravendb-client"])
        .stdout(Stdio::inherit())
        .status()?;

    println!("Running `cargo test`...");
    let cargo_test_status = Command::new("cargo")
        .args(["test", "-p", "ravendb-client"])
        .stdout(Stdio::inherit())
        .status()?;

    println!("Running `cargo audit`...");
    let cargo_audit_status = Command::new("cargo")
        .args(["audit"])
        .stdout(Stdio::inherit())
        .status()?;

    println!("Running `cargo fmt`...");
    let cargo_fmt_status = Command::new("cargo")
        .args(["fmt"])
        .stdout(Stdio::inherit())
        .status()?;

    print_error_with_status_code("cargo check", cargo_check_status);
    print_error_with_status_code("cargo clippy", cargo_clippy_status);
    print_error_with_status_code("cargo build", cargo_build_status);
    print_error_with_status_code("cargo test", cargo_test_status);
    print_error_with_status_code("cargo audit", cargo_audit_status);
    print_error_with_status_code("cargo fmt", cargo_fmt_status);

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
