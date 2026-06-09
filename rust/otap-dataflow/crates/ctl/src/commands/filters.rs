// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! CLI-to-troubleshoot filter translation helpers.
//!
//! The command layer accepts user-oriented filter flags while the
//! troubleshooting layer works with normalized filter structs. This module is
//! the small conversion boundary that keeps filtering behavior shared between
//! group events, pipeline events, logs, and metrics commands.

use crate::args::{EventKind, LogsFilterArgs, MetricsFilterArgs};
use crate::troubleshoot::{EventFilters, LogFilters, MetricsFilters, NormalizedEventKind};

/// Build group event filters directly from CLI arguments.
pub(crate) fn group_event_filters(
    kinds: Vec<EventKind>,
    pipeline_group_id: Option<String>,
    pipeline_id: Option<String>,
    node_id: Option<String>,
    contains: Option<String>,
) -> EventFilters {
    EventFilters {
        kinds: map_event_kinds(kinds),
        pipeline_group_id,
        pipeline_id,
        node_id,
        contains,
    }
}

/// Build pipeline event filters directly from CLI arguments.
pub(crate) fn pipeline_event_filters(
    kinds: Vec<EventKind>,
    node_id: Option<String>,
    contains: Option<String>,
) -> EventFilters {
    EventFilters {
        kinds: map_event_kinds(kinds),
        pipeline_group_id: None,
        pipeline_id: None,
        node_id,
        contains,
    }
}

/// Convert log filter CLI arguments into the shared troubleshoot filter model.
pub(crate) fn log_filters_from_args(args: LogsFilterArgs) -> LogFilters {
    LogFilters {
        level: args.level,
        target: args.target,
        event: args.event,
        pipeline_group_id: args.pipeline_group_id,
        pipeline_id: args.pipeline_id,
        node_id: args.node_id,
        contains: args.contains,
    }
}

/// Convert metrics filter CLI arguments into the shared troubleshoot filter model.
pub(crate) fn metrics_filters_from_args(args: MetricsFilterArgs) -> MetricsFilters {
    MetricsFilters {
        metric_sets: args.metric_sets,
        metric_names: args.metric_names,
        pipeline_group_id: args.pipeline_group_id,
        pipeline_id: args.pipeline_id,
        core_id: args.core_id,
        node_id: args.node_id,
    }
}

/// Build scoped log and metrics filters for a single pipeline target.
pub(crate) fn pipeline_scope_filters(
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> (LogFilters, MetricsFilters) {
    (
        LogFilters {
            pipeline_group_id: Some(pipeline_group_id.to_string()),
            pipeline_id: Some(pipeline_id.to_string()),
            ..LogFilters::default()
        },
        MetricsFilters {
            pipeline_group_id: Some(pipeline_group_id.to_string()),
            pipeline_id: Some(pipeline_id.to_string()),
            ..MetricsFilters::default()
        },
    )
}

fn map_event_kinds(kinds: Vec<EventKind>) -> Vec<NormalizedEventKind> {
    kinds
        .into_iter()
        .map(|kind| match kind {
            EventKind::Request => NormalizedEventKind::Request,
            EventKind::Success => NormalizedEventKind::Success,
            EventKind::Error => NormalizedEventKind::Error,
            EventKind::Log => NormalizedEventKind::Log,
        })
        .collect()
}
