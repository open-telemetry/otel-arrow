// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the flat file API key authentication extension.

use std::io::Write;

use futures::{FutureExt, StreamExt};
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::shared::capability::auth::api_key_provider::ApiKeyProvider;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension as SharedExtension};
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use secrecy::{ExposeSecret, SecretString};
use tempfile::NamedTempFile;

use super::config::{Config, default_key_secret_file_refresh};
use super::*;
use crate::common::background_refresh::BackgroundProviderSource;

fn valid_attributes() -> serde_json::Map<String, serde_json::Value> {
    serde_json::from_value(serde_json::json!({
        "http.header_name": "x-api-key",
        "http.header_scheme": "ApiKey"
    }))
    .expect("attributes are an object")
}

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

/// Scenario: A valid inline config omits the optional file refresh interval.
/// Guarantees: Parsing preserves the key and attributes and applies the default interval.
#[test]
fn config_defaults_apply() {
    let config = config_from_json(serde_json::json!({
        "key_secret": "test-key",
        "attributes": {
            "http.header_name": "x-api-key",
            "http.header_scheme": "ApiKey"
        }
    }))
    .expect("config is valid");

    assert_eq!(
        config.key_secret.as_ref().map(SecretString::expose_secret),
        Some("test-key")
    );
    assert_eq!(
        config.attributes.get("http.header_name"),
        Some(&serde_json::json!("x-api-key"))
    );
    assert_eq!(
        config.key_secret_file_refresh,
        default_key_secret_file_refresh()
    );
}

/// Scenario: Config parsing receives no key source or an empty inline key.
/// Guarantees: Both invalid secret configurations are rejected before startup.
#[test]
fn config_key_source_is_required_and_non_empty() {
    for value in [
        serde_json::json!({"attributes": {"http.header_name": "x-api-key"}}),
        serde_json::json!({
            "key_secret": "",
            "attributes": {"http.header_name": "x-api-key"}
        }),
    ] {
        assert!(config_from_json(value).is_err());
    }
}

/// Scenario: Config parsing receives missing or incorrectly typed HTTP attributes.
/// Guarantees: Header metadata required by API key consumers is rejected unless well formed.
#[test]
fn config_http_attributes_are_validated() {
    for attributes in [
        serde_json::json!({}),
        serde_json::json!({"http.header_name": ""}),
        serde_json::json!({"http.header_name": 42}),
        serde_json::json!({"http.header_name": "x-api-key", "http.header_scheme": false}),
    ] {
        assert!(
            config_from_json(serde_json::json!({
                "key_secret": "test-key",
                "attributes": attributes
            }))
            .is_err()
        );
    }
}

/// Scenario: Config parsing receives an unknown field.
/// Guarantees: Unsupported settings are rejected rather than silently ignored.
#[test]
fn config_rejects_unknown_fields() {
    assert!(
        config_from_json(serde_json::json!({
            "key_secret": "test-key",
            "attributes": {"http.header_name": "x-api-key"},
            "unexpected": true
        }))
        .is_err()
    );
}

/// Scenario: File refresh intervals are configured around the scheduler boundaries.
/// Guarantees: Ten seconds through 365 days are accepted and values outside are rejected.
#[test]
fn config_file_refresh_validates_boundaries() {
    for (duration, valid) in [
        ("9s", false),
        ("10s", true),
        ("365d", true),
        ("366d", false),
    ] {
        let result = config_from_json(serde_json::json!({
            "key_secret_file": "api-key",
            "key_secret_file_refresh": duration,
            "attributes": {"http.header_name": "x-api-key"}
        }));
        assert_eq!(result.is_ok(), valid, "unexpected result for {duration}");
    }
}

/// Scenario: The flat-file API key authentication factory is registered.
/// Guarantees: Its URN and shared API key provider capability are advertised.
#[test]
fn factory_is_registered_with_capability() {
    assert_eq!(
        FLAT_FILE_API_KEY_AUTH_EXTENSION.name,
        FLAT_FILE_API_KEY_AUTH_URN
    );
    let capabilities = FLAT_FILE_API_KEY_AUTH_EXTENSION
        .capabilities
        .as_ref()
        .expect("active extension advertises capabilities");
    assert!(capabilities.shared.contains(&"api_key_provider"));
}

fn create_bundle(config: serde_json::Value) -> Result<ExtensionBundle, ConfigError> {
    let (ext_ctx, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name: otel_arrow_dfe_config::ExtensionId = "flat-file-api-key-auth".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        FLAT_FILE_API_KEY_AUTH_URN.into(),
        config,
    ));
    let extension_config = ExtensionConfig::new(name.clone());
    create(&ext_ctx, name, user_config, &extension_config)
}

/// Scenario: The factory creates an extension from a valid inline configuration.
/// Guarantees: Wiring produces a shared active extension bundle.
#[test]
fn create_builds_shared_active_bundle() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let bundle = create_bundle(serde_json::json!({
        "key_secret": "test-key",
        "attributes": {"http.header_name": "x-api-key"}
    }))
    .expect("valid config wires successfully");

    assert!(bundle.local().is_none());
    let shared = bundle.shared().expect("shared variant is produced");
    assert_eq!(shared.variant(), ExtensionVariant::Shared);
    assert!(!shared.is_passive());
}

fn make_extension(config: Config) -> FlatFileApiKeyAuthExtension {
    let (tx, _rx) = watch::channel(None);
    let refresh_policy = if config.key_secret_file.is_some() {
        BackgroundProviderRefreshPolicy::periodic(config.key_secret_file_refresh)
            .expect("valid periodic refresh policy")
    } else {
        BackgroundProviderRefreshPolicy::once()
    };
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<FlatFileApiKeyAuthMetrics>(EmptyAttributes());
    FlatFileApiKeyAuthExtension::new(
        "test-ext",
        FlatFileApiKeyAuth::new(config),
        refresh_policy,
        tx,
        BackgroundProviderMetricsTracker::new(metric_set),
    )
}

fn inline_config() -> Config {
    Config {
        key_secret: Some("test-key".into()),
        key_secret_file: None,
        key_secret_file_refresh: Duration::from_secs(60),
        attributes: valid_attributes(),
    }
}

/// Scenario: An API key is acquired from an inline secret.
/// Guarantees: The key and configured HTTP attributes are returned without expiration.
#[tokio::test]
async fn get_inline_api_key() {
    let key = make_extension(inline_config())
        .get_api_key()
        .await
        .expect("key acquired");

    assert_eq!(key.expose_value(), "test-key");
    assert_eq!(key.get_http_header_name_attribute(), Some("x-api-key"));
    assert_eq!(key.get_http_header_scheme_attribute(), Some("ApiKey"));
    assert!(key.get_expires_on().is_none());
}

/// Scenario: Both inline and file key forms are configured.
/// Guarantees: The trimmed file value takes precedence while preserving non-newline whitespace.
#[tokio::test]
async fn key_file_takes_precedence() {
    let mut file = NamedTempFile::new().expect("file created");
    file.write_all(b"file-key  \r\n").expect("content written");
    let source = FlatFileApiKeyAuth::new(Config {
        key_secret: Some("inline-key".into()),
        key_secret_file: Some(file.path().into()),
        key_secret_file_refresh: Duration::from_secs(10),
        attributes: valid_attributes(),
    });

    let key = source.fetch().await.expect("key acquired");
    assert_eq!(key.expose_value(), "file-key  ");
}

/// Scenario: An API key file is rewritten between two acquisitions.
/// Guarantees: Each acquisition re-reads the file and observes the rotated key.
#[tokio::test]
async fn key_file_rotation_takes_effect() {
    let directory = tempfile::tempdir().expect("tempdir created");
    let path = directory.path().join("api-key");
    std::fs::write(&path, "key-1").expect("initial key written");
    let source = FlatFileApiKeyAuth::new(Config {
        key_secret: None,
        key_secret_file: Some(path.clone()),
        key_secret_file_refresh: Duration::from_secs(300),
        attributes: valid_attributes(),
    });

    let first = source.fetch().await.expect("first key acquired");
    std::fs::write(&path, "key-2").expect("rotated key written");
    let second = source.fetch().await.expect("second key acquired");
    assert_eq!(first.expose_value(), "key-1");
    assert_eq!(second.expose_value(), "key-2");
}

/// Scenario: The active refresh loop reaches its interval after an API key file rotates.
/// Guarantees: The loop re-reads the file and publishes the rotated key to subscribers.
#[tokio::test(start_paused = true)]
async fn background_refresh_publishes_rotated_key() {
    let directory = tempfile::tempdir().expect("tempdir created");
    let path = directory.path().join("api-key");
    std::fs::write(&path, "key-1").expect("initial key written");
    let extension = make_extension(Config {
        key_secret: None,
        key_secret_file: Some(path.clone()),
        key_secret_file_refresh: Duration::from_secs(300),
        attributes: valid_attributes(),
    });
    let mut stream = extension.api_key_stream();

    let (control_tx, control_rx) = tokio::sync::mpsc::channel(1);
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let control = ControlChannel::new(SharedReceiver::mpsc(control_rx), shutdown_rx);
    let effect_handler =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler("test-ext".into());
    let task = tokio::spawn(Box::new(extension).start(control, effect_handler));

    assert_eq!(
        stream
            .next()
            .await
            .expect("initial key published")
            .expose_value(),
        "key-1"
    );
    std::fs::write(&path, "key-2").expect("rotated key written");
    tokio::time::advance(Duration::from_secs(299)).await;
    assert!(stream.next().now_or_never().is_none());
    tokio::time::advance(Duration::from_secs(1)).await;
    assert_eq!(
        stream
            .next()
            .await
            .expect("rotated key published")
            .expose_value(),
        "key-2"
    );

    drop(control_tx);
    let _terminal_state = task
        .await
        .expect("refresh task joins")
        .expect("refresh loop exits cleanly");
}

/// Scenario: API key files contain invalid UTF-8 or only line endings.
/// Guarantees: Acquisition rejects invalid and empty file-derived keys.
#[tokio::test]
async fn key_file_rejects_invalid_content() {
    for content in [&[0xff, 0xfe][..], b"\r\n"] {
        let mut file = NamedTempFile::new().expect("file created");
        file.write_all(content).expect("content written");
        let source = FlatFileApiKeyAuth::new(Config {
            key_secret: None,
            key_secret_file: Some(file.path().into()),
            key_secret_file_refresh: Duration::from_secs(10),
            attributes: valid_attributes(),
        });
        assert!(source.fetch().await.is_err());
    }
}

/// Scenario: An API key file exceeds the shared four-megabyte file limit.
/// Guarantees: Acquisition rejects the file instead of loading oversized secret data.
#[tokio::test]
async fn key_file_rejects_oversized_content() {
    let directory = tempfile::tempdir().expect("tempdir created");
    let path = directory.path().join("api-key");
    std::fs::write(&path, vec![b'x'; 5 * 1024 * 1024]).expect("oversized key written");
    let source = FlatFileApiKeyAuth::new(Config {
        key_secret: None,
        key_secret_file: Some(path),
        key_secret_file_refresh: Duration::from_secs(300),
        attributes: valid_attributes(),
    });

    let error = source
        .fetch()
        .await
        .expect_err("oversized file is rejected");
    assert!(error.to_string().contains("too large"));
}

/// Scenario: A stream subscribes before the first direct API key acquisition.
/// Guarantees: The acquired key is published to the waiting subscriber.
#[tokio::test]
async fn api_key_stream_receives_acquisition() {
    let extension = make_extension(inline_config());
    let mut stream = extension.api_key_stream();
    let acquired = extension.get_api_key().await.expect("key acquired");
    let streamed = stream.next().await.expect("key published");
    assert_eq!(streamed.expose_value(), acquired.expose_value());
}
