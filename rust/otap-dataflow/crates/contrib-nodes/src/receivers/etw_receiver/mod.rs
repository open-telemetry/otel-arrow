// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! ETW (Event Tracing for Windows) Receiver
//!
//! Receives ETW events from Windows ETW sessions, converts them to OTAP Arrow
//! log record batches, and forwards them through the pipeline.
//!
//! This module is compiled only on Windows (`#[cfg(target_os = "windows")]`
//! in the parent `receivers/mod.rs`).
//!
//! ## Multi-core fan-out
//!
//! Windows allows only one real-time ETW session per session name. The engine
//! may instantiate the receiver once per allocated core.  To reconcile these
//! two models, one session is created **per `session_name`** with N consumer
//! channels (one per core).  The `ProcessTrace` callback round-robins events
//! across all channels so each core receives an even share of the event stream.
//!
//! ```text
//! ProcessTrace OS thread  (one per session_name, lazily spawned)
//! callback: txs[next].try_send(data); next = (next+1) % N
//!       |          |          |
//!     tx[0]      tx[1]      tx[2]
//!       v          v          v
//!      mpsc       mpsc       mpsc
//!       v          v          v
//!  +--------+ +--------+ +--------+
//!  | core 0 | | core 1 | | core 2 |
//!  | rx[0]  | | rx[1]  | | rx[2]  |
//!  +--------+ +--------+ +--------+
//! ```
//!
//! ## Quick start
//!
//! ```yaml
//! etw:
//!   type: receiver:etw
//!   config:
//!     providers:
//!       - guid: "d2387720-2907-5677-8625-c1bdc4155197"
//!         level: verbose
//!     batching:
//!       max_size: 100
//!       max_duration: "100ms"
//! ```

mod arrow_records_encoder;
mod session;

use arrow_records_encoder::EtwArrowRecordsBuilder;
use session::EtwEventData;

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ReceiverFactory;
use otap_df_engine::config::ReceiverConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::node::NodeId;
use otap_df_engine::receiver::ReceiverWrapper;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_engine::{
    MessageSourceLocalEffectHandlerExtension,
    effect_handler::TelemetryTimerCancelHandle,
    error::{Error, ReceiverErrorKind, format_error_sources},
    local::receiver as local,
};
use otap_df_otap::OTAP_RECEIVER_FACTORIES;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::{otel_info, otel_warn};
use otap_df_telemetry_macros::metric_set;
use serde::Deserialize;
use serde_json::Value;
use tokio::time::{self, MissedTickBehavior};

use std::cell::RefCell;
use std::num::NonZeroU16;
use std::rc::Rc;
use std::sync::Arc;
use std::sync::atomic::Ordering;
use std::time::Duration;

/// URN for the ETW receiver.
pub const ETW_RECEIVER_URN: &str = "urn:otel:receiver:etw";

// ── Defaults ─────────────────────────────────────────────────────────────────

// 512 is non-zero, so `unwrap()` never panics (evaluated at compile time).
const DEFAULT_BATCH_MAX_SIZE: NonZeroU16 = NonZeroU16::new(512).unwrap();
const DEFAULT_BATCH_MAX_DURATION: Duration = Duration::from_millis(100);

/// Upper bound on the time spent draining queued events during `DrainIngress`.
///
/// The drain budget is computed as 90% of the remaining time until the
/// deadline, but capped at this value so that a generous deadline does not
/// stall shutdown while a busy provider keeps producing events.
const MAX_DRAIN_BUDGET: Duration = Duration::from_secs(1);

// ── Configuration ────────────────────────────────────────────────────────────

/// Trace level filter for ETW providers, matching the standard five ETW levels.
#[derive(Debug, Clone, Deserialize, Default, PartialEq)]
#[serde(rename_all = "snake_case")]
enum TraceLevel {
    /// Critical errors only.
    Critical,
    /// Errors and critical events.
    Error,
    /// Warnings, errors, and critical events.
    Warning,
    /// Informational events and above (default).
    #[default]
    Information,
    /// All events including verbose/debug output.
    Verbose,
}

/// Configuration for a single ETW provider to trace.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct ProviderConfig {
    /// The ETW provider name (e.g. `"Microsoft-Windows-Kernel-Process"`).
    /// Mutually exclusive with `guid`.
    #[serde(default)]
    pub name: Option<String>,

    /// The ETW provider GUID (e.g. `"22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"`).
    /// Mutually exclusive with `name`.
    #[serde(default)]
    pub guid: Option<String>,

    /// Trace level filter. Defaults to `information`.
    #[serde(default)]
    pub level: TraceLevel,

    /// Optional keywords bitmask to further filter events.
    /// When omitted, all keywords are matched.
    #[serde(default)]
    pub keywords: Option<u64>,
}

/// In-memory OTAP log batching policy.
///
/// Decoded ETW events are accumulated into OTAP log batches before they are
/// sent downstream.  Batching improves throughput but increases the number of
/// decoded records that can be lost if the process exits before a flush.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct BatchConfig {
    /// Maximum number of log records per emitted OTAP batch.
    /// Must be greater than zero; `0` is rejected at deserialization time.
    #[serde(default = "default_batch_max_size")]
    max_size: NonZeroU16,
    /// Maximum time to hold a non-empty batch before flushing it downstream.
    #[serde(default = "default_batch_max_duration")]
    #[serde(with = "humantime_serde")]
    max_duration: Duration,
}

impl Default for BatchConfig {
    fn default() -> Self {
        Self {
            max_size: default_batch_max_size(),
            max_duration: default_batch_max_duration(),
        }
    }
}

/// Top-level configuration for the ETW receiver.
#[derive(Debug, Clone, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
struct Config {
    /// One or more ETW providers to subscribe to.
    pub providers: Vec<ProviderConfig>,

    /// Name of the ETW trace session. Defaults to `"OtelArrowETW"`.
    #[serde(default = "default_session_name")]
    pub session_name: String,

    /// OTAP log batching limits.
    #[serde(default)]
    pub batching: Option<BatchConfig>,
}

impl Config {
    /// Validates domain rules that cannot be expressed by the type system alone.
    ///
    /// # Rules
    ///
    /// * At least one provider must be specified.
    /// * Each provider must specify exactly one of `name` or `guid` (not both, not neither).
    ///
    /// # Errors
    ///
    /// Returns [`otap_df_config::error::Error::InvalidUserConfig`] when a rule is violated.
    fn validate(&self) -> Result<(), otap_df_config::error::Error> {
        if self.providers.is_empty() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "at least one ETW provider must be configured".to_string(),
            });
        }

        for (i, provider) in self.providers.iter().enumerate() {
            match (&provider.name, &provider.guid) {
                (Some(_), Some(_)) => {
                    return Err(otap_df_config::error::Error::InvalidUserConfig {
                        error: format!(
                            "provider[{i}]: 'name' and 'guid' are mutually exclusive - specify one, not both"
                        ),
                    });
                }
                (None, None) => {
                    return Err(otap_df_config::error::Error::InvalidUserConfig {
                        error: format!("provider[{i}]: either 'name' or 'guid' must be specified"),
                    });
                }
                _ => {} // Valid: exactly one of name or guid is specified.
            }
        }

        if let Some(ref batching) = self.batching {
            if batching.max_duration.is_zero() {
                return Err(otap_df_config::error::Error::InvalidUserConfig {
                    error: "ETW receiver `batching.max_duration` must be greater than zero"
                        .to_string(),
                });
            }
        }

        Ok(())
    }
}

fn default_session_name() -> String {
    "OtelArrowETW".to_string()
}

const fn default_batch_max_size() -> NonZeroU16 {
    DEFAULT_BATCH_MAX_SIZE
}

const fn default_batch_max_duration() -> Duration {
    DEFAULT_BATCH_MAX_DURATION
}

// ── Receiver struct ──────────────────────────────────────────────────────────

/// ETW receiver that subscribes to Windows ETW trace sessions and converts
/// events into OTAP Arrow log records.
///
/// Each per-core instance holds its own `event_rx` obtained from the
/// per-session-name ETW session at factory time.
struct EtwReceiver {
    config: Config,
    batching: BatchConfig,
    metrics: Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    /// Per-core consumer channel from the per-session-name ETW session.
    event_rx: tokio::sync::mpsc::Receiver<EtwEventData>,
    /// Shared atomic counters written by the `!Send` `ProcessTrace` callback
    /// (queue-full drops and decode failures). Folded into
    /// `metrics` on each `CollectTelemetry` tick and before terminal snapshots.
    session_wide_metrics: Arc<session::SessionWideMetrics>,
}

impl EtwReceiver {
    /// Create a new receiver, acquiring one consumer channel from the
    /// per-session-name ETW session.
    ///
    /// Called at **factory time** (once per allocated core).  The first call
    /// for a given `session_name` lazily initializes the session and spawns
    /// the `ProcessTrace` thread.
    fn from_config(
        pipeline: PipelineContext,
        config: &Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let cfg: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;
        cfg.validate()?;

        let num_cores = pipeline.num_cores();
        let metrics = pipeline.register_metrics::<EtwReceiverMetrics>();
        let batching = cfg.batching.clone().unwrap_or_default();

        // Acquire this core's consumer channel from the per-session-name
        // session.  The first call initializes the session; subsequent calls
        // pop from the pre-allocated pool.  The shared telemetry handle bridges
        // the `!Send` ProcessTrace callback to the async receivers: the producer
        // writes the session-scoped atomics that whichever core drains first
        // folds into its own metric set.
        let (event_rx, session_wide_metrics) =
            session::subscribe(&cfg, num_cores).map_err(|e| {
                otap_df_config::error::Error::InvalidUserConfig {
                    error: format!("ETW session initialization failed: {e}"),
                }
            })?;

        Ok(EtwReceiver {
            config: cfg,
            batching,
            metrics: Rc::new(RefCell::new(metrics)),
            event_rx,
            session_wide_metrics,
        })
    }
}

// ── Batch flush helpers ──────────────────────────────────────────────────────

/// Build the pending Arrow batch from `builder`, recording the failure metric
/// on error.
///
/// `builder` is always reset to a fresh empty builder (its contents are taken
/// via [`std::mem::take`]) regardless of outcome. Returns `Ok(None)` when the
/// builder is empty (nothing to flush). On a `build()` failure the
/// `received_events_forward_failed` metric is recorded and the error is
/// returned - a build failure indicates an encoding bug rather than a transient
/// downstream condition, so it is surfaced to the caller.
fn build_pending_batch(
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    builder: &mut EtwArrowRecordsBuilder,
) -> Result<Option<(OtapPdata, u64)>, Error> {
    if builder.is_empty() {
        return Ok(None);
    }

    let item_count = u64::from(builder.len());

    let payload = match std::mem::take(builder).build() {
        Ok(payload) => payload,
        Err(error) => {
            metrics
                .borrow_mut()
                .received_events_forward_failed
                .add(item_count);
            return Err(Error::ReceiverError {
                receiver: effect_handler.receiver_id(),
                kind: ReceiverErrorKind::Transport,
                error: "failed to build ETW Arrow batch".to_owned(),
                source_detail: format_error_sources(&error),
            });
        }
    };

    Ok(Some((
        OtapPdata::new_todo_context(payload.into()),
        item_count,
    )))
}

/// Flush the current Arrow batch downstream via the awaiting effect handler.
///
/// `builder` is always reset to a fresh empty builder regardless of outcome.
///
/// A forward failure is treated as a **per-batch loss event**, not a fatal
/// error: the `received_events_forward_failed` metric is incremented, a warning
/// is logged, and `Ok(())` is returned so the receiver stays alive. On the
/// awaiting path the only reachable send failure is the downstream channel
/// being closed (backpressure parks rather than failing), which can happen
/// transiently while a downstream node restarts; the control plane
/// (`DrainIngress`/`Shutdown`) decides when the receiver actually stops.
/// A `build()` failure still propagates, since it signals an encoding bug.
async fn flush_batch(
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    builder: &mut EtwArrowRecordsBuilder,
) -> Result<(), Error> {
    let Some((pdata, item_count)) = build_pending_batch(effect_handler, metrics, builder)? else {
        return Ok(());
    };

    if let Err(error) = effect_handler.send_message_with_source_node(pdata).await {
        metrics
            .borrow_mut()
            .received_events_forward_failed
            .add(item_count);
        let error_msg = error.to_string();
        otel_warn!(
            "etw_receiver.forward_failed",
            message = "failed to forward ETW Arrow batch downstream; dropping batch",
            dropped_events = item_count,
            error = error_msg.as_str(),
        );
        return Ok(());
    }

    metrics
        .borrow_mut()
        .received_events_forwarded
        .add(item_count);
    Ok(())
}

/// Non-blocking sibling of [`flush_batch`] used during shutdown/teardown paths
/// (`DrainIngress` and the channel-closed branch).
///
/// `builder` is always reset to a fresh empty builder regardless of outcome.
/// The flush uses `try_send_message_with_source_node` so a slow or
/// shutting-down downstream cannot park the task past the drain deadline.
///
/// Like [`flush_batch`], a forward failure (downstream channel full or closed)
/// is treated as a per-batch loss event: the `received_events_forward_failed`
/// metric is incremented, a warning is logged, and `Ok(())` is returned so the
/// receiver stays alive. A `build()` failure still propagates.
fn try_flush_batch(
    effect_handler: &local::EffectHandler<OtapPdata>,
    metrics: &Rc<RefCell<MetricSet<EtwReceiverMetrics>>>,
    builder: &mut EtwArrowRecordsBuilder,
) -> Result<(), Error> {
    let Some((pdata, item_count)) = build_pending_batch(effect_handler, metrics, builder)? else {
        return Ok(());
    };

    if let Err(error) = effect_handler.try_send_message_with_source_node(pdata) {
        metrics
            .borrow_mut()
            .received_events_forward_failed
            .add(item_count);
        let error_msg = error.to_string();
        otel_warn!(
            "etw_receiver.forward_failed",
            message = "failed to forward ETW Arrow batch downstream; dropping batch",
            dropped_events = item_count,
            error = error_msg.as_str(),
        );
        return Ok(());
    }

    metrics
        .borrow_mut()
        .received_events_forwarded
        .add(item_count);
    Ok(())
}

// ── Producer-side atomic folding ─────────────────────────────────────────────

/// The per-session producer-side deltas claimed by a single `swap(0)` pass over
/// the shared [`session::SessionWideMetrics`] atomics.
///
/// Returned by [`take_event_counts`] and applied to a metric set by
/// [`EventCountsDelta::apply`]. Splitting the `swap` (claim) from the metric
/// `add` (apply) keeps the borrow of the `RefCell<MetricSet>` short and, more
/// importantly, makes the "claim exactly once" semantics unit-testable without
/// constructing a full registered `MetricSet` (which requires a pipeline
/// context). See the boundary-case tests in this module.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct EventCountsDelta {
    total: u64,
    dropped_slow_worker: u64,
    invalid: u64,
    kernel_events_lost: u64,
    kernel_real_time_buffers_lost: u64,
    kernel_log_buffers_lost: u64,
    kernel_buffers_written: u64,
}

impl EventCountsDelta {
    /// True when every claimed delta is zero (nothing to fold this pass).
    ///
    /// Only used by the boundary-case tests (the production fold path always
    /// applies unconditionally, skipping zero deltas inside [`Self::apply`]).
    #[cfg(test)]
    fn is_empty(&self) -> bool {
        self.total == 0
            && self.dropped_slow_worker == 0
            && self.invalid == 0
            && self.kernel_events_lost == 0
            && self.kernel_real_time_buffers_lost == 0
            && self.kernel_log_buffers_lost == 0
            && self.kernel_buffers_written == 0
    }

    /// Fold the claimed deltas into `metrics`. Each non-zero delta becomes a
    /// counter `add()`; zero deltas are skipped so we never touch a counter we
    /// did not advance.
    fn apply(&self, metrics: &mut EtwReceiverMetrics) {
        if self.total > 0 {
            metrics.received_events_total.add(self.total);
        }
        if self.dropped_slow_worker > 0 {
            metrics
                .received_events_dropped_slow_worker
                .add(self.dropped_slow_worker);
        }
        if self.invalid > 0 {
            metrics.received_events_invalid.add(self.invalid);
        }
        if self.kernel_events_lost > 0 {
            metrics
                .received_events_lost_kernel
                .add(self.kernel_events_lost);
        }
        if self.kernel_real_time_buffers_lost > 0 {
            metrics
                .kernel_real_time_buffers_lost
                .add(self.kernel_real_time_buffers_lost);
        }
        if self.kernel_log_buffers_lost > 0 {
            metrics
                .kernel_log_buffers_lost
                .add(self.kernel_log_buffers_lost);
        }
        if self.kernel_buffers_written > 0 {
            metrics
                .kernel_buffers_written
                .add(self.kernel_buffers_written);
        }
    }
}

/// Atomically claim (via `swap(0)`) the running producer-side totals from the
/// shared session telemetry, returning the deltas for one fold pass.
///
/// Because every field is `swap(0)`-ed, the returned delta is claimed by
/// **exactly one** caller even when several per-core receivers race here
/// concurrently - there is no double counting. See
/// [`EtwReceiver::drain_event_counts`] for the cross-core aggregation and
/// shutdown-race reasoning.
fn take_event_counts(telemetry: &session::SessionWideMetrics) -> EventCountsDelta {
    #[inline]
    fn claim_if_nonzero(a: &std::sync::atomic::AtomicU64) -> u64 {
        if a.load(Ordering::Relaxed) == 0 {
            return 0;
        }
        a.swap(0, Ordering::Relaxed)
    }

    EventCountsDelta {
        total: telemetry.total.swap(0, Ordering::Relaxed),
        dropped_slow_worker: claim_if_nonzero(&telemetry.dropped_slow_worker),
        invalid: claim_if_nonzero(&telemetry.decode_failed),
        kernel_events_lost: claim_if_nonzero(&telemetry.kernel_events_lost),
        kernel_real_time_buffers_lost: claim_if_nonzero(&telemetry.kernel_real_time_buffers_lost),
        kernel_log_buffers_lost: claim_if_nonzero(&telemetry.kernel_log_buffers_lost),
        kernel_buffers_written: claim_if_nonzero(&telemetry.kernel_buffers_written),
    }
}

// ── Event processing loop ────────────────────────────────────────────────────

impl EtwReceiver {
    /// Fold the per-session atomic counters - written by the `!Send`
    /// `ProcessTrace` callback - into this core's metric set.
    ///
    /// Each atomic is `swap(0)`-ed so the value is consumed exactly once: the
    /// running total since the last drain becomes a counter `add()`. Call this
    /// on every `CollectTelemetry` tick and immediately before any terminal
    /// `snapshot()` so the final report is not lossy.
    ///
    /// ## Cross-core aggregation: per-field claim, summed across cores
    ///
    /// The `session_wide_metrics` atomics are **shared across all cores** of a
    /// `session_name` (one `Arc<SessionWideMetrics>` per session). Each field is
    /// drained with an independent `swap(0)`, so a given field's accumulated
    /// delta is claimed by **exactly one** core. The three swaps are *not* a
    /// single transaction: if two cores tick concurrently they may split the
    /// fields (one core claims `total`, another claims `dropped_slow_worker`,
    /// etc.). In the common, uncontended case a single core claims all three.
    ///
    /// This is correct for the aggregate because:
    ///
    /// * Each `swap(0)` is atomic, so every field's delta is claimed by exactly
    ///   one core - no double counting and no loss, even when two cores tick
    ///   concurrently and split the fields between them.
    /// * Every per-core `MetricSet` for this receiver shares the *same*
    ///   attribute key (the receiver registers metrics without a per-core
    ///   dimension), so the telemetry registry **sums** the per-core snapshots
    ///   under one series. Which core claimed which field is irrelevant; the
    ///   session-wide total is therefore exact.
    ///
    /// The trade-off is intentional: the producer-side counters
    /// (`received_events_total`, `received_events_dropped_slow_worker`, and
    /// `received_events_invalid`) are **session-scoped, not
    /// per-core**. They cannot be attributed to an individual core - if they
    /// were given a per-core attribute they would land on an arbitrary core
    /// each interval and produce noisy, meaningless per-core series.
    ///
    /// ## Shutdown race: residual delta after the last snapshot
    ///
    /// On `DrainIngress`/`Shutdown` each core calls this function once more
    /// before its terminal `snapshot()`. The `swap(0)` still guarantees no
    /// double counting across the concurrently-terminating cores. However, a
    /// producer increment that lands *after* the last surviving core has taken
    /// its terminal snapshot can no longer be drained (that core's loop has
    /// exited). Losing this final residual delta at teardown is acceptable: it
    /// is bounded by the events the `ProcessTrace` thread produces in the
    /// narrow window between the last snapshot and channel close, and the
    /// session thread is detached for the process lifetime anyway.
    fn drain_event_counts(&self) {
        let deltas = take_event_counts(&self.session_wide_metrics);
        let mut m = self.metrics.borrow_mut();
        deltas.apply(&mut m);
    }

    /// Main event loop when an ETW session is active.
    ///
    /// Consumes events from the per-core MPSC channel, accumulates them into
    /// Arrow batches, and flushes when either `max_size` or `max_duration`
    /// thresholds are reached.  Control messages are processed with priority.
    async fn run_event_loop(
        &mut self,
        ctrl_chan: &mut local::ControlChannel<OtapPdata>,
        effect_handler: &local::EffectHandler<OtapPdata>,
        telemetry_timer_handle: TelemetryTimerCancelHandle<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        let mut channel_alive = true;
        let mut builder = EtwArrowRecordsBuilder::new();
        let batch_max_size = self.batching.max_size.get();

        // Use `interval_at` so the first tick fires one `max_duration` from now
        // rather than immediately (which `interval` would do, causing a
        // pointless flush of an empty batch at startup).
        let mut flush_interval = time::interval_at(
            time::Instant::now() + self.batching.max_duration,
            self.batching.max_duration,
        );
        flush_interval.set_missed_tick_behavior(MissedTickBehavior::Delay);

        loop {
            tokio::select! {
                biased; // Prioritise control messages over data.

                ctrl_msg = ctrl_chan.recv() => {
                    match ctrl_msg {
                        Ok(NodeControlMsg::DrainIngress { deadline, .. }) => {
                            let _ = telemetry_timer_handle.cancel().await;

                            // Drain remaining events before flushing the final
                            // batch, but bound the work by a deadline.  The ETW
                            // `ProcessTrace` producer thread keeps round-robining
                            // events into this core's channel, so a busy provider
                            // would otherwise let `try_recv` spin past the
                            // deadline indefinitely and risk a forced kill.
                            let now = std::time::Instant::now();
                            let remaining = deadline.saturating_duration_since(now);
                            let drain_budget =
                                std::cmp::min(remaining * 9 / 10, MAX_DRAIN_BUDGET);
                            let drain_deadline = now + drain_budget;

                            while std::time::Instant::now() < drain_deadline {
                                match self.event_rx.try_recv() {
                                    Ok(event) => {
                                        builder.append(&event);
                                        if builder.len() >= batch_max_size {
                                            try_flush_batch(
                                                effect_handler,
                                                &self.metrics,
                                                &mut builder,
                                            )?;
                                        }
                                    }
                                    // Empty or Disconnected - nothing more to do.
                                    Err(_) => break,
                                }
                            }

                            // Flush the trailing partial batch (no-op if empty).
                            // Use the non-blocking flush so a slow downstream
                            // cannot park us past the deadline.
                            try_flush_batch(effect_handler, &self.metrics, &mut builder)?;
                            effect_handler.notify_receiver_drained().await?;
                            // Fold producer-side atomics before the final snapshot.
                            self.drain_event_counts();
                            let snapshot = self.metrics.borrow().snapshot();
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        }
                        Ok(NodeControlMsg::Shutdown { deadline, .. }) => {
                            let _ = telemetry_timer_handle.cancel().await;
                            otel_info!(
                                "etw_receiver.shutdown",
                                message = "ETW receiver shutting down",
                            );
                            // Fold producer-side atomics before the final snapshot.
                            self.drain_event_counts();
                            let snapshot = self.metrics.borrow().snapshot();
                            return Ok(TerminalState::new(deadline, [snapshot]));
                        }
                        Ok(NodeControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            // Fold the producer-side atomics into the metric set
                            // first (separate borrow scope), then report.
                            self.drain_event_counts();
                            let mut m = self.metrics.borrow_mut();
                            let _ = metrics_reporter.report(&mut m);
                        }
                        Ok(NodeControlMsg::MemoryPressureChanged { .. }) => {
                            // TODO: Implement shedding if needed.
                        }
                        Err(e) => {
                            return Err(Error::ChannelRecvError(e));
                        }
                        _ => {
                            // Other control messages - ignore for now.
                        }
                    }
                }

                // Timer-based batch flush. Skip once the channel is closed:
                // the final batch was already flushed when the channel ended,
                // so there is nothing left to flush periodically.
                _ = flush_interval.tick(), if channel_alive => {
                    flush_batch(effect_handler, &self.metrics, &mut builder).await?;
                }

                // Receive event data from the ETW session.
                // Only poll when the channel is still alive to avoid
                // spinning on a closed channel.
                event_data = self.event_rx.recv(), if channel_alive => {
                    match event_data {
                        Some(event) => {
                            builder.append(&event);

                            // Flush when batch is full
                            if builder.len() >= batch_max_size {
                                flush_batch(effect_handler, &self.metrics, &mut builder).await?;
                            }
                        }
                        None => {
                            // Channel closed - the ETW session thread has
                            // exited (process shutdown or unrecoverable error).
                            // Flush any remaining events, then stop polling.
                            // Use the non-blocking flush so a slow or
                            // already-closed downstream cannot park us while
                            // the session is tearing down.
                            try_flush_batch(effect_handler, &self.metrics, &mut builder)?;
                            channel_alive = false;
                            otel_info!(
                                "etw_receiver.session_ended",
                                message = "ETW session event channel closed",
                            );
                        }
                    }
                }
            }
        }
    }
}

// ── Factory registration ─────────────────────────────────────────────────────

/// Register the ETW receiver in the pipeline factory.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_RECEIVER_FACTORIES)]
pub static ETW_RECEIVER: ReceiverFactory<OtapPdata> = ReceiverFactory {
    name: ETW_RECEIVER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             receiver_config: &ReceiverConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ReceiverWrapper::local(
            EtwReceiver::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            receiver_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    supported_rate_units: &[],
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

// ── Receiver trait implementation ────────────────────────────────────────────

#[async_trait(?Send)]
impl local::Receiver<OtapPdata> for EtwReceiver {
    async fn start(
        mut self: Box<Self>,
        mut ctrl_chan: local::ControlChannel<OtapPdata>,
        effect_handler: local::EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        // Start periodic telemetry collection.
        let telemetry_timer_handle = effect_handler
            .start_periodic_telemetry(Duration::from_secs(1))
            .await?;

        otel_info!(
            "etw_receiver.start",
            session_name = self.config.session_name.as_str(),
            provider_count = self.config.providers.len(),
            batch_max_size = self.batching.max_size.get(),
            batch_max_duration_ms = self.batching.max_duration.as_millis() as u64,
        );

        self.run_event_loop(&mut ctrl_chan, &effect_handler, telemetry_timer_handle)
            .await
    }
}

// ── Telemetry ────────────────────────────────────────────────────────────────

/// Receiver-level metrics for the ETW receiver.
///
/// # Counter algebra
///
/// There is a single ingress denominator:
///
/// ```text
/// received_events_total
///     = received_events_forwarded        (events in successfully-sent batches)
///     + received_events_forward_failed   (events in batches that failed to build/send)
///     + received_events_dropped_slow_worker (events dropped by internal backpressure)
///     + events still buffered in the per-core channel and/or in the in-flight builder at snapshot time
/// ```
///
/// Derived rates:
///
/// * slow-worker drop rate = `received_events_dropped_slow_worker / received_events_total`
/// * forward-failure rate = `received_events_forward_failed / received_events_total`
///
/// Notes:
///
/// * `received_events_invalid` (TDH decode failures) is **orthogonal** to the
///   delivery accounting: a decode failure does *not* drop the event - it is
///   still enqueued and forwarded with empty `decoded_fields`. Treat it as a
///   quality signal, not a loss bucket.
/// * `received_events_total`, `received_events_dropped_slow_worker`, and
///   `received_events_invalid` are **session-scoped**
///   (shared across all per-core receivers of a `session_name`) - see
///   [`EtwReceiver::drain_event_counts`]. `received_events_forwarded`,
///   `received_events_forward_failed`, and `received_events_rejected_memory_pressure`
///   are per-core but reported under a single summed series.
#[metric_set(name = "receiver.etw")]
#[derive(Debug, Default, Clone)]
pub struct EtwReceiverMetrics {
    /// Total number of ETW events the session **produced** - counted on the
    /// producer (`ProcessTrace`) side before any channel send, including
    /// events about to be dropped due to a full per-core channel.
    ///
    /// This is the ingress denominator and the authoritative total. The
    /// slow-worker drop rate is `received_events_dropped_slow_worker /
    /// received_events_total`. See the "Counter algebra" section on
    /// [`EtwReceiverMetrics`].
    #[metric(unit = "{event}")]
    pub received_events_total: Counter<u64>,

    /// Number of ETW events successfully forwarded downstream.
    #[metric(unit = "{event}")]
    pub received_events_forwarded: Counter<u64>,

    /// Number of ETW events that failed to parse.
    #[metric(unit = "{event}")]
    pub received_events_invalid: Counter<u64>,

    /// Number of ETW events that could not be forwarded downstream and were
    /// therefore dropped. Counts both batch `build()` failures (an encoding
    /// bug) and send failures (the downstream channel was full or closed).
    #[metric(unit = "{event}")]
    pub received_events_forward_failed: Counter<u64>,

    /// Number of ETW events dropped due to process-wide memory pressure.
    #[metric(unit = "{event}")]
    pub received_events_rejected_memory_pressure: Counter<u64>,

    /// Events dropped because the consuming worker (this core's async receiver
    /// loop) couldn't drain the internal channel fast enough (the ETW consumer
    /// thread filled it). Contrast `received_events_forward_failed`, which is
    /// downstream backpressure.
    #[metric(unit = "{event}")]
    pub received_events_dropped_slow_worker: Counter<u64>,

    /// Events lost inside the kernel ETW buffers before `one_collect` ever saw
    /// them (buffer overrun), from `TraceStats::events_lost`. Distinct from
    /// `received_events_dropped_slow_worker`, which is our own downstream
    /// backpressure after the event was already received.
    #[metric(unit = "{event}")]
    pub received_events_lost_kernel: Counter<u64>,

    /// Real-time delivery buffers lost because the consumer could not drain
    /// the ETW real-time buffers fast enough, from
    /// `TraceStats::real_time_buffers_lost`.
    #[metric(unit = "{buffer}")]
    pub kernel_real_time_buffers_lost: Counter<u64>,

    /// Log buffers that could not be flushed, from
    /// `TraceStats::log_buffers_lost`.
    #[metric(unit = "{buffer}")]
    pub kernel_log_buffers_lost: Counter<u64>,

    /// Total ETW buffers written by the session, from
    /// `TraceStats::buffers_written`. A throughput/health denominator rather
    /// than a loss signal.
    #[metric(unit = "{buffer}")]
    pub kernel_buffers_written: Counter<u64>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_config(providers: Vec<ProviderConfig>) -> Config {
        Config {
            session_name: "test-session".to_string(),
            providers,
            batching: None,
        }
    }

    fn provider_with_guid(guid: &str) -> ProviderConfig {
        ProviderConfig {
            name: None,
            guid: Some(guid.to_string()),
            level: TraceLevel::default(),
            keywords: None,
        }
    }

    fn provider_with_name(name: &str) -> ProviderConfig {
        ProviderConfig {
            name: Some(name.to_string()),
            guid: None,
            level: TraceLevel::default(),
            keywords: None,
        }
    }

    #[test]
    fn validate_accepts_single_guid_provider() {
        let cfg = make_config(vec![provider_with_guid(
            "22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716",
        )]);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_accepts_single_name_provider() {
        let cfg = make_config(vec![provider_with_name("Microsoft-Windows-PowerShell")]);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_accepts_multiple_providers() {
        let cfg = make_config(vec![
            provider_with_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"),
            provider_with_name("Microsoft-Windows-PowerShell"),
        ]);
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn validate_rejects_empty_providers() {
        let cfg = make_config(vec![]);
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("at least one ETW provider"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn validate_rejects_both_name_and_guid() {
        let cfg = make_config(vec![ProviderConfig {
            name: Some("SomeProvider".to_string()),
            guid: Some("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716".to_string()),
            level: TraceLevel::default(),
            keywords: None,
        }]);
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("mutually exclusive"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn validate_rejects_neither_name_nor_guid() {
        let cfg = make_config(vec![ProviderConfig {
            name: None,
            guid: None,
            level: TraceLevel::default(),
            keywords: None,
        }]);
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("either 'name' or 'guid' must be specified"),
            "unexpected error: {msg}"
        );
    }

    #[test]
    fn validate_reports_correct_index() {
        let cfg = make_config(vec![
            provider_with_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716"),
            ProviderConfig {
                name: None,
                guid: None,
                level: TraceLevel::default(),
                keywords: None,
            },
        ]);
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("provider[1]"),
            "expected error at index 1, got: {msg}"
        );
    }

    #[test]
    fn validate_rejects_zero_batch_duration() {
        let cfg = Config {
            session_name: "test".to_string(),
            providers: vec![provider_with_guid("22fb2cd6-0e7b-422b-a0c7-2fad1fd0e716")],
            batching: Some(BatchConfig {
                max_size: NonZeroU16::new(100).expect("100 is non-zero"),
                max_duration: Duration::ZERO,
            }),
        };
        let err = cfg.validate().unwrap_err();
        let msg = err.to_string();
        assert!(
            msg.contains("max_duration"),
            "expected max_duration error, got: {msg}"
        );
    }

    #[test]
    fn default_batch_config() {
        let cfg = BatchConfig::default();
        assert_eq!(cfg.max_size, DEFAULT_BATCH_MAX_SIZE);
        assert_eq!(cfg.max_duration, DEFAULT_BATCH_MAX_DURATION);
    }

    // ── Producer-side atomic folding boundary cases ──────────────────
    //
    // These exercise the "first core drains the whole delta" design of
    // `take_event_counts` / `EventCountsDelta::apply` without needing a
    // real ETW session or a registered `MetricSet`:
    //
    // 1. A normal fold claims exactly the producer-side counts and applying
    //    them advances the metric counters.
    // 2. `swap(0)` semantics: a second claim on the same atomics, with no new
    //    producer increments in between, sees zero - so two concurrently
    //    ticking cores can never double-count.
    // 3. Terminal path: after a final drain, a *late* producer increment is
    //    left unclaimed (residual loss at teardown is acceptable and bounded).

    use session::SessionWideMetrics;
    use std::sync::atomic::Ordering;

    /// Simulate the producer (`ProcessTrace` callback) recording its
    /// per-event counters into the shared session telemetry.
    fn bump_producer(telemetry: &SessionWideMetrics, total: u64, dropped: u64, invalid: u64) {
        let _ = telemetry.total.fetch_add(total, Ordering::Relaxed);
        let _ = telemetry
            .dropped_slow_worker
            .fetch_add(dropped, Ordering::Relaxed);
        let _ = telemetry
            .decode_failed
            .fetch_add(invalid, Ordering::Relaxed);
    }

    #[test]
    fn take_event_counts_claims_full_delta_and_resets() {
        let telemetry = SessionWideMetrics::default();
        // 10 events total, 3 dropped slow-worker, 2 decode failures.
        bump_producer(&telemetry, 10, 3, 2);

        let deltas = take_event_counts(&telemetry);
        assert_eq!(
            deltas,
            EventCountsDelta {
                total: 10,
                dropped_slow_worker: 3,
                invalid: 2,
                ..Default::default()
            }
        );

        // The atomics are now drained to zero.
        assert_eq!(telemetry.total.load(Ordering::Relaxed), 0);
        assert_eq!(telemetry.dropped_slow_worker.load(Ordering::Relaxed), 0);
        assert_eq!(telemetry.decode_failed.load(Ordering::Relaxed), 0);

        // Applying the deltas advances the corresponding metric counters and
        // upholds the counter-algebra invariant.
        let mut metrics = EtwReceiverMetrics::default();
        deltas.apply(&mut metrics);
        assert_eq!(metrics.received_events_total.get(), 10);
        assert_eq!(metrics.received_events_dropped_slow_worker.get(), 3);
        assert_eq!(metrics.received_events_invalid.get(), 2);
        // Consumer-side counter is removed from metric set; only producer-side
        // `received_events_total` exists now.
    }

    #[test]
    fn take_event_counts_claims_kernel_trace_stats_and_resets() {
        let telemetry = SessionWideMetrics::default();
        let _ = telemetry.total.fetch_add(4, Ordering::Relaxed);
        let _ = telemetry.kernel_events_lost.fetch_add(2, Ordering::Relaxed);
        let _ = telemetry
            .kernel_real_time_buffers_lost
            .fetch_add(3, Ordering::Relaxed);
        let _ = telemetry
            .kernel_log_buffers_lost
            .fetch_add(1, Ordering::Relaxed);
        let _ = telemetry
            .kernel_buffers_written
            .fetch_add(9, Ordering::Relaxed);

        let deltas = take_event_counts(&telemetry);
        assert_eq!(
            deltas,
            EventCountsDelta {
                total: 4,
                kernel_events_lost: 2,
                kernel_real_time_buffers_lost: 3,
                kernel_log_buffers_lost: 1,
                kernel_buffers_written: 9,
                ..Default::default()
            }
        );

        assert_eq!(telemetry.total.load(Ordering::Relaxed), 0);
        assert_eq!(telemetry.kernel_events_lost.load(Ordering::Relaxed), 0);
        assert_eq!(
            telemetry
                .kernel_real_time_buffers_lost
                .load(Ordering::Relaxed),
            0
        );
        assert_eq!(telemetry.kernel_log_buffers_lost.load(Ordering::Relaxed), 0);
        assert_eq!(telemetry.kernel_buffers_written.load(Ordering::Relaxed), 0);

        let mut metrics = EtwReceiverMetrics::default();
        deltas.apply(&mut metrics);
        assert_eq!(metrics.received_events_total.get(), 4);
        assert_eq!(metrics.received_events_lost_kernel.get(), 2);
        assert_eq!(metrics.kernel_real_time_buffers_lost.get(), 3);
        assert_eq!(metrics.kernel_log_buffers_lost.get(), 1);
        assert_eq!(metrics.kernel_buffers_written.get(), 9);
    }

    #[test]
    fn second_claim_without_new_increments_is_empty_no_double_count() {
        // Models two cores ticking back-to-back: the first claim takes the
        // whole delta, the second sees nothing. This is exactly why
        // concurrent per-core drains cannot double count.
        let telemetry = SessionWideMetrics::default();
        bump_producer(&telemetry, 7, 1, 0);

        let first = take_event_counts(&telemetry);
        assert_eq!(first.total, 7);
        assert_eq!(first.dropped_slow_worker, 1);
        assert!(!first.is_empty());

        let second = take_event_counts(&telemetry);
        assert!(
            second.is_empty(),
            "second claim with no intervening producer increments must be empty, got {second:?}"
        );

        // Folding both into a single metric set sums to the producer total
        // exactly once (the registry sums per-core snapshots the same way).
        let mut metrics = EtwReceiverMetrics::default();
        first.apply(&mut metrics);
        second.apply(&mut metrics);
        assert_eq!(metrics.received_events_total.get(), 7);
        assert_eq!(metrics.received_events_dropped_slow_worker.get(), 1);
    }

    #[test]
    fn late_producer_increment_after_terminal_drain_is_residual_loss() {
        // Terminal path: a core performs its final drain, then the producer
        // records one more event before the channel actually closes. That
        // residual delta is left unclaimed - acceptable, bounded loss.
        let telemetry = SessionWideMetrics::default();
        bump_producer(&telemetry, 5, 0, 0);

        // Final drain on the last surviving core.
        let terminal = take_event_counts(&telemetry);
        assert_eq!(terminal.total, 5);

        let mut metrics = EtwReceiverMetrics::default();
        terminal.apply(&mut metrics);
        assert_eq!(metrics.received_events_total.get(), 5);

        // A late producer increment lands after the terminal snapshot.
        let _ = telemetry.total.fetch_add(1, Ordering::Relaxed);

        // The receiver loop has exited, so this residual is never folded into
        // the metric set - it remains visible only in the atomic, confirming
        // the documented "residual loss at teardown" behaviour.
        assert_eq!(telemetry.total.load(Ordering::Relaxed), 1);
        assert_eq!(
            metrics.received_events_total.get(),
            5,
            "metric must not reflect the post-terminal increment"
        );
    }

    #[test]
    fn empty_deltas_apply_is_noop() {
        let deltas = EventCountsDelta::default();
        assert!(deltas.is_empty());
        let mut metrics = EtwReceiverMetrics::default();
        deltas.apply(&mut metrics);
        assert_eq!(metrics.received_events_total.get(), 0);
        assert_eq!(metrics.received_events_dropped_slow_worker.get(), 0);
        assert_eq!(metrics.received_events_invalid.get(), 0);
        assert_eq!(metrics.received_events_lost_kernel.get(), 0);
        assert_eq!(metrics.kernel_real_time_buffers_lost.get(), 0);
        assert_eq!(metrics.kernel_log_buffers_lost.get(), 0);
        assert_eq!(metrics.kernel_buffers_written.get(), 0);
    }
}

/// End-to-end Windows integration tests for the ETW receiver. See
/// [`windows_e2e_tests`] for details.
#[cfg(test)]
mod windows_e2e_tests;
