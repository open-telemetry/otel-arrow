// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for startup-only flat file SASL authentication.

use std::path::PathBuf;

use secrecy::{ExposeSecret, SecretString};
use serde::Deserialize;

/// Inline username and the file containing its SASL password.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// SASL username supplied inline, redacted in debug output.
    pub username: SecretString,

    /// Path to a UTF-8 password file read once at extension construction.
    pub password_secret_file: PathBuf,
}

impl Config {
    /// Rejects empty required fields without restricting SASL username syntax.
    pub fn validate(&self) -> Result<(), String> {
        if self.username.expose_secret().is_empty() {
            return Err("`username` cannot be empty".to_string());
        }
        if self.password_secret_file.as_os_str().is_empty() {
            return Err("`password_secret_file` cannot be empty".to_string());
        }
        Ok(())
    }
}
