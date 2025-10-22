// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::OTAP_RECEIVER_FACTORIES;
use crate::otap_grpc::otlp::server::{
    LogsServiceServer, MetricsServiceServer, RouteResponse, Settings, SharedState,
    TraceServiceServer,
};
use crate::pdata::OtapPdata;

use crate::compression::CompressionMethod;
use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::experimental::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ReceiverErrorKind, format_error_sources};
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::shared::receiver as shared;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio_stream::wrappers::TcpListenerStream;
use tonic::codec::EnabledCompressionEncodings;
use tonic::transport::Server;

/// URN for the OTLP Receiver
pub const OTLP_RECEIVER_URN: &str = "urn:otel:otlp:receiver";

/// Configuration for OTLP Receiver
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The endpoint details: protocol, name, port.
    listening_addr: SocketAddr,

    /// Compression methods accepted
    /// TODO: this should be (CompressionMethod, CompressionMethod), with separate settings
    /// (as tonic supports) for request and response compression.
    compression_method: Option<CompressionMethod>,

    /// Maximum number of concurrent (in-flight) requests (default: 1000)
    #[serde(default = "default_max_concurrent_requests")]
    max_concurrent_requests: usize,

    /// Whether to wait for the result (default: true)
    ///
    /// When enabled, the receiver will not send a response until the
    /// immediate downstream component has acknowledged receipt of the
    /// data.  This does not guarantee that data has been fully
    /// processed or successfully exported to the final destination,
    /// since components are able acknowledge early.
    ///
    /// Note when wait_for_result=false, it is impossible to
    /// see a failure, errors are effectively suppressed.
    #[serde(default = "default_wait_for_result")]
    wait_for_result: bool,
}

const fn default_max_concurrent_requests() -> usize {
    1000
}

const fn default_wait_for_result() -> bool {
    // See https://github.com/open-telemetry/otel-arrow/issues/1311
    // This matches the OTel Collector default for wait_for_result, presently.
    false
}

/// Receiver implementation that receives OTLP grpc service requests and decodes the data into OTAP.
pub struct OTLPReceiver {
    config: Config,
    metrics: MetricSet<OtlpReceiverMetrics>,
}

/// State shared between gRPC server task and the effect handler.
struct SharedStates {
    logs: Option<SharedState>,
    metrics: Option<SharedState>,
    traces: Option<SharedState>,
}

/// Declares the OTLP receiver as a shared receiver factory
///
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTLP_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTLP_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig| {
        Ok(ReceiverWrapper::shared(
            OTLPReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
};

impl OTLPReceiver {
    /// Creates a new OTLPReceiver from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // Register OTLP receiver metrics for this node.
        let metrics = pipeline_ctx.register_metrics::<OtlpReceiverMetrics>();

        Ok(Self { config, metrics })
    }

    fn route_ack_response(&self, states: &SharedStates, ack: AckMsg<OtapPdata>) -> RouteResponse {
        let calldata = ack.calldata;
        let resp = Ok(());
        match ack.accepted.signal_type() {
            SignalType::Logs => states
                .logs
                .as_ref()
                .map(|state| state.route_response(calldata, resp)),
            SignalType::Metrics => states
                .metrics
                .as_ref()
                .map(|state| state.route_response(calldata, resp)),
            SignalType::Traces => states
                .traces
                .as_ref()
                .map(|state| state.route_response(calldata, resp)),
        }
        .unwrap_or(RouteResponse::None)
    }

    fn route_nack_response(
        &self,
        states: &SharedStates,
        mut nack: NackMsg<OtapPdata>,
    ) -> RouteResponse {
        let calldata = std::mem::take(&mut nack.calldata);
        let sigtype = nack.refused.signal_type();
        let resp = Err(nack);
        match sigtype {
            SignalType::Logs => states
                .logs
                .as_ref()
                .map(|state| state.route_response(calldata, resp)),
            SignalType::Metrics => states
                .metrics
                .as_ref()
                .map(|state| state.route_response(calldata, resp)),
            SignalType::Traces => states
                .traces
                .as_ref()
                .map(|state| state.route_response(calldata, resp)),
        }
        .unwrap_or(RouteResponse::None)
    }

    fn handle_acknack_response(&mut self, resp: RouteResponse) {
        match resp {
            RouteResponse::Sent => self.metrics.acks_sent.inc(),
            RouteResponse::Expired => self.metrics.nacks_sent.inc(),
            RouteResponse::Invalid => self.metrics.acks_nacks_expired.inc(),
            RouteResponse::None => {}
        }
    }
}

/// OTLP receiver metrics.
//
// TODO: The following were unused, would have to be implemented in
// a different location:
//
// /// Number of bytes received.
// #[metric(unit = "By")]
// pub bytes_received: Counter<u64>,
// /// Number of messages received.
// #[metric(unit = "{msg}")]
// pub messages_received: Counter<u64>,
#[metric_set(name = "otlp.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct OtlpReceiverMetrics {
    /// Number of acks sent.
    #[metric(unit = "{acks}")]
    pub acks_sent: Counter<u64>,

    /// Number of nacks sent.
    #[metric(unit = "{nacks}")]
    pub nacks_sent: Counter<u64>,

    /// Number of invalid/expired acks/nacks.
    #[metric(unit = "{ack_or_nack}")]
    pub acks_nacks_expired: Counter<u64>,
}

#[async_trait]
impl shared::Receiver<OtapPdata> for OTLPReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel<OtapPdata>,
        effect_handler: shared::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // Make the receiver mutable so we can update metrics on telemetry collection.
        let listener = effect_handler.tcp_listener(self.config.listening_addr)?;
        let listener_stream = TcpListenerStream::new(listener);

        let mut compression = EnabledCompressionEncodings::default();
        let _ = self
            .config
            .compression_method
            .as_ref()
            .map(|method| compression.enable(method.map_to_compression_encoding()));

        let settings = Settings {
            max_concurrent_requests: self.config.max_concurrent_requests,
            wait_for_result: self.config.wait_for_result,
            accept_compression_encodings: compression,
            send_compression_encodings: compression,
        };

        let logs_server = LogsServiceServer::new(effect_handler.clone(), &settings);
        let metrics_server = MetricsServiceServer::new(effect_handler.clone(), &settings);
        let traces_server = TraceServiceServer::new(effect_handler.clone(), &settings);

        let states = SharedStates {
            logs: logs_server.common.state(),
            metrics: metrics_server.common.state(),
            traces: traces_server.common.state(),
        };

        let server = Server::builder()
            .add_service(logs_server)
            .add_service(metrics_server)
            .add_service(traces_server);

        // Start periodic telemetry collection
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        tokio::select! {
            biased;

            // Process internal events
            ctrl_msg_result = async {
                loop {
                    match ctrl_msg_recv.recv().await {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            let snapshot = self.metrics.snapshot();
                            _ = telemetry_cancel_handle.cancel().await;
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        },
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            // Report current receiver metrics.
                            _ = metrics_reporter.report(&mut self.metrics);
                        },
                        Ok(NodeControlMsg::Ack(ack)) => {
                            self.handle_acknack_response(self.route_ack_response(&states, ack));
                        },
                        Ok(NodeControlMsg::Nack(nack)) => {
                            self.handle_acknack_response(self.route_nack_response(&states, nack));
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
                    let source_detail = format_error_sources(&error);
                    return Err(Error::ReceiverError {
                        receiver: effect_handler.receiver_id(),
                        kind: ReceiverErrorKind::Transport,
                        error: error.to_string(),
                        source_detail,
                    });
                }
            }
        }

        Ok(TerminalState::new(
            Instant::now().add(Duration::from_secs(1)),
            [self.metrics],
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::pdata::OtlpProtoBytes;
    use crate::proto::opentelemetry::collector::logs::v1::logs_service_client::LogsServiceClient;
    use crate::proto::opentelemetry::collector::logs::v1::{
        ExportLogsServiceRequest, ExportLogsServiceResponse,
    };
    use crate::proto::opentelemetry::collector::metrics::v1::metrics_service_client::MetricsServiceClient;
    use crate::proto::opentelemetry::collector::metrics::v1::{
        ExportMetricsServiceRequest, ExportMetricsServiceResponse,
    };
    use crate::proto::opentelemetry::collector::trace::v1::trace_service_client::TraceServiceClient;
    use crate::proto::opentelemetry::collector::trace::v1::{
        ExportTraceServiceRequest, ExportTraceServiceResponse,
    };
    use crate::proto::opentelemetry::common::v1::{InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use crate::proto::opentelemetry::metrics::v1::{ResourceMetrics, ScopeMetrics};
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::NackMsg;
    use otap_df_engine::control::{AckMsg, NodeControlMsg};
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use prost::Message;
    use std::pin::Pin;
    use std::time::{Duration, Instant};
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

    #[test]
    fn test_config_parsing() {
        use serde_json::json;

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let config_with_max_concurrent_requests = json!({
            "listening_addr": "127.0.0.1:4317",
            "max_concurrent_requests": 5000
        });
        let receiver =
            OTLPReceiver::from_config(pipeline_ctx.clone(), &config_with_max_concurrent_requests)
                .unwrap();
        assert_eq!(receiver.config.max_concurrent_requests, 5000);

        let config_default = json!({
            "listening_addr": "127.0.0.1:4317"
        });
        let receiver = OTLPReceiver::from_config(pipeline_ctx.clone(), &config_default).unwrap();
        assert_eq!(receiver.config.max_concurrent_requests, 1000);

        let config_full = json!({
            "listening_addr": "127.0.0.1:4317",
            "compression_method": "gzip",
            "max_concurrent_requests": 2500
        });
        let receiver = OTLPReceiver::from_config(pipeline_ctx, &config_full).unwrap();
        assert_eq!(receiver.config.max_concurrent_requests, 2500);
    }

    fn scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
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

                ctx.send_shutdown(Instant::now(), "Test")
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
                // Receive logs pdata
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for logs message")
                    .expect("No logs message received");

                // Validate logs payload
                let logs_proto: OtlpProtoBytes = logs_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");
                assert!(matches!(logs_proto, OtlpProtoBytes::ExportLogsRequest(_)));

                let expected = create_logs_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, logs_proto.as_bytes());

                // Send Ack back to unblock the gRPC handler
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack for logs");
                }

                // Receive metrics pdata
                let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for metrics message")
                    .expect("No metrics message received");

                // Validate metrics payload
                let metrics_proto: OtlpProtoBytes = metrics_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");
                assert!(matches!(
                    metrics_proto,
                    OtlpProtoBytes::ExportMetricsRequest(_)
                ));

                let expected = create_metrics_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, metrics_proto.as_bytes());

                // Send Ack back to unblock the gRPC handler
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(metrics_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack for metrics");
                }

                // Receive trace pdata
                let trace_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for trace message")
                    .expect("No trace message received");

                // Validate trace payload
                let trace_proto: OtlpProtoBytes = trace_pdata
                    .clone()
                    .payload()
                    .try_into()
                    .expect("can convert to OtlpProtoBytes");
                assert!(matches!(
                    trace_proto,
                    OtlpProtoBytes::ExportTracesRequest(_)
                ));

                let expected = create_traces_service_request();
                let mut expected_bytes = Vec::new();
                expected.encode(&mut expected_bytes).unwrap();
                assert_eq!(&expected_bytes, trace_proto.as_bytes());

                // Send Ack back to unblock the gRPC handler
                if let Some((_node_id, ack)) =
                    crate::pdata::Context::next_ack(AckMsg::new(trace_pdata))
                {
                    ctx.send_control_msg(NodeControlMsg::Ack(ack))
                        .await
                        .expect("Failed to send Ack for traces");
                }
            })
        }
    }

    #[test]
    fn test_otlp_receiver_ack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        // Create a proper pipeline context for the test
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config: Config {
                    wait_for_result: true,
                    listening_addr: addr,
                    compression_method: None,
                    max_concurrent_requests: 1000,
                },
                metrics: pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint))
            .run_validation_concurrent(validation_procedure());
    }

    #[test]
    fn test_otlp_receiver_nack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTLP_RECEIVER_URN));

        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);

        let receiver = ReceiverWrapper::shared(
            OTLPReceiver {
                config: Config {
                    wait_for_result: true,
                    listening_addr: addr,
                    compression_method: None,
                    max_concurrent_requests: 1000,
                },
                metrics: pipeline_ctx.register_metrics::<OtlpReceiverMetrics>(),
            },
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let nack_scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let mut logs_client = LogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server");

                let result = logs_client.export(create_logs_service_request()).await;

                assert!(result.is_err(), "Expected error response");
                let status = result.unwrap_err();

                // Verify we get UNAVAILABLE status code
                assert_eq!(status.code(), tonic::Code::Unavailable);
                assert!(status.message().contains("Test nack reason"));
                assert!(status.message().contains("Pipeline processing failed"));

                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let nack_validation = |mut ctx: NotSendValidateContext<OtapPdata>| {
            Box::pin(async move {
                // Receive the logs pdata, create Nack message and send it back
                let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("Timed out waiting for logs message")
                    .expect("No logs message received");

                let nack = NackMsg::new("Test nack reason", logs_pdata);
                if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                    ctx.send_control_msg(NodeControlMsg::Nack(nack))
                        .await
                        .expect("Failed to send Nack");
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(nack_scenario)
            .run_validation_concurrent(nack_validation);
    }
}
