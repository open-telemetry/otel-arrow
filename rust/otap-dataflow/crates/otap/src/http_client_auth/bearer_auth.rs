// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Consumer-side adapter over a bound `bearer_token_provider` capability.
//!
//! Centralizes everything an exporter needs to authenticate outgoing requests
//! with a bearer token, so the exporter itself stays auth-agnostic: it drives
//! [`BearerAuth::poll_refresh`] in its `select!` loop, asks
//! [`BearerAuth::is_ready`] before admitting data, and stamps
//! [`BearerAuth::header`] onto each request. The cached credential is an
//! `http::HeaderValue`, which both transports accept (tonic's `MetadataMap` is
//! backed by an `http::HeaderMap`), so core and contrib nodes on either
//! protocol can share this adapter.
//!
//! The division of labor mirrors the capability design: the **provider**
//! (extension) owns credential acquisition, background refresh, and startup
//! readiness gating; this **adapter** only subscribes to the provider's token
//! stream, caches the built `Authorization` header, and tracks whether that
//! cached token is still usable. The exporter is the "dumb caller".

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
