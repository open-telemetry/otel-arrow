// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

mod app;
mod view;

use self::app::{
    AppState, BundlePane, ConditionRow, CoreRow, DetailHeader, DiagnosisPane, EngineSummaryPane,
    EngineTab, EventPane, EvidenceRow, FindingRow, FocusArea, GroupSummaryPane, GroupTab,
    LogFeedState, LogRow, MetricRow, MetricsPane, OperationPane, OperationRow,
    PipelineInventoryRow, PipelineSummaryPane, PipelineTab, ProbeFailureRow, StatCard, StatusChip,
    TimelineRow, Tone, UiCommandContext, View,
};
use self::view::draw_ui;
use crate::args::{ColorChoice, MetricsShape, UiArgs};
use crate::error::CliError;
use crate::style::HumanStyle;
use crate::troubleshoot::{
    BundleMetrics, DiagnosisFinding, DiagnosisReport, DiagnosisStatus, EventFilters,
    EvidenceExcerpt, FindingSeverity, GroupsBundle, GroupsDescribeReport, LogFilters,
    MetricsFilters, NormalizedEvent, NormalizedEventKind, PipelineBundle, PipelineDescribeReport,
    describe_groups, diagnose_group_shutdown, diagnose_pipeline_rollout,
    diagnose_pipeline_shutdown, extract_events_from_group_status, filter_logs,
    filter_metrics_compact,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use humantime::format_duration;
use otap_df_admin_api::{
    AdminClient, HttpAdminClientSettings, engine, groups, pipelines, telemetry,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::io;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;
use tokio::sync::mpsc;

pub(crate) async fn run_ui(
    client: &AdminClient,
    args: UiArgs,
    color: ColorChoice,
    command_context: UiCommandContext,
) -> Result<(), CliError> {
    let mut session = TerminalSession::new()?;
    let mut app = AppState::new(
        args.start_view,
        HumanStyle::resolve(color, true).is_enabled(),
        args.logs_tail,
    );
    app.set_command_context(command_context);

    refresh_view(client, &mut app, &args).await;

    let (tx, mut rx) = mpsc::unbounded_channel();
    let stop = Arc::new(AtomicBool::new(false));
    let event_thread = spawn_event_thread(stop.clone(), tx);

    let mut refresh = tokio::time::interval(args.refresh_interval);
    let _ = refresh.tick().await;

    loop {
        session.draw(&app)?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => break,
            _ = refresh.tick() => {
                refresh_view(client, &mut app, &args).await;
            }
            Some(event) = rx.recv() => {
                match event {
                    UiEvent::Terminal(event) => {
                        match handle_event(event, &mut app) {
                            EventOutcome::Quit => break,
                            EventOutcome::Refresh => refresh_view(client, &mut app, &args).await,
                            EventOutcome::Continue => {}
                        }
                    }
                    UiEvent::TerminalError(error) => {
                        app.last_error = Some(error);
                    }
                }
            }
        }
    }

    stop.store(true, Ordering::Relaxed);
    let _ = event_thread.join();
    Ok(())
}

#[derive(Debug)]
enum UiEvent {
    Terminal(Event),
    TerminalError(String),
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum EventOutcome {
    Continue,
    Refresh,
    Quit,
}

struct TerminalSession {
    terminal: ratatui::Terminal<ratatui::backend::CrosstermBackend<io::Stdout>>,
}

impl TerminalSession {
    fn new() -> Result<Self, io::Error> {
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen)?;
        let backend = ratatui::backend::CrosstermBackend::new(stdout);
        let terminal = ratatui::Terminal::new(backend)?;
        Ok(Self { terminal })
    }

    fn draw(&mut self, app: &AppState) -> Result<(), CliError> {
        let _ = self.terminal.draw(|frame| draw_ui(frame, app))?;
        Ok(())
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = disable_raw_mode();
        let _ = execute!(self.terminal.backend_mut(), LeaveAlternateScreen);
        let _ = self.terminal.show_cursor();
    }
}

fn spawn_event_thread(
    stop: Arc<AtomicBool>,
    tx: mpsc::UnboundedSender<UiEvent>,
) -> thread::JoinHandle<()> {
    thread::spawn(move || {
        while !stop.load(Ordering::Relaxed) {
            match event::poll(Duration::from_millis(100)) {
                Ok(true) => match event::read() {
                    Ok(event) => {
                        if tx.send(UiEvent::Terminal(event)).is_err() {
                            break;
                        }
                    }
                    Err(err) => {
                        let _ = tx.send(UiEvent::TerminalError(err.to_string()));
                        break;
                    }
                },
                Ok(false) => {}
                Err(err) => {
                    let _ = tx.send(UiEvent::TerminalError(err.to_string()));
                    break;
                }
            }
        }
    })
}

fn handle_event(event: Event, app: &mut AppState) -> EventOutcome {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key_event(key, app),
        Event::Resize(_, _) => EventOutcome::Continue,
        _ => EventOutcome::Continue,
    }
}

fn handle_key_event(key: KeyEvent, app: &mut AppState) -> EventOutcome {
    if app.filter_mode {
        return handle_filter_key(key, app);
    }

    if app.show_command_overlay {
        return match key.code {
            KeyCode::Char('q') => EventOutcome::Quit,
            KeyCode::Esc | KeyCode::Char('c') => {
                app.show_command_overlay = false;
                EventOutcome::Continue
            }
            KeyCode::Char('?') => {
                app.show_command_overlay = false;
                app.show_help = true;
                EventOutcome::Continue
            }
            KeyCode::Char('/') => {
                app.show_command_overlay = false;
                app.show_help = false;
                app.start_filter_input();
                EventOutcome::Continue
            }
            _ => EventOutcome::Continue,
        };
    }

    match key.code {
        KeyCode::Char('q') => EventOutcome::Quit,
        KeyCode::Char('?') => {
            app.show_command_overlay = false;
            app.show_help = !app.show_help;
            EventOutcome::Continue
        }
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
                EventOutcome::Continue
            } else if app.show_command_overlay {
                app.show_command_overlay = false;
                EventOutcome::Continue
            } else if !app.filter_query.is_empty() {
                app.clear_filter();
                EventOutcome::Refresh
            } else {
                app.focus = FocusArea::List;
                EventOutcome::Continue
            }
        }
        KeyCode::Char('/') => {
            app.show_command_overlay = false;
            app.show_help = false;
            app.start_filter_input();
            EventOutcome::Continue
        }
        KeyCode::Char('c') => {
            app.show_help = false;
            app.show_command_overlay = !app.show_command_overlay;
            EventOutcome::Continue
        }
        KeyCode::Tab => {
            app.cycle_view(1);
            EventOutcome::Refresh
        }
        KeyCode::BackTab => {
            app.cycle_view(-1);
            EventOutcome::Refresh
        }
        KeyCode::Enter => {
            if app.view == View::Engine && app.focus == FocusArea::List {
                if let Some((group_id, pipeline_id)) = app.selected_engine_pipeline_target() {
                    app.view = View::Pipelines;
                    app.pipeline_selected = Some(format!("{group_id}:{pipeline_id}"));
                    app.focus = FocusArea::Detail;
                    app.reset_scroll();
                    return EventOutcome::Refresh;
                }
            }
            app.focus = FocusArea::Detail;
            app.reset_scroll();
            EventOutcome::Continue
        }
        KeyCode::Backspace => {
            app.focus = FocusArea::List;
            app.reset_scroll();
            EventOutcome::Continue
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.cycle_tab(-1);
            EventOutcome::Refresh
        }
        KeyCode::Right => {
            app.cycle_tab(1);
            EventOutcome::Refresh
        }
        KeyCode::Char('l') if app.focus == FocusArea::Detail => {
            app.cycle_tab(1);
            EventOutcome::Refresh
        }
        KeyCode::Up | KeyCode::Char('k') => handle_vertical_motion(app, -1),
        KeyCode::Down | KeyCode::Char('j') => handle_vertical_motion(app, 1),
        KeyCode::PageUp => handle_vertical_motion(app, -10),
        KeyCode::PageDown => handle_vertical_motion(app, 10),
        KeyCode::Char('g') => {
            app.move_selection_to_edge(false);
            app.reset_scroll();
            EventOutcome::Refresh
        }
        KeyCode::Char('G') => {
            app.move_selection_to_edge(true);
            app.reset_scroll();
            EventOutcome::Refresh
        }
        KeyCode::Char('r') => EventOutcome::Refresh,
        KeyCode::Char('e') => {
            select_detail_tab(app, DetailJump::Events);
            EventOutcome::Refresh
        }
        KeyCode::Char('l') => {
            select_detail_tab(app, DetailJump::Logs);
            EventOutcome::Refresh
        }
        KeyCode::Char('m') => {
            select_detail_tab(app, DetailJump::Metrics);
            EventOutcome::Refresh
        }
        KeyCode::Char('d') => {
            select_detail_tab(app, DetailJump::Diagnose);
            EventOutcome::Refresh
        }
        KeyCode::Char('b') => {
            select_detail_tab(app, DetailJump::Bundle);
            EventOutcome::Refresh
        }
        KeyCode::Char('o') => {
            if app.view == View::Pipelines {
                app.pipeline_tab = PipelineTab::Rollout;
                app.reset_scroll();
                EventOutcome::Refresh
            } else {
                EventOutcome::Continue
            }
        }
        _ => EventOutcome::Continue,
    }
}

fn handle_filter_key(key: KeyEvent, app: &mut AppState) -> EventOutcome {
    match key.code {
        KeyCode::Esc => {
            app.cancel_filter_input();
            EventOutcome::Continue
        }
        KeyCode::Enter => {
            app.apply_filter_input();
            app.reset_scroll();
            EventOutcome::Refresh
        }
        KeyCode::Backspace => {
            let _ = app.filter_input.pop();
            EventOutcome::Continue
        }
        KeyCode::Char(character)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            app.filter_input.push(character);
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}

pub(crate) fn build_command_context(
    settings: &HttpAdminClientSettings,
    color: ColorChoice,
    args: &UiArgs,
) -> UiCommandContext {
    let target_url = canonical_target_url(&settings.endpoint);
    let mut prefix_args = vec!["dfctl".to_string(), "--url".to_string(), target_url.clone()];
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
        if let Some(path) = &tls.config.key_file {
            prefix_args.push("--client-key-file".to_string());
            prefix_args.push(path.display().to_string());
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

fn handle_vertical_motion(app: &mut AppState, delta: isize) -> EventOutcome {
    match app.focus {
        FocusArea::List => {
            app.move_selection(delta);
            app.reset_scroll();
            EventOutcome::Refresh
        }
        FocusArea::Detail => {
            if delta.is_negative() {
                app.detail_scroll = app
                    .detail_scroll
                    .saturating_sub(delta.unsigned_abs() as u16);
            } else {
                app.detail_scroll = app.detail_scroll.saturating_add(delta as u16);
            }
            EventOutcome::Continue
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DetailJump {
    Events,
    Logs,
    Metrics,
    Diagnose,
    Bundle,
}

fn select_detail_tab(app: &mut AppState, jump: DetailJump) {
    app.reset_scroll();
    match (app.view, jump) {
        (View::Pipelines, DetailJump::Events) => app.pipeline_tab = PipelineTab::Events,
        (View::Pipelines, DetailJump::Logs) => app.pipeline_tab = PipelineTab::Logs,
        (View::Pipelines, DetailJump::Metrics) => app.pipeline_tab = PipelineTab::Metrics,
        (View::Pipelines, DetailJump::Diagnose) => app.pipeline_tab = PipelineTab::Diagnose,
        (View::Pipelines, DetailJump::Bundle) => app.pipeline_tab = PipelineTab::Bundle,
        (View::Groups, DetailJump::Events) => app.group_tab = GroupTab::Events,
        (View::Groups, DetailJump::Logs) => app.group_tab = GroupTab::Logs,
        (View::Groups, DetailJump::Metrics) => app.group_tab = GroupTab::Metrics,
        (View::Groups, DetailJump::Diagnose) => app.group_tab = GroupTab::Diagnose,
        (View::Groups, DetailJump::Bundle) => app.group_tab = GroupTab::Bundle,
        (View::Engine, DetailJump::Logs) => app.engine_tab = EngineTab::Logs,
        (View::Engine, DetailJump::Metrics) => app.engine_tab = EngineTab::Metrics,
        _ => {}
    }
}

async fn refresh_view(client: &AdminClient, app: &mut AppState, args: &UiArgs) {
    let result = match app.view {
        View::Pipelines => refresh_pipelines_view(client, app, args).await,
        View::Groups => refresh_groups_view(client, app, args).await,
        View::Engine => refresh_engine_view(client, app, args).await,
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

    let describe = super::fetch_pipeline_describe(client, &pipeline_group_id, &pipeline_id).await?;
    let header = pipeline_header(&describe);

    app.pipelines.summary = build_pipeline_summary_pane(&describe, header.clone());
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
        PipelineTab::Summary | PipelineTab::Events => {}
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
            let metrics = filter_metrics_compact(
                &client
                    .telemetry()
                    .metrics_compact(&telemetry::MetricsOptions::default())
                    .await?,
                &MetricsFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..MetricsFilters::default()
                },
            );
            app.pipelines.metrics = build_metrics_pane(
                metrics,
                add_header_chip(header.clone(), chip("scope", "pipeline", Tone::Muted)),
            );
        }
        PipelineTab::Rollout => {
            let rollout_status = maybe_fetch_rollout(client, &describe).await?;
            app.pipelines.rollout = build_rollout_pane(&describe, rollout_status.as_ref());
        }
        PipelineTab::Diagnose => {
            let logs = filter_logs(
                &super::fetch_logs(client, None, Some(args.logs_tail)).await?,
                &LogFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..LogFilters::default()
                },
            );
            let metrics = filter_metrics_compact(
                &client
                    .telemetry()
                    .metrics_compact(&telemetry::MetricsOptions::default())
                    .await?,
                &MetricsFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..MetricsFilters::default()
                },
            );
            let diagnosis =
                if let Some(rollout_status) = maybe_fetch_rollout(client, &describe).await? {
                    diagnose_pipeline_rollout(&describe, Some(&rollout_status), &logs, &metrics)
                } else {
                    diagnose_pipeline_shutdown(&describe, None, &logs, &metrics)
                };
            app.pipelines.diagnosis = build_diagnosis_pane(
                diagnosis,
                add_header_chip(header.clone(), chip("scope", "pipeline", Tone::Muted)),
            );
        }
        PipelineTab::Bundle => {
            let logs = filter_logs(
                &super::fetch_logs(client, None, Some(args.logs_tail)).await?,
                &LogFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..LogFilters::default()
                },
            );
            let metrics = filter_metrics_compact(
                &client
                    .telemetry()
                    .metrics_compact(&telemetry::MetricsOptions::default())
                    .await?,
                &MetricsFilters {
                    pipeline_group_id: Some(pipeline_group_id.clone()),
                    pipeline_id: Some(pipeline_id.clone()),
                    ..MetricsFilters::default()
                },
            );
            let rollout_status = maybe_fetch_rollout(client, &describe).await?;
            let diagnosis = if let Some(status) = rollout_status.as_ref() {
                diagnose_pipeline_rollout(&describe, Some(status), &logs, &metrics)
            } else {
                diagnose_pipeline_shutdown(&describe, None, &logs, &metrics)
            };
            let bundle = PipelineBundle {
                metadata: super::bundle_metadata(args.logs_tail, MetricsShape::Compact),
                describe,
                diagnosis,
                rollout_status,
                shutdown_status: None,
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

    app.groups.summary = build_group_summary_pane(&group_id, &describe, header.clone());
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
        GroupTab::Summary | GroupTab::Events => {}
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
            let metrics = filter_metrics_compact(
                &client
                    .telemetry()
                    .metrics_compact(&telemetry::MetricsOptions::default())
                    .await?,
                &MetricsFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..MetricsFilters::default()
                },
            );
            app.groups.metrics = build_metrics_pane(
                metrics,
                add_header_chip(header.clone(), chip("scope", "group", Tone::Muted)),
            );
        }
        GroupTab::Diagnose => {
            let logs = filter_logs(
                &super::fetch_logs(client, None, Some(args.logs_tail)).await?,
                &LogFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..LogFilters::default()
                },
            );
            let metrics = filter_metrics_compact(
                &client
                    .telemetry()
                    .metrics_compact(&telemetry::MetricsOptions::default())
                    .await?,
                &MetricsFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..MetricsFilters::default()
                },
            );
            let diagnosis = diagnose_group_shutdown(&subset, &logs, &metrics);
            app.groups.diagnosis = build_diagnosis_pane(
                diagnosis,
                add_header_chip(header.clone(), chip("scope", "group", Tone::Muted)),
            );
        }
        GroupTab::Bundle => {
            let logs = filter_logs(
                &super::fetch_logs(client, None, Some(args.logs_tail)).await?,
                &LogFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..LogFilters::default()
                },
            );
            let metrics = filter_metrics_compact(
                &client
                    .telemetry()
                    .metrics_compact(&telemetry::MetricsOptions::default())
                    .await?,
                &MetricsFilters {
                    pipeline_group_id: Some(group_id.clone()),
                    ..MetricsFilters::default()
                },
            );
            let diagnosis = diagnose_group_shutdown(&subset, &logs, &metrics);
            let bundle = GroupsBundle {
                metadata: super::bundle_metadata(args.logs_tail, MetricsShape::Compact),
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

    match app.engine_tab {
        EngineTab::Summary => {}
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
            let metrics = client
                .telemetry()
                .metrics_compact(&telemetry::MetricsOptions::default())
                .await?;
            app.engine.metrics = build_metrics_pane(
                metrics,
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
        let response = super::fetch_logs(client, None, Some(logs_tail)).await?;
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

    let response = super::fetch_logs(client, feed.next_seq, Some(logs_tail)).await?;
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

async fn maybe_fetch_rollout(
    client: &AdminClient,
    describe: &PipelineDescribeReport,
) -> Result<Option<pipelines::RolloutStatus>, CliError> {
    let Some(summary) = &describe.status.rollout else {
        return Ok(None);
    };
    match super::fetch_rollout(
        client,
        &describe.details.pipeline_group_id,
        &describe.details.pipeline_id,
        &summary.rollout_id,
    )
    .await
    {
        Ok(status) => Ok(Some(status)),
        Err(CliError::Message { exit_code: 3, .. }) => Ok(None),
        Err(err) => Err(err),
    }
}

fn selected_group_status(status: &groups::Status, group_id: &str) -> groups::Status {
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

fn build_pipeline_summary_pane(
    describe: &PipelineDescribeReport,
    header: DetailHeader,
) -> PipelineSummaryPane {
    PipelineSummaryPane {
        header: Some(header),
        stats: vec![
            card(
                "Running",
                format!(
                    "{}/{}",
                    describe.status.running_cores, describe.status.total_cores
                ),
                if describe.status.running_cores == describe.status.total_cores {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            card(
                "Generation",
                describe
                    .details
                    .active_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                Tone::Accent,
            ),
            card(
                "Conditions",
                describe.status.conditions.len().to_string(),
                Tone::Muted,
            ),
            card(
                "Events",
                describe.recent_events.len().to_string(),
                if describe.recent_events.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Accent
                },
            ),
        ],
        conditions: condition_rows(&describe.status.conditions),
        cores: core_rows(&describe.status),
        events: event_rows(&describe.recent_events),
    }
}

fn build_group_summary_pane(
    group_id: &str,
    describe: &GroupsDescribeReport,
    header: DetailHeader,
) -> GroupSummaryPane {
    let pipelines = pipeline_inventory_rows(&describe.status.pipelines, false);
    let problem_pipelines = pipelines
        .iter()
        .filter(|row| matches!(row.tone, Tone::Warning | Tone::Failure | Tone::Accent))
        .cloned()
        .collect::<Vec<_>>();

    GroupSummaryPane {
        header: Some(header),
        stats: vec![
            card(
                "Pipelines",
                describe.summary.total_pipelines.to_string(),
                Tone::Accent,
            ),
            card(
                "Running",
                describe.summary.running_pipelines.to_string(),
                Tone::Success,
            ),
            card(
                "Ready",
                describe.summary.ready_pipelines.to_string(),
                if describe.summary.ready_pipelines == describe.summary.total_pipelines {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            card(
                "Terminal",
                describe.summary.terminal_pipelines.to_string(),
                if describe.summary.terminal_pipelines == describe.summary.total_pipelines {
                    Tone::Muted
                } else {
                    Tone::Warning
                },
            ),
        ],
        problem_pipelines,
        pipelines,
        events: event_rows(
            &describe
                .recent_events
                .iter()
                .filter(|event| event.pipeline_group_id == group_id)
                .cloned()
                .collect::<Vec<_>>(),
        ),
    }
}

fn build_engine_summary_pane(
    status: &engine::Status,
    livez: &engine::ProbeResponse,
    readyz: &engine::ProbeResponse,
    header: DetailHeader,
) -> EngineSummaryPane {
    let ready_pipelines = status
        .pipelines
        .values()
        .filter(|pipeline| pipeline_is_ready(pipeline))
        .count();
    let failing = readyz
        .failing
        .iter()
        .map(|failure| ProbeFailureRow {
            pipeline: failure.pipeline.clone(),
            condition: format!(
                "{:?}={:?}",
                failure.condition.kind, failure.condition.status
            )
            .to_ascii_lowercase(),
            message: failure.condition.message.clone().unwrap_or_default(),
            tone: Tone::Failure,
        })
        .collect::<Vec<_>>();

    EngineSummaryPane {
        header: Some(header),
        stats: vec![
            card(
                "Pipelines",
                status.pipelines.len().to_string(),
                Tone::Accent,
            ),
            card(
                "Ready",
                ready_pipelines.to_string(),
                probe_tone_engine(readyz.status),
            ),
            card(
                "Livez",
                format!("{:?}", livez.status).to_ascii_lowercase(),
                probe_tone_engine(livez.status),
            ),
            card(
                "Failing",
                failing.len().to_string(),
                if failing.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Failure
                },
            ),
        ],
        pipelines: pipeline_inventory_rows(&status.pipelines, true),
        failing,
    }
}

fn build_metrics_pane(
    metrics: telemetry::CompactMetricsResponse,
    header: DetailHeader,
) -> MetricsPane {
    let rows = metric_rows(&metrics);
    MetricsPane {
        header: Some(add_header_chip(
            header,
            chip(
                "sets",
                metrics.metric_sets.len().to_string(),
                if metrics.metric_sets.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Accent
                },
            ),
        )),
        timestamp: Some(metrics.timestamp.clone()),
        rows,
        empty_message: "No compact metrics match the current scope.".to_string(),
    }
}

fn build_rollout_pane(
    describe: &PipelineDescribeReport,
    rollout_status: Option<&pipelines::RolloutStatus>,
) -> OperationPane {
    let Some(status) = rollout_status else {
        return OperationPane {
            header: Some(add_header_chip(
                pipeline_header(describe),
                chip("rollout", "none", Tone::Muted),
            )),
            stats: Vec::new(),
            rows: Vec::new(),
            empty_message: "No active rollout for the selected pipeline.".to_string(),
        };
    };

    OperationPane {
        header: Some(DetailHeader {
            title: format!("Rollout {}", status.rollout_id),
            subtitle: Some(format!(
                "{}/{}",
                status.pipeline_group_id, status.pipeline_id
            )),
            chips: vec![
                chip(
                    "state",
                    format!("{:?}", status.state).to_ascii_lowercase(),
                    rollout_tone(status.state),
                ),
                chip("action", status.action.clone(), Tone::Accent),
                chip("target", status.target_generation.to_string(), Tone::Accent),
                chip(
                    "previous",
                    status
                        .previous_generation
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    Tone::Muted,
                ),
            ],
        }),
        stats: vec![
            card("Started", status.started_at.clone(), Tone::Muted),
            card("Updated", status.updated_at.clone(), Tone::Muted),
            card("Cores", status.cores.len().to_string(), Tone::Accent),
            card(
                "Failure",
                status
                    .failure_reason
                    .clone()
                    .unwrap_or_else(|| "none".to_string()),
                if status.failure_reason.is_some() {
                    Tone::Failure
                } else {
                    Tone::Muted
                },
            ),
        ],
        rows: status
            .cores
            .iter()
            .map(|core| OperationRow {
                core: core.core_id.to_string(),
                state: core.state.clone(),
                current_generation: core.target_generation.to_string(),
                previous_generation: core
                    .previous_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                updated_at: core.updated_at.clone(),
                detail: core.detail.clone().unwrap_or_default(),
                tone: state_tone(&core.state),
            })
            .collect(),
        empty_message: "No rollout core state is available.".to_string(),
    }
}

fn build_diagnosis_pane(diagnosis: DiagnosisReport, header: DetailHeader) -> DiagnosisPane {
    let mut evidence = Vec::<EvidenceRow>::new();
    for finding in &diagnosis.findings {
        evidence.extend(finding.evidence.iter().map(evidence_row));
    }

    DiagnosisPane {
        header: Some(add_header_chip(
            header,
            chip(
                "status",
                format!("{:?}", diagnosis.status).to_ascii_lowercase(),
                diagnosis_tone(diagnosis.status),
            ),
        )),
        summary: diagnosis.summary.clone(),
        findings: diagnosis.findings.iter().map(finding_row).collect(),
        evidence,
        next_steps: diagnosis.recommended_next_steps,
    }
}

fn build_bundle_pane(
    title: &str,
    subtitle: Option<String>,
    metadata: &crate::troubleshoot::BundleMetadata,
    preview: String,
) -> BundlePane {
    BundlePane {
        header: Some(DetailHeader {
            title: title.to_string(),
            subtitle,
            chips: vec![
                chip(
                    "shape",
                    format!("{:?}", metadata.metrics_shape).to_ascii_lowercase(),
                    Tone::Accent,
                ),
                chip("logs", metadata.logs_limit.to_string(), Tone::Muted),
            ],
        }),
        stats: vec![
            card("Collected", metadata.collected_at.clone(), Tone::Muted),
            card("Preview bytes", preview.len().to_string(), Tone::Accent),
        ],
        preview,
    }
}

fn pipeline_header(describe: &PipelineDescribeReport) -> DetailHeader {
    DetailHeader {
        title: format!(
            "{}/{}",
            describe.details.pipeline_group_id, describe.details.pipeline_id
        ),
        subtitle: Some("Pipeline".to_string()),
        chips: vec![
            chip(
                "live",
                format!("{:?}", describe.livez.status).to_ascii_lowercase(),
                probe_tone(describe.livez.status),
            ),
            chip(
                "ready",
                format!("{:?}", describe.readyz.status).to_ascii_lowercase(),
                probe_tone(describe.readyz.status),
            ),
            chip(
                "running",
                format!(
                    "{}/{}",
                    describe.status.running_cores, describe.status.total_cores
                ),
                if describe.status.running_cores == describe.status.total_cores {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            chip(
                "generation",
                describe
                    .status
                    .active_generation
                    .or(describe.details.active_generation)
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                Tone::Accent,
            ),
            chip(
                "rollout",
                describe
                    .status
                    .rollout
                    .as_ref()
                    .map(|value| format!("{:?}", value.state).to_ascii_lowercase())
                    .unwrap_or_else(|| "none".to_string()),
                describe
                    .status
                    .rollout
                    .as_ref()
                    .map_or(Tone::Muted, |value| rollout_tone(value.state)),
            ),
        ],
    }
}

fn group_header(group_id: &str, describe: &GroupsDescribeReport) -> DetailHeader {
    DetailHeader {
        title: group_id.to_string(),
        subtitle: Some("Group".to_string()),
        chips: vec![
            chip(
                "pipelines",
                describe.summary.total_pipelines.to_string(),
                Tone::Accent,
            ),
            chip(
                "running",
                describe.summary.running_pipelines.to_string(),
                Tone::Success,
            ),
            chip(
                "ready",
                describe.summary.ready_pipelines.to_string(),
                if describe.summary.ready_pipelines == describe.summary.total_pipelines {
                    Tone::Success
                } else {
                    Tone::Warning
                },
            ),
            chip(
                "terminal",
                describe.summary.terminal_pipelines.to_string(),
                if describe.summary.terminal_pipelines == describe.summary.total_pipelines {
                    Tone::Muted
                } else {
                    Tone::Warning
                },
            ),
        ],
    }
}

fn engine_header(
    status: &engine::Status,
    livez: &engine::ProbeResponse,
    readyz: &engine::ProbeResponse,
) -> DetailHeader {
    DetailHeader {
        title: "Engine".to_string(),
        subtitle: Some(status.generated_at.clone()),
        chips: vec![
            chip(
                "livez",
                format!("{:?}", livez.status).to_ascii_lowercase(),
                probe_tone_engine(livez.status),
            ),
            chip(
                "readyz",
                format!("{:?}", readyz.status).to_ascii_lowercase(),
                probe_tone_engine(readyz.status),
            ),
            chip(
                "pipelines",
                status.pipelines.len().to_string(),
                Tone::Accent,
            ),
            chip(
                "failing",
                readyz.failing.len().to_string(),
                if readyz.failing.is_empty() {
                    Tone::Muted
                } else {
                    Tone::Failure
                },
            ),
        ],
    }
}

fn add_header_chip(mut header: DetailHeader, chip: StatusChip) -> DetailHeader {
    header.chips.push(chip);
    header
}

fn chip(label: impl Into<String>, value: impl Into<String>, tone: Tone) -> StatusChip {
    StatusChip {
        label: label.into(),
        value: value.into(),
        tone,
    }
}

fn card(label: impl Into<String>, value: impl Into<String>, tone: Tone) -> StatCard {
    StatCard {
        label: label.into(),
        value: value.into(),
        tone,
    }
}

fn event_rows(events: &[NormalizedEvent]) -> Vec<TimelineRow> {
    events
        .iter()
        .map(|event| TimelineRow {
            time: event.time.clone(),
            kind: format!("{:?}", event.kind).to_ascii_lowercase(),
            scope: event_scope(event),
            message: event_message(event),
            tone: event_tone(event.kind),
        })
        .collect()
}

fn log_rows(entries: &[telemetry::LogEntry]) -> Vec<LogRow> {
    entries
        .iter()
        .map(|entry| LogRow {
            time: entry.timestamp.clone(),
            level: entry.level.clone(),
            target: entry.target.clone(),
            message: entry.rendered.clone(),
            tone: state_tone(&entry.level),
        })
        .collect()
}

fn metric_rows(metrics: &telemetry::CompactMetricsResponse) -> Vec<MetricRow> {
    let mut rows = Vec::new();
    for metric_set in &metrics.metric_sets {
        let set = metric_set_label(metric_set);
        for (metric, value) in &metric_set.metrics {
            rows.push(MetricRow {
                metric_set: set.clone(),
                metric: metric.clone(),
                instrument: String::new(),
                unit: String::new(),
                value: metric_value_string(value),
            });
        }
    }
    rows
}

fn condition_rows(conditions: &[pipelines::Condition]) -> Vec<ConditionRow> {
    conditions
        .iter()
        .map(|condition| ConditionRow {
            kind: format!("{:?}", condition.kind).to_ascii_lowercase(),
            status: format!("{:?}", condition.status).to_ascii_lowercase(),
            reason: condition
                .reason
                .as_ref()
                .map(|value| value.as_str().to_ascii_lowercase())
                .unwrap_or_default(),
            message: condition.message.clone().unwrap_or_default(),
            tone: condition_tone(condition),
        })
        .collect()
}

fn core_rows(status: &pipelines::Status) -> Vec<CoreRow> {
    if let Some(instances) = &status.instances {
        instances
            .iter()
            .map(|instance| CoreRow {
                core: instance.core_id.to_string(),
                generation: instance.deployment_generation.to_string(),
                phase: phase_label(&instance.status.phase),
                heartbeat: instance.status.last_heartbeat_time.clone(),
                delete_pending: bool_label(instance.status.delete_pending),
                tone: phase_tone(&instance.status.phase),
            })
            .collect()
    } else {
        status
            .cores
            .iter()
            .map(|(core_id, core)| CoreRow {
                core: core_id.to_string(),
                generation: status
                    .active_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "active".to_string()),
                phase: phase_label(&core.phase),
                heartbeat: core.last_heartbeat_time.clone(),
                delete_pending: bool_label(core.delete_pending),
                tone: phase_tone(&core.phase),
            })
            .collect()
    }
}

fn pipeline_inventory_rows(
    pipelines: &BTreeMap<String, pipelines::Status>,
    include_group_prefix: bool,
) -> Vec<PipelineInventoryRow> {
    pipelines
        .iter()
        .map(|(name, status)| {
            let pipeline = if include_group_prefix {
                name.clone()
            } else {
                name.split_once(':')
                    .map_or_else(|| name.clone(), |(_, pipeline)| pipeline.to_string())
            };
            let (_, tone) = classify_pipeline(status);
            PipelineInventoryRow {
                pipeline,
                running: format!("{}/{}", status.running_cores, status.total_cores),
                ready: bool_label(pipeline_is_ready(status)),
                active_generation: status
                    .active_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                rollout: status
                    .rollout
                    .as_ref()
                    .map(|value| format!("{:?}", value.state).to_ascii_lowercase())
                    .unwrap_or_else(|| "none".to_string()),
                tone,
            }
        })
        .collect()
}

fn finding_row(finding: &DiagnosisFinding) -> FindingRow {
    FindingRow {
        severity: format!("{:?}", finding.severity).to_ascii_lowercase(),
        code: finding.code.clone(),
        summary: finding.summary.clone(),
        tone: match finding.severity {
            FindingSeverity::Info => Tone::Accent,
            FindingSeverity::Warning => Tone::Warning,
            FindingSeverity::Error => Tone::Failure,
        },
    }
}

fn evidence_row(evidence: &EvidenceExcerpt) -> EvidenceRow {
    EvidenceRow {
        source: evidence.source.clone(),
        time: evidence.time.clone().unwrap_or_default(),
        message: evidence.message.clone(),
    }
}

fn event_scope(event: &NormalizedEvent) -> String {
    let mut parts = vec![format!(
        "{}/{} c{}",
        event.pipeline_group_id, event.pipeline_id, event.core_id
    )];
    if let Some(node_id) = &event.node_id {
        parts.push(format!("node={node_id}"));
    }
    parts.join(" ")
}

fn event_message(event: &NormalizedEvent) -> String {
    let mut parts = vec![event.name.clone()];
    if let Some(message) = &event.message {
        parts.push(message.clone());
    }
    if let Some(detail) = &event.detail {
        parts.push(detail.clone());
    }
    parts.join(" - ")
}

fn metric_set_label(metric_set: &telemetry::MetricSet) -> String {
    let attrs = metric_set
        .attributes
        .iter()
        .take(3)
        .map(|(key, value)| format!("{key}={}", attribute_value_string(value)))
        .collect::<Vec<_>>()
        .join(" ");
    if attrs.is_empty() {
        metric_set.name.clone()
    } else {
        format!("{} [{}]", metric_set.name, attrs)
    }
}

fn attribute_value_string(value: &telemetry::AttributeValue) -> String {
    match value {
        telemetry::AttributeValue::String(value) => value.clone(),
        telemetry::AttributeValue::Int(value) => value.to_string(),
        telemetry::AttributeValue::UInt(value) => value.to_string(),
        telemetry::AttributeValue::Double(value) => value.to_string(),
        telemetry::AttributeValue::Boolean(value) => value.to_string(),
        telemetry::AttributeValue::Map(value) => serde_json::to_string(value).unwrap_or_default(),
    }
}

fn metric_value_string(value: &telemetry::MetricValue) -> String {
    match value {
        telemetry::MetricValue::U64(value) => value.to_string(),
        telemetry::MetricValue::F64(value) => format!("{value:.3}"),
        telemetry::MetricValue::Mmsc(value) => format!(
            "min={:.3} max={:.3} sum={:.3} count={}",
            value.min, value.max, value.sum, value.count
        ),
    }
}

fn classify_pipeline(status: &pipelines::Status) -> (&'static str, Tone) {
    if status.rollout.is_some() {
        return ("roll", Tone::Accent);
    }
    let has_failure = status.cores.values().any(|core| {
        matches!(
            core.phase,
            pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_)
        )
    }) || status.instances.as_ref().is_some_and(|instances| {
        instances.iter().any(|instance| {
            matches!(
                instance.status.phase,
                pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_)
            )
        })
    });
    if has_failure {
        return ("fail", Tone::Failure);
    }
    if pipeline_is_terminal(status) {
        return ("stop", Tone::Muted);
    }
    if !pipeline_is_ready(status) || status.running_cores < status.total_cores {
        return ("warn", Tone::Warning);
    }
    ("ok", Tone::Success)
}

fn bool_label(value: bool) -> String {
    if value {
        "yes".to_string()
    } else {
        "no".to_string()
    }
}

fn phase_label(phase: &pipelines::Phase) -> String {
    format!("{phase:?}").to_ascii_lowercase()
}

fn condition_tone(condition: &pipelines::Condition) -> Tone {
    match condition.status {
        pipelines::ConditionStatus::True => Tone::Success,
        pipelines::ConditionStatus::False => Tone::Failure,
        pipelines::ConditionStatus::Unknown => Tone::Warning,
    }
}

fn phase_tone(phase: &pipelines::Phase) -> Tone {
    match phase {
        pipelines::Phase::Running | pipelines::Phase::Stopped | pipelines::Phase::Deleted => {
            Tone::Success
        }
        pipelines::Phase::Pending
        | pipelines::Phase::Starting
        | pipelines::Phase::Draining
        | pipelines::Phase::Updating
        | pipelines::Phase::RollingBack
        | pipelines::Phase::Deleting(_) => Tone::Warning,
        pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_) => Tone::Failure,
    }
}

fn event_tone(kind: NormalizedEventKind) -> Tone {
    match kind {
        NormalizedEventKind::Request => Tone::Accent,
        NormalizedEventKind::Success => Tone::Success,
        NormalizedEventKind::Error => Tone::Failure,
        NormalizedEventKind::Log => Tone::Muted,
    }
}

fn rollout_tone(state: pipelines::PipelineRolloutState) -> Tone {
    match state {
        pipelines::PipelineRolloutState::Pending | pipelines::PipelineRolloutState::Running => {
            Tone::Warning
        }
        pipelines::PipelineRolloutState::Succeeded => Tone::Success,
        pipelines::PipelineRolloutState::Failed
        | pipelines::PipelineRolloutState::RollbackFailed => Tone::Failure,
        pipelines::PipelineRolloutState::RollingBack => Tone::Warning,
    }
}

fn diagnosis_tone(status: DiagnosisStatus) -> Tone {
    match status {
        DiagnosisStatus::Healthy => Tone::Success,
        DiagnosisStatus::InProgress | DiagnosisStatus::Blocked => Tone::Warning,
        DiagnosisStatus::Failed => Tone::Failure,
        DiagnosisStatus::Unknown => Tone::Muted,
    }
}

fn probe_tone(status: pipelines::ProbeStatus) -> Tone {
    match status {
        pipelines::ProbeStatus::Ok => Tone::Success,
        pipelines::ProbeStatus::Failed => Tone::Failure,
    }
}

fn probe_tone_engine(status: engine::ProbeStatus) -> Tone {
    match status {
        engine::ProbeStatus::Ok => Tone::Success,
        engine::ProbeStatus::Failed => Tone::Failure,
    }
}

fn state_tone(value: &str) -> Tone {
    let lowered = value.to_ascii_lowercase();
    if lowered.contains("fail") || lowered.contains("error") || lowered.contains("reject") {
        Tone::Failure
    } else if lowered.contains("warn")
        || lowered.contains("pending")
        || lowered.contains("drain")
        || lowered.contains("running")
        || lowered.contains("wait")
        || lowered.contains("rollback")
    {
        Tone::Warning
    } else if lowered.contains("ok")
        || lowered.contains("ready")
        || lowered.contains("success")
        || lowered.contains("succeeded")
        || lowered.contains("completed")
    {
        Tone::Success
    } else {
        Tone::Muted
    }
}

fn pipeline_is_ready(status: &pipelines::Status) -> bool {
    status.conditions.iter().any(|condition| {
        condition.kind == pipelines::ConditionKind::Ready
            && condition.status == pipelines::ConditionStatus::True
    })
}

fn pipeline_is_terminal(status: &pipelines::Status) -> bool {
    let phases_terminal = if let Some(instances) = &status.instances {
        !instances.is_empty()
            && instances.iter().all(|instance| {
                matches!(
                    instance.status.phase,
                    pipelines::Phase::Stopped
                        | pipelines::Phase::Deleted
                        | pipelines::Phase::Failed(_)
                        | pipelines::Phase::Rejected(_)
                )
            })
    } else {
        !status.cores.is_empty()
            && status.cores.values().all(|core| {
                matches!(
                    core.phase,
                    pipelines::Phase::Stopped
                        | pipelines::Phase::Deleted
                        | pipelines::Phase::Failed(_)
                        | pipelines::Phase::Rejected(_)
                )
            })
    };
    phases_terminal && status.running_cores == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::UiStartView;
    use otap_df_admin_api::{AdminEndpoint, HttpAdminClientSettings};
    use std::time::Duration;

    #[test]
    fn selected_group_status_filters_other_groups() {
        let pipeline = serde_json::json!({
            "conditions": [],
            "totalCores": 1,
            "runningCores": 1,
            "cores": {
                "0": {
                    "phase": "running",
                    "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                    "conditions": [],
                    "deletePending": false
                }
            }
        });
        let status = groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: [
                (
                    "tenant-a:ingest".to_string(),
                    serde_json::from_value(pipeline.clone())
                        .expect("pipeline fixture should deserialize"),
                ),
                (
                    "tenant-b:export".to_string(),
                    serde_json::from_value(pipeline).expect("pipeline fixture should deserialize"),
                ),
            ]
            .into_iter()
            .collect(),
        };

        let filtered = selected_group_status(&status, "tenant-a");
        assert_eq!(filtered.pipelines.len(), 1);
        assert!(filtered.pipelines.contains_key("tenant-a:ingest"));
    }

    #[test]
    fn engine_enter_drills_to_pipeline_view() {
        let mut app = AppState::new(UiStartView::Engine, true, 200);
        let pipeline = serde_json::json!({
            "conditions": [],
            "totalCores": 1,
            "runningCores": 1,
            "cores": {
                "0": {
                    "phase": "running",
                    "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                    "conditions": [],
                    "deletePending": false
                }
            }
        });
        app.engine_status = Some(engine::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: [(
                "tenant-a:ingest".to_string(),
                serde_json::from_value(pipeline).expect("pipeline fixture should deserialize"),
            )]
            .into_iter()
            .collect(),
        });
        app.ensure_selection();

        let outcome = handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &mut app);

        assert_eq!(outcome, EventOutcome::Refresh);
        assert_eq!(app.view, View::Pipelines);
        assert_eq!(app.pipeline_selected.as_deref(), Some("tenant-a:ingest"));
    }

    #[test]
    fn detail_focus_uses_h_and_l_to_cycle_tabs() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.focus = FocusArea::Detail;
        app.pipeline_tab = PipelineTab::Summary;

        let outcome = handle_key_event(
            KeyEvent::new(KeyCode::Char('l'), KeyModifiers::NONE),
            &mut app,
        );
        assert_eq!(outcome, EventOutcome::Refresh);
        assert_eq!(app.pipeline_tab, PipelineTab::Events);

        let outcome = handle_key_event(
            KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
            &mut app,
        );
        assert_eq!(outcome, EventOutcome::Refresh);
        assert_eq!(app.pipeline_tab, PipelineTab::Summary);
    }

    #[test]
    fn command_overlay_toggles_and_escape_closes_it() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);

        let outcome = handle_key_event(
            KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
            &mut app,
        );
        assert_eq!(outcome, EventOutcome::Continue);
        assert!(app.show_command_overlay);

        let outcome = handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut app);
        assert_eq!(outcome, EventOutcome::Continue);
        assert!(!app.show_command_overlay);
    }

    #[test]
    fn build_command_context_emits_canonical_prefix() {
        let endpoint =
            AdminEndpoint::from_url("https://admin.example.com:8443/engine-a").expect("endpoint");
        let settings = HttpAdminClientSettings::new(endpoint)
            .with_connect_timeout(Duration::from_secs(5))
            .with_timeout(Duration::from_secs(9))
            .with_tcp_nodelay(false);
        let args = UiArgs {
            start_view: UiStartView::Pipelines,
            refresh_interval: Duration::from_secs(4),
            logs_tail: 150,
        };

        let context = build_command_context(&settings, ColorChoice::Always, &args);

        assert_eq!(
            context.target_url,
            "https://admin.example.com:8443/engine-a"
        );
        assert_eq!(
            context.prefix_args,
            vec![
                "dfctl",
                "--url",
                "https://admin.example.com:8443/engine-a",
                "--color",
                "always",
                "--connect-timeout",
                "5s",
                "--request-timeout",
                "9s",
                "--tcp-nodelay",
                "false",
            ]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
        );
    }
}
