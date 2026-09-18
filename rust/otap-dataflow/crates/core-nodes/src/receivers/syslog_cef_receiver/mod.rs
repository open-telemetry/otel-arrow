// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = SYSLOG_CEF_RECEIVER_URN,
    target = "otel.receiver.syslog_cef",
);

use self::arrow_records_encoder::ArrowRecordsBuilder;
use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::admission::{
    AdmissionContext, AdmissionDecision, AdmissionDimension, LocalAdmissionGate,
};
use otel_arrow_dfe_engine::config::ReceiverConfig;
use otel_arrow_dfe_engine::context::{
    NodeAttributeSet, NodeWithCustomAttributeSet, PipelineContext,
};
use otel_arrow_dfe_engine::control::NodeControlMsg;
use otel_arrow_dfe_engine::memory_limiter::LocalReceiverAdmissionState;
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{MessageSourceLocalEffectHandlerExtension, ReceiverFactory};
use otel_arrow_dfe_engine::{
    error::{Error, ReceiverErrorKind, format_error_sources},
    local::receiver as local,
};
use otel_arrow_dfe_otap::OTAP_RECEIVER_FACTORIES;
use otel_arrow_dfe_otap::metrics::ReceiverMetrics;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_telemetry::common_attributes::{
    ReceiverRejectionErrorType, SignalRegistrationAttributes,
};
use otel_arrow_dfe_telemetry::instrument::{Counter, UpDownCounter};
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use serde::Deserialize;
use serde_json::Value;
use std::borrow::Cow;
use std::cell::{Cell, RefCell};
use std::net::SocketAddr;
use std::num::{NonZeroU16, NonZeroU64};
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncReadExt, BufReader};

use otel_arrow_dfe_config::tls::TlsServerConfig;
use otel_arrow_dfe_otap::tls_utils::{accept_tls_connection, build_tls_acceptor};

/// Arrow records encoder for syslog messages
pub mod arrow_records_encoder;
/// Parser module for syslog message parsing
pub mod parser;

/// URN for the syslog cef receiver
pub const SYSLOG_CEF_RECEIVER_URN: &str = "urn:otel:receiver:syslog_cef";

/// Default maximum time to wait before flushing an Arrow batch.
const DEFAULT_MAX_BATCH_DURATION: Duration = Duration::from_millis(100);
/// Default maximum number of messages to build an Arrow batch.
const DEFAULT_MAX_BATCH_SIZE: u16 = 100;

/// Maximum allowed size (in bytes) for a single syslog message.
///
/// Messages exceeding this limit are truncated and a `received_logs_truncated`
/// metric is incremented. 16 KiB covers virtually all real-world syslog and
/// CEF messages (RFC 5424 Section 6.1 recommends supporting messages of at least
/// 2048 bytes).
pub const MAX_MESSAGE_SIZE: usize = 16 * 1024;

/// Initial capacity for the per-connection TCP message buffer.
///
/// Most syslog messages are well under 4 KiB, so this avoids early
/// reallocations. The buffer can grow up to [`MAX_MESSAGE_SIZE`] if needed.
const INITIAL_MSG_BUFFER_CAPACITY: usize = 4096;

/// Maximum time to wait for spawned TCP tasks to drain during shutdown.
const MAX_TASK_DRAIN_WAIT: Duration = Duration::from_secs(1);

/// TCP message framing mode.
#[derive(Debug, Clone, Copy, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
enum TcpFraming {
    /// Messages are terminated by a newline character.
    #[default]
    Newline,
    /// Messages use RFC 6587 section 3.4.1 octet-counted framing.
    OctetCounting,
    /// Detect framing from the first byte of each message.
    Auto,
}

impl TcpFraming {
    const fn as_str(self) -> &'static str {
        match self {
            Self::Newline => "newline",
            Self::OctetCounting => "octet_counting",
            Self::Auto => "auto",
        }
    }
}

/// TCP-specific settings for the syslog CEF receiver.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TcpConfig {
    /// The address to listen on for TCP connections.
    listening_addr: SocketAddr,
    /// TCP message framing mode.
    #[serde(default)]
    framing: TcpFraming,
    /// TLS configuration for secure TCP connections (Syslog over TLS, RFC 5425).
    ///
    /// When configured, TCP connections will require TLS.
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

impl Protocol {
    const fn as_str(&self) -> &'static str {
        match self {
            Self::Tcp(_) => "tcp",
            Self::Udp(_) => "udp",
        }
    }
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
    max_batch_duration_ms: Option<NonZeroU64>,
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
    /// Returns the effective max batch duration, using the configured value or the default.
    fn max_batch_duration(&self) -> Duration {
        self.batch
            .as_ref()
            .and_then(|b| b.max_batch_duration_ms)
            .map(|ms| Duration::from_millis(ms.get()))
            .unwrap_or(DEFAULT_MAX_BATCH_DURATION)
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

/// Result of a bounded line read operation.
#[derive(Debug)]
enum BoundedReadResult {
    /// A complete line was read (ending with `\n`, which is included in the buffer).
    Complete,
    /// The buffer reached [`MAX_MESSAGE_SIZE`] before a newline was found.
    /// The message was truncated; only the first [`MAX_MESSAGE_SIZE`] bytes are
    /// in the buffer.
    Truncated,
    /// EOF was reached. The buffer may contain a partial message without trailing
    /// `\n`, or may be empty if no data was available.
    Eof,
}

/// Per-connection state used while reading a TCP frame.
struct TcpFrameState {
    message: Vec<u8>,
    octet_count: usize,
    octet_count_digits: usize,
    expected_octets: Option<usize>,
    detected_framing: Option<TcpFraming>,
}

impl TcpFrameState {
    fn new() -> Self {
        Self {
            message: Vec::with_capacity(INITIAL_MSG_BUFFER_CAPACITY),
            octet_count: 0,
            octet_count_digits: 0,
            expected_octets: None,
            detected_framing: None,
        }
    }

    fn clear_message(&mut self) {
        self.message.clear();
        self.detected_framing = None;
    }
}

/// Errors encountered while decoding TCP message framing.
#[derive(Debug)]
enum TcpFrameReadError {
    Io(std::io::Error),
    InvalidOctetCount,
    IncompleteFrame,
    MessageTooLarge,
}

impl TcpFrameReadError {
    const fn kind(&self) -> &'static str {
        match self {
            Self::Io(_) => "io",
            Self::InvalidOctetCount => "invalid_octet_count",
            Self::IncompleteFrame => "incomplete_frame",
            Self::MessageTooLarge => "message_too_large",
        }
    }
}

/// Reads bytes from `reader` into `buf` until one of:
/// - A newline (`\n`) is found -> returns [`BoundedReadResult::Complete`]
/// - `buf` reaches `max_size` bytes without a newline -> returns
///   [`BoundedReadResult::Truncated`]
/// - EOF is reached -> returns [`BoundedReadResult::Eof`]
///
/// This prevents unbounded memory growth from malicious or misbehaving clients
/// that send data without newline delimiters.
///
/// Uses [`AsyncReadExt::take`] to cap the read at `max_size` bytes, then
/// delegates to [`AsyncBufReadExt::read_until`] for the actual newline search.
///
/// # Important
///
/// The caller should pass `buf` in the same state as left by the previous call.
/// Because `read_until` appends to `buf`, and `select!` cancellation may interrupt
/// a read mid-stream, `buf` may contain partial data from a cancelled read that
/// the next call must continue from. Do **not** clear `buf` between calls unless
/// you are discarding the current message.
async fn read_line_bounded<R: AsyncBufRead + Unpin + ?Sized>(
    reader: &mut R,
    buf: &mut Vec<u8>,
    max_size: usize,
) -> std::io::Result<BoundedReadResult> {
    // Account for data already in buf from previous cancelled reads
    // (select! cancellation can leave partial data in buf).
    let remaining = max_size.saturating_sub(buf.len());
    if remaining == 0 {
        return Ok(BoundedReadResult::Truncated);
    }
    let n = (&mut *reader)
        .take(remaining as u64)
        .read_until(b'\n', buf)
        .await?;
    match n {
        0 => Ok(BoundedReadResult::Eof),
        _ if buf.last() == Some(&b'\n') => Ok(BoundedReadResult::Complete),
        _ if buf.len() >= max_size => Ok(BoundedReadResult::Truncated),
        _ => Ok(BoundedReadResult::Eof), // partial data before connection close
    }
}

/// Reads one TCP-framed syslog message.
///
/// Octet-counted state is updated only after `fill_buf` completes, so cancelling
/// this future does not lose bytes or framing progress.
async fn read_tcp_frame<R: AsyncBufRead + Unpin + ?Sized>(
    reader: &mut R,
    framing: TcpFraming,
    state: &mut TcpFrameState,
    max_size: usize,
) -> Result<BoundedReadResult, TcpFrameReadError> {
    let framing = if framing == TcpFraming::Auto {
        if let Some(detected_framing) = state.detected_framing {
            detected_framing
        } else {
            let available = reader.fill_buf().await.map_err(TcpFrameReadError::Io)?;
            let Some(&first_byte) = available.first() else {
                return Ok(BoundedReadResult::Eof);
            };
            let detected_framing = if matches!(first_byte, b'1'..=b'9') {
                TcpFraming::OctetCounting
            } else {
                TcpFraming::Newline
            };
            state.detected_framing = Some(detected_framing);
            detected_framing
        }
    } else {
        framing
    };

    if framing == TcpFraming::Newline {
        let result = read_line_bounded(reader, &mut state.message, max_size)
            .await
            .map_err(TcpFrameReadError::Io)?;
        if matches!(
            result,
            BoundedReadResult::Complete | BoundedReadResult::Truncated
        ) {
            state.detected_framing = None;
        }
        return Ok(result);
    }

    loop {
        if state.expected_octets.is_none() {
            let available = reader.fill_buf().await.map_err(TcpFrameReadError::Io)?;
            let Some(&byte) = available.first() else {
                return if state.octet_count_digits == 0 && state.message.is_empty() {
                    Ok(BoundedReadResult::Eof)
                } else {
                    Err(TcpFrameReadError::IncompleteFrame)
                };
            };

            match byte {
                b'0'..=b'9' => {
                    if state.octet_count_digits == 0 && byte == b'0' {
                        return Err(TcpFrameReadError::InvalidOctetCount);
                    }
                    state.octet_count = state
                        .octet_count
                        .checked_mul(10)
                        .and_then(|value| value.checked_add(usize::from(byte - b'0')))
                        .ok_or(TcpFrameReadError::InvalidOctetCount)?;
                    state.octet_count_digits += 1;
                    reader.consume(1);
                    if state.octet_count > max_size {
                        return Err(TcpFrameReadError::MessageTooLarge);
                    }
                }
                b' ' if state.octet_count_digits > 0 => {
                    reader.consume(1);
                    if state.octet_count == 0 {
                        return Err(TcpFrameReadError::InvalidOctetCount);
                    }

                    state.expected_octets = Some(state.octet_count);
                    state.octet_count = 0;
                    state.octet_count_digits = 0;
                }
                _ => return Err(TcpFrameReadError::InvalidOctetCount),
            }
            continue;
        }

        let expected_octets = state
            .expected_octets
            .expect("octet count must be available");
        let remaining = expected_octets.saturating_sub(state.message.len());
        if remaining == 0 {
            state.expected_octets = None;
            state.detected_framing = None;
            return Ok(BoundedReadResult::Complete);
        }

        let available = reader.fill_buf().await.map_err(TcpFrameReadError::Io)?;
        if available.is_empty() {
            return Err(TcpFrameReadError::IncompleteFrame);
        }
        let consumed = remaining.min(available.len());
        state.message.extend_from_slice(&available[..consumed]);
        reader.consume(consumed);
    }
}

const fn effective_tcp_framing(
    configured_framing: TcpFraming,
    discard_until_newline: bool,
) -> TcpFraming {
    if discard_until_newline {
        TcpFraming::Newline
    } else {
        configured_framing
    }
}

/// Syslog CEF receiver that can listen on TCP or UDP
#[allow(dead_code)]
struct SyslogCefReceiver {
    config: Config,
    /// Receiver metrics
    metrics: Rc<RefCell<SyslogCefReceiverMetrics>>,
    admission_state: LocalReceiverAdmissionState,
    rate_limiter: Option<LocalAdmissionGate>,
}

impl SyslogCefReceiver {
    /// Construct with pipeline context registering metrics
    fn with_pipeline(pipeline: PipelineContext, config: Config) -> Self {
        let metrics = Rc::new(RefCell::new(SyslogCefReceiverMetrics::register(
            &pipeline,
            config.protocol.as_str(),
        )));
        SyslogCefReceiver {
            config,
            metrics,
            admission_state: LocalReceiverAdmissionState::from_process_state(
                &pipeline.memory_pressure_state(),
            ),
            rate_limiter: None,
        }
    }

    /// Creates a new SyslogCefReceiver from a configuration object
    fn from_config(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otel_arrow_dfe_config::error::Error> {
        let cfg: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(SyslogCefReceiver::with_pipeline(pipeline, cfg))
    }
}

/// Discards any buffered records without sending downstream.
///
/// Used when memory pressure is active: flushing downstream could block behind the
/// full pipeline that pressure is trying to protect.
fn drop_syslog_batch(
    metrics: &Rc<RefCell<SyslogCefReceiverMetrics>>,
    arrow_records_builder: &mut ArrowRecordsBuilder,
) {
    let items = u64::from(arrow_records_builder.len());
    if items == 0 {
        return;
    }
    *arrow_records_builder = ArrowRecordsBuilder::new();
    metrics.borrow_mut().record_rejection(
        SyslogCefProtocol::Tcp,
        ReceiverRejectionErrorType::MemoryPressure,
        items,
    );
}

/// Runs the node-local Syslog receive work and records its terminal outcome.
fn process_syslog_message(
    metrics: &Rc<RefCell<SyslogCefReceiverMetrics>>,
    protocol: SyslogCefProtocol,
    message: &[u8],
    admission_state: &LocalReceiverAdmissionState,
    rate_limiter: &Option<LocalAdmissionGate>,
    arrow_records_builder: &mut ArrowRecordsBuilder,
) -> Result<(), ReceiverRejectionErrorType> {
    let processing = metrics.borrow().received.processing();
    let completed = processing.run(|processing| {
        processing.set_payload_size_with(|| message.len());
        if admission_state.should_shed_ingress() {
            return Err(
                processing.refused(SignalType::Logs, ReceiverRejectionErrorType::MemoryPressure)
            );
        }
        if !admit_syslog_message(rate_limiter) {
            return Err(processing.refused(SignalType::Logs, ReceiverRejectionErrorType::RateLimit));
        }
        let parsed = parser::parse(message).map_err(|_| {
            processing.refused(SignalType::Logs, ReceiverRejectionErrorType::InvalidRequest)
        })?;
        arrow_records_builder.append_syslog(parsed);
        Ok((SignalType::Logs, ()))
    });
    let mut metrics = metrics.borrow_mut();
    let result = metrics.received.record(completed);
    if let Err(error_type) = result {
        metrics.record_rejection(protocol, error_type, 1);
    }
    result
}

/// Applies message-rate admission for one framed syslog message.
fn admit_syslog_message(rate_limiter: &Option<LocalAdmissionGate>) -> bool {
    match rate_limiter
        .as_ref()
        .map(|limiter| limiter.admit(1, AdmissionContext::EMPTY))
    {
        Some(AdmissionDecision::Throttle { .. } | AdmissionDecision::Oversized) => false,
        Some(AdmissionDecision::WouldThrottle) => true,
        Some(AdmissionDecision::Admit) | None => true,
    }
}

fn warn_tcp_rate_limit_drop_once(peer_addr: SocketAddr, warned: &mut bool) {
    if !*warned {
        *warned = true;
        otel_warn!(
            "syslog_cef_receiver.rate_limit.drop",
            peer = %peer_addr,
            message = "Dropping TCP syslog messages due to rate limiting under memory pressure"
        );
    }
}

#[cfg(test)]
fn local_rate_gate(
    policy: otel_arrow_dfe_config::policy::RateLimiterPolicy,
    admission_state: LocalReceiverAdmissionState,
) -> LocalAdmissionGate {
    otel_arrow_dfe_engine::admission::AdmissionBinder::configured("test", policy)
        .bind_local(AdmissionDimension::Messages, admission_state)
        .expect("bind test admission")
        .expect("configured test admission")
}

/// Add the syslog receiver to the receiver factory
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static SYSLOG_CEF_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: SYSLOG_CEF_RECEIVER_URN,
    create:
        |pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         receiver_config: &ReceiverConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            let admission = pipeline.admission().clone();
            let mut receiver = SyslogCefReceiver::from_config(pipeline, &node_config.config)?;
            receiver.rate_limiter = admission
                .bind_local(
                    AdmissionDimension::Messages,
                    receiver.admission_state.clone(),
                )
                .map_err(
                    |error| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                        error: error.to_string(),
                    },
                )?;
            Ok(ReceiverWrapper::local(
                receiver,
                node,
                node_config,
                receiver_config,
            ))
        },
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otel_arrow_dfe_config::validation::validate_typed_config::<Config>,
};

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for SyslogCefReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        match &self.config.protocol {
            Protocol::Tcp(tcp_config) => {
                otel_info!(
                    "syslog_cef_receiver.start",
                    protocol = "TCP",
                    framing = tcp_config.framing.as_str(),
                    listening_addr = tcp_config.listening_addr.to_string()
                );

                let listener = effect_handler.tcp_listener(tcp_config.listening_addr)?;

                // Build TLS acceptor if TLS is configured
                let maybe_tls_acceptor = build_tls_acceptor(tcp_config.tls.as_ref())
                    .await
                    .map_err(|e| Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Configuration,
                        error: format!("Failed to configure TLS: {}", e),
                        source_detail: format_error_sources(&e),
                    })?;

                // Extract handshake timeout from TLS config (if present)
                let maybe_handshake_timeout = tcp_config
                    .tls
                    .as_ref()
                    .and_then(|tls| tls.handshake_timeout);

                if maybe_tls_acceptor.is_some() {
                    otel_info!(
                        "syslog_cef_receiver.tls_enabled",
                        message = "TLS enabled for Syslog/CEF TCP receiver"
                    );
                }

                // Resolve effective batching settings from config
                let max_batch_duration = self.config.max_batch_duration();
                let max_batch_size = self.config.max_batch_size();
                let tcp_framing = tcp_config.framing;

                let shutdown_flag = Rc::new(Cell::new(false));
                // Counter to track active connection tasks for graceful shutdown
                let active_task_count = Rc::new(Cell::new(0usize));

                loop {
                    tokio::select! {
                        biased; // Prioritize control messages over data

                        // Process incoming control messages.
                        ctrl_msg = ctrl_chan.recv() => {
                            match ctrl_msg {
                                Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                                    // Receiver-first shutdown stops new accepts immediately, but
                                    // for TCP we still wait for already accepted connection tasks
                                    // to flush their per-connection buffers before reporting
                                    // ReceiverDrained to the runtime.
                                    shutdown_flag.set(true); // Signal all connection tasks to flush and exit

                                    // Wait for active tasks to finish flushing.
                                    // Use 90% of remaining time (keeping 10% buffer for cleanup),
                                    // capped at MAX_TASK_DRAIN_WAIT.
                                    let time_until_deadline = deadline.saturating_duration_since(Instant::now());
                                    let drain_wait = std::cmp::min(time_until_deadline * 9 / 10, MAX_TASK_DRAIN_WAIT);
                                    let drain_result = tokio::time::timeout(drain_wait, async {
                                        while active_task_count.get() > 0 {
                                            tokio::task::yield_now().await;
                                        }
                                    }).await;

                                    if drain_result.is_err() {
                                        otel_warn!(
                                            "syslog_cef_receiver.drain_ingress.timeout",
                                            active_tasks = active_task_count.get(),
                                            message = "Ingress drain timeout expired with tasks still active"
                                        );
                                    }

                                    // Once all connection tasks have either flushed or timed out,
                                    // the runtime can safely advance to downstream shutdown.
                                    effect_handler.notify_receiver_drained().await?;

                                    let snapshots = self.metrics.borrow_mut().terminal_snapshots();
                                    return Ok(TerminalState::new(deadline, snapshots));
                                }
                                Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                                    shutdown_flag.set(true);
                                    let snapshots = self.metrics.borrow_mut().terminal_snapshots();
                                    return Ok(TerminalState::new(deadline, snapshots));
                                }
                                Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                    let mut m = self.metrics.borrow_mut();
                                    let _ = m.report(&mut metrics_reporter);
                                }
                                Ok(NodeControlMsg::MemoryPressureChanged { update }) => {
                                    self.admission_state.apply(update);
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
                                    if self.admission_state.should_shed_ingress() {
                                        self.metrics.borrow_mut().record_connection_rejection();
                                        continue;
                                    }

                                    // Clone the effect handler so the spawned task can send messages.
                                    let effect_handler = effect_handler.clone();
                                    let metrics = self.metrics.clone();
                                    let admission_state = self.admission_state.clone();
                                    let rate_limiter = self.rate_limiter.clone();

                                    // Clone TLS acceptor for the spawned task
                                    let tls_acceptor = maybe_tls_acceptor.clone();
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
                                        task_active_count.set(task_active_count.get() + 1);
                                        metrics.borrow_mut().record_connection_active(true);

                                        // Perform TLS handshake if configured, creating a unified reader type
                                        let mut reader: Box<dyn AsyncBufRead + Unpin> = if let Some(acceptor) = tls_acceptor {
                                            // Use configured timeout or fall back to 10 seconds (the serde default)
                                            let timeout = tls_handshake_timeout
                                                .unwrap_or(Duration::from_secs(10));
                                            match accept_tls_connection(socket, &acceptor, timeout).await {
                                                Ok(tls_stream) => {
                                                    otel_debug!(
                                                        "syslog_cef_receiver.tls.handshake.success",
                                                        peer = %peer_addr,
                                                        message = "TLS handshake completed"
                                                    );
                                                    Box::new(BufReader::new(tls_stream))
                                                }
                                                Err(e) => {
                                                    otel_warn!(
                                                        "syslog_cef_receiver.tls.handshake.failed",
                                                        peer = %peer_addr,
                                                        error = %e,
                                                        message = "TLS handshake failed, closing connection"
                                                    );
                                                    metrics.borrow_mut().record_transport_error(SyslogCefProtocol::Tcp);
                                                    task_active_count.set(task_active_count.get() - 1);
                                                metrics.borrow_mut().record_connection_active(false);
                                                    return;
                                                }
                                            }
                                        } else {
                                            Box::new(BufReader::new(socket))
                                        };

                                        let mut frame_state = TcpFrameState::new();
                                        let mut discard_until_newline = false;
                                        let mut warned_rate_limit_drop = false;

                                        let mut arrow_records_builder = ArrowRecordsBuilder::new();

                                        let start = tokio::time::Instant::now() + max_batch_duration;
                                        let mut interval = tokio::time::interval_at(start, max_batch_duration);

                                        loop {
                                            // Check for shutdown signal (simple bool check - very cheap)
                                            if task_shutdown_flag.get() {
                                                if arrow_records_builder.len() > 0 {
                                                    match arrow_records_builder.build() {
                                                        Ok(arrow_records) => {
                                                            let _ = effect_handler.try_send_message_with_source_node(
                                                                OtapPdata::new_todo_context(arrow_records.into()).with_peer_addr(peer_addr)
                                                            );
                                                        }
                                                        Err(e) => {
                                                            otel_warn!("syslog_cef_receiver.arrow_records.build_failed", error = %e, message = "Failed to build Arrow records, dropping batch");
                                                        }
                                                    }
                                                }
                                                task_active_count.set(task_active_count.get() - 1);
                                                metrics.borrow_mut().record_connection_active(false);
                                                break;
                                            }

                                            if admission_state.should_shed_ingress() {
                                                otel_warn!(
                                                    "syslog_cef_receiver.memory_pressure.disconnect",
                                                    peer = %peer_addr,
                                                    message = "Closing TCP syslog connection due to memory pressure"
                                                );
                                                drop_syslog_batch(
                                                    &metrics,
                                                    &mut arrow_records_builder,
                                                );
                                                metrics.borrow_mut().record_connection_rejection();
                                                task_active_count.set(task_active_count.get() - 1);
                                                metrics.borrow_mut().record_connection_active(false);
                                                break;
                                            }

                                            tokio::select! {
                                                biased; // Prioritize incoming data over timeout

                                                // Handle incoming data
                                                read_result = read_tcp_frame(
                                                    &mut *reader,
                                                    effective_tcp_framing(tcp_framing, discard_until_newline),
                                                    &mut frame_state,
                                                    MAX_MESSAGE_SIZE,
                                                ) => {
                                                    match read_result {
                                                        Ok(BoundedReadResult::Eof) => {
                                                            // EOF reached - connection closed
                                                            // Check if there's an incomplete line to process
                                                            if discard_until_newline {
                                                                frame_state.clear_message();
                                                            } else if !frame_state.message.is_empty() {
                                                                // Remove trailing newline if present
                                                                let message_bytes = if frame_state.message.last() == Some(&b'\n') {
                                                                    &frame_state.message[..frame_state.message.len()-1]
                                                                } else {
                                                                    &frame_state.message[..]
                                                                };
                                                                match process_syslog_message(
                                                                    &metrics,
                                                                    SyslogCefProtocol::Tcp,
                                                                    message_bytes,
                                                                    &admission_state,
                                                                    &rate_limiter,
                                                                    &mut arrow_records_builder,
                                                                ) {
                                                                    Err(ReceiverRejectionErrorType::MemoryPressure) => {
                                                                        otel_warn!(
                                                                            "syslog_cef_receiver.memory_pressure.disconnect",
                                                                            peer = %peer_addr,
                                                                            message = "Closing TCP syslog connection due to memory pressure"
                                                                        );
                                                                        drop_syslog_batch(
                                                                            &metrics,
                                                                            &mut arrow_records_builder,
                                                                        );
                                                                        metrics.borrow_mut().record_connection_rejection();
                                                                        task_active_count.set(task_active_count.get() - 1);
                                                                        metrics.borrow_mut().record_connection_active(false);
                                                                        break;
                                                                    }
                                                                    Err(ReceiverRejectionErrorType::RateLimit) => {
                                                                        warn_tcp_rate_limit_drop_once(
                                                                            peer_addr,
                                                                            &mut warned_rate_limit_drop,
                                                                        );
                                                                        frame_state.clear_message();
                                                                        continue;
                                                                    }
                                                                    Ok(()) | Err(_) => {}
                                                                }
                                                            }
                                                            // Send any remaining records before closing
                                                            if arrow_records_builder.len() > 0 {
                                                                match arrow_records_builder.build() {
                                                                    Ok(arrow_records) => {
                                                                        let _ = effect_handler.send_message_with_source_node(
                                                                            OtapPdata::new_todo_context(arrow_records.into()).with_peer_addr(peer_addr)
                                                                        ).await;
                                                                    }
                                                                    Err(e) => {
                                                                        otel_warn!("syslog_cef_receiver.arrow_records.build_failed", error = %e, message = "Failed to build Arrow records, dropping batch");
                                                                    }
                                                                }
                                                            }

                                                            // Decrement active connections on EOF
                                                            task_active_count.set(task_active_count.get() - 1);
                                                            metrics.borrow_mut().record_connection_active(false);
                                                            break;
                                                        }
                                                        Ok(bounded_result) => {
                                                            if discard_until_newline {
                                                                if matches!(bounded_result, BoundedReadResult::Complete) {
                                                                    discard_until_newline = false;
                                                                }
                                                                frame_state.clear_message();
                                                                continue;
                                                            }

                                                            if matches!(bounded_result, BoundedReadResult::Truncated) {
                                                                metrics.borrow_mut().record_truncation();
                                                            }

                                                            // TODO: When a message exceeds MAX_MESSAGE_SIZE, the truncated
                                                            // head is emitted as one record and the remaining tail bytes become
                                                            // a separate record with no syslog header context (severity, timestamp,
                                                            // etc.). Consider adding fragment-correlation metadata (e.g. a shared
                                                            // attribute linking head and tail) or synthesizing a syslog header on
                                                            // the continuation fragment so downstream consumers can associate the
                                                            // pieces. See https://github.com/open-telemetry/otel-arrow/pull/2452#discussion_r3004024837

                                                            // Strip trailing newline if present
                                                            // (Complete has it, Truncated does not)
                                                            let message_to_parse = if frame_state.message.last() == Some(&b'\n') {
                                                                &frame_state.message[..frame_state.message.len()-1]
                                                            } else {
                                                                &frame_state.message[..]
                                                            };
                                                            match process_syslog_message(
                                                                &metrics,
                                                                SyslogCefProtocol::Tcp,
                                                                message_to_parse,
                                                                &admission_state,
                                                                &rate_limiter,
                                                                &mut arrow_records_builder,
                                                            ) {
                                                                Err(ReceiverRejectionErrorType::MemoryPressure) => {
                                                                    otel_warn!(
                                                                        "syslog_cef_receiver.memory_pressure.disconnect",
                                                                        peer = %peer_addr,
                                                                        message = "Closing TCP syslog connection due to memory pressure"
                                                                    );
                                                                    frame_state.clear_message();
                                                                    drop_syslog_batch(
                                                                        &metrics,
                                                                        &mut arrow_records_builder,
                                                                    );
                                                                    metrics.borrow_mut().record_connection_rejection();
                                                                    task_active_count.set(task_active_count.get() - 1);
                                                                    metrics.borrow_mut().record_connection_active(false);
                                                                    break;
                                                                }
                                                                Err(ReceiverRejectionErrorType::RateLimit) => {
                                                                    warn_tcp_rate_limit_drop_once(
                                                                        peer_addr,
                                                                        &mut warned_rate_limit_drop,
                                                                    );
                                                                    if matches!(bounded_result, BoundedReadResult::Truncated) {
                                                                        discard_until_newline = true;
                                                                    }
                                                                    frame_state.clear_message();
                                                                    continue;
                                                                }
                                                                Err(ReceiverRejectionErrorType::InvalidRequest) => {
                                                                    frame_state.clear_message();
                                                                    continue;
                                                                }
                                                                Ok(()) | Err(_) => {}
                                                            }

                                                            // Clear the bytes for the next iteration
                                                            frame_state.clear_message();

                                                            if arrow_records_builder.len() >= max_batch_size {
                                                                // Build the Arrow records to send them
                                                                match arrow_records_builder.build() {
                                                                    Ok(arrow_records) => {
                                                                        // Reset the builder for the next batch
                                                                        arrow_records_builder = ArrowRecordsBuilder::new();

                                                                        // Reset the timer since we already built an arrow record batch due to size constraint
                                                                        interval.reset();

                                                                        let _ = effect_handler.send_message_with_source_node(
                                                                            OtapPdata::new_todo_context(arrow_records.into()).with_peer_addr(peer_addr)
                                                                        ).await;
                                                                    }
                                                                    Err(e) => {
                                                                        otel_warn!("syslog_cef_receiver.arrow_records.build_failed", error = %e, message = "Failed to build Arrow records, dropping batch");
                                                                        arrow_records_builder = ArrowRecordsBuilder::new();
                                                                        interval.reset();
                                                                    }
                                                                }
                                                            }
                                                        }
                                                        Err(e) => {
                                                            match &e {
                                                                TcpFrameReadError::MessageTooLarge => {
                                                                    metrics.borrow_mut().record_rejection(
                                                                        SyslogCefProtocol::Tcp,
                                                                        ReceiverRejectionErrorType::InvalidRequest,
                                                                        1,
                                                                    );
                                                                }
                                                                TcpFrameReadError::InvalidOctetCount
                                                                | TcpFrameReadError::IncompleteFrame => {
                                                                    metrics.borrow_mut().record_rejection(
                                                                        SyslogCefProtocol::Tcp,
                                                                        ReceiverRejectionErrorType::InvalidRequest,
                                                                        1,
                                                                    );
                                                                }
                                                                TcpFrameReadError::Io(error) => {
                                                                    metrics.borrow_mut().record_transport_error(
                                                                        SyslogCefProtocol::Tcp,
                                                                    );
                                                                    otel_warn!(
                                                                        "syslog_cef_receiver.tcp.read_error",
                                                                        peer = %peer_addr,
                                                                        error = %error,
                                                                        message = "Closing TCP syslog connection after read error"
                                                                    );
                                                                }
                                                            }
                                                            if !matches!(e, TcpFrameReadError::Io(_)) {
                                                                otel_warn!(
                                                                    "syslog_cef_receiver.tcp.framing_error",
                                                                    peer = %peer_addr,
                                                                    error_type = e.kind(),
                                                                    message = "Closing TCP syslog connection after framing error"
                                                                );
                                                            }
                                                            // Send any remaining records before closing due to error
                                                            if arrow_records_builder.len() > 0 {
                                                                match arrow_records_builder.build() {
                                                                    Ok(arrow_records) => {
                                                                        let _ = effect_handler.send_message_with_source_node(
                                                                            OtapPdata::new_todo_context(arrow_records.into()).with_peer_addr(peer_addr)
                                                                        ).await;
                                                                    }
                                                                    Err(e) => {
                                                                        otel_warn!("syslog_cef_receiver.arrow_records.build_failed", error = %e, message = "Failed to build Arrow records, dropping batch");
                                                                    }
                                                                }
                                                            }

                                                            // Decrement active connections on read error
                                                            task_active_count.set(task_active_count.get() - 1);
                                                            metrics.borrow_mut().record_connection_active(false);
                                                            break;
                                                        }
                                                    }
                                                }

                                                // Handle timeout - send any accumulated records
                                                _ = interval.tick() => {
                                                    if arrow_records_builder.len() > 0 {
                                                        // Build the Arrow records and send them
                                                        match arrow_records_builder.build() {
                                                            Ok(arrow_records) => {
                                                                // Reset the builder for the next batch
                                                                arrow_records_builder = ArrowRecordsBuilder::new();

                                                                let _ = effect_handler.send_message_with_source_node(
                                                                    OtapPdata::new_todo_context(arrow_records.into()).with_peer_addr(peer_addr)
                                                                ).await;
                                                            }
                                                            Err(e) => {
                                                                otel_warn!("syslog_cef_receiver.arrow_records.build_failed", error = %e, message = "Failed to build Arrow records, dropping batch");
                                                                arrow_records_builder = ArrowRecordsBuilder::new();
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
                    "syslog_cef_receiver.start",
                    protocol = "UDP",
                    listening_addr = udp_config.listening_addr.to_string()
                );

                let socket = effect_handler.udp_socket(udp_config.listening_addr)?;
                let mut buf = vec![0u8; MAX_MESSAGE_SIZE];
                let mut arrow_records_builder = ArrowRecordsBuilder::new();

                let max_batch_duration = self.config.max_batch_duration();
                let max_batch_size = self.config.max_batch_size();

                let start = tokio::time::Instant::now() + max_batch_duration;
                let mut interval = tokio::time::interval_at(start, max_batch_duration);

                loop {
                    tokio::select! {
                        biased; // Prioritize control messages over data

                        // Process incoming control messages.
                        ctrl_msg = ctrl_chan.recv() => {
                            match ctrl_msg {
                                Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                                    // UDP has no long-lived connection tasks, so receiver-first
                                    // drain just means: stop ingesting new packets, flush the
                                    // current batch buffer once, then report ReceiverDrained.

                                    if arrow_records_builder.len() > 0 {
                                        match arrow_records_builder.build() {
                                            Ok(arrow_records) => {
                                                let _ = effect_handler.try_send_message_with_source_node(
                                                    OtapPdata::new_todo_context(arrow_records.into())
                                                );
                                            }
                                            Err(e) => {
                                                otel_warn!("syslog_cef_receiver.arrow_records.build_failed", error = %e, message = "Failed to build Arrow records, dropping batch");
                                            }
                                        }
                                    }

                                    effect_handler.notify_receiver_drained().await?;

                                    let snapshots = self.metrics.borrow_mut().terminal_snapshots();
                                    return Ok(TerminalState::new(deadline, snapshots));
                                }
                                Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                                    let snapshots = self.metrics.borrow_mut().terminal_snapshots();
                                    return Ok(TerminalState::new(deadline, snapshots));
                                }
                                Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                                    let mut m = self.metrics.borrow_mut();
                                    let _ = m.report(&mut metrics_reporter);
                                }
                                Ok(NodeControlMsg::MemoryPressureChanged { update }) => {
                                    self.admission_state.apply(update);
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
                                    // If the datagram filled the entire buffer, it was likely
                                    // truncated by the OS (the actual message may have been larger).
                                    // A message exactly MAX_MESSAGE_SIZE bytes would also trigger
                                    // this, but there is no way to distinguish the two cases with
                                    // UDP -- this heuristic is the best we can do.
                                    if n == buf.len() {
                                        self.metrics.borrow_mut().record_truncation();
                                    }

                                    if process_syslog_message(
                                        &self.metrics,
                                        SyslogCefProtocol::Udp,
                                        &buf[..n],
                                        &self.admission_state,
                                        &self.rate_limiter,
                                        &mut arrow_records_builder,
                                    )
                                    .is_err()
                                    {
                                        continue;
                                    }

                                    if arrow_records_builder.len() >= max_batch_size {
                                        // Build the Arrow records to send them
                                        match arrow_records_builder.build() {
                                            Ok(arrow_records) => {
                                                // Reset the builder for the next batch
                                                arrow_records_builder = ArrowRecordsBuilder::new();

                                                // Reset the timer since we already built an arrow record batch due to size constraint
                                                interval.reset();

                                                // Do not propagate downstream send errors; keep the UDP receiver running.
                                                let _ = effect_handler.send_message_with_source_node(OtapPdata::new_todo_context(arrow_records.into())).await;
                                            }
                                            Err(e) => {
                                                otel_warn!("syslog_cef_receiver.arrow_records.build_failed", error = %e, message = "Failed to build Arrow records, dropping batch");
                                                arrow_records_builder = ArrowRecordsBuilder::new();
                                                interval.reset();
                                            }
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
                                match arrow_records_builder.build() {
                                    Ok(arrow_records) => {
                                        // Reset the builder for the next batch
                                        arrow_records_builder = ArrowRecordsBuilder::new();

                                        // Do not propagate downstream send errors; keep the UDP receiver running.
                                        let _ = effect_handler.send_message_with_source_node(OtapPdata::new_todo_context(arrow_records.into())).await;
                                    }
                                    Err(e) => {
                                        otel_warn!("syslog_cef_receiver.arrow_records.build_failed", error = %e, message = "Failed to build Arrow records, dropping batch");
                                        arrow_records_builder = ArrowRecordsBuilder::new();
                                    }
                                }
                            }
                        },
                    }
                }
            }
        }
    }
}

/// Transport protocol used to receive a syslog/cef request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, AttributeEnum)]
pub enum SyslogCefProtocol {
    /// Syslog over TCP.
    Tcp,
    /// Syslog over UDP.
    Udp,
}

/// Node identity extended with the configured Syslog transport protocol.
#[attribute_set(scope, name = "node.protocol.attrs")]
#[derive(Debug, Clone, Default, Hash)]
struct NodeWithProtocolAttributeSet {
    /// Base node attributes.
    #[compose]
    node_attrs: NodeAttributeSet,
    /// Transport protocol associated with the receiver metrics.
    protocol: Cow<'static, str>,
}

/// Custom node identity extended with the configured Syslog transport protocol.
#[attribute_set(scope, name = "node.custom.protocol.attrs")]
#[derive(Debug, Clone, Default, Hash)]
struct NodeWithCustomProtocolAttributeSet {
    /// Base node and custom telemetry attributes.
    #[compose]
    node_custom_attrs: NodeWithCustomAttributeSet,
    /// Transport protocol associated with the receiver metrics.
    protocol: Cow<'static, str>,
}

fn register_syslog_entity(
    pipeline_ctx: &PipelineContext,
    protocol: &'static str,
) -> otel_arrow_dfe_telemetry::registry::EntityKey {
    if pipeline_ctx.has_custom_node_attributes() {
        pipeline_ctx.register_entity(NodeWithCustomProtocolAttributeSet {
            node_custom_attrs: pipeline_ctx.node_with_custom_attribute_set(),
            protocol: protocol.into(),
        })
    } else {
        pipeline_ctx.register_entity(NodeWithProtocolAttributeSet {
            node_attrs: pipeline_ctx.node_attribute_set(),
            protocol: protocol.into(),
        })
    }
}

/// Protocol and bounded error type dimensions for a rejected syslog request.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyslogCefRejectionAttributes {
    /// Transport on which the request was rejected.
    pub protocol: SyslogCefProtocol,
    /// Reason the request was rejected.
    #[attribute_key = "error.type"]
    pub error_type: ReceiverRejectionErrorType,
}

/// Protocol dimension for a transport-level receiver error.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyslogCefTransportAttributes {
    /// Transport that surfaced the error.
    pub protocol: SyslogCefProtocol,
}

/// Rejections metrics for syslog cef
#[metric_set(
    name = "receiver.syslog_cef.rejections",
    registration_attributes = SignalRegistrationAttributes,
    measurement_attributes = SyslogCefRejectionAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct SyslogCefRejectionMetrics {
    /// Number of items rejected
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
}

/// Truncations metrics for syslog cef
#[metric_set(
    name = "receiver.syslog_cef.truncations",
    registration_attributes = SignalRegistrationAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct SyslogCefTruncationMetrics {
    /// Truncated payloads
    #[metric(unit = "{item}")]
    pub items: Counter<u64>,
}

/// Transport-level syslog cef errors.
#[metric_set(
    name = "receiver.syslog_cef.transport",
    measurement_attributes = SyslogCefTransportAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct SyslogCefTransportMetrics {
    /// Number of transport-level server errors.
    #[metric(unit = "{error}")]
    pub errors: Counter<u64>,
}

/// Connection metrics for syslog cef
#[metric_set(name = "receiver.syslog_cef.connections")]
#[derive(Debug, Default, Clone)]
pub struct SyslogCefConnectionMetrics {
    /// Active connections
    #[metric(unit = "{connection}")]
    pub active: UpDownCounter<u64>,
    /// Rejected connections
    #[metric(unit = "{connection}")]
    pub rejected: Counter<u64>,
}

/// Shared bounded-cardinality Syslog CEF receiver metrics tracker.
pub struct SyslogCefReceiverMetrics {
    received: ReceiverMetrics,
    rejections: otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet<SyslogCefRejectionMetrics>,
    transport: otel_arrow_dfe_telemetry::metrics::MeasurementMetricSet<SyslogCefTransportMetrics>,
    truncations: otel_arrow_dfe_telemetry::metrics::MetricSet<SyslogCefTruncationMetrics>,
    connections: otel_arrow_dfe_telemetry::metrics::MetricSet<SyslogCefConnectionMetrics>,
}

impl SyslogCefReceiverMetrics {
    /// Registers all syslog cef receiver metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext, protocol: &'static str) -> Self {
        let signal_attrs = SignalRegistrationAttributes {
            signal: SignalType::Logs,
        };
        let entity = register_syslog_entity(pipeline_ctx, protocol);
        let registrar = pipeline_ctx.metric_set_registrar_for_entity(entity);
        Self {
            received: ReceiverMetrics::register_with(&registrar, pipeline_ctx.node_interests()),
            rejections: SyslogCefRejectionMetrics::register(pipeline_ctx, &signal_attrs),
            transport: SyslogCefTransportMetrics::register(pipeline_ctx),
            truncations: SyslogCefTruncationMetrics::register(pipeline_ctx, &signal_attrs),
            connections: SyslogCefConnectionMetrics::register(pipeline_ctx),
        }
    }

    /// Records a rejection
    pub fn record_rejection(
        &mut self,
        protocol: SyslogCefProtocol,
        error_type: ReceiverRejectionErrorType,
        count: u64,
    ) {
        self.rejections
            .with(SyslogCefRejectionAttributes {
                protocol,
                error_type,
            })
            .items
            .add(count);
    }

    /// Records a transport-level error
    pub fn record_transport_error(&mut self, protocol: SyslogCefProtocol) {
        self.transport
            .with(SyslogCefTransportAttributes { protocol })
            .errors
            .inc();
    }

    /// Records a truncated payload
    pub fn record_truncation(&mut self) {
        self.truncations.items.inc();
    }

    /// Records an active connection
    pub fn record_connection_rejection(&mut self) {
        self.connections.rejected.inc();
    }

    /// Records an active connection change
    pub fn record_connection_active(&mut self, is_add: bool) {
        if is_add {
            self.connections.active.inc();
        } else {
            self.connections.active.dec();
        }
    }

    /// Reports every touched syslog cef receiver metric bucket.
    pub fn report(
        &mut self,
        reporter: &mut otel_arrow_dfe_telemetry::reporter::MetricsReporter,
    ) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        self.received.report(reporter)?;
        reporter.report_measurement(&mut self.rejections)?;
        reporter.report_measurement(&mut self.transport)?;
        reporter.report(&mut self.truncations)?;
        reporter.report(&mut self.connections)
    }

    /// Takes every touched syslog cef receiver metric bucket for terminal handoff.
    pub fn terminal_snapshots(
        &mut self,
    ) -> Vec<otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot> {
        let mut snapshots = self.received.terminal_snapshots();
        snapshots.extend(self.rejections.terminal_snapshots());
        snapshots.extend(self.transport.terminal_snapshots());
        snapshots.extend(self.truncations.terminal_snapshots());
        snapshots.extend(self.connections.terminal_snapshots());
        snapshots
    }

    /// Returns a rejection bucket for inspection without marking it for export.
    #[must_use]
    pub fn rejections_for(
        &self,
        protocol: SyslogCefProtocol,
        error_type: ReceiverRejectionErrorType,
    ) -> &SyslogCefRejectionMetrics {
        self.rejections.get(SyslogCefRejectionAttributes {
            protocol,
            error_type,
        })
    }

    /// Returns an transport bucket for inspection without marking it for export.
    #[must_use]
    pub fn transport_for(&self, protocol: SyslogCefProtocol) -> &SyslogCefTransportMetrics {
        self.transport
            .get(SyslogCefTransportAttributes { protocol })
    }
}

#[cfg(test)]
impl Config {
    /// Creates a new Config for TCP. Test-only helper.
    #[must_use]
    const fn new_tcp(listening_addr: SocketAddr) -> Self {
        Self {
            protocol: Protocol::Tcp(TcpConfig {
                listening_addr,
                framing: TcpFraming::Newline,
                tls: None,
            }),
            batch: None,
        }
    }

    /// Creates a new Config for UDP. Test-only helper.
    #[must_use]
    const fn new_udp(listening_addr: SocketAddr) -> Self {
        Self {
            protocol: Protocol::Udp(UdpConfig { listening_addr }),
            batch: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_pdata::PayloadData;

    // Test-only constructor, not compiled in production
    impl SyslogCefReceiver {
        #[allow(dead_code)]
        fn new(config: Config) -> Self {
            let (pipeline_ctx, _) =
                otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests(
                    otel_arrow_dfe_engine::Interests::NODE_OUTPUT_METRICS,
                );

            SyslogCefReceiver {
                config,
                metrics: Rc::new(RefCell::new(SyslogCefReceiverMetrics::register(
                    &pipeline_ctx,
                    "tcp",
                ))),
                admission_state: LocalReceiverAdmissionState::from_process_state(
                    &otel_arrow_dfe_engine::memory_limiter::MemoryPressureState::default(),
                ),
                rate_limiter: None,
            }
        }
    }
    use otel_arrow_dfe_config::node::NodeUserConfig;
    use otel_arrow_dfe_engine::receiver::ReceiverWrapper;
    use otel_arrow_dfe_engine::testing::{
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

    /// Scenario: A pending Syslog batch is dropped before downstream handoff.
    /// Guarantees: Buffered records are cleared without rewriting the node-local success outcome.
    #[test]
    fn drop_syslog_batch_discards_records_without_downstream_send() {
        let receiver = SyslogCefReceiver::new(Config::new_tcp(
            "127.0.0.1:0".parse().expect("valid loopback address"),
        ));
        let mut arrow_records_builder = ArrowRecordsBuilder::new();
        process_syslog_message(
            &receiver.metrics,
            SyslogCefProtocol::Tcp,
            b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg",
            &receiver.admission_state,
            &receiver.rate_limiter,
            &mut arrow_records_builder,
        )
        .expect("valid Syslog message should be appended");

        drop_syslog_batch(&receiver.metrics, &mut arrow_records_builder);

        assert_eq!(arrow_records_builder.len(), 0);
        let mut m = receiver.metrics.borrow_mut();
        let snapshots = m.received.terminal_snapshots();
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.received"
                && snapshot.measurement_attribute_value("signal") == Some("logs")
                && snapshot.measurement_attribute_value("outcome") == Some("success")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .any(|metric| metric.name == "messages")
        }));
        assert!(
            m.rejections_for(
                SyslogCefProtocol::Tcp,
                ReceiverRejectionErrorType::MemoryPressure
            )
            .items
            .get()
                > 0
        );
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

    /// Scenario: A TCP listener in auto mode receives octet-counted CEF and newline syslog.
    /// Guarantees: Both framing styles are decoded and delivered through the receiver pipeline.
    fn tcp_auto_framing_scenario(
        listening_addr: SocketAddr,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                let mut stream = TcpStream::connect(listening_addr)
                    .await
                    .expect("Failed to connect to TCP server");

                let octet_payload = b"<14>1 2026-09-17T12:53:13-04:00 QC_BASTION_01 - - - - CEF:0|Palo Alto Networks|PAN-OS|11.1.13-h5|end|TRAFFIC|1|src=10.3.163.24";
                let octet_prefix = format!("{} ", octet_payload.len());
                stream
                    .write_all(octet_prefix.as_bytes())
                    .await
                    .expect("Failed to write octet count");
                stream
                    .write_all(octet_payload)
                    .await
                    .expect("Failed to write octet-counted message");
                stream
                    .write_all(b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 - newline\n")
                    .await
                    .expect("Failed to write newline message");
                stream.flush().await.expect("Failed to flush messages");

                tokio::time::sleep(Duration::from_millis(150)).await;
                drop(stream);

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
                use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                // Check that messages have been received through the effect_handler

                // Read the first message
                let message1_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for first message")
                    .expect("No first message received")
                    .payload();

                // Extract arrow_records for further validation
                let PayloadData::OtapArrowRecords(arrow_records) = message1_received.into_data()
                else {
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
                use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                // Check that messages have been received through the effect_handler

                // Read the first message
                let message1_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for first message")
                    .expect("No first message received")
                    .payload();

                // Extract arrow_records for further validation
                let PayloadData::OtapArrowRecords(arrow_records) = message1_received.into_data()
                else {
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
                use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                // Check that messages have been received through the effect_handler
                // Note: Messages might come in separate batches due to timing

                let mut total_records = 0;
                let mut received_messages = Vec::new();

                // Collect all messages within a reasonable timeout
                while total_records < 2 {
                    match timeout(Duration::from_secs(3), ctx.recv()).await {
                        Ok(Ok(message)) => {
                            let PayloadData::OtapArrowRecords(arrow_records) =
                                message.payload().into_data()
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
        let listening_port = otel_arrow_dfe_test_net::pick_unused_loopback_udp_port();
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
        let listening_port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
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

    /// Scenario: The receiver is configured to automatically detect TCP framing.
    /// Guarantees: Octet-counted CEF and newline syslog are emitted as parsed records.
    #[test]
    fn test_syslog_cef_receiver_tcp_auto_framing() {
        let test_runtime = TestRuntime::new();

        let listening_port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
        let listening_addr: SocketAddr = format!("127.0.0.1:{listening_port}").parse().unwrap();

        let mut config = Config::new_tcp(listening_addr);
        let Protocol::Tcp(tcp) = &mut config.protocol else {
            panic!("expected TCP config");
        };
        tcp.framing = TcpFraming::Auto;

        let receiver = SyslogCefReceiver::new(config);
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(SYSLOG_CEF_RECEIVER_URN));
        let receiver_wrapper = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver_wrapper)
            .run_test(tcp_auto_framing_scenario(listening_addr))
            .run_validation(tcp_validation_procedure());
    }

    #[test]
    fn test_syslog_cef_receiver_tcp_incomplete() {
        let test_runtime = TestRuntime::new();

        // addr and port for the TCP server to run at
        let listening_port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
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

    /// Test closure that sends a message exceeding MAX_MESSAGE_SIZE to verify
    /// truncation handling -- the receiver must not crash or exhaust memory.
    fn tcp_truncation_scenario(
        listening_addr: SocketAddr,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                let mut stream = TcpStream::connect(listening_addr)
                    .await
                    .expect("Failed to connect to TCP server");

                // Build an oversized syslog message (> MAX_MESSAGE_SIZE = 16 KiB).
                // Start with a valid RFC 5424 header so the truncated prefix is parseable.
                let header = b"<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - ";
                let padding_len = MAX_MESSAGE_SIZE + 500 - header.len();
                let mut oversized = Vec::with_capacity(MAX_MESSAGE_SIZE + 500 + 1);
                oversized.extend_from_slice(header);
                oversized.extend(std::iter::repeat_n(b'X', padding_len));
                oversized.push(b'\n');

                stream
                    .write_all(&oversized)
                    .await
                    .expect("Failed to write oversized message");
                stream.flush().await.expect("Failed to flush");

                // Send a normal-sized message afterward to verify the receiver
                // keeps working after truncation.
                let normal =
                    b"<165>1 2024-01-15T10:31:00.456Z host.example.com myapp 1234 ID123 - Normal\n";
                stream
                    .write_all(normal)
                    .await
                    .expect("Failed to write normal message");
                stream.flush().await.expect("Failed to flush");

                tokio::time::sleep(Duration::from_millis(200)).await;
                drop(stream);
                tokio::time::sleep(Duration::from_millis(100)).await;

                ctx.send_shutdown(Instant::now(), "Test")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation for the TCP truncation test.
    ///
    /// The oversized message is split into two reads by `read_line_bounded`:
    /// 1. The truncated head (first `MAX_MESSAGE_SIZE` bytes) -- contains the
    ///    valid syslog header and parses successfully.
    /// 2. The tail (remaining 500 bytes of padding + `\n`) -- parsed as an
    ///    RFC 3164 content-only message (the parser accepts any non-empty input).
    /// 3. The normal-sized message sent afterward.
    ///
    /// All three parse successfully, so we expect exactly 3 log records.
    fn tcp_truncation_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

                let mut total_records = 0;

                loop {
                    match timeout(Duration::from_secs(3), ctx.recv()).await {
                        Ok(Ok(message)) => {
                            let PayloadData::OtapArrowRecords(arrow_records) =
                                message.payload().into_data()
                            else {
                                panic!("Expected OtapArrowRecords variant");
                            };

                            let logs_batch = arrow_records
                                .get(ArrowPayloadType::Logs)
                                .expect("Expected Logs batch");
                            total_records += logs_batch.num_rows();
                        }
                        Ok(Err(_)) => break,
                        Err(_) => break,
                    }
                }

                // Oversized message produces 2 records (truncated head + tail),
                // plus 1 normal message = 3 total.
                assert_eq!(
                    total_records, 3,
                    "Expected 3 log records after truncation (head + tail + normal), got {total_records}"
                );
            })
        }
    }

    #[test]
    fn test_syslog_cef_receiver_tcp_truncation() {
        let test_runtime = TestRuntime::new();

        let listening_port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
        let listening_addr: SocketAddr = format!("127.0.0.1:{listening_port}").parse().unwrap();

        let config = Config::new_tcp(listening_addr);
        let receiver = SyslogCefReceiver::new(config);

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(SYSLOG_CEF_RECEIVER_URN));
        let receiver_wrapper = ReceiverWrapper::local(
            receiver,
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver_wrapper)
            .run_test(tcp_truncation_scenario(listening_addr))
            .run_validation(tcp_truncation_validation_procedure());
    }
}

#[cfg(test)]
mod read_line_bounded_tests {
    use super::*;
    use tokio::io::{AsyncWriteExt, BufReader};

    /// Helper: create a BufReader over the write-end of a duplex stream,
    /// write `data`, then close the writer so EOF is visible.
    async fn make_reader(data: &[u8]) -> BufReader<tokio::io::DuplexStream> {
        let (reader_half, mut writer_half) = tokio::io::duplex(64 * 1024);
        writer_half.write_all(data).await.unwrap();
        drop(writer_half); // close so reads can hit EOF
        BufReader::new(reader_half)
    }

    #[tokio::test]
    async fn empty_reader_returns_eof() {
        let mut reader = make_reader(b"").await;
        let mut buf = Vec::new();
        let result = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(result, BoundedReadResult::Eof));
        assert!(buf.is_empty());
    }

    #[tokio::test]
    async fn complete_line() {
        let mut reader = make_reader(b"hello\n").await;
        let mut buf = Vec::new();
        let result = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(result, BoundedReadResult::Complete));
        assert_eq!(buf, b"hello\n");
    }

    #[tokio::test]
    async fn truncation_when_no_newline_exceeds_limit() {
        // 100 bytes of 'A', no newline, with max_size=64
        let data = vec![b'A'; 100];
        let mut reader = make_reader(&data).await;
        let mut buf = Vec::new();
        let result = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(result, BoundedReadResult::Truncated));
        assert_eq!(buf.len(), 64);
        assert!(buf.iter().all(|&b| b == b'A'));
    }

    #[tokio::test]
    async fn complete_line_at_exact_limit() {
        // 63 bytes + '\n' = 64 total, exactly at max_size
        // Should return Complete, not Truncated
        let mut data = vec![b'B'; 63];
        data.push(b'\n');
        let mut reader = make_reader(&data).await;
        let mut buf = Vec::new();
        let result = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(
            matches!(result, BoundedReadResult::Complete),
            "A line exactly at max_size ending with newline should be Complete"
        );
        assert_eq!(buf.len(), 64);
        assert_eq!(buf.last(), Some(&b'\n'));
    }

    #[tokio::test]
    async fn eof_with_partial_data() {
        // 10 bytes, no newline, then stream closes -- under the limit
        let mut reader = make_reader(b"partial").await;
        let mut buf = Vec::new();
        let result = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(result, BoundedReadResult::Eof));
        assert_eq!(buf, b"partial");
    }

    #[tokio::test]
    async fn multiple_calls_after_truncation() {
        // "AAAA...AAA\nBBBB\n" where A's exceed the limit
        // Call 1: Truncated (first 64 bytes of A's)
        // Call 2: Complete (remaining A's + \n)
        // Call 3: Complete ("BBBB\n")
        let mut data = vec![b'A'; 100];
        data.push(b'\n');
        data.extend_from_slice(b"BBBB\n");
        let mut reader = make_reader(&data).await;

        // First call: truncated at 64 bytes
        let mut buf = Vec::new();
        let r1 = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(r1, BoundedReadResult::Truncated));
        assert_eq!(buf.len(), 64);

        // Second call: reads remaining A's up to the \n
        buf.clear();
        let r2 = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(r2, BoundedReadResult::Complete));
        // 100 - 64 = 36 remaining A's + 1 newline = 37 bytes
        assert_eq!(buf.len(), 37);
        assert_eq!(buf.last(), Some(&b'\n'));

        // Third call: clean next line
        buf.clear();
        let r3 = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(r3, BoundedReadResult::Complete));
        assert_eq!(buf, b"BBBB\n");
    }

    #[tokio::test]
    async fn pre_filled_buf_caps_at_max_size() {
        // Simulate select! cancellation: buf already has 50 bytes from a
        // previous cancelled read. With max_size=64, only 14 more bytes
        // should be read, preventing buf from exceeding max_size.
        let data = vec![b'Z'; 100]; // plenty of data, no newline
        let mut reader = make_reader(&data).await;

        // Pre-fill buf with 50 bytes (as if a cancelled read left them)
        let mut buf = vec![b'A'; 50];
        let result = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(result, BoundedReadResult::Truncated));
        // buf should be exactly max_size, not 50 + 64
        assert_eq!(buf.len(), 64);
        // First 50 bytes are the pre-filled A's, next 14 are Z's from the reader
        assert!(buf[..50].iter().all(|&b| b == b'A'));
        assert!(buf[50..].iter().all(|&b| b == b'Z'));
    }

    #[tokio::test]
    async fn pre_filled_buf_at_limit_returns_truncated_immediately() {
        // buf is already at max_size -- should return Truncated without reading
        let mut reader = make_reader(b"should not be read\n").await;
        let mut buf = vec![b'X'; 64];
        let result = read_line_bounded(&mut reader, &mut buf, 64).await.unwrap();
        assert!(matches!(result, BoundedReadResult::Truncated));
        assert_eq!(buf.len(), 64);
        // Verify nothing was read from the stream
        assert!(buf.iter().all(|&b| b == b'X'));
    }
}

#[cfg(test)]
mod tcp_frame_reader_tests {
    use super::*;
    use crate::receivers::syslog_cef_receiver::parser::parsed_message::ParsedSyslogMessage;
    use tokio::io::{AsyncWriteExt, BufReader};

    async fn make_reader(data: &[u8]) -> BufReader<tokio::io::DuplexStream> {
        let (reader_half, mut writer_half) = tokio::io::duplex(64 * 1024);
        writer_half.write_all(data).await.unwrap();
        drop(writer_half);
        BufReader::new(reader_half)
    }

    fn octet_frame(payload: &[u8]) -> Vec<u8> {
        let mut frame = payload.len().to_string().into_bytes();
        frame.push(b' ');
        frame.extend_from_slice(payload);
        frame
    }

    /// Scenario: An RFC 5424 message containing CEF arrives with RFC 6587 framing.
    /// Guarantees: The transport removes the octet count before shared syslog parsing.
    #[tokio::test]
    async fn octet_counted_cef_over_rfc5424_is_unframed_before_parsing() {
        let payload = b"<14>1 2026-09-17T12:53:13-04:00 QC_BASTION_01 - - - - CEF:0|Palo Alto Networks|PAN-OS|11.1.13-h5|end|TRAFFIC|1|src=10.3.163.24";
        let mut reader = make_reader(&octet_frame(payload)).await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(
            &mut reader,
            TcpFraming::OctetCounting,
            &mut state,
            MAX_MESSAGE_SIZE,
        )
        .await
        .unwrap();

        assert!(matches!(result, BoundedReadResult::Complete));
        assert_eq!(state.message, payload);
        assert!(matches!(
            parser::parse(&state.message).unwrap(),
            ParsedSyslogMessage::CefWithRfc5424(_, _)
        ));
    }

    /// Scenario: Multiple octet-counted messages share one persistent TCP connection.
    /// Guarantees: Each declared payload is returned as one independent message.
    #[tokio::test]
    async fn reads_multiple_octet_counted_frames_without_delimiters() {
        let mut data = octet_frame(b"first");
        data.extend_from_slice(&octet_frame(b"second"));
        let mut reader = make_reader(&data).await;
        let mut state = TcpFrameState::new();

        let first = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64)
            .await
            .unwrap();
        assert!(matches!(first, BoundedReadResult::Complete));
        assert_eq!(state.message, b"first");

        state.clear_message();
        let second = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64)
            .await
            .unwrap();
        assert!(matches!(second, BoundedReadResult::Complete));
        assert_eq!(state.message, b"second");
    }

    /// Scenario: An octet-counted message contains LF and NUL delimiter bytes.
    /// Guarantees: MSG-LEN, not payload content, determines the message boundary.
    #[tokio::test]
    async fn preserves_delimiter_bytes_inside_octet_counted_payload() {
        let payload = b"<34>first line\nsecond line\0tail";
        let mut reader = make_reader(&octet_frame(payload)).await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64)
            .await
            .unwrap();

        assert!(matches!(result, BoundedReadResult::Complete));
        assert_eq!(state.message, payload);
    }

    /// Scenario: The octet count and payload arrive across separate TCP reads.
    /// Guarantees: Partial transport reads preserve framing progress and payload bytes.
    #[tokio::test]
    async fn reads_fragmented_octet_counted_frame() {
        let (reader_half, mut writer_half) = tokio::io::duplex(64);
        let mut reader = BufReader::new(reader_half);
        let mut state = TcpFrameState::new();

        let read = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64);
        let write = async move {
            writer_half.write_all(b"1").await.unwrap();
            tokio::task::yield_now().await;
            writer_half.write_all(b"1 hello").await.unwrap();
            tokio::task::yield_now().await;
            writer_half.write_all(b" world").await.unwrap();
        };

        let (result, ()) = tokio::join!(read, write);
        assert!(matches!(result.unwrap(), BoundedReadResult::Complete));
        assert_eq!(state.message, b"hello world");
    }

    /// Scenario: Auto framing receives octet-counted and newline messages in sequence.
    /// Guarantees: One TCP listener can decode both supported framing styles.
    #[tokio::test]
    async fn auto_detects_octet_counted_and_newline_frames() {
        let mut data = octet_frame(b"<34>first");
        data.extend_from_slice(b"<34>second\n");
        let mut reader = make_reader(&data).await;
        let mut state = TcpFrameState::new();

        let first = read_tcp_frame(&mut reader, TcpFraming::Auto, &mut state, 64)
            .await
            .unwrap();
        assert!(matches!(first, BoundedReadResult::Complete));
        assert_eq!(state.message, b"<34>first");

        state.clear_message();
        let second = read_tcp_frame(&mut reader, TcpFraming::Auto, &mut state, 64)
            .await
            .unwrap();
        assert!(matches!(second, BoundedReadResult::Complete));
        assert_eq!(state.message, b"<34>second\n");
    }

    /// Scenario: An auto-framed newline message begins with the ASCII digit zero.
    /// Guarantees: Only RFC 6587 NONZERO-DIGIT prefixes select octet-counting.
    #[tokio::test]
    async fn auto_treats_leading_zero_as_newline_framing() {
        let mut reader = make_reader(b"0 newline message\n").await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(&mut reader, TcpFraming::Auto, &mut state, 64)
            .await
            .unwrap();

        assert!(matches!(result, BoundedReadResult::Complete));
        assert_eq!(state.message, b"0 newline message\n");
    }

    /// Scenario: Auto mode discards a digit-leading continuation of a newline frame.
    /// Guarantees: Discarding remains newline-framed and the following frame stays aligned.
    #[tokio::test]
    async fn auto_mode_preserves_newline_framing_while_discarding() {
        let mut reader = make_reader(b"<34>1234567\n5 hello").await;
        let mut state = TcpFrameState::new();

        let head = read_tcp_frame(
            &mut reader,
            effective_tcp_framing(TcpFraming::Auto, false),
            &mut state,
            5,
        )
        .await
        .unwrap();
        assert!(matches!(head, BoundedReadResult::Truncated));

        state.clear_message();
        let tail = read_tcp_frame(
            &mut reader,
            effective_tcp_framing(TcpFraming::Auto, true),
            &mut state,
            64,
        )
        .await
        .unwrap();
        assert!(matches!(tail, BoundedReadResult::Complete));
        assert_eq!(state.message, b"234567\n");

        state.clear_message();
        let next = read_tcp_frame(
            &mut reader,
            effective_tcp_framing(TcpFraming::Auto, false),
            &mut state,
            64,
        )
        .await
        .unwrap();
        assert!(matches!(next, BoundedReadResult::Complete));
        assert_eq!(state.message, b"hello");
    }

    /// Scenario: An octet-counted payload is exactly the configured maximum size.
    /// Guarantees: The inclusive message-size boundary is accepted without truncation.
    #[tokio::test]
    async fn accepts_octet_counted_frame_at_size_limit() {
        let payload = vec![b'A'; 64];
        let mut reader = make_reader(&octet_frame(&payload)).await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64)
            .await
            .unwrap();

        assert!(matches!(result, BoundedReadResult::Complete));
        assert_eq!(state.message, payload);
    }

    /// Scenario: Octet-counted input has no numeric length prefix.
    /// Guarantees: Malformed framing is rejected instead of entering the shared parser.
    #[tokio::test]
    async fn rejects_invalid_octet_count_prefix() {
        let mut reader = make_reader(b"x <34>message").await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64).await;

        assert!(matches!(result, Err(TcpFrameReadError::InvalidOctetCount)));
    }

    /// Scenario: Octet-counted input declares an empty payload.
    /// Guarantees: A zero length is rejected as invalid RFC 6587 framing.
    #[tokio::test]
    async fn rejects_zero_octet_count() {
        let mut reader = make_reader(b"0 ").await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64).await;

        assert!(matches!(result, Err(TcpFrameReadError::InvalidOctetCount)));
    }

    /// Scenario: Octet-counted input uses a leading zero in its length.
    /// Guarantees: Length prefixes follow RFC 6587's nonzero-leading decimal syntax.
    #[tokio::test]
    async fn rejects_leading_zero_octet_count() {
        let mut reader = make_reader(b"05 hello").await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64).await;

        assert!(matches!(result, Err(TcpFrameReadError::InvalidOctetCount)));
    }

    /// Scenario: A connection closes before the declared payload is complete.
    /// Guarantees: Partial frames are rejected and never emitted as syslog messages.
    #[tokio::test]
    async fn rejects_incomplete_octet_counted_frame() {
        let mut reader = make_reader(b"5 abc").await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64).await;

        assert!(matches!(result, Err(TcpFrameReadError::IncompleteFrame)));
    }

    /// Scenario: A frame declares a payload larger than the receiver limit.
    /// Guarantees: Oversized input is rejected before allocating or reading its payload.
    #[tokio::test]
    async fn rejects_oversized_octet_counted_frame() {
        let mut reader = make_reader(b"65 ").await;
        let mut state = TcpFrameState::new();

        let result = read_tcp_frame(&mut reader, TcpFraming::OctetCounting, &mut state, 64).await;

        assert!(matches!(result, Err(TcpFrameReadError::MessageTooLarge)));
        assert!(state.message.is_empty());
    }
}

#[cfg(test)]
mod config_tests {
    use super::*;

    /// Scenario: TCP framing is omitted from configuration.
    /// Guarantees: Existing configurations continue to use newline framing.
    #[test]
    fn valid_tcp() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            }
        });
        let config: Config = serde_json::from_value(json).unwrap();
        let Protocol::Tcp(tcp) = config.protocol else {
            panic!("expected TCP config");
        };
        assert_eq!(tcp.framing, TcpFraming::Newline);
    }

    /// Scenario: Each documented TCP framing mode is configured explicitly.
    /// Guarantees: Newline, octet-counting, and auto framing values deserialize.
    #[test]
    fn valid_tcp_framing_modes() {
        for (value, expected) in [
            ("newline", TcpFraming::Newline),
            ("octet_counting", TcpFraming::OctetCounting),
            ("auto", TcpFraming::Auto),
        ] {
            let json = serde_json::json!({
                "protocol": {
                    "tcp": {
                        "listening_addr": "127.0.0.1:5140",
                        "framing": value
                    }
                }
            });
            let config: Config = serde_json::from_value(json).unwrap();
            let Protocol::Tcp(tcp) = config.protocol else {
                panic!("expected TCP config");
            };
            assert_eq!(tcp.framing, expected);
        }
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
        // in the same config must be rejected -- only one protocol per instance.
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
                "max_batch_duration_ms": 50,
                "max_size": 200
            }
        });
        let config: Config = serde_json::from_value(json).expect("should parse");
        assert_eq!(config.max_batch_duration(), Duration::from_millis(50));
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
                "max_batch_duration_ms": 25
            }
        });
        let config: Config = serde_json::from_value(json).expect("should parse");
        assert_eq!(config.max_batch_duration(), Duration::from_millis(25));
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
        assert_eq!(config.max_batch_duration(), DEFAULT_MAX_BATCH_DURATION);
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
                "max_batch_duration_ms": 50,
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
    fn batch_zero_max_batch_duration_rejected() {
        let json = serde_json::json!({
            "protocol": {
                "tcp": {
                    "listening_addr": "127.0.0.1:5140"
                }
            },
            "batch": {
                "max_batch_duration_ms": 0
            }
        });
        let config: Result<Config, _> = serde_json::from_value(json);
        assert!(
            config.is_err(),
            "max_batch_duration_ms of 0 should be rejected"
        );
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
    use otel_arrow_dfe_config::policy::{
        RateLimitAggregation, RateLimitEnforcement, RateLimitPressure, RateLimitUnit,
        RateLimiterPolicy, TokenBucketPolicy,
    };
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::local::receiver::Receiver;
    use otel_arrow_dfe_engine::memory_limiter::MemoryPressureLevel;
    use otel_arrow_dfe_engine::message::Sender;
    use otel_arrow_dfe_engine::testing::{
        setup_test_runtime, test_node, test_pipeline_ctx_with_interests,
    };
    use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
    use std::time::Instant;
    use tokio::io::AsyncWriteExt;
    use tokio::net::TcpStream;
    use tokio::net::UdpSocket;
    use tokio::time::Duration;

    fn test_pipeline_context() -> PipelineContext {
        test_pipeline_ctx_with_interests(Interests::NODE_OUTPUT_METRICS).0
    }

    fn registered_shared_metric_protocol(config: Config) -> (Vec<String>, bool) {
        let (pipeline, registry) = test_pipeline_ctx_with_interests(Interests::NODE_OUTPUT_METRICS);
        let _receiver = SyslogCefReceiver::with_pipeline(pipeline, config);
        let mut entities = Vec::new();
        let mut protocol_is_measurement_attribute = false;
        registry.visit_current_metrics_with_item_attrs(
            |descriptor, entity, item_attributes, _| {
                if descriptor.name == "receiver.received"
                    || descriptor.name == "receiver.processing"
                {
                    entities.push(entity.attributes_to_string());
                    protocol_is_measurement_attribute |=
                        item_attributes.iter().any(|(key, _)| *key == "protocol");
                }
            },
            true,
        );
        (entities, protocol_is_measurement_attribute)
    }

    /// Scenario: TCP and UDP Syslog receivers register shared receiver metrics.
    /// Guarantees: every shared metric set carries the configured protocol as a fixed entity
    /// attribute instead of adding protocol to per-message measurement attributes.
    #[test]
    fn shared_metrics_register_configured_protocol_entity_attribute() {
        let (tcp, tcp_protocol_is_measurement_attribute) = registered_shared_metric_protocol(
            Config::new_tcp("127.0.0.1:0".parse().expect("valid TCP address")),
        );
        assert!(!tcp.is_empty());
        assert!(tcp.iter().all(|entity| entity.contains("protocol=tcp")));
        assert!(!tcp_protocol_is_measurement_attribute);

        let (udp, udp_protocol_is_measurement_attribute) = registered_shared_metric_protocol(
            Config::new_udp("127.0.0.1:0".parse().expect("valid UDP address")),
        );
        assert!(!udp.is_empty());
        assert!(udp.iter().all(|entity| entity.contains("protocol=udp")));
        assert!(!udp_protocol_is_measurement_attribute);
    }

    /// Scenario: a Syslog receiver node has a custom telemetry identity attribute.
    /// Guarantees: the Syslog-owned protocol entity composes custom node identity without
    /// exposing protocol as a per-message measurement attribute.
    #[test]
    fn shared_metrics_protocol_entity_preserves_custom_node_identity() {
        use otel_arrow_dfe_config::node::NodeKind;
        use otel_arrow_dfe_config::pipeline::telemetry::{
            AttributeValue as ConfigAttributeValue, TelemetryAttribute,
        };
        use otel_arrow_dfe_engine::context::ControllerContext;
        use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
        use std::collections::HashMap;

        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry.clone());
        let mut custom = HashMap::new();
        let _ = custom.insert(
            "custom.identity.foo".to_string(),
            TelemetryAttribute::new(ConfigAttributeValue::String("bar".to_string())),
        );
        let pipeline = controller
            .pipeline_context_with("test_grp".into(), "test_pipeline".into(), 0, 1, 0)
            .with_node_context(
                "syslog".into(),
                SYSLOG_CEF_RECEIVER_URN.into(),
                NodeKind::Receiver,
                custom,
            );

        let _metrics = SyslogCefReceiverMetrics::register(&pipeline, "tcp");
        let mut entities = Vec::new();
        registry.visit_current_metrics_with_item_attrs(
            |descriptor, entity, _, _| {
                if descriptor.name == "receiver.received"
                    || descriptor.name == "receiver.processing"
                {
                    entities.push((entity.schema_name(), entity.attributes_to_string()));
                }
            },
            true,
        );

        assert!(!entities.is_empty());
        assert!(
            entities
                .iter()
                .all(|(schema, _)| *schema == "node.custom.protocol.attrs")
        );
        assert!(entities.iter().all(|(_, rendered)| {
            rendered.contains("protocol=tcp")
                && rendered.contains("node.id=syslog")
                && rendered.contains("custom={custom.identity.foo=bar}")
        }));
    }

    fn received_count(
        snaps: &[otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot],
        outcome: Option<&str>,
    ) -> u64 {
        let mut total = 0;
        for s in snaps {
            if s.descriptor().name != "receiver.received"
                || s.measurement_attribute_value("signal") != Some("logs")
            {
                continue;
            }
            if let Some(idx) = s
                .descriptor()
                .metrics
                .iter()
                .position(|f| f.name == "messages")
            {
                if let Some(o) = outcome {
                    if s.measurement_attribute_value("outcome") == Some(o) {
                        total += s.get_metrics()[idx].to_u64_lossy();
                    }
                } else {
                    total += s.get_metrics()[idx].to_u64_lossy();
                }
            }
        }
        total
    }

    fn rejection_count(
        snaps: &[otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot],
        error_type: Option<&str>,
    ) -> u64 {
        let mut total = 0;
        for s in snaps {
            if s.descriptor().name != "receiver.syslog_cef.rejections" {
                continue;
            }
            if let Some(idx) = s
                .descriptor()
                .metrics
                .iter()
                .position(|f| f.name == "items")
            {
                if let Some(e) = error_type {
                    if s.measurement_attribute_value("error.type") == Some(e) {
                        total += s.get_metrics()[idx].to_u64_lossy();
                    }
                } else {
                    total += s.get_metrics()[idx].to_u64_lossy();
                }
            }
        }
        total
    }

    fn truncation_count(snaps: &[otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot]) -> u64 {
        let mut total = 0;
        for s in snaps {
            if s.descriptor().name != "receiver.syslog_cef.truncations" {
                continue;
            }
            if let Some(idx) = s
                .descriptor()
                .metrics
                .iter()
                .position(|f| f.name == "items")
            {
                total += s.get_metrics()[idx].to_u64_lossy();
            }
        }
        total
    }

    fn messages_per_second_policy_with(
        enforcement: RateLimitEnforcement,
        allow: u64,
        burst: u64,
    ) -> RateLimiterPolicy {
        RateLimiterPolicy {
            enforcement,
            aggregation: RateLimitAggregation::ReceiverInstance,
            unit: RateLimitUnit::Messages,
            pressure: RateLimitPressure::Soft,
            token_bucket: TokenBucketPolicy {
                allow,
                interval: Duration::from_secs(1),
                burst: Some(burst),
            },
        }
    }

    fn messages_per_second_policy() -> RateLimiterPolicy {
        messages_per_second_policy_with(RateLimitEnforcement::Enforce, 1, 1)
    }

    fn oversized_syslog_message() -> Vec<u8> {
        let header = b"<34>1 2024-01-15T10:30:45.123Z mymachine.example.com su - ID47 - ";
        let padding_len = MAX_MESSAGE_SIZE + 500 - header.len();
        let mut oversized = Vec::with_capacity(MAX_MESSAGE_SIZE + 500 + 1);
        oversized.extend_from_slice(header);
        oversized.extend(std::iter::repeat_n(b'X', padding_len));
        oversized.push(b'\n');
        oversized
    }

    #[test]
    fn udp_telemetry_success_and_failure_and_total() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();

            // addr and port for the UDP server to run at
            let listening_port = otel_arrow_dfe_test_net::pick_unused_loopback_udp_port();
            let listening_addr: SocketAddr = format!("127.0.0.1:{listening_port}").parse().unwrap();

            // Receiver with metrics enabled via pipeline.
            // Use max_batch_size=1 so that the single record is flushed
            // immediately in the recv_from handler instead of waiting for
            // the interval tick, which avoids timing-dependent flakiness.
            let receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Udp(UdpConfig { listening_addr }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );

            // Keep downstream open so the batch send can complete.
            let (out_tx, mut _out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );

            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            // Telemetry reporter for effect handler
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_udp_ok"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );

            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(16);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);

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

            // Allow the receiver task to process the incoming messages.
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
            let mut snaps = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snaps.push(s);
            }
            assert_eq!(received_count(&snaps, None), 2, "total == 2");
            assert_eq!(
                received_count(&snaps, Some("success")),
                1,
                "node-local success == 1"
            );
            assert_eq!(
                rejection_count(&snaps, Some("invalid_request")),
                1,
                "invalid == 1"
            );
        }));
    }

    #[test]
    fn udp_telemetry_success_when_downstream_closed() {
        use otel_arrow_dfe_engine::testing::setup_test_runtime;
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();

            // Address
            let port = otel_arrow_dfe_test_net::pick_unused_loopback_udp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

            // Receiver with pipeline metrics.
            // Use max_batch_size=1 so that the single record is flushed
            // immediately in the recv_from handler instead of waiting for
            // the interval tick, which avoids timing-dependent flakiness.
            let receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Udp(UdpConfig {
                        listening_addr: addr,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );

            // Close downstream to prove handoff failure does not rewrite receiver outcome.
            let (tx, rx) = otel_arrow_dfe_channel::mpsc::Channel::new(1);
            drop(rx);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(tx)),
            );

            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            // Telemetry reporter for effect handler
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(2);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_refused"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );

            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);

            // Start receiver
            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });
            // Allow bind
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Send one valid message; downstream handoff will fail later.
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

            let mut snap = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snap.push(s);
            }
            assert_eq!(
                received_count(&snap, Some("success")),
                1,
                "node-local processing succeeded"
            );
        }));
    }

    /// Scenario: immediate shutdown terminates a classified UDP message still waiting in a batch.
    /// Guarantees: the buffered message retains node-local success and is not handed downstream.
    #[test]
    fn udp_shutdown_preserves_buffered_message_success() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();
            let port = otel_arrow_dfe_test_net::pick_unused_loopback_udp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
            let receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Udp(UdpConfig {
                        listening_addr: addr,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: NonZeroU64::new(10_000),
                        max_size: NonZeroU16::new(2),
                    }),
                },
            );

            let (out_tx, out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(1);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );
            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(2);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_udp_shutdown"),
                senders,
                None,
                pipe_tx,
                reporter,
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );
            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);
            let handle =
                tokio::task::spawn_local(
                    async move { Box::new(receiver).start(ctrl_chan, eh).await },
                );

            tokio::time::sleep(Duration::from_millis(50)).await;
            let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let _ = sock
                .send_to(b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg", addr)
                .await
                .unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;

            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(1),
                reason: "test".into(),
            });
            let terminal_state = handle.await.unwrap().unwrap();

            assert_eq!(received_count(terminal_state.metrics(), Some("success")), 1);
            assert!(out_rx.try_recv().is_err());
        }));
    }

    /// Scenario: receiver-first drain reaches a classified TCP message still waiting in a batch.
    /// Guarantees: the buffered message is handed downstream and reported once as successful.
    #[test]
    fn tcp_drain_flushes_buffered_message() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();
            let port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
            let receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Tcp(TcpConfig {
                        listening_addr: addr,
                        framing: TcpFraming::Newline,
                        tls: None,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: NonZeroU64::new(10_000),
                        max_size: NonZeroU16::new(2),
                    }),
                },
            );

            let (out_tx, out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(1);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );
            let (pipe_tx, mut pipe_rx) =
                otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (_metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(2);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_tcp_drain"),
                senders,
                None,
                pipe_tx,
                reporter,
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );
            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);
            let handle =
                tokio::task::spawn_local(
                    async move { Box::new(receiver).start(ctrl_chan, eh).await },
                );

            tokio::time::sleep(Duration::from_millis(50)).await;
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream
                .write_all(b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg\n")
                .await
                .unwrap();
            stream.flush().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;

            let _ = ctrl_tx.send(NodeControlMsg::DrainIngress {
                deadline: Instant::now() + Duration::from_secs(1),
                reason: "test".into(),
            });
            let terminal_state = handle.await.unwrap().unwrap();

            assert_eq!(received_count(terminal_state.metrics(), Some("success")), 1);
            assert!(out_rx.recv().await.is_ok());
            assert!(matches!(
                pipe_rx.recv().await,
                Ok(otel_arrow_dfe_engine::control::RuntimeControlMsg::ReceiverDrained { .. })
            ));
        }));
    }

    #[test]
    fn udp_sheds_ingress_under_hard_memory_pressure() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();

            let port = otel_arrow_dfe_test_net::pick_unused_loopback_udp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

            pipeline
                .memory_pressure_state()
                .set_level_for_tests(MemoryPressureLevel::Hard);

            let receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Udp(UdpConfig {
                        listening_addr: addr,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );

            let (out_tx, mut _out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );

            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_memory_pressure"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );

            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(16);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);

            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            tokio::time::sleep(Duration::from_millis(50)).await;

            let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let _ = sock
                .send_to(b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg", addr)
                .await
                .unwrap();

            tokio::time::sleep(Duration::from_millis(150)).await;
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            let mut snap = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snap.push(s);
            }
            assert_eq!(received_count(&snap, None), 1, "total == 1");
            assert_eq!(
                received_count(&snap, Some("success")),
                0,
                "node-local success == 0"
            );
            assert_eq!(
                rejection_count(&snap, Some("memory_pressure")),
                1,
                "memory-pressure dropped == 1"
            );
        }));
    }

    fn run_udp_under_capacity_rate_limit_test(enforcement: RateLimitEnforcement) {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();
            pipeline
                .memory_pressure_state()
                .set_level_for_tests(MemoryPressureLevel::Soft);

            let port = otel_arrow_dfe_test_net::pick_unused_loopback_udp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
            let mut receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Udp(UdpConfig {
                        listening_addr: addr,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );
            receiver.rate_limiter = Some(local_rate_gate(
                messages_per_second_policy_with(enforcement, 2, 2),
                receiver.admission_state.clone(),
            ));

            let (out_tx, out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );
            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_udp_rate_limit_under_capacity"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );
            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(16);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);
            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            tokio::time::sleep(Duration::from_millis(50)).await;
            let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let _ = sock
                .send_to(b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg", addr)
                .await
                .unwrap();
            let _ = tokio::time::timeout(Duration::from_secs(1), out_rx.recv())
                .await
                .expect("under-capacity UDP message should be forwarded")
                .expect("downstream channel should remain open");

            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            let mut snap = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snap.push(s);
            }
            assert_eq!(received_count(&snap, None), 1);
            assert_eq!(received_count(&snap, Some("success")), 1);
        }));
    }

    /// Scenario: a UDP syslog receiver in enforce mode remains below its rate limit.
    /// Guarantees: the datagram is forwarded with ordinary metrics and no refusal telemetry.
    #[test]
    fn udp_enforce_under_capacity_is_transparent() {
        run_udp_under_capacity_rate_limit_test(RateLimitEnforcement::Enforce);
    }

    /// Scenario: a UDP syslog receiver in observe-only mode remains below its rate limit.
    /// Guarantees: the datagram is forwarded with ordinary metrics and no refusal telemetry.
    #[test]
    fn udp_observe_only_under_capacity_is_transparent() {
        run_udp_under_capacity_rate_limit_test(RateLimitEnforcement::ObserveOnly);
    }

    fn run_tcp_under_capacity_rate_limit_test(enforcement: RateLimitEnforcement) {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();
            pipeline
                .memory_pressure_state()
                .set_level_for_tests(MemoryPressureLevel::Soft);

            let port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();
            let mut receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Tcp(TcpConfig {
                        listening_addr: addr,
                        framing: TcpFraming::Newline,
                        tls: None,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );
            receiver.rate_limiter = Some(local_rate_gate(
                messages_per_second_policy_with(enforcement, 2, 2),
                receiver.admission_state.clone(),
            ));

            let (out_tx, out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );
            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_tcp_rate_limit_under_capacity"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );
            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(16);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);
            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            tokio::time::sleep(Duration::from_millis(50)).await;
            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream
                .write_all(b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg\n")
                .await
                .unwrap();
            stream.flush().await.unwrap();
            let _ = tokio::time::timeout(Duration::from_secs(1), out_rx.recv())
                .await
                .expect("under-capacity TCP message should be forwarded")
                .expect("downstream channel should remain open");
            drop(stream);

            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            let mut snap = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snap.push(s);
            }
            assert_eq!(received_count(&snap, None), 1);
            assert_eq!(received_count(&snap, Some("success")), 1);
        }));
    }

    /// Scenario: a TCP syslog receiver in enforce mode remains below its rate limit.
    /// Guarantees: the framed message is forwarded with ordinary metrics and no refusal telemetry.
    #[test]
    fn tcp_enforce_under_capacity_is_transparent() {
        run_tcp_under_capacity_rate_limit_test(RateLimitEnforcement::Enforce);
    }

    /// Scenario: a TCP syslog receiver in observe-only mode remains below its rate limit.
    /// Guarantees: the framed message is forwarded with ordinary metrics and no refusal telemetry.
    #[test]
    fn tcp_observe_only_under_capacity_is_transparent() {
        run_tcp_under_capacity_rate_limit_test(RateLimitEnforcement::ObserveOnly);
    }

    /// Scenario: a UDP syslog receiver exceeds its message-rate bucket under soft pressure.
    /// Guarantees: over-limit datagrams are dropped before parsing and counted as rate refusals.
    #[test]
    fn udp_refuses_messages_over_rate_limit_under_soft_pressure() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();
            pipeline
                .memory_pressure_state()
                .set_level_for_tests(MemoryPressureLevel::Soft);

            let port = otel_arrow_dfe_test_net::pick_unused_loopback_udp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

            let mut receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Udp(UdpConfig {
                        listening_addr: addr,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );
            receiver.rate_limiter = Some(local_rate_gate(
                messages_per_second_policy(),
                receiver.admission_state.clone(),
            ));

            let (out_tx, mut _out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );

            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_udp_rate_limit"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );

            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(16);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);

            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            tokio::time::sleep(Duration::from_millis(50)).await;

            let sock = UdpSocket::bind("127.0.0.1:0").await.unwrap();
            let msg = b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg";
            let _ = sock.send_to(msg, addr).await.unwrap();
            let _ = sock.send_to(msg, addr).await.unwrap();

            tokio::time::sleep(Duration::from_millis(150)).await;
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            let mut snap = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snap.push(s);
            }
            assert_eq!(received_count(&snap, None), 2);
            assert_eq!(received_count(&snap, Some("success")), 1);
        }));
    }

    /// Scenario: a TCP syslog receiver exceeds its message-rate bucket under soft pressure.
    /// Guarantees: over-limit framed lines are dropped without closing the active connection.
    #[test]
    fn tcp_refuses_messages_over_rate_limit_without_closing_connection() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();
            pipeline
                .memory_pressure_state()
                .set_level_for_tests(MemoryPressureLevel::Soft);

            let port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

            let mut receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Tcp(TcpConfig {
                        listening_addr: addr,
                        framing: TcpFraming::Newline,
                        tls: None,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );
            receiver.rate_limiter = Some(local_rate_gate(
                messages_per_second_policy(),
                receiver.admission_state.clone(),
            ));

            let (out_tx, mut _out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );

            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_tcp_rate_limit"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );

            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(16);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);

            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            tokio::time::sleep(Duration::from_millis(50)).await;

            let mut stream = TcpStream::connect(addr).await.unwrap();
            let msg = b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg\n";
            stream.write_all(msg).await.unwrap();
            stream.flush().await.unwrap();
            stream.write_all(msg).await.unwrap();
            stream.flush().await.unwrap();
            drop(stream);

            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            let mut snap = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snap.push(s);
            }
            assert_eq!(received_count(&snap, None), 2);
            assert_eq!(received_count(&snap, Some("success")), 1);
        }));
    }

    /// Scenario: an oversized TCP syslog line is split into bounded-read fragments under a message-rate limit.
    /// Guarantees: each emitted fragment is rate checked while preserving tail-fragment parsing.
    #[test]
    fn tcp_oversized_line_charges_rate_limit_per_fragment() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();
            pipeline
                .memory_pressure_state()
                .set_level_for_tests(MemoryPressureLevel::Soft);

            let port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

            let mut receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Tcp(TcpConfig {
                        listening_addr: addr,
                        framing: TcpFraming::Newline,
                        tls: None,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );
            receiver.rate_limiter = Some(local_rate_gate(
                messages_per_second_policy(),
                receiver.admission_state.clone(),
            ));

            let (out_tx, mut _out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );

            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_tcp_rate_limit_oversized"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );

            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(16);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);

            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            tokio::time::sleep(Duration::from_millis(50)).await;

            let mut stream = TcpStream::connect(addr).await.unwrap();
            stream.write_all(&oversized_syslog_message()).await.unwrap();
            stream.flush().await.unwrap();
            let normal = b"<34>1 2024-01-15T10:30:46.123Z host app - ID2 msg\n";
            stream.write_all(normal).await.unwrap();
            stream.flush().await.unwrap();
            drop(stream);

            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            let mut snap = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snap.push(s);
            }
            assert_eq!(received_count(&snap, None), 3);
            assert_eq!(received_count(&snap, Some("success")), 1);
            assert_eq!(truncation_count(&snap), 1);
        }));
    }

    /// Scenario: a rejected TCP syslog line continues across three bounded fragments through newline.
    /// Guarantees: all continuation fragments are uncounted and the next complete message is admitted.
    #[test]
    fn tcp_rate_rejected_three_fragment_line_discards_all_continuations() {
        let (rt, local) = setup_test_runtime();
        rt.block_on(local.run_until(async move {
            let pipeline = test_pipeline_context();
            pipeline
                .memory_pressure_state()
                .set_level_for_tests(MemoryPressureLevel::Soft);

            let port = otel_arrow_dfe_test_net::pick_unused_loopback_tcp_port();
            let addr: SocketAddr = format!("127.0.0.1:{port}").parse().unwrap();

            let mut receiver = SyslogCefReceiver::with_pipeline(
                pipeline,
                Config {
                    protocol: Protocol::Tcp(TcpConfig {
                        listening_addr: addr,
                        framing: TcpFraming::Newline,
                        tls: None,
                    }),
                    batch: Some(BatchConfig {
                        max_batch_duration_ms: None,
                        max_size: NonZeroU16::new(1),
                    }),
                },
            );
            receiver.rate_limiter = Some(local_rate_gate(
                messages_per_second_policy(),
                receiver.admission_state.clone(),
            ));

            let (out_tx, mut _out_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(8);
            let mut senders = std::collections::HashMap::new();
            let _ = senders.insert(
                "".into(),
                Sender::Local(otel_arrow_dfe_engine::local::message::LocalSender::mpsc(
                    out_tx,
                )),
            );

            let (pipe_tx, _pipe_rx) = otel_arrow_dfe_engine::control::runtime_ctrl_msg_channel(10);
            let (metrics_rx, reporter) = MetricsReporter::create_new_and_receiver(4);
            let eh = otel_arrow_dfe_engine::local::receiver::EffectHandler::new(
                test_node("syslog_tcp_rate_limit_rejected_oversized"),
                senders,
                None,
                pipe_tx,
                reporter.clone(),
                otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
            );

            let (ctrl_tx, ctrl_rx) = otel_arrow_dfe_channel::mpsc::Channel::new(16);
            let ctrl_rx = otel_arrow_dfe_engine::message::Receiver::Local(
                otel_arrow_dfe_engine::local::message::LocalReceiver::mpsc(ctrl_rx),
            );
            let ctrl_chan = otel_arrow_dfe_engine::local::receiver::ControlChannel::new(ctrl_rx);

            let handle = tokio::task::spawn_local(async move {
                let _ = Box::new(receiver).start(ctrl_chan, eh).await;
            });

            tokio::time::sleep(Duration::from_millis(50)).await;

            let mut stream = TcpStream::connect(addr).await.unwrap();
            let normal_first = b"<34>1 2024-01-15T10:30:45.123Z host app - ID1 msg\n";
            stream.write_all(normal_first).await.unwrap();
            stream.flush().await.unwrap();

            let header = b"<34>1 2024-01-15T10:30:45.123Z host app - ID2 ";
            let mut oversized = Vec::with_capacity((MAX_MESSAGE_SIZE * 2) + 501);
            oversized.extend_from_slice(header);
            oversized.extend(std::iter::repeat_n(
                b'X',
                (MAX_MESSAGE_SIZE * 2) + 500 - header.len(),
            ));
            oversized.push(b'\n');
            stream
                .write_all(&oversized[..MAX_MESSAGE_SIZE])
                .await
                .unwrap();
            stream.flush().await.unwrap();

            tokio::time::sleep(Duration::from_millis(1_200)).await;

            stream
                .write_all(&oversized[MAX_MESSAGE_SIZE..MAX_MESSAGE_SIZE * 2])
                .await
                .unwrap();
            stream.flush().await.unwrap();
            tokio::time::sleep(Duration::from_millis(50)).await;

            stream
                .write_all(&oversized[MAX_MESSAGE_SIZE * 2..])
                .await
                .unwrap();
            let normal_second = b"<34>1 2024-01-15T10:30:46.123Z host app - ID2 msg\n";
            stream.write_all(normal_second).await.unwrap();
            stream.flush().await.unwrap();
            drop(stream);

            tokio::time::sleep(Duration::from_millis(200)).await;
            let _ = ctrl_tx.send(NodeControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            });
            let _ = ctrl_tx.send(NodeControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "test".into(),
            });
            let _ = handle.await;

            let mut snap = vec![metrics_rx.recv_async().await.unwrap()];
            while let Ok(s) = metrics_rx.try_recv() {
                snap.push(s);
            }
            assert_eq!(received_count(&snap, None), 3);
            assert_eq!(received_count(&snap, Some("success")), 2);
            assert_eq!(truncation_count(&snap), 1);
        }));
    }

    #[test]
    fn terminal_snapshots_preserve_enum_attribute_values_once() {
        let pipeline_ctx = test_pipeline_context();
        let mut metrics = SyslogCefReceiverMetrics::register(&pipeline_ctx, "tcp");

        let completed = metrics.received.processing().run(|processing| {
            processing.set_payload_size_with(|| 64);
            Ok::<_, otel_arrow_dfe_otap::metrics::ErrorWithOutcome<()>>((SignalType::Logs, ()))
        });
        metrics
            .received
            .record(completed)
            .expect("receiver processing succeeds");
        metrics.record_rejection(
            SyslogCefProtocol::Udp,
            ReceiverRejectionErrorType::MemoryPressure,
            1,
        );
        metrics.record_transport_error(SyslogCefProtocol::Udp);
        metrics.truncations.items.add(1);
        metrics.connections.active.add(1);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 5);

        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.received"
                && snapshot.measurement_attribute_value("outcome") == Some("success")
                && snapshot.measurement_attribute_value("signal") == Some("logs")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.syslog_cef.rejections"
                && snapshot.measurement_attribute_value("protocol") == Some("udp")
                && snapshot.measurement_attribute_value("error.type") == Some("memory_pressure")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.syslog_cef.transport"
                && snapshot.measurement_attribute_value("protocol") == Some("udp")
        }));
        assert!(
            snapshots.iter().any(|snapshot| {
                snapshot.descriptor().name == "receiver.syslog_cef.truncations"
            })
        );
        assert!(
            snapshots.iter().any(|snapshot| {
                snapshot.descriptor().name == "receiver.syslog_cef.connections"
            })
        );
    }
}
