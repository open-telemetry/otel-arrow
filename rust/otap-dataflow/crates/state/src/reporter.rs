// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A reporter of observed events.

use crate::store::ObservedEvent;
use std::time::Duration;

/// A sharable/clonable observed event reporter sending events to an `ObservedStore`.
#[derive(Clone)]
pub struct ObservedEventReporter {
    timeout: Duration,
    sender: flume::Sender<ObservedEvent>,
}

impl ObservedEventReporter {
    /// Creates a new `ObservedEventReporter` with the given sender channel.
    #[must_use]
    pub(crate) fn new(timeout: Duration, sender: flume::Sender<ObservedEvent>) -> Self {
        Self { timeout, sender }
    }

    /// Report an observed event.
    ///
    /// Note: This method does not return an error if sending the event to the reporting channel
    /// fails, as this is not sufficient reason to interrupt the normal flow of the system under
    /// observation. However, an error message is logged to the standard error output.
    #[allow(
        clippy::print_stderr,
        reason = "Use `eprintln!` while waiting for a decision on a framework for debugging/tracing."
    )]
    pub fn report(&self, event: ObservedEvent) {
        match self.sender.send_timeout(event, self.timeout) {
            Err(flume::SendTimeoutError::Timeout(event)) => {
                eprintln!("Timeout sending observed event: {event:?}")
            }
            Err(flume::SendTimeoutError::Disconnected(event)) => {
                eprintln!("Disconnected event: {event:?}")
            }
            Ok(_) => {}
        }
    }
}
