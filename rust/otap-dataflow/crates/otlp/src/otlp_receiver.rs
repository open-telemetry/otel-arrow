// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

use crate::grpc::{LogsServiceImpl, MetricsServiceImpl, TraceServiceImpl, OTLPRequest};
use crate::grpc::grpc_stubs::proto::{collector::{logs::v1::logs_service_server::LogsServiceServer,
    metrics::v1::metrics_service_server::MetricsServiceServer,
    trace::v1::trace_service_server::TraceServiceServer}};
use otap_df_engine::receiver::{EffectHandler, Receiver, ControlMsgChannel};
use otap_df_engine::error::Error;
use otap_df_engine::message::ControlMsg;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::codec::CompressionEncoding;
use tonic::transport::Server;

struct OTLPReceiver {
    listening_addr: SocketAddr,
}

#[async_trait(?Send)]
impl Receiver for OTLPReceiver {
    type PData = OTLPRequest;
    async fn start(
        self: Box<Self>,
        ctrl_msg_recv: ControlMsgChannel,
        effect_handler: EffectHandler<Self::PData>,
    ) -> Result<(), Error<Self::PData>> {

        // create listener on addr provided from config
        let listener = effect_handler.tcp_listener(self.listening_addr)?;
        let listener_stream = TcpListenerStream::new(listener);

        let addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let listener = effect_handler.tcp_listener(addr)?;
        let local_addr = listener.local_addr().unwrap();

        // Notify the test of the actual bound address.
        let _ = self.port_notifier.send(local_addr);

        //create services for the grpc server and clone the effect handler to pass message
        let logs_service = LogsServiceImpl::new(effect_handler.clone());
        let metrics_service = MetricsServiceImpl::new(effect_handler.clone());
        let trace_service = TraceServiceImpl::new(effect_handler.clone());

   

        //start event loop
        loop {
            tokio::select! {
                // Process an internal event.
                mut ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(ControlMsg::Shutdown {reason}) => {
                            // break event loop
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
                // poll the server
                result = Server::builder()
                .add_service(LogsServiceServer::new(logs_service).send_compressed(CompressionEncoding::Zstd).accept_compressed(CompressionEncoding::Zstd))
                .add_service(MetricsServiceServer::new(metrics_service).send_compressed(CompressionEncoding::Zstd).accept_compressed(CompressionEncoding::Zstd))
                .add_service(TraceServiceServer::new(trace_service).send_compressed(CompressionEncoding::Zstd).accept_compressed(CompressionEncoding::Zstd))
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
    use otap_df_engine::receiver::{EffectHandler, Receiver, ControlMsgChannel};
    use otap_df_engine::error::Error;
    use otap_df_engine::message::ControlMsg;
    use otap_df_engine::testing::receiver::ReceiverTestRuntime;
    use otap_df_engine::testing::{CtrMsgCounters, TestMsg};
    use crate::grpc::grpc_stubs::proto::{collector::{logs::v1::logs_service_client::LogsServiceClient,
        metrics::v1::metrics_service_client::MetricsServiceClient,
        trace::v1::trace_service_client::TraceServiceClient}};
    use std::net::SocketAddr;
    use tokio::time::{Duration, sleep};
    use crate::otlp_receiver::OTLPReceiver;
    use serde_json::Value;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::sync::oneshot;
    use tokio::time::{Duration, sleep};

    #[test]
    fn test_receiver() {
        let mut test_runtime = ReceiverTestRuntime::new(10);

        // Create a oneshot channel to receive the listening address from MyReceiver.
        let (port_tx, port_rx) = oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr =
        format!("{grpc_addr}:{grpc_port}")
            .parse()
            .map_err(|e: AddrParseError| Error::OtlpError {
                error: e.to_string(),
            })?;

        let receiver = OTLPReceiver {
            listening_addr: addr 
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