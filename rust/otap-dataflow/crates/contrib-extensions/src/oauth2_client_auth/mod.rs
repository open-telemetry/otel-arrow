// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OAuth 2.0 Client Auth extension.
//!
//! Acquires and refreshes OAuth 2.0 access tokens using the client-credentials
//! grant and exposes them to data-path nodes through the `BearerTokenProvider`
//! capability. See `docs/oauth2-client-auth-extension.md` for the design.

pub mod config;
pub mod error;

/// URN under which this extension is registered.
pub const OAUTH2_CLIENT_AUTH_URN: &str = "urn:otel:extension:oauth2_client_auth";
