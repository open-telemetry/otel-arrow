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

use crate::effect_handler::{EffectHandlerCore, TimerCancelHandle};
use crate::error::Error;
use crate::local::message::LocalSender;
use crate::message::Message;
use crate::node::NodeId;
use async_trait::async_trait;
use otap_df_config::PortName;
use std::collections::HashMap;
use std::time::Duration;

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
    pub(crate) core: EffectHandlerCore,

    /// A sender used to forward messages from the processor.
    /// Supports multiple named output ports.
    msg_senders: HashMap<PortName, LocalSender<PData>>,
    /// Cached default sender for fast access in the hot path
    default_sender: Option<LocalSender<PData>>,
}

/// Implementation for the `!Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given processor name.
    #[must_use]
    pub fn new(
        node_id: NodeId,
        msg_senders: HashMap<PortName, LocalSender<PData>>,
        default_port: Option<PortName>,
    ) -> Self {
        let core = EffectHandlerCore::new(node_id);

        // Determine and cache the default sender
        let default_sender = if let Some(ref port) = default_port {
            msg_senders.get(port).cloned()
        } else if msg_senders.len() == 1 {
            msg_senders.values().next().cloned()
        } else {
            None
        };

        EffectHandler {
            core,
            msg_senders,
            default_sender,
        }
    }

    /// Returns the id of the processor associated with this handler.
    #[must_use]
    pub fn processor_id(&self) -> NodeId {
        self.core.node_id()
    }

    /// Returns the list of connected out ports for this processor.
    #[must_use]
    pub fn connected_ports(&self) -> Vec<PortName> {
        self.msg_senders.keys().cloned().collect()
    }

    /// Sends a message to the next node(s) in the pipeline using the default port.
    ///
    /// If a default port is configured (either explicitly or deduced when a single port is
    /// connected), it will be used. Otherwise, an error is returned.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent or [`Error::ProcessorError`]
    /// if the default port is not configured.
    #[inline]
    pub async fn send_message(&self, data: PData) -> Result<(), Error<PData>> {
        match &self.default_sender {
            Some(sender) => sender.send(data).await.map_err(Error::ChannelSendError),
            None => Err(Error::ProcessorError {
                processor: self.processor_id(),
                error:
                    "Ambiguous default out port: multiple ports connected and no default configured"
                        .to_string(),
            }),
        }
    }

    /// Sends a message to a specific named out port.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent, or
    /// [`Error::ProcessorError`] if the port does not exist.
    #[inline]
    pub async fn send_message_to<P>(&self, port: P, data: PData) -> Result<(), Error<PData>>
    where
        P: Into<PortName>,
    {
        let port_name: PortName = port.into();
        match self.msg_senders.get(&port_name) {
            Some(sender) => sender.send(data).await.map_err(Error::ChannelSendError),
            None => Err(Error::ProcessorError {
                processor: self.processor_id(),
                error: format!(
                    "Unknown out port '{port_name}' for node {}",
                    self.processor_id()
                ),
            }),
        }
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
    /// Current limitation: Only one timer can be started by an exporter at a time.
    pub async fn start_periodic_timer(
        &self,
        duration: Duration,
    ) -> Result<TimerCancelHandle, Error<PData>> {
        self.core.start_periodic_timer(duration).await
    }

    // More methods will be added in the future as needed.
}

#[cfg(test)]
mod tests {
    #![allow(missing_docs)]
    use super::*;
    use crate::local::message::LocalSender;
    use crate::testing::test_node;
    use otap_df_channel::mpsc;
    use std::borrow::Cow;
    use std::collections::{HashMap, HashSet};
    use tokio::time::{Duration, timeout};

    fn channel<T>(capacity: usize) -> (mpsc::Sender<T>, mpsc::Receiver<T>) {
        mpsc::Channel::new(capacity)
    }

    #[tokio::test]
    async fn effect_handler_send_message_to_named_port() {
        let (a_tx, a_rx) = channel::<u64>(10);
        let (b_tx, b_rx) = channel::<u64>(10);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::MpscSender(a_tx));
        let _ = senders.insert("b".into(), LocalSender::MpscSender(b_tx));

        let eh = EffectHandler::new(test_node("proc"), senders, None);
        eh.send_message_to("b", 42).await.unwrap();

        // Ensure only 'b' received
        assert!(
            timeout(Duration::from_millis(50), a_rx.recv())
                .await
                .is_err()
        );
        assert_eq!(b_rx.recv().await.unwrap(), 42);
    }

    #[tokio::test]
    async fn effect_handler_send_message_single_port_fallback() {
        let (tx, rx) = channel::<u64>(10);
        let mut senders = HashMap::new();
        let _ = senders.insert("only".into(), LocalSender::MpscSender(tx));

        let eh = EffectHandler::new(test_node("proc"), senders, None);

        eh.send_message(7).await.unwrap();
        assert_eq!(rx.recv().await.unwrap(), 7);
    }

    #[tokio::test]
    async fn effect_handler_send_message_uses_default_port() {
        let (a_tx, a_rx) = channel::<u64>(10);
        let (b_tx, b_rx) = channel::<u64>(10);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::MpscSender(a_tx));
        let _ = senders.insert("b".into(), LocalSender::MpscSender(b_tx));

        let eh = EffectHandler::new(test_node("proc"), senders, Some("a".into()));

        eh.send_message(11).await.unwrap();

        assert_eq!(a_rx.recv().await.unwrap(), 11);
        assert!(
            timeout(Duration::from_millis(50), b_rx.recv())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn effect_handler_send_message_ambiguous_without_default() {
        let (a_tx, a_rx) = channel::<u64>(10);
        let (b_tx, b_rx) = channel::<u64>(10);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::MpscSender(a_tx));
        let _ = senders.insert("b".into(), LocalSender::MpscSender(b_tx));

        let eh = EffectHandler::new(test_node("proc"), senders, None);

        let res = eh.send_message(5).await;
        assert!(res.is_err());

        // Nothing should be received on either port
        assert!(
            timeout(Duration::from_millis(50), a_rx.recv())
                .await
                .is_err()
        );
        assert!(
            timeout(Duration::from_millis(50), b_rx.recv())
                .await
                .is_err()
        );
    }

    #[tokio::test]
    async fn effect_handler_connected_ports_lists_all() {
        let (a_tx, _a_rx) = channel::<u64>(1);
        let (b_tx, _b_rx) = channel::<u64>(1);

        let mut senders = HashMap::new();
        let _ = senders.insert("a".into(), LocalSender::MpscSender(a_tx));
        let _ = senders.insert("b".into(), LocalSender::MpscSender(b_tx));

        let eh = EffectHandler::new(test_node("proc"), senders, None);

        let ports: HashSet<_> = eh.connected_ports().into_iter().collect();
        let expected: HashSet<_> = [Cow::from("a"), Cow::from("b")].into_iter().collect();
        assert_eq!(ports, expected);
    }
}
