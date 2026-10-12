// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// HTTP Client authentication provider.
mod http_client_auth_provider;

mod agent_fed_auth;

mod api_key_auth;

mod basic_auth;

mod bearer_auth;

pub use http_client_auth_provider::*;
