// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// This module provides adapters for testing OTAP protocol handling.
// Metrics is the only signal implemented, therefore this code has not
// been made signal-generic, and it only supports an Output type (not
// an Input type).

use crate::proto::opentelemetry::arrow::v1::{
    BatchArrowRecords, BatchStatus, StatusCode,
    arrow_metrics_service_server::{ArrowMetricsService, ArrowMetricsServiceServer},
};

use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;

use super::service_type::{ServiceOutputType, TestReceiver};
use super::tcp_stream::ShutdownableTcpListenerStream;
use std::pin::Pin;
use tokio_stream::Stream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use super::error;
use snafu::{OptionExt, ResultExt};

/// OTAP metrics service type for testing the OTAP-to-OTLP conversion
/// for metrics in this crate.
#[derive(Debug)]
#[cfg(test)]
pub struct OTAPMetricsOutputType;

/// Translates OTAP arrow data to OTLP metrics using logic from the
/// top-level crate::otlp.
#[cfg(test)]
pub struct OTAPMetricsAdapter {
    receiver: TestReceiver<ExportMetricsServiceRequest>,
}

#[cfg(test)]
impl OTAPMetricsAdapter {
    fn new(receiver: TestReceiver<ExportMetricsServiceRequest>) -> Self {
        Self { receiver }
    }
}

#[tonic::async_trait]
impl ArrowMetricsService for OTAPMetricsAdapter {
    type ArrowMetricsStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;

    async fn arrow_metrics(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowMetricsStream>, Status> {
        let mut input_stream = request.into_inner();
        let receiver = self.receiver.clone();

        // Create a channel to send batch statuses back to the client
        let (tx, rx) = tokio::sync::mpsc::channel(100); // TODO?

        // Spawn a task to process incoming arrow records and convert them to OTLP
        tokio::spawn(async move {
            let mut related_data = crate::otlp::related_data::RelatedData::default();

            // Helper function to process a batch and send appropriate status
            async fn process_and_send(
                batch: &BatchArrowRecords,
                related_data: &mut crate::otlp::related_data::RelatedData,
                receiver: &TestReceiver<ExportMetricsServiceRequest>,
                tx: &tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
            ) -> Result<(), ()> {
                let status_result = match process_arrow_metrics(batch, related_data, receiver).await
                {
                    Ok(_) => (StatusCode::Ok, "Successfully processed".to_string()),
                    Err(e) => {
                        // Truncate the error message to 100 code points
                        // There are no std helpers for this (on purpose)
                        let err_msg = e.to_string();
                        let upto = err_msg
                            .char_indices()
                            .map(|(i, _)| i)
                            .nth(100)
                            .unwrap_or(err_msg.len());

                        (StatusCode::InvalidArgument, err_msg[..upto].to_string())
                    }
                };

                tx.send(Ok(BatchStatus {
                    batch_id: batch.batch_id,
                    status_code: status_result.0 as i32,
                    status_message: status_result.1,
                }))
                .await
                .map_err(|_| ())
            }

            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                if process_and_send(&batch, &mut related_data, &receiver, &tx)
                    .await
                    .is_err()
                {
                    break;
                }
            }
        });

        // Convert the mpsc receiver to a Stream
        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::ArrowMetricsStream
        ))
    }
}

impl ServiceOutputType for OTAPMetricsOutputType {
    type Request = ExportMetricsServiceRequest;
    type Server = ArrowMetricsServiceServer<OTAPMetricsAdapter>;

    fn signal() -> &'static str {
        "metrics"
    }

    fn protocol() -> &'static str {
        "otelarrow" // matches the exporter and receiver name
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: ShutdownableTcpListenerStream,
    ) -> tokio::task::JoinHandle<error::Result<()>> {
        tokio::spawn(async move {
            let adapter = OTAPMetricsAdapter::new(receiver);
            Server::builder()
                .add_service(ArrowMetricsServiceServer::new(adapter))
                .serve_with_incoming(incoming)
                .await
                .context(error::TonicTransportSnafu)
        })
    }
}

/// Receives an Arrow batch and convert to OTLP.
async fn process_arrow_metrics(
    batch: &BatchArrowRecords,
    related_data: &mut crate::otlp::related_data::RelatedData,
    receiver: &TestReceiver<ExportMetricsServiceRequest>,
) -> error::Result<()> {
    use crate::decode::record_message::RecordMessage;
    use arrow::ipc::reader::StreamReader;
    use std::io::Cursor;

    // Process each arrow payload
    for payload in &batch.arrow_payloads {
        // Create a reader for the Arrow record
        let reader =
            StreamReader::try_new(Cursor::new(&payload.record), None).context(error::ArrowSnafu)?;

        // We expect only one batch per payload field.
        let arrow_batch = reader
            .into_iter()
            .next()
            .context(error::EmptyBatchSnafu)?
            .context(error::ArrowSnafu)?;

        // Create a record message
        let record_message = RecordMessage {
            batch_id: batch.batch_id,
            schema_id: payload.schema_id.clone(),
            payload_type: crate::proto::opentelemetry::arrow::v1::ArrowPayloadType::try_from(
                payload.r#type,
            )
            .context(error::InvalidPayloadSnafu)?,
            record: arrow_batch,
        };

        // Convert Arrow payload to OTLP metrics
        let otlp_metrics = crate::otlp::metric::metrics_from(&record_message.record, related_data)
            .context(error::OTelArrowSnafu)?;

        // Send this individual metrics item to the receiver
        receiver
            .process_export_request::<ExportMetricsServiceRequest>(
                Request::new(otlp_metrics),
                "metrics",
            )
            .await
            .context(error::TonicStatusSnafu)?;
    }

    Ok(())
}
