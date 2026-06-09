// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! AppState construction and navigation behavior for selection, focus, and tabs.
//!
//! State methods in this module keep the core interaction model predictable:
//! selected resources are preserved when possible, list movement wraps
//! consistently, tab changes reset scroll where appropriate, and focus labels
//! remain derived from state instead of duplicated by renderers.

use super::*;

impl AppState {
    /// Creates a fresh TUI state tree with the requested initial view.
    pub(crate) fn new(start_view: UiStartView, color_enabled: bool, logs_tail: usize) -> Self {
        Self {
            view: start_view.into(),
            focus: FocusArea::List,
            pipeline_tab: PipelineTab::Summary,
            group_tab: GroupTab::Summary,
            engine_tab: EngineTab::Summary,
            color_enabled,
            filter_query: String::new(),
            filter_input: String::new(),
            modal: UiModal::None,
            detail_scroll: 0,
            pipeline_selected: None,
            group_selected: None,
            engine_selected: None,
            terminal_size: None,
            last_refresh: None,
            last_error: None,
            groups_status: None,
            engine_status: None,
            engine_livez: None,
            engine_readyz: None,
            engine_vitals: EngineVitals::default(),
            command_context: UiCommandContext::local_default(logs_tail),
            activity_indicator: ActivityIndicator::default(),
            pipelines: PipelinePaneState::default(),
            groups: GroupPaneState::default(),
            engine: EnginePaneState::new(),
        }
    }

    /// Replaces the CLI command context used by equivalent-command hints.
    pub(crate) fn set_command_context(&mut self, command_context: UiCommandContext) {
        self.command_context = command_context;
    }

    /// Records the most recent terminal size for layout and mouse hit testing.
    pub(crate) fn set_terminal_size(&mut self, width: u16, height: u16) {
        self.terminal_size = Some((width, height));
    }

    /// Builds the filtered selectable pipeline list from group status.
    pub(crate) fn pipeline_items(&self) -> Vec<PipelineItem> {
        let mut items = Vec::new();
        let Some(status) = &self.groups_status else {
            return items;
        };

        for (key, pipeline) in &status.pipelines {
            let (pipeline_group_id, pipeline_id) =
                split_pipeline_key(key).unwrap_or((key.as_str(), ""));
            let haystack = format!("{pipeline_group_id} {pipeline_id}");
            if !self.filter_query.is_empty()
                && !haystack
                    .to_ascii_lowercase()
                    .contains(&self.filter_query.to_ascii_lowercase())
            {
                continue;
            }

            let (status_badge, tone) = classify_pipeline_row(pipeline);
            items.push(PipelineItem {
                key: key.clone(),
                status_badge,
                tone,
                pipeline_group_id: pipeline_group_id.to_string(),
                pipeline_id: pipeline_id.to_string(),
                running: format!("{}/{}", pipeline.running_cores, pipeline.total_cores),
                active_generation: pipeline
                    .active_generation
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "none".to_string()),
                rollout: pipeline
                    .rollout
                    .as_ref()
                    .map(|value| format!("{:?}", value.state).to_ascii_lowercase())
                    .unwrap_or_else(|| "none".to_string()),
            });
        }

        items
    }

    /// Builds the filtered selectable group list by aggregating pipeline status.
    pub(crate) fn group_items(&self) -> Vec<GroupItem> {
        let Some(status) = &self.groups_status else {
            return Vec::new();
        };

        let mut grouped = BTreeMap::<String, GroupItem>::new();
        for (key, pipeline) in &status.pipelines {
            let (group_id, _) = split_pipeline_key(key).unwrap_or((key.as_str(), ""));
            let entry = grouped
                .entry(group_id.to_string())
                .or_insert_with(|| GroupItem {
                    group_id: group_id.to_string(),
                    status_badge: "ok".to_string(),
                    tone: Tone::Success,
                    pipelines: 0,
                    running: 0,
                    ready: 0,
                    terminal: 0,
                });
            entry.pipelines += 1;
            if pipeline.running_cores > 0 {
                entry.running += 1;
            }
            if pipeline_is_ready(pipeline) {
                entry.ready += 1;
            }
            if pipeline_is_terminal(pipeline) {
                entry.terminal += 1;
            }
            let (_, pipeline_tone) = classify_pipeline_row(pipeline);
            entry.tone = combine_tones(entry.tone, pipeline_tone);
        }

        grouped
            .into_values()
            .filter(|item| {
                self.filter_query.is_empty()
                    || item
                        .group_id
                        .to_ascii_lowercase()
                        .contains(&self.filter_query.to_ascii_lowercase())
            })
            .map(|mut item| {
                item.tone = if item.tone == Tone::Failure {
                    Tone::Failure
                } else if item.tone == Tone::Accent {
                    Tone::Accent
                } else if item.ready < item.pipelines
                    || item.running < item.pipelines
                    || item.terminal > 0
                {
                    Tone::Warning
                } else {
                    Tone::Success
                };
                item.status_badge = tone_badge(item.tone).to_string();
                item
            })
            .collect()
    }

    /// Builds the filtered engine-level pipeline list from engine status.
    pub(crate) fn engine_pipeline_items(&self) -> Vec<EnginePipelineItem> {
        let Some(status) = &self.engine_status else {
            return Vec::new();
        };

        status
            .pipelines
            .iter()
            .filter(|(key, _)| {
                self.filter_query.is_empty()
                    || key
                        .to_ascii_lowercase()
                        .contains(&self.filter_query.to_ascii_lowercase())
            })
            .map(|(key, pipeline)| {
                let (status_badge, tone) = classify_pipeline_row(pipeline);
                EnginePipelineItem {
                    key: key.clone(),
                    status_badge,
                    tone,
                    running: format!("{}/{}", pipeline.running_cores, pipeline.total_cores),
                    active_generation: pipeline
                        .active_generation
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "none".to_string()),
                    rollout: pipeline
                        .rollout
                        .as_ref()
                        .map(|value| format!("{:?}", value.state).to_ascii_lowercase())
                        .unwrap_or_else(|| "none".to_string()),
                }
            })
            .collect()
    }

    /// Ensures current list selections still point to visible rows.
    pub(crate) fn ensure_selection(&mut self) {
        self.pipeline_selected = ensure_selected_key(
            self.pipeline_selected.take(),
            &self.pipeline_items(),
            |item| item.key.clone(),
        );
        self.group_selected =
            ensure_selected_key(self.group_selected.take(), &self.group_items(), |item| {
                item.group_id.clone()
            });
        self.engine_selected = ensure_selected_key(
            self.engine_selected.take(),
            &self.engine_pipeline_items(),
            |item| item.key.clone(),
        );
    }

    /// Moves the selected row in the active top-level list.
    pub(crate) fn move_selection(&mut self, delta: isize) {
        match self.view {
            View::Pipelines => {
                let keys = self
                    .pipeline_items()
                    .into_iter()
                    .map(|item| item.key)
                    .collect::<Vec<_>>();
                move_key_selection(&mut self.pipeline_selected, &keys, delta);
            }
            View::Groups => {
                let keys = self
                    .group_items()
                    .into_iter()
                    .map(|item| item.group_id)
                    .collect::<Vec<_>>();
                move_key_selection(&mut self.group_selected, &keys, delta);
            }
            View::Engine => {
                let keys = self
                    .engine_pipeline_items()
                    .into_iter()
                    .map(|item| item.key)
                    .collect::<Vec<_>>();
                move_key_selection(&mut self.engine_selected, &keys, delta);
            }
        }
    }

    /// Moves the selected row to the first or last row of the active list.
    pub(crate) fn move_selection_to_edge(&mut self, end: bool) {
        match self.view {
            View::Pipelines => {
                let keys = self
                    .pipeline_items()
                    .into_iter()
                    .map(|item| item.key)
                    .collect::<Vec<_>>();
                select_key_edge(&mut self.pipeline_selected, &keys, end);
            }
            View::Groups => {
                let keys = self
                    .group_items()
                    .into_iter()
                    .map(|item| item.group_id)
                    .collect::<Vec<_>>();
                select_key_edge(&mut self.group_selected, &keys, end);
            }
            View::Engine => {
                let keys = self
                    .engine_pipeline_items()
                    .into_iter()
                    .map(|item| item.key)
                    .collect::<Vec<_>>();
                select_key_edge(&mut self.engine_selected, &keys, end);
            }
        }
    }

    /// Selects a row by visible list index and returns whether selection changed.
    pub(crate) fn select_list_index(&mut self, index: usize) -> bool {
        match self.view {
            View::Pipelines => self
                .pipeline_items()
                .get(index)
                .map(|item| update_selection(&mut self.pipeline_selected, &item.key))
                .unwrap_or(false),
            View::Groups => self
                .group_items()
                .get(index)
                .map(|item| update_selection(&mut self.group_selected, &item.group_id))
                .unwrap_or(false),
            View::Engine => self
                .engine_pipeline_items()
                .get(index)
                .map(|item| update_selection(&mut self.engine_selected, &item.key))
                .unwrap_or(false),
        }
    }

    /// Returns the selected pipeline target from the pipeline list.
    pub(crate) fn selected_pipeline_target(&self) -> Option<(String, String)> {
        let key = self.pipeline_selected.as_deref()?;
        let (group_id, pipeline_id) = split_pipeline_key(key)?;
        Some((group_id.to_string(), pipeline_id.to_string()))
    }

    /// Returns the selected group id from the group list.
    pub(crate) fn selected_group_id(&self) -> Option<String> {
        self.group_selected.clone()
    }

    /// Returns the selected pipeline target from the engine pipeline list.
    pub(crate) fn selected_engine_pipeline_target(&self) -> Option<(String, String)> {
        let key = self.engine_selected.as_deref()?;
        let (group_id, pipeline_id) = split_pipeline_key(key)?;
        Some((group_id.to_string(), pipeline_id.to_string()))
    }

    /// Returns tab titles for the active top-level view.
    pub(crate) fn current_tab_titles(&self) -> Vec<&'static str> {
        match self.view {
            View::Pipelines => PipelineTab::ALL.iter().map(|tab| tab.title()).collect(),
            View::Groups => GroupTab::ALL.iter().map(|tab| tab.title()).collect(),
            View::Engine => EngineTab::ALL.iter().map(|tab| tab.title()).collect(),
        }
    }

    /// Returns the selected tab index for the active top-level view.
    pub(crate) fn current_tab_index(&self) -> usize {
        match self.view {
            View::Pipelines => PipelineTab::ALL
                .iter()
                .position(|tab| tab == &self.pipeline_tab)
                .unwrap_or(0),
            View::Groups => GroupTab::ALL
                .iter()
                .position(|tab| tab == &self.group_tab)
                .unwrap_or(0),
            View::Engine => EngineTab::ALL
                .iter()
                .position(|tab| tab == &self.engine_tab)
                .unwrap_or(0),
        }
    }

    /// Cycles between top-level views and resets focus to the list.
    pub(crate) fn cycle_view(&mut self, delta: isize) {
        let current_index = View::ALL
            .iter()
            .position(|view| view == &self.view)
            .unwrap_or(0);
        let next = wrap_index(current_index, View::ALL.len(), delta);
        self.select_view(View::ALL[next]);
    }

    /// Selects a top-level view and resets detail navigation state.
    pub(crate) fn select_view(&mut self, view: View) {
        self.view = view;
        self.focus = FocusArea::List;
        self.detail_scroll = 0;
    }

    /// Cycles tabs within the active top-level view.
    pub(crate) fn cycle_tab(&mut self, delta: isize) {
        self.detail_scroll = 0;
        match self.view {
            View::Pipelines => {
                let index = PipelineTab::ALL
                    .iter()
                    .position(|tab| tab == &self.pipeline_tab)
                    .unwrap_or(0);
                self.pipeline_tab =
                    PipelineTab::ALL[wrap_index(index, PipelineTab::ALL.len(), delta)];
            }
            View::Groups => {
                let index = GroupTab::ALL
                    .iter()
                    .position(|tab| tab == &self.group_tab)
                    .unwrap_or(0);
                self.group_tab = GroupTab::ALL[wrap_index(index, GroupTab::ALL.len(), delta)];
            }
            View::Engine => {
                let index = EngineTab::ALL
                    .iter()
                    .position(|tab| tab == &self.engine_tab)
                    .unwrap_or(0);
                self.engine_tab = EngineTab::ALL[wrap_index(index, EngineTab::ALL.len(), delta)];
            }
        }
    }

    /// Selects a visible tab index and focuses the detail pane.
    pub(crate) fn select_current_tab(&mut self, index: usize) {
        self.detail_scroll = 0;
        self.focus = FocusArea::Detail;
        match self.view {
            View::Pipelines => {
                if let Some(tab) = PipelineTab::ALL.get(index).copied() {
                    self.pipeline_tab = tab;
                }
            }
            View::Groups => {
                if let Some(tab) = GroupTab::ALL.get(index).copied() {
                    self.group_tab = tab;
                }
            }
            View::Engine => {
                if let Some(tab) = EngineTab::ALL.get(index).copied() {
                    self.engine_tab = tab;
                }
            }
        }
    }

    /// Returns the title for the active left-side list.
    pub(crate) fn current_list_title(&self) -> &'static str {
        match self.view {
            View::Pipelines => "Pipelines",
            View::Groups => "Groups",
            View::Engine => "Engine Pipelines",
        }
    }

    /// Returns a short label for the current selection.
    pub(crate) fn current_selection_label(&self) -> String {
        match self.view {
            View::Pipelines => self
                .pipeline_selected
                .clone()
                .unwrap_or_else(|| "none".to_string()),
            View::Groups => self
                .group_selected
                .clone()
                .unwrap_or_else(|| "none".to_string()),
            View::Engine => self
                .engine_selected
                .clone()
                .unwrap_or_else(|| "engine".to_string()),
        }
    }

    /// Returns a short label for the current focus area.
    pub(crate) fn current_focus_label(&self) -> &'static str {
        match self.focus {
            FocusArea::List => "list",
            FocusArea::Detail => "detail",
        }
    }

    /// Returns the target URL displayed in the TUI header.
    pub(crate) fn target_url(&self) -> &str {
        &self.command_context.target_url
    }

    /// Marks the TUI as actively refreshing or executing an operation.
    pub(crate) fn begin_activity(&mut self) {
        self.activity_indicator.active = true;
    }

    /// Clears the active indicator and resets its animation frame.
    pub(crate) fn end_activity(&mut self) {
        self.activity_indicator.active = false;
        self.activity_indicator.frame = 0;
    }

    /// Returns true while the activity indicator should animate.
    pub(crate) fn is_activity_active(&self) -> bool {
        self.activity_indicator.active
    }

    /// Returns the current activity animation frame.
    pub(crate) fn activity_frame(&self) -> u16 {
        self.activity_indicator.frame
    }

    /// Advances the activity animation frame when activity is active.
    pub(crate) fn advance_activity_frame(&mut self) {
        if self.activity_indicator.active {
            self.activity_indicator.frame = self.activity_indicator.frame.wrapping_add(1);
        }
    }

    /// Returns the header for the currently active detail tab.
    pub(crate) fn active_header(&self) -> Option<&DetailHeader> {
        match self.view {
            View::Pipelines => match self.pipeline_tab {
                PipelineTab::Summary => self.pipelines.summary.header.as_ref(),
                PipelineTab::Details => self.pipelines.details.header.as_ref(),
                PipelineTab::Config => self.pipelines.config.header.as_ref(),
                PipelineTab::Events => self.pipelines.events.header.as_ref(),
                PipelineTab::Logs => self.pipelines.logs.header.as_ref(),
                PipelineTab::Metrics => self.pipelines.metrics.header.as_ref(),
                PipelineTab::Rollout => self.pipelines.rollout.header.as_ref(),
                PipelineTab::Shutdown => self.pipelines.shutdown.header.as_ref(),
                PipelineTab::Diagnose => self.pipelines.diagnosis.header.as_ref(),
                PipelineTab::Bundle => self.pipelines.bundle.header.as_ref(),
            },
            View::Groups => match self.group_tab {
                GroupTab::Summary => self.groups.summary.header.as_ref(),
                GroupTab::Details => self.groups.details.header.as_ref(),
                GroupTab::Events => self.groups.events.header.as_ref(),
                GroupTab::Logs => self.groups.logs.header.as_ref(),
                GroupTab::Metrics => self.groups.metrics.header.as_ref(),
                GroupTab::Shutdown => self.groups.shutdown.header.as_ref(),
                GroupTab::Diagnose => self.groups.diagnosis.header.as_ref(),
                GroupTab::Bundle => self.groups.bundle.header.as_ref(),
            },
            View::Engine => match self.engine_tab {
                EngineTab::Summary => self.engine.summary.header.as_ref(),
                EngineTab::Details => self.engine.details.header.as_ref(),
                EngineTab::Logs => self.engine.logs.header.as_ref(),
                EngineTab::Metrics => self.engine.metrics.header.as_ref(),
            },
        }
    }

    /// Returns the title for the currently active detail tab.
    pub(crate) fn current_detail_title(&self) -> String {
        match self.view {
            View::Pipelines => self.pipeline_tab.title().to_string(),
            View::Groups => self.group_tab.title().to_string(),
            View::Engine => self.engine_tab.title().to_string(),
        }
    }
}
