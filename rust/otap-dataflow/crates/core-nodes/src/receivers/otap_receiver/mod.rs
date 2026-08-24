// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP receiver node
//!
//! ToDo: implement Ack and Nack control message, wait for receiver node to receive a Ack control message then the service can send a response back
//! ToDo: implement config control message to handle live changing configuration
//! ToDo: Add HTTP support
//! ToDo: Implement proper deadline function for Shutdown ctrl msg
//!

otap_df_telemetry::otel_component_scope!(urn = OTAP_RECEIVER_URN, target = "otel.receiver.otap",);

use otap_df_config::tls::TlsServerConfig;
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::compression::CompressionMethod;
use otap_df_otap::memory_pressure_layer::{MemoryPressureLayer, ReceiverRejectionMetrics};
use otap_df_otap::otap_grpc::middleware::zstd_header::ZstdRequestHeaderAdapter;
use otap_df_otap::otap_grpc::otlp::server::{RouteResponse, SharedState};
use otap_df_otap::otap_grpc::{
    ArrowLogsServiceImpl, ArrowMetricsServiceImpl, ArrowTracesServiceImpl, OtapReceiverTelemetry,
    OtapStreamTaskManager, Settings,
};
use otap_df_otap::pdata::OtapPdata;
use otap_df_otap::tls_utils::{build_tls_acceptor, create_tls_stream};

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::clock;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ReceiverErrorKind, format_error_sources};
use otap_df_engine::memory_limiter::SharedReceiverAdmissionState;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::shared::receiver as shared;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::proto::opentelemetry::arrow::v1::{
    arrow_logs_service_server::ArrowLogsServiceServer,
    arrow_metrics_service_server::ArrowMetricsServiceServer,
    arrow_traces_service_server::ArrowTracesServiceServer,
};
use otap_df_telemetry::common_attributes::{
    Outcome, ReceiverRejectionAttributes, ReceiverRejectionErrorType, SignalAttributes,
    SignalOutcomeAttributes,
};
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use parking_lot::Mutex;
use serde::de::Error as SerdeError;
use serde::{Deserialize, Deserializer};
use serde_json::Value;
use std::net::SocketAddr;
use std::num::NonZeroUsize;
use std::ops::Add;
use std::sync::Arc;
use std::task::Poll;
use std::time::{Duration, Instant};
use tokio_util::sync::CancellationToken;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;
use tonic_middleware::MiddlewareLayer;

const OTAP_RECEIVER_URN: &str = "urn:otel:receiver:otap";

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

    /// Maximum number of concurrent wait-for-result requests admitted from one OTAP stream. Clamped to `max_concurrent_requests` at runtime.
    #[serde(
        default = "default_max_concurrent_requests_per_stream",
        deserialize_with = "deserialize_positive_max_concurrent_requests_per_stream"
    )]
    max_concurrent_requests_per_stream: NonZeroUsize,

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
    pub tls: Option<TlsServerConfig>,
}

const fn default_max_concurrent_requests() -> usize {
    1000
}

fn default_max_concurrent_requests_per_stream() -> NonZeroUsize {
    NonZeroUsize::new(16).expect("default per-stream concurrency must be non-zero")
}

fn deserialize_positive_usize<'de, D>(
    deserializer: D,
    field_name: &str,
) -> Result<NonZeroUsize, D::Error>
where
    D: Deserializer<'de>,
{
    let value = usize::deserialize(deserializer)?;
    NonZeroUsize::new(value)
        .ok_or_else(|| D::Error::custom(format!("{field_name} must be greater than 0")))
}

fn deserialize_positive_max_concurrent_requests_per_stream<'de, D>(
    deserializer: D,
) -> Result<NonZeroUsize, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_positive_usize(deserializer, "max_concurrent_requests_per_stream")
}

const fn default_wait_for_result() -> bool {
    // See https://github.com/open-telemetry/otel-arrow/issues/1311
    // This matches the OTel Collector default for wait_for_result, presently.
    false
}

/// A Receiver that listens for OTAP messages
pub struct OTAPReceiver {
    config: Config,
    metrics: Arc<SharedOtapReceiverMetrics>,
    admission_state: SharedReceiverAdmissionState,
}

/// Declares the OTAP receiver as a shared receiver factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Receiver)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static OTAP_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: OTAP_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
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
        let metrics = Arc::new(SharedOtapReceiverMetrics::new(
            OtapReceiverMetrics::register(&pipeline_ctx),
        ));

        Ok(OTAPReceiver {
            config,
            metrics,
            admission_state: SharedReceiverAdmissionState::from_process_state(
                &pipeline_ctx.memory_pressure_state(),
            ),
        })
    }

    fn route_ack_response(
        &self,
        states: &SharedStates,
        ack: AckMsg<OtapPdata>,
    ) -> (SignalType, RouteResponse) {
        let calldata = ack.unwind.route.calldata;
        let resp = Ok(());
        let signal = ack.accepted.signal_type();
        let state = match signal {
            SignalType::Logs => states.logs.as_ref(),
            SignalType::Metrics => states.metrics.as_ref(),
            SignalType::Traces => states.traces.as_ref(),
        };

        (
            signal,
            state
                .map(|s| s.route_response(calldata, resp))
                .unwrap_or(RouteResponse::None),
        )
    }

    fn route_nack_response(
        &self,
        states: &SharedStates,
        mut nack: NackMsg<OtapPdata>,
    ) -> (SignalType, RouteResponse) {
        let calldata = std::mem::take(&mut nack.unwind.route.calldata);
        let signal_type = nack.refused.signal_type();
        let resp = Err(nack);
        let state = match signal_type {
            SignalType::Logs => states.logs.as_ref(),
            SignalType::Metrics => states.metrics.as_ref(),
            SignalType::Traces => states.traces.as_ref(),
        };

        (
            signal_type,
            state
                .map(|s| s.route_response(calldata, resp))
                .unwrap_or(RouteResponse::None),
        )
    }

    fn handle_ack_response(&mut self, signal: SignalType, resp: RouteResponse) {
        let mut metrics = self.metrics.lock();
        match resp {
            RouteResponse::Sent => metrics.record_acknowledgement(signal, Outcome::Success),
            RouteResponse::Expired | RouteResponse::Invalid => {
                metrics.record_acknowledgement(signal, Outcome::Failure);
            }
            RouteResponse::None => {}
        }
    }

    fn handle_nack_response(&mut self, signal: SignalType, resp: RouteResponse) {
        let mut metrics = self.metrics.lock();
        match resp {
            RouteResponse::Sent => metrics.record_acknowledgement(signal, Outcome::Refused),
            RouteResponse::Expired | RouteResponse::Invalid => {
                metrics.record_acknowledgement(signal, Outcome::Failure);
            }
            RouteResponse::None => {}
        }
    }

    fn terminal_state(&mut self, deadline: Instant) -> TerminalState {
        TerminalState::new(deadline, self.metrics.lock().terminal_snapshots())
    }
}

/// Lifecycle and payload metrics for admitted OTAP batches.
#[metric_set(
    name = "receiver.otap.batches",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtapBatchMetrics {
    /// Number of batches admitted to the pipeline send path.
    #[metric(unit = "{batch}")]
    pub started: Counter<u64>,
    /// Number of admitted batches whose receiver work terminated.
    #[metric(unit = "{batch}")]
    pub completed: Counter<u64>,
    /// Protobuf-encoded batch bytes after gRPC transport decompression.
    #[metric(unit = "By")]
    pub payload_size: Counter<u64>,
}

/// OTAP acknowledgement routing results.
#[metric_set(
    name = "receiver.otap.acknowledgements",
    measurement_attributes = SignalOutcomeAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtapAcknowledgementMetrics {
    /// Number of routed or invalid acknowledgement responses.
    #[metric(unit = "{response}")]
    pub responses: Counter<u64>,
}

/// OTAP streams and batches rejected before pipeline admission.
#[metric_set(
    name = "receiver.otap.rejections",
    measurement_attributes = ReceiverRejectionAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtapRejectionMetrics {
    /// Number of rejected OTAP streaming RPCs.
    #[metric(unit = "{stream}")]
    pub streams: Counter<u64>,
    /// Number of rejected OTAP batches within admitted streams.
    #[metric(unit = "{batch}")]
    pub batches: Counter<u64>,
}

/// Bounded-cardinality OTAP receiver metrics tracker.
pub struct OtapReceiverMetrics {
    batches: MeasurementMetricSet<OtapBatchMetrics>,
    acknowledgements: MeasurementMetricSet<OtapAcknowledgementMetrics>,
    rejections: MeasurementMetricSet<OtapRejectionMetrics>,
}

impl std::fmt::Debug for OtapReceiverMetrics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OtapReceiverMetrics").finish()
    }
}

impl OtapReceiverMetrics {
    /// Registers all OTAP receiver metric sets for a pipeline node.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            batches: OtapBatchMetrics::register(pipeline_ctx),
            acknowledgements: OtapAcknowledgementMetrics::register(pipeline_ctx),
            rejections: OtapRejectionMetrics::register(pipeline_ctx),
        }
    }

    /// Records an OTAP batch and its protobuf-encoded size after pipeline admission.
    pub fn record_batch_admitted(&mut self, signal: SignalType, payload_bytes: u64) {
        let batches = self.batches.with(SignalAttributes { signal });
        batches.started.inc();
        if payload_bytes > 0 {
            batches.payload_size.add(payload_bytes);
        }
    }

    /// Records termination of receiver work for an admitted OTAP batch.
    pub fn record_batch_completed(&mut self, signal: SignalType) {
        self.batches
            .with(SignalAttributes { signal })
            .completed
            .inc();
    }

    /// Records the outcome of routing an acknowledgement response.
    pub fn record_acknowledgement(&mut self, signal: SignalType, outcome: Outcome) {
        self.acknowledgements
            .with(SignalOutcomeAttributes { signal, outcome })
            .responses
            .inc();
    }

    /// Records an OTAP streaming RPC rejected before admission.
    pub fn record_stream_rejection(&mut self, error_type: ReceiverRejectionErrorType) {
        self.rejections
            .with(ReceiverRejectionAttributes { error_type })
            .streams
            .inc();
    }

    /// Records an OTAP batch rejected before admission.
    pub fn record_batch_rejection(&mut self, error_type: ReceiverRejectionErrorType) {
        self.rejections
            .with(ReceiverRejectionAttributes { error_type })
            .batches
            .inc();
    }

    /// Returns an acknowledgement bucket for inspection without marking it for export.
    #[must_use]
    pub fn acknowledgements_for(
        &self,
        signal: SignalType,
        outcome: Outcome,
    ) -> &OtapAcknowledgementMetrics {
        self.acknowledgements
            .get(SignalOutcomeAttributes { signal, outcome })
    }

    /// Returns a batch lifecycle bucket for inspection without marking it for export.
    #[must_use]
    pub fn batches_for(&self, signal: SignalType) -> &OtapBatchMetrics {
        self.batches.get(SignalAttributes { signal })
    }

    /// Returns a rejection bucket for inspection without marking it for export.
    #[must_use]
    pub fn rejections_for(&self, error_type: ReceiverRejectionErrorType) -> &OtapRejectionMetrics {
        self.rejections
            .get(ReceiverRejectionAttributes { error_type })
    }

    /// Reports every touched OTAP receiver metric bucket.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.batches)?;
        reporter.report_measurement(&mut self.acknowledgements)?;
        reporter.report_measurement(&mut self.rejections)
    }

    /// Takes every touched OTAP receiver metric bucket for terminal handoff.
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.batches.terminal_snapshots();
        snapshots.extend(self.acknowledgements.terminal_snapshots());
        snapshots.extend(self.rejections.terminal_snapshots());
        snapshots
    }
}

struct SharedOtapReceiverMetrics(Mutex<OtapReceiverMetrics>);

impl SharedOtapReceiverMetrics {
    fn new(metrics: OtapReceiverMetrics) -> Self {
        Self(Mutex::new(metrics))
    }

    fn lock(&self) -> parking_lot::MutexGuard<'_, OtapReceiverMetrics> {
        self.0.lock()
    }
}

impl ReceiverRejectionMetrics for SharedOtapReceiverMetrics {
    fn record_rejection(&self, error_type: ReceiverRejectionErrorType) {
        self.lock().record_stream_rejection(error_type);
    }

    fn record_item_rejection(&self, error_type: ReceiverRejectionErrorType) {
        self.lock().record_batch_rejection(error_type);
    }
}

impl OtapReceiverTelemetry for SharedOtapReceiverMetrics {
    fn record_batch_admitted(&self, signal: SignalType, payload_bytes: u64) {
        self.lock().record_batch_admitted(signal, payload_bytes);
    }

    fn record_batch_completed(&self, signal: SignalType) {
        self.lock().record_batch_completed(signal);
    }
}

/// State shared between gRPC server task and the effect handler.
struct SharedStates {
    logs: Option<SharedState>,
    metrics: Option<SharedState>,
    traces: Option<SharedState>,
}

impl SharedStates {
    fn is_empty(&self) -> bool {
        self.logs.as_ref().is_none_or(SharedState::is_empty)
            && self.metrics.as_ref().is_none_or(SharedState::is_empty)
            && self.traces.as_ref().is_none_or(SharedState::is_empty)
    }

    fn force_shutdown(&self, reason: &str) {
        if let Some(state) = &self.logs {
            state.force_shutdown(SignalType::Logs, reason);
        }
        if let Some(state) = &self.metrics {
            state.force_shutdown(SignalType::Metrics, reason);
        }
        if let Some(state) = &self.traces {
            state.force_shutdown(SignalType::Traces, reason);
        }
    }
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
        otel_info!(
            "otap_receiver.start",
            listening_addr = %self.config.listening_addr
        );

        // create listener on addr provided from config
        let listener = effect_handler.tcp_listener(self.config.listening_addr)?;
        let listener_stream = TcpListenerStream::new(listener);

        let stream_tasks = OtapStreamTaskManager::new();
        let settings = Settings {
            response_stream_channel_size: self.config.response_stream_channel_size,
            max_concurrent_requests: self.config.max_concurrent_requests,
            max_concurrent_requests_per_stream: self
                .config
                .max_concurrent_requests_per_stream
                .get(),
            wait_for_result: self.config.wait_for_result,
            admission_state: self.admission_state.clone(),
            receiver_metrics: Some(self.metrics.clone()),
            stream_tasks: stream_tasks.clone(),
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

        let maybe_tls_acceptor =
            build_tls_acceptor(self.config.tls.as_ref())
                .await
                .map_err(|e| Error::ReceiverError {
                    receiver: effect_handler.receiver_id(),
                    kind: ReceiverErrorKind::Configuration,
                    error: format!("Failed to configure TLS: {}", e),
                    source_detail: format_error_sources(&e),
                })?;

        let handshake_timeout = self.config.tls.as_ref().and_then(|t| t.handshake_timeout);

        let server = server_builder
            .layer(MemoryPressureLayer::with_metrics(
                self.admission_state.clone(),
                self.metrics.clone(),
            ))
            .layer(MiddlewareLayer::new(ZstdRequestHeaderAdapter::default()))
            .add_service(logs_server)
            .add_service(metrics_server)
            .add_service(traces_server);

        let grpc_shutdown = CancellationToken::new();
        let server_task = {
            let grpc_shutdown = grpc_shutdown.clone();
            async {
                match maybe_tls_acceptor {
                    Some(tls_acceptor) => {
                        let tls_stream =
                            create_tls_stream(listener_stream, tls_acceptor, handshake_timeout);
                        server
                            .serve_with_incoming_shutdown(tls_stream, async move {
                                grpc_shutdown.cancelled().await;
                            })
                            .await
                    }
                    None => {
                        server
                            .serve_with_incoming_shutdown(listener_stream, async move {
                                grpc_shutdown.cancelled().await;
                            })
                            .await
                    }
                }
            }
        };
        tokio::pin!(server_task);

        let mut server_task_done = false;
        let mut draining_deadline: Option<Instant> = None;
        let mut draining_reason: Option<String> = None;
        let mut drain_deadline_sleep: Option<clock::Sleep> = None;
        let mut terminal_deadline: Option<Instant> = None;
        let mut terminal_error: Option<Error> = None;

        loop {
            if let Some(deadline) = draining_deadline {
                // DrainIngress is receiver-first shutdown: stop accepting new RPCs
                // immediately, but keep the event loop alive until the serving task
                // has exited and all in-flight wait_for_result state has been
                // resolved or force-failed at the deadline.
                grpc_shutdown.cancel();

                if clock::now() >= deadline {
                    if let Some(reason) = draining_reason.as_deref() {
                        states.force_shutdown(reason);
                    }
                    drain_deadline_sleep = None;
                } else if drain_deadline_sleep.is_none() {
                    drain_deadline_sleep = Some(clock::sleep_until(deadline));
                }

                if server_task_done && states.is_empty() && stream_tasks.is_empty() {
                    match effect_handler.notify_receiver_drained().await {
                        Ok(()) => terminal_deadline = Some(deadline),
                        Err(error) => terminal_error = Some(error),
                    }
                    stream_tasks.close();
                    stream_tasks.cancel();
                    break;
                }
            } else {
                drain_deadline_sleep = None;
            }

            let mut drain_sleep = std::pin::pin!(std::future::poll_fn(|cx| {
                if let Some(sleep) = drain_deadline_sleep.as_mut() {
                    sleep.as_mut().poll(cx)
                } else {
                    Poll::Pending
                }
            }));

            tokio::select! {
                biased;

                ctrl_msg = ctrl_msg_recv.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, reason })
                            if draining_deadline.is_none() => {
                                otel_info!("otap_receiver.drain_ingress");
                                // Latch the first drain request and close ingress.
                                // This stops new admissions, but does not yet report
                                // ReceiverDrained because previously admitted batches
                                // may still need to finish their wait_for_result path.
                                draining_deadline = Some(deadline);
                                draining_reason = Some(reason);
                                grpc_shutdown.cancel();
                            }
                        Ok(NodeControlMsg::Shutdown { deadline, reason }) => {
                            otel_info!("otap_receiver.shutdown");
                            grpc_shutdown.cancel();
                            states.force_shutdown(&reason);
                            stream_tasks.close();
                            terminal_deadline = Some(deadline);
                            break;
                        }
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            _ = self.metrics.lock().report(&mut metrics_reporter);
                        }
                        Ok(NodeControlMsg::MemoryPressureChanged { update }) => {
                            self.admission_state.apply(update);
                        }
                        Ok(NodeControlMsg::Ack(ack)) => {
                            let (signal, response) = self.route_ack_response(&states, ack);
                            self.handle_ack_response(signal, response);
                        }
                        Ok(NodeControlMsg::Nack(nack)) => {
                            let (signal, response) = self.route_nack_response(&states, nack);
                            self.handle_nack_response(signal, response);
                        }
                        Err(e) => {
                            terminal_error = Some(Error::ChannelRecvError(e));
                            stream_tasks.close();
                            stream_tasks.cancel();
                            break;
                        }
                        _ => {}
                    }
                }

                result = &mut server_task, if !server_task_done => {
                    server_task_done = true;
                    if let Err(error) = result {
                        let source_detail = format_error_sources(&error);
                        terminal_error = Some(Error::ReceiverError {
                            receiver: effect_handler.receiver_id(),
                            kind: ReceiverErrorKind::Transport,
                            error: error.to_string(),
                            source_detail,
                        });
                        stream_tasks.close();
                        stream_tasks.cancel();
                        break;
                    }

                    if draining_deadline.is_none() {
                        terminal_deadline =
                            Some(clock::now().add(Duration::from_secs(1)));
                        stream_tasks.close();
                        stream_tasks.cancel();
                        break;
                    }

                    stream_tasks.close();
                }

                _ = &mut drain_sleep => {
                    if let Some(reason) = draining_reason.as_deref() {
                        // The receiver missed the graceful-drain deadline. Force any
                        // remaining in-flight wait_for_result subscriptions to fail so
                        // the runtime can eventually observe ReceiverDrained.
                        states.force_shutdown(reason);
                    }
                    drain_deadline_sleep = None;
                }

                _ = stream_tasks.wait(), if server_task_done && draining_deadline.is_some() => {}
            }
        }

        grpc_shutdown.cancel();
        stream_tasks.close();
        if terminal_error.is_some() {
            stream_tasks.cancel();
        }

        if !server_task_done {
            let server_result = if terminal_error.is_some() {
                server_task.await
            } else {
                let deadline =
                    terminal_deadline.unwrap_or_else(|| clock::now().add(Duration::from_secs(1)));
                tokio::select! {
                    result = &mut server_task => result,
                    _ = clock::sleep_until(deadline) => {
                        stream_tasks.cancel();
                        server_task.await
                    }
                }
            };

            if let Err(error) = server_result {
                let source_detail = format_error_sources(&error);
                _ = terminal_error.get_or_insert_with(|| Error::ReceiverError {
                    receiver: effect_handler.receiver_id(),
                    kind: ReceiverErrorKind::Transport,
                    error: error.to_string(),
                    source_detail,
                });
            }
        }

        if terminal_error.is_some() {
            stream_tasks.cancel();
            stream_tasks.wait().await;
        } else if let Some(deadline) = terminal_deadline {
            tokio::select! {
                _ = stream_tasks.wait() => {}
                _ = clock::sleep_until(deadline) => {
                    stream_tasks.cancel();
                    stream_tasks.wait().await;
                }
            }
        }

        if let Some(error) = terminal_error {
            return Err(error);
        }

        let deadline =
            terminal_deadline.unwrap_or_else(|| clock::now().add(Duration::from_secs(1)));
        Ok(self.terminal_state(deadline))
    }
}

#[cfg(test)]
mod tests {
    use crate::receivers::otap_receiver::{OTAP_RECEIVER_URN, OTAPReceiver, OtapReceiverMetrics};
    use async_stream::stream;
    use otap_df_config::SignalType;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
    use otap_df_engine::receiver::ReceiverWrapper;
    use otap_df_engine::testing::{
        receiver::{NotSendValidateContext, TestContext, TestRuntime},
        test_node,
    };
    use otap_df_otap::memory_pressure_layer::ReceiverRejectionMetrics;
    use otap_df_otap::otap_mock::create_otap_batch;
    use otap_df_otap::pdata::OtapPdata;
    use otap_df_otap::testing::{next_ack, next_nack};
    use otap_df_pdata::Producer;
    use otap_df_pdata::TryIntoWithOptions;
    use otap_df_pdata::otap::OtapArrowRecords;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, arrow_logs_service_client::ArrowLogsServiceClient,
        arrow_metrics_service_client::ArrowMetricsServiceClient,
        arrow_traces_service_client::ArrowTracesServiceClient,
    };
    use otap_df_telemetry::common_attributes::{Outcome, ReceiverRejectionErrorType};
    use otap_df_telemetry::metrics::MetricSetSnapshot;
    use otap_df_telemetry::reporter::MetricsReporter;
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
        telemetry: (flume::Receiver<MetricSetSnapshot>, MetricsReporter),
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
                        let mut metrics_records = create_otap_batch(batch_id, ArrowPayloadType::UnivariateMetrics);
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
                    3,
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
                    3,
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
                    3,
                    "traces",
                )
                .await;

                assert_receiver_telemetry(&ctx, telemetry, None).await;

                // Finally, send a Shutdown event to terminate the receiver.
                ctx.send_shutdown(Instant::now(), "Test")
                    .await
                    .expect("Failed to send Shutdown");
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
                        .try_into_with_default()
                        .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_metrics_message =
                        create_otap_batch(batch_id, ArrowPayloadType::UnivariateMetrics);
                    assert!(matches!(metrics_records, _expected_metrics_message));

                    // Send ACK if wait_for_result is enabled
                    if let Some((_node_id, ack)) = next_ack(AckMsg::new(metrics_pdata)) {
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
                        .try_into_with_default()
                        .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_logs_message =
                        create_otap_batch(batch_id, ArrowPayloadType::Logs);
                    assert!(matches!(logs_records, _expected_logs_message));

                    // Send ACK if wait_for_result is enabled
                    if let Some((_node_id, ack)) = next_ack(AckMsg::new(logs_pdata)) {
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
                        .try_into_with_default()
                        .expect("Could convert pdata to OTAPData");

                    // Assert that the message received is what the test client sent.
                    let _expected_traces_message =
                        create_otap_batch(batch_id, ArrowPayloadType::Spans);
                    assert!(matches!(traces_records, _expected_traces_message));

                    // Send ACK if wait_for_result is enabled
                    if let Some((_node_id, ack)) = next_ack(AckMsg::new(traces_pdata)) {
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
        telemetry: (flume::Receiver<MetricSetSnapshot>, MetricsReporter),
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
                        let mut metrics_records = create_otap_batch(batch_id, ArrowPayloadType::UnivariateMetrics);
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
                    3,
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
                    3,
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
                    3,
                    "traces",
                )
                .await;

                assert_receiver_telemetry(&ctx, telemetry, Some("refused")).await;

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
                    if let Some((_node_id, nack)) = next_nack(nack) {
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
                    if let Some((_node_id, nack)) = next_nack(nack) {
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
                    if let Some((_node_id, nack)) = next_nack(nack) {
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
        expected_batch_count: i64,
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
            (0..expected_batch_count).collect::<HashSet<_>>(),
            "Did not receive responses for all expected batch IDs in {}. Got: {:?}",
            signal_name,
            received_batch_ids
        );
    }

    async fn assert_receiver_telemetry(
        ctx: &TestContext<OtapPdata>,
        (metrics_rx, metrics_reporter): (flume::Receiver<MetricSetSnapshot>, MetricsReporter),
        acknowledgement_outcome: Option<&str>,
    ) {
        ctx.send_control_msg(NodeControlMsg::CollectTelemetry { metrics_reporter })
            .await
            .expect("receiver should accept telemetry collection");

        let expected_snapshots = if acknowledgement_outcome.is_some() {
            6
        } else {
            3
        };
        let mut batch_signals = HashSet::new();
        let mut acknowledgement_signals = HashSet::new();
        for _ in 0..expected_snapshots {
            let snapshot = timeout(Duration::from_secs(3), metrics_rx.recv_async())
                .await
                .expect("timed out collecting OTAP receiver telemetry")
                .expect("OTAP receiver telemetry channel closed");
            match snapshot.descriptor().name {
                "receiver.otap.batches" => {
                    let signal = snapshot
                        .measurement_attribute_value("signal")
                        .expect("batch metrics should carry signal");
                    assert!(
                        batch_signals.insert(signal),
                        "batch metrics should emit once per signal"
                    );
                    assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), 3);
                    assert_eq!(snapshot.get_metrics()[1].to_u64_lossy(), 3);
                    assert!(
                        snapshot.get_metrics()[2].to_u64_lossy() > 0,
                        "admitted OTAP payload bytes should be positive"
                    );
                }
                "receiver.otap.acknowledgements" => {
                    let signal = snapshot
                        .measurement_attribute_value("signal")
                        .expect("acknowledgement metrics should carry signal");
                    assert!(
                        acknowledgement_signals.insert(signal),
                        "acknowledgement metrics should emit once per signal"
                    );
                    assert_eq!(
                        snapshot.measurement_attribute_value("outcome"),
                        acknowledgement_outcome
                    );
                    assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), 3);
                }
                name => panic!("unexpected OTAP receiver metric set: {name}"),
            }
        }

        assert_eq!(batch_signals, HashSet::from(["logs", "metrics", "traces"]));
        if acknowledgement_outcome.is_some() {
            assert_eq!(
                acknowledgement_signals,
                HashSet::from(["logs", "metrics", "traces"])
            );
        } else {
            assert!(acknowledgement_signals.is_empty());
        }
    }

    async fn assert_rejection_telemetry(
        ctx: &TestContext<OtapPdata>,
        (metrics_rx, metrics_reporter): (flume::Receiver<MetricSetSnapshot>, MetricsReporter),
        error_type: &str,
        expected_streams: u64,
        expected_batches: u64,
    ) {
        ctx.send_control_msg(NodeControlMsg::CollectTelemetry { metrics_reporter })
            .await
            .expect("receiver should accept rejection telemetry collection");
        let snapshot = timeout(Duration::from_secs(3), metrics_rx.recv_async())
            .await
            .expect("timed out collecting OTAP rejection telemetry")
            .expect("OTAP rejection telemetry channel closed");
        assert_eq!(snapshot.descriptor().name, "receiver.otap.rejections");
        assert_eq!(
            snapshot.measurement_attribute_value("error.type"),
            Some(error_type)
        );
        assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), expected_streams);
        assert_eq!(snapshot.get_metrics()[1].to_u64_lossy(), expected_batches);
        assert!(
            metrics_rx.try_recv().is_err(),
            "a pre-admission rejection must not emit a batch lifecycle snapshot"
        );
    }

    async fn assert_single_batch_lifecycle_telemetry(
        ctx: &TestContext<OtapPdata>,
        (metrics_rx, metrics_reporter): (flume::Receiver<MetricSetSnapshot>, MetricsReporter),
    ) {
        ctx.send_control_msg(NodeControlMsg::CollectTelemetry { metrics_reporter })
            .await
            .expect("receiver should accept lifecycle telemetry collection");
        let snapshot = timeout(Duration::from_secs(3), metrics_rx.recv_async())
            .await
            .expect("timed out collecting OTAP lifecycle telemetry")
            .expect("OTAP lifecycle telemetry channel closed");
        assert_eq!(snapshot.descriptor().name, "receiver.otap.batches");
        assert_eq!(snapshot.measurement_attribute_value("signal"), Some("logs"));
        assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), 1);
        assert_eq!(snapshot.get_metrics()[1].to_u64_lossy(), 1);
        assert!(snapshot.get_metrics()[2].to_u64_lossy() > 0);
        assert!(
            metrics_rx.try_recv().is_err(),
            "pipeline send failure is lifecycle termination, not a rejection"
        );
    }

    async fn assert_concurrency_rejection_telemetry(
        ctx: &TestContext<OtapPdata>,
        (metrics_rx, metrics_reporter): (flume::Receiver<MetricSetSnapshot>, MetricsReporter),
    ) {
        ctx.send_control_msg(NodeControlMsg::CollectTelemetry { metrics_reporter })
            .await
            .expect("receiver should accept concurrency telemetry collection");
        let mut saw_batch = false;
        let mut saw_ack = false;
        let mut saw_rejection = false;
        for _ in 0..3 {
            let snapshot = timeout(Duration::from_secs(3), metrics_rx.recv_async())
                .await
                .expect("timed out collecting OTAP concurrency telemetry")
                .expect("OTAP concurrency telemetry channel closed");
            match snapshot.descriptor().name {
                "receiver.otap.batches" => {
                    saw_batch = true;
                    assert_eq!(snapshot.measurement_attribute_value("signal"), Some("logs"));
                    assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), 1);
                    assert_eq!(snapshot.get_metrics()[1].to_u64_lossy(), 1);
                    assert!(snapshot.get_metrics()[2].to_u64_lossy() > 0);
                }
                "receiver.otap.acknowledgements" => {
                    saw_ack = true;
                    assert_eq!(snapshot.measurement_attribute_value("signal"), Some("logs"));
                    assert_eq!(
                        snapshot.measurement_attribute_value("outcome"),
                        Some("success")
                    );
                    assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), 1);
                }
                "receiver.otap.rejections" => {
                    saw_rejection = true;
                    assert_eq!(
                        snapshot.measurement_attribute_value("error.type"),
                        Some("concurrency_limit")
                    );
                    assert_eq!(snapshot.get_metrics()[0].to_u64_lossy(), 0);
                    assert_eq!(snapshot.get_metrics()[1].to_u64_lossy(), 1);
                }
                name => panic!("unexpected concurrency metric set: {name}"),
            }
        }
        assert!(saw_batch && saw_ack && saw_rejection);
    }

    /// Scenario: Three valid batches per signal use fire-and-forget OTAP delivery.
    /// Guarantees: Lifecycle snapshots complete with positive bytes and no acknowledgement bucket.
    #[test]
    fn test_otap_receiver() {
        let test_runtime = TestRuntime::new();

        // addr and port for the server to run at
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
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
        let telemetry = MetricsReporter::create_new_and_receiver(16);
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint, telemetry))
            .run_validation(validation_procedure());
    }

    /// Scenario: A malformed OTAP logs batch reaches the real receiver stream decoder.
    /// Guarantees: Invalid-request rejection increments once without starting a batch lifecycle.
    #[test]
    fn invalid_batch_emits_rejection_without_lifecycle_metrics() {
        let test_runtime = TestRuntime::new();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));
        let telemetry_registry_handle = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let config = serde_json::json!({
            "listening_addr": addr.to_string(),
            "response_stream_channel_size": 4
        });
        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline_ctx, &config).unwrap(),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );
        let telemetry = MetricsReporter::create_new_and_receiver(4);
        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let mut client = ArrowLogsServiceClient::connect(grpc_endpoint)
                    .await
                    .expect("connect to OTAP receiver");
                let invalid_stream = stream! {
                    yield otap_df_pdata::proto::opentelemetry::arrow::v1::BatchArrowRecords {
                        batch_id: 17,
                        arrow_payloads: vec![
                            otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayload {
                                schema_id: "invalid".to_owned(),
                                r#type: ArrowPayloadType::Logs as i32,
                                record: vec![1, 2, 3],
                            },
                        ],
                        headers: Vec::new(),
                    };
                };
                let mut response = client
                    .arrow_logs(invalid_stream)
                    .await
                    .expect("open invalid OTAP stream")
                    .into_inner();
                assert!(
                    timeout(Duration::from_secs(3), response.message())
                        .await
                        .expect("invalid OTAP stream should terminate")
                        .expect("invalid OTAP stream should close cleanly")
                        .is_none()
                );
                assert_rejection_telemetry(&ctx, telemetry, "invalid_request", 0, 1).await;
                ctx.send_shutdown(Instant::now(), "invalid batch test complete")
                    .await
                    .expect("shutdown OTAP receiver");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(|_ctx| async {});
    }

    /// Scenario: Hard memory pressure rejects a valid OTAP logs stream at ingress.
    /// Guarantees: Memory-pressure stream rejection increments without any batch lifecycle.
    #[test]
    fn memory_pressure_stream_rejection_has_no_batch_lifecycle_metrics() {
        let test_runtime = TestRuntime::new();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));
        let telemetry_registry_handle = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(telemetry_registry_handle);
        let pressure_state = controller_ctx.memory_pressure_state();
        pressure_state.configure(
            otap_df_engine::memory_limiter::MemoryPressureBehaviorConfig {
                retry_after_secs: 1,
                fail_readiness_on_hard: true,
                mode: otap_df_config::policy::MemoryLimiterMode::Enforce,
            },
        );
        pressure_state
            .set_level_for_tests(otap_df_engine::memory_limiter::MemoryPressureLevel::Hard);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let config = serde_json::json!({
            "listening_addr": addr.to_string(),
            "response_stream_channel_size": 4
        });
        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline_ctx, &config).unwrap(),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );
        let telemetry = MetricsReporter::create_new_and_receiver(4);
        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let mut client = ArrowLogsServiceClient::connect(grpc_endpoint)
                    .await
                    .expect("connect to OTAP receiver");
                let valid_stream = stream! {
                    let mut producer = Producer::new();
                    let mut records = create_otap_batch(1, ArrowPayloadType::Logs);
                    yield producer.produce_bar(&mut records).expect("encode valid OTAP batch");
                };
                let status = match client.arrow_logs(valid_stream).await {
                    Err(status) => status,
                    Ok(response) => {
                        timeout(Duration::from_secs(3), response.into_inner().message())
                            .await
                            .expect("memory-pressure stream should respond")
                            .expect_err("memory-pressure stream should return a gRPC error")
                    }
                };
                assert_eq!(status.code(), tonic::Code::ResourceExhausted);
                assert_rejection_telemetry(&ctx, telemetry, "memory_pressure", 1, 0).await;
                ctx.send_shutdown(Instant::now(), "memory pressure test complete")
                    .await
                    .expect("shutdown OTAP receiver");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(|_ctx| async {});
    }

    /// Scenario: A valid OTAP logs batch reaches a closed pipeline output channel.
    /// Guarantees: Its lifecycle starts and completes once without a rejection metric.
    #[test]
    fn pipeline_send_failure_completes_lifecycle_without_rejection() {
        let test_runtime = TestRuntime::new();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));
        let telemetry_registry_handle = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let config = serde_json::json!({
            "listening_addr": addr.to_string(),
            "response_stream_channel_size": 4
        });
        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline_ctx, &config).unwrap(),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );
        let telemetry = MetricsReporter::create_new_and_receiver(4);
        let output_closed = Arc::new(tokio::sync::Notify::new());
        let scenario_output_closed = output_closed.clone();
        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                scenario_output_closed.notified().await;
                let mut client = ArrowLogsServiceClient::connect(grpc_endpoint)
                    .await
                    .expect("connect to OTAP receiver");
                let valid_stream = stream! {
                    let mut producer = Producer::new();
                    let mut records = create_otap_batch(1, ArrowPayloadType::Logs);
                    yield producer.produce_bar(&mut records).expect("encode valid OTAP batch");
                };
                let mut response = client
                    .arrow_logs(valid_stream)
                    .await
                    .expect("open OTAP stream")
                    .into_inner();
                assert!(
                    timeout(Duration::from_secs(3), response.message())
                        .await
                        .expect("failed pipeline stream should terminate")
                        .expect("failed pipeline stream should close cleanly")
                        .is_none()
                );
                assert_single_batch_lifecycle_telemetry(&ctx, telemetry).await;
                ctx.send_shutdown(Instant::now(), "pipeline send failure test complete")
                    .await
                    .expect("shutdown OTAP receiver");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };
        let validation = move |ctx: NotSendValidateContext<OtapPdata>| async move {
            drop(ctx);
            output_closed.notify_one();
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    /// Scenario: A second logs stream arrives while one wait-for-result batch owns the only slot.
    /// Guarantees: The second batch is rejected for concurrency without entering its lifecycle.
    #[test]
    fn concurrency_rejection_excludes_rejected_batch_from_lifecycle() {
        let test_runtime = TestRuntime::new();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let node_config = Arc::new(NodeUserConfig::new_receiver_config(OTAP_RECEIVER_URN));
        let telemetry_registry_handle = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let config = serde_json::json!({
            "listening_addr": addr.to_string(),
            "response_stream_channel_size": 4,
            "max_concurrent_requests": 1,
            "max_concurrent_requests_per_stream": 1,
            "wait_for_result": true
        });
        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline_ctx, &config).unwrap(),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );
        let telemetry = MetricsReporter::create_new_and_receiver(8);
        let first_admitted = Arc::new(tokio::sync::Notify::new());
        let release_first = Arc::new(tokio::sync::Notify::new());
        let scenario_first_admitted = first_admitted.clone();
        let scenario_release_first = release_first.clone();
        let scenario = move |ctx: TestContext<OtapPdata>| {
            Box::pin(async move {
                let first_endpoint = grpc_endpoint.clone();
                let first_request = tokio::spawn(async move {
                    let mut client = ArrowLogsServiceClient::connect(first_endpoint)
                        .await
                        .expect("connect first OTAP client");
                    let first_stream = stream! {
                        let mut producer = Producer::new();
                        let mut records = create_otap_batch(1, ArrowPayloadType::Logs);
                        yield producer.produce_bar(&mut records).expect("encode first OTAP batch");
                    };
                    let mut response = client
                        .arrow_logs(first_stream)
                        .await
                        .expect("open first OTAP stream")
                        .into_inner();
                    response
                        .message()
                        .await
                        .expect("first OTAP response should decode")
                        .expect("first OTAP response should be present")
                });

                timeout(Duration::from_secs(3), scenario_first_admitted.notified())
                    .await
                    .expect("first batch should reach the pipeline");
                let mut second_client = ArrowLogsServiceClient::connect(grpc_endpoint)
                    .await
                    .expect("connect second OTAP client");
                let second_stream = stream! {
                    let mut producer = Producer::new();
                    let mut records = create_otap_batch(2, ArrowPayloadType::Logs);
                    yield producer.produce_bar(&mut records).expect("encode second OTAP batch");
                };
                let second_status = timeout(
                    Duration::from_secs(3),
                    second_client
                        .arrow_logs(second_stream)
                        .await
                        .expect("open second OTAP stream")
                        .into_inner()
                        .message(),
                )
                .await
                .expect("second stream should receive concurrency response")
                .expect("second concurrency response should decode")
                .expect("second concurrency response should be present");
                assert_eq!(second_status.status_code, 14);
                assert!(
                    second_status
                        .status_message
                        .contains("Too many concurrent requests")
                );

                scenario_release_first.notify_one();
                let first_status = timeout(Duration::from_secs(3), first_request)
                    .await
                    .expect("first request should complete after ACK")
                    .expect("first request task should not panic");
                assert_eq!(first_status.status_code, 0);

                assert_concurrency_rejection_telemetry(&ctx, telemetry).await;
                ctx.send_shutdown(Instant::now(), "concurrency test complete")
                    .await
                    .expect("shutdown OTAP receiver");
            }) as Pin<Box<dyn Future<Output = ()>>>
        };
        let validation = move |mut ctx: NotSendValidateContext<OtapPdata>| async move {
            let pdata = timeout(Duration::from_secs(3), ctx.recv())
                .await
                .expect("first batch should reach validation")
                .expect("first batch should be present");
            first_admitted.notify_one();
            release_first.notified().await;
            let (_, ack) = next_ack(AckMsg::new(pdata)).expect("first batch should await ACK");
            ctx.send_control_msg(NodeControlMsg::Ack(ack))
                .await
                .expect("send first batch ACK");
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }

    #[test]
    fn test_config_parsing() {
        use otap_df_otap::compression::CompressionMethod;
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
        assert_eq!(receiver.config.max_concurrent_requests_per_stream.get(), 16);
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
        assert_eq!(receiver.config.max_concurrent_requests_per_stream.get(), 16);
        assert!(!receiver.config.wait_for_result);
        assert!(receiver.config.compression_method.is_none());
        assert!(receiver.config.timeout.is_none());

        // Test with full configuration including gzip compression
        let config_full_gzip = json!({
            "listening_addr": "127.0.0.1:4319",
            "response_stream_channel_size": 150,
            "compression_method": "gzip",
            "max_concurrent_requests": 2500,
            "max_concurrent_requests_per_stream": 32,
            "wait_for_result": true,
            "timeout": "30s"
        });
        let receiver = OTAPReceiver::from_config(pipeline_ctx.clone(), &config_full_gzip).unwrap();
        assert_eq!(receiver.config.listening_addr.to_string(), "127.0.0.1:4319");
        assert_eq!(receiver.config.response_stream_channel_size, 150);
        assert_eq!(receiver.config.max_concurrent_requests, 2500);
        assert_eq!(receiver.config.max_concurrent_requests_per_stream.get(), 32);
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
        let receiver =
            OTAPReceiver::from_config(pipeline_ctx.clone(), &config_with_deflate).unwrap();
        assert_eq!(receiver.config.listening_addr.to_string(), "127.0.0.1:4321");
        assert_eq!(receiver.config.response_stream_channel_size, 75);
        assert!(matches!(
            receiver.config.compression_method,
            Some(CompressionMethod::Deflate)
        ));
        assert!(receiver.config.timeout.is_none());

        let config_with_zero_per_stream_limit = json!({
            "listening_addr": "127.0.0.1:4322",
            "response_stream_channel_size": 75,
            "max_concurrent_requests_per_stream": 0
        });
        let err = match OTAPReceiver::from_config(pipeline_ctx, &config_with_zero_per_stream_limit)
        {
            Ok(_) => panic!("zero per-stream in-flight limit should be rejected"),
            Err(err) => err,
        };
        assert!(
            format!("{err}").contains("max_concurrent_requests_per_stream must be greater than 0")
        );
    }

    /// Scenario: OTAP batch lifecycles, acknowledgements, and rejections span bounded dimensions.
    /// Guarantees: Counts remain isolated by signal, outcome, rejection scope, and error type.
    #[test]
    fn receiver_metrics_are_partitioned_by_context() {
        use serde_json::json;

        let telemetry_registry_handle = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let config = json!({
            "listening_addr": "127.0.0.1:4317",
            "response_stream_channel_size": 100
        });
        let receiver = OTAPReceiver::from_config(pipeline_ctx, &config).unwrap();

        receiver
            .metrics
            .record_rejection(ReceiverRejectionErrorType::MemoryPressure);
        receiver
            .metrics
            .record_item_rejection(ReceiverRejectionErrorType::ConcurrencyLimit);
        receiver
            .metrics
            .lock()
            .record_acknowledgement(SignalType::Logs, Outcome::Success);
        receiver
            .metrics
            .lock()
            .record_batch_admitted(SignalType::Logs, 42);
        receiver
            .metrics
            .lock()
            .record_batch_completed(SignalType::Logs);

        let metrics = receiver.metrics.lock();
        assert_eq!(metrics.batches_for(SignalType::Logs).started.get(), 1);
        assert_eq!(metrics.batches_for(SignalType::Logs).completed.get(), 1);
        assert_eq!(metrics.batches_for(SignalType::Logs).payload_size.get(), 42);
        assert_eq!(metrics.batches_for(SignalType::Metrics).started.get(), 0);
        assert_eq!(
            metrics
                .rejections_for(ReceiverRejectionErrorType::MemoryPressure)
                .streams
                .get(),
            1
        );
        assert_eq!(
            metrics
                .rejections_for(ReceiverRejectionErrorType::MemoryPressure)
                .batches
                .get(),
            0
        );
        assert_eq!(
            metrics
                .rejections_for(ReceiverRejectionErrorType::ConcurrencyLimit)
                .batches
                .get(),
            1
        );
        assert_eq!(
            metrics
                .acknowledgements_for(SignalType::Logs, Outcome::Success)
                .responses
                .get(),
            1
        );
        assert_eq!(
            metrics
                .acknowledgements_for(SignalType::Metrics, Outcome::Success)
                .responses
                .get(),
            0
        );
    }

    /// Scenario: OTAP receiver metric sets are transferred into terminal snapshots twice.
    /// Guarantees: Touched lifecycle, acknowledgement, and rejection buckets emit once.
    #[test]
    fn terminal_snapshots_preserve_enum_attribute_values_once() {
        let telemetry_registry_handle = otap_df_telemetry::registry::TelemetryRegistryHandle::new();
        let controller_ctx =
            otap_df_engine::context::ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut metrics = OtapReceiverMetrics::register(&pipeline_ctx);

        metrics.record_batch_admitted(SignalType::Traces, 64);
        metrics.record_batch_completed(SignalType::Traces);
        metrics.record_acknowledgement(SignalType::Traces, Outcome::Refused);
        metrics.record_batch_rejection(ReceiverRejectionErrorType::InvalidRequest);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 3);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.otap.batches"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.otap.acknowledgements"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot.measurement_attribute_value("outcome") == Some("refused")
        }));
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "receiver.otap.rejections"
                && snapshot.measurement_attribute_value("error.type") == Some("invalid_request")
        }));
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: Three valid batches per signal are ACKed through real OTAP gRPC streams.
    /// Guarantees: Lifecycle and successful acknowledgement snapshots match every admitted batch.
    #[test]
    fn test_otap_receiver_ack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
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

        let telemetry = MetricsReporter::create_new_and_receiver(16);
        test_runtime
            .set_receiver(receiver)
            .run_test(scenario(grpc_endpoint, telemetry))
            .run_validation_concurrent(validation_procedure());
    }

    /// Scenario: Three valid batches per signal are NACKed through real OTAP gRPC streams.
    /// Guarantees: Lifecycle and refused acknowledgement snapshots match every admitted batch.
    #[test]
    fn test_otap_receiver_nack() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
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

        let telemetry = MetricsReporter::create_new_and_receiver(16);
        test_runtime
            .set_receiver(receiver)
            .run_test(nack_scenario(grpc_endpoint, telemetry)) // Use NACK-specific scenario
            .run_validation_concurrent(nack_validation_procedure()); // Use NACK-specific validation
    }

    // When wait_for_result is enabled, shutdown must resolve every in-flight batch
    // response stream to an explicit unavailable status instead of leaving the OTAP
    // client hanging on the server-side wait path.
    #[test]
    fn test_otap_receiver_shutdown_completes_inflight_waits() {
        let test_runtime = TestRuntime::new();

        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
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
            "wait_for_result": true
        });

        let receiver = ReceiverWrapper::shared(
            OTAPReceiver::from_config(pipeline_ctx, &config).unwrap(),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let request_started = Arc::new(tokio::sync::Notify::new());
        let scenario_started = request_started.clone();
        let validation_started = request_started.clone();
        let shutdown_reason = "shutdown while OTAP batches are waiting";

        let scenario = move |ctx: TestContext<OtapPdata>| {
            let request_started = scenario_started.clone();
            Box::pin(async move {
                let request_handle = tokio::spawn(async move {
                    let mut client = ArrowLogsServiceClient::connect(grpc_endpoint.clone())
                        .await
                        .expect("connect otap receiver");

                    #[allow(tail_expr_drop_order)]
                    let logs_stream = stream! {
                        let mut producer = Producer::new();
                        for batch_id in 0..1 {
                            let mut logs_records = create_otap_batch(batch_id, ArrowPayloadType::Logs);
                            let bar = producer.produce_bar(&mut logs_records).unwrap();
                            yield bar;
                        }
                    };

                    let response = client
                        .arrow_logs(logs_stream)
                        .await
                        .expect("arrow_logs request should succeed");

                    validate_batch_responses(
                        response.into_inner(),
                        14,
                        &format!("Pipeline processing failed: {shutdown_reason}"),
                        1,
                        "logs",
                    )
                    .await;
                });

                timeout(Duration::from_secs(3), request_started.notified())
                    .await
                    .expect("timed out waiting for OTAP request to reach the pipeline");

                ctx.send_shutdown(Instant::now() + Duration::from_secs(1), shutdown_reason)
                    .await
                    .expect("failed to send shutdown");

                timeout(Duration::from_secs(3), request_handle)
                    .await
                    .expect("OTAP wait_for_result request should complete on shutdown")
                    .unwrap();
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        let validation = move |mut ctx: NotSendValidateContext<OtapPdata>| {
            let request_started = validation_started.clone();
            Box::pin(async move {
                let _pdata = timeout(Duration::from_secs(3), ctx.recv())
                    .await
                    .expect("timed out waiting for OTAP pdata")
                    .expect("no OTAP pdata received");
                request_started.notify_one();
            }) as Pin<Box<dyn Future<Output = ()>>>
        };

        test_runtime
            .set_receiver(receiver)
            .run_test(scenario)
            .run_validation_concurrent(validation);
    }
}
