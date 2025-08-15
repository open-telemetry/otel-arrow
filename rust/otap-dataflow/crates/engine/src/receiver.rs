// SPDX-License-Identifier: Apache-2.0

//! Receiver wrapper used to provide a unified interface to the pipeline engine that abstracts over
//! the fact that receiver implementations may be `!Send` or `Send`.
//!
//! For more details on the `!Send` implementation of a receiver, see [`local::Receiver`].
//! See [`shared::Receiver`] for the Send implementation.

use crate::config::ReceiverConfig;
use crate::control::{Controllable, NodeControlMsg, PipelineCtrlMsgSender};
use crate::error::Error;
use crate::local::message::{LocalReceiver, LocalSender};
use crate::local::receiver as local;
use crate::message::{Receiver, Sender};
use crate::node::{Node, NodeId, NodeWithPDataSender};
use crate::shared::message::{SharedReceiver, SharedSender};
use crate::shared::receiver as shared;
use otap_df_channel::error::SendError;
use otap_df_channel::mpsc;
use otap_df_config::PortName;
use otap_df_config::node::NodeUserConfig;
use std::collections::HashMap;
use std::sync::Arc;

/// A wrapper for the receiver that allows for both `Send` and `!Send` receivers.
///
/// Note: This is useful for creating a single interface for the receiver regardless of their
/// 'sendability'.
pub enum ReceiverWrapper<PData> {
    /// A receiver with a `!Send` implementation.
    Local {
        /// Index node identifier.
        node_id: NodeId,
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the node.
        runtime_config: ReceiverConfig,
        /// The receiver instance.
        receiver: Box<dyn local::Receiver<PData>>,
        /// A sender for control messages.
        control_sender: LocalSender<NodeControlMsg>,
        /// A receiver for control messages.
        control_receiver: LocalReceiver<NodeControlMsg>,
        /// Senders for PData messages per out port.
        pdata_senders: HashMap<PortName, LocalSender<PData>>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<LocalReceiver<PData>>,
    },
    /// A receiver with a `Send` implementation.
    Shared {
        /// Index node identifier.
        node_id: NodeId,
        /// The user configuration for the node, including its name and channel settings.
        user_config: Arc<NodeUserConfig>,
        /// The runtime configuration for the node.
        runtime_config: ReceiverConfig,
        /// The receiver instance.
        receiver: Box<dyn shared::Receiver<PData>>,
        /// A sender for control messages.
        control_sender: SharedSender<NodeControlMsg>,
        /// A receiver for control messages.
        control_receiver: SharedReceiver<NodeControlMsg>,
        /// Senders for PData messages per out port.
        pdata_senders: HashMap<PortName, SharedSender<PData>>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<SharedReceiver<PData>>,
    },
}

#[async_trait::async_trait(?Send)]
impl<PData> Controllable for ReceiverWrapper<PData> {
    /// Returns the control message sender for the receiver.
    fn control_sender(&self) -> Sender<NodeControlMsg> {
        match self {
            ReceiverWrapper::Local { control_sender, .. } => Sender::Local(control_sender.clone()),
            ReceiverWrapper::Shared { control_sender, .. } => {
                Sender::Shared(control_sender.clone())
            }
        }
    }
}

impl<PData> ReceiverWrapper<PData> {
    /// Creates a new `ReceiverWrapper` with the given receiver and configuration.
    pub fn local<R>(
        receiver: R,
        node_id: NodeId,
        user_config: Arc<NodeUserConfig>,
        config: &ReceiverConfig,
    ) -> Self
    where
        R: local::Receiver<PData> + 'static,
    {
        let (control_sender, control_receiver) =
            mpsc::Channel::new(config.control_channel.capacity);

        ReceiverWrapper::Local {
            node_id,
            user_config,
            runtime_config: config.clone(),
            receiver: Box::new(receiver),
            control_sender: LocalSender::MpscSender(control_sender),
            control_receiver: LocalReceiver::MpscReceiver(control_receiver),
            pdata_senders: HashMap::new(),
            pdata_receiver: None,
        }
    }

    /// Creates a new `ReceiverWrapper` with the given receiver and configuration.
    pub fn shared<R>(
        receiver: R,
        node_id: NodeId,
        user_config: Arc<NodeUserConfig>,
        config: &ReceiverConfig,
    ) -> Self
    where
        R: shared::Receiver<PData> + 'static,
    {
        let (control_sender, control_receiver) =
            tokio::sync::mpsc::channel(config.control_channel.capacity);

        ReceiverWrapper::Shared {
            node_id,
            user_config,
            runtime_config: config.clone(),
            receiver: Box::new(receiver),
            control_sender: SharedSender::MpscSender(control_sender),
            control_receiver: SharedReceiver::MpscReceiver(control_receiver),
            pdata_senders: HashMap::new(),
            pdata_receiver: None,
        }
    }

    /// Starts the receiver and begins receiver incoming data.
    pub async fn start(
        self,
        pipeline_ctrl_msg_tx: PipelineCtrlMsgSender,
    ) -> Result<(), Error<PData>> {
        match self {
            ReceiverWrapper::Local {
                node_id,
                receiver,
                control_receiver,
                pdata_senders,
                user_config,
                ..
            } => {
                let msg_senders = if pdata_senders.is_empty() {
                    return Err(Error::ReceiverError {
                        receiver: node_id.clone(),
                        error: "The pdata sender must be defined at this stage".to_owned(),
                    });
                } else {
                    pdata_senders
                };
                let default_port = user_config.default_out_port.clone();
                let ctrl_msg_chan = local::ControlChannel::new(Receiver::Local(control_receiver));
                let effect_handler = local::EffectHandler::new(
                    node_id,
                    msg_senders,
                    default_port,
                    pipeline_ctrl_msg_tx,
                );
                receiver.start(ctrl_msg_chan, effect_handler).await
            }
            ReceiverWrapper::Shared {
                node_id,
                receiver,
                control_receiver,
                pdata_senders,
                user_config,
                ..
            } => {
                let msg_senders = if pdata_senders.is_empty() {
                    return Err(Error::ReceiverError {
                        receiver: node_id.clone(),
                        error: "The pdata sender must be defined at this stage".to_owned(),
                    });
                } else {
                    pdata_senders
                };
                let default_port = user_config.default_out_port.clone();
                let ctrl_msg_chan = shared::ControlChannel::new(control_receiver);
                let effect_handler = shared::EffectHandler::new(
                    node_id,
                    msg_senders,
                    default_port,
                    pipeline_ctrl_msg_tx,
                );
                receiver.start(ctrl_msg_chan, effect_handler).await
            }
        }
    }

    /// Returns the PData receiver.
    pub fn take_pdata_receiver(&mut self) -> Receiver<PData> {
        match self {
            ReceiverWrapper::Local { pdata_receiver, .. } => {
                Receiver::Local(pdata_receiver.take().expect("pdata_receiver is None"))
            }
            ReceiverWrapper::Shared { pdata_receiver, .. } => {
                Receiver::Shared(pdata_receiver.take().expect("pdata_receiver is None"))
            }
        }
    }
}

#[async_trait::async_trait(?Send)]
impl<PData> Node for ReceiverWrapper<PData> {
    fn is_shared(&self) -> bool {
        match self {
            ReceiverWrapper::Local { .. } => false,
            ReceiverWrapper::Shared { .. } => true,
        }
    }

    fn node_id(&self) -> NodeId {
        match self {
            ReceiverWrapper::Local { node_id, .. } => node_id.clone(),
            ReceiverWrapper::Shared { node_id, .. } => node_id.clone(),
        }
    }

    fn user_config(&self) -> Arc<NodeUserConfig> {
        match self {
            ReceiverWrapper::Local {
                user_config: config,
                ..
            } => config.clone(),
            ReceiverWrapper::Shared {
                user_config: config,
                ..
            } => config.clone(),
        }
    }

    /// Sends a control message to the node.
    async fn send_control_msg(&self, msg: NodeControlMsg) -> Result<(), SendError<NodeControlMsg>> {
        match self {
            ReceiverWrapper::Local { control_sender, .. } => control_sender.send(msg).await,
            ReceiverWrapper::Shared { control_sender, .. } => control_sender.send(msg).await,
        }
    }
}

impl<PData> NodeWithPDataSender<PData> for ReceiverWrapper<PData> {
    fn set_pdata_sender(
        &mut self,
        node_id: NodeId,
        port: PortName,
        sender: Sender<PData>,
    ) -> Result<(), Error<PData>> {
        match (self, sender) {
            (ReceiverWrapper::Local { pdata_senders, .. }, Sender::Local(sender)) => {
                let _ = pdata_senders.insert(port, sender);
                Ok(())
            }
            (ReceiverWrapper::Shared { pdata_senders, .. }, Sender::Shared(sender)) => {
                let _ = pdata_senders.insert(port, sender);
                Ok(())
            }
            (ReceiverWrapper::Local { .. }, _) => Err(Error::ProcessorError {
                processor: node_id,
                error: "Expected a local sender for PData".to_owned(),
            }),
            (ReceiverWrapper::Shared { .. }, _) => Err(Error::ProcessorError {
                processor: node_id,
                error: "Expected a shared sender for PData".to_owned(),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ReceiverWrapper;
    use crate::local::receiver as local;
    use crate::receiver::Error;
    use crate::shared::receiver as shared;
    use crate::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use crate::testing::{CtrlMsgCounters, TestMsg, test_node};
    use async_trait::async_trait;
    use otap_df_config::node::NodeUserConfig;
    use serde_json::Value;
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, sleep, timeout};

    /// A test receiver that counts message events.
    /// Works with any type of receiver !Send or Send.
    pub struct TestReceiver {
        /// Counter for different message types
        ctrl_msg_counters: CtrlMsgCounters,
        port_notifier: oneshot::Sender<SocketAddr>,
    }

    impl TestReceiver {
        /// Creates a new test node
        pub fn new(
            ctrl_msg_counters: CtrlMsgCounters,
            port_notifier: oneshot::Sender<SocketAddr>,
        ) -> Self {
            TestReceiver {
                ctrl_msg_counters,
                port_notifier,
            }
        }
    }

    #[async_trait(?Send)]
    impl local::Receiver<TestMsg> for TestReceiver {
        async fn start(
            self: Box<Self>,
            mut ctrl_msg_recv: local::ControlChannel,
            effect_handler: local::EffectHandler<TestMsg>,
        ) -> Result<(), Error<TestMsg>> {
            // Bind to an ephemeral port.
            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let listener = effect_handler.tcp_listener(addr)?;
            let local_addr = listener.local_addr().unwrap();

            // Notify the test of the actual bound address.
            let _ = self.port_notifier.send(local_addr);

            loop {
                tokio::select! {
                    // Process incoming control messages.
                    ctrl_msg = ctrl_msg_recv.recv() => {
                        let ctrl_msg = ctrl_msg?;
                        self.ctrl_msg_counters.update_with(&ctrl_msg);
                        if ctrl_msg.is_shutdown() {
                            break;
                        }
                    }

                    // Process incoming TCP connections.
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((mut socket, peer_addr)) => {
                                // Clone the effect handler so the spawned task can send messages.
                                let effect_handler = effect_handler.clone();
                                // Spawn a task to handle the connection.
                                // ToDo should be abstract that and expose a method in the effect handler?
                                _ = tokio::task::spawn_local(async move {
                                    let mut buf = [0u8; 1024];
                                    loop {
                                        match socket.read(&mut buf).await {
                                            Ok(0) => {
                                                break;
                                            },
                                            Ok(n) => {
                                                let received = String::from_utf8_lossy(&buf[..n]).to_string();
                                                // Create a TestMsg from the received data and send it.
                                                let msg = TestMsg(received);
                                                if let Err(e) = effect_handler.send_message(msg).await {
                                                    panic!("Error sending message via effect handler: {e}");
                                                }
                                                // Echo back an acknowledgment.
                                                let _ = socket.write_all(b"ack").await;
                                            },
                                            Err(e) => {
                                                panic!("Error reading from {peer_addr}: {e}");
                                            }
                                        }
                                    }
                                });
                            },
                            Err(e) => {
                                panic!("Error accepting connection: {e}");
                            }
                        }
                    }
                    // A timeout branch in case no events occur.
                    () = sleep(Duration::from_secs(1)) => {
                        // You could do periodic tasks here.
                    }
                }

                // For this test, exit the loop after 5 timer ticks.
                if self.ctrl_msg_counters.get_timer_tick_count() >= 5 {
                    break;
                }
            }

            Ok(())
        }
    }

    #[async_trait]
    impl shared::Receiver<TestMsg> for TestReceiver {
        async fn start(
            self: Box<Self>,
            mut ctrl_msg_recv: shared::ControlChannel,
            effect_handler: shared::EffectHandler<TestMsg>,
        ) -> Result<(), Error<TestMsg>> {
            // Bind to an ephemeral port.
            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let listener = effect_handler.tcp_listener(addr)?;
            let local_addr = listener.local_addr().unwrap();

            // Notify the test of the actual bound address.
            let _ = self.port_notifier.send(local_addr);

            loop {
                tokio::select! {
                    // Process incoming control messages.
                    ctrl_msg = ctrl_msg_recv.recv() => {
                        let ctrl_msg = ctrl_msg?;
                        self.ctrl_msg_counters.update_with(&ctrl_msg);
                        if ctrl_msg.is_shutdown() {
                            break;
                        }
                    }

                    // Process incoming TCP connections.
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((mut socket, peer_addr)) => {
                                // Clone the effect handler so the spawned task can send messages.
                                let effect_handler = effect_handler.clone();
                                // Spawn a task to handle the connection.
                                // ToDo should be abstract that and expose a method in the effect handler?
                                _ = tokio::task::spawn_local(async move {
                                    let mut buf = [0u8; 1024];
                                    loop {
                                        match socket.read(&mut buf).await {
                                            Ok(0) => {
                                                break;
                                            },
                                            Ok(n) => {
                                                let received = String::from_utf8_lossy(&buf[..n]).to_string();
                                                // Create a TestMsg from the received data and send it.
                                                let msg = TestMsg(received);
                                                if let Err(e) = effect_handler.send_message(msg).await {
                                                    panic!("Error sending message via effect handler: {e}");
                                                }
                                                // Echo back an acknowledgment.
                                                let _ = socket.write_all(b"ack").await;
                                            },
                                            Err(e) => {
                                                panic!("Error reading from {peer_addr}: {e}");
                                            }
                                        }
                                    }
                                });
                            },
                            Err(e) => {
                                panic!("Error accepting connection: {e}");
                            }
                        }
                    }
                    // A timeout branch in case no events occur.
                    () = sleep(Duration::from_secs(1)) => {
                        // You could do periodic tasks here.
                    }
                }

                // For this test, exit the loop after 5 timer ticks.
                if self.ctrl_msg_counters.get_timer_tick_count() >= 5 {
                    break;
                }
            }

            Ok(())
        }
    }

    /// Test closure that simulates a typical receiver scenario.
    fn scenario(
        port_rx: oneshot::Receiver<SocketAddr>,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // Wait for the receiver to send the listening address.
                let addr: SocketAddr = port_rx.await.expect("Failed to receive listening address");

                // Connect to the receiver's socket.
                let mut stream = TcpStream::connect(addr)
                    .await
                    .expect("Failed to connect to receiver");

                // Send some test data.
                stream
                    .write_all(b"Hello from test client")
                    .await
                    .expect("Failed to send data");

                // Optionally, read an echo (acknowledgment) from the receiver.
                let mut buf = [0u8; 1024];
                let len = stream
                    .read(&mut buf)
                    .await
                    .expect("Failed to read response");
                assert_eq!(&buf[..len], b"ack", "Expected acknowledgment from receiver");

                // Send a few TimerTick events from the test.
                for _ in 0..3 {
                    ctx.send_timer_tick()
                        .await
                        .expect("Failed to send TimerTick");
                    ctx.sleep(Duration::from_millis(100)).await;
                }

                ctx.send_config(Value::Null)
                    .await
                    .expect("Failed to send config");

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(200), "Test")
                    .await
                    .expect("Failed to send Shutdown");

                // Close the TCP connection.
                let _ = stream.shutdown().await;
            })
        }
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<TestMsg>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Assert that the message received is what the test client sent.
                assert!(matches!(received, TestMsg(msg) if msg == "Hello from test client"));
                ctx.counters().assert(3, 0, 1, 1);
            })
        }
    }

    /// Test for the receiver in a `!Send` implementation.
    #[test]
    fn test_receiver_local() {
        let test_runtime = TestRuntime::new();
        let user_config = Arc::new(NodeUserConfig::new_receiver_config("test_receiver"));

        // Create a oneshot channel to receive the listening address from the receiver.
        let (port_tx, port_rx) = oneshot::channel();
        let receiver = ReceiverWrapper::local(
            TestReceiver::new(test_runtime.counters(), port_tx),
            test_node("recv"),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(port_rx))
            .run_validation(validation_procedure());
    }

    /// Test the receiver with a shared (Send) implementation.
    #[test]
    fn test_receiver_shared() {
        let test_runtime = TestRuntime::new();
        let user_config = Arc::new(NodeUserConfig::new_receiver_config("test_receiver"));

        // Create a oneshot channel to receive the listening address from the receiver.
        let (port_tx, port_rx) = oneshot::channel();
        let receiver = ReceiverWrapper::shared(
            TestReceiver::new(test_runtime.counters(), port_tx),
            test_node("recv"),
            user_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(port_rx))
            .run_validation(validation_procedure());
    }
}
