// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Exporter used to measure the performance of the OTAP data pipeline.
//!
//! ToDo – Future developments / improvements:
//! - Replace this exporter with a processor that could be combined with a Noop exporter to achieve
//!   the same functionality. The advantage would be to allow performance measurements anywhere in
//!   the pipeline.
//! - Measure the number of memory allocations for the current thread. This would allow measuring
//!   the memory used by the pipeline. This is possible using `mimalloc-rust-sys`.
//! - Measure per-thread CPU usage. This would allow measuring the pipeline’s CPU load. This is
//!   possible using the "libc" crate function `getrusage(RUSAGE\_THREAD)`.
//! - Measure network usage either via a cgroup or via eBPF.
//! - Measure per-thread perf counters (see crates perfcnt, perfcnt2, or direct perf\_event\_open
//!   via nix/libc). We could measure task-clock, context switches, page faults, ...
//! - Measure the latency of signals traversing the pipeline. This would require adding a timestamp
//!   in the headers of pdata messages.
//! - Support live reconfiguration via control message.

use crate::OTAP_EXPORTER_FACTORIES;
use crate::metrics::ExporterPDataMetrics;
use crate::pdata::OtapPdata;
use crate::perf_exporter::config::Config;
use crate::perf_exporter::metrics::PerfExporterPdataMetrics;
use async_trait::async_trait;
#[cfg(feature = "jemalloc-metrics")]
use jemalloc_ctl::thread::{allocatedp, deallocatedp};
#[cfg(target_os = "linux")]
use nix::sys::resource::{UsageWho, getrusage};
#[cfg(target_os = "linux")]
use nix::sys::time::TimeValLike;
#[cfg(target_os = "linux")]
use nix::unistd::{SysconfVar, gettid, sysconf};
use otap_df_config::SignalType;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::{AckMsg, NodeControlMsg};
use otap_df_engine::error::{Error, ExporterErrorKind};
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter as local;
use otap_df_engine::message::{Message, MessageChannel};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{ExporterFactory, distributed_slice};
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use serde_json::Value;
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::Instant;
use tokio::time::Duration;

/// The URN for the OTAP Perf exporter
pub const OTAP_PERF_EXPORTER_URN: &str = "urn:otel:otap:perf:exporter";

/// Perf Exporter that emits performance data
pub struct PerfExporter {
    config: Config,
    metrics: MetricSet<PerfExporterPdataMetrics>,
    pdata_metrics: MetricSet<ExporterPDataMetrics>,
    last_telemetry: Instant,
    thread_usage: ThreadUsageTracker,
    last_window: VecDeque<IntervalSample>,
}

#[derive(Clone, Copy)]
struct IntervalSample {
    elapsed_secs: f64,
    logs: u64,
    metrics: u64,
    spans: u64,
    end: Instant,
}

#[derive(Clone, Copy, Debug)]
struct ThreadUsageSample {
    user_cpu_ns: u64,
    system_cpu_ns: u64,
    rss_bytes: u64,
    minor_faults: u64,
    major_faults: u64,
    #[cfg(feature = "jemalloc-metrics")]
    jemalloc_allocated: u64,
    #[cfg(feature = "jemalloc-metrics")]
    jemalloc_deallocated: u64,
}

impl ThreadUsageSample {}

#[derive(Debug)]
struct ThreadUsageTracker {
    last: Option<ThreadUsageSample>,
    page_size: u64,
}

impl ThreadUsageTracker {
    fn new() -> Self {
        Self {
            last: None,
            page_size: detect_page_size(),
        }
    }

    fn record_delta(&mut self) -> Option<ThreadUsageDelta> {
        let current = capture_thread_usage(self.page_size)?;
        let previous = self.last.replace(current);
        Some(ThreadUsageDelta { previous, current })
    }
}

#[derive(Clone, Copy, Debug)]
struct ThreadUsageDelta {
    previous: Option<ThreadUsageSample>,
    current: ThreadUsageSample,
}

/// Declares the OTAP Perf exporter as a local exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static PERF_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: OTAP_PERF_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig| {
        Ok(ExporterWrapper::local(
            PerfExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
};

impl PerfExporter {
    /// creates a perf exporter with the provided config
    #[must_use]
    pub fn new(pipeline_ctx: PipelineContext, config: Config) -> Self {
        let metrics = pipeline_ctx.register_metrics::<PerfExporterPdataMetrics>();
        let pdata_metrics = pipeline_ctx.register_metrics::<ExporterPDataMetrics>();
        let last_telemetry = Instant::now();

        PerfExporter {
            config,
            metrics,
            pdata_metrics,
            last_telemetry,
            thread_usage: ThreadUsageTracker::new(),
            last_window: VecDeque::new(),
        }
    }

    /// Creates a new PerfExporter from a configuration object
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        Ok(PerfExporter::new(
            pipeline_ctx,
            serde_json::from_value(config.clone()).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: e.to_string(),
                }
            })?,
        ))
    }

    fn terminal_state(&self, deadline: Instant) -> TerminalState {
        let mut snapshots = Vec::new();

        if self.metrics.needs_flush() {
            snapshots.push(self.metrics.snapshot());
        }

        if self.pdata_metrics.needs_flush() {
            snapshots.push(self.pdata_metrics.snapshot());
        }

        TerminalState::new(deadline, snapshots)
    }
}

fn detect_page_size() -> u64 {
    #[cfg(target_os = "linux")]
    {
        sysconf(SysconfVar::PAGE_SIZE)
            .ok()
            .flatten()
            .and_then(|value| u64::try_from(value).ok())
            .filter(|value| *value > 0)
            .unwrap_or(4096)
    }
    #[cfg(not(target_os = "linux"))]
    {
        4096
    }
}

#[cfg(target_os = "linux")]
fn capture_thread_usage(page_size: u64) -> Option<ThreadUsageSample> {
    let usage = getrusage(UsageWho::RUSAGE_THREAD).ok()?;
    let user_cpu_ns = usage.user_time().num_nanoseconds().max(0) as u64;
    let system_cpu_ns = usage.system_time().num_nanoseconds().max(0) as u64;
    let rss_bytes = read_thread_rss_bytes(page_size).unwrap_or(0);
    let minor_faults = usage.minor_page_faults().max(0) as u64;
    let major_faults = usage.major_page_faults().max(0) as u64;
    #[cfg(feature = "jemalloc-metrics")]
    let jemalloc = capture_jemalloc_thread_alloc().unwrap_or((0, 0));

    Some(ThreadUsageSample {
        user_cpu_ns,
        system_cpu_ns,
        rss_bytes,
        minor_faults,
        major_faults,
        #[cfg(feature = "jemalloc-metrics")]
        jemalloc_allocated: jemalloc.0,
        #[cfg(feature = "jemalloc-metrics")]
        jemalloc_deallocated: jemalloc.1,
    })
}

#[cfg(not(target_os = "linux"))]
fn capture_thread_usage(_page_size: u64) -> Option<ThreadUsageSample> {
    None
}

#[cfg(feature = "jemalloc-metrics")]
#[allow(dead_code)]
fn capture_jemalloc_thread_alloc() -> Option<(u64, u64)> {
    let allocated = allocatedp::read().ok()?;
    let deallocated = deallocatedp::read().ok()?;
    Some((allocated.get(), deallocated.get()))
}

#[cfg(not(feature = "jemalloc-metrics"))]
#[allow(dead_code)]
fn capture_jemalloc_thread_alloc() -> Option<(u64, u64)> {
    None
}

#[cfg(target_os = "linux")]
fn read_thread_rss_bytes(page_size: u64) -> Option<u64> {
    let tid = gettid();
    let statm_path = format!("/proc/self/task/{}/statm", tid);
    let contents = std::fs::read_to_string(statm_path).ok()?;
    let mut parts = contents.split_whitespace();
    let _ = parts.next()?;
    let resident_pages: u64 = parts.next()?.parse().ok()?;

    Some(resident_pages.saturating_mul(page_size))
}

#[async_trait(?Send)]
impl local::Exporter<OtapPdata> for PerfExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: MessageChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // init variables for tracking
        // let mut average_pipeline_latency: f64 = 0.0;

        effect_handler.info("Starting Perf Exporter\n").await;

        // Start telemetry collection tick as a dedicated control message.
        let timer_cancel_handle = effect_handler
            .start_periodic_telemetry(Duration::from_millis(self.config.frequency()))
            .await?;

        // Loop until a Shutdown event is received.
        loop {
            let msg = msg_chan.recv().await?;
            match msg {
                Message::Control(NodeControlMsg::CollectTelemetry { metrics_reporter }) => {
                    let now = Instant::now();
                    let elapsed_secs = now
                        .duration_since(self.last_telemetry)
                        .as_secs_f64()
                        .max(1e-6);

                    if self.config.display_throughput() {
                        // Capture the last interval sample before clearing metrics.
                        self.last_window.push_back(IntervalSample {
                            elapsed_secs,
                            logs: self.metrics.logs.get(),
                            metrics: self.metrics.metrics.get(),
                            spans: self.metrics.spans.get(),
                            end: now,
                        });

                        // Keep only the last 10s worth of samples.
                        let window = Duration::from_secs(10);
                        while let Some(front) = self.last_window.front() {
                            if now.duration_since(front.end) > window {
                                _ = self.last_window.pop_front();
                            } else {
                                break;
                            }
                        }

                        let (total_logs, total_metrics, total_spans, total_secs) = self
                            .last_window
                            .iter()
                            .fold((0u64, 0u64, 0u64, 0f64), |(l, m, s, d), sample| {
                                (
                                    l + sample.logs,
                                    m + sample.metrics,
                                    s + sample.spans,
                                    d + sample.elapsed_secs,
                                )
                            });

                        let total_secs = total_secs.max(1e-6);
                        let logs_per_sec = total_logs as f64 / total_secs;
                        let metrics_per_sec = total_metrics as f64 / total_secs;
                        let spans_per_sec = total_spans as f64 / total_secs;

                        effect_handler
                            .info(&format!(
                                "PerfExporter throughput (avg last {:.2}s): logs {:.2}/s, metrics {:.2}/s, spans {:.2}/s",
                                total_secs, logs_per_sec, metrics_per_sec, spans_per_sec,
                            ))
                            .await;
                    }

                    if self.config.cpu_usage() || self.config.mem_usage() {
                        if let Some(delta) = self.thread_usage.record_delta() {
                            let mut status_segments = Vec::new();

                            if self.config.cpu_usage() {
                                if let Some(previous) = delta.previous {
                                    let cpu_user_delta = delta
                                        .current
                                        .user_cpu_ns
                                        .saturating_sub(previous.user_cpu_ns);
                                    let cpu_system_delta = delta
                                        .current
                                        .system_cpu_ns
                                        .saturating_sub(previous.system_cpu_ns);
                                    let cpu_total_delta =
                                        cpu_user_delta.saturating_add(cpu_system_delta);

                                    self.metrics.thread_cpu_user_ns.add(cpu_user_delta);
                                    self.metrics.thread_cpu_system_ns.add(cpu_system_delta);
                                    self.metrics.thread_cpu_total_ns.add(cpu_total_delta);

                                    let cpu_total_pct =
                                        (cpu_total_delta as f64 / (elapsed_secs * 1e9)) * 100.0;
                                    let cpu_user_pct =
                                        (cpu_user_delta as f64 / (elapsed_secs * 1e9)) * 100.0;
                                    let cpu_system_pct =
                                        (cpu_system_delta as f64 / (elapsed_secs * 1e9)) * 100.0;

                                    status_segments.push(format!(
                                        "CPU total {:.2}% (user {:.2}%, sys {:.2}%)",
                                        cpu_total_pct, cpu_user_pct, cpu_system_pct,
                                    ));
                                }
                            }

                            if self.config.mem_usage() {
                                let rss_bytes = delta.current.rss_bytes;
                                self.metrics.thread_rss_bytes.set(rss_bytes);

                                let mut rss_delta = 0;
                                let mut fault_bytes = 0;
                                #[cfg(feature = "jemalloc-metrics")]
                                let mut alloc_delta = 0;
                                #[cfg(feature = "jemalloc-metrics")]
                                let mut dealloc_delta = 0;

                                if let Some(previous) = delta.previous {
                                    rss_delta = rss_bytes.abs_diff(previous.rss_bytes);

                                    let minor_faults = delta
                                        .current
                                        .minor_faults
                                        .saturating_sub(previous.minor_faults);
                                    let major_faults = delta
                                        .current
                                        .major_faults
                                        .saturating_sub(previous.major_faults);

                                    let minor_fault_bytes =
                                        minor_faults.saturating_mul(self.thread_usage.page_size);
                                    let major_fault_bytes =
                                        major_faults.saturating_mul(self.thread_usage.page_size);
                                    fault_bytes =
                                        minor_fault_bytes.saturating_add(major_fault_bytes);

                                    self.metrics.thread_rss_delta_bytes.add(rss_delta);
                                    self.metrics.thread_minor_fault_bytes.add(minor_fault_bytes);
                                    self.metrics.thread_major_fault_bytes.add(major_fault_bytes);
                                    self.metrics.thread_fault_bytes.add(fault_bytes);

                                    #[cfg(feature = "jemalloc-metrics")]
                                    {
                                        alloc_delta = delta
                                            .current
                                            .jemalloc_allocated
                                            .saturating_sub(previous.jemalloc_allocated);
                                        dealloc_delta = delta
                                            .current
                                            .jemalloc_deallocated
                                            .saturating_sub(previous.jemalloc_deallocated);

                                        self.metrics.thread_alloc_bytes.add(alloc_delta);
                                        self.metrics.thread_dealloc_bytes.add(dealloc_delta);
                                    }
                                }

                                let rss_mib = rss_bytes as f64 / (1024.0 * 1024.0);
                                let rss_throughput_mib =
                                    rss_delta as f64 / (1024.0 * 1024.0) / elapsed_secs;
                                let fault_throughput_mib =
                                    fault_bytes as f64 / (1024.0 * 1024.0) / elapsed_secs;
                                #[cfg(feature = "jemalloc-metrics")]
                                let alloc_throughput_mib =
                                    alloc_delta as f64 / (1024.0 * 1024.0) / elapsed_secs;
                                #[cfg(feature = "jemalloc-metrics")]
                                let dealloc_throughput_mib =
                                    dealloc_delta as f64 / (1024.0 * 1024.0) / elapsed_secs;

                                #[cfg(feature = "jemalloc-metrics")]
                                status_segments.push(format!(
                                    "Memory {:.2} MiB RSS, {:.2} MiB/s faults, {:.2} MiB/s Δ (alloc {:.2} MiB/s, free {:.2} MiB/s, {} B RSS change, {} B fault bytes)",
                                    rss_mib,
                                    fault_throughput_mib,
                                    rss_throughput_mib,
                                    alloc_throughput_mib,
                                    dealloc_throughput_mib,
                                    rss_delta,
                                    fault_bytes,
                                ));
                                #[cfg(not(feature = "jemalloc-metrics"))]
                                status_segments.push(format!(
                                    "Memory {:.2} MiB RSS, {:.2} MiB/s faults, {:.2} MiB/s Δ ({} B RSS change, {} B fault bytes)",
                                    rss_mib, fault_throughput_mib, rss_throughput_mib, rss_delta, fault_bytes,
                                ));
                            }

                            if !status_segments.is_empty() {
                                effect_handler
                                    .info(&format!(
                                        "PerfExporter thread stats over {:.2}s: {}",
                                        elapsed_secs,
                                        status_segments.join(" | "),
                                    ))
                                    .await;
                            }
                        }
                    }

                    self.last_telemetry = now;

                    _ = metrics_reporter.report(&mut self.metrics);
                    _ = metrics_reporter.report(&mut self.pdata_metrics);
                }
                // ToDo: Handle configuration changes
                Message::Control(NodeControlMsg::Config { .. }) => {}
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    _ = timer_cancel_handle.cancel().await;
                    return Ok(self.terminal_state(deadline));
                }
                Message::PData(mut pdata) => {
                    // Capture signal type before moving pdata into try_from
                    let signal_type = pdata.signal_type();

                    // Increment consumed for this signal
                    self.pdata_metrics.inc_consumed(signal_type);

                    let payload = pdata.take_payload();
                    let _ = effect_handler.notify_ack(AckMsg::new(pdata)).await?;

                    let batch: OtapArrowRecords = match payload.try_into() {
                        Ok(batch) => batch,
                        Err(_) => {
                            self.pdata_metrics.inc_failed(signal_type);
                            continue;
                        }
                    };

                    // Increment counters per type of OTLP signals
                    match signal_type {
                        SignalType::Metrics => {
                            self.metrics.metrics.add(batch.batch_length() as u64);
                        }
                        SignalType::Logs => {
                            self.metrics.logs.add(batch.batch_length() as u64);
                        }
                        SignalType::Traces => {
                            self.metrics.spans.add(batch.batch_length() as u64);
                        }
                    }

                    // ToDo (LQ) We need to introduce pdata headers without hpack encoding for data coming from other nodes
                    // decode the headers which are hpack encoded
                    // check for timestamp
                    // get time delta between now and timestamp
                    // calculate average
                    // ToDo Temporary disable latency calculation until we have a better way to add the timestamp header
                    // let mut decoder = Decoder::new();
                    // let header_list =
                    //     decoder
                    //         .decode(&batch.headers)
                    //         .map_err(|_| Error::ExporterError {
                    //             exporter: effect_handler.exporter_id(),
                    //             error: "Failed to decode batch headers".to_owned(),
                    //         })?;
                    // find the timestamp header and parse it
                    // timestamp will be added in the receiver to enable pipeline latency calculation
                    // let timestamp_pair = header_list.iter().find(|(name, _)| name == b"timestamp");
                    // if let Some((_, value)) = timestamp_pair {
                    //     let timestamp =
                    //         decode_timestamp(value).map_err(|error| Error::ExporterError {
                    //             exporter: effect_handler.exporter_id(),
                    //             error,
                    //         })?;
                    //     let current_unix_time = SystemTime::now()
                    //         .duration_since(UNIX_EPOCH)
                    //         .map_err(|error| Error::ExporterError {
                    //             exporter: effect_handler.exporter_id(),
                    //             error: error.to_string(),
                    //         })?;
                    //     let latency = (current_unix_time - timestamp).as_secs_f64();
                    //     average_pipeline_latency = update_average(
                    //         latency,
                    //         average_pipeline_latency,
                    //         self.config.smoothing_factor() as f64,
                    //     );
                    // }

                    // Successful perf reporting: mark as exported for this signal
                    self.pdata_metrics.inc_exported(signal_type);

                    // ToDo Report disk, io, cpu, mem usage once gauge metrics are implemented
                }
                _ => {
                    return Err(Error::ExporterError {
                        exporter: effect_handler.exporter_id(),
                        kind: ExporterErrorKind::Other,
                        error: "Unknown control message".to_owned(),
                        source_detail: String::new(),
                    });
                }
            }
        }
    }
}

// uses the exponential moving average formula to update the average
// fn update_average(new_value: f64, old_average: f64, smoothing_factor: f64) -> f64 {
//     // update the average using a exponential moving average which allows new data points to have a greater impact depending on the smoothing factor
//     smoothing_factor * new_value + (1.0 - smoothing_factor) * old_average
// }

// decodes the byte array from the timestamp header and gets the equivalent duration value
// fn decode_timestamp(timestamp: &[u8]) -> Result<Duration, String> {
//     let timestamp_string = std::str::from_utf8(timestamp).map_err(|error| error.to_string())?;
//     let timestamp_parts: Vec<&str> = timestamp_string.split(":").collect();
//     let secs = timestamp_parts[0]
//         .parse::<u64>()
//         .map_err(|error| error.to_string())?;
//     let nanosecs = timestamp_parts[1]
//         .parse::<u32>()
//         .map_err(|error| error.to_string())?;
//
//     Ok(Duration::new(secs, nanosecs))
// }

#[cfg(test)]
mod tests {
    use crate::fixtures::{
        SimpleDataGenOptions, create_simple_logs_arrow_record_batches,
        create_simple_metrics_arrow_record_batches, create_simple_trace_arrow_record_batches,
    };
    use crate::pdata::OtapPdata;
    use crate::perf_exporter::config::Config;
    use crate::perf_exporter::exporter::{OTAP_PERF_EXPORTER_URN, PerfExporter};
    use otap_df_config::node::NodeUserConfig;
    use otap_df_engine::context::ControllerContext;
    use otap_df_engine::error::Error;
    use otap_df_engine::exporter::ExporterWrapper;
    use otap_df_engine::testing::exporter::TestContext;
    use otap_df_engine::testing::exporter::TestRuntime;
    use otap_df_engine::testing::test_node;
    use otap_df_pdata::Consumer;
    use otap_df_pdata::otap::{OtapArrowRecords, from_record_messages};
    use otap_df_telemetry::registry::MetricsRegistryHandle;
    use std::future::Future;
    use std::ops::Add;
    use std::sync::Arc;
    use std::time::Instant;
    use tokio::time::{Duration, sleep};

    /// Test closure that simulates a typical test scenario by sending timer ticks, config,
    /// data message, and shutdown control messages.
    ///
    fn scenario()
    -> impl FnOnce(TestContext<OtapPdata>) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |ctx| {
            Box::pin(async move {
                // send some messages to the exporter to calculate pipeline statistics
                for i in 0..3 {
                    let mut traces_batch_data =
                        create_simple_trace_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i,
                            num_rows: 5,
                            ..Default::default()
                        });
                    let mut logs_batch_data =
                        create_simple_logs_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i + 1,
                            num_rows: 5,
                            ..Default::default()
                        });
                    let mut metrics_batch_data =
                        create_simple_metrics_arrow_record_batches(SimpleDataGenOptions {
                            id_offset: 3 * i + 2,
                            num_rows: 5,
                            ..Default::default()
                        });

                    let trace_batch_data = from_record_messages(
                        Consumer::default()
                            .consume_bar(&mut traces_batch_data)
                            .unwrap(),
                    );
                    let logs_batch_data = from_record_messages(
                        Consumer::default()
                            .consume_bar(&mut logs_batch_data)
                            .unwrap(),
                    );
                    let metrics_batch_data = from_record_messages(
                        Consumer::default()
                            .consume_bar(&mut metrics_batch_data)
                            .unwrap(),
                    );

                    // Send a data message
                    ctx.send_pdata(OtapPdata::new_default(
                        OtapArrowRecords::Traces(trace_batch_data).into(),
                    ))
                    .await
                    .expect("Failed to send data message");
                    ctx.send_pdata(OtapPdata::new_default(
                        OtapArrowRecords::Logs(logs_batch_data).into(),
                    ))
                    .await
                    .expect("Failed to send data message");
                    ctx.send_pdata(OtapPdata::new_default(
                        OtapArrowRecords::Metrics(metrics_batch_data).into(),
                    ))
                    .await
                    .expect("Failed to send data message");
                }

                // TODO ADD DELAY BETWEEN HERE
                _ = sleep(Duration::from_millis(5000));

                // Send shutdown
                ctx.send_shutdown(
                    Instant::now().add(Duration::from_millis(200)),
                    "test complete",
                )
                .await
                .expect("Failed to send Shutdown");
            })
        }
    }

    /// Validation closure that checks the expected counter values
    fn validation_procedure(
        metrics_registry_handle: MetricsRegistryHandle,
    ) -> impl FnOnce(
        TestContext<OtapPdata>,
        Result<(), Error>,
    ) -> std::pin::Pin<Box<dyn Future<Output = ()>>> {
        |_, exporter_result| {
            Box::pin(async move {
                assert!(exporter_result.is_ok());

                metrics_registry_handle.visit_current_metrics(
                    |_metrics_descriptor, _attrs, _metric_values| {
                        // ToDo Check the counters, once the timer tick control message is implemented in the test infrastructure.
                    },
                );
            })
        }
    }

    #[test]
    fn test_exporter_local() {
        let test_runtime = TestRuntime::new();
        let config = Config::new(1000, 0.3, true, true, true, true, true, false);
        let node_config = Arc::new(NodeUserConfig::new_exporter_config(OTAP_PERF_EXPORTER_URN));
        let metrics_registry_handle = MetricsRegistryHandle::new();
        let controller_ctx = ControllerContext::new(metrics_registry_handle.clone());
        let pipeline_ctx =
            controller_ctx.pipeline_context_with("grp".into(), "pipeline".into(), 0, 0);
        let exporter = ExporterWrapper::local(
            PerfExporter::new(pipeline_ctx, config),
            test_node(test_runtime.config().name.clone()),
            node_config,
            test_runtime.config(),
        );

        test_runtime
            .set_exporter(exporter)
            .run_test(scenario())
            .run_validation(validation_procedure(metrics_registry_handle));
    }
}
