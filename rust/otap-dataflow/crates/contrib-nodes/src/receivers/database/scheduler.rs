// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Non-overlapping delay-based query scheduling.

use std::time::Duration;
use tokio::time::{Instant, sleep_until};

/// Schedule for one query whose next interval starts after completion.
pub(crate) struct QueryScheduler {
    interval: Duration,
    next_due: Instant,
}

impl QueryScheduler {
    /// Creates a schedule that runs immediately on receiver startup.
    pub(crate) fn new(interval: Duration) -> Self {
        Self {
            interval,
            next_due: Instant::now(),
        }
    }

    /// Waits for the current deadline without modifying the next deadline.
    pub(crate) async fn wait(&self) {
        sleep_until(self.next_due).await;
    }

    /// Starts a full delay interval after the previous poll has completed.
    pub(crate) fn complete(&mut self) {
        self.next_due = Instant::now() + self.interval;
    }
}
