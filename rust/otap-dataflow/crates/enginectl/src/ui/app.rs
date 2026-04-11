// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::args::UiStartView;
use otap_df_admin_api::{engine, groups, telemetry};
use std::collections::BTreeMap;
use std::time::Instant;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum View {
    Pipelines,
    Groups,
    Engine,
}

impl View {
    pub(crate) const ALL: [Self; 3] = [Self::Pipelines, Self::Groups, Self::Engine];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Pipelines => "Pipelines",
            Self::Groups => "Groups",
            Self::Engine => "Engine",
        }
    }
}

impl From<UiStartView> for View {
    fn from(value: UiStartView) -> Self {
        match value {
            UiStartView::Pipelines => Self::Pipelines,
            UiStartView::Groups => Self::Groups,
            UiStartView::Engine => Self::Engine,
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum FocusArea {
    List,
    Detail,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum PipelineTab {
    Summary,
    Events,
    Logs,
    Metrics,
    Rollout,
    Diagnose,
    Bundle,
}

impl PipelineTab {
    pub(crate) const ALL: [Self; 7] = [
        Self::Summary,
        Self::Events,
        Self::Logs,
        Self::Metrics,
        Self::Rollout,
        Self::Diagnose,
        Self::Bundle,
    ];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Events => "Events",
            Self::Logs => "Logs",
            Self::Metrics => "Metrics",
            Self::Rollout => "Rollout",
            Self::Diagnose => "Diagnose",
            Self::Bundle => "Bundle",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum GroupTab {
    Summary,
    Events,
    Logs,
    Metrics,
    Diagnose,
    Bundle,
}

impl GroupTab {
    pub(crate) const ALL: [Self; 6] = [
        Self::Summary,
        Self::Events,
        Self::Logs,
        Self::Metrics,
        Self::Diagnose,
        Self::Bundle,
    ];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Events => "Events",
            Self::Logs => "Logs",
            Self::Metrics => "Metrics",
            Self::Diagnose => "Diagnose",
            Self::Bundle => "Bundle",
        }
    }
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum EngineTab {
    Summary,
    Logs,
    Metrics,
}

impl EngineTab {
    pub(crate) const ALL: [Self; 3] = [Self::Summary, Self::Logs, Self::Metrics];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Logs => "Logs",
            Self::Metrics => "Metrics",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PipelineItem {
    pub(crate) key: String,
    pub(crate) pipeline_group_id: String,
    pub(crate) pipeline_id: String,
    pub(crate) running: String,
    pub(crate) active_generation: String,
    pub(crate) rollout: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GroupItem {
    pub(crate) group_id: String,
    pub(crate) pipelines: usize,
    pub(crate) running: usize,
    pub(crate) ready: usize,
    pub(crate) terminal: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnginePipelineItem {
    pub(crate) key: String,
    pub(crate) running: String,
    pub(crate) active_generation: String,
    pub(crate) rollout: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LogFeedState {
    pub(crate) scope_key: Option<String>,
    pub(crate) next_seq: Option<u64>,
    pub(crate) response: Option<telemetry::LogsResponse>,
    pub(crate) rendered: String,
}

impl LogFeedState {
    pub(crate) fn reset(&mut self, scope_key: String) {
        self.scope_key = Some(scope_key);
        self.next_seq = None;
        self.response = None;
        self.rendered.clear();
    }

    pub(crate) fn clear(&mut self) {
        self.scope_key = None;
        self.next_seq = None;
        self.response = None;
        self.rendered.clear();
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PipelinePaneState {
    pub(crate) target_key: Option<String>,
    pub(crate) summary: String,
    pub(crate) events: String,
    pub(crate) logs: LogFeedState,
    pub(crate) metrics: String,
    pub(crate) rollout: String,
    pub(crate) diagnosis: String,
    pub(crate) bundle: String,
}

impl PipelinePaneState {
    pub(crate) fn reset(&mut self, target_key: String) {
        self.target_key = Some(target_key.clone());
        self.summary.clear();
        self.events.clear();
        self.logs.reset(format!("pipeline:{target_key}"));
        self.metrics.clear();
        self.rollout.clear();
        self.diagnosis.clear();
        self.bundle.clear();
    }

    pub(crate) fn clear(&mut self) {
        self.target_key = None;
        self.summary.clear();
        self.events.clear();
        self.logs.clear();
        self.metrics.clear();
        self.rollout.clear();
        self.diagnosis.clear();
        self.bundle.clear();
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GroupPaneState {
    pub(crate) group_id: Option<String>,
    pub(crate) summary: String,
    pub(crate) events: String,
    pub(crate) logs: LogFeedState,
    pub(crate) metrics: String,
    pub(crate) diagnosis: String,
    pub(crate) bundle: String,
}

impl GroupPaneState {
    pub(crate) fn reset(&mut self, group_id: String) {
        self.group_id = Some(group_id.clone());
        self.summary.clear();
        self.events.clear();
        self.logs.reset(format!("group:{group_id}"));
        self.metrics.clear();
        self.diagnosis.clear();
        self.bundle.clear();
    }

    pub(crate) fn clear(&mut self) {
        self.group_id = None;
        self.summary.clear();
        self.events.clear();
        self.logs.clear();
        self.metrics.clear();
        self.diagnosis.clear();
        self.bundle.clear();
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EnginePaneState {
    pub(crate) summary: String,
    pub(crate) logs: LogFeedState,
    pub(crate) metrics: String,
}

impl EnginePaneState {
    pub(crate) fn new() -> Self {
        let mut state = Self::default();
        state.logs.reset("engine".to_string());
        state
    }
}

#[derive(Clone, Debug)]
pub(crate) struct AppState {
    pub(crate) view: View,
    pub(crate) focus: FocusArea,
    pub(crate) pipeline_tab: PipelineTab,
    pub(crate) group_tab: GroupTab,
    pub(crate) engine_tab: EngineTab,
    pub(crate) color_enabled: bool,
    pub(crate) filter_query: String,
    pub(crate) filter_input: String,
    pub(crate) filter_mode: bool,
    pub(crate) show_help: bool,
    pub(crate) detail_scroll: u16,
    pub(crate) pipeline_selected: Option<String>,
    pub(crate) group_selected: Option<String>,
    pub(crate) engine_selected: Option<String>,
    pub(crate) last_refresh: Option<Instant>,
    pub(crate) last_error: Option<String>,
    pub(crate) groups_status: Option<groups::Status>,
    pub(crate) engine_status: Option<engine::Status>,
    pub(crate) engine_livez: Option<engine::ProbeResponse>,
    pub(crate) engine_readyz: Option<engine::ProbeResponse>,
    pub(crate) pipelines: PipelinePaneState,
    pub(crate) groups: GroupPaneState,
    pub(crate) engine: EnginePaneState,
}

impl AppState {
    pub(crate) fn new(start_view: UiStartView, color_enabled: bool, _logs_tail: usize) -> Self {
        Self {
            view: start_view.into(),
            focus: FocusArea::List,
            pipeline_tab: PipelineTab::Summary,
            group_tab: GroupTab::Summary,
            engine_tab: EngineTab::Summary,
            color_enabled,
            filter_query: String::new(),
            filter_input: String::new(),
            filter_mode: false,
            show_help: false,
            detail_scroll: 0,
            pipeline_selected: None,
            group_selected: None,
            engine_selected: None,
            last_refresh: None,
            last_error: None,
            groups_status: None,
            engine_status: None,
            engine_livez: None,
            engine_readyz: None,
            pipelines: PipelinePaneState::default(),
            groups: GroupPaneState::default(),
            engine: EnginePaneState::new(),
        }
    }

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

            items.push(PipelineItem {
                key: key.clone(),
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

    pub(crate) fn group_items(&self) -> Vec<GroupItem> {
        let Some(status) = &self.groups_status else {
            return Vec::new();
        };

        let mut grouped: BTreeMap<String, GroupItem> = BTreeMap::new();
        for (key, pipeline) in &status.pipelines {
            let (group_id, _) = split_pipeline_key(key).unwrap_or((key.as_str(), ""));
            let entry = grouped
                .entry(group_id.to_string())
                .or_insert_with(|| GroupItem {
                    group_id: group_id.to_string(),
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
            .collect()
    }

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
            .map(|(key, pipeline)| EnginePipelineItem {
                key: key.clone(),
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
            })
            .collect()
    }

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

    pub(crate) fn selected_pipeline_target(&self) -> Option<(String, String)> {
        let key = self.pipeline_selected.as_deref()?;
        let (group_id, pipeline_id) = split_pipeline_key(key)?;
        Some((group_id.to_string(), pipeline_id.to_string()))
    }

    pub(crate) fn selected_group_id(&self) -> Option<String> {
        self.group_selected.clone()
    }

    pub(crate) fn selected_engine_pipeline_target(&self) -> Option<(String, String)> {
        let key = self.engine_selected.as_deref()?;
        let (group_id, pipeline_id) = split_pipeline_key(key)?;
        Some((group_id.to_string(), pipeline_id.to_string()))
    }

    pub(crate) fn current_tab_titles(&self) -> Vec<&'static str> {
        match self.view {
            View::Pipelines => PipelineTab::ALL.iter().map(|tab| tab.title()).collect(),
            View::Groups => GroupTab::ALL.iter().map(|tab| tab.title()).collect(),
            View::Engine => EngineTab::ALL.iter().map(|tab| tab.title()).collect(),
        }
    }

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

    pub(crate) fn cycle_view(&mut self, delta: isize) {
        let current_index = View::ALL
            .iter()
            .position(|view| view == &self.view)
            .unwrap_or(0);
        let next = wrap_index(current_index, View::ALL.len(), delta);
        self.view = View::ALL[next];
        self.focus = FocusArea::List;
        self.detail_scroll = 0;
    }

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

    pub(crate) fn active_detail_text(&self) -> &str {
        match self.view {
            View::Pipelines => match self.pipeline_tab {
                PipelineTab::Summary => self.pipelines.summary.as_str(),
                PipelineTab::Events => self.pipelines.events.as_str(),
                PipelineTab::Logs => self.pipelines.logs.rendered.as_str(),
                PipelineTab::Metrics => self.pipelines.metrics.as_str(),
                PipelineTab::Rollout => self.pipelines.rollout.as_str(),
                PipelineTab::Diagnose => self.pipelines.diagnosis.as_str(),
                PipelineTab::Bundle => self.pipelines.bundle.as_str(),
            },
            View::Groups => match self.group_tab {
                GroupTab::Summary => self.groups.summary.as_str(),
                GroupTab::Events => self.groups.events.as_str(),
                GroupTab::Logs => self.groups.logs.rendered.as_str(),
                GroupTab::Metrics => self.groups.metrics.as_str(),
                GroupTab::Diagnose => self.groups.diagnosis.as_str(),
                GroupTab::Bundle => self.groups.bundle.as_str(),
            },
            View::Engine => match self.engine_tab {
                EngineTab::Summary => self.engine.summary.as_str(),
                EngineTab::Logs => self.engine.logs.rendered.as_str(),
                EngineTab::Metrics => self.engine.metrics.as_str(),
            },
        }
    }

    pub(crate) fn active_detail_title(&self) -> String {
        match self.view {
            View::Pipelines => {
                let label = self
                    .pipeline_selected
                    .as_deref()
                    .unwrap_or("no pipeline selected");
                format!("{}: {label}", self.pipeline_tab.title())
            }
            View::Groups => {
                let label = self
                    .group_selected
                    .as_deref()
                    .unwrap_or("no group selected");
                format!("{}: {label}", self.group_tab.title())
            }
            View::Engine => format!("{}: engine", self.engine_tab.title()),
        }
    }

    pub(crate) fn current_list_title(&self) -> &'static str {
        match self.view {
            View::Pipelines => "Pipelines",
            View::Groups => "Groups",
            View::Engine => "Engine Pipelines",
        }
    }

    pub(crate) fn start_filter_input(&mut self) {
        self.filter_mode = true;
        self.filter_input = self.filter_query.clone();
    }

    pub(crate) fn apply_filter_input(&mut self) {
        self.filter_query = self.filter_input.trim().to_string();
        self.filter_mode = false;
        self.ensure_selection();
    }

    pub(crate) fn cancel_filter_input(&mut self) {
        self.filter_mode = false;
        self.filter_input.clear();
    }

    pub(crate) fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.filter_mode = false;
        self.filter_input.clear();
        self.ensure_selection();
    }

    pub(crate) fn reset_scroll(&mut self) {
        self.detail_scroll = 0;
    }
}

fn ensure_selected_key<T, F>(selected: Option<String>, items: &[T], map: F) -> Option<String>
where
    F: Fn(&T) -> String,
{
    if items.is_empty() {
        return None;
    }
    if let Some(selected) = selected {
        if items.iter().any(|item| map(item) == selected) {
            return Some(selected);
        }
    }
    Some(map(&items[0]))
}

fn move_key_selection(selection: &mut Option<String>, keys: &[String], delta: isize) {
    if keys.is_empty() {
        *selection = None;
        return;
    }

    let current = selection
        .as_ref()
        .and_then(|selected| keys.iter().position(|key| key == selected))
        .unwrap_or(0);
    let next = wrap_index(current, keys.len(), delta);
    *selection = Some(keys[next].clone());
}

fn select_key_edge(selection: &mut Option<String>, keys: &[String], end: bool) {
    if keys.is_empty() {
        *selection = None;
        return;
    }
    *selection = Some(if end {
        keys[keys.len() - 1].clone()
    } else {
        keys[0].clone()
    });
}

fn wrap_index(current: usize, len: usize, delta: isize) -> usize {
    let len = len as isize;
    (((current as isize) + delta).rem_euclid(len)) as usize
}

fn split_pipeline_key(key: &str) -> Option<(&str, &str)> {
    key.split_once(':')
}

fn pipeline_is_ready(status: &otap_df_admin_api::pipelines::Status) -> bool {
    status.conditions.iter().any(|condition| {
        condition.kind == otap_df_admin_api::pipelines::ConditionKind::Ready
            && condition.status == otap_df_admin_api::pipelines::ConditionStatus::True
    })
}

fn pipeline_is_terminal(status: &otap_df_admin_api::pipelines::Status) -> bool {
    let phases_terminal = if let Some(instances) = &status.instances {
        !instances.is_empty()
            && instances.iter().all(|instance| {
                matches!(
                    instance.status.phase,
                    otap_df_admin_api::pipelines::Phase::Stopped
                        | otap_df_admin_api::pipelines::Phase::Deleted
                        | otap_df_admin_api::pipelines::Phase::Failed(_)
                        | otap_df_admin_api::pipelines::Phase::Rejected(_)
                )
            })
    } else {
        !status.cores.is_empty()
            && status.cores.values().all(|core| {
                matches!(
                    core.phase,
                    otap_df_admin_api::pipelines::Phase::Stopped
                        | otap_df_admin_api::pipelines::Phase::Deleted
                        | otap_df_admin_api::pipelines::Phase::Failed(_)
                        | otap_df_admin_api::pipelines::Phase::Rejected(_)
                )
            })
    };
    phases_terminal && status.running_cores == 0
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_admin_api::pipelines::Status as PipelineStatus;
    use serde_json::json;

    fn pipeline_status(running_cores: usize, total_cores: usize, ready: bool) -> PipelineStatus {
        let conditions = if ready {
            vec![json!({
                "type": "Ready",
                "status": "True",
                "reason": "Ready",
                "message": "ok"
            })]
        } else {
            Vec::new()
        };
        serde_json::from_value(json!({
            "conditions": conditions.clone(),
            "totalCores": total_cores,
            "runningCores": running_cores,
            "cores": {
                "0": {
                    "phase": if running_cores > 0 { "running" } else { "stopped" },
                    "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                    "conditions": conditions,
                    "deletePending": false
                }
            }
        }))
        .expect("status fixture should deserialize")
    }

    #[test]
    fn pipeline_items_apply_filter() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([
                ("tenant-a:ingest".to_string(), pipeline_status(1, 1, true)),
                ("tenant-b:export".to_string(), pipeline_status(1, 1, true)),
            ]),
        });
        app.filter_query = "tenant-a".to_string();

        let items = app.pipeline_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].key, "tenant-a:ingest");
    }

    #[test]
    fn group_items_aggregate_counts() {
        let mut app = AppState::new(UiStartView::Groups, true, 200);
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([
                ("tenant-a:ingest".to_string(), pipeline_status(1, 1, true)),
                (
                    "tenant-a:transform".to_string(),
                    pipeline_status(0, 1, false),
                ),
            ]),
        });

        let items = app.group_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].group_id, "tenant-a");
        assert_eq!(items[0].pipelines, 2);
        assert_eq!(items[0].running, 1);
        assert_eq!(items[0].ready, 1);
        assert_eq!(items[0].terminal, 1);
    }

    #[test]
    fn filter_apply_updates_selection() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([
                ("tenant-a:ingest".to_string(), pipeline_status(1, 1, true)),
                ("tenant-b:export".to_string(), pipeline_status(1, 1, true)),
            ]),
        });
        app.pipeline_selected = Some("tenant-b:export".to_string());
        app.filter_input = "tenant-a".to_string();

        app.apply_filter_input();

        assert_eq!(app.pipeline_selected.as_deref(), Some("tenant-a:ingest"));
    }
}
