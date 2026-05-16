// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Interactive TUI entrypoint plus shared imports for the internal UI modules.
//!
//! The UI module owns the `dfctl ui` event loop and coordinates state refresh,
//! input handling, rendering, modal workflows, and admin API mutations. Its
//! submodules are split by responsibility so the TUI can stay operationally
//! useful without mixing terminal lifecycle code, app state transitions,
//! command recipes, pane construction, and drawing logic in one place.

mod actions;
mod app;
mod editor;
mod input;
mod object_details;
mod panes;
mod refresh;
mod rows;
mod session;
mod view;

use self::actions::{execute_ui_action, load_pipeline_details};
use self::app::{
    AppState, BundlePane, ConditionRow, ConfigPane, CoreRow, DetailHeader, DiagnosisPane,
    EngineSummaryPane, EngineTab, EngineVitals, EventPane, EvidenceRow, FindingRow, FocusArea,
    GroupShutdownPane, GroupShutdownRow, GroupSummaryPane, GroupTab, LogFeedState, LogRow,
    MetricRow, MetricsPane, ObjectDetailRow, ObjectDetailsPane, OperationPane, OperationRow,
    PaletteAction, PipelineInventoryRow, PipelineSummaryPane, PipelineTab, ProbeFailureRow,
    StatCard, StatusChip, TimelineRow, Tone, UiAction, UiCommandContext, UiModal, View,
};
use self::editor::stage_pipeline_editor_draft;
use self::input::handle_event;
use self::object_details::*;
use self::panes::*;
use self::refresh::refresh_view;
use self::rows::*;
use self::session::{
    EventOutcome, TerminalSession, UiEvent, restart_event_thread, spawn_event_thread,
    stop_event_thread,
};
use self::view::{compute_ui_layout, draw_ui, state_table_row_hit_index, tab_hit_index};
use crate::args::{ColorChoice, MetricsShape, UiArgs};
use crate::error::CliError;
use crate::style::HumanStyle;
use crate::troubleshoot::{
    BundleMetrics, DiagnosisFinding, DiagnosisReport, DiagnosisStatus, EventFilters,
    EvidenceExcerpt, FindingSeverity, GroupsBundle, GroupsDescribeReport, LogFilters,
    MetricsFilters, NormalizedEvent, NormalizedEventKind, PipelineBundle, PipelineDescribeReport,
    describe_groups, diagnose_group_shutdown, diagnose_pipeline_rollout,
    diagnose_pipeline_shutdown, extract_events_from_group_status, filter_logs,
    filter_metrics_compact, group_shutdown_snapshot,
};
use crate::{parse_pipeline_config_content, serialize_pipeline_config_yaml};
use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEvent, KeyEventKind,
        KeyModifiers, MouseButton, MouseEvent, MouseEventKind,
    },
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use humantime::format_duration;
use otap_df_admin_api::{
    AdminClient, HttpAdminClientSettings, engine, groups, operations::OperationOptions, pipelines,
    telemetry,
};
use otap_df_config::pipeline::PipelineConfig;
use ratatui::layout::Rect;
use serde::Serialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::env;
use std::io::{self, Write};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::Duration;
use tempfile::Builder;
use tokio::{sync::mpsc, time::MissedTickBehavior};

pub(crate) use self::refresh::build_command_context;

#[cfg(test)]
use self::editor::{build_yaml_diff, split_editor_command};
#[cfg(test)]
use self::input::handle_key_event;
#[cfg(test)]
use self::refresh::{extract_engine_vitals, selected_group_status, update_engine_vitals};

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

    let (tx, mut rx) = mpsc::channel(256);
    let mut stop = Arc::new(AtomicBool::new(false));
    let mut event_thread = Some(spawn_event_thread(stop.clone(), tx.clone()));
    let refresh_in_flight = Arc::new(AtomicBool::new(false));

    let mut refresh = tokio::time::interval(args.refresh_interval);
    refresh.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let _ = refresh.tick().await;
    let mut activity_tick = tokio::time::interval(Duration::from_millis(120));
    activity_tick.set_missed_tick_behavior(MissedTickBehavior::Skip);
    let _ = activity_tick.tick().await;

    let result = loop {
        session.draw(&mut app)?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => break Ok(()),
            _ = activity_tick.tick(), if app.is_activity_active() => {
                app.advance_activity_frame();
            }
            _ = refresh.tick() => {
                request_refresh(client, &mut app, &args, &tx, &refresh_in_flight);
            }
            Some(event) = rx.recv() => {
                match event {
                    UiEvent::RefreshComplete(refreshed) => {
                        apply_refreshed_app(&mut app, *refreshed);
                        app.end_activity();
                    }
                    UiEvent::Terminal(event) => {
                        match handle_event(event, &mut app) {
                            EventOutcome::Quit => break Ok(()),
                            EventOutcome::Refresh => {
                                request_refresh(client, &mut app, &args, &tx, &refresh_in_flight);
                            }
                            EventOutcome::OpenPipelineEditor { group_id, pipeline_id } => {
                                if let Err(err) = stage_pipeline_editor_draft(
                                    client,
                                    &mut session,
                                    &tx,
                                    &mut stop,
                                    &mut event_thread,
                                    &mut app,
                                    &group_id,
                                    &pipeline_id,
                                )
                                .await
                                {
                                    app.last_error = Some(err.to_string());
                                }
                                request_refresh(client, &mut app, &args, &tx, &refresh_in_flight);
                            }
                            EventOutcome::Execute(action) => {
                                if let Err(err) =
                                    execute_ui_action(client, &mut app, &args, action).await
                                {
                                    app.last_error = Some(err.to_string());
                                }
                                request_refresh(client, &mut app, &args, &tx, &refresh_in_flight);
                            }
                            EventOutcome::Continue => {}
                        }
                    }
                    UiEvent::TerminalError(error) => {
                        app.last_error = Some(error);
                    }
                }
            }
        }
    };

    stop_event_thread(&stop, &mut event_thread);
    result
}

fn request_refresh(
    client: &AdminClient,
    app: &mut AppState,
    args: &UiArgs,
    tx: &mpsc::Sender<UiEvent>,
    refresh_in_flight: &Arc<AtomicBool>,
) {
    if spawn_refresh_task(
        client.clone(),
        app.clone(),
        args.clone(),
        tx.clone(),
        refresh_in_flight.clone(),
    ) {
        app.begin_activity();
    }
}

fn spawn_refresh_task(
    client: AdminClient,
    mut app: AppState,
    args: UiArgs,
    tx: mpsc::Sender<UiEvent>,
    refresh_in_flight: Arc<AtomicBool>,
) -> bool {
    if refresh_in_flight
        .compare_exchange(false, true, Ordering::Relaxed, Ordering::Relaxed)
        .is_err()
    {
        return false;
    }

    let _handle = tokio::spawn(async move {
        refresh_view(&client, &mut app, &args).await;
        refresh_in_flight.store(false, Ordering::Relaxed);
        let _ = tx.send(UiEvent::RefreshComplete(Box::new(app))).await;
    });
    true
}

fn apply_refreshed_app(app: &mut AppState, refreshed: AppState) {
    if app.view != refreshed.view
        || app.pipeline_selected != refreshed.pipeline_selected
        || app.group_selected != refreshed.group_selected
        || app.engine_selected != refreshed.engine_selected
    {
        return;
    }

    let view = app.view;
    let focus = app.focus;
    let pipeline_tab = app.pipeline_tab;
    let group_tab = app.group_tab;
    let engine_tab = app.engine_tab;
    let color_enabled = app.color_enabled;
    let filter_query = app.filter_query.clone();
    let filter_input = app.filter_input.clone();
    let modal = app.modal.clone();
    let detail_scroll = app.detail_scroll;
    let terminal_size = app.terminal_size;
    let command_context = app.command_context.clone();
    let activity_indicator = app.activity_indicator.clone();

    *app = refreshed;
    app.view = view;
    app.focus = focus;
    app.pipeline_tab = pipeline_tab;
    app.group_tab = group_tab;
    app.engine_tab = engine_tab;
    app.color_enabled = color_enabled;
    app.filter_query = filter_query;
    app.filter_input = filter_input;
    app.modal = modal;
    app.detail_scroll = detail_scroll;
    app.terminal_size = terminal_size;
    app.command_context = command_context;
    app.activity_indicator = activity_indicator;
}

#[cfg(test)]
mod tests;
