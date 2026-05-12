// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Groups-scoped command runner.
//!
//! This module is the execution layer for commands that operate on the whole
//! engine group set rather than on one specific pipeline. It translates parsed
//! CLI arguments into admin SDK calls, troubleshooting helpers, watch streams,
//! support bundles, and human or machine-readable output.

use crate::args::{
    GroupDiagnoseCommand, GroupEventsCommand, GroupsArgs, GroupsCommand, MetricsShape,
};
use crate::commands::fetch::fetch_logs;
use crate::commands::filters::group_event_filters;
use crate::commands::output::{
    build_bundle_metadata, build_operation_options, duration_to_admin_timeout_secs,
    group_shutdown_output_to_stream_output, validate_group_shutdown_output_mode,
    write_group_shutdown_command_output, write_read_command_output, write_support_bundle_output,
};
use crate::commands::watch::{watch_group_events, watch_groups_shutdown};
use crate::error::CliError;
use crate::render::{
    render_diagnosis, render_groups_describe, render_groups_shutdown, render_groups_status,
};
use crate::style::HumanStyle;
use crate::troubleshoot::{
    BundleMetrics, GroupsBundle, LogFilters, MetricsFilters, describe_groups,
    diagnose_group_shutdown, extract_events_from_group_status, filter_logs, filter_metrics_compact,
    filter_metrics_full, tail_events,
};
use otap_df_admin_api::AdminClient;
use otap_df_admin_api::telemetry::MetricsOptions;
use serde::Serialize;
use std::io::Write;

/// Executes group-scoped commands against the admin API.
///
/// Read-only commands fetch the latest group status, telemetry, or event
/// snapshots and render them through the selected output mode. Mutating
/// commands validate CLI output constraints before calling the SDK so shell
/// scripts and agents receive predictable behavior for `--wait`, `--watch`,
/// `--dry-run`, and structured output combinations.
pub(crate) async fn run(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    args: GroupsArgs,
) -> Result<(), CliError> {
    match args.command {
        GroupsCommand::Describe(output) => {
            let status = client.groups().status().await?;
            let report = describe_groups(status);
            write_read_command_output(stdout, output.output, &report, || {
                Ok(render_groups_describe(&human_style, &report))
            })
        }
        GroupsCommand::Events(args) => match args.command {
            GroupEventsCommand::Get(args) => {
                let status = client.groups().status().await?;
                let filters = group_event_filters(
                    args.filters.kinds,
                    args.filters.pipeline_group_id,
                    args.filters.pipeline_id,
                    args.filters.node_id,
                    args.filters.contains,
                );
                let events = tail_events(
                    extract_events_from_group_status(&status, Some(&filters)),
                    args.filters.tail,
                );
                write_read_command_output(stdout, args.output.output, &events, || {
                    Ok(crate::render::render_events(&human_style, &events))
                })
            }
            GroupEventsCommand::Watch(args) => {
                let filters = group_event_filters(
                    args.filters.kinds,
                    args.filters.pipeline_group_id,
                    args.filters.pipeline_id,
                    args.filters.node_id,
                    args.filters.contains,
                );
                watch_group_events(
                    client,
                    stdout,
                    human_style,
                    filters,
                    args.filters.tail,
                    args.interval,
                    args.output.output,
                )
                .await
            }
        },
        GroupsCommand::Diagnose(args) => match args.command {
            GroupDiagnoseCommand::Shutdown(args) => {
                let status = client.groups().status().await?;
                let logs = filter_logs(
                    &fetch_logs(client, None, Some(args.logs_limit)).await?,
                    &LogFilters::default(),
                );
                let metrics = filter_metrics_compact(
                    &client
                        .telemetry()
                        .metrics_compact(&MetricsOptions::default())
                        .await?,
                    &MetricsFilters::default(),
                );
                let report = diagnose_group_shutdown(&status, &logs, &metrics);
                write_read_command_output(stdout, args.output.output, &report, || {
                    Ok(render_diagnosis(&human_style, &report))
                })
            }
        },
        GroupsCommand::Bundle(args) => {
            let status = client.groups().status().await?;
            let describe = describe_groups(status);
            let logs = filter_logs(
                &fetch_logs(client, None, Some(args.logs_limit)).await?,
                &LogFilters::default(),
            );
            let (metrics, diagnosis) = match args.metrics_shape {
                MetricsShape::Compact => {
                    let metrics = filter_metrics_compact(
                        &client
                            .telemetry()
                            .metrics_compact(&MetricsOptions::default())
                            .await?,
                        &MetricsFilters::default(),
                    );
                    let diagnosis = diagnose_group_shutdown(&describe.status, &logs, &metrics);
                    (BundleMetrics::Compact(metrics), diagnosis)
                }
                MetricsShape::Full => {
                    let compact_metrics = filter_metrics_compact(
                        &client
                            .telemetry()
                            .metrics_compact(&MetricsOptions::default())
                            .await?,
                        &MetricsFilters::default(),
                    );
                    let diagnosis =
                        diagnose_group_shutdown(&describe.status, &logs, &compact_metrics);
                    let metrics = filter_metrics_full(
                        &client
                            .telemetry()
                            .metrics(&MetricsOptions::default())
                            .await?,
                        &MetricsFilters::default(),
                    );
                    (BundleMetrics::Full(metrics), diagnosis)
                }
            };
            let bundle = GroupsBundle {
                metadata: build_bundle_metadata(args.logs_limit, args.metrics_shape),
                describe,
                diagnosis,
                logs,
                metrics,
            };
            write_support_bundle_output(stdout, args.output, args.file.as_deref(), &bundle)
        }
        GroupsCommand::Status(output) => {
            let status = client.groups().status().await?;
            write_read_command_output(stdout, output.output, &status, || {
                Ok(render_groups_status(&human_style, &status))
            })
        }
        GroupsCommand::Shutdown(args) => {
            validate_group_shutdown_output_mode(args.output, args.watch)?;
            if args.dry_run {
                let report = GroupShutdownPreflight {
                    mode: "preflight-only",
                    operation: "groups.shutdown",
                    server_validation: false,
                    wait: args.wait,
                    wait_timeout_secs: duration_to_admin_timeout_secs(args.wait_timeout),
                };
                return write_group_shutdown_command_output(stdout, args.output, &report, || {
                    Ok(render_group_shutdown_preflight(&human_style, &report))
                });
            }
            let response = client
                .groups()
                .shutdown(&build_operation_options(args.wait, args.wait_timeout))
                .await?;
            if args.watch {
                watch_groups_shutdown(
                    client,
                    stdout,
                    human_style,
                    response.status,
                    args.wait_timeout,
                    args.watch_interval,
                    group_shutdown_output_to_stream_output(args.output)?,
                )
                .await?;
                return Ok(());
            }
            write_group_shutdown_command_output(stdout, args.output, &response, || {
                Ok(render_groups_shutdown(&human_style, &response))
            })?;
            match response.status {
                otap_df_admin_api::groups::ShutdownStatus::Accepted
                | otap_df_admin_api::groups::ShutdownStatus::Completed => Ok(()),
                otap_df_admin_api::groups::ShutdownStatus::Failed
                | otap_df_admin_api::groups::ShutdownStatus::Timeout => {
                    Err(CliError::outcome_failure(format!(
                        "groups shutdown ended with status {:?}",
                        response.status
                    )))
                }
            }
        }
    }
}

/// Machine-readable result emitted by `groups shutdown --dry-run`.
///
/// The admin SDK currently has no server-side preflight endpoint for a full
/// group shutdown, so this payload documents exactly what the CLI validated
/// locally and which options would be sent by a real shutdown request.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct GroupShutdownPreflight {
    /// Indicates that no admin mutation was sent.
    mode: &'static str,
    /// Stable operation identifier for scripts and agents.
    operation: &'static str,
    /// Whether the request was validated by the server.
    server_validation: bool,
    /// Whether the real request would wait for completion.
    wait: bool,
    /// Effective wait timeout rounded up to full seconds for JSON output.
    wait_timeout_secs: u64,
}

/// Renders the dry-run group shutdown report for human output.
fn render_group_shutdown_preflight(style: &HumanStyle, report: &GroupShutdownPreflight) -> String {
    [
        style.header("group shutdown dry-run"),
        format!("{}: {}", style.label("mode"), report.mode),
        format!("{}: {}", style.label("operation"), report.operation),
        format!(
            "{}: {}",
            style.label("server_validation"),
            report.server_validation
        ),
        format!("{}: {}", style.label("wait"), report.wait),
        format!(
            "{}: {}",
            style.label("wait_timeout_secs"),
            report.wait_timeout_secs
        ),
    ]
    .join("\n")
}
