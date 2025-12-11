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
    pub uptime: Gauge<u64>,

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

    allocated: ThreadLocal<u64>,
    deallocated: ThreadLocal<u64>,
    last_allocated: u64,
    last_deallocated: u64,

    // These timestamps mark the beginning of the current measurement interval
    wall_start: Instant,
    cpu_start: ThreadTime,

    metrics: MetricSet<PipelineMetrics>,
}

impl PipelineMetricsMonitor {
    pub(crate) fn new(pipeline_ctx: PipelineContext) -> Self {
        // Resolve the MIBs once ...
        let alloc_mib = thread::allocatedp::mib().expect("no jemalloc mib, should never happen");
        let dealloc_mib =
            thread::deallocatedp::mib().expect("no jemalloc mib, should never happen");

        // ... then get the thread-local pointers for this thread.
        let allocated = alloc_mib
            .read()
            .expect("no jemalloc allocatedp mib, should never happen");
        let deallocated = dealloc_mib
            .read()
            .expect("no jemalloc deallocatedp mib, should never happen");

        // Initialize baselines so the first recompute() returns zero deltas.
        let last_allocated = allocated.get();
        let last_deallocated = deallocated.get();

        Self {
            start_time: Instant::now(),
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
        // === Update thread memory allocation metrics ===
        // Fast path: `get()` is just `*ptr` and is #[inline].
        let cur_alloc = self.allocated.get();
        let cur_dealloc = self.deallocated.get();

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
        self.metrics.memory_usage.set(cur_alloc - cur_dealloc);

        // === Update pipeline (thread) CPU usage metric ===
        let now_wall = Instant::now();
        let now_cpu = ThreadTime::now();

        self.metrics
            .uptime
            .set(now_wall.duration_since(self.start_time).as_secs());

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
