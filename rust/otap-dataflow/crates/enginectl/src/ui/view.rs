// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::app::{
    ActionMenuState, AppState, BundlePane, ConditionRow, ConfigPane, CoreRow, DiagnosisPane,
    EngineSummaryPane, EventPane, FindingRow, FocusArea, GroupShutdownPane, GroupShutdownRow,
    GroupSummaryPane, LogFeedState, LogRow, MetricRow, MetricsPane, OperationPane,
    PipelineInventoryRow, PipelineSummaryPane, ProbeFailureRow, StatCard, TimelineRow, Tone,
    UI_TITLE, View,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Tabs, Wrap},
};

const MIN_TERMINAL_WIDTH: u16 = 100;
const MIN_TERMINAL_HEIGHT: u16 = 25;

pub(crate) fn draw_ui(frame: &mut Frame<'_>, app: &AppState) {
    let area = frame.area();
    if area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT {
        draw_resize_required(frame, area, app);
        return;
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Length(3),
            Constraint::Min(10),
            Constraint::Length(2),
        ])
        .split(area);

    draw_title_bar(frame, sections[0], app);
    draw_top_tabs(frame, sections[1], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(39), Constraint::Percentage(61)])
        .split(sections[2]);

    draw_left_list(frame, body[0], app);
    draw_detail(frame, body[1], app);
    draw_status_bar(frame, sections[3], app);

    if app.show_help() {
        draw_help_overlay(frame, area, app);
    }
    if app.is_filter_mode() {
        draw_filter_overlay(frame, area, app);
    }
    if app.show_command_overlay() {
        draw_command_overlay(frame, area, app);
    }
    if let Some(menu) = app.action_menu() {
        draw_action_menu_overlay(frame, area, menu, app);
    }
    if let Some(confirm) = app.shutdown_confirm() {
        draw_shutdown_confirm_overlay(frame, area, confirm, app);
    }
    if let Some(editor) = app.scale_editor() {
        draw_scale_editor_overlay(frame, area, editor, app);
    }
}

fn draw_resize_required(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let message = Paragraph::new(format!(
        "dfctl ui needs at least {MIN_TERMINAL_WIDTH}x{MIN_TERMINAL_HEIGHT}\ncurrent size: {}x{}\n\nResize the terminal and try again.",
        area.width, area.height
    ))
    .block(block_with_title(
        "Terminal Too Small",
        false,
        app.color_enabled,
    ))
    .wrap(Wrap { trim: false });
    frame.render_widget(message, centered_rect(60, 30, area));
}

fn draw_top_tabs(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let titles = View::ALL
        .iter()
        .map(|view| Line::from(view.title()))
        .collect::<Vec<_>>();
    let selected = View::ALL
        .iter()
        .position(|view| view == &app.view)
        .unwrap_or(0);

    let tabs = Tabs::new(titles)
        .block(block_with_title("Views", false, app.color_enabled))
        .select(selected)
        .highlight_style(selected_style(app.color_enabled))
        .style(body_style(app.color_enabled));
    frame.render_widget(tabs, area);
}

fn draw_title_bar(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(62), Constraint::Percentage(38)])
        .split(area);

    let title = Paragraph::new(Line::from(vec![Span::styled(
        UI_TITLE,
        title_style(app.color_enabled),
    )]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(app.color_enabled)),
    )
    .alignment(Alignment::Left);
    frame.render_widget(title, sections[0]);

    let target = Paragraph::new(Line::from(vec![
        Span::styled("target ", muted_style(app.color_enabled)),
        Span::styled(app.target_url(), header_style(app.color_enabled)),
    ]))
    .block(
        Block::default()
            .borders(Borders::ALL)
            .border_style(border_style(app.color_enabled)),
    )
    .alignment(Alignment::Right)
    .wrap(Wrap { trim: true });
    frame.render_widget(target, sections[1]);
}

fn draw_left_list(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    match app.view {
        View::Pipelines => {
            let items = app.pipeline_items();
            let rows = items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    Row::new(vec![
                        badge_cell(&item.status_badge, item.tone, app.color_enabled),
                        Cell::from(item.pipeline_group_id.clone()),
                        Cell::from(item.pipeline_id.clone()),
                        Cell::from(item.running.clone()),
                        Cell::from(item.active_generation.clone()),
                        Cell::from(item.rollout.clone()),
                    ])
                    .style(stripe_style(index, app.color_enabled))
                })
                .collect::<Vec<_>>();
            let selected = items
                .iter()
                .position(|item| Some(item.key.as_str()) == app.pipeline_selected.as_deref());
            draw_state_table(
                frame,
                area,
                app,
                rows,
                vec![
                    "st".to_string(),
                    "group".to_string(),
                    "pipeline".to_string(),
                    "running".to_string(),
                    "active".to_string(),
                    "rollout".to_string(),
                ],
                [
                    Constraint::Length(6),
                    Constraint::Percentage(22),
                    Constraint::Percentage(30),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(12),
                ],
                selected,
                app.current_list_title(),
                app.focus == FocusArea::List,
                None,
            );
        }
        View::Groups => {
            let items = app.group_items();
            let rows = items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    Row::new(vec![
                        badge_cell(&item.status_badge, item.tone, app.color_enabled),
                        Cell::from(item.group_id.clone()),
                        Cell::from(item.pipelines.to_string()),
                        Cell::from(item.running.to_string()),
                        Cell::from(item.ready.to_string()),
                        Cell::from(item.terminal.to_string()),
                    ])
                    .style(stripe_style(index, app.color_enabled))
                })
                .collect::<Vec<_>>();
            let selected = items
                .iter()
                .position(|item| Some(item.group_id.as_str()) == app.group_selected.as_deref());
            draw_state_table(
                frame,
                area,
                app,
                rows,
                vec![
                    "st".to_string(),
                    "group".to_string(),
                    "pipes".to_string(),
                    "run".to_string(),
                    "ready".to_string(),
                    "term".to_string(),
                ],
                [
                    Constraint::Length(6),
                    Constraint::Percentage(44),
                    Constraint::Length(7),
                    Constraint::Length(7),
                    Constraint::Length(7),
                    Constraint::Length(7),
                ],
                selected,
                app.current_list_title(),
                app.focus == FocusArea::List,
                None,
            );
        }
        View::Engine => {
            let items = app.engine_pipeline_items();
            let rows = items
                .iter()
                .enumerate()
                .map(|(index, item)| {
                    Row::new(vec![
                        badge_cell(&item.status_badge, item.tone, app.color_enabled),
                        Cell::from(item.key.clone()),
                        Cell::from(item.running.clone()),
                        Cell::from(item.active_generation.clone()),
                        Cell::from(item.rollout.clone()),
                    ])
                    .style(stripe_style(index, app.color_enabled))
                })
                .collect::<Vec<_>>();
            let selected = items
                .iter()
                .position(|item| Some(item.key.as_str()) == app.engine_selected.as_deref());
            draw_state_table(
                frame,
                area,
                app,
                rows,
                vec![
                    "st".to_string(),
                    "pipeline".to_string(),
                    "running".to_string(),
                    "active".to_string(),
                    "rollout".to_string(),
                ],
                [
                    Constraint::Length(6),
                    Constraint::Percentage(52),
                    Constraint::Length(10),
                    Constraint::Length(10),
                    Constraint::Length(12),
                ],
                selected,
                app.current_list_title(),
                app.focus == FocusArea::List,
                None,
            );
        }
    }
}

fn draw_detail(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Min(8),
        ])
        .split(area);

    draw_detail_header(frame, sections[0], app);
    draw_detail_tabs(frame, sections[1], app);
    draw_detail_body(frame, sections[2], app);
}

fn draw_detail_header(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let Some(header) = app.active_header() else {
        let placeholder = Paragraph::new("Loading current selection…")
            .block(block_with_title(
                &app.current_detail_title(),
                app.focus == FocusArea::Detail,
                app.color_enabled,
            ))
            .style(muted_style(app.color_enabled));
        frame.render_widget(placeholder, area);
        return;
    };

    let title = header.title.clone();
    let subtitle = header.subtitle.clone().unwrap_or_default();
    let chips = if header.chips.is_empty() {
        Line::from("")
    } else {
        Line::from(
            header
                .chips
                .iter()
                .flat_map(|chip| {
                    [
                        Span::styled(
                            format!(" {}:{} ", chip.label, chip.value),
                            chip_style(chip.tone, app.color_enabled),
                        ),
                        Span::raw(" "),
                    ]
                })
                .collect::<Vec<_>>(),
        )
    };

    let lines = vec![
        Line::from(vec![
            Span::styled(title, title_style(app.color_enabled)),
            Span::raw("  "),
            Span::styled(subtitle, muted_style(app.color_enabled)),
        ]),
        chips,
    ];

    let header_widget = Paragraph::new(lines)
        .block(block_with_title(
            &app.current_detail_title(),
            app.focus == FocusArea::Detail,
            app.color_enabled,
        ))
        .wrap(Wrap { trim: false });
    frame.render_widget(header_widget, area);
}

fn draw_detail_tabs(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let tabs = Tabs::new(
        app.current_tab_titles()
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>(),
    )
    .block(block_with_title("Tabs", false, app.color_enabled))
    .select(app.current_tab_index())
    .highlight_style(selected_style(app.color_enabled))
    .style(body_style(app.color_enabled));
    frame.render_widget(tabs, area);
}

fn draw_detail_body(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    match app.view {
        View::Pipelines => match app.pipeline_tab {
            super::app::PipelineTab::Summary => {
                draw_pipeline_summary(frame, area, &app.pipelines.summary, app)
            }
            super::app::PipelineTab::Config => {
                draw_config_pane(frame, area, &app.pipelines.config, app)
            }
            super::app::PipelineTab::Events => {
                draw_events_pane(frame, area, &app.pipelines.events, app)
            }
            super::app::PipelineTab::Logs => draw_logs_pane(frame, area, &app.pipelines.logs, app),
            super::app::PipelineTab::Metrics => {
                draw_metrics_pane(frame, area, &app.pipelines.metrics, app)
            }
            super::app::PipelineTab::Rollout => {
                draw_operation_pane(frame, area, &app.pipelines.rollout, app)
            }
            super::app::PipelineTab::Shutdown => {
                draw_operation_pane(frame, area, &app.pipelines.shutdown, app)
            }
            super::app::PipelineTab::Diagnose => {
                draw_diagnosis_pane(frame, area, &app.pipelines.diagnosis, app)
            }
            super::app::PipelineTab::Bundle => {
                draw_bundle_pane(frame, area, &app.pipelines.bundle, app)
            }
        },
        View::Groups => match app.group_tab {
            super::app::GroupTab::Summary => {
                draw_group_summary(frame, area, &app.groups.summary, app)
            }
            super::app::GroupTab::Events => draw_events_pane(frame, area, &app.groups.events, app),
            super::app::GroupTab::Logs => draw_logs_pane(frame, area, &app.groups.logs, app),
            super::app::GroupTab::Metrics => {
                draw_metrics_pane(frame, area, &app.groups.metrics, app)
            }
            super::app::GroupTab::Shutdown => {
                draw_group_shutdown_pane(frame, area, &app.groups.shutdown, app)
            }
            super::app::GroupTab::Diagnose => {
                draw_diagnosis_pane(frame, area, &app.groups.diagnosis, app)
            }
            super::app::GroupTab::Bundle => draw_bundle_pane(frame, area, &app.groups.bundle, app),
        },
        View::Engine => match app.engine_tab {
            super::app::EngineTab::Summary => {
                draw_engine_summary(frame, area, &app.engine.summary, app)
            }
            super::app::EngineTab::Logs => draw_logs_pane(frame, area, &app.engine.logs, app),
            super::app::EngineTab::Metrics => {
                draw_metrics_pane(frame, area, &app.engine.metrics, app)
            }
        },
    }
}

fn draw_config_pane(frame: &mut Frame<'_>, area: Rect, pane: &ConfigPane, app: &AppState) {
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
    .style(muted_style(app.color_enabled))
    .wrap(Wrap { trim: false });
    frame.render_widget(note, sections[1]);

    let preview = Paragraph::new(pane.preview.clone())
        .block(block_with_title(
            &pane.preview_title,
            false,
            app.color_enabled,
        ))
        .style(body_style(app.color_enabled))
        .scroll((app.detail_scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, sections[2]);
}

fn draw_pipeline_summary(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &PipelineSummaryPane,
    app: &AppState,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(9),
            Constraint::Min(8),
        ])
        .split(area);
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(42), Constraint::Percentage(58)])
        .split(sections[1]);
    draw_condition_table(frame, middle[0], &pane.conditions, app.color_enabled, None);
    draw_timeline_table(
        frame,
        middle[1],
        &pane.events,
        app.color_enabled,
        0,
        Some(4),
        "Recent Events",
    );
    draw_core_table(
        frame,
        sections[2],
        &pane.cores,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
        "Core Instances",
    );
}

fn draw_group_summary(frame: &mut Frame<'_>, area: Rect, pane: &GroupSummaryPane, app: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(9),
            Constraint::Min(8),
        ])
        .split(area);
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);

    let middle = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(48), Constraint::Percentage(52)])
        .split(sections[1]);
    draw_pipeline_inventory_table(
        frame,
        middle[0],
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
        middle[1],
        &pane.events,
        app.color_enabled,
        0,
        Some(4),
        "Recent Events",
    );
    draw_pipeline_inventory_table(
        frame,
        sections[2],
        &pane.pipelines,
        app.color_enabled,
        usize::from(app.detail_scroll),
        None,
        "Group Inventory",
        Some("No pipelines are visible for this group."),
    );
}

fn draw_engine_summary(
    frame: &mut Frame<'_>,
    area: Rect,
    pane: &EngineSummaryPane,
    app: &AppState,
) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(5),
            Constraint::Length(8),
            Constraint::Min(8),
        ])
        .split(area);
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

fn draw_events_pane(frame: &mut Frame<'_>, area: Rect, pane: &EventPane, app: &AppState) {
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

fn draw_logs_pane(frame: &mut Frame<'_>, area: Rect, pane: &LogFeedState, app: &AppState) {
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

fn draw_metrics_pane(frame: &mut Frame<'_>, area: Rect, pane: &MetricsPane, app: &AppState) {
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

fn draw_operation_pane(frame: &mut Frame<'_>, area: Rect, pane: &OperationPane, app: &AppState) {
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

fn draw_group_shutdown_pane(
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
    .style(muted_style(app.color_enabled))
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

fn draw_diagnosis_pane(frame: &mut Frame<'_>, area: Rect, pane: &DiagnosisPane, app: &AppState) {
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

fn draw_bundle_pane(frame: &mut Frame<'_>, area: Rect, pane: &BundlePane, app: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(5), Constraint::Min(8)])
        .split(area);
    draw_stat_cards(frame, sections[0], &pane.stats, app.color_enabled);

    let preview = Paragraph::new(pane.preview.clone())
        .block(block_with_title("Bundle Preview", false, app.color_enabled))
        .style(body_style(app.color_enabled))
        .scroll((app.detail_scroll, 0))
        .wrap(Wrap { trim: false });
    frame.render_widget(preview, sections[1]);
}

fn draw_stat_cards(frame: &mut Frame<'_>, area: Rect, cards: &[StatCard], color_enabled: bool) {
    if cards.is_empty() {
        let placeholder = Paragraph::new("No summary cards available.")
            .block(block_with_title("Overview", false, color_enabled))
            .style(muted_style(color_enabled));
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

    for (chunk, card) in chunks.iter().zip(cards.iter()) {
        let widget = Paragraph::new(vec![
            Line::from(Span::styled(
                card.value.clone(),
                tone_style(card.tone, color_enabled).add_modifier(Modifier::BOLD),
            )),
            Line::from(Span::styled(card.label.clone(), muted_style(color_enabled))),
        ])
        .block(block_with_title("", false, color_enabled))
        .alignment(Alignment::Center);
        frame.render_widget(widget, *chunk);
    }
}

fn draw_condition_table(
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

fn draw_core_table(
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

fn draw_timeline_table(
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

fn draw_log_table(
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

fn draw_metrics_table(
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

fn draw_operation_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[super::app::OperationRow],
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

fn draw_group_shutdown_table(
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

fn draw_findings_table(
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

fn draw_evidence_table(
    frame: &mut Frame<'_>,
    area: Rect,
    rows: &[super::app::EvidenceRow],
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

fn draw_pipeline_inventory_table(
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

fn draw_probe_failures(
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

fn draw_state_table<const N: usize>(
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
        .style(muted_style(app.color_enabled))
        .alignment(Alignment::Center);
        frame.render_widget(placeholder, area);
        return;
    }

    let header = Row::new(
        headers
            .into_iter()
            .map(|value| Cell::from(value).style(header_style(app.color_enabled))),
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(block_with_title(title, focused, app.color_enabled))
        .column_spacing(1)
        .row_highlight_style(selected_style(app.color_enabled))
        .highlight_symbol("▶ ");
    let mut state = TableState::default();
    state.select(selected);
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_table_block<const N: usize>(
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
            .style(muted_style(color_enabled))
            .alignment(Alignment::Center);
        frame.render_widget(placeholder, area);
        return;
    }

    let header = Row::new(
        headers
            .into_iter()
            .map(|value| Cell::from(value).style(header_style(color_enabled))),
    );
    let table = Table::new(rows, widths)
        .header(header)
        .block(block_with_title(title, false, color_enabled))
        .column_spacing(1);
    frame.render_widget(table, area);
}

fn draw_empty_overlay(frame: &mut Frame<'_>, area: Rect, message: &str, color_enabled: bool) {
    let overlay = centered_rect(50, 40, area);
    frame.render_widget(Clear, overlay);
    frame.render_widget(
        Paragraph::new(message.to_string())
            .block(block_with_title("Empty", false, color_enabled))
            .style(muted_style(color_enabled))
            .alignment(Alignment::Center)
            .wrap(Wrap { trim: false }),
        overlay,
    );
}

fn draw_status_bar(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let age = app
        .last_refresh
        .map(|instant| format!("{}s", instant.elapsed().as_secs()))
        .unwrap_or_else(|| "never".to_string());
    let filter = if app.filter_query.is_empty() {
        "none".to_string()
    } else {
        app.filter_query.clone()
    };

    let mut spans = vec![
        Span::styled("view", key_style(app.color_enabled)),
        Span::raw(format!("={}  ", app.view.title())),
        Span::styled("focus", key_style(app.color_enabled)),
        Span::raw(format!("={}  ", app.current_focus_label())),
        Span::styled("selection", key_style(app.color_enabled)),
        Span::raw(format!("={}  ", app.current_selection_label())),
        Span::styled("refresh", key_style(app.color_enabled)),
        Span::raw(format!("={}  ", age)),
        Span::styled("filter", key_style(app.color_enabled)),
        Span::raw(format!("={}  ", filter)),
        Span::styled("Tab", key_style(app.color_enabled)),
        Span::raw(" views  "),
        Span::styled("Enter", key_style(app.color_enabled)),
        Span::raw(" focus  "),
        Span::styled("h/l", key_style(app.color_enabled)),
        Span::raw(" tabs  "),
        Span::styled("/", key_style(app.color_enabled)),
        Span::raw(" filter  "),
        Span::styled("a", key_style(app.color_enabled)),
        Span::raw(" actions  "),
        Span::styled("c", key_style(app.color_enabled)),
        Span::raw(" command"),
    ];

    if let Some(error) = &app.last_error {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("error: {error}"),
            chip_style(Tone::Failure, app.color_enabled),
        ));
    }

    let footer = Paragraph::new(Line::from(spans))
        .block(block_with_title("", false, app.color_enabled))
        .wrap(Wrap { trim: false });
    frame.render_widget(footer, area);
}

fn draw_help_overlay(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let overlay = centered_rect(74, 74, area);
    let lines = vec![
        Line::from(Span::styled("Navigation", title_style(app.color_enabled))),
        Line::from("Tab / Shift-Tab: switch top-level views"),
        Line::from("Up/Down or j/k: move list selection or scroll detail"),
        Line::from("Left/Right or h/l in detail focus: switch detail tabs"),
        Line::from("Enter: focus the detail pane"),
        Line::from("Backspace: return focus to the list"),
        Line::from("g / G: jump to first / last row"),
        Line::from("PgUp / PgDn: page through list/detail data"),
        Line::from(""),
        Line::from(Span::styled(
            "Troubleshooting",
            title_style(app.color_enabled),
        )),
        Line::from("e: events  l: logs  m: metrics  d: diagnose  b: bundle"),
        Line::from("o: rollout tab (pipelines view)"),
        Line::from("a: open context actions for the current selection"),
        Line::from("c: show equivalent CLI for the current pane"),
        Line::from("Config tab: e edit, Enter redeploy staged draft, d discard draft"),
        Line::from("r: refresh immediately"),
        Line::from(""),
        Line::from(Span::styled("Other", title_style(app.color_enabled))),
        Line::from("/: filter current resource list"),
        Line::from("Esc: clear filter / close modal / return focus"),
        Line::from("q: quit"),
    ];
    let help = Paragraph::new(lines)
        .block(block_with_title("dfctl ui help", false, app.color_enabled))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(help, overlay);
}

fn draw_command_overlay(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let overlay = centered_rect(78, 70, area);
    let recipe = app.current_command_recipe();
    let mut lines = vec![
        Line::from(Span::styled(recipe.title, title_style(app.color_enabled))),
        Line::from(Span::styled(
            format!(
                "Current pane: {} / {}",
                app.view.title(),
                app.current_detail_title()
            ),
            muted_style(app.color_enabled),
        )),
        Line::from(""),
        Line::from(recipe.description),
    ];

    if !recipe.commands.is_empty() {
        lines.push(Line::from(""));
        for command in &recipe.commands {
            lines.push(Line::from(Span::styled(
                command.label.clone(),
                header_style(app.color_enabled),
            )));
            lines.push(Line::from(Span::styled(
                command.command.clone(),
                body_style(app.color_enabled),
            )));
            lines.push(Line::from(""));
        }
    }

    if let Some(note) = &recipe.note {
        lines.push(Line::from(Span::styled(
            "Note",
            title_style(app.color_enabled),
        )));
        lines.push(Line::from(note.clone()));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "Esc or c closes this overlay.",
        muted_style(app.color_enabled),
    )));

    let command = Paragraph::new(lines)
        .block(block_with_title("Equivalent CLI", false, app.color_enabled))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(command, overlay);
}

fn draw_filter_overlay(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let overlay = centered_rect(60, 20, area);
    let text = Paragraph::new(vec![
        Line::from(format!("Filter {}", app.current_list_title())),
        Line::from("Press Enter to apply, Esc to cancel."),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", key_style(app.color_enabled)),
            Span::raw(app.filter_input.clone()),
        ]),
    ])
    .block(block_with_title("Filter", false, app.color_enabled))
    .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(text, overlay);
}

fn draw_action_menu_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    menu: &ActionMenuState,
    app: &AppState,
) {
    let overlay = centered_rect(66, 48, area);
    let mut lines = vec![
        Line::from(Span::styled(
            menu.title.clone(),
            title_style(app.color_enabled),
        )),
        Line::from(Span::styled(
            format!("target: {}", menu.target),
            muted_style(app.color_enabled),
        )),
        Line::from(""),
    ];

    for (index, entry) in menu.entries.iter().enumerate() {
        let prefix = if index == menu.selected { "▶ " } else { "  " };
        let label_style = if entry.enabled {
            if index == menu.selected {
                selected_style(app.color_enabled)
            } else {
                header_style(app.color_enabled)
            }
        } else {
            muted_style(app.color_enabled)
        };
        lines.push(Line::from(Span::styled(
            format!("{prefix}{}", entry.label),
            label_style,
        )));
        lines.push(Line::from(Span::styled(
            format!("   {}", entry.detail),
            muted_style(app.color_enabled),
        )));
        lines.push(Line::from(""));
    }

    lines.push(Line::from(Span::styled(
        "Enter executes the selected action. Esc closes this menu.",
        muted_style(app.color_enabled),
    )));

    let widget = Paragraph::new(lines)
        .block(block_with_title("Actions", false, app.color_enabled))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(widget, overlay);
}

fn draw_shutdown_confirm_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    confirm: &super::app::ShutdownConfirmState,
    app: &AppState,
) {
    let overlay = centered_rect(60, 24, area);
    let widget = Paragraph::new(vec![
        Line::from(Span::styled(
            confirm.title.clone(),
            title_style(app.color_enabled),
        )),
        Line::from(""),
        Line::from(confirm.prompt.clone()),
        Line::from(""),
        Line::from(Span::styled(
            "Enter or y confirms. Esc or n cancels.",
            muted_style(app.color_enabled),
        )),
    ])
    .block(block_with_title("Confirm", false, app.color_enabled))
    .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(widget, overlay);
}

fn draw_scale_editor_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    editor: &super::app::ScaleEditorState,
    app: &AppState,
) {
    let overlay = centered_rect(58, 24, area);
    let widget = Paragraph::new(vec![
        Line::from(Span::styled(
            "Set pipeline core count",
            title_style(app.color_enabled),
        )),
        Line::from(format!(
            "target: {}/{}",
            editor.group_id, editor.pipeline_id
        )),
        Line::from(format!("current: {}", editor.current_cores)),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", key_style(app.color_enabled)),
            Span::raw(editor.input.clone()),
        ]),
        Line::from(""),
        Line::from(Span::styled(
            "Enter submits the resize. Esc cancels.",
            muted_style(app.color_enabled),
        )),
    ])
    .block(block_with_title("Scale Editor", false, app.color_enabled))
    .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(widget, overlay);
}

fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Percentage((100 - percent_y) / 2),
            Constraint::Percentage(percent_y),
            Constraint::Percentage((100 - percent_y) / 2),
        ])
        .split(area);
    Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage((100 - percent_x) / 2),
            Constraint::Percentage(percent_x),
            Constraint::Percentage((100 - percent_x) / 2),
        ])
        .flex(Flex::Center)
        .split(vertical[1])[1]
}

fn block_with_title(title: &str, focused: bool, color_enabled: bool) -> Block<'static> {
    let border_style = if focused {
        focus_border_style(color_enabled)
    } else {
        border_style(color_enabled)
    };
    let title = if focused {
        format!("{title} [focus]")
    } else {
        title.to_string()
    };
    Block::default()
        .borders(Borders::ALL)
        .border_style(border_style)
        .title(Span::styled(title, header_style(color_enabled)))
}

fn body_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::White)
    } else {
        Style::default()
    }
}

fn title_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn header_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn muted_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    }
}

fn border_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::DarkGray)
    } else {
        Style::default()
    }
}

fn focus_border_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn selected_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Black)
            .bg(Color::LightCyan)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    }
}

fn key_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn tone_style(tone: Tone, color_enabled: bool) -> Style {
    if !color_enabled {
        return match tone {
            Tone::Success | Tone::Accent | Tone::Warning | Tone::Failure => {
                Style::default().add_modifier(Modifier::BOLD)
            }
            Tone::Muted | Tone::Neutral => Style::default(),
        };
    }

    match tone {
        Tone::Neutral => Style::default().fg(Color::White),
        Tone::Accent => Style::default()
            .fg(Color::Cyan)
            .add_modifier(Modifier::BOLD),
        Tone::Success => Style::default()
            .fg(Color::Green)
            .add_modifier(Modifier::BOLD),
        Tone::Warning => Style::default()
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD),
        Tone::Failure => Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
        Tone::Muted => Style::default().fg(Color::DarkGray),
    }
}

fn chip_style(tone: Tone, color_enabled: bool) -> Style {
    if !color_enabled {
        return tone_style(tone, color_enabled).add_modifier(Modifier::BOLD);
    }

    let (fg, bg) = match tone {
        Tone::Neutral => (Color::White, Color::Black),
        Tone::Accent => (Color::Black, Color::Cyan),
        Tone::Success => (Color::Black, Color::Green),
        Tone::Warning => (Color::Black, Color::Yellow),
        Tone::Failure => (Color::White, Color::Red),
        Tone::Muted => (Color::White, Color::DarkGray),
    };
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

fn stripe_style(index: usize, color_enabled: bool) -> Style {
    if color_enabled && index % 2 == 1 {
        Style::default().bg(Color::Rgb(18, 21, 26))
    } else {
        Style::default()
    }
}

fn badge_cell(text: &str, tone: Tone, color_enabled: bool) -> Cell<'static> {
    Cell::from(Span::styled(
        format!(" {text} "),
        chip_style(tone, color_enabled),
    ))
}

fn tone_badge(tone: Tone) -> &'static str {
    match tone {
        Tone::Accent => "roll",
        Tone::Success => "ok",
        Tone::Warning => "warn",
        Tone::Failure => "fail",
        Tone::Muted => "stop",
        Tone::Neutral => "info",
    }
}

fn card(label: impl Into<String>, value: impl Into<String>, tone: Tone) -> StatCard {
    StatCard {
        label: label.into(),
        value: value.into(),
        tone,
    }
}

fn slice<T>(rows: &[T], offset: usize, limit: Option<usize>, area_rows: usize) -> &[T] {
    if rows.is_empty() {
        return rows;
    }
    let max_rows = limit.unwrap_or(area_rows.max(1)).min(area_rows.max(1));
    let capped_offset = offset.min(rows.len().saturating_sub(1));
    let end = (capped_offset + max_rows).min(rows.len());
    &rows[capped_offset..end]
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::UiStartView;
    use crate::ui::app::{
        ConfigPane, DetailHeader, PipelineSummaryPane, StatCard, StatusChip, Tone, UiCommandContext,
    };
    use ratatui::{Terminal, backend::TestBackend};
    use std::time::Duration;

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

    #[test]
    fn title_bar_and_command_overlay_render() {
        let backend = TestBackend::new(140, 40);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.toggle_command_overlay();
        app.pipeline_selected = Some("tenant-a:ingest".to_string());
        app.pipeline_tab = super::super::app::PipelineTab::Logs;
        app.set_command_context(UiCommandContext {
            target_url: "https://admin.example.com:8443/engine-a".to_string(),
            prefix_args: vec![
                "dfctl".to_string(),
                "--url".to_string(),
                "https://admin.example.com:8443/engine-a".to_string(),
            ],
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
        assert!(rendered.contains(UI_TITLE));
        assert!(rendered.contains("Equivalent CLI"));
        assert!(rendered.contains("telemetry logs watch"));
        assert!(rendered.contains("https://admin.example.com:8443/engine-a"));
    }

    #[test]
    fn config_pane_renders_preview_and_note() {
        let backend = TestBackend::new(140, 40);
        let mut terminal = Terminal::new(backend).expect("test terminal should initialize");
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.focus = FocusArea::Detail;
        app.pipeline_selected = Some("tenant-a:ingest".to_string());
        app.pipeline_tab = super::super::app::PipelineTab::Config;
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
}
