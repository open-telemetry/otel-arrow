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

use crate::control::{ExtensionControlMsg, ExtensionControlSender};
use crate::error::Error;
use crate::extension::ExtensionWrapper;
use futures::stream::{FuturesUnordered, StreamExt};
use otap_df_telemetry::otel_warn;
use otap_df_telemetry::reporter::MetricsReporter;
use std::time::{Duration, Instant};
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

/// Holds the spawned extension tasks, control senders, and passive
/// wrappers for the duration of an extension-hosting run.
pub(crate) struct ExtensionLifecycle {
    /// Active+background extension `JoinHandle`s, awaited concurrently
    /// with the data path.
    futures: FuturesUnordered<JoinHandle<Result<(), Error>>>,
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
    /// misbehaving extension can't hang the host runtime indefinitely.
    shutdown_deadline: Option<Instant>,
}

impl ExtensionLifecycle {
    /// Spawn all active+background extensions onto `local_tasks` and
    /// stash the passive ones. Active+background extensions begin
    /// running concurrently with the data path; passive extensions
    /// have no lifecycle but must remain owned for their capability
    /// state to remain valid.
    pub fn spawn(
        extensions: Vec<ExtensionWrapper>,
        local_tasks: &LocalSet,
        metrics_reporter: MetricsReporter,
    ) -> Self {
        let futures = FuturesUnordered::new();
        let mut shutdown_senders = Vec::new();
        let mut passive = Vec::new();

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
            let fut = async move {
                match ext_wrapper.start(ext_metrics_reporter.clone()).await {
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
                            extension = ext_id.as_ref(),
                            error = format!("{e}"),
                        );
                        Err(e)
                    }
                }
            };
            futures.push(local_tasks.spawn_local(fut));
        }

        Self {
            futures,
            shutdown_senders,
            _passive: passive,
            shutdown_broadcast_fired: false,
            shutdown_deadline: None,
        }
    }

    /// Returns `true` if there are no remaining active+background
    /// extension tasks to await.
    pub fn is_empty(&self) -> bool {
        self.futures.is_empty()
    }

    /// Awaits the next active+background extension task to complete.
    /// Returns `None` when no extension tasks remain.
    pub async fn next_completion(&mut self) -> Option<Result<Result<(), Error>, JoinError>> {
        self.futures.next().await
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
        let sends = self.shutdown_senders.iter().map(|sender| {
            let msg = ExtensionControlMsg::Shutdown {
                deadline,
                reason: reason.clone(),
            };
            async move {
                match tokio::time::timeout(EXTENSION_SHUTDOWN_SEND_TIMEOUT, sender.sender.send(msg))
                    .await
                {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        otel_warn!(
                            "extension.shutdown.send_failed",
                            extension = sender.name.as_ref(),
                            error = format!("{e}"),
                        );
                    }
                    Err(_elapsed) => {
                        otel_warn!(
                            "extension.shutdown.send_timeout",
                            extension = sender.name.as_ref(),
                            timeout_ms = EXTENSION_SHUTDOWN_SEND_TIMEOUT.as_millis() as u64,
                        );
                    }
                }
            }
        });
        let _: Vec<()> = futures::future::join_all(sends).await;
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
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        otel_warn!("extension.shutdown.task.error", error = format!("{e}"));
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
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Regression test: a misbehaving extension that never returns
    /// must not stall `drain_until_deadline` past its deadline. The
    /// deadline is injected directly to keep the test fast.
    #[test]
    fn drain_until_deadline_is_bounded_for_stuck_extension() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures = FuturesUnordered::new();
            futures.push(tokio::task::spawn_local(async {
                // Misbehaving extension that ignores `Shutdown` and
                // never returns from `start()`. `pending` is cancelled
                // when the surrounding `LocalSet` drops at the end of
                // the test, so this does not actually run forever.
                std::future::pending::<()>().await;
                Ok(())
            }));

            let injected_deadline = Instant::now() + Duration::from_millis(100);
            let mut lifecycle = ExtensionLifecycle {
                futures,
                shutdown_senders: Vec::new(),
                _passive: Vec::new(),
                shutdown_broadcast_fired: true,
                shutdown_deadline: Some(injected_deadline),
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
