// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file SASL credential source and capability implementation.

use std::time::Instant;

use async_trait::async_trait;
use futures::StreamExt;
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::SaslCredential;
use otel_arrow_dfe_engine::capability::auth::sasl_credential_provider::SaslCredentialStream;
use otel_arrow_dfe_engine::shared::capability::auth::sasl_credential_provider::SaslCredentialProvider as SharedSaslCredentialProvider;
use tokio_stream::wrappers::WatchStream;

use crate::common::background_refresh::BackgroundProviderSource;
use crate::common::secret_file::{ReadSecretFileError, read_secret_file};

use super::FlatFileSaslAuthExtension;
use super::config::Config;
use super::error::Error;

/// Acquires SASL credentials from an inline username and a local password file.
#[derive(Clone)]
pub struct FlatFileSaslAuth {
    config: Config,
}

impl FlatFileSaslAuth {
    /// Creates the source; acquisition and retry scheduling belong to the shared provider.
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

#[async_trait]
impl BackgroundProviderSource<SaslCredential> for FlatFileSaslAuth {
    type Error = Error;

    async fn fetch(&self) -> Result<SaslCredential, Error> {
        let path = &self.config.password_secret_file;
        let password = read_secret_file(path).await.map_err(|error| match error {
            ReadSecretFileError::Read(source) => Error::ReadCredentialFile {
                path: path.clone(),
                source,
            },
            ReadSecretFileError::InvalidUtf8 => Error::CredentialAcquisition {
                path: path.clone(),
                message: "file does not contain valid UTF-8".to_string(),
            },
        })?;
        SaslCredential::new(self.config.username.clone(), password).map_err(|error| {
            Error::CredentialAcquisition {
                path: path.clone(),
                message: error.to_string(),
            }
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
impl SharedSaslCredentialProvider for FlatFileSaslAuthExtension {
    async fn get_credential(&self) -> Result<SaslCredential, CapabilityError> {
        self.get_value().await
    }

    fn credential_stream(&self) -> SaslCredentialStream {
        let stream = WatchStream::new(self.subscribe()).filter_map(|value| async move { value });
        Box::pin(stream)
    }
}
