// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file user pass extension.

use std::path::PathBuf;
use std::time::Instant;

use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use secrecy::SecretString;

use crate::common::background_refresh::BackgroundProviderSource;
use crate::common::secret_file::{ReadSecretFileError, read_secret_file};
use crate::flat_file_user_pass_auth::config::Config;
use crate::flat_file_user_pass_auth::error::Error;

#[derive(Clone)]
pub struct FlatFileUserPassAuth {
    /// The configuration.
    config: Config,
}

impl FlatFileUserPassAuth {
    pub fn new(config: Config) -> Self {
        Self { config }
    }
}

/// Reads a credential value, preferring the file form (re-read on each call so
/// the credential can rotate without a restart) over the inline value.
///
/// File reads go through the collector's shared size-limited reader: this runs
/// on the per-acquisition path, so an oversized or hostile path would otherwise
/// be re-read into memory on every refresh.
async fn read_credential(
    file: Option<&PathBuf>,
    inline: Option<&SecretString>,
    field: &str,
) -> Result<SecretString, Error> {
    if let Some(path) = file {
        return read_secret_file(path).await.map_err(|error| match error {
            ReadSecretFileError::Read(source) => Error::ReadCredentialFile {
                path: path.clone(),
                source,
            },
            ReadSecretFileError::InvalidUtf8 => Error::CredentialAcquisition {
                message: format!("`{field}_file` does not contain valid UTF-8"),
            },
        });
    }
    if let Some(value) = inline {
        return Ok(value.clone());
    }
    Err(Error::CredentialAcquisition {
        message: format!("no `{field}` or `{field}_file` configured"),
    })
}

#[async_trait]
impl BackgroundProviderSource<BasicAuthCredential> for FlatFileUserPassAuth {
    type Error = Error;

    /// Fetch a single credential.
    async fn fetch(&self) -> Result<BasicAuthCredential, Error> {
        let username = read_credential(
            self.config.username_file.as_ref(),
            self.config.username.as_ref(),
            "username",
        )
        .await?;
        let password = read_credential(
            self.config.password_secret_file.as_ref(),
            self.config.password_secret.as_ref(),
            "password_secret",
        )
        .await?;

        BasicAuthCredential::new(username, password).map_err(|e| Error::CredentialAcquisition {
            message: e.to_string(),
        })
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("flat_file_user_pass_auth.credential_refresh_failed", error = %error);
    }

    fn expires_on(value: &BasicAuthCredential) -> Option<Instant> {
        value.expires_on()
    }
}
