// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Flat file user pass extension.

use std::time::Instant;

use async_trait::async_trait;
use futures::StreamExt;
use otel_arrow_dfe_engine::capability::CapabilityError;
use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use otel_arrow_dfe_engine::capability::auth::basic_auth_provider::BasicAuthCredentialStream;
use otel_arrow_dfe_engine::shared::capability::auth::basic_auth_provider::BasicAuthProvider as SharedBasicAuthProvider;
use tokio_stream::wrappers::WatchStream;

use crate::common::background_refresh::BackgroundProviderSource;
use crate::common::user_pass_file::read_user_pass;
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

#[async_trait]
impl BackgroundProviderSource<BasicAuthCredential> for FlatFileUserPassAuth {
    type Error = Error;

    /// Fetch a single credential.
    async fn fetch(&self) -> Result<BasicAuthCredential, Error> {
        let (username, password) = read_user_pass(
            self.config.username_file.as_deref(),
            self.config.username.as_ref(),
            self.config.password_secret_file.as_deref(),
            self.config.password_secret.as_ref(),
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
