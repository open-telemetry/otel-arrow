// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for the state crate.

use crate::store::ObservedEvent;

/// All errors that can occur in the state crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The observed event channel was already closed.
    #[error("Failed to send observed event because the channel was closed: {event:?}")]
    ChannelClosed {
        /// The event that failed to be sent.
        event: ObservedEvent,
    },

    /// The observed event channel was full and the event could not be sent in time.
    #[error(
        "Failed to send observed event because the channel was full and the event could not be sent in time: {event:?}"
    )]
    ChannelTimeout {
        /// The event that failed to be sent.
        event: ObservedEvent,
    },
}
