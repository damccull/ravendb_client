use std::process::Command;

use crate::DynError;

pub fn ci() -> Result<(), DynError> {
    println!("Running `cargo check`...");
    let check = Command::new("cargo").args(["check"]).output()?;
    println!("{}", std::str::from_utf8(&check.stdout)?);
    println!("{}", std::str::from_utf8(&check.stderr)?);

    println!("Running `cargo clippy`...");
    let clippy = Command::new("cargo").args(["clippy"]).output()?;
    println!("{}", std::str::from_utf8(&clippy.stdout)?);
    println!("{}", std::str::from_utf8(&clippy.stderr)?);

    println!("Running `cargo build`...");
    let build = Command::new("cargo").args(["build"]).output()?;
    println!("{}", std::str::from_utf8(&build.stdout)?);
    println!("{}", std::str::from_utf8(&build.stderr)?);

    println!("Running `cargo test`...");
    let test = Command::new("cargo").args(["test"]).output()?;
    println!("{}", std::str::from_utf8(&test.stdout)?);
    println!("{}", std::str::from_utf8(&test.stderr)?);

    println!("Running `cargo audit`...");
    let audit = Command::new("cargo").args(["audit"]).output()?;
    println!("{}", std::str::from_utf8(&audit.stdout)?);
    println!("{}", std::str::from_utf8(&audit.stderr)?);

    println!("Running `cargo fmt`...");
    let fmt = Command::new("cargo").args(["fmt"]).output()?;
    println!("{}", std::str::from_utf8(&fmt.stdout)?);
    println!("{}", std::str::from_utf8(&fmt.stderr)?);

    Ok(())
}
