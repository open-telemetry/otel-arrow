// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the flat file user pass extension.

use otel_arrow_dfe_config::error::Error as ConfigError;
use secrecy::{ExposeSecret, SecretString};

use super::config::Config;
use super::*;
use crate::flat_file_user_pass_auth::config::default_password_secret_file_refresh;

// -- Config tests -------------------------------------------

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

#[test]
fn config_defaults_apply() {
    let cfg = config_from_json(serde_json::json!({
        "username": "<test_username>",
        "password_secret": "<test_secret>",
        "password_secret_file": "<test_secret_path>"
    }))
    .expect("config is valid");
    assert_eq!(
        SecretString::expose_secret(&cfg.username),
        "<test_username>"
    );
    assert_eq!(
        cfg.password_secret
            .as_ref()
            .map(SecretString::expose_secret),
        Some("<test_secret>")
    );
    assert_eq!(
        cfg.password_secret_file.as_ref().and_then(|p| p.to_str()),
        Some("<test_secret_path>")
    );
    assert_eq!(
        cfg.password_secret_file_refresh,
        default_password_secret_file_refresh()
    );
}

#[test]
fn config_username_required() {
    assert!(
        config_from_json(serde_json::json!({
           "password_secret": "<test_secret>",
           "password_secret_file": "<test_secret_path>"
        }))
        .is_err()
    )
}

#[test]
fn config_secret_required() {
    assert!(
        config_from_json(serde_json::json!({
           "username": "<test_username>",
        }))
        .is_err()
    )
}

#[test]
fn config_password_secret_file_refresh_rejects_zero() {
    assert!(config_from_json(serde_json::json!({ "password_secret_file_refresh": "0s" })).is_err())
}

#[test]
fn factory_is_registered_with_capability() {
    assert_eq!(
        FLAT_FILE_USER_PASS_AUTH_EXTENSION.name,
        FLAT_FILE_USER_PASS_AUTH_URN
    );
    let capabilities = FLAT_FILE_USER_PASS_AUTH_EXTENSION
        .capabilities
        .as_ref()
        .expect("active extension advertises capabilities");
    assert!(
        capabilities.shared.contains(&"basic_auth_provider"),
        "BasicAuthProvider must be advertised as a shared capability"
    );
}
