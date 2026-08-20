// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::print_stdout)] // The benchmark result is its stdout contract.

//! Repeatable latency and allocation baseline for config snapshot redaction.

use otap_df_config::node::{NodeUserConfig, REDACTED_HEADER_VALUE};
use otap_df_config::redaction::{
    CONFIG_REDACTORS, ConfigRedactor, RedactedString, RedactionError, SecretField,
    redact_typed_config_in_place,
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::hint::black_box;
use std::time::{Duration, Instant};

const WARMUP_ITERATIONS: usize = 200;
const MEASURED_ITERATIONS: usize = 1_000;
const TYPED_BENCHMARK_URN: &str = "urn:otel:exporter:benchmark-typed";

#[global_allocator]
static ALLOCATOR: dhat::Alloc = dhat::Alloc;

#[derive(Deserialize, Serialize)]
struct TypedBenchmarkConfig {
    password: RedactedString,
    #[serde(flatten)]
    remaining: BTreeMap<String, Value>,
}

fn redact_typed_benchmark_config(config: &mut Value) -> Result<(), RedactionError> {
    redact_typed_config_in_place::<TypedBenchmarkConfig>(
        config,
        &[SecretField::required("password")],
    )
}

#[allow(unsafe_code)]
#[linkme::distributed_slice(CONFIG_REDACTORS)]
static TYPED_BENCHMARK_REDACTOR: ConfigRedactor =
    ConfigRedactor::new(TYPED_BENCHMARK_URN, redact_typed_benchmark_config);

#[derive(Serialize)]
struct WorkloadResult {
    workload: &'static str,
    warmup_iterations: usize,
    measured_iterations: usize,
    input_bytes: usize,
    expected_hash: u64,
    latency_ns: LatencyStats,
    allocation: AllocationStats,
}

#[derive(Serialize)]
struct LatencyStats {
    p50: u128,
    p95: u128,
    p99: u128,
    mean: f64,
    stdev: f64,
}

#[derive(Serialize)]
struct AllocationStats {
    total_bytes: u64,
    total_blocks: u64,
    bytes_per_iteration: f64,
    bytes_to_input_ratio: f64,
    blocks_per_iteration: f64,
    peak_live_bytes: usize,
    retained_bytes_after_iterations: usize,
}

fn main() {
    let results = vec![
        run_workload(
            "unregistered_typical",
            unregistered_typed_workload(),
            registered_redaction,
        ),
        run_workload("typed_typical", typed_workload(), registered_redaction),
        run_workload(
            "unregistered_large",
            large_nested_workload(),
            registered_redaction,
        ),
        run_workload("typed_large", large_typed_workload(), registered_redaction),
    ];
    println!(
        "{}",
        serde_json::to_string_pretty(&results).expect("benchmark results should serialize")
    );
    enforce_targets(&results);
}

fn enforce_targets(results: &[WorkloadResult]) {
    let get = |name| {
        results
            .iter()
            .find(|result| result.workload == name)
            .unwrap_or_else(|| panic!("missing benchmark workload {name}"))
    };
    let unregistered_typical = get("unregistered_typical");
    let typed_typical = get("typed_typical");
    let unregistered_large = get("unregistered_large");
    let typed_large = get("typed_large");

    assert!(
        typed_typical.latency_ns.p99 < 1_000_000,
        "typed typical p99 must remain below 1 ms"
    );
    assert!(
        typed_large.latency_ns.p99 < 5_000_000,
        "typed large p99 must remain below 5 ms"
    );
    assert!(
        typed_typical.allocation.bytes_per_iteration
            <= unregistered_typical.allocation.bytes_per_iteration * 5.0,
        "typed typical allocation must remain within 5x unregistered"
    );
    assert!(
        typed_large.allocation.bytes_per_iteration
            <= unregistered_large.allocation.bytes_per_iteration * 5.0,
        "typed large allocation must remain within 5x unregistered"
    );
    assert!(
        results
            .iter()
            .all(|result| result.allocation.retained_bytes_after_iterations == 0),
        "redaction workloads must retain zero bytes"
    );
}

fn run_workload(
    workload: &'static str,
    (input, expected): (NodeUserConfig, NodeUserConfig),
    redact: fn(&NodeUserConfig) -> NodeUserConfig,
) -> WorkloadResult {
    let input_bytes = serde_json::to_vec(&input)
        .expect("benchmark input should serialize")
        .len();
    for _ in 0..WARMUP_ITERATIONS {
        let actual = redact(&input);
        assert_eq!(actual, expected, "warmup output must match the oracle");
        let _ = black_box(actual);
    }

    let mut samples = Vec::with_capacity(MEASURED_ITERATIONS);
    for _ in 0..MEASURED_ITERATIONS {
        let started = Instant::now();
        let actual = redact(&input);
        let elapsed = started.elapsed();

        assert_eq!(actual, expected, "measured output must match the oracle");
        let _ = black_box(&actual);
        samples.push(elapsed);
    }

    let profiler = dhat::Profiler::builder().testing().build();
    let before = dhat::HeapStats::get();
    for _ in 0..MEASURED_ITERATIONS {
        let actual = redact(&input);
        assert_eq!(actual, expected, "allocation output must match the oracle");
        let _ = black_box(actual);
    }
    let after = dhat::HeapStats::get();
    drop(profiler);

    let total_bytes = after.total_bytes.saturating_sub(before.total_bytes);
    let total_blocks = after.total_blocks.saturating_sub(before.total_blocks);
    let retained_bytes_after_iterations = after.curr_bytes.saturating_sub(before.curr_bytes);

    WorkloadResult {
        workload,
        warmup_iterations: WARMUP_ITERATIONS,
        measured_iterations: MEASURED_ITERATIONS,
        input_bytes,
        expected_hash: stable_hash(&expected),
        latency_ns: latency_stats(&mut samples),
        allocation: AllocationStats {
            total_bytes,
            total_blocks,
            bytes_per_iteration: total_bytes as f64 / MEASURED_ITERATIONS as f64,
            bytes_to_input_ratio: total_bytes as f64
                / MEASURED_ITERATIONS as f64
                / input_bytes as f64,
            blocks_per_iteration: total_blocks as f64 / MEASURED_ITERATIONS as f64,
            peak_live_bytes: after.max_bytes.saturating_sub(before.curr_bytes),
            retained_bytes_after_iterations,
        },
    }
}

fn registered_redaction(input: &NodeUserConfig) -> NodeUserConfig {
    input
        .try_redacted_for_snapshot()
        .expect("registered benchmark redaction should succeed")
}

fn latency_stats(samples: &mut [Duration]) -> LatencyStats {
    samples.sort_unstable();
    let nanos = samples.iter().map(Duration::as_nanos).collect::<Vec<_>>();
    let mean = nanos.iter().sum::<u128>() as f64 / nanos.len() as f64;
    let variance = nanos
        .iter()
        .map(|sample| {
            let delta = *sample as f64 - mean;
            delta * delta
        })
        .sum::<f64>()
        / nanos.len() as f64;

    LatencyStats {
        p50: percentile(&nanos, 50),
        p95: percentile(&nanos, 95),
        p99: percentile(&nanos, 99),
        mean,
        stdev: variance.sqrt(),
    }
}

fn percentile(sorted: &[u128], percentile: usize) -> u128 {
    let index = (sorted.len() * percentile).div_ceil(100).saturating_sub(1);
    sorted[index]
}

fn stable_hash(value: &NodeUserConfig) -> u64 {
    let serialized = serde_json::to_vec(value).expect("expected output should serialize");
    let mut hasher = DefaultHasher::new();
    serialized.hash(&mut hasher);
    hasher.finish()
}

fn large_nested_workload() -> (NodeUserConfig, NodeUserConfig) {
    let mut input_backends = Vec::with_capacity(100);
    let mut expected_backends = Vec::with_capacity(100);

    for index in 0..100 {
        input_backends.push(json!({
            "name": format!("backend-{index}"),
            "payload": "x".repeat(128),
            "headers": {
                "authorization": format!("secret-{index}"),
                "x-tenant": format!("tenant-{index}")
            }
        }));
        expected_backends.push(json!({
            "name": format!("backend-{index}"),
            "payload": "x".repeat(128),
            "headers": {
                "authorization": REDACTED_HEADER_VALUE,
                "x-tenant": REDACTED_HEADER_VALUE
            }
        }));
    }

    (
        node_config(json!({ "backends": input_backends })),
        node_config(json!({ "backends": expected_backends })),
    )
}

fn large_typed_workload() -> (NodeUserConfig, NodeUserConfig) {
    let (unregistered_input, unregistered_expected) = large_nested_workload();
    let mut input = unregistered_input.config;
    let mut expected = unregistered_expected.config;
    let _ = input
        .as_object_mut()
        .expect("large input should be an object")
        .insert(
            "password".to_owned(),
            Value::String("large-typed-secret".to_owned()),
        );
    let _ = expected
        .as_object_mut()
        .expect("large expected output should be an object")
        .insert(
            "password".to_owned(),
            Value::String(REDACTED_HEADER_VALUE.to_owned()),
        );
    (
        node_config_with_type(TYPED_BENCHMARK_URN, input),
        node_config_with_type(TYPED_BENCHMARK_URN, expected),
    )
}

fn typed_workload() -> (NodeUserConfig, NodeUserConfig) {
    let input = node_config_with_type(
        TYPED_BENCHMARK_URN,
        json!({
            "password": "typed-benchmark-secret",
            "endpoint": "https://backend.example",
            "headers": {
                "authorization": "header-secret"
            },
            "metadata": {
                "literal_marker": REDACTED_HEADER_VALUE,
                "enabled": true
            }
        }),
    );
    let expected = node_config_with_type(
        TYPED_BENCHMARK_URN,
        json!({
            "password": REDACTED_HEADER_VALUE,
            "endpoint": "https://backend.example",
            "headers": {
                "authorization": REDACTED_HEADER_VALUE
            },
            "metadata": {
                "literal_marker": REDACTED_HEADER_VALUE,
                "enabled": true
            }
        }),
    );
    (input, expected)
}

fn unregistered_typed_workload() -> (NodeUserConfig, NodeUserConfig) {
    let (typed_input, typed_expected) = typed_workload();
    let mut expected = typed_expected.config;
    expected["password"] = Value::String("typed-benchmark-secret".to_owned());
    (node_config(typed_input.config), node_config(expected))
}

fn node_config(config: Value) -> NodeUserConfig {
    node_config_with_type("urn:otel:exporter:benchmark", config)
}

fn node_config_with_type(component_type: &str, config: Value) -> NodeUserConfig {
    serde_json::from_value(json!({
        "type": component_type,
        "config": config
    }))
    .expect("benchmark config should deserialize")
}
