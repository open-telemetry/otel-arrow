// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Integration-style tests for UI refresh, input, and command-context behavior.
//!
//! These tests document the guarantees expected from the TUI state machine:
//! selections remain stable across refreshes, mouse and keyboard shortcuts map
//! to the intended state transitions, redacted command recipes stay safe to
//! display, and editor/config helpers preserve operator intent.

use super::*;

use crate::args::UiStartView;
use crossterm::event::{MouseButton, MouseEvent, MouseEventKind};
use otap_df_admin_api::config::tls::{TlsClientConfig, TlsConfig};
use otap_df_admin_api::{AdminEndpoint, HttpAdminClientSettings, telemetry};
use ratatui::layout::Rect;
use std::collections::BTreeMap;
use std::time::Duration;

/// Scenario: a group-focused status view is derived from a fleet-wide group
/// snapshot.
/// Guarantees: only pipelines from the selected group remain in the
/// filtered status structure used by the TUI.
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

/// Scenario: the engine view has a selected pipeline and the operator hits
/// Enter from the list.
/// Guarantees: the TUI drills into the pipelines view, preserves the
/// selected pipeline key, and triggers a refresh.
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

/// Scenario: detail focus is active and the operator cycles tabs with the
/// vim-style `h` and `l` keys.
/// Guarantees: the pipeline detail tab changes in both directions and each
/// keypress requests a refresh.
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
    assert_eq!(app.pipeline_tab, PipelineTab::Details);

    let outcome = handle_key_event(
        KeyEvent::new(KeyCode::Char('h'), KeyModifiers::NONE),
        &mut app,
    );
    assert_eq!(outcome, EventOutcome::Refresh);
    assert_eq!(app.pipeline_tab, PipelineTab::Summary);
}

/// Scenario: the pipeline config tab is focused and the operator presses
/// `e`.
/// Guarantees: the event loop requests the external editor flow for the
/// selected pipeline target.
#[test]
fn config_tab_edit_key_opens_pipeline_editor() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.focus = FocusArea::Detail;
    app.pipeline_tab = PipelineTab::Config;
    app.pipeline_selected = Some("tenant-a:ingest".to_string());

    let outcome = handle_key_event(
        KeyEvent::new(KeyCode::Char('e'), KeyModifiers::NONE),
        &mut app,
    );

    assert_eq!(
        outcome,
        EventOutcome::OpenPipelineEditor {
            group_id: "tenant-a".to_string(),
            pipeline_id: "ingest".to_string(),
        }
    );
}

/// Scenario: a staged pipeline config draft exists while the config tab is
/// focused and the operator presses `d`.
/// Guarantees: the staged draft is cleared and the UI asks for a refresh.
#[test]
fn config_tab_discard_key_clears_staged_draft() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.focus = FocusArea::Detail;
    app.pipeline_tab = PipelineTab::Config;
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.stage_pipeline_config_draft(
        "nodes: {}\n".to_string(),
        "nodes:\n  changed: true\n".to_string(),
        None,
        String::new(),
        Some("invalid".to_string()),
    );

    let outcome = handle_key_event(
        KeyEvent::new(KeyCode::Char('d'), KeyModifiers::NONE),
        &mut app,
    );

    assert_eq!(outcome, EventOutcome::Refresh);
    assert!(app.pipelines.config_draft.is_none());
}

/// Scenario: the operator toggles the equivalent-command overlay and then
/// dismisses it with Escape.
/// Guarantees: the overlay visibility state flips on and then returns to
/// the normal browsing mode.
#[test]
fn command_overlay_toggles_and_escape_closes_it() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);

    let outcome = handle_key_event(
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::NONE),
        &mut app,
    );
    assert_eq!(outcome, EventOutcome::Continue);
    assert!(app.show_command_overlay());

    let outcome = handle_key_event(KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE), &mut app);
    assert_eq!(outcome, EventOutcome::Continue);
    assert!(!app.show_command_overlay());
}

/// Scenario: the operator clicks a different top-level view tab with the
/// mouse.
/// Guarantees: the selected view changes, focus returns to the list, and
/// the UI requests a refresh.
#[test]
fn mouse_click_on_view_tabs_switches_views() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.set_terminal_size(140, 40);
    let layout = compute_ui_layout(Rect::new(0, 0, 140, 40)).expect("layout should exist");
    let titles = View::ALL
        .iter()
        .map(|view| view.title())
        .collect::<Vec<_>>();
    let groups_tab = view::tab_regions(layout.top_tabs, &titles)
        .get(1)
        .copied()
        .expect("groups tab region should exist");

    let outcome = handle_event(
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: groups_tab.x + 1,
            row: groups_tab.y,
            modifiers: KeyModifiers::NONE,
        }),
        &mut app,
    );

    assert_eq!(outcome, EventOutcome::Refresh);
    assert_eq!(app.view, View::Groups);
    assert_eq!(app.focus, FocusArea::List);
}

/// Scenario: a detail tab is selected by mouse while focus is still on the
/// resource list.
/// Guarantees: the matching detail tab becomes active, focus moves to the
/// detail pane, and the UI refreshes.
#[test]
fn mouse_click_on_detail_tabs_switches_focus_and_tab() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.set_terminal_size(140, 40);
    app.focus = FocusArea::List;
    let layout = compute_ui_layout(Rect::new(0, 0, 140, 40)).expect("layout should exist");
    let titles = app.current_tab_titles();
    let metrics_index = titles
        .iter()
        .position(|title| *title == "Metrics")
        .expect("metrics tab should exist");
    let metrics_tab = view::tab_regions(layout.detail_tabs, &titles)
        .get(metrics_index)
        .copied()
        .expect("metrics tab region should exist");

    let outcome = handle_event(
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: metrics_tab.x + 1,
            row: metrics_tab.y,
            modifiers: KeyModifiers::NONE,
        }),
        &mut app,
    );

    assert_eq!(outcome, EventOutcome::Refresh);
    assert_eq!(app.pipeline_tab, PipelineTab::Metrics);
    assert_eq!(app.focus, FocusArea::Detail);
}

/// Scenario: the operator opens the command palette and filters for the
/// pipeline Details tab.
/// Guarantees: executing the filtered palette item switches to the Pipelines
/// view, focuses the detail pane, and selects the new object-details tab.
#[test]
fn command_palette_filters_and_switches_to_pipeline_details() {
    let mut app = AppState::new(UiStartView::Groups, true, 200);

    let outcome = handle_key_event(
        KeyEvent::new(KeyCode::Char(':'), KeyModifiers::NONE),
        &mut app,
    );
    assert_eq!(outcome, EventOutcome::Continue);
    assert!(app.show_command_palette());

    for character in "pipeline details".chars() {
        let outcome = handle_key_event(
            KeyEvent::new(KeyCode::Char(character), KeyModifiers::NONE),
            &mut app,
        );
        assert_eq!(outcome, EventOutcome::Continue);
    }

    let outcome = handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &mut app);

    assert_eq!(outcome, EventOutcome::Refresh);
    assert_eq!(app.view, View::Pipelines);
    assert_eq!(app.pipeline_tab, PipelineTab::Details);
    assert_eq!(app.focus, FocusArea::Detail);
    assert!(!app.show_command_palette());
}

/// Scenario: the command palette is filtered to a current-selection
/// destructive action.
/// Guarantees: selecting the pipeline shutdown action opens the existing
/// confirmation flow rather than executing immediately.
#[test]
fn command_palette_pipeline_shutdown_action_uses_confirmation() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.open_command_palette();

    for character in "action shutdown".chars() {
        app.push_palette_input(character);
    }

    let outcome = handle_key_event(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE), &mut app);

    assert_eq!(outcome, EventOutcome::Continue);
    assert!(app.shutdown_confirm().is_some());
}

/// Scenario: the operator clicks a different row in the pipeline list.
/// Guarantees: the clicked pipeline becomes selected, list focus is
/// preserved, and a refresh is requested.
#[test]
fn mouse_click_on_pipeline_list_row_selects_pipeline() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.set_terminal_size(140, 40);
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
    app.groups_status = Some(groups::Status {
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
    });
    app.ensure_selection();
    let layout = compute_ui_layout(Rect::new(0, 0, 140, 40)).expect("layout should exist");

    let outcome = handle_event(
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: layout.list.x + 2,
            row: layout.list.y + 3,
            modifiers: KeyModifiers::NONE,
        }),
        &mut app,
    );

    assert_eq!(outcome, EventOutcome::Refresh);
    assert_eq!(app.pipeline_selected.as_deref(), Some("tenant-b:export"));
    assert_eq!(app.focus, FocusArea::List);
}

/// Scenario: detail focus is active but the operator scrolls over the
/// resource list with the mouse wheel.
/// Guarantees: list selection still moves, list focus is restored, and the
/// event loop refreshes the view.
#[test]
fn mouse_scroll_over_list_moves_selection_even_when_detail_has_focus() {
    let mut app = AppState::new(UiStartView::Groups, true, 200);
    app.set_terminal_size(140, 40);
    app.focus = FocusArea::Detail;
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
    app.groups_status = Some(groups::Status {
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
    });
    app.ensure_selection();
    let layout = compute_ui_layout(Rect::new(0, 0, 140, 40)).expect("layout should exist");

    let outcome = handle_event(
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::ScrollDown,
            column: layout.list.x + 2,
            row: layout.list.y + 2,
            modifiers: KeyModifiers::NONE,
        }),
        &mut app,
    );

    assert_eq!(outcome, EventOutcome::Refresh);
    assert_eq!(app.group_selected.as_deref(), Some("tenant-b"));
    assert_eq!(app.focus, FocusArea::List);
}

/// Scenario: the operator clicks inside the detail body while the list has
/// focus.
/// Guarantees: focus moves to the detail pane and the UI refreshes to
/// render the focus change.
#[test]
fn mouse_click_on_detail_body_switches_focus_to_detail() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.set_terminal_size(140, 40);
    app.focus = FocusArea::List;
    let layout = compute_ui_layout(Rect::new(0, 0, 140, 40)).expect("layout should exist");

    let outcome = handle_event(
        Event::Mouse(MouseEvent {
            kind: MouseEventKind::Down(MouseButton::Left),
            column: layout.detail.x + 2,
            row: layout.detail.y + 2,
            modifiers: KeyModifiers::NONE,
        }),
        &mut app,
    );

    assert_eq!(outcome, EventOutcome::Refresh);
    assert_eq!(app.focus, FocusArea::Detail);
}

/// Scenario: the UI command context is built from a non-default HTTP admin
/// client configuration.
/// Guarantees: the generated prefix arguments encode the canonical target
/// URL and every non-default connection flag needed to reproduce the UI.
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
    assert!(!context.sensitive_args_redacted);
}

/// Scenario: the UI command context includes an mTLS client key path.
/// Guarantees: equivalent-command display redacts the private key path so it is
/// not leaked in screenshots or copied diagnostics.
#[test]
fn build_command_context_redacts_client_key_path() {
    let endpoint =
        AdminEndpoint::from_url("https://admin.example.com:8443/engine-a").expect("endpoint");
    let settings = HttpAdminClientSettings::new(endpoint).with_tls(TlsClientConfig {
        config: TlsConfig {
            key_file: Some("/secret/admin-client.key".into()),
            ..TlsConfig::default()
        },
        ..TlsClientConfig::default()
    });
    let args = UiArgs {
        start_view: UiStartView::Pipelines,
        refresh_interval: Duration::from_secs(4),
        logs_tail: 150,
    };

    let context = build_command_context(&settings, ColorChoice::Auto, &args);

    assert!(context.sensitive_args_redacted);
    assert!(
        context
            .prefix_args
            .contains(&"<client-key-file>".to_string())
    );
    assert!(
        !context
            .prefix_args
            .contains(&"/secret/admin-client.key".to_string())
    );
}

/// Scenario: the configured editor command contains quoted arguments with
/// embedded whitespace.
/// Guarantees: the parser preserves the quoted argument as a single token.
#[test]
fn split_editor_command_supports_quoted_arguments() {
    let parts = split_editor_command("code --wait \"config draft.yaml\"")
        .expect("editor command should parse");
    assert_eq!(
        parts,
        vec!["code", "--wait", "config draft.yaml"]
            .into_iter()
            .map(str::to_string)
            .collect::<Vec<_>>()
    );
}

/// Scenario: the staged and committed YAML differ by at least one line.
/// Guarantees: the generated diff marks both the removed and added lines
/// with the expected headers.
#[test]
fn build_yaml_diff_marks_line_changes() {
    let diff = build_yaml_diff("a: 1\nb: 2\n", "a: 1\nb: 3\n");
    assert!(diff.contains("--- current"));
    assert!(diff.contains("+++ edited"));
    assert!(diff.contains("- b: 2"));
    assert!(diff.contains("+ b: 3"));
}

/// Scenario: engine vitals are derived from the canonical dotted metric
/// names emitted by the engine metrics set.
/// Guarantees: CPU, RSS, pressure state, and pressure detail are all mapped
/// into the condensed header status model.
#[test]
fn extract_engine_vitals_maps_cpu_rss_and_pressure() {
    let metrics = telemetry::CompactMetricsResponse {
        timestamp: "2026-01-01T00:00:00Z".to_string(),
        metric_sets: vec![telemetry::MetricSet {
            name: "engine".to_string(),
            attributes: BTreeMap::new(),
            metrics: BTreeMap::from([
                (
                    "cpu.utilization".to_string(),
                    telemetry::MetricValue::F64(0.234),
                ),
                (
                    "memory.rss".to_string(),
                    telemetry::MetricValue::U64(512 * 1024 * 1024),
                ),
                (
                    "memory.pressure.state".to_string(),
                    telemetry::MetricValue::U64(1),
                ),
                (
                    "process.memory.usage.bytes".to_string(),
                    telemetry::MetricValue::U64(768 * 1024 * 1024),
                ),
                (
                    "process.memory.soft.limit.bytes".to_string(),
                    telemetry::MetricValue::U64(1024 * 1024 * 1024),
                ),
                (
                    "process.memory.hard.limit.bytes".to_string(),
                    telemetry::MetricValue::U64(2 * 1024 * 1024 * 1024),
                ),
            ]),
        }],
    };

    let vitals = extract_engine_vitals(&metrics);

    assert_eq!(vitals.cpu_utilization, "23.4%");
    assert_eq!(vitals.memory_rss, "512.0 MiB");
    assert_eq!(vitals.pressure_state, "soft");
    assert_eq!(vitals.pressure_tone, Tone::Warning);
    assert!(
        vitals
            .pressure_detail
            .expect("pressure detail should be present")
            .contains("usage/limits 768.0 MiB / 1.0 GiB / 2.0 GiB")
    );
    assert!(!vitals.stale);
}

/// Scenario: engine vitals are derived from the legacy underscore metric
/// names.
/// Guarantees: the TUI still understands the backward-compatible metric
/// aliases.
#[test]
fn extract_engine_vitals_accepts_legacy_underscore_names() {
    let metrics = telemetry::CompactMetricsResponse {
        timestamp: "2026-01-01T00:00:00Z".to_string(),
        metric_sets: vec![telemetry::MetricSet {
            name: "engine".to_string(),
            attributes: BTreeMap::new(),
            metrics: BTreeMap::from([
                (
                    "cpu_utilization".to_string(),
                    telemetry::MetricValue::F64(0.125),
                ),
                (
                    "memory_rss".to_string(),
                    telemetry::MetricValue::U64(256 * 1024 * 1024),
                ),
                (
                    "memory_pressure_state".to_string(),
                    telemetry::MetricValue::U64(0),
                ),
            ]),
        }],
    };

    let vitals = extract_engine_vitals(&metrics);

    assert_eq!(vitals.cpu_utilization, "12.5%");
    assert_eq!(vitals.memory_rss, "256.0 MiB");
    assert_eq!(vitals.pressure_state, "normal");
}

/// Scenario: a previous engine-vitals snapshot exists but the next metrics
/// refresh fails.
/// Guarantees: the previous values are retained and the snapshot is marked
/// stale instead of being cleared.
#[test]
fn update_engine_vitals_marks_previous_snapshot_stale_when_metrics_fail() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.engine_vitals = EngineVitals {
        cpu_utilization: "12.5%".to_string(),
        cpu_tone: Tone::Accent,
        memory_rss: "256.0 MiB".to_string(),
        memory_tone: Tone::Accent,
        pressure_state: "normal".to_string(),
        pressure_tone: Tone::Success,
        pressure_detail: Some("usage/limits 200.0 MiB / 1.0 GiB / 2.0 GiB".to_string()),
        stale: false,
    };

    update_engine_vitals(&mut app, None, Some("connection reset"));

    assert!(app.engine_vitals.stale);
    assert_eq!(app.engine_vitals.cpu_utilization, "12.5%");
    assert_eq!(app.engine_vitals.memory_rss, "256.0 MiB");
}
