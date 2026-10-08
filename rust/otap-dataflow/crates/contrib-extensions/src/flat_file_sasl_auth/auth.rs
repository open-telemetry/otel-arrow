// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Acquisition and capability implementation for file-backed SASL credentials.

use std::time::Instant;

use async_trait::async_trait;
use futures::StreamExt;
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::SaslCredential;
use otel_arrow_dfe_engine::capability::auth::sasl_credential_provider::SaslCredentialStream;
use otel_arrow_dfe_engine::shared::capability::auth::sasl_credential_provider::SaslCredentialProvider;
use tokio_stream::wrappers::WatchStream;

use super::FlatFileSaslAuthExtension;
use super::config::Config;
use super::error::Error;
use crate::common::background_refresh::BackgroundProviderSource;
use crate::common::user_pass_file::read_user_pass;

pub(super) struct FlatFileSaslAuth {
    config: Config,
}

impl FlatFileSaslAuth {
    pub(super) fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl BackgroundProviderSource<SaslCredential> for FlatFileSaslAuth {
    type Error = Error;

    async fn fetch(&self) -> Result<SaslCredential, Error> {
        let (username, password) = read_user_pass(
            Some(&self.config.username_file),
            None,
            Some(&self.config.password_secret_file),
            None,
        )
        .await?;
        SaslCredential::new(username, password).map_err(|error| Error::CredentialAcquisition {
            message: error.to_string(),
        })
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("flat_file_sasl_auth.credential_refresh_failed", error = %error);
    }

    fn expires_on(value: &SaslCredential) -> Option<Instant> {
        value.expires_on()
    }
}

#[async_trait]
impl SaslCredentialProvider for FlatFileSaslAuthExtension {
    async fn get_credential(&self) -> Result<SaslCredential, CapabilityError> {
        self.get_value().await
    }

    fn credential_stream(&self) -> SaslCredentialStream {
        Box::pin(WatchStream::new(self.subscribe()).filter_map(|value| async move { value }))
    }
}
