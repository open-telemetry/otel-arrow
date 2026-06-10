// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension lifecycle holder for an extension-hosting runtime.
//!
//! Owns spawned active/background extension tasks, shutdown senders,
//! and passive extension wrappers. Encapsulates "extensions start
//! first, shut down last" on the normal drain path.
//!
//! TODO: on internal node/extension failure the orderly drain
//! protocol is skipped; remaining data-path tasks are force-cancelled
//! by `LocalSet` drop, so the stop-last guarantee currently applies
//! only to the normal drain path. Fix in a follow-up PR by making
//! pipeline shutdown orchestrated.

use crate::context::ExtensionContext;
use crate::control::{ExtensionShutdownChannel, ShutdownPayload};
use crate::error::Error;
use crate::extension::ExtensionWrapper;
use crate::extension_monitor::{
    ExtensionKey, ExtensionLifecycleEvent, ExtensionMetricsMonitor, ExtensionOutcome,
};
use futures::FutureExt;
use futures::stream::{FuturesUnordered, StreamExt};
use otap_df_telemetry::otel_warn;
use otap_df_telemetry::registry::EntityKey;
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::{HashMap, HashSet};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::{self, JoinError, JoinHandle, LocalSet};
use tokio::time::Instant as TokioInstant;

/// Cleanup window granted to extensions after the data path has drained.
pub(crate) const EXTENSION_SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// Slack past `EXTENSION_SHUTDOWN_GRACE` before the runtime hard-stops draining.
pub(crate) const EXTENSION_SHUTDOWN_DRAIN_SLACK: Duration = Duration::from_millis(500);

const DEFAULT_SHUTDOWN_REASON: &str = "host data-path drained";

pub(crate) enum LifecycleEvent {
    Completion(Result<Result<(), Error>, JoinError>),
    MonitorTick(Instant),
}

#[derive(Debug, Clone, Copy)]
pub(crate) enum LifecyclePhase {
    Running,
    ShuttingDown { deadline: Instant },
}

pub(crate) struct ExtensionLifecycle {
    futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>>,
    task_id_to_key: HashMap<task::Id, ExtensionKey>,
    shutdown_channels: Vec<(ExtensionKey, ExtensionShutdownChannel)>,
    _passive: Vec<ExtensionWrapper>,
    phase: LifecyclePhase,
    monitor: ExtensionMetricsMonitor,
    started_rx: mpsc::UnboundedReceiver<ExtensionKey>,
    // Keys we have not yet seen a spawn signal for. Each event (start signal
    // or task completion) calls `remove`, which is idempotent — so a task
    // that signals and then completes can never under-count its slot.
    pending_starts: HashSet<ExtensionKey>,
}

impl ExtensionLifecycle {
    /// Spawn active+background extensions, stash passive ones, register each
    /// non-passive wrapper with `monitor`.
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
        let mut shutdown_channels: Vec<(ExtensionKey, ExtensionShutdownChannel)> = Vec::new();
        let mut passive = Vec::new();
        let mut pending_starts: HashSet<ExtensionKey> = HashSet::new();
        let (started_tx, started_rx) = mpsc::unbounded_channel::<ExtensionKey>();

        for (mut ext_wrapper, entity_key) in extensions {
            if ext_wrapper.is_passive() {
                passive.push(ext_wrapper);
                continue;
            }
            let ext_id = ext_wrapper.name();
            let key = ExtensionKey::new(ext_id.clone(), ext_wrapper.variant());
            let control_sender = ext_wrapper.extension_control_sender();
            let control_sender_keepalive = ext_wrapper.extension_control_sender();
            let shutdown_channel = ext_wrapper.take_shutdown_sender();
            let telemetry_guard = ext_wrapper.take_telemetry_guard();
            monitor.register(ext_ctx, key.clone(), entity_key, control_sender);
            if let Some(channel) = shutdown_channel {
                shutdown_channels.push((key.clone(), channel));
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
                drop(telemetry_guard);
                drop(control_sender_keepalive);
                (task_key, res)
            };
            let handle = local_tasks.spawn_local(fut);
            let _ = task_id_to_key.insert(handle.id(), key.clone());
            futures.push(handle);
            let _ = pending_starts.insert(key);
        }
        // Drop the seed so `started_rx.recv()` returns None once all clones drop.
        drop(started_tx);

        Self {
            futures,
            task_id_to_key,
            shutdown_channels,
            _passive: passive,
            phase: LifecyclePhase::Running,
            monitor,
            started_rx,
            pending_starts,
        }
    }

    #[allow(dead_code)]
    pub fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Yields the next extension completion or monitor tick. Spawn-handshake
    /// signals on `started_rx` are drained silently — they only exist to
    /// power `wait_all_spawned`.
    pub async fn next_event(&mut self) -> LifecycleEvent {
        loop {
            let shutdown_initiated = matches!(self.phase, LifecyclePhase::ShuttingDown { .. });
            let Self {
                futures,
                task_id_to_key,
                shutdown_channels,
                monitor,
                started_rx,
                pending_starts,
                ..
            } = self;
            if futures.is_empty() {
                tokio::select! {
                    biased;
                    Some(key) = started_rx.recv() => {
                        let _ = pending_starts.remove(&key);
                        continue;
                    }
                    now = monitor.next_tick() => return LifecycleEvent::MonitorTick(now),
                }
            }
            tokio::select! {
                biased;
                Some(key) = started_rx.recv() => {
                    let _ = pending_starts.remove(&key);
                    continue;
                }
                Some(joined) = futures.next() => {
                    if let Some(k) = Self::pending_key_for(&joined, task_id_to_key) {
                        let _ = pending_starts.remove(&k);
                    }
                    return LifecycleEvent::Completion(
                        Self::route_joined(monitor, task_id_to_key, shutdown_channels, shutdown_initiated, joined)
                    );
                }
                now = monitor.next_tick() => return LifecycleEvent::MonitorTick(now),
            }
        }
    }

    /// Returns once every non-passive extension has been polled at least once
    /// (its task body sent the spawn handshake). Tasks that complete or panic
    /// before signalling are routed through the normal completion path; the
    /// first surfaced error is returned. A task that signals and then
    /// completes (success or failure) before the barrier observes the
    /// completion is also surfaced as an error — `Ok(())` upgrades to
    /// `ExtensionExitedBeforeShutdown` via `route_joined`.
    pub async fn wait_all_spawned(&mut self) -> Result<(), Error> {
        loop {
            let Self {
                futures,
                task_id_to_key,
                shutdown_channels,
                monitor,
                started_rx,
                pending_starts,
                ..
            } = self;

            // Surface any ready completion before considering the barrier
            // satisfied. During the barrier every completion is an error,
            // so a task that signaled and then died must not be masked by
            // pending_starts having already been cleared by the signal.
            if let Some(Some(joined)) = futures.next().now_or_never() {
                if let Some(k) = Self::pending_key_for(&joined, task_id_to_key) {
                    let _ = pending_starts.remove(&k);
                }
                match Self::route_joined(monitor, task_id_to_key, shutdown_channels, false, joined)
                {
                    Ok(Ok(())) => unreachable!(
                        "route_joined(shutdown_initiated=false) must upgrade Ok(()) to ExtensionExitedBeforeShutdown",
                    ),
                    Ok(Err(e)) => return Err(e),
                    Err(e) => {
                        return Err(Error::JoinTaskError {
                            is_canceled: e.is_cancelled(),
                            is_panic: e.is_panic(),
                            error: e.to_string(),
                        });
                    }
                }
            }

            if pending_starts.is_empty() {
                return Ok(());
            }
            tokio::select! {
                biased;
                Some(key) = started_rx.recv() => {
                    let _ = pending_starts.remove(&key);
                }
                Some(joined) = futures.next(), if !futures.is_empty() => {
                    if let Some(k) = Self::pending_key_for(&joined, task_id_to_key) {
                        let _ = pending_starts.remove(&k);
                    }
                    match Self::route_joined(monitor, task_id_to_key, shutdown_channels, false, joined) {
                        Ok(Ok(())) => unreachable!(
                            "route_joined(shutdown_initiated=false) must upgrade Ok(()) to ExtensionExitedBeforeShutdown",
                        ),
                        Ok(Err(e)) => return Err(e),
                        Err(e) => {
                            return Err(Error::JoinTaskError {
                                is_canceled: e.is_cancelled(),
                                is_panic: e.is_panic(),
                                error: e.to_string(),
                            });
                        }
                    }
                }
                else => return Ok(()),
            }
        }
    }

    /// Returns the `ExtensionKey` associated with a `joined` outcome so the
    /// caller can prune `pending_starts`. Falls back to `task_id_to_key`
    /// when the task panicked before signalling (and thus before its body
    /// could embed the key in the `Ok` payload).
    fn pending_key_for(
        joined: &Result<(ExtensionKey, Result<(), Error>), JoinError>,
        task_id_to_key: &HashMap<task::Id, ExtensionKey>,
    ) -> Option<ExtensionKey> {
        match joined {
            Ok((k, _)) => Some(k.clone()),
            Err(e) => task_id_to_key.get(&e.id()).cloned(),
        }
    }

    /// Records the completion in the monitor, prunes the shutdown channel for
    /// this extension, and upgrades an early `Ok(())` to
    /// `ExtensionExitedBeforeShutdown` when shutdown hasn't been initiated.
    fn route_joined(
        monitor: &mut ExtensionMetricsMonitor,
        task_id_to_key: &mut HashMap<task::Id, ExtensionKey>,
        shutdown_channels: &mut Vec<(ExtensionKey, ExtensionShutdownChannel)>,
        shutdown_initiated: bool,
        joined: Result<(ExtensionKey, Result<(), Error>), JoinError>,
    ) -> Result<Result<(), Error>, JoinError> {
        match joined {
            Ok((key, res)) => {
                task_id_to_key.retain(|_, k| k != &key);
                shutdown_channels.retain(|(k, _)| k != &key);
                let res = match res {
                    Ok(()) if !shutdown_initiated => Err(Error::ExtensionExitedBeforeShutdown {
                        extension: key.id.to_string(),
                    }),
                    other => other,
                };
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
                    shutdown_channels.retain(|(k, _)| k != &key);
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

    pub fn monitor_tick(&mut self, now: Instant, reporter: &mut MetricsReporter) {
        self.monitor.tick(now, reporter);
    }

    #[allow(dead_code)]
    pub fn shutdown_initiated(&self) -> bool {
        matches!(self.phase, LifecyclePhase::ShuttingDown { .. })
    }

    /// Broadcasts `Shutdown` to every active+background extension via its
    /// priority oneshot channel. Single-shot: subsequent calls are no-ops.
    pub fn initiate_shutdown(&mut self, reason: Option<&str>) {
        if matches!(self.phase, LifecyclePhase::ShuttingDown { .. }) {
            return;
        }
        if self.shutdown_channels.is_empty() {
            return;
        }

        let deadline = Instant::now() + EXTENSION_SHUTDOWN_GRACE;
        self.phase = LifecyclePhase::ShuttingDown { deadline };

        let reason = reason.unwrap_or(DEFAULT_SHUTDOWN_REASON).to_string();
        for (key, channel) in self.shutdown_channels.drain(..) {
            let payload = ShutdownPayload {
                deadline,
                reason: reason.clone(),
            };
            match channel.sender.send(payload) {
                Ok(()) => {
                    self.monitor
                        .apply_event(ExtensionLifecycleEvent::ShutdownSent { key });
                }
                Err(_payload) => {
                    otel_warn!(
                        "extension.shutdown.receiver_dropped",
                        extension = channel.name.as_ref(),
                    );
                }
            }
        }
    }

    /// Drains remaining extension tasks, bounded by the shutdown deadline.
    pub async fn drain_until_deadline(&mut self) {
        if self.futures.is_empty() {
            return;
        }
        let deadline = match self.phase {
            LifecyclePhase::ShuttingDown { deadline } => deadline,
            LifecyclePhase::Running => {
                let deadline = Instant::now() + EXTENSION_SHUTDOWN_GRACE;
                self.phase = LifecyclePhase::ShuttingDown { deadline };
                deadline
            }
        };
        let drain_deadline = TokioInstant::from_std(deadline + EXTENSION_SHUTDOWN_DRAIN_SLACK);

        let futures = &mut self.futures;
        let task_id_to_key = &mut self.task_id_to_key;
        let shutdown_channels = &mut self.shutdown_channels;
        let monitor = &mut self.monitor;
        let drain = async {
            while let Some(result) = futures.next().await {
                match result {
                    Ok((key, Ok(()))) => {
                        task_id_to_key.retain(|_, k| k != &key);
                        shutdown_channels.retain(|(k, _)| k != &key);
                        monitor.apply_event(ExtensionLifecycleEvent::Completed {
                            key,
                            outcome: ExtensionOutcome::Ok,
                        });
                    }
                    Ok((key, Err(e))) => {
                        task_id_to_key.retain(|_, k| k != &key);
                        shutdown_channels.retain(|(k, _)| k != &key);
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
                            shutdown_channels.retain(|(k, _)| k != &key);
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
            self.monitor.mark_stragglers_as_timeout();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::control::{
        ExtensionControlMsg, ExtensionControlSender, ExtensionShutdownChannel, ShutdownPayload,
    };
    use crate::extension::wrapper::ExtensionVariant;
    use tokio::sync::oneshot;

    fn make_shutdown_channel(
        id: &'static str,
    ) -> (
        ExtensionKey,
        ExtensionShutdownChannel,
        oneshot::Receiver<ShutdownPayload>,
    ) {
        let key = ExtensionKey::new(id.into(), ExtensionVariant::Local);
        let (tx, rx) = oneshot::channel();
        let channel = ExtensionShutdownChannel {
            name: id.into(),
            sender: tx,
        };
        (key, channel, rx)
    }

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
                shutdown_channels: Vec::new(),
                _passive: Vec::new(),
                phase: LifecyclePhase::ShuttingDown {
                    deadline: injected_deadline,
                },
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx,
                pending_starts: HashSet::new(),
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
            let entity_key =
                ext_ctx.register_extension_entity("boom".into(), ExtensionVariant::Local);

            let mut monitor = ExtensionMetricsMonitor::new(
                ext_ctx.clone(),
                Duration::from_millis(50),
                Duration::from_millis(50),
            );
            monitor.register(&ext_ctx, key.clone(), entity_key, None);

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
                shutdown_channels: Vec::new(),
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor,
                started_rx,
                pending_starts: HashSet::new(),
            };

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

    /// `initiate_shutdown` is single-shot: a second call must be a no-op and
    /// leave the phase in `ShuttingDown`.
    #[test]
    fn initiate_shutdown_is_single_shot() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            let (key_a, channel_a, mut rx_a) = make_shutdown_channel("a");
            let (key_b, channel_b, mut rx_b) = make_shutdown_channel("b");

            let mut lifecycle = ExtensionLifecycle {
                futures: FuturesUnordered::new(),
                task_id_to_key: HashMap::new(),
                shutdown_channels: vec![(key_a, channel_a), (key_b, channel_b)],
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx: mpsc::unbounded_channel().1,
                pending_starts: HashSet::new(),
            };

            lifecycle.initiate_shutdown(Some("first"));
            lifecycle.initiate_shutdown(Some("second"));

            assert!(
                lifecycle.shutdown_initiated(),
                "phase must transition to ShuttingDown on the first call"
            );
            assert!(
                lifecycle.shutdown_channels.is_empty(),
                "oneshot senders must be drained after the first call \
                 so the second call is a structural no-op"
            );

            let payload_a = rx_a
                .try_recv()
                .expect("a must receive exactly one Shutdown");
            assert_eq!(payload_a.reason, "first");
            let payload_b = rx_b
                .try_recv()
                .expect("b must receive exactly one Shutdown");
            assert_eq!(payload_b.reason, "first");
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

    /// Two scopes (today: pipelines) each own a distinct `ExtensionLifecycle`;
    /// a shutdown in one must not reach the other's extensions.
    #[test]
    fn initiate_shutdown_in_one_scope_does_not_reach_another_scope() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ctx_a, _reg_a) = crate::testing::test_extension_ctx();
            let (ctx_b, _reg_b) = crate::testing::test_extension_ctx();

            let (key_a, channel_a, mut rx_a) = make_shutdown_channel("a");
            let (key_b, channel_b, mut rx_b) = make_shutdown_channel("b");

            let mut life_a = ExtensionLifecycle {
                futures: FuturesUnordered::new(),
                task_id_to_key: HashMap::new(),
                shutdown_channels: vec![(key_a, channel_a)],
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor: ExtensionMetricsMonitor::disabled(ctx_a),
                started_rx: mpsc::unbounded_channel().1,
                pending_starts: HashSet::new(),
            };
            let life_b = ExtensionLifecycle {
                futures: FuturesUnordered::new(),
                task_id_to_key: HashMap::new(),
                shutdown_channels: vec![(key_b, channel_b)],
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor: ExtensionMetricsMonitor::disabled(ctx_b),
                started_rx: mpsc::unbounded_channel().1,
                pending_starts: HashSet::new(),
            };

            life_a.initiate_shutdown(Some("a-only"));

            assert!(
                rx_a.try_recv().is_ok(),
                "scope A's extension must receive its own shutdown"
            );
            assert!(
                rx_b.try_recv().is_err(),
                "scope B's extension must NOT receive scope A's shutdown"
            );
            assert!(
                !life_b.shutdown_initiated(),
                "scope B's phase must remain Running"
            );
        }));
    }

    /// `CollectTelemetry` fanout from one scope's monitor must not reach
    /// extensions in another scope.
    #[test]
    fn collect_telemetry_fanout_is_scoped_to_owning_monitor() {
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
                sender: Sender::new_local_mpsc_sender(tx_a),
            };
            let sender_b = ExtensionControlSender {
                sender: Sender::new_local_mpsc_sender(tx_b),
            };

            let key_a = ExtensionKey::new("a".into(), ExtensionVariant::Local);
            let key_b = ExtensionKey::new("b".into(), ExtensionVariant::Local);
            let ent_a = ctx_a.register_extension_entity("a".into(), ExtensionVariant::Local);
            let ent_b = ctx_b.register_extension_entity("b".into(), ExtensionVariant::Local);
            monitor_a.register(&ctx_a, key_a.clone(), ent_a, Some(sender_a));
            monitor_b.register(&ctx_b, key_b.clone(), ent_b, Some(sender_b));

            let (rep_tx, _rep_rx) = flume::bounded(8);
            let mut reporter = MetricsReporter::new(rep_tx);

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

    /// On task completion, the extension must be pruned from
    /// `shutdown_channels` so a subsequent `initiate_shutdown` does not signal
    /// a dropped oneshot receiver.
    #[test]
    fn completed_extensions_are_pruned_from_shutdown_channels() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            let (key_a, channel_a, _rx_a) = make_shutdown_channel("a");
            let (key_b, channel_b, _rx_b) = make_shutdown_channel("b");

            let futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>> =
                FuturesUnordered::new();
            let handle = local_tasks.spawn_local({
                let key = key_a.clone();
                async move { (key, Ok(())) }
            });
            let mut task_id_to_key = HashMap::new();
            let _ = task_id_to_key.insert(handle.id(), key_a.clone());
            futures.push(handle);

            let (_started_tx, started_rx) = mpsc::unbounded_channel();
            let mut lifecycle = ExtensionLifecycle {
                futures,
                task_id_to_key,
                shutdown_channels: vec![(key_a.clone(), channel_a), (key_b.clone(), channel_b)],
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx,
                pending_starts: HashSet::new(),
            };

            match lifecycle.next_event().await {
                LifecycleEvent::Completion(Ok(Err(Error::ExtensionExitedBeforeShutdown {
                    extension,
                }))) => {
                    assert_eq!(extension, "a");
                }
                LifecycleEvent::Completion(other) => {
                    panic!(
                        "expected Completion(Ok(Err(ExtensionExitedBeforeShutdown))) for A, got {other:?}"
                    )
                }
                LifecycleEvent::MonitorTick(_) => {
                    panic!("expected Completion before any monitor tick")
                }
            }

            assert!(
                !lifecycle.shutdown_channels.iter().any(|(k, _)| k == &key_a),
                "completed extension A must be pruned from `shutdown_channels`"
            );
            assert!(
                lifecycle.shutdown_channels.iter().any(|(k, _)| k == &key_b),
                "non-completed extension B must remain in `shutdown_channels`"
            );
        }));
    }

    /// After a completion is routed, `initiate_shutdown` must not signal the
    /// completed extension's oneshot — even if its receiver is still alive.
    #[test]
    fn initiate_shutdown_skips_already_completed_extension() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            let (key_a, channel_a, mut rx_a) = make_shutdown_channel("a");
            let (key_b, channel_b, mut rx_b) = make_shutdown_channel("b");

            let futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>> =
                FuturesUnordered::new();
            let handle = local_tasks.spawn_local({
                let key = key_a.clone();
                async move { (key, Ok(())) }
            });
            let mut task_id_to_key = HashMap::new();
            let _ = task_id_to_key.insert(handle.id(), key_a.clone());
            futures.push(handle);

            let (_started_tx, started_rx) = mpsc::unbounded_channel();
            let mut lifecycle = ExtensionLifecycle {
                futures,
                task_id_to_key,
                shutdown_channels: vec![(key_a.clone(), channel_a), (key_b.clone(), channel_b)],
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx,
                pending_starts: HashSet::new(),
            };

            let _ = lifecycle.next_event().await;
            lifecycle.initiate_shutdown(Some("after completion"));

            assert!(
                rx_a.try_recv().is_err(),
                "initiate_shutdown must not signal completed extension A"
            );
            assert!(
                rx_b.try_recv().is_ok(),
                "initiate_shutdown must still reach non-completed extension B"
            );
        }));
    }

    #[test]
    fn wait_all_spawned_returns_after_extensions_signal_spawned() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            let (started_tx, started_rx) = mpsc::unbounded_channel::<ExtensionKey>();
            let futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>> =
                FuturesUnordered::new();
            let mut task_id_to_key = HashMap::new();
            let mut pending_starts: HashSet<ExtensionKey> = HashSet::new();

            for name in ["x", "y"] {
                let key = ExtensionKey::local(name);
                let started_tx = started_tx.clone();
                let task_key = key.clone();
                let handle = local_tasks.spawn_local(async move {
                    let _ = started_tx.send(task_key.clone());
                    std::future::pending::<()>().await;
                    (task_key, Ok::<(), Error>(()))
                });
                let _ = task_id_to_key.insert(handle.id(), key.clone());
                futures.push(handle);
                let _ = pending_starts.insert(key);
            }
            drop(started_tx);

            let mut lifecycle = ExtensionLifecycle {
                futures,
                task_id_to_key,
                shutdown_channels: Vec::new(),
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx,
                pending_starts,
            };

            let outcome =
                tokio::time::timeout(Duration::from_secs(1), lifecycle.wait_all_spawned()).await;

            assert!(outcome.is_ok(), "wait_all_spawned must not hang");
            assert!(outcome.unwrap().is_ok());
            assert!(lifecycle.pending_starts.is_empty());
        }));
    }

    #[test]
    fn wait_all_spawned_surfaces_completion_when_outstanding_signals_reach_zero() {
        // When several tasks signal before the barrier observes them and one
        // of those tasks has *also* already completed with an error, the
        // barrier must surface that error rather than returning Ok just
        // because pending_starts emptied. Otherwise the caller starts node
        // tasks against an already-dead extension.
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            let (started_tx, started_rx) = mpsc::unbounded_channel::<ExtensionKey>();
            let futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>> =
                FuturesUnordered::new();
            let mut task_id_to_key = HashMap::new();
            let mut pending_starts: HashSet<ExtensionKey> = HashSet::new();

            // A: signals, then fails immediately.
            let a_key = ExtensionKey::local("a-fast-fail");
            {
                let started_tx = started_tx.clone();
                let task_key = a_key.clone();
                let handle = local_tasks.spawn_local(async move {
                    let _ = started_tx.send(task_key.clone());
                    (
                        task_key,
                        Err::<(), Error>(Error::ExtensionExitedBeforeShutdown {
                            extension: "a-fast-fail".into(),
                        }),
                    )
                });
                let _ = task_id_to_key.insert(handle.id(), a_key.clone());
                futures.push(handle);
                let _ = pending_starts.insert(a_key);
            }

            // B: signals, then parks forever.
            let b_key = ExtensionKey::local("b-pending");
            {
                let started_tx = started_tx.clone();
                let task_key = b_key.clone();
                let handle = local_tasks.spawn_local(async move {
                    let _ = started_tx.send(task_key.clone());
                    std::future::pending::<()>().await;
                    (task_key, Ok::<(), Error>(()))
                });
                let _ = task_id_to_key.insert(handle.id(), b_key.clone());
                futures.push(handle);
                let _ = pending_starts.insert(b_key);
            }

            drop(started_tx);

            let mut lifecycle = ExtensionLifecycle {
                futures,
                task_id_to_key,
                shutdown_channels: Vec::new(),
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx,
                pending_starts,
            };

            let outcome =
                tokio::time::timeout(Duration::from_millis(200), lifecycle.wait_all_spawned())
                    .await;

            let res = outcome.expect("barrier must not hang");
            assert!(
                res.is_err(),
                "barrier must surface a completed-with-error task even when all signals have already been consumed; got Ok",
            );
        }));
    }

    #[test]
    fn wait_all_spawned_surfaces_panic_before_signal() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            let (_started_tx, started_rx) = mpsc::unbounded_channel::<ExtensionKey>();
            let futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>> =
                FuturesUnordered::new();
            let mut task_id_to_key = HashMap::new();

            let key = ExtensionKey::local("paniker");
            let handle = local_tasks.spawn_local(async {
                panic!("synthetic first-poll panic");
                #[allow(unreachable_code)]
                (ExtensionKey::local("paniker"), Ok::<(), Error>(()))
            });
            let _ = task_id_to_key.insert(handle.id(), key.clone());
            futures.push(handle);

            let mut lifecycle = ExtensionLifecycle {
                futures,
                task_id_to_key,
                shutdown_channels: Vec::new(),
                _passive: Vec::new(),
                phase: LifecyclePhase::Running,
                monitor: ExtensionMetricsMonitor::disabled(ext_ctx),
                started_rx,
                pending_starts: HashSet::from([key]),
            };

            let prev_hook = std::panic::take_hook();
            std::panic::set_hook(Box::new(|_| {}));

            let outcome =
                tokio::time::timeout(Duration::from_secs(1), lifecycle.wait_all_spawned()).await;

            std::panic::set_hook(prev_hook);

            let res = outcome.expect("barrier must not hang on panic-before-signal");
            match res {
                Err(Error::JoinTaskError { is_panic, .. }) => assert!(is_panic),
                other => panic!("expected JoinTaskError(is_panic), got {other:?}"),
            }
            assert!(lifecycle.pending_starts.is_empty());
        }));
    }

    use crate::channel_metrics::ChannelMetricsRegistry;
    use crate::extension::ExtensionWrapper;

    #[derive(Clone)]
    struct ShutdownAwaitExt {
        observed_close: std::rc::Rc<std::cell::Cell<bool>>,
        observed_shutdown: std::rc::Rc<std::cell::Cell<bool>>,
    }

    #[async_trait::async_trait(?Send)]
    impl crate::local::extension::Extension for ShutdownAwaitExt {
        async fn start(
            self: std::rc::Rc<Self>,
            mut ctrl: crate::local::extension::ControlChannel,
            _eh: crate::extension::wrapper::EffectHandler,
        ) -> Result<crate::terminal_state::TerminalState, Error> {
            loop {
                match ctrl.recv().await {
                    Ok(ExtensionControlMsg::Shutdown { .. }) => {
                        self.observed_shutdown.set(true);
                        break;
                    }
                    Ok(_) => continue,
                    Err(_) => {
                        self.observed_close.set(true);
                        return Err(Error::ExtensionExitedBeforeShutdown {
                            extension: "shutdown-await".into(),
                        });
                    }
                }
            }
            Ok(crate::terminal_state::TerminalState::default())
        }
    }

    #[test]
    fn extension_control_channel_stays_open_with_pipeline_metrics_disabled() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();

            let observed_close = std::rc::Rc::new(std::cell::Cell::new(false));
            let observed_shutdown = std::rc::Rc::new(std::cell::Cell::new(false));
            let ext = std::rc::Rc::new(ShutdownAwaitExt {
                observed_close: observed_close.clone(),
                observed_shutdown: observed_shutdown.clone(),
            });

            let cfg = crate::config::ExtensionConfig::new("disabled-monitor");
            let user = std::sync::Arc::new(otap_df_config::extension::ExtensionUserConfig::new(
                "urn:otap:extension:test".into(),
                serde_json::Value::Null,
            ));
            let wrapper = ExtensionWrapper::builder("disabled-monitor".into(), user, &cfg)
                .active()
                .local(ext)
                .build()
                .unwrap()
                .take_local()
                .unwrap();
            let entity_key = ext_ctx
                .register_extension_entity("disabled-monitor".into(), ExtensionVariant::Local);

            let monitor = ExtensionMetricsMonitor::disabled(ext_ctx.clone());
            let (tx, _rx) = flume::bounded(1);
            let reporter = MetricsReporter::new(tx);

            let mut lifecycle = ExtensionLifecycle::spawn(
                vec![(wrapper, entity_key)],
                &local_tasks,
                reporter,
                &ext_ctx,
                monitor,
            );

            let barrier =
                tokio::time::timeout(Duration::from_secs(1), lifecycle.wait_all_spawned())
                    .await
                    .expect("spawn barrier must not hang");
            barrier.expect("spawn barrier must succeed");

            tokio::time::sleep(Duration::from_millis(50)).await;
            assert!(
                !observed_close.get(),
                "extension's control channel must not close before Shutdown is delivered"
            );

            lifecycle.initiate_shutdown(Some("test"));
            lifecycle.drain_until_deadline().await;

            assert!(
                observed_shutdown.get(),
                "extension must have observed exactly one Shutdown"
            );
            assert!(
                !observed_close.get(),
                "extension must not have seen RecvError::Closed before Shutdown"
            );
        }));
    }

    #[test]
    fn extension_telemetry_guard_held_for_full_extension_lifetime() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, registry) = crate::testing::test_extension_ctx();

            let observed_close = std::rc::Rc::new(std::cell::Cell::new(false));
            let observed_shutdown = std::rc::Rc::new(std::cell::Cell::new(false));
            let ext = std::rc::Rc::new(ShutdownAwaitExt {
                observed_close: observed_close.clone(),
                observed_shutdown: observed_shutdown.clone(),
            });

            let cfg = crate::config::ExtensionConfig::new("guarded");
            let user = std::sync::Arc::new(otap_df_config::extension::ExtensionUserConfig::new(
                "urn:otap:extension:test".into(),
                serde_json::Value::Null,
            ));
            let mut bundle = ExtensionWrapper::builder("guarded".into(), user, &cfg)
                .active()
                .local(ext)
                .build()
                .unwrap();

            let mut channel_metrics = ChannelMetricsRegistry::default();
            let keys = bundle.wire_telemetry(
                "guarded".into(),
                &ext_ctx,
                &mut channel_metrics,
                false,
            );
            let entity_key = keys
                .local
                .expect("wire_telemetry must register the local variant's entity");
            let wrapper = bundle.take_local().unwrap();

            assert!(
                registry.visit_entity(entity_key, |_| ()).is_some(),
                "baseline: entity must be registered before spawn"
            );

            let monitor = ExtensionMetricsMonitor::disabled(ext_ctx.clone());
            let (tx, _rx) = flume::bounded(1);
            let reporter = MetricsReporter::new(tx);

            let mut lifecycle = ExtensionLifecycle::spawn(
                vec![(wrapper, entity_key)],
                &local_tasks,
                reporter,
                &ext_ctx,
                monitor,
            );

            tokio::time::timeout(
                Duration::from_secs(1),
                lifecycle.wait_all_spawned(),
            )
            .await
            .expect("spawn barrier must not hang")
            .expect("spawn barrier must succeed");

            tokio::time::sleep(Duration::from_millis(30)).await;
            assert!(
                registry.visit_entity(entity_key, |_| ()).is_some(),
                "entity must remain registered for the entire duration of the extension's start().await",
            );

            lifecycle.initiate_shutdown(Some("test"));
            lifecycle.drain_until_deadline().await;

            assert!(observed_shutdown.get(), "extension must have observed Shutdown");
            assert!(!observed_close.get(), "extension control channel must not have closed early");

            drop(lifecycle);

            assert!(
                registry.visit_entity(entity_key, |_| ()).is_none(),
                "EntityTelemetryGuard must unregister the entity after start() returns",
            );
        }));
    }
}
