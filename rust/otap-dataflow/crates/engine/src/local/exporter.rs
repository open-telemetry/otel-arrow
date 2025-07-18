// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement local exporters (!Send).
//!
//! An exporter is an egress node that sends data from a pipeline to external systems, performing
//! the necessary conversions from the internal pdata format to the format required by the external
//! system.
//!
//! Exporters can operate in various ways, including:
//!
//! 1. Sending telemetry data to remote endpoints via network protocols,
//! 2. Writing data to files or databases,
//! 3. Pushing data to message queues or event buses,
//! 4. Or any other method of exporting telemetry data to external systems.
//!
//! # Lifecycle
//!
//! 1. The exporter is instantiated and configured
//! 2. The `start` method is called, which begins the exporter's operation
//! 3. The exporter processes both internal control messages and pipeline data (pdata)
//! 4. The exporter shuts down when it receives a `Shutdown` control message or encounters a fatal
//!    error
//!
//! # Thread Safety
//!
//! This implementation is designed to be used in a single-threaded environment.
//! The `Exporter` trait does not require the `Send` bound, allowing for the use of non-thread-safe
//! types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own exporter instance.

use crate::effect_handler::LocalEffectHandlerCore;
use crate::error::Error;
use crate::message::{ControlMsg, MessageChannel, Sender};
use async_trait::async_trait;
use std::borrow::Cow;
use std::marker::PhantomData;
/// A trait for egress exporters (!Send definition).
#[async_trait( ? Send)]
pub trait Exporter<PData> {
    /// Starts the exporter and begins exporting incoming data.
    ///
    /// The pipeline engine will call this function to start the exporter in a separate task.
    /// Exporters are assigned their own dedicated task at pipeline initialization because their
    /// primary function involves interacting with the external world, and the pipeline has no
    /// prior knowledge of when these interactions will occur.
    ///
    /// The exporter is taken as `Box<Self>` so the method takes ownership of the exporter once `start` is called.
    /// This lets it move into an independent task, after which the pipeline can only
    /// reach it through the control-message channel.
    ///
    /// Because ownership is now exclusive, the code inside `start` can freely use
    /// `&mut self` to update internal state without worrying about aliasing or
    /// borrowing rules at the call-site. That keeps the public API simple (no
    /// exterior `&mut` references to juggle) while still allowing the exporter to
    /// mutate itself as much as it needs during its run loop.
    ///
    /// Exporters are expected to process both internal control messages and pipeline data messages,
    /// prioritizing control messages over data messages. This prioritization guarantee is ensured
    /// by the `MessageChannel` implementation.
    ///
    /// # Parameters
    ///
    /// - `msg_chan`: A channel to receive pdata or control messages. Control messages are
    ///   prioritized over pdata messages.
    /// - `effect_handler`: A handler to perform side effects such as network operations.
    ///
    /// # Errors
    ///
    /// Returns an [`Error`] if an unrecoverable error occurs.
    ///
    /// # Cancellation Safety
    ///
    /// This method should be cancellation safe and clean up any resources when dropped.
    async fn start(
        self: Box<Self>,
        msg_chan: MessageChannel<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<(), Error<PData>>;
}

/// A `!Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: LocalEffectHandlerCore,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    _pd: PhantomData<PData>,
}

/// Implementation for the `!Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given exporter name.
    #[must_use]
    pub fn new(name: Cow<'static, str>) -> Self {
        EffectHandler {
            core: LocalEffectHandlerCore {
                node_name: name,
                control_sender: None,
            },
            _pd: PhantomData,
        }
    }

    /// Creates a new local (!Send) `EffectHandler` with the given exporter name and control sender.
    #[must_use]
    pub fn with_control_sender(
        name: Cow<'static, str>,
        control_sender: Sender<ControlMsg>,
    ) -> Self {
        EffectHandler {
            core: LocalEffectHandlerCore {
                node_name: name,
                control_sender: Some(control_sender),
            },
            _pd: PhantomData,
        }
    }

    /// Returns the name of the exporter associated with this handler.
    #[must_use]
    pub fn exporter_name(&self) -> Cow<'static, str> {
        self.core.node_name()
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
    use crate::message::{ControlMsg, Sender};
    use otap_df_channel::mpsc;

    #[derive(Debug, Clone, PartialEq)]
    struct TestData {
        id: u64,
        payload: String,
    }

    #[tokio::test]
    async fn test_effect_handler_creation() {
        let handler = EffectHandler::<TestData>::new("test_exporter".into());

        assert_eq!(handler.exporter_name(), "test_exporter");
    }

    #[tokio::test]
    async fn test_effect_handler_with_control_sender() {
        let (sender, _receiver) = mpsc::Channel::new(100);
        let handler = EffectHandler::<TestData>::with_control_sender(
            "test_exporter".into(),
            Sender::Local(sender),
        );

        assert_eq!(handler.exporter_name(), "test_exporter");
    }

    #[tokio::test]
    async fn test_effect_handler_send_ack() {
        let (sender, receiver) = mpsc::Channel::new(100);
        let handler = EffectHandler::<TestData>::with_control_sender(
            "test_exporter".into(),
            Sender::Local(sender),
        );

        let result = handler.send_ack(123).await;
        assert!(result.is_ok());

        let received_msg = receiver.recv().await.unwrap();
        assert_eq!(received_msg, ControlMsg::Ack { id: 123 });
    }

    #[tokio::test]
    async fn test_effect_handler_send_nack() {
        let (sender, receiver) = mpsc::Channel::new(100);
        let handler = EffectHandler::<TestData>::with_control_sender(
            "test_exporter".into(),
            Sender::Local(sender),
        );

        let result = handler.send_nack(456, "Test error").await;
        assert!(result.is_ok());

        let received_msg = receiver.recv().await.unwrap();
        assert_eq!(
            received_msg,
            ControlMsg::Nack {
                id: 456,
                reason: "Test error".to_string(),
            }
        );
    }

    #[tokio::test]
    async fn test_effect_handler_send_ack_no_sender() {
        let handler = EffectHandler::<TestData>::new("test_exporter".into());

        let result = handler.send_ack(123).await;
        assert!(result.is_ok()); // Should succeed even without a control sender
    }

    #[tokio::test]
    async fn test_effect_handler_send_nack_no_sender() {
        let handler = EffectHandler::<TestData>::new("test_exporter".into());

        let result = handler.send_nack(456, "Test error").await;
        assert!(result.is_ok()); // Should succeed even without a control sender
    }

    #[tokio::test]
    async fn test_effect_handler_exporter_name() {
        let handler = EffectHandler::<TestData>::new("my_custom_exporter".into());

        assert_eq!(handler.exporter_name(), "my_custom_exporter");
    }

    #[tokio::test]
    async fn test_effect_handler_send_multiple_acks() {
        let (sender, receiver) = mpsc::Channel::new(100);
        let handler = EffectHandler::<TestData>::with_control_sender(
            "test_exporter".into(),
            Sender::Local(sender),
        );

        let result1 = handler.send_ack(111).await;
        let result2 = handler.send_ack(222).await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let received_msg1 = receiver.recv().await.unwrap();
        let received_msg2 = receiver.recv().await.unwrap();

        assert_eq!(received_msg1, ControlMsg::Ack { id: 111 });
        assert_eq!(received_msg2, ControlMsg::Ack { id: 222 });
    }

    #[tokio::test]
    async fn test_effect_handler_send_mixed_ack_nack() {
        let (sender, receiver) = mpsc::Channel::new(100);
        let handler = EffectHandler::<TestData>::with_control_sender(
            "test_exporter".into(),
            Sender::Local(sender),
        );

        let result1 = handler.send_ack(333).await;
        let result2 = handler.send_nack(444, "Mixed test error").await;

        assert!(result1.is_ok());
        assert!(result2.is_ok());

        let received_msg1 = receiver.recv().await.unwrap();
        let received_msg2 = receiver.recv().await.unwrap();

        assert_eq!(received_msg1, ControlMsg::Ack { id: 333 });
        assert_eq!(
            received_msg2,
            ControlMsg::Nack {
                id: 444,
                reason: "Mixed test error".to_string(),
            }
        );
    }
}
