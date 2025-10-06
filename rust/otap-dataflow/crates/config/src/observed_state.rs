// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration of the observed store.

use serde::{Deserialize, Serialize};
use std::time::Duration;
use schemars::JsonSchema;

/// Configuration for the observed state store.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ObservedStateSettings {
    /// The size of the reporting channel.
    pub reporting_channel_size: usize,

    /// The max duration to wait when reporting an observed event.
    pub reporting_timeout: Duration,
}

impl Default for ObservedStateSettings {
    fn default() -> Self {
        Self {
            reporting_channel_size: 100,
            reporting_timeout: Duration::from_millis(1),
        }
    }
}
