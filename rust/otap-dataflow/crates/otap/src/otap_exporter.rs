// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use crate::OTAP_EXPORTER_FACTORIES;
use crate::grpc::OtapArrowBytes;
use crate::pdata::OtapPdata;
use async_stream::stream;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_otlp::compression::CompressionMethod;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
    arrow_logs_service_client::ArrowLogsServiceClient,
    arrow_metrics_service_client::ArrowMetricsServiceClient,
    arrow_traces_service_client::ArrowTracesServiceClient,
};
use serde::Deserialize;
use serde_json::Value;
use std::sync::Arc;

/// The URN for the OTAP exporter
pub const OTAP_EXPORTER_URN: &str = "urn:otel:otap:exporter";

/// Configuration for the OTAP Exporter
#[derive(Debug, Deserialize)]
pub struct Config {
    grpc_endpoint: String,
    compression_method: Option<CompressionMethod>,
}

/// Exporter that sends OTAP data via gRPC
pub struct OTAPExporter {
    config: Config,
}

/// Declares the OTAP exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static OTAP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTAP_EXPORTER_URN,
    create: |node: NodeId, node_config: Arc<NodeUserConfig>, exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            OTAPExporter::from_config(&node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

impl OTAPExporter {
    /// Creates a new OTAPExporter
    #[must_use]
    pub fn new(grpc_endpoint: String, compression_method: Option<CompressionMethod>) -> Self {
        OTAPExporter {
            config: Config {
                grpc_endpoint,
                compression_method,
            },
        }
    }

    /// Creates a new OTAPExporter from a configuration object
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(OTAPExporter { config })
    }
}

/// Implement the local exporter trait for a OTAP Exporter
#[async_trait(?Send)]
impl local::Exporter<OtapPdata> for OTAPExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error<OtapPdata>> {
        effect_handler
            .info(&format!(
                "Exporting OTLP traffic to endpoint: {}",
                self.config.grpc_endpoint
            ))
            .await;
        // start a grpc client and connect to the server
        let mut arrow_metrics_client =
            ArrowMetricsServiceClient::connect(self.config.grpc_endpoint.clone())
                .await
                .map_err(|error| Error::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    error: error.to_string(),
                })?;

        let mut arrow_logs_client =
            ArrowLogsServiceClient::connect(self.config.grpc_endpoint.clone())
                .await
                .map_err(|error| Error::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    error: error.to_string(),
                })?;

        let mut arrow_traces_client =
            ArrowTracesServiceClient::connect(self.config.grpc_endpoint.clone())
                .await
                .map_err(|error| Error::ExporterError {
                    exporter: effect_handler.exporter_id(),
                    error: error.to_string(),
                })?;

        if let Some(compression) = self.config.compression_method {
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
        // Loop until a Shutdown event is received.
        loop {
            match msg_chan.recv().await? {
                // handle control messages
                Message::Control(NodeControlMsg::TimerTick { .. })
                | Message::Control(NodeControlMsg::Config { .. }) => {}
                // shutdown the exporter
                Message::Control(NodeControlMsg::Shutdown { .. }) => {
                    // ToDo: add proper deadline function
                    break;
                }
                //send data
                Message::PData(message) => {
                    let message: OtapArrowBytes = message.try_into()?;

                    match message {
                        // match on OTAPData type and use the respective client to send message
                        // ToDo: Add Ack/Nack handling, send a signal that data has been exported
                        // check what message the data is
                        OtapArrowBytes::ArrowMetrics(req) => {
                            // handle stream differently here?
                            // ToDo: [LQ or someone else] Check if there is a better way to handle that.
                            let request_stream = stream! {
                                yield req;
                            };
                            _ = arrow_metrics_client
                                .arrow_metrics(request_stream)
                                .await
                                .map_err(|error| Error::ExporterError {
                                    exporter: effect_handler.exporter_id(),
                                    error: error.to_string(),
                                })?;
                        }
                        OtapArrowBytes::ArrowLogs(req) => {
                            let request_stream = stream! {
                                yield req;
                            };
                            _ = arrow_logs_client.arrow_logs(request_stream).await.map_err(
                                |error| Error::ExporterError {
                                    exporter: effect_handler.exporter_id(),
                                    error: error.to_string(),
                                },
                            )?;
                        }
                        OtapArrowBytes::ArrowTraces(req) => {
                            let request_stream = stream! {
                                yield req;
                            };
                            _ = arrow_traces_client
                                .arrow_traces(request_stream)
                                .await
                                .map_err(|error| Error::ExporterError {
                                    exporter: effect_handler.exporter_id(),
                                    error: error.to_string(),
                                })?;
                        }
                    }
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_id(),
                        error: "Unknown control message".to_owned(),
                    });
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use crate::grpc::OtapArrowBytes;
    use crate::mock::{
        ArrowLogsServiceMock, ArrowMetricsServiceMock, ArrowTracesServiceMock,
        create_batch_arrow_record,
    };
    use crate::otap_exporter::OTAP_EXPORTER_URN;
    use crate::otap_exporter::OTAPExporter;
    use crate::pdata::OtapPdata;

    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::{
        exporter::{TestContext, TestRuntime},
        test_node,
    };
    use otap_df_otlp::compression::CompressionMethod;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, arrow_logs_service_server::ArrowLogsServiceServer,
        arrow_metrics_service_server::ArrowMetricsServiceServer,
        arrow_traces_service_server::ArrowTracesServiceServer,
    };
    use serde_json::json;
    use std::net::SocketAddr;
    use std::sync::Arc;
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
                let metric_message = OtapArrowBytes::ArrowMetrics(create_batch_arrow_record(
                    METRIC_BATCH_ID,
                    ArrowPayloadType::MultivariateMetrics,
                ));
                ctx.send_pdata(metric_message.into())
                    .await
                    .expect("Failed to send metric message");

                let log_message = OtapArrowBytes::ArrowLogs(create_batch_arrow_record(
                    LOG_BATCH_ID,
                    ArrowPayloadType::Logs,
                ));
                ctx.send_pdata(log_message.into())
                    .await
                    .expect("Failed to send log message");

                let trace_message = OtapArrowBytes::ArrowTraces(create_batch_arrow_record(
                    TRACE_BATCH_ID,
                    ArrowPayloadType::Spans,
                ));
                ctx.send_pdata(trace_message.into())
                    .await
                    .expect("Failed to send trace message");

                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
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
        Result<(), Error<OtapPdata>>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                assert!(exporter_result.is_ok());

                // check that the message was properly sent from the exporter
                let metrics_received: OtapArrowBytes =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .try_into()
                        .expect("Could convert pdata to OTAPData");

                // Assert that the message received is what the exporter sent
                let _expected_metrics_message = create_batch_arrow_record(
                    METRIC_BATCH_ID,
                    ArrowPayloadType::MultivariateMetrics,
                );
                assert!(matches!(metrics_received, _expected_metrics_message));

                let logs_received: OtapArrowBytes =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .try_into()
                        .expect("Could convert pdata to OTAPData");
                let _expected_logs_message =
                    create_batch_arrow_record(LOG_BATCH_ID, ArrowPayloadType::Logs);
                assert!(matches!(logs_received, _expected_logs_message));

                let traces_received: OtapArrowBytes =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .try_into()
                        .expect("Could convert pdata to OTAPData");

                let _expected_trace_message =
                    create_batch_arrow_record(TRACE_BATCH_ID, ArrowPayloadType::Spans);
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
        let exporter = ExporterWrapper::local(
            OTAPExporter::new(grpc_endpoint, None),
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
            "compression_method": "Gzip"
        });

        let exporter = OTAPExporter::from_config(&json_config).expect("Config should be valid");

        assert_eq!(exporter.config.grpc_endpoint, "http://localhost:4317");
        match exporter.config.compression_method {
            Some(ref method) => match method {
                CompressionMethod::Gzip => {} // success
                other => panic!("Expected Gzip, got {:?}", other),
            },
            None => panic!("Expected Some compression method"),
        }
    }

    #[test]
    fn test_from_config_missing_required_field() {
        let json_config = json!({
            "compression_method": "Gzip"
        });

        let result = OTAPExporter::from_config(&json_config);

        assert!(result.is_err());
        if let Err(err) = result {
            let err_msg = format!("{}", err);
            assert!(err_msg.contains("missing field `grpc_endpoint`"));
        }
    }
}
