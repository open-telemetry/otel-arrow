// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the flat file user pass extension.

use std::path::PathBuf;
use std::time::Duration;

use otel_arrow_dfe_engine::capability::auth::BasicAuthCredential;
use secrecy::SecretString;
use serde::Deserialize;

use crate::common::background_refresh::BackgroundProviderRefreshPolicy;
use crate::flat_file_user_pass_auth::*;

/// Default password secret file refresh (~1 hr).
pub(crate) fn default_password_secret_file_refresh() -> Duration {
    DEFAULT_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL
}

/// Configuration for the HTTP Basic Client Auth extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Inline username. Required unless `username_file` is configured.
    ///
    /// Held as a [`SecretString`] so it is redacted from `Debug` output.
    #[serde(default)]
    pub username: Option<SecretString>,

    /// File containing the UTF-8 username; takes precedence over `username`.
    #[serde(default)]
    pub username_file: Option<PathBuf>,

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

    /// Refresh duration for either credential file (if specified). Accepts
    /// human-readable durations (e.g. `5m`, `1h`, `1d`).
    /// Default value: `1h`. Minimum value: `10s`. Maximum value: `365d`.
    #[serde(
        with = "humantime_serde",
        default = "default_password_secret_file_refresh"
    )]
    pub password_secret_file_refresh: Duration,
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    pub fn validate(&self) -> Result<(), String> {
        if self.username_file.is_none() {
            if let Some(username) = self.username.as_ref() {
                BasicAuthCredential::validate_username(username).map_err(|e| e.to_string())?;
            } else {
                return Err("either `username` or `username_file` must be set".to_string());
            }
        }

        for (field, path) in [
            ("username_file", self.username_file.as_ref()),
            ("password_secret_file", self.password_secret_file.as_ref()),
        ] {
            if path.is_some_and(|path| path.as_os_str().is_empty()) {
                return Err(format!("`{field}` must be a non-empty path"));
            }
        }

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

        if self.has_file_source() {
            BackgroundProviderRefreshPolicy::periodic(self.password_secret_file_refresh)
                .map(|_| ())
                .map_err(|error| format!("invalid `password_secret_file_refresh`: {error}"))?;
        }

        Ok(())
    }

    /// Whether either credential value is acquired from a file.
    pub(crate) fn has_file_source(&self) -> bool {
        self.username_file.is_some() || self.password_secret_file.is_some()
    }
}
