// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Layout, hit-testing, and shared tab geometry for the TUI.

use super::*;

pub(crate) const MIN_TERMINAL_WIDTH: u16 = 100;
pub(crate) const MIN_TERMINAL_HEIGHT: u16 = 25;
pub(super) const HEADER_HEIGHT: u16 = 2;
pub(super) const STATUS_BAR_HEIGHT: u16 = 2;
pub(super) const DETAIL_HEADER_HEIGHT: u16 = 1;
pub(super) const DETAIL_TABS_HEIGHT: u16 = 1;
pub(super) const ENGINE_VITALS_WIDTH: u16 = 38;
pub(super) const TAB_SEPARATOR: &str = " | ";
pub(super) const TAB_SEPARATOR_WIDTH: u16 = 3;

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

pub(super) fn draw_tab_bar(
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

pub(super) fn centered_rect(percent_x: u16, percent_y: u16, area: Rect) -> Rect {
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
