// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Linux user_events receiver.
#[cfg(all(feature = "user_events-receiver", target_os = "linux"))]
pub mod user_events_receiver;
