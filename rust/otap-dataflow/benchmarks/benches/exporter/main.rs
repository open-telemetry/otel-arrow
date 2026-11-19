// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Measures exporter startup and batch delivery through the last successful ACK.
//! Fixture construction and graceful shutdown are outside the measured interval.

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otel_arrow_dfe_channel::mpsc;
use otel_arrow_dfe_core_nodes::exporters::otap_exporter::OTAP_EXPORTER;
use otel_arrow_dfe_dev_nodes::exporters::perf_exporter::PERF_EXPORTER;
use otel_arrow_dfe_engine::{
    ExporterFactory, Interests,
    control::{
        Controllable, NodeControlMsg, PipelineCompletionMsg, pipeline_completion_msg_channel,
        runtime_ctrl_msg_channel,
    },
    message::Receiver,
    node::NodeWithPDataReceiver,
    testing::{exporter::create_exporter_from_factory, test_node, test_pipeline_runtime_services},
};
use otel_arrow_dfe_otap::{otap_mock::create_otap_batch_with_rows, pdata::OtapPdata};
use otel_arrow_dfe_pdata::{
    proto::opentelemetry::arrow::v1::{
        ArrowPayloadType, BatchArrowRecords, BatchStatus, StatusCode,
        arrow_logs_service_server::{ArrowLogsService, ArrowLogsServiceServer},
        arrow_metrics_service_server::{ArrowMetricsService, ArrowMetricsServiceServer},
        arrow_traces_service_server::{ArrowTracesService, ArrowTracesServiceServer},
    },
    proto::opentelemetry::metrics::v1::{Gauge, Metric, NumberDataPoint},
    testing::round_trip::to_otap_metrics,
};
use otel_arrow_dfe_telemetry::InternalTelemetrySystem;
use serde_json::{Value, json};
use std::{
    pin::Pin,
    time::{Duration, Instant},
};
use tokio::{net::TcpListener, runtime::Runtime};
use tokio_stream::{
    Stream,
    wrappers::{ReceiverStream, TcpListenerStream},
};
use tonic::{Request, Response, Status, codec::CompressionEncoding, transport::Server};

#[path = "../support/local_runtime.rs"]
mod local_runtime;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

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

fn fixtures(size: usize) -> Vec<OtapPdata> {
    const ROWS: usize = 50;
    // Metric items are data points, not metric descriptors. Give each row a point.
    let metrics = (0..ROWS)
        .map(|i| {
            Metric::build()
                .name(format!("bench.gauge.{i}"))
                .data_gauge(Gauge::new(vec![
                    NumberDataPoint::build().value_int(1i64).finish(),
                ]))
                .finish()
        })
        .collect();
    let mut batches = [
        create_otap_batch_with_rows(0, ArrowPayloadType::Spans, ROWS),
        create_otap_batch_with_rows(1, ArrowPayloadType::Logs, ROWS),
        to_otap_metrics(metrics),
    ]
    .map(|batch| {
        OtapPdata::new_default(batch.into()).test_subscribe_to(
            Interests::ACKS_OR_NACKS,
            Default::default(),
            1,
        )
    });
    for batch in &mut batches {
        assert_eq!(batch.num_items(), ROWS);
    }
    batches
        .iter()
        .cycle()
        .take(size * batches.len())
        .cloned()
        .collect()
}

async fn measure_export(
    factory: &ExporterFactory<OtapPdata>,
    config: &Value,
    signals: &[OtapPdata],
) -> Duration {
    let mut exporter =
        create_exporter_from_factory(factory, config.clone()).expect("valid exporter config");
    let (tx, rx) = mpsc::Channel::new(100);
    exporter
        .set_pdata_receiver(test_node("exporter"), Receiver::new_local_mpsc_receiver(rx))
        .expect("exporter accepts the data receiver");
    let ctrl = exporter.control_sender();
    let (runtime_tx, _runtime_rx) = runtime_ctrl_msg_channel(10);
    let (completion_tx, mut completions) = pipeline_completion_msg_channel(signals.len());
    let telemetry = InternalTelemetrySystem::default();
    let reporter = telemetry.reporter();
    let services = test_pipeline_runtime_services();
    let start = Instant::now();
    let task = tokio::task::spawn_local(exporter.start(
        runtime_tx,
        completion_tx,
        reporter,
        Interests::empty(),
        services,
    ));
    tokio::time::timeout(Duration::from_secs(30), async {
        for signal in signals {
            tx.send_async(signal.clone())
                .await
                .expect("exporter input is open");
        }
        for _ in signals {
            match completions
                .recv()
                .await
                .expect("exporter exited before all ACKs")
            {
                PipelineCompletionMsg::DeliverAck { .. } => {}
                PipelineCompletionMsg::DeliverNack { nack } => {
                    panic!("exporter NACK: {}", nack.reason)
                }
            }
        }
    })
    .await
    .expect("exporter did not acknowledge every batch");
    let elapsed = start.elapsed();

    ctrl.send(NodeControlMsg::Shutdown {
        deadline: Instant::now() + Duration::from_secs(2),
        reason: "benchmark complete".into(),
    })
    .await
    .expect("exporter control is open");
    drop(tx);
    let _terminal = task
        .await
        .expect("exporter task should join")
        .expect("clean exporter shutdown");
    elapsed
}

fn bench_exporter(c: &mut Criterion) {
    let rt = local_runtime::build_local_runtime("exporter-bench");
    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    let _ = core_affinity::set_for_current(*cores.last().expect("no cores found"));

    let server_rt = Runtime::new().expect("mock server runtime");
    let listener = server_rt
        .block_on(TcpListener::bind("127.0.0.1:0"))
        .expect("bind mock server");
    let addr = listener.local_addr().expect("bound server address");
    let (stop, stopped) = tokio::sync::oneshot::channel();
    let server = server_rt.spawn(async move {
        Server::builder()
            .add_service(
                ArrowLogsServiceServer::new(ArrowLogsServiceMock::default())
                    .accept_compressed(CompressionEncoding::Zstd),
            )
            .add_service(
                ArrowTracesServiceServer::new(ArrowTracesServiceMock::default())
                    .accept_compressed(CompressionEncoding::Zstd),
            )
            .add_service(
                ArrowMetricsServiceServer::new(ArrowMetricsServiceMock::default())
                    .accept_compressed(CompressionEncoding::Zstd),
            )
            .serve_with_incoming_shutdown(TcpListenerStream::new(listener), async {
                let _ = stopped.await;
            })
            .await
    });

    let cases = [
        (
            "perf_exporter_full_config_enabled",
            &PERF_EXPORTER,
            json!({}),
        ),
        (
            "perf_exporter_full_config_disabled",
            &PERF_EXPORTER,
            json!({
                "self_usage": false, "cpu_usage": false, "mem_usage": false, "disk_usage": false, "io_usage": false,
            }),
        ),
        (
            "otap_exporter",
            &OTAP_EXPORTER,
            json!({"grpc_endpoint": format!("http://{addr}")}),
        ),
    ];
    let mut group = c.benchmark_group("exporter");
    for size in [2, 4, 8, 16, 32] {
        let signals = fixtures(size);
        for (name, factory, config) in &cases {
            let _ = group.bench_function(BenchmarkId::new(*name, size), |b| {
                b.iter_custom(|iters| {
                    rt.block_on(async {
                        let mut total = Duration::ZERO;
                        for _ in 0..iters {
                            total += measure_export(factory, config, &signals).await;
                        }
                        total
                    })
                });
            });
        }
    }
    group.finish();
    stop.send(()).expect("mock server still running");
    server_rt
        .block_on(server)
        .expect("mock server should join")
        .expect("clean mock server shutdown");
}

criterion_group!(benches, bench_exporter);
criterion_main!(benches);
