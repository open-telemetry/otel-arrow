// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::OTAP_EXPORTER_FACTORIES;
use crate::compression::CompressionMethod;
use crate::metrics::ExporterPDataMetrics;
use crate::otap_grpc::otlp::client::{LogsServiceClient, MetricsServiceClient, TraceServiceClient};
use crate::pdata::{Context, OtapPayload, OtapPayloadHelpers, OtapPdata, OtlpProtoBytes};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::experimental::SignalType;
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
use otap_df_telemetry::metrics::MetricSet;
use otel_arrow_rust::otlp::logs::LogsProtoBytesEncoder;
use otel_arrow_rust::otlp::metrics::MetricsProtoBytesEncoder;
use otel_arrow_rust::otlp::traces::TracesProtoBytesEncoder;
use otel_arrow_rust::otlp::{ProtoBuffer, ProtoBytesEncoder};
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;
use tonic::transport::Channel;

/// The URN for the OTLP exporter
pub const OTLP_EXPORTER_URN: &str = "urn:otel:otlp:exporter";

/// Configuration for the OTLP Exporter
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The gRPC endpoint to connect to
    pub grpc_endpoint: String,
    /// The compression method to use for the gRPC connection
    pub compression_method: Option<CompressionMethod>,
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
        effect_handler
            .info(&format!(
                "Exporting OTLP traffic to endpoint: {}",
                self.config.grpc_endpoint
            ))
            .await;

        let exporter_id = effect_handler.exporter_id();
        let timer_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        let channel = Channel::from_shared(self.config.grpc_endpoint.clone())
            .map_err(|e| {
                let source_detail = format_error_sources(&e);
                Error::ExporterError {
                    exporter: exporter_id.clone(),
                    kind: ExporterErrorKind::Connect,
                    error: format!("grpc channel error {e}"),
                    source_detail,
                }
            })?
            .connect_lazy();

        // start a grpc client and connect to the server
        let mut metrics_client = MetricsServiceClient::new(channel.clone());
        let mut logs_client = LogsServiceClient::new(channel.clone());
        let mut trace_client = TraceServiceClient::new(channel.clone());

        if let Some(ref compression) = self.config.compression_method {
            let encoding = compression.map_to_compression_encoding();

            logs_client = logs_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
            metrics_client = metrics_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
            trace_client = trace_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
        }

        // reuse the encoder and the buffer across pdatas
        let mut logs_encoder = LogsProtoBytesEncoder::new();
        let mut metrics_encoder = MetricsProtoBytesEncoder::new();
        let mut traces_encoder = TracesProtoBytesEncoder::new();
        let mut proto_buffer = ProtoBuffer::new();

        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    _ = timer_cancel_handle.cancel().await;
                    return Ok(TerminalState::new(deadline, [self.pdata_metrics]));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.pdata_metrics);
                }
                Message::PData(pdata) => {
                    // Capture signal type before moving pdata into try_from
                    let signal_type = pdata.signal_type();

                    // Keep context for Ack/Nack delivery
                    let (context, payload) = pdata.into_parts();
                    self.pdata_metrics.inc_consumed(signal_type);

                    match (signal_type, payload) {
                        // use optimized direct encoding OTAP -> OTLP bytes directly
                        (SignalType::Logs, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            match handle_otap_export(
                                otap_batch,
                                context,
                                &mut proto_buffer,
                                &mut logs_encoder,
                                &mut logs_client,
                                &effect_handler,
                            )
                            .await
                            {
                                Ok(()) => self.pdata_metrics.logs_exported.inc(),
                                _ => self.pdata_metrics.logs_failed.inc(),
                            }
                        }
                        (SignalType::Metrics, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            match handle_otap_export(
                                otap_batch,
                                context,
                                &mut proto_buffer,
                                &mut metrics_encoder,
                                &mut metrics_client,
                                &effect_handler,
                            )
                            .await
                            {
                                Ok(()) => self.pdata_metrics.metrics_exported.inc(),
                                _ => self.pdata_metrics.metrics_failed.inc(),
                            }
                        }
                        (SignalType::Traces, OtapPayload::OtapArrowRecords(otap_batch)) => {
                            match handle_otap_export(
                                otap_batch,
                                context,
                                &mut proto_buffer,
                                &mut traces_encoder,
                                &mut trace_client,
                                &effect_handler,
                            )
                            .await
                            {
                                Ok(()) => self.pdata_metrics.traces_exported.inc(),
                                _ => self.pdata_metrics.traces_failed.inc(),
                            }
                        }
                        (_, OtapPayload::OtlpBytes(service_req)) => {
                            _ = match service_req {
                                OtlpProtoBytes::ExportLogsRequest(bytes) => {
                                    match handle_otlp_export(
                                        bytes,
                                        context,
                                        &mut logs_client,
                                        &effect_handler,
                                        |b| OtlpProtoBytes::ExportLogsRequest(b.to_vec()).into(),
                                    )
                                    .await
                                    {
                                        Ok(()) => self.pdata_metrics.logs_exported.inc(),
                                        _ => self.pdata_metrics.logs_failed.inc(),
                                    }
                                }
                                OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                                    match handle_otlp_export(
                                        bytes,
                                        context,
                                        &mut metrics_client,
                                        &effect_handler,
                                        |b| OtlpProtoBytes::ExportMetricsRequest(b.to_vec()).into(),
                                    )
                                    .await
                                    {
                                        Ok(()) => self.pdata_metrics.metrics_exported.inc(),
                                        _ => self.pdata_metrics.metrics_failed.inc(),
                                    }
                                }
                                OtlpProtoBytes::ExportTracesRequest(bytes) => {
                                    match handle_otlp_export(
                                        bytes,
                                        context,
                                        &mut trace_client,
                                        &effect_handler,
                                        |b| OtlpProtoBytes::ExportTracesRequest(b.to_vec()).into(),
                                    )
                                    .await
                                    {
                                        Ok(()) => self.pdata_metrics.traces_exported.inc(),
                                        _ => self.pdata_metrics.traces_failed.inc(),
                                    }
                                }
                            };
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
async fn handle_export_result<T>(
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

/// Generic function for encoding OTAP records to protobuf, exporting via gRPC,
/// and handling Ack/Nack delivery.
async fn handle_otap_export<Enc: ProtoBytesEncoder, T2, Resp, S>(
    mut otap_batch: otel_arrow_rust::otap::OtapArrowRecords,
    context: Context,
    proto_buffer: &mut ProtoBuffer,
    encoder: &mut Enc,
    client: &mut crate::otap_grpc::otlp::client::OtlpServiceClient<T2, Resp, S>,
    effect_handler: &EffectHandler<OtapPdata>,
) -> Result<(), Error>
where
    T2: tonic::client::GrpcService<tonic::body::Body>,
    T2::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    T2::ResponseBody: Send + 'static,
    <T2::ResponseBody as tonic::transport::Body>::Error:
        Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    S: crate::otap_grpc::otlp::client::ServiceDescriptor,
    Resp: prost::Message + Default + Send + 'static,
{
    // Encode OTAP records to protobuf
    proto_buffer.clear();
    encoder
        .encode(&mut otap_batch, proto_buffer)
        .map_err(|e| Error::ExporterError {
            exporter: effect_handler.exporter_id(),
            kind: ExporterErrorKind::Other,
            error: format!("encoding error: {}", e),
            source_detail: "".to_string(),
        })?;

    let bytes = proto_buffer.as_ref().to_vec();
    if !context.may_return_payload() {
        // drop before the export, payload not requested
        let _drop = otap_batch.take_payload();
    }
    let saved_payload: OtapPayload = otap_batch.into();

    // Export and handle result with Ack/Nack
    let result = client.export(bytes).await;
    handle_export_result(result, context, saved_payload, effect_handler).await
}

/// Generic function for exporting OTLP bytes via gRPC and handling Ack/Nack delivery.
async fn handle_otlp_export<T2, Resp, S>(
    bytes: Vec<u8>,
    context: Context,
    client: &mut crate::otap_grpc::otlp::client::OtlpServiceClient<T2, Resp, S>,
    effect_handler: &EffectHandler<OtapPdata>,
    save_payload_fn: impl FnOnce(&[u8]) -> OtapPayload,
) -> Result<(), Error>
where
    T2: tonic::client::GrpcService<tonic::body::Body>,
    T2::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
    T2::ResponseBody: Send + 'static,
    <T2::ResponseBody as tonic::transport::Body>::Error:
        Into<Box<dyn std::error::Error + Send + Sync>> + Send,
    S: crate::otap_grpc::otlp::client::ServiceDescriptor,
    Resp: prost::Message + Default + Send + 'static,
{
    let saved_payload = if context.may_return_payload() {
        save_payload_fn(&bytes)
    } else {
        save_payload_fn(&[])
    };

    let result = client.export(bytes).await;
    handle_export_result(result, context, saved_payload, effect_handler).await
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::otlp_grpc::OTLPData;
    use crate::otlp_mock::{LogsServiceMock, MetricsServiceMock, TraceServiceMock};
    use crate::proto::opentelemetry::collector::{
        logs::v1::{ExportLogsServiceRequest, logs_service_server::LogsServiceServer},
        metrics::v1::{ExportMetricsServiceRequest, metrics_service_server::MetricsServiceServer},
        trace::v1::{ExportTraceServiceRequest, trace_service_server::TraceServiceServer},
    };
    use crate::testing::TestCallData;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::Interests;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::{Controllable, PipelineCtrlMsgSender, pipeline_ctrl_msg_channel};
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::local::message::{LocalReceiver, LocalSender};
    use otap_df_engine::message::{Receiver, Sender};
    use otap_df_engine::node::NodeWithPDataReceiver;
    use otap_df_engine::testing::create_not_send_channel;
    use otap_df_engine::testing::{
        exporter::{TestContext, TestRuntime},
        test_node,
    };
    use otap_df_telemetry::metrics::MetricSetSnapshot;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use prost::Message;
    use std::net::SocketAddr;
    use std::time::Instant;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};
    use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;

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
    fn scenario()
    -> impl FnOnce(TestContext<OtapPdata>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // Send a data message
                let req = ExportLogsServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                let logs_pdata =
                    OtapPdata::new_default(OtlpProtoBytes::ExportLogsRequest(req_bytes).into())
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
                let metrics_pdata =
                    OtapPdata::new_default(OtlpProtoBytes::ExportMetricsRequest(req_bytes).into())
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
                let traces_pdata =
                    OtapPdata::new_default(OtlpProtoBytes::ExportTracesRequest(req_bytes).into())
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
    ) -> impl FnOnce(
        TestContext<OtapPdata>,
        Result<(), Error>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                assert!(exporter_result.is_ok());

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
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc_endpoint,
                    compression_method: None,
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

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle.clone());
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let mut exporter = ExporterWrapper::local(
            OTLPExporter {
                config: Config {
                    grpc_endpoint,
                    compression_method: None,
                },
                pdata_metrics: pipeline_ctx.register_metrics::<ExporterPDataMetrics>(),
            },
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::MpscSender(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::MpscReceiver(pdata_rx));
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
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone()),
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
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone()),
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
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone()),
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
                OtlpProtoBytes::ExportLogsRequest(req_bytes.clone()),
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
            let logs_exported_count = metrics.get_metrics()[4]; // logs exported
            assert_eq!(logs_exported_count, 2);
            let logs_failed_count = metrics.get_metrics()[5]; // logs failed
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
