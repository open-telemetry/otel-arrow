// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the shared refresh scheduling helpers.

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::time::{Duration, Instant};

use async_trait::async_trait;
use otel_arrow_dfe_engine::capability::auth::BearerToken;
use otel_arrow_dfe_engine::capability::auth::bearer_token_provider::BearerTokenProvider;
use otel_arrow_dfe_engine::control::ExtensionControlMsg;
use otel_arrow_dfe_engine::shared::extension::{ControlChannel, Extension};
use otel_arrow_dfe_engine::shared::message::SharedReceiver;
use otel_arrow_dfe_telemetry::instrument::{Counter, Mmsc};
use otel_arrow_dfe_telemetry::registry::TelemetryRegistryHandle;
use otel_arrow_dfe_telemetry::testing::EmptyAttributes;
use otel_arrow_dfe_telemetry_macros::metric_set;

use crate::common::background_refresh::{
    BackgroundFetchOutcome, BackgroundProviderExtension, BackgroundProviderMetrics,
    BackgroundProviderMetricsTracker, BackgroundProviderRefreshPolicy, BackgroundProviderSource,
};

use super::provider::{
    jitter_refresh, negative_cache_window_secs, retry_backoff_secs, schedule_next,
};

#[metric_set(name = "test.background_refresh")]
#[derive(Debug, Default, Clone)]
struct TestMetrics {
    /// Successful acquisitions.
    #[metric(unit = "{acquisition}")]
    successes: Counter<u64>,
    /// Failed acquisitions.
    #[metric(unit = "{acquisition}")]
    failures: Counter<u64>,
    /// Publications.
    #[metric(unit = "{publication}")]
    publishes: Counter<u64>,
    /// Acquisition duration.
    #[metric(unit = "ms")]
    latency: Mmsc,
}

impl BackgroundProviderMetrics for TestMetrics {
    fn successes(&mut self) -> &mut Counter<u64> {
        &mut self.successes
    }
    fn failures(&mut self) -> &mut Counter<u64> {
        &mut self.failures
    }
    fn publishes(&mut self) -> &mut Counter<u64> {
        &mut self.publishes
    }
    fn success_latency(&mut self) -> &mut Mmsc {
        &mut self.latency
    }
}

type TestExtension<S> =
    BackgroundProviderExtension<S, TestMetrics, BearerToken, BearerTokenProvider>;

fn new_test_extension<S>(
    name: &'static str,
    source: S,
    policy: BackgroundProviderRefreshPolicy,
) -> TestExtension<S>
where
    S: BackgroundProviderSource<BearerToken>,
{
    let (tx, _rx) = tokio::sync::watch::channel(None);
    let registry = TelemetryRegistryHandle::new();
    let metrics = registry.register_metric_set::<TestMetrics>(EmptyAttributes());
    BackgroundProviderExtension::new(
        name,
        source,
        policy,
        tx,
        BackgroundProviderMetricsTracker::new(metrics),
    )
}

// -- schedule_next timing tests --------------------------------

/// Next-refresh delay used for non-expiring values (~1 year). The loop is still
/// woken by control messages in the meantime.
const NON_EXPIRING_REFRESH_SECS: u64 = 365 * 24 * 60 * 60;

/// Scenario: Build expiry-driven policies with buffers below, at, and above the usability margin.
/// Guarantees: Only a buffer strictly greater than the usability margin is accepted.
#[tokio::test]
async fn ctor_validates_expiry_buffer() {
    let usable_margin = Duration::from_secs(1);
    let non_expiring_refresh_interval = Duration::from_secs(10);

    assert!(
        BackgroundProviderRefreshPolicy::expiry_driven(
            usable_margin,
            non_expiring_refresh_interval,
            Duration::from_secs(0)
        )
        .is_err()
    );
    assert!(
        BackgroundProviderRefreshPolicy::expiry_driven(
            usable_margin,
            non_expiring_refresh_interval,
            Duration::from_secs(1)
        )
        .is_err()
    );
    assert!(
        BackgroundProviderRefreshPolicy::expiry_driven(
            usable_margin,
            non_expiring_refresh_interval,
            Duration::from_secs(2)
        )
        .is_ok()
    );
}

/// Scenario: Build periodic policies around the scheduler's supported interval boundaries.
/// Guarantees: Only intervals from ten seconds through 365 days inclusive are accepted.
#[test]
fn periodic_validates_interval_bounds() {
    assert!(BackgroundProviderRefreshPolicy::periodic(Duration::from_secs(9)).is_err());
    assert!(BackgroundProviderRefreshPolicy::periodic(Duration::from_secs(10)).is_ok());
    assert!(
        BackgroundProviderRefreshPolicy::periodic(Duration::from_secs(365 * 24 * 60 * 60)).is_ok()
    );
    assert!(
        BackgroundProviderRefreshPolicy::periodic(Duration::from_secs(365 * 24 * 60 * 60 + 1))
            .is_err()
    );
}

/// Scenario: Schedule the next refresh for a token expiring in ~1 hour with a 5m buffer.
/// Guarantees: The refresh is scheduled `expiry_buffer` before expiry (~3300s out), ahead of expiry.
#[tokio::test]
async fn schedule_next_refreshes_before_expiry() {
    let token = BearerToken::with_expiry(
        "t".to_owned(),
        Some(Instant::now() + Duration::from_secs(3600)),
    );
    let refresh_at = schedule_next(
        token.expires_on(),
        Duration::from_secs(300),
        Duration::from_secs(NON_EXPIRING_REFRESH_SECS),
    );
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs_f64();
    assert!((secs - 3300.0).abs() < 5.0, "expected ~3300s, got {secs}");
}

/// Scenario: Schedule the next refresh for a token expiring in 5s, where subtracting the buffer
/// would land in the past.
/// Guarantees: The schedule floors at MIN_TOKEN_REFRESH_INTERVAL_SECS (~10s) rather than the past.
#[tokio::test]
async fn schedule_next_floors_near_expiry() {
    let token = BearerToken::with_expiry(
        "t".to_owned(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    let refresh_at = schedule_next(
        token.expires_on(),
        Duration::from_secs(300),
        Duration::from_secs(NON_EXPIRING_REFRESH_SECS),
    );
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs_f64();
    assert!((secs - 10.0).abs() < 2.0, "expected ~10s floor, got {secs}");
}

/// Scenario: Schedule the next refresh for a token with no known expiry.
/// Guarantees: The refresh is pushed far into the future (~1 year), so it is not needlessly refreshed.
#[tokio::test]
async fn schedule_next_pushes_non_expiring_far_out() {
    let token = BearerToken::without_expiry("t".to_owned());
    let refresh_at = schedule_next(
        token.expires_on(),
        Duration::from_secs(300),
        Duration::from_secs(NON_EXPIRING_REFRESH_SECS),
    );
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs();
    assert!(
        secs > 300 * 24 * 60 * 60,
        "expected far-future refresh, got {secs}s"
    );
}

// -- retry backoff tests ---------------------------------------

/// Scenario: Compute the retry backoff for increasing consecutive-failure counts.
/// Guarantees: It doubles from the base delay and is clamped at the max without overflowing the shift.
#[test]
fn retry_backoff_grows_exponentially_and_caps() {
    // Zero prior failures starts at the base retry interval (10s).
    assert_eq!(retry_backoff_secs(0), 10);
    // Each consecutive failure doubles the base delay.
    assert_eq!(retry_backoff_secs(1), 20);
    assert_eq!(retry_backoff_secs(2), 40);
    assert_eq!(retry_backoff_secs(3), 80);
    assert_eq!(retry_backoff_secs(4), 160);
    // Growth is clamped at the max (300s) and stays there.
    assert_eq!(retry_backoff_secs(5), 300);
    assert_eq!(retry_backoff_secs(6), 300);
    // A very large failure count must not overflow the shift.
    assert_eq!(retry_backoff_secs(u32::MAX), 300);
}

/// Scenario: Compute the slow-path negative-cache window for a growing failure streak, where the
/// count includes the failure that opened the current cooldown.
/// Guarantees: The window matches the backoff the refresh loop is waiting out for the same streak,
/// so cache-miss callers cannot keep probing a token endpoint the loop has already backed off from.
#[test]
fn negative_cache_window_tracks_retry_backoff() {
    // No failure recorded yet: the window degenerates to the base delay, and
    // `recently_failed` gates on `last_failure` being set anyway.
    assert_eq!(negative_cache_window_secs(0), 10);
    // The first failure holds the slow path off for the base delay, matching
    // `retry_backoff_secs(0)` that the loop uses for its first retry.
    assert_eq!(negative_cache_window_secs(1), retry_backoff_secs(0));
    // Each further failure widens both in lockstep.
    assert_eq!(negative_cache_window_secs(2), retry_backoff_secs(1));
    assert_eq!(negative_cache_window_secs(3), retry_backoff_secs(2));
    // A sustained outage settles at the 300s cap rather than the old fixed 10s
    // window, which is the regression this guards.
    assert_eq!(negative_cache_window_secs(6), 300);
    assert_eq!(negative_cache_window_secs(u32::MAX), 300);
}

// -- jitter_refresh tests --------------------------------------

/// Scenario: Jitter a refresh target that sits exactly at the minimum-refresh floor.
/// Guarantees: With no slack above the floor, the target is returned unchanged (no busy-loop pull to now).
#[tokio::test]
async fn jitter_refresh_preserves_min_interval_floor() {
    let target = tokio::time::Instant::now() + Duration::from_secs(10);
    for _ in 0..1000 {
        assert_eq!(
            jitter_refresh(target),
            target,
            "near-floor target must not be jittered earlier"
        );
    }
}

// Scenario: Jitter a far-out refresh target repeatedly.
/// Guarantees: Jitter only moves the target earlier, never by more than REFRESH_JITTER_SECS, and
/// never before the min-interval floor.
#[tokio::test]
async fn jitter_refresh_stays_within_bounds() {
    let now = tokio::time::Instant::now();
    let target = now + Duration::from_secs(3600);
    let floor = now + Duration::from_secs(10);
    for _ in 0..1000 {
        let jittered = jitter_refresh(target);
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

/// Scenario: A once-policy cache is populated on demand before the active task starts.
/// Guarantees: Startup and arbitrarily late consumers reuse that first value without another fetch.
#[tokio::test(start_paused = true)]
async fn once_reuses_value_acquired_before_start() {
    struct Source(Arc<AtomicUsize>);

    #[async_trait]
    impl BackgroundProviderSource<BearerToken> for Source {
        type Error = std::io::Error;

        fn expires_on(_value: &BearerToken) -> Option<Instant> {
            None
        }

        async fn fetch(&self) -> Result<BearerToken, Self::Error> {
            let _ = self.0.fetch_add(1, Ordering::SeqCst);
            Ok(BearerToken::without_expiry("initial"))
        }

        fn log_refresh_failure(&self, error: &Self::Error) {
            panic!("unexpected acquisition failure: {error}");
        }
    }

    let calls = Arc::new(AtomicUsize::new(0));
    let extension = new_test_extension(
        "once-test",
        Source(Arc::clone(&calls)),
        BackgroundProviderRefreshPolicy::once(),
    );
    assert_eq!(
        extension.get_value().await.unwrap().expose_token(),
        "initial"
    );
    let (ctrl_tx, ctrl_rx) = tokio::sync::mpsc::channel(1);
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let ctrl = ControlChannel::new(SharedReceiver::mpsc(ctrl_rx), shutdown_rx);
    let (effects, probe) =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
            "once-test".into(),
            Duration::from_secs(30),
        );
    let task = tokio::spawn(Box::new(extension.clone()).start(ctrl, effects));
    probe
        .wait_ready()
        .await
        .expect("cached startup signals readiness");
    tokio::time::advance(Duration::from_secs(10 * 365 * 24 * 60 * 60)).await;
    assert_eq!(
        extension.get_value().await.unwrap().expose_token(),
        "initial"
    );
    assert_eq!(
        extension
            .subscribe()
            .borrow()
            .as_ref()
            .unwrap()
            .expose_token(),
        "initial"
    );
    assert_eq!(calls.load(Ordering::SeqCst), 1);
    drop(ctrl_tx);
    let _ = task.await.expect("join").expect("clean shutdown");
}

/// Scenario: A periodic source backed by an internal SDK cache returns `Updated` on rotation,
/// `Unchanged` on in-memory cache hits, and `Err` with `clear_cache_on_error() == true` when
/// its cached token expires during an outage.
/// Guarantees: `Unchanged` polls do not emit duplicate values or inflate acquisition/publish
/// counters, and a refresh error evicts the cached token so `get_value` fails instead of serving
/// an expired non-expiring token.
#[tokio::test(start_paused = true)]
async fn periodic_cached_source_skips_unchanged_publishes_and_clears_cache_on_error() {
    struct CachedSdkSource(Arc<AtomicUsize>);

    #[async_trait]
    impl BackgroundProviderSource<BearerToken> for CachedSdkSource {
        type Error = std::io::Error;

        fn expires_on(_value: &BearerToken) -> Option<Instant> {
            None
        }

        async fn fetch(&self) -> Result<BearerToken, Self::Error> {
            self.fetch_outcome()
                .await
                .map(BackgroundFetchOutcome::into_value)
        }

        async fn fetch_outcome(&self) -> Result<BackgroundFetchOutcome<BearerToken>, Self::Error> {
            match self.0.fetch_add(1, Ordering::SeqCst) {
                0 => Ok(BackgroundFetchOutcome::Updated(
                    BearerToken::without_expiry("tok-1"),
                )),
                1 => Ok(BackgroundFetchOutcome::Unchanged(
                    BearerToken::without_expiry("tok-1"),
                )),
                2 => Ok(BackgroundFetchOutcome::Updated(
                    BearerToken::without_expiry("tok-2"),
                )),
                3 => Err(std::io::Error::other("sdk token expired and mds down")),
                _ => Ok(BackgroundFetchOutcome::Unchanged(
                    BearerToken::without_expiry("tok-3"),
                )),
            }
        }

        fn clear_cache_on_error(&self) -> bool {
            true
        }

        fn log_refresh_failure(&self, _error: &Self::Error) {}
    }

    let step = Arc::new(AtomicUsize::new(0));
    let extension = new_test_extension(
        "cached-sdk-test",
        CachedSdkSource(Arc::clone(&step)),
        BackgroundProviderRefreshPolicy::periodic(Duration::from_secs(10)).unwrap(),
    );
    let mut rx = extension.subscribe();
    let (ctrl_tx, ctrl_rx) = tokio::sync::mpsc::channel(1);
    let (_shutdown_tx, shutdown_rx) = tokio::sync::oneshot::channel();
    let ctrl = ControlChannel::new(SharedReceiver::mpsc(ctrl_rx), shutdown_rx);
    let (effects, probe) =
        otel_arrow_dfe_engine::testing::test_extension_effect_handler_with_readiness(
            "cached-sdk-test".into(),
            Duration::from_secs(30),
        );
    let task = tokio::spawn(Box::new(extension.clone()).start(ctrl, effects));

    // Step 0: initial Updated("tok-1") is published and signals readiness.
    probe.wait_ready().await.expect("ready");
    assert_eq!(
        rx.borrow_and_update().as_ref().unwrap().expose_token(),
        "tok-1"
    );

    // Step 1 (t = 10s): Unchanged("tok-1") does not mark the watch channel as changed.
    tokio::time::advance(Duration::from_secs(10)).await;
    tokio::task::yield_now().await;
    assert_eq!(step.load(Ordering::SeqCst), 2);
    assert!(
        !rx.has_changed().unwrap(),
        "Unchanged poll must not re-publish to watch subscribers"
    );

    // Step 2 (t = 20s): Updated("tok-2") publishes the rotated token.
    tokio::time::advance(Duration::from_secs(10)).await;
    tokio::task::yield_now().await;
    assert_eq!(step.load(Ordering::SeqCst), 3);
    assert!(rx.has_changed().unwrap());
    assert_eq!(
        rx.borrow_and_update().as_ref().unwrap().expose_token(),
        "tok-2"
    );

    // Step 3 (t = 30s): Err with clear_cache_on_error() == true evicts the cached token.
    tokio::time::advance(Duration::from_secs(10)).await;
    tokio::task::yield_now().await;
    assert_eq!(step.load(Ordering::SeqCst), 4);
    assert!(
        rx.borrow_and_update().is_none(),
        "watch channel cache must be evicted to None on error"
    );
    assert!(
        extension.get_value().await.is_err(),
        "get_value must fail after cache eviction instead of serving stale non-expiring token"
    );

    // Step 4 (after retry backoff <= 10s): Unchanged("tok-3") while watch cache is None
    // repopulates the watch channel via the `self.inner.tx.borrow().is_none()` fallback.
    tokio::time::advance(Duration::from_secs(10)).await;
    tokio::task::yield_now().await;
    assert_eq!(step.load(Ordering::SeqCst), 5);
    assert!(
        rx.has_changed().unwrap(),
        "Unchanged outcome while watch cache is None must publish to repopulate cache"
    );
    assert_eq!(
        rx.borrow_and_update().as_ref().unwrap().expose_token(),
        "tok-3"
    );
    assert_eq!(extension.get_value().await.unwrap().expose_token(), "tok-3");

    let (report_rx, metrics_reporter) =
        otel_arrow_dfe_telemetry::reporter::MetricsReporter::create_new_and_receiver(4);
    ctrl_tx
        .send(ExtensionControlMsg::CollectTelemetry { metrics_reporter })
        .await
        .unwrap();
    let snapshot = report_rx.recv_async().await.unwrap();
    let values = snapshot.get_metrics();
    // Steps 0, 2, and 4 publish (3 successes/publications); step 1 (Unchanged while
    // populated) is skipped, and step 3 counts as 1 failure.
    assert_eq!(values[0].to_u64_lossy(), 3, "successes should be 3");
    assert_eq!(values[1].to_u64_lossy(), 1, "failures should be 1");
    assert_eq!(values[2].to_u64_lossy(), 3, "publishes should be 3");

    drop(ctrl_tx);
    let _ = task.await.expect("join").expect("clean shutdown");
}
