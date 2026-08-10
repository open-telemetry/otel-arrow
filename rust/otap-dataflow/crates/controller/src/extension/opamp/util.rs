// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use serde::{self, Deserialize, Serialize};

/// Jitter range: the returned delay is multiplied by a random factor
/// in the range `[1 - JITTER_FRACTION, 1 + JITTER_FRACTION]`.
const JITTER_FRACTION: f64 = 0.2;

#[derive(Deserialize, Debug, Clone, Serialize)]
pub struct ExponentialBackoff {
    #[serde(with = "humantime_serde", rename = "initial")]
    current: Duration,

    #[serde(with = "humantime_serde")]
    max: Duration,

    #[serde(default = "default_factor")]
    factor: f64,
}

impl ExponentialBackoff {
    pub fn new(initial: Duration, max: Duration) -> Self {
        Self {
            current: initial,
            max,
            factor: 2.0,
        }
    }

    /// Returns the current backoff duration (with jitter) and advances
    /// the internal state for the next retry.
    ///
    /// Jitter is applied as +/- 20% to prevent thundering herd effects
    /// when multiple clients reconnect simultaneously.
    pub fn next_delay(&mut self) -> Duration {
        let base = self.current;

        self.current = Duration::from_secs_f64(
            (self.current.as_secs_f64() * self.factor).min(self.max.as_secs_f64()),
        );

        apply_jitter(base)
    }

    pub fn set_current(&mut self, current: Duration) {
        self.current = current;
    }
}

/// Apply jitter to a duration. Uses a simple deterministic-ish approach
/// based on the current time's nanosecond component to avoid adding a
/// random number generator dependency.
fn apply_jitter(base: Duration) -> Duration {
    // Use the nanosecond component of the current time as a cheap source of
    // pseudo-randomness. This is NOT cryptographically secure, but for backoff
    // jitter it provides sufficient variation across different clients.
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .subsec_nanos();

    // Map nanos to a jitter multiplier in [1 - JITTER_FRACTION, 1 + JITTER_FRACTION]
    let normalized = (nanos as f64) / 999_999_999.0;
    let jitter_multiplier = 1.0 - JITTER_FRACTION + (normalized * 2.0 * JITTER_FRACTION);

    Duration::from_secs_f64(base.as_secs_f64() * jitter_multiplier)
}

fn default_factor() -> f64 {
    2.0
}
