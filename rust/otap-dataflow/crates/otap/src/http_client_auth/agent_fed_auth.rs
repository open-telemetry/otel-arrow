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

#[cfg(test)]
mod tests {
    use std::{
        sync::Arc,
        time::{Duration, Instant},
    };

    use http::header::AUTHORIZATION;
    use otel_arrow_dfe_engine::capability::auth::{
        BearerToken, agent_fed_credential_provider::AgentFedCredentialSnapshot,
    };
    use serde_json::Map;

    use super::{AgentFedHttpClientStreamAuthProviderBuilder, HttpClientStreamAuthProviderBuilder};

    fn snapshot(token: BearerToken) -> AgentFedCredentialSnapshot {
        AgentFedCredentialSnapshot::new(token, Arc::new(Map::new()))
    }

    /// Scenario: an agent-fed snapshot contains a bearer token with a known expiry.
    /// Guarantees: the builder emits the expected Authorization value and preserves expiry.
    #[test]
    fn builds_bearer_authorization_header_and_preserves_expiry() {
        let expires_on = Instant::now() + Duration::from_secs(300);
        let credential = snapshot(BearerToken::with_expiry(
            "agent-fed-token".to_owned(),
            Some(expires_on),
        ));

        let header = AgentFedHttpClientStreamAuthProviderBuilder::build_auth_header(credential)
            .expect("header should be valid");

        assert_eq!(header.header_name, AUTHORIZATION);
        assert_eq!(header.header_value, "Bearer agent-fed-token");
        assert_eq!(header.expires_on, Some(expires_on));
    }

    /// Scenario: an agent-fed snapshot contains a bearer token without a known expiry.
    /// Guarantees: the builder preserves the absent expiry and reports its provider name.
    #[test]
    fn supports_non_expiring_token_and_reports_provider_name() {
        let credential = snapshot(BearerToken::without_expiry("agent-fed-token".to_owned()));

        let header = AgentFedHttpClientStreamAuthProviderBuilder::build_auth_header(credential)
            .expect("header should be valid");

        assert_eq!(header.expires_on, None);
        assert_eq!(
            AgentFedHttpClientStreamAuthProviderBuilder::name().as_ref(),
            "AgentFedAuth"
        );
    }

    /// Scenario: an agent-fed snapshot contains a token invalid in an HTTP header value.
    /// Guarantees: the builder rejects the token instead of producing an invalid header.
    #[test]
    fn rejects_malformed_token() {
        let credential = snapshot(BearerToken::without_expiry("bad\ntoken".to_owned()));

        let error = AgentFedHttpClientStreamAuthProviderBuilder::build_auth_header(credential)
            .err()
            .expect("header should be rejected");

        assert!(error.starts_with("Malformed token:"));
    }
}
