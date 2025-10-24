// Copyright The OpenTelemetry Authors
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

use crate::pdata::OtapPdata;
use arrow::{
    array::{RecordBatch, UInt16Array},
    datatypes::{DataType, Field, Schema},
};
use otel_arrow_rust::{
    Consumer,
    otap::{Logs, Metrics, OtapArrowRecords, Traces, from_record_messages},
    proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, BatchArrowRecords, BatchStatus, StatusCode,
        arrow_logs_service_server::ArrowLogsService,
        arrow_metrics_service_server::ArrowMetricsService,
        arrow_traces_service_server::ArrowTracesService,
    },
    schema::consts,
};
use std::{pin::Pin, sync::Arc};
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
            let mut consumer = Consumer::default();

            // Process messages until stream ends or error occurs
            while let Ok(Some(mut batch)) = input_stream.message().await {
                let batch_data = consumer.consume_bar(&mut batch).unwrap();
                let pdata = OtapArrowRecords::Logs(from_record_messages(batch_data));
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match sender_clone
                    .send(OtapPdata::new_default(pdata.into()))
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
            let mut consumer = Consumer::default();

            // Process messages until stream ends or error occurs
            while let Ok(Some(mut batch)) = input_stream.message().await {
                let batch_data = consumer.consume_bar(&mut batch).unwrap();
                let pdata = OtapArrowRecords::Metrics(from_record_messages(batch_data));
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match sender_clone
                    .send(OtapPdata::new_default(pdata.into()))
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
            let mut consume = Consumer::default();

            // Process messages until stream ends or error occurs
            while let Ok(Some(mut batch)) = input_stream.message().await {
                let batch_data = consume.consume_bar(&mut batch).unwrap();
                let pdata = OtapArrowRecords::Traces(from_record_messages(batch_data));
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                let status_result = match sender_clone
                    .send(OtapPdata::new_default(pdata.into()))
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

        Ok(Response::new(Box::pin(output) as Self::ArrowTracesStream))
    }
}

/// creates a basic batch arrow record to use for testing
#[must_use]
pub fn create_otap_batch(batch_id: i64, payload_type: ArrowPayloadType) -> OtapArrowRecords {
    let record_batch = RecordBatch::try_new(
        Arc::new(Schema::new(vec![Field::new(
            consts::ID,
            DataType::UInt16,
            true,
        )])),
        vec![Arc::new(UInt16Array::from_iter_values(vec![
            batch_id as u16,
        ]))],
    )
    .unwrap();

    let mut otap_batch = match payload_type {
        ArrowPayloadType::Logs => OtapArrowRecords::Logs(Logs::default()),
        ArrowPayloadType::Spans => OtapArrowRecords::Traces(Traces::default()),
        ArrowPayloadType::UnivariateMetrics | ArrowPayloadType::MultivariateMetrics => {
            OtapArrowRecords::Metrics(Metrics::default())
        }
        _ => {
            panic!("unexpected payload_type")
        }
    };

    otap_batch.set(payload_type, record_batch);

    otap_batch
}
