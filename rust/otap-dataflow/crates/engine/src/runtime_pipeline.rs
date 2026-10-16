// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of runtime pipeline configuration structures used by the engine and derived from the pipeline configuration.

use crate::Interests;
use crate::ReceivedAtNode;
use crate::Unwindable;
use crate::channel_metrics::{
    ChannelMetricsHandle, NodeCompletionMetrics, NodeInputItemMetrics, NodeInputMetrics,
    NodeInputSizeMetrics, NodeOutputItemMetrics, NodeOutputMetrics, NodeOutputSizeMetrics,
};
use crate::completion_emission_metrics::{
    CompletionEmissionMetricsHandle, make_completion_emission_metrics,
};
use crate::context::PipelineContext;
use crate::control::{
    ControlSenders, Controllable, NodeControlMsg, PipelineCompletionMsgReceiver,
    PipelineCompletionMsgSender, RuntimeCtrlMsgReceiver, RuntimeCtrlMsgSender,
};
use crate::entity_context::{
    NodeTaskContext, NodeTelemetryGuard, NodeTelemetryHandle, instrument_with_node_context,
};
use crate::error::{Error, TypedError};
use crate::flow_metrics::{
    FlowDroppedItemsMetrics, FlowDurationMetricSet, FlowInputItemsMetrics, FlowInputMessageMetrics,
    FlowInputSizeMetrics, FlowOutputItemsMetrics, FlowOutputMessageMetrics, FlowOutputSizeMetrics,
    build_flow_metric_state,
};
use crate::memory_limiter::MemoryPressureChanged;
use crate::node::{Node, NodeDefs, NodeId, NodeType, NodeWithPDataReceiver, NodeWithPDataSender};
use crate::pipeline_ctrl::{
    NodeMetricHandles, PipelineCompletionMsgDispatcher, RuntimeCtrlMsgManager,
    snapshot_node_metrics_with_handles,
};
use crate::processor::FlowMetricHook;
use crate::readiness::ReadinessProbeError;
use crate::runtime_services::PipelineRuntimeServices;
use crate::terminal_state::{TerminalMetricsDeadline, TerminalState};
use crate::{exporter::ExporterWrapper, processor::ProcessorWrapper, receiver::ReceiverWrapper};
use otel_arrow_dfe_config::DeployedPipelineKey;
use otel_arrow_dfe_config::pipeline::PipelineConfig;
use otel_arrow_dfe_config::policy::TelemetryPolicy;
use otel_arrow_dfe_telemetry::event::ObservedEventReporter;
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::{MetricsReporter, ReportOutcome};
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

/// Build output-message metric sets indexed by sorted output port name,
/// matching the `output_port_index` layout used in `RouteData`.
fn make_output_metrics(
    telemetry_handle: &Option<NodeTelemetryHandle>,
    pipeline_context: &PipelineContext,
) -> Vec<MeasurementMetricSet<NodeOutputMetrics>> {
    telemetry_handle
        .as_ref()
        .map(|h| {
            let mut keys = h.output_channel_keys();
            keys.sort_by(|a, b| a.0.cmp(&b.0));
            keys.iter()
                .map(|(_, key)| {
                    NodeOutputMetrics::register(
                        &pipeline_context.metric_set_registrar_for_entity(*key),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Build completion-duration metric sets indexed by sorted output port name.
fn make_output_completion_metrics(
    telemetry_handle: &Option<NodeTelemetryHandle>,
    pipeline_context: &PipelineContext,
) -> Vec<MeasurementMetricSet<NodeCompletionMetrics>> {
    telemetry_handle
        .as_ref()
        .map(|h| {
            let mut keys = h.output_channel_keys();
            keys.sort_by(|a, b| a.0.cmp(&b.0));
            keys.iter()
                .map(|(_, key)| {
                    NodeCompletionMetrics::register(
                        &pipeline_context.metric_set_registrar_for_entity(*key),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Build optional output-item metric sets indexed by sorted output port name.
fn make_output_item_metrics(
    telemetry_handle: &Option<NodeTelemetryHandle>,
    pipeline_context: &PipelineContext,
) -> Vec<MeasurementMetricSet<NodeOutputItemMetrics>> {
    telemetry_handle
        .as_ref()
        .map(|h| {
            let mut keys = h.output_channel_keys();
            keys.sort_by(|a, b| a.0.cmp(&b.0));
            keys.iter()
                .map(|(_, key)| {
                    NodeOutputItemMetrics::register(
                        &pipeline_context.metric_set_registrar_for_entity(*key),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Build optional output-size metric sets indexed by sorted output port name.
fn make_output_size_metrics(
    telemetry_handle: &Option<NodeTelemetryHandle>,
    pipeline_context: &PipelineContext,
) -> Vec<MeasurementMetricSet<NodeOutputSizeMetrics>> {
    telemetry_handle
        .as_ref()
        .map(|h| {
            let mut keys = h.output_channel_keys();
            keys.sort_by(|a, b| a.0.cmp(&b.0));
            keys.iter()
                .map(|(_, key)| {
                    NodeOutputSizeMetrics::register(
                        &pipeline_context.metric_set_registrar_for_entity(*key),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

/// Build per-node metric handles for the runtime control manager.
///
/// - `has_input`: whether to register input metrics (false for receivers).
/// - `has_outputs`: whether to register output metrics (false for exporters).
/// - `node_interests`: the effective metric interests for this node.
fn make_node_metric_handles(
    telemetry_handle: &Option<NodeTelemetryHandle>,
    pipeline_context: &PipelineContext,
    has_input: bool,
    has_outputs: bool,
    node_interests: Interests,
    completion_emission: Option<CompletionEmissionMetricsHandle>,
) -> NodeMetricHandles {
    let input_metrics_enabled = has_input && node_interests.contains(Interests::NODE_INPUT_METRICS);
    let output_metrics_enabled =
        has_outputs && node_interests.contains(Interests::NODE_OUTPUT_METRICS);
    let completion_duration_enabled = node_interests.contains(Interests::NODE_COMPLETION_DURATION);
    let item_counts_enabled = node_interests.contains(Interests::NODE_ITEM_COUNTS);
    let size_enabled = node_interests.contains(Interests::NODE_SIZE);

    let input = if input_metrics_enabled {
        telemetry_handle
            .as_ref()
            .and_then(|h| h.input_channel_key())
            .map(|key| {
                NodeInputMetrics::register(&pipeline_context.metric_set_registrar_for_entity(key))
            })
    } else {
        None
    };
    let input_completion = if completion_duration_enabled && has_input {
        telemetry_handle
            .as_ref()
            .and_then(|h| h.input_channel_key())
            .map(|key| {
                NodeCompletionMetrics::register(
                    &pipeline_context.metric_set_registrar_for_entity(key),
                )
            })
    } else {
        None
    };
    let input_size = if size_enabled && has_input {
        telemetry_handle
            .as_ref()
            .and_then(|h| h.input_channel_key())
            .map(|key| {
                NodeInputSizeMetrics::register(
                    &pipeline_context.metric_set_registrar_for_entity(key),
                )
            })
    } else {
        None
    };
    let input_items = if item_counts_enabled && has_input {
        telemetry_handle
            .as_ref()
            .and_then(|h| h.input_channel_key())
            .map(|key| {
                NodeInputItemMetrics::register(
                    &pipeline_context.metric_set_registrar_for_entity(key),
                )
            })
    } else {
        None
    };
    let outputs = if output_metrics_enabled {
        make_output_metrics(telemetry_handle, pipeline_context)
    } else {
        Vec::new()
    };
    let output_completion = if completion_duration_enabled && has_outputs && !has_input {
        make_output_completion_metrics(telemetry_handle, pipeline_context)
    } else {
        Vec::new()
    };
    let output_items = if item_counts_enabled && has_outputs {
        make_output_item_metrics(telemetry_handle, pipeline_context)
    } else {
        Vec::new()
    };
    let output_size = if size_enabled && has_outputs {
        make_output_size_metrics(telemetry_handle, pipeline_context)
    } else {
        Vec::new()
    };
    NodeMetricHandles {
        registry: pipeline_context.metrics_registry(),
        input,
        input_completion,
        input_items,
        input_size,
        outputs,
        output_completion,
        output_items,
        output_size,
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
        otel_arrow_dfe_telemetry::registry::EntityKey,
    )>,

    /// A precomputed map of all node IDs to their Node trait objects (? @@@) for efficient access
    /// Indexed by NodeIndex
    nodes: NodeDefs<PData, PipeNode>,
    /// Channel metrics handles collected during build.
    channel_metrics: Vec<ChannelMetricsHandle>,
    /// Admission refusal metrics handles collected during node construction.
    admission_metrics: Vec<crate::admission::metrics::AdmissionMetricsHandle>,
    /// Flags controlling pipeline-internal metrics collection/reporting.
    telemetry_policy: TelemetryPolicy,
    /// Optional one-shot notification fired at the runtime-ready boundary.
    startup_callback: Option<Box<dyn FnOnce()>>,
    #[cfg(test)]
    readiness_drop_callback: Option<Box<dyn FnMut()>>,
}

async fn flush_metrics_reporter(
    metrics_reporter: &MetricsReporter,
    phase: &'static str,
    terminal_metrics_deadline: &TerminalMetricsDeadline,
) {
    if let Err(err) = metrics_reporter
        .flush_until(terminal_metrics_deadline.get())
        .await
    {
        otel_arrow_dfe_telemetry::otel_warn!(
            "metrics.collection.flush.fail",
            phase,
            error = err.to_string()
        );
    }
}

/// Reports snapshots carried by a terminal state within its shutdown deadline.
pub(crate) async fn report_terminal_metrics(
    metrics_reporter: &MetricsReporter,
    terminal_state: TerminalState,
    terminal_metrics_deadline: &TerminalMetricsDeadline,
) {
    let deadline = terminal_state.deadline();
    terminal_metrics_deadline.record(deadline);
    report_metric_snapshots(
        metrics_reporter,
        terminal_state.into_metrics(),
        "terminal",
        terminal_metrics_deadline.get(),
    )
    .await;
}

/// Best-effort reports a sequence of snapshots and flushes the collector.
///
/// Reporting continues after individual failures, but every operation shares
/// one absolute `deadline` so a long sequence cannot extend shutdown.
async fn report_metric_snapshots(
    metrics_reporter: &MetricsReporter,
    snapshots: impl IntoIterator<Item = MetricSetSnapshot>,
    phase: &'static str,
    deadline: std::time::Instant,
) {
    for snapshot in snapshots {
        match metrics_reporter
            .report_snapshot_reliably_until(snapshot, deadline)
            .await
        {
            Ok(ReportOutcome::Sent) => {}
            Ok(ReportOutcome::Deferred) => {
                otel_arrow_dfe_telemetry::otel_warn!(
                    "metrics.terminal.reporting.deferred",
                    phase,
                    message = "Terminal metric snapshot was deferred because the standalone reporter channel is full"
                );
            }
            Err(err) => {
                otel_arrow_dfe_telemetry::otel_warn!(
                    "metrics.terminal.reporting.fail",
                    phase,
                    error = err.to_string()
                );
            }
        }
    }
    if let Err(err) = metrics_reporter.flush_until(deadline).await {
        otel_arrow_dfe_telemetry::otel_warn!(
            "metrics.collection.flush.fail",
            phase,
            error = err.to_string()
        );
    }
}

/// Build a flat edge list from a pipeline's connections, resolving node
/// names to indices. Names not found in the map are silently skipped.
fn connection_edges<'a>(
    connections: impl Iterator<Item = &'a otel_arrow_dfe_config::pipeline::PipelineConnection>,
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
            otel_arrow_dfe_telemetry::registry::EntityKey,
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
            admission_metrics: Default::default(),
            telemetry_policy,
            startup_callback: None,
            #[cfg(test)]
            readiness_drop_callback: None,
        }
    }

    /// Registers a one-shot callback for successful runtime initialization.
    #[must_use]
    pub fn with_startup_callback(mut self, callback: impl FnOnce() + 'static) -> Self {
        self.startup_callback = Some(Box::new(callback));
        self
    }

    #[cfg(test)]
    #[must_use]
    fn with_readiness_drop_callback(mut self, callback: impl FnMut() + 'static) -> Self {
        self.readiness_drop_callback = Some(Box::new(callback));
        self
    }

    pub(crate) fn set_channel_metrics(&mut self, channel_metrics: Vec<ChannelMetricsHandle>) {
        self.channel_metrics = channel_metrics;
    }

    pub(crate) fn set_admission_metrics(
        &mut self,
        admission_metrics: Vec<crate::admission::metrics::AdmissionMetricsHandle>,
    ) {
        self.admission_metrics = admission_metrics;
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
            config: pipeline_config,
            receivers,
            processors,
            exporters,
            extensions,
            nodes: _nodes,
            channel_metrics,
            admission_metrics,
            telemetry_policy,
            mut startup_callback,
            #[cfg(test)]
            mut readiness_drop_callback,
        } = self;

        let metric_level = telemetry_policy.runtime_metrics;
        // Single-threaded runtime so we can drive !Send node tasks on the core thread.
        let rt = Builder::new_current_thread()
            .enable_all()
            .build()
            .expect("Failed to create runtime");
        let local_tasks = LocalSet::new();
        let runtime_services = PipelineRuntimeServices::new(Default::default())?;
        // ToDo create an optimized version of FuturesUnordered that can be used for !Send, !Sync tasks
        let mut futures = FuturesUnordered::new();
        let mut receiver_readiness = FuturesUnordered::new();

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

        // Lifecycle invariant: extensions start first, shut down last.
        // `start()` is invoked on every active extension before any
        // data-path node task is spawned, and `Shutdown` is delivered
        // to extensions only after the data path has fully drained.
        //
        // Extensions that need to gate node startup on runtime readiness
        // opt in via `builder.active().with_readiness_probe()` (or
        // `.with_readiness_probe_timeout_override(t)`) and fire it
        // from `start()` via `EffectHandler::signal_ready()`. The engine
        // awaits the registered probes via `wait_all_ready` before
        // spawning data-path tasks.
        let terminal_metrics_deadline = TerminalMetricsDeadline::default();
        let (forced_shutdown_trigger, forced_shutdown_signal) =
            crate::forced_shutdown::ForcedShutdownTrigger::pair();
        let mut extension_lifecycle = crate::extension_lifecycle::ExtensionLifecycle::spawn(
            extensions,
            &local_tasks,
            metrics_reporter.clone(),
            terminal_metrics_deadline.clone(),
            &ext_ctx,
            ext_monitor,
        );

        if let Err(barrier_err) =
            rt.block_on(local_tasks.run_until(extension_lifecycle.wait_all_spawned()))
        {
            extension_lifecycle.initiate_shutdown(Some("spawn barrier failed"));
            rt.block_on(local_tasks.run_until(extension_lifecycle.drain_until_deadline()));
            return Err(barrier_err);
        }

        // TODO: make the readiness gate interruptible by an operator stop.
        // `wait_all_ready` blocks here (bounded by each probe's timeout) before
        // the runtime control-message manager is spawned, so a shutdown request
        // that arrives during startup is only handled once the gate resolves or
        // times out. With a long timeout override this can delay an operator
        // stop. Plumbing the runtime control receiver into this wait is a
        // follow-up; today the per-probe timeout bounds the worst-case wait.
        if let Err(readiness_err) =
            rt.block_on(local_tasks.run_until(extension_lifecycle.wait_all_ready()))
        {
            extension_lifecycle.initiate_shutdown(Some("extension readiness gate failed"));
            rt.block_on(local_tasks.run_until(extension_lifecycle.drain_until_deadline()));
            return Err(readiness_err);
        }

        let mut control_senders = ControlSenders::default();
        let mut node_metric_entries: Vec<(usize, NodeMetricHandles)> = Vec::new();
        let mut node_telemetry_guards: Vec<NodeTelemetryGuard> = Vec::new();

        // Build a name->index map from NodeDefs so we can resolve flow_metric
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
        let pipeline_connections =
            connection_edges(pipeline_config.connection_iter(), &node_name_to_index);

        // Build flow_metric state and per-node role assignments up front.
        let mut flow_metric_state = build_flow_metric_state(
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
            let node_config = pipeline_config
                .nodes()
                .get(node_id.name.as_ref())
                .expect("runtime exporter has pipeline configuration");
            let node_interests = Interests::for_node(metric_level, node_config);
            control_senders.register(
                node_id.clone(),
                NodeType::Exporter,
                exporter.control_sender(),
            );
            let telemetry_guard = exporter.take_telemetry_guard();
            let node_entity_key = telemetry_guard.as_ref().map(|t| t.entity_key());
            let telemetry_handle = telemetry_guard.as_ref().map(|t| t.handle());
            node_telemetry_guards.extend(telemetry_guard);
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
                    node_interests,
                    completion_emission_metrics.clone(),
                ),
            ));
            let runtime_ctrl_msg_tx = runtime_ctrl_msg_tx.clone();
            let pipeline_completion_msg_tx = pipeline_completion_msg_tx.clone();
            let effect_metrics_reporter = metrics_reporter.clone();
            let final_metrics_reporter = metrics_reporter.clone();
            let exporter_terminal_metrics_deadline = terminal_metrics_deadline.clone();
            let exporter_runtime_services = runtime_services.clone();
            let fut = async move {
                match exporter
                    .start_with_completion_metrics(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        effect_metrics_reporter,
                        node_interests,
                        completion_emission_metrics,
                        exporter_runtime_services,
                    )
                    .await
                {
                    Ok(terminal_state) => {
                        report_terminal_metrics(
                            &final_metrics_reporter,
                            terminal_state,
                            &exporter_terminal_metrics_deadline,
                        )
                        .await;
                        Ok(())
                    }
                    Err(err) => {
                        flush_metrics_reporter(
                            &final_metrics_reporter,
                            "terminal_error",
                            &exporter_terminal_metrics_deadline,
                        )
                        .await;
                        Err(err)
                    }
                }
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
            let node_config = pipeline_config
                .nodes()
                .get(node_id.name.as_ref())
                .expect("runtime processor has pipeline configuration");
            let node_interests = Interests::for_node(metric_level, node_config);
            control_senders.register(
                node_id.clone(),
                NodeType::Processor,
                processor.control_sender(),
            );
            let telemetry_guard = processor.take_telemetry_guard();
            let node_entity_key = telemetry_guard.as_ref().map(|t| t.entity_key());
            let telemetry_handle = telemetry_guard.as_ref().map(|t| t.handle());
            node_telemetry_guards.extend(telemetry_guard);
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
                    node_interests,
                    completion_emission_metrics.clone(),
                ),
            ));
            let runtime_ctrl_msg_tx = runtime_ctrl_msg_tx.clone();
            let pipeline_completion_msg_tx = pipeline_completion_msg_tx.clone();
            let metrics_reporter = metrics_reporter.clone();
            let final_metrics_reporter = metrics_reporter.clone();
            let processor_terminal_metrics_deadline = terminal_metrics_deadline.clone();
            let processor_forced_shutdown_signal = forced_shutdown_signal.clone();
            let processor_runtime_services = runtime_services.clone();
            // Extract flow metric roles for this processor node.
            // Compute pipeline-wide flags before moving metric sets into handlers.
            let flow_active = flow_metric_state.is_active();
            let flow_needs_timing = flow_metric_state.needs_timing();
            let flow_is_start = flow_metric_state.start_nodes.contains_key(&node_id.index);
            let flow_is_end = flow_metric_state.end_nodes.contains_key(&node_id.index);
            let flow_input_message_metric: Option<MeasurementMetricSet<FlowInputMessageMetrics>> =
                flow_metric_state
                    .start_nodes
                    .get(&node_id.index)
                    .and_then(|&id| flow_metric_state.input_message_metrics[id].take());
            let flow_input_items_metric: Option<MeasurementMetricSet<FlowInputItemsMetrics>> =
                flow_metric_state
                    .start_nodes
                    .get(&node_id.index)
                    .and_then(|&id| flow_metric_state.input_items_metrics[id].take());
            let flow_input_size_metric: Option<MeasurementMetricSet<FlowInputSizeMetrics>> =
                flow_metric_state
                    .start_nodes
                    .get(&node_id.index)
                    .and_then(|&id| flow_metric_state.input_size_metrics[id].take());
            let flow_duration_metric: Option<FlowDurationMetricSet> = flow_metric_state
                .end_nodes
                .get(&node_id.index)
                .and_then(|&id| flow_metric_state.duration_metrics[id].take());
            let flow_output_items_metric: Option<MeasurementMetricSet<FlowOutputItemsMetrics>> =
                flow_metric_state
                    .end_nodes
                    .get(&node_id.index)
                    .and_then(|&id| flow_metric_state.output_items_metrics[id].take());
            let flow_output_message_metric: Option<MeasurementMetricSet<FlowOutputMessageMetrics>> =
                flow_metric_state
                    .end_nodes
                    .get(&node_id.index)
                    .and_then(|&id| flow_metric_state.output_message_metrics[id].take());
            let flow_output_size_metric: Option<MeasurementMetricSet<FlowOutputSizeMetrics>> =
                flow_metric_state
                    .end_nodes
                    .get(&node_id.index)
                    .and_then(|&id| flow_metric_state.output_size_metrics[id].take());
            // Register per-node drop decision metric set when this
            // processor declares the drop capability and lies within a
            // flow that enables dropped. Each decision node gets its own
            // entity tagged with `flow.node.decision` = this node's name.
            let mut flow_dropped_items_metric: Option<
                MeasurementMetricSet<FlowDroppedItemsMetrics>,
            > = None;
            if processor.runtime_requirements().makes_drop_decisions
                && let Some(candidate) = flow_metric_state.decision_candidates.get(&node_id.index)
            {
                let mut attrs = candidate.attrs.clone();
                attrs.decision = std::borrow::Cow::Owned(node_id.name.to_string());
                let entity_key = pipeline_context.metrics_registry().register_entity(attrs);
                flow_dropped_items_metric = Some(FlowDroppedItemsMetrics::register(
                    &pipeline_context.metric_set_registrar_for_entity(entity_key),
                ));
            }
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
                        flow_input_message_metric,
                        flow_input_items_metric,
                        flow_input_size_metric,
                        flow_duration_metric,
                        flow_output_items_metric,
                        flow_output_message_metric,
                        flow_output_size_metric,
                        flow_dropped_items_metric,
                        flow_active,
                        flow_needs_timing,
                        processor_terminal_metrics_deadline.clone(),
                        processor_forced_shutdown_signal,
                        processor_runtime_services,
                    )
                    .await;
                flush_metrics_reporter(
                    &final_metrics_reporter,
                    "terminal",
                    &processor_terminal_metrics_deadline,
                )
                .await;
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
            let startup_error_slot = if let Some(probe) = receiver.take_startup_readiness_probe() {
                let error_slot = Rc::new(RefCell::new(None));
                let readiness_node_id = node_id.clone();
                let readiness_error_slot = Rc::clone(&error_slot);
                receiver_readiness.push(async move {
                    (
                        readiness_node_id,
                        probe.wait_ready().await,
                        readiness_error_slot,
                    )
                });
                Some(error_slot)
            } else {
                None
            };
            let node_config = pipeline_config
                .nodes()
                .get(node_id.name.as_ref())
                .expect("runtime receiver has pipeline configuration");
            let node_interests = Interests::for_node(metric_level, node_config);
            control_senders.register(
                node_id.clone(),
                NodeType::Receiver,
                receiver.control_sender(),
            );
            let telemetry_guard = receiver.take_telemetry_guard();
            let node_entity_key = telemetry_guard.as_ref().map(|t| t.entity_key());
            let telemetry_handle = telemetry_guard.as_ref().map(|t| t.handle());
            node_telemetry_guards.extend(telemetry_guard);
            // Collect per-node metrics for the controller (receivers have no input data channel).
            node_metric_entries.push((
                node_id.index,
                make_node_metric_handles(
                    &telemetry_handle,
                    &pipeline_context,
                    false,
                    true,
                    node_interests,
                    None,
                ),
            ));
            let runtime_ctrl_msg_tx = runtime_ctrl_msg_tx.clone();
            let pipeline_completion_msg_tx = pipeline_completion_msg_tx.clone();
            let effect_metrics_reporter = metrics_reporter.clone();
            let final_metrics_reporter = metrics_reporter.clone();
            let receiver_terminal_metrics_deadline = terminal_metrics_deadline.clone();
            let receiver_runtime_services = runtime_services.clone();
            let fut = async move {
                match receiver
                    .start(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        effect_metrics_reporter,
                        node_interests,
                        receiver_runtime_services,
                    )
                    .await
                {
                    Ok(terminal_state) => {
                        report_terminal_metrics(
                            &final_metrics_reporter,
                            terminal_state,
                            &receiver_terminal_metrics_deadline,
                        )
                        .await;
                        Ok(())
                    }
                    Err(err) => match startup_error_slot {
                        Some(error_slot) => {
                            *error_slot.borrow_mut() = Some(err);
                            flush_metrics_reporter(
                                &final_metrics_reporter,
                                "terminal_error",
                                &receiver_terminal_metrics_deadline,
                            )
                            .await;
                            match error_slot.borrow_mut().take() {
                                Some(err) => Err(err),
                                None => Ok(()),
                            }
                        }
                        None => {
                            flush_metrics_reporter(
                                &final_metrics_reporter,
                                "terminal_error",
                                &receiver_terminal_metrics_deadline,
                            )
                            .await;
                            Err(err)
                        }
                    },
                }
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
        let final_channel_metrics = channel_metrics.clone();
        let final_admission_metrics = admission_metrics.clone();
        let final_metrics_reporter = metrics_reporter.clone();
        let manager_pipeline_context = pipeline_context.clone();
        let manager_metrics_reporter = metrics_reporter.clone();
        let manager_telemetry_policy = telemetry_policy.clone();
        let manager_memory_pressure_rx = memory_pressure_rx;
        let manager_terminal_metrics_deadline = terminal_metrics_deadline.clone();
        let dispatcher_pipeline_context = pipeline_context.clone();
        let dispatcher_metrics_reporter = metrics_reporter.clone();
        let dispatcher_telemetry_policy = telemetry_policy.clone();
        let dispatcher_terminal_metrics_deadline = terminal_metrics_deadline.clone();
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
                admission_metrics,
                node_metric_handles,
                manager_terminal_metrics_deadline,
                forced_shutdown_trigger,
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
                dispatcher_terminal_metrics_deadline,
            );
            dispatcher.run().await
        }));

        // Drive data-path and extension tasks concurrently. On both the
        // happy and error paths, cleanup unconditionally broadcasts
        // `Shutdown` and bounded-drains extensions (see
        // `EXTENSION_SHUTDOWN_GRACE`). The "extensions stop last"
        // guarantee applies to the normal drain path only; see the
        // `extension_lifecycle` module docs.
        let result = rt.block_on(async {
            local_tasks
                .run_until(async {
                    let loop_result: Result<Vec<_>, Error> = async {
                        let mut task_results = Vec::new();
                        let mut startup_complete = receiver_readiness.is_empty();
                        if startup_complete
                            && let Some(callback) = startup_callback.take()
                        {
                            callback();
                        }

                        loop {
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
                                Some((receiver, result, startup_error_slot)) = receiver_readiness.next(),
                                    if !receiver_readiness.is_empty() =>
                                {
                                    if let Err(ReadinessProbeError::SignallerDropped) = result {
                                        #[cfg(test)]
                                        if let Some(callback) = readiness_drop_callback.as_mut() {
                                            callback();
                                        }
                                        if let Some(error) = startup_error_slot.borrow_mut().take() {
                                            return Err(error);
                                        }
                                        return Err(Error::ReceiverReadinessSignallerDropped {
                                            receiver,
                                        });
                                    }
                                }
                                else => break,
                            }

                            if !startup_complete && receiver_readiness.is_empty() {
                                startup_complete = true;
                                if let Some(callback) = startup_callback.take() {
                                    callback();
                                }
                            }

                            // Data path drained: hand off the bounded wait
                            // for remaining extension tasks to
                            // `drain_until_deadline` below.
                            if startup_complete && futures.is_empty() {
                                extension_lifecycle
                                    .initiate_shutdown(Some("pipeline data-path drained"));
                                break;
                            }
                        }
                        Ok(task_results)
                    }
                    .await;

                    // Unconditional cleanup. `initiate_shutdown` is
                    // idempotent (no-op on the happy path).
                    extension_lifecycle
                        .initiate_shutdown(Some("pipeline data-path drained"));
                    extension_lifecycle.drain_until_deadline().await;
                    // Final monitor flush so per-extension counters reflect
                    // any terminal `ShutdownTimeout` entries on the way out.
                    let mut final_monitor_reporter = metrics_reporter.clone();
                    extension_lifecycle
                        .monitor_tick(std::time::Instant::now(), &mut final_monitor_reporter);
                    if let Err(err) = extension_lifecycle
                        .finish_metrics_reporting_until(
                            &final_monitor_reporter,
                            terminal_metrics_deadline.get(),
                        )
                        .await
                    {
                        otel_arrow_dfe_telemetry::otel_warn!(
                            "extension.lifecycle.metrics.final_reporting.fail",
                            error = err.to_string()
                        );
                    }

                    let final_snapshots = snapshot_node_metrics_with_handles(
                        &final_node_metric_handles,
                    )
                    .into_iter()
                    .chain(
                        final_channel_metrics
                            .iter()
                            .flat_map(ChannelMetricsHandle::terminal_snapshots),
                    )
                    .chain(
                        final_admission_metrics
                            .iter()
                            .flat_map(|metrics| metrics.terminal_snapshots()),
                    );
                    report_metric_snapshots(
                        &final_metrics_reporter,
                        final_snapshots,
                        "pipeline_final",
                        terminal_metrics_deadline.get(),
                    )
                    .await;

                    let task_results = loop_result?;
                    Ok(task_results)
                })
                .await
        });
        drop(node_telemetry_guards);
        result
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

#[cfg(test)]
mod tests {
    use std::sync::{Arc, mpsc};
    use std::thread;

    use super::*;
    use crate::attributes::{
        ChannelImplementation, ChannelKind, ChannelMode, ChannelType, EngineEntityAttributeSet,
    };
    use crate::channel_metrics::ChannelSenderMetrics;
    use crate::config::{ExporterConfig, ReceiverConfig};
    use crate::context::ControllerContext;
    use crate::control::{pipeline_completion_msg_channel, runtime_ctrl_msg_channel};
    use crate::entity_context::{NodeTelemetryGuard, NodeTelemetryHandle};
    use crate::exporter::ExporterWrapper;
    use crate::local::{exporter as local_exporter, receiver as local_receiver};
    use crate::message::ExporterInbox;
    use crate::receiver::ReceiverWrapper;
    use crate::terminal_state::TerminalState;
    use crate::wiring_contract::WiringContract;
    use crate::{ExporterFactory, PipelineFactory, ReceiverFactory};
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_config::observed_state::{ObservedStateSettings, SendPolicy};
    use otel_arrow_dfe_config::pipeline::PipelineConfig;
    use otel_arrow_dfe_config::pipeline::telemetry::TelemetryConfig;
    use otel_arrow_dfe_config::policy::{ChannelCapacityPolicy, TelemetryPolicy};
    use otel_arrow_dfe_config::{DeployedPipelineKey, PipelineGroupId, PipelineId, SignalType};
    use otel_arrow_dfe_state::store::ObservedStateStore;
    use otel_arrow_dfe_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use otel_arrow_dfe_telemetry::{InternalTelemetrySystem, LogContext};
    use tokio::time::{Duration, timeout};
    use tokio_util::sync::CancellationToken;

    const STARTUP_FAILURE_MESSAGE: &str = "distinctive receiver startup failure";

    struct StartupFailingReceiver;

    #[async_trait::async_trait(?Send)]
    impl local_receiver::Receiver<()> for StartupFailingReceiver {
        async fn start(
            self: Box<Self>,
            _ctrl: local_receiver::ControlChannel<()>,
            effect_handler: local_receiver::EffectHandler<()>,
        ) -> Result<TerminalState, Error> {
            let receiver = effect_handler.receiver_id();
            Err(Error::ReceiverError {
                receiver,
                kind: crate::error::ReceiverErrorKind::Connect,
                error: STARTUP_FAILURE_MESSAGE.to_owned(),
                source_detail: "test receiver failed before signaling readiness".to_owned(),
            })
        }
    }

    struct PendingExporter;

    #[async_trait::async_trait(?Send)]
    impl local_exporter::Exporter<()> for PendingExporter {
        async fn start(
            self: Box<Self>,
            _inbox: ExporterInbox<()>,
            _effect_handler: local_exporter::EffectHandler<()>,
        ) -> Result<TerminalState, Error> {
            std::future::pending().await
        }
    }

    fn create_startup_failing_receiver(
        _pipeline_ctx: PipelineContext,
        node: NodeId,
        node_config: Arc<NodeUserConfig>,
        receiver_config: &ReceiverConfig,
        _capabilities: &crate::capability::registry::Capabilities,
    ) -> Result<ReceiverWrapper<()>, otel_arrow_dfe_config::error::Error> {
        Ok(
            ReceiverWrapper::local(StartupFailingReceiver, node, node_config, receiver_config)
                .with_startup_readiness(),
        )
    }

    fn create_pending_exporter(
        _pipeline_ctx: PipelineContext,
        node: NodeId,
        node_config: Arc<NodeUserConfig>,
        exporter_config: &ExporterConfig,
        _capabilities: &crate::capability::registry::Capabilities,
    ) -> Result<ExporterWrapper<()>, otel_arrow_dfe_config::error::Error> {
        Ok(ExporterWrapper::local(
            PendingExporter,
            node,
            node_config,
            exporter_config,
        ))
    }

    static STARTUP_FAILURE_RECEIVER_FACTORIES: &[ReceiverFactory<()>] = &[ReceiverFactory {
        name: "urn:test:receiver:startup-failure",
        create: create_startup_failing_receiver,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: otel_arrow_dfe_config::validation::no_config,
    }];

    static PENDING_EXPORTER_FACTORIES: &[ExporterFactory<()>] = &[ExporterFactory {
        name: "urn:test:exporter:pending",
        create: create_pending_exporter,
        context_declarations: None,
        wiring_contract: WiringContract::UNRESTRICTED,
        validate_config: otel_arrow_dfe_config::validation::no_config,
    }];

    /// Scenario: completion duration, item counts, and size are enabled without message interests.
    /// Guarantees: optional node measurements register independently without input/output message sets.
    #[test]
    fn optional_node_metrics_do_not_require_direction_interests() {
        let (pipeline_context, registry) = crate::testing::test_pipeline_ctx();
        let node_entity_key = pipeline_context.register_node_entity();
        let telemetry_handle = NodeTelemetryHandle::new(registry.clone(), node_entity_key);

        let input_key = pipeline_context.register_node_channel_entity(
            "input-channel".into(),
            "input".into(),
            ChannelKind::Pdata,
            ChannelMode::Local,
            ChannelType::Mpsc,
            ChannelImplementation::Internal,
        );
        telemetry_handle.set_input_channel_key(input_key);
        let output_key = pipeline_context.register_node_channel_entity(
            "output-channel".into(),
            "default".into(),
            ChannelKind::Pdata,
            ChannelMode::Local,
            ChannelType::Mpsc,
            ChannelImplementation::Internal,
        );
        telemetry_handle.add_output_channel_key("default".into(), output_key);

        let handles = make_node_metric_handles(
            &Some(telemetry_handle),
            &pipeline_context,
            true,
            true,
            Interests::NODE_COMPLETION_DURATION
                | Interests::NODE_ITEM_COUNTS
                | Interests::NODE_SIZE,
            None,
        );

        assert!(handles.input.is_none());
        assert!(handles.input_completion.is_some());
        assert!(handles.input_items.is_some());
        assert!(handles.input_size.is_some());
        assert!(handles.outputs.is_empty());
        assert!(handles.output_completion.is_empty());
        assert_eq!(handles.output_items.len(), 1);
        assert_eq!(handles.output_size.len(), 1);
        assert_eq!(registry.metric_set_count(), 5);
    }

    /// Scenario: a pipeline has pending terminal metrics when telemetry cleanup starts.
    /// Guarantees: the final snapshot reaches the registry before entity handles are released.
    #[tokio::test(flavor = "current_thread")]
    async fn terminal_snapshot_is_aggregated_before_telemetry_cleanup() {
        let registry = TelemetryRegistryHandle::new();
        let config = TelemetryConfig::default();
        let metrics_system = InternalTelemetrySystem::new(
            &config,
            config.reporting_interval,
            registry.clone(),
            None,
            SendPolicy::default(),
            LogContext::new,
            None,
        )
        .expect("ITS telemetry system should initialize");
        let reporter = metrics_system.reporter();
        let collector_task = tokio::spawn(metrics_system.collector().run_collection_loop());

        let entity_key = registry.register_entity(EngineEntityAttributeSet);
        let telemetry_handle = NodeTelemetryHandle::new(registry.clone(), entity_key);
        let telemetry_guard = NodeTelemetryGuard::new(telemetry_handle.clone());
        let mut metric_set = telemetry_handle
            .entity_handle()
            .register_measurement_metric_set_for_entity::<ChannelSenderMetrics>(entity_key);
        metric_set
            .with(SignalOutcomeAttributes {
                signal: SignalType::Logs,
                outcome: Outcome::Success,
            })
            .messages
            .inc();

        report_terminal_metrics(
            &reporter,
            TerminalState::new(std::time::Instant::now(), metric_set.terminal_snapshots()),
            &TerminalMetricsDeadline::default(),
        )
        .await;

        let mut message_count = None;
        registry.visit_current_metrics(|_, _, metrics| {
            for (field, value) in metrics {
                if field.name == "messages" {
                    message_count = Some(value.to_u64_lossy());
                }
            }
        });
        assert_eq!(message_count, Some(1));

        drop(telemetry_guard);
        assert_eq!(registry.metric_set_count(), 1);
        assert_eq!(registry.entity_count(), 1);

        let export_batch = registry.drain_metric_export_batch();
        let exported = export_batch
            .metric_sets
            .iter()
            .find(|metric_set| metric_set.descriptor.name == "channel.sender")
            .expect("terminal metric set should remain exportable after cleanup");
        let message_count_index = exported
            .descriptor
            .metrics
            .iter()
            .position(|field| field.name == "messages")
            .expect("messages descriptor should exist");
        assert_eq!(exported.values[message_count_index].to_u64_lossy(), 1);

        assert_eq!(registry.metric_set_count(), 0);
        assert_eq!(registry.entity_count(), 0);

        collector_task.abort();
        assert!(
            collector_task
                .await
                .expect_err("collector task should be cancelled")
                .is_cancelled()
        );
    }

    /// Scenario: a runtime receiver fails before readiness while its terminal metrics flush waits
    /// for collector acknowledgement.
    /// Guarantees: the runtime consumes the dropped-signaller result before collector release,
    /// never invokes its startup callback, and returns the original receiver error and diagnostic.
    #[test]
    fn receiver_bind_error_wins_over_signaller_drop_during_metrics_flush() {
        let pipeline_group_id = PipelineGroupId::from("test-group");
        let pipeline_id = PipelineId::from("test-pipeline");
        let pipeline_config = PipelineConfig::from_yaml(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            "
nodes:
  receiver:
    type: urn:test:receiver:startup-failure
  exporter:
    type: urn:test:exporter:pending
connections:
  - from: receiver
    to: exporter
",
        )
        .expect("startup failure test pipeline config should parse");
        let telemetry = InternalTelemetrySystem::default();
        let registry = telemetry.registry();
        let metrics_reporter = telemetry.reporter();
        let collector = telemetry.collector();
        let (collector_running_tx, collector_running_rx) = mpsc::channel();
        let (release_collector_tx, release_collector_rx) = mpsc::channel();
        let collector_cancel = CancellationToken::new();
        let collector_cancel_for_thread = collector_cancel.clone();
        let collector_thread = thread::spawn(move || {
            let collection_loop = collector.run(collector_cancel_for_thread);
            collector_running_tx
                .send(())
                .expect("collector running notification should be observed");
            release_collector_rx
                .recv()
                .expect("collector release signal should be sent");
            Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("collector test runtime should build")
                .block_on(collection_loop)
        });
        collector_running_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("collector should enter the running state");
        let pipeline_ctx = ControllerContext::new(registry.clone()).pipeline_context_with(
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            0,
            1,
            0,
        );
        let pipeline_entity_key = pipeline_ctx.register_pipeline_entity();
        let observed_state_store =
            ObservedStateStore::new(&ObservedStateSettings::default(), registry);
        let event_reporter = observed_state_store.reporter(SendPolicy::default());
        let (readiness_drop_tx, readiness_drop_rx) = mpsc::channel();
        let (startup_callback_tx, startup_callback_rx) = mpsc::channel();
        let (pipeline_result_tx, pipeline_result_rx) = mpsc::channel();

        let pipeline_thread = thread::spawn(move || {
            let channel_capacity_policy = ChannelCapacityPolicy::default();
            let (runtime_ctrl_tx, runtime_ctrl_rx) =
                runtime_ctrl_msg_channel(channel_capacity_policy.control.pipeline);
            let (pipeline_completion_tx, _pipeline_completion_rx) =
                pipeline_completion_msg_channel(channel_capacity_policy.control.completion);
            let pipeline_factory = PipelineFactory::new(
                STARTUP_FAILURE_RECEIVER_FACTORIES,
                &[],
                PENDING_EXPORTER_FACTORIES,
                &[],
            );
            let pipeline = pipeline_factory
                .build(
                    pipeline_ctx.clone(),
                    pipeline_config,
                    channel_capacity_policy,
                    TelemetryPolicy::default(),
                    std::collections::BTreeMap::new(),
                    None,
                    None,
                )
                .expect("startup failure test pipeline should build")
                .with_startup_callback(move || {
                    startup_callback_tx
                        .send(())
                        .expect("startup callback receiver should remain available");
                })
                .with_readiness_drop_callback(move || {
                    readiness_drop_tx
                        .send(())
                        .expect("readiness drop notification should be observed");
                });
            let pipeline_key = DeployedPipelineKey {
                pipeline_group_id,
                pipeline_id,
                core_id: 0,
                deployment_generation: 0,
            };
            let _pipeline_entity_guard = crate::entity_context::set_pipeline_entity_key(
                pipeline_ctx.metrics_registry(),
                pipeline_entity_key,
            );
            let (_memory_pressure_tx, memory_pressure_rx) =
                watch::channel(MemoryPressureChanged::initial());

            pipeline_result_tx
                .send(pipeline.run_forever(
                    pipeline_key,
                    pipeline_ctx,
                    event_reporter,
                    metrics_reporter,
                    Duration::from_secs(1),
                    memory_pressure_rx,
                    runtime_ctrl_tx,
                    runtime_ctrl_rx,
                    pipeline_completion_tx,
                    _pipeline_completion_rx,
                ))
                .expect("pipeline result receiver should remain available");
        });

        readiness_drop_rx
            .recv_timeout(Duration::from_secs(1))
            .expect("runtime should observe the dropped readiness signaller");
        assert!(
            !matches!(startup_callback_rx.try_recv(), Ok(())),
            "startup callback must not run before the receiver reports readiness"
        );
        assert!(
            matches!(
                pipeline_result_rx.try_recv(),
                Err(mpsc::TryRecvError::Empty)
            ),
            "the collector must hold the receiver's terminal metrics flush acknowledgment"
        );

        release_collector_tx
            .send(())
            .expect("collector should remain gated until terminal flush is observed");
        let result = pipeline_result_rx
            .recv_timeout(Duration::from_secs(2))
            .expect("startup failure pipeline should terminate within the timeout");

        pipeline_thread
            .join()
            .expect("startup failure pipeline thread should not panic");
        assert!(
            startup_callback_rx.try_recv().is_err(),
            "startup callback must never run after a receiver startup failure"
        );
        assert!(matches!(
            result,
            Err(Error::ReceiverError {
                kind: crate::error::ReceiverErrorKind::Connect,
                ref error,
                ref source_detail,
                ..
            }) if error == STARTUP_FAILURE_MESSAGE
                && source_detail == "test receiver failed before signaling readiness"
        ));
        collector_cancel.cancel();
        collector_thread
            .join()
            .expect("collector thread should not panic")
            .expect("collector loop should stop after cancellation");
    }

    /// Scenario: a receiver drops its readiness signaller but remains running without producing a
    /// startup error.
    /// Guarantees: the readiness probe still reports SignallerDropped immediately instead of
    /// being kept open by engine-owned state.
    #[tokio::test(flavor = "current_thread")]
    async fn receiver_signaller_drop_is_not_masked_while_task_remains_running() {
        let (signaller, probe) = crate::readiness::ReadinessSignaller::pair(Duration::from_secs(1));
        let receiver_task = async move {
            drop(signaller);
            std::future::pending::<()>().await;
        };
        tokio::pin!(receiver_task);

        let result = timeout(Duration::from_millis(20), async {
            tokio::select! {
                biased;
                _ = &mut receiver_task => panic!("receiver task must remain running"),
                result = probe.wait_ready() => result,
            }
        })
        .await
        .expect("signaller drop should be observed without waiting for the receiver task");

        assert_eq!(result, Err(ReadinessProbeError::SignallerDropped));
    }
}
