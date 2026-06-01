// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-scope extension lifecycle and telemetry monitor. Owns its own
//! tick cadence; the host plugs `next_tick()` into its `select!` and
//! feeds events via [`ExtensionLifecycleEvent`].

use crate::context::ExtensionContext;
use crate::control::{ExtensionControlMsg, ExtensionControlSender};
use crate::extension::wrapper::ExtensionVariant;
use otap_df_config::ExtensionId;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::otel_warn;
use otap_df_telemetry::registry::{EntityKey, TelemetryRegistryHandle};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use std::future;
use std::time::{Duration, Instant};
use tokio::time::{Interval, MissedTickBehavior, interval_at};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ExtensionRuntimeState {
    Pending = 0,
    Spawned = 1,
    ShutdownSent = 2,
    CompletedOk = 3,
    Failed = 4,
    TimedOut = 5,
}

#[derive(Debug, Clone)]
pub(crate) enum ExtensionOutcome {
    Ok,
    Err(#[allow(dead_code)] String),
    JoinPanic,
    JoinCancelled,
    ShutdownTimeout,
}

/// Composite key identifying one extension instance at a scope.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub(crate) struct ExtensionKey {
    pub id: ExtensionId,
    pub variant: ExtensionVariant,
}

impl ExtensionKey {
    pub(crate) fn new(id: ExtensionId, variant: ExtensionVariant) -> Self {
        Self { id, variant }
    }

    #[cfg(test)]
    pub(crate) fn local(id: impl Into<ExtensionId>) -> Self {
        Self::new(id.into(), ExtensionVariant::Local)
    }
}

#[derive(Debug, Clone)]
pub(crate) enum ExtensionLifecycleEvent {
    Spawned {
        key: ExtensionKey,
    },
    ShutdownSent {
        key: ExtensionKey,
    },
    Completed {
        key: ExtensionKey,
        outcome: ExtensionOutcome,
    },
}

#[metric_set(name = "extension.lifecycle")]
#[derive(Debug, Default, Clone)]
pub struct ExtensionLifecycleMetrics {
    #[metric(unit = "{spawn}")]
    pub spawned: Counter<u64>,

    #[metric(unit = "{shutdown}")]
    pub shutdown_sent: Counter<u64>,

    #[metric(unit = "{completion}")]
    pub completed_ok: Counter<u64>,

    #[metric(unit = "{completion}")]
    pub completed_error: Counter<u64>,

    #[metric(unit = "{completion}")]
    pub completed_panic: Counter<u64>,

    #[metric(unit = "{completion}")]
    pub completed_cancelled: Counter<u64>,

    #[metric(unit = "{timeout}")]
    pub shutdown_timeout: Counter<u64>,

    #[metric(unit = "{state}")]
    pub state: Gauge<u64>,
}

struct ExtensionMonitorEntry {
    key: ExtensionKey,
    state: ExtensionRuntimeState,
    lifecycle_metrics: MetricSet<ExtensionLifecycleMetrics>,
    control_sender: Option<ExtensionControlSender>,
    registry: TelemetryRegistryHandle,
}

impl Drop for ExtensionMonitorEntry {
    fn drop(&mut self) {
        let _ = self
            .registry
            .unregister_metric_set(self.lifecycle_metrics.metric_set_key());
    }
}

pub(crate) struct ExtensionMetricsMonitor {
    registry: TelemetryRegistryHandle,
    entries: Vec<ExtensionMonitorEntry>,
    interval: Option<Interval>,
    collect_telemetry_interval: Duration,
    last_collect_telemetry: Option<Instant>,
}

impl ExtensionMetricsMonitor {
    pub(crate) fn new(
        host_ctx: ExtensionContext,
        tick_interval: Duration,
        collect_telemetry_interval: Duration,
    ) -> Self {
        let registry = host_ctx.metrics_registry();

        let start = tokio::time::Instant::now() + tick_interval;
        let mut interval = interval_at(start, tick_interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        Self {
            registry,
            entries: Vec::new(),
            interval: Some(interval),
            collect_telemetry_interval,
            last_collect_telemetry: None,
        }
    }

    /// No-op monitor used when extension telemetry is disabled via policy.
    pub(crate) fn disabled(host_ctx: ExtensionContext) -> Self {
        Self {
            registry: host_ctx.metrics_registry(),
            entries: Vec::new(),
            interval: None,
            collect_telemetry_interval: Duration::from_secs(u64::MAX / 2),
            last_collect_telemetry: None,
        }
    }

    /// Register an extension. `control_sender` is `Some` for active
    /// extensions, `None` for passive.
    pub(crate) fn register(
        &mut self,
        host_ctx: &ExtensionContext,
        key: ExtensionKey,
        entity_key: EntityKey,
        control_sender: Option<ExtensionControlSender>,
    ) {
        if self.interval.is_none() {
            return;
        }
        let lifecycle_metrics =
            host_ctx.register_metric_set_for_entity::<ExtensionLifecycleMetrics>(entity_key);
        let mut entry = ExtensionMonitorEntry {
            key,
            state: ExtensionRuntimeState::Pending,
            lifecycle_metrics,
            control_sender,
            registry: self.registry.clone(),
        };
        entry
            .lifecycle_metrics
            .state
            .set(ExtensionRuntimeState::Pending as u64);
        self.entries.push(entry);
    }

    pub(crate) fn apply_event(&mut self, event: ExtensionLifecycleEvent) {
        if self.interval.is_none() {
            return;
        }
        match event {
            ExtensionLifecycleEvent::Spawned { key } => self.on_spawned(&key),
            ExtensionLifecycleEvent::ShutdownSent { key } => self.on_shutdown_sent(&key),
            ExtensionLifecycleEvent::Completed { key, outcome } => self.on_completed(&key, outcome),
        }
    }

    #[allow(dead_code)]
    pub(crate) fn apply_events<I>(&mut self, events: I)
    where
        I: IntoIterator<Item = ExtensionLifecycleEvent>,
    {
        for ev in events {
            self.apply_event(ev);
        }
    }

    fn entry_mut(&mut self, key: &ExtensionKey) -> Option<&mut ExtensionMonitorEntry> {
        self.entries.iter_mut().find(|e| &e.key == key)
    }

    fn on_spawned(&mut self, key: &ExtensionKey) {
        if let Some(entry) = self.entry_mut(key)
            && matches!(entry.state, ExtensionRuntimeState::Pending)
        {
            entry.state = ExtensionRuntimeState::Spawned;
            entry.lifecycle_metrics.spawned.add(1);
            entry
                .lifecycle_metrics
                .state
                .set(ExtensionRuntimeState::Spawned as u64);
        }
    }

    fn on_shutdown_sent(&mut self, key: &ExtensionKey) {
        if let Some(entry) = self.entry_mut(key)
            && matches!(entry.state, ExtensionRuntimeState::Spawned)
        {
            entry.state = ExtensionRuntimeState::ShutdownSent;
            entry.lifecycle_metrics.shutdown_sent.add(1);
            entry
                .lifecycle_metrics
                .state
                .set(ExtensionRuntimeState::ShutdownSent as u64);
        }
    }

    fn on_completed(&mut self, key: &ExtensionKey, outcome: ExtensionOutcome) {
        let Some(entry) = self.entry_mut(key) else {
            return;
        };
        if matches!(
            entry.state,
            ExtensionRuntimeState::CompletedOk
                | ExtensionRuntimeState::Failed
                | ExtensionRuntimeState::TimedOut
        ) {
            return;
        }
        let new_state = match outcome {
            ExtensionOutcome::Ok => {
                entry.lifecycle_metrics.completed_ok.add(1);
                ExtensionRuntimeState::CompletedOk
            }
            ExtensionOutcome::Err(_) => {
                entry.lifecycle_metrics.completed_error.add(1);
                ExtensionRuntimeState::Failed
            }
            ExtensionOutcome::JoinPanic => {
                entry.lifecycle_metrics.completed_panic.add(1);
                ExtensionRuntimeState::Failed
            }
            ExtensionOutcome::JoinCancelled => {
                entry.lifecycle_metrics.completed_cancelled.add(1);
                ExtensionRuntimeState::Failed
            }
            ExtensionOutcome::ShutdownTimeout => {
                entry.lifecycle_metrics.shutdown_timeout.add(1);
                ExtensionRuntimeState::TimedOut
            }
        };
        entry.state = new_state;
        entry.lifecycle_metrics.state.set(new_state as u64);
    }

    /// Marks non-terminal entries (`Pending`/`Spawned`/`ShutdownSent`)
    /// as `TimedOut`. Used by the host after the drain deadline elapses.
    pub(crate) fn mark_pending_as_timeout(&mut self) {
        if self.interval.is_none() {
            return;
        }
        let pending: Vec<ExtensionKey> = self
            .entries
            .iter()
            .filter(|e| {
                matches!(
                    e.state,
                    ExtensionRuntimeState::Pending
                        | ExtensionRuntimeState::Spawned
                        | ExtensionRuntimeState::ShutdownSent
                )
            })
            .map(|e| e.key.clone())
            .collect();
        for key in pending {
            self.on_completed(&key, ExtensionOutcome::ShutdownTimeout);
        }
    }

    /// Awaits the next monitor tick. Never resolves when disabled.
    pub(crate) async fn next_tick(&mut self) -> Instant {
        match self.interval.as_mut() {
            Some(interval) => interval.tick().await.into_std(),
            None => future::pending::<Instant>().await,
        }
    }

    pub(crate) fn tick(&mut self, now: Instant, reporter: &mut MetricsReporter) {
        if self.interval.is_none() {
            return;
        }
        self.maybe_collect_telemetry(now, reporter);
        self.refresh_state_gauges();
        self.report(reporter);
    }

    /// Re-asserts each entry's `state` gauge so long-running extensions
    /// stay visible on every scrape regardless of framework reset semantics.
    pub(crate) fn refresh_state_gauges(&mut self) {
        for entry in &mut self.entries {
            entry.lifecycle_metrics.state.set(entry.state as u64);
        }
    }

    fn maybe_collect_telemetry(&mut self, now: Instant, reporter: &MetricsReporter) {
        let due = match self.last_collect_telemetry {
            None => true,
            Some(last) => now.duration_since(last) >= self.collect_telemetry_interval,
        };
        if !due {
            return;
        }
        self.last_collect_telemetry = Some(now);
        for entry in &self.entries {
            if !matches!(entry.state, ExtensionRuntimeState::Spawned) {
                continue;
            }
            let Some(sender) = entry.control_sender.as_ref() else {
                continue;
            };
            let msg = ExtensionControlMsg::CollectTelemetry {
                metrics_reporter: reporter.clone(),
            };
            if let Err(err) = sender.sender.try_send(msg) {
                otel_warn!(
                    "extension.collect_telemetry.try_send_failed",
                    extension = entry.key.id.as_ref(),
                    error = format!("{err}"),
                );
            }
        }
    }

    fn report(&mut self, reporter: &mut MetricsReporter) {
        for entry in &mut self.entries {
            if let Err(err) = reporter.report(&mut entry.lifecycle_metrics) {
                otel_warn!(
                    "extension.lifecycle.metrics.reporting.fail",
                    extension = entry.key.id.as_ref(),
                    error = err.to_string(),
                );
            }
        }
    }

    /// Returns the lifecycle state for `key`, or `None` if absent.
    #[cfg(test)]
    pub(crate) fn state_for(&self, key: &ExtensionKey) -> Option<ExtensionRuntimeState> {
        self.entries.iter().find(|e| &e.key == key).map(|e| e.state)
    }

    /// Returns the `completed_panic` counter for `key`, or `None` if absent.
    #[cfg(test)]
    pub(crate) fn completed_panic_count(&self, key: &ExtensionKey) -> Option<u64> {
        self.entries
            .iter()
            .find(|e| &e.key == key)
            .map(|e| e.lifecycle_metrics.completed_panic.get())
    }

    /// Returns the `completed_cancelled` counter for `key`, or `None` if absent.
    #[cfg(test)]
    #[allow(dead_code)] // kept for symmetry with completed_panic_count
    pub(crate) fn completed_cancelled_count(&self, key: &ExtensionKey) -> Option<u64> {
        self.entries
            .iter()
            .find(|e| &e.key == key)
            .map(|e| e.lifecycle_metrics.completed_cancelled.get())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::ExtensionControlSender;
    use crate::message::Sender;
    use otap_df_channel::mpsc;
    use std::time::Duration;

    fn fresh_monitor() -> (ExtensionMetricsMonitor, ExtensionContext) {
        let (ctx, _registry) = crate::testing::test_extension_ctx();
        let monitor = ExtensionMetricsMonitor::new(
            ctx.clone(),
            Duration::from_millis(50),
            Duration::from_millis(50),
        );
        (monitor, ctx)
    }

    fn make_sender(id: &str) -> ExtensionControlSender {
        let (tx, _rx) = mpsc::Channel::new(8);
        ExtensionControlSender {
            name: id.to_string().into(),
            sender: Sender::new_local_mpsc_sender(tx),
        }
    }

    fn count_in_state(monitor: &ExtensionMetricsMonitor, state: ExtensionRuntimeState) -> usize {
        monitor
            .entries
            .iter()
            .filter(|e| std::mem::discriminant(&e.state) == std::mem::discriminant(&state))
            .count()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn register_creates_per_extension_metric_set() {
        let (mut monitor, ctx) = fresh_monitor();
        let ext_key = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        monitor.register(&ctx, ExtensionKey::local("ext1"), ext_key, None);
        assert_eq!(monitor.entries.len(), 1);
        assert!(matches!(
            monitor.entries[0].state,
            ExtensionRuntimeState::Pending
        ));
        monitor.refresh_state_gauges();
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.state.get(),
            ExtensionRuntimeState::Pending as u64
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn lifecycle_transitions_update_per_entry_state() {
        let (mut monitor, ctx) = fresh_monitor();
        let ext_key = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext1");
        monitor.register(&ctx, key.clone(), ext_key, Some(make_sender("ext1")));

        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: key.clone() });
        monitor.refresh_state_gauges();
        assert_eq!(count_in_state(&monitor, ExtensionRuntimeState::Spawned), 1);
        assert_eq!(monitor.entries[0].lifecycle_metrics.spawned.get(), 1);

        monitor.apply_event(ExtensionLifecycleEvent::ShutdownSent { key: key.clone() });
        monitor.refresh_state_gauges();
        assert_eq!(count_in_state(&monitor, ExtensionRuntimeState::Spawned), 0);
        assert_eq!(
            count_in_state(&monitor, ExtensionRuntimeState::ShutdownSent),
            1
        );

        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: key.clone(),
            outcome: ExtensionOutcome::Ok,
        });
        monitor.refresh_state_gauges();
        assert_eq!(
            count_in_state(&monitor, ExtensionRuntimeState::ShutdownSent),
            0
        );
        assert_eq!(monitor.entries[0].lifecycle_metrics.completed_ok.get(), 1);
        assert!(matches!(
            monitor.entries[0].state,
            ExtensionRuntimeState::CompletedOk
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn local_and_shared_same_id_tracked_independently() {
        let (mut monitor, ctx) = fresh_monitor();
        let ext_key = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let local = ExtensionKey::new("ext1".into(), ExtensionVariant::Local);
        let shared = ExtensionKey::new("ext1".into(), ExtensionVariant::Shared);
        monitor.register(&ctx, local.clone(), ext_key, None);
        monitor.register(&ctx, shared.clone(), ext_key, None);
        assert_eq!(monitor.entries.len(), 2);

        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: local.clone() });
        assert!(matches!(
            monitor.entries[0].state,
            ExtensionRuntimeState::Spawned
        ));
        assert!(matches!(
            monitor.entries[1].state,
            ExtensionRuntimeState::Pending
        ));
        monitor.refresh_state_gauges();
        assert_eq!(count_in_state(&monitor, ExtensionRuntimeState::Spawned), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_sent_is_idempotent() {
        let (mut monitor, ctx) = fresh_monitor();
        let ext_key = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext1");
        monitor.register(&ctx, key.clone(), ext_key, None);
        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: key.clone() });
        monitor.apply_event(ExtensionLifecycleEvent::ShutdownSent { key: key.clone() });
        monitor.apply_event(ExtensionLifecycleEvent::ShutdownSent { key: key.clone() });
        monitor.refresh_state_gauges();
        assert_eq!(
            count_in_state(&monitor, ExtensionRuntimeState::ShutdownSent),
            1
        );
        assert_eq!(monitor.entries[0].lifecycle_metrics.shutdown_sent.get(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn failure_outcomes_count_per_entry_failures() {
        let (mut monitor, ctx) = fresh_monitor();
        for name in ["a", "b", "c"] {
            let ent = ctx.register_extension_entity(name.into(), ExtensionVariant::Local);
            let key = ExtensionKey::local(name);
            monitor.register(&ctx, key.clone(), ent, None);
            monitor.apply_event(ExtensionLifecycleEvent::Spawned { key });
        }
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: ExtensionKey::local("a"),
            outcome: ExtensionOutcome::Err("boom".into()),
        });
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: ExtensionKey::local("b"),
            outcome: ExtensionOutcome::JoinPanic,
        });
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: ExtensionKey::local("c"),
            outcome: ExtensionOutcome::JoinCancelled,
        });
        assert_eq!(count_in_state(&monitor, ExtensionRuntimeState::Failed), 3);
        assert_eq!(count_in_state(&monitor, ExtensionRuntimeState::TimedOut), 0);
        let a = monitor
            .entries
            .iter()
            .find(|e| e.key.id.as_ref() == "a")
            .unwrap();
        assert_eq!(a.lifecycle_metrics.completed_error.get(), 1);
        let b = monitor
            .entries
            .iter()
            .find(|e| e.key.id.as_ref() == "b")
            .unwrap();
        assert_eq!(b.lifecycle_metrics.completed_panic.get(), 1);
        let c = monitor
            .entries
            .iter()
            .find(|e| e.key.id.as_ref() == "c")
            .unwrap();
        assert_eq!(c.lifecycle_metrics.completed_cancelled.get(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn shutdown_timeout_counts_per_entry_timed_out() {
        let (mut monitor, ctx) = fresh_monitor();
        let ent = ctx.register_extension_entity("a".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("a");
        monitor.register(&ctx, key.clone(), ent, None);
        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: key.clone() });
        monitor.apply_event(ExtensionLifecycleEvent::ShutdownSent { key: key.clone() });
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key,
            outcome: ExtensionOutcome::ShutdownTimeout,
        });
        assert_eq!(count_in_state(&monitor, ExtensionRuntimeState::TimedOut), 1);
        assert_eq!(count_in_state(&monitor, ExtensionRuntimeState::Failed), 0);
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.shutdown_timeout.get(),
            1
        );
        assert!(matches!(
            monitor.entries[0].state,
            ExtensionRuntimeState::TimedOut
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async fn refresh_state_gauges_reasserts_after_clear() {
        // The `state` gauge is the monitor's only long-running absolute
        // value. If anything ever clears it between lifecycle events,
        // `refresh_state_gauges` must re-assert it from cached state.
        let (mut monitor, ctx) = fresh_monitor();
        let ent = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext1");
        monitor.register(&ctx, key.clone(), ent, None);
        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key });

        // After the event, the entry is in Spawned and the gauge
        // reflects that on the next refresh.
        monitor.refresh_state_gauges();
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.state.get(),
            ExtensionRuntimeState::Spawned as u64
        );

        // Force-zero the gauge to simulate a clear; the cached state
        // must be re-asserted on the next tick.
        monitor.entries[0].lifecycle_metrics.state.reset();
        assert_eq!(monitor.entries[0].lifecycle_metrics.state.get(), 0);

        monitor.refresh_state_gauges();
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.state.get(),
            ExtensionRuntimeState::Spawned as u64,
            "refresh_state_gauges must re-assert from cached entry.state"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn local_and_shared_get_distinct_metric_set_keys() {
        let (mut monitor, ctx) = fresh_monitor();
        let local_ent = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let shared_ent = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Shared);
        let local = ExtensionKey::new("ext1".into(), ExtensionVariant::Local);
        let shared = ExtensionKey::new("ext1".into(), ExtensionVariant::Shared);
        monitor.register(&ctx, local, local_ent, None);
        monitor.register(&ctx, shared, shared_ent, None);

        let local_key = monitor.entries[0].lifecycle_metrics.metric_set_key();
        let shared_key = monitor.entries[1].lifecycle_metrics.metric_set_key();
        assert_ne!(
            local_key, shared_key,
            "local and shared variants of the same extension id must own distinct MetricSetKeys"
        );

        // Mutate one variant's counter and confirm the other is
        // untouched — proving the storage really is independent.
        monitor.entries[0].lifecycle_metrics.spawned.add(7);
        assert_eq!(monitor.entries[0].lifecycle_metrics.spawned.get(), 7);
        assert_eq!(monitor.entries[1].lifecycle_metrics.spawned.get(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn disabled_monitor_is_inert() {
        let (ctx, _registry) = crate::testing::test_extension_ctx();
        let mut monitor = ExtensionMetricsMonitor::disabled(ctx.clone());
        let ent = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext1");
        monitor.register(&ctx, key.clone(), ent, None);
        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key });
        assert!(monitor.entries.is_empty());
        assert!(monitor.interval.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn dropping_monitor_unregisters_metric_sets() {
        let (ctx, registry) = crate::testing::test_extension_ctx();
        let before = registry.metric_set_count();
        {
            let mut monitor = ExtensionMetricsMonitor::new(
                ctx.clone(),
                Duration::from_millis(50),
                Duration::from_millis(50),
            );
            for name in ["a", "b"] {
                let ent = ctx.register_extension_entity(name.into(), ExtensionVariant::Local);
                monitor.register(&ctx, ExtensionKey::local(name), ent, None);
            }
            assert_eq!(registry.metric_set_count(), before + 2);
        }
        assert_eq!(registry.metric_set_count(), before);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn mark_pending_as_timeout_transitions_stragglers() {
        // After the drain deadline elapses, any non-terminal entry
        // must surface as TimedOut — including `Pending` (task
        // cancelled before first poll, JoinSet returned a keyless
        // JoinError) so the state gauge does not stick at Pending.
        let (mut monitor, ctx) = fresh_monitor();
        for name in ["never_spawned", "spawned", "shutting", "ok"] {
            let ent = ctx.register_extension_entity(name.into(), ExtensionVariant::Local);
            monitor.register(&ctx, ExtensionKey::local(name), ent, None);
        }
        // "never_spawned" stays in Pending: no Spawned event ever fires.
        monitor.apply_event(ExtensionLifecycleEvent::Spawned {
            key: ExtensionKey::local("spawned"),
        });
        monitor.apply_event(ExtensionLifecycleEvent::Spawned {
            key: ExtensionKey::local("shutting"),
        });
        monitor.apply_event(ExtensionLifecycleEvent::ShutdownSent {
            key: ExtensionKey::local("shutting"),
        });
        monitor.apply_event(ExtensionLifecycleEvent::Spawned {
            key: ExtensionKey::local("ok"),
        });
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: ExtensionKey::local("ok"),
            outcome: ExtensionOutcome::Ok,
        });

        monitor.mark_pending_as_timeout();

        assert_eq!(count_in_state(&monitor, ExtensionRuntimeState::TimedOut), 3);
        assert_eq!(
            count_in_state(&monitor, ExtensionRuntimeState::CompletedOk),
            1
        );
        assert_eq!(
            count_in_state(&monitor, ExtensionRuntimeState::Pending),
            0,
            "no entry may remain in Pending after reconciliation"
        );
        let timed_out_counter_sum: u64 = monitor
            .entries
            .iter()
            .map(|e| e.lifecycle_metrics.shutdown_timeout.get())
            .sum();
        assert_eq!(timed_out_counter_sum, 3);
    }

    #[tokio::test(flavor = "current_thread")]
    async fn out_of_order_spawned_after_completed_does_not_resurrect_state() {
        // Terminal states are sticky: a stale `Spawned` arriving after
        // `Completed` must not regress the entry.
        let (mut monitor, ctx) = fresh_monitor();
        let ent = ctx.register_extension_entity("ext".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext");
        monitor.register(&ctx, key.clone(), ent, None);

        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: key.clone() });
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: key.clone(),
            outcome: ExtensionOutcome::Ok,
        });
        assert_eq!(monitor.entries[0].state, ExtensionRuntimeState::CompletedOk);

        // Stray Spawned after terminal must not resurrect.
        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key });
        assert_eq!(
            monitor.entries[0].state,
            ExtensionRuntimeState::CompletedOk,
            "Spawned after Completed must not resurrect a terminal entry"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn duplicate_completed_does_not_double_count_or_flip_state() {
        // A second `Completed` for an already-terminal entry must be a
        // no-op (no double-count, no terminal flip).
        let (mut monitor, ctx) = fresh_monitor();
        let ent = ctx.register_extension_entity("ext".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext");
        monitor.register(&ctx, key.clone(), ent, None);

        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: key.clone() });
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: key.clone(),
            outcome: ExtensionOutcome::Ok,
        });
        assert_eq!(monitor.entries[0].state, ExtensionRuntimeState::CompletedOk);
        assert_eq!(monitor.entries[0].lifecycle_metrics.completed_ok.get(), 1);

        // Duplicate Completed Ok → ignored.
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: key.clone(),
            outcome: ExtensionOutcome::Ok,
        });
        // Late ShutdownTimeout for the same key → ignored, no flip.
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key,
            outcome: ExtensionOutcome::ShutdownTimeout,
        });

        assert_eq!(monitor.entries[0].state, ExtensionRuntimeState::CompletedOk);
        assert_eq!(monitor.entries[0].lifecycle_metrics.completed_ok.get(), 1);
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.shutdown_timeout.get(),
            0
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn mark_pending_as_timeout_is_inert_when_disabled() {
        let (ctx, _registry) = crate::testing::test_extension_ctx();
        let mut monitor = ExtensionMetricsMonitor::disabled(ctx.clone());
        // No entries are ever registered when disabled; just ensure it
        // is a safe no-op and does not panic.
        monitor.mark_pending_as_timeout();
        assert!(monitor.entries.is_empty());
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maybe_collect_telemetry_targets_only_spawned_with_sender() {
        // CollectTelemetry must only target `Spawned`-with-sender
        // entries; others are skipped.
        let (mut monitor, ctx) = fresh_monitor();
        let (tx_spawned, rx_spawned) = mpsc::Channel::new(8);
        let sender_spawned = ExtensionControlSender {
            name: "spawned".into(),
            sender: Sender::new_local_mpsc_sender(tx_spawned),
        };
        let (tx_done, rx_done) = mpsc::Channel::new(8);
        let sender_done = ExtensionControlSender {
            name: "done".into(),
            sender: Sender::new_local_mpsc_sender(tx_done),
        };

        let e_spawned = ctx.register_extension_entity("spawned".into(), ExtensionVariant::Local);
        monitor.register(
            &ctx,
            ExtensionKey::local("spawned"),
            e_spawned,
            Some(sender_spawned),
        );
        monitor.apply_event(ExtensionLifecycleEvent::Spawned {
            key: ExtensionKey::local("spawned"),
        });

        // Registered but never spawned — has a sender but wrong state.
        let e_pending = ctx.register_extension_entity("pending".into(), ExtensionVariant::Local);
        monitor.register(
            &ctx,
            ExtensionKey::local("pending"),
            e_pending,
            Some(make_sender("pending")),
        );

        // Spawned but no sender — wrong sender, right state.
        let e_no_send = ctx.register_extension_entity("no_send".into(), ExtensionVariant::Local);
        monitor.register(&ctx, ExtensionKey::local("no_send"), e_no_send, None);
        monitor.apply_event(ExtensionLifecycleEvent::Spawned {
            key: ExtensionKey::local("no_send"),
        });

        // Spawned then completed — already done.
        let e_done = ctx.register_extension_entity("done".into(), ExtensionVariant::Local);
        monitor.register(&ctx, ExtensionKey::local("done"), e_done, Some(sender_done));
        monitor.apply_event(ExtensionLifecycleEvent::Spawned {
            key: ExtensionKey::local("done"),
        });
        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: ExtensionKey::local("done"),
            outcome: ExtensionOutcome::Ok,
        });

        let (tx, _rx) = flume::bounded(1);
        let reporter = MetricsReporter::new(tx);
        monitor.maybe_collect_telemetry(Instant::now(), &reporter);

        assert!(
            rx_spawned.try_recv().is_ok(),
            "spawned-with-sender should receive"
        );
        assert!(
            rx_done.try_recv().is_err(),
            "completed entry should be skipped"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn maybe_collect_telemetry_respects_interval_gating() {
        // First call within a tick window dispatches; subsequent calls
        // before the interval elapses must skip dispatching so we don't
        // overwhelm extensions with collect requests.
        let (ctx, _registry) = crate::testing::test_extension_ctx();
        let mut monitor = ExtensionMetricsMonitor::new(
            ctx.clone(),
            Duration::from_millis(50),
            Duration::from_secs(60), // wide so the second call is gated
        );
        let (tx, rx) = mpsc::Channel::new(8);
        let sender = ExtensionControlSender {
            name: "ext1".into(),
            sender: Sender::new_local_mpsc_sender(tx),
        };
        let ent = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        monitor.register(&ctx, ExtensionKey::local("ext1"), ent, Some(sender));
        monitor.apply_event(ExtensionLifecycleEvent::Spawned {
            key: ExtensionKey::local("ext1"),
        });

        let (rep_tx, _rep_rx) = flume::bounded(1);
        let reporter = MetricsReporter::new(rep_tx);

        let t0 = Instant::now();
        monitor.maybe_collect_telemetry(t0, &reporter);
        assert!(rx.try_recv().is_ok(), "first call must dispatch");

        monitor.maybe_collect_telemetry(t0 + Duration::from_millis(1), &reporter);
        assert!(
            rx.try_recv().is_err(),
            "second call within interval must be gated"
        );

        monitor.maybe_collect_telemetry(t0 + Duration::from_secs(61), &reporter);
        assert!(
            rx.try_recv().is_ok(),
            "call past the interval must dispatch again"
        );
    }

    /// The `state` gauge's integer encoding is part of the public
    /// telemetry contract. Downstream dashboards and alerts rely on
    /// these specific values — a refactor must not silently re-order
    /// the discriminants.
    #[test]
    fn state_gauge_integer_encoding_is_stable() {
        assert_eq!(ExtensionRuntimeState::Pending as u64, 0);
        assert_eq!(ExtensionRuntimeState::Spawned as u64, 1);
        assert_eq!(ExtensionRuntimeState::ShutdownSent as u64, 2);
        assert_eq!(ExtensionRuntimeState::CompletedOk as u64, 3);
        assert_eq!(ExtensionRuntimeState::Failed as u64, 4);
        assert_eq!(ExtensionRuntimeState::TimedOut as u64, 5);
    }

    /// When the extension's control channel is full,
    /// `maybe_collect_telemetry` must log a warning and move on without
    /// flipping the entry's state or bumping any lifecycle counter.
    #[tokio::test(flavor = "current_thread")]
    async fn collect_telemetry_try_send_failure_does_not_skew_state_or_counters() {
        let (ctx, _registry) = crate::testing::test_extension_ctx();
        let mut monitor = ExtensionMetricsMonitor::new(
            ctx.clone(),
            Duration::from_millis(50),
            Duration::from_millis(50),
        );

        // Capacity-1 control channel that we pre-fill so the
        // CollectTelemetry try_send hits "channel full".
        let (tx, rx) = mpsc::Channel::new(1);
        let sender = ExtensionControlSender {
            name: "ext1".into(),
            sender: Sender::new_local_mpsc_sender(tx),
        };
        // Pre-fill so the next try_send fails.
        sender
            .sender
            .try_send(ExtensionControlMsg::Shutdown {
                deadline: Instant::now(),
                reason: "filler".into(),
            })
            .expect("pre-fill should succeed");

        let ent = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        monitor.register(&ctx, ExtensionKey::local("ext1"), ent, Some(sender));
        monitor.apply_event(ExtensionLifecycleEvent::Spawned {
            key: ExtensionKey::local("ext1"),
        });

        let spawned_before = monitor.entries[0].lifecycle_metrics.spawned.get();
        let completed_ok_before = monitor.entries[0].lifecycle_metrics.completed_ok.get();
        let completed_err_before = monitor.entries[0].lifecycle_metrics.completed_error.get();
        let shutdown_sent_before = monitor.entries[0].lifecycle_metrics.shutdown_sent.get();

        let (rep_tx, _rep_rx) = flume::bounded(1);
        let reporter = MetricsReporter::new(rep_tx);
        monitor.maybe_collect_telemetry(Instant::now(), &reporter);

        // No state flip, no counter movement — failure is observability-only.
        assert_eq!(
            monitor.entries[0].state,
            ExtensionRuntimeState::Spawned,
            "try_send failure must not flip the entry state"
        );
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.spawned.get(),
            spawned_before
        );
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.completed_ok.get(),
            completed_ok_before
        );
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.completed_error.get(),
            completed_err_before
        );
        assert_eq!(
            monitor.entries[0].lifecycle_metrics.shutdown_sent.get(),
            shutdown_sent_before
        );

        // Drain the pre-fill so the rx is not leaked.
        let _ = rx.try_recv();
    }

    /// The `state` gauge must remain asserted across many consecutive
    /// ticks for long-running extensions — no transient zeros between
    /// ticks even though gauges are reset semantics on the framework
    /// side.
    #[tokio::test(flavor = "current_thread")]
    async fn state_gauge_stays_asserted_across_multiple_ticks() {
        let (ctx, _registry) = crate::testing::test_extension_ctx();
        let mut monitor = ExtensionMetricsMonitor::new(
            ctx.clone(),
            Duration::from_millis(50),
            Duration::from_secs(60), // wide so CollectTelemetry stays gated
        );
        let ent = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext1");
        monitor.register(&ctx, key.clone(), ent, None);
        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key });

        let (rep_tx, _rep_rx) = flume::bounded(64);
        let mut reporter = MetricsReporter::new(rep_tx);

        let mut now = Instant::now();
        for cycle in 0..5 {
            // Simulate the framework clearing the gauge between ticks.
            monitor.entries[0].lifecycle_metrics.state.reset();

            monitor.tick(now, &mut reporter);

            assert_eq!(
                monitor.entries[0].lifecycle_metrics.state.get(),
                ExtensionRuntimeState::Spawned as u64,
                "cycle {cycle}: state gauge must be re-asserted on every tick"
            );
            assert_eq!(
                monitor.entries[0].state,
                ExtensionRuntimeState::Spawned,
                "cycle {cycle}: cached entry state must not drift"
            );
            now += Duration::from_millis(50);
        }
    }
}
