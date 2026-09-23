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
