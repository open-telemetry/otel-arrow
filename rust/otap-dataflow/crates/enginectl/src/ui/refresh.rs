// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Refresh, polling, and UI command-context helpers.
//!
//! Refresh code fetches admin API snapshots and merges them into a new
//! `AppState` for the event loop to display. It also builds the redacted command
//! context used by the equivalent-CLI overlay and extracts engine vitals from
//! telemetry, keeping network polling separate from input and rendering.

use super::*;
use crate::BIN_NAME;

/// Builds the redacted CLI prefix used by the TUI equivalent-command overlay.
pub(crate) fn build_command_context(
    settings: &HttpAdminClientSettings,
    color: ColorChoice,
    args: &UiArgs,
) -> UiCommandContext {
    let target_url = canonical_target_url(&settings.endpoint);
    let mut prefix_args = vec![
        BIN_NAME.to_string(),
        "--url".to_string(),
        target_url.clone(),
    ];
    let mut sensitive_args_redacted = false;
    let defaults = HttpAdminClientSettings::new(settings.endpoint.clone());

    if color != ColorChoice::Auto {
        prefix_args.push("--color".to_string());
        prefix_args.push(match color {
            ColorChoice::Auto => "auto".to_string(),
            ColorChoice::Always => "always".to_string(),
            ColorChoice::Never => "never".to_string(),
        });
    }
    if settings.connect_timeout != defaults.connect_timeout {
        prefix_args.push("--connect-timeout".to_string());
        prefix_args.push(format_duration(settings.connect_timeout).to_string());
    }
    if settings.timeout != defaults.timeout {
        if let Some(timeout) = settings.timeout {
            prefix_args.push("--request-timeout".to_string());
            prefix_args.push(format_duration(timeout).to_string());
        }
    }
    if settings.tcp_nodelay != defaults.tcp_nodelay {
        prefix_args.push("--tcp-nodelay".to_string());
        prefix_args.push(settings.tcp_nodelay.to_string());
    }
    if settings.tcp_keepalive != defaults.tcp_keepalive {
        if let Some(keepalive) = settings.tcp_keepalive {
            prefix_args.push("--tcp-keepalive".to_string());
            prefix_args.push(format_duration(keepalive).to_string());
        }
    }
    if settings.tcp_keepalive_interval != defaults.tcp_keepalive_interval {
        if let Some(interval) = settings.tcp_keepalive_interval {
            prefix_args.push("--tcp-keepalive-interval".to_string());
            prefix_args.push(format_duration(interval).to_string());
        }
    }
    if let Some(tls) = &settings.tls {
        if let Some(path) = &tls.ca_file {
            prefix_args.push("--ca-file".to_string());
            prefix_args.push(path.display().to_string());
        }
        if let Some(path) = &tls.config.cert_file {
            prefix_args.push("--client-cert-file".to_string());
            prefix_args.push(path.display().to_string());
        }
        if tls.config.key_file.is_some() {
            prefix_args.push("--client-key-file".to_string());
            prefix_args.push("<client-key-file>".to_string());
            sensitive_args_redacted = true;
        }
        if tls.include_system_ca_certs_pool == Some(false) {
            prefix_args.push("--include-system-ca-certs".to_string());
            prefix_args.push("false".to_string());
        }
        if tls.insecure_skip_verify == Some(true) {
            prefix_args.push("--insecure-skip-verify".to_string());
            prefix_args.push("true".to_string());
        }
    }

    UiCommandContext {
        target_url,
        prefix_args,
        sensitive_args_redacted,
        refresh_interval: args.refresh_interval,
        logs_tail: args.logs_tail,
    }
}

fn canonical_target_url(endpoint: &otap_df_admin_api::AdminEndpoint) -> String {
    let mut url = format!(
        "{}://{}:{}",
        endpoint.scheme.as_str(),
        endpoint.host,
        endpoint.port
    );
    if let Some(base_path) = &endpoint.base_path {
        url.push_str(base_path.trim_end_matches('/'));
    }
    url
}

/// Refreshes the active TUI view and records any user-visible refresh error.
pub(super) async fn refresh_view(client: &AdminClient, app: &mut AppState, args: &UiArgs) {
    let compact_metrics = client
        .telemetry()
        .metrics_compact(&telemetry::MetricsOptions::default())
        .await;
    let compact_metrics_error = compact_metrics.as_ref().err().map(ToString::to_string);
    update_engine_vitals(
        app,
        compact_metrics.as_ref().ok(),
        compact_metrics_error.as_deref(),
    );

    let result = match app.view {
        View::Pipelines => {
            refresh_pipelines_view(
                client,
                app,
                args,
                compact_metrics.as_ref().ok(),
                compact_metrics_error.as_deref(),
            )
            .await
        }
        View::Groups => {
            refresh_groups_view(
                client,
                app,
                args,
                compact_metrics.as_ref().ok(),
                compact_metrics_error.as_deref(),
            )
            .await
        }
        View::Engine => {
            refresh_engine_view(
                client,
                app,
                args,
                compact_metrics.as_ref().ok(),
                compact_metrics_error.as_deref(),
            )
            .await
        }
    };

    match result {
        Ok(()) => {
            app.last_refresh = Some(std::time::Instant::now());
            app.last_error = None;
        }
        Err(err) => {
            app.last_error = Some(err.to_string());
        }
    }
}

async fn refresh_pipelines_view(
    client: &AdminClient,
    app: &mut AppState,
    args: &UiArgs,
    compact_metrics: Option<&telemetry::CompactMetricsResponse>,
    compact_metrics_error: Option<&str>,
) -> Result<(), CliError> {
    let groups_status = client.groups().status().await?;
    app.groups_status = Some(groups_status);
    app.ensure_selection();

    let Some((pipeline_group_id, pipeline_id)) = app.selected_pipeline_target() else {
        app.pipelines.clear();
        return Ok(());
    };
    let target_key = format!("{pipeline_group_id}:{pipeline_id}");
    if app.pipelines.target_key.as_deref() != Some(target_key.as_str()) {
        app.pipelines.reset(target_key.clone());
    }

    let describe = crate::fetch_pipeline_describe(client, &pipeline_group_id, &pipeline_id).await?;
    app.pipelines.describe = Some(describe.clone());
    let header = pipeline_header(&describe);
    let config_yaml = serialize_pipeline_config_yaml(&describe.details.pipeline)?;
    app.pipelines.config_yaml = Some(config_yaml.clone());
    let rollout_status = refresh_active_rollout(client, app, &describe).await?;
    let shutdown_status =
        refresh_active_shutdown(client, app, &pipeline_group_id, &pipeline_id).await?;

    app.pipelines.rollout = build_rollout_pane(&describe, rollout_status.as_ref());
    app.pipelines.shutdown = build_shutdown_pane(&describe, shutdown_status.as_ref());
    app.pipelines.config = build_config_pane(
        &header,
        &pipeline_group_id,
        &pipeline_id,
        &config_yaml,
        app.pipelines.config_draft.as_ref(),
    );

    app.pipelines.summary = build_pipeline_summary_pane(&describe, header.clone());
    app.pipelines.details = build_pipeline_details_pane(
        &describe,
        header.clone(),
        rollout_status.as_ref(),
        shutdown_status.as_ref(),
    );
    app.pipelines.events = EventPane {
        header: Some(add_header_chip(
            header.clone(),
            chip(
                "events",
                describe.recent_events.len().to_string(),
                Tone::Muted,
            ),
        )),
        rows: event_rows(&describe.recent_events),
        empty_message: "No recent events for the selected pipeline.".to_string(),
    };

    match app.pipeline_tab {
        PipelineTab::Summary
        | PipelineTab::Details
        | PipelineTab::Config
        | PipelineTab::Events
        | PipelineTab::Rollout
        | PipelineTab::Shutdown => {}
        PipelineTab::Logs => {
            let filters = LogFilters {
                pipeline_group_id: Some(pipeline_group_id.clone()),
                pipeline_id: Some(pipeline_id.clone()),
                ..LogFilters::default()
            };
            refresh_log_feed(
                client,
                &mut app.pipelines.logs,
                &format!("pipeline:{target_key}"),
                filters,
                args.logs_tail,
            )
            .await?;
            app.pipelines.logs.header = Some(add_header_chip(
                header.clone(),
                chip(
                    "logs",
                    app.pipelines.logs.rows.len().to_string(),
                    if app.pipelines.logs.rows.is_empty() {
                        Tone::Muted
                    } else {
                        Tone::Accent
                    },
                ),
            ));
        }
        PipelineTab::Metrics => {
            let metrics = require_filtered_metrics(
                compact_metrics,
                compact_metrics_error,
                &MetricsFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..MetricsFilters::default()
                },
            )?;
            app.pipelines.metrics = build_metrics_pane(
                metrics,
                add_header_chip(header.clone(), chip("scope", "pipeline", Tone::Muted)),
            );
        }
        PipelineTab::Diagnose => {
            let logs = filter_logs(
                &crate::fetch_logs(client, None, Some(args.logs_tail)).await?,
                &LogFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..LogFilters::default()
                },
            );
            let metrics = require_filtered_metrics(
                compact_metrics,
                compact_metrics_error,
                &MetricsFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..MetricsFilters::default()
                },
            )?;
            let diagnosis = if let Some(rollout_status) = rollout_status.as_ref() {
                diagnose_pipeline_rollout(&describe, Some(rollout_status), &logs, &metrics)
            } else {
                diagnose_pipeline_shutdown(&describe, shutdown_status.as_ref(), &logs, &metrics)
            };
            app.pipelines.diagnosis = build_diagnosis_pane(
                diagnosis,
                add_header_chip(header.clone(), chip("scope", "pipeline", Tone::Muted)),
            );
        }
        PipelineTab::Bundle => {
            let logs = filter_logs(
                &crate::fetch_logs(client, None, Some(args.logs_tail)).await?,
                &LogFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..LogFilters::default()
                },
            );
            let metrics = require_filtered_metrics(
                compact_metrics,
                compact_metrics_error,
                &MetricsFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..MetricsFilters::default()
                },
            )?;
            let diagnosis = if let Some(status) = rollout_status.as_ref() {
                diagnose_pipeline_rollout(&describe, Some(status), &logs, &metrics)
            } else {
                diagnose_pipeline_shutdown(&describe, shutdown_status.as_ref(), &logs, &metrics)
            };
            let bundle = PipelineBundle {
                metadata: crate::build_bundle_metadata(args.logs_tail, MetricsShape::Compact),
                describe,
                diagnosis,
                rollout_status,
                shutdown_status,
                logs,
                metrics: BundleMetrics::Compact(metrics),
            };
            app.pipelines.bundle = build_bundle_pane(
                "Pipeline Bundle",
                Some(format!("{pipeline_group_id}/{pipeline_id}")),
                &bundle.metadata,
                serialize_pretty_json(&bundle)?,
            );
        }
    }

    Ok(())
}

async fn refresh_groups_view(
    client: &AdminClient,
    app: &mut AppState,
    args: &UiArgs,
    compact_metrics: Option<&telemetry::CompactMetricsResponse>,
    compact_metrics_error: Option<&str>,
) -> Result<(), CliError> {
    let groups_status = client.groups().status().await?;
    app.groups_status = Some(groups_status);
    app.ensure_selection();

    let Some(group_id) = app.selected_group_id() else {
        app.groups.clear();
        return Ok(());
    };
    if app.groups.group_id.as_deref() != Some(group_id.as_str()) {
        app.groups.reset(group_id.clone());
    }

    let subset = selected_group_status(
        app.groups_status
            .as_ref()
            .expect("groups status should be set"),
        &group_id,
    );
    let describe = describe_groups(subset.clone());
    let header = group_header(&group_id, &describe);
    app.groups.shutdown = build_group_shutdown_pane(
        &group_id,
        &subset,
        &header,
        app.groups.active_shutdown.as_ref(),
    );

    app.groups.summary = build_group_summary_pane(&group_id, &describe, header.clone());
    app.groups.details = build_group_details_pane(&group_id, &describe, header.clone());
    let events = extract_events_from_group_status(
        &subset,
        Some(&EventFilters {
            pipeline_group_id: Some(group_id.clone()),
            ..EventFilters::default()
        }),
    );
    app.groups.events = EventPane {
        header: Some(add_header_chip(
            header.clone(),
            chip("events", events.len().to_string(), Tone::Muted),
        )),
        rows: event_rows(&events),
        empty_message: "No recent events for the selected group.".to_string(),
    };

    match app.group_tab {
        GroupTab::Summary | GroupTab::Details | GroupTab::Events | GroupTab::Shutdown => {}
        GroupTab::Logs => {
            refresh_log_feed(
                client,
                &mut app.groups.logs,
                &format!("group:{group_id}"),
                LogFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..LogFilters::default()
                },
                args.logs_tail,
            )
            .await?;
            app.groups.logs.header = Some(add_header_chip(
                header.clone(),
                chip(
                    "logs",
                    app.groups.logs.rows.len().to_string(),
                    if app.groups.logs.rows.is_empty() {
                        Tone::Muted
                    } else {
                        Tone::Accent
                    },
                ),
            ));
        }
        GroupTab::Metrics => {
            let metrics = require_filtered_metrics(
                compact_metrics,
                compact_metrics_error,
                &MetricsFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..MetricsFilters::default()
                },
            )?;
            app.groups.metrics = build_metrics_pane(
                metrics,
                add_header_chip(header.clone(), chip("scope", "group", Tone::Muted)),
            );
        }
        GroupTab::Diagnose => {
            let logs = filter_logs(
                &crate::fetch_logs(client, None, Some(args.logs_tail)).await?,
                &LogFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..LogFilters::default()
                },
            );
            let metrics = require_filtered_metrics(
                compact_metrics,
                compact_metrics_error,
                &MetricsFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..MetricsFilters::default()
                },
            )?;
            let diagnosis = diagnose_group_shutdown(&subset, &logs, &metrics);
            app.groups.diagnosis = build_diagnosis_pane(
                diagnosis,
                add_header_chip(header.clone(), chip("scope", "group", Tone::Muted)),
            );
        }
        GroupTab::Bundle => {
            let logs = filter_logs(
                &crate::fetch_logs(client, None, Some(args.logs_tail)).await?,
                &LogFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..LogFilters::default()
                },
            );
            let metrics = require_filtered_metrics(
                compact_metrics,
                compact_metrics_error,
                &MetricsFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..MetricsFilters::default()
                },
            )?;
            let diagnosis = diagnose_group_shutdown(&subset, &logs, &metrics);
            let bundle = GroupsBundle {
                metadata: crate::build_bundle_metadata(args.logs_tail, MetricsShape::Compact),
                describe,
                diagnosis,
                logs,
                metrics: BundleMetrics::Compact(metrics),
            };
            app.groups.bundle = build_bundle_pane(
                "Group Bundle",
                Some(group_id.clone()),
                &bundle.metadata,
                serialize_pretty_json(&bundle)?,
            );
        }
    }

    Ok(())
}

async fn refresh_engine_view(
    client: &AdminClient,
    app: &mut AppState,
    args: &UiArgs,
    compact_metrics: Option<&telemetry::CompactMetricsResponse>,
    compact_metrics_error: Option<&str>,
) -> Result<(), CliError> {
    let status = client.engine().status().await?;
    let livez = client.engine().livez().await?;
    let readyz = client.engine().readyz().await?;
    app.engine_status = Some(status.clone());
    app.engine_livez = Some(livez.clone());
    app.engine_readyz = Some(readyz.clone());
    app.ensure_selection();

    let header = engine_header(&status, &livez, &readyz);
    app.engine.summary = build_engine_summary_pane(&status, &livez, &readyz, header.clone());
    app.engine.details =
        build_engine_details_pane(&status, &livez, &readyz, &app.engine_vitals, header.clone());

    match app.engine_tab {
        EngineTab::Summary | EngineTab::Details => {}
        EngineTab::Logs => {
            refresh_log_feed(
                client,
                &mut app.engine.logs,
                "engine",
                LogFilters::default(),
                args.logs_tail,
            )
            .await?;
            app.engine.logs.header = Some(add_header_chip(
                header.clone(),
                chip(
                    "logs",
                    app.engine.logs.rows.len().to_string(),
                    if app.engine.logs.rows.is_empty() {
                        Tone::Muted
                    } else {
                        Tone::Accent
                    },
                ),
            ));
        }
        EngineTab::Metrics => {
            let metrics = require_compact_metrics(compact_metrics, compact_metrics_error)?;
            app.engine.metrics = build_metrics_pane(
                metrics.clone(),
                add_header_chip(header, chip("scope", "engine", Tone::Muted)),
            );
        }
    }

    Ok(())
}

async fn refresh_log_feed(
    client: &AdminClient,
    feed: &mut LogFeedState,
    scope_key: &str,
    filters: LogFilters,
    logs_tail: usize,
) -> Result<(), CliError> {
    if feed.scope_key.as_deref() != Some(scope_key) {
        feed.reset(scope_key.to_string());
    }

    if feed.next_seq.is_none() {
        let response = crate::fetch_logs(client, None, Some(logs_tail)).await?;
        let filtered = trim_logs_response(filter_logs(&response, &filters), logs_tail);
        feed.next_seq = Some(response.next_seq);
        feed.response = Some(filtered.clone());
        feed.rows = log_rows(&filtered.logs);
        feed.empty_message = if filtered.logs.is_empty() {
            "No retained logs match the current scope.".to_string()
        } else {
            String::new()
        };
        return Ok(());
    }

    let response = crate::fetch_logs(client, feed.next_seq, Some(logs_tail)).await?;
    let filtered = filter_logs(&response, &filters);
    feed.next_seq = Some(response.next_seq);
    if let Some(existing) = &mut feed.response {
        existing.oldest_seq = existing
            .logs
            .first()
            .map(|entry| entry.seq)
            .or(filtered.oldest_seq);
        existing.newest_seq = filtered.newest_seq;
        existing.next_seq = response.next_seq;
        existing.truncated_before_seq = filtered.truncated_before_seq;
        existing.dropped_on_ingest = filtered.dropped_on_ingest;
        existing.dropped_on_retention = filtered.dropped_on_retention;
        existing.retained_bytes = filtered.retained_bytes;
        existing.logs.extend(filtered.logs);
        if existing.logs.len() > logs_tail {
            let drain = existing.logs.len() - logs_tail;
            let _ = existing.logs.drain(0..drain);
        }
        feed.rows = log_rows(&existing.logs);
        feed.empty_message = if existing.logs.is_empty() {
            "No retained logs match the current scope.".to_string()
        } else {
            String::new()
        };
    } else {
        let filtered = trim_logs_response(filtered, logs_tail);
        feed.rows = log_rows(&filtered.logs);
        feed.empty_message = if filtered.logs.is_empty() {
            "No retained logs match the current scope.".to_string()
        } else {
            String::new()
        };
        feed.response = Some(filtered);
    }

    Ok(())
}

/// Updates the header vitals rail from compact metrics or marks existing data stale.
pub(super) fn update_engine_vitals(
    app: &mut AppState,
    compact_metrics: Option<&telemetry::CompactMetricsResponse>,
    compact_metrics_error: Option<&str>,
) {
    if let Some(metrics) = compact_metrics {
        app.engine_vitals = extract_engine_vitals(metrics);
    } else {
        app.engine_vitals.stale = true;
        if app.engine_vitals.pressure_detail.is_none() {
            app.engine_vitals.pressure_detail =
                compact_metrics_error.map(|error| format!("metrics unavailable: {error}"));
        }
    }
}

/// Extracts engine CPU, memory, and pressure values from compact metrics.
pub(super) fn extract_engine_vitals(metrics: &telemetry::CompactMetricsResponse) -> EngineVitals {
    let Some(metric_set) = metrics.metric_sets.iter().find(|set| set.name == "engine") else {
        return EngineVitals {
            stale: false,
            pressure_detail: Some("engine set not present".to_string()),
            ..EngineVitals::default()
        };
    };

    let cpu = metric_value_f64_any(metric_set, &["cpu.utilization", "cpu_utilization"])
        .map(|value| format!("{:.1}%", value.clamp(0.0, 1.0) * 100.0))
        .unwrap_or_else(|| "n/a".to_string());
    let rss = metric_value_u64_any(metric_set, &["memory.rss", "memory_rss"])
        .map(format_bytes_iec)
        .unwrap_or_else(|| "n/a".to_string());
    let pressure_raw = metric_value_u64_any(
        metric_set,
        &["memory.pressure.state", "memory_pressure_state"],
    );
    let (pressure_state, pressure_tone) = match pressure_raw {
        Some(0) => ("normal".to_string(), Tone::Success),
        Some(1) => ("soft".to_string(), Tone::Warning),
        Some(2) => ("hard".to_string(), Tone::Failure),
        Some(value) => (format!("unknown({value})"), Tone::Muted),
        None => ("n/a".to_string(), Tone::Muted),
    };
    let pressure_detail = match (
        metric_value_u64_any(
            metric_set,
            &["process.memory.usage.bytes", "process_memory_usage_bytes"],
        ),
        metric_value_u64_any(
            metric_set,
            &[
                "process.memory.soft.limit.bytes",
                "process_memory_soft_limit_bytes",
            ],
        ),
        metric_value_u64_any(
            metric_set,
            &[
                "process.memory.hard.limit.bytes",
                "process_memory_hard_limit_bytes",
            ],
        ),
    ) {
        (Some(0), Some(0), Some(0)) => None,
        (Some(usage), Some(soft), Some(hard)) => Some(format!(
            "usage/limits {} / {} / {}",
            format_bytes_iec(usage),
            format_bytes_iec(soft),
            format_bytes_iec(hard)
        )),
        (Some(usage), _, _) => Some(format!("usage {}", format_bytes_iec(usage))),
        _ => None,
    };

    EngineVitals {
        cpu_utilization: cpu,
        cpu_tone: Tone::Accent,
        memory_rss: rss,
        memory_tone: Tone::Accent,
        pressure_state,
        pressure_tone,
        pressure_detail,
        stale: false,
    }
}

fn require_compact_metrics<'a>(
    compact_metrics: Option<&'a telemetry::CompactMetricsResponse>,
    compact_metrics_error: Option<&str>,
) -> Result<&'a telemetry::CompactMetricsResponse, CliError> {
    compact_metrics.ok_or_else(|| CliError::Message {
        exit_code: 6,
        message: compact_metrics_error
            .map(|error| format!("failed to fetch compact metrics: {error}"))
            .unwrap_or_else(|| "failed to fetch compact metrics".to_string()),
        print: true,
    })
}

fn require_filtered_metrics(
    compact_metrics: Option<&telemetry::CompactMetricsResponse>,
    compact_metrics_error: Option<&str>,
    filters: &MetricsFilters,
) -> Result<telemetry::CompactMetricsResponse, CliError> {
    Ok(filter_metrics_compact(
        require_compact_metrics(compact_metrics, compact_metrics_error)?,
        filters,
    ))
}

fn metric_value_u64(metric_set: &telemetry::MetricSet, name: &str) -> Option<u64> {
    match metric_set.metrics.get(name) {
        Some(telemetry::MetricValue::U64(value)) => Some(*value),
        Some(telemetry::MetricValue::F64(value)) => Some(value.max(0.0) as u64),
        Some(telemetry::MetricValue::Mmsc(_)) | None => None,
    }
}

fn metric_value_f64(metric_set: &telemetry::MetricSet, name: &str) -> Option<f64> {
    match metric_set.metrics.get(name) {
        Some(telemetry::MetricValue::F64(value)) => Some(*value),
        Some(telemetry::MetricValue::U64(value)) => Some(*value as f64),
        Some(telemetry::MetricValue::Mmsc(_)) | None => None,
    }
}

fn metric_value_u64_any(metric_set: &telemetry::MetricSet, names: &[&str]) -> Option<u64> {
    names
        .iter()
        .find_map(|name| metric_value_u64(metric_set, name))
}

fn metric_value_f64_any(metric_set: &telemetry::MetricSet, names: &[&str]) -> Option<f64> {
    names
        .iter()
        .find_map(|name| metric_value_f64(metric_set, name))
}

fn format_bytes_iec(bytes: u64) -> String {
    const UNITS: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    if bytes < 1024 {
        return format!("{bytes} B");
    }

    let mut value = bytes as f64;
    let mut unit_index = 0usize;
    while value >= 1024.0 && unit_index < UNITS.len() - 1 {
        value /= 1024.0;
        unit_index += 1;
    }

    format!("{value:.1} {}", UNITS[unit_index])
}

async fn refresh_active_rollout(
    client: &AdminClient,
    app: &mut AppState,
    describe: &PipelineDescribeReport,
) -> Result<Option<pipelines::RolloutStatus>, CliError> {
    let rollout_id = app.pipelines.active_rollout_id.clone().or_else(|| {
        describe
            .status
            .rollout
            .as_ref()
            .filter(|rollout| !rollout_is_terminal(rollout.state))
            .map(|rollout| rollout.rollout_id.clone())
    });
    let Some(rollout_id) = rollout_id else {
        app.pipelines.active_rollout_id = None;
        return Ok(None);
    };

    match client
        .pipelines()
        .rollout_status(
            describe.details.pipeline_group_id.as_ref(),
            describe.details.pipeline_id.as_ref(),
            &rollout_id,
        )
        .await?
    {
        Some(status) => {
            if rollout_is_terminal(status.state) {
                app.pipelines.active_rollout_id = None;
            } else {
                app.pipelines.active_rollout_id = Some(status.rollout_id.clone());
            }
            Ok(Some(status))
        }
        None => {
            if describe
                .status
                .rollout
                .as_ref()
                .is_none_or(|rollout| rollout_is_terminal(rollout.state))
            {
                app.pipelines.active_rollout_id = None;
            }
            Ok(None)
        }
    }
}

async fn refresh_active_shutdown(
    client: &AdminClient,
    app: &mut AppState,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<Option<pipelines::ShutdownStatus>, CliError> {
    let Some(shutdown_id) = app.pipelines.active_shutdown_id.clone() else {
        return Ok(None);
    };

    match client
        .pipelines()
        .shutdown_status(pipeline_group_id, pipeline_id, &shutdown_id)
        .await?
    {
        Some(status) => {
            app.pipelines.active_shutdown_id = Some(status.shutdown_id.clone());
            Ok(Some(status))
        }
        None => {
            app.pipelines.active_shutdown_id = None;
            Ok(None)
        }
    }
}

/// Filters a full group status response down to one pipeline group.
pub(super) fn selected_group_status(status: &groups::Status, group_id: &str) -> groups::Status {
    groups::Status {
        generated_at: status.generated_at.clone(),
        pipelines: status
            .pipelines
            .iter()
            .filter(|(key, _)| {
                key.split_once(':')
                    .is_some_and(|(group, _)| group == group_id)
            })
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect(),
    }
}

fn trim_logs_response(
    mut response: telemetry::LogsResponse,
    logs_tail: usize,
) -> telemetry::LogsResponse {
    if response.logs.len() > logs_tail {
        let drain = response.logs.len() - logs_tail;
        let _ = response.logs.drain(0..drain);
    }
    response.oldest_seq = response.logs.first().map(|entry| entry.seq);
    response.newest_seq = response.logs.last().map(|entry| entry.seq);
    response
}

fn serialize_pretty_json(value: &impl Serialize) -> Result<String, CliError> {
    serde_json::to_string_pretty(value)
        .map_err(|err| CliError::config(format!("failed to serialize UI preview: {err}")))
}
