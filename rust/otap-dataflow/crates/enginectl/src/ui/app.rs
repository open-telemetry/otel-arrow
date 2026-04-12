// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::args::UiStartView;
use crate::troubleshoot::PipelineDescribeReport;
use humantime::format_duration;
use otap_df_admin_api::{engine, groups, pipelines, telemetry};
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::policy::CoreAllocation;
use std::collections::BTreeMap;
use std::time::{Duration, Instant, SystemTime};

pub(crate) const UI_TITLE: &str = "OpenTelemetry - Rust Dataflow Engine";

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

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandLine {
    pub(crate) label: String,
    pub(crate) command: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandRecipe {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) commands: Vec<CommandLine>,
    pub(crate) note: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiCommandContext {
    pub(crate) target_url: String,
    pub(crate) prefix_args: Vec<String>,
    pub(crate) refresh_interval: Duration,
    pub(crate) logs_tail: usize,
}

impl UiCommandContext {
    pub(crate) fn local_default(logs_tail: usize) -> Self {
        Self {
            target_url: "http://127.0.0.1:8085".to_string(),
            prefix_args: vec![
                "dfctl".to_string(),
                "--url".to_string(),
                "http://127.0.0.1:8085".to_string(),
            ],
            refresh_interval: Duration::from_secs(2),
            logs_tail,
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

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum UiModal {
    #[default]
    None,
    Help,
    Filter,
    Command,
    ActionMenu(ActionMenuState),
    ConfirmShutdown(ShutdownConfirmState),
    ScaleEditor(ScaleEditorState),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum UiAction {
    PipelineEditAndRedeploy {
        group_id: String,
        pipeline_id: String,
    },
    PipelineDeployDraft {
        group_id: String,
        pipeline_id: String,
    },
    PipelineScale {
        group_id: String,
        pipeline_id: String,
        target_cores: usize,
    },
    PipelineSetCoreCount {
        group_id: String,
        pipeline_id: String,
        current_cores: usize,
    },
    PipelineShutdown {
        group_id: String,
        pipeline_id: String,
    },
    GroupShutdown {
        group_id: String,
        pipelines: Vec<String>,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionMenuEntry {
    pub(crate) label: String,
    pub(crate) detail: String,
    pub(crate) action: UiAction,
    pub(crate) enabled: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionMenuState {
    pub(crate) title: String,
    pub(crate) target: String,
    pub(crate) entries: Vec<ActionMenuEntry>,
    pub(crate) selected: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShutdownConfirmState {
    pub(crate) title: String,
    pub(crate) prompt: String,
    pub(crate) action: UiAction,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScaleEditorState {
    pub(crate) group_id: String,
    pub(crate) pipeline_id: String,
    pub(crate) current_cores: usize,
    pub(crate) input: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GroupShutdownRow {
    pub(crate) pipeline: String,
    pub(crate) running: String,
    pub(crate) terminal: String,
    pub(crate) phases: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GroupShutdownPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) rows: Vec<GroupShutdownRow>,
    pub(crate) empty_message: String,
    pub(crate) note: Option<String>,
}

#[derive(Clone, Debug)]
pub(crate) struct ActiveGroupShutdown {
    pub(crate) group_id: String,
    pub(crate) pipeline_count: usize,
    pub(crate) started_at: SystemTime,
    pub(crate) wait_timeout: Duration,
    pub(crate) submission_errors: Vec<String>,
    pub(crate) request_count: usize,
}

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum PipelineTab {
    Summary,
    Config,
    Events,
    Logs,
    Metrics,
    Rollout,
    Shutdown,
    Diagnose,
    Bundle,
}

impl PipelineTab {
    pub(crate) const ALL: [Self; 9] = [
        Self::Summary,
        Self::Config,
        Self::Events,
        Self::Logs,
        Self::Metrics,
        Self::Rollout,
        Self::Shutdown,
        Self::Diagnose,
        Self::Bundle,
    ];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Config => "Config",
            Self::Events => "Events",
            Self::Logs => "Logs",
            Self::Metrics => "Metrics",
            Self::Rollout => "Rollout",
            Self::Shutdown => "Shutdown",
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
    Shutdown,
    Diagnose,
    Bundle,
}

impl GroupTab {
    pub(crate) const ALL: [Self; 7] = [
        Self::Summary,
        Self::Events,
        Self::Logs,
        Self::Metrics,
        Self::Shutdown,
        Self::Diagnose,
        Self::Bundle,
    ];

    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Events => "Events",
            Self::Logs => "Logs",
            Self::Metrics => "Metrics",
            Self::Shutdown => "Shutdown",
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

#[derive(Copy, Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum Tone {
    #[default]
    Neutral,
    Accent,
    Success,
    Warning,
    Failure,
    Muted,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct StatusChip {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DetailHeader {
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) chips: Vec<StatusChip>,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct StatCard {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConditionRow {
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) reason: String,
    pub(crate) message: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CoreRow {
    pub(crate) core: String,
    pub(crate) generation: String,
    pub(crate) phase: String,
    pub(crate) heartbeat: String,
    pub(crate) delete_pending: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TimelineRow {
    pub(crate) time: String,
    pub(crate) kind: String,
    pub(crate) scope: String,
    pub(crate) message: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct LogRow {
    pub(crate) time: String,
    pub(crate) level: String,
    pub(crate) target: String,
    pub(crate) message: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct MetricRow {
    pub(crate) metric_set: String,
    pub(crate) metric: String,
    pub(crate) instrument: String,
    pub(crate) unit: String,
    pub(crate) value: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct OperationRow {
    pub(crate) core: String,
    pub(crate) state: String,
    pub(crate) current_generation: String,
    pub(crate) previous_generation: String,
    pub(crate) updated_at: String,
    pub(crate) detail: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FindingRow {
    pub(crate) severity: String,
    pub(crate) code: String,
    pub(crate) summary: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EvidenceRow {
    pub(crate) source: String,
    pub(crate) time: String,
    pub(crate) message: String,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PipelineInventoryRow {
    pub(crate) pipeline: String,
    pub(crate) running: String,
    pub(crate) ready: String,
    pub(crate) active_generation: String,
    pub(crate) rollout: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProbeFailureRow {
    pub(crate) pipeline: String,
    pub(crate) condition: String,
    pub(crate) message: String,
    pub(crate) tone: Tone,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PipelineSummaryPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) conditions: Vec<ConditionRow>,
    pub(crate) cores: Vec<CoreRow>,
    pub(crate) events: Vec<TimelineRow>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GroupSummaryPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) problem_pipelines: Vec<PipelineInventoryRow>,
    pub(crate) pipelines: Vec<PipelineInventoryRow>,
    pub(crate) events: Vec<TimelineRow>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EngineSummaryPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) pipelines: Vec<PipelineInventoryRow>,
    pub(crate) failing: Vec<ProbeFailureRow>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EventPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) rows: Vec<TimelineRow>,
    pub(crate) empty_message: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct MetricsPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) timestamp: Option<String>,
    pub(crate) rows: Vec<MetricRow>,
    pub(crate) empty_message: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct OperationPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) rows: Vec<OperationRow>,
    pub(crate) empty_message: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct DiagnosisPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) summary: String,
    pub(crate) findings: Vec<FindingRow>,
    pub(crate) evidence: Vec<EvidenceRow>,
    pub(crate) next_steps: Vec<String>,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct BundlePane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) preview: String,
}

#[derive(Clone, Debug, Default)]
pub(crate) struct ConfigPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) note: Option<String>,
    pub(crate) preview_title: String,
    pub(crate) preview: String,
}

#[derive(Clone, Debug)]
pub(crate) struct PipelineConfigDraft {
    pub(crate) original_yaml: String,
    pub(crate) edited_yaml: String,
    pub(crate) parsed: Option<PipelineConfig>,
    pub(crate) diff: String,
    pub(crate) error: Option<String>,
}

impl PipelineConfigDraft {
    pub(crate) fn is_deployable(&self) -> bool {
        self.parsed.is_some() && self.edited_yaml != self.original_yaml
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct LogFeedState {
    pub(crate) scope_key: Option<String>,
    pub(crate) next_seq: Option<u64>,
    pub(crate) response: Option<telemetry::LogsResponse>,
    pub(crate) header: Option<DetailHeader>,
    pub(crate) rows: Vec<LogRow>,
    pub(crate) empty_message: String,
}

impl LogFeedState {
    pub(crate) fn reset(&mut self, scope_key: String) {
        self.scope_key = Some(scope_key);
        self.next_seq = None;
        self.response = None;
        self.header = None;
        self.rows.clear();
        self.empty_message.clear();
    }

    pub(crate) fn clear(&mut self) {
        self.scope_key = None;
        self.next_seq = None;
        self.response = None;
        self.header = None;
        self.rows.clear();
        self.empty_message.clear();
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct PipelinePaneState {
    pub(crate) target_key: Option<String>,
    pub(crate) describe: Option<PipelineDescribeReport>,
    pub(crate) active_rollout_id: Option<String>,
    pub(crate) active_shutdown_id: Option<String>,
    pub(crate) config_yaml: Option<String>,
    pub(crate) config_draft: Option<PipelineConfigDraft>,
    pub(crate) summary: PipelineSummaryPane,
    pub(crate) config: ConfigPane,
    pub(crate) events: EventPane,
    pub(crate) logs: LogFeedState,
    pub(crate) metrics: MetricsPane,
    pub(crate) rollout: OperationPane,
    pub(crate) shutdown: OperationPane,
    pub(crate) diagnosis: DiagnosisPane,
    pub(crate) bundle: BundlePane,
}

impl PipelinePaneState {
    pub(crate) fn reset(&mut self, target_key: String) {
        self.target_key = Some(target_key.clone());
        self.describe = None;
        self.active_rollout_id = None;
        self.active_shutdown_id = None;
        self.config_yaml = None;
        self.config_draft = None;
        self.summary = PipelineSummaryPane::default();
        self.config = ConfigPane::default();
        self.events = EventPane::default();
        self.logs.reset(format!("pipeline:{target_key}"));
        self.metrics = MetricsPane::default();
        self.rollout = OperationPane::default();
        self.shutdown = OperationPane::default();
        self.diagnosis = DiagnosisPane::default();
        self.bundle = BundlePane::default();
    }

    pub(crate) fn clear(&mut self) {
        self.target_key = None;
        self.describe = None;
        self.active_rollout_id = None;
        self.active_shutdown_id = None;
        self.config_yaml = None;
        self.config_draft = None;
        self.summary = PipelineSummaryPane::default();
        self.config = ConfigPane::default();
        self.events = EventPane::default();
        self.logs.clear();
        self.metrics = MetricsPane::default();
        self.rollout = OperationPane::default();
        self.shutdown = OperationPane::default();
        self.diagnosis = DiagnosisPane::default();
        self.bundle = BundlePane::default();
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct GroupPaneState {
    pub(crate) group_id: Option<String>,
    pub(crate) active_shutdown: Option<ActiveGroupShutdown>,
    pub(crate) summary: GroupSummaryPane,
    pub(crate) events: EventPane,
    pub(crate) logs: LogFeedState,
    pub(crate) metrics: MetricsPane,
    pub(crate) shutdown: GroupShutdownPane,
    pub(crate) diagnosis: DiagnosisPane,
    pub(crate) bundle: BundlePane,
}

impl GroupPaneState {
    pub(crate) fn reset(&mut self, group_id: String) {
        self.group_id = Some(group_id.clone());
        if self
            .active_shutdown
            .as_ref()
            .is_some_and(|shutdown| shutdown.group_id != group_id)
        {
            self.active_shutdown = None;
        }
        self.summary = GroupSummaryPane::default();
        self.events = EventPane::default();
        self.logs.reset(format!("group:{group_id}"));
        self.metrics = MetricsPane::default();
        self.shutdown = GroupShutdownPane::default();
        self.diagnosis = DiagnosisPane::default();
        self.bundle = BundlePane::default();
    }

    pub(crate) fn clear(&mut self) {
        self.group_id = None;
        self.active_shutdown = None;
        self.summary = GroupSummaryPane::default();
        self.events = EventPane::default();
        self.logs.clear();
        self.metrics = MetricsPane::default();
        self.shutdown = GroupShutdownPane::default();
        self.diagnosis = DiagnosisPane::default();
        self.bundle = BundlePane::default();
    }
}

#[derive(Clone, Debug, Default)]
pub(crate) struct EnginePaneState {
    pub(crate) summary: EngineSummaryPane,
    pub(crate) logs: LogFeedState,
    pub(crate) metrics: MetricsPane,
}

impl EnginePaneState {
    pub(crate) fn new() -> Self {
        let mut state = Self::default();
        state.logs.reset("engine".to_string());
        state
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct PipelineItem {
    pub(crate) key: String,
    pub(crate) status_badge: String,
    pub(crate) tone: Tone,
    pub(crate) pipeline_group_id: String,
    pub(crate) pipeline_id: String,
    pub(crate) running: String,
    pub(crate) active_generation: String,
    pub(crate) rollout: String,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GroupItem {
    pub(crate) group_id: String,
    pub(crate) status_badge: String,
    pub(crate) tone: Tone,
    pub(crate) pipelines: usize,
    pub(crate) running: usize,
    pub(crate) ready: usize,
    pub(crate) terminal: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnginePipelineItem {
    pub(crate) key: String,
    pub(crate) status_badge: String,
    pub(crate) tone: Tone,
    pub(crate) running: String,
    pub(crate) active_generation: String,
    pub(crate) rollout: String,
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
    pub(crate) modal: UiModal,
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
    pub(crate) command_context: UiCommandContext,
    pub(crate) pipelines: PipelinePaneState,
    pub(crate) groups: GroupPaneState,
    pub(crate) engine: EnginePaneState,
}

impl AppState {
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
            last_refresh: None,
            last_error: None,
            groups_status: None,
            engine_status: None,
            engine_livez: None,
            engine_readyz: None,
            command_context: UiCommandContext::local_default(logs_tail),
            pipelines: PipelinePaneState::default(),
            groups: GroupPaneState::default(),
            engine: EnginePaneState::new(),
        }
    }

    pub(crate) fn set_command_context(&mut self, command_context: UiCommandContext) {
        self.command_context = command_context;
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

    pub(crate) fn current_list_title(&self) -> &'static str {
        match self.view {
            View::Pipelines => "Pipelines",
            View::Groups => "Groups",
            View::Engine => "Engine Pipelines",
        }
    }

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

    pub(crate) fn current_focus_label(&self) -> &'static str {
        match self.focus {
            FocusArea::List => "list",
            FocusArea::Detail => "detail",
        }
    }

    pub(crate) fn target_url(&self) -> &str {
        &self.command_context.target_url
    }

    pub(crate) fn active_header(&self) -> Option<&DetailHeader> {
        match self.view {
            View::Pipelines => match self.pipeline_tab {
                PipelineTab::Summary => self.pipelines.summary.header.as_ref(),
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
                GroupTab::Events => self.groups.events.header.as_ref(),
                GroupTab::Logs => self.groups.logs.header.as_ref(),
                GroupTab::Metrics => self.groups.metrics.header.as_ref(),
                GroupTab::Shutdown => self.groups.shutdown.header.as_ref(),
                GroupTab::Diagnose => self.groups.diagnosis.header.as_ref(),
                GroupTab::Bundle => self.groups.bundle.header.as_ref(),
            },
            View::Engine => match self.engine_tab {
                EngineTab::Summary => self.engine.summary.header.as_ref(),
                EngineTab::Logs => self.engine.logs.header.as_ref(),
                EngineTab::Metrics => self.engine.metrics.header.as_ref(),
            },
        }
    }

    pub(crate) fn current_detail_title(&self) -> String {
        match self.view {
            View::Pipelines => self.pipeline_tab.title().to_string(),
            View::Groups => self.group_tab.title().to_string(),
            View::Engine => self.engine_tab.title().to_string(),
        }
    }

    pub(crate) fn current_command_recipe(&self) -> CommandRecipe {
        match self.view {
            View::Pipelines => self.pipeline_command_recipe(),
            View::Groups => self.group_command_recipe(),
            View::Engine => self.engine_command_recipe(),
        }
    }

    pub(crate) fn start_filter_input(&mut self) {
        self.modal = UiModal::Filter;
        self.filter_input = self.filter_query.clone();
    }

    pub(crate) fn apply_filter_input(&mut self) {
        self.filter_query = self.filter_input.trim().to_string();
        self.modal = UiModal::None;
        self.ensure_selection();
    }

    pub(crate) fn cancel_filter_input(&mut self) {
        self.modal = UiModal::None;
        self.filter_input.clear();
    }

    pub(crate) fn clear_filter(&mut self) {
        self.filter_query.clear();
        self.modal = UiModal::None;
        self.filter_input.clear();
        self.ensure_selection();
    }

    pub(crate) fn reset_scroll(&mut self) {
        self.detail_scroll = 0;
    }

    pub(crate) fn is_filter_mode(&self) -> bool {
        matches!(self.modal, UiModal::Filter)
    }

    pub(crate) fn show_help(&self) -> bool {
        matches!(self.modal, UiModal::Help)
    }

    pub(crate) fn show_command_overlay(&self) -> bool {
        matches!(self.modal, UiModal::Command)
    }

    pub(crate) fn action_menu(&self) -> Option<&ActionMenuState> {
        match &self.modal {
            UiModal::ActionMenu(menu) => Some(menu),
            _ => None,
        }
    }

    pub(crate) fn shutdown_confirm(&self) -> Option<&ShutdownConfirmState> {
        match &self.modal {
            UiModal::ConfirmShutdown(confirm) => Some(confirm),
            _ => None,
        }
    }

    pub(crate) fn scale_editor(&self) -> Option<&ScaleEditorState> {
        match &self.modal {
            UiModal::ScaleEditor(editor) => Some(editor),
            _ => None,
        }
    }

    pub(crate) fn action_menu_mut(&mut self) -> Option<&mut ActionMenuState> {
        match &mut self.modal {
            UiModal::ActionMenu(menu) => Some(menu),
            _ => None,
        }
    }

    pub(crate) fn scale_editor_mut(&mut self) -> Option<&mut ScaleEditorState> {
        match &mut self.modal {
            UiModal::ScaleEditor(editor) => Some(editor),
            _ => None,
        }
    }

    pub(crate) fn has_deployable_pipeline_config_draft(&self) -> bool {
        self.pipelines
            .config_draft
            .as_ref()
            .is_some_and(PipelineConfigDraft::is_deployable)
    }

    pub(crate) fn discard_pipeline_config_draft(&mut self) {
        self.pipelines.config_draft = None;
    }

    pub(crate) fn stage_pipeline_config_draft(
        &mut self,
        original_yaml: String,
        edited_yaml: String,
        parsed: Option<PipelineConfig>,
        diff: String,
        error: Option<String>,
    ) {
        self.pipelines.config_draft = Some(PipelineConfigDraft {
            original_yaml,
            edited_yaml,
            parsed,
            diff,
            error,
        });
    }

    pub(crate) fn hide_modal(&mut self) {
        self.modal = UiModal::None;
    }

    pub(crate) fn toggle_help(&mut self) {
        self.modal = if self.show_help() {
            UiModal::None
        } else {
            UiModal::Help
        };
    }

    pub(crate) fn toggle_command_overlay(&mut self) {
        self.modal = if self.show_command_overlay() {
            UiModal::None
        } else {
            UiModal::Command
        };
    }

    pub(crate) fn open_action_menu(&mut self) -> bool {
        let Some(menu) = self.build_action_menu() else {
            return false;
        };
        self.modal = UiModal::ActionMenu(menu);
        true
    }

    pub(crate) fn open_shutdown_confirm(&mut self, action: UiAction) {
        let (title, prompt) = match &action {
            UiAction::PipelineShutdown {
                group_id,
                pipeline_id,
            } => (
                "Confirm Pipeline Shutdown".to_string(),
                format!("Submit shutdown for pipeline {group_id}/{pipeline_id}?"),
            ),
            UiAction::GroupShutdown {
                group_id,
                pipelines,
            } => (
                "Confirm Group Shutdown".to_string(),
                format!(
                    "Submit shutdown for all {} pipelines currently visible in group {group_id}?",
                    pipelines.len()
                ),
            ),
            _ => return,
        };
        self.modal = UiModal::ConfirmShutdown(ShutdownConfirmState {
            title,
            prompt,
            action,
        });
    }

    pub(crate) fn open_scale_editor(
        &mut self,
        group_id: String,
        pipeline_id: String,
        current_cores: usize,
    ) {
        self.modal = UiModal::ScaleEditor(ScaleEditorState {
            group_id,
            pipeline_id,
            current_cores,
            input: current_cores.to_string(),
        });
    }

    pub(crate) fn selected_group_pipeline_keys(&self) -> Vec<String> {
        let Some(group_id) = self.selected_group_id() else {
            return Vec::new();
        };
        self.groups_status
            .as_ref()
            .map(|status| {
                status
                    .pipelines
                    .keys()
                    .filter(|key| {
                        key.split_once(':')
                            .is_some_and(|(group, _)| group == group_id)
                    })
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    }

    fn pipeline_command_recipe(&self) -> CommandRecipe {
        let Some((group_id, pipeline_id)) = self.selected_pipeline_target() else {
            return empty_recipe(
                "Equivalent CLI",
                "Wait for the first refresh and select a pipeline.",
            );
        };
        let selected_key = format!("{group_id}:{pipeline_id}");
        let pipeline_status = self
            .groups_status
            .as_ref()
            .and_then(|status| status.pipelines.get(&selected_key));

        match self.pipeline_tab {
            PipelineTab::Summary => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Describe pipeline {group_id}/{pipeline_id}."),
                commands: vec![command_line(
                    "describe",
                    self.command(vec![
                        "pipelines".to_string(),
                        "describe".to_string(),
                        group_id.clone(),
                        pipeline_id.clone(),
                    ]),
                )],
                note: None,
            },
            PipelineTab::Config => {
                let mut commands = vec![command_line(
                    "get config",
                    self.command(vec![
                        "pipelines".to_string(),
                        "get".to_string(),
                        group_id.clone(),
                        pipeline_id.clone(),
                        "--output".to_string(),
                        "yaml".to_string(),
                    ]),
                )];
                let (description, note) = if self.has_deployable_pipeline_config_draft() {
                    commands.insert(
                        0,
                        command_line(
                            "reconfigure",
                            self.command(vec![
                                "pipelines".to_string(),
                                "reconfigure".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                "--file".to_string(),
                                "<edited.yaml>".to_string(),
                                "--watch".to_string(),
                            ]),
                        ),
                    );
                    (
                        format!(
                            "Review the staged config draft for {group_id}/{pipeline_id} and redeploy it."
                        ),
                        Some(
                            "The UI keeps the edited YAML in memory. The CLI equivalent expects a file path."
                                .to_string(),
                        ),
                    )
                } else {
                    (
                        format!(
                            "Inspect the committed config for {group_id}/{pipeline_id}."
                        ),
                        Some(
                            "Use the action menu or press 'e' on this tab to open the config in an external editor."
                                .to_string(),
                        ),
                    )
                };
                CommandRecipe {
                    title: "Equivalent CLI".to_string(),
                    description,
                    commands,
                    note,
                }
            }
            PipelineTab::Events => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Watch normalized controller events for {group_id}/{pipeline_id}."
                ),
                commands: vec![command_line(
                    "watch events",
                    self.command(vec![
                        "pipelines".to_string(),
                        "events".to_string(),
                        "watch".to_string(),
                        group_id.clone(),
                        pipeline_id.clone(),
                        "--watch-interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                    ]),
                )],
                note: None,
            },
            PipelineTab::Logs => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch retained logs for {group_id}/{pipeline_id}."),
                commands: vec![command_line(
                    "watch logs",
                    self.command(vec![
                        "telemetry".to_string(),
                        "logs".to_string(),
                        "watch".to_string(),
                        "--tail".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                        "--group".to_string(),
                        group_id.clone(),
                        "--pipeline".to_string(),
                        pipeline_id.clone(),
                    ]),
                )],
                note: None,
            },
            PipelineTab::Metrics => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch compact metrics for {group_id}/{pipeline_id}."),
                commands: vec![command_line(
                    "watch metrics",
                    self.command(vec![
                        "telemetry".to_string(),
                        "metrics".to_string(),
                        "watch".to_string(),
                        "--shape".to_string(),
                        "compact".to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                        "--group".to_string(),
                        group_id.clone(),
                        "--pipeline".to_string(),
                        pipeline_id.clone(),
                    ]),
                )],
                note: None,
            },
            PipelineTab::Rollout => {
                if let Some(rollout_id) = self
                    .pipelines
                    .active_rollout_id
                    .clone()
                    .or_else(|| pipeline_status.and_then(active_rollout_id))
                {
                    CommandRecipe {
                        title: "Equivalent CLI".to_string(),
                        description: format!(
                            "Watch active rollout {rollout_id} for {group_id}/{pipeline_id}."
                        ),
                        commands: vec![command_line(
                            "watch rollout",
                            self.command(vec![
                                "pipelines".to_string(),
                                "rollouts".to_string(),
                                "watch".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                rollout_id.clone(),
                                "--interval".to_string(),
                                format_duration_arg(self.command_context.refresh_interval),
                            ]),
                        )],
                        note: None,
                    }
                } else {
                    CommandRecipe {
                        title: "Equivalent CLI".to_string(),
                        description: format!(
                            "No active rollout is visible for {group_id}/{pipeline_id}."
                        ),
                        commands: vec![command_line(
                            "watch rollout",
                            self.command(vec![
                                "pipelines".to_string(),
                                "rollouts".to_string(),
                                "watch".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                "<rollout-id>".to_string(),
                                "--interval".to_string(),
                                format_duration_arg(self.command_context.refresh_interval),
                            ]),
                        )],
                        note: Some(
                            "This pane only has an exact CLI equivalent while a rollout is active."
                                .to_string(),
                        ),
                    }
                }
            }
            PipelineTab::Shutdown => {
                if let Some(shutdown_id) = self.pipelines.active_shutdown_id.clone() {
                    CommandRecipe {
                        title: "Equivalent CLI".to_string(),
                        description: format!(
                            "Watch shutdown {shutdown_id} for {group_id}/{pipeline_id}."
                        ),
                        commands: vec![command_line(
                            "watch shutdown",
                            self.command(vec![
                                "pipelines".to_string(),
                                "shutdowns".to_string(),
                                "watch".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                shutdown_id,
                                "--interval".to_string(),
                                format_duration_arg(self.command_context.refresh_interval),
                            ]),
                        )],
                        note: None,
                    }
                } else {
                    CommandRecipe {
                        title: "Equivalent CLI".to_string(),
                        description: format!(
                            "No active shutdown is tracked for {group_id}/{pipeline_id}."
                        ),
                        commands: vec![command_line(
                            "watch shutdown",
                            self.command(vec![
                                "pipelines".to_string(),
                                "shutdowns".to_string(),
                                "watch".to_string(),
                                group_id.clone(),
                                pipeline_id.clone(),
                                "<shutdown-id>".to_string(),
                                "--interval".to_string(),
                                format_duration_arg(self.command_context.refresh_interval),
                            ]),
                        )],
                        note: Some(
                            "This pane has an exact CLI equivalent only after a shutdown has been submitted."
                                .to_string(),
                        ),
                    }
                }
            }
            PipelineTab::Diagnose => {
                let (label, command) = if let Some(rollout_id) = pipeline_status
                    .and_then(|status| status.rollout.as_ref())
                    .map(|rollout| rollout.rollout_id.clone())
                {
                    (
                        "diagnose rollout",
                        self.command(vec![
                            "pipelines".to_string(),
                            "diagnose".to_string(),
                            "rollout".to_string(),
                            group_id.clone(),
                            pipeline_id.clone(),
                            "--rollout-id".to_string(),
                            rollout_id,
                            "--logs-limit".to_string(),
                            self.command_context.logs_tail.to_string(),
                        ]),
                    )
                } else {
                    (
                        "diagnose shutdown",
                        self.command(vec![
                            "pipelines".to_string(),
                            "diagnose".to_string(),
                            "shutdown".to_string(),
                            group_id.clone(),
                            pipeline_id.clone(),
                            "--logs-limit".to_string(),
                            self.command_context.logs_tail.to_string(),
                        ]),
                    )
                };
                CommandRecipe {
                    title: "Equivalent CLI".to_string(),
                    description: format!("Run diagnosis for {group_id}/{pipeline_id}."),
                    commands: vec![command_line(label, command)],
                    note: None,
                }
            }
            PipelineTab::Bundle => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Export a troubleshooting bundle for {group_id}/{pipeline_id}."
                ),
                commands: vec![command_line(
                    "bundle",
                    self.command(vec![
                        "pipelines".to_string(),
                        "bundle".to_string(),
                        group_id.clone(),
                        pipeline_id.clone(),
                        "--logs-limit".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--metrics-shape".to_string(),
                        "compact".to_string(),
                        "--output".to_string(),
                        "json".to_string(),
                    ]),
                )],
                note: None,
            },
        }
    }

    fn group_command_recipe(&self) -> CommandRecipe {
        let Some(group_id) = self.selected_group_id() else {
            return empty_recipe(
                "Equivalent CLI",
                "Wait for the first refresh and select a group.",
            );
        };

        match self.group_tab {
            GroupTab::Summary => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Describe fleet-wide groups state while focusing on group {group_id}."
                ),
                commands: vec![command_line(
                    "describe groups",
                    self.command(vec!["groups".to_string(), "describe".to_string()]),
                )],
                note: Some(format!(
                    "The UI summary is scoped to group {group_id} client-side. The CLI command is fleet-wide."
                )),
            },
            GroupTab::Events => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch normalized events for group {group_id}."),
                commands: vec![command_line(
                    "watch events",
                    self.command(vec![
                        "groups".to_string(),
                        "events".to_string(),
                        "watch".to_string(),
                        "--group".to_string(),
                        group_id.clone(),
                        "--watch-interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                    ]),
                )],
                note: None,
            },
            GroupTab::Logs => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch retained logs for group {group_id}."),
                commands: vec![command_line(
                    "watch logs",
                    self.command(vec![
                        "telemetry".to_string(),
                        "logs".to_string(),
                        "watch".to_string(),
                        "--tail".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                        "--group".to_string(),
                        group_id.clone(),
                    ]),
                )],
                note: None,
            },
            GroupTab::Metrics => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!("Watch compact metrics for group {group_id}."),
                commands: vec![command_line(
                    "watch metrics",
                    self.command(vec![
                        "telemetry".to_string(),
                        "metrics".to_string(),
                        "watch".to_string(),
                        "--shape".to_string(),
                        "compact".to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                        "--group".to_string(),
                        group_id.clone(),
                    ]),
                )],
                note: None,
            },
            GroupTab::Shutdown => {
                let pipeline_keys = self.selected_group_pipeline_keys();
                let mut commands = Vec::new();
                for key in pipeline_keys {
                    if let Some((group, pipeline)) = split_pipeline_key(&key) {
                        commands.push(command_line(
                            &format!("shutdown {pipeline}"),
                            self.command(vec![
                                "pipelines".to_string(),
                                "shutdown".to_string(),
                                group.to_string(),
                                pipeline.to_string(),
                                "--watch".to_string(),
                            ]),
                        ));
                    }
                }
                CommandRecipe {
                    title: "Equivalent CLI".to_string(),
                    description: format!(
                        "Submit shutdowns for each pipeline currently visible in group {group_id}."
                    ),
                    commands,
                    note: Some(
                        "The UI implements group shutdown client-side because the current admin SDK does not expose a single group-scoped shutdown command."
                            .to_string(),
                    ),
                }
            }
            GroupTab::Diagnose => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Diagnose coordinated shutdown while focusing on group {group_id}."
                ),
                commands: vec![command_line(
                    "diagnose shutdown",
                    self.command(vec![
                        "groups".to_string(),
                        "diagnose".to_string(),
                        "shutdown".to_string(),
                        "--logs-limit".to_string(),
                        self.command_context.logs_tail.to_string(),
                    ]),
                )],
                note: Some(format!(
                    "The UI diagnosis is scoped to group {group_id} client-side. The CLI command is fleet-wide."
                )),
            },
            GroupTab::Bundle => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: format!(
                    "Export a fleet-wide troubleshooting bundle while focusing on group {group_id}."
                ),
                commands: vec![command_line(
                    "bundle",
                    self.command(vec![
                        "groups".to_string(),
                        "bundle".to_string(),
                        "--logs-limit".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--metrics-shape".to_string(),
                        "compact".to_string(),
                        "--output".to_string(),
                        "json".to_string(),
                    ]),
                )],
                note: Some(format!(
                    "The UI bundle preview is scoped to group {group_id} client-side. The CLI command is fleet-wide."
                )),
            },
        }
    }

    fn engine_command_recipe(&self) -> CommandRecipe {
        match self.engine_tab {
            EngineTab::Summary => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: "Inspect engine status and readiness.".to_string(),
                commands: vec![
                    command_line(
                        "status",
                        self.command(vec!["engine".to_string(), "status".to_string()]),
                    ),
                    command_line(
                        "livez",
                        self.command(vec!["engine".to_string(), "livez".to_string()]),
                    ),
                    command_line(
                        "readyz",
                        self.command(vec!["engine".to_string(), "readyz".to_string()]),
                    ),
                ],
                note: None,
            },
            EngineTab::Logs => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: "Watch retained engine logs.".to_string(),
                commands: vec![command_line(
                    "watch logs",
                    self.command(vec![
                        "telemetry".to_string(),
                        "logs".to_string(),
                        "watch".to_string(),
                        "--tail".to_string(),
                        self.command_context.logs_tail.to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                    ]),
                )],
                note: None,
            },
            EngineTab::Metrics => CommandRecipe {
                title: "Equivalent CLI".to_string(),
                description: "Watch compact engine metrics.".to_string(),
                commands: vec![command_line(
                    "watch metrics",
                    self.command(vec![
                        "telemetry".to_string(),
                        "metrics".to_string(),
                        "watch".to_string(),
                        "--shape".to_string(),
                        "compact".to_string(),
                        "--interval".to_string(),
                        format_duration_arg(self.command_context.refresh_interval),
                    ]),
                )],
                note: None,
            },
        }
    }

    fn command<S>(&self, args: impl IntoIterator<Item = S>) -> String
    where
        S: Into<String>,
    {
        let mut parts = self.command_context.prefix_args.clone();
        parts.extend(args.into_iter().map(Into::into));
        shell_join(parts)
    }

    fn build_action_menu(&self) -> Option<ActionMenuState> {
        match self.view {
            View::Pipelines => self.build_pipeline_action_menu(),
            View::Groups => self.build_group_action_menu(),
            View::Engine => None,
        }
    }

    fn build_pipeline_action_menu(&self) -> Option<ActionMenuState> {
        let (group_id, pipeline_id) = self.selected_pipeline_target()?;
        let mut entries = Vec::new();

        match self.selected_pipeline_scale_support() {
            Some(PipelineScaleSupport::Supported { current_cores }) => {
                entries.push(ActionMenuEntry {
                    label: "Scale up".to_string(),
                    detail: format!(
                        "Increase from {current_cores} to {} cores.",
                        current_cores + 1
                    ),
                    action: UiAction::PipelineScale {
                        group_id: group_id.clone(),
                        pipeline_id: pipeline_id.clone(),
                        target_cores: current_cores + 1,
                    },
                    enabled: true,
                });
                entries.push(ActionMenuEntry {
                    label: "Scale down".to_string(),
                    detail: if current_cores > 1 {
                        format!(
                            "Decrease from {current_cores} to {} cores.",
                            current_cores - 1
                        )
                    } else {
                        "Already at the minimum of 1 core.".to_string()
                    },
                    action: UiAction::PipelineScale {
                        group_id: group_id.clone(),
                        pipeline_id: pipeline_id.clone(),
                        target_cores: current_cores.saturating_sub(1),
                    },
                    enabled: current_cores > 1,
                });
                entries.push(ActionMenuEntry {
                    label: "Set core count…".to_string(),
                    detail: format!("Edit the explicit core_count starting from {current_cores}."),
                    action: UiAction::PipelineSetCoreCount {
                        group_id: group_id.clone(),
                        pipeline_id: pipeline_id.clone(),
                        current_cores,
                    },
                    enabled: true,
                });
            }
            Some(PipelineScaleSupport::Unsupported { reason }) => {
                for label in ["Scale up", "Scale down", "Set core count…"] {
                    entries.push(ActionMenuEntry {
                        label: label.to_string(),
                        detail: reason.clone(),
                        action: UiAction::PipelineSetCoreCount {
                            group_id: group_id.clone(),
                            pipeline_id: pipeline_id.clone(),
                            current_cores: 0,
                        },
                        enabled: false,
                    });
                }
            }
            None => {}
        }

        entries.push(ActionMenuEntry {
            label: "Edit and redeploy…".to_string(),
            detail: format!(
                "Open the current config for {group_id}/{pipeline_id} in an external editor."
            ),
            action: UiAction::PipelineEditAndRedeploy {
                group_id: group_id.clone(),
                pipeline_id: pipeline_id.clone(),
            },
            enabled: true,
        });

        entries.push(ActionMenuEntry {
            label: "Shutdown pipeline".to_string(),
            detail: format!("Submit shutdown for {group_id}/{pipeline_id}."),
            action: UiAction::PipelineShutdown {
                group_id: group_id.clone(),
                pipeline_id: pipeline_id.clone(),
            },
            enabled: true,
        });

        Some(ActionMenuState::new(
            "Pipeline Actions",
            format!("{group_id}/{pipeline_id}"),
            entries,
        ))
    }

    fn build_group_action_menu(&self) -> Option<ActionMenuState> {
        let group_id = self.selected_group_id()?;
        let pipelines = self.selected_group_pipeline_keys();
        if pipelines.is_empty() {
            return None;
        }

        Some(ActionMenuState::new(
            "Group Actions",
            group_id.clone(),
            vec![ActionMenuEntry {
                label: "Shutdown group".to_string(),
                detail: format!(
                    "Submit pipeline shutdowns for all {} pipelines currently visible in {group_id}.",
                    pipelines.len()
                ),
                action: UiAction::GroupShutdown {
                    group_id,
                    pipelines,
                },
                enabled: true,
            }],
        ))
    }

    pub(crate) fn selected_pipeline_scale_support(&self) -> Option<PipelineScaleSupport> {
        let describe = self.pipelines.describe.as_ref()?;
        let Some(policies) = describe.details.pipeline.policies() else {
            return Some(PipelineScaleSupport::Unsupported {
                reason: "Scaling in the TUI requires an explicit pipeline resources.core_allocation policy.".to_string(),
            });
        };
        let Some(resources) = policies.resources() else {
            return Some(PipelineScaleSupport::Unsupported {
                reason: "Scaling in the TUI requires an explicit pipeline resources.core_allocation policy.".to_string(),
            });
        };
        match resources.core_allocation {
            CoreAllocation::CoreCount { count } => {
                if count > 0 {
                    Some(PipelineScaleSupport::Supported {
                        current_cores: count,
                    })
                } else {
                    Some(PipelineScaleSupport::Unsupported {
                        reason:
                            "Scaling in the TUI is disabled for core_count=0 because that means all available cores."
                                .to_string(),
                    })
                }
            }
            CoreAllocation::AllCores => Some(PipelineScaleSupport::Unsupported {
                reason:
                    "Scaling in the TUI is disabled for all_cores allocations. Use a full reconfigure instead."
                        .to_string(),
            }),
            CoreAllocation::CoreSet { .. } => Some(PipelineScaleSupport::Unsupported {
                reason:
                    "Scaling in the TUI is disabled for core_set allocations. Use a full reconfigure instead."
                        .to_string(),
            }),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PipelineScaleSupport {
    Supported { current_cores: usize },
    Unsupported { reason: String },
}

impl ActionMenuState {
    fn new(
        title: impl Into<String>,
        target: impl Into<String>,
        entries: Vec<ActionMenuEntry>,
    ) -> Self {
        let selected = entries.iter().position(|entry| entry.enabled).unwrap_or(0);
        Self {
            title: title.into(),
            target: target.into(),
            entries,
            selected,
        }
    }
}

fn empty_recipe(title: &str, description: &str) -> CommandRecipe {
    CommandRecipe {
        title: title.to_string(),
        description: description.to_string(),
        commands: Vec::new(),
        note: None,
    }
}

fn command_line(label: &str, command: String) -> CommandLine {
    CommandLine {
        label: label.to_string(),
        command,
    }
}

fn format_duration_arg(duration: Duration) -> String {
    format_duration(duration).to_string()
}

fn shell_join(parts: Vec<String>) -> String {
    parts
        .into_iter()
        .map(|part| shell_quote(&part))
        .collect::<Vec<_>>()
        .join(" ")
}

fn shell_quote(value: &str) -> String {
    if value.is_empty() {
        return "''".to_string();
    }

    if value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | ':' | '.' | '_' | '-' | '='))
    {
        return value.to_string();
    }

    format!("'{}'", value.replace('\'', "'\"'\"'"))
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

fn classify_pipeline_row(status: &pipelines::Status) -> (String, Tone) {
    if has_active_rollout(status) {
        return ("roll".to_string(), Tone::Accent);
    }
    let has_failure = status.cores.values().any(|core| {
        matches!(
            core.phase,
            pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_)
        )
    }) || status.instances.as_ref().is_some_and(|instances| {
        instances.iter().any(|instance| {
            matches!(
                instance.status.phase,
                pipelines::Phase::Failed(_) | pipelines::Phase::Rejected(_)
            )
        })
    });
    if has_failure {
        return ("fail".to_string(), Tone::Failure);
    }
    if pipeline_is_terminal(status) {
        return ("stop".to_string(), Tone::Muted);
    }
    if !pipeline_is_ready(status) || status.running_cores < status.total_cores {
        return ("warn".to_string(), Tone::Warning);
    }
    ("ok".to_string(), Tone::Success)
}

fn combine_tones(left: Tone, right: Tone) -> Tone {
    use Tone::{Accent, Failure, Muted, Neutral, Success, Warning};
    match (left, right) {
        (Failure, _) | (_, Failure) => Failure,
        (Warning, _) | (_, Warning) => Warning,
        (Accent, _) | (_, Accent) => Accent,
        (Success, _) | (_, Success) => Success,
        (Muted, _) | (_, Muted) => Muted,
        _ => Neutral,
    }
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

fn pipeline_is_ready(status: &pipelines::Status) -> bool {
    status.conditions.iter().any(|condition| {
        condition.kind == pipelines::ConditionKind::Ready
            && condition.status == pipelines::ConditionStatus::True
    })
}

fn pipeline_is_terminal(status: &pipelines::Status) -> bool {
    let phases_terminal = if let Some(instances) = &status.instances {
        !instances.is_empty()
            && instances.iter().all(|instance| {
                matches!(
                    instance.status.phase,
                    pipelines::Phase::Stopped
                        | pipelines::Phase::Deleted
                        | pipelines::Phase::Failed(_)
                        | pipelines::Phase::Rejected(_)
                )
            })
    } else {
        !status.cores.is_empty()
            && status.cores.values().all(|core| {
                matches!(
                    core.phase,
                    pipelines::Phase::Stopped
                        | pipelines::Phase::Deleted
                        | pipelines::Phase::Failed(_)
                        | pipelines::Phase::Rejected(_)
                )
            })
    };
    phases_terminal && status.running_cores == 0
}

fn has_active_rollout(status: &pipelines::Status) -> bool {
    status
        .rollout
        .as_ref()
        .is_some_and(|rollout| !rollout_is_terminal(rollout.state))
}

fn active_rollout_id(status: &pipelines::Status) -> Option<String> {
    status
        .rollout
        .as_ref()
        .filter(|rollout| !rollout_is_terminal(rollout.state))
        .map(|rollout| rollout.rollout_id.clone())
}

fn rollout_is_terminal(state: pipelines::PipelineRolloutState) -> bool {
    matches!(
        state,
        pipelines::PipelineRolloutState::Succeeded
            | pipelines::PipelineRolloutState::Failed
            | pipelines::PipelineRolloutState::RollbackFailed
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_admin_api::pipelines::Status as PipelineStatus;
    use serde_json::json;
    use std::time::Duration;

    fn pipeline_status(
        running_cores: usize,
        total_cores: usize,
        ready: bool,
        with_rollout: bool,
    ) -> PipelineStatus {
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
        let mut value = json!({
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
        });
        if with_rollout {
            value["rollout"] = json!({
                "rolloutId": "rollout-0",
                "state": "running",
                "targetGeneration": 1,
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:01Z"
            });
        }
        serde_json::from_value(value).expect("status fixture should deserialize")
    }

    fn pipeline_status_with_rollout_state(state: &str) -> PipelineStatus {
        let mut value = json!({
            "conditions": [{
                "type": "Ready",
                "status": "True",
                "reason": "Ready",
                "message": "ok"
            }],
            "totalCores": 2,
            "runningCores": 2,
            "activeGeneration": 0,
            "cores": {
                "0": {
                    "phase": "running",
                    "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                    "conditions": [],
                    "deletePending": false
                },
                "1": {
                    "phase": "running",
                    "lastHeartbeatTime": "2026-01-01T00:00:00Z",
                    "conditions": [],
                    "deletePending": false
                }
            },
            "rollout": {
                "rolloutId": "rollout-0",
                "state": state,
                "targetGeneration": 1,
                "startedAt": "2026-01-01T00:00:00Z",
                "updatedAt": "2026-01-01T00:00:01Z"
            }
        });
        if state == "succeeded" {
            value["rollout"]["failureReason"] = serde_json::Value::Null;
        }
        serde_json::from_value(value).expect("status fixture should deserialize")
    }

    #[test]
    fn pipeline_items_apply_filter() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([
                (
                    "tenant-a:ingest".to_string(),
                    pipeline_status(1, 1, true, false),
                ),
                (
                    "tenant-b:export".to_string(),
                    pipeline_status(1, 1, true, false),
                ),
            ]),
        });
        app.filter_query = "tenant-a".to_string();

        let items = app.pipeline_items();
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].key, "tenant-a:ingest");
        assert_eq!(items[0].status_badge, "ok");
    }

    #[test]
    fn group_items_aggregate_counts() {
        let mut app = AppState::new(UiStartView::Groups, true, 200);
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([
                (
                    "tenant-a:ingest".to_string(),
                    pipeline_status(1, 1, true, false),
                ),
                (
                    "tenant-a:transform".to_string(),
                    pipeline_status(0, 1, false, false),
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
        assert_eq!(items[0].status_badge, "warn");
    }

    #[test]
    fn filter_apply_updates_selection() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([
                (
                    "tenant-a:ingest".to_string(),
                    pipeline_status(1, 1, true, false),
                ),
                (
                    "tenant-b:export".to_string(),
                    pipeline_status(1, 1, true, true),
                ),
            ]),
        });
        app.pipeline_selected = Some("tenant-b:export".to_string());
        app.filter_input = "tenant-a".to_string();

        app.apply_filter_input();

        assert_eq!(app.pipeline_selected.as_deref(), Some("tenant-a:ingest"));
    }

    #[test]
    fn rollout_pipeline_items_show_roll_badge() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([(
                "tenant-a:ingest".to_string(),
                pipeline_status(1, 1, true, true),
            )]),
        });

        let items = app.pipeline_items();
        assert_eq!(items[0].status_badge, "roll");
        assert_eq!(items[0].tone, Tone::Accent);
    }

    #[test]
    fn terminal_rollout_pipeline_items_fall_back_to_ok_badge() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([(
                "tenant-a:ingest".to_string(),
                pipeline_status_with_rollout_state("succeeded"),
            )]),
        });

        let items = app.pipeline_items();
        assert_eq!(items[0].status_badge, "ok");
        assert_eq!(items[0].tone, Tone::Success);
        assert_eq!(items[0].rollout, "succeeded");
    }

    #[test]
    fn rollout_recipe_ignores_terminal_rollout_summary() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.pipeline_selected = Some("tenant-a:ingest".to_string());
        app.pipeline_tab = PipelineTab::Rollout;
        app.groups_status = Some(groups::Status {
            generated_at: "2026-01-01T00:00:00Z".to_string(),
            pipelines: BTreeMap::from([(
                "tenant-a:ingest".to_string(),
                pipeline_status_with_rollout_state("succeeded"),
            )]),
        });

        let recipe = app.current_command_recipe();

        assert!(recipe.description.contains("No active rollout is visible"));
        assert!(recipe.commands[0].command.contains("<rollout-id>"));
    }

    #[test]
    fn pipeline_logs_recipe_uses_selected_target_and_context() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.pipeline_selected = Some("tenant-a:ingest".to_string());
        app.pipeline_tab = PipelineTab::Logs;
        app.set_command_context(UiCommandContext {
            target_url: "https://admin.example.com:8443/engine-a".to_string(),
            prefix_args: vec![
                "dfctl".to_string(),
                "--url".to_string(),
                "https://admin.example.com:8443/engine-a".to_string(),
                "--color".to_string(),
                "always".to_string(),
            ],
            refresh_interval: Duration::from_secs(5),
            logs_tail: 250,
        });

        let recipe = app.current_command_recipe();

        assert_eq!(recipe.commands.len(), 1);
        assert!(recipe.commands[0].command.contains(
            "telemetry logs watch --tail 250 --interval 5s --group tenant-a --pipeline ingest"
        ));
        assert!(recipe.commands[0].command.contains("--color always"));
    }

    #[test]
    fn pipeline_config_recipe_points_to_get_and_editor_flow() {
        let mut app = AppState::new(UiStartView::Pipelines, true, 200);
        app.pipeline_selected = Some("tenant-a:ingest".to_string());
        app.pipeline_tab = PipelineTab::Config;

        let recipe = app.current_command_recipe();

        assert_eq!(recipe.commands.len(), 1);
        assert_eq!(
            recipe.commands[0].command,
            "dfctl --url http://127.0.0.1:8085 pipelines get tenant-a ingest --output yaml"
        );
        assert!(
            recipe
                .note
                .expect("config recipe should include note")
                .contains("external editor")
        );
    }

    #[test]
    fn group_summary_recipe_mentions_client_side_scope() {
        let mut app = AppState::new(UiStartView::Groups, true, 200);
        app.group_selected = Some("tenant-a".to_string());

        let recipe = app.current_command_recipe();

        assert_eq!(
            recipe.commands[0].command,
            "dfctl --url http://127.0.0.1:8085 groups describe"
        );
        assert!(
            recipe
                .note
                .expect("group summary recipe should include note")
                .contains("client-side")
        );
    }
}
