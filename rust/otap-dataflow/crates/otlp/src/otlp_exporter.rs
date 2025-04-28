use crate::grpc::OTLPRequest;
use otap_df_engine::exporter::{EffectHandler, Receiver, ControlMsgChannel, SendableMode};
use crate::grpc::{LogsServiceImpl, MetricsServiceImpl, TraceServiceImpl, OTLPRequest};
use crate::grpc::grpc_stubs::proto::collector::{logs::v1::logs_service_client::LogsServiceClient,
    metrics::v1::metrics_service_client::MetricsServiceClient,
    trace::v1::trace_service_client::TraceServiceClient};
use otap_df_engine::receiver::{EffectHandler, Receiver, ControlMsgChannel, SendableMode};
use otap_df_engine::error::Error;
use otap_df_engine::message::ControlMsg;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::codec::CompressionEncoding;
use tonic::transport::Server;

struct OTLPExporter {
    grpc_addr: &str,
    grpc_port: u16
}

impl OTLPExporter {
    /// Creates a new test exporter with the given counter.
    pub fn new(grpc_addr: &str, grpc_port: u16) -> Self {
        OTLPExporter { grpc_addr: grpc_addr, grpc_port: grpc_port }
    }
}

#[async_trait(?Send)]
impl Exporter for TestExporter {
    type PData = OTLPRequest;
    type Mode = LocalMode;

    async fn start(
        self: Box<Self>,
        mut msg_chan: MessageChannel<Self::PData>,
        effect_handler: EffectHandler<Self::PData, Self::Mode>,
    ) -> Result<(), Error<Self::PData>> {
        // Loop until a Shutdown event is received.
        // start a grpc client and send data from it to the provided addr

        // sent signals should we send ack and nack control messages??/
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone()).await?;
        let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone()).await?;
        let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone()).await?;
   
        loop {
            match msg_chan.recv().await? {
                Message::Control(ControlMsg::TimerTick { .. }) => {
                }
                Message::Control(ControlMsg::Config { .. }) => {
                }
                Message::Control(ControlMsg::Shutdown { .. }) => {
                    break;
                }
                Message::PData(message) => {
                    match message {
                        OTLPRequest::Metrics(req) => {
                            let metrics_response = metrics_client.export(req).await;
                            //check response and return error 
                        }
                        OTLPRequest::Logs(req) => {
                            let logs_response = logs_client.export(req).await;
                            //check response and return error 
                        }
                        OTLPRequest::Traces(req) => {
                            let trace_response = trace_client.export(req).await;
                            //check response and return error 
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
    use crate::exporter::{EffectHandler, Error, Exporter, MessageChannel};
    use crate::message::{ControlMsg, Message};
    use crate::receiver::LocalMode;
    use crate::testing::exporter::ExporterTestRuntime;
    use crate::testing::{CtrMsgCounters, TestMsg};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::time::Duration;

    #[test]
    fn test_exporter() {
        let mut test_runtime = ExporterTestRuntime::new(10);
        let exporter = OTLPExporter::new("127.0.0.1", 1111);

        test_runtime.start_exporter(exporter);
        test_runtime.start_test(|ctx| async move {
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

            // Send a data message
            ctx.send_data("Hello Exporter")
                .await
                .expect("Failed to send data message");

            // Allow some time for processing
            ctx.sleep(Duration::from_millis(100)).await;

            // Send shutdown
            ctx.send_shutdown("test complete")
                .await
                .expect("Failed to send Shutdown");
        });


        test_runtime.validate(|_ctx| async move {
            counters.assert(
                3, // timer tick
                1, // message
                1, // config
                1, // shutdown
            );
        });
    }
}
