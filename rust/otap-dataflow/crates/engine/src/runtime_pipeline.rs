// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use crate::Interests;
use crate::ReceivedAtNode;
use crate::Unwindable;
use crate::channel_metrics::{ChannelMetricsHandle, ConsumedMetrics, ProducedMetrics};
use crate::completion_emission_metrics::{
    CompletionEmissionMetricsHandle, make_completion_emission_metrics,
};
use crate::context::PipelineContext;
use crate::control::{
    ControlSenders, Controllable, NodeControlMsg, PipelineCompletionMsgReceiver,
    PipelineCompletionMsgSender, RuntimeCtrlMsgReceiver, RuntimeCtrlMsgSender,
};
use crate::entity_context::{NodeTaskContext, NodeTelemetryHandle, instrument_with_node_context};
use crate::error::{Error, TypedError};
use crate::flow_metrics::{
    FlowDurationMetrics, FlowSignalsIncomingMetrics, FlowSignalsOutgoingMetrics,
    build_flow_metric_state,
};
use crate::memory_limiter::MemoryPressureChanged;
use crate::node::{Node, NodeDefs, NodeId, NodeType, NodeWithPDataReceiver, NodeWithPDataSender};
use crate::pipeline_ctrl::{
    NodeMetricHandles, PipelineCompletionMsgDispatcher, RuntimeCtrlMsgManager,
    report_node_metrics_with_handles,
};
use crate::processor::FlowMetricHook;
use crate::terminal_state::TerminalState;
use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};
use otap_df_config::DeployedPipelineKey;
use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::policy::TelemetryPolicy;
use otap_df_telemetry::event::ObservedEventReporter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::fmt::Debug;
use std::rc::Rc;
use std::time::Duration;
use tokio::runtime::Builder;
use tokio::sync::watch;
use tokio::task::LocalSet;

/// Cadence at which the per-pipeline extension monitor reports its
/// lifecycle and aggregate metric sets.
const EXTENSION_MONITOR_TICK_INTERVAL: Duration = Duration::from_secs(1);

/// Cadence at which the extension monitor fans `CollectTelemetry` out
/// to active extensions so they can refresh their own metric sets.
const EXTENSION_MONITOR_COLLECT_TELEMETRY_INTERVAL: Duration = Duration::from_secs(10);

/// Build produced-request metric sets indexed by sorted output port name,
/// matching the `output_port_index` layout used in `RouteData`.
fn make_produced_metrics(
    telemetry_handle: &Option<NodeTelemetryHandle>,
    pipeline_context: &PipelineContext,
) -> Vec<MetricSet<ProducedMetrics>> {
    telemetry_handle
        .as_ref()
        .map(|h| {
            let mut keys = h.output_channel_keys();
            keys.sort_by(|a, b| a.0.cmp(&b.0));
            keys.iter()
                .map(|(_, key)| {
                    pipeline_context.register_metric_set_for_entity::<ProducedMetrics>(*key)
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Build per-node metric handles for the runtime control manager.
///
/// - `has_input`: whether to register consumed-request metrics (false for receivers).
/// - `has_outputs`: whether to register produced-request metrics (false for exporters).
fn make_node_metric_handles(
    telemetry_handle: &Option<NodeTelemetryHandle>,
    pipeline_context: &PipelineContext,
    has_input: bool,
    has_outputs: bool,
    completion_emission: Option<CompletionEmissionMetricsHandle>,
) -> NodeMetricHandles {
    let consumed = if has_input {
        telemetry_handle
            .as_ref()
            .and_then(|h| h.input_channel_key())
            .map(|key| pipeline_context.register_metric_set_for_entity::<ConsumedMetrics>(key))
    } else {
        None
    };
    let produced = if has_outputs {
        make_produced_metrics(telemetry_handle, pipeline_context)
    } else {
        Vec::new()
    };
    NodeMetricHandles {
        registry: pipeline_context.metrics_registry(),
        input: consumed,
        outputs: produced,
        completion_emission,
    }
}

/// Represents a runtime pipeline configuration that includes nodes with their respective configurations and instances.
///
/// Note: Having a Debug bound on `PData` allows us to use it in logging and debugging contexts,
/// which is useful for tracing the pipeline's execution and state.
pub struct RuntimePipeline<PData: Debug> {
    /// The pipeline configuration that defines the structure and behavior of the pipeline.
    config: PipelineConfig,
    /// A map node id to receiver runtime node.
    receivers: Vec<ReceiverWrapper<PData>>,
    /// A map node id to processor runtime node.
    processors: Vec<ProcessorWrapper<PData>>,
    /// A map node id to exporter runtime node.
    exporters: Vec<ExporterWrapper<PData>>,
    /// Extension wrappers that survived the build-time consumed-tracker pruning,
    /// each paired with the telemetry entity key registered for that variant.
    /// One entry per surviving local-or-shared variant. Active extensions in
    /// this list have their lifecycle tasks spawned by `run_forever` before
    /// data-path nodes; passive extensions hold their instance factories so
    /// `Capabilities::require_*` calls keep working at run time but are not
    /// spawned themselves.
    extensions: Vec<(
        crate::extension::ExtensionWrapper,
        otap_df_telemetry::registry::EntityKey,
    )>,

    /// A precomputed map of all node IDs to their Node trait objects (? @@@) for efficient access
    /// Indexed by NodeIndex
    nodes: NodeDefs<PData, PipeNode>,
    /// Channel metrics handles collected during build.
    channel_metrics: Vec<ChannelMetricsHandle>,
    /// Flags controlling pipeline-internal metrics collection/reporting.
    telemetry_policy: TelemetryPolicy,
}

pub(crate) fn report_terminal_metrics(
    metrics_reporter: &MetricsReporter,
    terminal_state: TerminalState,
) {
    for snapshot in terminal_state.into_metrics() {
        let _ = metrics_reporter.try_report_snapshot(snapshot);
    }
}

/// Build a flat edge list from a pipeline's connections, resolving node
/// names to indices. Names not found in the map are silently skipped.
fn connection_edges<'a>(
    connections: impl Iterator<Item = &'a otap_df_config::pipeline::PipelineConnection>,
    node_name_to_index: &HashMap<String, usize>,
) -> Vec<(usize, usize)> {
    let mut edges = Vec::new();
    for conn in connections {
        let from_indices: Vec<usize> = conn
            .from_nodes()
            .into_iter()
            .filter_map(|name| node_name_to_index.get(name.as_ref()).copied())
            .collect();
        let to_indices: Vec<usize> = conn
            .to_nodes()
            .into_iter()
            .filter_map(|name| node_name_to_index.get(name.as_ref()).copied())
            .collect();
        for &src in &from_indices {
            for &dst in &to_indices {
                edges.push((src, dst));
            }
        }
    }
    edges
}

/// PipeNode contains runtime-specific info.
pub(crate) struct PipeNode {
    index: usize, // NodeIndex into the appropriate vector w/ offset precomputed
}

impl PipeNode {
    /// Construct a pipe node with index referring to one an entry in
    /// the appropriate RuntimePipeline Vec.
    pub(crate) const fn new(index: usize) -> Self {
        Self { index }
    }
}

impl<PData: 'static + Debug + Clone> RuntimePipeline<PData> {
    /// Creates a new `RuntimePipeline` from the given pipeline configuration and nodes.
    #[must_use]
    pub(crate) fn new(
        config: PipelineConfig,
        receivers: Vec<ReceiverWrapper<PData>>,
        processors: Vec<ProcessorWrapper<PData>>,
        exporters: Vec<ExporterWrapper<PData>>,
        extensions: Vec<(
            crate::extension::ExtensionWrapper,
            otap_df_telemetry::registry::EntityKey,
        )>,
        nodes: NodeDefs<PData, PipeNode>,
        telemetry_policy: TelemetryPolicy,
    ) -> Self {
        Self {
            config,
            receivers,
            processors,
            exporters,
            extensions,
            nodes,
            channel_metrics: Default::default(),
            telemetry_policy,
        }
    }

    pub(crate) fn set_channel_metrics(&mut self, channel_metrics: Vec<ChannelMetricsHandle>) {
        self.channel_metrics = channel_metrics;
    }

    /// Returns the number of nodes in the pipeline.
    #[must_use]
    pub const fn node_count(&self) -> usize {
        self.receivers.len() + self.processors.len() + self.exporters.len()
    }

    /// Returns a reference to the pipeline configuration.
    #[must_use]
    pub const fn config(&self) -> &PipelineConfig {
        &self.config
    }
}

impl<PData: 'static + Debug + Clone + ReceivedAtNode + Unwindable + FlowMetricHook>
    RuntimePipeline<PData>
{
    /// Runs the pipeline forever, starting all nodes and handling their tasks.
    /// Returns an error if any node fails to start or if any task encounters an error.
    pub fn run_forever(
        self,
        pipeline_key: DeployedPipelineKey,
        pipeline_context: PipelineContext,
        event_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        control_plane_metrics_flush_interval: Duration,
        memory_pressure_rx: watch::Receiver<MemoryPressureChanged>,
        runtime_ctrl_msg_tx: RuntimeCtrlMsgSender<PData>,
        runtime_ctrl_msg_rx: RuntimeCtrlMsgReceiver<PData>,
        pipeline_completion_msg_tx: PipelineCompletionMsgSender<PData>,
        pipeline_completion_msg_rx: PipelineCompletionMsgReceiver<PData>,
    ) -> Result<Vec<()>, Error> {
        use futures::stream::{FuturesUnordered, StreamExt};

        let RuntimePipeline {
            config: _config,
            receivers,
            processors,
            exporters,
            extensions,
            nodes: _nodes,
            channel_metrics,
            telemetry_policy,
        } = self;

        let metric_level = telemetry_policy.runtime_metrics;
        let node_interests = Interests::from_metric_level(metric_level);

        // Single-threaded runtime so we can drive !Send node tasks on the core thread.
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        let local_tasks = LocalSet::new();
        // ToDo create an optimized version of FuturesUnordered that can be used for !Send, !Sync tasks
        let mut futures = FuturesUnordered::new();

        // Build the extension monitor (or its no-op variant when
        // pipeline-internal telemetry is disabled) and pass it into
        // the lifecycle. The lifecycle records spawn/shutdown/
        // completion events and drives the monitor's `CollectTelemetry`
        // fan-out from its own tick cadence.
        //
        // Construction enters the runtime so `tokio::time::interval_at`
        // can attach to the timer driver; the guard is dropped before
        // `rt.block_on` so the runtime can be re-entered there.
        let ext_ctx = pipeline_context.extension_context();
        let ext_monitor = {
            let _enter = rt.enter();
            if telemetry_policy.pipeline_metrics {
                crate::extension_monitor::ExtensionMetricsMonitor::new(
                    ext_ctx.clone(),
                    EXTENSION_MONITOR_TICK_INTERVAL,
                    EXTENSION_MONITOR_COLLECT_TELEMETRY_INTERVAL,
                )
            } else {
                crate::extension_monitor::ExtensionMetricsMonitor::disabled(ext_ctx.clone())
            }
        };

        // Lifecycle invariant: "extensions start first, shut down last".
        // Concretely, `start()` is invoked on every active extension before
        // any data-path node task is spawned, and `Shutdown` is delivered
        // to extensions only after the data path has fully drained.
        //
        // NOTE: this orders *lifecycle calls*, not init completion.
        // `start()` is async, so invoking it merely enqueues a future onto
        // the LocalSet; the extension's init body runs concurrently with
        // the data path once polling begins. Capability *construction*
        // happens at build time (before any spawn), so structural wiring
        // is always in place — but if an extension performs deferred async
        // init in `start()` (opening a connection, loading config, warming
        // a cache), capability consumers may observe the pre-init state
        // until that work completes.
        //
        // Today, extensions handle this themselves (e.g., produce final
        // state at capability construction time, or have the capability
        // surface a not-ready error/default until init progresses).
        //
        // TODO: Revisit when an extension actually needs an init-complete
        // guarantee. Likely shape: opt-in readiness probe registered at
        // build time (e.g. `builder.with_readiness_probe()` returns a
        // handle the extension fires from `start()`); the engine awaits
        // only the registered probes via `try_join_all` before spawning
        // data-path tasks. Extensions that don't opt in keep today's
        // behavior with zero overhead.
        let mut extension_lifecycle = crate::extension_lifecycle::ExtensionLifecycle::spawn(
            extensions,
            &local_tasks,
            metrics_reporter.clone(),
            &ext_ctx,
            ext_monitor,
        );

        let mut control_senders = ControlSenders::default();
        let mut node_metric_entries: Vec<(usize, NodeMetricHandles)> = Vec::new();

        // Build a name→index map from NodeDefs so we can resolve flow_metric
        // config before processors are spawned.
        let node_name_to_index: HashMap<String, usize> = _nodes
            .iter()
            .map(|(nid, _)| (nid.name.to_string(), nid.index))
            .collect();

        // Collect local and shared processor node indices for flow_metric validation.
        let processor_indices: HashSet<usize> = processors
            .iter()
            .map(|p| match p {
                ProcessorWrapper::Local { node_id, .. }
                | ProcessorWrapper::Shared { node_id, .. } => node_id.index,
            })
            .collect();

        // Build edge list from pipeline connections for flow metric
        // interleaving detection.
        let pipeline_connections = connection_edges(_config.connection_iter(), &node_name_to_index);

        // Build flow_metric state and per-node role assignments up front.
        let flow_metric_state = build_flow_metric_state(
            &telemetry_policy,
            &node_name_to_index,
            &processor_indices,
            &pipeline_context,
            &pipeline_connections,
        )?;

        // Spawn node tasks and register their control senders, scoping telemetry where available.
        for exporter in exporters {
            let mut exporter = exporter;
            let node_id = exporter.node_id();
            control_senders.register(
                node_id.clone(),
                NodeType::Exporter,
                exporter.control_sender(),
            );
            let telemetry_guard = exporter.take_telemetry_guard();
            let node_entity_key = telemetry_guard.as_ref().map(|t| t.entity_key());
            let telemetry_handle = telemetry_guard.as_ref().map(|t| t.handle());
            let completion_emission_metrics =
                make_completion_emission_metrics(&telemetry_handle, metric_level);
            // Collect per-node metrics for the controller (exporters have no output channels).
            node_metric_entries.push((
                node_id.index,
                make_node_metric_handles(
                    &telemetry_handle,
                    &pipeline_context,
                    true,
                    false,
                    completion_emission_metrics.clone(),
                ),
            ));
            let runtime_ctrl_msg_tx = runtime_ctrl_msg_tx.clone();
            let pipeline_completion_msg_tx = pipeline_completion_msg_tx.clone();
            let effect_metrics_reporter = metrics_reporter.clone();
            let final_metrics_reporter = metrics_reporter.clone();
            let fut = async move {
                let result = exporter
                    .start_with_completion_metrics(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        effect_metrics_reporter,
                        node_interests,
                        completion_emission_metrics,
                    )
                    .await
                    .map(|terminal_state| {
                        report_terminal_metrics(&final_metrics_reporter, terminal_state);
                    });
                drop(telemetry_guard);
                result
            };
            if let Some(handle) = telemetry_handle {
                let input_key = handle.input_channel_key();
                let output_keys = handle.output_channel_keys();
                let node_ctx =
                    NodeTaskContext::new(node_entity_key, Some(handle), input_key, output_keys);
                futures.push(local_tasks.spawn_local(instrument_with_node_context(node_ctx, fut)));
            } else if let Some(key) = node_entity_key {
                let node_ctx = NodeTaskContext::new(Some(key), None, None, Vec::new());
                futures.push(local_tasks.spawn_local(instrument_with_node_context(node_ctx, fut)));
            } else {
                futures.push(local_tasks.spawn_local(fut));
            }
        }
        for processor in processors {
            let mut processor = processor;
            let node_id = processor.node_id();
            control_senders.register(
                node_id.clone(),
                NodeType::Processor,
                processor.control_sender(),
            );
            let telemetry_guard = processor.take_telemetry_guard();
            let node_entity_key = telemetry_guard.as_ref().map(|t| t.entity_key());
            let telemetry_handle = telemetry_guard.as_ref().map(|t| t.handle());
            let completion_emission_metrics =
                make_completion_emission_metrics(&telemetry_handle, metric_level);
            // Collect per-node metrics for the controller.
            node_metric_entries.push((
                node_id.index,
                make_node_metric_handles(
                    &telemetry_handle,
                    &pipeline_context,
                    true,
                    true,
                    completion_emission_metrics.clone(),
                ),
            ));
            let runtime_ctrl_msg_tx = runtime_ctrl_msg_tx.clone();
            let pipeline_completion_msg_tx = pipeline_completion_msg_tx.clone();
            let metrics_reporter = metrics_reporter.clone();
            // Extract flow metric roles for this processor node.
            let flow_is_start = flow_metric_state.start_nodes.contains_key(&node_id.index);
            let flow_is_end = flow_metric_state.end_nodes.contains_key(&node_id.index);
            let flow_signals_incoming_metric: Option<MetricSet<FlowSignalsIncomingMetrics>> =
                flow_metric_state
                    .start_nodes
                    .get(&node_id.index)
                    .and_then(|&id| flow_metric_state.signals_incoming_metrics[id].clone());
            let flow_duration_metric: Option<MetricSet<FlowDurationMetrics>> = flow_metric_state
                .end_nodes
                .get(&node_id.index)
                .and_then(|&id| flow_metric_state.duration_metrics[id].clone());
            let flow_signals_outgoing_metric: Option<MetricSet<FlowSignalsOutgoingMetrics>> =
                flow_metric_state
                    .end_nodes
                    .get(&node_id.index)
                    .and_then(|&id| flow_metric_state.signals_outgoing_metrics[id].clone());
            let flow_active = flow_metric_state.is_active();
            let fut = async move {
                let result = processor
                    .start_with_completion_metrics(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        metrics_reporter,
                        node_interests,
                        completion_emission_metrics,
                        flow_is_start,
                        flow_is_end,
                        flow_signals_incoming_metric,
                        flow_duration_metric,
                        flow_signals_outgoing_metric,
                        flow_active,
                    )
                    .await;
                drop(telemetry_guard);
                result
            };
            if let Some(handle) = telemetry_handle {
                let input_key = handle.input_channel_key();
                let output_keys = handle.output_channel_keys();
                let node_ctx =
                    NodeTaskContext::new(node_entity_key, Some(handle), input_key, output_keys);
                futures.push(local_tasks.spawn_local(instrument_with_node_context(node_ctx, fut)));
            } else if let Some(key) = node_entity_key {
                let node_ctx = NodeTaskContext::new(Some(key), None, None, Vec::new());
                futures.push(local_tasks.spawn_local(instrument_with_node_context(node_ctx, fut)));
            } else {
                futures.push(local_tasks.spawn_local(fut));
            }
        }
        for receiver in receivers {
            let mut receiver = receiver;
            let node_id = receiver.node_id();
            control_senders.register(
                node_id.clone(),
                NodeType::Receiver,
                receiver.control_sender(),
            );
            let telemetry_guard = receiver.take_telemetry_guard();
            let node_entity_key = telemetry_guard.as_ref().map(|t| t.entity_key());
            let telemetry_handle = telemetry_guard.as_ref().map(|t| t.handle());
            // Collect per-node metrics for the controller (receivers have no input data channel).
            node_metric_entries.push((
                node_id.index,
                make_node_metric_handles(&telemetry_handle, &pipeline_context, false, true, None),
            ));
            let runtime_ctrl_msg_tx = runtime_ctrl_msg_tx.clone();
            let pipeline_completion_msg_tx = pipeline_completion_msg_tx.clone();
            let effect_metrics_reporter = metrics_reporter.clone();
            let final_metrics_reporter = metrics_reporter.clone();
            let fut = async move {
                let result = receiver
                    .start(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        effect_metrics_reporter,
                        node_interests,
                    )
                    .await
                    .map(|terminal_state| {
                        report_terminal_metrics(&final_metrics_reporter, terminal_state);
                    });
                drop(telemetry_guard);
                result
            };
            if let Some(handle) = telemetry_handle {
                let input_key = handle.input_channel_key();
                let output_keys = handle.output_channel_keys();
                let node_ctx =
                    NodeTaskContext::new(node_entity_key, Some(handle), input_key, output_keys);
                futures.push(local_tasks.spawn_local(instrument_with_node_context(node_ctx, fut)));
            } else if let Some(key) = node_entity_key {
                let node_ctx = NodeTaskContext::new(Some(key), None, None, Vec::new());
                futures.push(local_tasks.spawn_local(instrument_with_node_context(node_ctx, fut)));
            } else {
                futures.push(local_tasks.spawn_local(fut));
            }
        }

        // Build the per-node metric handles table indexed by node_id.
        let max_node = node_metric_entries
            .iter()
            .map(|(id, _)| *id)
            .max()
            .unwrap_or(0);
        let mut node_metric_handles: Vec<Option<NodeMetricHandles>> =
            (0..=max_node).map(|_| None).collect();
        for (id, handles) in node_metric_entries {
            node_metric_handles[id] = Some(handles);
        }
        let node_metric_handles = Rc::new(RefCell::new(node_metric_handles));

        // Drop the original sender so the channel closes when all node tasks complete.
        // Each node holds its own clone and without this drop, the PipelinCtrlMsgManager
        // can only exit via a timeout.
        drop(runtime_ctrl_msg_tx);
        drop(pipeline_completion_msg_tx);

        // Spawn the control-plane task that routes node control messages to the pipeline engine.
        let return_control_senders = control_senders.clone();
        let return_node_metric_handles = node_metric_handles.clone();
        let final_node_metric_handles = node_metric_handles.clone();
        let final_metrics_reporter = metrics_reporter.clone();
        let manager_pipeline_context = pipeline_context.clone();
        let manager_metrics_reporter = metrics_reporter.clone();
        let manager_telemetry_policy = telemetry_policy.clone();
        let manager_memory_pressure_rx = memory_pressure_rx;
        let dispatcher_pipeline_context = pipeline_context.clone();
        let dispatcher_metrics_reporter = metrics_reporter.clone();
        let dispatcher_telemetry_policy = telemetry_policy.clone();
        futures.push(local_tasks.spawn_local(async move {
            let manager = RuntimeCtrlMsgManager::new(
                pipeline_key,
                manager_pipeline_context,
                runtime_ctrl_msg_rx,
                manager_memory_pressure_rx,
                control_senders,
                event_reporter,
                manager_metrics_reporter,
                control_plane_metrics_flush_interval,
                manager_telemetry_policy,
                channel_metrics,
                node_metric_handles,
            );
            manager.run().await
        }));

        futures.push(local_tasks.spawn_local(async move {
            let dispatcher = PipelineCompletionMsgDispatcher::new(
                dispatcher_pipeline_context,
                pipeline_completion_msg_rx,
                return_control_senders,
                return_node_metric_handles,
                dispatcher_metrics_reporter,
                control_plane_metrics_flush_interval,
                dispatcher_telemetry_policy,
            );
            dispatcher.run().await
        }));

        // Drive all local tasks until completion, returning the first error if any.
        // Data-path tasks (`futures`) and extension tasks (owned by
        // `extension_lifecycle`) run concurrently. When the data-path
        // drains, broadcast `Shutdown` to extensions so they can
        // terminate gracefully, then continue draining extension
        // futures.
        //
        // Errors from either side short-circuit out of the inner
        // async block. The outer code then unconditionally
        // broadcasts `Shutdown` and bounded-drains extensions before
        // propagating the error. This guarantees that extensions
        // owning sockets, files, or background work get the same
        // cleanup signal on the error path that they would on the
        // normal path, and that a misbehaving extension that ignores
        // `Shutdown` cannot hang the pipeline beyond
        // `EXTENSION_SHUTDOWN_GRACE`.
        rt.block_on(async {
            local_tasks
                .run_until(async {
                    // Inner async block isolates the loop's
                    // `return Err(...)` short-circuits from the
                    // outer cleanup, giving us an "async finally"
                    // shape: cleanup always runs, then the loop's
                    // result is propagated.
                    let loop_result: Result<Vec<_>, Error> = async {
                        let mut task_results = Vec::new();
                        loop {
                            // `biased;`: prefer data-path completions when both
                            // arms are simultaneously ready. Functionally
                            // optional — kept to make intent explicit.
                            tokio::select! {
                                biased;
                                Some(result) = futures.next(), if !futures.is_empty() => {
                                    match result {
                                        Ok(Ok(res)) => task_results.push(res),
                                        Ok(Err(e)) => return Err(e),
                                        Err(e) => return Err(Error::JoinTaskError {
                                            is_canceled: e.is_cancelled(),
                                            is_panic: e.is_panic(),
                                            error: e.to_string(),
                                        }),
                                    }
                                }
                                event = extension_lifecycle.next_event() => {
                                    match event {
                                        crate::extension_lifecycle::LifecycleEvent::Completion(result) => {
                                            match result {
                                                Ok(Ok(())) => {}
                                                Ok(Err(e)) => return Err(e),
                                                Err(e) => return Err(Error::JoinTaskError {
                                                    is_canceled: e.is_cancelled(),
                                                    is_panic: e.is_panic(),
                                                    error: e.to_string(),
                                                }),
                                            }
                                        }
                                        crate::extension_lifecycle::LifecycleEvent::MonitorTick(now) => {
                                            let mut reporter = metrics_reporter.clone();
                                            extension_lifecycle.monitor_tick(now, &mut reporter);
                                        }
                                    }
                                }
                                else => break,
                            }

                            // Once the data path is drained, broadcast
                            // `Shutdown` and exit the loop so the
                            // bounded `drain_until_deadline` below owns
                            // the wait for remaining extension tasks —
                            // staying in this `select!` would let a
                            // misbehaving extension hang the pipeline.
                            if futures.is_empty() {
                                extension_lifecycle
                                    .broadcast_shutdown(Some("pipeline data-path drained"))
                                    .await;
                                break;
                            }
                        }
                        Ok(task_results)
                    }
                    .await;

                    // "Async finally": unconditional cleanup that
                    // runs on both the normal and error paths.
                    // `broadcast_shutdown` is idempotent (latched by
                    // `shutdown_broadcast_fired`), so a no-op on the
                    // normal path; on the error path it ensures
                    // already-started extensions still receive
                    // `Shutdown` before the pipeline exits, so they
                    // can release sockets, files, or background work
                    // cleanly. `drain_until_deadline` then bounds the
                    // wait so a misbehaving extension can't hang the
                    // runtime.
                    extension_lifecycle
                        .broadcast_shutdown(Some("pipeline data-path drained"))
                        .await;
                    extension_lifecycle.drain_until_deadline().await;
                    // Final monitor flush so the host-scope aggregate
                    // and per-extension lifecycle counters reflect the
                    // terminal state (incl. any `ShutdownTimeout`
                    // entries) on the way out.
                    let mut final_monitor_reporter = metrics_reporter.clone();
                    extension_lifecycle
                        .monitor_tick(std::time::Instant::now(), &mut final_monitor_reporter);

                    let task_results = loop_result?;
                    let mut final_metrics_reporter = final_metrics_reporter.clone();
                    if let Err(err) = report_node_metrics_with_handles(
                        &final_node_metric_handles,
                        &mut final_metrics_reporter,
                    ) {
                        otap_df_telemetry::otel_warn!(
                            "node.metrics.final.reporting.fail",
                            error = err.to_string()
                        );
                    }
                    Ok(task_results)
                })
                .await
        })
    }
}

impl<PData: 'static + Debug + Clone> RuntimePipeline<PData> {
    /// Gets a reference to any node by its ID as a Node trait object
    #[must_use]
    pub fn get_node(&self, node_id: usize) -> Option<&dyn Node<PData>> {
        let ndef = self.nodes.get(node_id)?;

        match ndef.ntype {
            NodeType::Receiver => self
                .receivers
                .get(ndef.inner.index)
                .map(|r| r as &dyn Node<PData>),
            NodeType::Processor => self
                .processors
                .get(ndef.inner.index)
                .map(|p| p as &dyn Node<PData>),
            NodeType::Exporter => self
                .exporters
                .get(ndef.inner.index)
                .map(|e| e as &dyn Node<PData>),
        }
    }

    /// Gets a mutable NodeWithPDataSender reference (processors and receivers).
    #[must_use]
    pub fn get_mut_node_with_pdata_sender(
        &mut self,
        node_id: usize,
    ) -> Option<&mut dyn NodeWithPDataSender<PData>> {
        let ndef = self.nodes.get(node_id)?;

        match ndef.ntype {
            NodeType::Receiver => self
                .receivers
                .get_mut(ndef.inner.index)
                .map(|r| r as &mut dyn NodeWithPDataSender<PData>),
            NodeType::Processor => self
                .processors
                .get_mut(ndef.inner.index)
                .map(|p| p as &mut dyn NodeWithPDataSender<PData>),
            NodeType::Exporter => None,
        }
    }

    /// Gets a mutable NodeWithPDataReceiver reference (processors and exporters).
    #[must_use]
    pub fn get_mut_node_with_pdata_receiver(
        &mut self,
        node_id: usize,
    ) -> Option<&mut dyn NodeWithPDataReceiver<PData>> {
        let ndef = self.nodes.get(node_id)?;

        match ndef.ntype {
            NodeType::Receiver => None,
            NodeType::Processor => self
                .processors
                .get_mut(ndef.inner.index)
                .map(|p| p as &mut dyn NodeWithPDataReceiver<PData>),
            NodeType::Exporter => self
                .exporters
                .get_mut(ndef.inner.index)
                .map(|e| e as &mut dyn NodeWithPDataReceiver<PData>),
        }
    }

    /// Sends a node control message to the specified node.
    pub async fn send_node_control_message(
        &self,
        node_id: &NodeId,
        ctrl_msg: NodeControlMsg<PData>,
    ) -> Result<(), TypedError<NodeControlMsg<PData>>> {
        match self.nodes.get(node_id.index) {
            Some(ndef) => match ndef.ntype {
                NodeType::Receiver => {
                    self.receivers
                        .get(ndef.inner.index)
                        .expect("precomputed")
                        .send_control_msg(ctrl_msg)
                        .await
                }
                NodeType::Processor => {
                    self.processors
                        .get(ndef.inner.index)
                        .expect("precomputed")
                        .send_control_msg(ctrl_msg)
                        .await
                }
                NodeType::Exporter => {
                    self.exporters
                        .get(ndef.inner.index)
                        .expect("precomputed")
                        .send_control_msg(ctrl_msg)
                        .await
                }
            }
            .map_err(|e| TypedError::NodeControlMsgSendError {
                node_id: node_id.index,
                error: e,
            }),
            None => Err(TypedError::Error(Error::InternalError {
                message: format!("node {node_id:?}"),
            })),
        }
    }
}
