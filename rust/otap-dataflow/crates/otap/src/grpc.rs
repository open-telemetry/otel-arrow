// SPDX-License-Identifier: Apache-2.0

//!
//! Provides a set of structs and enums that interact with the gRPC Server with BiDirectional streaming
//!
//! Implements the necessary service traits for OTLP data
//!
//! ToDo: Modify OTAPData -> Optimize message transport
//! ToDo: Handle Ack and Nack, return proper batch status
//!

use crate::proto::opentelemetry::experimental::arrow::v1::{
    BatchArrowRecords, BatchStatus, StatusCode, arrow_logs_service_server::ArrowLogsService,
    arrow_metrics_service_server::ArrowMetricsService,
    arrow_traces_service_server::ArrowTracesService,
};
use otap_df_engine::shared::receiver as shared;
use std::pin::Pin;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::codec::CompressionEncoding;
use tonic::{Request, Response, Status};

/// struct that implements the ArrowLogsService trait
pub struct ArrowLogsServiceImpl {
    effect_handler: shared::EffectHandler<OTAPData>,
}

impl ArrowLogsServiceImpl {
    /// create a new ArrowLogsServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTAPData>) -> Self {
        Self { effect_handler }
    }
}
/// struct that implements the ArrowMetricsService trait
pub struct ArrowMetricsServiceImpl {
    effect_handler: shared::EffectHandler<OTAPData>,
}

impl ArrowMetricsServiceImpl {
    /// create a new ArrowMetricsServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTAPData>) -> Self {
        Self { effect_handler }
    }
}

/// struct that implements the ArrowTracesService trait
pub struct ArrowTracesServiceImpl {
    effect_handler: shared::EffectHandler<OTAPData>,
}

impl ArrowTracesServiceImpl {
    /// create a new ArrowTracesServiceImpl struct with a sendable effect handler
    #[must_use]
    pub fn new(effect_handler: shared::EffectHandler<OTAPData>) -> Self {
        Self { effect_handler }
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
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let effect_handler_clone = self.effect_handler.clone();

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match effect_handler_clone
                    .send_message(OTAPData::ArrowLogs(batch))
                    .await
                {
                    Ok(_) => (StatusCode::Ok, "Successfully received".to_string()),
                    Err(error) => (StatusCode::Canceled, error.to_string()),
                };

                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id: batch_id,
                        status_code: status_result.0 as i32,
                        status_message: status_result.1,
                    }))
                    .await;
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
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let effect_handler_clone = self.effect_handler.clone();

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match effect_handler_clone
                    .send_message(OTAPData::ArrowMetrics(batch))
                    .await
                {
                    Ok(_) => (StatusCode::Ok, "Successfully received".to_string()),
                    Err(error) => (StatusCode::Canceled, error.to_string()),
                };
                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id: batch_id,
                        status_code: status_result.0 as i32,
                        status_message: status_result.1,
                    }))
                    .await;
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
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let effect_handler_clone = self.effect_handler.clone();

        // create a stream to output result to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match effect_handler_clone
                    .send_message(OTAPData::ArrowTraces(batch))
                    .await
                {
                    Ok(_) => (StatusCode::Ok, "Successfully received".to_string()),
                    Err(error) => (StatusCode::Canceled, error.to_string()),
                };
                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id: batch_id,
                        status_code: status_result.0 as i32,
                        status_message: status_result.1,
                    }))
                    .await;
            }
        });

        Ok(Response::new(Box::pin(output) as Self::ArrowTracesStream))
    }
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

/// Enum to represent various compression methods
#[derive(Debug)]
pub enum CompressionMethod {
    /// Fastest compression
    Zstd,
    /// Most compatible compression method
    Gzip,
    /// Used for legacy systems
    Deflate,
}

impl CompressionMethod {
    /// map the compression method to the proper tonic compression encoding equivalent
    /// use the CompressionMethod enum to abstract from tonic
    #[must_use]
    pub fn map_to_compression_encoding(&self) -> CompressionEncoding {
        match *self {
            CompressionMethod::Gzip => CompressionEncoding::Gzip,
            CompressionMethod::Zstd => CompressionEncoding::Zstd,
            CompressionMethod::Deflate => CompressionEncoding::Deflate,
        }
    }
}
