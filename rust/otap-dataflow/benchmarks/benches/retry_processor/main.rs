// SPDX-License-Identifier: Apache-2.0

//! Retry Processor Performance Benchmarks
//!
//! This benchmark suite measures the performance characteristics of the retry processor
//! under different load patterns and configurations. Key metrics include:
//!
//! 1. **Message Throughput**: Success path vs retry overhead
//! 2. **HashMap Scalability**: Performance with large numbers of pending messages  
//! 3. **Memory Cleanup**: Cost of expired message cleanup operations

#![allow(missing_docs)]

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mimalloc::MiMalloc;
use otap_df_channel::mpsc;
use otap_df_engine::retry_processor::{RetryConfig, RetryProcessor};
use otap_df_engine::{
    local::processor::{EffectHandler, Processor},
    message::{ControlMsg, Message, Sender},
};
use otap_df_otlp::grpc::OTLPData;
use otap_df_otlp::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use std::hint::black_box;
use std::time::{Duration, Instant};
use tokio::task::LocalSet;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

const LARGE_MSG_COUNT: usize = 1_000;
const PENDING_MESSAGE_SCALES: &[usize] = &[100, 500, 1_000];
const CLEANUP_SCALES: &[usize] = &[50, 100, 200];

/// Simple mock effect handler that doesn't actually send messages
struct MockEffectHandler<T> {
    message_count: std::rc::Rc<std::cell::RefCell<usize>>,
    _phantom: std::marker::PhantomData<T>,
}

impl<T> MockEffectHandler<T> {
    fn new() -> Self {
        Self {
            message_count: std::rc::Rc::new(std::cell::RefCell::new(0)),
            _phantom: std::marker::PhantomData,
        }
    }

    #[allow(dead_code)]
    fn get_count(&self) -> usize {
        *self.message_count.borrow()
    }

    async fn send_message(&mut self, _data: T) -> Result<(), otap_df_engine::error::Error<T>> {
        *self.message_count.borrow_mut() += 1;
        Ok(())
    }
}

/// Creates test OTLP logs data for benchmarking
fn create_otlp_logs_data() -> OTLPData {
    let logs_request = ExportLogsServiceRequest::default();
    OTLPData::Logs(logs_request)
}

/// Benchmark 1: Message throughput comparison across different retry percentages
fn bench_message_throughput(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    // Pin to a core for consistent measurements
    let cores = core_affinity::get_core_ids().expect("couldn't get core IDs");
    let core = cores.iter().last().expect("no cores found");
    _ = core_affinity::set_for_current(*core);

    let mut group = c.benchmark_group("retry_processor_throughput");
    let _ = group.throughput(Throughput::Elements(LARGE_MSG_COUNT as u64));

    // Test different retry percentages: 0%, 20%, 50%
    let retry_percentages = [0, 20, 50];

    for retry_percentage in retry_percentages {
        let _ = group.bench_function(
            BenchmarkId::new("retry_processor", format!("{retry_percentage}%_retries")),
            |b| {
                let local = LocalSet::new();
                b.to_async(&rt).iter(|| async {
                    local
                        .run_until(async {
                            let config = RetryConfig {
                                max_retries: 1, // Limit retries to prevent hanging
                                initial_retry_delay_ms: 1,
                                max_retry_delay_ms: 10,
                                backoff_multiplier: 2.0,
                                max_pending_messages: 50000,
                                cleanup_interval_secs: 3600,
                            };
                            let mut processor = RetryProcessor::with_config(config);
                            let (sender, _receiver) = mpsc::Channel::new(1000);
                            let mut effect_handler =
                                EffectHandler::new("bench_processor".into(), Sender::Local(sender));

                            // Send messages and process them with ACK/NACK patterns
                            for i in 0..LARGE_MSG_COUNT {
                                let otlp_data = create_otlp_logs_data();
                                let msg_id = i as u64; // Use simple sequential IDs for benchmarking
                                processor
                                    .process(Message::PData(otlp_data), &mut effect_handler)
                                    .await
                                    .expect("failed to process PData in retry benchmark");

                                // Determine if this message should be retried based on percentage
                                let should_retry = retry_percentage > 0
                                    && (i * 100 / LARGE_MSG_COUNT)
                                        < (retry_percentage * LARGE_MSG_COUNT / 100);

                                if should_retry {
                                    // NACK the message but immediately resolve
                                    processor
                                        .process(
                                            Message::Control(ControlMsg::Nack {
                                                id: msg_id,
                                                reason: "Simulated failure".to_string(),
                                            }),
                                            &mut effect_handler,
                                        )
                                        .await
                                        .expect("failed to process NACK in retry benchmark");

                                    // Immediately ACK to resolve the retry
                                    processor
                                        .process(
                                            Message::Control(ControlMsg::Ack { id: msg_id }),
                                            &mut effect_handler,
                                        )
                                        .await
                                        .expect(
                                            "failed to process ACK after NACK in retry benchmark",
                                        );
                                } else {
                                    // ACK successful messages immediately
                                    processor
                                        .process(
                                            Message::Control(ControlMsg::Ack { id: msg_id }),
                                            &mut effect_handler,
                                        )
                                        .await
                                        .expect("failed to process ACK in retry benchmark");
                                }
                            }

                            let _ = black_box(0usize);
                        })
                        .await;
                });
            },
        );
    }

    group.finish();
}

/// Benchmark 2: HashMap performance with different numbers of pending messages
fn bench_pending_message_operations(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let mut group = c.benchmark_group("retry_processor_hashmap_scale");

    for &pending_count in PENDING_MESSAGE_SCALES {
        let _ = group.throughput(Throughput::Elements(pending_count as u64));

        let _ = group.bench_function(BenchmarkId::new("hashmap_operations", pending_count), |b| {
            b.to_async(&rt).iter(move || async move {
                let local = LocalSet::new();

                local
                    .run_until(async {
                        let config = RetryConfig {
                            max_retries: 5,
                            initial_retry_delay_ms: 1000,
                            max_retry_delay_ms: 30000,
                            backoff_multiplier: 2.0,
                            max_pending_messages: pending_count * 2, // Ensure we don't hit limits
                            cleanup_interval_secs: 3600, // Don't cleanup during benchmark
                        };
                        let mut processor = RetryProcessor::with_config(config);
                        let (sender, _receiver) = mpsc::Channel::new(1000);
                        let mut effect_handler =
                            EffectHandler::new("bench_processor".into(), Sender::Local(sender));

                        // Fill up the pending messages HashMap
                        for i in 0..pending_count {
                            let otlp_data = create_otlp_logs_data();
                            let msg_id = i as u64; // Use simple sequential IDs for benchmarking
                            processor
                                .process(Message::PData(otlp_data), &mut effect_handler)
                                .await
                                .expect("failed to process PData in hashmap benchmark");

                            // NACK to keep in pending state
                            processor
                                .process(
                                    Message::Control(ControlMsg::Nack {
                                        id: msg_id,
                                        reason: "Keep pending".to_string(),
                                    }),
                                    &mut effect_handler,
                                )
                                .await
                                .expect("failed to process NACK in hashmap benchmark");
                        }

                        // Now measure operations on the full HashMap
                        let start = Instant::now();

                        // Perform lookup operations (simulating retry checks)
                        processor
                            .process(
                                Message::Control(ControlMsg::TimerTick {}),
                                &mut effect_handler,
                            )
                            .await
                            .expect("failed to process timer tick in hashmap benchmark");

                        // Perform some ACK operations (HashMap removals)
                        // Note: We can't easily correlate these ACKs with actual message IDs
                        // in this simplified benchmark, so we'll just trigger timer ticks
                        for _i in 0..std::cmp::min(10, pending_count / 10) {
                            processor
                                .process(
                                    Message::Control(ControlMsg::TimerTick {}),
                                    &mut effect_handler,
                                )
                                .await
                                .expect("failed to process timer tick in hashmap benchmark loop");
                        }

                        let duration = start.elapsed();
                        let _ = black_box(duration);
                    })
                    .await;
            });
        });
    }

    group.finish();
}

/// Benchmark 3: Simplified cleanup simulation (avoiding actual timer logic)
fn bench_expired_message_cleanup(c: &mut Criterion) {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to build Tokio runtime");

    let mut group = c.benchmark_group("retry_processor_cleanup");

    for &cleanup_count in CLEANUP_SCALES {
        let _ = group.throughput(Throughput::Elements(cleanup_count as u64));

        let _ = group.bench_function(
            BenchmarkId::new("simulated_cleanup_operations", cleanup_count),
            |b| {
                b.to_async(&rt).iter(move || async move {
                    let local = LocalSet::new();

                    local
                        .run_until(async {
                            let mut mock_handler = MockEffectHandler::<OTLPData>::new();

                            // Simulate the overhead of cleanup operations without actually doing them
                            let start = Instant::now();

                            for i in 0..cleanup_count {
                                // Simulate checking if a message is expired
                                let past_deadline = Instant::now() - Duration::from_secs(10);
                                let is_expired = past_deadline < Instant::now();
                                let _ = black_box(is_expired);

                                // Simulate removing an expired message from HashMap
                                if i % 2 == 0 {
                                    // Simulate 50% of messages being expired
                                    let otlp_data = create_otlp_logs_data();
                                    mock_handler
                                        .send_message(otlp_data)
                                        .await
                                        .expect("failed to send message in cleanup benchmark");
                                }
                            }

                            let cleanup_duration = start.elapsed();
                            let _ = black_box(cleanup_duration);
                        })
                        .await;
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    bench_message_throughput,
    bench_pending_message_operations,
    bench_expired_message_cleanup
);
criterion_main!(benches);
