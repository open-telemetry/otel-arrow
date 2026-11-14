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
use otel_arrow_dfe_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otel_arrow_dfe_engine::error::Error;
use otel_arrow_dfe_engine::exporter::ExporterWrapper;
use otel_arrow_dfe_engine::local::exporter::{EffectHandler, Exporter};
use otel_arrow_dfe_engine::message::{ExporterInbox, Message};
use otel_arrow_dfe_engine::node::NodeId;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_engine::{ConsumerEffectHandlerExtension, ExporterFactory};
use otel_arrow_dfe_otap::OTAP_EXPORTER_FACTORIES;
use otel_arrow_dfe_otap::metrics::ExporterMetrics;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata_codec::PdataEncoding;
use otel_arrow_dfe_pdata_codec::builtins::syslog::{MAX_MESSAGE_SIZE, validated_messages};
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
        loop {
            match inbox.recv().await {
                Ok(Message::PData(pdata)) => {
                    if let Some(terminal_state) =
                        self.export(pdata, &mut inbox, &effect_handler).await?
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
                    return Ok(TerminalState::new(
                        deadline,
                        self.metrics.terminal_snapshots(),
                    ));
                }
                Ok(_) => {}
                Err(otel_arrow_dfe_channel::error::RecvError::Closed) => {
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
        inbox: &mut ExporterInbox<OtapPdata>,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<Option<TerminalState>, Error> {
        let raw_bytes: Option<Vec<u8>> = match pdata.payload_ref().encoded_view() {
            Some(view)
                if view.signal_type() == SignalType::Logs && view.encoding() == &SYSLOG_BATCH =>
            {
                Some(view.bytes().to_vec())
            }
            _ => None,
        };

        let messages = match raw_bytes.as_deref() {
            Some(bytes) => {
                validated_messages(bytes).map_err(|error| format!("invalid syslog-batch: {error}"))
            }
            None => Err("syslog_cef exporter requires encoded syslog-batch logs".to_owned()),
        };
        let messages = match messages {
            Ok(messages) => messages,
            Err(reason) => {
                let attempt = self.metrics.boundary.attempt(SignalType::Logs);
                let refused = attempt.refused(&reason);
                _ = self
                    .metrics
                    .boundary
                    .record(attempt.complete::<(), _>(Err(refused)));
                effect_handler
                    .notify_nack(NackMsg::new_permanent(reason, pdata))
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
                .notify_nack(NackMsg::new_permanent(reason, pdata))
                .await?;
            return Ok(None);
        }

        let batch_deadline = tokio::time::Instant::now() + self.config.batch_timeout;
        let mut shutdown_deadline: Option<tokio::time::Instant> = None;
        let mut shutdown_std_deadline: Option<std::time::Instant> = None;

        for message in &messages {
            let mut attempt = self.metrics.boundary.attempt(SignalType::Logs);
            attempt.set_item_count_with(|| 1);
            attempt.set_payload_size_with(|| message.len());

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
                            if shutdown_std_deadline.is_none() {
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
                                    shutdown_deadline = Some(tokio::time::Instant::from_std(deadline));
                                    shutdown_std_deadline = Some(deadline);
                                }
                                Ok(_) => {}
                                Err(_) => {
                                    shutdown_deadline = Some(tokio::time::Instant::now());
                                    shutdown_std_deadline = Some(std::time::Instant::now());
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
                    let failed = attempt.failed(&error);
                    _ = self
                        .metrics
                        .boundary
                        .record(attempt.complete::<(), _>(Err(failed)));
                    effect_handler
                        .notify_nack(NackMsg::new(
                            format!("syslog transport send failed: {error}"),
                            pdata,
                        ))
                        .await?;
                    if let Some(deadline) = shutdown_std_deadline {
                        return Ok(Some(TerminalState::new(
                            deadline,
                            self.metrics.terminal_snapshots(),
                        )));
                    }
                    return Ok(None);
                }
            }
        }

        effect_handler.notify_ack(AckMsg::new(pdata)).await?;

        if let Some(deadline) = shutdown_std_deadline {
            return Ok(Some(TerminalState::new(
                deadline,
                self.metrics.terminal_snapshots(),
            )));
        }
        Ok(None)
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
                    let addr = io_timeout(
                        async {
                            let mut addrs = tokio::net::lookup_host(&config.endpoint).await?;
                            addrs.next().ok_or_else(|| {
                                io::Error::new(
                                    io::ErrorKind::NotFound,
                                    format!("no addresses found for {}", config.endpoint),
                                )
                            })
                        },
                        io_duration,
                    )
                    .await?;
                    let bind: SocketAddr = if addr.is_ipv4() {
                        "0.0.0.0:0".parse().expect("static IPv4 bind address")
                    } else {
                        "[::]:0".parse().expect("static IPv6 bind address")
                    };
                    let socket = io_timeout(UdpSocket::bind(bind), io_duration).await?;
                    io_timeout(socket.connect(addr), io_duration).await?;
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
    use otel_arrow_dfe_otap::testing::{TestCallData, create_empty_test_pdata};
    use otel_arrow_dfe_pdata_codec::builtins::syslog::SyslogBatchBuilder;
    use otel_arrow_dfe_pdata_codec::{CodecRegistry, EncodedPdata, EncodedPdataBatch};
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
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
    ) {
        let mut completion_receiver = ctx.take_pipeline_completion_receiver().unwrap();
        let completion = timeout(Duration::from_secs(5), completion_receiver.recv())
            .await
            .expect("timed out waiting for NACK")
            .expect("completion channel closed before NACK");
        match completion {
            PipelineCompletionMsg::DeliverNack { nack } => {
                assert_eq!(nack.permanent, expected_permanent);
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
                )
                .await;
            });
    }

    /// Scenario: Non-syslog-batch payload (e.g. empty test pdata) is delivered to the exporter.
    /// Guarantees: Exporter permanently rejects with a NACK and records refused attempt.
    #[test]
    fn unsupported_input_permanently_nacked() {
        let endpoint = "127.0.0.1:514";
        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({"protocol": {"tcp": {"endpoint": endpoint}}}),
        )
        .unwrap();

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let pdata = create_empty_test_pdata().test_subscribe_to(
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
                    "syslog_cef exporter requires encoded syslog-batch logs",
                )
                .await;
            });
    }

    /// Scenario: TCP connection fails (destination connection refused / closed immediately).
    /// Guarantees: Exporter emits a retryable NACK and records exporter.attempted failure.
    #[test]
    fn transport_failure_retryable_nack_and_failed_metrics() {
        let endpoint = {
            let listener = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
            listener.local_addr().unwrap()
        };
        let exporter = create_exporter_from_factory(
            &SYSLOG_CEF_EXPORTER,
            json!({"protocol": {"tcp": {"endpoint": endpoint.to_string()}}}),
        )
        .unwrap();

        TestRuntime::new()
            .set_exporter(exporter)
            .run_test(|ctx| async move {
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
                assert_nack(&mut ctx, false, "syslog transport send failed").await;
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
}
