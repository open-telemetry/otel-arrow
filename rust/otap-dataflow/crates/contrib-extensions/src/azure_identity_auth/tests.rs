// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the Azure Identity Auth extension.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use azure_core::credentials::{AccessToken, TokenCredential, TokenRequestOptions};
use azure_core::time::{Duration as AzureDuration, OffsetDateTime};
use futures::StreamExt;
use otap_df_config::error::Error as ConfigError;
use otap_df_engine::shared::capability::auth::bearer_token_provider::BearerTokenProvider as SharedBearerTokenProvider;
use otap_df_telemetry::registry::TelemetryRegistryHandle;
use otap_df_telemetry::testing::EmptyAttributes;
use tokio::sync::watch;

use super::auth::Auth;
use super::config::{AuthMethod, Config};
use super::error::Error;
use super::extension::AzureIdentityAuthExtension;
use super::metrics::{AzureIdentityAuthMetrics, AzureIdentityAuthMetricsTracker};
use super::*;

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
    assert_eq!(cfg.startup_timeout, std::time::Duration::from_secs(30));
}

#[test]
fn startup_timeout_parses_and_rejects_zero() {
    let cfg = config_from_json(serde_json::json!({ "startup_timeout": "45s" }))
        .expect("human-readable duration parses");
    assert_eq!(cfg.startup_timeout, std::time::Duration::from_secs(45));

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

// -- Credential construction tests ------------------------------

#[test]
fn managed_identity_system_assigned_credential_constructs() {
    otap_df_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({ "method": "managed_identity" })).unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

#[test]
fn managed_identity_user_assigned_credential_constructs() {
    otap_df_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({
        "method": "managed_identity",
        "client_id": "00000000-0000-0000-0000-000000000000",
    }))
    .unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

#[test]
fn development_credential_constructs() {
    otap_df_otap::crypto::ensure_crypto_provider();
    let cfg = config_from_json(serde_json::json!({ "method": "development" })).unwrap();
    assert!(Auth::new(&cfg).is_ok());
}

#[test]
fn workload_identity_credential_construct_is_attempted() {
    otap_df_otap::crypto::ensure_crypto_provider();
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
    AzureIdentityAuthExtension::new("test-ext", auth, tx, make_tracker())
}

fn make_tracker() -> AzureIdentityAuthMetricsTracker {
    let registry = TelemetryRegistryHandle::new();
    let metric_set = registry.register_metric_set::<AzureIdentityAuthMetrics>(EmptyAttributes());
    AzureIdentityAuthMetricsTracker::new(metric_set)
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

    // Debug formatting is exercised for observability tooling.
    assert!(format!("{tracker:?}").contains("AzureIdentityAuthMetricsTracker"));

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
        otap_df_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
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

// -- schedule_next timing tests --------------------------------

/// Scenario: schedule the next refresh for a token expiring in ~1 hour.
/// Guarantees: the refresh is scheduled TOKEN_EXPIRY_BUFFER_SECS before expiry
/// (~3301s out), so refresh happens ahead of expiry with buffer to spare.
#[tokio::test]
async fn schedule_next_refreshes_before_expiry() {
    use otap_df_engine::capability::auth::BearerToken;
    use std::time::{Duration, Instant};

    let token = BearerToken::with_expiry(
        "t".to_owned(),
        Some(Instant::now() + Duration::from_secs(3600)),
    );
    let refresh_at = extension::schedule_next(&token);
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs_f64();
    // 3600 - TOKEN_EXPIRY_BUFFER_SECS (299) = 3301, allowing execution slack.
    assert!((secs - 3301.0).abs() < 5.0, "expected ~3301s, got {secs}");
}

/// Scenario: schedule the next refresh for a token expiring in only 5s, where
/// subtracting the expiry buffer would underflow past now.
/// Guarantees: the schedule floors at MIN_TOKEN_REFRESH_INTERVAL_SECS (~10s)
/// instead of scheduling in the past.
#[tokio::test]
async fn schedule_next_floors_near_expiry() {
    use otap_df_engine::capability::auth::BearerToken;
    use std::time::{Duration, Instant};

    // Expires in 5s: the refresh target underflows past `now`, so the
    // MIN_TOKEN_REFRESH_INTERVAL_SECS (10s) floor applies.
    let token = BearerToken::with_expiry(
        "t".to_owned(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    let refresh_at = extension::schedule_next(&token);
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs_f64();
    assert!((secs - 10.0).abs() < 2.0, "expected ~10s floor, got {secs}");
}

/// Scenario: schedule the next refresh for a non-expiring token.
/// Guarantees: the refresh is pushed far into the future (~1 year), so a
/// non-expiring token is not needlessly refreshed.
#[tokio::test]
async fn schedule_next_pushes_non_expiring_far_out() {
    use otap_df_engine::capability::auth::BearerToken;

    let token = BearerToken::without_expiry("t".to_owned());
    let refresh_at = extension::schedule_next(&token);
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs();
    // Non-expiring tokens push the refresh ~1 year out.
    assert!(
        secs > 300 * 24 * 60 * 60,
        "expected far-future refresh, got {secs}s"
    );
}

// -- retry backoff tests ---------------------------------------

#[test]
fn retry_backoff_grows_exponentially_and_caps() {
    // Zero prior failures starts at the base retry interval (10s).
    assert_eq!(extension::retry_backoff_secs(0), 10);
    // Each consecutive failure doubles the base delay.
    assert_eq!(extension::retry_backoff_secs(1), 20);
    assert_eq!(extension::retry_backoff_secs(2), 40);
    assert_eq!(extension::retry_backoff_secs(3), 80);
    assert_eq!(extension::retry_backoff_secs(4), 160);
    // Growth is clamped at the max (300s) and stays there.
    assert_eq!(extension::retry_backoff_secs(5), 300);
    assert_eq!(extension::retry_backoff_secs(6), 300);
    // A very large failure count must not overflow the shift.
    assert_eq!(extension::retry_backoff_secs(u32::MAX), 300);
}

// -- jitter_refresh tests --------------------------------------

#[tokio::test]
async fn jitter_refresh_preserves_min_interval_floor() {
    use std::time::Duration;

    // A target exactly at the 10s minimum-refresh floor has no slack to jitter,
    // so it must be returned unchanged rather than pulled toward `now` (which
    // would busy-loop the refresh task while the token is still fresh).
    let target = tokio::time::Instant::now() + Duration::from_secs(10);
    for _ in 0..1000 {
        assert_eq!(
            extension::jitter_refresh(target),
            target,
            "near-floor target must not be jittered earlier"
        );
    }
}

#[tokio::test]
async fn jitter_refresh_stays_within_bounds() {
    use std::time::Duration;

    // A far-out target is jittered earlier by at most REFRESH_JITTER_SECS (60s)
    // and never earlier than the 10s floor from `now`.
    let now = tokio::time::Instant::now();
    let target = now + Duration::from_secs(3600);
    let floor = now + Duration::from_secs(10);
    for _ in 0..1000 {
        let jittered = extension::jitter_refresh(target);
        assert!(
            jittered <= target,
            "jitter must only move the refresh earlier"
        );
        assert!(
            jittered >= target - Duration::from_secs(60),
            "jitter must not exceed REFRESH_JITTER_SECS"
        );
        assert!(
            jittered >= floor,
            "jitter must not precede the min-interval floor"
        );
    }
}
