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
use otap_df_otap::proto::opentelemetry::experimental::arrow::v1::{
    ArrowPayload, ArrowPayloadType, BatchArrowRecords,
};
use otap_df_perf::{config::Config, perf_exporter::PerfExporter};
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};
use tokio::task::LocalSet;
use tokio::time::Duration;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const MSG_COUNT: usize = 100_000;
const CHANNEL_SIZE: usize = 256;
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
    }
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
                // for _ in 0..3 {
                //     let traces_batch_data = create_batch_arrow_record_helper(
                //         TRACES_BATCH_ID,
                //         ArrowPayloadType::Spans,
                //         MESSAGE_LEN,
                //         ROW_SIZE,
                //     );
                //     let logs_batch_data = create_batch_arrow_record_helper(
                //         LOGS_BATCH_ID,
                //         ArrowPayloadType::Logs,
                //         MESSAGE_LEN,
                //         ROW_SIZE,
                //     );
                //     let metrics_batch_data = create_batch_arrow_record_helper(
                //         METRICS_BATCH_ID,
                //         ArrowPayloadType::UnivariateMetrics,
                //         MESSAGE_LEN,
                //         ROW_SIZE,
                //     );

                //     pdata_sender.send(traces_batch_data).await;
                //     pdata_sender.send(logs_batch_data).await;
                //     pdata_sender.send(metrics_batch_data).await;
                // }

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

    // group.finish();
}

criterion_group!(benches, bench_exporter);
criterion_main!(benches);
