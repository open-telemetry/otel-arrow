// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub(crate) mod common;
pub(crate) mod user_events;

pub(crate) use common::{
    CollectInitError, CollectedDrain, CollectedEvent, EventSource, UserEventsSource,
};
