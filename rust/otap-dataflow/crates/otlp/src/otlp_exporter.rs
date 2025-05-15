// SPDX-License-Identifier: Apache-2.0
// ToDo: Handle Ack and Nack messages in the pipeline


use crate::grpc::{OTLPData, CompressionMethod};
use crate::proto::opentelemetry::collector::{logs::v1::logs_service_client::LogsServiceClient,
    metrics::v1::metrics_service_client::MetricsServiceClient,
    trace::v1::trace_service_client::TraceServiceClient
    profiles::v1development::profiles_service_client::ProfilesServiceClient};
use otap_df_engine::local::exporter as local;
use otap_df_engine::error::Error;
use otap_df_engine::message::{ControlMsg, Message};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};
use tonic::codec::CompressionEncoding;


///
struct OTLPExporter {
    grpc_endpoint: String,
    compression_method: Option<CompressionMethod>
}

impl OTLPExporter {
    /// Creates a new test exporter with the given counter.
    pub fn new(grpc_endpoint: String, compression_method: Option<CompressionMethod>) -> Self {
        OTLPExporter { grpc_endpoint: grpc_endpoint, compression_method: compression_method }
    }
}

///
#[async_trait(?Send)]
impl local::Exporter<OTLPData> for OTLPExporter {

    async fn start(
        self: Box<Self>,
        mut msg_chan: local::MessageChannel<OTLPData>,
        effect_handler: local::EffectHandler<OTLPData>,
    ) -> Result<(), Error<OTLPData>> {

        // check for compression
        let compression_encoding = match self.compression_method {
            Some(CompressionMethod::Gzip) => Some(CompressionEncoding::Gzip),
            Some(CompressionMethod::Zstd) => Some(CompressionEncoding::Zstd),
            Some(CompressionMethod::Deflate) => Some(CompressionEncoding::Deflate),
            _ => None,
        };
        // Loop until a Shutdown event is received.

        // start a grpc client and connect to the server
        let mut metrics_client = MetricsServiceClient::connect(self.grpc_endpoint.clone()).await.expect("Metrics client couldn't connect to server");
        let mut logs_client = LogsServiceClient::connect(self.grpc_endpoint.clone()).await.expect("Logs client couldn't connect to server");
        let mut traces_client = TraceServiceClient::connect(self.grpc_endpoint.clone()).await.expect("Trace client couldn't connect to server");
        let mut profiles_client = ProfilesServiceClient::connect(self.grpc_endpoint.clone()).await.expect("Trace client couldn't connect to server");
        if let Err(error) = metrics_client {
            return Err(Error::ExporterError {
                exporter: effect_handler.exporter_name(),
                error: error.to_string()
            });
        } else if let Err(error) = logs_client {
            return Err(Error::ExporterError {
                exporter: effect_handler.exporter_name(),
                error: error.to_string()
            });
        } else if let Err(error) = traces_client {
            return Err(Error::ExporterError {
                exporter: effect_handler.exporter_name(),
                error: error.to_string()
            });
        }


        if let Some(encoding) = compression_encoding {
            metrics_client = metrics_client.send_compressed(encoding).accept_compressed(encoding);
            logs_client = logs_client.send_compressed(encoding).accept_compressed(encoding);
            traces_client = traces_client.send_compressed(encoding).accept_compressed(encoding);
        }
   
        loop {
            match msg_chan.recv().await? {
                // handle control messages
                //
                Message::Control(ControlMsg::TimerTick { .. }) |
                Message::Control(ControlMsg::Config { .. }) => {
                }
                // shutdown the exporter
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    break;
                }
                //send data
                Message::PData(message) => {
                    match message {
                        // match on OTLPData type and use the respective client to send message
                        OTLPData::Metrics(req) => {
                            let _ = metrics_client.export(req).await;
                        }
                        OTLPData::Logs(req) => {
                            let _ = logs_client.export(req).await;
                        }
                        OTLPData::Traces(req) => {
                            let _ = traces_client.export(req).await;
                        }
                        OTLPData::Profiles(req) => {
                            let _ = profiles_client.export(req).await;
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
    use crate::proto::opentelemetry::collector::logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse,
    };
    use crate::proto::opentelemetry::collector::metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
    };
    use crate::proto::opentelemetry::collector::trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse,
    };
    use crate::otlp_exporter::OTLPExporter;
    use crate::grpc::OTLPData;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::message::{ControlMsg, Message};
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::local::exporter as local;
    use otap_df_engine::error::Error;
    use otap_df_engine::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
    use otap_df_channel::error::RecvError;
    use tokio::sync::mpsc::{Receiver, channel};
    use serde_json::Value;
    use std::future::Future;
    use std::net::SocketAddr;
    use tokio::time::{Duration, timeout};
    use tokio::time::sleep;
    use crate::mock::{LogsServiceMock, MetricsServiceMock, TraceServiceMock};
    use crate::mock::start_mock_server;
    use crate::proto::opentelemetry::collector::{logs::v1::logs_service_server::LogsServiceServer,
        metrics::v1::metrics_service_server::MetricsServiceServer,
        trace::v1::trace_service_server::TraceServiceServer};
    use tonic::transport::Server;
    use tokio::runtime::Runtime;
    
    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario() -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
    {
        |ctx| {
            Box::pin(async move {
                // Send 3 TimerTick events.
                for _ in 0..3 {
                    ctx.send_timer_tick()
                        .await
                        .expect("Failed to send TimerTick");
                    ctx.sleep(Duration::from_millis(50)).await;
                }

                // Send a Config event.
                ctx.send_config(Value::Null)
                    .await
                    .expect("Failed to send Config");

                // spin up server for the exporter to send messages to
  
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


                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(mut receiver: Receiver<OTLPData>)
    -> impl FnOnce(TestContext<OTLPData>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {

                // check that the message was properly sent from the exporter
                let metrics_received = timeout(Duration::from_secs(3), receiver.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");

                // Assert that the message received is what the exporter sent
                let expected_metrics_message = ExportMetricsServiceRequest::default();
                assert!(matches!(metrics_received, expected_metrics_message));

                let logs_received = timeout(Duration::from_secs(3), receiver.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");
                let expected_logs_message = ExportLogsServiceRequest::default();
                assert!(matches!(logs_received, expected_logs_message));


                let traces_received = timeout(Duration::from_secs(3), receiver.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");

                let expected_trace_message =  ExportTraceServiceRequest::default();
                assert!(matches!(traces_received, expected_trace_message));
            })
        }
    }

    #[test]
    fn test_otlp_exporter() {
        let test_runtime = TestRuntime::new();
        let (sender, mut receiver) = tokio::sync::mpsc::channel(32);
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = "4317";
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let exporter = ExporterWrapper::local(
            OTLPExporter::new(grpc_endpoint, None),
            test_runtime.config(),
        );

        let tokio_rt = Runtime::new().unwrap();

        // let mock_logs_service = LogsServiceServer::new(LogsServiceMock::new(sender.clone()));
        // let mock_metrics_service = MetricsServiceServer::new(MetricsServiceMock::new(sender.clone()));
        // let mock_trace_service = TraceServiceServer::new(TraceServiceMock::new(sender.clone()));
    
        // let _ = tokio_rt.spawn(async move {
        //     let _ = Server::builder().add_service(mock_logs_service).add_service(mock_metrics_service).add_service(mock_trace_service).serve_with_shutdown(listening_addr, async {
        //         // Wait for the shutdown signal
        //         drop(shutdown_signal.await.ok())
        //     }).await;
        // });

        let _ = tokio_rt.spawn(async move {
            let _ = start_mock_server(sender, listening_addr, shutdown_signal).await;
        })
        // let server_handle = start_mock_server(sender, listening_addr, shutdown_signal);

        
        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(receiver));


        let _ = shutdown_sender.send("Shutdown");
    }   

}
