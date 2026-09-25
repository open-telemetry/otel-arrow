// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the flat file user pass extension.

use std::io::Write;

use futures::{FutureExt, StreamExt};
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::shared::capability::auth::basic_auth_provider::BasicAuthProvider;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension as SharedExtension};
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use secrecy::{ExposeSecret, SecretString};
use tempfile::NamedTempFile;

use super::config::Config;
use super::*;
use crate::common::background_refresh::BackgroundProviderSource;
use crate::flat_file_user_pass_auth::config::default_password_secret_file_refresh;

// -- Config tests -------------------------------------------

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

/// Scenario: A valid config omits the optional file refresh interval.
/// Guarantees: Config parsing preserves supplied values and applies the default refresh interval.
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

/// Scenario: Config parsing receives missing, empty, or invalid usernames.
/// Guarantees: Every invalid username configuration is rejected.
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

/// Scenario: Config parsing receives a missing, empty, or invalid password secret.
/// Guarantees: Every invalid password secret configuration is rejected.
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

/// Scenario: Config parsing receives an unsupported password file refresh interval.
/// Guarantees: The invalid refresh interval is rejected.
#[test]
fn config_password_secret_file_refresh_rejects_invalid() {
    assert!(
        config_from_json(serde_json::json!({
        "username": "test",
        "password_secret": "<test_secret>",
        "password_secret_file": "<test_secret_path>",
        "password_secret_file_refresh": "9s" }))
        .is_err()
    )
}

/// Scenario: Config parsing receives the minimum supported password file refresh interval.
/// Guarantees: The inclusive ten-second scheduler boundary is accepted.
#[test]
fn config_password_secret_file_refresh_accepts_minimum() {
    let cfg = config_from_json(serde_json::json!({
        "username": "test",
        "password_secret_file": "<test_secret_path>",
        "password_secret_file_refresh": "10s"
    }))
    .expect("minimum refresh interval is valid");

    assert_eq!(
        cfg.password_secret_file_refresh,
        MINIMUM_BASIC_AUTH_CREDENTIAL_REFRESH_INTERVAL
    );
}

/// Scenario: Config validation receives a refresh interval that cannot form an `Instant` deadline.
/// Guarantees: An unsupported refresh interval is rejected before the extension starts.
#[test]
fn config_password_secret_file_refresh_rejects_unsupported_maximum() {
    let cfg = Config {
        username: "test".into(),
        password_secret: None,
        password_secret_file: Some("<test_secret_path>".into()),
        password_secret_file_refresh: Duration::MAX,
    };

    let err = cfg
        .validate()
        .expect_err("an unsupported refresh interval must be rejected");
    assert!(err.contains("too large for this platform"));
}

/// Scenario: Config parsing receives an unrecognized field.
/// Guarantees: Unknown fields are rejected instead of being silently ignored.
#[test]
fn config_rejects_unknown_fields() {
    assert!(
        config_from_json(serde_json::json!({
            "username": "test",
            "password_secret": "test_pass",
            "unexpected": true
        }))
        .is_err()
    );
}

// -- Factory tests ------------------------------------------

/// Scenario: The flat-file authentication factory is registered.
/// Guarantees: Its URN and shared basic authentication capability are advertised.
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

/// Scenario: The factory's `create` hook runs against a valid config.
/// Guarantees: Wiring succeeds and yields a shared, active extension bundle usable by the engine.
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

/// Scenario: The factory's `create` hook runs against a config that fails validation.
/// Guarantees: Wiring fails fast with InvalidUserConfig instead of building a broken extension.
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
    let refresh_policy = if config.password_secret_file.is_some() {
        BackgroundProviderRefreshPolicy::periodic(config.password_secret_file_refresh)
            .expect("valid periodic refresh policy")
    } else {
        BackgroundProviderRefreshPolicy::once()
    };
    FlatFileUserPassAuthExtension::new(
        "test-ext",
        FlatFileUserPassAuth::new(config),
        refresh_policy,
        tx,
        make_tracker(),
    )
}

fn make_tracker() -> BackgroundProviderMetricsTracker<FlatFileUserPassAuthMetrics> {
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<FlatFileUserPassAuthMetrics>(EmptyAttributes());
    BackgroundProviderMetricsTracker::new(metric_set)
}

/// Scenario: Credentials are acquired from an inline password secret.
/// Guarantees: The configured username and password are returned without expiration.
#[tokio::test]
async fn get_credential() {
    let ext = make_extension();

    let credential = ext.get_credential().await.expect("first acquisition");
    assert_eq!(credential.expose_username(), "test_user");
    assert_eq!(credential.expose_password(), "test_pass");
    assert!(credential.expires_on().is_none());
}

/// Scenario: Credentials are acquired from a readable password secret file.
/// Guarantees: The file password is trimmed only at line endings and remains non-expiring.
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
    assert!(credential.expires_on().is_none());
}

/// Scenario: Both inline and file password forms are configured.
/// Guarantees: The file password takes precedence over the inline password.
#[tokio::test]
async fn password_file_takes_precedence_over_inline_secret() {
    let mut named_file = NamedTempFile::new().expect("file created");
    named_file.write_all(b"file_pass").expect("content written");

    let source = FlatFileUserPassAuth::new(Config {
        username: "test_user".into(),
        password_secret: Some("inline_pass".into()),
        password_secret_file: Some(named_file.path().into()),
        password_secret_file_refresh: Duration::from_secs(300),
    });

    let credential = source.fetch().await.expect("credential acquired");
    assert_eq!(credential.expose_password(), "file_pass");
}

/// Scenario: A password file is rewritten between two credential acquisitions.
/// Guarantees: Each acquisition re-reads the file and observes the rotated password.
#[tokio::test]
async fn password_file_rotation_takes_effect() {
    let dir = tempfile::tempdir().expect("tempdir created");
    let password_path = dir.path().join("password");
    std::fs::write(&password_path, "password-1").expect("initial password written");
    let source = FlatFileUserPassAuth::new(Config {
        username: "test_user".into(),
        password_secret: None,
        password_secret_file: Some(password_path.clone()),
        password_secret_file_refresh: Duration::from_secs(300),
    });

    let first = source.fetch().await.expect("first credential acquired");
    std::fs::write(&password_path, "password-2").expect("rotated password written");
    let second = source.fetch().await.expect("second credential acquired");

    assert_eq!(first.expose_password(), "password-1");
    assert_eq!(second.expose_password(), "password-2");
}

/// Scenario: The active refresh loop reaches the scheduled refresh after a password file rotates.
/// Guarantees: The loop re-reads the file and publishes the rotated credential to subscribers.
#[tokio::test(start_paused = true)]
async fn background_refresh_publishes_rotated_password() {
    let dir = tempfile::tempdir().expect("tempdir created");
    let password_path = dir.path().join("password");
    std::fs::write(&password_path, "password-1").expect("initial password written");
    let extension = make_extension_with_config(Config {
        username: "test_user".into(),
        password_secret: None,
        password_secret_file: Some(password_path.clone()),
        password_secret_file_refresh: Duration::from_secs(300),
    });
    let mut stream = extension.credential_stream();

    let (control_tx, control_rx) = tokio::sync::mpsc::channel(1);
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let control = ControlChannel::new(SharedReceiver::mpsc(control_rx), shutdown_rx);
    let effect_handler =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler("test-ext".into());
    let task = tokio::spawn(Box::new(extension).start(control, effect_handler));

    let initial = stream.next().await.expect("initial credential published");
    assert_eq!(initial.expose_password(), "password-1");

    std::fs::write(&password_path, "password-2").expect("rotated password written");
    tokio::time::advance(Duration::from_secs(299)).await;
    assert!(
        stream.next().now_or_never().is_none(),
        "the file must not be polled before the configured interval"
    );
    tokio::time::advance(Duration::from_secs(1)).await;

    let rotated = stream.next().await.expect("rotated credential published");
    assert_eq!(rotated.expose_password(), "password-2");

    drop(control_tx);
    let _terminal_state = task
        .await
        .expect("refresh task joins")
        .expect("refresh loop exits cleanly");
}

/// Scenario: A credential is acquired from a password file with periodic polling configured.
/// Guarantees: The polling interval is not represented as credential expiry.
#[tokio::test]
async fn file_credential_does_not_inherit_polling_interval_as_expiry() {
    let mut named_file = NamedTempFile::new().expect("file created");
    named_file.write_all(b"test_pass").expect("content written");
    let source = FlatFileUserPassAuth::new(Config {
        username: "test_user".into(),
        password_secret: None,
        password_secret_file: Some(named_file.path().into()),
        password_secret_file_refresh: Duration::from_secs(300),
    });

    let credential = source.fetch().await.expect("credential acquired");
    assert!(credential.expires_on().is_none());
}

/// Scenario: A password secret file contains bytes that are not valid UTF-8.
/// Guarantees: Acquisition fails with an explicit encoding error instead of using mangled bytes.
#[tokio::test]
async fn get_credential_file_rejects_non_utf8_content() {
    let mut named_file = NamedTempFile::new().expect("file created");
    named_file
        .write_all(&[0xff, 0xfe, 0xfd])
        .expect("content written");

    let source = FlatFileUserPassAuth::new(Config {
        username: "test_user".into(),
        password_secret: None,
        password_secret_file: Some(named_file.path().into()),
        password_secret_file_refresh: Duration::from_secs(10),
    });

    let err = source
        .fetch()
        .await
        .expect_err("non-UTF-8 credentials must be rejected");
    assert!(
        err.to_string().contains("valid UTF-8"),
        "unexpected error: {err}"
    );
}

/// Scenario: A password secret file contains an empty password or a control character.
/// Guarantees: Acquisition applies Basic Auth password validation to file-derived values.
#[tokio::test]
async fn get_credential_file_rejects_invalid_password_content() {
    for content in ["", "test\tpass"] {
        let mut named_file = NamedTempFile::new().expect("file created");
        named_file
            .write_all(content.as_bytes())
            .expect("content written");

        let source = FlatFileUserPassAuth::new(Config {
            username: "test_user".into(),
            password_secret: None,
            password_secret_file: Some(named_file.path().into()),
            password_secret_file_refresh: Duration::from_secs(10),
        });

        let err = source
            .fetch()
            .await
            .expect_err("invalid Basic Auth passwords must be rejected");
        assert!(
            err.to_string().contains("Password is invalid"),
            "unexpected error for {content:?}: {err}"
        );
    }
}

/// Scenario: A password secret file exceeds the collector's shared four-megabyte size limit.
/// Guarantees: Acquisition rejects the file instead of loading oversized credential data.
#[tokio::test]
async fn get_credential_file_rejects_oversized_content() {
    let dir = tempfile::tempdir().expect("tempdir created");
    let password_path = dir.path().join("password");
    std::fs::write(&password_path, vec![b'x'; 5 * 1024 * 1024])
        .expect("oversized password written");
    let source = FlatFileUserPassAuth::new(Config {
        username: "test_user".into(),
        password_secret: None,
        password_secret_file: Some(password_path),
        password_secret_file_refresh: Duration::from_secs(300),
    });

    let err = source
        .fetch()
        .await
        .expect_err("oversized credential files must be rejected");
    assert!(
        err.to_string().contains("too large"),
        "unexpected error: {err}"
    );
}

/// Scenario: An unreadable password file and a valid inline password are both configured.
/// Guarantees: Acquisition reports the file failure instead of falling back to the inline value.
#[tokio::test]
async fn password_file_failure_does_not_fallback_to_inline_secret() {
    let dir = tempfile::tempdir().expect("tempdir created");
    let missing_path = dir.path().join("missing-password");
    let source = FlatFileUserPassAuth::new(Config {
        username: "test_user".into(),
        password_secret: Some("inline_pass".into()),
        password_secret_file: Some(missing_path.clone()),
        password_secret_file_refresh: Duration::from_secs(300),
    });

    let err = source
        .fetch()
        .await
        .expect_err("an unreadable preferred file must fail acquisition");
    assert!(
        err.to_string()
            .contains(&missing_path.to_string_lossy().to_string()),
        "error must name the preferred file, got: {err}"
    );
}

/// Scenario: Credential acquisition references an unreadable password secret file.
/// Guarantees: Acquisition returns an error that names the offending path.
#[tokio::test]
async fn get_credential_file_failure() {
    let dir = tempfile::tempdir().expect("tempdir created");
    let missing_path = dir.path().join("missing-password");
    let source = FlatFileUserPassAuth::new(Config {
        username: "test_user".into(),
        password_secret: None,
        password_secret_file: Some(missing_path.clone()),
        password_secret_file_refresh: Duration::from_secs(300),
    });

    let err = source
        .fetch()
        .await
        .expect_err("an unreadable credential file must fail acquisition");
    assert!(
        err.to_string()
            .contains(&missing_path.to_string_lossy().to_string()),
        "error must name the offending path, got: {err}"
    );
}

/// Scenario: A credential is requested directly and then through the credential stream.
/// Guarantees: Both paths return the configured username and inline password.
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

/// Scenario: A credential stream subscribes before the first credential acquisition.
/// Guarantees: A later acquisition is published to the waiting stream subscriber.
#[tokio::test]
async fn credential_stream_receives_later_acquisition() {
    let ext = make_extension();
    let mut stream = ext.credential_stream();

    let acquired = ext.get_value().await.expect("credential acquired");
    let streamed = stream.next().await.expect("credential published");

    assert_eq!(streamed.expose_username(), acquired.expose_username());
    assert_eq!(streamed.expose_password(), acquired.expose_password());
}
