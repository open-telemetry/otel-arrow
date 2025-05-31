// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP exporter node
//!
//! ToDo: Handle Ack and Nack messages in the pipeline
//! ToDo: Handle configuratin changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

use crate::grpc::OTAPData;
use crate::proto::opentelemetry::experimental::arrow::v1::{
    arrow_logs_service_client::ArrowLogsServiceClient,
    arrow_metrics_service_client::ArrowMetricsServiceClient,
    arrow_traces_service_client::ArrowTracesServiceClient,
};
use async_stream::stream;
use async_trait::async_trait;
use otap_df_engine::error::Error;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{ControlMsg, Message, MessageChannel};
use otap_df_otlp::compression::CompressionMethod;

/// Exporter that sends OTAP data via gRPC
struct OTAPExporter {
    grpc_endpoint: String,
    compression_method: Option<CompressionMethod>,
}

impl OTAPExporter {
    /// Creates a new OTAP exporter
    #[must_use]
    #[allow(dead_code)]
    pub fn new(grpc_endpoint: String, compression_method: Option<CompressionMethod>) -> Self {
        OTAPExporter {
            grpc_endpoint,
            compression_method,
        }
    }
}

/// Implement the local exporter trait for a OTAP Exporter
#[async_trait(?Send)]
impl local::Exporter<OTAPData> for OTAPExporter {
    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTAPData>,
        effect_handler: local::EffectHandler<OTAPData>,
    ) -> Result<(), Error<OTAPData>> {
        // start a grpc client and connect to the server
        let mut arrow_metrics_client =
            ArrowMetricsServiceClient::connect(self.grpc_endpoint.clone())
                .await
                .map_err(|error| Error::ExporterError {
                    exporter: effect_handler.exporter_name(),
                    error: error.to_string(),
                })?;

        let mut arrow_logs_client = ArrowLogsServiceClient::connect(self.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_name(),
                error: error.to_string(),
            })?;

        let mut arrow_traces_client = ArrowTracesServiceClient::connect(self.grpc_endpoint.clone())
            .await
            .map_err(|error| Error::ExporterError {
                exporter: effect_handler.exporter_name(),
                error: error.to_string(),
            })?;

        if let Some(compression) = self.compression_method {
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
                Message::Control(ControlMsg::TimerTick { .. })
                | Message::Control(ControlMsg::Config { .. }) => {}
                // shutdown the exporter
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    // ToDo: add proper deadline function
                    break;
                }
                //send data
                Message::PData(message) => {
                    match message {
                        // match on OTAPData type and use the respective client to send message
                        // ToDo: Add Ack/Nack handling, send a signal that data has been exported
                        // check what message the data is
                        OTAPData::ArrowMetrics(req) => {
                            // handle stream differently here?
                            // ToDo: [LQ or someone else] Check if there is a better way to handle that.
                            let request_stream = stream! {
                                yield req;
                            };
                            _ = arrow_metrics_client
                                .arrow_metrics(request_stream)
                                .await
                                .map_err(|error| Error::ExporterError {
                                    exporter: effect_handler.exporter_name(),
                                    error: error.to_string(),
                                })?;
                        }
                        OTAPData::ArrowLogs(req) => {
                            let request_stream = stream! {
                                yield req;
                            };
                            _ = arrow_logs_client.arrow_logs(request_stream).await.map_err(
                                |error| Error::ExporterError {
                                    exporter: effect_handler.exporter_name(),
                                    error: error.to_string(),
                                },
                            )?;
                        }
                        OTAPData::ArrowTraces(req) => {
                            let request_stream = stream! {
                                yield req;
                            };
                            _ = arrow_traces_client
                                .arrow_traces(request_stream)
                                .await
                                .map_err(|error| Error::ExporterError {
                                    exporter: effect_handler.exporter_name(),
                                    error: error.to_string(),
                                })?;
                        }
                    }
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_name(),
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

    use crate::grpc::OTAPData;
    use crate::mock::{
        ArrowLogsServiceMock, ArrowMetricsServiceMock, ArrowTracesServiceMock,
        create_batch_arrow_record,
    };
    use crate::otap_exporter::OTAPExporter;
    use crate::proto::opentelemetry::experimental::arrow::v1::{
        ArrowPayloadType, BatchArrowRecords, arrow_logs_service_server::ArrowLogsServiceServer,
        arrow_metrics_service_server::ArrowMetricsServiceServer,
        arrow_traces_service_server::ArrowTracesServiceServer,
    };
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use std::net::SocketAddr;
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
    -> impl FnOnce(TestContext<OTAPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // Send a data message
                let metric_message = OTAPData::ArrowMetrics(create_batch_arrow_record(
                    METRIC_BATCH_ID,
                    ArrowPayloadType::MultivariateMetrics,
                ));
                ctx.send_pdata(metric_message)
                    .await
                    .expect("Failed to send metric message");

                let log_message = OTAPData::ArrowLogs(create_batch_arrow_record(
                    LOG_BATCH_ID,
                    ArrowPayloadType::Logs,
                ));
                ctx.send_pdata(log_message)
                    .await
                    .expect("Failed to send log message");

                let trace_message = OTAPData::ArrowTraces(create_batch_arrow_record(
                    TRACE_BATCH_ID,
                    ArrowPayloadType::Spans,
                ));
                ctx.send_pdata(trace_message)
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
        mut receiver: tokio::sync::mpsc::Receiver<OTAPData>,
    ) -> impl FnOnce(TestContext<OTAPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_| {
            Box::pin(async move {
                // check that the message was properly sent from the exporter
                let metrics_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Assert that the message received is what the exporter sent
                let _expected_metrics_message = create_batch_arrow_record(
                    METRIC_BATCH_ID,
                    ArrowPayloadType::MultivariateMetrics,
                );
                assert!(matches!(metrics_received, _expected_metrics_message));

                let logs_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");
                let _expected_logs_message =
                    create_batch_arrow_record(LOG_BATCH_ID, ArrowPayloadType::Logs);
                assert!(matches!(logs_received, _expected_logs_message));

                let traces_received = timeout(Duration::from_secs(3), receiver.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

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
        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        // tokio runtime to run grpc server in the background
        let tokio_rt = Runtime::new().unwrap();

        // run a gRPC concurrently to receive data from the exporter
        _ = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            let mock_logs_service =
                ArrowLogsServiceServer::new(ArrowLogsServiceMock::new(sender.clone()));
            let mock_metrics_service =
                ArrowMetricsServiceServer::new(ArrowMetricsServiceMock::new(sender.clone()));
            let mock_trace_service =
                ArrowTracesServiceServer::new(ArrowTracesServiceMock::new(sender.clone()));
            _ = Server::builder()
                .add_service(mock_logs_service)
                .add_service(mock_metrics_service)
                .add_service(mock_trace_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    // Wait for the shutdown signal
                    drop(shutdown_signal.await.ok())
                })
                .await
                .expect("Test gRPC server has failed");
        });

        let exporter = ExporterWrapper::local(
            OTAPExporter::new(grpc_endpoint, None),
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(receiver));

        _ = shutdown_sender.send("Shutdown");
    }
}
