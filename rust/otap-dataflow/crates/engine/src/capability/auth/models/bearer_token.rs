// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The shared [`BearerToken`] credential.
//!
//! One token type for both sides of the data path: minted and handed out by a
//! [`BearerTokenProvider`](super::super::bearer_token_provider::BearerTokenProvider)
//! (carrying an optional expiry so consumers know when to refresh), and
//! presented by a caller for a
//! [`BearerTokenAuthorizer`](super::super::bearer_token_authorizer::BearerTokenAuthorizer)
//! to validate (where no expiry is set — freshness is the authorizer's concern).

use secrecy::{ExposeSecret, SecretString};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime};

/// An OAuth/OIDC-style bearer token.
///
/// The secret is wrapped in [`SecretString`], which zeroizes on drop and masks
/// itself in [`Debug`] output, so it cannot leak into logs or telemetry. The
/// `SecretString` sits behind an [`Arc`] so cloning a token (handing it to
/// multiple subscribers, or returning it from `get_token` on the hot path) is a
/// cheap refcount bump that shares one plaintext allocation rather than copying
/// the secret bytes.
///
/// `expires_on` is a monotonic [`Instant`] — providers convert a credential's
/// absolute wall-clock expiry to an `Instant` once, so the value is immune to
/// wall-clock jumps thereafter. `None` means the token has no known expiry: a
/// provider that reports none, or a token presented for validation (where
/// freshness is decided by the authorizer, not carried on the token).
#[derive(Clone, Debug)]
pub struct BearerToken {
    secret: Arc<SecretString>,
    expires_on: Option<Instant>,
}

impl BearerToken {
    /// Creates a token with no known expiry.
    ///
    /// Accepts anything convertible into [`SecretString`] (e.g. a `String`),
    /// which is then shared behind an [`Arc`]. Use
    /// [`with_expiry`](Self::with_expiry) or
    /// [`from_absolute_expiry`](Self::from_absolute_expiry) when the token has a
    /// known lifetime, and [`from_header_value`](Self::from_header_value) to
    /// parse a whole `Authorization` header value.
    #[must_use]
    pub fn new(secret: impl Into<SecretString>) -> Self {
        Self {
            secret: Arc::new(secret.into()),
            expires_on: None,
        }
    }

    /// Creates a token with an optional monotonic expiry.
    #[must_use]
    pub fn with_expiry(secret: impl Into<SecretString>, expires_on: Option<Instant>) -> Self {
        Self {
            secret: Arc::new(secret.into()),
            expires_on,
        }
    }

    /// Creates a token from a secret and an **absolute** wall-clock expiry.
    ///
    /// Credential services that report an absolute expiry often give it as
    /// calendar time ([`SystemTime`]), but [`BearerToken`] stores a monotonic
    /// [`Instant`] (see the field docs for why). Every such provider has to
    /// perform the same wall-clock-to-monotonic conversion; this constructor
    /// centralizes it so no provider gets it subtly wrong.
    ///
    /// It measures how far `expires_on` is from *now* and offsets the current
    /// `Instant` by that duration. An `expires_on` already in the past (or a
    /// backwards clock) clamps to "expires immediately" (`Instant::now()`)
    /// rather than producing a time before now.
    #[must_use]
    pub fn from_absolute_expiry(secret: impl Into<SecretString>, expires_on: SystemTime) -> Self {
        let remaining = expires_on
            .duration_since(SystemTime::now())
            .unwrap_or(Duration::ZERO);
        Self::with_expiry(secret, Some(Instant::now() + remaining))
    }

    /// Creates a token from a whole `Authorization` header value, with no
    /// expiry.
    ///
    /// Strips a leading, case-insensitive `Bearer ` scheme prefix when present
    /// (e.g. `"Bearer eyJ…"` → `"eyJ…"`), and otherwise treats the trimmed
    /// value as the token verbatim. Surrounding whitespace is trimmed either
    /// way. A caller that already has the bare token should use
    /// [`new`](Self::new).
    #[must_use]
    pub fn from_header_value(header_value: &str) -> Self {
        let token = strip_bearer_prefix(header_value).unwrap_or_else(|| header_value.trim());
        Self::new(token.to_owned())
    }

    /// Exposes the bearer token secret, for the authorizer to validate or for
    /// injection into an `Authorization` header.
    ///
    /// Named `expose_token` (rather than a plain getter) so every plaintext
    /// access is explicit and greppable.
    #[must_use]
    pub fn expose_token(&self) -> &str {
        self.secret.expose_secret()
    }

    /// The monotonic instant at which this token expires, if known.
    #[must_use]
    pub fn expires_on(&self) -> Option<Instant> {
        self.expires_on
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_holds_bare_token_without_expiry() {
        let token = BearerToken::new("eyJ.abc.sig".to_owned());
        assert_eq!(token.expose_token(), "eyJ.abc.sig");
        assert_eq!(token.expires_on(), None);
    }

    #[test]
    fn with_expiry_round_trips_accessors() {
        let now = Instant::now();
        let token = BearerToken::with_expiry("super-secret".to_owned(), Some(now));
        assert_eq!(token.expose_token(), "super-secret");
        assert_eq!(token.expires_on(), Some(now));

        let non_expiring = BearerToken::with_expiry("s".to_owned(), None);
        assert_eq!(non_expiring.expires_on(), None);
    }

    #[test]
    fn from_absolute_expiry_converts_future_wall_clock_to_instant() {
        let before = Instant::now();
        let token = BearerToken::from_absolute_expiry(
            "s".to_owned(),
            SystemTime::now() + Duration::from_secs(60),
        );
        let after = Instant::now();
        let expiry = token.expires_on().expect("future expiry is set");
        // The converted instant lands ~60s ahead of when we called it.
        assert!(expiry >= before + Duration::from_secs(59));
        assert!(expiry <= after + Duration::from_secs(61));
    }

    #[test]
    fn from_absolute_expiry_clamps_past_wall_clock_to_now() {
        let before = Instant::now();
        let token = BearerToken::from_absolute_expiry(
            "s".to_owned(),
            SystemTime::now() - Duration::from_secs(60),
        );
        let after = Instant::now();
        let expiry = token.expires_on().expect("expiry is set");
        // A past expiry clamps to "now", never a time before now.
        assert!(expiry >= before);
        assert!(expiry <= after);
    }

    #[test]
    fn clone_shares_the_same_secret_allocation() {
        let token = BearerToken::new("super-secret".to_owned());
        let cloned = token.clone();
        // Both handles observe the same plaintext...
        assert_eq!(token.expose_token(), cloned.expose_token());
        // ...backed by one shared allocation (a clone is a refcount bump, not a
        // fresh copy of the secret bytes).
        assert!(std::ptr::eq(token.expose_token(), cloned.expose_token()));
    }

    #[test]
    fn debug_never_leaks_the_secret() {
        let token = BearerToken::new("super-secret-token".to_owned());
        let rendered = format!("{token:?}");
        assert!(
            !rendered.contains("super-secret-token"),
            "secret leaked: {rendered}"
        );
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
            BearerToken::from_header_value("Bearer  eyJ.abc.sig").expose_token(),
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
}
