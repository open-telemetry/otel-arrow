// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file SASL credential source and capability implementation.

use std::path::Path;
use std::time::Instant;

use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::auth::{SaslCredential, SaslCredentialError};
use secrecy::SecretString;

use crate::common::background_refresh::BackgroundProviderSource;
use crate::common::secret_file::{ReadSecretFileError, read_secret_file};

use super::config::Config;
use super::error::Error;

/// Acquires SASL credentials from an inline or file-backed username and a local password file.
#[derive(Clone)]
pub struct FlatFileSaslAuth {
    config: Config,
}

async fn read_credential_file(path: &Path, field: &'static str) -> Result<SecretString, Error> {
    read_secret_file(path).await.map_err(|error| match error {
        ReadSecretFileError::Read(source) => Error::ReadCredentialFile {
            field,
            path: path.to_owned(),
            source,
        },
        ReadSecretFileError::InvalidUtf8 => Error::CredentialAcquisition {
            field,
            path: Some(path.to_owned()),
            message: "file does not contain valid UTF-8".to_string(),
        },
    })
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
        let username_field = if self.config.username_file.is_some() {
            "username_file"
        } else {
            "username"
        };
        let username = if let Some(path) = &self.config.username_file {
            read_credential_file(path, username_field).await?
        } else {
            self.config
                .username
                .clone()
                .ok_or_else(|| Error::CredentialAcquisition {
                    field: username_field,
                    path: None,
                    message: "either `username` or `username_file` must be set".to_string(),
                })?
        };
        let password =
            read_credential_file(&self.config.password_secret_file, "password_secret_file").await?;
        SaslCredential::new(username, password).map_err(|error| {
            let (field, path) = match &error {
                SaslCredentialError::InvalidUsername { .. } => {
                    (username_field, self.config.username_file.clone())
                }
                SaslCredentialError::InvalidPassword { .. } => (
                    "password_secret_file",
                    Some(self.config.password_secret_file.clone()),
                ),
            };
            Error::CredentialAcquisition {
                field,
                path,
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
