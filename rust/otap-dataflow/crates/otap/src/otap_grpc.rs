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

use futures::stream;
use otap_df_engine::{Interests, ProducerEffectHandlerExtension, shared::receiver as shared};
use otel_arrow_rust::{
    Consumer,
    otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces, from_record_messages},
    proto::opentelemetry::arrow::v1::{
        BatchArrowRecords, BatchStatus, StatusCode, arrow_logs_service_server::ArrowLogsService,
        arrow_metrics_service_server::ArrowMetricsService,
        arrow_traces_service_server::ArrowTracesService,
    },
};
use std::marker::PhantomData;
use std::pin::Pin;
use tokio::sync::oneshot;
use tokio_stream::Stream;
use tonic::{Request, Response, Status};

use crate::{
    otap_grpc::otlp::server::{AckSubscriptionState, SlotGuard},
    pdata::{Context, OtapPdata},
};

pub mod common;
pub mod middleware;
pub mod otlp;
pub mod receiver_settings;

pub use receiver_settings::GrpcServerSettings;

/// Common settings for OTLP receivers.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Maximum concurrent requests per receiver instance (per core).
    pub max_concurrent_requests: usize,
    /// Whether the receiver should wait.
    pub wait_for_result: bool,
}

/// struct that implements the ArrowLogsService trait
pub struct ArrowLogsServiceImpl {
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<AckSubscriptionState>,
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
        let output = stream_arrow_batch_statuses::<Logs, _>(
            input_stream,
            self.effect_handler.clone(),
            self.state.clone(),
            OtapArrowRecords::Logs,
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
        let output = stream_arrow_batch_statuses::<Metrics, _>(
            input_stream,
            self.effect_handler.clone(),
            self.state.clone(),
            OtapArrowRecords::Metrics,
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
        let output = stream_arrow_batch_statuses::<Traces, _>(
            input_stream,
            self.effect_handler.clone(),
            self.state.clone(),
            OtapArrowRecords::Traces,
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
/// emits a success or error status before continuing with the next request.
///
/// This design replaces the previous channel-plus-background-task approach. Expressing the control
/// flow as a single `Stream` keeps backpressure aligned with gRPC demand, removes the bookkeeping
/// around extra channels/tasks, and makes it easier to follow how every request progresses through
/// decoding, dispatch, and acknowledgement.
fn stream_arrow_batch_statuses<T, F>(
    input_stream: tonic::Streaming<BatchArrowRecords>,
    effect_handler: shared::EffectHandler<OtapPdata>,
    state: Option<AckSubscriptionState>,
    otap_batch: F,
) -> Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>
where
    T: OtapBatchStore + 'static,
    F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static,
{
    struct ArrowBatchStreamState<T, F>
    where
        T: OtapBatchStore + 'static,
        F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static,
    {
        input_stream: tonic::Streaming<BatchArrowRecords>,
        consumer: Consumer,
        effect_handler: shared::EffectHandler<OtapPdata>,
        state: Option<AckSubscriptionState>,
        otap_batch: F,
        finished: bool,
        _marker: PhantomData<fn() -> T>,
    }

    enum StreamStep {
        Yield(Result<BatchStatus, Status>),
        Done,
    }

    enum BatchProcessingResult {
        Emit(BatchStatus),
        Terminate,
    }

    impl<T, F> ArrowBatchStreamState<T, F>
    where
        T: OtapBatchStore + 'static,
        F: Fn(T) -> OtapArrowRecords + Send + Copy + 'static,
    {
        async fn next_item(&mut self) -> StreamStep {
            if self.finished {
                return StreamStep::Done;
            }

            match self.input_stream.message().await {
                Ok(Some(batch)) => match self.process_batch(batch).await {
                    BatchProcessingResult::Emit(status) => StreamStep::Yield(Ok(status)),
                    BatchProcessingResult::Terminate => StreamStep::Done,
                },
                Ok(None) => {
                    self.finished = true;
                    StreamStep::Done
                }
                Err(status) => {
                    self.finished = true;
                    StreamStep::Yield(Err(status))
                }
            }
        }

        async fn process_batch(&mut self, mut batch: BatchArrowRecords) -> BatchProcessingResult {
            let batch_id = batch.batch_id;

            let batch = match self.consumer.consume_bar(&mut batch) {
                Ok(batch) => batch,
                Err(e) => {
                    log::error!("Error decoding OTAP Batch: {e:?}. Closing stream");
                    return BatchProcessingResult::Terminate;
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
                        return BatchProcessingResult::Emit(BatchStatus {
                            batch_id,
                            status_code: StatusCode::Unavailable as i32,
                            status_message: format!(
                                "Pipeline processing failed: {}",
                                "Too many concurrent requests"
                            ),
                        });
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
                return BatchProcessingResult::Terminate;
            };

            if let Some((_cancel_guard, rx)) = cancel_rx {
                match rx.await {
                    Ok(Ok(())) => {}
                    Ok(Err(nack)) => {
                        return BatchProcessingResult::Emit(BatchStatus {
                            batch_id,
                            status_code: StatusCode::Unavailable as i32,
                            status_message: format!("Pipeline processing failed: {}", nack.reason),
                        });
                    }
                    Err(_) => {
                        log::error!("Response channel closed unexpectedly");
                        return BatchProcessingResult::Terminate;
                    }
                }
            }

            BatchProcessingResult::Emit(BatchStatus {
                batch_id,
                status_code: StatusCode::Ok as i32,
                status_message: "Successfully received".to_string(),
            })
        }
    }

    let state = ArrowBatchStreamState::<T, F> {
        input_stream,
        consumer: Consumer::default(),
        effect_handler,
        state,
        otap_batch,
        finished: false,
        _marker: PhantomData,
    };

    let stream = stream::unfold(state, |mut state| async move {
        match state.next_item().await {
            StreamStep::Yield(item) => Some((item, state)),
            StreamStep::Done => None,
        }
    });

    Box::pin(stream)
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
