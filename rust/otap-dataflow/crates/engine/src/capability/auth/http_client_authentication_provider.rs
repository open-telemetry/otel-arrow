// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! The `HttpClientAuthenticationProvider` capability.

use std::time::Instant;

use http::HeaderValue;
use otap_df_engine_macros::capability;

/// Manages credentials and injects HTTP Authorization headers.
#[capability(
    name = "http_client_authentication_provider",
    description = "Adds authentication header to http client requests"
)]
#[async_trait(?Send)]
pub trait HttpClientAuthenticationProvider {
    /// Whether the credential stream is still live and worth polling. Once the
    /// provider closes it, this returns `false` and the last cached credentials
    /// (if any) keeps being used.
    fn is_active(&self) -> bool;

    /// Whether a usable credential is cached: present and, if it expires,
    /// comfortably before expiry. Authentication should only be performed when
    /// this is `true`.
    fn is_ready(&self) -> bool;

    /// A human-readable reason [`is_ready`](Self::is_ready) is false.
    fn not_ready_reason(&self) -> &'static str;

    /// The cached `Authorization` header to stamp on a request, together with
    /// the generation of the credential it was built from, cloned for the
    /// per-request send (a cheap refcount bump). `None` when no credential is
    /// cached; callers should gate on [`is_ready`](Self::is_ready) first.
    fn header(&self) -> Option<(HeaderValue, u64)>;

    /// The instant at which a currently-usable, expiring credential crosses the
    /// usability margin (when [`is_ready`](Self::is_ready) flips to false).
    /// `None` when no usable credential is cached or the credential never
    /// expires, so the caller arms no timer in those cases. When `Some`, it is
    /// always in the future: a usable credential is by definition still beyond
    /// the margin.
    fn refresh_deadline(&self) -> Option<Instant>;

    /// Drops the cached credentials *if* `generation` is still the one
    /// currently cached. Called when the server rejects a request (HTTP 401, or
    /// gRPC `UNAUTHENTICATED`) so the rejected credential is not sent again.
    ///
    /// The generation guard makes a stale 401 harmless: if a newer credentials
    /// was cached (or the rejected credentials already cleared) after the
    /// failing request was sent, `generation` no longer matches the current one
    /// and the still-valid credentials is kept, avoiding a needless
    /// back-pressure stall.
    fn invalidate(&mut self, generation: u64);

    /// Awaits the next published credential and refreshes the cache.
    async fn poll_refresh(&mut self, events: &HttpClientAuthenticationProviderEvents);
}

/// The warnings this adapter can raise, supplied by the owning component so
/// each event name is namespaced to that component (e.g.
/// `otlp.exporter.grpc.*`) rather than to the provider. Event macros (eg
/// `otel_warn!`) const-validate the event name, so the name has to be a literal
/// at the emitting call site; passing the emitters as function pointers
/// satisfies that without making the adapter generic over a marker type.
#[derive(Clone, Copy)]
pub struct HttpClientAuthenticationProviderEvents {
    /// A published credential could not be turned into an `Authorization` header.
    pub invalid: fn(),

    /// The provider closed its stream; no further refreshes will arrive.
    pub stream_closed: fn(),
}
