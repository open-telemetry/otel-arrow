// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Azure Identity Auth extension.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Condvar, Mutex};

use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use azure_core::time::{Duration as AzureDuration, OffsetDateTime};
use futures::StreamExt;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider as SharedBearerTokenProvider;
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use tokio::sync::watch;

use super::auth::Auth;
use super::config::{AuthMethod, Config};
use super::error::Error;
use super::metrics::AzureIdentityAuthMetrics;
use super::*;
use crate::common::background_refresh::{
    BackgroundProviderMetricsTracker, BackgroundProviderSource,
};

// -- Config tests -------------------------------------------

fn config_from_json(value: serde_json::Value) -> Result<Config, ConfigError> {
    parse_config(&value)
}

#[test]
fn config_defaults_apply() {
    let cfg = config_from_json(serde_json::json!({})).expect("empty config is valid");
    assert_eq!(cfg.method, AuthMethod::ManagedIdentity);
    assert_eq!(cfg.scope, "https://monitor.azure.com/.default");
    assert!(cfg.client_id.is_none());
    assert!(cfg.tenant_id.is_none());
    assert!(cfg.token_file_path.is_none());
    assert_eq!(cfg.startup_timeout, Duration::from_secs(30));
}

#[test]
fn startup_timeout_parses_and_rejects_zero() {
    let cfg = config_from_json(serde_json::json!({ "startup_timeout": "45s" }))
        .expect("human-readable duration parses");
    assert_eq!(cfg.startup_timeout, Duration::from_secs(45));

    assert!(
        config_from_json(serde_json::json!({ "startup_timeout": "0s" })).is_err(),
        "zero startup_timeout must be rejected"
    );
}

#[test]
fn method_aliases_deserialize() {
    let cases = [
        ("msi", AuthMethod::ManagedIdentity),
        ("managed_identity", AuthMethod::ManagedIdentity),
        ("managedidentity", AuthMethod::ManagedIdentity),
        ("dev", AuthMethod::Development),
        ("developer", AuthMethod::Development),
        ("cli", AuthMethod::Development),
        ("development", AuthMethod::Development),
        ("wif", AuthMethod::WorkloadIdentity),
        ("workload_identity", AuthMethod::WorkloadIdentity),
        ("workloadidentity", AuthMethod::WorkloadIdentity),
    ];
    for (alias, expected) in cases {
        let cfg = config_from_json(serde_json::json!({ "method": alias }))
            .unwrap_or_else(|e| panic!("alias `{alias}` should deserialize: {e}"));
        assert_eq!(cfg.method, expected, "alias `{alias}`");
    }
}

#[test]
fn empty_scope_is_rejected() {
    let err = config_from_json(serde_json::json!({ "scope": "   " }))
        .expect_err("whitespace scope must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

#[test]
fn unknown_fields_are_rejected() {
    let err = config_from_json(serde_json::json!({ "bogus": true }))
        .expect_err("unknown field must be rejected");
    assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
}

#[test]
fn per_method_fields_are_validated() {
    // `tenant_id` / `token_file_path` only apply to workload_identity.
    assert!(
        config_from_json(serde_json::json!({ "method": "managed_identity", "tenant_id": "t" }))
            .is_err()
    );
    assert!(
        config_from_json(
            serde_json::json!({ "method": "development", "token_file_path": "/tmp/x" })
        )
        .is_err()
    );
    // `client_id` is not valid for developer tooling.
    assert!(
        config_from_json(serde_json::json!({ "method": "development", "client_id": "c" })).is_err()
    );
    // Valid combinations still pass.
    assert!(
        config_from_json(serde_json::json!({ "method": "managed_identity", "client_id": "c" }))
            .is_ok()
    );
    assert!(
        config_from_json(serde_json::json!({
            "method": "workload_identity",
            "tenant_id": "t",
            "token_file_path": "/tmp/x",
            "client_id": "c",
        }))
        .is_ok()
    );
}

#[test]
fn validate_config_hook_accepts_valid_config() {
    assert!(validate_config(&serde_json::json!({ "method": "managed_identity" })).is_ok());
}

#[test]
fn factory_is_registered_with_capability() {
    assert_eq!(AZURE_IDENTITY_AUTH_EXTENSION.name, AZURE_IDENTITY_AUTH_URN);
    let capabilities = AZURE_IDENTITY_AUTH_EXTENSION
        .capabilities
        .as_ref()
        .expect("active extension advertises capabilities");
    assert!(
        capabilities.shared.contains(&"bearer_token_provider"),
        "BearerTokenProvider must be advertised as a shared capability"
    );
}

/// Invokes the factory's `create` hook with `config` against a throwaway
/// extension context, mirroring how the engine wires the extension.
fn create_bundle(config: serde_json::Value) -> Result<ExtensionBundle, ConfigError> {
    let (ext_ctx, _registry) = otel_arrow_dfe_engine::testing::test_extension_ctx();
    let name: otel_arrow_dfe_config::ExtensionId = "azure-identity-auth".into();
    let user_config = Arc::new(ExtensionUserConfig::new(
        AZURE_IDENTITY_AUTH_URN.into(),
        config,
    ));
    let extension_config = ExtensionConfig::new(name.clone());
    create(&ext_ctx, name, user_config, &extension_config)
}

// Scenario: The factory's `create` hook runs against a valid managed-identity config.
// Guarantees: Wiring succeeds and yields a shared, active extension bundle usable by the engine.
#[test]
fn create_builds_a_shared_active_bundle() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let bundle = create_bundle(serde_json::json!({ "method": "managed_identity" }))
        .expect("a valid config wires successfully");
    assert!(
        bundle.local().is_none(),
        "the Azure identity auth extension has no local variant"
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
    let Err(err) = create_bundle(serde_json::json!({ "method": "development", "client_id": "c" }))
    else {
        panic!("client_id is not valid for the development method");
    };
    assert!(
        matches!(err, ConfigError::InvalidUserConfig { .. }),
        "expected InvalidUserConfig, got {err:?}"
    );
}

// -- Credential construction tests ------------------------------

/// Scenario: System-assigned managed identity is configured during extension construction.
/// Guarantees: Auth setup succeeds without performing SDK environment detection on this thread.
#[test]
fn managed_identity_system_assigned_auth_defers_credential_construction() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({ "method": "managed_identity" })).unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

/// Scenario: User-assigned managed identity is configured during extension construction.
/// Guarantees: The client ID is accepted while SDK credential construction remains deferred.
#[test]
fn managed_identity_user_assigned_auth_defers_credential_construction() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({
        "method": "managed_identity",
        "client_id": "00000000-0000-0000-0000-000000000000",
    }))
    .unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

#[test]
fn development_credential_constructs() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({ "method": "development" })).unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

#[test]
fn workload_identity_credential_construct_is_attempted() {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({
        "method": "workload_identity",
        "client_id": "test-client",
        "tenant_id": "test-tenant",
        "token_file_path": "/tmp/does-not-exist",
    }))
    .unwrap();
    // Construction only validates configuration; a missing env/file surfaces as
    // a CreateCredential error. Both outcomes are acceptable here.
    match Auth::new(&cfg) {
        Ok(_) => {}
        Err(Error::CreateCredential { method, .. }) => {
            assert_eq!(method, AuthMethod::WorkloadIdentity);
        }
        Err(other) => panic!("unexpected error: {other:?}"),
    }
}

// -- Token acquisition / cache tests ---------------------------

#[derive(Debug)]
struct MockCredential {
    token: String,
    expires_in: AzureDuration,
    call_count: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl TokenCredential for MockCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        let _ = self.call_count.fetch_add(1, Ordering::SeqCst);
        Ok(AccessToken {
            token: self.token.clone().into(),
            expires_on: OffsetDateTime::now_utc() + self.expires_in,
        })
    }
}

#[derive(Debug)]
struct FailingCredential {
    call_count: Arc<AtomicUsize>,
}

#[derive(Debug, Default)]
struct BlockingGate {
    started: tokio::sync::Notify,
    released: Mutex<bool>,
    released_changed: Condvar,
}

impl BlockingGate {
    fn block(&self) {
        self.started.notify_one();
        let released = self.released.lock().expect("gate lock");
        let (released, _) = self
            .released_changed
            .wait_timeout_while(released, Duration::from_secs(10), |released| !*released)
            .expect("gate wait");
        let was_released = *released;
        drop(released);
        assert!(was_released, "blocking acquisition was not released");
    }

    fn release(&self) {
        *self.released.lock().expect("gate lock") = true;
        self.released_changed.notify_all();
    }

    async fn wait_until_started(&self) {
        tokio::time::timeout(Duration::from_secs(10), self.started.notified())
            .await
            .expect("blocking acquisition must start");
    }
}

struct ReleaseBlockingGate(Arc<BlockingGate>);

impl Drop for ReleaseBlockingGate {
    fn drop(&mut self) {
        self.0.release();
    }
}

#[derive(Debug)]
struct BlockingCredential {
    gate: Arc<BlockingGate>,
    calls: Arc<AtomicUsize>,
}

#[async_trait::async_trait]
impl TokenCredential for BlockingCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        let _ = self.calls.fetch_add(1, Ordering::SeqCst);
        self.gate.block();
        Ok(AccessToken {
            token: "blocking-token".to_string().into(),
            expires_on: OffsetDateTime::now_utc() + AzureDuration::minutes(60),
        })
    }
}

async fn assert_pipeline_executor_progresses(auth: Auth, gate: Arc<BlockingGate>) {
    let _release_on_failure = ReleaseBlockingGate(Arc::clone(&gate));
    let fetch = tokio::spawn(async move { auth.fetch().await });

    gate.wait_until_started().await;
    tokio::time::timeout(
        Duration::from_secs(10),
        tokio::time::sleep(Duration::from_millis(1)),
    )
    .await
    .expect("an unrelated timer must progress while managed identity work is blocked");

    gate.release();
    let _ = tokio::time::timeout(Duration::from_secs(10), fetch)
        .await
        .expect("fetch must finish after release")
        .expect("fetch task")
        .expect("managed identity token");
}

#[async_trait::async_trait]
impl TokenCredential for FailingCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        let _ = self.call_count.fetch_add(1, Ordering::SeqCst);
        Err(azure_core::error::Error::with_message(
            azure_core::error::ErrorKind::Credential,
            "mock credential failure",
        ))
    }
}

fn make_extension(credential: Arc<dyn TokenCredential>) -> AzureIdentityAuthExtension {
    let auth = Auth::from_credential(credential, "test_scope".to_string());
    let (tx, _rx) = watch::channel(None);
    AzureIdentityAuthExtension::new(
        "test-ext",
        auth,
        BackgroundProviderRefreshPolicy::expiry_driven(
            TOKEN_USABLE_MARGIN,
            NON_EXPIRING_TOKEN_REFRESH_INTERVAL,
            Duration::from_secs(TOKEN_EXPIRY_BUFFER_SECS),
        )
        .expect("valid refresh_policy"),
        tx,
        make_tracker(),
    )
}

/// Scenario: Managed-identity credential construction performs synchronous blocking work.
/// Guarantees: Construction runs outside the single-threaded pipeline executor.
#[tokio::test(flavor = "current_thread")]
async fn managed_identity_credential_construction_is_isolated() {
    let gate = Arc::new(BlockingGate::default());
    let factory_gate = Arc::clone(&gate);
    let auth = Auth::from_managed_identity_factory(
        move || {
            factory_gate.block();
            Ok(Arc::new(MockCredential {
                token: "constructed-token".to_string(),
                expires_in: AzureDuration::minutes(60),
                call_count: Arc::new(AtomicUsize::new(0)),
            }))
        },
        "test_scope".to_string(),
    );

    assert_pipeline_executor_progresses(auth, gate).await;
}

/// Scenario: A managed-identity credential performs synchronous work while its future is polled.
/// Guarantees: Token acquisition runs outside the single-threaded pipeline executor.
#[tokio::test(flavor = "current_thread")]
async fn managed_identity_token_acquisition_is_isolated() {
    let gate = Arc::new(BlockingGate::default());
    let credential_gate = Arc::clone(&gate);
    let calls = Arc::new(AtomicUsize::new(0));
    let auth = Auth::from_managed_identity_factory(
        move || {
            Ok(Arc::new(BlockingCredential {
                gate: Arc::clone(&credential_gate),
                calls: Arc::clone(&calls),
            }))
        },
        "test_scope".to_string(),
    );

    assert_pipeline_executor_progresses(auth, gate).await;
}

/// One completed acquisition: its call index and how many others were active when it began.
type CompletedAcquisition = (usize, usize);

/// Blocks only its first acquisition and reports each completion and any overlap.
#[derive(Debug)]
struct SerializedCredential {
    gate: Arc<BlockingGate>,
    calls: Arc<AtomicUsize>,
    active: AtomicUsize,
    completed: tokio::sync::mpsc::UnboundedSender<CompletedAcquisition>,
}

#[async_trait::async_trait]
impl TokenCredential for SerializedCredential {
    async fn get_token(
        &self,
        _scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        let call = self.calls.fetch_add(1, Ordering::SeqCst);
        let overlapping = self.active.fetch_add(1, Ordering::SeqCst);
        if call == 0 {
            self.gate.block();
        }
        let _ = self.active.fetch_sub(1, Ordering::SeqCst);
        let _ = self.completed.send((call, overlapping));
        Ok(AccessToken {
            token: format!("serialized-token-{call}").into(),
            expires_on: OffsetDateTime::now_utc() + AzureDuration::minutes(60),
        })
    }
}

/// Scenario: A caller is cancelled while its managed-identity acquisition is blocked, then a
/// replacement caller requests a token before the blocked task is released.
/// Guarantees: Cancellation completes while the blocking task keeps admission, the replacement
/// cannot start until that task finishes, the cached credential is reused, and the runtime is
/// dropped only after every blocking acquisition has finished.
#[test]
fn cancelled_managed_identity_fetch_retains_admission_until_task_finishes() {
    let runtime = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("pipeline runtime");
    let gate = Arc::new(BlockingGate::default());
    // Declared after the runtime so a failing assertion releases the task before runtime drop.
    let _release_on_failure = ReleaseBlockingGate(Arc::clone(&gate));
    let credential_gate = Arc::clone(&gate);
    let calls = Arc::new(AtomicUsize::new(0));
    let credential_calls = Arc::clone(&calls);
    let constructions = Arc::new(AtomicUsize::new(0));
    let factory_constructions = Arc::clone(&constructions);
    let (completed_tx, mut completed) = tokio::sync::mpsc::unbounded_channel();
    let auth = Auth::from_managed_identity_factory(
        move || {
            let _ = factory_constructions.fetch_add(1, Ordering::SeqCst);
            Ok(Arc::new(SerializedCredential {
                gate: Arc::clone(&credential_gate),
                calls: Arc::clone(&credential_calls),
                active: AtomicUsize::new(0),
                completed: completed_tx.clone(),
            }))
        },
        "test_scope".to_string(),
    );
    let wait = Duration::from_secs(10);

    runtime.block_on(async {
        let first_auth = auth.clone();
        let first = tokio::spawn(async move { first_auth.fetch().await });
        gate.wait_until_started().await;
        first.abort();
        let cancelled = tokio::time::timeout(wait, first)
            .await
            .expect("cancellation must complete while the blocking task is still running")
            .expect_err("first fetch was cancelled");
        assert!(cancelled.is_cancelled());
        assert!(
            !auth.managed_identity_admission_available(),
            "the blocking task must keep admission after its caller is cancelled"
        );

        let second = auth.fetch();
        tokio::pin!(second);
        assert!(
            futures::poll!(&mut second).is_pending(),
            "the replacement must wait for admission"
        );
        assert_eq!(calls.load(Ordering::SeqCst), 1);

        gate.release();
        let first_completion = tokio::time::timeout(wait, completed.recv())
            .await
            .expect("the released blocking task must finish")
            .expect("completion channel open");
        assert_eq!(first_completion, (0, 0));

        let token = tokio::time::timeout(wait, second)
            .await
            .expect("the replacement must finish after admission is released")
            .expect("replacement token");
        assert_eq!(token.expose_token(), "serialized-token-1");
        let second_completion = completed.try_recv().expect("replacement completion");
        assert_eq!(
            second_completion,
            (1, 0),
            "the replacement must not start while the cancelled caller's task is running"
        );
        assert!(auth.managed_identity_admission_available());
    });

    assert_eq!(calls.load(Ordering::SeqCst), 2);
    assert_eq!(constructions.load(Ordering::SeqCst), 1);
    drop(runtime);
}

/// Scenario: Managed-identity credential construction panics in a blocking task.
/// Guarantees: The panic is reported as a task error and admission is released for a later attempt.
#[tokio::test(flavor = "current_thread")]
async fn managed_identity_task_panic_releases_admission() {
    let constructions = Arc::new(AtomicUsize::new(0));
    let factory_constructions = Arc::clone(&constructions);
    let auth = Auth::from_managed_identity_factory(
        move || {
            assert_ne!(
                factory_constructions.fetch_add(1, Ordering::SeqCst),
                0,
                "injected credential construction panic"
            );
            Ok(Arc::new(MockCredential {
                token: "recovered-token".to_string(),
                expires_in: AzureDuration::minutes(60),
                call_count: Arc::new(AtomicUsize::new(0)),
            }))
        },
        "test_scope".to_string(),
    );
    let error = auth.fetch().await.expect_err("task panic must be reported");
    assert!(matches!(error, Error::ManagedIdentityTask { source } if source.is_panic()));
    let token = tokio::time::timeout(Duration::from_secs(10), auth.fetch())
        .await
        .expect("admission must be released after a panic")
        .expect("next acquisition succeeds");
    assert_eq!(token.expose_token(), "recovered-token");
    assert_eq!(constructions.load(Ordering::SeqCst), 2);
}

/// Scenario: Managed-identity construction fails once, then token acquisition fails twice.
/// Guarantees: SDK errors propagate, construction is retried, and the constructed credential is reused.
#[tokio::test(flavor = "current_thread")]
async fn managed_identity_errors_preserve_credential_cache() {
    let constructions = Arc::new(AtomicUsize::new(0));
    let factory_constructions = Arc::clone(&constructions);
    let calls = Arc::new(AtomicUsize::new(0));
    let credential_calls = Arc::clone(&calls);
    let auth = Auth::from_managed_identity_factory(
        move || {
            if factory_constructions.fetch_add(1, Ordering::SeqCst) == 0 {
                return Err(Error::CreateCredential {
                    method: AuthMethod::ManagedIdentity,
                    source: azure_core::error::Error::with_message(
                        azure_core::error::ErrorKind::Credential,
                        "mock construction failure",
                    ),
                });
            }
            Ok(Arc::new(FailingCredential {
                call_count: Arc::clone(&credential_calls),
            }))
        },
        "test_scope".to_string(),
    );

    assert!(matches!(
        auth.fetch().await,
        Err(Error::CreateCredential { .. })
    ));
    for _ in 0..2 {
        assert!(matches!(
            auth.fetch().await,
            Err(Error::TokenAcquisition { .. })
        ));
    }
    assert_eq!(constructions.load(Ordering::SeqCst), 2);
    assert_eq!(calls.load(Ordering::SeqCst), 2);
}

#[derive(Debug)]
struct RuntimeCredential {
    runtime_id: tokio::runtime::Id,
}

#[async_trait::async_trait]
impl TokenCredential for RuntimeCredential {
    async fn get_token(
        &self,
        scopes: &[&str],
        _options: Option<TokenRequestOptions<'_>>,
    ) -> azure_core::Result<AccessToken> {
        assert_eq!(tokio::runtime::Handle::current().id(), self.runtime_id);
        assert_eq!(scopes, &["test_scope"]);
        tokio::time::sleep(Duration::from_millis(1)).await;
        Ok(AccessToken {
            token: "runtime-token".to_string().into(),
            expires_on: OffsetDateTime::now_utc() + AzureDuration::minutes(60),
        })
    }
}

/// Scenario: Managed-identity construction and async acquisition require Tokio runtime services.
/// Guarantees: Both use the pipeline runtime and its timer driver progresses the SDK future.
#[tokio::test(flavor = "current_thread")]
async fn managed_identity_uses_pipeline_runtime() {
    let runtime_id = tokio::runtime::Handle::current().id();
    let auth = Auth::from_managed_identity_factory(
        move || {
            assert_eq!(tokio::runtime::Handle::current().id(), runtime_id);
            Ok(Arc::new(RuntimeCredential { runtime_id }))
        },
        "test_scope".to_string(),
    );

    let token = tokio::time::timeout(Duration::from_secs(10), auth.fetch())
        .await
        .expect("SDK timer must progress on the pipeline runtime")
        .expect("token acquisition");
    assert_eq!(token.expose_token(), "runtime-token");
}

fn make_tracker() -> BackgroundProviderMetricsTracker<AzureIdentityAuthMetrics> {
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<AzureIdentityAuthMetrics>(EmptyAttributes());
    BackgroundProviderMetricsTracker::new(metric_set)
}

#[tokio::test]
async fn get_token_slow_path_then_fast_path_caches() {
    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "tok".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    let first = ext.get_token().await.expect("first acquisition");
    assert_eq!(first.expose_token(), "tok");
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // Fresh cached token is returned without another credential call.
    let second = ext.get_token().await.expect("cached acquisition");
    assert_eq!(second.expose_token(), "tok");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "fast path must not re-fetch"
    );
}

#[tokio::test]
async fn near_expiry_token_is_refreshed() {
    let calls = Arc::new(AtomicUsize::new(0));
    // Expiry inside the usability safety margin -> always treated as stale.
    let credential = Arc::new(MockCredential {
        token: "tok".to_string(),
        expires_in: AzureDuration::seconds(5),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    let _ = ext.get_token().await.expect("first");
    let _ = ext.get_token().await.expect("second");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        2,
        "stale token must be refreshed on each call"
    );
}

#[tokio::test]
async fn get_token_error_maps_to_capability_error() {
    let ext = make_extension(Arc::new(FailingCredential {
        call_count: Arc::new(AtomicUsize::new(0)),
    }));
    let err = ext
        .get_token()
        .await
        .expect_err("failing credential errors");
    assert_eq!(err.capability, "bearer_token_provider");
    assert_eq!(err.extension, "test-ext");
}

#[tokio::test]
async fn clones_share_one_token_cache() {
    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "shared".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });

    // The engine hands the capability to each consumer as a clone of the same
    // extension; model that with two clones sharing one `Arc<Inner>`.
    let consumer_a = make_extension(credential);
    let consumer_b = consumer_a.clone();

    // Consumer A's first call takes the slow path and fetches exactly once.
    let a = consumer_a.get_token().await.expect("A acquires");
    assert_eq!(a.expose_token(), "shared");
    assert_eq!(calls.load(Ordering::SeqCst), 1);

    // Consumer B (a separate clone) sees the same cached token on the fast
    // path -- no second credential call. This proves clones share one cache
    // and refresh loop rather than each keeping its own.
    let b = consumer_b.get_token().await.expect("B acquires");
    assert_eq!(b.expose_token(), "shared");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "clones must share one cache; B must not re-fetch"
    );

    // A token published through one clone is also visible via another clone's
    // stream subscription (shared watch channel).
    let mut stream_b = consumer_b.token_stream();
    let streamed = stream_b
        .next()
        .await
        .expect("stream yields the shared token");
    assert_eq!(streamed.expose_token(), "shared");
}

// -- Metrics tracker tests -------------------------------------

#[test]
fn metrics_tracker_records_snapshots_and_reports() {
    let mut tracker = make_tracker();

    // A fresh tracker snapshots to all-zero values.
    let before = tracker.snapshot();
    assert!(
        before.get_metrics().iter().all(|m| m.is_zero()),
        "a new tracker starts at zero"
    );

    tracker.record_success(12.5);
    tracker.record_failure();
    tracker.record_publish();

    // Every metric is non-zero once each counter/latency has been recorded.
    let after = tracker.snapshot();
    assert!(
        after.get_metrics().iter().all(|m| !m.is_zero()),
        "every metric is non-zero after recording"
    );

    // Reporting flushes the recorded metrics to the telemetry channel.
    let (rx, mut reporter) =
        otel_arrow_dfe_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
    tracker.report(&mut reporter).expect("report succeeds");
    assert!(
        rx.try_recv().is_ok(),
        "reporter received the metric snapshot"
    );
}

#[tokio::test]
async fn get_token_throttles_after_recent_failure() {
    let calls = Arc::new(AtomicUsize::new(0));
    let ext = make_extension(Arc::new(FailingCredential {
        call_count: Arc::clone(&calls),
    }));

    // First miss actually hits the credential and fails.
    let _ = ext.get_token().await.expect_err("first attempt fails");
    // Second miss within the cooldown is throttled by the negative cache: it
    // errors without a further credential call.
    let _ = ext.get_token().await.expect_err("second attempt throttled");
    assert_eq!(
        calls.load(Ordering::SeqCst),
        1,
        "recent failure must throttle the next slow-path fetch"
    );
}

#[tokio::test]
async fn token_stream_yields_published_token() {
    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "streamed".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    let mut stream = ext.token_stream();
    // Acquiring a token publishes it onto the watch channel.
    let _ = ext.get_token().await.expect("token acquired");
    let published = stream.next().await.expect("stream yields a value");
    assert_eq!(published.expose_token(), "streamed");
}

// Scenario: A consumer subscribes to token_stream() after a token was already published.
// Guarantees: The late subscription immediately yields the current token, honoring the
// BearerTokenProvider contract that a subscriber created after a publish never misses the
// already-current token (so consumers need no separate get_token() seeding step).
#[tokio::test]
async fn token_stream_replays_current_token_to_late_subscriber() {
    use std::time::Duration;

    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "streamed".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    // Publish a token BEFORE anyone subscribes.
    let _ = ext.get_token().await.expect("token acquired");

    // A subscription created after the publish must still promptly observe the
    // current token instead of blocking until the next refresh.
    let mut stream = ext.token_stream();
    let published = tokio::time::timeout(Duration::from_millis(200), stream.next())
        .await
        .expect("late subscriber must receive the current token promptly")
        .expect("stream is not closed");
    assert_eq!(published.expose_token(), "streamed");
}

#[tokio::test]
async fn token_stream_skips_initial_none() {
    use std::time::Duration;

    let calls = Arc::new(AtomicUsize::new(0));
    let credential = Arc::new(MockCredential {
        token: "streamed".to_string(),
        expires_in: AzureDuration::minutes(60),
        call_count: Arc::clone(&calls),
    });
    let ext = make_extension(credential);

    let mut stream = ext.token_stream();
    // The cache starts as `None`; the stream must filter it out and stay
    // pending rather than yielding a spurious value.
    let before = tokio::time::timeout(Duration::from_millis(50), stream.next()).await;
    assert!(
        before.is_err(),
        "stream must not yield before a token is published"
    );

    // Once a token is published, the stream yields it.
    let _ = ext.get_token().await.expect("token acquired");
    let published = tokio::time::timeout(Duration::from_millis(200), stream.next())
        .await
        .expect("stream yields after publish")
        .expect("stream is not closed");
    assert_eq!(published.expose_token(), "streamed");
}
