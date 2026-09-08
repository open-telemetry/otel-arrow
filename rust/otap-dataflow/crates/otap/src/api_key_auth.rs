// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `api_key_provider` capability.

use std::str::FromStr;
use std::time::Instant;

use async_trait::async_trait;
use futures::StreamExt;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::capability::auth::api_key_provider::{
    API_KEY_USABLE_MARGIN, ApiKeyStream,
};
use otel_arrow_dfe_engine::local::capability::auth::api_key_provider::ApiKeyProvider;

use crate::http_client_auth_provider::*;

/// Consumer-side API Key authenticator: subscribes to a provider's API Key
/// stream, caches the built header, and reports usability.
///
/// All API Key expiry/stream state lives here, so an exporter holds one of these
/// and never touches an API Key directly.
pub struct ApiKeyAuth {
    /// Subscription to the provider's API Key refreshes.
    stream: ApiKeyStream,
    /// Whether the stream is still live and worth polling.
    stream_active: bool,
    /// The header built from the latest API Key.
    cached_header: Option<(HeaderName, HeaderValue)>,
    /// Expiry of the API Key behind `cached_header` (`None` = non-expiring).
    cached_expiry: Option<Instant>,
    /// Monotonically increasing id of the currently cached API Key, bumped on
    /// each successful refresh (starts at 0, meaning "no API Key yet"). Stamped
    /// onto each request so a later 401 can be matched to the exact API Key
    /// generation it used, letting a rejection for an already-replaced API Key
    /// be ignored.
    generation: u64,
}

impl ApiKeyAuth {
    /// Subscribes to `provider`'s API Key stream, raising warnings through
    /// `events`. Per the `ApiKeyProvider::api_key_stream` contract, a
    /// subscription created after an API Key has been published immediately yields
    /// that current API Key, so the exporter needs no separate `get_api_key()`
    /// seeding step.
    #[must_use]
    pub fn new(provider: Box<dyn ApiKeyProvider>) -> Self {
        Self {
            stream: provider.api_key_stream(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
        }
    }
}

#[async_trait(?Send)]
impl HttpClientAuthProvider for ApiKeyAuth {
    fn is_active(&self) -> bool {
        self.stream_active
    }

    fn is_ready(&self) -> bool {
        match (self.cached_header.is_some(), self.cached_expiry) {
            (false, _) => false,
            (true, None) => true, // non-expiring API Key
            (true, Some(expires_on)) => expires_on > Instant::now() + API_KEY_USABLE_MARGIN,
        }
    }

    fn not_ready_reason(&self) -> &'static str {
        if self.cached_header.is_some() {
            "api key at/near expiry; awaiting refresh"
        } else {
            "api key unavailable"
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
            .and_then(|expires_on| expires_on.checked_sub(API_KEY_USABLE_MARGIN))
    }

    fn invalidate(&mut self, generation: u64) {
        if generation == self.generation && self.cached_header.is_some() {
            self.cached_header = None;
            self.cached_expiry = None;
        }
    }

    async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents) {
        match self.stream.next().await {
            Some(api_key) => {
                let header_name = match api_key.get_http_header_name_attribute() {
                    Some(header) => match HeaderName::from_str(header) {
                        Ok(header) => header,
                        Err(_) => {
                            (events.invalid)(
                                "API Key configured HTTP header attribute is malformed",
                            );
                            return;
                        }
                    },
                    None => {
                        (events.invalid)("API Key HTTP header attribute not configured");
                        return;
                    }
                };

                let header_value = if let Some(scheme) = api_key.get_http_header_scheme_attribute()
                {
                    HeaderValue::from_str(&format!("{scheme} {}", api_key.expose_value()))
                } else {
                    HeaderValue::from_str(api_key.expose_value())
                };

                match header_value {
                    Ok(mut header_value) => {
                        // Redact in `Debug`, exclude from HPACK indexing.
                        header_value.set_sensitive(true);
                        self.cached_header = Some((header_name, header_value));
                        self.cached_expiry = api_key.get_expires_on();
                        // A new cached API Key starts a new generation, so a 401 for
                        // an earlier API Key no longer matches and is ignored.
                        self.generation = self.generation.wrapping_add(1);
                    }
                    Err(_) => {
                        // Malformed token: keep the previous cached API Key (if any).
                        (events.invalid)("Malformed API Key");
                    }
                }
            }
            None => {
                // Provider closed its stream; no further refreshes will arrive.
                // Keep using the last cached token. Not expected with a
                // watch-backed provider while we hold its handle, so warn.
                self.stream_active = false;
                (events.stream_closed)();
            }
        }
    }
}
