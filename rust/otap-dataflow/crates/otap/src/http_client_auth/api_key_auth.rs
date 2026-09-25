// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `api_key_provider` capability.

use std::str::FromStr;
use std::time::Duration;

use http::{HeaderName, HeaderValue};
use otel_arrow_dfe_engine::capability::auth::ApiKey;
use otel_arrow_dfe_engine::capability::auth::api_key_provider::API_KEY_USABLE_MARGIN;

use crate::http_client_auth::http_client_auth_provider::*;

const NAME: &str = "ApiKeyAuth";

pub struct ApiKeyHttpClientStreamAuthProviderBuilder {}

impl HttpClientStreamAuthProviderBuilder for ApiKeyHttpClientStreamAuthProviderBuilder {
    type Auth = ApiKey;

    const AUTH_NEAR_EXPIRY_NOT_READY_REASON: &'static str =
        "api key at/near expiry; awaiting refresh";

    const AUTH_UNAVAILABLE_NOT_READY_REASON: &'static str = "api key unavailable";

    const AUTH_USABLE_MARGIN: Duration = API_KEY_USABLE_MARGIN;

    fn name() -> HttpClientAuthProviderName {
        NAME.into()
    }

    fn build_auth_header(auth: Self::Auth) -> Result<HttpClientAuthHeader, String> {
        let header_name = match auth.get_http_header_name_attribute() {
            Some(header) => HeaderName::from_str(header).map_err(|e| {
                format!("API Key configured HTTP header attribute is malformed: {e}")
            })?,
            None => {
                return Err("API Key HTTP header attribute not configured".into());
            }
        };

        let header_value = if let Some(scheme) = auth.get_http_header_scheme_attribute() {
            HeaderValue::from_str(&format!("{scheme} {}", auth.expose_value()))
        } else {
            HeaderValue::from_str(auth.expose_value())
        }
        .map_err(|e| format!("Malformed API Key: {e}"))?;

        Ok(HttpClientAuthHeader::new(
            header_name,
            header_value,
            auth.get_expires_on(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use std::time::{Duration, Instant};

    use otel_arrow_dfe_engine::capability::auth::ApiKey;

    use super::{ApiKeyHttpClientStreamAuthProviderBuilder, HttpClientStreamAuthProviderBuilder};

    fn build_auth_header_and_return_error(api_key: ApiKey) -> String {
        ApiKeyHttpClientStreamAuthProviderBuilder::build_auth_header(api_key)
            .err()
            .expect("header should be rejected")
    }

    /// Scenario: an API key specifies a custom header, scheme, and known expiry.
    /// Guarantees: the builder emits the configured header and preserves the expiry.
    #[test]
    fn builds_schemed_header_and_preserves_expiry() {
        let expires_on = Instant::now() + Duration::from_secs(300);
        let api_key = ApiKey::new("secret-key")
            .with_http_header_name_attribute("x-api-key")
            .with_http_header_scheme_attribute("ApiKey")
            .with_expiry(expires_on);

        let header = ApiKeyHttpClientStreamAuthProviderBuilder::build_auth_header(api_key)
            .expect("header should be valid");

        assert_eq!(header.header_name, "x-api-key");
        assert_eq!(header.header_value, "ApiKey secret-key");
        assert_eq!(header.expires_on, Some(expires_on));
    }

    /// Scenario: an API key specifies a custom header without an authentication scheme.
    /// Guarantees: the builder uses the raw key value and reports its provider name.
    #[test]
    fn builds_unschemed_header_and_reports_provider_name() {
        let api_key = ApiKey::new("secret-key").with_http_header_name_attribute("x-api-key");

        let header = ApiKeyHttpClientStreamAuthProviderBuilder::build_auth_header(api_key)
            .expect("header should be valid");

        assert_eq!(header.header_value, "secret-key");
        assert_eq!(header.expires_on, None);
        assert_eq!(
            ApiKeyHttpClientStreamAuthProviderBuilder::name().as_ref(),
            "ApiKeyAuth"
        );
    }

    /// Scenario: an API key omits its required HTTP header-name attribute.
    /// Guarantees: the builder rejects the incomplete configuration with a clear error.
    #[test]
    fn rejects_missing_header_name() {
        let error = build_auth_header_and_return_error(ApiKey::new("secret-key"));

        assert_eq!(error, "API Key HTTP header attribute not configured");
    }

    /// Scenario: an API key specifies a malformed HTTP header name.
    /// Guarantees: the builder rejects the invalid name before constructing a header.
    #[test]
    fn rejects_malformed_header_name() {
        let api_key = ApiKey::new("secret-key").with_http_header_name_attribute("invalid header");

        let error = build_auth_header_and_return_error(api_key);

        assert!(error.starts_with("API Key configured HTTP header attribute is malformed:"));
    }

    /// Scenario: an API key value contains a character forbidden in HTTP header values.
    /// Guarantees: the builder rejects the key instead of producing an invalid header.
    #[test]
    fn rejects_malformed_api_key_value() {
        let api_key = ApiKey::new("bad\nkey").with_http_header_name_attribute("x-api-key");

        let error = build_auth_header_and_return_error(api_key);

        assert!(error.starts_with("Malformed API Key:"));
    }
}
