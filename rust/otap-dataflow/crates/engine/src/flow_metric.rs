// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flow metrics that sum per-message compute duration and signal counts
//! across a contiguous range of processor nodes.
//!
//! Flow metrics are declared in the pipeline telemetry policy YAML and
//! managed entirely by the engine. On the forward path, each processor's
//! per-message compute time is measured by the EffectHandler's
//! `Instant`-based send marker (advanced by `take_elapsed_since_send_marker_ns`
//! at each `send_message` call) and accumulated onto the PData's
//! `flow_compute_ns` field. The start and end nodes record item counts, and
//! the end node records the total compute duration into the flow metric entity.

use std::borrow::Cow;
use std::cell::Cell;
use std::collections::{HashMap, HashSet};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use otap_df_telemetry::instrument::Mmsc;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::{attribute_set, metric_set};

use crate::attributes::PipelineAttributeSet;
use crate::context::PipelineContext;
use otap_df_config::policy::{FlowMetric, TelemetryPolicy};

/// Metric set emitted by the start node of a flow range.
#[metric_set(name = "flow")]
#[derive(Debug, Default, Clone)]
pub struct FlowSignalsIncomingMetrics {
    /// Number of signal items (log records, spans, or metric data points)
    /// entering the flow range.
    #[metric(name = "flow.signals.incoming", unit = "{item}")]
    pub signals_incoming: Mmsc,
}

/// Duration metric set emitted by the end node of a flow range.
#[metric_set(name = "flow")]
#[derive(Debug, Default, Clone)]
pub struct FlowDurationMetrics {
    /// Sum of per-node compute durations (nanoseconds) for messages traversing the flow range.
    #[metric(name = "flow.compute.duration", unit = "ns")]
    pub compute_duration: Mmsc,
}

/// Outgoing signal metric set emitted by the end node of a flow range.
#[metric_set(name = "flow")]
#[derive(Debug, Default, Clone)]
pub struct FlowSignalsOutgoingMetrics {
    /// Number of signal items (log records, spans, or metric data points) leaving the flow range.
    #[metric(name = "flow.signals.outgoing", unit = "{item}")]
    pub signals_outgoing: Mmsc,
}

/// Entity attributes that scope a flow metric set.
#[attribute_set(name = "flow.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct FlowAttributeSet {
    /// User-given flow name.
    #[attribute(key = "flow.name")]
    pub flow_name: Cow<'static, str>,
    /// Name of the processor node where the measurement begins.
    #[attribute(key = "flow.start_node")]
    pub start_node: Cow<'static, str>,
    /// Name of the processor node where the measurement ends.
    #[attribute(key = "flow.end_node")]
    pub end_node: Cow<'static, str>,
    /// Pipeline attributes.
    #[compose]
    pub pipeline_attrs: PipelineAttributeSet,
}

/// Index of a flow_metric within the pipeline's flow_metric table.
pub type FlowMetricId = usize;

/// Per-pipeline flow_metric state.
///
/// Holds the start/stop node lookup tables used during processor wiring.
/// Metric sets are cloned to the processor `EffectHandler` (local or
/// shared) at build time; reporting happens from the processor's own
/// telemetry path, not from this state.
pub(crate) struct PipelineFlowMetricState {
    /// Incoming signal metric sets indexed by internal flow index.
    pub signals_incoming_metrics: Vec<Option<MetricSet<FlowSignalsIncomingMetrics>>>,
    /// Duration metric sets indexed by internal flow index.
    pub duration_metrics: Vec<Option<MetricSet<FlowDurationMetrics>>>,
    /// Outgoing signal metric sets indexed by internal flow index.
    pub signals_outgoing_metrics: Vec<Option<MetricSet<FlowSignalsOutgoingMetrics>>>,
    /// Mapping from node index → flow metric index where this node is the end node.
    pub end_nodes: HashMap<usize, usize>,
    /// Mapping from node index → flow metric index where this node is the start node.
    pub start_nodes: HashMap<usize, usize>,
}

/// Start-side measurements for a node that begins a flow_metric range.
///
/// Groups the metric set and its accumulator so that they share the
/// `Option` on `FlowMetricState` — non-start nodes pay no allocation for
/// either.
#[derive(Clone)]
pub(crate) struct IncomingFlowMetrics<Accumulator> {
    /// Incoming signal measurement, if enabled for this flow.
    pub signals_incoming: Option<(MetricSet<FlowSignalsIncomingMetrics>, Accumulator)>,
}

/// Stop-side measurements for a node that terminates a flow_metric range.
///
/// Groups the metric set and its accumulators so that they share the
/// `Option` on `FlowMetricState` — non-stop nodes pay no allocation for
/// either.
#[derive(Clone)]
pub(crate) struct EndFlowMetrics<Accumulator> {
    /// Duration measurement, if enabled for this flow.
    pub duration: Option<(MetricSet<FlowDurationMetrics>, Accumulator)>,
    /// Outgoing signal measurement, if enabled for this flow.
    pub signals_outgoing: Option<(MetricSet<FlowSignalsOutgoingMetrics>, Accumulator)>,
}

/// Per-`EffectHandler` flow_metric state.
///
/// Groups the flow_metric-related fields that every processor
/// `EffectHandler` carries. Generic over the cell types used to store
/// the per-message send marker and the periodic-report accumulator —
/// the local (`!Send`) handler instantiates it with `Rc<Cell<_>>` /
/// `Cell<_>`, the shared (`Send + Sync`) handler with
/// `Arc<Mutex<_>>` / `Arc<Mutex<_>>`. The plain fields
/// (`is_start`, `active`) are identical in both instantiations and
/// live here once. Start- and stop-side state lives in
/// [`IncomingFlowMetrics`] and [`EndFlowMetrics`] and is `None` for nodes
/// that are not the corresponding endpoint of a flow_metric.
///
/// All fields are `pub(crate)` so the local/shared `EffectHandler`
/// methods can read and write them directly through the
/// `flow_metric.<field>` access path.
#[derive(Clone)]
pub(crate) struct FlowMetricState<Marker, Accumulator> {
    /// Marker for the most recent timing point on the current message's
    /// path through `process()`. Armed by `begin_process_timing` before
    /// each PData `process()` call and advanced by
    /// `take_elapsed_since_send_marker_ns` on each send. Stays `None`
    /// when flow_metrics are inactive on this pipeline; otherwise stays
    /// `Some(_)` once armed.
    pub last_send_marker: Marker,
    /// Whether this node is a flow metric start node.
    pub is_start: bool,
    /// Whether this node is a flow metric end node.
    pub is_end: bool,
    /// Whether any flow_metric is configured in this pipeline.
    /// When false, `begin_process_timing` and
    /// `take_elapsed_since_send_marker_ns` are no-ops.
    pub active: bool,
    /// Start-side measurements (metric set + accumulator).
    /// `None` when this node is not a flow_metric start node — non-start
    /// nodes pay no allocation cost for the metric set or accumulator.
    pub incoming: IncomingFlowMetrics<Accumulator>,
    /// End-side measurements.
    pub end: EndFlowMetrics<Accumulator>,
}

/// Concrete `FlowMetricState` for the local (`!Send`) `EffectHandler`.
pub(crate) type LocalFlowMetricState = FlowMetricState<Rc<Cell<Option<Instant>>>, Cell<Mmsc>>;

/// Concrete `FlowMetricState` for the shared (`Send + Sync`)
/// `EffectHandler`.
pub(crate) type SharedFlowMetricState =
    FlowMetricState<Arc<Mutex<Option<Instant>>>, Arc<Mutex<Mmsc>>>;

impl Default for LocalFlowMetricState {
    fn default() -> Self {
        Self {
            last_send_marker: Rc::new(Cell::new(None)),
            is_start: false,
            is_end: false,
            active: false,
            incoming: IncomingFlowMetrics {
                signals_incoming: None,
            },
            end: EndFlowMetrics {
                duration: None,
                signals_outgoing: None,
            },
        }
    }
}

impl Default for SharedFlowMetricState {
    fn default() -> Self {
        Self {
            last_send_marker: Arc::new(Mutex::new(None)),
            is_start: false,
            is_end: false,
            active: false,
            incoming: IncomingFlowMetrics {
                signals_incoming: None,
            },
            end: EndFlowMetrics {
                duration: None,
                signals_outgoing: None,
            },
        }
    }
}

/// Build flow_metric state from the telemetry policy configuration.
///
/// Resolves flow_metric start/stop node names to node indices, validates
/// that endpoints are processor nodes (local or shared; receivers and
/// exporters are rejected) and ranges don't overlap, registers metric
/// entities, and builds the lookup tables used for processor wiring.
///
/// FlowMetrics are explicit opt-in via the telemetry policy YAML and require
/// no separate metric-level gating. When configured, the engine wires both
/// item-count and duration accumulation for the declared range.
///
/// Returns `Err(Error::ConfigError(InvalidUserConfig))` if any flow_metric
/// references an unknown node, has a non-processor endpoint, or overlaps
/// with another flow_metric — these are configuration mistakes that should
/// fail pipeline startup rather than be silently skipped.
pub(crate) fn build_flow_metric_state(
    telemetry_policy: &TelemetryPolicy,
    node_name_to_index: &HashMap<String, usize>,
    processor_indices: &HashSet<usize>,
    pipeline_context: &PipelineContext,
) -> Result<PipelineFlowMetricState, crate::error::Error> {
    let mut signals_incoming_metrics = Vec::new();
    let mut duration_metrics = Vec::new();
    let mut signals_outgoing_metrics = Vec::new();
    let mut end_nodes: HashMap<usize, usize> = HashMap::new();
    let mut start_nodes: HashMap<usize, usize> = HashMap::new();

    let pipeline_attrs = pipeline_context.pipeline_attribute_set();

    for flow_config in &telemetry_policy.flow_metrics {
        let start_idx = node_name_to_index
            .get(&flow_config.bounds.start_node)
            .copied();
        let end_idx = node_name_to_index
            .get(&flow_config.bounds.end_node)
            .copied();

        let (Some(start_idx), Some(end_idx)) = (start_idx, end_idx) else {
            return Err(invalid_flow_metric_config(format!(
                "flow metric `{}` references unknown node(s): start=`{}`, end=`{}`",
                flow_config.name, flow_config.bounds.start_node, flow_config.bounds.end_node
            )));
        };

        if !processor_indices.contains(&start_idx) || !processor_indices.contains(&end_idx) {
            return Err(invalid_flow_metric_config(format!(
                "flow metric `{}` start/end nodes must be processors: start=`{}`, end=`{}`",
                flow_config.name, flow_config.bounds.start_node, flow_config.bounds.end_node
            )));
        }

        if start_nodes.contains_key(&start_idx)
            || end_nodes.contains_key(&start_idx)
            || start_nodes.contains_key(&end_idx)
            || end_nodes.contains_key(&end_idx)
        {
            return Err(invalid_flow_metric_config(format!(
                "flow metric `{}` overlaps with another flow metric (non-overlapping ranges only): start=`{}`, end=`{}`",
                flow_config.name, flow_config.bounds.start_node, flow_config.bounds.end_node
            )));
        }

        let attrs = FlowAttributeSet {
            flow_name: Cow::Owned(flow_config.name.clone()),
            start_node: Cow::Owned(flow_config.bounds.start_node.clone()),
            end_node: Cow::Owned(flow_config.bounds.end_node.clone()),
            pipeline_attrs: pipeline_attrs.clone(),
        };

        let entity_key = pipeline_context.metrics_registry().register_entity(attrs);
        let signals_incoming_metric = flow_config.has(FlowMetric::SignalsIncoming).then(|| {
            pipeline_context
                .metrics_registry()
                .register_metric_set_for_entity::<FlowSignalsIncomingMetrics>(entity_key)
        });
        let duration_metric = flow_config.has(FlowMetric::ComputeDuration).then(|| {
            pipeline_context
                .metrics_registry()
                .register_metric_set_for_entity::<FlowDurationMetrics>(entity_key)
        });
        let signals_outgoing_metric = flow_config.has(FlowMetric::SignalsOutgoing).then(|| {
            pipeline_context
                .metrics_registry()
                .register_metric_set_for_entity::<FlowSignalsOutgoingMetrics>(entity_key)
        });

        let id = duration_metrics.len();
        signals_incoming_metrics.push(signals_incoming_metric);
        duration_metrics.push(duration_metric);
        signals_outgoing_metrics.push(signals_outgoing_metric);
        let _ = end_nodes.insert(end_idx, id);
        let _ = start_nodes.insert(start_idx, id);
    }

    Ok(PipelineFlowMetricState {
        signals_incoming_metrics,
        duration_metrics,
        signals_outgoing_metrics,
        end_nodes,
        start_nodes,
    })
}

fn invalid_flow_metric_config(error: String) -> crate::error::Error {
    crate::error::Error::ConfigError(Box::new(otap_df_config::error::Error::InvalidUserConfig {
        error,
    }))
}

/// Saturating cast of `u128` nanoseconds to `u64`.
///
/// Used by EffectHandlers when computing elapsed durations between
/// `Instant`s for flow_metric accumulation.
#[inline]
#[must_use]
pub fn nanos_u64(ns: u128) -> u64 {
    ns.min(u128::from(u64::MAX)) as u64
}

impl PipelineFlowMetricState {
    /// Create an empty flow_metric state (no flow_metrics configured).
    #[must_use]
    #[allow(dead_code)]
    pub fn empty() -> Self {
        Self {
            signals_incoming_metrics: Vec::new(),
            duration_metrics: Vec::new(),
            signals_outgoing_metrics: Vec::new(),
            end_nodes: HashMap::new(),
            start_nodes: HashMap::new(),
        }
    }

    /// Returns `true` if any flow_metrics are configured.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.signals_incoming_metrics.iter().any(Option::is_some)
            || self.duration_metrics.iter().any(Option::is_some)
            || self.signals_outgoing_metrics.iter().any(Option::is_some)
    }
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use super::*;
    use crate::testing::test_pipeline_ctx;

    fn one_flow_metric_state() -> PipelineFlowMetricState {
        let (ctx, _) = test_pipeline_ctx();
        let entity_key = ctx
            .metrics_registry()
            .register_entity(FlowAttributeSet::default());
        let signals_incoming_metric = ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowSignalsIncomingMetrics>(entity_key);
        let duration_metric = ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowDurationMetrics>(entity_key);
        let signals_outgoing_metric = ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowSignalsOutgoingMetrics>(entity_key);
        PipelineFlowMetricState {
            signals_incoming_metrics: vec![Some(signals_incoming_metric)],
            duration_metrics: vec![Some(duration_metric)],
            signals_outgoing_metrics: vec![Some(signals_outgoing_metric)],
            end_nodes: HashMap::from([(2, 0)]),
            start_nodes: HashMap::from([(0, 0)]),
        }
    }

    #[test]
    fn empty_state_is_inactive() {
        let state = PipelineFlowMetricState::empty();
        assert!(!state.is_active());
    }

    #[test]
    fn nonempty_state_is_active() {
        let state = one_flow_metric_state();
        assert!(state.is_active());
    }

    #[test]
    fn direct_record_increments_mmsc() {
        let mut state = one_flow_metric_state();
        state.duration_metrics[0]
            .as_mut()
            .unwrap()
            .compute_duration
            .record(100.0);
        state.duration_metrics[0]
            .as_mut()
            .unwrap()
            .compute_duration
            .record(200.0);

        let snap = state.duration_metrics[0]
            .as_mut()
            .unwrap()
            .compute_duration
            .get();
        assert_eq!(snap.count, 2);
        assert!((snap.min - 100.0).abs() < f64::EPSILON);
        assert!((snap.max - 200.0).abs() < f64::EPSILON);
        assert!((snap.sum - 300.0).abs() < f64::EPSILON);
    }

    #[test]
    fn direct_record_increments_items_mmsc() {
        let mut state = one_flow_metric_state();
        state.signals_incoming_metrics[0]
            .as_mut()
            .unwrap()
            .signals_incoming
            .record(10.0);
        state.signals_incoming_metrics[0]
            .as_mut()
            .unwrap()
            .signals_incoming
            .record(20.0);
        state.signals_outgoing_metrics[0]
            .as_mut()
            .unwrap()
            .signals_outgoing
            .record(7.0);
        state.signals_outgoing_metrics[0]
            .as_mut()
            .unwrap()
            .signals_outgoing
            .record(8.0);

        let consumed = state.signals_incoming_metrics[0]
            .as_mut()
            .unwrap()
            .signals_incoming
            .get();
        assert_eq!(consumed.count, 2);
        assert!((consumed.sum - 30.0).abs() < f64::EPSILON);

        let produced = state.signals_outgoing_metrics[0]
            .as_mut()
            .unwrap()
            .signals_outgoing
            .get();
        assert_eq!(produced.count, 2);
        assert!((produced.sum - 15.0).abs() < f64::EPSILON);
    }

    // -- build_flow_metric_state validation tests --

    use otap_df_config::policy::{FlowBounds, FlowMetric, FlowMetricConfig, TelemetryPolicy};

    fn policy_with(flow_metrics: Vec<FlowMetricConfig>) -> TelemetryPolicy {
        TelemetryPolicy {
            flow_metrics,
            ..TelemetryPolicy::default()
        }
    }

    fn sw(name: &str, start: &str, stop: &str) -> FlowMetricConfig {
        FlowMetricConfig {
            name: name.to_string(),
            bounds: FlowBounds {
                start_node: start.to_string(),
                end_node: stop.to_string(),
            },
            metrics: None,
        }
    }

    /// Asserts that `err` is `Error::ConfigError(InvalidUserConfig)` whose
    /// message mentions the given flow_metric name.
    fn assert_invalid_user_config(err: &crate::error::Error, sw_name: &str) {
        match err {
            crate::error::Error::ConfigError(boxed) => match boxed.as_ref() {
                otap_df_config::error::Error::InvalidUserConfig { error } => {
                    assert!(
                        error.contains(sw_name),
                        "expected error to mention `{sw_name}`, got: {error}"
                    );
                }
                other => panic!("expected InvalidUserConfig, got: {other:?}"),
            },
            other => panic!("expected ConfigError(InvalidUserConfig), got: {other:?}"),
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
    fn valid_flow_metric_is_registered() {
        let (ctx, _) = test_pipeline_ctx();
        // Shared processors are accepted because the caller in runtime_pipeline
        // includes them in `processor_indices`; this validator is kind-agnostic.
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "c")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .expect("valid config should build");

        assert_eq!(state.duration_metrics.len(), 1);
        assert!(state.start_nodes.contains_key(&0)); // "a" = index 0
        assert_eq!(state.end_nodes.get(&2), Some(&0)); // "c" = index 2
    }

    #[test]
    fn duration_only_registers_only_duration_metric_set() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let mut flow = sw("duration_only", "a", "b");
        flow.metrics = Some(vec![FlowMetric::ComputeDuration]);
        let policy = policy_with(vec![flow]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .expect("duration-only config should build");

        assert!(state.duration_metrics[0].is_some());
        assert!(state.signals_incoming_metrics[0].is_none());
        assert!(state.signals_outgoing_metrics[0].is_none());
    }

    #[test]
    fn unknown_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "missing")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw1");
    }

    #[test]
    fn non_processor_start_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["recv", "proc1", "proc2"], &["recv"]);
        let policy = policy_with(vec![sw("sw1", "recv", "proc2")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw1");
    }

    #[test]
    fn non_processor_end_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["proc1", "proc2", "exp"], &["exp"]);
        let policy = policy_with(vec![sw("sw1", "proc1", "exp")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw1");
    }

    #[test]
    fn shared_start_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        // Two flow_metrics share "a" as start node.
        let policy = policy_with(vec![sw("sw1", "a", "b"), sw("sw2", "a", "d")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw2");
    }

    #[test]
    fn shared_end_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        // Two flow_metrics share "d" as stop node.
        let policy = policy_with(vec![sw("sw1", "a", "d"), sw("sw2", "c", "d")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw2");
    }

    #[test]
    fn stop_of_one_is_start_of_another_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "b"), sw("sw2", "b", "c")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw2");
    }

    #[test]
    fn disjoint_flow_metrics_are_both_registered() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        // Non-overlapping: a→b and c→d.
        let policy = policy_with(vec![sw("sw1", "a", "b"), sw("sw2", "c", "d")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx)
            .expect("disjoint config should build");

        assert_eq!(state.duration_metrics.len(), 2);
        assert!(state.start_nodes.contains_key(&0)); // "a"
        assert!(state.start_nodes.contains_key(&2)); // "c"
        assert_eq!(state.end_nodes.get(&1), Some(&0)); // "b" → sw1
        assert_eq!(state.end_nodes.get(&3), Some(&1)); // "d" → sw2
    }
}
