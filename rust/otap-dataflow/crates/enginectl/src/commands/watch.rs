// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Long-lived watch loops for events, logs, metrics, and operation status.
//!
//! Watch commands are optimized for long-running interactive sessions and
//! automation streams. This module centralizes polling, tailing, de-duplication,
//! Ctrl-C handling, and stream emission so event, log, metric, rollout, and
//! shutdown watches share the same liveness and output behavior.

use crate::args::{MetricsShape, StreamOutput};
use crate::commands::fetch::{fetch_logs, fetch_pipeline_status, fetch_rollout, fetch_shutdown};
use crate::commands::output::{duration_to_admin_timeout_secs, write_human_stream_line};
use crate::error::CliError;
use crate::render::{
    render_event_line, render_group_shutdown_watch, render_metrics_compact, render_metrics_full,
    render_rollout_status, render_shutdown_status, write_event_output, write_log_event,
    write_stream_snapshot,
};
use crate::style::HumanStyle;
use crate::troubleshoot::{
    EventFilters, LogFilters, MetricsFilters, NormalizedEvent, extract_events_from_group_status,
    extract_events_from_pipeline_status, filter_logs, filter_metrics_compact, filter_metrics_full,
    group_shutdown_snapshot, tail_events,
};
use otap_df_admin_api::telemetry::MetricsOptions;
use otap_df_admin_api::{AdminClient, telemetry};
use std::collections::{HashSet, VecDeque};
use std::io::Write;
use std::time::{Duration, SystemTime};

/// Watch group-scoped recent events by polling group status.
pub(crate) async fn watch_group_events(
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
    let mut seen = SeenEvents::default();
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

/// Watch one pipeline's recent events by polling pipeline status.
pub(crate) async fn watch_pipeline_events(
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
    let mut seen = SeenEvents::default();
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

/// Watch the client-side groups shutdown heuristic until every pipeline turns terminal.
pub(crate) async fn watch_groups_shutdown(
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
                duration_to_admin_timeout_secs(wait_timeout)
            )));
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => return Ok(()),
            _ = tokio::time::sleep(interval) => {}
        }
    }
}

/// Watch a rollout resource until it reaches a terminal state.
pub(crate) async fn watch_rollout(
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
        fetch_rollout(client, pipeline_group_id, pipeline_id, rollout_id).await?
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

        current = fetch_rollout(client, pipeline_group_id, pipeline_id, rollout_id).await?;
    }
}

/// Watch a shutdown resource until it reaches a terminal state.
pub(crate) async fn watch_shutdown(
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
        fetch_shutdown(client, pipeline_group_id, pipeline_id, shutdown_id).await?
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

        current = fetch_shutdown(client, pipeline_group_id, pipeline_id, shutdown_id).await?;
    }
}

/// Watch retained logs starting from the current cursor strategy.
pub(crate) async fn watch_logs(
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

/// Watch telemetry metrics in either compact or full shape.
pub(crate) async fn watch_metrics(
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

fn emit_events(
    stdout: &mut dyn Write,
    human_style: HumanStyle,
    output: StreamOutput,
    events: &[NormalizedEvent],
    seen: &mut SeenEvents,
) -> Result<(), CliError> {
    for event in events {
        let identity = event.identity_key();
        if !seen.insert(identity) {
            continue;
        }
        match output {
            StreamOutput::Human => {
                write_human_stream_line(stdout, &render_event_line(&human_style, event))?
            }
            StreamOutput::Ndjson => write_event_output(stdout, "event", event)?,
        }
    }
    Ok(())
}

#[derive(Debug)]
struct SeenEvents {
    order: VecDeque<String>,
    keys: HashSet<String>,
    capacity: usize,
}

impl Default for SeenEvents {
    fn default() -> Self {
        Self {
            order: VecDeque::new(),
            keys: HashSet::new(),
            capacity: 4096,
        }
    }
}

impl SeenEvents {
    fn insert(&mut self, key: String) -> bool {
        if self.keys.contains(&key) {
            return false;
        }

        _ = self.keys.insert(key.clone());
        self.order.push_back(key);
        while self.order.len() > self.capacity {
            if let Some(oldest) = self.order.pop_front() {
                _ = self.keys.remove(&oldest);
            }
        }
        true
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
            StreamOutput::Human => write_human_stream_line(
                stdout,
                &crate::render::render_log_line(&human_style, entry),
            )?,
            StreamOutput::Ndjson => write_log_event(stdout, entry)?,
        }
    }
    Ok(())
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
