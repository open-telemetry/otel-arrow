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
    pub(crate) fn new(interval: Duration) -> Self {
        Self {
            interval,
            next_due: Instant::now(),
        }
    }

    pub(crate) async fn wait(&self) {
        sleep_until(self.next_due).await;
    }

    pub(crate) fn complete(&mut self) {
        self.next_due = Instant::now() + self.interval;
    }
}

#[cfg(test)]
mod tests {
    use super::QueryScheduler;
    use std::time::Duration;

    /// Scenario: A poll completes after its originally scheduled interval boundary.
    /// Guarantees: The next poll waits one complete interval instead of catching up.
    #[tokio::test(start_paused = true)]
    async fn schedules_from_completion() {
        let mut scheduler = QueryScheduler::new(Duration::from_secs(30));
        scheduler.wait().await;
        tokio::time::advance(Duration::from_secs(90)).await;
        scheduler.complete();

        let wait = scheduler.wait();
        tokio::pin!(wait);
        assert!(
            tokio::time::timeout(Duration::from_secs(29), &mut wait)
                .await
                .is_err()
        );
        tokio::time::advance(Duration::from_secs(1)).await;
        wait.await;
    }
}
