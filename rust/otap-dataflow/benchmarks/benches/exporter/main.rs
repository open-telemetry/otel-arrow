// SPDX-License-Identifier: Apache-2.0

//! This benchmark compares the performance of different perf exporter configurations

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fluke_hpack::Encoder;
use mimalloc::MiMalloc;
use otap_df_channel::mpsc;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::Exporter;
use otap_df_engine::message::{ControlMsg, Receiver, Sender};
use otap_df_otap::grpc::OTAPData;
use otap_df_otap::mock::{ArrowLogsServiceMock, ArrowMetricsServiceMock, ArrowTracesServiceMock};
use otap_df_otap::otap_exporter::OTAPExporter;
use otap_df_otap::proto::opentelemetry::experimental::arrow::v1::{
    ArrowPayload, ArrowPayloadType, BatchArrowRecords,
    arrow_logs_service_server::ArrowLogsServiceServer,
    arrow_metrics_service_server::ArrowMetricsServiceServer,
    arrow_traces_service_server::ArrowTracesServiceServer,
};
use otap_df_otlp::grpc::OTLPData;
use otap_df_otlp::mock::{
    LogsServiceMock, MetricsServiceMock, ProfilesServiceMock, TraceServiceMock,
};
use otap_df_otlp::otlp_exporter::OTLPExporter;
use otap_df_otlp::proto::opentelemetry::collector::{
    logs::v1::{ExportLogsServiceRequest, logs_service_server::LogsServiceServer},
    metrics::v1::{ExportMetricsServiceRequest, metrics_service_server::MetricsServiceServer},
    profiles::v1development::{
        ExportProfilesServiceRequest, profiles_service_server::ProfilesServiceServer,
    },
    trace::v1::{ExportTraceServiceRequest, trace_service_server::TraceServiceServer},
};
use otap_df_perf::{config::Config, perf_exporter::PerfExporter};
use std::net::SocketAddr;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::net::TcpListener;
use tokio::runtime::Runtime;
use tokio::task::LocalSet;
use tokio::time::Duration;
use tonic::codegen::tokio_stream::wrappers::TcpListenerStream;
use tonic::transport::Server;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const TRACES_BATCH_ID: i64 = 0;
const LOGS_BATCH_ID: i64 = 1;
const METRICS_BATCH_ID: i64 = 2;
const ROW_SIZE: usize = 10;
const MESSAGE_LEN: usize = 5;

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
    let timestamp = SystemTime::now().duration_since(UNIX_EPOCH).unwrap();

    let secs = timestamp.as_secs().to_string();
    let nanos = timestamp.subsec_nanos().to_string();

    // string formatted with timestamp secs : timestamp subsec_nanos
    let timestamp_string = format!("{}:{}", secs, nanos);
    let timestamp_bytes = timestamp_string.as_bytes();

    // convert time to
    let headers = vec![(b"timestamp" as &[u8], timestamp_bytes)];
    let mut encoder = Encoder::new();
    let encoded_headers = encoder.encode(headers);
    BatchArrowRecords {
        batch_id: batch_id,
        arrow_payloads: arrow_payloads,
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

    let mut batches = Vec::new();
    let mut otap_signals = Vec::new();
    let mut otlp_signals = Vec::new();
    // let mut group = c.benchmark_group("exporter");
    // _ = group.throughput(Throughput::Elements(MSG_COUNT as u64));

    for _ in 0..3 {
        let traces_batch_data = create_batch_arrow_record_helper(
            TRACES_BATCH_ID,
            ArrowPayloadType::Spans,
            MESSAGE_LEN,
            ROW_SIZE,
        );
        let logs_batch_data = create_batch_arrow_record_helper(
            LOGS_BATCH_ID,
            ArrowPayloadType::Logs,
            MESSAGE_LEN,
            ROW_SIZE,
        );
        let metrics_batch_data = create_batch_arrow_record_helper(
            METRICS_BATCH_ID,
            ArrowPayloadType::UnivariateMetrics,
            MESSAGE_LEN,
            ROW_SIZE,
        );

        batches.push(traces_batch_data);
        batches.push(logs_batch_data);
        batches.push(metrics_batch_data);

        let arrow_traces_batch_data = create_batch_arrow_record_helper(
            TRACES_BATCH_ID,
            ArrowPayloadType::Spans,
            MESSAGE_LEN,
            ROW_SIZE,
        );
        let arrow_logs_batch_data = create_batch_arrow_record_helper(
            LOGS_BATCH_ID,
            ArrowPayloadType::Logs,
            MESSAGE_LEN,
            ROW_SIZE,
        );
        let arrow_metrics_batch_data = create_batch_arrow_record_helper(
            METRICS_BATCH_ID,
            ArrowPayloadType::UnivariateMetrics,
            MESSAGE_LEN,
            ROW_SIZE,
        );

        otap_signals.push(OTAPData::ArrowTraces(arrow_traces_batch_data));
        otap_signals.push(OTAPData::ArrowLogs(arrow_logs_batch_data));
        otap_signals.push(OTAPData::ArrowMetrics(arrow_metrics_batch_data));

        let metric_message = OTLPData::Metrics(ExportMetricsServiceRequest::default());
        let log_message = OTLPData::Logs(ExportLogsServiceRequest::default());
        let trace_message = OTLPData::Traces(ExportTraceServiceRequest::default());
        otlp_signals.push(metric_message);
        otlp_signals.push(log_message);
        otlp_signals.push(trace_message);
    }

    // run server out here
    let grpc_addr = "127.0.0.1";
    let otap_grpc_port = portpicker::pick_unused_port().expect("No free ports");
    let otlp_grpc_port = portpicker::pick_unused_port().expect("No free ports");

    let otap_listening_addr: SocketAddr = format!("{grpc_addr}:{otap_grpc_port}").parse().unwrap();

    let (otap_shutdown_sender, otap_shutdown_signal) = tokio::sync::oneshot::channel();
    let (otap_sender, otap_receiver) = tokio::sync::mpsc::channel(32);

    let tokio_rt = Runtime::new().unwrap();
    _ = tokio_rt.spawn(async move {
        let tcp_listener = TcpListener::bind(otap_listening_addr).await.unwrap();
        let tcp_stream = TcpListenerStream::new(tcp_listener);
        let mock_logs_service =
            ArrowLogsServiceServer::new(ArrowLogsServiceMock::new(otap_sender.clone()));
        let mock_metrics_service =
            ArrowMetricsServiceServer::new(ArrowMetricsServiceMock::new(otap_sender.clone()));
        let mock_trace_service =
            ArrowTracesServiceServer::new(ArrowTracesServiceMock::new(otap_sender.clone()));
        _ = Server::builder()
            .add_service(mock_logs_service)
            .add_service(mock_metrics_service)
            .add_service(mock_trace_service)
            .serve_with_incoming_shutdown(tcp_stream, async {
                // Wait for the shutdown signal
                drop(otap_shutdown_signal.await.ok())
            })
            .await
            .expect("Test gRPC server has failed");
    });
    let otlp_grpc_port = portpicker::pick_unused_port().expect("No free ports");
    let otlp_listening_addr: SocketAddr = format!("{grpc_addr}:{otlp_grpc_port}").parse().unwrap();
    let (otlp_shutdown_sender, otlp_shutdown_signal) = tokio::sync::oneshot::channel();
    let (otlp_sender, otlp_receiver) = tokio::sync::mpsc::channel(32);
    _ = tokio_rt.spawn(async move {
        let tcp_listener = TcpListener::bind(otlp_listening_addr).await.unwrap();
        let tcp_stream = TcpListenerStream::new(tcp_listener);
        let mock_logs_service = LogsServiceServer::new(LogsServiceMock::new(otlp_sender.clone()));
        let mock_metrics_service =
            MetricsServiceServer::new(MetricsServiceMock::new(otlp_sender.clone()));
        let mock_trace_service =
            TraceServiceServer::new(TraceServiceMock::new(otlp_sender.clone()));
        _ = Server::builder()
            .add_service(mock_logs_service)
            .add_service(mock_metrics_service)
            .add_service(mock_trace_service)
            .serve_with_incoming_shutdown(tcp_stream, async {
                // Wait for the shutdown signal
                drop(otlp_shutdown_signal.await.ok())
            })
            .await
            .expect("Test gRPC server has failed");
    });

    // Benchmark the `start` function
    let _ = c.bench_with_input(
        BenchmarkId::new("perf_exporter", "batch"),
        &batches,
        |b, batches| {
            b.to_async(&rt).iter(|| async {
                // start perf exporter
                let config = Config::new(1000, 0.3, true, true, true, true, true);
                let exporter_config = ExporterConfig::new("perf_exporter");
                let exporter =
                    ExporterWrapper::local(PerfExporter::new(config, None), &exporter_config);
                // in the background send signals

                let (control_tx, control_rx) = mpsc::Channel::new(100);
                let (pdata_tx, pdata_rx) = mpsc::Channel::new(100);
                let control_sender = Sender::Local(control_tx);
                let control_receiver = Receiver::Local(control_rx);
                let pdata_sender = Sender::Local(pdata_tx);
                let pdata_receiver = Receiver::Local(pdata_rx);

                let local = LocalSet::new();
                let run_exporter_handle = local.spawn_local(async move {
                    exporter
                        .start(control_receiver, pdata_receiver)
                        .await
                        .expect("Exporter event loop failed");
                });

                for batch in batches {
                    pdata_sender.send(batch.clone()).await;
                }

                control_sender.send(ControlMsg::TimerTick {}).await;
                control_sender
                    .send(ControlMsg::Shutdown {
                        deadline: Duration::from_millis(2000),
                        reason: "shutdown".to_string(),
                    })
                    .await;
            });
        },
    );

    let _ = c.bench_with_input(
        BenchmarkId::new("otap_exporter", "otap_signals"),
        &(otap_signals, otap_grpc_port),
        |b, input| {
            b.to_async(&rt).iter(|| async {
                // start perf exporter
                let exporter_config = ExporterConfig::new("otap_exporter");
                let grpc_addr = "127.0.0.1";
                let (otap_signals, otlp_grpc_port) = input;
                let grpc_endpoint = format!("http://{grpc_addr}:{otlp_grpc_port}");
                let exporter = ExporterWrapper::local(
                    OTAPExporter::new(grpc_endpoint, None),
                    &exporter_config,
                );
                // in the background send signals

                let (control_tx, control_rx) = mpsc::Channel::new(100);
                let (pdata_tx, pdata_rx) = mpsc::Channel::new(100);
                let control_sender = Sender::Local(control_tx);
                let control_receiver = Receiver::Local(control_rx);
                let pdata_sender = Sender::Local(pdata_tx);
                let pdata_receiver = Receiver::Local(pdata_rx);

                let local = LocalSet::new();
                let run_exporter_handle = local.spawn_local(async move {
                    exporter
                        .start(control_receiver, pdata_receiver)
                        .await
                        .expect("Exporter event loop failed");
                });

                for otap_signal in otap_signals {
                    pdata_sender.send(otap_signal.clone()).await;
                }

                control_sender
                    .send(ControlMsg::Shutdown {
                        deadline: Duration::from_millis(2000),
                        reason: "shutdown".to_string(),
                    })
                    .await;
            });
        },
    );

    let _ = c.bench_with_input(
        BenchmarkId::new("otlp_exporter", "otlp_signals"),
        &(otlp_signals, otlp_grpc_port),
        |b, input| {
            b.to_async(&rt).iter(|| async {
                // start perf exporter
                let exporter_config = ExporterConfig::new("otap_exporter");
                let (otlp_signals, otlp_grpc_port) = input;
                let grpc_addr = "127.0.0.1";
                let grpc_endpoint = format!("http://{grpc_addr}:{otlp_grpc_port}");
                let exporter = ExporterWrapper::local(
                    OTLPExporter::new(grpc_endpoint, None),
                    &exporter_config,
                );
                // in the background send signals

                let (control_tx, control_rx) = mpsc::Channel::new(100);
                let (pdata_tx, pdata_rx) = mpsc::Channel::new(100);
                let control_sender = Sender::Local(control_tx);
                let control_receiver = Receiver::Local(control_rx);
                let pdata_sender = Sender::Local(pdata_tx);
                let pdata_receiver = Receiver::Local(pdata_rx);

                let local = LocalSet::new();
                let run_exporter_handle = local.spawn_local(async move {
                    exporter
                        .start(control_receiver, pdata_receiver)
                        .await
                        .expect("Exporter event loop failed");
                });

                for otlp_signal in otlp_signals {
                    pdata_sender.send(otlp_signal.clone()).await;
                }

                control_sender
                    .send(ControlMsg::Shutdown {
                        deadline: Duration::from_millis(2000),
                        reason: "shutdown".to_string(),
                    })
                    .await;
            });
        },
    );

    // group.finish();

    _ = otlp_shutdown_sender.send("Shutdown");
    _ = otap_shutdown_sender.send("Shutdown");
}

criterion_group!(benches, bench_exporter);
criterion_main!(benches);
