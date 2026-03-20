//! Configuration for the log sampling processor.

use super::samplers::{RatioConfig, ZipConfig};
use otap_df_config::error::Error as ConfigError;
use serde::Deserialize;

/// Configuration for the log sampling processor.
#[derive(Debug, Clone, Deserialize)]
pub struct Config {
    /// The sampling policy to apply.
    pub policy: Policy,
}

/// Sampling policy configuration.
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
