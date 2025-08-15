// SPDX-License-Identifier: Apache-2.0

//! Processor wrapper used to provide a unified interface to the pipeline engine that abstracts over
//! the fact that processor implementations may be `!Send` or `Send`.
//!
//! For more details on the `!Send` implementation of a processor, see [`local::Processor`].
//! See [`shared::Processor`] for the Send implementation.

use crate::config::ProcessorConfig;
use crate::control::{Controllable, NodeControlMsg, PipelineCtrlMsgSender};
use crate::error::Error;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::local::processor as local;
use crate::message::{MessageChannel, Receiver, Sender};
use crate::node::{Node, NodeId, NodeWithPDataReceiver, NodeWithPDataSender};
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::shared::processor as shared;
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use otap_df_config::PortName;
use otap_df_config::node::NodeUserConfig;
use std::collections::HashMap;
use std::sync::Arc;

/// A wrapper for the processor that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the processor regardless of the effect
/// handler type. This is the only type that the pipeline engine will use in order to be agnostic to
/// the effect handler type.
pub enum ProcessorWrapper<PData> {
    /// A processor with a `!Send` implementation.
    Local {
        /// Index node identifier.
        node: NodeId,
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the processor.
        runtime_config: ProcessorConfig,
        /// The processor instance.
        processor: Box<dyn local::Processor<PData>>,
        /// A sender for control messages.
        control_sender: LocalSender<NodeControlMsg>,
        /// A receiver for control messages.
        control_receiver: LocalReceiver<NodeControlMsg>,
        /// Senders for PData messages per out port.
        pdata_senders: HashMap<PortName, LocalSender<PData>>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<LocalReceiver<PData>>,
    },
    /// A processor with a `Send` implementation.
    Shared {
        /// Index node identifier.
        node: NodeId,
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the processor.
        runtime_config: ProcessorConfig,
        /// The processor instance.
        processor: Box<dyn shared::Processor<PData>>,
        /// A sender for control messages.
        control_sender: SharedSender<NodeControlMsg>,
        /// A receiver for control messages.
        control_receiver: SharedReceiver<NodeControlMsg>,
        /// Senders for PData messages per out port.
        pdata_senders: HashMap<PortName, SharedSender<PData>>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<SharedReceiver<PData>>,
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
        /// The message channel
        message_channel: MessageChannel<PData>,
        /// The local effect handler
        effect_handler: local::EffectHandler<PData>,
    },
    /// A processor with a `Send` implementation.
    Shared {
        /// The processor instance.
        processor: Box<dyn shared::Processor<PData>>,
        /// Message channel
        message_channel: MessageChannel<PData>,
        /// The shared effect handler
        effect_handler: shared::EffectHandler<PData>,
    },
}

impl<PData> ProcessorWrapper<PData> {
    /// Creates a new local `ProcessorWrapper` with the given processor and appropriate effect handler.
    pub fn local<P>(
        processor: P,
        node: NodeId,
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
            node,
            user_config,
            runtime_config,
            processor: Box::new(processor),
            control_sender: LocalSender::MpscSender(control_sender),
            control_receiver: LocalReceiver::MpscReceiver(control_receiver),
            pdata_senders: HashMap::new(),
            pdata_receiver: None,
        }
    }

    /// Creates a new shared `ProcessorWrapper` with the given processor and appropriate effect handler.
    pub fn shared<P>(
        processor: P,
        node: NodeId,
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
            node,
            user_config,
            runtime_config,
            processor: Box::new(processor),
            control_sender: SharedSender::MpscSender(control_sender),
            control_receiver: SharedReceiver::MpscReceiver(control_receiver),
            pdata_senders: HashMap::new(),
            pdata_receiver: None,
        }
    }

    /// Prepare the processor runtime components without starting the processing loop.
    /// This allows external control over the message processing loop.
    pub async fn prepare_runtime(self) -> Result<ProcessorWrapperRuntime<PData>, Error<PData>> {
        match self {
            ProcessorWrapper::Local {
                node,
                processor,
                runtime_config,
                control_receiver,
                pdata_senders,
                pdata_receiver,
                user_config,
                ..
            } => {
                let message_channel = MessageChannel::new(
                    Receiver::Local(control_receiver),
                    Receiver::Local(pdata_receiver.ok_or_else(|| Error::ProcessorError {
                        processor: runtime_config.name.clone(),
                        error: "The pdata receiver must be defined at this stage".to_owned(),
                    })?),
                );
                let default_port = user_config.default_out_port.clone();
                let effect_handler = local::EffectHandler::new(node, pdata_senders, default_port);
                Ok(ProcessorWrapperRuntime::Local {
                    processor,
                    effect_handler,
                    message_channel,
                })
            }
            ProcessorWrapper::Shared {
                node,
                processor,
                runtime_config,
                control_receiver,
                pdata_senders,
                pdata_receiver,
                user_config,
                ..
            } => {
                let message_channel = MessageChannel::new(
                    Receiver::Shared(control_receiver),
                    Receiver::Shared(pdata_receiver.ok_or_else(|| Error::ProcessorError {
                        processor: runtime_config.name.clone(),
                        error: "The pdata receiver must be defined at this stage".to_owned(),
                    })?),
                );
                let default_port = user_config.default_out_port.clone();
                let effect_handler = shared::EffectHandler::new(node, pdata_senders, default_port);
                Ok(ProcessorWrapperRuntime::Shared {
                    processor,
                    effect_handler,
                    message_channel,
                })
            }
        }
    }

    /// Start the processor and run the message processing loop.
    pub async fn start(
        self,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender,
    ) -> Result<(), Error<PData>> {
        let runtime = self.prepare_runtime().await?;

        match runtime {
            ProcessorWrapperRuntime::Local {
                mut processor,
                mut message_channel,
                mut effect_handler,
            } => {
                effect_handler
                    .core
                    .set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);
                while let Ok(msg) = message_channel.recv().await {
                    processor.process(msg, &mut effect_handler).await?;
                }
            }
            ProcessorWrapperRuntime::Shared {
                mut processor,
                mut message_channel,
                mut effect_handler,
            } => {
                effect_handler
                    .core
                    .set_pipeline_ctrl_msg_sender(pipeline_ctrl_msg_tx);
                while let Ok(msg) = message_channel.recv().await {
                    processor.process(msg, &mut effect_handler).await?;
                }
            }
        }
        Ok(())
    }

    /// Takes the PData receiver from the wrapper and returns it.
    pub fn take_pdata_receiver(&mut self) -> Receiver<PData> {
        match self {
            ProcessorWrapper::Local { pdata_receiver, .. } => {
                Receiver::Local(pdata_receiver.take().expect("pdata_receiver is None"))
            }
            ProcessorWrapper::Shared { pdata_receiver, .. } => {
                Receiver::Shared(pdata_receiver.take().expect("pdata_receiver is None"))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<PData> Node for ProcessorWrapper<PData> {
    fn is_shared(&self) -> bool {
        match self {
            ProcessorWrapper::Local { .. } => false,
            ProcessorWrapper::Shared { .. } => true,
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            ProcessorWrapper::Local { node, .. } => node.clone(),
            ProcessorWrapper::Shared { node, .. } => node.clone(),
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
    async fn send_control_msg(&self, msg: NodeControlMsg) -> Result<(), SendError<NodeControlMsg>> {
        match self {
            ProcessorWrapper::Local { control_sender, .. } => control_sender.send(msg).await,
            ProcessorWrapper::Shared { control_sender, .. } => control_sender.send(msg).await,
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<PData> Controllable for ProcessorWrapper<PData> {
    /// Sends a control message to the node.
    async fn send_node_control_msg(
        &self,
        msg: NodeControlMsg,
    ) -> Result<(), SendError<NodeControlMsg>> {
        self.control_sender().send(msg).await
    }

    /// Returns the control message sender for the processor.
    fn control_sender(&self) -> Sender<NodeControlMsg> {
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
        node: NodeId,
        port: PortName,
        sender: Sender<PData>,
    ) -> Result<(), Error<PData>> {
        match (self, sender) {
            (ProcessorWrapper::Local { pdata_senders, .. }, Sender::Local(sender)) => {
                let _ = pdata_senders.insert(port, sender);
                Ok(())
            }
            (ProcessorWrapper::Shared { pdata_senders, .. }, Sender::Shared(sender)) => {
                let _ = pdata_senders.insert(port, sender);
                Ok(())
            }
            (ProcessorWrapper::Local { .. }, _) => Err(Error::ProcessorError {
                processor: node.name.clone(),
                error: "Expected a local sender for PData".to_owned(),
            }),
            (ProcessorWrapper::Shared { .. }, _) => Err(Error::ProcessorError {
                processor: node.name.clone(),
                error: "Expected a shared sender for PData".to_owned(),
            }),
        }
    }
}

impl<PData> NodeWithPDataReceiver<PData> for ProcessorWrapper<PData> {
    fn set_pdata_receiver(
        &mut self,
        node: NodeId,
        receiver: Receiver<PData>,
    ) -> Result<(), Error<PData>> {
        match (self, receiver) {
            (ProcessorWrapper::Local { pdata_receiver, .. }, Receiver::Local(receiver)) => {
                *pdata_receiver = Some(receiver);
                Ok(())
            }
            (ProcessorWrapper::Shared { pdata_receiver, .. }, Receiver::Shared(receiver)) => {
                *pdata_receiver = Some(receiver);
                Ok(())
            }
            (ProcessorWrapper::Local { .. }, _) => Err(Error::ProcessorError {
                processor: node.name.clone(),
                error: "Expected a local sender for PData".to_owned(),
            }),
            (ProcessorWrapper::Shared { .. }, _) => Err(Error::ProcessorError {
                processor: node.name.clone(),
                error: "Expected a shared sender for PData".to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::control::NodeControlMsg::{Config, Shutdown, TimerTick};
    use crate::local::processor as local;
    use crate::message::Message;
    use crate::processor::{Error, ProcessorWrapper};
    use crate::shared::processor as shared;
    use crate::testing::processor::TestRuntime;
    use crate::testing::processor::{TestContext, ValidateContext};
    use crate::testing::{CtrlMsgCounters, TestMsg, test_node};
    use async_trait::async_trait;
    use otap_df_config::node::NodeUserConfig;
    use serde_json::Value;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::time::Duration;

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
        ) -> Result<(), Error<TestMsg>> {
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
        ) -> Result<(), Error<TestMsg>> {
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
                    Duration::from_millis(200),
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
}
