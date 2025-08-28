// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::OTAP_EXPORTER_FACTORIES;
use crate::grpc::otlp::client::{LogsServiceClient, MetricsServiceClient, TraceServiceClient};
use crate::metrics::ExporterPDataMetrics;
use crate::pdata::{OtapPdata, OtlpProtoBytes};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_otlp::compression::CompressionMethod;
use otap_df_telemetry::metrics::MetricSet;
use serde::Deserialize;
use std::sync::Arc;
use std::time::Duration;

/// The URN for the OTLP exporter
pub const OTLP_EXPORTER_URN: &str = "urn:otel:otlp:exporter";

/// Configuration for the OTLP Exporter
#[derive(Debug, Deserialize)]
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
    ) -> Result<(), Error> {
        effect_handler
            .info(&format!(
                "Exporting OTLP traffic to endpoint: {}",
                self.config.grpc_endpoint
            ))
            .await;

        let exporter_id = effect_handler.exporter_id();
        let _ = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        // start a grpc client and connect to the server
        let mut metrics_client = MetricsServiceClient::connect(self.config.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: error.to_string(),
            })?;
        let mut logs_client = LogsServiceClient::connect(self.config.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: error.to_string(),
            })?;
        let mut trace_client = TraceServiceClient::connect(self.config.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: error.to_string(),
            })?;

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

        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { .. }) => break,
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report(&mut self.pdata_metrics);
                }
                Message::PData(pdata) => {
                    // Capture signal type before moving pdata into try_from
                    let signal_type = pdata.signal_type();

                    self.pdata_metrics.inc_consumed(signal_type);
                    let service_req: OtlpProtoBytes = pdata
                        .try_into()
                        .inspect_err(|_| self.pdata_metrics.inc_failed(signal_type))?;

                    _ = match service_req {
                        OtlpProtoBytes::ExportLogsRequest(bytes) => {
                            _ = logs_client.export(bytes).await.map_err(|e| {
                                self.pdata_metrics.logs_failed.inc();
                                Error::ExporterError {
                                    exporter: exporter_id.clone(),
                                    error: e.to_string(),
                                }
                            })?;
                            self.pdata_metrics.logs_exported.inc();
                        }
                        OtlpProtoBytes::ExportMetricsRequest(bytes) => {
                            _ = metrics_client.export(bytes).await.map_err(|e| {
                                self.pdata_metrics.metrics_failed.inc();
                                Error::ExporterError {
                                    exporter: exporter_id.clone(),
                                    error: e.to_string(),
                                }
                            })?;
                            self.pdata_metrics.metrics_exported.inc();
                        }
                        OtlpProtoBytes::ExportTracesRequest(bytes) => {
                            _ = trace_client.export(bytes).await.map_err(|e| {
                                self.pdata_metrics.traces_failed.inc();
                                Error::ExporterError {
                                    exporter: exporter_id.clone(),
                                    error: e.to_string(),
                                }
                            })?;
                            self.pdata_metrics.traces_exported.inc();
                        }
                    };
                }
                _ => {
                    // ignore unhandled messages
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::{
        exporter::{TestContext, TestRuntime},
        test_node,
    };
    use otap_df_otlp::grpc::OTLPData;
    use otap_df_otlp::mock::{LogsServiceMock, MetricsServiceMock, TraceServiceMock};
    use otap_df_otlp::proto::opentelemetry::collector::{
        logs::v1::{ExportLogsServiceRequest, logs_service_server::LogsServiceServer},
        metrics::v1::{ExportMetricsServiceRequest, metrics_service_server::MetricsServiceServer},
        trace::v1::{ExportTraceServiceRequest, trace_service_server::TraceServiceServer},
    };
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use prost::Message;
    use std::net::SocketAddr;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};
    use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;

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
                ctx.send_pdata(OtlpProtoBytes::ExportLogsRequest(req_bytes).into())
                    .await
                    .expect("Failed to send log message");

                let req = ExportMetricsServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                ctx.send_pdata(OtlpProtoBytes::ExportMetricsRequest(req_bytes).into())
                    .await
                    .expect("Failed to send metric message");

                let req = ExportTraceServiceRequest::default();
                let mut req_bytes = vec![];
                req.encode(&mut req_bytes).unwrap();
                ctx.send_pdata(OtlpProtoBytes::ExportTracesRequest(req_bytes).into())
                    .await
                    .expect("Failed to send metric message");

                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
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
            .run_validation(validation_procedure(receiver));

        _ = shutdown_sender.send("Shutdown");
    }
}
