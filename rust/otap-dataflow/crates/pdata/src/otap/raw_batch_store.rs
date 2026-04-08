// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! A raw batch store that provides payload-type-indexed storage of Arrow
//! [`RecordBatch`]es without OTAP schema validation.
//!
//! [`RawBatchStore`] is the inner storage type used by the validated
//! [`OtapBatchStore`](super::OtapBatchStore) implementations (`Logs`, `Metrics`,
//! `Traces`). It can also be used directly by terminal consumers (e.g. the
//! Parquet exporter) that legitimately transform batches in ways that may not
//! conform to the OTAP wire-protocol schema.

use arrow::array::RecordBatch;

use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;

// ---------------------------------------------------------------------------
// Position lookup — maps ArrowPayloadType discriminants to array indices
// ---------------------------------------------------------------------------

/// Sentinel value for unused slots in [`POSITION_LOOKUP`].
pub const UNUSED_INDEX: usize = 99;

/// Maps [`ArrowPayloadType`] enum discriminants to positions in the
/// per-signal batch arrays. Shared across Logs, Metrics, and Traces
/// (each signal only uses a subset of the indices).
pub const POSITION_LOOKUP: &[usize] = &[
    UNUSED_INDEX, // Unknown = 0,
    // common:
    0,            // ResourceAttrs = 1,
    1,            // ScopeAttrs = 2,
    UNUSED_INDEX, // 3
    UNUSED_INDEX, // 4
    UNUSED_INDEX, // 5
    UNUSED_INDEX, // 6
    UNUSED_INDEX, // 7
    UNUSED_INDEX, // 8
    UNUSED_INDEX, // 9
    // metrics:
    2,            // UnivariateMetrics = 10,
    3,            // NumberDataPoints = 11,
    4,            // SummaryDataPoints = 12,
    5,            // HistogramDataPoints = 13,
    6,            // ExpHistogramDataPoints = 14,
    7,            // NumberDpAttrs = 15,
    8,            // SummaryDpAttrs = 16,
    9,            // HistogramDpAttrs = 17,
    10,           // ExpHistogramDpAttrs = 18,
    11,           // NumberDpExemplars = 19,
    12,           // HistogramDpExemplars = 20,
    13,           // ExpHistogramDpExemplars = 21,
    14,           // NumberDpExemplarAttrs = 22,
    15,           // HistogramDpExemplarAttrs = 23,
    16,           // ExpHistogramDpExemplarAttrs = 24,
    17,           // MultivariateMetrics = 25,
    18,           // MetricAttrs = 26,
    UNUSED_INDEX, // 27
    UNUSED_INDEX, // 28
    UNUSED_INDEX, // 29
    // logs:
    2,            // Logs = 30,
    3,            // LogAttrs = 31,
    UNUSED_INDEX, // 32
    UNUSED_INDEX, // 33
    UNUSED_INDEX, // 34
    UNUSED_INDEX, // 35
    UNUSED_INDEX, // 36
    UNUSED_INDEX, // 37
    UNUSED_INDEX, // 38
    UNUSED_INDEX, // 39
    // traces:
    2, // Spans = 40,
    3, // SpanAttrs = 41,
    4, // SpanEvents = 42,
    5, // SpanLinks = 43,
    6, // SpanEventAttrs = 44,
    7, // SpanLinkAttrs = 45,
];

// ---------------------------------------------------------------------------
// Constants — type masks and counts for each signal type
// ---------------------------------------------------------------------------

/// Bitmask of valid [`ArrowPayloadType`] values for the Logs signal.
pub const LOGS_TYPE_MASK: u64 = (1 << ArrowPayloadType::ResourceAttrs as u64)
    + (1 << ArrowPayloadType::ScopeAttrs as u64)
    + (1 << ArrowPayloadType::Logs as u64)
    + (1 << ArrowPayloadType::LogAttrs as u64);

/// Number of payload slots for the Logs signal.
pub const LOGS_COUNT: usize = 4;

/// Bitmask of valid [`ArrowPayloadType`] values for the Metrics signal.
pub const METRICS_TYPE_MASK: u64 = (1 << ArrowPayloadType::ResourceAttrs as u64)
    + (1 << ArrowPayloadType::ScopeAttrs as u64)
    + (1 << ArrowPayloadType::UnivariateMetrics as u64)
    + (1 << ArrowPayloadType::MultivariateMetrics as u64)
    + (1 << ArrowPayloadType::NumberDataPoints as u64)
    + (1 << ArrowPayloadType::SummaryDataPoints as u64)
    + (1 << ArrowPayloadType::HistogramDataPoints as u64)
    + (1 << ArrowPayloadType::ExpHistogramDataPoints as u64)
    + (1 << ArrowPayloadType::NumberDpAttrs as u64)
    + (1 << ArrowPayloadType::SummaryDpAttrs as u64)
    + (1 << ArrowPayloadType::HistogramDpAttrs as u64)
    + (1 << ArrowPayloadType::ExpHistogramDpAttrs as u64)
    + (1 << ArrowPayloadType::NumberDpExemplars as u64)
    + (1 << ArrowPayloadType::HistogramDpExemplars as u64)
    + (1 << ArrowPayloadType::ExpHistogramDpExemplars as u64)
    + (1 << ArrowPayloadType::NumberDpExemplarAttrs as u64)
    + (1 << ArrowPayloadType::HistogramDpExemplarAttrs as u64)
    + (1 << ArrowPayloadType::ExpHistogramDpExemplarAttrs as u64)
    + (1 << ArrowPayloadType::MetricAttrs as u64);

/// Number of payload slots for the Metrics signal.
pub const METRICS_COUNT: usize = 19;

/// Bitmask of valid [`ArrowPayloadType`] values for the Traces signal.
pub const TRACES_TYPE_MASK: u64 = (1 << ArrowPayloadType::ResourceAttrs as u64)
    + (1 << ArrowPayloadType::ScopeAttrs as u64)
    + (1 << ArrowPayloadType::Spans as u64)
    + (1 << ArrowPayloadType::SpanAttrs as u64)
    + (1 << ArrowPayloadType::SpanEvents as u64)
    + (1 << ArrowPayloadType::SpanLinks as u64)
    + (1 << ArrowPayloadType::SpanEventAttrs as u64)
    + (1 << ArrowPayloadType::SpanLinkAttrs as u64);

/// Number of payload slots for the Traces signal.
pub const TRACES_COUNT: usize = 8;

// ---------------------------------------------------------------------------
// Type aliases
// ---------------------------------------------------------------------------

/// Raw (unvalidated) batch store for the Logs signal.
pub type RawLogsStore = RawBatchStore<LOGS_TYPE_MASK, LOGS_COUNT>;

/// Raw (unvalidated) batch store for the Metrics signal.
pub type RawMetricsStore = RawBatchStore<METRICS_TYPE_MASK, METRICS_COUNT>;

/// Raw (unvalidated) batch store for the Traces signal.
pub type RawTracesStore = RawBatchStore<TRACES_TYPE_MASK, TRACES_COUNT>;

// ---------------------------------------------------------------------------
// RawBatchStore
// ---------------------------------------------------------------------------

/// A fixed-size, payload-type-indexed store of optional [`RecordBatch`]es.
///
/// The `TYPE_MASK` const generic is a bitmask indicating which
/// [`ArrowPayloadType`] values are valid for this store. The `COUNT` const
/// generic is the number of slots in the backing array.
///
/// This type provides **no** OTAP schema validation. Callers that need
/// validation should use the [`OtapBatchStore`](super::OtapBatchStore) trait
/// implementations which wrap this type.
#[derive(Clone, Debug, PartialEq)]
pub struct RawBatchStore<const TYPE_MASK: u64, const COUNT: usize> {
    batches: [Option<RecordBatch>; COUNT],
}

impl<const TYPE_MASK: u64, const COUNT: usize> Default for RawBatchStore<TYPE_MASK, COUNT> {
    fn default() -> Self {
        Self {
            batches: std::array::from_fn(|_| None),
        }
    }
}

impl<const TYPE_MASK: u64, const COUNT: usize> RawBatchStore<TYPE_MASK, COUNT> {
    /// Create a new empty store with all slots set to `None`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a store from a pre-built batch array.
    #[must_use]
    pub fn from_batches(batches: [Option<RecordBatch>; COUNT]) -> Self {
        Self { batches }
    }

    /// Check whether the given payload type is valid for this store.
    #[must_use]
    pub fn is_valid_type(payload_type: ArrowPayloadType) -> bool {
        TYPE_MASK & (1 << payload_type as u64) != 0
    }

    /// Read-only access to the underlying batch array as a slice.
    #[must_use]
    pub fn batches(&self) -> &[Option<RecordBatch>] {
        &self.batches
    }

    /// Mutable access to the underlying batch array as a slice.
    pub fn batches_mut(&mut self) -> &mut [Option<RecordBatch>] {
        &mut self.batches
    }

    /// Consume the store and return the underlying batch array.
    #[must_use]
    pub fn into_batches(self) -> [Option<RecordBatch>; COUNT] {
        self.batches
    }

    /// Get a reference to the batch for the given payload type, if present.
    ///
    /// Returns `None` if the payload type is not valid for this store or if
    /// no batch has been set for that slot.
    #[must_use]
    pub fn get(&self, payload_type: ArrowPayloadType) -> Option<&RecordBatch> {
        if !Self::is_valid_type(payload_type) {
            return None;
        }
        let idx = POSITION_LOOKUP[payload_type as usize];
        debug_assert!(idx != UNUSED_INDEX);
        self.batches[idx].as_ref()
    }

    /// Set the batch for the given payload type.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `payload_type` is not valid for this store.
    /// Callers must ensure the type is valid (see [`Self::is_valid_type`]).
    pub fn set(&mut self, payload_type: ArrowPayloadType, record_batch: RecordBatch) {
        debug_assert!(
            Self::is_valid_type(payload_type),
            "payload type {payload_type:?} is not valid for this store"
        );
        let idx = POSITION_LOOKUP[payload_type as usize];
        debug_assert!(idx != UNUSED_INDEX);
        self.batches[idx] = Some(record_batch);
    }

    /// Remove the batch for the given payload type.
    ///
    /// # Panics
    ///
    /// Panics in debug builds if `payload_type` is not valid for this store.
    /// Callers must ensure the type is valid (see [`Self::is_valid_type`]).
    pub fn remove(&mut self, payload_type: ArrowPayloadType) {
        debug_assert!(
            Self::is_valid_type(payload_type),
            "payload type {payload_type:?} is not valid for this store"
        );
        let idx = POSITION_LOOKUP[payload_type as usize];
        debug_assert!(idx != UNUSED_INDEX);
        self.batches[idx] = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn raw_logs_store_basic_operations() {
        use arrow::array::UInt16Array;
        use arrow::datatypes::{DataType, Field, Schema};
        use std::sync::Arc;

        let mut store = RawLogsStore::new();

        // All slots start as None
        assert!(store.get(ArrowPayloadType::Logs).is_none());
        assert!(store.get(ArrowPayloadType::LogAttrs).is_none());

        // Invalid type returns None
        assert!(store.get(ArrowPayloadType::Spans).is_none());

        // Set a batch
        let schema = Arc::new(Schema::new(vec![Field::new("id", DataType::UInt16, true)]));
        let batch =
            RecordBatch::try_new(schema, vec![Arc::new(UInt16Array::from(vec![1u16]))]).unwrap();
        store.set(ArrowPayloadType::Logs, batch);
        assert!(store.get(ArrowPayloadType::Logs).is_some());

        // Remove it
        store.remove(ArrowPayloadType::Logs);
        assert!(store.get(ArrowPayloadType::Logs).is_none());
    }

    /// Exhaustively verify that `is_valid_type` returns `true` for exactly
    /// the payload types listed in `allowed_payload_types()` for each signal,
    /// and `false` for every other known payload type.
    #[test]
    fn type_mask_matches_allowed_payload_types() {
        use crate::otap::{Logs, Metrics, OtapBatchStore, Traces};
        use std::collections::HashSet;

        // Union of all known payload types across all signals, plus Unknown.
        let all_types: HashSet<ArrowPayloadType> = std::iter::once(ArrowPayloadType::Unknown)
            .chain(Logs::allowed_payload_types().iter().copied())
            .chain(Metrics::allowed_payload_types().iter().copied())
            .chain(Traces::allowed_payload_types().iter().copied())
            .collect();

        let cases: &[(&str, fn(ArrowPayloadType) -> bool, &[ArrowPayloadType])] = &[
            (
                "Logs",
                RawLogsStore::is_valid_type,
                Logs::allowed_payload_types(),
            ),
            (
                "Metrics",
                RawMetricsStore::is_valid_type,
                Metrics::allowed_payload_types(),
            ),
            (
                "Traces",
                RawTracesStore::is_valid_type,
                Traces::allowed_payload_types(),
            ),
        ];

        for &(signal, is_valid, allowed) in cases {
            let allowed_set: HashSet<_> = allowed.iter().copied().collect();
            for &pt in &all_types {
                let expected = allowed_set.contains(&pt);
                assert_eq!(
                    is_valid(pt),
                    expected,
                    "{signal}: is_valid_type({pt:?}) should be {expected}"
                );
            }
        }
    }

    #[test]
    fn into_batches_returns_correct_length() {
        let store = RawLogsStore::new();
        assert_eq!(store.into_batches().len(), LOGS_COUNT);

        let store = RawMetricsStore::new();
        assert_eq!(store.into_batches().len(), METRICS_COUNT);

        let store = RawTracesStore::new();
        assert_eq!(store.into_batches().len(), TRACES_COUNT);
    }
}
