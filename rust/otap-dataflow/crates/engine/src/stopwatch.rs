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
use std::collections::{HashMap, HashSet};

use otap_df_telemetry::instrument::Mmsc;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::{attribute_set, metric_set};

use crate::Interests;
use crate::attributes::PipelineAttributeSet;
use crate::context::PipelineContext;
use otap_df_config::policy::TelemetryPolicy;

/// Metric set for a single stopwatch.
#[metric_set(name = "stopwatch")]
#[derive(Debug, Default, Clone)]
pub struct StopwatchMetrics {
    /// Sum of per-node synchronous compute durations (nanoseconds)
    /// for messages traversing the stopwatch range.
    #[metric(name = "stopwatch.compute.duration", unit = "ns")]
    pub compute_duration: Mmsc,
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
/// Holds the start/stop node lookup tables used during processor wiring.
/// Metric sets are cloned to the local processor `EffectHandler` at build
/// time; reporting happens from the processor's own telemetry path, not
/// from this state.
pub(crate) struct StopwatchState {
    /// Metric sets indexed by internal stopwatch index.
    /// Cloned to processors at build time; not reported from here.
    pub metrics: Vec<MetricSet<StopwatchMetrics>>,
    /// Mapping from node index → stopwatch metric index where this node is
    /// the **stop** node (recording happens here on the forward path).
    pub stop_nodes: HashMap<usize, usize>,
    /// Set of node indices that are **start** nodes.
    pub start_nodes: HashSet<usize>,
}

/// Build stopwatch state from the telemetry policy configuration.
///
/// Resolves stopwatch start/stop node names to node indices, validates
/// that endpoints are processor nodes and ranges don't overlap, registers
/// metric entities, and builds the lookup tables used for processor wiring.
///
/// Stopwatches require the per-node `PROCESS_DURATION` interest to measure
/// elapsed time. If `node_interests` does not include `PROCESS_DURATION`
/// (e.g. `runtime_metrics: basic` or `none`), all configured stopwatches
/// are skipped with a single warning so users see a clear signal rather
/// than silent zero-duration metrics.
pub(crate) fn build_stopwatch_state(
    telemetry_policy: &TelemetryPolicy,
    node_name_to_index: &HashMap<String, usize>,
    processor_indices: &HashSet<usize>,
    node_interests: Interests,
    pipeline_context: &PipelineContext,
) -> StopwatchState {
    let mut metrics = Vec::new();
    let mut stop_nodes: HashMap<usize, usize> = HashMap::new();
    let mut start_nodes: HashSet<usize> = HashSet::new();

    if !telemetry_policy.stopwatches.is_empty()
        && !node_interests.contains(Interests::PROCESS_DURATION)
    {
        otap_df_telemetry::otel_warn!(
            "stopwatch.config.metric_level_disabled",
            count = telemetry_policy.stopwatches.len() as u64,
            "Stopwatches require runtime_metrics level that enables PROCESS_DURATION (normal or detailed); skipping all configured stopwatches"
        );
        return StopwatchState {
            metrics,
            stop_nodes,
            start_nodes,
        };
    }

    let pipeline_attrs = pipeline_context.pipeline_attribute_set();

    for sw_config in &telemetry_policy.stopwatches {
        let start_idx = node_name_to_index.get(&sw_config.start_node).copied();
        let stop_idx = node_name_to_index.get(&sw_config.stop_node).copied();

        let (Some(start_idx), Some(stop_idx)) = (start_idx, stop_idx) else {
            otap_df_telemetry::otel_warn!(
                "stopwatch.config.invalid",
                name = sw_config.name,
                start = sw_config.start_node,
                stop = sw_config.stop_node,
                "Stopwatch references unknown node(s), skipping"
            );
            continue;
        };

        // Stopwatches only work on local processors (they own the Cell<Mmsc>
        // accumulator).  Warn and skip if either endpoint is a receiver or
        // exporter.
        if !processor_indices.contains(&start_idx) || !processor_indices.contains(&stop_idx) {
            otap_df_telemetry::otel_warn!(
                "stopwatch.config.non_processor",
                name = sw_config.name,
                start = sw_config.start_node,
                stop = sw_config.stop_node,
                "Stopwatch start/stop nodes must be processors, skipping"
            );
            continue;
        }

        // Non-overlapping: a node cannot appear in either role across
        // stopwatches. The second definition would silently overwrite the
        // first accumulator on PData.
        if start_nodes.contains(&start_idx)
            || stop_nodes.contains_key(&start_idx)
            || start_nodes.contains(&stop_idx)
            || stop_nodes.contains_key(&stop_idx)
        {
            otap_df_telemetry::otel_warn!(
                "stopwatch.config.overlap",
                name = sw_config.name,
                start = sw_config.start_node,
                stop = sw_config.stop_node,
                "Stopwatch overlaps with another stopwatch (non-overlapping ranges only), skipping"
            );
            continue;
        }

        let attrs = StopwatchAttributeSet {
            stopwatch_name: Cow::Owned(sw_config.name.clone()),
            start_node: Cow::Owned(sw_config.start_node.clone()),
            stop_node: Cow::Owned(sw_config.stop_node.clone()),
            pipeline_attrs: pipeline_attrs.clone(),
        };

        let entity_key = pipeline_context.metrics_registry().register_entity(attrs);
        let metric_set = pipeline_context
            .metrics_registry()
            .register_metric_set_for_entity::<StopwatchMetrics>(entity_key);

        let id = metrics.len();
        metrics.push(metric_set);
        let _ = stop_nodes.insert(stop_idx, id);
        let _ = start_nodes.insert(start_idx);
    }

    StopwatchState {
        metrics,
        stop_nodes,
        start_nodes,
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
            start_nodes: HashSet::new(),
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
            stop_nodes: HashMap::from([(2, 0)]),
            start_nodes: HashSet::from([0]),
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
        state.metrics[0].compute_duration.record(100.0);
        state.metrics[0].compute_duration.record(200.0);

        let snap = state.metrics[0].compute_duration.get();
        assert_eq!(snap.count, 2);
        assert!((snap.min - 100.0).abs() < f64::EPSILON);
        assert!((snap.max - 200.0).abs() < f64::EPSILON);
        assert!((snap.sum - 300.0).abs() < f64::EPSILON);
    }

    // -- build_stopwatch_state validation tests --

    use otap_df_config::policy::{StopwatchConfig, TelemetryPolicy};

    fn policy_with(stopwatches: Vec<StopwatchConfig>) -> TelemetryPolicy {
        TelemetryPolicy {
            stopwatches,
            ..TelemetryPolicy::default()
        }
    }

    fn sw(name: &str, start: &str, stop: &str) -> StopwatchConfig {
        StopwatchConfig {
            name: name.to_string(),
            start_node: start.to_string(),
            stop_node: stop.to_string(),
        }
    }

    /// Helper: build name→index and processor_indices for a set of node
    /// names.  All nodes are processors unless listed in `non_processors`.
    fn test_maps(
        all_nodes: &[&str],
        non_processors: &[&str],
    ) -> (HashMap<String, usize>, HashSet<usize>) {
        let name_to_index: HashMap<String, usize> = all_nodes
            .iter()
            .enumerate()
            .map(|(i, &n)| (n.to_string(), i))
            .collect();
        let processor_indices: HashSet<usize> = all_nodes
            .iter()
            .enumerate()
            .filter(|&(_, &n)| !non_processors.contains(&n))
            .map(|(i, _)| i)
            .collect();
        (name_to_index, processor_indices)
    }

    #[test]
    fn valid_stopwatch_is_registered() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "c")]);

        let state =
            build_stopwatch_state(&policy, &names, &procs, Interests::PROCESS_DURATION, &ctx);

        assert_eq!(state.metrics.len(), 1);
        assert!(state.start_nodes.contains(&0)); // "a" = index 0
        assert_eq!(state.stop_nodes.get(&2), Some(&0)); // "c" = index 2
    }

    #[test]
    fn unknown_node_is_skipped() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "missing")]);

        let state =
            build_stopwatch_state(&policy, &names, &procs, Interests::PROCESS_DURATION, &ctx);

        assert!(state.metrics.is_empty());
        assert!(state.start_nodes.is_empty());
        assert!(state.stop_nodes.is_empty());
    }

    #[test]
    fn non_processor_start_node_is_skipped() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["recv", "proc1", "proc2"], &["recv"]);
        let policy = policy_with(vec![sw("sw1", "recv", "proc2")]);

        let state =
            build_stopwatch_state(&policy, &names, &procs, Interests::PROCESS_DURATION, &ctx);

        assert!(state.metrics.is_empty());
    }

    #[test]
    fn non_processor_stop_node_is_skipped() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["proc1", "proc2", "exp"], &["exp"]);
        let policy = policy_with(vec![sw("sw1", "proc1", "exp")]);

        let state =
            build_stopwatch_state(&policy, &names, &procs, Interests::PROCESS_DURATION, &ctx);

        assert!(state.metrics.is_empty());
    }

    #[test]
    fn shared_start_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        // Two stopwatches share "a" as start node.
        let policy = policy_with(vec![sw("sw1", "a", "b"), sw("sw2", "a", "d")]);

        let state =
            build_stopwatch_state(&policy, &names, &procs, Interests::PROCESS_DURATION, &ctx);

        // Only the first should be registered.
        assert_eq!(state.metrics.len(), 1);
        assert!(state.start_nodes.contains(&0)); // "a"
        assert_eq!(state.stop_nodes.get(&1), Some(&0)); // "b"
        assert!(!state.stop_nodes.contains_key(&3)); // "d" not registered
    }

    #[test]
    fn shared_stop_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        // Two stopwatches share "d" as stop node.
        let policy = policy_with(vec![sw("sw1", "a", "d"), sw("sw2", "c", "d")]);

        let state =
            build_stopwatch_state(&policy, &names, &procs, Interests::PROCESS_DURATION, &ctx);

        // Only the first should be registered.
        assert_eq!(state.metrics.len(), 1);
        assert!(state.start_nodes.contains(&0)); // "a"
        assert!(!state.start_nodes.contains(&2)); // "c" not registered
    }

    #[test]
    fn stop_of_one_is_start_of_another_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "b"), sw("sw2", "b", "c")]);

        let state =
            build_stopwatch_state(&policy, &names, &procs, Interests::PROCESS_DURATION, &ctx);

        assert_eq!(state.metrics.len(), 1);
        assert!(state.start_nodes.contains(&0));
        assert_eq!(state.stop_nodes.get(&1), Some(&0));
        assert!(!state.start_nodes.contains(&2));
        assert!(!state.stop_nodes.contains_key(&2));
    }

    #[test]
    fn disjoint_stopwatches_are_both_registered() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        // Non-overlapping: a→b and c→d.
        let policy = policy_with(vec![sw("sw1", "a", "b"), sw("sw2", "c", "d")]);

        let state =
            build_stopwatch_state(&policy, &names, &procs, Interests::PROCESS_DURATION, &ctx);

        assert_eq!(state.metrics.len(), 2);
        assert!(state.start_nodes.contains(&0)); // "a"
        assert!(state.start_nodes.contains(&2)); // "c"
        assert_eq!(state.stop_nodes.get(&1), Some(&0)); // "b" → sw1
        assert_eq!(state.stop_nodes.get(&3), Some(&1)); // "d" → sw2
    }

    #[test]
    fn stopwatches_skipped_when_process_duration_disabled() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "b")]);

        // Empty interests => PROCESS_DURATION not set; all stopwatches skipped.
        let state = build_stopwatch_state(&policy, &names, &procs, Interests::empty(), &ctx);

        assert!(state.metrics.is_empty());
        assert!(state.start_nodes.is_empty());
        assert!(state.stop_nodes.is_empty());
    }
}
