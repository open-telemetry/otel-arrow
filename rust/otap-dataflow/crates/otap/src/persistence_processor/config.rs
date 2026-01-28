// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration for the persistence processor.
//!
//! # Dispatch Strategy
//!
//! The persistence processor should be connected with `RoundRobin` (or `Random`/`LeastLoaded`)
//! dispatch on its incoming edge. Using `Broadcast` will cause each message to be persisted
//! multiple times (once per core), leading to:
//! - N× storage consumption
//! - N× duplicate messages forwarded downstream
//!
//! See the [module documentation](super) for more details.

use std::path::PathBuf;
use std::time::Duration;

use byte_unit::{Byte, Unit};
use quiver::config::RetentionPolicy;
use serde::Deserialize;

/// Default retention size cap (10 GiB).
fn default_retention_size_cap() -> Byte {
    Byte::from_u64_with_unit(10, Unit::GiB).expect("valid constant")
}

/// Default size cap policy.
const fn default_size_cap_policy() -> SizeCapPolicy {
    SizeCapPolicy::Backpressure
}

/// Default poll interval for checking for available bundles.
const fn default_poll_interval() -> Duration {
    Duration::from_millis(100)
}

/// Default OTLP handling mode.
const fn default_otlp_handling() -> OtlpHandling {
    OtlpHandling::PassThrough
}

/// Default maximum time a segment can stay open (1 seconds).
const fn default_max_segment_open_duration() -> Duration {
    Duration::from_secs(1)
}

/// Default initial retry interval (1 second).
const fn default_initial_retry_interval() -> Duration {
    Duration::from_secs(1)
}

/// Default maximum retry interval (30 seconds).
const fn default_max_retry_interval() -> Duration {
    Duration::from_secs(30)
}

/// Default retry backoff multiplier.
const fn default_retry_multiplier() -> f64 {
    2.0
}

/// Default maximum bundles in-flight to downstream.
const fn default_max_in_flight() -> usize {
    1000
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

    /// Maximum total disk space to use across all cores (e.g., "500 GiB", "100 MB").
    /// Supports both IEC (KiB, MiB, GiB) and SI (KB, MB, GB) units.
    ///
    /// This is the **total** disk budget for the pipeline. The processor automatically
    /// divides this across all cores (each core gets `retention_size_cap / num_cores`).
    /// For example, with a 10 GiB cap on a 4-core pipeline, each core gets ~2.5 GiB.
    #[serde(default = "default_retention_size_cap")]
    pub retention_size_cap: Byte,

    /// Maximum age of data to retain (e.g., "24h", "7d").
    /// When set, data older than this will be eligible for removal.
    /// TODO: This setting is currently a placeholder and is not yet enforced.
    #[serde(with = "humantime_serde", default)]
    pub max_age: Option<Duration>,

    /// Policy when size cap is reached.
    #[serde(default = "default_size_cap_policy")]
    pub size_cap_policy: SizeCapPolicy,

    /// Interval for polling for available bundles.
    #[serde(with = "humantime_serde", default = "default_poll_interval")]
    pub poll_interval: Duration,

    /// How to handle incoming OTLP data.
    /// Default: `pass_through` (store as opaque bytes for efficiency).
    #[serde(default = "default_otlp_handling")]
    pub otlp_handling: OtlpHandling,

    /// Maximum time a segment can stay open before automatic finalization.
    ///
    /// Data becomes visible to downstream only after segment finalization.
    /// Lower values reduce latency but increase I/O overhead.
    #[serde(
        with = "humantime_serde",
        default = "default_max_segment_open_duration"
    )]
    pub max_segment_open_duration: Duration,

    // ─── Retry Configuration ────────────────────────────────────────────────
    /// Initial retry delay after first NACK (default: 1s).
    ///
    /// Used as the base for exponential backoff calculation.
    #[serde(with = "humantime_serde", default = "default_initial_retry_interval")]
    pub initial_retry_interval: Duration,

    /// Maximum retry delay (default: 30s).
    ///
    /// Caps the exponential backoff to prevent excessively long waits.
    #[serde(with = "humantime_serde", default = "default_max_retry_interval")]
    pub max_retry_interval: Duration,

    /// Multiplier for exponential backoff (default: 2.0).
    ///
    /// Each retry delay is: min(initial * multiplier^retry_count, max_interval).
    #[serde(default = "default_retry_multiplier")]
    pub retry_multiplier: f64,

    /// Maximum bundles in-flight to downstream (default: 1000).
    ///
    /// Limits concurrent downstream delivery attempts to prevent thundering
    /// herd problems after extended outages or restarts with large backlogs.
    /// When at limit, new sends are deferred until ACKs free up slots.
    #[serde(default = "default_max_in_flight")]
    pub max_in_flight: usize,
}

impl PersistenceProcessorConfig {
    /// Get the retention size cap in bytes.
    #[must_use]
    pub const fn size_cap_bytes(&self) -> u64 {
        self.retention_size_cap.as_u64()
    }

    /// Get the retention policy for the storage engine.
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
        // Default is 10 GiB
        assert_eq!(config.size_cap_bytes(), 10 * 1024 * 1024 * 1024);
        assert_eq!(config.size_cap_policy, SizeCapPolicy::Backpressure);
        assert_eq!(config.poll_interval, Duration::from_millis(100));
        assert!(config.max_age.is_none());
        // Default OTLP handling is pass_through
        assert_eq!(config.otlp_handling, OtlpHandling::PassThrough);
        // Default retry configuration
        assert_eq!(config.initial_retry_interval, Duration::from_secs(1));
        assert_eq!(config.max_retry_interval, Duration::from_secs(30));
        assert!((config.retry_multiplier - 2.0).abs() < f64::EPSILON);
        assert_eq!(config.max_in_flight, 1000);
    }

    #[test]
    fn test_retry_config_serde() {
        let json = r#"{
            "path": "/tmp/test",
            "initial_retry_interval": "500ms",
            "max_retry_interval": "1m",
            "retry_multiplier": 1.5,
            "max_in_flight": 500
        }"#;
        let config: PersistenceProcessorConfig = serde_json::from_str(json).unwrap();
        assert_eq!(config.initial_retry_interval, Duration::from_millis(500));
        assert_eq!(config.max_retry_interval, Duration::from_secs(60));
        assert!((config.retry_multiplier - 1.5).abs() < f64::EPSILON);
        assert_eq!(config.max_in_flight, 500);
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
