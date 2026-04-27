// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared table and summary-card rendering helpers used across the TUI.
//!
//! Tables are the dominant operational display in `dfctl ui`, so this module
//! keeps column definitions, empty-state handling, row highlighting, striping,
//! and compact summary cards consistent. Higher-level pane renderers decide
//! which table to show; this module decides how those rows are drawn.

use super::*;

/// Draw horizontal summary cards, adapting to short terminal heights.
pub(super) fn draw_stat_cards(
    frame: &mut Frame<'_>,
    area: Rect,
    cards: &[StatCard],
    color_enabled: bool,
) {
    if cards.is_empty() {
        let placeholder = Paragraph::new("No summary cards available.")
            .block(block_with_title("Overview", false, color_enabled))
            .style(subtle_panel_text_style(color_enabled));
        frame.render_widget(placeholder, area);
        return;
    }

    let constraints = cards
        .iter()
        .map(|_| Constraint::Fill(1))
        .collect::<Vec<_>>();
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(constraints)
        .split(area);

    if area.height < 5 {
        for (chunk, card) in chunks.iter().zip(cards.iter()) {
            let widget = Paragraph::new(Line::from(vec![
                Span::styled(format!("{} ", card.label), muted_style(color_enabled)),
                Span::styled(
                    card.value.clone(),
                    tone_style(card.tone, color_enabled).add_modifier(Modifier::BOLD),
                ),
            ]))
            .block(card_block(card.tone, color_enabled))
            .style(card_panel_style(card.tone, color_enabled))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: true });
            frame.render_widget(widget, *chunk);
        }
        return;
    }

    for (chunk, card) in chunks.iter().zip(cards.iter()) {
        let widget = Paragraph::new(vec![
            Line::from(Span::styled(
                card.value.clone(),
                tone_style(card.tone, color_enabled).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(card.label.clone(), muted_style(color_enabled))),
        ])
        .block(card_block(card.tone, color_enabled))
        .style(card_panel_style(card.tone, color_enabled))
        .alignment(Alignment::Center);
        frame.render_widget(widget, *chunk);
    }
}

/// Draw pipeline readiness/liveness conditions.
pub(super) fn draw_condition_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[ConditionRow],
    color_enabled: bool,
    limit: Option<usize>,
) {
    let table_rows = rows
        .iter()
        .enumerate()
        .take(limit.unwrap_or(usize::MAX))
        .map(|(index, row)| {
            Row::new(vec![
                badge_cell(&row.status, row.tone, color_enabled),
                Cell::from(row.kind.clone()),
                Cell::from(row.reason.clone()),
                Cell::from(row.message.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "st".to_string(),
            "condition".to_string(),
            "reason".to_string(),
            "message".to_string(),
        ],
        [
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(18),
            Constraint::Fill(1),
        ],
        "Conditions",
        None,
        color_enabled,
    );
}

/// Draw per-core runtime state for a pipeline.
pub(super) fn draw_core_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[CoreRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
    title: &str,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(row.core.clone()),
                Cell::from(row.generation.clone()),
                Cell::from(Span::styled(
                    row.phase.clone(),
                    tone_style(row.tone, color_enabled),
                )),
                Cell::from(row.delete_pending.clone()),
                Cell::from(row.heartbeat.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "core".to_string(),
            "gen".to_string(),
            "phase".to_string(),
            "delete".to_string(),
            "heartbeat".to_string(),
        ],
        [
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Length(14),
            Constraint::Length(8),
            Constraint::Fill(1),
        ],
        title,
        Some("No runtime instances are visible."),
        color_enabled,
    );
}

/// Draw a normalized event timeline table.
pub(super) fn draw_timeline_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[TimelineRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
    title: &str,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(row.time.clone()),
                Cell::from(Span::styled(
                    row.kind.clone(),
                    tone_style(row.tone, color_enabled),
                )),
                Cell::from(row.scope.clone()),
                Cell::from(row.message.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "time".to_string(),
            "kind".to_string(),
            "scope".to_string(),
            "message".to_string(),
        ],
        [
            Constraint::Length(20),
            Constraint::Length(10),
            Constraint::Percentage(34),
            Constraint::Fill(1),
        ],
        title,
        Some("No event rows are available."),
        color_enabled,
    );
}

/// Draw retained log entries in a scoped log pane.
pub(super) fn draw_log_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[LogRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(row.time.clone()),
                Cell::from(Span::styled(
                    row.level.clone(),
                    tone_style(row.tone, color_enabled),
                )),
                Cell::from(Span::styled(row.target.clone(), muted_style(color_enabled))),
                Cell::from(row.message.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "time".to_string(),
            "level".to_string(),
            "target".to_string(),
            "message".to_string(),
        ],
        [
            Constraint::Length(20),
            Constraint::Length(9),
            Constraint::Percentage(24),
            Constraint::Fill(1),
        ],
        "Retained Logs",
        Some("No retained logs are available."),
        color_enabled,
    );
}

/// Draw compact metrics rows in a scoped metrics pane.
pub(super) fn draw_metrics_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[MetricRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(row.metric_set.clone()),
                Cell::from(row.metric.clone()),
                Cell::from(row.value.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec!["set".to_string(), "metric".to_string(), "value".to_string()],
        [
            Constraint::Percentage(42),
            Constraint::Percentage(34),
            Constraint::Fill(1),
        ],
        "Compact Metrics",
        Some("No compact metrics are visible."),
        color_enabled,
    );
}

/// Draw per-core rollout or shutdown operation state.
pub(super) fn draw_operation_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[OperationRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(row.core.clone()),
                Cell::from(Span::styled(
                    row.state.clone(),
                    tone_style(row.tone, color_enabled),
                )),
                Cell::from(row.current_generation.clone()),
                Cell::from(row.previous_generation.clone()),
                Cell::from(row.updated_at.clone()),
                Cell::from(row.detail.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "core".to_string(),
            "state".to_string(),
            "target".to_string(),
            "prev".to_string(),
            "updated".to_string(),
            "detail".to_string(),
        ],
        [
            Constraint::Length(6),
            Constraint::Length(12),
            Constraint::Length(8),
            Constraint::Length(8),
            Constraint::Length(20),
            Constraint::Fill(1),
        ],
        "Per-Core State",
        Some("No operation rows are available."),
        color_enabled,
    );
}

/// Draw the synthetic per-pipeline state used by group shutdown tracking.
pub(super) fn draw_group_shutdown_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[GroupShutdownRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(row.pipeline.clone()),
                Cell::from(row.running.clone()),
                Cell::from(Span::styled(
                    row.terminal.clone(),
                    tone_style(row.tone, color_enabled),
                )),
                Cell::from(row.phases.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "pipeline".to_string(),
            "running".to_string(),
            "terminal".to_string(),
            "phases".to_string(),
        ],
        [
            Constraint::Percentage(34),
            Constraint::Length(10),
            Constraint::Length(10),
            Constraint::Fill(1),
        ],
        "Group Shutdown State",
        Some("No group shutdown rows are available."),
        color_enabled,
    );
}

/// Draw diagnosis findings.
pub(super) fn draw_findings_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[FindingRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(Span::styled(
                    row.severity.clone(),
                    tone_style(row.tone, color_enabled),
                )),
                Cell::from(row.code.clone()),
                Cell::from(row.summary.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "severity".to_string(),
            "code".to_string(),
            "summary".to_string(),
        ],
        [
            Constraint::Length(10),
            Constraint::Length(28),
            Constraint::Fill(1),
        ],
        "Findings",
        Some("No diagnosis findings were produced."),
        color_enabled,
    );
}

/// Draw evidence excerpts attached to diagnosis findings.
pub(super) fn draw_evidence_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[EvidenceRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(row.time.clone()),
                Cell::from(row.source.clone()),
                Cell::from(row.message.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "time".to_string(),
            "source".to_string(),
            "message".to_string(),
        ],
        [
            Constraint::Length(20),
            Constraint::Length(20),
            Constraint::Fill(1),
        ],
        "Evidence",
        Some("No supporting evidence excerpts are available."),
        color_enabled,
    );
}

/// Draw pipeline inventory rows for group and engine overview panes.
pub(super) fn draw_pipeline_inventory_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[PipelineInventoryRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
    title: &str,
    empty_message: Option<&str>,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                badge_cell(tone_badge(row.tone), row.tone, color_enabled),
                Cell::from(row.pipeline.clone()),
                Cell::from(row.running.clone()),
                Cell::from(row.ready.clone()),
                Cell::from(row.active_generation.clone()),
                Cell::from(row.rollout.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "st".to_string(),
            "pipeline".to_string(),
            "running".to_string(),
            "ready".to_string(),
            "active".to_string(),
            "rollout".to_string(),
        ],
        [
            Constraint::Length(6),
            Constraint::Percentage(38),
            Constraint::Length(10),
            Constraint::Length(7),
            Constraint::Length(10),
            Constraint::Length(12),
        ],
        title,
        empty_message,
        color_enabled,
    );
}

/// Draw engine readiness probe failures.
pub(super) fn draw_probe_failures(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[ProbeFailureRow],
    color_enabled: bool,
) {
    let table_rows = rows
        .iter()
        .enumerate()
        .take(area.height.saturating_sub(3) as usize)
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(row.pipeline.clone()),
                Cell::from(Span::styled(
                    row.condition.clone(),
                    tone_style(row.tone, color_enabled),
                )),
                Cell::from(row.message.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "pipeline".to_string(),
            "condition".to_string(),
            "message".to_string(),
        ],
        [
            Constraint::Percentage(34),
            Constraint::Length(18),
            Constraint::Fill(1),
        ],
        "Probe Failures",
        Some("Engine probes currently report no failing pipelines."),
        color_enabled,
    );
}

/// Draw field/value/detail rows for the object detail pane.
pub(super) fn draw_object_detail_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[ObjectDetailRow],
    color_enabled: bool,
    offset: usize,
    limit: Option<usize>,
) {
    let table_rows = slice(rows, offset, limit, area.height.saturating_sub(3) as usize)
        .iter()
        .enumerate()
        .map(|(index, row)| {
            Row::new(vec![
                Cell::from(Span::styled(row.field.clone(), header_style(color_enabled))),
                Cell::from(Span::styled(
                    row.value.clone(),
                    tone_style(row.tone, color_enabled),
                )),
                Cell::from(row.detail.clone()),
            ])
            .style(stripe_style(index, color_enabled))
        })
        .collect::<Vec<_>>();
    draw_table_block(
        frame,
        area,
        table_rows,
        vec![
            "field".to_string(),
            "value".to_string(),
            "detail".to_string(),
        ],
        [
            Constraint::Length(22),
            Constraint::Length(28),
            Constraint::Fill(1),
        ],
        "Object Details",
        Some("No object details are available."),
        color_enabled,
    );
}

/// Draw a selectable table owned by the left-hand resource list.
pub(super) fn draw_state_table<const N: usize>(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &AppState,
    rows: Vec<Row<'static>>,
    headers: Vec<String>,
    widths: [Constraint; N],
    selected: Option<usize>,
    title: &str,
    focused: bool,
    empty_message: Option<&str>,
) {
    if rows.is_empty() {
        let placeholder = Paragraph::new(if app.filter_query.is_empty() {
            empty_message.unwrap_or("Waiting for first refresh.")
        } else {
            "No resources match the current filter."
        })
        .block(block_with_title(title, focused, app.color_enabled))
        .style(subtle_panel_text_style(app.color_enabled))
        .alignment(Alignment::Center);
        frame.render_widget(placeholder, area);
        return;
    }

    let header = Row::new(
        headers
            .into_iter()
            .map(|value| Cell::from(value).style(table_header_style(app.color_enabled))),
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(block_with_title(title, focused, app.color_enabled))
        .style(panel_style(app.color_enabled))
        .column_spacing(1)
        .row_highlight_style(selected_style(app.color_enabled))
        .highlight_symbol("▶ ");
    let mut state = TableState::default();
    state.select(selected);
    frame.render_stateful_widget(table, area, &mut state);
}

/// Draw a non-selectable table with a standard empty-state placeholder.
pub(super) fn draw_table_block<const N: usize>(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: Vec<Row<'static>>,
    headers: Vec<String>,
    widths: [Constraint; N],
    title: &str,
    empty_message: Option<&str>,
    color_enabled: bool,
) {
    if rows.is_empty() {
        let placeholder = Paragraph::new(empty_message.unwrap_or("No rows are available."))
            .block(block_with_title(title, false, color_enabled))
            .style(subtle_panel_text_style(color_enabled))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, area);
        return;
    }

    let header = Row::new(
        headers
            .into_iter()
            .map(|value| Cell::from(value).style(table_header_style(color_enabled))),
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(block_with_title(title, false, color_enabled))
        .style(panel_style(color_enabled))
        .column_spacing(1);
    frame.render_widget(table, area);
}

/// Draw a centered empty-state overlay.
pub(super) fn draw_empty_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    message: &str,
    color_enabled: bool,
) {
    let overlay = centered_rect(50, 40, area);
    frame.render_widget(Clear, overlay);
    frame.render_widget(
        Paragraph::new(message.to_string())
            .block(block_with_title("Empty", false, color_enabled))
            .style(subtle_panel_text_style(color_enabled))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        overlay,
    );
}
