// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Theme, block, and inline styling helpers for the TUI.

use super::*;

pub(super) fn block_with_title(title: &str, focused: bool, color_enabled: bool) -> Block<'static> {
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

pub(super) fn card_block(tone: Tone, color_enabled: bool) -> Block<'static> {
    Block::default()
        .borders(Borders::ALL)
        .border_type(BorderType::Rounded)
        .border_style(card_border_style(tone, color_enabled))
        .style(card_panel_style(tone, color_enabled))
}

pub(super) fn page_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(230, 238, 246))
            .bg(Color::Rgb(9, 14, 20))
    } else {
        Style::default()
    }
}

pub(super) fn panel_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(230, 238, 246))
            .bg(Color::Rgb(16, 23, 33))
    } else {
        Style::default()
    }
}

pub(super) fn subtle_panel_text_style(color_enabled: bool) -> Style {
    panel_style(color_enabled).fg(muted_color(color_enabled))
}

pub(super) fn card_panel_style(tone: Tone, color_enabled: bool) -> Style {
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

pub(super) fn body_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(body_color(color_enabled))
    } else {
        Style::default()
    }
}

pub(super) fn title_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(140, 224, 255))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

pub(super) fn open_brand_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(245, 168, 0))
            .add_modifier(Modifier::BOLD)
    } else {
        title_style(color_enabled)
    }
}

pub(super) fn telemetry_brand_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(66, 92, 199))
            .add_modifier(Modifier::BOLD)
    } else {
        title_style(color_enabled)
    }
}

pub(super) fn header_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(159, 215, 223))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

pub(super) fn table_header_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(177, 233, 223))
            .bg(Color::Rgb(28, 40, 54))
            .add_modifier(Modifier::BOLD)
    } else {
        header_style(color_enabled)
    }
}

pub(super) fn muted_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(muted_color(color_enabled))
    } else {
        Style::default()
    }
}

pub(super) fn border_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::Rgb(73, 93, 115))
    } else {
        Style::default()
    }
}

pub(super) fn focus_border_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(96, 218, 223))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

pub(super) fn selected_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(body_color(color_enabled))
            .bg(Color::Rgb(29, 74, 91))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::REVERSED | Modifier::BOLD)
    }
}

pub(super) fn tab_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(153, 178, 198))
            .bg(Color::Rgb(25, 34, 46))
            .add_modifier(Modifier::BOLD)
    } else {
        header_style(color_enabled)
    }
}

pub(super) fn selected_tab_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(235, 247, 248))
            .bg(Color::Rgb(33, 94, 110))
            .add_modifier(Modifier::BOLD)
    } else {
        selected_style(color_enabled)
    }
}

pub(super) fn key_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(255, 208, 117))
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().add_modifier(Modifier::BOLD)
    }
}

pub(super) fn separator_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default().fg(Color::Rgb(83, 103, 122))
    } else {
        Style::default()
    }
}

pub(super) fn target_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(Color::Rgb(188, 204, 223))
            .add_modifier(Modifier::BOLD)
    } else {
        header_style(color_enabled)
    }
}

pub(super) fn object_strip_style(color_enabled: bool) -> Style {
    if color_enabled {
        Style::default()
            .fg(body_color(color_enabled))
            .bg(Color::Rgb(18, 26, 36))
    } else {
        Style::default()
    }
}

pub(super) fn tone_style(tone: Tone, color_enabled: bool) -> Style {
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

pub(super) fn chip_style(tone: Tone, color_enabled: bool) -> Style {
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

pub(super) fn stripe_style(index: usize, color_enabled: bool) -> Style {
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

pub(super) fn card_border_style(tone: Tone, color_enabled: bool) -> Style {
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

pub(super) fn body_color(color_enabled: bool) -> Color {
    if color_enabled {
        Color::Rgb(230, 238, 246)
    } else {
        Color::Reset
    }
}

pub(super) fn muted_color(color_enabled: bool) -> Color {
    if color_enabled {
        Color::Rgb(129, 145, 163)
    } else {
        Color::Reset
    }
}

pub(super) fn badge_cell(text: &str, tone: Tone, color_enabled: bool) -> Cell<'static> {
    Cell::from(Span::styled(
        format!(" {text} "),
        chip_style(tone, color_enabled),
    ))
}

pub(super) fn tone_badge(tone: Tone) -> &'static str {
    match tone {
        Tone::Accent => "roll",
        Tone::Success => "ok",
        Tone::Warning => "warn",
        Tone::Failure => "fail",
        Tone::Muted => "stop",
        Tone::Neutral => "info",
    }
}

pub(super) fn card(label: impl Into<String>, value: impl Into<String>, tone: Tone) -> StatCard {
    StatCard {
        label: label.into(),
        value: value.into(),
        tone,
    }
}

pub(super) fn build_detail_header_line(app: &AppState, header: &DetailHeader) -> Line<'static> {
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

pub(super) fn detail_chip_text(chip: &StatusChip) -> String {
    format!(" {}:{} ", chip.label, chip.value)
}

pub(super) fn detail_chip_spans(chips: &[StatusChip], color_enabled: bool) -> Vec<Span<'static>> {
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

pub(super) fn detail_chip_width(chips: &[StatusChip]) -> u16 {
    let mut width: u16 = 0;
    for (index, chip) in chips.iter().enumerate() {
        if index > 0 {
            width = width.saturating_add(1);
        }
        width = width.saturating_add(detail_chip_text(chip).chars().count() as u16);
    }
    width
}

pub(super) fn slice<T>(rows: &[T], offset: usize, limit: Option<usize>, area_rows: usize) -> &[T] {
    if rows.is_empty() {
        return rows;
    }
    let max_rows = limit.unwrap_or(area_rows.max(1)).min(area_rows.max(1));
    let capped_offset = offset.min(rows.len().saturating_sub(1));
    let end = (capped_offset + max_rows).min(rows.len());
    &rows[capped_offset..end]
}
