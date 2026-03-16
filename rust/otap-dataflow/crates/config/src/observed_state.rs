// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration of the observed store.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the observed state store.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObservedStateSettings {
    /// The size of the bounded reporting channel used for log (observational)
    /// events.  Engine lifecycle events use a separate unbounded channel and
    /// are not affected by this setting.
    pub reporting_channel_size: usize,

    /// Policy applied to engine lifecycle events **only** when sent through
    /// the fallback (lossy) reporter path.
    ///
    /// In production the controller obtains an engine reporter via
    /// `ObservedStateStore::engine_reporter`, which delivers engine events
    /// through a dedicated unbounded channel, bypassing this policy entirely.
    /// The policy remains available for backward-compatible or test scenarios
    /// that create a plain `ObservedStateStore::reporter`.
    pub engine_events: SendPolicy,

    /// Internal logging
    pub logging_events: SendPolicy,
}

/// How to act when an asynchronous event can't be sent.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SendPolicy {
    /// If set, wait for a timeout.
    pub blocking_timeout: Option<Duration>,

    /// If failed, issue a raw error to the console.
    pub console_fallback: bool,
}

impl Default for ObservedStateSettings {
    fn default() -> Self {
        Self {
            reporting_channel_size: 100,
            engine_events: SendPolicy {
                blocking_timeout: Some(Duration::from_millis(1)),
                console_fallback: true,
            },
            logging_events: SendPolicy {
                blocking_timeout: None,
                console_fallback: false,
            },
        }
    }
}

impl Default for SendPolicy {
    fn default() -> Self {
        // This is used in tests.
        Self {
            blocking_timeout: None,
            console_fallback: true,
        }
    }
}
