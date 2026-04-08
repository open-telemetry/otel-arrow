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
//!   Process-wide Resident Set Size — physical memory currently held in RAM.
//!   Matches what external tools report (e.g. `kubectl top pod`, `htop`, `ps rss`).
//!
//! - `cpu_utilization` (`Gauge<f64>`, `{1}`):
//!   Process-wide CPU utilization as a ratio in `[0, 1]`, normalized across **all
//!   logical CPU cores on the system** (not just the cores assigned to the engine).
//!   Computed as `cpu_delta / (wall_delta × num_system_cores)` over the last
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

use crate::memory_limiter::MemoryPressureState;
use cpu_time::ProcessTime;
use otap_df_telemetry::instrument::{Gauge, ObserveUpDownCounter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::registry::{EntityKey, TelemetryRegistryHandle};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use std::time::Instant;

/// Engine-wide metrics emitted once per engine instance.
#[metric_set(name = "engine.metrics")]
#[derive(Debug, Default, Clone)]
pub struct EngineMetrics {
    /// Process-wide Resident Set Size — physical RAM currently used by the process.
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

/// Monitors and reports engine-wide metrics.
///
/// Created by the controller and driven by a periodic timer in a dedicated
/// background task. Call [`update`](Self::update) to sample current values
/// and [`report`](Self::report) to flush them to the metrics pipeline.
pub struct EngineMetricsMonitor {
    metrics: MetricSet<EngineMetrics>,
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
    #[must_use]
    pub fn new(
        registry: TelemetryRegistryHandle,
        entity_key: EntityKey,
        reporter: MetricsReporter,
        memory_pressure_state: MemoryPressureState,
    ) -> Self {
        let metrics = registry.register_metric_set_for_entity::<EngineMetrics>(entity_key);
        let num_cores = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(1);
        Self {
            metrics,
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
        self.wall_start = now_wall;
        self.cpu_start = now_cpu;
    }

    /// Flushes sampled metrics to the reporting pipeline.
    ///
    /// Returns an error only if the metrics channel is permanently closed.
    /// A full channel is silently tolerated (non-blocking, try-send semantics).
    pub fn report(&mut self) -> Result<(), otap_df_telemetry::error::Error> {
        self.reporter.report(&mut self.metrics)
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
        let _ = self
            .registry
            .unregister_metric_set(self.metrics.metric_set_key());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ControllerContext;
    use otap_df_telemetry::registry::TelemetryRegistryHandle;

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
            mode: otap_df_config::policy::MemoryLimiterMode::Enforce,
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
}
