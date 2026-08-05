// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The [`AuthzDecision`] an authorizer returns for an admission check.

use super::{AuthorizedIdentity, DenyReason};

/// The outcome of an admission decision.
///
/// A [`Deny`](AuthzDecision::Deny) is a **successful** decision, not an error:
/// the authorizer reached a verdict and the answer was "no". An error from an
/// authorizer's `authorize` call means it could not reach a decision at all.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthzDecision {
    /// The request is admitted, carrying the authenticated identity so a
    /// consumer can propagate it downstream (e.g. for multi-tenant routing).
    Allow {
        /// The authenticated principal the authorizer identified.
        identity: AuthorizedIdentity,
    },
    /// The request is denied.
    Deny {
        /// Coarse, low-cardinality category -- safe to use as a metric label.
        reason: DenyReason,
        /// Optional human-readable detail for logs only; never use as a metric
        /// label. Safe to surface to operators, but do not leak policy
        /// internals to untrusted callers.
        detail: Option<String>,
    },
}

impl AuthzDecision {
    /// Constructs an [`Allow`](AuthzDecision::Allow) decision carrying `identity`.
    #[must_use]
    pub const fn allow(identity: AuthorizedIdentity) -> Self {
        Self::Allow { identity }
    }

    /// Constructs an [`Allow`](AuthzDecision::Allow) decision with an empty
    /// identity, for authorizers that allow without a meaningful principal.
    #[must_use]
    pub const fn allow_anonymous() -> Self {
        Self::Allow {
            identity: AuthorizedIdentity::new(),
        }
    }

    /// Constructs a [`Deny`](AuthzDecision::Deny) with a coarse `reason` and no
    /// detail.
    #[must_use]
    pub const fn deny(reason: DenyReason) -> Self {
        Self::Deny {
            reason,
            detail: None,
        }
    }

    /// Constructs a [`Deny`](AuthzDecision::Deny) with a coarse `reason` and a
    /// human-readable `detail` (for logs only -- never a metric label).
    #[must_use]
    pub fn deny_with_detail(reason: DenyReason, detail: impl Into<String>) -> Self {
        Self::Deny {
            reason,
            detail: Some(detail.into()),
        }
    }

    /// Returns `true` for any [`Allow`](AuthzDecision::Allow).
    #[must_use]
    pub const fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    /// The authenticated identity when allowed, or `None` when denied.
    #[must_use]
    pub const fn identity(&self) -> Option<&AuthorizedIdentity> {
        match self {
            Self::Allow { identity } => Some(identity),
            Self::Deny { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: build Allow (with identity and anonymous) and Deny (with and
    /// without detail) decisions and inspect them via the accessors.
    /// Guarantees: `is_allowed`/`identity` reflect the variant, the identity is
    /// carried through Allow, and Deny stores the exact reason and optional
    /// detail while reporting no identity.
    #[test]
    fn decision_constructors_and_predicate() {
        let identity = AuthorizedIdentity::new()
            .with_subject("system:serviceaccount:default:my-sa")
            .with_audience("https://my-service.example");
        let allowed = AuthzDecision::allow(identity.clone());
        assert!(allowed.is_allowed());
        assert_eq!(allowed.identity(), Some(&identity));
        assert_eq!(
            allowed.identity().and_then(|i| i.subject()),
            Some("system:serviceaccount:default:my-sa")
        );

        assert!(AuthzDecision::allow_anonymous().is_allowed());
        assert_eq!(
            AuthzDecision::allow_anonymous().identity(),
            Some(&AuthorizedIdentity::new())
        );

        assert!(!AuthzDecision::deny(DenyReason::MissingCredential).is_allowed());
        assert!(
            !AuthzDecision::deny_with_detail(DenyReason::NotPermitted, "rbac_failed").is_allowed()
        );
        assert_eq!(
            AuthzDecision::deny(DenyReason::InvalidCredential).identity(),
            None
        );

        assert_eq!(
            AuthzDecision::deny_with_detail(DenyReason::NotPermitted, "nope"),
            AuthzDecision::Deny {
                reason: DenyReason::NotPermitted,
                detail: Some("nope".to_owned())
            }
        );
        assert_eq!(
            AuthzDecision::deny(DenyReason::MissingCredential),
            AuthzDecision::Deny {
                reason: DenyReason::MissingCredential,
                detail: None
            }
        );
    }
}
