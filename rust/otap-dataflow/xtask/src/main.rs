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

use std::io::{BufRead, BufReader};
use std::process::{Command, ExitStatus, Stdio};
use std::sync::mpsc;
use std::thread;
use std::time::Instant;

mod diagnostics;
mod genproto;
mod structure_check;

#[cfg(not(tarpaulin_include))]
fn main() -> anyhow::Result<()> {
    let mut args = std::env::args().skip(1);
    let task = args.next();

    match task {
        None => print_help(),
        Some(task) => match task.as_str() {
            "check" => check_all(parse_check_options(&args.collect::<Vec<_>>())?),
            "quick-check" => {
                ensure_no_extra_args("quick-check", &args.collect::<Vec<_>>())?;
                quick_check()
            }
            "check-benches" => {
                ensure_no_extra_args("check-benches", &args.collect::<Vec<_>>())?;
                check_benches()
            }
            "compile-proto" => {
                ensure_no_extra_args("compile-proto", &args.collect::<Vec<_>>())?;
                genproto::compile_proto()?;
                Ok(())
            }
            "structure-check" => {
                ensure_no_extra_args("structure-check", &args.collect::<Vec<_>>())?;
                structure_check::run()
            }
            "help" => {
                ensure_no_extra_args("help", &args.collect::<Vec<_>>())?;
                print_help()
            }
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
  - check [--diagnostics]: Run the required full validation suite: structure check, cargo fmt --all, cargo clippy --workspace --all-targets, and cargo test --workspace. The optional diagnostics flag prints end-of-run timing and hotspot summaries.
  - quick-check: Run a faster iterative subset: structure check, cargo fmt --all, cargo clippy --workspace --lib --bins --tests, and cargo test --workspace --lib --bins --tests --no-run. This is not a replacement for `cargo xtask check`.
  - check-benches: Lint and compile bench targets only.
  - structure-check: Validate the entire structure of the project.
  - compile-proto: Compile the protobufs files
"
    );
    Ok(())
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CheckOptions {
    diagnostics: bool,
}

fn parse_check_options(args: &[String]) -> anyhow::Result<CheckOptions> {
    let mut options = CheckOptions::default();
    for arg in args {
        match arg.as_str() {
            "--diagnostics" => options.diagnostics = true,
            _ => anyhow::bail!("Unknown `cargo xtask check` argument: {arg}"),
        }
    }
    Ok(options)
}

fn ensure_no_extra_args(task: &str, args: &[String]) -> anyhow::Result<()> {
    if args.is_empty() {
        Ok(())
    } else {
        anyhow::bail!(
            "`cargo xtask {task}` does not accept extra arguments: {}",
            args.join(" ")
        );
    }
}

fn check_all(options: CheckOptions) -> anyhow::Result<()> {
    let mut diagnostics = options
        .diagnostics
        .then(diagnostics::DiagnosticsCollector::new);

    run_structure_step(diagnostics.as_mut())?;
    format_all(diagnostics.as_mut())?;
    clippy_all(options, diagnostics.as_mut())?;
    test_all(options, diagnostics.as_mut())?;

    if let Some(diagnostics) = diagnostics.as_ref() {
        diagnostics.print_summary();
    }

    Ok(())
}

fn quick_check() -> anyhow::Result<()> {
    structure_check::run()?;
    format_all(None)?;
    clippy_quick()?;
    test_quick()?;
    Ok(())
}

fn check_benches() -> anyhow::Result<()> {
    clippy_benches()?;
    compile_benches()?;
    Ok(())
}

fn run_structure_step(
    mut diagnostics: Option<&mut diagnostics::DiagnosticsCollector>,
) -> anyhow::Result<()> {
    let start = Instant::now();
    let result = structure_check::run();
    let duration = start.elapsed();

    if let Some(diagnostics) = &mut diagnostics {
        diagnostics.record_step("structure", duration, step_status_from_result(&result));
    }

    if result.is_err() {
        if let Some(diagnostics) = &mut diagnostics {
            diagnostics.print_summary();
        }
    }

    result
}

fn format_all(
    mut diagnostics: Option<&mut diagnostics::DiagnosticsCollector>,
) -> anyhow::Result<()> {
    println!("🚀 Formatting workspace with cargo fmt...");
    let start = Instant::now();
    let result = run("cargo", &["fmt", "--all"]);
    let duration = start.elapsed();

    if let Some(diagnostics) = &mut diagnostics {
        diagnostics.record_step("fmt", duration, step_status_from_result(&result));
    }

    if result.is_err() {
        if let Some(diagnostics) = &mut diagnostics {
            diagnostics.print_summary();
        }
    }

    result?;
    println!("✅ Formatting completed successfully.\n");
    Ok(())
}

fn clippy_all(
    options: CheckOptions,
    diagnostics: Option<&mut diagnostics::DiagnosticsCollector>,
) -> anyhow::Result<()> {
    println!("🚀 Linting workspace with cargo clippy...");
    let mut args = vec![
        "clippy".to_owned(),
        "--workspace".to_owned(),
        "--all-features".to_owned(),
        "--all-targets".to_owned(),
        "--".to_owned(),
        "-D".to_owned(),
        "warnings".to_owned(),
    ];
    if options.diagnostics {
        args.push("--timings".to_owned());
    }
    args.extend(["--", "-D", "warnings"].into_iter().map(ToOwned::to_owned));

    let start = Instant::now();
    let result = if options.diagnostics {
        let outcome = run_capture("cargo", &args)?;
        let status = outcome.status;
        if let Some(diagnostics) = diagnostics {
            diagnostics.record_compile_step("clippy", &outcome.lines);
            diagnostics.record_step("clippy", start.elapsed(), step_status_from_status(status));
            if !status.success() {
                diagnostics.print_summary();
            }
        }
        ensure_success("cargo", status)
    } else {
        let result = run(
            "cargo",
            &[
                "clippy",
                "--workspace",
                "--all-targets",
                "--",
                "-D",
                "warnings",
            ],
        );
        if let Some(diagnostics) = diagnostics {
            diagnostics.record_step("clippy", start.elapsed(), step_status_from_result(&result));
            if result.is_err() {
                diagnostics.print_summary();
            }
        }
        result
    };

    result?;
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

fn test_all(
    options: CheckOptions,
    diagnostics: Option<&mut diagnostics::DiagnosticsCollector>,
) -> anyhow::Result<()> {
    println!("🚀 Running workspace tests with cargo test...");
    let mut args = vec!["test".to_owned(), "--workspace".to_owned()];
    if options.diagnostics {
        args.push("--timings".to_owned());
    }

    let start = Instant::now();
    let result = if options.diagnostics {
        let outcome = run_capture("cargo", &args)?;
        let status = outcome.status;
        if let Some(diagnostics) = diagnostics {
            diagnostics.record_compile_step("test", &outcome.lines);
            diagnostics.record_test_binaries(&outcome.lines);
            diagnostics.record_step("test", start.elapsed(), step_status_from_status(status));
            if !status.success() {
                diagnostics.print_summary();
            }
        }
        ensure_success("cargo", status)
    } else {
        let result = run("cargo", &["test", "--workspace"]);
        if let Some(diagnostics) = diagnostics {
            diagnostics.record_step("test", start.elapsed(), step_status_from_result(&result));
            if result.is_err() {
                diagnostics.print_summary();
            }
        }
        result
    };

    result?;
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

    ensure_success(command, status)
}

struct CommandOutcome {
    status: ExitStatus,
    lines: Vec<diagnostics::CapturedLine>,
}

fn run_capture(command: &str, args: &[String]) -> anyhow::Result<CommandOutcome> {
    let mut child = Command::new(command)
        .args(args)
        .env("CARGO_TERM_COLOR", "always")
        .env("CARGO_TERM_HYPERLINKS", "false")
        .env("CLICOLOR_FORCE", "1")
        .env_remove("NO_COLOR")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?;

    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture stdout from `{command}`"))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| anyhow::anyhow!("failed to capture stderr from `{command}`"))?;

    let (tx, rx) = mpsc::channel::<diagnostics::CapturedLine>();
    let stdout_tx = tx.clone();
    let stdout_handle = thread::spawn(move || stream_lines(stdout, false, stdout_tx));
    let stderr_handle = thread::spawn(move || stream_lines(stderr, true, tx));

    let mut lines = Vec::new();
    for line in rx {
        if line.is_stderr {
            eprintln!("{}", line.text);
        } else {
            println!("{}", line.text);
        }
        lines.push(line);
    }

    stdout_handle
        .join()
        .map_err(|_| anyhow::anyhow!("stdout reader thread panicked"))??;
    stderr_handle
        .join()
        .map_err(|_| anyhow::anyhow!("stderr reader thread panicked"))??;

    let status = child.wait()?;
    Ok(CommandOutcome { status, lines })
}

fn stream_lines<R: std::io::Read>(
    reader: R,
    is_stderr: bool,
    tx: mpsc::Sender<diagnostics::CapturedLine>,
) -> anyhow::Result<()> {
    for line in BufReader::new(reader).lines() {
        tx.send(diagnostics::CapturedLine {
            is_stderr,
            text: line?,
        })?;
    }
    Ok(())
}

fn ensure_success(command: &str, status: ExitStatus) -> anyhow::Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!(
            "Command `{}` failed with status {}",
            command,
            status
        ))
    }
}

fn step_status_from_result(result: &anyhow::Result<()>) -> diagnostics::StepStatus {
    if result.is_ok() {
        diagnostics::StepStatus::Succeeded
    } else {
        diagnostics::StepStatus::Failed
    }
}

fn step_status_from_status(status: ExitStatus) -> diagnostics::StepStatus {
    if status.success() {
        diagnostics::StepStatus::Succeeded
    } else {
        diagnostics::StepStatus::Failed
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_check_options_accepts_diagnostics_flag() {
        let args = vec!["--diagnostics".to_owned()];
        let options = parse_check_options(&args).expect("flag should parse");
        assert!(options.diagnostics);
    }

    #[test]
    fn parse_check_options_rejects_unknown_flag() {
        let args = vec!["--unknown".to_owned()];
        assert!(parse_check_options(&args).is_err());
    }

    #[test]
    fn non_check_tasks_reject_extra_args() {
        let args = vec!["--diagnostics".to_owned()];
        assert!(ensure_no_extra_args("quick-check", &args).is_err());
    }
}
