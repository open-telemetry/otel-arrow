// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `dfctl` library entrypoints for the OTAP Dataflow Engine admin CLI.

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

pub use args::Cli;

pub(crate) use commands::fetch::{fetch_logs, fetch_pipeline_describe};
pub(crate) use commands::output::bundle_metadata;
pub(crate) use pipeline_config_io::{
    parse_pipeline_config_content, serialize_pipeline_config_yaml,
};

use crate::args::{Cli as ParsedCli, Command};
use crate::config::resolve_connection;
use crate::crypto::ensure_crypto_provider;
use crate::error::CliError;
use crate::style::HumanStyle;
use std::io::Write;

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
    ensure_crypto_provider()?;
    let ParsedCli {
        connection,
        color,
        command,
    } = cli;
    if matches!(command, Command::Ui(_)) && !stdout_is_terminal {
        return Err(CliError::invalid_usage(
            "`dfctl ui` requires an interactive terminal",
        ));
    }

    let human_style = HumanStyle::resolve(color, stdout_is_terminal);
    let resolved = resolve_connection(&connection)?;
    let ui_command_context = match &command {
        Command::Ui(args) => Some(ui::build_command_context(&resolved.settings, color, args)),
        _ => None,
    };
    let client = otap_df_admin_api::AdminClient::builder()
        .http(resolved.settings)
        .build()?;

    match command {
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
