// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end benchmarks for `make_item_batches`, the entry point used by the
//! batch processor for OTAP data, covering both splitting and merging.
//!
//! Inputs are built from scopes of 1000 items each (metrics: 100 gauges of
//! 10 data points per scope), with 10 resource, 5 scope and 3 item
//! attributes. Input sizes and limits are chosen so every output is full,
//! which makes the produced batches independent of how the implementation
//! decides where to cut.
//!
//! Scenarios (`batching/{scenario}/{signal}`):
//!
//! - `merge`: 10 inputs of 1000 items merged into one output (no limit).
//! - `mixed`: inputs of 7000, 1000, 7000, 1000 items with a limit of 2000,
//!   producing 8 outputs: large inputs are split and each tail is merged
//!   with the following small input.
//! - `mixed_transport`: `mixed` with transport-optimized inputs, as handed to
//!   the batch processor by an OTAP receiver. Logs and traces only: metrics
//!   were split before being decoded, which produced different batches.
//! - `split`: 2 inputs of 6000 items split into 24 outputs of 500.

use std::num::NonZeroU64;

use criterion::{BatchSize, BenchmarkId, Criterion, criterion_group, criterion_main};
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::batching::make_item_batches;
use otel_arrow_dfe_pdata::testing::fixtures::{
    DataGenerator, LogsConfig, MetricsConfig, TracesConfig,
};
use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;

criterion_group!(benches, bench_all);
criterion_main!(benches);

/// Items per scope. Every input is a whole number of scopes.
const SCOPE_ITEMS: usize = 1000;

struct Scenario {
    name: &'static str,
    /// Scopes per input.
    inputs: &'static [usize],
    max: Option<u64>,
    transport: bool,
    /// Expected number of outputs, checked before timing.
    outputs: usize,
}

const SCENARIOS: &[Scenario] = &[
    Scenario {
        name: "merge",
        inputs: &[1; 10],
        max: None,
        transport: false,
        outputs: 1,
    },
    Scenario {
        name: "mixed",
        inputs: &[7, 1, 7, 1],
        max: Some(2000),
        transport: false,
        outputs: 8,
    },
    Scenario {
        name: "mixed_transport",
        inputs: &[7, 1, 7, 1],
        max: Some(2000),
        transport: true,
        outputs: 8,
    },
    Scenario {
        name: "split",
        inputs: &[6, 6],
        max: Some(500),
        transport: false,
        outputs: 24,
    },
];

fn bench_all(c: &mut Criterion) {
    for s in SCENARIOS {
        let mut group = c.benchmark_group(format!("batching/{}", s.name));
        for signal in [SignalType::Logs, SignalType::Traces, SignalType::Metrics] {
            if s.transport && signal == SignalType::Metrics {
                continue;
            }
            let inputs: Vec<OtapArrowRecords> = s
                .inputs
                .iter()
                .map(|&n| input(signal, n, s.transport))
                .collect();
            bench(&mut group, s, signal, &inputs);
        }
        group.finish();
    }
}

fn bench(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    s: &Scenario,
    signal: SignalType,
    data: &[OtapArrowRecords],
) {
    let max = s.max.and_then(NonZeroU64::new);
    let run = |inputs: Vec<OtapArrowRecords>| {
        make_item_batches(signal, max, inputs)
            .expect("batching failed")
            .len()
    };
    // Every implementation must produce the same batches for the comparison
    // to be fair.
    assert_eq!(run(data.to_vec()), s.outputs, "{} {signal:?}", s.name);

    let name = match signal {
        SignalType::Logs => "logs",
        SignalType::Traces => "traces",
        SignalType::Metrics => "metrics",
    };
    let _ = group.bench_with_input(BenchmarkId::from_parameter(name), data, |b, data| {
        b.iter_batched(
            || data.to_vec(),
            |inputs| std::hint::black_box(run(inputs)),
            BatchSize::SmallInput,
        )
    });
}

/// An input of `scopes` scopes of [SCOPE_ITEMS] items under one resource.
fn input(signal: SignalType, scopes: usize, transport: bool) -> OtapArrowRecords {
    let mut rec = match signal {
        SignalType::Logs => {
            let mut datagen = DataGenerator::with_logs_config(
                LogsConfig::new(SCOPE_ITEMS)
                    .with_scopes_per_resource(scopes)
                    .with_resource_attrs(10)
                    .with_scope_attrs(5)
                    .with_log_attrs(3),
            );
            otlp_to_otap(&datagen.generate_logs_from_config().into())
        }
        SignalType::Traces => {
            let mut datagen = DataGenerator::with_traces_config(
                TracesConfig::new(SCOPE_ITEMS)
                    .with_scopes_per_resource(scopes)
                    .with_resource_attrs(10)
                    .with_scope_attrs(5)
                    .with_span_attrs(3),
            );
            otlp_to_otap(&datagen.generate_traces_from_config().into())
        }
        SignalType::Metrics => {
            let mut datagen = DataGenerator::with_metrics_config(
                MetricsConfig::new()
                    .with_gauges(vec![10; SCOPE_ITEMS / 10])
                    .with_scopes_per_resource(scopes)
                    .with_resource_attrs(10)
                    .with_scope_attrs(5)
                    .with_metric_attrs(3),
            );
            otlp_to_otap(&datagen.generate_metrics_from_config().into())
        }
    };
    assert_eq!(rec.num_items(), scopes * SCOPE_ITEMS);
    if transport {
        rec.encode_transport_optimized().expect("encode");
    }
    rec
}
