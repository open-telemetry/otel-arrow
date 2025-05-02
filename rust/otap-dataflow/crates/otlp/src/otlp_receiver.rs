// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).
//! 
//! ToDo implement Ack and Nack control message, wait for receiver node to receive a Ack control message then the service can send a response back 
//! ToDo implement config control message to handle live changing configuration

use crate::grpc::{LogsServiceImpl, MetricsServiceImpl, TraceServiceImpl, OTLPRequest};
use crate::grpc_stubs::proto::collector::{logs::v1::logs_service_server::LogsServiceServer,
    metrics::v1::metrics_service_server::MetricsServiceServer,
    trace::v1::trace_service_server::TraceServiceServer};
use otap_df_engine::receiver::{EffectHandlerTrait, Receiver, ControlMsgChannel, SendEffectHandler};
use otap_df_engine::error::Error;
use otap_df_engine::message::ControlMsg;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::codec::CompressionEncoding;
use tonic::transport::Server;

/// Enum to represent varioous compression methods
#[derive(Debug)]
pub enum CompressionMethod {
    /// Fastest compression
    Zstd,
    /// Most compatible compression method
    Gzip,
    /// Used for legacy systems
    Deflate,
    /// Don't use any compression, this is the default if no compression is specified
    None, 
}

/// A Receiver that listens for OTLP messages
pub struct OTLPReceiver {
    listening_addr: SocketAddr,
    compression: Option<CompressionMethod>
}

#[async_trait(?Send)]
impl Receiver<OTLPRequest, SendEffectHandler<OTLPRequest>>  for OTLPReceiver
{
    async fn start(
        self: Box<Self>,
        ctrl_msg_recv: ControlMsgChannel,
        effect_handler: SendEffectHandler<OTLPRequest>,
    ) -> Result<(), Error<OTLPRequest>> {

        // create listener on addr provided from config
        
        // check for compression method
        let compression_encoding = match self.compression {
            Some(CompressionMethod::Gzip) => Some(CompressionEncoding::Gzip),
            Some(CompressionMethod::Zstd) => Some(CompressionEncoding::Zstd),
            Some(CompressionMethod::Deflate) => Some(CompressionEncoding::Deflate),
            Some(CompressionMethod::None) => None,
            _ => None,
        };


        //start event loop
        loop {
            //create services for the grpc server and clone the effect handler to pass message
            let logs_service = LogsServiceImpl::new(effect_handler.clone());
            let metrics_service = MetricsServiceImpl::new(effect_handler.clone());
            let trace_service = TraceServiceImpl::new(effect_handler.clone());

            let logs_service_server;
            let metrics_service_server;
            let trace_service_server;

            // check if a compression method was set
            if let Some(encoding) = compression_encoding {
                // define servicees with compression
                logs_service_server = LogsServiceServer::new(logs_service).send_compressed(encoding).accept_compressed(encoding);
                metrics_service_server = MetricsServiceServer::new(metrics_service).send_compressed(encoding).accept_compressed(encoding);
                trace_service_server = TraceServiceServer::new(trace_service).send_compressed(encoding).accept_compressed(encoding);
            } else {
                // define servicees without compression
                logs_service_server = LogsServiceServer::new(logs_service);
                metrics_service_server = MetricsServiceServer::new(metrics_service);
                trace_service_server = TraceServiceServer::new(trace_service);
            }

            let listener = effect_handler.tcp_listener(self.listening_addr)?;
            let listener_stream = TcpListenerStream::new(listener);
            
            tokio::select! {
                biased; //prioritize ctrl_msg over all other blocks

                // Process internal event
                mut ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(ControlMsg::Shutdown {reason, deadline}) => {
                            // wait for deadline then shutdown
                            let _ = sleep(deadline);
                            break;
                        },
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message do nothing
                            
                        }
                    }
                }
                // Poll the grpc server
                result = Server::builder()
                .add_service(logs_service_server)
                .add_service(metrics_service_server)
                .add_service(trace_service_server)
                .serve_with_incoming(listener_stream)=> {
                    if let Err(e) = result {
                        break;
                    }
                }
                // A timeout branch in case no events occur.
                _ = sleep(Duration::from_secs(1)) => {
                    // wait for next event
                }
            }
        }
        //Exit event loop
        Ok(())
    }

}

#[cfg(test)]
mod tests {
    use crate::grpc::OTLPRequest;
    use crate::otlp_receiver::OTLPReceiver;
    use crate::grpc_stubs::proto::collector::{
        logs::v1::{logs_service_client::LogsServiceClient, ExportLogsServiceRequest}, 
        metrics::v1::{metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest}, 
        trace::v1::{trace_service_client::TraceServiceClient, ExportTraceServiceRequest}};
    use otap_df_engine::error::Error;
    use otap_df_engine::message::ControlMsg;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use otap_df_engine::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
    use std::net::SocketAddr;
    use tokio::time::{Duration, timeout};
    use std::future::Future;
    use std::pin::Pin;

    /// A type alias for a test receiver with sendable effect handler
    type ReceiverWithSendEffectHandler = OTLPReceiver;

    /// Test closure that simulates a typical receiver scenario.
    fn scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // send data to the receiver

                // connect to the different clients and call export to send a message
                let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone()).await.expect("Failed to connect to server from Metrics Service Client");

                let _metrics_response = metrics_client.export(ExportMetricsServiceRequest::default()).await.expect("Failed to receive response after sending Metrics Request");
    
                let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Logs Service Client");
                let _logs_response = logs_client.export(ExportLogsServiceRequest::default()).await.expect("Failed to receive response after sending Logs Request");
    
                let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Trace Service Client");
                let _traces_response = traces_client.export(ExportTraceServiceRequest::default()).await.expect("Failed to receive response after sending Trace Request");

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(200), "Test")
                    .await
                    .expect("Failed to send Shutdown");
            })
        }
    }



    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OTLPRequest>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                
                // read from the effect handler
                let metrics_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Assert that the message received is what the test client sent.
                let expected_metrics_message = ExportMetricsServiceRequest::default();
                assert!(matches!(metrics_received, expected_metrics_message));

                let logs_received = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");
                let expected_logs_message = ExportLogsServiceRequest::default();
                assert!(matches!(logs_received, expected_logs_message));


                let traces_received = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");

                let expected_trace_message =  ExportTraceServiceRequest::default();
                assert!(matches!(traces_received, expected_trace_message));

                // check that control messages were received
                ctx.counters().assert(0, 0, 0, 1);

            })
        }
    }

    #[test]
    fn test_receiver_with_send_effect_handler() {
        let test_runtime = TestRuntime::new();

        // addr and port for the server to run at
        let grpc_addr = "127.0.0.1";
        let grpc_port = "4317";
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        // create our receiver
        let receiver = ReceiverWrapper::with_send(
            OTLPReceiver {
                listening_addr: addr,
                compression: None
            },
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint))
            .run_validation(validation_procedure());
    }
}

