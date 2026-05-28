// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-scope extension lifecycle and telemetry monitor.
//!
//! Mirrors `PipelineMetricsMonitor` in shape but is scope-agnostic: one
//! instance per extension-hosting scope (today: pipeline; later: engine,
//! group, node). Owns its own tick cadence; the host plugs `next_tick()`
//! into its `select!`. Decoupled from `ExtensionLifecycle` via
//! [`ExtensionLifecycleEvent`].

use crate::context::ExtensionContext;
use crate::control::{ExtensionControlMsg, ExtensionControlSender};
use crate::extension::wrapper::ExtensionVariant;
use otap_df_config::ExtensionId;
use otap_df_telemetry::instrument::{Counter, Gauge, UpDownCounter};
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
    Err(String),
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

#[metric_set(name = "extension.host")]
#[derive(Debug, Default, Clone)]
pub struct ExtensionAggregateMetrics {
    #[metric(unit = "{extension}")]
    pub total: Gauge<u64>,
    
    #[metric(unit = "{extension}")]
    pub running: UpDownCounter<u64>,
    
    #[metric(unit = "{extension}")]
    pub shutting_down: UpDownCounter<u64>,
    
    #[metric(unit = "{completion}")]
    pub failed: Counter<u64>,
    
    #[metric(unit = "{timeout}")]
    pub timed_out: Counter<u64>,
}

struct ExtensionMonitorEntry {
    key: ExtensionKey,
    entity_key: EntityKey,
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
    aggregate: Option<MetricSet<ExtensionAggregateMetrics>>,
    entries: Vec<ExtensionMonitorEntry>,
    interval: Option<Interval>,
    collect_telemetry_interval: Duration,
    last_collect_telemetry: Option<Instant>,
}

impl ExtensionMetricsMonitor {
    pub(crate) fn new(
        host_ctx: ExtensionContext,
        host_entity_key: EntityKey,
        tick_interval: Duration,
        collect_telemetry_interval: Duration,
    ) -> Self {
        let registry = host_ctx.metrics_registry();
        let aggregate = host_ctx
            .register_metric_set_for_entity::<ExtensionAggregateMetrics>(host_entity_key);

        let start = tokio::time::Instant::now() + tick_interval;
        let mut interval = interval_at(start, tick_interval);
        interval.set_missed_tick_behavior(MissedTickBehavior::Skip);

        Self {
            registry,
            aggregate: Some(aggregate),
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
            aggregate: None,
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
            entity_key,
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
        if let Some(agg) = self.aggregate.as_mut() {
            agg.total.set(self.entries.len() as u64);
        }
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
        let mut found = false;
        if let Some(entry) = self.entry_mut(key) {
            entry.state = ExtensionRuntimeState::Spawned;
            entry.lifecycle_metrics.spawned.add(1);
            entry
                .lifecycle_metrics
                .state
                .set(ExtensionRuntimeState::Spawned as u64);
            found = true;
        }
        if found {
            if let Some(agg) = self.aggregate.as_mut() {
                agg.running.add(1);
            }
        }
    }

    fn on_shutdown_sent(&mut self, key: &ExtensionKey) {
        let mut transitioned = false;
        if let Some(entry) = self.entry_mut(key) {
            if matches!(entry.state, ExtensionRuntimeState::Spawned) {
                entry.state = ExtensionRuntimeState::ShutdownSent;
                entry.lifecycle_metrics.shutdown_sent.add(1);
                entry
                    .lifecycle_metrics
                    .state
                    .set(ExtensionRuntimeState::ShutdownSent as u64);
                transitioned = true;
            }
        }
        if transitioned {
            if let Some(agg) = self.aggregate.as_mut() {
                agg.running.sub(1);
                agg.shutting_down.add(1);
            }
        }
    }

    fn on_completed(&mut self, key: &ExtensionKey, outcome: ExtensionOutcome) {
        let Some(entry) = self.entry_mut(key) else {
            return;
        };
        let prev_state = entry.state;
        let (new_state, agg_failed, agg_timeout) = match outcome {
            ExtensionOutcome::Ok => {
                entry.lifecycle_metrics.completed_ok.add(1);
                (ExtensionRuntimeState::CompletedOk, 0u64, 0u64)
            }
            ExtensionOutcome::Err(_) => {
                entry.lifecycle_metrics.completed_error.add(1);
                (ExtensionRuntimeState::Failed, 1, 0)
            }
            ExtensionOutcome::JoinPanic => {
                entry.lifecycle_metrics.completed_panic.add(1);
                (ExtensionRuntimeState::Failed, 1, 0)
            }
            ExtensionOutcome::JoinCancelled => {
                entry.lifecycle_metrics.completed_cancelled.add(1);
                (ExtensionRuntimeState::Failed, 1, 0)
            }
            ExtensionOutcome::ShutdownTimeout => {
                entry.lifecycle_metrics.shutdown_timeout.add(1);
                (ExtensionRuntimeState::TimedOut, 0, 1)
            }
        };
        entry.state = new_state;
        entry.lifecycle_metrics.state.set(new_state as u64);
        let _ = entry.entity_key;

        if let Some(agg) = self.aggregate.as_mut() {
            match prev_state {
                ExtensionRuntimeState::Spawned => agg.running.sub(1),
                ExtensionRuntimeState::ShutdownSent => agg.shutting_down.sub(1),
                _ => {}
            }
            if agg_failed > 0 {
                agg.failed.add(agg_failed);
            }
            if agg_timeout > 0 {
                agg.timed_out.add(agg_timeout);
            }
        }
    }

    /// Awaits the next monitor tick. `disabled()` returns a never-ready
    /// future so the `select!` arm never fires.
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
        self.report(reporter);
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
        if let Some(agg) = self.aggregate.as_mut() {
            if let Err(err) = reporter.report(agg) {
                otel_warn!(
                    "extension.host.metrics.reporting.fail",
                    error = err.to_string(),
                );
            }
        }
    }
}

impl Drop for ExtensionMetricsMonitor {
    fn drop(&mut self) {
        if let Some(agg) = self.aggregate.as_ref() {
            let _ = self.registry.unregister_metric_set(agg.metric_set_key());
        }
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
        let host_key = ctx.register_extension_entity("host".into(), ExtensionVariant::Local);
        let monitor = ExtensionMetricsMonitor::new(
            ctx.clone(),
            host_key,
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

    #[tokio::test(flavor = "current_thread")]
    async
    fn register_creates_per_extension_metric_set() {
        let (mut monitor, ctx) = fresh_monitor();
        let ext_key = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        monitor.register(&ctx, ExtensionKey::local("ext1"), ext_key, None);
        assert_eq!(monitor.entries.len(), 1);
        assert!(matches!(
            monitor.entries[0].state,
            ExtensionRuntimeState::Pending
        ));
        assert_eq!(monitor.aggregate.as_ref().unwrap().total.get(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async
    fn lifecycle_transitions_update_per_entry_and_aggregate() {
        let (mut monitor, ctx) = fresh_monitor();
        let ext_key = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext1");
        monitor.register(&ctx, key.clone(), ext_key, Some(make_sender("ext1")));

        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: key.clone() });
        let agg = monitor.aggregate.as_ref().unwrap();
        assert_eq!(agg.running.get(), 1);
        assert_eq!(monitor.entries[0].lifecycle_metrics.spawned.get(), 1);

        monitor.apply_event(ExtensionLifecycleEvent::ShutdownSent { key: key.clone() });
        let agg = monitor.aggregate.as_ref().unwrap();
        assert_eq!(agg.running.get(), 0);
        assert_eq!(agg.shutting_down.get(), 1);

        monitor.apply_event(ExtensionLifecycleEvent::Completed {
            key: key.clone(),
            outcome: ExtensionOutcome::Ok,
        });
        let agg = monitor.aggregate.as_ref().unwrap();
        assert_eq!(agg.shutting_down.get(), 0);
        assert_eq!(monitor.entries[0].lifecycle_metrics.completed_ok.get(), 1);
        assert!(matches!(
            monitor.entries[0].state,
            ExtensionRuntimeState::CompletedOk
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async
    fn local_and_shared_same_id_tracked_independently() {
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
        assert_eq!(monitor.aggregate.as_ref().unwrap().running.get(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async
    fn shutdown_sent_is_idempotent() {
        let (mut monitor, ctx) = fresh_monitor();
        let ext_key = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext1");
        monitor.register(&ctx, key.clone(), ext_key, None);
        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: key.clone() });
        monitor.apply_event(ExtensionLifecycleEvent::ShutdownSent { key: key.clone() });
        monitor.apply_event(ExtensionLifecycleEvent::ShutdownSent { key: key.clone() });
        let agg = monitor.aggregate.as_ref().unwrap();
        assert_eq!(agg.shutting_down.get(), 1);
        assert_eq!(monitor.entries[0].lifecycle_metrics.shutdown_sent.get(), 1);
    }

    #[tokio::test(flavor = "current_thread")]
    async
    fn failure_outcomes_count_aggregate_failed() {
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
        let agg = monitor.aggregate.as_ref().unwrap();
        assert_eq!(agg.failed.get(), 3);
        assert_eq!(agg.timed_out.get(), 0);
    }

    #[tokio::test(flavor = "current_thread")]
    async
    fn shutdown_timeout_counts_aggregate_timed_out() {
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
        let agg = monitor.aggregate.as_ref().unwrap();
        assert_eq!(agg.timed_out.get(), 1);
        assert_eq!(agg.failed.get(), 0);
        assert!(matches!(
            monitor.entries[0].state,
            ExtensionRuntimeState::TimedOut
        ));
    }

    #[tokio::test(flavor = "current_thread")]
    async
    fn disabled_monitor_is_inert() {
        let (ctx, _registry) = crate::testing::test_extension_ctx();
        let mut monitor = ExtensionMetricsMonitor::disabled(ctx.clone());
        let ent = ctx.register_extension_entity("ext1".into(), ExtensionVariant::Local);
        let key = ExtensionKey::local("ext1");
        monitor.register(&ctx, key.clone(), ent, None);
        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key });
        assert!(monitor.entries.is_empty());
        assert!(monitor.aggregate.is_none());
    }

    #[tokio::test(flavor = "current_thread")]
    async
    fn dropping_monitor_unregisters_metric_sets() {
        let (ctx, registry) = crate::testing::test_extension_ctx();
        let host = ctx.register_extension_entity("host".into(), ExtensionVariant::Local);
        let before = registry.metric_set_count();
        {
            let mut monitor = ExtensionMetricsMonitor::new(
                ctx.clone(),
                host,
                Duration::from_millis(50),
                Duration::from_millis(50),
            );
            for name in ["a", "b"] {
                let ent = ctx.register_extension_entity(name.into(), ExtensionVariant::Local);
                monitor.register(&ctx, ExtensionKey::local(name), ent, None);
            }
            assert_eq!(registry.metric_set_count(), before + 3);
        }
        assert_eq!(registry.metric_set_count(), before);
    }
}
