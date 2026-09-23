// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `agent_fed_auth_provider` capability.

use std::time::Duration;

use http::HeaderValue;
use otel_arrow_dfe_engine::capability::auth::{
    agent_fed_credential_provider::AgentFedCredentialSnapshot,
    bearer_token_provider::TOKEN_USABLE_MARGIN,
};

use crate::http_client_auth::http_client_auth_provider::*;

const NAME: &str = "AgentFedAuth";

pub struct AgentFedHttpClientStreamAuthProviderBuilder {}

impl HttpClientStreamAuthProviderBuilder for AgentFedHttpClientStreamAuthProviderBuilder {
    type Auth = AgentFedCredentialSnapshot;

    const AUTH_NEAR_EXPIRY_NOT_READY_REASON: &'static str =
        "agent-fed bearer token at/near expiry; awaiting refresh";

    const AUTH_UNAVAILABLE_NOT_READY_REASON: &'static str = "agent-fed bearer token unavailable";

    const AUTH_USABLE_MARGIN: Duration = TOKEN_USABLE_MARGIN;

    fn name() -> HttpClientAuthProviderName {
        NAME.into()
    }

    fn build_auth_header(auth: Self::Auth) -> Result<HttpClientAuthHeader, String> {
        let token = auth.token();

        let value = HeaderValue::from_str(&format!("Bearer {}", token.expose_token()))
            .map_err(|e| format!("Malformed token: {e}"))?;

        Ok(HttpClientAuthHeader::new(
            http::header::AUTHORIZATION,
            value,
            token.expires_on(),
        ))
    }
}
