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
//! This implementation is designed for use in both single-threaded and multi-threaded environments.
//! The `Processor` trait requires the `Send` bound, enabling the use of thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own processor instance.

use crate::effect_handler::{EffectHandlerCore, TimerCancelHandle};
use crate::error::Error;
use crate::message::Message;
use crate::shared::message::SharedSender;
use async_trait::async_trait;
use otap_df_channel::error::SendError;
use otap_df_config::{NodeId, PortName};
use std::time::Duration;
use std::collections::HashMap;

/// A trait for processors in the pipeline (Send definition).
#[async_trait]
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

/// A `Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    pub(crate) core: EffectHandlerCore,

    /// A sender used to forward messages from the processor.
    /// Supports multiple named output ports.
    msg_senders: HashMap<PortName, SharedSender<PData>>,
    /// Optional default port to use when calling send_message.
    default_port: Option<PortName>,
}

/// Implementation for the `Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` with the given processor name and pdata sender.
    #[must_use]
    pub fn new(
        node_id: NodeId,
        msg_senders: HashMap<PortName, SharedSender<PData>>,
        default_port: Option<PortName>,
    ) -> Self {
        let core = EffectHandlerCore::new(node_id);
        EffectHandler { core, msg_senders, default_port }
    }

    /// Returns the id of the processor associated with this handler.
    #[must_use]
    pub fn processor_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Returns the list of connected out ports for this processor.
    pub fn connected_ports(&self) -> Vec<PortName> {
        self.msg_senders.keys().cloned().collect()
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    pub async fn send_message(&self, data: PData) -> Result<(), SendError<PData>> {
        let port = if let Some(p) = &self.default_port {
            p.clone()
        } else if self.msg_senders.len() == 1 {
            self.msg_senders.keys().next().cloned().expect("non-empty")
        } else {
            return Err(SendError::Closed(data));
        };
        self.send_message_to(port, data).await
    }

    /// Sends a message to a specific named out port.
    pub async fn send_message_to<P>(&self, port: P, data: PData) -> Result<(), SendError<PData>>
    where
        P: Into<PortName>,
    {
        let port_name: PortName = port.into();
        let sender = match self.msg_senders.get(&port_name).cloned() {
            Some(s) => s,
            None => return Err(SendError::Closed(data)),
        };
        sender.send(data).await
    }

    /// Print an info message to stdout.
    ///
    /// This method provides a standardized way for processors to output
    /// informational messages without blocking the async runtime.
    pub async fn info(&self, message: &str) {
        self.core.info(message).await;
    }

    /// Starts a cancellable periodic timer that emits TimerTick on the control channel.
    /// Returns a handle that can be used to cancel the timer.
    ///
    /// Current limitation: Only one timer can be started by a processor at a time.
    pub async fn start_periodic_timer(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle, Error<PData>> {
        self.core.start_periodic_timer(duration).await
    }

    // More methods will be added in the future as needed.
}
