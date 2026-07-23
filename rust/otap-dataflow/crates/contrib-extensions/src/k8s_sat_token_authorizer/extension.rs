// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The Kubernetes SAT authorizer extension: `Arc<Inner>` state and the
//! `BearerTokenAuthorizer` capability implementation with a bounded decision
//! cache.
//!
//! This is a **passive** extension: it runs no event loop. The Kubernetes
//! client is constructed lazily on the first `authorize()` call (a construction
//! or API-server failure is undetermined, so callers fail closed and the next
//! request retries).

use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use otap_df_engine::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as BearerTokenAuthorizerCap;
use otap_df_engine::capability::auth::{
    AuthorizedIdentity, AuthzDecision, BearerToken, DenyReason,
};
use otap_df_engine::capability::{CapabilityError, CapabilityErrorSource};
use otap_df_engine::shared::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as SharedBearerTokenAuthorizer;
use tokio::sync::OnceCell;

use super::config::ResourceAttributesConfig;
use super::reviewer::{AccessOutcome, AuthenticatedUser, ReviewOutcome, Reviewer};

/// How an authenticated identity is admitted.
enum Admission {
    /// Admit any authenticated service account (audience-only admission).
    Any,
    /// Admit only service accounts whose username is in the allow-list.
    AllowList(HashSet<String>),
    /// Admit only identities Kubernetes RBAC permits for the configured
    /// resource/verb, checked via `SubjectAccessReview`.
    Rbac(ResourceAttributesConfig),
}

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
/// Every clone (each consumer receives one) observes the same [`Inner`] state
/// via `Arc`, so they share one lazily-built Kubernetes client and one decision
/// cache.
#[derive(Clone)]
pub struct K8sSatTokenAuthorizerExtension {
    inner: Arc<Inner>,
}

/// Shared state behind [`K8sSatTokenAuthorizerExtension`].
struct Inner {
    /// Audiences requested on every `TokenReview`.
    audiences: Vec<String>,
    /// The Kubernetes reviewer, built on first use. A failed build leaves the
    /// cell empty so the next request retries.
    reviewer: OnceCell<Arc<Reviewer>>,
    /// How an authenticated identity is admitted.
    admission: Admission,
    /// Bounded, TTL'd decision cache. Its critical sections are short and never
    /// span an `.await`, so a `std` `Mutex` is appropriate.
    cache: Mutex<DecisionCache>,
    /// Pre-tagged capability error builder.
    cap_err: CapabilityErrorSource<BearerTokenAuthorizerCap>,
}

impl K8sSatTokenAuthorizerExtension {
    /// Builds a new extension instance.
    ///
    /// `allowed_service_accounts` and `resource_attributes` are mutually
    /// exclusive (enforced at config validation); when both are `None` any
    /// authenticated account is admitted.
    #[must_use]
    pub fn new(
        name: &str,
        audiences: Vec<String>,
        allowed_service_accounts: Option<HashSet<String>>,
        resource_attributes: Option<ResourceAttributesConfig>,
        cache_ttl: Duration,
        cache_max_entries: usize,
    ) -> Self {
        let admission = match (allowed_service_accounts, resource_attributes) {
            // RBAC takes precedence if somehow both are set; config validation
            // rejects that combination before we get here.
            (_, Some(attrs)) => Admission::Rbac(attrs),
            (Some(allow_list), None) => Admission::AllowList(allow_list),
            (None, None) => Admission::Any,
        };
        Self {
            inner: Arc::new(Inner {
                audiences,
                reviewer: OnceCell::new(),
                admission,
                cache: Mutex::new(DecisionCache::new(cache_ttl, cache_max_entries)),
                cap_err: CapabilityErrorSource::new(name.to_owned().into()),
            }),
        }
    }

    /// Returns the lazily-built reviewer, constructing the Kubernetes client on
    /// first use. A construction failure is undetermined (fail closed) and
    /// leaves the cell empty so the next call retries.
    async fn reviewer(&self) -> Result<Arc<Reviewer>, CapabilityError> {
        let inner = &self.inner;
        inner
            .reviewer
            .get_or_try_init(|| async {
                Reviewer::try_new(inner.audiences.clone())
                    .await
                    .map(Arc::new)
                    .map_err(|e| inner.cap_err.error(e))
            })
            .await
            .map(Arc::clone)
    }
}

impl Inner {
    /// Builds the `Allow` identity for an authenticated user, carrying the SA
    /// subject and the audience it was accepted for.
    fn allow(&self, user: &AuthenticatedUser) -> AuthzDecision {
        let mut identity = AuthorizedIdentity::new();
        if let Some(user) = &user.username {
            identity = identity.with_subject(user);
        }
        // Surface the audience the token was accepted for so downstream routing
        // can use it; prefer the API server's confirmed audience.
        if let Some(audience) = user
            .audiences
            .first()
            .cloned()
            .or_else(|| self.audiences.first().cloned())
        {
            identity = identity.with_audience(&audience);
        }
        AuthzDecision::allow(identity)
    }

    /// Admits `user` using the non-RBAC strategies (any / allow-list). The RBAC
    /// strategy needs an async API call and is handled in `authorize`.
    fn admit_local(&self, user: &AuthenticatedUser) -> AuthzDecision {
        match &self.admission {
            Admission::Any => self.allow(user),
            Admission::AllowList(allowed) => match &user.username {
                Some(name) if allowed.contains(name) => self.allow(user),
                _ => AuthzDecision::deny_with_detail(
                    DenyReason::NotPermitted,
                    "service account not in allow-list",
                ),
            },
            // Handled by the async RBAC path; never reached here.
            Admission::Rbac(_) => AuthzDecision::deny(DenyReason::NotPermitted),
        }
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
            return Ok(AuthzDecision::deny(DenyReason::MissingCredential));
        }

        // Fast path: a still-valid cached decision avoids a TokenReview.
        let now = Instant::now();
        if let Some(decision) = inner
            .cache
            .lock()
            .ok()
            .and_then(|cache| cache.get(token, now))
        {
            return Ok(decision);
        }

        // Lazily build the Kubernetes client on first use; a build failure is
        // undetermined, so fail closed and let the next request retry.
        let reviewer = self.reviewer().await?;

        // Perform the TokenReview (no lock held across the await). A request
        // failure is undetermined: fail closed and do not cache.
        let outcome = reviewer
            .review(token)
            .await
            .map_err(|err| inner.cap_err.error(err))?;

        let decision = match outcome {
            ReviewOutcome::Unauthenticated { error } => match error {
                Some(detail) => {
                    AuthzDecision::deny_with_detail(DenyReason::InvalidCredential, detail)
                }
                None => AuthzDecision::deny(DenyReason::InvalidCredential),
            },
            ReviewOutcome::Authenticated(user) => match &inner.admission {
                // RBAC admission needs a second API call (SubjectAccessReview).
                // A request failure is undetermined: fail closed, do not cache.
                Admission::Rbac(attrs) => {
                    match reviewer
                        .check_access(&user, attrs)
                        .await
                        .map_err(|err| inner.cap_err.error(err))?
                    {
                        AccessOutcome::Allowed => inner.allow(&user),
                        AccessOutcome::Denied { reason } => match reason {
                            Some(detail) => {
                                AuthzDecision::deny_with_detail(DenyReason::NotPermitted, detail)
                            }
                            None => AuthzDecision::deny(DenyReason::NotPermitted),
                        },
                    }
                }
                _ => inner.admit_local(&user),
            },
        };

        // Cache the reached decision (allow or deny).
        if let Ok(mut cache) = inner.cache.lock() {
            cache.insert(token.to_owned(), decision.clone(), Instant::now());
        }
        Ok(decision)
    }
}

#[cfg(test)]
impl K8sSatTokenAuthorizerExtension {
    /// Runs the non-RBAC admission step (any / allow-list). Test-only.
    pub(crate) fn admit_for_test(
        &self,
        username: Option<String>,
        audiences: Vec<String>,
    ) -> AuthzDecision {
        self.inner.admit_local(&AuthenticatedUser {
            username,
            audiences,
            ..Default::default()
        })
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
