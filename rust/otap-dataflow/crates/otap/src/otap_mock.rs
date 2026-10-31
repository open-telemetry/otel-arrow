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
use otel_arrow_dfe_pdata::{
    Consumer,
    otap::{
        Logs, Metrics, OtapArrowRecords, Traces, from_record_messages, testing::complete_batch,
    },
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
                let batch_data = consumer
                    .consume_bar(&mut batch)
                    .expect("failed to decode log batch in mock OTAP stream");
                let pdata =
                    OtapArrowRecords::Logs(from_record_messages(batch_data).expect("valid"));
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
                let batch_data = consumer
                    .consume_bar(&mut batch)
                    .expect("failed to decode metrics batch in mock OTAP stream");
                let pdata =
                    OtapArrowRecords::Metrics(from_record_messages(batch_data).expect("valid"));
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
                let batch_data = consume
                    .consume_bar(&mut batch)
                    .expect("failed to decode trace batch in mock OTAP stream");
                let pdata =
                    OtapArrowRecords::Traces(from_record_messages(batch_data).expect("valid"));
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

/// Creates an OTAP batch with `num_rows` rows (for testing).
#[must_use]
pub fn create_otap_batch_with_rows(
    batch_id: i64,
    payload_type: ArrowPayloadType,
    num_rows: usize,
) -> OtapArrowRecords {
    assert!(num_rows > 0, "num_rows must be > 0");
    let base = u16::try_from(batch_id).expect("batch_id must fit in u16");
    let ids = UInt16Array::from_iter_values((0..num_rows).map(|i| {
        let offset = u16::try_from(i).expect("row index must fit in u16");
        base.checked_add(offset)
            .expect("batch_id + row index must fit in u16")
    }));

    let id_field = Field::new(consts::ID, DataType::UInt16, true).with_metadata(
        std::collections::HashMap::from([(
            consts::metadata::COLUMN_ENCODING.to_string(),
            consts::metadata::encodings::PLAIN.to_string(),
        )]),
    );
    let record_batch =
        RecordBatch::try_new(Arc::new(Schema::new(vec![id_field])), vec![Arc::new(ids)])
            .expect("failed to build test OTAP batch record");

    let record_batch = complete_batch(payload_type, record_batch);

    let mut otap_batch = match payload_type {
        ArrowPayloadType::Logs => OtapArrowRecords::Logs(Logs::default()),
        ArrowPayloadType::Spans => OtapArrowRecords::Traces(Traces::default()),
        ArrowPayloadType::UnivariateMetrics => OtapArrowRecords::Metrics(Metrics::default()),
        _ => panic!("unexpected payload_type"),
    };

    otap_batch.set(payload_type, record_batch).expect("valid");

    otap_batch
}

/// Creates a basic batch arrow record to use for testing
#[must_use]
pub fn create_otap_batch(batch_id: i64, payload_type: ArrowPayloadType) -> OtapArrowRecords {
    create_otap_batch_with_rows(batch_id, payload_type, 1)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: create_otap_batch_with_rows builds a multi-row OTAP batch
    /// with sequential IDs starting at batch_id for each supported payload type.
    /// Guarantees: the resulting batch contains exactly num_rows rows for the
    /// requested payload type, so benchmark workloads have the intended size.
    #[test]
    fn test_create_otap_batch_with_rows_row_count() {
        for payload_type in [
            ArrowPayloadType::Logs,
            ArrowPayloadType::Spans,
            ArrowPayloadType::UnivariateMetrics,
        ] {
            let batch = create_otap_batch_with_rows(1, payload_type, 50);
            let record_batch = batch.get(payload_type).expect("payload must be set");
            assert_eq!(record_batch.num_rows(), 50);
        }
    }

    /// Scenario: create_otap_batch delegates to the multi-row helper with a
    /// single row.
    /// Guarantees: existing callers of create_otap_batch keep receiving a
    /// one-row batch after the helper refactor.
    #[test]
    fn test_create_otap_batch_single_row() {
        let batch = create_otap_batch(1, ArrowPayloadType::Logs);
        let record_batch = batch
            .get(ArrowPayloadType::Logs)
            .expect("payload must be set");
        assert_eq!(record_batch.num_rows(), 1);
    }

    /// Scenario: a multi-row batch from create_otap_batch_with_rows goes
    /// through decode_transport_optimized_ids, as happens in exporter paths.
    /// Guarantees: the ID column is marked plain-encoded, so decoding leaves
    /// the sequential IDs intact instead of cumulative-summing them as if
    /// they were delta-encoded.
    #[test]
    fn test_create_otap_batch_with_rows_ids_survive_decode() {
        let mut batch = create_otap_batch_with_rows(1, ArrowPayloadType::Logs, 3);
        batch
            .decode_transport_optimized_ids()
            .expect("decode must succeed");
        let record_batch = batch
            .get(ArrowPayloadType::Logs)
            .expect("payload must be set");
        let ids = record_batch
            .column_by_name(consts::ID)
            .expect("ID column must exist")
            .as_any()
            .downcast_ref::<UInt16Array>()
            .expect("ID column must be UInt16");
        assert_eq!(ids.values(), &[1, 2, 3]);
    }

    /// Scenario: create_otap_batch_with_rows is called with num_rows == 0.
    /// Guarantees: the helper rejects empty batches by panicking, so
    /// benchmarks cannot silently generate zero-sized workloads.
    #[test]
    #[should_panic(expected = "num_rows must be > 0")]
    fn test_create_otap_batch_with_rows_rejects_zero_rows() {
        let _ = create_otap_batch_with_rows(1, ArrowPayloadType::Logs, 0);
    }
}
