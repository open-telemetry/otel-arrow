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
use crate::otap_grpc::otlp::server::{RouteResponse, SharedState};
use crate::otap_grpc::{
    ArrowLogsServiceImpl, ArrowMetricsServiceImpl, ArrowTracesServiceImpl, Settings,
};
use crate::pdata::OtapPdata;
#[cfg(feature = "experimental-tls")]
use crate::tls_utils::{build_tls_acceptor, create_tls_stream};
#[cfg(feature = "experimental-tls")]
use otap_df_config::tls::TlsServerConfig;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
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
use otap_df_pdata::proto::opentelemetry::arrow::v1::{
    arrow_logs_service_server::ArrowLogsServiceServer,
    arrow_metrics_service_server::ArrowMetricsServiceServer,
    arrow_traces_service_server::ArrowTracesServiceServer,
};
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use std::net::SocketAddr;
use std::ops::Add;
use std::sync::Arc;
use std::time::{Duration, Instant};
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

    /// Size of the channel used to buffer outgoing responses to the client.
    response_stream_channel_size: usize,

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

    /// Timeout for RPC requests. If not specified, no timeout is applied.
    /// Format: humantime format (e.g., "30s", "5m", "1h", "500ms")
    #[serde(default, with = "humantime_serde")]
    pub timeout: Option<Duration>,

    /// TLS configuration
    #[cfg(feature = "experimental-tls")]
    pub tls: Option<TlsServerConfig>,
}

const fn default_max_concurrent_requests() -> usize {
    1000
}

const fn default_wait_for_result() -> bool {
    // See https://github.com/open-telemetry/otel-arrow/issues/1311
    // This matches the OTel Collector default for wait_for_result, presently.
    false
}

/// A Receiver that listens for OTAP messages
pub struct OTAPReceiver {
    config: Config,
    metrics: MetricSet<OtapReceiverMetrics>,
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
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

impl OTAPReceiver {
    /// Creates a new OTAPReceiver from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // Register OTAP receiver metrics for this node.
        let metrics = pipeline_ctx.register_metrics::<OtapReceiverMetrics>();

        Ok(OTAPReceiver { config, metrics })
    }

    fn route_ack_response(&self, states: &SharedStates, ack: AckMsg<OtapPdata>) -> RouteResponse {
        let calldata = ack.calldata;
        let resp = Ok(());
        let state = match ack.accepted.signal_type() {
            SignalType::Logs => states.logs.as_ref(),
            SignalType::Metrics => states.metrics.as_ref(),
            SignalType::Traces => states.traces.as_ref(),
        };

        state
            .map(|s| s.route_response(calldata, resp))
            .unwrap_or(RouteResponse::None)
    }

    fn route_nack_response(
        &self,
        states: &SharedStates,
        mut nack: NackMsg<OtapPdata>,
    ) -> RouteResponse {
        let calldata = std::mem::take(&mut nack.calldata);
        let signal_type = nack.refused.signal_type();
        let resp = Err(nack);
        let state = match signal_type {
            SignalType::Logs => states.logs.as_ref(),
            SignalType::Metrics => states.metrics.as_ref(),
            SignalType::Traces => states.traces.as_ref(),
        };

        state
            .map(|s| s.route_response(calldata, resp))
            .unwrap_or(RouteResponse::None)
    }

    fn handle_ack_response(&mut self, resp: RouteResponse) {
        match resp {
            RouteResponse::Sent => self.metrics.acks_sent.inc(),
            RouteResponse::Expired => self.metrics.acks_nacks_invalid_or_expired.inc(),
            RouteResponse::Invalid => self.metrics.acks_nacks_invalid_or_expired.inc(),
            RouteResponse::None => {}
        }
    }

    fn handle_nack_response(&mut self, resp: RouteResponse) {
        match resp {
            RouteResponse::Sent => self.metrics.nacks_sent.inc(),
            RouteResponse::Expired => self.metrics.acks_nacks_invalid_or_expired.inc(),
            RouteResponse::Invalid => self.metrics.acks_nacks_invalid_or_expired.inc(),
            RouteResponse::None => {}
        }
    }
}

/// OTAP receiver metrics.
#[metric_set(name = "otap.receiver.metrics")]
#[derive(Debug, Default, Clone)]
pub struct OtapReceiverMetrics {
    /// Number of acks sent.
    #[metric(unit = "{acks}")]
    pub acks_sent: Counter<u64>,

    /// Number of nacks sent.
    #[metric(unit = "{nacks}")]
    pub nacks_sent: Counter<u64>,

    /// Number of invalid/expired acks/nacks.
    #[metric(unit = "{ack_or_nack}")]
    pub acks_nacks_invalid_or_expired: Counter<u64>,
}

/// State shared between gRPC server task and the effect handler.
struct SharedStates {
    logs: Option<SharedState>,
    metrics: Option<SharedState>,
    traces: Option<SharedState>,
}

// Use the async_trait due to the need for thread safety because of tonic requiring Send and Sync traits
// The Shared version of the receiver allows us to implement a Receiver that requires the effect handler to be Send and Sync
//
#[async_trait]
impl shared::Receiver<OtapPdata> for OTAPReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_msg_recv: shared::ControlChannel<OtapPdata>,
        effect_handler: shared::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otap_df_telemetry::otel_info!(
            "receiver.start",
            listening_addr = %self.config.listening_addr
        );

        // create listener on addr provided from config
        let listener = effect_handler.tcp_listener(self.config.listening_addr)?;
        let listener_stream = TcpListenerStream::new(listener);

        let settings = Settings {
            response_stream_channel_size: self.config.response_stream_channel_size,
            max_concurrent_requests: self.config.max_concurrent_requests,
            wait_for_result: self.config.wait_for_result,
        };

        //create services for the grpc server and clone the effect handler to pass message
        let logs_service = ArrowLogsServiceImpl::new(effect_handler.clone(), &settings);
        let metrics_service = ArrowMetricsServiceImpl::new(effect_handler.clone(), &settings);
        let traces_service = ArrowTracesServiceImpl::new(effect_handler.clone(), &settings);

        let states = SharedStates {
            logs: logs_service.state(),
            metrics: metrics_service.state(),
            traces: traces_service.state(),
        };

        let mut logs_server = ArrowLogsServiceServer::new(logs_service);
        let mut metrics_server = ArrowMetricsServiceServer::new(metrics_service);
        let mut traces_server = ArrowTracesServiceServer::new(traces_service);

        // apply the tonic compression if it is set
        if let Some(ref compression) = self.config.compression_method {
            let encoding = compression.map_to_compression_encoding();

            logs_server = logs_server
                .send_compressed(encoding)
                .accept_compressed(encoding);
            metrics_server = metrics_server
                .send_compressed(encoding)
                .accept_compressed(encoding);
            traces_server = traces_server
                .send_compressed(encoding)
                .accept_compressed(encoding);
        }

        let mut server_builder = Server::builder();

        // Apply timeout if configured
        if let Some(timeout) = self.config.timeout {
            server_builder = server_builder.timeout(timeout);
        }

        #[cfg(feature = "experimental-tls")]
        let maybe_tls_acceptor =
            build_tls_acceptor(self.config.tls.as_ref())
                .await
                .map_err(|e| Error::ReceiverError {
                    receiver: effect_handler.receiver_id(),
                    kind: ReceiverErrorKind::Configuration,
                    error: format!("Failed to configure TLS: {}", e),
                    source_detail: format_error_sources(&e),
                })?;

        #[cfg(feature = "experimental-tls")]
        let handshake_timeout = self.config.tls.as_ref().and_then(|t| t.handshake_timeout);

        let server = server_builder
            .layer(MiddlewareLayer::new(ZstdRequestHeaderAdapter::default()))
            .add_service(logs_server)
            .add_service(metrics_server)
            .add_service(traces_server);

        // Start periodic telemetry collection
        let telemetry_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        tokio::select! {
            biased; //prioritize ctrl_msg over all other blocks

            // Process internal events
            ctrl_msg_result = async {
                loop {
                    match ctrl_msg_recv.recv().await {
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            otap_df_telemetry::otel_info!("otap.receiver.shutdown");
                            let snapshot = self.metrics.snapshot();
                            _ = telemetry_cancel_handle.cancel().await;
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        },
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            // Report current receiver metrics.
                            _ = metrics_reporter.report(&mut self.metrics);
                        },
                        Ok(NodeControlMsg::Ack(ack)) => {
                            self.handle_ack_response(self.route_ack_response(&states, ack));
                        },
                        Ok(NodeControlMsg::Nack(nack)) => {
                            self.handle_nack_response(self.route_nack_response(&states, nack));
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
            // Note: Unlike otlp_receiver.rs, this uses an inline match because the server
            // has a middleware layer applied which changes the Router type, making it
            // incompatible with a shared helper function without complex generic bounds.
            result = async {
                #[cfg(feature = "experimental-tls")]
                match maybe_tls_acceptor {
                    Some(tls_acceptor) => {
                        let tls_stream = create_tls_stream(listener_stream, tls_acceptor, handshake_timeout);
                        server.serve_with_incoming(tls_stream).await
                    }
                    None => server.serve_with_incoming(listener_stream).await,
                }
                #[cfg(not(feature = "experimental-tls"))]
                {
                    server.serve_with_incoming(listener_stream).await
                }
            } => {
                if let Err(error) = result {
                    // Report receiver error
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
    use crate::otap_mock::create_otap_batch;
    use crate::otap_receiver::{OTAP_RECEIVER_URN, OTAPReceiver};
    use crate::pdata::OtapPdata;
    use async_stream::stream;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_pdata::Producer;
    use otap_df_pdata::otap::OtapArrowRecords;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, arrow_logs_service_client::ArrowLogsServiceClient,
        arrow_metrics_service_client::ArrowMetricsServiceClient,
        arrow_traces_service_client::ArrowTracesServiceClient,
    };
    use std::collections::HashSet;
    use std::future::Future;
    use std::net::SocketAddr;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::time::Instant;
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
                let metrics_response = arrow_metrics_client
                    .arrow_metrics(metrics_stream)
                    .await
                    .expect("Failed to receive response after sending Metrics Request");

                validate_batch_responses(
                    metrics_response.into_inner(),
                    0,
                    "Successfully received",
                    "metrics",
                )
                .await;

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
                let logs_response = arrow_logs_client
                    .arrow_logs(logs_stream)
                    .await
                    .expect("Failed to receive response after sending Logs Request");

                validate_batch_responses(
                    logs_response.into_inner(),
                    0,
                    "Successfully received",
                    "logs",
                )
                .await;

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
                let traces_response = arrow_traces_client
                    .arrow_traces(traces_stream)
                    .await
                    .expect("Failed to receive response after sending Trace Request");

                validate_batch_responses(
                    traces_response.into_inner(),
                    0,
                    "Successfully received",
                    "traces",
                )
                .await;

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Instant::now(), "Test")
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
    /// Also sends ACKs when wait_for_result is enabled.
    fn validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // check that messages have been sent through the effect_handler

                // read from the effect handler
                for batch_id in 0..3 {
                    let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received");

                    // Validate the payload
                    let metrics_records: OtapArrowRecords = metrics_pdata
                        .clone()
                        .payload()
                        .try_into()
                        .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_metrics_message =
                        create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                    assert!(matches!(metrics_records, _expected_metrics_message));

                    // Send ACK if wait_for_result is enabled
                    if let Some((_node_id, ack)) =
                        crate::pdata::Context::next_ack(AckMsg::new(metrics_pdata))
                    {
                        ctx.send_control_msg(NodeControlMsg::Ack(ack))
                            .await
                            .expect("Failed to send Ack for metrics");
                    }
                }

                for batch_id in 0..3 {
                    let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received");

                    // Validate the payload
                    let logs_records: OtapArrowRecords = logs_pdata
                        .clone()
                        .payload()
                        .try_into()
                        .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_logs_message =
                        create_otap_batch(batch_id, ArrowPayloadType::Logs);
                    assert!(matches!(logs_records, _expected_logs_message));

                    // Send ACK if wait_for_result is enabled
                    if let Some((_node_id, ack)) =
                        crate::pdata::Context::next_ack(AckMsg::new(logs_pdata))
                    {
                        ctx.send_control_msg(NodeControlMsg::Ack(ack))
                            .await
                            .expect("Failed to send Ack for logs");
                    }
                }

                for batch_id in 0..3 {
                    let traces_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received");

                    // Validate the payload
                    let traces_records: OtapArrowRecords = traces_pdata
                        .clone()
                        .payload()
                        .try_into()
                        .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_traces_message =
                        create_otap_batch(batch_id, ArrowPayloadType::Spans);
                    assert!(matches!(traces_records, _expected_traces_message));

                    // Send ACK if wait_for_result is enabled
                    if let Some((_node_id, ack)) =
                        crate::pdata::Context::next_ack(AckMsg::new(traces_pdata))
                    {
                        ctx.send_control_msg(NodeControlMsg::Ack(ack))
                            .await
                            .expect("Failed to send Ack for traces");
                    }
                }
            })
        }
    }

    /// Test scenario for NACK functionality - expects error responses for all signals
    fn nack_scenario(
        grpc_endpoint: String,
    ) -> impl FnOnce(TestContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        move |ctx| {
            Box::pin(async move {
                // Test NACK with metrics
                let mut arrow_metrics_client =
                    ArrowMetricsServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("Failed to connect to server");

                #[allow(tail_expr_drop_order)]
                let metrics_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut metrics_records = create_otap_batch(batch_id, ArrowPayloadType::MultivariateMetrics);
                        let bar = producer.produce_bar(&mut metrics_records).unwrap();
                        yield bar
                    }
                };

                let metrics_response = arrow_metrics_client
                    .arrow_metrics(metrics_stream)
                    .await
                    .expect("Failed to receive response after sending Metrics Request");

                validate_batch_responses(
                    metrics_response.into_inner(),
                    14, // `StatusCode::Unavailable`
                    &format!(
                        "Pipeline processing failed: {}",
                        "Test NACK reason for metrics"
                    ),
                    "metrics",
                )
                .await;

                // Test NACK with logs
                let mut arrow_logs_client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                    .await
                    .expect("Failed to connect to server");

                #[allow(tail_expr_drop_order)]
                let logs_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut logs_records = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                        let bar = producer.produce_bar(&mut logs_records).unwrap();
                        yield bar;
                    }
                };

                let logs_response = arrow_logs_client
                    .arrow_logs(logs_stream)
                    .await
                    .expect("Failed to receive response after sending Logs Request");

                validate_batch_responses(
                    logs_response.into_inner(),
                    14, // `StatusCode::Unavailable`
                    &format!(
                        "Pipeline processing failed: {}",
                        "Test NACK reason for logs"
                    ),
                    "logs",
                )
                .await;

                // Test NACK with traces
                let mut arrow_traces_client =
                    ArrowTracesServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("Failed to connect to server");

                #[allow(tail_expr_drop_order)]
                let traces_stream = stream! {
                    let mut producer = Producer::new();
                    for batch_id in 0..3 {
                        let mut traces_records = create_otap_batch(batch_id, ArrowPayloadType::Spans);
                        let bar = producer.produce_bar(&mut traces_records).unwrap();
                        yield bar;
                    }
                };

                let traces_response = arrow_traces_client
                    .arrow_traces(traces_stream)
                    .await
                    .expect("Failed to receive response after sending Trace Request");

                validate_batch_responses(
                    traces_response.into_inner(),
                    14, // `StatusCode::Unavailable`
                    &format!(
                        "Pipeline processing failed: {}",
                        "Test NACK reason for traces"
                    ),
                    "traces",
                )
                .await;

                // Shutdown
                ctx.send_shutdown(Instant::now(), "Test complete")
                    .await
                    .expect("Failed to send shutdown");
            }) as Pin<Box<dyn Future<Output = ()>>>
        }
    }

    /// Validation procedure that sends NACKs for all signal types
    fn nack_validation_procedure()
    -> impl FnOnce(NotSendValidateContext<OtapPdata>) -> Pin<Box<dyn Future<Output = ()>>> {
        |mut ctx| {
            Box::pin(async move {
                // NACK metrics (3 batches)
                for _batch_id in 0..3 {
                    let metrics_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for metrics")
                        .expect("No metrics received");

                    let nack = NackMsg::new("Test NACK reason for metrics", metrics_pdata);
                    if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                        ctx.send_control_msg(NodeControlMsg::Nack(nack))
                            .await
                            .expect("Failed to send Nack for metrics");
                    }
                }

                // NACK logs (3 batches)
                for _batch_id in 0..3 {
                    let logs_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for logs")
                        .expect("No logs received");

                    let nack = NackMsg::new("Test NACK reason for logs", logs_pdata);
                    if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                        ctx.send_control_msg(NodeControlMsg::Nack(nack))
                            .await
                            .expect("Failed to send Nack for logs");
                    }
                }

                // NACK traces (3 batches)
                for _batch_id in 0..3 {
                    let traces_pdata = timeout(Duration::from_secs(3), ctx.recv())
                        .await
                        .expect("Timed out waiting for traces")
                        .expect("No traces received");

                    let nack = NackMsg::new("Test NACK reason for traces", traces_pdata);
                    if let Some((_node_id, nack)) = crate::pdata::Context::next_nack(nack) {
                        ctx.send_control_msg(NodeControlMsg::Nack(nack))
                            .await
                            .expect("Failed to send Nack for traces");
                    }
                }
            }) as Pin<Box<dyn Future<Output = ()>>>
        }
    }

    /// Helper function to validate batch status responses with configurable expectations
    async fn validate_batch_responses<S>(
        mut inbound_stream: S,
        expected_status_code: i32,
        expected_status_message: &str,
        signal_name: &str,
    ) where
        S: futures::Stream<
                Item = Result<
                    otap_df_pdata::proto::opentelemetry::arrow::v1::BatchStatus,
                    tonic::Status,
                >,
            > + Unpin,
    {
        use futures::StreamExt;

        let mut received_batch_ids = HashSet::new();

        // Process each item in the response stream
        while let Some(result) = inbound_stream.next().await {
            assert!(
                result.is_ok(),
                "Expected successful response from server for {}",
                signal_name
            );
            let batch_status = result.unwrap();
            let batch_id = batch_status.batch_id;

            // Check for duplicates
            assert!(
                received_batch_ids.insert(batch_id),
                "Received duplicate response for batch ID {} in {}",
                batch_id,
                signal_name
            );

            assert_eq!(
                batch_status.status_code, expected_status_code,
                "Expected status code {} for batch ID {} in {}",
                expected_status_code, batch_id, signal_name
            );

            assert_eq!(
                batch_status.status_message, expected_status_message,
                "Expected status message '{}' for batch ID {} in {}",
                expected_status_message, batch_id, signal_name
            );
        }

        // Verify we received all expected batch IDs
        assert_eq!(
            received_batch_ids,
            (0..3).collect::<HashSet<_>>(),
            "Did not receive responses for all expected batch IDs in {}. Got: {:?}",
            signal_name,
            received_batch_ids
        );
    }

    #[test]
    fn test_otap_receiver() {
        let test_runtime = TestRuntime::new();

        // addr and port for the server to run at
        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let response_stream_channel_size = 100;

        // create our receiver
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));

        // Create a proper pipeline context for the test
        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;
        use serde_json::json;

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        // Create config JSON
        let config = json!({
            "listening_addr": addr.to_string(),
            "response_stream_channel_size": response_stream_channel_size
        });

        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline_ctx, &config).unwrap(),
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

    #[test]
    fn test_config_parsing() {
        use crate::compression::CompressionMethod;
        use serde_json::json;

        let telemetry_registry_handle = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        // Test with custom max_concurrent_requests, max_concurrent_requests defaults to 1000
        let config_with_max_concurrent_requests = json!({
            "listening_addr": "127.0.0.1:4317",
            "response_stream_channel_size": 100,
            "max_concurrent_requests": 5000
        });
        let receiver =
            OTAPReceiver::from_config(pipeline_ctx.clone(), &config_with_max_concurrent_requests)
                .unwrap();
        assert_eq!(receiver.config.listening_addr.to_string(), "127.0.0.1:4317");
        assert_eq!(receiver.config.response_stream_channel_size, 100);
        assert_eq!(receiver.config.max_concurrent_requests, 5000);
        assert!(!receiver.config.wait_for_result);
        assert!(receiver.config.compression_method.is_none());
        assert!(receiver.config.timeout.is_none());

        // Test with minimal required fields, max_concurrent_requests defaults to 1000, wait_for_result defaults to false
        let config_minimal = json!({
            "listening_addr": "127.0.0.1:4318",
            "response_stream_channel_size": 200
        });
        let receiver = OTAPReceiver::from_config(pipeline_ctx.clone(), &config_minimal).unwrap();
        assert_eq!(receiver.config.listening_addr.to_string(), "127.0.0.1:4318");
        assert_eq!(receiver.config.response_stream_channel_size, 200);
        assert_eq!(receiver.config.max_concurrent_requests, 1000);
        assert!(!receiver.config.wait_for_result);
        assert!(receiver.config.compression_method.is_none());
        assert!(receiver.config.timeout.is_none());

        // Test with full configuration including gzip compression
        let config_full_gzip = json!({
            "listening_addr": "127.0.0.1:4319",
            "response_stream_channel_size": 150,
            "compression_method": "gzip",
            "max_concurrent_requests": 2500,
            "wait_for_result": true,
            "timeout": "30s"
        });
        let receiver = OTAPReceiver::from_config(pipeline_ctx.clone(), &config_full_gzip).unwrap();
        assert_eq!(receiver.config.listening_addr.to_string(), "127.0.0.1:4319");
        assert_eq!(receiver.config.response_stream_channel_size, 150);
        assert_eq!(receiver.config.max_concurrent_requests, 2500);
        assert!(receiver.config.wait_for_result);
        assert!(matches!(
            receiver.config.compression_method,
            Some(CompressionMethod::Gzip)
        ));
        assert_eq!(receiver.config.timeout, Some(Duration::from_secs(30)));

        // Test with zstd compression
        let config_with_zstd = json!({
            "listening_addr": "127.0.0.1:4320",
            "response_stream_channel_size": 50,
            "compression_method": "zstd",
            "wait_for_result": false
        });
        let receiver = OTAPReceiver::from_config(pipeline_ctx.clone(), &config_with_zstd).unwrap();
        assert_eq!(receiver.config.listening_addr.to_string(), "127.0.0.1:4320");
        assert_eq!(receiver.config.response_stream_channel_size, 50);
        assert!(!receiver.config.wait_for_result);
        assert!(matches!(
            receiver.config.compression_method,
            Some(CompressionMethod::Zstd)
        ));
        assert!(receiver.config.timeout.is_none());

        // Test with deflate compression
        let config_with_deflate = json!({
            "listening_addr": "127.0.0.1:4321",
            "response_stream_channel_size": 75,
            "compression_method": "deflate"
        });
        let receiver = OTAPReceiver::from_config(pipeline_ctx, &config_with_deflate).unwrap();
        assert_eq!(receiver.config.listening_addr.to_string(), "127.0.0.1:4321");
        assert_eq!(receiver.config.response_stream_channel_size, 75);
        assert!(matches!(
            receiver.config.compression_method,
            Some(CompressionMethod::Deflate)
        ));
        assert!(receiver.config.timeout.is_none());
    }

    #[test]
    fn test_otap_receiver_ack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));

        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;
        use serde_json::json;

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let config = json!({
            "listening_addr": addr.to_string(),
            "response_stream_channel_size": 100,
            "wait_for_result": true  // Enable ACK handling
        });

        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline_ctx, &config).unwrap(),
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
    fn test_otap_receiver_nack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = portpicker::pick_unused_port().expect("No free ports");
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();

        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));

        use otap_df_engine::context::ControllerContext;
        use otap_df_telemetry::registry::TelemetryRegistryHandle;
        use serde_json::json;

        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let config = json!({
            "listening_addr": addr.to_string(),
            "response_stream_channel_size": 100,
            "wait_for_result": true  // Enable NACK handling
        });

        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline_ctx, &config).unwrap(),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_receiver(receiver)
            .run_test(nack_scenario(grpc_endpoint)) // Use NACK-specific scenario
            .run_validation_concurrent(nack_validation_procedure()); // Use NACK-specific validation
    }
}
