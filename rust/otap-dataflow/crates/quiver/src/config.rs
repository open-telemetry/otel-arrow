// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration structures for Quiver's persistence engine.

use std::num::NonZeroU64;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::error::{QuiverError, Result};
use crate::segment_store::SegmentReadMode;

/// Controls the durability/throughput tradeoff for ingested data.
///
/// This determines whether the write-ahead log (WAL) is used to protect
/// data in the open segment before it is finalized to disk.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum DurabilityMode {
    /// Full WAL protection: each bundle is written to the WAL before acknowledgement.
    ///
    /// On crash, only bundles written since the last WAL fsync are lost
    /// (controlled by [`WalConfig::flush_interval`]).
    ///
    /// This is the safest option but has lower throughput (~125 bundles/sec typical).
    #[default]
    Wal,

    /// Segment-only durability: WAL is disabled, data is only durable after
    /// segment finalization.
    ///
    /// On crash, the entire open segment is lost (potentially thousands of bundles).
    /// Provides ~3x higher throughput than WAL mode.
    ///
    /// Use this when:
    /// - Throughput is more important than durability
    /// - Data can be re-fetched from upstream on crash
    /// - You have other durability guarantees (e.g., upstream acknowledgement)
    SegmentOnly,
}

/// Top-level configuration for the persistence engine.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct QuiverConfig {
    /// Controls durability vs throughput tradeoff.
    pub durability: DurabilityMode,
    /// Write-ahead log tuning parameters (ignored when durability is `SegmentOnly`).
    pub wal: WalConfig,
    /// Segment creation tuning parameters.
    pub segment: SegmentConfig,
    /// Retention controls for finalized data.
    pub retention: RetentionConfig,
    /// Read mode for segment files (mmap vs standard I/O).
    pub read_mode: SegmentReadMode,
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
        // Only validate WAL config if WAL is enabled
        if self.durability == DurabilityMode::Wal {
            self.wal.validate()?;
        }
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
            durability: DurabilityMode::default(),
            wal: WalConfig::default(),
            segment: SegmentConfig::default(),
            retention: RetentionConfig::default(),
            read_mode: SegmentReadMode::default(),
            data_dir: PathBuf::from("./quiver_data"),
        }
    }
}

/// Builder wrapper to allow incremental overrides while keeping future fields
/// backwards compatible.
#[derive(Debug, Default)]
pub struct QuiverConfigBuilder {
    durability: DurabilityMode,
    wal: WalConfig,
    segment: SegmentConfig,
    retention: RetentionConfig,
    read_mode: SegmentReadMode,
    data_dir: PathBuf,
}

impl QuiverConfigBuilder {
    /// Sets the durability mode.
    #[must_use]
    pub const fn durability(mut self, durability: DurabilityMode) -> Self {
        self.durability = durability;
        self
    }

    /// Applies a custom WAL configuration.
    #[must_use]
    pub const fn wal(mut self, wal: WalConfig) -> Self {
        self.wal = wal;
        self
    }

    /// Applies a custom segment configuration.
    #[must_use]
    pub const fn segment(mut self, segment: SegmentConfig) -> Self {
        self.segment = segment;
        self
    }

    /// Applies a custom retention configuration.
    #[must_use]
    pub const fn retention(mut self, retention: RetentionConfig) -> Self {
        self.retention = retention;
        self
    }

    /// Sets the segment read mode (mmap vs standard I/O).
    #[must_use]
    pub const fn read_mode(mut self, read_mode: SegmentReadMode) -> Self {
        self.read_mode = read_mode;
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
            durability: self.durability,
            wal: self.wal,
            segment: self.segment,
            retention: self.retention,
            read_mode: self.read_mode,
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
///
/// Note: WAL disk usage is tracked in the shared [`DiskBudget`](crate::DiskBudget)
/// alongside segments, so the configured disk budget cap applies to the
/// combined total of WAL files and segment files.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct WalConfig {
    /// Maximum on-disk footprint (across active + rotated files).
    ///
    /// This is an internal WAL-specific cap that triggers backpressure when
    /// exceeded. The shared [`DiskBudget`](crate::DiskBudget) provides the
    /// overall disk limit that includes both WAL and segment storage.
    pub max_size_bytes: NonZeroU64,
    /// Maximum number of rotated WAL files retained during rotation.
    pub max_rotated_files: u16,
    /// Target data size to keep in the active WAL file before rotating.
    pub rotation_target_bytes: NonZeroU64,
    /// Preferred fsync cadence for durability vs. latency.
    pub flush_interval: Duration,
}

impl WalConfig {
    fn validate(&self) -> Result<()> {
        if self.max_rotated_files == 0 {
            return Err(QuiverError::invalid_config(
                "max_rotated_files must be at least 1",
            ));
        }
        if self.rotation_target_bytes > self.max_size_bytes {
            return Err(QuiverError::invalid_config(
                "rotation_target_bytes must not exceed max_size_bytes",
            ));
        }
        Ok(())
    }
}

impl Default for WalConfig {
    fn default() -> Self {
        Self {
            // WAL max should accommodate (max_rotated_files + 1) * rotation_target_bytes
            // with some headroom. At 10 rotated files × 64 MB = 640 MB theoretical max,
            // 128 MB is conservative (allows ~2 rotations worth). The WAL naturally
            // stays smaller because rotation purges old files.
            max_size_bytes: NonZeroU64::new(128 * 1024 * 1024).expect("non-zero"),
            max_rotated_files: 10,
            rotation_target_bytes: NonZeroU64::new(64 * 1024 * 1024).expect("non-zero"),
            flush_interval: Duration::from_millis(25),
        }
    }
}

/// Segment tuning parameters.
///
/// Segments are finalized (sealed and written to disk) when *either* trigger fires:
///
/// 1. **Size trigger**: The open segment's estimated payload size reaches
///    [`target_size_bytes`](Self::target_size_bytes).
/// 2. **Time trigger**: The segment has been open longer than
///    [`max_open_duration`](Self::max_open_duration).
///
/// This dual-trigger approach balances efficiency and latency:
///
/// - **High-throughput workloads** hit the size trigger frequently, producing
///   consistently-sized segments with good compression and low metadata overhead.
/// - **Low-throughput or bursty workloads** hit the time trigger, ensuring data
///   becomes visible to downstream subscribers within a bounded latency even when
///   volume is insufficient to fill a segment.
///
/// # Tuning Guidance
///
/// | Scenario | Recommendation |
/// |----------|----------------|
/// | High throughput (>10 MB/s) | Increase `target_size_bytes` to 64-128 MB for better compression |
/// | Latency-sensitive | Decrease `max_open_duration` to 1-2 seconds |
/// | Memory-constrained | Decrease `target_size_bytes` (each core buffers up to this amount) |
/// | Low volume (<1 MB/s) | Default is fine; `max_open_duration` dominates |
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SegmentConfig {
    /// Target payload size (in bytes) for a finalized segment.
    ///
    /// When the open segment's estimated size reaches this threshold, finalization
    /// is triggered. Larger values improve compression and reduce per-segment
    /// overhead, but increase memory usage and latency to visibility.
    ///
    /// Default: 32 MB. Recommended range: 8-128 MB.
    pub target_size_bytes: NonZeroU64,

    /// Maximum time a segment may remain open before forced finalization.
    ///
    /// This ensures bounded latency for downstream visibility even when ingest
    /// volume is low. At high throughput, [`target_size_bytes`](Self::target_size_bytes)
    /// typically triggers first.
    ///
    /// Default: 5 seconds.
    pub max_open_duration: Duration,

    /// Soft cap on the number of streams active within a segment.
    ///
    /// Each unique `(slot, schema_fingerprint)` pair creates a new stream.
    /// Exceeding this limit may trigger early finalization to bound memory.
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
#[derive(Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RetentionConfig {
    /// Optional maximum wall-clock retention irrespective of size.
    ///
    /// When set, segments older than this duration are automatically deleted
    /// during maintenance, regardless of whether subscribers have consumed them.
    /// When `None` (the default), segments are retained indefinitely until
    /// consumed by all subscribers or evicted by size-based retention.
    pub max_age: Option<Duration>,
}

impl RetentionConfig {
    fn validate(&self) -> Result<()> {
        if self.max_age.is_some_and(|d| d.is_zero()) {
            return Err(QuiverError::invalid_config(
                "max_age must be greater than zero when set",
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::num::NonZeroU64;

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
                max_age: Some(Duration::ZERO),
            },
            ..QuiverConfig::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn empty_data_dir_is_rejected() {
        let cfg = QuiverConfig {
            data_dir: PathBuf::new(),
            ..QuiverConfig::default()
        };
        assert!(cfg.validate().is_err());
    }

    #[test]
    fn with_data_dir_overrides_path() {
        let cfg = QuiverConfig::default();
        let new_dir = cfg.with_data_dir("/tmp/with-data-dir");

        assert_eq!(new_dir.data_dir, PathBuf::from("/tmp/with-data-dir"));
        assert_eq!(cfg.wal, new_dir.wal);
        assert_eq!(cfg.segment, new_dir.segment);
        assert_eq!(cfg.retention, new_dir.retention);
    }

    #[test]
    fn builder_overrides_sub_configs() {
        let wal = WalConfig {
            max_size_bytes: NonZeroU64::new(1).unwrap(),
            max_rotated_files: 1,
            rotation_target_bytes: NonZeroU64::new(1).unwrap(),
            flush_interval: Duration::from_millis(1),
        };
        let segment = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1024).unwrap(),
            max_open_duration: Duration::from_secs(1),
            max_stream_count: 1,
        };
        let retention = RetentionConfig {
            max_age: Some(Duration::from_secs(1)),
        };

        let cfg = QuiverConfig::builder()
            .wal(wal.clone())
            .segment(segment.clone())
            .retention(retention.clone())
            .build()
            .expect("builder should validate overrides");

        assert_eq!(cfg.wal, wal);
        assert_eq!(cfg.segment, segment);
        assert_eq!(cfg.retention, retention);
    }

    #[test]
    fn builder_falls_back_to_default_data_dir() {
        let cfg = QuiverConfig::builder()
            .data_dir("")
            .build()
            .expect("builder should fill default data dir");

        assert_eq!(cfg.data_dir, PathBuf::from("./quiver_data"));
    }

    #[test]
    fn wal_validate_rejects_zero_rotated_files() {
        let wal = WalConfig {
            max_size_bytes: NonZeroU64::new(1).unwrap(),
            max_rotated_files: 0,
            rotation_target_bytes: NonZeroU64::new(1).unwrap(),
            flush_interval: Duration::from_millis(1),
        };

        assert!(wal.validate().is_err());
    }

    #[test]
    fn wal_validate_rejects_rotation_target_exceeding_cap() {
        let wal = WalConfig {
            max_size_bytes: NonZeroU64::new(1024).unwrap(),
            max_rotated_files: 1,
            rotation_target_bytes: NonZeroU64::new(2048).unwrap(),
            flush_interval: Duration::from_millis(1),
        };

        assert!(wal.validate().is_err());
    }

    #[test]
    fn segment_validate_rejects_zero_streams() {
        let segment = SegmentConfig {
            target_size_bytes: NonZeroU64::new(1).unwrap(),
            max_open_duration: Duration::from_secs(1),
            max_stream_count: 0,
        };

        assert!(segment.validate().is_err());
    }
}
