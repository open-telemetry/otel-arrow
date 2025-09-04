// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration of the observed store.

use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Configuration for the observed store system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The size of the reporting channel.
    pub reporting_channel_size: usize,

    /// The max duration to wait when reporting an observed event.
    pub reporting_timeout: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reporting_channel_size: 100,
            reporting_timeout: Duration::from_millis(1),
        }
    }
}
