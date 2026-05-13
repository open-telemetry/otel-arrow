// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Searchable command palette assembly and state transitions.
//!
//! The palette module exposes the keyboard-driven command discovery surface
//! inside the TUI. It builds context-aware entries, filters them from user
//! input, tracks selection, and maps the selected entry back to a high-level
//! `PaletteAction` without embedding that behavior in the generic key handler.

use super::*;

impl AppState {
    /// Opens the command palette modal with empty input.
    pub(crate) fn open_command_palette(&mut self) {
        self.modal = UiModal::CommandPalette(CommandPaletteState::default());
    }

    /// Returns true when the command palette modal is visible.
    pub(crate) fn show_command_palette(&self) -> bool {
        matches!(self.modal, UiModal::CommandPalette(_))
    }

    /// Returns read-only command palette state when the modal is visible.
    pub(crate) fn command_palette(&self) -> Option<&CommandPaletteState> {
        match &self.modal {
            UiModal::CommandPalette(palette) => Some(palette),
            _ => None,
        }
    }

    /// Returns mutable command palette state when the modal is visible.
    pub(crate) fn command_palette_mut(&mut self) -> Option<&mut CommandPaletteState> {
        match &mut self.modal {
            UiModal::CommandPalette(palette) => Some(palette),
            _ => None,
        }
    }

    /// Appends one character to the command palette filter input.
    pub(crate) fn push_palette_input(&mut self, character: char) {
        if let Some(palette) = self.command_palette_mut() {
            palette.input.push(character);
        }
        self.clamp_palette_selection();
    }

    /// Removes the last character from the command palette filter input.
    pub(crate) fn pop_palette_input(&mut self) {
        if let Some(palette) = self.command_palette_mut() {
            let _ = palette.input.pop();
        }
        self.clamp_palette_selection();
    }

    /// Moves the selected command palette row with wraparound.
    pub(crate) fn move_palette_selection(&mut self, delta: isize) {
        let len = self.filtered_palette_entries().len();
        if len == 0 {
            if let Some(palette) = self.command_palette_mut() {
                palette.selected = 0;
            }
            return;
        }

        let selected = self
            .command_palette()
            .map(|palette| palette.selected)
            .unwrap_or_default();
        let next = wrap_index(selected.min(len - 1), len, delta);
        if let Some(palette) = self.command_palette_mut() {
            palette.selected = next;
        }
    }

    /// Returns the enabled action selected in the filtered command palette.
    pub(crate) fn selected_palette_action(&self) -> Option<PaletteAction> {
        let entries = self.filtered_palette_entries();
        let selected = self
            .command_palette()
            .map(|palette| palette.selected)
            .unwrap_or_default();
        entries
            .get(selected.min(entries.len().saturating_sub(1)))
            .filter(|entry| entry.enabled)
            .map(|entry| entry.action.clone())
    }

    /// Builds command palette entries filtered by the current palette input.
    pub(crate) fn filtered_palette_entries(&self) -> Vec<CommandPaletteEntry> {
        let query = self
            .command_palette()
            .map(|palette| palette.input.trim().to_ascii_lowercase())
            .unwrap_or_default();
        let entries = self.build_palette_entries();
        if query.is_empty() {
            return entries;
        }

        let terms = query
            .split_whitespace()
            .map(str::to_string)
            .collect::<Vec<_>>();
        entries
            .into_iter()
            .filter(|entry| {
                let haystack = format!("{} {} {}", entry.label, entry.detail, entry.keywords)
                    .to_ascii_lowercase();
                terms.iter().all(|term| haystack.contains(term))
            })
            .collect()
    }

    fn clamp_palette_selection(&mut self) {
        let len = self.filtered_palette_entries().len();
        if let Some(palette) = self.command_palette_mut() {
            palette.selected = if len == 0 {
                0
            } else {
                palette.selected.min(len - 1)
            };
        }
    }

    fn build_palette_entries(&self) -> Vec<CommandPaletteEntry> {
        let mut entries = vec![
            palette_entry(
                "Refresh now",
                "Fetch the latest state from the admin API.",
                "reload poll update",
                PaletteAction::Refresh,
                true,
            ),
            palette_entry(
                "Show equivalent CLI",
                "Open the command recipe for the current pane.",
                "command shell copy learn",
                PaletteAction::OpenCommandOverlay,
                true,
            ),
            palette_entry(
                "Filter list",
                "Filter the current resource list by text.",
                "search narrow slash",
                PaletteAction::OpenFilter,
                true,
            ),
            palette_entry(
                "Show help",
                "Open the keybinding and navigation help overlay.",
                "question keys shortcuts",
                PaletteAction::OpenHelp,
                true,
            ),
            palette_entry(
                "Focus list",
                "Move keyboard focus to the resource list.",
                "left resources rows",
                PaletteAction::Focus(FocusArea::List),
                true,
            ),
            palette_entry(
                "Focus details",
                "Move keyboard focus to the selected object's detail pane.",
                "right object pane",
                PaletteAction::Focus(FocusArea::Detail),
                true,
            ),
            palette_entry(
                "Quit",
                "Leave the interactive UI.",
                "exit close",
                PaletteAction::Quit,
                true,
            ),
        ];

        if !self.filter_query.is_empty() {
            entries.push(palette_entry(
                "Clear filter",
                "Remove the active resource-list filter.",
                "reset search",
                PaletteAction::ClearFilter,
                true,
            ));
        }

        for view in View::ALL {
            entries.push(palette_entry(
                format!("View: {}", view.title()),
                format!("Switch to the {} view.", view.title()),
                "top level tab",
                PaletteAction::SwitchView(view),
                true,
            ));
        }

        for tab in PipelineTab::ALL {
            entries.push(palette_entry(
                format!("Pipeline tab: {}", tab.title()),
                format!("Switch to Pipelines / {}.", tab.title()),
                "pipeline detail object",
                PaletteAction::SelectPipelineTab(tab),
                true,
            ));
        }
        for tab in GroupTab::ALL {
            entries.push(palette_entry(
                format!("Group tab: {}", tab.title()),
                format!("Switch to Groups / {}.", tab.title()),
                "group detail object",
                PaletteAction::SelectGroupTab(tab),
                true,
            ));
        }
        for tab in EngineTab::ALL {
            entries.push(palette_entry(
                format!("Engine tab: {}", tab.title()),
                format!("Switch to Engine / {}.", tab.title()),
                "engine detail object",
                PaletteAction::SelectEngineTab(tab),
                true,
            ));
        }

        if let Some(menu) = self.build_action_menu() {
            entries.push(palette_entry(
                "Open context actions",
                "Show all actions available for the current selection.",
                "mutate action menu",
                PaletteAction::OpenActionMenu,
                true,
            ));
            for entry in menu.entries {
                entries.push(palette_entry(
                    format!("Action: {}", entry.label),
                    entry.detail,
                    "context mutate selected",
                    PaletteAction::Execute(entry.action),
                    entry.enabled,
                ));
            }
        }

        entries
    }
}

fn palette_entry(
    label: impl Into<String>,
    detail: impl Into<String>,
    keywords: impl Into<String>,
    action: PaletteAction,
    enabled: bool,
) -> CommandPaletteEntry {
    CommandPaletteEntry {
        label: label.into(),
        detail: detail.into(),
        keywords: keywords.into(),
        action,
        enabled,
    }
}
