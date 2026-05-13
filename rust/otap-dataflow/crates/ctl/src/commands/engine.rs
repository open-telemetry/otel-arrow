// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine-scoped command runner.
//!
//! This module implements commands that inspect the engine as a whole rather
//! than a group or pipeline. It translates `engine status`, `engine livez`, and
//! `engine readyz` into admin SDK calls and emits the same engine-level state in
//! human tables or machine-readable formats.

use crate::args::{EngineArgs, EngineCommand};
use crate::commands::output::write_read_command_output;
use crate::error::CliError;
use crate::render::{render_engine_probe, render_engine_status};
use crate::style::HumanStyle;
use otap_df_admin_api::AdminClient;
use std::io::Write;

/// Execute engine-scoped commands.
pub(crate) async fn run(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    args: EngineArgs,
) -> Result<(), CliError> {
    match args.command {
        EngineCommand::Status(output) => {
            let status = client.engine().status().await?;
            write_read_command_output(stdout, output.output, &status, || {
                Ok(render_engine_status(&human_style, &status))
            })
        }
        EngineCommand::Livez(output) => {
            let probe = client.engine().livez().await?;
            write_read_command_output(stdout, output.output, &probe, || {
                Ok(render_engine_probe(&human_style, &probe))
            })
        }
        EngineCommand::Readyz(output) => {
            let probe = client.engine().readyz().await?;
            write_read_command_output(stdout, output.output, &probe, || {
                Ok(render_engine_probe(&human_style, &probe))
            })
        }
    }
}
