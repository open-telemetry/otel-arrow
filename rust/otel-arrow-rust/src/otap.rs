// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains various types and methods for interacting with and manipulating
//! OTAP data / record batches

use arrow::array::RecordBatch;

use crate::{
    decode::record_message::RecordMessage, proto::opentelemetry::arrow::v1::ArrowPayloadType,
};

#[allow(missing_docs)]
pub mod transform;

/// The OtapBatch enum is used to represent a batch of OTAP data.
pub enum OtapBatch {
    /// Represents a batch of logs data.
    Logs(Logs),
    /// Represents a batch of metrics data.
    Metrics(Metrics),
    /// Represents a batch of spans data.
    Traces(Traces),
}

impl OtapBatch {
    /// Set the record batch for the given payload type. If the payload type is not valid
    /// for this type of telemetry signal, this method does nothing.
    pub fn set(&mut self, payload_type: ArrowPayloadType, record_batch: RecordBatch) {
        match self {
            Self::Logs(logs) => logs.set(payload_type, record_batch),
            Self::Metrics(metrics) => metrics.set(payload_type, record_batch),
            Self::Traces(spans) => spans.set(payload_type, record_batch),
        }
    }

    /// Get the record batch for the given payload type, if this payload type was included
    /// in the batch. If the payload type is not valid for this type of telemetry signal, this
    /// also method returns None.
    #[must_use]
    pub fn get(&self, payload_type: ArrowPayloadType) -> Option<&RecordBatch> {
        match self {
            Self::Logs(logs) => logs.get(payload_type),
            Self::Metrics(metrics) => metrics.get(payload_type),
            Self::Traces(spans) => spans.get(payload_type),
        }
    }
}

/// The ArrowBatchStore helper trait is used to define a common interface for
/// storing and retrieving Arrow record batches in a type-safe manner. It is
/// implemented by various structs that represent each signal type and provides
/// methods to efficiently set and get record batches.
trait OtapBatchStore: Sized + Default {
    // Internally, implementers should use a bitmask for the types they support.
    // The offsets in the bitmask should correspond to the ArrowPayloadType enum values.
    const TYPE_MASK: u64;

    /// Mutable access to the batch array. The array the implementer returns
    /// should be the size of the number of types it supports, and it should expect
    /// that types to be positioned in the array according to the POSITION_LOOKUP array.
    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>];
    fn batches(&self) -> &[Option<RecordBatch>];

    fn new() -> Self {
        Self::default()
    }

    /// Check if the given payload type is valid for this store.
    /// This is done by checking if the bitmask for the type is set in the
    /// implementer's TYPE_MASK.
    fn is_valid_type(payload_type: ArrowPayloadType) -> bool {
        Self::TYPE_MASK & (1 << payload_type as u64) != 0
    }

    fn set(&mut self, payload_type: ArrowPayloadType, record_batch: RecordBatch) {
        if Self::is_valid_type(payload_type) {
            self.batches_mut()[POSITION_LOOKUP[payload_type as usize]] = Some(record_batch);
        }
    }

    fn get(&self, payload_type: ArrowPayloadType) -> Option<&RecordBatch> {
        if !Self::is_valid_type(payload_type) {
            None
        } else {
            self.batches()[POSITION_LOOKUP[payload_type as usize]].as_ref()
        }
    }
}

/// Convert the list of decoded messages into an OtapBatchStore implementation
#[allow(private_bounds)]
#[must_use]
pub fn from_record_messages<T: OtapBatchStore>(record_messages: Vec<RecordMessage>) -> T {
    let mut batch_store = T::new();
    for message in record_messages {
        batch_store.set(message.payload_type, message.record);
    }

    batch_store
}

/// The POSITION_LOOKUP array is used to map the ArrowPayloadType enum values to
/// positions in predefined arrays for each telemetry signal type. This allows
/// for efficient storage and retrieval of record batches based on their payload type.
const POSITION_LOOKUP: &[usize] = &[
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
    UNUSED_INDEX, // 26
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
const UNUSED_INDEX: usize = 99;

/// Store of record batches for a batch of OTAP logs data.
#[derive(Default)]
pub struct Logs {
    batches: [Option<RecordBatch>; 4],
}

impl OtapBatchStore for Logs {
    const TYPE_MASK: u64 = (1 << ArrowPayloadType::ResourceAttrs as u64)
        + (1 << ArrowPayloadType::ScopeAttrs as u64)
        + (1 << ArrowPayloadType::Logs as u64)
        + (1 << ArrowPayloadType::LogAttrs as u64);

    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>] {
        &mut self.batches
    }

    fn batches(&self) -> &[Option<RecordBatch>] {
        &self.batches
    }
}

/// Store of record batches for a batch of OTAP metrics data.
#[derive(Default)]
pub struct Metrics {
    batches: [Option<RecordBatch>; 18],
}

impl OtapBatchStore for Metrics {
    const TYPE_MASK: u64 = (1 << ArrowPayloadType::ResourceAttrs as u64)
        + (1 << ArrowPayloadType::ScopeAttrs as u64)
        + (1 << ArrowPayloadType::UnivariateMetrics as u64)
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
        + (1 << ArrowPayloadType::MultivariateMetrics as u64);

    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>] {
        &mut self.batches
    }

    fn batches(&self) -> &[Option<RecordBatch>] {
        &self.batches
    }
}

/// Store of record batches for a batch of OTAP traces data.
#[derive(Default)]
pub struct Traces {
    batches: [Option<RecordBatch>; 8],
}

impl OtapBatchStore for Traces {
    const TYPE_MASK: u64 = (1 << ArrowPayloadType::ResourceAttrs as u64)
        + (1 << ArrowPayloadType::ScopeAttrs as u64)
        + (1 << ArrowPayloadType::Spans as u64)
        + (1 << ArrowPayloadType::SpanAttrs as u64)
        + (1 << ArrowPayloadType::SpanEvents as u64)
        + (1 << ArrowPayloadType::SpanLinks as u64)
        + (1 << ArrowPayloadType::SpanEventAttrs as u64)
        + (1 << ArrowPayloadType::SpanLinkAttrs as u64);

    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>] {
        &mut self.batches
    }

    fn batches(&self) -> &[Option<RecordBatch>] {
        &self.batches
    }
}

#[cfg(test)]
mod test {
    use arrow::array::{RecordBatch, UInt8Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_log_getset() {
        let mut otap_batch = OtapBatch::Logs(Logs::new());

        // for purpose of this test, the shape of the data doesn't really matter...
        let schema = Schema::new(vec![Field::new("a", DataType::UInt8, false)]);
        let record_batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(
            UInt8Array::from_iter_values(vec![1]),
        )])
        .unwrap();

        // the assertions here are maybe a bit more robust than the ones
        // below for metrics and spans, but the logic this is testing for each of these is
        // effectively the same for each of the three types

        // assert everything is null to being with
        assert!(otap_batch.get(ArrowPayloadType::ResourceAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::ScopeAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::Logs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());

        // assert it does not panic or anything if try to get an invalid type
        assert!(otap_batch.get(ArrowPayloadType::Unknown).is_none());
        assert!(
            otap_batch
                .get(ArrowPayloadType::MultivariateMetrics)
                .is_none()
        );

        // add some batches and assert everything is correct
        otap_batch.set(ArrowPayloadType::Logs, record_batch.clone());
        assert!(otap_batch.get(ArrowPayloadType::ResourceAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::ScopeAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::Logs).is_some());
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());

        otap_batch.set(ArrowPayloadType::LogAttrs, record_batch.clone());
        assert!(otap_batch.get(ArrowPayloadType::ResourceAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::ScopeAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::Logs).is_some());
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_some());

        otap_batch.set(ArrowPayloadType::ResourceAttrs, record_batch.clone());
        assert!(otap_batch.get(ArrowPayloadType::ResourceAttrs).is_some());
        assert!(otap_batch.get(ArrowPayloadType::ScopeAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::Logs).is_some());
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_some());

        otap_batch.set(ArrowPayloadType::ScopeAttrs, record_batch.clone());
        assert!(otap_batch.get(ArrowPayloadType::ResourceAttrs).is_some());
        assert!(otap_batch.get(ArrowPayloadType::ScopeAttrs).is_some());
        assert!(otap_batch.get(ArrowPayloadType::Logs).is_some());
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_some());
    }

    #[test]
    fn test_metrics_getset() {
        let mut otap_batch = OtapBatch::Metrics(Metrics::new());

        // for purpose of this test, the shape of the data doesn't really matter...
        let schema = Schema::new(vec![Field::new("a", DataType::UInt8, false)]);
        let record_batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(
            UInt8Array::from_iter_values(vec![1]),
        )])
        .unwrap();

        let metric_types = [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
        ];

        for metric_type in metric_types.iter() {
            assert!(otap_batch.get(*metric_type).is_none());
            otap_batch.set(*metric_type, record_batch.clone());
            assert!(
                otap_batch.get(*metric_type).is_some(),
                "Failed for type: {:?}",
                metric_type
            );
        }
    }

    #[test]
    fn test_traces_getset() {
        let mut otap_batch = OtapBatch::Traces(Traces::new());

        // for purpose of this test, the shape of the data doesn't really matter...
        let schema = Schema::new(vec![Field::new("a", DataType::UInt8, false)]);
        let record_batch = RecordBatch::try_new(Arc::new(schema), vec![Arc::new(
            UInt8Array::from_iter_values(vec![1]),
        )])
        .unwrap();

        let trace_types = [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::Spans,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
        ];

        for trace_type in trace_types.iter() {
            assert!(otap_batch.get(*trace_type).is_none());
            otap_batch.set(*trace_type, record_batch.clone());
            assert!(
                otap_batch.get(*trace_type).is_some(),
                "Failed for type: {:?}",
                trace_type
            );
        }
    }
}
