// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration of the observed store.

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// Configuration for the observed state store.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The size of the reporting channel.
    pub reporting_channel_size: usize,

    /// The max duration to wait when reporting an observed event.
    pub reporting_timeout: Duration,

    /// Maximum allowed silence before a heartbeat is considered missing.
    #[serde(default = "default_heartbeat_timeout")]
    pub heartbeat_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reporting_channel_size: 100,
            reporting_timeout: Duration::from_millis(1),
            heartbeat_timeout: default_heartbeat_timeout(),
        }
    }
}

const fn default_heartbeat_timeout() -> Duration {
    Duration::from_secs(30)
}
