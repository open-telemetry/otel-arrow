// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file API key authentication source and capability implementation.

use std::time::Instant;

use async_trait::async_trait;
use futures::StreamExt;
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::api_key_provider::ApiKeyStream;
use otel_arrow_dfe_engine::capability::auth::*;
use otel_arrow_dfe_engine::shared::capability::auth::api_key_provider::ApiKeyProvider as SharedApiKeyProvider;
use secrecy::{ExposeSecret, SecretString};
use tokio_stream::wrappers::WatchStream;

use crate::common::background_refresh::BackgroundProviderSource;
use crate::common::secret_file::{ReadSecretFileError, read_secret_file};
use crate::flat_file_api_key_auth::FlatFileApiKeyAuthExtension;
use crate::flat_file_api_key_auth::config::Config;
use crate::flat_file_api_key_auth::error::Error;

#[derive(Clone)]
pub struct FlatFileApiKeyAuth {
    config: Config,
    attributes: ApiKeyAttributes,
}

impl FlatFileApiKeyAuth {
    pub fn new(config: Config) -> Result<Self, String> {
        let attributes =
            ApiKeyAttributes::from_map(config.attributes.clone()).map_err(|e| e.to_string())?;

        Ok(Self { config, attributes })
    }
}

async fn read_api_key(config: &Config) -> Result<SecretString, Error> {
    if let Some(path) = &config.key_secret_file {
        let key = read_secret_file(path).await.map_err(|error| match error {
            ReadSecretFileError::Read(source) => Error::ReadCredentialFile {
                path: path.clone(),
                source,
            },
            ReadSecretFileError::InvalidUtf8 => Error::CredentialAcquisition {
                message: "`key_secret_file` does not contain valid UTF-8".to_string(),
            },
        })?;
        if key.expose_secret().is_empty() {
            return Err(Error::CredentialAcquisition {
                message: "API key cannot be empty".to_string(),
            });
        }
        return Ok(key);
    }

    config
        .key_secret
        .clone()
        .ok_or_else(|| Error::CredentialAcquisition {
            message: "no `key_secret` or `key_secret_file` configured".to_string(),
        })
}

#[async_trait]
impl BackgroundProviderSource<ApiKey> for FlatFileApiKeyAuth {
    type Error = Error;

    async fn fetch(&self) -> Result<ApiKey, Error> {
        let key = read_api_key(&self.config).await?;
        Ok(ApiKey::new(key).with_attributes(self.attributes.clone()))
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("flat_file_api_key_auth.credential_refresh_failed", error = %error);
    }

    fn expires_on(value: &ApiKey) -> Option<Instant> {
        value.get_expires_on()
    }
}

#[async_trait]
impl SharedApiKeyProvider for FlatFileApiKeyAuthExtension {
    async fn get_api_key(&self) -> Result<ApiKey, CapabilityError> {
        self.get_value().await
    }

    fn api_key_stream(&self) -> ApiKeyStream {
        let stream = WatchStream::new(self.subscribe()).filter_map(|value| async move { value });
        Box::pin(stream)
    }
}
