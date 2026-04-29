// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pane rendering for summaries, logs, metrics, config previews, and operation details.
//!
//! This module arranges render-ready pane models into terminal sections and
//! chooses which shared table or text widgets to draw. It intentionally works
//! with already prepared pane data so layout decisions stay separate from
//! refresh-time aggregation and troubleshooting logic.

use super::*;

/// Draw the pipeline configuration pane, including the edit note and YAML or diff preview.
pub(super) fn draw_config_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &ConfigPane,
    app: &AppState,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Min(8),
        ])
        .split(area);
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);

    let note = Paragraph::new(
        pane.note
            .clone()
            .unwrap_or_else(|| "No config note is available.".to_string()),
    )
    .block(block_with_title("Config Note", false, app.color_enabled))
    .style(subtle_panel_text_style(app.color_enabled))
    .wrap(Wrap { trim: false });
    frame.render_widget(note, sections[1]);

    let preview = Paragraph::new(pane.preview.clone())
        .block(block_with_title(
            &pane.preview_title,
            false,
            app.color_enabled,
        ))
        .style(panel_style(app.color_enabled))
        .scroll((app.detail_scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, sections[2]);
}

/// Draw the selected pipeline summary with cards, conditions, events, and core rows.
pub(super) fn draw_pipeline_summary(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &PipelineSummaryPane,
    app: &AppState,
) {
    let sections = if area.height >= 16 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(4),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(2),
            ])
            .split(area)
    };
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);
    draw_condition_table(
        frame,
        sections[1],
        &pane.conditions,
        app.color_enabled,
        None,
    );
    draw_timeline_table(
        frame,
        sections[2],
        &pane.events,
        app.color_enabled,
        0,
        Some(4),
        "Recent Events",
    );
    draw_core_table(
        frame,
        sections[3],
        &pane.cores,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
        "Core Instances",
    );
}

/// Draw the object detail pane for engine, group, or pipeline details.
pub(super) fn draw_object_details_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &ObjectDetailsPane,
    app: &AppState,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
        .split(area);
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);
    draw_object_detail_table(
        frame,
        sections[1],
        &pane.rows,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
    );
    if pane.rows.is_empty() {
        draw_empty_overlay(frame, sections[1], &pane.empty_message, app.color_enabled);
    }
}

/// Draw the selected group summary with attention rows, recent events, and inventory.
pub(super) fn draw_group_summary(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &GroupSummaryPane,
    app: &AppState,
) {
    let sections = if area.height >= 16 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Length(4),
                Constraint::Min(4),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(2),
            ])
            .split(area)
    };
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);
    draw_pipeline_inventory_table(
        frame,
        sections[1],
        &pane.problem_pipelines,
        app.color_enabled,
        0,
        Some(4),
        if pane.problem_pipelines.is_empty() {
            "Problem Pipelines"
        } else {
            "Attention Needed"
        },
        Some("No pipelines currently need attention."),
    );
    draw_timeline_table(
        frame,
        sections[2],
        &pane.events,
        app.color_enabled,
        0,
        Some(4),
        "Recent Events",
    );
    draw_pipeline_inventory_table(
        frame,
        sections[3],
        &pane.pipelines,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
        "Group Inventory",
        Some("No pipelines are visible for this group."),
    );
}

/// Draw the engine summary with global cards, probe failures, and pipeline inventory.
pub(super) fn draw_engine_summary(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &EngineSummaryPane,
    app: &AppState,
) {
    let sections = if area.height >= 12 {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(4),
                Constraint::Min(4),
            ])
            .split(area)
    } else {
        Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),
                Constraint::Length(3),
                Constraint::Min(2),
            ])
            .split(area)
    };
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);

    draw_probe_failures(frame, sections[1], &pane.failing, app.color_enabled);
    draw_pipeline_inventory_table(
        frame,
        sections[2],
        &pane.pipelines,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
        "Fleet Pipelines",
        Some("No pipelines are visible in the engine status."),
    );
}

/// Draw a full-height event timeline pane.
pub(super) fn draw_events_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &EventPane,
    app: &AppState,
) {
    draw_timeline_table(
        frame,
        area,
        &pane.rows,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
        "Event Timeline",
    );
    if pane.rows.is_empty() {
        draw_empty_overlay(frame, area, &pane.empty_message, app.color_enabled);
    }
}

/// Draw retained logs with retention and cursor summary cards.
pub(super) fn draw_logs_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &LogFeedState,
    app: &AppState,
) {
    let stats = pane
        .response
        .as_ref()
        .map(|response| {
            vec![
                card(
                    "Visible",
                    pane.rows.len().to_string(),
                    if pane.rows.is_empty() {
                        Tone::Muted
                    } else {
                        Tone::Accent
                    },
                ),
                card("Retained", response.retained_bytes.to_string(), Tone::Muted),
                card(
                    "Dropped",
                    response.dropped_on_retention.to_string(),
                    if response.dropped_on_retention > 0 {
                        Tone::Warning
                    } else {
                        Tone::Muted
                    },
                ),
                card("Cursor", response.next_seq.to_string(), Tone::Muted),
            ]
        })
        .unwrap_or_default();

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
        .split(area);
    draw_stat_cards(frame, sections[0], &stats, app.color_enabled);
    draw_log_table(
        frame,
        sections[1],
        &pane.rows,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
    );
    if pane.rows.is_empty() {
        draw_empty_overlay(frame, sections[1], &pane.empty_message, app.color_enabled);
    }
}

/// Draw compact metrics with timestamp and row-count summary cards.
pub(super) fn draw_metrics_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &MetricsPane,
    app: &AppState,
) {
    let stats = vec![
        card(
            "Timestamp",
            pane.timestamp.clone().unwrap_or_else(|| "n/a".to_string()),
            Tone::Muted,
        ),
        card(
            "Rows",
            pane.rows.len().to_string(),
            if pane.rows.is_empty() {
                Tone::Muted
            } else {
                Tone::Accent
            },
        ),
    ];
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
        .split(area);
    draw_stat_cards(frame, sections[0], &stats, app.color_enabled);
    draw_metrics_table(
        frame,
        sections[1],
        &pane.rows,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
    );
    if pane.rows.is_empty() {
        draw_empty_overlay(frame, sections[1], &pane.empty_message, app.color_enabled);
    }
}

/// Draw rollout or shutdown operation state.
pub(super) fn draw_operation_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &OperationPane,
    app: &AppState,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
        .split(area);
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);
    draw_operation_table(
        frame,
        sections[1],
        &pane.rows,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
    );
    if pane.rows.is_empty() {
        draw_empty_overlay(frame, sections[1], &pane.empty_message, app.color_enabled);
    }
}

/// Draw group shutdown state tracked by the TUI.
pub(super) fn draw_group_shutdown_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &GroupShutdownPane,
    app: &AppState,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(4),
            Constraint::Min(8),
        ])
        .split(area);
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);

    let note = Paragraph::new(
        pane.note
            .clone()
            .unwrap_or_else(|| "No group shutdown note is available.".to_string()),
    )
    .block(block_with_title("Shutdown Note", false, app.color_enabled))
    .style(subtle_panel_text_style(app.color_enabled))
    .wrap(Wrap { trim: false });
    frame.render_widget(note, sections[1]);

    draw_group_shutdown_table(
        frame,
        sections[2],
        &pane.rows,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
    );
    if pane.rows.is_empty() {
        draw_empty_overlay(frame, sections[2], &pane.empty_message, app.color_enabled);
    }
}

/// Draw diagnosis summary, findings, and evidence tables.
pub(super) fn draw_diagnosis_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &DiagnosisPane,
    app: &AppState,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(6),
            Constraint::Length(9),
            Constraint::Min(8),
        ])
        .split(area);

    let summary_lines = std::iter::once(Line::from(Span::styled(
        pane.summary.clone(),
        body_style(app.color_enabled),
    )))
    .chain(
        pane.next_steps
            .iter()
            .take(3)
            .map(|step| Line::from(format!("next: {step}"))),
    )
    .collect::<Vec<_>>();
    let summary = Paragraph::new(summary_lines)
        .block(block_with_title(
            "Diagnosis Summary",
            false,
            app.color_enabled,
        ))
        .style(panel_style(app.color_enabled))
        .wrap(Wrap { trim: false });
    frame.render_widget(summary, sections[0]);

    draw_findings_table(
        frame,
        sections[1],
        &pane.findings,
        app.color_enabled,
        usize::from(app.detail_scroll),
        Some(5),
    );
    draw_evidence_table(
        frame,
        sections[2],
        &pane.evidence,
        app.color_enabled,
        0,
        Some(6),
    );
}

/// Draw a support-bundle preview pane.
pub(super) fn draw_bundle_pane(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &BundlePane,
    app: &AppState,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
        .split(area);
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);

    let preview = Paragraph::new(pane.preview.clone())
        .block(block_with_title("Bundle Preview", false, app.color_enabled))
        .style(panel_style(app.color_enabled))
        .scroll((app.detail_scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, sections[1]);
}
