// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The shared [`BearerToken`] credential.
//!
//! One token type for both sides of the data path: minted and handed out by a
//! [`BearerTokenProvider`](super::super::bearer_token_provider::BearerTokenProvider)
//! (carrying an optional expiry so consumers know when to refresh), and
//! presented by a caller for a
//! [`BearerTokenAuthorizer`](super::super::bearer_token_authorizer::BearerTokenAuthorizer)
//! to validate. The token secret is treated as opaque: expiry is supplied
//! explicitly from the issuer's response metadata (OAuth `expires_in` /
//! `expires_on`), never derived by parsing the token bytes.

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
/// `expires_on` is a monotonic [`Instant`] -- an absolute wall-clock expiry is
/// converted to an `Instant` once, so the value is immune to wall-clock jumps
/// thereafter. `None` means no known expiry. The token secret is opaque to this
/// type: an expiry is only ever what a caller supplies from the issuer's
/// response metadata, never parsed out of the token itself.
#[derive(Clone, Debug)]
pub struct BearerToken {
    secret: Arc<SecretString>,
    expires_on: Option<Instant>,
}

impl BearerToken {
    /// Creates a token with **no known expiry**.
    ///
    /// The secret is treated as opaque, so this constructor never infers an
    /// expiry (a bearer token's lifetime is issuer metadata, not something to
    /// parse out of the token). Use it for an inbound token presented for
    /// validation, or an issuer that reports no expiry.
    ///
    /// A provider minting a token from an issuer response **must** carry the
    /// issuer's expiry through [`with_expiry`](Self::with_expiry) or
    /// [`from_absolute_expiry`](Self::from_absolute_expiry) instead, so the
    /// token is not treated as never-expiring. [`from_header_value`](Self::from_header_value)
    /// parses a whole `Authorization` header value (also without an expiry).
    ///
    /// Accepts anything convertible into [`SecretString`] (e.g. a `String`),
    /// which is then shared behind an [`Arc`].
    #[must_use]
    pub fn without_expiry(secret: impl Into<SecretString>) -> Self {
        Self {
            secret: Arc::new(secret.into()),
            expires_on: None,
        }
    }

    /// Creates a token with an explicit optional monotonic expiry.
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
    /// rather than producing a time before now. An `expires_on` so far in the
    /// future that the monotonic offset would overflow `Instant` is treated as
    /// no known expiry (`None`) rather than panicking, so a bad or sentinel
    /// expiry cannot crash the node.
    #[must_use]
    pub fn from_absolute_expiry(secret: impl Into<SecretString>, expires_on: SystemTime) -> Self {
        Self::with_expiry(secret, absolute_to_monotonic(expires_on))
    }

    /// Creates a token from a secret and a **relative** lifetime.
    ///
    /// Token endpoints commonly report a lifetime as "seconds from now" (e.g.
    /// OAuth 2.0 `expires_in`, RFC 6749 section 5.1) rather than an absolute
    /// instant. Building a [`SystemTime`] from that to hand to
    /// [`from_absolute_expiry`](Self::from_absolute_expiry) panics on overflow,
    /// so this constructor centralizes the safe conversion: an issuer reporting
    /// an absurd or sentinel lifetime is treated as no known expiry (`None`)
    /// rather than crashing the node.
    ///
    /// It also avoids the wall-clock round-trip
    /// [`from_absolute_expiry`](Self::from_absolute_expiry) has to perform, so
    /// it cannot pick up clock skew between the two reads. Prefer it whenever
    /// the issuer reports a relative lifetime.
    #[must_use]
    pub fn from_relative_expiry(secret: impl Into<SecretString>, expires_in: Duration) -> Self {
        Self::with_expiry(secret, Instant::now().checked_add(expires_in))
    }

    /// Creates a token from a valid bearer `Authorization` header value, with
    /// no expiry.
    ///
    /// The scheme is matched case-insensitively and must be followed by one or
    /// more ASCII spaces and a valid RFC 6750 `b64token`. Returns `None` for a
    /// missing scheme, a non-bearer scheme, or a malformed token. A caller that
    /// already has the bare token should use
    /// [`without_expiry`](Self::without_expiry).
    #[must_use]
    pub fn from_header_value(header_value: &str) -> Option<Self> {
        let token = extract_bearer_token(header_value)?;
        Some(Self::without_expiry(token.to_owned()))
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
    pub const fn expires_on(&self) -> Option<Instant> {
        self.expires_on
    }
}

/// Extracts a non-empty token from a case-insensitive bearer authorization
/// header.
fn extract_bearer_token(header_value: &str) -> Option<&str> {
    let trimmed = header_value.trim_matches([' ', '\t']);
    let (scheme, rest) = trimmed.split_once(' ')?;
    if !scheme.eq_ignore_ascii_case("Bearer") {
        return None;
    }
    let token = rest.trim_matches(' ');
    is_valid_b64token(token).then_some(token)
}

fn is_valid_b64token(token: &str) -> bool {
    let mut saw_token_byte = false;
    let mut saw_padding = false;

    for byte in token.bytes() {
        if byte == b'=' {
            saw_padding = true;
        } else if saw_padding
            || !(byte.is_ascii_alphanumeric()
                || matches!(byte, b'-' | b'.' | b'_' | b'~' | b'+' | b'/'))
        {
            return false;
        } else {
            saw_token_byte = true;
        }
    }

    saw_token_byte
}

/// Converts an absolute wall-clock expiry to a monotonic [`Instant`] offset from
/// now. An expiry already in the past clamps to "now" (never a time before now);
/// one so far in the future that the offset overflows `Instant` yields `None`
/// (treated as no known expiry) rather than panicking.
fn absolute_to_monotonic(expires_on: SystemTime) -> Option<Instant> {
    let remaining = expires_on
        .duration_since(SystemTime::now())
        .unwrap_or(Duration::ZERO);
    Instant::now().checked_add(remaining)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: construct a token from an opaque string via `without_expiry`.
    /// Guarantees: the token exposes the exact bytes and reports no known expiry
    /// (the constructor never infers one from the secret).
    #[test]
    fn without_expiry_holds_opaque_token() {
        let token = BearerToken::without_expiry("opaque-secret-token".to_owned());
        assert_eq!(token.expose_token(), "opaque-secret-token");
        assert_eq!(token.expires_on(), None);
    }

    /// Scenario: build tokens with `with_expiry`, both with and without an
    /// expiry instant.
    /// Guarantees: the secret and the optional expiry round-trip through the
    /// accessors unchanged.
    #[test]
    fn with_expiry_round_trips_accessors() {
        let now = Instant::now();
        let token = BearerToken::with_expiry("super-secret".to_owned(), Some(now));
        assert_eq!(token.expose_token(), "super-secret");
        assert_eq!(token.expires_on(), Some(now));

        let non_expiring = BearerToken::with_expiry("s".to_owned(), None);
        assert_eq!(non_expiring.expires_on(), None);
    }

    /// Scenario: build a token from an absolute wall-clock expiry set 60s in the
    /// future.
    /// Guarantees: the stored monotonic expiry lands ~60s ahead of construction
    /// time (within a small slack window).
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

    /// Scenario: build a token from an absolute wall-clock expiry already 60s in
    /// the past.
    /// Guarantees: the expiry clamps to "now" (never a time before
    /// construction), so a stale credential is treated as immediately expired.
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

    /// Scenario: build a token from an absurdly far-future absolute expiry whose
    /// monotonic offset could overflow `Instant`.
    /// Guarantees: construction never panics; the expiry is either a valid
    /// future instant or falls back to `None` (no known expiry).
    #[test]
    fn from_absolute_expiry_does_not_panic_on_far_future() {
        let far = SystemTime::now() + Duration::from_secs(60 * 60 * 24 * 365 * 1000);
        let token = BearerToken::from_absolute_expiry("s".to_owned(), far);
        // Either it fit (a future instant) or overflowed to None -- never a panic.
        if let Some(expiry) = token.expires_on() {
            assert!(expiry > Instant::now());
        }
    }

    /// Scenario: build a token from a relative lifetime of 60s.
    /// Guarantees: the stored monotonic expiry lands ~60s ahead of construction
    /// time, so an issuer's `expires_in` is carried through faithfully.
    #[test]
    fn from_relative_expiry_offsets_from_now() {
        let before = Instant::now();
        let token = BearerToken::from_relative_expiry("s".to_owned(), Duration::from_secs(60));
        let after = Instant::now();
        let expiry = token.expires_on().expect("future expiry is set");
        assert!(expiry >= before + Duration::from_secs(60));
        assert!(expiry <= after + Duration::from_secs(60));
    }

    /// Scenario: build a token from a relative lifetime large enough that
    /// offsetting `Instant` (or a wall-clock `SystemTime`) would overflow.
    /// Guarantees: construction yields no known expiry instead of panicking, so
    /// an issuer reporting an absurd or sentinel lifetime cannot crash the node.
    #[test]
    fn from_relative_expiry_does_not_panic_on_absurd_lifetime() {
        let token = BearerToken::from_relative_expiry("s".to_owned(), Duration::MAX);
        assert_eq!(token.expires_on(), None);
        assert_eq!(token.expose_token(), "s");
    }

    /// Scenario: build a token from a zero relative lifetime.
    /// Guarantees: the expiry is "now" rather than `None`, so an already-spent
    /// token is treated as expired instead of never-expiring.
    #[test]
    fn from_relative_expiry_zero_expires_immediately() {
        let before = Instant::now();
        let token = BearerToken::from_relative_expiry("s".to_owned(), Duration::ZERO);
        let after = Instant::now();
        let expiry = token
            .expires_on()
            .expect("zero lifetime still sets an expiry");
        assert!(expiry >= before);
        assert!(expiry <= after);
    }

    /// Scenario: clone a token and compare the exposed secret of both handles.
    /// Guarantees: the clone shares one plaintext allocation (pointer-equal), so
    /// cloning is a refcount bump rather than a copy of the secret bytes.
    #[test]
    fn clone_shares_the_same_secret_allocation() {
        let token = BearerToken::without_expiry("super-secret".to_owned());
        let cloned = token.clone();
        // Both handles observe the same plaintext...
        assert_eq!(token.expose_token(), cloned.expose_token());
        // ...backed by one shared allocation (a clone is a refcount bump, not a
        // fresh copy of the secret bytes).
        assert!(std::ptr::eq(token.expose_token(), cloned.expose_token()));
    }

    /// Scenario: render a token with the `Debug` formatter.
    /// Guarantees: the secret value never appears in `Debug` output, preventing
    /// credential leakage into logs or telemetry.
    #[test]
    fn debug_never_leaks_the_secret() {
        let token = BearerToken::without_expiry("super-secret-token".to_owned());
        let rendered = format!("{token:?}");
        assert!(
            !rendered.contains("super-secret-token"),
            "secret leaked: {rendered}"
        );
    }

    /// Scenario: parse Authorization header values that carry a `Bearer` scheme,
    /// including mixed case and extra surrounding whitespace.
    /// Guarantees: the scheme prefix and surrounding whitespace are stripped,
    /// leaving only the bare token.
    #[test]
    fn from_header_value_strips_bearer_prefix() {
        assert_eq!(
            BearerToken::from_header_value("Bearer eyJ.abc.sig")
                .expect("valid bearer header")
                .expose_token(),
            "eyJ.abc.sig"
        );
        // Case-insensitive scheme and surrounding/extra whitespace.
        assert_eq!(
            BearerToken::from_header_value("  bEaReR   eyJ.abc.sig  ")
                .expect("valid case-insensitive bearer header")
                .expose_token(),
            "eyJ.abc.sig"
        );
    }

    /// Scenario: parse a header value that has no scheme prefix.
    /// Guarantees: a bare value cannot construct a token through the header
    /// parser.
    #[test]
    fn from_header_value_rejects_missing_scheme() {
        assert!(BearerToken::from_header_value("  eyJ.abc.sig  ").is_none());
    }

    /// Scenario: parse a header value carrying a non-`Bearer` scheme (e.g.
    /// `Basic ...`).
    /// Guarantees: a non-bearer authorization scheme is rejected at
    /// construction.
    #[test]
    fn from_header_value_rejects_non_bearer_scheme() {
        assert!(BearerToken::from_header_value("Basic dXNlcjpwYXNz").is_none());
    }

    /// Scenario: parse values that merely start with the letters "Bearer" but
    /// have no whitespace separator (e.g. `BearerToken`, `Bearer-eyJ...`).
    /// Guarantees: values without a scheme separator are rejected.
    #[test]
    fn from_header_value_requires_whitespace_separator() {
        assert!(BearerToken::from_header_value("BearerToken").is_none());
        assert!(BearerToken::from_header_value("Bearer-eyJ.abc.sig").is_none());
    }

    /// Scenario: parse a header value that is exactly the scheme word `Bearer`
    /// with no token following it.
    /// Guarantees: a scheme without a credential is rejected.
    #[test]
    fn from_header_value_rejects_scheme_only() {
        assert!(BearerToken::from_header_value("Bearer").is_none());
        assert!(BearerToken::from_header_value("Bearer   ").is_none());
    }

    /// Scenario: parse a header value with multiple ASCII spaces between the
    /// scheme and token.
    /// Guarantees: the RFC 6750 `1*SP` separator accepts more than one space.
    #[test]
    fn from_header_value_accepts_multiple_spaces() {
        assert_eq!(
            BearerToken::from_header_value("Bearer  eyJ.abc.sig")
                .expect("valid bearer header")
                .expose_token(),
            "eyJ.abc.sig"
        );
    }

    /// Scenario: parse a header value with a tab between the scheme and token.
    /// Guarantees: a tab is rejected because RFC 6750 requires one or more
    /// ASCII spaces between `Bearer` and the credential.
    #[test]
    fn from_header_value_rejects_tab_separator() {
        assert!(BearerToken::from_header_value("Bearer\teyJ.abc.sig").is_none());
    }

    /// Scenario: parse bearer credentials containing every RFC 6750 `b64token`
    /// symbol and optional trailing padding.
    /// Guarantees: valid bearer-token characters and trailing `=` padding are
    /// accepted without changing the credential.
    #[test]
    fn from_header_value_accepts_b64token_characters() {
        let token = "AZaz09-._~+/==";
        assert_eq!(
            BearerToken::from_header_value(&format!("Bearer {token}"))
                .expect("valid b64token")
                .expose_token(),
            token
        );
    }

    /// Scenario: parse bearer credentials containing punctuation outside the
    /// RFC 6750 `b64token` grammar or padding followed by more token bytes.
    /// Guarantees: malformed bearer credentials are rejected before reaching
    /// an authorizer.
    #[test]
    fn from_header_value_rejects_invalid_b64token_characters() {
        assert!(BearerToken::from_header_value("Bearer abc:def").is_none());
        assert!(BearerToken::from_header_value("Bearer abc=def").is_none());
    }

    /// Scenario: parse a header value containing two `Bearer ` prefixes (e.g.
    /// `Bearer Bearer eyJ...`).
    /// Guarantees: whitespace inside the credential is rejected.
    #[test]
    fn from_header_value_rejects_whitespace_in_token() {
        assert!(BearerToken::from_header_value("Bearer first second").is_none());
    }

    /// Scenario: parse empty and whitespace-only header values.
    /// Guarantees: both are rejected rather than constructing an empty token.
    #[test]
    fn from_header_value_rejects_empty_or_whitespace() {
        assert!(BearerToken::from_header_value("").is_none());
        assert!(BearerToken::from_header_value("   ").is_none());
    }

    /// Scenario: pass a header-looking value (`Bearer eyJ...`) to `without_expiry` rather
    /// than `from_header_value`.
    /// Guarantees: `without_expiry` never strips a scheme, so the value is stored verbatim
    /// (the scheme-stripping behavior is exclusive to `from_header_value`).
    #[test]
    fn without_expiry_does_not_strip_scheme() {
        // `without_expiry` is for a bare token; unlike `from_header_value` it never strips a
        // scheme, so a value that looks like a header is stored verbatim.
        assert_eq!(
            BearerToken::without_expiry("Bearer eyJ.abc.sig".to_owned()).expose_token(),
            "Bearer eyJ.abc.sig"
        );
    }
}
