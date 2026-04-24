// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Distributed stopwatch that sums synchronous compute duration across a
//! contiguous range of processor nodes.
//!
//! Stopwatches are declared in the pipeline telemetry policy YAML and
//! managed entirely by the engine.  On the forward path, each processor's
//! `timed()` elapsed is accumulated onto the PData's `stopwatch_compute_ns`
//! field.  The stop node takes the total and records it into the stopwatch
//! metric entity.

use std::borrow::Cow;
use std::collections::HashMap;

use otap_df_telemetry::instrument::Mmsc;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::{attribute_set, metric_set};

use crate::attributes::PipelineAttributeSet;

/// Metric set for a single stopwatch.
#[metric_set(name = "stopwatch")]
#[derive(Debug, Default, Clone)]
pub struct StopwatchMetrics {
    /// Sum of per-node synchronous compute durations (nanoseconds)
    /// for messages successfully traversing the stopwatch range.
    #[metric(name = "stopwatch.compute.success.duration", unit = "ns")]
    pub compute_duration_success: Mmsc,
}

/// Entity attributes that scope a stopwatch metric set.
#[attribute_set(name = "stopwatch.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct StopwatchAttributeSet {
    /// User-given stopwatch name.
    #[attribute(key = "stopwatch.name")]
    pub stopwatch_name: Cow<'static, str>,
    /// Name of the processor node where the measurement begins.
    #[attribute(key = "stopwatch.start_node")]
    pub start_node: Cow<'static, str>,
    /// Name of the processor node where the measurement ends.
    #[attribute(key = "stopwatch.stop_node")]
    pub stop_node: Cow<'static, str>,
    /// Pipeline attributes.
    #[compose]
    pub pipeline_attrs: PipelineAttributeSet,
}

/// Index of a stopwatch within the pipeline's stopwatch table.
pub type StopwatchId = usize;

/// Per-pipeline stopwatch state.
///
/// Holds the metric sets and the start/stop node lookup tables.
/// Shared between the `EffectHandlerCore` (for recording at stop nodes)
/// and the `PipelineCompletionMsgDispatcher` (for periodic reporting).
pub(crate) struct StopwatchState {
    /// Metric sets indexed by `StopwatchId`.
    pub metrics: Vec<MetricSet<StopwatchMetrics>>,
    /// Mapping from node index → list of stopwatch IDs where this node is
    /// the **stop** node (recording happens here on the forward path).
    pub stop_nodes: HashMap<usize, Vec<StopwatchId>>,
    /// Mapping from node index → list of stopwatch IDs where this node is
    /// the **start** node (accumulator is initialised here on the forward path).
    pub start_nodes: HashMap<usize, Vec<StopwatchId>>,
}

impl StopwatchState {
    /// Report all stopwatch metric sets through the given reporter so that
    /// the telemetry dispatcher can pick them up.
    pub fn report(&mut self, reporter: &mut otap_df_telemetry::reporter::MetricsReporter) {
        for m in &mut self.metrics {
            let _ = reporter.report(m);
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
impl StopwatchState {
    /// Create an empty stopwatch state (no stopwatches configured).
    #[must_use]
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            metrics: Vec::new(),
            stop_nodes: HashMap::new(),
            start_nodes: HashMap::new(),
        }
    }

    /// Returns `true` if any stopwatches are configured.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        !self.metrics.is_empty()
    }
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use super::*;
    use crate::testing::test_pipeline_ctx;

    fn one_stopwatch_state() -> StopwatchState {
        let (ctx, _) = test_pipeline_ctx();
        let entity_key = ctx
            .metrics_registry()
            .register_entity(StopwatchAttributeSet::default());
        let metric_set = ctx
            .metrics_registry()
            .register_metric_set_for_entity::<StopwatchMetrics>(entity_key);
        StopwatchState {
            metrics: vec![metric_set],
            stop_nodes: HashMap::from([(2, vec![0])]),
            start_nodes: HashMap::from([(0, vec![0])]),
        }
    }

    #[test]
    fn empty_state_is_inactive() {
        let state = StopwatchState::empty();
        assert!(!state.is_active());
    }

    #[test]
    fn nonempty_state_is_active() {
        let state = one_stopwatch_state();
        assert!(state.is_active());
    }

    #[test]
    fn direct_record_increments_mmsc() {
        let mut state = one_stopwatch_state();
        state.metrics[0].compute_duration_success.record(100.0);
        state.metrics[0].compute_duration_success.record(200.0);

        let snap = state.metrics[0].compute_duration_success.get();
        assert_eq!(snap.count, 2);
        assert!((snap.min - 100.0).abs() < f64::EPSILON);
        assert!((snap.max - 200.0).abs() < f64::EPSILON);
        assert!((snap.sum - 300.0).abs() < f64::EPSILON);
    }
}
