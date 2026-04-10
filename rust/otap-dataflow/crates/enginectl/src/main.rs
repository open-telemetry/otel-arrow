// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Binary entrypoint for the `dfctl` OTAP Dataflow Engine admin CLI.

use clap::Parser;
use std::io::{self, IsTerminal, Write};

fn main() -> std::process::ExitCode {
    let cli = otap_df_enginectl::Cli::parse();
    let stdout_is_terminal = io::stdout().is_terminal();
    let mut stdout = io::stdout().lock();
    let mut stderr = io::stderr().lock();

    match run(cli, stdout_is_terminal, &mut stdout, &mut stderr) {
        Ok(code) => code,
        Err(code) => code,
    }
}

fn run(
    cli: otap_df_enginectl::Cli,
    stdout_is_terminal: bool,
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

    match runtime.block_on(otap_df_enginectl::run_with_terminal(
        cli,
        stdout,
        stdout_is_terminal,
    )) {
        Ok(()) => Ok(std::process::ExitCode::SUCCESS),
        Err(err) => {
            if err.should_print() {
                let _ = writeln!(stderr, "error: {err}");
            }
            Err(std::process::ExitCode::from(err.exit_code()))
        }
    }
}
