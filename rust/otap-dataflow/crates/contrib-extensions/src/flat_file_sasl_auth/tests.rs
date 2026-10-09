// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration, capability wiring, and refresh tests for file-backed SASL credentials.

use std::collections::{HashMap, HashSet};
use std::future::Future;
use std::path::Path;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_trait::async_trait;
use futures::{FutureExt, StreamExt};
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_config::extension::ExtensionUserConfig;
use otel_arrow_dfe_engine::capability::auth::SaslCredential;
use otel_arrow_dfe_engine::capability::auth::sasl_credential_provider::SaslCredentialProvider;
use otel_arrow_dfe_engine::capability::registry::CapabilityRegistry;
use otel_arrow_dfe_engine::config::ExtensionConfig;
use otel_arrow_dfe_engine::control::{ExtensionControlMsg, ShutdownPayload};
use otel_arrow_dfe_engine::extension::ExtensionBundle;
use otel_arrow_dfe_engine::extension::wrapper::{ExtensionLifecycle, ExtensionVariant};
use otel_arrow_dfe_engine::shared::capability::auth::sasl_credential_provider::SaslCredentialProvider as SharedSaslCredentialProvider;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension as SharedExtension};
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_engine::terminal_state::TerminalState;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use secrecy::ExposeSecret;
use tempfile::TempDir;
use tokio::sync::{mpsc, oneshot, watch};
use tokio::task::JoinHandle;
use tokio_stream::wrappers::WatchStream;

use super::auth::FlatFileSaslAuth;
use super::config::Config;
use super::error::Error;
use super::metrics::FlatFileSaslAuthMetrics;
use super::{
    DEFAULT_SASL_CREDENTIAL_REFRESH_INTERVAL, FLAT_FILE_SASL_AUTH_EXTENSION,
    FLAT_FILE_SASL_AUTH_URN, FlatFileSaslAuthExtension, create, parse_config, validate_config,
};
use crate::common::background_refresh::{
    BackgroundProviderExtension, BackgroundProviderMetricsTracker, BackgroundProviderRefreshPolicy,
    BackgroundProviderSource,
};

const TEST_USERNAME: &str = "configured-sasl-user";
const REFRESH_INTERVAL: Duration = Duration::from_secs(60);

fn test_directory() -> TempDir {
    tempfile::tempdir_in(".").expect("test directory created in the workspace")
}

fn config_for(path: &Path) -> Config {
    Config {
        username: TEST_USERNAME.into(),
        password_secret_file: path.to_owned(),
        password_secret_file_refresh: REFRESH_INTERVAL,
    }
}

fn config_json(path: &Path) -> serde_json::Value {
    serde_json::json!({
        "username": TEST_USERNAME,
        "password_secret_file": path,
    })
}

fn make_tracker() -> BackgroundProviderMetricsTracker<FlatFileSaslAuthMetrics> {
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<FlatFileSaslAuthMetrics>(EmptyAttributes());
    BackgroundProviderMetricsTracker::new(metric_set)
}

fn make_extension(config: Config) -> FlatFileSaslAuthExtension {
    let policy = BackgroundProviderRefreshPolicy::periodic(config.password_secret_file_refresh)
        .expect("valid refresh interval");
    let (tx, _rx) = watch::channel(None);
    FlatFileSaslAuthExtension::new(
        "sasl-test",
        FlatFileSaslAuth::new(config),
        policy,
        tx,
        make_tracker(),
    )
}

fn create_bundle(config: serde_json::Value) -> Result<ExtensionBundle, ConfigError> {
    let (context, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name = "sasl-test".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        FLAT_FILE_SASL_AUTH_URN.into(),
        config,
    ));
    create(
        &context,
        name,
        user_config,
        &ExtensionConfig::new("sasl-test"),
    )
}

/// Keep the paused clock stationary while the filesystem's blocking pool finishes.
/// A Tokio timeout alone can auto-advance past a refresh before disk I/O completes.
async fn observe<F: Future>(future: F) -> F::Output {
    tokio::pin!(future);
    let started = Instant::now();
    loop {
        tokio::select! {
            biased;
            output = &mut future => return output,
            _ = tokio::task::yield_now() => {
                assert!(started.elapsed() < Duration::from_secs(10), "observation stalled");
            }
        }
    }
}

struct RunningExtension {
    // The control sender stays alive until priority shutdown is delivered.
    _control_tx: mpsc::Sender<ExtensionControlMsg>,
    shutdown_tx: oneshot::Sender<ShutdownPayload>,
    task: JoinHandle<Result<TerminalState, otel_arrow_dfe_engine::error::Error>>,
}

fn start_extension<E: SharedExtension + 'static>(extension: E) -> RunningExtension {
    let (control_tx, control_rx) = mpsc::channel(1);
    let (shutdown_tx, shutdown_rx) = oneshot::channel();
    let control = ControlChannel::new(SharedReceiver::mpsc(control_rx), shutdown_rx);
    let handler = otel_arrow_dfe_engine::testing::test_extension_effect_handler("sasl-test".into());
    // This is a Shared extension; spawning its Send lifecycle future exercises that contract.
    let task = tokio::spawn(Box::new(extension).start(control, handler));
    RunningExtension {
        _control_tx: control_tx,
        shutdown_tx,
        task,
    }
}

impl RunningExtension {
    async fn shutdown(self) -> TerminalState {
        let deadline = Instant::now() + Duration::from_secs(1);
        self.shutdown_tx
            .send(ShutdownPayload {
                deadline,
                reason: "SASL test finished".to_owned(),
            })
            .expect("extension accepts priority shutdown");
        let terminal = observe(self.task)
            .await
            .expect("extension task joins")
            .expect("extension exits cleanly");
        assert_eq!(terminal.deadline(), deadline);
        assert_eq!(
            terminal.metrics().len(),
            1,
            "final auth metrics are returned"
        );
        terminal
    }
}

/// Scenario: The required SASL username and password file omit a refresh interval.
/// Guarantees: Parsing preserves both fields and applies the one-hour periodic default.
#[test]
fn config_defaults_and_explicit_interval() {
    let path = Path::new("sasl-password");
    let config = parse_config(&config_json(path)).expect("valid config");
    assert_eq!(config.username.expose_secret(), TEST_USERNAME);
    assert_eq!(config.password_secret_file, path);
    assert_eq!(
        config.password_secret_file_refresh,
        DEFAULT_SASL_CREDENTIAL_REFRESH_INTERVAL
    );
    assert_eq!(
        DEFAULT_SASL_CREDENTIAL_REFRESH_INTERVAL,
        Duration::from_secs(3600)
    );

    let mut value = config_json(path);
    value["password_secret_file_refresh"] = "23s".into();
    let config = parse_config(&value).expect("explicit interval is valid");
    assert_eq!(config.password_secret_file_refresh, Duration::from_secs(23));
    validate_config(&value).expect("factory validation accepts the same config");
}

/// Scenario: Required fields are absent, empty, or have the wrong JSON type.
/// Guarantees: Parsing and the factory validation hook reject every invalid configuration.
#[test]
fn config_rejects_missing_empty_and_mistyped_fields() {
    for value in [
        serde_json::json!({}),
        serde_json::json!({"password_secret_file": "password"}),
        serde_json::json!({"username": TEST_USERNAME}),
        serde_json::json!({"username": "", "password_secret_file": "password"}),
        serde_json::json!({"username": TEST_USERNAME, "password_secret_file": ""}),
        serde_json::json!({"username": null, "password_secret_file": "password"}),
        serde_json::json!({"username": TEST_USERNAME, "password_secret_file": 42}),
    ] {
        assert!(
            matches!(
                parse_config(&value),
                Err(ConfigError::InvalidUserConfig { .. })
            ),
            "invalid config accepted: {value}"
        );
        assert!(validate_config(&value).is_err());
    }
    let error = parse_config(&serde_json::json!({
        "username": TEST_USERNAME,
        "password_secret_file": "",
    }))
    .expect_err("empty path is invalid");
    assert!(error.to_string().contains("password_secret_file"));
}

/// Scenario: Config includes unknown fields or removed username-file, inline-password, and AKV forms.
/// Guarantees: Unsupported secret sources fail validation instead of being silently ignored.
#[test]
fn config_rejects_unknown_and_removed_fields() {
    for field in [
        "unexpected",
        "username_file",
        "username_secret_file",
        "username_secret_file_refresh",
        "password_secret",
        "password",
        "key_vault",
        "azure_key_vault",
    ] {
        let mut value = config_json(Path::new("password"));
        value[field] = "unsupported-secret-value".into();
        let error = validate_config(&value).expect_err("unknown field rejected");
        assert!(error.to_string().contains(field));
        assert!(!error.to_string().contains("unsupported-secret-value"));
    }
}

/// Scenario: Refresh intervals lie on the inclusive bounds or one nanosecond beyond them.
/// Guarantees: Parsing and direct validation accept only ten seconds through 365 days.
#[test]
fn config_refresh_interval_has_exact_inclusive_bounds() {
    let minimum = Duration::from_secs(10);
    let maximum = Duration::from_secs(365 * 24 * 60 * 60);
    for (interval, text, valid) in [
        (minimum, "10s", true),
        (maximum, "365days", true),
        (minimum - Duration::from_nanos(1), "9s 999999999ns", false),
        (maximum + Duration::from_nanos(1), "365days 1ns", false),
        (Duration::ZERO, "0s", false),
    ] {
        let mut config = config_for(Path::new("password"));
        config.password_secret_file_refresh = interval;
        assert_eq!(config.validate().is_ok(), valid, "interval: {text}");
        let mut value = config_json(Path::new("password"));
        value["password_secret_file_refresh"] = text.into();
        assert_eq!(parse_config(&value).is_ok(), valid, "interval: {text}");
    }
    let mut value = config_json(Path::new("password"));
    value["password_secret_file_refresh"] = "not-a-duration".into();
    assert!(parse_config(&value).is_err());
}

/// Scenario: The distributed factory builds an extension before its password file exists.
/// Guarantees: The exact URN registers Active+Shared SASL capabilities and an initially unsignaled readiness pair.
#[test]
fn factory_registers_typed_shared_capability_and_readiness() {
    assert_eq!(
        FLAT_FILE_SASL_AUTH_URN,
        "urn:otel:extension:flat_file_sasl_auth"
    );
    assert_eq!(FLAT_FILE_SASL_AUTH_EXTENSION.name, FLAT_FILE_SASL_AUTH_URN);
    assert!(
        otel_arrow_dfe_otap::OTAP_EXTENSION_FACTORIES
            .iter()
            .any(|factory| std::ptr::eq(factory, &FLAT_FILE_SASL_AUTH_EXTENSION))
    );
    let capabilities = FLAT_FILE_SASL_AUTH_EXTENSION
        .capabilities
        .as_ref()
        .expect("factory advertises capabilities");
    assert_eq!(capabilities.shared, &["sasl_credential_provider"]);
    assert!(capabilities.local.is_empty());

    let directory = test_directory();
    let path = directory.path().join("not-created");
    let bundle = create_bundle(config_json(&path)).expect("creation does not read the file");
    assert!(bundle.local().is_none());
    let shared = bundle.shared().expect("shared variant");
    assert_eq!(shared.variant(), ExtensionVariant::Shared);
    assert!(!shared.is_passive());
    let otel_arrow_dfe_engine::extension::ExtensionWrapper::Shared {
        lifecycle:
            ExtensionLifecycle::Active {
                readiness_probe,
                readiness_signaller,
                ..
            },
        ..
    } = shared
    else {
        panic!("expected active shared lifecycle");
    };
    assert_eq!(
        readiness_probe.as_ref().expect("readiness probe").timeout(),
        Duration::from_secs(5)
    );
    assert!(
        !readiness_signaller
            .as_ref()
            .expect("readiness signaller")
            .is_ready()
    );

    let mut registry = CapabilityRegistry::new();
    bundle
        .register_into(Some(capabilities), &mut registry)
        .expect("capability registered");
    let bindings = HashMap::from([("sasl_credential_provider".into(), "sasl-test".into())]);
    let known_extensions = HashSet::from(["sasl-test".into()]);
    let resolved = otel_arrow_dfe_engine::testing::capability::resolve_bindings_for_test(
        &bindings,
        &registry,
        &known_extensions,
    )
    .expect("typed binding resolves");
    let provider = resolved
        .require_shared::<SaslCredentialProvider>()
        .expect("shared SASL provider constructed");
    assert!(
        provider.credential_stream().next().now_or_never().is_none(),
        "factory leaves the initial watch cache empty"
    );
    assert!(matches!(
        create_bundle(serde_json::json!({})),
        Err(ConfigError::InvalidUserConfig { .. })
    ));
}

/// Scenario: File contents have spaces, embedded line endings, and trailing CR/LF characters.
/// Guarantees: Acquisition strips only terminal CR/LF, preserves all other bytes, and assigns no expiry.
#[tokio::test]
async fn acquisition_preserves_sasl_values_and_redacts_debug() {
    let directory = test_directory();
    let path = directory.path().join("password");
    let password = "  private-password:\tinside\nline  ";
    std::fs::write(&path, format!("{password}\r\n\r\n")).expect("password written");
    let mut config = config_for(&path);
    let username = "sasl:user\twith\ncontrols";
    config.username = username.into();
    // SASL consumers, not the provider, impose mechanism-specific restrictions.
    let parsed = parse_config(&serde_json::json!({
        "username": username,
        "password_secret_file": path,
    }))
    .expect("colon and control characters are valid SASL username material");
    assert_eq!(parsed.username.expose_secret(), username);
    let config_debug = format!("{config:?}");
    assert!(!config_debug.contains("sasl:user"));
    let credential = FlatFileSaslAuth::new(config)
        .fetch()
        .await
        .expect("credential acquired");
    assert_eq!(credential.expose_username(), username);
    assert_eq!(credential.expose_password(), password);
    assert!(credential.expires_on().is_none());
    let debug = format!("{credential:?}");
    assert!(!debug.contains("sasl:user"));
    assert!(!debug.contains("private-password"));
}

fn assert_safe_file_error(error: &Error, path: &Path) {
    let display = error.to_string();
    assert!(display.contains("password_secret_file"), "{display}");
    assert!(
        display.contains(path.to_string_lossy().as_ref()),
        "{display}"
    );
    for secret in [TEST_USERNAME, "private-password"] {
        assert!(!display.contains(secret));
        assert!(!format!("{error:?}").contains(secret));
    }
}

/// Scenario: A password file is empty, contains only CR/LF, or contains invalid UTF-8.
/// Guarantees: Acquisition errors identify the password field and file path without disclosing contents.
#[tokio::test]
async fn acquisition_rejects_empty_and_invalid_utf8_with_context() {
    let directory = test_directory();
    let path = directory.path().join("password");
    for contents in [&b""[..], &b"\r\n\r\n"[..], &b"private-password\xff"[..]] {
        std::fs::write(&path, contents).expect("invalid contents written");
        let error = FlatFileSaslAuth::new(config_for(&path))
            .fetch()
            .await
            .expect_err("invalid content is rejected");
        assert!(matches!(error, Error::CredentialAcquisition { .. }));
        assert_safe_file_error(&error, &path);
    }
}

/// Scenario: Password acquisition references a missing file or a directory instead of a file.
/// Guarantees: Portable read failures preserve the path and password-field context without credentials.
#[tokio::test]
async fn acquisition_reports_unreadable_paths_without_secrets() {
    let directory = test_directory();
    for path in [
        directory.path().join("missing"),
        directory.path().to_owned(),
    ] {
        let error = FlatFileSaslAuth::new(config_for(&path))
            .fetch()
            .await
            .expect_err("unreadable password path rejected");
        assert!(matches!(error, Error::ReadCredentialFile { .. }));
        assert_safe_file_error(&error, &path);
    }
    // Oversized reads are already covered by common::secret_file::tests::rejects_oversized_file.
}

/// Scenario: Two subscribers precede acquisition, a third subscribes late, and disk changes without refresh.
/// Guarantees: SASL consumers receive independent immediate snapshots and cached get does not reread disk.
#[tokio::test]
async fn cache_and_independent_streams_share_the_last_published_snapshot() {
    let directory = test_directory();
    let path = directory.path().join("password");
    std::fs::write(&path, "first-password").expect("password written");
    let extension = make_extension(config_for(&path));
    let mut first = extension.credential_stream();
    let mut second = extension.clone().credential_stream();
    assert!(first.next().now_or_never().is_none());
    assert!(second.next().now_or_never().is_none());

    let acquired = extension
        .get_credential()
        .await
        .expect("cache miss fetches");
    assert_eq!(acquired.expose_username(), TEST_USERNAME);
    assert_eq!(acquired.expose_password(), "first-password");
    for credential in [
        first.next().await.expect("first subscriber"),
        second.next().await.expect("independent second subscriber"),
        extension
            .credential_stream()
            .next()
            .now_or_never()
            .expect("late subscription immediately resolves")
            .expect("late subscription has a snapshot"),
    ] {
        assert_eq!(credential.expose_password(), "first-password");
    }

    std::fs::write(&path, "unpublished-password").expect("file changed");
    let cached = extension.get_credential().await.expect("cached credential");
    assert_eq!(cached.expose_password(), "first-password");
    assert_eq!(
        extension
            .subscribe()
            .borrow()
            .as_ref()
            .expect("watch cache populated")
            .expose_password(),
        "first-password"
    );
    assert!(first.next().now_or_never().is_none());
    assert!(second.next().now_or_never().is_none());
}

/// Scenario: A real SASL adapter runs while the password file changes before its periodic deadline.
/// Guarantees: Both streams publish rotation only at the interval, keep the configured username, and shutdown returns metrics.
#[tokio::test(start_paused = true)]
async fn periodic_rotation_publishes_to_independent_sasl_streams() {
    let directory = test_directory();
    let path = directory.path().join("password");
    std::fs::write(&path, "password-one").expect("initial password written");
    let extension = make_extension(config_for(&path));
    let consumer = extension.clone();
    let mut first = consumer.credential_stream();
    let mut second = consumer.credential_stream();
    let running = start_extension(extension);
    let initial = observe(first.next()).await.expect("initial publication");
    assert_eq!(initial.expose_username(), TEST_USERNAME);
    assert_eq!(initial.expose_password(), "password-one");
    assert_eq!(
        observe(second.next())
            .await
            .expect("second subscriber")
            .expose_password(),
        "password-one"
    );

    std::fs::write(&path, "password-two\r\n").expect("rotated password written");
    tokio::time::advance(REFRESH_INTERVAL - Duration::from_secs(1)).await;
    assert!(first.next().now_or_never().is_none());
    assert!(second.next().now_or_never().is_none());
    assert_eq!(
        consumer
            .get_credential()
            .await
            .expect("cached value")
            .expose_password(),
        "password-one"
    );
    tokio::time::advance(Duration::from_secs(1)).await;
    for credential in [
        observe(first.next()).await.expect("rotated credential"),
        observe(second.next()).await.expect("independent rotation"),
    ] {
        assert_eq!(credential.expose_username(), TEST_USERNAME);
        assert_eq!(credential.expose_password(), "password-two");
        assert!(credential.expires_on().is_none());
    }
    let _terminal = running.shutdown().await;
}

/// A test-only observer delegates every acquisition to the real SASL source.
/// Its bounded channel observes completed background failures without polling disk or sleeping.
struct ObservedSource {
    source: FlatFileSaslAuth,
    failures: mpsc::Sender<()>,
}

#[async_trait]
impl BackgroundProviderSource<SaslCredential> for ObservedSource {
    type Error = Error;

    fn expires_on(value: &SaslCredential) -> Option<Instant> {
        FlatFileSaslAuth::expires_on(value)
    }

    async fn fetch(&self) -> Result<SaslCredential, Error> {
        self.source.fetch().await
    }

    fn log_refresh_failure(&self, error: &Error) {
        self.source.log_refresh_failure(error);
        self.failures
            .try_send(())
            .expect("one observed failure fits in the bounded channel");
    }
}

type ObservedExtension = BackgroundProviderExtension<
    ObservedSource,
    FlatFileSaslAuthMetrics,
    SaslCredential,
    SaslCredentialProvider,
>;

fn observed_extension(config: Config) -> (ObservedExtension, mpsc::Receiver<()>) {
    let policy = BackgroundProviderRefreshPolicy::periodic(config.password_secret_file_refresh)
        .expect("valid refresh interval");
    let (failures, receiver) = mpsc::channel(1);
    let (tx, _rx) = watch::channel(None);
    let extension = ObservedExtension::new(
        "sasl-test",
        ObservedSource {
            source: FlatFileSaslAuth::new(config),
            failures,
        },
        policy,
        tx,
        make_tracker(),
    );
    (extension, receiver)
}

/// Scenario: The initial real file acquisition fails and the password file is repaired before retry.
/// Guarantees: Failure publishes nothing, leaves subscriptions open, and the bounded first retry publishes recovery.
#[tokio::test(start_paused = true)]
async fn initial_failure_has_no_publication_and_retry_recovers() {
    let directory = test_directory();
    let path = directory.path().join("missing-password");
    let (extension, mut failures) = observed_extension(config_for(&path));
    let mut stream =
        Box::pin(WatchStream::new(extension.subscribe()).filter_map(|value| async move { value }));
    let consumer = extension.clone();
    let running = start_extension(extension);
    observe(failures.recv())
        .await
        .expect("initial fetch failed");
    assert!(consumer.subscribe().borrow().is_none());
    assert!(
        stream.next().now_or_never().is_none(),
        "stream is open but silent"
    );

    std::fs::write(&path, "recovered-password").expect("file repaired");
    // The framework jitters the first retry within 5..=10 seconds.
    // Advancing its upper bound and observing the completed I/O avoids random timing assertions.
    tokio::time::advance(Duration::from_secs(10)).await;
    let recovered = observe(stream.next()).await.expect("retry publication");
    assert_eq!(recovered.expose_username(), TEST_USERNAME);
    assert_eq!(recovered.expose_password(), "recovered-password");
    assert_eq!(
        consumer
            .get_value()
            .await
            .expect("recovered cache")
            .expose_password(),
        "recovered-password"
    );
    let _terminal = running.shutdown().await;
}

/// Scenario: A periodic refresh reads invalid UTF-8 after publishing a good credential, then the file recovers.
/// Guarantees: Failure preserves the last-good cache, emits no stream item, and retry updates the same open stream.
#[tokio::test(start_paused = true)]
async fn failed_refresh_retains_snapshot_and_open_stream_until_retry() {
    let directory = test_directory();
    let path = directory.path().join("password");
    std::fs::write(&path, "last-good-password").expect("initial password written");
    let (extension, mut failures) = observed_extension(config_for(&path));
    let consumer = extension.clone();
    let mut stream =
        Box::pin(WatchStream::new(extension.subscribe()).filter_map(|value| async move { value }));
    let running = start_extension(extension);
    let initial = observe(stream.next()).await.expect("initial publication");
    assert_eq!(initial.expose_password(), "last-good-password");

    std::fs::write(&path, b"private-password\xff").expect("invalid file written");
    tokio::time::advance(REFRESH_INTERVAL).await;
    observe(failures.recv()).await.expect("refresh failed");
    assert!(
        stream.next().now_or_never().is_none(),
        "failure neither publishes nor closes the stream"
    );
    assert_eq!(
        consumer
            .get_value()
            .await
            .expect("last good cache")
            .expose_password(),
        "last-good-password"
    );
    assert_eq!(
        consumer
            .subscribe()
            .borrow()
            .as_ref()
            .expect("cached snapshot retained")
            .expose_password(),
        "last-good-password"
    );

    std::fs::write(&path, "new-good-password").expect("file repaired");
    tokio::time::advance(Duration::from_secs(10)).await;
    let recovered = observe(stream.next()).await.expect("retry publication");
    assert_eq!(recovered.expose_username(), TEST_USERNAME);
    assert_eq!(recovered.expose_password(), "new-good-password");
    let _terminal = running.shutdown().await;
}
