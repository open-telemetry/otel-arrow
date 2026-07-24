// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared authorization logic used by both capability variants.
//!
//! [`Core`] holds the immutable, thread-safe state (audiences, lazily-built
//! Kubernetes client, admission policy) and performs the full authenticate +
//! admit decision for a cache miss. It carries no cache: each variant wraps its
//! own cache (a `Mutex` for the shared variant, a `RefCell` for the local one)
//! around this common logic.

use std::collections::HashSet;
use std::sync::Arc;
use std::time::Instant;

use otap_df_engine::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as BearerTokenAuthorizerCap;
use otap_df_engine::capability::auth::{
    AuthorizedIdentity, AuthzDecision, BearerToken, DenyReason,
};
use otap_df_engine::capability::{CapabilityError, CapabilityErrorSource};
use tokio::sync::OnceCell;

use super::cache::DecisionStore;
use super::config::ResourceAttributesConfig;
use super::reviewer::{AccessOutcome, AuthenticatedUser, ReviewOutcome, Reviewer};

/// How an authenticated identity is admitted.
pub(crate) enum Admission {
    /// Admit any authenticated service account (audience-only admission).
    Any,
    /// Admit only service accounts whose username is in the allow-list.
    AllowList(HashSet<String>),
    /// Admit only identities Kubernetes RBAC permits for the configured
    /// resource/verb, checked via `SubjectAccessReview`.
    Rbac(ResourceAttributesConfig),
}

/// Immutable, thread-safe authorization core shared by both variants.
///
/// `Core` is `Send + Sync`, so the shared variant can place it behind an `Arc`
/// and the local variant behind an `Rc`; both share this one implementation of
/// the authenticate + admit logic and the lazily-built Kubernetes client.
pub(crate) struct Core {
    /// Audiences requested on every `TokenReview`.
    audiences: Vec<String>,
    /// The Kubernetes reviewer, built on first use. A failed build leaves the
    /// cell empty so the next request retries.
    reviewer: OnceCell<Arc<Reviewer>>,
    /// How an authenticated identity is admitted.
    admission: Admission,
    /// Pre-tagged capability error builder.
    cap_err: CapabilityErrorSource<BearerTokenAuthorizerCap>,
}

impl Core {
    /// Builds the core from the validated configuration pieces.
    ///
    /// `allowed` and `resource_attributes` are mutually exclusive (enforced at
    /// config validation); when both are `None` any authenticated account is
    /// admitted.
    pub(crate) fn new(
        name: &str,
        audiences: Vec<String>,
        allowed: Option<HashSet<String>>,
        resource_attributes: Option<ResourceAttributesConfig>,
    ) -> Self {
        let admission = match (allowed, resource_attributes) {
            // RBAC takes precedence if somehow both are set; config validation
            // rejects that combination before we get here.
            (_, Some(attrs)) => Admission::Rbac(attrs),
            (Some(allow_list), None) => Admission::AllowList(allow_list),
            (None, None) => Admission::Any,
        };
        Self {
            audiences,
            reviewer: OnceCell::new(),
            admission,
            cap_err: CapabilityErrorSource::new(name.to_owned().into()),
        }
    }

    /// The decision for an empty credential: a missing credential (401).
    pub(crate) fn missing() -> AuthzDecision {
        AuthzDecision::deny(DenyReason::MissingCredential)
    }

    /// The full authorize flow shared by both capability variants.
    ///
    /// This is the single source of truth for the request path: empty-credential
    /// short-circuit, cache lookup, [`decide`](Self::decide) on a miss, then
    /// cache store. The only per-variant difference is `store`'s
    /// interior-mutability strategy (`Mutex` vs `RefCell`), injected via
    /// [`DecisionStore`]; the shared and local wrappers each call this and add no
    /// logic of their own, so the two variants cannot drift.
    pub(crate) async fn authorize(
        &self,
        credential: &BearerToken,
        store: &impl DecisionStore,
    ) -> Result<AuthzDecision, CapabilityError> {
        let token = credential.expose_token();

        // An empty credential is a missing credential (401); never round-trip it
        // to the API server.
        if token.is_empty() {
            return Ok(Self::missing());
        }

        // Fast path: a still-valid cached decision avoids a TokenReview.
        let now = Instant::now();
        if let Some(decision) = store.get(token, now) {
            return Ok(decision);
        }

        // Slow path: reach a decision (no lock/borrow held across the await) and
        // cache it.
        let decision = self.decide(token).await?;
        store.insert(token.to_owned(), decision.clone(), Instant::now());
        Ok(decision)
    }

    /// Reaches a decision for a non-empty `token` on a cache miss: builds the
    /// client if needed, authenticates via `TokenReview`, then admits.
    ///
    /// Returns [`CapabilityError`] only when no decision could be reached (client
    /// build failed, or a review request failed) so callers fail closed; a
    /// reached deny is `Ok(Deny{..})`.
    pub(crate) async fn decide(&self, token: &str) -> Result<AuthzDecision, CapabilityError> {
        // Lazily build the Kubernetes client on first use; a build failure is
        // undetermined, so fail closed and let the next request retry.
        let reviewer = self.reviewer().await?;

        // Perform the TokenReview (a request failure is undetermined).
        let outcome = reviewer
            .review(token)
            .await
            .map_err(|err| self.cap_err.error(err))?;

        let decision = match outcome {
            ReviewOutcome::Unauthenticated { error } => match error {
                Some(detail) => {
                    AuthzDecision::deny_with_detail(DenyReason::InvalidCredential, detail)
                }
                None => AuthzDecision::deny(DenyReason::InvalidCredential),
            },
            ReviewOutcome::Authenticated(user) => match &self.admission {
                // RBAC admission needs a second API call (SubjectAccessReview).
                // A request failure is undetermined: fail closed.
                Admission::Rbac(attrs) => {
                    match reviewer
                        .check_access(&user, attrs)
                        .await
                        .map_err(|err| self.cap_err.error(err))?
                    {
                        AccessOutcome::Allowed => self.allow(&user),
                        AccessOutcome::Denied { reason } => match reason {
                            Some(detail) => {
                                AuthzDecision::deny_with_detail(DenyReason::NotPermitted, detail)
                            }
                            None => AuthzDecision::deny(DenyReason::NotPermitted),
                        },
                    }
                }
                _ => self.admit_local(&user),
            },
        };
        Ok(decision)
    }

    /// Returns the lazily-built reviewer, constructing the Kubernetes client on
    /// first use. A construction failure is undetermined (fail closed) and
    /// leaves the cell empty so the next call retries.
    async fn reviewer(&self) -> Result<Arc<Reviewer>, CapabilityError> {
        self.reviewer
            .get_or_try_init(|| async {
                Reviewer::try_new(self.audiences.clone())
                    .await
                    .map(Arc::new)
                    .map_err(|e| self.cap_err.error(e))
            })
            .await
            .map(Arc::clone)
    }

    /// Builds the `Allow` identity for an authenticated user, carrying the SA
    /// subject and the audience it was accepted for.
    fn allow(&self, user: &AuthenticatedUser) -> AuthzDecision {
        let mut identity = AuthorizedIdentity::new();
        if let Some(subject) = &user.username {
            identity = identity.with_subject(subject);
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
    /// strategy needs an async API call and is handled in [`Core::decide`].
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

#[cfg(test)]
impl Core {
    /// Runs the non-RBAC admission step (any / allow-list). Test-only.
    pub(crate) fn admit_for_test(
        &self,
        username: Option<String>,
        audiences: Vec<String>,
    ) -> AuthzDecision {
        self.admit_local(&AuthenticatedUser {
            username,
            audiences,
            ..Default::default()
        })
    }
}
