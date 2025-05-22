// SPDX-License-Identifier: Apache-2.0

//!
//! Provides a set of structs and enums that interact with the gRPC Server with BiDirectional streaming
//!
//! Implements the necessary service traits for OTLP data
//!
//! ToDo: Modify OTAPData -> Optimize message transport
//! ToDo: Handle Ack and Nack, return proper batch status
//!

use crate::grpc::OTAPData;
use crate::proto::opentelemetry::experimental::arrow::v1::{
    ArrowPayload, BatchArrowRecords, BatchStatus, StatusCode,
    arrow_logs_service_server::ArrowLogsService, arrow_metrics_service_server::ArrowMetricsService,
    arrow_traces_service_server::ArrowTracesService,
};
use otap_df_engine::shared::receiver as shared;
use std::pin::Pin;
use tokio::sync::mpsc::Sender;
use tokio::task::spawn_local;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::codec::CompressionEncoding;
use tonic::{Request, Response, Status};

/// struct that implements the ArrowLogsService trait
pub struct ArrowLogsServiceMock {
    sender: Sender<OTAPData>,
}

impl ArrowLogsServiceMock {
    /// create a new ArrowLogsServiceMock struct with a sendable effect handler
    pub fn new(sender: Sender<OTAPData>) -> Self {
        Self { sender }
    }
}
/// struct that implements the ArrowMetricsService trait
pub struct ArrowMetricsServiceMock {
    sender: Sender<OTAPData>,
}

impl ArrowMetricsServiceMock {
    /// create a new ArrowMetricsServiceMock struct with a sendable effect handler
    pub fn new(sender: Sender<OTAPData>) -> Self {
        Self { sender }
    }
}

/// struct that implements the ArrowTracesService trait
pub struct ArrowTracesServiceMock {
    sender: Sender<OTAPData>,
}

impl ArrowTracesServiceMock {
    /// create a new ArrowTracesServiceMock struct with a sendable effect handler
    pub fn new(sender: Sender<OTAPData>) -> Self {
        Self { sender }
    }
}

#[tonic::async_trait]
impl ArrowLogsService for ArrowLogsServiceMock {
    type ArrowLogsStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_logs(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowLogsStream>, Status> {
        let mut input_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let sender_clone = self.sender.clone();

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = spawn_local(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(mut batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match sender_clone.send(OTAPData::ArrowLogs(batch)).await {
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
impl ArrowMetricsService for ArrowMetricsServiceMock {
    type ArrowMetricsStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_metrics(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowMetricsStream>, Status> {
        let mut input_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let sender_clone = self.sender.clone();

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = spawn_local(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(mut batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match sender_clone.send(OTAPData::ArrowMetrics(batch)).await {
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
impl ArrowTracesService for ArrowTracesServiceMock {
    type ArrowTracesStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_traces(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowTracesStream>, Status> {
        let mut input_stream = request.into_inner();
        let (tx, rx) = tokio::sync::mpsc::channel(100);
        let sender_clone = self.sender.clone();

        // create a stream to output result to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = spawn_local(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(mut batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match sender_clone.send(OTAPData::ArrowTraces(batch)).await {
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

/// creates a basic batch arrow record to use for testing
pub fn create_batch_arrow_record(batch_id: i64) -> BatchArrowRecords {
    let arrow_payload = ArrowPayload {
        schema_id: "0".to_string(),
        r#type: 0,
        record: vec![0],
    };
    BatchArrowRecords {
        batch_id: batch_id,
        arrow_payloads: vec![arrow_payload],
        headers: vec![0],
    }
}
