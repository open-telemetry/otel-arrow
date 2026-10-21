// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::{
    framer::LineConfig,
    multiline::{GroupingMode, MultilineConfig},
    multiline_pattern::{BoundaryPattern, PatternMode},
};
use super::*;
use otel_arrow_dfe_filelog_checkpoint as checkpoint;
use sha2::{Digest, Sha256};
use std::{
    path::Path,
    time::{Duration, Instant},
};

/// Actionable configuration rejection; no filesystem side effects have occurred.
#[derive(Debug, thiserror::Error)]
#[error("filelog {setting}: {reason}")]
pub struct ConfigError {
    /// Setting or interacting resource formula that failed.
    pub setting: String,
    /// Constraint and contributing values where applicable.
    pub reason: String,
}
impl ConfigError {
    pub(super) fn new(setting: impl Into<String>, reason: impl Into<String>) -> Self {
        Self {
            setting: setting.into(),
            reason: reason.into(),
        }
    }
}

impl From<ConfigError> for otel_arrow_dfe_config::error::Error {
    fn from(error: ConfigError) -> Self {
        Self::InvalidUserConfig {
            error: error.to_string(),
        }
    }
}

/// Configuration-only resource estimates; runtime ledgers must add their own state.
#[derive(Debug)]
pub struct ResourceEstimates {
    /// Normative provisional candidate/identity model, capped at 1 GiB.
    pub identity_reconciliation_bytes: u64,
    /// Provisional reader table and one shared turn buffer.
    pub reader_table_bytes: u64,
    /// Primitive payload growth allowance, excluding runtime reader metadata.
    pub framer_payload_bytes: u64,
    /// Framer payload plus one owned source-turn remainder; runtime metadata is additional.
    pub partial_state_floor_bytes: u64,
    /// Simultaneous retained batch and one maximum logical carry-over record.
    pub batch_and_carry_over_bytes: u64,
    /// Compiled path programs, owned roots, and bounded matching scratch.
    pub path_matcher_bytes: u64,
    /// Retained multiline program allowance, including metadata.
    pub multiline_program_bytes: u64,
    /// One worker's multiline scratch bound, including deferred fallback.
    pub multiline_scratch_bytes: u64,
    /// Checked sum of path policy and multiline program/scratch charges.
    pub matching_bytes: u64,
    /// Portable file and recovery phase bounds.
    pub checkpoint: checkpoint::CheckpointBounds,
    /// Maximum encoded snapshot section under the configured fingerprint window.
    pub snapshot_bytes: u64,
    /// Conservative logical record size, including enabled metadata.
    pub record_bytes: u64,
}

/// Validated settings and immutable compiled programs. This is not startup readiness.
#[derive(Debug)]
pub struct CompiledConfig {
    inputs: ValidatedInputs,
    paths: PathPolicy,
}

/// Validated settings and primitive inputs retained after transferring the path policy.
#[derive(Debug)]
pub struct ValidatedInputs {
    config: Config,
    checkpoint_id: String,
    namespace_component: String,
    line: LineConfig,
    grouping: Option<GroupingMode>,
    boundary: Option<BoundaryPattern>,
    profile: checkpoint::FramingProfileParams,
    resources: ResourceEstimates,
}
impl CompiledConfig {
    /// Borrow the validated inputs without transferring either owned part.
    #[must_use]
    pub fn inputs(&self) -> &ValidatedInputs {
        &self.inputs
    }

    /// Lexical policy; filesystem exclusion proofs remain discovery-owned.
    #[must_use]
    pub fn paths(&self) -> &PathPolicy {
        &self.paths
    }

    /// Move validated inputs and worker-owned matching scratch to their respective owners.
    #[must_use]
    pub fn into_parts(self) -> (ValidatedInputs, PathPolicy) {
        (self.inputs, self.paths)
    }
}

impl ValidatedInputs {
    /// Validated parsed values, borrowed to prevent mutation after compilation.
    #[must_use]
    pub fn settings(&self) -> &Config {
        &self.config
    }
    /// Effective logical namespace, stable across runtime placement changes.
    #[must_use]
    pub fn checkpoint_id(&self) -> &str {
        &self.checkpoint_id
    }
    /// Encoded component beneath the engine's `filelog/@v1` directory.
    #[must_use]
    pub fn namespace_component(&self) -> &str {
        &self.namespace_component
    }
    /// Validated line-framer inputs.
    #[must_use]
    pub fn line(&self) -> LineConfig {
        self.line
    }
    /// Optional grouping semantics.
    #[must_use]
    pub fn grouping(&self) -> Option<GroupingMode> {
        self.grouping
    }
    /// Immutable program; worker scratch must be admitted before creating a matcher.
    #[must_use]
    pub fn boundary(&self) -> Option<&BoundaryPattern> {
        self.boundary.as_ref()
    }
    /// Canonical checkpoint compatibility inputs.
    #[must_use]
    pub fn profile(&self) -> &checkpoint::FramingProfileParams {
        &self.profile
    }
    /// Completed configuration models, without a claim of aggregate runtime admission.
    #[must_use]
    pub fn resources(&self) -> &ResourceEstimates {
        &self.resources
    }
    /// Store inputs use the same validator that opening the store uses.
    #[cfg(target_os = "linux")]
    pub fn store_config(&self) -> Result<checkpoint::store::StoreConfig, ConfigError> {
        store_config(&self.config, &self.profile, self.resources.snapshot_bytes)
    }
}

impl Config {
    /// Parse the engine's opaque node configuration, retaining the field path in errors.
    pub fn from_json(value: serde_json::Value) -> Result<Self, ConfigError> {
        serde_path_to_error::deserialize(value)
            .map_err(|error| ConfigError::new(error.path().to_string(), error.inner().to_string()))
    }

    /// Compile before workers or namespace access. `base` freezes relative glob
    /// interpretation; it is unrelated to the engine-owned checkpoint state root.
    /// Only the three configured logical IDs participate in automatic ID selection.
    pub fn compile(
        self,
        group: &str,
        pipeline: &str,
        node: &str,
        base: &Path,
    ) -> Result<CompiledConfig, ConfigError> {
        self.validate_scalars()?;
        let checkpoint_id = match &self.checkpoint.id {
            Some(id) => id.clone(),
            None => derive_id([group, pipeline, node])?,
        };
        let namespace_component = checkpoint::namespace_component(&checkpoint_id)
            .map_err(|e| ConfigError::new("checkpoint.id", e.to_string()))?;
        let line = LineConfig {
            encoding: self.encoding,
            on_decode_error: self.on_decode_error,
            max_line_bytes: size("framing.max_line_bytes", self.framing.max_line_bytes)?,
            max_record_bytes: size("framing.max_record_bytes", self.framing.max_record_bytes)?,
            oversize: self.framing.max_log_size_behavior,
        };
        line.validate().map_err(|e| {
            let reason = format!(
                "{e}; bounds must fit allocation arithmetic and a complete encoded unit \
                (raw or ASCII/fail: 1, other ASCII: 3, UTF: 4 bytes)"
            );
            ConfigError::new("framing.max_line_bytes/max_record_bytes", reason)
        })?;
        let ml = &self.framing.multiline;
        let (grouping, pattern, field) = match (&ml.line_start_pattern, &ml.line_end_pattern) {
            (Some(_), Some(_)) => {
                return Err(ConfigError::new(
                    "framing.multiline",
                    "configure at most one boundary pattern",
                ));
            }
            (Some(p), None) => (
                Some(GroupingMode::Start),
                Some(p),
                "framing.multiline.line_start_pattern",
            ),
            (None, Some(p)) => (
                Some(GroupingMode::End),
                Some(p),
                "framing.multiline.line_end_pattern",
            ),
            (None, None) => (None, None, "framing.multiline"),
        };
        let boundary = pattern
            .map(|p| {
                if p.is_empty() {
                    return Err(ConfigError::new(field, "pattern must not be empty"));
                }
                BoundaryPattern::compile(
                    p,
                    if self.encoding == Encoding::Raw {
                        PatternMode::Raw
                    } else {
                        PatternMode::Text
                    },
                )
                .map_err(|e| ConfigError::new(field, e.to_string()))
            })
            .transpose()?;
        let multiline_mode = match (grouping, pattern) {
            (Some(GroupingMode::Start), Some(p)) => checkpoint::MultilineMode::StartPattern {
                regex_profile_version: 1,
                pattern: p.clone(),
            },
            (Some(GroupingMode::End), Some(p)) => checkpoint::MultilineMode::EndPattern {
                regex_profile_version: 1,
                pattern: p.clone(),
            },
            _ => checkpoint::MultilineMode::Newline,
        };
        let profile = checkpoint::FramingProfileParams {
            fingerprint_profile_version: 1,
            fingerprint_bytes: u16::try_from(self.identity.fingerprint_bytes)
                .map_err(|e| ConfigError::new("identity.fingerprint_bytes", e.to_string()))?,
            ignored_header_bytes: u32::try_from(self.identity.ignored_header_bytes)
                .map_err(|e| ConfigError::new("identity.ignored_header_bytes", e.to_string()))?,
            encoding: match self.encoding {
                Encoding::Utf8 => checkpoint::FramingEncoding::Utf8,
                Encoding::Ascii => checkpoint::FramingEncoding::Ascii,
                Encoding::Utf16Le => checkpoint::FramingEncoding::Utf16Le,
                Encoding::Utf16Be => checkpoint::FramingEncoding::Utf16Be,
                Encoding::Raw => checkpoint::FramingEncoding::Raw,
            },
            on_decode_error: match self.on_decode_error {
                OnDecodeError::PreserveRaw => checkpoint::FramingOnDecodeError::PreserveRaw,
                OnDecodeError::Replace => checkpoint::FramingOnDecodeError::Replace,
                OnDecodeError::Fail => checkpoint::FramingOnDecodeError::Fail,
            },
            multiline_mode,
            max_line_bytes: self.framing.max_line_bytes,
            max_record_bytes: self.framing.max_record_bytes,
            max_log_size_behavior: match self.framing.max_log_size_behavior {
                OversizeBehavior::Split => checkpoint::MaxLogSizeBehavior::Split,
                OversizeBehavior::Truncate => checkpoint::MaxLogSizeBehavior::Truncate,
            },
            max_multiline_lines: self.framing.max_multiline_lines,
            force_flush_period_millis: u64::try_from(self.framing.force_flush_period.as_millis())
                .map_err(|e| {
                ConfigError::new("framing.force_flush_period", e.to_string())
            })?,
        };
        let _ = profile
            .digest()
            .map_err(|e| ConfigError::new("framing", e.to_string()))?;
        let paths = PathPolicy::compile(&self.include, &self.exclude, base)?;
        let resources = self.resources(line, grouping, &paths, boundary.as_ref())?;
        #[cfg(target_os = "linux")]
        let _ = store_config(&self, &profile, resources.snapshot_bytes)?;
        Ok(CompiledConfig {
            paths,
            inputs: ValidatedInputs {
                config: self,
                checkpoint_id,
                namespace_component,
                line,
                grouping,
                boundary,
                profile,
                resources,
            },
        })
    }

    fn validate_scalars(&self) -> Result<(), ConfigError> {
        for (name, value, min, max) in [
            ("max_recursion_depth", self.max_recursion_depth, 1, 1024),
            (
                "discovery.reconcile_jitter_percent",
                self.discovery.reconcile_jitter_percent,
                0,
                25,
            ),
            (
                "identity.fingerprint_bytes",
                self.identity.fingerprint_bytes,
                16,
                65535,
            ),
            (
                "identity.ignored_header_bytes",
                self.identity.ignored_header_bytes,
                0,
                u64::from(u32::MAX),
            ),
            (
                "framing.max_multiline_lines",
                u64::from(self.framing.max_multiline_lines),
                1,
                u64::from(u32::MAX),
            ),
            (
                "limits.max_tracked_files",
                self.limits.max_tracked_files,
                1,
                u64::from(u32::MAX),
            ),
            ("batch.max_records", self.batch.max_records, 1, 65535),
            (
                "checkpoint.compact_after_bytes",
                self.checkpoint.compact_after_bytes,
                checkpoint::WAL_MAX_TX_FRAME_BYTES,
                u64::MAX,
            ),
            (
                "checkpoint.compact_after_transactions",
                self.checkpoint.compact_after_transactions,
                1,
                u64::MAX,
            ),
            (
                "checkpoint.max_consecutive_failures",
                self.checkpoint.max_consecutive_failures,
                1,
                u64::MAX,
            ),
            ("retry.max_attempts", self.retry.max_attempts, 1, u64::MAX),
        ] {
            range(name, value, min, max)?;
        }
        for (name, value) in [
            (
                "limits.max_pending_candidates",
                self.limits.max_pending_candidates,
            ),
            ("limits.max_open_files", self.limits.max_open_files),
            (
                "limits.max_read_bytes_per_turn",
                self.limits.max_read_bytes_per_turn,
            ),
            (
                "limits.max_partial_state_bytes",
                self.limits.max_partial_state_bytes,
            ),
            ("batch.max_bytes", self.batch.max_bytes),
        ] {
            range(name, value, 1, u64::MAX)?;
            let _ = size(name, value)?;
        }
        if self.limits.max_read_bytes_per_turn > isize::MAX as u64 {
            return Err(ConfigError::new(
                "limits.max_read_bytes_per_turn",
                "source-turn allocation exceeds target isize::MAX",
            ));
        }
        if self.limits.max_open_files > self.limits.max_tracked_files {
            return Err(ConfigError::new(
                "limits.max_open_files",
                "must not exceed limits.max_tracked_files",
            ));
        }
        let populations = self
            .limits
            .max_pending_candidates
            .checked_add(self.limits.max_open_files)
            .and_then(|n| n.checked_add(self.limits.max_tracked_files))
            .ok_or_else(|| {
                ConfigError::new("limits", "candidate/reader population sum overflows")
            })?;
        let _ = size("limits candidate/reader population", populations)?;
        let now = Instant::now();
        for (name, duration, zero) in [
            (
                "discovery.reconcile_interval",
                self.discovery.reconcile_interval,
                false,
            ),
            (
                "reader.eof_reprobe_interval",
                self.reader.eof_reprobe_interval,
                false,
            ),
            (
                "framing.force_flush_period",
                self.framing.force_flush_period,
                true,
            ),
            ("batch.max_flush_period", self.batch.max_flush_period, false),
            ("rotation.rotate_wait", self.rotation.rotate_wait, false),
            (
                "checkpoint.sync_interval",
                self.checkpoint.sync_interval,
                true,
            ),
            (
                "checkpoint.ownership_timeout",
                self.checkpoint.ownership_timeout,
                false,
            ),
            ("retry.initial_backoff", self.retry.initial_backoff, false),
            ("retry.max_backoff", self.retry.max_backoff, false),
            ("drain_timeout", self.drain_timeout, false),
            (
                "worker_shutdown_timeout",
                self.worker_shutdown_timeout,
                false,
            ),
        ] {
            if (!zero && duration.is_zero()) || now.checked_add(duration).is_none() {
                return Err(ConfigError::new(
                    name,
                    "must satisfy its nonzero policy and fit a runtime deadline",
                ));
            }
        }
        duration_range(
            "discovery.reconcile_interval",
            self.discovery.reconcile_interval,
            Duration::from_millis(100),
            Duration::from_secs(86400),
        )?;
        duration_range(
            "reader.eof_reprobe_interval",
            self.reader.eof_reprobe_interval,
            Duration::from_millis(10),
            Duration::from_secs(3600),
        )?;
        let jitter = self
            .discovery
            .reconcile_interval
            .checked_mul(
                u32::try_from(self.discovery.reconcile_jitter_percent).map_err(|e| {
                    ConfigError::new("discovery.reconcile_jitter_percent", e.to_string())
                })?,
            )
            .and_then(|d| d.checked_div(100))
            .and_then(|d| d.checked_add(self.discovery.reconcile_interval))
            .ok_or_else(|| ConfigError::new("discovery", "jitter duration overflows"))?;
        if now.checked_add(jitter).is_none() {
            return Err(ConfigError::new("discovery", "jittered deadline overflows"));
        }
        if !self.framing.force_flush_period.is_zero()
            && (!self
                .framing
                .force_flush_period
                .subsec_nanos()
                .is_multiple_of(1_000_000)
                || self.framing.force_flush_period >= self.rotation.rotate_wait)
        {
            return Err(ConfigError::new(
                "framing.force_flush_period",
                "must be exact whole milliseconds and strictly less than rotation.rotate_wait",
            ));
        }
        let _ = u64::try_from(self.checkpoint.retention.as_nanos()).map_err(|e| {
            ConfigError::new(
                "checkpoint.retention",
                format!("must fit u64 nanoseconds: {e}"),
            )
        })?;
        // Duration's u64 seconds plus nanoseconds always fit positive i128 nanoseconds.
        if self.retry.max_backoff < self.retry.initial_backoff {
            return Err(ConfigError::new(
                "retry.max_backoff",
                "must be at least retry.initial_backoff",
            ));
        }
        if self
            .drain_timeout
            .checked_add(self.worker_shutdown_timeout)
            .and_then(|d| now.checked_add(d))
            .is_none()
        {
            return Err(ConfigError::new(
                "drain_timeout/worker_shutdown_timeout",
                "combined deadline overflows",
            ));
        }
        Ok(())
    }

    fn resources(
        &self,
        line: LineConfig,
        grouping: Option<GroupingMode>,
        paths: &PathPolicy,
        boundary: Option<&BoundaryPattern>,
    ) -> Result<ResourceEstimates, ConfigError> {
        let f = self.identity.fingerprint_bytes;
        let l = &self.limits;
        let p = 44 + checkpoint::ADVISORY_PATH_STORED_MAX_BYTES as u64;
        let identity_inputs = format!(
            "max_tracked_files={}, max_pending_candidates={}, max_open_files={}, fingerprint_bytes={f}",
            l.max_tracked_files, l.max_pending_candidates, l.max_open_files,
        );
        let identity = (|| {
            let candidates = l
                .max_pending_candidates
                .checked_add(l.max_open_files)?
                .checked_mul(5 * f + 4 * p + 2048)?;
            let open = l.max_open_files.checked_mul(10 * f + 10 * p + 4096)?;
            let records = l.max_tracked_files.checked_mul(f + p + 34 + 384)?;
            candidates
                .checked_add(open)?
                .checked_add(records)?
                .checked_add(l.max_tracked_files.checked_mul(1024)?)
        })()
        .ok_or_else(|| {
            ConfigError::new(
                "limits/identity",
                format!("identity_reconciliation_bytes arithmetic overflow; {identity_inputs}"),
            )
        })?;
        if identity > 1 << 30 {
            return Err(ConfigError::new(
                "limits/identity",
                format!(
                    "identity_reconciliation_bytes={identity} exceeds ceiling=1073741824; {identity_inputs}"
                ),
            ));
        }
        let reader_inputs = format!(
            "limits.max_tracked_files={}, identity.fingerprint_bytes={f}, limits.max_read_bytes_per_turn={}",
            l.max_tracked_files, l.max_read_bytes_per_turn,
        );
        let readers = l
            .max_tracked_files
            .checked_mul(f + 2 * p + 1024)
            .and_then(|n| n.checked_add(l.max_read_bytes_per_turn))
            .ok_or_else(|| {
                ConfigError::new(
                    "reader_table_bytes",
                    format!("arithmetic overflow; {reader_inputs}"),
                )
            })?;
        let _ = usize::try_from(readers).map_err(|_| {
            ConfigError::new(
                "reader_table_bytes",
                format!("{readers} exceeds target usize; {reader_inputs}"),
            )
        })?;
        let checkpoint_inputs = format!(
            "checkpoint.compact_after_bytes={}, checkpoint.compact_after_transactions={}, limits.max_tracked_files={}, identity.fingerprint_bytes={f}",
            self.checkpoint.compact_after_bytes,
            self.checkpoint.compact_after_transactions,
            l.max_tracked_files,
        );
        let snapshot = checkpoint::maximum_snapshot_bytes(
            u32::try_from(l.max_tracked_files)
                .map_err(|e| ConfigError::new("limits.max_tracked_files", e.to_string()))?,
            u16::try_from(f)
                .map_err(|e| ConfigError::new("identity.fingerprint_bytes", e.to_string()))?,
        )
        .map_err(|e| ConfigError::new("checkpoint", format!("{e}; {checkpoint_inputs}")))?;
        let checkpoint = checkpoint::CheckpointBounds::new(
            snapshot,
            self.checkpoint.compact_after_bytes,
            self.checkpoint.compact_after_transactions,
        )
        .map_err(|e| ConfigError::new("checkpoint", format!("{e}; {checkpoint_inputs}")))?;
        let payload = if let Some(mode) = grouping {
            MultilineConfig {
                line,
                max_lines: self.framing.max_multiline_lines,
                mode,
            }
            .payload_peak_bytes()
            .map_err(|e| ConfigError::new("framing", e.to_string()))?
        } else {
            line.payload_peak_bytes()
                .map_err(|e| ConfigError::new("framing", e.to_string()))?
        } as u64;
        let partial_floor = payload
            .checked_add(l.max_read_bytes_per_turn)
            .ok_or_else(|| {
                ConfigError::new(
                    "limits.max_partial_state_bytes",
                    "framer payload plus source-turn remainder overflows",
                )
            })?;
        if l.max_partial_state_bytes < partial_floor {
            return Err(ConfigError::new(
                "limits.max_partial_state_bytes",
                format!(
                    "{} is below partial_state_floor_bytes={partial_floor}; framer_payload_bytes={payload}, \
                 limits.max_read_bytes_per_turn={}, framing.max_line_bytes={}, framing.max_record_bytes={}, \
                 encoding={:?}, on_decode_error={:?}, multiline={}; runtime reader metadata is additional",
                    l.max_partial_state_bytes,
                    l.max_read_bytes_per_turn,
                    self.framing.max_line_bytes,
                    self.framing.max_record_bytes,
                    self.encoding,
                    self.on_decode_error,
                    grouping.is_some()
                ),
            ));
        }
        let record_bytes = record_bound(self, grouping.is_some())?;
        if record_bytes > self.batch.max_bytes {
            return Err(ConfigError::new(
                "batch.max_bytes",
                format!(
                    "{} is less than maximum logical record {record_bytes}, including framing and include_file_metadata={:?}",
                    self.batch.max_bytes, self.include_file_metadata
                ),
            ));
        }
        let batch_and_carry_over_bytes = self.batch.max_bytes.checked_add(record_bytes)
            .ok_or_else(|| ConfigError::new("batch.max_bytes", format!(
                "batch_and_carry_over_bytes arithmetic overflow; batch.max_bytes={}, maximum_record_bytes={record_bytes}", self.batch.max_bytes
            )))?;
        let _ = usize::try_from(batch_and_carry_over_bytes).map_err(|_| {
            ConfigError::new(
                "batch.max_bytes",
                format!(
                    "batch_and_carry_over_bytes={batch_and_carry_over_bytes} exceeds target usize"
                ),
            )
        })?;
        let path_matcher_bytes = paths.retained_bytes()?;
        let (multiline_program_bytes, multiline_scratch_bytes) =
            boundary.map_or((0, 0), |pattern| {
                let memory = pattern.memory_estimate();
                (
                    memory.program_heap_bound as u64,
                    memory.worker_heap_bound as u64,
                )
            });
        let matching_bytes = path_matcher_bytes
            .checked_add(multiline_program_bytes)
            .and_then(|n| n.checked_add(multiline_scratch_bytes))
            .ok_or_else(|| {
                ConfigError::new(
                    "include/exclude/framing.multiline",
                    "matching memory sum overflows",
                )
            })?;
        Ok(ResourceEstimates {
            identity_reconciliation_bytes: identity,
            reader_table_bytes: readers,
            framer_payload_bytes: payload,
            partial_state_floor_bytes: partial_floor,
            batch_and_carry_over_bytes,
            path_matcher_bytes,
            multiline_program_bytes,
            multiline_scratch_bytes,
            matching_bytes,
            checkpoint,
            snapshot_bytes: snapshot,
            record_bytes,
        })
    }
}

fn derive_id(ids: [&str; 3]) -> Result<String, ConfigError> {
    let mut hash = Sha256::new();
    hash.update(b"otel-arrow-filelog-checkpoint-id-v1\0");
    for id in ids {
        let len = u32::try_from(id.len())
            .map_err(|e| ConfigError::new("checkpoint.id logical key", e.to_string()))?;
        hash.update(len.to_be_bytes());
        hash.update(id.as_bytes());
    }
    Ok(format!("auto-{}", hex::encode(hash.finalize())))
}
fn size(name: &str, value: u64) -> Result<usize, ConfigError> {
    usize::try_from(value).map_err(|e| ConfigError::new(name, format!("must fit usize: {e}")))
}
fn range(name: &str, value: u64, min: u64, max: u64) -> Result<(), ConfigError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(ConfigError::new(
            name,
            format!("{value} must be in {min}..={max}"),
        ))
    }
}
fn duration_range(
    name: &str,
    value: Duration,
    min: Duration,
    max: Duration,
) -> Result<(), ConfigError> {
    if (min..=max).contains(&value) {
        Ok(())
    } else {
        Err(ConfigError::new(
            name,
            format!("{value:?} must be in {min:?}..={max:?}"),
        ))
    }
}

#[cfg(target_os = "linux")]
fn store_config(
    config: &Config,
    profile: &checkpoint::FramingProfileParams,
    snapshot: u64,
) -> Result<checkpoint::store::StoreConfig, ConfigError> {
    let replay = checkpoint::ReplayConfig::new(
        u32::try_from(config.limits.max_tracked_files)
            .map_err(|e| ConfigError::new("limits.max_tracked_files", e.to_string()))?,
        snapshot,
        profile,
    )
    .map_err(|e| ConfigError::new("checkpoint", e.to_string()))?;
    checkpoint::store::StoreConfig {
        replay,
        max_wal_bytes: config.checkpoint.compact_after_bytes,
        max_wal_transactions: config.checkpoint.compact_after_transactions,
    }
    .validate()
    .map_err(|e| {
        ConfigError::new(
            "checkpoint",
            format!(
                "{e}; identity.fingerprint_bytes={}",
                config.identity.fingerprint_bytes
            ),
        )
    })
}

fn record_bound(c: &Config, multiline: bool) -> Result<u64, ConfigError> {
    // Text conversion of Windows UTF-16 evidence can expand 4096 stored bytes
    // to 6144 UTF-8 bytes. Reserve that portable maximum for path AND basename.
    let text = 6144u64;
    let mut attrs = "otel.arrow.filelog.terminal_unterminated".len() as u64 + 1;
    if c.framing.max_log_size_behavior == OversizeBehavior::Split {
        for (key, bytes) in [
            ("id", 64),
            ("index", 8),
            ("is_last", 1),
            ("body.start", 8),
            ("body.end", 8),
            ("frame.start", 8),
            ("frame.end", 8),
        ] {
            attrs += "otel.arrow.filelog.fragment.".len() as u64 + key.len() as u64 + bytes;
        }
    }
    match c.include_file_metadata {
        FileMetadata::None => (),
        FileMetadata::Name => attrs += "log.file.name".len() as u64 + text,
        FileMetadata::Path | FileMetadata::Native => {
            attrs += "log.file.path".len() as u64 + "log.file.name".len() as u64 + 2 * text;
            if c.include_file_metadata == FileMetadata::Native {
                for (key, bytes) in [
                    ("kind", 15),
                    ("native", checkpoint::ADVISORY_PATH_STORED_MAX_BYTES as u64),
                    ("truncated", 1),
                    ("sha256", 64),
                ] {
                    attrs += "otel.arrow.filelog.path.".len() as u64 + key.len() as u64 + bytes;
                }
            }
        }
    }
    let body = if multiline {
        c.framing.max_record_bytes
    } else {
        c.framing.max_line_bytes.min(c.framing.max_record_bytes)
    };
    body.checked_add(attrs)
        .and_then(|v| v.checked_add(128))
        .ok_or_else(|| ConfigError::new("batch/framing", "logical record size overflows"))
}
