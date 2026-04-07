// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the temporal reaggregation processor.

use std::num::{NonZeroU16, NonZeroUsize};
use std::time::Duration;

use otap_df_config::error::Error as ConfigError;
use serde::{Deserialize, Serialize};

const DEFAULT_PERIOD_SECONDS: u64 = 60;
const MIN_PERIOD_MILLIS: u64 = 100;
const MIN_INBOUND_REQUEST_LIMIT: usize = 1;
const MIN_OUTBOUND_REQUEST_LIMIT: usize = 2;

/// Configuration for the temporal reaggregation processor.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// The interval at which the processor aggregates and emits metrics.
    /// Must be at least [`MIN_PERIOD_MILLIS`]ms. Default: [`DEFAULT_PERIOD_SECONDS`]s.
    #[serde(with = "humantime_serde", default = "default_period")]
    pub period: Duration,

    /// Limits the number of pending requests for ack/nack tracking.
    #[serde(default = "default_inbound_request_limit")]
    pub inbound_request_limit: NonZeroUsize,

    /// Limits the number of outbound requests for ack/nack tracking.
    #[serde(default = "default_outbound_request_limit")]
    pub outbound_request_limit: NonZeroUsize,

    /// Maximum number of unique metric streams to track in a single aggregating
    /// batch. When exceeded, the current batch is flushed early.
    #[serde(default = "default_max_stream_cardinality")]
    pub max_stream_cardinality: NonZeroU16,
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

        if self.inbound_request_limit.get() < MIN_INBOUND_REQUEST_LIMIT {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "inbound_request_limit must be at least {}, got {}",
                    MIN_INBOUND_REQUEST_LIMIT,
                    self.inbound_request_limit.get()
                ),
            });
        }

        if self.outbound_request_limit.get() < MIN_OUTBOUND_REQUEST_LIMIT {
            return Err(ConfigError::InvalidUserConfig {
                error: format!(
                    "outbound_request_limit must be at least {}, got {}",
                    MIN_OUTBOUND_REQUEST_LIMIT,
                    self.outbound_request_limit.get()
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
            inbound_request_limit: default_inbound_request_limit(),
            outbound_request_limit: default_outbound_request_limit(),
            max_stream_cardinality: default_max_stream_cardinality(),
        }
    }
}

const fn default_inbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(1024).expect("ok")
}

const fn default_outbound_request_limit() -> NonZeroUsize {
    NonZeroUsize::new(2048).expect("ok")
}

const fn default_max_stream_cardinality() -> NonZeroU16 {
    NonZeroU16::new(u16::MAX / 4).expect("ok")
}

fn default_period() -> Duration {
    Duration::from_secs(DEFAULT_PERIOD_SECONDS)
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
    fn test_validate_minimum_inbound_request_limit() {
        // The exact minimum should be accepted.
        let accept_min = Config {
            inbound_request_limit: NonZeroUsize::new(MIN_INBOUND_REQUEST_LIMIT).unwrap(),
            ..Default::default()
        };
        assert!(accept_min.validate().is_ok());

        // The default should be accepted.
        let accept_default = Config::default();
        assert!(accept_default.validate().is_ok());

        // Note: MIN_INBOUND_REQUEST_LIMIT is 1 and the field is NonZeroUsize,
        // so it is impossible to construct a value below the minimum at runtime.
        // The type system enforces this constraint.
    }

    #[test]
    fn test_validate_minimum_outbound_request_limit() {
        // A value of 1 is below MIN_OUTBOUND_REQUEST_LIMIT (2) and should be rejected.
        let reject_below_min = Config {
            outbound_request_limit: NonZeroUsize::new(1).unwrap(),
            ..Default::default()
        };
        assert!(reject_below_min.validate().is_err());

        // The exact minimum should be accepted.
        let accept_min = Config {
            outbound_request_limit: NonZeroUsize::new(MIN_OUTBOUND_REQUEST_LIMIT).unwrap(),
            ..Default::default()
        };
        assert!(accept_min.validate().is_ok());

        // A value above the minimum should be accepted.
        let accept_above_min = Config {
            outbound_request_limit: NonZeroUsize::new(MIN_OUTBOUND_REQUEST_LIMIT + 1).unwrap(),
            ..Default::default()
        };
        assert!(accept_above_min.validate().is_ok());

        // The default should be accepted.
        let accept_default = Config::default();
        assert!(accept_default.validate().is_ok());
    }

    #[test]
    fn test_deserialize_with_max_stream_cardinality() {
        let config: Config = serde_json::from_value(json!({
            "max_stream_cardinality": 100
        }))
        .unwrap();
        assert_eq!(config.max_stream_cardinality.get(), 100);
    }

    #[test]
    fn test_deserialize_rejects_zero_max_stream_cardinality() {
        let result: Result<Config, _> = serde_json::from_value(json!({
            "max_stream_cardinality": 0
        }));
        assert!(result.is_err(), "zero max_stream_cardinality should fail");
    }

    #[test]
    fn test_validate_minimum_period() {
        let reject_zero = Config {
            period: Duration::ZERO,
            ..Default::default()
        };
        assert!(reject_zero.validate().is_err());

        let reject_sub_100ms = Config {
            period: Duration::from_millis(50),
            ..Default::default()
        };
        assert!(reject_sub_100ms.validate().is_err());

        let accept_100ms = Config {
            period: Duration::from_millis(MIN_PERIOD_MILLIS),
            ..Default::default()
        };
        assert!(accept_100ms.validate().is_ok());
    }
}
