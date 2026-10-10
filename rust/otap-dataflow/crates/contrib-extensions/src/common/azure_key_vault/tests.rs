// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Offline tests for shared Key Vault configuration and safe diagnostics.

use super::config::Config;
use super::testing::config_value;

pub(super) fn parse(value: serde_json::Value) -> Config {
    serde_json::from_value(value).expect("config deserializes")
}

/// Scenario: Identity, timeout, and secret versions are omitted.
/// Guarantees: Startup defaults to managed identity, 30 seconds, and latest versions.
#[test]
fn startup_defaults() {
    let config = parse(config_value());
    config.validate().expect("valid defaults");
    assert_eq!(
        config.identity.method,
        crate::common::azure_identity::AuthMethod::ManagedIdentity
    );
    assert!(config.identity.client_id.is_none());
    assert!(config.username_secret.version.is_none());
    assert!(config.password_secret.version.is_none());
    assert_eq!(config.startup_timeout, std::time::Duration::from_secs(30));
}

/// Scenario: An endpoint selects public, US Government, or China Key Vault.
/// Guarantees: Supported Azure endpoints accept only the HTTPS data-plane root.
#[test]
fn supported_cloud_endpoints() {
    for endpoint in [
        "https://test-vault.vault.azure.net",
        "https://test-vault.vault.azure.net/",
        "https://test-vault.vault.azure.net:443/",
        "https://test-vault.vault.usgovcloudapi.net/",
        "https://test-vault.vault.azure.cn/",
    ] {
        let mut value = config_value();
        value["vault_url"] = endpoint.into();
        parse(value).validate().expect("supported Azure endpoint");
    }
}

/// Scenario: Endpoint input contains unsafe hosts, userinfo, or normalized paths.
/// Guarantees: Validation rejects token destinations outside supported vault roots.
#[test]
fn rejects_unsafe_endpoints() {
    for endpoint in [
        "",
        "http://test-vault.vault.azure.net",
        "https://localhost",
        "https://127.0.0.1/",
        "https://test-vault.vault.azure.net.evil.example",
        "https://vault.azure.net/",
        "https://test-vault.other.vault.azure.net/",
        "https://test-vault.vault.azure.net:8443",
        "https://user:secret@test-vault.vault.azure.net/",
        "https://@test-vault.vault.azure.net/",
        "https://test-vault.vault.azure.net/secrets",
        "https://test-vault.vault.azure.net/x/..",
        "https://test-vault.vault.azure.net/?token=secret",
        "https://test-vault.vault.azure.net/#secret",
        "https://test-vault.vault.azure.net\\",
        " https://test-vault.vault.azure.net",
    ] {
        let mut value = config_value();
        value["vault_url"] = endpoint.into();
        let error = parse(value).validate().expect_err("unsafe endpoint");
        assert!(!error.contains(endpoint) || endpoint.is_empty());
        assert!(!error.contains("token=secret"));
    }
}

/// Scenario: Shared source construction bypasses the extension's configuration hook.
/// Guarantees: Unsafe token destinations are still rejected before creating an SDK client or identity.
#[test]
fn source_constructor_enforces_endpoint_validation() {
    let mut config = parse(config_value());
    config.vault_url = "https://unsafe.example".to_owned();
    let error = super::Source::new(config).unwrap_err();
    assert_eq!(error.stage, super::error::Stage::Client);
    assert_eq!(error.kind, super::error::FailureKind::Configuration);
    assert!(!error.to_string().contains("unsafe.example"));
}

/// Scenario: Secret names and versions contain invalid or empty path segments.
/// Guarantees: Both references reject unsafe input and accept independent version pins.
#[test]
fn validates_both_secret_references() {
    for field in ["username_secret", "password_secret"] {
        for name in ["", "name/value", "..", "name?query", " name", "name_"] {
            let mut value = config_value();
            value[field]["name"] = name.into();
            assert!(parse(value).validate().is_err());
        }
        for version in [
            "",
            "latest",
            "../secret",
            "not-hexadecimal-version-xxxxxxxxx",
        ] {
            let mut value = config_value();
            value[field]["version"] = version.into();
            assert!(parse(value).validate().is_err());
        }
    }
    let mut value = config_value();
    value["username_secret"]["version"] = "0123456789abcdef0123456789abcdef".into();
    value["password_secret"]["version"] = "fedcba9876543210fedcba9876543210".into();
    let config = parse(value);
    config.validate().expect("independent pins");
    assert_ne!(
        config.username_secret.version,
        config.password_secret.version
    );
}

/// Scenario: Unknown options appear at each configuration nesting level.
/// Guarantees: Typos, refresh options, and credential fallbacks are never accepted.
#[test]
fn rejects_unknown_options_and_zero_timeout() {
    for field in [
        "refresh_interval",
        "username",
        "password_secret_file",
        "scope",
    ] {
        let mut value = config_value();
        value[field] = "unexpected".into();
        assert!(serde_json::from_value::<Config>(value).is_err());
    }
    for field in ["identity", "username_secret", "password_secret"] {
        let mut value = config_value();
        value[field]["unknown"] = true.into();
        assert!(serde_json::from_value::<Config>(value).is_err());
    }
    let mut value = config_value();
    value["startup_timeout"] = "0s".into();
    assert!(parse(value).validate().is_err());
}

/// Scenario: SDK failures contain tokens, response messages, and service error codes.
/// Guarantees: Only typed status/category and acquisition stage survive conversion.
#[test]
fn classifies_sdk_errors_without_retaining_sensitive_data() {
    use super::error::{Error, FailureKind, Stage};
    use azure_core::error::ErrorKind;
    use azure_core::http::StatusCode;

    for (status, expected) in [
        (401, FailureKind::Authentication),
        (403, FailureKind::Authorization),
        (404, FailureKind::NotFound),
        (408, FailureKind::Transient),
        (429, FailureKind::Throttled),
        (500, FailureKind::Transient),
        (503, FailureKind::Transient),
        (400, FailureKind::Service),
        (302, FailureKind::Service),
    ] {
        let sdk_error = azure_core::Error::with_message(
            ErrorKind::HttpResponse {
                status: StatusCode::from(status),
                error_code: Some("sensitive-service-error-code".to_owned()),
                raw_response: None,
            },
            "secret-body bearer-token password username",
        );
        let safe = Error::from_sdk(Stage::Password, sdk_error);
        assert_eq!(safe.kind, expected);
        assert_eq!(safe.stage, Stage::Password);
        assert!(std::error::Error::source(&safe).is_none());
        let output = format!("{safe:?} {safe}");
        assert!(!output.contains("secret-body"));
        assert!(!output.contains("bearer-token"));
        assert!(!output.contains("sensitive-service-error-code"));
    }
    for (kind, expected) in [
        (ErrorKind::Credential, FailureKind::Authentication),
        (ErrorKind::Connection, FailureKind::Transient),
        (ErrorKind::Io, FailureKind::Transient),
        (ErrorKind::DataConversion, FailureKind::InvalidResponse),
        (ErrorKind::Other, FailureKind::Service),
    ] {
        let safe = Error::from_sdk(
            Stage::Username,
            azure_core::Error::with_message(kind, "sensitive-token"),
        );
        assert_eq!(safe.kind, expected);
        assert!(!format!("{safe:?} {safe}").contains("sensitive-token"));
        safe.log_failure();
    }
}

/// Scenario: SDK requests use latest or independent pinned versions and negotiate a Key Vault challenge.
/// Guarantees: The exact request paths are forwarded, both values validate, and no Azure Monitor scope is requested.
#[tokio::test]
async fn sdk_forwards_versions_and_uses_key_vault_challenge_scope() {
    use super::testing::*;
    use azure_core::http::RetryOptions;
    use secrecy::ExposeSecret;
    use std::sync::Arc;
    for pinned in [false, true] {
        let mut config = config();
        if pinned {
            config.username_secret.version = Some("0123456789abcdef0123456789abcdef".into());
            config.password_secret.version = Some("fedcba9876543210fedcba9876543210".into());
        }
        let username_path = format!(
            "/secrets/sasl-username/{}",
            config.username_secret.version.as_deref().unwrap_or("")
        );
        let password_path = format!(
            "/secrets/sasl-password/{}",
            config.password_secret.version.as_deref().unwrap_or("")
        );
        let transport = FakeTransport::new(vec![
            secret("user-value-marker"),
            secret("password-value-marker"),
        ]);
        let credential = Arc::new(FakeCredential::default());
        let source = source(
            config,
            Arc::clone(&transport),
            Arc::clone(&credential),
            RetryOptions::none(),
        );
        let pair = source.fetch_pair().await.expect("both SDK reads succeed");
        assert_eq!(pair.username.expose_secret(), "user-value-marker");
        assert_eq!(pair.password.expose_secret(), "password-value-marker");
        assert_eq!(
            *transport.requests.lock().unwrap(),
            vec![
                (username_path.clone(), false),
                (username_path, true),
                (password_path, true),
            ]
        );
        assert_eq!(
            *credential.scopes.lock().unwrap(),
            vec![vec!["https://vault.azure.net/.default".to_owned()]]
        );
        assert!(!format!("{source:?} {pair:?}").contains("password-value-marker"));
        assert!(!format!("{source:?} {pair:?}").contains("user-value-marker"));
    }
}

/// Scenario: SDK responses fail for either secret with typed service status codes.
/// Guarantees: Authentication, authorization, missing, throttled, and transient failures have safe stage-specific diagnostics.
#[tokio::test]
async fn sdk_classifies_both_secret_failures_and_redacts_response_bodies() {
    use super::error::{FailureKind, Stage};
    use super::testing::*;
    use azure_core::http::RetryOptions;
    use std::sync::Arc;
    for stage in [Stage::Username, Stage::Password] {
        for (status, expected) in [
            (401, FailureKind::Authentication),
            (403, FailureKind::Authorization),
            (404, FailureKind::NotFound),
            (429, FailureKind::Throttled),
            (503, FailureKind::Transient),
        ] {
            let mut replies = Vec::new();
            if stage == Stage::Password {
                replies.push(secret("username-sensitive-marker"));
            }
            replies.push(failure(status));
            let transport = FakeTransport::new(replies);
            let source = source(
                config(),
                transport,
                Arc::new(FakeCredential::default()),
                RetryOptions::none(),
            );
            let error = source.fetch_pair().await.unwrap_err();
            assert_eq!(error.stage, stage);
            assert_eq!(error.kind, expected);
            assert!(std::error::Error::source(&error).is_none());
            let message = format!("{error:?} {error}");
            for marker in [TOKEN, PAYLOAD, "username-sensitive-marker"] {
                assert!(!message.contains(marker));
            }
        }
    }
}

/// Scenario: Identity acquisition fails or a vault challenges for an unrelated resource.
/// Guarantees: Safe authentication/challenge errors are returned and an untrusted challenge receives no token.
#[tokio::test]
async fn sdk_authentication_and_challenge_validation_are_safe() {
    use super::error::FailureKind;
    use super::testing::*;
    use azure_core::http::RetryOptions;
    use std::sync::Arc;
    let transport = FakeTransport::new(vec![]);
    let credential = Arc::new(FakeCredential {
        fail: true,
        ..Default::default()
    });
    let source = source(
        config(),
        Arc::clone(&transport),
        credential,
        RetryOptions::none(),
    );
    let error = source.fetch_pair().await.unwrap_err();
    assert_eq!(error.kind, FailureKind::Authentication);
    assert_eq!(transport.request_count(), 1);
    assert!(!format!("{error:?} {error}").contains(TOKEN));

    let transport = FakeTransport::with_challenge("https://evil.example");
    let credential = Arc::new(FakeCredential::default());
    let source = super::testing::source(
        config(),
        Arc::clone(&transport),
        Arc::clone(&credential),
        RetryOptions::none(),
    );
    assert!(source.fetch_pair().await.is_err());
    assert!(credential.scopes.lock().unwrap().is_empty());
    assert_eq!(transport.request_count(), 1);
}

/// Scenario: Either secret contains missing/empty/malformed values or inactive metadata.
/// Guarantees: No validated pair is returned, with exact invalid-value or malformed-response classification.
#[tokio::test]
async fn sdk_rejects_invalid_values_and_metadata_for_each_secret() {
    use super::error::{FailureKind, Stage};
    use super::testing::*;
    use azure_core::http::RetryOptions;
    use std::sync::Arc;
    for stage in [Stage::Username, Stage::Password] {
        for (body, expected) in [
            (serde_json::json!({}), FailureKind::InvalidValue),
            (serde_json::json!({"value":""}), FailureKind::InvalidValue),
            (
                serde_json::json!({"value":12}),
                FailureKind::InvalidResponse,
            ),
            (
                serde_json::json!({"value":"sensitive-value", "attributes":{"enabled":false}}),
                FailureKind::InvalidValue,
            ),
            (
                serde_json::json!({"value":"sensitive-value", "attributes":{"exp":1}}),
                FailureKind::InvalidValue,
            ),
            (
                serde_json::json!({"value":"sensitive-value", "attributes":{"nbf":4102444800i64}}),
                FailureKind::InvalidValue,
            ),
        ] {
            let mut replies = Vec::new();
            if stage == Stage::Password {
                replies.push(secret("user"));
            }
            replies.push(Reply::Json(200, body));
            let source = source(
                config(),
                FakeTransport::new(replies),
                Arc::new(FakeCredential::default()),
                RetryOptions::none(),
            );
            let error = source.fetch_pair().await.unwrap_err();
            assert_eq!(error.stage, stage);
            assert_eq!(error.kind, expected);
            assert!(!format!("{error:?} {error}").contains("sensitive-value"));
        }
    }
}

/// Scenario: The production SDK retry policy sees a transient secret-read failure.
/// Guarantees: Its built-in retry recovers without adding a custom source retry layer.
#[tokio::test]
async fn sdk_default_retry_recovers_transient_failure() {
    use super::testing::*;
    use azure_core::http::RetryOptions;
    use std::sync::Arc;
    let transport = FakeTransport::new(vec![failure(503), secret("user"), secret("password")]);
    let source = source(
        config(),
        Arc::clone(&transport),
        Arc::new(FakeCredential::default()),
        RetryOptions::default(),
    );
    let _ = source
        .fetch_pair()
        .await
        .expect("SDK retries recover within timeout");
    assert_eq!(transport.request_count(), 4);
}

/// Scenario: SDK debug logs and operational events observe successful reads and secret-bearing errors.
/// Guarantees: Captured diagnostics contain stages/categories but no token, secret value, or raw error-body marker.
#[tokio::test]
async fn sdk_and_operational_telemetry_redact_sensitive_material() {
    use super::testing::*;
    use azure_core::http::RetryOptions;
    use std::io::Write;
    use std::sync::{Arc, Mutex};
    use tracing::instrument::WithSubscriber;

    #[derive(Clone)]
    struct Capture(Arc<Mutex<Vec<u8>>>);

    impl Write for Capture {
        fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
            self.0.lock().unwrap().extend_from_slice(bytes);
            Ok(bytes.len())
        }
        fn flush(&mut self) -> std::io::Result<()> {
            Ok(())
        }
    }
    impl<'a> tracing_subscriber::fmt::MakeWriter<'a> for Capture {
        type Writer = Self;
        fn make_writer(&'a self) -> Self::Writer {
            self.clone()
        }
    }

    let capture = Capture(Arc::new(Mutex::new(Vec::new())));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .without_time()
        .with_max_level(tracing::Level::DEBUG)
        .with_writer(capture.clone())
        .finish();
    async {
        let source = source(
            config(),
            FakeTransport::new(vec![secret("username-value-marker"), failure(403)]),
            Arc::new(FakeCredential::default()),
            RetryOptions::none(),
        );
        let error = source.fetch_pair().await.unwrap_err();
        error.log_failure();
        let source = super::testing::source(
            config(),
            FakeTransport::new(vec![]),
            Arc::new(FakeCredential {
                fail: true,
                ..Default::default()
            }),
            RetryOptions::none(),
        );
        source.fetch_pair().await.unwrap_err().log_failure();
    }
    .with_subscriber(subscriber)
    .await;
    let logs = String::from_utf8(capture.0.lock().unwrap().clone()).unwrap();
    assert!(logs.contains("stage=\"password\""));
    assert!(logs.contains("authorization"));
    assert!(logs.contains("authentication"));
    for marker in [TOKEN, PAYLOAD, "username-value-marker"] {
        assert!(
            !logs.contains(marker),
            "sensitive material leaked into diagnostics"
        );
    }
}
