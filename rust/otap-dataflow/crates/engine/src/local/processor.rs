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
//! This implementation is designed to be used in a single-threaded environment.
//! The `Processor` trait does not require the `Send` bound, allowing for the use of non-thread-safe
//! types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own processor instance.

use crate::effect_handler::LocalEffectHandlerCore;
use crate::error::Error;
use crate::message::{ControlMsg, Message, Sender};
use async_trait::async_trait;
use std::borrow::Cow;

/// A trait for processors in the pipeline (!Send definition).
#[async_trait(?Send)]
pub trait Processor<PData> {
    /// Processes a message and optionally produces effects, such as generating new pdata messages.
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
        effect_handler: &mut EffectHandler<PData>,
    ) -> Result<(), Error<PData>>;
}

/// A `!Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: LocalEffectHandlerCore,

    /// A sender used to forward messages from the processor.
    msg_sender: Sender<PData>,
}

/// Implementation for the `!Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given processor name.
    #[must_use]
    pub fn new(name: Cow<'static, str>, msg_sender: Sender<PData>) -> Self {
        EffectHandler {
            core: LocalEffectHandlerCore {
                node_name: name,
                control_sender: None,
            },
            msg_sender,
        }
    }

    /// Creates a new local (!Send) `EffectHandler` with the given processor name and control sender.
    #[must_use]
    pub fn with_control_sender(
        name: Cow<'static, str>,
        msg_sender: Sender<PData>,
        control_sender: Sender<ControlMsg>,
    ) -> Self {
        EffectHandler {
            core: LocalEffectHandlerCore {
                node_name: name,
                control_sender: Some(control_sender),
            },
            msg_sender,
        }
    }

    /// Returns the name of the processor associated with this handler.
    #[must_use]
    pub fn processor_name(&self) -> Cow<'static, str> {
        self.core.node_name()
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    pub async fn send_message(&self, data: PData) -> Result<(), Error<PData>> {
        self.msg_sender.send(data).await.map_err(Error::from)
    }

    /// Sends an ACK control message upstream to indicate successful processing.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the control message could not be sent.
    pub async fn send_ack(&self, id: u64) -> Result<(), Error<PData>> {
        self.core.send_ack(id).await
    }

    /// Sends a NACK control message upstream to indicate failed processing.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the control message could not be sent.
    pub async fn send_nack(&self, id: u64, reason: &str) -> Result<(), Error<PData>> {
        self.core.send_nack(id, reason).await
    }

    // More methods will be added in the future as needed.
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::message::Sender;
    use otap_df_channel::mpsc;

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        id: u64,
        payload: String,
    }

    #[tokio::test]
    async fn test_effect_handler_creation() {
        let (sender, _receiver) = mpsc::Channel::<TestData>::new(100);
        let handler = EffectHandler::new("test_processor".into(), Sender::Local(sender));

        assert_eq!(handler.processor_name(), "test_processor");
    }

    #[tokio::test]
    async fn test_effect_handler_send_message() {
        let (sender, receiver) = mpsc::Channel::<TestData>::new(100);
        let handler = EffectHandler::new("test_processor".into(), Sender::Local(sender));

        let test_data = TestData {
            id: 123,
            payload: "test message".to_string(),
        };

        let result = handler.send_message(test_data.clone()).await;
        assert!(result.is_ok());

        let received_msg = receiver.recv().await.unwrap();
        assert_eq!(received_msg, test_data);
    }

    #[tokio::test]
    async fn test_effect_handler_send_multiple_messages() {
        let (sender, receiver) = mpsc::Channel::<TestData>::new(100);
        let handler = EffectHandler::new("test_processor".into(), Sender::Local(sender));

        let test_data1 = TestData {
            id: 1,
            payload: "message 1".to_string(),
        };
        let test_data2 = TestData {
            id: 2,
            payload: "message 2".to_string(),
        };

        let result1 = handler.send_message(test_data1.clone()).await;
        let result2 = handler.send_message(test_data2.clone()).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let received_msg1 = receiver.recv().await.unwrap();
        let received_msg2 = receiver.recv().await.unwrap();

        assert_eq!(received_msg1, test_data1);
        assert_eq!(received_msg2, test_data2);
    }

    #[tokio::test]
    async fn test_effect_handler_processor_name() {
        let (sender, _receiver) = mpsc::Channel::<TestData>::new(100);
        let handler = EffectHandler::new("my_custom_processor".into(), Sender::Local(sender));

        assert_eq!(handler.processor_name(), "my_custom_processor");
    }
}
