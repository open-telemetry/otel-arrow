// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).
//! 
//! TODO: implement Ack control message, wait for receiver node to receive a Ack control message then the service can send a response back 
//! TODO: implement config control message to handle live changing configuration

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
        let listener = effect_handler.tcp_listener(self.listening_addr)?;
        let mut listener_stream = TcpListenerStream::new(listener);
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
            
            tokio::select! {
                biased; //prioritize ctrl_msg over all other blocks

                // Process internal event
                mut ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(ControlMsg::Shutdown {reason, deadline}) => {
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
                .serve_with_incoming(&mut listener_stream)=> {
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
    use otap_df_engine::error::Error;
    use otap_df_engine::message::ControlMsg;
    use otap_df_engine::testing::receiver::ReceiverTestRuntime;
    use otap_df_engine::testing::{CtrMsgCounters, TestMsg};
    use crate::grpc::grpc_stubs::proto::collector::{logs::v1::logs_service_client::LogsServiceClient,
        metrics::v1::metrics_service_client::MetricsServiceClient,
        trace::v1::trace_service_client::TraceServiceClient};
    use std::net::SocketAddr;
    use tokio::time::{Duration, sleep};
    use crate::otlp_receiver::OTLPReceiver;
    use serde_json::Value;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::oneshot;

    #[test]
    fn test_receiver() {
        let mut test_runtime = ReceiverTestRuntime::new(10);

        // Create a oneshot channel to receive the listening address from MyReceiver.
        let (port_tx, port_rx) = oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let receiver = OTLPReceiver {
            listening_addr: addr,
            compression: None
        };



        //start grpc client 

        test_runtime.start_receiver(receiver);
        test_runtime.start_test(|ctx| async move {
            // Wait for the receiver to send the listening address.
            let addr: SocketAddr = port_rx.await.expect("Failed to receive listening address");
            let grpc_endpoint_clone = grpc_endpoint.clone();
            // Connect to the receiver's socket.
            //send test otlp data here

            let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint_clone.clone()).await.unwrap();

            let metrics_response = metrics_client.export(ExportMetricsServiceRequest::default()).await;

            let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                .await
                .unwrap();
            let logs_response = logs_client.export(ExportLogsServiceRequest::default()).await;

            let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone())
                .await
                .unwrap();
            let traces_response = traces_client.export(ExportTraceServiceRequest::default()).await;

            // Finally, send a Shutdown event to terminate the receiver.
            ctx.send_shutdown("Test")
                .await
                .expect("Failed to send Shutdown");

            
        });
        let counters = test_runtime.counters();
        test_runtime.validate(|mut ctx| async move {
            counters.assert(0, 0, 0, 1);
            let pdata_rx = ctx.pdata_rx().expect("No pdata_rx");
            let metrics_received = tokio::time::timeout(Duration::from_secs(3), pdata_rx.recv())
                .await
                .expect("Timed out waiting for message")
                .expect("No message received");

            // Assert that the message received is what the test client sent.
            assert!(matches!(metrics_received, ExportMetricsServiceRequest::default()));

            let logs_received = tokio::time::timeout(Duration::from_secs(3), pdata_rx.recv())
            .await
            .expect("Timed out waiting for message")
            .expect("No message received");
            assert!(matches!(logs_received, ExportLogsServiceRequest::default()));

            let traces_received = tokio::time::timeout(Duration::from_secs(3), pdata_rx.recv())
            .await
            .expect("Timed out waiting for message")
            .expect("No message received");
            assert!(matches!(traces_received, ExportTracesServiceRequest::default()));
            

        });
    }


}

#[cfg(test)]
mod tests {
    use super::{
        ControlMsgChannel, EffectHandlerTrait, NotSendEffectHandler, ReceiverWrapper,
        SendEffectHandler,
    };
    use crate::receiver::{Error, Receiver};
    use crate::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use crate::testing::{CtrlMsgCounters, TestMsg, exec_in_send_env};
    use async_trait::async_trait;
    use serde_json::Value;
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, sleep, timeout};


    /// A type alias for a test receiver with sendable effect handler
    type ReceiverWithSendEffectHandler = GenericTestReceiver<SendEffectHandler<TestMsg>>;

    /// Test closure that simulates a typical receiver scenario.
    fn scenario(
        port_rx: oneshot::Receiver<SocketAddr>,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // Wait for the receiver to send the listening address.
                let addr: SocketAddr = port_rx.await.expect("Failed to receive listening address");

                // Connect to the receiver's socket.
                let mut stream = TcpStream::connect(addr)
                    .await
                    .expect("Failed to connect to receiver");

                // Send some test data.
                stream
                    .write_all(b"Hello from test client")
                    .await
                    .expect("Failed to send data");

                // Optionally, read an echo (acknowledgment) from the receiver.
                let mut buf = [0u8; 1024];
                let len = stream
                    .read(&mut buf)
                    .await
                    .expect("Failed to read response");
                assert_eq!(&buf[..len], b"ack", "Expected acknowledgment from receiver");

                // Send a few TimerTick events from the test.
                for _ in 0..3 {
                    ctx.send_timer_tick()
                        .await
                        .expect("Failed to send TimerTick");
                    ctx.sleep(Duration::from_millis(100)).await;
                }

                ctx.send_config(Value::Null)
                    .await
                    .expect("Failed to send config");

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(200), "Test")
                    .await
                    .expect("Failed to send Shutdown");

                // Close the TCP connection.
                let _ = stream.shutdown().await;
            })
        }
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<TestMsg>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let received = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received");

                // Assert that the message received is what the test client sent.
                assert!(matches!(received, TestMsg(msg) if msg == "Hello from test client"));
                ctx.counters().assert(3, 0, 1, 1);
            })
        }
    }


    #[test]
    fn test_receiver_with_send_effect_handler() {
        let test_runtime = TestRuntime::new();

        // Create a oneshot channel to receive the listening address from the receiver.
        let (port_tx, port_rx) = oneshot::channel();
        let receiver = ReceiverWrapper::with_send(
            ReceiverWithSendEffectHandler::with_send_effect_handler(
                test_runtime.counters(),
                |effect_handler| {
                    exec_in_send_env(|| {
                        _ = effect_handler.receiver_name();
                    });
                },
                port_tx,
            ),
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(port_rx))
            .run_validation(validation_procedure());
    }
}