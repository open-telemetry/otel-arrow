// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the temporal reaggregation processor.

use std::time::Duration;

use otap_df_config::error::Error as ConfigError;
use serde::{Deserialize, Serialize};

const DEFAULT_PERIOD_SECONDS: u64 = 60;

const MIN_PERIOD_MILLIS: u64 = 100;

fn default_period() -> Duration {
    Duration::from_secs(DEFAULT_PERIOD_SECONDS)
}

/// Configuration for the temporal reaggregation processor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The interval at which the processor aggregates and emits metrics.
    /// Must be at least 100ms. Default: 60s.
    #[serde(with = "humantime_serde", default = "default_period")]
    pub period: Duration,
}

impl Config {
    /// Validates the configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.period < Duration::from_millis(MIN_PERIOD_MILLIS) {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "period must be at least {}ms, got {:?}",
                    MIN_PERIOD_MILLIS, self.period
                ),
            });
        }
        Ok(())
    }
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

    #[test]
    fn test_validate_minimum_period() {
        let reject_zero = Config {
            period: Duration::ZERO,
        };
        assert!(reject_zero.validate().is_err());

        let reject_sub_100ms = Config {
            period: Duration::from_millis(50),
        };
        assert!(reject_sub_100ms.validate().is_err());

        let accept_100ms = Config {
            period: Duration::from_millis(MIN_PERIOD_MILLIS),
        };
        assert!(accept_100ms.validate().is_ok());
    }
}
