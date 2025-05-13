// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).
//! 
//! ToDo implement Ack and Nack control message, wait for receiver node to receive a Ack control message then the service can send a response back 
//! ToDo implement config control message to handle live changing configuration

use crate::grpc::{LogsServiceImpl, MetricsServiceImpl, TraceServiceImpl, ProfilesServiceImpl, OTLPRequest};
use crate::proto::opentelemetry::collector::{logs::v1::logs_service_server::LogsServiceServer,
    metrics::v1::metrics_service_server::MetricsServiceServer,
    trace::v1::trace_service_server::TraceServiceServer,
    profiles::v1development::profiles_service_server::ProfilesServiceServer};
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
}

/// A Receiver that listens for OTLP messages
pub struct OTLPReceiver {
    listening_addr: SocketAddr,
    compression: Option<CompressionMethod>,
}

#[async_trait]
impl Receiver<OTLPRequest, SendEffectHandler<OTLPRequest>>  for OTLPReceiver
{
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: ControlMsgChannel,
        effect_handler: SendEffectHandler<OTLPRequest>,
    ) -> Result<(), Error<OTLPRequest>> {

        // create listener on addr provided from config
        let listener = effect_handler.tcp_listener(self.listening_addr)?;
        let mut listener_stream = TcpListenerStream::new(listener);

        // check for compression method
        let compression_encoding = match self.compression {
            Some(CompressionMethod::Gzip) => Some(CompressionEncoding::Gzip),
            Some(CompressionMethod::Zstd) => Some(CompressionEncoding::Zstd),
            Some(CompressionMethod::Deflate) => Some(CompressionEncoding::Deflate),
            _ => None,
        };


        //start event loop
        loop {

                    //create services for the grpc server and clone the effect handler to pass message
            let logs_service = LogsServiceImpl::new(effect_handler.clone());
            let metrics_service = MetricsServiceImpl::new(effect_handler.clone());
            let trace_service = TraceServiceImpl::new(effect_handler.clone());
            let profiles_service = ProfilesServiceImpl::new(effect_handler.clone());

            let logs_service_server;
            let metrics_service_server;
            let trace_service_server;
            let profiles_service_server;

            // check if a compression method was set
            if let Some(encoding) = compression_encoding {
                // define servicees with compression
                logs_service_server = LogsServiceServer::new(logs_service).send_compressed(encoding).accept_compressed(encoding);
                metrics_service_server = MetricsServiceServer::new(metrics_service).send_compressed(encoding).accept_compressed(encoding);
                trace_service_server = TraceServiceServer::new(trace_service).send_compressed(encoding).accept_compressed(encoding);
                profiles_service_server = ProfilesServiceServer::new(profiles_service).send_compressed(encoding).accept_compressed(encoding);
            } else {
                // define servicees without compression
                logs_service_server = LogsServiceServer::new(logs_service);
                metrics_service_server = MetricsServiceServer::new(metrics_service);
                trace_service_server = TraceServiceServer::new(trace_service);
                profiles_service_server = ProfilesServiceServer::new(profiles_service);
            }
            
            tokio::select! {
                biased; //prioritize ctrl_msg over all other blocks

                // Process internal event
                ctrl_msg = ctrl_msg_recv.recv() => {
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
                .add_service(profiles_service_server)
                .serve_with_incoming(&mut listener_stream)=> {
                    if let Err(e) = result {
                        // Report receiver error
                        return Err(Error::ReceiverError{receiver: effect_handler.receiver_name().to_string(), error: e.to_string()});
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
    use crate::proto::opentelemetry::collector::{
        logs::v1::{logs_service_client::LogsServiceClient, ExportLogsServiceRequest}, 
        metrics::v1::{metrics_service_client::MetricsServiceClient, ExportMetricsServiceRequest}, 
        trace::v1::{trace_service_client::TraceServiceClient, ExportTraceServiceRequest},
        profiles::v1development::{profiles_service_client::ProfilesServiceClient, ExportProfilesServiceRequest}};
    use otap_df_engine::error::Error;
    use otap_df_engine::message::ControlMsg;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use std::net::SocketAddr;
    use tokio::time::{Duration, timeout};
    use std::future::Future;
    use std::pin::Pin;

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

                let mut profiles_client = ProfilesServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Profile Service Client");
                let _profiles_response = profiles_client.export(ExportProfilesServiceRequest::default()).await.expect("Failed to receive response after sending Profile Request");

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(0), "Test")
                    .await
                    .expect("Failed to send Shutdown");

                // server should be down after shutdown
                let fail_metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone()).await;
                assert!(fail_metrics_client.is_err(), "Server did not shutdown");
            })
        }
    }



    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OTLPRequest>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler

                // read from the effect handler
                let metrics_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Assert that the message received is what the test client sent.
                let _expected_metrics_message = ExportMetricsServiceRequest::default();
                assert!(matches!(metrics_received, _expected_metrics_message));

                let logs_received = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");
                let _expected_logs_message = ExportLogsServiceRequest::default();
                assert!(matches!(logs_received, _expected_logs_message));


                let traces_received = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");

                let _expected_trace_message =  ExportTraceServiceRequest::default();
                assert!(matches!(traces_received, _expected_trace_message));

                let profiles_received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");
                let _expected_profile_message = ExportProfilesServiceRequest::default();
                assert!(matches!(profiles_received, _expected_profile_message));

            })
        }
    }

    #[test]
    fn test_otlp_receiver() {
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
                compression: None,
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

