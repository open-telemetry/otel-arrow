// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//!
//! Provides a set of structs and enums that interact with the gRPC Server with BiDirectional streaming
//!
//! Implements the necessary service traits for OTLP data
//!
//! ToDo: Modify OTAPData -> Optimize message transport
//! ToDo: Handle Ack and Nack, return proper batch status
//! ToDo: Change how channel sizes are handled? Currently defined when creating otap_receiver -> passing channel size to the ServiceImpl
//!

use otap_df_engine::{Interests, ProducerEffectHandlerExtension, shared::receiver as shared};
use otap_df_pdata::{
    Consumer,
    otap::{Logs, Metrics, OtapArrowRecords, OtapBatchStore, Traces, from_record_messages},
    proto::opentelemetry::arrow::v1::{
        BatchArrowRecords, BatchStatus, StatusCode, arrow_logs_service_server::ArrowLogsService,
        arrow_metrics_service_server::ArrowMetricsService,
        arrow_traces_service_server::ArrowTracesService,
    },
};
use std::pin::Pin;
use tokio::sync::oneshot;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

use crate::{
    otap_grpc::otlp::server::{SharedState, SlotGuard},
    pdata::{Context, OtapPdata},
};

pub mod client_settings;
pub mod middleware;
pub mod otlp;
pub mod server_settings;

/// Common settings for OTLP receivers.
#[derive(Clone, Debug)]
pub struct Settings {
    /// Size of the channel used to buffer outgoing responses to the client.
    pub response_stream_channel_size: usize,
    /// Maximum concurrent requests per receiver instance (per core).
    pub max_concurrent_requests: usize,
    /// Whether the receiver should wait.
    pub wait_for_result: bool,
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
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowLogsStream>, Status> {
        let mut input_stream = request.into_inner();
        // ToDo [LQ] How can we abstract this to avoid any dependency on Tokio inside receiver implementations.
        let (tx, rx) = tokio::sync::mpsc::channel(self.settings.response_stream_channel_size);
        let effect_handler_clone = self.effect_handler.clone();
        let state_clone = self.state.clone();

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        // ToDo [LQ] How can we abstract this to avoid any dependency on Tokio inside receiver implementations.
        _ = tokio::spawn(async move {
            let mut consumer = Consumer::default();

            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // accept the batch data and handle output response
                if accept_data::<Logs, _>(
                    OtapArrowRecords::Logs,
                    &mut consumer,
                    batch,
                    &effect_handler_clone,
                    state_clone.clone(),
                    &tx,
                )
                .await
                .is_err()
                {
                    // end loop if error occurs
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(output) as Self::ArrowLogsStream))
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
        let mut input_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(self.settings.response_stream_channel_size);
        let effect_handler_clone = self.effect_handler.clone();
        let state_clone = self.state.clone();

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            let mut consumer = Consumer::default();

            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // accept the batch data and handle output response
                if accept_data::<Metrics, _>(
                    OtapArrowRecords::Metrics,
                    &mut consumer,
                    batch,
                    &effect_handler_clone,
                    state_clone.clone(),
                    &tx,
                )
                .await
                .is_err()
                {
                    // end loop if error occurs
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(output) as Self::ArrowMetricsStream))
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
        let mut input_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(self.settings.response_stream_channel_size);
        let effect_handler_clone = self.effect_handler.clone();
        let state_clone = self.state.clone();

        // create a stream to output result to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            let mut consumer = Consumer::default();

            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // accept the batch data and handle output response
                if accept_data::<Traces, _>(
                    OtapArrowRecords::Traces,
                    &mut consumer,
                    batch,
                    &effect_handler_clone,
                    state_clone.clone(),
                    &tx,
                )
                .await
                .is_err()
                {
                    // end loop if error occurs
                    break;
                }
            }
        });

        Ok(Response::new(Box::pin(output) as Self::ArrowTracesStream))
    }
}

/// handles sending the data down the pipeline via effect_handler and generating the appropriate response
async fn accept_data<T: OtapBatchStore, F>(
    otap_batch: F,
    consumer: &mut Consumer,
    mut batch: BatchArrowRecords,
    effect_handler: &shared::EffectHandler<OtapPdata>,
    state: Option<SharedState>,
    tx: &tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
) -> Result<(), ()>
where
    F: Fn(T) -> OtapArrowRecords,
{
    let batch_id = batch.batch_id;
    let batch = consumer.consume_bar(&mut batch).map_err(|e| {
        log::error!("Error decoding OTAP Batch: {e:?}. Closing stream");
    })?;

    let batch = from_record_messages::<T>(batch);
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
                    log::error!("Mutex poisoned");
                    return Err(());
                }
            }
        }; // MutexGuard is dropped here

        let (key, rx) = match allocation_result {
            None => {
                log::error!("Too many concurrent requests");

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
                    log::error!("Error sending BatchStatus response: {e:?}");
                })?;

                return Ok(());
            }
            Some(pair) => pair,
        };

        // Enter the subscription. Slot key becomes calldata.
        effect_handler.subscribe_to(
            Interests::ACKS | Interests::NACKS,
            key.into(),
            &mut otap_pdata,
        );
        Some((SlotGuard { key, state }, rx))
    } else {
        None
    };

    // Send and wait for Ack/Nack
    match effect_handler.send_message(otap_pdata).await {
        Ok(_) => {}
        Err(e) => {
            log::error!("Failed to send to pipeline: {e}");
            return Err(());
        }
    };

    // If backpressure, await a response. The guard will cancel and return the
    // slot if Tonic times-out this task.
    if let Some((_cancel_guard, rx)) = cancel_rx {
        match rx.await {
            Ok(Ok(())) => {
                // Received Ack
                // Behavior is similar to `wait_for_result` set to `false` case
                // No need to send a response here since success response is sent
                // before returning from the function anyway
            }
            Ok(Err(nack)) => {
                // Received Nack
                // TODO: Use more specific status codes based on nack reason/type
                // when more detailed error information is available from the pipeline
                tx.send(Ok(BatchStatus {
                    batch_id,
                    status_code: StatusCode::Unavailable as i32,
                    status_message: format!("Pipeline processing failed: {}", nack.reason),
                }))
                .await
                .map_err(|e| {
                    log::error!("Error sending BatchStatus response: {e:?}");
                })?;

                return Ok(());
            }
            Err(_) => {
                log::error!("Response channel closed unexpectedly");
                return Err(());
            }
        }
    }

    tx.send(Ok(BatchStatus {
        batch_id,
        status_code: StatusCode::Ok as i32,
        status_message: "Successfully received".to_string(),
    }))
    .await
    .map_err(|e| {
        log::error!("Error sending BatchStatus response: {e:?}");
    })
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
