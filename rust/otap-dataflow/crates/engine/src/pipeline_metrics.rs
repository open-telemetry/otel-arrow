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
//! - `cpu_time` (`DeltaCounter<f64>`, `{s}`):
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
//! - `memory_allocated_delta` / `memory_freed_delta` (`DeltaCounter<u64>`, `{By}`; jemalloc only):
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
use otap_df_telemetry::instrument::{DeltaCounter, Gauge, ObserveCounter, ObserveUpDownCounter};
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
    pub memory_allocated_delta: DeltaCounter<u64>,

    /// Bytes returned to the heap by this pipeline during the last measurement window.
    #[metric(unit = "{By}")]
    pub memory_freed_delta: DeltaCounter<u64>,

    /// Total CPU seconds used by the pipeline since start.
    #[metric(unit = "{s}")]
    pub cpu_time: DeltaCounter<f64>,

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

    rusage_thread_supported: bool,

    // These timestamps mark the beginning of the current measurement interval
    wall_start: Instant,
    cpu_start: ThreadTime,

    metrics: MetricSet<PipelineMetrics>,
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

        let rusage_thread_supported = Self::init_rusage_baseline();

        Self {
            start_time: now,
            jemalloc_supported,
            #[cfg(not(windows))]
            allocated,
            #[cfg(not(windows))]
            deallocated,
            last_allocated,
            last_deallocated,
            rusage_thread_supported,
            wall_start: now,
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
