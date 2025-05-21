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
            "compile-proto" => {
                // compile_proto_otlp()?;
                compile_proto_otap()?;
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
  - compile-proto: Compile the protobufs files
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

fn compile_proto_otlp() -> anyhow::Result<()> {
    tonic_build::configure()
        .out_dir("crates/otlp/src/proto")
        .compile_protos(
            &[
                "opentelemetry/proto/common/v1/common.proto",
                "opentelemetry/proto/resource/v1/resource.proto",
                "opentelemetry/proto/profiles/v1development/profiles.proto",
                "opentelemetry/proto/trace/v1/trace.proto",
                "opentelemetry/proto/metrics/v1/metrics.proto",
                "opentelemetry/proto/logs/v1/logs.proto",
                "opentelemetry/proto/collector/logs/v1/logs_service.proto",
                "opentelemetry/proto/collector/trace/v1/trace_service.proto",
                "opentelemetry/proto/collector/metrics/v1/metrics_service.proto",
                "opentelemetry/proto/collector/profiles/v1development/profiles_service.proto",
            ],
            &["../../proto/opentelemetry-proto"],
        )
        .expect("Failed to compile OTLP protos.");
    Ok(())
}

fn compile_proto_otap() -> anyhow::Result<()> {
    tonic_build::configure()
        .out_dir("crates/otap/src/proto")
        .compile_protos(
            &[
                "proto/experimental/arrow/v1/arrow_service.proto"
            ],
            &["../../proto/opentelemetry"],
        )
        .expect("Failed to compile OTLP protos.");
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
