// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the metric start time processor.

use std::time::Duration;

use otel_arrow_dfe_config::error::Error as ConfigError;
use serde::{Deserialize, Serialize};

/// Default garbage collection interval (10 minutes), matching the Go collector's
/// `metricstarttimeprocessor` default.
pub const DEFAULT_GC_INTERVAL: Duration = Duration::from_secs(10 * 60);

/// Strategy for adjusting metric start times on cumulative data points.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// Drops the first observed point per cumulative series, sets subsequent points'
    /// `start_time_unix_nano` to the initial point's timestamp, and subtracts the initial
    /// reference values from subsequent cumulative points until a counter reset is detected.
    #[default]
    SubtractInitialPoint,
}

/// Configuration for the metric start time processor.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Adjustment strategy to apply. Defaults to `subtract_initial_point`.
    #[serde(default)]
    pub strategy: Strategy,

    /// How long to wait before sweeping cached series and resources that have not been seen
    /// since the previous sweep. Must be strictly positive. Defaults to `10m`.
    #[serde(with = "humantime_serde", default = "default_gc_interval")]
    pub gc_interval: Duration,
}

impl Config {
    /// Validates the processor configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        if self.gc_interval.is_zero() {
            return Err(ConfigError::InvalidUserConfig {
                error: "gc_interval must be greater than zero".to_string(),
            });
        }
        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            strategy: Strategy::default(),
            gc_interval: default_gc_interval(),
        }
    }
}

const fn default_gc_interval() -> Duration {
    DEFAULT_GC_INTERVAL
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_default_config() {
        let cfg = Config::default();
        assert_eq!(cfg.strategy, Strategy::SubtractInitialPoint);
        assert_eq!(cfg.gc_interval, Duration::from_secs(600));
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_deserialize_empty_config() {
        let cfg: Config = serde_json::from_value(json!({})).unwrap();
        assert_eq!(cfg.strategy, Strategy::SubtractInitialPoint);
        assert_eq!(cfg.gc_interval, Duration::from_secs(600));
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_deserialize_custom_config() {
        let cfg: Config = serde_json::from_value(json!({
            "strategy": "subtract_initial_point",
            "gc_interval": "5m"
        }))
        .unwrap();
        assert_eq!(cfg.strategy, Strategy::SubtractInitialPoint);
        assert_eq!(cfg.gc_interval, Duration::from_secs(300));
        assert!(cfg.validate().is_ok());
    }

    #[test]
    fn test_validate_rejects_zero_gc_interval() {
        let cfg: Config = serde_json::from_value(json!({
            "gc_interval": "0s"
        }))
        .unwrap();
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn test_deserialize_rejects_unknown_strategy() {
        let res: Result<Config, _> = serde_json::from_value(json!({
            "strategy": "unknown_strategy"
        }));
        assert!(res.is_err());
    }
}
