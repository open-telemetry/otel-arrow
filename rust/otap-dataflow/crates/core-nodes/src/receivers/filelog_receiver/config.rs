// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parsed Filelog settings and compilation into immutable inputs.
//!
//! Filesystem verification and integrated runtime admission are separate startup gates.

use super::decoder::{Encoding, OnDecodeError};
use super::framer::OversizeBehavior;
use serde::{Deserialize, Deserializer};
use std::time::Duration;

mod compile;
mod paths;
pub use compile::{CompiledConfig, ConfigError, ResourceEstimates};
pub use paths::PathPolicy;
#[cfg(test)]
mod tests;

fn bytes<'de, D: Deserializer<'de>>(d: D) -> Result<u64, D::Error> {
    otel_arrow_dfe_config::byte_units::deserialize_u64(d)?
        .ok_or_else(|| serde::de::Error::custom("byte size must not be null"))
}

#[derive(Deserialize)]
#[serde(remote = "Encoding")]
enum EncodingDef {
    #[serde(rename = "utf-8")]
    Utf8,
    #[serde(rename = "ascii")]
    Ascii,
    #[serde(rename = "utf-16le")]
    Utf16Le,
    #[serde(rename = "utf-16be")]
    Utf16Be,
    #[serde(rename = "raw")]
    Raw,
}
#[derive(Deserialize)]
#[serde(remote = "OnDecodeError", rename_all = "snake_case")]
enum DecodeDef {
    PreserveRaw,
    Replace,
    Fail,
}
#[derive(Deserialize)]
#[serde(remote = "OversizeBehavior", rename_all = "snake_case")]
enum OversizeDef {
    Split,
    Truncate,
}

/// Configured FileMetadata policy.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum FileMetadata {
    /// No exported file metadata.
    #[default]
    None,
    /// Lossless basename.
    Name,
    /// Lossless path and basename.
    Path,
    /// Bounded native evidence and lossless text when available.
    Native,
}

/// Configured StartAt policy.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum StartAt {
    /// Skip existing bytes of an unrelated new file.
    #[default]
    End,
    /// Read an unrelated new file from zero.
    Beginning,
}

/// Configured RecoveryMismatch policy.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryMismatch {
    /// Start a new identity from zero.
    #[default]
    Beginning,
    /// Explicitly skip existing bytes.
    SkipToEnd,
    /// Quarantine the mismatch.
    Fail,
}

/// Configured TruncatePolicy policy.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TruncatePolicy {
    /// Quarantine detectable truncation.
    #[default]
    Fail,
    /// Accept the gap and read the new epoch.
    ReadNew,
}

/// Configured RetryExhaustion policy.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RetryExhaustion {
    /// Continue periodic attempts while paused.
    #[default]
    Pause,
    /// Stop the receiver.
    Fail,
    /// Authorize explicit loss after retry exhaustion.
    DropAndContinue,
}

/// Configured RegexProfile policy.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RegexProfile {
    /// The bounded version-one executable profile.
    #[default]
    #[serde(rename = "re2-v1")]
    Re2V1,
}

/// Parsed config settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Config {
    /// `include` in the Phase 1 configuration contract.
    pub include: Vec<String>,
    /// `exclude` in the Phase 1 configuration contract.
    pub exclude: Vec<String>,
    /// `include_file_metadata` in the Phase 1 configuration contract.
    pub include_file_metadata: FileMetadata,
    /// `recursive` in the Phase 1 configuration contract.
    pub recursive: bool,
    /// `follow_symlinks` in the Phase 1 configuration contract.
    pub follow_symlinks: bool,
    /// `max_recursion_depth` in the Phase 1 configuration contract.
    pub max_recursion_depth: u64,
    /// `start_at` in the Phase 1 configuration contract.
    pub start_at: StartAt,
    /// `discovery` in the Phase 1 configuration contract.
    pub discovery: Discovery,
    /// `reader` in the Phase 1 configuration contract.
    pub reader: Reader,
    /// `ignore_older_than` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub ignore_older_than: Duration,
    /// `identity` in the Phase 1 configuration contract.
    pub identity: Identity,
    /// `encoding` in the Phase 1 configuration contract.
    #[serde(with = "EncodingDef")]
    pub encoding: Encoding,
    /// `on_decode_error` in the Phase 1 configuration contract.
    #[serde(with = "DecodeDef")]
    pub on_decode_error: OnDecodeError,
    /// `framing` in the Phase 1 configuration contract.
    pub framing: Framing,
    /// `limits` in the Phase 1 configuration contract.
    pub limits: Limits,
    /// `batch` in the Phase 1 configuration contract.
    pub batch: Batch,
    /// `rotation` in the Phase 1 configuration contract.
    pub rotation: Rotation,
    /// `checkpoint` in the Phase 1 configuration contract.
    pub checkpoint: Checkpoint,
    /// `retry` in the Phase 1 configuration contract.
    pub retry: Retry,
    /// `on_retry_exhaustion` in the Phase 1 configuration contract.
    pub on_retry_exhaustion: RetryExhaustion,
    /// `drain_timeout` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub drain_timeout: Duration,
    /// `worker_shutdown_timeout` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub worker_shutdown_timeout: Duration,
}
impl Default for Config {
    fn default() -> Self {
        Self {
            include: Vec::new(),
            exclude: Vec::new(),
            include_file_metadata: FileMetadata::None,
            recursive: true,
            follow_symlinks: false,
            max_recursion_depth: 64,
            start_at: StartAt::End,
            discovery: Discovery::default(),
            reader: Reader::default(),
            ignore_older_than: Duration::ZERO,
            identity: Identity::default(),
            encoding: Encoding::Utf8,
            on_decode_error: OnDecodeError::PreserveRaw,
            framing: Framing::default(),
            limits: Limits::default(),
            batch: Batch::default(),
            rotation: Rotation::default(),
            checkpoint: Checkpoint::default(),
            retry: Retry::default(),
            on_retry_exhaustion: RetryExhaustion::Pause,
            drain_timeout: Duration::from_secs(10),
            worker_shutdown_timeout: Duration::from_secs(5),
        }
    }
}

/// Parsed discovery settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Discovery {
    /// `reconcile_interval` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub reconcile_interval: Duration,
    /// `reconcile_jitter_percent` in the Phase 1 configuration contract.
    pub reconcile_jitter_percent: u64,
}
impl Default for Discovery {
    fn default() -> Self {
        Self {
            reconcile_interval: Duration::from_secs(5),
            reconcile_jitter_percent: 10,
        }
    }
}

/// Parsed reader settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Reader {
    /// `eof_reprobe_interval` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub eof_reprobe_interval: Duration,
}
impl Default for Reader {
    fn default() -> Self {
        Self {
            eof_reprobe_interval: Duration::from_millis(250),
        }
    }
}

/// Parsed identity settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Identity {
    /// `fingerprint_bytes` in the Phase 1 configuration contract.
    pub fingerprint_bytes: u64,
    /// `ignored_header_bytes` in the Phase 1 configuration contract.
    pub ignored_header_bytes: u64,
    /// `on_recovery_mismatch` in the Phase 1 configuration contract.
    pub on_recovery_mismatch: RecoveryMismatch,
}
impl Default for Identity {
    fn default() -> Self {
        Self {
            fingerprint_bytes: 1000,
            ignored_header_bytes: 0,
            on_recovery_mismatch: RecoveryMismatch::Beginning,
        }
    }
}

/// Parsed framing settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Framing {
    /// `max_line_bytes` in the Phase 1 configuration contract.
    #[serde(deserialize_with = "bytes")]
    pub max_line_bytes: u64,
    /// `max_record_bytes` in the Phase 1 configuration contract.
    #[serde(deserialize_with = "bytes")]
    pub max_record_bytes: u64,
    /// `max_log_size_behavior` in the Phase 1 configuration contract.
    #[serde(with = "OversizeDef")]
    pub max_log_size_behavior: OversizeBehavior,
    /// `force_flush_period` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub force_flush_period: Duration,
    /// `multiline` in the Phase 1 configuration contract.
    pub multiline: Multiline,
    /// `max_multiline_lines` in the Phase 1 configuration contract.
    pub max_multiline_lines: u32,
}
impl Default for Framing {
    fn default() -> Self {
        Self {
            max_line_bytes: 1024 * 1024,
            max_record_bytes: 1024 * 1024,
            max_log_size_behavior: OversizeBehavior::Split,
            force_flush_period: Duration::ZERO,
            multiline: Multiline::default(),
            max_multiline_lines: 500,
        }
    }
}

/// Parsed multiline settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Multiline {
    /// `regex_profile` in the Phase 1 configuration contract.
    pub regex_profile: RegexProfile,
    /// `line_start_pattern` in the Phase 1 configuration contract.
    pub line_start_pattern: Option<String>,
    /// `line_end_pattern` in the Phase 1 configuration contract.
    pub line_end_pattern: Option<String>,
}
impl Default for Multiline {
    fn default() -> Self {
        Self {
            regex_profile: RegexProfile::Re2V1,
            line_start_pattern: None,
            line_end_pattern: None,
        }
    }
}

/// Parsed limits settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Limits {
    /// `max_tracked_files` in the Phase 1 configuration contract.
    pub max_tracked_files: u64,
    /// `max_pending_candidates` in the Phase 1 configuration contract.
    pub max_pending_candidates: u64,
    /// `max_open_files` in the Phase 1 configuration contract.
    pub max_open_files: u64,
    /// `max_read_bytes_per_turn` in the Phase 1 configuration contract.
    #[serde(deserialize_with = "bytes")]
    pub max_read_bytes_per_turn: u64,
    /// `max_partial_state_bytes` in the Phase 1 configuration contract.
    #[serde(deserialize_with = "bytes")]
    pub max_partial_state_bytes: u64,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            max_tracked_files: 10000,
            max_pending_candidates: 10000,
            max_open_files: 512,
            max_read_bytes_per_turn: 128 * 1024,
            max_partial_state_bytes: 256 * 1024 * 1024,
        }
    }
}

/// Parsed batch settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Batch {
    /// `max_records` in the Phase 1 configuration contract.
    pub max_records: u64,
    /// `max_bytes` in the Phase 1 configuration contract.
    #[serde(deserialize_with = "bytes")]
    pub max_bytes: u64,
    /// `max_flush_period` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub max_flush_period: Duration,
}
impl Default for Batch {
    fn default() -> Self {
        Self {
            max_records: 1024,
            max_bytes: 8 * 1024 * 1024,
            max_flush_period: Duration::from_secs(1),
        }
    }
}

/// Parsed rotation settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Rotation {
    /// `rotate_wait` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub rotate_wait: Duration,
    /// `on_truncate` in the Phase 1 configuration contract.
    pub on_truncate: TruncatePolicy,
}
impl Default for Rotation {
    fn default() -> Self {
        Self {
            rotate_wait: Duration::from_secs(5),
            on_truncate: TruncatePolicy::Fail,
        }
    }
}

/// Parsed checkpoint settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Checkpoint {
    /// `id` in the Phase 1 configuration contract.
    pub id: Option<String>,
    /// `sync_interval` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub sync_interval: Duration,
    /// `compact_after_bytes` in the Phase 1 configuration contract.
    #[serde(deserialize_with = "bytes")]
    pub compact_after_bytes: u64,
    /// `compact_after_transactions` in the Phase 1 configuration contract.
    pub compact_after_transactions: u64,
    /// `retention` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub retention: Duration,
    /// `ownership_timeout` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub ownership_timeout: Duration,
    /// `max_consecutive_failures` in the Phase 1 configuration contract.
    pub max_consecutive_failures: u64,
}
impl Default for Checkpoint {
    fn default() -> Self {
        Self {
            id: None,
            sync_interval: Duration::ZERO,
            compact_after_bytes: 64 * 1024 * 1024,
            compact_after_transactions: 10000,
            retention: Duration::from_secs(7 * 86400),
            ownership_timeout: Duration::from_secs(30),
            max_consecutive_failures: 5,
        }
    }
}

/// Parsed retry settings; defaults do not imply validation.
#[derive(Debug, Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Retry {
    /// `max_attempts` in the Phase 1 configuration contract.
    pub max_attempts: u64,
    /// `initial_backoff` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub initial_backoff: Duration,
    /// `max_backoff` in the Phase 1 configuration contract.
    #[serde(with = "humantime_serde")]
    pub max_backoff: Duration,
}
impl Default for Retry {
    fn default() -> Self {
        Self {
            max_attempts: 8,
            initial_backoff: Duration::from_millis(100),
            max_backoff: Duration::from_secs(5),
        }
    }
}
