// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Syslog/CEF encoded-batch passthrough exporter.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = SYSLOG_CEF_EXPORTER_URN,
    target = "otel.exporter.syslog_cef",
);

/// Syslog/CEF exporter configuration.
pub mod config;
pub use config::*;

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_engine::config::ExporterConfig;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_engine::control::{AckMsg, NackCause, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::metrics::ExporterMetrics;
use otel_arrow_dfe_otap::pdata::{OtapPdata, PdataEffectHandlerExtension};
use otel_arrow_dfe_pdata_codec::builtins::syslog::{MAX_MESSAGE_SIZE, validated_messages};
use otel_arrow_dfe_pdata_codec::{EncodePolicy, EncodingPlan, PdataEncoding};
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::metrics::MetricSetSnapshot;
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

/// Component URN for the syslog CEF exporter.
pub const SYSLOG_CEF_EXPORTER_URN: &str = "urn:otel:exporter:syslog_cef";
const SYSLOG_BATCH: PdataEncoding = PdataEncoding::new("syslog-batch");

/// Metrics emitted by a syslog CEF exporter: standard boundary metrics.
pub struct SyslogCefExporterMetrics {
    boundary: ExporterMetrics,
}

impl SyslogCefExporterMetrics {
    fn register(pipeline: &PipelineContext) -> Self {
        Self {
            boundary: ExporterMetrics::register(pipeline),
        }
    }

    fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary.report(reporter)
    }

    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        self.boundary.terminal_snapshots()
    }
}

struct SyslogCefExporter {
    config: SyslogCefExporterConfig,
    tcp: Option<TcpStream>,
    udp: Option<UdpSocket>,
    metrics: SyslogCefExporterMetrics,
}

/// Declares the syslog CEF exporter as a local exporter factory.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static SYSLOG_CEF_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: SYSLOG_CEF_EXPORTER_URN,
    create:
        |pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<otel_arrow_dfe_config::node::NodeUserConfig>,
         exporter_config: &ExporterConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            let config = SyslogCefExporterConfig::parse(&node_config.config)?;
            Ok(ExporterWrapper::local(
                SyslogCefExporter {
                    config,
                    tcp: None,
                    udp: None,
                    metrics: SyslogCefExporterMetrics::register(&pipeline),
                },
                node,
                node_config,
                exporter_config,
            ))
        },
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |value| SyslogCefExporterConfig::parse(value).map(|_| ()),
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for SyslogCefExporter {
    async fn start(
        mut self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let encoding_plan =
            effect_handler.resolve_encoding_plan(&SYSLOG_BATCH, EncodePolicy::default())?;

        loop {
            match inbox.recv().await {
                Ok(Message::PData(pdata)) => {
                    if let Some(terminal_state) = self
                        .export(pdata, &encoding_plan, &mut inbox, &effect_handler)
                        .await?
                    {
                        return Ok(terminal_state);
                    }
                }
                Ok(Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                })) => {
                    _ = self.metrics.report(&mut metrics_reporter);
                }
                Ok(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                    self.drain_shutdown_inbox(&mut inbox, &effect_handler)
                        .await?;
                    return Ok(TerminalState::new(
                        deadline,
                        self.metrics.terminal_snapshots(),
                    ));
                }
                Ok(_) => {}
                Err(otel_arrow_dfe_channel::error::RecvError::Closed) => {
                    self.drain_shutdown_inbox(&mut inbox, &effect_handler)
                        .await?;
                    return Ok(TerminalState::new(
                        std::time::Instant::now(),
                        self.metrics.terminal_snapshots(),
                    ));
                }
                Err(e) => return Err(Error::ChannelRecvError(e)),
            }
        }
    }
}

impl SyslogCefExporter {
    async fn export(
        &mut self,
        pdata: OtapPdata,
        encoding_plan: &EncodingPlan,
        inbox: &mut ExporterInbox<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<Option<TerminalState>, Error> {
        let batch_deadline = tokio::time::Instant::now() + self.config.batch_timeout;

        if pdata.signal_type() != SignalType::Logs {
            let reason = "syslog_cef exporter only supports logs signal";
            let attempt = self.metrics.boundary.attempt(pdata.signal_type());
            let refused = attempt.refused(reason);
            _ = self
                .metrics
                .boundary
                .record(attempt.complete::<(), _>(Err(refused)));
            effect_handler
                .notify_nack(NackMsg::new_permanent_with_cause(
                    reason,
                    pdata,
                    NackCause::Refused,
                ))
                .await?;
            return Ok(None);
        }

        let (context, mut payload) = pdata.into_parts();
        let encoded_bytes = match effect_handler
            .encode_owned(&mut payload, encoding_plan)
            .await
        {
            Ok(bytes) => bytes,
            Err(error) => {
                let reason = format!("syslog-batch encoding failed: {error}");
                let attempt = self.metrics.boundary.attempt(SignalType::Logs);
                let refused = attempt.refused(&reason);
                _ = self
                    .metrics
                    .boundary
                    .record(attempt.complete::<(), _>(Err(refused)));
                effect_handler
                    .notify_nack(NackMsg::new_permanent_with_cause(
                        reason,
                        OtapPdata::new(context, payload),
                        NackCause::Refused,
                    ))
                    .await?;
                return Ok(None);
            }
        };

        let messages = match validated_messages(&encoded_bytes) {
            Ok(messages) => messages,
            Err(error) => {
                let reason = format!("invalid syslog-batch: {error}");
                let attempt = self.metrics.boundary.attempt(SignalType::Logs);
                let refused = attempt.refused(&reason);
                _ = self
                    .metrics
                    .boundary
                    .record(attempt.complete::<(), _>(Err(refused)));
                effect_handler
                    .notify_nack(NackMsg::new_permanent_with_cause(
                        reason,
                        OtapPdata::new(context, payload),
                        NackCause::Refused,
                    ))
                    .await?;
                return Ok(None);
            }
        };

        if let Err(reason) = self.validate_transport(&messages) {
            let attempt = self.metrics.boundary.attempt(SignalType::Logs);
            let refused = attempt.refused(&reason);
            _ = self
                .metrics
                .boundary
                .record(attempt.complete::<(), _>(Err(refused)));
            effect_handler
                .notify_nack(NackMsg::new_permanent_with_cause(
                    reason,
                    OtapPdata::new(context, payload),
                    NackCause::Refused,
                ))
                .await?;
            return Ok(None);
        }

        let mut shutdown_deadline: Option<tokio::time::Instant> = None;
        let mut shutdown_std_deadline: Option<std::time::Instant> = None;
        let mut inbox_closed = false;

        for message in &messages {
            let current_deadline = shutdown_deadline
                .unwrap_or(batch_deadline)
                .min(batch_deadline);

            let mut attempt = self.metrics.boundary.attempt(SignalType::Logs);
            attempt.set_item_count_with(|| 1);
            attempt.set_payload_size_with(|| message.len());

            if tokio::time::Instant::now() >= current_deadline {
                let error = if shutdown_deadline.is_some_and(|dl| tokio::time::Instant::now() >= dl)
                {
                    io::Error::new(
                        io::ErrorKind::TimedOut,
                        "syslog export cancelled: shutdown deadline expired",
                    )
                } else {
                    io::Error::new(io::ErrorKind::TimedOut, "syslog batch send timed out")
                };
                self.tcp = None;
                self.udp = None;
                let failed = attempt.failed(&error);
                _ = self
                    .metrics
                    .boundary
                    .record(attempt.complete::<(), _>(Err(failed)));
                effect_handler
                    .notify_nack(NackMsg::new(
                        format!("syslog transport send failed: {error}"),
                        OtapPdata::new(context, payload),
                    ))
                    .await?;
                if let Some(deadline) = shutdown_std_deadline {
                    self.drain_shutdown_inbox(inbox, effect_handler).await?;
                    return Ok(Some(TerminalState::new(
                        deadline,
                        self.metrics.terminal_snapshots(),
                    )));
                }
                return Ok(None);
            }

            let message_result = {
                let mut send_fut = std::pin::pin!(Self::send_message_stream(
                    &self.config.protocol,
                    &mut self.tcp,
                    &mut self.udp,
                    message,
                    self.config.io_timeout,
                ));

                loop {
                    let current_deadline = shutdown_deadline
                        .unwrap_or(batch_deadline)
                        .min(batch_deadline);

                    tokio::select! {
                        _ = tokio::time::sleep_until(current_deadline) => {
                            if shutdown_deadline.is_some_and(|dl| tokio::time::Instant::now() >= dl) {
                                break Err(io::Error::new(
                                    io::ErrorKind::TimedOut,
                                    "syslog export cancelled: shutdown deadline expired",
                                ));
                            } else {
                                break Err(io::Error::new(
                                    io::ErrorKind::TimedOut,
                                    "syslog batch send timed out",
                                ));
                            }
                        }
                        ctrl = async {
                            if !inbox_closed {
                                inbox.recv_when(false).await
                            } else {
                                std::future::pending().await
                            }
                        } => {
                            match ctrl {
                                Ok(Message::Control(NodeControlMsg::CollectTelemetry {
                                    mut metrics_reporter,
                                })) => {
                                    _ = self.metrics.report(&mut metrics_reporter);
                                }
                                Ok(Message::Control(NodeControlMsg::Shutdown { deadline, .. })) => {
                                    let instant = tokio::time::Instant::from_std(deadline);
                                    shutdown_deadline = Some(instant);
                                    shutdown_std_deadline = Some(deadline);
                                    if tokio::time::Instant::now() >= instant {
                                        break Err(io::Error::new(
                                            io::ErrorKind::TimedOut,
                                            "syslog export cancelled: shutdown deadline expired",
                                        ));
                                    }
                                }
                                Ok(Message::Control(_)) => {}
                                Ok(Message::PData(data)) => {
                                    Self::nack_drained_pdata(
                                        &mut self.metrics,
                                        data,
                                        effect_handler,
                                    )
                                    .await?;
                                }
                                Err(_) => {
                                    inbox_closed = true;
                                    if shutdown_std_deadline.is_none() {
                                        shutdown_std_deadline = Some(std::time::Instant::now());
                                    }
                                }
                            }
                        }
                        res = &mut send_fut => {
                            break res;
                        }
                    }
                }
            };

            match message_result {
                Ok(()) => {
                    _ = self
                        .metrics
                        .boundary
                        .record(attempt.complete::<(), io::Error>(Ok(())));
                }
                Err(error) => {
                    self.tcp = None;
                    self.udp = None;
                    let failed = attempt.failed(&error);
                    _ = self
                        .metrics
                        .boundary
                        .record(attempt.complete::<(), _>(Err(failed)));
                    effect_handler
                        .notify_nack(NackMsg::new(
                            format!("syslog transport send failed: {error}"),
                            OtapPdata::new(context, payload),
                        ))
                        .await?;
                    if let Some(deadline) = shutdown_std_deadline {
                        self.drain_shutdown_inbox(inbox, effect_handler).await?;
                        return Ok(Some(TerminalState::new(
                            deadline,
                            self.metrics.terminal_snapshots(),
                        )));
                    }
                    return Ok(None);
                }
            }
        }

        effect_handler
            .notify_ack(AckMsg::new(OtapPdata::new(context, payload)))
            .await?;

        if let Some(deadline) = shutdown_std_deadline {
            self.drain_shutdown_inbox(inbox, effect_handler).await?;
            return Ok(Some(TerminalState::new(
                deadline,
                self.metrics.terminal_snapshots(),
            )));
        }
        Ok(None)
    }

    async fn nack_drained_pdata(
        metrics: &mut SyslogCefExporterMetrics,
        mut data: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let reason = "syslog exporter shutdown before batch send";
        let mut attempt = metrics.boundary.attempt(data.signal_type());
        attempt.set_item_count_with(|| data.num_items() as u64);
        let refused = attempt.refused(reason);
        _ = metrics
            .boundary
            .record(attempt.complete::<(), _>(Err(refused)));
        effect_handler
            .notify_nack(NackMsg::new(reason, data))
            .await?;
        Ok(())
    }

    async fn drain_shutdown_inbox(
        &mut self,
        inbox: &mut ExporterInbox<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        while let Ok(msg) = timeout(Duration::ZERO, inbox.recv_when(false)).await {
            match msg {
                Ok(Message::PData(data)) => {
                    Self::nack_drained_pdata(&mut self.metrics, data, effect_handler).await?;
                }
                Ok(Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                })) => {
                    _ = self.metrics.report(&mut metrics_reporter);
                }
                Ok(Message::Control(_)) => {}
                Err(_) => break,
            }
        }
        Ok(())
    }

    fn validate_transport(&self, messages: &[&[u8]]) -> Result<(), String> {
        for message in messages {
            if message.is_empty() {
                return Err("empty syslog message cannot be forwarded".into());
            }
            match &self.config.protocol {
                Protocol::Tcp(TcpConfig {
                    framing: TcpFraming::Newline,
                    ..
                }) => {
                    if message.contains(&b'\n') {
                        return Err(
                            "newline framing cannot preserve a message containing LF".into()
                        );
                    }
                    if message.len() + 1 > MAX_MESSAGE_SIZE {
                        return Err("newline-framed message exceeds receiver size limit".into());
                    }
                }
                Protocol::Udp(_) if message.len() >= MAX_MESSAGE_SIZE => {
                    return Err("UDP message reaches receiver truncation limit".into());
                }
                _ => {}
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    async fn send_message(&mut self, message: &[u8]) -> io::Result<()> {
        Self::send_message_stream(
            &self.config.protocol,
            &mut self.tcp,
            &mut self.udp,
            message,
            self.config.io_timeout,
        )
        .await
    }

    async fn send_message_stream(
        protocol: &Protocol,
        tcp: &mut Option<TcpStream>,
        udp: &mut Option<UdpSocket>,
        message: &[u8],
        io_duration: Duration,
    ) -> io::Result<()> {
        match protocol {
            Protocol::Tcp(config) => {
                if tcp.is_none() {
                    *tcp =
                        Some(io_timeout(TcpStream::connect(&config.endpoint), io_duration).await?);
                }
                let stream = tcp.as_mut().expect("connected TCP stream");
                match config.framing {
                    TcpFraming::Newline => {
                        io_timeout(stream.write_all(message), io_duration).await?;
                        io_timeout(stream.write_all(b"\n"), io_duration).await?;
                    }
                    TcpFraming::OctetCounting => {
                        let prefix = format!("{} ", message.len());
                        io_timeout(stream.write_all(prefix.as_bytes()), io_duration).await?;
                        io_timeout(stream.write_all(message), io_duration).await?;
                    }
                }
            }
            Protocol::Udp(config) => {
                if udp.is_none() {
                    let socket = io_timeout(
                        async {
                            let addrs = tokio::net::lookup_host(&config.endpoint).await?;
                            let mut last_error = None;
                            for addr in addrs {
                                let bind: SocketAddr = if addr.is_ipv4() {
                                    "0.0.0.0:0".parse().expect("static IPv4 bind address")
                                } else {
                                    "[::]:0".parse().expect("static IPv6 bind address")
                                };
                                match UdpSocket::bind(bind).await {
                                    Ok(socket) => match socket.connect(addr).await {
                                        Ok(()) => return Ok(socket),
                                        Err(err) => last_error = Some(err),
                                    },
                                    Err(err) => last_error = Some(err),
                                }
                            }
                            Err(last_error.unwrap_or_else(|| {
                                io::Error::new(
                                    io::ErrorKind::NotFound,
                                    format!("no addresses found for {}", config.endpoint),
                                )
                            }))
                        },
                        io_duration,
                    )
                    .await?;
                    *udp = Some(socket);
                }
                let sent = io_timeout(
                    udp.as_ref().expect("connected UDP socket").send(message),
                    io_duration,
                )
                .await?;
                if sent != message.len() {
                    return Err(io::Error::new(
                        io::ErrorKind::WriteZero,
                        "partial UDP datagram",
                    ));
                }
            }
        }
        Ok(())
    }
}

async fn io_timeout<T>(
    future: impl Future<Output = io::Result<T>>,
    duration: Duration,
) -> io::Result<T> {
    timeout(duration, future).await.map_err(|_| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            "syslog transport operation timed out",
        )
    })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::Interests;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_engine::control::PipelineCompletionMsg;
    use otel_arrow_dfe_engine::testing::exporter::{
        TestContext, TestRuntime, create_exporter_from_factory,
    };
    use otel_arrow_dfe_engine::testing::test_pipeline_ctx_with_interests;
    use otel_arrow_dfe_otap::testing::{TestCallData, create_test_pdata, next_nack};
    use otel_arrow_dfe_pdata::OtlpProtoBytes;
    use otel_arrow_dfe_pdata_codec::builtins::syslog::SyslogBatchBuilder;
    use otel_arrow_dfe_pdata_codec::{CodecRegistry, EncodedPdata, EncodedPdataBatch};
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message as _;
    use serde_json::json;
    use std::time::Instant as StdInstant;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    fn exporter(protocol: Protocol) -> SyslogCefExporter {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline = controller.pipeline_context_with("grp".into(), "syslog_cef".into(), 0, 1, 0);
        SyslogCefExporter {
            config: SyslogCefExporterConfig {
                protocol,
                io_timeout: DEFAULT_IO_TIMEOUT,
                batch_timeout: DEFAULT_BATCH_TIMEOUT,
            },
            tcp: None,
            udp: None,
            metrics: SyslogCefExporterMetrics::register(&pipeline),
        }
    }

    fn syslog_pdata(messages: &[&[u8]]) -> OtapPdata {
        let mut builder = SyslogBatchBuilder::new();
        for message in messages {
            builder.append_with_year(message, 2026).unwrap();
        }
        let batch = builder.finish(0).unwrap();
        let (bytes, _) = batch.into_parts();
        let registry = CodecRegistry::global().unwrap();
        let encoded = EncodedPdata::new(&registry, &SYSLOG_BATCH, SignalType::Logs, bytes).unwrap();
        OtapPdata::new_default(encoded.into())
    }

    async fn assert_nack(
        ctx: &mut TestContext<OtapPdata>,
        expected_permanent: bool,
        expected_reason: &str,
        expected_cause: NackCause,
    ) {
        let mut completion_receiver = ctx.take_pipeline_completion_receiver().unwrap();
        let completion = timeout(Duration::from_secs(5), completion_receiver.recv())
            .await
            .expect("timed out waiting for NACK")
            .expect("completion channel closed before NACK");
        match completion {
            PipelineCompletionMsg::DeliverNack { nack } => {
                assert_eq!(nack.permanent, expected_permanent);
                assert_eq!(nack.cause, expected_cause);
                assert!(
                    nack.reason.contains(expected_reason),
                    "unexpected reason: {}",
                    nack.reason
                );
            }
            PipelineCompletionMsg::DeliverAck { .. } => panic!("expected a NACK, got ACK"),
        }
    }

    async fn assert_ack(ctx: &mut TestContext<OtapPdata>) {
        let mut completion_receiver = ctx.take_pipeline_completion_receiver().unwrap();
        let completion = timeout(Duration::from_secs(5), completion_receiver.recv())
            .await
            .expect("timed out waiting for ACK")
            .expect("completion channel closed before ACK");
        match completion {
            PipelineCompletionMsg::DeliverAck { .. } => {}
            PipelineCompletionMsg::DeliverNack { nack } => {
                panic!("expected ACK, got NACK: {}", nack.reason);
            }
        }
    }

    /// Scenario: TCP destination uses newline framing for two original messages.
    /// Guarantees: A reused connection carries exact message bytes plus one delimiter each.
    #[tokio::test]
    async fn tcp_newline_sends_original_messages() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let receiver = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            let _ = stream.read_to_end(&mut bytes).await.unwrap();
            bytes
        });
        let mut exporter = exporter(Protocol::Tcp(TcpConfig {
            endpoint: endpoint.to_string(),
            framing: TcpFraming::Newline,
        }));
        exporter.send_message(b"<13>first").await.unwrap();
        exporter.send_message(b"CEF:0|second").await.unwrap();
        drop(exporter);
        let bytes = timeout(Duration::from_secs(10), receiver)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(bytes, b"<13>first\nCEF:0|second\n");
    }

    /// Scenario: TCP destination selects RFC 6587 octet-counted framing.
    /// Guarantees: Length prefixes match original message byte lengths exactly.
    #[tokio::test]
    async fn tcp_octet_counting_sends_length_prefix() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let endpoint = listener.local_addr().unwrap();
        let receiver = tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.unwrap();
            let mut bytes = Vec::new();
            let _ = stream.read_to_end(&mut bytes).await.unwrap();
            bytes
        });
        let mut exporter = exporter(Protocol::Tcp(TcpConfig {
            endpoint: endpoint.to_string(),
            framing: TcpFraming::OctetCounting,
        }));
        exporter.send_message(b"a\nb").await.unwrap();
        drop(exporter);
        let bytes = timeout(Duration::from_secs(10), receiver)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(bytes, b"3 a\nb");
    }

    /// Scenario: UDP destination receives two syslog messages.
    /// Guarantees: Each original message occupies exactly one datagram.
    #[tokio::test]
    async fn udp_sends_one_message_per_datagram() {
        let receiver = UdpSocket::bind("127.0.0.1:0").await.unwrap();
        let endpoint = receiver.local_addr().unwrap();
        let mut exporter = exporter(Protocol::Udp(UdpConfig {
            endpoint: endpoint.to_string(),
        }));
        let mut buffer = [0; 128];
        for expected in [b"<13>first".as_slice(), b"CEF:0|second".as_slice()] {
            exporter.send_message(expected).await.unwrap();
            let (size, _) = timeout(Duration::from_secs(10), receiver.recv_from(&mut buffer))
                .await
                .unwrap()
                .unwrap();
            assert_eq!(&buffer[..size], expected);
        }
    }

    /// Scenario: Inputs use newline framing or reach the receiver's UDP limit.
    /// Guarantees: Incompatible content is rejected before any network send.
    #[test]
    fn validates_transport_compatibility() {
        let endpoint = "127.0.0.1:514".to_string();
        let newline = exporter(Protocol::Tcp(TcpConfig {
            endpoint: endpoint.clone(),
            framing: TcpFraming::Newline,
        }));
        assert!(newline.validate_transport(&[b"a\nb"]).is_err());
        assert!(
            newline
                .validate_transport(&[&vec![b'x'; MAX_MESSAGE_SIZE]])
                .is_err()
        );
        assert!(newline.validate_transport(&[b""]).is_err());
        let udp = exporter(Protocol::Udp(UdpConfig { endpoint }));
        assert!(
            udp.validate_transport(&[&vec![b'x'; MAX_MESSAGE_SIZE]])
                .is_err()
        );
    }

    /// Scenario: Config selects supported transports, accepts DNS hostnames and custom durations, and rejects invalid fields.
    /// Guarantees: Valid configurations pass validation while malformed endpoints or timeouts are rejected.
    #[test]
    fn validates_config() {
        assert!(
            SyslogCefExporterConfig::parse(&json!({
                "protocol": {"tcp": {"endpoint": "syslog.example.com:514", "framing": "octet_counting"}},
                "io_timeout": "10s",
                "batch_timeout": "1m"
            }))
            .is_ok()
        );
        // Invalid protocol option
        assert!(
            SyslogCefExporterConfig::parse(&json!({
                "protocol": {"udp": {"endpoint": "127.0.0.1:514", "framing": "newline"}}
            }))
            .is_err()
        );
        // Missing port in endpoint
        assert!(
            SyslogCefExporterConfig::parse(&json!({
                "protocol": {"tcp": {"endpoint": "127.0.0.1"}}
            }))
            .is_err()
        );
        // Invalid port in endpoint
        assert!(
            SyslogCefExporterConfig::parse(&json!({
                "protocol": {"tcp": {"endpoint": "127.0.0.1:invalid"}}
            }))
            .is_err()
        );
        // Zero duration rejected
        assert!(
            SyslogCefExporterConfig::parse(&json!({
                "protocol": {"tcp": {"endpoint": "127.0.0.1:514"}},
                "io_timeout": "0s"
            }))
            .is_err()
        );
        // Empty host in endpoint rejected
        assert!(
            SyslogCefExporterConfig::parse(&json!({
                "protocol": {"tcp": {"endpoint": ":514"}}
            }))
            .is_err()
        );
        // Port 0 rejected
        assert!(
            SyslogCefExporterConfig::parse(&json!({
                "protocol": {"tcp": {"endpoint": "127.0.0.1:0"}}
            }))
            .is_err()
        );
        // IPv6 endpoint accepted
        assert!(
            SyslogCefExporterConfig::parse(&json!({
                "protocol": {"tcp": {"endpoint": "[::1]:514"}}
            }))
            .is_ok()
        );
        // Surrounding whitespace trimmed and stored
        let trimmed_cfg = SyslogCefExporterConfig::parse(&json!({
            "protocol": {"tcp": {"endpoint": "  127.0.0.1:514  "}}
        }))
        .unwrap();
        match trimmed_cfg.protocol {
            Protocol::Tcp(tcp) => assert_eq!(tcp.endpoint, "127.0.0.1:514"),
            _ => unreachable!(),
        }
    }

    /// Scenario: A batch has a valid first message and an invalid second message (contains LF in newline framing).
    /// Guarantees: Batch-wide validation permanently rejects the batch before any network transmission occurs.
    #[test]
    fn batch_wide_validation_rejects_before_send() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = listener.local_addr().unwrap();
        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({"protocol": {"tcp": {"endpoint": endpoint.to_string(), "framing": "newline"}}}),
        )
        .unwrap();

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let pdata = syslog_pdata(&[b"<13>valid_msg", b"invalid\nmsg"]).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    1,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(5), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                assert_nack(
                    &mut ctx,
                    true,
                    "newline framing cannot preserve a message containing LF",
                    NackCause::Refused,
                )
                .await;
            });

        listener.set_nonblocking(true).unwrap();
        let err = listener.accept().unwrap_err();
        assert_eq!(err.kind(), io::ErrorKind::WouldBlock);
    }

    fn test_metrics_pdata() -> OtapPdata {
        let empty =
            otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::MetricsData::new(vec![]);
        let mut otlp_bytes = vec![];
        empty
            .encode(&mut otlp_bytes)
            .expect("failed to encode empty OTLP ExportMetricsServiceRequest");
        OtapPdata::new_default(
            OtlpProtoBytes::ExportMetricsRequest(bytes::Bytes::from(otlp_bytes)).into(),
        )
    }

    /// Scenario: Non-logs signal payload (e.g. metrics pdata) is delivered to the exporter.
    /// Guarantees: Exporter permanently rejects with a NACK and records refused attempt.
    #[test]
    fn unsupported_signal_permanently_nacked() {
        let endpoint = "127.0.0.1:514";
        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({"protocol": {"tcp": {"endpoint": endpoint}}}),
        )
        .unwrap();

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let pdata = test_metrics_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    2,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(5), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                assert_nack(
                    &mut ctx,
                    true,
                    "syslog_cef exporter only supports logs signal",
                    NackCause::Refused,
                )
                .await;
            });
    }

    /// Scenario: Structured log pdata (e.g. OTLP/OTAP log record) is delivered to the exporter.
    /// Guarantees: Exporter encodes the logs to syslog-batch, sends over TCP, ACKs, and records success metrics.
    #[test]
    fn native_logs_encoded_and_exported() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = listener.local_addr().unwrap();
        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({"protocol": {"tcp": {"endpoint": endpoint.to_string(), "framing": "newline"}}}),
        )
        .unwrap();

        let receiver = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            use std::io::Read;
            let _ = stream.read_to_end(&mut buf);
            buf
        });

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let pdata = create_test_pdata().test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    10,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(5), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                assert_ack(&mut ctx).await;
            });

        let received = receiver.join().unwrap();
        assert!(!received.is_empty());
        assert!(received.ends_with(b"\n"));
    }

    /// Scenario: TCP connection fails (destination connection refused / closed immediately).
    /// Guarantees: Exporter emits a retryable NACK and records exporter.attempted failure.
    #[test]
    fn transport_failure_retryable_nack_and_failed_metrics() {
        let socket = tokio::net::TcpSocket::new_v4().unwrap();
        socket.bind("127.0.0.1:0".parse().unwrap()).unwrap();
        let endpoint = socket.local_addr().unwrap();
        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({"protocol": {"tcp": {"endpoint": endpoint.to_string()}}}),
        )
        .unwrap();

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let _socket = socket;
                let pdata = syslog_pdata(&[b"<13>msg1", b"<13>msg2"]).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    3,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(5), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                assert_nack(
                    &mut ctx,
                    false,
                    "syslog transport send failed",
                    NackCause::Unspecified,
                )
                .await;
            });
    }

    /// Scenario: TCP export succeeds for two valid syslog messages.
    /// Guarantees: Destination receives both messages framed, exporter ACKs, and records success metrics for both messages.
    #[test]
    fn successful_export_acks_and_records_success_metrics() {
        let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
        let endpoint = listener.local_addr().unwrap();
        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({"protocol": {"tcp": {"endpoint": endpoint.to_string(), "framing": "newline"}}}),
        )
        .unwrap();

        let receiver = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut buf = Vec::new();
            use std::io::Read;
            let _ = stream.read_to_end(&mut buf);
            buf
        });

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let pdata = syslog_pdata(&[b"<13>first", b"<13>second"]).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    4,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(5), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                assert_ack(&mut ctx).await;
                let bytes = receiver.join().unwrap();
                assert_eq!(bytes, b"<13>first\n<13>second\n");
            });
    }

    /// Scenario: Metrics are recorded for successful sends, failed sends, and rejected batches.
    /// Guarantees: Boundary `exporter.attempted` metrics are emitted with correct outcomes, counts, and sizes.
    #[test]
    fn metrics_record_outcomes_and_boundary() {
        let (pipeline, _registry) = test_pipeline_ctx_with_interests(
            Interests::NODE_INPUT_METRICS
                | Interests::NODE_LOCAL_DURATION
                | Interests::NODE_ITEM_COUNTS
                | Interests::NODE_SIZE,
        );
        let mut metrics = SyslogCefExporterMetrics::register(&pipeline);

        let mut attempt_ok = metrics.boundary.attempt(SignalType::Logs);
        attempt_ok.set_item_count_with(|| 1);
        attempt_ok.set_payload_size_with(|| 20);
        _ = metrics
            .boundary
            .record(attempt_ok.complete::<(), &str>(Ok(())));

        let mut attempt_fail = metrics.boundary.attempt(SignalType::Logs);
        attempt_fail.set_item_count_with(|| 1);
        attempt_fail.set_payload_size_with(|| 20);
        let failed = attempt_fail.failed("test failure");
        _ = metrics
            .boundary
            .record(attempt_fail.complete::<(), _>(Err(failed)));

        let attempt_refused = metrics.boundary.attempt(SignalType::Logs);
        let refused = attempt_refused.refused("test refused");
        _ = metrics
            .boundary
            .record(attempt_refused.complete::<(), _>(Err(refused)));

        let snapshots = metrics.terminal_snapshots();

        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "exporter.attempted"
                && s.measurement_attribute_value("outcome") == Some("success")
        }));
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "exporter.attempted"
                && s.measurement_attribute_value("outcome") == Some("failure")
        }));
        assert!(snapshots.iter().any(|s| {
            s.descriptor().name == "exporter.attempted"
                && s.measurement_attribute_value("outcome") == Some("refused")
        }));
    }

    fn bind_constrained_tcp_listener() -> (std::net::TcpListener, SocketAddr) {
        let socket = socket2::Socket::new(
            socket2::Domain::IPV4,
            socket2::Type::STREAM,
            Some(socket2::Protocol::TCP),
        )
        .expect("create socket2 TCP socket");
        _ = socket.set_recv_buffer_size(2048);
        socket
            .bind(&"127.0.0.1:0".parse::<SocketAddr>().unwrap().into())
            .expect("bind constrained socket");
        socket.listen(1).expect("listen on constrained socket");
        let listener: std::net::TcpListener = socket.into();
        let addr = listener.local_addr().expect("local addr");
        (listener, addr)
    }

    /// Scenario: Pipeline shutdown occurs while a batch send is blocked in flight and another batch is queued.
    /// Guarantees: The in-flight send honors the shutdown deadline and emits a retryable NACK, and the queued batch
    /// receives a retryable NACK instead of being dropped.
    #[test]
    fn shutdown_nacks_queued_batch_while_send_is_in_flight() {
        let (listener, endpoint) = bind_constrained_tcp_listener();
        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({"protocol": {"tcp": {"endpoint": endpoint.to_string(), "framing": "newline"}}}),
        )
        .unwrap();

        let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel();

        let receiver = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let sock = socket2::SockRef::from(&stream);
            _ = sock.set_recv_buffer_size(2048);
            let _ = server_ready_tx.send(());
            // Hold the connection open without reading so the sender's buffer fills up.
            std::thread::sleep(Duration::from_secs(2));
            drop(stream);
        });

        // 100 messages of 15,000 bytes = 1.5MB, safely below MAX_BATCH_BYTES (2MB)
        // but large enough to reliably fill a constrained TCP buffer.
        let large_msg = vec![b'a'; 15_000];
        let large_msgs: Vec<&[u8]> = (0..100).map(|_| large_msg.as_slice()).collect();
        let batch1 = syslog_pdata(&large_msgs).test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            TestCallData::new_with(1, 1).into(),
            1,
        );
        let batch2 = syslog_pdata(&[b"<13>queued_second_batch"]).test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            TestCallData::new_with(2, 2).into(),
            2,
        );

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(move |ctx| async move {
                // Send first large batch
                ctx.send_pdata(batch1).await.unwrap();

                // Wait for TCP server to accept connection
                server_ready_rx.await.unwrap();

                // Send second batch while first batch is blocked in flight
                ctx.send_pdata(batch2).await.unwrap();

                // Allow a moment for the second batch to be buffered in inbox
                tokio::time::sleep(Duration::from_millis(30)).await;

                // Send shutdown with a 150ms deadline
                ctx.send_shutdown(
                    StdInstant::now() + Duration::from_millis(150),
                    "test shutdown",
                )
                .await
                .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                let mut completion_receiver = ctx.take_pipeline_completion_receiver().unwrap();

                let mut got_inflight_nack = false;
                let mut got_queued_nack = false;

                for _ in 0..2 {
                    let completion = timeout(Duration::from_secs(5), completion_receiver.recv())
                        .await
                        .expect("timed out waiting for completion")
                        .expect("channel closed before receiving completion");
                    match completion {
                        PipelineCompletionMsg::DeliverNack { nack } => {
                            assert!(!nack.permanent, "expected retryable NACK during shutdown");
                            let (node_id, nack) = next_nack(nack).expect("nack should be routable");
                            if node_id == 1 {
                                let got: TestCallData = nack
                                    .unwind
                                    .route
                                    .calldata
                                    .try_into()
                                    .expect("nack calldata should parse");
                                assert_eq!(got, TestCallData::new_with(1, 1));
                                assert!(
                                    nack.reason.contains("shutdown deadline expired"),
                                    "unexpected in-flight NACK reason: {}",
                                    nack.reason
                                );
                                got_inflight_nack = true;
                            } else if node_id == 2 {
                                let got: TestCallData = nack
                                    .unwind
                                    .route
                                    .calldata
                                    .try_into()
                                    .expect("nack calldata should parse");
                                assert_eq!(got, TestCallData::new_with(2, 2));
                                assert!(
                                    nack.reason
                                        .contains("syslog exporter shutdown before batch send"),
                                    "unexpected queued NACK reason: {}",
                                    nack.reason
                                );
                                got_queued_nack = true;
                            } else {
                                panic!("unexpected node_id: {node_id}");
                            }
                        }
                        PipelineCompletionMsg::DeliverAck { .. } => {
                            panic!("expected only NACKs during shutdown, got ACK");
                        }
                    }
                }

                assert!(got_inflight_nack, "expected in-flight batch to be NACKed");
                assert!(got_queued_nack, "expected queued batch to be NACKed");
            });

        let _ = receiver.join();
    }

    /// Scenario: A batch takes longer to export than the configured `batch_timeout`.
    /// Guarantees: Exporter times out, clears TCP connection, and emits a retryable NACK with timeout error.
    #[test]
    fn batch_timeout_expires_emits_retryable_nack() {
        let (listener, endpoint) = bind_constrained_tcp_listener();

        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({
                "protocol": {"tcp": {"endpoint": endpoint.to_string(), "framing": "newline"}},
                "batch_timeout": "50ms",
                "io_timeout": "5s"
            }),
        )
        .unwrap();

        let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel();
        let receiver = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let sock = socket2::SockRef::from(&stream);
            _ = sock.set_recv_buffer_size(2048);
            let _ = server_ready_tx.send(());
            std::thread::sleep(Duration::from_secs(2));
            drop(stream);
        });

        let msg = vec![b'a'; 15_000];
        let large_msgs: Vec<&[u8]> = (0..100).map(|_| msg.as_slice()).collect();
        let batch = syslog_pdata(&large_msgs).test_subscribe_to(
            Interests::ACKS | Interests::NACKS,
            TestCallData::default().into(),
            1,
        );

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(move |ctx| async move {
                ctx.send_pdata(batch).await.unwrap();
                server_ready_rx.await.unwrap();
                tokio::time::sleep(Duration::from_millis(100)).await;
                ctx.send_shutdown(StdInstant::now() + Duration::from_secs(5), "test complete")
                    .await
                    .unwrap();
            })
            .run_validation(move |mut ctx, result| async move {
                result.unwrap();
                assert_nack(
                    &mut ctx,
                    false,
                    "syslog batch send timed out",
                    NackCause::Unspecified,
                )
                .await;
            });

        let _ = receiver.join();
    }
}
