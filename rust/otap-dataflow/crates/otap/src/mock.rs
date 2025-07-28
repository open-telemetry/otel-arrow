// SPDX-License-Identifier: Apache-2.0

//!
//! Provides a set of structs and enums that interact with the gRPC Server with BiDirectional streaming
//!
//! Implements the necessary service traits for OTLP data
//!
//! ToDo: Modify OTAPData -> Optimize message transport
//! ToDo: Handle Ack and Nack, return proper batch status
//! ToDo: [LQ] Improve the pipeline test infrastructure to allow testing the tuple `pdata channel -> OTAP Exporter - grpc -> OTAP receiver -> pdata channel`
//!

use crate::{grpc::OtapArrowBytes, pdata::OtapPdata};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
    ArrowPayload, ArrowPayloadType, BatchArrowRecords, BatchStatus, StatusCode,
    arrow_logs_service_server::ArrowLogsService, arrow_metrics_service_server::ArrowMetricsService,
    arrow_traces_service_server::ArrowTracesService,
};
use std::pin::Pin;
use tokio::sync::mpsc::Sender;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;
use tonic::{Request, Response, Status};

/// struct that implements the ArrowLogsService trait
pub struct ArrowLogsServiceMock {
    sender: Sender<OtapPdata>,
}

impl ArrowLogsServiceMock {
    /// create a new ArrowLogsServiceMock struct with a sendable effect handler
    #[must_use]
    pub fn new(sender: Sender<OtapPdata>) -> Self {
        Self { sender }
    }
}
/// struct that implements the ArrowMetricsService trait
pub struct ArrowMetricsServiceMock {
    sender: Sender<OtapPdata>,
}

impl ArrowMetricsServiceMock {
    /// create a new ArrowMetricsServiceMock struct with a sendable effect handler
    #[must_use]
    pub fn new(sender: Sender<OtapPdata>) -> Self {
        Self { sender }
    }
}

/// struct that implements the ArrowTracesService trait
pub struct ArrowTracesServiceMock {
    sender: Sender<OtapPdata>,
}

impl ArrowTracesServiceMock {
    /// create a new ArrowTracesServiceMock struct with a sendable effect handler
    #[must_use]
    pub fn new(sender: Sender<OtapPdata>) -> Self {
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
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match sender_clone.send(OtapArrowBytes::ArrowLogs(batch).into()).await
                {
                    Ok(_) => (StatusCode::Ok, "Successfully received".to_string()),
                    Err(error) => (StatusCode::Canceled, error.to_string()),
                };

                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id,
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
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match sender_clone
                    .send(OtapArrowBytes::ArrowMetrics(batch).into())
                    .await
                {
                    Ok(_) => (StatusCode::Ok, "Successfully received".to_string()),
                    Err(error) => (StatusCode::Canceled, error.to_string()),
                };
                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id,
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
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result =
                    match sender_clone.send(OtapArrowBytes::ArrowTraces(batch).into()).await {
                        Ok(_) => (StatusCode::Ok, "Successfully received".to_string()),
                        Err(error) => (StatusCode::Canceled, error.to_string()),
                    };
                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id,
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
#[must_use]
pub fn create_batch_arrow_record(
    batch_id: i64,
    payload_type: ArrowPayloadType,
) -> BatchArrowRecords {
    let arrow_payload = ArrowPayload {
        schema_id: "0".to_string(),
        r#type: payload_type as i32,
        record: vec![0],
    };
    BatchArrowRecords {
        batch_id,
        arrow_payloads: vec![arrow_payload],
        headers: vec![0],
    }
}
