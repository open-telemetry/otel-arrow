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
