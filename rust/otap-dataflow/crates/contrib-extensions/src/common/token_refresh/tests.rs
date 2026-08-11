// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Unit tests for the shared refresh scheduling helpers.

use std::time::{Duration, Instant};

use otap_df_engine::capability::auth::BearerToken;

use super::provider::{
    jitter_refresh, negative_cache_window_secs, retry_backoff_secs, schedule_next,
};

// -- schedule_next timing tests --------------------------------

// Scenario: Schedule the next refresh for a token expiring in ~1 hour with a 5m buffer.
// Guarantees: The refresh is scheduled `expiry_buffer` before expiry (~3300s out), ahead of expiry.
#[tokio::test]
async fn schedule_next_refreshes_before_expiry() {
    let token = BearerToken::with_expiry(
        "t".to_owned(),
        Some(Instant::now() + Duration::from_secs(3600)),
    );
    let refresh_at = schedule_next(&token, Duration::from_secs(300));
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs_f64();
    assert!((secs - 3300.0).abs() < 5.0, "expected ~3300s, got {secs}");
}

// Scenario: Schedule the next refresh for a token expiring in 5s, where subtracting the buffer
// would land in the past.
// Guarantees: The schedule floors at MIN_TOKEN_REFRESH_INTERVAL_SECS (~10s) rather than the past.
#[tokio::test]
async fn schedule_next_floors_near_expiry() {
    let token = BearerToken::with_expiry(
        "t".to_owned(),
        Some(Instant::now() + Duration::from_secs(5)),
    );
    let refresh_at = schedule_next(&token, Duration::from_secs(300));
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs_f64();
    assert!((secs - 10.0).abs() < 2.0, "expected ~10s floor, got {secs}");
}

// Scenario: Schedule the next refresh for a token with no known expiry.
// Guarantees: The refresh is pushed far into the future (~1 year), so it is not needlessly refreshed.
#[tokio::test]
async fn schedule_next_pushes_non_expiring_far_out() {
    let token = BearerToken::without_expiry("t".to_owned());
    let refresh_at = schedule_next(&token, Duration::from_secs(300));
    let secs = refresh_at
        .saturating_duration_since(tokio::time::Instant::now())
        .as_secs();
    assert!(
        secs > 300 * 24 * 60 * 60,
        "expected far-future refresh, got {secs}s"
    );
}

// -- retry backoff tests ---------------------------------------

// Scenario: Compute the retry backoff for increasing consecutive-failure counts.
// Guarantees: It doubles from the base delay and is clamped at the max without overflowing the shift.
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

// Scenario: Compute the slow-path negative-cache window for a growing failure streak, where the
// count includes the failure that opened the current cooldown.
// Guarantees: The window matches the backoff the refresh loop is waiting out for the same streak,
// so cache-miss callers cannot keep probing a token endpoint the loop has already backed off from.
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

// Scenario: Jitter a refresh target that sits exactly at the minimum-refresh floor.
// Guarantees: With no slack above the floor, the target is returned unchanged (no busy-loop pull to now).
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
// Guarantees: Jitter only moves the target earlier, never by more than REFRESH_JITTER_SECS, and
// never before the min-interval floor.
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
