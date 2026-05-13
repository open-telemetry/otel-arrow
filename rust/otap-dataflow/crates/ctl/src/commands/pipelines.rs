// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-scoped command runner.
//!
//! This module implements commands that inspect or mutate one pipeline within a
//! pipeline group. It coordinates admin SDK calls, local config loading,
//! diagnosis helpers, support bundles, rollout and shutdown watches, and
//! consistent human or machine-readable output for pipeline operations.

use crate::args::{
    MetricsShape, PipelineDiagnoseCommand, PipelineEventsCommand, PipelinesArgs, PipelinesCommand,
    RolloutCommand, ShutdownCommand,
};
use crate::commands::fetch::{
    fetch_logs, fetch_pipeline_describe, fetch_pipeline_status, fetch_rollout, fetch_shutdown,
};
use crate::commands::filters::{pipeline_event_filters, pipeline_scope_filters};
use crate::commands::output::{
    build_bundle_metadata, build_operation_options, duration_to_admin_timeout_secs,
    mutation_output_to_stream_output, validate_mutation_output_mode, write_mutation_command_output,
    write_read_command_output, write_support_bundle_output,
};
use crate::commands::watch::{watch_pipeline_events, watch_rollout, watch_shutdown};
use crate::error::CliError;
use crate::pipeline_config_io::load_pipeline_config;
use crate::render::{
    render_diagnosis, render_pipeline_describe, render_pipeline_details, render_pipeline_probe,
    render_pipeline_status, render_rollout_status, render_shutdown_status,
};
use crate::style::HumanStyle;
use crate::troubleshoot::{
    BundleMetrics, PipelineBundle, diagnose_pipeline_rollout, diagnose_pipeline_shutdown,
    extract_events_from_pipeline_status, filter_logs, filter_metrics_compact, filter_metrics_full,
    tail_events,
};
use otap_df_admin_api::AdminClient;
use otap_df_admin_api::telemetry::MetricsOptions;
use serde::Serialize;
use std::io::Write;

/// Execute pipeline-scoped commands.
pub(crate) async fn run(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    args: PipelinesArgs,
) -> Result<(), CliError> {
    match args.command {
        PipelinesCommand::Get(args) => {
            let details = client
                .pipelines()
                .details(&args.target.pipeline_group_id, &args.target.pipeline_id)
                .await?;
            let Some(details) = details else {
                return Err(CliError::not_found(format!(
                    "pipeline '{}/{}' was not found",
                    args.target.pipeline_group_id, args.target.pipeline_id
                )));
            };
            write_read_command_output(stdout, args.output.output, &details, || {
                render_pipeline_details(&human_style, &details)
            })
        }
        PipelinesCommand::Describe(args) => {
            let report = fetch_pipeline_describe(
                client,
                &args.target.pipeline_group_id,
                &args.target.pipeline_id,
            )
            .await?;
            write_read_command_output(stdout, args.output.output, &report, || {
                Ok(render_pipeline_describe(&human_style, &report))
            })
        }
        PipelinesCommand::Events(args) => match args.command {
            PipelineEventsCommand::Get(args) => {
                let status = fetch_pipeline_status(
                    client,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                )
                .await?;
                let filters = pipeline_event_filters(
                    args.filters.kinds,
                    args.filters.node_id,
                    args.filters.contains,
                );
                let events = tail_events(
                    extract_events_from_pipeline_status(
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                        &status,
                        Some(&filters),
                    ),
                    args.filters.tail,
                );
                write_read_command_output(stdout, args.output.output, &events, || {
                    Ok(crate::render::render_events(&human_style, &events))
                })
            }
            PipelineEventsCommand::Watch(args) => {
                let filters = pipeline_event_filters(
                    args.filters.kinds,
                    args.filters.node_id,
                    args.filters.contains,
                );
                watch_pipeline_events(
                    client,
                    stdout,
                    human_style,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    filters,
                    args.filters.tail,
                    args.interval,
                    args.output.output,
                )
                .await
            }
        },
        PipelinesCommand::Diagnose(args) => match args.command {
            PipelineDiagnoseCommand::Rollout(args) => {
                let describe = fetch_pipeline_describe(
                    client,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                )
                .await?;
                let filters = pipeline_scope_filters(
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                );
                let logs = filter_logs(
                    &fetch_logs(client, None, Some(args.logs_limit)).await?,
                    &filters.0,
                );
                let metrics = filter_metrics_compact(
                    &client
                        .telemetry()
                        .metrics_compact(&MetricsOptions::default())
                        .await?,
                    &filters.1,
                );
                let rollout_status = if let Some(rollout_id) = &args.rollout_id {
                    Some(
                        fetch_rollout(
                            client,
                            &args.target.pipeline_group_id,
                            &args.target.pipeline_id,
                            rollout_id,
                        )
                        .await?,
                    )
                } else {
                    None
                };
                let report =
                    diagnose_pipeline_rollout(&describe, rollout_status.as_ref(), &logs, &metrics);
                write_read_command_output(stdout, args.output.output, &report, || {
                    Ok(render_diagnosis(&human_style, &report))
                })
            }
            PipelineDiagnoseCommand::Shutdown(args) => {
                let describe = fetch_pipeline_describe(
                    client,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                )
                .await?;
                let filters = pipeline_scope_filters(
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                );
                let logs = filter_logs(
                    &fetch_logs(client, None, Some(args.logs_limit)).await?,
                    &filters.0,
                );
                let metrics = filter_metrics_compact(
                    &client
                        .telemetry()
                        .metrics_compact(&MetricsOptions::default())
                        .await?,
                    &filters.1,
                );
                let shutdown_status = if let Some(shutdown_id) = &args.shutdown_id {
                    Some(
                        fetch_shutdown(
                            client,
                            &args.target.pipeline_group_id,
                            &args.target.pipeline_id,
                            shutdown_id,
                        )
                        .await?,
                    )
                } else {
                    None
                };
                let report = diagnose_pipeline_shutdown(
                    &describe,
                    shutdown_status.as_ref(),
                    &logs,
                    &metrics,
                );
                write_read_command_output(stdout, args.output.output, &report, || {
                    Ok(render_diagnosis(&human_style, &report))
                })
            }
        },
        PipelinesCommand::Bundle(args) => {
            let describe = fetch_pipeline_describe(
                client,
                &args.target.pipeline_group_id,
                &args.target.pipeline_id,
            )
            .await?;
            let filters =
                pipeline_scope_filters(&args.target.pipeline_group_id, &args.target.pipeline_id);
            let logs = filter_logs(
                &fetch_logs(client, None, Some(args.logs_limit)).await?,
                &filters.0,
            );
            let rollout_status = if let Some(rollout_id) = &args.rollout_id {
                Some(
                    fetch_rollout(
                        client,
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                        rollout_id,
                    )
                    .await?,
                )
            } else {
                None
            };
            let shutdown_status = if let Some(shutdown_id) = &args.shutdown_id {
                Some(
                    fetch_shutdown(
                        client,
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                        shutdown_id,
                    )
                    .await?,
                )
            } else {
                None
            };
            let (metrics, diagnosis) = match args.metrics_shape {
                MetricsShape::Compact => {
                    let metrics = filter_metrics_compact(
                        &client
                            .telemetry()
                            .metrics_compact(&MetricsOptions::default())
                            .await?,
                        &filters.1,
                    );
                    let diagnosis = if let Some(status) = shutdown_status.as_ref() {
                        diagnose_pipeline_shutdown(&describe, Some(status), &logs, &metrics)
                    } else {
                        diagnose_pipeline_rollout(
                            &describe,
                            rollout_status.as_ref(),
                            &logs,
                            &metrics,
                        )
                    };
                    (BundleMetrics::Compact(metrics), diagnosis)
                }
                MetricsShape::Full => {
                    let compact_metrics = filter_metrics_compact(
                        &client
                            .telemetry()
                            .metrics_compact(&MetricsOptions::default())
                            .await?,
                        &filters.1,
                    );
                    let diagnosis = if let Some(status) = shutdown_status.as_ref() {
                        diagnose_pipeline_shutdown(&describe, Some(status), &logs, &compact_metrics)
                    } else {
                        diagnose_pipeline_rollout(
                            &describe,
                            rollout_status.as_ref(),
                            &logs,
                            &compact_metrics,
                        )
                    };
                    let metrics = filter_metrics_full(
                        &client
                            .telemetry()
                            .metrics(&MetricsOptions::default())
                            .await?,
                        &filters.1,
                    );
                    (BundleMetrics::Full(metrics), diagnosis)
                }
            };
            let bundle = PipelineBundle {
                metadata: build_bundle_metadata(args.logs_limit, args.metrics_shape),
                describe,
                diagnosis,
                rollout_status,
                shutdown_status,
                logs,
                metrics,
            };
            write_support_bundle_output(stdout, args.output, args.file.as_deref(), &bundle)
        }
        PipelinesCommand::Status(args) => {
            let status = client
                .pipelines()
                .status(&args.target.pipeline_group_id, &args.target.pipeline_id)
                .await?;
            let Some(status) = status else {
                return Err(CliError::not_found(format!(
                    "pipeline '{}/{}' was not found",
                    args.target.pipeline_group_id, args.target.pipeline_id
                )));
            };
            write_read_command_output(stdout, args.output.output, &status, || {
                Ok(render_pipeline_status(&human_style, &status))
            })
        }
        PipelinesCommand::Livez(args) => {
            let probe = client
                .pipelines()
                .livez(&args.target.pipeline_group_id, &args.target.pipeline_id)
                .await?;
            write_read_command_output(stdout, args.output.output, &probe, || {
                Ok(render_pipeline_probe(&human_style, &probe))
            })
        }
        PipelinesCommand::Readyz(args) => {
            let probe = client
                .pipelines()
                .readyz(&args.target.pipeline_group_id, &args.target.pipeline_id)
                .await?;
            write_read_command_output(stdout, args.output.output, &probe, || {
                Ok(render_pipeline_probe(&human_style, &probe))
            })
        }
        PipelinesCommand::Reconfigure(args) => {
            validate_mutation_output_mode(args.output, args.watch)?;
            let pipeline = load_pipeline_config(
                &args.file,
                &args.target.pipeline_group_id,
                &args.target.pipeline_id,
            )?;
            let request = otap_df_admin_api::pipelines::ReconfigureRequest {
                pipeline,
                step_timeout_secs: duration_to_admin_timeout_secs(args.step_timeout),
                drain_timeout_secs: duration_to_admin_timeout_secs(args.drain_timeout),
            };
            if args.dry_run {
                let report = PipelineReconfigurePreflight {
                    mode: "preflight-only",
                    operation: "pipelines.reconfigure",
                    server_validation: false,
                    target: PipelineTargetReport {
                        pipeline_group_id: args.target.pipeline_group_id.clone(),
                        pipeline_id: args.target.pipeline_id.clone(),
                    },
                    config_source: args.file.display().to_string(),
                    step_timeout_secs: request.step_timeout_secs,
                    drain_timeout_secs: request.drain_timeout_secs,
                    wait: args.wait,
                    wait_timeout_secs: duration_to_admin_timeout_secs(args.wait_timeout),
                };
                return write_mutation_command_output(
                    stdout,
                    args.output,
                    "preflight_only",
                    &report,
                    || Ok(render_pipeline_reconfigure_preflight(&human_style, &report)),
                );
            }
            let outcome = client
                .pipelines()
                .reconfigure(
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    &request,
                    &build_operation_options(args.wait, args.wait_timeout),
                )
                .await?;

            match outcome {
                otap_df_admin_api::pipelines::ReconfigureOutcome::Accepted(status) => {
                    write_mutation_command_output(
                        stdout,
                        args.output,
                        "accepted",
                        &status,
                        || Ok(render_rollout_status(&human_style, &status)),
                    )?;
                    if args.watch {
                        let rollout_id = status.rollout_id.clone();
                        watch_rollout(
                            client,
                            stdout,
                            human_style,
                            &args.target.pipeline_group_id,
                            &args.target.pipeline_id,
                            &rollout_id,
                            args.watch_interval,
                            mutation_output_to_stream_output(args.output)?,
                            Some(status),
                        )
                        .await
                    } else {
                        Ok(())
                    }
                }
                otap_df_admin_api::pipelines::ReconfigureOutcome::Completed(status) => {
                    write_mutation_command_output(stdout, args.output, "completed", &status, || {
                        Ok(render_rollout_status(&human_style, &status))
                    })
                }
                otap_df_admin_api::pipelines::ReconfigureOutcome::Failed(status) => {
                    write_mutation_command_output(stdout, args.output, "failed", &status, || {
                        Ok(render_rollout_status(&human_style, &status))
                    })?;
                    Err(CliError::outcome_failure(format!(
                        "pipeline rollout '{}' failed",
                        status.rollout_id
                    )))
                }
                otap_df_admin_api::pipelines::ReconfigureOutcome::TimedOut(status) => {
                    write_mutation_command_output(
                        stdout,
                        args.output,
                        "timed_out",
                        &status,
                        || Ok(render_rollout_status(&human_style, &status)),
                    )?;
                    Err(CliError::outcome_failure(format!(
                        "pipeline rollout '{}' timed out",
                        status.rollout_id
                    )))
                }
            }
        }
        PipelinesCommand::Shutdown(args) => {
            validate_mutation_output_mode(args.output, args.watch)?;
            if args.dry_run {
                let report = PipelineShutdownPreflight {
                    mode: "preflight-only",
                    operation: "pipelines.shutdown",
                    server_validation: false,
                    target: PipelineTargetReport {
                        pipeline_group_id: args.target.pipeline_group_id.clone(),
                        pipeline_id: args.target.pipeline_id.clone(),
                    },
                    wait: args.wait,
                    wait_timeout_secs: duration_to_admin_timeout_secs(args.wait_timeout),
                };
                return write_mutation_command_output(
                    stdout,
                    args.output,
                    "preflight_only",
                    &report,
                    || Ok(render_pipeline_shutdown_preflight(&human_style, &report)),
                );
            }
            let outcome = client
                .pipelines()
                .shutdown(
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    &build_operation_options(args.wait, args.wait_timeout),
                )
                .await?;

            match outcome {
                otap_df_admin_api::pipelines::ShutdownOutcome::Accepted(status) => {
                    write_mutation_command_output(
                        stdout,
                        args.output,
                        "accepted",
                        &status,
                        || Ok(render_shutdown_status(&human_style, &status)),
                    )?;
                    if args.watch {
                        let shutdown_id = status.shutdown_id.clone();
                        watch_shutdown(
                            client,
                            stdout,
                            human_style,
                            &args.target.pipeline_group_id,
                            &args.target.pipeline_id,
                            &shutdown_id,
                            args.watch_interval,
                            mutation_output_to_stream_output(args.output)?,
                            Some(status),
                        )
                        .await
                    } else {
                        Ok(())
                    }
                }
                otap_df_admin_api::pipelines::ShutdownOutcome::Completed(status) => {
                    write_mutation_command_output(stdout, args.output, "completed", &status, || {
                        Ok(render_shutdown_status(&human_style, &status))
                    })
                }
                otap_df_admin_api::pipelines::ShutdownOutcome::Failed(status) => {
                    write_mutation_command_output(stdout, args.output, "failed", &status, || {
                        Ok(render_shutdown_status(&human_style, &status))
                    })?;
                    Err(CliError::outcome_failure(format!(
                        "pipeline shutdown '{}' failed",
                        status.shutdown_id
                    )))
                }
                otap_df_admin_api::pipelines::ShutdownOutcome::TimedOut(status) => {
                    write_mutation_command_output(
                        stdout,
                        args.output,
                        "timed_out",
                        &status,
                        || Ok(render_shutdown_status(&human_style, &status)),
                    )?;
                    Err(CliError::outcome_failure(format!(
                        "pipeline shutdown '{}' timed out",
                        status.shutdown_id
                    )))
                }
            }
        }
        PipelinesCommand::Rollouts(args) => match args.command {
            RolloutCommand::Get(args) => {
                let status = fetch_rollout(
                    client,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    &args.target.rollout_id,
                )
                .await?;
                write_read_command_output(stdout, args.output.output, &status, || {
                    Ok(render_rollout_status(&human_style, &status))
                })
            }
            RolloutCommand::Watch(args) => {
                watch_rollout(
                    client,
                    stdout,
                    human_style,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    &args.target.rollout_id,
                    args.interval,
                    args.output.output,
                    None,
                )
                .await
            }
        },
        PipelinesCommand::RolloutStatus(args) => {
            let status = fetch_rollout(
                client,
                &args.target.pipeline_group_id,
                &args.target.pipeline_id,
                &args.target.rollout_id,
            )
            .await?;
            write_read_command_output(stdout, args.output.output, &status, || {
                Ok(render_rollout_status(&human_style, &status))
            })
        }
        PipelinesCommand::Shutdowns(args) => match args.command {
            ShutdownCommand::Get(args) => {
                let status = fetch_shutdown(
                    client,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    &args.target.shutdown_id,
                )
                .await?;
                write_read_command_output(stdout, args.output.output, &status, || {
                    Ok(render_shutdown_status(&human_style, &status))
                })
            }
            ShutdownCommand::Watch(args) => {
                watch_shutdown(
                    client,
                    stdout,
                    human_style,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    &args.target.shutdown_id,
                    args.interval,
                    args.output.output,
                    None,
                )
                .await
            }
        },
        PipelinesCommand::ShutdownStatus(args) => {
            let status = fetch_shutdown(
                client,
                &args.target.pipeline_group_id,
                &args.target.pipeline_id,
                &args.target.shutdown_id,
            )
            .await?;
            write_read_command_output(stdout, args.output.output, &status, || {
                Ok(render_shutdown_status(&human_style, &status))
            })
        }
    }
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PipelineTargetReport {
    pipeline_group_id: String,
    pipeline_id: String,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PipelineReconfigurePreflight {
    mode: &'static str,
    operation: &'static str,
    server_validation: bool,
    target: PipelineTargetReport,
    config_source: String,
    step_timeout_secs: u64,
    drain_timeout_secs: u64,
    wait: bool,
    wait_timeout_secs: u64,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct PipelineShutdownPreflight {
    mode: &'static str,
    operation: &'static str,
    server_validation: bool,
    target: PipelineTargetReport,
    wait: bool,
    wait_timeout_secs: u64,
}

fn render_pipeline_reconfigure_preflight(
    style: &HumanStyle,
    report: &PipelineReconfigurePreflight,
) -> String {
    [
        style.header("pipeline reconfigure dry-run"),
        format!("{}: {}", style.label("mode"), report.mode),
        format!("{}: {}", style.label("operation"), report.operation),
        format!(
            "{}: {}/{}",
            style.label("target"),
            report.target.pipeline_group_id,
            report.target.pipeline_id
        ),
        format!("{}: {}", style.label("config_source"), report.config_source),
        format!(
            "{}: {}",
            style.label("server_validation"),
            report.server_validation
        ),
        format!(
            "{}: {}",
            style.label("step_timeout_secs"),
            report.step_timeout_secs
        ),
        format!(
            "{}: {}",
            style.label("drain_timeout_secs"),
            report.drain_timeout_secs
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

fn render_pipeline_shutdown_preflight(
    style: &HumanStyle,
    report: &PipelineShutdownPreflight,
) -> String {
    [
        style.header("pipeline shutdown dry-run"),
        format!("{}: {}", style.label("mode"), report.mode),
        format!("{}: {}", style.label("operation"), report.operation),
        format!(
            "{}: {}/{}",
            style.label("target"),
            report.target.pipeline_group_id,
            report.target.pipeline_id
        ),
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
