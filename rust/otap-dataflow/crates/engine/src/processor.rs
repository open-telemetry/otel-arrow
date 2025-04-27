// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement processors.
//!
//! A processor is a node in the pipeline that transforms, filters, or otherwise processes messages
//! as they flow through the pipeline. Processors can perform operations such as:
//!
//! 1. Filtering messages based on certain criteria
//! 2. Transforming message content or format
//! 3. Aggregating multiple messages into a single message
//! 4. Splitting a single message into multiple messages
//! 5. Adding or removing attributes from messages
//!
//! # Lifecycle
//!
//! 1. The processor is instantiated and configured
//! 2. The processor receives and processes both data messages and control messages
//! 3. For each message, the processor can transform it, filter it, or split it into multiple messages
//! 4. The processor can maintain state between processing calls if needed
//! 5. The processor responds to control messages such as Config, TimerTick, or Shutdown
//! 6. The processor shuts down when it receives a `Shutdown` control message or encounters a fatal error
//!
//! # Thread Safety
//!
//! Note that this trait uses `#[async_trait(?Send)]`, meaning implementations
//! are not required to be thread-safe. If you need to implement a processor that requires `Send`,
//! you can use the [`SendEffectHandler`] type. The default effect handler is `!Send` (see
//! [`NotSendEffectHandler`]).
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own processor instance.

use crate::config::ProcessorConfig;
use crate::error::Error;
use crate::message::{Message, PDataReceiver};
use async_trait::async_trait;
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use std::rc::Rc;
use std::sync::Arc;

/// A trait for processors in the pipeline.
#[async_trait(?Send)]
pub trait Processor<PData, EF = NotSendEffectHandler<PData>>
where
    EF: EffectHandlerTrait<PData>,
{
    /// Processes a message and optionally produces new messages.
    ///
    /// This method is called by the pipeline engine for each message that arrives at the processor.
    /// Unlike receivers, processors have known inputs (messages from previous stages), so the pipeline
    /// engine can control when to call this method and when the processor executes.
    ///
    /// This approach allows for greater flexibility and optimization, giving the pipeline engine
    /// the ability to decide whether to spawn one task per processor or one task for a group of processors.
    /// The method signature uses `&mut self` rather than `Box<Self>` because the engine only wants to
    /// temporarily allow mutation of the processor instance, not transfer ownership.
    ///
    /// The processor can:
    /// - Transform the message and return a new message
    /// - Filter the message by returning None
    /// - Split the message into multiple messages by returning a vector
    /// - Handle control messages (e.g., Config, TimerTick, Shutdown)
    ///
    /// # Parameters
    ///
    /// - `msg`: The message to process, which can be either a data message or a control message
    /// - `effect_handler`: A handler to perform side effects such as sending messages to the next node.
    ///    This can be either Send or !Send depending on the processor's Mode type.
    ///
    /// # Returns
    ///
    /// - `Ok(())`: The processor successfully processed the message
    /// - `Err(Error)`: The processor encountered an error and could not process the message
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the processor encounters an unrecoverable error.
    async fn process(
        &mut self,
        msg: Message<PData>,
        effect_handler: &mut EF,
    ) -> Result<(), Error<PData>>;
}

/// Handles side effects for the processor.
///
/// The `PData` type parameter represents the type of message the processor will consume and
/// produce.
///
/// 2 implementations are provided:
///
/// - [`NotSendEffectHandler<PData>`]: For thread-local (!Send) processors. Uses `Rc` internally.
///   It's the default and preferred effect handler.
/// - [`SendEffectHandler<PData>`]: For thread-safe (Send) exporters. Uses `Arc` internally and
///   supports sending across thread boundaries.
///
/// Note for implementers: Effect handler implementations are designed to be cloned so the cost of
/// cloning should be minimal.
#[async_trait(?Send)]
pub trait EffectHandlerTrait<PData> {
    /// Returns the name of the processor associated with this handler.
    fn processor_name(&self) -> &str;

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    async fn send_message(&self, data: PData) -> Result<(), Error<PData>>;

    // More methods will be added in the future as needed.
}

/// A `!Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct NotSendEffectHandler<PData> {
    /// The name of the processor.
    processor_name: Rc<str>,
    /// A sender used to forward messages from the processor.
    msg_sender: mpsc::Sender<PData>,
}

/// Implementation for the `!Send` effect handler.
impl<PData> NotSendEffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given processor name.
    /// This is the default and preferred effect handler for this project.
    ///
    /// Use this constructor when your processor doesn't need to be sent across threads or
    /// when it uses components that aren't `Send`.
    pub fn new<S: AsRef<str>>(processor_name: S, msg_sender: mpsc::Sender<PData>) -> Self {
        NotSendEffectHandler {
            processor_name: Rc::from(processor_name.as_ref()),
            msg_sender,
        }
    }
}

#[async_trait(?Send)]
impl<PData> EffectHandlerTrait<PData> for NotSendEffectHandler<PData> {
    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    fn processor_name(&self) -> &str {
        &self.processor_name
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ReceiverError`] if the message could not be sent.
    async fn send_message(&self, data: PData) -> Result<(), Error<PData>> {
        self.msg_sender.send_async(data).await?;
        Ok(())
    }
}

/// A `Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct SendEffectHandler<PData> {
    /// The name of the processor.
    processor_name: Arc<str>,
    /// A sender used to forward messages from the processor.
    msg_sender: tokio::sync::mpsc::Sender<PData>,
}

/// Implementation for the `Send` effect handler.
impl<PData> SendEffectHandler<PData> {
    /// Creates a new "sendable" effect handler with the given exporter name.
    pub fn new<S: AsRef<str>>(
        processor_name: S,
        msg_sender: tokio::sync::mpsc::Sender<PData>,
    ) -> Self {
        SendEffectHandler {
            processor_name: Arc::from(processor_name.as_ref()),
            msg_sender,
        }
    }
}

#[async_trait(?Send)]
impl<PData> EffectHandlerTrait<PData> for SendEffectHandler<PData> {
    /// Returns the name of the processor associated with this handler.
    #[must_use]
    fn processor_name(&self) -> &str {
        &self.processor_name
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ReceiverError`] if the message could not be sent.
    async fn send_message(&self, data: PData) -> Result<(), Error<PData>> {
        self.msg_sender
            .send(data)
            .await
            .map_err(|tokio::sync::mpsc::error::SendError(pdata)| {
                Error::ChannelSendError(SendError::Full(pdata))
            })
    }
}

/// A wrapper for the processor that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the processor regardless of the effect
/// handler type. This is the only type that the pipeline engine will use in order to be agnostic to
/// the effect handler type.
pub enum ProcessorWrapper<PData> {
    /// A processor with a `!Send` effect handler.
    NotSend {
        /// The processor instance.
        processor: Box<dyn Processor<PData, NotSendEffectHandler<PData>>>,
        /// The effect handler for the processor.
        effect_handler: NotSendEffectHandler<PData>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<mpsc::Receiver<PData>>,
    },
    /// A processor with a `Send` effect handler.
    Send {
        /// The processor instance.
        processor: Box<dyn Processor<PData, SendEffectHandler<PData>>>,
        /// The effect handler for the processor.
        effect_handler: SendEffectHandler<PData>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<tokio::sync::mpsc::Receiver<PData>>,
    },
}

impl<PData> ProcessorWrapper<PData> {
    /// Creates a new `ProcessorWrapper` with the given processor and `!Send` effect handler.
    pub fn with_not_send<P>(processor: P, config: &ProcessorConfig) -> Self
    where
        P: Processor<PData, NotSendEffectHandler<PData>> + 'static,
    {
        let (pdata_sender, pdata_receiver) =
            mpsc::Channel::new(config.output_pdata_channel.capacity);
        ProcessorWrapper::NotSend {
            effect_handler: NotSendEffectHandler::new(&config.name, pdata_sender),
            processor: Box::new(processor),
            pdata_receiver: Some(pdata_receiver),
        }
    }

    /// Creates a new `ProcessorWrapper` with the given processor and `Send` effect handler.
    pub fn with_send<P>(processor: P, config: &ProcessorConfig) -> Self
    where
        P: Processor<PData, SendEffectHandler<PData>> + 'static,
    {
        let (pdata_sender, pdata_receiver) =
            tokio::sync::mpsc::channel(config.output_pdata_channel.capacity);
        ProcessorWrapper::Send {
            effect_handler: SendEffectHandler::new(&config.name, pdata_sender),
            processor: Box::new(processor),
            pdata_receiver: Some(pdata_receiver),
        }
    }

    /// Call the processor's `process` method.
    pub async fn process(&mut self, msg: Message<PData>) -> Result<(), Error<PData>> {
        match self {
            ProcessorWrapper::NotSend {
                effect_handler,
                processor,
                ..
            } => processor.process(msg, effect_handler).await,
            ProcessorWrapper::Send {
                effect_handler,
                processor,
                ..
            } => processor.process(msg, effect_handler).await,
        }
    }

    /// Takes the PData receiver from the wrapper and returns it.
    pub fn take_pdata_receiver(&mut self) -> PDataReceiver<PData> {
        match self {
            ProcessorWrapper::NotSend { pdata_receiver, .. } => {
                PDataReceiver::NotSend(pdata_receiver.take().expect("pdata_receiver is None"))
            }
            ProcessorWrapper::Send { pdata_receiver, .. } => {
                PDataReceiver::Send(pdata_receiver.take().expect("pdata_receiver is None"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::ControlMsg::{Config, Shutdown, TimerTick};
    use crate::message::Message;
    use crate::processor::{
        EffectHandlerTrait, Error, NotSendEffectHandler, Processor, ProcessorWrapper,
        SendEffectHandler,
    };
    use crate::testing::processor::TestRuntime;
    use crate::testing::processor::{TestContext, ValidateContext};
    use crate::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::pin::Pin;

    /// A generic test processor that counts message events
    /// Works with any effect handler that implements EffectHandlerTrait
    pub struct GenericTestProcessor<EF> {
        /// Counter for different message types
        ctrl_msg_counters: CtrlMsgCounters,
        /// Optional callback for testing sendable effect handlers
        test_send_eh: Option<fn(&EF)>,
    }

    impl<EF> GenericTestProcessor<EF> {
        /// Creates a new test node with the given counter
        pub fn new(ctrl_msg_counters: CtrlMsgCounters) -> Self {
            GenericTestProcessor {
                ctrl_msg_counters,
                test_send_eh: None,
            }
        }

        /// Creates a new test node with a callback for PData messages
        pub fn with_send_effect_handler(
            ctrl_msg_counters: CtrlMsgCounters,
            callback: fn(&EF),
        ) -> Self {
            GenericTestProcessor {
                ctrl_msg_counters,
                test_send_eh: Some(callback),
            }
        }
    }

    #[async_trait(?Send)]
    impl<EF> Processor<TestMsg, EF> for GenericTestProcessor<EF>
    where
        EF: EffectHandlerTrait<TestMsg> + Clone + 'static,
    {
        async fn process(
            &mut self,
            msg: Message<TestMsg>,
            effect_handler: &mut EF,
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
                    if let Some(test_send_eh) = self.test_send_eh {
                        // Call the test callback if provided.
                        test_send_eh(&effect_handler);
                    }
                    effect_handler
                        .send_message(TestMsg(format!("{} RECEIVED", data.0)))
                        .await?;
                }
            }
            Ok(())
        }
    }

    /// A type alias for a test processor with regular effect handler
    type ProcessorWithNotSendEffectHandler = GenericTestProcessor<NotSendEffectHandler<TestMsg>>;

    /// A type alias for a test processor with sendable effect handler
    type ProcessorWithSendEffectHandler = GenericTestProcessor<SendEffectHandler<TestMsg>>;

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
                ctx.process(Message::shutdown_ctrl_msg("no reason"))
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
    fn test_receiver_with_not_send_effect_handler() {
        let test_runtime = TestRuntime::new();
        let processor = ProcessorWrapper::with_not_send(
            ProcessorWithNotSendEffectHandler::new(test_runtime.counters()),
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure());
    }

    #[test]
    fn test_receiver_with_send_effect_handler() {
        let test_runtime = TestRuntime::new();
        let processor = ProcessorWrapper::with_send(
            ProcessorWithSendEffectHandler::with_send_effect_handler(
                test_runtime.counters(),
                |eh| {
                    exec_in_send_env(|| {
                        _ = eh.processor_name();
                    });
                },
            ),
            test_runtime.config(),
        );

        test_runtime
            .set_processor(processor)
            .run_test(scenario())
            .validate(validation_procedure());
    }
}
