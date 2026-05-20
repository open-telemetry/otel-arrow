// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension lifecycle holder for the runtime pipeline.
//!
//! Owns the spawned active+background extension tasks, the control
//! senders used to broadcast `Shutdown` to them, and the passive
//! extension wrappers that must outlive the run for capability
//! handles to remain valid. Encapsulates the "extensions start
//! first, shut down last" invariant so the runtime pipeline doesn't
//! interleave that policy with task-driving code.
//!
//! ## Shutdown timing
//!
//! Extensions shut down strictly after all data-path tasks (nodes
//! and the dispatcher) have terminated. Because shutdown is
//! sequential — not simultaneous with the data path — the
//! extension shutdown deadline is computed locally as
//! `now() + EXTENSION_SHUTDOWN_GRACE` rather than reusing the
//! pipeline-wide deadline that drove the data-path drain. This
//! gives extensions a fresh cleanup budget starting from the
//! moment the data path is fully drained.
//!
//! See `runtime_pipeline.rs::run_forever` for how this is wired in.

use crate::control::{ExtensionControlMsg, ExtensionControlSender};
use crate::error::Error;
use crate::extension::ExtensionContext;
use crate::extension::ExtensionWrapper;
use futures::FutureExt;
use futures::future::LocalBoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};
use otap_df_config::ExtensionId;
use otap_df_telemetry::instrument::{Counter, Gauge};
use otap_df_telemetry::metrics::MetricSet;
use otap_df_telemetry::otel_warn;
use otap_df_telemetry::registry::{EntityKey, TelemetryRegistryHandle};
use otap_df_telemetry::reporter::MetricsReporter;
use otap_df_telemetry_macros::metric_set;
use std::collections::{HashMap, HashSet};
use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};
use tokio::task::{AbortHandle, JoinError, LocalSet};
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

const SHUTDOWN_REASON: &str = "pipeline data-path drained";

/// Wrapped extension task future used in [`ExtensionLifecycle::futures`].
///
/// Each spawned extension `JoinHandle` is wrapped so the yielded tuple
/// always carries the [`ExtensionId`] — even when the inner task is
/// cancelled and the join produces an [`Err(JoinError)`]. Without this
/// wrapping, cancellation would leave the per-extension entry stranded
/// in `pending` and risk being misattributed as a `shutdown.timeout`.
type ExtensionFuture = LocalBoxFuture<'static, (ExtensionId, Result<Result<(), Error>, JoinError>)>;

/// Per-extension lifecycle metrics.
///
/// Receivers / processors / exporters do not have an equivalent per-node
/// metric set because their drain progress is surfaced via the
/// pipeline-wide `pipeline.runtime_control` set. Extensions need a
/// dedicated per-instance set because they shut down strictly after the
/// data path drains (see this module's `Shutdown timing` docs) and have
/// their own cooperative grace window — engine-wide drain metrics cannot
/// attribute slowness to a specific extension.
#[metric_set(name = "extension.lifecycle")]
#[derive(Debug, Default, Clone)]
pub struct ExtensionLifecycleMetrics {
    /// `1` while the extension's active task is running, `0` after it
    /// has terminated (either cooperatively or via the bounded drain
    /// timeout).
    #[metric(name = "active", unit = "{1}")]
    pub active: Gauge<u64>,
    /// `1` once `Shutdown` has been successfully delivered to this
    /// extension's control channel, `0` otherwise. Lets observers
    /// gate `shutdown.duration` interpretation on a single boolean
    /// rather than cross-referencing `shutdown.send_failed` /
    /// `shutdown.send_timeout`. Stays `0` if the extension exited
    /// before broadcast or the send failed/timed out.
    #[metric(name = "shutdown.delivered", unit = "{1}")]
    pub shutdown_delivered: Gauge<u64>,
    /// Wall-clock nanoseconds spent in cooperative shutdown for this
    /// extension, measured from the moment `Shutdown` was
    /// successfully delivered to the moment the task completed.
    /// Stamped exactly once at task exit and only when
    /// `shutdown.delivered == 1`. Operators should treat this value
    /// as meaningful only when `shutdown.delivered == 1`; otherwise
    /// it reflects the Gauge default (`0`) and not a measurement.
    #[metric(name = "shutdown.duration", unit = "ns")]
    pub shutdown_duration_ns: Gauge<u64>,
    /// Cumulative count of times this extension failed to exit within
    /// [`EXTENSION_SHUTDOWN_GRACE`] after `Shutdown` was successfully
    /// delivered. Pairs with the `extension.shutdown.timeout` warn
    /// event. Not incremented when `Shutdown` could not be delivered
    /// (see `shutdown.send_failed` / `shutdown.send_timeout`).
    #[metric(name = "shutdown.timeout", unit = "{1}")]
    pub shutdown_timeout: Counter<u64>,
    /// Cumulative count of `Shutdown` sends that failed because the
    /// extension's control channel was closed. Pairs with the
    /// `extension.shutdown.send_failed` warn event.
    #[metric(name = "shutdown.send_failed", unit = "{1}")]
    pub shutdown_send_failed: Counter<u64>,
    /// Cumulative count of `Shutdown` sends that exceeded
    /// [`EXTENSION_SHUTDOWN_SEND_TIMEOUT`]. Pairs with the
    /// `extension.shutdown.send_timeout` warn event.
    #[metric(name = "shutdown.send_timeout", unit = "{1}")]
    pub shutdown_send_timeout: Counter<u64>,
    /// Cumulative count of task errors observed (an `Err(_)` return
    /// from `start()`, including panics caught inside the task
    /// wrapper and surfaced as `Err`). Pairs with
    /// `extension.task.error`, `extension.task.panic`, and
    /// `extension.shutdown.task.error`. Not incremented for task
    /// cancellation (`extension.shutdown.task.join_error`), which is
    /// a runtime event rather than an extension fault.
    #[metric(name = "task.error", unit = "{1}")]
    pub task_error: Counter<u64>,
}

/// Per-extension lifecycle state owned by [`ExtensionLifecycle`].
///
/// Held in a `HashMap` keyed by [`ExtensionId`] so completions reported
/// by the task can be attributed back to the right extension, and so
/// extensions that miss the drain deadline can be ticked as timeouts.
struct ExtensionMetricsEntry {
    metrics: MetricSet<ExtensionLifecycleMetrics>,
    /// Set to `true` once a terminal metric flush has been emitted for
    /// this extension (either via `record_completion` or
    /// `record_timeout`). Prevents double-counting across the normal
    /// and timeout paths.
    finalized: bool,
    /// `Some(instant)` once `Shutdown` has been successfully delivered
    /// to this extension. Anchors per-extension
    /// `shutdown.duration_ns` and gates `shutdown.timeout` so the
    /// runtime never attributes cooperative-shutdown latency or
    /// timeout to an extension that never actually received the
    /// signal. Remains `None` if the extension exited before
    /// broadcast, or if its `Shutdown` send failed / timed out (in
    /// which case `shutdown.send_failed` / `shutdown.send_timeout`
    /// are the authoritative signals).
    shutdown_delivered_at: Option<Instant>,
}

/// Holds the spawned extension tasks, control senders, and passive
/// wrappers for the duration of a pipeline run.
pub(crate) struct ExtensionLifecycle {
    /// Wrapped active+background extension futures, awaited
    /// concurrently with the data path. Each future yields the
    /// extension's id alongside the join result so completions can be
    /// attributed back to the right per-extension metric set even
    /// when the underlying task is cancelled.
    futures: FuturesUnordered<ExtensionFuture>,
    /// Abort handles for the tasks in [`Self::futures`], kept
    /// alongside the wrapped futures (which can no longer call
    /// `JoinHandle::abort` directly). Used by
    /// [`Self::drain_until_deadline`] and `Drop` to force-cancel
    /// extensions that overran the cooperative shutdown budget.
    abort_handles: Vec<AbortHandle>,
    /// Control senders for the extensions in [`Self::futures`], used
    /// once to broadcast `Shutdown` after the data path drains.
    shutdown_senders: Vec<ExtensionControlSender>,
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
    /// misbehaving extension can't hang the pipeline indefinitely.
    shutdown_deadline: Option<Instant>,
    /// Per-extension lifecycle metric sets, plus a finalized flag to
    /// keep the normal and timeout paths from double-counting.
    /// Reporting is inline (after each lifecycle event) — matching the
    /// `ComputeDuration::report` pattern in `process_duration.rs` —
    /// rather than via `CollectTelemetry`, since lifecycle events are
    /// rare and bounded per extension.
    ext_metrics: HashMap<ExtensionId, ExtensionMetricsEntry>,
    /// Extensions whose active task has not yet completed. Updated by
    /// [`Self::next_completion`]; the residue after
    /// [`Self::drain_until_deadline`] returns is exactly the set of
    /// extensions that timed out.
    pending: HashSet<ExtensionId>,
    /// Pipeline-level `MetricsReporter` used to flush per-extension
    /// metric sets inline. Cloned from the `metrics_reporter` passed
    /// to [`Self::spawn`].
    metrics_reporter: MetricsReporter,
    /// Tracks per-extension entities so they can be unregistered when
    /// the lifecycle is dropped, mirroring `PipelineMetricsMonitor::Drop`.
    registered_entities: Vec<EntityKey>,
    /// Registry handle used to unregister metric sets and entities on
    /// drop.
    registry: TelemetryRegistryHandle,
}

impl ExtensionLifecycle {
    /// Spawn all active+background extensions onto `local_tasks` and
    /// stash the passive ones. Active+background extensions begin
    /// running concurrently with the data path; passive extensions
    /// have no lifecycle but must remain owned for their capability
    /// state to remain valid.
    ///
    /// The parent attribute hierarchy for each extension's metrics
    /// entity is decided by the supplied [`ExtensionContext`] — this
    /// code is deliberately blind to whether that scope is pipeline or
    /// engine.
    pub fn spawn(
        extensions: Vec<ExtensionWrapper>,
        local_tasks: &LocalSet,
        metrics_reporter: MetricsReporter,
        extension_context: &ExtensionContext,
    ) -> Self {
        let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
        let mut abort_handles: Vec<AbortHandle> = Vec::new();
        let mut shutdown_senders = Vec::new();
        let mut passive = Vec::new();
        let mut ext_metrics: HashMap<ExtensionId, ExtensionMetricsEntry> = HashMap::new();
        let mut pending: HashSet<ExtensionId> = HashSet::new();
        let mut registered_entities = Vec::new();
        let registry = extension_context.metrics_registry();

        for ext_wrapper in extensions {
            if ext_wrapper.is_passive() {
                passive.push(ext_wrapper);
                continue;
            }
            if let Some(sender) = ext_wrapper.extension_control_sender() {
                shutdown_senders.push(sender);
            }
            let ext_metrics_reporter = metrics_reporter.clone();
            let ext_id = ext_wrapper.name();

            // Register a per-extension entity through the
            // `ExtensionContext` so the parent attribute hierarchy
            // (pipeline today, possibly engine later) stays opaque to
            // this code. Then attach the scope-agnostic lifecycle
            // metric set.
            let entity_key = extension_context.register_extension_entity(&ext_id);
            registered_entities.push(entity_key);
            let mut lifecycle_metrics =
                registry.register_metric_set_for_entity::<ExtensionLifecycleMetrics>(entity_key);
            // Mark the extension active immediately so a snapshot taken
            // before the task is scheduled still observes it.
            lifecycle_metrics.active.set(1);
            let _ = metrics_reporter.clone().report(&mut lifecycle_metrics);
            let _ = ext_metrics.insert(
                ext_id.clone(),
                ExtensionMetricsEntry {
                    metrics: lifecycle_metrics,
                    finalized: false,
                    shutdown_delivered_at: None,
                },
            );
            let _ = pending.insert(ext_id.clone());

            let fut_ext_id = ext_id.clone();
            let fut = async move {
                // Wrap the extension's `start()` in `catch_unwind` so a
                // panic inside user code is attributed to *this*
                // extension's metrics (via `fut_ext_id`) instead of
                // surfacing as an opaque `JoinError(panic)` that we
                // cannot pin to a specific id.
                let outcome = AssertUnwindSafe(ext_wrapper.start(ext_metrics_reporter.clone()))
                    .catch_unwind()
                    .await;
                let result = match outcome {
                    Ok(Ok(terminal_state)) => {
                        crate::runtime_pipeline::report_terminal_metrics(
                            &ext_metrics_reporter,
                            terminal_state,
                        );
                        Ok(())
                    }
                    Ok(Err(e)) => {
                        otel_warn!(
                            "extension.task.error",
                            extension = fut_ext_id.as_ref(),
                            error = format!("{e}"),
                        );
                        Err(e)
                    }
                    Err(panic) => {
                        let msg = panic_payload_to_string(&*panic);
                        otel_warn!(
                            "extension.task.panic",
                            extension = fut_ext_id.as_ref(),
                            error = msg.as_str(),
                        );
                        Err(Error::InternalError {
                            message: format!("extension {fut_ext_id} panicked: {msg}"),
                        })
                    }
                };
                (fut_ext_id, result)
            };
            let handle = local_tasks.spawn_local(fut);
            abort_handles.push(handle.abort_handle());
            // Wrap the join so the yielded tuple always carries the
            // extension id, even on `JoinError` (cancellation). The
            // inner task normally returns its own id; we fall back to
            // the captured `id_for_wrap` only when the join errors.
            let id_for_wrap = ext_id.clone();
            futures.push(Box::pin(async move {
                match handle.await {
                    Ok((id, res)) => (id, Ok(res)),
                    Err(e) => (id_for_wrap, Err(e)),
                }
            }));
        }

        Self {
            futures,
            abort_handles,
            shutdown_senders,
            _passive: passive,
            shutdown_broadcast_fired: false,
            shutdown_deadline: None,
            ext_metrics,
            pending,
            metrics_reporter,
            registered_entities,
            registry,
        }
    }

    /// Returns `true` if there are no remaining active+background
    /// extension tasks to await.
    pub fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Awaits the next active+background extension task to complete.
    /// Returns `None` when no extension tasks remain.
    ///
    /// Side effect: stamps the per-extension `shutdown.duration` and
    /// flips `active` to `0` for whichever extension just finished,
    /// then flushes its metric set inline so observers see the
    /// transition immediately.
    pub async fn next_completion(&mut self) -> Option<Result<Result<(), Error>, JoinError>> {
        let (ext_id, joined) = self.futures.next().await?;
        Some(match joined {
            Ok(result) => {
                self.record_completion(&ext_id, result.is_err());
                Ok(result)
            }
            Err(e) => {
                // Panics are caught inside the task wrapper and
                // attributed to a specific extension, so a `JoinError`
                // here can only mean the task was cancelled by the
                // runtime (e.g. `LocalSet` drop, explicit abort). That
                // is not an extension fault — finalize the entry so
                // it isn't later misattributed as a `shutdown.timeout`,
                // but don't bump `task.error` for innocent siblings.
                self.record_cancellation(&ext_id);
                Err(e)
            }
        })
    }

    /// Broadcasts `Shutdown` to all active+background extensions.
    /// Idempotent. Sends fan out concurrently, each bounded by
    /// [`EXTENSION_SHUTDOWN_SEND_TIMEOUT`]; the deadline carried in
    /// the message is `now() + EXTENSION_SHUTDOWN_GRACE` (a fresh
    /// cleanup window, not a continuation of the pipeline deadline).
    pub async fn broadcast_shutdown(&mut self) {
        if self.shutdown_broadcast_fired || self.shutdown_senders.is_empty() {
            return;
        }
        self.shutdown_broadcast_fired = true;

        let now = Instant::now();
        let deadline = now + EXTENSION_SHUTDOWN_GRACE;
        self.shutdown_deadline = Some(deadline);

        let sends = self.shutdown_senders.iter().map(|sender| {
            let msg = ExtensionControlMsg::Shutdown {
                deadline,
                reason: SHUTDOWN_REASON.to_string(),
            };
            let ext_id = sender.name.clone();
            async move {
                let outcome =
                    tokio::time::timeout(EXTENSION_SHUTDOWN_SEND_TIMEOUT, sender.sender.send(msg))
                        .await;
                (ext_id, outcome)
            }
        });
        let results: Vec<_> = futures::future::join_all(sends).await;
        for (ext_id, outcome) in results {
            match outcome {
                Ok(Ok(())) => {
                    // Anchor `shutdown.duration` on successful
                    // delivery, using a per-send timestamp so the
                    // recorded duration reflects only cooperative
                    // shutdown work — not how long the control
                    // channel queued the message. Also flip
                    // `shutdown.delivered` to 1 so observers can gate
                    // duration interpretation on a single boolean.
                    // If the entry was already finalized (the
                    // extension completed before `broadcast_shutdown`
                    // was even called and `record_completion` ran via
                    // the main select loop), leave it alone — that
                    // earlier finalize is authoritative and the
                    // extension never participated in cooperative
                    // shutdown.
                    let delivered_at = Instant::now();
                    if let Some(entry) = self.ext_metrics.get_mut(&ext_id) {
                        if !entry.finalized {
                            entry.shutdown_delivered_at = Some(delivered_at);
                            entry.metrics.shutdown_delivered.set(1);
                        }
                    }
                }
                Ok(Err(e)) => {
                    otel_warn!(
                        "extension.shutdown.send_failed",
                        extension = ext_id.as_ref(),
                        error = format!("{e}"),
                    );
                    self.record_send_failed(&ext_id);
                }
                Err(_elapsed) => {
                    otel_warn!(
                        "extension.shutdown.send_timeout",
                        extension = ext_id.as_ref(),
                        timeout_ms = EXTENSION_SHUTDOWN_SEND_TIMEOUT.as_millis() as u64,
                    );
                    self.record_send_timeout(&ext_id);
                }
            }
        }
    }

    /// Drain remaining active+background extension tasks, but never
    /// past the shutdown deadline.
    ///
    /// `Shutdown` is cooperative — extensions may ignore it or take
    /// longer than the grace window to exit. Without this bound, an
    /// extension that never returns from `start()` would hang the
    /// pipeline forever. After the deadline elapses, any still-running
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
            while let Some((ext_id, result)) = self.futures.next().await {
                match result {
                    Ok(Ok(())) => {
                        self.record_completion(&ext_id, false);
                    }
                    Ok(Err(e)) => {
                        otel_warn!(
                            "extension.shutdown.task.error",
                            extension = ext_id.as_ref(),
                            error = format!("{e}"),
                        );
                        self.record_completion(&ext_id, true);
                    }
                    Err(e) => {
                        otel_warn!(
                            "extension.shutdown.task.join_error",
                            extension = ext_id.as_ref(),
                            is_canceled = e.is_cancelled(),
                            is_panic = e.is_panic(),
                            error = e.to_string()
                        );
                        // Panics are caught and attributed inside the
                        // task wrapper, so a `JoinError` here can only
                        // be cancellation. Finalize the entry so it
                        // isn't misattributed as a `shutdown.timeout`,
                        // but don't fan `task.error` out to all
                        // pending extensions.
                        self.record_cancellation(&ext_id);
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
            // Abort surviving tasks BEFORE recording the timeout so the
            // `active=0` gauge we're about to publish matches reality:
            // once we mark the extension timed out, the runtime has
            // already cancelled its task and it will not run again.
            // `AbortHandle::abort` is best-effort cancellation, but it
            // guarantees the future will not be polled further.
            for handle in &self.abort_handles {
                handle.abort();
            }
            // Anything still in `pending` after the bounded drain
            // failed to honour `Shutdown` within the grace window.
            // Tick the per-extension `shutdown.timeout` counter so
            // operators can isolate the misbehaving extension(s).
            for ext_id in self.pending.clone() {
                self.record_timeout(&ext_id);
            }
        }
    }

    /// Stamp `shutdown.duration` (only if `Shutdown` was successfully
    /// delivered), flip `active` to 0, bump `task_error` on failure,
    /// then flush.
    fn record_completion(&mut self, ext_id: &ExtensionId, is_error: bool) {
        let _ = self.pending.remove(ext_id);
        let mut reporter = self.metrics_reporter.clone();
        let Some(entry) = self.ext_metrics.get_mut(ext_id) else {
            return;
        };
        if entry.finalized {
            return;
        }
        entry.finalized = true;
        if let Some(started) = entry.shutdown_delivered_at {
            let elapsed = started.elapsed().as_nanos() as u64;
            entry.metrics.shutdown_duration_ns.set(elapsed);
        }
        entry.metrics.active.set(0);
        if is_error {
            entry.metrics.task_error.inc();
        }
        let _ = reporter.report(&mut entry.metrics);
    }

    /// Record that this extension blew past the drain deadline.
    /// Only increments `shutdown.timeout` when `Shutdown` was
    /// actually delivered — if delivery failed, the failure is
    /// already attributed via `shutdown.send_failed` /
    /// `shutdown.send_timeout` and counting a timeout on top would
    /// double-attribute the same underlying event.
    fn record_timeout(&mut self, ext_id: &ExtensionId) {
        let mut reporter = self.metrics_reporter.clone();
        let Some(entry) = self.ext_metrics.get_mut(ext_id) else {
            return;
        };
        if entry.finalized {
            return;
        }
        entry.finalized = true;
        if entry.shutdown_delivered_at.is_some() {
            entry.metrics.shutdown_timeout.inc();
        }
        entry.metrics.active.set(0);
        let _ = reporter.report(&mut entry.metrics);
    }

    fn record_send_failed(&mut self, ext_id: &ExtensionId) {
        let mut reporter = self.metrics_reporter.clone();
        let Some(entry) = self.ext_metrics.get_mut(ext_id) else {
            return;
        };
        entry.metrics.shutdown_send_failed.inc();
        let _ = reporter.report(&mut entry.metrics);
    }

    fn record_send_timeout(&mut self, ext_id: &ExtensionId) {
        let mut reporter = self.metrics_reporter.clone();
        let Some(entry) = self.ext_metrics.get_mut(ext_id) else {
            return;
        };
        entry.metrics.shutdown_send_timeout.inc();
        let _ = reporter.report(&mut entry.metrics);
    }

    /// Finalize an extension whose task was cancelled by the runtime
    /// (`JoinError`). Flushes `active=0`, removes from `pending` (so
    /// it isn't later misattributed as a `shutdown.timeout`), and
    /// marks the entry finalized. Does NOT bump `task.error` or
    /// `shutdown.timeout` — cancellation is a runtime event, not an
    /// extension fault. Idempotent via the `finalized` flag.
    fn record_cancellation(&mut self, ext_id: &ExtensionId) {
        let _ = self.pending.remove(ext_id);
        let mut reporter = self.metrics_reporter.clone();
        let Some(entry) = self.ext_metrics.get_mut(ext_id) else {
            return;
        };
        if entry.finalized {
            return;
        }
        entry.finalized = true;
        entry.metrics.active.set(0);
        let _ = reporter.report(&mut entry.metrics);
    }
}

/// Best-effort extraction of a human-readable message from a panic
/// payload caught via [`FutureExt::catch_unwind`]. Panics commonly
/// carry `&'static str` or `String`; anything else degrades to a
/// fixed placeholder so we never lose the panic event itself.
fn panic_payload_to_string(payload: &(dyn std::any::Any + Send)) -> String {
    if let Some(s) = payload.downcast_ref::<&'static str>() {
        (*s).to_string()
    } else if let Some(s) = payload.downcast_ref::<String>() {
        s.clone()
    } else {
        "<non-string panic payload>".to_string()
    }
}

impl Drop for ExtensionLifecycle {
    fn drop(&mut self) {
        // `drain_until_deadline` deliberately returns with surviving
        // futures when its bound fires. Abort those tasks BEFORE
        // unregistering their metric entities so a still-running task
        // cannot race the registry tear-down and report into a freed
        // slot. `AbortHandle::abort` is best-effort cancellation, but
        // it guarantees the user future will not be polled again.
        for handle in &self.abort_handles {
            handle.abort();
        }
        // Defensive: flip `active` to 0 for any extension that never
        // reached a terminal recorder (`record_completion` /
        // `record_timeout`). This only happens when the lifecycle is
        // dropped without a prior `drain_until_deadline`; the normal
        // shutdown path already finalized every entry. Without this,
        // a scraper observing the registry between abort and
        // unregister would see `active=1` for tasks the runtime has
        // just cancelled.
        let mut reporter = self.metrics_reporter.clone();
        for entry in self.ext_metrics.values_mut() {
            if !entry.finalized {
                entry.finalized = true;
                entry.metrics.active.set(0);
                let _ = reporter.report(&mut entry.metrics);
            }
        }
        // Unregister per-extension metric sets and entities so a
        // dropped lifecycle doesn't leak registry slots, mirroring
        // `PipelineMetricsMonitor::Drop`.
        for entry in self.ext_metrics.values() {
            let _ = self
                .registry
                .unregister_metric_set(entry.metrics.metric_set_key());
        }
        for key in self.registered_entities.drain(..) {
            let _ = self.registry.unregister_entity(key);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::context::{ControllerContext, PipelineContext};
    use otap_df_telemetry::registry::TelemetryRegistryHandle;
    use otap_df_telemetry::reporter::MetricsReporter;
    use std::borrow::Cow;
    use std::future::Future;

    /// Push a wrapped extension future + abort handle in the same
    /// shape `ExtensionLifecycle::spawn` produces, so tests exercise
    /// the real wrapping (id-preserving on `JoinError`) instead of
    /// hand-rolling it at every call site.
    fn push_test_extension<F>(
        futures: &mut FuturesUnordered<ExtensionFuture>,
        abort_handles: &mut Vec<AbortHandle>,
        ext_id: ExtensionId,
        fut: F,
    ) where
        F: Future<Output = (ExtensionId, Result<(), Error>)> + 'static,
    {
        let handle = tokio::task::spawn_local(fut);
        abort_handles.push(handle.abort_handle());
        let id_for_wrap = ext_id;
        futures.push(Box::pin(async move {
            match handle.await {
                Ok((id, res)) => (id, Ok(res)),
                Err(e) => (id_for_wrap, Err(e)),
            }
        }));
    }

    fn make_pipeline_context() -> PipelineContext {
        let registry = TelemetryRegistryHandle::new();
        ControllerContext::new(registry).pipeline_context_with("g".into(), "p".into(), 0, 1, 0)
    }

    fn empty_metrics_reporter() -> MetricsReporter {
        let (_rx, reporter) = MetricsReporter::create_new_and_receiver(64);
        reporter
    }

    fn make_test_lifecycle(
        futures: FuturesUnordered<ExtensionFuture>,
        abort_handles: Vec<AbortHandle>,
        ext_ids: &[&'static str],
    ) -> ExtensionLifecycle {
        let pipeline_ctx = make_pipeline_context();
        let ext_ctx = ExtensionContext::from_pipeline(&pipeline_ctx);
        let registry = ext_ctx.metrics_registry();
        let mut ext_metrics: HashMap<ExtensionId, ExtensionMetricsEntry> = HashMap::new();
        let mut pending: HashSet<ExtensionId> = HashSet::new();
        let mut registered_entities = Vec::new();
        for id in ext_ids {
            let ext_id: ExtensionId = Cow::Borrowed(id);
            let entity_key = ext_ctx.register_extension_entity(&ext_id);
            registered_entities.push(entity_key);
            let mut metrics =
                registry.register_metric_set_for_entity::<ExtensionLifecycleMetrics>(entity_key);
            // Mirror what `spawn` and `broadcast_shutdown` do on
            // successful delivery so tests exercise the post-delivery
            // code paths by default and `active` transitions from
            // `1` to `0` on completion/timeout instead of starting at
            // its default `0`. Tests that need the pre-delivery
            // scenario can clear these.
            metrics.active.set(1);
            metrics.shutdown_delivered.set(1);
            let _ = ext_metrics.insert(
                ext_id.clone(),
                ExtensionMetricsEntry {
                    metrics,
                    finalized: false,
                    shutdown_delivered_at: Some(Instant::now()),
                },
            );
            let _ = pending.insert(ext_id);
        }
        ExtensionLifecycle {
            futures,
            abort_handles,
            shutdown_senders: Vec::new(),
            _passive: Vec::new(),
            shutdown_broadcast_fired: true,
            shutdown_deadline: None,
            ext_metrics,
            pending,
            metrics_reporter: empty_metrics_reporter(),
            registered_entities,
            registry,
        }
    }

    /// Regression test: a misbehaving extension that never returns
    /// must not stall `drain_until_deadline` past its deadline. The
    /// deadline is injected directly to keep the test fast.
    #[test]
    fn drain_until_deadline_is_bounded_for_stuck_extension() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("stuck"),
                async {
                    // Misbehaving extension that ignores `Shutdown` and
                    // never returns from `start()`. `pending` is cancelled
                    // when the surrounding `LocalSet` drops at the end of
                    // the test, so this does not actually run forever.
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("stuck"), Ok(()))
                },
            );

            let injected_deadline = Instant::now() + Duration::from_millis(100);
            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &["stuck"]);
            lifecycle.shutdown_deadline = Some(injected_deadline);

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

            // The stuck extension should have its per-ext
            // `shutdown.timeout` counter ticked and `active` flipped
            // to `0` so operators can isolate it.
            let entry = lifecycle
                .ext_metrics
                .get(&Cow::Borrowed("stuck"))
                .expect("metric entry exists");
            assert_eq!(entry.metrics.shutdown_timeout.get(), 1);
            assert_eq!(entry.metrics.active.get(), 0);
        }));
    }

    /// A normally-completing extension drained via `next_completion`
    /// gets `active = 0` and a non-zero `shutdown_duration_ns` (since
    /// we injected a `shutdown_started_at`).
    #[test]
    fn next_completion_records_per_extension_completion() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("clean"),
                async {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    (Cow::Borrowed("clean"), Ok(()))
                },
            );

            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &["clean"]);
            // Backdate the broadcast so elapsed > 0 deterministically.
            if let Some(entry) = lifecycle.ext_metrics.get_mut(&Cow::Borrowed("clean")) {
                entry.shutdown_delivered_at = Some(Instant::now() - Duration::from_millis(1));
            }

            let result = lifecycle
                .next_completion()
                .await
                .expect("one extension queued");
            assert!(matches!(result, Ok(Ok(()))));

            let entry = lifecycle
                .ext_metrics
                .get(&Cow::Borrowed("clean"))
                .expect("metric entry exists");
            assert_eq!(entry.metrics.active.get(), 0);
            assert_eq!(entry.metrics.task_error.get(), 0);
            assert!(
                entry.metrics.shutdown_duration_ns.get() > 0,
                "shutdown_duration_ns should be stamped on completion",
            );
            assert_eq!(
                entry.metrics.shutdown_delivered.get(),
                1,
                "shutdown.delivered should remain 1 across terminal flush",
            );
            assert!(!lifecycle.pending.contains(&Cow::Borrowed("clean")));
        }));
    }

    /// An extension whose task returns `Err(_)` gets `task_error`
    /// ticked alongside the normal completion bookkeeping.
    #[test]
    fn next_completion_records_task_error_on_err() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("boom"),
                async {
                    (
                        Cow::Borrowed("boom"),
                        Err(Error::InternalError {
                            message: "boom".to_string(),
                        }),
                    )
                },
            );

            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &["boom"]);

            let result = lifecycle
                .next_completion()
                .await
                .expect("one extension queued");
            assert!(matches!(result, Ok(Err(_))));

            let entry = lifecycle
                .ext_metrics
                .get(&Cow::Borrowed("boom"))
                .expect("metric entry exists");
            assert_eq!(entry.metrics.active.get(), 0);
            assert_eq!(entry.metrics.task_error.get(), 1);
        }));
    }

    /// Regression: an extension whose `Shutdown` was never delivered
    /// (send failed / timed out) must NOT have `shutdown.duration` or
    /// `shutdown.timeout` recorded — those signals are reserved for
    /// extensions that actually received the cooperative shutdown
    /// request. The send-side counters are the authoritative signal
    /// for delivery failure.
    #[test]
    fn record_completion_skips_duration_when_shutdown_not_delivered() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("undelivered"),
                async { (Cow::Borrowed("undelivered"), Ok(())) },
            );
            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &["undelivered"]);
            // Simulate the "send failed / timed out" outcome: no
            // delivery anchor and `shutdown.delivered` stays 0.
            if let Some(entry) = lifecycle.ext_metrics.get_mut(&Cow::Borrowed("undelivered")) {
                entry.shutdown_delivered_at = None;
                entry.metrics.shutdown_delivered.set(0);
            }

            let _ = lifecycle.next_completion().await.expect("queued");

            let entry = lifecycle
                .ext_metrics
                .get(&Cow::Borrowed("undelivered"))
                .expect("metric entry exists");
            assert_eq!(
                entry.metrics.shutdown_duration_ns.get(),
                0,
                "duration must not be stamped when Shutdown was never delivered",
            );
            assert_eq!(
                entry.metrics.shutdown_delivered.get(),
                0,
                "shutdown.delivered must stay 0 when Shutdown was never delivered",
            );
            assert_eq!(entry.metrics.active.get(), 0);
        }));
    }

    /// Regression: `record_timeout` must not increment
    /// `shutdown.timeout` for an extension whose `Shutdown` was never
    /// delivered — that would double-attribute the same underlying
    /// failure already counted as `shutdown.send_failed` or
    /// `shutdown.send_timeout`.
    #[test]
    fn record_timeout_skips_increment_when_shutdown_not_delivered() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("stuck-no-delivery"),
                async {
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("stuck-no-delivery"), Ok(()))
                },
            );

            let injected_deadline = Instant::now() + Duration::from_millis(50);
            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &["stuck-no-delivery"]);
            lifecycle.shutdown_deadline = Some(injected_deadline);
            if let Some(entry) = lifecycle
                .ext_metrics
                .get_mut(&Cow::Borrowed("stuck-no-delivery"))
            {
                entry.shutdown_delivered_at = None;
                entry.metrics.shutdown_delivered.set(0);
            }

            lifecycle.drain_until_deadline().await;

            let entry = lifecycle
                .ext_metrics
                .get(&Cow::Borrowed("stuck-no-delivery"))
                .expect("metric entry exists");
            assert_eq!(
                entry.metrics.shutdown_timeout.get(),
                0,
                "timeout must not be ticked when Shutdown was never delivered",
            );
            assert_eq!(entry.metrics.active.get(), 0);
        }));
    }

    /// Regression: a panic inside the extension's `start()` body is
    /// caught by the task wrapper and surfaced as a per-extension
    /// `task.error` increment with the panic message attributed to
    /// the correct extension id — instead of a `JoinError(panic)`
    /// that would force us to fan out the counter to innocents.
    #[test]
    fn panic_in_extension_is_attributed_via_task_error() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("panicker"),
                async {
                    let payload = AssertUnwindSafe(async { panic!("boom from extension") })
                        .catch_unwind()
                        .await;
                    let result = match payload {
                        Ok(()) => Ok(()),
                        Err(p) => Err(Error::InternalError {
                            message: panic_payload_to_string(&*p),
                        }),
                    };
                    (Cow::Borrowed("panicker"), result)
                },
            );

            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &["panicker"]);
            let result = lifecycle.next_completion().await.expect("queued");
            assert!(matches!(result, Ok(Err(_))));

            let entry = lifecycle
                .ext_metrics
                .get(&Cow::Borrowed("panicker"))
                .expect("metric entry exists");
            assert_eq!(entry.metrics.task_error.get(), 1);
            assert_eq!(entry.metrics.active.get(), 0);
        }));
    }

    /// Regression: a task cancelled by the runtime (e.g. via
    /// `AbortHandle::abort`) yields a `JoinError`, but the per-extension
    /// entry must still be finalized — `active` flipped to `0`, removed
    /// from `pending` — so a later `drain_until_deadline` does NOT
    /// misattribute the cancellation as a `shutdown.timeout`, and the
    /// `task.error` counter is NOT bumped (cancellation is a runtime
    /// event, not an extension fault).
    #[test]
    fn next_completion_finalizes_entry_on_cancellation() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("cancelled"),
                async {
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("cancelled"), Ok(()))
                },
            );

            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &["cancelled"]);
            // Force a `JoinError` by aborting the spawned task before
            // awaiting the wrapped future.
            for handle in &lifecycle.abort_handles {
                handle.abort();
            }

            let result = lifecycle.next_completion().await.expect("queued");
            assert!(
                matches!(&result, Err(e) if e.is_cancelled()),
                "cancellation must surface as Err(JoinError::cancelled), got {result:?}",
            );

            assert!(
                !lifecycle.pending.contains(&Cow::Borrowed("cancelled")),
                "cancelled extension must be removed from `pending` so a later \
                 `drain_until_deadline` does not misattribute it as a timeout",
            );
            let entry = lifecycle
                .ext_metrics
                .get(&Cow::Borrowed("cancelled"))
                .expect("metric entry exists");
            assert_eq!(entry.metrics.active.get(), 0);
            assert_eq!(
                entry.metrics.task_error.get(),
                0,
                "cancellation is a runtime event, not an extension fault",
            );
            assert_eq!(
                entry.metrics.shutdown_timeout.get(),
                0,
                "cancellation must not be misattributed as a shutdown timeout",
            );
        }));
    }
}
