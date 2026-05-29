// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension lifecycle holder for an extension-hosting runtime.
//!
//! Owns the spawned active+background extension tasks, the control
//! senders used to broadcast `Shutdown` to them, and the passive
//! extension wrappers that must outlive the run for capability
//! handles to remain valid. Encapsulates the "extensions start
//! first, shut down last" invariant so the host runtime doesn't
//! interleave that policy with task-driving code.
//!
//! ## Shutdown timing
//!
//! Extensions shut down strictly after all data-path tasks (nodes
//! and the dispatcher) have terminated. Because shutdown is
//! sequential — not simultaneous with the data path — the
//! extension shutdown deadline is computed locally as
//! `now() + EXTENSION_SHUTDOWN_GRACE` rather than reusing the
//! host's data-path drain deadline. This gives extensions a fresh
//! cleanup budget starting from the moment the data path is fully
//! drained.
//!
//! See `runtime_pipeline.rs::run_forever` for how this is wired in
//! at pipeline scope today.

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
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::task::{JoinError, JoinHandle, LocalSet};
use tokio::time::Instant as TokioInstant;

/// Cleanup window granted to extensions after the data path has
/// drained. Extensions that don't terminate within this window will
/// be left to the runtime's natural drop semantics when
/// `run_forever` returns.
pub(crate) const EXTENSION_SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// Slack added past `EXTENSION_SHUTDOWN_GRACE` before the runtime
/// hard-stops draining extension tasks. The extension's cooperative
/// budget is exactly `EXTENSION_SHUTDOWN_GRACE`; the runtime then
/// waits this much longer for the task to actually return after
/// cleanup completes (context switch + `JoinHandle` poll latency).
/// Without it, an extension that finishes right at the deadline
/// would race the drain timeout and be reported as a timeout despite
/// terminating correctly.
pub(crate) const EXTENSION_SHUTDOWN_DRAIN_SLACK: Duration = Duration::from_millis(500);

/// Per-extension upper bound on how long `broadcast_shutdown` will
/// wait to enqueue `Shutdown`. Prevents one backed-up extension from
/// stalling delivery to the others.
pub(crate) const EXTENSION_SHUTDOWN_SEND_TIMEOUT: Duration = Duration::from_millis(500);

/// Default reason recorded in the `Shutdown` broadcast when the caller
/// does not supply one. Hosts should pass a scope-appropriate reason
/// (e.g., `"pipeline data-path drained"`) via [`ExtensionLifecycle::broadcast_shutdown`]
/// where possible.
const DEFAULT_SHUTDOWN_REASON: &str = "host data-path drained";

/// Event surfaced by [`ExtensionLifecycle::next_event`] so the host's
/// outer `select!` can drive completions and monitor ticks through a
/// single `&mut` borrow of the lifecycle.
pub(crate) enum LifecycleEvent {
    /// An active+background extension task finished. The inner result
    /// preserves the same `Result<Result<(), Error>, JoinError>` shape
    /// callers already match on for direct task completions.
    Completion(Result<Result<(), Error>, JoinError>),
    /// The per-scope extension monitor's tick interval fired. The host
    /// should call [`ExtensionLifecycle::monitor_tick`] with the
    /// returned `Instant`.
    MonitorTick(Instant),
}

/// Holds the spawned extension tasks, control senders, and passive
/// wrappers for the duration of an extension-hosting run.
pub(crate) struct ExtensionLifecycle {
    /// Active+background extension `JoinHandle`s. Each yields its
    /// `ExtensionKey` alongside the lifecycle result so completion
    /// outcomes can be routed to the monitor without a side channel.
    futures: FuturesUnordered<JoinHandle<(ExtensionKey, Result<(), Error>)>>,
    /// Control senders for the extensions in [`Self::futures`], paired
    /// with their key so `broadcast_shutdown` can record a per-key
    /// `ShutdownSent` event in the monitor.
    shutdown_senders: Vec<(ExtensionKey, ExtensionControlSender)>,
    /// Passive extensions held alive for the duration of the run so
    /// any state their capability instances reference (via cloned
    /// `Arc`s minted by the builder) survives until `run_forever`
    /// returns and this struct is dropped.
    _passive: Vec<ExtensionWrapper>,
    /// One-shot latch: `true` after `Shutdown` has been broadcast.
    /// Prevents re-firing on subsequent loop iterations.
    shutdown_broadcast_fired: bool,
    /// Deadline established when [`Self::broadcast_shutdown`] fires.
    /// Used by [`Self::drain_until_deadline`] to bound how long the
    /// runtime will wait for extensions to honour `Shutdown` so a
    /// misbehaving extension can't hang the host runtime indefinitely.
    shutdown_deadline: Option<Instant>,
    /// Per-scope lifecycle/telemetry monitor. Records spawn/shutdown/
    /// completion events for each registered extension and drives the
    /// `CollectTelemetry` fan-out on its own tick cadence.
    monitor: ExtensionMetricsMonitor,
    /// Receives an `ExtensionKey` from each spawned task as its first
    /// action. `next_event` drains this internally so the monitor's
    /// `Spawned` event reflects "task body began executing" rather
    /// than the earlier "task registered with the executor".
    started_rx: mpsc::UnboundedReceiver<ExtensionKey>,
}

impl ExtensionLifecycle {
    /// Spawn all active+background extensions onto `local_tasks` and
    /// stash the passive ones. Active+background extensions begin
    /// running concurrently with the data path; passive extensions
    /// have no lifecycle but must remain owned for their capability
    /// state to remain valid.
    ///
    /// Each non-passive wrapper is registered with `monitor` so spawn/
    /// shutdown/completion events are recorded and `CollectTelemetry`
    /// fan-out has a routing target.
    pub fn spawn(
        extensions: Vec<(ExtensionWrapper, EntityKey)>,
        local_tasks: &LocalSet,
        metrics_reporter: MetricsReporter,
        ext_ctx: &ExtensionContext,
        mut monitor: ExtensionMetricsMonitor,
    ) -> Self {
        let futures = FuturesUnordered::new();
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
                // Signal "task body is running" as the first action so
                // the monitor's Spawned event reflects actual execution,
                // not just registration with the LocalSet.
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
            futures.push(local_tasks.spawn_local(fut));
        }
        // Drop the seed sender so `started_rx.recv()` returns `None`
        // once every per-task clone has been dropped (i.e., once all
        // spawned tasks have either signalled or been cancelled).
        drop(started_tx);

        Self {
            futures,
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
    #[allow(dead_code)] // host uses `next_event`; kept as public introspection
    pub fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Awaits either the next active+background extension completion
    /// or the next monitor tick, whichever fires first. Hosts plug
    /// this into their main `select!` so the monitor's tick cadence
    /// shares the same task as completion handling — avoiding the
    /// double-`&mut` borrow that two separate methods would require.
    ///
    /// `Spawned` signals from started tasks are absorbed internally
    /// (recorded into the monitor) and never surface to the host —
    /// the host only ever sees completions and monitor ticks.
    pub async fn next_event(&mut self) -> LifecycleEvent {
        loop {
            let Self {
                futures,
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
                    return LifecycleEvent::Completion(Self::route_joined(monitor, joined));
                }
                now = monitor.next_tick() => return LifecycleEvent::MonitorTick(now),
            }
        }
    }

    /// Maps a joined task result to the outer `Result` shape expected
    /// by `next_event` callers and records the completion outcome in
    /// the monitor (when the task didn't panic/cancel).
    fn route_joined(
        monitor: &mut ExtensionMetricsMonitor,
        joined: Result<(ExtensionKey, Result<(), Error>), JoinError>,
    ) -> Result<Result<(), Error>, JoinError> {
        match joined {
            Ok((key, res)) => {
                let outcome = match &res {
                    Ok(()) => ExtensionOutcome::Ok,
                    Err(e) => ExtensionOutcome::Err(e.to_string()),
                };
                monitor.apply_event(ExtensionLifecycleEvent::Completed { key, outcome });
                Ok(res)
            }
            Err(e) => {
                // The task panicked or was cancelled; the key is lost
                // with the task frame. Stragglers still in `Spawned`/
                // `ShutdownSent` are reconciled by `drain_until_deadline`.
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
    /// [`EXTENSION_SHUTDOWN_SEND_TIMEOUT`]; the deadline carried in
    /// the message is `now() + EXTENSION_SHUTDOWN_GRACE` (a fresh
    /// cleanup window, not a continuation of the host-scope deadline).
    ///
    /// `reason` is propagated verbatim in the `Shutdown` message so
    /// the host can describe what triggered the shutdown in scope-
    /// appropriate terms; `None` falls back to [`DEFAULT_SHUTDOWN_REASON`].
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

    /// Drain remaining active+background extension tasks, but never
    /// past the shutdown deadline.
    ///
    /// `Shutdown` is cooperative — extensions may ignore it or take
    /// longer than the grace window to exit. Without this bound, an
    /// extension that never returns from `start()` would hang the host
    /// runtime forever. After the deadline elapses, any still-running
    /// futures are dropped with a warning; the runtime's natural drop
    /// semantics take over once the lifecycle holder itself is dropped.
    ///
    /// No-op if there are no remaining futures or if shutdown has not
    /// been broadcast (in which case there is no deadline yet).
    pub async fn drain_until_deadline(&mut self) {
        if self.futures.is_empty() {
            return;
        }
        // If the caller invokes drain without a prior broadcast there
        // is no deadline yet — synthesize one from the same grace
        // window so we still bound the wait.
        let deadline = self
            .shutdown_deadline
            .get_or_insert_with(|| Instant::now() + EXTENSION_SHUTDOWN_GRACE);
        // See `EXTENSION_SHUTDOWN_DRAIN_SLACK` for rationale.
        let drain_deadline = TokioInstant::from_std(*deadline + EXTENSION_SHUTDOWN_DRAIN_SLACK);

        let drain = async {
            while let Some(result) = self.futures.next().await {
                match result {
                    Ok((key, Ok(()))) => {
                        self.monitor.apply_event(ExtensionLifecycleEvent::Completed {
                            key,
                            outcome: ExtensionOutcome::Ok,
                        });
                    }
                    Ok((key, Err(e))) => {
                        otel_warn!("extension.shutdown.task.error", error = format!("{e}"));
                        self.monitor.apply_event(ExtensionLifecycleEvent::Completed {
                            key,
                            outcome: ExtensionOutcome::Err(e.to_string()),
                        });
                    }
                    Err(e) => {
                        otel_warn!(
                            "extension.shutdown.task.join_error",
                            is_canceled = e.is_cancelled(),
                            is_panic = e.is_panic(),
                            error = e.to_string()
                        );
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
            // Any extension still in `Spawned` or `ShutdownSent` after
            // the bounded drain timed out is reconciled in the monitor
            // as a shutdown timeout.
            self.monitor.mark_pending_as_timeout();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::extension::wrapper::ExtensionVariant;

    /// Regression test: a misbehaving extension that never returns
    /// must not stall `drain_until_deadline` past its deadline. The
    /// deadline is injected directly to keep the test fast.
    #[test]
    fn drain_until_deadline_is_bounded_for_stuck_extension() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let (ext_ctx, _registry) = crate::testing::test_extension_ctx();
            let key = ExtensionKey::new("stuck".into(), ExtensionVariant::Local);
            let futures = FuturesUnordered::new();
            futures.push(tokio::task::spawn_local({
                let key = key.clone();
                async move {
                    // Misbehaving extension that ignores `Shutdown` and
                    // never returns from `start()`. `pending` is cancelled
                    // when the surrounding `LocalSet` drops at the end of
                    // the test, so this does not actually run forever.
                    std::future::pending::<()>().await;
                    (key, Ok(()))
                }
            }));

            let injected_deadline = Instant::now() + Duration::from_millis(100);
            let (_started_tx, started_rx) = mpsc::unbounded_channel();
            let mut lifecycle = ExtensionLifecycle {
                futures,
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

            // The drain must return shortly after the deadline +
            // slack, not hang on the never-completing extension task.
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
                "stuck extension should still be in `futures` after the bounded drain timed out",
            );
        }));
    }
}
