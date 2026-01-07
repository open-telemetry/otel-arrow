// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-oriented metrics for the OTAP engine.
//!
//! These metrics are per pipeline instance and are sampled from the current
//! pipeline thread. The engine uses a thread-per-core / share-nothing design:
//! each pipeline specification instantiates one pinned thread per allocated CPU.
//! When multiple pipeline specifications share the same CPU set, multiple pinned
//! pipeline threads time-slice on that core (e.g. 2 pipeline specs => 2 threads
//! pinned per allocated CPU). Each pipeline thread runs a single-threaded async
//! runtime, so blocking operations directly affect that pipeline's metrics.
//! The engine is NUMA-aware; these metrics help localize performance issues to a
//! specific pipeline thread and its NUMA placement.
//!
//! **Metrics and interpretation**
//! - `uptime` (`Gauge<f64>`, `{s}`):
//!   Wall-clock time since this pipeline instance started. Useful for detecting
//!   restarts, warm-up effects, and aligning deltas to a known lifetime.
//!
//! - `cpu_time` (`Counter<f64>`, `{s}`):
//!   Cumulative CPU seconds consumed by the pipeline thread since start.
//!   It advances only when the thread is scheduled on a CPU.
//!   The per-interval increase (`cpu_time_delta`) is the CPU budget actually used.
//!
//! - `cpu_utilization` (`Gauge<f64>`, `{1}`):
//!   Ratio of `cpu_time_delta / wall_time_delta` over the last interval,
//!   clamped to `[0, 1]`. Interpreted as "how CPU-bound this pipeline thread is".
//!   On a core shared by multiple pipeline specs, this reflects the pipeline's
//!   share of that core; summing `cpu_utilization` across co-pinned pipelines
//!   approximates total core load.
//!
//! - `memory_usage` (`ObserveUpDownCounter<u64>`, `{By}`, jemalloc only):
//!   Current heap bytes in use by the pipeline thread:
//!   `memory_allocated - memory_freed`. A rising long-term trend indicates a leak
//!   or persistent buffering growth inside this pipeline.
//!
//! - `memory_allocated` / `memory_freed` (`ObserveCounter<u64>`, `{By}`, jemalloc only):
//!   Cumulative heap bytes allocated/freed by this pipeline since start.
//!   These are baselines for long-term trends and sanity checks.
//!
//! - `memory_allocated_delta` / `memory_freed_delta` (`Counter<u64>`, `{By}`; jemalloc only):
//!   Per-interval heap bytes allocated/freed. They capture allocation churn
//!   even when `memory_usage` is stable. High churn increases allocator and cache
//!   pressure and often correlates with latency variance.
//!
//! - `context_switches_voluntary` (`ObserveCounter<u64>`, `{1}`; OS-dependent):
//!   Cumulative voluntary context switches for the pipeline thread since start
//!   (`getrusage(RUSAGE_THREAD).ru_nvcsw`, normalized to start at 0).
//!   These happen when the thread blocks or yields (I/O waits, lock contention,
//!   channel backpressure, async awaits). For diagnosis, look at the rate
//!   between samples.
//!
//! - `context_switches_involuntary` (`ObserveCounter<u64>`, `{1}`; OS-dependent):
//!   Cumulative involuntary context switches (scheduler preemption) for the pipeline
//!   thread since start (`ru_nivcsw`, normalized to start at 0).
//!   High rate => CPU contention / time-slice expiration. In this engine it often
//!   indicates a core shared by multiple pipeline threads (multiple specs per CPU)
//!   or interference from other system work on that core.
//!
//! - `page_faults_minor` / `page_faults_major`
//!   (`ObserveCounter<u64>`, `{1}`; OS-dependent):
//!   Cumulative minor/major page faults for the pipeline thread since start
//!   (`ru_minflt`/`ru_majflt`, normalized to start at 0). For diagnosis, look at the
//!   rate between samples. Minor faults indicate working-set or locality
//!   churn. Major faults indicate disk I/O due to swapping and almost always imply
//!   severe performance degradation.
//!
//! **Derived signals (not emitted explicitly)**
//! - `net_allocated_delta = memory_allocated_delta - memory_freed_delta` `{By}`
//! - `allocation_rate = memory_allocated_delta / interval_s` `{By/s}`
//! - `deallocation_rate = memory_freed_delta / interval_s` `{By/s}`
//! - `net_allocation_rate = net_allocated_delta / interval_s` `{By/s}`
//! - `cpu_time_delta_s = cpu_time(t) - cpu_time(t-1)` `{s}`
//!
//! **Example diagnosis scenarios**
//! - *CPU-bound pipeline / hot loop*:
//!   `cpu_utilization` near `1.0` on a dedicated core (or near its expected share
//!   on shared cores), rising `cpu_time_delta`,
//!   low `context_switches_*` (flat slope). If throughput drops without more CPU, look
//!   for algorithmic regressions or upstream backpressure.
//!
//! - *Blocked on I/O or locks*:
//!   low `cpu_utilization`, high `context_switches_voluntary` (steep slope).
//!   Example: exporter waiting on network or processor waiting on a mutex.
//!   For receiver pipelines, this can be normal when traffic is low because the
//!   async runtime awaits sockets.
//!   If paired with high queue depth or inflight counts,
//!   indicates downstream slowness.
//!
//! - *CPU contention / co-pinned pipelines*:
//!   medium/high `cpu_utilization` but high `context_switches_involuntary` (steep slope)
//!   and latency spikes. Suggests scheduler preemption from co-pinned pipelines
//!   or other system work; consider allocating more cores, reducing pipeline specs
//!   per core, or isolating workloads.
//!
//! - *Per-core oversubscription signal*:
//!   several pipelines pinned to the same CPU each report stable `cpu_utilization`
//!   around `1/N` with elevated `context_switches_involuntary` (steep slope); the core is
//!   saturated and pipelines are competing.
//!
//! - *Allocator churn*:
//!   stable `memory_usage` but high `memory_allocated_delta` and
//!   `memory_freed_delta` rates. Example: repeatedly rebuilding large buffers
//!   per batch. Often manifests as higher p99 latency despite stable mean.
//!
//! - *Memory leak or unbounded buffering*:
//!   `memory_usage` trending upward over minutes/hours, with positive
//!   `net_allocated_delta` each interval. Helps attribute leaks to a specific
//!   pipeline rather than the whole process.
//!
//! - *Poor locality / working-set thrash*:
//!   rising `page_faults_minor` (steep slope) together with churn and/or falling
//!   `cpu_utilization`. Example: large hash tables or random access patterns
//!   exceeding CPU caches. If localized to one NUMA node, revisit shard placement.
//!
//! - *Swap-induced instability*:
//!   non-zero or spiking `page_faults_major` (steep slope) almost always aligns with
//!   dramatic latency spikes or stalls. Action: reduce memory footprint,
//!   increase RAM, or isolate workloads.
//!
//! - *Receiver load skew under `SO_REUSEPORT`*:
//!   one receiver pipeline thread shows much higher `cpu_utilization` /
//!   `cpu_time_delta` than its peers on the same port, indicating uneven
//!   4-tuple distribution, affinity mismatch, or per-thread downstream imbalance.
//!
//! **Availability**
//! - Memory metrics are only populated when the global allocator is jemalloc.
//!   With other allocators, memory metrics remain at their default (zero) values.
//! - Scheduling and page-fault metrics are sampled per thread using `getrusage`
//!   and are currently available on Linux/FreeBSD/OpenBSD. On unsupported
//!   platforms they remain unchanged.

use crate::context::PipelineContext;
use cpu_time::ThreadTime;
use otap_df_telemetry::instrument::{Counter, Gauge, ObserveCounter, ObserveUpDownCounter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry_macros::metric_set;
use std::time::Instant;

#[cfg(not(windows))]
use tikv_jemalloc_ctl::thread;
#[cfg(not(windows))]
use tikv_jemalloc_ctl::thread::ThreadLocal;

#[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
use nix::sys::resource::{UsageWho, getrusage};

/// Per-pipeline instance general metrics.
///
/// Derived metrics (not emitted explicitly):
/// - net_allocated_delta = memory_allocated_delta - memory_freed_delta  [{By}]
/// - allocation_rate     = memory_allocated_delta / interval_s         [{By/s}]
/// - deallocation_rate   = memory_freed_delta / interval_s             [{By/s}]
/// - net_allocation_rate = net_allocated_delta / interval_s            [{By/s}]
/// - cpu_utilization_ratio = (cpu_time_delta_s / wall_time_delta_s)    [{1}]
///   where cpu_time_delta_s is the per-interval increase in cpu_time.
#[metric_set(name = "pipeline.metrics")]
#[derive(Debug, Default, Clone)]
pub struct PipelineMetrics {
    /// The time the pipeline instance has been running.
    #[metric(unit = "{s}")]
    pub uptime: Gauge<f64>,

    /// The amount of heap memory in use by the pipeline instance running on a specific core.
    #[metric(unit = "{By}")]
    pub memory_usage: ObserveUpDownCounter<u64>,

    /// Memory allocated to the heap by the pipeline instance since start.
    #[metric(unit = "{By}")]
    pub memory_allocated: ObserveCounter<u64>,

    /// Memory freed to the heap by the pipeline instance since start.
    #[metric(unit = "{By}")]
    pub memory_freed: ObserveCounter<u64>,

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
    /// Reported as a ratio in the range [0, 1].
    #[metric(unit = "{1}")]
    pub cpu_utilization: Gauge<f64>,

    /// Number of times the pipeline thread voluntarily yielded the CPU since start.
    ///
    /// A rising rate can indicate scheduling pressure due to blocking work
    /// (I/O, locks, backpressure) and helps distinguish "waiting" from "CPU-bound".
    #[metric(unit = "{1}")]
    pub context_switches_voluntary: ObserveCounter<u64>,

    /// Number of times the pipeline thread was preempted by the scheduler since start.
    ///
    /// A rising rate often indicates CPU contention or time-slice expirations.
    /// When paired with `cpu_utilization`, it helps diagnose CPU time lost to preemption.
    #[metric(unit = "{1}")]
    pub context_switches_involuntary: ObserveCounter<u64>,

    /// Number of minor page faults (served without disk I/O) since start.
    ///
    /// A rising rate indicates memory access pain (poor locality, working-set churn),
    /// even if heap bytes are stable.
    #[metric(unit = "{1}")]
    pub page_faults_minor: ObserveCounter<u64>,

    /// Number of major page faults (requiring disk I/O) since start.
    ///
    /// Any non-trivial rate typically indicates severe memory pressure or swapping,
    /// and correlates strongly with pipeline latency spikes.
    #[metric(unit = "{1}")]
    pub page_faults_major: ObserveCounter<u64>,
    // ToDo Add pipeline_network_io_received
    // ToDo Add pipeline_network_io_sent
}

/// Tokio runtime metrics sampled from the pipeline thread.
///
/// These metrics are collected from [`tokio::runtime::Handle::try_current`] / `Handle::metrics()`
/// and are updated at the same cadence as [`PipelineMetrics`].
///
/// **Notes**
/// - Tokio exposes runtime-level (aggregate) metrics; it does not expose per-task numeric
///   metrics through the stable API.
/// - Some Tokio metrics are per-worker; this metric set reports aggregates across all workers.
/// - When compiled without `tokio_unstable`, only Tokio's stable metrics are included.
///   Additional metrics are included when compiling with `RUSTFLAGS="--cfg tokio_unstable" cargo build --release`.
#[metric_set(name = "tokio.runtime")]
#[derive(Debug, Default, Clone)]
pub struct TokioRuntimeMetrics {
    /// Number of worker threads used by the Tokio runtime.
    ///
    /// For Tokio's `current_thread` runtime, this is always `1`.
    #[metric(unit = "{thread}")]
    pub worker_count: ObserveUpDownCounter<u64>,

    /// Current number of alive tasks in the runtime.
    ///
    /// This value increases when tasks are spawned and decreases when tasks complete.
    #[metric(unit = "{task}")]
    pub task_active_count: ObserveUpDownCounter<u64>,

    /// Current number of tasks pending in the runtime's global/injection queue.
    ///
    /// This queue is used when tasks are spawned or woken from outside the runtime.
    #[metric(unit = "{task}")]
    pub global_task_queue_size: ObserveUpDownCounter<u64>,

    /// Total worker busy time, summed across all runtime workers, since runtime creation.
    ///
    /// With multiple workers, this sum can exceed wall-clock time because workers can be busy
    /// concurrently. This metric is only available on targets with 64-bit atomics.
    #[cfg(target_has_atomic = "64")]
    #[metric(unit = "{s}")]
    pub worker_busy_time: ObserveCounter<f64>,

    /// Total number of times runtime workers parked, summed across all workers, since runtime creation.
    ///
    /// A worker "parks" when it becomes idle and waits for new work.
    /// This metric is only available on targets with 64-bit atomics.
    #[cfg(target_has_atomic = "64")]
    #[metric(unit = "{park}")]
    pub worker_park_count: ObserveCounter<u64>,

    /// Total number of park/unpark transitions, summed across all workers, since runtime creation.
    ///
    /// An odd value means at least one worker is currently parked; an even value means all workers
    /// are currently active. This metric is only available on targets with 64-bit atomics.
    #[cfg(target_has_atomic = "64")]
    #[metric(unit = "{unpark}")]
    pub worker_park_unpark_count: ObserveCounter<u64>,

    /// Current number of tasks pending in the blocking thread pool.
    ///
    /// This counts tasks spawned using `spawn_blocking` that have not yet started.
    #[cfg(tokio_unstable)]
    #[metric(unit = "{task}")]
    pub blocking_task_queue_size: ObserveUpDownCounter<u64>,

    /// Current number of threads created for the blocking thread pool.
    #[cfg(tokio_unstable)]
    #[metric(unit = "{thread}")]
    pub blocking_thread_count: ObserveUpDownCounter<u64>,

    /// Current number of idle threads in the blocking thread pool.
    #[cfg(tokio_unstable)]
    #[metric(unit = "{thread}")]
    pub blocking_thread_idle_count: ObserveUpDownCounter<u64>,

    /// Current number of tasks pending in worker-local queues, summed across all workers.
    #[cfg(tokio_unstable)]
    #[metric(unit = "{task}")]
    pub worker_local_queue_size: ObserveUpDownCounter<u64>,

    /// Total number of tasks spawned in the runtime since it was created.
    ///
    /// This counter is monotonically increasing. This metric requires `tokio_unstable` and
    /// 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{spawn}")]
    pub spawned_tasks_count: ObserveCounter<u64>,

    /// Total number of tasks scheduled from outside the runtime since it was created.
    ///
    /// These schedules go through the global injection queue and can be slower than worker-local scheduling.
    /// This metric requires `tokio_unstable` and 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{remote}")]
    pub remote_schedule_count: ObserveCounter<u64>,

    /// Total number of times tasks were forced to yield after exhausting their cooperative budget.
    ///
    /// This metric requires `tokio_unstable` and 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{yield}")]
    pub budget_forced_yield_count: ObserveCounter<u64>,

    /// Total number of "noop" unpark events, summed across workers, since runtime creation.
    ///
    /// A noop unpark is when a worker is unparked but finds no work before parking again.
    /// This metric requires `tokio_unstable` and 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{noop}")]
    pub worker_noop_count: ObserveCounter<u64>,

    /// Total number of successful steal operations, summed across workers, since runtime creation.
    ///
    /// This metric requires `tokio_unstable` and 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{steal}")]
    pub worker_steal_success_count: ObserveCounter<u64>,

    /// Total number of steal attempts, summed across workers, since runtime creation.
    ///
    /// This metric requires `tokio_unstable` and 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{steal}")]
    pub worker_steal_attempt_count: ObserveCounter<u64>,

    /// Total number of task polls, summed across workers, since runtime creation.
    ///
    /// This metric requires `tokio_unstable` and 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{poll}")]
    pub worker_poll_count: ObserveCounter<u64>,

    /// Total number of tasks scheduled onto worker-local queues, summed across workers, since runtime creation.
    ///
    /// This metric requires `tokio_unstable` and 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{schedule}")]
    pub worker_local_schedule_count: ObserveCounter<u64>,

    /// Total number of worker local-queue overflows, summed across workers, since runtime creation.
    ///
    /// This metric only applies to Tokio's multi-threaded scheduler. This metric requires
    /// `tokio_unstable` and 64-bit atomics.
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{overflow}")]
    pub worker_overflow_count: ObserveCounter<u64>,

    /// Total number of file descriptors registered with the runtime's I/O driver.
    ///
    /// This metric requires `tokio_unstable` and 64-bit atomics (and Tokio's `net` feature).
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{fd}")]
    pub io_driver_fd_registered_count: ObserveCounter<u64>,

    /// Total number of file descriptors deregistered from the runtime's I/O driver.
    ///
    /// This metric requires `tokio_unstable` and 64-bit atomics (and Tokio's `net` feature).
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{fd}")]
    pub io_driver_fd_deregistered_count: ObserveCounter<u64>,

    /// Total number of ready events processed by the runtime's I/O driver.
    ///
    /// This metric requires `tokio_unstable` and 64-bit atomics (and Tokio's `net` feature).
    #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
    #[metric(unit = "{ready}")]
    pub io_driver_ready_count: ObserveCounter<u64>,
}

/// Per-thread allocation counters.
pub(crate) struct PipelineMetricsMonitor {
    start_time: Instant,

    jemalloc_supported: bool,

    #[cfg(not(windows))]
    allocated: Option<ThreadLocal<u64>>,
    #[cfg(not(windows))]
    deallocated: Option<ThreadLocal<u64>>,
    last_allocated: u64,
    last_deallocated: u64,

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
    rusage_thread_supported: bool,

    // These timestamps mark the beginning of the current measurement interval
    wall_start: Instant,
    cpu_start: ThreadTime,

    metrics: MetricSet<PipelineMetrics>,
    tokio_rt: Option<tokio::runtime::RuntimeMetrics>,
    tokio_metrics: MetricSet<TokioRuntimeMetrics>,
}

impl PipelineMetricsMonitor {
    pub(crate) fn new(pipeline_ctx: PipelineContext) -> Self {
        let now = Instant::now();

        #[cfg(not(windows))]
        let (jemalloc_supported, allocated, deallocated, last_allocated, last_deallocated) = {
            // Try to initialize jemalloc thread-local stats. If the global allocator is not
            // jemalloc, these calls will fail and memory-related metrics will remain unchanged.
            let jemalloc_init = (|| {
                let alloc_mib = thread::allocatedp::mib().ok()?;
                let dealloc_mib = thread::deallocatedp::mib().ok()?;

                let allocated = alloc_mib.read().ok()?;
                let deallocated = dealloc_mib.read().ok()?;

                let last_allocated = allocated.get();
                let last_deallocated = deallocated.get();

                Some((allocated, deallocated, last_allocated, last_deallocated))
            })();

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
            }
        };

        #[cfg(windows)]
        let (jemalloc_supported, last_allocated, last_deallocated) = (false, 0, 0);

        #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
        let rusage_thread_supported = Self::init_rusage_baseline();
        let tokio_rt = tokio::runtime::Handle::try_current()
            .ok()
            .map(|handle| handle.metrics());

        Self {
            start_time: now,
            jemalloc_supported,
            #[cfg(not(windows))]
            allocated,
            #[cfg(not(windows))]
            deallocated,
            last_allocated,
            last_deallocated,
            #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
            rusage_thread_supported,
            wall_start: now,
            cpu_start: ThreadTime::now(),
            metrics: pipeline_ctx.register_metrics::<PipelineMetrics>(),
            tokio_rt,
            tokio_metrics: pipeline_ctx.register_metrics::<TokioRuntimeMetrics>(),
        }
    }

    /// Returns a mutable reference to the metrics struct.
    pub fn metrics_mut(&mut self) -> &mut MetricSet<PipelineMetrics> {
        &mut self.metrics
    }

    /// Returns a mutable reference to the Tokio runtime metrics struct.
    pub fn tokio_metrics_mut(&mut self) -> &mut MetricSet<TokioRuntimeMetrics> {
        &mut self.tokio_metrics
    }

    #[cfg(test)]
    pub fn update_metrics(&mut self) {
        self.update_tokio_metrics();
        self.update_pipeline_metrics();
    }

    /// Updates the per-pipeline (thread) internal metrics.
    ///
    /// These metrics include CPU usage and, where available, per-thread scheduling/page-fault
    /// signals and jemalloc-derived heap metrics.
    pub fn update_pipeline_metrics(&mut self) {
        // === Update thread memory allocation metrics (jemalloc only) ===
        #[cfg(not(windows))]
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

                self.metrics.memory_allocated.observe(cur_alloc);
                self.metrics.memory_freed.observe(cur_dealloc);
                self.metrics.memory_allocated_delta.add(delta_alloc);
                self.metrics.memory_freed_delta.add(delta_dealloc);
                self.metrics
                    .memory_usage
                    .observe(cur_alloc.saturating_sub(cur_dealloc));
            }
        }

        // === Update thread scheduling / page-fault metrics (when available) ===
        self.update_rusage_metrics();

        // === Update pipeline (thread) CPU usage metric ===
        let now_wall = Instant::now();
        let now_cpu = ThreadTime::now();
        let uptime = now_wall.duration_since(self.start_time).as_secs_f64();

        self.metrics.uptime.set(uptime);

        let wall_duration = now_wall.duration_since(self.wall_start);
        let cpu_duration = now_cpu.duration_since(self.cpu_start);

        let wall_micros = wall_duration.as_micros();

        self.metrics.cpu_time.add(cpu_duration.as_secs_f64());

        // Prevent division by zero if updates happen too fast
        if wall_micros > 0 {
            let cpu_micros = cpu_duration.as_micros();

            // Calculation: CPU Time / Wall Time, as a ratio.
            let usage_ratio = cpu_micros as f64 / wall_micros as f64;

            // Clamp to ensure we don't report > 1.0 due to timing skew.
            let usage_ratio = usage_ratio.clamp(0.0, 1.0);
            self.metrics.cpu_utilization.set(usage_ratio);
        } else {
            self.metrics.cpu_utilization.set(0.0);
        }

        // Reset time anchors for the next interval
        self.wall_start = now_wall;
        self.cpu_start = now_cpu;
    }

    /// Updates Tokio runtime metrics visible to the current thread.
    ///
    /// If the current thread is not within a Tokio runtime, this method is a no-op.
    pub fn update_tokio_metrics(&mut self) {
        if self.tokio_rt.is_none() {
            self.tokio_rt = tokio::runtime::Handle::try_current()
                .ok()
                .map(|handle| handle.metrics());
        }

        let Some(tokio_rt) = self.tokio_rt.as_ref() else {
            return;
        };

        let num_workers_usize = tokio_rt.num_workers();
        let num_workers = u64::try_from(num_workers_usize).unwrap_or(u64::MAX);
        let num_alive_tasks = u64::try_from(tokio_rt.num_alive_tasks()).unwrap_or(u64::MAX);
        let global_queue_depth = u64::try_from(tokio_rt.global_queue_depth()).unwrap_or(u64::MAX);

        self.tokio_metrics.worker_count.observe(num_workers);
        self.tokio_metrics
            .task_active_count
            .observe(num_alive_tasks);
        self.tokio_metrics
            .global_task_queue_size
            .observe(global_queue_depth);

        #[cfg(target_has_atomic = "64")]
        {
            let mut total_busy_s = 0.0f64;
            let mut total_park_count = 0u64;
            let mut total_park_unpark_count = 0u64;

            for worker in 0..num_workers_usize {
                total_busy_s += tokio_rt.worker_total_busy_duration(worker).as_secs_f64();
                total_park_count =
                    total_park_count.saturating_add(tokio_rt.worker_park_count(worker));
                total_park_unpark_count = total_park_unpark_count
                    .saturating_add(tokio_rt.worker_park_unpark_count(worker));
            }

            self.tokio_metrics.worker_busy_time.observe(total_busy_s);
            self.tokio_metrics
                .worker_park_count
                .observe(total_park_count);
            self.tokio_metrics
                .worker_park_unpark_count
                .observe(total_park_unpark_count);
        }

        #[cfg(tokio_unstable)]
        {
            let blocking_queue_depth =
                u64::try_from(tokio_rt.blocking_queue_depth()).unwrap_or(u64::MAX);
            let num_blocking_threads =
                u64::try_from(tokio_rt.num_blocking_threads()).unwrap_or(u64::MAX);
            let num_idle_blocking_threads =
                u64::try_from(tokio_rt.num_idle_blocking_threads()).unwrap_or(u64::MAX);

            let mut local_queue_depth_sum = 0u64;
            for worker in 0..num_workers_usize {
                local_queue_depth_sum = local_queue_depth_sum.saturating_add(
                    u64::try_from(tokio_rt.worker_local_queue_depth(worker)).unwrap_or(u64::MAX),
                );
            }

            self.tokio_metrics
                .blocking_task_queue_size
                .observe(blocking_queue_depth);
            self.tokio_metrics
                .blocking_thread_count
                .observe(num_blocking_threads);
            self.tokio_metrics
                .blocking_thread_idle_count
                .observe(num_idle_blocking_threads);
            self.tokio_metrics
                .worker_local_queue_size
                .observe(local_queue_depth_sum);
        }

        #[cfg(all(tokio_unstable, target_has_atomic = "64"))]
        {
            self.tokio_metrics
                .spawned_tasks_count
                .observe(tokio_rt.spawned_tasks_count());
            self.tokio_metrics
                .remote_schedule_count
                .observe(tokio_rt.remote_schedule_count());
            self.tokio_metrics
                .budget_forced_yield_count
                .observe(tokio_rt.budget_forced_yield_count());
            self.tokio_metrics
                .io_driver_fd_registered_count
                .observe(tokio_rt.io_driver_fd_registered_count());
            self.tokio_metrics
                .io_driver_fd_deregistered_count
                .observe(tokio_rt.io_driver_fd_deregistered_count());
            self.tokio_metrics
                .io_driver_ready_count
                .observe(tokio_rt.io_driver_ready_count());

            let mut worker_noop_sum = 0u64;
            let mut worker_steal_sum = 0u64;
            let mut worker_steal_ops_sum = 0u64;
            let mut worker_poll_sum = 0u64;
            let mut worker_local_schedule_sum = 0u64;
            let mut worker_overflow_sum = 0u64;

            for worker in 0..num_workers_usize {
                worker_noop_sum =
                    worker_noop_sum.saturating_add(tokio_rt.worker_noop_count(worker));
                worker_steal_sum =
                    worker_steal_sum.saturating_add(tokio_rt.worker_steal_count(worker));
                worker_steal_ops_sum =
                    worker_steal_ops_sum.saturating_add(tokio_rt.worker_steal_operations(worker));
                worker_poll_sum =
                    worker_poll_sum.saturating_add(tokio_rt.worker_poll_count(worker));
                worker_local_schedule_sum = worker_local_schedule_sum
                    .saturating_add(tokio_rt.worker_local_schedule_count(worker));
                worker_overflow_sum =
                    worker_overflow_sum.saturating_add(tokio_rt.worker_overflow_count(worker));
            }

            self.tokio_metrics
                .worker_noop_count
                .observe(worker_noop_sum);
            self.tokio_metrics
                .worker_steal_success_count
                .observe(worker_steal_sum);
            self.tokio_metrics
                .worker_steal_attempt_count
                .observe(worker_steal_ops_sum);
            self.tokio_metrics
                .worker_poll_count
                .observe(worker_poll_sum);
            self.tokio_metrics
                .worker_local_schedule_count
                .observe(worker_local_schedule_sum);
            self.tokio_metrics
                .worker_overflow_count
                .observe(worker_overflow_sum);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
    fn init_rusage_baseline() -> bool {
        #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
        {
            if let Ok(_usage) = getrusage(UsageWho::RUSAGE_THREAD) {
                return true;
            }
        }
        false
    }

    #[cfg(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd"))]
    fn update_rusage_metrics(&mut self) {
        if !self.rusage_thread_supported {
            return;
        }

        match getrusage(UsageWho::RUSAGE_THREAD) {
            Ok(usage) => {
                let voluntary =
                    u64::try_from(usage.voluntary_context_switches()).unwrap_or_default();
                let involuntary =
                    u64::try_from(usage.involuntary_context_switches()).unwrap_or_default();
                let minor_faults = u64::try_from(usage.minor_page_faults()).unwrap_or_default();
                let major_faults = u64::try_from(usage.major_page_faults()).unwrap_or_default();

                self.metrics.context_switches_voluntary.observe(voluntary);
                self.metrics
                    .context_switches_involuntary
                    .observe(involuntary);
                self.metrics.page_faults_minor.observe(minor_faults);
                self.metrics.page_faults_major.observe(major_faults);
            }
            Err(_) => {
                self.rusage_thread_supported = false;
            }
        }
    }

    #[cfg(not(any(target_os = "linux", target_os = "freebsd", target_os = "openbsd")))]
    fn update_rusage_metrics(&mut self) {}
}

#[cfg(all(test, not(windows), feature = "jemalloc-testing"))]
mod jemalloc_tests {
    use super::*;
    use crate::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    // Ensure jemalloc is the allocator for this crate's tests so that
    // tikv_jemalloc_ctl can read per-thread allocation counters.
    //
    // Run this test with:
    // `cargo test -p otap-df-engine --lib --features jemalloc-testing pipeline_metrics_monitor_black_box_updates_jemalloc`
    #[global_allocator]
    static GLOBAL: tikv_jemallocator::Jemalloc = tikv_jemallocator::Jemalloc;

    #[test]
    fn pipeline_metrics_monitor_black_box_updates_jemalloc() {
        let registry = MetricsRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);

        let mut monitor = PipelineMetricsMonitor::new(pipeline_ctx);

        // First update establishes baselines.
        monitor.update_metrics();
        let cpu0 = monitor.metrics.cpu_time.get();
        let cs_vol0 = monitor.metrics.context_switches_voluntary.get();
        let cs_invol0 = monitor.metrics.context_switches_involuntary.get();
        let pf_min0 = monitor.metrics.page_faults_minor.get();
        let pf_maj0 = monitor.metrics.page_faults_major.get();
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
        assert!(monitor.metrics.cpu_utilization.get() <= 1.0);
        assert!(monitor.metrics.memory_allocated.get() >= mem0);
        assert!(monitor.metrics.memory_allocated_delta.get() > 0);

        // Scheduling / page-fault metrics should be monotonic when supported.
        if monitor.rusage_thread_supported {
            assert!(monitor.metrics.context_switches_voluntary.get() >= cs_vol0);
            assert!(monitor.metrics.context_switches_involuntary.get() >= cs_invol0);
            assert!(monitor.metrics.page_faults_minor.get() >= pf_min0);
            assert!(monitor.metrics.page_faults_major.get() >= pf_maj0);
        } else {
            assert_eq!(monitor.metrics.context_switches_voluntary.get(), cs_vol0);
            assert_eq!(
                monitor.metrics.context_switches_involuntary.get(),
                cs_invol0
            );
            assert_eq!(monitor.metrics.page_faults_minor.get(), pf_min0);
            assert_eq!(monitor.metrics.page_faults_major.get(), pf_maj0);
        }
    }
}

#[cfg(all(test, not(feature = "jemalloc-testing")))]
mod non_jemalloc_tests {
    use super::*;
    use crate::context::ControllerContext;
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use std::hint::black_box;
    use std::time::{Duration, Instant};

    #[test]
    fn pipeline_metrics_monitor_does_not_update_memory_without_jemalloc() {
        let registry = MetricsRegistryHandle::new();
        let controller = ControllerContext::new(registry);
        let pipeline_ctx = controller.pipeline_context_with("grp".into(), "pipe".into(), 0, 0);

        let mut monitor = PipelineMetricsMonitor::new(pipeline_ctx);

        monitor.update_metrics();
        let cpu0 = monitor.metrics.cpu_time.get();
        let cs_vol0 = monitor.metrics.context_switches_voluntary.get();
        let cs_invol0 = monitor.metrics.context_switches_involuntary.get();
        let pf_min0 = monitor.metrics.page_faults_minor.get();
        let pf_maj0 = monitor.metrics.page_faults_major.get();

        // Allocate and burn CPU.
        let mut v = Vec::with_capacity(10_000);
        for i in 0..10_000u64 {
            v.push(i);
        }
        let _ = black_box(&v);

        let start = Instant::now();
        while start.elapsed() < Duration::from_millis(20) {
            let _ = black_box(1u64.wrapping_mul(2));
        }
        std::thread::sleep(Duration::from_millis(5));

        monitor.update_metrics();

        // CPU metrics should still update.
        assert!(monitor.metrics.cpu_time.get() >= cpu0);
        assert!(monitor.metrics.cpu_utilization.get() <= 1.0);

        // Scheduling / page fault metrics should be monotonic when supported.
        if monitor.rusage_thread_supported {
            assert!(monitor.metrics.context_switches_voluntary.get() >= cs_vol0);
            assert!(monitor.metrics.context_switches_involuntary.get() >= cs_invol0);
            assert!(monitor.metrics.page_faults_minor.get() >= pf_min0);
            assert!(monitor.metrics.page_faults_major.get() >= pf_maj0);
        } else {
            assert_eq!(monitor.metrics.context_switches_voluntary.get(), cs_vol0);
            assert_eq!(
                monitor.metrics.context_switches_involuntary.get(),
                cs_invol0
            );
            assert_eq!(monitor.metrics.page_faults_minor.get(), pf_min0);
            assert_eq!(monitor.metrics.page_faults_major.get(), pf_maj0);
        }

        // Memory-related metrics should remain unchanged (zero) without jemalloc.
        assert_eq!(monitor.metrics.memory_allocated.get(), 0);
        assert_eq!(monitor.metrics.memory_freed.get(), 0);
        assert_eq!(monitor.metrics.memory_usage.get(), 0);
        assert_eq!(monitor.metrics.memory_allocated_delta.get(), 0);
        assert_eq!(monitor.metrics.memory_freed_delta.get(), 0);
    }
}
