// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{
    borrow::Cow,
    pin::Pin,
    task::{Context, Poll},
    time::{Duration, Instant},
};

use bitflags::bitflags;
use futures::Stream;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::{
    capability::{ExtensionCapability, registry::Capabilities},
    local::capability::auth::bearer_token_provider::BearerTokenProvider,
};

use crate::http_client_auth::{
    agent_fed_auth::AgentFedHttpClientStreamAuthProviderBuilder,
    api_key_auth::ApiKeyHttpClientStreamAuthProviderBuilder,
    basic_auth::BasicHttpClientStreamAuthProviderBuilder,
    bearer_auth::BearerHttpClientStreamAuthProviderBuilder,
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
    fn poll_refresh(
        &mut self,
        cx: &mut Context<'_>,
        events: &HttpClientAuthProviderEvents,
    ) -> Poll<bool>;
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

/// An abstraction over [`Capabilities`] to enable testing.
trait CapabilityResolver {
    /// See [`Capabilities::optional_local`].
    fn optional_local<C: ExtensionCapability>(
        &self,
    ) -> Result<Option<Box<C::Local>>, otel_arrow_dfe_engine::capability::registry::Error>;
}

impl CapabilityResolver for Capabilities {
    fn optional_local<C: ExtensionCapability>(
        &self,
    ) -> Result<Option<Box<C::Local>>, otel_arrow_dfe_engine::capability::registry::Error> {
        self.optional_local::<C>()
    }
}

/// Create an [`HttpClientAuthProvider`] using the registered [`Capabilities`].
pub fn new_http_client_auth_provider(
    capabilities: &Capabilities,
    supported_providers: HttpClientAuthProviders,
) -> Result<Option<Box<dyn HttpClientAuthProvider>>, otel_arrow_dfe_config::error::Error> {
    new_http_client_auth_provider_from_resolver(capabilities, supported_providers)
}

/// Create an [`HttpClientAuthProvider`] using the provided [`BearerTokenProvider`].
pub fn new_http_client_auth_provider_from_bearer_token_provider(
    bearer_token_provider: Box<dyn BearerTokenProvider>,
) -> impl HttpClientAuthProvider {
    HttpClientStreamAuthProvider::<BearerHttpClientStreamAuthProviderBuilder>::new(
        bearer_token_provider.token_stream(),
    )
}

fn new_http_client_auth_provider_from_resolver<T: CapabilityResolver>(
    capabilities: &T,
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
            .map(|p| {
                HttpClientStreamAuthProvider::<BearerHttpClientStreamAuthProviderBuilder>::new(
                    p.token_stream()
                )
            }) {
            providers.push(Box::new(bearer_auth));
        }

        // Optionally resolve an agent fed credential provider. A bound provider supplies refreshed bearer tokens.
        if let Some(agent_fed_auth) = capabilities
            .optional_local::<otel_arrow_dfe_engine::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider>()
            .map_err(|e| otel_arrow_dfe_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            })?
            .map(|p| {
                HttpClientStreamAuthProvider::<AgentFedHttpClientStreamAuthProviderBuilder>::new(
                    p.credential_stream()
                )
            }) {
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
            .map(|p| {
                HttpClientStreamAuthProvider::<ApiKeyHttpClientStreamAuthProviderBuilder>::new(
                    p.api_key_stream()
                )
            }) {
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
            .map(|p| {
                HttpClientStreamAuthProvider::<BasicHttpClientStreamAuthProviderBuilder>::new(
                    p.credential_stream()
                )
            }) {
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

pub(crate) struct HttpClientAuthHeader {
    pub header_name: HeaderName,
    pub header_value: HeaderValue,
    pub expires_on: Option<Instant>,
}

impl HttpClientAuthHeader {
    pub fn new(
        header_name: HeaderName,
        header_value: HeaderValue,
        expires_on: Option<Instant>,
    ) -> HttpClientAuthHeader {
        Self {
            header_name,
            header_value,
            expires_on,
        }
    }
}

pub(crate) trait HttpClientStreamAuthProviderBuilder {
    type Auth;
    const AUTH_NEAR_EXPIRY_NOT_READY_REASON: &'static str;
    const AUTH_UNAVAILABLE_NOT_READY_REASON: &'static str;
    const AUTH_USABLE_MARGIN: Duration;

    fn name() -> HttpClientAuthProviderName;

    fn build_auth_header(auth: Self::Auth) -> Result<HttpClientAuthHeader, String>;
}

pub(crate) struct HttpClientStreamAuthProvider<TProvider: HttpClientStreamAuthProviderBuilder> {
    /// Subscription to the provider's auth refreshes.
    stream: Pin<Box<dyn Stream<Item = TProvider::Auth> + 'static>>,
    /// Whether the stream is still live and worth polling.
    stream_active: bool,
    /// The header built from the latest auth.
    cached_header: Option<(HeaderName, HeaderValue)>,
    /// Expiry of the auth behind `cached_header` (`None` = non-expiring).
    cached_expiry: Option<Instant>,
    /// Monotonically increasing id of the currently cached auth, bumped on each
    /// successful refresh (starts at 0, meaning "no auth yet"). Stamped onto
    /// each request so a later 401 can be matched to the exact auth generation
    /// it used, letting a rejection for an already-replaced auth be ignored.
    generation: u64,
}

impl<TProvider: HttpClientStreamAuthProviderBuilder> HttpClientStreamAuthProvider<TProvider> {
    pub fn new(
        stream: Pin<Box<dyn Stream<Item = TProvider::Auth> + 'static>>,
    ) -> HttpClientStreamAuthProvider<TProvider> {
        Self {
            stream,
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
        }
    }
}

impl<TProvider: HttpClientStreamAuthProviderBuilder> HttpClientAuthProvider
    for HttpClientStreamAuthProvider<TProvider>
{
    fn name(&self) -> HttpClientAuthProviderName {
        TProvider::name()
    }

    fn is_active(&self) -> bool {
        self.stream_active
    }

    fn is_ready(&self) -> bool {
        match (self.cached_header.is_some(), self.cached_expiry) {
            (false, _) => false,
            (true, None) => true, // non-expiring API Key
            (true, Some(expires_on)) => expires_on > Instant::now() + TProvider::AUTH_USABLE_MARGIN,
        }
    }

    fn not_ready_reason(&self) -> &'static str {
        if self.cached_header.is_some() {
            TProvider::AUTH_NEAR_EXPIRY_NOT_READY_REASON
        } else {
            TProvider::AUTH_UNAVAILABLE_NOT_READY_REASON
        }
    }

    fn header(&self) -> Option<(HeaderName, HeaderValue, u64)> {
        self.cached_header
            .clone()
            .map(|(name, value)| (name, value, self.generation))
    }

    fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_expiry
            .and_then(|expires_on| expires_on.checked_sub(TProvider::AUTH_USABLE_MARGIN))
    }

    fn invalidate(&mut self, generation: u64) {
        if generation == self.generation && self.cached_header.is_some() {
            self.cached_header = None;
            self.cached_expiry = None;
        }
    }

    fn poll_refresh(
        &mut self,
        cx: &mut Context<'_>,
        events: &HttpClientAuthProviderEvents,
    ) -> Poll<bool> {
        match self.stream.as_mut().poll_next(cx) {
            Poll::Pending => Poll::Pending,
            Poll::Ready(Some(auth)) => {
                match TProvider::build_auth_header(auth) {
                    Ok(mut auth_header) => {
                        // Redact in `Debug`, exclude from HPACK indexing.
                        auth_header.header_value.set_sensitive(true);
                        self.cached_header =
                            Some((auth_header.header_name, auth_header.header_value));
                        self.cached_expiry = auth_header.expires_on;
                        // A new cached auth starts a new generation, so a 401 for
                        // an earlier auth no longer matches and is ignored.
                        self.generation = self.generation.wrapping_add(1);
                        Poll::Ready(true)
                    }
                    Err(e) => {
                        // Malformed header: keep the previous cached auth (if any).
                        events.emit_invalid(self, &e);
                        Poll::Ready(false)
                    }
                }
            }
            Poll::Ready(None) => {
                // Provider closed its stream; no further refreshes will arrive.
                // Keep using the last cached auth. Not expected with a
                // watch-backed provider while we hold its handle, so warn.
                self.stream_active = false;
                events.emit_stream_closed(self);
                Poll::Ready(false)
            }
        }
    }
}

#[cfg(any(test, feature = "test-utils"))]
pub mod test_support {
    //! Test doubles shared by the nodes that consume this adapter, so every
    //! consumer's test suite drives the same provider behavior instead of each
    //! maintaining its own copy.

    use futures::{Stream, StreamExt};
    use http::header;
    use std::{
        pin::Pin,
        sync::{Arc, atomic::AtomicBool},
        time::Duration,
    };

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
        closed_flag: Option<Arc<AtomicBool>>,
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
                closed_flag: None,
            }
        }

        /// Create a provider which closes when it has issued all its values.
        #[must_use]
        pub fn closing(
            header_name: HeaderName,
            header_values: Vec<(String, Option<Duration>)>,
            closed: Arc<AtomicBool>,
        ) -> MockHttpClientAuthProvider {
            let published = futures::stream::iter(header_values);

            Self {
                stream: published.boxed(),
                header_name,
                cached_header: None,
                cached_expiry: None,
                generation: 0,
                stream_active: true,
                closed_flag: Some(closed),
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
                closed_flag: None,
            }
        }
    }

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

        fn poll_refresh(
            &mut self,
            cx: &mut Context<'_>,
            events: &HttpClientAuthProviderEvents,
        ) -> Poll<bool> {
            match self.stream.as_mut().poll_next(cx) {
                Poll::Pending => Poll::Pending,
                Poll::Ready(Some((value, duration))) => {
                    match HeaderValue::from_str(&value) {
                        Ok(mut value) => {
                            // Redact in `Debug`, exclude from HPACK indexing.
                            value.set_sensitive(true);
                            self.cached_header = Some(value);
                            self.cached_expiry = duration.map(|v| Instant::now() + v);
                            // A new cached token starts a new generation, so a 401 for
                            // an earlier token no longer matches and is ignored.
                            self.generation = self.generation.wrapping_add(1);
                            Poll::Ready(true)
                        }
                        Err(e) => {
                            // Malformed token: keep the previous cached token (if any).
                            events.emit_invalid(self, &format!("Malformed token: {e}"));
                            Poll::Ready(false)
                        }
                    }
                }
                Poll::Ready(None) => {
                    // Provider closed its stream; no further refreshes will arrive.
                    // Keep using the last cached token. Not expected with a
                    // watch-backed provider while we hold its handle, so warn.
                    self.stream_active = false;
                    events.emit_stream_closed(self);
                    _ = self
                        .closed_flag
                        .as_ref()
                        .inspect(|v| v.store(true, std::sync::atomic::Ordering::Release));
                    Poll::Ready(false)
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::{
        any::{Any, TypeId},
        cell::{Cell, RefCell},
        collections::HashMap,
        future::poll_fn,
        sync::Arc,
    };

    use otel_arrow_dfe_engine::{
        capability::{
            CapabilityError,
            auth::{
                agent_fed_credential_provider::{
                    AgentFedCredentialSnapshot, AgentFedCredentialSnapshotStream,
                },
                api_key_provider::ApiKeyStream,
                basic_auth_provider::BasicAuthCredentialStream,
                bearer_token_provider::TokenStream,
                *,
            },
        },
        local::capability::auth::{
            agent_fed_credential_provider::AgentFedCredentialProvider,
            api_key_provider::ApiKeyProvider, basic_auth_provider::BasicAuthProvider,
            bearer_token_provider::BearerTokenProvider,
        },
    };
    use tonic::async_trait;

    use crate::http_client_auth::http_client_auth_provider::test_support::MockHttpClientAuthProvider;

    use super::*;
    use futures::StreamExt;
    use futures::stream;

    struct MockCapabilities {
        registrations: RefCell<HashMap<TypeId, Box<dyn Any>>>,
    }

    impl MockCapabilities {
        pub fn new() -> MockCapabilities {
            Self {
                registrations: HashMap::new().into(),
            }
        }

        pub fn with_local<C: ExtensionCapability>(self, provider: Box<C::Local>) -> Self {
            let type_id = TypeId::of::<C>();

            let any: Box<dyn Any> = Box::new(provider);

            _ = self.registrations.borrow_mut().insert(type_id, any);

            self
        }
    }

    impl CapabilityResolver for MockCapabilities {
        fn optional_local<C: ExtensionCapability>(
            &self,
        ) -> Result<Option<Box<C::Local>>, otel_arrow_dfe_engine::capability::registry::Error>
        {
            let type_id = TypeId::of::<C>();

            match self.registrations.borrow_mut().remove(&type_id) {
                None => Ok(None),
                Some(any) => match any.downcast::<Box<C::Local>>().map(|v| *v) {
                    Ok(typed) => Ok(Some(typed)),
                    Err(_) => {
                        panic!("Capability type mismatch")
                    }
                },
            }
        }
    }

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

    struct MockAgentFedCredentialProvider {}

    #[async_trait(?Send)]
    impl AgentFedCredentialProvider for MockAgentFedCredentialProvider {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            unreachable!()
        }

        fn credential_stream(&self) -> AgentFedCredentialSnapshotStream {
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

    struct MockBasicAuthProvider {}

    #[async_trait(?Send)]
    impl BasicAuthProvider for MockBasicAuthProvider {
        async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError> {
            unreachable!()
        }

        fn credential_stream(&self) -> BasicAuthCredentialStream {
            stream::empty().boxed_local()
        }
    }

    #[test]
    fn resolve_bearer_auth_from_bearer_token_provider() {
        let capabilities = MockCapabilities::new()
            .with_local::<bearer_token_provider::BearerTokenProvider>(Box::new(
                MockBearerTokenProvider {},
            ));

        assert!(
            new_http_client_auth_provider_from_resolver(
                &capabilities,
                HttpClientAuthProviders::empty()
            )
            .unwrap()
            .is_none()
        );
        assert!(
            new_http_client_auth_provider_from_resolver(
                &capabilities,
                HttpClientAuthProviders::BEARER_TOKEN
            )
            .unwrap()
            .is_some()
        );
    }

    #[test]
    fn resolve_bearer_auth_from_agent_fed_credential_provider() {
        let capabilities = MockCapabilities::new()
            .with_local::<agent_fed_credential_provider::AgentFedCredentialProvider>(
            Box::new(MockAgentFedCredentialProvider {}),
        );

        assert!(
            new_http_client_auth_provider_from_resolver(
                &capabilities,
                HttpClientAuthProviders::empty()
            )
            .unwrap()
            .is_none()
        );
        assert!(
            new_http_client_auth_provider_from_resolver(
                &capabilities,
                HttpClientAuthProviders::BEARER_TOKEN
            )
            .unwrap()
            .is_some()
        );
    }

    #[test]
    fn resolve_api_key_auth_from_api_key_provider() {
        let capabilities = MockCapabilities::new()
            .with_local::<api_key_provider::ApiKeyProvider>(Box::new(MockApiKeyProvider {}));

        assert!(
            new_http_client_auth_provider_from_resolver(
                &capabilities,
                HttpClientAuthProviders::empty()
            )
            .unwrap()
            .is_none()
        );
        assert!(
            new_http_client_auth_provider_from_resolver(
                &capabilities,
                HttpClientAuthProviders::API_KEY
            )
            .unwrap()
            .is_some()
        );
    }

    #[test]
    fn resolve_basic_auth_from_basic_auth_provider() {
        let capabilities = MockCapabilities::new()
            .with_local::<basic_auth_provider::BasicAuthProvider>(Box::new(
                MockBasicAuthProvider {},
            ));

        assert!(
            new_http_client_auth_provider_from_resolver(
                &capabilities,
                HttpClientAuthProviders::empty()
            )
            .unwrap()
            .is_none()
        );
        assert!(
            new_http_client_auth_provider_from_resolver(
                &capabilities,
                HttpClientAuthProviders::BASIC
            )
            .unwrap()
            .is_some()
        );
    }

    #[test]
    fn new_http_client_auth_provider_allows_empty_providers() {
        let auth = new_http_client_auth_provider_from_providers(vec![]).expect("success");

        assert!(auth.is_none())
    }

    #[test]
    fn new_http_client_auth_provider_allows_single_provider() {
        let auth = new_http_client_auth_provider_from_providers(vec![Box::new(
            MockHttpClientAuthProvider::never_publishes(),
        )])
        .expect("success");

        assert!(auth.is_some())
    }

    #[test]
    fn new_http_client_auth_provider_rejects_multiple_providers() {
        assert!(
            new_http_client_auth_provider_from_providers(vec![
                Box::new(MockHttpClientAuthProvider::never_publishes()),
                Box::new(MockHttpClientAuthProvider::never_publishes()),
            ])
            .is_err()
        )
    }

    struct MockHttpClientStreamAuthProviderBuilder {}

    impl HttpClientStreamAuthProviderBuilder for MockHttpClientStreamAuthProviderBuilder {
        type Auth = BearerToken;

        const AUTH_NEAR_EXPIRY_NOT_READY_REASON: &'static str =
            "mock token at/near expiry; awaiting refresh";

        const AUTH_UNAVAILABLE_NOT_READY_REASON: &'static str = "mock token unavailable";

        const AUTH_USABLE_MARGIN: Duration = Duration::from_secs(10);

        fn name() -> HttpClientAuthProviderName {
            "MockAuth".into()
        }

        fn build_auth_header(auth: Self::Auth) -> Result<HttpClientAuthHeader, String> {
            let value = match HeaderValue::from_str(&format!("Bearer {}", auth.expose_token())) {
                Ok(value) => value,
                Err(e) => {
                    return Err(e.to_string());
                }
            };
            Ok(HttpClientAuthHeader::new(
                http::header::AUTHORIZATION,
                value,
                auth.expires_on(),
            ))
        }
    }

    thread_local! {
        /// Number of `invalid` notifications raised on this test thread.
        static INVALID: Cell<usize> = const { Cell::new(0) };
        /// Number of `stream_closed` notifications raised on this test thread.
        static STREAM_CLOSURES: Cell<usize> = const { Cell::new(0) };
    }

    /// Recording event hooks. The hooks take no receiver, so the counters are
    /// thread-local; the test harness gives each test its own thread, and every
    /// test resets them before use.
    const TEST_EVENTS: HttpClientAuthProviderEvents = HttpClientAuthProviderEvents {
        invalid: |_, _| INVALID.set(INVALID.get() + 1),
        stream_closed: |_| STREAM_CLOSURES.set(STREAM_CLOSURES.get() + 1),
    };

    fn reset_events() {
        INVALID.set(0);
        STREAM_CLOSURES.set(0);
    }

    /// Builds an adapter holding a usable, non-expiring token at `generation`,
    /// with an inert (empty) stream so only `invalidate` behavior is exercised.
    fn auth_with_cached_token(
        generation: u64,
    ) -> HttpClientStreamAuthProvider<MockHttpClientStreamAuthProviderBuilder> {
        HttpClientStreamAuthProvider {
            stream: stream::empty().boxed_local(),
            stream_active: false,
            cached_header: Some((
                http::header::AUTHORIZATION,
                HeaderValue::from_static("Bearer test-token"),
            )),
            cached_expiry: None,
            generation,
        }
    }

    /// Builds a token-less adapter subscribed to a finite stream that publishes
    /// `tokens` in order and then ends, so a test can drive `poll_refresh` one
    /// publication at a time and also reach the stream-closed branch.
    fn auth_over(
        tokens: Vec<BearerToken>,
    ) -> HttpClientStreamAuthProvider<MockHttpClientStreamAuthProviderBuilder> {
        reset_events();
        HttpClientStreamAuthProvider {
            stream: stream::iter(tokens).boxed_local(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
        }
    }

    /// Scenario: a 401 names the token generation currently cached.
    /// Guarantees: the rejected token is dropped so intake back-pressures until the
    /// provider's next publication, instead of the rejected token being sent again.
    #[test]
    fn invalidate_drops_the_matching_generation() {
        let mut auth = auth_with_cached_token(7);
        assert!(auth.is_ready());

        auth.invalidate(7);

        assert!(
            !auth.is_ready(),
            "a 401 for the cached generation must clear the token"
        );
    }

    /// Scenario: a 401 names an older generation than the one now cached, i.e. a
    /// newer token was published after the failing request was sent.
    /// Guarantees: the still-valid current token is kept, so a stale rejection
    /// does not stall exports until an unnecessary extra refresh.
    #[test]
    fn invalidate_ignores_a_stale_generation() {
        let mut auth = auth_with_cached_token(7);

        auth.invalidate(6);

        assert!(
            auth.is_ready(),
            "a 401 for a superseded generation must not clear the newer token"
        );
    }

    /// Scenario: the provider publishes its first token on the subscription.
    /// Guarantees: the adapter caches an `Authorization: Bearer <token>` header,
    /// marks it sensitive so it is redacted in `Debug` and excluded from the
    /// HPACK dynamic table, reports readiness, and stamps a non-zero generation
    /// so a later rejection can name exactly this token.
    #[tokio::test]
    async fn poll_refresh_caches_the_published_token_as_a_sensitive_header() {
        let mut auth = auth_over(vec![BearerToken::without_expiry("first")]);

        assert!(poll_fn(|cx| auth.poll_refresh(cx, &TEST_EVENTS)).await);

        assert!(
            auth.is_ready(),
            "a published token must make the adapter ready"
        );
        let (_, header, generation) = auth.header().expect("a cached token must yield a header");
        assert_eq!(header.to_str().unwrap(), "Bearer first");
        assert!(
            header.is_sensitive(),
            "the credential must be marked sensitive so it is never HPACK-indexed"
        );
        assert_eq!(
            generation, 1,
            "the first cached token must not reuse the \
            'no token yet' generation, so a rejection can be attributed"
        );
    }

    /// Scenario: a refresh publishes a token whose bytes cannot form a header
    /// value, while a usable token is already cached.
    /// Guarantees: the malformed publication is reported and dropped, and the
    /// previously cached token keeps being used at its own generation, so a
    /// single bad refresh cannot stall exports.
    #[tokio::test]
    async fn a_malformed_refresh_is_reported_and_leaves_the_cached_token_intact() {
        let mut auth = auth_over(vec![
            BearerToken::without_expiry("good"),
            BearerToken::without_expiry("bad\nvalue"),
        ]);

        assert!(poll_fn(|cx| auth.poll_refresh(cx, &TEST_EVENTS)).await);
        assert!(!poll_fn(|cx| auth.poll_refresh(cx, &TEST_EVENTS)).await);

        assert_eq!(
            INVALID.get(),
            1,
            "a token that cannot become a header value must be reported"
        );
        let (_, header, generation) = auth.header().expect("the earlier token must be kept");
        assert_eq!(header.to_str().unwrap(), "Bearer good");
        assert_eq!(
            generation, 1,
            "a rejected publication must not advance the generation"
        );
    }

    /// Scenario: the provider closes its token stream after publishing a token.
    /// Guarantees: the closure is reported, the adapter stops advertising itself
    /// as pollable so the exporter's `select!` arm goes quiet instead of
    /// busy-looping on a dead stream, and the last token stays usable.
    #[tokio::test]
    async fn a_closed_stream_is_reported_and_the_last_token_stays_usable() {
        let mut auth = auth_over(vec![BearerToken::without_expiry("last")]);

        assert!(poll_fn(|cx| auth.poll_refresh(cx, &TEST_EVENTS)).await);
        assert!(!poll_fn(|cx| auth.poll_refresh(cx, &TEST_EVENTS)).await);

        assert_eq!(
            STREAM_CLOSURES.get(),
            1,
            "the provider closing its stream must be reported"
        );
        assert!(
            !auth.is_active(),
            "a closed stream must not be polled again"
        );
        assert!(
            auth.is_ready(),
            "closing the stream must not discard the last usable token"
        );
    }

    /// Scenario: no token has been published yet.
    /// Guarantees: the adapter is not ready, hands back no header to stamp, arms
    /// no refresh timer, and reports the reason that distinguishes "never
    /// arrived" from "expiring", so the NACK text tells an operator which it is.
    #[test]
    fn an_adapter_without_a_token_is_unusable_and_says_why() {
        let auth = auth_over(vec![]);

        assert!(!auth.is_ready());
        assert!(auth.header().is_none());
        assert!(auth.refresh_deadline().is_none());
        assert_eq!(
            auth.not_ready_reason(),
            "agent-fed bearer token unavailable"
        );
    }

    /// Scenario: the cached token is still valid but expires inside the
    /// usability margin.
    /// Guarantees: it is treated as unusable so the exporter back-pressures
    /// rather than sending a request that could outlive its token, no refresh
    /// timer is armed for an already-lapsed margin, and the reason names expiry.
    #[tokio::test]
    async fn a_token_inside_the_usability_margin_is_not_usable() {
        let mut auth = auth_over(vec![BearerToken::with_expiry(
            "expiring",
            Some(Instant::now() + MockHttpClientStreamAuthProviderBuilder::AUTH_USABLE_MARGIN / 2),
        )]);

        assert!(poll_fn(|cx| auth.poll_refresh(cx, &TEST_EVENTS)).await);

        assert!(
            !auth.is_ready(),
            "a token inside the usability margin must gate intake"
        );
        assert!(
            auth.refresh_deadline().is_none(),
            "an already-lapsed margin must arm no timer"
        );
        assert_eq!(
            auth.not_ready_reason(),
            "agent-fed bearer token at/near expiry; awaiting refresh"
        );
    }

    /// Scenario: the cached token expires comfortably beyond the usability
    /// margin.
    /// Guarantees: it is usable now, and the reported deadline is exactly the
    /// instant readiness flips, so the exporter wakes to gate intake before a
    /// near-expiry batch is admitted rather than after.
    #[tokio::test]
    async fn refresh_deadline_is_the_instant_readiness_lapses() {
        let expires_on =
            Instant::now() + MockHttpClientStreamAuthProviderBuilder::AUTH_USABLE_MARGIN * 10;
        let mut auth = auth_over(vec![BearerToken::with_expiry(
            "long-lived",
            Some(expires_on),
        )]);

        assert!(poll_fn(|cx| auth.poll_refresh(cx, &TEST_EVENTS)).await);

        assert!(auth.is_ready());
        assert_eq!(
            auth.refresh_deadline(),
            Some(expires_on - MockHttpClientStreamAuthProviderBuilder::AUTH_USABLE_MARGIN),
            "the timer must fire when the token enters the usability margin"
        );
    }

    /// Scenario: the provider publishes a token with no known expiry.
    /// Guarantees: it is usable and arms no refresh timer, so the exporter does
    /// not register a timer that can never be justified by an expiry.
    #[tokio::test]
    async fn a_non_expiring_token_arms_no_refresh_deadline() {
        let mut auth = auth_over(vec![BearerToken::without_expiry("forever")]);

        assert!(poll_fn(|cx| auth.poll_refresh(cx, &TEST_EVENTS)).await);

        assert!(auth.is_ready());
        assert!(auth.refresh_deadline().is_none());
    }

    /// Scenario: a completed export reports the generation the server rejected.
    /// Guarantees: the exporter's rejection hand-off drops exactly that token, so
    /// the retry waits for the provider's next publication instead of replaying
    /// the rejected credential.
    #[test]
    fn apply_auth_rejection_drops_the_reported_generation() {
        let mut auth: Option<Box<dyn HttpClientAuthProvider>> =
            Some(Box::new(auth_with_cached_token(3)));

        apply_auth_rejection(&mut auth, Some(3));

        assert!(!auth.expect("the adapter is retained").is_ready());
    }

    /// Scenario: an export completes without naming a rejected generation (it
    /// succeeded, or failed for a non-auth reason).
    /// Guarantees: the cached token survives, so ordinary transport failures do
    /// not stall intake behind an unnecessary refresh.
    #[test]
    fn apply_auth_rejection_keeps_the_token_when_nothing_was_rejected() {
        let mut auth: Option<Box<dyn HttpClientAuthProvider>> =
            Some(Box::new(auth_with_cached_token(3)));

        apply_auth_rejection(&mut auth, None);

        assert!(auth.expect("the adapter is retained").is_ready());
    }

    /// Scenario: no provider is bound, so the exporter holds no adapter.
    /// Guarantees: the shared rejection hand-off is a no-op rather than a panic,
    /// which is what lets the exporter call it unconditionally on every
    /// completion.
    #[test]
    fn apply_auth_rejection_without_a_bound_provider_is_a_no_op() {
        let mut auth: Option<Box<dyn HttpClientAuthProvider>> = None;

        apply_auth_rejection(&mut auth, Some(1));

        assert!(auth.is_none());
    }
}
