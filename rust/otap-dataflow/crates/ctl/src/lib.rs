// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `dfctl` library entrypoints for the OTAP Dataflow Engine admin CLI.
//!
//! This crate keeps the command implementation behind a small public surface so
//! tests, the binary wrapper, and future embedders can execute the same parsed
//! command tree. The module root owns cross-command wiring such as crypto setup,
//! connection resolution, terminal-aware styling, diagnostics routing, and the
//! shared installed binary name used by help text, generated metadata, and TUI
//! command hints.

mod args;
mod commands;
mod config;
mod crypto;
mod error;
mod pipeline_config_io;
mod render;
mod style;
mod troubleshoot;
mod ui;

pub use args::{Cli, ErrorFormat};

/// Installed command name used in help, completions, generated command
/// metadata, diagnostics, and TUI command hints.
pub const BIN_NAME: &str = "dfctl";

pub(crate) use commands::fetch::{fetch_logs, fetch_pipeline_describe};
pub(crate) use commands::output::build_bundle_metadata;
pub(crate) use pipeline_config_io::{
    parse_pipeline_config_content, serialize_pipeline_config_yaml,
};

use crate::args::{Cli as ParsedCli, Command};
use crate::config::{ResolvedConnection, resolve_connection};
use crate::crypto::ensure_crypto_provider;
use crate::error::CliError;
use crate::style::HumanStyle;
use std::io::{self, Write};

/// Executes a parsed `dfctl` command and writes command output to `stdout`.
pub async fn run(cli: ParsedCli, stdout: &mut dyn Write) -> Result<(), CliError> {
    run_with_terminal(cli, stdout, false).await
}

/// Executes a parsed `dfctl` command and styles human output based on terminal detection.
pub async fn run_with_terminal(
    cli: ParsedCli,
    stdout: &mut dyn Write,
    stdout_is_terminal: bool,
) -> Result<(), CliError> {
    let mut stderr = io::sink();
    run_with_terminal_and_diagnostics(cli, stdout, stdout_is_terminal, &mut stderr).await
}

/// Executes a parsed command and writes requested diagnostics to `stderr`.
pub async fn run_with_terminal_and_diagnostics(
    cli: ParsedCli,
    stdout: &mut dyn Write,
    stdout_is_terminal: bool,
    stderr: &mut dyn Write,
) -> Result<(), CliError> {
    let ParsedCli {
        agent: _,
        connection,
        verbose,
        quiet,
        error_format: _,
        color,
        command,
    } = cli;
    let command = match command {
        Command::Completions(args) => {
            commands::completions::run(stdout, args)?;
            return Ok(());
        }
        Command::Commands(args) => {
            let human_style = HumanStyle::resolve(color, stdout_is_terminal);
            commands::catalog::run(stdout, human_style, args)?;
            return Ok(());
        }
        Command::Schemas(args) => {
            let human_style = HumanStyle::resolve(color, stdout_is_terminal);
            commands::schemas::run(stdout, human_style, args)?;
            return Ok(());
        }
        other => other,
    };

    if matches!(command, Command::Ui(_)) && !stdout_is_terminal {
        return Err(CliError::invalid_usage(format!(
            "`{BIN_NAME} ui` requires an interactive terminal"
        )));
    }

    let human_style = HumanStyle::resolve(color, stdout_is_terminal);
    let resolved = resolve_connection(&connection)?;
    write_diagnostics(stderr, verbose, quiet, &resolved)?;
    let command = match command {
        Command::Config(args) => {
            return commands::config::run(stdout, human_style, args, &resolved);
        }
        other => other,
    };
    let ui_command_context = match &command {
        Command::Ui(args) => Some(ui::build_command_context(&resolved.settings, color, args)),
        _ => None,
    };

    ensure_crypto_provider()?;
    let client = otap_df_admin_api::AdminClient::builder()
        .http(resolved.settings)
        .build()?;

    match command {
        Command::Completions(_) => unreachable!("completions returned before client creation"),
        Command::Commands(_) => unreachable!("commands returned before client creation"),
        Command::Schemas(_) => unreachable!("schemas returned before client creation"),
        Command::Config(_) => unreachable!("config commands returned before client creation"),
        Command::Ui(args) => {
            ui::run_ui(
                &client,
                args,
                color,
                ui_command_context.expect("ui command context should be present"),
            )
            .await
        }
        Command::Engine(args) => commands::engine::run(&client, stdout, human_style, args).await,
        Command::Groups(args) => commands::groups::run(&client, stdout, human_style, args).await,
        Command::Pipelines(args) => {
            commands::pipelines::run(&client, stdout, human_style, args).await
        }
        Command::Telemetry(args) => {
            commands::telemetry::run(&client, stdout, human_style, args).await
        }
    }
}

fn write_diagnostics(
    stderr: &mut dyn Write,
    verbose: u8,
    quiet: bool,
    resolved: &ResolvedConnection,
) -> Result<(), CliError> {
    if quiet || verbose == 0 {
        return Ok(());
    }

    writeln!(stderr, "{BIN_NAME}: target={}", resolved.display_url())?;
    if verbose > 1 {
        let settings = &resolved.settings;
        let request_timeout = settings
            .timeout
            .map(|timeout| humantime::format_duration(timeout).to_string())
            .unwrap_or_else(|| "none".to_string());
        let tcp_keepalive = settings
            .tcp_keepalive
            .map(|timeout| humantime::format_duration(timeout).to_string())
            .unwrap_or_else(|| "none".to_string());
        writeln!(
            stderr,
            "{BIN_NAME}: connect_timeout={} request_timeout={} tcp_nodelay={} tcp_keepalive={}",
            humantime::format_duration(settings.connect_timeout),
            request_timeout,
            settings.tcp_nodelay,
            tcp_keepalive
        )?;
    }
    Ok(())
}
