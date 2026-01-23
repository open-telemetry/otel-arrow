// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the persistence processor.

use std::path::PathBuf;
use std::time::Duration;

use byte_unit::{Byte, Unit};
use quiver::config::RetentionPolicy;
use serde::Deserialize;

/// Default retention size cap (500 GiB).
fn default_retention_size_cap() -> Byte {
    Byte::from_u64_with_unit(500, Unit::GiB).expect("valid constant")
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

    /// Maximum disk space to use (e.g., "500 GiB", "100 MB", or raw bytes).
    /// Supports both IEC (KiB, MiB, GiB) and SI (KB, MB, GB) units.
    #[serde(default = "default_retention_size_cap")]
    pub retention_size_cap: Byte,

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
    /// Get the retention size cap in bytes.
    #[must_use]
    pub fn size_cap_bytes(&self) -> u64 {
        self.retention_size_cap.as_u64()
    }

    /// Get the retention policy for Quiver.
    #[must_use]
    pub fn retention_policy(&self) -> RetentionPolicy {
        self.size_cap_policy.into()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size_cap_serde() {
        // Test string with IEC units
        let json = r#"{"path": "/tmp/test", "retention_size_cap": "500 GiB"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.size_cap_bytes(), 500 * 1024 * 1024 * 1024);

        // Test string with SI units
        let json = r#"{"path": "/tmp/test", "retention_size_cap": "100 MB"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.size_cap_bytes(), 100 * 1000 * 1000);

        // Test raw bytes as string
        let json = r#"{"path": "/tmp/test", "retention_size_cap": "1073741824"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.size_cap_bytes(), 1024 * 1024 * 1024);

        // Test decimal values
        let json = r#"{"path": "/tmp/test", "retention_size_cap": "1.5 GiB"}"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(
            config.size_cap_bytes(),
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
        // Default is 500 GiB
        assert_eq!(config.size_cap_bytes(), 500 * 1024 * 1024 * 1024);
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
