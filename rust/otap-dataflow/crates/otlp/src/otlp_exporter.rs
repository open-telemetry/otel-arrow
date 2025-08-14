// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use crate::OTLP_EXPORTER_FACTORIES;
use crate::compression::CompressionMethod;
use crate::grpc::OTLPData;
use crate::proto::opentelemetry::collector::{
    logs::v1::logs_service_client::LogsServiceClient,
    metrics::v1::metrics_service_client::MetricsServiceClient,
    profiles::v1development::profiles_service_client::ProfilesServiceClient,
    trace::v1::trace_service_client::TraceServiceClient,
};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::node::NodeUniq;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{Message, MessageChannel};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::sync::Arc;

/// The URN for the OTLP exporter
pub const OTLP_EXPORTER_URN: &str = "urn:otel:otlp:exporter";

/// Configuration for the OTLP exporter
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The gRPC endpoint to connect to
    pub grpc_endpoint: String,
    /// The compression method to use for the gRPC connection
    pub compression_method: Option<CompressionMethod>,
}

/// Exporter that sends OTLP data via gRPC
pub struct OTLPExporter {
    grpc_endpoint: String,
    compression_method: Option<CompressionMethod>,
}

/// Declares the OTLP exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTLP_EXPORTER_FACTORIES)]
pub static OTLP_EXPORTER: ExporterFactory<OTLPData> = ExporterFactory {
    name: OTLP_EXPORTER_URN,
    create: |node: NodeUniq, node_config: Arc<NodeUserConfig>, exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            OTLPExporter::from_config(&node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

impl OTLPExporter {
    /// Creates a new OTLP exporter
    #[allow(dead_code)]
    #[must_use]
    pub fn new(grpc_endpoint: String, compression_method: Option<CompressionMethod>) -> Self {
        OTLPExporter {
            grpc_endpoint,
            compression_method,
        }
    }

    /// Creates a new OTLPExporter from a configuration object
    #[allow(clippy::result_large_err)]
    pub fn from_config(config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(OTLPExporter {
            grpc_endpoint: config.grpc_endpoint,
            compression_method: config.compression_method,
        })
    }
}

/// Implement the local exporter trait for a OTLP Exporter
#[async_trait(?Send)]
impl local::Exporter<OTLPData> for OTLPExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTLPData>,
        effect_handler: local::EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {
        effect_handler
            .info(&format!(
                "Exporting OTLP traffic to gRPC endpoint: {}",
                self.grpc_endpoint
            ))
            .await;

        // start a grpc client and connect to the server
        let mut metrics_client = MetricsServiceClient::connect(self.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: error.to_string(),
            })?;

        let mut logs_client = LogsServiceClient::connect(self.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: error.to_string(),
            })?;

        let mut trace_client = TraceServiceClient::connect(self.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: error.to_string(),
            })?;
        let mut profiles_client = ProfilesServiceClient::connect(self.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_id(),
                error: error.to_string(),
            })?;

        if let Some(compression) = self.compression_method {
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
            profiles_client = profiles_client
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
                    match message {
                        // match on OTLPData type and use the respective client to send message
                        // ToDo: Add Ack/Nack handling, send a signal that data has been exported
                        OTLPData::Metrics(req) => {
                            _ = metrics_client.export(req).await.map_err(|error| {
                                Error::ExporterError {
                                    exporter: effect_handler.exporter_id(),
                                    error: error.to_string(),
                                }
                            })?;
                        }
                        OTLPData::Logs(req) => {
                            _ = logs_client.export(req).await.map_err(|error| {
                                Error::ExporterError {
                                    exporter: effect_handler.exporter_id(),
                                    error: error.to_string(),
                                }
                            })?;
                        }
                        OTLPData::Traces(req) => {
                            _ = trace_client.export(req).await.map_err(|error| {
                                Error::ExporterError {
                                    exporter: effect_handler.exporter_id(),
                                    error: error.to_string(),
                                }
                            })?;
                        }
                        OTLPData::Profiles(req) => {
                            _ = profiles_client.export(req).await.map_err(|error| {
                                Error::ExporterError {
                                    exporter: effect_handler.exporter_id(),
                                    error: error.to_string(),
                                }
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

    use crate::grpc::OTLPData;
    use crate::mock::{LogsServiceMock, MetricsServiceMock, ProfilesServiceMock, TraceServiceMock};
    use crate::otlp_exporter::{OTLP_EXPORTER_URN, OTLPExporter};
    use crate::proto::opentelemetry::collector::{
        logs::v1::{ExportLogsServiceRequest, logs_service_server::LogsServiceServer},
        metrics::v1::{ExportMetricsServiceRequest, metrics_service_server::MetricsServiceServer},
        profiles::v1development::{
            ExportProfilesServiceRequest, profiles_service_server::ProfilesServiceServer,
        },
        trace::v1::{ExportTraceServiceRequest, trace_service_server::TraceServiceServer},
    };
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use std::net::SocketAddr;
    use std::sync::Arc;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};
    use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario()
    -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // Send a data message
                let metric_message = OTLPData::Metrics(ExportMetricsServiceRequest::default());
                ctx.send_pdata(metric_message)
                    .await
                    .expect("Failed to send metric message");

                let log_message = OTLPData::Logs(ExportLogsServiceRequest::default());
                ctx.send_pdata(log_message)
                    .await
                    .expect("Failed to send log message");

                let trace_message = OTLPData::Traces(ExportTraceServiceRequest::default());
                ctx.send_pdata(trace_message)
                    .await
                    .expect("Failed to send trace message");

                let profile_message = OTLPData::Profiles(ExportProfilesServiceRequest::default());
                ctx.send_pdata(profile_message)
                    .await
                    .expect("Failed to send profile message");

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
        TestContext<OTLPData>,
        Result<(), Error<OTLPData>>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                assert!(exporter_result.is_ok());

                // check that the message was properly sent from the exporter
                let metrics_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Assert that the message received is what the exporter sent
                let _expected_metrics_message = ExportMetricsServiceRequest::default();
                assert!(matches!(metrics_received, _expected_metrics_message));

                let logs_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");
                let _expected_logs_message = ExportLogsServiceRequest::default();
                assert!(matches!(logs_received, _expected_logs_message));

                let traces_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                let _expected_trace_message = ExportTraceServiceRequest::default();
                assert!(matches!(traces_received, _expected_trace_message));

                let profiles_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                let _expected_profiles_message = ExportProfilesServiceRequest::default();
                assert!(matches!(profiles_received, _expected_profiles_message));
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
            let mock_profiles_service =
                ProfilesServiceServer::new(ProfilesServiceMock::new(sender.clone()));
            Server::builder()
                .add_service(mock_logs_service)
                .add_service(mock_metrics_service)
                .add_service(mock_trace_service)
                .add_service(mock_profiles_service)
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
        let exporter = ExporterWrapper::local(
            OTLPExporter::new(grpc_endpoint, None),
            test_runtime.test_node(),
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
