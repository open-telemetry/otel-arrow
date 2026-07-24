// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Throughput benchmark for the internal metric-set to OTLP bridge.
//!
//! Each timed iteration covers the computational path used by the internal
//! telemetry receiver:
//!
//! 1. update and snapshot hot metric sets;
//! 2. enqueue and collect the snapshots;
//! 3. aggregate them in, then drain them from, the metric registry; and
//! 4. project the drained metric sets into serialized OTLP metrics.
//!
//! Registration, periodic timer waiting, downstream pipeline delivery,
//! exporters, and network I/O are intentionally excluded. The collector is
//! drained synchronously to measure bridge capacity without scheduler noise.
//! Criterion throughput is expressed as emitted univariate OTLP metrics.

use std::collections::HashMap;
use std::hint::black_box;

use criterion::{Criterion, Throughput, criterion_group, criterion_main};
use otap_df_config::observed_state::SendPolicy;
use otap_df_config::pipeline::telemetry::AttributeValue as ConfigAttributeValue;
use otap_df_config::pipeline::telemetry::TelemetryConfig;
use otap_df_telemetry::instrument::{Counter, Gauge, Mmsc, ObserveCounter, ObserveUpDownCounter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::metrics::otlp::{
    MetricView, MetricViewSelector, MetricViewStream, MetricsOtlpEncoder,
};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::{InternalTelemetrySystem, LogContext};
use otap_df_telemetry_macros::{attribute_set, metric_set};

#[cfg(not(windows))]
use tikv_jemallocator::Jemalloc;

#[cfg(not(windows))]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

const ENTITY_COUNTS: [usize; 2] = [100, 1_000];
const METRICS_PER_SET: u64 = 8;
const SCOPE_NAME: &str = "benchmark.internal_metrics_bridge";
const WORKLOAD_KIND: &str = "representative";

#[attribute_set(scope, name = "benchmark.internal_metrics_bridge.attrs")]
#[derive(Debug, Clone, Hash)]
struct BenchmarkAttributes {
    /// Unique entity identifier.
    #[attribute_key = "benchmark.entity.id"]
    entity_id: u64,

    /// Stable workload discriminator used by scope-attribute views.
    #[attribute_key = "benchmark.workload.kind"]
    workload_kind: String,
}

#[metric_set(name = "benchmark.internal_metrics_bridge")]
#[derive(Debug, Default)]
struct BenchmarkMetrics {
    /// Delta request count.
    #[metric(unit = "{request}")]
    requests: Counter<u64>,

    /// Delta error count.
    #[metric(unit = "{error}")]
    errors: Counter<u64>,

    /// Current cumulative byte count.
    #[metric(unit = "By")]
    bytes: ObserveCounter<u64>,

    /// Current queue depth.
    #[metric(unit = "{item}")]
    queue_depth: Gauge<u64>,

    /// Current CPU utilization ratio.
    #[metric(unit = "{1}")]
    cpu_utilization: Gauge<f64>,

    /// Current memory usage.
    #[metric(unit = "By")]
    memory_usage: ObserveUpDownCounter<u64>,

    /// Batch-size distribution summary.
    #[metric(unit = "{item}")]
    batch_size: Mmsc,

    /// Compute-duration distribution summary.
    #[metric(unit = "ns")]
    compute_duration: Mmsc,
}

impl BenchmarkMetrics {
    fn populate(&mut self) {
        self.requests.add(128);
        self.errors.add(1);
        self.bytes.observe(64 * 1024);
        self.queue_depth.set(16);
        self.cpu_utilization.set(0.75);
        self.memory_usage.observe(8 * 1024 * 1024);
        self.batch_size.record(128.0);
        self.compute_duration.record(42_000.0);
    }
}

struct BridgeBenchmark {
    _telemetry: InternalTelemetrySystem,
    registry: TelemetryRegistryHandle,
    collector: std::sync::Arc<otap_df_telemetry::collector::InternalCollector>,
    reporter: otap_df_telemetry::reporter::MetricsReporter,
    metric_sets: Vec<MetricSet<BenchmarkMetrics>>,
    encoder: MetricsOtlpEncoder,
}

impl BridgeBenchmark {
    fn new(entity_count: usize, views: Vec<MetricView>) -> Self {
        let registry = TelemetryRegistryHandle::new();
        let config = TelemetryConfig {
            reporting_channel_size: entity_count + 1,
            ..Default::default()
        };
        let telemetry = InternalTelemetrySystem::new(
            &config,
            config.reporting_interval,
            registry.clone(),
            None,
            SendPolicy::default(),
            LogContext::new,
            None,
        )
        .expect("benchmark telemetry system should initialize");
        let resource_bytes = telemetry.internal_telemetry_settings().resource_field_bytes;
        let encoder = MetricsOtlpEncoder::new_with_views(&resource_bytes, views)
            .expect("resource bytes should be valid");
        let collector = telemetry.collector();
        let reporter = telemetry.reporter();
        let metric_sets = (0..entity_count)
            .map(|entity_id| {
                registry.register_metric_set::<BenchmarkMetrics>(BenchmarkAttributes {
                    entity_id: entity_id as u64,
                    workload_kind: WORKLOAD_KIND.to_owned(),
                })
            })
            .collect();

        Self {
            _telemetry: telemetry,
            registry,
            collector,
            reporter,
            metric_sets,
            encoder,
        }
    }

    fn run_once(&mut self) -> usize {
        for metric_set in &mut self.metric_sets {
            metric_set.populate();
            self.reporter
                .report(metric_set)
                .expect("benchmark snapshot should be accepted");
        }
        self.collector.collect_pending();
        let batch = self.registry.drain_metric_export_batch();
        assert_eq!(batch.metric_sets.len(), self.metric_sets.len());
        self.encoder
            .encode(&batch)
            .expect("benchmark batch should encode")
            .expect("benchmark batch should not be empty")
            .as_bytes()
            .len()
    }
}

fn views() -> Vec<MetricView> {
    [
        ("requests", "bridge_requests", false),
        ("errors", "bridge_errors", false),
        ("bytes", "bridge_bytes", false),
        ("queue.depth", "bridge_queue_depth", true),
        ("cpu.utilization", "bridge_cpu_utilization", true),
        ("compute.duration", "bridge_compute_duration", true),
    ]
    .into_iter()
    .map(
        |(instrument_name, stream_name, match_scope_attribute)| MetricView {
            selector: MetricViewSelector {
                scope_name: Some(SCOPE_NAME.to_owned()),
                scope_attributes: if match_scope_attribute {
                    HashMap::from([(
                        "benchmark.workload.kind".to_owned(),
                        ConfigAttributeValue::String(WORKLOAD_KIND.to_owned()),
                    )])
                } else {
                    HashMap::new()
                },
                instrument_name: Some(instrument_name.to_owned()),
            },
            stream: MetricViewStream {
                name: Some(stream_name.to_owned()),
                description: None,
            },
        },
    )
    .collect()
}

fn bench_internal_metrics_bridge(c: &mut Criterion) {
    for entity_count in ENTITY_COUNTS {
        let mut group =
            c.benchmark_group(format!("internal_metrics_bridge/{entity_count}_entities"));
        _ = group.throughput(Throughput::Elements(entity_count as u64 * METRICS_PER_SET));

        let mut without_views = BridgeBenchmark::new(entity_count, Vec::new());
        _ = group.bench_function("no_views", |b| {
            b.iter(|| black_box(without_views.run_once()));
        });

        let mut with_views = BridgeBenchmark::new(entity_count, views());
        _ = group.bench_function("six_views", |b| {
            b.iter(|| black_box(with_views.run_once()));
        });

        group.finish();
    }
}

criterion_group!(benches, bench_internal_metrics_bridge);
criterion_main!(benches);
