// SPDX-License-Identifier: Apache-2.0

//! Processor wrapper used to provide a unified interface to the pipeline engine that abstracts over
//! the fact that processor implementations may be `!Send` or `Send`.
//!
//! For more details on the `!Send` implementation of a processor, see [`local::Processor`].
//! See [`shared::Processor`] for the Send implementation.

use crate::config::ProcessorConfig;
use crate::error::Error;
use crate::local::processor as local;
use crate::message::{ControlMsg, Message, Receiver, Sender};
use crate::shared::processor as shared;
use otap_df_channel::mpsc;

/// A wrapper for the processor that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the processor regardless of the effect
/// handler type. This is the only type that the pipeline engine will use in order to be agnostic to
/// the effect handler type.
pub enum ProcessorWrapper<PData> {
    /// A processor with a `!Send` implementation.
    Local {
        /// The processor instance.
        processor: Box<dyn local::Processor<PData>>,
        /// The effect handler for the processor.
        effect_handler: local::EffectHandler<PData>,
        /// A sender for control messages.
        control_sender: Sender<ControlMsg>,
        /// A receiver for control messages.
        control_receiver: Receiver<ControlMsg>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<Receiver<PData>>,
    },
    /// A processor with a `Send` implementation.
    Shared {
        /// The processor instance.
        processor: Box<dyn shared::Processor<PData>>,
        /// The effect handler for the processor.
        effect_handler: shared::EffectHandler<PData>,
        /// A sender for control messages.
        control_sender: tokio::sync::mpsc::Sender<ControlMsg>,
        /// A receiver for control messages.
        control_receiver: tokio::sync::mpsc::Receiver<ControlMsg>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<tokio::sync::mpsc::Receiver<PData>>,
    },
}

impl<PData> ProcessorWrapper<PData> {
    /// Creates a new local `ProcessorWrapper` with the given processor and appropriate effect handler.
    pub fn local<P>(processor: P, config: &ProcessorConfig) -> Self
    where
        P: local::Processor<PData> + 'static,
    {
        let (control_sender, control_receiver) =
            mpsc::Channel::new(config.control_channel.capacity);
        let (pdata_sender, pdata_receiver) =
            mpsc::Channel::new(config.output_pdata_channel.capacity);

        ProcessorWrapper::Local {
            processor: Box::new(processor),
            effect_handler: local::EffectHandler::new(
                config.name.clone(),
                Sender::Local(pdata_sender),
            ),
            control_sender: Sender::Local(control_sender),
            control_receiver: Receiver::Local(control_receiver),
            pdata_receiver: Some(Receiver::Local(pdata_receiver)),
        }
    }

    /// Creates a new shared `ProcessorWrapper` with the given processor and appropriate effect handler.
    pub fn shared<P>(processor: P, config: &ProcessorConfig) -> Self
    where
        P: shared::Processor<PData> + 'static,
    {
        let (control_sender, control_receiver) =
            tokio::sync::mpsc::channel(config.control_channel.capacity);
        let (pdata_sender, pdata_receiver) =
            tokio::sync::mpsc::channel(config.output_pdata_channel.capacity);

        ProcessorWrapper::Shared {
            processor: Box::new(processor),
            effect_handler: shared::EffectHandler::new(config.name.clone(), pdata_sender),
            control_sender,
            control_receiver,
            pdata_receiver: Some(pdata_receiver),
        }
    }

    /// Call the processor's `process` method.
    pub async fn process(&mut self, msg: Message<PData>) -> Result<(), Error<PData>> {
        match self {
            ProcessorWrapper::Local {
                effect_handler,
                processor,
                ..
            } => processor.process(msg, effect_handler).await,
            ProcessorWrapper::Shared {
                effect_handler,
                processor,
                ..
            } => processor.process(msg, effect_handler).await,
        }
    }

    /// Takes the PData receiver from the wrapper and returns it.
    pub fn take_pdata_receiver(&mut self) -> Receiver<PData> {
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

#[cfg(test)]
mod tests {
    use crate::local::processor as local;
    use crate::message::ControlMsg::{Config, Shutdown, TimerTick};
    use crate::message::Message;
    use crate::processor::{Error, ProcessorWrapper};
    use crate::shared::processor as shared;
    use crate::testing::processor::TestRuntime;
    use crate::testing::processor::{TestContext, ValidateContext};
    use crate::testing::{CtrlMsgCounters, TestMsg};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::pin::Pin;
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
        let processor = ProcessorWrapper::local(
            TestProcessor::new(test_runtime.counters()),
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
        let processor = ProcessorWrapper::shared(
            TestProcessor::new(test_runtime.counters()),
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure());
    }
}
