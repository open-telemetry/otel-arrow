// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Instant;

use bitflags::bitflags;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::{
    capability::registry::Capabilities,
    local::capability::auth::bearer_token_provider::BearerTokenProvider,
};
use tonic::async_trait;

use crate::{api_key_auth::ApiKeyAuth, bearer_auth::BearerAuth};

/// The warnings this adapter can raise, supplied by the owning component so
/// each event name is namespaced to that component (e.g.
/// `otlp.exporter.grpc.*`) rather than to the provider. Event macros (eg
/// `otel_warn!`) const-validate the event name, so the name has to be a literal
/// at the emitting call site; passing the emitters as function pointers
/// satisfies that without making the adapter generic over a marker type.
#[derive(Clone, Copy)]
pub struct HttpClientAuthProviderEvents {
    /// A published credential could not be turned into a header.
    pub invalid: fn(&str),

    /// The provider closed its stream; no further refreshes will arrive.
    pub stream_closed: fn(),
}

/// Manages credentials and injects HTTP Authorization headers.
#[async_trait(?Send)]
pub trait HttpClientAuthProvider {
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

    /// The cached header to stamp on a request, together with
    /// the generation of the credential it was built from, cloned for the
    /// per-request send (a cheap refcount bump). `None` when no credential is
    /// cached; callers should gate on [`is_ready`](Self::is_ready) first.
    fn header(&self) -> Option<(HeaderName, HeaderValue, u64)>;

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
    async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents);
}

bitflags! {
    /// An 8-bit flags struct intended to store supported HTTP Client
    /// authentication providers.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HttpClientAuthProviders: u8 {
        /// Bearer Token authentication.
        const BEARER_TOKEN = 0b00000001;
        /// API Key authentication;
        const API_KEY      = 0b00000010;
    }
}

/// Applies an auth rejection reported by a completed export to the adapter:
/// drops the rejected auth generation so it is not sent again, leaving the
/// consumer back-pressured until the provider's next publication.
///
/// Takes the exporter's `Option<Box<dyn HttpClientAuthProvider>>` directly so
/// the common "rejection reported, provider may or may not be bound" shape is
/// expressed once. A no-op when no provider is bound (`rejected_generation` is
/// `None`) or the rejection is stale (a newer auth was already cached), per
/// [`HttpClientAuthProvider::invalidate`]'s generation guard.
pub fn apply_auth_rejection(
    auth: &mut Option<Box<dyn HttpClientAuthProvider>>,
    rejected_generation: Option<u64>,
) {
    if let (Some(generation), Some(adapter)) = (rejected_generation, auth.as_mut()) {
        adapter.invalidate(generation);
    }
}

/// Create an [`HttpClientAuthProvider`] using the registered [`Capabilities`].
pub fn new_http_client_auth_provider(
    capabilities: &Capabilities,
    supported_providers: HttpClientAuthProviders,
) -> Result<Option<Box<dyn HttpClientAuthProvider>>, otel_arrow_dfe_config::error::Error> {
    // Optionally resolve a bound bearer token provider. A bound provider supplies refreshed tokens.
    let bearer_auth = match supported_providers.contains(HttpClientAuthProviders::BEARER_TOKEN) {
        true => capabilities
            .optional_local::<otel_arrow_dfe_engine::capability::auth::bearer_token_provider::BearerTokenProvider>()
            .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?
            .map(BearerAuth::new),
        false => None
    };

    // Optionally resolve a bound api key provider. A bound provider supplies refreshed API Keys.
    let api_key_auth = match supported_providers.contains(HttpClientAuthProviders::API_KEY) {
        true => capabilities
            .optional_local::<otel_arrow_dfe_engine::capability::auth::api_key_provider::ApiKeyProvider>()
            .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?
            .map(ApiKeyAuth::new),
        false => None
    };

    let count_of_providers = vec![bearer_auth.is_some(), api_key_auth.is_some()]
        .into_iter()
        .filter(|v| *v)
        .count();
    if count_of_providers > 1 {
        return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
            error: "Multiple authentication providers cannot be bound to a single component".into(),
        });
    }

    Ok(if let Some(bearer_auth) = bearer_auth {
        Some(Box::new(bearer_auth))
    } else if let Some(api_key_auth) = api_key_auth {
        Some(Box::new(api_key_auth))
    } else {
        // Absent bindings keeps the default (no-auth) behavior
        None
    })
}

/// Create an [`HttpClientAuthProvider`] using the registered [`BearerTokenProvider`].
pub fn new_http_client_auth_provider_from_token_provider(
    token_provider: impl Into<Box<dyn BearerTokenProvider>>,
) -> impl HttpClientAuthProvider {
    BearerAuth::new(token_provider.into())
}

impl<T: BearerTokenProvider + 'static> From<T> for Box<dyn HttpClientAuthProvider> {
    fn from(value: T) -> Self {
        Box::new(BearerAuth::new(Box::new(value)))
    }
}
