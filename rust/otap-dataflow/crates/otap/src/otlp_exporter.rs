// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Asynchronous OTLP exporter implementation.
//!
//! The exporter receives pipeline messages on a single-threaded Tokio runtime. Each payload is
//! encoded (when necessary) and handed off to a gRPC export RPC. We keep the gRPC futures in a
//! lightweight in-flight queue which enforces the configured concurrency limit. As soon as a
//! request finishes we forward the Ack/Nack to the pipeline controller so the dataflow can make
//! progress.

use crate::OTAP_EXPORTER_FACTORIES;
use crate::metrics::ExporterPDataMetrics;
use crate::otap_grpc::client_settings::GrpcClientSettings;
use crate::otap_grpc::otlp::client::{LogsServiceClient, MetricsServiceClient, TraceServiceClient};
use crate::pdata::{Context, OtapPdata};
use async_trait::async_trait;
use bytes::Bytes;
use futures::future::FutureExt;
use futures::stream::{FuturesUnordered, StreamExt};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ExporterErrorKind, format_error_sources};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::otlp::logs::LogsProtoBytesEncoder;
use otap_df_pdata::otlp::metrics::MetricsProtoBytesEncoder;
use otap_df_pdata::otlp::traces::TracesProtoBytesEncoder;
use otap_df_pdata::otlp::{ProtoBuffer, ProtoBytesEncoder};
use otap_df_pdata::{OtapArrowRecords, OtapPayload, OtapPayloadHelpers, OtlpProtoBytes};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_debug, otel_info};
use serde::Deserialize;
use std::future::Future;
use std::sync::Arc;
use std::time::Duration;
use tonic::codec::CompressionEncoding;
use tonic::transport::Channel;

/// The URN for the OTLP exporter
pub const OTLP_EXPORTER_URN: &str = "urn:otel:otlp:exporter";

/// Configuration for the OTLP Exporter
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Shared gRPC client settings reused across OTLP exports.
    #[serde(flatten)]
    pub grpc: GrpcClientSettings,
    /// Maximum number of concurrent in-flight export RPCs.
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight: usize,
}

pub(crate) const fn default_max_in_flight() -> usize {
    5
}

/// Exporter that sends OTLP data via gRPC
pub struct OTLPExporter {
    config: Config,
    pdata_metrics: MetricSet<ExporterPDataMetrics>,
}

/// Declare the OTLP Exporter as a local exporter factory
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static OTLP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTLP_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            OTLPExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

impl OTLPExporter {
    /// create a new instance of the `[OTLPExporter]` from json config value
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let pdata_metrics = pipeline_ctx.register_metrics::<ExporterPDataMetrics>();

        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        Ok(Self {
            config,
            pdata_metrics,
        })
    }
}

#[async_trait(?Send)]
impl Exporter<OtapPdata> for OTLPExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!(
            "otlp.exporter.grpc.start",
            grpc_endpoint = self.config.grpc.grpc_endpoint.as_str()
        );

        self.config.grpc.log_proxy_info();

        let exporter_id = effect_handler.exporter_id();
        let timer_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let channel = self
            .config
            .grpc
            .connect_channel_lazy(None)
            .await
            .map_err(|e| {
                let source_detail = format_error_sources(&e);
                Error::ExporterError {
                    exporter: exporter_id.clone(),
                    kind: ExporterErrorKind::Connect,
                    error: format!("grpc channel error {e}"),
                    source_detail,
                }
            })?;

        let compression = self.config.grpc.compression_encoding();
        let max_in_flight = self.config.max_in_flight.max(1);

        // reuse the encoder and the buffer across pdatas
        let mut logs_proto_encoder = LogsProtoBytesEncoder::new();
        let mut metrics_proto_encoder = MetricsProtoBytesEncoder::new();
        let mut traces_proto_encoder = TracesProtoBytesEncoder::new();

        let mut logs_proto_buffer = ProtoBuffer::with_capacity(8 * 1024);
        let mut metrics_proto_buffer = ProtoBuffer::with_capacity(8 * 1024);
        let mut traces_proto_buffer = ProtoBuffer::with_capacity(8 * 1024);

        let mut grpc_clients = GrpcClientPool::new(max_in_flight, channel, compression);
        grpc_clients.prepopulate_clients();

        let mut inflight_exports = InFlightExports::new();
        let mut pending_msg: Option<Message<OtapPdata>> = None;

        // Main loop: 1) finish ready completions, 2) biased wait for either a completion
        // or the next message, 3) dispatch work while respecting the in-flight budget.
        loop {
            // Backpressure guard: when full and a message is parked, only drain completions.
            if inflight_exports.len() >= max_in_flight && pending_msg.is_some() {
                if let Some(completed) = inflight_exports.next_completion().await {
                    let client = finalize_completed_export(
                        completed,
                        &effect_handler,
                        &mut self.pdata_metrics,
                    )
                    .await;
                    grpc_clients.release(client);
                }
                continue;
            }

            // Opportunistically drain completions before we park on a recv.
            while let Some(completed) = inflight_exports.next_completion().now_or_never().flatten()
            {
                let client =
                    finalize_completed_export(completed, &effect_handler, &mut self.pdata_metrics)
                        .await;
                grpc_clients.release(client);
            }

            // Prefer completions if any are ready, otherwise biased select between completion and recv.
            let msg = if let Some(msg) = pending_msg.take() {
                msg
            } else if inflight_exports.is_empty() {
                let msg = msg_chan.recv().await?;
                otel_debug!("otlp.exporter.grpc.receive");
                msg
            } else {
                let completion_fut = inflight_exports.next_completion().fuse();
                let recv_fut = msg_chan.recv().fuse();
                futures::pin_mut!(completion_fut, recv_fut);

                futures::select_biased! {
                    completed = completion_fut => {
                        if let Some(completed) = completed {
                            let client = finalize_completed_export(
                                completed,
                                &effect_handler,
                                &mut self.pdata_metrics,
                            )
                            .await;
                            grpc_clients.release(client);
                        }
                        continue;
                    }
                    msg = recv_fut => {
                        let msg = msg?;
                        otel_debug!("otlp.exporter.grpc.receive");
                        msg
                    },
                }
            };

            match msg {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    otel_info!("otlp.exporter.grpc.shutdown");
                    debug_assert!(
                        pending_msg.is_none(),
                        "pending message should have been drained before shutdown"
                    );
                    while !inflight_exports.is_empty() {
                        if let Some(completed) = inflight_exports.next_completion().await {
                            let client = finalize_completed_export(
                                completed,
                                &effect_handler,
                                &mut self.pdata_metrics,
                            )
                            .await;
                            grpc_clients.release(client);
                        }
                    }
                    _ = timer_cancel_handle.cancel().await;
                    return Ok(TerminalState::new(deadline, [self.pdata_metrics]));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.pdata_metrics);
                }
                Message::PData(pdata) => {
                    if inflight_exports.len() >= max_in_flight {
                        pending_msg = Some(Message::PData(pdata));
                        continue;
                    }

                    let signal_type = pdata.signal_type();
                    let (context, payload) = pdata.into_parts();
                    self.pdata_metrics.inc_consumed(signal_type);

                    // Dispatch based on signal type and the concrete payload representation.
                    match (signal_type, payload) {
                        (SignalType::Logs, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            dispatch_otap_export(
                                otap_batch,
                                context,
                                SignalType::Logs,
                                &exporter_id,
                                &mut logs_proto_buffer,
                                &mut logs_proto_encoder,
                                |encoded| {
                                    let client = SignalClient::Logs(grpc_clients.take_logs());
                                    make_export_future(encoded, client)
                                },
                                &mut inflight_exports,
                                &mut self.pdata_metrics.logs_failed,
                                &effect_handler,
                            )
                            .await;
                        }
                        (SignalType::Metrics, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            dispatch_otap_export(
                                otap_batch,
                                context,
                                SignalType::Metrics,
                                &exporter_id,
                                &mut metrics_proto_buffer,
                                &mut metrics_proto_encoder,
                                |encoded| {
                                    let client = SignalClient::Metrics(grpc_clients.take_metrics());
                                    make_export_future(encoded, client)
                                },
                                &mut inflight_exports,
                                &mut self.pdata_metrics.metrics_failed,
                                &effect_handler,
                            )
                            .await;
                        }
                        (SignalType::Traces, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            dispatch_otap_export(
                                otap_batch,
                                context,
                                SignalType::Traces,
                                &exporter_id,
                                &mut traces_proto_buffer,
                                &mut traces_proto_encoder,
                                |encoded| {
                                    let client = SignalClient::Traces(grpc_clients.take_traces());
                                    make_export_future(encoded, client)
                                },
                                &mut inflight_exports,
                                &mut self.pdata_metrics.traces_failed,
                                &effect_handler,
                            )
                            .await;
                        }
                        (_, OtapPayload::OtlpBytes(service_req)) => {
                            let prepared = match service_req {
                                OtlpProtoBytes::ExportLogsRequest(bytes) => {
                                    prepare_otlp_export(bytes, context, SignalType::Logs, |b| {
                                        OtlpProtoBytes::ExportLogsRequest(b).into()
                                    })
                                }
                                OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                                    prepare_otlp_export(bytes, context, SignalType::Metrics, |b| {
                                        OtlpProtoBytes::ExportMetricsRequest(b).into()
                                    })
                                }
                                OtlpProtoBytes::ExportTracesRequest(bytes) => {
                                    prepare_otlp_export(bytes, context, SignalType::Traces, |b| {
                                        OtlpProtoBytes::ExportTracesRequest(b).into()
                                    })
                                }
                            };

                            let client = match signal_type {
                                SignalType::Logs => SignalClient::Logs(grpc_clients.take_logs()),
                                SignalType::Metrics => {
                                    SignalClient::Metrics(grpc_clients.take_metrics())
                                }
                                SignalType::Traces => {
                                    SignalClient::Traces(grpc_clients.take_traces())
                                }
                            };
                            let future = make_export_future(prepared, client);
                            inflight_exports.push(future);
                        }
                    }
                }
                _ => {
                    // ignore unhandled messages
                }
            }
        }
    }
}

/// Helper function to handle export result and send Ack/Nack accordingly.
async fn route_export_result<T>(
    result: Result<T, tonic::Status>,
    context: Context,
    saved_payload: OtapPayload,
    effect_handler: &EffectHandler<OtapPdata>,
) -> Result<(), Error> {
    match result {
        Ok(_) => {
            effect_handler
                .notify_ack(AckMsg::new(OtapPdata::new(context, saved_payload)))
                .await?;
            Ok(())
        }
        Err(e) => {
            let error_msg = e.to_string();
            effect_handler
                .notify_nack(NackMsg::new(
                    &error_msg,
                    OtapPdata::new(context, saved_payload),
                ))
                .await?;
            let source_detail = format_error_sources(&e);
            Err(Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                kind: ExporterErrorKind::Transport,
                error: error_msg,
                source_detail,
            })
        }
    }
}

struct EncodedExport {
    bytes: Bytes,
    context: Context,
    saved_payload: OtapPayload,
    signal_type: SignalType,
}

/// Encoding failed before the request was sent; we still need to surface a Nack with payload.
struct EncodingFailure {
    error: Error,
    context: Context,
    saved_payload: OtapPayload,
}

fn prepare_otap_export<Enc: ProtoBytesEncoder>(
    mut otap_batch: OtapArrowRecords,
    context: Context,
    proto_buffer: &mut ProtoBuffer,
    encoder: &mut Enc,
    exporter: &NodeId,
    signal_type: SignalType,
) -> Result<EncodedExport, Box<EncodingFailure>> {
    proto_buffer.clear();
    if let Err(e) = encoder.encode(&mut otap_batch, proto_buffer) {
        let error = Error::ExporterError {
            exporter: exporter.clone(),
            kind: ExporterErrorKind::Other,
            error: format!("encoding error: {}", e),
            source_detail: "".to_string(),
        };

        if !context.may_return_payload() {
            let _drop = otap_batch.take_payload();
        }
        let saved_payload: OtapPayload = otap_batch.into();

        return Err(Box::new(EncodingFailure {
            error,
            context,
            saved_payload,
        }));
    }

    // Maintain the buffer's capacity across repeated calls.
    let (bytes, next_capacity) = proto_buffer.take_into_bytes();
    proto_buffer.ensure_capacity(next_capacity);

    if !context.may_return_payload() {
        // drop before the export, payload not requested
        let _drop = otap_batch.take_payload();
    }
    let saved_payload: OtapPayload = otap_batch.into();

    Ok(EncodedExport {
        bytes,
        context,
        saved_payload,
        signal_type,
    })
}

fn prepare_otlp_export(
    bytes: Bytes,
    context: Context,
    signal_type: SignalType,
    save_payload_fn: impl FnOnce(Bytes) -> OtapPayload,
) -> EncodedExport {
    let saved_payload = if context.may_return_payload() {
        save_payload_fn(bytes.clone())
    } else {
        save_payload_fn(Bytes::new())
    };

    EncodedExport {
        bytes,
        context,
        saved_payload,
        signal_type,
    }
}

/// Encode an OTAP Arrow batch and enqueue the export task; on encoding failure, emit a Nack.
#[allow(clippy::too_many_arguments)]
async fn dispatch_otap_export<Enc, Fut, MakeFuture>(
    otap_batch: OtapArrowRecords,
    context: Context,
    signal_type: SignalType,
    exporter_id: &NodeId,
    proto_buffer: &mut ProtoBuffer,
    encoder: &mut Enc,
    make_future: MakeFuture,
    inflight: &mut InFlightExports<Fut, CompletedExport>,
    failed_counter: &mut Counter<u64>,
    effect_handler: &EffectHandler<OtapPdata>,
) where
    Enc: ProtoBytesEncoder,
    Fut: Future<Output = CompletedExport>,
    MakeFuture: FnOnce(EncodedExport) -> Fut,
{
    match prepare_otap_export(
        otap_batch,
        context,
        proto_buffer,
        encoder,
        exporter_id,
        signal_type,
    ) {
        Ok(encoded) => {
            inflight.push(make_future(encoded));
        }
        Err(error) => {
            failed_counter.inc();
            _ = notify_prepare_error(error, effect_handler).await;
        }
    }
}

async fn notify_prepare_error(
    error: Box<EncodingFailure>,
    effect_handler: &EffectHandler<OtapPdata>,
) -> Result<(), Error> {
    let EncodingFailure {
        error,
        context,
        saved_payload,
    } = *error;

    effect_handler
        .notify_nack(NackMsg::new(
            error.to_string(),
            OtapPdata::new(context, saved_payload),
        ))
        .await?;

    Ok(())
}

/// Applies the Ack/Nack side effects for a completed gRPC export and returns the reusable client.
async fn finalize_completed_export(
    completed: CompletedExport,
    effect_handler: &EffectHandler<OtapPdata>,
    pdata_metrics: &mut MetricSet<ExporterPDataMetrics>,
) -> SignalClient {
    let CompletedExport {
        result,
        context,
        saved_payload,
        signal_type,
        client,
    } = completed;

    match route_export_result(result, context, saved_payload, effect_handler).await {
        Ok(()) => pdata_metrics.add_exported(signal_type, 1),
        Err(_) => pdata_metrics.add_failed(signal_type, 1),
    }

    client
}

/// Builds an export future for the provided payload, borrowing a signal-specific client from the pool.
fn make_export_future(
    prepared: EncodedExport,
    client: SignalClient,
) -> impl Future<Output = CompletedExport> {
    let EncodedExport {
        bytes,
        context,
        saved_payload,
        signal_type,
    } = prepared;

    async move {
        match client {
            SignalClient::Logs(mut client) => {
                let result = client.export(bytes).await.map(|_| ());
                CompletedExport {
                    result,
                    context,
                    saved_payload,
                    signal_type,
                    client: SignalClient::Logs(client),
                }
            }
            SignalClient::Metrics(mut client) => {
                let result = client.export(bytes).await.map(|_| ());
                CompletedExport {
                    result,
                    context,
                    saved_payload,
                    signal_type,
                    client: SignalClient::Metrics(client),
                }
            }
            SignalClient::Traces(mut client) => {
                let result = client.export(bytes).await.map(|_| ());
                CompletedExport {
                    result,
                    context,
                    saved_payload,
                    signal_type,
                    client: SignalClient::Traces(client),
                }
            }
        }
    }
}

/// FIFO-ish wrapper around the in-flight export RPCs.
pub(crate) struct InFlightExports<Fut, Output>
where
    Fut: Future<Output = Output>,
{
    futures: FuturesUnordered<Fut>,
}

impl<Fut, Output> InFlightExports<Fut, Output>
where
    Fut: Future<Output = Output>,
{
    pub(crate) fn new() -> Self {
        Self {
            futures: FuturesUnordered::new(),
        }
    }

    pub(crate) fn len(&self) -> usize {
        self.futures.len()
    }

    pub(crate) fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    pub(crate) fn push(&mut self, future: Fut) {
        self.futures.push(future);
    }

    /// Returns a future that resolves once the next export finishes.
    pub(crate) fn next_completion(&mut self) -> impl Future<Output = Option<Output>> + '_ {
        self.futures.next()
    }
}

/// Keeps a small stash of gRPC clients so each export can reuse an existing connection.
struct GrpcClientPool {
    base_channel: Channel,
    compression: Option<CompressionEncoding>,
    logs: Vec<LogsServiceClient<Channel>>,
    metrics: Vec<MetricsServiceClient<Channel>>,
    traces: Vec<TraceServiceClient<Channel>>,
}

impl GrpcClientPool {
    fn new(
        max_in_flight: usize,
        base_channel: Channel,
        compression: Option<CompressionEncoding>,
    ) -> Self {
        Self {
            base_channel,
            compression,
            logs: Vec::with_capacity(max_in_flight),
            metrics: Vec::with_capacity(max_in_flight),
            traces: Vec::with_capacity(max_in_flight),
        }
    }

    /// Eagerly build up to `max_in_flight` clients per signal to avoid first-call setup.
    fn prepopulate_clients(&mut self) {
        let logs_cap = self.logs.capacity();
        for _ in 0..logs_cap {
            self.logs.push(self.make_logs_client());
        }

        let metrics_cap = self.metrics.capacity();
        for _ in 0..metrics_cap {
            self.metrics.push(self.make_metrics_client());
        }

        let traces_cap = self.traces.capacity();
        for _ in 0..traces_cap {
            self.traces.push(self.make_traces_client());
        }
    }

    #[inline(always)]
    fn take_logs(&mut self) -> LogsServiceClient<Channel> {
        self.logs
            .pop()
            .expect("client pool underflow: take_logs called with empty pool")
    }

    #[inline(always)]
    fn take_metrics(&mut self) -> MetricsServiceClient<Channel> {
        self.metrics
            .pop()
            .expect("client pool underflow: take_metrics called with empty pool")
    }

    #[inline(always)]
    fn take_traces(&mut self) -> TraceServiceClient<Channel> {
        self.traces
            .pop()
            .expect("client pool underflow: take_traces called with empty pool")
    }

    fn release(&mut self, client: SignalClient) {
        match client {
            SignalClient::Logs(client) => self.logs.push(client),
            SignalClient::Metrics(client) => self.metrics.push(client),
            SignalClient::Traces(client) => self.traces.push(client),
        }
    }

    fn make_logs_client(&self) -> LogsServiceClient<Channel> {
        let mut client = LogsServiceClient::new(self.base_channel.clone());
        if let Some(encoding) = self.compression {
            client = client.send_compressed(encoding);
        }
        client
    }

    fn make_metrics_client(&self) -> MetricsServiceClient<Channel> {
        let mut client = MetricsServiceClient::new(self.base_channel.clone());
        if let Some(encoding) = self.compression {
            client = client.send_compressed(encoding);
        }
        client
    }

    fn make_traces_client(&self) -> TraceServiceClient<Channel> {
        let mut client = TraceServiceClient::new(self.base_channel.clone());
        if let Some(encoding) = self.compression {
            client = client.send_compressed(encoding);
        }
        client
    }
}

enum SignalClient {
    Logs(LogsServiceClient<Channel>),
    Metrics(MetricsServiceClient<Channel>),
    Traces(TraceServiceClient<Channel>),
}

/// Captures everything we need once a single export RPC has completed.
struct CompletedExport {
    result: Result<(), tonic::Status>,
    context: Context,
    saved_payload: OtapPayload,
    signal_type: SignalType,
    client: SignalClient,
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::otlp_grpc::OTLPData;
    use crate::otlp_mock::{LogsServiceMock, MetricsServiceMock, TraceServiceMock};
    use crate::testing::TestCallData;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::Interests;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::{
        exporter::{TestContext, TestRuntime},
        test_node,
    };
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::logs::v1::logs_service_server::LogsServiceServer;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::metrics_service_server::MetricsServiceServer;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::ExportTraceServiceRequest;
    use otap_df_pdata::proto::opentelemetry::collector::trace::v1::trace_service_server::TraceServiceServer;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use prost::Message;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::time::Instant;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};
    use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;

    // Imports only used by tests that are skipped on Windows
    #[cfg(not(windows))]
    use {
        otap_df_engine::control::{Controllable, PipelineCtrlMsgSender, pipeline_ctrl_msg_channel},
        otap_df_engine::local::message::{LocalReceiver, LocalSender},
        otap_df_engine::message::{Receiver, Sender},
        otap_df_engine::node::NodeWithPDataReceiver,
        otap_df_engine::testing::create_not_send_channel,
        otap_df_telemetry::metrics::MetricSetSnapshot,
        otap_df_telemetry::reporter::MetricsReporter,
    };

    /// Helper function to wait for and validate an Ack or Nack message with the expected node_id
    async fn wait_for_ack_or_nack(
        pipeline_ctrl_rx: &mut otap_df_engine::control::PipelineCtrlMsgReceiver<OtapPdata>,
        expect_ack: bool,
        expected_node_id: usize,
        context: &str,
    ) -> Result<(), String> {
        let result = timeout(Duration::from_secs(1), async {
            loop {
                match pipeline_ctrl_rx.recv().await {
                    Ok(otap_df_engine::control::PipelineControlMsg::DeliverAck {
                        node_id, ..
                    }) => {
                        if !expect_ack {
                            return Err(format!("Got Ack but expected Nack {}", context));
                        }
                        if node_id != expected_node_id {
                            return Err(format!(
                                "Expected node_id {} but got {} {}",
                                expected_node_id, node_id, context
                            ));
                        }
                        return Ok(());
                    }
                    Ok(otap_df_engine::control::PipelineControlMsg::DeliverNack {
                        node_id,
                        ..
                    }) => {
                        if expect_ack {
                            return Err(format!("Got Nack but expected Ack {}", context));
                        }
                        if node_id != expected_node_id {
                            return Err(format!(
                                "Expected node_id {} but got {} {}",
                                expected_node_id, node_id, context
                            ));
                        }
                        return Ok(());
                    }
                    Ok(_) => continue, // Skip non-Ack/Nack messages
                    Err(_) => return Err(format!("Channel closed {}", context)),
                }
            }
        })
        .await;

        match result {
            Ok(Ok(())) => Ok(()),
            Ok(Err(e)) => Err(e),
            Err(_) => Err(format!("Timeout waiting for Ack/Nack {}", context)),
        }
    }

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario() -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // Send a data message
                let req = ExportLogsServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                let logs_pdata = OtapPdata::new_default(
                    OtlpProtoBytes::ExportLogsRequest(Bytes::from(req_bytes)).into(),
                )
                .test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    123,
                );
                ctx.send_pdata(logs_pdata)
                    .await
                    .expect("Failed to send log message");

                let req = ExportMetricsServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                let metrics_pdata = OtapPdata::new_default(
                    OtlpProtoBytes::ExportMetricsRequest(Bytes::from(req_bytes)).into(),
                )
                .test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    123,
                );
                ctx.send_pdata(metrics_pdata)
                    .await
                    .expect("Failed to send metric message");

                let req = ExportTraceServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                let traces_pdata = OtapPdata::new_default(
                    OtlpProtoBytes::ExportTracesRequest(Bytes::from(req_bytes)).into(),
                )
                .test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    TestCallData::default().into(),
                    123,
                );
                ctx.send_pdata(traces_pdata)
                    .await
                    .expect("Failed to send metric message");

                // Send shutdown
                ctx.send_shutdown(Instant::now() + Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(
        mut receiver: tokio::sync::mpsc::Receiver<OTLPData>,
    ) -> impl FnOnce(TestContext<OtapPdata>, Result<(), Error>) -> Pin<Box<dyn Future<Output = ()>>>
    {
        |_, exporter_result| {
            Box::pin(async move {
                exporter_result.unwrap();

                // check that the message was properly sent from the exporter
                let logs_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message");
                // Assert that the message received is what the exporter sent
                let _expected_logs_message = ExportLogsServiceRequest::default();
                assert!(matches!(logs_received, _expected_logs_message));

                let metrics_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                let _expected_metrics_message = ExportMetricsServiceRequest::default();
                assert!(matches!(metrics_received, _expected_metrics_message));

                let traces_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");
                let _expected_trace_message = ExportTraceServiceRequest::default();
                assert!(matches!(traces_received, _expected_trace_message));
            })
        }
    }

    #[test]
    fn test_otlp_exporter() {
        let test_runtime = TestRuntime::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        // tokio runtime to run grpc server in the background
        let tokio_rt = Runtime::new().unwrap();

        // run a gRPC concurrently to receive data from the exporter
        _ = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            // Signal that the server is ready to accept connections
            let _ = ready_sender.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            let mock_logs_service = LogsServiceServer::new(LogsServiceMock::new(sender.clone()));
            let mock_metrics_service =
                MetricsServiceServer::new(MetricsServiceMock::new(sender.clone()));
            let mock_trace_service = TraceServiceServer::new(TraceServiceMock::new(sender.clone()));
            Server::builder()
                .add_service(mock_logs_service)
                .add_service(mock_metrics_service)
                .add_service(mock_trace_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    // Wait for the shutdown signal
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("Test gRPC server has failed");
        });

        // Wait for the server to be ready before creating the exporter
        tokio_rt
            .block_on(ready_receiver)
            .expect("Server failed to start");

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc: GrpcClientSettings {
                        grpc_endpoint: grpc_endpoint.clone(),
                        ..Default::default()
                    },
                    max_in_flight: 32,
                },
                pdata_metrics: pipeline_ctx.register_metrics::<ExporterPDataMetrics>(),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(|mut ctx, result| {
                Box::pin(async move {
                    // Validate that we received 3 Acks
                    let mut ack_count = 0;
                    let mut pipeline_ctrl_rx = ctx.take_pipeline_ctrl_receiver().unwrap();

                    // Validate that we received 3 Acks with correct node_id
                    for i in 0..3 {
                        wait_for_ack_or_nack(
                            &mut pipeline_ctrl_rx,
                            true,
                            123,
                            &format!("for export #{}", i + 1),
                        )
                        .await
                        .expect("Failed to receive Ack");
                        ack_count += 1;
                    }

                    assert_eq!(ack_count, 3, "Expected 3 Acks for 3 successful exports");
                    validation_procedure(receiver)(ctx, result).await;
                })
            });

        _ = shutdown_sender.send("Shutdown");
    }

    // Skipping on Windows due to flakiness: https://github.com/open-telemetry/otel-arrow/issues/1611
    #[cfg(not(windows))]
    #[test]
    fn test_receiver_not_ready_on_start_and_reconnect() {
        // the purpose of this test is to that the exporter behaves as expected in the face of
        // server that may start and stop asynchronously of the exporter. it ensures the exporter
        // doesn't exit early if it can't make the initial connection, and also that the grpc
        // client will reconnect in the event of a server shutdown

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");

        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle.clone());
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc: GrpcClientSettings {
                        grpc_endpoint: grpc_endpoint.clone(),
                        ..Default::default()
                    },
                    max_in_flight: 32,
                },
                pdata_metrics: pipeline_ctx.register_metrics::<ExporterPDataMetrics>(),
            },
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (pipeline_ctrl_msg_tx, pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(2);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        // channels for coordinating the test
        let (server_startup_sender, mut server_startup_receiver) = tokio::sync::mpsc::channel(1);
        let (server_start_ack_sender, server_start_ack_receiver) = tokio::sync::mpsc::channel(1);
        let (shutdown_sender1, shutdown_signal1) = tokio::sync::oneshot::channel();
        let (shutdown_sender2, shutdown_signal2) = tokio::sync::oneshot::channel();
        let (server_shutdown_ack_sender, server_shutdown_ack_receiver) =
            tokio::sync::mpsc::channel(1);
        let (req_sender, req_receiver) = tokio::sync::mpsc::channel(32);

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<OtapPdata>,
            metrics_reporter: MetricsReporter,
        ) -> Result<(), Error> {
            exporter
                .start(pipeline_ctrl_msg_tx, metrics_reporter)
                .await
                .map(|_| ())
        }

        async fn drive_test(
            server_startup_sender: tokio::sync::mpsc::Sender<bool>,
            mut server_startup_ack_receiver: tokio::sync::mpsc::Receiver<bool>,
            mut server_shutdown_ack_receiver: tokio::sync::mpsc::Receiver<bool>,
            server_shutdown_signal1: tokio::sync::oneshot::Sender<bool>,
            server_shutdown_signal2: tokio::sync::oneshot::Sender<bool>,
            pdata_tx: Sender<OtapPdata>,
            control_sender: Sender<NodeControlMsg<OtapPdata>>,
            mut pipeline_ctrl_msg_rx: otap_df_engine::control::PipelineCtrlMsgReceiver<OtapPdata>,
            mut req_receiver: tokio::sync::mpsc::Receiver<OTLPData>,
            metrics_receiver: flume::Receiver<MetricSetSnapshot>,
            metrics_reporter: MetricsReporter,
        ) -> Result<(), Error> {
            // pdata
            let req = ExportLogsServiceRequest::default();
            let mut req_bytes = vec![];
            req.encode(&mut req_bytes).unwrap();

            // send a request while the server isn't running and check how we handle it
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone().into()),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();
            // Wait for NACK since server is down
            wait_for_ack_or_nack(&mut pipeline_ctrl_msg_rx, false, 123, "when server is down")
                .await
                .expect("Expected Nack when server down");

            // wait a bit before starting the server. This will ensure the exporter no-longer exits
            // when start is called if the endpoint can't be reached
            tokio::time::sleep(Duration::from_millis(100)).await;
            server_startup_sender.send(true).await.unwrap();
            _ = server_startup_ack_receiver.recv().await.unwrap();

            // send a pdata
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone().into()),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();
            // ensure server got request
            _ = req_receiver.recv().await.unwrap();
            // Wait for ACK since server is up
            wait_for_ack_or_nack(&mut pipeline_ctrl_msg_rx, true, 123, "when server is up")
                .await
                .expect("Expected Ack when server up");

            // stop the server
            server_shutdown_signal1.send(true).unwrap();
            _ = server_shutdown_ack_receiver.recv().await.unwrap();

            // send a request while the server isn't running and check that we still handle it correctly
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone().into()),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();
            // Wait for NACK since server is down again
            wait_for_ack_or_nack(
                &mut pipeline_ctrl_msg_rx,
                false,
                123,
                "when server is down again",
            )
            .await
            .expect("Expected Nack when server down again");

            // restart the server
            server_startup_sender.send(true).await.unwrap();
            _ = server_startup_ack_receiver.recv().await.unwrap();

            // send another pdata. This ensures the client can reconnect after it was shut down
            let pdata = OtapPdata::new_default(OtapPayload::OtlpBytes(
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone().into()),
            ))
            .test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                TestCallData::default().into(),
                123,
            );
            pdata_tx.send(pdata).await.unwrap();
            _ = req_receiver.recv().await.unwrap();
            // Wait for ACK after reconnect
            wait_for_ack_or_nack(&mut pipeline_ctrl_msg_rx, true, 123, "after reconnect")
                .await
                .expect("Expected Ack after reconnect");

            // check the metrics:
            control_sender
                .send(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: metrics_reporter.clone(),
                })
                .await
                .unwrap();
            let metrics = metrics_receiver.recv_async().await.unwrap();
            let logs_exported_count = metrics.get_metrics()[4].to_u64_lossy(); // logs exported
            assert_eq!(logs_exported_count, 2);
            let logs_failed_count = metrics.get_metrics()[5].to_u64_lossy(); // logs failed
            assert_eq!(logs_failed_count, 2);

            control_sender
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_millis(10),
                    reason: "shutting down".into(),
                })
                .await
                .unwrap();

            server_shutdown_signal2.send(true).unwrap();

            Ok(())
        }

        async fn run_server(
            listening_addr: String,
            startup_ack_sender: tokio::sync::mpsc::Sender<bool>,
            shutdown_signal: tokio::sync::oneshot::Receiver<bool>,
            req_sender: tokio::sync::mpsc::Sender<OTLPData>,
        ) {
            let listening_addr: SocketAddr = listening_addr.to_string().parse().unwrap();
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let tcp_stream = TcpListenerStream::new(tcp_listener);

            let logs_service = LogsServiceServer::new(LogsServiceMock::new(req_sender));

            Server::builder()
                .add_service(logs_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    startup_ack_sender.send(true).await.unwrap();
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("uh oh server failed");
        }

        let server_handle = tokio_rt.spawn(async move {
            // start the server when the signal is received
            let listening_addr = format!("{grpc_addr}:{grpc_port}");
            _ = server_startup_receiver.recv().await.unwrap();
            run_server(
                listening_addr.clone(),
                server_start_ack_sender.clone(),
                shutdown_signal1,
                req_sender.clone(),
            )
            .await;

            // ack server shutdown for first time
            server_shutdown_ack_sender.send(true).await.unwrap();

            // when the server shuts down, wait until it should restart & restart it
            _ = server_startup_receiver.recv().await.unwrap();
            run_server(
                listening_addr.clone(),
                server_start_ack_sender.clone(),
                shutdown_signal2,
                req_sender.clone(),
            )
            .await;
        });

        let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

        let (exporter_result, test_drive_result) = tokio_rt.block_on(async move {
            tokio::join!(
                start_exporter(exporter, pipeline_ctrl_msg_tx, metrics_reporter.clone()),
                drive_test(
                    server_startup_sender,
                    server_start_ack_receiver,
                    server_shutdown_ack_receiver,
                    shutdown_sender1,
                    shutdown_sender2,
                    pdata_tx,
                    control_sender,
                    pipeline_ctrl_msg_rx,
                    req_receiver,
                    metrics_rx,
                    metrics_reporter
                )
            )
        });

        // assert no error
        exporter_result.unwrap();
        test_drive_result.unwrap();

        tokio_rt
            .block_on(server_handle)
            .expect("server shutdown success");
    }
}
