use async_trait::async_trait;
use otap_df_engine::control::ControlMsg;
use otap_df_engine::{error::Error, local::receiver as local};
use serde_json::Value;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, BufReader};

#[allow(dead_code)]
const SYLOG_CEF_RECEIVER_URN: &str = "urn:otel:syslog_cef:receiver";

const BATCH_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100); // Maximum time to wait before building an Arrow batch
const MAX_BATCH_SIZE: usize = 100; // Maximum number of messages to build an Arrow batch

/// Protocol type for the receiver
#[derive(Debug, Clone)]
#[allow(dead_code)]
enum Protocol {
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
}

/// Syslog CEF receiver that can listen on TCP or UDP
#[allow(dead_code)]
struct SyslogCefReceiver {
    listening_addr: SocketAddr,
    /// The protocol to use for receiving messages
    protocol: Protocol,
}

impl SyslogCefReceiver {
    /// Creates a new SyslogCefReceiver with the specified listening address.
    #[must_use]
    #[allow(dead_code)]
    fn new(listening_addr: SocketAddr) -> Self {
        SyslogCefReceiver {
            listening_addr,
            protocol: Protocol::Udp,
        }
    }

    /// Creates a new SyslogCefReceiver from a configuration object
    #[must_use]
    #[allow(dead_code)]
    fn from_config(_config: &Value) -> Self {
        // ToDo: implement config parsing
        SyslogCefReceiver {
            listening_addr: "127.0.0.1:4317".parse().expect("Invalid socket address"),
            protocol: Protocol::Udp,
        }
    }
}

#[async_trait( ? Send)]
impl local::Receiver<Vec<u8>> for SyslogCefReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_chan: local::ControlChannel,
        effect_handler: local::EffectHandler<Vec<u8>>,
    ) -> Result<(), Error<Vec<u8>>> {
        match self.protocol {
            Protocol::Tcp => {
                let listener = effect_handler.tcp_listener(self.listening_addr)?;
                loop {
                    tokio::select! {
                        biased; //Prioritize control messages over data

                        // Process incoming control messages.
                        ctrl_msg = ctrl_chan.recv() => {
                            match ctrl_msg {
                                Ok(ControlMsg::Shutdown {..}) => {
                                // ToDo: Add proper deadline function
                                break;
                                },
                            Err(e) => {
                                return Err(Error::ChannelRecvError(e));
                                }
                            _ => {
                                // ToDo: Handle other control messages if needed
                                }
                            }
                        }

                        // Process incoming TCP connections.
                        accept_result = listener.accept() => {
                            match accept_result {
                                Ok((socket, _peer_addr)) => {
                                    // Clone the effect handler so the spawned task can send messages.
                                    let effect_handler = effect_handler.clone();
                                    // Spawn a task to handle the connection.
                                    // ToDo should this be abstracted and exposed a method in the effect handler?
                                    _ = tokio::task::spawn_local(async move {
                                        let mut reader = BufReader::new(socket);
                                        let mut line = String::new();

                                        loop {
                                            line.clear();

                                            // ToDo: Need to handle malicious input
                                            // This could lead to memory exhaustion if there is no newline in the input.
                                            match reader.read_line(&mut line).await {
                                                Ok(0) => {
                                                    // EOF reached - connection closed
                                                    // read_line() handles incomplete lines (without \n) automatically
                                                    break;
                                                },
                                                Ok(_) => {

                                                    let is_complete_line = line.ends_with('\n');
                                                    if !is_complete_line {
                                                        // ToDo: Handle incomplete lines
                                                        // Do we process the incomplete line with partial data or discard it?
                                                        // Handle incomplete line (log, emit metrics, etc.)
                                                    }

                                                    // ToDo: Validate the received data before processing
                                                    if let Err(_e) = effect_handler.send_message(line.as_bytes().to_vec()).await {
                                                        return; // Break out of the entire task
                                                    }
                                                },
                                                Err(_e) => {
                                                    break; // ToDo: Handle read error properly
                                                }
                                            }
                                        }
                                    });
                                },
                                Err(e) => {
                                    return Err(Error::ReceiverError{receiver: effect_handler.receiver_id(), error: e.to_string()});
                                }
                            }
                        }
                    }
                }
            }
            Protocol::Udp => {
                let socket = effect_handler.udp_socket(self.listening_addr)?;
                let mut buf = [0u8; 1024]; // ToDo: Find out the maximum allowed size for syslog messages

                let start = tokio::time::Instant::now() + BATCH_TIMEOUT;
                let mut interval = tokio::time::interval_at(start, BATCH_TIMEOUT);

                loop {
                    tokio::select! {
                        biased; //Prioritize control messages over data

                        // Process incoming control messages.
                        ctrl_msg = ctrl_chan.recv() => {
                            match ctrl_msg {
                                Ok(ControlMsg::Shutdown {..}) => {
                                // ToDo: Add proper deadline function
                                break;
                                },
                            Err(e) => {
                                return Err(Error::ChannelRecvError(e));
                                }
                            _ => {
                                // ToDo: Handle other control messages if needed
                                }
                            }
                        }

                        result = socket.recv_from(&mut buf) => {
                            match result {
                                Ok((n, _peer_addr)) => {
                                    // ToDo: Validate the received data before processing
                                    // ToDo: Consider logging or using peer_addr for security/auditing
                                    effect_handler.send_message(buf[..n].to_vec()).await?;
                                },
                                Err(e) => {
                                    return Err(Error::ReceiverError{receiver: effect_handler.receiver_id(), error: e.to_string()});
                                }
                            }
                        },
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::rc::Rc;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpStream, UdpSocket};
    use tokio::time::{Duration, timeout};

    /// Test closure that simulates a typical UDP syslog receiver scenario.
    fn udp_scenario(
        listening_addr: SocketAddr,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // Create a UDP socket to send test data
                let socket = UdpSocket::bind("127.0.0.1:0")
                    .await
                    .expect("Failed to bind UDP socket");

                // Sample syslog CEF message
                let test_message = b"<134>1 2023-06-25T10:30:00.123Z test-host test-app 1234 ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|3|src=10.0.0.1 dst=10.0.0.2";

                // Send the test message to the receiver
                let _bytes_sent = socket
                    .send_to(test_message, listening_addr)
                    .await
                    .expect("Failed to send UDP message");

                // Send another test message
                let test_message2 = b"<86>1 2023-06-25T10:31:00.456Z host2 app2 5678 ID48 - CEF:0|Vendor|Product|1.1|200|test event|5|msg=test message";
                let _bytes_sent2 = socket
                    .send_to(test_message2, listening_addr)
                    .await
                    .expect("Failed to send second UDP message");

                // Wait a bit for messages to be processed
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(0), "Test")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Test closure that simulates a TCP syslog receiver scenario.
    fn tcp_scenario(
        listening_addr: SocketAddr,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // Connect to the TCP server
                let mut stream = TcpStream::connect(listening_addr)
                    .await
                    .expect("Failed to connect to TCP server");

                // Sample syslog CEF messages
                let test_message1 = b"<134>1 2023-06-25T10:30:00.123Z test-host test-app 1234 ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|3|src=10.0.0.1 dst=10.0.0.2\n";
                let test_message2 = b"<86>1 2023-06-25T10:31:00.456Z host2 app2 5678 ID48 - CEF:0|Vendor|Product|1.1|200|test event|5|msg=test message\n";

                // Send test messages
                stream
                    .write_all(test_message1)
                    .await
                    .expect("Failed to write first message");
                stream.flush().await.expect("Failed to flush first message");

                stream
                    .write_all(test_message2)
                    .await
                    .expect("Failed to write second message");
                stream
                    .flush()
                    .await
                    .expect("Failed to flush second message");

                // Wait a bit for messages to be processed
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Close the connection
                drop(stream);

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(0), "Test")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Test closure that simulates a TCP syslog receiver scenario with incomplete lines.
    fn tcp_incomplete_scenario(
        listening_addr: SocketAddr,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // Connect to the TCP server
                let mut stream = TcpStream::connect(listening_addr)
                    .await
                    .expect("Failed to connect to TCP server");

                // Sample syslog CEF messages - one with newline, one without
                let test_message1 = b"<134>1 2023-06-25T10:30:00.123Z test-host test-app 1234 ID47 - CEF:0|Security|threatmanager|1.0|100|complete message|3|src=10.0.0.1\n";
                let test_message2 = b"<86>1 2023-06-25T10:31:00.456Z host2 app2 5678 ID48 - CEF:0|Vendor|Product|1.1|200|incomplete message|5|msg=no newline";

                // Send complete message with newline
                stream
                    .write_all(test_message1)
                    .await
                    .expect("Failed to write first message");
                stream.flush().await.expect("Failed to flush first message");

                // Send incomplete message without newline
                stream
                    .write_all(test_message2)
                    .await
                    .expect("Failed to write second message");
                stream
                    .flush()
                    .await
                    .expect("Failed to flush second message");

                // Wait a bit for messages to be processed
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Close the connection - this should trigger sending of remaining data
                drop(stream);

                // Wait a bit more for the EOF handling
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(0), "Test")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the received messages for UDP test.
    fn udp_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<Vec<u8>>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // Check that messages have been received through the effect_handler

                // Read the first message
                let message1_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for first message")
                    .expect("No first message received");

                // Verify the content of the first message
                let expected_message1 = b"<134>1 2023-06-25T10:30:00.123Z test-host test-app 1234 ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|3|src=10.0.0.1 dst=10.0.0.2";
                assert_eq!(
                    message1_received,
                    expected_message1.to_vec(),
                    "First message content mismatch"
                );

                // Read the second message
                let message2_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for second message")
                    .expect("No second message received");

                // Verify the content of the second message
                let expected_message2 = b"<86>1 2023-06-25T10:31:00.456Z host2 app2 5678 ID48 - CEF:0|Vendor|Product|1.1|200|test event|5|msg=test message";
                assert_eq!(
                    message2_received,
                    expected_message2.to_vec(),
                    "Second message content mismatch"
                );
            })
        }
    }

    /// Validation closure that checks the received messages for TCP test.
    fn tcp_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<Vec<u8>>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // Read the first message
                let message1_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for first message")
                    .expect("No first message received");

                // Verify the content of the first message
                let expected_message1 = b"<134>1 2023-06-25T10:30:00.123Z test-host test-app 1234 ID47 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] CEF:0|Security|threatmanager|1.0|100|worm successfully stopped|3|src=10.0.0.1 dst=10.0.0.2\n";
                assert_eq!(
                    message1_received,
                    expected_message1.to_vec(),
                    "First message content mismatch"
                );

                // Read the second message
                let message2_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for second message")
                    .expect("No second message received");

                // Verify the content of the second message
                let expected_message2 = b"<86>1 2023-06-25T10:31:00.456Z host2 app2 5678 ID48 - CEF:0|Vendor|Product|1.1|200|test event|5|msg=test message\n";
                assert_eq!(
                    message2_received,
                    expected_message2.to_vec(),
                    "Second message content mismatch"
                );
            })
        }
    }

    /// Validation closure that checks the received messages for TCP incomplete test.
    fn tcp_incomplete_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<Vec<u8>>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // Read the first message (complete with newline)
                let message1_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for first message")
                    .expect("No first message received");

                // Verify the content of the first message
                let expected_message1 = b"<134>1 2023-06-25T10:30:00.123Z test-host test-app 1234 ID47 - CEF:0|Security|threatmanager|1.0|100|complete message|3|src=10.0.0.1\n";
                assert_eq!(
                    message1_received,
                    expected_message1.to_vec(),
                    "First message content mismatch"
                );

                // Read the second message (incomplete, should be sent on EOF)
                let message2_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for second message")
                    .expect("No second message received");

                // Verify the content of the second message (no newline)
                let expected_message2 = b"<86>1 2023-06-25T10:31:00.456Z host2 app2 5678 ID48 - CEF:0|Vendor|Product|1.1|200|incomplete message|5|msg=no newline";
                assert_eq!(
                    message2_received,
                    expected_message2.to_vec(),
                    "Second message content mismatch"
                );
            })
        }
    }

    #[test]
    fn test_syslog_cef_receiver_udp() {
        let test_runtime = TestRuntime::new();

        // addr and port for the UDP server to run at
        let listening_port = portpicker::pick_unused_port().expect("No free ports");
        let listening_addr: SocketAddr = format!("127.0.0.1:{listening_port}").parse().unwrap();

        // create our UDP receiver
        let node_config = Rc::new(NodeUserConfig::new_exporter_config(SYLOG_CEF_RECEIVER_URN));
        let receiver = ReceiverWrapper::local(
            SyslogCefReceiver::new(listening_addr),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(udp_scenario(listening_addr))
            .run_validation(udp_validation_procedure());
    }

    #[test]
    fn test_syslog_cef_receiver_tcp() {
        let test_runtime = TestRuntime::new();

        // addr and port for the TCP server to run at
        let listening_port = portpicker::pick_unused_port().expect("No free ports");
        let listening_addr: SocketAddr = format!("127.0.0.1:{listening_port}").parse().unwrap();

        // create our TCP receiver - we need to modify the receiver to support TCP
        let mut receiver = SyslogCefReceiver::new(listening_addr);
        receiver.protocol = Protocol::Tcp;

        let node_config = Rc::new(NodeUserConfig::new_exporter_config(SYLOG_CEF_RECEIVER_URN));
        let receiver_wrapper = ReceiverWrapper::local(receiver, node_config, test_runtime.config());

        // run the test
        test_runtime
            .set_receiver(receiver_wrapper)
            .run_test(tcp_scenario(listening_addr))
            .run_validation(tcp_validation_procedure());
    }

    #[test]
    fn test_syslog_cef_receiver_tcp_incomplete() {
        let test_runtime = TestRuntime::new();

        // addr and port for the TCP server to run at
        let listening_port = portpicker::pick_unused_port().expect("No free ports");
        let listening_addr: SocketAddr = format!("127.0.0.1:{listening_port}").parse().unwrap();

        // create our TCP receiver
        let mut receiver = SyslogCefReceiver::new(listening_addr);
        receiver.protocol = Protocol::Tcp;

        let node_config = Rc::new(NodeUserConfig::new_exporter_config(SYLOG_CEF_RECEIVER_URN));
        let receiver_wrapper = ReceiverWrapper::local(receiver, node_config, test_runtime.config());

        // run the test
        test_runtime
            .set_receiver(receiver_wrapper)
            .run_test(tcp_incomplete_scenario(listening_addr))
            .run_validation(tcp_incomplete_validation_procedure());
    }
}
