// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Telemetry configuration specification.

pub mod metrics;

use serde::{Deserialize, Serialize};
use schemars::JsonSchema;

/// Configuration for the telemetry metrics system.
#[derive(Clone, Serialize, Deserialize, Debug, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct TelemetryConfig {
    /// The size of the reporting channel.
    #[serde(default = "default_reporting_channel_size")]
    pub reporting_channel_size: usize,

    /// Configuration for the metrics dispatcher.
    #[serde(default)]
    pub metrics: metrics::MetricsDispatcherConfig,

    // TODO: Add tracing and events configuration.    
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            reporting_channel_size: default_reporting_channel_size(),
            metrics: metrics::MetricsDispatcherConfig::default(),
        }
    }
}

/// Default reporting channel size.
fn default_reporting_channel_size() -> usize {
    100
}
