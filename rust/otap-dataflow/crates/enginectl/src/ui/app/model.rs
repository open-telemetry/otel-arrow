// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core UI data models, pane state, modal state, and render-ready row types.
//!
//! This module defines the data contract shared by refresh code, input handlers,
//! pane builders, and renderers. The structs intentionally store already scoped,
//! classified, or formatted data where useful so the rendering layer can stay
//! simple and the rest of the TUI can reason about one authoritative state tree.

use super::*;
use crate::BIN_NAME;

use std::time::{Duration, Instant, SystemTime};

/// Top-level TUI object family currently selected by the user.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum View {
    Pipelines,
    Groups,
    Engine,
}

impl View {
    /// Display order for the first-level tab bar.
    pub(crate) const ALL: [Self; 3] = [Self::Engine, Self::Groups, Self::Pipelines];

    /// Returns the tab title shown in the TUI.
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Pipelines => "Pipelines",
            Self::Groups => "Groups",
            Self::Engine => "Engine",
        }
    }
}

/// One executable CLI command shown by the command overlay.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandLine {
    pub(crate) label: String,
    pub(crate) command: String,
}

/// TUI recipe that explains how to reproduce the current view from the CLI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandRecipe {
    pub(crate) title: String,
    pub(crate) description: String,
    pub(crate) commands: Vec<CommandLine>,
    pub(crate) note: Option<String>,
}

/// Connection and display context used to build equivalent CLI commands in the TUI.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct UiCommandContext {
    pub(crate) target_url: String,
    pub(crate) prefix_args: Vec<String>,
    pub(crate) sensitive_args_redacted: bool,
    pub(crate) refresh_interval: Duration,
    pub(crate) logs_tail: usize,
}

impl UiCommandContext {
    /// Builds a local fallback context for tests and early UI initialization.
    pub(crate) fn local_default(logs_tail: usize) -> Self {
        Self {
            target_url: "http://127.0.0.1:8085".to_string(),
            prefix_args: vec![
                BIN_NAME.to_string(),
                "--url".to_string(),
                "http://127.0.0.1:8085".to_string(),
            ],
            sensitive_args_redacted: false,
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

/// Which major UI region receives keyboard navigation.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum FocusArea {
    List,
    Detail,
}

/// Active modal overlay, if any.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) enum UiModal {
    #[default]
    None,
    Help,
    Filter,
    Command,
    CommandPalette(CommandPaletteState),
    ActionMenu(ActionMenuState),
    ConfirmShutdown(ShutdownConfirmState),
    ScaleEditor(ScaleEditorState),
}

/// Command palette action resolved from a selected palette entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) enum PaletteAction {
    SwitchView(View),
    SelectPipelineTab(PipelineTab),
    SelectGroupTab(GroupTab),
    SelectEngineTab(EngineTab),
    Focus(FocusArea),
    OpenHelp,
    OpenFilter,
    OpenCommandOverlay,
    OpenActionMenu,
    ClearFilter,
    Refresh,
    Quit,
    Execute(UiAction),
}

/// One row in the command palette.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct CommandPaletteEntry {
    pub(crate) label: String,
    pub(crate) detail: String,
    pub(crate) keywords: String,
    pub(crate) action: PaletteAction,
    pub(crate) enabled: bool,
}

/// Mutable input and selection state for the command palette modal.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CommandPaletteState {
    pub(crate) input: String,
    pub(crate) selected: usize,
}

/// User-triggered operation that may execute admin calls or edit local state.
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

/// One row in the context-sensitive action menu.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionMenuEntry {
    pub(crate) label: String,
    pub(crate) detail: String,
    pub(crate) action: UiAction,
    pub(crate) enabled: bool,
}

/// Mutable state for the context-sensitive action menu modal.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ActionMenuState {
    pub(crate) title: String,
    pub(crate) target: String,
    pub(crate) entries: Vec<ActionMenuEntry>,
    pub(crate) selected: usize,
}

/// Confirmation prompt for destructive shutdown actions.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ShutdownConfirmState {
    pub(crate) title: String,
    pub(crate) prompt: String,
    pub(crate) action: UiAction,
}

/// Modal state for editing a pipeline's target core count.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct ScaleEditorState {
    pub(crate) group_id: String,
    pub(crate) pipeline_id: String,
    pub(crate) current_cores: usize,
    pub(crate) input: String,
}

/// Render row for group shutdown progress.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct GroupShutdownRow {
    pub(crate) pipeline: String,
    pub(crate) running: String,
    pub(crate) terminal: String,
    pub(crate) phases: String,
    pub(crate) tone: Tone,
}

/// Pane state for the group shutdown progress tab.
#[derive(Clone, Debug, Default)]
pub(crate) struct GroupShutdownPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) rows: Vec<GroupShutdownRow>,
    pub(crate) empty_message: String,
    pub(crate) note: Option<String>,
}

/// Client-side tracking state for an in-progress group shutdown action.
#[derive(Clone, Debug)]
pub(crate) struct ActiveGroupShutdown {
    pub(crate) group_id: String,
    pub(crate) pipeline_count: usize,
    pub(crate) started_at: SystemTime,
    pub(crate) wait_timeout: Duration,
    pub(crate) submission_errors: Vec<String>,
    pub(crate) request_count: usize,
}

/// Second-level tab selection for a pipeline object.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum PipelineTab {
    Summary,
    Details,
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
    /// Display order for pipeline detail tabs.
    pub(crate) const ALL: [Self; 10] = [
        Self::Summary,
        Self::Details,
        Self::Config,
        Self::Events,
        Self::Logs,
        Self::Metrics,
        Self::Rollout,
        Self::Shutdown,
        Self::Diagnose,
        Self::Bundle,
    ];

    /// Returns the tab title shown in the TUI.
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Details => "Details",
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

/// Second-level tab selection for a group object.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum GroupTab {
    Summary,
    Details,
    Events,
    Logs,
    Metrics,
    Shutdown,
    Diagnose,
    Bundle,
}

impl GroupTab {
    /// Display order for group detail tabs.
    pub(crate) const ALL: [Self; 8] = [
        Self::Summary,
        Self::Details,
        Self::Events,
        Self::Logs,
        Self::Metrics,
        Self::Shutdown,
        Self::Diagnose,
        Self::Bundle,
    ];

    /// Returns the tab title shown in the TUI.
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Details => "Details",
            Self::Events => "Events",
            Self::Logs => "Logs",
            Self::Metrics => "Metrics",
            Self::Shutdown => "Shutdown",
            Self::Diagnose => "Diagnose",
            Self::Bundle => "Bundle",
        }
    }
}

/// Second-level tab selection for the engine object.
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub(crate) enum EngineTab {
    Summary,
    Details,
    Logs,
    Metrics,
}

impl EngineTab {
    /// Display order for engine detail tabs.
    pub(crate) const ALL: [Self; 4] = [Self::Summary, Self::Details, Self::Logs, Self::Metrics];

    /// Returns the tab title shown in the TUI.
    pub(crate) const fn title(self) -> &'static str {
        match self {
            Self::Summary => "Summary",
            Self::Details => "Details",
            Self::Logs => "Logs",
            Self::Metrics => "Metrics",
        }
    }
}

/// Semantic color category used by TUI rows, cards, and chips.
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

/// Compact key/value status shown in detail headers.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct StatusChip {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) tone: Tone,
}

/// Header content shared by summary and detail panes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct DetailHeader {
    pub(crate) title: String,
    pub(crate) subtitle: Option<String>,
    pub(crate) chips: Vec<StatusChip>,
}

/// Small summary value card shown at the top of panes.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct StatCard {
    pub(crate) label: String,
    pub(crate) value: String,
    pub(crate) tone: Tone,
}

/// Engine-level CPU, memory, and pressure values shown in the header rail.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EngineVitals {
    pub(crate) cpu_utilization: String,
    pub(crate) cpu_tone: Tone,
    pub(crate) memory_rss: String,
    pub(crate) memory_tone: Tone,
    pub(crate) pressure_state: String,
    pub(crate) pressure_tone: Tone,
    pub(crate) pressure_detail: Option<String>,
    pub(crate) stale: bool,
}

impl Default for EngineVitals {
    fn default() -> Self {
        Self {
            cpu_utilization: "n/a".to_string(),
            cpu_tone: Tone::Muted,
            memory_rss: "n/a".to_string(),
            memory_tone: Tone::Muted,
            pressure_state: "n/a".to_string(),
            pressure_tone: Tone::Muted,
            pressure_detail: None,
            stale: false,
        }
    }
}

/// Render row for a pipeline condition table.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ConditionRow {
    pub(crate) kind: String,
    pub(crate) status: String,
    pub(crate) reason: String,
    pub(crate) message: String,
    pub(crate) tone: Tone,
}

/// Render row for a pipeline core table.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct CoreRow {
    pub(crate) core: String,
    pub(crate) generation: String,
    pub(crate) phase: String,
    pub(crate) heartbeat: String,
    pub(crate) delete_pending: String,
    pub(crate) tone: Tone,
}

/// Render row for event timelines.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct TimelineRow {
    pub(crate) time: String,
    pub(crate) kind: String,
    pub(crate) scope: String,
    pub(crate) message: String,
    pub(crate) tone: Tone,
}

/// Render row for retained logs.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct LogRow {
    pub(crate) time: String,
    pub(crate) level: String,
    pub(crate) target: String,
    pub(crate) message: String,
    pub(crate) tone: Tone,
}

/// Render row for metrics tables.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct MetricRow {
    pub(crate) metric_set: String,
    pub(crate) metric: String,
    pub(crate) instrument: String,
    pub(crate) unit: String,
    pub(crate) value: String,
}

/// Render row for rollout or shutdown operation state.
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

/// Render row for diagnosis findings.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct FindingRow {
    pub(crate) severity: String,
    pub(crate) code: String,
    pub(crate) summary: String,
    pub(crate) tone: Tone,
}

/// Render row for diagnosis evidence excerpts.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct EvidenceRow {
    pub(crate) source: String,
    pub(crate) time: String,
    pub(crate) message: String,
}

/// Render row for group or engine pipeline inventories.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct PipelineInventoryRow {
    pub(crate) pipeline: String,
    pub(crate) running: String,
    pub(crate) ready: String,
    pub(crate) active_generation: String,
    pub(crate) rollout: String,
    pub(crate) tone: Tone,
}

/// Render row for engine-level readiness or liveness failures.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ProbeFailureRow {
    pub(crate) pipeline: String,
    pub(crate) condition: String,
    pub(crate) message: String,
    pub(crate) tone: Tone,
}

/// Render row for generic object detail tables.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ObjectDetailRow {
    pub(crate) field: String,
    pub(crate) value: String,
    pub(crate) detail: String,
    pub(crate) tone: Tone,
}

/// Pane state for structured object details.
#[derive(Clone, Debug, Default)]
pub(crate) struct ObjectDetailsPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) rows: Vec<ObjectDetailRow>,
    pub(crate) empty_message: String,
}

/// Pane state for the pipeline summary tab.
#[derive(Clone, Debug, Default)]
pub(crate) struct PipelineSummaryPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) conditions: Vec<ConditionRow>,
    pub(crate) cores: Vec<CoreRow>,
    pub(crate) events: Vec<TimelineRow>,
}

/// Pane state for the group summary tab.
#[derive(Clone, Debug, Default)]
pub(crate) struct GroupSummaryPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) problem_pipelines: Vec<PipelineInventoryRow>,
    pub(crate) pipelines: Vec<PipelineInventoryRow>,
    pub(crate) events: Vec<TimelineRow>,
}

/// Pane state for the engine summary tab.
#[derive(Clone, Debug, Default)]
pub(crate) struct EngineSummaryPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) pipelines: Vec<PipelineInventoryRow>,
    pub(crate) failing: Vec<ProbeFailureRow>,
}

/// Pane state for event tabs.
#[derive(Clone, Debug, Default)]
pub(crate) struct EventPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) rows: Vec<TimelineRow>,
    pub(crate) empty_message: String,
}

/// Pane state for metrics tabs.
#[derive(Clone, Debug, Default)]
pub(crate) struct MetricsPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) timestamp: Option<String>,
    pub(crate) rows: Vec<MetricRow>,
    pub(crate) empty_message: String,
}

/// Pane state for rollout and shutdown operation tabs.
#[derive(Clone, Debug, Default)]
pub(crate) struct OperationPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) rows: Vec<OperationRow>,
    pub(crate) empty_message: String,
}

/// Pane state for diagnosis tabs.
#[derive(Clone, Debug, Default)]
pub(crate) struct DiagnosisPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) summary: String,
    pub(crate) findings: Vec<FindingRow>,
    pub(crate) evidence: Vec<EvidenceRow>,
    pub(crate) next_steps: Vec<String>,
}

/// Pane state for support bundle previews.
#[derive(Clone, Debug, Default)]
pub(crate) struct BundlePane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) preview: String,
}

/// Pane state for pipeline configuration preview and edit status.
#[derive(Clone, Debug, Default)]
pub(crate) struct ConfigPane {
    pub(crate) header: Option<DetailHeader>,
    pub(crate) stats: Vec<StatCard>,
    pub(crate) note: Option<String>,
    pub(crate) preview_title: String,
    pub(crate) preview: String,
}

/// Draft pipeline configuration captured from an editor session.
#[derive(Clone, Debug)]
pub(crate) struct PipelineConfigDraft {
    pub(crate) original_yaml: String,
    pub(crate) edited_yaml: String,
    pub(crate) parsed: Option<PipelineConfig>,
    pub(crate) diff: String,
    pub(crate) error: Option<String>,
}

impl PipelineConfigDraft {
    /// Returns true when the draft parsed successfully and differs from the original.
    pub(crate) fn is_deployable(&self) -> bool {
        self.parsed.is_some() && self.edited_yaml != self.original_yaml
    }
}

/// Incremental retained-log state for one object scope.
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
    /// Resets the feed for a new scope and clears existing log rows.
    pub(crate) fn reset(&mut self, scope_key: String) {
        self.scope_key = Some(scope_key);
        self.next_seq = None;
        self.response = None;
        self.header = None;
        self.rows.clear();
        self.empty_message.clear();
    }

    /// Clears scope and retained log state.
    pub(crate) fn clear(&mut self) {
        self.scope_key = None;
        self.next_seq = None;
        self.response = None;
        self.header = None;
        self.rows.clear();
        self.empty_message.clear();
    }
}

/// Cached data backing all pipeline detail tabs.
#[derive(Clone, Debug, Default)]
pub(crate) struct PipelinePaneState {
    pub(crate) target_key: Option<String>,
    pub(crate) describe: Option<PipelineDescribeReport>,
    pub(crate) active_rollout_id: Option<String>,
    pub(crate) active_shutdown_id: Option<String>,
    pub(crate) config_yaml: Option<String>,
    pub(crate) config_draft: Option<PipelineConfigDraft>,
    pub(crate) summary: PipelineSummaryPane,
    pub(crate) details: ObjectDetailsPane,
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
    /// Resets cached pipeline data when the selected pipeline changes.
    pub(crate) fn reset(&mut self, target_key: String) {
        self.target_key = Some(target_key.clone());
        self.describe = None;
        self.active_rollout_id = None;
        self.active_shutdown_id = None;
        self.config_yaml = None;
        self.config_draft = None;
        self.summary = PipelineSummaryPane::default();
        self.details = ObjectDetailsPane::default();
        self.config = ConfigPane::default();
        self.events = EventPane::default();
        self.logs.reset(format!("pipeline:{target_key}"));
        self.metrics = MetricsPane::default();
        self.rollout = OperationPane::default();
        self.shutdown = OperationPane::default();
        self.diagnosis = DiagnosisPane::default();
        self.bundle = BundlePane::default();
    }

    /// Clears all cached pipeline data when no pipeline is selected.
    pub(crate) fn clear(&mut self) {
        self.target_key = None;
        self.describe = None;
        self.active_rollout_id = None;
        self.active_shutdown_id = None;
        self.config_yaml = None;
        self.config_draft = None;
        self.summary = PipelineSummaryPane::default();
        self.details = ObjectDetailsPane::default();
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

/// Cached data backing all group detail tabs.
#[derive(Clone, Debug, Default)]
pub(crate) struct GroupPaneState {
    pub(crate) group_id: Option<String>,
    pub(crate) active_shutdown: Option<ActiveGroupShutdown>,
    pub(crate) summary: GroupSummaryPane,
    pub(crate) details: ObjectDetailsPane,
    pub(crate) events: EventPane,
    pub(crate) logs: LogFeedState,
    pub(crate) metrics: MetricsPane,
    pub(crate) shutdown: GroupShutdownPane,
    pub(crate) diagnosis: DiagnosisPane,
    pub(crate) bundle: BundlePane,
}

impl GroupPaneState {
    /// Resets cached group data when the selected group changes.
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
        self.details = ObjectDetailsPane::default();
        self.events = EventPane::default();
        self.logs.reset(format!("group:{group_id}"));
        self.metrics = MetricsPane::default();
        self.shutdown = GroupShutdownPane::default();
        self.diagnosis = DiagnosisPane::default();
        self.bundle = BundlePane::default();
    }

    /// Clears all cached group data when no group is selected.
    pub(crate) fn clear(&mut self) {
        self.group_id = None;
        self.active_shutdown = None;
        self.summary = GroupSummaryPane::default();
        self.details = ObjectDetailsPane::default();
        self.events = EventPane::default();
        self.logs.clear();
        self.metrics = MetricsPane::default();
        self.shutdown = GroupShutdownPane::default();
        self.diagnosis = DiagnosisPane::default();
        self.bundle = BundlePane::default();
    }
}

/// Cached data backing all engine detail tabs.
#[derive(Clone, Debug, Default)]
pub(crate) struct EnginePaneState {
    pub(crate) summary: EngineSummaryPane,
    pub(crate) details: ObjectDetailsPane,
    pub(crate) logs: LogFeedState,
    pub(crate) metrics: MetricsPane,
}

impl EnginePaneState {
    /// Creates engine pane state with an initialized engine-scoped log feed.
    pub(crate) fn new() -> Self {
        let mut state = Self::default();
        state.logs.reset("engine".to_string());
        state
    }
}

/// Selectable row in the pipeline list.
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

/// Selectable row in the group list.
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

/// Selectable row in the engine pipeline list.
#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnginePipelineItem {
    pub(crate) key: String,
    pub(crate) status_badge: String,
    pub(crate) tone: Tone,
    pub(crate) running: String,
    pub(crate) active_generation: String,
    pub(crate) rollout: String,
}

/// Small animated activity marker state for the TUI title bar.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct ActivityIndicator {
    pub(crate) active: bool,
    pub(crate) frame: u16,
}

/// Root mutable state for one interactive TUI session.
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
    pub(crate) terminal_size: Option<(u16, u16)>,
    pub(crate) last_refresh: Option<Instant>,
    pub(crate) last_error: Option<String>,
    pub(crate) groups_status: Option<groups::Status>,
    pub(crate) engine_status: Option<engine::Status>,
    pub(crate) engine_livez: Option<engine::ProbeResponse>,
    pub(crate) engine_readyz: Option<engine::ProbeResponse>,
    pub(crate) engine_vitals: EngineVitals,
    pub(crate) command_context: UiCommandContext,
    pub(crate) activity_indicator: ActivityIndicator,
    pub(crate) pipelines: PipelinePaneState,
    pub(crate) groups: GroupPaneState,
    pub(crate) engine: EnginePaneState,
}
