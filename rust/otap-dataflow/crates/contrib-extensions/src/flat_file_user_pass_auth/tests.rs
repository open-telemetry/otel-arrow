// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the flat file user pass extension.

use std::io::Write;

use futures::StreamExt;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::shared::capability::auth::basic_auth_provider::BasicAuthProvider;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use secrecy::{ExposeSecret, SecretString};
use tempfile::NamedTempFile;

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
fn config_username_required_and_valid() {
    assert!(
        config_from_json(serde_json::json!({
           "username": ""
        }))
        .is_err()
    );

    assert!(
        config_from_json(serde_json::json!({
           "username": "invalid:colon"
        }))
        .is_err()
    );

    assert!(
        config_from_json(serde_json::json!({
           "username": "invalid\tcontrol"
        }))
        .is_err()
    );

    assert!(
        config_from_json(serde_json::json!({
           "password_secret": "<test_secret>",
           "password_secret_file": "<test_secret_path>"
        }))
        .is_err()
    );
}

#[test]
fn config_secret_required_and_valid() {
    assert!(
        config_from_json(serde_json::json!({
           "username": "<test_username>",
        }))
        .is_err()
    );

    assert!(
        config_from_json(serde_json::json!({
           "username": "<test_username>",
           "password_secret": "",
        }))
        .is_err()
    );

    assert!(
        config_from_json(serde_json::json!({
           "username": "<test_username>",
           "password_secret": "password\tcontrol",
        }))
        .is_err()
    );
}

#[test]
fn config_password_secret_file_refresh_rejects_invalid() {
    assert!(
        config_from_json(serde_json::json!({
        "username": "test",
        "password_secret": "<test_secret>",
        "password_secret_file": "<test_secret_path>",
        "password_secret_file_refresh": "4m" }))
        .is_err()
    )
}

// -- Factory tests ------------------------------------------

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

/// Invokes the factory's `create` hook with `config` against a throwaway
/// extension context, mirroring how the engine wires the extension.
fn create_bundle(config: serde_json::Value) -> Result<ExtensionBundle, ConfigError> {
    let (ext_ctx, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name: otel_arrow_dfe_config::ExtensionId = "flat-file-user-pass-auth".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        FLAT_FILE_USER_PASS_AUTH_URN.into(),
        config,
    ));
    let extension_config = ExtensionConfig::new(name.clone());
    create(&ext_ctx, name, user_config, &extension_config)
}

// Scenario: The factory's `create` hook runs against a valid config.
// Guarantees: Wiring succeeds and yields a shared, active extension bundle usable by the engine.
#[test]
fn create_builds_a_shared_active_bundle() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let bundle = create_bundle(serde_json::json!({
            "username": "test_name",
            "password_secret": "test_pass" }))
    .expect("a valid config wires successfully");
    assert!(
        bundle.local().is_none(),
        "the auth extension has no local variant"
    );
    let shared = bundle.shared().expect("a shared variant is produced");
    assert_eq!(shared.variant(), ExtensionVariant::Shared);
    assert!(
        !shared.is_passive(),
        "the extension must be active so its refresh loop runs"
    );
}

// Scenario: The factory's `create` hook runs against a config that fails validation.
// Guarantees: Wiring fails fast with InvalidUserConfig instead of building a broken extension.
#[test]
fn create_rejects_an_invalid_config() {
    let Err(err) = create_bundle(serde_json::json!({})) else {
        panic!("expected error");
    };
    assert!(
        matches!(err, ConfigError::InvalidUserConfig { .. }),
        "expected InvalidUserConfig, got {err:?}"
    );
}

// -- Token acquisition / cache tests ---------------------------

fn make_extension() -> FlatFileUserPassAuthExtension {
    make_extension_with_config(Config {
        username: "test_user".into(),
        password_secret: Some("test_pass".into()),
        password_secret_file: None,
        password_secret_file_refresh: Duration::from_secs(60),
    })
}

fn make_extension_with_config(config: Config) -> FlatFileUserPassAuthExtension {
    let (tx, _rx) = watch::channel(None);
    FlatFileUserPassAuthExtension::new(
        "test-ext",
        FlatFileUserPassAuth::new(config),
        BackgroundProviderRefreshPolicy::new(
            BASIC_AUTH_CREDENTIAL_USABLE_MARGIN,
            NON_EXPIRING_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL,
            BASIC_AUTH_CREDENTIAL_EXPIRY_BUFFER_SECS,
        )
        .expect("valid refresh_policy"),
        tx,
        make_tracker(),
    )
}

fn make_tracker() -> BackgroundProviderMetricsTracker<FlatFileUserPassAuthMetrics> {
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<FlatFileUserPassAuthMetrics>(EmptyAttributes());
    BackgroundProviderMetricsTracker::new(metric_set)
}

#[tokio::test]
async fn get_credential() {
    let ext = make_extension();

    let credential = ext.get_credential().await.expect("first acquisition");
    assert_eq!(credential.expose_username(), "test_user");
    assert_eq!(credential.expose_password(), "test_pass");
    assert!(credential.expires_on().is_none());
}

#[tokio::test]
async fn get_credential_file_success() {
    let mut named_file = NamedTempFile::new().expect("file created");

    let content = "test_pass  \r\n";
    named_file
        .write_all(content.as_bytes())
        .expect("content written");

    let ext = make_extension_with_config(Config {
        username: "test_user".into(),
        password_secret: None,
        password_secret_file: Some(named_file.path().into()),
        password_secret_file_refresh: Duration::from_secs(10),
    });

    let credential = ext.get_credential().await.expect("first acquisition");
    assert_eq!(credential.expose_username(), "test_user");
    assert_eq!(credential.expose_password(), "test_pass  ");
    assert!(credential.expires_on().is_some());
}

#[tokio::test]
async fn get_credential_file_failure() {
    let ext = make_extension_with_config(Config {
        username: "test_user".into(),
        password_secret: None,
        password_secret_file: Some("/ext/invalid_file_secret".into()),
        password_secret_file_refresh: Duration::from_secs(10),
    });

    assert!(ext.get_credential().await.is_err())
}

#[tokio::test]
async fn credential_stream() {
    let ext = make_extension();

    let credential_first = ext.get_value().await.expect("first acquisition");
    assert_eq!(credential_first.expose_username(), "test_user");
    assert_eq!(credential_first.expose_password(), "test_pass");

    let credential_second = ext
        .credential_stream()
        .next()
        .await
        .expect("second acquisition");
    assert_eq!(credential_second.expose_username(), "test_user");
    assert_eq!(credential_second.expose_password(), "test_pass");
}
