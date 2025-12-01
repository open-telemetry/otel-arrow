// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration structures for Quiver's persistence engine.

use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{QuiverError, Result};

/// Top-level configuration for the persistence engine.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuiverConfig {
    /// Write-ahead log tuning parameters.
    pub wal: WalConfig,
    /// Segment creation tuning parameters.
    pub segment: SegmentConfig,
    /// Retention controls for finalized data.
    pub retention: RetentionConfig,
    /// Optional override for the base data directory.
    pub data_dir: PathBuf,
}

impl QuiverConfig {
    /// Returns a builder seeded with [`Default`] values.
    #[must_use]
    pub fn builder() -> QuiverConfigBuilder {
        QuiverConfigBuilder::default()
    }

    /// Ensures the configuration holds sane, non-zero values.
    pub fn validate(&self) -> Result<()> {
        self.wal.validate()?;
        self.segment.validate()?;
        self.retention.validate()?;

        if self.data_dir.as_os_str().is_empty() {
            return Err(QuiverError::invalid_config("data_dir must not be empty"));
        }

        Ok(())
    }

    /// Convenience helper that clones the configuration and replaces the
    /// `data_dir` while keeping other settings intact.
    pub fn with_data_dir<P: AsRef<Path>>(&self, data_dir: P) -> Self {
        let mut clone = self.clone();
        clone.data_dir = data_dir.as_ref().to_path_buf();
        clone
    }
}

impl Default for QuiverConfig {
    fn default() -> Self {
        Self {
            wal: WalConfig::default(),
            segment: SegmentConfig::default(),
            retention: RetentionConfig::default(),
            data_dir: PathBuf::from("./quiver_data"),
        }
    }
}

/// Builder wrapper to allow incremental overrides while keeping future fields
/// backwards compatible.
#[derive(Debug, Default)]
pub struct QuiverConfigBuilder {
    wal: WalConfig,
    segment: SegmentConfig,
    retention: RetentionConfig,
    data_dir: PathBuf,
}

impl QuiverConfigBuilder {
    /// Applies a custom WAL configuration.
    #[must_use]
    pub fn wal(mut self, wal: WalConfig) -> Self {
        self.wal = wal;
        self
    }

    /// Applies a custom segment configuration.
    #[must_use]
    pub fn segment(mut self, segment: SegmentConfig) -> Self {
        self.segment = segment;
        self
    }

    /// Applies a custom retention configuration.
    #[must_use]
    pub fn retention(mut self, retention: RetentionConfig) -> Self {
        self.retention = retention;
        self
    }

    /// Overrides the storage directory.
    #[must_use]
    pub fn data_dir<P: AsRef<Path>>(mut self, data_dir: P) -> Self {
        self.data_dir = data_dir.as_ref().to_path_buf();
        self
    }

    /// Consumes the builder and validates the resulting configuration.
    pub fn build(self) -> Result<QuiverConfig> {
        let cfg = QuiverConfig {
            wal: self.wal,
            segment: self.segment,
            retention: self.retention,
            data_dir: if self.data_dir.as_os_str().is_empty() {
                PathBuf::from("./quiver_data")
            } else {
                self.data_dir
            },
        };
        cfg.validate()?;
        Ok(cfg)
    }
}

/// Write-ahead-log related controls.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WalConfig {
    /// Maximum on-disk footprint (across active + rotated chunks).
    pub max_size_bytes: NonZeroU64,
    /// Maximum number of chunk files retained during rotation.
    pub max_chunk_count: u16,
    /// Preferred fsync cadence for durability vs. latency.
    pub flush_interval: Duration,
}

impl WalConfig {
    fn validate(&self) -> Result<()> {
        if self.max_chunk_count == 0 {
            return Err(QuiverError::invalid_config(
                "max_chunk_count must be at least 1",
            ));
        }
        Ok(())
    }
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            max_size_bytes: NonZeroU64::new(4 * 1024 * 1024 * 1024).expect("non-zero"),
            max_chunk_count: 10,
            flush_interval: Duration::from_millis(25),
        }
    }
}

/// Segment tuning parameters.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentConfig {
    /// Target payload size for a finalized segment.
    pub target_size_bytes: NonZeroU64,
    /// Optional time-slicing to prevent long-lived open segments.
    pub max_open_duration: Duration,
    /// Soft cap on the number of streams active within a segment.
    pub max_stream_count: u32,
}

impl SegmentConfig {
    fn validate(&self) -> Result<()> {
        if self.max_stream_count == 0 {
            return Err(QuiverError::invalid_config(
                "max_stream_count must be at least 1",
            ));
        }
        Ok(())
    }
}

impl Default for SegmentConfig {
    fn default() -> Self {
        Self {
            target_size_bytes: NonZeroU64::new(32 * 1024 * 1024).expect("non-zero"),
            max_open_duration: Duration::from_secs(5),
            max_stream_count: 512,
        }
    }
}

/// Policy that determines how storage pressure is relieved.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum RetentionPolicy {
    /// Strict durability: pause ingestion instead of deleting unprocessed data.
    #[default]
    Backpressure,
    /// Trade durability for throughput by deleting the oldest data first.
    DropOldest,
}

/// Retention-related configuration.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RetentionConfig {
    /// Total bytes allowed for finalized segments on disk.
    pub size_cap_bytes: NonZeroU64,
    /// Policy applied when usage exceeds [`RetentionConfig::size_cap_bytes`].
    pub policy: RetentionPolicy,
    /// Optional maximum wall-clock retention irrespective of size.
    pub max_age: Duration,
}

impl RetentionConfig {
    fn validate(&self) -> Result<()> {
        if self.max_age.is_zero() {
            return Err(QuiverError::invalid_config(
                "max_age must be greater than zero",
            ));
        }
        Ok(())
    }
}

impl Default for RetentionConfig {
    fn default() -> Self {
        Self {
            size_cap_bytes: NonZeroU64::new(500 * 1024 * 1024 * 1024).expect("non-zero"),
            policy: RetentionPolicy::Backpressure,
            max_age: Duration::from_secs(72 * 60 * 60),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_round_trip() {
        let cfg = QuiverConfig::builder()
            .data_dir("/tmp/quiver-test")
            .build()
            .expect("config builds");
        assert_eq!(cfg.data_dir, PathBuf::from("/tmp/quiver-test"));
    }

    #[test]
    fn invalid_policy_is_rejected() {
        let cfg = QuiverConfig {
            retention: RetentionConfig {
                max_age: Duration::ZERO,
                ..RetentionConfig::default()
            },
            ..QuiverConfig::default()
        };
        assert!(cfg.validate().is_err());
    }
}
