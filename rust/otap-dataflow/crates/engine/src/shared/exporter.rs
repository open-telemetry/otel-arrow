// SPDX-License-Identifier: Apache-2.0

//! Trait and structures used to implement shared exporters (Send bound).
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
//! This implementation is designed for use in both single-threaded and multi-threaded environments.  
//! The `Exporter` trait requires the `Send` bound, enabling the use of thread-safe types.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline
//! in parallel on different cores, each with its own exporter instance.

use crate::effect_handler::SharedEffectHandlerCore;
use crate::error::Error;
use crate::message::{ControlMsg, Message};
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use std::borrow::Cow;
use std::marker::PhantomData;
use std::pin::Pin;
use std::time::Duration;
use tokio::time::{Instant, Sleep, sleep_until};

/// A trait for egress exporters (Send definition).
#[async_trait]
pub trait Exporter<PData> {
    /// Similar to local::exporter::Exporter::start, but operates in a Send context.
    async fn start(
        self: Box<Self>,
        msg_chan: MessageChannel<PData>,
        effect_handler: EffectHandler<PData>,
    ) -> Result<(), Error<PData>>;
}

/// A channel for receiving control and pdata messages.
///
/// Control messages are prioritized until the first `Shutdown` is received.
/// After that, only pdata messages are considered, up to the deadline.
///
/// Note: This approach is used to implement a graceful shutdown. The engine will first close all
/// data sources in the pipeline, and then send a shutdown message with a deadline to all nodes in
/// the pipeline.
pub struct MessageChannel<PData> {
    control_rx: Option<tokio::sync::mpsc::Receiver<ControlMsg>>,
    pdata_rx: Option<tokio::sync::mpsc::Receiver<PData>>,
    /// Once a Shutdown is seen, this is set to `Some(instant)` at which point
    /// no more pdata will be accepted.
    shutting_down_deadline: Option<Instant>,
    /// Holds the ControlMsg::Shutdown until after we’ve drained pdata.
    pending_shutdown: Option<ControlMsg>,
}

impl<PData> MessageChannel<PData> {
    /// Creates a new `MessageChannel` with the given control and data receivers.
    #[must_use]
    pub fn new(
        control_rx: tokio::sync::mpsc::Receiver<ControlMsg>,
        pdata_rx: tokio::sync::mpsc::Receiver<PData>,
    ) -> Self {
        MessageChannel {
            control_rx: Some(control_rx),
            pdata_rx: Some(pdata_rx),
            shutting_down_deadline: None,
            pending_shutdown: None,
        }
    }

    /// Asynchronously receives the next message to process.
    ///
    /// Order of precedence:
    ///
    /// 1. Before a `Shutdown` is seen: control messages are always
    ///    returned ahead of pdata.
    /// 2. After the first `Shutdown` is received:
    ///    - All further control messages are silently discarded.
    ///    - Pending pdata are drained until the shutdown deadline.
    /// 3. When the deadline expires (or was `0`): the stored `Shutdown` is returned.
    ///    Subsequent calls return `RecvError::Closed`.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if both channels are closed, or if the
    /// shutdown deadline has passed.
    pub async fn recv(&mut self) -> Result<Message<PData>, RecvError> {
        let mut sleep_until_deadline: Option<Pin<Box<Sleep>>> = None;

        loop {
            if self.control_rx.is_none() || self.pdata_rx.is_none() {
                // MessageChannel has been shutdown
                return Err(RecvError::Closed);
            }

            // Draining mode: Shutdown pending
            if let Some(dl) = self.shutting_down_deadline {
                // If the deadline has passed, emit the pending Shutdown now.
                if Instant::now() >= dl {
                    let shutdown = self
                        .pending_shutdown
                        .take()
                        .expect("pending_shutdown must exist");
                    self.shutdown();
                    return Ok(Message::Control(shutdown));
                }

                if sleep_until_deadline.is_none() {
                    // Create a sleep timer for the deadline
                    sleep_until_deadline = Some(Box::pin(sleep_until(dl)));
                }

                // Drain pdata first, then timer, then other control msgs
                tokio::select! {
                    biased;

                    // 1) Any pdata?
                    pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv() => match pdata {
                        Some(pdata) => return Ok(Message::PData(pdata)),
                        None => {
                            // pdata channel closed → emit Shutdown
                            let shutdown = self.pending_shutdown
                                .take()
                                .expect("pending_shutdown must exist");
                            self.shutdown();
                            return Ok(Message::Control(shutdown));
                        }
                    },

                    // 2) Deadline hit?
                    _ = sleep_until_deadline.as_mut().expect("sleep_until_deadline must exist") => {
                        let shutdown = self.pending_shutdown
                            .take()
                            .expect("pending_shutdown must exist");
                        self.shutdown();
                        return Ok(Message::Control(shutdown));
                    }
                }
            }

            // Normal mode: no shutdown yet
            tokio::select! {
                biased;

                // A) Control first
                ctrl = self.control_rx.as_mut().expect("control_rx must exist").recv() => match ctrl {
                    Some(ControlMsg::Shutdown { deadline, reason }) => {
                        if deadline.is_zero() {
                            // Immediate shutdown, no draining
                            self.shutdown();
                            return Ok(Message::Control(ControlMsg::Shutdown { deadline: Duration::ZERO, reason }));
                        }
                        // Begin draining mode, but don’t return Shutdown yet
                        let when = Instant::now() + deadline;
                        self.shutting_down_deadline = Some(when);
                        self.pending_shutdown = Some(ControlMsg::Shutdown { deadline: Duration::ZERO, reason });
                        continue; // re-enter the loop into draining mode
                    }
                    Some(msg) => return Ok(Message::Control(msg)),
                    None  => return Err(RecvError::Closed),
                },

                // B) Then pdata
                pdata = self.pdata_rx.as_mut().expect("pdata_rx must exist").recv() => {
                    match pdata {
                        Some(pdata) => {
                            return Ok(Message::PData(pdata));
                        }
                        None => {
                            return Err(RecvError::Closed);
                        }
                    }
                }
            }
        }
    }

    fn shutdown(&mut self) {
        self.shutting_down_deadline = None;
        drop(self.control_rx.take().expect("control_rx must exist"));
        drop(self.pdata_rx.take().expect("pdata_rx must exist"));
    }
}

/// A `Send` implementation of the EffectHandler.
#[derive(Clone)]
pub struct EffectHandler<PData> {
    core: SharedEffectHandlerCore,

    /// A 0 size type used to parameterize the `EffectHandler` with the type of message the exporter
    /// will consume.
    _pd: PhantomData<PData>,
}

/// Implementation for the `Send` effect handler.
impl<PData> EffectHandler<PData> {
    /// Creates a new shared (Send) `EffectHandler` with the given exporter name.
    #[must_use]
    pub fn new(name: Cow<'static, str>) -> Self {
        EffectHandler {
            core: SharedEffectHandlerCore {
                node_name: name,
                control_sender: None,
            },
            _pd: PhantomData,
        }
    }

    /// Creates a new shared (Send) `EffectHandler` with the given exporter name and control sender.
    #[must_use]
    pub fn with_control_sender(
        name: Cow<'static, str>,
        control_sender: tokio::sync::mpsc::Sender<ControlMsg>,
    ) -> Self {
        EffectHandler {
            core: SharedEffectHandlerCore {
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
    use std::time::Duration;
    use tokio::sync::mpsc;
    use tokio::time;

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
        let (sender, _receiver) = mpsc::channel(100);
        let handler =
            EffectHandler::<TestData>::with_control_sender("test_exporter".into(), sender);

        assert_eq!(handler.exporter_name(), "test_exporter");
    }

    #[tokio::test]
    async fn test_effect_handler_send_ack() {
        let (sender, mut receiver) = mpsc::channel(100);
        let handler =
            EffectHandler::<TestData>::with_control_sender("test_exporter".into(), sender);

        let result = handler.send_ack(123).await;
        assert!(result.is_ok());

        let received_msg = receiver.recv().await.unwrap();
        assert_eq!(received_msg, ControlMsg::Ack { id: 123 });
    }

    #[tokio::test]
    async fn test_effect_handler_send_nack() {
        let (sender, mut receiver) = mpsc::channel(100);
        let handler =
            EffectHandler::<TestData>::with_control_sender("test_exporter".into(), sender);

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
    async fn test_message_channel_creation() {
        let (control_tx, control_rx) = mpsc::channel::<ControlMsg>(100);
        let (data_tx, data_rx) = mpsc::channel::<String>(100);

        let channel = MessageChannel::new(control_rx, data_rx);

        // Drop senders to avoid unused warnings
        drop(control_tx);
        drop(data_tx);

        // Channel should be created successfully
        // We can't access private fields directly, so we just verify creation succeeded
        drop(channel);
    }

    #[tokio::test]
    async fn test_message_channel_recv_control_message() {
        let (control_tx, control_rx) = mpsc::channel::<ControlMsg>(100);
        let (data_tx, data_rx) = mpsc::channel::<String>(100);

        let mut channel = MessageChannel::new(control_rx, data_rx);

        // Send a control message
        control_tx.send(ControlMsg::Ack { id: 123 }).await.unwrap();

        // Receive the message
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::Control(ControlMsg::Ack { id }) => assert_eq!(id, 123),
            _ => panic!("Expected ACK control message"),
        }

        drop(control_tx);
        drop(data_tx);
    }

    #[tokio::test]
    async fn test_message_channel_recv_pdata_message() {
        let (control_tx, control_rx) = mpsc::channel(100);
        let (data_tx, data_rx) = mpsc::channel(100);

        let mut channel = MessageChannel::new(control_rx, data_rx);

        let test_data = TestData {
            id: 456,
            payload: "test data".to_string(),
        };

        // Send a pdata message
        data_tx.send(test_data.clone()).await.unwrap();

        // Receive the message
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::PData(data) => assert_eq!(data, test_data),
            _ => panic!("Expected PData message"),
        }

        drop(control_tx);
        drop(data_tx);
    }

    #[tokio::test]
    async fn test_message_channel_control_priority() {
        let (control_tx, control_rx) = mpsc::channel(100);
        let (data_tx, data_rx) = mpsc::channel(100);

        let mut channel = MessageChannel::new(control_rx, data_rx);

        let test_data = TestData {
            id: 789,
            payload: "test data".to_string(),
        };

        // Send both control and data messages
        data_tx.send(test_data.clone()).await.unwrap();
        control_tx.send(ControlMsg::Ack { id: 999 }).await.unwrap();

        // Control message should be received first
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::Control(ControlMsg::Ack { id }) => assert_eq!(id, 999),
            _ => panic!("Expected ACK control message first"),
        }

        // Then the data message
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::PData(data) => assert_eq!(data, test_data),
            _ => panic!("Expected PData message second"),
        }

        drop(control_tx);
        drop(data_tx);
    }

    #[tokio::test]
    async fn test_message_channel_immediate_shutdown() {
        let (control_tx, control_rx) = mpsc::channel::<ControlMsg>(100);
        let (data_tx, data_rx) = mpsc::channel::<String>(100);

        let mut channel = MessageChannel::new(control_rx, data_rx);

        // Send immediate shutdown (zero deadline)
        control_tx
            .send(ControlMsg::Shutdown {
                deadline: Duration::ZERO,
                reason: "Test shutdown".to_string(),
            })
            .await
            .unwrap();

        // Should receive shutdown immediately
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::Control(ControlMsg::Shutdown { deadline, reason }) => {
                assert_eq!(deadline, Duration::ZERO);
                assert_eq!(reason, "Test shutdown");
            }
            _ => panic!("Expected Shutdown control message"),
        }

        // Further calls should return closed
        let result = channel.recv().await;
        assert!(result.is_err());

        drop(control_tx);
        drop(data_tx);
    }

    #[tokio::test]
    async fn test_message_channel_graceful_shutdown() {
        let (control_tx, control_rx) = mpsc::channel(100);
        let (data_tx, data_rx) = mpsc::channel(100);

        let mut channel = MessageChannel::new(control_rx, data_rx);

        let test_data = TestData {
            id: 123,
            payload: "test data".to_string(),
        };

        // Send data message first
        data_tx.send(test_data.clone()).await.unwrap();

        // Send graceful shutdown with deadline
        control_tx
            .send(ControlMsg::Shutdown {
                deadline: Duration::from_millis(100),
                reason: "Graceful shutdown".to_string(),
            })
            .await
            .unwrap();

        // Should receive the data message first (draining mode)
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::PData(data) => assert_eq!(data, test_data),
            _ => panic!("Expected PData message first"),
        }

        // Close data channel to trigger shutdown
        drop(data_tx);

        // Should receive shutdown after data is drained
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::Control(ControlMsg::Shutdown { deadline, reason }) => {
                assert_eq!(deadline, Duration::ZERO);
                assert_eq!(reason, "Graceful shutdown");
            }
            _ => panic!("Expected Shutdown control message"),
        }

        drop(control_tx);
    }

    #[tokio::test]
    async fn test_message_channel_shutdown_deadline_expiry() {
        let (control_tx, control_rx) = mpsc::channel::<ControlMsg>(100);
        let (data_tx, data_rx) = mpsc::channel::<String>(100);

        let mut channel = MessageChannel::new(control_rx, data_rx);

        // Send graceful shutdown with very short deadline
        control_tx
            .send(ControlMsg::Shutdown {
                deadline: Duration::from_millis(1),
                reason: "Quick shutdown".to_string(),
            })
            .await
            .unwrap();

        // Wait for deadline to expire
        time::sleep(Duration::from_millis(5)).await;

        // Should receive shutdown after deadline expires
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::Control(ControlMsg::Shutdown { deadline, reason }) => {
                assert_eq!(deadline, Duration::ZERO);
                assert_eq!(reason, "Quick shutdown");
            }
            _ => panic!("Expected Shutdown control message"),
        }

        drop(control_tx);
        drop(data_tx);
    }

    #[tokio::test]
    async fn test_message_channel_closed_channels() {
        let (control_tx, control_rx) = mpsc::channel::<ControlMsg>(100);
        let (data_tx, data_rx) = mpsc::channel::<String>(100);

        let mut channel = MessageChannel::new(control_rx, data_rx);

        // Close both channels
        drop(control_tx);
        drop(data_tx);

        // Should receive closed error
        let result = channel.recv().await;
        assert!(result.is_err());

        match result.unwrap_err() {
            RecvError::Closed => (),
            _ => panic!("Expected closed error"),
        }
    }

    #[tokio::test]
    async fn test_message_channel_shutdown_discards_further_control() {
        let (control_tx, control_rx) = mpsc::channel::<ControlMsg>(100);
        let (data_tx, data_rx) = mpsc::channel::<String>(100);

        let mut channel = MessageChannel::new(control_rx, data_rx);

        // Send shutdown
        control_tx
            .send(ControlMsg::Shutdown {
                deadline: Duration::from_millis(50),
                reason: "First shutdown".to_string(),
            })
            .await
            .unwrap();

        // Send another control message that should be discarded
        control_tx.send(ControlMsg::Ack { id: 123 }).await.unwrap();

        // Close data channel to trigger shutdown
        drop(data_tx);

        // Should receive only the shutdown, not the ACK
        let result = channel.recv().await;
        assert!(result.is_ok());

        match result.unwrap() {
            Message::Control(ControlMsg::Shutdown { reason, .. }) => {
                assert_eq!(reason, "First shutdown");
            }
            _ => panic!("Expected Shutdown control message"),
        }

        drop(control_tx);
    }
}
