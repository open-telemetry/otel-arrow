// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Thin SASL adapter over protocol-neutral Key Vault secret acquisition.

use std::time::Instant;

use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::auth::SaslCredential;

use super::config::Config;
use super::error::{Error, FailureKind, Stage};
use crate::common::azure_key_vault::Source;
use crate::common::background_refresh::BackgroundProviderSource;

/// SASL-specific credential construction; identity and SDK logic are shared.
#[derive(Debug)]
pub struct Auth {
    source: Source,
}

impl Auth {
    /// Initializes identity/client, without secret network I/O.
    pub fn new(config: Config) -> Result<Self, Error> {
        Source::new(config).map(|source| Self { source })
    }

    #[cfg(test)]
    pub(crate) fn from_source(source: Source) -> Self {
        Self { source }
    }
}

// Send futures follow the existing Active+Shared provider capability contract
// and the Azure SDK's TokenCredential interface; no new cross-core task is added.
#[async_trait]
impl BackgroundProviderSource<SaslCredential> for Auth {
    type Error = Error;

    async fn fetch(&self) -> Result<SaslCredential, Error> {
        let pair = self.source.fetch_pair().await?;
        SaslCredential::new(pair.username, pair.password).map_err(|_| Error {
            stage: Stage::Acquisition,
            kind: FailureKind::InvalidValue,
        })
    }

    fn expires_on(_value: &SaslCredential) -> Option<Instant> {
        // Key Vault attributes are checked at startup, never a polling schedule.
        None
    }

    fn log_refresh_failure(&self, error: &Error) {
        error.log_failure();
    }
}
