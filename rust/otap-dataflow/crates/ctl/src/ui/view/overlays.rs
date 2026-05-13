// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Modal overlays and the footer status bar for the TUI.
//!
//! Overlay rendering handles transient UI surfaces that sit above the main
//! panes: help, equivalent commands, command palette, filters, action menus,
//! confirmations, and scale editors. Grouping them here keeps modal visual
//! behavior consistent and avoids scattering `Clear` overlays through the main
//! renderer.

use super::*;
use crate::BIN_NAME;

/// Draw the footer status bar with navigation hints, refresh age, filter state, and last error.
pub(super) fn draw_status_bar(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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
        Span::styled(":", key_style(app.color_enabled)),
        Span::raw(" palette  "),
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

/// Draw the keyboard shortcut help overlay.
pub(super) fn draw_help_overlay(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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
        Line::from(": open searchable command palette"),
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
        .block(block_with_title(
            &format!("{BIN_NAME} ui help"),
            false,
            app.color_enabled,
        ))
        .style(panel_style(app.color_enabled))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(help, overlay);
}

/// Draw the overlay explaining the equivalent non-interactive command for the current pane.
pub(super) fn draw_command_overlay(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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

/// Draw the searchable command palette overlay.
pub(super) fn draw_command_palette_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    palette: &CommandPaletteState,
    app: &AppState,
) {
    let overlay = centered_rect(72, 66, area);
    let entries = app.filtered_palette_entries();
    let selected = palette.selected.min(entries.len().saturating_sub(1));
    let visible = entries
        .iter()
        .enumerate()
        .skip(selected.saturating_sub(5))
        .take(10)
        .collect::<Vec<_>>();

    let mut lines = vec![
        Line::from(Span::styled(
            "Command Palette",
            title_style(app.color_enabled),
        )),
        Line::from(Span::styled(
            "Type to filter commands, views, tabs, and current-selection actions.",
            muted_style(app.color_enabled),
        )),
        Line::from(""),
        Line::from(vec![
            Span::styled("> ", key_style(app.color_enabled)),
            Span::raw(palette.input.clone()),
        ]),
        Line::from(""),
    ];

    if visible.is_empty() {
        lines.push(Line::from(Span::styled(
            "No commands match the current filter.",
            muted_style(app.color_enabled),
        )));
    } else {
        for (index, entry) in visible {
            let prefix = if index == selected { "▶ " } else { "  " };
            let label_style = if !entry.enabled {
                muted_style(app.color_enabled)
            } else if index == selected {
                selected_style(app.color_enabled)
            } else {
                header_style(app.color_enabled)
            };
            lines.push(Line::from(Span::styled(
                format!("{prefix}{}", entry.label),
                label_style,
            )));
            lines.push(Line::from(Span::styled(
                format!("   {}", entry.detail),
                muted_style(app.color_enabled),
            )));
        }
    }

    lines.push(Line::from(""));
    lines.push(Line::from(Span::styled(
        "Enter runs the selected command. Esc closes. Up/Down or j/k changes selection.",
        muted_style(app.color_enabled),
    )));

    let widget = Paragraph::new(lines)
        .block(block_with_title("Palette", false, app.color_enabled))
        .style(panel_style(app.color_enabled))
        .wrap(Wrap { trim: false });
    frame.render_widget(Clear, overlay);
    frame.render_widget(widget, overlay);
}

/// Draw the resource-list filter input overlay.
pub(super) fn draw_filter_overlay(frame: &mut Frame<'_>, area: Rect, app: &AppState) {
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

/// Draw the context action menu for the currently selected object.
pub(super) fn draw_action_menu_overlay(
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

/// Draw the destructive-action confirmation overlay.
pub(super) fn draw_shutdown_confirm_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    confirm: &ShutdownConfirmState,
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

/// Draw the core-count editor used by pipeline scale actions.
pub(super) fn draw_scale_editor_overlay(
    frame: &mut Frame<'_>,
    area: Rect,
    editor: &ScaleEditorState,
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
