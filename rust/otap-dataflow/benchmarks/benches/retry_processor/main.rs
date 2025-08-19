// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Retry Processor Performance Benchmarks
//!
//! This benchmark suite measures the performance characteristics of the retry processor
//! using a **microbenchmark approach** that focuses on individual operation latency
//! rather than system-level throughput.
//!
//! For details on the retry processor's ACK/NACK feedback loop architecture and behavior,
//! see the [`otap_df_engine::retry_processor`] module documentation.
//!
//! ## Microbenchmark Approach
//!
//! This suite measures:
//! - **Individual operation latency**: Time for single ACK/NACK/PData operations
//! - **Scaling characteristics**: How timer tick performance changes with pending message count
//! - **Component bottlenecks**: Which specific code paths are most expensive
//!
//! We deliberately avoid throughput testing here because:
//! - Criterion automatically calculates throughput from timing data
//! - The retry processor operates in a single async task context
//! - Individual operation latency is more useful for component optimization
//! - Throughput testing is better suited for integration-level pipeline benchmarks
//!
//! ## Test Scales
//!
//! HashMap size testing uses small, representative scales (10, 50, 100) since:
//! - HashMap operations are O(1), so massive scales don't provide additional insight
//! - Criterion performs statistical sampling for reliable measurements
//! - Smaller scales keep benchmark execution fast and focused

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use mimalloc::MiMalloc;
use otap_df_channel::mpsc;
use otap_df_engine::retry_processor::{RetryConfig, RetryProcessor};
use otap_df_engine::{
    control::NodeControlMsg,
    local::message::LocalSender,
    local::processor::{EffectHandler, Processor},
    message::Message,
};
use otap_df_otlp::grpc::OTLPData;
use otap_df_otlp::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use std::collections::HashMap;
use std::hint::black_box;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

// Test with small, representative HashMap sizes for microbenchmarking
const HASHMAP_SIZES: &[usize] = &[10, 50, 100];

/// Creates a retry processor with some pending messages for testing
fn create_processor_with_pending(
    pending_count: usize,
) -> (
    RetryProcessor<OTLPData>,
    EffectHandler<OTLPData>,
    mpsc::Receiver<OTLPData>,
) {
    let config = RetryConfig {
        max_retries: 3,
        initial_retry_delay_ms: 1000,
        max_retry_delay_ms: 30000,
        backoff_multiplier: 2.0,
        max_pending_messages: (pending_count * 2).max(100), // Ensure minimum capacity
        cleanup_interval_secs: 3600,
    };
    let processor = RetryProcessor::with_config(config);
    let (sender, receiver) = mpsc::Channel::new(1000);
    let mut senders_map = HashMap::new();
    let _ = senders_map.insert("out".into(), LocalSender::MpscSender(sender));
    let effect_handler = EffectHandler::new("bench_processor".into(), senders_map, None);

    // Pre-populate with pending messages by adding and NACKing them
    // Note: This setup is not timed as part of the benchmark
    (processor, effect_handler, receiver)
}

/// Creates test OTLP logs data for benchmarking
fn create_otlp_logs_data() -> OTLPData {
    let logs_request = ExportLogsServiceRequest::default();
    OTLPData::Logs(logs_request)
}

/// Benchmark 1: Individual message operations
fn bench_individual_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let mut group = c.benchmark_group("retry_processor_operations");

    // Benchmark: Process single PData message
    let _ = group.bench_function("process_pdata", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut processor, mut effect_handler, _receiver) = create_processor_with_pending(0);
            let otlp_data = create_otlp_logs_data();

            processor
                .process(Message::PData(otlp_data), &mut effect_handler)
                .await
                .expect("Failed to process PData message");
            black_box(());
        });
    });

    // Benchmark: Process single ACK
    let _ = group.bench_function("process_ack", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut processor, mut effect_handler, _receiver) = create_processor_with_pending(0);

            // First add a message to have something to ACK
            let otlp_data = create_otlp_logs_data();
            processor
                .process(Message::PData(otlp_data), &mut effect_handler)
                .await
                .expect("Failed to process PData message");

            // Now benchmark the ACK operation
            processor
                .process(
                    Message::Control(NodeControlMsg::Ack { id: 1 }),
                    &mut effect_handler,
                )
                .await
                .expect("Failed to process ACK message");
            black_box(());
        });
    });

    // Benchmark: Process single NACK
    let _ = group.bench_function("process_nack", |b| {
        b.to_async(&rt).iter(|| async {
            let (mut processor, mut effect_handler, _receiver) = create_processor_with_pending(0);

            // First add a message to have something to NACK
            let otlp_data = create_otlp_logs_data();
            processor
                .process(Message::PData(otlp_data), &mut effect_handler)
                .await
                .expect("Failed to process PData message");

            // Now benchmark the NACK operation
            processor
                .process(
                    Message::Control(NodeControlMsg::Nack {
                        id: 1,
                        reason: "Benchmark failure".to_string(),
                    }),
                    &mut effect_handler,
                )
                .await
                .expect("Failed to process NACK message");
            black_box(());
        });
    });

    group.finish();
}

/// Benchmark 2: Timer tick performance with different HashMap sizes
fn bench_timer_tick_scaling(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let mut group = c.benchmark_group("retry_processor_timer_tick");

    for &hashmap_size in HASHMAP_SIZES {
        let _ = group.bench_function(
            BenchmarkId::new("timer_tick_with_pending", hashmap_size),
            |b| {
                b.to_async(&rt).iter(|| async {
                    // Setup: Create processor with N pending messages (not timed)
                    let (mut processor, mut effect_handler, _receiver) =
                        create_processor_with_pending(hashmap_size);

                    // Pre-populate with pending messages
                    for i in 0..hashmap_size {
                        let otlp_data = create_otlp_logs_data();
                        processor
                            .process(Message::PData(otlp_data), &mut effect_handler)
                            .await
                            .expect("Failed to process PData message");
                        // NACK to keep in pending state
                        processor
                            .process(
                                Message::Control(NodeControlMsg::Nack {
                                    id: i as u64 + 1,
                                    reason: "Keep pending".to_string(),
                                }),
                                &mut effect_handler,
                            )
                            .await
                            .expect("Failed to process NACK message");
                    }

                    // Benchmark: Timer tick that needs to check all pending messages
                    processor
                        .process(
                            Message::Control(NodeControlMsg::TimerTick {}),
                            &mut effect_handler,
                        )
                        .await
                        .expect("Failed to process timer tick");
                    black_box(());
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_individual_operations,
    bench_timer_tick_scaling
);
criterion_main!(benches);
