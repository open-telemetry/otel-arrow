// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Subscriber simulation utilities.

/// Delay configuration for simulating slow subscribers.
#[derive(Debug, Clone, Copy, Default)]
pub struct SubscriberDelay {
    /// Delay in milliseconds per bundle consumed
    pub per_bundle_ms: u64,
}

impl SubscriberDelay {
    /// Creates a new delay configuration.
    pub fn new(per_bundle_ms: u64) -> Self {
        Self { per_bundle_ms }
    }

    /// Applies the delay if configured.
    pub fn apply(&self) {
        if self.per_bundle_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.per_bundle_ms));
        }
    }
}
