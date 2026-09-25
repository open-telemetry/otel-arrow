// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `bearer_token_provider` capability.
//!
//! Centralizes everything an exporter needs to authenticate outgoing requests
//! with a bearer token, so the exporter itself stays auth-agnostic: it drives
//! [`HttpClientAuthProvider::poll_refresh`] in its `select!` loop, asks
//! [`HttpClientAuthProvider::is_ready`] before admitting data, and stamps the
//! value returned by [`HttpClientAuthProvider::header`] onto each request. The
//! cached credential is an [`http::HeaderValue`], which both transports accept
//! (gRPC's `MetadataMap` is backed by an [`http::HeaderMap`]), so core and
//! contrib nodes on either protocol can share the provider implementation.
//!
//! The division of labor mirrors the capability design: the **provider**
//! (extension) owns credential acquisition, background refresh, and startup
//! readiness gating; the shared HTTP client auth provider subscribes to the
//! token stream, caches the built `Authorization` header, and tracks whether
//! that cached token is still usable. This module supplies the bearer-specific
//! conversion from a published token to that header.

use std::time::Duration;

use http::HeaderValue;
use otel_arrow_dfe_engine::capability::auth::BearerToken;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::TOKEN_USABLE_MARGIN;

use crate::http_client_auth::http_client_auth_provider::*;

const NAME: &str = "BearerAuth";

pub struct BearerHttpClientStreamAuthProviderBuilder {}

impl HttpClientStreamAuthProviderBuilder for BearerHttpClientStreamAuthProviderBuilder {
    type Auth = BearerToken;

    const AUTH_NEAR_EXPIRY_NOT_READY_REASON: &'static str =
        "bearer token at/near expiry; awaiting refresh";

    const AUTH_UNAVAILABLE_NOT_READY_REASON: &'static str = "bearer token unavailable";

    const AUTH_USABLE_MARGIN: Duration = TOKEN_USABLE_MARGIN;

    fn name() -> HttpClientAuthProviderName {
        NAME.into()
    }

    fn build_auth_header(auth: Self::Auth) -> Result<HttpClientAuthHeader, String> {
        let value = HeaderValue::from_str(&format!("Bearer {}", auth.expose_token()))
            .map_err(|e| format!("Malformed token: {e}"))?;

        Ok(HttpClientAuthHeader::new(
            http::header::AUTHORIZATION,
            value,
            auth.expires_on(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use http::header::AUTHORIZATION;
    use otel_arrow_dfe_engine::capability::auth::BearerToken;

    use super::{BearerHttpClientStreamAuthProviderBuilder, HttpClientStreamAuthProviderBuilder};

    /// Scenario: a bearer token has a known expiry.
    /// Guarantees: the builder emits the expected Authorization value and preserves expiry.
    #[test]
    fn builds_bearer_authorization_header_and_preserves_expiry() {
        let expires_on = Instant::now() + Duration::from_secs(300);
        let token = BearerToken::with_expiry("secret-token".to_owned(), Some(expires_on));

        let header = BearerHttpClientStreamAuthProviderBuilder::build_auth_header(token)
            .expect("header should be valid");

        assert_eq!(header.header_name, AUTHORIZATION);
        assert_eq!(header.header_value, "Bearer secret-token");
        assert_eq!(header.expires_on, Some(expires_on));
    }

    /// Scenario: a bearer token has no known expiry.
    /// Guarantees: the builder preserves the absent expiry and reports its provider name.
    #[test]
    fn supports_non_expiring_token_and_reports_provider_name() {
        let token = BearerToken::without_expiry("secret-token".to_owned());

        let header = BearerHttpClientStreamAuthProviderBuilder::build_auth_header(token)
            .expect("header should be valid");

        assert_eq!(header.expires_on, None);
        assert_eq!(
            BearerHttpClientStreamAuthProviderBuilder::name().as_ref(),
            "BearerAuth"
        );
    }

    /// Scenario: a bearer token contains a character forbidden in HTTP header values.
    /// Guarantees: the builder rejects the token instead of producing an invalid header.
    #[test]
    fn rejects_malformed_token() {
        let token = BearerToken::without_expiry("bad\ntoken".to_owned());

        let error = BearerHttpClientStreamAuthProviderBuilder::build_auth_header(token)
            .err()
            .expect("header should be rejected");

        assert!(error.starts_with("Malformed token:"));
    }
}
