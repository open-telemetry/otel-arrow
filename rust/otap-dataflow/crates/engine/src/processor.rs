// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement processors.
//!
//! A processor is a node in the dataflow that transforms, filters, or otherwise processes messages
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
//! 2. The `process` method is called for each incoming message
//! 3. The processor processes both internal control messages and data messages
//! 4. The processor shuts down when it receives a `Shutdown` control message or encounters a fatal error
//!
//! # Thread Safety
//!
//! Note that this trait uses `#[async_trait(?Send)]`, meaning implementations
//! are not required to be thread-safe. To ensure scalability, the dataflow engine will start
//! multiple instances of the same dataflow in parallel, each with its own processor instance.

use crate::NodeName;
use crate::error::Error;
use crate::message::Message;
use async_trait::async_trait;
use otap_df_channel::mpsc;
use std::rc::Rc;

/// A trait for processors in the dataflow pipeline.
#[async_trait(?Send)]
pub trait Processor {
    /// The type of messages handled by the processor.
    type Msg;

    /// Processes a message and optionally produces new messages.
    ///
    /// This method is called for each message that arrives at the processor. The processor can:
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
        msg: Message<Self::Msg>,
        effect_handler: &mut EffectHandler<Self::Msg>,
    ) -> Result<Option<Vec<Self::Msg>>, Error<Self::Msg>>;
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
    pub fn processor_name(&self) -> &str {
        &self.processor_name
    }

    /// Sends a message to the next node(s) in the dataflow.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ProcessorError`] if the message could not be sent.
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
    use async_trait::async_trait;
    use otap_df_channel::mpsc;
    use serde_json::Value;
    use std::cell::RefCell;
    use std::rc::Rc;
    use tokio::runtime::Builder;
    use tokio::task::LocalSet;

    /// A test message.
    #[derive(Debug, PartialEq)]
    struct TestMsg(String);

    struct TestProcessor {
        timer_tick_count: Rc<RefCell<usize>>,
        message_count: Rc<RefCell<usize>>,
        config_count: Rc<RefCell<usize>>,
        shutdown_count: Rc<RefCell<usize>>,
    }

    #[async_trait(?Send)]
    impl Processor for TestProcessor {
        type Msg = TestMsg;

        async fn process(
            &mut self,
            msg: Message<Self::Msg>,
            _effect_handler: &mut EffectHandler<Self::Msg>,
        ) -> Result<Option<Vec<Self::Msg>>, Error<Self::Msg>> {
            match msg {
                Message::Control { control } => match control {
                    TimerTick {} => {
                        *self.timer_tick_count.borrow_mut() += 1;
                        Ok(None)
                    }
                    Config { .. } => {
                        *self.config_count.borrow_mut() += 1;
                        Ok(None)
                    }
                    Shutdown { .. } => {
                        *self.shutdown_count.borrow_mut() += 1;
                        Ok(None)
                    }
                    _ => Ok(None),
                },
                Message::Data { data } => {
                    *self.message_count.borrow_mut() += 1;
                    // Append " RECEIVED" to the message content.
                    let processed_message = TestMsg(format!("{} RECEIVED", data.0));
                    Ok(Some(vec![processed_message]))
                }
            }
        }
    }

    #[test]
    fn test_processor() {
        let rt = Builder::new_current_thread().enable_all().build().unwrap();
        let local_tasks = LocalSet::new();

        // Create shared counters to keep track of events.
        let timer_tick_count = Rc::new(RefCell::new(0));
        let message_count = Rc::new(RefCell::new(0));
        let config_count = Rc::new(RefCell::new(0));
        let shutdown_count = Rc::new(RefCell::new(0));

        // Create the exporter instance.
        let mut processor = Box::new(TestProcessor {
            timer_tick_count: timer_tick_count.clone(),
            message_count: message_count.clone(),
            config_count: config_count.clone(),
            shutdown_count: shutdown_count.clone(),
        });

        // Create a channel for the effect handler
        let (tx, _rx) = mpsc::Channel::new(10);

        // Spawn the exporter's event loop.
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
        assert_eq!(*timer_tick_count.borrow(), 1, "TimerTick count mismatch");
        assert_eq!(*message_count.borrow(), 1, "Message count mismatch");
        assert_eq!(*config_count.borrow(), 1, "Config count mismatch");
        assert_eq!(*shutdown_count.borrow(), 1, "Shutdown count mismatch");
    }
}
