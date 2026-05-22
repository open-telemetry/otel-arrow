// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extension lifecycle holder for the runtime pipeline.
//!
//! Owns the spawned active+background extension tasks, the control
//! senders used to broadcast `Shutdown`, and the passive extension
//! wrappers that must outlive the run. Encapsulates the
//! "extensions start first, shut down last" invariant.
//!
//! Extensions shut down strictly after all data-path tasks have
//! terminated, so the extension shutdown deadline is computed
//! locally as `now() + EXTENSION_SHUTDOWN_GRACE` rather than
//! reusing the pipeline-wide data-path deadline.

use crate::control::{ExtensionControlMsg, ExtensionControlSender};
use crate::error::Error;
use crate::extension::ExtensionContext;
use crate::extension::ExtensionVariant;
use crate::extension::ExtensionWrapper;
use crate::extension_lifecycle_metrics::{ExtensionLifecycleMetrics, ExtensionMetricsEntry};
use futures::FutureExt;
use futures::future::LocalBoxFuture;
use futures::stream::{FuturesUnordered, StreamExt};
use otap_df_config::ExtensionId;
use otap_df_telemetry::otel_warn;
use otap_df_telemetry::registry::{EntityKey, TelemetryRegistryHandle};
use otap_df_telemetry::reporter::MetricsReporter;
use std::collections::{HashMap, HashSet};
use std::panic::AssertUnwindSafe;
use std::time::{Duration, Instant};
use tokio::task::{AbortHandle, JoinError, LocalSet};
use tokio::time::Instant as TokioInstant;

/// Cleanup window granted to extensions after the data path drains.
pub(crate) const EXTENSION_SHUTDOWN_GRACE: Duration = Duration::from_secs(5);

/// Slack past `EXTENSION_SHUTDOWN_GRACE` before the runtime
/// hard-stops draining, to absorb context-switch / join-poll
/// latency for extensions that finish right at the deadline.
pub(crate) const EXTENSION_SHUTDOWN_DRAIN_SLACK: Duration = Duration::from_millis(500);

/// Per-extension upper bound on how long `broadcast_shutdown` will
/// wait to enqueue `Shutdown`, so one backed-up extension can't
/// stall delivery to the others.
pub(crate) const EXTENSION_SHUTDOWN_SEND_TIMEOUT: Duration = Duration::from_millis(500);

const SHUTDOWN_REASON: &str = "pipeline data-path drained";

/// Composite key for per-lifecycle state.
///
/// The engine treats local and shared variants of one extension id
/// as a single extension (they share an entity row in the
/// registry), but each variant runs its own task with its own
/// lifecycle — so per-lifecycle state must be keyed `(id, variant)`
/// to keep their tasks, senders, and metric sets from colliding.
type ExtensionKey = (ExtensionId, ExtensionVariant);

/// Wrapped extension task future used in [`ExtensionLifecycle::futures`].
///
/// Each spawned `JoinHandle` is wrapped so the yielded tuple always
/// carries the [`ExtensionKey`] — even on `JoinError` from
/// cancellation — so the per-extension entry never gets stranded.
type ExtensionFuture =
    LocalBoxFuture<'static, (ExtensionKey, Result<Result<(), Error>, JoinError>)>;

/// Per-extension outcome of one `Shutdown` send attempt in
/// [`ExtensionLifecycle::broadcast_shutdown`]. Encoded as a sum
/// type so the join consumer pattern-matches a single value
/// instead of a `(Result<Result<(), _>, _>, Option<Instant>)` tuple
/// where only one shape (success → `Some(at)`) is valid — letting
/// the type system enforce what was previously a tuple-level
/// invariant.
enum BroadcastSendResult {
    /// `Shutdown` was enqueued; carries the instant the send completed.
    Sent(Instant),
    /// Send did not reach the extension's control channel — either
    /// the channel was closed (receiver dropped) or the send did
    /// not complete before the shutdown grace deadline. Carries a
    /// human-readable reason for the warn log.
    SendFailed(String),
}

/// Holds the spawned extension tasks, control senders, and passive
/// wrappers for the duration of a pipeline run.
pub(crate) struct ExtensionLifecycle {
    /// Wrapped active+background extension futures. Each yields the
    /// [`ExtensionKey`] alongside the join result so completions can
    /// always be attributed, even on cancellation.
    futures: FuturesUnordered<ExtensionFuture>,
    /// Abort handles paired with [`Self::futures`], used by
    /// [`Self::drain_until_deadline`] and `Drop` to force-cancel
    /// extensions that overran the cooperative shutdown budget.
    abort_handles: Vec<AbortHandle>,
    /// Control senders used once to broadcast `Shutdown`. Paired
    /// with the variant so the per-send metric lookup uses the same
    /// composite key as [`Self::ext_metrics`].
    shutdown_senders: Vec<(ExtensionVariant, ExtensionControlSender)>,
    /// Passive extensions held alive for the run so any state their
    /// capability handles reference survives until drop.
    _passive: Vec<ExtensionWrapper>,
    /// One-shot latch: `true` after `Shutdown` has been broadcast.
    shutdown_broadcast_fired: bool,
    /// Deadline established by [`Self::broadcast_shutdown`]; bounds
    /// [`Self::drain_until_deadline`].
    shutdown_deadline: Option<Instant>,
    /// Per-extension lifecycle state. Keyed `(id, variant)` so each
    /// running lifecycle is tracked independently even though the
    /// engine treats both variants as the same extension.
    ext_metrics: HashMap<ExtensionKey, ExtensionMetricsEntry>,
    /// Extensions whose active task has not yet completed. Residue
    /// after [`Self::drain_until_deadline`] is the timed-out set.
    pending: HashSet<ExtensionKey>,
    /// Per-extension entities, unregistered on drop.
    registered_entities: Vec<EntityKey>,
    /// Registry handle used to publish per-extension metric set
    /// snapshots directly (no channel hop — lifecycle events are
    /// rare enough that the registry mutex is uncontested) and to
    /// unregister entities on drop.
    registry: TelemetryRegistryHandle,
}

impl ExtensionLifecycle {
    /// Spawn all active+background extensions onto `local_tasks` and
    /// stash the passive ones. The parent attribute hierarchy for
    /// each extension's metrics entity is decided by the supplied
    /// [`ExtensionContext`].
    pub fn spawn(
        extensions: Vec<ExtensionWrapper>,
        local_tasks: &LocalSet,
        metrics_reporter: MetricsReporter,
        extension_context: &ExtensionContext,
    ) -> Self {
        let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
        let mut abort_handles: Vec<AbortHandle> = Vec::new();
        let mut shutdown_senders: Vec<(ExtensionVariant, ExtensionControlSender)> = Vec::new();
        let mut passive = Vec::new();
        let mut ext_metrics: HashMap<ExtensionKey, ExtensionMetricsEntry> = HashMap::new();
        let mut pending: HashSet<ExtensionKey> = HashSet::new();
        let mut registered_entities = Vec::new();
        let registry = extension_context.metrics_registry();

        for ext_wrapper in extensions {
            if ext_wrapper.is_passive() {
                passive.push(ext_wrapper);
                continue;
            }
            let variant = ext_wrapper.variant();
            if let Some(sender) = ext_wrapper.extension_control_sender() {
                shutdown_senders.push((variant, sender));
            }
            let ext_metrics_reporter = metrics_reporter.clone();
            let ext_id = ext_wrapper.name();
            let key: ExtensionKey = (ext_id.clone(), variant);

            // Register a per-lifecycle entity keyed by `(id, variant)`
            // so local and shared variants are tracked as distinct
            // entities everywhere — registry, scrapers, and the
            // per-lifecycle metric set below.
            let entity_key = extension_context.register_extension_entity(&ext_id, variant);
            registered_entities.push(entity_key);
            let mut entry = ExtensionMetricsEntry::new(
                registry.register_metric_set_for_entity::<ExtensionLifecycleMetrics>(entity_key),
            );
            // Publish initial `active = 1` so a scrape taken before
            // the task is scheduled still observes running state.
            entry.publish(&registry);
            let _ = ext_metrics.insert(key.clone(), entry);
            let _ = pending.insert(key.clone());

            let fut_ext_id = ext_id.clone();
            let fut = async move {
                // `catch_unwind` so a panic in user code is attributed
                // to *this* extension instead of surfacing as an
                // opaque `JoinError(panic)`.
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
                            extension = format!("{fut_ext_id}/{}", variant.as_str()),
                            error = format!("{e}"),
                        );
                        Err(e)
                    }
                    Err(panic) => {
                        let msg = panic_payload_to_string(&*panic);
                        otel_warn!(
                            "extension.task.panic",
                            extension = format!("{fut_ext_id}/{}", variant.as_str()),
                            error = msg.as_str(),
                        );
                        Err(Error::InternalError {
                            message: format!(
                                "extension {fut_ext_id}/{} panicked: {msg}",
                                variant.as_str()
                            ),
                        })
                    }
                };
                (fut_ext_id, result)
            };
            let handle = local_tasks.spawn_local(fut);
            abort_handles.push(handle.abort_handle());
            // Wrap the join so the yielded tuple always carries the
            // composite key, even on `JoinError` (cancellation).
            let key_for_wrap = key.clone();
            futures.push(Box::pin(async move {
                match handle.await {
                    Ok((id, res)) => ((id, variant), Ok(res)),
                    Err(e) => (key_for_wrap, Err(e)),
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
    /// Returns `None` when no tasks remain. Stamps
    /// `shutdown.duration` and flips `active` to `0` for the
    /// completing extension, then flushes inline.
    pub async fn next_completion(&mut self) -> Option<Result<Result<(), Error>, JoinError>> {
        let (key, joined) = self.futures.next().await?;
        Some(match joined {
            Ok(result) => {
                self.record_completion(&key, result.is_err());
                Ok(result)
            }
            Err(e) => {
                // Panics are caught inside the task wrapper, so a
                // `JoinError` here can only be runtime cancellation.
                // Finalize so it isn't later misattributed as a
                // `shutdown.timeout`, but don't bump `task.error`.
                self.record_cancellation(&key);
                Err(e)
            }
        })
    }

    /// Broadcasts `Shutdown` to all active+background extensions.
    /// Idempotent. Sends fan out concurrently, each bounded by
    /// [`EXTENSION_SHUTDOWN_SEND_TIMEOUT`]; the message carries
    /// `now() + EXTENSION_SHUTDOWN_GRACE` as the cooperative deadline.
    pub async fn broadcast_shutdown(&mut self) {
        if self.shutdown_broadcast_fired || self.shutdown_senders.is_empty() {
            return;
        }
        self.shutdown_broadcast_fired = true;

        let now = Instant::now();
        let deadline = now + EXTENSION_SHUTDOWN_GRACE;
        self.shutdown_deadline = Some(deadline);

        // Capture `Instant::now()` *inside* each per-send future so
        // each extension's `shutdown.duration` anchor reflects when
        // its OWN send completed — capturing after `join_all`
        // returned would let a slow send for A skew B's duration.
        let sends = self
            .shutdown_senders
            .iter()
            .filter_map(|(variant, sender)| {
                let key: ExtensionKey = (sender.name.clone(), *variant);
                // Skip variants whose lifecycle entry already finalized
                // (extension errored or panicked before broadcast).
                // Their receiver was dropped when the task exited, so
                // sending now would attribute a spurious
                // `shutdown.send_failed` on top of the earlier
                // `task.error` — confusing telemetry for an extension
                // that never had a chance to handle `Shutdown`.
                if self.ext_metrics.get(&key).is_some_and(|e| e.is_finalized()) {
                    return None;
                }
                let msg = ExtensionControlMsg::Shutdown {
                    deadline,
                    reason: SHUTDOWN_REASON.to_string(),
                };
                Some(async move {
                    let outcome = tokio::time::timeout(
                        EXTENSION_SHUTDOWN_SEND_TIMEOUT,
                        sender.sender.send(msg),
                    )
                    .await;
                    let result = match outcome {
                        Ok(Ok(())) => BroadcastSendResult::Sent(Instant::now()),
                        Ok(Err(e)) => BroadcastSendResult::SendFailed(format!("{e}")),
                        Err(_elapsed) => BroadcastSendResult::SendFailed(format!(
                            "send did not complete within {}ms",
                            EXTENSION_SHUTDOWN_SEND_TIMEOUT.as_millis(),
                        )),
                    };
                    (key, result)
                })
            });
        let results: Vec<_> = futures::future::join_all(sends).await;
        for (key, result) in results {
            match result {
                BroadcastSendResult::Sent(at) => {
                    // If the entry was already finalized (extension
                    // completed before broadcast), `mark_shutdown_sent`
                    // is a no-op: that earlier finalize is authoritative.
                    if let Some(entry) = self.ext_metrics.get_mut(&key) {
                        entry.mark_shutdown_sent(at);
                    }
                }
                BroadcastSendResult::SendFailed(err) => {
                    otel_warn!(
                        "extension.shutdown.send_failed",
                        extension = format!("{}/{}", key.0.as_ref(), key.1.as_str()),
                        error = err,
                    );
                    self.record_send_failed(&key);
                }
            }
        }
    }

    /// Drain remaining tasks, bounded by the shutdown deadline.
    /// `Shutdown` is cooperative — without this bound, a hung
    /// extension would hang the pipeline forever. No-op if no
    /// futures remain.
    pub async fn drain_until_deadline(&mut self) {
        if self.futures.is_empty() {
            return;
        }
        // If drain is called without a prior broadcast, synthesize a
        // deadline from the same grace window so we still bound it.
        let deadline = self
            .shutdown_deadline
            .get_or_insert_with(|| Instant::now() + EXTENSION_SHUTDOWN_GRACE);
        let drain_deadline = TokioInstant::from_std(*deadline + EXTENSION_SHUTDOWN_DRAIN_SLACK);

        let drain = async {
            while let Some((key, result)) = self.futures.next().await {
                match result {
                    Ok(Ok(())) => {
                        self.record_completion(&key, false);
                    }
                    Ok(Err(e)) => {
                        otel_warn!(
                            "extension.shutdown.task.error",
                            extension = format!("{}/{}", key.0.as_ref(), key.1.as_str()),
                            error = format!("{e}"),
                        );
                        self.record_completion(&key, true);
                    }
                    Err(e) => {
                        otel_warn!(
                            "extension.shutdown.task.join_error",
                            extension = format!("{}/{}", key.0.as_ref(), key.1.as_str()),
                            is_canceled = e.is_cancelled(),
                            is_panic = e.is_panic(),
                            error = e.to_string()
                        );
                        // Panics are caught inside the task wrapper,
                        // so a `JoinError` here can only be
                        // cancellation — don't bump `task.error`.
                        self.record_cancellation(&key);
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
            // Abort surviving tasks BEFORE recording the timeout so
            // the `active=0` we're about to publish matches reality.
            for handle in &self.abort_handles {
                handle.abort();
            }
            // Anything still in `pending` failed to honour `Shutdown`
            // within the grace window.
            for key in self.pending.clone() {
                otel_warn!(
                    "extension.shutdown.timeout.extension",
                    extension = format!("{}/{}", key.0.as_ref(), key.1.as_str()),
                );
                self.record_timeout(&key);
            }
        }
    }

    /// Stamp `shutdown.duration` (only if the send succeeded), flip
    /// `active` to 0, bump `task.error` on failure, bump
    /// `task.early_exit` when this completion lands before
    /// `broadcast_shutdown` was called, then publish.
    fn record_completion(&mut self, key: &ExtensionKey, is_error: bool) {
        let _ = self.pending.remove(key);
        let pre_broadcast = !self.shutdown_broadcast_fired;
        if let Some(entry) = self.ext_metrics.get_mut(key) {
            entry.record_completion(&self.registry, is_error, pre_broadcast);
        }
    }

    /// Record a drain-deadline timeout. Only increments
    /// `shutdown.timeout` when `Shutdown` was actually sent into the
    /// channel — otherwise the failure is already attributed via
    /// `shutdown.send_failed`.
    fn record_timeout(&mut self, key: &ExtensionKey) {
        let _ = self.pending.remove(key);
        if let Some(entry) = self.ext_metrics.get_mut(key) {
            entry.record_timeout(&self.registry);
        }
    }

    fn record_send_failed(&mut self, key: &ExtensionKey) {
        if let Some(entry) = self.ext_metrics.get_mut(key) {
            entry.record_send_failed(&self.registry);
        }
    }

    /// Finalize an entry whose task was cancelled by the runtime.
    /// Flips `active=0` and removes from `pending` so it isn't later
    /// misattributed as a `shutdown.timeout`. Does NOT bump
    /// `task.error` — cancellation is not an extension fault.
    fn record_cancellation(&mut self, key: &ExtensionKey) {
        let _ = self.pending.remove(key);
        if let Some(entry) = self.ext_metrics.get_mut(key) {
            entry.record_cancellation(&self.registry);
        }
    }
}

/// Best-effort extraction of a human-readable message from a panic
/// payload caught via [`FutureExt::catch_unwind`].
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
        // Abort surviving tasks before touching the registry so they
        // can't race tear-down.
        for handle in &self.abort_handles {
            handle.abort();
        }
        // Publish terminal state for any entry that didn't reach a
        // `record_*` path. Uses the same direct-accumulate path as
        // every other publish — Drop is not special.
        //
        // We deliberately do NOT unregister the per-extension metric
        // sets: the terminal canonical state we just wrote must
        // survive for a post-mortem scrape. Entity rows are
        // reference-counted; we decrement once here to undo
        // `register_extension_entity`'s bump, leaving the metric
        // set's retain in place.
        for entry in self.ext_metrics.values_mut() {
            entry.finalize_on_drop(&self.registry);
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
    use std::borrow::Cow;
    use std::future::Future;

    // ============================================================
    // Test helpers
    // ============================================================

    /// Push a wrapped extension future in the same shape `spawn`
    /// produces, so tests exercise the real key-preserving wrapping.
    fn push_test_extension<F>(
        futures: &mut FuturesUnordered<ExtensionFuture>,
        abort_handles: &mut Vec<AbortHandle>,
        ext_id: ExtensionId,
        variant: ExtensionVariant,
        fut: F,
    ) where
        F: Future<Output = (ExtensionId, Result<(), Error>)> + 'static,
    {
        let handle = tokio::task::spawn_local(fut);
        abort_handles.push(handle.abort_handle());
        let key_for_wrap: ExtensionKey = (ext_id, variant);
        futures.push(Box::pin(async move {
            match handle.await {
                Ok((id, res)) => ((id, variant), Ok(res)),
                Err(e) => (key_for_wrap, Err(e)),
            }
        }));
    }

    fn make_pipeline_context() -> PipelineContext {
        let registry = TelemetryRegistryHandle::new();
        ControllerContext::new(registry).pipeline_context_with("g".into(), "p".into(), 0, 1, 0)
    }

    /// Description of one extension to seed into a test lifecycle.
    struct TestExt {
        id: &'static str,
        variant: ExtensionVariant,
        /// `true` if `Shutdown` was successfully enqueued into the
        /// extension's control channel. Not a delivery receipt.
        delivered: bool,
    }

    impl TestExt {
        fn local(id: &'static str) -> Self {
            Self {
                id,
                variant: ExtensionVariant::Local,
                delivered: true,
            }
        }
        fn shared(id: &'static str) -> Self {
            Self {
                id,
                variant: ExtensionVariant::Shared,
                delivered: true,
            }
        }
        fn undelivered(mut self) -> Self {
            self.delivered = false;
            self
        }
    }

    fn key_of(t: &TestExt) -> ExtensionKey {
        (Cow::Borrowed(t.id), t.variant)
    }

    /// Build a lifecycle pre-seeded with per-extension entries.
    /// `broadcast_fired = true` so direct calls to
    /// `drain_until_deadline` don't synthesize a fresh deadline.
    fn make_test_lifecycle(
        futures: FuturesUnordered<ExtensionFuture>,
        abort_handles: Vec<AbortHandle>,
        exts: &[TestExt],
    ) -> ExtensionLifecycle {
        let pipeline_ctx = make_pipeline_context();
        let ext_ctx = ExtensionContext::from_pipeline(&pipeline_ctx);
        let registry = ext_ctx.metrics_registry();
        let mut ext_metrics: HashMap<ExtensionKey, ExtensionMetricsEntry> = HashMap::new();
        let mut pending: HashSet<ExtensionKey> = HashSet::new();
        let mut registered_entities = Vec::new();
        for ext in exts {
            let ext_id: ExtensionId = Cow::Borrowed(ext.id);
            let entity_key = ext_ctx.register_extension_entity(&ext_id, ext.variant);
            registered_entities.push(entity_key);
            let mut entry = ExtensionMetricsEntry::new(
                registry.register_metric_set_for_entity::<ExtensionLifecycleMetrics>(entity_key),
            );
            if ext.delivered {
                entry.mark_shutdown_sent(Instant::now());
            }
            let key = (ext_id, ext.variant);
            let _ = ext_metrics.insert(key.clone(), entry);
            let _ = pending.insert(key);
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
            registered_entities,
            registry,
        }
    }

    /// `name -> u64` map per metric set currently in the registry.
    fn collect_registry_metrics(
        registry: &TelemetryRegistryHandle,
    ) -> Vec<HashMap<&'static str, u64>> {
        let mut out = Vec::new();
        registry.visit_current_metrics_with_zeroes(
            |_desc, _attrs, iter| {
                let mut map: HashMap<&'static str, u64> = HashMap::new();
                for (field, value) in iter {
                    let n = match value {
                        otap_df_telemetry::metrics::MetricValue::U64(n) => n,
                        otap_df_telemetry::metrics::MetricValue::F64(n) => n as u64,
                        otap_df_telemetry::metrics::MetricValue::Mmsc(s) => s.count,
                    };
                    let _ = map.insert(field.name, n);
                }
                out.push(map);
            },
            true,
        );
        out
    }

    /// Pull the value of a registry metric for a specific extension
    /// variant. Returns `0` if no row matches or the metric is
    /// absent. Used by tests that need to assert counter totals
    /// (the local `MetricSet` is cleared after every direct publish,
    /// so the registry is the source of truth).
    fn registry_metric_for(
        registry: &TelemetryRegistryHandle,
        ext_id: &str,
        variant: ExtensionVariant,
        metric_name: &str,
    ) -> u64 {
        let want_variant = variant.as_str();
        let mut found: u64 = 0;
        registry.visit_current_metrics_with_zeroes(
            |_desc, attrs, iter| {
                let mut id_ok = false;
                let mut variant_ok = false;
                for (k, v) in attrs.iter_attributes() {
                    if k == "extension.id" && v.to_string_value() == ext_id {
                        id_ok = true;
                    }
                    if k == "extension.variant" && v.to_string_value() == want_variant {
                        variant_ok = true;
                    }
                }
                if !(id_ok && variant_ok) {
                    return;
                }
                for (field, value) in iter {
                    if field.name == metric_name {
                        found = match value {
                            otap_df_telemetry::metrics::MetricValue::U64(n) => n,
                            otap_df_telemetry::metrics::MetricValue::F64(n) => n as u64,
                            otap_df_telemetry::metrics::MetricValue::Mmsc(s) => s.count,
                        };
                    }
                }
            },
            true,
        );
        found
    }

    // ============================================================
    // Single-extension scenarios
    // ============================================================

    /// A misbehaving extension that never returns must not stall
    /// `drain_until_deadline` past its deadline.
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
                ExtensionVariant::Local,
                async {
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("stuck"), Ok(()))
                },
            );

            let injected_deadline = Instant::now() + Duration::from_millis(100);
            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("stuck")]);
            lifecycle.shutdown_deadline = Some(injected_deadline);

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
                "stuck extension should still be in `futures` after the bounded drain timed out",
            );

            let key: ExtensionKey = (Cow::Borrowed("stuck"), ExtensionVariant::Local);
            let entry = lifecycle.ext_metrics.get(&key).expect("entry exists");
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "stuck",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                1
            );
            assert_eq!(entry.active(), 0);
        }));
    }

    /// Normal completion: `active = 0`, non-zero duration.
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
                ExtensionVariant::Local,
                async {
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    (Cow::Borrowed("clean"), Ok(()))
                },
            );

            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("clean")]);
            let key: ExtensionKey = (Cow::Borrowed("clean"), ExtensionVariant::Local);
            // Backdate so elapsed > 0 deterministically.
            if let Some(entry) = lifecycle.ext_metrics.get_mut(&key) {
                entry.mark_shutdown_sent(Instant::now() - Duration::from_millis(1));
            }

            let result = lifecycle.next_completion().await.expect("queued");
            assert!(matches!(result, Ok(Ok(()))));

            let entry = lifecycle.ext_metrics.get(&key).expect("entry exists");
            assert_eq!(entry.active(), 0);
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "clean",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                0
            );
            assert!(
                entry.shutdown_duration_ns().unwrap_or(0) > 0,
                "shutdown_duration_ns should be stamped on completion",
            );
            assert_eq!(entry.shutdown_sent(), 1);
            assert!(!lifecycle.pending.contains(&key));
        }));
    }

    /// `Err(_)` from `start()` increments `task.error`.
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
                ExtensionVariant::Local,
                async {
                    (
                        Cow::Borrowed("boom"),
                        Err(Error::InternalError {
                            message: "boom".to_string(),
                        }),
                    )
                },
            );

            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("boom")]);
            let key: ExtensionKey = (Cow::Borrowed("boom"), ExtensionVariant::Local);

            let result = lifecycle.next_completion().await.expect("queued");
            assert!(matches!(result, Ok(Err(_))));

            let entry = lifecycle.ext_metrics.get(&key).expect("entry exists");
            assert_eq!(entry.active(), 0);
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "boom",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                1
            );
        }));
    }

    /// A task that resolves before `broadcast_shutdown` was called
    /// is a lifecycle-contract anomaly: extensions are supposed to
    /// run until they receive `Shutdown`. The clean `Ok(())` case
    /// must bump `task.early_exit` (and only that — no `task.error`).
    #[test]
    fn clean_exit_before_broadcast_bumps_early_exit() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("early-clean"),
                ExtensionVariant::Local,
                async { (Cow::Borrowed("early-clean"), Ok(())) },
            );

            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("early-clean").undelivered()],
            );
            // The helper latches `shutdown_broadcast_fired=true` so
            // `broadcast_shutdown` short-circuits in other tests.
            // Here we need the opposite — simulate a completion that
            // arrives BEFORE broadcast was called.
            lifecycle.shutdown_broadcast_fired = false;

            let result = lifecycle.next_completion().await.expect("queued");
            assert!(matches!(result, Ok(Ok(()))));

            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "early-clean",
                    ExtensionVariant::Local,
                    "task.early_exit",
                ),
                1,
                "clean exit before broadcast must bump task.early_exit",
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "early-clean",
                    ExtensionVariant::Local,
                    "task.error",
                ),
                0,
                "clean exit must not bump task.error",
            );
        }));
    }

    /// An error return before `broadcast_shutdown` ran bumps BOTH
    /// `task.error` AND `task.early_exit`: the two counters are
    /// orthogonal and describe different aspects of the same event
    /// (it errored; it errored before being asked to stop).
    #[test]
    fn error_exit_before_broadcast_bumps_both_counters() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("early-err"),
                ExtensionVariant::Local,
                async {
                    (
                        Cow::Borrowed("early-err"),
                        Err(Error::InternalError {
                            message: "early boom".to_string(),
                        }),
                    )
                },
            );

            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("early-err").undelivered()],
            );
            lifecycle.shutdown_broadcast_fired = false;

            let result = lifecycle.next_completion().await.expect("queued");
            assert!(matches!(result, Ok(Err(_))));

            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "early-err",
                    ExtensionVariant::Local,
                    "task.early_exit",
                ),
                1,
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "early-err",
                    ExtensionVariant::Local,
                    "task.error",
                ),
                1,
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "early-err",
                    ExtensionVariant::Local,
                    "shutdown.send_failed",
                ),
                0,
                "early err exit must not be misattributed as send_failed",
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "early-err",
                    ExtensionVariant::Local,
                    "shutdown.sent",
                ),
                0,
                "broadcast never ran — shutdown.sent must remain 0",
            );
        }));
    }

    /// A completion that arrives AFTER `broadcast_shutdown` has run
    /// is the normal path; `task.early_exit` must stay at 0. This
    /// is the inverse of the early-exit tests above.
    #[test]
    fn completion_after_broadcast_does_not_bump_early_exit() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("normal"),
                ExtensionVariant::Local,
                async { (Cow::Borrowed("normal"), Ok(())) },
            );

            // `make_test_lifecycle` defaults `shutdown_broadcast_fired=true`,
            // so a completion now models "Shutdown was already
            // broadcast; extension is honoring it."
            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("normal")]);
            assert!(lifecycle.shutdown_broadcast_fired);

            let _ = lifecycle.next_completion().await.expect("queued");

            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "normal",
                    ExtensionVariant::Local,
                    "task.early_exit",
                ),
                0,
                "post-broadcast completion must not bump task.early_exit",
            );
        }));
    }

    /// Cancellation by the runtime is not the extension's choice;
    /// it must NOT bump `task.early_exit` even if it lands before
    /// broadcast. This guards the orthogonal-attribution
    /// invariant: `record_cancellation` is a different code path
    /// from `record_completion` and must remain so.
    #[test]
    fn cancellation_before_broadcast_does_not_bump_early_exit() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("cancelled-early"),
                ExtensionVariant::Local,
                async {
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("cancelled-early"), Ok(()))
                },
            );

            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("cancelled-early").undelivered()],
            );
            lifecycle.shutdown_broadcast_fired = false;
            for handle in &lifecycle.abort_handles {
                handle.abort();
            }

            let _ = lifecycle.next_completion().await.expect("queued");

            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "cancelled-early",
                    ExtensionVariant::Local,
                    "task.early_exit",
                ),
                0,
                "cancellation must not be attributed as early_exit",
            );
        }));
    }

    /// Undelivered `Shutdown` must NOT record `shutdown.duration`
    /// or `shutdown.timeout` — those are reserved for extensions
    /// that actually received the request.
    #[test]
    fn record_completion_skips_duration_when_shutdown_not_sent() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("undelivered"),
                ExtensionVariant::Local,
                async { (Cow::Borrowed("undelivered"), Ok(())) },
            );
            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("undelivered").undelivered()],
            );
            let key: ExtensionKey = (Cow::Borrowed("undelivered"), ExtensionVariant::Local);

            let _ = lifecycle.next_completion().await.expect("queued");

            let entry = lifecycle.ext_metrics.get(&key).expect("entry exists");
            assert_eq!(
                entry.shutdown_duration_ns(),
                None,
                "duration must not be stamped when Shutdown was never delivered",
            );
            assert_eq!(entry.shutdown_sent(), 0);
            assert_eq!(entry.active(), 0);
        }));
    }

    /// `record_timeout` must not bump `shutdown.timeout` if
    /// `Shutdown` was never delivered.
    #[test]
    fn record_timeout_skips_increment_when_shutdown_not_sent() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("stuck-no-delivery"),
                ExtensionVariant::Local,
                async {
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("stuck-no-delivery"), Ok(()))
                },
            );

            let injected_deadline = Instant::now() + Duration::from_millis(50);
            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("stuck-no-delivery").undelivered()],
            );
            lifecycle.shutdown_deadline = Some(injected_deadline);

            lifecycle.drain_until_deadline().await;

            let key: ExtensionKey = (Cow::Borrowed("stuck-no-delivery"), ExtensionVariant::Local);
            let entry = lifecycle.ext_metrics.get(&key).expect("entry exists");
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "stuck-no-delivery",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                0,
                "timeout must not be ticked when Shutdown was never delivered",
            );
            assert_eq!(entry.active(), 0);
        }));
    }

    /// A panic in `start()` is caught and attributed via `task.error`.
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
                ExtensionVariant::Local,
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

            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("panicker")]);
            let key: ExtensionKey = (Cow::Borrowed("panicker"), ExtensionVariant::Local);
            let result = lifecycle.next_completion().await.expect("queued");
            assert!(matches!(result, Ok(Err(_))));

            let entry = lifecycle.ext_metrics.get(&key).expect("entry exists");
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "panicker",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                1
            );
            assert_eq!(entry.active(), 0);
        }));
    }

    /// A runtime-cancelled task is finalized (so it isn't later
    /// misattributed as a `shutdown.timeout`) but does NOT bump
    /// `task.error`.
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
                ExtensionVariant::Local,
                async {
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("cancelled"), Ok(()))
                },
            );

            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("cancelled")]);
            let key: ExtensionKey = (Cow::Borrowed("cancelled"), ExtensionVariant::Local);
            for handle in &lifecycle.abort_handles {
                handle.abort();
            }

            let result = lifecycle.next_completion().await.expect("queued");
            assert!(
                matches!(&result, Err(e) if e.is_cancelled()),
                "cancellation must surface as Err(JoinError::cancelled), got {result:?}",
            );

            assert!(!lifecycle.pending.contains(&key));
            let entry = lifecycle.ext_metrics.get(&key).expect("entry exists");
            assert_eq!(entry.active(), 0);
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "cancelled",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                0
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "cancelled",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                0
            );
        }));
    }

    // ============================================================
    // Composite key (Local + Shared with same id)
    // ============================================================

    /// Local and shared variants of one id own independent entries.
    #[test]
    fn local_and_shared_variants_have_independent_entries() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("dual"),
                ExtensionVariant::Local,
                async { (Cow::Borrowed("dual"), Ok(())) },
            );
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("dual"),
                ExtensionVariant::Shared,
                async {
                    (
                        Cow::Borrowed("dual"),
                        Err(Error::InternalError {
                            message: "shared variant failed".into(),
                        }),
                    )
                },
            );

            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("dual"), TestExt::shared("dual")],
            );
            assert_eq!(
                lifecycle.ext_metrics.len(),
                2,
                "composite key must keep local and shared variants in separate entries",
            );

            // Drain both completions.
            let _ = lifecycle.next_completion().await.expect("first");
            let _ = lifecycle.next_completion().await.expect("second");

            let local_key: ExtensionKey = (Cow::Borrowed("dual"), ExtensionVariant::Local);
            let shared_key: ExtensionKey = (Cow::Borrowed("dual"), ExtensionVariant::Shared);

            let local_entry = lifecycle.ext_metrics.get(&local_key).expect("local entry");
            let shared_entry = lifecycle
                .ext_metrics
                .get(&shared_key)
                .expect("shared entry");

            assert_eq!(local_entry.active(), 0);
            assert_eq!(shared_entry.active(), 0);
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "dual",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                0,
                "local variant succeeded — must not inherit shared variant's error",
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "dual",
                    ExtensionVariant::Shared,
                    "task.error"
                ),
                1,
                "shared variant returned Err — must increment its own task_error",
            );
        }));
    }

    /// When only one of (local, shared) for the same id times out,
    /// the timeout attaches to the right variant and does not bleed
    /// into the other.
    #[test]
    fn timeout_attributes_to_correct_variant_only() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            // Local completes cleanly.
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("dual-mix"),
                ExtensionVariant::Local,
                async { (Cow::Borrowed("dual-mix"), Ok(())) },
            );
            // Shared variant never returns.
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("dual-mix"),
                ExtensionVariant::Shared,
                async {
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("dual-mix"), Ok(()))
                },
            );

            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("dual-mix"), TestExt::shared("dual-mix")],
            );
            lifecycle.shutdown_deadline = Some(Instant::now() + Duration::from_millis(80));

            lifecycle.drain_until_deadline().await;

            let local_key: ExtensionKey = (Cow::Borrowed("dual-mix"), ExtensionVariant::Local);
            let shared_key: ExtensionKey = (Cow::Borrowed("dual-mix"), ExtensionVariant::Shared);

            let local_entry = lifecycle.ext_metrics.get(&local_key).expect("local");
            let shared_entry = lifecycle.ext_metrics.get(&shared_key).expect("shared");
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "dual-mix",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                0,
                "local variant completed cleanly — must not be marked timed out",
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "dual-mix",
                    ExtensionVariant::Shared,
                    "shutdown.timeout"
                ),
                1,
                "shared variant blew past deadline — must be marked timed out",
            );
            assert_eq!(local_entry.active(), 0);
            assert_eq!(shared_entry.active(), 0);
            assert_eq!(
                shared_entry.shutdown_duration_ns(),
                None,
                "timed-out variant must not record a misleading shutdown.duration",
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "dual-mix",
                    ExtensionVariant::Shared,
                    "shutdown.duration"
                ),
                0,
                "registry must not surface a duration for a timed-out variant",
            );
        }));
    }

    // ============================================================
    // Multi-extension scenarios
    // ============================================================

    /// Five extensions all complete cleanly: all flushed,
    /// `pending` empty, each has its own non-zero duration.
    #[test]
    fn many_extensions_all_complete_cleanly() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let ids = ["a", "b", "c", "d", "e"];
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            for id in ids {
                push_test_extension(
                    &mut futures,
                    &mut abort_handles,
                    Cow::Borrowed(id),
                    ExtensionVariant::Local,
                    async move {
                        tokio::time::sleep(Duration::from_millis(2)).await;
                        (Cow::Borrowed(id), Ok(()))
                    },
                );
            }
            let exts: Vec<TestExt> = ids.iter().copied().map(TestExt::local).collect();
            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &exts);
            // Backdate every entry so each duration is deterministically > 0.
            for id in ids {
                let key: ExtensionKey = (Cow::Borrowed(id), ExtensionVariant::Local);
                if let Some(entry) = lifecycle.ext_metrics.get_mut(&key) {
                    entry.mark_shutdown_sent(Instant::now() - Duration::from_millis(1));
                }
            }

            for _ in ids {
                let _ = lifecycle.next_completion().await.expect("queued");
            }

            assert!(lifecycle.pending.is_empty());
            for id in ids {
                let key: ExtensionKey = (Cow::Borrowed(id), ExtensionVariant::Local);
                let entry = lifecycle.ext_metrics.get(&key).expect("entry");
                assert_eq!(entry.active(), 0, "{id}: active");
                assert_eq!(
                    registry_metric_for(
                        &lifecycle.registry,
                        id,
                        ExtensionVariant::Local,
                        "task.error"
                    ),
                    0,
                    "{id}: no errors"
                );
                assert!(
                    entry.shutdown_duration_ns().unwrap_or(0) > 0,
                    "{id}: duration > 0"
                );
            }
        }));
    }

    /// Mixed outcomes (clean / err / panic / timeout) across
    /// extensions are isolated — no cross-contamination of metrics.
    #[test]
    fn mixed_outcomes_across_extensions_are_independent() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            // clean
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("clean"),
                ExtensionVariant::Local,
                async { (Cow::Borrowed("clean"), Ok(())) },
            );
            // err
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("err"),
                ExtensionVariant::Local,
                async {
                    (
                        Cow::Borrowed("err"),
                        Err(Error::InternalError {
                            message: "x".into(),
                        }),
                    )
                },
            );
            // panic
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("panic"),
                ExtensionVariant::Local,
                async {
                    let payload = AssertUnwindSafe(async { panic!("p") }).catch_unwind().await;
                    let result = match payload {
                        Ok(()) => Ok(()),
                        Err(p) => Err(Error::InternalError {
                            message: panic_payload_to_string(&*p),
                        }),
                    };
                    (Cow::Borrowed("panic"), result)
                },
            );
            // stuck → times out
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("stuck"),
                ExtensionVariant::Local,
                async {
                    std::future::pending::<()>().await;
                    (Cow::Borrowed("stuck"), Ok(()))
                },
            );

            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[
                    TestExt::local("clean"),
                    TestExt::local("err"),
                    TestExt::local("panic"),
                    TestExt::local("stuck"),
                ],
            );
            lifecycle.shutdown_deadline = Some(Instant::now() + Duration::from_millis(80));

            lifecycle.drain_until_deadline().await;

            let clean: ExtensionKey = (Cow::Borrowed("clean"), ExtensionVariant::Local);
            let err: ExtensionKey = (Cow::Borrowed("err"), ExtensionVariant::Local);
            let pan: ExtensionKey = (Cow::Borrowed("panic"), ExtensionVariant::Local);
            let stuck: ExtensionKey = (Cow::Borrowed("stuck"), ExtensionVariant::Local);

            let e_clean = lifecycle.ext_metrics.get(&clean).unwrap();
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "clean",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                0
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "clean",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                0
            );
            assert_eq!(e_clean.active(), 0);

            let e_err = lifecycle.ext_metrics.get(&err).unwrap();
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "err",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                1
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "err",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                0
            );
            assert_eq!(e_err.active(), 0);

            let e_pan = lifecycle.ext_metrics.get(&pan).unwrap();
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "panic",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                1
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "panic",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                0
            );
            assert_eq!(e_pan.active(), 0);

            let e_stuck = lifecycle.ext_metrics.get(&stuck).unwrap();
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "stuck",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                0,
                "stuck extension is timed out by runtime, not faulted"
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "stuck",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                1
            );
            assert_eq!(e_stuck.active(), 0);
        }));
    }

    /// Per-extension durations are anchored independently: a slow
    /// send for A must not skew B's duration. (Regression for the
    /// pre-fix post-`join_all` timestamp capture.)
    #[test]
    fn per_extension_durations_are_independent() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let mut futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let mut abort_handles: Vec<AbortHandle> = Vec::new();
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("slow"),
                ExtensionVariant::Local,
                async { (Cow::Borrowed("slow"), Ok(())) },
            );
            push_test_extension(
                &mut futures,
                &mut abort_handles,
                Cow::Borrowed("fast"),
                ExtensionVariant::Local,
                async { (Cow::Borrowed("fast"), Ok(())) },
            );

            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("slow"), TestExt::local("fast")],
            );
            let slow_key: ExtensionKey = (Cow::Borrowed("slow"), ExtensionVariant::Local);
            let fast_key: ExtensionKey = (Cow::Borrowed("fast"), ExtensionVariant::Local);
            // `slow` was anchored 50ms ago; `fast` was anchored just now.
            lifecycle
                .ext_metrics
                .get_mut(&slow_key)
                .unwrap()
                .mark_shutdown_sent(Instant::now() - Duration::from_millis(50));
            lifecycle
                .ext_metrics
                .get_mut(&fast_key)
                .unwrap()
                .mark_shutdown_sent(Instant::now());

            let _ = lifecycle.next_completion().await;
            let _ = lifecycle.next_completion().await;

            let slow = lifecycle.ext_metrics.get(&slow_key).unwrap();
            let fast = lifecycle.ext_metrics.get(&fast_key).unwrap();
            let slow_dur = slow.shutdown_duration_ns().expect("slow duration");
            let fast_dur = fast.shutdown_duration_ns().expect("fast duration");
            assert!(
                slow_dur > fast_dur,
                "slow ({slow_dur} ns) must have larger duration than fast ({fast_dur} ns) \
                 because its delivery anchor is older",
            );
            // sanity: slow >= 40ms (allow scheduler jitter).
            assert!(
                slow_dur >= 40_000_000,
                "slow duration {slow_dur} ns < 40ms — anchor was not honored",
            );
        }));
    }

    // ============================================================
    // `broadcast_shutdown` semantics (without real senders)
    // ============================================================

    /// `broadcast_shutdown` with no senders is a no-op and leaves
    /// the one-shot latch unconsumed.
    #[test]
    fn broadcast_shutdown_noop_with_no_senders() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &[]);
            // Reset latch so we can observe the no-op path.
            lifecycle.shutdown_broadcast_fired = false;
            lifecycle.broadcast_shutdown().await;
            assert!(
                !lifecycle.shutdown_broadcast_fired,
                "no-op broadcast must not consume the one-shot latch",
            );
        }));
    }

    /// Calling `broadcast_shutdown` twice is idempotent.
    #[test]
    fn broadcast_shutdown_is_idempotent() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &[]);
            lifecycle.broadcast_shutdown().await; // already true from helper, exits early
            lifecycle.broadcast_shutdown().await; // exits early
            assert!(lifecycle.shutdown_broadcast_fired);
        }));
    }

    /// `broadcast_shutdown` must NOT send to a variant whose entry
    /// was already finalized before the broadcast — e.g. an
    /// extension that returned `Err(_)` or panicked, so its
    /// `record_completion` ran from the select loop before
    /// `broadcast_shutdown` was even called. Its receiver is gone
    /// with the task; sending would surface a spurious
    /// `shutdown.send_failed` on top of the genuine `task.error`,
    /// double-attributing a single failure. Healthy sibling
    /// variants on the same extension id must still receive
    /// `Shutdown`.
    #[test]
    fn broadcast_shutdown_skips_already_finalized_entries() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[
                    TestExt::local("twin").undelivered(),
                    TestExt::shared("twin").undelivered(),
                ],
            );

            // Real channels for both variants. We keep both
            // receivers alive so we can observe what each one
            // actually got (or did not get).
            let (local_tx, mut local_rx) = tokio::sync::mpsc::channel::<ExtensionControlMsg>(4);
            let (shared_tx, mut shared_rx) = tokio::sync::mpsc::channel::<ExtensionControlMsg>(4);
            lifecycle.shutdown_senders.push((
                ExtensionVariant::Local,
                ExtensionControlSender {
                    name: Cow::Borrowed("twin"),
                    sender: crate::message::Sender::Shared(
                        crate::shared::message::SharedSender::mpsc(local_tx),
                    ),
                },
            ));
            lifecycle.shutdown_senders.push((
                ExtensionVariant::Shared,
                ExtensionControlSender {
                    name: Cow::Borrowed("twin"),
                    sender: crate::message::Sender::Shared(
                        crate::shared::message::SharedSender::mpsc(shared_tx),
                    ),
                },
            ));

            // Finalize the Local variant first: simulates the
            // extension erroring out before broadcast ran.
            let local_key: ExtensionKey = (Cow::Borrowed("twin"), ExtensionVariant::Local);
            lifecycle.record_completion(&local_key, true);

            // Reset the latch (helper sets it to `true` so default
            // broadcast calls are no-ops).
            lifecycle.shutdown_broadcast_fired = false;
            lifecycle.broadcast_shutdown().await;

            // Shared variant must receive `Shutdown`.
            assert!(
                matches!(
                    shared_rx.try_recv(),
                    Ok(ExtensionControlMsg::Shutdown { .. })
                ),
                "live sibling variant must receive Shutdown",
            );

            // Local variant's channel must still be empty —
            // broadcast skipped it.
            assert!(
                matches!(
                    local_rx.try_recv(),
                    Err(tokio::sync::mpsc::error::TryRecvError::Empty)
                ),
                "finalized variant must NOT receive Shutdown",
            );

            // Telemetry for the finalized Local variant: only
            // `task.error == 1` (from `record_completion`). No
            // spurious `shutdown.send_failed` from a broadcast that
            // should never have happened.
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "twin",
                    ExtensionVariant::Local,
                    "task.error",
                ),
                1,
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "twin",
                    ExtensionVariant::Local,
                    "shutdown.send_failed",
                ),
                0,
                "must not attribute send_failed on a pre-finalized variant",
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "twin",
                    ExtensionVariant::Local,
                    "shutdown.sent",
                ),
                0,
                "finalized variant must not be credited with shutdown.sent",
            );

            // And the Shared sibling really did get a Shutdown
            // attributed to it. `mark_shutdown_sent` updates
            // the entry's in-memory state but does not publish on
            // its own — a subsequent terminal recorder is what
            // pushes the snapshot to the registry. Assert the
            // local state here, which is what the next publish
            // would write.
            let shared_key: ExtensionKey = (Cow::Borrowed("twin"), ExtensionVariant::Shared);
            let shared_entry = lifecycle
                .ext_metrics
                .get(&shared_key)
                .expect("shared entry must exist");
            assert_eq!(
                shared_entry.shutdown_sent(),
                1,
                "live sibling variant must be marked shutdown-sent",
            );
            assert!(
                !shared_entry.is_finalized(),
                "shutdown delivery alone must not finalize the entry",
            );
        }));
    }

    // ============================================================
    // Drop-time direct accumulation into the registry
    // ============================================================

    /// Drop publishes terminal `active = 0` directly into the
    /// registry.
    #[test]
    fn drop_publishes_terminal_state_directly_into_registry() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle = make_test_lifecycle(
                futures,
                abort_handles,
                &[TestExt::local("first"), TestExt::shared("first")],
            );
            // Drive both entries through a terminal recorder so canonical
            // state is meaningful before Drop runs.
            for key in lifecycle.ext_metrics.keys().cloned().collect::<Vec<_>>() {
                lifecycle.record_completion(&key, false);
            }
            let registry = lifecycle.registry.clone();
            drop(lifecycle);

            let snapshots = collect_registry_metrics(&registry);
            assert_eq!(snapshots.len(), 2);
            for s in snapshots {
                assert_eq!(
                    s.get("active").copied(),
                    Some(0),
                    "Drop must publish active=0 directly into registry",
                );
                assert_eq!(s.get("shutdown.sent").copied(), Some(1));
            }
        }));
    }

    /// Drop is the safety net for entries that never reached a
    /// terminal `record_*`: they still get `active = 0` in the
    /// registry.
    #[test]
    fn drop_finalizes_non_finalized_entries() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("orphan")]);
            let registry = lifecycle.registry.clone();
            // No record_* calls — entry stays non-finalized through Drop.
            drop(lifecycle);

            let snapshots = collect_registry_metrics(&registry);
            assert_eq!(snapshots.len(), 1);
            assert_eq!(snapshots[0].get("active").copied(), Some(0));
        }));
    }

    /// Drop after a `record_*` call must NOT double-publish counter
    /// deltas (e.g. `shutdown_timeout` should remain `1`, not `2`).
    /// Counters are sent as deltas with `clear_values` after each
    /// publish, so Drop's republish contributes a zero delta for them.
    #[test]
    fn drop_does_not_double_count_counters() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("once")]);
            let key: ExtensionKey = (Cow::Borrowed("once"), ExtensionVariant::Local);
            lifecycle.record_timeout(&key);
            let registry = lifecycle.registry.clone();
            drop(lifecycle);

            let snapshots = collect_registry_metrics(&registry);
            assert_eq!(snapshots.len(), 1);
            assert_eq!(
                snapshots[0].get("shutdown.timeout").copied(),
                Some(1),
                "Drop must not double-bump the shutdown.timeout counter",
            );
            assert_eq!(snapshots[0].get("active").copied(), Some(0));
        }));
    }

    // ============================================================
    // Active gauge canonical-state correctness
    // ============================================================

    /// Sequence of events on the same extension: `record_send_failed`
    /// (task is still running, `active` must stay `1` in the
    /// canonical state) followed by `record_completion` (task done,
    /// `active=0`). Gauges replace, so the last publish wins.
    #[test]
    fn active_canonical_state_survives_intermediate_events() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("seq")]);
            let key: ExtensionKey = (Cow::Borrowed("seq"), ExtensionVariant::Local);

            // 1. record_send_failed: task still running, active must stay 1.
            lifecycle.record_send_failed(&key);
            {
                let entry = lifecycle.ext_metrics.get(&key).unwrap();
                assert_eq!(
                    entry.active(),
                    1,
                    "send_failed does not terminate the task — canonical active must stay 1",
                );
            }

            // 2. record_completion: task done, active must flip to 0.
            lifecycle.record_completion(&key, false);
            {
                let entry = lifecycle.ext_metrics.get(&key).unwrap();
                assert_eq!(entry.active(), 0);
            }

            let registry = lifecycle.registry.clone();
            drop(lifecycle);

            let snapshots = collect_registry_metrics(&registry);
            assert_eq!(snapshots.len(), 1);
            assert_eq!(
                snapshots[0].get("active").copied(),
                Some(0),
                "registry must reflect final active=0",
            );
            assert_eq!(
                snapshots[0].get("shutdown.send_failed").copied(),
                Some(1),
                "send_failed counter delta must survive into the registry",
            );
        }));
    }

    /// Calling a terminal recorder twice on the same extension does
    /// not double-count: `finalized` guards against it.
    #[test]
    fn finalized_guard_prevents_double_recording() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("dup")]);
            let key: ExtensionKey = (Cow::Borrowed("dup"), ExtensionVariant::Local);
            lifecycle.record_completion(&key, true);
            // Second call must be a no-op for the counter.
            lifecycle.record_completion(&key, true);
            lifecycle.record_timeout(&key); // also no-op
            lifecycle.record_cancellation(&key); // also no-op

            let entry = lifecycle.ext_metrics.get(&key).unwrap();
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "dup",
                    ExtensionVariant::Local,
                    "task.error"
                ),
                1
            );
            assert_eq!(
                registry_metric_for(
                    &lifecycle.registry,
                    "dup",
                    ExtensionVariant::Local,
                    "shutdown.timeout"
                ),
                0
            );
            assert_eq!(entry.active(), 0);
        }));
    }

    /// Unknown extension keys passed to `record_*` are silently
    /// no-op.
    #[test]
    fn record_methods_are_noop_for_unknown_key() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle = make_test_lifecycle(futures, abort_handles, &[]);
            let key: ExtensionKey = (Cow::Borrowed("ghost"), ExtensionVariant::Local);
            // None of these should panic / cause side effects.
            lifecycle.record_completion(&key, true);
            lifecycle.record_timeout(&key);
            lifecycle.record_send_failed(&key);
            lifecycle.record_cancellation(&key);
        }));
    }

    // ============================================================
    // End-to-end: record path lands in registry
    // ============================================================

    /// `record_completion` writes directly into the registry. After
    /// the call returns, the registry already contains the expected
    /// values for every metric — no drain step required.
    #[test]
    fn record_completion_lands_in_registry() {
        let (rt, local_tasks) = crate::testing::setup_test_runtime();
        rt.block_on(local_tasks.run_until(async {
            let futures: FuturesUnordered<ExtensionFuture> = FuturesUnordered::new();
            let abort_handles: Vec<AbortHandle> = Vec::new();
            let mut lifecycle =
                make_test_lifecycle(futures, abort_handles, &[TestExt::local("e2e")]);
            let key: ExtensionKey = (Cow::Borrowed("e2e"), ExtensionVariant::Local);
            lifecycle
                .ext_metrics
                .get_mut(&key)
                .unwrap()
                .mark_shutdown_sent(Instant::now() - Duration::from_millis(3));
            lifecycle.record_completion(&key, false);

            let registry = lifecycle.registry.clone();
            // No drain needed — record_completion publishes directly.

            let snapshots = collect_registry_metrics(&registry);
            assert_eq!(snapshots.len(), 1);
            assert_eq!(snapshots[0].get("active").copied(), Some(0));
            assert_eq!(snapshots[0].get("shutdown.sent").copied(), Some(1));
            assert!(
                snapshots[0].get("shutdown.duration").copied().unwrap_or(0) > 0,
                "shutdown.duration must be > 0 with a backdated anchor",
            );
        }));
    }

    /// Just exercise the helper builder paths so future refactors of
    /// the helper signature can't silently break the fixture API.
    #[test]
    fn test_helpers_compile_and_key_extraction_works() {
        let l = TestExt::local("x");
        let s = TestExt::shared("x").undelivered();
        assert_eq!(key_of(&l), (Cow::Borrowed("x"), ExtensionVariant::Local));
        assert_eq!(key_of(&s), (Cow::Borrowed("x"), ExtensionVariant::Shared));
        assert!(!s.delivered);
    }
}
