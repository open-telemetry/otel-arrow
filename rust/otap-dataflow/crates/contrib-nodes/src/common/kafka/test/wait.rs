// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Async retry helpers.
//!
//! Kafka integration tests repeatedly hand-roll "loop up to N times, sleep M
//! ms, check condition" (e.g. poll until a committed offset advances, or until
//! a consumer receives an assignment). These helpers collapse that pattern into
//! a single call with an explicit overall `timeout` and per-attempt `interval`.

use std::future::Future;
use std::time::Duration;

use tokio::time::Instant;

/// Polls `predicate` until it returns `true` or `timeout` elapses, sleeping
/// `interval` between attempts. Returns whether the predicate ever succeeded.
///
/// The predicate is evaluated at least once (before the first sleep) so a
/// zero timeout still performs one check.
pub(crate) async fn poll_until<F>(timeout: Duration, interval: Duration, mut predicate: F) -> bool
where
    F: FnMut() -> bool,
{
    let deadline = Instant::now() + timeout;
    loop {
        if predicate() {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(interval).await;
    }
}

/// Async-predicate variant of [`poll_until`], for cases that must drive
/// `recv()` or query the broker between attempts.
pub(crate) async fn poll_until_async<F, Fut>(
    timeout: Duration,
    interval: Duration,
    mut predicate: F,
) -> bool
where
    F: FnMut() -> Fut,
    Fut: Future<Output = bool>,
{
    let deadline = Instant::now() + timeout;
    loop {
        if predicate().await {
            return true;
        }
        if Instant::now() >= deadline {
            return false;
        }
        tokio::time::sleep(interval).await;
    }
}
