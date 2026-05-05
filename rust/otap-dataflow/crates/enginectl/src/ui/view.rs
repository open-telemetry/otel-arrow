// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! TUI layout, hit-testing, theming, and rendering helpers.
//!
//! The view module is deliberately limited to drawing render-ready state and
//! resolving mouse geometry. Its submodules split top-level chrome, pane
//! drawing, table widgets, modal overlays, layout calculation, and theme
//! constants so visual changes do not leak back into refresh, input, or
//! mutation code.

#[path = "view/chrome.rs"]
mod chrome;
#[path = "view/layout.rs"]
mod layout;
#[path = "view/overlays.rs"]
mod overlays;
#[path = "view/panes.rs"]
mod panes;
#[path = "view/tables.rs"]
mod tables;
#[path = "view/theme.rs"]
mod theme;

pub(crate) use self::chrome::draw_ui;
#[cfg(test)]
pub(crate) use self::layout::tab_regions;
pub(crate) use self::layout::{compute_ui_layout, state_table_row_hit_index, tab_hit_index};

use super::app::{
    ActionMenuState, AppState, BundlePane, CommandPaletteState, ConditionRow, ConfigPane, CoreRow,
    DetailHeader, DiagnosisPane, EngineSummaryPane, EngineTab, EngineVitals, EventPane,
    EvidenceRow, FindingRow, FocusArea, GroupShutdownPane, GroupShutdownRow, GroupSummaryPane,
    GroupTab, LogFeedState, LogRow, MetricRow, MetricsPane, ObjectDetailRow, ObjectDetailsPane,
    OperationPane, OperationRow, PipelineInventoryRow, PipelineSummaryPane, PipelineTab,
    ProbeFailureRow, ScaleEditorState, ShutdownConfirmState, StatCard, StatusChip, TimelineRow,
    Tone, View,
};
use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Direction, Flex, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Cell, Clear, Paragraph, Row, Table, TableState, Wrap},
};

use self::layout::*;
use self::overlays::*;
use self::panes::*;
use self::tables::*;
use self::theme::*;

#[cfg(test)]
#[path = "view/tests.rs"]
mod tests;
