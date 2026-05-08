// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! STEF metrics receiver compatible with the Collector contrib STEF exporter.

use crate::stef_grpc::{
    STEF_DESTINATION_SERVICE_NAME, STEF_DESTINATION_STREAM_PATH, StefClientMessage,
    StefDataResponse, StefDestinationCapabilities, StefDictionaryLimits, StefServerMessage,
    stef_server_message,
};
use async_stream::try_stream;
use async_trait::async_trait;
use futures::Stream;
use futures::future::BoxFuture;
use http::{Request, Response};
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::MessageSourceSharedEffectHandlerExtension;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::clock;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ReceiverErrorKind, format_error_sources};
use otap_df_engine::memory_limiter::SharedReceiverAdmissionState;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::shared::receiver as shared;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::memory_pressure_layer::MemoryPressureLayer;
use otap_df_otap::otap_grpc::common;
use otap_df_otap::otap_grpc::otlp::server_new::OtlpServerSettings;
use otap_df_otap::otap_grpc::server_settings::GrpcServerSettings;
use otap_df_otap::otlp_metrics::OtlpReceiverMetrics;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_otap::tls_utils::{build_tls_acceptor, create_tls_stream};
use otap_df_pdata::stef::{
    METRICS_ROOT_STRUCT_NAME, METRICS_WIRE_SCHEMA, decode_metrics_otap_with_count,
};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::otel_info;
use parking_lot::Mutex;
use serde::Deserialize;
use serde_json::Value;
use std::convert::Infallible;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use std::task::Poll;
use std::time::Duration;
use tokio_util::sync::CancellationToken;
use tonic::body::Body;
use tonic::server::{Grpc, NamedService, StreamingService};
use tonic::transport::Server;
use tonic::{Response as TonicResponse, Status};
use tower::ServiceBuilder;
use tower::limit::GlobalConcurrencyLimitLayer;

/// The URN for the STEF receiver.
pub const STEF_RECEIVER_URN: &str = "urn:otel:receiver:stef";

const TELEMETRY_INTERVAL: Duration = Duration::from_secs(1);

/// Configuration for the STEF metrics receiver.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// gRPC server settings.
    #[serde(flatten)]
    pub grpc: GrpcServerSettings,

    /// Maximum advertised STEF dictionary bytes. `0` means no advertised limit.
    #[serde(default)]
    pub max_dict_bytes: u64,
}

/// Receiver that accepts STEF metrics over gRPC and forwards OTAP metrics records downstream.
pub struct StefReceiver {
    config: Config,
    metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
    admission_state: SharedReceiverAdmissionState,
}

/// Declares the STEF receiver factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static STEF_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: STEF_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        let mut receiver = StefReceiver::from_config(pipeline, &node_config.config)?;
        common::tune_max_concurrent_requests(
            &mut receiver.config.grpc,
            receiver_config.output_pdata_channel.capacity,
        );
        Ok(ReceiverWrapper::shared(
            receiver,
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

impl StefReceiver {
    /// Creates a STEF receiver from node configuration.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        let metrics = Arc::new(Mutex::new(
            pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
        ));
        Ok(Self {
            config,
            metrics,
            admission_state: SharedReceiverAdmissionState::from_process_state(
                &pipeline_ctx.memory_pressure_state(),
            ),
        })
    }
}

#[async_trait]
impl shared::Receiver<OtapPdata> for StefReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel<OtapPdata>,
        effect_handler: shared::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!(
            "stef.receiver.grpc.start",
            message = "Starting STEF gRPC receiver",
            endpoint = %self.config.grpc.listening_addr
        );

        let settings = self.config.grpc.build_settings();
        let listener = effect_handler.tcp_listener(self.config.grpc.listening_addr)?;
        let incoming = self.config.grpc.build_tcp_incoming(listener);
        let max_concurrent = self.config.grpc.max_concurrent_requests.max(1);
        let limit_layer = ServiceBuilder::new()
            .layer(MemoryPressureLayer::with_otlp_metrics(
                self.admission_state.clone(),
                self.metrics.clone(),
            ))
            .layer(GlobalConcurrencyLimitLayer::new(max_concurrent));

        let mut server =
            common::apply_server_tuning(Server::builder(), &self.config.grpc).layer(limit_layer);
        if let Some(timeout) = self.config.grpc.timeout {
            server = server.timeout(timeout);
        }

        let service = StefDestinationServer::new(
            effect_handler.clone(),
            settings,
            self.metrics.clone(),
            self.config.max_dict_bytes,
        );
        let server = server.add_service(service);

        let maybe_tls_acceptor = build_tls_acceptor(self.config.grpc.tls.as_ref())
            .await
            .map_err(|e| Error::ReceiverError {
                receiver: effect_handler.receiver_id(),
                kind: ReceiverErrorKind::Configuration,
                error: format!("Failed to configure TLS: {}", e),
                source_detail: format_error_sources(&e),
            })?;
        let handshake_timeout = self
            .config
            .grpc
            .tls
            .as_ref()
            .and_then(|tls| tls.handshake_timeout);
        let shutdown = CancellationToken::new();
        let mut server_task: Pin<
            Box<dyn Future<Output = Result<(), tonic::transport::Error>> + Send>,
        > = {
            let shutdown = shutdown.clone();
            match maybe_tls_acceptor {
                Some(tls_acceptor) => {
                    let tls_stream = create_tls_stream(incoming, tls_acceptor, handshake_timeout);
                    Box::pin(server.serve_with_incoming_shutdown(tls_stream, async move {
                        shutdown.cancelled().await;
                    }))
                }
                None => Box::pin(server.serve_with_incoming_shutdown(incoming, async move {
                    shutdown.cancelled().await;
                })),
            }
        };

        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(TELEMETRY_INTERVAL)
            .await?;

        loop {
            tokio::select! {
                biased;

                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            _ = metrics_reporter.report(&mut *self.metrics.lock());
                        }
                        Ok(NodeControlMsg::MemoryPressureChanged { update }) => {
                            self.admission_state.apply(update);
                        }
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            otel_info!("stef.receiver.drain_ingress");
                            shutdown.cancel();
                            _ = telemetry_cancel_handle.cancel().await;
                            effect_handler.notify_receiver_drained().await?;
                            return Ok(TerminalState::new(deadline, [self.metrics.lock().snapshot()]));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            otel_info!("stef.receiver.shutdown");
                            shutdown.cancel();
                            _ = telemetry_cancel_handle.cancel().await;
                            return Ok(TerminalState::new(deadline, [self.metrics.lock().snapshot()]));
                        }
                        Ok(_) => {}
                        Err(e) => {
                            shutdown.cancel();
                            _ = telemetry_cancel_handle.cancel().await;
                            return Err(Error::ChannelRecvError(e));
                        }
                    }
                }

                result = &mut server_task => {
                    _ = telemetry_cancel_handle.cancel().await;
                    if let Err(error) = result {
                        let source_detail = format_error_sources(&error);
                        return Err(Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Transport,
                            error: error.to_string(),
                            source_detail,
                        });
                    }
                    return Ok(TerminalState::new(clock::now(), [self.metrics.lock().snapshot()]));
                }
            }
        }
    }
}

#[derive(Clone)]
struct StefDestinationServer {
    effect_handler: shared::EffectHandler<OtapPdata>,
    settings: OtlpServerSettings,
    metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
    max_dict_bytes: u64,
}

impl StefDestinationServer {
    const fn new(
        effect_handler: shared::EffectHandler<OtapPdata>,
        settings: OtlpServerSettings,
        metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
        max_dict_bytes: u64,
    ) -> Self {
        Self {
            effect_handler,
            settings,
            metrics,
            max_dict_bytes,
        }
    }
}

impl tower::Service<Request<Body>> for StefDestinationServer {
    type Response = Response<Body>;
    type Error = Infallible;
    type Future = BoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&mut self, _cx: &mut std::task::Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }

    fn call(&mut self, req: Request<Body>) -> Self::Future {
        match req.uri().path() {
            STEF_DESTINATION_STREAM_PATH => {
                let common = self.clone();
                Box::pin(async move {
                    let mut grpc = Grpc::new(tonic_prost::ProstCodec::default());
                    if let Some(limit) = common.settings.max_decoding_message_size {
                        grpc = grpc.max_decoding_message_size(limit);
                    }
                    grpc = grpc.apply_compression_config(
                        common.settings.request_compression_encodings,
                        common.settings.response_compression_encodings,
                    );
                    let service = StefStreamService::new(
                        common.effect_handler,
                        common.metrics,
                        common.max_dict_bytes,
                    );
                    Ok(grpc.streaming(service, req).await)
                })
            }
            _ => Box::pin(async move { Ok(unimplemented_resp()) }),
        }
    }
}

impl NamedService for StefDestinationServer {
    const NAME: &'static str = STEF_DESTINATION_SERVICE_NAME;
}

struct StefStreamService {
    effect_handler: Option<shared::EffectHandler<OtapPdata>>,
    metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
    max_dict_bytes: u64,
}

impl StefStreamService {
    const fn new(
        effect_handler: shared::EffectHandler<OtapPdata>,
        metrics: Arc<Mutex<MetricSet<OtlpReceiverMetrics>>>,
        max_dict_bytes: u64,
    ) -> Self {
        Self {
            effect_handler: Some(effect_handler),
            metrics,
            max_dict_bytes,
        }
    }
}

impl StreamingService<StefClientMessage> for StefStreamService {
    type Response = StefServerMessage;
    type ResponseStream = Pin<Box<dyn Stream<Item = Result<StefServerMessage, Status>> + Send>>;
    type Future = BoxFuture<'static, Result<TonicResponse<Self::ResponseStream>, Status>>;

    #[allow(tail_expr_drop_order)]
    fn call(
        &mut self,
        request: tonic::Request<tonic::Streaming<StefClientMessage>>,
    ) -> Self::Future {
        let mut inbound = request.into_inner();
        let effect_handler = self
            .effect_handler
            .take()
            .expect("`StefStreamService` is not reused for multiple calls");
        let metrics = self.metrics.clone();
        let max_dict_bytes = self.max_dict_bytes;

        Box::pin(async move {
            metrics.lock().requests_started.inc();
            let first = inbound
                .message()
                .await?
                .ok_or_else(|| Status::invalid_argument("missing STEF first message"))?;
            let first_message = first
                .first_message
                .ok_or_else(|| Status::invalid_argument("missing STEF first message"))?;
            if first_message.root_struct_name != METRICS_ROOT_STRUCT_NAME {
                return Err(Status::invalid_argument("unsupported STEF root struct"));
            }
            if !first.stef_bytes.is_empty() || first.is_end_of_chunk {
                return Err(Status::invalid_argument(
                    "STEF first message must not include data",
                ));
            }

            let output = try_stream! {
                yield capabilities_message(max_dict_bytes);

                let mut chunk = Vec::with_capacity(64 * 1024);
                let mut ack_record_id = 0_u64;
                while let Some(message) = inbound.message().await? {
                    if message.first_message.is_some() {
                        Err(Status::invalid_argument("unexpected STEF first message"))?;
                    }
                    metrics.lock().request_bytes.add(message.stef_bytes.len() as u64);
                    chunk.extend_from_slice(&message.stef_bytes);
                    if !message.is_end_of_chunk {
                        continue;
                    }

                    let (otap_records, point_count) = decode_metrics_otap_with_count(&chunk)
                        .map_err(|e| Status::invalid_argument(format!("STEF decode error: {e}")))?;
                    let pdata = OtapPdata::new(
                        Context::default(),
                        otap_records.into(),
                    );
                    effect_handler
                        .send_message_with_source_node(pdata)
                        .await
                        .map_err(|e| Status::internal(format!("failed to send to pipeline: {e}")))?;

                    ack_record_id = ack_record_id.saturating_add(point_count);
                    chunk.clear();
                    yield ack_message(ack_record_id);
                }

                metrics.lock().requests_completed.inc();
            };

            Ok(TonicResponse::new(Box::pin(output) as Self::ResponseStream))
        })
    }
}

fn capabilities_message(max_dict_bytes: u64) -> StefServerMessage {
    StefServerMessage {
        message: Some(stef_server_message::Message::Capabilities(
            StefDestinationCapabilities {
                dictionary_limits: Some(StefDictionaryLimits { max_dict_bytes }),
                schema: METRICS_WIRE_SCHEMA.to_vec(),
            },
        )),
    }
}

fn ack_message(ack_record_id: u64) -> StefServerMessage {
    StefServerMessage {
        message: Some(stef_server_message::Message::Response(StefDataResponse {
            ack_record_id,
            bad_data_record_id_ranges: Vec::new(),
        })),
    }
}

fn unimplemented_resp() -> Response<Body> {
    let mut response = Response::new(Body::default());
    let headers = response.headers_mut();
    _ = headers.insert(
        Status::GRPC_STATUS,
        (tonic::Code::Unimplemented as i32).into(),
    );
    _ = headers.insert(
        http::header::CONTENT_TYPE,
        tonic::metadata::GRPC_CONTENT_TYPE,
    );
    response
}
