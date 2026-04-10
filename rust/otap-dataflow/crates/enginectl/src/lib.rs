// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! `df_enginectl` library entrypoints for the OTAP Dataflow Engine admin CLI.

mod args;
mod config;
mod error;
mod render;

pub use args::Cli;

use crate::args::{
    Cli as ParsedCli, Command, EngineCommand, GroupsCommand, LogsCommand, MetricsCommand,
    MetricsShape, MutationOutput, PipelinesCommand, ReadOutput, RolloutCommand, ShutdownCommand,
    StreamOutput, TelemetryCommand,
};
use crate::config::resolve_connection;
use crate::error::CliError;
use crate::render::{
    render_engine_probe, render_engine_status, render_groups_shutdown, render_groups_status,
    render_logs, render_metrics_compact, render_metrics_full, render_pipeline_details,
    render_pipeline_probe, render_pipeline_status, render_rollout_status, render_shutdown_status,
    write_human, write_log_event, write_mutation_output, write_read_output, write_stream_snapshot,
};
use otap_df_admin_api::config::pipeline::PipelineConfig;
use otap_df_admin_api::operations::OperationOptions;
use otap_df_admin_api::telemetry::{LogsQuery, MetricsOptions};
use otap_df_admin_api::{AdminClient, telemetry};
use serde::Serialize;
use std::fs;
use std::io::{Read, Write};
use std::path::Path;
use std::sync::OnceLock;
use std::time::Duration;

/// Executes a parsed `df_enginectl` command and writes command output to `stdout`.
pub async fn run(cli: ParsedCli, stdout: &mut dyn Write) -> Result<(), CliError> {
    ensure_crypto_provider()?;
    let resolved = resolve_connection(&cli.connection)?;
    let client = AdminClient::builder().http(resolved.settings).build()?;

    match cli.command {
        Command::Engine(args) => match args.command {
            EngineCommand::Status(output) => {
                let status = client.engine().status().await?;
                emit_read(stdout, output.output, &status, || {
                    Ok(render_engine_status(&status))
                })
            }
            EngineCommand::Livez(output) => {
                let probe = client.engine().livez().await?;
                emit_read(stdout, output.output, &probe, || {
                    Ok(render_engine_probe(&probe))
                })
            }
            EngineCommand::Readyz(output) => {
                let probe = client.engine().readyz().await?;
                emit_read(stdout, output.output, &probe, || {
                    Ok(render_engine_probe(&probe))
                })
            }
        },
        Command::Groups(args) => match args.command {
            GroupsCommand::Status(output) => {
                let status = client.groups().status().await?;
                emit_read(stdout, output.output, &status, || {
                    Ok(render_groups_status(&status))
                })
            }
            GroupsCommand::Shutdown(args) => {
                let response = client
                    .groups()
                    .shutdown(&operation_options(args.wait, args.wait_timeout))
                    .await?;
                emit_read(stdout, args.output.output, &response, || {
                    Ok(render_groups_shutdown(&response))
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
                    render_pipeline_details(&details)
                })
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
                    Ok(render_pipeline_status(&status))
                })
            }
            PipelinesCommand::Livez(args) => {
                let probe = client
                    .pipelines()
                    .livez(&args.target.pipeline_group_id, &args.target.pipeline_id)
                    .await?;
                emit_read(stdout, args.output.output, &probe, || {
                    Ok(render_pipeline_probe(&probe))
                })
            }
            PipelinesCommand::Readyz(args) => {
                let probe = client
                    .pipelines()
                    .readyz(&args.target.pipeline_group_id, &args.target.pipeline_id)
                    .await?;
                emit_read(stdout, args.output.output, &probe, || {
                    Ok(render_pipeline_probe(&probe))
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
                            Ok(render_rollout_status(&status))
                        })?;
                        if args.watch {
                            let rollout_id = status.rollout_id.clone();
                            watch_rollout(
                                &client,
                                stdout,
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
                            Ok(render_rollout_status(&status))
                        })
                    }
                    otap_df_admin_api::pipelines::ReconfigureOutcome::Failed(status) => {
                        emit_mutation(stdout, args.output, "failed", &status, || {
                            Ok(render_rollout_status(&status))
                        })?;
                        Err(CliError::outcome_failure(format!(
                            "pipeline rollout '{}' failed",
                            status.rollout_id
                        )))
                    }
                    otap_df_admin_api::pipelines::ReconfigureOutcome::TimedOut(status) => {
                        emit_mutation(stdout, args.output, "timed_out", &status, || {
                            Ok(render_rollout_status(&status))
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
                            Ok(render_shutdown_status(&status))
                        })?;
                        if args.watch {
                            let shutdown_id = status.shutdown_id.clone();
                            watch_shutdown(
                                &client,
                                stdout,
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
                            Ok(render_shutdown_status(&status))
                        })
                    }
                    otap_df_admin_api::pipelines::ShutdownOutcome::Failed(status) => {
                        emit_mutation(stdout, args.output, "failed", &status, || {
                            Ok(render_shutdown_status(&status))
                        })?;
                        Err(CliError::outcome_failure(format!(
                            "pipeline shutdown '{}' failed",
                            status.shutdown_id
                        )))
                    }
                    otap_df_admin_api::pipelines::ShutdownOutcome::TimedOut(status) => {
                        emit_mutation(stdout, args.output, "timed_out", &status, || {
                            Ok(render_shutdown_status(&status))
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
                    let logs = client
                        .telemetry()
                        .logs(&LogsQuery {
                            after: args.after,
                            limit: args.limit,
                        })
                        .await?;
                    let Some(logs) = logs else {
                        return Err(CliError::not_found(
                            "retained admin logs are not available on this engine",
                        ));
                    };
                    emit_read(stdout, args.output.output, &logs, || Ok(render_logs(&logs)))
                }
                LogsCommand::Watch(args) => {
                    watch_logs(
                        &client,
                        stdout,
                        args.after,
                        args.tail,
                        args.limit,
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
                            let metrics = client.telemetry().metrics_compact(&options).await?;
                            emit_read(stdout, args.output.output, &metrics, || {
                                Ok(render_metrics_compact(&metrics))
                            })
                        }
                        MetricsShape::Full => {
                            let metrics = client.telemetry().metrics(&options).await?;
                            emit_read(stdout, args.output.output, &metrics, || {
                                Ok(render_metrics_full(&metrics))
                            })
                        }
                    }
                }
                MetricsCommand::Watch(args) => {
                    watch_metrics(
                        &client,
                        stdout,
                        args.shape,
                        MetricsOptions {
                            reset: args.reset,
                            keep_all_zeroes: args.keep_all_zeroes,
                        },
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
    pipeline_group_id: &str,
    pipeline_id: &str,
    rollout_id: &str,
    output: ReadOutput,
) -> Result<(), CliError> {
    let status = client
        .pipelines()
        .rollout_status(pipeline_group_id, pipeline_id, rollout_id)
        .await?;
    let Some(status) = status else {
        return Err(CliError::not_found(format!(
            "rollout '{}' for pipeline '{}/{}' was not found",
            rollout_id, pipeline_group_id, pipeline_id
        )));
    };
    emit_read(stdout, output, &status, || {
        Ok(render_rollout_status(&status))
    })
}

async fn get_shutdown(
    client: &AdminClient,
    stdout: &mut dyn Write,
    pipeline_group_id: &str,
    pipeline_id: &str,
    shutdown_id: &str,
    output: ReadOutput,
) -> Result<(), CliError> {
    let status = client
        .pipelines()
        .shutdown_status(pipeline_group_id, pipeline_id, shutdown_id)
        .await?;
    let Some(status) = status else {
        return Err(CliError::not_found(format!(
            "shutdown '{}' for pipeline '{}/{}' was not found",
            shutdown_id, pipeline_group_id, pipeline_id
        )));
    };
    emit_read(stdout, output, &status, || {
        Ok(render_shutdown_status(&status))
    })
}

async fn watch_rollout(
    client: &AdminClient,
    stdout: &mut dyn Write,
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
                || Ok(render_rollout_status(&current)),
                &current,
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
                || Ok(render_shutdown_status(&current)),
                &current,
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
    after: Option<u64>,
    tail: Option<usize>,
    limit: Option<usize>,
    interval: Duration,
    output: StreamOutput,
) -> Result<(), CliError> {
    let mut cursor = after;

    if after.is_none() {
        if let Some(tail) = tail {
            let response = client
                .telemetry()
                .logs(&LogsQuery {
                    after: None,
                    limit: Some(tail),
                })
                .await?
                .ok_or_else(|| {
                    CliError::not_found("retained admin logs are not available on this engine")
                })?;
            emit_logs(stdout, output, &response.logs)?;
            cursor = Some(response.next_seq);
        } else {
            let response = client
                .telemetry()
                .logs(&LogsQuery {
                    after: None,
                    limit: Some(1),
                })
                .await?
                .ok_or_else(|| {
                    CliError::not_found("retained admin logs are not available on this engine")
                })?;
            cursor = Some(response.next_seq);
        }
    }

    loop {
        let response = client
            .telemetry()
            .logs(&LogsQuery {
                after: cursor,
                limit,
            })
            .await?
            .ok_or_else(|| {
                CliError::not_found("retained admin logs are not available on this engine")
            })?;
        emit_logs(stdout, output, &response.logs)?;
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
    shape: MetricsShape,
    options: MetricsOptions,
    interval: Duration,
    output: StreamOutput,
) -> Result<(), CliError> {
    loop {
        match shape {
            MetricsShape::Compact => {
                let metrics = client.telemetry().metrics_compact(&options).await?;
                write_stream_snapshot(
                    stdout,
                    output,
                    "telemetry_metrics",
                    || Ok(render_metrics_compact(&metrics)),
                    &metrics,
                )?;
            }
            MetricsShape::Full => {
                let metrics = client.telemetry().metrics(&options).await?;
                write_stream_snapshot(
                    stdout,
                    output,
                    "telemetry_metrics",
                    || Ok(render_metrics_full(&metrics)),
                    &metrics,
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
    output: StreamOutput,
    logs: &[telemetry::LogEntry],
) -> Result<(), CliError> {
    for entry in logs {
        match output {
            StreamOutput::Human => write_human(
                stdout,
                &format!(
                    "{} [{}] {} {}",
                    entry.timestamp, entry.level, entry.target, entry.rendered
                ),
            )?,
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

fn load_pipeline_config(
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

    let parse_result = if looks_like_json(&content) {
        PipelineConfig::from_json(
            pipeline_group_id.to_string().into(),
            pipeline_id.to_string().into(),
            &content,
        )
    } else {
        PipelineConfig::from_yaml(
            pipeline_group_id.to_string().into(),
            pipeline_id.to_string().into(),
            &content,
        )
    };

    parse_result.map_err(|err| {
        CliError::config(format!(
            "failed to parse pipeline config for '{}/{}': {err}",
            pipeline_group_id, pipeline_id
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
            "df_enginectl",
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
            "df_enginectl",
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
            "df_enginectl",
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
            "df_enginectl",
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
            "df_enginectl",
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
            "df_enginectl",
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
                None,
                Some(2),
                None,
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
