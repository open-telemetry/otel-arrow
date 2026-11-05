// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Syslog/CEF encoded-batch passthrough exporter.

otel_arrow_dfe_telemetry::otel_component_scope!(
    urn = SYSLOG_EXPORTER_URN,
    target = "otel.exporter.syslog",
);

use async_trait::async_trait;
use linkme::distributed_slice;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::node::NodeUserConfig;
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
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata_codec::PdataEncoding;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::MetricSet;
use otel_arrow_dfe_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::future::Future;
use std::io;
use std::net::SocketAddr;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::net::{TcpStream, UdpSocket};
use tokio::time::timeout;

use crate::receivers::syslog_cef_receiver::{MAX_MESSAGE_SIZE, validated_messages};

/// Component URN for the syslog exporter.
pub const SYSLOG_EXPORTER_URN: &str = "urn:otel:exporter:syslog";
const SYSLOG_BATCH: PdataEncoding = PdataEncoding::new("syslog-batch");
const IO_TIMEOUT: Duration = Duration::from_secs(5);
const BATCH_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
enum TcpFraming {
    #[default]
    Newline,
    OctetCounting,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct TcpConfig {
    endpoint: SocketAddr,
    #[serde(default)]
    framing: TcpFraming,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UdpConfig {
    endpoint: SocketAddr,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Protocol {
    Tcp(TcpConfig),
    Udp(UdpConfig),
}

/// Configuration for a single TCP or UDP syslog destination.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SyslogExporterConfig {
    protocol: Protocol,
}

impl SyslogExporterConfig {
    fn parse(value: &Value) -> Result<Self, ConfigError> {
        serde_json::from_value(value.clone()).map_err(|error| ConfigError::InvalidUserConfig {
            error: format!("Invalid syslog exporter config: {error}"),
        })
    }
}

/// Locally accepted and failed syslog messages and their original payload bytes.
#[metric_set(name = "exporter.syslog")]
#[derive(Debug, Default, Clone)]
struct SyslogMetrics {
    #[metric(unit = "{message}")]
    sent_messages: Counter<u64>,
    #[metric(unit = "By")]
    sent_bytes: Counter<u64>,
    #[metric(unit = "{message}")]
    failed_messages: Counter<u64>,
    #[metric(unit = "By")]
    failed_bytes: Counter<u64>,
    #[metric(unit = "{batch}")]
    rejected_batches: Counter<u64>,
}

/// One connection per pipeline-local exporter, with no additional send queue.
pub struct SyslogExporter {
    config: SyslogExporterConfig,
    tcp: Option<TcpStream>,
    udp: Option<UdpSocket>,
    metrics: MetricSet<SyslogMetrics>,
}

/// Declares the syslog exporter as a local exporter factory.
#[allow(unsafe_code)]
#[otel_arrow_dfe_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static SYSLOG_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: SYSLOG_EXPORTER_URN,
    create:
        |pipeline: PipelineContext,
         node: NodeId,
         node_config: Arc<NodeUserConfig>,
         exporter_config: &ExporterConfig,
         _capabilities: &otel_arrow_dfe_engine::capability::registry::Capabilities| {
            let config = SyslogExporterConfig::parse(&node_config.config)?;
            Ok(ExporterWrapper::local(
                SyslogExporter {
                    config,
                    tcp: None,
                    udp: None,
                    metrics: SyslogMetrics::register(&pipeline),
                },
                node,
                node_config,
                exporter_config,
            ))
        },
    context_declarations: None,
    wiring_contract: otel_arrow_dfe_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: |value| SyslogExporterConfig::parse(value).map(|_| ()),
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for SyslogExporter {
    async fn start(
        mut self: Box<Self>,
        mut inbox: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        loop {
            match inbox.recv().await? {
                Message::PData(pdata) => self.export(pdata, &effect_handler).await?,
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    // Dropping the connection closes it; there is no background send task.
                    return Ok(TerminalState::new(
                        deadline,
                        self.metrics.terminal_snapshots(),
                    ));
                }
                _ => {}
            }
        }
    }
}

impl SyslogExporter {
    async fn export(
        &mut self,
        pdata: OtapPdata,
        effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let view = pdata.payload_ref().encoded_view();
        let messages = match view.as_ref() {
            Some(view)
                if view.signal_type() == SignalType::Logs && view.encoding() == &SYSLOG_BATCH =>
            {
                validated_messages(view.bytes())
                    .map_err(|error| format!("invalid syslog-batch: {error}"))
            }
            _ => Err("syslog exporter requires encoded syslog-batch logs".to_owned()),
        };
        let messages = match messages {
            Ok(messages) => messages,
            Err(reason) => {
                self.metrics.rejected_batches.inc();
                effect_handler
                    .notify_nack(NackMsg::new_permanent(reason, pdata))
                    .await?;
                return Ok(());
            }
        };
        if let Err(reason) = self.validate_transport(&messages) {
            self.metrics.rejected_batches.inc();
            effect_handler
                .notify_nack(NackMsg::new_permanent(reason, pdata))
                .await?;
            return Ok(());
        }

        let mut sent = 0;
        let result = timeout(BATCH_TIMEOUT, async {
            for message in &messages {
                self.send_message(message).await?;
                self.metrics.sent_messages.inc();
                self.metrics.sent_bytes.add(message.len() as u64);
                sent += 1;
            }
            Ok::<(), io::Error>(())
        })
        .await
        .map_err(|_| io::Error::new(io::ErrorKind::TimedOut, "syslog batch send timed out"))
        .and_then(|result| result);
        if let Err(error) = result {
            // A TCP write may have reached the peer only in part. Discard the
            // stream before any retry, and report the remainder as failed.
            self.tcp = None;
            self.metrics
                .failed_messages
                .add((messages.len() - sent) as u64);
            self.metrics.failed_bytes.add(
                messages[sent..]
                    .iter()
                    .map(|message| message.len() as u64)
                    .sum(),
            );
            effect_handler
                .notify_nack(NackMsg::new(
                    format!("syslog transport send failed: {error}"),
                    pdata,
                ))
                .await?;
            return Ok(());
        }
        effect_handler.notify_ack(AckMsg::new(pdata)).await?;
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

    async fn send_message(&mut self, message: &[u8]) -> io::Result<()> {
        match &self.config.protocol {
            Protocol::Tcp(config) => {
                if self.tcp.is_none() {
                    self.tcp = Some(io_timeout(TcpStream::connect(config.endpoint)).await?);
                }
                let stream = self.tcp.as_mut().expect("connected TCP stream");
                match config.framing {
                    TcpFraming::Newline => {
                        io_timeout(stream.write_all(message)).await?;
                        io_timeout(stream.write_all(b"\n")).await?;
                    }
                    TcpFraming::OctetCounting => {
                        let prefix = format!("{} ", message.len());
                        io_timeout(stream.write_all(prefix.as_bytes())).await?;
                        io_timeout(stream.write_all(message)).await?;
                    }
                }
            }
            Protocol::Udp(config) => {
                if self.udp.is_none() {
                    let bind: SocketAddr = if config.endpoint.is_ipv4() {
                        "0.0.0.0:0".parse().expect("static IPv4 bind address")
                    } else {
                        "[::]:0".parse().expect("static IPv6 bind address")
                    };
                    let socket = io_timeout(UdpSocket::bind(bind)).await?;
                    io_timeout(socket.connect(config.endpoint)).await?;
                    self.udp = Some(socket);
                }
                let sent = io_timeout(
                    self.udp
                        .as_ref()
                        .expect("connected UDP socket")
                        .send(message),
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

async fn io_timeout<T>(future: impl Future<Output = io::Result<T>>) -> io::Result<T> {
    timeout(IO_TIMEOUT, future).await.map_err(|_| {
        io::Error::new(
            io::ErrorKind::TimedOut,
            "syslog transport operation timed out",
        )
    })?
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_engine::context::ControllerContext;
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use serde_json::json;
    use tokio::io::AsyncReadExt;
    use tokio::net::TcpListener;

    fn exporter(protocol: Protocol) -> SyslogExporter {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline = controller.pipeline_context_with("grp".into(), "syslog".into(), 0, 1, 0);
        SyslogExporter {
            config: SyslogExporterConfig { protocol },
            tcp: None,
            udp: None,
            metrics: SyslogMetrics::register(&pipeline),
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
            endpoint,
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
            endpoint,
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
        let mut exporter = exporter(Protocol::Udp(UdpConfig { endpoint }));
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
        let endpoint = "127.0.0.1:514".parse().unwrap();
        let newline = exporter(Protocol::Tcp(TcpConfig {
            endpoint,
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

    /// Scenario: Config selects supported transports and rejects invalid fields.
    /// Guarantees: Only a single explicit destination and known framing mode are accepted.
    #[test]
    fn validates_config() {
        assert!(
            SyslogExporterConfig::parse(&json!({
                "protocol": {"tcp": {"endpoint": "127.0.0.1:514", "framing": "octet_counting"}}
            }))
            .is_ok()
        );
        assert!(
            SyslogExporterConfig::parse(&json!({
                "protocol": {"udp": {"endpoint": "127.0.0.1:514", "framing": "newline"}}
            }))
            .is_err()
        );
    }
}
