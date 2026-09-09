// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `basic_auth_provider` capability.
//!
//! Centralizes everything an exporter needs to authenticate outgoing requests
//! with a credential, so the exporter itself stays auth-agnostic: it drives
//! [`BearerAuth::poll_refresh`] in its `select!` loop, asks
//! [`BearerAuth::is_ready`] before admitting data, and stamps
//! [`BearerAuth::header`] onto each request. The cached credential is an
//! `http::HeaderValue`, which both transports accept (tonic's `MetadataMap` is
//! backed by an `http::HeaderMap`), so core and contrib nodes on either
//! protocol can share this adapter.
//!
//! The division of labor mirrors the capability design: the **provider**
//! (extension) owns credential acquisition, background refresh, and startup
//! readiness gating; this **adapter** only subscribes to the provider's credential
//! stream, caches the built `Authorization` header, and tracks whether that
//! cached credential is still usable. The exporter is the "dumb caller".

use std::time::Instant;

use async_trait::async_trait;
use base64::{Engine as _, engine::general_purpose};
use futures::StreamExt;
use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::{
    BASIC_AUTH_CREDENTIAL_USABLE_MARGIN, BasicAuthCredentialStream,
};
use otel_arrow_dfe_engine::local::capability::auth::basic_auth_provider::BasicAuthProvider;

use crate::http_client_auth_provider::*;

/// Consumer-side basic-auth authenticator: subscribes to a provider's credential
/// stream, caches the built `Authorization` header, and reports usability.
///
/// All credential/expiry/stream state lives here, so an exporter holds one of these
/// and never touches a credential directly.
pub struct BasicAuth {
    /// Subscription to the provider's credential refreshes.
    stream: BasicAuthCredentialStream,
    /// Whether the stream is still live and worth polling.
    stream_active: bool,
    /// The `Authorization: Basic <base64(username:password)>` header built from the latest credential.
    cached_header: Option<HeaderValue>,
    /// Expiry of the credential behind `cached_header` (`None` = non-expiring).
    cached_expiry: Option<Instant>,
    /// Monotonically increasing id of the currently cached credential, bumped on each
    /// successful refresh (starts at 0, meaning "no credential yet"). Stamped onto
    /// each request so a later 401 can be matched to the exact credential generation
    /// it used, letting a rejection for an already-replaced credential be ignored.
    generation: u64,
}

impl BasicAuth {
    /// Subscribes to `provider`'s credential stream, raising warnings through
    /// `events`. Per the `BasicAuthProvider::credential_stream` contract, a
    /// subscription created after a credential has been published immediately yields
    /// that current credential, so the exporter needs no separate `get_credential()`
    /// seeding step.
    #[must_use]
    pub fn new(provider: Box<dyn BasicAuthProvider>) -> Self {
        Self {
            stream: provider.credential_stream(),
            stream_active: true,
            cached_header: None,
            cached_expiry: None,
            generation: 0,
        }
    }
}

#[async_trait(?Send)]
impl HttpClientAuthProvider for BasicAuth {
    fn is_active(&self) -> bool {
        self.stream_active
    }

    fn is_ready(&self) -> bool {
        match (self.cached_header.is_some(), self.cached_expiry) {
            (false, _) => false,
            (true, None) => true, // non-expiring credential
            (true, Some(expires_on)) => {
                expires_on > Instant::now() + BASIC_AUTH_CREDENTIAL_USABLE_MARGIN
            }
        }
    }

    fn not_ready_reason(&self) -> &'static str {
        if self.cached_header.is_some() {
            "credential at/near expiry; awaiting refresh"
        } else {
            "credential unavailable"
        }
    }

    fn header(&self) -> Option<(HeaderName, HeaderValue, u64)> {
        self.cached_header
            .clone()
            .map(|header| (http::header::AUTHORIZATION, header, self.generation))
    }

    fn refresh_deadline(&self) -> Option<Instant> {
        if !self.is_ready() {
            return None;
        }
        self.cached_expiry
            .and_then(|expires_on| expires_on.checked_sub(BASIC_AUTH_CREDENTIAL_USABLE_MARGIN))
    }

    fn invalidate(&mut self, generation: u64) {
        if generation == self.generation && self.cached_header.is_some() {
            self.cached_header = None;
            self.cached_expiry = None;
        }
    }

    async fn poll_refresh(&mut self, events: &HttpClientAuthProviderEvents) {
        match self.stream.next().await {
            Some(credential) => {
                let credentials = format!(
                    "{}:{}",
                    credential.expose_username(),
                    credential.expose_password()
                );
                let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

                match HeaderValue::from_str(&format!("Basic {encoded_credentials}")) {
                    Ok(mut value) => {
                        // Redact in `Debug`, exclude from HPACK indexing.
                        value.set_sensitive(true);
                        self.cached_header = Some(value);
                        self.cached_expiry = credential.expires_on();
                        // A new cached credential starts a new generation, so a 401 for
                        // an earlier credential no longer matches and is ignored.
                        self.generation = self.generation.wrapping_add(1);
                    }
                    Err(_) => {
                        // Malformed credential: keep the previous cached credential (if any).
                        (events.invalid)("Malformed credential");
                    }
                }
            }
            None => {
                // Provider closed its stream; no further refreshes will arrive.
                // Keep using the last cached credential. Not expected with a
                // watch-backed provider while we hold its handle, so warn.
                self.stream_active = false;
                (events.stream_closed)();
            }
        }
    }
}
