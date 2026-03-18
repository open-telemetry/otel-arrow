// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Runtime-control manager for handling timer-based operations.
//!
//! This module provides the `RuntimeCtrlMsgManager` which is responsible for managing
//! timers for nodes in the pipeline. It handles scheduling, cancellation, and expiration
//! of recurring timers, using a priority queue for efficient timer management.
//!
//! Note 1: This manager is designed for single-threaded async execution.
//! Note 2: Other runtime-control messages can be added in the future, but currently only timers
//! are supported.

use crate::channel_metrics::{ConsumedMetrics, ProducedMetrics};
use crate::context::PipelineContext;
use crate::control::RouteData;
use crate::control::UnwindData;
use crate::control::{
    AckMsg, ControlSenders, NackMsg, NodeControlMsg, PipelineResultMsg, PipelineResultMsgReceiver,
    RuntimeControlMsg, RuntimeCtrlMsgReceiver,
};
use crate::error::Error;
use crate::pipeline_metrics::PipelineMetricsMonitor;
use crate::{Interests, RequestOutcome, Unwindable};
use otap_df_config::DeployedPipelineKey;
use otap_df_config::MetricLevel;
use otap_df_config::policy::TelemetryPolicy;
use otap_df_telemetry::error::Error as TelemetryError;
use otap_df_telemetry::event::{EngineEvent, ObservedEventReporter};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry::{otel_debug, otel_warn};
use std::cell::RefCell;
use std::cmp::Reverse;
use std::collections::{BinaryHeap, HashMap, HashSet, VecDeque};
use std::rc::Rc;
use std::time::{Duration, Instant};

/// Threshold for the pending sends buffer. When the buffer exceeds this size,
/// a warning is logged to help operators diagnose sustained backpressure.
const PENDING_SENDS_WARN_THRESHOLD: usize = 100;

/// Maximum number of consecutive runtime-control messages handled before the
/// manager forces one due expiry pass.
const RUNTIME_CTRL_BURST: usize = 64;

/// Represents delayed data with scheduling information.
#[derive(Debug)]
struct Delayed<PData> {
    /// When to resume processing this data.
    when: Instant,
    /// Target node ID for the delayed data.
    node_id: usize,
    /// The delayed data payload.
    data: Box<PData>,
}

/// For BinaryHeap ordering - earlier times have higher priority (min-heap behavior).
impl<PData> Ord for Delayed<PData> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse ordering for min-heap (earlier times first)
        other.when.cmp(&self.when)
    }
}

impl<PData> PartialOrd for Delayed<PData> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<PData> PartialEq for Delayed<PData> {
    fn eq(&self, other: &Self) -> bool {
        self.when == other.when
    }
}

impl<PData> Eq for Delayed<PData> {}

/// Timer state for a node.
struct TimerState {
    scheduled_time: Instant,
    duration: Duration,
    is_canceled: bool,
}

/// Manages runtime-control messages such as recurrent and cancelable timers.
///
/// This manager is responsible for managing timers for nodes in the pipeline.
/// It uses a priority queue to efficiently handle timer expirations and cancellations.
///
/// Design notes:
/// - Only one timer per node is supported at a time.
/// - All data structures are optimized for single-threaded async use.
/// - The timer_states map consolidates all timer information for efficiency and correctness.
/// - The combination of `timer_map` and `canceled` ensures correctness and avoids spurious timer
///   events.
///
/// A reusable per-node repeating timer set.
///
/// Manages scheduling, cancellation, and expiration for recurrent timers keyed by NodeId.
/// Optimized for single-threaded async use.
struct TimerSet {
    timers: BinaryHeap<Reverse<(Instant, usize)>>,
    /// Maps node ID to timer state (scheduled time, duration, and cancellation status).
    timer_states: HashMap<usize, TimerState>,
}

impl TimerSet {
    fn new() -> Self {
        Self {
            timers: BinaryHeap::new(),
            timer_states: HashMap::new(),
        }
    }

    /// Schedule or replace a repeating timer for node_id.
    fn start(&mut self, node_id: usize, duration: Duration) {
        let when = Instant::now() + duration;
        self.timers.push(Reverse((when, node_id)));
        let _ = self.timer_states.insert(
            node_id,
            TimerState {
                scheduled_time: when,
                duration,
                is_canceled: false,
            },
        );
    }

    /// Cancel an existing timer for node_id.
    fn cancel(&mut self, node_id: usize) {
        // Mark the timer as canceled.
        if let Some(timer_state) = self.timer_states.get_mut(&node_id) {
            timer_state.is_canceled = true;
        }
    }

    /// Cancel all timers.
    fn cancel_all(&mut self) {
        self.timers.clear();
        self.timer_states.clear();
    }

    /// Peek the next expiration instant, if any.
    fn next_expiry(&self) -> Option<Instant> {
        self.timers.peek().map(|Reverse((when, _))| *when)
    }

    /// Fire all due timers at or before `now`, invoking the provided callback per firing node.
    /// Reschedules recurring timers automatically when still active.
    fn fire_due<F: FnMut(&usize)>(&mut self, now: Instant, mut on_fire: F) {
        while let Some(Reverse((when, node_id))) = self.timers.peek().cloned() {
            if when > now {
                break;
            }
            // Pop the entry and validate it.
            let _ = self.timers.pop();
            if let Some(timer_state) = self.timer_states.get_mut(&node_id) {
                if !timer_state.is_canceled && timer_state.scheduled_time == when {
                    // Fire callback
                    on_fire(&node_id);

                    // Schedule next recurrence based on the original scheduled time to prevent drift
                    let next_when = timer_state.scheduled_time + timer_state.duration;
                    self.timers.push(Reverse((next_when, node_id)));
                    timer_state.scheduled_time = next_when;
                } else if timer_state.is_canceled {
                    // Clean up canceled timers
                    let _ = self.timer_states.remove(&node_id);
                }
            }
        }
    }
}

fn opt_min<T: Ord>(a: Option<T>, b: Option<T>) -> Option<T> {
    match (a, b) {
        (Some(a), Some(b)) => Some(std::cmp::min(a, b)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

/// Per-node metrics handles for recording consumed/produced outcomes.
pub(crate) struct NodeMetricHandles {
    /// Registry handle for automatic unregistration on drop.
    pub(crate) registry: TelemetryRegistryHandle,
    /// Consumed-request metrics for the node's input channel.
    pub(crate) input: Option<MetricSet<ConsumedMetrics>>,
    /// Produced-request metrics indexed by output port.
    pub(crate) outputs: Vec<MetricSet<ProducedMetrics>>,
}

pub(crate) fn report_node_metrics_with_handles(
    node_metric_handles: &Rc<RefCell<Vec<Option<NodeMetricHandles>>>>,
    metrics_reporter: &mut MetricsReporter,
) -> Result<(), TelemetryError> {
    let mut handles_guard = node_metric_handles.borrow_mut();
    for handles in handles_guard.iter_mut().flatten() {
        if let Some(input) = &mut handles.input {
            metrics_reporter.report(input)?;
        }
        for output in &mut handles.outputs {
            metrics_reporter.report(output)?;
        }
    }
    Ok(())
}

impl Drop for NodeMetricHandles {
    fn drop(&mut self) {
        if let Some(input) = self.input.take() {
            let _ = self.registry.unregister_metric_set(input.metric_set_key());
        }
        for output in self.outputs.drain(..) {
            let _ = self.registry.unregister_metric_set(output.metric_set_key());
        }
    }
}

/// Manages runtime-control messages and per-node recurring timers (tick and telemetry).
///
/// Internally uses two TimerSet instances: one for generic TimerTick and one for
/// CollectTelemetry events. It receives Start*/Cancel* requests and emits the
/// corresponding NodeControlMsg to nodes when timers expire.
pub struct RuntimeCtrlMsgManager<PData> {
    /// The key identifying the deployed pipeline this manager is responsible for.
    pipeline_key: DeployedPipelineKey,
    /// Context information about the pipeline.
    pipeline_context: PipelineContext,
    /// Receives control messages from nodes (e.g., start/cancel timer).
    runtime_ctrl_msg_receiver: RuntimeCtrlMsgReceiver<PData>,
    /// Allows sending control messages back to nodes.
    control_senders: ControlSenders<PData>,
    /// Repeating timers for generic TimerTick.
    tick_timers: TimerSet,
    /// Repeating timers for telemetry collection (CollectTelemetry).
    telemetry_timers: TimerSet,
    /// Delayed data in activation order.
    delayed_data: BinaryHeap<Delayed<PData>>,
    /// Event reporter used to report major events influencing the pipeline's behavior.
    event_reporter: ObservedEventReporter,
    /// Global metrics reporter.
    metrics_reporter: MetricsReporter,
    /// Channel metrics handles for periodic reporting.
    channel_metrics: Vec<crate::channel_metrics::ChannelMetricsHandle>,

    /// Per-node metrics handles for recording consumed/produced outcomes.
    node_metric_handles: Rc<RefCell<Vec<Option<NodeMetricHandles>>>>,

    /// Flags controlling capture of internal engine metrics.
    telemetry: TelemetryPolicy,

    /// Messages that could not be delivered because the target node's control
    /// channel was full. Buffered here instead of blocking the event loop,
    /// which would cause a circular-wait stall on the single-threaded
    /// LocalSet runtime.
    pending_sends: VecDeque<(usize, NodeControlMsg<PData>)>,
}

impl<PData> RuntimeCtrlMsgManager<PData> {
    /// Creates a new RuntimeCtrlMsgManager.
    #[must_use]
    pub(crate) fn new(
        pipeline_key: DeployedPipelineKey,
        pipeline_context: PipelineContext,
        runtime_ctrl_msg_receiver: RuntimeCtrlMsgReceiver<PData>,
        control_senders: ControlSenders<PData>,
        event_reporter: ObservedEventReporter,
        metrics_reporter: MetricsReporter,
        telemetry_policy: TelemetryPolicy,
        channel_metrics: Vec<crate::channel_metrics::ChannelMetricsHandle>,
        node_metric_handles: Rc<RefCell<Vec<Option<NodeMetricHandles>>>>,
    ) -> Self {
        Self {
            pipeline_key,
            pipeline_context,
            runtime_ctrl_msg_receiver,
            control_senders,
            tick_timers: TimerSet::new(),
            telemetry_timers: TimerSet::new(),
            delayed_data: BinaryHeap::new(),
            event_reporter,
            metrics_reporter,
            channel_metrics,
            node_metric_handles,
            telemetry: telemetry_policy,
            pending_sends: VecDeque::new(),
        }
    }

    /// Runs the runtime-control manager event loop.
    pub async fn run(mut self) -> Result<(), Error> {
        let internal_telemetry_enabled =
            self.telemetry.pipeline_metrics || self.telemetry.tokio_metrics;
        let mut pipeline_metrics_monitor = internal_telemetry_enabled
            .then(|| PipelineMetricsMonitor::new(self.pipeline_context.clone()));

        let mut is_draining_ingress = false;
        let mut shutdown_deadline: Option<Instant> = None;
        let mut shutdown_reason: Option<String> = None;
        let mut pending_receivers: HashSet<usize> = HashSet::new();
        let mut downstream_shutdown_sent = false;
        let mut consecutive_runtime_ctrl = 0usize;

        // Single reusable timer for retrying buffered sends. Created once,
        // reset only when `pending_sends` transitions from empty to non-empty.
        // This avoids allocating a new Sleep future on every loop iteration
        // (the standard tokio::select! pinned-sleep pattern).
        let retry_delay = tokio::time::sleep(Duration::from_millis(5));
        tokio::pin!(retry_delay);
        let mut retry_armed = false;

        loop {
            // Drain any buffered sends before processing new messages.
            self.drain_pending_sends();

            // Arm the retry timer when pending sends appear; disarm when drained.
            if !self.pending_sends.is_empty() && !retry_armed {
                retry_delay
                    .as_mut()
                    .reset(tokio::time::Instant::now() + Duration::from_millis(5));
                retry_armed = true;
            } else if self.pending_sends.is_empty() {
                retry_armed = false;
            }

            let now = Instant::now();

            if let Some(deadline) = shutdown_deadline {
                if now >= deadline {
                    if let Some(reason) = shutdown_reason.as_ref() {
                        for node_id in self.control_senders.non_receiver_ids() {
                            self.send(
                                node_id,
                                NodeControlMsg::Shutdown {
                                    deadline,
                                    reason: reason.clone(),
                                },
                            );
                        }
                        for node_id in pending_receivers.iter().copied() {
                            self.send(
                                node_id,
                                NodeControlMsg::Shutdown {
                                    deadline,
                                    reason: reason.clone(),
                                },
                            );
                        }
                    }
                    break;
                }
            }

            let next_earliest = if is_draining_ingress {
                shutdown_deadline
            } else {
                let next_expiry = self.tick_timers.next_expiry();
                let next_tel_expiry = self.telemetry_timers.next_expiry();
                let next_delay_expiry = self.delayed_data.peek().map(|d| d.when);
                opt_min(opt_min(next_expiry, next_tel_expiry), next_delay_expiry)
            };

            if consecutive_runtime_ctrl >= RUNTIME_CTRL_BURST
                && next_earliest.is_some_and(|when| when <= now)
            {
                self.handle_due_events(now, &mut pipeline_metrics_monitor);
                consecutive_runtime_ctrl = 0;
                continue;
            }

            tokio::select! {
                biased;

                msg = self.runtime_ctrl_msg_receiver.recv() => {
                    let Some(msg) = msg.ok() else { break; };
                    consecutive_runtime_ctrl += 1;
                    match msg {
                        RuntimeControlMsg::Shutdown { deadline, reason } => {
                            if is_draining_ingress {
                                continue;
                            }
                            self.event_reporter.report(EngineEvent::shutdown_requested(
                                self.pipeline_key.clone(),
                                Some(reason.clone()),
                            ));
                            is_draining_ingress = true;
                            shutdown_deadline = Some(deadline);
                            shutdown_reason = Some(reason.clone());
                            pending_receivers = self.control_senders.receiver_ids().into_iter().collect();
                            self.tick_timers.cancel_all();
                            self.telemetry_timers.cancel_all();
                            self.flush_delayed_data_now(now);

                            for node_id in pending_receivers.iter().copied() {
                                self.send(
                                    node_id,
                                    NodeControlMsg::DrainIngress {
                                        deadline,
                                        reason: reason.clone(),
                                    },
                                );
                            }

                            if pending_receivers.is_empty() {
                                for node_id in self.control_senders.non_receiver_ids() {
                                    self.send(
                                        node_id,
                                        NodeControlMsg::Shutdown {
                                            deadline,
                                            reason: reason.clone(),
                                        },
                                    );
                                }
                                downstream_shutdown_sent = true;
                            }
                        },
                        RuntimeControlMsg::ReceiverDrained { node_id } => {
                            if !is_draining_ingress {
                                continue;
                            }
                            let _ = pending_receivers.remove(&node_id);
                            if pending_receivers.is_empty() && !downstream_shutdown_sent {
                                let deadline = shutdown_deadline.unwrap_or(now);
                                let reason = shutdown_reason
                                    .clone()
                                    .unwrap_or_else(|| "pipeline shutting down".to_owned());
                                for node_id in self.control_senders.non_receiver_ids() {
                                    self.send(
                                        node_id,
                                        NodeControlMsg::Shutdown {
                                            deadline,
                                            reason: reason.clone(),
                                        },
                                    );
                                }
                                downstream_shutdown_sent = true;
                            }
                        }
                        RuntimeControlMsg::StartTimer { node_id, duration } => {
                            if is_draining_ingress {
                                otel_debug!(
                                    "pipeline.draining.ignored_start_timer",
                                    node_id = node_id,
                                );
                            } else {
                                self.tick_timers.start(node_id, duration);
                            }
                        }
                        RuntimeControlMsg::CancelTimer { node_id } => {
                            if !is_draining_ingress {
                                self.tick_timers.cancel(node_id);
                            }
                        }
                        RuntimeControlMsg::StartTelemetryTimer { node_id, duration } => {
                            if is_draining_ingress {
                                otel_debug!(
                                    "pipeline.draining.ignored_start_telemetry_timer",
                                    node_id = node_id,
                                    "Ignoring StartTelemetryTimer during shutdown draining"
                                );
                            } else {
                                self.telemetry_timers.start(node_id, duration);
                            }
                        }
                        RuntimeControlMsg::CancelTelemetryTimer { node_id, .. } => {
                            if !is_draining_ingress {
                                self.telemetry_timers.cancel(node_id);
                            }
                        }
                        RuntimeControlMsg::DelayData { node_id, when, data } => {
                            if is_draining_ingress {
                                self.send(
                                    node_id,
                                    NodeControlMsg::DelayedData {
                                        when: now,
                                        data,
                                    },
                                );
                            } else {
                                let delayed = Delayed { node_id, when, data };
                                self.delayed_data.push(delayed);
                            }
                        }
                    }
                }
                _ = async {
                    if let Some(when) = next_earliest {
                        if when > now {
                            let tokio_when = tokio::time::Instant::from_std(when);
                            tokio::time::sleep_until(tokio_when).await;
                        }
                    }
                }, if next_earliest.is_some() => {
                    consecutive_runtime_ctrl = 0;
                    if !is_draining_ingress {
                        self.handle_due_events(Instant::now(), &mut pipeline_metrics_monitor);
                    }
                }

                _ = &mut retry_delay, if retry_armed => {
                    retry_armed = false;
                    continue;
                }
            }
        }

        if self.telemetry.channel_metrics >= MetricLevel::Normal {
            let _ = self.report_node_metrics();
        }

        Ok(())
    }

    fn flush_delayed_data_now(&mut self, when: Instant) {
        while let Some(delayed) = self.delayed_data.pop() {
            self.send(
                delayed.node_id,
                NodeControlMsg::DelayedData {
                    when,
                    data: delayed.data,
                },
            );
        }
    }

    fn handle_due_events(
        &mut self,
        now: Instant,
        pipeline_metrics_monitor: &mut Option<PipelineMetricsMonitor>,
    ) {
        let mut to_send: Vec<(usize, NodeControlMsg<PData>)> = Vec::new();

        self.tick_timers.fire_due(now, |node_id| {
            to_send.push((*node_id, NodeControlMsg::TimerTick {}));
        });

        let metrics_reporter = self.metrics_reporter.clone();
        self.telemetry_timers.fire_due(now, |node_id| {
            to_send.push((
                *node_id,
                NodeControlMsg::CollectTelemetry {
                    metrics_reporter: metrics_reporter.clone(),
                },
            ));
        });

        while self
            .delayed_data
            .peek()
            .map(|d| d.when <= now)
            .unwrap_or(false)
        {
            let delayed = self.delayed_data.pop().expect("ok");
            to_send.push((
                delayed.node_id,
                NodeControlMsg::DelayedData {
                    when: delayed.when,
                    data: delayed.data,
                },
            ));
        }

        if let Some(pipeline_metrics_monitor) = pipeline_metrics_monitor.as_mut() {
            if self.telemetry.pipeline_metrics {
                pipeline_metrics_monitor.update_pipeline_metrics();
                if let Err(err) = self
                    .metrics_reporter
                    .report(pipeline_metrics_monitor.metrics_mut())
                {
                    otel_warn!("pipeline.metrics.reporting.fail", error = err.to_string());
                }
            }

            if self.telemetry.tokio_metrics {
                pipeline_metrics_monitor.update_tokio_metrics();
                if let Err(err) = self
                    .metrics_reporter
                    .report(pipeline_metrics_monitor.tokio_metrics_mut())
                {
                    otel_warn!("tokio.metrics.reporting.fail", error = err.to_string());
                }
            }
        }

        if self.telemetry.channel_metrics >= MetricLevel::Basic {
            for metrics in &self.channel_metrics {
                if let Err(err) = metrics.report(&mut self.metrics_reporter) {
                    otel_warn!("channel.metrics.reporting.fail", error = err.to_string());
                }
            }
        }
        if self.telemetry.channel_metrics >= MetricLevel::Normal {
            if let Err(err) = self.report_node_metrics() {
                otel_warn!("node.metrics.reporting.fail", error = err.to_string());
            }
        }

        for (node_id, msg) in to_send {
            self.send(node_id, msg);
        }
    }

    /// Report all per-node consumed/produced metric sets.
    fn report_node_metrics(&mut self) -> Result<(), TelemetryError> {
        report_node_metrics_with_handles(&self.node_metric_handles, &mut self.metrics_reporter)
    }

    /// Non-blocking send: try to deliver immediately, buffer on backpressure.
    ///
    /// Previously this method was `async` and fell back to `sender.send(msg).await`
    /// when the channel was full, which blocked the entire single-threaded event
    /// loop and caused a circular-wait stall.
    ///
    /// Now we never block: on `Full` the message is pushed to `pending_sends` and
    /// retried on subsequent loop iterations via `drain_pending_sends()`.
    fn send(&mut self, node_id: usize, msg: NodeControlMsg<PData>) {
        if let Some(sender) = self.control_senders.get(node_id) {
            match sender.try_send(msg) {
                Ok(()) => {}
                Err(otap_df_channel::error::SendError::Full(msg)) => {
                    self.pending_sends.push_back((node_id, msg));
                    if self.pending_sends.len() == PENDING_SENDS_WARN_THRESHOLD {
                        otel_warn!(
                            "pipeline.ctrl.pending_sends.high",
                            count = self.pending_sends.len(),
                            "Pending sends buffer reached threshold; \
                             a node's control channel may be persistently full"
                        );
                    }
                }
                Err(otap_df_channel::error::SendError::Closed(_)) => {
                    // Ignore closed channel
                }
            }
        }
    }

    /// Best-effort drain of buffered sends.  Messages that still cannot be
    /// delivered are re-queued at the back of the same deque (no allocation).
    fn drain_pending_sends(&mut self) {
        let n = self.pending_sends.len();
        for _ in 0..n {
            let Some((node_id, msg)) = self.pending_sends.pop_front() else {
                break;
            };
            if let Some(sender) = self.control_senders.get(node_id) {
                match sender.try_send(msg) {
                    Ok(()) => {}
                    Err(otap_df_channel::error::SendError::Full(msg)) => {
                        self.pending_sends.push_back((node_id, msg));
                    }
                    Err(otap_df_channel::error::SendError::Closed(_)) => {
                        // Drop message for closed channel
                    }
                }
            }
        }
    }
}

/// Dedicated return-path dispatcher for Ack/Nack unwinding.
pub struct PipelineResultMsgDispatcher<PData> {
    pipeline_result_msg_receiver: PipelineResultMsgReceiver<PData>,
    control_senders: ControlSenders<PData>,
    node_metric_handles: Rc<RefCell<Vec<Option<NodeMetricHandles>>>>,
    pending_sends: VecDeque<(usize, NodeControlMsg<PData>)>,
}

impl<PData> PipelineResultMsgDispatcher<PData> {
    #[must_use]
    pub(crate) fn new(
        pipeline_result_msg_receiver: PipelineResultMsgReceiver<PData>,
        control_senders: ControlSenders<PData>,
        node_metric_handles: Rc<RefCell<Vec<Option<NodeMetricHandles>>>>,
    ) -> Self {
        Self {
            pipeline_result_msg_receiver,
            control_senders,
            node_metric_handles,
            pending_sends: VecDeque::new(),
        }
    }

    fn send(&mut self, node_id: usize, msg: NodeControlMsg<PData>) {
        if let Some(sender) = self.control_senders.get(node_id) {
            match sender.try_send(msg) {
                Ok(()) => {}
                Err(otap_df_channel::error::SendError::Full(msg)) => {
                    self.pending_sends.push_back((node_id, msg));
                    if self.pending_sends.len() == PENDING_SENDS_WARN_THRESHOLD {
                        otel_warn!(
                            "pipeline.return.pending_sends.high",
                            count = self.pending_sends.len(),
                            "Pending return sends buffer reached threshold"
                        );
                    }
                }
                Err(otap_df_channel::error::SendError::Closed(_)) => {}
            }
        }
    }

    fn drain_pending_sends(&mut self) {
        let n = self.pending_sends.len();
        for _ in 0..n {
            let Some((node_id, msg)) = self.pending_sends.pop_front() else {
                break;
            };
            if let Some(sender) = self.control_senders.get(node_id) {
                match sender.try_send(msg) {
                    Ok(()) => {}
                    Err(otap_df_channel::error::SendError::Full(msg)) => {
                        self.pending_sends.push_back((node_id, msg));
                    }
                    Err(otap_df_channel::error::SendError::Closed(_)) => {}
                }
            }
        }
    }

    fn record_frame_metrics(
        &mut self,
        node_id: usize,
        interests: Interests,
        route: &RouteData,
        outcome: RequestOutcome,
        now_ns: u64,
    ) {
        let mut handles_guard = self.node_metric_handles.borrow_mut();
        if let Some(Some(handles)) = handles_guard.get_mut(node_id) {
            if interests.contains(Interests::CONSUMER_METRICS) {
                if let Some(input) = &mut handles.input {
                    match outcome {
                        RequestOutcome::Success => input.consumed_success.inc(),
                        RequestOutcome::Failure => input.consumed_failure.inc(),
                        RequestOutcome::Refused => input.consumed_refused.inc(),
                    }
                    if route.entry_time_ns > 0 && now_ns > 0 {
                        let duration_ns = now_ns.saturating_sub(route.entry_time_ns);
                        input.consumed_duration_ns.record(duration_ns as f64);
                    }
                }
            }
            if interests.contains(Interests::PRODUCER_METRICS) {
                let port = route.output_port_index as usize;
                if let Some(output) = handles.outputs.get_mut(port) {
                    match outcome {
                        RequestOutcome::Success => output.produced_success.inc(),
                        RequestOutcome::Failure => output.produced_failure.inc(),
                        RequestOutcome::Refused => output.produced_refused.inc(),
                    }
                    if !interests.contains(Interests::CONSUMER_METRICS)
                        && route.entry_time_ns > 0
                        && now_ns > 0
                    {
                        let duration_ns = now_ns.saturating_sub(route.entry_time_ns);
                        output.produced_duration_ns.record(duration_ns as f64);
                    }
                }
            }
        }
    }
}

impl<PData: Unwindable> PipelineResultMsgDispatcher<PData> {
    /// Runs the return-path dispatcher until all return senders are dropped.
    pub async fn run(mut self) -> Result<(), Error> {
        let retry_delay = tokio::time::sleep(Duration::from_millis(5));
        tokio::pin!(retry_delay);
        let mut retry_armed = false;

        loop {
            self.drain_pending_sends();

            if !self.pending_sends.is_empty() && !retry_armed {
                retry_delay
                    .as_mut()
                    .reset(tokio::time::Instant::now() + Duration::from_millis(5));
                retry_armed = true;
            } else if self.pending_sends.is_empty() {
                retry_armed = false;
            }

            tokio::select! {
                biased;

                msg = self.pipeline_result_msg_receiver.recv() => {
                    let Some(msg) = msg.ok() else { break; };
                    match msg {
                        PipelineResultMsg::DeliverAck { ack } => self.unwind_ack(ack),
                        PipelineResultMsg::DeliverNack { nack } => self.unwind_nack(nack),
                    }
                }

                _ = &mut retry_delay, if retry_armed => {
                    retry_armed = false;
                }
            }
        }

        Ok(())
    }

    fn unwind_frames(
        &mut self,
        pdata: &mut PData,
        now_ns: u64,
        outcome: RequestOutcome,
        interest: Interests,
    ) -> Option<(usize, RouteData)> {
        loop {
            match pdata.pop_frame() {
                None => return None,
                Some(frame) => {
                    if frame.interests.intersects(Interests::PIPELINE_METRICS) {
                        self.record_frame_metrics(
                            frame.node_id,
                            frame.interests,
                            &frame.route,
                            outcome,
                            now_ns,
                        );
                    }
                    if frame.interests.contains(interest) {
                        if !frame.interests.contains(Interests::RETURN_DATA) {
                            pdata.drop_payload();
                        }
                        return Some((frame.node_id, frame.route));
                    }
                }
            }
        }
    }

    fn unwind_ack(&mut self, mut ack: AckMsg<PData>) {
        let now_ns = ack.unwind.return_time_ns;
        if let Some((node_id, route)) = self.unwind_frames(
            &mut ack.accepted,
            now_ns,
            RequestOutcome::Success,
            Interests::ACKS,
        ) {
            ack.unwind = UnwindData::new(route, now_ns);
            self.send(node_id, NodeControlMsg::Ack(ack));
        }
    }

    fn unwind_nack(&mut self, mut nack: NackMsg<PData>) {
        let now_ns = nack.unwind.return_time_ns;
        let outcome = if nack.permanent {
            RequestOutcome::Refused
        } else {
            RequestOutcome::Failure
        };
        if let Some((node_id, route)) =
            self.unwind_frames(&mut nack.refused, now_ns, outcome, Interests::NACKS)
        {
            nack.unwind = UnwindData::new(route, now_ns);
            self.send(node_id, NodeControlMsg::Nack(nack));
        }
    }
}

// Test-only helpers to introspect internal state without exposing fields publicly.
#[cfg(test)]
impl<PData> RuntimeCtrlMsgManager<PData> {
    pub(crate) fn test_tick_count(&self) -> usize {
        self.tick_timers.timers.len()
    }

    pub(crate) fn test_telemetry_count(&self) -> usize {
        self.telemetry_timers.timers.len()
    }

    pub(crate) fn test_control_senders_len(&self) -> usize {
        self.control_senders.len()
    }

    pub(crate) fn test_push_tick_heap(&mut self, when: Instant, node_id: usize) {
        self.tick_timers.timers.push(Reverse((when, node_id)));
    }

    pub(crate) fn test_pop_tick_heap(&mut self) -> Option<(Instant, usize)> {
        self.tick_timers
            .timers
            .pop()
            .map(|Reverse((when, node))| (when, node))
    }

    pub(crate) fn test_tick_heap_len(&self) -> usize {
        self.tick_timers.timers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::channel_metrics::{ConsumedMetrics, ProducedMetrics};
    use crate::context::ControllerContext;
    use crate::control::{AckMsg, Frame, NackMsg, RouteData, nanos_since_birth};
    use crate::control::{
        NodeControlMsg, PipelineResultMsg, RuntimeControlMsg, pipeline_result_msg_channel,
        runtime_ctrl_msg_channel,
    };
    use crate::message::{Receiver, Sender};
    use crate::node::{NodeId, NodeType};
    use crate::shared::message::{SharedReceiver, SharedSender};
    use crate::testing::test_nodes;
    use otap_df_config::observed_state::{ObservedStateSettings, SendPolicy};
    use otap_df_config::{PipelineGroupId, PipelineId};
    use otap_df_state::store::ObservedStateStore;
    use otap_df_telemetry::metrics::{MetricSetSnapshot, MetricValue};
    use otap_df_telemetry::registry::MetricSetKey;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::cell::RefCell;
    use std::collections::HashMap;
    use std::rc::Rc;
    use std::time::{Duration, Instant};
    use tokio::task::LocalSet;
    use tokio::time::timeout;

    fn empty_node_metric_handles() -> Rc<RefCell<Vec<Option<NodeMetricHandles>>>> {
        Rc::new(RefCell::new(Vec::new()))
    }

    fn create_mock_control_sender<PData>() -> (
        Sender<NodeControlMsg<PData>>,
        Receiver<NodeControlMsg<PData>>,
    ) {
        create_mock_control_sender_with_capacity(10)
    }

    fn create_mock_control_sender_with_capacity<PData>(
        capacity: usize,
    ) -> (
        Sender<NodeControlMsg<PData>>,
        Receiver<NodeControlMsg<PData>>,
    ) {
        let (tx, rx) = tokio::sync::mpsc::channel(capacity);
        (
            Sender::Shared(SharedSender::mpsc(tx)),
            Receiver::Shared(SharedReceiver::mpsc(rx)),
        )
    }

    fn build_test_manager<PData>(
        pipeline_capacity: usize,
        control_senders: ControlSenders<PData>,
    ) -> (
        RuntimeCtrlMsgManager<PData>,
        crate::control::RuntimeCtrlMsgSender<PData>,
        crate::entity_context::PipelineEntityScope,
    ) {
        let (pipeline_tx, pipeline_rx) = runtime_ctrl_msg_channel(pipeline_capacity);

        let metrics_system = otap_df_telemetry::InternalTelemetrySystem::default();
        let metrics_reporter = metrics_system.reporter();
        let observed_state_store =
            ObservedStateStore::new(&ObservedStateSettings::default(), metrics_system.registry());
        let pipeline_group_id: PipelineGroupId = Default::default();
        let pipeline_id: PipelineId = Default::default();
        let core_id = 0;
        let thread_id = 0;
        let controller_context = ControllerContext::new(metrics_system.registry());
        let pipeline_context = PipelineContext::new(
            controller_context,
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            core_id,
            1, // num_cores
            thread_id,
        );

        let pipeline_entity_key = pipeline_context.register_pipeline_entity();
        let pipeline_entity_guard = crate::entity_context::set_pipeline_entity_key(
            pipeline_context.metrics_registry(),
            pipeline_entity_key,
        );

        let manager = RuntimeCtrlMsgManager::new(
            DeployedPipelineKey {
                pipeline_group_id,
                pipeline_id,
                core_id,
            },
            pipeline_context,
            pipeline_rx,
            control_senders,
            observed_state_store.reporter(SendPolicy::default()),
            metrics_reporter,
            TelemetryPolicy::default(),
            Vec::new(),
            empty_node_metric_handles(),
        );

        (manager, pipeline_tx, pipeline_entity_guard)
    }

    fn setup_test_manager_with_capacities<PData: Clone>(
        pipeline_capacity: usize,
        control_capacity: usize,
    ) -> (
        RuntimeCtrlMsgManager<PData>,
        crate::control::RuntimeCtrlMsgSender<PData>,
        ControlSenders<PData>,
        HashMap<usize, Receiver<NodeControlMsg<PData>>>,
        Vec<NodeId>,
        crate::entity_context::PipelineEntityScope,
    ) {
        let mut control_senders = ControlSenders::new();
        let mut control_receivers = HashMap::new();

        // Create mock control senders for test nodes
        let nodes = test_nodes(vec!["node1", "node2", "node3"]);
        for node in &nodes {
            let (sender, receiver) = create_mock_control_sender_with_capacity(control_capacity);
            control_senders.register(node.clone(), NodeType::Processor, sender);
            let _ = control_receivers.insert(node.index, receiver);
        }

        let (manager, pipeline_tx, pipeline_entity_guard) =
            build_test_manager(pipeline_capacity, control_senders.clone());
        (
            manager,
            pipeline_tx,
            control_senders,
            control_receivers,
            nodes,
            pipeline_entity_guard,
        )
    }

    fn setup_test_manager<PData: Clone>() -> (
        RuntimeCtrlMsgManager<PData>,
        crate::control::RuntimeCtrlMsgSender<PData>,
        HashMap<usize, Receiver<NodeControlMsg<PData>>>,
        Vec<NodeId>,
        crate::entity_context::PipelineEntityScope,
    ) {
        let (
            manager,
            pipeline_tx,
            _control_senders,
            control_receivers,
            nodes,
            pipeline_entity_guard,
        ) = setup_test_manager_with_capacities(10, 10);
        (
            manager,
            pipeline_tx,
            control_receivers,
            nodes,
            pipeline_entity_guard,
        )
    }

    /// Validates the core timer workflow:
    /// 1. StartTimer message scheduling
    /// 2. Timer expiration after specified duration
    /// 3. TimerTick message delivery to the correct node
    /// 4. Automatic timer recurrence (key feature of the manager)
    #[tokio::test]
    async fn test_run_start_timer_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<()>();

                let node = nodes.first().expect("ok");
                let duration = Duration::from_millis(100);

                // Start the manager in the background using spawn_local (not Send)
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Send StartTimer message to schedule a recurring timer
                let start_msg = RuntimeControlMsg::StartTimer {
                    node_id: node.index,
                    duration,
                };
                pipeline_tx.send(start_msg).await.unwrap();

                // Wait for the timer to expire and verify TimerTick delivery
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let tick_result =
                    timeout(Duration::from_millis(200), async { receiver.recv().await }).await;

                assert!(
                    tick_result.is_ok(),
                    "Should receive TimerTick within timeout"
                );
                match tick_result.unwrap() {
                    Ok(NodeControlMsg::TimerTick {}) => {
                        // Success - received expected TimerTick
                    }
                    Ok(other) => panic!("Expected TimerTick, got {other:?}"),
                    Err(e) => panic!("Failed to receive message: {e:?}"),
                }

                // Verify automatic recurrence - should get another tick
                let second_tick_result =
                    timeout(Duration::from_millis(150), async { receiver.recv().await }).await;

                assert!(
                    second_tick_result.is_ok(),
                    "Should receive second TimerTick for recurring timer"
                );

                // Clean shutdown
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "".to_owned(),
                    })
                    .await
                    .unwrap();
                drop(pipeline_tx);
                let _ = timeout(Duration::from_millis(100), manager_handle).await;
            })
            .await;
    }

    /// Validates that:
    /// 1. A timer can be started normally
    /// 2. CancelTimer messages properly prevent timer execution
    /// 3. No TimerTick messages are delivered for canceled timers
    /// 4. The cancellation is processed before the timer would naturally expire
    #[tokio::test]
    async fn test_run_cancel_timer_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<()>();

                let node = nodes.first().expect("ok");
                let duration = Duration::from_millis(100);

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Schedule a timer
                let start_msg = RuntimeControlMsg::StartTimer {
                    node_id: node.index,
                    duration,
                };
                pipeline_tx.send(start_msg).await.unwrap();

                // Immediately cancel the timer before it expires
                let cancel_msg = RuntimeControlMsg::CancelTimer {
                    node_id: node.index,
                };
                pipeline_tx.send(cancel_msg).await.unwrap();

                // Wait and verify no TimerTick is received (timeout expected)
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let tick_result =
                    timeout(Duration::from_millis(200), async { receiver.recv().await }).await;

                assert!(
                    tick_result.is_err(),
                    "Should not receive TimerTick for canceled timer"
                );

                // Clean shutdown
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "".to_owned(),
                    })
                    .await
                    .unwrap();
                drop(pipeline_tx);
                let _ = timeout(Duration::from_millis(100), manager_handle).await;
            })
            .await;
    }

    /// Validates the manager's ability to handle multiple timers simultaneously:
    /// 1. Multiple nodes can have active timers concurrently
    /// 2. Each timer fires independently based on its own duration
    /// 3. Timer messages are delivered to the correct recipients
    #[tokio::test]
    async fn test_run_multiple_timers_integration() {
        let local = LocalSet::new();

        local.run_until(async {
            let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                setup_test_manager::<()>();

            let node1 = nodes.first().expect("ok");
            let node2 = nodes.get(1).expect("ok");
            let duration1 = Duration::from_millis(80);  // Shorter - should fire first
            let duration2 = Duration::from_millis(120); // Longer - should fire second

            // Start the manager in the background
            let manager_handle = tokio::task::spawn_local(async move {
                manager.run().await
            });

            // Schedule timers for both nodes
            let start_msg1 = RuntimeControlMsg::StartTimer {
                node_id: node1.index,
                duration: duration1,
            };
            let start_msg2 = RuntimeControlMsg::StartTimer {
                node_id: node2.index,
                duration: duration2,
            };

            pipeline_tx.send(start_msg1).await.unwrap();
            pipeline_tx.send(start_msg2).await.unwrap();

            // Extract receivers for both nodes
            let mut receiver1 = control_receivers.remove(&node1.index).unwrap();
            let mut receiver2 = control_receivers.remove(&node2.index).unwrap();

            // Use select! to handle whichever timer fires first, with overall timeout
            let mut node1_received = false;
            let mut node2_received = false;
            let start_time = Instant::now();

            // Wait for both timers to fire (within a reasonable timeout)
            while (!node1_received || !node2_received) && start_time.elapsed() < Duration::from_millis(300) {
                tokio::select! {
                    // Node1 timer tick
                    result1 = receiver1.recv(), if !node1_received => {
                        match result1 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node1_received = true;
                                // Verify node1 fired within expected timeframe (should be ~80ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(60) && elapsed <= Duration::from_millis(140),
                                       "Node1 timer should fire around 80ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node1, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node1: {e:?}"),
                        }
                    }

                    // Node2 timer tick
                    result2 = receiver2.recv(), if !node2_received => {
                        match result2 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node2_received = true;
                                // Verify node2 fired within expected timeframe (should be ~120ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(100) && elapsed <= Duration::from_millis(180),
                                       "Node2 timer should fire around 120ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node2, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node2: {e:?}"),
                        }
                    }

                    // Timeout protection
                    _ = tokio::time::sleep(Duration::from_millis(50)) => {
                        // Continue the loop - this prevents infinite blocking
                    }
                }
            }

            // Verify both timers fired
            assert!(node1_received, "Node1 should have received TimerTick");
            assert!(node2_received, "Node2 should have received TimerTick");

            // Clean shutdown
            pipeline_tx.send(RuntimeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(1),
                reason: "".to_owned()
            }).await.unwrap();
            drop(pipeline_tx);
            let _ = timeout(Duration::from_millis(100), manager_handle).await;
        }).await;
    }

    /// Validates that starting a new timer for an existing node properly replaces
    /// the old timer rather than creating duplicate timers:
    /// 1. Initial timer is scheduled with a longer duration
    /// 2. Replacement timer is scheduled with shorter duration
    /// 3. The timer fires based on the new (shorter) duration, not the original
    /// 4. This tests the outdated timer detection logic in the run() method
    #[tokio::test]
    async fn test_run_timer_replacement_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<()>();

		let node = nodes.first().expect("ok");
                let first_duration = Duration::from_millis(150); // Original (longer)
                let second_duration = Duration::from_millis(80); // Replacement (shorter)

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move {
                    manager.run().await
                });

                // Schedule initial timer
                let start_msg1 = RuntimeControlMsg::StartTimer {
                    node_id: node.index,
                    duration: first_duration,
                };
                pipeline_tx.send(start_msg1).await.unwrap();

                // Wait a bit, then replace with a shorter timer
                tokio::time::sleep(Duration::from_millis(20)).await;
                let start_msg2 = RuntimeControlMsg::StartTimer {
                    node_id: node.index,
                    duration: second_duration,
                };
                pipeline_tx.send(start_msg2).await.unwrap();

                // Measure timing to verify the replacement worked
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let start_time = Instant::now();

                let tick_result =
                    timeout(Duration::from_millis(200), async { receiver.recv().await }).await;

                let elapsed = start_time.elapsed();

                assert!(tick_result.is_ok(), "Should receive TimerTick");
                // Should fire approximately after second_duration (80ms), not first_duration (150ms)
                // Allow some tolerance for timing variations in test environment
                assert!(
                    elapsed >= Duration::from_millis(70) && elapsed <= Duration::from_millis(130),
                    "Timer should fire based on second duration (~80ms), but fired after {elapsed:?}"
                );

                // Clean shutdown
                pipeline_tx.send(RuntimeControlMsg::Shutdown {
                    deadline: Instant::now() + Duration::from_secs(1),
                    reason: "".to_owned()
                }).await.unwrap();
                drop(pipeline_tx);
                let _ = timeout(Duration::from_millis(100), manager_handle).await;
            })
            .await;
    }

    /// Validates that the manager responds properly to shutdown requests:
    /// 1. The run() method terminates cleanly when receiving a Shutdown message
    /// 2. No hanging tasks or resource leaks
    /// 3. Shutdown completes within reasonable time
    #[tokio::test]
    async fn test_run_shutdown_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, _control_receivers, _, _pipeline_entity_guard) =
                    setup_test_manager::<()>();

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Send shutdown message
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "".to_owned(),
                    })
                    .await
                    .unwrap();

                // Drop the sender to allow the manager to exit draining mode.
                // After shutdown, the manager continues running to allow cleanup messages
                // until all senders are dropped.
                drop(pipeline_tx);

                // Manager should terminate cleanly within timeout
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    /// Validates that a StartTelemetryTimer results in a CollectTelemetry control message delivered to the node.
    #[tokio::test]
    async fn test_run_start_telemetry_timer_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<()>();

                let node = nodes.first().expect("ok");
                let duration = Duration::from_millis(60);

                // Start the manager in the background using spawn_local (not Send)
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Send StartTelemetryTimer message to schedule a recurring telemetry timer
                let start_msg = RuntimeControlMsg::StartTelemetryTimer {
                    node_id: node.index,
                    duration,
                };
                pipeline_tx.send(start_msg).await.unwrap();

                // Wait for the telemetry timer to expire and verify CollectTelemetry delivery
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let telemetry_result =
                    timeout(Duration::from_millis(200), async { receiver.recv().await }).await;

                assert!(
                    telemetry_result.is_ok(),
                    "Should receive CollectTelemetry within timeout"
                );
                match telemetry_result.unwrap() {
                    Ok(NodeControlMsg::CollectTelemetry { .. }) => {
                        // Success - received expected CollectTelemetry
                    }
                    Ok(other) => panic!("Expected CollectTelemetry, got {other:?}"),
                    Err(e) => panic!("Failed to receive message: {e:?}"),
                }

                // Clean shutdown
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "".to_owned(),
                    })
                    .await
                    .unwrap();
                drop(pipeline_tx);
                let _ = timeout(Duration::from_millis(100), manager_handle).await;
            })
            .await;
    }

    /// Validates error resilience when the manager tries to send TimerTick
    /// to a node that doesn't have a registered control sender:
    /// 1. Timer can be scheduled for non-existent node
    /// 2. Manager doesn't crash when trying to send to missing sender
    /// 3. Manager continues operating normally after the error
    /// 4. This tests the defensive programming in the timer expiration logic
    #[tokio::test]
    async fn test_run_no_control_sender_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (pipeline_tx, pipeline_rx) = runtime_ctrl_msg_channel(10);
                // Create a dummy MetricsReporter for testing
                let metrics_system = otap_df_telemetry::InternalTelemetrySystem::default();
                let metrics_reporter = metrics_system.reporter();
                let observed_state_store = ObservedStateStore::new(
                    &ObservedStateSettings::default(),
                    metrics_system.registry(),
                );
                let pipeline_group_id: PipelineGroupId = Default::default();
                let pipeline_id: PipelineId = Default::default();
                let core_id = 0;
                let pipeline_key = DeployedPipelineKey {
                    pipeline_group_id: pipeline_group_id.clone(),
                    pipeline_id: pipeline_id.clone(),
                    core_id,
                };
                let thread_id = 0;
                let controller_context = ControllerContext::new(metrics_system.registry());
                let pipeline_context = PipelineContext::new(
                    controller_context,
                    pipeline_group_id.clone(),
                    pipeline_id.clone(),
                    core_id,
                    1, // num_cores
                    thread_id,
                );
                let pipeline_entity_key = pipeline_context.register_pipeline_entity();
                let _pipeline_entity_guard = crate::entity_context::set_pipeline_entity_key(
                    pipeline_context.metrics_registry(),
                    pipeline_entity_key,
                );

                // Create manager with empty control_senders map (no registered nodes)
                let manager = RuntimeCtrlMsgManager::<()>::new(
                    pipeline_key,
                    pipeline_context,
                    pipeline_rx,
                    ControlSenders::new(),
                    observed_state_store.reporter(SendPolicy::default()),
                    metrics_reporter,
                    TelemetryPolicy::default(),
                    Vec::new(),
                    empty_node_metric_handles(),
                );
                let duration = Duration::from_millis(50);

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Send StartTimer for node with no control sender
                let start_msg = RuntimeControlMsg::StartTimer {
                    node_id: 1234,
                    duration,
                };
                pipeline_tx.send(start_msg).await.unwrap();

                // Wait for timer to expire - manager should handle this gracefully
                // (no way to verify TimerTick delivery since no receiver exists)
                tokio::time::sleep(Duration::from_millis(100)).await;

                // Manager should still be responsive for shutdown
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "".to_owned(),
                    })
                    .await
                    .unwrap();

                // Drop the sender to let the manager exit draining mode
                drop(pipeline_tx);

                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(
                    shutdown_result.is_ok(),
                    "Manager should handle missing control sender gracefully"
                );
            })
            .await;
    }

    /// Validates that timers fire in the correct chronological order regardless
    /// of the order they were registered:
    /// 1. Timers are registered in non-chronological order
    /// 2. They fire in chronological order (shortest duration first)
    /// 3. This tests the BinaryHeap priority queue implementation
    /// 4. Uses select! to handle timers in any order while validating proper sequencing
    #[tokio::test]
    async fn test_run_timer_ordering_integration() {
        let local = LocalSet::new();

        local.run_until(async {
            let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                setup_test_manager::<()>();

            // Use different durations to test timer ordering
            let node1 = nodes.first().expect("ok");
            let node2 = nodes.get(1).expect("ok");
            let node3 = nodes.get(2).expect("ok");

            // Start the manager in the background
            let manager_handle = tokio::task::spawn_local(async move {
                manager.run().await
            });

            // Send timers in non-chronological order to test priority queue
            let start_msg1 = RuntimeControlMsg::StartTimer {
                node_id: node1.index,
                duration: Duration::from_millis(120), // Should fire third
            };
            let start_msg2 = RuntimeControlMsg::StartTimer {
                node_id: node2.index,
                duration: Duration::from_millis(60),  // Should fire first
            };
            let start_msg3 = RuntimeControlMsg::StartTimer {
                node_id: node3.index,
                duration: Duration::from_millis(90),  // Should fire second
            };

            pipeline_tx.send(start_msg1).await.unwrap();
            pipeline_tx.send(start_msg2).await.unwrap();
            pipeline_tx.send(start_msg3).await.unwrap();

            let mut receiver1 = control_receivers.remove(&node1.index).unwrap();
            let mut receiver2 = control_receivers.remove(&node2.index).unwrap();
            let mut receiver3 = control_receivers.remove(&node3.index).unwrap();

            // Track which timers have fired and in what order
            let mut node1_received = false;
            let mut node2_received = false;
            let mut node3_received = false;
            let mut firing_order = Vec::new();
            let start_time = Instant::now();

            // Use select! to handle whichever timer fires first, validating the order
            while (!node1_received || !node2_received || !node3_received) && start_time.elapsed() < Duration::from_millis(400) {
                tokio::select! {
                    // Node1 timer tick (120ms - should be last)
                    result1 = receiver1.recv(), if !node1_received => {
                        match result1 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node1_received = true;
                                firing_order.push((node1.index, start_time.elapsed()));
                                // Verify node1 fired within expected timeframe (should be ~120ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(100) && elapsed <= Duration::from_millis(180),
                                       "Node1 timer should fire around 120ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node1, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node1: {e:?}"),
                        }
                    }

                    // Node2 timer tick (60ms - should be first)
                    result2 = receiver2.recv(), if !node2_received => {
                        match result2 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node2_received = true;
                                firing_order.push((node2.index, start_time.elapsed()));
                                // Verify node2 fired within expected timeframe (should be ~60ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(40) && elapsed <= Duration::from_millis(100),
                                       "Node2 timer should fire around 60ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node2, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node2: {e:?}"),
                        }
                    }

                    // Node3 timer tick (90ms - should be second)
                    result3 = receiver3.recv(), if !node3_received => {
                        match result3 {
                            Ok(NodeControlMsg::TimerTick {}) => {
                                node3_received = true;
                                firing_order.push((node3.index, start_time.elapsed()));
                                // Verify node3 fired within expected timeframe (should be ~90ms)
                                let elapsed = start_time.elapsed();
                                assert!(elapsed >= Duration::from_millis(70) && elapsed <= Duration::from_millis(130),
                                       "Node3 timer should fire around 90ms, but fired after {elapsed:?}");
                            }
                            Ok(other) => panic!("Expected TimerTick for node3, got {other:?}"),
                            Err(e) => panic!("Failed to receive message for node3: {e:?}"),
                        }
                    }

                    // Timeout protection
                    _ = tokio::time::sleep(Duration::from_millis(30)) => {
                        // Continue the loop - this prevents infinite blocking
                    }
                }
            }

            // Verify all timers fired
            assert!(node1_received, "Node1 should have received TimerTick");
            assert!(node2_received, "Node2 should have received TimerTick");
            assert!(node3_received, "Node3 should have received TimerTick");

            // Verify the firing order is correct (node2 first, node3 second, node1 third)
            // Sort by elapsed time to get the actual firing order
            firing_order.sort_by_key(|&(_, elapsed)| elapsed);

            assert_eq!(firing_order.len(), 3, "Should have received exactly 3 timer events");
            assert_eq!(firing_order[0].0, node2.index, "Node2 (60ms) should fire first");
            assert_eq!(firing_order[1].0, node3.index, "Node3 (90ms) should fire second");
            assert_eq!(firing_order[2].0, node1.index, "Node1 (120ms) should fire third");

            // Clean shutdown
            pipeline_tx.send(RuntimeControlMsg::Shutdown {
                deadline: Instant::now() + Duration::from_secs(1),
                reason: "".to_owned()
            }).await.unwrap();
            drop(pipeline_tx);
            let _ = timeout(Duration::from_millis(100), manager_handle).await;
        }).await;
    }

    /// Validates that the RuntimeCtrlMsgManager is created with correct
    /// initial state for all internal data structures.
    #[tokio::test]
    async fn test_manager_creation() {
        let (manager, _pipeline_tx, _control_receivers, _, _pipeline_entity_guard) =
            setup_test_manager::<()>();

        // Verify manager is created with correct initial state
        assert_eq!(
            manager.tick_timers.timers.len(),
            0,
            "Timer queue should be empty initially"
        );
        assert_eq!(
            manager.tick_timers.timer_states.len(),
            0,
            "Timer states map should be empty initially"
        );
        let tick_count = manager.test_tick_count();
        assert_eq!(tick_count, 0, "Tick timer queue should be empty initially");

        let telemetry_count = manager.test_telemetry_count();
        assert_eq!(
            telemetry_count, 0,
            "Telemetry timer queue should be empty initially"
        );

        assert_eq!(
            manager.test_control_senders_len(),
            3,
            "Should have 3 mock control senders"
        );
    }

    /// Validates the internal timer priority queue data structure:
    /// 1. Timers are stored in a min-heap (earliest expiration first)
    /// 2. BinaryHeap with Reverse wrapper creates correct ordering
    /// 3. Multiple timers are ordered correctly regardless of insertion order
    ///
    /// This is a unit test of the data structure, separate from the run() method.
    #[tokio::test]
    async fn test_timer_heap_ordering() {
        let (mut manager, _pipeline_tx, _control_receivers, nodes, _pipeline_entity_guard) =
            setup_test_manager::<()>();

        let node1 = nodes.first().expect("ok");
        let node2 = nodes.get(1).expect("ok");
        let node3 = nodes.get(2).expect("ok");

        let now = Instant::now();
        let when1 = now + Duration::from_millis(300); // Latest
        let when2 = now + Duration::from_millis(100); // Earliest - should be popped first
        let when3 = now + Duration::from_millis(200); // Middle

        // Add timers in non-chronological order to test heap behavior
        manager.test_push_tick_heap(when1, node1.index);
        manager.test_push_tick_heap(when2, node2.index);
        manager.test_push_tick_heap(when3, node3.index);

        // Verify heap maintains correct size
        assert_eq!(
            manager.test_tick_heap_len(),
            3,
            "All timers should be in the heap"
        );

        // Pop timers and verify they come out in chronological order (min-heap behavior)
        if let Some(Reverse((first_when, first_node))) = manager.tick_timers.timers.pop() {
            assert_eq!(first_when, when2, "Earliest timer should be popped first");
            assert_eq!(
                first_node, node2.index,
                "Correct node should be associated with earliest timer"
            );
        }

        if let Some((second_when, second_node)) = manager.test_pop_tick_heap() {
            assert_eq!(second_when, when3, "Middle timer should be popped second");
            assert_eq!(
                second_node, node3.index,
                "Correct node should be associated with middle timer"
            );
        }

        if let Some((third_when, third_node)) = manager.test_pop_tick_heap() {
            assert_eq!(third_when, when1, "Latest timer should be popped last");
            assert_eq!(
                third_node, node1.index,
                "Correct node should be associated with latest timer"
            );
        }
    }

    #[test]
    fn test_delayed_data_heap_ordering() {
        let now = Instant::now();
        let mut delayed_heap = BinaryHeap::new();

        let data1 = Box::new("data1".to_string());
        let data2 = Box::new("data2".to_string());
        let data3 = Box::new("data3".to_string());

        let delayed1 = Delayed {
            when: now + Duration::from_millis(300),
            node_id: 1,
            data: data1.clone(),
        };
        let delayed2 = Delayed {
            when: now + Duration::from_millis(100),
            node_id: 2,
            data: data2.clone(),
        };
        let delayed3 = Delayed {
            when: now + Duration::from_millis(200),
            node_id: 3,
            data: data3.clone(),
        };

        // Insert in non-chronological order to test heap behavior
        delayed_heap.push(delayed1);
        delayed_heap.push(delayed2);
        delayed_heap.push(delayed3);

        assert_eq!(delayed_heap.len(), 3,);

        let first = delayed_heap.pop().expect("should exist");
        assert_eq!(first.when, now + Duration::from_millis(100));
        assert_eq!(first.node_id, 2,);
        assert_eq!(*first.data, *data2);

        let second = delayed_heap.pop().expect("should exist");
        assert_eq!(second.when, now + Duration::from_millis(200));
        assert_eq!(second.node_id, 3,);
        assert_eq!(*second.data, *data3);

        let third = delayed_heap.pop().expect("should exist");
        assert_eq!(third.when, now + Duration::from_millis(300));
        assert_eq!(third.node_id, 1,);
        assert_eq!(*third.data, *data1);

        assert!(delayed_heap.is_empty());
    }

    #[tokio::test]
    async fn test_delay_data_integration() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<String>();

                let node = nodes.first().expect("ok");
                let delay_duration = Duration::from_millis(100);
                let test_data = Box::new("test_delayed_data".to_string());
                let delay_time = Instant::now() + delay_duration;

                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                let delay_msg = RuntimeControlMsg::DelayData {
                    node_id: node.index,
                    when: delay_time,
                    data: test_data.clone(),
                };
                pipeline_tx.send(delay_msg).await.unwrap();

                // Wait for delayed data to be delivered
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let delayed_result = async { receiver.recv().await };

                match delayed_result.await {
                    Ok(NodeControlMsg::DelayedData { when, data }) => {
                        assert_eq!(*data, *test_data);
                        assert_eq!(when, delay_time);
                    }
                    Ok(other) => panic!("Expected DelayedData, got {other:?}"),
                    Err(e) => panic!("Failed to receive message: {e:?}"),
                }

                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "".to_owned(),
                    })
                    .await
                    .unwrap();

                // Drop the sender to let the manager exit draining mode
                drop(pipeline_tx);

                let _ = manager_handle.await;
            })
            .await;
    }

    #[tokio::test]
    async fn test_due_timer_tick_progress_under_runtime_ctrl_burst() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (
                    mut manager,
                    pipeline_tx,
                    _control_senders,
                    mut control_receivers,
                    nodes,
                    _pipeline_entity_guard,
                ) = setup_test_manager_with_capacities::<String>(128, 10);

                let noisy_node = nodes[0].clone();
                let target = nodes[1].clone();
                manager
                    .tick_timers
                    .start(target.index, Duration::from_millis(1));
                tokio::time::sleep(Duration::from_millis(5)).await;

                for _ in 0..96 {
                    pipeline_tx
                        .send(RuntimeControlMsg::StartTimer {
                            node_id: noisy_node.index,
                            duration: Duration::from_secs(60),
                        })
                        .await
                        .unwrap();
                }

                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                let mut receiver = control_receivers.remove(&target.index).unwrap();
                let msg = timeout(Duration::from_millis(500), receiver.recv())
                    .await
                    .expect("TimerTick should make progress under runtime control burst")
                    .expect("target control channel should stay open");
                assert!(matches!(msg, NodeControlMsg::TimerTick {}));

                drop(pipeline_tx);
                let shutdown_result = timeout(Duration::from_millis(200), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    #[tokio::test]
    async fn test_due_collect_telemetry_progress_under_runtime_ctrl_burst() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (
                    mut manager,
                    pipeline_tx,
                    _control_senders,
                    mut control_receivers,
                    nodes,
                    _pipeline_entity_guard,
                ) = setup_test_manager_with_capacities::<String>(128, 10);

                let noisy_node = nodes[0].clone();
                let target = nodes[1].clone();
                manager
                    .telemetry_timers
                    .start(target.index, Duration::from_millis(1));
                tokio::time::sleep(Duration::from_millis(5)).await;

                for _ in 0..96 {
                    pipeline_tx
                        .send(RuntimeControlMsg::StartTimer {
                            node_id: noisy_node.index,
                            duration: Duration::from_secs(60),
                        })
                        .await
                        .unwrap();
                }

                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                let mut receiver = control_receivers.remove(&target.index).unwrap();
                let msg = timeout(Duration::from_millis(500), receiver.recv())
                    .await
                    .expect("CollectTelemetry should make progress under runtime control burst")
                    .expect("target control channel should stay open");
                assert!(matches!(msg, NodeControlMsg::CollectTelemetry { .. }));

                drop(pipeline_tx);
                let shutdown_result = timeout(Duration::from_millis(200), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    #[tokio::test]
    async fn test_due_delayed_data_progress_under_runtime_ctrl_burst() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (
                    mut manager,
                    pipeline_tx,
                    _control_senders,
                    mut control_receivers,
                    nodes,
                    _pipeline_entity_guard,
                ) = setup_test_manager_with_capacities::<String>(128, 10);

                let noisy_node = nodes[0].clone();
                let target = nodes[1].clone();
                manager.delayed_data.push(Delayed {
                    node_id: target.index,
                    when: Instant::now(),
                    data: Box::new("burst_delayed".to_owned()),
                });

                for _ in 0..96 {
                    pipeline_tx
                        .send(RuntimeControlMsg::StartTimer {
                            node_id: noisy_node.index,
                            duration: Duration::from_secs(60),
                        })
                        .await
                        .unwrap();
                }

                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                let mut receiver = control_receivers.remove(&target.index).unwrap();
                let msg = timeout(Duration::from_millis(500), receiver.recv())
                    .await
                    .expect("DelayedData should make progress under runtime control burst")
                    .expect("target control channel should stay open");
                assert!(matches!(
                    msg,
                    NodeControlMsg::DelayedData { ref data, .. } if **data == "burst_delayed"
                ));

                drop(pipeline_tx);
                let shutdown_result = timeout(Duration::from_millis(200), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    /// Validates that nodes can send cleanup messages (e.g., CancelTimer) after
    /// shutdown has been initiated.
    ///
    /// After receiving a Shutdown message, the manager enters "draining" mode where
    /// it continues to process cleanup messages until all senders drop their channel.
    /// This allows processors and exporters to cancel timers, etc. during cleanup.
    #[tokio::test]
    async fn test_shutdown_allows_nodes_to_send_cleanup_messages() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<()>();

                let node = nodes.first().expect("ok");

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Simulate a node starting a timer (like processors do for telemetry)
                let start_msg = RuntimeControlMsg::StartTimer {
                    node_id: node.index,
                    duration: Duration::from_secs(1), // Long duration - won't fire during test
                };
                pipeline_tx.send(start_msg).await.unwrap();

                // Small delay to ensure timer is registered
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Send shutdown - the manager enters draining mode but continues running
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "test shutdown".to_owned(),
                    })
                    .await
                    .unwrap();

                // After shutdown, nodes should still be able to send cleanup messages
                // (e.g., CancelTelemetryTimer). In a real pipeline, processors try to
                // cancel their telemetry timers when their message channel closes.
                // The manager is still running in draining mode, so this should succeed.
                let cancel_result = pipeline_tx
                    .send(RuntimeControlMsg::CancelTimer {
                        node_id: node.index,
                    })
                    .await;

                // The channel should remain open until all nodes have completed.
                assert!(
                    cancel_result.is_ok(),
                    "Nodes should be able to send control messages during cleanup, \
                     but the channel was closed prematurely"
                );

                // Drop the sender to let the manager exit draining mode
                drop(pipeline_tx);

                // Manager should terminate cleanly
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(
                    shutdown_result.is_ok(),
                    "Manager should shutdown cleanly after cleanup"
                );

                // Cleanup - drain the control receiver so it doesn't complain
                let mut receiver = control_receivers.remove(&node.index).unwrap();
                while receiver.recv().await.is_ok() {}
            })
            .await;
    }

    /// Validates that duplicate shutdown messages are ignored during draining.
    ///
    /// Once the manager enters draining mode, subsequent Shutdown messages should
    /// be silently ignored to prevent re-triggering shutdown logic.
    #[tokio::test]
    async fn test_duplicate_shutdown_ignored_during_draining() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, _control_receivers, _nodes, _pipeline_entity_guard) =
                    setup_test_manager::<()>();

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Send first shutdown
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "first shutdown".to_owned(),
                    })
                    .await
                    .unwrap();

                // Small delay to ensure first shutdown is processed
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Send duplicate shutdown - should be ignored
                let duplicate_result = pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "duplicate shutdown".to_owned(),
                    })
                    .await;

                // Channel should still be open (manager didn't crash)
                assert!(
                    duplicate_result.is_ok(),
                    "Duplicate shutdown should be accepted (and ignored)"
                );

                // Drop the sender to let the manager exit draining mode
                drop(pipeline_tx);

                // Manager should terminate cleanly
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    /// Validates that DeliverAck messages are processed during draining.
    ///
    /// Ack messages should still be delivered to nodes during shutdown so that
    /// in-flight acknowledgments can complete.
    #[tokio::test]
    async fn test_deliver_ack_during_draining() {
        use crate::control::AckMsg;

        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, _control_receivers, _nodes, _pipeline_entity_guard) =
                    setup_test_manager::<String>();
                let (return_tx, return_rx) = pipeline_result_msg_channel(10);
                let dispatcher = PipelineResultMsgDispatcher::new(
                    return_rx,
                    ControlSenders::new(),
                    empty_node_metric_handles(),
                );

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });
                let dispatcher_handle =
                    tokio::task::spawn_local(async move { dispatcher.run().await });

                // Send shutdown first
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "test shutdown".to_owned(),
                    })
                    .await
                    .unwrap();

                // Small delay to ensure shutdown is processed and we're in draining mode
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Send DeliverAck during draining - should be processed
                let ack = AckMsg::new("ack_data".to_owned());
                return_tx
                    .send(PipelineResultMsg::DeliverAck { ack })
                    .await
                    .unwrap();

                // String PData has no context stack, so unwind_ack is a no-op.
                // Just verify the controller processes it without crashing.

                // Drop the sender to let the manager exit draining mode
                drop(return_tx);
                drop(pipeline_tx);

                let dispatcher_result =
                    timeout(Duration::from_millis(100), dispatcher_handle).await;
                assert!(
                    dispatcher_result.is_ok(),
                    "Return dispatcher should shutdown cleanly"
                );

                // Manager should terminate cleanly
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    /// Validates that DeliverNack messages are processed during draining.
    ///
    /// Nack messages should still be delivered to nodes during shutdown so that
    /// in-flight negative acknowledgments can complete.
    #[tokio::test]
    async fn test_deliver_nack_during_draining() {
        use crate::control::NackMsg;

        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, _control_receivers, _nodes, _pipeline_entity_guard) =
                    setup_test_manager::<String>();
                let (return_tx, return_rx) = pipeline_result_msg_channel(10);
                let dispatcher = PipelineResultMsgDispatcher::new(
                    return_rx,
                    ControlSenders::new(),
                    empty_node_metric_handles(),
                );

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });
                let dispatcher_handle =
                    tokio::task::spawn_local(async move { dispatcher.run().await });

                // Send shutdown first
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "test shutdown".to_owned(),
                    })
                    .await
                    .unwrap();

                // Small delay to ensure shutdown is processed and we're in draining mode
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Send DeliverNack during draining - should be processed
                let nack = NackMsg::new("test failure", "nack_data".to_owned());
                return_tx
                    .send(PipelineResultMsg::DeliverNack { nack })
                    .await
                    .unwrap();

                // String PData has no context stack, so unwind_nack is a no-op.
                // Just verify the controller processes it without crashing.

                // Drop the sender to let the manager exit draining mode
                drop(return_tx);
                drop(pipeline_tx);

                let dispatcher_result =
                    timeout(Duration::from_millis(100), dispatcher_handle).await;
                assert!(
                    dispatcher_result.is_ok(),
                    "Return dispatcher should shutdown cleanly"
                );

                // Manager should terminate cleanly
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    /// Validates that the draining deadline is respected.
    ///
    /// If the draining deadline is exceeded, the manager should force shutdown
    /// even if there are still active senders.
    #[tokio::test]
    async fn test_draining_deadline_forces_shutdown() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, _control_receivers, _nodes, _pipeline_entity_guard) =
                    setup_test_manager::<()>();

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Send shutdown with a very short deadline
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_millis(50),
                        reason: "test shutdown with short deadline".to_owned(),
                    })
                    .await
                    .unwrap();

                // Don't drop the sender - the manager should still exit due to deadline
                // Wait for the manager to exit (should happen after ~50ms deadline)
                let shutdown_result = timeout(Duration::from_millis(200), manager_handle).await;

                assert!(
                    shutdown_result.is_ok(),
                    "Manager should shutdown when draining deadline is exceeded, even with active senders"
                );
            })
            .await;
    }

    /// Validates that TimerTick messages are NOT fired during draining.
    ///
    /// When draining, the manager should not fire any new timer ticks - it should
    /// only process messages that help complete in-flight work (like Ack/Nack).
    #[tokio::test]
    async fn test_timer_tick_does_not_fire_during_draining() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<String>();

                let node = nodes.first().expect("ok");

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Start a timer that would fire very soon
                pipeline_tx
                    .send(RuntimeControlMsg::StartTimer {
                        node_id: node.index,
                        duration: Duration::from_millis(10),
                    })
                    .await
                    .unwrap();

                // Small delay to ensure timer is registered
                tokio::time::sleep(Duration::from_millis(5)).await;

                // Send shutdown before the timer fires
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_millis(500),
                        reason: "test shutdown before timer fires".to_owned(),
                    })
                    .await
                    .unwrap();

                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let shutdown = timeout(Duration::from_millis(100), receiver.recv())
                    .await
                    .expect("processor should be shut down during draining")
                    .expect("processor control channel should stay open");
                assert!(
                    matches!(shutdown, NodeControlMsg::Shutdown { .. }),
                    "Processors should receive Shutdown immediately during draining"
                );

                // Wait longer than the timer interval - during draining, timer should NOT fire
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify no TimerTick was received during draining after ingress drain
                let msg = timeout(Duration::from_millis(100), receiver.recv()).await;
                assert!(
                    msg.is_err(),
                    "Should NOT receive TimerTick during draining - timer ticks are suppressed"
                );

                // Drop the sender to let the manager exit draining mode
                drop(pipeline_tx);

                // Manager should terminate cleanly
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    /// Validates that StartTimer messages are ignored during draining.
    ///
    /// When draining, new timer registration requests should be silently ignored
    /// since we don't want to start new work during shutdown.
    #[tokio::test]
    async fn test_start_timer_ignored_during_draining() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<String>();

                let node = nodes.first().expect("ok");

                // Start the manager in the background
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                // Send shutdown first to enter draining mode
                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "test shutdown".to_owned(),
                    })
                    .await
                    .unwrap();

                // Small delay to ensure shutdown is processed
                tokio::time::sleep(Duration::from_millis(10)).await;

                // Try to start a timer during draining - should be ignored
                pipeline_tx
                    .send(RuntimeControlMsg::StartTimer {
                        node_id: node.index,
                        duration: Duration::from_millis(10),
                    })
                    .await
                    .unwrap();

                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let shutdown = timeout(Duration::from_millis(100), receiver.recv())
                    .await
                    .expect("processor should be shut down during draining")
                    .expect("processor control channel should stay open");
                assert!(
                    matches!(shutdown, NodeControlMsg::Shutdown { .. }),
                    "Processors should receive Shutdown immediately during draining"
                );

                // Wait longer than the timer interval
                tokio::time::sleep(Duration::from_millis(50)).await;

                // Verify no TimerTick was received - StartTimer should have been ignored during draining
                let msg = timeout(Duration::from_millis(100), receiver.recv()).await;
                assert!(
                    msg.is_err(),
                    "Should NOT receive TimerTick - StartTimer should be ignored during draining"
                );

                // Drop the sender to let the manager exit draining mode
                drop(pipeline_tx);

                // Manager should terminate cleanly
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    #[tokio::test]
    async fn test_start_telemetry_timer_ignored_during_draining() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<String>();

                let node = nodes.first().expect("ok");
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "test shutdown".to_owned(),
                    })
                    .await
                    .unwrap();

                tokio::time::sleep(Duration::from_millis(10)).await;

                pipeline_tx
                    .send(RuntimeControlMsg::StartTelemetryTimer {
                        node_id: node.index,
                        duration: Duration::from_millis(10),
                    })
                    .await
                    .unwrap();

                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let shutdown = timeout(Duration::from_millis(100), receiver.recv())
                    .await
                    .expect("processor should be shut down during draining")
                    .expect("processor control channel should stay open");
                assert!(matches!(shutdown, NodeControlMsg::Shutdown { .. }));

                let msg = timeout(Duration::from_millis(100), receiver.recv()).await;
                assert!(
                    msg.is_err(),
                    "StartTelemetryTimer should be ignored during draining"
                );

                drop(pipeline_tx);
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    #[tokio::test]
    async fn test_new_delay_data_returned_immediately_during_draining() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<String>();

                let node = nodes.first().expect("ok");
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });

                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "test shutdown".to_owned(),
                    })
                    .await
                    .unwrap();

                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let shutdown = timeout(Duration::from_millis(100), receiver.recv())
                    .await
                    .expect("processor should receive shutdown during draining")
                    .expect("processor control channel should stay open");
                assert!(matches!(shutdown, NodeControlMsg::Shutdown { .. }));

                let original_when = Instant::now() + Duration::from_secs(30);
                pipeline_tx
                    .send(RuntimeControlMsg::DelayData {
                        node_id: node.index,
                        when: original_when,
                        data: Box::new("drain_retry".to_owned()),
                    })
                    .await
                    .unwrap();

                let msg = timeout(Duration::from_millis(100), receiver.recv())
                    .await
                    .expect("DelayedData should be returned immediately during draining")
                    .expect("processor control channel should stay open");

                match msg {
                    NodeControlMsg::DelayedData { when, data } => {
                        assert_eq!(*data, "drain_retry");
                        assert!(
                            when < original_when,
                            "DelayedData should be returned immediately, not at its original wake time"
                        );
                    }
                    other => panic!("Expected DelayedData, got {other:?}"),
                }

                drop(pipeline_tx);
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    #[tokio::test]
    async fn test_queued_delayed_data_flushed_when_draining_begins() {
        let local = LocalSet::new();

        local
            .run_until(async {
                let (manager, pipeline_tx, mut control_receivers, nodes, _pipeline_entity_guard) =
                    setup_test_manager::<String>();

                let node = nodes.first().expect("ok");
                let original_when = Instant::now() + Duration::from_secs(30);
                pipeline_tx
                    .send(RuntimeControlMsg::DelayData {
                        node_id: node.index,
                        when: original_when,
                        data: Box::new("queued_retry".to_owned()),
                    })
                    .await
                    .unwrap();

                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });
                tokio::time::sleep(Duration::from_millis(10)).await;

                pipeline_tx
                    .send(RuntimeControlMsg::Shutdown {
                        deadline: Instant::now() + Duration::from_secs(1),
                        reason: "test shutdown".to_owned(),
                    })
                    .await
                    .unwrap();

                let mut receiver = control_receivers.remove(&node.index).unwrap();
                let msg = timeout(Duration::from_millis(100), receiver.recv())
                    .await
                    .expect("Queued delayed data should flush when draining begins")
                    .expect("processor control channel should stay open");
                match msg {
                    NodeControlMsg::DelayedData { when, data } => {
                        assert_eq!(*data, "queued_retry");
                        assert!(
                            when < original_when,
                            "Queued delayed data should be flushed immediately during shutdown"
                        );
                    }
                    other => panic!("Expected DelayedData, got {other:?}"),
                }

                let shutdown = timeout(Duration::from_millis(100), receiver.recv())
                    .await
                    .expect("processor should receive shutdown after delayed-data flush")
                    .expect("processor control channel should stay open");
                assert!(matches!(shutdown, NodeControlMsg::Shutdown { .. }));

                drop(pipeline_tx);
                let shutdown_result = timeout(Duration::from_millis(100), manager_handle).await;
                assert!(shutdown_result.is_ok(), "Manager should shutdown cleanly");
            })
            .await;
    }

    /// A test PData type that carries a real frame stack, allowing the
    /// controller's `unwind_ack`/`unwind_nack` to pop frames and record metrics.
    #[derive(Debug, Clone)]
    struct TestPData {
        frames: Vec<Frame>,
    }

    impl TestPData {
        fn new() -> Self {
            Self { frames: Vec::new() }
        }

        fn push_frame(&mut self, frame: Frame) {
            self.frames.push(frame);
        }
    }

    impl Unwindable for TestPData {
        fn has_frames(&self) -> bool {
            !self.frames.is_empty()
        }
        fn pop_frame(&mut self) -> Option<Frame> {
            self.frames.pop()
        }
        fn drop_payload(&mut self) {}
    }

    impl crate::ReceivedAtNode for TestPData {
        fn received_at_node(&mut self, _node_id: usize, _node_interests: Interests) {}
    }

    /// Labels for identifying metric set snapshots by their MetricSetKey.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    enum MetricLabel {
        RecvProduced,
        ProcConsumed,
        ProcProduced,
        ExpConsumed,
    }

    /// Return value from setup_test_manager_with_metrics.
    struct MetricsTestHarness {
        manager: RuntimeCtrlMsgManager<TestPData>,
        metrics_reporter: MetricsReporter,
        pipeline_tx: crate::control::RuntimeCtrlMsgSender<TestPData>,
        nodes: Vec<NodeId>,
        _guard: crate::entity_context::PipelineEntityScope,
        snapshot_rx: flume::Receiver<MetricSetSnapshot>,
        key_labels: HashMap<MetricSetKey, MetricLabel>,
        node_metric_handles: Rc<RefCell<Vec<Option<NodeMetricHandles>>>>,
    }

    /// Helper: create a manager with `NodeMetricHandles` wired up for a 3-node
    /// pipeline, using a test MetricsReporter whose snapshots can be collected
    /// after shutdown.
    ///
    /// node0 = receiver (no input, 1 output)
    /// node1 = processor (1 input, 1 output)
    /// node2 = exporter  (1 input, no outputs)
    fn setup_test_manager_with_metrics() -> MetricsTestHarness {
        let (pipeline_tx, pipeline_rx) = runtime_ctrl_msg_channel(10);
        let mut control_senders = ControlSenders::new();

        let nodes = test_nodes(vec!["receiver", "processor", "exporter"]);
        let node_types = [NodeType::Receiver, NodeType::Processor, NodeType::Exporter];
        for (node, nt) in nodes.iter().zip(node_types.iter()) {
            let (sender, _receiver) = create_mock_control_sender();
            control_senders.register(node.clone(), *nt, sender);
        }

        let metrics_system = otap_df_telemetry::InternalTelemetrySystem::default();
        // Create a reporter with a receiver so tests can collect snapshots.
        let (snapshot_rx, metrics_reporter) = MetricsReporter::create_new_and_receiver(64);
        let observed_state_store =
            ObservedStateStore::new(&ObservedStateSettings::default(), metrics_system.registry());
        let pipeline_group_id: PipelineGroupId = Default::default();
        let pipeline_id: PipelineId = Default::default();
        let controller_context = ControllerContext::new(metrics_system.registry());
        let pipeline_context = PipelineContext::new(
            controller_context,
            pipeline_group_id.clone(),
            pipeline_id.clone(),
            0,
            1,
            0,
        );

        let pipeline_entity_key = pipeline_context.register_pipeline_entity();
        let pipeline_entity_guard = crate::entity_context::set_pipeline_entity_key(
            pipeline_context.metrics_registry(),
            pipeline_entity_key,
        );

        let registry = pipeline_context.metrics_registry();

        let recv_out_key = pipeline_context.register_channel_entity(
            "recv:out".into(),
            "output".into(),
            "pdata",
            "local",
            "mpsc",
            "internal",
        );
        let proc_in_key = pipeline_context.register_channel_entity(
            "proc:in".into(),
            "input".into(),
            "pdata",
            "local",
            "mpsc",
            "internal",
        );
        let proc_out_key = pipeline_context.register_channel_entity(
            "proc:out".into(),
            "output".into(),
            "pdata",
            "local",
            "mpsc",
            "internal",
        );
        let exp_in_key = pipeline_context.register_channel_entity(
            "exp:in".into(),
            "input".into(),
            "pdata",
            "local",
            "mpsc",
            "internal",
        );

        let recv_produced: MetricSet<ProducedMetrics> =
            registry.register_metric_set_for_entity(recv_out_key);
        let proc_consumed: MetricSet<ConsumedMetrics> =
            registry.register_metric_set_for_entity(proc_in_key);
        let proc_produced: MetricSet<ProducedMetrics> =
            registry.register_metric_set_for_entity(proc_out_key);
        let exp_consumed: MetricSet<ConsumedMetrics> =
            registry.register_metric_set_for_entity(exp_in_key);

        // Save metric set keys for snapshot identification.
        let mut key_labels = HashMap::new();
        let _ = key_labels.insert(recv_produced.metric_set_key(), MetricLabel::RecvProduced);
        let _ = key_labels.insert(proc_consumed.metric_set_key(), MetricLabel::ProcConsumed);
        let _ = key_labels.insert(proc_produced.metric_set_key(), MetricLabel::ProcProduced);
        let _ = key_labels.insert(exp_consumed.metric_set_key(), MetricLabel::ExpConsumed);

        let mut node_metric_handles: Vec<Option<NodeMetricHandles>> = Vec::new();
        let max_idx = nodes.iter().map(|n| n.index).max().unwrap_or(0);
        for _ in 0..=max_idx {
            node_metric_handles.push(None);
        }
        node_metric_handles[nodes[0].index] = Some(NodeMetricHandles {
            registry: registry.clone(),
            input: None,
            outputs: vec![recv_produced],
        });
        node_metric_handles[nodes[1].index] = Some(NodeMetricHandles {
            registry: registry.clone(),
            input: Some(proc_consumed),
            outputs: vec![proc_produced],
        });
        node_metric_handles[nodes[2].index] = Some(NodeMetricHandles {
            registry: registry.clone(),
            input: Some(exp_consumed),
            outputs: Vec::new(),
        });

        let telemetry_policy = TelemetryPolicy {
            channel_metrics: MetricLevel::Detailed,
            ..Default::default()
        };

        let node_metric_handles = Rc::new(RefCell::new(node_metric_handles));

        let manager = RuntimeCtrlMsgManager::new(
            DeployedPipelineKey {
                pipeline_group_id,
                pipeline_id,
                core_id: 0,
            },
            pipeline_context,
            pipeline_rx,
            control_senders,
            observed_state_store.reporter(SendPolicy::default()),
            metrics_reporter.clone(),
            telemetry_policy,
            Vec::new(),
            node_metric_handles.clone(),
        );

        MetricsTestHarness {
            manager,
            metrics_reporter: metrics_reporter.clone(),
            pipeline_tx,
            nodes,
            _guard: pipeline_entity_guard,
            snapshot_rx,
            key_labels,
            node_metric_handles,
        }
    }

    /// Collect all snapshots from the receiver into a map keyed by MetricLabel.
    /// If multiple snapshots share a key, their values are merged (summed).
    fn collect_snapshots(
        rx: &flume::Receiver<MetricSetSnapshot>,
        key_labels: &HashMap<MetricSetKey, MetricLabel>,
    ) -> HashMap<MetricLabel, Vec<MetricValue>> {
        let mut result: HashMap<MetricLabel, Vec<MetricValue>> = HashMap::new();
        while let Ok(snapshot) = rx.try_recv() {
            let Some(&label) = key_labels.get(&snapshot.key()) else {
                continue;
            };
            let values = snapshot.get_metrics().to_vec();
            let _ = result
                .entry(label)
                .and_modify(|existing| {
                    for (dst, src) in existing.iter_mut().zip(values.iter()) {
                        dst.add_in_place(*src);
                    }
                })
                .or_insert(values);
        }
        result
    }

    /// Extract u64 from a MetricValue, panicking with context on mismatch.
    fn assert_u64(values: &[MetricValue], index: usize, expected: u64, msg: &str) {
        match values[index] {
            MetricValue::U64(v) => assert_eq!(v, expected, "{msg}"),
            other => panic!("{msg}: expected U64, got {other:?}"),
        }
    }

    /// Extract Mmsc from a MetricValue, returning the snapshot for further assertions.
    fn assert_mmsc(
        values: &[MetricValue],
        index: usize,
        msg: &str,
    ) -> otap_df_telemetry::instrument::MmscSnapshot {
        match values[index] {
            MetricValue::Mmsc(snap) => snap,
            other => panic!("{msg}: expected Mmsc, got {other:?}"),
        }
    }

    // ConsumerMetrics field indices (defined by #[metric_set] field order):
    const CONSUMER_DURATION: usize = 0;
    const CONSUMER_SUCCESS: usize = 1;
    const CONSUMER_FAILURE: usize = 2;
    const CONSUMER_REFUSED: usize = 3;
    // ProducedMetrics field indices:
    const PRODUCER_DURATION: usize = 0;
    const PRODUCER_SUCCESS: usize = 1;
    const PRODUCER_FAILURE: usize = 2;
    const PRODUCER_REFUSED: usize = 3;

    /// Build a TestPData with frames simulating a 3-node pipeline:
    /// receiver(node0) → processor(node1) → exporter(node2).
    ///
    /// Frames are pushed bottom-to-top (receiver first, exporter last on top).
    fn build_3node_pdata(nodes: &[NodeId], with_timestamp: bool) -> TestPData {
        let mut pdata = TestPData::new();

        // Node 0 (receiver): producer metrics + acks/nacks (receives the ack back)
        pdata.push_frame(Frame {
            node_id: nodes[0].index,
            interests: Interests::PRODUCER_METRICS | Interests::ACKS | Interests::NACKS,
            route: RouteData {
                calldata: Default::default(),
                entry_time_ns: 0,
                output_port_index: 0,
            },
        });

        // Node 1 (processor): consumer + producer metrics + acks/nacks
        let entry_time_ns = if with_timestamp {
            nanos_since_birth()
        } else {
            0
        };
        pdata.push_frame(Frame {
            node_id: nodes[1].index,
            interests: Interests::CONSUMER_METRICS
                | Interests::PRODUCER_METRICS
                | Interests::ENTRY_TIMESTAMP
                | Interests::ACKS
                | Interests::NACKS,
            route: RouteData {
                calldata: Default::default(),
                entry_time_ns,
                output_port_index: 0,
            },
        });

        // Node 2 (exporter): consumer metrics only (no acks subscription — terminal node)
        let entry_time_ns = if with_timestamp {
            nanos_since_birth()
        } else {
            0
        };
        pdata.push_frame(Frame {
            node_id: nodes[2].index,
            interests: Interests::CONSUMER_METRICS | Interests::ENTRY_TIMESTAMP,
            route: RouteData {
                calldata: Default::default(),
                entry_time_ns,
                output_port_index: 0,
            },
        });

        pdata
    }

    /// Build a TestPData with frames simulating a 3-node pipeline where
    /// NO node subscribes to acks/nacks (all frames are metrics-only).
    /// This lets the controller unwind all frames in a single pass,
    /// including the receiver's producer-only frame.
    fn build_3node_pdata_no_subscribers(nodes: &[NodeId], with_timestamp: bool) -> TestPData {
        let mut pdata = TestPData::new();

        // Node 0 (receiver): producer metrics only — no ACKS/NACKS, no CONSUMER_METRICS.
        let entry_time_ns = if with_timestamp {
            nanos_since_birth()
        } else {
            0
        };
        pdata.push_frame(Frame {
            node_id: nodes[0].index,
            interests: Interests::PRODUCER_METRICS | Interests::ENTRY_TIMESTAMP,
            route: RouteData {
                calldata: Default::default(),
                entry_time_ns,
                output_port_index: 0,
            },
        });

        // Node 1 (processor): consumer + producer metrics, no ACKS/NACKS.
        let entry_time_ns = if with_timestamp {
            nanos_since_birth()
        } else {
            0
        };
        pdata.push_frame(Frame {
            node_id: nodes[1].index,
            interests: Interests::CONSUMER_METRICS
                | Interests::PRODUCER_METRICS
                | Interests::ENTRY_TIMESTAMP,
            route: RouteData {
                calldata: Default::default(),
                entry_time_ns,
                output_port_index: 0,
            },
        });

        // Node 2 (exporter): consumer metrics only.
        let entry_time_ns = if with_timestamp {
            nanos_since_birth()
        } else {
            0
        };
        pdata.push_frame(Frame {
            node_id: nodes[2].index,
            interests: Interests::CONSUMER_METRICS | Interests::ENTRY_TIMESTAMP,
            route: RouteData {
                calldata: Default::default(),
                entry_time_ns,
                output_port_index: 0,
            },
        });

        pdata
    }

    /// Helper: run the manager, send messages, shut down, collect snapshots.
    async fn run_and_collect(
        harness: MetricsTestHarness,
        send_fn: impl FnOnce(&[NodeId]) -> Vec<PipelineResultMsg<TestPData>>,
    ) -> HashMap<MetricLabel, Vec<MetricValue>> {
        let MetricsTestHarness {
            manager,
            mut metrics_reporter,
            pipeline_tx,
            nodes,
            _guard,
            snapshot_rx,
            key_labels,
            node_metric_handles,
        } = harness;

        let local = LocalSet::new();
        local
            .run_until(async {
                let msgs = send_fn(&nodes);
                let (return_tx, return_rx) = pipeline_result_msg_channel(32);
                let return_dispatcher = PipelineResultMsgDispatcher::new(
                    return_rx,
                    ControlSenders::new(),
                    node_metric_handles.clone(),
                );
                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });
                let dispatcher_handle =
                    tokio::task::spawn_local(async move { return_dispatcher.run().await });

                for msg in msgs {
                    return_tx.send(msg).await.unwrap();
                }

                drop(return_tx);
                drop(pipeline_tx);

                let dispatcher_result =
                    timeout(Duration::from_millis(500), dispatcher_handle).await;
                assert!(
                    dispatcher_result.is_ok(),
                    "Return dispatcher should shut down cleanly"
                );

                let result = timeout(Duration::from_millis(500), manager_handle).await;
                assert!(result.is_ok(), "Manager should shut down cleanly");

                report_node_metrics_with_handles(&node_metric_handles, &mut metrics_reporter)
                    .expect("Final node metrics flush should succeed");

                // Keep _guard alive for the scope of this closure
                drop(_guard);
                collect_snapshots(&snapshot_rx, &key_labels)
            })
            .await
    }

    /// Verify that ack correctly records consumed_success and produced_success
    /// via the full manager lifecycle and shutdown metrics flush.
    #[tokio::test]
    async fn test_ack_lifecycle_consumed_produced_metrics() {
        let harness = setup_test_manager_with_metrics();
        let nodes_clone = harness.nodes.clone();
        let snapshots = run_and_collect(harness, |nodes| {
            let pdata = build_3node_pdata(nodes, false);
            vec![PipelineResultMsg::DeliverAck {
                ack: AckMsg::new(pdata),
            }]
        })
        .await;

        // Exporter consumed: success=1, failure=0, refused=0
        let exp = &snapshots[&MetricLabel::ExpConsumed];
        assert_u64(exp, CONSUMER_SUCCESS, 1, "Exporter consumed_success");
        assert_u64(exp, CONSUMER_FAILURE, 0, "Exporter consumed_failure");
        assert_u64(exp, CONSUMER_REFUSED, 0, "Exporter consumed_refused");

        // Processor consumed: success=1
        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        assert_u64(proc_c, CONSUMER_SUCCESS, 1, "Processor consumed_success");

        // Processor produced: success=1
        let proc_p = &snapshots[&MetricLabel::ProcProduced];
        assert_u64(proc_p, PRODUCER_SUCCESS, 1, "Processor produced_success");

        // Receiver produced: unwind_ack delivers to first ACKS subscriber (processor)
        // so receiver frame is never popped → no metrics recorded.
        assert!(
            !snapshots.contains_key(&MetricLabel::RecvProduced),
            "Receiver produced should have no metrics (ack delivered at processor)"
        );

        drop(nodes_clone);
    }

    /// Verify that non-permanent nack records consumed_failure / produced_failure.
    #[tokio::test]
    async fn test_nack_lifecycle_failure_metrics() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            let pdata = build_3node_pdata(nodes, false);
            vec![PipelineResultMsg::DeliverNack {
                nack: NackMsg::new("transient error", pdata),
            }]
        })
        .await;

        let exp = &snapshots[&MetricLabel::ExpConsumed];
        assert_u64(exp, CONSUMER_FAILURE, 1, "Exporter consumed_failure");
        assert_u64(exp, CONSUMER_SUCCESS, 0, "Exporter consumed_success");
        assert_u64(exp, CONSUMER_REFUSED, 0, "Exporter consumed_refused");

        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        assert_u64(proc_c, CONSUMER_FAILURE, 1, "Processor consumed_failure");

        let proc_p = &snapshots[&MetricLabel::ProcProduced];
        assert_u64(proc_p, PRODUCER_FAILURE, 1, "Processor produced_failure");
    }

    /// Verify that permanent nack records consumed_refused / produced_refused.
    #[tokio::test]
    async fn test_permanent_nack_lifecycle_refused_metrics() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            let pdata = build_3node_pdata(nodes, false);
            vec![PipelineResultMsg::DeliverNack {
                nack: NackMsg::new_permanent("permanent refusal", pdata),
            }]
        })
        .await;

        let exp = &snapshots[&MetricLabel::ExpConsumed];
        assert_u64(exp, CONSUMER_REFUSED, 1, "Exporter consumed_refused");
        assert_u64(exp, CONSUMER_SUCCESS, 0, "Exporter consumed_success");
        assert_u64(exp, CONSUMER_FAILURE, 0, "Exporter consumed_failure");

        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        assert_u64(proc_c, CONSUMER_REFUSED, 1, "Processor consumed_refused");

        let proc_p = &snapshots[&MetricLabel::ProcProduced];
        assert_u64(proc_p, PRODUCER_REFUSED, 1, "Processor produced_refused");
    }

    /// Verify that consumed_duration_ns (Mmsc histogram) is recorded
    /// when entry_time_ns > 0 and return_time_ns > 0.
    #[tokio::test]
    async fn test_ack_lifecycle_duration_histogram() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            let pdata = build_3node_pdata(nodes, true);
            let mut ack = AckMsg::new(pdata);
            ack.unwind.return_time_ns = nanos_since_birth();
            vec![PipelineResultMsg::DeliverAck { ack }]
        })
        .await;

        // Exporter consumed duration: 1 observation, min > 0
        let exp = &snapshots[&MetricLabel::ExpConsumed];
        let snap = assert_mmsc(exp, CONSUMER_DURATION, "Exporter duration");
        assert_eq!(snap.count, 1, "Exporter should have 1 duration observation");
        assert!(snap.min > 0.0, "Duration min should be > 0");
        assert!(snap.max >= snap.min, "Duration max >= min");

        // Processor consumed duration: 1 observation, min > 0
        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        let snap = assert_mmsc(proc_c, CONSUMER_DURATION, "Processor consumed duration");
        assert_eq!(
            snap.count, 1,
            "Processor should have 1 consumed duration observation"
        );
        assert!(snap.min > 0.0, "Processor consumed duration should be > 0");

        // Processor produced duration: should be 0 observations because the
        // processor frame has CONSUMER_METRICS, so produced_duration_ns is
        // suppressed (one duration histogram per component).
        let proc_p = &snapshots[&MetricLabel::ProcProduced];
        let snap = assert_mmsc(proc_p, PRODUCER_DURATION, "Processor produced duration");
        assert_eq!(
            snap.count, 0,
            "Processor should have 0 produced duration observations (suppressed by CONSUMER_METRICS)"
        );
    }

    /// Verify that produced_duration_ns is recorded for producer-only frames
    /// (receiver) but NOT for frames that also have CONSUMER_METRICS (processor).
    /// Uses a no-subscriber pipeline so all frames are popped in a single unwind.
    #[tokio::test]
    async fn test_ack_lifecycle_produced_duration_histogram() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            let pdata = build_3node_pdata_no_subscribers(nodes, true);
            let mut ack = AckMsg::new(pdata);
            ack.unwind.return_time_ns = nanos_since_birth();
            vec![PipelineResultMsg::DeliverAck { ack }]
        })
        .await;

        // Receiver produced duration: 1 observation, min > 0
        // (producer-only frame, no CONSUMER_METRICS → produced_duration recorded)
        let recv_p = &snapshots[&MetricLabel::RecvProduced];
        let snap = assert_mmsc(recv_p, PRODUCER_DURATION, "Receiver produced duration");
        assert_eq!(
            snap.count, 1,
            "Receiver should have 1 produced duration observation"
        );
        assert!(snap.min > 0.0, "Receiver produced duration should be > 0");
        assert!(
            snap.max >= snap.min,
            "Receiver produced duration max >= min"
        );

        // Processor produced duration: 0 observations
        // (merged frame has CONSUMER_METRICS → produced_duration suppressed)
        let proc_p = &snapshots[&MetricLabel::ProcProduced];
        let snap = assert_mmsc(proc_p, PRODUCER_DURATION, "Processor produced duration");
        assert_eq!(
            snap.count, 0,
            "Processor should have 0 produced duration observations"
        );

        // Processor consumed duration: 1 observation (still works)
        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        let snap = assert_mmsc(proc_c, CONSUMER_DURATION, "Processor consumed duration");
        assert_eq!(
            snap.count, 1,
            "Processor should have 1 consumed duration observation"
        );
    }

    /// Verify that produced_duration_ns is NOT recorded when entry_time_ns is 0.
    #[tokio::test]
    async fn test_produced_duration_not_recorded_without_timestamp() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            let pdata = build_3node_pdata_no_subscribers(nodes, false);
            vec![PipelineResultMsg::DeliverAck {
                ack: AckMsg::new(pdata),
            }]
        })
        .await;

        // Receiver produced duration: 0 observations (no timestamp)
        let recv_p = &snapshots[&MetricLabel::RecvProduced];
        let snap = assert_mmsc(recv_p, PRODUCER_DURATION, "Receiver produced duration");
        assert_eq!(
            snap.count, 0,
            "No produced duration should be recorded when entry_time_ns == 0"
        );
    }

    /// Verify that when entry_time_ns is 0 (or return_time_ns is 0), no duration histogram is recorded.
    #[tokio::test]
    async fn test_ack_lifecycle_no_duration_without_timestamp() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            let pdata = build_3node_pdata(nodes, false);
            vec![PipelineResultMsg::DeliverAck {
                ack: AckMsg::new(pdata),
            }]
        })
        .await;

        let exp = &snapshots[&MetricLabel::ExpConsumed];
        let snap = assert_mmsc(exp, CONSUMER_DURATION, "Exporter duration");
        assert_eq!(
            snap.count, 0,
            "No duration should be recorded when entry_time_ns == 0"
        );

        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        let snap = assert_mmsc(proc_c, CONSUMER_DURATION, "Processor duration");
        assert_eq!(
            snap.count, 0,
            "No duration should be recorded when entry_time_ns == 0"
        );
    }

    /// Verify multiple acks accumulate counters correctly through the lifecycle.
    #[tokio::test]
    async fn test_multiple_acks_lifecycle_accumulate() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            (0..3)
                .map(|_| {
                    let pdata = build_3node_pdata(nodes, false);
                    PipelineResultMsg::DeliverAck {
                        ack: AckMsg::new(pdata),
                    }
                })
                .collect()
        })
        .await;

        let exp = &snapshots[&MetricLabel::ExpConsumed];
        assert_u64(
            exp,
            CONSUMER_SUCCESS,
            3,
            "Exporter 3 consumed_success after 3 acks",
        );

        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        assert_u64(proc_c, CONSUMER_SUCCESS, 3, "Processor 3 consumed_success");

        let proc_p = &snapshots[&MetricLabel::ProcProduced];
        assert_u64(proc_p, PRODUCER_SUCCESS, 3, "Processor 3 produced_success");
    }

    /// Verify mixed ack and nack messages accumulate correctly.
    #[tokio::test]
    async fn test_mixed_ack_nack_lifecycle() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            vec![
                PipelineResultMsg::DeliverAck {
                    ack: AckMsg::new(build_3node_pdata(nodes, false)),
                },
                PipelineResultMsg::DeliverNack {
                    nack: NackMsg::new("transient", build_3node_pdata(nodes, false)),
                },
                PipelineResultMsg::DeliverNack {
                    nack: NackMsg::new_permanent("refused", build_3node_pdata(nodes, false)),
                },
            ]
        })
        .await;

        // Exporter consumed: 1 success + 1 failure + 1 refused
        let exp = &snapshots[&MetricLabel::ExpConsumed];
        assert_u64(exp, CONSUMER_SUCCESS, 1, "Exporter consumed_success");
        assert_u64(exp, CONSUMER_FAILURE, 1, "Exporter consumed_failure");
        assert_u64(exp, CONSUMER_REFUSED, 1, "Exporter consumed_refused");

        // Processor consumed: same pattern
        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        assert_u64(proc_c, CONSUMER_SUCCESS, 1, "Processor consumed_success");
        assert_u64(proc_c, CONSUMER_FAILURE, 1, "Processor consumed_failure");
        assert_u64(proc_c, CONSUMER_REFUSED, 1, "Processor consumed_refused");

        // Processor produced: 1 success + 1 failure + 1 refused
        let proc_p = &snapshots[&MetricLabel::ProcProduced];
        assert_u64(proc_p, PRODUCER_SUCCESS, 1, "Processor produced_success");
        assert_u64(proc_p, PRODUCER_FAILURE, 1, "Processor produced_failure");
        assert_u64(proc_p, PRODUCER_REFUSED, 1, "Processor produced_refused");
    }

    /// Simulate the real two-pass unwind for a receiver→processor→exporter pipeline.
    ///
    /// Pass 1: full stack [recv, proc, exp] — unwinds exp and proc frames,
    ///         delivers ack to processor (first ACKS subscriber).
    /// Pass 2: processor re-notifies with just the receiver frame — unwinds recv,
    ///         recording producer duration on the receiver's output.
    ///
    /// This is the scenario where producer.duration must be recorded for the receiver.
    #[tokio::test]
    async fn test_two_pass_unwind_receiver_produced_duration() {
        let harness = setup_test_manager_with_metrics();
        let snapshots = run_and_collect(harness, |nodes| {
            // --- Pass 1: full 3-node stack (processor subscribes to ACKS) ---
            let pdata_full = build_3node_pdata(nodes, true);
            let mut ack1 = AckMsg::new(pdata_full);
            ack1.unwind.return_time_ns = nanos_since_birth();

            // --- Pass 2: only the receiver frame remains (processor re-notified) ---
            let mut pdata_recv_only = TestPData::new();
            pdata_recv_only.push_frame(Frame {
                node_id: nodes[0].index,
                interests: Interests::PRODUCER_METRICS | Interests::ENTRY_TIMESTAMP,
                route: RouteData {
                    calldata: Default::default(),
                    entry_time_ns: nanos_since_birth(),
                    output_port_index: 0,
                },
            });
            let mut ack2 = AckMsg::new(pdata_recv_only);
            ack2.unwind.return_time_ns = nanos_since_birth();

            vec![
                PipelineResultMsg::DeliverAck { ack: ack1 },
                PipelineResultMsg::DeliverAck { ack: ack2 },
            ]
        })
        .await;

        // From pass 1: exporter and processor consumer metrics are recorded.
        let exp = &snapshots[&MetricLabel::ExpConsumed];
        assert_u64(exp, CONSUMER_SUCCESS, 1, "Exporter consumed_success");
        let snap = assert_mmsc(exp, CONSUMER_DURATION, "Exporter consumed duration");
        assert_eq!(snap.count, 1, "Exporter should have 1 consumed duration");
        assert!(snap.min > 0.0, "Exporter consumed duration > 0");

        let proc_c = &snapshots[&MetricLabel::ProcConsumed];
        assert_u64(proc_c, CONSUMER_SUCCESS, 1, "Processor consumed_success");
        let snap = assert_mmsc(proc_c, CONSUMER_DURATION, "Processor consumed duration");
        assert_eq!(snap.count, 1, "Processor should have 1 consumed duration");

        // From pass 1: processor produced counter recorded.
        let proc_p = &snapshots[&MetricLabel::ProcProduced];
        assert_u64(proc_p, PRODUCER_SUCCESS, 1, "Processor produced_success");

        // From pass 2: receiver produced counter AND duration recorded.
        let recv_p = &snapshots[&MetricLabel::RecvProduced];
        assert_u64(recv_p, PRODUCER_SUCCESS, 1, "Receiver produced_success");
        let snap = assert_mmsc(recv_p, PRODUCER_DURATION, "Receiver produced duration");
        assert_eq!(
            snap.count, 1,
            "Receiver should have 1 produced duration observation from two-pass unwind"
        );
        assert!(snap.min > 0.0, "Receiver produced duration should be > 0");
    }

    #[tokio::test]
    async fn test_return_lane_progress_while_runtime_ctrl_lane_is_busy() {
        use crate::control::AckMsg;

        fn pdata_for_node(node_id: usize) -> TestPData {
            let mut pdata = TestPData::new();
            pdata.push_frame(Frame {
                node_id,
                interests: Interests::ACKS,
                route: RouteData {
                    calldata: Default::default(),
                    entry_time_ns: 0,
                    output_port_index: 0,
                },
            });
            pdata
        }

        let local = LocalSet::new();

        local
            .run_until(async {
                let (
                    manager,
                    pipeline_tx,
                    control_senders,
                    mut control_receivers,
                    nodes,
                    _pipeline_entity_guard,
                ) = setup_test_manager_with_capacities::<TestPData>(128, 10);
                let noisy_node = nodes[0].clone();
                let target = nodes[1].clone();

                for _ in 0..96 {
                    pipeline_tx
                        .send(RuntimeControlMsg::StartTimer {
                            node_id: noisy_node.index,
                            duration: Duration::from_secs(60),
                        })
                        .await
                        .unwrap();
                }

                let (return_tx, return_rx) = pipeline_result_msg_channel(8);
                return_tx
                    .send(PipelineResultMsg::DeliverAck {
                        ack: AckMsg::new(pdata_for_node(target.index)),
                    })
                    .await
                    .unwrap();

                let dispatcher = PipelineResultMsgDispatcher::new(
                    return_rx,
                    control_senders,
                    empty_node_metric_handles(),
                );

                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });
                let dispatcher_handle =
                    tokio::task::spawn_local(async move { dispatcher.run().await });

                let mut receiver = control_receivers.remove(&target.index).unwrap();
                let msg = timeout(Duration::from_millis(500), receiver.recv())
                    .await
                    .expect("Ack should make progress while the runtime control lane is busy")
                    .expect("target control channel should stay open");
                assert!(matches!(msg, NodeControlMsg::Ack(_)));

                drop(return_tx);
                drop(pipeline_tx);

                let dispatcher_result =
                    timeout(Duration::from_millis(200), dispatcher_handle).await;
                assert!(
                    dispatcher_result.is_ok(),
                    "Return dispatcher should shut down cleanly"
                );
                let manager_result = timeout(Duration::from_millis(200), manager_handle).await;
                assert!(manager_result.is_ok(), "Manager should shut down cleanly");
            })
            .await;
    }

    /// Demonstrates a realistic circular wait between the manager and an active
    /// node task.
    ///
    /// This models what happens in production with an exporter like
    /// `AzureMonitorExporter`, which, after a successful export, sends multiple
    /// acks in a tight loop:
    ///
    /// ```ignore
    /// for (_, context, payload) in completed_messages {
    ///     effect_handler.notify_ack(AckMsg::new(…)).await?;
    ///     //             ^^^^^^^^^ sends DeliverAck to pipeline ctrl channel
    /// }
    /// ```
    ///
    /// Setup:
    ///   - Pipeline ctrl channel (nodes → manager): capacity 3
    ///   - Node A control channel (manager → A):    capacity 1
    ///   - Node B control channel (manager → B):    capacity 10
    ///
    /// The circular wait forms as follows:
    ///   1. Pre-load pipeline ctrl with [DeliverAck{A}, DeliverAck{A}, DeliverAck{B}].
    ///   2. Spawn Node A as an active task that loops sending DeliverAck{A} to
    ///      the pipeline ctrl channel (simulating batchexport ack loop).
    ///      Pipeline ctrl is full, so Node A blocks immediately.
    ///   3. Manager processes the two DeliverAck{A}s (freeing slots that Node A
    ///      promptly refills), sending Acks to A's control channel.
    ///      The first Ack succeeds (fills A's cap-1 channel).
    ///      The second Ack finds A's channel full → manager blocks on `.await`.
    ///   4. Now both are stuck:
    ///      - Manager is blocked sending to A's control channel (full)
    ///      - Node A is blocked sending to pipeline ctrl channel (full, refilled
    ///        after manager freed the initial two slots)
    ///      - Neither can make progress.
    ///   5. DeliverAck{B} sits in the pipeline ctrl queue — never processed.
    ///
    /// The test asserts Node B receives its ack within 500 ms.  The non-blocking
    /// `try_send` + `pending_sends` buffering in `send()` prevents the manager
    /// from blocking on Node A's full control channel, so Node B's ack is
    /// delivered promptly.
    #[tokio::test]
    async fn test_circular_wait_between_node_and_manager() {
        use crate::control::AckMsg;

        // Build a TestPData whose single frame routes the ack to `node_id`.
        fn pdata_for_node(node_id: usize) -> TestPData {
            let mut pdata = TestPData::new();
            pdata.push_frame(Frame {
                node_id,
                interests: Interests::ACKS,
                route: RouteData {
                    calldata: Default::default(),
                    entry_time_ns: 0,
                    output_port_index: 0,
                },
            });
            pdata
        }

        let local = LocalSet::new();

        local
            .run_until(async {
                // --- Custom setup with specific channel capacities ---
                let (return_tx, return_rx) = pipeline_result_msg_channel(3);
                let mut control_senders = ControlSenders::new();

                let nodes = test_nodes(vec!["node_a", "node_b"]);
                let node_a = nodes[0].clone();
                let node_b = nodes[1].clone();

                // Node A: control channel capacity 1 — fills up after one message
                let (tx_a, rx_a) = tokio::sync::mpsc::channel::<NodeControlMsg<TestPData>>(1);
                control_senders.register(
                    node_a.clone(),
                    NodeType::Processor,
                    Sender::Shared(SharedSender::mpsc(tx_a)),
                );

                // Node B: control channel capacity 10 — plenty of room
                let (tx_b, rx_b) = tokio::sync::mpsc::channel::<NodeControlMsg<TestPData>>(10);
                control_senders.register(
                    node_b.clone(),
                    NodeType::Processor,
                    Sender::Shared(SharedSender::mpsc(tx_b)),
                );

                let dispatcher = PipelineResultMsgDispatcher::new(
                    return_rx,
                    control_senders,
                    empty_node_metric_handles(),
                );

                // Pre-load the shared return channel: [DeliverAck{A}, DeliverAck{A}, DeliverAck{B}]
                // This fills the channel (cap=3) before anyone starts consuming.
                return_tx
                    .send(PipelineResultMsg::DeliverAck {
                        ack: AckMsg::new(pdata_for_node(node_a.index)),
                    })
                    .await
                    .unwrap();
                return_tx
                    .send(PipelineResultMsg::DeliverAck {
                        ack: AckMsg::new(pdata_for_node(node_a.index)),
                    })
                    .await
                    .unwrap();
                return_tx
                    .send(PipelineResultMsg::DeliverAck {
                        ack: AckMsg::new(pdata_for_node(node_b.index)),
                    })
                    .await
                    .unwrap();

                // Spawn Node A: simulates an exporter that's acking a batch of
                // messages in a tight loop (just like AzureMonitorExporter's
                // `for msg in completed_messages { notify_ack(..).await; }` loop).
                //
                // Node A keeps sending DeliverAck to the shared return channel.
                // When the channel is full, Node A blocks — and since it never
                // drains its own control channel (rx_a), the dispatcher can't
                // deliver acks to it either.
                let node_a_tx = return_tx.clone();
                let node_a_index = node_a.index;
                let _node_a_handle = tokio::task::spawn_local(async move {
                    let _rx_a = rx_a; // keep A's ctrl channel open (not closed)
                    loop {
                        if node_a_tx
                            .send(PipelineResultMsg::DeliverAck {
                                ack: AckMsg::new(pdata_for_node(node_a_index)),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });

                // Start the dispatcher
                let dispatcher_handle =
                    tokio::task::spawn_local(async move { dispatcher.run().await });

                // Assert Node B receives its ack within 500 ms.
                // With the non-blocking try_send fix, the dispatcher buffers
                // messages for Node A's full channel and keeps processing,
                // so Node B's ack arrives promptly.
                let mut receiver_b = Receiver::Shared(SharedReceiver::mpsc(rx_b));
                let received = timeout(Duration::from_millis(500), receiver_b.recv()).await;

                assert!(
                    received.is_ok(),
                    "Node B should receive its Ack within 500 ms, but the \
                     dispatcher is stuck in a circular wait with Node A: the \
                     dispatcher is blocked sending to Node A's full control \
                     channel, while Node A is blocked sending to the full \
                     shared return channel"
                );

                // Cleanup
                drop(return_tx);
                dispatcher_handle.abort();
            })
            .await;
    }

    #[tokio::test]
    async fn test_runtime_ctrl_progress_while_return_lane_is_busy() {
        use crate::control::AckMsg;

        fn pdata_for_node(node_id: usize) -> TestPData {
            let mut pdata = TestPData::new();
            pdata.push_frame(Frame {
                node_id,
                interests: Interests::ACKS,
                route: RouteData {
                    calldata: Default::default(),
                    entry_time_ns: 0,
                    output_port_index: 0,
                },
            });
            pdata
        }

        let local = LocalSet::new();

        local
            .run_until(async {
                let mut control_senders = ControlSenders::new();
                let nodes = test_nodes(vec!["node_a", "node_timer"]);
                let node_a = nodes[0].clone();
                let node_timer = nodes[1].clone();

                let (tx_a, rx_a) = tokio::sync::mpsc::channel::<NodeControlMsg<TestPData>>(1);
                control_senders.register(
                    node_a.clone(),
                    NodeType::Processor,
                    Sender::Shared(SharedSender::mpsc(tx_a)),
                );

                let (tx_timer, rx_timer) =
                    tokio::sync::mpsc::channel::<NodeControlMsg<TestPData>>(10);
                control_senders.register(
                    node_timer.clone(),
                    NodeType::Processor,
                    Sender::Shared(SharedSender::mpsc(tx_timer)),
                );

                let (mut manager, pipeline_tx, _pipeline_entity_guard) =
                    build_test_manager(16, control_senders.clone());
                manager.tick_timers.start(node_timer.index, Duration::from_millis(1));
                tokio::time::sleep(Duration::from_millis(5)).await;

                let (return_tx, return_rx) = pipeline_result_msg_channel(3);
                for _ in 0..3 {
                    return_tx
                        .send(PipelineResultMsg::DeliverAck {
                            ack: AckMsg::new(pdata_for_node(node_a.index)),
                        })
                        .await
                        .unwrap();
                }

                let dispatcher = PipelineResultMsgDispatcher::new(
                    return_rx,
                    control_senders,
                    empty_node_metric_handles(),
                );

                let node_a_tx = return_tx.clone();
                let node_a_index = node_a.index;
                let node_a_handle = tokio::task::spawn_local(async move {
                    let _rx_a = rx_a;
                    for _ in 0..200 {
                        if node_a_tx
                            .send(PipelineResultMsg::DeliverAck {
                                ack: AckMsg::new(pdata_for_node(node_a_index)),
                            })
                            .await
                            .is_err()
                        {
                            break;
                        }
                    }
                });

                let manager_handle = tokio::task::spawn_local(async move { manager.run().await });
                let dispatcher_handle =
                    tokio::task::spawn_local(async move { dispatcher.run().await });

                let mut receiver_timer = Receiver::Shared(SharedReceiver::mpsc(rx_timer));
                let received = timeout(Duration::from_millis(500), receiver_timer.recv()).await;
                assert!(
                    received.is_ok(),
                    "TimerTick should make progress even while the return lane is under sustained Ack load"
                );
                let msg = received.unwrap().expect("timer control channel should stay open");
                assert!(matches!(msg, NodeControlMsg::TimerTick {}));

                drop(return_tx);
                drop(pipeline_tx);

                let _ = timeout(Duration::from_millis(500), node_a_handle).await;
                let dispatcher_result =
                    timeout(Duration::from_millis(500), dispatcher_handle).await;
                assert!(
                    dispatcher_result.is_ok(),
                    "Return dispatcher should shut down cleanly"
                );
                let manager_result = timeout(Duration::from_millis(500), manager_handle).await;
                assert!(manager_result.is_ok(), "Manager should shut down cleanly");
            })
            .await;
    }
}
