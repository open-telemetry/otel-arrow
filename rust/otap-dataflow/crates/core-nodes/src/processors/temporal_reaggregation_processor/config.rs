// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the temporal reaggregation processor.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Default aggregation period (60 seconds).
const DEFAULT_PERIOD_SECONDS: u64 = 60;

fn default_period() -> Duration {
    Duration::from_secs(DEFAULT_PERIOD_SECONDS)
}

/// Configuration for the temporal reaggregation processor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The interval at which the processor aggregates and emits metrics.
    /// Default: 60s.
    #[serde(with = "humantime_serde", default = "default_period")]
    pub period: Duration,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            period: default_period(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.period, Duration::from_secs(60));
    }

    #[test]
    fn test_deserialize_empty_config() {
        let config: Config = serde_json::from_value(json!({})).unwrap();
        assert_eq!(config.period, Duration::from_secs(60));
    }

    #[test]
    fn test_deserialize_with_period() {
        let config: Config = serde_json::from_value(json!({
            "period": "30s"
        }))
        .unwrap();
        assert_eq!(config.period, Duration::from_secs(30));
    }
}
