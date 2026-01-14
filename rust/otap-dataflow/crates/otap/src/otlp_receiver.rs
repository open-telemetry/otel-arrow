// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! # OTLP receiver architecture
//!
//! Once the TCP listener is bound the receiver builds three OTLP-specific gRPC servers (logs,
//! metrics, traces). Each server is backed by the shared codecs in `otap_grpc::otlp::server`,
//! producing lazily-decoded [`OtapPdata`](OtapPdata) that are pushed straight into
//! the pipeline. The `AckRegistry` aggregates the per-signal subscription maps that connect
//! incoming requests with their ACK/NACK responses when `wait_for_result` is enabled.
//!
//! A `tokio::select!` drives two responsibilities concurrently:
//! - poll control messages from the engine (shutdown, telemetry collection, ACK/NACK forwarding)
//! - serve the gRPC endpoints with the tuned concurrency settings derived from downstream channel
//!   capacity.
//!
//! Periodic telemetry snapshots update the `OtlpReceiverMetrics` counters, which focus on ACK/NACK
//! behaviour today.

use crate::OTAP_RECEIVER_FACTORIES;
use crate::otap_grpc::otlp::server_new::{
    LogsServiceServer, MetricsServiceServer, OtlpServerSettings, TraceServiceServer,
};
use crate::pdata::OtapPdata;
#[cfg(feature = "experimental-tls")]
use crate::tls_utils::{build_tls_acceptor, create_tls_stream};
#[cfg(feature = "experimental-tls")]
use otap_df_config::tls::TlsServerConfig;

use crate::otap_grpc::common;
use crate::otap_grpc::common::AckRegistry;
use crate::otap_grpc::server_settings::GrpcServerSettings;
use crate::otlp_http::HttpServerSettings;
use crate::shared_concurrency::SharedConcurrencyLayer;
use async_trait::async_trait;
use linkme::distributed_slice;
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
use parking_lot::Mutex;
use serde::Deserialize;
use serde_json::Value;
use std::future::Future;
use std::ops::Add;
use std::pin::Pin;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tokio_util::sync::CancellationToken;
use tonic::transport::Server;
use tower::limit::GlobalConcurrencyLimitLayer;
use tower::util::Either;

/// URN for the OTLP Receiver
pub const OTLP_RECEIVER_URN: &str = "urn:otel:otlp:receiver";

/// Interval for periodic telemetry collection.
const TELEMETRY_INTERVAL: Duration = Duration::from_secs(1);

/// Configuration for OTLP Receiver
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Shared gRPC server settings reused across gRPC-based receivers.
    #[serde(flatten)]
    pub grpc: GrpcServerSettings,

    /// Optional OTLP/HTTP server settings.
    ///
    /// When configured, the receiver will start an additional HTTP server implementing
    /// `POST /v1/logs`, `POST /v1/metrics`, and `POST /v1/traces`.
    #[serde(default)]
    pub http: Option<HttpServerSettings>,

    /// TLS configuration
    #[cfg(feature = "experimental-tls")]
    pub tls: Option<TlsServerConfig>,
}

/// gRPC receiver that ingests OTLP signals and forwards them into the OTAP pipeline.
///
/// The implementation mirrors the OTAP Arrow receiver layout: a shared [`GrpcServerSettings`]
/// drives listener creation, per-signal tonic services are registered on a single server, and the
/// receiver is wrapped in a [`ReceiverFactory`] so the dataflow engine can build it from
/// configuration.
///
/// Several optimisations keep the hot path inexpensive:
/// - Incoming request bodies stay in their serialized OTLP form thanks to the custom
///   [`OtlpBytesCodec`](crate::otap_grpc::otlp::server_new::OtlpBytesCodec), allowing downstream stages
///   to decode lazily.
/// - `AckRegistry` maintains per-signal ACK subscription slots so `wait_for_result` lookups avoid
///   extra bookkeeping and route responses directly back to callers.
/// - Metrics are wired through a `MetricSet`, letting periodic snapshots flush ACK/NACK counters
///   without rebuilding instruments.
pub struct OTLPReceiver {
    config: Config,
    // Arc<Mutex<...>> so we can share metrics with the gRPC services which are `Send` due to
    // tonic requirements.
    metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
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
        let metrics = Arc::new(Mutex::new(
            pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
        ));

        Ok(Self { config, metrics })
    }

    fn tune_max_concurrent_requests(&mut self, downstream_capacity: usize) {
        // Set both protocols to downstream_capacity.
        // When HTTP is enabled, they'll share a single semaphore pool,
        // so the true ceiling is downstream_capacity (not 2x).
        // When HTTP is disabled, only gRPC uses the capacity.
        common::tune_max_concurrent_requests(&mut self.config.grpc, downstream_capacity);
        if let Some(http) = self.config.http.as_mut() {
            crate::otlp_http::tune_max_concurrent_requests(http, downstream_capacity);
        }
    }

    fn build_signal_services(
        &self,
        effect_handler: &shared::EffectHandler<OtapPdata>,
        settings: &OtlpServerSettings,
    ) -> (
        LogsServiceServer,
        MetricsServiceServer,
        TraceServiceServer,
        AckRegistry,
    ) {
        let http_wait = self
            .config
            .http
            .as_ref()
            .is_some_and(|http| http.wait_for_result);
        let grpc_wait = settings.wait_for_result;
        let wait_for_result_any = grpc_wait || http_wait;

        // When HTTP is enabled it always shares the same semaphore pool as gRPC.
        // That means the maximum number of concurrent wait-for-result subscriptions that can be
        // active across BOTH protocols is bounded by the shared semaphore capacity.
        let shared_ack_slot_capacity = if wait_for_result_any {
            settings.max_concurrent_requests.max(1)
        } else {
            0
        };

        let logs_slot = wait_for_result_any
            .then(|| crate::otap_grpc::otlp::server_new::AckSlot::new(shared_ack_slot_capacity));
        let metrics_slot = wait_for_result_any
            .then(|| crate::otap_grpc::otlp::server_new::AckSlot::new(shared_ack_slot_capacity));
        let traces_slot = wait_for_result_any
            .then(|| crate::otap_grpc::otlp::server_new::AckSlot::new(shared_ack_slot_capacity));

        let logs_server = LogsServiceServer::new(
            effect_handler.clone(),
            settings,
            self.metrics.clone(),
            grpc_wait.then(|| logs_slot.clone()).flatten(),
        );
        let metrics_server = MetricsServiceServer::new(
            effect_handler.clone(),
            settings,
            self.metrics.clone(),
            grpc_wait.then(|| metrics_slot.clone()).flatten(),
        );
        let traces_server = TraceServiceServer::new(
            effect_handler.clone(),
            settings,
            self.metrics.clone(),
            grpc_wait.then(|| traces_slot.clone()).flatten(),
        );

        let ack_registry = AckRegistry::new(logs_slot, metrics_slot, traces_slot);

        (logs_server, metrics_server, traces_server, ack_registry)
    }

    fn handle_ack(&mut self, registry: &AckRegistry, ack: AckMsg<OtapPdata>) {
        let resp = common::route_ack_response(registry, ack);
        let mut metrics = self.metrics.lock();
        common::handle_route_response(
            resp,
            &mut *metrics,
            |metrics| metrics.acks_received.inc(),
            |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
        );
    }

    fn handle_nack(&mut self, registry: &AckRegistry, nack: NackMsg<OtapPdata>) {
        let resp = common::route_nack_response(registry, nack);
        let mut metrics = self.metrics.lock();
        common::handle_route_response(
            resp,
            &mut *metrics,
            |metrics| metrics.nacks_received.inc(),
            |metrics| metrics.acks_nacks_invalid_or_expired.inc(),
        );
    }

    async fn handle_control_message(
        &mut self,
        msg: NodeControlMsg<OtapPdata>,
        registry: &AckRegistry,
        telemetry_cancel_handle: &mut Option<
            otap_df_engine::effect_handler::TelemetryTimerCancelHandle<OtapPdata>,
        >,
    ) -> Result<Option<TerminalState>, Error> {
        match msg {
            NodeControlMsg::Shutdown { deadline, .. } => {
                let snapshot = self.metrics.lock().snapshot();
                if let Some(handle) = telemetry_cancel_handle.take() {
                    _ = handle.cancel().await;
                }
                Ok(Some(TerminalState::new(deadline, [snapshot])))
            }
            NodeControlMsg::CollectTelemetry {
                mut metrics_reporter,
            } => {
                _ = metrics_reporter.report(&mut *self.metrics.lock());
                Ok(None)
            }
            NodeControlMsg::Ack(ack) => {
                self.handle_ack(registry, ack);
                Ok(None)
            }
            NodeControlMsg::Nack(nack) => {
                self.handle_nack(registry, nack);
                Ok(None)
            }
            _ => Ok(None),
        }
    }

    fn map_transport_error<E: std::error::Error + Send + Sync + 'static>(
        &self,
        effect_handler: &shared::EffectHandler<OtapPdata>,
        error: E,
    ) -> Error {
        let source_detail = format_error_sources(&error);
        Error::ReceiverError {
            receiver: effect_handler.receiver_id(),
            kind: ReceiverErrorKind::Transport,
            error: error.to_string(),
            source_detail,
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
    /// Number of acks received from downstream (routed back to the caller).
    #[metric(unit = "{acks}")]
    pub acks_received: Counter<u64>,

    /// Number of nacks received from downstream (routed back to the caller).
    #[metric(unit = "{nacks}")]
    pub nacks_received: Counter<u64>,

    /// Number of invalid/expired acks/nacks.
    #[metric(unit = "{ack_or_nack}")]
    pub acks_nacks_invalid_or_expired: Counter<u64>,

    /// Number of OTLP RPCs started.
    #[metric(unit = "{requests}")]
    pub requests_started: Counter<u64>,

    /// Number of OTLP RPCs completed (success + nack).
    #[metric(unit = "{requests}")]
    pub requests_completed: Counter<u64>,

    /// Number of OTLP RPCs rejected before entering the pipeline (e.g. slot exhaustion).
    #[metric(unit = "{requests}")]
    pub rejected_requests: Counter<u64>,

    /// Number of transport-level errors surfaced by tonic/server.
    #[metric(unit = "{errors}")]
    pub transport_errors: Counter<u64>,

    /// Total bytes received across OTLP requests (payload bytes).
    #[metric(unit = "By")]
    pub request_bytes: Counter<u64>,
}

#[async_trait]
impl shared::Receiver<OtapPdata> for OTLPReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel<OtapPdata>,
        effect_handler: shared::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otap_df_telemetry::otel_info!("Receiver.Start", message = "Starting OTLP Receiver");

        // Make the receiver mutable so we can update metrics on telemetry collection.
        let config = &self.config.grpc;
        let listener = effect_handler.tcp_listener(config.listening_addr)?;
        // Wrap the raw listener to enforce keepalive/nodelay socket tuning on accepted streams.
        let incoming = config.build_tcp_incoming(listener);
        let settings = config.build_settings();

        let (logs_server, metrics_server, traces_server, ack_registry) =
            self.build_signal_services(&effect_handler, &settings);
        let max_concurrent_requests = config.max_concurrent_requests.max(1);

        // Build gRPC server concurrency strategy based on HTTP being enabled:
        // - If HTTP is enabled, use a shared semaphore so both protocols draw from the same pool.
        // - If HTTP is disabled, preserve prior gRPC-only behavior (GlobalConcurrencyLimitLayer).
        let (limit_layer, shared_semaphore) = if self.config.http.is_some() {
            let shared = Arc::new(Semaphore::new(max_concurrent_requests));
            (
                Either::Left(SharedConcurrencyLayer::new(shared.clone())),
                Some(shared),
            )
        } else {
            (
                Either::Right(GlobalConcurrencyLimitLayer::new(max_concurrent_requests)),
                None,
            )
        };

        let mut server = common::apply_server_tuning(Server::builder(), config).layer(limit_layer);

        if let Some(timeout) = config.timeout {
            server = server.timeout(timeout);
        }

        let server = server
            .add_service(logs_server)
            .add_service(metrics_server)
            .add_service(traces_server);

        #[cfg(feature = "experimental-tls")]
        let maybe_tls_acceptor =
            build_tls_acceptor(self.config.tls.as_ref())
                .await
                .map_err(|e| Error::ReceiverError {
                    receiver: effect_handler.receiver_id(),
                    kind: ReceiverErrorKind::Configuration,
                    error: format!("Failed to configure TLS: {}", e),
                    source_detail: format_error_sources(&e),
                })?;

        #[cfg(feature = "experimental-tls")]
        let handshake_timeout = self.config.tls.as_ref().and_then(|t| t.handshake_timeout);

        let mut telemetry_cancel_handle = Some(
            effect_handler
                .start_periodic_telemetry(TELEMETRY_INTERVAL)
                .await?,
        );

        let mut server_task: Pin<
            Box<dyn Future<Output = Result<(), tonic::transport::Error>> + Send>,
        > = {
            #[cfg(feature = "experimental-tls")]
            {
                match maybe_tls_acceptor {
                    Some(tls_acceptor) => {
                        let tls_stream =
                            create_tls_stream(incoming, tls_acceptor, handshake_timeout);
                        Box::pin(server.serve_with_incoming(tls_stream))
                    }
                    None => Box::pin(server.serve_with_incoming(incoming)),
                }
            }
            #[cfg(not(feature = "experimental-tls"))]
            {
                Box::pin(server.serve_with_incoming(incoming))
            }
        };

        let http_enabled = self.config.http.is_some();
        let http_shutdown = CancellationToken::new();
        let mut http_task: Pin<Box<dyn Future<Output = std::io::Result<()>> + Send>> =
            if let Some(http) = self.config.http.clone() {
                // Pass the shared semaphore to HTTP server so both protocols
                // draw from the same concurrency pool
                Box::pin(crate::otlp_http::serve(
                    effect_handler.clone(),
                    http,
                    ack_registry.clone(),
                    self.metrics.clone(),
                    shared_semaphore,
                    http_shutdown.clone(),
                ))
            } else {
                Box::pin(std::future::pending())
            };

        let mut http_task_done = false;
        let terminal_state: TerminalState;

        loop {
            tokio::select! {
                biased;
                // Control-plane messages take priority over serving to react quickly to shutdown/telemetry.
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(msg) => {
                            if let Some(terminal) = self
                                .handle_control_message(msg, &ack_registry, &mut telemetry_cancel_handle)
                                .await?
                            {
                                http_shutdown.cancel();
                                terminal_state = terminal;
                                break;
                            }
                        }
                        Err(e) => {
                            if let Some(handle) = telemetry_cancel_handle.take() {
                                _ = handle.cancel().await;
                            }
                            return Err(Error::ChannelRecvError(e));
                        }
                    }
                },

                // gRPC serving loop; exits on transport error or graceful stop.
                result = &mut server_task => {
                    if let Some(handle) = telemetry_cancel_handle.take() {
                        _ = handle.cancel().await;
                    }
                    if let Err(error) = result {
                        self.metrics.lock().transport_errors.inc();
                        return Err(self.map_transport_error(&effect_handler, error));
                    }
                    terminal_state = TerminalState::new(
                        Instant::now().add(Duration::from_secs(1)),
                        [self.metrics.lock().snapshot()],
                    );
                    http_shutdown.cancel();
                    break;
                }

                // HTTP serving loop; exits on IO error.
                result = &mut http_task => {
                    if let Some(handle) = telemetry_cancel_handle.take() {
                        _ = handle.cancel().await;
                    }
                    if let Err(error) = result {
                        self.metrics.lock().transport_errors.inc();
                        return Err(self.map_transport_error(&effect_handler, error));
                    }
                    http_task_done = true;
                    terminal_state = TerminalState::new(
                        Instant::now().add(Duration::from_secs(1)),
                        [self.metrics.lock().snapshot()],
                    );
                    break;
                }
            }
        }

        http_shutdown.cancel();

        if http_enabled && !http_task_done {
            if let Err(error) = http_task.await {
                self.metrics.lock().transport_errors.inc();
                return Err(self.map_transport_error(&effect_handler, error));
            }
        }

        Ok(terminal_state)
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
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::time::{Duration, Instant};
    use tokio::time::timeout;

    use bytes::Bytes;
    use http_body_util::BodyExt;
    use http_body_util::Full;
    use hyper::Method;
    use hyper::client::conn::http1;
    use hyper::header::{CONTENT_ENCODING, CONTENT_TYPE, HOST};
    use hyper_util::rt::TokioIo;
    use tokio::net::TcpStream;

    fn test_config(addr: SocketAddr) -> Config {
        let grpc = GrpcServerSettings {
            listening_addr: addr,
            max_concurrent_requests: 1000,
            wait_for_result: true,
            ..Default::default()
        };
        Config {
            grpc,
            http: None,
            #[cfg(feature = "experimental-tls")]
            tls: None,
        }
    }

    async fn post_otlp_http_with_encoding(
        addr: SocketAddr,
        path: &'static str,
        body: Vec<u8>,
        content_encoding: Option<&'static str>,
    ) -> Result<(http::StatusCode, Bytes), Box<dyn std::error::Error + Send + Sync>> {
        let stream = TcpStream::connect(addr).await?;
        let (mut sender, conn) = http1::handshake(TokioIo::new(stream)).await?;
        _ = tokio::spawn(async move {
            let _ = conn.await;
        });

        let mut builder = http::Request::builder()
            .method(Method::POST)
            .uri(path)
            .header(HOST, "localhost")
            .header(CONTENT_TYPE, "application/x-protobuf");

        if let Some(encoding) = content_encoding {
            builder = builder.header(CONTENT_ENCODING, encoding);
        }

        let req = builder.body(Full::new(Bytes::from(body)))?;

        let resp = sender.send_request(req).await?;
        let status = resp.status();
        let body = resp.into_body().collect().await?.to_bytes();
        Ok((status, body))
    }

    async fn post_otlp_http(
        addr: SocketAddr,
        path: &'static str,
        body: Vec<u8>,
    ) -> Result<(http::StatusCode, Bytes), Box<dyn std::error::Error + Send + Sync>> {
        post_otlp_http_with_encoding(addr, path, body, None).await
    }

    async fn send_http_request(
        addr: SocketAddr,
        method: Method,
        path: &'static str,
    ) -> Result<(http::StatusCode, Bytes), Box<dyn std::error::Error + Send + Sync>> {
        let stream = TcpStream::connect(addr).await?;
        let (mut sender, conn) = http1::handshake(TokioIo::new(stream)).await?;
        _ = tokio::spawn(async move {
            let _ = conn.await;
        });

        let req = http::Request::builder()
            .method(method)
            .uri(path)
            .header(HOST, "localhost")
            .body(Full::new(Bytes::new()))?;

        let resp = sender.send_request(req).await?;
        let status = resp.status();
        let body = resp.into_body().collect().await?.to_bytes();
        Ok((status, body))
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

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
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

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
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
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config: test_config(addr),
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
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
    fn test_otlp_http_receiver_ack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{grpc_addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 1000,
            wait_for_result: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let request = create_logs_service_request();
                let mut request_bytes = Vec::new();
                request.encode(&mut request_bytes).unwrap();

                let (status, body) = post_otlp_http(http_listen, "/v1/logs", request_bytes)
                    .await
                    .expect("http request should succeed");

                assert_eq!(status, http::StatusCode::OK);

                let mut expected = Vec::new();
                ExportLogsServiceResponse::default()
                    .encode(&mut expected)
                    .unwrap();
                assert_eq!(body.as_ref(), expected.as_slice());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for logs message")
                    .expect("No logs message received");

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

                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack");
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    #[test]
    fn test_otlp_http_rejects_oversized_identity_body() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: false,
            max_request_body_size: 1024,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let request_bytes = vec![0u8; 2048];
                let (status, body) = post_otlp_http(http_listen, "/v1/logs", request_bytes)
                    .await
                    .expect("http request should succeed");

                assert_eq!(status, http::StatusCode::BAD_REQUEST);
                assert!(!body.is_empty());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        _ = test_runtime.set_receiver(receiver).run_test(scenario);
    }

    #[test]
    fn test_otlp_http_rejects_oversized_gzip_body() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: false,
            max_request_body_size: 1024,
            accept_compressed_requests: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                use flate2::Compression;
                use flate2::write::GzEncoder;
                use std::io::Write;

                let decoded = vec![0u8; 2048];
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&decoded).unwrap();
                let compressed = encoder.finish().unwrap();

                let (status, body) =
                    post_otlp_http_with_encoding(http_listen, "/v1/logs", compressed, Some("gzip"))
                        .await
                        .expect("http request should succeed");

                assert_eq!(status, http::StatusCode::BAD_REQUEST);
                assert!(!body.is_empty());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        _ = test_runtime.set_receiver(receiver).run_test(scenario);
    }

    #[test]
    fn test_otlp_http_accepts_gzip_body_exactly_at_limit() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: true,
            max_request_body_size: 1024,
            accept_compressed_requests: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                use flate2::Compression;
                use flate2::write::GzEncoder;
                use std::io::Write;

                // Create payload that decompresses to EXACTLY the limit (1024 bytes)
                let decoded = vec![0u8; 1024];
                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&decoded).unwrap();
                let compressed = encoder.finish().unwrap();

                let (status, _body) =
                    post_otlp_http_with_encoding(http_listen, "/v1/logs", compressed, Some("gzip"))
                        .await
                        .expect("http request should succeed");

                // Should be accepted (not 413)
                assert_eq!(status, http::StatusCode::OK);

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for logs message")
                    .expect("No logs message received");

                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack");
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    #[test]
    fn test_otlp_http_rejects_invalid_gzip_body_with_400() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: false,
            max_request_body_size: 1024,
            accept_compressed_requests: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Not valid gzip bytes.
                let body = vec![0xde, 0xad, 0xbe, 0xef];
                let (status, resp_body) =
                    post_otlp_http_with_encoding(http_listen, "/v1/logs", body, Some("gzip"))
                        .await
                        .expect("http request should succeed");

                assert_eq!(status, http::StatusCode::BAD_REQUEST);
                assert!(!resp_body.is_empty());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        _ = test_runtime.set_receiver(receiver).run_test(scenario);
    }

    #[test]
    fn test_otlp_http_rejects_invalid_deflate_body_with_400() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: false,
            max_request_body_size: 1024,
            accept_compressed_requests: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Not valid zlib/deflate bytes.
                let body = vec![0x00, 0x00, 0x00, 0x00];
                let (status, resp_body) =
                    post_otlp_http_with_encoding(http_listen, "/v1/logs", body, Some("deflate"))
                        .await
                        .expect("http request should succeed");

                assert_eq!(status, http::StatusCode::BAD_REQUEST);
                assert!(!resp_body.is_empty());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        _ = test_runtime.set_receiver(receiver).run_test(scenario);
    }

    #[test]
    fn test_otlp_http_timeout_late_ack_and_recovery() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 1,
            wait_for_result: true,
            timeout: Some(Duration::from_millis(200)),
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let request = create_logs_service_request();
                let mut request_bytes = Vec::new();
                request.encode(&mut request_bytes).unwrap();

                // First request: no timely ACK -> should time out.
                let (status1, body1) =
                    post_otlp_http(http_listen, "/v1/logs", request_bytes.clone())
                        .await
                        .expect("http request should succeed");
                assert_eq!(status1, http::StatusCode::SERVICE_UNAVAILABLE);
                assert!(!body1.is_empty());

                // Wait long enough for the validation side to send a late ACK and for the
                // request future to have been dropped (SlotGuard cancel).
                tokio::time::sleep(Duration::from_millis(350)).await;

                // Second request: should succeed, proving capacity/slot recovery.
                let (status2, _body2) = post_otlp_http(http_listen, "/v1/logs", request_bytes)
                    .await
                    .expect("http request should succeed");
                assert_eq!(status2, http::StatusCode::OK);

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                // Receive first pdata and intentionally ACK it late.
                let pdata1 = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for first message")
                    .expect("No first message received");

                tokio::time::sleep(Duration::from_millis(300)).await;

                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata1))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send late Ack");
                }

                // Receive second pdata and ACK promptly so the HTTP caller succeeds.
                let pdata2 = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for second message")
                    .expect("No second message received");

                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata2))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack");
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    #[test]
    fn test_otlp_http_accepts_zstd_body() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: true,
            max_request_body_size: 1024 * 1024,
            accept_compressed_requests: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let request = create_logs_service_request();
                let mut request_bytes = Vec::new();
                request.encode(&mut request_bytes).unwrap();

                let compressed = zstd::stream::encode_all(request_bytes.as_slice(), 0).unwrap();
                let (status, _body) =
                    post_otlp_http_with_encoding(http_listen, "/v1/logs", compressed, Some("zstd"))
                        .await
                        .expect("http request should succeed");

                assert_eq!(status, http::StatusCode::OK);

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for logs message")
                    .expect("No logs message received");

                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack");
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    #[test]
    fn test_otlp_http_rejects_invalid_zstd_body_with_400() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: false,
            max_request_body_size: 1024,
            accept_compressed_requests: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Not valid zstd bytes.
                let body = vec![0xde, 0xad, 0xbe, 0xef];
                let (status, resp_body) =
                    post_otlp_http_with_encoding(http_listen, "/v1/logs", body, Some("zstd"))
                        .await
                        .expect("http request should succeed");

                assert_eq!(status, http::StatusCode::BAD_REQUEST);
                assert!(!resp_body.is_empty());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        _ = test_runtime.set_receiver(receiver).run_test(scenario);
    }

    #[test]
    fn test_otlp_http_rejects_compressed_when_disabled() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: false,
            max_request_body_size: 1024 * 1024,
            accept_compressed_requests: false, // Explicitly disable compression
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                use flate2::Compression;
                use flate2::write::GzEncoder;
                use std::io::Write;

                // Create valid gzip compressed data
                let request = create_logs_service_request();
                let mut request_bytes = Vec::new();
                request.encode(&mut request_bytes).unwrap();

                let mut encoder = GzEncoder::new(Vec::new(), Compression::default());
                encoder.write_all(&request_bytes).unwrap();
                let compressed = encoder.finish().unwrap();

                // Send compressed request when compression is disabled
                let (status, body) =
                    post_otlp_http_with_encoding(http_listen, "/v1/logs", compressed, Some("gzip"))
                        .await
                        .expect("http request should succeed");

                // Should reject with 415 Unsupported Media Type
                assert_eq!(status, http::StatusCode::UNSUPPORTED_MEDIA_TYPE);
                assert!(body.is_empty());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        _ = test_runtime.set_receiver(receiver).run_test(scenario);
    }

    #[test]
    fn test_otlp_receiver_nack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config: test_config(addr),
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
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

    #[test]
    fn test_otlp_http_receiver_nack() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: true, // Enable wait_for_result to test NACK path
            max_request_body_size: 1024 * 1024,
            accept_compressed_requests: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let nack_scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                // Send HTTP request
                let request = create_logs_service_request();
                let mut request_bytes = Vec::new();
                request.encode(&mut request_bytes).unwrap();

                let (status, body) = post_otlp_http(http_listen, "/v1/logs", request_bytes)
                    .await
                    .expect("http request should succeed");

                // Should receive 503 Service Unavailable with NACK reason
                assert_eq!(status, http::StatusCode::SERVICE_UNAVAILABLE);
                assert!(!body.is_empty(), "Response body should contain NACK reason");

                // Decode the RpcStatus response
                let rpc_status = crate::otlp_http::RpcStatus::decode(body.as_ref())
                    .expect("Should decode RpcStatus");

                // Verify the NACK reason is included
                assert_eq!(rpc_status.code, 14); // gRPC UNAVAILABLE code
                assert!(
                    rpc_status.message.contains("Test nack reason"),
                    "Response should contain nack reason, got: {}",
                    rpc_status.message
                );
                assert!(
                    rpc_status.message.contains("Pipeline processing failed"),
                    "Response should contain error prefix, got: {}",
                    rpc_status.message
                );

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

    #[test]
    fn test_http_rejects_non_protobuf_content_type() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{grpc_addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 1000,
            wait_for_result: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let request = create_logs_service_request();
                let mut request_bytes = Vec::new();
                request.encode(&mut request_bytes).unwrap();

                // Send with JSON content type
                let stream = TcpStream::connect(http_listen).await.unwrap();
                let (mut sender, conn) = http1::handshake(TokioIo::new(stream)).await.unwrap();
                _ = tokio::spawn(async move {
                    let _ = conn.await;
                });

                let req = http::Request::builder()
                    .method(Method::POST)
                    .uri("/v1/logs")
                    .header(HOST, "localhost")
                    .header(CONTENT_TYPE, "application/json")
                    .body(Full::new(Bytes::from(request_bytes)))
                    .unwrap();

                let resp = sender.send_request(req).await.unwrap();
                assert_eq!(resp.status(), http::StatusCode::UNSUPPORTED_MEDIA_TYPE);

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        // No validation needed - request rejected at HTTP layer before reaching the pipeline
        let validation = |mut _ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {}) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    #[test]
    fn test_http_accepts_protobuf_with_charset() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{grpc_addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 1000,
            wait_for_result: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let request = create_logs_service_request();
                let mut request_bytes = Vec::new();
                request.encode(&mut request_bytes).unwrap();

                // Send with UTF-8 charset suffix
                let stream = TcpStream::connect(http_listen).await.unwrap();
                let (mut sender, conn) = http1::handshake(TokioIo::new(stream)).await.unwrap();
                _ = tokio::spawn(async move {
                    let _ = conn.await;
                });

                let req = http::Request::builder()
                    .method(Method::POST)
                    .uri("/v1/logs")
                    .header(HOST, "localhost")
                    .header(CONTENT_TYPE, "application/x-protobuf; charset=utf-8")
                    .body(Full::new(Bytes::from(request_bytes)))
                    .unwrap();

                let resp = sender.send_request(req).await.unwrap();
                assert_eq!(resp.status(), http::StatusCode::OK);

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for logs message")
                    .expect("No logs message received");

                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack");
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    #[test]
    fn test_http_unknown_path_returns_404_even_for_get() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();
        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: false,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let (status, body) = send_http_request(http_listen, Method::GET, "/v1/unknown")
                    .await
                    .expect("http request should succeed");
                assert_eq!(status, http::StatusCode::NOT_FOUND);
                assert!(body.is_empty());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        _ = test_runtime.set_receiver(receiver).run_test(scenario);
    }

    #[test]
    fn test_http_known_path_returns_405_for_get() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();
        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 16,
            wait_for_result: false,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let (status, body) = send_http_request(http_listen, Method::GET, "/v1/logs")
                    .await
                    .expect("http request should succeed");
                assert_eq!(status, http::StatusCode::METHOD_NOT_ALLOWED);
                assert!(body.is_empty());

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        _ = test_runtime.set_receiver(receiver).run_test(scenario);
    }

    #[test]
    fn test_mixed_protocol_concurrent_traffic() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let grpc_listen: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{grpc_addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        // Enable wait_for_result for both
        config.grpc.wait_for_result = true;
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 1000,
            wait_for_result: true,
            ..Default::default()
        });

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let mut handles = Vec::new();

                // Launch 5 gRPC requests
                for _ in 0..5 {
                    let endpoint = grpc_endpoint.clone();
                    handles.push(tokio::spawn(async move {
                        let mut client = LogsServiceClient::connect(endpoint).await.unwrap();
                        let result = client.export(create_logs_service_request()).await;
                        assert!(result.is_ok(), "gRPC request failed");
                    }));
                }

                // Launch 5 HTTP requests
                for _ in 0..5 {
                    let addr = http_listen;
                    handles.push(tokio::spawn(async move {
                        let request = create_logs_service_request();
                        let mut request_bytes = Vec::new();
                        request.encode(&mut request_bytes).unwrap();
                        let (status, _) = post_otlp_http(addr, "/v1/logs", request_bytes)
                            .await
                            .unwrap();
                        assert_eq!(status, http::StatusCode::OK, "HTTP request failed");
                    }));
                }

                // Wait for all requests to complete
                for handle in handles {
                    handle.await.unwrap();
                }

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        // We expect 10 requests total
        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                for _ in 0..10 {
                    let pdata = timeout(Duration::from_secs(5), ctx.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received");

                    // Ack everything so the clients unblock and succeed
                    if let Some((_node_id, ack)) =
                        crate::pdata::Context::next_ack(AckMsg::new(pdata))
                    {
                        ctx.send_control_msg(NodeControlMsg::Ack(ack))
                            .await
                            .expect("Failed to send Ack");
                    }
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    #[test]
    fn test_shared_concurrency_across_grpc_and_http() {
        let test_runtime = TestRuntime::new();

        let addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{addr}:{grpc_port}");
        let grpc_listen: SocketAddr = format!("{addr}:{grpc_port}").parse().unwrap();

        let http_port = portpicker::pick_unused_port().expect("No free ports");
        let http_listen: SocketAddr = format!("{addr}:{http_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut config = test_config(grpc_listen);
        config.grpc.max_concurrent_requests = 1;
        config.grpc.wait_for_result = true;
        config.http = Some(HttpServerSettings {
            listening_addr: http_listen,
            max_concurrent_requests: 1,
            wait_for_result: false,
            timeout: Some(Duration::from_millis(200)),
            ..Default::default()
        });

        // Coordinate between scenario and validation so we only fire the HTTP request once the
        // gRPC request has definitely reached the receiver (and therefore acquired the shared
        // permit).
        let grpc_started = Arc::new(tokio::sync::Notify::new());
        let grpc_started_scenario = grpc_started.clone();
        let grpc_started_validation = grpc_started.clone();

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config,
                metrics: Arc::new(Mutex::new(
                    pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
                )),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let grpc_started = grpc_started_scenario.clone();

                // Start one gRPC request that will hold the single shared permit.
                let endpoint = grpc_endpoint.clone();
                let grpc_handle = tokio::spawn(async move {
                    let mut client = LogsServiceClient::connect(endpoint).await.unwrap();
                    client.export(create_logs_service_request()).await
                });

                // Wait until the validation side observes the gRPC request.
                timeout(Duration::from_secs(3), grpc_started.notified())
                    .await
                    .expect("Timed out waiting for gRPC to start");

                // While gRPC is in-flight, HTTP should be unable to acquire a permit and should
                // return 503 (permit acquisition timeout).
                let request = create_logs_service_request();
                let mut request_bytes = Vec::new();
                request.encode(&mut request_bytes).unwrap();

                let (status, _body) = post_otlp_http(http_listen, "/v1/logs", request_bytes)
                    .await
                    .expect("HTTP request should return a response");

                assert_eq!(
                    status,
                    http::StatusCode::SERVICE_UNAVAILABLE,
                    "HTTP should be rejected while gRPC holds the shared permit"
                );

                // The validation side will ACK the gRPC request; it should then complete.
                let grpc_result = grpc_handle.await.unwrap();
                assert!(grpc_result.is_ok(), "gRPC request should succeed after ACK");

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                let grpc_started = grpc_started_validation.clone();
                let pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for gRPC message")
                    .expect("No gRPC message received");

                grpc_started.notify_one();

                // Hold the request long enough for the HTTP request to observe permit contention.
                tokio::time::sleep(Duration::from_millis(300)).await;

                if let Some((_node_id, ack)) = crate::pdata::Context::next_ack(AckMsg::new(pdata)) {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack");
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }
}
