// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-scoped notification that graceful shutdown has expired.
//!
//! The watch channel holds one boolean and stays within one pipeline's local
//! runtime. It provides retained fanout without introducing an unbounded queue
//! or cross-core coordination.

use tokio::sync::watch;

/// Triggers forced shutdown for every processor in one pipeline runtime.
#[derive(Debug)]
pub(crate) struct ForcedShutdownTrigger {
    state: watch::Sender<bool>,
}

impl ForcedShutdownTrigger {
    /// Retains the forced-shutdown state and wakes all current waiters.
    pub(crate) fn trigger(&self) {
        let _ = self.state.send_replace(true);
    }
}

/// Observes when one pipeline runtime has entered forced shutdown.
#[derive(Clone, Debug)]
pub(crate) struct ForcedShutdownSignal {
    state: watch::Receiver<bool>,
}

impl ForcedShutdownSignal {
    /// Waits until forced shutdown is triggered, including when it happened before this call.
    pub(crate) async fn triggered(mut self) {
        while !*self.state.borrow_and_update() {
            if self.state.changed().await.is_err() {
                std::future::pending::<()>().await;
            }
        }
    }
}

/// Creates one retained forced-shutdown notification for a pipeline runtime.
pub(crate) fn new_forced_shutdown_signal() -> (ForcedShutdownTrigger, ForcedShutdownSignal) {
    let (state, receiver) = watch::channel(false);
    (
        ForcedShutdownTrigger { state },
        ForcedShutdownSignal { state: receiver },
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Forced shutdown is triggered before current and late observers begin waiting.
    /// Guarantees: The retained notification wakes every observer without requiring another trigger.
    #[tokio::test]
    async fn forced_shutdown_is_retained_for_late_waiters() {
        let (trigger, signal) = new_forced_shutdown_signal();
        let current = signal.clone();
        let current = tokio::spawn(current.triggered());

        trigger.trigger();

        current.await.expect("current waiter completes");
        signal.triggered().await;
    }
}
