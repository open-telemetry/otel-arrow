// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the flat file user pass extension.

use std::path::PathBuf;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

/// Default password secret file refresh (~1 hr).
fn default_password_secret_file_refresh() -> Duration {
    Duration::from_secs(60 * 60)
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
    /// human-readable durations (e.g. `5m`, `30s`). Must be non-zero.
    #[serde(
        with = "humantime_serde",
        default = "default_password_secret_file_refresh"
    )]
    pub password_secret_file_refresh: Duration,
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    ///
    /// Rejects a secret.
    pub fn validate(&self) -> Result<(), String> {
        if self.username.expose_secret().is_empty() {
            return Err("`username` must be specified".to_string());
        }

        let secret_fields_set = self
            .password_secret
            .as_ref()
            .is_some_and(|s| !s.expose_secret().is_empty())
            || self.password_secret_file.is_some();

        if !secret_fields_set {
            return Err(
                "either `password_secret` or `password_secret_file` must be set".to_string(),
            );
        }

        if self.password_secret_file_refresh.is_zero() {
            return Err("`password_secret_file_refresh` must be greater than zero".to_string());
        }

        Ok(())
    }
}
