// SPDX-License-Identifier: Apache-2.0

//! See <https://github.com/matklad/cargo-xtask/>.
//!
//! This binary defines various auxiliary build commands, which are not
//! expressible with just `cargo`.
//!
//! This binary is integrated into the `cargo` command line by using an alias in
//! `.cargo/config`.

// This crate is a CLI tool and can use stdout and stderr for logging.
#![allow(clippy::print_stdout)]
#![allow(clippy::print_stderr)]

use std::process::Command;

mod structure_check;

#[cfg(not(tarpaulin_include))]
fn main() -> anyhow::Result<()> {
    let task = std::env::args().nth(1);

    match task {
        None => print_help(),
        Some(task) => match task.as_str() {
            "check" => {
                structure_check::run()?;
                format_all()?;
                clippy_all()?;
                test_all()?;
                Ok(())
            }
            "structure-check" => structure_check::run(),
            "help" => print_help(),
            _ => {
                eprintln!("Unknown task: {}", task);
                print_help()
            }
        },
    }
}

/// Prints help message.
#[cfg(not(tarpaulin_include))]
pub fn print_help() -> anyhow::Result<()> {
    println!(
        "
Usage: Execute the command using `cargo xtask <task>`, e.g., `cargo xtask check`.

Tasks:
  - check: Run all checks.
  - structure-check: Validate the entire structure of the project.
"
    );
    Ok(())
}

fn format_all() -> anyhow::Result<()> {
    println!("🚀 Formatting workspace with cargo fmt...");
    run("cargo", &["fmt", "--all"])?;
    println!("✅ Formatting completed successfully.\n");
    Ok(())
}

fn clippy_all() -> anyhow::Result<()> {
    println!("🚀 Linting workspace with cargo clippy...");
    run("cargo", &["clippy", "--workspace", "--", "-D", "warnings"])?;
    println!("✅ Clippy linting passed without warnings.\n");
    Ok(())
}

fn test_all() -> anyhow::Result<()> {
    println!("🚀 Running workspace tests with cargo test...");
    run("cargo", &["test", "--workspace"])?;
    println!("✅ All tests passed successfully.\n");
    Ok(())
}

fn run(command: &str, args: &[&str]) -> anyhow::Result<()> {
    let status = Command::new(command).args(args).status()?;

    if !status.success() {
        Err(anyhow::anyhow!(
            "Command `{}` failed with status {}",
            command,
            status
        ))
    } else {
        Ok(())
    }
}
