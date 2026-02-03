// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Token provider extension trait.

use super::ExtensionTrait;

/// Represents a bearer token with its expiration time.
#[derive(Debug, Clone)]
pub struct BearerToken {
    /// The token string.
    pub token: String,

    /// The expiration time as a UNIX timestamp (seconds since epoch).
    pub expires_on: i64,
}

/// A trait for components that can provide authentication tokens.
///
/// Extensions implementing this trait can be looked up by other components
/// (e.g., exporters) to obtain tokens for authentication.
pub trait TokenProvider: ExtensionTrait {
    /// Returns an authentication token.
    async fn get_token(&self) -> BearerToken;
}
