// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Top-level chrome and primary view dispatch for the TUI.
//!
//! Chrome rendering owns the frame-level composition: title bar, target, engine
//! vitals, first-level tabs, resource list, detail body, status bar, and modal
//! overlays. It is the only renderer that knows how the major UI regions fit
//! together; specialized pane and table rendering stays delegated to sibling
//! modules.

use super::*;
use crate::BIN_NAME;

/// Draw the full TUI frame, including chrome, the active view, and any modal overlays.
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
    if let Some(palette) = app.command_palette() {
        draw_command_palette_overlay(frame, area, palette, app);
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

/// Draw the two-line header containing the title, target, vitals, and top-level tabs.
pub(super) fn draw_header(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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

/// Draw the fallback message shown when the terminal is too small for the normal layout.
pub(super) fn draw_resize_required(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let message = Paragraph::new(format!(
        "{BIN_NAME} ui needs at least {MIN_TERMINAL_WIDTH}x{MIN_TERMINAL_HEIGHT}\ncurrent size: {}x{}\n\nResize the terminal and try again.",
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

/// Draw the first-level Engine, Groups, and Pipelines tab bar.
pub(super) fn draw_top_tabs(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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

/// Draw the title, OpenTelemetry brand label, and current target URL.
pub(super) fn draw_title_bar(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let mut spans = brand_spans(app);
    spans.extend([
        Span::styled(" - Rust Dataflow Engine", title_style(app.color_enabled)),
        Span::styled("  |  ", separator_style(app.color_enabled)),
        Span::styled("target ", muted_style(app.color_enabled)),
        Span::styled(app.target_url(), target_style(app.color_enabled)),
    ]);
    let line = Line::from(spans);
    let title = Paragraph::new(line)
        .alignment(Alignment::Left)
        .style(page_style(app.color_enabled));
    frame.render_widget(title, area);
}

/// Build the styled OpenTelemetry brand spans, shimmering them while background activity runs.
pub(super) fn brand_spans(app: &AppState) -> Vec<Span<'static>> {
    if !app.color_enabled || !app.is_activity_active() {
        return vec![
            Span::styled("Open", open_brand_style(app.color_enabled)),
            Span::styled("Telemetry", telemetry_brand_style(app.color_enabled)),
        ];
    }

    "OpenTelemetry"
        .chars()
        .enumerate()
        .map(|(index, character)| {
            Span::styled(
                character.to_string(),
                shimmering_brand_style(index, app.activity_frame()),
            )
        })
        .collect()
}

fn shimmering_brand_style(index: usize, frame: u16) -> Style {
    let base_style = if index < "Open".len() {
        open_brand_style(true)
    } else {
        telemetry_brand_style(true)
    };
    let brand_len = "OpenTelemetry".len();
    let cycle_len = brand_len + 4;
    let center = (usize::from(frame) % cycle_len) as isize - 2;
    let distance = (index as isize - center).unsigned_abs();

    match distance {
        0 => base_style
            .fg(Color::Rgb(238, 250, 255))
            .bg(Color::Rgb(35, 72, 86))
            .add_modifier(Modifier::BOLD),
        1 => base_style
            .fg(Color::Rgb(170, 231, 232))
            .add_modifier(Modifier::BOLD),
        2 => base_style.fg(Color::Rgb(113, 177, 211)),
        _ => base_style,
    }
}

/// Draw the compact engine CPU and memory strip in the title bar.
pub(super) fn draw_engine_vitals_panel(
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

/// Draw the selectable object list for the current top-level view.
pub(super) fn draw_left_list(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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

/// Draw the selected object's detail area.
pub(super) fn draw_detail(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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

/// Draw the detail header line and status chips for the selected object.
pub(super) fn draw_detail_header(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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

/// Draw the second-level tab bar for the selected object.
pub(super) fn draw_detail_tabs(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let titles = app.current_tab_titles();
    draw_tab_bar(
        frame,
        area,
        &titles,
        app.current_tab_index(),
        app.color_enabled,
    );
}

/// Dispatch drawing to the active detail pane for the selected view and tab.
pub(super) fn draw_detail_body(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    match app.view {
        View::Pipelines => match app.pipeline_tab {
            PipelineTab::Summary => draw_pipeline_summary(frame, area, &app.pipelines.summary, app),
            PipelineTab::Details => {
                draw_object_details_pane(frame, area, &app.pipelines.details, app)
            }
            PipelineTab::Config => draw_config_pane(frame, area, &app.pipelines.config, app),
            PipelineTab::Events => draw_events_pane(frame, area, &app.pipelines.events, app),
            PipelineTab::Logs => draw_logs_pane(frame, area, &app.pipelines.logs, app),
            PipelineTab::Metrics => draw_metrics_pane(frame, area, &app.pipelines.metrics, app),
            PipelineTab::Rollout => draw_operation_pane(frame, area, &app.pipelines.rollout, app),
            PipelineTab::Shutdown => draw_operation_pane(frame, area, &app.pipelines.shutdown, app),
            PipelineTab::Diagnose => {
                draw_diagnosis_pane(frame, area, &app.pipelines.diagnosis, app)
            }
            PipelineTab::Bundle => draw_bundle_pane(frame, area, &app.pipelines.bundle, app),
        },
        View::Groups => match app.group_tab {
            GroupTab::Summary => draw_group_summary(frame, area, &app.groups.summary, app),
            GroupTab::Details => draw_object_details_pane(frame, area, &app.groups.details, app),
            GroupTab::Events => draw_events_pane(frame, area, &app.groups.events, app),
            GroupTab::Logs => draw_logs_pane(frame, area, &app.groups.logs, app),
            GroupTab::Metrics => draw_metrics_pane(frame, area, &app.groups.metrics, app),
            GroupTab::Shutdown => draw_group_shutdown_pane(frame, area, &app.groups.shutdown, app),
            GroupTab::Diagnose => draw_diagnosis_pane(frame, area, &app.groups.diagnosis, app),
            GroupTab::Bundle => draw_bundle_pane(frame, area, &app.groups.bundle, app),
        },
        View::Engine => match app.engine_tab {
            EngineTab::Summary => draw_engine_summary(frame, area, &app.engine.summary, app),
            EngineTab::Details => draw_object_details_pane(frame, area, &app.engine.details, app),
            EngineTab::Logs => draw_logs_pane(frame, area, &app.engine.logs, app),
            EngineTab::Metrics => draw_metrics_pane(frame, area, &app.engine.metrics, app),
        },
    }
}
