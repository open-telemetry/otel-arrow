// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the flat file user pass extension.

use std::path::PathBuf;
use std::time::Duration;

use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use secrecy::SecretString;
use serde::Deserialize;

use crate::flat_file_user_pass_auth::*;

/// Default password secret file refresh (~1 hr).
pub(crate) fn default_password_secret_file_refresh() -> Duration {
    DEFAULT_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL
}

/// Configuration for the HTTP Basic Client Auth extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Username. Must be non-empty.
    ///
    /// Held as a [`SecretString`] so it is redacted from `Debug` output.
    pub username: SecretString,

    /// Password secret.
    ///
    /// Held as a [`SecretString`] so it is redacted from `Debug` output.
    /// Note that the raw pipeline config retains the cleartext;
    /// prefer `password_secret_file`.
    #[serde(default)]
    pub password_secret: Option<SecretString>,

    /// Path to a file holding the password secret. Re-read at
    /// `password_secret_file_refresh` interval and takes precedence over
    /// `password_secret`.
    #[serde(default)]
    pub password_secret_file: Option<PathBuf>,

    /// Refresh duration for the password secret file (if specified). Accepts
    /// human-readable durations (e.g. `5m`, `1h`, `1d`). Must be non-zero.
    /// Default value: `1h`. Mimimum value: `5m`.
    #[serde(
        with = "humantime_serde",
        default = "default_password_secret_file_refresh"
    )]
    pub password_secret_file_refresh: Duration,
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    pub fn validate(&self) -> Result<(), String> {
        BasicAuthCredential::validate_username(&self.username).map_err(|e| e.to_string())?;

        if self.password_secret_file.is_none() {
            if let Some(password_secret) = self.password_secret.as_ref() {
                BasicAuthCredential::validate_password(password_secret)
                    .map_err(|e| e.to_string())?;
            } else {
                return Err(
                    "either `password_secret` or `password_secret_file` must be set".to_string(),
                );
            }
        }

        if self.password_secret_file_refresh < MINIMUM_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL {
            return Err(
                "`password_secret_file_refresh` must be greater than or equal to `5m`".to_string(),
            );
        }

        Ok(())
    }
}
