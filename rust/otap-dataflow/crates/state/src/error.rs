// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Errors for the state crate.

use crate::event::{EventType, ObservedEvent};
use crate::phase::PipelinePhase;

/// All errors that can occur in the state crate.
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// The observed event channel was already closed.
    #[error("Failed to send observed event because the channel was closed: {event:?}")]
    ChannelClosed {
        /// The event that failed to be sent.
        event: Box<ObservedEvent>,
    },

    /// The observed event channel was full and the event could not be sent in time.
    #[error(
        "Failed to send observed event because the channel was full and the event could not be sent in time: {event:?}"
    )]
    ChannelTimeout {
        /// The event that failed to be sent.
        event: Box<ObservedEvent>,
    },

    /// Error returned for truly invalid (phase, event) pairs.
    #[error("Invalid transition: phase={phase:?}, event={event:?}, msg={message}")]
    InvalidTransition {
        /// The current phase when the event was applied.
        phase: PipelinePhase,
        /// The event that was applied.
        event: Box<EventType>,
        /// Error message.
        message: &'static str,
    },
}
