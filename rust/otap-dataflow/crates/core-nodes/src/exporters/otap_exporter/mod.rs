// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the OTAP exporter node
//!
//! ToDo: Handle configuration changes
//! ToDo: Implement proper deadline function for Shutdown ctrl msg

otap_df_telemetry::otel_component_scope!(urn = OTAP_EXPORTER_URN, target = "otel.exporter.otap",);

use async_stream::stream;
use async_trait::async_trait;
use futures::stream::{FuturesUnordered, StreamExt};
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NackMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ExporterErrorKind, format_error_sources};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_pdata::Producer;
use otap_df_pdata::TryIntoWithOptions;
use otap_df_pdata::encode::producer::ProducerOptions;
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::{BatchArrowRecords, BatchStatus, StatusCode};
use otap_df_pdata::proto::opentelemetry::arrow::v1::{
    arrow_logs_service_client::ArrowLogsServiceClient,
    arrow_metrics_service_client::ArrowMetricsServiceClient,
    arrow_traces_service_client::ArrowTracesServiceClient,
};
use otap_df_telemetry::common_attributes::SignalAttributes;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::HistogramNormal;
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSetSnapshot};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use parking_lot::Mutex;
use serde_json::Value;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::task::JoinHandle;
use tonic::metadata::MetadataMap;
use tonic::transport::Channel;
use tonic::{IntoStreamingRequest, Response, Status, Streaming};

/// The URN for the OTAP exporter
pub const OTAP_EXPORTER_URN: &str = "urn:otel:exporter:otap";

pub mod config;
mod metrics;
use config::Config;
use metrics::{OtapExporterErrorType, OtapExporterMetrics as OtapExporterTerminalMetrics};

/// Exporter that sends OTAP data via gRPC
pub struct OTAPExporter {
    config: Config,
    metrics: OtapExporterTerminalMetrics,
    stream_metrics: OtapExporterStreamMetricSets,
}

struct StreamBatch {
    pdata: OtapPdata,
    records: OtapArrowRecords,
    export_started_at: Instant,
}

/// OTAP stream work partitioned by signal.
#[metric_set(
    name = "exporter.otap.streams",
    measurement_attributes = SignalAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct OtapExporterStreamMetrics {
    /// Time spent waiting to enqueue a batch into the per-signal stream task.
    #[metric(name = "enqueue.duration", unit = "s")]
    pub enqueue_duration_seconds: HistogramNormal,
    /// Occupancy of the per-signal stream task queue before enqueueing a batch.
    #[metric(name = "enqueue.depth", unit = "{batch}")]
    pub enqueue_depth: HistogramNormal,
    /// Time spent encoding an OTAP batch into outbound Arrow batch records.
    #[metric(name = "encode.duration", unit = "s")]
    pub encode_duration_seconds: HistogramNormal,
    /// Time spent enqueueing a yielded batch into the response correlation queue.
    #[metric(name = "correlation.enqueue.duration", unit = "s")]
    pub correlation_enqueue_duration_seconds: HistogramNormal,
    /// Occupancy of the response correlation queue before enqueueing a yielded batch.
    #[metric(name = "correlation.depth", unit = "{batch}")]
    pub correlation_depth: HistogramNormal,
    /// Time spent waiting for the next server response on an OTAP stream.
    #[metric(name = "response.wait.duration", unit = "s")]
    pub response_wait_duration_seconds: HistogramNormal,
    /// Number of yielded batches actively awaiting a matching server response.
    #[metric(name = "response.active", unit = "{batch}")]
    pub response_active: HistogramNormal,
}

/// Fixed-memory timing aggregation owned by one OTAP stream worker.
#[derive(Debug, Default)]
struct OtapStreamWorkerMetrics {
    encode_duration_seconds: HistogramNormal,
    correlation_enqueue_duration_seconds: HistogramNormal,
    correlation_depth: HistogramNormal,
    response_wait_duration_seconds: HistogramNormal,
    response_active: HistogramNormal,
}

/// Request-side metrics captured by Tonic's mandatory `Send` request stream.
#[derive(Debug, Default)]
struct OtapRequestStreamMetrics {
    encode_duration_seconds: HistogramNormal,
    correlation_enqueue_duration_seconds: HistogramNormal,
    correlation_depth: HistogramNormal,
}

/// Tonic requires every outbound streaming request to be `Send`, even though
/// the owning worker runs with `spawn_local`. Keep synchronization confined to
/// the metrics captured inside that request stream; all other worker metrics
/// remain pipeline-local in `Rc<RefCell<_>>`.
#[derive(Debug, Clone)]
struct OtapRequestStreamMetricsHandle {
    metrics: Arc<Mutex<OtapRequestStreamMetrics>>,
}

impl OtapRequestStreamMetricsHandle {
    fn new() -> Self {
        Self {
            metrics: Arc::new(Mutex::new(OtapRequestStreamMetrics::default())),
        }
    }

    fn record_encode(&self, duration_seconds: f64) {
        self.metrics
            .lock()
            .encode_duration_seconds
            .record(duration_seconds);
    }

    fn record_correlation_enqueue(&self, duration_seconds: f64, depth: usize) {
        let mut metrics = self.metrics.lock();
        metrics
            .correlation_enqueue_duration_seconds
            .record(duration_seconds);
        metrics.correlation_depth.record(depth as f64);
    }

    fn take(&self) -> OtapRequestStreamMetrics {
        std::mem::take(&mut *self.metrics.lock())
    }
}

/// Pipeline-local handle used to record and collect one stream worker's metrics.
#[derive(Debug, Clone)]
struct OtapStreamWorkerMetricsHandle {
    signal: SignalType,
    metrics: Rc<RefCell<OtapStreamWorkerMetrics>>,
    request_metrics: OtapRequestStreamMetricsHandle,
}

impl OtapStreamWorkerMetricsHandle {
    fn new(signal: SignalType) -> Self {
        Self {
            signal,
            metrics: Rc::new(RefCell::new(OtapStreamWorkerMetrics::default())),
            request_metrics: OtapRequestStreamMetricsHandle::new(),
        }
    }

    #[cfg(test)]
    fn record_encode(&self, duration_seconds: f64) {
        self.request_metrics.record_encode(duration_seconds);
    }

    #[cfg(test)]
    fn record_correlation_enqueue(&self, duration_seconds: f64, depth: usize) {
        self.request_metrics
            .record_correlation_enqueue(duration_seconds, depth);
    }

    fn record_response_wait(&self, duration_seconds: f64, active: usize) {
        let mut metrics = self.metrics.borrow_mut();
        metrics
            .response_wait_duration_seconds
            .record(duration_seconds);
        metrics.response_active.record(active as f64);
    }

    fn take(&self) -> OtapStreamWorkerMetrics {
        let mut metrics = self.metrics.take();
        let request_metrics = self.request_metrics.take();
        metrics
            .encode_duration_seconds
            .merge(request_metrics.encode_duration_seconds);
        metrics
            .correlation_enqueue_duration_seconds
            .merge(request_metrics.correlation_enqueue_duration_seconds);
        metrics
            .correlation_depth
            .merge(request_metrics.correlation_depth);
        metrics
    }

    fn request_metrics(&self) -> OtapRequestStreamMetricsHandle {
        self.request_metrics.clone()
    }
}

#[inline]
fn elapsed_seconds(start: Instant) -> f64 {
    start.elapsed().as_secs_f64()
}

/// Bounded-cardinality OTAP exporter metrics tracker.
#[derive(Debug)]
struct OtapExporterStreamMetricSets {
    streams: MeasurementMetricSet<OtapExporterStreamMetrics>,
}

impl OtapExporterStreamMetricSets {
    fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            streams: OtapExporterStreamMetrics::register(pipeline_ctx),
        }
    }

    fn record_stream_enqueue(&mut self, signal: SignalType, duration_seconds: f64, depth: usize) {
        let metrics = self.streams.with(SignalAttributes { signal });
        metrics.enqueue_duration_seconds.record(duration_seconds);
        metrics.enqueue_depth.record(depth as f64);
    }

    fn merge_stream_worker_metrics(&mut self, workers: &[OtapStreamWorkerMetricsHandle]) {
        for worker in workers {
            let worker_metrics = worker.take();
            let metrics = self.streams.with(SignalAttributes {
                signal: worker.signal,
            });
            metrics
                .encode_duration_seconds
                .merge(worker_metrics.encode_duration_seconds);
            metrics
                .correlation_enqueue_duration_seconds
                .merge(worker_metrics.correlation_enqueue_duration_seconds);
            metrics
                .correlation_depth
                .merge(worker_metrics.correlation_depth);
            metrics
                .response_wait_duration_seconds
                .merge(worker_metrics.response_wait_duration_seconds);
            metrics
                .response_active
                .merge(worker_metrics.response_active);
        }
    }

    fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        reporter.report_measurement(&mut self.streams)
    }

    fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        self.streams.terminal_snapshots()
    }

    #[cfg(test)]
    fn streams_for(&self, signal: SignalType) -> &OtapExporterStreamMetrics {
        self.streams.get(SignalAttributes { signal })
    }
}

/// Declares the OTAP exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static OTAP_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTAP_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ExporterWrapper::local(
            OTAPExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config,
};

/// Validates the OTAP exporter configuration at config load time.
///
/// Runs before any node is started (initial load and live reconfigure), so bad
/// configuration is rejected fast and attributed to the offending node rather
/// than surfacing as an opaque client error at startup.
fn validate_config(config: &Value) -> Result<(), otap_df_config::error::Error> {
    let cfg: Config = serde_json::from_value(config.clone()).map_err(|e| {
        otap_df_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        }
    })?;
    cfg.grpc
        .validate()
        .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
            error: e.to_string(),
        })?;
    Ok(())
}

enum EnqueueResult {
    Done,
    /// The stream queue was full. The caller should wait for capacity while
    /// continuing to poll the control channel, then retry.
    QueueFull(StreamBatch, Instant, usize),
}

impl OTAPExporter {
    /// Creates a new OTAPExporter
    #[must_use]
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Self {
        let metrics = OtapExporterTerminalMetrics::register(&pipeline_ctx);
        let stream_metrics = OtapExporterStreamMetricSets::register(&pipeline_ctx);
        OTAPExporter {
            config,
            metrics,
            stream_metrics,
        }
    }

    /// Creates a new OTAPExporter from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        // Defense-in-depth: validate the shared gRPC settings (including the
        // ASCII/reserved/duplicate `headers` checks) here too, not only in the
        // factory `validate_config` hook. This guarantees that any construction
        // path (including direct/programmatic `from_config` callers) rejects
        // invalid or gRPC-reserved headers up front, so `start()` can never
        // build stream metadata from an unvalidated config.
        config
            .grpc
            .validate()
            .map_err(|e| otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?;

        Ok(OTAPExporter::new(pipeline_ctx, config))
    }

    async fn handle_pdata_metrics_update(
        &mut self,
        update: PDataMetricsUpdate,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        match update {
            PDataMetricsUpdate::IncFailed(signal_type, pdata, export_duration, error_type) => {
                self.metrics
                    .record_failure(signal_type, error_type, export_duration);
                effect_handler
                    .notify_nack(NackMsg::new("export failed", pdata))
                    .await?;
            }
            PDataMetricsUpdate::IncExported(signal_type, pdata, export_duration) => {
                self.metrics.record_success(signal_type, export_duration);
                effect_handler.notify_ack(AckMsg::new(pdata)).await?;
            }
        }
        Ok(())
    }

    async fn enqueue_stream_batch(
        &mut self,
        sender: &Sender<StreamBatch>,
        signal: SignalType,
        pdata: OtapPdata,
        message: OtapArrowRecords,
        export_started_at: Instant,
    ) -> Result<EnqueueResult, Error> {
        let queue_depth = sender.max_capacity() - sender.capacity();
        let enqueue_start = Instant::now();

        match sender.try_send(StreamBatch {
            pdata,
            records: message,
            export_started_at,
        }) {
            Ok(()) => {
                self.stream_metrics.record_stream_enqueue(
                    signal,
                    elapsed_seconds(enqueue_start),
                    queue_depth,
                );
                Ok(EnqueueResult::Done)
            }
            Err(tokio::sync::mpsc::error::TrySendError::Full(item)) => {
                // Queue is full -- return to caller so it can wait for capacity
                // while still polling the control channel in the main select.
                Ok(EnqueueResult::QueueFull(item, enqueue_start, queue_depth))
            }
            Err(tokio::sync::mpsc::error::TrySendError::Closed(_)) => {
                self.stream_metrics.record_stream_enqueue(
                    signal,
                    elapsed_seconds(enqueue_start),
                    queue_depth,
                );
                Ok(EnqueueResult::Done)
            }
        }
    }

    async fn drain_pdata_metrics_updates(
        &mut self,
        pdata_metrics_rx: &mut Receiver<PDataMetricsUpdate>,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        while let Ok(update) = pdata_metrics_rx.try_recv() {
            self.handle_pdata_metrics_update(update, effect_handler)
                .await?;
        }
        Ok(())
    }

    async fn await_stream_handles_and_drain_metrics(
        &mut self,
        handles: Vec<JoinHandle<()>>,
        pdata_metrics_rx: &mut Receiver<PDataMetricsUpdate>,
        effect_handler: &local::EffectHandler<OtapPdata>,
    ) -> Result<(), Error> {
        let mut handles = handles.into_iter().collect::<FuturesUnordered<_>>();

        while !handles.is_empty() {
            tokio::select! {
                _ = handles.next() => {}
                update = pdata_metrics_rx.recv() => {
                    let Some(update) = update else {
                        break;
                    };
                    self.handle_pdata_metrics_update(update, effect_handler).await?;
                }
            }
        }

        self.drain_pdata_metrics_updates(pdata_metrics_rx, effect_handler)
            .await
    }
}

/// Implement the local exporter trait for a OTAP Exporter
#[async_trait(?Send)]
impl local::Exporter<OtapPdata> for OTAPExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!(
            "otap_exporter.start",
            grpc_endpoint = self.config.grpc.grpc_endpoint.as_str(),
            message = "Starting OTAP Exporter"
        );

        let exporter_id = effect_handler.exporter_id();

        // Run the optional startup check (dns resolution or eager connect) before creating the
        // lazy channel used for normal runtime traffic.
        self.config.grpc.run_startup_check().await.map_err(|e| {
            let source_detail = format_error_sources(&e);
            Error::ExporterError {
                exporter: exporter_id.clone(),
                kind: ExporterErrorKind::Connect,
                error: format!("startup check failed: {e}"),
                source_detail,
            }
        })?;

        let channel = self
            .config
            .grpc
            .connect_channel_lazy(self.config.timeout)
            .await
            .map_err(|e| {
                let source_detail = format_error_sources(&e);
                Error::ExporterError {
                    exporter: exporter_id,
                    kind: ExporterErrorKind::Connect,
                    error: format!("grpc channel error {e}"),
                    source_detail,
                }
            })?;

        // start a grpc client and connect to the server
        let mut arrow_metrics_client = ArrowMetricsServiceClient::new(channel.clone());
        let mut arrow_logs_client = ArrowLogsServiceClient::new(channel.clone());
        let mut arrow_traces_client = ArrowTracesServiceClient::new(channel.clone());

        if let Some(ref compression) = self.config.compression_method {
            let encoding = compression.map_to_compression_encoding();
            arrow_logs_client = arrow_logs_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
            arrow_metrics_client = arrow_metrics_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
            arrow_traces_client = arrow_traces_client
                .send_compressed(encoding)
                .accept_compressed(encoding);
        }

        // Each signal type is exported through one or more long-lived stream
        // workers. The exporter task only converts incoming pdata and enqueues
        // it into a bounded per-stream queue; the worker owns the gRPC request
        // stream and response correlation for that queue.
        //
        // Keeping these queues bounded preserves backpressure. Increasing
        // `streams_per_signal` adds independently progressing gRPC streams
        // instead of hiding pressure behind a deeper single queue.
        let stream_queue_capacity = self.config.stream_queue_capacity;
        let streams_per_signal = self.config.streams_per_signal;
        // This channel carries only terminal pdata outcomes and uses awaited
        // sends so ACK/NACK delivery cannot be dropped. High-frequency stream
        // timings stay in fixed-memory per-worker aggregators and are merged
        // during collection instead of competing for this bounded channel.
        let (pdata_metrics_tx, mut pdata_metrics_rx) = tokio::sync::mpsc::channel(64);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        let ipc_compression = matches!(
            self.config.arrow.payload_compression,
            Some(config::ArrowPayloadCompression::Zstd)
        )
        .then(|| arrow_ipc::CompressionType::ZSTD);

        // Build the static request-header metadata template ONCE, outside any
        // hot path. `None` when no `headers` are configured, which keeps the
        // stream-open path allocation-free for the common case. The template is
        // shared across all stream workers via `Rc` (the workers run on the same
        // thread-local set, so no atomic refcount is needed); each stream attaches
        // a clone as its initial metadata when it opens (see `stream_arrow_batches`),
        // so the per-`BatchArrowRecords` send path is never touched.
        let static_metadata = self.config.grpc.build_static_metadata().map(Rc::new);

        // Operational breadcrumb: confirm which static header KEYS were loaded
        // (never the values, which may be credentials). Lets an operator verify
        // an auth/tenant header was actually picked up from config when
        // diagnosing a backend auth/routing failure.
        if static_metadata.is_some() {
            let mut header_keys: Vec<&str> = self
                .config
                .grpc
                .headers
                .keys()
                .map(String::as_str)
                .collect();
            header_keys.sort_unstable();
            let header_count = header_keys.len();
            let header_names = header_keys.join(",");
            otel_debug!(
                "otap_exporter.static_headers",
                count = header_count,
                header_names = header_names.as_str(),
                message = "Attaching static headers to OTAP streams"
            );
        }

        // Tonic clients are cheap to clone because they share the underlying
        // Channel. Each clone below is used by exactly one worker, which lets
        // the workers drive separate streaming RPCs concurrently.
        let (logs_senders, logs_handles, logs_worker_metrics) = spawn_stream_workers(
            arrow_logs_client,
            SignalType::Logs,
            ipc_compression,
            stream_queue_capacity,
            streams_per_signal,
            pdata_metrics_tx.clone(),
            shutdown_rx.clone(),
            static_metadata.clone(),
        );
        let (metrics_senders, metrics_handles, metrics_worker_metrics) = spawn_stream_workers(
            arrow_metrics_client,
            SignalType::Metrics,
            ipc_compression,
            stream_queue_capacity,
            streams_per_signal,
            pdata_metrics_tx.clone(),
            shutdown_rx.clone(),
            static_metadata.clone(),
        );
        let (traces_senders, traces_handles, traces_worker_metrics) = spawn_stream_workers(
            arrow_traces_client,
            SignalType::Traces,
            ipc_compression,
            stream_queue_capacity,
            streams_per_signal,
            pdata_metrics_tx.clone(),
            shutdown_rx.clone(),
            static_metadata.clone(),
        );
        let stream_worker_metrics = logs_worker_metrics
            .into_iter()
            .chain(metrics_worker_metrics)
            .chain(traces_worker_metrics)
            .collect::<Vec<_>>();

        // Loop until a Shutdown event is received.
        let mut pending: Option<(Sender<StreamBatch>, StreamBatch, Instant, SignalType, usize)> =
            None;
        loop {
            let pending_sender_inner = pending.as_ref().map(|(sender, _, _, _, _)| sender.clone());
            let pending_send_promise = match pending_sender_inner.as_ref() {
                Some(sender) => futures::future::Either::Left(sender.reserve()),
                None => futures::future::Either::Right(std::future::pending()),
            };

            tokio::select! {
                permit = pending_send_promise => {
                    match permit {
                        Ok(permit) => {
                            let (_, item, enqueue_start, signal, queue_depth) =
                                pending.take().expect("pending batch retained");
                            self.stream_metrics.record_stream_enqueue(
                                signal,
                                elapsed_seconds(enqueue_start),
                                queue_depth,
                            );
                            permit.send(item);
                        }
                        Err(_) => {
                            let (_, _, enqueue_start, signal, queue_depth) =
                                pending.take().expect("pending batch retained");
                            self.stream_metrics.record_stream_enqueue(
                                signal,
                                elapsed_seconds(enqueue_start),
                                queue_depth,
                            );
                        }
                    }
                }
                msg = msg_chan.recv_when(pending.is_none()) => match msg? {
                    // handle control messages
                    Message::Control(NodeControlMsg::TimerTick { .. })
                    | Message::Control(NodeControlMsg::Config { .. }) => {}
                    Message::Control(NodeControlMsg::CollectTelemetry {
                        mut metrics_reporter,
                    }) => {
                        self.stream_metrics
                            .merge_stream_worker_metrics(&stream_worker_metrics);
                        _ = self.metrics.report(&mut metrics_reporter);
                        _ = self.stream_metrics.report(&mut metrics_reporter);
                    }
                    // shutdown the exporter
                    Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                        // TODO: There is a race condition somewhere,
                        // causing this shutdown message to be never received.
                        // Noticed when load testing
                        // or more easily, when exporter is hitting errors
                        // like endpoint not available. (the backoff sleep
                        // might be causing it?)
                        otel_info!(
                            "otap_exporter.shutdown",
                            message = "OTAP Exporter shutting down"
                        );
                        _ = shutdown_tx.send_replace(true);
                        drop(pdata_metrics_tx);
                        self.await_stream_handles_and_drain_metrics(
                            logs_handles
                                .into_iter()
                                .chain(metrics_handles)
                                .chain(traces_handles)
                                .collect(),
                            &mut pdata_metrics_rx,
                            &effect_handler,
                        )
                        .await?;
                        self.stream_metrics
                            .merge_stream_worker_metrics(&stream_worker_metrics);
                        return Ok(TerminalState::new(
                            deadline,
                            {
                                let mut snapshots = self.metrics.terminal_snapshots();
                                snapshots.extend(self.stream_metrics.terminal_snapshots());
                                snapshots
                            },
                        ))
                    }
                    //send data
                    Message::PData(mut pdata) => {
                        let export_started_at = Instant::now();
                        let signal_type = pdata.signal_type();

                        let payload = pdata.take_payload();

                        let message: OtapArrowRecords = match payload.try_into_with_default() {
                            Ok(m) => m,
                            Err(e) => {
                                self.metrics.record_failure(
                                    signal_type,
                                    OtapExporterErrorType::PayloadConversion,
                                    export_started_at.elapsed(),
                                );
                                effect_handler.notify_nack(NackMsg::new("payload conversion failed", pdata)).await?;
                                return Err(e.into());
                            }
                        };

                        // Route each batch to the stream with the smallest
                        // local backlog. This is intentionally based on queue
                        // occupancy, not response latency: queue depth is the
                        // backpressure signal available before enqueueing.
                        let sender = match signal_type {
                            SignalType::Logs => least_loaded_stream_sender(&logs_senders),
                            SignalType::Metrics => least_loaded_stream_sender(&metrics_senders),
                            SignalType::Traces => least_loaded_stream_sender(&traces_senders),
                        };

                        // Try to enqueue. If the stream queue is full, store the item
                        // as pending. In the next iteration, we will wait for capacity
                        // while continuing to poll the control channel.
                        if let EnqueueResult::QueueFull(item, enqueue_start, queue_depth) = self
                            .enqueue_stream_batch(
                                sender,
                                signal_type,
                                pdata,
                                message,
                                export_started_at,
                            )
                            .await?
                        {
                            pending = Some((
                                sender.clone(),
                                item,
                                enqueue_start,
                                signal_type,
                                queue_depth,
                            ));
                        }
                    }
                    _ => {
                        return Err(Error::ExporterError {
                            exporter: effect_handler.exporter_id(),
                            kind: ExporterErrorKind::Other,
                            error: "Unknown control message".to_owned(),
                            source_detail: "".to_owned()
                        });
                    }
                },
                metrics_update = pdata_metrics_rx.recv() => {
                    if let Some(update) = metrics_update {
                        self.handle_pdata_metrics_update(update, &effect_handler).await?;
                    }
                }
            }
        }
    }
}

/// Starts the per-signal stream worker pool.
///
/// Each worker receives batches through its own bounded queue and turns those
/// batches into one streaming RPC. Multiple workers therefore mean multiple
/// independent HTTP/2 request/response streams for the same signal type. That
/// is the behavior controlled by `streams_per_signal`.
fn spawn_stream_workers<T>(
    client: T,
    signal_type: SignalType,
    ipc_compression: Option<arrow_ipc::CompressionType>,
    stream_queue_capacity: usize,
    streams_per_signal: usize,
    pdata_metrics_tx: Sender<PDataMetricsUpdate>,
    shutdown_rx: tokio::sync::watch::Receiver<bool>,
    static_metadata: Option<Rc<MetadataMap>>,
) -> (
    Vec<Sender<StreamBatch>>,
    Vec<JoinHandle<()>>,
    Vec<OtapStreamWorkerMetricsHandle>,
)
where
    T: StreamingArrowService + Clone + 'static,
{
    let mut senders = Vec::with_capacity(streams_per_signal);
    let mut handles = Vec::with_capacity(streams_per_signal);
    let mut worker_metrics = Vec::with_capacity(streams_per_signal);

    for _ in 0..streams_per_signal {
        // The queue is per stream, not shared across the pool. This keeps
        // backpressure local to the stream that is lagging and gives the
        // exporter a useful depth signal for least-loaded routing.
        let (sender, receiver) = tokio::sync::mpsc::channel::<StreamBatch>(stream_queue_capacity);
        let metrics = OtapStreamWorkerMetricsHandle::new(signal_type);
        senders.push(sender);
        worker_metrics.push(metrics.clone());
        handles.push(tokio::task::spawn_local(stream_arrow_batches(
            client.clone(),
            signal_type,
            ipc_compression,
            receiver,
            pdata_metrics_tx.clone(),
            metrics,
            shutdown_rx.clone(),
            static_metadata.clone(),
        )));
    }

    (senders, handles, worker_metrics)
}

/// Selects the stream queue with the smallest current backlog.
///
/// `tokio::sync::mpsc::Sender` exposes remaining capacity rather than length,
/// so occupancy is computed as `max_capacity - capacity`. The config rejects
/// `streams_per_signal = 0`, which makes the final `expect` an invariant check.
fn least_loaded_stream_sender(senders: &[Sender<StreamBatch>]) -> &Sender<StreamBatch> {
    senders
        .iter()
        .min_by_key(|sender| sender.max_capacity() - sender.capacity())
        .expect("streams_per_signal validation must create at least one stream")
}

#[async_trait]
trait StreamingArrowService {
    async fn handle_req_stream(
        &mut self,
        req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
    ) -> Result<Response<Streaming<BatchStatus>>, Status>;
}

#[async_trait]
impl StreamingArrowService for ArrowLogsServiceClient<Channel> {
    async fn handle_req_stream(
        &mut self,
        req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
    ) -> Result<Response<Streaming<BatchStatus>>, Status> {
        self.arrow_logs(req_stream).await
    }
}

#[async_trait]
impl StreamingArrowService for ArrowMetricsServiceClient<Channel> {
    async fn handle_req_stream(
        &mut self,
        req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
    ) -> Result<Response<Streaming<BatchStatus>>, Status> {
        self.arrow_metrics(req_stream).await
    }
}

#[async_trait]
impl StreamingArrowService for ArrowTracesServiceClient<Channel> {
    async fn handle_req_stream(
        &mut self,
        req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
    ) -> Result<Response<Streaming<BatchStatus>>, Status> {
        self.arrow_traces(req_stream).await
    }
}

enum PDataMetricsUpdate {
    IncExported(SignalType, OtapPdata, Duration),
    IncFailed(SignalType, OtapPdata, Duration, OtapExporterErrorType),
}

struct CorrelatedPdata {
    batch_id: i64,
    pdata: OtapPdata,
    export_started_at: Instant,
}

async fn stream_arrow_batches<T: StreamingArrowService>(
    mut client: T,
    signal_type: SignalType,
    ipc_compression: Option<arrow_ipc::CompressionType>,
    otap_batches_rx: Receiver<StreamBatch>,
    pdata_metrics_tx: Sender<PDataMetricsUpdate>,
    worker_metrics: OtapStreamWorkerMetricsHandle,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    static_metadata: Option<Rc<MetadataMap>>,
) {
    let otap_batches_rx = Arc::new(tokio::sync::Mutex::new(otap_batches_rx));
    let mut shutdown = false;

    // we'll do an exponential backoff if there was an error creating the streaming request
    const MAX_BACKOFF: Duration = Duration::from_secs(10);
    const INITIAL_BACKOFF: Duration = Duration::from_millis(10);
    const BACKOFF_MULTIPLIER: u32 = 2;
    let mut failed_request_backoff = INITIAL_BACKOFF;

    // send streams of batches to the server until shutdown
    while !shutdown {
        let mut rx = otap_batches_rx.lock().await;
        tokio::select! {
            // wait to receive the first batch to create the streaming request
            first_batch = rx.recv() => {
                drop(rx);
                let StreamBatch {
                    pdata: first_pdata,
                    records: first_batch,
                    export_started_at: first_export_started_at,
                } = match first_batch {
                    Some(f) => f,

                    None => {
                        // no more batches
                        break
                    }
                };

                // correlation channel: req_stream sends OtapPdata for each batch yielded,
                // res_stream receives them to pair with server responses.
                let (correlation_tx, mut correlation_rx) = tokio::sync::mpsc::channel::<CorrelatedPdata>(64);

                // Clone first_pdata before moving it into the stream, so we can
                // NACK it if the connection fails before the stream is polled.
                let first_pdata_fallback = first_pdata.clone();

                // create the request stream
                let req_stream = create_req_stream(
                    first_pdata,
                    first_batch,
                    first_export_started_at,
                    otap_batches_rx.clone(),
                    signal_type,
                    ipc_compression,
                    pdata_metrics_tx.clone(),
                    worker_metrics.request_metrics(),
                    correlation_tx.clone(),
                );

                // Attach the configured static `headers` as the stream's initial
                // request metadata. gRPC sends these once when the stream opens,
                // so the cost (one `MetadataMap` clone) is paid per stream
                // (re)connect, never per `BatchArrowRecords`; the per-message
                // hot path in `create_req_stream` is untouched. When no headers
                // are configured this is a cheap `Option` check with no clone.
                //
                // The assignment REPLACES the request's metadata map. That is
                // correct here because `into_streaming_request()` yields a fresh
                // `Request` with empty user metadata and tonic injects transport
                // metadata (content-type, grpc-*, user-agent) below this layer.
                // If a future change starts seeding request metadata before this
                // point, switch to merging into `metadata_mut()` instead.
                let mut req_stream = req_stream.into_streaming_request();
                if let Some(static_metadata) = &static_metadata {
                    *req_stream.metadata_mut() = (**static_metadata).clone();
                }

                let req_fut = client.handle_req_stream(req_stream);
                let connect_res = tokio::select! {
                    res = req_fut => res,
                    _ = shutdown_rx.changed() => {
                        drop(correlation_tx);
                        break;
                    }
                };

                match connect_res {
                    Ok(res) => {
                        // reset the reconnect timeout backoff
                        failed_request_backoff = INITIAL_BACKOFF;

                        // handle server responses until error or shutdown
                        shutdown = handle_res_stream(
                            res.into_inner(),
                            pdata_metrics_tx.clone(),
                            worker_metrics.clone(),
                            signal_type,
                            shutdown_rx.clone(),
                            correlation_rx,
                        ).await;
                    }
                    Err(e) => {
                        let error_type = OtapExporterErrorType::from_grpc_status(&e);
                        // there was an error initiating the streaming request
                        // drain any pdata that was already correlated
                        drop(correlation_tx);
                        fail_stream_open_pdata(
                            &pdata_metrics_tx,
                            signal_type,
                            &mut correlation_rx,
                            first_pdata_fallback,
                            first_export_started_at,
                            error_type,
                        )
                        .await;
                        otel_error!(
                            "otap_exporter.request_failed",
                            message = "Failed to connect, retrying after backoff",
                            error = %e,
                            backoff = ?failed_request_backoff
                        );
                        // Shutdown must preempt reconnect backoff. Otherwise a
                        // failed stream setup can keep the exporter alive until
                        // the current backoff expires, delaying graceful drain
                        // even though the runtime has already requested stop.
                        tokio::select! {
                            _ = tokio::time::sleep(failed_request_backoff) => {}
                            _ = shutdown_rx.changed() => {
                                shutdown = *shutdown_rx.borrow();
                            }
                        }
                        failed_request_backoff = std::cmp::min(failed_request_backoff * BACKOFF_MULTIPLIER, MAX_BACKOFF);
                    }
                };
            }
            _ = shutdown_rx.changed() => {
                 shutdown = *shutdown_rx.borrow();
            }
        }
    }
}

async fn fail_stream_open_pdata(
    pdata_metrics_tx: &Sender<PDataMetricsUpdate>,
    signal_type: SignalType,
    correlation_rx: &mut Receiver<CorrelatedPdata>,
    first_pdata_fallback: OtapPdata,
    first_export_started_at: Instant,
    error_type: OtapExporterErrorType,
) {
    let mut drained = false;
    while let Ok(correlated) = correlation_rx.try_recv() {
        drained = true;
        _ = pdata_metrics_tx
            .send(PDataMetricsUpdate::IncFailed(
                signal_type,
                correlated.pdata,
                correlated.export_started_at.elapsed(),
                error_type,
            ))
            .await;
    }
    if !drained {
        _ = pdata_metrics_tx
            .send(PDataMetricsUpdate::IncFailed(
                signal_type,
                first_pdata_fallback,
                first_export_started_at.elapsed(),
                error_type,
            ))
            .await;
    }
}

#[allow(tail_expr_drop_order)]
fn create_req_stream(
    first_pdata: OtapPdata,
    mut first_batch: OtapArrowRecords,
    first_export_started_at: Instant,
    remaining_batches_rx: Arc<tokio::sync::Mutex<Receiver<StreamBatch>>>,
    signal_type: SignalType,
    ipc_compression: Option<arrow_ipc::CompressionType>,
    pdata_metrics_tx: Sender<PDataMetricsUpdate>,
    request_metrics: OtapRequestStreamMetricsHandle,
    correlation_tx: Sender<CorrelatedPdata>,
) -> impl IntoStreamingRequest<Message = BatchArrowRecords> {
    stream! {
        let mut producer = Producer::new_with_options(ProducerOptions {
            ipc_compression
        });

        // send the first batch
        let encode_start = Instant::now();
        let bar_result = producer.produce_bar(&mut first_batch);
        request_metrics.record_encode(elapsed_seconds(encode_start));
        match bar_result {
            Ok(bar) => {
                let correlation_depth =
                    correlation_tx.max_capacity() - correlation_tx.capacity();
                let correlation_start = Instant::now();
                match correlation_tx.reserve().await {
                    Ok(permit) => {
                        request_metrics.record_correlation_enqueue(
                            elapsed_seconds(correlation_start),
                            correlation_depth,
                        );
                        permit.send(CorrelatedPdata {
                            batch_id: bar.batch_id,
                            pdata: first_pdata,
                            export_started_at: first_export_started_at,
                        });
                        yield bar;
                    }
                    Err(_) => {
                        _ = pdata_metrics_tx
                            .send(PDataMetricsUpdate::IncFailed(
                                signal_type,
                                first_pdata,
                                first_export_started_at.elapsed(),
                                OtapExporterErrorType::Internal,
                            ))
                            .await;
                    }
                }
            }
            Err(_) => {
                _ = pdata_metrics_tx.send(PDataMetricsUpdate::IncFailed(
                    signal_type,
                    first_pdata,
                    first_export_started_at.elapsed(),
                    OtapExporterErrorType::Encoding,
                )).await;
            }
        };

        let mut rx = remaining_batches_rx.lock().await;
        // send the remaining batches
        while let Some(StreamBatch {
            pdata,
            records: mut otap_batch,
            export_started_at,
        }) = rx.recv().await {
            let encode_start = Instant::now();
            let bar_result = producer.produce_bar(&mut otap_batch);
            request_metrics.record_encode(elapsed_seconds(encode_start));
            match bar_result {
                Ok(bar) => {
                    let correlation_depth =
                        correlation_tx.max_capacity() - correlation_tx.capacity();
                    let correlation_start = Instant::now();
                    match correlation_tx.reserve().await {
                        Ok(permit) => {
                            request_metrics.record_correlation_enqueue(
                                elapsed_seconds(correlation_start),
                                correlation_depth,
                            );
                            permit.send(CorrelatedPdata {
                                batch_id: bar.batch_id,
                                pdata,
                                export_started_at,
                            });
                            yield bar;
                        }
                        Err(_) => {
                            _ = pdata_metrics_tx
                                .send(PDataMetricsUpdate::IncFailed(
                                    signal_type,
                                    pdata,
                                    export_started_at.elapsed(),
                                    OtapExporterErrorType::Internal,
                                ))
                                .await;
                        }
                    }
                }
                Err(_) => {
                    _ = pdata_metrics_tx.send(PDataMetricsUpdate::IncFailed(
                        signal_type,
                        pdata,
                        export_started_at.elapsed(),
                        OtapExporterErrorType::Encoding,
                    )).await;
                }
            }
        }
    }
}

async fn handle_res_stream(
    mut res_stream: Streaming<BatchStatus>,
    pdata_metrics_tx: Sender<PDataMetricsUpdate>,
    worker_metrics: OtapStreamWorkerMetricsHandle,
    signal_type: SignalType,
    mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    mut correlation_rx: Receiver<CorrelatedPdata>,
) -> bool {
    let mut shutdown = false;
    let mut correlated_by_batch_id = HashMap::new();

    // handle streaming responses until shutdown
    while !shutdown {
        tokio::select! {
            res = async {
                let response_wait_start = Instant::now();
                let res = res_stream.message().await;
                (res, elapsed_seconds(response_wait_start))
            } => {
                let (res, duration_seconds) = res;
                worker_metrics.record_response_wait(
                    duration_seconds,
                    correlated_by_batch_id.len() + correlation_rx.len(),
                );
                match res {
                    Ok(Some(status)) => {
                        drain_correlation_rx(&mut correlation_rx, &mut correlated_by_batch_id);
                        if let Some(correlated) = correlated_by_batch_id.remove(&status.batch_id) {
                            if batch_status_is_ok(&status) {
                                _ = pdata_metrics_tx
                                    .send(PDataMetricsUpdate::IncExported(
                                        signal_type,
                                        correlated.pdata,
                                        correlated.export_started_at.elapsed(),
                                    ))
                                    .await;
                            } else {
                                otel_warn!(
                                    "otap_exporter.batch_status_failed",
                                    batch_id = status.batch_id,
                                    status_code = status.status_code,
                                    status_message = status.status_message.as_str(),
                                    message = "OTAP server rejected exported batch"
                                );
                                _ = pdata_metrics_tx
                                    .send(PDataMetricsUpdate::IncFailed(
                                        signal_type,
                                        correlated.pdata,
                                        correlated.export_started_at.elapsed(),
                                        OtapExporterErrorType::from_batch_status(
                                            status.status_code,
                                        ),
                                    ))
                                    .await;
                            }
                        } else {
                            otel_warn!(
                                "otap_exporter.batch_status_unmatched",
                                batch_id = status.batch_id,
                                status_code = status.status_code,
                                status_message = status.status_message.as_str(),
                                message = "Received OTAP batch status without a correlated request"
                            );
                        }
                    },
                    Ok(None) => {
                        // sender disconnected
                        fail_correlated_pdata(
                            &pdata_metrics_tx,
                            signal_type,
                            &mut correlation_rx,
                            &mut correlated_by_batch_id,
                            OtapExporterErrorType::Transport,
                        )
                        .await;
                        break
                    }
                    Err(grpc_status) => {
                        otel_warn!(
                            "otap_exporter.response_stream_failed",
                            status = %grpc_status,
                            message = "OTAP response stream failed"
                        );
                        fail_correlated_pdata(
                            &pdata_metrics_tx,
                            signal_type,
                            &mut correlation_rx,
                            &mut correlated_by_batch_id,
                            OtapExporterErrorType::from_grpc_status(&grpc_status),
                        )
                        .await;
                        break
                    }
                };
            }
            _ = shutdown_rx.changed() => {
                shutdown = *shutdown_rx.borrow();
                if shutdown {
                    fail_correlated_pdata(
                        &pdata_metrics_tx,
                        signal_type,
                        &mut correlation_rx,
                        &mut correlated_by_batch_id,
                        OtapExporterErrorType::Shutdown,
                    )
                    .await;
                }
            }
        }
    }

    shutdown
}

const fn batch_status_is_ok(status: &BatchStatus) -> bool {
    status.status_code == StatusCode::Ok as i32
}

fn drain_correlation_rx(
    correlation_rx: &mut Receiver<CorrelatedPdata>,
    correlated_by_batch_id: &mut HashMap<i64, CorrelatedPdata>,
) {
    while let Ok(correlated) = correlation_rx.try_recv() {
        _ = correlated_by_batch_id.insert(correlated.batch_id, correlated);
    }
}

async fn fail_correlated_pdata(
    pdata_metrics_tx: &Sender<PDataMetricsUpdate>,
    signal_type: SignalType,
    correlation_rx: &mut Receiver<CorrelatedPdata>,
    correlated_by_batch_id: &mut HashMap<i64, CorrelatedPdata>,
    error_type: OtapExporterErrorType,
) {
    correlation_rx.close();
    while let Some(correlated) = correlation_rx.recv().await {
        _ = correlated_by_batch_id.insert(correlated.batch_id, correlated);
    }

    for (_, correlated) in correlated_by_batch_id.drain() {
        _ = pdata_metrics_tx
            .send(PDataMetricsUpdate::IncFailed(
                signal_type,
                correlated.pdata,
                correlated.export_started_at.elapsed(),
                error_type,
            ))
            .await;
    }
}

#[cfg(test)]
mod tests {
    use crate::exporters::otap_exporter::OTAP_EXPORTER_URN;
    use crate::exporters::otap_exporter::OTAPExporter;
    use crate::exporters::otap_exporter::OtapExporterStreamMetricSets;
    use crate::exporters::otap_exporter::OtapStreamWorkerMetricsHandle;
    use crate::exporters::otap_exporter::config::ArrowPayloadCompression;
    use otap_df_otap::otap_mock::{
        ArrowLogsServiceMock, ArrowMetricsServiceMock, ArrowTracesServiceMock, create_otap_batch,
    };
    use otap_df_otap::pdata::OtapPdata;
    use secrecy::ExposeSecret;

    use otap_df_config::SignalType;
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::Interests;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::control::CallData;
    use otap_df_engine::control::Controllable;
    use otap_df_engine::control::NodeControlMsg;
    use otap_df_engine::control::PipelineCompletionMsg;
    use otap_df_engine::control::PipelineCompletionMsgReceiver;
    use otap_df_engine::control::PipelineCompletionMsgSender;
    use otap_df_engine::control::RuntimeCtrlMsgSender;
    use otap_df_engine::control::{pipeline_completion_msg_channel, runtime_ctrl_msg_channel};
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::local::message::LocalReceiver;
    use otap_df_engine::local::message::LocalSender;
    use otap_df_engine::message::Receiver;
    use otap_df_engine::message::Sender;
    use otap_df_engine::node::NodeWithPDataReceiver;
    use otap_df_engine::testing::create_not_send_channel;
    use otap_df_engine::testing::{
        exporter::{TestContext, TestRuntime},
        test_node,
    };
    use otap_df_otap::compression::CompressionMethod;
    use otap_df_pdata::TryIntoWithOptions;
    use otap_df_pdata::otap::OtapArrowRecords;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, BatchArrowRecords, BatchStatus, StatusCode,
        arrow_logs_service_server::ArrowLogsServiceServer,
        arrow_metrics_service_server::ArrowMetricsServiceServer,
        arrow_traces_service_server::ArrowTracesServiceServer,
    };
    use otap_df_telemetry::descriptor::Instrument;
    use otap_df_telemetry::metrics::{MetricSetSnapshot, MetricValue};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use serde_json::json;
    use std::collections::HashMap;
    use std::net::SocketAddr;
    use std::ops::Add;
    use std::rc::Rc;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::net::TcpListener;
    use tokio::runtime::Runtime;
    use tokio::time::{Duration, timeout};
    use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
    use tonic::transport::Server;
    use tonic::{IntoStreamingRequest, Response, Status, Streaming};

    const METRIC_BATCH_ID: i64 = 0;
    const LOG_BATCH_ID: i64 = 1;
    const TRACE_BATCH_ID: i64 = 2;

    fn calldata_with_id(id: u64) -> CallData {
        smallvec::smallvec!(id.into())
    }

    fn calldata_id(pdata: &OtapPdata) -> u64 {
        pdata
            .source_route()
            .expect("test pdata should retain route calldata")
            .calldata[0]
            .into()
    }

    /// Scenario: OTAP stream timings are recorded for one signal while another remains untouched.
    /// Guarantees: Every timing remains isolated in its bounded signal-attribute bucket.
    #[test]
    fn otap_stream_metrics_are_partitioned_by_signal() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut metrics = OtapExporterStreamMetricSets::register(&pipeline_ctx);
        let worker = OtapStreamWorkerMetricsHandle::new(SignalType::Metrics);
        worker.record_encode(0.030);
        metrics.merge_stream_worker_metrics(&[worker]);

        assert_eq!(
            metrics
                .streams_for(SignalType::Metrics)
                .encode_duration_seconds
                .get()
                .summary()
                .1,
            0.030
        );
        assert_eq!(
            metrics
                .streams_for(SignalType::Logs)
                .encode_duration_seconds
                .get()
                .count(),
            0
        );
    }

    /// Scenario: OTAP stream metrics are transferred into terminal snapshots twice.
    /// Guarantees: Touched buckets include bounded signal attributes and documented units once, then clear.
    #[test]
    fn otap_exporter_terminal_snapshots_preserve_attributes_once() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut metrics = OtapExporterStreamMetricSets::register(&pipeline_ctx);
        let worker = OtapStreamWorkerMetricsHandle::new(SignalType::Traces);
        worker.record_response_wait(7.0, 2);
        metrics.merge_stream_worker_metrics(&[worker]);

        let snapshots = metrics.terminal_snapshots();
        assert_eq!(snapshots.len(), 1);
        assert!(snapshots.iter().any(|snapshot| {
            snapshot.descriptor().name == "exporter.otap.streams"
                && snapshot.measurement_attribute_value("signal") == Some("traces")
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .all(|metric| metric.instrument == Instrument::ExponentialHistogram)
                && snapshot
                    .descriptor()
                    .metrics
                    .iter()
                    .all(|metric| match metric.name {
                        "enqueue.duration"
                        | "encode.duration"
                        | "correlation.enqueue.duration"
                        | "response.wait.duration" => metric.unit == "s",
                        "enqueue.depth" | "correlation.depth" | "response.active" => {
                            metric.unit == "{batch}"
                        }
                        _ => false,
                    })
        }));
        assert!(metrics.terminal_snapshots().is_empty());
    }

    /// Scenario: One stream worker records more observations than the former update channel held.
    /// Guarantees: Collection retains every timing sample and clears the worker interval exactly once.
    #[test]
    fn stream_worker_metrics_are_lossless_and_interval_scoped() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let mut metrics = OtapExporterStreamMetricSets::register(&pipeline_ctx);
        let worker = OtapStreamWorkerMetricsHandle::new(SignalType::Logs);

        for value in 1..=128 {
            worker.record_encode(value as f64);
            worker.record_correlation_enqueue(value as f64, value);
            worker.record_response_wait(value as f64, value);
        }

        metrics.merge_stream_worker_metrics(std::slice::from_ref(&worker));
        let stream_metrics = metrics.streams_for(SignalType::Logs);
        assert_eq!(stream_metrics.encode_duration_seconds.get().count(), 128);
        assert_eq!(
            stream_metrics
                .correlation_enqueue_duration_seconds
                .get()
                .count(),
            128
        );
        assert_eq!(stream_metrics.correlation_depth.get().count(), 128);
        assert_eq!(
            stream_metrics.response_wait_duration_seconds.get().count(),
            128
        );
        assert_eq!(stream_metrics.response_active.get().count(), 128);

        metrics.merge_stream_worker_metrics(&[worker]);
        assert_eq!(
            metrics
                .streams_for(SignalType::Logs)
                .encode_duration_seconds
                .get()
                .count(),
            128,
            "collecting an empty worker interval must not duplicate observations"
        );
    }

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    fn scenario()
    -> impl FnOnce(TestContext<OtapPdata>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // Send a data message
                let metric_message =
                    create_otap_batch(METRIC_BATCH_ID, ArrowPayloadType::UnivariateMetrics);
                ctx.send_pdata(OtapPdata::new_default(metric_message.into()))
                    .await
                    .expect("Failed to send metric message");

                let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                ctx.send_pdata(OtapPdata::new_default(log_message.into()))
                    .await
                    .expect("Failed to send log message");

                let trace_message = create_otap_batch(TRACE_BATCH_ID, ArrowPayloadType::Spans);
                ctx.send_pdata(OtapPdata::new_default(trace_message.into()))
                    .await
                    .expect("Failed to send trace message");

                tokio::time::sleep(Duration::from_millis(500)).await;

                // Send shutdown
                ctx.send_shutdown(
                    Instant::now().add(Duration::from_millis(200)),
                    "test complete",
                )
                .await
                .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(
        mut receiver: tokio::sync::mpsc::Receiver<OtapPdata>,
    ) -> impl FnOnce(
        TestContext<OtapPdata>,
        Result<(), Error>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                exporter_result.unwrap();

                // check that the message was properly sent from the exporter
                let metrics_received: OtapArrowRecords =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .payload()
                        .try_into_with_default()
                        .expect("Could convert pdata to OTAPData");

                // Assert that the message received is what the exporter sent
                let _expected_metrics_message =
                    create_otap_batch(METRIC_BATCH_ID, ArrowPayloadType::UnivariateMetrics);
                assert!(matches!(metrics_received, _expected_metrics_message));

                let logs_received: OtapArrowRecords =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .payload()
                        .try_into_with_default()
                        .expect("Could convert pdata to OTAPData");
                let _expected_logs_message =
                    create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                assert!(matches!(logs_received, _expected_logs_message));

                let traces_received: OtapArrowRecords =
                    timeout(Duration::from_secs(3), receiver.recv())
                        .await
                        .expect("Timed out waiting for message")
                        .expect("No message received")
                        .payload()
                        .try_into_with_default()
                        .expect("Could convert pdata to OTAPData");

                let _expected_trace_message =
                    create_otap_batch(TRACE_BATCH_ID, ArrowPayloadType::Spans);
                assert!(matches!(traces_received, _expected_trace_message));
            })
        }
    }

    #[test]
    fn test_otap_exporter() {
        let test_runtime = TestRuntime::new();
        let (sender, receiver) = tokio::sync::mpsc::channel(32);
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        // tokio runtime to run grpc server in the background
        let tokio_rt = Runtime::new().unwrap();

        // run a gRPC concurrently to receive data from the exporter
        _ = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            // Signal that the server is ready to accept connections
            let _ = ready_sender.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            let mock_logs_service =
                ArrowLogsServiceServer::new(ArrowLogsServiceMock::new(sender.clone()));
            let mock_metrics_service =
                ArrowMetricsServiceServer::new(ArrowMetricsServiceMock::new(sender.clone()));
            let mock_trace_service =
                ArrowTracesServiceServer::new(ArrowTracesServiceMock::new(sender.clone()));
            Server::builder()
                .add_service(mock_logs_service)
                .add_service(mock_metrics_service)
                .add_service(mock_trace_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    // Wait for the shutdown signal
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("Test gRPC server has failed");
        });

        // Wait for the server to be ready before creating the exporter
        tokio_rt
            .block_on(ready_receiver)
            .expect("Server failed to start");

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let config = json!({
            "grpc_endpoint": grpc_endpoint,
            "compression_method": "none",
        });
        // Create a proper pipeline context for the benchmark
        let controller_ctx = ControllerContext::new(test_runtime.metrics_registry());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let exporter = ExporterWrapper::local(
            OTAPExporter::from_config(pipeline_ctx, &config).expect("Config should be valid"),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(receiver));

        _ = shutdown_sender.send("Shutdown");
    }

    #[test]
    fn test_from_config_success() {
        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "compression_method": "gzip"
        });

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");

        assert_eq!(exporter.config.grpc.grpc_endpoint, "http://localhost:4317");
        assert_eq!(exporter.config.stream_queue_capacity, 64);
        assert_eq!(exporter.config.streams_per_signal, 1);
        match exporter.config.compression_method {
            Some(ref method) => match method {
                CompressionMethod::Gzip => {} // success
                other => panic!("Expected Gzip, got {other:?}"),
            },
            None => panic!("Expected Some compression method"),
        }
    }

    #[test]
    fn test_from_config_with_timeout() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let config_with_timeout = json!({
            "grpc_endpoint": "http://localhost:4317",
            "timeout": "45s"
        });
        let exporter = OTAPExporter::from_config(pipeline_ctx.clone(), &config_with_timeout)
            .expect("Config should be valid");
        assert_eq!(exporter.config.timeout, Some(Duration::from_secs(45)));

        let config_with_timeout_ms = json!({
            "grpc_endpoint": "http://localhost:4317",
            "timeout": "250ms"
        });
        let exporter = OTAPExporter::from_config(pipeline_ctx, &config_with_timeout_ms)
            .expect("Config should be valid");
        assert_eq!(exporter.config.timeout, Some(Duration::from_millis(250)));
    }

    #[test]
    fn test_from_config_with_stream_queue_capacity() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "stream_queue_capacity": 256
        });
        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");
        assert_eq!(exporter.config.stream_queue_capacity, 256);
    }

    #[test]
    fn test_from_config_with_streams_per_signal() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "streams_per_signal": 4
        });
        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");
        assert_eq!(exporter.config.streams_per_signal, 4);
    }

    #[test]
    fn test_from_config_rejects_zero_stream_queue_capacity() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "stream_queue_capacity": 0
        });
        let err = match OTAPExporter::from_config(pipeline_ctx, &json_config) {
            Ok(_) => panic!("zero stream queue capacity should fail"),
            Err(err) => err,
        };
        assert!(format!("{err}").contains("stream_queue_capacity must be greater than 0"));
    }

    #[test]
    fn test_from_config_rejects_zero_streams_per_signal() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "streams_per_signal": 0
        });
        let err = match OTAPExporter::from_config(pipeline_ctx, &json_config) {
            Ok(_) => panic!("zero streams per signal should fail"),
            Err(err) => err,
        };
        assert!(format!("{err}").contains("streams_per_signal must be greater than 0"));
    }

    #[test]
    fn test_from_config_accepts_headers() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        // The OTAP exporter applies static `headers` as initial stream metadata,
        // so a valid non-empty map must now be accepted (it was previously
        // rejected before native support landed).
        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "headers": { "authorization": "Basic dXNlcjpwYXNz", "x-scope-orgid": "tenant-1" }
        });
        let exporter = OTAPExporter::from_config(pipeline_ctx, &json_config)
            .expect("valid headers should be accepted");
        assert_eq!(exporter.config.grpc.headers.len(), 2);
        assert_eq!(
            exporter
                .config
                .grpc
                .headers
                .get("authorization")
                .map(|v| v.expose_secret()),
            Some("Basic dXNlcjpwYXNz")
        );
        assert_eq!(
            exporter
                .config
                .grpc
                .headers
                .get("x-scope-orgid")
                .map(|v| v.expose_secret()),
            Some("tenant-1")
        );
    }

    #[test]
    fn test_validate_config_accepts_headers() {
        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "headers": { "x-tenant": "acme" }
        });
        super::validate_config(&json_config).expect("valid headers should pass validation");
    }

    #[test]
    fn test_validate_config_rejects_reserved_headers() {
        // Header validation is delegated to `GrpcClientSettings::validate()`, which
        // rejects gRPC-reserved metadata (here, the `grpc-` prefix) so a bad header
        // is still caught loudly at config load rather than silently mangled.
        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "headers": { "grpc-timeout": "1S" }
        });
        let err = super::validate_config(&json_config)
            .expect_err("reserved gRPC metadata must fail validation");
        assert!(format!("{err}").contains("reserved by the gRPC protocol"));
    }

    #[test]
    fn test_from_config_rejects_invalid_header_value() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        // `from_config` validates the shared gRPC settings itself (defense in
        // depth), so a header whose value is not valid gRPC metadata is rejected
        // at construction time, not only via the factory `validate_config` hook.
        // Reserved-name rejection is covered separately by
        // `test_validate_config_rejects_reserved_headers`. The value here carries a
        // control character (a newline): high-byte bytes are accepted as obs-text,
        // so a non-visible control byte is the value class actually rejected. It is
        // written as an escape so this source file stays ASCII-only.
        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "headers": { "x-tenant": "bad\nvalue" }
        });
        // `OTAPExporter` is not `Debug`, so match rather than `expect_err`.
        let err = match OTAPExporter::from_config(pipeline_ctx, &json_config) {
            Ok(_) => panic!("invalid metadata value must be rejected by from_config"),
            Err(err) => err,
        };
        assert!(format!("{err}").contains("must be visible ASCII"));
    }

    #[test]
    fn test_from_config_accepts_empty_headers() {
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        // An absent/empty `headers` map must remain valid (backwards compatible).
        let json_config = json!({
            "grpc_endpoint": "http://localhost:4317",
            "headers": {}
        });
        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");
        assert!(exporter.config.grpc.headers.is_empty());
    }

    #[test]
    fn test_from_config_missing_required_field() {
        let json_config = json!({
            "compression_method": "gzip"
        });

        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let result = OTAPExporter::from_config(pipeline_ctx, &json_config);

        assert!(result.is_err());
        if let Err(err) = result {
            let err_msg = format!("{err}");
            assert!(err_msg.contains("missing field `grpc_endpoint`"));
        }
    }

    #[test]
    fn test_double_compression_enabled_by_default() {
        let json_config = json!({
            "grpc_endpoint": "localhost:4317"
        });
        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");

        assert!(
            matches!(
                exporter.config.compression_method,
                Some(CompressionMethod::Zstd)
            ),
            "expected Some(Zstd) received {:?}",
            exporter.config.compression_method
        );
        assert!(
            matches!(
                exporter.config.arrow.payload_compression,
                Some(ArrowPayloadCompression::Zstd)
            ),
            "expected Some(Zstd) received {:?}",
            exporter.config.arrow.payload_compression
        );
    }

    #[test]
    fn test_can_manually_disable_compression_via_config() {
        let json_config = json!({
            "grpc_endpoint": "localhost:4317",
            "compression_method": "none",
            "arrow": {
                "payload_compression": "none"
            }
        });
        // Create a proper pipeline context for the test
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let exporter =
            OTAPExporter::from_config(pipeline_ctx, &json_config).expect("Config should be valid");
        assert!(
            exporter.config.compression_method.is_none(),
            "expected None received {:?}",
            exporter.config.compression_method
        );
        assert!(
            exporter.config.arrow.payload_compression.is_none(),
            "expected None received {:?}",
            exporter.config.arrow.payload_compression
        );
    }

    /// Scenario: The OTAP endpoint becomes available after an initial failed logs export.
    /// Guarantees: Reconnection succeeds and one success plus one failure are reported for logs.
    #[test]
    fn test_receiver_not_ready_on_start() {
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut exporter = ExporterWrapper::local(
            OTAPExporter::from_config(
                pipeline_ctx,
                &serde_json::json!({
                    "grpc_endpoint": grpc_endpoint,
                    "compression_method": "none",
                }),
            )
            .unwrap(),
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(16);
        let (pipeline_completion_msg_tx, pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(16);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        let (req_sender, req_receiver) = tokio::sync::mpsc::channel(1);
        let (server_startup_sender, mut server_startup_receiver) = tokio::sync::mpsc::channel(1);
        let (server_start_ack_sender, server_start_ack_receiver) = tokio::sync::mpsc::channel(1);
        let (server_shutdown_sender, server_shutdown_signal) = tokio::sync::oneshot::channel();

        async fn start_exporter(
            exporter: ExporterWrapper<OtapPdata>,
            runtime_ctrl_msg_tx: RuntimeCtrlMsgSender<OtapPdata>,
            pipeline_completion_msg_tx: PipelineCompletionMsgSender<OtapPdata>,
            metrics_reporter: MetricsReporter,
        ) -> Result<(), Error> {
            _ = exporter
                .start(
                    runtime_ctrl_msg_tx,
                    pipeline_completion_msg_tx,
                    metrics_reporter,
                    Interests::empty(),
                )
                .await;
            Ok(())
        }

        async fn drive_test(
            server_startup_sender: tokio::sync::mpsc::Sender<bool>,
            mut server_startup_ack_receiver: tokio::sync::mpsc::Receiver<bool>,
            server_shutdown_sender1: tokio::sync::oneshot::Sender<bool>,
            pdata_tx: Sender<OtapPdata>,
            control_sender: Sender<NodeControlMsg<OtapPdata>>,
            mut req_receiver: tokio::sync::mpsc::Receiver<OtapPdata>,
            metrics_receiver: flume::Receiver<MetricSetSnapshot>,
            metrics_reporter: MetricsReporter,
            mut pipeline_completion_msg_rx: PipelineCompletionMsgReceiver<OtapPdata>,
        ) {
            // send a request while the server isn't running and check how we handle it
            let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
            let pdata = OtapPdata::new_default(log_message.into()).test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                Default::default(),
                0,
            );
            pdata_tx
                .send(pdata)
                .await
                .expect("Failed to send log message");

            // Wait for a NACK from the pipeline-completion channel (server is down)
            timeout(Duration::from_secs(5), async {
                loop {
                    match pipeline_completion_msg_rx.recv().await {
                        Ok(PipelineCompletionMsg::DeliverNack { .. }) => break,
                        Ok(PipelineCompletionMsg::DeliverAck { .. }) => continue,
                        Err(_) => panic!("pipeline result channel closed"),
                    }
                }
            })
            .await
            .expect("Timed out waiting for NACK");

            // Now start the server
            server_startup_sender.send(true).await.unwrap();
            _ = server_startup_ack_receiver.recv().await.unwrap();

            // send another pdata now that the server has started
            let log_message = create_otap_batch(LOG_BATCH_ID + 1, ArrowPayloadType::Logs);
            let pdata = OtapPdata::new_default(log_message.into()).test_subscribe_to(
                Interests::ACKS | Interests::NACKS,
                Default::default(),
                0,
            );
            pdata_tx
                .send(pdata)
                .await
                .expect("Failed to send log message");
            _ = req_receiver.recv().await.unwrap(); // ensure we got response

            // Wait for an ACK from the pipeline-completion channel (server is up)
            timeout(Duration::from_secs(5), async {
                loop {
                    match pipeline_completion_msg_rx.recv().await {
                        Ok(PipelineCompletionMsg::DeliverAck { .. }) => break,
                        Ok(PipelineCompletionMsg::DeliverNack { .. }) => continue,
                        Err(_) => panic!("pipeline result channel closed"),
                    }
                }
            })
            .await
            .expect("Timed out waiting for ACK");

            // check the metrics:
            control_sender
                .send(NodeControlMsg::CollectTelemetry {
                    metrics_reporter: metrics_reporter.clone(),
                })
                .await
                .unwrap();
            let mut logs_exported_count = 0;
            let mut logs_failed_count = 0;
            for _ in 0..3 {
                let metrics = metrics_receiver.recv_async().await.unwrap();
                if metrics.descriptor().name == "exporter.exports"
                    && metrics.measurement_attribute_value("signal") == Some("logs")
                {
                    match metrics.measurement_attribute_value("outcome") {
                        Some("success") => {
                            logs_exported_count = metrics.get_metrics()[0].to_u64_lossy();
                        }
                        Some("failure") => {
                            logs_failed_count = metrics.get_metrics()[0].to_u64_lossy();
                        }
                        _ => {}
                    }
                }
            }
            assert_eq!(logs_exported_count, 1);
            assert_eq!(logs_failed_count, 1);

            control_sender
                .send(NodeControlMsg::Shutdown {
                    deadline: Instant::now().add(Duration::from_millis(10)),
                    reason: "shutting down".into(),
                })
                .await
                .unwrap();

            server_shutdown_sender1.send(true).unwrap();
        }

        async fn run_server(
            listening_addr: String,
            startup_ack_sender: tokio::sync::mpsc::Sender<bool>,
            shutdown_signal: tokio::sync::oneshot::Receiver<bool>,
            req_sender: tokio::sync::mpsc::Sender<OtapPdata>,
        ) {
            let listening_addr: SocketAddr = listening_addr.to_string().parse().unwrap();
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let tcp_stream = TcpListenerStream::new(tcp_listener);

            let logs_service = ArrowLogsServiceServer::new(ArrowLogsServiceMock::new(req_sender));

            Server::builder()
                .add_service(logs_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    startup_ack_sender.send(true).await.unwrap();
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("uh oh server failed");
        }

        let server_handle = tokio_rt.spawn(async move {
            let listening_addr = format!("{grpc_addr}:{grpc_port}");

            // wait for signal to start the server
            _ = server_startup_receiver.recv().await.unwrap();
            run_server(
                listening_addr.clone(),
                server_start_ack_sender.clone(),
                server_shutdown_signal,
                req_sender.clone(),
            )
            .await;
        });
        let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(3);

        let _ = tokio_rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            let metrics_reporter_start_exporter = metrics_reporter.clone();
            let _fut = local_set.spawn_local(async move {
                start_exporter(
                    exporter,
                    runtime_ctrl_msg_tx,
                    pipeline_completion_msg_tx,
                    metrics_reporter_start_exporter,
                )
                .await
            });
            tokio::join!(
                local_set,
                drive_test(
                    server_startup_sender,
                    server_start_ack_receiver,
                    server_shutdown_sender,
                    pdata_tx,
                    control_sender,
                    req_receiver,
                    metrics_rx,
                    metrics_reporter,
                    pipeline_completion_msg_rx,
                )
            )
        });

        tokio_rt
            .block_on(server_handle)
            .expect("server shutdown success");
    }

    /// Mock StreamingArrowService that consumes one batch from the request stream
    /// (triggering correlation_tx.send) then returns Err.
    /// This tests the drained-correlation path in stream_arrow_batches.
    struct MockConsumeAndFail;

    #[async_trait::async_trait]
    impl super::StreamingArrowService for MockConsumeAndFail {
        async fn handle_req_stream(
            &mut self,
            req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
        ) -> Result<Response<Streaming<BatchStatus>>, Status> {
            use tokio_stream::StreamExt;
            let mut stream = Box::pin(req_stream.into_streaming_request().into_inner());
            // Consume the first batch; this polls create_req_stream, which
            // sends the corresponding OtapPdata to correlation_tx.
            let _ = stream.next().await;
            Err(Status::unavailable("mock failure after consume"))
        }
    }

    /// Scenario: Stream creation fails after the request stream correlates its first PData batch.
    /// Guarantees: The correlated batch is drained and reported as a terminal export failure.
    #[tokio::test]
    async fn test_stream_arrow_batches_drain_correlation_on_error() {
        use super::{OtapExporterErrorType, PDataMetricsUpdate, StreamBatch, stream_arrow_batches};

        let (batches_tx, batches_rx) = tokio::sync::mpsc::channel(4);
        let (metrics_tx, mut metrics_rx) = tokio::sync::mpsc::channel(4);
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
        let pdata = OtapPdata::new_default(log_message.into());
        let payload = pdata.clone();
        batches_tx
            .send(StreamBatch {
                pdata,
                records: payload.payload().try_into_with_default().unwrap(),
                export_started_at: Instant::now(),
            })
            .await
            .unwrap();
        // Drop sender so the function exits after processing
        drop(batches_tx);

        stream_arrow_batches(
            MockConsumeAndFail,
            SignalType::Logs,
            None,
            batches_rx,
            metrics_tx,
            OtapStreamWorkerMetricsHandle::new(SignalType::Logs),
            shutdown_rx,
            None,
        )
        .await;

        // The drained pdata should come through as Failed. Attribution metrics
        // can be emitted before the failure update.
        timeout(Duration::from_secs(1), async {
            loop {
                if let PDataMetricsUpdate::IncFailed(SignalType::Logs, _, _, error_type) =
                    metrics_rx.recv().await.expect("channel closed")
                {
                    assert_eq!(error_type, OtapExporterErrorType::Unavailable);
                    break;
                }
            }
        })
        .await
        .expect("timed out waiting for IncFailed");
    }

    /// A stream-creation failure may leave the OTAP exporter in reconnect backoff.
    /// Shutdown still needs to terminate that loop promptly instead of waiting for
    /// the full backoff delay before the exporter can exit.
    struct MockAlwaysFail;

    #[async_trait::async_trait]
    impl super::StreamingArrowService for MockAlwaysFail {
        async fn handle_req_stream(
            &mut self,
            _req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
        ) -> Result<Response<Streaming<BatchStatus>>, Status> {
            Err(Status::unavailable("mock request creation failure"))
        }
    }

    /// Scenario: Stream creation repeatedly fails while the worker is in reconnect backoff.
    /// Guarantees: Shutdown interrupts the backoff and reports every accepted batch as failed.
    #[tokio::test]
    async fn test_stream_arrow_batches_shutdown_interrupts_retry_backoff() {
        use super::{OtapExporterErrorType, PDataMetricsUpdate, StreamBatch, stream_arrow_batches};

        let (batches_tx, batches_rx) = tokio::sync::mpsc::channel(8);
        let (metrics_tx, mut metrics_rx) = tokio::sync::mpsc::channel(8);
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        for batch_id in 0..5 {
            let log_message = create_otap_batch(LOG_BATCH_ID + batch_id, ArrowPayloadType::Logs);
            let pdata = OtapPdata::new_default(log_message.into());
            let payload = pdata.clone();
            batches_tx
                .send(StreamBatch {
                    pdata,
                    records: payload.payload().try_into_with_default().unwrap(),
                    export_started_at: Instant::now(),
                })
                .await
                .unwrap();
        }

        // Production drives `stream_arrow_batches` via `spawn_local` (the exporter
        // runs on a thread-local set), so the worker future is `!Send`. Mirror that
        // here with a `LocalSet` rather than `tokio::spawn`, which would require
        // `Send` and does not reflect how the exporter actually runs.
        let local = tokio::task::LocalSet::new();
        local
            .run_until(async move {
                let handle = tokio::task::spawn_local(stream_arrow_batches(
                    MockAlwaysFail,
                    SignalType::Logs,
                    None,
                    batches_rx,
                    metrics_tx,
                    OtapStreamWorkerMetricsHandle::new(SignalType::Logs),
                    shutdown_rx,
                    None,
                ));

                for attempt in 0..4 {
                    let update = metrics_rx.recv().await.expect("metrics channel closed");
                    match update {
                        PDataMetricsUpdate::IncFailed(SignalType::Logs, _, _, error_type) => {
                            assert_eq!(error_type, OtapExporterErrorType::Unavailable);
                        }
                        _ => {
                            panic!("expected IncFailed update for failed stream attempt #{attempt}")
                        }
                    }
                }

                _ = shutdown_tx.send_replace(true);
                timeout(Duration::from_millis(40), handle)
                    .await
                    .expect("shutdown should interrupt reconnect backoff promptly")
                    .unwrap();
            })
            .await;
    }

    /// gRPC service mock that returns statuses out of request order.
    struct ArrowLogsServiceOutOfOrderStatusMock;

    #[tonic::async_trait]
    impl otap_df_pdata::proto::opentelemetry::arrow::v1::arrow_logs_service_server::ArrowLogsService
        for ArrowLogsServiceOutOfOrderStatusMock
    {
        type ArrowLogsStream = std::pin::Pin<
            Box<dyn tokio_stream::Stream<Item = Result<BatchStatus, Status>> + Send + 'static>,
        >;

        async fn arrow_logs(
            &self,
            request: tonic::Request<Streaming<BatchArrowRecords>>,
        ) -> Result<Response<Self::ArrowLogsStream>, Status> {
            let mut input_stream = request.into_inner();
            let (tx, rx) = tokio::sync::mpsc::channel(2);

            _ = tokio::spawn(async move {
                let first_batch = input_stream
                    .message()
                    .await
                    .expect("first request should decode")
                    .expect("first request should be present");
                let second_batch = input_stream
                    .message()
                    .await
                    .expect("second request should decode")
                    .expect("second request should be present");

                let _ = tx
                    .send(Ok(BatchStatus {
                        batch_id: second_batch.batch_id,
                        status_code: StatusCode::Unavailable as i32,
                        status_message: "second batch rejected".into(),
                    }))
                    .await;
                let _ = tx
                    .send(Ok(BatchStatus {
                        batch_id: first_batch.batch_id,
                        status_code: StatusCode::Ok as i32,
                        status_message: "first batch accepted".into(),
                    }))
                    .await;
            });

            Ok(Response::new(
                Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)) as Self::ArrowLogsStream,
            ))
        }
    }

    /// Scenario: One stream receives success and failure statuses out of request order.
    /// Guarantees: Batch IDs route ACK/NACK correctly and both outcomes emit pdata and duration metrics.
    #[test]
    fn test_out_of_order_batch_status_uses_batch_id_correlation() {
        use otap_df_pdata::proto::opentelemetry::arrow::v1::arrow_logs_service_server::ArrowLogsServiceServer;

        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut exporter = ExporterWrapper::local(
            OTAPExporter::from_config(
                pipeline_ctx,
                &json!({
                    "grpc_endpoint": grpc_endpoint,
                    "compression_method": "none",
                    "streams_per_signal": 1,
                    "stream_queue_capacity": 4
                }),
            )
            .unwrap(),
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(2);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(16);
        let (pipeline_completion_msg_tx, mut pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(16);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        let (server_shutdown_tx, server_shutdown_rx) = tokio::sync::oneshot::channel();
        let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel();

        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let server_handle = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let _ = server_ready_tx.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            let service = ArrowLogsServiceServer::new(ArrowLogsServiceOutOfOrderStatusMock);

            Server::builder()
                .add_service(service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = server_shutdown_rx.await;
                })
                .await
                .expect("server failed");
        });

        let (metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(16);

        let _ = tokio_rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            let mr = metrics_reporter.clone();
            let _exporter_fut = local_set.spawn_local(async move {
                let _ = exporter
                    .start(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        mr,
                        Interests::empty(),
                    )
                    .await;
            });

            tokio::join!(local_set, async {
                server_ready_rx
                    .await
                    .expect("server should bind before exporter traffic starts");

                let first_id = 11_u64;
                let first_message = create_otap_batch(LOG_BATCH_ID + 10, ArrowPayloadType::Logs);
                let first_pdata = OtapPdata::new_default(first_message.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    calldata_with_id(first_id),
                    0,
                );

                let second_id = 21_u64;
                let second_message = create_otap_batch(LOG_BATCH_ID + 20, ArrowPayloadType::Logs);
                let second_pdata = OtapPdata::new_default(second_message.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    calldata_with_id(second_id),
                    0,
                );

                pdata_tx.send(first_pdata).await.expect("send first pdata");
                pdata_tx
                    .send(second_pdata)
                    .await
                    .expect("send second pdata");

                let mut ack_id = None;
                let mut nack_id = None;
                timeout(Duration::from_secs(5), async {
                    while ack_id.is_none() || nack_id.is_none() {
                        match pipeline_completion_msg_rx.recv().await {
                            Ok(PipelineCompletionMsg::DeliverAck { ack }) => {
                                ack_id = Some(calldata_id(&ack.accepted));
                            }
                            Ok(PipelineCompletionMsg::DeliverNack { nack }) => {
                                nack_id = Some(calldata_id(&nack.refused));
                            }
                            Err(_) => panic!("pipeline result channel closed"),
                        }
                    }
                })
                .await
                .expect("timed out waiting for ACK and NACK");

                assert_eq!(ack_id, Some(first_id));
                assert_eq!(nack_id, Some(second_id));

                control_sender
                    .send(NodeControlMsg::CollectTelemetry {
                        metrics_reporter: metrics_reporter.clone(),
                    })
                    .await
                    .expect("collect exporter telemetry");
                let mut export_outcomes = HashMap::new();
                let mut duration_outcomes = HashMap::new();
                for _ in 0..4 {
                    let snapshot = timeout(Duration::from_secs(3), metrics_rx.recv_async())
                        .await
                        .expect("timed out collecting exporter telemetry")
                        .expect("exporter telemetry channel closed");
                    if snapshot.measurement_attribute_value("signal") != Some("logs") {
                        continue;
                    }
                    let Some(outcome) = snapshot.measurement_attribute_value("outcome") else {
                        continue;
                    };
                    if snapshot.descriptor().name == "exporter.exports" {
                        let _ = export_outcomes
                            .insert(outcome, snapshot.get_metrics()[0].to_u64_lossy());
                        let MetricValue::Distribution(duration) = &snapshot.get_metrics()[1] else {
                            panic!("export duration should be a histogram")
                        };
                        let _ = duration_outcomes.insert(outcome, duration.count());
                    }
                }
                assert_eq!(export_outcomes.get("success"), Some(&1));
                assert_eq!(export_outcomes.get("failure"), Some(&1));
                assert_eq!(duration_outcomes.get("success"), Some(&1));
                assert_eq!(duration_outcomes.get("failure"), Some(&1));

                control_sender
                    .send(NodeControlMsg::Shutdown {
                        deadline: Instant::now().add(Duration::from_millis(10)),
                        reason: "test done".into(),
                    })
                    .await
                    .unwrap();
                server_shutdown_tx.send(true).unwrap();
            })
        });

        tokio_rt
            .block_on(server_handle)
            .expect("server shutdown success");
    }

    /// gRPC service mock that accepts a request batch but leaves the response
    /// stream pending until the test releases it.
    struct ArrowLogsServicePendingStatusMock {
        sender: tokio::sync::mpsc::Sender<()>,
        release: Arc<tokio::sync::Notify>,
    }

    #[tonic::async_trait]
    impl otap_df_pdata::proto::opentelemetry::arrow::v1::arrow_logs_service_server::ArrowLogsService
        for ArrowLogsServicePendingStatusMock
    {
        type ArrowLogsStream = std::pin::Pin<
            Box<dyn tokio_stream::Stream<Item = Result<BatchStatus, Status>> + Send + 'static>,
        >;

        async fn arrow_logs(
            &self,
            request: tonic::Request<Streaming<BatchArrowRecords>>,
        ) -> Result<Response<Self::ArrowLogsStream>, Status> {
            let mut input_stream = request.into_inner();
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let notify = self.sender.clone();
            let release = Arc::clone(&self.release);

            _ = tokio::spawn(async move {
                if let Ok(Some(_batch)) = input_stream.message().await {
                    let _ = notify.send(()).await;
                    let _keep_response_open = tx;
                    release.notified().await;
                }
            });

            Ok(Response::new(
                Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)) as Self::ArrowLogsStream,
            ))
        }
    }

    /// Tests that exporter shutdown NACKs a batch already yielded to the OTAP
    /// request stream when no corresponding BatchStatus has arrived yet.
    #[test]
    fn test_shutdown_nacks_correlated_pdata() {
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut exporter = ExporterWrapper::local(
            OTAPExporter::from_config(
                pipeline_ctx,
                &json!({
                    "grpc_endpoint": grpc_endpoint,
                    "compression_method": "none",
                    "streams_per_signal": 1,
                    "stream_queue_capacity": 4
                }),
            )
            .unwrap(),
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(16);
        let (pipeline_completion_msg_tx, mut pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(16);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        let (batch_received_tx, mut batch_received_rx) = tokio::sync::mpsc::channel(1);
        let release_response = Arc::new(tokio::sync::Notify::new());
        let release_response_for_service = Arc::clone(&release_response);
        let release_response_for_test = Arc::clone(&release_response);
        let (server_shutdown_tx, server_shutdown_rx) = tokio::sync::oneshot::channel();
        let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel();

        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let server_handle = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let _ = server_ready_tx.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            let pending_service = ArrowLogsServiceServer::new(ArrowLogsServicePendingStatusMock {
                sender: batch_received_tx,
                release: release_response_for_service,
            });

            Server::builder()
                .add_service(pending_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = server_shutdown_rx.await;
                })
                .await
                .expect("server failed");
        });

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

        let _ = tokio_rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            let mr = metrics_reporter.clone();
            let _exporter_fut = local_set.spawn_local(async move {
                let _ = exporter
                    .start(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        mr,
                        Interests::empty(),
                    )
                    .await;
            });

            tokio::join!(local_set, async {
                server_ready_rx
                    .await
                    .expect("server should bind before exporter traffic starts");

                let pdata_id = 31_u64;
                let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                let pdata = OtapPdata::new_default(log_message.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    calldata_with_id(pdata_id),
                    0,
                );
                pdata_tx.send(pdata).await.expect("send pdata");

                _ = timeout(Duration::from_secs(5), batch_received_rx.recv())
                    .await
                    .expect("timed out waiting for server to receive batch");

                control_sender
                    .send(NodeControlMsg::Shutdown {
                        deadline: Instant::now().add(Duration::from_millis(10)),
                        reason: "test done".into(),
                    })
                    .await
                    .unwrap();

                let nack_id = timeout(Duration::from_secs(5), async {
                    loop {
                        match pipeline_completion_msg_rx.recv().await {
                            Ok(PipelineCompletionMsg::DeliverNack { nack }) => {
                                break calldata_id(&nack.refused);
                            }
                            Ok(PipelineCompletionMsg::DeliverAck { .. }) => continue,
                            Err(_) => panic!("pipeline result channel closed"),
                        }
                    }
                })
                .await
                .expect("Timed out waiting for shutdown NACK");

                assert_eq!(nack_id, pdata_id);

                release_response_for_test.notify_waiters();
                server_shutdown_tx.send(true).unwrap();
            })
        });

        tokio_rt
            .block_on(server_handle)
            .expect("server shutdown success");
    }

    /// gRPC service mock that returns a gRPC error in the response stream
    /// after processing the first batch.
    struct ArrowLogsServiceGrpcErrorMock {
        sender: tokio::sync::mpsc::Sender<()>,
    }

    #[tonic::async_trait]
    impl otap_df_pdata::proto::opentelemetry::arrow::v1::arrow_logs_service_server::ArrowLogsService
        for ArrowLogsServiceGrpcErrorMock
    {
        type ArrowLogsStream = std::pin::Pin<
            Box<dyn tokio_stream::Stream<Item = Result<BatchStatus, Status>> + Send + 'static>,
        >;

        async fn arrow_logs(
            &self,
            request: tonic::Request<Streaming<BatchArrowRecords>>,
        ) -> Result<Response<Self::ArrowLogsStream>, Status> {
            let mut input_stream = request.into_inner();
            let (tx, rx) = tokio::sync::mpsc::channel(1);
            let notify = self.sender.clone();

            _ = tokio::spawn(async move {
                // Read the first batch
                if let Ok(Some(_batch)) = input_stream.message().await {
                    // Notify the test that we received the batch
                    let _ = notify.send(()).await;
                    // Send a gRPC error instead of a success status
                    let _ = tx.send(Err(Status::internal("mock gRPC error"))).await;
                }
            });

            Ok(Response::new(
                Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx)) as Self::ArrowLogsStream,
            ))
        }
    }

    /// Tests that when the gRPC server returns an error in the response stream
    /// (after the connection was successfully established), the exporter sends
    /// a NACK for the corresponding pdata.
    #[test]
    fn test_grpc_error_in_response_stream() {
        use otap_df_pdata::proto::opentelemetry::arrow::v1::arrow_logs_service_server::ArrowLogsServiceServer;

        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut exporter = ExporterWrapper::local(
            OTAPExporter::from_config(
                pipeline_ctx,
                &json!({
                    "grpc_endpoint": grpc_endpoint,
                    "compression_method": "none",
                }),
            )
            .unwrap(),
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(16);
        let (pipeline_completion_msg_tx, mut pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(16);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        let (batch_received_tx, mut batch_received_rx) = tokio::sync::mpsc::channel(1);
        let (server_shutdown_tx, server_shutdown_rx) = tokio::sync::oneshot::channel();
        let (server_ready_tx, server_ready_rx) = tokio::sync::oneshot::channel();

        // Start gRPC server that returns errors
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let server_handle = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let _ = server_ready_tx.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            let error_service = ArrowLogsServiceServer::new(ArrowLogsServiceGrpcErrorMock {
                sender: batch_received_tx,
            });

            Server::builder()
                .add_service(error_service)
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = server_shutdown_rx.await;
                })
                .await
                .expect("server failed");
        });

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

        let _ = tokio_rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            let mr = metrics_reporter.clone();
            let _exporter_fut = local_set.spawn_local(async move {
                let _ = exporter
                    .start(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        mr,
                        Interests::empty(),
                    )
                    .await;
            });

            tokio::join!(local_set, async {
                server_ready_rx
                    .await
                    .expect("server should bind before exporter traffic starts");

                // Send a batch with ACK/NACK subscription
                let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                let pdata = OtapPdata::new_default(log_message.into()).test_subscribe_to(
                    Interests::ACKS | Interests::NACKS,
                    Default::default(),
                    0,
                );
                pdata_tx.send(pdata).await.expect("send pdata");

                // Wait for server to receive the batch
                _ = timeout(Duration::from_secs(5), batch_received_rx.recv())
                    .await
                    .expect("timed out waiting for server to receive batch");

                // Wait for NACK (server returned gRPC error)
                timeout(Duration::from_secs(5), async {
                    loop {
                        match pipeline_completion_msg_rx.recv().await {
                            Ok(PipelineCompletionMsg::DeliverNack { .. }) => break,
                            Ok(PipelineCompletionMsg::DeliverAck { .. }) => continue,
                            Err(_) => panic!("pipeline result channel closed"),
                        }
                    }
                })
                .await
                .expect("Timed out waiting for NACK from gRPC error");

                // Shutdown
                control_sender
                    .send(NodeControlMsg::Shutdown {
                        deadline: Instant::now().add(Duration::from_millis(10)),
                        reason: "test done".into(),
                    })
                    .await
                    .unwrap();
                server_shutdown_tx.send(true).unwrap();
            })
        });

        tokio_rt
            .block_on(server_handle)
            .expect("server shutdown success");
    }

    /// Capture from a stream-open request: (signal label, value of the
    /// `authorization` header, value of the tenant header), each `None` when
    /// the header was absent from the initial stream metadata.
    type CapturedHeaders = (&'static str, Option<String>, Option<String>);

    const HDR_AUTH: &str = "authorization";
    const HDR_AUTH_VAL: &str = "Basic dXNlcjpwYXNz";
    const HDR_TENANT: &str = "x-scope-orgid";
    const HDR_TENANT_VAL: &str = "tenant-1";

    type BatchStatusStream =
        std::pin::Pin<Box<dyn tokio_stream::Stream<Item = Result<BatchStatus, Status>> + Send>>;

    /// Reads the configured static headers from a stream-open request's initial
    /// metadata (before the body is consumed), labeled by signal type.
    fn capture_configured(
        request: &tonic::Request<Streaming<BatchArrowRecords>>,
        signal: &'static str,
    ) -> CapturedHeaders {
        let get = |name: &str| {
            request
                .metadata()
                .get(name)
                .and_then(|value| value.to_str().ok())
                .map(str::to_string)
        };
        (signal, get(HDR_AUTH), get(HDR_TENANT))
    }

    /// Spawns a responder that ACKs every received batch so the exporter's
    /// stream completes cleanly, and returns the response stream.
    fn ack_all_batches(mut input_stream: Streaming<BatchArrowRecords>) -> BatchStatusStream {
        let (tx, rx) = tokio::sync::mpsc::channel(8);
        _ = tokio::spawn(async move {
            while let Ok(Some(batch)) = input_stream.message().await {
                let _ = tx
                    .send(Ok(BatchStatus {
                        batch_id: batch.batch_id,
                        status_code: StatusCode::Ok as i32,
                        status_message: "ok".into(),
                    }))
                    .await;
            }
        });
        Box::pin(tokio_stream::wrappers::ReceiverStream::new(rx))
    }

    /// Per-signal mocks that record the initial stream-open metadata and ACK.
    struct CapturingLogsMock {
        captured: tokio::sync::mpsc::Sender<CapturedHeaders>,
    }
    struct CapturingMetricsMock {
        captured: tokio::sync::mpsc::Sender<CapturedHeaders>,
    }
    struct CapturingTracesMock {
        captured: tokio::sync::mpsc::Sender<CapturedHeaders>,
    }

    #[tonic::async_trait]
    impl otap_df_pdata::proto::opentelemetry::arrow::v1::arrow_logs_service_server::ArrowLogsService
        for CapturingLogsMock
    {
        type ArrowLogsStream = BatchStatusStream;
        async fn arrow_logs(
            &self,
            request: tonic::Request<Streaming<BatchArrowRecords>>,
        ) -> Result<Response<Self::ArrowLogsStream>, Status> {
            let _ = self.captured.try_send(capture_configured(&request, "logs"));
            Ok(Response::new(ack_all_batches(request.into_inner())))
        }
    }

    #[tonic::async_trait]
    impl otap_df_pdata::proto::opentelemetry::arrow::v1::arrow_metrics_service_server::ArrowMetricsService
        for CapturingMetricsMock
    {
        type ArrowMetricsStream = BatchStatusStream;
        async fn arrow_metrics(
            &self,
            request: tonic::Request<Streaming<BatchArrowRecords>>,
        ) -> Result<Response<Self::ArrowMetricsStream>, Status> {
            let _ = self.captured.try_send(capture_configured(&request, "metrics"));
            Ok(Response::new(ack_all_batches(request.into_inner())))
        }
    }

    #[tonic::async_trait]
    impl otap_df_pdata::proto::opentelemetry::arrow::v1::arrow_traces_service_server::ArrowTracesService
        for CapturingTracesMock
    {
        type ArrowTracesStream = BatchStatusStream;
        async fn arrow_traces(
            &self,
            request: tonic::Request<Streaming<BatchArrowRecords>>,
        ) -> Result<Response<Self::ArrowTracesStream>, Status> {
            let _ = self.captured.try_send(capture_configured(&request, "traces"));
            Ok(Response::new(ack_all_batches(request.into_inner())))
        }
    }

    /// Mock `StreamingArrowService` that records the request's static-header
    /// metadata on every stream open, then fails (without consuming the request
    /// stream) so the worker reconnects and opens another stream. Used to prove
    /// the headers are re-applied on each (re)open, not just the first.
    struct MockRecordMetadata {
        recorded: Arc<std::sync::Mutex<Vec<Option<String>>>>,
        header_name: String,
    }

    #[async_trait::async_trait]
    impl super::StreamingArrowService for MockRecordMetadata {
        async fn handle_req_stream(
            &mut self,
            req_stream: impl IntoStreamingRequest<Message = BatchArrowRecords> + Send,
        ) -> Result<Response<Streaming<BatchStatus>>, Status> {
            let request = req_stream.into_streaming_request();
            let value = request
                .metadata()
                .get(self.header_name.as_str())
                .and_then(|v| v.to_str().ok())
                .map(str::to_string);
            self.recorded.lock().unwrap().push(value);
            // Fail without consuming the request stream so the queued batch is
            // not drained into this stream; the worker then reconnects and opens
            // a fresh stream for the next batch.
            Err(Status::unavailable("mock records metadata then fails"))
        }
    }

    /// End-to-end: configured static `headers` must be attached as initial
    /// metadata on the outbound Arrow log, metric, AND trace streams (issue
    /// #3314 acceptance criteria), and multiple headers must all be present.
    #[test]
    fn test_otap_exporter_sends_static_headers() {
        let test_runtime = TestRuntime::new();
        let (captured_tx, mut captured_rx) = tokio::sync::mpsc::channel::<CapturedHeaders>(8);
        let collected: Arc<std::sync::Mutex<Vec<CapturedHeaders>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let tokio_rt = Runtime::new().unwrap();

        let cap_logs = captured_tx.clone();
        let cap_metrics = captured_tx.clone();
        let cap_traces = captured_tx;
        let server_handle = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let _ = ready_sender.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            Server::builder()
                .add_service(ArrowLogsServiceServer::new(CapturingLogsMock {
                    captured: cap_logs,
                }))
                .add_service(ArrowMetricsServiceServer::new(CapturingMetricsMock {
                    captured: cap_metrics,
                }))
                .add_service(ArrowTracesServiceServer::new(CapturingTracesMock {
                    captured: cap_traces,
                }))
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("Test gRPC server has failed");
        });

        tokio_rt
            .block_on(ready_receiver)
            .expect("Server failed to start");

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let config = json!({
            "grpc_endpoint": grpc_endpoint,
            "compression_method": "none",
            "headers": { HDR_AUTH: HDR_AUTH_VAL, HDR_TENANT: HDR_TENANT_VAL },
        });
        let controller_ctx = ControllerContext::new(test_runtime.metrics_registry());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let exporter = ExporterWrapper::local(
            OTAPExporter::from_config(pipeline_ctx, &config).expect("Config should be valid"),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let collected_scenario = collected.clone();
        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| async move {
                // One batch per signal opens one stream per signal.
                for (batch_id, payload) in [
                    (METRIC_BATCH_ID, ArrowPayloadType::UnivariateMetrics),
                    (LOG_BATCH_ID, ArrowPayloadType::Logs),
                    (TRACE_BATCH_ID, ArrowPayloadType::Spans),
                ] {
                    let msg = create_otap_batch(batch_id, payload);
                    ctx.send_pdata(OtapPdata::new_default(msg.into()))
                        .await
                        .expect("Failed to send pdata");
                }

                // Deterministically wait for all three stream-open captures (no
                // fixed sleep), then shut down. Always shut down so a missing
                // capture surfaces as a validation failure rather than a hang.
                for _ in 0..3 {
                    if let Ok(Some(capture)) =
                        timeout(Duration::from_secs(5), captured_rx.recv()).await
                    {
                        collected_scenario.lock().unwrap().push(capture);
                    }
                }

                ctx.send_shutdown(
                    Instant::now().add(Duration::from_millis(200)),
                    "test complete",
                )
                .await
                .expect("Failed to send Shutdown");
            })
            .run_validation(move |_, exporter_result| async move {
                exporter_result.expect("exporter should shut down cleanly");
                let captures = collected.lock().unwrap();
                for signal in ["logs", "metrics", "traces"] {
                    let entry = captures
                        .iter()
                        .find(|(s, _, _)| *s == signal)
                        .unwrap_or_else(|| panic!("no stream-open capture for {signal} stream"));
                    assert_eq!(
                        entry.1.as_deref(),
                        Some(HDR_AUTH_VAL),
                        "authorization header missing on {signal} stream"
                    );
                    assert_eq!(
                        entry.2.as_deref(),
                        Some(HDR_TENANT_VAL),
                        "tenant header missing on {signal} stream"
                    );
                }
            });

        _ = shutdown_sender.send("Shutdown");
        // Await the server task so teardown is deterministic and no background
        // task is left pending at runtime drop.
        tokio_rt
            .block_on(server_handle)
            .expect("server task should shut down cleanly");
    }

    /// Backwards compatibility: with no `headers` configured, the exporter must
    /// open streams with NO static metadata (the zero-alloc fast path), not send
    /// empty or stray header values.
    #[test]
    fn test_otap_exporter_without_headers_sends_no_metadata() {
        let test_runtime = TestRuntime::new();
        let (captured_tx, mut captured_rx) = tokio::sync::mpsc::channel::<CapturedHeaders>(4);
        let collected: Arc<std::sync::Mutex<Vec<CapturedHeaders>>> =
            Arc::new(std::sync::Mutex::new(Vec::new()));
        let (shutdown_sender, shutdown_signal) = tokio::sync::oneshot::channel();
        let (ready_sender, ready_receiver) = tokio::sync::oneshot::channel();
        let grpc_addr = "127.0.0.1";
        let grpc_port = otap_df_test_net::pick_unused_loopback_tcp_port();
        let grpc_endpoint = format!("http://{grpc_addr}:{grpc_port}");
        let listening_addr: SocketAddr = format!("{grpc_addr}:{grpc_port}").parse().unwrap();
        let tokio_rt = Runtime::new().unwrap();

        let server_handle = tokio_rt.spawn(async move {
            let tcp_listener = TcpListener::bind(listening_addr).await.unwrap();
            let _ = ready_sender.send(());
            let tcp_stream = TcpListenerStream::new(tcp_listener);
            Server::builder()
                .add_service(ArrowLogsServiceServer::new(CapturingLogsMock {
                    captured: captured_tx,
                }))
                .serve_with_incoming_shutdown(tcp_stream, async {
                    let _ = shutdown_signal.await;
                })
                .await
                .expect("Test gRPC server has failed");
        });

        tokio_rt
            .block_on(ready_receiver)
            .expect("Server failed to start");

        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        // No `headers` key at all.
        let config = json!({
            "grpc_endpoint": grpc_endpoint,
            "compression_method": "none",
        });
        let controller_ctx = ControllerContext::new(test_runtime.metrics_registry());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);
        let exporter = ExporterWrapper::local(
            OTAPExporter::from_config(pipeline_ctx, &config).expect("Config should be valid"),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        let collected_scenario = collected.clone();
        test_runtime
            .set_exporter(exporter)
            .run_test(move |ctx| async move {
                let msg = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                ctx.send_pdata(OtapPdata::new_default(msg.into()))
                    .await
                    .expect("Failed to send log message");

                if let Ok(Some(capture)) = timeout(Duration::from_secs(5), captured_rx.recv()).await
                {
                    collected_scenario.lock().unwrap().push(capture);
                }

                ctx.send_shutdown(
                    Instant::now().add(Duration::from_millis(200)),
                    "test complete",
                )
                .await
                .expect("Failed to send Shutdown");
            })
            .run_validation(move |_, exporter_result| async move {
                exporter_result.expect("exporter should shut down cleanly");
                let captures = collected.lock().unwrap();
                let entry = captures
                    .iter()
                    .find(|(s, _, _)| *s == "logs")
                    .expect("logs stream should have opened");
                assert_eq!(entry.1, None, "no authorization header should be sent");
                assert_eq!(entry.2, None, "no tenant header should be sent");
            });

        _ = shutdown_sender.send("Shutdown");
        // Await the server task so teardown is deterministic and no background
        // task is left pending at runtime drop.
        tokio_rt
            .block_on(server_handle)
            .expect("server task should shut down cleanly");
    }

    /// Scenario: An OTAP stream reopens after a failure while static headers are configured.
    /// Guarantees: Every stream open carries the configured metadata, including reconnects.
    #[tokio::test]
    async fn test_stream_arrow_batches_applies_headers_on_every_open() {
        use super::{StreamBatch, stream_arrow_batches};

        let (batches_tx, batches_rx) = tokio::sync::mpsc::channel(4);
        let (metrics_tx, _metrics_rx) = tokio::sync::mpsc::channel(16);
        let (_shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);

        // Two batches => two stream opens (the mock fails each without consuming
        // the request stream, so neither drains the other's queued batch).
        for _ in 0..2 {
            let msg = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
            let pdata = OtapPdata::new_default(msg.into());
            let payload = pdata.clone();
            batches_tx
                .send(StreamBatch {
                    pdata,
                    records: payload.payload().try_into_with_default().unwrap(),
                    export_started_at: Instant::now(),
                })
                .await
                .unwrap();
        }
        drop(batches_tx);

        let mut headers = HashMap::new();
        let _ = headers.insert(HDR_AUTH.to_string(), HDR_AUTH_VAL.into());
        let settings = otap_df_otap::otap_grpc::client_settings::GrpcClientSettings {
            headers,
            ..Default::default()
        };
        let static_metadata = settings.build_static_metadata().map(Rc::new);
        assert!(
            static_metadata.is_some(),
            "test setup should produce metadata"
        );

        let recorded = Arc::new(std::sync::Mutex::new(Vec::<Option<String>>::new()));
        stream_arrow_batches(
            MockRecordMetadata {
                recorded: recorded.clone(),
                header_name: HDR_AUTH.to_string(),
            },
            SignalType::Logs,
            None,
            batches_rx,
            metrics_tx,
            OtapStreamWorkerMetricsHandle::new(SignalType::Logs),
            shutdown_rx,
            static_metadata,
        )
        .await;

        let recorded = recorded.lock().unwrap();
        assert_eq!(
            recorded.len(),
            2,
            "each of the two stream opens must apply the static headers"
        );
        for (i, value) in recorded.iter().enumerate() {
            assert_eq!(
                value.as_deref(),
                Some(HDR_AUTH_VAL),
                "authorization header missing on stream open #{i} (must be re-applied per open)"
            );
        }
    }

    #[test]
    fn test_otap_exporter_connection_failure_backoff() {
        use std::ops::Add;
        use tokio::runtime::Runtime;
        use tokio::time::timeout;

        let tokio_rt = Runtime::new().unwrap();
        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut exporter = ExporterWrapper::local(
            OTAPExporter::from_config(
                pipeline_ctx,
                &serde_json::json!({
                    "grpc_endpoint": "http://127.0.0.1:56790",
                    "compression_method": "none",
                    "stream_queue_capacity": 10,
                }),
            )
            .unwrap(),
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(16);
        let (pipeline_completion_msg_tx, _pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(16);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .unwrap();

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

        tokio_rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            let exporter_handle = local_set.spawn_local(async move {
                exporter
                    .start(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        metrics_reporter,
                        Interests::empty(),
                    )
                    .await
            });

            local_set
                .run_until(async move {
                    let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                    let pdata1 = OtapPdata::new_default(log_message.into());
                    pdata_tx.send(pdata1).await.unwrap();

                    // Wait enough time for handle_req_stream to fail with connection refused
                    // and enter the Err(e) branch which sleeps for 50ms (INITIAL_BACKOFF).
                    tokio::time::sleep(Duration::from_millis(200)).await;

                    control_sender
                        .send(NodeControlMsg::Shutdown {
                            deadline: Instant::now().add(Duration::from_millis(500)),
                            reason: "shutdown test".into(),
                        })
                        .await
                        .unwrap();

                    let shutdown_result = timeout(Duration::from_secs(1), exporter_handle).await;
                    assert!(shutdown_result.is_ok(), "Expected clean shutdown");
                })
                .await;
        });
    }

    #[test]
    fn test_otap_exporter_deadlock_on_full_queue_shutdown() {
        use std::ops::Add;
        use tokio::runtime::Runtime;
        use tokio::time::timeout;

        let tokio_rt = Runtime::new().unwrap();

        let test_runtime = TestRuntime::<OtapPdata>::new();
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
        let telemetry_registry_handle = TelemetryRegistryHandle::new();
        let controller_ctx = ControllerContext::new(telemetry_registry_handle);
        let node_id = test_node(test_runtime.config().name.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 1, 0);

        let mut exporter = ExporterWrapper::local(
            OTAPExporter::from_config(
                pipeline_ctx,
                &serde_json::json!({
                    "grpc_endpoint": "http://127.0.0.1:56789",
                    "compression_method": "none",
                    "stream_queue_capacity": 1,
                }),
            )
            .unwrap(),
            node_id.clone(),
            node_config,
            test_runtime.config(),
        );

        let control_sender = exporter.control_sender();
        let (pdata_tx, pdata_rx) = create_not_send_channel::<OtapPdata>(1);
        let pdata_tx = Sender::Local(LocalSender::mpsc(pdata_tx));
        let pdata_rx = Receiver::Local(LocalReceiver::mpsc(pdata_rx));
        let (runtime_ctrl_msg_tx, _runtime_ctrl_msg_rx) = runtime_ctrl_msg_channel(16);
        let (pipeline_completion_msg_tx, _pipeline_completion_msg_rx) =
            pipeline_completion_msg_channel(16);
        exporter
            .set_pdata_receiver(node_id.clone(), pdata_rx)
            .expect("Failed to set PData Receiver");

        let (_metrics_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(1);

        tokio_rt.block_on(async move {
            let local_set = tokio::task::LocalSet::new();
            let exporter_handle = local_set.spawn_local(async move {
                exporter
                    .start(
                        runtime_ctrl_msg_tx,
                        pipeline_completion_msg_tx,
                        metrics_reporter,
                        Interests::empty(),
                    )
                    .await
            });

            local_set
                .run_until(async move {
                    // Send first batch -- exporter forwards it to a stream worker.
                    let log_message = create_otap_batch(LOG_BATCH_ID, ArrowPayloadType::Logs);
                    let pdata1 = OtapPdata::new_default(log_message.into());
                    pdata_tx.send(pdata1).await.unwrap();

                    tokio::time::sleep(Duration::from_millis(50)).await;

                    // Send second batch -- fills the stream queue (capacity=1).
                    let log_message = create_otap_batch(LOG_BATCH_ID + 1, ArrowPayloadType::Logs);
                    let pdata2 = OtapPdata::new_default(log_message.into());
                    pdata_tx.send(pdata2).await.unwrap();

                    // Send third batch in a background task -- this will block inside
                    // enqueue_stream_batch waiting for queue space, simulating full
                    // backpressure with an unreachable downstream.
                    let log_message = create_otap_batch(LOG_BATCH_ID + 2, ArrowPayloadType::Logs);
                    let pdata3 = OtapPdata::new_default(log_message.into());
                    let pdata_tx_clone = pdata_tx.clone();
                    let send_handle = tokio::task::spawn_local(async move {
                        _ = pdata_tx_clone.send(pdata3).await;
                    });

                    tokio::time::sleep(Duration::from_millis(50)).await;

                    // Request shutdown -- before the fix, the exporter would deadlock
                    // because the main loop was blocked in enqueue_stream_batch and
                    // could never process this Shutdown control message.
                    control_sender
                        .send(NodeControlMsg::Shutdown {
                            deadline: Instant::now().add(Duration::from_millis(10)),
                            reason: "shutdown test".into(),
                        })
                        .await
                        .unwrap();

                    // The exporter must shut down within 200ms, not hang forever.
                    let shutdown_result =
                        timeout(Duration::from_millis(200), exporter_handle).await;
                    assert!(
                        shutdown_result.is_ok(),
                        "Expected exporter to shut down successfully and not deadlock"
                    );

                    send_handle.abort();
                    drop(pdata_tx);
                })
                .await;
        });
    }
}
