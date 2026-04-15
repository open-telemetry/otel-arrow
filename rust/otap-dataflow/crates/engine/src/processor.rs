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
use crate::channel_mode::{LocalMode, SharedMode, wrap_control_channel_metrics};
use crate::completion_emission_metrics::CompletionEmissionMetricsHandle;
use crate::config::ProcessorConfig;
use crate::context::PipelineContext;
use crate::control::{
    Controllable, NodeControlMsg, PipelineCompletionMsgSender, RuntimeCtrlMsgSender,
};
use crate::effect_handler::SourceTagging;
use crate::entity_context::NodeTelemetryGuard;
use crate::error::{Error, ProcessorErrorKind};
use crate::local::message::{LocalReceiver, LocalSender};
use crate::local::processor as local;
use crate::message::{Message, ProcessorInbox, Receiver, Sender};
use crate::node::{Node, NodeId, NodeWithPDataReceiver, NodeWithPDataSender};
use crate::node_local_scheduler::NodeLocalSchedulerHandle;
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::shared::processor as shared;
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use otap_df_config::PortName;
use otap_df_config::node::NodeUserConfig;
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

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
}

impl ProcessorRuntimeRequirements {
    /// Runtime requirements for a processor that does not need any optional
    /// engine services.
    #[must_use]
    pub const fn none() -> Self {
        Self {
            local_wakeups: None,
        }
    }

    /// Runtime requirements for a processor that uses local wakeups.
    #[must_use]
    pub const fn with_local_wakeups(live_slots: usize) -> Self {
        Self {
            local_wakeups: Some(LocalWakeupRequirements::new(live_slots)),
        }
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
                    wrap_control_channel_metrics::<LocalMode, PData>(
                        &node_id,
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
                    wrap_control_channel_metrics::<SharedMode, PData>(
                        &node_id,
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
                runtime_config: _,
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
                let maybe_local_scheduler = runtime_requirements
                    .local_wakeups
                    .map(|requirements| NodeLocalSchedulerHandle::new(requirements.live_slots));
                let inbox = if let Some(local_scheduler) = maybe_local_scheduler.clone() {
                    ProcessorInbox::new_with_local_scheduler(
                        Receiver::Local(control_receiver),
                        pdata_receiver,
                        local_scheduler,
                        node_id.index,
                        node_interests,
                    )
                } else {
                    ProcessorInbox::new(
                        Receiver::Local(control_receiver),
                        pdata_receiver,
                        node_id.index,
                        node_interests,
                    )
                };
                let default_port = user_config.default_output.clone();
                let mut effect_handler = local::EffectHandler::new(
                    node_id,
                    pdata_senders,
                    default_port,
                    metrics_reporter,
                );
                effect_handler.set_source_tagging(source_tag);
                if let Some(local_scheduler) = maybe_local_scheduler {
                    effect_handler.core.set_local_scheduler(local_scheduler);
                }
                Ok(ProcessorWrapperRuntime::Local {
                    processor,
                    effect_handler,
                    inbox,
                })
            }
            ProcessorWrapper::Shared {
                node_id,
                runtime_config: _,
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
                let maybe_local_scheduler = runtime_requirements
                    .local_wakeups
                    .map(|requirements| NodeLocalSchedulerHandle::new(requirements.live_slots));
                let inbox = if let Some(local_scheduler) = maybe_local_scheduler.clone() {
                    ProcessorInbox::new_with_local_scheduler(
                        Receiver::Shared(control_receiver),
                        pdata_receiver,
                        local_scheduler,
                        node_id.index,
                        node_interests,
                    )
                } else {
                    ProcessorInbox::new(
                        Receiver::Shared(control_receiver),
                        pdata_receiver,
                        node_id.index,
                        node_interests,
                    )
                };
                let default_port = user_config.default_output.clone();
                let mut effect_handler = shared::EffectHandler::new(
                    node_id,
                    pdata_senders,
                    default_port,
                    metrics_reporter,
                );
                effect_handler.set_source_tagging(source_tag);
                if let Some(local_scheduler) = maybe_local_scheduler {
                    effect_handler.core.set_local_scheduler(local_scheduler);
                }
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
        PData: ReceivedAtNode,
    {
        self.start_with_completion_metrics(
            runtime_ctrl_msg_tx,
            pipeline_completion_msg_tx,
            metrics_reporter,
            node_interests,
            None,
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
    ) -> Result<(), Error>
    where
        PData: ReceivedAtNode,
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

                // Start periodic telemetry collection
                let telemetry_cancel_handle = effect_handler
                    .start_periodic_telemetry(Duration::from_secs(1))
                    .await?;

                while let Ok(msg) = inbox.recv_when(processor.accept_pdata()).await {
                    processor.process(msg, &mut effect_handler).await?;
                }
                // Cancel periodic collection
                _ = telemetry_cancel_handle.cancel().await;
                // Collect final metrics before exiting
                processor
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry { metrics_reporter }),
                        &mut effect_handler,
                    )
                    .await?
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

                // Start periodic telemetry collection
                let telemetry_cancel_handle = effect_handler
                    .start_periodic_telemetry(Duration::from_secs(1))
                    .await?;

                while let Ok(msg) = inbox.recv_when(processor.accept_pdata()).await {
                    processor.process(msg, &mut effect_handler).await?;
                }
                // Cancel periodic collection
                _ = telemetry_cancel_handle.cancel().await;
                // Collect final metrics before exiting
                processor
                    .process(
                        Message::Control(NodeControlMsg::CollectTelemetry { metrics_reporter }),
                        &mut effect_handler,
                    )
                    .await?
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
    use crate::control::NodeControlMsg::{Config, Shutdown, TimerTick};
    use crate::local::processor as local;
    use crate::message::Message;
    use crate::processor::{
        Error, ProcessorRuntimeRequirements, ProcessorWrapper, validate_local_wakeup_requirements,
    };
    use crate::shared::processor as shared;
    use crate::testing::processor::TestRuntime;
    use crate::testing::processor::{TestContext, ValidateContext};
    use crate::testing::{CtrlMsgCounters, TestMsg, test_node};
    use async_trait::async_trait;
    use otap_df_config::node::NodeUserConfig;
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
}
