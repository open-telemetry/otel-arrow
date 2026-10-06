// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Logs sampling policies for the `logger:` macro argument. You
//! must use `otel_component_scope!` to enable this feature.

use tracing::{Dispatch, Event, Metadata};

/// Logs sampler decides when to accept a `tracing` event using only
/// the metadata, before the event is fully evaluated. May be
/// stateful.
pub trait Sampler {
    /// Observes one enabled log event and decides to keep or drop.
    /// By default, keep.
    fn should_sample(&mut self, _metadata: &Metadata<'_>) -> bool {
        true
    }

    /// Handles one selected event.
    /// By default, forward to the active subscriber.
    fn emit(&mut self, event: &Event<'_>, dispatch: &Dispatch) {
        dispatch.event(event);
    }
}

impl<S: Sampler + ?Sized> Sampler for &mut S {
    fn should_sample(&mut self, metadata: &Metadata<'_>) -> bool {
        (**self).should_sample(metadata)
    }

    fn emit(&mut self, event: &Event<'_>, dispatch: &Dispatch) {
        (**self).emit(event, dispatch);
    }
}
