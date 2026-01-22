// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the persistence processor.

use std::path::PathBuf;
use std::time::Duration;

use quiver::config::RetentionPolicy;
use serde::Deserialize;

use otap_df_config::error::Error as ConfigError;

/// Default retention size cap (500 GB).
fn default_retention_size_cap() -> String {
    "500GB".to_string()
}

/// Default size cap policy.
fn default_size_cap_policy() -> SizeCapPolicy {
    SizeCapPolicy::Backpressure
}

/// Default poll interval for checking Quiver for bundles.
fn default_poll_interval() -> Duration {
    Duration::from_millis(100)
}

/// Default OTLP handling mode.
fn default_otlp_handling() -> OtlpHandling {
    OtlpHandling::PassThrough
}

/// Default maximum segment duration (5 seconds).
fn default_max_segment_duration() -> Duration {
    Duration::from_secs(5)
}

/// Default maximum bundles to forward per timer tick.
///
/// Limits how many bundles are sent downstream per tick to prevent
/// the processor from blocking on downstream backpressure and being
/// unable to receive new data.
fn default_max_bundles_per_tick() -> usize {
    100
}

/// How to handle incoming OTLP data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OtlpHandling {
    /// Store OTLP bytes as opaque binary (default).
    /// Most CPU-efficient for OTLP-to-OTLP pipelines.
    /// Data is not queryable but pass-through is very fast.
    #[default]
    PassThrough,
    /// Convert OTLP to Arrow format before storing.
    /// More CPU overhead but enables querying/filtering on stored data.
    ConvertToArrow,
}

/// Size cap policy when retention limit is reached.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SizeCapPolicy {
    /// Apply backpressure when at capacity (default, no data loss).
    #[default]
    Backpressure,
    /// Drop oldest data when at capacity (allows controlled data loss).
    DropOldest,
}

impl From<SizeCapPolicy> for RetentionPolicy {
    fn from(policy: SizeCapPolicy) -> Self {
        match policy {
            SizeCapPolicy::Backpressure => RetentionPolicy::Backpressure,
            SizeCapPolicy::DropOldest => RetentionPolicy::DropOldest,
        }
    }
}

/// Configuration for the persistence processor.
#[derive(Debug, Clone, Deserialize)]
pub struct PersistenceProcessorConfig {
    /// Directory path for persistent storage.
    pub path: PathBuf,

    /// Maximum disk space to use (e.g., "500GB", "100MB").
    #[serde(default = "default_retention_size_cap")]
    pub retention_size_cap: String,

    /// Maximum age of data to retain (e.g., "24h", "7d").
    /// When set, data older than this will be eligible for removal.
    #[serde(with = "humantime_serde", default)]
    pub max_age: Option<Duration>,

    /// Policy when size cap is reached.
    #[serde(default = "default_size_cap_policy")]
    pub size_cap_policy: SizeCapPolicy,

    /// Interval for polling Quiver for available bundles.
    #[serde(with = "humantime_serde", default = "default_poll_interval")]
    pub poll_interval: Duration,

    /// How to handle incoming OTLP data.
    /// Default: `pass_through` (store as opaque bytes for efficiency).
    #[serde(default = "default_otlp_handling")]
    pub otlp_handling: OtlpHandling,

    /// Maximum duration a segment stays open before automatic finalization.
    ///
    /// Data becomes visible to downstream only after segment finalization.
    /// Lower values reduce latency but increase I/O overhead.
    /// Default: 5 seconds.
    #[serde(with = "humantime_serde", default = "default_max_segment_duration")]
    pub max_segment_duration: Duration,

    /// Maximum number of bundles to forward downstream per timer tick.
    ///
    /// This prevents the processor from blocking indefinitely when downstream
    /// applies backpressure. Remaining bundles will be picked up on the next
    /// timer tick. Setting to 0 disables the limit (drains all available bundles).
    /// Default: 100.
    #[serde(default = "default_max_bundles_per_tick")]
    pub max_bundles_per_tick: usize,
}

impl PersistenceProcessorConfig {
    /// Parse the retention size cap string into bytes.
    pub fn parse_size_cap(&self) -> Result<u64, ConfigError> {
        parse_size_string(&self.retention_size_cap).map_err(|e| ConfigError::InvalidUserConfig {
            error: format!("invalid retention_size_cap: {}", e),
        })
    }

    /// Get the retention policy for Quiver.
    #[must_use]
    pub fn retention_policy(&self) -> RetentionPolicy {
        self.size_cap_policy.into()
    }
}

/// Parse a human-readable size string like "500GB" into bytes.
fn parse_size_string(s: &str) -> Result<u64, String> {
    let s = s.trim().to_uppercase();

    let (num_str, multiplier) = if s.ends_with("TB") {
        (&s[..s.len() - 2], 1024u64 * 1024 * 1024 * 1024)
    } else if s.ends_with("GB") {
        (&s[..s.len() - 2], 1024u64 * 1024 * 1024)
    } else if s.ends_with("MB") {
        (&s[..s.len() - 2], 1024u64 * 1024)
    } else if s.ends_with("KB") {
        (&s[..s.len() - 2], 1024u64)
    } else if s.ends_with('B') {
        (&s[..s.len() - 1], 1u64)
    } else {
        // Assume bytes if no suffix
        (s.as_str(), 1u64)
    };

    let num: f64 = num_str
        .trim()
        .parse()
        .map_err(|_| format!("invalid number: {}", num_str))?;

    Ok((num * multiplier as f64) as u64)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_size_string() {
        assert_eq!(
            parse_size_string("500GB").unwrap(),
            500 * 1024 * 1024 * 1024
        );
        assert_eq!(parse_size_string("100MB").unwrap(), 100 * 1024 * 1024);
        assert_eq!(parse_size_string("1TB").unwrap(), 1024 * 1024 * 1024 * 1024);
        assert_eq!(parse_size_string("1024KB").unwrap(), 1024 * 1024);
        assert_eq!(parse_size_string("1024B").unwrap(), 1024);
        assert_eq!(parse_size_string("1024").unwrap(), 1024);
        assert_eq!(
            parse_size_string("1.5GB").unwrap(),
            (1.5 * 1024.0 * 1024.0 * 1024.0) as u64
        );
    }

    #[test]
    fn test_size_cap_policy_serde() {
        let json = r#"{"path": "/tmp/test", "size_cap_policy": "drop_oldest"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.size_cap_policy, SizeCapPolicy::DropOldest);

        let json = r#"{"path": "/tmp/test", "size_cap_policy": "backpressure"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.size_cap_policy, SizeCapPolicy::Backpressure);
    }

    #[test]
    fn test_config_defaults() {
        let json = r#"{"path": "/tmp/test"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.retention_size_cap, "500GB");
        assert_eq!(config.size_cap_policy, SizeCapPolicy::Backpressure);
        assert_eq!(config.poll_interval, Duration::from_millis(100));
        assert!(config.max_age.is_none());
        // Default OTLP handling is pass_through
        assert_eq!(config.otlp_handling, OtlpHandling::PassThrough);
    }

    #[test]
    fn test_otlp_handling_serde() {
        let json = r#"{"path": "/tmp/test", "otlp_handling": "pass_through"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.otlp_handling, OtlpHandling::PassThrough);

        let json = r#"{"path": "/tmp/test", "otlp_handling": "convert_to_arrow"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.otlp_handling, OtlpHandling::ConvertToArrow);
    }
}
