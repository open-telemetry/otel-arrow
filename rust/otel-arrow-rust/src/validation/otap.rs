// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// This module provides adapters for testing OTAP protocol handling.
// Metrics is the only signal implemented, therefore this code has not
// been made signal-generic, and it only supports an Output type (not
// an Input type).

use crate::{
    Consumer,
    proto::opentelemetry::{
        arrow::v1::{
            BatchArrowRecords, BatchStatus, StatusCode,
            arrow_logs_service_server::{ArrowLogsService, ArrowLogsServiceServer},
            arrow_metrics_service_server::{ArrowMetricsService, ArrowMetricsServiceServer},
        },
        collector::logs::v1::ExportLogsServiceRequest,
    },
};

use crate::proto::opentelemetry::collector::metrics::v1::ExportMetricsServiceRequest;

use super::service_type::{ServiceOutputType, TestReceiver};
use super::tcp_stream::ShutdownableTcpListenerStream;
use std::pin::Pin;
use tokio_stream::Stream;
use tonic::transport::Server;
use tonic::{Request, Response, Status};

use super::error;
use snafu::ResultExt;

const OTAP_PROTOCOL_NAME: &str = "otelarrow"; // matches the exporter and receiver name

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
            // Helper function to process a batch and send appropriate status
            async fn process_and_send(
                consumer: &mut crate::Consumer,
                batch: &mut BatchArrowRecords,
                receiver: &TestReceiver<ExportMetricsServiceRequest>,
                tx: &tokio::sync::mpsc::Sender<Result<BatchStatus, Status>>,
            ) -> Result<(), ()> {
                let status_result = match process_arrow_metrics(consumer, batch, receiver).await {
                    Ok(_) => (StatusCode::Ok, "Successfully processed".to_string()),
                    Err(e) => (StatusCode::InvalidArgument, truncate_error(e.to_string())),
                };

                tx.send(Ok(BatchStatus {
                    batch_id: batch.batch_id,
                    status_code: status_result.0 as i32,
                    status_message: status_result.1,
                }))
                .await
                .map_err(|_| ())
            }

            let mut consumer = crate::Consumer::default();
            // Process messages until stream ends or error occurs
            while let Ok(Some(mut batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                if process_and_send(&mut consumer, &mut batch, &receiver, &tx)
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
        OTAP_PROTOCOL_NAME
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
    consumer: &mut crate::Consumer,
    batch: &mut BatchArrowRecords,
    receiver: &TestReceiver<ExportMetricsServiceRequest>,
) -> error::Result<()> {
    let otlp_metrics = consumer
        .consume_metrics_batches(batch)
        .context(error::OTelArrowSnafu)?;
    receiver
        .process_export_request::<ExportMetricsServiceRequest>(
            Request::new(otlp_metrics),
            "metrics",
        )
        .await
        .context(error::TonicStatusSnafu)?;

    Ok(())
}

// OTAP logs service type for testing the OTAP-to-OTLP conversion
#[derive(Debug)]
#[cfg(test)]
pub struct OTAPLogsOutputType;

#[cfg(test)]
pub struct OTAPLogsAdapter {
    receiver: TestReceiver<ExportLogsServiceRequest>,
}

#[cfg(test)]
impl OTAPLogsAdapter {
    fn new(receiver: TestReceiver<ExportLogsServiceRequest>) -> Self {
        Self { receiver }
    }
}

#[tonic::async_trait]
impl ArrowLogsService for OTAPLogsAdapter {
    type ArrowLogsStream =
        Pin<Box<dyn Stream<Item = Result<BatchStatus, Status>> + Send + 'static>>;
    async fn arrow_logs(
        &self,
        request: Request<tonic::Streaming<BatchArrowRecords>>,
    ) -> Result<Response<Self::ArrowLogsStream>, Status> {
        let mut input_stream = request.into_inner();
        let receiver = self.receiver.clone();

        let (tx, rx) = tokio::sync::mpsc::channel(100);

        tokio::spawn(async move {
            let mut consumer = Consumer::default();
            while let Ok(Some(mut batch)) = input_stream.message().await {
                let status_result = match consumer.consume_logs_batches(&mut batch) {
                    Ok(otlp_logs) => {
                        receiver
                            .process_export_request::<ExportLogsServiceRequest>(
                                Request::new(otlp_logs),
                                "logs",
                            )
                            .await
                            .context(error::TonicStatusSnafu)
                            .unwrap();

                        (StatusCode::Ok, "Successfully processed".to_string())
                    }
                    Err(e) => (StatusCode::InvalidArgument, truncate_error(e.to_string())),
                };

                let tx_result = tx
                    .send(Ok(BatchStatus {
                        batch_id: batch.batch_id,
                        status_code: status_result.0 as i32,
                        status_message: status_result.1,
                    }))
                    .await;

                if tx_result.is_err() {
                    break;
                }
            }
        });

        let output_stream = tokio_stream::wrappers::ReceiverStream::new(rx);
        Ok(Response::new(
            Box::pin(output_stream) as Self::ArrowLogsStream
        ))
    }
}

impl ServiceOutputType for OTAPLogsOutputType {
    type Request = ExportLogsServiceRequest;
    type Server = ArrowLogsServiceServer<OTAPLogsAdapter>;

    fn signal() -> &'static str {
        "logs"
    }

    fn protocol() -> &'static str {
        OTAP_PROTOCOL_NAME
    }

    fn create_server(
        receiver: TestReceiver<Self::Request>,
        incoming: ShutdownableTcpListenerStream,
    ) -> tokio::task::JoinHandle<error::Result<()>> {
        tokio::spawn(async move {
            let adapter = OTAPLogsAdapter::new(receiver);
            Server::builder()
                .add_service(ArrowLogsServiceServer::new(adapter))
                .serve_with_incoming(incoming)
                .await
                .context(error::TonicTransportSnafu)
        })
    }
}

// Truncate the error message to 100 code points
fn truncate_error(err_msg: String) -> String {
    let upto = err_msg
        .char_indices()
        .map(|(i, _)| i)
        .nth(100)
        .unwrap_or(err_msg.len());

    err_msg[..upto].to_string()
}
