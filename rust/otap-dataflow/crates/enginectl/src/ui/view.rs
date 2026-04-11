// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::app::{AppState, FocusArea, View};
use ratatui::{
    Frame,
    layout::{Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Tabs, Wrap},
};

const MIN_TERMINAL_WIDTH: u16 = 100;
const MIN_TERMINAL_HEIGHT: u16 = 24;

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
            Constraint::Min(8),
            Constraint::Length(2),
        ])
        .split(area);

    draw_top_tabs(frame, sections[0], app);

    let body = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(38), Constraint::Percentage(62)])
        .split(sections[1]);

    draw_left_list(frame, body[0], app);
    draw_detail(frame, body[1], app);
    draw_status_bar(frame, sections[2], app);

    if app.show_help {
        draw_help_overlay(frame, area, app);
    }
    if app.filter_mode {
        draw_filter_overlay(frame, area, app);
    }
}

fn draw_resize_required(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let message = Paragraph::new(format!(
        "dfctl ui needs at least {MIN_TERMINAL_WIDTH}x{MIN_TERMINAL_HEIGHT}\ncurrent size: {}x{}\n\nResize the terminal and try again.",
        area.width, area.height
    ))
    .block(
        Block::default()
            .title(title_span("Terminal Too Small", app.color_enabled))
            .borders(Borders::ALL),
    )
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
        .block(Block::default().title("Views").borders(Borders::ALL))
        .select(selected)
        .highlight_style(selected_style(app.color_enabled))
        .style(base_style(app.color_enabled));
    frame.render_widget(tabs, area);
}

fn draw_left_list(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    match app.view {
        View::Pipelines => {
            let items = app.pipeline_items();
            let rows = items
                .iter()
                .map(|item| {
                    Row::new(vec![
                        Cell::from(item.pipeline_group_id.clone()),
                        Cell::from(item.pipeline_id.clone()),
                        Cell::from(item.running.clone()),
                        Cell::from(item.active_generation.clone()),
                        Cell::from(item.rollout.clone()),
                    ])
                })
                .collect::<Vec<_>>();
            let selected = items
                .iter()
                .position(|item| Some(item.key.as_str()) == app.pipeline_selected.as_deref());
            draw_table(
                frame,
                area,
                app,
                rows,
                vec![
                    "group".to_string(),
                    "pipeline".to_string(),
                    "running".to_string(),
                    "active_gen".to_string(),
                    "rollout".to_string(),
                ],
                [
                    Constraint::Percentage(24),
                    Constraint::Percentage(32),
                    Constraint::Length(10),
                    Constraint::Length(12),
                    Constraint::Length(12),
                ],
                selected,
            );
        }
        View::Groups => {
            let items = app.group_items();
            let rows = items
                .iter()
                .map(|item| {
                    Row::new(vec![
                        Cell::from(item.group_id.clone()),
                        Cell::from(item.pipelines.to_string()),
                        Cell::from(item.running.to_string()),
                        Cell::from(item.ready.to_string()),
                        Cell::from(item.terminal.to_string()),
                    ])
                })
                .collect::<Vec<_>>();
            let selected = items
                .iter()
                .position(|item| Some(item.group_id.as_str()) == app.group_selected.as_deref());
            draw_table(
                frame,
                area,
                app,
                rows,
                vec![
                    "group".to_string(),
                    "pipelines".to_string(),
                    "running".to_string(),
                    "ready".to_string(),
                    "terminal".to_string(),
                ],
                [
                    Constraint::Percentage(46),
                    Constraint::Length(10),
                    Constraint::Length(9),
                    Constraint::Length(7),
                    Constraint::Length(10),
                ],
                selected,
            );
        }
        View::Engine => {
            let items = app.engine_pipeline_items();
            let rows = items
                .iter()
                .map(|item| {
                    Row::new(vec![
                        Cell::from(item.key.clone()),
                        Cell::from(item.running.clone()),
                        Cell::from(item.active_generation.clone()),
                        Cell::from(item.rollout.clone()),
                    ])
                })
                .collect::<Vec<_>>();
            let selected = items
                .iter()
                .position(|item| Some(item.key.as_str()) == app.engine_selected.as_deref());
            draw_table(
                frame,
                area,
                app,
                rows,
                vec![
                    "pipeline".to_string(),
                    "running".to_string(),
                    "active_gen".to_string(),
                    "rollout".to_string(),
                ],
                [
                    Constraint::Percentage(54),
                    Constraint::Length(10),
                    Constraint::Length(12),
                    Constraint::Length(12),
                ],
                selected,
            );
        }
    }
}

fn draw_table<const N: usize>(
    frame: &mut Frame<'_>,
    area: Rect,
    app: &AppState,
    rows: Vec<Row<'static>>,
    headers: Vec<String>,
    widths: [Constraint; N],
    selected: Option<usize>,
) {
    if rows.is_empty() {
        let placeholder = Paragraph::new(if app.filter_query.is_empty() {
            "No resources available."
        } else {
            "No resources match the current filter."
        })
        .block(
            Block::default()
                .title(app.current_list_title())
                .borders(Borders::ALL),
        );
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
        .block(
            Block::default()
                .title(format!(
                    "{}{}",
                    app.current_list_title(),
                    if app.focus == FocusArea::List {
                        " [focus]"
                    } else {
                        ""
                    }
                ))
                .borders(Borders::ALL),
        )
        .column_spacing(1)
        .row_highlight_style(selected_style(app.color_enabled))
        .highlight_symbol(">> ");
    let mut state = TableState::default();
    state.select(selected);
    frame.render_stateful_widget(table, area, &mut state);
}

fn draw_detail(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let sections = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);

    let tabs = Tabs::new(
        app.current_tab_titles()
            .into_iter()
            .map(Line::from)
            .collect::<Vec<_>>(),
    )
    .block(Block::default().title("Detail").borders(Borders::ALL))
    .select(app.current_tab_index())
    .highlight_style(selected_style(app.color_enabled))
    .style(base_style(app.color_enabled));
    frame.render_widget(tabs, sections[0]);

    let text = if app.active_detail_text().is_empty() {
        "Loading…"
    } else {
        app.active_detail_text()
    };
    let paragraph = Paragraph::new(text.to_string())
        .block(
            Block::default()
                .title(format!(
                    "{}{}",
                    app.active_detail_title(),
                    if app.focus == FocusArea::Detail {
                        " [focus]"
                    } else {
                        ""
                    }
                ))
                .borders(Borders::ALL),
        )
        .style(base_style(app.color_enabled))
        .wrap(Wrap { trim: false })
        .scroll((app.detail_scroll, 0));
    frame.render_widget(paragraph, sections[1]);
}

fn draw_status_bar(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let age = app
        .last_refresh
        .map(|instant| format!("{}s ago", instant.elapsed().as_secs()))
        .unwrap_or_else(|| "never".to_string());
    let filter = if app.filter_query.is_empty() {
        "none".to_string()
    } else {
        app.filter_query.clone()
    };

    let mut spans = vec![
        Span::styled("Tab/Shift-Tab", key_style(app.color_enabled)),
        Span::raw(" views  "),
        Span::styled("Enter", key_style(app.color_enabled)),
        Span::raw(" focus  "),
        Span::styled("Backspace", key_style(app.color_enabled)),
        Span::raw(" back  "),
        Span::styled("/", key_style(app.color_enabled)),
        Span::raw(" filter  "),
        Span::styled("r", key_style(app.color_enabled)),
        Span::raw(" refresh  "),
        Span::styled("?", key_style(app.color_enabled)),
        Span::raw(" help  "),
        Span::styled("q", key_style(app.color_enabled)),
        Span::raw(" quit"),
        Span::raw(format!("  refresh={age} filter={filter}")),
    ];
    if let Some(error) = &app.last_error {
        spans.push(Span::raw("  "));
        spans.push(Span::styled(
            format!("error: {error}"),
            error_style(app.color_enabled),
        ));
    }

    let footer = Paragraph::new(Line::from(spans))
        .block(Block::default().borders(Borders::ALL))
        .wrap(Wrap { trim: false });
    frame.render_widget(footer, area);
}

fn draw_help_overlay(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let overlay = centered_rect(70, 70, area);
    let lines = vec![
        Line::from("Navigation"),
        Line::from("Tab / Shift-Tab: switch top-level views"),
        Line::from("Up/Down or j/k: move list selection or scroll detail"),
        Line::from("Left/Right or h/l: switch detail tabs"),
        Line::from("Enter: focus the detail pane"),
        Line::from("Backspace: return focus to the list"),
        Line::from("g / G: jump to first / last row"),
        Line::from("PgUp / PgDn: page list or detail"),
        Line::from(""),
        Line::from("Troubleshooting"),
        Line::from("e: events tab"),
        Line::from("l: logs tab"),
        Line::from("m: metrics tab"),
        Line::from("d: diagnose tab"),
        Line::from("b: bundle tab"),
        Line::from("r: refresh now"),
        Line::from(""),
        Line::from("Other"),
        Line::from("/: filter current list"),
        Line::from("Esc: clear filter / close modal"),
        Line::from("q: quit"),
    ];
    let help = Paragraph::new(lines)
        .block(
            Block::default()
                .title(title_span("dfctl ui help", app.color_enabled))
                .borders(Borders::ALL),
        )
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(help, overlay);
}

fn draw_filter_overlay(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
    let overlay = centered_rect(60, 20, area);
    let text = Paragraph::new(vec![
        Line::from("Filter the current resource list."),
        Line::from("Press Enter to apply, Esc to cancel."),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", key_style(app.color_enabled)),
            Span::raw(app.filter_input.clone()),
        ]),
    ])
    .block(
        Block::default()
            .title(title_span("Filter", app.color_enabled))
            .borders(Borders::ALL),
    )
    .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(text, overlay);
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

fn base_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::Cyan)
    } else {
        Style::default()
    }
}

fn header_style(color_enabled: bool) -> Style {
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
            .fg(Color::Yellow)
            .add_modifier(Modifier::BOLD | Modifier::REVERSED)
    } else {
        Style::default().add_modifier(Modifier::REVERSED)
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

fn error_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

fn title_span<'a>(text: &'a str, color_enabled: bool) -> Line<'a> {
    Line::from(Span::styled(text, header_style(color_enabled)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::args::UiStartView;
    use crate::ui::app::AppState;
    use ratatui::{Terminal, backend::TestBackend};

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
}
