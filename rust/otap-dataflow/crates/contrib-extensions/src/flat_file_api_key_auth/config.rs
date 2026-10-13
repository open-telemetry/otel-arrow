// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the flat file API key authentication extension.

use std::path::PathBuf;
use std::time::Duration;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;
use serde_json::{Map, Value};

use crate::common::background_refresh::BackgroundProviderRefreshPolicy;
use crate::flat_file_api_key_auth::*;

/// Default API key secret file refresh (~1 hr).
pub(crate) fn default_key_secret_file_refresh() -> Duration {
    DEFAULT_API_KEY_REFRESH_INTERVAL
}

/// Configuration for the flat file API key authentication extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// API key supplied inline.
    #[serde(default)]
    pub key_secret: Option<SecretString>,

    /// Path to a file holding the API key. Takes precedence over `key_secret`.
    #[serde(default)]
    pub key_secret_file: Option<PathBuf>,

    /// Refresh interval for the API key file.
    #[serde(with = "humantime_serde", default = "default_key_secret_file_refresh")]
    pub key_secret_file_refresh: Duration,

    /// Metadata made available to API key consumers.
    #[serde(default)]
    pub attributes: Map<String, Value>,
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    pub fn validate(&self) -> Result<(), String> {
        if self.key_secret_file.is_none() {
            match self.key_secret.as_ref() {
                Some(key) if !key.expose_secret().is_empty() => {}
                Some(_) => return Err("`key_secret` cannot be empty".to_string()),
                None => {
                    return Err("either `key_secret` or `key_secret_file` must be set".to_string());
                }
            }
        }

        if self.key_secret_file.is_some() {
            BackgroundProviderRefreshPolicy::periodic(self.key_secret_file_refresh)
                .map(|_| ())
                .map_err(|error| format!("invalid `key_secret_file_refresh`: {error}"))?;
        }

        Ok(())
    }
}
