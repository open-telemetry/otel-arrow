// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file user pass extension.

use std::path::PathBuf;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::StreamExt;
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BasicAuthCredentialStream;
use otel_arrow_dfe_engine::shared::capability::auth::basic_auth_provider::BasicAuthProvider as SharedBasicAuthProvider;
use otel_arrow_dfe_otap::tls_utils::read_file_with_limit_async;
use secrecy::{ExposeSecret, SecretString};
use tokio_stream::wrappers::WatchStream;

use crate::common::background_refresh::BackgroundProviderSource;
use crate::flat_file_user_pass_auth::FlatFileUserPassAuthExtension;
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
    file_refresh: Duration,
    inline: Option<&str>,
    field: &str,
) -> Result<(String, Option<Duration>), Error> {
    if let Some(path) = file {
        let contents =
            read_file_with_limit_async(path)
                .await
                .map_err(|source| Error::ReadCredentialFile {
                    path: path.clone(),
                    source,
                })?;
        let contents = String::from_utf8(contents).map_err(|_| Error::CredentialAcquisition {
            message: format!("`{field}_file` does not contain valid UTF-8"),
        })?;
        return Ok((
            contents.trim_end_matches(&['\r', '\n'][..]).to_string(),
            Some(file_refresh),
        ));
    }
    if let Some(value) = inline {
        return Ok((value.to_owned(), None));
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
        let (password, expiry) = read_credential(
            self.config.password_secret_file.as_ref(),
            self.config.password_secret_file_refresh,
            self.config
                .password_secret
                .as_ref()
                .map(SecretString::expose_secret),
            "password_secret",
        )
        .await?;

        let mut credential =
            BasicAuthCredential::new(SecretString::expose_secret(&self.config.username), password)
                .map_err(|e| Error::CredentialAcquisition {
                    message: e.to_string(),
                })?;

        if let Some(expiry) = expiry {
            credential = credential.with_expiry(
                Instant::now() + expiry.max(super::BASIC_AUTH_CREDENTIAL_EXPIRY_BUFFER_SECS * 2),
            );
        }

        Ok(credential)
    }

    fn log_refresh_failure(&self, error: &Error) {
        otel_warn!("flat_file_user_pass_auth.credential_refresh_failed", error = %error);
    }

    fn expires_on(value: &BasicAuthCredential) -> Option<Instant> {
        value.expires_on()
    }
}

#[async_trait]
impl SharedBasicAuthProvider for FlatFileUserPassAuthExtension {
    async fn get_credential(&self) -> Result<BasicAuthCredential, CapabilityError> {
        self.get_value().await
    }

    fn credential_stream(&self) -> BasicAuthCredentialStream {
        let rx = self.subscribe();
        // Yield the current cached value immediately, then each refresh. The
        // initial `None` (and any future `None`) is filtered out. The stream
        // item is a plain `BasicAuthCredential`: a refresh failure does not terminate
        // the subscription, it simply does not emit until the next success.
        let stream = WatchStream::new(rx).filter_map(|opt| async move { opt });
        Box::pin(stream)
    }
}
