// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

pub mod grpc;
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
// use tonic::codec::CompressionEncoding;
use tonic::transport::Server;

struct OTLPReceiver {
    listening_addr: SocketAddr,
}

#[async_trait(?Send)]
impl Receiver for OTLPReceiver {
    type Msg = OTLPRequest;
    async fn start(
        self: Box<Self>,
        ctrl_msg_recv: ControlMsgChannel,
        effect_handler: EffectHandler<Self::Msg>,
    ) -> Result<(), Error<Self::Msg>> {

        // create listener on addr provided from config
        let listener = effect_handler.tcp_listener(self.listening_addr)?;
        let listener_stream = TcpListenerStream::new(listener);

        //create services for the grpc server and clone the effect handler to pass message
        let logs_service = LogsServiceImpl {
            effect_handler: effect_handler.clone(),
        };
        let metrics_service = MetricsServiceImpl {
            effect_handler: effect_handler.clone(),
        };
        let trace_service = TraceServiceImpl {
            effect_handler: effect_handler.clone(),
        };


        //start event loop
        loop {
            tokio::select! {
                // Process an internal event.
                // ctrl_msg = ctrl_msg_recv.recv() => {
                //     match ctrl_msg {
                //         Ok(ControlMsg::Shutdown {reason}) => {
                //             // break event loop
                //             break;
                //         },
                //         Err(e) => {
                //             return Err(Error::ChannelRecvError(e));
                //         }
                //         _ => {
                //             // unknown control message do nothing
                            
                //         }
                //     }
                // }
                // start the grpc server
                server = Server::builder()
                .add_service(LogsServiceServer::new(logs_service))
                .add_service(MetricsServiceServer::new(metrics_service))
                .add_service(TraceServiceServer::new(trace_service))
                .serve_with_incoming(listener_stream.clone()) => {
                    if let Err(e) = server {
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


// #[cfg(test)]
// mod tests {
//     use crate::message::ControlMsg;
//     use crate::receiver::{EffectHandler, Error, Receiver};
//     use async_trait::async_trait;
//     use otap_df_channel::mpsc;
//     use std::net::SocketAddr;
//     use tokio::io::{AsyncReadExt, AsyncWriteExt};
//     use tokio::net::TcpStream;
//     use tokio::runtime::Builder;
//     use tokio::sync::oneshot;
//     use tokio::task::LocalSet;
//     use tokio::time::{Duration, sleep};

//     use super::ControlMsgChannel;


//     #[test]
//     fn test_receiver() {
//         let rt = Builder::new_current_thread().enable_all().build().unwrap();
//         let local_tasks = LocalSet::new();
//         // Create a oneshot channel to receive the listening address from MyReceiver.
//         let (port_tx, port_rx) = oneshot::channel();

//         // Create an MPSC channel for internal events.
//         let (event_tx, event_rx) = mpsc::Channel::new(10);
//         let event_receiver = ControlMsgChannel { rx: event_rx };

//         // Create an MPSC channel for messages from the effect handler.
//         let (msg_tx, msg_rx) = mpsc::Channel::new(10);
//         let addr = 
//         let receiver = Box::new(OTLPReceiver {
//             listening_addr: addr,
//         });

//         // Spawn the receiver's event loop.
//         _ = local_tasks.spawn_local(async move {
//             receiver
//                 .start(event_receiver, EffectHandler::new("receiver", msg_tx))
//                 .await
//                 .expect("Should not happen");
//         });

//         // Spawn a task to simulate client activity and send events.
//         _ = local_tasks.spawn_local(async move {
//             // Wait for the receiver to send the listening address.
//             let addr: SocketAddr = port_rx.await.expect("Failed to receive listening address");
//             println!("Test received listening address: {addr}");

//             // Connect to the receiver's socket.
//             let mut stream = TcpStream::connect(addr)
//                 .await
//                 .expect("Failed to connect to receiver");

//             // Send some test data.
//             stream
//                 .write_all(b"Hello from test client")
//                 .await
//                 .expect("Failed to send data");

//             // Optionally, read an echo (acknowledgment) from the receiver.
//             let mut buf = [0u8; 1024];
//             let n = stream
//                 .read(&mut buf)
//                 .await
//                 .expect("Failed to read response");
//             println!(
//                 "Test client received response: {}",
//                 String::from_utf8_lossy(&buf[..n])
//             );

//             // Send a few TimerTick events from the test.
//             for _ in 0..3 {
//                 let result = event_tx.send_async(ControlMsg::TimerTick {}).await;
//                 assert!(result.is_ok(), "Failed to send TimerTick event");
//                 sleep(Duration::from_millis(100)).await;
//             }

//             // Finally, send a Shutdown event to terminate the receiver.
//             let result = event_tx
//                 .send_async(ControlMsg::Shutdown {
//                     reason: "Test".to_string(),
//                 })
//                 .await;
//             assert!(result.is_ok(), "Failed to send Shutdown event");

//             // Close the TCP connection.
//             let _ = stream.shutdown().await;
//         });

//         rt.block_on(local_tasks);

//         // After the tasks complete, check that a message was sent by the receiver.
//         let received = rt
//             .block_on(async { tokio::time::timeout(Duration::from_secs(3), msg_rx.recv()).await })
//             .expect("Timed out waiting for message")
//             .expect("No message received");

//         // Assert that the message received is what the test client sent.
//         assert!(matches!(received, TestMsg(msg) if msg == "Hello from test client"));
//     }
// }
