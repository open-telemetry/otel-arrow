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
//! are not required to be thread-safe. To ensure scalability, the pipeline engine will start
//! multiple instances of the same pipeline in parallel, each with its own processor instance.

use crate::NodeName;
use crate::error::Error;
use crate::message::Message;
use crate::receiver::{LocalMode, SendableMode, ThreadMode};
use async_trait::async_trait;
use otap_df_channel::mpsc;
use std::marker::PhantomData;
use std::rc::Rc;
use std::sync::Arc;

/// A trait for processors in the pipeline.
#[async_trait(?Send)]
pub trait Processor {
    /// The type of messages handled by the processor.
    type PData;

    /// The threading mode used by this processor
    type Mode: ThreadMode;

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
        msg: Message<Self::PData>,
        effect_handler: &mut EffectHandler<Self::PData, Self::Mode>,
    ) -> Result<(), Error<Self::PData>>;
}

/// Handles side effects such as sending messages to the next node.
///
/// The `Msg` type parameter represents the type of message the processor
/// will eventually produce.
///
/// Note for implementers: The `EffectHandler` is designed to be cloned and shared across tasks
/// so the cost of cloning should be minimal.
pub struct EffectHandler<Msg, Mode: ThreadMode = LocalMode> {
    /// The name of the processor.
    processor_name: Mode::NameRef,
    /// A sender used to forward messages from the processor.
    msg_sender: mpsc::Sender<Msg>,
    /// Marker for the thread mode.
    _mode: PhantomData<Mode>,
}

impl<Msg, Mode: ThreadMode> Clone for EffectHandler<Msg, Mode> {
    fn clone(&self) -> Self {
        EffectHandler {
            processor_name: self.processor_name.clone(),
            msg_sender: self.msg_sender.clone(),
            _mode: PhantomData,
        }
    }
}

// Implementation for any mode
impl<Msg, Mode: ThreadMode> EffectHandler<Msg, Mode> {
    /// Returns the name of the processor associated with this handler.
    #[must_use]
    pub fn processor_name(&self) -> NodeName {
        // Convert to NodeName (Rc<str>) to maintain compatibility with existing API
        Rc::from(self.processor_name.as_ref())
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    pub async fn send_message(&self, data: Msg) -> Result<(), Error<Msg>> {
        self.msg_sender.send_async(data).await?;
        Ok(())
    }
}

// Implementation specific to LocalMode (default, non-Send)
impl<Msg> EffectHandler<Msg, LocalMode> {
    /// Creates a new local (non-Send) `EffectHandler` with the given processor name.
    /// This is the default mode that maintains backward compatibility.
    pub fn new<S: AsRef<str>>(processor_name: S, msg_sender: mpsc::Sender<Msg>) -> Self {
        EffectHandler {
            processor_name: Rc::from(processor_name.as_ref()),
            msg_sender,
            _mode: PhantomData,
        }
    }
}

// Implementation for SendableMode (Send)
impl<Msg: Send + 'static> EffectHandler<Msg, SendableMode> {
    /// Creates a new thread-safe (Send) `EffectHandler` with the given processor name.
    /// Use this when you need an EffectHandler that can be sent across thread boundaries.
    pub fn new_sendable<S: AsRef<str>>(processor_name: S, msg_sender: mpsc::Sender<Msg>) -> Self {
        EffectHandler {
            processor_name: Arc::from(processor_name.as_ref()),
            msg_sender,
            _mode: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::message::ControlMsg::{Config, Shutdown, TimerTick};
    use crate::message::Message;
    use crate::processor::{EffectHandler, Error, Processor};
    use crate::receiver::LocalMode;
    use crate::testing::processor::ProcessorTestRuntime;
    use crate::testing::{CtrMsgCounters, TestMsg};
    use async_trait::async_trait;
    use serde_json::Value;

    struct TestProcessor {
        counters: CtrMsgCounters,
    }

    #[async_trait(?Send)]
    impl Processor for TestProcessor {
        type PData = TestMsg;
        type Mode = LocalMode;

        async fn process(
            &mut self,
            msg: Message<Self::PData>,
            effect_handler: &mut EffectHandler<Self::PData, Self::Mode>,
        ) -> Result<(), Error<Self::PData>> {
            match msg {
                Message::Control(control) => match control {
                    TimerTick {} => {
                        self.counters.increment_timer_tick();
                    }
                    Config { .. } => {
                        self.counters.increment_config();
                    }
                    Shutdown { .. } => {
                        self.counters.increment_shutdown();
                    }
                    _ => {}
                },
                Message::PData(data) => {
                    self.counters.increment_message();
                    effect_handler
                        .send_message(TestMsg(format!("{} RECEIVED", data.0)))
                        .await?;
                }
            }
            Ok(())
        }
    }

    #[test]
    fn test_processor() {
        let counters = CtrMsgCounters::new();
        let mut test_runtime = ProcessorTestRuntime::new(
            TestProcessor {
                counters: counters.clone(),
            },
            10,
        );

        test_runtime.start_test(|mut context| async move {
            // Process a TimerTick event.
            context
                .process(Message::timer_tick_ctrl_msg())
                .await
                .expect("Processor failed on TimerTick");
            assert!(context.drain_pdata().await.is_empty());

            // Process a Message event.
            context
                .process(Message::data_msg(TestMsg("Hello".to_owned())))
                .await
                .expect("Processor failed on Message");
            let msgs = context.drain_pdata().await;
            assert_eq!(msgs.len(), 1);
            assert_eq!(msgs[0], TestMsg("Hello RECEIVED".to_string()));

            // Process a Config event.
            context
                .process(Message::config_ctrl_msg(Value::Null))
                .await
                .expect("Processor failed on Config");
            assert!(context.drain_pdata().await.is_empty());

            // Process a Shutdown event.
            context
                .process(Message::shutdown_ctrl_msg("no reason"))
                .await
                .expect("Processor failed on Shutdown");
            assert!(context.drain_pdata().await.is_empty());
        });
        test_runtime.validate(|| async move {
            counters.assert(
                1, // timer tick
                1, // message
                1, // config
                1, // shutdown
            );
        });
    }
}
