// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the temporal reaggregation processor.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Default aggregation period (60 seconds).
const DEFAULT_PERIOD_MS: u64 = 60_000;

fn default_period() -> Duration {
    Duration::from_millis(DEFAULT_PERIOD_MS)
}

/// Configuration for the temporal reaggregation processor.
///
/// # Example
///
/// ```yaml
/// config:
///   period: 60s
///   pass_through:
///     gauge: false
///     summary: false
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The interval at which the processor aggregates and emits metrics.
    /// Default: 60s.
    #[serde(with = "humantime_serde", default = "default_period")]
    pub period: Duration,

    /// Controls which metric types are passed through without aggregation.
    #[serde(default)]
    pub pass_through: PassThrough,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            period: default_period(),
            pass_through: PassThrough::default(),
        }
    }
}

/// Controls which metric types are passed through without aggregation.
///
/// Gauges and summaries involve actual data loss when aggregated (not just
/// resolution loss), so they can optionally be passed through unchanged.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PassThrough {
    /// If true, gauge metrics are passed through unchanged instead of
    /// being aggregated.
    #[serde(default)]
    pub gauge: bool,

    /// If true, summary metrics are passed through unchanged instead of
    /// being aggregated.
    #[serde(default)]
    pub summary: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.period, Duration::from_secs(60));
        assert!(!config.pass_through.gauge);
        assert!(!config.pass_through.summary);
    }

    #[test]
    fn test_deserialize_empty_config() {
        let config: Config = serde_json::from_value(json!({})).unwrap();
        assert_eq!(config.period, Duration::from_secs(60));
        assert!(!config.pass_through.gauge);
        assert!(!config.pass_through.summary);
    }

    #[test]
    fn test_deserialize_full_config() {
        let config: Config = serde_json::from_value(json!({
            "period": "30s",
            "pass_through": {
                "gauge": true,
                "summary": true
            }
        }))
        .unwrap();
        assert_eq!(config.period, Duration::from_secs(30));
        assert!(config.pass_through.gauge);
        assert!(config.pass_through.summary);
    }

    #[test]
    fn test_deserialize_partial_pass_through() {
        let config: Config = serde_json::from_value(json!({
            "period": "5s",
            "pass_through": {
                "gauge": true
            }
        }))
        .unwrap();
        assert_eq!(config.period, Duration::from_secs(5));
        assert!(config.pass_through.gauge);
        assert!(!config.pass_through.summary);
    }
}
