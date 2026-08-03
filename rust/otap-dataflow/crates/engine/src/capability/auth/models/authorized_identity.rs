// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The [`AuthorizedIdentity`] an authorizer emits when it admits a request.

use std::collections::BTreeMap;
use std::slice;

/// The value of a single claim.
///
/// Claims are heterogeneous: some are single-valued (`sub`, `aud`), others are
/// inherently multi-valued (`groups`, an X.509 certificate's repeated `OU`
/// attributes, or its Subject Alternative Names). [`ClaimValue`] captures both
/// without forcing every claim into a `Vec`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ClaimValue {
    /// A single-valued claim (e.g. `sub`, `aud`, `uid`).
    One(String),
    /// A multi-valued claim (e.g. `groups`, X.509 `OU`, SANs).
    Many(Vec<String>),
}

impl ClaimValue {
    /// Views the value as a slice, unifying [`One`](ClaimValue::One) (a
    /// single-element slice) and [`Many`](ClaimValue::Many). No allocation.
    #[must_use]
    pub fn as_slice(&self) -> &[String] {
        match self {
            ClaimValue::One(value) => slice::from_ref(value),
            ClaimValue::Many(values) => values.as_slice(),
        }
    }

    /// Returns the single string for a [`One`](ClaimValue::One) value, or `None`
    /// for a [`Many`](ClaimValue::Many).
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            ClaimValue::One(value) => Some(value.as_str()),
            ClaimValue::Many(_) => None,
        }
    }

    /// Returns `true` if `value` is present (for `One`, an exact match; for
    /// `Many`, membership).
    #[must_use]
    pub fn contains(&self, value: &str) -> bool {
        self.as_slice().iter().any(|v| v == value)
    }
}

/// The authenticated principal an authorizer identified while allowing a
/// request.
///
/// Scheme-agnostic by design, so one type serves every authentication method
/// (Kubernetes service-account tokens, OIDC/JWT, mutual TLS, ...). It has three
/// parts:
///
/// - `principal`: an optional canonical, human-facing name for the caller. It is
///   *best-effort* and its exact meaning is per-scheme (an SA username for a
///   Kubernetes SAT authorizer, `sub` for OIDC, a SPIFFE SAN or Subject DN for
///   mutual TLS). Use it for logging and coarse identity, not for policy
///   matching.
/// - `scheme`: an optional tag naming the authentication method that produced
///   the identity (e.g. `k8s_sat`, `oidc`, `mtls`), so a consumer knows which
///   claim vocabulary to expect.
/// - `claims`: the verified attributes, keyed by claim name. This is the single
///   source of truth a downstream tenant/authorization resolver matches on.
///   Names follow standard vocabularies where they exist (`sub`, `aud`, `iss`,
///   `groups` from JWT) and are otherwise namespaced by scheme (`k8s.namespace`,
///   `x509.subject.cn`, `x509.san.uri`, ...). Values are [`ClaimValue`] so
///   multi-valued claims (`groups`, SANs, repeated `OU`) are represented
///   faithfully.
///
/// `subject()`/`audience()` (and their `with_*` builders) are thin sugar over
/// the `sub`/`aud` claims, kept because those two are near-universal.
///
/// Marked `#[non_exhaustive]`: the identity is the input to downstream
/// per-tenant/route authorization and is expected to grow, so additions stay
/// non-breaking.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
#[non_exhaustive]
pub struct AuthorizedIdentity {
    principal: Option<String>,
    scheme: Option<String>,
    claims: BTreeMap<String, ClaimValue>,
}

impl AuthorizedIdentity {
    /// The `sub` (subject) claim name.
    pub const CLAIM_SUBJECT: &'static str = "sub";
    /// The `aud` (audience) claim name.
    pub const CLAIM_AUDIENCE: &'static str = "aud";

    /// Creates an empty identity (no principal, scheme, or claims).
    #[must_use]
    pub const fn new() -> Self {
        Self {
            principal: None,
            scheme: None,
            claims: BTreeMap::new(),
        }
    }

    /// Sets the canonical, best-effort principal name (for logging / coarse
    /// identity; not for policy matching).
    #[must_use]
    pub fn with_principal(mut self, principal: impl Into<String>) -> Self {
        self.principal = Some(principal.into());
        self
    }

    /// Sets the authentication scheme tag (e.g. `k8s_sat`, `oidc`, `mtls`).
    #[must_use]
    pub fn with_scheme(mut self, scheme: impl Into<String>) -> Self {
        self.scheme = Some(scheme.into());
        self
    }

    /// Sets a claim to an arbitrary [`ClaimValue`].
    #[must_use]
    pub fn with_claim(mut self, name: impl Into<String>, value: ClaimValue) -> Self {
        let _ = self.claims.insert(name.into(), value);
        self
    }

    /// Sets a single-valued claim.
    #[must_use]
    pub fn with_claim_str(self, name: impl Into<String>, value: impl Into<String>) -> Self {
        self.with_claim(name, ClaimValue::One(value.into()))
    }

    /// Sets a multi-valued claim.
    #[must_use]
    pub fn with_claim_values(self, name: impl Into<String>, values: Vec<String>) -> Self {
        self.with_claim(name, ClaimValue::Many(values))
    }

    /// Sets the `sub` claim (the principal the credential represents).
    #[must_use]
    pub fn with_subject(self, subject: impl Into<String>) -> Self {
        self.with_claim_str(Self::CLAIM_SUBJECT, subject)
    }

    /// Sets the `aud` claim (the audience the credential was accepted for).
    #[must_use]
    pub fn with_audience(self, audience: impl Into<String>) -> Self {
        self.with_claim_str(Self::CLAIM_AUDIENCE, audience)
    }

    /// The canonical principal name, if set.
    #[must_use]
    pub fn principal(&self) -> Option<&str> {
        self.principal.as_deref()
    }

    /// The authentication scheme tag, if set.
    #[must_use]
    pub fn scheme(&self) -> Option<&str> {
        self.scheme.as_deref()
    }

    /// The value of a claim by name, if present.
    #[must_use]
    pub fn claim(&self, name: &str) -> Option<&ClaimValue> {
        self.claims.get(name)
    }

    /// The single string value of a claim, if present and single-valued.
    #[must_use]
    pub fn claim_str(&self, name: &str) -> Option<&str> {
        self.claim(name).and_then(ClaimValue::as_str)
    }

    /// Iterates over all claims as `(name, value)` pairs, ordered by name.
    pub fn claims(&self) -> impl Iterator<Item = (&str, &ClaimValue)> {
        self.claims
            .iter()
            .map(|(name, value)| (name.as_str(), value))
    }

    /// The `sub` claim (the principal the credential represents), if known.
    #[must_use]
    pub fn subject(&self) -> Option<&str> {
        self.claim_str(Self::CLAIM_SUBJECT)
    }

    /// The `aud` claim (the audience the credential was accepted for), if known.
    #[must_use]
    pub fn audience(&self) -> Option<&str> {
        self.claim_str(Self::CLAIM_AUDIENCE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: build an identity via the `with_subject`/`with_audience` sugar
    /// and read it back through both the typed accessors and the generic claim
    /// map.
    /// Guarantees: `subject`/`audience` are stored as the `sub`/`aud` claims
    /// (single source of truth), so both access paths agree, and a `new`
    /// identity reports `None`/empty everywhere.
    #[test]
    fn subject_audience_are_sugar_over_claims() {
        let identity = AuthorizedIdentity::new()
            .with_subject("system:serviceaccount:default:my-sa")
            .with_audience("https://my-service.example");
        assert_eq!(
            identity.subject(),
            Some("system:serviceaccount:default:my-sa")
        );
        assert_eq!(identity.audience(), Some("https://my-service.example"));
        // Same values are visible through the generic claim accessors.
        assert_eq!(
            identity.claim_str("sub"),
            Some("system:serviceaccount:default:my-sa")
        );
        assert_eq!(
            identity.claim_str("aud"),
            Some("https://my-service.example")
        );

        let empty = AuthorizedIdentity::new();
        assert_eq!(empty.subject(), None);
        assert_eq!(empty.audience(), None);
        assert_eq!(empty.principal(), None);
        assert_eq!(empty.scheme(), None);
        assert_eq!(empty.claims().count(), 0);
    }

    /// Scenario: build an identity carrying a principal, scheme, single-valued
    /// and multi-valued claims (mirroring a Kubernetes SAT identity).
    /// Guarantees: principal/scheme round-trip, single- and multi-valued claims
    /// are readable via the map, and `Many` membership works.
    #[test]
    fn principal_scheme_and_multi_valued_claims_round_trip() {
        let identity = AuthorizedIdentity::new()
            .with_principal("system:serviceaccount:team-a:sender")
            .with_scheme("k8s_sat")
            .with_subject("system:serviceaccount:team-a:sender")
            .with_claim_str("k8s.namespace", "team-a")
            .with_claim_values(
                "groups",
                vec![
                    "system:serviceaccounts".to_string(),
                    "system:serviceaccounts:team-a".to_string(),
                ],
            );

        assert_eq!(
            identity.principal(),
            Some("system:serviceaccount:team-a:sender")
        );
        assert_eq!(identity.scheme(), Some("k8s_sat"));
        assert_eq!(identity.claim_str("k8s.namespace"), Some("team-a"));

        let groups = identity.claim("groups").expect("groups present");
        assert!(groups.contains("system:serviceaccounts:team-a"));
        assert!(!groups.contains("system:serviceaccounts:other"));
        // A multi-valued claim has no single string form.
        assert_eq!(groups.as_str(), None);
        assert_eq!(groups.as_slice().len(), 2);
    }

    /// Scenario: compare a `ClaimValue::One` against a `ClaimValue::Many`.
    /// Guarantees: `as_slice` unifies both as a slice (length 1 for `One`), and
    /// `contains` matches membership in each.
    #[test]
    fn claim_value_slice_and_contains() {
        let one = ClaimValue::One("only".to_string());
        assert_eq!(one.as_slice(), &["only".to_string()]);
        assert!(one.contains("only"));
        assert!(!one.contains("nope"));

        let many = ClaimValue::Many(vec!["a".to_string(), "b".to_string()]);
        assert_eq!(many.as_str(), None);
        assert!(many.contains("b"));
        assert!(!many.contains("c"));
    }
}
