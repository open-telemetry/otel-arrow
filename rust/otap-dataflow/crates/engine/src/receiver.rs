// SPDX-License-Identifier: Apache-2.0

//! Set of traits and structures used to implement receivers.
//!
//! A receiver is an ingress node that feeds a pipeline with data from external sources while
//! performing the necessary conversions to produce messages in an OTEL compatible format (OTLP or
//! OTAP).
//!
//! A receiver can operate in various ways, including:
//!
//! 1. Listening on a socket to receive push-based telemetry data,
//! 2. Being notified of changes in a local directory (e.g. log file monitoring),
//! 3. Actively scraping an endpoint to retrieve the latest metrics from a system,
//! 4. Or using any other method to receive or extract telemetry data from external sources.

use crate::NodeName;
use crate::error::Error;
use crate::message::ControlMsg;
use async_trait::async_trait;
use otap_df_channel::error::RecvError;
use otap_df_channel::mpsc;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use tokio::net::TcpListener;

/// A trait for ingress receivers.
///
/// Receivers are responsible for accepting data from external sources and converting
/// it into messages that can be processed by the pipeline.
///
/// # Lifecycle
///
/// 1. The receiver is instantiated and configured
/// 2. The `start` method is called, which begins the receiver's operation
/// 3. The receiver processes both internal control messages and external data
/// 4. The receiver shuts down when it receives a `Shutdown` control message or encounters a fatal error
///
/// # Thread Safety
///
/// Note that this trait uses `#[async_trait(?Send)]`, meaning implementations
/// are not required to be thread-safe. To ensure scalability, the pipeline engine will start
/// multiple instances of the same pipeline in parallel, each with its own receiver instance.
///
/// Through the `Mode` type parameter, receivers can be configured to be either thread-local (`LocalMode`)
/// or thread-safe (`SendableMode`). This allows you to choose the appropriate threading model based on
/// your receiver's requirements and performance considerations.
#[async_trait( ? Send)]
pub trait Receiver {
    /// The type of messages processed by the receiver.
    type PData;

    /// The threading mode used by this receiver
    type Mode: ThreadMode;

    /// Starts the receiver and begins processing incoming data and control messages.
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
    ///    This can be either Send or !Send depending on the receiver's Mode type.
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
        effect_handler: EffectHandler<Self::PData, Self::Mode>,
    ) -> Result<(), Error<Self::PData>>;
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

/// Defines the thread-safety behavior for a pipeline component.
///
/// The `ThreadMode` trait is a crucial abstraction that allows components (receivers, processors, exporters)
/// to specify their threading behavior. By implementing either `LocalMode` (!Send) or `SendableMode` (Send),
/// components can declare whether they can be safely sent across thread boundaries.
///
/// For this project, `LocalMode` is the preferred and recommended mode for most components.
/// `SendableMode` exists primarily to support specific implementations that cannot use `LocalMode`,
/// such as OTLP Receivers based on Tonic GRPC services (which don't yet support ?Send trait declarations,
/// see <https://github.com/hyperium/tonic/issues/2171>).
///
/// General guidelines for choosing a mode:
/// - Use `LocalMode` for most components (preferred for this project)
/// - Use `SendableMode` only when integrating with libraries that require Send traits
pub trait ThreadMode: Clone + 'static {
    /// The reference type used for the node name.
    type NameRef: AsRef<str> + Clone;
}

/// Marker trait for Send-supporting handlers.
///
/// Components that implement this trait can safely be sent across thread boundaries.
/// This enables greater parallelism in pipeline execution at the potential cost of
/// synchronization overhead.
pub trait SendMode: ThreadMode {}

/// A thread-local (non-Send) mode for effect handlers.
///
/// Components using `LocalMode` are restricted to the thread they were created on,
/// but can use non-Send dependencies like `Rc` and avoid synchronization overhead.
/// This is the default and preferred mode for this project.
#[derive(Clone, Debug)]
pub struct LocalMode;

impl ThreadMode for LocalMode {
    type NameRef = Rc<str>;
}

/// A thread-safe (Send) mode for effect handlers.
///
/// Components using `SendableMode` can be sent across thread boundaries,
/// enabling greater parallelism. They use thread-safe dependencies like `Arc`
/// instead of `Rc` and must ensure all their state is `Send + Sync`.
#[derive(Clone, Debug)]
pub struct SendableMode;

impl ThreadMode for SendableMode {
    type NameRef = Arc<str>;
}

impl SendMode for SendableMode {}

/// Handles side effects such as opening network listeners or sending messages.
///
/// The `Msg` type parameter represents the type of message the receiver
/// will eventually produce, while the `Mode` type parameter determines the threading behavior.
///
/// # Thread Safety Options
///
/// - `EffectHandler<Msg, LocalMode>`: For thread-local (!Send) receivers. Uses `Rc` internally and is
///   the default for backward compatibility. Created with `EffectHandler::new()`.
/// - `EffectHandler<Msg, SendableMode>`: For thread-safe (Send) receivers. Uses `Arc` internally and
///   supports sending across thread boundaries. Created with `EffectHandler::new_sendable()`.
///
/// Choose the appropriate mode based on your component's requirements. Use `LocalMode` when thread safety
/// isn't needed or when using !Send dependencies, and `SendableMode` when the component must be shared
/// across threads.
///
/// Note for implementers: The `EffectHandler` is designed to be cloned and shared across tasks
/// so the cost of cloning should be minimal.
pub struct EffectHandler<Msg, Mode: ThreadMode = LocalMode> {
    /// The name of the receiver.
    receiver_name: Mode::NameRef,

    /// A sender used to forward messages from the receiver.
    msg_sender: mpsc::Sender<Msg>,

    /// Marker for the thread mode.
    _mode: PhantomData<Mode>,
}

impl<Msg, Mode: ThreadMode> Clone for EffectHandler<Msg, Mode> {
    fn clone(&self) -> Self {
        EffectHandler {
            receiver_name: self.receiver_name.clone(),
            msg_sender: self.msg_sender.clone(),
            _mode: PhantomData,
        }
    }
}

// Implementation for any mode
impl<Msg, Mode: ThreadMode> EffectHandler<Msg, Mode> {
    /// Returns the name of the receiver associated with this handler.
    #[must_use]
    pub fn receiver_name(&self) -> NodeName {
        // Convert to NodeName (Rc<str>) to maintain compatibility with existing API
        let s = self.receiver_name.as_ref();
        Rc::from(s)
    }

    /// Sends a message to the next node(s) in the pipeline.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::ReceiverError`] if the message could not be sent.
    pub async fn send_message(&self, data: Msg) -> Result<(), Error<Msg>> {
        self.msg_sender.send_async(data).await?;
        Ok(())
    }
}

// Implementation specific to LocalMode (default, non-Send)
impl<Msg> EffectHandler<Msg, LocalMode> {
    /// Creates a new local (non-Send) `EffectHandler` with the given receiver name.
    /// This is the default mode that maintains backward compatibility.
    pub fn new<S: AsRef<str>>(receiver_name: S, msg_sender: mpsc::Sender<Msg>) -> Self {
        EffectHandler {
            receiver_name: Rc::from(receiver_name.as_ref()),
            msg_sender,
            _mode: PhantomData,
        }
    }

    /// Creates a non-blocking TCP listener on the given address with `SO_REUSE` settings.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    pub fn tcp_listener(&self, addr: SocketAddr) -> Result<TcpListener, Error<Msg>> {
        // Helper closure to convert errors.
        let err = |error: std::io::Error| Error::IoError {
            node: self.receiver_name.clone(),
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
}

// Implementation for SendableMode (Send)
impl<Msg: Send + 'static> EffectHandler<Msg, SendableMode> {
    /// Creates a new thread-safe (Send) `EffectHandler` with the given receiver name.
    /// Use this when you need an EffectHandler that can be sent across thread boundaries.
    pub fn new_sendable<S: AsRef<str>>(receiver_name: S, msg_sender: mpsc::Sender<Msg>) -> Self {
        EffectHandler {
            receiver_name: Arc::from(receiver_name.as_ref()),
            msg_sender,
            _mode: PhantomData,
        }
    }

    /// Creates a non-blocking TCP listener on the given address with `SO_REUSE` settings.
    ///
    /// # Errors
    ///
    /// Returns an [`Error::IoError`] if any step in the process fails.
    pub fn tcp_listener(&self, addr: SocketAddr) -> Result<TcpListener, Error<Msg>> {
        // Helper function to convert errors - not using a closure to avoid move issues
        let name = self.receiver_name.to_string(); // Convert to owned String for thread safety
        let make_err = |error: std::io::Error| Error::IoError {
            node: Rc::from(name.as_str()),
            error,
        };

        // Create a SO_REUSEADDR + SO_REUSEPORT listener.
        let sock = match socket2::Socket::new(
            match addr {
                SocketAddr::V4(_) => socket2::Domain::IPV4,
                SocketAddr::V6(_) => socket2::Domain::IPV6,
            },
            socket2::Type::STREAM,
            None,
        ) {
            Ok(s) => s,
            Err(e) => return Err(make_err(e)),
        };

        // Allows multiple sockets to bind to an address/port combination even if a socket in the
        // TIME_WAIT state currently occupies that combination.
        // Goal: Restarting the server quickly without waiting for the OS to release a port.
        if let Err(e) = sock.set_reuse_address(true) {
            return Err(make_err(e));
        }

        // Explicitly allows multiple sockets to simultaneously bind and listen to the exact same
        // IP and port. Incoming connections or packets are distributed between the sockets
        // (load balancing).
        // Goal: Load balancing incoming connections.
        if let Err(e) = sock.set_reuse_port(true) {
            return Err(make_err(e));
        }

        if let Err(e) = sock.set_nonblocking(true) {
            return Err(make_err(e));
        }

        if let Err(e) = sock.bind(&addr.into()) {
            return Err(make_err(e));
        }

        if let Err(e) = sock.listen(8192) {
            return Err(make_err(e));
        }

        match TcpListener::from_std(sock.into()) {
            Ok(listener) => Ok(listener),
            Err(e) => Err(make_err(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::ControlMsgChannel;
    use crate::receiver::{EffectHandler, Error, LocalMode, Receiver};
    use crate::testing::receiver::ReceiverTestRuntime;
    use crate::testing::{CtrMsgCounters, TestMsg};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, sleep};

    struct TestReceiver {
        ctrl_msg_counters: CtrMsgCounters,
        port_notifier: oneshot::Sender<SocketAddr>,
    }

    #[async_trait(?Send)]
    impl Receiver for TestReceiver {
        type PData = TestMsg;
        type Mode = LocalMode;

        async fn start(
            self: Box<Self>,
            ctrl_msg_recv: ControlMsgChannel,
            effect_handler: EffectHandler<Self::PData, Self::Mode>,
        ) -> Result<(), Error<Self::PData>> {
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

    #[test]
    fn test_receiver() {
        let mut test_runtime = ReceiverTestRuntime::new(10);

        // Create a oneshot channel to receive the listening address from MyReceiver.
        let (port_tx, port_rx) = oneshot::channel();
        let receiver = TestReceiver {
            port_notifier: port_tx,
            ctrl_msg_counters: test_runtime.counters(),
        };

        test_runtime.start_receiver(receiver);
        test_runtime.start_test(|ctx| async move {
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
            _ = stream
                .read(&mut buf)
                .await
                .expect("Failed to read response");

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
            ctx.send_shutdown("Test")
                .await
                .expect("Failed to send Shutdown");

            // Close the TCP connection.
            let _ = stream.shutdown().await;
        });

        let counters = test_runtime.counters();
        test_runtime.validate(|mut ctx| async move {
            let pdata_rx = ctx.pdata_rx().expect("No pdata_rx");
            let received = tokio::time::timeout(Duration::from_secs(3), pdata_rx.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");

            // Assert that the message received is what the test client sent.
            assert!(matches!(received, TestMsg(msg) if msg == "Hello from test client"));
            counters.assert(3, 0, 1, 1);
        });
    }
}