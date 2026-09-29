// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `basic_auth_provider` capability.

use std::time::Duration;

use base64::{Engine as _, engine::general_purpose};
use http::HeaderValue;
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BASIC_AUTH_CREDENTIAL_USABLE_MARGIN;

use crate::http_client_auth::http_client_auth_provider::*;

const NAME: &str = "BasicAuth";

pub struct BasicHttpClientStreamAuthProviderBuilder {}

impl HttpClientStreamAuthProviderBuilder for BasicHttpClientStreamAuthProviderBuilder {
    type Auth = BasicAuthCredential;

    const AUTH_NEAR_EXPIRY_NOT_READY_REASON: &'static str =
        "credential at/near expiry; awaiting refresh";

    const AUTH_UNAVAILABLE_NOT_READY_REASON: &'static str = "credential unavailable";

    const AUTH_USABLE_MARGIN: Duration = BASIC_AUTH_CREDENTIAL_USABLE_MARGIN;

    fn name() -> HttpClientAuthProviderName {
        NAME.into()
    }

    fn build_auth_header(auth: Self::Auth) -> Result<HttpClientAuthHeader, String> {
        let credentials = format!("{}:{}", auth.expose_username(), auth.expose_password());
        let encoded_credentials = general_purpose::STANDARD.encode(credentials.as_bytes());

        let value = HeaderValue::from_str(&format!("Basic {encoded_credentials}"))
            .map_err(|e| format!("Malformed credential: {e}"))?;

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

    use base64::{Engine as _, engine::general_purpose};
    use http::header::AUTHORIZATION;
    use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;

    use super::{BasicHttpClientStreamAuthProviderBuilder, HttpClientStreamAuthProviderBuilder};

    /// Scenario: a basic-auth credential with a known expiry is converted into an HTTP header.
    /// Guarantees: the builder emits the expected Authorization value and preserves expiry.
    #[test]
    fn builds_basic_authorization_header_and_preserves_expiry() {
        let expires_on = Instant::now() + Duration::from_secs(300);
        let credential = BasicAuthCredential::new("Aladdin", "open sesame")
            .expect("credential should be valid")
            .with_expiry(expires_on);

        let header = BasicHttpClientStreamAuthProviderBuilder::build_auth_header(credential)
            .expect("header should be valid");

        assert_eq!(header.header_name, AUTHORIZATION);
        assert_eq!(header.header_value, "Basic QWxhZGRpbjpvcGVuIHNlc2FtZQ==");
        assert_eq!(header.expires_on, Some(expires_on));
    }

    /// Scenario: a non-expiring basic-auth credential contains colons in its password.
    /// Guarantees: the builder separates username and password once and reports its provider name.
    #[test]
    fn supports_colons_in_password_and_reports_provider_name() {
        let credential =
            BasicAuthCredential::new("user", "part:part").expect("credential should be valid");

        let header = BasicHttpClientStreamAuthProviderBuilder::build_auth_header(credential)
            .expect("header should be valid");
        let encoded = header
            .header_value
            .to_str()
            .expect("header should contain ASCII")
            .strip_prefix("Basic ")
            .expect("header should use the Basic scheme");
        let decoded = general_purpose::STANDARD
            .decode(encoded)
            .expect("credentials should be Base64 encoded");

        assert_eq!(decoded, b"user:part:part");
        assert_eq!(header.expires_on, None);
        assert_eq!(
            BasicHttpClientStreamAuthProviderBuilder::name().as_ref(),
            "BasicAuth"
        );
    }
}
