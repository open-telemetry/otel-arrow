// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! STEF metrics exporter compatible with the Collector contrib STEF destination service.

use crate::stef_grpc::{
    StefClientFirstMessage, StefClientMessage, StefDataResponse, StefDestinationClient,
    stef_server_message,
};
use async_trait::async_trait;
use bytes::Bytes;
use futures::stream;
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
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::metrics::ExporterPDataMetrics;
use otap_df_otap::otap_grpc::client_settings::GrpcClientSettings;
use otap_df_otap::pdata::{Context, OtapPdata};
use otap_df_pdata::otlp::metrics::MetricsProtoBytesEncoder;
use otap_df_pdata::otlp::{ProtoBuffer, ProtoBytesEncoder};
use otap_df_pdata::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;
use otap_df_pdata::proto::opentelemetry::metrics::v1::metric;
use otap_df_pdata::stef::{METRICS_ROOT_STRUCT_NAME, METRICS_WIRE_SCHEMA, encode_metrics_request};
use otap_df_pdata::{OtapArrowRecords, OtapPayload, OtapPayloadHelpers, OtlpProtoBytes};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_debug, otel_info, otel_warn};
use prost::Message as ProstMessage;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tonic::Request;
use tonic::codec::CompressionEncoding;
use tonic::transport::Channel;

/// The URN for the STEF exporter.
pub const STEF_EXPORTER_URN: &str = "urn:otel:exporter:stef";

/// Configuration for the STEF metrics exporter.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Shared gRPC client settings for the STEF destination.
    #[serde(flatten)]
    pub grpc: GrpcClientSettings,
}

/// Exporter that sends OTLP metrics as STEF over gRPC.
pub struct StefExporter {
    config: Config,
    pdata_metrics: MetricSet<ExporterPDataMetrics>,
}

/// Declares the STEF exporter factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static STEF_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: STEF_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            StefExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

impl StefExporter {
    /// Creates a STEF exporter from node configuration.
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
impl Exporter<OtapPdata> for StefExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!(
            "stef.exporter.grpc.start",
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

        let mut metrics_proto_encoder = MetricsProtoBytesEncoder::new();
        let mut metrics_proto_buffer = ProtoBuffer::with_capacity(8 * 1024);

        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    otel_info!("stef.exporter.grpc.shutdown");
                    _ = timer_cancel_handle.cancel().await;
                    return Ok(TerminalState::new(deadline, [self.pdata_metrics]));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.pdata_metrics);
                }
                Message::PData(pdata) => {
                    let signal_type = pdata.signal_type();
                    let (context, payload) = pdata.into_parts();
                    self.pdata_metrics.inc_consumed(signal_type);

                    let prepared = prepare_metrics_export(
                        payload,
                        context,
                        signal_type,
                        &mut metrics_proto_buffer,
                        &mut metrics_proto_encoder,
                    );

                    match prepared {
                        Ok(export) => {
                            let result =
                                export_stef_metrics(channel.clone(), compression, &export).await;
                            route_export_result(
                                result,
                                export.context,
                                export.saved_payload,
                                &effect_handler,
                                &mut self.pdata_metrics,
                            )
                            .await;
                        }
                        Err(failure) => {
                            self.pdata_metrics.add_failed(signal_type, 1);
                            notify_prepare_error(failure, &effect_handler).await?;
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

struct PreparedStefExport {
    bytes: Vec<u8>,
    record_count: u64,
    context: Context,
    saved_payload: OtapPayload,
}

struct PrepareFailure {
    reason: String,
    context: Context,
    saved_payload: OtapPayload,
}

fn prepare_metrics_export(
    payload: OtapPayload,
    context: Context,
    signal_type: SignalType,
    proto_buffer: &mut ProtoBuffer,
    encoder: &mut MetricsProtoBytesEncoder,
) -> Result<PreparedStefExport, Box<PrepareFailure>> {
    if signal_type != SignalType::Metrics {
        return Err(Box::new(PrepareFailure {
            reason: "STEF exporter currently supports metrics only".to_owned(),
            saved_payload: save_payload(&context, signal_type, payload),
            context,
        }));
    }

    let (request, saved_payload) = match payload {
        OtapPayload::OtlpBytes(OtlpProtoBytes::ExportMetricsRequest(bytes)) => {
            let request = ExportMetricsServiceRequest::decode(bytes.as_ref()).map_err(|e| {
                Box::new(PrepareFailure {
                    reason: format!("metrics protobuf decode error: {e}"),
                    saved_payload: if context.may_return_payload() {
                        OtlpProtoBytes::ExportMetricsRequest(bytes.clone()).into()
                    } else {
                        OtlpProtoBytes::ExportMetricsRequest(Bytes::new()).into()
                    },
                    context: context.clone(),
                })
            })?;
            let saved_payload = if context.may_return_payload() {
                OtlpProtoBytes::ExportMetricsRequest(bytes).into()
            } else {
                OtlpProtoBytes::ExportMetricsRequest(Bytes::new()).into()
            };
            (request, saved_payload)
        }
        OtapPayload::OtapArrowRecords(mut otap_batch) => {
            proto_buffer.clear();
            if let Err(e) = encoder.encode(&mut otap_batch, proto_buffer) {
                let saved_payload = save_otap_batch(&context, otap_batch);
                return Err(Box::new(PrepareFailure {
                    reason: format!("metrics OTAP encode error: {e}"),
                    context,
                    saved_payload,
                }));
            }
            let (bytes, next_capacity) = proto_buffer.take_into_bytes();
            proto_buffer.ensure_capacity(next_capacity);
            let request = match ExportMetricsServiceRequest::decode(bytes.as_ref()) {
                Ok(request) => request,
                Err(e) => {
                    let saved_payload = save_otap_batch(&context, otap_batch);
                    return Err(Box::new(PrepareFailure {
                        reason: format!("metrics protobuf decode error: {e}"),
                        context: context.clone(),
                        saved_payload,
                    }));
                }
            };
            let saved_payload = save_otap_batch(&context, otap_batch);
            (request, saved_payload)
        }
        other => {
            return Err(Box::new(PrepareFailure {
                reason: "STEF exporter received non-metrics OTLP payload".to_owned(),
                saved_payload: save_payload(&context, signal_type, other),
                context,
            }));
        }
    };

    let record_count = count_metric_points(&request);
    let bytes = match encode_metrics_request(&request) {
        Ok(bytes) => bytes,
        Err(e) => {
            return Err(Box::new(PrepareFailure {
                reason: format!("STEF metrics encode error: {e}"),
                context,
                saved_payload,
            }));
        }
    };

    Ok(PreparedStefExport {
        bytes,
        record_count,
        context,
        saved_payload,
    })
}

async fn export_stef_metrics(
    channel: Channel,
    compression: Option<CompressionEncoding>,
    export: &PreparedStefExport,
) -> Result<(), tonic::Status> {
    let mut client = StefDestinationClient::new(channel);
    if let Some(compression) = compression {
        client = client
            .send_compressed(compression)
            .accept_compressed(compression);
    }

    let (tx, rx) = mpsc::channel(2);
    tx.send(StefClientMessage {
        first_message: Some(StefClientFirstMessage {
            root_struct_name: METRICS_ROOT_STRUCT_NAME.to_owned(),
        }),
        stef_bytes: Vec::new(),
        is_end_of_chunk: false,
    })
    .await
    .map_err(|_| tonic::Status::internal("failed to queue STEF first message"))?;

    let outbound = stream::unfold(rx, |mut rx| async {
        rx.recv().await.map(|message| (message, rx))
    });

    let mut inbound = client.stream(Request::new(outbound)).await?.into_inner();
    let caps = inbound
        .message()
        .await?
        .and_then(|message| match message.message {
            Some(stef_server_message::Message::Capabilities(caps)) => Some(caps),
            _ => None,
        })
        .ok_or_else(|| tonic::Status::unavailable("missing STEF capabilities"))?;
    if caps.schema != METRICS_WIRE_SCHEMA {
        return Err(tonic::Status::failed_precondition(
            "STEF destination metrics schema is incompatible",
        ));
    }

    tx.send(StefClientMessage {
        first_message: None,
        stef_bytes: export.bytes.clone(),
        is_end_of_chunk: true,
    })
    .await
    .map_err(|_| tonic::Status::unavailable("failed to send STEF chunk"))?;
    drop(tx);

    if export.record_count == 0 {
        return Ok(());
    }

    while let Some(message) = inbound.message().await? {
        if let Some(stef_server_message::Message::Response(response)) = message.message {
            validate_response(&response)?;
            if response.ack_record_id >= export.record_count {
                return Ok(());
            }
        }
    }

    Err(tonic::Status::unavailable("STEF stream closed before ack"))
}

fn validate_response(response: &StefDataResponse) -> Result<(), tonic::Status> {
    if response.bad_data_record_id_ranges.is_empty() {
        Ok(())
    } else {
        Err(tonic::Status::data_loss(
            "STEF destination reported bad data",
        ))
    }
}

async fn route_export_result(
    result: Result<(), tonic::Status>,
    context: Context,
    saved_payload: OtapPayload,
    effect_handler: &EffectHandler<OtapPdata>,
    pdata_metrics: &mut MetricSet<ExporterPDataMetrics>,
) {
    match result {
        Ok(()) => {
            if effect_handler
                .notify_ack(AckMsg::new(OtapPdata::new(context, saved_payload)))
                .await
                .is_ok()
            {
                pdata_metrics.metrics_exported.add(1);
            }
        }
        Err(e) => {
            let error_msg = e.to_string();
            if effect_handler
                .notify_nack(NackMsg::new(
                    &error_msg,
                    OtapPdata::new(context, saved_payload),
                ))
                .await
                .is_ok()
            {
                pdata_metrics.metrics_failed.add(1);
            }
            otel_warn!(
                "stef.exporter.grpc.export_error",
                message = "STEF exporter request did not succeed",
                error = %e
            );
        }
    }
}

async fn notify_prepare_error(
    failure: Box<PrepareFailure>,
    effect_handler: &EffectHandler<OtapPdata>,
) -> Result<(), Error> {
    let PrepareFailure {
        reason,
        context,
        saved_payload,
    } = *failure;
    otel_debug!("stef.exporter.prepare_error", error = reason.as_str());
    effect_handler
        .notify_nack(NackMsg::new(reason, OtapPdata::new(context, saved_payload)))
        .await
}

fn save_payload(context: &Context, signal_type: SignalType, payload: OtapPayload) -> OtapPayload {
    if context.may_return_payload() {
        payload
    } else {
        OtapPayload::empty(signal_type)
    }
}

fn save_otap_batch(context: &Context, mut otap_batch: OtapArrowRecords) -> OtapPayload {
    if !context.may_return_payload() {
        let _drop = otap_batch.take_payload();
    }
    otap_batch.into()
}

fn count_metric_points(request: &ExportMetricsServiceRequest) -> u64 {
    request
        .resource_metrics
        .iter()
        .flat_map(|resource| &resource.scope_metrics)
        .flat_map(|scope| &scope.metrics)
        .map(|metric| match metric.data.as_ref() {
            Some(metric::Data::Gauge(gauge)) => gauge.data_points.len() as u64,
            Some(metric::Data::Sum(sum)) => sum.data_points.len() as u64,
            Some(metric::Data::Histogram(histogram)) => histogram.data_points.len() as u64,
            Some(metric::Data::ExponentialHistogram(histogram)) => {
                histogram.data_points.len() as u64
            }
            Some(metric::Data::Summary(summary)) => summary.data_points.len() as u64,
            None => 0,
        })
        .sum()
}
