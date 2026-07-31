// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared authorization logic used by both capability variants.
//!
//! [`Core`] holds the immutable, thread-safe state (audiences, lazily-built
//! Kubernetes client, admission policy) and performs the full authenticate +
//! admit decision for a cache miss. It carries no cache: each variant wraps its
//! own cache (a `Mutex` for the shared variant, a `RefCell` for the local one)
//! around this common logic.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use otap_df_engine::capability::auth::bearer_token_authorizer::BearerTokenAuthorizer as BearerTokenAuthorizerCap;
use otap_df_engine::capability::auth::{
    AuthorizedIdentity, AuthzDecision, BearerToken, ClaimValue, DenyReason,
};
use otap_df_engine::capability::{CapabilityError, CapabilityErrorSource};
use tokio::sync::OnceCell;

use super::cache::DecisionStore;
use super::config::{AudienceConfig, ResourceAttributesConfig};
use super::reviewer::{AccessOutcome, AuthenticatedUser, ReviewOutcome, Reviewer};

/// Authentication scheme tag emitted on every identity from this authorizer.
const SCHEME: &str = "k8s_sat";

/// How an authenticated identity is admitted, per audience.
pub(crate) enum Admission {
    /// Admit any authenticated service account (audience-only admission).
    Any,
    /// Admit only service accounts whose username is in the allow-list.
    AllowList(HashSet<String>),
    /// Admit only identities Kubernetes RBAC permits for the configured
    /// resource/verb, checked via `SubjectAccessReview`.
    Rbac(ResourceAttributesConfig),
}

impl Admission {
    /// Derives the admission strategy for a single audience entry.
    fn from_audience(entry: &AudienceConfig) -> Self {
        if let Some(attrs) = &entry.resource_attributes {
            Admission::Rbac(attrs.clone())
        } else if let Some(allow_list) = entry.allowed_service_account_set() {
            Admission::AllowList(allow_list)
        } else {
            Admission::Any
        }
    }
}

/// Immutable, thread-safe authorization core shared by both variants.
///
/// `Core` is `Send + Sync`, so the shared variant can place it behind an `Arc`
/// and the local variant behind an `Rc`; both share this one implementation of
/// the authenticate + admit logic and the lazily-built Kubernetes client.
pub(crate) struct Core {
    /// Union of all configured audiences, requested on every `TokenReview`.
    audiences: Vec<String>,
    /// Per-audience admission policy. Admission uses the entry for the audience
    /// `TokenReview` confirms, so an identity is only ever admitted for the
    /// audience it was minted for (no cross-tenant admission).
    admission_by_audience: HashMap<String, Admission>,
    /// The Kubernetes reviewer, built on first use. A failed build leaves the
    /// cell empty so the next request retries.
    reviewer: OnceCell<Arc<Reviewer>>,
    /// Pre-tagged capability error builder.
    cap_err: CapabilityErrorSource<BearerTokenAuthorizerCap>,
}

impl Core {
    /// Builds the core from the validated audience entries.
    ///
    /// Each entry contributes its audience to the `TokenReview` request and its
    /// own admission policy to the per-audience map.
    pub(crate) fn new(name: &str, audiences: Vec<AudienceConfig>) -> Self {
        let requested_audiences = audiences
            .iter()
            .map(|a| a.audience.trim().to_string())
            .collect();
        let admission_by_audience = audiences
            .iter()
            .map(|a| (a.audience.trim().to_string(), Admission::from_audience(a)))
            .collect();
        Self {
            audiences: requested_audiences,
            admission_by_audience,
            reviewer: OnceCell::new(),
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
        store.insert(token, decision.clone(), Instant::now());
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
            ReviewOutcome::Authenticated(user) => {
                // Admit against the entry for the confirmed audience, not a
                // global list, so a service account is only trusted for the
                // audience it was minted for. `match_audience` resolves the
                // single bound audience and fails closed on ambiguity.
                let audience = match self.match_audience(&user.audiences) {
                    Ok(audience) => audience,
                    Err(deny) => return Ok(deny),
                };
                let admission = &self.admission_by_audience[&audience];

                match admission {
                    // RBAC admission needs a second API call (SubjectAccessReview).
                    // A request failure is undetermined: fail closed.
                    Admission::Rbac(attrs) => {
                        match reviewer
                            .check_access(&user, attrs)
                            .await
                            .map_err(|err| self.cap_err.error(err))?
                        {
                            AccessOutcome::Allowed => Self::allow(&user, &audience),
                            AccessOutcome::Denied { reason } => match reason {
                                Some(detail) => AuthzDecision::deny_with_detail(
                                    DenyReason::NotPermitted,
                                    detail,
                                ),
                                None => AuthzDecision::deny(DenyReason::NotPermitted),
                            },
                        }
                    }
                    _ => Self::admit_non_rbac(&user, &audience, admission),
                }
            }
        };
        Ok(decision)
    }

    /// Selects the single configured audience among those `TokenReview`
    /// confirmed, or returns the deny decision to use.
    ///
    /// A token can be confirmed for several configured audiences at once; their
    /// order in `confirmed` is unspecified by Kubernetes. To keep policy
    /// selection deterministic and preserve cross-tenant isolation, this admits
    /// only an unambiguous single match:
    ///
    /// - exactly one configured audience matched -> `Ok(audience)`;
    /// - none matched -> `Err(Deny(NotPermitted, "token audience is not bound"))`;
    /// - two or more distinct configured audiences matched -> `Err(Deny(..))`
    ///   ("ambiguous policy"), failing closed rather than picking one tenant's
    ///   policy nondeterministically.
    fn match_audience(&self, confirmed: &[String]) -> Result<String, AuthzDecision> {
        // Distinct confirmed audiences that are configured. Dedup so a repeated
        // audience in the response is not mistaken for an ambiguous match.
        let mut matched: Vec<&str> = confirmed
            .iter()
            .map(String::as_str)
            .filter(|aud| self.admission_by_audience.contains_key(*aud))
            .collect();
        matched.sort_unstable();
        matched.dedup();

        match matched.as_slice() {
            [] => Err(AuthzDecision::deny_with_detail(
                DenyReason::NotPermitted,
                "token audience is not bound",
            )),
            [audience] => Ok((*audience).to_owned()),
            _ => Err(AuthzDecision::deny_with_detail(
                DenyReason::NotPermitted,
                "token confirmed for multiple bound audiences; ambiguous policy",
            )),
        }
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

    /// Builds the `Allow` identity for an authenticated user.
    ///
    /// Emits every claim the `TokenReview` verified so a downstream tenant /
    /// authorization resolver can match on them: the SA username as the
    /// `principal` and `sub` claim, the matched `aud`, the parsed
    /// `k8s.namespace` / `k8s.serviceaccount`, plus `uid`, `groups`, and any
    /// `extra` attributes (namespaced `extra.<key>`). Scheme is `k8s_sat`.
    fn allow(user: &AuthenticatedUser, audience: &str) -> AuthzDecision {
        let mut identity = AuthorizedIdentity::new()
            .with_scheme(SCHEME)
            .with_audience(audience);

        if let Some(username) = &user.username {
            identity = identity
                .with_principal(username)
                .with_subject(username);
            // The SA username encodes namespace + name; surface them as their
            // own claims so a resolver need not re-parse the username.
            if let Some((namespace, name)) = parse_service_account(username) {
                identity = identity
                    .with_claim_str("k8s.namespace", namespace)
                    .with_claim_str("k8s.serviceaccount", name);
            }
        }
        if let Some(uid) = &user.uid {
            identity = identity.with_claim_str("uid", uid);
        }
        if !user.groups.is_empty() {
            identity = identity.with_claim_values("groups", user.groups.iter().map(String::as_str));
        }
        for (key, values) in &user.extra {
            identity =
                identity.with_claim(&format!("extra.{key}"), ClaimValue::Many(values.clone()));
        }

        AuthzDecision::allow(identity)
    }

    /// Admits `user` for `audience` using the non-RBAC strategies (any /
    /// allow-list). The RBAC strategy needs an async API call and is handled in
    /// [`Core::decide`].
    fn admit_non_rbac(
        user: &AuthenticatedUser,
        audience: &str,
        admission: &Admission,
    ) -> AuthzDecision {
        match admission {
            Admission::Any => Self::allow(user, audience),
            Admission::AllowList(allowed) => match &user.username {
                Some(name) if allowed.contains(name) => Self::allow(user, audience),
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

/// Parses a service-account username into its `(namespace, name)` parts.
///
/// Kubernetes SA usernames have the form
/// `system:serviceaccount:<namespace>:<name>`. Returns `None` for any other
/// username shape (e.g. a non-SA user) or when either part is empty.
fn parse_service_account(username: &str) -> Option<(&str, &str)> {
    const PREFIX: &str = "system:serviceaccount:";
    let rest = username.strip_prefix(PREFIX)?;
    let (namespace, name) = rest.split_once(':')?;
    if namespace.is_empty() || name.is_empty() {
        return None;
    }
    Some((namespace, name))
}

#[cfg(test)]
impl Core {
    /// Runs the non-RBAC admission step for `audience`'s entry. Test-only.
    ///
    /// Returns a `NotPermitted` deny when the audience is not bound, mirroring
    /// the request path.
    pub(crate) fn admit_for_test(&self, username: Option<String>, audience: &str) -> AuthzDecision {
        let Some(admission) = self.admission_by_audience.get(audience) else {
            return AuthzDecision::deny_with_detail(
                DenyReason::NotPermitted,
                "token audience is not bound",
            );
        };
        let user = AuthenticatedUser {
            username,
            audiences: vec![audience.to_owned()],
            ..Default::default()
        };
        Self::admit_non_rbac(&user, audience, admission)
    }

    /// Builds the `Allow` identity for a fully-populated user. Test-only.
    pub(crate) fn allow_for_test(&self, user: &AuthenticatedUser, audience: &str) -> AuthzDecision {
        Self::allow(user, audience)
    }

    /// Resolves the confirmed audiences to a single bound audience, or the deny.
    /// Test-only wrapper over [`Core::match_audience`].
    pub(crate) fn match_audience_for_test(
        &self,
        confirmed: &[String],
    ) -> Result<String, AuthzDecision> {
        self.match_audience(confirmed)
    }
}
