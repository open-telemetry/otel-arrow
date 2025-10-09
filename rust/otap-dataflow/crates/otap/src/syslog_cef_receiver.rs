// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use self::arrow_records_encoder::ArrowRecordsBuilder;
use crate::OTAP_RECEIVER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::{error::Error, local::receiver as local};
use otap_df_telemetry::instrument::{Counter, UpDownCounter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::cell::RefCell;
use std::net::SocketAddr;
use std::rc::Rc;
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, BufReader};

/// Arrow records encoder for syslog messages
pub mod arrow_records_encoder;
/// Parser module for syslog message parsing
pub mod parser;

/// URN for the syslog cef receiver
pub const SYSLOG_CEF_RECEIVER_URN: &str = "urn:otel:syslog_cef:receiver";

const BATCH_TIMEOUT: std::time::Duration = std::time::Duration::from_millis(100); // Maximum time to wait before building an Arrow batch
const MAX_BATCH_SIZE: u16 = 100; // Maximum number of messages to build an Arrow batch

/// Protocol type for the receiver
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(dead_code)]
enum Protocol {
    /// TCP protocol
    Tcp,
    /// UDP protocol
    Udp,
}

/// config for a syslog cef receiver
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    listening_addr: SocketAddr,
    /// The protocol to use for receiving messages
    protocol: Protocol,
}

impl Config {
    #[must_use]
    #[allow(dead_code)]
    pub fn new(listening_addr: SocketAddr, protocol: Protocol) -> Self {
        Self {
            listening_addr,
            protocol,
        }
    }
}

/// Syslog CEF receiver that can listen on TCP or UDP
#[allow(dead_code)]
struct SyslogCefReceiver {
    config: Config,
    /// RFC-aligned internal telemetry for this receiver
    metrics: Rc<RefCell<MetricSet<SyslogCefReceiverMetrics>>>,
}

impl SyslogCefReceiver {
    /// Creates a new SyslogCefReceiver with the specified listening address.
    #[must_use]
    #[allow(dead_code)]
    fn with_pipeline(pipeline: PipelineContext, config: Config) -> Self {
        SyslogCefReceiver {
            config,
            metrics: Rc::new(RefCell::new(
                pipeline.register_metrics::<SyslogCefReceiverMetrics>(),
            )),
        }
    }

    /// Creates a new SyslogCefReceiver from a configuration object
    fn from_config(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let cfg: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(SyslogCefReceiver::with_pipeline(pipeline, cfg))
    }
}

/// Add the syslog receiver to the receiver factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static SYSLOG_CEF_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: SYSLOG_CEF_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::local(
            SyslogCefReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
};

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for SyslogCefReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // Start periodic telemetry collection (1s), similar to other nodes
        let timer_cancel_handle = effect_handler
            .start_periodic_telemetry(std::time::Duration::from_secs(1))
            .await?;

        match self.config.protocol {
            Protocol::Tcp => {
                let listener = effect_handler.tcp_listener(self.config.listening_addr)?;

                loop {
                    tokio::select! {
                        biased; // Prioritize control messages over data

                        // Process incoming control messages.
                        ctrl_msg = ctrl_chan.recv() => {
                            match ctrl_msg {
                                Ok(NodeControlMsg::Shutdown { .. }) => {
                                    _ = timer_cancel_handle.cancel().await;
                                    break;
                                }
                                Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                    let mut guard = self.metrics.borrow_mut();
                                    let _ = metrics_reporter.report(&mut guard);
                                }
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
                                    // Track active connections
                                    {
                                        let mut m = self.metrics.borrow_mut();
                                        m.tcp_connections_active.inc();
                                    }

                                    // Clone the effect handler so the spawned task can send messages and record telemetry.
                                    let effect_handler = effect_handler.clone();
                                    // Clone the Rc pointer (not the inner MetricSet) so all tasks update the same metrics instance.
                                    let metrics = Rc::clone(&self.metrics);

                                    // Spawn a task to handle the connection.
                                    // TODO: Use the JoinHandle for graceful shutdown when wiring stop signals.
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

                                                                // Count total received at socket level before parsing
                                                                {
                                                                    let mut m = metrics.borrow_mut();
                                                                    m.received_logs_total.inc();
                                                                }
                                                                match parser::parse(message_bytes) {
                                                                    Ok(parsed_message) => {
                                                                        arrow_records_builder.append_syslog(parsed_message);
                                                                    }
                                                                    Err(_e) => {
                                                                        // parse error => count one failed item
                                                                        let mut m = metrics.borrow_mut();
                                                                        m.received_logs_failure.inc();
                                                                    }
                                                                }
                                                            }

                                                            // Send any remaining records before closing
                                                            if arrow_records_builder.len() > 0 {
                                                                let items = u64::from(arrow_records_builder.len());
                                                                let arrow_records = arrow_records_builder
                                                                    .build()
                                                                    .expect("Failed to build Arrow records");

                                                                // send downstream
                                                                match effect_handler
                                                                    .send_message(OtapPdata::new_todo_context(arrow_records.into()))
                                                                    .await
                                                                {
                                                                    Ok(_) => {
                                                                        let mut m = metrics.borrow_mut();
                                                                        m.received_logs_success.add(items);
                                                                    }
                                                                    Err(_) => {
                                                                        let mut m = metrics.borrow_mut();
                                                                        m.received_logs_refused.add(items);
                                                                    }
                                                                }
                                                            }

                                                            // Decrement active connections on EOF
                                                            {
                                                                let mut m = metrics.borrow_mut();
                                                                m.tcp_connections_active.dec();
                                                            }
                                                            break;
                                                        }

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

                                                            // Count total received at socket level before parsing
                                                            {
                                                                let mut m = metrics.borrow_mut();
                                                                m.received_logs_total.inc();
                                                            }
                                                            match parser::parse(message_to_parse) {
                                                                Ok(parsed) => {
                                                                    arrow_records_builder.append_syslog(parsed);
                                                                }
                                                                Err(_e) => {
                                                                    // parsing error counts as one failed item
                                                                    let mut m = metrics.borrow_mut();
                                                                    m.received_logs_failure.inc();
                                                                    // Skip this message
                                                                    line_bytes.clear();
                                                                    continue;
                                                                }
                                                            };

                                                            // Clear the bytes for the next iteration
                                                            line_bytes.clear();

                                                            // Flush when batch size threshold is reached
                                                            if arrow_records_builder.len() >= MAX_BATCH_SIZE {
                                                                // Build the Arrow records to send them
                                                                let items = u64::from(arrow_records_builder.len());
                                                                let arrow_records = arrow_records_builder
                                                                    .build()
                                                                    .expect("Failed to build Arrow records");

                                                                // Reset the builder for the next batch
                                                                arrow_records_builder = ArrowRecordsBuilder::new();

                                                                // Reset the timer since we already built an arrow record batch due to size constraint
                                                                interval.reset();

                                                                match effect_handler
                                                                    .send_message(OtapPdata::new_todo_context(arrow_records.into()))
                                                                    .await
                                                                {
                                                                    Ok(_) => {
                                                                        let mut m = metrics.borrow_mut();
                                                                        m.received_logs_success.add(items);
                                                                    }
                                                                    Err(_) => {
                                                                        let mut m = metrics.borrow_mut();
                                                                        m.received_logs_refused.add(items);
                                                                    }
                                                                }
                                                            }
                                                        }

                                                        Err(_e) => {
                                                            // Send any remaining records before closing due to error
                                                            if arrow_records_builder.len() > 0 {
                                                                let items = u64::from(arrow_records_builder.len());
                                                                let arrow_records = arrow_records_builder
                                                                    .build()
                                                                    .expect("Failed to build Arrow records");

                                                                match effect_handler
                                                                    .send_message(OtapPdata::new_todo_context(arrow_records.into()))
                                                                    .await
                                                                {
                                                                    Ok(_) => {
                                                                        let mut m = metrics.borrow_mut();
                                                                        m.received_logs_success.add(items);
                                                                    }
                                                                    Err(_) => {
                                                                        let mut m = metrics.borrow_mut();
                                                                        m.received_logs_refused.add(items);
                                                                    }
                                                                }
                                                            }

                                                            // Decrement active connections on read error
                                                            {
                                                                let mut m = metrics.borrow_mut();
                                                                m.tcp_connections_active.dec();
                                                            }
                                                            break; // ToDo: Handle read error properly
                                                        }
                                                    }
                                                }

                                                // Handle timeout - send any accumulated records
                                                _ = interval.tick() => {
                                                    if arrow_records_builder.len() > 0 {
                                                        // Build the Arrow records and send them
                                                        let items = u64::from(arrow_records_builder.len());
                                                        let arrow_records = arrow_records_builder
                                                            .build()
                                                            .expect("Failed to build Arrow records");

                                                        // Reset the builder for the next batch
                                                        arrow_records_builder = ArrowRecordsBuilder::new();

                                                        match effect_handler
                                                            .send_message(OtapPdata::new_todo_context(arrow_records.into()))
                                                            .await
                                                        {
                                                            Ok(_) => {
                                                                let mut m = metrics.borrow_mut();
                                                                m.received_logs_success.add(items);
                                                            }
                                                            Err(_) => {
                                                                let mut m = metrics.borrow_mut();
                                                                m.received_logs_refused.add(items);
                                                            }
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    });
                                }

                                Err(e) => {
                                    return Err(Error::ReceiverError {
                                        receiver: effect_handler.receiver_id(),
                                        error: e.to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }

            Protocol::Udp => {
                let socket = effect_handler.udp_socket(self.config.listening_addr)?;
                let mut buf = [0u8; 1024]; // ToDo: Find out the maximum allowed size for syslog messages
                let mut arrow_records_builder = ArrowRecordsBuilder::new();

                let start = tokio::time::Instant::now() + BATCH_TIMEOUT;
                let mut interval = tokio::time::interval_at(start, BATCH_TIMEOUT);

                loop {
                    tokio::select! {
                        biased; // Prioritize control messages over data

                        // Process incoming control messages.
                        ctrl_msg = ctrl_chan.recv() => {
                            match ctrl_msg {
                                Ok(NodeControlMsg::Shutdown { .. }) => {
                                    _ = timer_cancel_handle.cancel().await;
                                    // ToDo: Add proper deadline function
                                    break;
                                }
                                Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                    let mut guard = self.metrics.borrow_mut();
                                    let _ = metrics_reporter.report(&mut guard);
                                }
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
                                    // Count total received at socket level before parsing
                                    {
                                        let mut m = self.metrics.borrow_mut();
                                        m.received_logs_total.inc();
                                    }
                                    match parser::parse(&buf[..n]) {
                                        Ok(parsed) => {
                                            arrow_records_builder.append_syslog(parsed);
                                        }
                                        Err(_e) => {
                                            // parsing failed => one failed item
                                            let mut m = self.metrics.borrow_mut();
                                            m.received_logs_failure.inc();
                                            continue; // Skip this message
                                        }
                                    };

                                    if arrow_records_builder.len() >= MAX_BATCH_SIZE {
                                        // Build the Arrow records to send them
                                        let items = u64::from(arrow_records_builder.len());
                                        let arrow_records = arrow_records_builder
                                            .build()
                                            .expect("Failed to build Arrow records");

                                        // Reset the builder for the next batch
                                        arrow_records_builder = ArrowRecordsBuilder::new();

                                        // Reset the timer since we already built an arrow record batch due to size constraint
                                        interval.reset();

                                        match effect_handler
                                            .send_message(OtapPdata::new_todo_context(arrow_records.into()))
                                            .await
                                        {
                                            Ok(_) => {
                                                let mut m = self.metrics.borrow_mut();
                                                m.received_logs_success.add(items);
                                            }
                                            Err(_) => {
                                                let mut m = self.metrics.borrow_mut();
                                                m.received_logs_refused.add(items);
                                            }
                                        }
                                    }
                                }

                                Err(e) => {
                                    return Err(Error::ReceiverError {
                                        receiver: effect_handler.receiver_id(),
                                        error: e.to_string(),
                                    });
                                }
                            }
                        },

                        _ = interval.tick() => {
                            // Check if we have any records to send
                            if arrow_records_builder.len() > 0 {
                                // Build the Arrow records and send them
                                let items = u64::from(arrow_records_builder.len());
                                let arrow_records = arrow_records_builder
                                    .build()
                                    .expect("Failed to build Arrow records");

                                // Reset the builder for the next batch
                                arrow_records_builder = ArrowRecordsBuilder::new();

                                match effect_handler
                                    .send_message(OtapPdata::new_todo_context(arrow_records.into()))
                                    .await
                                {
                                    Ok(_) => {
                                        let mut m = self.metrics.borrow_mut();
                                        m.received_logs_success.add(items);
                                    }
                                    Err(_) => {
                                        let mut m = self.metrics.borrow_mut();
                                        m.received_logs_refused.add(items);
                                    }
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

/// RFC-aligned metrics for Syslog CEF receiver.
///
/// Key fields follow the component universal telemetry RFC for receivers:
/// received.items.<signal>.<outcome>
#[metric_set(name = "syslog_cef.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct SyslogCefReceiverMetrics {
    /// Number of log records successfully forwarded downstream
    #[metric(unit = "{item}")]
    pub received_logs_success: Counter<u64>,

    /// Number of log records that failed to be received (parse errors, etc.)
    #[metric(unit = "{item}")]
    pub received_logs_failure: Counter<u64>,

    /// Number of log records refused by downstream (backpressure/unavailable)
    #[metric(unit = "{item}")]
    pub received_logs_refused: Counter<u64>,

    /// Total number of log records observed at the socket before parsing
    #[metric(unit = "{item}")]
    pub received_logs_total: Counter<u64>,

    /// Number of active TCP connections
    #[metric(unit = "{conn}")]
    pub tcp_connections_active: UpDownCounter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pdata::OtapPayload;
    use arrow::array::Array;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpStream, UdpSocket};
    use tokio::time::{Duration, timeout};

    // Test-only constructor moved here to keep all test code at the end of the file.
    impl SyslogCefReceiver {
        #[allow(dead_code)]
        fn new(config: Config) -> Self {
            let registry = otap_df_telemetry::registry::MetricsRegistryHandle::new();
            let controller = otap_df_engine::context::ControllerContext::new(registry);
            let pipeline = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);
            SyslogCefReceiver::with_pipeline(pipeline, config)
        }
    }

    /// Test closure that simulates a typical UDP syslog receiver scenario.
    fn udp_scenario(
        listening_addr: SocketAddr,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
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
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
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
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
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
                    .expect("No first message received")
                    .payload();

                // Extract arrow_records for further validation
                let OtapPayload::OtapArrowRecords(arrow_records) = message1_received else {
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
                    .expect("No first message received")
                    .payload();

                // Extract arrow_records for further validation
                let OtapPayload::OtapArrowRecords(arrow_records) = message1_received else {
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
                            let OtapPayload::OtapArrowRecords(arrow_records) = message.payload()
                            else {
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

        let config = Config::new(listening_addr, Protocol::Udp);
        // create our UDP receiver
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(SYSLOG_CEF_RECEIVER_URN));
        let receiver = ReceiverWrapper::local(
            SyslogCefReceiver::new(config),
            test_node(test_runtime.config().name.clone()),
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

        let config = Config::new(listening_addr, Protocol::Tcp);
        // create our TCP receiver - we need to modify the receiver to support TCP
        let receiver = SyslogCefReceiver::new(config);

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(SYSLOG_CEF_RECEIVER_URN));
        let receiver_wrapper = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

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

        let config = Config::new(listening_addr, Protocol::Tcp);
        // create our TCP receiver
        let receiver = SyslogCefReceiver::new(config);

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(SYSLOG_CEF_RECEIVER_URN));
        let receiver_wrapper = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver_wrapper)
            .run_test(tcp_incomplete_scenario(listening_addr))
            .run_validation(tcp_incomplete_validation_procedure());
    }
}

#[cfg(test)]
mod telemetry_tests {
    use super::*;
    use otap_df_channel::mpsc;
    use otap_df_engine::control::{NodeControlMsg, pipeline_ctrl_msg_channel};
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::local::receiver::Receiver;
    use otap_df_engine::local::receiver::{ControlChannel, EffectHandler};
    use otap_df_engine::message::Receiver as EngineReceiver;
    use otap_df_engine::testing::test_node;
    use otap_df_telemetry::MetricsSystem;
    use portpicker::pick_unused_port;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use tokio::net::UdpSocket;
    use tokio::time::{Duration, sleep};

    fn collect_syslog_metrics_map(
        registry: &otap_df_telemetry::registry::MetricsRegistryHandle,
    ) -> HashMap<&'static str, u64> {
        let mut out = HashMap::new();
        registry.visit_current_metrics(|desc, _attrs, iter| {
            if desc.name == "syslog_cef.receiver.metrics" {
                for (field, value) in iter {
                    let _ = out.insert(field.name, value);
                }
            }
        });
        out
    }

    // Telemetry helpers mirroring signal_type_router tests
    mod telemetry_helpers {
        use super::*;
        use otap_df_telemetry::config::Config as TelemetryConfig;
        use otap_df_telemetry::registry::MetricsRegistryHandle;
        use otap_df_telemetry::reporter::MetricsReporter;
        use tokio::task::JoinHandle;

        pub fn start_telemetry() -> (MetricsRegistryHandle, MetricsReporter, JoinHandle<()>) {
            let telemetry = MetricsSystem::new(TelemetryConfig::default());
            let registry = telemetry.registry();
            let reporter = telemetry.reporter();
            let collector_task = tokio::task::spawn_local(async move {
                let _ = telemetry.run_collection_loop().await;
            });
            (registry, reporter, collector_task)
        }

        pub fn stop_telemetry(reporter: MetricsReporter, collector_task: JoinHandle<()>) {
            drop(reporter);
            collector_task.abort();
        }
    }

    #[test]
    fn test_udp_metrics_success_and_failure_collect() {
        use otap_df_engine::testing::setup_test_runtime;
        use telemetry_helpers::{start_telemetry, stop_telemetry};

        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            // Telemetry setup
            let (registry, reporter, collector_task) = start_telemetry();

            // Pipeline context
            let controller = otap_df_engine::context::ControllerContext::new(registry.clone());
            let pipeline = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);

            // UDP config + receiver instance (with pipeline-bound metrics)
            let port = pick_unused_port().expect("No free UDP port");
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
            let cfg_json = serde_json::json!({
                "listening_addr": addr,
                "protocol": "udp"
            });
            let receiver = SyslogCefReceiver::from_config(pipeline, &cfg_json).expect("config");

            // Out channel (keep receiver alive to avoid refused)
            let (out_tx, mut _out_rx) = mpsc::Channel::new(8);
            let mut senders = HashMap::new();
            let _ = senders.insert("".into(), LocalSender::MpscSender(out_tx));

            // Pipeline control for effect handler
            let (pipe_tx, _pipe_rx) = pipeline_ctrl_msg_channel(10);
            let eh = EffectHandler::new(test_node("syslog_udp_ok"), senders, None, pipe_tx);

            // Control channel for receiver
            let (ctrl_tx, ctrl_rx) = mpsc::Channel::new(16);
            let ctrl_rx = EngineReceiver::Local(LocalReceiver::MpscReceiver(ctrl_rx));
            let ctrl_chan = ControlChannel::new(ctrl_rx);

            // Start receiver inside LocalSet
            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            // Give it a moment to bind
            sleep(Duration::from_millis(50)).await;

            // Send one valid CEF and one invalid message
            let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let _ = sock
                .send_to(
                    b"CEF:0|Security|threatmanager|1.0|100|worm stopped|10|src=10.0.0.1",
                    addr,
                )
                .await
                .unwrap();
            // Send a malformed CEF message to ensure parsing failure is recorded
            let _ = sock.send_to(b"CEF:1|", addr).await.unwrap();

            // Wait for flush interval
            sleep(Duration::from_millis(150)).await;

            // Trigger telemetry snapshot
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            sleep(Duration::from_millis(50)).await;

            // Inspect metrics
            let map = collect_syslog_metrics_map(&registry);
            let get = |k: &str| map.get(k).copied().unwrap_or(0);
            assert_eq!(get("received.logs.success"), 1, "success == 1");
            assert_eq!(get("received.logs.failure"), 1, "failure == 1");
            assert_eq!(get("received.logs.refused"), 0, "refused == 0");
            assert_eq!(
                get("received.logs.total"),
                2,
                "total == 2 (one valid + one invalid)"
            );
            assert_eq!(
                get("tcp.connections.active"),
                0,
                "active tcp conns == 0 for UDP"
            );

            // Shutdown receiver to allow LocalSet to complete
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(50),
                reason: "test done".into(),
            });
            let _ = handle.await;

            // Stop telemetry collector
            stop_telemetry(reporter, collector_task);
        }));
    }

    #[test]
    fn test_udp_metrics_refused_on_closed_downstream() {
        use otap_df_engine::testing::setup_test_runtime;
        use telemetry_helpers::{start_telemetry, stop_telemetry};

        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            // Telemetry system
            let (registry, reporter, collector_task) = start_telemetry();

            // Pipeline context
            let controller = otap_df_engine::context::ControllerContext::new(registry.clone());
            let pipeline = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);

            // UDP config
            let port = pick_unused_port().expect("No free UDP port");
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
            let cfg_json = serde_json::json!({
                "listening_addr": addr,
                "protocol": "udp"
            });
            let receiver = SyslogCefReceiver::from_config(pipeline, &cfg_json).expect("config");

            // Out channel: drop receiver side to force send error => refused
            let (out_tx, out_rx) = mpsc::Channel::new(1);
            drop(out_rx); // downstream closed
            let mut senders = HashMap::new();
            let _ = senders.insert("".into(), LocalSender::MpscSender(out_tx));

            // Pipeline control for effect handler
            let (pipe_tx, _pipe_rx) = pipeline_ctrl_msg_channel(10);
            let eh = EffectHandler::new(test_node("syslog_udp_refused"), senders, None, pipe_tx);

            // Control channel
            let (ctrl_tx, ctrl_rx) = mpsc::Channel::new(16);
            let ctrl_rx = EngineReceiver::Local(LocalReceiver::MpscReceiver(ctrl_rx));
            let ctrl_chan = ControlChannel::new(ctrl_rx);

            // Start receiver
            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            // Give it a moment to bind
            sleep(Duration::from_millis(50)).await;

            // Send a valid message that will be flushed and refused downstream
            let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let _ = sock
                .send_to(
                    b"CEF:0|Security|threatmanager|1.0|100|worm stopped|10|src=10.0.0.1",
                    addr,
                )
                .await
                .unwrap();

            // Wait for flush interval
            sleep(Duration::from_millis(150)).await;

            // Trigger telemetry snapshot
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            sleep(Duration::from_millis(50)).await;

            // Inspect metrics
            let map = collect_syslog_metrics_map(&registry);
            let get = |k: &str| map.get(k).copied().unwrap_or(0);
            assert!(get("received.logs.refused") >= 1, "refused >= 1");
            assert!(get("received.logs.total") >= 1, "total >= 1");
            assert!(
                get("tcp.connections.active") == 0,
                "active tcp conns == 0 for UDP"
            );

            // Shutdown receiver
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Duration::from_millis(50),
                reason: "test done".into(),
            });
            let _ = handle.await;

            // Stop telemetry collector
            stop_telemetry(reporter, collector_task);
        }));
    }
}
