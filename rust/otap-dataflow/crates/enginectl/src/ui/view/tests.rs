// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Render tests for TUI layout, chrome, panes, and theme behavior.
//!
//! These tests use Ratatui's test backend to verify visible UI guarantees
//! without requiring a real terminal. They protect layout fallbacks, title and
//! command overlays, tab ordering, shimmer behavior, and representative pane
//! rendering from accidental regressions.

use super::*;

use crate::args::UiStartView;
use crate::ui::app::{
    ConfigPane, DetailHeader, EngineVitals, PipelineSummaryPane, StatCard, StatusChip, Tone,
    UiCommandContext,
};
use ratatui::{Terminal, backend::TestBackend};
use std::time::Duration;

/// Scenario: the TUI is rendered in a terminal that is below the supported
/// minimum dimensions.
/// Guarantees: the renderer switches to the explicit resize-required
/// message instead of drawing the normal layout.
#[test]
fn small_terminal_shows_resize_message() {
    let backend = TestBackend::new(60, 10);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    let app = AppState::new(UiStartView::Pipelines, true, 200);

    let _ = terminal
        .draw(|frame| draw_ui(frame, &app))
        .expect("draw should succeed");

    let buffer = terminal.backend().buffer().clone();
    let rendered = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(rendered.contains("Terminal Too Small"));
}

/// Scenario: a populated pipeline summary pane is rendered with a detail
/// header and summary cards.
/// Guarantees: the header title, status chip, and summary card labels are
/// all present in the rendered buffer.
#[test]
fn pipeline_summary_renders_structured_header() {
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.focus = FocusArea::Detail;
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.pipelines.summary = PipelineSummaryPane {
        header: Some(DetailHeader {
            title: "tenant-a/ingest".to_string(),
            subtitle: Some("Pipeline".to_string()),
            chips: vec![StatusChip {
                label: "ready".to_string(),
                value: "ok".to_string(),
                tone: Tone::Success,
            }],
        }),
        stats: vec![StatCard {
            label: "Running".to_string(),
            value: "1/1".to_string(),
            tone: Tone::Success,
        }],
        ..PipelineSummaryPane::default()
    };

    let _ = terminal
        .draw(|frame| draw_ui(frame, &app))
        .expect("draw should succeed");

    let buffer = terminal.backend().buffer().clone();
    let rendered = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(rendered.contains("tenant-a/ingest"));
    assert!(rendered.contains("ready:ok"));
    assert!(rendered.contains("Running"));
}

/// Scenario: the command overlay is opened while the logs pane is selected.
/// Guarantees: the render output includes the branded title bar, the
/// equivalent CLI heading, and the generated command content.
#[test]
fn title_bar_and_command_overlay_render() {
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.toggle_command_overlay();
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.pipeline_tab = PipelineTab::Logs;
    app.set_command_context(UiCommandContext {
        target_url: "https://admin.example.com:8443/engine-a".to_string(),
        prefix_args: vec![
            "dfctl".to_string(),
            "--url".to_string(),
            "https://admin.example.com:8443/engine-a".to_string(),
        ],
        sensitive_args_redacted: false,
        refresh_interval: Duration::from_secs(2),
        logs_tail: 200,
    });

    let _ = terminal
        .draw(|frame| draw_ui(frame, &app))
        .expect("draw should succeed");

    let buffer = terminal.backend().buffer().clone();
    let rendered = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(rendered.contains("OpenTelemetry - Rust Dataflow Engine"));
    assert!(rendered.contains("Equivalent CLI"));
    assert!(rendered.contains("telemetry logs watch"));
    assert!(rendered.contains("https://admin.example.com:8443/engine-a"));
}

/// Scenario: activity is in progress while the title bar is rendered.
/// Guarantees: the shimmer splits the brand into fixed-position character
/// spans without changing the visible OpenTelemetry label text.
#[test]
fn active_brand_shimmer_preserves_label_text() {
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    assert_eq!(chrome::brand_spans(&app).len(), 2);

    app.begin_activity();
    app.advance_activity_frame();
    app.advance_activity_frame();

    let active_spans = chrome::brand_spans(&app);
    let label = active_spans
        .iter()
        .map(|span| span.content.as_ref())
        .collect::<String>();
    assert_eq!(label, "OpenTelemetry");
    assert_eq!(active_spans.len(), "OpenTelemetry".len());
    assert!(
        active_spans
            .iter()
            .any(|span| span.style != open_brand_style(true)
                && span.style != telemetry_brand_style(true))
    );

    app.end_activity();
    assert_eq!(chrome::brand_spans(&app).len(), 2);
}

/// Scenario: the header renders a fresh engine-vitals snapshot.
/// Guarantees: the condensed vitals rail shows CPU, RSS, and pressure
/// values without reintroducing the removed dedicated panel title.
#[test]
fn header_renders_engine_vitals_rail() {
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.engine_vitals = EngineVitals {
        cpu_utilization: "23.4%".to_string(),
        cpu_tone: Tone::Accent,
        memory_rss: "512.0 MiB".to_string(),
        memory_tone: Tone::Accent,
        pressure_state: "soft".to_string(),
        pressure_tone: Tone::Warning,
        pressure_detail: Some("usage/limits 768.0 MiB / 1.0 GiB / 2.0 GiB".to_string()),
        stale: false,
    };

    let _ = terminal
        .draw(|frame| draw_ui(frame, &app))
        .expect("draw should succeed");

    let buffer = terminal.backend().buffer().clone();
    let rendered = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(rendered.contains("23.4%"));
    assert!(rendered.contains("512.0 MiB"));
    assert!(rendered.contains("soft"));
    assert!(!rendered.contains("Engine Vitals"));
}

/// Scenario: the config pane is rendered for a deployable staged draft.
/// Guarantees: the preview title, operator note, and diff content all
/// appear in the rendered buffer.
#[test]
fn config_pane_renders_preview_and_note() {
    let backend = TestBackend::new(140, 40);
    let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
    let mut app = AppState::new(UiStartView::Pipelines, true, 200);
    app.focus = FocusArea::Detail;
    app.pipeline_selected = Some("tenant-a:ingest".to_string());
    app.pipeline_tab = PipelineTab::Config;
    app.pipelines.config = ConfigPane {
        header: Some(DetailHeader {
            title: "tenant-a/ingest".to_string(),
            subtitle: Some("Pipeline".to_string()),
            chips: vec![StatusChip {
                label: "config".to_string(),
                value: "draft".to_string(),
                tone: Tone::Accent,
            }],
        }),
        stats: vec![StatCard {
            label: "Status".to_string(),
            value: "deployable".to_string(),
            tone: Tone::Success,
        }],
        note: Some("Press Enter to redeploy the staged draft.".to_string()),
        preview_title: "Canonical Diff".to_string(),
        preview: "--- current\n+++ edited\n- old\n+ new".to_string(),
    };

    let _ = terminal
        .draw(|frame| draw_ui(frame, &app))
        .expect("draw should succeed");

    let buffer = terminal.backend().buffer().clone();
    let rendered = buffer
        .content
        .iter()
        .map(|cell| cell.symbol())
        .collect::<String>();
    assert!(rendered.contains("Canonical Diff"));
    assert!(rendered.contains("Press Enter to redeploy"));
    assert!(rendered.contains("--- current"));
}

/// Scenario: the top-level view order is exposed through the shared view
/// enumeration.
/// Guarantees: the tab order remains Engine, Groups, then Pipelines.
#[test]
fn top_level_view_order_is_engine_groups_pipelines() {
    let titles = View::ALL.map(View::title);
    assert_eq!(titles, ["Engine", "Groups", "Pipelines"]);
}

/// Scenario: tab hit-testing regions are computed for a tab bar that uses
/// pipe separators between tabs.
/// Guarantees: each tab region accounts for the separator width and maps to
/// the expected horizontal coordinates.
#[test]
fn tab_regions_account_for_pipe_separators() {
    let regions = tab_regions(Rect::new(0, 0, 40, 1), &["Engine", "Groups", "Pipelines"]);
    assert_eq!(regions.len(), 3);
    assert_eq!(regions[0], Rect::new(0, 0, 8, 1));
    assert_eq!(regions[1], Rect::new(11, 0, 8, 1));
    assert_eq!(regions[2], Rect::new(22, 0, 11, 1));
}
