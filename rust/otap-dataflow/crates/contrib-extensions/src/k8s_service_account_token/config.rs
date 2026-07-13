// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the Kubernetes Service Account Token extension.

use std::path::PathBuf;
use std::time::Duration;

use serde::Deserialize;

/// Default path of the projected service account token mounted into a pod.
pub(crate) const DEFAULT_TOKEN_FILE_PATH: &str =
    "/var/run/secrets/kubernetes.io/serviceaccount/token";

/// Default path of the projected service account token.
fn default_token_file_path() -> PathBuf {
    PathBuf::from(DEFAULT_TOKEN_FILE_PATH)
}

/// Default startup readiness timeout.
///
/// Reading a local file is fast, so this is deliberately much smaller than the
/// Azure extension's 30 s; it still leaves room for one retry (~10 s) if the
/// token file is not yet present at pod startup.
fn default_startup_timeout() -> Duration {
    Duration::from_secs(15)
}

/// Configuration for the Kubernetes Service Account Token extension.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Path to the mounted (projected) service account token file. Defaults to
    /// the standard in-cluster mount
    /// (`/var/run/secrets/kubernetes.io/serviceaccount/token`). Point this at a
    /// projected token with a specific audience when using one.
    #[serde(default = "default_token_file_path")]
    pub token_file_path: PathBuf,
    /// How long the engine holds data-path node startup waiting for this
    /// extension to publish its first token, before aborting pipeline startup.
    /// Accepts human-readable durations (e.g. `15s`, `1m`). Must be non-zero.
    #[serde(with = "humantime_serde", default = "default_startup_timeout")]
    pub startup_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            token_file_path: default_token_file_path(),
            startup_timeout: default_startup_timeout(),
        }
    }
}

impl Config {
    /// Validates the configuration beyond what deserialization checks.
    pub fn validate(&self) -> Result<(), String> {
        if self.token_file_path.as_os_str().is_empty() {
            return Err("`token_file_path` must not be empty".to_string());
        }
        if self.startup_timeout.is_zero() {
            return Err("`startup_timeout` must be greater than zero".to_string());
        }
        Ok(())
    }
}
