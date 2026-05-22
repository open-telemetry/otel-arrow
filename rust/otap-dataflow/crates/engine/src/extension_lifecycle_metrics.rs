// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-extension lifecycle metrics.
//!
//! Two pieces live here:
//!
//! 1. [`ExtensionLifecycleMetrics`] — the `metric_set` definition that
//!    backs every per-extension row in the registry.
//! 2. [`ExtensionMetricsEntry`] — the per-extension state wrapper held
//!    by `ExtensionLifecycle`. It carries a canonical copy of every
//!    gauge value alongside the [`MetricSet`] so the framework's
//!    `clear_values` (which zeroes gauges after each publish) doesn't
//!    lose the last observed value. Every [`Self::publish`] re-applies
//!    the canonical gauges before snapshotting into the registry.
//!
//! All metric state mutations live here as methods on
//! `ExtensionMetricsEntry`. The lifecycle owns key lookup and
//! orchestration; this module owns the per-entry semantics
//! (idempotency, gauge canonicalization, counter increments).

use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::{MetricSet, MetricSetHandler};
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry_macros::metric_set;
use std::time::Instant;

/// Per-extension lifecycle metrics. Receivers/processors/exporters
/// surface drain progress via `pipeline.runtime_control`; extensions
/// need a dedicated per-instance set because they shut down strictly
/// after the data path drains and have their own grace window.
#[metric_set(name = "extension.lifecycle")]
#[derive(Debug, Default, Clone)]
pub struct ExtensionLifecycleMetrics {
    /// `1` while the extension task is running, `0` after it terminates.
    #[metric(name = "active", unit = "{1}")]
    pub active: Gauge<u64>,
    /// `1` once `Shutdown` has been successfully enqueued into the
    /// extension's control channel, `0` otherwise. **Not** a delivery
    /// receipt: the extension may still not have polled `recv()`. It
    /// only marks "we successfully handed the message to the
    /// channel"; cooperative honoring is observed via `shutdown.duration`
    /// (set on terminal exit) and `shutdown.timeout` (set if the
    /// grace window expires before exit).
    #[metric(name = "shutdown.sent", unit = "{1}")]
    pub shutdown_sent: Gauge<u64>,
    /// Nanoseconds between successful `Shutdown` enqueue and task
    /// exit. Meaningful only when `shutdown.sent == 1`.
    #[metric(name = "shutdown.duration", unit = "ns")]
    pub shutdown_duration_ns: Gauge<u64>,
    /// Count of times the extension failed to exit within the
    /// shutdown grace window after the `Shutdown` message was sent.
    /// Not incremented when the send itself failed.
    #[metric(name = "shutdown.timeout", unit = "{1}")]
    pub shutdown_timeout: Counter<u64>,
    /// Count of `Shutdown` sends that failed to reach the
    /// extension's control channel — either the channel was closed
    /// (receiver dropped) or the send did not complete before the
    /// shutdown grace deadline.
    #[metric(name = "shutdown.send_failed", unit = "{1}")]
    pub shutdown_send_failed: Counter<u64>,
    /// Count of task errors (an `Err(_)` return from `start()`,
    /// including caught panics). Not incremented for runtime
    /// cancellation.
    #[metric(name = "task.error", unit = "{1}")]
    pub task_error: Counter<u64>,
    /// Count of extensions whose task resolved before
    /// `broadcast_shutdown` was called. Extensions are expected to
    /// run until they receive `Shutdown`; exiting earlier — even
    /// cleanly via `Ok(())` — is a lifecycle-contract anomaly worth
    /// surfacing. Orthogonal to `task.error`: a clean early exit
    /// bumps only this; an early error bumps both. Cancellation by
    /// the runtime is not the extension's choice and is not counted.
    #[metric(name = "task.early_exit", unit = "{1}")]
    pub task_early_exit: Counter<u64>,
}

/// Per-extension lifecycle state owned by
/// `super::extension_lifecycle::ExtensionLifecycle`.
///
/// Carries a canonical copy of every gauge value alongside the
/// [`MetricSet`]; every publish re-applies those canonical values
/// before writing to the registry. Required because the framework's
/// `clear_values` resets gauges to zero after every publish, so the
/// canonical mirror is the only durable record of the last observed
/// gauge value.
pub(crate) struct ExtensionMetricsEntry {
    metrics: MetricSet<ExtensionLifecycleMetrics>,
    /// Set once a terminal publish has been emitted, to prevent
    /// double-counting across normal and timeout paths.
    finalized: bool,
    /// Anchor for `shutdown.duration`; `None` if `Shutdown` was
    /// never successfully sent into the channel.
    shutdown_sent_at: Option<Instant>,
    /// Canonical gauge values, mirrored into `metrics` on every publish.
    active: u64,
    shutdown_sent: u64,
    shutdown_duration_ns: Option<u64>,
}

impl ExtensionMetricsEntry {
    /// Build a fresh entry for a just-spawned extension. `active = 1`
    /// from the start so the first publish observes running state.
    pub(crate) fn new(metrics: MetricSet<ExtensionLifecycleMetrics>) -> Self {
        Self {
            metrics,
            finalized: false,
            shutdown_sent_at: None,
            active: 1,
            shutdown_sent: 0,
            shutdown_duration_ns: None,
        }
    }

    /// Apply canonical gauges to the metric set, accumulate a
    /// snapshot directly into the registry, then `clear_values` so
    /// counter deltas don't double-count on the next publish.
    /// Gauges are restored from canonical state at the top of the
    /// next call, so the post-clear zero doesn't leak out.
    pub(crate) fn publish(&mut self, registry: &TelemetryRegistryHandle) {
        self.metrics.active.set(self.active);
        self.metrics.shutdown_sent.set(self.shutdown_sent);
        if let Some(d) = self.shutdown_duration_ns {
            self.metrics.shutdown_duration_ns.set(d);
        }
        let snap = self.metrics.snapshot();
        registry.accumulate_metric_set_snapshot(snap.key(), snap.get_metrics());
        self.metrics.clear_values();
    }

    /// Returns `true` once a terminal recorder has run for this
    /// entry. The lifecycle uses this to skip broadcast sends to
    /// extensions that already exited (cleanly or via crash) before
    /// shutdown — their receivers are dropped and sending would
    /// attribute a spurious `shutdown.send_failed` on top of the
    /// earlier `task.error` / cancellation.
    pub(crate) fn is_finalized(&self) -> bool {
        self.finalized
    }

    /// Stamp successful `Shutdown` enqueue so a subsequent terminal
    /// publish can compute `shutdown.duration`. `at` is the instant
    /// the send completed — capture it at the call site so it isn't
    /// skewed by post-send work. No-op if the entry has already been
    /// finalized: an earlier terminal publish is authoritative.
    ///
    /// **Not a delivery receipt** — sets `shutdown.sent`, meaning
    /// the message landed in the channel buffer. Whether the
    /// extension actually consumed and honored it is observable only
    /// via the terminal `shutdown.duration` / `shutdown.timeout`
    /// signals.
    pub(crate) fn mark_shutdown_sent(&mut self, at: Instant) {
        if self.finalized {
            return;
        }
        self.shutdown_sent_at = Some(at);
        self.shutdown_sent = 1;
    }

    /// Terminal: task completed (`Ok` or `Err`). Stamps
    /// `shutdown.duration` (only when the send succeeded), flips
    /// `active` to 0, bumps `task.error` on failure, bumps
    /// `task.early_exit` when the task resolved before
    /// `broadcast_shutdown` ran, then publishes.
    pub(crate) fn record_completion(
        &mut self,
        registry: &TelemetryRegistryHandle,
        is_error: bool,
        pre_broadcast: bool,
    ) {
        if self.finalized {
            return;
        }
        self.finalized = true;
        if let Some(started) = self.shutdown_sent_at {
            let elapsed = started.elapsed().as_nanos() as u64;
            self.shutdown_duration_ns = Some(elapsed);
        }
        self.active = 0;
        if is_error {
            self.metrics.task_error.inc();
        }
        if pre_broadcast {
            self.metrics.task_early_exit.inc();
        }
        self.publish(registry);
    }

    /// Terminal: drain-deadline timeout. Only bumps
    /// `shutdown.timeout` when `Shutdown` was actually sent into the
    /// channel — otherwise the failure is already attributed via
    /// `shutdown.send_failed`.
    pub(crate) fn record_timeout(&mut self, registry: &TelemetryRegistryHandle) {
        if self.finalized {
            return;
        }
        self.finalized = true;
        if self.shutdown_sent_at.is_some() {
            self.metrics.shutdown_timeout.inc();
        }
        self.active = 0;
        self.publish(registry);
    }

    /// Non-terminal: bump `shutdown.send_failed` and publish. The
    /// task is still running; `active` stays at its current value.
    pub(crate) fn record_send_failed(&mut self, registry: &TelemetryRegistryHandle) {
        self.metrics.shutdown_send_failed.inc();
        self.publish(registry);
    }

    /// Terminal: task cancelled by the runtime. Flips `active=0`
    /// without bumping `task.error` — cancellation is not an
    /// extension fault.
    pub(crate) fn record_cancellation(&mut self, registry: &TelemetryRegistryHandle) {
        if self.finalized {
            return;
        }
        self.finalized = true;
        self.active = 0;
        self.publish(registry);
    }

    /// Drop-time safety net: publish terminal state so the registry
    /// always observes `active=0` for an entry that didn't reach a
    /// `record_*` path. **Always** publishes — the per-`finalized`
    /// guard only governs whether to flip `active`, not whether to
    /// publish, so a follow-up Drop after a normal `record_*` still
    /// re-asserts the terminal gauge state (idempotent: clean
    /// counter delta of 0, gauges unchanged).
    pub(crate) fn finalize_on_drop(&mut self, registry: &TelemetryRegistryHandle) {
        if !self.finalized {
            self.finalized = true;
            self.active = 0;
        }
        self.publish(registry);
    }
}

#[cfg(test)]
impl ExtensionMetricsEntry {
    pub(crate) fn active(&self) -> u64 {
        self.active
    }

    pub(crate) fn shutdown_sent(&self) -> u64 {
        self.shutdown_sent
    }

    pub(crate) fn shutdown_duration_ns(&self) -> Option<u64> {
        self.shutdown_duration_ns
    }
}
