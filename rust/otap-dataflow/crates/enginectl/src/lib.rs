// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `dfctl` library entrypoints for the OTAP Dataflow Engine admin CLI.

mod args;
mod config;
mod error;
mod render;
mod style;
mod troubleshoot;
mod ui;

pub use args::Cli;

use crate::args::{
    BundleOutput, Cli as ParsedCli, Command, EngineCommand, EventKind, GroupDiagnoseCommand,
    GroupEventsCommand, GroupShutdownOutput, GroupsCommand, LogsCommand, LogsFilterArgs,
    MetricsCommand, MetricsFilterArgs, MetricsShape, MutationOutput, PipelineDiagnoseCommand,
    PipelineEventsCommand, PipelinesCommand, ReadOutput, RolloutCommand, ShutdownCommand,
    StreamOutput, TelemetryCommand,
};
use crate::config::resolve_connection;
use crate::error::CliError;
use crate::render::{
    render_diagnosis, render_engine_probe, render_engine_status, render_event_line, render_events,
    render_group_shutdown_watch, render_groups_describe, render_groups_shutdown,
    render_groups_status, render_logs, render_metrics_compact, render_metrics_full,
    render_pipeline_describe, render_pipeline_details, render_pipeline_probe,
    render_pipeline_status, render_rollout_status, render_shutdown_status, write_bundle_output,
    write_event_output, write_human, write_log_event, write_mutation_output, write_read_output,
    write_stream_snapshot,
};
use crate::style::HumanStyle;
use crate::troubleshoot::{
    BundleMetadata, BundleMetrics, BundleMetricsShape, EventFilters, GroupsBundle, LogFilters,
    MetricsFilters, NormalizedEvent, NormalizedEventKind, PipelineBundle, PipelineDescribeReport,
    describe_groups, describe_pipeline, diagnose_group_shutdown, diagnose_pipeline_rollout,
    diagnose_pipeline_shutdown, extract_events_from_group_status,
    extract_events_from_pipeline_status, filter_logs, filter_metrics_compact, filter_metrics_full,
    group_shutdown_snapshot, tail_events,
};
use otap_df_admin_api::config::pipeline::PipelineConfig;
use otap_df_admin_api::operations::OperationOptions;
use otap_df_admin_api::telemetry::{LogsQuery, MetricsOptions};
use otap_df_admin_api::{AdminClient, telemetry};
use serde::Serialize;
use std::collections::BTreeSet;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::OnceLock;
use std::time::{Duration, SystemTime};

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
    let client = AdminClient::builder().http(resolved.settings).build()?;

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
        Command::Engine(args) => match args.command {
            EngineCommand::Status(output) => {
                let status = client.engine().status().await?;
                emit_read(stdout, output.output, &status, || {
                    Ok(render_engine_status(&human_style, &status))
                })
            }
            EngineCommand::Livez(output) => {
                let probe = client.engine().livez().await?;
                emit_read(stdout, output.output, &probe, || {
                    Ok(render_engine_probe(&human_style, &probe))
                })
            }
            EngineCommand::Readyz(output) => {
                let probe = client.engine().readyz().await?;
                emit_read(stdout, output.output, &probe, || {
                    Ok(render_engine_probe(&human_style, &probe))
                })
            }
        },
        Command::Groups(args) => match args.command {
            GroupsCommand::Describe(output) => {
                let status = client.groups().status().await?;
                let report = describe_groups(status);
                emit_read(stdout, output.output, &report, || {
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
                    emit_read(stdout, args.output.output, &events, || {
                        Ok(render_events(&human_style, &events))
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
                        &client,
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
                        &fetch_logs(&client, None, Some(args.logs_limit)).await?,
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
                    emit_read(stdout, args.output.output, &report, || {
                        Ok(render_diagnosis(&human_style, &report))
                    })
                }
            },
            GroupsCommand::Bundle(args) => {
                let status = client.groups().status().await?;
                let describe = describe_groups(status);
                let logs = filter_logs(
                    &fetch_logs(&client, None, Some(args.logs_limit)).await?,
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
                    metadata: bundle_metadata(args.logs_limit, args.metrics_shape),
                    describe,
                    diagnosis,
                    logs,
                    metrics,
                };
                write_bundle(stdout, args.output, args.file.as_deref(), &bundle)
            }
            GroupsCommand::Status(output) => {
                let status = client.groups().status().await?;
                emit_read(stdout, output.output, &status, || {
                    Ok(render_groups_status(&human_style, &status))
                })
            }
            GroupsCommand::Shutdown(args) => {
                validate_group_shutdown_output(args.output, args.watch)?;
                let response = client
                    .groups()
                    .shutdown(&operation_options(args.wait, args.wait_timeout))
                    .await?;
                if args.watch {
                    watch_groups_shutdown(
                        &client,
                        stdout,
                        human_style,
                        response.status,
                        args.wait_timeout,
                        args.watch_interval,
                        group_shutdown_stream_output(args.output)?,
                    )
                    .await?;
                    return Ok(());
                }
                emit_group_shutdown(stdout, args.output, &response, || {
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
        },
        Command::Pipelines(args) => match args.command {
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
                emit_read(stdout, args.output.output, &details, || {
                    render_pipeline_details(&human_style, &details)
                })
            }
            PipelinesCommand::Describe(args) => {
                let report = fetch_pipeline_describe(
                    &client,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                )
                .await?;
                emit_read(stdout, args.output.output, &report, || {
                    Ok(render_pipeline_describe(&human_style, &report))
                })
            }
            PipelinesCommand::Events(args) => match args.command {
                PipelineEventsCommand::Get(args) => {
                    let status = fetch_pipeline_status(
                        &client,
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
                    emit_read(stdout, args.output.output, &events, || {
                        Ok(render_events(&human_style, &events))
                    })
                }
                PipelineEventsCommand::Watch(args) => {
                    let filters = pipeline_event_filters(
                        args.filters.kinds,
                        args.filters.node_id,
                        args.filters.contains,
                    );
                    watch_pipeline_events(
                        &client,
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
                        &client,
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                    )
                    .await?;
                    let filters = pipeline_scope_filters(
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                    );
                    let logs = filter_logs(
                        &fetch_logs(&client, None, Some(args.logs_limit)).await?,
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
                                &client,
                                &args.target.pipeline_group_id,
                                &args.target.pipeline_id,
                                rollout_id,
                            )
                            .await?,
                        )
                    } else {
                        None
                    };
                    let report = diagnose_pipeline_rollout(
                        &describe,
                        rollout_status.as_ref(),
                        &logs,
                        &metrics,
                    );
                    emit_read(stdout, args.output.output, &report, || {
                        Ok(render_diagnosis(&human_style, &report))
                    })
                }
                PipelineDiagnoseCommand::Shutdown(args) => {
                    let describe = fetch_pipeline_describe(
                        &client,
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                    )
                    .await?;
                    let filters = pipeline_scope_filters(
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                    );
                    let logs = filter_logs(
                        &fetch_logs(&client, None, Some(args.logs_limit)).await?,
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
                                &client,
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
                    emit_read(stdout, args.output.output, &report, || {
                        Ok(render_diagnosis(&human_style, &report))
                    })
                }
            },
            PipelinesCommand::Bundle(args) => {
                let describe = fetch_pipeline_describe(
                    &client,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                )
                .await?;
                let filters = pipeline_scope_filters(
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                );
                let logs = filter_logs(
                    &fetch_logs(&client, None, Some(args.logs_limit)).await?,
                    &filters.0,
                );
                let rollout_status = if let Some(rollout_id) = &args.rollout_id {
                    Some(
                        fetch_rollout(
                            &client,
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
                            &client,
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
                            diagnose_pipeline_shutdown(
                                &describe,
                                Some(status),
                                &logs,
                                &compact_metrics,
                            )
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
                    metadata: bundle_metadata(args.logs_limit, args.metrics_shape),
                    describe,
                    diagnosis,
                    rollout_status,
                    shutdown_status,
                    logs,
                    metrics,
                };
                write_bundle(stdout, args.output, args.file.as_deref(), &bundle)
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
                emit_read(stdout, args.output.output, &status, || {
                    Ok(render_pipeline_status(&human_style, &status))
                })
            }
            PipelinesCommand::Livez(args) => {
                let probe = client
                    .pipelines()
                    .livez(&args.target.pipeline_group_id, &args.target.pipeline_id)
                    .await?;
                emit_read(stdout, args.output.output, &probe, || {
                    Ok(render_pipeline_probe(&human_style, &probe))
                })
            }
            PipelinesCommand::Readyz(args) => {
                let probe = client
                    .pipelines()
                    .readyz(&args.target.pipeline_group_id, &args.target.pipeline_id)
                    .await?;
                emit_read(stdout, args.output.output, &probe, || {
                    Ok(render_pipeline_probe(&human_style, &probe))
                })
            }
            PipelinesCommand::Reconfigure(args) => {
                validate_mutation_output(args.output, args.watch)?;
                let pipeline = load_pipeline_config(
                    &args.file,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                )?;
                let request = otap_df_admin_api::pipelines::ReconfigureRequest {
                    pipeline,
                    step_timeout_secs: duration_to_secs_ceil(args.step_timeout),
                    drain_timeout_secs: duration_to_secs_ceil(args.drain_timeout),
                };
                let outcome = client
                    .pipelines()
                    .reconfigure(
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                        &request,
                        &operation_options(args.wait, args.wait_timeout),
                    )
                    .await?;

                match outcome {
                    otap_df_admin_api::pipelines::ReconfigureOutcome::Accepted(status) => {
                        emit_mutation(stdout, args.output, "accepted", &status, || {
                            Ok(render_rollout_status(&human_style, &status))
                        })?;
                        if args.watch {
                            let rollout_id = status.rollout_id.clone();
                            watch_rollout(
                                &client,
                                stdout,
                                human_style,
                                &args.target.pipeline_group_id,
                                &args.target.pipeline_id,
                                &rollout_id,
                                args.watch_interval,
                                stream_output_from_mutation(args.output)?,
                                Some(status),
                            )
                            .await
                        } else {
                            Ok(())
                        }
                    }
                    otap_df_admin_api::pipelines::ReconfigureOutcome::Completed(status) => {
                        emit_mutation(stdout, args.output, "completed", &status, || {
                            Ok(render_rollout_status(&human_style, &status))
                        })
                    }
                    otap_df_admin_api::pipelines::ReconfigureOutcome::Failed(status) => {
                        emit_mutation(stdout, args.output, "failed", &status, || {
                            Ok(render_rollout_status(&human_style, &status))
                        })?;
                        Err(CliError::outcome_failure(format!(
                            "pipeline rollout '{}' failed",
                            status.rollout_id
                        )))
                    }
                    otap_df_admin_api::pipelines::ReconfigureOutcome::TimedOut(status) => {
                        emit_mutation(stdout, args.output, "timed_out", &status, || {
                            Ok(render_rollout_status(&human_style, &status))
                        })?;
                        Err(CliError::outcome_failure(format!(
                            "pipeline rollout '{}' timed out",
                            status.rollout_id
                        )))
                    }
                }
            }
            PipelinesCommand::Shutdown(args) => {
                validate_mutation_output(args.output, args.watch)?;
                let outcome = client
                    .pipelines()
                    .shutdown(
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                        &operation_options(args.wait, args.wait_timeout),
                    )
                    .await?;

                match outcome {
                    otap_df_admin_api::pipelines::ShutdownOutcome::Accepted(status) => {
                        emit_mutation(stdout, args.output, "accepted", &status, || {
                            Ok(render_shutdown_status(&human_style, &status))
                        })?;
                        if args.watch {
                            let shutdown_id = status.shutdown_id.clone();
                            watch_shutdown(
                                &client,
                                stdout,
                                human_style,
                                &args.target.pipeline_group_id,
                                &args.target.pipeline_id,
                                &shutdown_id,
                                args.watch_interval,
                                stream_output_from_mutation(args.output)?,
                                Some(status),
                            )
                            .await
                        } else {
                            Ok(())
                        }
                    }
                    otap_df_admin_api::pipelines::ShutdownOutcome::Completed(status) => {
                        emit_mutation(stdout, args.output, "completed", &status, || {
                            Ok(render_shutdown_status(&human_style, &status))
                        })
                    }
                    otap_df_admin_api::pipelines::ShutdownOutcome::Failed(status) => {
                        emit_mutation(stdout, args.output, "failed", &status, || {
                            Ok(render_shutdown_status(&human_style, &status))
                        })?;
                        Err(CliError::outcome_failure(format!(
                            "pipeline shutdown '{}' failed",
                            status.shutdown_id
                        )))
                    }
                    otap_df_admin_api::pipelines::ShutdownOutcome::TimedOut(status) => {
                        emit_mutation(stdout, args.output, "timed_out", &status, || {
                            Ok(render_shutdown_status(&human_style, &status))
                        })?;
                        Err(CliError::outcome_failure(format!(
                            "pipeline shutdown '{}' timed out",
                            status.shutdown_id
                        )))
                    }
                }
            }
            PipelinesCommand::Rollouts(args) => match args.command {
                RolloutCommand::Get(args) => {
                    get_rollout(
                        &client,
                        stdout,
                        human_style,
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                        &args.target.rollout_id,
                        args.output.output,
                    )
                    .await
                }
                RolloutCommand::Watch(args) => {
                    watch_rollout(
                        &client,
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
                get_rollout(
                    &client,
                    stdout,
                    human_style,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    &args.target.rollout_id,
                    args.output.output,
                )
                .await
            }
            PipelinesCommand::Shutdowns(args) => match args.command {
                ShutdownCommand::Get(args) => {
                    get_shutdown(
                        &client,
                        stdout,
                        human_style,
                        &args.target.pipeline_group_id,
                        &args.target.pipeline_id,
                        &args.target.shutdown_id,
                        args.output.output,
                    )
                    .await
                }
                ShutdownCommand::Watch(args) => {
                    watch_shutdown(
                        &client,
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
                get_shutdown(
                    &client,
                    stdout,
                    human_style,
                    &args.target.pipeline_group_id,
                    &args.target.pipeline_id,
                    &args.target.shutdown_id,
                    args.output.output,
                )
                .await
            }
        },
        Command::Telemetry(args) => match args.command {
            TelemetryCommand::Logs(args) => match args.command {
                LogsCommand::Get(args) => {
                    let logs = filter_logs(
                        &fetch_logs(&client, args.after, args.limit).await?,
                        &log_filters_from_args(args.filters),
                    );
                    emit_read(stdout, args.output.output, &logs, || {
                        Ok(render_logs(&human_style, &logs))
                    })
                }
                LogsCommand::Watch(args) => {
                    watch_logs(
                        &client,
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
                            emit_read(stdout, args.output.output, &metrics, || {
                                Ok(render_metrics_compact(&human_style, &metrics))
                            })
                        }
                        MetricsShape::Full => {
                            let metrics = filter_metrics_full(
                                &client.telemetry().metrics(&options).await?,
                                &metrics_filters_from_args(args.filters),
                            );
                            emit_read(stdout, args.output.output, &metrics, || {
                                Ok(render_metrics_full(&human_style, &metrics))
                            })
                        }
                    }
                }
                MetricsCommand::Watch(args) => {
                    watch_metrics(
                        &client,
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
        },
    }
}

async fn get_rollout(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    pipeline_group_id: &str,
    pipeline_id: &str,
    rollout_id: &str,
    output: ReadOutput,
) -> Result<(), CliError> {
    let status = fetch_rollout(client, pipeline_group_id, pipeline_id, rollout_id).await?;
    emit_read(stdout, output, &status, || {
        Ok(render_rollout_status(&human_style, &status))
    })
}

async fn get_shutdown(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    pipeline_group_id: &str,
    pipeline_id: &str,
    shutdown_id: &str,
    output: ReadOutput,
) -> Result<(), CliError> {
    let status = fetch_shutdown(client, pipeline_group_id, pipeline_id, shutdown_id).await?;
    emit_read(stdout, output, &status, || {
        Ok(render_shutdown_status(&human_style, &status))
    })
}

async fn fetch_logs(
    client: &AdminClient,
    after: Option<u64>,
    limit: Option<usize>,
) -> Result<telemetry::LogsResponse, CliError> {
    let logs = client.telemetry().logs(&LogsQuery { after, limit }).await?;
    logs.ok_or_else(|| CliError::not_found("retained admin logs are not available on this engine"))
}

async fn fetch_rollout(
    client: &AdminClient,
    pipeline_group_id: &str,
    pipeline_id: &str,
    rollout_id: &str,
) -> Result<otap_df_admin_api::pipelines::RolloutStatus, CliError> {
    client
        .pipelines()
        .rollout_status(pipeline_group_id, pipeline_id, rollout_id)
        .await?
        .ok_or_else(|| {
            CliError::not_found(format!(
                "rollout '{}' for pipeline '{}/{}' was not found",
                rollout_id, pipeline_group_id, pipeline_id
            ))
        })
}

async fn fetch_shutdown(
    client: &AdminClient,
    pipeline_group_id: &str,
    pipeline_id: &str,
    shutdown_id: &str,
) -> Result<otap_df_admin_api::pipelines::ShutdownStatus, CliError> {
    client
        .pipelines()
        .shutdown_status(pipeline_group_id, pipeline_id, shutdown_id)
        .await?
        .ok_or_else(|| {
            CliError::not_found(format!(
                "shutdown '{}' for pipeline '{}/{}' was not found",
                shutdown_id, pipeline_group_id, pipeline_id
            ))
        })
}

async fn fetch_pipeline_status(
    client: &AdminClient,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<otap_df_admin_api::pipelines::Status, CliError> {
    client
        .pipelines()
        .status(pipeline_group_id, pipeline_id)
        .await?
        .ok_or_else(|| {
            CliError::not_found(format!(
                "pipeline '{}/{}' was not found",
                pipeline_group_id, pipeline_id
            ))
        })
}

async fn fetch_pipeline_describe(
    client: &AdminClient,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<PipelineDescribeReport, CliError> {
    let details = client
        .pipelines()
        .details(pipeline_group_id, pipeline_id)
        .await?;
    let Some(details) = details else {
        return Err(CliError::not_found(format!(
            "pipeline '{}/{}' was not found",
            pipeline_group_id, pipeline_id
        )));
    };
    let status = fetch_pipeline_status(client, pipeline_group_id, pipeline_id).await?;
    let livez = client
        .pipelines()
        .livez(pipeline_group_id, pipeline_id)
        .await?;
    let readyz = client
        .pipelines()
        .readyz(pipeline_group_id, pipeline_id)
        .await?;
    Ok(describe_pipeline(details, status, livez, readyz))
}

fn group_event_filters(
    kinds: Vec<EventKind>,
    pipeline_group_id: Option<String>,
    pipeline_id: Option<String>,
    node_id: Option<String>,
    contains: Option<String>,
) -> EventFilters {
    EventFilters {
        kinds: map_event_kinds(kinds),
        pipeline_group_id,
        pipeline_id,
        node_id,
        contains,
    }
}

fn pipeline_event_filters(
    kinds: Vec<EventKind>,
    node_id: Option<String>,
    contains: Option<String>,
) -> EventFilters {
    EventFilters {
        kinds: map_event_kinds(kinds),
        pipeline_group_id: None,
        pipeline_id: None,
        node_id,
        contains,
    }
}

fn log_filters_from_args(args: LogsFilterArgs) -> LogFilters {
    LogFilters {
        level: args.level,
        target: args.target,
        event: args.event,
        pipeline_group_id: args.pipeline_group_id,
        pipeline_id: args.pipeline_id,
        node_id: args.node_id,
        contains: args.contains,
    }
}

fn metrics_filters_from_args(args: MetricsFilterArgs) -> MetricsFilters {
    MetricsFilters {
        metric_sets: args.metric_sets,
        metric_names: args.metric_names,
        pipeline_group_id: args.pipeline_group_id,
        pipeline_id: args.pipeline_id,
        core_id: args.core_id,
        node_id: args.node_id,
    }
}

fn pipeline_scope_filters(
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> (LogFilters, MetricsFilters) {
    (
        LogFilters {
            pipeline_group_id: Some(pipeline_group_id.to_string()),
            pipeline_id: Some(pipeline_id.to_string()),
            ..LogFilters::default()
        },
        MetricsFilters {
            pipeline_group_id: Some(pipeline_group_id.to_string()),
            pipeline_id: Some(pipeline_id.to_string()),
            ..MetricsFilters::default()
        },
    )
}

fn map_event_kinds(kinds: Vec<EventKind>) -> Vec<NormalizedEventKind> {
    kinds
        .into_iter()
        .map(|kind| match kind {
            EventKind::Request => NormalizedEventKind::Request,
            EventKind::Success => NormalizedEventKind::Success,
            EventKind::Error => NormalizedEventKind::Error,
            EventKind::Log => NormalizedEventKind::Log,
        })
        .collect()
}

async fn watch_group_events(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    filters: EventFilters,
    tail: Option<usize>,
    interval: Duration,
    output: StreamOutput,
) -> Result<(), CliError> {
    let initial = client.groups().status().await?;
    let initial_events = tail_events(
        extract_events_from_group_status(&initial, Some(&filters)),
        tail,
    );
    let mut seen = BTreeSet::new();
    emit_events(stdout, human_style, output, &initial_events, &mut seen)?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(interval) => {}
        }
        let status = client.groups().status().await?;
        let events = extract_events_from_group_status(&status, Some(&filters));
        emit_events(stdout, human_style, output, &events, &mut seen)?;
    }
}

async fn watch_pipeline_events(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    pipeline_group_id: &str,
    pipeline_id: &str,
    filters: EventFilters,
    tail: Option<usize>,
    interval: Duration,
    output: StreamOutput,
) -> Result<(), CliError> {
    let status = fetch_pipeline_status(client, pipeline_group_id, pipeline_id).await?;
    let initial_events = tail_events(
        extract_events_from_pipeline_status(
            pipeline_group_id,
            pipeline_id,
            &status,
            Some(&filters),
        ),
        tail,
    );
    let mut seen = BTreeSet::new();
    emit_events(stdout, human_style, output, &initial_events, &mut seen)?;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(interval) => {}
        }
        let status = fetch_pipeline_status(client, pipeline_group_id, pipeline_id).await?;
        let events = extract_events_from_pipeline_status(
            pipeline_group_id,
            pipeline_id,
            &status,
            Some(&filters),
        );
        emit_events(stdout, human_style, output, &events, &mut seen)?;
    }
}

fn emit_events(
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    output: StreamOutput,
    events: &[NormalizedEvent],
    seen: &mut BTreeSet<String>,
) -> Result<(), CliError> {
    for event in events {
        let identity = event.identity_key();
        if !seen.insert(identity) {
            continue;
        }
        match output {
            StreamOutput::Human => write_human(stdout, &render_event_line(&human_style, event))?,
            StreamOutput::Ndjson => write_event_output(stdout, "event", event)?,
        }
    }
    Ok(())
}

async fn watch_groups_shutdown(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    request_status: otap_df_admin_api::groups::ShutdownStatus,
    wait_timeout: Duration,
    interval: Duration,
    output: StreamOutput,
) -> Result<(), CliError> {
    // TODO: Replace this client-side heuristic with a first-class group shutdown
    // resource once the admin API and SDK expose coordinated shutdown ids/status.
    let started_at = SystemTime::now();
    let mut last_generated_at = None::<String>;

    loop {
        let status = client.groups().status().await?;
        let snapshot = group_shutdown_snapshot(request_status, &status, started_at);
        if last_generated_at.as_deref() != Some(snapshot.generated_at.as_str()) {
            write_stream_snapshot(
                stdout,
                output,
                "group_shutdown",
                || Ok(render_group_shutdown_watch(&human_style, &snapshot)),
                &snapshot,
                human_style,
            )?;
            last_generated_at = Some(snapshot.generated_at.clone());
        }
        if snapshot.all_terminal {
            return Ok(());
        }
        if started_at.elapsed().unwrap_or_default() >= wait_timeout {
            return Err(CliError::outcome_failure(format!(
                "groups shutdown did not reach terminal pipeline phases within {}s",
                duration_to_secs_ceil(wait_timeout)
            )));
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

async fn watch_rollout(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    pipeline_group_id: &str,
    pipeline_id: &str,
    rollout_id: &str,
    interval: Duration,
    output: StreamOutput,
    initial: Option<otap_df_admin_api::pipelines::RolloutStatus>,
) -> Result<(), CliError> {
    let mut current = if let Some(initial) = initial {
        initial
    } else {
        client
            .pipelines()
            .rollout_status(pipeline_group_id, pipeline_id, rollout_id)
            .await?
            .ok_or_else(|| {
                CliError::not_found(format!(
                    "rollout '{}' for pipeline '{}/{}' was not found",
                    rollout_id, pipeline_group_id, pipeline_id
                ))
            })?
    };
    let mut last_updated = None::<String>;

    loop {
        if last_updated.as_deref() != Some(current.updated_at.as_str()) {
            write_stream_snapshot(
                stdout,
                output,
                "pipeline_rollout",
                || Ok(render_rollout_status(&human_style, &current)),
                &current,
                human_style,
            )?;
            last_updated = Some(current.updated_at.clone());
        }

        if rollout_is_terminal(current.state) {
            return if current.state == otap_df_admin_api::pipelines::PipelineRolloutState::Succeeded
            {
                Ok(())
            } else {
                Err(CliError::outcome_failure(format!(
                    "pipeline rollout '{}' ended in state {:?}",
                    current.rollout_id, current.state
                )))
            };
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(interval) => {}
        }

        current = client
            .pipelines()
            .rollout_status(pipeline_group_id, pipeline_id, rollout_id)
            .await?
            .ok_or_else(|| {
                CliError::not_found(format!(
                    "rollout '{}' for pipeline '{}/{}' was not found",
                    rollout_id, pipeline_group_id, pipeline_id
                ))
            })?;
    }
}

async fn watch_shutdown(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    pipeline_group_id: &str,
    pipeline_id: &str,
    shutdown_id: &str,
    interval: Duration,
    output: StreamOutput,
    initial: Option<otap_df_admin_api::pipelines::ShutdownStatus>,
) -> Result<(), CliError> {
    let mut current = if let Some(initial) = initial {
        initial
    } else {
        client
            .pipelines()
            .shutdown_status(pipeline_group_id, pipeline_id, shutdown_id)
            .await?
            .ok_or_else(|| {
                CliError::not_found(format!(
                    "shutdown '{}' for pipeline '{}/{}' was not found",
                    shutdown_id, pipeline_group_id, pipeline_id
                ))
            })?
    };
    let mut last_updated = None::<String>;

    loop {
        if last_updated.as_deref() != Some(current.updated_at.as_str()) {
            write_stream_snapshot(
                stdout,
                output,
                "pipeline_shutdown",
                || Ok(render_shutdown_status(&human_style, &current)),
                &current,
                human_style,
            )?;
            last_updated = Some(current.updated_at.clone());
        }

        if shutdown_is_terminal(&current.state) {
            return if shutdown_is_success(&current.state) {
                Ok(())
            } else {
                Err(CliError::outcome_failure(format!(
                    "pipeline shutdown '{}' ended in state {}",
                    current.shutdown_id, current.state
                )))
            };
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(interval) => {}
        }

        current = client
            .pipelines()
            .shutdown_status(pipeline_group_id, pipeline_id, shutdown_id)
            .await?
            .ok_or_else(|| {
                CliError::not_found(format!(
                    "shutdown '{}' for pipeline '{}/{}' was not found",
                    shutdown_id, pipeline_group_id, pipeline_id
                ))
            })?;
    }
}

async fn watch_logs(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    after: Option<u64>,
    tail: Option<usize>,
    limit: Option<usize>,
    filters: LogFilters,
    interval: Duration,
    output: StreamOutput,
) -> Result<(), CliError> {
    let mut cursor = after;

    if after.is_none() {
        if let Some(tail) = tail {
            let response = fetch_logs(client, None, Some(tail)).await?;
            let filtered = filter_logs(&response, &filters);
            emit_logs(stdout, human_style, output, &filtered.logs)?;
            cursor = Some(response.next_seq);
        } else {
            let response = fetch_logs(client, None, Some(1)).await?;
            cursor = Some(response.next_seq);
        }
    }

    loop {
        let response = fetch_logs(client, cursor, limit).await?;
        let filtered = filter_logs(&response, &filters);
        emit_logs(stdout, human_style, output, &filtered.logs)?;
        cursor = Some(response.next_seq);

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

async fn watch_metrics(
    client: &AdminClient,
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    shape: MetricsShape,
    options: MetricsOptions,
    filters: MetricsFilters,
    interval: Duration,
    output: StreamOutput,
) -> Result<(), CliError> {
    loop {
        match shape {
            MetricsShape::Compact => {
                let metrics = filter_metrics_compact(
                    &client.telemetry().metrics_compact(&options).await?,
                    &filters,
                );
                write_stream_snapshot(
                    stdout,
                    output,
                    "telemetry_metrics",
                    || Ok(render_metrics_compact(&human_style, &metrics)),
                    &metrics,
                    human_style,
                )?;
            }
            MetricsShape::Full => {
                let metrics =
                    filter_metrics_full(&client.telemetry().metrics(&options).await?, &filters);
                write_stream_snapshot(
                    stdout,
                    output,
                    "telemetry_metrics",
                    || Ok(render_metrics_full(&human_style, &metrics)),
                    &metrics,
                    human_style,
                )?;
            }
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

fn emit_logs(
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    output: StreamOutput,
    logs: &[telemetry::LogEntry],
) -> Result<(), CliError> {
    for entry in logs {
        match output {
            StreamOutput::Human => {
                write_human(stdout, &render::render_log_line(&human_style, entry))?
            }
            StreamOutput::Ndjson => write_log_event(stdout, entry)?,
        }
    }
    Ok(())
}

fn emit_read<T: Serialize>(
    stdout: &mut dyn Write,
    output: ReadOutput,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        ReadOutput::Human => write_human(stdout, &human()?),
        ReadOutput::Json | ReadOutput::Yaml => write_read_output(stdout, output, value),
    }
}

fn emit_mutation<T: Serialize>(
    stdout: &mut dyn Write,
    output: MutationOutput,
    outcome: &str,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        MutationOutput::Human => write_human(stdout, &human()?),
        MutationOutput::Json | MutationOutput::Yaml | MutationOutput::Ndjson => {
            write_mutation_output(stdout, output, outcome, value)
        }
    }
}

fn emit_group_shutdown<T: Serialize>(
    stdout: &mut dyn Write,
    output: GroupShutdownOutput,
    value: &T,
    human: impl FnOnce() -> Result<String, CliError>,
) -> Result<(), CliError> {
    match output {
        GroupShutdownOutput::Human => write_human(stdout, &human()?),
        GroupShutdownOutput::Json => write_read_output(stdout, ReadOutput::Json, value),
        GroupShutdownOutput::Yaml => write_read_output(stdout, ReadOutput::Yaml, value),
        GroupShutdownOutput::Ndjson => write_event_output(stdout, "snapshot", value),
    }
}

fn validate_group_shutdown_output(
    output: GroupShutdownOutput,
    watch: bool,
) -> Result<(), CliError> {
    if watch
        && matches!(
            output,
            GroupShutdownOutput::Json | GroupShutdownOutput::Yaml
        )
    {
        return Err(CliError::invalid_usage(
            "--watch requires --output human or --output ndjson",
        ));
    }
    if !watch && matches!(output, GroupShutdownOutput::Ndjson) {
        return Err(CliError::invalid_usage("--output ndjson requires --watch"));
    }
    Ok(())
}

fn group_shutdown_stream_output(output: GroupShutdownOutput) -> Result<StreamOutput, CliError> {
    match output {
        GroupShutdownOutput::Human => Ok(StreamOutput::Human),
        GroupShutdownOutput::Ndjson => Ok(StreamOutput::Ndjson),
        GroupShutdownOutput::Json | GroupShutdownOutput::Yaml => Err(CliError::invalid_usage(
            "--watch requires --output human or --output ndjson",
        )),
    }
}

fn bundle_metadata(logs_limit: usize, shape: MetricsShape) -> BundleMetadata {
    BundleMetadata {
        collected_at: humantime::format_rfc3339_seconds(SystemTime::now()).to_string(),
        logs_limit,
        metrics_shape: match shape {
            MetricsShape::Compact => BundleMetricsShape::Compact,
            MetricsShape::Full => BundleMetricsShape::Full,
        },
    }
}

fn write_bundle<T: Serialize>(
    stdout: &mut dyn Write,
    output: BundleOutput,
    path: Option<&Path>,
    value: &T,
) -> Result<(), CliError> {
    match path {
        Some(path) if path != Path::new("-") => {
            let mut file = fs::File::create(path).map_err(|err| {
                CliError::config(format!(
                    "failed to create bundle output file '{}': {err}",
                    path.display()
                ))
            })?;
            write_bundle_output(&mut file, output, value)
        }
        _ => write_bundle_output(stdout, output, value),
    }
}

fn validate_mutation_output(output: MutationOutput, watch: bool) -> Result<(), CliError> {
    if watch && matches!(output, MutationOutput::Json | MutationOutput::Yaml) {
        return Err(CliError::invalid_usage(
            "--watch requires --output human or --output ndjson",
        ));
    }
    if !watch && matches!(output, MutationOutput::Ndjson) {
        return Err(CliError::invalid_usage("--output ndjson requires --watch"));
    }
    Ok(())
}

fn stream_output_from_mutation(output: MutationOutput) -> Result<StreamOutput, CliError> {
    match output {
        MutationOutput::Human => Ok(StreamOutput::Human),
        MutationOutput::Ndjson => Ok(StreamOutput::Ndjson),
        MutationOutput::Json | MutationOutput::Yaml => Err(CliError::invalid_usage(
            "--watch requires --output human or --output ndjson",
        )),
    }
}

fn operation_options(wait: bool, wait_timeout: Duration) -> OperationOptions {
    OperationOptions {
        wait,
        timeout_secs: duration_to_secs_ceil(wait_timeout),
    }
}

fn duration_to_secs_ceil(duration: Duration) -> u64 {
    let secs = duration.as_secs();
    if duration.subsec_nanos() == 0 {
        secs
    } else {
        secs.saturating_add(1)
    }
    .max(1)
}

pub(crate) fn load_pipeline_config(
    path: &Path,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<PipelineConfig, CliError> {
    let mut content = String::new();
    if path == Path::new("-") {
        _ = std::io::stdin().read_to_string(&mut content)?;
    } else {
        content = fs::read_to_string(path).map_err(|err| {
            CliError::config(format!(
                "failed to read pipeline file '{}': {err}",
                path.display()
            ))
        })?;
    }

    parse_pipeline_config_content(&content, pipeline_group_id, pipeline_id)
}

pub(crate) fn parse_pipeline_config_content(
    content: &str,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<PipelineConfig, CliError> {
    let parse_result = if looks_like_json(content) {
        PipelineConfig::from_json(
            pipeline_group_id.to_string().into(),
            pipeline_id.to_string().into(),
            content,
        )
    } else {
        PipelineConfig::from_yaml(
            pipeline_group_id.to_string().into(),
            pipeline_id.to_string().into(),
            content,
        )
    };

    parse_result.map_err(|err| {
        CliError::config(format!(
            "failed to parse pipeline config for '{}/{}': {err}",
            pipeline_group_id, pipeline_id
        ))
    })
}

pub(crate) fn serialize_pipeline_config_yaml(
    pipeline: &PipelineConfig,
) -> Result<String, CliError> {
    serde_yaml::to_string(pipeline).map_err(|err| {
        CliError::config(format!(
            "failed to serialize pipeline config to YAML: {err}"
        ))
    })
}

fn looks_like_json(content: &str) -> bool {
    matches!(content.chars().find(|ch| !ch.is_whitespace()), Some('{'))
}

fn rollout_is_terminal(state: otap_df_admin_api::pipelines::PipelineRolloutState) -> bool {
    matches!(
        state,
        otap_df_admin_api::pipelines::PipelineRolloutState::Succeeded
            | otap_df_admin_api::pipelines::PipelineRolloutState::Failed
            | otap_df_admin_api::pipelines::PipelineRolloutState::RollbackFailed
    )
}

fn shutdown_is_terminal(state: &str) -> bool {
    matches!(state, "succeeded" | "failed")
}

fn shutdown_is_success(state: &str) -> bool {
    state == "succeeded"
}

fn ensure_crypto_provider() -> Result<(), CliError> {
    static INIT: OnceLock<Result<(), String>> = OnceLock::new();

    INIT.get_or_init(|| {
        #[cfg(feature = "crypto-openssl")]
        {
            let _ = rustls_openssl::default_provider().install_default();
            Ok(())
        }

        #[cfg(all(feature = "crypto-aws-lc", not(feature = "crypto-openssl")))]
        {
            let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
            Ok(())
        }

        #[cfg(all(
            feature = "crypto-ring",
            not(feature = "crypto-openssl"),
            not(feature = "crypto-aws-lc")
        ))]
        {
            let _ = rustls::crypto::ring::default_provider().install_default();
            Ok(())
        }

        #[cfg(not(any(
            feature = "crypto-ring",
            feature = "crypto-aws-lc",
            feature = "crypto-openssl"
        )))]
        {
            Err(
                "admin TLS support requires one of the crypto features: crypto-ring, crypto-aws-lc, or crypto-openssl"
                    .to_string(),
            )
        }
    })
    .clone()
    .map_err(CliError::config)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::ColorChoice;
    use crate::style::HumanStyle;
    use clap::Parser;
    use otap_df_admin_api::config::pipeline::{PipelineConfigBuilder, PipelineType};
    use serde_json::json;
    use tempfile::tempdir;
    use wiremock::matchers::{body_json, method, path, query_param};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn pipeline_config() -> PipelineConfig {
        PipelineConfigBuilder::new()
            .add_receiver("ingress", "receiver:otlp", None)
            .add_exporter("egress", "exporter:debug", None)
            .to("ingress", "egress")
            .build(PipelineType::Otap, "tenant-a", "ingest")
            .expect("pipeline config")
    }

    #[tokio::test]
    async fn engine_status_json_command_hits_expected_route() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "generatedAt": "2026-01-01T00:00:00Z",
                "pipelines": {}
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "engine",
            "status",
            "--output",
            "json",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\"generatedAt\""));
    }

    #[tokio::test]
    async fn metrics_get_uses_compact_route_shape() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("format", "json_compact"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "metric_sets": []
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "telemetry",
            "metrics",
            "get",
            "--shape",
            "compact",
            "--output",
            "json",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");
    }

    #[tokio::test]
    async fn metrics_get_uses_full_route_shape() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("format", "json"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "metric_sets": []
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "telemetry",
            "metrics",
            "get",
            "--shape",
            "full",
            "--output",
            "json",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");
    }

    #[tokio::test]
    async fn logs_get_human_color_always_emits_ansi() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/logs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "oldest_seq": 1,
                "newest_seq": 1,
                "next_seq": 2,
                "truncated_before_seq": null,
                "dropped_on_ingest": 0,
                "dropped_on_retention": 0,
                "retained_bytes": 10,
                "logs": [
                    {
                        "seq": 1,
                        "timestamp": "2026-01-01T00:00:00Z",
                        "level": "INFO",
                        "target": "test.target",
                        "event_name": "log.1",
                        "file": null,
                        "line": null,
                        "rendered": "hello world",
                        "contexts": []
                    }
                ]
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "--color",
            "always",
            "telemetry",
            "logs",
            "get",
            "--limit",
            "1",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\u{1b}["));
        assert!(output.contains("hello world"));
    }

    #[tokio::test]
    async fn metrics_get_human_auto_stays_plain_off_terminal() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("format", "json_compact"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "metric_sets": [
                    {
                        "name": "engine.runtime",
                        "attributes": {},
                        "metrics": {
                            "pipelines": 3
                        }
                    }
                ]
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "telemetry",
            "metrics",
            "get",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(!output.contains("\u{1b}["));
        assert!(output.contains("metric_set: engine.runtime"));
    }

    #[tokio::test]
    async fn metrics_get_human_color_always_emits_ansi_on_terminal() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("format", "json_compact"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "metric_sets": [
                    {
                        "name": "engine.runtime",
                        "attributes": {},
                        "metrics": {
                            "pipelines": 3
                        }
                    }
                ]
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "--color",
            "always",
            "telemetry",
            "metrics",
            "get",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run_with_terminal(cli, &mut stdout, true)
            .await
            .expect("run_with_terminal");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\u{1b}["));
        assert!(output.contains("engine.runtime"));
    }

    #[tokio::test]
    async fn metrics_json_output_ignores_color_setting() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("format", "json_compact"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "metric_sets": []
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "--color",
            "always",
            "telemetry",
            "metrics",
            "get",
            "--output",
            "json",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run_with_terminal(cli, &mut stdout, true)
            .await
            .expect("run_with_terminal");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(!output.contains("\u{1b}["));
        assert!(output.contains("\"timestamp\""));
    }

    #[tokio::test]
    async fn reconfigure_sends_wait_query_and_request_body() {
        let server = MockServer::start().await;
        let dir = tempdir().expect("tempdir");
        let file_path = dir.path().join("pipeline.yaml");
        fs::write(
            &file_path,
            serde_yaml::to_string(&pipeline_config()).expect("yaml"),
        )
        .expect("write");

        Mock::given(method("PUT"))
            .and(path("/api/v1/groups/tenant-a/pipelines/ingest"))
            .and(query_param("wait", "true"))
            .and(query_param("timeout_secs", "30"))
            .and(body_json(json!({
                "pipeline": serde_json::to_value(pipeline_config()).expect("value"),
                "stepTimeoutSecs": 60,
                "drainTimeoutSecs": 60
            })))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "rolloutId": "rollout-1",
                "pipelineGroupId": "tenant-a",
                "pipelineId": "ingest",
                "action": "replace",
                "state": "succeeded",
                "targetGeneration": 2,
                "previousGeneration": 1,
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:00Z",
                "cores": []
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "pipelines",
            "reconfigure",
            "tenant-a",
            "ingest",
            "--file",
            file_path.to_str().expect("path"),
            "--wait",
            "--wait-timeout",
            "30s",
            "--output",
            "json",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");
    }

    #[tokio::test]
    async fn rollout_watch_emits_ndjson_and_stops_on_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/api/v1/groups/tenant-a/pipelines/ingest/rollouts/rollout-1",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "rolloutId": "rollout-1",
                "pipelineGroupId": "tenant-a",
                "pipelineId": "ingest",
                "action": "replace",
                "state": "succeeded",
                "targetGeneration": 2,
                "previousGeneration": 1,
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:01Z",
                "cores": []
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "pipelines",
            "rollouts",
            "watch",
            "tenant-a",
            "ingest",
            "rollout-1",
            "--output",
            "ndjson",
            "--interval",
            "1ms",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\"resource\":\"pipeline_rollout\""));
    }

    #[tokio::test]
    async fn shutdown_watch_emits_ndjson_and_stops_on_success() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path(
                "/api/v1/groups/tenant-a/pipelines/ingest/shutdowns/shutdown-1",
            ))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "shutdownId": "shutdown-1",
                "pipelineGroupId": "tenant-a",
                "pipelineId": "ingest",
                "state": "succeeded",
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:01Z",
                "cores": []
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "pipelines",
            "shutdowns",
            "watch",
            "tenant-a",
            "ingest",
            "shutdown-1",
            "--output",
            "ndjson",
            "--interval",
            "1ms",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\"resource\":\"pipeline_shutdown\""));
    }

    #[tokio::test]
    async fn logs_watch_uses_next_seq_as_after_cursor() {
        ensure_crypto_provider().expect("crypto provider");
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/logs"))
            .and(query_param("limit", "2"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "oldest_seq": 49,
                "newest_seq": 50,
                "next_seq": 50,
                "truncated_before_seq": null,
                "dropped_on_ingest": 0,
                "dropped_on_retention": 0,
                "retained_bytes": 10,
                "logs": [
                    {
                        "seq": 49,
                        "timestamp": "2026-01-01T00:00:00Z",
                        "level": "INFO",
                        "target": "test",
                        "event_name": "log.49",
                        "file": null,
                        "line": null,
                        "rendered": "log 49",
                        "contexts": []
                    },
                    {
                        "seq": 50,
                        "timestamp": "2026-01-01T00:00:01Z",
                        "level": "INFO",
                        "target": "test",
                        "event_name": "log.50",
                        "file": null,
                        "line": null,
                        "rendered": "log 50",
                        "contexts": []
                    }
                ]
            })))
            .expect(1)
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/logs"))
            .and(query_param("after", "50"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "oldest_seq": 49,
                "newest_seq": 50,
                "next_seq": 50,
                "truncated_before_seq": null,
                "dropped_on_ingest": 0,
                "dropped_on_retention": 0,
                "retained_bytes": 10,
                "logs": []
            })))
            .mount(&server)
            .await;

        let client = AdminClient::builder()
            .http(otap_df_admin_api::HttpAdminClientSettings::new(
                otap_df_admin_api::AdminEndpoint::from_url(&server.uri()).expect("endpoint"),
            ))
            .build()
            .expect("client");

        let mut stdout = Vec::new();
        let result = tokio::time::timeout(
            Duration::from_millis(10),
            watch_logs(
                &client,
                &mut stdout,
                HumanStyle::resolve(ColorChoice::Never, false),
                None,
                Some(2),
                None,
                LogFilters::default(),
                Duration::from_millis(1),
                StreamOutput::Ndjson,
            ),
        )
        .await;

        assert!(result.is_err(), "watch should still be polling");
        let output = String::from_utf8(stdout).expect("utf8");
        assert_eq!(
            output.lines().count(),
            2,
            "expected retained logs only once"
        );
        assert!(output.contains("\"seq\":49"));
        assert!(output.contains("\"seq\":50"));
    }

    #[tokio::test]
    async fn metrics_watch_human_color_always_styles_stream_header() {
        ensure_crypto_provider().expect("crypto provider");
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/metrics"))
            .and(query_param("format", "json_compact"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "timestamp": "2026-01-01T00:00:00Z",
                "metric_sets": [
                    {
                        "name": "engine.runtime",
                        "attributes": {},
                        "metrics": {
                            "pipelines": 3
                        }
                    }
                ]
            })))
            .mount(&server)
            .await;

        let client = AdminClient::builder()
            .http(otap_df_admin_api::HttpAdminClientSettings::new(
                otap_df_admin_api::AdminEndpoint::from_url(&server.uri()).expect("endpoint"),
            ))
            .build()
            .expect("client");

        let mut stdout = Vec::new();
        let result = tokio::time::timeout(
            Duration::from_millis(10),
            watch_metrics(
                &client,
                &mut stdout,
                HumanStyle::resolve(ColorChoice::Always, false),
                MetricsShape::Compact,
                MetricsOptions {
                    reset: false,
                    keep_all_zeroes: false,
                },
                MetricsFilters::default(),
                Duration::from_millis(1),
                StreamOutput::Human,
            ),
        )
        .await;

        assert!(result.is_err(), "watch should still be polling");
        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\u{1b}["));
        assert!(output.contains("[telemetry_metrics]"));
    }

    #[tokio::test]
    async fn pipeline_describe_json_fetches_details_status_and_probes() {
        let server = MockServer::start().await;
        let details_value = serde_json::to_value(pipeline_config()).expect("pipeline value");

        Mock::given(method("GET"))
            .and(path("/api/v1/groups/tenant-a/pipelines/ingest"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "pipelineGroupId": "tenant-a",
                "pipelineId": "ingest",
                "activeGeneration": 7,
                "pipeline": details_value,
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/groups/tenant-a/pipelines/ingest/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "conditions": [
                    {
                        "type": "Ready",
                        "status": "True",
                        "reason": "Running"
                    }
                ],
                "totalCores": 1,
                "runningCores": 1,
                "cores": {
                    "0": {
                        "phase": "running",
                        "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                        "conditions": [],
                        "deletePending": false,
                        "recentEvents": [
                            {
                                "Engine": {
                                    "key": {
                                        "pipeline_group_id": "tenant-a",
                                        "pipeline_id": "ingest",
                                        "core_id": 0
                                    },
                                    "time": "2026-01-01T00:00:00Z",
                                    "type": {
                                        "Success": "Ready"
                                    }
                                }
                            }
                        ]
                    }
                },
                "activeGeneration": 7
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/groups/tenant-a/pipelines/ingest/livez"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "ok"
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/groups/tenant-a/pipelines/ingest/readyz"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "status": "ok"
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "pipelines",
            "describe",
            "tenant-a",
            "ingest",
            "--output",
            "json",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\"details\""));
        assert!(output.contains("\"recentEvents\""));
        assert!(output.contains("\"livez\""));
        assert!(output.contains("\"readyz\""));
    }

    #[tokio::test]
    async fn logs_get_filters_by_pipeline_scope_client_side() {
        let server = MockServer::start().await;
        Mock::given(method("GET"))
            .and(path("/api/v1/telemetry/logs"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "oldest_seq": 1,
                "newest_seq": 2,
                "next_seq": 3,
                "truncated_before_seq": null,
                "dropped_on_ingest": 0,
                "dropped_on_retention": 0,
                "retained_bytes": 128,
                "logs": [
                    {
                        "seq": 1,
                        "timestamp": "2026-01-01T00:00:00Z",
                        "level": "INFO",
                        "target": "controller",
                        "event_name": "matching",
                        "file": null,
                        "line": null,
                        "rendered": "matched",
                        "contexts": [
                            {
                                "entity_key": "EntityKey(1)",
                                "schema_name": "node.attrs",
                                "attributes": {
                                    "pipeline.group.id": { "String": "tenant-a" },
                                    "pipeline.id": { "String": "ingest" },
                                    "node.id": { "String": "receiver" }
                                }
                            }
                        ]
                    },
                    {
                        "seq": 2,
                        "timestamp": "2026-01-01T00:00:01Z",
                        "level": "INFO",
                        "target": "controller",
                        "event_name": "other",
                        "file": null,
                        "line": null,
                        "rendered": "other",
                        "contexts": [
                            {
                                "entity_key": "EntityKey(2)",
                                "schema_name": "node.attrs",
                                "attributes": {
                                    "pipeline.group.id": { "String": "tenant-b" },
                                    "pipeline.id": { "String": "egress" },
                                    "node.id": { "String": "receiver" }
                                }
                            }
                        ]
                    }
                ]
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "telemetry",
            "logs",
            "get",
            "--group",
            "tenant-a",
            "--pipeline",
            "ingest",
            "--output",
            "json",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\"seq\": 1"));
        assert!(!output.contains("\"seq\": 2"));
    }

    #[tokio::test]
    async fn groups_shutdown_watch_ndjson_uses_status_heuristic() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path("/api/v1/groups/shutdown"))
            .and(query_param("wait", "false"))
            .and(query_param("timeout_secs", "60"))
            .respond_with(ResponseTemplate::new(202).set_body_json(json!({
                "status": "accepted"
            })))
            .mount(&server)
            .await;

        Mock::given(method("GET"))
            .and(path("/api/v1/groups/status"))
            .respond_with(ResponseTemplate::new(200).set_body_json(json!({
                "generatedAt": "2026-01-01T00:00:00Z",
                "pipelines": {
                    "tenant-a:ingest": {
                        "conditions": [],
                        "totalCores": 1,
                        "runningCores": 0,
                        "cores": {
                            "0": {
                                "phase": "stopped",
                                "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                                "conditions": [],
                                "deletePending": false
                            }
                        }
                    }
                }
            })))
            .mount(&server)
            .await;

        let cli = Cli::try_parse_from([
            "dfctl",
            "--url",
            &server.uri(),
            "groups",
            "shutdown",
            "--watch",
            "--output",
            "ndjson",
        ])
        .expect("parse");

        let mut stdout = Vec::new();
        run(cli, &mut stdout).await.expect("run");

        let output = String::from_utf8(stdout).expect("utf8");
        assert!(output.contains("\"resource\":\"group_shutdown\""));
        assert!(output.contains("\"allTerminal\":true"));
    }

    #[test]
    fn mutation_output_validation_enforces_watch_contract() {
        assert!(validate_mutation_output(MutationOutput::Human, true).is_ok());
        assert!(validate_mutation_output(MutationOutput::Ndjson, true).is_ok());
        assert!(validate_mutation_output(MutationOutput::Json, true).is_err());
        assert!(validate_mutation_output(MutationOutput::Ndjson, false).is_err());
    }

    #[test]
    fn duration_rounds_up_to_whole_seconds() {
        assert_eq!(duration_to_secs_ceil(Duration::from_millis(1)), 1);
        assert_eq!(duration_to_secs_ceil(Duration::from_secs(2)), 2);
        assert_eq!(duration_to_secs_ceil(Duration::from_millis(2500)), 3);
    }
}
