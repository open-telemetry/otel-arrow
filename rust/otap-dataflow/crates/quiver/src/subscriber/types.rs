// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Core type definitions for subscriber management.
//!
//! These types form the vocabulary used throughout the subscriber module for
//! identifying subscribers, referencing bundles, and recording outcomes.

use std::fmt;

use crate::segment::SegmentSeq;

// ─────────────────────────────────────────────────────────────────────────────
// Subscriber Identification
// ─────────────────────────────────────────────────────────────────────────────

/// Maximum length for subscriber IDs.
///
/// Subscriber IDs are persisted in the ack log, so we limit their size to
/// prevent unbounded storage growth and ensure efficient serialization.
pub const MAX_SUBSCRIBER_ID_LEN: usize = 256;

/// Unique identifier for a subscriber.
///
/// Subscriber IDs are human-readable strings that identify independent
/// consumers of the segment stream. Examples: "exporter-otlp", "backup-s3".
///
/// # Validity
///
/// Valid subscriber IDs:
/// - Are non-empty
/// - Are at most [`MAX_SUBSCRIBER_ID_LEN`] bytes
/// - Contain only ASCII alphanumeric characters, hyphens, and underscores
///
/// Use [`SubscriberId::new`] to validate and create instances.
#[derive(Clone, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct SubscriberId(String);

impl SubscriberId {
    /// Creates a new subscriber ID after validation.
    ///
    /// # Errors
    ///
    /// Returns an error if the ID is empty, too long, or contains invalid
    /// characters.
    pub fn new(id: impl Into<String>) -> Result<Self, InvalidSubscriberId> {
        let id = id.into();

        if id.is_empty() {
            return Err(InvalidSubscriberId::Empty);
        }

        if id.len() > MAX_SUBSCRIBER_ID_LEN {
            return Err(InvalidSubscriberId::TooLong {
                len: id.len(),
                max: MAX_SUBSCRIBER_ID_LEN,
            });
        }

        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(InvalidSubscriberId::InvalidCharacters);
        }

        Ok(Self(id))
    }

    /// Returns the ID as a string slice.
    #[inline]
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Consumes the ID and returns the underlying string.
    #[inline]
    #[must_use]
    pub fn into_string(self) -> String {
        self.0
    }
}

impl AsRef<str> for SubscriberId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for SubscriberId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Error returned when creating an invalid [`SubscriberId`].
#[derive(Clone, Debug, PartialEq, Eq, thiserror::Error)]
pub enum InvalidSubscriberId {
    /// The subscriber ID is empty.
    #[error("subscriber ID cannot be empty")]
    Empty,

    /// The subscriber ID exceeds the maximum length.
    #[error("subscriber ID too long ({len} bytes, max {max})")]
    TooLong {
        /// Actual length of the invalid ID.
        len: usize,
        /// Maximum allowed length.
        max: usize,
    },

    /// The subscriber ID contains invalid characters.
    #[error("subscriber ID contains invalid characters (allowed: a-z, A-Z, 0-9, -, _)")]
    InvalidCharacters,
}

// ─────────────────────────────────────────────────────────────────────────────
// Bundle Identification
// ─────────────────────────────────────────────────────────────────────────────

/// Index of a bundle within a segment.
///
/// Bundles within a segment are numbered sequentially starting from 0.
/// This type provides a distinct type from raw u32 for clarity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct BundleIndex(u32);

impl BundleIndex {
    /// Creates a new bundle index.
    #[must_use]
    pub const fn new(raw: u32) -> Self {
        Self(raw)
    }

    /// Returns the raw numeric value.
    #[inline]
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }

    /// Returns the next sequential bundle index.
    #[must_use]
    pub const fn next(self) -> Self {
        Self(self.0.saturating_add(1))
    }
}

impl From<u32> for BundleIndex {
    fn from(raw: u32) -> Self {
        Self::new(raw)
    }
}

impl From<BundleIndex> for u32 {
    fn from(idx: BundleIndex) -> Self {
        idx.0
    }
}

impl fmt::Display for BundleIndex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Lightweight reference to a specific bundle within a segment.
///
/// A `BundleRef` uniquely identifies a bundle across the entire Quiver store
/// by combining the segment sequence number with the bundle's index within
/// that segment.
///
/// This type is designed to be cheap to copy and hold during retry backoff
/// periods. The actual bundle data can be re-read from the memory-mapped
/// segment file via `claim_bundle()` when needed.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BundleRef {
    /// The segment containing the bundle.
    pub segment_seq: SegmentSeq,
    /// The bundle's index within the segment.
    pub bundle_index: BundleIndex,
}

impl BundleRef {
    /// Creates a new bundle reference.
    #[must_use]
    pub const fn new(segment_seq: SegmentSeq, bundle_index: BundleIndex) -> Self {
        Self {
            segment_seq,
            bundle_index,
        }
    }
}

impl fmt::Display for BundleRef {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}", self.segment_seq, self.bundle_index)
    }
}

impl PartialOrd for BundleRef {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for BundleRef {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.segment_seq
            .cmp(&other.segment_seq)
            .then(self.bundle_index.cmp(&other.bundle_index))
    }
}

// ─────────────────────────────────────────────────────────────────────────────
// Acknowledgment Outcomes
// ─────────────────────────────────────────────────────────────────────────────

/// Terminal outcome of bundle processing.
///
/// These are the only outcomes recorded in the durable ack log. Transient
/// states (like "pending retry") are managed by the embedding layer and
/// not persisted by Quiver.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum AckOutcome {
    /// Bundle was successfully processed.
    Acked,
    /// Bundle was permanently dropped after exhausting retries.
    Dropped,
}

impl AckOutcome {
    /// Returns the byte representation for serialization.
    #[must_use]
    pub const fn as_byte(self) -> u8 {
        match self {
            Self::Acked => 0x01,
            Self::Dropped => 0x02,
        }
    }

    /// Parses the outcome from its byte representation.
    #[must_use]
    pub const fn from_byte(byte: u8) -> Option<Self> {
        match byte {
            0x01 => Some(Self::Acked),
            0x02 => Some(Self::Dropped),
            _ => None,
        }
    }
}

impl fmt::Display for AckOutcome {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Acked => write!(f, "acked"),
            Self::Dropped => write!(f, "dropped"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─────────────────────────────────────────────────────────────────────────
    // SubscriberId tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn subscriber_id_valid() {
        let id = SubscriberId::new("exporter-otlp").unwrap();
        assert_eq!(id.as_str(), "exporter-otlp");
    }

    #[test]
    fn subscriber_id_with_underscores() {
        let id = SubscriberId::new("backup_s3_us_west").unwrap();
        assert_eq!(id.as_str(), "backup_s3_us_west");
    }

    #[test]
    fn subscriber_id_alphanumeric() {
        let id = SubscriberId::new("subscriber123").unwrap();
        assert_eq!(id.as_str(), "subscriber123");
    }

    #[test]
    fn subscriber_id_empty_error() {
        let err = SubscriberId::new("").unwrap_err();
        assert_eq!(err, InvalidSubscriberId::Empty);
    }

    #[test]
    fn subscriber_id_too_long_error() {
        let long_id = "x".repeat(MAX_SUBSCRIBER_ID_LEN + 1);
        let err = SubscriberId::new(long_id).unwrap_err();
        assert!(matches!(err, InvalidSubscriberId::TooLong { .. }));
    }

    #[test]
    fn subscriber_id_invalid_chars_error() {
        let err = SubscriberId::new("has spaces").unwrap_err();
        assert_eq!(err, InvalidSubscriberId::InvalidCharacters);

        let err = SubscriberId::new("has.dots").unwrap_err();
        assert_eq!(err, InvalidSubscriberId::InvalidCharacters);

        let err = SubscriberId::new("has/slash").unwrap_err();
        assert_eq!(err, InvalidSubscriberId::InvalidCharacters);
    }

    #[test]
    fn subscriber_id_display() {
        let id = SubscriberId::new("test-sub").unwrap();
        assert_eq!(format!("{id}"), "test-sub");
    }

    #[test]
    fn subscriber_id_ordering() {
        let a = SubscriberId::new("aaa").unwrap();
        let b = SubscriberId::new("bbb").unwrap();
        assert!(a < b);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BundleIndex tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn bundle_index_basic() {
        let idx = BundleIndex::new(42);
        assert_eq!(idx.raw(), 42);
    }

    #[test]
    fn bundle_index_next() {
        let idx = BundleIndex::new(10);
        assert_eq!(idx.next(), BundleIndex::new(11));
    }

    #[test]
    fn bundle_index_saturating() {
        let idx = BundleIndex::new(u32::MAX);
        assert_eq!(idx.next(), BundleIndex::new(u32::MAX));
    }

    #[test]
    fn bundle_index_conversions() {
        let idx = BundleIndex::from(99u32);
        let raw: u32 = idx.into();
        assert_eq!(raw, 99);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // BundleRef tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn bundle_ref_creation() {
        let seg = SegmentSeq::new(100);
        let idx = BundleIndex::new(5);
        let bundle_ref = BundleRef::new(seg, idx);

        assert_eq!(bundle_ref.segment_seq, seg);
        assert_eq!(bundle_ref.bundle_index, idx);
    }

    #[test]
    fn bundle_ref_display() {
        let bundle_ref = BundleRef::new(SegmentSeq::new(42), BundleIndex::new(7));
        assert_eq!(format!("{bundle_ref}"), "42:7");
    }

    #[test]
    fn bundle_ref_ordering() {
        let a = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(5));
        let b = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(10));
        let c = BundleRef::new(SegmentSeq::new(2), BundleIndex::new(1));

        assert!(a < b);
        assert!(b < c);
        assert!(a < c);
    }

    #[test]
    fn bundle_ref_equality() {
        let a = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(5));
        let b = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(5));
        let c = BundleRef::new(SegmentSeq::new(1), BundleIndex::new(6));

        assert_eq!(a, b);
        assert_ne!(a, c);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // AckOutcome tests
    // ─────────────────────────────────────────────────────────────────────────

    #[test]
    fn ack_outcome_byte_roundtrip() {
        assert_eq!(
            AckOutcome::from_byte(AckOutcome::Acked.as_byte()),
            Some(AckOutcome::Acked)
        );
        assert_eq!(
            AckOutcome::from_byte(AckOutcome::Dropped.as_byte()),
            Some(AckOutcome::Dropped)
        );
    }

    #[test]
    fn ack_outcome_invalid_byte() {
        assert_eq!(AckOutcome::from_byte(0x00), None);
        assert_eq!(AckOutcome::from_byte(0x03), None);
        assert_eq!(AckOutcome::from_byte(0xFF), None);
    }

    #[test]
    fn ack_outcome_display() {
        assert_eq!(format!("{}", AckOutcome::Acked), "acked");
        assert_eq!(format!("{}", AckOutcome::Dropped), "dropped");
    }
}
