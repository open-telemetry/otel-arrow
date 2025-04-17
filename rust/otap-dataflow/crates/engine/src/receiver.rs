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
use std::net::SocketAddr;
use std::rc::Rc;
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
#[async_trait( ? Send)]
pub trait Receiver {
    /// The type of messages processed by the receiver.
    type PData;

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
        effect_handler: EffectHandler<Self::PData>,
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

/// Handles side effects such as opening network listeners or sending messages.
///
/// The `Msg` type parameter represents the type of message the receiver
/// will eventually produce.
///
/// Note for implementers: The `EffectHandler` is designed to be cloned and shared across tasks
/// so the cost of cloning should be minimal.
pub struct EffectHandler<Msg> {
    /// The name of the receiver.
    receiver_name: NodeName,

    /// A sender used to forward messages from the receiver.
    msg_sender: mpsc::Sender<Msg>,
}

impl<Msg> Clone for EffectHandler<Msg> {
    fn clone(&self) -> Self {
        EffectHandler {
            receiver_name: self.receiver_name.clone(),
            msg_sender: self.msg_sender.clone(),
        }
    }
}

impl<Msg> EffectHandler<Msg> {
    /// Creates a new `EffectHandler` with the given receiver name.
    pub fn new<S: AsRef<str>>(receiver_name: S, msg_sender: mpsc::Sender<Msg>) -> Self {
        EffectHandler {
            receiver_name: Rc::from(receiver_name.as_ref()),
            msg_sender,
        }
    }

    /// Returns the name of the receiver associated with this handler.
    #[must_use]
    pub fn receiver_name(&self) -> NodeName {
        self.receiver_name.clone()
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

#[cfg(test)]
mod tests {
    use super::ControlMsgChannel;
    use crate::message::ControlMsg;
    use crate::receiver::{EffectHandler, Error, Receiver};
    use crate::testing::{ReceiverTestRuntime, TestMsg};
    use async_trait::async_trait;
    use std::net::SocketAddr;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, sleep};

    struct TestReceiver {
        port_notifier: oneshot::Sender<SocketAddr>,
    }

    #[async_trait(?Send)]
    impl Receiver for TestReceiver {
        type PData = TestMsg;

        async fn start(
            self: Box<Self>,
            ctrl_msg_recv: ControlMsgChannel,
            effect_handler: EffectHandler<Self::PData>,
        ) -> Result<(), Error<Self::PData>> {
            // Bind to an ephemeral port.
            let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
            let listener = effect_handler.tcp_listener(addr)?;
            let local_addr = listener.local_addr().unwrap();
            println!("Receiver is listening on {}", local_addr);

            // Notify the test of the actual bound address.
            let _ = self.port_notifier.send(local_addr);

            let mut tick_count = 0;
            loop {
                tokio::select! {
                    // Process an internal event.
                    ctrl_msg = ctrl_msg_recv.recv() => {
                        match ctrl_msg {
                            Ok(ControlMsg::Shutdown {reason}) => {
                                println!("Received Shutdown event: {reason}");
                                break;
                            },
                            Ok(ControlMsg::TimerTick {}) => {
                                println!("Received TimerTick event.");
                                tick_count += 1;
                            },
                            Err(e) => {
                                return Err(Error::ChannelRecvError(e));
                            }
                            _ => {
                                eprintln!("Unknown control message received");
                            }
                        }
                    }
                    // Accept incoming TCP connections.
                    accept_result = listener.accept() => {
                        match accept_result {
                            Ok((mut socket, peer_addr)) => {
                                println!("Accepted connection from {peer_addr}");
                                // Clone the effect handler so the spawned task can send messages.
                                let effect_handler = effect_handler.clone();
                                // Spawn a task to handle the connection.
                                // ToDo should be abstract that and expose a method in the effect handler?
                                _ = tokio::task::spawn_local(async move {
                                    let mut buf = [0u8; 1024];
                                    loop {
                                        match socket.read(&mut buf).await {
                                            Ok(0) => {
                                                println!("Connection from {peer_addr} closed.");
                                                break;
                                            },
                                            Ok(n) => {
                                                let received = String::from_utf8_lossy(&buf[..n]).to_string();
                                                println!("Received from {peer_addr}: {received}");
                                                // Create a TestMsg from the received data and send it.
                                                let msg = TestMsg(received);
                                                if let Err(e) = effect_handler.send_message(msg).await {
                                                    eprintln!("Error sending message via effect handler: {e}");
                                                }
                                                // Echo back an acknowledgment.
                                                let _ = socket.write_all(b"ack").await;
                                            },
                                            Err(e) => {
                                                eprintln!("Error reading from {peer_addr}: {e}");
                                                break;
                                            }
                                        }
                                    }
                                });
                            },
                            Err(e) => {
                                eprintln!("Error accepting connection: {e}");
                            }
                        }
                    }
                    // A timeout branch in case no events occur.
                    () = sleep(Duration::from_secs(1)) => {
                        // You could do periodic tasks here.
                    }
                }

                // For this test, exit the loop after 5 timer ticks.
                if tick_count >= 5 {
                    println!("Timer tick count reached threshold; shutting down event loop.");
                    break;
                }
            }

            println!("Event loop terminated gracefully.");
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
        };

        test_runtime.start_receiver(receiver);
        test_runtime.spawn_with_context(|ctx| async move {
            // Wait for the receiver to send the listening address.
            let addr: SocketAddr = port_rx.await.expect("Failed to receive listening address");
            println!("Test received listening address: {addr}");

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
            let n = stream
                .read(&mut buf)
                .await
                .expect("Failed to read response");
            println!(
                "Test client received response: {}",
                String::from_utf8_lossy(&buf[..n])
            );

            // Send a few TimerTick events from the test.
            for _ in 0..3 {
                ctx.send_timer_tick()
                    .await
                    .expect("Failed to send TimerTick");
                ctx.sleep(Duration::from_millis(100)).await;
            }

            // Finally, send a Shutdown event to terminate the receiver.
            ctx.send_shutdown("Test")
                .await
                .expect("Failed to send Shutdown");

            // Close the TCP connection.
            let _ = stream.shutdown().await;
        });

        // Use the run_until method to run the test and validate the received message
        let received = test_runtime.run_until(|mut ctx| async move {
            let pdata_rx = ctx.pdata_rx().expect("No pdata_rx");
            tokio::time::timeout(Duration::from_secs(3), pdata_rx.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received")
        });

        // Assert that the message received is what the test client sent.
        assert!(matches!(received, TestMsg(msg) if msg == "Hello from test client"));
    }
}
