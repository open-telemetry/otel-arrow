// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration of the metric system.

use serde::{Deserialize, Serialize};

/// Configuration for the telemetry metrics system.
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The size of the reporting channel.
    pub reporting_channel_size: usize,
    /// The interval at which metrics are flushed and aggregated by the collector.
    pub flush_interval: std::time::Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reporting_channel_size: 100,
            flush_interval: std::time::Duration::from_secs(1),
        }
    }
}
