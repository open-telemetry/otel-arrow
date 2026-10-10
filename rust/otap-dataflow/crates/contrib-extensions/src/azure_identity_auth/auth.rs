// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Azure credential construction and token acquisition.

use std::sync::Arc;
use std::time::Instant;

use async_trait::async_trait;
use azure_core::credentials::{TokenCredential, TokenRequestOptions};
use otel_arrow_dfe_engine::capability::auth::BearerToken;

use super::config::Config;
use super::error::Error;
use crate::common::azure_identity::{IdentityOptions, create_credential};
use crate::common::background_refresh::BackgroundProviderSource;

/// Wraps an Azure credential plus the scope it acquires tokens for.
#[derive(Clone)]
pub struct Auth {
    credential: Arc<dyn TokenCredential>,
    scope: String,
}

impl Auth {
    /// Builds an `Auth` from the extension configuration.
    pub fn new(config: &Config) -> Result<Self, Error> {
        let credential = create_credential(IdentityOptions::from(config)).map_err(|source| {
            Error::CreateCredential {
                method: config.method,
                source,
            }
        })?;
        Ok(Self {
            credential,
            scope: config.scope.clone(),
        })
    }

    /// Builds an `Auth` from an already-constructed credential. Test-only.
    #[cfg(test)]
    pub(crate) fn from_credential(credential: Arc<dyn TokenCredential>, scope: String) -> Self {
        Self { credential, scope }
    }
}

#[async_trait]
impl BackgroundProviderSource<BearerToken> for Auth {
    type Error = Error;

    /// Acquires a single token (no retries) and converts it into a
    /// [`BearerToken`].
    async fn fetch(&self) -> Result<BearerToken, Error> {
        let access = self
            .credential
            .get_token(&[&self.scope], Some(TokenRequestOptions::default()))
            .await
            .map_err(|source| Error::TokenAcquisition { source })?;

        // Let the capability crate centralize the absolute-expiry -> monotonic
        // `Instant` conversion so every provider handles it the same way.
        Ok(BearerToken::from_absolute_expiry(
            access.token.secret().to_owned(),
            access.expires_on.into(),
        ))
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("azure_identity_auth.token_refresh_failed", error = %error);
    }

    fn expires_on(value: &BearerToken) -> Option<Instant> {
        value.expires_on()
    }
}
