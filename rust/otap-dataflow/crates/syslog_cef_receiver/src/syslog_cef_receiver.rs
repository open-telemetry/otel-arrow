// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

use async_trait::async_trait;
use otap_df_engine::control::ControlMsg;
use otap_df_engine::{error::Error, local::receiver as local};
use otap_df_otap::pdata::OtapPdata;
use serde_json::Value;
use std::net::SocketAddr;
use tokio::io::{AsyncBufReadExt, BufReader};

use crate::arrow_records_encoder::ArrowRecordsBuilder;

#[allow(dead_code)]
const SYLOG_CEF_RECEIVER_URN: &str = "urn:otel:syslog_cef:receiver";

const BATCH_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100); // Maximum time to wait before building an Arrow batch
const MAX_BATCH_SIZE: u16 = 100; // Maximum number of messages to build an Arrow batch

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

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for SyslogCefReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_chan: local::ControlChannel,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error<OtapPdata>> {
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
                                        let mut line_bytes = Vec::new();

                                        let mut arrow_records_builder = ArrowRecordsBuilder::new();

                                        let start = tokio::time::Instant::now() + BATCH_TIMEOUT;
                                        let mut interval = tokio::time::interval_at(start, BATCH_TIMEOUT);

                                        loop {
                                            tokio::select! {
                                                biased; // Prioritize incoming data over timeout

                                                // Handle incoming data
                                                read_result = reader.read_until(b'\n', &mut line_bytes) => {
                                                    // ToDo: Need to handle malicious input
                                                    // This could lead to memory exhaustion if there is no newline in the input.
                                                    match read_result {
                                                        Ok(0) => {
                                                            // EOF reached - connection closed
                                                            // Check if there's an incomplete line to process
                                                            if !line_bytes.is_empty() {
                                                                // Remove trailing newline if present
                                                                let message_bytes = if line_bytes.last() == Some(&b'\n') {
                                                                    &line_bytes[..line_bytes.len()-1]
                                                                } else {
                                                                    &line_bytes[..]
                                                                };
                                                                if let Ok(parsed_message) = crate::parser::parse(message_bytes) {
                                                                    arrow_records_builder.append_syslog(parsed_message);
                                                                }
                                                            }

                                                            // Send any remaining records before closing
                                                            if arrow_records_builder.len() > 0 {
                                                                let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");
                                                                let _ = effect_handler.send_message(OtapPdata::from(arrow_records)).await;
                                                            }
                                                            break;
                                                        },
                                                        Ok(_) => {

                                                            let is_complete_line = line_bytes.last() == Some(&b'\n');
                                                            if !is_complete_line {
                                                                // ToDo: Handle incomplete lines
                                                                // Do we process the incomplete line with partial data or discard it?
                                                                // Handle incomplete line (log, emit metrics, etc.)
                                                            }

                                                            // Strip the newline character for parsing
                                                            let message_to_parse = if is_complete_line {
                                                                &line_bytes[..line_bytes.len()-1]
                                                            } else {
                                                                &line_bytes[..]
                                                            };

                                                            let parsed_message = match crate::parser::parse(message_to_parse) {
                                                                Ok(parsed) => parsed,
                                                                Err(_e) => {
                                                                    // ToDo: Handle parsing error (log, emit metrics, etc.)
                                                                    continue; // Skip this message
                                                                }
                                                            };

                                                            arrow_records_builder.append_syslog(parsed_message);

                                                            // Clear the bytes for the next iteration
                                                            line_bytes.clear();

                                                            if arrow_records_builder.len() >= MAX_BATCH_SIZE {
                                                                // Build the Arrow records to send them
                                                                let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");

                                                                // Reset the builder for the next batch
                                                                arrow_records_builder = ArrowRecordsBuilder::new();

                                                                // Reset the timer since we already built an arrow record batch due to size constraint
                                                                interval.reset();

                                                                if let Err(_e) = effect_handler.send_message(OtapPdata::from(arrow_records)).await {
                                                                    return; // Break out of the entire task
                                                                }
                                                            }
                                                        },
                                                        Err(_e) => {
                                                            // Send any remaining records before closing due to error
                                                            if arrow_records_builder.len() > 0 {
                                                                let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");
                                                                let _ = effect_handler.send_message(OtapPdata::from(arrow_records)).await;
                                                            }
                                                            break; // ToDo: Handle read error properly
                                                        }
                                                    }
                                                }

                                                // Handle timeout - send any accumulated records
                                                _ = interval.tick() => {
                                                    if arrow_records_builder.len() > 0 {
                                                        // Build the Arrow records and send them
                                                        let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");

                                                        // Reset the builder for the next batch
                                                        arrow_records_builder = ArrowRecordsBuilder::new();

                                                        if let Err(_e) = effect_handler.send_message(OtapPdata::from(arrow_records)).await {
                                                            return; // Break out of the entire task
                                                        }
                                                    }
                                                },
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
                let mut arrow_records_builder = ArrowRecordsBuilder::new();

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
                        },

                        result = socket.recv_from(&mut buf) => {
                            match result {
                                Ok((n, _peer_addr)) => {
                                    // ToDo: Validate the received data before processing
                                    // ToDo: Consider logging or using peer_addr for security/auditing
                                    let parsed_message = match crate::parser::parse(&buf[..n]) {
                                        Ok(parsed) => parsed,
                                        Err(_e) => {
                                            // ToDo: Handle parsing error (log, emit metrics, etc.)
                                            continue; // Skip this message
                                        }
                                    };

                                    arrow_records_builder.append_syslog(parsed_message);

                                    if arrow_records_builder.len() >= MAX_BATCH_SIZE {
                                        // Build the Arrow records to send them
                                        let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");

                                        // Reset the builder for the next batch
                                        arrow_records_builder = ArrowRecordsBuilder::new();

                                        // Reset the timer since we already built an arrow record batch due to size constraint
                                        interval.reset();

                                        effect_handler.send_message(OtapPdata::from(arrow_records)).await?;
                                    }
                                },
                                Err(e) => {
                                    return Err(Error::ReceiverError{receiver: effect_handler.receiver_id(), error: e.to_string()});
                                }
                            }
                        },

                        _ = interval.tick() => {
                            // Check if we have any records to send
                            if arrow_records_builder.len() > 0 {
                                // Build the Arrow records and send them
                                let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");

                                // Reset the builder for the next batch
                                arrow_records_builder = ArrowRecordsBuilder::new();

                                effect_handler.send_message(OtapPdata::from(arrow_records)).await?;
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
    use arrow::array::Array;
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

                let test_message1 = b"<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";

                // Send the test message to the receiver
                let _bytes_sent = socket
                    .send_to(test_message1, listening_addr)
                    .await
                    .expect("Failed to send UDP message");

                // Send another test message
                let test_message2 = b"<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully";
                let _bytes_sent2 = socket
                    .send_to(test_message2, listening_addr)
                    .await
                    .expect("Failed to send second UDP message");

                // Wait a bit for messages to be processed
                tokio::time::sleep(Duration::from_millis(150)).await;

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
                let test_message1 = b"<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8\n";
                let test_message2 = b"<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully\n";

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
                tokio::time::sleep(Duration::from_millis(150)).await;

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

                // Sample syslog messages - one with newline, one without
                let test_message1 = b"<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8\n";
                let test_message2 = b"<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully";

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
                tokio::time::sleep(Duration::from_millis(150)).await;

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
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                // Check that messages have been received through the effect_handler

                // Read the first message
                let message1_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for first message")
                    .expect("No first message received");

                // Extract arrow_records for further validation
                let OtapPdata::OtapArrowRecords(arrow_records) = message1_received else {
                    panic!("Expected OtapArrowRecords::Logs variant")
                };

                // Check that the ArrowRecords contains the expected payload types
                let logs_record_batch = arrow_records
                    .get(ArrowPayloadType::Logs)
                    .expect("Expected Logs record batch to be present");

                // Verify the number of log records
                assert_eq!(
                    logs_record_batch.num_rows(),
                    2,
                    "Expected 2 log records in the batch"
                );

                // Assert that LogAttrs payload is always present and has records
                let log_attrs_batch = arrow_records
                    .get(ArrowPayloadType::LogAttrs)
                    .expect("LogAttrs batch should always be present");
                assert!(
                    log_attrs_batch.num_rows() > 0,
                    "LogAttrs batch should have positive number of rows"
                );

                // Verify the Arrow schema contains expected columns
                let schema = logs_record_batch.schema();
                let column_names: Vec<&str> =
                    schema.fields().iter().map(|f| f.name().as_str()).collect();

                // Check for essential log record columns
                assert!(
                    column_names.contains(&"body"),
                    "Logs record batch should contain 'body' column"
                );
                assert!(
                    column_names.contains(&"severity_number"),
                    "Logs record batch should contain 'severity_number' column"
                );
                assert!(
                    column_names.contains(&"severity_text"),
                    "Logs record batch should contain 'severity_text' column"
                );
                assert!(
                    column_names.contains(&"time_unix_nano"),
                    "Logs record batch should contain 'time_unix_nano' column"
                );

                // Validate using Arrow record batch methods directly
                // Check the body column to verify message content
                let body_column = logs_record_batch
                    .column_by_name("body")
                    .expect("Body column should exist");

                // The body column is a struct with fields: type (UInt8) and str (Dictionary)
                let struct_array = body_column
                    .as_any()
                    .downcast_ref::<arrow::array::StructArray>()
                    .expect("Body column should be a StructArray");

                // Get the str field which contains the actual string content
                let str_field = struct_array
                    .column_by_name("str")
                    .expect("Body struct should have 'str' field");

                // The str field is a Dictionary array
                let dict_array = str_field
                    .as_any()
                    .downcast_ref::<arrow::array::DictionaryArray<arrow::datatypes::UInt16Type>>()
                    .expect("str field should be a Dictionary array");

                // Get the values from the dictionary
                let values = dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<arrow::array::StringArray>()
                    .expect("Dictionary values should be StringArray");

                // Expected test messages
                let expected_message1 = "<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";
                let expected_message2 = "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully";

                // Get the actual body content for each record
                let body1_key = dict_array.key(0).expect("First record should exist");
                let body2_key = dict_array.key(1).expect("Second record should exist");

                let body1 = values.value(body1_key);
                let body2 = values.value(body2_key);

                // Verify that the body content matches the input messages
                assert_eq!(
                    body1, expected_message1,
                    "First message body content mismatch"
                );
                assert_eq!(
                    body2, expected_message2,
                    "Second message body content mismatch"
                );
            })
        }
    }

    /// Validation closure that checks the received messages for TCP test.
    fn tcp_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                // Check that messages have been received through the effect_handler

                // Read the first message
                let message1_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for first message")
                    .expect("No first message received");

                // Extract arrow_records for further validation
                let OtapPdata::OtapArrowRecords(arrow_records) = message1_received else {
                    panic!("Expected OtapArrowRecords::Logs variant")
                };

                // Check that the ArrowRecords contains the expected payload types
                let logs_record_batch = arrow_records
                    .get(ArrowPayloadType::Logs)
                    .expect("Expected Logs record batch to be present");

                // Verify the number of log records
                assert_eq!(
                    logs_record_batch.num_rows(),
                    2,
                    "Expected 2 log records in the batch"
                );

                // Assert that LogAttrs payload is always present and has records
                let log_attrs_batch = arrow_records
                    .get(ArrowPayloadType::LogAttrs)
                    .expect("LogAttrs batch should always be present");
                assert!(
                    log_attrs_batch.num_rows() > 0,
                    "LogAttrs batch should have positive number of rows"
                );

                // Verify the Arrow schema contains expected columns
                let schema = logs_record_batch.schema();
                let column_names: Vec<&str> =
                    schema.fields().iter().map(|f| f.name().as_str()).collect();

                // Check for essential log record columns
                assert!(
                    column_names.contains(&"body"),
                    "Logs record batch should contain 'body' column"
                );
                assert!(
                    column_names.contains(&"severity_number"),
                    "Logs record batch should contain 'severity_number' column"
                );
                assert!(
                    column_names.contains(&"severity_text"),
                    "Logs record batch should contain 'severity_text' column"
                );
                assert!(
                    column_names.contains(&"time_unix_nano"),
                    "Logs record batch should contain 'time_unix_nano' column"
                );

                // Validate using Arrow record batch methods directly
                // Check the body column to verify message content
                let body_column = logs_record_batch
                    .column_by_name("body")
                    .expect("Body column should exist");

                // The body column is a struct with fields: type (UInt8) and str (Dictionary)
                let struct_array = body_column
                    .as_any()
                    .downcast_ref::<arrow::array::StructArray>()
                    .expect("Body column should be a StructArray");

                // Get the str field which contains the actual string content
                let str_field = struct_array
                    .column_by_name("str")
                    .expect("Body struct should have 'str' field");

                // The str field is a Dictionary array
                let dict_array = str_field
                    .as_any()
                    .downcast_ref::<arrow::array::DictionaryArray<arrow::datatypes::UInt16Type>>()
                    .expect("str field should be a Dictionary array");

                // Get the values from the dictionary
                let values = dict_array
                    .values()
                    .as_any()
                    .downcast_ref::<arrow::array::StringArray>()
                    .expect("Dictionary values should be StringArray");

                // Expected test messages
                let expected_message1 = "<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";
                let expected_message2 = "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully";

                // Get the actual body content for each record
                let body1_key = dict_array.key(0).expect("First record should exist");
                let body2_key = dict_array.key(1).expect("Second record should exist");

                let body1 = values.value(body1_key);
                let body2 = values.value(body2_key);

                // Verify that the body content matches the input messages
                assert_eq!(
                    body1, expected_message1,
                    "First message body content mismatch"
                );
                assert_eq!(
                    body2, expected_message2,
                    "Second message body content mismatch"
                );
            })
        }
    }

    /// Validation closure that checks the received messages for TCP incomplete test.
    fn tcp_incomplete_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                // Check that messages have been received through the effect_handler
                // Note: Messages might come in separate batches due to timing

                let mut total_records = 0;
                let mut received_messages = Vec::new();

                // Collect all messages within a reasonable timeout
                while total_records < 2 {
                    match timeout(Duration::from_secs(3), ctx.recv()).await {
                        Ok(Ok(message)) => {
                            let OtapPdata::OtapArrowRecords(arrow_records) = message else {
                                panic!("Expected OtapArrowRecords variant")
                            };

                            let logs_record_batch = arrow_records
                                .get(ArrowPayloadType::Logs)
                                .expect("Expected Logs record batch to be present");

                            total_records += logs_record_batch.num_rows();
                            received_messages.push(arrow_records);
                        }
                        Ok(Err(_)) => break, // Channel closed
                        Err(_) => break,     // Timeout
                    }
                }

                // Verify we received exactly 2 records total
                assert_eq!(
                    total_records, 2,
                    "Expected 2 log records total across all batches"
                );

                // Validate the content by checking the first message (should contain at least one record)
                let first_arrow_records = &received_messages[0];
                let logs_record_batch = first_arrow_records
                    .get(ArrowPayloadType::Logs)
                    .expect("Expected Logs record batch to be present");

                // Assert that LogAttrs payload is always present and has records
                let log_attrs_batch = first_arrow_records
                    .get(ArrowPayloadType::LogAttrs)
                    .expect("LogAttrs batch should always be present");
                assert!(
                    log_attrs_batch.num_rows() > 0,
                    "LogAttrs batch should have positive number of rows"
                );

                // Verify the Arrow schema contains expected columns
                let schema = logs_record_batch.schema();
                let column_names: Vec<&str> =
                    schema.fields().iter().map(|f| f.name().as_str()).collect();

                // Check for essential log record columns
                assert!(
                    column_names.contains(&"body"),
                    "Logs record batch should contain 'body' column"
                );
                assert!(
                    column_names.contains(&"severity_number"),
                    "Logs record batch should contain 'severity_number' column"
                );
                assert!(
                    column_names.contains(&"severity_text"),
                    "Logs record batch should contain 'severity_text' column"
                );
                assert!(
                    column_names.contains(&"time_unix_nano"),
                    "Logs record batch should contain 'time_unix_nano' column"
                );

                // Get all the message bodies from all batches for validation
                let mut all_bodies = Vec::new();
                for arrow_records in &received_messages {
                    let logs_batch = arrow_records
                        .get(ArrowPayloadType::Logs)
                        .expect("Expected Logs record batch to be present");

                    let body_column = logs_batch
                        .column_by_name("body")
                        .expect("Body column should exist");

                    let struct_array = body_column
                        .as_any()
                        .downcast_ref::<arrow::array::StructArray>()
                        .expect("Body column should be a StructArray");

                    let str_field = struct_array
                        .column_by_name("str")
                        .expect("Body struct should have 'str' field");

                    let dict_array = str_field.as_any().downcast_ref::<arrow::array::DictionaryArray<arrow::datatypes::UInt16Type>>()
                        .expect("str field should be a Dictionary array");

                    let values = dict_array
                        .values()
                        .as_any()
                        .downcast_ref::<arrow::array::StringArray>()
                        .expect("Dictionary values should be StringArray");

                    for i in 0..logs_batch.num_rows() {
                        let key = dict_array.key(i).expect("Record should exist");
                        let body = values.value(key);
                        all_bodies.push(body);
                    }
                }

                // Expected test messages
                let expected_message1 = "<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - 'su root' failed for lonvick on /dev/pts/8";
                let expected_message2 = "<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 [exampleSDID@32473 iut=\"3\" eventSource=\"Application\" eventID=\"1011\"] Application started successfully";

                // Verify that both expected messages are present (order doesn't matter)
                assert!(
                    all_bodies.contains(&expected_message1),
                    "First message not found in received bodies"
                );
                assert!(
                    all_bodies.contains(&expected_message2),
                    "Second message not found in received bodies"
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
