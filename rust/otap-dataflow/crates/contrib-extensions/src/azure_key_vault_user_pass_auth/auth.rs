// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Basic Auth validation over the shared Azure identity and Key Vault source.

use std::time::Instant;

use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::auth::{BasicAuthCredential, BasicAuthCredentialError};

use super::config::Config;
use super::error::{Error, FailureKind, Stage};
use crate::common::azure_key_vault::Source;
use crate::common::background_refresh::BackgroundProviderSource;

/// Acquires both secrets, then validates the complete Basic Auth credential.
#[derive(Debug)]
pub struct Auth {
    source: Source,
}

impl Auth {
    /// Constructs identity/client without retrieving secrets.
    pub fn new(config: Config) -> Result<Self, Error> {
        Source::new(config).map(|source| Self { source })
    }

    #[cfg(test)]
    pub(crate) fn from_source(source: Source) -> Self {
        Self { source }
    }
}

// Send follows the existing Active+Shared provider and SDK contracts. The
// engine owns the task; no additional cross-core task or secret cache is added.
#[async_trait]
impl BackgroundProviderSource<BasicAuthCredential> for Auth {
    type Error = Error;

    async fn fetch(&self) -> Result<BasicAuthCredential, Error> {
        let pair = self.source.fetch_pair().await?;
        BasicAuthCredential::new(pair.username, pair.password).map_err(|error| Error {
            stage: match error {
                BasicAuthCredentialError::InvalidUsername { .. } => Stage::Username,
                BasicAuthCredentialError::InvalidPassword { .. } => Stage::Password,
            },
            kind: FailureKind::InvalidValue,
        })
    }

    fn expires_on(_value: &BasicAuthCredential) -> Option<Instant> {
        None
    }

    fn log_refresh_failure(&self, error: &Error) {
        error.log_failure();
    }
}
