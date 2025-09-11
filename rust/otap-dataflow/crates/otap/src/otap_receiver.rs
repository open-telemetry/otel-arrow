// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP receiver node
//!
//! ToDo: implement Ack and Nack control message, wait for receiver node to receive a Ack control message then the service can send a response back
//! ToDo: implement config control message to handle live changing configuration
//! ToDo: Add HTTP support
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//!

use crate::OTAP_RECEIVER_FACTORIES;
use crate::compression::CompressionMethod;
use crate::otap_grpc::middleware::zstd_header::ZstdRequestHeaderAdapter;
use crate::otap_grpc::{ArrowLogsServiceImpl, ArrowMetricsServiceImpl, ArrowTracesServiceImpl};
use crate::pdata::OtapPdata;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::shared::receiver as shared;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
    arrow_logs_service_server::ArrowLogsServiceServer,
    arrow_metrics_service_server::ArrowMetricsServiceServer,
    arrow_traces_service_server::ArrowTracesServiceServer,
};
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_middleware::MiddlewareLayer;

const OTAP_RECEIVER_URN: &str = "urn:otel:otap:receiver";

/// Configuration for the OTAP Receiver
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    listening_addr: SocketAddr,
    compression_method: Option<CompressionMethod>,
    message_size: usize,
}

/// A Receiver that listens for OTAP messages
pub struct OTAPReceiver {
    config: Config,
}

/// Declares the OTAP receiver as a shared receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTAP_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTAP_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
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
            config: Config {
                listening_addr,
                compression_method,
                message_size,
            },
        }
    }

    /// Creates a new OTAPReceiver from a configuration object
    pub fn from_config(
        _pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        Ok(OTAPReceiver { config })
    }
}

// Use the async_trait due to the need for thread safety because of tonic requiring Send and Sync traits
// The Shared version of the receiver allows us to implement a Receiver that requires the effect handler to be Send and Sync
//
#[async_trait]
impl shared::Receiver<OtapPdata> for OTAPReceiver {
    async fn start(
        self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel<OtapPdata>,
        effect_handler: shared::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        // create listener on addr provided from config
        let listener = effect_handler.tcp_listener(self.config.listening_addr)?;
        let listener_stream = TcpListenerStream::new(listener);

        //create services for the grpc server and clone the effect handler to pass message
        let logs_service =
            ArrowLogsServiceImpl::new(effect_handler.clone(), self.config.message_size);
        let metrics_service =
            ArrowMetricsServiceImpl::new(effect_handler.clone(), self.config.message_size);
        let trace_service =
            ArrowTracesServiceImpl::new(effect_handler.clone(), self.config.message_size);

        let mut logs_service_server = ArrowLogsServiceServer::new(logs_service);
        let mut metrics_service_server = ArrowMetricsServiceServer::new(metrics_service);
        let mut trace_service_server = ArrowTracesServiceServer::new(trace_service);

        // apply the tonic compression if it is set
        if let Some(ref compression) = self.config.compression_method {
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

        let server = Server::builder()
            .layer(MiddlewareLayer::new(ZstdRequestHeaderAdapter::default()))
            .add_service(logs_service_server)
            .add_service(metrics_service_server)
            .add_service(trace_service_server);

        tokio::select! {
            biased; //prioritize ctrl_msg over all other blocks

            // Process internal events
            ctrl_msg_result = async {
                loop {
                    match ctrl_msg_recv.recv().await {
                        Ok(NodeControlMsg::Shutdown {..}) => {
                            // ToDo: add proper deadline function
                            return Ok(());
                        },
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message do nothing
                        }
                    }
                }
            } => {
                return ctrl_msg_result;
            },

            // Run server
            result = server.serve_with_incoming(listener_stream) => {
                if let Err(error) = result {
                    // Report receiver error
                    return Err(Error::ReceiverError{receiver: effect_handler.receiver_id(), error: error.to_string()});
                }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::otap_mock::create_otap_batch;
    use crate::otap_receiver::{OTAP_RECEIVER_URN, OTAPReceiver};
    use crate::pdata::OtapPdata;
    use async_stream::stream;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otel_arrow_rust::Producer;
    use otel_arrow_rust::otap::OtapArrowRecords;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, arrow_logs_service_client::ArrowLogsServiceClient,
        arrow_metrics_service_client::ArrowMetricsServiceClient,
        arrow_traces_service_client::ArrowTracesServiceClient,
    };
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::sync::Arc;
    use tokio::time::{Duration, timeout};

    /// Test closure that simulates a typical receiver scenario.
    fn scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
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
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut metrics_records = create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                        let bar = producer.produce_bar(&mut metrics_records).unwrap();
                        yield bar
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
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut logs_records = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                        let bar = producer.produce_bar(&mut logs_records).unwrap();
                        yield bar;
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
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut traces_records = create_otap_batch(batch_id, ArrowPayloadType::Spans);
                        let bar = producer.produce_bar(&mut traces_records).unwrap();
                        yield bar;
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
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler

                // read from the effect handler
                for batch_id in 0..3 {
                    let metrics_received: OtapArrowRecords =
                        timeout(Duration::from_secs(3), ctx.recv())
                            .await
                            .expect("Timed out waiting for message")
                            .expect("No message received")
                            .payload()
                            .try_into()
                            .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_metrics_message =
                        create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                    assert!(matches!(metrics_received, _expected_metrics_message));
                }

                for batch_id in 0..3 {
                    let logs_received: OtapArrowRecords =
                        timeout(Duration::from_secs(3), ctx.recv())
                            .await
                            .expect("Timed out waiting for message")
                            .expect("No message received")
                            .payload()
                            .try_into()
                            .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_logs_message =
                        create_otap_batch(batch_id, ArrowPayloadType::Logs);
                    assert!(matches!(logs_received, _expected_logs_message));
                }

                for batch_id in 0..3 {
                    let traces_received: OtapArrowRecords =
                        timeout(Duration::from_secs(3), ctx.recv())
                            .await
                            .expect("Timed out waiting for message")
                            .expect("No message received")
                            .payload()
                            .try_into()
                            .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_traces_message =
                        create_otap_batch(batch_id, ArrowPayloadType::Spans);
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
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));
        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::new(addr, None, message_size),
            test_node(test_runtime.config().name.clone()),
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
