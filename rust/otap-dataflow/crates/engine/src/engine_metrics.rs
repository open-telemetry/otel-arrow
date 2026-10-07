// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Engine-level metrics for the OTAP dataflow engine.
//!
//! Unlike per-pipeline metrics (which are sampled on each pipeline thread),
//! engine metrics are emitted **once per engine instance** by a dedicated
//! background task spawned by the controller.
//!
//! **Metrics**
//! - `memory_rss` (`ObserveUpDownCounter<u64>`, `{By}`):
//!   Process-wide Resident Set Size -- physical memory currently held in RAM.
//!   Matches what external tools report (e.g. `kubectl top pod`, `htop`, `ps rss`).
//!
//! - `cpu_utilization` (`Gauge<f64>`, `{1}`):
//!   Process-wide CPU utilization as a ratio in `[0, 1]`, normalized across **all
//!   logical CPU cores on the system** (not just the cores assigned to the engine).
//!   Computed as `cpu_delta / (wall_delta x num_system_cores)` over the last
//!   measurement interval. A value of `1.0` means 100% of all system cores are
//!   in use; `0.5` on an 8-core machine corresponds to 4 fully loaded cores.
//!   Aligned with the OTel semantic convention `process.cpu.utilization`.
//!
//! - `memory_pressure_state` (`Gauge<u64>`, `{state}`):
//!   Process-wide memory limiter state encoded as `0=normal`, `1=soft`, `2=hard`.
//!
//! - `process_memory_usage_bytes`, `process_memory_soft_limit_bytes`,
//!   `process_memory_hard_limit_bytes` (`Gauge<u64>`, `{By}`):
//!   Process-wide memory limiter sample and effective limits.
//!
//!   We emit utilization directly (rather than a cumulative `cpu_time` counter)
//!   so that users can read the metric as-is without requiring PromQL `rate()`
//!   or similar query-time derivations.
//!
//!   TODO: Also emit a cumulative `cpu_time` counter (like the Go Collector's
//!   `process_cpu_seconds_total`) for users who prefer query-time computation.
//!
//! - `engine.console_output` (`ObserveCounter<u64>`), one bucket per `console.stream`
//!   (`stdout` or `stderr`) in a single registration: activity of the process-wide
//!   console output service since the controller run started, so output lost to
//!   a stalled or failed writer stays visible after the run recovers. The controller
//!   drains and reports console warnings before the final sample and keeps
//!   observability alive for the handoff. Later diagnostic drops fail the run
//!   instead of being silently lost after metric export has ended.
//!   `frames_submitted`, `frames_enqueue_failed`, `frames_written`, and
//!   `diagnostics_dropped` count frames (`{frame}`), `bytes_written` counts bytes
//!   (`By`), and `write_errors` counts failed writes (`{error}`).

use crate::memory_limiter::MemoryPressureState;
use cpu_time::ProcessTime;
use otel_arrow_dfe_telemetry::instrument::{Gauge, ObserveCounter, ObserveUpDownCounter};
use otel_arrow_dfe_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otel_arrow_dfe_telemetry::output_service::{OutputService, OutputStats, ServiceStats};
use otel_arrow_dfe_telemetry::registry::{EntityKey, TelemetryRegistryHandle};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::{AttributeEnum, attribute_set, metric_set};
use std::time::Instant;

/// Engine-wide metrics emitted once per engine instance.
#[metric_set(name = "engine")]
#[derive(Debug, Default, Clone)]
pub struct EngineMetrics {
    /// Process-wide Resident Set Size -- physical RAM currently used by the process.
    /// Matches what external tools report (e.g. `kubectl top pod`, `htop`, `ps rss`).
    #[metric(unit = "{By}")]
    pub memory_rss: ObserveUpDownCounter<u64>,

    /// Process-wide CPU utilization as a ratio in [0, 1], normalized across all
    /// logical CPU cores on the system (not just engine-assigned cores).
    /// Aligned with the OTel semantic convention `process.cpu.utilization`.
    ///
    /// The `cpu.mode` attribute is not set; this reports combined user + system time.
    #[metric(unit = "{1}")]
    pub cpu_utilization: Gauge<f64>,

    /// Process-wide memory limiter state encoded as `0=normal`, `1=soft`, `2=hard`.
    #[metric(unit = "{state}")]
    pub memory_pressure_state: Gauge<u64>,

    /// Most recent process-wide memory limiter sample, in bytes.
    #[metric(unit = "{By}")]
    pub process_memory_usage_bytes: Gauge<u64>,

    /// Effective process-wide memory limiter soft limit, in bytes.
    #[metric(unit = "{By}")]
    pub process_memory_soft_limit_bytes: Gauge<u64>,

    /// Effective process-wide memory limiter hard limit, in bytes.
    #[metric(unit = "{By}")]
    pub process_memory_hard_limit_bytes: Gauge<u64>,
}

/// Standard stream served by one console output writer.
#[derive(Debug, Clone, Copy, AttributeEnum)]
pub enum ConsoleStream {
    /// Process standard output.
    Stdout,
    /// Process standard error.
    Stderr,
}

/// Identifies the stream a [`ConsoleOutputMetrics`] bucket describes.
#[attribute_set(item, measurement)]
#[derive(Debug, Clone, Copy)]
pub struct ConsoleStreamAttributes {
    /// The stream the bucket describes.
    pub console_stream: ConsoleStream,
}

/// Activity of the process-wide console output service for one stream.
#[metric_set(
    name = "engine.console_output",
    measurement_attributes = ConsoleStreamAttributes
)]
#[derive(Debug, Default, Clone)]
pub struct ConsoleOutputMetrics {
    /// Frames accepted into the stream's queue.
    #[metric(unit = "{frame}")]
    pub frames_submitted: ObserveCounter<u64>,

    /// Frames rejected because the queue was closed or full, the frame was too large,
    /// or its writer had failed.
    #[metric(unit = "{frame}")]
    pub frames_enqueue_failed: ObserveCounter<u64>,

    /// Frames written to the stream.
    #[metric(unit = "{frame}")]
    pub frames_written: ObserveCounter<u64>,

    /// Bytes written to the stream.
    #[metric(unit = "By")]
    pub bytes_written: ObserveCounter<u64>,

    /// Failed writes or flushes. Each one stops the stream's writer.
    #[metric(unit = "{error}")]
    pub write_errors: ObserveCounter<u64>,

    /// Best-effort diagnostics dropped because the queue was full or the frame
    /// exceeded the stream's byte budget.
    #[metric(unit = "{frame}")]
    pub diagnostics_dropped: ObserveCounter<u64>,
}

impl ConsoleOutputMetrics {
    /// Records one stream's activity since `baseline`.
    ///
    /// The service counters live as long as the process, but each engine run
    /// registers this set afresh, and its cumulative start time is that
    /// registration. Subtracting the totals seen at that start keeps an earlier
    /// run's activity out of this run's series.
    fn observe_since(&mut self, stats: &OutputStats, baseline: &OutputStats) {
        self.frames_submitted.observe(
            stats
                .frames_submitted
                .saturating_sub(baseline.frames_submitted),
        );
        self.frames_enqueue_failed.observe(
            stats
                .frames_enqueue_failed
                .saturating_sub(baseline.frames_enqueue_failed),
        );
        self.frames_written
            .observe(stats.frames_written.saturating_sub(baseline.frames_written));
        self.bytes_written
            .observe(stats.bytes_written.saturating_sub(baseline.bytes_written));
        self.write_errors
            .observe(stats.write_errors.saturating_sub(baseline.write_errors));
        self.diagnostics_dropped.observe(
            stats
                .diagnostics_dropped
                .saturating_sub(baseline.diagnostics_dropped),
        );
    }
}

/// Monitors and reports engine-wide metrics.
///
/// Created by the controller and driven by a periodic timer in a dedicated
/// background task. Call [`update`](Self::update) to sample current values
/// and [`report`](Self::report) to flush them to the metrics pipeline.
pub struct EngineMetricsMonitor {
    metrics: MetricSet<EngineMetrics>,
    /// Both console streams as buckets of one registration, so the registry never
    /// holds two sets with the same metric identity for the engine entity.
    console_output: MeasurementMetricSet<ConsoleOutputMetrics>,
    /// Process-wide console output totals at the start of this run or monitor.
    console_baseline: ServiceStats,
    reporter: MetricsReporter,
    registry: TelemetryRegistryHandle,
    /// Wall-clock anchor for the current measurement interval.
    wall_start: Instant,
    /// Process-wide CPU time anchor for the current measurement interval.
    cpu_start: ProcessTime,
    /// Total number of logical CPU cores available on the system.
    num_cores: usize,
    /// Shared process-wide memory limiter state.
    memory_pressure_state: MemoryPressureState,
}

impl EngineMetricsMonitor {
    /// Creates a new engine metrics monitor.
    ///
    /// The caller must have already registered the engine entity via
    /// [`ControllerContext::register_engine_entity`](crate::context::ControllerContext::register_engine_entity).
    /// Standalone callers start console counters at construction; controller runs
    /// use [`Self::with_console_baseline`] to include startup diagnostics.
    #[must_use]
    pub fn new(
        registry: TelemetryRegistryHandle,
        entity_key: EntityKey,
        reporter: MetricsReporter,
        memory_pressure_state: MemoryPressureState,
    ) -> Self {
        Self::with_console_baseline(
            registry,
            entity_key,
            reporter,
            memory_pressure_state,
            OutputService::stats(),
        )
    }

    /// Creates a monitor whose console output metrics count activity after `console_baseline`.
    /// A controller run supplies its start snapshot to include startup diagnostics.
    #[must_use]
    pub fn with_console_baseline(
        registry: TelemetryRegistryHandle,
        entity_key: EntityKey,
        reporter: MetricsReporter,
        memory_pressure_state: MemoryPressureState,
        console_baseline: ServiceStats,
    ) -> Self {
        let metrics = registry.register_metric_set_for_entity::<EngineMetrics>(entity_key);
        let console_output = registry
            .register_metric_set_with_measurement_attributes_for_entity::<ConsoleOutputMetrics>(
                entity_key,
            );
        let num_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        Self {
            metrics,
            console_output,
            console_baseline,
            reporter,
            registry,
            wall_start: Instant::now(),
            cpu_start: ProcessTime::now(),
            num_cores,
            memory_pressure_state,
        }
    }

    /// Samples current engine-wide metrics (RSS, CPU utilization, etc.).
    pub fn update(&mut self) {
        self.update_with_console_stats(OutputService::stats());
    }

    fn update_with_console_stats(&mut self, console_stats: ServiceStats) {
        self.metrics.memory_rss.observe(get_rss_bytes());

        // Compute process-wide CPU utilization normalized across all cores.
        let now_wall = Instant::now();
        let now_cpu = ProcessTime::now();
        let wall_delta = now_wall.duration_since(self.wall_start);
        let cpu_delta = now_cpu.duration_since(self.cpu_start);
        let wall_secs = wall_delta.as_secs_f64();
        if wall_secs > 0.0 {
            let utilization =
                (cpu_delta.as_secs_f64() / (wall_secs * self.num_cores as f64)).clamp(0.0, 1.0);
            self.metrics.cpu_utilization.set(utilization);
        } else {
            self.metrics.cpu_utilization.set(0.0);
        }
        self.metrics
            .memory_pressure_state
            .set(self.memory_pressure_state.level() as u64);
        self.metrics
            .process_memory_usage_bytes
            .set(self.memory_pressure_state.usage_bytes());
        self.metrics
            .process_memory_soft_limit_bytes
            .set(self.memory_pressure_state.soft_limit_bytes());
        self.metrics
            .process_memory_hard_limit_bytes
            .set(self.memory_pressure_state.hard_limit_bytes());
        self.observe_console_output(console_stats);
        self.wall_start = now_wall;
        self.cpu_start = now_cpu;
    }

    /// Records both streams' console output activity since the supplied baseline.
    fn observe_console_output(&mut self, stats: ServiceStats) {
        let baseline = self.console_baseline;
        self.console_output
            .with(ConsoleStreamAttributes {
                console_stream: ConsoleStream::Stdout,
            })
            .observe_since(&stats.stdout, &baseline.stdout);
        self.console_output
            .with(ConsoleStreamAttributes {
                console_stream: ConsoleStream::Stderr,
            })
            .observe_since(&stats.stderr, &baseline.stderr);
    }

    /// Flushes sampled metrics to the reporting pipeline.
    ///
    /// Returns an error only if the metrics channel is permanently closed.
    /// A full channel is silently tolerated (non-blocking, try-send semantics).
    pub fn report(&mut self) -> Result<(), otel_arrow_dfe_telemetry::error::Error> {
        self.reporter.report(&mut self.metrics)?;
        self.reporter.report_measurement(&mut self.console_output)
    }

    /// Samples and reliably hands off final values without exceeding `deadline`.
    /// Returns the sampled console totals so the controller can detect later losses.
    ///
    /// The collector barrier completes before this monitor unregisters its
    /// metric-set key, preventing an accepted terminal snapshot from arriving
    /// after the registry entry has been removed.
    pub async fn finish_reporting_until(
        &mut self,
        deadline: Instant,
    ) -> Result<ServiceStats, otel_arrow_dfe_telemetry::error::Error> {
        let console_stats = OutputService::stats();
        self.update_with_console_stats(console_stats);
        let _ = self
            .reporter
            .report_reliably_until(&mut self.metrics, deadline)
            .await?;
        let _ = self
            .reporter
            .report_measurement_reliably_until(&mut self.console_output, deadline)
            .await?;
        self.reporter.flush_until(deadline).await?;
        Ok(console_stats)
    }
}

/// Returns the current process-wide RSS (Resident Set Size) in bytes.
fn get_rss_bytes() -> u64 {
    memory_stats::memory_stats()
        .map(|stats| stats.physical_mem as u64)
        .unwrap_or(0)
}

impl Drop for EngineMetricsMonitor {
    fn drop(&mut self) {
        for key in [
            self.metrics.metric_set_key(),
            self.console_output.metric_set_key(),
        ] {
            let _ = self.registry.unregister_metric_set(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ControllerContext;
    use otel_arrow_dfe_telemetry::metrics::{MetricSetHandler, MetricValue};
    use otel_arrow_dfe_telemetry::output_service::{
        Frame, OutputSink, OutputStream, StreamId, SubmitError,
    };
    use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
    use std::io;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicBool, Ordering};

    #[test]
    fn engine_metrics_reports_nonzero_rss() {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry.clone());
        let entity_key = controller.register_engine_entity();
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(16);

        let mut monitor = EngineMetricsMonitor::new(
            registry,
            entity_key,
            reporter,
            controller.memory_pressure_state(),
        );
        monitor.update();

        assert!(
            monitor.metrics.memory_rss.get() > 0,
            "memory_rss should report non-zero process RSS"
        );
    }

    #[test]
    fn engine_metrics_report_succeeds() {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry.clone());
        let entity_key = controller.register_engine_entity();
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(16);

        let mut monitor = EngineMetricsMonitor::new(
            registry,
            entity_key,
            reporter,
            controller.memory_pressure_state(),
        );
        monitor.update();
        assert!(monitor.report().is_ok());
    }

    #[test]
    fn engine_metrics_cpu_utilization_in_range() {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry.clone());
        let entity_key = controller.register_engine_entity();
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(16);

        let mut monitor = EngineMetricsMonitor::new(
            registry,
            entity_key,
            reporter,
            controller.memory_pressure_state(),
        );

        // Do a small busy-spin so there is measurable CPU time.
        let start = Instant::now();
        while start.elapsed() < std::time::Duration::from_millis(10) {
            let _ = std::hint::black_box(0u64.wrapping_add(1));
        }

        monitor.update();
        let util = monitor.metrics.cpu_utilization.get();
        assert!(
            (0.0..=1.0).contains(&util),
            "cpu_utilization should be in [0, 1], got {util}"
        );
    }

    #[test]
    fn engine_metrics_expose_process_memory_limiter_usage_and_limits() {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry.clone());
        let state = controller.memory_pressure_state();
        state.configure(crate::memory_limiter::MemoryPressureBehaviorConfig {
            retry_after_secs: 1,
            fail_readiness_on_hard: true,
            mode: otel_arrow_dfe_config::policy::MemoryLimiterMode::Enforce,
        });
        state.set_sample_for_tests(
            crate::memory_limiter::MemoryPressureLevel::Soft,
            95,
            90,
            100,
        );

        let entity_key = controller.register_engine_entity();
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(16);
        let mut monitor = EngineMetricsMonitor::new(registry, entity_key, reporter, state);

        monitor.update();

        assert_eq!(monitor.metrics.memory_pressure_state.get(), 1);
        assert_eq!(monitor.metrics.process_memory_usage_bytes.get(), 95);
        assert_eq!(monitor.metrics.process_memory_soft_limit_bytes.get(), 90);
        assert_eq!(monitor.metrics.process_memory_hard_limit_bytes.get(), 100);
    }

    /// Sink whose writes block until the test releases them.
    struct StalledSink(Arc<AtomicBool>);

    impl OutputSink for StalledSink {
        fn write_frame(&mut self, _frame: &[u8]) -> io::Result<()> {
            while self.0.load(Ordering::Acquire) {
                std::thread::sleep(std::time::Duration::from_millis(5));
            }
            Ok(())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    const STDOUT: ConsoleStreamAttributes = ConsoleStreamAttributes {
        console_stream: ConsoleStream::Stdout,
    };
    const STDERR: ConsoleStreamAttributes = ConsoleStreamAttributes {
        console_stream: ConsoleStream::Stderr,
    };

    /// Scenario: engine metric descriptors are compared with the operator reference.
    /// Guarantees: each instrument is documented with its actual OTLP scope and derived name.
    #[test]
    fn engine_metric_reference_matches_descriptors() {
        let reference = include_str!("../telemetry.md");
        for descriptor in [
            EngineMetrics::default().descriptor(),
            ConsoleOutputMetrics::default().descriptor(),
        ] {
            for field in descriptor.metrics {
                let row = format!("| `{}` | `{}` |", descriptor.name, field.name);
                assert!(
                    reference.contains(&row),
                    "missing metric reference row: {row}"
                );
            }
        }
    }

    /// Scenario: diagnostics are dropped on a saturated stderr stream, and the engine metrics
    /// then sample the console output totals.
    /// Guarantees: the stderr bucket of `engine.console_output` reports the drop count while
    /// the stdout bucket stays at zero, so lost diagnostics remain visible after the run
    /// recovers.
    #[test]
    fn engine_metrics_expose_dropped_console_diagnostics() {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry.clone());
        let entity_key = controller.register_engine_entity();
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(16);
        // The stream below is private to this test, so its counters start at zero.
        let mut monitor = EngineMetricsMonitor::with_console_baseline(
            registry,
            entity_key,
            reporter,
            controller.memory_pressure_state(),
            ServiceStats::default(),
        );

        let stalled = Arc::new(AtomicBool::new(true));
        let stream = OutputStream::start(
            StreamId::Stderr,
            1,
            1024 * 1024,
            true,
            Box::new(StalledSink(Arc::clone(&stalled))),
        )
        .expect("writer thread spawns");
        let handle = stream.handle();
        let dropped = (0..16)
            .filter(|_| {
                handle.try_submit(Frame::line("diagnostic")) == Err(SubmitError::WouldBlock)
            })
            .count() as u64;
        assert!(
            dropped > 0,
            "the stalled one-frame queue must drop diagnostics"
        );

        monitor.observe_console_output(ServiceStats {
            stdout: OutputStats::default(),
            stderr: stream.stats(),
        });

        assert_eq!(
            monitor.console_output.get(STDERR).diagnostics_dropped.get(),
            dropped
        );
        assert_eq!(
            monitor.console_output.get(STDOUT).diagnostics_dropped.get(),
            0
        );
        assert!(monitor.report().is_ok());
        stalled.store(false, Ordering::Release);
        let _ = stream.shutdown(std::time::Duration::from_secs(5));
    }

    /// Scenario: two engine runs follow each other in one process, and the first run's
    /// console streams already wrote output and dropped diagnostics before the second
    /// run's monitor starts.
    /// Guarantees: each run's `engine.console_output` values count only that run's console
    /// activity, so the second run's fresh cumulative start time never covers the first
    /// run's drops.
    #[test]
    fn console_output_counts_only_activity_since_the_monitor_started() {
        let first_run_totals = ServiceStats {
            stdout: OutputStats {
                frames_submitted: 10,
                frames_written: 10,
                bytes_written: 640,
                ..OutputStats::default()
            },
            stderr: OutputStats {
                frames_submitted: 4,
                frames_enqueue_failed: 3,
                frames_written: 4,
                bytes_written: 200,
                diagnostics_dropped: 3,
                ..OutputStats::default()
            },
        };
        // Each run builds its own registry, as `Controller::run_pipelines` does.
        let first_registry = TelemetryRegistryHandle::new();
        let first_controller = ControllerContext::new(first_registry.clone());
        let (_first_rx, first_reporter) = MetricsReporter::create_new_and_receiver(16);
        let mut first_run = EngineMetricsMonitor::with_console_baseline(
            first_registry,
            first_controller.register_engine_entity(),
            first_reporter,
            first_controller.memory_pressure_state(),
            ServiceStats::default(),
        );
        first_run.observe_console_output(first_run_totals);
        assert_eq!(
            first_run
                .console_output
                .get(STDERR)
                .diagnostics_dropped
                .get(),
            3
        );
        drop(first_run);

        let second_run_totals = ServiceStats {
            stdout: OutputStats {
                frames_submitted: 12,
                frames_written: 12,
                bytes_written: 704,
                ..OutputStats::default()
            },
            stderr: OutputStats {
                frames_submitted: 4,
                frames_enqueue_failed: 4,
                frames_written: 4,
                bytes_written: 200,
                write_errors: 1,
                diagnostics_dropped: 4,
                ..OutputStats::default()
            },
        };
        // The second run starts where the process-wide totals stood after the first run.
        let second_registry = TelemetryRegistryHandle::new();
        let second_controller = ControllerContext::new(second_registry.clone());
        let (_second_rx, second_reporter) = MetricsReporter::create_new_and_receiver(16);
        let mut second_run = EngineMetricsMonitor::with_console_baseline(
            second_registry,
            second_controller.register_engine_entity(),
            second_reporter,
            second_controller.memory_pressure_state(),
            first_run_totals,
        );
        second_run.observe_console_output(second_run_totals);

        let stdout = second_run.console_output.get(STDOUT);
        assert_eq!(stdout.frames_submitted.get(), 2);
        assert_eq!(stdout.frames_written.get(), 2);
        assert_eq!(stdout.bytes_written.get(), 64);
        let stderr = second_run.console_output.get(STDERR);
        assert_eq!(stderr.frames_submitted.get(), 0);
        assert_eq!(stderr.frames_enqueue_failed.get(), 1);
        assert_eq!(stderr.frames_written.get(), 0);
        assert_eq!(stderr.bytes_written.get(), 0);
        assert_eq!(stderr.write_errors.get(), 1);
        assert_eq!(stderr.diagnostics_dropped.get(), 1);
    }

    /// Scenario: the engine metrics monitor registers its console output metrics, samples
    /// both streams, and the registry exports the reported values.
    /// Guarantees: both streams share one `engine.console_output` registration, so the
    /// registry holds no second set with the same metric identity for the engine entity,
    /// and the export still carries each stream's values under its own `console.stream`
    /// attribute.
    #[test]
    fn console_output_streams_share_one_registration() {
        let registry = TelemetryRegistryHandle::new();
        let controller = ControllerContext::new(registry.clone());
        let entity_key = controller.register_engine_entity();
        let (rx, reporter) = MetricsReporter::create_new_and_receiver(16);
        let registered_before = registry.metric_set_count();
        let mut monitor = EngineMetricsMonitor::with_console_baseline(
            registry.clone(),
            entity_key,
            reporter,
            controller.memory_pressure_state(),
            ServiceStats::default(),
        );
        // One `engine` set and one `engine.console_output` set for both streams.
        assert_eq!(registry.metric_set_count(), registered_before + 2);

        monitor.observe_console_output(ServiceStats {
            stdout: OutputStats {
                frames_written: 2,
                ..OutputStats::default()
            },
            stderr: OutputStats {
                diagnostics_dropped: 5,
                ..OutputStats::default()
            },
        });
        monitor.report().expect("the metrics channel is open");
        for snapshot in rx.try_iter() {
            registry.accumulate_metric_set_snapshot(
                snapshot.key(),
                snapshot.bucket(),
                snapshot.get_metrics(),
            );
        }

        let batch = registry.drain_metric_export_batch();
        let mut streams: Vec<_> = batch
            .metric_sets
            .iter()
            .filter(|set| set.descriptor.name == "engine.console_output")
            .map(|set| {
                let value = |name: &str| {
                    let index = set
                        .descriptor
                        .metrics
                        .iter()
                        .position(|field| field.name == name)
                        .expect("console output field exists");
                    set.values[index].clone()
                };
                (
                    set.item_attributes.clone(),
                    value("frames.written"),
                    value("diagnostics.dropped"),
                )
            })
            .collect();
        streams.sort_by(|left, right| left.0.cmp(&right.0));
        assert_eq!(
            streams,
            vec![
                (
                    vec![("console.stream".to_owned(), "stderr".to_owned())],
                    MetricValue::U64(0),
                    MetricValue::U64(5),
                ),
                (
                    vec![("console.stream".to_owned(), "stdout".to_owned())],
                    MetricValue::U64(2),
                    MetricValue::U64(0),
                ),
            ]
        );
    }
}
