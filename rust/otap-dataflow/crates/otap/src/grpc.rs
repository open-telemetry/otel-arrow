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

use otap_df_engine::shared::receiver as shared;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
    BatchArrowRecords, BatchStatus, StatusCode, arrow_logs_service_server::ArrowLogsService,
    arrow_metrics_service_server::ArrowMetricsService,
    arrow_traces_service_server::ArrowTracesService,
};
use std::pin::Pin;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

/// struct that implements the ArrowLogsService trait
pub struct ArrowLogsServiceImpl {
    effect_handler: shared::EffectHandler<OTAPData>,
    channel_size: usize,
}

impl ArrowLogsServiceImpl {
    /// create a new ArrowLogsServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTAPData>, channel_size: usize) -> Self {
        Self {
            effect_handler,
            channel_size,
        }
    }
}
/// struct that implements the ArrowMetricsService trait
pub struct ArrowMetricsServiceImpl {
    effect_handler: shared::EffectHandler<OTAPData>,
    channel_size: usize,
}

impl ArrowMetricsServiceImpl {
    /// create a new ArrowMetricsServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTAPData>, channel_size: usize) -> Self {
        Self {
            effect_handler,
            channel_size,
        }
    }
}

/// struct that implements the ArrowTracesService trait
pub struct ArrowTracesServiceImpl {
    effect_handler: shared::EffectHandler<OTAPData>,
    channel_size: usize,
}

impl ArrowTracesServiceImpl {
    /// create a new ArrowTracesServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTAPData>, channel_size: usize) -> Self {
        Self {
            effect_handler,
            channel_size,
        }
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
        let (tx, rx) = tokio::sync::mpsc::channel(self.channel_size);
        let effect_handler_clone = self.effect_handler.clone();

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        // ToDo [LQ] How can we abstract this to avoid any dependency on Tokio inside receiver implementations.
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // accept the batch data and handle output response
                if accept_data(OTAPData::ArrowLogs, batch, &effect_handler_clone, &tx)
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
        let (tx, rx) = tokio::sync::mpsc::channel(self.channel_size);
        let effect_handler_clone = self.effect_handler.clone();

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // accept the batch data and handle output response
                if accept_data(OTAPData::ArrowMetrics, batch, &effect_handler_clone, &tx)
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
        let (tx, rx) = tokio::sync::mpsc::channel(self.channel_size);
        let effect_handler_clone = self.effect_handler.clone();

        // create a stream to output result to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // accept the batch data and handle output response
                if accept_data(OTAPData::ArrowTraces, batch, &effect_handler_clone, &tx)
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

/// handles sending the data down the pipeline via effect_handler and generating the approriate response
async fn accept_data<OTAPDataType>(
    otap_data: OTAPDataType,
    batch: BatchArrowRecords,
    effect_handler: &shared::EffectHandler<OTAPData>,
    tx: &tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
) -> Result<(), ()>
where
    OTAPDataType: Fn(BatchArrowRecords) -> OTAPData,
{
    let batch_id = batch.batch_id;
    let status_result = match effect_handler.send_message(otap_data(batch)).await {
        Ok(_) => (StatusCode::Ok, "Successfully received".to_string()),
        Err(error) => (StatusCode::Canceled, error.to_string()),
    };
    // ToDo Add Ack/Nack management once supported by the pipeline engine.
    tx.send(Ok(BatchStatus {
        batch_id,
        status_code: status_result.0 as i32,
        status_message: status_result.1,
    }))
    .await
    .map_err(|_| ())
}

/// Enum to describe the Arrow data
#[derive(Debug, Clone)]
pub enum OTAPData {
    /// Metrics object
    ArrowMetrics(BatchArrowRecords),
    /// Logs object
    ArrowLogs(BatchArrowRecords),
    /// Trace object
    ArrowTraces(BatchArrowRecords),
}
