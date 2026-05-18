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

const SHUTDOWN_REASON: &str = "pipeline data-path drained";

/// Holds the spawned extension tasks, control senders, and passive
/// wrappers for the duration of a pipeline run.
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
    /// misbehaving extension can't hang the pipeline indefinitely.
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
    /// Idempotent — subsequent calls are no-ops.
    ///
    /// The deadline is computed locally as
    /// `now() + EXTENSION_SHUTDOWN_GRACE`. Extensions are expected
    /// to be invoked only after every data-path task has terminated,
    /// so this is the start of the extension cleanup window — not a
    /// continuation of the pipeline-wide deadline.
    pub fn broadcast_shutdown(&mut self) {
        if self.shutdown_broadcast_fired || self.shutdown_senders.is_empty() {
            return;
        }
        self.shutdown_broadcast_fired = true;

        let deadline = Instant::now() + EXTENSION_SHUTDOWN_GRACE;
        self.shutdown_deadline = Some(deadline);
        for sender in &self.shutdown_senders {
            // Non-blocking: data path has already drained so the
            // small mpsc should have room. Warn on the rare failure
            // (extension backed up or already torn down) rather than
            // silently dropping the cleanup signal.
            if let Err(e) = sender.sender.try_send(ExtensionControlMsg::Shutdown {
                deadline,
                reason: SHUTDOWN_REASON.to_string(),
            }) {
                otel_warn!(
                    "extension.shutdown.send_failed",
                    extension = sender.name.as_ref(),
                    error = format!("{e}"),
                );
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
