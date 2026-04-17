// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Linux user_events collection via one_collect.

mod session;

pub(crate) use session::{UserEventsSession, UserEventsSessionConfig, UserEventsSubscription};
