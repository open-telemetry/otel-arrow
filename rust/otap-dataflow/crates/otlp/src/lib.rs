// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTLP nodes (receiver, exporter, processor).

pub mod grpc;
use crate::grpc::{LogsServiceImpl, MetricsServiceImpl, TraceServiceImpl, OTLPRequest};
use grpc_stubs::proto::collector::logs::v1::logs_service_server::LogsServiceServer;
use grpc_stubs::proto::collector::metrics::v1::metrics_service_server::MetricsServiceServer;
use grpc_stubs::proto::collector::trace::v1::trace_service_server::TraceServiceServer;
use otap_df_engine::receiver::{EffectHandler, Receiver, ControlMsgChannel};
use otap_df_engine::error::Error;
use otap_df_engine::message::ControlMsg;
use async_trait::async_trait;
use std::net::SocketAddr;
use tokio::time::{Duration, sleep};
use tonic::codec::CompressionEncoding;
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
                ctrl_msg = ctrl_msg_recv.recv() => {
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
                            ()
                        }
                    }
                }
                // handle grpc server events
                grpc_server = Server::builder()
                .add_service(LogsServiceServer::new(logs_service).send_compressed(CompressionEncoding::Zstd).accept_compressed(CompressionEncoding::Zstd))
                .add_service(MetricsServiceServer::new(metrics_service).send_compressed(CompressionEncoding::Zstd).accept_compressed(CompressionEncoding::Zstd))
                .add_service(TraceServiceServer::new(trace_service).send_compressed(CompressionEncoding::Zstd).accept_compressed(CompressionEncoding::Zstd))
                .serve(listener) => {
                    grpc_server.await?;
                }
                // A timeout branch in case no events occur.
                () = sleep(Duration::from_secs(1)) => {
                    // wait for next event
                }
            }
        }
        //Exit event loop
        Ok(())
    }

}
