// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::app::{
    ActionMenuState, AppState, BundlePane, ConditionRow, ConfigPane, CoreRow, DiagnosisPane,
    EngineSummaryPane, EngineVitals, EventPane, FindingRow, FocusArea, GroupShutdownPane,
    GroupShutdownRow, GroupSummaryPane, LogFeedState, LogRow, MetricRow, MetricsPane,
    OperationPane, PipelineInventoryRow, PipelineSummaryPane, ProbeFailureRow, StatCard,
    StatusChip, TimelineRow, Tone, View,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

pub(crate) const MIN_TERMINAL_WIDTH: u16 = 100;
pub(crate) const MIN_TERMINAL_HEIGHT: u16 = 25;
const HEADER_HEIGHT: u16 = 2;
const STATUS_BAR_HEIGHT: u16 = 2;
const DETAIL_HEADER_HEIGHT: u16 = 1;
const DETAIL_TABS_HEIGHT: u16 = 1;
const ENGINE_VITALS_WIDTH: u16 = 38;
const TAB_SEPARATOR: &str = " | ";
const TAB_SEPARATOR_WIDTH: u16 = 3;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiLayout {
    pub(crate) header: Rect,
    pub(crate) title_bar: Rect,
    pub(crate) top_tabs: Rect,
    pub(crate) engine_vitals: Rect,
    pub(crate) list: Rect,
    pub(crate) detail: Rect,
    pub(crate) detail_tabs: Rect,
    pub(crate) status: Rect,
}

pub(crate) fn compute_ui_layout(area: Rect) -> Option<UiLayout> {
    if area.width < MIN_TERMINAL_WIDTH || area.height < MIN_TERMINAL_HEIGHT {
        return None;
    }

    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(10),
            Constraint::Length(STATUS_BAR_HEIGHT),
        ])
        .split(area);
    let body = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(28), Constraint::Percentage(72)])
        .split(sections[1]);
    let header = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(sections[0]);
    let header_top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(ENGINE_VITALS_WIDTH)])
        .split(header[0]);
    let detail = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(DETAIL_HEADER_HEIGHT),
            Constraint::Length(DETAIL_TABS_HEIGHT),
            Constraint::Min(6),
        ])
        .split(body[1]);

    Some(UiLayout {
        header: sections[0],
        title_bar: header_top[0],
        top_tabs: header[1],
        engine_vitals: header_top[1],
        list: body[0],
        detail: body[1],
        detail_tabs: detail[1],
        status: sections[2],
    })
}

pub(crate) fn tab_hit_index(area: Rect, labels: &[&str], column: u16, row: u16) -> Option<usize> {
    if area.height == 0 || row < area.y || row >= area.y + area.height {
        return None;
    }

    tab_regions(area, labels)
        .into_iter()
        .enumerate()
        .find_map(|(index, region)| {
            (column >= region.x && column < region.x.saturating_add(region.width)).then_some(index)
        })
}

pub(crate) fn state_table_row_hit_index(area: Rect, row: u16, item_count: usize) -> Option<usize> {
    if item_count == 0 {
        return None;
    }

    let inner = Block::default().borders(Borders::ALL).inner(area);
    if inner.height <= 1 {
        return None;
    }

    let data_start = inner.y.saturating_add(1);
    let data_end = inner.y.saturating_add(inner.height);
    if row < data_start || row >= data_end {
        return None;
    }

    let index = usize::from(row.saturating_sub(data_start));
    (index < item_count).then_some(index)
}

pub(crate) fn draw_ui(frame: &mut Frame<'_>, app: &AppState) {
    let area = frame.area();
    let Some(layout) = compute_ui_layout(area) else {
        draw_resize_required(frame, area, app);
        return;
    };

    frame.render_widget(Block::default().style(page_style(app.color_enabled)), area);
    draw_header(frame, layout.header, app);
    draw_left_list(frame, layout.list, app);
    draw_detail(frame, layout.detail, app);
    draw_status_bar(frame, layout.status, app);

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

fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(1), Constraint::Length(1)])
        .split(area);
    let top = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Min(0), Constraint::Length(ENGINE_VITALS_WIDTH)])
        .split(sections[0]);

    draw_title_bar(frame, top[0], app);
    draw_top_tabs(frame, sections[1], app);
    draw_engine_vitals_panel(frame, top[1], &app.engine_vitals, app);
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
    let selected = View::ALL
        .iter()
        .position(|view| view == &app.view)
        .unwrap_or(0);
    let titles = View::ALL
        .iter()
        .map(|view| view.title())
        .collect::<Vec<_>>();
    draw_tab_bar(frame, area, &titles, selected, app.color_enabled);
}

fn draw_title_bar(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let line = Line::from(vec![
        Span::styled("Open", open_brand_style(app.color_enabled)),
        Span::styled("Telemetry", telemetry_brand_style(app.color_enabled)),
        Span::styled(" - Rust Dataflow Engine", title_style(app.color_enabled)),
        Span::styled("  |  ", separator_style(app.color_enabled)),
        Span::styled("target ", muted_style(app.color_enabled)),
        Span::styled(app.target_url(), target_style(app.color_enabled)),
    ]);
    let title = Paragraph::new(line)
        .alignment(Alignment::Left)
        .style(page_style(app.color_enabled));
    frame.render_widget(title, area);
}

fn draw_engine_vitals_panel(
    frame: &mut Frame<'_>,
    area: Rect,
    vitals: &EngineVitals,
    app: &AppState,
) {
    let mut spans = vec![
        Span::styled("CPU ", muted_style(app.color_enabled)),
        Span::styled(
            vitals.cpu_utilization.clone(),
            tone_style(vitals.cpu_tone, app.color_enabled),
        ),
        Span::styled(TAB_SEPARATOR, separator_style(app.color_enabled)),
        Span::styled("RSS ", muted_style(app.color_enabled)),
        Span::styled(
            vitals.memory_rss.clone(),
            tone_style(vitals.memory_tone, app.color_enabled),
        ),
        Span::styled(TAB_SEPARATOR, separator_style(app.color_enabled)),
        Span::styled(
            vitals.pressure_state.clone(),
            tone_style(vitals.pressure_tone, app.color_enabled),
        ),
    ];
    if vitals.stale {
        spans.push(Span::styled(
            TAB_SEPARATOR,
            separator_style(app.color_enabled),
        ));
        spans.push(Span::styled("stale", muted_style(app.color_enabled)));
    }
    let widget = Paragraph::new(Line::from(spans))
        .alignment(Alignment::Right)
        .style(page_style(app.color_enabled));
    frame.render_widget(widget, area);
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
            Constraint::Length(1),
            Constraint::Length(1),
            Constraint::Min(6),
        ])
        .split(area);

    draw_detail_header(frame, sections[0], app);
    draw_detail_tabs(frame, sections[1], app);
    draw_detail_body(frame, sections[2], app);
}

fn draw_detail_header(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    frame.render_widget(
        Block::default().style(object_strip_style(app.color_enabled)),
        area,
    );

    let (left_line, chip_line, chip_width) = if let Some(header) = app.active_header() {
        (
            build_detail_header_line(app, header),
            Some(Line::from(detail_chip_spans(
                &header.chips,
                app.color_enabled,
            ))),
            detail_chip_width(&header.chips),
        )
    } else {
        (
            Line::from(vec![
                Span::styled(app.current_detail_title(), header_style(app.color_enabled)),
                Span::styled(TAB_SEPARATOR, separator_style(app.color_enabled)),
                Span::styled("Loading current selection…", muted_style(app.color_enabled)),
            ]),
            None,
            0,
        )
    };

    let strip = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Min(0),
            Constraint::Length(chip_width.min(area.width)),
        ])
        .split(area);

    let left = Paragraph::new(left_line)
        .style(object_strip_style(app.color_enabled))
        .alignment(Alignment::Left);
    frame.render_widget(left, strip[0]);

    if let Some(chips) = chip_line {
        let right = Paragraph::new(chips)
            .style(object_strip_style(app.color_enabled))
            .alignment(Alignment::Right);
        frame.render_widget(right, strip[1]);
    }
}

fn draw_detail_tabs(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let titles = app.current_tab_titles();
    draw_tab_bar(
        frame,
        area,
        &titles,
        app.current_tab_index(),
        app.color_enabled,
    );
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

fn draw_pipeline_summary(
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

fn draw_group_summary(frame: &mut Frame<'_>, area: Rect, pane: &GroupSummaryPane, app: &AppState) {
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

fn draw_engine_summary(
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

fn draw_bundle_pane(frame: &mut Frame<'_>, area: Rect, pane: &BundlePane, app: &AppState) {
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

fn draw_stat_cards(frame: &mut Frame<'_>, area: Rect, cards: &[StatCard], color_enabled: bool) {
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

fn draw_empty_overlay(frame: &mut Frame<'_>, area: Rect, message: &str, color_enabled: bool) {
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
        .style(panel_style(app.color_enabled))
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
        .style(panel_style(app.color_enabled))
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
        .style(panel_style(app.color_enabled))
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
    .style(panel_style(app.color_enabled))
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
        .style(panel_style(app.color_enabled))
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
    .style(panel_style(app.color_enabled))
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
    .style(panel_style(app.color_enabled))
    .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(widget, overlay);
}

fn draw_tab_bar(
    frame: &mut Frame<'_>,
    area: Rect,
    labels: &[&str],
    selected: usize,
    color_enabled: bool,
) {
    let mut spans = Vec::new();
    for (index, label) in labels.iter().enumerate() {
        let style = if index == selected {
            selected_tab_style(color_enabled)
        } else {
            tab_style(color_enabled)
        };
        spans.push(Span::styled(format!(" {label} "), style));
        if index + 1 < labels.len() {
            spans.push(Span::styled(TAB_SEPARATOR, separator_style(color_enabled)));
        }
    }

    let widget = Paragraph::new(Line::from(spans))
        .style(page_style(color_enabled))
        .wrap(Wrap { trim: true });
    frame.render_widget(widget, area);
}

pub(crate) fn tab_regions(area: Rect, labels: &[&str]) -> Vec<Rect> {
    if area.height == 0 || labels.is_empty() {
        return Vec::new();
    }

    let mut x = area.x;
    let right = area.x.saturating_add(area.width);
    let mut regions = Vec::with_capacity(labels.len());
    for label in labels {
        let width = (label.chars().count() as u16).saturating_add(2);
        if x >= right {
            break;
        }
        let capped_width = width.min(right.saturating_sub(x));
        if capped_width > 0 {
            regions.push(Rect::new(x, area.y, capped_width, 1));
        }
        x = x.saturating_add(width).saturating_add(TAB_SEPARATOR_WIDTH);
    }
    regions
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
        .border_type(BorderType::Rounded)
        .border_style(border_style)
        .style(panel_style(color_enabled))
        .title(Span::styled(title, header_style(color_enabled)))
}

fn card_block(tone: Tone, color_enabled: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(card_border_style(tone, color_enabled))
        .style(card_panel_style(tone, color_enabled))
}

fn page_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(230, 238, 246))
            .bg(Color::Rgb(9, 14, 20))
    } else {
        Style::default()
    }
}

fn panel_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(230, 238, 246))
            .bg(Color::Rgb(16, 23, 33))
    } else {
        Style::default()
    }
}

fn subtle_panel_text_style(color_enabled: bool) -> Style {
    panel_style(color_enabled).fg(muted_color(color_enabled))
}

fn card_panel_style(tone: Tone, color_enabled: bool) -> Style {
    if !color_enabled {
        return Style::default();
    }

    let bg = match tone {
        Tone::Accent => Color::Rgb(18, 42, 52),
        Tone::Success => Color::Rgb(19, 39, 28),
        Tone::Warning => Color::Rgb(51, 41, 20),
        Tone::Failure => Color::Rgb(53, 23, 30),
        Tone::Muted => Color::Rgb(21, 27, 35),
        Tone::Neutral => Color::Rgb(18, 26, 36),
    };
    Style::default().fg(body_color(color_enabled)).bg(bg)
}

fn body_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(body_color(color_enabled))
    } else {
        Style::default()
    }
}

fn title_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(140, 224, 255))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn open_brand_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(245, 168, 0))
            .add_modifier(Modifier::BOLD)
    } else {
        title_style(color_enabled)
    }
}

fn telemetry_brand_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(66, 92, 199))
            .add_modifier(Modifier::BOLD)
    } else {
        title_style(color_enabled)
    }
}

fn header_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(159, 215, 223))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn table_header_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(177, 233, 223))
            .bg(Color::Rgb(28, 40, 54))
            .add_modifier(Modifier::BOLD)
    } else {
        header_style(color_enabled)
    }
}

fn muted_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(muted_color(color_enabled))
    } else {
        Style::default()
    }
}

fn border_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::Rgb(73, 93, 115))
    } else {
        Style::default()
    }
}

fn focus_border_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(96, 218, 223))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn selected_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(body_color(color_enabled))
            .bg(Color::Rgb(29, 74, 91))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    }
}

fn tab_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(153, 178, 198))
            .bg(Color::Rgb(25, 34, 46))
            .add_modifier(Modifier::BOLD)
    } else {
        header_style(color_enabled)
    }
}

fn selected_tab_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(235, 247, 248))
            .bg(Color::Rgb(33, 94, 110))
            .add_modifier(Modifier::BOLD)
    } else {
        selected_style(color_enabled)
    }
}

fn key_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(255, 208, 117))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn separator_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::Rgb(83, 103, 122))
    } else {
        Style::default()
    }
}

fn target_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(188, 204, 223))
            .add_modifier(Modifier::BOLD)
    } else {
        header_style(color_enabled)
    }
}

fn object_strip_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(body_color(color_enabled))
            .bg(Color::Rgb(18, 26, 36))
    } else {
        Style::default()
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
        Tone::Neutral => Style::default().fg(body_color(color_enabled)),
        Tone::Accent => Style::default()
            .fg(Color::Rgb(93, 218, 233))
            .add_modifier(Modifier::BOLD),
        Tone::Success => Style::default()
            .fg(Color::Rgb(116, 229, 140))
            .add_modifier(Modifier::BOLD),
        Tone::Warning => Style::default()
            .fg(Color::Rgb(247, 198, 102))
            .add_modifier(Modifier::BOLD),
        Tone::Failure => Style::default()
            .fg(Color::Rgb(255, 121, 121))
            .add_modifier(Modifier::BOLD),
        Tone::Muted => Style::default().fg(muted_color(color_enabled)),
    }
}

fn chip_style(tone: Tone, color_enabled: bool) -> Style {
    if !color_enabled {
        return tone_style(tone, color_enabled).add_modifier(Modifier::BOLD);
    }

    let (fg, bg) = match tone {
        Tone::Neutral => (Color::Rgb(235, 243, 247), Color::Rgb(39, 50, 64)),
        Tone::Accent => (Color::Rgb(235, 247, 248), Color::Rgb(33, 94, 110)),
        Tone::Success => (Color::Rgb(234, 247, 237), Color::Rgb(34, 103, 53)),
        Tone::Warning => (Color::Rgb(255, 244, 217), Color::Rgb(112, 81, 23)),
        Tone::Failure => (Color::Rgb(255, 236, 236), Color::Rgb(125, 43, 55)),
        Tone::Muted => (Color::Rgb(225, 231, 238), Color::Rgb(65, 79, 96)),
    };
    Style::default().fg(fg).bg(bg).add_modifier(Modifier::BOLD)
}

fn stripe_style(index: usize, color_enabled: bool) -> Style {
    if color_enabled && index % 2 == 1 {
        Style::default().bg(Color::Rgb(21, 30, 42))
    } else {
        Style::default().bg(if color_enabled {
            Color::Rgb(16, 23, 33)
        } else {
            Color::Reset
        })
    }
}

fn card_border_style(tone: Tone, color_enabled: bool) -> Style {
    if !color_enabled {
        return Style::default();
    }

    let color = match tone {
        Tone::Accent => Color::Rgb(67, 153, 176),
        Tone::Success => Color::Rgb(71, 142, 89),
        Tone::Warning => Color::Rgb(169, 128, 53),
        Tone::Failure => Color::Rgb(168, 73, 89),
        Tone::Muted => Color::Rgb(84, 96, 111),
        Tone::Neutral => Color::Rgb(91, 112, 134),
    };
    Style::default().fg(color)
}

fn body_color(color_enabled: bool) -> Color {
    if color_enabled {
        Color::Rgb(230, 238, 246)
    } else {
        Color::Reset
    }
}

fn muted_color(color_enabled: bool) -> Color {
    if color_enabled {
        Color::Rgb(129, 145, 163)
    } else {
        Color::Reset
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

fn build_detail_header_line(app: &AppState, header: &super::app::DetailHeader) -> Line<'static> {
    let mut spans = vec![Span::styled(
        app.current_detail_title(),
        header_style(app.color_enabled),
    )];

    if !header.title.is_empty() {
        spans.push(Span::styled(
            TAB_SEPARATOR,
            separator_style(app.color_enabled),
        ));
        spans.push(Span::styled(
            header.title.clone(),
            title_style(app.color_enabled),
        ));
    }

    if let Some(subtitle) = header.subtitle.as_deref() {
        if !subtitle.is_empty() {
            spans.push(Span::styled(
                TAB_SEPARATOR,
                separator_style(app.color_enabled),
            ));
            spans.push(Span::styled(
                subtitle.to_string(),
                muted_style(app.color_enabled),
            ));
        }
    }

    Line::from(spans)
}

fn detail_chip_text(chip: &StatusChip) -> String {
    format!(" {}:{} ", chip.label, chip.value)
}

fn detail_chip_spans(chips: &[StatusChip], color_enabled: bool) -> Vec<Span<'static>> {
    let mut spans = Vec::new();
    for (index, chip) in chips.iter().enumerate() {
        if index > 0 {
            spans.push(Span::raw(" "));
        }
        spans.push(Span::styled(
            detail_chip_text(chip),
            chip_style(chip.tone, color_enabled),
        ));
    }
    spans
}

fn detail_chip_width(chips: &[StatusChip]) -> u16 {
    let mut width: u16 = 0;
    for (index, chip) in chips.iter().enumerate() {
        if index > 0 {
            width = width.saturating_add(1);
        }
        width = width.saturating_add(detail_chip_text(chip).chars().count() as u16);
    }
    width
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
        ConfigPane, DetailHeader, EngineVitals, PipelineSummaryPane, StatCard, StatusChip, Tone,
        UiCommandContext,
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
        assert!(rendered.contains("OpenTelemetry - Rust Dataflow Engine"));
        assert!(rendered.contains("Equivalent CLI"));
        assert!(rendered.contains("telemetry logs watch"));
        assert!(rendered.contains("https://admin.example.com:8443/engine-a"));
    }

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

    #[test]
    fn top_level_view_order_is_engine_groups_pipelines() {
        let titles = View::ALL.map(View::title);
        assert_eq!(titles, ["Engine", "Groups", "Pipelines"]);
    }

    #[test]
    fn tab_regions_account_for_pipe_separators() {
        let regions = tab_regions(Rect::new(0, 0, 40, 1), &["Engine", "Groups", "Pipelines"]);
        assert_eq!(regions.len(), 3);
        assert_eq!(regions[0], Rect::new(0, 0, 8, 1));
        assert_eq!(regions[1], Rect::new(11, 0, 8, 1));
        assert_eq!(regions[2], Rect::new(22, 0, 11, 1));
    }
}
