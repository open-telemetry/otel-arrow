// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The Kubernetes Service Account Token extension: `Arc<Inner>` state, the
//! `BearerTokenProvider` capability implementation, and the background refresh
//! loop driven by the active `Extension::start()` task.

use std::path::Path;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::StreamExt;
use notify::{Event, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use otap_df_engine::capability::{
    BearerToken, BearerTokenProvider as BearerTokenProviderCap, CapabilityError,
    CapabilityErrorSource, TokenStream,
};
use otap_df_engine::control::ExtensionControlMsg;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::extension::EffectHandler;
use otap_df_engine::shared::capability::BearerTokenProvider as SharedBearerTokenProvider;
use otap_df_engine::shared::extension::{ControlChannel, Extension as SharedExtension};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_telemetry::otel_warn;
use tokio::sync::{mpsc, watch};
use tokio_stream::wrappers::WatchStream;

use super::metrics::K8sServiceAccountTokenMetricsTracker;
use super::token_source::TokenSource;

/// Safety margin before actual expiry within which a cached token is treated as
/// no longer usable.
const TOKEN_USABLE_MARGIN_SECS: u64 = 30;
/// Reschedule delay after a failed read (e.g. the token file is briefly absent).
const TOKEN_REFRESH_RETRY_SECS: u64 = 10;

/// Shared, clonable Kubernetes Service Account Token extension.
///
/// Every clone (consumers + the background refresh task) observes the same
/// [`Inner`] state via `Arc`, so they share one token cache and refresh loop.
#[derive(Clone)]
pub struct K8sServiceAccountTokenExtension {
    inner: Arc<Inner>,
}

/// Shared state behind [`K8sServiceAccountTokenExtension`].
struct Inner {
    /// Reads the token and declares its refresh mechanics.
    source: Arc<dyn TokenSource>,
    /// Token cache + pub/sub for `token_stream()`.
    tx: watch::Sender<Option<BearerToken>>,
    /// Pre-tagged capability error builder.
    cap_err: CapabilityErrorSource<BearerTokenProviderCap>,
    /// Coalesces concurrent slow-path reads onto one in-flight read.
    fetch_lock: tokio::sync::Mutex<()>,
    /// Instant of the most recent failed read (negative cache) used to throttle
    /// slow-path retries.
    last_failure: Mutex<Option<Instant>>,
    /// Metric tracker. Its critical sections are short and never span an
    /// `.await`, so a `std` `Mutex` is appropriate.
    metrics: Mutex<K8sServiceAccountTokenMetricsTracker>,
}

impl K8sServiceAccountTokenExtension {
    /// Builds a new extension instance.
    #[must_use]
    pub fn new(
        name: &str,
        source: Arc<dyn TokenSource>,
        tx: watch::Sender<Option<BearerToken>>,
        metrics: K8sServiceAccountTokenMetricsTracker,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                source,
                tx,
                cap_err: CapabilityErrorSource::new(name.to_owned().into()),
                fetch_lock: tokio::sync::Mutex::new(()),
                last_failure: Mutex::new(None),
                metrics: Mutex::new(metrics),
            }),
        }
    }
}

impl Inner {
    /// Returns the cached token if present and still comfortably before expiry.
    fn current_fresh_token(&self) -> Option<BearerToken> {
        let token = self.tx.borrow().clone()?;
        match token.expires_on() {
            Some(expires_on) => {
                let margin = Duration::from_secs(TOKEN_USABLE_MARGIN_SECS);
                if Instant::now() + margin < expires_on {
                    Some(token)
                } else {
                    None
                }
            }
            None => Some(token),
        }
    }

    /// Returns true if the most recent read failed within the retry cooldown.
    fn recently_failed(&self) -> bool {
        let guard = match self.last_failure.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };
        match *guard {
            Some(failed_at) => failed_at.elapsed() < Duration::from_secs(TOKEN_REFRESH_RETRY_SECS),
            None => false,
        }
    }

    /// Reads a token and publishes it to consumers.
    async fn refresh_once(&self) -> Result<BearerToken, super::error::Error> {
        let start = Instant::now();
        match self.source.get_token().await {
            Ok(token) => {
                let latency_ms = start.elapsed().as_secs_f64() * 1_000.0;
                // `send_replace` updates the cache even with no active receivers.
                let _ = self.tx.send_replace(Some(token.clone()));
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.record_success(latency_ms);
                    metrics.record_publish();
                }
                if let Ok(mut f) = self.last_failure.lock() {
                    *f = None;
                }
                Ok(token)
            }
            Err(err) => {
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.record_failure();
                }
                if let Ok(mut f) = self.last_failure.lock() {
                    *f = Some(Instant::now());
                }
                Err(err)
            }
        }
    }

    /// Reads and publishes a token, returning the next timer instant (the
    /// source's `exp`-derived backstop on success, or a short retry delay on
    /// failure) and whether the read succeeded.
    async fn refresh_and_next(&self) -> (Option<tokio::time::Instant>, bool) {
        // Share the slow-path `fetch_lock` so a scheduled/watch-driven re-read
        // and a concurrent cache-miss coalesce onto one in-flight read.
        let _guard = self.fetch_lock.lock().await;
        match self.refresh_once().await {
            Ok(token) => (
                self.source
                    .next_refresh(&token)
                    .map(tokio::time::Instant::from_std),
                true,
            ),
            Err(error) => {
                otel_warn!(
                    "k8s_service_account_token.token_read_failed",
                    error = %error
                );
                (
                    Some(
                        tokio::time::Instant::now() + Duration::from_secs(TOKEN_REFRESH_RETRY_SECS),
                    ),
                    false,
                )
            }
        }
    }
}

/// Sets up a best-effort watcher on the token's mount directory.
///
/// kubelet rotates the projected token by atomically swapping a symlink in this
/// directory, which does not change the inode the token path resolves to, so we
/// watch the directory rather than the file. Any non-access change signals a
/// possible rotation; the receiver debounces bursts from one swap.
fn spawn_dir_watcher(
    dir: &Path,
) -> Result<(RecommendedWatcher, mpsc::UnboundedReceiver<()>), notify::Error> {
    let (tx, rx) = mpsc::unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            // Access-only events never change the token contents.
            if matches!(event.kind, EventKind::Access(_)) {
                return;
            }
            let _ = tx.send(());
        }
    })?;
    watcher.watch(dir, RecursiveMode::NonRecursive)?;
    Ok((watcher, rx))
}

/// Resolves when the timer at `next` elapses, or never when `next` is `None`.
async fn wait_timer(next: Option<tokio::time::Instant>) {
    match next {
        Some(instant) => tokio::time::sleep_until(instant).await,
        None => std::future::pending::<()>().await,
    }
}

/// Resolves when the directory watcher signals a change, coalescing a burst of
/// events from one atomic swap into a single wake-up. Never resolves once the
/// watcher is absent or its thread has gone away.
async fn wait_file_event(events: &mut Option<mpsc::UnboundedReceiver<()>>) {
    match events {
        Some(rx) => match rx.recv().await {
            Some(()) => while rx.try_recv().is_ok() {},
            None => {
                *events = None;
                std::future::pending::<()>().await;
            }
        },
        None => std::future::pending::<()>().await,
    }
}

#[async_trait]
impl SharedBearerTokenProvider for K8sServiceAccountTokenExtension {
    async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
        // Fast path: lock-free read of the watch cache.
        if let Some(token) = self.inner.current_fresh_token() {
            return Ok(token);
        }

        // Slow path: coalesce concurrent cache-miss callers onto a single
        // in-flight read, with a double-check after acquiring the lock.
        let _guard = self.inner.fetch_lock.lock().await;
        if let Some(token) = self.inner.current_fresh_token() {
            return Ok(token);
        }
        if self.inner.recently_failed() {
            return Err(self
                .inner
                .cap_err
                .error("token read throttled after recent failure"));
        }
        self.inner
            .refresh_once()
            .await
            .map_err(|err| self.inner.cap_err.error(err))
    }

    fn token_stream(&self) -> TokenStream {
        let rx = self.inner.tx.subscribe();
        // Yield the current cached value immediately, then each refresh; filter
        // out the initial (and any future) `None`.
        let stream = WatchStream::new(rx).filter_map(|opt| async move { opt });
        Box::pin(stream)
    }
}

#[async_trait]
impl SharedExtension for K8sServiceAccountTokenExtension {
    async fn start(
        self: Box<Self>,
        mut ctrl: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        let inner = Arc::clone(&self.inner);

        // Best-effort watch on the token's mount directory: the primary refresh
        // trigger. If setup fails, the `exp`-derived backstop timer alone still
        // keeps the token current.
        let (_watcher, mut file_events) = match inner.source.watch_dir() {
            Some(dir) => match spawn_dir_watcher(&dir) {
                Ok((watcher, rx)) => (Some(watcher), Some(rx)),
                Err(error) => {
                    otel_warn!(
                        "k8s_service_account_token.watch_setup_failed",
                        error = %error
                    );
                    (None, None)
                }
            },
            None => (None, None),
        };

        // Read immediately on startup.
        let mut next_refresh: Option<tokio::time::Instant> = Some(tokio::time::Instant::now());
        // Signal readiness only after the first token is published, so consumers
        // never observe an empty cache.
        let mut ready_signaled = false;

        loop {
            let mut trigger_refresh = false;
            tokio::select! {
                ctrl_msg = ctrl.recv() => {
                    match ctrl_msg {
                        Ok(ExtensionControlMsg::Shutdown { deadline, .. }) => {
                            let snapshot = inner.metrics.lock().ok().map(|m| m.snapshot());
                            return Ok(match snapshot {
                                Some(snapshot) => TerminalState::new(deadline, [snapshot]),
                                None => TerminalState::default(),
                            });
                        }
                        Err(_) => break,
                        Ok(ExtensionControlMsg::Config { .. }) => {}
                        Ok(ExtensionControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            if let Ok(mut metrics) = inner.metrics.lock() {
                                let _ = metrics.report(&mut metrics_reporter);
                            }
                        }
                    }
                }
                _ = wait_timer(next_refresh) => { trigger_refresh = true; }
                _ = wait_file_event(&mut file_events) => { trigger_refresh = true; }
            }

            if trigger_refresh {
                let (next, ok) = inner.refresh_and_next().await;
                next_refresh = next;
                if ok && !ready_signaled {
                    effect_handler.signal_ready();
                    ready_signaled = true;
                }
            }
        }

        Ok(TerminalState::default())
    }
}
