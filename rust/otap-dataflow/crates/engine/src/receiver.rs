// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement receivers.
//!
//! A receiver is an ingress node that feeds a pipeline with data from external sources while
//! performing the necessary conversions to produce messages in a format recognized by the rest of
//! downstream pipeline nodes (e.g. OTLP or OTAP message format).
//!
//! A receiver can operate in various ways, including:
//!
//! 1. Listening on a socket to receive push-based telemetry data,
//! 2. Being notified of changes in a local directory (e.g. log file monitoring),
//! 3. Actively scraping an endpoint to retrieve the latest metrics from a system,
//! 4. Or using any other method to receive or extract telemetry data from external sources.
//!
//! # Lifecycle
//!
//! 1. The receiver is instantiated and configured.
//! 2. The `start` method is called, which begins the receiver's operation.
//! 3. The receiver processes both internal control messages and external data.
//! 4. The receiver shuts down when it receives a `Shutdown` control message or encounters a fatal
//!    error.
//!
//! # Thread Safety
//!
//! Note: By default, receivers use the `#[async_trait(?Send)]` attribute, allowing implementations
//! that are not thread-safe (`!Send`). If thread-safety (`Send`) is required, explicitly choose
//! the [`SendEffectHandler`] type. The default handler, [`NotSendEffectHandler`], is non-thread-safe.
//!
//! # Scalability
//!
//! To ensure scalability, the pipeline engine will start multiple instances of the same pipeline in
//! parallel on different cores, each with its own receiver instance.

use crate::config::ReceiverConfig;
use crate::error::Error;
use crate::message::{ControlMsg, PDataReceiver};
use async_trait::async_trait;
use otap_df_channel::error::{RecvError, SendError};
use otap_df_channel::mpsc;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use tokio::net::TcpListener;

/// A trait for ingress receivers.
///
/// Receivers are responsible for accepting data from external sources and converting
/// it into messages that can be processed by the pipeline.
#[async_trait( ? Send)]
pub trait Receiver<PData, EF = NotSendEffectHandler<PData>>
where
    EF: EffectHandlerTrait<PData>,
{
    /// Starts the receiver and begins processing incoming external data and control messages.
    ///
    /// The pipeline engine will call this function to start the receiver in a separate task.
    /// Receivers are assigned their own dedicated task at pipeline initialization because their
    /// primary function involves interacting with the external world, and the pipeline has no
    /// prior knowledge of when these interactions will occur.
    ///
    /// The `Box<Self>` signature indicates that when this method is called, the receiver takes
    /// exclusive ownership of its instance. This approach is necessary because a receiver cannot
    /// yield control back to the pipeline engine - it must independently manage its inputs and
    /// processing timing. The only way the pipeline engine can interact with the receiver after
    /// starting it is through the control message channel.
    ///
    /// Receivers are expected to process both internal control messages and external sources and
    /// use the EffectHandler to send messages to the next node(s) in the pipeline.
    ///
    /// Important note: Receivers are expected to process internal control messages in priority over
    /// external data.
    ///
    /// # Parameters
    ///
    /// - `ctrl_msg_chan`: A channel to receive control messages.
    /// - `effect_handler`: A handler to perform side effects such as opening a listener.
    ///    This can be either Send or !Send depending on the `EF` generic parameter.
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
        ctrl_msg_chan: ControlMsgChannel,
        effect_handler: EF,
    ) -> Result<(), Error<PData>>;
}

/// A channel for receiving control messages.
///
/// This structure wraps a receiver end of a channel that carries [`ControlMsg`]
/// values used to control the behavior of a receiver at runtime.
pub struct ControlMsgChannel {
    rx: mpsc::Receiver<ControlMsg>,
}

impl ControlMsgChannel {
    /// Creates a new `ControlMsgChannel` with the given receiver.
    #[must_use]
    pub fn new(rx: mpsc::Receiver<ControlMsg>) -> Self {
        ControlMsgChannel { rx }
    }

    /// Asynchronously receives the next control message.
    ///
    /// # Errors
    ///
    /// Returns a [`RecvError`] if the channel is closed.
    pub async fn recv(&self) -> Result<ControlMsg, RecvError> {
        self.rx.recv().await
    }
}

/// Handles side effects for the receiver such as opening network listeners or sending messages.
///
/// The `PData` type parameter represents the type of message the receiver will produce.
///
/// 2 implementations are provided:
///
/// - [`NotSendEffectHandler<PData>`]: For thread-local (!Send) receivers. Uses `Rc` internally.
///   It's the default and preferred effect handler.
/// - [`SendEffectHandler<PData>`]: For thread-safe (Send) receivers. Uses `Arc` internally and
///   supports sending across thread boundaries.
///
/// Note for implementers: Effect handler implementations are designed to be cloned so the cost of
/// cloning should be minimal.
#[async_trait(?Send)]
pub trait EffectHandlerTrait<PData> {
    /// Returns the name of the receiver associated with this handler.
    fn receiver_name(&self) -> &str;

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    async fn send_message(&self, data: PData) -> Result<(), Error<PData>>;

    /// Creates a non-blocking TCP listener on the given address with socket options defined by the
    /// pipeline engine implementation. It's important for receiver implementer to create TCP
    /// listeners via this method to ensure the scalability and the serviceability of the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    fn tcp_listener(&self, addr: SocketAddr) -> Result<TcpListener, Error<PData>> {
        // Helper closure to convert errors.
        let err = |error: std::io::Error| Error::IoError {
            node: self.receiver_name().to_owned(),
            error,
        };

        // Create a SO_REUSEADDR + SO_REUSEPORT listener.
        let sock = socket2::Socket::new(
            match addr {
                SocketAddr::V4(_) => socket2::Domain::IPV4,
                SocketAddr::V6(_) => socket2::Domain::IPV6,
            },
            socket2::Type::STREAM,
            None,
        )
        .map_err(err)?;

        // Allows multiple sockets to bind to an address/port combination even if a socket in the
        // TIME_WAIT state currently occupies that combination.
        // Goal: Restarting the server quickly without waiting for the OS to release a port.
        sock.set_reuse_address(true).map_err(err)?;
        // Explicitly allows multiple sockets to simultaneously bind and listen to the exact same
        // IP and port. Incoming connections or packets are distributed between the sockets
        // (load balancing).
        // Goal: Load balancing incoming connections.
        sock.set_reuse_port(true).map_err(err)?;
        sock.set_nonblocking(true).map_err(err)?;
        sock.bind(&addr.into()).map_err(err)?;
        sock.listen(8192).map_err(err)?;

        TcpListener::from_std(sock.into()).map_err(err)
    }

    // More methods will be added in the future as needed.
}

/// A `!Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct NotSendEffectHandler<PData> {
    /// The name of the receiver.
    receiver_name: Rc<str>,

    /// A sender used to forward messages from the receiver.
    msg_sender: mpsc::Sender<PData>,
}

/// Implementation for the `!Send` effect handler.
impl<PData> NotSendEffectHandler<PData> {
    /// Creates a new local (!Send) `EffectHandler` with the given receiver name.
    /// This is the default and preferred effect handler for this project.
    ///
    /// Use this constructor when your receiver doesn't need to be sent across threads or
    /// when it uses components that aren't `Send`.
    pub fn new<S: AsRef<str>>(receiver_name: S, msg_sender: mpsc::Sender<PData>) -> Self {
        NotSendEffectHandler {
            receiver_name: Rc::from(receiver_name.as_ref()),
            msg_sender,
        }
    }
}

#[async_trait(?Send)]
impl<PData> EffectHandlerTrait<PData> for NotSendEffectHandler<PData> {
    /// Returns the name of the receiver associated with this handler.
    #[must_use]
    fn receiver_name(&self) -> &str {
        &self.receiver_name
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    async fn send_message(&self, data: PData) -> Result<(), Error<PData>> {
        self.msg_sender.send_async(data).await?;
        Ok(())
    }
}

/// A `Send` implementation of the EffectHandlerTrait.
#[derive(Clone)]
pub struct SendEffectHandler<PData> {
    /// The name of the receiver.
    receiver_name: Arc<str>,

    /// A sender used to forward messages from the receiver.
    msg_sender: tokio::sync::mpsc::Sender<PData>,
}

/// Implementation for the `Send` effect handler.
impl<PData> SendEffectHandler<PData> {
    /// Creates a new sendable effect handler with the given receiver name.
    ///
    /// Use this constructor when your receiver do need to be sent across threads or
    /// when it uses components that are `Send`.
    pub fn new<S: AsRef<str>>(
        receiver_name: S,
        msg_sender: tokio::sync::mpsc::Sender<PData>,
    ) -> Self {
        SendEffectHandler {
            receiver_name: Arc::from(receiver_name.as_ref()),
            msg_sender,
        }
    }
}

#[async_trait(?Send)]
impl<PData> EffectHandlerTrait<PData> for SendEffectHandler<PData> {
    /// Returns the name of the receiver associated with this handler.
    #[must_use]
    fn receiver_name(&self) -> &str {
        &self.receiver_name
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ChannelSendError`] if the message could not be sent.
    async fn send_message(&self, data: PData) -> Result<(), Error<PData>> {
        self.msg_sender
            .send(data)
            .await
            .map_err(|tokio::sync::mpsc::error::SendError(pdata)| {
                Error::ChannelSendError(SendError::Full(pdata))
            })
    }
}

/// A wrapper for the receiver that allows for both `Send` and `!Send` effect handlers.
///
/// Note: This is useful for creating a single interface for the receiver regardless of the effect
/// handler type. This is the only type that the pipeline engine will use in order to be agnostic to
/// the effect handler type.
pub enum ReceiverWrapper<PData> {
    /// A receiver with a `!Send` effect handler.
    NotSend {
        /// The receiver instance.
        receiver: Box<dyn Receiver<PData, NotSendEffectHandler<PData>>>,
        /// The effect handler for the receiver.
        effect_handler: NotSendEffectHandler<PData>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<mpsc::Receiver<PData>>,
    },
    /// A receiver with a `Send` effect handler.
    Send {
        /// The receiver instance.
        receiver: Box<dyn Receiver<PData, SendEffectHandler<PData>>>,
        /// The effect handler for the receiver.
        effect_handler: SendEffectHandler<PData>,
        /// A receiver for pdata messages.
        pdata_receiver: Option<tokio::sync::mpsc::Receiver<PData>>,
    },
}

impl<PData> ReceiverWrapper<PData> {
    /// Creates a new `ReceiverWrapper` with the given receiver and appropriate effect handler.
    pub fn new<R, H>(receiver: R, config: &ReceiverConfig) -> Self
    where
        R: Receiver<PData, H> + 'static,
        H: EffectHandlerFactory<PData, R>,
    {
        H::create_wrapper(receiver, config)
    }

    /// Starts the receiver and begins receiver incoming data.
    pub async fn start(self, ctrl_msg_chan: ControlMsgChannel) -> Result<(), Error<PData>> {
        match self {
            ReceiverWrapper::NotSend {
                effect_handler,
                receiver,
                ..
            } => receiver.start(ctrl_msg_chan, effect_handler).await,
            ReceiverWrapper::Send {
                effect_handler,
                receiver,
                ..
            } => receiver.start(ctrl_msg_chan, effect_handler).await,
        }
    }

    /// Returns the PData receiver.
    pub fn take_pdata_receiver(&mut self) -> PDataReceiver<PData> {
        match self {
            ReceiverWrapper::NotSend { pdata_receiver, .. } => {
                PDataReceiver::NotSend(pdata_receiver.take().expect("pdata_receiver is None"))
            }
            ReceiverWrapper::Send { pdata_receiver, .. } => {
                PDataReceiver::Send(pdata_receiver.take().expect("pdata_receiver is None"))
            }
        }
    }
}

/// A trait that provides factory methods for creating effect handlers
/// and wrapping receivers.
pub trait EffectHandlerFactory<PData, R>
where
    R: Receiver<PData, Self> + 'static,
    Self: EffectHandlerTrait<PData> + Sized,
{
    /// Creates a new `ReceiverWrapper` with the appropriate type of
    /// effect handler for the given receiver.
    fn create_wrapper(receiver: R, config: &ReceiverConfig) -> ReceiverWrapper<PData>;
}

impl<PData, R> EffectHandlerFactory<PData, R> for NotSendEffectHandler<PData>
where
    R: Receiver<PData, Self> + 'static,
{
    fn create_wrapper(receiver: R, config: &ReceiverConfig) -> ReceiverWrapper<PData> {
        let (pdata_sender, pdata_receiver) =
            mpsc::Channel::new(config.output_pdata_channel.capacity);

        ReceiverWrapper::NotSend {
            effect_handler: NotSendEffectHandler::new(&config.name, pdata_sender),
            receiver: Box::new(receiver),
            pdata_receiver: Some(pdata_receiver),
        }
    }
}

impl<PData, R> EffectHandlerFactory<PData, R> for SendEffectHandler<PData>
where
    R: Receiver<PData, Self> + 'static,
{
    fn create_wrapper(receiver: R, config: &ReceiverConfig) -> ReceiverWrapper<PData> {
        let (pdata_sender, pdata_receiver) =
            tokio::sync::mpsc::channel(config.output_pdata_channel.capacity);

        ReceiverWrapper::Send {
            effect_handler: SendEffectHandler::new(&config.name, pdata_sender),
            receiver: Box::new(receiver),
            pdata_receiver: Some(pdata_receiver),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{
        ControlMsgChannel, EffectHandlerTrait, NotSendEffectHandler, ReceiverWrapper,
        SendEffectHandler,
    };
    use crate::receiver::{Error, Receiver};
    use crate::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use crate::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, sleep, timeout};

    /// A generic test receiver that counts message events
    /// Works with any effect handler that implements EffectHandlerTrait
    pub struct GenericTestReceiver<EF> {
        /// Counter for different message types
        ctrl_msg_counters: CtrlMsgCounters,
        /// Optional callback for testing sendable effect handlers
        test_send_eh: Option<fn(&EF)>,
        port_notifier: oneshot::Sender<SocketAddr>,
    }

    impl<EF> GenericTestReceiver<EF> {
        /// Creates a new test node
        pub fn new(
            ctrl_msg_counters: CtrlMsgCounters,
            port_notifier: oneshot::Sender<SocketAddr>,
        ) -> Self {
            GenericTestReceiver {
                ctrl_msg_counters,
                test_send_eh: None,
                port_notifier,
            }
        }

        /// Creates a new test node which requires a [`Send`] effect handler
        pub fn with_send_effect_handler(
            ctrl_msg_counters: CtrlMsgCounters,
            callback: fn(&EF),
            port_notifier: oneshot::Sender<SocketAddr>,
        ) -> Self {
            GenericTestReceiver {
                ctrl_msg_counters,
                test_send_eh: Some(callback),
                port_notifier,
            }
        }
    }

    #[async_trait(?Send)]
    impl<EF> Receiver<TestMsg, EF> for GenericTestReceiver<EF>
    where
        EF: EffectHandlerTrait<TestMsg> + Clone + 'static,
    {
        async fn start(
            self: Box<Self>,
            ctrl_msg_recv: ControlMsgChannel,
            effect_handler: EF,
        ) -> Result<(), Error<TestMsg>> {
            // Bind to an ephemeral port.
            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let listener = effect_handler.tcp_listener(addr)?;
            let local_addr = listener.local_addr().unwrap();

            // Notify the test of the actual bound address.
            let _ = self.port_notifier.send(local_addr);

            if let Some(test_send_eh) = self.test_send_eh {
                // Call the test callback if provided.
                test_send_eh(&effect_handler);
            }

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

    /// A type alias for a test receiver with regular effect handler
    type ReceiverWithNotSendEffectHandler = GenericTestReceiver<NotSendEffectHandler<TestMsg>>;

    /// A type alias for a test receiver with sendable effect handler
    type ReceiverWithSendEffectHandler = GenericTestReceiver<SendEffectHandler<TestMsg>>;

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

    #[test]
    fn test_receiver_with_not_send_effect_handler() {
        let test_runtime = TestRuntime::new();

        // Create a oneshot channel to receive the listening address from the receiver.
        let (port_tx, port_rx) = oneshot::channel();
        let receiver = ReceiverWrapper::new(
            ReceiverWithNotSendEffectHandler::new(test_runtime.counters(), port_tx),
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(port_rx))
            .run_validation(validation_procedure());
    }

    #[test]
    fn test_receiver_with_send_effect_handler() {
        let test_runtime = TestRuntime::new();

        // Create a oneshot channel to receive the listening address from the receiver.
        let (port_tx, port_rx) = oneshot::channel();
        let receiver = ReceiverWrapper::new(
            ReceiverWithSendEffectHandler::with_send_effect_handler(
                test_runtime.counters(),
                |effect_handler| {
                    exec_in_send_env(|| {
                        _ = effect_handler.receiver_name();
                    });
                },
                port_tx,
            ),
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(port_rx))
            .run_validation(validation_procedure());
    }
}
