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

use async_trait::async_trait;
use futures::{
    StreamExt,
    stream::{self, FuturesUnordered},
};
use otap_df_engine::{
    Interests, ProducerEffectHandlerExtension, control::NackMsg, shared::receiver as shared,
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
use std::future::Future;
use std::marker::PhantomData;
use std::pin::Pin;
use std::task::{Context as TaskContext, Poll};
use tokio::sync::oneshot;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::{
    otap_grpc::otlp::server::{AckSubscriptionState, SlotGuard},
    pdata::{Context, OtapPdata},
};

pub mod client_settings;
pub mod common;
pub mod middleware;
pub mod otlp;
pub mod server_settings;

pub use client_settings::GrpcClientSettings;
pub use server_settings::GrpcServerSettings;

/// Common settings for OTLP receivers.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Maximum concurrent requests per receiver instance (per core).
    pub max_concurrent_requests: usize,
    /// Whether the receiver should wait.
    pub wait_for_result: bool,
}

/// Abstraction over inbound OTAP Arrow request streams.
#[async_trait]
pub trait ArrowRequestStream: Send + 'static {
    /// Returns the next OTAP Arrow batch in the stream.
    async fn next_message(&mut self) -> Result<Option<BatchArrowRecords>, Status>;
}

#[async_trait]
impl ArrowRequestStream for tonic::Streaming<BatchArrowRecords> {
    async fn next_message(&mut self) -> Result<Option<BatchArrowRecords>, Status> {
        self.message().await
    }
}

pub(crate) fn per_connection_limit(settings: &Settings) -> usize {
    if settings.wait_for_result {
        settings.max_concurrent_requests.max(1)
    } else {
        1
    }
}

/// struct that implements the ArrowLogsService trait
pub struct ArrowLogsServiceImpl {
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<AckSubscriptionState>,
    max_in_flight_per_connection: usize,
}

impl ArrowLogsServiceImpl {
    /// create a new ArrowLogsServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            effect_handler,
            state: settings
                .wait_for_result
                .then(|| AckSubscriptionState::new(settings.max_concurrent_requests)),
            max_in_flight_per_connection: per_connection_limit(settings),
        }
    }

    /// Get this server's shared state for Ack/Nack routing
    #[must_use]
    pub fn state(&self) -> Option<AckSubscriptionState> {
        self.state.clone()
    }
}
/// struct that implements the ArrowMetricsService trait
pub struct ArrowMetricsServiceImpl {
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<AckSubscriptionState>,
    max_in_flight_per_connection: usize,
}

impl ArrowMetricsServiceImpl {
    /// create a new ArrowMetricsServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            effect_handler,
            state: settings
                .wait_for_result
                .then(|| AckSubscriptionState::new(settings.max_concurrent_requests)),
            max_in_flight_per_connection: per_connection_limit(settings),
        }
    }

    /// Get this server's shared state for Ack/Nack routing
    #[must_use]
    pub fn state(&self) -> Option<AckSubscriptionState> {
        self.state.clone()
    }
}

/// struct that implements the ArrowTracesService trait
pub struct ArrowTracesServiceImpl {
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<AckSubscriptionState>,
    max_in_flight_per_connection: usize,
}

impl ArrowTracesServiceImpl {
    /// create a new ArrowTracesServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OtapPdata>, settings: &Settings) -> Self {
        Self {
            effect_handler,
            state: settings
                .wait_for_result
                .then(|| AckSubscriptionState::new(settings.max_concurrent_requests)),
            max_in_flight_per_connection: per_connection_limit(settings),
        }
    }

    /// Get this server's shared state for Ack/Nack routing
    #[must_use]
    pub fn state(&self) -> Option<AckSubscriptionState> {
        self.state.clone()
    }
}

#[tonic::async_trait]
impl ArrowLogsService for ArrowLogsServiceImpl {
    type ArrowLogsStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_logs(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowLogsStream>, Status> {
        let input_stream = request.into_inner();
        let output = stream_arrow_batch_statuses::<_, Logs, _>(
            input_stream,
            self.effect_handler.clone(),
            self.state.clone(),
            OtapArrowRecords::Logs,
            self.max_in_flight_per_connection,
        );
        Ok(Response::new(output))
    }
}

#[tonic::async_trait]
impl ArrowMetricsService for ArrowMetricsServiceImpl {
    type ArrowMetricsStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_metrics(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowMetricsStream>, Status> {
        let input_stream = request.into_inner();
        let output = stream_arrow_batch_statuses::<_, Metrics, _>(
            input_stream,
            self.effect_handler.clone(),
            self.state.clone(),
            OtapArrowRecords::Metrics,
            self.max_in_flight_per_connection,
        );
        Ok(Response::new(output))
    }
}

#[tonic::async_trait]
impl ArrowTracesService for ArrowTracesServiceImpl {
    type ArrowTracesStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_traces(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowTracesStream>, Status> {
        let input_stream = request.into_inner();
        let output = stream_arrow_batch_statuses::<_, Traces, _>(
            input_stream,
            self.effect_handler.clone(),
            self.state.clone(),
            OtapArrowRecords::Traces,
            self.max_in_flight_per_connection,
        );
        Ok(Response::new(output))
    }
}

/// Streams `BatchStatus` updates for the Arrow gRPC services.
///
/// `ArrowLogsServiceImpl::arrow_logs`, `ArrowMetricsServiceImpl::arrow_metrics`, and
/// `ArrowTracesServiceImpl::arrow_traces` all delegate to this helper. Each service passes its
/// inbound `Streaming<BatchArrowRecords>` plus a converter that turns a decoded batch into the
/// signal-specific variant of `OtapArrowRecords`. The returned stream forwards every received Arrow
/// batch to the pipeline and yields the corresponding `BatchStatus` updates the OTLP Arrow clients
/// expect to read.
///
/// Internally an `ArrowBatchStreamState` pulls the next `BatchArrowRecords` from the tonic stream,
/// decodes it into `OtapPdata`, and optionally registers an `AckSubscriptionState` slot when
/// `wait_for_result` is enabled. Once the pipeline acknowledges (or rejects) the batch, the stream
/// emits a success or error status before continuing with the next request. To avoid per-connection
/// serialization, the state now keeps up to `max_in_flight_per_connection` batches in flight: it
/// eagerly reads, decodes, and dispatches new Arrow batches while prior ones wait for ACK/NACK
/// responses, only falling back to serialized processing once the limit is reached.
///
/// This design replaces the previous channel-plus-background-task approach. Expressing the control
/// flow as a single `Stream` keeps backpressure aligned with gRPC demand, removes the bookkeeping
/// around extra channels/tasks, and makes it easier to follow how every request progresses through
/// decoding, dispatch, acknowledgement, and now limited parallelism.
pub(crate) fn stream_arrow_batch_statuses<S, T, F>(
    input_stream: S,
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<AckSubscriptionState>,
    otap_batch: F,
    max_in_flight_per_connection: usize,
) -> Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>
where
    S: ArrowRequestStream + Send,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static,
{
    let state = ArrowBatchStreamState::new(
        input_stream,
        effect_handler,
        state,
        otap_batch,
        max_in_flight_per_connection,
    );
    Box::pin(build_status_stream(state).boxed())
}

fn build_status_stream<S, T, F>(
    state: ArrowBatchStreamState<S, T, F>,
) -> impl Stream<Item = Result<BatchStatus, Status>>
where
    S: ArrowRequestStream + Send,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static,
{
    stream::unfold(state, |mut state| async move {
        match state.next_item().await {
            StreamStep::Yield(item) => Some((item, state)),
            StreamStep::Done => None,
        }
    })
}

pub(crate) struct ArrowBatchStreamState<S, T, F>
where
    S: ArrowRequestStream + Send,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static,
{
    input_stream: S,
    consumer: Consumer,
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<AckSubscriptionState>,
    otap_batch: F,
    in_flight: FuturesUnordered<AckWaitFuture>,
    max_in_flight: usize,
    finished: bool,
    _marker: PhantomData<fn() -> T>,
}

enum StreamStep {
    Yield(Result<BatchStatus, Status>),
    Done,
}

enum PreparedBatch {
    Enqueued,
    Immediate(StreamStep),
}

impl<S, T, F> ArrowBatchStreamState<S, T, F>
where
    S: ArrowRequestStream + Send,
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static,
{
    fn new(
        input_stream: S,
        effect_handler: shared::EffectHandler<OtapPdata>,
        state: Option<AckSubscriptionState>,
        otap_batch: F,
        max_in_flight_per_connection: usize,
    ) -> Self {
        Self {
            input_stream,
            consumer: Consumer::default(),
            effect_handler,
            state,
            otap_batch,
            in_flight: FuturesUnordered::new(),
            max_in_flight: max_in_flight_per_connection.max(1),
            finished: false,
            _marker: PhantomData,
        }
    }

    async fn next_item(&mut self) -> StreamStep {
        if let Some(step) = self.fill_inflight().await {
            return step;
        }

        match self.in_flight.next().await {
            Some(step) => {
                if matches!(step, StreamStep::Done) {
                    self.finished = true;
                }
                step
            }
            None => StreamStep::Done,
        }
    }

    async fn fill_inflight(&mut self) -> Option<StreamStep> {
        while !self.finished && self.in_flight.len() < self.max_in_flight {
            match self.input_stream.next_message().await {
                Ok(Some(batch)) => match self.enqueue_batch(batch).await {
                    PreparedBatch::Enqueued => continue,
                    PreparedBatch::Immediate(step) => return Some(step),
                },
                Ok(None) => {
                    self.finished = true;
                    break;
                }
                Err(status) => {
                    self.finished = true;
                    return Some(StreamStep::Yield(Err(status)));
                }
            }
        }
        None
    }

    async fn enqueue_batch(&mut self, mut batch: BatchArrowRecords) -> PreparedBatch {
        let batch_id = batch.batch_id;

        let batch = match self.consumer.consume_bar(&mut batch) {
            Ok(batch) => batch,
            Err(e) => {
                log::error!("Error decoding OTAP Batch: {e:?}. Closing stream");
                self.finished = true;
                return PreparedBatch::Immediate(StreamStep::Done);
            }
        };

        let batch = from_record_messages::<T>(batch);
        let otap_batch_as_otap_arrow_records = (self.otap_batch)(batch);
        let mut otap_pdata =
            OtapPdata::new(Context::default(), otap_batch_as_otap_arrow_records.into());

        let cancel_rx = if let Some(state) = self.state.clone() {
            let allocation_result = state.0.lock().allocate(|| oneshot::channel());
            let (key, rx) = match allocation_result {
                None => {
                    log::error!("Too many concurrent requests");
                    return PreparedBatch::Immediate(StreamStep::Yield(Ok(BatchStatus {
                        batch_id,
                        status_code: StatusCode::Unavailable as i32,
                        status_message: "Pipeline processing failed: Too many concurrent requests"
                            .to_string(),
                    })));
                }
                Some(pair) => pair,
            };

            self.effect_handler.subscribe_to(
                Interests::ACKS | Interests::NACKS,
                key.into(),
                &mut otap_pdata,
            );
            Some((SlotGuard { key, state }, rx))
        } else {
            None
        };

        if let Err(e) = self.effect_handler.send_message(otap_pdata).await {
            log::error!("Failed to send to pipeline: {e}");
            self.finished = true;
            return PreparedBatch::Immediate(StreamStep::Done);
        };

        if let Some((cancel_guard, rx)) = cancel_rx {
            self.in_flight
                .push(AckWaitFuture::new(batch_id, cancel_guard, rx));
            PreparedBatch::Enqueued
        } else {
            PreparedBatch::Immediate(StreamStep::Yield(Ok(success_status(batch_id))))
        }
    }
}

struct AckWaitFuture {
    batch_id: i64,
    cancel_guard: Option<SlotGuard>,
    rx: oneshot::Receiver<Result<(), NackMsg<OtapPdata>>>,
}

impl AckWaitFuture {
    fn new(
        batch_id: i64,
        cancel_guard: SlotGuard,
        rx: oneshot::Receiver<Result<(), NackMsg<OtapPdata>>>,
    ) -> Self {
        Self {
            batch_id,
            cancel_guard: Some(cancel_guard),
            rx,
        }
    }
}

impl Future for AckWaitFuture {
    type Output = StreamStep;

    fn poll(self: Pin<&mut Self>, cx: &mut TaskContext<'_>) -> Poll<Self::Output> {
        let this = self.get_mut();
        let rx = Pin::new(&mut this.rx);
        match rx.poll(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(result) => {
                let _ = this.cancel_guard.take();
                let step = match result {
                    Ok(Ok(())) => StreamStep::Yield(Ok(success_status(this.batch_id))),
                    Ok(Err(nack)) => StreamStep::Yield(Ok(nack_status(this.batch_id, nack.reason))),
                    Err(_) => {
                        log::error!("Response channel closed unexpectedly");
                        StreamStep::Done
                    }
                };
                Poll::Ready(step)
            }
        }
    }
}

fn success_status(batch_id: i64) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: StatusCode::Ok as i32,
        status_message: "Successfully received".to_string(),
    }
}

fn nack_status(batch_id: i64, reason: String) -> BatchStatus {
    BatchStatus {
        batch_id,
        status_code: StatusCode::Unavailable as i32,
        status_message: format!("Pipeline processing failed: {reason}"),
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
