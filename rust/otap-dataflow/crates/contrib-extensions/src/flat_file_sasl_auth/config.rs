// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for file-backed SASL credentials.

use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

use crate::common::background_refresh::BackgroundProviderRefreshPolicy;

fn default_credentials_file_refresh() -> Duration {
    Duration::from_secs(60 * 60)
}

/// Configuration for the flat-file SASL extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// File containing the UTF-8 username.
    pub username_file: PathBuf,
    /// File containing the UTF-8 password.
    pub password_secret_file: PathBuf,
    /// How often to re-read both files. Defaults to one hour; accepts durations
    /// from ten seconds through 365 days.
    #[serde(with = "humantime_serde", default = "default_credentials_file_refresh")]
    pub credentials_file_refresh: Duration,
}

impl Config {
    /// Validates paths and polling bounds before startup.
    pub fn validate(&self) -> Result<(), String> {
        for (field, path) in [
            ("username_file", &self.username_file),
            ("password_secret_file", &self.password_secret_file),
        ] {
            if path.as_os_str().is_empty() {
                return Err(format!("`{field}` must be a non-empty path"));
            }
        }
        BackgroundProviderRefreshPolicy::periodic(self.credentials_file_refresh)
            .map(|_| ())
            .map_err(|error| format!("invalid `credentials_file_refresh`: {error}"))
    }
}
