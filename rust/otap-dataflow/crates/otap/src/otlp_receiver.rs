// SPDX-License-Identifier: Apache-2.0
// Copyright The OpenTelemetry Authors

use crate::OTAP_RECEIVER_FACTORIES;
use crate::grpc::otlp::server::{LogsServiceServer, MetricsServiceServer, TraceServiceServer};
use crate::pdata::OtapPdata;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::error::Error;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::shared::receiver as shared;
use otap_df_otlp::compression::CompressionMethod;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use otap_df_telemetry::registry::MetricsKey;
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use otap_df_engine::context::PipelineContext;

/// URN for the OTLP Receiver
pub const OTLP_RECEIVER_URN: &str = "urn::otel::otlp::receiver";

/// Configuration for OTLP Receiver
#[derive(Debug, Deserialize)]
pub struct Config {
    listening_addr: SocketAddr,
    compression_method: Option<CompressionMethod>,
}

/// Receiver implementation that receives OTLP grpc service requests and decodes the data into OTAP.
pub struct OTLPReceiver {
    config: Config,
    metrics: MetricSet<OtlpReceiverMetrics>,
}

/// Declares the OTLP receiver as a shared receiver factory
///
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTLP_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTLP_RECEIVER_URN,
    create: |pipeline: PipelineContext, node_config: Arc<NodeUserConfig>, receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::shared(
            OTLPReceiver::from_config(pipeline, &node_config.config)?,
            node_config,
            receiver_config,
        ))
    },
};

impl OTLPReceiver {
    /// Creates a new OTLPReceiver from a configuration object
    pub fn from_config(pipeline_ctx: PipelineContext, config: &Value) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // Register OTLP receiver metrics for this node.
        let metrics = pipeline_ctx.register_metrics::<OtlpReceiverMetrics>();

        Ok(OTLPReceiver { config, metrics })
    }
}

/// OTLP receiver metrics moved into the node module.
#[metric_set(name = "otlp.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct OtlpReceiverMetrics {
    key: Option<MetricsKey>,
    /// Number of bytes received.
    #[metric(name = "bytes.received", unit = "By")]
    pub bytes_received: Counter<u64>,
    /// Number of messages received.
    #[metric(name = "messages.received", unit = "{msg}")]
    pub messages_received: Counter<u64>,
}

#[async_trait]
impl shared::Receiver<OtapPdata> for OTLPReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel,
        effect_handler: shared::EffectHandler<OtapPdata>,
    ) -> Result<(), Error<OtapPdata>> {
        // Make the receiver mutable so we can update metrics on telemetry collection.
    let listener = effect_handler.tcp_listener(self.config.listening_addr)?;
        let mut listener_stream = TcpListenerStream::new(listener);

        loop {
            let mut logs_service_server = LogsServiceServer::new(effect_handler.clone());
            let mut metrics_service_server = MetricsServiceServer::new(effect_handler.clone());
            let mut trace_service_server = TraceServiceServer::new(effect_handler.clone());

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

            tokio::select! {
                biased;

                // Process internal event
                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::Shutdown {..}) => {
                            break;
                        },
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            // Report current receiver metrics.
                            _ = metrics_reporter.report(&mut self.metrics).await;
                        },
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // unknown control message do nothing
                        }
                    }
                }
                result = Server::builder()
                    .add_service(logs_service_server)
                    .add_service(metrics_service_server)
                    .add_service(trace_service_server)
                    .serve_with_incoming(&mut listener_stream) => {
                        if let Err(error) = result {
                            return Err(Error::ReceiverError {
                                receiver: effect_handler.receiver_id(),
                                error: error.to_string()
                            })
                        }
                    }
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::pdata::OtlpProtoBytes;

    use super::*;

    use std::pin::Pin;
    use std::time::Duration;

    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::receiver::{NotSendValidateContext, TestContext, TestRuntime};
    use otap_df_otlp::proto::opentelemetry::collector::logs::v1::logs_service_client::LogsServiceClient;
    use otap_df_otlp::proto::opentelemetry::collector::logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse,
    };
    use otap_df_otlp::proto::opentelemetry::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
    use otap_df_otlp::proto::opentelemetry::collector::metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
    };
    use otap_df_otlp::proto::opentelemetry::collector::trace::v1::trace_service_client::TraceServiceClient;
    use otap_df_otlp::proto::opentelemetry::collector::trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse,
    };
    use otap_df_otlp::proto::opentelemetry::common::v1::{InstrumentationScope, KeyValue};
    use otap_df_otlp::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otap_df_otlp::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};
    use otap_df_otlp::proto::opentelemetry::resource::v1::Resource;
    use otap_df_otlp::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans};
    use prost::Message;
    use tokio::time::timeout;

    fn create_logs_service_request() -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                resource: Some(Resource {
                    attributes: vec![KeyValue {
                        key: "a".to_string(),
                        ..Default::default()
                    }],
                    ..Default::default()
                }),
                scope_logs: vec![ScopeLogs {
                    scope: Some(InstrumentationScope {
                        attributes: vec![KeyValue {
                            key: "b".to_string(),
                            ..Default::default()
                        }],
                        ..Default::default()
                    }),
                    log_records: vec![
                        LogRecord {
                            time_unix_nano: 1,
                            attributes: vec![KeyValue {
                                key: "c".to_string(),
                                ..Default::default()
                            }],
                            ..Default::default()
                        },
                        LogRecord {
                            time_unix_nano: 2,
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn create_metrics_service_request() -> ExportMetricsServiceRequest {
        ExportMetricsServiceRequest {
            resource_metrics: vec![ResourceMetrics {
                resource: Some(Resource {
                    ..Default::default()
                }),
                scope_metrics: vec![ScopeMetrics {
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    fn create_traces_service_request() -> ExportTraceServiceRequest {
        ExportTraceServiceRequest {
            resource_spans: vec![ResourceSpans {
                resource: None,
                scope_spans: vec![
                    ScopeSpans {
                        ..Default::default()
                    },
                    ScopeSpans {
                        ..Default::default()
                    },
                ],
                schema_url: "opentelemetry.io/schema/traces".to_string(),
            }],
        }
    }

    fn scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Logs Service Client");

                let logs_response = logs_client
                    .export(create_logs_service_request())
                    .await
                    .expect("Can send log request")
                    .into_inner();
                assert_eq!(
                    logs_response,
                    ExportLogsServiceResponse {
                        partial_success: None
                    }
                );

                let mut metrics_client = MetricsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Metrics Service Client");
                let metrics_response = metrics_client
                    .export(create_metrics_service_request())
                    .await
                    .expect("can send metrics request")
                    .into_inner();
                assert_eq!(
                    metrics_response,
                    ExportMetricsServiceResponse {
                        partial_success: None
                    }
                );

                let mut traces_client = TraceServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server from Traces Service Client");
                let traces_response = traces_client
                    .export(create_traces_service_request())
                    .await
                    .expect("can send traces request")
                    .into_inner();
                assert_eq!(
                    traces_response,
                    ExportTraceServiceResponse {
                        partial_success: None
                    }
                );

                ctx.send_shutdown(Duration::from_millis(0), "Test")
                    .await
                    .expect("Failed to send Shutdown");

                let fail_client = LogsServiceClient::connect(grpc_endpoint.clone()).await;
                assert!(fail_client.is_err(), "Server did not shutdown");
            })
        }
    }

    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                let logs_pdata: OtlpProtoBytes = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");

                assert!(matches!(logs_pdata, OtlpProtoBytes::ExportLogsRequest(_)));

                let expected = create_logs_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, logs_pdata.as_bytes());

                let metrics_pdata: OtlpProtoBytes = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");
                assert!(matches!(
                    metrics_pdata,
                    OtlpProtoBytes::ExportMetricsRequest(_)
                ));

                let expected = create_metrics_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, metrics_pdata.as_bytes());

                let trace_pdata: OtlpProtoBytes = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for message")
                    .expect("No message received")
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");
                assert!(matches!(
                    trace_pdata,
                    OtlpProtoBytes::ExportTracesRequest(_)
                ));

                let expected = create_traces_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, trace_pdata.as_bytes());
            })
        }
    }

    #[test]
    fn test_otlp_receiver() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));
        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config: Config {
                    listening_addr: addr,
                    compression_method: None,
                },
                metrics: OtlpReceiverMetrics::default(),
            },
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint))
            .run_validation(validation_procedure());
    }
}
