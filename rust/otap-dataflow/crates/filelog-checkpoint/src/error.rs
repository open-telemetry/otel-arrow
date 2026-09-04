// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Structured codec errors.

use crate::FileId;

/// Structural failure while decoding checkpoint bytes.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum DecodeError {
    /// A fixed-width object had the wrong total size.
    #[error("{context} has length {actual}, expected {expected}")]
    InvalidLength {
        /// Object being decoded.
        context: &'static str,
        /// Required size.
        expected: usize,
        /// Observed size.
        actual: usize,
    },
    /// Fewer bytes remained than a field required.
    #[error("truncated input: needed {needed} bytes, {available} available")]
    Truncated {
        /// Required bytes.
        needed: usize,
        /// Remaining bytes.
        available: usize,
    },
    /// A declared length exceeded its absolute format maximum.
    #[error("field {field} declared length {declared} exceeds maximum {max}")]
    LengthExceedsMaximum {
        /// Field name.
        field: &'static str,
        /// Untrusted declared length.
        declared: u64,
        /// Maximum permitted length.
        max: u64,
    },
    /// Magic bytes were not recognized.
    #[error("bad magic for {context}")]
    BadMagic {
        /// Object carrying the magic.
        context: &'static str,
    },
    /// A format or envelope version is unsupported.
    #[error("unsupported version {found} in {context}; migration required")]
    UnsupportedVersion {
        /// Versioned object.
        context: &'static str,
        /// Version found on disk.
        found: u16,
    },
    /// A reserved field or bit was nonzero.
    #[error("reserved field {field} was nonzero: {value:#x}")]
    ReservedFieldNonZero {
        /// Field name.
        field: &'static str,
        /// Observed value.
        value: u64,
    },
    /// A structural discriminant was unknown.
    #[error("unknown discriminant for {field}: {value:#x}")]
    UnknownDiscriminant {
        /// Field name.
        field: &'static str,
        /// Observed value.
        value: u32,
    },
    /// A CRC-32C checksum did not match.
    #[error("CRC-32C mismatch in {context}: stored {stored:#010x}, computed {computed:#010x}")]
    ChecksumMismatch {
        /// Protected object.
        context: &'static str,
        /// Stored checksum.
        stored: u32,
        /// Recomputed checksum.
        computed: u32,
    },
    /// A string field was not valid UTF-8.
    #[error("field {field} is not valid UTF-8")]
    InvalidUtf8 {
        /// Field name.
        field: &'static str,
    },
    /// Checked arithmetic overflowed.
    #[error("arithmetic overflow while computing {context}")]
    ArithmeticOverflow {
        /// Calculation being performed.
        context: &'static str,
    },
    /// Defined fields did not consume an exact length-delimited payload.
    #[error("{context} declared {declared} bytes but fields consumed {consumed}")]
    UnconsumedBytes {
        /// Framed object.
        context: &'static str,
        /// Declared payload length.
        declared: usize,
        /// Bytes consumed by defined fields.
        consumed: usize,
    },
    /// Bytes followed a complete snapshot.
    #[error("{context} has {remaining} trailing bytes")]
    TrailingBytes {
        /// Container name.
        context: &'static str,
        /// Unexpected bytes.
        remaining: usize,
    },
    /// A transaction sequence did not match the required next value.
    #[error("WAL sequence out of order: expected {expected}, found {found}")]
    SequenceOutOfOrder {
        /// Required sequence.
        expected: u64,
        /// Sequence found.
        found: u64,
    },
    /// A caller supplied zero as the next expected WAL sequence.
    #[error("expected WAL sequence must be nonzero")]
    ExpectedSequenceZero,
    /// A transaction carried zero operations.
    #[error("transaction {sequence} has zero operations")]
    EmptyTransaction {
        /// Transaction sequence.
        sequence: u64,
    },
    /// A transaction exceeded its class operation limit.
    #[error("transaction {sequence} has {op_count} operations, maximum {max}")]
    TooManyOperations {
        /// Transaction sequence.
        sequence: u64,
        /// Operation count.
        op_count: u16,
        /// Class-specific maximum.
        max: u16,
    },
    /// Progress and non-progress operations were mixed.
    #[error("transaction {sequence} mixes progress and non-progress operations")]
    MixedTransactionClass {
        /// Transaction sequence.
        sequence: u64,
    },
    /// A progress transaction repeated one file key.
    #[error("transaction {sequence} repeats progress for {file_id:?}")]
    DuplicateProgressFileId {
        /// Transaction sequence.
        sequence: u64,
        /// Repeated key.
        file_id: FileId,
    },
    /// A transaction body was outside the structural range.
    #[error("transaction {sequence} body length {len} is outside {min}..={max}")]
    TransactionBodyOutOfBounds {
        /// Transaction sequence.
        sequence: u64,
        /// Body length.
        len: u64,
        /// Minimum body length.
        min: u64,
        /// Maximum body length.
        max: u64,
    },
    /// Length complement did not match.
    #[error("transaction {sequence} has an invalid body length complement")]
    LengthComplementMismatch {
        /// Transaction sequence.
        sequence: u64,
    },
    /// Artifact namespace digest did not match the expected namespace.
    #[error("{context} namespace digest does not match")]
    NamespaceMismatch {
        /// Artifact name.
        context: &'static str,
    },
    /// A snapshot declared more records than the caller permits.
    #[error("snapshot declares {declared} records, caller maximum is {max}")]
    SnapshotRecordCountExceedsLimit {
        /// Untrusted record count from the validated snapshot header.
        declared: u32,
        /// Caller-provided maximum record count.
        max: u32,
    },
    /// An authenticated snapshot count cannot fit in the supplied bytes.
    #[error(
        "snapshot declares {declared} records, but {snapshot_bytes} bytes can physically contain at most {max}"
    )]
    SnapshotRecordCountExceedsPhysicalMaximum {
        /// Untrusted record count from the validated snapshot header.
        declared: u32,
        /// Maximum complete minimum-width record frames the bytes can contain.
        max: u64,
        /// Complete supplied snapshot width.
        snapshot_bytes: usize,
    },
    /// A snapshot record violated a self-contained reachable-state rule.
    #[error("snapshot record {file_id:?} is invalid: {reason}")]
    InvalidSnapshotState {
        /// Record key.
        file_id: FileId,
        /// Specific violated invariant.
        reason: &'static str,
    },
    /// A mandatory string was empty.
    #[error("field {field} must be non-empty")]
    EmptyRequiredField {
        /// Field name.
        field: &'static str,
    },
    /// A conditionally absent field was present.
    #[error("field {field} must be absent")]
    UnexpectedPresentField {
        /// Field name.
        field: &'static str,
    },
    /// A snapshot repeated a file key.
    #[error("duplicate file_id {file_id:?} in snapshot")]
    DuplicateFileId {
        /// Repeated key.
        file_id: FileId,
    },
    /// Two live snapshot records claimed one locator.
    #[error("live locator is claimed by both {first:?} and {second:?}")]
    DuplicateLiveLocator {
        /// First claimant.
        first: FileId,
        /// Second claimant.
        second: FileId,
    },
    /// An advisory path violated its shape rules.
    #[error("advisory path field {field} is invalid: {reason}")]
    InvalidAdvisoryPath {
        /// Field name.
        field: &'static str,
        /// Violated rule.
        reason: &'static str,
    },
    /// A committed-frontier guard is not canonical for its carried offset.
    #[error("field {field} is not a valid committed-frontier guard for offset {offset}")]
    InvalidCommittedFrontierGuard {
        /// Operation field carrying the guard.
        field: &'static str,
        /// Offset that determines the required window and empty digest.
        offset: u64,
    },
}

/// Failure while encoding a constructed checkpoint value.
#[derive(Debug, thiserror::Error, PartialEq, Eq)]
pub enum EncodeError {
    /// A field exceeded its absolute format maximum.
    #[error("field {field} length {len} exceeds maximum {max}")]
    FieldTooLong {
        /// Field name.
        field: &'static str,
        /// Actual length.
        len: usize,
        /// Maximum length.
        max: usize,
    },
    /// A mandatory field was empty.
    #[error("field {field} must be non-empty")]
    RequiredFieldEmpty {
        /// Field name.
        field: &'static str,
    },
    /// A value has no valid version 1 representation.
    #[error("field {field} is invalid: {reason}")]
    InvalidFieldValue {
        /// Field name.
        field: &'static str,
        /// Violated rule.
        reason: &'static str,
    },
    /// A current-version encoder was asked to emit a reserved reason.
    #[error("field {field} uses reserved reason {reason_code:#06x}")]
    ReservedReasonCode {
        /// Field name.
        field: &'static str,
        /// Reserved code.
        reason_code: u16,
    },
    /// A transaction carried zero operations.
    #[error("transaction {sequence} has zero operations")]
    EmptyTransaction {
        /// Transaction sequence.
        sequence: u64,
    },
    /// A transaction exceeded its class operation limit.
    #[error("transaction {sequence} has {op_count} operations, maximum {max}")]
    TooManyOperations {
        /// Transaction sequence.
        sequence: u64,
        /// Actual operation count.
        op_count: usize,
        /// Class-specific maximum.
        max: u16,
    },
    /// Progress and non-progress operations were mixed.
    #[error("transaction {sequence} mixes progress and non-progress operations")]
    MixedTransactionClass {
        /// Transaction sequence.
        sequence: u64,
    },
    /// A progress transaction repeated one file key.
    #[error("transaction {sequence} repeats progress for {file_id:?}")]
    DuplicateProgressFileId {
        /// Transaction sequence.
        sequence: u64,
        /// Repeated key.
        file_id: FileId,
    },
    /// A transaction body exceeded the hard limit.
    #[error("transaction {sequence} body length {len} exceeds maximum {max}")]
    TransactionBodyTooLarge {
        /// Transaction sequence.
        sequence: u64,
        /// Encoded body length.
        len: u64,
        /// Maximum body length.
        max: u64,
    },
    /// A snapshot repeated a file key.
    #[error("duplicate file_id {file_id:?} in snapshot")]
    DuplicateFileId {
        /// Repeated key.
        file_id: FileId,
    },
    /// Two live snapshot records claimed one locator.
    #[error("live locator is claimed by both {first:?} and {second:?}")]
    DuplicateLiveLocator {
        /// First claimant.
        first: FileId,
        /// Second claimant.
        second: FileId,
    },
    /// A snapshot record violated a self-contained invariant.
    #[error("snapshot record {file_id:?} is invalid: {reason}")]
    InvalidSnapshotState {
        /// Record key.
        file_id: FileId,
        /// Violated invariant.
        reason: &'static str,
    },
    /// An advisory path could not be represented.
    #[error("advisory path is invalid: {reason}")]
    InvalidAdvisoryPath {
        /// Violated invariant.
        reason: &'static str,
    },
    /// Checked encoding arithmetic overflowed.
    #[error("arithmetic overflow while computing {context}")]
    ArithmeticOverflow {
        /// Calculation being performed.
        context: &'static str,
    },
}
