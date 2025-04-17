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
use async_trait::async_trait;
use otap_df_channel::mpsc;
use std::rc::Rc;

/// A trait for processors in the pipeline.
#[async_trait(?Send)]
pub trait Processor {
    /// The type of messages handled by the processor.
    type PData;

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
    /// - `effect_handler`: A handler to perform side effects such as sending messages to the next node
    ///
    /// # Returns
    ///
    /// - `Ok(Some(Vec<Msg>))`: The processor produced new messages to send to the next node
    /// - `Ok(None)`: The processor consumed the message without producing new messages
    /// - `Err(Error)`: The processor encountered an error and could not process the message
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if the processor encounters an unrecoverable error.
    async fn process(
        &mut self,
        msg: Message<Self::PData>,
        effect_handler: &mut EffectHandler<Self::PData>,
    ) -> Result<Option<Vec<Self::PData>>, Error<Self::PData>>;
}

/// Handles side effects such as sending messages to the next node.
///
/// The `Msg` type parameter represents the type of message the processor
/// will eventually produce.
///
/// Note for implementers: The `EffectHandler` is designed to be cloned and shared across tasks
/// so the cost of cloning should be minimal.
pub struct EffectHandler<Msg> {
    /// The name of the processor.
    processor_name: NodeName,
    /// A sender used to forward messages from the processor.
    msg_sender: mpsc::Sender<Msg>,
}

impl<Msg> Clone for EffectHandler<Msg> {
    fn clone(&self) -> Self {
        EffectHandler {
            processor_name: self.processor_name.clone(),
            msg_sender: self.msg_sender.clone(),
        }
    }
}

impl<Msg> EffectHandler<Msg> {
    /// Creates a new `EffectHandler` with the given processor name and message sender.
    pub fn new<S: AsRef<str>>(processor_name: S, msg_sender: mpsc::Sender<Msg>) -> Self {
        EffectHandler {
            processor_name: Rc::from(processor_name.as_ref()),
            msg_sender,
        }
    }

    /// Returns the name of the processor associated with this handler.
    #[must_use]
    pub fn processor_name(&self) -> NodeName {
        self.processor_name.clone()
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

#[cfg(test)]
mod tests {
    use crate::message::ControlMsg::{Config, Shutdown, TimerTick};
    use crate::message::Message;
    use crate::processor::{EffectHandler, Error, Processor};
    use crate::testing::{MessageCounter, TestMsg, setup_test_runtime};
    use async_trait::async_trait;
    use otap_df_channel::mpsc;
    use serde_json::Value;

    struct TestProcessor {
        counters: MessageCounter,
    }

    #[async_trait(?Send)]
    impl Processor for TestProcessor {
        type PData = TestMsg;

        async fn process(
            &mut self,
            msg: Message<Self::PData>,
            _effect_handler: &mut EffectHandler<Self::PData>,
        ) -> Result<Option<Vec<Self::PData>>, Error<Self::PData>> {
            match msg {
                Message::Control(control) => match control {
                    TimerTick {} => {
                        self.counters.increment_timer_tick();
                        Ok(None)
                    }
                    Config { .. } => {
                        self.counters.increment_config();
                        Ok(None)
                    }
                    Shutdown { .. } => {
                        self.counters.increment_shutdown();
                        Ok(None)
                    }
                    _ => Ok(None),
                },
                Message::PData(data) => {
                    self.counters.increment_message();
                    // Append " RECEIVED" to the message content.
                    let processed_message = TestMsg(format!("{} RECEIVED", data.0));
                    Ok(Some(vec![processed_message]))
                }
            }
        }
    }

    #[test]
    fn test_processor() {
        let (rt, local_tasks) = setup_test_runtime();
        let counters = MessageCounter::new();
        let mut processor = Box::new(TestProcessor {
            counters: counters.clone(),
        });

        // Create a channel for the effect handler
        let (tx, _rx) = mpsc::Channel::new(10);

        // Spawn the processor's event loop.
        _ = local_tasks.spawn_local(async move {
            let mut effect_handler = EffectHandler::new("test_processor", tx);

            // Process a TimerTick event.
            let result = processor
                .process(Message::timer_tick_ctrl_msg(), &mut effect_handler)
                .await
                .expect("Processor failed on TimerTick");
            assert!(result.is_none());

            // Process a Message event.
            let result = processor
                .process(
                    Message::data_msg(TestMsg("Hello".to_owned())),
                    &mut effect_handler,
                )
                .await
                .expect("Processor failed on Message");
            let msgs = result.expect("Expected a message vector for Event::Message");
            assert_eq!(msgs.len(), 1);
            assert_eq!(msgs[0], TestMsg("Hello RECEIVED".to_string()));

            // Process a Config event.
            let result = processor
                .process(Message::config_ctrl_msg(Value::Null), &mut effect_handler)
                .await
                .expect("Processor failed on Config");
            assert!(result.is_none());

            // Process a Shutdown event.
            let result = processor
                .process(Message::shutdown_ctrl_msg("no reason"), &mut effect_handler)
                .await
                .expect("Processor failed on Shutdown");
            assert!(result.is_none());
        });

        // Run all tasks.
        rt.block_on(local_tasks);

        // Finally, verify that each counter was updated exactly once.
        assert_eq!(
            counters.get_timer_tick_count(),
            1,
            "TimerTick count mismatch"
        );
        assert_eq!(counters.get_message_count(), 1, "Message count mismatch");
        assert_eq!(counters.get_config_count(), 1, "Config count mismatch");
        assert_eq!(counters.get_shutdown_count(), 1, "Shutdown count mismatch");
    }
}
