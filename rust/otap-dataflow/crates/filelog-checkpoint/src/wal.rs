// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Version 1 WAL header, operation, transaction, and scanner codec.

use std::collections::HashSet;

use crate::primitives::{
    AUDIT_REASON_MAX_BYTES, FILELOG_FORMAT_VERSION, FINGERPRINT_MAX_BYTES, FRAMING_PROFILE_VERSION,
    NAMESPACE_ID_MAX_BYTES, Reader, TX_ENVELOPE_VERSION, TX_MAGIC, WAL_MAGIC, Writer, crc32c,
    quarantine_reason_reserved,
};
use crate::{
    AdvisoryPath, CommittedFrontierGuard, DecodeError, EncodeError, FileId, FramingResume,
    LifecycleState, Locator,
};

/// Exact version 1 WAL header width.
pub const WAL_HEADER_BYTES: usize = 56;
/// Exact version 1 transaction header width.
pub const TX_HEADER_BYTES: usize = 36;
const TX_FRAME_CRC_BYTES: usize = 4;
/// Maximum operations in a progress-only transaction.
pub const WAL_MAX_OPS_PER_TX: u16 = 4096;
/// Maximum operations in a non-progress transaction.
pub const WAL_MAX_NON_PROGRESS_OPS_PER_TX: u16 = 256;
/// Absolute transaction body maximum.
pub const WAL_MAX_TX_BODY_BYTES: u64 = 16 * 1024 * 1024;
/// Minimum semantically valid transaction body.
pub const TX_MIN_BODY_BYTES: u64 = 34;
/// Largest structurally representable version 1 operation payload.
pub const MAX_OPERATION_PAYLOAD_BYTES: u64 = 131_095;
/// Largest semantically valid `update_fingerprint` operation payload.
pub const MAX_VALID_UPDATE_FINGERPRINT_PAYLOAD_BYTES: u64 = 131_094;
/// Largest progress-only transaction body.
pub const MAX_PROGRESS_TX_BODY_BYTES: u64 = 446_464;
/// Minimum complete version 1 transaction frame.
pub const TX_MIN_FRAME_BYTES: u64 =
    TX_HEADER_BYTES as u64 + TX_MIN_BODY_BYTES + TX_FRAME_CRC_BYTES as u64;
/// Maximum complete version 1 transaction frame.
pub const WAL_MAX_TX_FRAME_BYTES: u64 =
    TX_HEADER_BYTES as u64 + WAL_MAX_TX_BODY_BYTES + TX_FRAME_CRC_BYTES as u64;
/// Maximum complete progress-only transaction frame.
pub const MAX_PROGRESS_TX_FRAME_BYTES: u64 =
    TX_HEADER_BYTES as u64 + MAX_PROGRESS_TX_BODY_BYTES + TX_FRAME_CRC_BYTES as u64;

const OP_REGISTER_FILE: u8 = 0x01;
const OP_UPDATE_PROGRESS: u8 = 0x02;
const OP_RESET_AFTER_TRUNCATE: u8 = 0x03;
const OP_UPDATE_FINGERPRINT: u8 = 0x04;
const OP_UPDATE_METADATA: u8 = 0x05;
const OP_QUARANTINE_FILE: u8 = 0x06;
const OP_RESET_QUARANTINED_FILE: u8 = 0x07;
const OP_REMOVE_FILE: u8 = 0x08;
const METADATA_PATH_PRESENT: u8 = 0x01;

/// Fixed fields decoded from one version 1 WAL header.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WalHeader {
    /// WAL generation.
    pub generation: u64,
    /// Exact namespace digest.
    pub namespace_digest: [u8; 32],
}

/// Registers a newly observed file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegisterFile {
    /// New durable file key.
    pub file_id: FileId,
    /// Initial file epoch.
    pub file_epoch: u32,
    /// Initial committed offset.
    pub committed_offset: u64,
    /// Initial committed-frontier evidence.
    pub committed_frontier_guard: CommittedFrontierGuard,
    /// Initial fingerprint evidence.
    pub fingerprint: Vec<u8>,
    /// Bytes ignored before fingerprinting.
    pub ignored_header_bytes: u32,
    /// Initial locator.
    pub locator: Locator,
    /// Framing-profile structural version.
    pub framing_profile_version: u16,
    /// Framing-profile digest.
    pub framing_profile_digest: [u8; 32],
    /// Initial framing resume.
    pub framing_resume: FramingResume,
    /// Last-observed time as Unix nanoseconds.
    pub last_seen_time_unix_nano: u64,
    /// Initial advisory path.
    pub advisory_path: AdvisoryPath,
}

/// Advances one file's committed progress.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateProgress {
    /// Target file key.
    pub file_id: FileId,
    /// Expected stored offset.
    pub expected_committed_offset: u64,
    /// Expected stored epoch.
    pub expected_file_epoch: u32,
    /// New committed offset.
    pub new_committed_offset: u64,
    /// New frontier evidence.
    pub new_committed_frontier_guard: CommittedFrontierGuard,
    /// New framing resume.
    pub new_framing_resume: FramingResume,
    /// New last-observed time as Unix nanoseconds.
    pub new_last_seen_time_unix_nano: u64,
    /// Whether to finalize the rotated file atomically.
    pub finalize: bool,
}

/// Resets a file after truncation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetAfterTruncate {
    /// Target file key.
    pub file_id: FileId,
    /// Expected active epoch.
    pub expected_active_epoch: u32,
    /// Observed truncated size.
    pub observed_truncated_size: u64,
    /// Resulting epoch.
    pub resulting_epoch: u32,
    /// Resulting committed offset.
    pub new_committed_offset: u64,
    /// Resulting framing resume.
    pub new_framing_resume: FramingResume,
    /// Replacement fingerprint.
    pub new_fingerprint: Vec<u8>,
    /// Reset time as Unix nanoseconds.
    pub reset_time_unix_nano: u64,
    /// Opaque reset reason.
    pub reason_code: u16,
}

/// Strictly extends fingerprint evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateFingerprint {
    /// Target file key.
    pub file_id: FileId,
    /// Expected stored epoch.
    pub expected_file_epoch: u32,
    /// Expected current fingerprint.
    pub expected_fingerprint: Vec<u8>,
    /// Strictly extended fingerprint.
    pub new_fingerprint: Vec<u8>,
}

/// Updates advisory metadata without changing identity.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UpdateMetadata {
    /// Target file key.
    pub file_id: FileId,
    /// Expected lifecycle.
    pub expected_prior_state: LifecycleState,
    /// Expected stored epoch.
    pub expected_file_epoch: u32,
    /// New last-observed time as Unix nanoseconds.
    pub last_seen_time_unix_nano: u64,
    /// Optional replacement advisory path.
    pub advisory_path: Option<AdvisoryPath>,
}

/// Quarantines one file with immutable evidence.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QuarantineFile {
    /// Target file key.
    pub file_id: FileId,
    /// Expected active epoch.
    pub expected_file_epoch: u32,
    /// Opaque diagnostic reason.
    pub reason_code: u16,
    /// Immutable locator evidence.
    pub locator: Locator,
    /// Observed file size.
    pub observed_size: u64,
    /// Epoch at quarantine time.
    pub quarantine_epoch: u32,
    /// Quarantine time as Unix nanoseconds.
    pub quarantine_time_unix_nano: u64,
}

/// Administrative reset decision for a quarantined file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResetQuarantineAction {
    /// Resume at offset zero with a new epoch.
    ResetToBeginning,
    /// Resume at an externally selected end offset with a new epoch.
    ResetToEnd,
    /// Keep the immutable quarantined record unchanged.
    KeepFailed,
}

impl ResetQuarantineAction {
    const fn to_wire(self) -> u8 {
        match self {
            Self::ResetToBeginning => 1,
            Self::ResetToEnd => 2,
            Self::KeepFailed => 3,
        }
    }

    fn from_wire(value: u8) -> Result<Self, DecodeError> {
        match value {
            1 => Ok(Self::ResetToBeginning),
            2 => Ok(Self::ResetToEnd),
            3 => Ok(Self::KeepFailed),
            _ => Err(DecodeError::UnknownDiscriminant {
                field: "reset_quarantined_file.action",
                value: u32::from(value),
            }),
        }
    }
}

/// Audited operation over one quarantined file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResetQuarantinedFile {
    /// Target file key.
    pub file_id: FileId,
    /// Expected quarantine epoch.
    pub expected_quarantine_epoch: u32,
    /// Requested action.
    pub action: ResetQuarantineAction,
    /// Resulting epoch supplied by the caller.
    pub resulting_epoch: u32,
    /// Resulting offset supplied by the caller.
    pub resulting_offset: u64,
    /// Resulting frontier evidence supplied by the caller.
    pub new_committed_frontier_guard: CommittedFrontierGuard,
    /// Resulting framing resume supplied by the caller.
    pub new_framing_resume: FramingResume,
    /// Resulting fingerprint supplied by the caller.
    pub new_fingerprint: Vec<u8>,
    /// Action time as Unix nanoseconds.
    pub action_time_unix_nano: u64,
    /// Exact checkpoint namespace ID.
    pub namespace_id: String,
    /// Mandatory operator audit reason.
    pub audit_reason: String,
}

/// Removes one file record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoveFile {
    /// Target file key.
    pub file_id: FileId,
    /// Expected stored epoch.
    pub expected_file_epoch: u32,
    /// Expected lifecycle.
    pub expected_prior_state: LifecycleState,
    /// Opaque removal reason.
    pub removal_reason: u16,
    /// Removal time as Unix nanoseconds.
    pub removal_time_unix_nano: u64,
    /// Whether this is an operator-authorized removal.
    pub administrative: bool,
    /// Namespace ID present exactly for administrative removal.
    pub namespace_id: Option<String>,
    /// Audit reason present exactly for administrative removal.
    pub audit_reason: Option<String>,
}

/// One version 1 operation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    /// Register a file.
    RegisterFile(RegisterFile),
    /// Advance progress.
    UpdateProgress(UpdateProgress),
    /// Reset after truncation.
    ResetAfterTruncate(ResetAfterTruncate),
    /// Extend fingerprint evidence.
    UpdateFingerprint(UpdateFingerprint),
    /// Update advisory metadata.
    UpdateMetadata(UpdateMetadata),
    /// Quarantine a file.
    QuarantineFile(QuarantineFile),
    /// Apply an audited quarantine decision.
    ResetQuarantinedFile(ResetQuarantinedFile),
    /// Remove a file.
    RemoveFile(RemoveFile),
}

impl Operation {
    /// Returns the operation's target file key.
    #[must_use]
    pub const fn file_id(&self) -> FileId {
        match self {
            Self::RegisterFile(op) => op.file_id,
            Self::UpdateProgress(op) => op.file_id,
            Self::ResetAfterTruncate(op) => op.file_id,
            Self::UpdateFingerprint(op) => op.file_id,
            Self::UpdateMetadata(op) => op.file_id,
            Self::QuarantineFile(op) => op.file_id,
            Self::ResetQuarantinedFile(op) => op.file_id,
            Self::RemoveFile(op) => op.file_id,
        }
    }

    // Guard canonicality is a format-level decode requirement. Resume, locator,
    // lifecycle, and prior-state reachability are validated during PR1B replay.
    fn validate_decoded_structure(&self) -> Result<(), DecodeError> {
        let (field, guard, offset) = match self {
            Self::RegisterFile(op) => (
                "register_file.committed_frontier_guard",
                &op.committed_frontier_guard,
                op.committed_offset,
            ),
            Self::UpdateProgress(op) => (
                "update_progress.new_committed_frontier_guard",
                &op.new_committed_frontier_guard,
                op.new_committed_offset,
            ),
            Self::ResetQuarantinedFile(op) => (
                "reset_quarantined_file.new_committed_frontier_guard",
                &op.new_committed_frontier_guard,
                op.resulting_offset,
            ),
            Self::ResetAfterTruncate(_)
            | Self::UpdateFingerprint(_)
            | Self::UpdateMetadata(_)
            | Self::QuarantineFile(_)
            | Self::RemoveFile(_) => return Ok(()),
        };
        if !guard.valid_for_offset(offset) {
            return Err(DecodeError::InvalidCommittedFrontierGuard { field, offset });
        }
        Ok(())
    }

    fn validate_for_encode(&self) -> Result<(), EncodeError> {
        match self {
            Self::RegisterFile(op) => {
                require_nonzero_epoch("register_file.file_epoch", op.file_epoch)?;
                if op.file_epoch != 1 {
                    return Err(invalid_field(
                        "register_file.file_epoch",
                        "must be one for a new record",
                    ));
                }
                require_guard_for_offset(
                    "register_file.committed_frontier_guard",
                    &op.committed_frontier_guard,
                    op.committed_offset,
                )?;
                if op.locator == Locator::Unspecified {
                    return Err(invalid_field(
                        "register_file.locator",
                        "must not be Unspecified",
                    ));
                }
                if op.framing_profile_version != FRAMING_PROFILE_VERSION {
                    return Err(invalid_field(
                        "register_file.framing_profile_version",
                        "version 1 producers must write version one",
                    ));
                }
                if op.framing_resume != FramingResume::Clean {
                    return Err(invalid_field(
                        "register_file.framing_resume",
                        "new records require a clean framing resume",
                    ));
                }
            }
            Self::UpdateProgress(op) => {
                require_nonzero_epoch(
                    "update_progress.expected_file_epoch",
                    op.expected_file_epoch,
                )?;
                if op.new_committed_offset < op.expected_committed_offset {
                    return Err(invalid_field(
                        "update_progress.new_committed_offset",
                        "must not move backward",
                    ));
                }
                require_guard_for_offset(
                    "update_progress.new_committed_frontier_guard",
                    &op.new_committed_frontier_guard,
                    op.new_committed_offset,
                )?;
                if !op
                    .new_framing_resume
                    .valid_for_offset(op.new_committed_offset)
                {
                    return Err(invalid_field(
                        "update_progress.new_framing_resume",
                        "is inconsistent with new_committed_offset",
                    ));
                }
                if op.finalize && op.new_framing_resume != FramingResume::Clean {
                    return Err(invalid_field(
                        "update_progress.new_framing_resume",
                        "finalization requires a clean framing resume",
                    ));
                }
            }
            Self::ResetAfterTruncate(op) => {
                require_nonzero_epoch(
                    "reset_after_truncate.expected_active_epoch",
                    op.expected_active_epoch,
                )?;
                require_next_epoch(
                    "reset_after_truncate.resulting_epoch",
                    op.expected_active_epoch,
                    op.resulting_epoch,
                )?;
                if op.new_committed_offset != 0 {
                    return Err(invalid_field(
                        "reset_after_truncate.new_committed_offset",
                        "must be zero",
                    ));
                }
                if op.new_framing_resume != FramingResume::Clean {
                    return Err(invalid_field(
                        "reset_after_truncate.new_framing_resume",
                        "must be clean",
                    ));
                }
                if op.reason_code != 1 {
                    return Err(invalid_field(
                        "reset_after_truncate.reason_code",
                        "version 1 producers must write reason one",
                    ));
                }
            }
            Self::UpdateFingerprint(op) => {
                require_nonzero_epoch(
                    "update_fingerprint.expected_file_epoch",
                    op.expected_file_epoch,
                )?;
                if op.new_fingerprint.len() <= op.expected_fingerprint.len()
                    || !op.new_fingerprint.starts_with(&op.expected_fingerprint)
                {
                    return Err(invalid_field(
                        "update_fingerprint.new_fingerprint",
                        "must strictly extend expected_fingerprint",
                    ));
                }
            }
            Self::UpdateMetadata(op) => {
                require_nonzero_epoch(
                    "update_metadata.expected_file_epoch",
                    op.expected_file_epoch,
                )?;
                if op.expected_prior_state == LifecycleState::RotatedFinalized {
                    return Err(invalid_field(
                        "update_metadata.expected_prior_state",
                        "must be Active or Quarantined",
                    ));
                }
            }
            Self::QuarantineFile(op) => {
                require_nonzero_epoch(
                    "quarantine_file.expected_file_epoch",
                    op.expected_file_epoch,
                )?;
                if quarantine_reason_reserved(op.reason_code) {
                    return Err(EncodeError::ReservedReasonCode {
                        field: "quarantine_file.reason_code",
                        reason_code: op.reason_code,
                    });
                }
                if op.locator == Locator::Unspecified {
                    return Err(invalid_field(
                        "quarantine_file.locator",
                        "must not be Unspecified",
                    ));
                }
                if op.quarantine_epoch != op.expected_file_epoch {
                    return Err(invalid_field(
                        "quarantine_file.quarantine_epoch",
                        "must equal expected_file_epoch",
                    ));
                }
            }
            Self::ResetQuarantinedFile(op) => {
                require_nonzero_epoch(
                    "reset_quarantined_file.expected_quarantine_epoch",
                    op.expected_quarantine_epoch,
                )?;
                if op.namespace_id.is_empty() {
                    return Err(EncodeError::RequiredFieldEmpty {
                        field: "reset_quarantined_file.namespace_id",
                    });
                }
                if op.audit_reason.is_empty() {
                    return Err(EncodeError::RequiredFieldEmpty {
                        field: "reset_quarantined_file.audit_reason",
                    });
                }
                match op.action {
                    ResetQuarantineAction::ResetToBeginning => {
                        require_next_epoch(
                            "reset_quarantined_file.resulting_epoch",
                            op.expected_quarantine_epoch,
                            op.resulting_epoch,
                        )?;
                        if op.resulting_offset != 0 {
                            return Err(invalid_field(
                                "reset_quarantined_file.resulting_offset",
                                "reset_to_beginning requires zero",
                            ));
                        }
                        require_guard_for_offset(
                            "reset_quarantined_file.new_committed_frontier_guard",
                            &op.new_committed_frontier_guard,
                            0,
                        )?;
                        if op.new_framing_resume != FramingResume::Clean {
                            return Err(invalid_field(
                                "reset_quarantined_file.new_framing_resume",
                                "reset_to_beginning requires clean",
                            ));
                        }
                    }
                    ResetQuarantineAction::ResetToEnd => {
                        require_next_epoch(
                            "reset_quarantined_file.resulting_epoch",
                            op.expected_quarantine_epoch,
                            op.resulting_epoch,
                        )?;
                        require_guard_for_offset(
                            "reset_quarantined_file.new_committed_frontier_guard",
                            &op.new_committed_frontier_guard,
                            op.resulting_offset,
                        )?;
                        if op.new_framing_resume != FramingResume::Clean {
                            return Err(invalid_field(
                                "reset_quarantined_file.new_framing_resume",
                                "reset_to_end requires clean",
                            ));
                        }
                    }
                    ResetQuarantineAction::KeepFailed => {
                        if op.resulting_epoch != op.expected_quarantine_epoch {
                            return Err(invalid_field(
                                "reset_quarantined_file.resulting_epoch",
                                "must equal expected_quarantine_epoch for keep_failed",
                            ));
                        }
                        // All other equality with stored state is deliberately a PR1B rule.
                        require_guard_for_offset(
                            "reset_quarantined_file.new_committed_frontier_guard",
                            &op.new_committed_frontier_guard,
                            op.resulting_offset,
                        )?;
                    }
                }
            }
            Self::RemoveFile(op) => {
                require_nonzero_epoch("remove_file.expected_file_epoch", op.expected_file_epoch)?;
                if op.removal_reason == 0 {
                    return Err(EncodeError::ReservedReasonCode {
                        field: "remove_file.removal_reason",
                        reason_code: op.removal_reason,
                    });
                }
                if !op.administrative && op.expected_prior_state != LifecycleState::Active {
                    return Err(invalid_field(
                        "remove_file.expected_prior_state",
                        "non-administrative removal requires Active",
                    ));
                }
                match (op.administrative, &op.namespace_id, &op.audit_reason) {
                    (true, Some(namespace), Some(reason)) => {
                        if namespace.is_empty() {
                            return Err(EncodeError::RequiredFieldEmpty {
                                field: "remove_file.namespace_id",
                            });
                        }
                        if reason.is_empty() {
                            return Err(EncodeError::RequiredFieldEmpty {
                                field: "remove_file.audit_reason",
                            });
                        }
                    }
                    (false, None, None) => {}
                    _ => {
                        return Err(invalid_field(
                            "remove_file.administrative",
                            "flag and administrative fields must agree",
                        ));
                    }
                }
            }
        }
        Ok(())
    }

    fn encode_payload(&self) -> Result<Vec<u8>, EncodeError> {
        self.validate_for_encode()?;
        let mut out = Writer::new();
        match self {
            Self::RegisterFile(op) => {
                out.u8(OP_REGISTER_FILE);
                out.bytes(op.file_id.as_bytes());
                out.u32(op.file_epoch);
                out.u64(op.committed_offset);
                op.committed_frontier_guard.write(&mut out);
                out.var_bytes(
                    "register_file.fingerprint",
                    &op.fingerprint,
                    FINGERPRINT_MAX_BYTES,
                )?;
                out.u32(op.ignored_header_bytes);
                op.locator.write(&mut out);
                out.u16(op.framing_profile_version);
                out.bytes(&op.framing_profile_digest);
                op.framing_resume.write(&mut out);
                out.u64(op.last_seen_time_unix_nano);
                op.advisory_path.write(&mut out);
            }
            Self::UpdateProgress(op) => {
                out.u8(OP_UPDATE_PROGRESS);
                out.bytes(op.file_id.as_bytes());
                out.u64(op.expected_committed_offset);
                out.u32(op.expected_file_epoch);
                out.u64(op.new_committed_offset);
                op.new_committed_frontier_guard.write(&mut out);
                op.new_framing_resume.write(&mut out);
                out.u64(op.new_last_seen_time_unix_nano);
                out.u8(u8::from(op.finalize));
            }
            Self::ResetAfterTruncate(op) => {
                out.u8(OP_RESET_AFTER_TRUNCATE);
                out.bytes(op.file_id.as_bytes());
                out.u32(op.expected_active_epoch);
                out.u64(op.observed_truncated_size);
                out.u32(op.resulting_epoch);
                out.u64(op.new_committed_offset);
                op.new_framing_resume.write(&mut out);
                out.var_bytes(
                    "reset_after_truncate.new_fingerprint",
                    &op.new_fingerprint,
                    FINGERPRINT_MAX_BYTES,
                )?;
                out.u64(op.reset_time_unix_nano);
                out.u16(op.reason_code);
            }
            Self::UpdateFingerprint(op) => {
                out.u8(OP_UPDATE_FINGERPRINT);
                out.bytes(op.file_id.as_bytes());
                out.u32(op.expected_file_epoch);
                out.var_bytes(
                    "update_fingerprint.expected_fingerprint",
                    &op.expected_fingerprint,
                    FINGERPRINT_MAX_BYTES,
                )?;
                out.var_bytes(
                    "update_fingerprint.new_fingerprint",
                    &op.new_fingerprint,
                    FINGERPRINT_MAX_BYTES,
                )?;
            }
            Self::UpdateMetadata(op) => {
                out.u8(OP_UPDATE_METADATA);
                out.bytes(op.file_id.as_bytes());
                out.u8(op.expected_prior_state.to_wire());
                out.u32(op.expected_file_epoch);
                out.u8(if op.advisory_path.is_some() {
                    METADATA_PATH_PRESENT
                } else {
                    0
                });
                out.u64(op.last_seen_time_unix_nano);
                if let Some(path) = &op.advisory_path {
                    path.write(&mut out);
                }
            }
            Self::QuarantineFile(op) => {
                out.u8(OP_QUARANTINE_FILE);
                out.bytes(op.file_id.as_bytes());
                out.u32(op.expected_file_epoch);
                out.u16(op.reason_code);
                op.locator.write(&mut out);
                out.u64(op.observed_size);
                out.u32(op.quarantine_epoch);
                out.u64(op.quarantine_time_unix_nano);
            }
            Self::ResetQuarantinedFile(op) => {
                out.u8(OP_RESET_QUARANTINED_FILE);
                out.bytes(op.file_id.as_bytes());
                out.u32(op.expected_quarantine_epoch);
                out.u8(op.action.to_wire());
                out.u32(op.resulting_epoch);
                out.u64(op.resulting_offset);
                op.new_committed_frontier_guard.write(&mut out);
                op.new_framing_resume.write(&mut out);
                out.var_bytes(
                    "reset_quarantined_file.new_fingerprint",
                    &op.new_fingerprint,
                    FINGERPRINT_MAX_BYTES,
                )?;
                out.u64(op.action_time_unix_nano);
                out.var_bytes(
                    "reset_quarantined_file.namespace_id",
                    op.namespace_id.as_bytes(),
                    NAMESPACE_ID_MAX_BYTES,
                )?;
                out.var_bytes(
                    "reset_quarantined_file.audit_reason",
                    op.audit_reason.as_bytes(),
                    AUDIT_REASON_MAX_BYTES,
                )?;
            }
            Self::RemoveFile(op) => {
                out.u8(OP_REMOVE_FILE);
                out.bytes(op.file_id.as_bytes());
                out.u32(op.expected_file_epoch);
                out.u8(op.expected_prior_state.to_wire());
                out.u16(op.removal_reason);
                out.u64(op.removal_time_unix_nano);
                out.u8(u8::from(op.administrative));
                match (op.administrative, &op.namespace_id, &op.audit_reason) {
                    (true, Some(namespace), Some(reason)) => {
                        out.var_bytes(
                            "remove_file.namespace_id",
                            namespace.as_bytes(),
                            NAMESPACE_ID_MAX_BYTES,
                        )?;
                        out.var_bytes(
                            "remove_file.audit_reason",
                            reason.as_bytes(),
                            AUDIT_REASON_MAX_BYTES,
                        )?;
                    }
                    (false, None, None) => {
                        out.u16(0);
                        out.u16(0);
                    }
                    _ => unreachable!("validated administrative field shape"),
                }
            }
        }
        Ok(out.finish())
    }

    fn decode_payload(bytes: &[u8]) -> Result<Self, DecodeError> {
        let mut input = Reader::new(bytes);
        let op_code = input.u8()?;
        let file_id = FileId::from_bytes(input.array()?);
        let operation = match op_code {
            OP_REGISTER_FILE => Self::RegisterFile(RegisterFile {
                file_id,
                file_epoch: input.u32()?,
                committed_offset: input.u64()?,
                committed_frontier_guard: CommittedFrontierGuard::read(&mut input)?,
                fingerprint: input
                    .var_bytes("register_file.fingerprint", FINGERPRINT_MAX_BYTES)?
                    .to_vec(),
                ignored_header_bytes: input.u32()?,
                locator: Locator::read(&mut input)?,
                framing_profile_version: input.u16()?,
                framing_profile_digest: input.array()?,
                framing_resume: FramingResume::read(&mut input)?,
                last_seen_time_unix_nano: input.u64()?,
                advisory_path: AdvisoryPath::read(&mut input)?,
            }),
            OP_UPDATE_PROGRESS => Self::UpdateProgress(UpdateProgress {
                file_id,
                expected_committed_offset: input.u64()?,
                expected_file_epoch: input.u32()?,
                new_committed_offset: input.u64()?,
                new_committed_frontier_guard: CommittedFrontierGuard::read(&mut input)?,
                new_framing_resume: FramingResume::read(&mut input)?,
                new_last_seen_time_unix_nano: input.u64()?,
                finalize: read_bool(&mut input, "update_progress.finalize")?,
            }),
            OP_RESET_AFTER_TRUNCATE => Self::ResetAfterTruncate(ResetAfterTruncate {
                file_id,
                expected_active_epoch: input.u32()?,
                observed_truncated_size: input.u64()?,
                resulting_epoch: input.u32()?,
                new_committed_offset: input.u64()?,
                new_framing_resume: FramingResume::read(&mut input)?,
                new_fingerprint: input
                    .var_bytes(
                        "reset_after_truncate.new_fingerprint",
                        FINGERPRINT_MAX_BYTES,
                    )?
                    .to_vec(),
                reset_time_unix_nano: input.u64()?,
                reason_code: input.u16()?,
            }),
            OP_UPDATE_FINGERPRINT => Self::UpdateFingerprint(UpdateFingerprint {
                file_id,
                expected_file_epoch: input.u32()?,
                expected_fingerprint: input
                    .var_bytes(
                        "update_fingerprint.expected_fingerprint",
                        FINGERPRINT_MAX_BYTES,
                    )?
                    .to_vec(),
                new_fingerprint: input
                    .var_bytes("update_fingerprint.new_fingerprint", FINGERPRINT_MAX_BYTES)?
                    .to_vec(),
            }),
            OP_UPDATE_METADATA => {
                let expected_prior_state = match input.u8()? {
                    1 => LifecycleState::Active,
                    3 => LifecycleState::Quarantined,
                    value => {
                        return Err(DecodeError::UnknownDiscriminant {
                            field: "update_metadata.expected_prior_state",
                            value: u32::from(value),
                        });
                    }
                };
                let expected_file_epoch = input.u32()?;
                let presence = input.u8()?;
                if presence & !METADATA_PATH_PRESENT != 0 {
                    return Err(DecodeError::ReservedFieldNonZero {
                        field: "update_metadata.presence_flags",
                        value: u64::from(presence),
                    });
                }
                let last_seen_time_unix_nano = input.u64()?;
                let advisory_path = if presence & METADATA_PATH_PRESENT != 0 {
                    Some(AdvisoryPath::read(&mut input)?)
                } else {
                    None
                };
                Self::UpdateMetadata(UpdateMetadata {
                    file_id,
                    expected_prior_state,
                    expected_file_epoch,
                    last_seen_time_unix_nano,
                    advisory_path,
                })
            }
            OP_QUARANTINE_FILE => Self::QuarantineFile(QuarantineFile {
                file_id,
                expected_file_epoch: input.u32()?,
                reason_code: input.u16()?,
                locator: Locator::read(&mut input)?,
                observed_size: input.u64()?,
                quarantine_epoch: input.u32()?,
                quarantine_time_unix_nano: input.u64()?,
            }),
            OP_RESET_QUARANTINED_FILE => {
                let expected_quarantine_epoch = input.u32()?;
                let action = ResetQuarantineAction::from_wire(input.u8()?)?;
                let resulting_epoch = input.u32()?;
                let resulting_offset = input.u64()?;
                let new_committed_frontier_guard = CommittedFrontierGuard::read(&mut input)?;
                let new_framing_resume = FramingResume::read(&mut input)?;
                let new_fingerprint = input
                    .var_bytes(
                        "reset_quarantined_file.new_fingerprint",
                        FINGERPRINT_MAX_BYTES,
                    )?
                    .to_vec();
                let action_time_unix_nano = input.u64()?;
                let namespace_id = input
                    .var_string(
                        "reset_quarantined_file.namespace_id",
                        NAMESPACE_ID_MAX_BYTES,
                    )?
                    .to_owned();
                if namespace_id.is_empty() {
                    return Err(DecodeError::EmptyRequiredField {
                        field: "reset_quarantined_file.namespace_id",
                    });
                }
                let audit_reason = input
                    .var_string(
                        "reset_quarantined_file.audit_reason",
                        AUDIT_REASON_MAX_BYTES,
                    )?
                    .to_owned();
                if audit_reason.is_empty() {
                    return Err(DecodeError::EmptyRequiredField {
                        field: "reset_quarantined_file.audit_reason",
                    });
                }
                Self::ResetQuarantinedFile(ResetQuarantinedFile {
                    file_id,
                    expected_quarantine_epoch,
                    action,
                    resulting_epoch,
                    resulting_offset,
                    new_committed_frontier_guard,
                    new_framing_resume,
                    new_fingerprint,
                    action_time_unix_nano,
                    namespace_id,
                    audit_reason,
                })
            }
            OP_REMOVE_FILE => {
                let expected_file_epoch = input.u32()?;
                let expected_prior_state =
                    LifecycleState::from_wire(input.u8()?, "remove_file.expected_prior_state")?;
                let removal_reason = input.u16()?;
                let removal_time_unix_nano = input.u64()?;
                let administrative = read_bool(&mut input, "remove_file.administrative")?;
                let namespace =
                    input.var_string("remove_file.namespace_id", NAMESPACE_ID_MAX_BYTES)?;
                let reason =
                    input.var_string("remove_file.audit_reason", AUDIT_REASON_MAX_BYTES)?;
                let (namespace_id, audit_reason) = if administrative {
                    if namespace.is_empty() {
                        return Err(DecodeError::EmptyRequiredField {
                            field: "remove_file.namespace_id",
                        });
                    }
                    if reason.is_empty() {
                        return Err(DecodeError::EmptyRequiredField {
                            field: "remove_file.audit_reason",
                        });
                    }
                    (Some(namespace.to_owned()), Some(reason.to_owned()))
                } else {
                    if !namespace.is_empty() {
                        return Err(DecodeError::UnexpectedPresentField {
                            field: "remove_file.namespace_id",
                        });
                    }
                    if !reason.is_empty() {
                        return Err(DecodeError::UnexpectedPresentField {
                            field: "remove_file.audit_reason",
                        });
                    }
                    (None, None)
                };
                Self::RemoveFile(RemoveFile {
                    file_id,
                    expected_file_epoch,
                    expected_prior_state,
                    removal_reason,
                    removal_time_unix_nano,
                    administrative,
                    namespace_id,
                    audit_reason,
                })
            }
            value => {
                return Err(DecodeError::UnknownDiscriminant {
                    field: "operation.op_code",
                    value: u32::from(value),
                });
            }
        };
        if input.remaining() != 0 {
            return Err(DecodeError::UnconsumedBytes {
                context: "WAL operation",
                declared: bytes.len(),
                consumed: input.position(),
            });
        }
        operation.validate_decoded_structure()?;
        Ok(operation)
    }
}

fn invalid_field(field: &'static str, reason: &'static str) -> EncodeError {
    EncodeError::InvalidFieldValue { field, reason }
}

fn require_nonzero_epoch(field: &'static str, epoch: u32) -> Result<(), EncodeError> {
    if epoch == 0 {
        return Err(invalid_field(field, "must be nonzero"));
    }
    Ok(())
}

fn require_next_epoch(
    field: &'static str,
    expected_epoch: u32,
    resulting_epoch: u32,
) -> Result<(), EncodeError> {
    let next = expected_epoch
        .checked_add(1)
        .ok_or_else(|| invalid_field(field, "expected epoch cannot be incremented"))?;
    if resulting_epoch != next {
        return Err(invalid_field(
            field,
            "must be exactly one greater than expected epoch",
        ));
    }
    Ok(())
}

fn require_guard_for_offset(
    field: &'static str,
    guard: &CommittedFrontierGuard,
    offset: u64,
) -> Result<(), EncodeError> {
    if !guard.valid_for_offset(offset) {
        return Err(invalid_field(
            field,
            "must have the required window and canonical empty digest",
        ));
    }
    Ok(())
}

fn read_bool(input: &mut Reader<'_>, field: &'static str) -> Result<bool, DecodeError> {
    match input.u8()? {
        0 => Ok(false),
        1 => Ok(true),
        value => Err(DecodeError::UnknownDiscriminant {
            field,
            value: u32::from(value),
        }),
    }
}

/// Encodes one self-delimiting version 1 operation frame.
pub fn encode_operation(operation: &Operation) -> Result<Vec<u8>, EncodeError> {
    let payload = operation.encode_payload()?;
    let payload_len =
        u32::try_from(payload.len()).map_err(|_| EncodeError::ArithmeticOverflow {
            context: "operation payload length",
        })?;
    let mut out = Writer::new();
    out.u32(payload_len);
    out.bytes(&payload);
    out.u32(crc32c(out.as_slice()));
    Ok(out.finish())
}

/// Decodes one operation frame and returns its exact consumed width.
pub fn decode_operation(bytes: &[u8]) -> Result<(Operation, usize), DecodeError> {
    let mut input = Reader::new(bytes);
    let declared = u64::from(input.u32()?);
    if declared > MAX_OPERATION_PAYLOAD_BYTES {
        return Err(DecodeError::LengthExceedsMaximum {
            field: "wal_operation.op_len",
            declared,
            max: MAX_OPERATION_PAYLOAD_BYTES,
        });
    }
    let payload_len = usize::try_from(declared).map_err(|_| DecodeError::ArithmeticOverflow {
        context: "operation payload length to usize",
    })?;
    let payload = input.exact(payload_len)?;
    let stored = input.u32()?;
    let consumed = input.position();
    let computed = crc32c(&bytes[..consumed - 4]);
    if stored != computed {
        return Err(DecodeError::ChecksumMismatch {
            context: "WAL operation",
            stored,
            computed,
        });
    }
    Ok((Operation::decode_payload(payload)?, consumed))
}

/// Structurally valid transaction class.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TransactionClass {
    /// Every operation advances progress.
    ProgressOnly,
    /// No operation advances progress.
    NonProgress,
}

impl TransactionClass {
    /// Returns the class-specific operation-count limit.
    #[must_use]
    pub const fn max_operations(self) -> u16 {
        match self {
            Self::ProgressOnly => WAL_MAX_OPS_PER_TX,
            Self::NonProgress => WAL_MAX_NON_PROGRESS_OPS_PER_TX,
        }
    }
}

/// One atomic, strictly sequenced version 1 WAL transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    /// Sequence number, beginning at one in each WAL generation.
    pub sequence: u64,
    /// Operations in exact on-disk order.
    pub operations: Vec<Operation>,
}

fn classify_operations(operations: &[Operation]) -> Option<TransactionClass> {
    let progress = operations
        .iter()
        .filter(|operation| matches!(operation, Operation::UpdateProgress(_)))
        .count();
    if progress == operations.len() && progress != 0 {
        Some(TransactionClass::ProgressOnly)
    } else if progress == 0 && !operations.is_empty() {
        Some(TransactionClass::NonProgress)
    } else {
        None
    }
}

fn validate_encode_transaction(transaction: &Transaction) -> Result<TransactionClass, EncodeError> {
    if transaction.sequence == 0 {
        return Err(EncodeError::InvalidFieldValue {
            field: "transaction.sequence",
            reason: "must be nonzero",
        });
    }
    if transaction.operations.is_empty() {
        return Err(EncodeError::EmptyTransaction {
            sequence: transaction.sequence,
        });
    }
    let Some(class) = classify_operations(&transaction.operations) else {
        return Err(EncodeError::MixedTransactionClass {
            sequence: transaction.sequence,
        });
    };
    if transaction.operations.len() > usize::from(class.max_operations()) {
        return Err(EncodeError::TooManyOperations {
            sequence: transaction.sequence,
            op_count: transaction.operations.len(),
            max: class.max_operations(),
        });
    }
    if class == TransactionClass::ProgressOnly {
        let mut file_ids = HashSet::with_capacity(transaction.operations.len());
        for operation in &transaction.operations {
            let file_id = operation.file_id();
            if !file_ids.insert(file_id) {
                return Err(EncodeError::DuplicateProgressFileId {
                    sequence: transaction.sequence,
                    file_id,
                });
            }
        }
    }
    Ok(class)
}

/// Encodes one complete transaction frame.
pub fn encode_transaction(transaction: &Transaction) -> Result<Vec<u8>, EncodeError> {
    let _class = validate_encode_transaction(transaction)?;
    let mut body = Writer::new();
    for operation in &transaction.operations {
        let frame = encode_operation(operation)?;
        let next_len = body.as_slice().len().checked_add(frame.len()).ok_or(
            EncodeError::ArithmeticOverflow {
                context: "transaction body length",
            },
        )?;
        if next_len as u64 > WAL_MAX_TX_BODY_BYTES {
            return Err(EncodeError::TransactionBodyTooLarge {
                sequence: transaction.sequence,
                len: next_len as u64,
                max: WAL_MAX_TX_BODY_BYTES,
            });
        }
        body.bytes(&frame);
    }
    let body_len =
        u64::try_from(body.as_slice().len()).map_err(|_| EncodeError::ArithmeticOverflow {
            context: "transaction body length",
        })?;
    if body_len > WAL_MAX_TX_BODY_BYTES {
        return Err(EncodeError::TransactionBodyTooLarge {
            sequence: transaction.sequence,
            len: body_len,
            max: WAL_MAX_TX_BODY_BYTES,
        });
    }
    let body_len_u32 = u32::try_from(body_len).map_err(|_| EncodeError::ArithmeticOverflow {
        context: "transaction body length to u32",
    })?;
    let op_count = u16::try_from(transaction.operations.len()).map_err(|_| {
        EncodeError::ArithmeticOverflow {
            context: "transaction operation count",
        }
    })?;

    let mut out = Writer::new();
    out.bytes(TX_MAGIC);
    out.u16(TX_ENVELOPE_VERSION);
    out.u16(0);
    out.u64(transaction.sequence);
    out.u32(body_len_u32);
    out.u32(!body_len_u32);
    out.u16(op_count);
    out.u16(0);
    out.u32(crc32c(out.as_slice()));
    out.bytes(body.as_slice());
    out.u32(crc32c(out.as_slice()));
    Ok(out.finish())
}

fn decode_transaction_body(
    sequence: u64,
    op_count: u16,
    body: &[u8],
) -> Result<Transaction, DecodeError> {
    let mut operations = Vec::with_capacity(usize::from(op_count));
    let mut cursor = 0usize;
    for _ in 0..op_count {
        let remaining = body.get(cursor..).ok_or(DecodeError::Truncated {
            needed: 4,
            available: 0,
        })?;
        let (operation, consumed) = decode_operation(remaining)?;
        cursor = cursor
            .checked_add(consumed)
            .ok_or(DecodeError::ArithmeticOverflow {
                context: "transaction operation cursor",
            })?;
        operations.push(operation);
    }
    if cursor != body.len() {
        return Err(DecodeError::UnconsumedBytes {
            context: "WAL transaction body",
            declared: body.len(),
            consumed: cursor,
        });
    }
    let Some(class) = classify_operations(&operations) else {
        return Err(if operations.is_empty() {
            DecodeError::EmptyTransaction { sequence }
        } else {
            DecodeError::MixedTransactionClass { sequence }
        });
    };
    if op_count > class.max_operations() {
        return Err(DecodeError::TooManyOperations {
            sequence,
            op_count,
            max: class.max_operations(),
        });
    }
    if class == TransactionClass::ProgressOnly {
        let mut file_ids = HashSet::with_capacity(operations.len());
        for operation in &operations {
            let file_id = operation.file_id();
            if !file_ids.insert(file_id) {
                return Err(DecodeError::DuplicateProgressFileId { sequence, file_id });
            }
        }
    }
    Ok(Transaction {
        sequence,
        operations,
    })
}

/// Result of scanning one transaction from a WAL suffix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransactionScan {
    /// One complete, validated transaction and its exact encoded width.
    Complete {
        /// Decoded transaction.
        transaction: Transaction,
        /// Bytes consumed from the supplied suffix.
        consumed: usize,
    },
    /// The supplied non-empty suffix cannot contain the complete transaction
    /// described by a validated header, or is shorter than a complete header.
    ///
    /// This codec does not know whether the slice reaches physical EOF. A
    /// future store may classify this as a permitted final torn tail only
    /// after proving EOF; otherwise it must read again without discarding.
    Incomplete {
        /// Incomplete suffix width.
        bytes: usize,
    },
}

/// Scans at most one transaction from a WAL suffix.
///
/// An empty suffix returns `None`. Callers can validate and apply a complete
/// transaction, advance by `consumed`, and drop it before scanning the next.
/// This keeps recovery memory bounded by one transaction. The expected
/// sequence must be nonzero; a fresh WAL generation begins at one.
///
/// `TransactionScan::Incomplete` describes only the supplied slice. It never
/// proves physical EOF and never authorizes truncation or byte discard.
pub fn scan_next_transaction(
    bytes: &[u8],
    expected_sequence: u64,
) -> Result<Option<TransactionScan>, DecodeError> {
    if expected_sequence == 0 {
        return Err(DecodeError::ExpectedSequenceZero);
    }
    if bytes.is_empty() {
        return Ok(None);
    }
    if bytes.len() < TX_HEADER_BYTES {
        return Ok(Some(TransactionScan::Incomplete { bytes: bytes.len() }));
    }
    let header_bytes = &bytes[..TX_HEADER_BYTES];
    let mut header = Reader::new(header_bytes);
    let magic = header.exact(8)?;
    let version = header.u16()?;
    let flags = header.u16()?;
    let sequence = header.u64()?;
    let body_len = header.u32()?;
    let complement = header.u32()?;
    let op_count = header.u16()?;
    let reserved = header.u16()?;
    let stored_header_crc = header.u32()?;

    if magic != TX_MAGIC {
        return Err(DecodeError::BadMagic {
            context: "WAL transaction header",
        });
    }
    if version != TX_ENVELOPE_VERSION {
        return Err(DecodeError::UnsupportedVersion {
            context: "WAL transaction envelope",
            found: version,
        });
    }
    if flags != 0 {
        return Err(DecodeError::ReservedFieldNonZero {
            field: "wal_transaction.tx_flags",
            value: u64::from(flags),
        });
    }
    if reserved != 0 {
        return Err(DecodeError::ReservedFieldNonZero {
            field: "wal_transaction.reserved",
            value: u64::from(reserved),
        });
    }
    if complement != !body_len {
        return Err(DecodeError::LengthComplementMismatch { sequence });
    }
    let computed_header_crc = crc32c(&header_bytes[..32]);
    if stored_header_crc != computed_header_crc {
        return Err(DecodeError::ChecksumMismatch {
            context: "WAL transaction header",
            stored: stored_header_crc,
            computed: computed_header_crc,
        });
    }
    let body_len_u64 = u64::from(body_len);
    if !(TX_MIN_BODY_BYTES..=WAL_MAX_TX_BODY_BYTES).contains(&body_len_u64) {
        return Err(DecodeError::TransactionBodyOutOfBounds {
            sequence,
            len: body_len_u64,
            min: TX_MIN_BODY_BYTES,
            max: WAL_MAX_TX_BODY_BYTES,
        });
    }
    if op_count == 0 {
        return Err(DecodeError::EmptyTransaction { sequence });
    }
    if op_count > WAL_MAX_OPS_PER_TX {
        return Err(DecodeError::TooManyOperations {
            sequence,
            op_count,
            max: WAL_MAX_OPS_PER_TX,
        });
    }
    if sequence != expected_sequence {
        return Err(DecodeError::SequenceOutOfOrder {
            expected: expected_sequence,
            found: sequence,
        });
    }
    let body_len_usize =
        usize::try_from(body_len).map_err(|_| DecodeError::ArithmeticOverflow {
            context: "transaction body length to usize",
        })?;
    let needed = TX_HEADER_BYTES
        .checked_add(body_len_usize)
        .and_then(|value| value.checked_add(TX_FRAME_CRC_BYTES))
        .ok_or(DecodeError::ArithmeticOverflow {
            context: "transaction frame length",
        })?;
    if bytes.len() < needed {
        return Ok(Some(TransactionScan::Incomplete { bytes: bytes.len() }));
    }
    let frame_crc_offset = needed - TX_FRAME_CRC_BYTES;
    let stored_frame_crc =
        u32::from_be_bytes(bytes[frame_crc_offset..needed].try_into().map_err(|_| {
            DecodeError::Truncated {
                needed: TX_FRAME_CRC_BYTES,
                available: needed - frame_crc_offset,
            }
        })?);
    let computed_frame_crc = crc32c(&bytes[..frame_crc_offset]);
    if stored_frame_crc != computed_frame_crc {
        return Err(DecodeError::ChecksumMismatch {
            context: "WAL transaction frame",
            stored: stored_frame_crc,
            computed: computed_frame_crc,
        });
    }
    let body = &bytes[TX_HEADER_BYTES..frame_crc_offset];
    Ok(Some(TransactionScan::Complete {
        transaction: decode_transaction_body(sequence, op_count, body)?,
        consumed: needed,
    }))
}

/// Encodes an exact version 1 WAL header.
pub fn encode_wal_header(generation: u64, checkpoint_id: &str) -> Result<Vec<u8>, EncodeError> {
    let namespace = crate::namespace_digest(checkpoint_id)?;
    let mut out = Writer::new();
    out.bytes(WAL_MAGIC);
    out.u16(FILELOG_FORMAT_VERSION);
    out.u16(0);
    out.u64(generation);
    out.bytes(&namespace);
    out.u32(crc32c(out.as_slice()));
    Ok(out.finish())
}

/// Decodes one exact version 1 WAL header.
///
/// This validates and returns the namespace digest encoded in the header. A
/// future store must compare that digest with the selected checkpoint
/// namespace before replaying any transaction.
pub fn decode_wal_header(bytes: &[u8]) -> Result<WalHeader, DecodeError> {
    if bytes.len() != WAL_HEADER_BYTES {
        return Err(DecodeError::InvalidLength {
            context: "WAL header",
            expected: WAL_HEADER_BYTES,
            actual: bytes.len(),
        });
    }
    let mut input = Reader::new(bytes);
    if input.exact(8)? != WAL_MAGIC {
        return Err(DecodeError::BadMagic {
            context: "WAL header",
        });
    }
    let version = input.u16()?;
    if version != FILELOG_FORMAT_VERSION {
        return Err(DecodeError::UnsupportedVersion {
            context: "WAL header",
            found: version,
        });
    }
    let flags = input.u16()?;
    if flags != 0 {
        return Err(DecodeError::ReservedFieldNonZero {
            field: "wal_header.flags",
            value: u64::from(flags),
        });
    }
    let generation = input.u64()?;
    let namespace_digest = input.array()?;
    let stored = input.u32()?;
    let computed = crc32c(&bytes[..52]);
    if stored != computed {
        return Err(DecodeError::ChecksumMismatch {
            context: "WAL header",
            stored,
            computed,
        });
    }
    Ok(WalHeader {
        generation,
        namespace_digest,
    })
}
