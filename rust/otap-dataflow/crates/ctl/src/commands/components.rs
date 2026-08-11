// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component-inventory command runner (RFC 0001).
//!
//! Implements `dfctl components`, which reports the running engine's component
//! inventory (`GET /api/v1/components`) as a human table or a machine-readable
//! format, using the shared read-output plumbing.

use crate::args::ComponentsArgs;
use crate::commands::output::write_read_command_output;
use crate::error::CliError;
use crate::render::render_components;
use crate::style::HumanStyle;
use otap_df_admin_api::AdminClient;
use std::io::Write;

/// Execute the `components` command.
pub(crate) async fn run(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    args: ComponentsArgs,
) -> Result<(), CliError> {
    let response = client.components().list().await?;
    write_read_command_output(stdout, args.output.output, &response, || {
        Ok(render_components(&human_style, &response))
    })
}
