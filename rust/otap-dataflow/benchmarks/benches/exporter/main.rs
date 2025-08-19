// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This benchmark compares the performance of different perf exporter configurations

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use fluke_hpack::Encoder;
use mimalloc::MiMalloc;
use otap_df_channel::mpsc;
use otap_df_engine::node::NodeWithPDataReceiver;
use otap_df_engine::{
    config::ExporterConfig,
    exporter::ExporterWrapper,
    message::{Receiver, Sender},
};
use otap_df_otap::{
    grpc::OtapArrowBytes,
    otap_exporter::OTAPExporter,
    perf_exporter::{config::Config, exporter::PerfExporter},
};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::{
    ArrowPayload, ArrowPayloadType, BatchArrowRecords, BatchStatus, StatusCode,
    arrow_logs_service_server::{ArrowLogsService, ArrowLogsServiceServer},
    arrow_metrics_service_server::{ArrowMetricsService, ArrowMetricsServiceServer},
    arrow_traces_service_server::{ArrowTracesService, ArrowTracesServiceServer},
};

use otap_df_otlp::{
    grpc::OTLPData,
    otlp_exporter::OTLPExporter,
    proto::opentelemetry::collector::{
        logs::v1::{
            ExportLogsServiceRequest, ExportLogsServiceResponse,
            logs_service_server::{LogsService, LogsServiceServer},
        },
        metrics::v1::{
            ExportMetricsServiceRequest, ExportMetricsServiceResponse,
            metrics_service_server::{MetricsService, MetricsServiceServer},
        },
        trace::v1::{
            ExportTraceServiceRequest, ExportTraceServiceResponse,
            trace_service_server::{TraceService, TraceServiceServer},
        },
    },
};

use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task::LocalSet;
use tokio::time::Duration;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

use tonic::{Request, Response, Status};

use otap_df_config::node::NodeUserConfig;
use otap_df_engine::control::{Controllable, NodeControlMsg, pipeline_ctrl_msg_channel};
use otap_df_otap::otap_exporter::OTAP_EXPORTER_URN;
use otap_df_otap::perf_exporter::exporter::OTAP_PERF_EXPORTER_URN;
use otap_df_otlp::otlp_exporter::OTLP_EXPORTER_URN;
use std::pin::Pin;
use std::sync::Arc;
use tokio_stream::Stream;
use tokio_stream::wrappers::ReceiverStream;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const TRACES_BATCH_ID: i64 = 0;
const LOGS_BATCH_ID: i64 = 1;
const METRICS_BATCH_ID: i64 = 2;

/// struct that implements the ArrowLogsService trait
#[derive(Default)]
pub struct ArrowLogsServiceMock {}

/// struct that implements the ArrowMetricsService trait
#[derive(Default)]
pub struct ArrowMetricsServiceMock {}

/// struct that implements the ArrowTracesService trait
#[derive(Default)]
pub struct ArrowTracesServiceMock {}

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

        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id,
                        status_code: StatusCode::Ok as i32,
                        status_message: "Successfully received".to_string(),
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
        // Provide client a stream to listen to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id,
                        status_code: StatusCode::Ok as i32,
                        status_message: "Successfully received".to_string(),
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
        // create a stream to output result to
        let output = ReceiverStream::new(rx);

        // write to the channel
        _ = tokio::spawn(async move {
            // Process messages until stream ends or error occurs
            while let Ok(Some(batch)) = input_stream.message().await {
                // Process batch and send status, break on client disconnection
                let batch_id = batch.batch_id;
                _ = tx
                    .send(Ok(BatchStatus {
                        batch_id,
                        status_code: StatusCode::Ok as i32,
                        status_message: "Successfully received".to_string(),
                    }))
                    .await;
            }
        });
        Ok(Response::new(Box::pin(output) as Self::ArrowTracesStream))
    }
}

/// struct that implements the Log Service trait
#[derive(Default)]
pub struct LogsServiceMock {}

/// struct that implements the Metrics Service trait
#[derive(Default)]
pub struct MetricsServiceMock {}

/// struct that implements the Trace Service trait
#[derive(Default)]
pub struct TraceServiceMock {}

#[tonic::async_trait]
impl LogsService for LogsServiceMock {
    async fn export(
        &self,
        _request: Request<ExportLogsServiceRequest>,
    ) -> Result<Response<ExportLogsServiceResponse>, Status> {
        Ok(Response::new(ExportLogsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl MetricsService for MetricsServiceMock {
    async fn export(
        &self,
        _request: Request<ExportMetricsServiceRequest>,
    ) -> Result<Response<ExportMetricsServiceResponse>, Status> {
        Ok(Response::new(ExportMetricsServiceResponse {
            partial_success: None,
        }))
    }
}

#[tonic::async_trait]
impl TraceService for TraceServiceMock {
    async fn export(
        &self,
        _request: Request<ExportTraceServiceRequest>,
    ) -> Result<Response<ExportTraceServiceResponse>, Status> {
        Ok(Response::new(ExportTraceServiceResponse {
            partial_success: None,
        }))
    }
}

#[must_use]
pub fn create_batch_arrow_record_helper(
    batch_id: i64,
    payload_type: ArrowPayloadType,
    message_len: usize,
    row_size: usize,
) -> BatchArrowRecords {
    // init arrow payload vec
    let mut arrow_payloads = Vec::new();
    // create ArrowPayload objects and add to the vector
    for _ in 0..message_len {
        let arrow_payload_object = ArrowPayload {
            schema_id: "0".to_string(),
            r#type: payload_type as i32,
            record: vec![1; row_size],
        };
        arrow_payloads.push(arrow_payload_object);
    }

    // create timestamp
    // unix timestamp -> number -> byte array &[u8]
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("failed to get duration");

    let secs = timestamp.as_secs().to_string();
    let nanos = timestamp.subsec_nanos().to_string();

    // string formatted with timestamp secs : timestamp subsec_nanos
    let timestamp_string = format!("{secs}:{nanos}");
    let timestamp_bytes = timestamp_string.as_bytes();

    // convert time to
    let headers = vec![(b"timestamp" as &[u8], timestamp_bytes)];
    let mut encoder = Encoder::new();
    let encoded_headers = encoder.encode(headers);
    BatchArrowRecords {
        batch_id,
        arrow_payloads,
        headers: encoded_headers,
    }
}

fn bench_exporter(c: &mut Criterion) {
    // Use a single-threaded Tokio runtime
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    // Pin the current thread to a core
    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    let core = cores.iter().last().expect("no cores found");
    _ = core_affinity::set_for_current(*core);

    // start grpc server to handle otap stream
    let grpc_addr = "127.0.0.1";
    let otap_grpc_port = portpicker::pick_unused_port().expect("No free ports");
    let otap_listening_addr: SocketAddr = format!("{grpc_addr}:{otap_grpc_port}")
        .parse()
        .expect("failed to parse otap address");
    let (otap_shutdown_sender, otap_shutdown_signal) = tokio::sync::oneshot::channel();

    let tokio_rt = Runtime::new().expect("failed to create tokio runtime");
    _ = tokio_rt.spawn(async move {
        let tcp_listener = TcpListener::bind(otap_listening_addr)
            .await
            .expect("failed to bind to otap address");
        let tcp_stream = TcpListenerStream::new(tcp_listener);
        let mock_logs_service = ArrowLogsServiceServer::new(ArrowLogsServiceMock::default());
        let mock_metrics_service =
            ArrowMetricsServiceServer::new(ArrowMetricsServiceMock::default());
        let mock_trace_service = ArrowTracesServiceServer::new(ArrowTracesServiceMock::default());
        Server::builder()
            .add_service(mock_logs_service)
            .add_service(mock_metrics_service)
            .add_service(mock_trace_service)
            .serve_with_incoming_shutdown(tcp_stream, async {
                // Wait for the shutdown signal
                let _ = otap_shutdown_signal.await;
            })
            .await
            .expect("Test gRPC server has failed");
    });

    // start grpc server to handle otlp requests
    let otlp_grpc_port = portpicker::pick_unused_port().expect("No free ports");
    let otlp_listening_addr: SocketAddr = format!("{grpc_addr}:{otlp_grpc_port}")
        .parse()
        .expect("failed to parse OTLP address");
    let (otlp_shutdown_sender, otlp_shutdown_signal) = tokio::sync::oneshot::channel();
    _ = tokio_rt.spawn(async move {
        let tcp_listener = TcpListener::bind(otlp_listening_addr)
            .await
            .expect("failed to bind to otlp address");
        let tcp_stream = TcpListenerStream::new(tcp_listener);
        let mock_logs_service = LogsServiceServer::new(LogsServiceMock::default());
        let mock_metrics_service = MetricsServiceServer::new(MetricsServiceMock::default());
        let mock_trace_service = TraceServiceServer::new(TraceServiceMock::default());
        Server::builder()
            .add_service(mock_logs_service)
            .add_service(mock_metrics_service)
            .add_service(mock_trace_service)
            .serve_with_incoming_shutdown(tcp_stream, async {
                // Wait for the shutdown signal
                let _ = otlp_shutdown_signal.await;
            })
            .await
            .expect("Test gRPC server has failed");
    });

    let mut group = c.benchmark_group("exporter");

    let message_len = 50;
    let row_size = 50;
    for size in [2, 4, 8, 16, 32] {
        // create data that will be used to benchmark the exporters
        let mut otap_signals = Vec::new();
        let mut otlp_signals = Vec::new();
        for _ in 0..size {
            let arrow_traces_batch_data = create_batch_arrow_record_helper(
                TRACES_BATCH_ID,
                ArrowPayloadType::Spans,
                message_len,
                row_size,
            );
            let arrow_logs_batch_data = create_batch_arrow_record_helper(
                LOGS_BATCH_ID,
                ArrowPayloadType::Logs,
                message_len,
                row_size,
            );
            let arrow_metrics_batch_data = create_batch_arrow_record_helper(
                METRICS_BATCH_ID,
                ArrowPayloadType::UnivariateMetrics,
                message_len,
                row_size,
            );

            otap_signals.push(OtapArrowBytes::ArrowTraces(arrow_traces_batch_data));
            otap_signals.push(OtapArrowBytes::ArrowLogs(arrow_logs_batch_data));
            otap_signals.push(OtapArrowBytes::ArrowMetrics(arrow_metrics_batch_data));

            let metric_message = OTLPData::Metrics(ExportMetricsServiceRequest::default());
            let log_message = OTLPData::Logs(ExportLogsServiceRequest::default());
            let trace_message = OTLPData::Traces(ExportTraceServiceRequest::default());
            otlp_signals.push(metric_message);
            otlp_signals.push(log_message);
            otlp_signals.push(trace_message);
        }

        // Benchmark the `start` function
        let _ = group.bench_with_input(
            BenchmarkId::new("perf_exporter_full_config_enabled", size),
            &otap_signals,
            |b, otap_signals| {
                b.to_async(&rt).iter(|| async {
                    // start perf exporter
                    let config = Config::new(1000, 0.3, true, true, true, true, true);
                    let exporter_config = ExporterConfig::new("perf_exporter");
                    let node_config =
                        Arc::new(NodeUserConfig::new_exporter_config(OTAP_PERF_EXPORTER_URN));
                    let mut exporter = ExporterWrapper::local(
                        PerfExporter::new(config, None),
                        node_config,
                        &exporter_config,
                    );

                    // create necessary senders and receivers to communicate with the exporter
                    let (pdata_tx, pdata_rx) = mpsc::Channel::new(100);
                    let control_sender = exporter.control_sender();
                    let pdata_sender = Sender::new_local_mpsc_sender(pdata_tx);
                    let pdata_receiver = Receiver::new_local_mpsc_receiver(pdata_rx);
                    let (node_req_tx, _node_req_rx) = pipeline_ctrl_msg_channel(10);

                    exporter
                        .set_pdata_receiver(exporter_config.name, pdata_receiver)
                        .expect("Failed to set PData receiver");
                    // start the exporter
                    let local = LocalSet::new();
                    let _run_exporter_handle = local.spawn_local(async move {
                        exporter
                            .start(node_req_tx)
                            .await
                            .expect("Exporter event loop failed");
                    });

                    // send signals to the exporter
                    for signal in otap_signals {
                        _ = pdata_sender.send(signal.clone().into()).await;
                    }

                    _ = control_sender.send(NodeControlMsg::TimerTick {}).await;
                    _ = control_sender
                        .send(NodeControlMsg::Shutdown {
                            deadline: Duration::from_millis(2000),
                            reason: "shutdown".to_string(),
                        })
                        .await;
                });
            },
        );
        let _ = group.bench_with_input(
            BenchmarkId::new("perf_exporter_full_config_disabled", size),
            &otap_signals,
            |b, otap_signals| {
                b.to_async(&rt).iter(|| async {
                    // start perf exporter
                    let config = Config::new(1000, 0.3, false, false, false, false, false);
                    let exporter_config = ExporterConfig::new("perf_exporter");
                    let node_config =
                        Arc::new(NodeUserConfig::new_exporter_config(OTAP_PERF_EXPORTER_URN));
                    let mut exporter = ExporterWrapper::local(
                        PerfExporter::new(config, None),
                        node_config,
                        &exporter_config,
                    );

                    // create necessary senders and receivers to communicate with the exporter
                    let (pdata_tx, pdata_rx) = mpsc::Channel::new(100);
                    let control_sender = exporter.control_sender();
                    let pdata_sender = Sender::new_local_mpsc_sender(pdata_tx);
                    let pdata_receiver = Receiver::new_local_mpsc_receiver(pdata_rx);
                    let (node_req_tx, _node_req_rx) = pipeline_ctrl_msg_channel(10);

                    exporter
                        .set_pdata_receiver(exporter_config.name, pdata_receiver)
                        .expect("Failed to set PData receiver");

                    // start the exporter
                    let local = LocalSet::new();
                    let _run_exporter_handle = local.spawn_local(async move {
                        exporter
                            .start(node_req_tx)
                            .await
                            .expect("Exporter event loop failed");
                    });

                    // send signals to the exporter
                    for otap_signal in otap_signals {
                        _ = pdata_sender.send(otap_signal.clone().into()).await;
                    }

                    _ = control_sender.send(NodeControlMsg::TimerTick {}).await;
                    _ = control_sender
                        .send(NodeControlMsg::Shutdown {
                            deadline: Duration::from_millis(2000),
                            reason: "shutdown".to_string(),
                        })
                        .await;
                });
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("otap_exporter", size),
            &(otap_signals, otap_grpc_port),
            |b, input| {
                b.to_async(&rt).iter(|| async {
                    // create otap exporter
                    let exporter_config = ExporterConfig::new("otap_exporter");
                    let grpc_addr = "127.0.0.1";
                    let (otap_signals, otlp_grpc_port) = input;
                    let grpc_endpoint = format!("http://{grpc_addr}:{otlp_grpc_port}");
                    let node_config =
                        Arc::new(NodeUserConfig::new_exporter_config(OTAP_EXPORTER_URN));
                    let mut exporter = ExporterWrapper::local(
                        OTAPExporter::new(grpc_endpoint, None),
                        node_config,
                        &exporter_config,
                    );

                    // create necessary senders and receivers to communicate with the exporter
                    let (pdata_tx, pdata_rx) = mpsc::Channel::new(100);
                    let control_sender = exporter.control_sender();
                    let pdata_sender = Sender::new_local_mpsc_sender(pdata_tx);
                    let pdata_receiver = Receiver::new_local_mpsc_receiver(pdata_rx);
                    let (node_req_tx, _node_req_rx) = pipeline_ctrl_msg_channel(10);

                    exporter
                        .set_pdata_receiver(exporter_config.name, pdata_receiver)
                        .expect("Failed to set PData receiver");

                    // start the exporter
                    let local = LocalSet::new();
                    let _run_exporter_handle = local.spawn_local(async move {
                        exporter
                            .start(node_req_tx)
                            .await
                            .expect("Exporter event loop failed");
                    });

                    // send signals to the exporter
                    for otap_signal in otap_signals {
                        _ = pdata_sender.send(otap_signal.clone().into()).await;
                    }

                    _ = control_sender
                        .send(NodeControlMsg::Shutdown {
                            deadline: Duration::from_millis(2000),
                            reason: "shutdown".to_string(),
                        })
                        .await;
                });
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("otlp_exporter", size),
            &(otlp_signals, otlp_grpc_port),
            |b, input| {
                b.to_async(&rt).iter(|| async {
                    // create otlp exporter
                    let exporter_config = ExporterConfig::new("otap_exporter");
                    let (otlp_signals, otlp_grpc_port) = input;
                    let grpc_addr = "127.0.0.1";
                    let grpc_endpoint = format!("http://{grpc_addr}:{otlp_grpc_port}");
                    let node_config =
                        Arc::new(NodeUserConfig::new_exporter_config(OTLP_EXPORTER_URN));
                    let mut exporter = ExporterWrapper::local(
                        OTLPExporter::new(grpc_endpoint, None),
                        node_config,
                        &exporter_config,
                    );

                    // create necessary senders and receivers to communicate with the exporter
                    let (pdata_tx, pdata_rx) = mpsc::Channel::new(100);
                    let pdata_sender = Sender::new_local_mpsc_sender(pdata_tx);
                    let pdata_receiver = Receiver::new_local_mpsc_receiver(pdata_rx);
                    let (node_req_tx, _node_req_rx) = pipeline_ctrl_msg_channel(10);

                    exporter
                        .set_pdata_receiver(exporter_config.name, pdata_receiver)
                        .expect("Failed to set PData receiver");
                    let control_sender = exporter.control_sender();

                    // start the exporter
                    let local = LocalSet::new();
                    let _run_exporter_handle = local.spawn_local(async move {
                        exporter
                            .start(node_req_tx)
                            .await
                            .expect("Exporter event loop failed");
                    });

                    // send signals to the exporter
                    for otlp_signal in otlp_signals {
                        _ = pdata_sender.send(otlp_signal.clone()).await;
                    }

                    _ = control_sender
                        .send(NodeControlMsg::Shutdown {
                            deadline: Duration::from_millis(2000),
                            reason: "shutdown".to_string(),
                        })
                        .await;
                });
            },
        );
    }

    group.finish();

    // shutdown the grpc servers
    _ = otlp_shutdown_sender.send("Shutdown");
    _ = otap_shutdown_sender.send("Shutdown");
}

criterion_group!(benches, bench_exporter);
criterion_main!(benches);
