// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use self::arrow_records_encoder::ArrowRecordsBuilder;
use crate::OTAP_RECEIVER_FACTORIES;
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{MessageSourceLocalEffectHandlerExtension, ReceiverFactory};
use otap_df_engine::{
    error::{Error, ReceiverErrorKind, format_error_sources},
    local::receiver as local,
};
use otap_df_telemetry::instrument::{Counter, UpDownCounter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::cell::{Cell, RefCell};
use std::net::SocketAddr;
use std::num::{NonZeroU16, NonZeroU64};
use std::rc::Rc;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, BufReader};

#[cfg(feature = "experimental-tls")]
use crate::tls_utils::{accept_tls_connection, build_tls_acceptor};
#[cfg(feature = "experimental-tls")]
use otap_df_config::tls::TlsServerConfig;
#[cfg(feature = "experimental-tls")]
use otap_df_telemetry::otel_debug;

/// Arrow records encoder for syslog messages
pub mod arrow_records_encoder;
/// Parser module for syslog message parsing
pub mod parser;

/// URN for the syslog cef receiver
pub const SYSLOG_CEF_RECEIVER_URN: &str = "urn:otel:syslog_cef:receiver";

/// Default maximum time to wait before flushing an Arrow batch.
const DEFAULT_FLUSH_TIMEOUT: Duration = Duration::from_millis(100);
/// Default maximum number of messages to build an Arrow batch.
const DEFAULT_MAX_BATCH_SIZE: u16 = 100;

/// Maximum time to wait for spawned TCP tasks to drain during shutdown.
const MAX_TASK_DRAIN_WAIT: Duration = Duration::from_secs(1);

/// TCP-specific settings for the syslog CEF receiver.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TcpConfig {
    /// The address to listen on for TCP connections.
    listening_addr: SocketAddr,
    /// TLS configuration for secure TCP connections (Syslog over TLS, RFC 5425).
    ///
    /// When configured, TCP connections will require TLS.
    #[cfg(feature = "experimental-tls")]
    tls: Option<TlsServerConfig>,
}

/// UDP-specific settings for the syslog CEF receiver.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UdpConfig {
    /// The address to listen on for UDP datagrams.
    listening_addr: SocketAddr,
}

/// Protocol configuration for the syslog CEF receiver.
///
/// Exactly one protocol variant must be specified.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
#[allow(clippy::large_enum_variant)] // This enum is only used for configuration, so the size is not a concern.
enum Protocol {
    /// TCP protocol settings.
    Tcp(TcpConfig),
    /// UDP protocol settings.
    Udp(UdpConfig),
}

/// Optional batching configuration for the syslog CEF receiver.
///
/// Controls how incoming log records are accumulated into Arrow batches
/// before being forwarded downstream. Reducing these values can limit
/// the scope of data loss for in-memory records that have not yet been
/// sent to the next node.
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct BatchConfig {
    /// Maximum time in milliseconds to wait before flushing an Arrow batch.
    /// Defaults to 100 ms when not specified. Must be greater than zero.
    #[serde(default)]
    flush_timeout_ms: Option<NonZeroU64>,
    /// Maximum number of messages to accumulate before building an Arrow batch.
    /// Defaults to 100 when not specified. Must be greater than zero.
    #[serde(default)]
    max_size: Option<NonZeroU16>,
}

/// Config for a syslog cef receiver
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    /// Protocol-specific configuration.
    protocol: Protocol,
    /// Optional batching configuration.
    /// When omitted, sensible defaults are used.
    #[serde(default)]
    batch: Option<BatchConfig>,
}

impl Config {
    /// Creates a new Config for TCP.
    #[must_use]
    #[allow(dead_code)]
    const fn new_tcp(listening_addr: SocketAddr) -> Self {
        Self {
            protocol: Protocol::Tcp(TcpConfig {
                listening_addr,
                #[cfg(feature = "experimental-tls")]
                tls: None,
            }),
            batch: None,
        }
    }

    /// Creates a new Config for UDP.
    #[must_use]
    #[allow(dead_code)]
    const fn new_udp(listening_addr: SocketAddr) -> Self {
        Self {
            protocol: Protocol::Udp(UdpConfig { listening_addr }),
            batch: None,
        }
    }

    /// Returns the effective flush timeout, using the configured value or the default.
    fn flush_timeout(&self) -> Duration {
        self.batch
            .as_ref()
            .and_then(|b| b.flush_timeout_ms)
            .map(|ms| Duration::from_millis(ms.get()))
            .unwrap_or(DEFAULT_FLUSH_TIMEOUT)
    }

    /// Returns the effective max batch size, using the configured value or the default.
    fn max_batch_size(&self) -> u16 {
        self.batch
            .as_ref()
            .and_then(|b| b.max_size)
            .map(|s| s.get())
            .unwrap_or(DEFAULT_MAX_BATCH_SIZE)
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
    /// Construct with pipeline context registering metrics
    fn with_pipeline(pipeline: PipelineContext, config: Config) -> Self {
        let metrics = pipeline.register_metrics::<SyslogCefReceiverMetrics>();
        SyslogCefReceiver {
            config,
            metrics: Rc::new(RefCell::new(metrics)),
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
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for SyslogCefReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // Start periodic telemetry collection (1s), similar to other nodes
        let timer_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        match &self.config.protocol {
            Protocol::Tcp(tcp_config) => {
                otel_info!(
                    "receiver.start",
                    protocol = "TCP",
                    listening_addr = tcp_config.listening_addr.to_string(),
                    message = "Starting Syslog/CEF receiver"
                );

                let listener = effect_handler.tcp_listener(tcp_config.listening_addr)?;

                // Build TLS acceptor if TLS is configured
                #[cfg(feature = "experimental-tls")]
                let maybe_tls_acceptor = build_tls_acceptor(tcp_config.tls.as_ref())
                    .await
                    .map_err(|e| Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Configuration,
                        error: format!("Failed to configure TLS: {}", e),
                        source_detail: format_error_sources(&e),
                    })?;

                // Extract handshake timeout from TLS config (if present)
                #[cfg(feature = "experimental-tls")]
                let maybe_handshake_timeout = tcp_config
                    .tls
                    .as_ref()
                    .and_then(|tls| tls.handshake_timeout);

                #[cfg(feature = "experimental-tls")]
                if maybe_tls_acceptor.is_some() {
                    otel_info!(
                        "receiver.tls_enabled",
                        message = "TLS enabled for Syslog/CEF TCP receiver"
                    );
                }

                // Resolve effective batching settings from config
                let flush_timeout = self.config.flush_timeout();
                let max_batch_size = self.config.max_batch_size();

                // Flag to signal spawned connection tasks to flush and exit on shutdown
                let shutdown_flag = Rc::new(Cell::new(false));
                // Counter to track active connection tasks for graceful shutdown
                let active_task_count = Rc::new(Cell::new(0usize));

                loop {
                    tokio::select! {
                        biased; // Prioritize control messages over data

                        // Process incoming control messages.
                        ctrl_msg = ctrl_chan.recv() => {
                            match ctrl_msg {
                                Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                                    let _ = timer_cancel_handle.cancel().await;
                                    shutdown_flag.set(true); // Signal all connection tasks to flush and exit

                                    // Wait for active tasks to finish flushing.
                                    // Use 90% of remaining time (keeping 10% buffer for cleanup),
                                    // capped at MAX_TASK_DRAIN_WAIT.
                                    let time_until_deadline = deadline.saturating_duration_since(std::time::Instant::now());
                                    let drain_wait = std::cmp::min(time_until_deadline * 9 / 10, MAX_TASK_DRAIN_WAIT);
                                    let drain_result = tokio::time::timeout(drain_wait, async {
                                        while active_task_count.get() > 0 {
                                            tokio::task::yield_now().await;
                                        }
                                    }).await;

                                    if drain_result.is_err() {
                                        otel_warn!(
                                            "receiver.shutdown.drain_timeout",
                                            active_tasks = active_task_count.get(),
                                            message = "Shutdown drain timeout expired with tasks still active"
                                        );
                                    }

                                    let snapshot = self.metrics.borrow().snapshot();
                                    return Ok(TerminalState::new(deadline, [snapshot]));
                                }
                                Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                    let mut m = self.metrics.borrow_mut();
                                    let _ = metrics_reporter.report(&mut m);
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
                                Ok((socket, peer_addr)) => {
                                    // Clone the effect handler so the spawned task can send messages.
                                    let effect_handler = effect_handler.clone();
                                    let metrics = self.metrics.clone();

                                    // Clone TLS acceptor for the spawned task
                                    #[cfg(feature = "experimental-tls")]
                                    let tls_acceptor = maybe_tls_acceptor.clone();
                                    #[cfg(feature = "experimental-tls")]
                                    let tls_handshake_timeout = maybe_handshake_timeout;

                                    // Clone shutdown flag for the spawned task
                                    let task_shutdown_flag = shutdown_flag.clone();
                                    // Clone active task counter for the spawned task
                                    let task_active_count = active_task_count.clone();

                                    // Spawn a task to handle the connection.
                                    // ToDo should this be abstracted and exposed a method in the effect handler?
                                    _ = tokio::task::spawn_local(async move {
                                        // If already shutting down, exit immediately (nothing to process yet)
                                        if task_shutdown_flag.get() {
                                            return;
                                        }

                                        // Now we're committed to handling this connection
                                        metrics.borrow_mut().tcp_connections_active.inc();
                                        task_active_count.set(task_active_count.get() + 1);

                                        // Perform TLS handshake if configured, creating a unified reader type
                                        #[cfg(feature = "experimental-tls")]
                                        let mut reader: Box<dyn tokio::io::AsyncBufRead + Unpin> = if let Some(acceptor) = tls_acceptor {
                                            // Use configured timeout or fall back to 10 seconds (the serde default)
                                            let timeout = tls_handshake_timeout
                                                .unwrap_or(Duration::from_secs(10));
                                            match accept_tls_connection(socket, &acceptor, timeout).await {
                                                Ok(tls_stream) => {
                                                    otel_debug!(
                                                        "tls.handshake.success",
                                                        peer = %peer_addr,
                                                        message = "TLS handshake completed"
                                                    );
                                                    Box::new(BufReader::new(tls_stream))
                                                }
                                                Err(e) => {
                                                    otel_warn!(
                                                        "tls.handshake.failed",
                                                        peer = %peer_addr,
                                                        error = %e,
                                                        message = "TLS handshake failed, closing connection"
                                                    );
                                                    metrics.borrow_mut().tcp_connections_active.dec();
                                                    metrics.borrow_mut().tls_handshake_failures.inc();
                                                    task_active_count.set(task_active_count.get() - 1);
                                                    return;
                                                }
                                            }
                                        } else {
                                            Box::new(BufReader::new(socket))
                                        };

                                        #[cfg(not(feature = "experimental-tls"))]
                                        let mut reader = BufReader::new(socket);

                                        // Suppress unused variable warning when TLS is disabled
                                        let _ = peer_addr;

                                        let mut line_bytes = Vec::new();

                                        let mut arrow_records_builder = ArrowRecordsBuilder::new();

                                        let start = tokio::time::Instant::now() + flush_timeout;
                                        let mut interval = tokio::time::interval_at(start, flush_timeout);

                                        loop {
                                            // Check for shutdown signal (simple bool check - very cheap)
                                            if task_shutdown_flag.get() {
                                                if arrow_records_builder.len() > 0 {
                                                    let items = u64::from(arrow_records_builder.len());
                                                    let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");
                                                    let res = effect_handler.try_send_message_with_source_node(
                                                        OtapPdata::new_todo_context(arrow_records.into())
                                                    );
                                                    let mut m = metrics.borrow_mut();
                                                    match &res {
                                                        Ok(_) => m.received_logs_forwarded.add(items),
                                                        Err(_) => m.received_logs_forward_failed.add(items),
                                                    }
                                                }
                                                metrics.borrow_mut().tcp_connections_active.dec();
                                                task_active_count.set(task_active_count.get() - 1);
                                                break;
                                            }

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
                                                                metrics.borrow_mut().received_logs_total.inc();

                                                                match parser::parse(message_bytes) {
                                                                    Ok(parsed_message) => {
                                                                        arrow_records_builder.append_syslog(parsed_message);
                                                                    }
                                                                    Err(_e) => {
                                                                        // parse error => count one failed item
                                                                        metrics.borrow_mut().received_logs_invalid.inc();
                                                                    }
                                                                }
                                                            }

                                                            // Send any remaining records before closing
                                                            if arrow_records_builder.len() > 0 {
                                                                let items = u64::from(arrow_records_builder.len());
                                                                let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");
                                                                let res = effect_handler.send_message_with_source_node(OtapPdata::new_todo_context(arrow_records.into())).await;

                                                                {
                                                                    let mut m = metrics.borrow_mut();
                                                                    match &res {
                                                                        Ok(_) => m.received_logs_forwarded.add(items),
                                                                        Err(_) => m.received_logs_forward_failed.add(items),
                                                                    }
                                                                }
                                                            }

                                                            // Decrement active connections on EOF
                                                            metrics.borrow_mut().tcp_connections_active.dec();
                                                            task_active_count.set(task_active_count.get() - 1);
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
                                                            metrics.borrow_mut().received_logs_total.inc();

                                                            match parser::parse(message_to_parse) {
                                                                Ok(parsed) => {
                                                                    arrow_records_builder.append_syslog(parsed);
                                                                }
                                                                Err(_e) => {
                                                                    // parsing error counts as one failed item
                                                                    metrics.borrow_mut().received_logs_invalid.inc();
                                                                    // Skip this message
                                                                    line_bytes.clear();
                                                                    continue;
                                                                }
                                                            };

                                                            // Clear the bytes for the next iteration
                                                            line_bytes.clear();

                                                            if arrow_records_builder.len() >= max_batch_size {
                                                                let items = u64::from(arrow_records_builder.len());

                                                                // Build the Arrow records to send them
                                                                let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");

                                                                // Reset the builder for the next batch
                                                                arrow_records_builder = ArrowRecordsBuilder::new();

                                                                // Reset the timer since we already built an arrow record batch due to size constraint
                                                                interval.reset();

                                                                let res = effect_handler.send_message_with_source_node(OtapPdata::new_todo_context(arrow_records.into())).await;
                                                                {
                                                                    let mut m = metrics.borrow_mut();
                                                                    match &res {
                                                                        Ok(_) => m.received_logs_forwarded.add(items),
                                                                        Err(_) => m.received_logs_forward_failed.add(items),
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Err(_e) => {
                                                            // Send any remaining records before closing due to error
                                                            if arrow_records_builder.len() > 0 {
                                                                let items = u64::from(arrow_records_builder.len());
                                                                let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");
                                                                let res = effect_handler.send_message_with_source_node(OtapPdata::new_todo_context(arrow_records.into())).await;

                                                                {
                                                                    let mut m = metrics.borrow_mut();
                                                                    match &res {
                                                                        Ok(_) => m.received_logs_forwarded.add(items),
                                                                        Err(_) => m.received_logs_forward_failed.add(items),
                                                                    }
                                                                }
                                                            }

                                                            // Decrement active connections on read error
                                                            metrics.borrow_mut().tcp_connections_active.dec();
                                                            task_active_count.set(task_active_count.get() - 1);
                                                            break; // ToDo: Handle read error properly
                                                        }
                                                    }
                                                }

                                                // Handle timeout - send any accumulated records
                                                _ = interval.tick() => {
                                                    if arrow_records_builder.len() > 0 {
                                                        // Build the Arrow records and send them
                                                        let items = u64::from(arrow_records_builder.len());
                                                        let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");

                                                        // Reset the builder for the next batch
                                                        arrow_records_builder = ArrowRecordsBuilder::new();

                                                        let res = effect_handler.send_message_with_source_node(OtapPdata::new_todo_context(arrow_records.into())).await;
                                                        {
                                                            let mut m = metrics.borrow_mut();
                                                            match &res {
                                                                Ok(_) => m.received_logs_forwarded.add(items),
                                                                Err(_) => m.received_logs_forward_failed.add(items),
                                                            }
                                                        }
                                                    }
                                                },
                                            }
                                        }
                                    });
                                }
                                Err(e) => {
                                    let source_detail = format_error_sources(&e);
                                    return Err(Error::ReceiverError {
                                        receiver: effect_handler.receiver_id(),
                                        kind: ReceiverErrorKind::Transport,
                                        error: e.to_string(),
                                        source_detail,
                                    });
                                }
                            }
                        }
                    }
                }
            }

            Protocol::Udp(udp_config) => {
                otel_info!(
                    "receiver.start",
                    protocol = "UDP",
                    listening_addr = udp_config.listening_addr.to_string(),
                    message = "Starting Syslog/CEF receiver"
                );

                let socket = effect_handler.udp_socket(udp_config.listening_addr)?;
                let mut buf = [0u8; 1024]; // ToDo: Find out the maximum allowed size for syslog messages
                let mut arrow_records_builder = ArrowRecordsBuilder::new();

                let flush_timeout = self.config.flush_timeout();
                let max_batch_size = self.config.max_batch_size();

                let start = tokio::time::Instant::now() + flush_timeout;
                let mut interval = tokio::time::interval_at(start, flush_timeout);

                loop {
                    tokio::select! {
                        biased; // Prioritize control messages over data

                        // Process incoming control messages.
                        ctrl_msg = ctrl_chan.recv() => {
                            match ctrl_msg {
                                Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                                    let _ = timer_cancel_handle.cancel().await;

                                    // Flush any remaining records before shutdown
                                    if arrow_records_builder.len() > 0 {
                                        let items = u64::from(arrow_records_builder.len());
                                        let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");
                                        let res = effect_handler.try_send_message_with_source_node(
                                            OtapPdata::new_todo_context(arrow_records.into())
                                        );
                                        let mut m = self.metrics.borrow_mut();
                                        match &res {
                                            Ok(_) => m.received_logs_forwarded.add(items),
                                            Err(_) => m.received_logs_forward_failed.add(items),
                                        }
                                    }

                                    let snapshot = self.metrics.borrow().snapshot();
                                    return Ok(TerminalState::new(deadline, [snapshot]));
                                }
                                Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                    let mut m = self.metrics.borrow_mut();
                                    let _ = metrics_reporter.report(&mut m);
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
                                    self.metrics.borrow_mut().received_logs_total.inc();

                                    let parsed_message = match parser::parse(&buf[..n]) {
                                        Ok(parsed) => parsed,
                                        Err(_e) => {
                                            // ToDo: Handle parsing error (log, emit metrics, etc.)
                                            self.metrics.borrow_mut().received_logs_invalid.inc();
                                            continue; // Skip this message
                                        }
                                    };

                                    arrow_records_builder.append_syslog(parsed_message);

                                    if arrow_records_builder.len() >= max_batch_size {
                                        // Build the Arrow records to send them
                                        let items = u64::from(arrow_records_builder.len());
                                        let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");

                                        // Reset the builder for the next batch
                                        arrow_records_builder = ArrowRecordsBuilder::new();

                                        // Reset the timer since we already built an arrow record batch due to size constraint
                                        interval.reset();

                                        let res = effect_handler.send_message_with_source_node(OtapPdata::new_todo_context(arrow_records.into())).await;
                                        {
                                            let mut m = self.metrics.borrow_mut();
                                            match &res {
                                                Ok(_) => m.received_logs_forwarded.add(items),
                                                Err(_) => m.received_logs_forward_failed.add(items),
                                            }
                                        }
                                        // Do not propagate downstream send errors; keep running
                                        // so that telemetry can still be collected (tests expect refused
                                        // to be counted and reported). We already incremented
                                        // `received_logs_forward_failed` above.
                                        if res.is_err() {
                                            // swallow error
                                        }
                                    }
                                }
                                Err(e) => {
                                    let source_detail = format_error_sources(&e);
                                    return Err(Error::ReceiverError {
                                        receiver: effect_handler.receiver_id(),
                                        kind: ReceiverErrorKind::Transport,
                                        error: e.to_string(),
                                        source_detail,
                                    });
                                }
                            }
                        },

                        _ = interval.tick() => {
                            // Check if we have any records to send
                            if arrow_records_builder.len() > 0 {
                                // Build the Arrow records and send them
                                let items = u64::from(arrow_records_builder.len());
                                let arrow_records = arrow_records_builder.build().expect("Failed to build Arrow records");

                                // Reset the builder for the next batch
                                arrow_records_builder = ArrowRecordsBuilder::new();

                                let res = effect_handler.send_message_with_source_node(OtapPdata::new_todo_context(arrow_records.into())).await;
                                {
                                    let mut m = self.metrics.borrow_mut();
                                    match &res {
                                        Ok(_) => m.received_logs_forwarded.add(items),
                                        Err(_) => m.received_logs_forward_failed.add(items),
                                    }
                                }
                                // Do not propagate downstream send errors; keep running
                                // so that telemetry can still be collected and reported.
                                if res.is_err() {
                                    // swallow error (already counted above)
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

/// RFC-aligned metrics for Syslog CEF receiver.
#[metric_set(name = "syslog_cef.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct SyslogCefReceiverMetrics {
    /// Number of log records successfully forwarded downstream
    #[metric(unit = "{item}")]
    pub received_logs_forwarded: Counter<u64>,

    /// Number of log records that failed to be parsed
    #[metric(unit = "{item}")]
    pub received_logs_invalid: Counter<u64>,

    /// Number of log records refused by downstream (backpressure/unavailable)
    #[metric(unit = "{item}")]
    pub received_logs_forward_failed: Counter<u64>,

    /// Total number of log records observed at the socket before parsing
    #[metric(unit = "{item}")]
    pub received_logs_total: Counter<u64>,

    /// Number of active TCP connections
    #[metric(unit = "{conn}")]
    pub tcp_connections_active: UpDownCounter<u64>,

    /// Number of TLS handshake failures
    #[cfg(feature = "experimental-tls")]
    #[metric(unit = "{error}")]
    pub tls_handshake_failures: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_pdata::OtapPayload;

    // Test-only constructor, not compiled in production
    impl SyslogCefReceiver {
        #[allow(dead_code)]
        fn new(config: Config) -> Self {
            // Create a standalone metrics set for tests (not bound to a pipeline)
            let telemetry_registry = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
            let metric_set = telemetry_registry.register_metric_set::<SyslogCefReceiverMetrics>(
                otap_df_telemetry::testing::EmptyAttributes(),
            );
            SyslogCefReceiver {
                config,
                metrics: Rc::new(RefCell::new(metric_set)),
            }
        }
    }
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
    use std::time::Instant;
    use tokio::io::AsyncWriteExt;
    use tokio::net::{TcpStream, UdpSocket};
    use tokio::time::{Duration, timeout};

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
                ctx.send_shutdown(Instant::now(), "Test")
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
                ctx.send_shutdown(Instant::now(), "Test")
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
                ctx.send_shutdown(Instant::now(), "Test")
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
                use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

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
                // Note: body column is not present when all messages are fully parsed
                // (all data is in attributes, no need for body)
                assert!(
                    !column_names.contains(&"body"),
                    "Logs record batch should NOT contain 'body' column when all messages are fully parsed"
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
            })
        }
    }

    /// Validation closure that checks the received messages for TCP test.
    fn tcp_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

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
                // Note: body column is not present when all messages are fully parsed
                // (all data is in attributes, no need for body)
                assert!(
                    !column_names.contains(&"body"),
                    "Logs record batch should NOT contain 'body' column when all messages are fully parsed"
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
            })
        }
    }

    /// Validation closure that checks the received messages for TCP incomplete test.
    fn tcp_incomplete_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

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
                // Note: body column is not present when all messages are fully parsed
                // (all data is in attributes, no need for body)
                assert!(
                    !column_names.contains(&"body"),
                    "Logs record batch should NOT contain 'body' column when all messages are fully parsed"
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
            })
        }
    }

    #[test]
    fn test_syslog_cef_receiver_udp() {
        let test_runtime = TestRuntime::new();

        // addr and port for the UDP server to run at
        let listening_port = portpicker::pick_unused_port().expect("No free ports");
        let listening_addr: SocketAddr = format!("127.0.0.1:{listening_port}").parse().unwrap();

        let config = Config::new_udp(listening_addr);
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(SYSLOG_CEF_RECEIVER_URN));
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

        let config = Config::new_tcp(listening_addr);
        // create our TCP receiver
        let receiver = SyslogCefReceiver::new(config);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(SYSLOG_CEF_RECEIVER_URN));
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

        let config = Config::new_tcp(listening_addr);
        // create our TCP receiver
        let receiver = SyslogCefReceiver::new(config);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(SYSLOG_CEF_RECEIVER_URN));
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
mod config_tests {
    use super::*;

    #[test]
    fn valid_tcp() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(config.is_ok(), "Valid TCP config should parse successfully");
    }

    #[test]
    fn valid_udp() {
        let json = serde_json::json!({
            "protocol": {
                "udp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(config.is_ok(), "Valid UDP config should parse successfully");
    }

    #[test]
    fn missing_protocol() {
        let json = serde_json::json!({});
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_err(),
            "Config without protocol field should be rejected"
        );
    }

    #[test]
    fn unknown_protocol() {
        let json = serde_json::json!({
            "protocol": {
                "unix": {
                    "path": "/var/run/syslog.sock"
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_err(),
            "Config with unknown protocol should be rejected"
        );
    }

    #[test]
    fn unknown_top_level_field_rejected() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            },
            "extra_field": "unexpected"
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_err(),
            "Config with unknown top-level field should be rejected"
        );
    }

    #[test]
    fn tcp_unknown_field_rejected() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140",
                    "unknown_option": true
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_err(),
            "TCP config with unknown field should be rejected"
        );
    }

    #[test]
    fn protocol_is_case_sensitive() {
        // Protocol variants are snake_case; uppercase/mixed-case must be rejected.
        for variant in &["TCP", "Tcp", "UDP", "Udp"] {
            let json = serde_json::json!({
                "protocol": {
                    *variant: {
                        "listening_addr": "127.0.0.1:5140"
                    }
                }
            });
            let config: Result<Config, _> = serde_json::from_value(json);
            assert!(
                config.is_err(),
                "Protocol variant '{variant}' should be rejected (expected lowercase)"
            );
        }
    }

    #[test]
    fn both_protocols_rejected() {
        // The Protocol enum is externally tagged, so specifying both tcp and udp
        // in the same config must be rejected — only one protocol per instance.
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                },
                "udp": {
                    "listening_addr": "127.0.0.1:5145"
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_err(),
            "Config with both tcp and udp should be rejected"
        );
    }

    #[cfg(feature = "experimental-tls")]
    #[test]
    fn valid_tcp_with_tls() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140",
                    "tls": {
                        "cert_file": "/path/to/cert.pem",
                        "key_file": "/path/to/key.pem"
                    }
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_ok(),
            "TCP config with valid TLS settings should parse successfully"
        );
    }

    #[cfg(feature = "experimental-tls")]
    #[test]
    fn valid_tcp_with_tls_and_client_ca() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140",
                    "tls": {
                        "cert_file": "/path/to/cert.pem",
                        "key_file": "/path/to/key.pem",
                        "client_ca_file": "/path/to/ca.pem"
                    }
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_ok(),
            "TCP config with TLS + mTLS client CA should parse successfully"
        );
    }

    #[cfg(feature = "experimental-tls")]
    #[test]
    fn valid_tcp_with_tls_handshake_timeout() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140",
                    "tls": {
                        "cert_file": "/path/to/cert.pem",
                        "key_file": "/path/to/key.pem",
                        "handshake_timeout": "5s"
                    }
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_ok(),
            "TCP config with TLS handshake timeout should parse successfully"
        );
    }

    #[cfg(feature = "experimental-tls")]
    #[test]
    fn udp_with_tls_rejected() {
        let json = serde_json::json!({
            "protocol": {
                "udp": {
                    "listening_addr": "127.0.0.1:5140",
                    "tls": {
                        "cert_file": "/path/to/cert.pem",
                        "key_file": "/path/to/key.pem"
                    }
                }
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_err(),
            "UDP config with TLS should be rejected (TLS is TCP-only)"
        );
    }

    #[test]
    fn valid_tcp_with_batch_config() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            },
            "batch": {
                "flush_timeout_ms": 50,
                "max_size": 200
            }
        });
        let config: Config = serde_json::from_value(json).expect("should parse");
        assert_eq!(config.flush_timeout(), Duration::from_millis(50));
        assert_eq!(config.max_batch_size(), 200);
    }

    #[test]
    fn valid_udp_with_partial_batch_config() {
        let json = serde_json::json!({
            "protocol": {
                "udp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            },
            "batch": {
                "flush_timeout_ms": 25
            }
        });
        let config: Config = serde_json::from_value(json).expect("should parse");
        assert_eq!(config.flush_timeout(), Duration::from_millis(25));
        assert_eq!(
            config.max_batch_size(),
            DEFAULT_MAX_BATCH_SIZE,
            "max_batch_size should fall back to default when not specified"
        );
    }

    #[test]
    fn batch_defaults_when_omitted() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            }
        });
        let config: Config = serde_json::from_value(json).expect("should parse");
        assert_eq!(config.flush_timeout(), DEFAULT_FLUSH_TIMEOUT);
        assert_eq!(config.max_batch_size(), DEFAULT_MAX_BATCH_SIZE);
    }

    #[test]
    fn batch_unknown_field_rejected() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            },
            "batch": {
                "flush_timeout_ms": 50,
                "unknown_field": true
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_err(),
            "Batch config with unknown field should be rejected"
        );
    }

    #[test]
    fn batch_zero_flush_timeout_rejected() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            },
            "batch": {
                "flush_timeout_ms": 0
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(config.is_err(), "flush_timeout_ms of 0 should be rejected");
    }

    #[test]
    fn batch_zero_max_size_rejected() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            },
            "batch": {
                "max_size": 0
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(config.is_err(), "max_size of 0 should be rejected");
    }
}

#[cfg(test)]
mod telemetry_tests {
    use super::*;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::local::receiver::Receiver;
    use otap_df_engine::message::Sender;
    use otap_df_engine::testing::{setup_test_runtime, test_node};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::time::Instant;
    use tokio::net::UdpSocket;
    use tokio::time::Duration;

    #[test]
    fn udp_telemetry_success_and_failure_and_total() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            // Build pipeline context to register metrics on the receiver
            let telemetry_registry = TelemetryRegistryHandle::new();
            let controller = ControllerContext::new(telemetry_registry.clone());
            let pipeline = controller.pipeline_context_with(
                otap_df_config::PipelineGroupId::from("test-group".to_string()),
                otap_df_config::PipelineId::from("test-pipeline".to_string()),
                0,
                1, // num_cores
                0,
            );

            // addr and port for the UDP server to run at
            let listening_port = portpicker::pick_unused_port().expect("No free ports");
            let listening_addr: SocketAddr = format!("127.0.0.1:{listening_port}").parse().unwrap();

            // Receiver with metrics enabled via pipeline
            let receiver =
                SyslogCefReceiver::with_pipeline(pipeline, Config::new_udp(listening_addr));

            // Keep downstream open to avoid refused
            let (out_tx, mut _out_rx) = otap_df_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otap_df_engine::local::message::LocalSender::mpsc(out_tx)),
            );

            let (pipe_tx, _pipe_rx) = otap_df_engine::control::pipeline_ctrl_msg_channel(10);
            // Telemetry reporter for effect handler
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otap_df_engine::local::receiver::EffectHandler::new(
                test_node("syslog_udp_ok"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
            );

            let (ctrl_tx, ctrl_rx) = otap_df_channel::mpsc::Channel::new(16);
            let ctrl_rx = otap_df_engine::message::Receiver::Local(
                otap_df_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otap_df_engine::local::receiver::ControlChannel::new(ctrl_rx);

            // Start receiver
            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            // Send one valid and one invalid UDP datagram
            let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let _ = sock
                .send_to(
                    b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg",
                    listening_addr,
                )
                .await
                .unwrap();
            // Our RFC3164 parser accepts arbitrary non-empty strings as content-only messages.
            // To exercise the "invalid" path, send an empty datagram which is rejected by the parser.
            let _ = sock.send_to(b"", listening_addr).await.unwrap();

            // Allow interval to tick
            tokio::time::sleep(Duration::from_millis(150)).await;

            // Trigger telemetry collection
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });

            // Shutdown
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            // Validate
            let snapshot = metrics_rx.recv_async().await.unwrap();
            let m = snapshot.get_metrics();
            // Order: forwarded, invalid, forward_failed, total, tcp_connections_active
            assert_eq!(m[3].to_u64_lossy(), 2, "total == 2");
            assert_eq!(m[0].to_u64_lossy(), 1, "forwarded == 1");
            assert_eq!(m[1].to_u64_lossy(), 1, "invalid == 1");
        }));
    }

    #[test]
    fn udp_telemetry_refused_when_downstream_closed() {
        use otap_df_engine::testing::setup_test_runtime;
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            // Build pipeline context
            let telemetry_registry = TelemetryRegistryHandle::new();
            let controller = ControllerContext::new(telemetry_registry.clone());
            let pipeline = controller.pipeline_context_with(
                otap_df_config::PipelineGroupId::from("grp".to_string()),
                otap_df_config::PipelineId::from("pipe".to_string()),
                0,
                1, // num_cores
                0,
            );

            // Address
            let port = portpicker::pick_unused_port().expect("No free ports");
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

            // Receiver with pipeline metrics
            let receiver = SyslogCefReceiver::with_pipeline(pipeline, Config::new_udp(addr));

            // Wire a closed downstream to force refused
            let (tx, rx) = otap_df_channel::mpsc::Channel::new(1);
            drop(rx);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otap_df_engine::local::message::LocalSender::mpsc(tx)),
            );

            let (pipe_tx, _pipe_rx) = otap_df_engine::control::pipeline_ctrl_msg_channel(10);
            // Telemetry reporter for effect handler
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(2);
            let eh = otap_df_engine::local::receiver::EffectHandler::new(
                test_node("syslog_refused"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
            );

            let (ctrl_tx, ctrl_rx) = otap_df_channel::mpsc::Channel::new(8);
            let ctrl_rx = otap_df_engine::message::Receiver::Local(
                otap_df_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otap_df_engine::local::receiver::ControlChannel::new(ctrl_rx);

            // Start receiver
            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });
            // Allow bind
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Send one valid message (will be refused)
            let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let _ = sock
                .send_to(b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg", addr)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(150)).await;
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter,
            });
            // Shutdown
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            let snap = metrics_rx.recv_async().await.unwrap();
            let m = snap.get_metrics();
            assert_eq!(m[2].to_u64_lossy(), 1, "forward_failed == 1");
        }));
    }
}
