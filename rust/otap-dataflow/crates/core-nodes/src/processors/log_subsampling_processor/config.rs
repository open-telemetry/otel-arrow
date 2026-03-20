//! Configuration for the log subsampling processor.
//!
//! Supports two subsampling policies:
//! - **Zip**: Emit at most N log records per time window
//! - **Ratio**: Emit a fixed fraction of log records

use otap_df_config::error::Error as ConfigError;
use serde::Deserialize;
use std::time::Duration;

/// Configuration for the log subsampling processor.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The subsampling policy to apply.
    pub policy: Policy,
}

/// Subsampling policy configuration.
///
/// Exactly one policy must be specified (externally tagged enum).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Policy {
    /// Emit at most N log records per time window.
    Zip(ZipConfig),
    /// Emit a fixed fraction of log records.
    Ratio(RatioConfig),
}

/// Configuration for zip sampling.
///
/// Emits at most `max_items` log records per `interval` time window.
#[derive(Debug, Clone, Deserialize)]
pub struct ZipConfig {
    /// Length of the sampling window (e.g., "60s", "1m").
    #[serde(with = "humantime_serde")]
    pub interval: Duration,
    /// Maximum records to emit per window.
    pub max_items: usize,
}

/// Configuration for ratio sampling.
///
/// Emits `emit` out of every `out_of` log records.
#[derive(Debug, Clone, Deserialize)]
pub struct RatioConfig {
    /// Numerator of the sampling fraction.
    pub emit: usize,
    /// Denominator of the sampling fraction.
    pub out_of: usize,
}

impl Config {
    /// Validates the configuration.
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration values are invalid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        match &self.policy {
            Policy::Zip(zip) => {
                if zip.interval.is_zero() {
                    return Err(ConfigError::InvalidUserConfig {
                        error: "zip.interval must be greater than 0".to_string(),
                    });
                }
                if zip.max_items == 0 {
                    return Err(ConfigError::InvalidUserConfig {
                        error: "zip.max_items must be greater than 0".to_string(),
                    });
                }
            }
            Policy::Ratio(ratio) => {
                if ratio.emit == 0 {
                    return Err(ConfigError::InvalidUserConfig {
                        error: "ratio.emit must be greater than 0".to_string(),
                    });
                }
                if ratio.out_of == 0 {
                    return Err(ConfigError::InvalidUserConfig {
                        error: "ratio.out_of must be greater than 0".to_string(),
                    });
                }
                if ratio.emit > ratio.out_of {
                    return Err(ConfigError::InvalidUserConfig {
                        error: "ratio.emit must be less than or equal to ratio.out_of".to_string(),
                    });
                }
            }
        }
        Ok(())
    }
}

/// Serde helper for human-readable durations.
mod humantime_serde {
    use serde::{Deserialize, Deserializer, de};
    use std::time::Duration;

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Duration, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        humantime::parse_duration(&s).map_err(de::Error::custom)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_zip_config() {
        let config = Config {
            policy: Policy::Zip(ZipConfig {
                interval: Duration::from_secs(60),
                max_items: 100,
            }),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_zip_interval_zero() {
        let config = Config {
            policy: Policy::Zip(ZipConfig {
                interval: Duration::ZERO,
                max_items: 100,
            }),
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("interval"));
    }

    #[test]
    fn test_invalid_zip_max_items_zero() {
        let config = Config {
            policy: Policy::Zip(ZipConfig {
                interval: Duration::from_secs(60),
                max_items: 0,
            }),
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("max_items"));
    }

    #[test]
    fn test_valid_ratio_config() {
        let config = Config {
            policy: Policy::Ratio(RatioConfig {
                emit: 1,
                out_of: 10,
            }),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_valid_ratio_config_equal() {
        let config = Config {
            policy: Policy::Ratio(RatioConfig { emit: 1, out_of: 1 }),
        };
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_invalid_ratio_emit_zero() {
        let config = Config {
            policy: Policy::Ratio(RatioConfig {
                emit: 0,
                out_of: 10,
            }),
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("emit"));
    }

    #[test]
    fn test_invalid_ratio_out_of_zero() {
        let config = Config {
            policy: Policy::Ratio(RatioConfig { emit: 1, out_of: 0 }),
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("out_of"));
    }

    #[test]
    fn test_invalid_ratio_emit_greater_than_out_of() {
        let config = Config {
            policy: Policy::Ratio(RatioConfig {
                emit: 10,
                out_of: 5,
            }),
        };
        let err = config.validate().unwrap_err();
        assert!(err.to_string().contains("emit"));
    }
}
