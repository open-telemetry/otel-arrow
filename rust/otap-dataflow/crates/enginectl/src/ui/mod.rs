// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

mod app;
mod view;

use self::app::{AppState, EngineTab, FocusArea, GroupTab, PipelineTab, View};
use self::view::draw_ui;
use crate::args::{ColorChoice, MetricsShape, UiArgs};
use crate::error::CliError;
use crate::render::{
    render_diagnosis, render_engine_probe, render_engine_status, render_events,
    render_groups_describe, render_logs, render_metrics_compact, render_pipeline_describe,
    render_rollout_status,
};
use crate::style::HumanStyle;
use crate::troubleshoot::{
    BundleMetrics, EventFilters, GroupsBundle, LogFilters, MetricsFilters, PipelineBundle,
    describe_groups, diagnose_group_shutdown, diagnose_pipeline_rollout,
    diagnose_pipeline_shutdown, extract_events_from_group_status, filter_logs,
    filter_metrics_compact,
};
use crossterm::{
    event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use otap_df_admin_api::{AdminClient, groups, telemetry};
use serde::Serialize;
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
) -> Result<(), CliError> {
    let mut session = TerminalSession::new()?;
    let mut app = AppState::new(
        args.start_view,
        HumanStyle::resolve(color, true).is_enabled(),
        args.logs_tail,
    );
    let text_style = HumanStyle::resolve(ColorChoice::Never, true);

    refresh_view(client, &mut app, &args, text_style).await;

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
                refresh_view(client, &mut app, &args, text_style).await;
            }
            Some(event) = rx.recv() => {
                match event {
                    UiEvent::Terminal(event) => {
                        match handle_event(event, &mut app) {
                            EventOutcome::Quit => break,
                            EventOutcome::Refresh => refresh_view(client, &mut app, &args, text_style).await,
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

    match key.code {
        KeyCode::Char('q') => EventOutcome::Quit,
        KeyCode::Char('?') => {
            app.show_help = !app.show_help;
            EventOutcome::Continue
        }
        KeyCode::Esc => {
            if app.show_help {
                app.show_help = false;
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
            app.start_filter_input();
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
        KeyCode::Left => {
            app.cycle_tab(-1);
            EventOutcome::Refresh
        }
        KeyCode::Right => {
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

async fn refresh_view(
    client: &AdminClient,
    app: &mut AppState,
    args: &UiArgs,
    text_style: HumanStyle,
) {
    let result = match app.view {
        View::Pipelines => refresh_pipelines_view(client, app, args, text_style).await,
        View::Groups => refresh_groups_view(client, app, args, text_style).await,
        View::Engine => refresh_engine_view(client, app, args, text_style).await,
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
    text_style: HumanStyle,
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
    app.pipelines.summary = render_pipeline_describe(&text_style, &describe);
    app.pipelines.events = render_events(&text_style, &describe.recent_events);

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
                text_style,
            )
            .await?;
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
            app.pipelines.metrics = render_metrics_compact(&text_style, &metrics);
        }
        PipelineTab::Rollout => {
            app.pipelines.rollout =
                if let Some(status) = maybe_fetch_rollout(client, &describe).await? {
                    render_rollout_status(&text_style, &status)
                } else {
                    "No active rollout for the selected pipeline.".to_string()
                };
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
            app.pipelines.diagnosis = render_diagnosis(&text_style, &diagnosis);
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
            app.pipelines.bundle = serialize_pretty_json(&bundle)?;
        }
    }

    Ok(())
}

async fn refresh_groups_view(
    client: &AdminClient,
    app: &mut AppState,
    args: &UiArgs,
    text_style: HumanStyle,
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
    app.groups.summary = render_groups_describe(&text_style, &describe);
    let events = extract_events_from_group_status(
        &subset,
        Some(&EventFilters {
            pipeline_group_id: Some(group_id.clone()),
            ..EventFilters::default()
        }),
    );
    app.groups.events = render_events(&text_style, &events);

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
                text_style,
            )
            .await?;
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
            app.groups.metrics = render_metrics_compact(&text_style, &metrics);
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
            app.groups.diagnosis = render_diagnosis(&text_style, &diagnosis);
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
            app.groups.bundle = serialize_pretty_json(&bundle)?;
        }
    }

    Ok(())
}

async fn refresh_engine_view(
    client: &AdminClient,
    app: &mut AppState,
    args: &UiArgs,
    text_style: HumanStyle,
) -> Result<(), CliError> {
    let status = client.engine().status().await?;
    let livez = client.engine().livez().await?;
    let readyz = client.engine().readyz().await?;
    app.engine_status = Some(status.clone());
    app.engine_livez = Some(livez.clone());
    app.engine_readyz = Some(readyz.clone());
    app.ensure_selection();

    app.engine.summary = format!(
        "{}\n\n{}\n\n{}",
        render_engine_probe(&text_style, &livez),
        render_engine_probe(&text_style, &readyz),
        render_engine_status(&text_style, &status)
    );

    match app.engine_tab {
        EngineTab::Summary => {}
        EngineTab::Logs => {
            refresh_log_feed(
                client,
                &mut app.engine.logs,
                "engine",
                LogFilters::default(),
                args.logs_tail,
                text_style,
            )
            .await?;
        }
        EngineTab::Metrics => {
            let metrics = client
                .telemetry()
                .metrics_compact(&telemetry::MetricsOptions::default())
                .await?;
            app.engine.metrics = render_metrics_compact(&text_style, &metrics);
        }
    }

    Ok(())
}

async fn refresh_log_feed(
    client: &AdminClient,
    feed: &mut app::LogFeedState,
    scope_key: &str,
    filters: LogFilters,
    logs_tail: usize,
    text_style: HumanStyle,
) -> Result<(), CliError> {
    if feed.scope_key.as_deref() != Some(scope_key) {
        feed.reset(scope_key.to_string());
    }

    if feed.next_seq.is_none() {
        let response = super::fetch_logs(client, None, Some(logs_tail)).await?;
        let filtered = trim_logs_response(filter_logs(&response, &filters), logs_tail);
        feed.next_seq = Some(response.next_seq);
        feed.response = Some(filtered.clone());
        feed.rendered = render_logs(&text_style, &filtered);
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
        feed.rendered = render_logs(&text_style, existing);
    } else {
        let filtered = trim_logs_response(filtered, logs_tail);
        feed.rendered = render_logs(&text_style, &filtered);
        feed.response = Some(filtered);
    }

    Ok(())
}

async fn maybe_fetch_rollout(
    client: &AdminClient,
    describe: &crate::troubleshoot::PipelineDescribeReport,
) -> Result<Option<otap_df_admin_api::pipelines::RolloutStatus>, CliError> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::UiStartView;

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
        app.engine_status = Some(otap_df_admin_api::engine::Status {
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
}
