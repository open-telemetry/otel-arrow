//! Configuration for the log subsampling processor.
//!
//! Supports two subsampling policies:
//! - **Zip**: Emit at most N log records per time window
//! - **Ratio**: Emit a fixed fraction of log records

use super::sample::{RatioConfig, ZipConfig};
use otap_df_config::error::Error as ConfigError;
use serde::Deserialize;

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

impl Config {
    /// Validates the configuration by delegating to the active policy's
    /// config type.
    ///
    /// # Errors
    ///
    /// Returns an error if any configuration values are invalid.
    pub fn validate(&self) -> Result<(), ConfigError> {
        match &self.policy {
            Policy::Zip(cfg) => cfg.validate(),
            Policy::Ratio(cfg) => cfg.validate(),
        }
    }
}
