// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Processor wrapper used to provide a unified interface to the pipeline engine that abstracts over
//! the fact that processor implementations may be `!Send` or `Send`.
//!
//! For more details on the `!Send` implementation of a processor, see [`local::Processor`].
//! See [`shared::Processor`] for the Send implementation.

use crate::Interests;
use crate::ReceivedAtNode;
use crate::channel_metrics::ChannelMetricsRegistry;
use crate::channel_mode::{LocalMode, SharedMode, wrap_node_control_channel_metrics};
use crate::completion_emission_metrics::CompletionEmissionMetricsHandle;
use crate::config::ProcessorConfig;
use crate::context::PipelineContext;
use crate::control::{
    Controllable, NodeControlMsg, PipelineCompletionMsgSender, RuntimeCtrlMsgSender,
};
use crate::effect_handler::SourceTagging;
use crate::entity_context::NodeTelemetryGuard;
use crate::error::{Error, ProcessorErrorKind};
use crate::flow_metrics::{
    FlowConsumedItemsMetrics, FlowDroppedItemsMetrics, FlowDurationMetrics,
    FlowProducedItemsMetrics,
};
use crate::local::message::{LocalReceiver, LocalSender};
use crate::local::processor as local;
use crate::message::{Message, ProcessorInbox, Receiver, Sender};
use crate::node::{Node, NodeId, NodeWithPDataReceiver, NodeWithPDataSender};
use crate::node_local_scheduler::NodeLocalSchedulerHandle;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::shared::processor as shared;
use crate::terminal_state::TerminalMetricsDeadline;
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use otap_df_config::PortName;
use otap_df_config::node::NodeUserConfig;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::HashMap;
use std::sync::Arc;

/// FlowMetric-relevant slice of a processor `EffectHandler`'s surface.
///
/// Implemented by both `local::processor::EffectHandler<PData>` and
/// `shared::processor::EffectHandler<PData>` so that PData-side flow_metric
/// hooks (see [`FlowMetricHook`]) can be written once, generic over
/// handler kind.
pub trait FlowMetricEffectHandler {
    /// Whether this node is the start of a flow_metric range.
    fn is_flow_start(&self) -> bool;
    /// Whether this node is the end of a flow_metric range.
    fn is_flow_end(&self) -> bool;
    /// Read elapsed nanoseconds since the last send-marker advance and
    /// advance the marker to "now". Returns 0 when no marker is armed
    /// (e.g. flow_metrics inactive on this pipeline).
    fn take_elapsed_since_send_marker_ns(&self) -> u64;
    /// Record a complete flow_metric transit total (nanoseconds) into the
    /// stop node's local accumulator.
    fn record_flow_duration(&self, total: u64);
    /// Record consumed items into the start node's local accumulator.
    fn record_flow_consumed_items(&self, items: u64);
    /// Record produced items into the stop node's local accumulator.
    fn record_flow_produced_items(&self, items: u64);
}

/// Per-`PData` hooks straddling a processor's `process()` call: an
/// `after_processor_receive` notification fires immediately after a
/// `Message::PData` is dequeued (before `process()` runs), and a
/// `before_processor_send` notification fires immediately before the
/// effect handler forwards a message to the output router.
///
/// The send-side hook covers **both** the plain `send_message[_to]`
/// family and the `send_message_with_source_node[_to]` family — every
/// send method on every processor handler invokes it exactly once.
/// Both methods default to no-ops; PData types with bookkeeping needs
/// (e.g. flow_metric accumulation on `OtapPdata`) override one or both.
///
/// `EffectHandler<PData>` is generic but lives in the engine crate, while
/// some `PData` types need bookkeeping defined in their own crate.
/// Inherent methods shadow extension-trait methods, so we route
/// per-`PData` behavior through this trait. PData types with nothing to
/// do can simply write `impl FlowMetricHook for MyPData {}`.
///
/// NOTE: This trait currently lives in `processor.rs` and only fires from
/// processor run loops / processor effect handlers because processors are
/// the only nodes that need pre-process and pre-send hooks today (for
/// flow_metric flow metric). If receivers or exporters ever need
/// analogous `before_*` / `after_*` hooks on PData, this trait should be
/// hoisted to a more generic location (e.g. a top-level `flow_hook` module
/// or `crate::lib`) and its `H: FlowMetricEffectHandler` bound generalized
/// so it can be invoked from receiver/exporter handlers as well.
pub trait FlowMetricHook: Sized {
    /// Invoked once per message immediately before the processor handler
    /// forwards it to the output router.
    fn before_processor_send<H: FlowMetricEffectHandler>(&mut self, _handler: &H) {}

    /// Invoked once per `Message::PData` immediately after it is dequeued
    /// by a processor's run loop and before `process()` runs. Lets PData
    /// types observe the *pre-process* state of the data — e.g. counting
    /// items entering a flow_metric start node before any filter or drop
    /// inside `process()`. Default impl is a no-op.
    fn after_processor_receive<H: FlowMetricEffectHandler>(&mut self, _handler: &H) {}
}

/// Processor-local wakeup requirements declared by a processor implementation.
///
/// `live_slots` is the maximum number of distinct wakeup slots that can be
/// live at the same time for one processor instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct LocalWakeupRequirements {
    /// Maximum number of concurrently live wakeup slots.
    pub live_slots: usize,
}

impl LocalWakeupRequirements {
    /// Create local wakeup requirements for a processor.
    #[must_use]
    pub const fn new(live_slots: usize) -> Self {
        Self { live_slots }
    }
}

/// Optional runtime services requested by a processor implementation.
///
/// This is the single source of truth for processor runtime wiring. For
/// example, `local_wakeups: Some(...)` both enables processor-local wakeups and
/// declares the live slot count that the runtime must provision.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct ProcessorRuntimeRequirements {
    /// Processor-local wakeup requirements, if the processor uses the local
    /// wakeup API.
    pub local_wakeups: Option<LocalWakeupRequirements>,
    /// Whether this processor drops signal items and therefore records the
    /// `dropped.items` flow metric when it lies within a flow that enables
    /// it. Defaults to `false`.
    pub makes_drop_decisions: bool,
}

impl ProcessorRuntimeRequirements {
    /// Runtime requirements for a processor that does not need any optional
    /// engine services.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            local_wakeups: None,
            makes_drop_decisions: false,
        }
    }

    /// Runtime requirements for a processor that uses local wakeups.
    #[must_use]
    pub const fn with_local_wakeups(live_slots: usize) -> Self {
        Self {
            local_wakeups: Some(LocalWakeupRequirements::new(live_slots)),
            makes_drop_decisions: false,
        }
    }

    /// Declare that this processor drops signal items, enabling
    /// `dropped.items` flow-metric recording.
    #[must_use]
    pub const fn with_drop_decisions(mut self) -> Self {
        self.makes_drop_decisions = true;
        self
    }
}

/// A wrapper for the processor that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the processor regardless of the effect
/// handler type. This is the only type that the pipeline engine will use in order to be agnostic to
/// the effect handler type.
pub enum ProcessorWrapper<PData> {
    /// A processor with a `!Send` implementation.
    Local {
        /// Index node identifier.
        node_id: NodeId,
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the processor.
        runtime_config: ProcessorConfig,
        /// The processor instance.
        processor: Box<dyn local::Processor<PData>>,
        /// A sender for control messages.
        control_sender: LocalSender<NodeControlMsg<PData>>,
        /// A receiver for control messages.
        control_receiver: LocalReceiver<NodeControlMsg<PData>>,
        /// Senders for PData messages per output port.
        /// Uses the generic `Sender` so local processors can still target shared channels when
        /// mixed local/shared wiring requires it.
        pdata_senders: HashMap<PortName, Sender<PData>>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<Receiver<PData>>,
        /// Telemetry guard for node lifecycle cleanup.
        telemetry: Option<NodeTelemetryGuard>,
        /// Whether outgoing messages need source node tagging.
        source_tag: SourceTagging,
    },
    /// A processor with a `Send` implementation.
    Shared {
        /// Index node identifier.
        node_id: NodeId,
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the processor.
        runtime_config: ProcessorConfig,
        /// The processor instance.
        processor: Box<dyn shared::Processor<PData>>,
        /// A sender for control messages.
        control_sender: SharedSender<NodeControlMsg<PData>>,
        /// A receiver for control messages.
        control_receiver: SharedReceiver<NodeControlMsg<PData>>,
        /// Senders for PData messages per output port.
        /// Uses `SharedSender` to keep the shared processor `Send` for multi-threaded execution.
        pdata_senders: HashMap<PortName, SharedSender<PData>>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<SharedReceiver<PData>>,
        /// Telemetry guard for node lifecycle cleanup.
        telemetry: Option<NodeTelemetryGuard>,
        /// Whether outgoing messages need source node tagging.
        source_tag: SourceTagging,
    },
}

/// Runtime components for a processor wrapper, containing all the necessary
/// components to run a processor independently.
///
/// This allows external control over the message processing loop, useful for testing and custom
/// processing scenarios.
#[allow(clippy::large_enum_variant)]
pub enum ProcessorWrapperRuntime<PData> {
    /// A processor with a `!Send` implementation.
    Local {
        /// The processor instance.
        processor: Box<dyn local::Processor<PData>>,
        /// The processor inbox
        inbox: ProcessorInbox<PData>,
        /// The local effect handler
        effect_handler: local::EffectHandler<PData>,
    },
    /// A processor with a `Send` implementation.
    Shared {
        /// The processor instance.
        processor: Box<dyn shared::Processor<PData>>,
        /// Processor inbox
        inbox: ProcessorInbox<PData>,
        /// The shared effect handler
        effect_handler: shared::EffectHandler<PData>,
    },
}

impl<PData> ProcessorWrapper<PData> {
    /// Creates a new local `ProcessorWrapper` with the given processor and appropriate effect handler.
    pub fn local<P>(
        processor: P,
        node_id: NodeId,
        user_config: Arc<NodeUserConfig>,
        config: &ProcessorConfig,
    ) -> Self
    where
        P: local::Processor<PData> + 'static,
    {
        let runtime_config = config.clone();
        let (control_sender, control_receiver) =
            mpsc::Channel::new(config.control_channel.capacity);

        ProcessorWrapper::Local {
            node_id,
            user_config,
            runtime_config,
            processor: Box::new(processor),
            control_sender: LocalSender::mpsc(control_sender),
            control_receiver: LocalReceiver::mpsc(control_receiver),
            pdata_senders: HashMap::new(),
            pdata_receiver: None,
            telemetry: None,
            source_tag: SourceTagging::Disabled,
        }
    }

    /// Creates a new shared `ProcessorWrapper` with the given processor and appropriate effect handler.
    pub fn shared<P>(
        processor: P,
        node_id: NodeId,
        user_config: Arc<NodeUserConfig>,
        config: &ProcessorConfig,
    ) -> Self
    where
        P: shared::Processor<PData> + 'static,
    {
        let runtime_config = config.clone();
        let (control_sender, control_receiver) =
            tokio::sync::mpsc::channel(config.control_channel.capacity);

        ProcessorWrapper::Shared {
            node_id,
            user_config,
            runtime_config,
            processor: Box::new(processor),
            control_sender: SharedSender::mpsc(control_sender),
            control_receiver: SharedReceiver::mpsc(control_receiver),
            pdata_senders: HashMap::new(),
            pdata_receiver: None,
            telemetry: None,
            source_tag: SourceTagging::Disabled,
        }
    }

    pub(crate) fn with_node_telemetry_guard(self, guard: NodeTelemetryGuard) -> Self {
        match self {
            ProcessorWrapper::Local {
                node_id,
                user_config,
                runtime_config,
                processor,
                control_sender,
                control_receiver,
                pdata_senders,
                pdata_receiver,
                source_tag,
                ..
            } => ProcessorWrapper::Local {
                node_id,
                user_config,
                runtime_config,
                processor,
                control_sender,
                control_receiver,
                pdata_senders,
                pdata_receiver,
                telemetry: Some(guard),
                source_tag,
            },
            ProcessorWrapper::Shared {
                node_id,
                user_config,
                runtime_config,
                processor,
                control_sender,
                control_receiver,
                pdata_senders,
                pdata_receiver,
                source_tag,
                ..
            } => ProcessorWrapper::Shared {
                node_id,
                user_config,
                runtime_config,
                processor,
                control_sender,
                control_receiver,
                pdata_senders,
                pdata_receiver,
                telemetry: Some(guard),
                source_tag,
            },
        }
    }

    pub(crate) const fn take_telemetry_guard(&mut self) -> Option<NodeTelemetryGuard> {
        match self {
            ProcessorWrapper::Local { telemetry, .. } => telemetry.take(),
            ProcessorWrapper::Shared { telemetry, .. } => telemetry.take(),
        }
    }

    pub(crate) fn runtime_requirements(&self) -> ProcessorRuntimeRequirements {
        match self {
            ProcessorWrapper::Local { processor, .. } => processor.runtime_requirements(),
            ProcessorWrapper::Shared { processor, .. } => processor.runtime_requirements(),
        }
    }

    pub(crate) fn with_control_channel_metrics(
        self,
        pipeline_ctx: &PipelineContext,
        channel_metrics: &mut ChannelMetricsRegistry,
        channel_metrics_enabled: bool,
    ) -> Self {
        match self {
            ProcessorWrapper::Local {
                node_id,
                runtime_config,
                control_sender,
                control_receiver,
                user_config,
                processor,
                pdata_senders,
                pdata_receiver,
                telemetry,
                source_tag,
            } => {
                let (control_sender, control_receiver) =
                    wrap_node_control_channel_metrics::<LocalMode, NodeControlMsg<PData>>(
                        node_id.name.as_ref(),
                        pipeline_ctx,
                        channel_metrics,
                        channel_metrics_enabled,
                        runtime_config.control_channel.capacity as u64,
                        control_sender,
                        control_receiver,
                    );

                ProcessorWrapper::Local {
                    node_id,
                    user_config,
                    runtime_config,
                    processor,
                    control_sender,
                    control_receiver,
                    pdata_senders,
                    pdata_receiver,
                    telemetry,
                    source_tag,
                }
            }
            ProcessorWrapper::Shared {
                node_id,
                runtime_config,
                control_sender,
                control_receiver,
                user_config,
                processor,
                pdata_senders,
                pdata_receiver,
                telemetry,
                source_tag,
            } => {
                let (control_sender, control_receiver) =
                    wrap_node_control_channel_metrics::<SharedMode, NodeControlMsg<PData>>(
                        node_id.name.as_ref(),
                        pipeline_ctx,
                        channel_metrics,
                        channel_metrics_enabled,
                        runtime_config.control_channel.capacity as u64,
                        control_sender,
                        control_receiver,
                    );

                ProcessorWrapper::Shared {
                    node_id,
                    user_config,
                    runtime_config,
                    processor,
                    control_sender,
                    control_receiver,
                    pdata_senders,
                    pdata_receiver,
                    telemetry,
                    source_tag,
                }
            }
        }
    }

    /// Prepare the processor runtime components without starting the processing loop.
    /// This allows external control over the message processing loop.
    pub async fn prepare_runtime(
        self,
        metrics_reporter: MetricsReporter,
        node_interests: Interests,
    ) -> Result<ProcessorWrapperRuntime<PData>, Error> {
        match self {
            ProcessorWrapper::Local {
                node_id,
                runtime_config,
                processor,
                control_receiver,
                pdata_senders,
                pdata_receiver,
                user_config,
                source_tag,
                ..
            } => {
                let runtime_requirements = processor.runtime_requirements();
                let pdata_receiver = pdata_receiver.ok_or_else(|| Error::ProcessorError {
                    processor: node_id.clone(),
                    kind: ProcessorErrorKind::Configuration,
                    error: "The pdata receiver must be defined at this stage".to_owned(),
                    source_detail: String::new(),
                })?;
                validate_local_wakeup_requirements(&node_id, runtime_requirements)?;
                let local_scheduler = NodeLocalSchedulerHandle::new(
                    runtime_config.input_pdata_channel.capacity,
                    runtime_requirements
                        .local_wakeups
                        .map(|requirements| requirements.live_slots)
                        .unwrap_or(0),
                );
                let inbox = ProcessorInbox::new_with_local_scheduler(
                    Receiver::Local(control_receiver),
                    pdata_receiver,
                    local_scheduler.clone(),
                    node_id.index,
                    node_interests,
                );
                let default_port = user_config.default_output.clone();
                let mut effect_handler = local::EffectHandler::new(
                    node_id,
                    pdata_senders,
                    default_port,
                    metrics_reporter,
                );
                effect_handler.set_source_tagging(source_tag);
                effect_handler.core.set_local_scheduler(local_scheduler);
                Ok(ProcessorWrapperRuntime::Local {
                    processor,
                    effect_handler,
                    inbox,
                })
            }
            ProcessorWrapper::Shared {
                node_id,
                runtime_config,
                processor,
                control_receiver,
                pdata_senders,
                pdata_receiver,
                user_config,
                source_tag,
                ..
            } => {
                let runtime_requirements = processor.runtime_requirements();
                let pdata_receiver =
                    Receiver::Shared(pdata_receiver.ok_or_else(|| Error::ProcessorError {
                        processor: node_id.clone(),
                        kind: ProcessorErrorKind::Configuration,
                        error: "The pdata receiver must be defined at this stage".to_owned(),
                        source_detail: String::new(),
                    })?);
                validate_local_wakeup_requirements(&node_id, runtime_requirements)?;
                let local_scheduler = NodeLocalSchedulerHandle::new(
                    runtime_config.input_pdata_channel.capacity,
                    runtime_requirements
                        .local_wakeups
                        .map(|requirements| requirements.live_slots)
                        .unwrap_or(0),
                );
                let inbox = ProcessorInbox::new_with_local_scheduler(
                    Receiver::Shared(control_receiver),
                    pdata_receiver,
                    local_scheduler.clone(),
                    node_id.index,
                    node_interests,
                );
                let default_port = user_config.default_output.clone();
                let mut effect_handler = shared::EffectHandler::new(
                    node_id,
                    pdata_senders,
                    default_port,
                    metrics_reporter,
                );
                effect_handler.set_source_tagging(source_tag);
                effect_handler.core.set_local_scheduler(local_scheduler);
                Ok(ProcessorWrapperRuntime::Shared {
                    processor,
                    effect_handler,
                    inbox,
                })
            }
        }
    }

    /// Start the processor and run the message processing loop.
    pub async fn start(
        self,
        runtime_ctrl_msg_tx: RuntimeCtrlMsgSender<PData>,
        pipeline_completion_msg_tx: PipelineCompletionMsgSender<PData>,
        metrics_reporter: MetricsReporter,
        node_interests: Interests,
    ) -> Result<(), Error>
    where
        PData: ReceivedAtNode + FlowMetricHook,
    {
        self.start_with_completion_metrics(
            runtime_ctrl_msg_tx,
            pipeline_completion_msg_tx,
            metrics_reporter,
            node_interests,
            None,
            false,
            false,
            None,
            None,
            None,
            None,
            false,
            false,
            TerminalMetricsDeadline::default(),
        )
        .await
    }

    pub(crate) async fn start_with_completion_metrics(
        self,
        runtime_ctrl_msg_tx: RuntimeCtrlMsgSender<PData>,
        pipeline_completion_msg_tx: PipelineCompletionMsgSender<PData>,
        metrics_reporter: MetricsReporter,
        node_interests: Interests,
        completion_emission_metrics: Option<CompletionEmissionMetricsHandle>,
        flow_is_start: bool,
        flow_is_end: bool,
        flow_consumed_items_metric: Option<MetricSet<FlowConsumedItemsMetrics>>,
        flow_duration_metric: Option<MetricSet<FlowDurationMetrics>>,
        flow_produced_items_metric: Option<MetricSet<FlowProducedItemsMetrics>>,
        flow_dropped_items_metric: Option<MetricSet<FlowDroppedItemsMetrics>>,
        flow_metrics_active: bool,
        flow_needs_timing: bool,
        terminal_metrics_deadline: TerminalMetricsDeadline,
    ) -> Result<(), Error>
    where
        PData: ReceivedAtNode + FlowMetricHook,
    {
        let runtime = self
            .prepare_runtime(metrics_reporter.clone(), node_interests)
            .await?;

        match runtime {
            ProcessorWrapperRuntime::Local {
                mut processor,
                mut inbox,
                mut effect_handler,
            } => {
                effect_handler
                    .core
                    .set_runtime_ctrl_msg_sender(runtime_ctrl_msg_tx);
                effect_handler
                    .core
                    .set_pipeline_completion_msg_sender(pipeline_completion_msg_tx);
                effect_handler.core.set_node_interests(node_interests);
                effect_handler
                    .core
                    .set_completion_emission_metrics(completion_emission_metrics.clone());
                effect_handler.set_flow_roles(
                    flow_is_start,
                    flow_is_end,
                    flow_consumed_items_metric.clone(),
                    flow_duration_metric.clone(),
                    flow_produced_items_metric.clone(),
                    flow_dropped_items_metric.clone(),
                    flow_metrics_active,
                    flow_needs_timing,
                );

                while let Ok(mut msg) = inbox.recv_when(processor.accept_pdata()).await {
                    if effect_handler.flow_metrics_active() {
                        match &mut msg {
                            Message::Control(NodeControlMsg::CollectTelemetry { .. })
                                if effect_handler.is_flow_start()
                                    || effect_handler.is_flow_end()
                                    || effect_handler.is_flow_decision() =>
                            {
                                effect_handler.report_flow_metrics();
                            }
                            Message::PData(data) => {
                                data.after_processor_receive(&effect_handler);
                                effect_handler.begin_process_timing();
                            }
                            _ => {}
                        }
                    }
                    processor.process(msg, &mut effect_handler).await?;
                }
                // Collect final metrics before exiting
                let terminal_metrics_deadline = terminal_metrics_deadline.get();
                if effect_handler.is_flow_start()
                    || effect_handler.is_flow_end()
                    || effect_handler.is_flow_decision()
                {
                    if let Err(error) = effect_handler
                        .report_flow_metrics_reliably(terminal_metrics_deadline)
                        .await
                    {
                        otap_df_telemetry::otel_warn!(
                            "processor.flow_metrics.final_reporting.fail",
                            error = error.to_string()
                        );
                    }
                }
                let (terminal_metrics_tx, terminal_metrics_rx) = flume::unbounded();
                let terminal_metrics_reporter = MetricsReporter::new(terminal_metrics_tx);
                let process_result = processor
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: terminal_metrics_reporter,
                        }),
                        &mut effect_handler,
                    )
                    .await;
                while let Ok(snapshot) = terminal_metrics_rx.try_recv() {
                    if let Err(error) = metrics_reporter
                        .report_snapshot_reliably_until(snapshot, terminal_metrics_deadline)
                        .await
                    {
                        otap_df_telemetry::otel_warn!(
                            "processor.metrics.final_reporting.fail",
                            error = error.to_string()
                        );
                    }
                }
                process_result?
            }
            ProcessorWrapperRuntime::Shared {
                mut processor,
                mut inbox,
                mut effect_handler,
            } => {
                effect_handler
                    .core
                    .set_runtime_ctrl_msg_sender(runtime_ctrl_msg_tx);
                effect_handler
                    .core
                    .set_pipeline_completion_msg_sender(pipeline_completion_msg_tx);
                effect_handler.core.set_node_interests(node_interests);
                effect_handler
                    .core
                    .set_completion_emission_metrics(completion_emission_metrics);
                effect_handler.set_flow_roles(
                    flow_is_start,
                    flow_is_end,
                    flow_consumed_items_metric.clone(),
                    flow_duration_metric.clone(),
                    flow_produced_items_metric.clone(),
                    flow_dropped_items_metric.clone(),
                    flow_metrics_active,
                    flow_needs_timing,
                );

                while let Ok(mut msg) = inbox.recv_when(processor.accept_pdata()).await {
                    if effect_handler.flow_metrics_active() {
                        match &mut msg {
                            Message::Control(NodeControlMsg::CollectTelemetry { .. })
                                if effect_handler.is_flow_start()
                                    || effect_handler.is_flow_end()
                                    || effect_handler.is_flow_decision() =>
                            {
                                effect_handler.report_flow_metrics();
                            }
                            Message::PData(data) => {
                                data.after_processor_receive(&effect_handler);
                                effect_handler.begin_process_timing();
                            }
                            _ => {}
                        }
                    }
                    processor.process(msg, &mut effect_handler).await?;
                }
                // Collect final metrics before exiting
                let terminal_metrics_deadline = terminal_metrics_deadline.get();
                if effect_handler.is_flow_start()
                    || effect_handler.is_flow_end()
                    || effect_handler.is_flow_decision()
                {
                    if let Err(error) = effect_handler
                        .report_flow_metrics_reliably(terminal_metrics_deadline)
                        .await
                    {
                        otap_df_telemetry::otel_warn!(
                            "processor.flow_metrics.final_reporting.fail",
                            error = error.to_string()
                        );
                    }
                }
                let (terminal_metrics_tx, terminal_metrics_rx) = flume::unbounded();
                let terminal_metrics_reporter = MetricsReporter::new(terminal_metrics_tx);
                let process_result = processor
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry {
                            metrics_reporter: terminal_metrics_reporter,
                        }),
                        &mut effect_handler,
                    )
                    .await;
                while let Ok(snapshot) = terminal_metrics_rx.try_recv() {
                    if let Err(error) = metrics_reporter
                        .report_snapshot_reliably_until(snapshot, terminal_metrics_deadline)
                        .await
                    {
                        otap_df_telemetry::otel_warn!(
                            "processor.metrics.final_reporting.fail",
                            error = error.to_string()
                        );
                    }
                }
                process_result?
            }
        }
        Ok(())
    }

    /// Takes the PData receiver from the wrapper and returns it.
    pub const fn take_pdata_receiver(&mut self) -> Receiver<PData> {
        match self {
            ProcessorWrapper::Local { pdata_receiver, .. } => {
                pdata_receiver.take().expect("pdata_receiver is None")
            }
            ProcessorWrapper::Shared { pdata_receiver, .. } => {
                Receiver::Shared(pdata_receiver.take().expect("pdata_receiver is None"))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<PData> Node<PData> for ProcessorWrapper<PData> {
    fn is_shared(&self) -> bool {
        match self {
            ProcessorWrapper::Local { .. } => false,
            ProcessorWrapper::Shared { .. } => true,
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            ProcessorWrapper::Local { node_id, .. } => node_id.clone(),
            ProcessorWrapper::Shared { node_id, .. } => node_id.clone(),
        }
    }

    fn user_config(&self) -> Arc<NodeUserConfig> {
        match self {
            ProcessorWrapper::Local {
                user_config: config,
                ..
            } => config.clone(),
            ProcessorWrapper::Shared {
                user_config: config,
                ..
            } => config.clone(),
        }
    }

    /// Sends a control message to the node.
    async fn send_control_msg(
        &self,
        msg: NodeControlMsg<PData>,
    ) -> Result<(), SendError<NodeControlMsg<PData>>> {
        match self {
            ProcessorWrapper::Local { control_sender, .. } => control_sender.send(msg).await,
            ProcessorWrapper::Shared { control_sender, .. } => control_sender.send(msg).await,
        }
    }
}

pub(crate) fn validate_local_wakeup_requirements(
    node_id: &NodeId,
    requirements: ProcessorRuntimeRequirements,
) -> Result<(), Error> {
    let Some(local_wakeups) = requirements.local_wakeups else {
        return Ok(());
    };

    if local_wakeups.live_slots == 0 {
        return Err(Error::ProcessorError {
            processor: node_id.clone(),
            kind: ProcessorErrorKind::Configuration,
            error: "processor-local wakeup requirement must declare at least one live slot"
                .to_owned(),
            source_detail: String::new(),
        });
    }

    Ok(())
}

#[async_trait::async_trait(?Send)]
impl<PData> Controllable<PData> for ProcessorWrapper<PData> {
    /// Returns the control message sender for the processor.
    fn control_sender(&self) -> Sender<NodeControlMsg<PData>> {
        match self {
            ProcessorWrapper::Local { control_sender, .. } => Sender::Local(control_sender.clone()),
            ProcessorWrapper::Shared { control_sender, .. } => {
                Sender::Shared(control_sender.clone())
            }
        }
    }
}

impl<PData> NodeWithPDataSender<PData> for ProcessorWrapper<PData> {
    fn set_pdata_sender(
        &mut self,
        node_id: NodeId,
        port: PortName,
        sender: Sender<PData>,
    ) -> Result<(), Error> {
        match (self, sender) {
            (ProcessorWrapper::Local { pdata_senders, .. }, sender) => {
                let _ = pdata_senders.insert(port, sender);
                Ok(())
            }
            (ProcessorWrapper::Shared { pdata_senders, .. }, Sender::Shared(sender)) => {
                let _ = pdata_senders.insert(port, sender);
                Ok(())
            }
            (ProcessorWrapper::Shared { .. }, _) => Err(Error::ProcessorError {
                processor: node_id,
                kind: ProcessorErrorKind::Configuration,
                error: "Expected a shared sender for PData".to_owned(),
                source_detail: String::new(),
            }),
        }
    }

    fn set_source_tagging(&mut self, value: SourceTagging) {
        match self {
            ProcessorWrapper::Local { source_tag, .. } => *source_tag = value,
            ProcessorWrapper::Shared { source_tag, .. } => *source_tag = value,
        }
    }
}

impl<PData> NodeWithPDataReceiver<PData> for ProcessorWrapper<PData> {
    fn set_pdata_receiver(
        &mut self,
        node_id: NodeId,
        receiver: Receiver<PData>,
    ) -> Result<(), Error> {
        match (self, receiver) {
            (ProcessorWrapper::Local { pdata_receiver, .. }, receiver) => {
                *pdata_receiver = Some(receiver);
                Ok(())
            }
            (ProcessorWrapper::Shared { pdata_receiver, .. }, Receiver::Shared(receiver)) => {
                *pdata_receiver = Some(receiver);
                Ok(())
            }
            (ProcessorWrapper::Shared { .. }, _) => Err(Error::ProcessorError {
                processor: node_id,
                kind: ProcessorErrorKind::Configuration,
                error: "Expected a shared receiver for PData".to_owned(),
                source_detail: String::new(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::control::{
        Controllable, NodeControlMsg,
        NodeControlMsg::{Config, Shutdown, TimerTick},
        pipeline_completion_msg_channel, runtime_ctrl_msg_channel,
    };
    use crate::flow_metrics::{
        FlowAttributeSet, FlowConsumedItemsMetrics, FlowDroppedItemsMetrics, FlowDurationMetrics,
        FlowProducedItemsMetrics,
    };
    use crate::local::message::{LocalReceiver, LocalSender};
    use crate::local::processor as local;
    use crate::message::{Message, Receiver, Sender};
    use crate::node::{NodeWithPDataReceiver, NodeWithPDataSender};
    use crate::processor::{
        Error, ProcessorRuntimeRequirements, ProcessorWrapper, validate_local_wakeup_requirements,
    };
    use crate::shared::processor as shared;
    use crate::testing::processor::TestRuntime;
    use crate::testing::processor::{TestContext, ValidateContext};
    use crate::testing::{CtrlMsgCounters, TestMsg, test_node};
    use async_trait::async_trait;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_telemetry::metrics::MetricValue;
    use serde_json::Value;
    use std::ops::Add;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::time::{Duration, Instant};

    /// A generic test processor that counts message events.
    /// Works with any type of processor !Send or Send.
    pub struct TestProcessor {
        /// Counter for different message types
        ctrl_msg_counters: CtrlMsgCounters,
    }

    impl TestProcessor {
        /// Creates a new test node with the given counter
        pub fn new(ctrl_msg_counters: CtrlMsgCounters) -> Self {
            TestProcessor { ctrl_msg_counters }
        }
    }

    #[async_trait(?Send)]
    impl local::Processor<TestMsg> for TestProcessor {
        async fn process(
            &mut self,
            msg: Message<TestMsg>,
            effect_handler: &mut local::EffectHandler<TestMsg>,
        ) -> Result<(), Error> {
            match msg {
                Message::Control(control) => match control {
                    TimerTick {} => {
                        self.ctrl_msg_counters.increment_timer_tick();
                    }
                    Config { .. } => {
                        self.ctrl_msg_counters.increment_config();
                    }
                    Shutdown { .. } => {
                        self.ctrl_msg_counters.increment_shutdown();
                    }
                    _ => {}
                },
                Message::PData(data) => {
                    self.ctrl_msg_counters.increment_message();
                    effect_handler
                        .send_message(TestMsg(format!("{} RECEIVED", data.0)))
                        .await?;
                }
            }
            Ok(())
        }
    }

    #[async_trait]
    impl shared::Processor<TestMsg> for TestProcessor {
        async fn process(
            &mut self,
            msg: Message<TestMsg>,
            effect_handler: &mut shared::EffectHandler<TestMsg>,
        ) -> Result<(), Error> {
            match msg {
                Message::Control(control) => match control {
                    TimerTick {} => {
                        self.ctrl_msg_counters.increment_timer_tick();
                    }
                    Config { .. } => {
                        self.ctrl_msg_counters.increment_config();
                    }
                    Shutdown { .. } => {
                        self.ctrl_msg_counters.increment_shutdown();
                    }
                    _ => {}
                },
                Message::PData(data) => {
                    self.ctrl_msg_counters.increment_message();
                    effect_handler
                        .send_message(TestMsg(format!("{} RECEIVED", data.0)))
                        .await?;
                }
            }
            Ok(())
        }
    }

    /// Test closure that simulates a typical processor scenario.
    fn scenario() -> impl FnOnce(TestContext<TestMsg>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |mut ctx| {
            Box::pin(async move {
                // Process a TimerTick event.
                ctx.process(Message::timer_tick_ctrl_msg())
                    .await
                    .expect("Processor failed on TimerTick");
                assert!(ctx.drain_pdata().await.is_empty());

                // Process a Message event.
                ctx.process(Message::data_msg(TestMsg("Hello".to_owned())))
                    .await
                    .expect("Processor failed on Message");
                let msgs = ctx.drain_pdata().await;
                assert_eq!(msgs.len(), 1);
                assert_eq!(msgs[0], TestMsg("Hello RECEIVED".to_string()));

                // Process a Config event.
                ctx.process(Message::config_ctrl_msg(Value::Null))
                    .await
                    .expect("Processor failed on Config");
                assert!(ctx.drain_pdata().await.is_empty());

                // Process a Shutdown event.
                ctx.process(Message::shutdown_ctrl_msg(
                    Instant::now().add(Duration::from_millis(200)),
                    "no reason",
                ))
                .await
                .expect("Processor failed on Shutdown");
                assert!(ctx.drain_pdata().await.is_empty());
            })
        }
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure() -> impl FnOnce(ValidateContext) -> Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                ctx.counters().assert(
                    1, // timer tick
                    1, // message
                    1, // config
                    1, // shutdown
                );
            })
        }
    }

    #[test]
    fn test_processor_local() {
        let test_runtime = TestRuntime::new();
        let user_config = Arc::new(NodeUserConfig::new_processor_config("test_processor"));
        let processor = ProcessorWrapper::local(
            TestProcessor::new(test_runtime.counters()),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure());
    }

    #[test]
    fn test_processor_shared() {
        let test_runtime = TestRuntime::new();
        let user_config = Arc::new(NodeUserConfig::new_processor_config("test_processor"));
        let processor = ProcessorWrapper::shared(
            TestProcessor::new(test_runtime.counters()),
            test_node(test_runtime.config().name.clone()),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure());
    }

    /// Scenario: a processor does not request any processor-local wakeup
    /// service from the runtime.
    /// Guarantees: validation succeeds without requiring any local wakeup
    /// capacity, so processors that do not use wakeups do not pay configuration
    /// or startup costs for that service.
    #[test]
    fn validate_local_wakeup_requirements_accepts_processors_without_wakeups() {
        assert!(
            validate_local_wakeup_requirements(
                &test_node("test_processor"),
                ProcessorRuntimeRequirements::none(),
            )
            .is_ok()
        );
    }

    /// Scenario: a processor declares local wakeups but reports an invalid live
    /// slot requirement of zero.
    /// Guarantees: validation rejects the configuration before startup, so the
    /// runtime never provisions an unusable local wakeup service.
    #[test]
    fn validate_local_wakeup_requirements_rejects_zero_live_slots() {
        let err = validate_local_wakeup_requirements(
            &test_node("test_processor"),
            ProcessorRuntimeRequirements::with_local_wakeups(0),
        )
        .expect_err("zero live slots must be rejected");

        let Error::ProcessorError { error, .. } = err else {
            panic!("expected processor configuration error");
        };
        assert_eq!(
            error,
            "processor-local wakeup requirement must declare at least one live slot"
        );
    }

    /// Scenario: a processor declares a positive local wakeup live slot count.
    /// Guarantees: validation succeeds so the declared slot count can act as
    /// the single source of truth for local wakeup runtime provisioning.
    #[test]
    fn validate_local_wakeup_requirements_accepts_positive_live_slots() {
        assert!(
            validate_local_wakeup_requirements(
                &test_node("test_processor"),
                ProcessorRuntimeRequirements::with_local_wakeups(6),
            )
            .is_ok()
        );
    }

    #[derive(Clone, Debug, Default)]
    struct FlowMetricTestPData {
        flow_compute_ns: u64,
        flow_metric_active: bool,
    }

    impl crate::ReceivedAtNode for FlowMetricTestPData {
        fn received_at_node(&mut self, _node_id: usize, _node_interests: crate::Interests) {}
    }

    impl crate::processor::FlowMetricHook for FlowMetricTestPData {
        fn before_processor_send<H: crate::processor::FlowMetricEffectHandler>(
            &mut self,
            handler: &H,
        ) {
            if !handler.is_flow_start() && !handler.is_flow_end() && !self.flow_metric_active {
                return;
            }

            if handler.is_flow_start() {
                self.flow_metric_active = true;
            }

            self.flow_compute_ns = self
                .flow_compute_ns
                .saturating_add(handler.take_elapsed_since_send_marker_ns());

            if handler.is_flow_end() && self.flow_metric_active && self.flow_compute_ns > 0 {
                handler.record_flow_duration(self.flow_compute_ns);
                handler.record_flow_produced_items(1);
                self.flow_compute_ns = 0;
                self.flow_metric_active = false;
            }
        }

        fn after_processor_receive<H: crate::processor::FlowMetricEffectHandler>(
            &mut self,
            handler: &H,
        ) {
            if handler.is_flow_start() {
                handler.record_flow_consumed_items(1);
            }
        }
    }

    struct SyncOnlyFlowMetricProcessor;

    #[async_trait(?Send)]
    impl local::Processor<FlowMetricTestPData> for SyncOnlyFlowMetricProcessor {
        async fn process(
            &mut self,
            msg: Message<FlowMetricTestPData>,
            effect_handler: &mut local::EffectHandler<FlowMetricTestPData>,
        ) -> Result<(), Error> {
            let Message::PData(data) = msg else {
                return Ok(());
            };

            let mut value = 0u64;
            for i in 0..50_000 {
                value = value.wrapping_add(std::hint::black_box(i));
            }
            let _ = std::hint::black_box(value);

            tokio::task::yield_now().await;
            effect_handler.send_message(data).await?;
            Ok(())
        }
    }

    /// Scenario: a flow is configured to collect only consumed items at its start node.
    /// Guarantees: telemetry reports the consumed-items counter and no end-node metrics.
    #[test]
    fn flow_opt_in_consumed_items_reports_only_start_metric() {
        let (pipeline_ctx, _) = crate::testing::test_pipeline_ctx();
        let entity_key = pipeline_ctx
            .metrics_registry()
            .register_entity(FlowAttributeSet::default());
        let incoming_metric = pipeline_ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowConsumedItemsMetrics>(entity_key);
        let (metrics_rx, metrics_reporter) =
            otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
        let mut handler = local::EffectHandler::<TestMsg>::new(
            test_node("proc"),
            std::collections::HashMap::new(),
            None,
            metrics_reporter,
        );
        handler.set_flow_roles(
            true,
            false,
            Some(incoming_metric),
            None,
            None,
            None,
            true,
            false,
        );

        handler.record_flow_consumed_items(3);
        handler.record_flow_duration(10);
        handler.record_flow_produced_items(4);
        handler.report_flow_metrics();

        let snapshot = metrics_rx
            .try_recv()
            .expect("incoming metric should report");
        let [MetricValue::U64(consumed_items)] = snapshot.get_metrics() else {
            panic!("expected consumed item metric only");
        };
        assert_eq!(*consumed_items, 3);
        assert!(metrics_rx.try_recv().is_err());
    }

    /// Scenario: a flow is configured to collect duration and produced items at its end node.
    /// Guarantees: telemetry reports the duration and produced-items metrics without a start-node metric.
    #[test]
    fn flow_opt_in_duration_and_produced_items_reports_only_end_metrics() {
        let (pipeline_ctx, _) = crate::testing::test_pipeline_ctx();
        let entity_key = pipeline_ctx
            .metrics_registry()
            .register_entity(FlowAttributeSet::default());
        let duration_metric = pipeline_ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowDurationMetrics>(entity_key);
        let produced_items_metric = pipeline_ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowProducedItemsMetrics>(entity_key);
        let (metrics_rx, metrics_reporter) =
            otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
        let mut handler = local::EffectHandler::<TestMsg>::new(
            test_node("proc"),
            std::collections::HashMap::new(),
            None,
            metrics_reporter,
        );
        handler.set_flow_roles(
            false,
            true,
            None,
            Some(duration_metric),
            Some(produced_items_metric),
            None,
            true,
            true,
        );

        handler.record_flow_consumed_items(3);
        handler.record_flow_duration(10);
        handler.record_flow_produced_items(4);
        handler.report_flow_metrics();

        let duration_snapshot = metrics_rx
            .try_recv()
            .expect("duration metric should report");
        let [MetricValue::Mmsc(duration)] = duration_snapshot.get_metrics() else {
            panic!("expected duration metric");
        };
        assert_eq!(duration.count, 1);
        let produced_items_snapshot = metrics_rx
            .try_recv()
            .expect("produced item metric should report");
        let [MetricValue::U64(produced_items)] = produced_items_snapshot.get_metrics() else {
            panic!("expected produced item metric");
        };
        assert_eq!(*produced_items, 4);
        assert!(metrics_rx.try_recv().is_err());
    }

    #[test]
    fn flow_decision_node_reports_dropped() {
        let (pipeline_ctx, _) = crate::testing::test_pipeline_ctx();
        let entity_key = pipeline_ctx
            .metrics_registry()
            .register_entity(FlowAttributeSet::default());
        let dropped_metric = pipeline_ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowDroppedItemsMetrics>(entity_key);
        let (metrics_rx, metrics_reporter) =
            otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
        let mut handler = local::EffectHandler::<TestMsg>::new(
            test_node("proc"),
            std::collections::HashMap::new(),
            None,
            metrics_reporter,
        );
        // A decision node that is neither start nor end of the flow range.
        // Drop-only: needs no per-message timing.
        handler.set_flow_roles(
            false,
            false,
            None,
            None,
            None,
            Some(dropped_metric),
            true,
            false,
        );
        assert!(handler.is_flow_decision());

        handler.record_flow_dropped_items(3);
        // Recording consumed/produced items here must be a no-op (not a start/end node).
        handler.record_flow_consumed_items(99);
        handler.record_flow_produced_items(99);
        handler.report_flow_metrics();

        let dropped_snapshot = metrics_rx.try_recv().expect("dropped metric should report");
        let [MetricValue::U64(dropped_items)] = dropped_snapshot.get_metrics() else {
            panic!("expected dropped metric");
        };
        assert_eq!(*dropped_items, 3);
        assert!(metrics_rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn flow_metric_auto_measures_process_without_timed() {
        let (pipeline_ctx, _) = crate::testing::test_pipeline_ctx();
        let attrs = FlowAttributeSet {
            flow_id: "auto_measure".into(),
            start_node: "auto_measure_processor".into(),
            end_node: "auto_measure_processor".into(),
            purpose: "".into(),
            decision: "".into(),
            pipeline_attrs: pipeline_ctx.pipeline_attribute_set(),
        };
        let entity_key = pipeline_ctx.metrics_registry().register_entity(attrs);
        let start_metric_set = pipeline_ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowConsumedItemsMetrics>(entity_key);
        let duration_metric_set = pipeline_ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowDurationMetrics>(entity_key);
        let outgoing_metric_set = pipeline_ctx
            .metrics_registry()
            .register_metric_set_for_entity::<FlowProducedItemsMetrics>(entity_key);

        let config = crate::config::ProcessorConfig::new("auto_measure_processor");
        let node_id = test_node(config.name.clone());
        let user_config = Arc::new(NodeUserConfig::new_processor_config(
            "auto_measure_processor",
        ));
        let mut processor = ProcessorWrapper::local(
            SyncOnlyFlowMetricProcessor,
            node_id.clone(),
            user_config,
            &config,
        );

        let (input_tx, input_rx) = otap_df_channel::mpsc::Channel::new(1);
        processor
            .set_pdata_receiver(
                node_id.clone(),
                Receiver::Local(LocalReceiver::mpsc(input_rx)),
            )
            .expect("input receiver should be accepted");

        let (output_tx, output_rx) = otap_df_channel::mpsc::Channel::new(1);
        processor
            .set_pdata_sender(
                node_id,
                "out".into(),
                Sender::Local(LocalSender::mpsc(output_tx)),
            )
            .expect("output sender should be accepted");

        let control_sender = processor.control_sender();
        let (metrics_rx, metrics_reporter) =
            otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(8);
        let collect_metrics_reporter = metrics_reporter.clone();
        let (runtime_ctrl_tx, _runtime_ctrl_rx) = runtime_ctrl_msg_channel(1);
        let (completion_tx, _completion_rx) = pipeline_completion_msg_channel(1);

        let local_tasks = tokio::task::LocalSet::new();
        local_tasks
            .run_until(async move {
                let processor_task = tokio::task::spawn_local(async move {
                    processor
                        .start_with_completion_metrics(
                            runtime_ctrl_tx,
                            completion_tx,
                            metrics_reporter,
                            crate::Interests::PROCESS_DURATION,
                            None,
                            true,
                            true,
                            Some(start_metric_set),
                            Some(duration_metric_set),
                            Some(outgoing_metric_set),
                            None,
                            true,
                            true,
                            crate::terminal_state::TerminalMetricsDeadline::default(),
                        )
                        .await
                });

                input_tx
                    .send(FlowMetricTestPData::default())
                    .expect("test input should enqueue");
                let _ = output_rx
                    .recv()
                    .await
                    .expect("processor should forward the test message");
                control_sender
                    .send(NodeControlMsg::CollectTelemetry {
                        metrics_reporter: collect_metrics_reporter,
                    })
                    .await
                    .expect("collect telemetry should enqueue");

                let snapshot =
                    tokio::time::timeout(Duration::from_secs(1), metrics_rx.recv_async())
                        .await
                        .expect("flow_metric metric should be reported")
                        .expect("metrics channel should remain open");
                processor_task.abort();
                let _ = processor_task.await;

                let [MetricValue::U64(consumed_items)] = snapshot.get_metrics() else {
                    panic!("expected one start flow consumed-item metric");
                };
                assert_eq!(*consumed_items, 1);

                let snapshot =
                    tokio::time::timeout(Duration::from_secs(1), metrics_rx.recv_async())
                        .await
                        .expect("flow_metric stop metric should be reported")
                        .expect("metrics channel should remain open");
                let [MetricValue::Mmsc(compute_duration)] = snapshot.get_metrics() else {
                    panic!("expected flow duration MMSC metric");
                };
                assert!(
                    compute_duration.count >= 1,
                    "flow_metric compute duration should have at least one observation"
                );
                assert!(
                    compute_duration.sum > 0.0,
                    "flow_metric compute duration sum should be non-zero"
                );
                let snapshot =
                    tokio::time::timeout(Duration::from_secs(1), metrics_rx.recv_async())
                        .await
                        .expect("flow produced-item metric should be reported")
                        .expect("metrics channel should remain open");
                let [MetricValue::U64(produced_items)] = snapshot.get_metrics() else {
                    panic!("expected flow produced-item metric");
                };
                assert_eq!(*produced_items, 1);
            })
            .await;
    }
}
