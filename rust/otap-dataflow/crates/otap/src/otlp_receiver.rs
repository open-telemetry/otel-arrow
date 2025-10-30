// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::OTAP_RECEIVER_FACTORIES;
use crate::otap_grpc::otlp::server::{
    LogsServiceServer, MetricsServiceServer, RouteResponse, Settings, SharedState,
    TraceServiceServer,
};
use crate::pdata::OtapPdata;

use crate::compression::CompressionMethod;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::byte_units;
use otap_df_config::experimental::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ReceiverErrorKind, format_error_sources};
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::shared::receiver as shared;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tonic::codec::EnabledCompressionEncodings;
use tonic::transport::Server;
use tonic::transport::server::TcpIncoming;

/// URN for the OTLP Receiver
pub const OTLP_RECEIVER_URN: &str = "urn:otel:otlp:receiver";

/// Configuration for OTLP Receiver
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The endpoint details: protocol, name, port.
    listening_addr: SocketAddr,

    /// Compression methods accepted (only used for requests, responses are not compressed as they
    /// are typically small).
    compression_method: Option<CompressionMethod>,

    // --- All the following settings have defaults that should be reasonable for most users ---
    // -----------------------------------------------------------------------------------------
    /// Maximum number of concurrent in-flight requests.
    /// Defaults to `0`, which means the receiver adopts the downstream pdata channel capacity so
    /// backpressure flows upstream automatically. Any non-zero value is still clamped to that
    /// capacity at runtime.
    #[serde(default = "default_max_concurrent_requests")]
    max_concurrent_requests: usize,

    /// Whether newly accepted sockets should have `TCP_NODELAY` enabled.
    /// Keeping this `true` (the default) avoids Nagle's algorithm and minimizes per-export latency.
    /// Disabling it trades slightly higher latency for fewer small TCP packets when workloads
    /// involve very bursty, tiny messages.
    #[serde(default = "default_tcp_nodelay")]
    tcp_nodelay: bool,

    /// TCP keepalive timeout for accepted sockets.
    /// The 45s default evicts dead clients in under a minute without incurring much background
    /// traffic. Raise it to reduce keepalive chatter, or set to `null` to disable kernel keepalives
    /// entirely (at the cost of slower leak detection on broken links).
    #[serde(default = "default_tcp_keepalive", with = "humantime_serde")]
    tcp_keepalive: Option<Duration>,

    /// Interval between TCP keepalive probes once keepalive is active.
    /// Defaults to 15s so the kernel confirms progress quickly after the keepalive timeout. Longer
    /// intervals reduce packets, shorter intervals detect stalled peers faster. Ignored if
    /// `tcp_keepalive` is `null`.
    #[serde(default = "default_tcp_keepalive_interval", with = "humantime_serde")]
    tcp_keepalive_interval: Option<Duration>,

    /// Number of TCP keepalive probes sent before a connection is declared dead.
    /// The default (5) balances resilience to transient loss with timely reclamation of resources.
    /// Smaller values clean up faster during outages, larger values favor noisy or lossy networks.
    #[serde(default = "default_tcp_keepalive_retries")]
    tcp_keepalive_retries: Option<u32>,

    /// Per-connection concurrency limit enforced by the transport layer.
    /// By default it mirrors the effective `max_concurrent_requests`, so transport- and
    /// application-level backpressure remain aligned. Lower values gate connection bursts earlier,
    /// while higher values only help if you also raise `max_concurrent_requests`. Set to `0` to
    /// revert to the derived default.
    #[serde(default)]
    transport_concurrency_limit: Option<usize>,

    /// Whether the gRPC server should shed load immediately once concurrency limits are hit.
    /// Leaving this `true` (default) results in fast `resource_exhausted` responses and protects
    /// the single-threaded runtime from unbounded queues. Turning it off allows requests to queue
    /// but increases memory usage and tail latency under sustained overload.
    #[serde(default = "default_load_shed")]
    load_shed: bool,

    /// Initial HTTP/2 stream window size, in bytes.
    /// Accepts plain integers or suffixed strings such as `8MiB`. The default 8MiB window reduces
    /// flow-control stalls for large OTLP batches; trimming it lowers per-stream memory but may
    /// throttle throughput, while increasing it benefits high-bandwidth deployments at the cost of
    /// larger buffers.
    #[serde(
        default = "default_initial_stream_window_size",
        deserialize_with = "byte_units::deserialize"
    )]
    initial_stream_window_size: Option<u32>,

    /// Initial HTTP/2 connection window size, in bytes.
    /// Accepts plain integers or suffixed strings such as `32MiB`. Defaults to 32MiB, giving room
    /// for several simultaneous large streams; adjust using the same trade-offs as the stream
    /// window but applied per connection.
    #[serde(
        default = "default_initial_connection_window_size",
        deserialize_with = "byte_units::deserialize"
    )]
    initial_connection_window_size: Option<u32>,

    /// Whether to rely on HTTP/2 adaptive window sizing instead of the manual values above.
    /// Disabled by default so the receiver uses predictable static windows. Enabling this lets tonic
    /// adjust flow-control windows dynamically, which can improve throughput on high-bandwidth links
    /// but makes memory usage and latency more workload dependent (and largely ignores the window
    /// sizes configured above).
    #[serde(default = "default_http2_adaptive_window")]
    http2_adaptive_window: bool,

    /// Maximum HTTP/2 frame size, in bytes.
    /// Accepts plain integers or suffixed strings such as `16KiB`. The 16KiB default matches the
    /// current tuning: large enough to keep framing overhead low for sizeable batches yet still
    /// bounded; larger values further decrease framing costs at the expense of bigger per-frame
    /// buffers, while smaller values force additional fragmentation and CPU work on jumbo exports.
    #[serde(
        default = "default_max_frame_size",
        deserialize_with = "byte_units::deserialize"
    )]
    max_frame_size: Option<u32>,

    /// Interval between HTTP/2 keepalive pings.
    /// The default 30s ping keeps intermediaries aware of idle-but-healthy connections. Shorten it
    /// to detect broken links faster, lengthen it to reduce ping traffic, or set to `null` to
    /// disable HTTP/2 keepalives.
    #[serde(default = "default_http2_keepalive_interval", with = "humantime_serde")]
    http2_keepalive_interval: Option<Duration>,

    /// Timeout waiting for an HTTP/2 keepalive acknowledgement.
    /// Defaults to 10s, balancing rapid detection of stalled peers with tolerance for transient
    /// network jitter. Decrease it for quicker failover or increase it for chatty-but-latent paths.
    #[serde(default = "default_http2_keepalive_timeout", with = "humantime_serde")]
    http2_keepalive_timeout: Option<Duration>,

    /// Upper bound on concurrently active HTTP/2 streams per connection.
    /// By default this tracks the effective `max_concurrent_requests`, keeping logical and transport
    /// concurrency aligned. Lower values improve fairness between chatty clients. Higher values
    /// matter only if you also raise `max_concurrent_requests`. Set to `0` to inherit the derived
    /// default.
    #[serde(default)]
    max_concurrent_streams: Option<u32>,

    /// Whether to wait for the result (default: false)
    ///
    /// When enabled, the receiver will not send a response until the
    /// immediate downstream component has acknowledged receipt of the
    /// data.  This does not guarantee that data has been fully
    /// processed or successfully exported to the final destination,
    /// since components are able acknowledge early.
    ///
    /// Note when wait_for_result=false, it is impossible to
    /// see a failure, errors are effectively suppressed.
    #[serde(default = "default_wait_for_result")]
    wait_for_result: bool,

    /// Timeout for RPC requests. If not specified, no timeout is applied.
    /// Format: humantime format (e.g., "30s", "5m", "1h", "500ms")
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,
}

const fn default_max_concurrent_requests() -> usize {
    0
}

const fn default_tcp_nodelay() -> bool {
    true
}

fn default_tcp_keepalive() -> Option<Duration> {
    Some(Duration::from_secs(45))
}

fn default_tcp_keepalive_interval() -> Option<Duration> {
    Some(Duration::from_secs(15))
}

fn default_tcp_keepalive_retries() -> Option<u32> {
    Some(5)
}

const fn default_load_shed() -> bool {
    true
}

fn default_initial_stream_window_size() -> Option<u32> {
    Some(8 * 1024 * 1024)
}

fn default_initial_connection_window_size() -> Option<u32> {
    Some(32 * 1024 * 1024)
}

fn default_max_frame_size() -> Option<u32> {
    Some(16 * 1024)
}

fn default_http2_keepalive_interval() -> Option<Duration> {
    Some(Duration::from_secs(30))
}

fn default_http2_keepalive_timeout() -> Option<Duration> {
    Some(Duration::from_secs(10))
}

const fn default_http2_adaptive_window() -> bool {
    false
}

const fn default_wait_for_result() -> bool {
    // See https://github.com/open-telemetry/otel-arrow/issues/1311
    // This matches the OTel Collector default for wait_for_result, presently.
    false
}

/// Receiver implementation that receives OTLP grpc service requests and decodes the data into OTAP.
pub struct OTLPReceiver {
    config: Config,
    metrics: MetricSet<OtlpReceiverMetrics>,
}

/// State shared between gRPC server task and the effect handler.
struct SharedStates {
    logs: Option<SharedState>,
    metrics: Option<SharedState>,
    traces: Option<SharedState>,
}

/// Declares the OTLP receiver as a shared receiver factory
///
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTLP_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTLP_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        let mut receiver = OTLPReceiver::from_config(pipeline, &node_config.config)?;
        receiver.tune_max_concurrent_requests(receiver_config.output_pdata_channel.capacity);

        Ok(ReceiverWrapper::shared(
            receiver,
            node,
            node_config,
            receiver_config,
        ))
    },
};

impl OTLPReceiver {
    /// Creates a new OTLPReceiver from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // Register OTLP receiver metrics for this node.
        let metrics = pipeline_ctx.register_metrics::<OtlpReceiverMetrics>();

        Ok(Self { config, metrics })
    }

    fn tune_max_concurrent_requests(&mut self, downstream_capacity: usize) {
        // Fall back to the downstream channel capacity when it is tighter than the user setting.
        let safe_capacity = downstream_capacity.max(1);
        if self.config.max_concurrent_requests == 0
            || self.config.max_concurrent_requests > safe_capacity
        {
            self.config.max_concurrent_requests = safe_capacity;
        }
    }

    fn route_ack_response(&self, states: &SharedStates, ack: AckMsg<OtapPdata>) -> RouteResponse {
        let calldata = ack.calldata;
        let resp = Ok(());
        let state = match ack.accepted.signal_type() {
            SignalType::Logs => states.logs.as_ref(),
            SignalType::Metrics => states.metrics.as_ref(),
            SignalType::Traces => states.traces.as_ref(),
        };

        state
            .map(|s| s.route_response(calldata, resp))
            .unwrap_or(RouteResponse::None)
    }

    fn route_nack_response(
        &self,
        states: &SharedStates,
        mut nack: NackMsg<OtapPdata>,
    ) -> RouteResponse {
        let calldata = std::mem::take(&mut nack.calldata);
        let signal_type = nack.refused.signal_type();
        let resp = Err(nack);
        let state = match signal_type {
            SignalType::Logs => states.logs.as_ref(),
            SignalType::Metrics => states.metrics.as_ref(),
            SignalType::Traces => states.traces.as_ref(),
        };

        state
            .map(|s| s.route_response(calldata, resp))
            .unwrap_or(RouteResponse::None)
    }

    fn handle_ack_response(&mut self, resp: RouteResponse) {
        match resp {
            RouteResponse::Sent => self.metrics.acks_sent.inc(),
            RouteResponse::Expired => self.metrics.acks_nacks_invalid_or_expired.inc(),
            RouteResponse::Invalid => self.metrics.acks_nacks_invalid_or_expired.inc(),
            RouteResponse::None => {}
        }
    }

    fn handle_nack_response(&mut self, resp: RouteResponse) {
        match resp {
            RouteResponse::Sent => self.metrics.nacks_sent.inc(),
            RouteResponse::Expired => self.metrics.acks_nacks_invalid_or_expired.inc(),
            RouteResponse::Invalid => self.metrics.acks_nacks_invalid_or_expired.inc(),
            RouteResponse::None => {}
        }
    }
}

/// OTLP receiver metrics.
//
// TODO: The following were unused, would have to be implemented in
// a different location:
//
// /// Number of bytes received.
// #[metric(unit = "By")]
// pub bytes_received: Counter<u64>,
// /// Number of messages received.
// #[metric(unit = "{msg}")]
// pub messages_received: Counter<u64>,
#[metric_set(name = "otlp.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct OtlpReceiverMetrics {
    /// Number of acks sent.
    #[metric(unit = "{acks}")]
    pub acks_sent: Counter<u64>,

    /// Number of nacks sent.
    #[metric(unit = "{nacks}")]
    pub nacks_sent: Counter<u64>,

    /// Number of invalid/expired acks/nacks.
    #[metric(unit = "{ack_or_nack}")]
    pub acks_nacks_invalid_or_expired: Counter<u64>,
}

#[async_trait]
impl shared::Receiver<OtapPdata> for OTLPReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel<OtapPdata>,
        effect_handler: shared::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // Make the receiver mutable so we can update metrics on telemetry collection.
        let listener = effect_handler.tcp_listener(self.config.listening_addr)?;
        let incoming = TcpIncoming::from(listener)
            .with_nodelay(Some(self.config.tcp_nodelay))
            .with_keepalive(self.config.tcp_keepalive)
            .with_keepalive_interval(self.config.tcp_keepalive_interval)
            .with_keepalive_retries(self.config.tcp_keepalive_retries);

        let mut compression = EnabledCompressionEncodings::default();
        let _ = self
            .config
            .compression_method
            .as_ref()
            .map(|method| compression.enable(method.map_to_compression_encoding()));

        let settings = Settings {
            max_concurrent_requests: self.config.max_concurrent_requests,
            wait_for_result: self.config.wait_for_result,
            accept_compression_encodings: compression,
            send_compression_encodings: EnabledCompressionEncodings::default(), // no response compression
        };

        let logs_server = LogsServiceServer::new(effect_handler.clone(), &settings);
        let metrics_server = MetricsServiceServer::new(effect_handler.clone(), &settings);
        let traces_server = TraceServiceServer::new(effect_handler.clone(), &settings);

        let states = SharedStates {
            logs: logs_server.common.state(),
            metrics: metrics_server.common.state(),
            traces: traces_server.common.state(),
        };

        let transport_limit = self
            .config
            .transport_concurrency_limit
            .and_then(|limit| if limit == 0 { None } else { Some(limit) })
            .unwrap_or(self.config.max_concurrent_requests)
            .max(1);

        let fallback_streams = self.config.max_concurrent_requests.min(u32::MAX as usize) as u32;

        let mut server_builder = Server::builder()
            .concurrency_limit_per_connection(transport_limit) // transport level
            .load_shed(self.config.load_shed)
            .initial_stream_window_size(self.config.initial_stream_window_size)
            .initial_connection_window_size(self.config.initial_connection_window_size)
            .max_frame_size(self.config.max_frame_size)
            .http2_adaptive_window(Some(self.config.http2_adaptive_window))
            .http2_keepalive_interval(self.config.http2_keepalive_interval)
            .http2_keepalive_timeout(self.config.http2_keepalive_timeout);

        let mut max_concurrent_streams = self
            .config
            .max_concurrent_streams
            .map(|value| if value == 0 { fallback_streams } else { value })
            .unwrap_or(fallback_streams);
        if max_concurrent_streams == 0 {
            max_concurrent_streams = 1;
        }
        server_builder = server_builder.max_concurrent_streams(Some(max_concurrent_streams));

        // Apply timeout if configured
        if let Some(timeout) = self.config.timeout {
            server_builder = server_builder.timeout(timeout);
        }

        let server = server_builder
            .add_service(logs_server)
            .add_service(metrics_server)
            .add_service(traces_server);

        // Start periodic telemetry collection
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        tokio::select! {
            biased;

            // Process internal events
            ctrl_msg_result = async {
                loop {
                    match ctrl_msg_recv.recv().await {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            let snapshot = self.metrics.snapshot();
                            _ = telemetry_cancel_handle.cancel().await;
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        },
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            // Report current receiver metrics.
                            _ = metrics_reporter.report(&mut self.metrics);
                        },
                        Ok(NodeControlMsg::Ack(ack)) => {
                            self.handle_ack_response(self.route_ack_response(&states, ack));
                        },
                        Ok(NodeControlMsg::Nack(nack)) => {
                            self.handle_nack_response(self.route_nack_response(&states, nack));
                        },
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message do nothing
                        }
                    }
                }
            } => {
                return ctrl_msg_result;
            },

            // Run server
            result = server.serve_with_incoming(incoming) => {
                if let Err(error) = result {
                    let source_detail = format_error_sources(&error);
                    return Err(Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Transport,
                        error: error.to_string(),
                        source_detail,
                    });
                }
            }
        }

        Ok(TerminalState::new(
            Instant::now().add(Duration::from_secs(1)),
            [self.metrics],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::pdata::OtlpProtoBytes;
    use crate::proto::opentelemetry::collector::logs::v1::logs_service_client::LogsServiceClient;
    use crate::proto::opentelemetry::collector::logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse,
    };
    use crate::proto::opentelemetry::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
    use crate::proto::opentelemetry::collector::metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
    };
    use crate::proto::opentelemetry::collector::trace::v1::trace_service_client::TraceServiceClient;
    use crate::proto::opentelemetry::collector::trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse,
    };
    use crate::proto::opentelemetry::common::v1::{InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use crate::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NackMsg;
    use otap_df_engine::control::{AckMsg, NodeControlMsg};
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use prost::Message;
    use std::pin::Pin;
    use std::time::{Duration, Instant};
    use tokio::time::timeout;

    fn test_config(addr: SocketAddr) -> Config {
        Config {
            listening_addr: addr,
            compression_method: None,
            max_concurrent_requests: 1000,
            tcp_nodelay: default_tcp_nodelay(),
            tcp_keepalive: default_tcp_keepalive(),
            tcp_keepalive_interval: default_tcp_keepalive_interval(),
            tcp_keepalive_retries: default_tcp_keepalive_retries(),
            transport_concurrency_limit: None,
            load_shed: default_load_shed(),
            initial_stream_window_size: default_initial_stream_window_size(),
            initial_connection_window_size: default_initial_connection_window_size(),
            max_frame_size: default_max_frame_size(),
            http2_adaptive_window: default_http2_adaptive_window(),
            http2_keepalive_interval: default_http2_keepalive_interval(),
            http2_keepalive_timeout: default_http2_keepalive_timeout(),
            max_concurrent_streams: None,
            wait_for_result: true,
            timeout: None,
        }
    }

    fn create_logs_service_request() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "a".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        attributes: vec![KeyValue {
                            key: "b".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    log_records: vec![
                        LogRecord {
                            time_unix_nano: 1,
                            attributes: vec![KeyValue {
                                key: "c".to_string(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                        LogRecord {
                            time_unix_nano: 2,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn create_metrics_service_request() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn create_traces_service_request() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![
                    ScopeSpans {
                        ..Default::default()
                    },
                    ScopeSpans {
                        ..Default::default()
                    },
                ],
                schema_url: "opentelemetry.io/schema/traces".to_string(),
            }],
        }
    }

    #[test]
    fn test_config_parsing() {
        use serde_json::json;

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let config_with_max_concurrent_requests = json!({
            "listening_addr": "127.0.0.1:4317",
            "max_concurrent_requests": 5000
        });
        let receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_with_max_concurrent_requests)
                .unwrap();
        assert_eq!(receiver.config.max_concurrent_requests, 5000);

        let config_default = json!({
            "listening_addr": "127.0.0.1:4317"
        });
        let receiver = OTLPReceiver::from_config(pipeline_ctx.clone(), &config_default).unwrap();
        assert_eq!(receiver.config.max_concurrent_requests, 0);
        assert!(receiver.config.tcp_nodelay);
        assert_eq!(receiver.config.tcp_keepalive, Some(Duration::from_secs(45)));
        assert_eq!(
            receiver.config.tcp_keepalive_interval,
            Some(Duration::from_secs(15))
        );
        assert_eq!(receiver.config.tcp_keepalive_retries, Some(5));
        assert_eq!(receiver.config.transport_concurrency_limit, None);
        assert!(receiver.config.load_shed);
        assert_eq!(
            receiver.config.initial_stream_window_size,
            Some(8 * 1024 * 1024)
        );
        assert_eq!(
            receiver.config.initial_connection_window_size,
            Some(32 * 1024 * 1024)
        );
        assert!(!receiver.config.http2_adaptive_window);
        assert_eq!(receiver.config.max_frame_size, Some(16 * 1024));
        assert_eq!(
            receiver.config.http2_keepalive_interval,
            Some(Duration::from_secs(30))
        );
        assert_eq!(
            receiver.config.http2_keepalive_timeout,
            Some(Duration::from_secs(10))
        );
        assert_eq!(receiver.config.max_concurrent_streams, None);

        let config_full = json!({
            "listening_addr": "127.0.0.1:4317",
            "compression_method": "gzip",
            "max_concurrent_requests": 2500
        });
        let receiver = OTLPReceiver::from_config(pipeline_ctx.clone(), &config_full).unwrap();
        assert_eq!(receiver.config.max_concurrent_requests, 2500);

        let config_with_server_overrides = json!({
            "listening_addr": "127.0.0.1:4317",
            "max_concurrent_requests": 512,
            "tcp_nodelay": false,
            "tcp_keepalive": "60s",
            "tcp_keepalive_interval": "20s",
            "tcp_keepalive_retries": 3,
            "transport_concurrency_limit": 256,
            "load_shed": false,
            "initial_stream_window_size": "4MiB",
            "initial_connection_window_size": "16MiB",
            "max_frame_size": "8MiB",
            "http2_keepalive_interval": "45s",
            "http2_keepalive_timeout": "20s",
            "max_concurrent_streams": 1024,
            "http2_adaptive_window": true
        });
        let receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_with_server_overrides).unwrap();
        assert_eq!(receiver.config.max_concurrent_requests, 512);
        assert!(!receiver.config.tcp_nodelay);
        assert_eq!(receiver.config.tcp_keepalive, Some(Duration::from_secs(60)));
        assert_eq!(
            receiver.config.tcp_keepalive_interval,
            Some(Duration::from_secs(20))
        );
        assert_eq!(receiver.config.tcp_keepalive_retries, Some(3));
        assert_eq!(receiver.config.transport_concurrency_limit, Some(256));
        assert!(!receiver.config.load_shed);
        assert_eq!(
            receiver.config.initial_stream_window_size,
            Some(4 * 1024 * 1024)
        );
        assert_eq!(
            receiver.config.initial_connection_window_size,
            Some(16 * 1024 * 1024)
        );
        assert_eq!(receiver.config.max_frame_size, Some(8 * 1024 * 1024));
        assert_eq!(
            receiver.config.http2_keepalive_interval,
            Some(Duration::from_secs(45))
        );
        assert_eq!(
            receiver.config.http2_keepalive_timeout,
            Some(Duration::from_secs(20))
        );
        assert_eq!(receiver.config.max_concurrent_streams, Some(1024));
        assert!(receiver.config.http2_adaptive_window);

        let config_with_timeout = json!({
            "listening_addr": "127.0.0.1:4317",
            "timeout": "30s"
        });
        let receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_with_timeout).unwrap();
        assert_eq!(receiver.config.timeout, Some(Duration::from_secs(30)));

        let config_with_timeout_ms = json!({
            "listening_addr": "127.0.0.1:4317",
            "timeout": "500ms"
        });
        let receiver = OTLPReceiver::from_config(pipeline_ctx, &config_with_timeout_ms).unwrap();
        assert_eq!(receiver.config.timeout, Some(Duration::from_millis(500)));
    }

    #[test]
    fn test_tune_max_concurrent_requests() {
        use serde_json::json;

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        // Defaults clamp to downstream capacity.
        let config_default = json!({
            "listening_addr": "127.0.0.1:4317"
        });
        let mut receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_default).unwrap();
        receiver.tune_max_concurrent_requests(128);
        assert_eq!(receiver.config.max_concurrent_requests, 128);

        // User provided smaller value is preserved.
        let config_small = json!({
            "listening_addr": "127.0.0.1:4317",
            "max_concurrent_requests": 32
        });
        let mut receiver = OTLPReceiver::from_config(pipeline_ctx.clone(), &config_small).unwrap();
        receiver.tune_max_concurrent_requests(128);
        assert_eq!(receiver.config.max_concurrent_requests, 32);

        // Config value of zero adopts downstream capacity.
        let config_zero = json!({
            "listening_addr": "127.0.0.1:4317",
            "max_concurrent_requests": 0
        });
        let mut receiver = OTLPReceiver::from_config(pipeline_ctx, &config_zero).unwrap();
        receiver.tune_max_concurrent_requests(256);
        assert_eq!(receiver.config.max_concurrent_requests, 256);
    }

    fn scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Logs Service Client");

                let logs_response = logs_client
                    .export(create_logs_service_request())
                    .await
                    .expect("Can send log request")
                    .into_inner();
                assert_eq!(
                    logs_response,
                    ExportLogsServiceResponse {
                        partial_success: None
                    }
                );

                let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Metrics Service Client");
                let metrics_response = metrics_client
                    .export(create_metrics_service_request())
                    .await
                    .expect("can send metrics request")
                    .into_inner();
                assert_eq!(
                    metrics_response,
                    ExportMetricsServiceResponse {
                        partial_success: None
                    }
                );

                let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Traces Service Client");
                let traces_response = traces_client
                    .export(create_traces_service_request())
                    .await
                    .expect("can send traces request")
                    .into_inner();
                assert_eq!(
                    traces_response,
                    ExportTraceServiceResponse {
                        partial_success: None
                    }
                );

                ctx.send_shutdown(Instant::now(), "Test")
                    .await
                    .expect("Failed to send Shutdown");

                let fail_client = LogsServiceClient::connect(grpc_endpoint.clone()).await;
                assert!(fail_client.is_err(), "Server did not shutdown");
            })
        }
    }

    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // Receive logs pdata
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for logs message")
                    .expect("No logs message received");

                // Validate logs payload
                let logs_proto: OtlpProtoBytes = logs_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");
                assert!(matches!(logs_proto, OtlpProtoBytes::ExportLogsRequest(_)));

                let expected = create_logs_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, logs_proto.as_bytes());

                // Send Ack back to unblock the gRPC handler
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack for logs");
                }

                // Receive metrics pdata
                let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for metrics message")
                    .expect("No metrics message received");

                // Validate metrics payload
                let metrics_proto: OtlpProtoBytes = metrics_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");
                assert!(matches!(
                    metrics_proto,
                    OtlpProtoBytes::ExportMetricsRequest(_)
                ));

                let expected = create_metrics_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, metrics_proto.as_bytes());

                // Send Ack back to unblock the gRPC handler
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(metrics_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack for metrics");
                }

                // Receive trace pdata
                let trace_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for trace message")
                    .expect("No trace message received");

                // Validate trace payload
                let trace_proto: OtlpProtoBytes = trace_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");
                assert!(matches!(
                    trace_proto,
                    OtlpProtoBytes::ExportTracesRequest(_)
                ));

                let expected = create_traces_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, trace_proto.as_bytes());

                // Send Ack back to unblock the gRPC handler
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(trace_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack for traces");
                }
            })
        }
    }

    #[test]
    fn test_otlp_receiver_ack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        // Create a proper pipeline context for the test
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config: test_config(addr),
                metrics: pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint))
            .run_validation_concurrent(validation_procedure());
    }

    #[test]
    fn test_otlp_receiver_nack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config: test_config(addr),
                metrics: pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let nack_scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server");

                let result = logs_client.export(create_logs_service_request()).await;

                assert!(result.is_err(), "Expected error response");
                let status = result.unwrap_err();

                // Verify we get UNAVAILABLE status code
                assert_eq!(status.code(), tonic::Code::Unavailable);
                assert!(status.message().contains("Test nack reason"));
                assert!(status.message().contains("Pipeline processing failed"));

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let nack_validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                // Receive the logs pdata, create Nack message and send it back
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for logs message")
                    .expect("No logs message received");

                let nack = NackMsg::new("Test nack reason", logs_pdata);
                if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                    ctx.send_control_msg(NodeControlMsg::Nack(nack))
                        .await
                        .expect("Failed to send Nack");
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(nack_scenario)
            .run_validation_concurrent(nack_validation);
    }
}
