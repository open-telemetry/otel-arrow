// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension lifecycle holder for an extension-hosting runtime.
//!
//! Owns the spawned active+background extension tasks, the control
//! senders used to broadcast `Shutdown` to them, and the passive
//! extension wrappers that must outlive the run. Encapsulates the
//! "extensions start first, shut down last" invariant.

use crate::context::ExtensionContext;
use crate::control::{ExtensionControlMsg, ExtensionControlSender};
use crate::error::Error;
use crate::extension::ExtensionWrapper;
use crate::extension_monitor::{
    ExtensionKey, ExtensionLifecycleEvent, ExtensionMetricsMonitor, ExtensionOutcome,
};
use futures::stream::{FuturesUnordered, StreamExt};
use otap_df_telemetry::otel_warn;
use otap_df_telemetry::registry::EntityKey;
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::{self, JoinError, JoinHandle, LocalSet};
use tokio::time::Instant as TokioInstant;

/// Cleanup window granted to extensions after the data path has drained.
pub(crate) const EXTENSION_SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// Slack added past `EXTENSION_SHUTDOWN_GRACE` before the runtime
/// hard-stops draining, covering `JoinHandle` poll latency for an
/// extension that finishes right at the deadline.
pub(crate) const EXTENSION_SHUTDOWN_DRAIN_SLACK: Duration = Duration::from_millis(500);

/// Per-extension upper bound on how long `broadcast_shutdown` will
/// wait to enqueue `Shutdown`.
pub(crate) const EXTENSION_SHUTDOWN_SEND_TIMEOUT: Duration = Duration::from_millis(500);

/// Default reason recorded in the `Shutdown` broadcast when the caller
/// does not supply one.
const DEFAULT_SHUTDOWN_REASON: &str = "host data-path drained";

/// Event surfaced by [`ExtensionLifecycle::next_event`].
pub(crate) enum LifecycleEvent {
    Completion(Result<Result<(), Error>, JoinError>),
    MonitorTick(Instant),
}

/// Holds the spawned extension tasks, control senders, and passive
/// wrappers for the duration of an extension-hosting run.
pub(crate) struct ExtensionLifecycle {
    /// Active+background extension tasks. Each yields its
    /// `ExtensionKey` alongside its result; on `JoinError`
    /// (panic/cancel) the key is recovered from [`Self::task_id_to_key`].
    futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>>,
    /// Maps live spawned task ids to their `ExtensionKey` so a
    /// panic/cancel still resolves to a terminal monitor event.
    task_id_to_key: HashMap<task::Id, ExtensionKey>,
    /// Control senders for the extensions in [`Self::futures`], paired
    /// with their key for monitor `ShutdownSent` attribution.
    shutdown_senders: Vec<(ExtensionKey, ExtensionControlSender)>,
    /// Passive extensions held alive for the duration of the run so
    /// state behind their capability handles survives until drop.
    _passive: Vec<ExtensionWrapper>,
    /// One-shot latch: `true` after `Shutdown` has been broadcast.
    shutdown_broadcast_fired: bool,
    /// Deadline established when [`Self::broadcast_shutdown`] fires.
    shutdown_deadline: Option<Instant>,
    /// Per-scope lifecycle/telemetry monitor.
    monitor: ExtensionMetricsMonitor,
    /// Receives an `ExtensionKey` from each spawned task as its first
    /// action, so monitor `Spawned` reflects actual execution rather
    /// than just `LocalSet` registration.
    started_rx: mpsc::UnboundedReceiver<ExtensionKey>,
}

impl ExtensionLifecycle {
    /// Spawn all active+background extensions onto `local_tasks` and
    /// stash the passive ones. Each non-passive wrapper is registered
    /// with `monitor`.
    pub fn spawn(
        extensions: Vec<(ExtensionWrapper, EntityKey)>,
        local_tasks: &LocalSet,
        metrics_reporter: MetricsReporter,
        ext_ctx: &ExtensionContext,
        mut monitor: ExtensionMetricsMonitor,
    ) -> Self {
        let futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>> =
            FuturesUnordered::new();
        let mut task_id_to_key: HashMap<task::Id, ExtensionKey> = HashMap::new();
        let mut shutdown_senders: Vec<(ExtensionKey, ExtensionControlSender)> = Vec::new();
        let mut passive = Vec::new();
        let (started_tx, started_rx) = mpsc::unbounded_channel::<ExtensionKey>();

        for (ext_wrapper, entity_key) in extensions {
            if ext_wrapper.is_passive() {
                passive.push(ext_wrapper);
                continue;
            }
            let ext_id = ext_wrapper.name();
            let key = ExtensionKey::new(ext_id.clone(), ext_wrapper.variant());
            let control_sender = ext_wrapper.extension_control_sender();
            monitor.register(ext_ctx, key.clone(), entity_key, control_sender.clone());
            if let Some(sender) = control_sender {
                shutdown_senders.push((key.clone(), sender));
            }
            let ext_metrics_reporter = metrics_reporter.clone();
            let task_key = key.clone();
            let started_tx = started_tx.clone();
            let fut = async move {
                let _ = started_tx.send(task_key.clone());
                let res = match ext_wrapper.start(ext_metrics_reporter.clone()).await {
                    Ok(terminal_state) => {
                        crate::runtime_pipeline::report_terminal_metrics(
                            &ext_metrics_reporter,
                            terminal_state,
                        );
                        Ok(())
                    }
                    Err(e) => {
                        otel_warn!(
                            "extension.task.error",
                            extension = task_key.id.as_ref(),
                            error = format!("{e}"),
                        );
                        Err(e)
                    }
                };
                (task_key, res)
            };
            let handle = local_tasks.spawn_local(fut);
            let _ = task_id_to_key.insert(handle.id(), key);
            futures.push(handle);
        }
        // Drop the seed sender so `started_rx.recv()` returns `None`
        // once every per-task clone has been dropped.
        drop(started_tx);

        Self {
            futures,
            task_id_to_key,
            shutdown_senders,
            _passive: passive,
            shutdown_broadcast_fired: false,
            shutdown_deadline: None,
            monitor,
            started_rx,
        }
    }

    /// Returns `true` if there are no remaining active+background
    /// extension tasks to await.
    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Awaits either the next active+background extension completion
    /// or the next monitor tick. `Spawned` signals are absorbed
    /// internally and never surface to the host.
    pub async fn next_event(&mut self) -> LifecycleEvent {
        loop {
            let Self {
                futures,
                task_id_to_key,
                monitor,
                started_rx,
                ..
            } = self;
            if futures.is_empty() {
                tokio::select! {
                    biased;
                    Some(key) = started_rx.recv() => {
                        monitor.apply_event(ExtensionLifecycleEvent::Spawned { key });
                        continue;
                    }
                    now = monitor.next_tick() => return LifecycleEvent::MonitorTick(now),
                }
            }
            tokio::select! {
                biased;
                Some(key) = started_rx.recv() => {
                    monitor.apply_event(ExtensionLifecycleEvent::Spawned { key });
                    continue;
                }
                Some(joined) = futures.next() => {
                    return LifecycleEvent::Completion(
                        Self::route_joined(monitor, task_id_to_key, joined)
                    );
                }
                now = monitor.next_tick() => return LifecycleEvent::MonitorTick(now),
            }
        }
    }

    /// Records the completion outcome in the monitor and reshapes the
    /// join result. Panic/cancel paths recover the key from
    /// `task_id_to_key`.
    fn route_joined(
        monitor: &mut ExtensionMetricsMonitor,
        task_id_to_key: &mut HashMap<task::Id, ExtensionKey>,
        joined: Result<(ExtensionKey, Result<(), Error>), JoinError>,
    ) -> Result<Result<(), Error>, JoinError> {
        match joined {
            Ok((key, res)) => {
                task_id_to_key.retain(|_, k| k != &key);
                let outcome = match &res {
                    Ok(()) => ExtensionOutcome::Ok,
                    Err(e) => ExtensionOutcome::Err(e.to_string()),
                };
                monitor.apply_event(ExtensionLifecycleEvent::Completed { key, outcome });
                Ok(res)
            }
            Err(e) => {
                let task_id = e.id();
                if let Some(key) = task_id_to_key.remove(&task_id) {
                    let outcome = if e.is_cancelled() {
                        ExtensionOutcome::JoinCancelled
                    } else {
                        ExtensionOutcome::JoinPanic
                    };
                    monitor.apply_event(ExtensionLifecycleEvent::Completed { key, outcome });
                } else {
                    otel_warn!(
                        "extension.task.join_error.unknown_task",
                        is_canceled = e.is_cancelled(),
                        is_panic = e.is_panic(),
                        task_id = format!("{task_id}"),
                    );
                }
                Err(e)
            }
        }
    }

    /// Drives one monitor tick: fans out `CollectTelemetry` to spawned
    /// extensions on cadence and reports lifecycle + aggregate metrics.
    pub fn monitor_tick(&mut self, now: Instant, reporter: &mut MetricsReporter) {
        self.monitor.tick(now, reporter);
    }

    /// Broadcasts `Shutdown` to all active+background extensions.
    /// Idempotent. Sends fan out concurrently, each bounded by
    /// [`EXTENSION_SHUTDOWN_SEND_TIMEOUT`]; `reason` is propagated
    /// verbatim (defaulting to [`DEFAULT_SHUTDOWN_REASON`]).
    pub async fn broadcast_shutdown(&mut self, reason: Option<&str>) {
        if self.shutdown_broadcast_fired || self.shutdown_senders.is_empty() {
            return;
        }
        self.shutdown_broadcast_fired = true;

        let deadline = Instant::now() + EXTENSION_SHUTDOWN_GRACE;
        self.shutdown_deadline = Some(deadline);

        let reason = reason.unwrap_or(DEFAULT_SHUTDOWN_REASON).to_string();
        let sends = self.shutdown_senders.iter().map(|(key, sender)| {
            let msg = ExtensionControlMsg::Shutdown {
                deadline,
                reason: reason.clone(),
            };
            async move {
                match tokio::time::timeout(EXTENSION_SHUTDOWN_SEND_TIMEOUT, sender.sender.send(msg))
                    .await
                {
                    Ok(Ok(())) => Some(key.clone()),
                    Ok(Err(e)) => {
                        otel_warn!(
                            "extension.shutdown.send_failed",
                            extension = sender.name.as_ref(),
                            error = format!("{e}"),
                        );
                        None
                    }
                    Err(_elapsed) => {
                        otel_warn!(
                            "extension.shutdown.send_timeout",
                            extension = sender.name.as_ref(),
                            timeout_ms = EXTENSION_SHUTDOWN_SEND_TIMEOUT.as_millis() as u64,
                        );
                        None
                    }
                }
            }
        });
        let delivered: Vec<Option<ExtensionKey>> = futures::future::join_all(sends).await;
        for key in delivered.into_iter().flatten() {
            self.monitor
                .apply_event(ExtensionLifecycleEvent::ShutdownSent { key });
        }
    }

    /// Drains remaining active+background extension tasks, bounded by
    /// the shutdown deadline so a misbehaving extension can't hang the
    /// host. No-op when there are no remaining futures.
    pub async fn drain_until_deadline(&mut self) {
        if self.futures.is_empty() {
            return;
        }
        let deadline = self
            .shutdown_deadline
            .get_or_insert_with(|| Instant::now() + EXTENSION_SHUTDOWN_GRACE);
        let drain_deadline = TokioInstant::from_std(*deadline + EXTENSION_SHUTDOWN_DRAIN_SLACK);

        let futures = &mut self.futures;
        let task_id_to_key = &mut self.task_id_to_key;
        let monitor = &mut self.monitor;
        let drain = async {
            while let Some(result) = futures.next().await {
                match result {
                    Ok((key, Ok(()))) => {
                        task_id_to_key.retain(|_, k| k != &key);
                        monitor.apply_event(ExtensionLifecycleEvent::Completed {
                            key,
                            outcome: ExtensionOutcome::Ok,
                        });
                    }
                    Ok((key, Err(e))) => {
                        task_id_to_key.retain(|_, k| k != &key);
                        otel_warn!("extension.shutdown.task.error", error = format!("{e}"));
                        monitor.apply_event(ExtensionLifecycleEvent::Completed {
                            key,
                            outcome: ExtensionOutcome::Err(e.to_string()),
                        });
                    }
                    Err(e) => {
                        let task_id = e.id();
                        otel_warn!(
                            "extension.shutdown.task.join_error",
                            is_canceled = e.is_cancelled(),
                            is_panic = e.is_panic(),
                            error = e.to_string()
                        );
                        if let Some(key) = task_id_to_key.remove(&task_id) {
                            let outcome = if e.is_cancelled() {
                                ExtensionOutcome::JoinCancelled
                            } else {
                                ExtensionOutcome::JoinPanic
                            };
                            monitor
                                .apply_event(ExtensionLifecycleEvent::Completed { key, outcome });
                        }
                    }
                }
            }
        };

        if tokio::time::timeout_at(drain_deadline, drain)
            .await
            .is_err()
        {
            otel_warn!(
                "extension.shutdown.timeout",
                grace_secs = EXTENSION_SHUTDOWN_GRACE.as_secs(),
                remaining = self.futures.len()
            );
            self.monitor.mark_pending_as_timeout();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::wrapper::ExtensionVariant;

    /// `drain_until_deadline` must return after the bounded deadline
    /// even if an extension never honours `Shutdown`.
    #[test]
    fn drain_until_deadline_is_bounded_for_stuck_extension() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();
            let key = ExtensionKey::new("stuck".into(), ExtensionVariant::Local);
            let futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>> =
                FuturesUnordered::new();
            let handle = local_tasks.spawn_local({
                let key = key.clone();
                async move {
                    std::future::pending::<()>().await;
                    (key, Ok(()))
                }
            });
            let mut task_id_to_key = HashMap::new();
            let _ = task_id_to_key.insert(handle.id(), key);
            futures.push(handle);

            let injected_deadline = Instant::now() + Duration::from_millis(100);
            let (_started_tx, started_rx) = mpsc::unbounded_channel();
            let mut lifecycle = ExtensionLifecycle {
                futures,
                task_id_to_key,
                shutdown_senders: Vec::new(),
                _passive: Vec::new(),
                shutdown_broadcast_fired: true,
                shutdown_deadline: Some(injected_deadline),
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx,
            };

            let start = Instant::now();
            lifecycle.drain_until_deadline().await;
            let elapsed = start.elapsed();

            let upper_bound = Duration::from_millis(100)
                + EXTENSION_SHUTDOWN_DRAIN_SLACK
                + Duration::from_secs(1);
            assert!(
                elapsed < upper_bound,
                "drain_until_deadline did not honor the deadline: elapsed={:?}, upper_bound={:?}",
                elapsed,
                upper_bound,
            );
            assert!(
                !lifecycle.futures.is_empty(),
                "stuck extension should still be present after the bounded drain timed out",
            );
        }));
    }

    /// A panicking extension task must surface as a terminal `Failed`
    /// state and bump the `completed_panic` counter.
    #[test]
    fn panicking_extension_task_reports_failed_terminal_state() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();
            let key = ExtensionKey::new("boom".into(), ExtensionVariant::Local);
            let entity_key = ext_ctx
                .register_extension_entity("boom".into(), ExtensionVariant::Local);

            let mut monitor = ExtensionMetricsMonitor::new(
                ext_ctx.clone(),
                Duration::from_millis(50),
                Duration::from_millis(50),
            );
            monitor.register(&ext_ctx, key.clone(), entity_key, None);
            monitor.apply_event(ExtensionLifecycleEvent::Spawned { key: key.clone() });

            let futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>> =
                FuturesUnordered::new();
            let handle = local_tasks.spawn_local({
                let task_key = key.clone();
                async move {
                    task::yield_now().await;
                    panic!("simulated extension panic");
                    #[allow(unreachable_code)]
                    {
                        let _: ExtensionKey = task_key;
                        (
                            ExtensionKey::new("never".into(), ExtensionVariant::Local),
                            Ok(()),
                        )
                    }
                }
            });
            let mut task_id_to_key = HashMap::new();
            let _ = task_id_to_key.insert(handle.id(), key.clone());
            futures.push(handle);

            let (_started_tx, started_rx) = mpsc::unbounded_channel();
            let mut lifecycle = ExtensionLifecycle {
                futures,
                task_id_to_key,
                shutdown_senders: Vec::new(),
                _passive: Vec::new(),
                shutdown_broadcast_fired: false,
                shutdown_deadline: None,
                monitor,
                started_rx,
            };

            // Silence the default panic hook so the join-induced backtrace
            // doesn't pollute test output.
            let prev_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));

            let result = lifecycle.next_event().await;

            std::panic::set_hook(prev_hook);

            match result {
                LifecycleEvent::Completion(Err(e)) => {
                    assert!(e.is_panic(), "expected panic-style JoinError, got {e:?}");
                }
                LifecycleEvent::Completion(Ok(inner)) => panic!(
                    "expected Completion(Err(JoinError)) for a panicking task, got Completion(Ok({inner:?}))"
                ),
                LifecycleEvent::MonitorTick(_) => panic!(
                    "expected the panicking task to surface as a Completion before any monitor tick"
                ),
            }

            assert_eq!(
                lifecycle.monitor.state_for(&key),
                Some(crate::extension_monitor::ExtensionRuntimeState::Failed),
                "panicking extension must surface as Failed in the monitor",
            );
        assert_eq!(
            lifecycle.monitor.completed_panic_count(&key),
            Some(1),
            "completed_panic counter must increment exactly once",
        );
        }));
    }

    /// `broadcast_shutdown` is a one-shot latch: a second call must not
    /// re-enqueue another `Shutdown` for any extension.
    #[test]
    fn broadcast_shutdown_is_idempotent_at_lifecycle_level() {
        use crate::control::ExtensionControlSender;
        use crate::message::Sender;
        use otap_df_channel::mpsc;

        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            let (tx_a, rx_a) = mpsc::Channel::new(8);
            let (tx_b, rx_b) = mpsc::Channel::new(8);
            let key_a = ExtensionKey::new("a".into(), ExtensionVariant::Local);
            let key_b = ExtensionKey::new("b".into(), ExtensionVariant::Local);

            let mut lifecycle = ExtensionLifecycle {
                futures: FuturesUnordered::new(),
                task_id_to_key: HashMap::new(),
                shutdown_senders: vec![
                    (
                        key_a.clone(),
                        ExtensionControlSender {
                            name: "a".into(),
                            sender: Sender::new_local_mpsc_sender(tx_a),
                        },
                    ),
                    (
                        key_b.clone(),
                        ExtensionControlSender {
                            name: "b".into(),
                            sender: Sender::new_local_mpsc_sender(tx_b),
                        },
                    ),
                ],
                _passive: Vec::new(),
                shutdown_broadcast_fired: false,
                shutdown_deadline: None,
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx: tokio::sync::mpsc::unbounded_channel().1,
            };

            lifecycle.broadcast_shutdown(Some("first")).await;
            lifecycle.broadcast_shutdown(Some("second")).await;

            let count = |rx: &mpsc::Receiver<ExtensionControlMsg>| {
                let mut n = 0;
                while rx.try_recv().is_ok() {
                    n += 1;
                }
                n
            };
            assert_eq!(
                count(&rx_a),
                1,
                "extension a must receive exactly one Shutdown"
            );
            assert_eq!(
                count(&rx_b),
                1,
                "extension b must receive exactly one Shutdown"
            );
            assert!(
                lifecycle.shutdown_broadcast_fired,
                "shutdown_broadcast_fired latch must remain set"
            );
        }));
    }

    /// Passive extensions never spawn a task and must not consume a
    /// monitor entry: monitor only tracks the live-task population.
    #[test]
    fn passive_extensions_are_not_registered_in_monitor() {
        use crate::extension::ExtensionWrapper;

        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            // Build a passive shared wrapper.
            let passive_cfg = crate::config::ExtensionConfig::new("passive_ext");
            let user = std::sync::Arc::new(otap_df_config::extension::ExtensionUserConfig::new(
                "urn:otap:extension:test".into(),
                serde_json::Value::Null,
            ));
            let mut passive_bundle =
                ExtensionWrapper::builder("passive_ext".into(), user.clone(), &passive_cfg)
                    .passive()
                    .cloned()
                    .shared("state".to_string())
                    .build()
                    .unwrap();
            let passive_w = passive_bundle.take_shared().unwrap();
            let passive_entity =
                ext_ctx.register_extension_entity("passive_ext".into(), ExtensionVariant::Shared);

            // Build an active local wrapper.
            #[derive(Clone)]
            struct ActiveExt;
            #[async_trait::async_trait(?Send)]
            impl crate::local::extension::Extension for ActiveExt {
                async fn start(
                    self: std::rc::Rc<Self>,
                    mut ctrl: crate::local::extension::ControlChannel,
                    _eh: crate::extension::wrapper::EffectHandler,
                ) -> Result<crate::terminal_state::TerminalState, Error> {
                    while let Ok(msg) = ctrl.recv().await {
                        if matches!(msg, ExtensionControlMsg::Shutdown { .. }) {
                            break;
                        }
                    }
                    Ok(Default::default())
                }
            }
            let active_cfg = crate::config::ExtensionConfig::new("active_ext");
            let mut active_bundle =
                ExtensionWrapper::builder("active_ext".into(), user, &active_cfg)
                    .active()
                    .local(std::rc::Rc::new(ActiveExt))
                    .build()
                    .unwrap();
            let active_w = active_bundle.take_local().unwrap();
            let active_entity =
                ext_ctx.register_extension_entity("active_ext".into(), ExtensionVariant::Local);

            let monitor = ExtensionMetricsMonitor::new(
                ext_ctx.clone(),
                Duration::from_millis(50),
                Duration::from_millis(50),
            );

            let (tx, _rx) = flume::bounded(1);
            let reporter = MetricsReporter::new(tx);
            let lifecycle = ExtensionLifecycle::spawn(
                vec![(passive_w, passive_entity), (active_w, active_entity)],
                &local_tasks,
                reporter,
                &ext_ctx,
                monitor,
            );

            let passive_key = ExtensionKey::new("passive_ext".into(), ExtensionVariant::Shared);
            let active_key = ExtensionKey::new("active_ext".into(), ExtensionVariant::Local);
            assert!(
                lifecycle.monitor.state_for(&passive_key).is_none(),
                "passive extension must not appear in the monitor"
            );
            assert!(
                lifecycle.monitor.state_for(&active_key).is_some(),
                "active extension must appear in the monitor"
            );

            drop(lifecycle);
        }));
    }

    /// Two scopes (today: pipelines) each own a distinct
    /// `ExtensionLifecycle`. A broadcast in one must not reach the
    /// other's extensions — there is no shared shutdown bus.
    #[test]
    fn broadcast_shutdown_in_one_scope_does_not_reach_another_scope() {
        use crate::control::ExtensionControlSender;
        use crate::message::Sender;
        use otap_df_channel::mpsc;

        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ctx_a, _reg_a) = crate::testing::test_extension_ctx();
            let (ctx_b, _reg_b) = crate::testing::test_extension_ctx();

            let (tx_a, rx_a) = mpsc::Channel::new(8);
            let (tx_b, rx_b) = mpsc::Channel::new(8);

            let mut life_a = ExtensionLifecycle {
                futures: FuturesUnordered::new(),
                task_id_to_key: HashMap::new(),
                shutdown_senders: vec![(
                    ExtensionKey::new("a".into(), ExtensionVariant::Local),
                    ExtensionControlSender {
                        name: "a".into(),
                        sender: Sender::new_local_mpsc_sender(tx_a),
                    },
                )],
                _passive: Vec::new(),
                shutdown_broadcast_fired: false,
                shutdown_deadline: None,
                monitor: ExtensionMetricsMonitor::disabled(ctx_a),
                started_rx: tokio::sync::mpsc::unbounded_channel().1,
            };
            let life_b = ExtensionLifecycle {
                futures: FuturesUnordered::new(),
                task_id_to_key: HashMap::new(),
                shutdown_senders: vec![(
                    ExtensionKey::new("b".into(), ExtensionVariant::Local),
                    ExtensionControlSender {
                        name: "b".into(),
                        sender: Sender::new_local_mpsc_sender(tx_b),
                    },
                )],
                _passive: Vec::new(),
                shutdown_broadcast_fired: false,
                shutdown_deadline: None,
                monitor: ExtensionMetricsMonitor::disabled(ctx_b),
                started_rx: tokio::sync::mpsc::unbounded_channel().1,
            };

            life_a.broadcast_shutdown(Some("a-only")).await;

            assert!(
                rx_a.try_recv().is_ok(),
                "scope A's extension must receive its own shutdown"
            );
            assert!(
                rx_b.try_recv().is_err(),
                "scope B's extension must NOT receive scope A's shutdown"
            );
            assert!(
                !life_b.shutdown_broadcast_fired,
                "scope B's latch must remain unset"
            );
        }));
    }

    /// Two scopes each own a distinct `ExtensionMetricsMonitor` with
    /// independent tick cadences and `last_collect_telemetry` gating.
    /// A `CollectTelemetry` fanout in one scope's monitor must not
    /// reach the other scope's extensions.
    #[test]
    fn collect_telemetry_fanout_is_scoped_to_owning_monitor() {
        use crate::control::ExtensionControlSender;
        use crate::message::Sender;
        use otap_df_channel::mpsc;

        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ctx_a, _reg_a) = crate::testing::test_extension_ctx();
            let (ctx_b, _reg_b) = crate::testing::test_extension_ctx();

            let mut monitor_a = ExtensionMetricsMonitor::new(
                ctx_a.clone(),
                Duration::from_millis(50),
                Duration::from_millis(50),
            );
            let mut monitor_b = ExtensionMetricsMonitor::new(
                ctx_b.clone(),
                Duration::from_millis(50),
                Duration::from_millis(50),
            );

            let (tx_a, rx_a) = mpsc::Channel::new(8);
            let (tx_b, rx_b) = mpsc::Channel::new(8);
            let sender_a = ExtensionControlSender {
                name: "a".into(),
                sender: Sender::new_local_mpsc_sender(tx_a),
            };
            let sender_b = ExtensionControlSender {
                name: "b".into(),
                sender: Sender::new_local_mpsc_sender(tx_b),
            };

            let key_a = ExtensionKey::new("a".into(), ExtensionVariant::Local);
            let key_b = ExtensionKey::new("b".into(), ExtensionVariant::Local);
            let ent_a = ctx_a.register_extension_entity("a".into(), ExtensionVariant::Local);
            let ent_b = ctx_b.register_extension_entity("b".into(), ExtensionVariant::Local);
            monitor_a.register(&ctx_a, key_a.clone(), ent_a, Some(sender_a));
            monitor_b.register(&ctx_b, key_b.clone(), ent_b, Some(sender_b));
            monitor_a.apply_event(ExtensionLifecycleEvent::Spawned { key: key_a });
            monitor_b.apply_event(ExtensionLifecycleEvent::Spawned { key: key_b });

            let (rep_tx, _rep_rx) = flume::bounded(8);
            let mut reporter = MetricsReporter::new(rep_tx);

            // Only scope A ticks. Scope B's monitor stays idle.
            monitor_a.tick(Instant::now(), &mut reporter);

            assert!(
                rx_a.try_recv().is_ok(),
                "scope A's extension must receive its scope's CollectTelemetry"
            );
            assert!(
                rx_b.try_recv().is_err(),
                "scope B's extension must NOT receive scope A's CollectTelemetry"
            );
        }));
    }
}
