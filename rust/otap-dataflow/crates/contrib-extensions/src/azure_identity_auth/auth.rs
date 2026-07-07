// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure credential construction and token acquisition.

use otap_df_engine::capability::BearerToken;

use super::config::Config;
use super::error::Error;

/// Acquires Azure access tokens for a configured scope.
///
/// This is a placeholder: it returns a fixed, non-expiring token so the
/// extension skeleton can run end-to-end. The real Azure credential
/// construction and token acquisition are added in a later change.
pub struct Auth;

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(_config: &Config) -> Result<Self, Error> {
        Ok(Self)
    }

    /// Acquires a single token (no retries).
    pub async fn get_token(&self) -> Result<BearerToken, Error> {
        Ok(BearerToken::new("stub-token".to_owned(), None))
    }
}
