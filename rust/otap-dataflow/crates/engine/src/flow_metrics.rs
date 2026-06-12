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
use std::collections::{HashMap, HashSet, VecDeque};
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
    #[metric(name = "signals.incoming", unit = "{item}")]
    pub signals_incoming: Mmsc,
}

/// Duration metric set emitted by the end node of a flow range.
#[metric_set(name = "flow")]
#[derive(Debug, Default, Clone)]
pub struct FlowDurationMetrics {
    /// Sum of per-node compute durations (nanoseconds) for messages traversing the flow range.
    #[metric(name = "compute.duration", unit = "ns")]
    pub compute_duration: Mmsc,
}

/// Outgoing signal metric set emitted by the end node of a flow range.
#[metric_set(name = "flow")]
#[derive(Debug, Default, Clone)]
pub struct FlowSignalsOutgoingMetrics {
    /// Number of signal items (log records, spans, or metric data points) leaving the flow range.
    #[metric(name = "signals.outgoing", unit = "{item}")]
    pub signals_outgoing: Mmsc,
}

/// Kept signal metric set emitted by a decision node within a flow range.
#[metric_set(name = "flow")]
#[derive(Debug, Default, Clone)]
pub struct FlowSignalsKeptMetrics {
    /// Number of signal items (log records, spans, or metric data points) a
    /// decision node chose to keep.
    #[metric(name = "signals.kept", unit = "{item}")]
    pub signals_kept: Mmsc,
}

/// Dropped signal metric set emitted by a decision node within a flow range.
#[metric_set(name = "flow")]
#[derive(Debug, Default, Clone)]
pub struct FlowSignalsDroppedMetrics {
    /// Number of signal items (log records, spans, or metric data points) a
    /// decision node chose to drop.
    #[metric(name = "signals.dropped", unit = "{item}")]
    pub signals_dropped: Mmsc,
}

/// Entity attributes that scope a flow metric set.
#[attribute_set(name = "flow.attrs")]
#[derive(Debug, Clone, Default, Hash)]
pub struct FlowAttributeSet {
    /// User-given flow identifier.
    #[attribute(key = "flow.id")]
    pub flow_id: Cow<'static, str>,
    /// Name of the processor node where the measurement begins.
    #[attribute(key = "flow.node.start")]
    pub start_node: Cow<'static, str>,
    /// Name of the processor node where the measurement ends.
    #[attribute(key = "flow.node.end")]
    pub end_node: Cow<'static, str>,
    /// Per-flow purpose differentiator (e.g. `filter`, `transform`). Always
    /// emitted as the `flow.purpose` scope attribute; carries an empty value
    /// when the flow declares no purpose. Lets OTel View selectors target
    /// distinct flavors of processor work while all flows share the single
    /// `flow` scope.
    #[attribute(key = "flow.purpose")]
    pub purpose: Cow<'static, str>,
    /// Name of the decision node that recorded `signals.kept` / `signals.dropped`.
    /// Always emitted as the `flow.node.decision` scope attribute; carries an
    /// empty value for the flow's incoming/outgoing/duration entity (which is
    /// not attributed to a specific decision node). Lets a single flow contain
    /// multiple decision nodes without conflation.
    ///
    /// Aggregation across this attribute differs by metric. `signals.dropped`
    /// sums correctly: drops at different decision nodes are disjoint (a dropped
    /// item never reaches a later node), so the sum equals the flow's total
    /// removed = `signals.incoming - signals.outgoing`. `signals.kept` does NOT
    /// sum: each node's kept count is relative to its own input, and for nodes
    /// in series those sets are nested subsets, so summing double-counts. The
    /// flow-wide kept count is simply `signals.outgoing`; the per-node
    /// `signals.kept` is only meaningful on its own entity.
    #[attribute(key = "flow.node.decision")]
    pub decision: Cow<'static, str>,
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
    /// Decision-node candidates keyed by node index. A processor at one of
    /// these indices that declares the keep/drop capability records
    /// `signals.kept` / `signals.dropped` attributed to itself via
    /// `flow.node.decision`. Populated only for flows that enable kept and/or
    /// dropped metrics, for every processor node within the flow's active
    /// range (start..=end).
    pub decision_candidates: HashMap<usize, DecisionCandidate>,
}

/// A node that may record `signals.kept` / `signals.dropped` for a flow.
///
/// Carries the flow's base attribute set (with an empty `flow.node.decision`)
/// so the runtime can register a per-node decision entity by filling in the
/// deciding node's name, plus which of the two metrics the flow enables.
#[derive(Clone)]
pub(crate) struct DecisionCandidate {
    /// Flow attribute set with an empty `decision` field; the runtime fills in
    /// the deciding node's name before registering the per-node entity.
    pub attrs: FlowAttributeSet,
    /// Whether the flow enables `signals.kept`.
    pub kept: bool,
    /// Whether the flow enables `signals.dropped`.
    pub dropped: bool,
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

/// Decision-side measurements for a node that records keep/drop decisions
/// within a flow_metric range.
///
/// Groups the metric sets and their accumulators so that they share the
/// `Option` on `FlowMetricState` — non-decision nodes pay no allocation for
/// either.
#[derive(Clone)]
pub(crate) struct DecisionFlowMetrics<Accumulator> {
    /// Kept signal measurement, if enabled for this flow and this node is a
    /// keep/drop decision node.
    pub signals_kept: Option<(MetricSet<FlowSignalsKeptMetrics>, Accumulator)>,
    /// Dropped signal measurement, if enabled for this flow and this node is a
    /// keep/drop decision node.
    pub signals_dropped: Option<(MetricSet<FlowSignalsDroppedMetrics>, Accumulator)>,
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
    /// Whether this node is a keep/drop decision node for some flow.
    pub is_decision: bool,
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
    /// Decision-side measurements (kept/dropped).
    pub decision: DecisionFlowMetrics<Accumulator>,
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
            is_decision: false,
            active: false,
            incoming: IncomingFlowMetrics {
                signals_incoming: None,
            },
            end: EndFlowMetrics {
                duration: None,
                signals_outgoing: None,
            },
            decision: DecisionFlowMetrics {
                signals_kept: None,
                signals_dropped: None,
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
            is_decision: false,
            active: false,
            incoming: IncomingFlowMetrics {
                signals_incoming: None,
            },
            end: EndFlowMetrics {
                duration: None,
                signals_outgoing: None,
            },
            decision: DecisionFlowMetrics {
                signals_kept: None,
                signals_dropped: None,
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
    pipeline_connections: &[(usize, usize)],
) -> Result<PipelineFlowMetricState, crate::error::Error> {
    let mut signals_incoming_metrics = Vec::new();
    let mut duration_metrics = Vec::new();
    let mut signals_outgoing_metrics = Vec::new();
    let mut end_nodes: HashMap<usize, usize> = HashMap::new();
    let mut start_nodes: HashMap<usize, usize> = HashMap::new();
    let mut decision_candidates: HashMap<usize, DecisionCandidate> = HashMap::new();

    let pipeline_attrs = pipeline_context.pipeline_attribute_set();
    let adjacency = build_adjacency(pipeline_connections);

    // Collect resolved (start, end, name) triples for the graph-based
    // interleave check that runs after all flow metrics are registered.
    let mut resolved_ranges: Vec<(usize, usize, String)> = Vec::new();

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
                flow_config.id, flow_config.bounds.start_node, flow_config.bounds.end_node
            )));
        };

        if !processor_indices.contains(&start_idx) || !processor_indices.contains(&end_idx) {
            return Err(invalid_flow_metric_config(format!(
                "flow metric `{}` start/end nodes must be processors: start=`{}`, end=`{}`",
                flow_config.id, flow_config.bounds.start_node, flow_config.bounds.end_node
            )));
        }

        if start_nodes.contains_key(&start_idx)
            || end_nodes.contains_key(&start_idx)
            || start_nodes.contains_key(&end_idx)
            || end_nodes.contains_key(&end_idx)
        {
            return Err(invalid_flow_metric_config(format!(
                "flow metric `{}` overlaps with another flow metric (non-overlapping ranges only): start=`{}`, end=`{}`",
                flow_config.id, flow_config.bounds.start_node, flow_config.bounds.end_node
            )));
        }

        let attrs = FlowAttributeSet {
            flow_id: Cow::Owned(flow_config.id.clone()),
            start_node: Cow::Owned(flow_config.bounds.start_node.clone()),
            end_node: Cow::Owned(flow_config.bounds.end_node.clone()),
            purpose: flow_config
                .purpose
                .clone()
                .map_or(Cow::Borrowed(""), Cow::Owned),
            decision: Cow::Borrowed(""),
            pipeline_attrs: pipeline_attrs.clone(),
        };

        let entity_key = pipeline_context
            .metrics_registry()
            .register_entity(attrs.clone());
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

        // Register keep/drop decision candidates for every processor node
        // within this flow's active range (inclusive of start and end). A
        // processor at one of these indices that declares the keep/drop
        // capability records `signals.kept` / `signals.dropped` attributed to
        // itself. A decision node must belong to at most one flow — otherwise
        // its single kept/dropped count could not be unambiguously attributed
        // to one `flow.id`. The pairwise overlap check below only rejects
        // shared start/end nodes, so we explicitly reject shared interior nodes
        // here (e.g. fan-in/merge topologies where two ranges meet at a node).
        let kept = flow_config.has(FlowMetric::SignalsKept);
        let dropped = flow_config.has(FlowMetric::SignalsDropped);
        if kept || dropped {
            let (mut range_nodes, _) = active_range(start_idx, end_idx, &adjacency);
            let _ = range_nodes.insert(start_idx);
            let _ = range_nodes.insert(end_idx);
            for node_idx in range_nodes {
                if processor_indices.contains(&node_idx) {
                    if let Some(existing) = decision_candidates.get(&node_idx) {
                        return Err(invalid_flow_metric_config(format!(
                            "flow metric `{}` shares keep/drop decision node `{}` with flow metric `{}`: a decision node may belong to at most one flow",
                            flow_config.id,
                            node_name_to_index
                                .iter()
                                .find(|&(_, &idx)| idx == node_idx)
                                .map_or("<unknown>", |(name, _)| name.as_str()),
                            existing.attrs.flow_id,
                        )));
                    }
                    let _ = decision_candidates.insert(
                        node_idx,
                        DecisionCandidate {
                            attrs: attrs.clone(),
                            kept,
                            dropped,
                        },
                    );
                }
            }
        }

        let id = duration_metrics.len();
        signals_incoming_metrics.push(signals_incoming_metric);
        duration_metrics.push(duration_metric);
        signals_outgoing_metrics.push(signals_outgoing_metric);
        let _ = end_nodes.insert(end_idx, id);
        let _ = start_nodes.insert(start_idx, id);
        resolved_ranges.push((start_idx, end_idx, flow_config.id.clone()));
    }

    if !resolved_ranges.is_empty() {
        validate_metric_ranges(&resolved_ranges, &adjacency)?;
    }

    Ok(PipelineFlowMetricState {
        signals_incoming_metrics,
        duration_metrics,
        signals_outgoing_metrics,
        end_nodes,
        start_nodes,
        decision_candidates,
    })
}

fn invalid_flow_metric_config(error: String) -> crate::error::Error {
    crate::error::Error::ConfigError(Box::new(otap_df_config::error::Error::InvalidUserConfig {
        error,
    }))
}

/// Compute the "active range" of a flow metric: the set of node indices
/// reachable from "start" via the forward adjacency list, stopping
/// expansion at "end".
fn active_range(
    start: usize,
    end: usize,
    adjacency: &HashMap<usize, Vec<usize>>,
) -> (HashSet<usize>, bool) {
    let mut visited = HashSet::new();
    let mut queue = VecDeque::new();
    let _ = visited.insert(start);
    queue.push_back(start);

    while let Some(node) = queue.pop_front() {
        if node == end {
            continue;
        }
        if let Some(neighbors) = adjacency.get(&node) {
            for &next in neighbors {
                if visited.insert(next) {
                    queue.push_back(next);
                }
            }
        }
    }

    // The end node does not alter the accumulator for this
    // flow metric range.
    let end_reachable = visited.remove(&end);
    (visited, end_reachable)
}

/// Validate flow metric ranges. We can fail range validation for one of two reasons:
/// 1. The ranges interleave, meaning the start node of one range falls within the
///    other's active range.
/// 2. The end node of a metric is not reachable from its start node.
fn validate_metric_ranges(
    ranges: &[(usize, usize, String)],
    adjacency: &HashMap<usize, Vec<usize>>,
) -> Result<(), crate::error::Error> {
    // Pre-compute the active range for each flow metric, rejecting any
    // whose end node is not reachable from its start node.
    let mut active_ranges: Vec<HashSet<usize>> = Vec::with_capacity(ranges.len());
    for &(start, end, ref name) in ranges {
        let (set, end_reachable) = active_range(start, end, adjacency);
        if !end_reachable {
            return Err(invalid_flow_metric_config(format!(
                "flow metric `{name}` end node is not reachable from its start node"
            )));
        }
        active_ranges.push(set);
    }

    // Pairwise check: does any flow metric's start fall inside another's
    // active range?
    for i in 0..ranges.len() {
        for j in (i + 1)..ranges.len() {
            let (start_j, _, ref name_j) = ranges[j];
            if active_ranges[i].contains(&start_j) {
                return Err(invalid_flow_metric_config(format!(
                    "flow metric `{}` interleaves with `{}`: \
                     start node of `{}` is reachable from start of `{}` \
                     before reaching its end node",
                    name_j, ranges[i].2, name_j, ranges[i].2,
                )));
            }
            let (start_i, _, ref name_i) = ranges[i];
            if active_ranges[j].contains(&start_i) {
                return Err(invalid_flow_metric_config(format!(
                    "flow metric `{}` interleaves with `{}`: \
                     start node of `{}` is reachable from start of `{}` \
                     before reaching its end node",
                    name_i, name_j, name_i, name_j,
                )));
            }
        }
    }
    Ok(())
}

/// Build a forward-edge adjacency list from a flat edge list.
fn build_adjacency(edges: &[(usize, usize)]) -> HashMap<usize, Vec<usize>> {
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(src, dst) in edges {
        adj.entry(src).or_default().push(dst);
    }
    adj
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
            decision_candidates: HashMap::new(),
        }
    }

    /// Returns `true` if any flow_metrics are configured.
    #[must_use]
    #[allow(dead_code)]
    pub fn is_active(&self) -> bool {
        self.signals_incoming_metrics.iter().any(Option::is_some)
            || self.duration_metrics.iter().any(Option::is_some)
            || self.signals_outgoing_metrics.iter().any(Option::is_some)
            || !self.decision_candidates.is_empty()
    }
}

#[cfg(all(test, feature = "test-utils"))]
mod tests {
    use super::*;
    use crate::testing::test_pipeline_ctx;
    use otap_df_telemetry::attributes::{AttributeSetHandler, AttributeValue};

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
            decision_candidates: HashMap::new(),
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
            id: name.to_string(),
            bounds: FlowBounds {
                start_node: start.to_string(),
                end_node: stop.to_string(),
            },
            metrics: None,
            purpose: None,
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

    /// Helper: build a flat edge list from `(from, to)` string pairs,
    /// resolving names through the provided `name_to_index` map.
    fn test_edges(
        edges: &[(&str, &str)],
        name_to_index: &HashMap<String, usize>,
    ) -> Vec<(usize, usize)> {
        edges
            .iter()
            .filter_map(|&(from, to)| {
                let src = *name_to_index.get(from)?;
                let dst = *name_to_index.get(to)?;
                Some((src, dst))
            })
            .collect()
    }

    #[test]
    fn valid_flow_metric_is_registered() {
        let (ctx, _) = test_pipeline_ctx();
        // Shared processors are accepted because the caller in runtime_pipeline
        // includes them in `processor_indices`; this validator is kind-agnostic.
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c")], &names);
        let policy = policy_with(vec![sw("sw1", "a", "c")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("valid config should build");

        assert_eq!(state.duration_metrics.len(), 1);
        assert!(state.start_nodes.contains_key(&0)); // "a" = index 0
        assert_eq!(state.end_nodes.get(&2), Some(&0)); // "c" = index 2
    }

    #[test]
    fn duration_only_registers_only_duration_metric_set() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let edges = test_edges(&[("a", "b")], &names);
        let mut flow = sw("duration_only", "a", "b");
        flow.metrics = Some(vec![FlowMetric::ComputeDuration]);
        let policy = policy_with(vec![flow]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("duration-only config should build");

        assert!(state.duration_metrics[0].is_some());
        assert!(state.signals_incoming_metrics[0].is_none());
        assert!(state.signals_outgoing_metrics[0].is_none());
    }

    #[test]
    fn kept_dropped_registers_decision_candidates_for_range_processors() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c")], &names);
        let mut flow = sw("decisions", "a", "c");
        flow.metrics = Some(vec![FlowMetric::SignalsKept, FlowMetric::SignalsDropped]);
        let policy = policy_with(vec![flow]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("kept/dropped config should build");

        // Every processor node in the range a..=c is a decision candidate.
        assert_eq!(state.decision_candidates.len(), 3);
        for idx in [0usize, 1, 2] {
            let candidate = state
                .decision_candidates
                .get(&idx)
                .expect("each range node should be a candidate");
            assert!(candidate.kept);
            assert!(candidate.dropped);
            assert!(candidate.attrs.decision.is_empty());
        }
        assert!(state.is_active());
    }

    #[test]
    fn kept_only_registers_candidates_without_dropped() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let edges = test_edges(&[("a", "b")], &names);
        let mut flow = sw("kept_only", "a", "b");
        flow.metrics = Some(vec![FlowMetric::SignalsKept]);
        let policy = policy_with(vec![flow]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("kept-only config should build");

        let candidate = state.decision_candidates.get(&0).expect("candidate");
        assert!(candidate.kept);
        assert!(!candidate.dropped);
    }

    #[test]
    fn no_kept_dropped_means_no_decision_candidates() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let edges = test_edges(&[("a", "b")], &names);
        let mut flow = sw("duration_only", "a", "b");
        flow.metrics = Some(vec![FlowMetric::ComputeDuration]);
        let policy = policy_with(vec![flow]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("config should build");

        assert!(state.decision_candidates.is_empty());
    }

    #[test]
    fn single_node_flow_is_supported() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["solo"], &[]);
        let mut flow = sw("single", "solo", "solo");
        flow.metrics = Some(vec![FlowMetric::SignalsKept, FlowMetric::SignalsDropped]);
        let policy = policy_with(vec![flow]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &[])
            .expect("single-node flow should build");

        assert_eq!(state.decision_candidates.len(), 1);
        assert!(state.decision_candidates.contains_key(&0));
    }

    #[test]
    fn shared_interior_decision_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        // Fan-in/merge topology: two ranges meet at interior node `m`.
        // a -> m -> b  and  c -> m -> d. Neither range contains the other's
        // start node, so the interleave check passes — but both ranges include
        // `m`, which would otherwise silently overwrite the decision candidate.
        let (names, procs) = test_maps(&["a", "c", "m", "b", "d"], &[]);
        let edges = test_edges(&[("a", "m"), ("c", "m"), ("m", "b"), ("m", "d")], &names);
        let mut flow1 = sw("flow1", "a", "b");
        flow1.metrics = Some(vec![FlowMetric::SignalsKept, FlowMetric::SignalsDropped]);
        let mut flow2 = sw("flow2", "c", "d");
        flow2.metrics = Some(vec![FlowMetric::SignalsKept, FlowMetric::SignalsDropped]);
        let policy = policy_with(vec![flow1, flow2]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .err()
            .expect("shared interior decision node should be rejected");
        assert_invalid_user_config(&err, "flow2");
    }

    #[test]
    fn unknown_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "missing")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &[])
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw1");
    }

    #[test]
    fn non_processor_start_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["recv", "proc1", "proc2"], &["recv"]);
        let policy = policy_with(vec![sw("sw1", "recv", "proc2")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &[])
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw1");
    }

    #[test]
    fn non_processor_end_node_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["proc1", "proc2", "exp"], &["exp"]);
        let policy = policy_with(vec![sw("sw1", "proc1", "exp")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &[])
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

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &[])
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

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &[])
            .err()
            .expect("expected Err");

        assert_invalid_user_config(&err, "sw2");
    }

    #[test]
    fn stop_of_one_is_start_of_another_is_rejected() {
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let policy = policy_with(vec![sw("sw1", "a", "b"), sw("sw2", "b", "c")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &[])
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

        let edges = test_edges(&[("a", "b"), ("b", "c"), ("c", "d")], &names);
        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("disjoint config should build");

        assert_eq!(state.duration_metrics.len(), 2);
        assert!(state.start_nodes.contains_key(&0)); // "a"
        assert!(state.start_nodes.contains_key(&2)); // "c"
        assert_eq!(state.end_nodes.get(&1), Some(&0)); // "b" → sw1
        assert_eq!(state.end_nodes.get(&3), Some(&1)); // "d" → sw2
    }

    // Validation of flow metric interleaving detection.

    #[test]
    fn interleaved_distinct_endpoints_linear_path_is_rejected() {
        // Toplogy: a -> b -> c -> d
        // Flow metrics: a->c and b->d
        // Vanilla interleaving on linear path. Should fail validation.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c"), ("c", "d")], &names);
        let policy = policy_with(vec![sw("sw1", "a", "c"), sw("sw2", "b", "d")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .err()
            .expect("expected Err for interleaved ranges");

        assert_invalid_user_config(&err, "sw2");
    }

    #[test]
    fn interleaved_reverse_order_is_rejected() {
        // Toplogy: a -> b -> c -> d
        // Flow metrics: a->c and b->d
        // Vanilla interleaving on linear path, but with reverse declaration order. Should fail validation.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c"), ("c", "d")], &names);
        let policy = policy_with(vec![sw("sw1", "b", "d"), sw("sw2", "a", "c")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .err()
            .expect("expected Err for interleaved ranges");

        assert_invalid_user_config(&err, "sw2");
    }

    #[test]
    fn disjoint_on_linear_path_is_accepted() {
        // Topology: a -> b -> c -> d -> e -> f
        // Flow metrics: a->c and d->f
        // Fully disjoint. Should pass validation.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d", "e", "f"], &[]);
        let edges = test_edges(
            &[("a", "b"), ("b", "c"), ("c", "d"), ("d", "e"), ("e", "f")],
            &names,
        );
        let policy = policy_with(vec![sw("sw1", "a", "c"), sw("sw2", "d", "f")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("disjoint ranges should build");

        assert_eq!(state.duration_metrics.len(), 2);
    }

    #[test]
    fn interleaved_on_branching_path_is_rejected() {
        // Topology: a -> b -> d, a -> c -> d, d -> e (Diamond pattern)
        // Flow metrics: a->d and b->e.
        // b is inside a->d's range, so these interleave. Should fail validation.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d", "e"], &[]);
        let edges = test_edges(
            &[("a", "b"), ("a", "c"), ("b", "d"), ("c", "d"), ("d", "e")],
            &names,
        );
        let policy = policy_with(vec![sw("sw1", "a", "d"), sw("sw2", "b", "e")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .err()
            .expect("expected Err for interleaved ranges on diamond");

        assert_invalid_user_config(&err, "sw2");
    }

    #[test]
    fn disjoint_on_separate_branches_is_accepted() {
        // Topology: a -> b -> c, a -> d -> e
        // Flow metrics: b->c and d->e
        // Separate branches, should have no overlap. Should pass validation.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d", "e"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c"), ("a", "d"), ("d", "e")], &names);
        let policy = policy_with(vec![sw("sw1", "b", "c"), sw("sw2", "d", "e")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("separate-branch ranges should build");

        assert_eq!(state.duration_metrics.len(), 2);
    }

    #[test]
    fn nested_ranges_are_rejected() {
        // Topology: a -> b -> c -> d
        // Flow metrics: a->d and b->c
        // The single-accumulator design does not support nesting. Should fail validation.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c"), ("c", "d")], &names);
        let policy = policy_with(vec![sw("sw1", "a", "d"), sw("sw2", "b", "c")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .err()
            .expect("expected Err for nested ranges");

        assert_invalid_user_config(&err, "sw2");
    }

    #[test]
    fn single_flow_metric_accepted() {
        // Topology: a -> b -> c
        // Flow metrics: a->b and b->c
        // Only one flow metric, so no interleaving possible. Should pass validation.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c")], &names);
        let policy = policy_with(vec![sw("sw1", "a", "c")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("single flow metric should build");

        assert_eq!(state.duration_metrics.len(), 1);
    }

    // Validation of active_range helper for interleaved flow metrics detection.

    #[test]
    fn active_range_linear() {
        // a(0) -> b(1) -> c(2) -> d(3)
        let adj = build_adjacency(&[(0, 1), (1, 2), (2, 3)]);
        let (range, _end_reachable) = active_range(0, 2, &adj);
        // Start=0 included, 1 is between, end=2 excluded.
        assert!(range.contains(&0));
        assert!(range.contains(&1));
        assert!(!range.contains(&2));
        assert!(!range.contains(&3));
    }

    #[test]
    fn active_range_diamond() {
        // a(0) -> b(1) -> d(3)
        // a(0) -> c(2) -> d(3)
        let adj = build_adjacency(&[(0, 1), (0, 2), (1, 3), (2, 3)]);
        let (range, _end_reachable) = active_range(0, 3, &adj);
        assert!(range.contains(&0));
        assert!(range.contains(&1));
        assert!(range.contains(&2));
        assert!(!range.contains(&3));
    }

    #[test]
    fn active_range_end_unreachable() {
        // a(0) -> b(1), end=5 not reachable from start=0.
        let adj = build_adjacency(&[(0, 1), (4, 5)]);
        let (range, _end_reachable) = active_range(0, 5, &adj);
        assert!(range.contains(&0));
        assert!(range.contains(&1));
        assert!(!range.contains(&5));
    }

    #[test]
    fn unreachable_end_is_rejected() {
        // Topology: a -> b -> c (no edge to d)
        // Flow metric: a->d. d is not reachable from a.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c")], &names);
        let policy = policy_with(vec![sw("sw1", "a", "d")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .err()
            .expect("expected Err for unreachable end node");

        assert_invalid_user_config(&err, "sw1");
    }

    #[test]
    fn unreachable_end_on_separate_branch_is_rejected() {
        // Topology: a -> b, a -> c -> d
        // Flow metric: b->d. d is reachable from a (via c) but not from b.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        let edges = test_edges(&[("a", "b"), ("a", "c"), ("c", "d")], &names);
        let policy = policy_with(vec![sw("sw1", "b", "d")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .err()
            .expect("expected Err for unreachable end node on separate branch");

        assert_invalid_user_config(&err, "sw1");
    }

    #[test]
    fn reverse_direction_is_rejected() {
        // Topology: a -> b -> c
        // Flow metric: c->a. Edges are forward-only; a is not reachable from c.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c")], &names);
        let policy = policy_with(vec![sw("sw1", "c", "a")]);

        let err = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .err()
            .expect("expected Err for reverse-direction flow metric");

        assert_invalid_user_config(&err, "sw1");
    }

    #[test]
    fn adjacent_nodes_are_accepted() {
        // Topology: a -> b
        // Flow metric: a->b. Minimal reachable range (single direct edge).
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b"], &[]);
        let edges = test_edges(&[("a", "b")], &names);
        let policy = policy_with(vec![sw("sw1", "a", "b")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("adjacent-node flow metric should build");

        assert_eq!(state.duration_metrics.len(), 1);
    }

    #[test]
    fn multi_hop_reachable_end_is_accepted() {
        // Topology: a -> b -> c -> d -> e
        // Flow metric: a->e. End is reachable via a long forward path.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d", "e"], &[]);
        let edges = test_edges(&[("a", "b"), ("b", "c"), ("c", "d"), ("d", "e")], &names);
        let policy = policy_with(vec![sw("sw1", "a", "e")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("multi-hop reachable flow metric should build");

        assert_eq!(state.duration_metrics.len(), 1);
    }

    #[test]
    fn diamond_end_reachable_via_either_branch_is_accepted() {
        // Topology: a -> b -> d, a -> c -> d
        // Flow metric: a->d. End is reachable via two parallel paths.
        let (ctx, _) = test_pipeline_ctx();
        let (names, procs) = test_maps(&["a", "b", "c", "d"], &[]);
        let edges = test_edges(&[("a", "b"), ("a", "c"), ("b", "d"), ("c", "d")], &names);
        let policy = policy_with(vec![sw("sw1", "a", "d")]);

        let state = build_flow_metric_state(&policy, &names, &procs, &ctx, &edges)
            .expect("diamond-reachable flow metric should build");

        assert_eq!(state.duration_metrics.len(), 1);
    }

    #[test]
    fn flow_attribute_set_exposes_purpose_scope_attribute() {
        // Descriptor must declare the `flow.purpose` key so View selectors can
        // match on it as a scope attribute.
        let descriptor = FlowAttributeSet::default().descriptor();
        assert!(
            descriptor.fields.iter().any(|f| f.key == "flow.purpose"),
            "flow.purpose missing from FlowAttributeSet descriptor"
        );

        // When set, the value is emitted under the `flow.purpose` key.
        let attrs = FlowAttributeSet {
            flow_id: "sampling".into(),
            start_node: "log_sampler".into(),
            end_node: "log_sampler".into(),
            purpose: "filter".into(),
            ..FlowAttributeSet::default()
        };
        let set: Vec<(&str, AttributeValue)> = attrs
            .iter_attributes()
            .map(|(key, value)| (key, value.clone()))
            .collect();
        assert!(
            set.contains(&("flow.purpose", AttributeValue::String("filter".to_string()))),
            "expected flow.purpose=filter in {set:?}"
        );

        // When unset, the key is still present but carries an empty value.
        let unset = FlowAttributeSet::default();
        let unset_set: Vec<(&str, AttributeValue)> = unset
            .iter_attributes()
            .map(|(key, value)| (key, value.clone()))
            .collect();
        assert!(
            unset_set.contains(&("flow.purpose", AttributeValue::String(String::new()))),
            "expected empty flow.purpose in {unset_set:?}"
        );
    }
}
