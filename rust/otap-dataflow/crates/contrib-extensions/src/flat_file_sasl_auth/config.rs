// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for flat file SASL authentication.

use std::path::PathBuf;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

use crate::common::background_refresh::BackgroundProviderRefreshPolicy;

use super::DEFAULT_SASL_CREDENTIAL_REFRESH_INTERVAL;

pub(crate) fn default_password_secret_file_refresh() -> Duration {
    DEFAULT_SASL_CREDENTIAL_REFRESH_INTERVAL
}

fn default_startup_timeout() -> Duration {
    Duration::from_secs(30)
}

/// Inline or file-backed username and the file containing its SASL password.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// SASL username supplied inline, redacted in debug output.
    /// Required unless `username_file` is set.
    #[serde(default)]
    pub username: Option<SecretString>,

    /// Path to a UTF-8 username file; takes precedence over `username`.
    #[serde(default)]
    pub username_file: Option<PathBuf>,

    /// Path to a UTF-8 password file read during acquisition and refresh.
    pub password_secret_file: PathBuf,

    /// Refresh interval for both credential files. Defaults to `1h`; accepts `10s` through `365d`.
    #[serde(
        with = "humantime_serde",
        default = "default_password_secret_file_refresh"
    )]
    pub password_secret_file_refresh: Duration,

    /// Maximum wait for initial acquisition before pipeline startup fails. Defaults to `30s`.
    #[serde(with = "humantime_serde", default = "default_startup_timeout")]
    pub startup_timeout: Duration,
}

impl Config {
    /// Rejects empty required fields without restricting SASL username syntax.
    pub fn validate(&self) -> Result<(), String> {
        if let Some(path) = &self.username_file {
            if path.as_os_str().is_empty() {
                return Err("`username_file` cannot be empty".to_string());
            }
        } else {
            match &self.username {
                Some(username) if !username.expose_secret().is_empty() => {}
                Some(_) => return Err("`username` cannot be empty".to_string()),
                None => return Err("either `username` or `username_file` must be set".to_string()),
            }
        }
        if self.password_secret_file.as_os_str().is_empty() {
            return Err("`password_secret_file` cannot be empty".to_string());
        }
        if self.startup_timeout.is_zero() {
            return Err("`startup_timeout` must be greater than zero".to_string());
        }
        BackgroundProviderRefreshPolicy::periodic(self.password_secret_file_refresh)
            .map(|_| ())
            .map_err(|error| format!("invalid `password_secret_file_refresh`: {error}"))
    }
}
