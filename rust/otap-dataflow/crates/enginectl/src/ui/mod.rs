// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Interactive TUI entrypoint plus shared imports for the internal UI modules.

mod actions;
mod app;
mod editor;
mod input;
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
    MetricRow, MetricsPane, OperationPane, OperationRow, PipelineInventoryRow, PipelineSummaryPane,
    PipelineTab, ProbeFailureRow, StatCard, StatusChip, TimelineRow, Tone, UiAction,
    UiCommandContext, View,
};
use self::editor::stage_pipeline_editor_draft;
use self::input::handle_event;
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
use tokio::sync::mpsc;

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

    let (tx, mut rx) = mpsc::unbounded_channel();
    let mut stop = Arc::new(AtomicBool::new(false));
    let mut event_thread = Some(spawn_event_thread(stop.clone(), tx.clone()));

    let mut refresh = tokio::time::interval(args.refresh_interval);
    let _ = refresh.tick().await;

    let result = loop {
        session.draw(&mut app)?;

        tokio::select! {
            _ = tokio::signal::ctrl_c() => break Ok(()),
            _ = refresh.tick() => {
                refresh_view(client, &mut app, &args).await;
            }
            Some(event) = rx.recv() => {
                match event {
                    UiEvent::Terminal(event) => {
                        match handle_event(event, &mut app) {
                            EventOutcome::Quit => break Ok(()),
                            EventOutcome::Refresh => refresh_view(client, &mut app, &args).await,
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
                                refresh_view(client, &mut app, &args).await;
                            }
                            EventOutcome::Execute(action) => {
                                if let Err(err) =
                                    execute_ui_action(client, &mut app, &args, action).await
                                {
                                    app.last_error = Some(err.to_string());
                                }
                                refresh_view(client, &mut app, &args).await;
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

#[cfg(test)]
mod tests;
