// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! File-backed SASL configuration, acquisition, and lifecycle tests.

use async_trait::async_trait;
use futures::{FutureExt, StreamExt};
use otel_arrow_dfe_engine::shared::capability::auth::sasl_credential_provider::SaslCredentialProvider as SharedSaslCredentialProvider;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension as SharedExtension};
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use std::time::{Duration, Instant};
use tempfile::TempDir;

use super::error::Error;
use super::*;
use crate::common::background_refresh::BackgroundProviderSource;

struct Fixture {
    _directory: TempDir,
    config: Config,
}

impl Fixture {
    fn new() -> Self {
        let directory = tempfile::tempdir().expect("temporary directory");
        let config = Config {
            username_file: directory.path().join("username"),
            password_secret_file: directory.path().join("password"),
            credentials_file_refresh: Duration::from_secs(300),
        };
        let fixture = Self {
            _directory: directory,
            config,
        };
        fixture.write_pair("user-1", "password-1");
        fixture
    }

    fn write_pair(&self, username: &str, password: &str) {
        std::fs::write(&self.config.username_file, username).expect("write username");
        std::fs::write(&self.config.password_secret_file, password).expect("write password");
    }

    fn source(&self) -> FlatFileSaslAuth {
        FlatFileSaslAuth::new(self.config.clone())
    }

    fn extension(&self) -> FlatFileSaslAuthExtension {
        make_provider(self.source(), self.config.credentials_file_refresh)
    }
}

fn make_provider<S: BackgroundProviderSource<SaslCredential>>(
    source: S,
    interval: Duration,
) -> BackgroundProviderExtension<S, FlatFileSaslAuthMetrics, SaslCredential, SaslCredentialProvider>
{
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<FlatFileSaslAuthMetrics>(EmptyAttributes());
    let (tx, _rx) = watch::channel(None);
    BackgroundProviderExtension::new(
        "test-ext",
        source,
        BackgroundProviderRefreshPolicy::periodic(interval).expect("valid polling interval"),
        tx,
        BackgroundProviderMetricsTracker::new(metric_set),
    )
}

fn config_json() -> serde_json::Value {
    serde_json::json!({
        "username_file": "username",
        "password_secret_file": "password"
    })
}

fn create_bundle(config: serde_json::Value) -> Result<ExtensionBundle, ConfigError> {
    let (context, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name: otel_arrow_dfe_config::ExtensionId = "test-ext".into();
    create(
        &context,
        name.clone(),
        Arc::new(ExtensionUserConfig::new(
            FLAT_FILE_SASL_AUTH_URN.into(),
            config,
        )),
        &ExtensionConfig::new(name),
    )
}

/// Scenario: Only the two file paths are configured.
/// Guarantees: Both paths survive parsing and polling defaults to one hour.
#[test]
fn config_defaults() {
    let config = parse_config(&config_json()).expect("valid configuration");
    assert_eq!(config.username_file, std::path::PathBuf::from("username"));
    assert_eq!(
        config.password_secret_file,
        std::path::PathBuf::from("password")
    );
    assert_eq!(config.credentials_file_refresh, Duration::from_secs(3600));
}

/// Scenario: Either required file path is missing, empty, or null.
/// Guarantees: Invalid file references are rejected before extension startup.
#[test]
fn config_requires_both_paths() {
    for field in ["username_file", "password_secret_file"] {
        for replacement in [
            None,
            Some(serde_json::json!("")),
            Some(serde_json::Value::Null),
        ] {
            let mut value = config_json();
            let fields = value.as_object_mut().expect("configuration object");
            if let Some(replacement) = replacement {
                let _ = fields.insert(field.into(), replacement);
            } else {
                let _ = fields.remove(field);
            }
            let error = parse_config(&value).expect_err("invalid file reference");
            assert!(error.to_string().contains(field));
        }
    }
}

/// Scenario: Configuration includes inline credentials or an unrecognized option.
/// Guarantees: Values cannot silently replace either required file source.
#[test]
fn config_rejects_inline_and_unknown_fields() {
    for field in ["username", "password", "password_secret", "unexpected"] {
        let mut value = config_json();
        value[field] = serde_json::json!("not-supported");
        assert!(
            parse_config(&value).is_err(),
            "field {field} must be rejected"
        );
    }
}

/// Scenario: Polling intervals are supplied on and outside the supported boundaries.
/// Guarantees: Only intervals from ten seconds through 365 days are accepted.
#[test]
fn config_validates_polling_bounds() {
    for (interval, valid) in [
        ("0s", false),
        ("9s", false),
        ("10s", true),
        ("30m", true),
        ("365d", true),
        ("365d 1s", false),
        ("invalid", false),
    ] {
        let mut value = config_json();
        value["credentials_file_refresh"] = serde_json::json!(interval);
        assert_eq!(parse_config(&value).is_ok(), valid, "interval {interval}");
    }
}

/// Scenario: The SASL extension factory is linked and receives valid file configuration.
/// Guarantees: It registers only the SASL capability and builds an active shared bundle.
#[test]
fn factory_registration_and_creation() {
    assert!(
        OTAP_EXTENSION_FACTORIES
            .iter()
            .any(|factory| factory.name == FLAT_FILE_SASL_AUTH_URN)
    );
    let capabilities = FLAT_FILE_SASL_AUTH_EXTENSION
        .capabilities
        .as_ref()
        .expect("advertised capability");
    assert_eq!(capabilities.shared, &["sasl_credential_provider"]);
    assert!(capabilities.local.is_empty());
    let bundle = create_bundle(config_json()).expect("factory creates bundle");
    assert!(bundle.local().is_none());
    let shared = bundle.shared().expect("shared extension");
    assert_eq!(shared.variant(), ExtensionVariant::Shared);
    assert!(!shared.is_passive());
}

/// Scenario: Factory validation and creation receive an incomplete file configuration.
/// Guarantees: Both entry points reject invalid configuration with InvalidUserConfig.
#[test]
fn factory_rejects_invalid_config() {
    assert!(matches!(
        validate_config(&serde_json::json!({})),
        Err(ConfigError::InvalidUserConfig { .. })
    ));
    assert!(matches!(
        create_bundle(serde_json::json!({})),
        Err(ConfigError::InvalidUserConfig { .. })
    ));
}

/// Scenario: Files contain trailing CR/LF, meaningful spaces, and a colon in the username.
/// Guarantees: Only line endings are stripped and Basic Auth restrictions do not affect SASL.
#[tokio::test]
async fn reads_pair_with_sasl_validation() {
    let fixture = Fixture::new();
    fixture.write_pair("sasl:user  \r\n", "password  \n");
    let credential = fixture
        .extension()
        .get_credential()
        .await
        .expect("credential");
    assert_eq!(credential.expose_username(), "sasl:user  ");
    assert_eq!(credential.expose_password(), "password  ");
    assert!(credential.expires_on().is_none());
}

/// Scenario: Either credential file is empty or contains only line endings.
/// Guarantees: Acquisition rejects the incomplete pair without disclosing the other value.
#[tokio::test]
async fn rejects_empty_values_without_disclosure() {
    let fixture = Fixture::new();
    for path in [
        &fixture.config.username_file,
        &fixture.config.password_secret_file,
    ] {
        for contents in ["", "\r\n"] {
            fixture.write_pair("sensitive-username", "sensitive-password");
            std::fs::write(path, contents).expect("write empty value");
            let error = fixture
                .source()
                .fetch()
                .await
                .expect_err("empty value rejected");
            assert!(matches!(error, Error::CredentialAcquisition { .. }));
            let diagnostic = format!("{error:?} {error}");
            assert!(!diagnostic.contains("sensitive-username"));
            assert!(!diagnostic.contains("sensitive-password"));
        }
    }
}

/// Scenario: Either credential file contains invalid UTF-8.
/// Guarantees: Acquisition identifies the offending field and never returns a partial pair.
#[tokio::test]
async fn rejects_invalid_utf8_in_either_file() {
    let fixture = Fixture::new();
    for (field, path) in [
        ("username_file", &fixture.config.username_file),
        ("password_secret_file", &fixture.config.password_secret_file),
    ] {
        fixture.write_pair("user-1", "password-1");
        std::fs::write(path, [0xff, 0xfe]).expect("write invalid UTF-8");
        let error = fixture
            .source()
            .fetch()
            .await
            .expect_err("invalid UTF-8 rejected");
        assert!(matches!(error, Error::InvalidUtf8 { .. }));
        assert!(error.to_string().contains(field));
        assert!(!error.to_string().contains("password-1"));
    }
}

/// Scenario: Either credential file is absent or its path points to a directory.
/// Guarantees: Acquisition identifies the failed source instead of supplying a fallback.
#[tokio::test]
async fn rejects_unreadable_files() {
    for username in [true, false] {
        let fixture = Fixture::new();
        let path = if username {
            &fixture.config.username_file
        } else {
            &fixture.config.password_secret_file
        };
        let field = if username {
            "username_file"
        } else {
            "password_secret_file"
        };
        std::fs::remove_file(path).expect("remove fixture file");
        for directory in [false, true] {
            if directory {
                std::fs::create_dir(path).expect("replace file with directory");
            }
            let error = fixture.source().fetch().await.expect_err("read must fail");
            assert!(matches!(error, Error::ReadCredentialFile { .. }));
            assert!(error.to_string().contains(field));
            assert!(error.to_string().contains(&path.display().to_string()));
        }
    }
}

/// Scenario: Either credential file exceeds the shared four-megabyte limit.
/// Guarantees: Pair acquisition rejects oversized inputs through the bounded reader.
#[tokio::test]
async fn rejects_oversized_files() {
    let fixture = Fixture::new();
    for path in [
        &fixture.config.username_file,
        &fixture.config.password_secret_file,
    ] {
        fixture.write_pair("user-1", "password-1");
        std::fs::write(path, vec![b'x'; 5 * 1024 * 1024]).expect("write oversized file");
        let error = fixture
            .source()
            .fetch()
            .await
            .expect_err("size limit enforced");
        assert!(matches!(error, Error::ReadCredentialFile { .. }));
        assert!(error.to_string().contains("too large"));
    }
}

/// Scenario: Username, password, then both files change between acquisitions.
/// Guarantees: Each acquisition reads both current values rather than retaining one field.
#[tokio::test]
async fn source_rereads_both_files() {
    let fixture = Fixture::new();
    let source = fixture.source();
    for (username, password) in [
        ("user-2", "password-1"),
        ("user-2", "password-2"),
        ("user-3", "password-3"),
    ] {
        fixture.write_pair(username, password);
        let credential = source.fetch().await.expect("updated pair");
        assert_eq!(credential.expose_username(), username);
        assert_eq!(credential.expose_password(), password);
    }
}

/// Scenario: A successful credential is formatted for diagnostics.
/// Guarantees: Both file-derived values are redacted.
#[tokio::test]
async fn credential_debug_redacts_both_values() {
    let fixture = Fixture::new();
    fixture.write_pair("sensitive-username", "sensitive-password");
    let credential = fixture.source().fetch().await.expect("credential");
    let diagnostic = format!("{credential:?}");
    assert!(!diagnostic.contains("sensitive-username"));
    assert!(!diagnostic.contains("sensitive-password"));
}

/// Scenario: Subscribers attach before and after initial acquisition, then a file changes.
/// Guarantees: Independent streams receive the cached pair immediately without re-reading files.
#[tokio::test]
async fn streams_deliver_current_cached_pair() {
    let fixture = Fixture::new();
    let extension = fixture.extension();
    let mut early = extension.credential_stream();
    assert!(early.next().now_or_never().is_none());
    let initial = extension
        .get_credential()
        .await
        .expect("initial acquisition");
    fixture.write_pair("user-2", "password-2");
    let mut late = extension.credential_stream();
    for stream in [&mut early, &mut late] {
        let credential = stream.next().await.expect("current credential");
        assert_eq!(credential.expose_username(), initial.expose_username());
        assert_eq!(credential.expose_password(), initial.expose_password());
        assert!(stream.next().now_or_never().is_none());
    }
    let cached = extension
        .get_credential()
        .await
        .expect("cached acquisition");
    assert_eq!(cached.expose_username(), "user-1");
    assert_eq!(cached.expose_password(), "password-1");
}

/// Scenario: Initial acquisition fails because either credential file is missing.
/// Guarantees: The provider returns a capability error and does not publish any credential.
#[tokio::test]
async fn initial_failure_does_not_publish() {
    for username in [true, false] {
        let fixture = Fixture::new();
        let path = if username {
            &fixture.config.username_file
        } else {
            &fixture.config.password_secret_file
        };
        std::fs::remove_file(path).expect("remove credential file");
        let extension = fixture.extension();
        let mut stream = extension.credential_stream();
        let error = extension
            .get_credential()
            .await
            .expect_err("acquisition failed");
        assert!(error.to_string().contains("test-ext"));
        assert!(stream.next().now_or_never().is_none());
    }
}

/// Scenario: The polling deadline passes after both credential files rotate.
/// Guarantees: The background task publishes a complete replacement only at the polling interval.
#[tokio::test(start_paused = true)]
async fn background_refresh_publishes_pair() {
    let fixture = Fixture::new();
    let extension = fixture.extension();
    let mut stream = extension.credential_stream();
    let (control_tx, control_rx) = tokio::sync::mpsc::channel(1);
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let control = ControlChannel::new(SharedReceiver::mpsc(control_rx), shutdown_rx);
    let effect = otel_arrow_dfe_engine::testing::test_extension_effect_handler("test-ext".into());
    // Exercise the existing shared-extension task contract, not data-path work.
    let task = tokio::spawn(Box::new(extension).start(control, effect));

    let initial = stream.next().await.expect("initial pair published");
    assert_eq!(initial.expose_username(), "user-1");
    assert_eq!(initial.expose_password(), "password-1");
    fixture.write_pair("user-2", "password-2");
    tokio::time::advance(Duration::from_secs(299)).await;
    assert!(stream.next().now_or_never().is_none());
    tokio::time::advance(Duration::from_secs(1)).await;
    let rotated = stream.next().await.expect("rotated pair published");
    assert_eq!(rotated.expose_username(), "user-2");
    assert_eq!(rotated.expose_password(), "password-2");

    drop(control_tx);
    let _ = task.await.expect("task joins").expect("clean shutdown");
}

struct ObservedSource {
    source: FlatFileSaslAuth,
    failures: tokio::sync::mpsc::Sender<()>,
}

#[async_trait]
impl BackgroundProviderSource<SaslCredential> for ObservedSource {
    type Error = Error;

    async fn fetch(&self) -> Result<SaslCredential, Error> {
        self.source.fetch().await
    }

    fn expires_on(value: &SaslCredential) -> Option<Instant> {
        value.expires_on()
    }

    fn log_refresh_failure(&self, error: &Error) {
        self.source.log_refresh_failure(error);
        self.failures
            .try_send(())
            .expect("observe completed failure");
    }
}

/// Scenario: Either file fails during refresh while the other has already rotated, then recovers.
/// Guarantees: No partial pair is published, the cache survives, and retry publishes the recovered pair.
#[tokio::test(start_paused = true)]
async fn failed_refresh_preserves_pair_and_recovers() {
    for username in [true, false] {
        let fixture = Fixture::new();
        let (failure_tx, mut failure_rx) = tokio::sync::mpsc::channel(1);
        let extension = make_provider(
            ObservedSource {
                source: fixture.source(),
                failures: failure_tx,
            },
            fixture.config.credentials_file_refresh,
        );
        let mut updates = extension.subscribe();
        let initial = extension.get_value().await.expect("initial pair");
        let _ = updates.borrow_and_update();
        let cached_provider = extension.clone();
        let (control_tx, control_rx) = tokio::sync::mpsc::channel(1);
        let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
        let control = ControlChannel::new(SharedReceiver::mpsc(control_rx), shutdown_rx);
        let effect =
            otel_arrow_dfe_engine::testing::test_extension_effect_handler("test-ext".into());
        fixture.write_pair("user-2", "password-2");
        let failed_path = if username {
            &fixture.config.username_file
        } else {
            &fixture.config.password_secret_file
        };
        std::fs::write(failed_path, [0xff]).expect("invalidate one file");
        // The observer reports real source failures after the shared task has handled them.
        let task = tokio::spawn(Box::new(extension).start(control, effect));
        failure_rx.recv().await.expect("refresh failure observed");
        assert!(!updates.has_changed().expect("channel open"));
        let cached = cached_provider
            .get_value()
            .await
            .expect("last good pair retained");
        assert_eq!(cached.expose_username(), initial.expose_username());
        assert_eq!(cached.expose_password(), initial.expose_password());

        fixture.write_pair("user-2", "password-2");
        updates.changed().await.expect("retry publishes pair");
        let recovered = updates.borrow_and_update().clone().expect("recovered pair");
        assert_eq!(recovered.expose_username(), "user-2");
        assert_eq!(recovered.expose_password(), "password-2");
        assert!(failure_rx.try_recv().is_err());
        drop(control_tx);
        let _ = task.await.expect("task joins").expect("clean shutdown");
    }
}
