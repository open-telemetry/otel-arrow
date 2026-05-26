// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Keyboard and mouse input routing for the TUI.
//!
//! Input handling translates low-level Crossterm events into state transitions
//! and high-level outcomes for the UI loop. The module owns keyboard shortcuts,
//! mouse hit-testing, modal-specific input behavior, and refresh/action
//! requests while leaving mutation execution and rendering to their dedicated
//! modules.

use super::*;

/// Routes one terminal event to keyboard, mouse, or resize handling.
pub(super) fn handle_event(event: Event, app: &mut AppState) -> EventOutcome {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => handle_key_event(key, app),
        Event::Mouse(mouse) => handle_mouse_event(mouse, app),
        Event::Resize(width, height) => {
            app.set_terminal_size(width, height);
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}

fn handle_mouse_event(mouse: MouseEvent, app: &mut AppState) -> EventOutcome {
    if app.is_filter_mode()
        || app.scale_editor().is_some()
        || app.shutdown_confirm().is_some()
        || app.action_menu().is_some()
        || app.show_help()
        || app.show_command_overlay()
        || app.show_command_palette()
    {
        return EventOutcome::Continue;
    }

    match mouse.kind {
        MouseEventKind::Down(MouseButton::Left) => handle_left_click(mouse.column, mouse.row, app),
        MouseEventKind::ScrollUp => handle_mouse_scroll(mouse.column, mouse.row, app, -3),
        MouseEventKind::ScrollDown => handle_mouse_scroll(mouse.column, mouse.row, app, 3),
        _ => EventOutcome::Continue,
    }
}

fn handle_left_click(column: u16, row: u16, app: &mut AppState) -> EventOutcome {
    let Some((width, height)) = app.terminal_size else {
        return EventOutcome::Continue;
    };
    let Some(layout) = compute_ui_layout(Rect::new(0, 0, width, height)) else {
        return EventOutcome::Continue;
    };

    let view_titles = View::ALL
        .iter()
        .map(|view| view.title())
        .collect::<Vec<_>>();
    if let Some(index) = tab_hit_index(layout.top_tabs, &view_titles, column, row) {
        let view = View::ALL[index];
        if app.view != view {
            app.hide_modal();
            app.select_view(view);
            return EventOutcome::Refresh;
        }
        return EventOutcome::Continue;
    }

    let detail_titles = app.current_tab_titles();
    if let Some(index) = tab_hit_index(layout.detail_tabs, &detail_titles, column, row) {
        let changed = index != app.current_tab_index() || app.focus != FocusArea::Detail;
        app.select_current_tab(index);
        return if changed {
            EventOutcome::Refresh
        } else {
            EventOutcome::Continue
        };
    }

    if rect_contains(layout.list, column, row) {
        return handle_list_click(layout.list, row, app);
    }

    if rect_contains(layout.detail, column, row) {
        let changed = app.focus != FocusArea::Detail;
        app.focus = FocusArea::Detail;
        return if changed {
            EventOutcome::Refresh
        } else {
            EventOutcome::Continue
        };
    }

    EventOutcome::Continue
}

fn handle_list_click(area: Rect, row: u16, app: &mut AppState) -> EventOutcome {
    let item_count = match app.view {
        View::Pipelines => app.pipeline_items().len(),
        View::Groups => app.group_items().len(),
        View::Engine => app.engine_pipeline_items().len(),
    };
    let Some(index) = state_table_row_hit_index(area, row, item_count) else {
        let changed = app.focus != FocusArea::List;
        app.focus = FocusArea::List;
        return if changed {
            EventOutcome::Refresh
        } else {
            EventOutcome::Continue
        };
    };

    let selection_changed = app.select_list_index(index);
    let focus_changed = app.focus != FocusArea::List;
    app.focus = FocusArea::List;
    app.reset_scroll();

    match app.view {
        View::Pipelines | View::Groups if selection_changed || focus_changed => {
            EventOutcome::Refresh
        }
        View::Engine => EventOutcome::Continue,
        _ => EventOutcome::Continue,
    }
}

fn handle_mouse_scroll(column: u16, row: u16, app: &mut AppState, delta: isize) -> EventOutcome {
    let Some((width, height)) = app.terminal_size else {
        return handle_vertical_motion(app, delta);
    };
    let Some(layout) = compute_ui_layout(Rect::new(0, 0, width, height)) else {
        return handle_vertical_motion(app, delta);
    };

    if rect_contains(layout.list, column, row) {
        app.focus = FocusArea::List;
        app.move_selection(delta);
        app.reset_scroll();
        return match app.view {
            View::Pipelines | View::Groups => EventOutcome::Refresh,
            View::Engine => EventOutcome::Continue,
        };
    }

    if rect_contains(layout.detail, column, row) {
        app.focus = FocusArea::Detail;
        return handle_vertical_motion(app, delta);
    }

    handle_vertical_motion(app, delta)
}

fn rect_contains(area: Rect, column: u16, row: u16) -> bool {
    column >= area.x
        && column < area.x.saturating_add(area.width)
        && row >= area.y
        && row < area.y.saturating_add(area.height)
}

/// Handles one key press and returns the resulting UI-loop action.
pub(super) fn handle_key_event(key: KeyEvent, app: &mut AppState) -> EventOutcome {
    if app.is_filter_mode() {
        return handle_filter_key(key, app);
    }

    if app.scale_editor().is_some() {
        return handle_scale_editor_key(key, app);
    }

    if app.shutdown_confirm().is_some() {
        return handle_shutdown_confirm_key(key, app);
    }

    if app.action_menu().is_some() {
        return handle_action_menu_key(key, app);
    }

    if app.show_command_palette() {
        return handle_command_palette_key(key, app);
    }

    if app.show_command_overlay() {
        return match key.code {
            KeyCode::Char('q') => EventOutcome::Quit,
            KeyCode::Esc | KeyCode::Char('c') => {
                app.hide_modal();
                EventOutcome::Continue
            }
            KeyCode::Char('?') => {
                app.toggle_help();
                EventOutcome::Continue
            }
            KeyCode::Char('/') => {
                app.start_filter_input();
                EventOutcome::Continue
            }
            _ => EventOutcome::Continue,
        };
    }

    if app.view == View::Pipelines
        && app.focus == FocusArea::Detail
        && app.pipeline_tab == PipelineTab::Config
    {
        match key.code {
            KeyCode::Char('e') => {
                let Some((group_id, pipeline_id)) = app.selected_pipeline_target() else {
                    return EventOutcome::Continue;
                };
                return EventOutcome::OpenPipelineEditor {
                    group_id,
                    pipeline_id,
                };
            }
            KeyCode::Char('d') => {
                app.discard_pipeline_config_draft();
                app.last_error = None;
                return EventOutcome::Refresh;
            }
            KeyCode::Enter if app.has_deployable_pipeline_config_draft() => {
                let Some((group_id, pipeline_id)) = app.selected_pipeline_target() else {
                    return EventOutcome::Continue;
                };
                return EventOutcome::Execute(UiAction::PipelineDeployDraft {
                    group_id,
                    pipeline_id,
                });
            }
            _ => {}
        }
    }

    match key.code {
        KeyCode::Char('q') => EventOutcome::Quit,
        KeyCode::Char('?') => {
            app.toggle_help();
            EventOutcome::Continue
        }
        KeyCode::Esc => {
            if app.show_help() || app.show_command_overlay() {
                app.hide_modal();
                EventOutcome::Continue
            } else if !app.filter_query.is_empty() {
                app.clear_filter();
                EventOutcome::Refresh
            } else {
                app.focus = FocusArea::List;
                EventOutcome::Continue
            }
        }
        KeyCode::Char('/') => {
            app.start_filter_input();
            EventOutcome::Continue
        }
        KeyCode::Char('c') => {
            app.toggle_command_overlay();
            EventOutcome::Continue
        }
        KeyCode::Char(':') => {
            app.open_command_palette();
            EventOutcome::Continue
        }
        KeyCode::Char('a') => {
            let _ = app.open_action_menu();
            EventOutcome::Continue
        }
        KeyCode::Tab => {
            app.hide_modal();
            app.cycle_view(1);
            EventOutcome::Refresh
        }
        KeyCode::BackTab => {
            app.hide_modal();
            app.cycle_view(-1);
            EventOutcome::Refresh
        }
        KeyCode::Enter => {
            if app.view == View::Engine && app.focus == FocusArea::List {
                if let Some((group_id, pipeline_id)) = app.selected_engine_pipeline_target() {
                    app.view = View::Pipelines;
                    app.pipeline_selected = Some(format!("{group_id}:{pipeline_id}"));
                    app.focus = FocusArea::Detail;
                    app.reset_scroll();
                    return EventOutcome::Refresh;
                }
            }
            app.focus = FocusArea::Detail;
            app.reset_scroll();
            EventOutcome::Continue
        }
        KeyCode::Backspace => {
            app.focus = FocusArea::List;
            app.reset_scroll();
            EventOutcome::Continue
        }
        KeyCode::Left | KeyCode::Char('h') => {
            app.cycle_tab(-1);
            EventOutcome::Refresh
        }
        KeyCode::Right => {
            app.cycle_tab(1);
            EventOutcome::Refresh
        }
        KeyCode::Char('l') if app.focus == FocusArea::Detail => {
            app.cycle_tab(1);
            EventOutcome::Refresh
        }
        KeyCode::Up | KeyCode::Char('k') => handle_vertical_motion(app, -1),
        KeyCode::Down | KeyCode::Char('j') => handle_vertical_motion(app, 1),
        KeyCode::PageUp => handle_vertical_motion(app, -10),
        KeyCode::PageDown => handle_vertical_motion(app, 10),
        KeyCode::Char('g') => {
            app.move_selection_to_edge(false);
            app.reset_scroll();
            EventOutcome::Refresh
        }
        KeyCode::Char('G') => {
            app.move_selection_to_edge(true);
            app.reset_scroll();
            EventOutcome::Refresh
        }
        KeyCode::Char('r') => EventOutcome::Refresh,
        KeyCode::Char('e') => {
            select_detail_tab(app, DetailJump::Events);
            EventOutcome::Refresh
        }
        KeyCode::Char('l') => {
            select_detail_tab(app, DetailJump::Logs);
            EventOutcome::Refresh
        }
        KeyCode::Char('m') => {
            select_detail_tab(app, DetailJump::Metrics);
            EventOutcome::Refresh
        }
        KeyCode::Char('d') => {
            select_detail_tab(app, DetailJump::Diagnose);
            EventOutcome::Refresh
        }
        KeyCode::Char('b') => {
            select_detail_tab(app, DetailJump::Bundle);
            EventOutcome::Refresh
        }
        KeyCode::Char('o') => {
            if app.view == View::Pipelines {
                app.pipeline_tab = PipelineTab::Rollout;
                app.reset_scroll();
                EventOutcome::Refresh
            } else {
                EventOutcome::Continue
            }
        }
        _ => EventOutcome::Continue,
    }
}

fn handle_command_palette_key(key: KeyEvent, app: &mut AppState) -> EventOutcome {
    match key.code {
        KeyCode::Esc => {
            app.hide_modal();
            EventOutcome::Continue
        }
        KeyCode::Backspace => {
            app.pop_palette_input();
            EventOutcome::Continue
        }
        KeyCode::Up | KeyCode::Char('k') if key.modifiers.is_empty() => {
            app.move_palette_selection(-1);
            EventOutcome::Continue
        }
        KeyCode::Down | KeyCode::Char('j') if key.modifiers.is_empty() => {
            app.move_palette_selection(1);
            EventOutcome::Continue
        }
        KeyCode::Enter => app
            .selected_palette_action()
            .map_or(EventOutcome::Continue, |action| {
                execute_palette_action(action, app)
            }),
        KeyCode::Char(character)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            app.push_palette_input(character);
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}

fn execute_palette_action(action: PaletteAction, app: &mut AppState) -> EventOutcome {
    match action {
        PaletteAction::SwitchView(view) => {
            app.hide_modal();
            app.select_view(view);
            EventOutcome::Refresh
        }
        PaletteAction::SelectPipelineTab(tab) => {
            app.hide_modal();
            app.select_view(View::Pipelines);
            app.pipeline_tab = tab;
            app.focus = FocusArea::Detail;
            app.reset_scroll();
            EventOutcome::Refresh
        }
        PaletteAction::SelectGroupTab(tab) => {
            app.hide_modal();
            app.select_view(View::Groups);
            app.group_tab = tab;
            app.focus = FocusArea::Detail;
            app.reset_scroll();
            EventOutcome::Refresh
        }
        PaletteAction::SelectEngineTab(tab) => {
            app.hide_modal();
            app.select_view(View::Engine);
            app.engine_tab = tab;
            app.focus = FocusArea::Detail;
            app.reset_scroll();
            EventOutcome::Refresh
        }
        PaletteAction::Focus(focus) => {
            app.hide_modal();
            app.focus = focus;
            app.reset_scroll();
            EventOutcome::Continue
        }
        PaletteAction::OpenHelp => {
            app.modal = UiModal::Help;
            EventOutcome::Continue
        }
        PaletteAction::OpenFilter => {
            app.start_filter_input();
            EventOutcome::Continue
        }
        PaletteAction::OpenCommandOverlay => {
            app.modal = UiModal::Command;
            EventOutcome::Continue
        }
        PaletteAction::OpenActionMenu => {
            app.hide_modal();
            if !app.open_action_menu() {
                app.last_error =
                    Some("no actions are available for the current selection".to_string());
            }
            EventOutcome::Continue
        }
        PaletteAction::ClearFilter => {
            app.clear_filter();
            EventOutcome::Refresh
        }
        PaletteAction::Refresh => {
            app.hide_modal();
            EventOutcome::Refresh
        }
        PaletteAction::Quit => EventOutcome::Quit,
        PaletteAction::Execute(action) => execute_selected_ui_action(app, action),
    }
}

fn handle_action_menu_key(key: KeyEvent, app: &mut AppState) -> EventOutcome {
    match key.code {
        KeyCode::Esc | KeyCode::Char('a') => {
            app.hide_modal();
            EventOutcome::Continue
        }
        KeyCode::Char('q') => EventOutcome::Quit,
        KeyCode::Char('?') => {
            app.toggle_help();
            EventOutcome::Continue
        }
        KeyCode::Char('/') => {
            app.start_filter_input();
            EventOutcome::Continue
        }
        KeyCode::Char('c') => {
            app.toggle_command_overlay();
            EventOutcome::Continue
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if let Some(menu) = app.action_menu_mut() {
                menu.selected = menu.selected.saturating_sub(1);
            }
            EventOutcome::Continue
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if let Some(menu) = app.action_menu_mut() {
                menu.selected = (menu.selected + 1).min(menu.entries.len().saturating_sub(1));
            }
            EventOutcome::Continue
        }
        KeyCode::Enter => {
            let selected = app
                .action_menu()
                .and_then(|menu| menu.entries.get(menu.selected))
                .filter(|entry| entry.enabled)
                .map(|entry| entry.action.clone());

            selected.map_or(EventOutcome::Continue, |action| {
                execute_selected_ui_action(app, action)
            })
        }
        _ => EventOutcome::Continue,
    }
}

fn execute_selected_ui_action(app: &mut AppState, action: UiAction) -> EventOutcome {
    match action {
        UiAction::PipelineEditAndRedeploy {
            group_id,
            pipeline_id,
        } => {
            app.hide_modal();
            EventOutcome::OpenPipelineEditor {
                group_id,
                pipeline_id,
            }
        }
        UiAction::PipelineSetCoreCount {
            group_id,
            pipeline_id,
            current_cores,
        } => {
            app.open_scale_editor(group_id, pipeline_id, current_cores);
            EventOutcome::Continue
        }
        action @ UiAction::PipelineShutdown { .. } | action @ UiAction::GroupShutdown { .. } => {
            app.open_shutdown_confirm(action);
            EventOutcome::Continue
        }
        action => {
            app.hide_modal();
            EventOutcome::Execute(action)
        }
    }
}

fn handle_shutdown_confirm_key(key: KeyEvent, app: &mut AppState) -> EventOutcome {
    match key.code {
        KeyCode::Esc | KeyCode::Char('n') => {
            app.hide_modal();
            EventOutcome::Continue
        }
        KeyCode::Char('q') => EventOutcome::Quit,
        KeyCode::Enter | KeyCode::Char('y') => {
            let action = app.shutdown_confirm().map(|confirm| confirm.action.clone());
            if let Some(action) = action {
                app.hide_modal();
                EventOutcome::Execute(action)
            } else {
                EventOutcome::Continue
            }
        }
        _ => EventOutcome::Continue,
    }
}

fn handle_scale_editor_key(key: KeyEvent, app: &mut AppState) -> EventOutcome {
    match key.code {
        KeyCode::Esc => {
            app.hide_modal();
            EventOutcome::Continue
        }
        KeyCode::Backspace => {
            if let Some(editor) = app.scale_editor_mut() {
                let _ = editor.input.pop();
            }
            EventOutcome::Continue
        }
        KeyCode::Enter => {
            let Some(editor) = app.scale_editor() else {
                return EventOutcome::Continue;
            };
            match editor.input.parse::<usize>() {
                Ok(target_cores) if target_cores > 0 => {
                    let action = UiAction::PipelineScale {
                        group_id: editor.group_id.clone(),
                        pipeline_id: editor.pipeline_id.clone(),
                        target_cores,
                    };
                    app.hide_modal();
                    EventOutcome::Execute(action)
                }
                _ => {
                    app.last_error = Some("core count must be a positive integer".to_string());
                    EventOutcome::Continue
                }
            }
        }
        KeyCode::Char(character)
            if character.is_ascii_digit()
                && !key
                    .modifiers
                    .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            if let Some(editor) = app.scale_editor_mut() {
                if editor.input == "0" {
                    editor.input.clear();
                }
                editor.input.push(character);
            }
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}

fn handle_filter_key(key: KeyEvent, app: &mut AppState) -> EventOutcome {
    match key.code {
        KeyCode::Esc => {
            app.cancel_filter_input();
            EventOutcome::Continue
        }
        KeyCode::Enter => {
            app.apply_filter_input();
            app.reset_scroll();
            EventOutcome::Refresh
        }
        KeyCode::Backspace => {
            let _ = app.filter_input.pop();
            EventOutcome::Continue
        }
        KeyCode::Char(character)
            if !key
                .modifiers
                .intersects(KeyModifiers::CONTROL | KeyModifiers::ALT) =>
        {
            app.filter_input.push(character);
            EventOutcome::Continue
        }
        _ => EventOutcome::Continue,
    }
}

fn handle_vertical_motion(app: &mut AppState, delta: isize) -> EventOutcome {
    match app.focus {
        FocusArea::List => {
            app.move_selection(delta);
            app.reset_scroll();
            EventOutcome::Refresh
        }
        FocusArea::Detail => {
            if delta.is_negative() {
                app.detail_scroll = app
                    .detail_scroll
                    .saturating_sub(delta.unsigned_abs() as u16);
            } else {
                app.detail_scroll = app.detail_scroll.saturating_add(delta as u16);
            }
            EventOutcome::Continue
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
enum DetailJump {
    Events,
    Logs,
    Metrics,
    Diagnose,
    Bundle,
}

fn select_detail_tab(app: &mut AppState, jump: DetailJump) {
    app.reset_scroll();
    match (app.view, jump) {
        (View::Pipelines, DetailJump::Events) => app.pipeline_tab = PipelineTab::Events,
        (View::Pipelines, DetailJump::Logs) => app.pipeline_tab = PipelineTab::Logs,
        (View::Pipelines, DetailJump::Metrics) => app.pipeline_tab = PipelineTab::Metrics,
        (View::Pipelines, DetailJump::Diagnose) => app.pipeline_tab = PipelineTab::Diagnose,
        (View::Pipelines, DetailJump::Bundle) => app.pipeline_tab = PipelineTab::Bundle,
        (View::Groups, DetailJump::Events) => app.group_tab = GroupTab::Events,
        (View::Groups, DetailJump::Logs) => app.group_tab = GroupTab::Logs,
        (View::Groups, DetailJump::Metrics) => app.group_tab = GroupTab::Metrics,
        (View::Groups, DetailJump::Diagnose) => app.group_tab = GroupTab::Diagnose,
        (View::Groups, DetailJump::Bundle) => app.group_tab = GroupTab::Bundle,
        (View::Engine, DetailJump::Logs) => app.engine_tab = EngineTab::Logs,
        (View::Engine, DetailJump::Metrics) => app.engine_tab = EngineTab::Metrics,
        _ => {}
    }
}
