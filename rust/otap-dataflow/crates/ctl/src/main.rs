// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Binary entrypoint for the `dfctl` OTAP Dataflow Engine admin CLI.
//!
//! This module is intentionally thin: it parses process arguments, detects
//! whether stdout is a terminal, creates the Tokio runtime, and maps library
//! errors to process exit codes. Keeping those process concerns here lets the
//! library remain easy to test with in-memory stdout/stderr streams while still
//! preserving the special terminal handling required by the interactive TUI.

use std::io::{self, IsTerminal, Write};

fn main() -> std::process::ExitCode {
    let cli = otap_df_ctl::Cli::parse_effective();
    let error_format = cli.error_format();
    let stdout_is_terminal = io::stdout().is_terminal();
    if cli.is_ui() {
        let mut stderr = io::stderr().lock();
        return match run_ui(cli, stdout_is_terminal, error_format, &mut stderr) {
            Ok(code) => code,
            Err(code) => code,
        };
    }

    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();
    match run(
        cli,
        stdout_is_terminal,
        error_format,
        &mut stdout,
        &mut stderr,
    ) {
        Ok(code) => code,
        Err(code) => code,
    }
}

fn run(
    cli: otap_df_ctl::Cli,
    stdout_is_terminal: bool,
    error_format: otap_df_ctl::ErrorFormat,
    stdout: &mut dyn Write,
    stderr: &mut dyn Write,
) -> Result<std::process::ExitCode, std::process::ExitCode> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| {
            let _ = writeln!(stderr, "error: failed to start tokio runtime: {err}");
            std::process::ExitCode::from(6)
        })?;

    match runtime.block_on(otap_df_ctl::run_with_terminal_and_diagnostics(
        cli,
        stdout,
        stdout_is_terminal,
        stderr,
    )) {
        Ok(()) => Ok(std::process::ExitCode::SUCCESS),
        Err(err) => {
            if err.should_print() {
                let _ = err.write_to(stderr, error_format);
            }
            Err(std::process::ExitCode::from(err.exit_code()))
        }
    }
}

fn run_ui(
    cli: otap_df_ctl::Cli,
    stdout_is_terminal: bool,
    error_format: otap_df_ctl::ErrorFormat,
    stderr: &mut dyn Write,
) -> Result<std::process::ExitCode, std::process::ExitCode> {
    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(|err| {
            let _ = writeln!(stderr, "error: failed to start tokio runtime: {err}");
            std::process::ExitCode::from(6)
        })?;

    match runtime.block_on(otap_df_ctl::run_with_terminal_and_diagnostics(
        cli,
        &mut io::sink(),
        stdout_is_terminal,
        stderr,
    )) {
        Ok(()) => Ok(std::process::ExitCode::SUCCESS),
        Err(err) => {
            if err.should_print() {
                let _ = err.write_to(stderr, error_format);
            }
            Err(std::process::ExitCode::from(err.exit_code()))
        }
    }
}
