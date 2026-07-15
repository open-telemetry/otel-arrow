// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `BearerTokenAuthorizer` capability.
//!
//! The request-facing access-control capability for data-path nodes (typically
//! receivers) that authenticate callers with an OAuth/OIDC-style **bearer
//! token**. Given the token presented on an inbound request, it returns a
//! single allow/deny [`AuthzDecision`].
//!
//! A `BearerTokenAuthorizer` performs **both** authentication (establishing who
//! the caller is from the token) and authorization (deciding whether that
//! caller may act), behind one call, so a receiver depends on this single
//! capability rather than orchestrating the steps itself. For example, a
//! Kubernetes service-account-token authorizer validates the token via the
//! `TokenReview` API (authentication) and then checks the returned service
//! account against a configured allow-list (authorization) — deriving both the
//! trust source and the allowed identities from its own configuration, so the
//! caller supplies only the token.
//!
//! This capability is bearer-specific by design (the credential is always a
//! token string) and **transport- and library-agnostic**: the token is carried
//! by [`BearerToken`], a secret-protecting wrapper built from plain `&str`
//! (a bare token or a whole `Authorization` header value), never from any
//! HTTP/RPC crate's request type. A receiver extracts it from whatever
//! transport it uses (gRPC, HTTP, …).
//!
//! The `#[capability]` proc macro expands the trait into:
//!
//! - `pub(crate) mod local::BearerTokenAuthorizer` (`!Send` trait variant)
//! - `pub(crate) mod shared::BearerTokenAuthorizer` (`Send` trait variant)
//! - A `SharedAsLocalBearerTokenAuthorizer` adapter
//! - A zero-sized `pub struct BearerTokenAuthorizer` registration handle
//! - `local_entry::<E>` / `shared_entry::<E>` factory bridges
//! - A `KNOWN_CAPABILITIES` distributed-slice entry

use super::error::CapabilityError;
use otap_df_engine_macros::capability;
use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;

/// A bearer token presented by a caller on an inbound request.
///
/// The counterpart to [`BearerToken`](super::bearer_token_provider::BearerToken)
/// on the *inbound* side: where that type is a token an extension *hands out*
/// for outbound requests (and carries an expiry), this is a token a caller
/// *presents* for validation, so it carries **no expiry** — freshness is the
/// authorizer's concern, decided while validating.
///
/// The secret is wrapped in [`SecretString`], which zeroizes on drop and masks
/// itself in [`Debug`] output, so a presented credential cannot leak into logs
/// or telemetry. The `SecretString` sits behind an [`Arc`] so cloning the
/// credential is a cheap refcount bump rather than copying the secret bytes.
#[derive(Clone, Debug)]
pub struct BearerToken {
    secret: Arc<SecretString>,
}

impl BearerToken {
    /// Creates a credential from a bare token — the token value alone, without
    /// any `Bearer ` scheme prefix.
    ///
    /// Accepts anything convertible into [`SecretString`] (e.g. a `String`),
    /// which is then shared behind an [`Arc`].
    #[must_use]
    pub fn new(token: impl Into<SecretString>) -> Self {
        Self {
            secret: Arc::new(token.into()),
        }
    }

    /// Creates a credential from a whole `Authorization` header value.
    ///
    /// Strips a leading, case-insensitive `Bearer ` scheme prefix when present
    /// (e.g. `"Bearer eyJ…"` → `"eyJ…"`), and otherwise treats the trimmed value
    /// as the token verbatim. Surrounding whitespace is trimmed either way.
    ///
    /// This is a convenience for callers that hold the raw header value; a
    /// caller that already has the bare token should use [`new`](Self::new).
    #[must_use]
    pub fn from_header_value(header_value: &str) -> Self {
        let token = strip_bearer_prefix(header_value).unwrap_or_else(|| header_value.trim());
        Self::new(token.to_owned())
    }

    /// Exposes the token secret, for the authorizer to validate.
    ///
    /// Named `expose_token` (rather than a plain getter) so every plaintext
    /// access is explicit and greppable.
    #[must_use]
    pub fn expose_token(&self) -> &str {
        self.secret.expose_secret()
    }
}

/// Strips a leading, case-insensitive `Bearer ` scheme from an `Authorization`
/// header value, returning the trimmed token. Returns `None` when no `Bearer `
/// scheme is present (including a bare token with no scheme), so the caller can
/// fall back to treating the whole value as the token.
fn strip_bearer_prefix(header_value: &str) -> Option<&str> {
    let trimmed = header_value.trim();
    let (scheme, rest) = trimmed.split_once(char::is_whitespace)?;
    if scheme.eq_ignore_ascii_case("Bearer") {
        Some(rest.trim())
    } else {
        None
    }
}

/// The authenticated principal an authorizer identified while allowing a
/// request.
///
/// Deliberately scheme-agnostic: `subject` is the principal the credential
/// represents and `audience` is the audience it was accepted for. For a
/// Kubernetes service-account-token authorizer, `subject` is the SA username
/// (`system:serviceaccount:<namespace>:<name>`) and `audience` is the matched
/// audience; other authorizers populate them from their own token model. Both
/// are optional so an authorizer that allows without a meaningful identity (e.g.
/// a static allow-list) can return an empty identity.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct AuthorizedIdentity {
    subject: Option<String>,
    audience: Option<String>,
}

impl AuthorizedIdentity {
    /// Creates an empty identity (no subject, no audience).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the subject (the principal the credential represents).
    #[must_use]
    pub fn with_subject(mut self, subject: impl Into<String>) -> Self {
        self.subject = Some(subject.into());
        self
    }

    /// Sets the audience the credential was accepted for.
    #[must_use]
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    /// The principal the credential represents, if known.
    #[must_use]
    pub fn subject(&self) -> Option<&str> {
        self.subject.as_deref()
    }

    /// The audience the credential was accepted for, if known.
    #[must_use]
    pub fn audience(&self) -> Option<&str> {
        self.audience.as_deref()
    }
}

/// The outcome of an authorization decision.
///
/// A [`Deny`](AuthzDecision::Deny) is a **successful** decision, not an error:
/// the authorizer reached a verdict and the answer was "no". An error from
/// [`BearerTokenAuthorizer::authorize`] means the authorizer could not reach a
/// decision at all (see that method's docs).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthzDecision {
    /// The request is permitted, carrying the authenticated identity so a
    /// consumer can propagate it downstream (e.g. for multi-tenant routing).
    Allow {
        /// The authenticated principal the authorizer identified.
        identity: AuthorizedIdentity,
    },
    /// The request is denied, optionally with a human-readable reason (safe to
    /// surface to operators; do not leak policy internals to untrusted callers).
    Deny {
        /// Why the request was denied, if the authorizer supplied a reason.
        reason: Option<String>,
    },
}

impl AuthzDecision {
    /// Constructs an [`Allow`](AuthzDecision::Allow) decision carrying `identity`.
    #[must_use]
    pub fn allow(identity: AuthorizedIdentity) -> Self {
        Self::Allow { identity }
    }

    /// Constructs an [`Allow`](AuthzDecision::Allow) decision with an empty
    /// identity, for authorizers that allow without a meaningful principal.
    #[must_use]
    pub fn allow_anonymous() -> Self {
        Self::Allow {
            identity: AuthorizedIdentity::new(),
        }
    }

    /// Constructs a [`Deny`](AuthzDecision::Deny) decision with a reason.
    #[must_use]
    pub fn deny(reason: impl Into<String>) -> Self {
        Self::Deny {
            reason: Some(reason.into()),
        }
    }

    /// Constructs a [`Deny`](AuthzDecision::Deny) decision with no reason.
    #[must_use]
    pub fn denied() -> Self {
        Self::Deny { reason: None }
    }

    /// Returns `true` for any [`Allow`](AuthzDecision::Allow).
    #[must_use]
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow { .. })
    }

    /// The authenticated identity when allowed, or `None` when denied.
    #[must_use]
    pub fn identity(&self) -> Option<&AuthorizedIdentity> {
        match self {
            Self::Allow { identity } => Some(identity),
            Self::Deny { .. } => None,
        }
    }
}

/// Authenticates and authorizes an inbound bearer token against a configured
/// policy.
#[capability(
    name = "bearer_token_authorizer",
    description = "Authenticates and authorizes an inbound bearer token against a configured policy"
)]
pub trait BearerTokenAuthorizer {
    /// Decides whether the caller presenting `credential` is permitted.
    ///
    /// `credential` wraps the bearer token exactly as presented (see
    /// [`BearerToken`] for constructing one from a bare token or a whole
    /// `Authorization` header value).
    ///
    /// The authorizer **authenticates** the token (verifying its
    /// signature/authenticity, expiry, issuer, audience, or via an external
    /// check such as a Kubernetes `TokenReview`) and then **authorizes** the
    /// resulting identity — both steps behind this one call, so a receiver need
    /// not authenticate separately. The trust source and the allowed identities
    /// come from the authorizer's own configuration, so the caller supplies
    /// only the credential.
    ///
    /// Returns `Ok(`[`AuthzDecision::Allow`]`)` or `Ok(`[`AuthzDecision::Deny`]`)`
    /// when the authorizer reaches a verdict — a deny (including an
    /// authentication failure such as a missing, expired, or untrusted token)
    /// is a normal outcome, not an error. Returns a [`CapabilityError`] only
    /// when the authorizer **cannot reach a decision** (e.g. its token-review or
    /// policy backend is unreachable). Callers must **fail closed** — treat an
    /// `Err` as a deny — since an undetermined decision must never grant access.
    async fn authorize(&self, credential: &BearerToken) -> Result<AuthzDecision, CapabilityError>;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_holds_bare_token() {
        let cred = BearerToken::new("eyJ.abc.sig".to_owned());
        assert_eq!(cred.expose_token(), "eyJ.abc.sig");
    }

    #[test]
    fn from_header_value_strips_bearer_prefix() {
        assert_eq!(
            BearerToken::from_header_value("Bearer eyJ.abc.sig").expose_token(),
            "eyJ.abc.sig"
        );
        // Case-insensitive scheme and surrounding/extra whitespace.
        assert_eq!(
            BearerToken::from_header_value("  bEaReR   eyJ.abc.sig  ").expose_token(),
            "eyJ.abc.sig"
        );
    }

    #[test]
    fn from_header_value_without_scheme_is_verbatim() {
        // A bare token (no scheme) is used as-is, only trimmed.
        assert_eq!(
            BearerToken::from_header_value("  eyJ.abc.sig  ").expose_token(),
            "eyJ.abc.sig"
        );
    }

    #[test]
    fn from_header_value_non_bearer_scheme_not_stripped() {
        // A non-Bearer scheme is not a bearer token; the whole value is kept so
        // it fails validation downstream rather than being silently mangled.
        assert_eq!(
            BearerToken::from_header_value("Basic dXNlcjpwYXNz").expose_token(),
            "Basic dXNlcjpwYXNz"
        );
    }

    #[test]
    fn from_header_value_requires_whitespace_separator() {
        // The `Bearer` prefix is only stripped when followed by whitespace. A
        // value that merely starts with the letters "Bearer" (no separator) is
        // NOT a scheme and must be kept verbatim, so we never mangle a token
        // that happens to begin with "Bearer".
        assert_eq!(
            BearerToken::from_header_value("BearerToken").expose_token(),
            "BearerToken"
        );
        assert_eq!(
            BearerToken::from_header_value("Bearer-eyJ.abc.sig").expose_token(),
            "Bearer-eyJ.abc.sig"
        );
    }

    #[test]
    fn from_header_value_scheme_only_is_verbatim() {
        // Just "Bearer" with no token following: no separator, so nothing is
        // stripped and the value is used verbatim (it will fail validation).
        assert_eq!(
            BearerToken::from_header_value("Bearer").expose_token(),
            "Bearer"
        );
    }

    #[test]
    fn from_header_value_accepts_any_whitespace_separator() {
        // The separator may be any whitespace (tab, multiple spaces, a mix), not
        // just a single ASCII space.
        assert_eq!(
            BearerToken::from_header_value("Bearer\teyJ.abc.sig").expose_token(),
            "eyJ.abc.sig"
        );
        assert_eq!(
            BearerToken::from_header_value("Bearer \t  eyJ.abc.sig").expose_token(),
            "eyJ.abc.sig"
        );
    }

    #[test]
    fn from_header_value_strips_only_the_leading_scheme() {
        // Only the single leading `Bearer ` scheme is stripped; a second
        // "Bearer" is part of the token and is preserved (not re-stripped).
        assert_eq!(
            BearerToken::from_header_value("Bearer Bearer eyJ.abc.sig").expose_token(),
            "Bearer eyJ.abc.sig"
        );
    }

    #[test]
    fn from_header_value_empty_or_whitespace_yields_empty_token() {
        // No token at all: an empty or whitespace-only header value trims to an
        // empty token (which downstream validation rejects).
        assert_eq!(BearerToken::from_header_value("").expose_token(), "");
        assert_eq!(BearerToken::from_header_value("   ").expose_token(), "");
    }

    #[test]
    fn new_does_not_strip_scheme() {
        // `new` is for a bare token; unlike `from_header_value` it never strips a
        // scheme, so a value that looks like a header is stored verbatim.
        assert_eq!(
            BearerToken::new("Bearer eyJ.abc.sig".to_owned()).expose_token(),
            "Bearer eyJ.abc.sig"
        );
    }

    #[test]
    fn debug_never_leaks_the_token() {
        let cred = BearerToken::new("super-secret-token".to_owned());
        let rendered = format!("{cred:?}");
        assert!(
            !rendered.contains("super-secret-token"),
            "token leaked: {rendered}"
        );
    }

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

        assert!(!AuthzDecision::denied().is_allowed());
        assert!(!AuthzDecision::deny("rbac_failed").is_allowed());
        assert_eq!(AuthzDecision::denied().identity(), None);

        assert_eq!(
            AuthzDecision::deny("nope"),
            AuthzDecision::Deny {
                reason: Some("nope".to_owned())
            }
        );
        assert_eq!(
            AuthzDecision::denied(),
            AuthzDecision::Deny { reason: None }
        );
    }
}
