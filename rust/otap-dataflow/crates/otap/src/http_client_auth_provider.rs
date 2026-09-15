// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{borrow::Cow, time::Instant};

use bitflags::bitflags;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::{
    capability::registry::Capabilities,
    local::capability::auth::bearer_token_provider::BearerTokenProvider,
};
use tonic::async_trait;

use crate::{
    agent_fed_auth::AgentFedAuth, api_key_auth::ApiKeyAuth, basic_auth::BasicAuth,
    bearer_auth::BearerAuth,
};

/// The warnings this adapter can raise, supplied by the owning component so
/// each event name is namespaced to that component (e.g.
/// `otlp.exporter.grpc.*`) rather than to the provider. Event macros (eg
/// `otel_warn!`) const-validate the event name, so the name has to be a literal
/// at the emitting call site; passing the emitters as function pointers
/// satisfies that without making the adapter generic over a marker type.
#[derive(Clone, Copy)]
pub struct HttpClientAuthProviderEvents {
    /// A published credential could not be turned into a header.
    pub invalid: fn(HttpClientAuthProviderName, &str),

    /// The provider closed its stream; no further refreshes will arrive.
    pub stream_closed: fn(HttpClientAuthProviderName),
}

impl HttpClientAuthProviderEvents {
    /// Emit an invalid event.
    pub fn emit_invalid(&self, source: &dyn HttpClientAuthProvider, error: &str) {
        (self.invalid)(source.name(), error)
    }

    /// Emit a stream_closed event.
    pub fn emit_stream_closed(&self, source: &dyn HttpClientAuthProvider) {
        (self.stream_closed)(source.name())
    }
}

/// Human-readable name of a provider.
pub type HttpClientAuthProviderName = Cow<'static, str>;

/// Manages credentials and injects HTTP Authorization headers.
#[async_trait(?Send)]
pub trait HttpClientAuthProvider {
    /// Human-readable name used in error messages and config validation.
    fn name(&self) -> HttpClientAuthProviderName;

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
    async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents) -> bool;
}

bitflags! {
    /// An 8-bit flags struct intended to store supported HTTP Client
    /// authentication providers.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct HttpClientAuthProviders: u8 {
        /// Bearer Token authentication.
        const BEARER_TOKEN = 0b00000001;
        /// API Key authentication.
        const API_KEY      = 0b00000010;
        /// Basic authentication.
        const BASIC        = 0b00000100;
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
    let mut providers: Vec<Box<dyn HttpClientAuthProvider>> = vec![];

    if supported_providers.contains(HttpClientAuthProviders::BEARER_TOKEN) {
        // Optionally resolve a bound bearer token provider. A bound provider supplies refreshed bearer tokens.
        if let Some(bearer_auth) = capabilities
            .optional_local::<otel_arrow_dfe_engine::capability::auth::bearer_token_provider::BearerTokenProvider>()
            .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?
            .map(BearerAuth::new) {
            providers.push(Box::new(bearer_auth));
        }

        // Optionally resolve an agent fed credential provider. A bound provider supplies refreshed bearer tokens.
        if let Some(agent_fed_auth) = capabilities
            .optional_local::<otel_arrow_dfe_engine::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider>()
            .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?
            .map(AgentFedAuth::new) {
            providers.push(Box::new(agent_fed_auth));
        }
    }

    if supported_providers.contains(HttpClientAuthProviders::API_KEY) {
        // Optionally resolve a bound api key provider. A bound provider supplies refreshed API Keys.
        if let Some(api_key_auth) = capabilities
            .optional_local::<otel_arrow_dfe_engine::capability::auth::api_key_provider::ApiKeyProvider>()
            .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?
            .map(ApiKeyAuth::new) {
            providers.push(Box::new(api_key_auth));
        }
    }

    if supported_providers.contains(HttpClientAuthProviders::BASIC) {
        // Optionally resolve a bound basic auth provider. A bound provider supplies refreshed credentials.
        if let Some(basic_auth) = capabilities
            .optional_local::<otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BasicAuthProvider>()
            .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?
            .map(BasicAuth::new) {
            providers.push(Box::new(basic_auth));
        }
    }

    new_http_client_auth_provider_from_providers(providers)
}

fn new_http_client_auth_provider_from_providers(
    providers: Vec<Box<dyn HttpClientAuthProvider>>,
) -> Result<Option<Box<dyn HttpClientAuthProvider>>, otel_arrow_dfe_config::error::Error> {
    let mut providers = providers.into_iter();

    Ok(if let Some(first_provider) = providers.next() {
        if providers.next().is_some() {
            return Err(otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: "Multiple authentication providers cannot be bound to a single component"
                    .into(),
            });
        }
        Some(first_provider)
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

#[cfg(any(test, feature = "test-utils"))]
pub mod test_support {
    //! Test doubles shared by the nodes that consume this adapter, so every
    //! consumer's test suite drives the same provider behavior instead of each
    //! maintaining its own copy.

    use futures::{Stream, StreamExt};
    use http::header;
    use std::{pin::Pin, time::Duration};

    use super::*;

    const NAME: &str = "MockAuth";

    const AUTH_USABLE_MARGIN: Duration = Duration::from_secs(30);

    type AuthStream = Pin<Box<dyn Stream<Item = (String, Option<Duration>)> + 'static>>;

    /// Test double for the `HttpClientAuthProvider` with configurable behavior.
    pub struct MockHttpClientAuthProvider {
        stream: AuthStream,
        header_name: HeaderName,
        cached_header: Option<HeaderValue>,
        cached_expiry: Option<Instant>,
        generation: u64,
        stream_active: bool,
    }

    impl MockHttpClientAuthProvider {
        /// Create a provider from a list of values with optional expiry.
        #[must_use]
        pub fn new(
            header_name: HeaderName,
            header_values: Vec<(String, Option<Duration>)>,
        ) -> MockHttpClientAuthProvider {
            let published = futures::stream::iter(header_values);

            Self {
                stream: published.chain(futures::stream::pending()).boxed(),
                header_name,
                cached_header: None,
                cached_expiry: None,
                generation: 0,
                stream_active: true,
            }
        }

        /// Create a provider which never publishes.
        #[must_use]
        pub fn never_publishes() -> MockHttpClientAuthProvider {
            let published = futures::stream::iter(vec![]);

            Self {
                stream: published.chain(futures::stream::pending()).boxed(),
                header_name: header::AUTHORIZATION,
                cached_header: None,
                cached_expiry: None,
                generation: 0,
                stream_active: true,
            }
        }
    }

    #[async_trait(?Send)]
    impl HttpClientAuthProvider for MockHttpClientAuthProvider {
        fn name(&self) -> HttpClientAuthProviderName {
            NAME.into()
        }

        fn is_active(&self) -> bool {
            self.stream_active
        }

        fn is_ready(&self) -> bool {
            match (self.cached_header.is_some(), self.cached_expiry) {
                (false, _) => false,
                (true, None) => true, // non-expiring token
                (true, Some(expires_on)) => expires_on > Instant::now() + AUTH_USABLE_MARGIN,
            }
        }

        fn not_ready_reason(&self) -> &'static str {
            if self.cached_header.is_some() {
                "auth at/near expiry; awaiting refresh"
            } else {
                "auth unavailable"
            }
        }

        fn header(&self) -> Option<(HeaderName, HeaderValue, u64)> {
            self.cached_header
                .clone()
                .map(|header| (self.header_name.clone(), header, self.generation))
        }

        fn refresh_deadline(&self) -> Option<Instant> {
            if !self.is_ready() {
                return None;
            }
            self.cached_expiry
                .and_then(|expires_on| expires_on.checked_sub(AUTH_USABLE_MARGIN))
        }

        fn invalidate(&mut self, generation: u64) {
            if generation == self.generation && self.cached_header.is_some() {
                self.cached_header = None;
                self.cached_expiry = None;
            }
        }

        async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents) -> bool {
            match self.stream.next().await {
                Some((value, duration)) => {
                    match HeaderValue::from_str(&value) {
                        Ok(mut value) => {
                            // Redact in `Debug`, exclude from HPACK indexing.
                            value.set_sensitive(true);
                            self.cached_header = Some(value);
                            self.cached_expiry = duration.map(|v| Instant::now() + v);
                            // A new cached token starts a new generation, so a 401 for
                            // an earlier token no longer matches and is ignored.
                            self.generation = self.generation.wrapping_add(1);
                            return true;
                        }
                        Err(e) => {
                            // Malformed token: keep the previous cached token (if any).
                            events.emit_invalid(self, &format!("Malformed token: {e}"));
                            return false;
                        }
                    }
                }
                None => {
                    // Provider closed its stream; no further refreshes will arrive.
                    // Keep using the last cached token. Not expected with a
                    // watch-backed provider while we hold its handle, so warn.
                    self.stream_active = false;
                    events.emit_stream_closed(self);
                    return false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use otel_arrow_dfe_engine::{
        capability::{
            CapabilityError,
            auth::{api_key_provider::ApiKeyStream, bearer_token_provider::TokenStream, *},
        },
        local::capability::auth::api_key_provider::ApiKeyProvider,
    };

    use super::*;
    use futures::StreamExt;
    use futures::stream;

    struct MockBearerTokenProvider {}

    #[async_trait(?Send)]
    impl BearerTokenProvider for MockBearerTokenProvider {
        async fn get_token(&self) -> Result<BearerToken, CapabilityError> {
            unreachable!()
        }

        fn token_stream(&self) -> TokenStream {
            stream::empty().boxed_local()
        }
    }

    struct MockApiKeyProvider {}

    #[async_trait(?Send)]
    impl ApiKeyProvider for MockApiKeyProvider {
        async fn get_api_key(&self) -> Result<ApiKey, CapabilityError> {
            unreachable!()
        }

        fn api_key_stream(&self) -> ApiKeyStream {
            stream::empty().boxed_local()
        }
    }

    #[test]
    fn new_http_client_auth_provider_allows_empty_providers() {
        let auth = new_http_client_auth_provider_from_providers(vec![]).expect("success");

        assert!(auth.is_none())
    }

    #[test]
    fn new_http_client_auth_provider_allows_single_provider() {
        let auth = new_http_client_auth_provider_from_providers(vec![Box::new(BearerAuth::new(
            Box::new(MockBearerTokenProvider {}),
        ))])
        .expect("success");

        assert!(auth.is_some())
    }

    #[test]
    fn new_http_client_auth_provider_rejects_multiple_providers() {
        assert!(
            new_http_client_auth_provider_from_providers(vec![
                Box::new(BearerAuth::new(Box::new(MockBearerTokenProvider {}))),
                Box::new(ApiKeyAuth::new(Box::new(MockApiKeyProvider {}))),
            ])
            .is_err()
        )
    }
}
