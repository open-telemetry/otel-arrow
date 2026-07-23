// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `bearer_token_provider` capability.
//!
//! Centralizes everything an exporter needs to authenticate outgoing requests
//! with a bearer token, so the exporter itself stays auth-agnostic: it drives
//! [`BearerAuth::poll_refresh`] in its `select!` loop, asks
//! [`BearerAuth::is_ready`] before admitting data, and stamps
//! [`BearerAuth::header`] onto each request.
//!
//! The division of labor mirrors the capability design: the **provider**
//! (extension) owns credential acquisition, background refresh, and startup
//! readiness gating; this **adapter** only subscribes to the provider's token
//! stream, caches the built `Authorization` header, and tracks whether that
//! cached token is still usable. The exporter is the "dumb caller".

use std::time::{Duration, Instant};

use futures::StreamExt;
use http::HeaderValue;
use otap_df_engine::capability::auth::bearer_token_provider::TokenStream;
use otap_df_engine::local::capability::auth::bearer_token_provider::BearerTokenProvider;
use otap_df_telemetry::otel_warn;

/// Safety margin before a cached token's expiry within which it is treated as
/// unusable, so the exporter back-pressures (awaiting a fresh token) rather than
/// sending a request that could outlive its token. The provider refreshes well
/// ahead of expiry, so this only bites in a degraded window (refresh failing,
/// cached token genuinely near expiry).
const TOKEN_USABLE_MARGIN: Duration = Duration::from_secs(30);

/// Consumer-side bearer-token authenticator: subscribes to a provider's token
/// stream, caches the built `Authorization` header, and reports usability.
///
/// All token/expiry/stream state lives here, so an exporter holds one of these
/// and never touches a token directly.
pub(crate) struct BearerAuth {
    /// Subscription to the provider's token refreshes.
    stream: TokenStream,
    /// Whether the stream is still live and worth polling.
    stream_active: bool,
    /// The `Authorization: Bearer <token>` header built from the latest token.
    cached_header: Option<HeaderValue>,
    /// Expiry of the token behind `cached_header` (`None` = non-expiring).
    cached_expiry: Option<Instant>,
}

impl BearerAuth {
    /// Subscribes to `provider`'s token stream.
    pub(crate) fn new(provider: Box<dyn BearerTokenProvider>) -> Self {
        Self {
            stream: provider.token_stream(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
        }
    }

    /// Whether the token stream is still live and worth polling. Once the
    /// provider closes it, this returns `false` and the last cached token (if
    /// any) keeps being used.
    pub(crate) fn is_active(&self) -> bool {
        self.stream_active
    }

    /// Whether a usable token is cached: present and, if it expires, comfortably
    /// before expiry. The exporter admits data only when this is `true`.
    pub(crate) fn is_ready(&self) -> bool {
        match (self.cached_header.is_some(), self.cached_expiry) {
            (false, _) => false,
            (true, None) => true, // non-expiring token
            (true, Some(expires_on)) => expires_on > Instant::now() + TOKEN_USABLE_MARGIN,
        }
    }

    /// A human-readable reason [`is_ready`](Self::is_ready) is false, for NACK
    /// messages.
    pub(crate) fn not_ready_reason(&self) -> &'static str {
        if self.cached_header.is_some() {
            "bearer token at/near expiry; awaiting refresh"
        } else {
            "bearer token unavailable"
        }
    }

    /// The cached `Authorization` header to stamp on a request, cloned for the
    /// per-request send (a cheap refcount bump). `None` when no token is cached;
    /// callers should gate on [`is_ready`](Self::is_ready) first.
    pub(crate) fn header(&self) -> Option<HeaderValue> {
        self.cached_header.clone()
    }

    /// The instant at which a currently-usable, expiring token crosses the
    /// usability margin (when [`is_ready`](Self::is_ready) flips to false).
    /// `None` when no usable token is cached or the token never expires, so the
    /// caller arms no timer in those cases. When `Some`, it is always in the
    /// future: a usable token is by definition still beyond the margin.
    pub(crate) fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_expiry
            .and_then(|expires_on| expires_on.checked_sub(TOKEN_USABLE_MARGIN))
    }

    /// Drops the cached token so [`is_ready`](Self::is_ready) returns false until
    /// the next refresh delivers a new one. Called when the server rejects the
    /// current token (HTTP 401) so a retry waits for a fresh token rather than
    /// reusing the rejected one.
    pub(crate) fn invalidate(&mut self) {
        self.cached_header = None;
        self.cached_expiry = None;
    }

    /// Awaits the next published token and refreshes the cache. Only meaningful
    /// while [`is_active`](Self::is_active); on stream close it flips inactive
    /// and keeps the last cached token. Malformed tokens and stream closure are
    /// logged internally.
    pub(crate) async fn poll_refresh(&mut self) {
        match self.stream.next().await {
            Some(token) => {
                match HeaderValue::from_str(&format!("Bearer {}", token.expose_token())) {
                    Ok(mut value) => {
                        // Redact in `Debug`, exclude from HPACK indexing.
                        value.set_sensitive(true);
                        self.cached_header = Some(value);
                        self.cached_expiry = token.expires_on();
                    }
                    Err(e) => {
                        // Malformed token: keep the previous cached token (if any).
                        otel_warn!("otlp.exporter.http.invalid_bearer_token", error = %e);
                    }
                }
            }
            None => {
                // Provider closed its stream; no further refreshes will arrive.
                // Keep using the last cached token. Not expected with a
                // watch-backed provider while we hold its handle, so warn.
                self.stream_active = false;
                otel_warn!(
                    "otlp.exporter.http.token_stream_closed",
                    message = "bearer token provider closed its stream; \
                        no further token refreshes will arrive"
                );
            }
        }
    }
}
