// SPDX-License-Identifier: Apache-2.0

//! Configuration of the metric system.

use serde::{Deserialize, Serialize};

/// Configuration for the telemetry metrics system.
#[derive(Clone, Serialize, Deserialize)]
pub struct Config {
    /// The size of the reporting channel.
    pub reporting_channel_size: usize,
    /// The interval at which metrics are flushed and aggregated by the collector.
    pub flush_interval: std::time::Duration,
}

/// Configuration for the HTTP server that exposes telemetry endpoints.
#[derive(Clone, Serialize, Deserialize)]
pub struct HttpServerConfig {
    /// The address to bind the HTTP server to (e.g., "127.0.0.1:8080").
    pub bind_address: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            reporting_channel_size: 100,
            flush_interval: std::time::Duration::from_secs(1),
        }
    }
}