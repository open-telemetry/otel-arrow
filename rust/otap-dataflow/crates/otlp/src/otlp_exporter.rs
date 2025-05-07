// ToDo: Handle Ack and Nack messages in the pipeline


use crate::grpc::{LogsServiceImpl, MetricsServiceImpl, TraceServiceImpl, OTLPRequest};
use crate::grpc::grpc_stubs::proto::collector::{logs::v1::logs_service_client::LogsServiceClient,
    metrics::v1::metrics_service_client::MetricsServiceClient,
    trace::v1::trace_service_client::TraceServiceClient};
use otap_df_engine::exporter::{EffectHandlerTrait, Exporter, MessageChannel, SendEffectHandler};
use otap_df_engine::error::Error;
use otap_df_engine::message::{ControlMsg, Message};
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};
use tonic::codec::CompressionEncoding;


#[derive(Debug)]
pub enum CompressionMethod {
    Zstd,
    Gzip,
    Deflate,
}

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

#[async_trait(?Send)]
impl Exporter<OTLPRequest, SendEffectHandler<OTLPRequest>> for OTLPExporter {

    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<OTLPRequest>,
        effect_handler: SendEffectHandler<OTLPRequest>,
    ) -> Result<(), Error<OTLPRequest>> {

        // check for compression
        let compression_encoding = match self.compression {
            Some(CompressionMethod::Gzip) => Some(CompressionEncoding::Gzip),
            Some(CompressionMethod::Zstd) => Some(CompressionEncoding::Zstd),
            Some(CompressionMethod::Deflate) => Some(CompressionEncoding::Deflate),
            Some(CompressionMethod::None) => None,
            _ => None,
        };
        // Loop until a Shutdown event is received.

        // start a grpc client and send data from it to the provided addr

        let mut metrics_client = MetricsServiceClient::connect(self.grpc_endpoint.clone()).await.expect("Couldn't connect to server");
        let mut logs_client = LogsServiceClient::connect(self.grpc_endpoint.clone()).await.expect("Couldn't connect to server");
        let mut traces_client = TraceServiceClient::connect(self.grpc_endpoint.clone()).await.expect("Couldn't connect to server");

        if let Some(encoding) = compression_encoding {
            metrics_client = metrics_client.send_compressed(encoding).accept_compressed(encoding);
            logs_client = logs_client.send_compressed(encoding).accept_compressed(encoding);
            traces_client = traces_client.send_compressed(encoding).accept_compressed(encoding);
        }
   
        loop {
            match msg_chan.recv().await? {
                // handle control messages
                Message::Control(ControlMsg::TimerTick { .. }) |
                Message::Control(ControlMsg::Config { .. }) => {
                }
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    break;
                }
                //send data
                Message::PData(message) => {
                    match message {
                        // match on OTLPRequest type and use the respective client to send message
                        OTLPRequest::Metrics(req) => {
                            let _ = metrics_client.export(req).await;
                        }
                        OTLPRequest::Logs(req) => {
                            let _ = logs_client.export(req).await;
                        }
                        OTLPRequest::Traces(req) => {
                            let _ = traces_client.export(req).await;
                        }
                    }
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_name().to_string(),
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
    use otap_df_engine::exporter::{
        EffectHandlerTrait, Error, Exporter, ExporterWrapper, MessageChannel,
        SendEffectHandler,
    };
    use crate::grpc::grpc_stubs::proto::collector::{metrics::v1::, trace::v1::, logs::v1, };

    use grpc_stubs::proto::collector::metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
    };
    use grpc_stubs::proto::collector::trace::v1::trace_service_server::TraceService;
    use grpc_stubs::proto::collector::trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse,
    };
    use otap_df_engine::message::{ControlMsg, Message};
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
    use async_trait::async_trait;
    use otap_df_channel::error::RecvError;
    use otap_df_channel::mpsc;
    use serde_json::Value;
    use std::future::Future;
    use std::time::Duration;
    use tokio::time::sleep;
    use crate::mock::run_mock_server;
    use crate::otlp_exporter::OTLPExporter;

    
    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario(receiver: Receiver) -> impl FnOnce(TestContext<TestMsg>) -> std::pin::Pin<Box<dyn Future<Output = ()>>>
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
                let metric_message = OTLPRequest::Metrics(ExportMetricsServiceRequest::default());
                ctx.send_pdata(metric_message)
                    .await
                    .expect("Failed to send metric message");


                
                let log_message = OTLPRequest::Logs(ExportLogsServiceRequest::default());
                ctx.send_pdata(log_message)
                    .await
                    .expect("Failed to send log message");

                let trace_message = OTLPRequest::Traces(ExportTraceServiceRequest::default());
                ctx.send_pdata(trace_message)
                    .await
                    .expect("Failed to send trace message");


                // Send shutdown
                ctx.send_shutdown(Duration::from_millis(200), "test complete")
                    .await
                    .expect("Failed to send Shutdown");


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
    fn test_exporter_with_send_effect_handler() {
        let test_runtime = TestRuntime::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        let (shutdown_sender, shutdown_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = "4317";
        let grpc_endpoint = format!("http::{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let exporter = ExporterWrapper::with_send(
            OTLPExporter(grpc_endpoint: grpc_endpoint),
            test_runtime.config(),
        );
        run_mock_server(sender: sender, listening_addr: addr, shutdown_signal: shutdown_receiver);
        test_runtime
            .set_exporter(exporter)
            .run_test(scenario(receiver: receiver));
        shutdown_sender.send("Shutdown").await;
    }   

}
