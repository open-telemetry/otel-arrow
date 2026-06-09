// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry-scoped command runner.
//!
//! This module implements retained-log and metrics commands exposed by the
//! admin API. It applies client-side filters, selects compact or full metric
//! shapes, and delegates long-running watch behavior so telemetry output stays
//! usable both interactively and in shell pipelines.

use crate::args::{LogsCommand, MetricsCommand, MetricsShape, TelemetryArgs, TelemetryCommand};
use crate::commands::fetch::fetch_logs;
use crate::commands::filters::{log_filters_from_args, metrics_filters_from_args};
use crate::commands::output::write_read_command_output;
use crate::commands::watch::{watch_logs, watch_metrics};
use crate::error::CliError;
use crate::render::{render_logs, render_metrics_compact, render_metrics_full};
use crate::style::HumanStyle;
use crate::troubleshoot::{filter_logs, filter_metrics_compact, filter_metrics_full};
use otap_df_admin_api::AdminClient;
use otap_df_admin_api::telemetry::MetricsOptions;
use std::io::Write;

/// Execute telemetry-scoped commands.
pub(crate) async fn run(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    args: TelemetryArgs,
) -> Result<(), CliError> {
    match args.command {
        TelemetryCommand::Logs(args) => match args.command {
            LogsCommand::Get(args) => {
                let logs = filter_logs(
                    &fetch_logs(client, args.after, args.limit).await?,
                    &log_filters_from_args(args.filters),
                );
                write_read_command_output(stdout, args.output.output, &logs, || {
                    Ok(render_logs(&human_style, &logs))
                })
            }
            LogsCommand::Watch(args) => {
                watch_logs(
                    client,
                    stdout,
                    human_style,
                    args.after,
                    args.tail,
                    args.limit,
                    log_filters_from_args(args.filters),
                    args.interval,
                    args.output.output,
                )
                .await
            }
        },
        TelemetryCommand::Metrics(args) => match args.command {
            MetricsCommand::Get(args) => {
                let options = MetricsOptions {
                    reset: args.reset,
                    keep_all_zeroes: args.keep_all_zeroes,
                };
                match args.shape {
                    MetricsShape::Compact => {
                        let metrics = filter_metrics_compact(
                            &client.telemetry().metrics_compact(&options).await?,
                            &metrics_filters_from_args(args.filters),
                        );
                        write_read_command_output(stdout, args.output.output, &metrics, || {
                            Ok(render_metrics_compact(&human_style, &metrics))
                        })
                    }
                    MetricsShape::Full => {
                        let metrics = filter_metrics_full(
                            &client.telemetry().metrics(&options).await?,
                            &metrics_filters_from_args(args.filters),
                        );
                        write_read_command_output(stdout, args.output.output, &metrics, || {
                            Ok(render_metrics_full(&human_style, &metrics))
                        })
                    }
                }
            }
            MetricsCommand::Watch(args) => {
                watch_metrics(
                    client,
                    stdout,
                    human_style,
                    args.shape,
                    MetricsOptions {
                        reset: args.reset,
                        keep_all_zeroes: args.keep_all_zeroes,
                    },
                    metrics_filters_from_args(args.filters),
                    args.interval,
                    args.output.output,
                )
                .await
            }
        },
    }
}
