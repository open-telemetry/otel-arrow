// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Filelog-specific version 1 checkpoint wire codec.
//!
//! The version 1 on-disk representation is the compatibility contract. This
//! crate's Rust API remains internal and experimental under repository policy.
//! It contains no filesystem, publication, replay-table, receiver-runtime, or
//! administration implementation.

mod current_marker;
mod error;
mod framing_profile;
mod primitives;
mod snapshot;
mod wal;

pub use current_marker::{CURRENT_BYTES, decode_current, encode_current};
pub use error::{DecodeError, EncodeError};
pub use framing_profile::{
    FramingEncoding, FramingOnDecodeError, FramingProfileParams, MaxLogSizeBehavior, MultilineMode,
};
pub use primitives::{
    ADVISORY_PATH_STORED_MAX_BYTES, AUDIT_REASON_MAX_BYTES, AdvisoryPath, AdvisoryPathKind,
    COMMITTED_FRONTIER_GUARD_WINDOW_BYTES, CommittedFrontierGuard, CommittedFrontierWindow,
    FILELOG_FORMAT_VERSION, FINGERPRINT_MAX_BYTES, FRAMING_PROFILE_VERSION, FileId, FramingResume,
    LifecycleState, Locator, NAMESPACE_ID_MAX_BYTES, crc32c, namespace_digest,
};
pub use snapshot::{
    QuarantineEvidence, SNAPSHOT_FOOTER_BYTES, SNAPSHOT_HEADER_BYTES,
    SNAPSHOT_MAX_RECORD_FRAME_BYTES, Snapshot, SnapshotRecord, decode_snapshot, encode_snapshot,
};
pub use wal::{
    MAX_OPERATION_PAYLOAD_BYTES, MAX_PROGRESS_TX_BODY_BYTES, MAX_PROGRESS_TX_FRAME_BYTES,
    MAX_VALID_UPDATE_FINGERPRINT_PAYLOAD_BYTES, Operation, QuarantineFile, RegisterFile,
    RemoveFile, ResetAfterTruncate, ResetQuarantineAction, ResetQuarantinedFile, TX_HEADER_BYTES,
    TX_MIN_BODY_BYTES, TX_MIN_FRAME_BYTES, Transaction, TransactionClass, TransactionScan,
    UpdateFingerprint, UpdateMetadata, UpdateProgress, WAL_HEADER_BYTES,
    WAL_MAX_NON_PROGRESS_OPS_PER_TX, WAL_MAX_OPS_PER_TX, WAL_MAX_TX_BODY_BYTES,
    WAL_MAX_TX_FRAME_BYTES, WalHeader, decode_operation, decode_wal_header, encode_operation,
    encode_transaction, encode_wal_header, scan_next_transaction,
};
