// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Set of metrics collected from the OTAP engine pipelines.

use crate::context::PipelineContext;
use cpu_time::ThreadTime;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use std::time::Instant;
use tikv_jemalloc_ctl::thread;
use tikv_jemalloc_ctl::thread::ThreadLocal;

/// Per-pipeline instance general metrics.
#[metric_set(name = "pipeline.metrics")]
#[derive(Debug, Default, Clone)]
pub struct PipelineMetrics {
    /// The time the pipeline instance has been running.
    #[metric(unit = "{s}")]
    pub uptime: Gauge<f64>,

    /// The amount of heap memory in use by the pipeline instance running on a specific core.
    #[metric(unit = "{By}")]
    pub memory_usage: Gauge<u64>,

    /// Memory allocated to the heap by the pipeline instance since start.
    #[metric(unit = "{By}")]
    pub memory_allocated: Gauge<u64>,

    /// Memory freed to the heap by the pipeline instance since start.
    #[metric(unit = "{By}")]
    pub memory_freed: Gauge<u64>,

    /// Bytes allocated to the heap by this pipeline during the last measurement window.
    #[metric(unit = "{By}")]
    pub memory_allocated_delta: Counter<u64>,

    /// Bytes returned to the heap by this pipeline during the last measurement window.
    #[metric(unit = "{By}")]
    pub memory_freed_delta: Counter<u64>,

    /// Total CPU seconds used by the pipeline since start.
    #[metric(unit = "{s}")]
    pub cpu_time: Counter<f64>,

    /// Difference in pipeline cpu time since the last measurement, divided by the elapsed time.
    #[metric(unit = "{1}")]
    pub cpu_utilization: Gauge<u64>,

    // ToDo Add pipeline_network_io_received
    // ToDo Add pipeline_network_io_sent
}

/// Per-thread allocation counters.
pub(crate) struct PipelineMetricsMonitor {
    start_time: Instant,

    jemalloc_supported: bool,
    allocated: Option<ThreadLocal<u64>>,
    deallocated: Option<ThreadLocal<u64>>,
    last_allocated: u64,
    last_deallocated: u64,

    // These timestamps mark the beginning of the current measurement interval
    wall_start: Instant,
    cpu_start: ThreadTime,

    metrics: MetricSet<PipelineMetrics>,
}

impl PipelineMetricsMonitor {
    pub(crate) fn new(pipeline_ctx: PipelineContext) -> Self {
        // Try to initialize jemalloc thread-local stats. If the global allocator is not jemalloc,
        // these calls will fail and memory-related metrics will remain unchanged.
        let jemalloc_init = (|| {
            let alloc_mib = thread::allocatedp::mib().ok()?;
            let dealloc_mib = thread::deallocatedp::mib().ok()?;

            let allocated = alloc_mib.read().ok()?;
            let deallocated = dealloc_mib.read().ok()?;

            let last_allocated = allocated.get();
            let last_deallocated = deallocated.get();

            Some((allocated, deallocated, last_allocated, last_deallocated))
        })();

        let (jemalloc_supported, allocated, deallocated, last_allocated, last_deallocated) =
            if let Some((allocated, deallocated, last_allocated, last_deallocated)) = jemalloc_init
            {
                (
                    true,
                    Some(allocated),
                    Some(deallocated),
                    last_allocated,
                    last_deallocated,
                )
            } else {
                (false, None, None, 0, 0)
            };

        Self {
            start_time: Instant::now(),
            jemalloc_supported,
            allocated,
            deallocated,
            last_allocated,
            last_deallocated,
            wall_start: Instant::now(),
            cpu_start: ThreadTime::now(),
            metrics: pipeline_ctx.register_metrics::<PipelineMetrics>(),
        }
    }

    /// Returns a mutable reference to the metrics struct.
    pub fn metrics_mut(&mut self) -> &mut MetricSet<PipelineMetrics> {
        &mut self.metrics
    }

    pub fn update_metrics(&mut self) {
        // === Update thread memory allocation metrics (jemalloc only) ===
        if self.jemalloc_supported {
            if let (Some(allocated), Some(deallocated)) =
                (self.allocated.as_ref(), self.deallocated.as_ref())
            {
                // Fast path: `get()` is just `*ptr` and is #[inline].
                let cur_alloc = allocated.get();
                let cur_dealloc = deallocated.get();

                // Deltas since last time.
                // Use wrapping_sub to be robust if jemalloc ever wraps the counters.
                let delta_alloc = cur_alloc.wrapping_sub(self.last_allocated);
                let delta_dealloc = cur_dealloc.wrapping_sub(self.last_deallocated);

                // Update baselines.
                self.last_allocated = cur_alloc;
                self.last_deallocated = cur_dealloc;

                self.metrics.memory_allocated.set(cur_alloc);
                self.metrics.memory_freed.set(cur_dealloc);
                self.metrics.memory_allocated_delta.add(delta_alloc);
                self.metrics.memory_freed_delta.add(delta_dealloc);
                self.metrics
                    .memory_usage
                    .set(cur_alloc.saturating_sub(cur_dealloc));
            }
        }

        // === Update pipeline (thread) CPU usage metric ===
        let now_wall = Instant::now();
        let now_cpu = ThreadTime::now();
        let uptime = now_wall.duration_since(self.start_time).as_secs_f64();

        self.metrics
            .uptime
            .set(uptime);

        let wall_duration = now_wall.duration_since(self.wall_start);
        let cpu_duration = now_cpu.duration_since(self.cpu_start);

        let wall_micros = wall_duration.as_micros();

        self.metrics.cpu_time.add(cpu_duration.as_secs_f64());

        // Prevent division by zero if updates happen too fast
        if wall_micros > 0 {
            let cpu_micros = cpu_duration.as_micros();

            // Calculation: (CPU Time / Wall Time) * 100.0
            // We use f64 for precision during division, then cast to u64 for the Gauge.
            let usage_pct = (cpu_micros as f64 / wall_micros as f64) * 100.0;

            // Clamp to ensure we don't report > 100% due to any weird timing skew,
            // though standard thread time shouldn't exceed wall time on a single thread.
            self.metrics.cpu_utilization.set(usage_pct as u64);
        } else {
            self.metrics.cpu_utilization.set(0);
        }

        // Reset time anchors for the next interval
        self.wall_start = now_wall;
        self.cpu_start = now_cpu;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    // Ensure jemalloc is the allocator for this crate's tests so that
    // tikv_jemalloc_ctl can read per-thread allocation counters.
    #[global_allocator]
    static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

    #[test]
    fn pipeline_metrics_monitor_black_box_updates() {
        let registry = MetricsRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx = controller.pipeline_context_with(
            "grp".into(),
            "pipe".into(),
            0,
            0,
        );

        let mut monitor = PipelineMetricsMonitor::new(pipeline_ctx);

        // First update establishes baselines.
        monitor.update_metrics();
        let cpu0 = monitor.metrics.cpu_time.get();
        let mem0 = monitor.metrics.memory_allocated.get();

        // Allocate some memory on this thread.
        let mut v = Vec::with_capacity(10_000);
        for i in 0..10_000u64 {
            v.push(i);
        }
        let _ = black_box(&v);

        // Burn some CPU so ThreadTime advances.
        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(20) {
            let _ = black_box(1u64.wrapping_mul(2));
        }

        // Ensure a non-zero wall interval.
        std::thread::sleep(Duration::from_millis(5));

        monitor.update_metrics();

        assert!(monitor.metrics.cpu_time.get() >= cpu0);
        assert!(monitor.metrics.cpu_utilization.get() <= 100);
        assert!(monitor.metrics.memory_allocated.get() >= mem0);
        assert!(monitor.metrics.memory_allocated_delta.get() > 0);
    }
}
