// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// This module provides adapters for testing OTAP protocol handling.
// Metrics is the only signal implemented, therefore this code has not
// been made signal-generic, and it only supports an Output type (not
// an Input type).

use crate::proto::opentelemetry::experimental::arrow::v1::{
    arrow_metrics_service_server::{ArrowMetricsService, ArrowMetricsServiceServer},
    BatchArrowRecords, BatchStatus, StatusCode,
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
pub struct OTAPMetricsOutputType;

/// Translates OTAP arrow data to OTLP metrics using logic from the
/// top-level crate::otlp.
pub struct OTAPMetricsAdapter {
    receiver: TestReceiver<ExportMetricsServiceRequest>,
}

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

            loop {
                match input_stream.message().await {
                    Ok(Some(batch)) => {
                        let batch_id = batch.batch_id;

                        // Process each arrow payload in the batch
                        match process_arrow_metrics(&batch, &mut related_data, &receiver).await {
                            Ok(_) => {
                                // Send success status back to client
                                if tx.send(Ok(BatchStatus {
                                        batch_id,
                                        status_code: StatusCode::Ok as i32,
                                        status_message: "Successfully processed".to_string(),
                                    }))
                                    .await
                                    .is_err()
                                {
                                    break; // Client disconnected
                                }
                            }
                            Err(e) => {
                                // Send error status back to client
                                if tx.send(Ok(BatchStatus {
                                        batch_id,
                                        status_code: StatusCode::InvalidArgument as i32,
                                        status_message: format!("Failed to process: {}", e),
                                    }))
                                    .await
                                    .is_err()
                                {
                                    break; // Client disconnected
                                }
                            }
                        }
                    }
                    Ok(None) => {
                        break;
                    }
                    Err(e) => {
                        // Error receiving batch from client
                        let _ = tx
                            .send(Ok(BatchStatus {
                                batch_id: -1, // Unknown batch ID for protocol errors
                                status_code: StatusCode::Internal as i32,
                                status_message: format!("Failed to receive batch: {}", e),
                            }))
                            .await;
                        break;
                    }
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

        // Get the first (and only) batch
        let arrow_batch = reader
            .into_iter()
            .next()
            .context(error::EmptyBatchSnafu)?
            .context(error::ArrowSnafu)?;

        // Create a record message
        let record_message = RecordMessage {
            batch_id: batch.batch_id,
            schema_id: payload.schema_id.clone(),
            payload_type: crate::opentelemetry::ArrowPayloadType::try_from(payload.r#type)
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
