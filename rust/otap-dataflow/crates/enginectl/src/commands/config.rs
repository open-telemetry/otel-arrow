// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Commands that inspect client-side `dfctl` configuration.

use crate::args::{ConfigArgs, ConfigCommand};
use crate::commands::output::emit_read;
use crate::config::{ResolvedConnection, ResolvedConnectionView};
use crate::error::CliError;
use crate::style::HumanStyle;
use std::io::Write;

pub(crate) fn run(
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    args: ConfigArgs,
    resolved: &ResolvedConnection,
) -> Result<(), CliError> {
    match args.command {
        ConfigCommand::View(output) => {
            let view = resolved.view();
            emit_read(stdout, output.output, &view, || {
                Ok(render_config_view(&human_style, &view))
            })
        }
    }
}

fn render_config_view(style: &HumanStyle, view: &ResolvedConnectionView) -> String {
    let mut lines = Vec::new();
    lines.push(style.header("dfctl configuration"));
    lines.push(format!("{}: {}", style.label("target"), view.url));
    lines.push(format!(
        "{}: {}",
        style.label("profile_file"),
        view.profile_file.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "{}: {}",
        style.label("connect_timeout"),
        view.connect_timeout
    ));
    lines.push(format!(
        "{}: {}",
        style.label("request_timeout"),
        view.request_timeout.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "{}: {}",
        style.label("tcp_nodelay"),
        view.tcp_nodelay
    ));
    lines.push(format!(
        "{}: {}",
        style.label("tcp_keepalive"),
        view.tcp_keepalive.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "{}: {}",
        style.label("tcp_keepalive_interval"),
        view.tcp_keepalive_interval.as_deref().unwrap_or("none")
    ));
    lines.push(format!(
        "{}: {}",
        style.label("tls"),
        if view.tls.enabled {
            "enabled"
        } else {
            "disabled"
        }
    ));
    if view.tls.enabled {
        lines.push(format!(
            "{}: {}",
            style.label("tls.ca_file"),
            view.tls.ca_file.as_deref().unwrap_or("none")
        ));
        lines.push(format!(
            "{}: {}",
            style.label("tls.client_cert_file"),
            view.tls.client_cert_file.as_deref().unwrap_or("none")
        ));
        lines.push(format!(
            "{}: {}",
            style.label("tls.client_key_file"),
            if view.tls.client_key_file_configured {
                "configured"
            } else {
                "none"
            }
        ));
        lines.push(format!(
            "{}: {}",
            style.label("tls.include_system_ca_certs"),
            format_optional_bool(view.tls.include_system_ca_certs)
        ));
        lines.push(format!(
            "{}: {}",
            style.label("tls.insecure_skip_verify"),
            format_optional_bool(view.tls.insecure_skip_verify)
        ));
    }
    lines.join("\n")
}

fn format_optional_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unset",
    }
}
