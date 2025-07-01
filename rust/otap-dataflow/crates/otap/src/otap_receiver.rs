// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP receiver node
//!
//! ToDo: implement Ack and Nack control message, wait for receiver node to receive a Ack control message then the service can send a response back
//! ToDo: implement config control message to handle live changing configuration
//! ToDo: Add HTTP support
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//!

use crate::OTAP_RECEIVER_FACTORIES;
use crate::grpc::{
    ArrowLogsServiceImpl, ArrowMetricsServiceImpl, ArrowTracesServiceImpl, OTAPData,
};
use crate::proto::opentelemetry::experimental::arrow::v1::{
    arrow_logs_service_server::ArrowLogsServiceServer,
    arrow_metrics_service_server::ArrowMetricsServiceServer,
    arrow_traces_service_server::ArrowTracesServiceServer,
};
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::control::ControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::shared::receiver as shared;
use otap_df_otlp::compression::CompressionMethod;
use serde_json::Value;
use std::net::SocketAddr;
use std::rc::Rc;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use otap_df_config::node::NodeUserConfig;

const OTAP_RECEIVER_URN: &str = "urn:otel:otap:receiver";

/// A Receiver that listens for OTAP messages
pub struct OTAPReceiver {
    listening_addr: SocketAddr,
    compression_method: Option<CompressionMethod>,
    message_size: usize,
}

/// Declares the OTAP exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTAP_RECEIVER: ReceiverFactory<OTAPData> = ReceiverFactory {
    name: OTAP_RECEIVER_URN,
    create: |node_config: Rc<NodeUserConfig>, receiver_config: &ReceiverConfig| {
        ReceiverWrapper::shared(OTAPReceiver::from_config(&node_config.config), node_config, receiver_config)
    },
};

impl OTAPReceiver {
    /// creates a new OTAP Receiver
    #[must_use]
    pub fn new(
        listening_addr: SocketAddr,
        compression_method: Option<CompressionMethod>,
        message_size: usize,
    ) -> Self {
        OTAPReceiver {
            listening_addr,
            compression_method,
            message_size,
        }
    }

    /// Creates a new OTAPReceiver from a configuration object
    #[must_use]
    pub fn from_config(_config: &Value) -> Self {
        // ToDo: implement config parsing
        OTAPReceiver {
            listening_addr: "127.0.0.1:4317".parse().expect("Invalid socket address"),
            compression_method: None,
            message_size: 100, // default message size
        }
    }
}

// Use the async_trait due to the need for thread safety because of tonic requiring Send and Sync traits
// The Shared version of the receiver allows us to implement a Receiver that requires the effect handler to be Send and Sync
//
#[async_trait]
impl shared::Receiver<OTAPData> for OTAPReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel,
        effect_handler: shared::EffectHandler<OTAPData>,
    ) -> Result<(), Error<OTAPData>> {
        // create listener on addr provided from config
        let listener = effect_handler.tcp_listener(self.listening_addr)?;
        let mut listener_stream = TcpListenerStream::new(listener);

        //start event loop
        loop {
            //create services for the grpc server and clone the effect handler to pass message
            let logs_service = ArrowLogsServiceImpl::new(effect_handler.clone(), self.message_size);
            let metrics_service =
                ArrowMetricsServiceImpl::new(effect_handler.clone(), self.message_size);
            let trace_service =
                ArrowTracesServiceImpl::new(effect_handler.clone(), self.message_size);

            let mut logs_service_server = ArrowLogsServiceServer::new(logs_service);
            let mut metrics_service_server = ArrowMetricsServiceServer::new(metrics_service);
            let mut trace_service_server = ArrowTracesServiceServer::new(trace_service);

            // apply the tonic compression if it is set
            if let Some(ref compression) = self.compression_method {
                let encoding = compression.map_to_compression_encoding();

                logs_service_server = logs_service_server
                    .send_compressed(encoding)
                    .accept_compressed(encoding);
                metrics_service_server = metrics_service_server
                    .send_compressed(encoding)
                    .accept_compressed(encoding);
                trace_service_server = trace_service_server
                    .send_compressed(encoding)
                    .accept_compressed(encoding);
            }

            tokio::select! {
                biased; //prioritize ctrl_msg over all other blocks
                // Process internal event
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(ControlMsg::Shutdown {..}) => {
                            // ToDo: add proper deadline function
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
                    if let Err(error) = result {
                        // Report receiver error
                        return Err(Error::ReceiverError{receiver: effect_handler.receiver_name(), error: error.to_string()});
                    }
                }
            }
        }
        //Exit event loop
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::grpc::OTAPData;
    use crate::mock::create_batch_arrow_record;
    use crate::otap_receiver::{OTAPReceiver, OTAP_RECEIVER_URN};
    use crate::proto::opentelemetry::experimental::arrow::v1::{
        ArrowPayloadType, arrow_logs_service_client::ArrowLogsServiceClient,
        arrow_metrics_service_client::ArrowMetricsServiceClient,
        arrow_traces_service_client::ArrowTracesServiceClient,
    };
    use async_stream::stream;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::rc::Rc;
    use tokio::time::{Duration, timeout};
    use otap_df_config::node::NodeUserConfig;

    /// Test closure that simulates a typical receiver scenario.
    fn scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // send data to the receiver

                // connect to the different clients and call export to send a message
                // let mut grpc_endpoint_clone = grpc_endpoint.clone();
                let mut arrow_metrics_client =
                    ArrowMetricsServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("Failed to connect to server from Metrics Service Client");
                #[allow(tail_expr_drop_order)]
                let metrics_stream = stream! {
                    for batch_id in 0..3 {
                        let metrics_records = create_batch_arrow_record(batch_id, ArrowPayloadType::MultivariateMetrics);
                        yield metrics_records;
                    }
                };
                let _metrics_response = arrow_metrics_client
                    .arrow_metrics(metrics_stream)
                    .await
                    .expect("Failed to receive response after sending Metrics Request");

                let mut arrow_logs_client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Logs Service Client");
                #[allow(tail_expr_drop_order)]
                let logs_stream = stream! {
                    for batch_id in 0..3 {
                        let logs_records = create_batch_arrow_record(batch_id, ArrowPayloadType::Logs);
                        yield logs_records;
                    }
                };
                let _logs_response = arrow_logs_client
                    .arrow_logs(logs_stream)
                    .await
                    .expect("Failed to receive response after sending Logs Request");

                let mut arrow_traces_client =
                    ArrowTracesServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("Failed to connect to server from Trace Service Client");
                #[allow(tail_expr_drop_order)]
                let traces_stream = stream! {
                    for batch_id in 0..3 {
                        let traces_records = create_batch_arrow_record(batch_id, ArrowPayloadType::Spans);
                        yield traces_records;
                    }
                };
                let _traces_response = arrow_traces_client
                    .arrow_traces(traces_stream)
                    .await
                    .expect("Failed to receive response after sending Trace Request");

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Duration::from_millis(0), "Test")
                    .await
                    .expect("Failed to send Shutdown");

                // server should be down after shutdown
                let fail_metrics_client =
                    ArrowMetricsServiceClient::connect(grpc_endpoint.clone()).await;
                assert!(fail_metrics_client.is_err(), "Server did not shutdown");
            })
        }
    }

    /// Validation closure that checks the received message and counters (!Send context).
    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OTAPData>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler

                // read from the effect handler
                for batch_id in 0..3 {
                    let metrics_received = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received");

                    // Assert that the message received is what the test client sent.
                    let _expected_metrics_message =
                        create_batch_arrow_record(batch_id, ArrowPayloadType::MultivariateMetrics);
                    assert!(matches!(metrics_received, _expected_metrics_message));
                }

                for batch_id in 0..3 {
                    let logs_received = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received");

                    // Assert that the message received is what the test client sent.
                    let _expected_logs_message =
                        create_batch_arrow_record(batch_id, ArrowPayloadType::Logs);
                    assert!(matches!(logs_received, _expected_logs_message));
                }

                for batch_id in 0..3 {
                    let traces_received = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received");

                    // Assert that the message received is what the test client sent.
                    let _expected_traces_message =
                        create_batch_arrow_record(batch_id, ArrowPayloadType::Spans);
                    assert!(matches!(traces_received, _expected_traces_message));
                }
            })
        }
    }

    #[test]
    fn test_otap_receiver() {
        let test_runtime = TestRuntime::new();

        // addr and port for the server to run at
        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let message_size = 100;

        // create our receiver
        let node_config = Rc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));
        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::new(addr, None, message_size),
            node_config,
            test_runtime.config(),
        );

        // run the test
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint))
            .run_validation(validation_procedure());
    }
}
