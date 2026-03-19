// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-scoped telemetry for the shared engine control plane.
//!
//! These metrics complement channel endpoint and per-node produced/consumed
//! metrics by instrumenting the two shared runtime actors that are otherwise
//! hard to observe from node-local telemetry alone:
//!
//! - `RuntimeCtrlMsgManager`
//! - `PipelineCompletionMsgDispatcher`
//!
//! The actor keeps raw runtime state locally and materializes snapshots on
//! meaningful transitions. This lets the engine export zero-valued gauge
//! transitions such as `pending_sends.buffered = 0` after a backlog drains,
//! which would otherwise be lost if the engine only flushed non-zero metric
//! sets.

use crate::context::PipelineContext;
use otap_df_config::MetricLevel;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::instrument::{Counter, Gauge, Mmsc};
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
#[cfg(test)]
use otap_df_telemetry::registry::MetricSetKey;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::{MetricsReporter, ReportOutcome};
use otap_df_telemetry_macros::metric_set;
use std::fmt::Debug;
use std::time::Instant;

/// Pipeline-scoped metrics for the shared runtime-control actor.
///
/// This set describes shutdown/drain progress, pending control backlogs, and
/// the state and throughput of the runtime-managed timer and delayed-data
/// machinery. It is the primary metric set for understanding whether the
/// pipeline runtime is still making progress under control-plane load and where
/// graceful shutdown is spending time.
#[metric_set(name = "pipeline.runtime_control")]
#[derive(Debug, Default, Clone)]
pub(crate) struct RuntimeControlMetrics {
    /// Whether the runtime is currently in graceful drain mode.
    ///
    /// This is the top-level liveness phase signal for shutdown analysis: `1`
    /// means ingress has been stopped and the runtime is trying to converge to
    /// quiescence, `0` means normal operation.
    ///
    /// Level: `basic`.
    #[metric(name = "drain.active", unit = "{1}")]
    pub drain_active: Gauge<u64>,
    /// Number of receivers that still need to report `ReceiverDrained`.
    ///
    /// This shows whether shutdown is still blocked in the receiver-first drain
    /// phase and helps identify a stuck or slow receiver delaying downstream
    /// shutdown.
    ///
    /// Level: `basic`.
    #[metric(name = "drain.pending_receivers", unit = "{node}")]
    pub drain_pending_receivers: Gauge<u64>,
    /// Number of node-control deliveries currently buffered for later retry.
    ///
    /// A sustained non-zero value indicates the runtime manager is no longer
    /// delivering control work immediately and is absorbing backpressure from
    /// full node-control inboxes instead of blocking the runtime.
    ///
    /// Level: `basic`.
    #[metric(name = "pending_sends.buffered", unit = "{message}")]
    pub pending_sends_buffered: Gauge<u64>,
    /// Number of ordinary per-node timers currently registered.
    ///
    /// This is useful for understanding timer pressure and for spotting leaks
    /// where timers are started but not canceled or consumed as expected.
    ///
    /// Level: `basic`.
    #[metric(name = "timers.active", unit = "{timer}")]
    pub timers_active: Gauge<u64>,
    /// Number of telemetry collection timers currently registered.
    ///
    /// This distinguishes telemetry scheduling pressure from normal timer
    /// traffic and helps explain unexpected `CollectTelemetry` activity.
    ///
    /// Level: `basic`.
    #[metric(name = "telemetry_timers.active", unit = "{timer}")]
    pub telemetry_timers_active: Gauge<u64>,
    /// Number of delayed-data wakeups currently queued in the runtime manager.
    ///
    /// This is a key liveness signal for processors that depend on delayed
    /// retries or one-shot wakeups: rising backlog means more admitted work is
    /// waiting on the shared runtime control path.
    ///
    /// Level: `basic`.
    #[metric(name = "delayed_data.queued", unit = "{message}")]
    pub delayed_data_queued: Gauge<u64>,
    /// Count of `RuntimeControlMsg::Shutdown` requests accepted by this runtime.
    ///
    /// This marks the start of the runtime-managed drain sequence and is the
    /// main counter used to correlate later drain-phase transitions.
    ///
    /// Level: `normal`.
    #[metric(name = "shutdown.received", unit = "{message}")]
    pub shutdown_received: Counter<u64>,
    /// Count of `DrainIngress` fan-out operations sent to receivers.
    ///
    /// This confirms that shutdown moved from “requested” to “stop admitting
    /// new ingress”, which is the first active liveness step of graceful drain.
    ///
    /// Level: `normal`.
    #[metric(name = "drain_ingress.sent", unit = "{message}")]
    pub drain_ingress_sent: Counter<u64>,
    /// Count of `ReceiverDrained` notifications received from receivers.
    ///
    /// This shows actual receiver-side drain progress and helps distinguish “no
    /// work admitted anymore” from “receivers have really finished draining”.
    ///
    /// Level: `normal`.
    #[metric(name = "receiver_drained.received", unit = "{message}")]
    pub receiver_drained_received: Counter<u64>,
    /// Count of downstream `Shutdown` fan-out operations to non-receivers.
    ///
    /// This marks the handoff from receiver draining to downstream convergence,
    /// which is useful when debugging where shutdown latency is spent.
    ///
    /// Level: `normal`.
    #[metric(name = "downstream_shutdown.sent", unit = "{message}")]
    pub downstream_shutdown_sent: Counter<u64>,
    /// Count of shutdowns that reached the deadline before natural completion.
    ///
    /// Any non-zero value here is a direct signal that graceful drain did not
    /// complete in time and that liveness relied on the shutdown deadline.
    ///
    /// Level: `normal`.
    #[metric(name = "shutdown.deadline_forced", unit = "{message}")]
    pub shutdown_deadline_forced: Counter<u64>,
    /// Count of `StartTimer` control messages received by the runtime manager.
    ///
    /// This helps quantify timer creation pressure and can explain growth in
    /// `timers.active` or bursts of later `timer_tick.sent` activity.
    ///
    /// Level: `normal`.
    #[metric(name = "start_timer.received", unit = "{message}")]
    pub start_timer_received: Counter<u64>,
    /// Count of `CancelTimer` control messages received by the runtime manager.
    ///
    /// Comparing this with timer starts helps diagnose timer churn, leaks, or
    /// mismatches between node scheduling and node cleanup behavior.
    ///
    /// Level: `normal`.
    #[metric(name = "cancel_timer.received", unit = "{message}")]
    pub cancel_timer_received: Counter<u64>,
    /// Count of telemetry timer start requests received by the runtime manager.
    ///
    /// This shows how much of the runtime-control stream is driven by metrics
    /// collection scheduling rather than ordinary data-plane coordination.
    ///
    /// Level: `normal`.
    #[metric(name = "start_telemetry_timer.received", unit = "{message}")]
    pub start_telemetry_timer_received: Counter<u64>,
    /// Count of telemetry timer cancel requests received by the runtime manager.
    ///
    /// This is mainly useful for checking that telemetry scheduling remains
    /// bounded and does not accumulate stale per-node timers.
    ///
    /// Level: `normal`.
    #[metric(name = "cancel_telemetry_timer.received", unit = "{message}")]
    pub cancel_telemetry_timer_received: Counter<u64>,
    /// Count of `DelayData` requests received by the runtime manager.
    ///
    /// This measures how much work is being converted into deferred wakeups, a
    /// common cause of control-plane load for retry and batching patterns.
    ///
    /// Level: `normal`.
    #[metric(name = "delay_data.received", unit = "{message}")]
    pub delay_data_received: Counter<u64>,
    /// Count of delayed-data items returned immediately because drain is active.
    ///
    /// This is important for shutdown liveness: it shows the runtime is
    /// flushing scheduled work back to origin nodes instead of leaving it
    /// stranded behind future wakeup times.
    ///
    /// Level: `normal`.
    #[metric(name = "delay_data.returned_during_drain", unit = "{message}")]
    pub delay_data_returned_during_drain: Counter<u64>,
    /// Count of `TimerTick` control messages sent to nodes.
    ///
    /// This confirms that due timers are actually being delivered and helps
    /// detect starvation where timers accumulate but ticks do not make progress.
    ///
    /// Level: `normal`.
    #[metric(name = "timer_tick.sent", unit = "{message}")]
    pub timer_tick_sent: Counter<u64>,
    /// Count of `CollectTelemetry` control messages sent to nodes.
    ///
    /// This helps confirm telemetry timers are progressing under load and that
    /// observability work is not being starved by other runtime-control traffic.
    ///
    /// Level: `normal`.
    #[metric(name = "collect_telemetry.sent", unit = "{message}")]
    pub collect_telemetry_sent: Counter<u64>,
    /// Count of delayed-data wakeups sent back to origin nodes.
    ///
    /// This is the completion-side counterpart to `delay_data.received` and is
    /// a direct indicator that deferred work is resuming rather than stalling.
    ///
    /// Level: `normal`.
    #[metric(name = "delayed_data.sent", unit = "{message}")]
    pub delayed_data_sent: Counter<u64>,
    /// Distribution of time spent waiting for receivers to finish draining.
    ///
    /// This isolates the receiver-gated phase of shutdown and is useful for
    /// determining whether slow graceful shutdown is ingress-bound or further
    /// downstream in the pipeline.
    ///
    /// Level: `detailed`.
    #[metric(name = "drain.receiver_phase_duration_ns", unit = "ns")]
    pub drain_receiver_phase_duration_ns: Mmsc,
    /// Distribution of total graceful shutdown duration for the runtime.
    ///
    /// This is the top-level latency metric for shutdown liveness and shows how
    /// long the runtime took to move from shutdown acceptance to completion or
    /// forced deadline exit.
    ///
    /// Level: `detailed`.
    #[metric(name = "drain.total_duration_ns", unit = "ns")]
    pub drain_total_duration_ns: Mmsc,
}

/// Pipeline-scoped metrics for the shared completion dispatcher.
///
/// This set describes how Ack/Nack completion traffic enters the dispatcher,
/// whether it is successfully delivered to interested upstream nodes, whether
/// completions are dropped because no upstream frame subscribed, and how much
/// unwind work is required per completion. It is the primary metric set for
/// understanding completion-path load, routing behavior, and unwind liveness.
#[metric_set(name = "pipeline.completion")]
#[derive(Debug, Default, Clone)]
pub(crate) struct PipelineCompletionMetrics {
    /// Number of completion deliveries currently buffered for later retry.
    ///
    /// A sustained backlog here means the completion dispatcher is making
    /// progress by deferring sends to full node-control inboxes instead of
    /// blocking, but it also indicates unwind traffic is under pressure.
    ///
    /// Level: `basic`.
    #[metric(name = "pending_sends.buffered", unit = "{message}")]
    pub pending_sends_buffered: Gauge<u64>,
    /// Count of `DeliverAck` messages received by the completion dispatcher.
    ///
    /// This shows inbound successful completion pressure on the shared
    /// completion path and helps explain downstream-to-upstream unwind load.
    ///
    /// Level: `normal`.
    #[metric(name = "deliver_ack.received", unit = "{message}")]
    pub deliver_ack_received: Counter<u64>,
    /// Count of `DeliverNack` messages received by the completion dispatcher.
    ///
    /// This is the failure-side counterpart to received acks and is especially
    /// useful when investigating retry storms or elevated downstream refusal
    /// rates.
    ///
    /// Level: `normal`.
    #[metric(name = "deliver_nack.received", unit = "{message}")]
    pub deliver_nack_received: Counter<u64>,
    /// Count of Ack control messages successfully sent to interested upstream nodes.
    ///
    /// This confirms that successful completions are not just arriving at the
    /// dispatcher but are actually making progress toward the closest interested
    /// upstream frame.
    ///
    /// Level: `normal`.
    #[metric(name = "ack.sent", unit = "{message}")]
    pub ack_sent: Counter<u64>,
    /// Count of Nack control messages successfully sent to interested upstream nodes.
    ///
    /// This shows refusal propagation progress through the completion path and
    /// is important when verifying that transient or permanent failures are not
    /// getting stranded under load.
    ///
    /// Level: `normal`.
    #[metric(name = "nack.sent", unit = "{message}")]
    pub nack_sent: Counter<u64>,
    /// Count of Ack completions dropped because no upstream frame was interested.
    ///
    /// This distinguishes "successful unwind with no listener" from send
    /// failure or dispatcher backlog, which is useful when validating routing
    /// expectations and subscription placement.
    ///
    /// Level: `normal`.
    #[metric(name = "ack.dropped_no_interest", unit = "{message}")]
    pub ack_dropped_no_interest: Counter<u64>,
    /// Count of Nack completions dropped because no upstream frame was interested.
    ///
    /// This is particularly important for liveness and correctness analysis:
    /// it means failure information reached the dispatcher but no upstream frame
    /// subscribed to receive it.
    ///
    /// Level: `normal`.
    #[metric(name = "nack.dropped_no_interest", unit = "{message}")]
    pub nack_dropped_no_interest: Counter<u64>,
    /// Distribution of unwind stack depth consumed per completion outcome.
    ///
    /// Larger values indicate completions are traversing more frames before
    /// finding an interested upstream node or concluding that none exists,
    /// which helps explain completion-path cost and routing behavior.
    ///
    /// Level: `detailed`.
    #[metric(name = "unwind.depth", unit = "{frame}")]
    pub unwind_depth: Mmsc,
}

struct RegisteredMetricSet<M: MetricSetHandler + Default + Debug + Send + Sync> {
    metrics: MetricSet<M>,
    registry: TelemetryRegistryHandle,
}

impl<M: MetricSetHandler + Default + Debug + Send + Sync> RegisteredMetricSet<M> {
    fn new(pipeline_ctx: &PipelineContext) -> Self {
        let entity_key = crate::entity_context::pipeline_entity_key().expect(
            "pipeline entity key not set; ensure pipeline entity is registered and instrumented",
        );
        Self {
            metrics: pipeline_ctx.register_metric_set_for_entity::<M>(entity_key),
            registry: pipeline_ctx.metrics_registry(),
        }
    }

    #[cfg(test)]
    fn metric_set_key(&self) -> MetricSetKey {
        self.metrics.metric_set_key()
    }
}

impl<M: MetricSetHandler + Default + Debug + Send + Sync> Drop for RegisteredMetricSet<M> {
    fn drop(&mut self) {
        let _ = self
            .registry
            .unregister_metric_set(self.metrics.metric_set_key());
    }
}

pub(crate) struct RuntimeControlMetricsState {
    level: MetricLevel,
    reporter: MetricsReporter,
    metrics: Option<RegisteredMetricSet<RuntimeControlMetrics>>,
    dirty: bool,
    drain_active: bool,
    drain_pending_receivers: u64,
    pending_sends_buffered: u64,
    timers_active: u64,
    telemetry_timers_active: u64,
    delayed_data_queued: u64,
    shutdown_received: u64,
    drain_ingress_sent: u64,
    receiver_drained_received: u64,
    downstream_shutdown_sent: u64,
    shutdown_deadline_forced: u64,
    start_timer_received: u64,
    cancel_timer_received: u64,
    start_telemetry_timer_received: u64,
    cancel_telemetry_timer_received: u64,
    delay_data_received: u64,
    delay_data_returned_during_drain: u64,
    timer_tick_sent: u64,
    collect_telemetry_sent: u64,
    delayed_data_sent: u64,
    drain_receiver_phase_duration_ns: Mmsc,
    drain_total_duration_ns: Mmsc,
    shutdown_started_at: Option<Instant>,
    receiver_phase_started_at: Option<Instant>,
}

impl RuntimeControlMetricsState {
    pub(crate) fn new(
        pipeline_ctx: &PipelineContext,
        reporter: MetricsReporter,
        level: MetricLevel,
        timers_active: usize,
        telemetry_timers_active: usize,
        delayed_data_queued: usize,
    ) -> Self {
        Self {
            level,
            reporter,
            metrics: (level != MetricLevel::None).then(|| RegisteredMetricSet::new(pipeline_ctx)),
            dirty: level != MetricLevel::None,
            drain_active: false,
            drain_pending_receivers: 0,
            pending_sends_buffered: 0,
            timers_active: timers_active as u64,
            telemetry_timers_active: telemetry_timers_active as u64,
            delayed_data_queued: delayed_data_queued as u64,
            shutdown_received: 0,
            drain_ingress_sent: 0,
            receiver_drained_received: 0,
            downstream_shutdown_sent: 0,
            shutdown_deadline_forced: 0,
            start_timer_received: 0,
            cancel_timer_received: 0,
            start_telemetry_timer_received: 0,
            cancel_telemetry_timer_received: 0,
            delay_data_received: 0,
            delay_data_returned_during_drain: 0,
            timer_tick_sent: 0,
            collect_telemetry_sent: 0,
            delayed_data_sent: 0,
            drain_receiver_phase_duration_ns: Mmsc::default(),
            drain_total_duration_ns: Mmsc::default(),
            shutdown_started_at: None,
            receiver_phase_started_at: None,
        }
    }

    #[cfg(test)]
    pub(crate) fn metric_set_key(&self) -> Option<MetricSetKey> {
        self.metrics
            .as_ref()
            .map(RegisteredMetricSet::metric_set_key)
    }

    pub(crate) fn set_pending_sends_buffered(&mut self, buffered: usize) {
        let buffered = buffered as u64;
        if self.pending_sends_buffered != buffered {
            self.pending_sends_buffered = buffered;
            self.dirty = true;
        }
    }

    pub(crate) fn set_timer_counts(
        &mut self,
        timers_active: usize,
        telemetry_timers_active: usize,
    ) {
        let timers_active = timers_active as u64;
        let telemetry_timers_active = telemetry_timers_active as u64;
        if self.timers_active != timers_active {
            self.timers_active = timers_active;
            self.dirty = true;
        }
        if self.telemetry_timers_active != telemetry_timers_active {
            self.telemetry_timers_active = telemetry_timers_active;
            self.dirty = true;
        }
    }

    pub(crate) fn set_delayed_data_queued(&mut self, queued: usize) {
        let queued = queued as u64;
        if self.delayed_data_queued != queued {
            self.delayed_data_queued = queued;
            self.dirty = true;
        }
    }

    pub(crate) fn record_shutdown_received(&mut self, now: Instant, pending_receivers: usize) {
        self.drain_active = true;
        self.drain_pending_receivers = pending_receivers as u64;
        self.shutdown_started_at = Some(now);
        self.receiver_phase_started_at = (pending_receivers > 0).then_some(now);
        if self.level >= MetricLevel::Normal {
            self.shutdown_received += 1;
        }
        self.dirty = true;
    }

    pub(crate) fn record_drain_ingress_sent(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.drain_ingress_sent += 1;
        }
        self.dirty = true;
    }

    pub(crate) fn record_receiver_drained(&mut self, now: Instant, pending_receivers: usize) {
        self.drain_pending_receivers = pending_receivers as u64;
        if self.level >= MetricLevel::Normal {
            self.receiver_drained_received += 1;
        }
        if pending_receivers == 0 {
            if self.level >= MetricLevel::Detailed {
                if let Some(started_at) = self.receiver_phase_started_at.take() {
                    self.drain_receiver_phase_duration_ns
                        .record(now.duration_since(started_at).as_nanos() as f64);
                }
            } else {
                self.receiver_phase_started_at = None;
            }
        }
        self.dirty = true;
    }

    pub(crate) fn record_downstream_shutdown_sent(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.downstream_shutdown_sent += 1;
        }
        self.dirty = true;
    }

    pub(crate) fn record_shutdown_deadline_forced(&mut self, now: Instant) {
        if self.level >= MetricLevel::Normal {
            self.shutdown_deadline_forced += 1;
        }
        if self.level >= MetricLevel::Detailed {
            if let Some(started_at) = self.shutdown_started_at.take() {
                self.drain_total_duration_ns
                    .record(now.duration_since(started_at).as_nanos() as f64);
            }
        } else {
            self.shutdown_started_at = None;
        }
        self.dirty = true;
    }

    pub(crate) fn record_shutdown_completed(&mut self, now: Instant) {
        if self.level >= MetricLevel::Detailed {
            if let Some(started_at) = self.shutdown_started_at.take() {
                self.drain_total_duration_ns
                    .record(now.duration_since(started_at).as_nanos() as f64);
                self.dirty = true;
            }
        } else {
            self.shutdown_started_at = None;
        }
    }

    pub(crate) fn record_start_timer_received(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.start_timer_received += 1;
            self.dirty = true;
        }
    }

    pub(crate) fn record_cancel_timer_received(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.cancel_timer_received += 1;
            self.dirty = true;
        }
    }

    pub(crate) fn record_start_telemetry_timer_received(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.start_telemetry_timer_received += 1;
            self.dirty = true;
        }
    }

    pub(crate) fn record_cancel_telemetry_timer_received(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.cancel_telemetry_timer_received += 1;
            self.dirty = true;
        }
    }

    pub(crate) fn record_delay_data_received(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.delay_data_received += 1;
            self.dirty = true;
        }
    }

    pub(crate) fn record_delay_data_returned_during_drain(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.delay_data_returned_during_drain += 1;
            self.dirty = true;
        }
    }

    pub(crate) fn record_due_events(
        &mut self,
        timer_ticks: usize,
        collect_telemetry: usize,
        delayed_data_sent: usize,
    ) {
        if self.level >= MetricLevel::Normal {
            self.timer_tick_sent += timer_ticks as u64;
            self.collect_telemetry_sent += collect_telemetry as u64;
            self.delayed_data_sent += delayed_data_sent as u64;
            if timer_ticks > 0 || collect_telemetry > 0 || delayed_data_sent > 0 {
                self.dirty = true;
            }
        }
    }

    pub(crate) fn report_if_needed(&mut self) -> Result<(), TelemetryError> {
        if !self.dirty {
            return Ok(());
        }
        let Some(registered) = self.metrics.as_mut() else {
            self.dirty = false;
            return Ok(());
        };

        registered
            .metrics
            .drain_active
            .set(u64::from(self.drain_active));
        registered
            .metrics
            .drain_pending_receivers
            .set(self.drain_pending_receivers);
        registered
            .metrics
            .pending_sends_buffered
            .set(self.pending_sends_buffered);
        registered.metrics.timers_active.set(self.timers_active);
        registered
            .metrics
            .telemetry_timers_active
            .set(self.telemetry_timers_active);
        registered
            .metrics
            .delayed_data_queued
            .set(self.delayed_data_queued);

        if self.level >= MetricLevel::Normal {
            registered.metrics.shutdown_received = self.shutdown_received.into();
            registered.metrics.drain_ingress_sent = self.drain_ingress_sent.into();
            registered.metrics.receiver_drained_received = self.receiver_drained_received.into();
            registered.metrics.downstream_shutdown_sent = self.downstream_shutdown_sent.into();
            registered.metrics.shutdown_deadline_forced = self.shutdown_deadline_forced.into();
            registered.metrics.start_timer_received = self.start_timer_received.into();
            registered.metrics.cancel_timer_received = self.cancel_timer_received.into();
            registered.metrics.start_telemetry_timer_received =
                self.start_telemetry_timer_received.into();
            registered.metrics.cancel_telemetry_timer_received =
                self.cancel_telemetry_timer_received.into();
            registered.metrics.delay_data_received = self.delay_data_received.into();
            registered.metrics.delay_data_returned_during_drain =
                self.delay_data_returned_during_drain.into();
            registered.metrics.timer_tick_sent = self.timer_tick_sent.into();
            registered.metrics.collect_telemetry_sent = self.collect_telemetry_sent.into();
            registered.metrics.delayed_data_sent = self.delayed_data_sent.into();
        } else {
            registered.metrics.shutdown_received = Counter::default();
            registered.metrics.drain_ingress_sent = Counter::default();
            registered.metrics.receiver_drained_received = Counter::default();
            registered.metrics.downstream_shutdown_sent = Counter::default();
            registered.metrics.shutdown_deadline_forced = Counter::default();
            registered.metrics.start_timer_received = Counter::default();
            registered.metrics.cancel_timer_received = Counter::default();
            registered.metrics.start_telemetry_timer_received = Counter::default();
            registered.metrics.cancel_telemetry_timer_received = Counter::default();
            registered.metrics.delay_data_received = Counter::default();
            registered.metrics.delay_data_returned_during_drain = Counter::default();
            registered.metrics.timer_tick_sent = Counter::default();
            registered.metrics.collect_telemetry_sent = Counter::default();
            registered.metrics.delayed_data_sent = Counter::default();
        }

        if self.level >= MetricLevel::Detailed {
            registered.metrics.drain_receiver_phase_duration_ns =
                self.drain_receiver_phase_duration_ns;
            registered.metrics.drain_total_duration_ns = self.drain_total_duration_ns;
        } else {
            registered.metrics.drain_receiver_phase_duration_ns = Mmsc::default();
            registered.metrics.drain_total_duration_ns = Mmsc::default();
        }

        match self
            .reporter
            .try_report_snapshot_with_outcome(registered.metrics.snapshot())?
        {
            ReportOutcome::Sent => {
                if self.level >= MetricLevel::Normal {
                    self.shutdown_received = 0;
                    self.drain_ingress_sent = 0;
                    self.receiver_drained_received = 0;
                    self.downstream_shutdown_sent = 0;
                    self.shutdown_deadline_forced = 0;
                    self.start_timer_received = 0;
                    self.cancel_timer_received = 0;
                    self.start_telemetry_timer_received = 0;
                    self.cancel_telemetry_timer_received = 0;
                    self.delay_data_received = 0;
                    self.delay_data_returned_during_drain = 0;
                    self.timer_tick_sent = 0;
                    self.collect_telemetry_sent = 0;
                    self.delayed_data_sent = 0;
                }
                if self.level >= MetricLevel::Detailed {
                    self.drain_receiver_phase_duration_ns = Mmsc::default();
                    self.drain_total_duration_ns = Mmsc::default();
                }
                self.dirty = false;
            }
            ReportOutcome::Deferred => {}
        }
        Ok(())
    }
}

pub(crate) struct PipelineCompletionMetricsState {
    level: MetricLevel,
    reporter: MetricsReporter,
    metrics: Option<RegisteredMetricSet<PipelineCompletionMetrics>>,
    dirty: bool,
    pending_sends_buffered: u64,
    deliver_ack_received: u64,
    deliver_nack_received: u64,
    ack_sent: u64,
    nack_sent: u64,
    ack_dropped_no_interest: u64,
    nack_dropped_no_interest: u64,
    unwind_depth: Mmsc,
}

impl PipelineCompletionMetricsState {
    pub(crate) fn new(
        pipeline_ctx: &PipelineContext,
        reporter: MetricsReporter,
        level: MetricLevel,
    ) -> Self {
        Self {
            level,
            reporter,
            metrics: (level != MetricLevel::None).then(|| RegisteredMetricSet::new(pipeline_ctx)),
            dirty: level != MetricLevel::None,
            pending_sends_buffered: 0,
            deliver_ack_received: 0,
            deliver_nack_received: 0,
            ack_sent: 0,
            nack_sent: 0,
            ack_dropped_no_interest: 0,
            nack_dropped_no_interest: 0,
            unwind_depth: Mmsc::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn metric_set_key(&self) -> Option<MetricSetKey> {
        self.metrics
            .as_ref()
            .map(RegisteredMetricSet::metric_set_key)
    }

    pub(crate) fn set_pending_sends_buffered(&mut self, buffered: usize) {
        let buffered = buffered as u64;
        if self.pending_sends_buffered != buffered {
            self.pending_sends_buffered = buffered;
            self.dirty = true;
        }
    }

    pub(crate) fn record_deliver_ack_received(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.deliver_ack_received += 1;
            self.dirty = true;
        }
    }

    pub(crate) fn record_deliver_nack_received(&mut self) {
        if self.level >= MetricLevel::Normal {
            self.deliver_nack_received += 1;
            self.dirty = true;
        }
    }

    pub(crate) fn record_ack_sent(&mut self, unwind_depth: usize) {
        if self.level >= MetricLevel::Normal {
            self.ack_sent += 1;
            self.dirty = true;
        }
        if self.level >= MetricLevel::Detailed {
            self.unwind_depth.record(unwind_depth as f64);
            self.dirty = true;
        }
    }

    pub(crate) fn record_nack_sent(&mut self, unwind_depth: usize) {
        if self.level >= MetricLevel::Normal {
            self.nack_sent += 1;
            self.dirty = true;
        }
        if self.level >= MetricLevel::Detailed {
            self.unwind_depth.record(unwind_depth as f64);
            self.dirty = true;
        }
    }

    pub(crate) fn record_ack_dropped_no_interest(&mut self, unwind_depth: usize) {
        if self.level >= MetricLevel::Normal {
            self.ack_dropped_no_interest += 1;
            self.dirty = true;
        }
        if self.level >= MetricLevel::Detailed {
            self.unwind_depth.record(unwind_depth as f64);
            self.dirty = true;
        }
    }

    pub(crate) fn record_nack_dropped_no_interest(&mut self, unwind_depth: usize) {
        if self.level >= MetricLevel::Normal {
            self.nack_dropped_no_interest += 1;
            self.dirty = true;
        }
        if self.level >= MetricLevel::Detailed {
            self.unwind_depth.record(unwind_depth as f64);
            self.dirty = true;
        }
    }

    pub(crate) fn report_if_needed(&mut self) -> Result<(), TelemetryError> {
        if !self.dirty {
            return Ok(());
        }
        let Some(registered) = self.metrics.as_mut() else {
            self.dirty = false;
            return Ok(());
        };

        registered
            .metrics
            .pending_sends_buffered
            .set(self.pending_sends_buffered);
        if self.level >= MetricLevel::Normal {
            registered.metrics.deliver_ack_received = self.deliver_ack_received.into();
            registered.metrics.deliver_nack_received = self.deliver_nack_received.into();
            registered.metrics.ack_sent = self.ack_sent.into();
            registered.metrics.nack_sent = self.nack_sent.into();
            registered.metrics.ack_dropped_no_interest = self.ack_dropped_no_interest.into();
            registered.metrics.nack_dropped_no_interest = self.nack_dropped_no_interest.into();
        } else {
            registered.metrics.deliver_ack_received = Counter::default();
            registered.metrics.deliver_nack_received = Counter::default();
            registered.metrics.ack_sent = Counter::default();
            registered.metrics.nack_sent = Counter::default();
            registered.metrics.ack_dropped_no_interest = Counter::default();
            registered.metrics.nack_dropped_no_interest = Counter::default();
        }

        if self.level >= MetricLevel::Detailed {
            registered.metrics.unwind_depth = self.unwind_depth;
        } else {
            registered.metrics.unwind_depth = Mmsc::default();
        }

        match self
            .reporter
            .try_report_snapshot_with_outcome(registered.metrics.snapshot())?
        {
            ReportOutcome::Sent => {
                if self.level >= MetricLevel::Normal {
                    self.deliver_ack_received = 0;
                    self.deliver_nack_received = 0;
                    self.ack_sent = 0;
                    self.nack_sent = 0;
                    self.ack_dropped_no_interest = 0;
                    self.nack_dropped_no_interest = 0;
                }
                if self.level >= MetricLevel::Detailed {
                    self.unwind_depth = Mmsc::default();
                }
                self.dirty = false;
            }
            ReportOutcome::Deferred => {}
        }
        Ok(())
    }
}
