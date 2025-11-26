// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! # OTLP receiver architecture
//!
//! Once the TCP listener is bound the receiver builds three OTLP-specific gRPC servers (logs,
//! metrics, traces). Each server is backed by the shared codecs in `otap_grpc::otlp::server`,
//! producing lazily-decoded [`OtapPdata`](crate::pdata::OtapPdata) that are pushed straight into
//! the pipeline. The `SignalAckRoutingState` bundle aggregates the per-signal subscription maps that
//! connect incoming requests with their ACK/NACK responses when `wait_for_result` is enabled.
//!
//! A `tokio::select!` drives two responsibilities concurrently:
//! - poll control messages from the engine (shutdown, telemetry collection, ACK/NACK forwarding)
//! - serve the gRPC endpoints with the tuned concurrency settings derived from downstream channel
//!   capacity.
//!
//! Periodic telemetry snapshots update the `OtlpReceiverMetrics` counters, which focus on ACK/NACK
//! behaviour today.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::otap_grpc::otlp::server::{LogsServiceServer, MetricsServiceServer, TraceServiceServer};
use crate::pdata::OtapPdata;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
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
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tower::limit::GlobalConcurrencyLimitLayer;
use tonic::transport::Server;
use crate::otap_grpc::common;
use crate::otap_grpc::common::AckRegistry;
use crate::otap_grpc::server_settings::GrpcServerSettings;

/// URN for the OTLP Receiver
pub const OTLP_RECEIVER_URN: &str = "urn:otel:otlp:receiver";

/// Configuration for OTLP Receiver
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Shared gRPC server settings reused across gRPC-based receivers.
    #[serde(flatten)]
    pub grpc: GrpcServerSettings,
}

/// gRPC receiver that ingests OTLP signals and forwards them into the OTAP pipeline.
///
/// The implementation mirrors the OTAP Arrow receiver layout: a shared [`GrpcServerConfig`] drives
/// listener creation, per-signal tonic services are registered on a single server, and the receiver
/// is wrapped in a [`ReceiverFactory`] so the dataflow engine can build it from configuration.
///
/// Several optimisations keep the hot path inexpensive:
/// - Incoming request bodies stay in their serialized OTLP form thanks to the custom
///   [`OtlpBytesCodec`](crate::otap_grpc::otlp::server::OtlpBytesCodec), allowing downstream stages
///   to decode lazily.
/// - `tune_max_concurrent_requests` clamps the gRPC concurrency to the downstream channel capacity,
///   preventing backlog buildup while still honouring user settings.
/// - `SignalAckRoutingState` maintains per-signal ACK subscription slots so `wait_for_result`
///   lookups avoid extra bookkeeping and route responses directly back to callers.
/// - Metrics are wired through a `MetricSet`, letting periodic snapshots flush ACK/NACK counters
///   without rebuilding instruments.
pub struct OTLPReceiver {
    config: Config,
    metrics: MetricSet<OtlpReceiverMetrics>,
}

/// Declares the OTLP receiver as a shared receiver factory.
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
        common::tune_max_concurrent_requests(&mut self.config.grpc, downstream_capacity);
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
        let config = &self.config.grpc;
        let listener = effect_handler.tcp_listener(config.listening_addr)?;
        // Wrap the raw listener to enforce keepalive/nodelay socket tuning on accepted streams.
        let incoming = config.build_tcp_incoming(listener);
        let settings = config.build_settings();

        let logs_server = LogsServiceServer::new(effect_handler.clone(), &settings);
        let metrics_server = MetricsServiceServer::new(effect_handler.clone(), &settings);
        let traces_server = TraceServiceServer::new(effect_handler.clone(), &settings);

        // Gather the per-signal subscription maps so ACK/NACK routing stays signal-aware.
        let ack_registry = AckRegistry::new(
            logs_server.common.state(),
            metrics_server.common.state(),
            traces_server.common.state(),
        );
        let max_concurrent_requests = config.max_concurrent_requests.max(1);
        let global_limit =
            GlobalConcurrencyLimitLayer::new(max_concurrent_requests);
        let mut server_builder =
            common::apply_server_tuning(Server::builder(), config).layer(global_limit);

        let server = server_builder
            .add_service(logs_server)
            .add_service(metrics_server)
            .add_service(traces_server);

        // Start periodic telemetry collection
        let mut telemetry_cancel_handle = Some(
            effect_handler
                .start_periodic_telemetry(Duration::from_secs(1))
                .await?,
        );
        let mut server_task = Box::pin(server.serve_with_incoming(incoming));

        loop {
            tokio::select! {
                biased;

                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            let snapshot = self.metrics.snapshot();
                            if let Some(handle) = telemetry_cancel_handle.take() {
                                _ = handle.cancel().await;
                            }
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        },
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            // Report current receiver metrics.
                            _ = metrics_reporter.report(&mut self.metrics);
                        },
                        Ok(NodeControlMsg::Ack(ack)) => {
                            let resp = common::route_ack_response(&ack_registry, ack);
                            common::handle_route_response(
                                resp,
                                &mut self.metrics,
                                |metrics| metrics.acks_sent.inc(),
                                |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
                            );
                        },
                        Ok(NodeControlMsg::Nack(nack)) => {
                            let resp = common::route_nack_response(&ack_registry, nack);
                            common::handle_route_response(
                                resp,
                                &mut self.metrics,
                                |metrics| metrics.nacks_sent.inc(),
                                |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
                            );
                        },
                        Err(e) => {
                            if let Some(handle) = telemetry_cancel_handle.take() {
                                _ = handle.cancel().await;
                            }
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message do nothing
                        }
                    }
                },

                result = &mut server_task => {
                    if let Some(handle) = telemetry_cancel_handle.take() {
                        _ = handle.cancel().await;
                    }
                    if let Err(error) = result {
                        let source_detail = format_error_sources(&error);
                        return Err(Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Transport,
                            error: error.to_string(),
                            source_detail,
                        });
                    }
                    break;
                }
            }
        }

        if let Some(handle) = telemetry_cancel_handle.take() {
            _ = handle.cancel().await;
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

    use crate::compression::CompressionMethod;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NackMsg;
    use otap_df_engine::control::{AckMsg, NodeControlMsg};
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_pdata::OtlpProtoBytes;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_client::LogsServiceClient;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse,
    };
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
    };
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::trace_service_client::TraceServiceClient;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse,
    };
    use otap_df_pdata::proto::opentelemetry::common::v1::{InstrumentationScope, KeyValue};
    use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_pdata::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};
    use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
    use otap_df_pdata::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans};
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use prost::Message;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::time::{Duration, Instant};
    use tokio::time::timeout;

    fn test_config(addr: SocketAddr) -> Config {
        let grpc = GrpcServerSettings {
            listening_addr: addr,
            max_concurrent_requests: 1000,
            wait_for_result: true,
            ..Default::default()
        };
        Config { grpc }
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
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 5000);

        let config_default = json!({
            "listening_addr": "127.0.0.1:4317"
        });
        let receiver = OTLPReceiver::from_config(pipeline_ctx.clone(), &config_default).unwrap();
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 0);
        assert!(receiver.config.grpc.request_compression.is_none());
        assert!(receiver.config.grpc.response_compression.is_none());
        assert!(
            receiver
                .config
                .grpc
                .preferred_response_compression()
                .is_none()
        );
        assert!(receiver.config.grpc.tcp_nodelay);
        assert_eq!(
            receiver.config.grpc.tcp_keepalive,
            Some(Duration::from_secs(45))
        );
        assert_eq!(
            receiver.config.grpc.tcp_keepalive_interval,
            Some(Duration::from_secs(15))
        );
        assert_eq!(receiver.config.grpc.tcp_keepalive_retries, Some(5));
        assert_eq!(receiver.config.grpc.transport_concurrency_limit, None);
        assert!(receiver.config.grpc.load_shed);
        assert_eq!(
            receiver.config.grpc.initial_stream_window_size,
            Some(8 * 1024 * 1024)
        );
        assert_eq!(
            receiver.config.grpc.initial_connection_window_size,
            Some(24 * 1024 * 1024)
        );
        assert!(!receiver.config.grpc.http2_adaptive_window);
        assert_eq!(receiver.config.grpc.max_frame_size, Some(16 * 1024));
        assert_eq!(
            receiver.config.grpc.max_decoding_message_size,
            Some(4 * 1024 * 1024)
        );
        assert_eq!(
            receiver.config.grpc.http2_keepalive_interval,
            Some(Duration::from_secs(30))
        );
        assert_eq!(
            receiver.config.grpc.http2_keepalive_timeout,
            Some(Duration::from_secs(10))
        );
        assert_eq!(receiver.config.grpc.max_concurrent_streams, None);

        let config_full = json!({
            "listening_addr": "127.0.0.1:4317",
            "request_compression": "gzip",
            "max_concurrent_requests": 2500
        });
        let receiver = OTLPReceiver::from_config(pipeline_ctx.clone(), &config_full).unwrap();
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 2500);
        assert_eq!(
            receiver.config.grpc.request_compression,
            Some(vec![CompressionMethod::Gzip])
        );
        assert!(receiver.config.grpc.response_compression.is_none());

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
            "max_decoding_message_size": "6MiB",
            "http2_keepalive_interval": "45s",
            "http2_keepalive_timeout": "20s",
            "max_concurrent_streams": 1024,
            "http2_adaptive_window": true
        });
        let receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_with_server_overrides).unwrap();
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 512);
        assert!(!receiver.config.grpc.tcp_nodelay);
        assert_eq!(
            receiver.config.grpc.tcp_keepalive,
            Some(Duration::from_secs(60))
        );
        assert_eq!(
            receiver.config.grpc.tcp_keepalive_interval,
            Some(Duration::from_secs(20))
        );
        assert_eq!(receiver.config.grpc.tcp_keepalive_retries, Some(3));
        assert_eq!(receiver.config.grpc.transport_concurrency_limit, Some(256));
        assert!(!receiver.config.grpc.load_shed);
        assert_eq!(
            receiver.config.grpc.initial_stream_window_size,
            Some(4 * 1024 * 1024)
        );
        assert_eq!(
            receiver.config.grpc.initial_connection_window_size,
            Some(16 * 1024 * 1024)
        );
        assert_eq!(receiver.config.grpc.max_frame_size, Some(8 * 1024 * 1024));
        assert_eq!(
            receiver.config.grpc.max_decoding_message_size,
            Some(6 * 1024 * 1024)
        );
        assert_eq!(
            receiver.config.grpc.http2_keepalive_interval,
            Some(Duration::from_secs(45))
        );
        assert_eq!(
            receiver.config.grpc.http2_keepalive_timeout,
            Some(Duration::from_secs(20))
        );
        assert_eq!(receiver.config.grpc.max_concurrent_streams, Some(1024));
        assert!(receiver.config.grpc.http2_adaptive_window);

        let config_with_compression_list = json!({
            "listening_addr": "127.0.0.1:4317",
            "request_compression": ["gzip", "zstd", "gzip"]
        });
        let receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_with_compression_list).unwrap();
        assert_eq!(
            receiver.config.grpc.request_compression,
            Some(vec![CompressionMethod::Gzip, CompressionMethod::Zstd])
        );
        assert!(receiver.config.grpc.response_compression.is_none());

        let config_with_compression_none = json!({
            "listening_addr": "127.0.0.1:4317",
            "request_compression": "none"
        });
        let receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_with_compression_none).unwrap();
        assert_eq!(receiver.config.grpc.request_compression, Some(vec![]));
        assert!(receiver.config.grpc.response_compression.is_none());

        let config_with_timeout = json!({
            "listening_addr": "127.0.0.1:4317",
            "timeout": "30s"
        });
        let receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_with_timeout).unwrap();
        assert_eq!(receiver.config.grpc.timeout, Some(Duration::from_secs(30)));

        let config_with_timeout_ms = json!({
            "listening_addr": "127.0.0.1:4317",
            "timeout": "500ms"
        });
        let receiver = OTLPReceiver::from_config(pipeline_ctx, &config_with_timeout_ms).unwrap();
        assert_eq!(
            receiver.config.grpc.timeout,
            Some(Duration::from_millis(500))
        );
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
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 128);

        // User provided smaller value is preserved.
        let config_small = json!({
            "listening_addr": "127.0.0.1:4317",
            "max_concurrent_requests": 32
        });
        let mut receiver = OTLPReceiver::from_config(pipeline_ctx.clone(), &config_small).unwrap();
        receiver.tune_max_concurrent_requests(128);
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 32);

        // Config value of zero adopts downstream capacity.
        let config_zero = json!({
            "listening_addr": "127.0.0.1:4317",
            "max_concurrent_requests": 0
        });
        let mut receiver = OTLPReceiver::from_config(pipeline_ctx, &config_zero).unwrap();
        receiver.tune_max_concurrent_requests(256);
        assert_eq!(receiver.config.grpc.max_concurrent_requests, 256);
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
