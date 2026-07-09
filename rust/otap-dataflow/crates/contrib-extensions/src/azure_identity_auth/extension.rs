// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The Azure Identity Auth extension: `Arc<Inner>` state, the
//! `BearerTokenProvider` capability implementation, and the background refresh
//! loop driven by the active `Extension::start()` task.

use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::StreamExt;
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
use tokio::sync::watch;
use tokio_stream::wrappers::WatchStream;

use super::auth::Auth;
use super::metrics::AzureIdentityAuthMetricsTracker;

/// Refresh this many seconds before `expires_on`.
const TOKEN_EXPIRY_BUFFER_SECS: u64 = 299;
/// Safety margin before actual expiry within which a cached token is treated as
/// no longer usable. Deliberately much smaller than `TOKEN_EXPIRY_BUFFER_SECS`:
/// the background loop refreshes ~5 min early, but if that refresh is failing a
/// still-valid token should keep being served (not treated as unusable 5 min
/// early), which also avoids stampeding the token endpoint during a transient
/// outage.
const TOKEN_USABLE_MARGIN_SECS: u64 = 30;
/// Floor between successful refreshes; avoids busy-looping on near-expired
/// tokens.
const MIN_TOKEN_REFRESH_INTERVAL_SECS: u64 = 10;
/// Reschedule delay after a failed acquisition.
const TOKEN_REFRESH_RETRY_SECS: u64 = 10;
/// Next-refresh delay used for non-expiring tokens (~1 year). The loop is still
/// woken by control messages in the meantime.
const NON_EXPIRING_REFRESH_SECS: u64 = 365 * 24 * 60 * 60;

/// Shared, clonable Azure Identity Auth extension.
///
/// Every clone (consumers + the background refresh task) observes the same
/// [`Inner`] state via `Arc`, so they share one token cache and refresh loop.
#[derive(Clone)]
pub struct AzureIdentityAuthExtension {
    inner: Arc<Inner>,
}

/// Shared state behind [`AzureIdentityAuthExtension`].
struct Inner {
    /// Azure credential + scope used to acquire tokens.
    auth: Auth,
    /// Token cache + pub/sub for `token_stream()`.
    tx: watch::Sender<Option<BearerToken>>,
    /// Pre-tagged capability error builder.
    cap_err: CapabilityErrorSource<BearerTokenProviderCap>,
    /// Coalesces concurrent slow-path fetches onto one in-flight request.
    fetch_lock: tokio::sync::Mutex<()>,
    /// Instant of the most recent failed acquisition (negative cache). Used to
    /// throttle slow-path retries so a failing token endpoint is not stampeded.
    last_failure: Mutex<Option<Instant>>,
    /// Metric tracker. Its critical sections are short and never span an
    /// `.await`, so a `std` `Mutex` is appropriate.
    metrics: Mutex<AzureIdentityAuthMetricsTracker>,
}

impl AzureIdentityAuthExtension {
    /// Builds a new extension instance.
    #[must_use]
    pub fn new(
        name: &str,
        auth: Auth,
        tx: watch::Sender<Option<BearerToken>>,
        metrics: AzureIdentityAuthMetricsTracker,
    ) -> Self {
        Self {
            inner: Arc::new(Inner {
                auth,
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
    /// Returns the cached token if it is present and still comfortably before
    /// its expiry (outside the usability safety margin).
    fn current_fresh_token(&self) -> Option<BearerToken> {
        // The token lives inside the watch channel behind a temporary read
        // guard; clone it out so we can return an owned value (and release the
        // guard, which would otherwise block the writer). `BearerToken` clones
        // are cheap: a refcount bump on the shared secret.
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

    /// Returns true if the most recent acquisition failed within the retry
    /// cooldown window. Used as a negative cache to throttle slow-path retries.
    fn recently_failed(&self) -> bool {
        // Open the shared box holding the last-failure timestamp. If the lock
        // is somehow poisoned, treat it as "no recent failure" and allow a
        // retry rather than failing here.
        let guard = match self.last_failure.lock() {
            Ok(guard) => guard,
            Err(_) => return false,
        };

        // If a failure timestamp is recorded, we are throttling only while it
        // is still within the cooldown window; otherwise (no failure recorded)
        // we are not throttling.
        match *guard {
            Some(failed_at) => failed_at.elapsed() < Duration::from_secs(TOKEN_REFRESH_RETRY_SECS),
            None => false,
        }
    }

    /// Acquires a token and publishes it to consumers.
    async fn refresh_once(&self) -> Result<BearerToken, super::error::Error> {
        let start = Instant::now();
        match self.auth.get_token().await {
            Ok(token) => {
                let latency_ms = start.elapsed().as_secs_f64() * 1_000.0;
                // Publish the token to consumers and update the cache. Using
                // `send_replace` (rather than `send`) ensures the cache is
                // updated even when no receivers are currently subscribed.
                let _ = self.tx.send_replace(Some(token.clone()));
                // Record success + publish under a single metrics lock.
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.record_success(latency_ms);
                    metrics.record_publish();
                }
                // Clear the negative cache: acquisitions are healthy again.
                if let Ok(mut f) = self.last_failure.lock() {
                    *f = None;
                }
                Ok(token)
            }
            Err(err) => {
                if let Ok(mut metrics) = self.metrics.lock() {
                    metrics.record_failure();
                }
                // Record the failure instant so the slow path can throttle
                // further attempts until the cooldown elapses.
                if let Ok(mut f) = self.last_failure.lock() {
                    *f = Some(Instant::now());
                }
                Err(err)
            }
        }
    }
}

/// Computes the next refresh instant from a freshly acquired token.
///
/// Refreshes `TOKEN_EXPIRY_BUFFER_SECS` before expiry, but never sooner than
/// `MIN_TOKEN_REFRESH_INTERVAL_SECS` from now; a non-expiring token pushes the
/// next refresh far into the future (the loop is still woken by control
/// messages in the meantime).
pub(crate) fn schedule_next(token: &BearerToken) -> tokio::time::Instant {
    let now = tokio::time::Instant::now();
    let min_next = now + Duration::from_secs(MIN_TOKEN_REFRESH_INTERVAL_SECS);
    match token.expires_on() {
        Some(expires_on) => {
            let target = tokio::time::Instant::from_std(expires_on)
                .checked_sub(Duration::from_secs(TOKEN_EXPIRY_BUFFER_SECS))
                .unwrap_or(now);
            target.max(min_next)
        }
        None => now + Duration::from_secs(NON_EXPIRING_REFRESH_SECS),
    }
}

#[async_trait]
impl SharedBearerTokenProvider for AzureIdentityAuthExtension {
    async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
        // Fast path: lock-free read of the watch cache.
        if let Some(token) = self.inner.current_fresh_token() {
            return Ok(token);
        }

        // Slow path: coalesce concurrent cache-miss callers onto a single
        // in-flight credential call, with a double-check after acquiring the
        // lock.
        let _guard = self.inner.fetch_lock.lock().await;
        if let Some(token) = self.inner.current_fresh_token() {
            return Ok(token);
        }
        // Negative cache: if the most recent acquisition failed within the
        // cooldown window, surface the throttle instead of hitting the token
        // endpoint again. The background loop keeps retrying on its own cadence.
        if self.inner.recently_failed() {
            return Err(self
                .inner
                .cap_err
                .error("token acquisition throttled after recent failure"));
        }
        self.inner
            .refresh_once()
            .await
            .map_err(|err| self.inner.cap_err.error(err))
    }

    fn token_stream(&self) -> TokenStream {
        let rx = self.inner.tx.subscribe();
        // Yield the current cached value immediately, then each refresh. The
        // initial `None` (and any future `None`) is filtered out. The stream
        // item is a plain `BearerToken`: a refresh failure does not terminate
        // the subscription, it simply does not emit until the next success.
        let stream = WatchStream::new(rx).filter_map(|opt| async move { opt });
        Box::pin(stream)
    }
}

#[async_trait]
impl SharedExtension for AzureIdentityAuthExtension {
    async fn start(
        self: Box<Self>,
        mut ctrl: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        let inner = Arc::clone(&self.inner);
        // Refresh immediately on startup.
        let mut next_refresh = tokio::time::Instant::now();
        // The engine holds data-path node startup until we signal readiness
        // (see `with_readiness_probe`). Fire once, after the first token is
        // published, so consumers never observe an empty cache.
        let mut ready_signaled = false;

        loop {
            tokio::select! {
                ctrl_msg = ctrl.recv() => {
                    match ctrl_msg {
                        // Graceful shutdown: return the final metric snapshot in
                        // the terminal state (the same contract nodes follow).
                        Ok(ExtensionControlMsg::Shutdown { deadline, .. }) => {
                            let snapshot = inner.metrics.lock().ok().map(|m| m.snapshot());
                            return Ok(match snapshot {
                                Some(snapshot) => TerminalState::new(deadline, [snapshot]),
                                None => TerminalState::default(),
                            });
                        }
                        // Control channel closed: exit without a snapshot.
                        Err(_) => break,
                        // Refresh cadence is governed by token lifetime; live
                        // reconfiguration is a no-op in v1.
                        Ok(ExtensionControlMsg::Config { .. }) => {}
                        Ok(ExtensionControlMsg::CollectTelemetry { mut metrics_reporter }) => {
                            if let Ok(mut metrics) = inner.metrics.lock() {
                                let _ = metrics.report(&mut metrics_reporter);
                            }
                        }
                    }
                }
                _ = tokio::time::sleep_until(next_refresh) => {
                    // Take the same `fetch_lock` the slow-path `get_token` uses,
                    // so a scheduled refresh and a concurrent cache-miss fetch
                    // coalesce onto one in-flight credential call instead of
                    // both hitting the token endpoint.
                    let _guard = inner.fetch_lock.lock().await;
                    match inner.refresh_once().await {
                        Ok(token) => {
                            next_refresh = schedule_next(&token);
                            if !ready_signaled {
                                effect_handler.signal_ready();
                                ready_signaled = true;
                            }
                        }
                        Err(error) => {
                            otel_warn!(
                                "azure_identity_auth.token_refresh_failed",
                                error = %error
                            );
                            next_refresh = tokio::time::Instant::now()
                                + Duration::from_secs(TOKEN_REFRESH_RETRY_SECS);
                        }
                    }
                }
            }
        }

        Ok(TerminalState::default())
    }
}
