// Copyright The OpenTelemetry Authors
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

mod genproto;
mod structure_check;

#[cfg(not(tarpaulin_include))]
fn main() -> anyhow::Result<()> {
    let task = std::env::args().nth(1);

    match task {
        None => print_help(),
        Some(task) => match task.as_str() {
            "check" => check_all(),
            "quick-check" => quick_check(),
            "check-benches" => check_benches(),
            "compile-proto" => {
                genproto::compile_proto()?;
                Ok(())
            }
            "structure-check" => structure_check::run(),
            "help" => print_help(),
            _ => {
                eprintln!("Unknown task: {task}");
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
  - check: Run the full check suite (format, clippy all-targets, full tests).
  - quick-check: Faster developer check (format, clippy on libs/bins/tests, compile test targets without running them).
  - check-benches: Lint and compile bench targets only.
  - structure-check: Validate the entire structure of the project.
  - compile-proto: Compile the protobufs files
"
    );
    Ok(())
}

fn check_all() -> anyhow::Result<()> {
    structure_check::run()?;
    format_all()?;
    clippy_all()?;
    test_all()?;
    Ok(())
}

fn quick_check() -> anyhow::Result<()> {
    structure_check::run()?;
    format_all()?;
    clippy_quick()?;
    test_quick()?;
    Ok(())
}

fn check_benches() -> anyhow::Result<()> {
    clippy_benches()?;
    compile_benches()?;
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
    run(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--all-targets",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    println!("✅ Clippy linting passed without warnings.\n");
    Ok(())
}

fn clippy_quick() -> anyhow::Result<()> {
    println!("🚀 Linting workspace targets for fast developer checks...");
    run(
        "cargo",
        &[
            "clippy",
            "--workspace",
            "--lib",
            "--bins",
            "--tests",
            "--",
            "-D",
            "warnings",
        ],
    )?;
    println!("✅ Fast clippy linting passed without warnings.\n");
    Ok(())
}

fn clippy_benches() -> anyhow::Result<()> {
    println!("🚀 Linting workspace bench targets with cargo clippy...");
    run(
        "cargo",
        &["clippy", "--workspace", "--benches", "--", "-D", "warnings"],
    )?;
    println!("✅ Bench clippy linting passed without warnings.\n");
    Ok(())
}

fn test_all() -> anyhow::Result<()> {
    println!("🚀 Running workspace tests with cargo test...");
    run("cargo", &["test", "--workspace"])?;
    println!("✅ All tests passed successfully.\n");
    Ok(())
}

fn test_quick() -> anyhow::Result<()> {
    println!("🚀 Compiling fast workspace test targets (no benches/examples/doctests)...");
    run(
        "cargo",
        &[
            "test",
            "--workspace",
            "--lib",
            "--bins",
            "--tests",
            "--no-run",
        ],
    )?;
    println!("✅ Fast workspace test targets compiled successfully.\n");
    Ok(())
}

fn compile_benches() -> anyhow::Result<()> {
    println!("🚀 Compiling workspace bench targets with cargo check...");
    run("cargo", &["check", "--workspace", "--benches"])?;
    println!("✅ Bench targets compiled successfully.\n");
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
