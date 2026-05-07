// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Provides a set of structs and enums that interact with the gRPC Server with BiDirectional
//! streaming.
//!
//! Implements the necessary service traits for OTLP data.
//!
//! ToDo: Modify OTAPData -> Optimize message transport
//! ToDo: Handle Ack and Nack, return proper batch status
//! ToDo: Change how channel sizes are handled? Currently defined when creating otap_receiver -> passing channel size to the ServiceImpl

use crate::pdata::{Context, OtapPdata};
use futures::future::BoxFuture;
use futures::stream::FuturesUnordered;
use futures::{FutureExt, StreamExt as FuturesStreamExt};
use otap_df_engine::{
    Interests, MessageSourceSharedEffectHandlerExtension, ProducerEffectHandlerExtension,
    memory_limiter::SharedReceiverAdmissionState, shared::receiver as shared,
};
use otap_df_pdata::{
    Consumer,
    otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces, from_record_messages},
    proto::opentelemetry::arrow::v1::{
        BatchArrowRecords, BatchStatus, StatusCode, arrow_logs_service_server::ArrowLogsService,
        arrow_metrics_service_server::ArrowMetricsService,
        arrow_traces_service_server::ArrowTracesService,
    },
};
use otap_df_telemetry::{otel_error, otel_warn};
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::oneshot;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status, Streaming};

pub mod client_settings;
pub mod common;
pub mod middleware;
pub mod otlp;
pub mod proxy;
pub mod server_settings;

use crate::memory_pressure_layer::{ReceiverRejectionMetrics, grpc_memory_pressure_status};
use crate::otap_grpc::otlp::server::SharedState;
pub use client_settings::GrpcClientSettings;
pub use server_settings::GrpcServerSettings;

/// Common settings for OTLP receivers.
#[derive(Clone, Debug)]
pub struct NewSettings {
    /// Maximum concurrent requests per receiver instance (per core).
    pub max_concurrent_requests: usize,
    /// Whether the receiver should wait.
    pub wait_for_result: bool,
}

/// Common settings for OTLP receivers.
#[derive(Clone)]
pub struct Settings {
    /// Size of the channel used to buffer outgoing responses to the client.
    pub response_stream_channel_size: usize,
    /// Maximum concurrent requests per receiver instance (per core).
    pub max_concurrent_requests: usize,
    /// Maximum in-flight wait-for-result requests admitted from a single stream.
    pub max_concurrent_requests_per_stream: usize,
    /// Whether the receiver should wait.
    pub wait_for_result: bool,
    /// Receiver-local memory pressure admission state.
    pub admission_state: SharedReceiverAdmissionState,
    /// Shared rejection counters used by both stream-open and per-batch shedding.
    pub receiver_rejection_metrics: Option<Arc<dyn ReceiverRejectionMetrics>>,
}

impl Settings {
    fn effective_max_concurrent_requests_per_stream(&self) -> usize {
        self.max_concurrent_requests_per_stream
            .min(self.max_concurrent_requests)
            .max(1)
    }
}

/// struct that implements the ArrowLogsService trait
pub struct ArrowLogsServiceImpl {
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<SharedState>,
    settings: Settings,
}

impl ArrowLogsServiceImpl {
    /// create a new ArrowLogsServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            effect_handler,
            state: settings
                .wait_for_result
                .then(|| SharedState::new(settings.max_concurrent_requests)),
            settings: settings.clone(),
        }
    }

    /// Get this server's shared state for Ack/Nack routing
    #[must_use]
    pub fn state(&self) -> Option<SharedState> {
        self.state.clone()
    }
}
/// struct that implements the ArrowMetricsService trait
pub struct ArrowMetricsServiceImpl {
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<SharedState>,
    settings: Settings,
}

impl ArrowMetricsServiceImpl {
    /// create a new ArrowMetricsServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            effect_handler,
            state: settings
                .wait_for_result
                .then(|| SharedState::new(settings.max_concurrent_requests)),
            settings: settings.clone(),
        }
    }

    /// Get this server's shared state for Ack/Nack routing
    #[must_use]
    pub fn state(&self) -> Option<SharedState> {
        self.state.clone()
    }
}

/// struct that implements the ArrowTracesService trait
pub struct ArrowTracesServiceImpl {
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<SharedState>,
    settings: Settings,
}

impl ArrowTracesServiceImpl {
    /// create a new ArrowTracesServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            effect_handler,
            state: settings
                .wait_for_result
                .then(|| SharedState::new(settings.max_concurrent_requests)),
            settings: settings.clone(),
        }
    }

    /// Get this server's shared state for Ack/Nack routing
    #[must_use]
    pub fn state(&self) -> Option<SharedState> {
        self.state.clone()
    }
}

#[tonic::async_trait]
impl ArrowLogsService for ArrowLogsServiceImpl {
    type ArrowLogsStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_logs(
        &self,
        request: Request<Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowLogsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(self.settings.response_stream_channel_size);

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        spawn_stream_handler::<Logs, _>(
            request.into_inner(),
            OtapArrowRecords::Logs,
            self.effect_handler.clone(),
            self.state.clone(),
            self.settings.clone(),
            tx,
        );

        Ok(Response::new(Box::pin(output) as Self::ArrowLogsStream))
    }
}

#[tonic::async_trait]
impl ArrowMetricsService for ArrowMetricsServiceImpl {
    type ArrowMetricsStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_metrics(
        &self,
        request: Request<Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowMetricsStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(self.settings.response_stream_channel_size);

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        spawn_stream_handler::<Metrics, _>(
            request.into_inner(),
            OtapArrowRecords::Metrics,
            self.effect_handler.clone(),
            self.state.clone(),
            self.settings.clone(),
            tx,
        );

        Ok(Response::new(Box::pin(output) as Self::ArrowMetricsStream))
    }
}

#[tonic::async_trait]
impl ArrowTracesService for ArrowTracesServiceImpl {
    type ArrowTracesStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_traces(
        &self,
        request: Request<Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowTracesStream>, Status> {
        let (tx, rx) = tokio::sync::mpsc::channel(self.settings.response_stream_channel_size);

        // create a stream to output result to
        let output = ReceiverStream::new(rx);

        spawn_stream_handler::<Traces, _>(
            request.into_inner(),
            OtapArrowRecords::Traces,
            self.effect_handler.clone(),
            self.state.clone(),
            self.settings.clone(),
            tx,
        );

        Ok(Response::new(Box::pin(output) as Self::ArrowTracesStream))
    }
}

type PendingResponseFuture = BoxFuture<'static, PendingResponse>;

enum PendingResponse {
    Ack { batch_id: i64 },
    Nack { batch_id: i64, reason: String },
    ChannelClosed { batch_id: i64 },
}

fn spawn_stream_handler<T, F>(
    input_stream: Streaming<BatchArrowRecords>,
    otap_batch: F,
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<SharedState>,
    settings: Settings,
    tx: tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
) where
    T: OtapBatchStore + Send + 'static,
    F: Fn(T) -> OtapArrowRecords + Copy + Send + 'static,
{
    _ = tokio::spawn(async move {
        handle_stream::<T, F>(
            input_stream,
            otap_batch,
            effect_handler,
            state,
            settings,
            tx,
        )
        .await;
    });
}

async fn handle_stream<T, F>(
    mut input_stream: Streaming<BatchArrowRecords>,
    otap_batch: F,
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<SharedState>,
    settings: Settings,
    tx: tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
) where
    T: OtapBatchStore + Send + 'static,
    F: Fn(T) -> OtapArrowRecords + Copy + Send + 'static,
{
    let mut consumer = Consumer::default();
    let max_pending = settings.effective_max_concurrent_requests_per_stream();
    let mut pending = FuturesUnordered::<PendingResponseFuture>::new();

    loop {
        if flush_ready_pending_responses(&mut pending, &tx)
            .await
            .is_err()
        {
            return;
        }

        while pending.len() >= max_pending {
            if let Some(response) = pending.next().await {
                if send_pending_response(response, &tx).await.is_err() {
                    return;
                }
            }
        }

        if reject_open_stream_for_memory_pressure(
            &settings.admission_state,
            settings.receiver_rejection_metrics.as_deref(),
            &tx,
        )
        .await
        {
            break;
        }

        tokio::select! {
            response = pending.next(), if !pending.is_empty() => {
                if let Some(response) = response
                    && send_pending_response(response, &tx).await.is_err()
                {
                    return;
                }
            }
            batch = input_stream.message() => {
                let batch = match batch {
                    Ok(Some(batch)) => batch,
                    Ok(None) | Err(_) => break,
                };

                match accept_data::<T, F>(
                    otap_batch,
                    &mut consumer,
                    batch,
                    &effect_handler,
                    state.clone(),
                    &settings.admission_state,
                    settings.receiver_rejection_metrics.as_deref(),
                    &tx,
                )
                .await {
                    Ok(Some(response)) => pending.push(response),
                    Ok(None) => {}
                    Err(_) => break,
                }
            }
        }
    }

    while let Some(response) = pending.next().await {
        if send_pending_response(response, &tx).await.is_err() {
            return;
        }
    }
}

async fn flush_ready_pending_responses(
    pending: &mut FuturesUnordered<PendingResponseFuture>,
    tx: &tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
) -> Result<(), ()> {
    while let Some(Some(response)) = pending.next().now_or_never() {
        send_pending_response(response, tx).await?;
    }
    Ok(())
}

async fn reject_open_stream_for_memory_pressure(
    admission_state: &SharedReceiverAdmissionState,
    rejection_metrics: Option<&dyn ReceiverRejectionMetrics>,
    tx: &tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
) -> bool {
    if !admission_state.should_shed_ingress() {
        return false;
    }

    if let Some(metrics) = rejection_metrics {
        metrics.record_memory_pressure_rejection();
    }

    otel_warn!(
        "otap.stream.memory_pressure",
        message = "Process memory pressure active while receiving an OTAP stream"
    );

    let _ = tx
        .send(Err(grpc_memory_pressure_status(admission_state)))
        .await
        .map_err(|e| {
            otel_error!(
                "otap.response.send_failed",
                error = ?e,
                message = "Error sending streamed memory pressure response"
            );
        })
        .ok();

    true
}

/// handles sending the data down the pipeline via effect_handler and generating the appropriate response
async fn accept_data<T: OtapBatchStore, F>(
    otap_batch: F,
    consumer: &mut Consumer,
    mut batch: BatchArrowRecords,
    effect_handler: &shared::EffectHandler<OtapPdata>,
    state: Option<SharedState>,
    admission_state: &SharedReceiverAdmissionState,
    rejection_metrics: Option<&dyn ReceiverRejectionMetrics>,
    tx: &tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
) -> Result<Option<PendingResponseFuture>, ()>
where
    F: Fn(T) -> OtapArrowRecords,
{
    let batch_id = batch.batch_id;
    if admission_state.should_shed_ingress() {
        if let Some(metrics) = rejection_metrics {
            metrics.record_memory_pressure_rejection();
        }

        otel_warn!(
            "otap.request.memory_pressure",
            message = "Process memory pressure active while receiving streamed batch"
        );

        tx.send(Ok(BatchStatus {
            batch_id,
            status_code: StatusCode::ResourceExhausted as i32,
            status_message: "Process memory pressure".to_string(),
        }))
        .await
        .map_err(|e| {
            otel_error!("otap.response.send_failed", error = ?e, message = "Error sending BatchStatus response");
        })?;

        return Ok(None);
    }

    let batch = consumer.consume_bar(&mut batch).map_err(|e| {
        otel_error!("otap.batch.decode_failed", error = ?e, message = "Error decoding OTAP Batch. Closing stream");
    })?;

    let batch = from_record_messages::<T>(batch).map_err(|e| {
        otel_error!("otap.batch.validation_failed", error = ?e, message = "Invalid OTAP batch. Closing stream");
    })?;
    let otap_batch_as_otap_arrow_records = otap_batch(batch);
    let mut otap_pdata =
        OtapPdata::new(Context::default(), otap_batch_as_otap_arrow_records.into());

    let cancel_rx = if let Some(state) = state {
        // Try to allocate a slot (under the mutex) for calldata.
        let allocation_result = {
            let guard_result = state.0.lock();
            match guard_result {
                Ok(mut guard) => guard.allocate(|| oneshot::channel()),
                Err(_) => {
                    otel_error!("otap.mutex.poisoned", message = "Mutex poisoned");
                    return Err(());
                }
            }
        }; // MutexGuard is dropped here

        let (key, rx) = match allocation_result {
            None => {
                otel_error!(
                    "otap.request.concurrency_limit",
                    message = "Too many concurrent requests"
                );
                if let Some(metrics) = rejection_metrics {
                    metrics.record_rejection();
                }

                // Send backpressure response
                tx.send(Ok(BatchStatus {
                    batch_id,
                    status_code: StatusCode::Unavailable as i32,
                    status_message: format!(
                        "Pipeline processing failed: {}",
                        "Too many concurrent requests"
                    ),
                }))
                .await
                .map_err(|e| {
                    otel_error!("otap.response.send_failed", error = ?e, message = "Error sending BatchStatus response");
                })?;

                return Ok(None);
            }
            Some(pair) => pair,
        };

        // Enter the subscription. Slot key becomes calldata.
        effect_handler.subscribe_to(
            Interests::ACKS | Interests::NACKS,
            key.into(),
            &mut otap_pdata,
        );
        Some((otlp::server::SlotGuard { key, state }, rx))
    } else {
        None
    };

    // Send to the pipeline. The Ack/Nack wait is returned to the stream driver
    // so the driver can continue reading up to the per-stream in-flight limit.
    match effect_handler
        .send_message_with_source_node(otap_pdata)
        .await
    {
        Ok(_) => {}
        Err(e) => {
            otel_error!("otap.pipeline.send_failed", error = ?e, message = "Failed to send to pipeline");
            return Err(());
        }
    };

    // If backpressure, await a response. The guard will cancel and return the
    // slot if Tonic times-out this task.
    if let Some((cancel_guard, rx)) = cancel_rx {
        return Ok(Some(
            wait_for_pending_response(batch_id, cancel_guard, rx).boxed(),
        ));
    }

    tx.send(Ok(BatchStatus {
        batch_id,
        status_code: StatusCode::Ok as i32,
        status_message: "Successfully received".to_string(),
    }))
    .await
    .map_err(|e| {
        otel_error!("otap.response.send_failed", error = ?e, message = "Error sending BatchStatus response");
    })?;

    Ok(None)
}

async fn wait_for_pending_response(
    batch_id: i64,
    _cancel_guard: otlp::server::SlotGuard,
    rx: oneshot::Receiver<Result<(), otap_df_engine::control::NackMsg<OtapPdata>>>,
) -> PendingResponse {
    match rx.await {
        Ok(Ok(())) => PendingResponse::Ack { batch_id },
        Ok(Err(nack)) => PendingResponse::Nack {
            batch_id,
            reason: nack.reason,
        },
        Err(_) => PendingResponse::ChannelClosed { batch_id },
    }
}

async fn send_pending_response(
    response: PendingResponse,
    tx: &tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
) -> Result<(), ()> {
    let status = match response {
        PendingResponse::Ack { batch_id } => BatchStatus {
            batch_id,
            status_code: StatusCode::Ok as i32,
            status_message: "Successfully received".to_string(),
        },
        PendingResponse::Nack { batch_id, reason } => BatchStatus {
            batch_id,
            status_code: StatusCode::Unavailable as i32,
            status_message: format!("Pipeline processing failed: {reason}"),
        },
        PendingResponse::ChannelClosed { batch_id } => {
            otel_error!(
                "otap.response.channel_closed",
                batch_id = batch_id,
                message = "Response channel closed unexpectedly"
            );
            return Err(());
        }
    };

    tx.send(Ok(status)).await.map_err(|e| {
        otel_error!("otap.response.send_failed", error = ?e, message = "Error sending BatchStatus response");
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use otap_df_config::policy::MemoryLimiterMode;
    use otap_df_engine::memory_limiter::{
        MemoryPressureBehaviorConfig, MemoryPressureLevel, MemoryPressureState,
    };
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tonic::Code;

    #[derive(Default)]
    struct CountingReceiverRejectionMetrics {
        calls: AtomicUsize,
    }

    impl ReceiverRejectionMetrics for CountingReceiverRejectionMetrics {
        fn record_rejection(&self) {
            let _ = self.calls.fetch_add(1, Ordering::Relaxed);
        }
    }

    #[tokio::test]
    async fn flush_ready_pending_responses_drains_ready_without_waiting() {
        let mut pending = FuturesUnordered::<PendingResponseFuture>::new();
        pending.push(futures::future::ready(PendingResponse::Ack { batch_id: 1_i64 }).boxed());
        pending.push(futures::future::pending::<PendingResponse>().boxed());
        pending.push(
            futures::future::ready(PendingResponse::Nack {
                batch_id: 2_i64,
                reason: "rejected".to_string(),
            })
            .boxed(),
        );
        let (tx, mut rx) = tokio::sync::mpsc::channel(4);

        flush_ready_pending_responses(&mut pending, &tx)
            .await
            .expect("ready responses should be flushed");

        assert_eq!(pending.len(), 1);
        let mut statuses = [
            rx.recv()
                .await
                .expect("first status should be sent")
                .expect("first status should be ok"),
            rx.recv()
                .await
                .expect("second status should be sent")
                .expect("second status should be ok"),
        ];
        statuses.sort_by_key(|status| status.batch_id);

        assert_eq!(statuses[0].batch_id, 1);
        assert_eq!(statuses[0].status_code, StatusCode::Ok as i32);
        assert_eq!(statuses[1].batch_id, 2);
        assert_eq!(statuses[1].status_code, StatusCode::Unavailable as i32);
        assert!(rx.try_recv().is_err());
    }

    #[tokio::test]
    async fn open_stream_rejection_stops_before_reading_next_batch() {
        let state = MemoryPressureState::default();
        state.configure(MemoryPressureBehaviorConfig {
            retry_after_secs: 3,
            fail_readiness_on_hard: true,
            mode: MemoryLimiterMode::Enforce,
        });
        state.set_level_for_tests(MemoryPressureLevel::Hard);

        let metrics = CountingReceiverRejectionMetrics::default();
        let local_state = SharedReceiverAdmissionState::from_process_state(&state);
        let (tx, mut rx) = tokio::sync::mpsc::channel(1);

        assert!(
            reject_open_stream_for_memory_pressure(&local_state, Some(&metrics), &tx).await,
            "hard pressure should reject an already-open stream before reading the next batch"
        );

        let response = rx.recv().await.expect("stream rejection should be emitted");
        let status = response.expect_err("memory pressure should surface as a gRPC stream error");
        assert_eq!(status.code(), Code::ResourceExhausted);
        assert_eq!(
            status
                .metadata()
                .get("grpc-retry-pushback-ms")
                .and_then(|value| value.to_str().ok()),
            Some("3000")
        );
        assert_eq!(metrics.calls.load(Ordering::Relaxed), 1);
    }
}

/// Enum to describe the Arrow data.
///
/// Within this type, the Arrow batches are serialized as Arrow IPC inside the
/// `arrow_payloads` field on `[BatchArrowRecords]`
#[derive(Debug, Clone)]
pub enum OtapArrowBytes {
    /// Metrics object
    ArrowMetrics(BatchArrowRecords),
    /// Logs object
    ArrowLogs(BatchArrowRecords),
    /// Trace object
    ArrowTraces(BatchArrowRecords),
}
