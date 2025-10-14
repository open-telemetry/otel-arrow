// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use crate::OTAP_EXPORTER_FACTORIES;
use crate::metrics::ExporterPDataMetrics;
use crate::pdata::OtapPdata;
use async_stream::stream;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::experimental::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::{Error, ExporterErrorKind, format_error_sources};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_telemetry::metrics::MetricSet;
use otel_arrow_rust::Producer;
use otel_arrow_rust::encode::producer::ProducerOptions;
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{BatchArrowRecords, BatchStatus};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
    arrow_logs_service_client::ArrowLogsServiceClient,
    arrow_metrics_service_client::ArrowMetricsServiceClient,
    arrow_traces_service_client::ArrowTracesServiceClient,
};
use serde_json::Value;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc::{Receiver, Sender};
use tonic::transport::Channel;
use tonic::{IntoStreamingRequest, Response, Status, Streaming};

/// The URN for the OTAP exporter
pub const OTAP_EXPORTER_URN: &str = "urn:otel:otap:exporter";

pub mod config;
use config::Config;

/// Exporter that sends OTAP data via gRPC
pub struct OTAPExporter {
    config: Config,
    pdata_metrics: MetricSet<ExporterPDataMetrics>,
}

/// Declares the OTAP exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static OTAP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTAP_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            OTAPExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

impl OTAPExporter {
    /// Creates a new OTAPExporter
    #[must_use]
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Self {
        let batch_metrics = pipeline_ctx.register_metrics::<ExporterPDataMetrics>();
        OTAPExporter {
            config,
            pdata_metrics: batch_metrics,
        }
    }

    /// Creates a new OTAPExporter from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(OTAPExporter::new(pipeline_ctx, config))
    }
}

/// Implement the local exporter trait for a OTAP Exporter
#[async_trait(?Send)]
impl local::Exporter<OtapPdata> for OTAPExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        effect_handler
            .info(&format!(
                "Exporting OTLP traffic to endpoint: {}",
                self.config.grpc_endpoint
            ))
            .await;

        let exporter_id = effect_handler.exporter_id();
        let channel = Channel::from_shared(self.config.grpc_endpoint.clone())
            .map_err(|e| {
                let source_detail = format_error_sources(&e);
                Error::ExporterError {
                    exporter: exporter_id,
                    kind: ExporterErrorKind::Connect,
                    error: format!("grpc channel error {e}"),
                    source_detail,
                }
            })?
            .connect_lazy();

        let timer_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        // start a grpc client and connect to the server
        let mut arrow_metrics_client = ArrowMetricsServiceClient::new(channel.clone());
        let mut arrow_logs_client = ArrowLogsServiceClient::new(channel.clone());
        let mut arrow_traces_client = ArrowTracesServiceClient::new(channel.clone());

        if let Some(ref compression) = self.config.compression_method {
            let encoding = compression.map_to_compression_encoding();
            arrow_logs_client = arrow_logs_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
            arrow_metrics_client = arrow_metrics_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
            arrow_traces_client = arrow_traces_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
        }

        // TODO comment on the purpose of these
        // TODO import so can use as just "channel" here
        // TODO check if we can use our local channel since we are already using `tokio::task::spawn_local`.
        let (logs_sender, logs_receiver) = tokio::sync::mpsc::channel(64);
        let (metrics_sender, metrics_receiver) = tokio::sync::mpsc::channel(64);
        let (traces_sender, traces_receiver) = tokio::sync::mpsc::channel(64);
        let (pdata_metrics_tx, mut pdata_metrics_rx) = tokio::sync::mpsc::channel(64);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let ipc_compression = matches!(
            self.config.arrow.payload_compression,
            Some(config::ArrowPayloadCompression::Zstd)
        )
        .then(|| arrow_ipc::CompressionType::ZSTD);

        // TODO check if we can expose/use spawn_local method in the effect handler
        let logs_handle = tokio::task::spawn_local(stream_arrow_batches(
            arrow_logs_client,
            SignalType::Logs,
            ipc_compression,
            logs_receiver,
            pdata_metrics_tx.clone(),
            shutdown_rx.clone(),
        ));
        let metrics_handle = tokio::task::spawn_local(stream_arrow_batches(
            arrow_metrics_client,
            SignalType::Metrics,
            ipc_compression,
            metrics_receiver,
            pdata_metrics_tx.clone(),
            shutdown_rx.clone(),
        ));
        let traces_handle = tokio::task::spawn_local(stream_arrow_batches(
            arrow_traces_client,
            SignalType::Traces,
            ipc_compression,
            traces_receiver,
            pdata_metrics_tx.clone(),
            shutdown_rx.clone(),
        ));

        // Loop until a Shutdown event is received.
        loop {
            tokio::select! {
                msg = msg_chan.recv() => match msg? {
                    // handle control messages
                    Message::Control(NodeControlMsg::TimerTick { .. })
                    | Message::Control(NodeControlMsg::Config { .. }) => {}
                    Message::Control(NodeControlMsg::CollectTelemetry {
                        mut metrics_reporter,
                    }) => {
                        _ = metrics_reporter.report(&mut self.pdata_metrics);
                    }
                    // shutdown the exporter
                    Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                        _ = shutdown_tx.send_replace(true);
                        _ = logs_handle.await;
                        _ = metrics_handle.await;
                        _ = traces_handle.await;
                        _ = timer_cancel_handle.cancel().await;
                        return Ok(TerminalState::new(deadline, [self.pdata_metrics]))
                    }
                    //send data
                    Message::PData(pdata) => {
                        // Capture signal type before moving pdata into try_from
                        let signal_type = pdata.signal_type();

                        self.pdata_metrics.inc_consumed(signal_type);
                        let (_context, payload) = pdata.into_parts();

                        // TODO(#1098): Note context is dropped.
                        let message: OtapArrowRecords = payload
                            .try_into()
                            .inspect_err(|_| self.pdata_metrics.inc_failed(signal_type))?;

                        _ = match signal_type {
                            SignalType::Logs => logs_sender.send(message).await,
                            SignalType::Metrics => metrics_sender.send(message).await,
                            SignalType::Traces => traces_sender.send(message).await,
                        };
                    }
                    _ => {
                        return Err(Error::ExporterError {
                            exporter: effect_handler.exporter_id(),
                            kind: ExporterErrorKind::Other,
                            error: "Unknown control message".to_owned(),
                            source_detail: "".to_owned()
                        });
                    }
                },
                metrics_update = pdata_metrics_rx.recv() => match metrics_update {
                    Some(PDataMetricsUpdate::IncFailed(signal_type)) => {
                        self.pdata_metrics.inc_failed(signal_type);
                    },
                    Some(PDataMetricsUpdate::IncExported(signal_type)) => {
                        self.pdata_metrics.inc_exported(signal_type);
                    },
                    _ => {}
                }
            }
        }
    }
}

#[async_trait]
trait StreamingArrowService {
    async fn handle_req_stream(
        &mut self,
        req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
    ) -> Result<Response<Streaming<BatchStatus>>, Status>;
}

#[async_trait]
impl StreamingArrowService for ArrowLogsServiceClient<Channel> {
    async fn handle_req_stream(
        &mut self,
        req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
    ) -> Result<Response<Streaming<BatchStatus>>, Status> {
        self.arrow_logs(req_stream).await
    }
}

#[async_trait]
impl StreamingArrowService for ArrowMetricsServiceClient<Channel> {
    async fn handle_req_stream(
        &mut self,
        req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
    ) -> Result<Response<Streaming<BatchStatus>>, Status> {
        self.arrow_metrics(req_stream).await
    }
}

#[async_trait]
impl StreamingArrowService for ArrowTracesServiceClient<Channel> {
    async fn handle_req_stream(
        &mut self,
        req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
    ) -> Result<Response<Streaming<BatchStatus>>, Status> {
        self.arrow_traces(req_stream).await
    }
}

enum PDataMetricsUpdate {
    IncExported(SignalType),
    IncFailed(SignalType),
}

async fn stream_arrow_batches<T: StreamingArrowService>(
    mut client: T,
    signal_type: SignalType,
    ipc_compression: Option<arrow_ipc::CompressionType>,
    otap_batches_rx: Receiver<OtapArrowRecords>,
    pdata_metrics_tx: Sender<PDataMetricsUpdate>,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) {
    let otap_batches_rx = Arc::new(tokio::sync::Mutex::new(otap_batches_rx));
    let mut shutdown = false;

    // we'll do an exponential backoff if there was an error creating the streaming request
    const MAX_BACKOFF: Duration = Duration::from_secs(10);
    const INITIAL_BACKOFF: Duration = Duration::from_millis(10);
    const BACKOFF_MULTIPLIER: u32 = 2;
    let mut failed_request_backoff = INITIAL_BACKOFF;

    // send streams of batches to the server until shutdown
    while !shutdown {
        let mut rx = otap_batches_rx.lock().await;
        tokio::select! {
            // wait to receive the first batch to create the streaming request
            first_batch = rx.recv() => {
                drop(rx);
                let first_batch = match first_batch {
                    Some(f) => f,

                    None => {
                        // no more batches
                        break
                    }
                };

                // create the request stream
                let req_stream = create_req_stream(
                    first_batch,
                    otap_batches_rx.clone(),
                    signal_type,
                    ipc_compression,
                    pdata_metrics_tx.clone()
                );
                match client.handle_req_stream(req_stream).await {
                    Ok(res) => {
                        // reset the reconnect timeout backoff
                        failed_request_backoff = INITIAL_BACKOFF;

                        // handle server responses until error or shutdown
                        shutdown = handle_res_stream(
                            res.into_inner(),
                            pdata_metrics_tx.clone(),
                            signal_type,
                            shutdown_rx.clone()
                        ).await;
                    }
                    Err(_e) => {
                        // there was an error initiating the streaming request
                        _ = pdata_metrics_tx.send(PDataMetricsUpdate::IncFailed(signal_type)).await;
                        log::error!("failed request, waiting {failed_request_backoff:?}");
                        tokio::time::sleep(failed_request_backoff).await;
                        failed_request_backoff = std::cmp::min(failed_request_backoff * BACKOFF_MULTIPLIER, MAX_BACKOFF);
                    }
                };
            }
            _ = shutdown_rx.changed() => {
                 shutdown = *shutdown_rx.borrow();
            }
        }
    }
}

#[allow(tail_expr_drop_order)]
fn create_req_stream(
    mut first_batch: OtapArrowRecords,
    remaining_batches_rx: Arc<tokio::sync::Mutex<Receiver<OtapArrowRecords>>>,
    signal_type: SignalType,
    ipc_compression: Option<arrow_ipc::CompressionType>,
    pdata_metrics_tx: Sender<PDataMetricsUpdate>,
) -> impl IntoStreamingRequest<Message = BatchArrowRecords> {
    stream! {
        let mut producer = Producer::new_with_options(ProducerOptions {
            ipc_compression
        });

        // send the first batch
        match producer.produce_bar(&mut first_batch) {
            Ok(bar) => yield bar,
            Err(_) => {
                _ = pdata_metrics_tx.send(PDataMetricsUpdate::IncFailed(signal_type));
            }
        };

        let mut rx = remaining_batches_rx.lock().await;
        // send the remaining batches
        while let Some(mut otap_batch) = rx.recv().await {
            match producer.produce_bar(&mut otap_batch) {
                Ok(bar) => yield bar,
                Err(_) => {
                    _ = pdata_metrics_tx.send(PDataMetricsUpdate::IncFailed(signal_type));
                }
            }
        }
    }
}

async fn handle_res_stream(
    mut res_stream: Streaming<BatchStatus>,
    pdata_metrics_tx: Sender<PDataMetricsUpdate>,
    signal_type: SignalType,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
) -> bool {
    let mut shutdown = false;

    // handle streaming responses until shutdown
    while !shutdown {
        tokio::select! {
            res = res_stream.message() => {
                match res {
                    Ok(Some(_val)) => {
                        _ = pdata_metrics_tx.send(PDataMetricsUpdate::IncExported(signal_type)).await;
                    },
                    Ok(None) => {
                        // sender disconnected
                        break
                    }
                    Err(_grpc_status) => {
                        _ = pdata_metrics_tx.send(PDataMetricsUpdate::IncFailed(signal_type)).await;
                        break
                    }
                };
            }
            _ = shutdown_rx.changed() => {
                shutdown = *shutdown_rx.borrow();
            }
        }
    }

    shutdown
}

#[cfg(test)]
mod tests {
    use crate::otap_exporter::OTAP_EXPORTER_URN;
    use crate::otap_exporter::OTAPExporter;
    use crate::otap_exporter::config::ArrowPayloadCompression;
    use crate::otap_mock::{
        ArrowLogsServiceMock, ArrowMetricsServiceMock, ArrowTracesServiceMock, create_otap_batch,
    };
    use crate::pdata::OtapPdata;

    use crate::compression::CompressionMethod;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::Controllable;
    use otap_df_engine::control::NodeControlMsg;
    use otap_df_engine::control::PipelineCtrlMsgSender;
    use otap_df_engine::control::pipeline_ctrl_msg_channel;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::local::message::LocalReceiver;
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::message::Receiver;
    use otap_df_engine::message::Sender;
    use otap_df_engine::node::NodeWithPDataReceiver;
    use otap_df_engine::testing::create_not_send_channel;
    use otap_df_engine::testing::{
        exporter::{TestContext, TestRuntime},
        test_node,
    };
    use otap_df_telemetry::metrics::MetricSetSnapshot;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use otel_arrow_rust::otap::OtapArrowRecords;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, arrow_logs_service_server::ArrowLogsServiceServer,
        arrow_metrics_service_server::ArrowMetricsServiceServer,
        arrow_traces_service_server::ArrowTracesServiceServer,
    };
    use serde_json::json;
    use std::net::SocketAddr;
    use std::ops::Add;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};
    use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;

    const METRIC_BATCH_ID: i64 = 0;
    const LOG_BATCH_ID: i64 = 1;
    const TRACE_BATCH_ID: i64 = 2;

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario()
    -> impl FnOnce(TestContext<OtapPdata>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // Send a data message
                let metric_message =
                    create_otap_batch(METRIC_BATCH_ID, ArrowPayloadType::MultivariateMetrics);
                ctx.send_pdata(OtapPdata::new_default(metric_message.into()))
                    .await
                    .expect("Failed to send metric message");

                let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                ctx.send_pdata(OtapPdata::new_default(log_message.into()))
                    .await
                    .expect("Failed to send log message");

                let trace_message = create_otap_batch(TRACE_BATCH_ID, ArrowPayloadType::Spans);
                ctx.send_pdata(OtapPdata::new_default(trace_message.into()))
                    .await
                    .expect("Failed to send trace message");

                tokio::time::sleep(Duration::from_millis(500)).await;

                // Send shutdown
                ctx.send_shutdown(
                    Instant::now().add(Duration::from_millis(200)),
                    "test complete",
                )
                .await
                .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(
        mut receiver: tokio::sync::mpsc::Receiver<OtapPdata>,
    ) -> impl FnOnce(
        TestContext<OtapPdata>,
        Result<(), Error>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                assert!(exporter_result.is_ok());

                // check that the message was properly sent from the exporter
                let metrics_received: OtapArrowRecords =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .payload()
                        .try_into()
                        .expect("Could convert pdata to OTAPData");

                // Assert that the message received is what the exporter sent
                let _expected_metrics_message =
                    create_otap_batch(METRIC_BATCH_ID, ArrowPayloadType::MultivariateMetrics);
                assert!(matches!(metrics_received, _expected_metrics_message));

                let logs_received: OtapArrowRecords =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .payload()
                        .try_into()
                        .expect("Could convert pdata to OTAPData");
                let _expected_logs_message =
                    create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                assert!(matches!(logs_received, _expected_logs_message));

                let traces_received: OtapArrowRecords =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .payload()
                        .try_into()
                        .expect("Could convert pdata to OTAPData");

                let _expected_trace_message =
                    create_otap_batch(TRACE_BATCH_ID, ArrowPayloadType::Spans);
                assert!(matches!(traces_received, _expected_trace_message));
            })
        }
    }

    #[test]
    fn test_otap_exporter() {
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
            let mock_logs_service =
                ArrowLogsServiceServer::new(ArrowLogsServiceMock::new(sender.clone()));
            let mock_metrics_service =
                ArrowMetricsServiceServer::new(ArrowMetricsServiceMock::new(sender.clone()));
            let mock_trace_service =
                ArrowTracesServiceServer::new(ArrowTracesServiceMock::new(sender.clone()));
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

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let config = json!({
            "grpc_endpoint": grpc_endpoint,
            "compression_method": "none"
        });
        // Create a proper pipeline context for the benchmark
        let controller_ctx = ControllerContext::new(test_runtime.metrics_registry());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let exporter = ExporterWrapper::local(
            OTAPExporter::from_config(pipeline_ctx, &config).expect("Config should be valid"),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(receiver));

        _ = shutdown_sender.send("Shutdown");
    }

    #[test]
    fn test_from_config_success() {
        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "compression_method": "gzip"
        });

        // Create a proper pipeline context for the test
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");

        assert_eq!(exporter.config.grpc_endpoint, "http://localhost:4317");
        match exporter.config.compression_method {
            Some(ref method) => match method {
                CompressionMethod::Gzip => {} // success
                other => panic!("Expected Gzip, got {other:?}"),
            },
            None => panic!("Expected Some compression method"),
        }
    }

    #[test]
    fn test_from_config_missing_required_field() {
        let json_config = json!({
            "compression_method": "gzip"
        });

        // Create a proper pipeline context for the test
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let result = OTAPExporter::from_config(pipeline_ctx, &json_config);

        assert!(result.is_err());
        if let Err(err) = result {
            let err_msg = format!("{err}");
            assert!(err_msg.contains("missing field `grpc_endpoint`"));
        }
    }

    #[test]
    fn test_double_compression_enabled_by_default() {
        let json_config = json!({
            "grpc_endpoint": "localhost:4317"
        });
        // Create a proper pipeline context for the test
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");

        assert!(
            matches!(
                exporter.config.compression_method,
                Some(CompressionMethod::Zstd)
            ),
            "expected Some(Zstd) received {:?}",
            exporter.config.compression_method
        );
        assert!(
            matches!(
                exporter.config.arrow.payload_compression,
                Some(ArrowPayloadCompression::Zstd)
            ),
            "expected Some(Zstd) received {:?}",
            exporter.config.arrow.payload_compression
        );
    }

    #[test]
    fn test_can_manually_disable_compression_via_config() {
        let json_config = json!({
            "grpc_endpoint": "localhost:4317",
            "compression_method": "none",
            "arrow": {
                "payload_compression": "none"
            }
        });
        // Create a proper pipeline context for the test
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");
        assert!(
            exporter.config.compression_method.is_none(),
            "expected None received {:?}",
            exporter.config.compression_method
        );
        assert!(
            exporter.config.arrow.payload_compression.is_none(),
            "expected None received {:?}",
            exporter.config.arrow.payload_compression
        );
    }

    #[test]
    fn test_receiver_not_ready_on_start() {
        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let mut exporter = ExporterWrapper::local(
            OTAPExporter::from_config(
                pipeline_ctx,
                &serde_json::json!({
                    "grpc_endpoint": grpc_endpoint,
                    "compression_method": "none"
                }),
            )
            .unwrap(),
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::MpscSender(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::MpscReceiver(pdata_rx));
        let (pipeline_ctrl_msg_tx, _pipeline_ctrl_msg_rx) = pipeline_ctrl_msg_channel(2);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        let (req_sender, req_receiver) = tokio::sync::mpsc::channel(1);
        let (server_startup_sender, mut server_startup_receiver) = tokio::sync::mpsc::channel(1);
        let (server_start_ack_sender, server_start_ack_receiver) = tokio::sync::mpsc::channel(1);
        let (server_shutdown_sender, server_shutdown_signal) = tokio::sync::oneshot::channel();

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            pipeline_ctrl_msg_tx: PipelineCtrlMsgSender<OtapPdata>,
            metrics_reporter: MetricsReporter,
        ) -> Result<(), Error> {
            _ = exporter.start(pipeline_ctrl_msg_tx, metrics_reporter).await;
            Ok(())
        }

        async fn drive_test(
            server_startup_sender: tokio::sync::mpsc::Sender<bool>,
            mut server_startup_ack_receiver: tokio::sync::mpsc::Receiver<bool>,
            server_shutdown_sender1: tokio::sync::oneshot::Sender<bool>,
            pdata_tx: Sender<OtapPdata>,
            control_sender: Sender<NodeControlMsg<OtapPdata>>,
            mut req_receiver: tokio::sync::mpsc::Receiver<OtapPdata>,
            metrics_receiver: flume::Receiver<MetricSetSnapshot>,
            metrics_reporter: MetricsReporter,
        ) {
            // send a request before while the server isn't running and check how we handle it
            let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
            pdata_tx
                .send(OtapPdata::new_default(log_message.into()))
                .await
                .expect("Failed to send log message");
            // TODO instead of sleeping here, once we handle ACK/NACK we should wait to get a NACK
            // from the control channel
            tokio::time::sleep(Duration::from_millis(5)).await;

            // wait a bit before starting the server. This will ensure the exporter no-long exits
            // when start is called if the endpoint can't be reached
            tokio::time::sleep(Duration::from_millis(100)).await;
            server_startup_sender.send(true).await.unwrap();
            _ = server_startup_ack_receiver.recv().await.unwrap();

            // send another pdata now that the server has started
            let log_message = create_otap_batch(LOG_BATCH_ID + 1, ArrowPayloadType::Logs);
            pdata_tx
                .send(OtapPdata::new_default(log_message.into()))
                .await
                .expect("Failed to send log message");
            _ = req_receiver.recv().await.unwrap(); // ensure we got response
            // TODO instead of sleeping here, once we handle ACK/NACK we should wait to get a ACK
            // from the control channel
            tokio::time::sleep(Duration::from_millis(50)).await;

            // check the metrics:
            control_sender
                .send(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: metrics_reporter.clone(),
                })
                .await
                .unwrap();
            let metrics = metrics_receiver.recv_async().await.unwrap();
            let logs_exported_count = metrics.get_metrics()[4]; // logs exported
            assert_eq!(logs_exported_count, 1);
            let logs_failed_count = metrics.get_metrics()[5]; // logs failed
            assert_eq!(logs_failed_count, 1);

            control_sender
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now().add(Duration::from_millis(10)),
                    reason: "shutting down".into(),
                })
                .await
                .unwrap();

            server_shutdown_sender1.send(true).unwrap();
        }

        async fn run_server(
            listening_addr: String,
            startup_ack_sender: tokio::sync::mpsc::Sender<bool>,
            shutdown_signal: tokio::sync::oneshot::Receiver<bool>,
            req_sender: tokio::sync::mpsc::Sender<OtapPdata>,
        ) {
            let listening_addr: SocketAddr = listening_addr.to_string().parse().unwrap();
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let tcp_stream = TcpListenerStream::new(tcp_listener);

            let logs_service = ArrowLogsServiceServer::new(ArrowLogsServiceMock::new(req_sender));

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
            let listening_addr = format!("{grpc_addr}:{grpc_port}");

            // wait for signal to start the server
            _ = server_startup_receiver.recv().await.unwrap();
            run_server(
                listening_addr.clone(),
                server_start_ack_sender.clone(),
                server_shutdown_signal,
                req_sender.clone(),
            )
            .await;
        });
        let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

        let _ = tokio_rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            let metrics_reporter_start_exporter = metrics_reporter.clone();
            let _fut = local_set.spawn_local(async move {
                start_exporter(
                    exporter,
                    pipeline_ctrl_msg_tx,
                    metrics_reporter_start_exporter,
                )
                .await
            });
            tokio::join!(
                local_set,
                drive_test(
                    server_startup_sender,
                    server_start_ack_receiver,
                    server_shutdown_sender,
                    pdata_tx,
                    control_sender,
                    req_receiver,
                    metrics_rx,
                    metrics_reporter
                )
            )
        });

        tokio_rt
            .block_on(server_handle)
            .expect("server shutdown success");
    }
}
