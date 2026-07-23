// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The Kubernetes SAT authorizer extension: `Arc<Inner>` state, the
//! `BearerTokenAuthorizer` capability implementation with a bounded decision
//! cache, and the client-initialization loop driven by the active
//! `Extension::start()` task.

use std::collections::HashSet;
use std::sync::Mutex;
use std::sync::{Arc, MutexGuard};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use otap_df_engine::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as BearerTokenAuthorizerCap;
use otap_df_engine::capability::auth::{
    AuthorizedIdentity, AuthzDecision, BearerToken, DenyReason,
};
use otap_df_engine::capability::{CapabilityError, CapabilityErrorSource};
use otap_df_engine::control::ExtensionControlMsg;
use otap_df_engine::error::Error as EngineError;
use otap_df_engine::extension::EffectHandler;
use otap_df_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as SharedBearerTokenAuthorizer;
use otap_df_engine::shared::extension::{ControlChannel, Extension as SharedExtension};
use otap_df_engine::terminal_state::TerminalState;
use otap_df_telemetry::otel_warn;
use tokio::sync::watch;

use super::metrics::K8sSatTokenAuthorizerMetricsTracker;
use super::reviewer::{ReviewOutcome, Reviewer};

/// Delay between attempts to construct the Kubernetes client during startup.
const CLIENT_INIT_RETRY_SECS: u64 = 10;

/// A cached admission decision and the instant it stops being valid.
struct CachedDecision {
    decision: AuthzDecision,
    expires_at: Instant,
}

/// Bounded, TTL'd cache of admission decisions keyed by the opaque token.
///
/// Keyed by the token string exactly as presented so a repeated request avoids
/// a `TokenReview` round-trip. Entries expire after `ttl`; the map is capped at
/// `max_entries` to bound memory.
struct DecisionCache {
    entries: std::collections::HashMap<String, CachedDecision>,
    ttl: Duration,
    max_entries: usize,
}

impl DecisionCache {
    fn new(ttl: Duration, max_entries: usize) -> Self {
        Self {
            entries: std::collections::HashMap::new(),
            ttl,
            max_entries,
        }
    }

    /// Returns the cached decision for `token` when present and unexpired.
    fn get(&self, token: &str, now: Instant) -> Option<AuthzDecision> {
        let entry = self.entries.get(token)?;
        if entry.expires_at > now {
            Some(entry.decision.clone())
        } else {
            None
        }
    }

    /// Inserts (or refreshes) the decision for `token`, evicting expired entries
    /// first and skipping the insert if still at capacity (so the cache never
    /// exceeds `max_entries`).
    fn insert(&mut self, token: String, decision: AuthzDecision, now: Instant) {
        let expires_at = now
            .checked_add(self.ttl)
            .unwrap_or_else(|| now + Duration::from_secs(1));
        let cached = CachedDecision {
            decision,
            expires_at,
        };

        // Updating an existing key never grows the map, so refresh it in place.
        if let Some(slot) = self.entries.get_mut(&token) {
            *slot = cached;
            return;
        }

        if self.entries.len() >= self.max_entries {
            // Reclaim space from expired entries before giving up on caching.
            self.entries.retain(|_, entry| entry.expires_at > now);
            if self.entries.len() >= self.max_entries {
                // Still full of live entries: skip caching rather than exceed
                // the bound. The decision is still returned to the caller.
                return;
            }
        }

        let _ = self.entries.insert(token, cached);
    }
}

/// Shared, clonable Kubernetes SAT authorizer extension.
///
/// Every clone (consumers + the initialization task) observes the same
/// [`Inner`] state via `Arc`, so they share one Kubernetes client, one decision
/// cache, and one metric set.
#[derive(Clone)]
pub struct K8sSatTokenAuthorizerExtension {
    inner: Arc<Inner>,
}

/// Shared state behind [`K8sSatTokenAuthorizerExtension`].
struct Inner {
    /// Audiences requested on every `TokenReview`.
    audiences: Vec<String>,
    /// The Kubernetes reviewer, published once the client is constructed in
    /// `start()`. `None` until then; a request arriving before readiness fails
    /// closed.
    reviewer: watch::Sender<Option<Arc<Reviewer>>>,
    /// Allow-list of admitted service-account usernames, or `None` to admit any
    /// authenticated account.
    allowed_service_accounts: Option<HashSet<String>>,
    /// Bounded, TTL'd decision cache. Its critical sections are short and never
    /// span an `.await`, so a `std` `Mutex` is appropriate.
    cache: Mutex<DecisionCache>,
    /// Pre-tagged capability error builder.
    cap_err: CapabilityErrorSource<BearerTokenAuthorizerCap>,
    /// Metric tracker. Its critical sections are short and never span an
    /// `.await`, so a `std` `Mutex` is appropriate.
    metrics: Mutex<K8sSatTokenAuthorizerMetricsTracker>,
}

impl K8sSatTokenAuthorizerExtension {
    /// Builds a new extension instance.
    #[must_use]
    pub fn new(
        name: &str,
        audiences: Vec<String>,
        allowed_service_accounts: Option<HashSet<String>>,
        cache_ttl: Duration,
        cache_max_entries: usize,
        metrics: K8sSatTokenAuthorizerMetricsTracker,
    ) -> Self {
        let (reviewer, _rx) = watch::channel(None);
        Self {
            inner: Arc::new(Inner {
                audiences,
                reviewer,
                allowed_service_accounts,
                cache: Mutex::new(DecisionCache::new(cache_ttl, cache_max_entries)),
                cap_err: CapabilityErrorSource::new(name.to_owned().into()),
                metrics: Mutex::new(metrics),
            }),
        }
    }
}

impl Inner {
    /// Locks the metric tracker, ignoring poisoning (a poisoned metrics lock
    /// must never fail an authorization decision).
    fn metrics(&self) -> Option<MutexGuard<'_, K8sSatTokenAuthorizerMetricsTracker>> {
        self.metrics.lock().ok()
    }

    /// Admits an authenticated identity against the configured allow-list,
    /// producing the final decision.
    fn admit(&self, username: Option<String>, audiences: Vec<String>) -> AuthzDecision {
        if let Some(allowed) = &self.allowed_service_accounts {
            match &username {
                Some(user) if allowed.contains(user) => {}
                _ => {
                    return AuthzDecision::deny_with_detail(
                        DenyReason::NotPermitted,
                        "service account not in allow-list",
                    );
                }
            }
        }

        let mut identity = AuthorizedIdentity::new();
        if let Some(user) = username {
            identity = identity.with_subject(&user);
        }
        // Surface the audience the token was accepted for so downstream routing
        // can use it; prefer the API server's confirmed audience.
        if let Some(audience) = audiences
            .into_iter()
            .next()
            .or_else(|| self.audiences.first().cloned())
        {
            identity = identity.with_audience(&audience);
        }
        AuthzDecision::allow(identity)
    }
}

#[async_trait]
impl SharedBearerTokenAuthorizer for K8sSatTokenAuthorizerExtension {
    async fn authorize(&self, credential: &BearerToken) -> Result<AuthzDecision, CapabilityError> {
        let inner = &self.inner;
        let token = credential.expose_token();

        // An empty credential is a missing credential (401); never round-trip it
        // to the API server.
        if token.is_empty() {
            let decision = AuthzDecision::deny(DenyReason::MissingCredential);
            if let Some(mut metrics) = inner.metrics() {
                metrics.record_deny();
            }
            return Ok(decision);
        }

        // Fast path: a still-valid cached decision avoids a TokenReview.
        let now = Instant::now();
        if let Some(decision) = inner
            .cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(token, now))
        {
            if let Some(mut metrics) = inner.metrics() {
                metrics.record_cache_hit();
                record_decision(&mut metrics, &decision);
            }
            return Ok(decision);
        }

        // The reviewer is published once the client is built in `start()`. If it
        // is not ready yet, fail closed: an undetermined decision must never
        // grant access.
        let reviewer = match inner.reviewer.borrow().clone() {
            Some(reviewer) => reviewer,
            None => {
                if let Some(mut metrics) = inner.metrics() {
                    metrics.record_error();
                }
                return Err(inner
                    .cap_err
                    .error("authorizer not ready: Kubernetes client is still initializing"));
            }
        };

        // Slow path: perform the TokenReview (no lock held across the await).
        let start = Instant::now();
        let outcome = reviewer.review(token).await;
        let latency_ms = start.elapsed().as_secs_f64() * 1_000.0;

        let outcome = match outcome {
            Ok(outcome) => {
                if let Some(mut metrics) = inner.metrics() {
                    metrics.record_token_review(latency_ms);
                }
                outcome
            }
            Err(err) => {
                if let Some(mut metrics) = inner.metrics() {
                    metrics.record_token_review(latency_ms);
                    metrics.record_error();
                }
                // Undetermined: the caller must fail closed. Do not cache.
                return Err(inner.cap_err.error(err));
            }
        };

        let decision = match outcome {
            ReviewOutcome::Authenticated {
                username,
                audiences,
            } => inner.admit(username, audiences),
            ReviewOutcome::Unauthenticated { error } => match error {
                Some(detail) => {
                    AuthzDecision::deny_with_detail(DenyReason::InvalidCredential, detail)
                }
                None => AuthzDecision::deny(DenyReason::InvalidCredential),
            },
        };

        // Cache the reached decision (allow or deny) and record it.
        if let Ok(mut cache) = inner.cache.lock() {
            cache.insert(token.to_owned(), decision.clone(), Instant::now());
        }
        if let Some(mut metrics) = inner.metrics() {
            record_decision(&mut metrics, &decision);
        }
        Ok(decision)
    }
}

/// Records the allow/deny counter for a reached decision.
fn record_decision(metrics: &mut K8sSatTokenAuthorizerMetricsTracker, decision: &AuthzDecision) {
    if decision.is_allowed() {
        metrics.record_allow();
    } else {
        metrics.record_deny();
    }
}

#[cfg(test)]
impl K8sSatTokenAuthorizerExtension {
    /// Runs the admission step against the configured allow-list. Test-only.
    pub(crate) fn admit_for_test(
        &self,
        username: Option<String>,
        audiences: Vec<String>,
    ) -> AuthzDecision {
        self.inner.admit(username, audiences)
    }

    /// Inserts a decision into the shared cache. Test-only.
    pub(crate) fn cache_insert_for_test(&self, token: &str, decision: AuthzDecision, now: Instant) {
        self.inner
            .cache
            .lock()
            .expect("cache lock")
            .insert(token.to_owned(), decision, now);
    }

    /// Reads a decision from the shared cache. Test-only.
    pub(crate) fn cache_get_for_test(&self, token: &str, now: Instant) -> Option<AuthzDecision> {
        self.inner.cache.lock().expect("cache lock").get(token, now)
    }

    /// Returns the number of live entries in the shared cache. Test-only.
    pub(crate) fn cache_len_for_test(&self) -> usize {
        self.inner.cache.lock().expect("cache lock").entries.len()
    }
}

#[async_trait]
impl SharedExtension for K8sSatTokenAuthorizerExtension {
    async fn start(
        self: Box<Self>,
        mut ctrl: ControlChannel,
        effect_handler: EffectHandler,
    ) -> Result<TerminalState, EngineError> {
        let inner = Arc::clone(&self.inner);

        // Build the Kubernetes client, retrying on a fixed cadence until it
        // succeeds or the engine shuts us down. The readiness probe holds
        // data-path node startup until we signal ready, so consumers never
        // observe a not-ready authorizer once the pipeline is running.
        loop {
            match Reviewer::try_new(inner.audiences.clone()).await {
                Ok(reviewer) => {
                    let _ = inner.reviewer.send_replace(Some(Arc::new(reviewer)));
                    effect_handler.signal_ready();
                    break;
                }
                Err(error) => {
                    otel_warn!(
                        "k8s_sat_token_authorizer.client_init_failed",
                        error = %error
                    );
                    // Race the retry delay against the control channel so a
                    // shutdown during initialization is honored promptly.
                    tokio::select! {
                        _ = tokio::time::sleep(Duration::from_secs(CLIENT_INIT_RETRY_SECS)) => {}
                        ctrl_msg = ctrl.recv() => {
                            match ctrl_msg {
                                Ok(ExtensionControlMsg::Shutdown { deadline, .. }) => {
                                    let snapshot = inner.metrics().map(|m| m.snapshot());
                                    return Ok(match snapshot {
                                        Some(snapshot) => TerminalState::new(deadline, [snapshot]),
                                        None => TerminalState::default(),
                                    });
                                }
                                Err(_) => return Ok(TerminalState::default()),
                                Ok(ExtensionControlMsg::Config { .. }) => {}
                                Ok(ExtensionControlMsg::CollectTelemetry {
                                    mut metrics_reporter,
                                }) => {
                                    if let Some(mut metrics) = inner.metrics() {
                                        let _ = metrics.report(&mut metrics_reporter);
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        // Steady state: the authorizer serves requests directly from the shared
        // state; this loop only services control messages (telemetry, shutdown).
        loop {
            match ctrl.recv().await {
                Ok(ExtensionControlMsg::Shutdown { deadline, .. }) => {
                    let snapshot = inner.metrics().map(|m| m.snapshot());
                    return Ok(match snapshot {
                        Some(snapshot) => TerminalState::new(deadline, [snapshot]),
                        None => TerminalState::default(),
                    });
                }
                Err(_) => break,
                Ok(ExtensionControlMsg::Config { .. }) => {}
                Ok(ExtensionControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    if let Some(mut metrics) = inner.metrics() {
                        let _ = metrics.report(&mut metrics_reporter);
                    }
                }
            }
        }

        Ok(TerminalState::default())
    }
}
