// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for the OTAP batch processor fed by many small requests.
//!
//! Each iteration sends `TOTAL_ITEMS / items_per_request` independently
//! encoded OTAP requests through a fresh processor configured like the nightly
//! batch processor suite, then flushes any remainder via the scheduled wakeup.
//! Small requests stress per-input and per-column overheads (sizing,
//! dictionary bookkeeping in concatenate) rather than per-row work.
//!
//! Scenarios: `batch_processor/{signal}/{items_per_request}`.

#![allow(missing_docs)]

use std::hint::black_box;
use std::sync::Arc;

use criterion::measurement::WallTime;
use criterion::{
    BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main,
};
use otel_arrow_dfe_channel::mpsc;
use otel_arrow_dfe_config::node::NodeUserConfig;
use otel_arrow_dfe_engine::Interests;
use otel_arrow_dfe_engine::context::ControllerContext;
use otel_arrow_dfe_engine::control::{
    NodeControlMsg, RuntimeCtrlMsgReceiver, runtime_ctrl_msg_channel,
};
use otel_arrow_dfe_engine::local::processor as local;
use otel_arrow_dfe_engine::message::{Message, Receiver, Sender};
use otel_arrow_dfe_engine::node::NodeWithPDataReceiver;
use otel_arrow_dfe_engine::node::NodeWithPDataSender;
use otel_arrow_dfe_engine::processor::{ProcessorWrapper, ProcessorWrapperRuntime};
use otel_arrow_dfe_engine::testing::node::test_node;
use otel_arrow_dfe_otap::pdata::OtapPdata;
use otel_arrow_dfe_pdata::proto::OtlpProtoMessage;
use otel_arrow_dfe_pdata::testing::fixtures::{
    DataGenerator, LogsConfig, MetricsConfig, TracesConfig,
};
use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;
use otel_arrow_dfe_pdata_codec::OtapPayload;
use otel_arrow_dfe_telemetry::InternalTelemetrySystem;
use serde_json::json;

use otel_arrow_dfe_core_nodes::processors::batch_processor::{
    OTAP_BATCH_PROCESSOR_FACTORY, OTAP_BATCH_PROCESSOR_URN,
};

/// Items sent per iteration. Equal to `MIN_SIZE`, so each iteration forms one
/// size-triggered batch.
const TOTAL_ITEMS: usize = 8192;

/// Batch processor `min_size`, matching the nightly batch processor suite.
const MIN_SIZE: usize = 8192;

/// Items per request. 8 matches the low-throughput small-request scenario;
/// the larger sizes are for contrast at the same total item count.
const ITEMS_PER_REQUEST: &[usize] = &[8, 64, 512];

/// Attribute counts per resource, scope and item.
const RESOURCE_ATTRS: usize = 10;
const SCOPE_ATTRS: usize = 5;
const ITEM_ATTRS: usize = 3;

/// Output channel capacity. One iteration emits at most a handful of batches.
const OUTPUT_CHANNEL_CAPACITY: usize = 64;

criterion_group!(benches, bench_batch_processor);
criterion_main!(benches);

fn bench_batch_processor(c: &mut Criterion) {
    // Single-threaded tokio runtime used for all benchmark iterations.
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let mut group = c.benchmark_group("batch_processor");
    let _ = group.throughput(Throughput::Elements(TOTAL_ITEMS as u64));

    for &items_per_request in ITEMS_PER_REQUEST {
        debug_assert_eq!(TOTAL_ITEMS % items_per_request, 0);
        let num_requests = TOTAL_ITEMS / items_per_request;

        // Generate data once, outside the benchmark loop.
        let logs = generate_logs(num_requests, items_per_request);
        let traces = generate_traces(num_requests, items_per_request);
        let metrics = generate_metrics(num_requests, items_per_request);

        bench_scenario(&mut group, &rt, "logs", items_per_request, &logs);
        bench_scenario(&mut group, &rt, "traces", items_per_request, &traces);
        bench_scenario(&mut group, &rt, "metrics", items_per_request, &metrics);
    }

    group.finish();
}

/// Run a single named benchmark scenario within the group.
fn bench_scenario(
    group: &mut BenchmarkGroup<'_, WallTime>,
    rt: &tokio::runtime::Runtime,
    signal: &str,
    items_per_request: usize,
    messages: &[OtapPdata],
) {
    let _ = group.bench_with_input(
        BenchmarkId::new(signal, items_per_request),
        messages,
        |b, messages| {
            b.iter_batched(
                || (messages.to_vec(), create_processor()),
                |(msgs, mut state)| {
                    rt.block_on(run_scenario(msgs, &mut state));
                },
                BatchSize::LargeInput,
            );
        },
    );
}

/// All state needed to drive the processor for a single benchmark iteration.
///
/// The `_ctrl_rx` field is never read from, but must be kept alive so the
/// runtime control channel remains open.
struct ProcessorState {
    processor: Box<dyn local::Processor<OtapPdata>>,
    effect_handler: local::EffectHandler<OtapPdata>,
    output_receiver: Receiver<OtapPdata>,
    _ctrl_rx: RuntimeCtrlMsgReceiver<OtapPdata>,
}

/// Create a fresh processor instance with all wiring needed for direct
/// `process()` calls.
fn create_processor() -> ProcessorState {
    let telemetry = InternalTelemetrySystem::default();
    let registry = telemetry.registry();
    let reporter = telemetry.reporter();

    let controller_ctx = ControllerContext::new(registry);
    let pipeline_ctx =
        controller_ctx.pipeline_context_with("bench_grp".into(), "bench_pipe".into(), 0, 1, 0);

    // Same processor config as the nightly batch processor suite.
    let mut node_config = NodeUserConfig::new_processor_config(OTAP_BATCH_PROCESSOR_URN);
    node_config.config = json!({
        "otap": { "min_size": MIN_SIZE, "sizer": "items" },
        "max_batch_duration": "200ms",
    });

    let config = otel_arrow_dfe_engine::config::ProcessorConfig::new("batch_bench");

    let mut wrapper: ProcessorWrapper<OtapPdata> = (OTAP_BATCH_PROCESSOR_FACTORY.create)(
        pipeline_ctx,
        test_node("batch_bench"),
        Arc::new(node_config),
        &config,
        &otel_arrow_dfe_engine::capability::registry::Capabilities::empty(),
    )
    .expect("failed to create processor");

    let (out_tx, out_rx) = mpsc::Channel::new(OUTPUT_CHANNEL_CAPACITY);
    let out_sender = Sender::new_local_mpsc_sender(out_tx);
    let _ = wrapper.set_pdata_sender(test_node("batch_bench"), "default".into(), out_sender);

    let (_, dummy_rx) = mpsc::Channel::<OtapPdata>::new(1);
    let dummy_receiver = Receiver::new_local_mpsc_receiver(dummy_rx);
    let _ = wrapper.set_pdata_receiver(test_node("batch_bench"), dummy_receiver);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build setup runtime");

    let runtime = rt
        .block_on(wrapper.prepare_runtime(
            reporter,
            Interests::empty(),
            otel_arrow_dfe_engine::testing::test_pipeline_runtime_services(),
        ))
        .expect("failed to prepare runtime");

    match runtime {
        ProcessorWrapperRuntime::Local {
            mut effect_handler,
            processor,
            ..
        } => {
            let (ctrl_tx, ctrl_rx) = runtime_ctrl_msg_channel(10);
            effect_handler.set_runtime_ctrl_msg_sender(ctrl_tx);

            let output_receiver = Receiver::new_local_mpsc_receiver(out_rx);
            ProcessorState {
                processor,
                effect_handler,
                output_receiver,
                _ctrl_rx: ctrl_rx,
            }
        }
        ProcessorWrapperRuntime::Shared { .. } => {
            unreachable!("batch processor is always local")
        }
    }
}

/// Run one complete benchmark iteration: process all messages then flush.
async fn run_scenario(messages: Vec<OtapPdata>, state: &mut ProcessorState) {
    for msg in messages {
        state
            .processor
            .process(Message::PData(msg), &mut state.effect_handler)
            .await
            .expect("process failed");
    }

    // Flush any remainder via the scheduled wakeup.
    if let Some((slot, when, revision)) = state.effect_handler.pop_wakeup() {
        state
            .processor
            .process(
                Message::Control(NodeControlMsg::Wakeup {
                    slot,
                    when,
                    revision,
                }),
                &mut state.effect_handler,
            )
            .await
            .expect("wakeup flush failed");
    }

    let mut output = Vec::with_capacity(OUTPUT_CHANNEL_CAPACITY);
    while let Ok(pdata) = state.output_receiver.try_recv() {
        output.push(pdata);
    }
    debug_assert!(!output.is_empty(), "batch processor emitted nothing");
    let _ = black_box(output);
}

/// Each request is converted to OTAP on its own, so like independently
/// received requests their arrays (including dictionary values) are distinct
/// allocations.
fn to_pdata(msg: OtlpProtoMessage) -> OtapPdata {
    OtapPdata::new_default(OtapPayload::from(otlp_to_otap(&msg)))
}

fn generate_logs(num_requests: usize, items_per_request: usize) -> Vec<OtapPdata> {
    let mut datagen = DataGenerator::with_logs_config(
        LogsConfig::new(items_per_request)
            .with_resource_attrs(RESOURCE_ATTRS)
            .with_scope_attrs(SCOPE_ATTRS)
            .with_log_attrs(ITEM_ATTRS),
    );
    (0..num_requests)
        .map(|_| to_pdata(datagen.generate_logs_from_config().into()))
        .collect()
}

fn generate_traces(num_requests: usize, items_per_request: usize) -> Vec<OtapPdata> {
    let mut datagen = DataGenerator::with_traces_config(
        TracesConfig::new(items_per_request)
            .with_resource_attrs(RESOURCE_ATTRS)
            .with_scope_attrs(SCOPE_ATTRS)
            .with_span_attrs(ITEM_ATTRS),
    );
    (0..num_requests)
        .map(|_| to_pdata(datagen.generate_traces_from_config().into()))
        .collect()
}

fn generate_metrics(num_requests: usize, items_per_request: usize) -> Vec<OtapPdata> {
    let mut datagen = DataGenerator::with_metrics_config(
        MetricsConfig::new()
            .with_gauges(vec![items_per_request])
            .with_resource_attrs(RESOURCE_ATTRS)
            .with_scope_attrs(SCOPE_ATTRS)
            .with_metric_attrs(ITEM_ATTRS),
    );
    (0..num_requests)
        .map(|_| to_pdata(datagen.generate_metrics_from_config().into()))
        .collect()
}
