// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains various types and methods for interacting with and manipulating
//! OTAP data / record batches

use arrow::{
    array::RecordBatch,
    datatypes::{UInt16Type, UInt32Type},
};
use transform::{
    materialize_parent_id_for_attributes, materialize_parent_id_for_exemplars,
    materialize_parent_ids_by_columns, remove_delta_encoding,
};

use crate::{
    decode::record_message::RecordMessage, error::Result,
    proto::opentelemetry::arrow::v1::ArrowPayloadType, schema::consts,
};

#[allow(missing_docs)]
pub mod transform;

/// The OtapBatch enum is used to represent a batch of OTAP data.
#[derive(Clone)]
#[allow(clippy::large_enum_variant)]
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

    /// Get the list of possible payload types associated with the batch of this type.
    /// Note: It's not guaranteed that this batch will actual contain all these
    /// payload types
    #[must_use]
    pub fn allowed_payload_types(&self) -> &'static [ArrowPayloadType] {
        match self {
            Self::Logs(_) => Logs::allowed_payload_types(),
            Self::Metrics(_) => Metrics::allowed_payload_types(),
            Self::Traces(_) => Traces::allowed_payload_types(),
        }
    }

    /// Decode the delta-encoded and quasi-delta encoded IDs & parent IDs
    /// on each Arrow Record Batch contained in this Otap Batch.
    pub fn decode_transport_optimized_ids(&mut self) -> Result<()> {
        match self {
            Self::Logs(_) => Logs::decode_transport_optimized_ids(self),
            Self::Metrics(_) => Metrics::decode_transport_optimized_ids(self),
            Self::Traces(_) => Traces::decode_transport_optimized_ids(self),
        }
    }
}

/// The ArrowBatchStore helper trait is used to define a common interface for
/// storing and retrieving Arrow record batches in a type-safe manner. It is
/// implemented by various structs that represent each signal type and provides
/// methods to efficiently set and get record batches.
trait OtapBatchStore: Sized + Default + Clone {
    // Internally, implementers should use a bitmask for the types they support.
    // The offsets in the bitmask should correspond to the ArrowPayloadType enum values.
    const TYPE_MASK: u64;

    /// Mutable access to the batch array. The array the implementer returns
    /// should be the size of the number of types it supports, and it should expect
    /// that types to be positioned in the array according to the POSITION_LOOKUP array.
    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>];
    fn batches(&self) -> &[Option<RecordBatch>];

    /// Return a list of the allowed payload types associated with this type of batch
    fn allowed_payload_types() -> &'static [ArrowPayloadType];

    /// Decode the delta-encoded and quasi-delta encoded IDs & parent IDs on each Arrow
    /// Arrow Record Batch contained in this Otap Batch. Internally, implementers should
    /// know which payloads use which types (u16 or u32) for ID/Parent ID fields as well
    /// as how the IDs are encoded.
    fn decode_transport_optimized_ids(otap_batch: &mut OtapBatch) -> Result<()>;

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
#[derive(Clone, Default)]
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

    fn allowed_payload_types() -> &'static [ArrowPayloadType] {
        &[
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::Logs,
            ArrowPayloadType::LogAttrs,
        ]
    }

    fn decode_transport_optimized_ids(otap_batch: &mut OtapBatch) -> Result<()> {
        if let Some(logs_rb) = otap_batch.get(ArrowPayloadType::Logs) {
            let rb = remove_delta_encoding::<UInt16Type>(logs_rb, consts::ID)?;
            otap_batch.set(ArrowPayloadType::Logs, rb);
        }

        for payload_type in [
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            if let Some(attrs_rb) = otap_batch.get(payload_type) {
                let rb = materialize_parent_id_for_attributes::<u16>(attrs_rb)?;
                otap_batch.set(payload_type, rb);
            }
        }

        Ok(())
    }
}

/// Store of record batches for a batch of OTAP metrics data.
#[derive(Clone, Default)]
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

    fn allowed_payload_types() -> &'static [ArrowPayloadType] {
        &[
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
            ArrowPayloadType::MultivariateMetrics,
        ]
    }

    fn decode_transport_optimized_ids(otap_batch: &mut OtapBatch) -> Result<()> {
        if let Some(metrics_rb) = otap_batch.get(ArrowPayloadType::UnivariateMetrics) {
            let rb = remove_delta_encoding::<UInt16Type>(metrics_rb, consts::ID)?;
            otap_batch.set(ArrowPayloadType::UnivariateMetrics, rb);
        }

        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            if let Some(attrs_rb) = otap_batch.get(payload_type) {
                let rb = materialize_parent_id_for_attributes::<u16>(attrs_rb)?;
                otap_batch.set(payload_type, rb);
            }
        }

        for payload_type in [
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
        ] {
            if let Some(rb) = otap_batch.get(payload_type) {
                let rb = remove_delta_encoding::<UInt32Type>(rb, consts::ID)?;
                let rb = remove_delta_encoding::<UInt16Type>(&rb, consts::PARENT_ID)?;
                otap_batch.set(payload_type, rb);
            }
        }

        for payload_type in [
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
        ] {
            if let Some(attrs_rb) = otap_batch.get(payload_type) {
                let rb = materialize_parent_id_for_attributes::<u32>(attrs_rb)?;
                otap_batch.set(payload_type, rb);
            }
        }

        for payload_type in [
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
        ] {
            if let Some(rb) = otap_batch.get(payload_type) {
                let rb = remove_delta_encoding::<UInt32Type>(rb, consts::ID)?;
                let rb = materialize_parent_id_for_exemplars::<u32>(&rb)?;
                otap_batch.set(payload_type, rb);
            }
        }

        Ok(())
    }
}

/// Store of record batches for a batch of OTAP traces data.
#[derive(Clone, Default)]
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

    fn allowed_payload_types() -> &'static [ArrowPayloadType] {
        &[
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::Spans,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
        ]
    }

    fn decode_transport_optimized_ids(otap_batch: &mut OtapBatch) -> Result<()> {
        if let Some(spans_rb) = otap_batch.get(ArrowPayloadType::Spans) {
            let rb = remove_delta_encoding::<UInt16Type>(spans_rb, consts::ID)?;
            otap_batch.set(ArrowPayloadType::Spans, rb);
        }

        for payload_type in [
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            if let Some(attrs_rb) = otap_batch.get(payload_type) {
                let rb = materialize_parent_id_for_attributes::<u16>(attrs_rb)?;
                otap_batch.set(payload_type, rb);
            }
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::SpanEvents) {
            let rb = remove_delta_encoding::<UInt32Type>(rb, consts::ID)?;
            let rb = materialize_parent_ids_by_columns::<u16>(&rb, [consts::NAME])?;
            otap_batch.set(ArrowPayloadType::SpanEvents, rb);
        }

        if let Some(rb) = otap_batch.get(ArrowPayloadType::SpanLinks) {
            let rb = remove_delta_encoding::<UInt32Type>(rb, consts::ID)?;
            let rb = materialize_parent_ids_by_columns::<u16>(&rb, [consts::TRACE_ID])?;
            otap_batch.set(ArrowPayloadType::SpanLinks, rb);
        }

        for payload_type in [
            ArrowPayloadType::SpanLinkAttrs,
            ArrowPayloadType::SpanEventAttrs,
        ] {
            if let Some(attrs_rb) = otap_batch.get(payload_type) {
                let rb = materialize_parent_id_for_attributes::<u32>(attrs_rb)?;
                otap_batch.set(payload_type, rb);
            }
        }

        Ok(())
    }
}

/// Return the child payload types for the given payload type
#[must_use]
pub fn child_payload_types(payload_type: ArrowPayloadType) -> &'static [ArrowPayloadType] {
    match payload_type {
        ArrowPayloadType::Logs => &[
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
        ],
        ArrowPayloadType::Spans => &[
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
        ],
        ArrowPayloadType::SpanEvents => &[ArrowPayloadType::SpanEventAttrs],
        ArrowPayloadType::SpanLinks => &[ArrowPayloadType::SpanLinkAttrs],
        ArrowPayloadType::UnivariateMetrics => &[
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
        ],
        ArrowPayloadType::NumberDataPoints => &[
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::NumberDpExemplars,
        ],
        ArrowPayloadType::NumberDpExemplarAttrs => &[ArrowPayloadType::NumberDpExemplarAttrs],
        ArrowPayloadType::SummaryDataPoints => &[ArrowPayloadType::SummaryDpAttrs],
        ArrowPayloadType::HistogramDataPoints => &[
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::HistogramDpExemplars,
        ],
        ArrowPayloadType::HistogramDpExemplars => &[ArrowPayloadType::HistogramDpExemplarAttrs],
        ArrowPayloadType::ExpHistogramDataPoints => &[
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
        ],

        ArrowPayloadType::ExpHistogramDpExemplarAttrs => {
            &[ArrowPayloadType::ExpHistogramDpExemplarAttrs]
        }
        ArrowPayloadType::MultivariateMetrics => {
            child_payload_types(ArrowPayloadType::MultivariateMetrics)
        }
        _ => &[],
    }
}

#[cfg(test)]
mod test {
    use arrow::array::{
        FixedSizeBinaryArray, Int64Array, RecordBatch, StringArray, UInt8Array, UInt16Array,
        UInt32Array,
    };
    use arrow::datatypes::{DataType, Field, Schema};
    use std::sync::Arc;

    use crate::otlp::attributes::store::AttributeValueType;

    use super::*;

    #[test]
    fn test_log_getset() {
        let mut otap_batch = OtapBatch::Logs(Logs::new());

        // for purpose of this test, the shape of the data doesn't really matter...
        let schema = Schema::new(vec![Field::new("a", DataType::UInt8, false)]);
        let record_batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(UInt8Array::from_iter_values(vec![1]))],
        )
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
        let record_batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(UInt8Array::from_iter_values(vec![1]))],
        )
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
        let record_batch = RecordBatch::try_new(
            Arc::new(schema),
            vec![Arc::new(UInt8Array::from_iter_values(vec![1]))],
        )
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

    #[test]
    fn test_log_decode_transport_optimized_ids() {
        let logs_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ID,
                DataType::UInt16,
                true,
            )])),
            vec![Arc::new(UInt16Array::from_iter(vec![
                Some(1),
                Some(1),
                None,
            ]))],
        )
        .unwrap();

        let attrs_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![1, 1, 1, 1])),
                Arc::new(UInt8Array::from_iter_values(
                    (0..4).map(|_| AttributeValueType::Str as u8),
                )),
                Arc::new(StringArray::from_iter_values((0..4).map(|_| "attr1"))),
                Arc::new(StringArray::from_iter_values(vec!["a", "a", "b", "b"])),
            ],
        )
        .unwrap();

        let mut batch = OtapBatch::Logs(Logs::default());
        batch.set(ArrowPayloadType::Logs, logs_rb);
        batch.set(ArrowPayloadType::LogAttrs, attrs_rb.clone());
        batch.set(ArrowPayloadType::ResourceAttrs, attrs_rb.clone());
        batch.set(ArrowPayloadType::ScopeAttrs, attrs_rb.clone());

        batch.decode_transport_optimized_ids().unwrap();

        // check log.ids
        let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
        let logs_ids = logs_rb
            .column_by_name(consts::ID)
            .unwrap()
            .as_any()
            .downcast_ref()
            .unwrap();
        let expected = UInt16Array::from_iter(vec![Some(1), Some(2), None]);
        assert_eq!(&expected, logs_ids);

        // check the attributes IDs
        for payload_type in [
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            let attrs_rb = batch.get(payload_type).unwrap();
            let attrs_parent_ids = attrs_rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt16Array::from_iter_values(vec![1, 2, 1, 2]);
            assert_eq!(&expected, attrs_parent_ids);
        }
    }

    #[test]
    fn test_metrics_decode_transport_optimized_ids() {
        let metrics_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ID,
                DataType::UInt16,
                true,
            )])),
            vec![Arc::new(UInt16Array::from_iter(vec![
                Some(1),
                Some(1),
                None,
            ]))],
        )
        .unwrap();

        let attrs_16_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![1, 1, 1, 1])),
                Arc::new(UInt8Array::from_iter_values(
                    (0..4).map(|_| AttributeValueType::Str as u8),
                )),
                Arc::new(StringArray::from_iter_values((0..4).map(|_| "attr1"))),
                Arc::new(StringArray::from_iter_values(vec!["a", "a", "b", "b"])),
            ],
        )
        .unwrap();

        let data_points_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true),
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter(vec![
                    Some(1),
                    Some(1),
                    None,
                    Some(1),
                ])),
                Arc::new(UInt16Array::from_iter_values(vec![1, 1, 1, 1])),
            ],
        )
        .unwrap();

        let attrs_32_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt32, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values(vec![1, 1, 1, 1])),
                Arc::new(UInt8Array::from_iter_values(
                    (0..4).map(|_| AttributeValueType::Str as u8),
                )),
                Arc::new(StringArray::from_iter_values((0..4).map(|_| "attr1"))),
                Arc::new(StringArray::from_iter_values(vec!["a", "a", "b", "b"])),
            ],
        )
        .unwrap();

        let exemplar_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true),
                Field::new(consts::PARENT_ID, DataType::UInt32, false),
                Field::new(consts::INT_VALUE, DataType::Int64, true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter(vec![
                    Some(1),
                    Some(1),
                    None,
                    Some(1),
                ])),
                Arc::new(UInt32Array::from_iter_values(vec![1, 1, 1, 1])),
                Arc::new(Int64Array::from_iter_values(vec![1, 1, 2, 2])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapBatch::Metrics(Metrics::default());
        otap_batch.set(ArrowPayloadType::UnivariateMetrics, metrics_rb);
        otap_batch.set(ArrowPayloadType::ResourceAttrs, attrs_16_rb.clone());
        otap_batch.set(ArrowPayloadType::ScopeAttrs, attrs_16_rb.clone());
        otap_batch.set(ArrowPayloadType::NumberDataPoints, data_points_rb.clone());
        otap_batch.set(ArrowPayloadType::NumberDpAttrs, attrs_32_rb.clone());
        otap_batch.set(ArrowPayloadType::NumberDpExemplars, exemplar_rb.clone());
        otap_batch.set(ArrowPayloadType::NumberDpExemplarAttrs, attrs_32_rb.clone());
        otap_batch.set(ArrowPayloadType::SummaryDataPoints, data_points_rb.clone());
        otap_batch.set(ArrowPayloadType::SummaryDpAttrs, attrs_32_rb.clone());
        otap_batch.set(
            ArrowPayloadType::HistogramDataPoints,
            data_points_rb.clone(),
        );
        otap_batch.set(ArrowPayloadType::HistogramDpAttrs, attrs_32_rb.clone());
        otap_batch.set(ArrowPayloadType::HistogramDpExemplars, exemplar_rb.clone());
        otap_batch.set(
            ArrowPayloadType::HistogramDpExemplarAttrs,
            attrs_32_rb.clone(),
        );
        otap_batch.set(
            ArrowPayloadType::ExpHistogramDataPoints,
            data_points_rb.clone(),
        );
        otap_batch.set(ArrowPayloadType::ExpHistogramDpAttrs, attrs_32_rb.clone());
        otap_batch.set(
            ArrowPayloadType::ExpHistogramDpExemplars,
            exemplar_rb.clone(),
        );
        otap_batch.set(
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
            attrs_32_rb.clone(),
        );

        otap_batch.decode_transport_optimized_ids().unwrap();

        let metrics_rb = otap_batch.get(ArrowPayloadType::UnivariateMetrics).unwrap();
        let span_ids = metrics_rb
            .column_by_name(consts::ID)
            .unwrap()
            .as_any()
            .downcast_ref()
            .unwrap();
        let expected = UInt16Array::from_iter(vec![Some(1), Some(2), None]);
        assert_eq!(&expected, span_ids);

        // check the attributes parent IDs
        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            let attrs_rb = otap_batch.get(payload_type).unwrap();
            let attrs_parent_ids = attrs_rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt16Array::from_iter_values(vec![1, 2, 1, 2]);
            assert_eq!(&expected, attrs_parent_ids);
        }

        for payload_type in [
            ArrowPayloadType::NumberDataPoints,
            ArrowPayloadType::SummaryDataPoints,
            ArrowPayloadType::HistogramDataPoints,
            ArrowPayloadType::ExpHistogramDataPoints,
        ] {
            let data_points_rb = otap_batch.get(payload_type).unwrap();
            let data_points_parent_ids = data_points_rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt16Array::from_iter_values(vec![1, 2, 3, 4]);
            assert_eq!(&expected, data_points_parent_ids);

            // check the data points IDs
            let data_points_ids = data_points_rb
                .column_by_name(consts::ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt32Array::from_iter(vec![Some(1), Some(2), None, Some(3)]);
            assert_eq!(&expected, data_points_ids);
        }

        // check data point attributes
        for payload_type in [
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::NumberDpAttrs,
            ArrowPayloadType::SummaryDpAttrs,
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::HistogramDpExemplarAttrs,
            ArrowPayloadType::NumberDpExemplarAttrs,
            ArrowPayloadType::ExpHistogramDpExemplarAttrs,
        ] {
            let attrs_rb = otap_batch.get(payload_type).unwrap();
            let attrs_parent_ids = attrs_rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt32Array::from_iter_values(vec![1, 2, 1, 2]);
            assert_eq!(&expected, attrs_parent_ids);
        }

        for payload_type in [
            ArrowPayloadType::NumberDpExemplars,
            ArrowPayloadType::HistogramDpExemplars,
            ArrowPayloadType::ExpHistogramDpExemplars,
        ] {
            let exemplar_rb = otap_batch.get(payload_type).unwrap();
            let exemplar_parent_ids = exemplar_rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt32Array::from_iter_values(vec![1, 2, 1, 2]);
            assert_eq!(&expected, exemplar_parent_ids);

            // check the exemplar IDs
            let exemplar_ids = exemplar_rb
                .column_by_name(consts::ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt32Array::from_iter(vec![Some(1), Some(2), None, Some(3)]);
            assert_eq!(&expected, exemplar_ids);
        }
    }

    #[test]
    fn test_trace_decode_transport_optimized_ids() {
        let spans_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![Field::new(
                consts::ID,
                DataType::UInt16,
                true,
            )])),
            vec![Arc::new(UInt16Array::from_iter(vec![
                Some(1),
                Some(1),
                None,
            ]))],
        )
        .unwrap();

        let attrs_16_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![1, 1, 1, 1])),
                Arc::new(UInt8Array::from_iter_values(
                    (0..4).map(|_| AttributeValueType::Str as u8),
                )),
                Arc::new(StringArray::from_iter_values((0..4).map(|_| "attr1"))),
                Arc::new(StringArray::from_iter_values(vec!["a", "a", "b", "b"])),
            ],
        )
        .unwrap();

        let events_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true),
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::NAME, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter(vec![
                    Some(1),
                    Some(1),
                    None,
                    Some(1),
                ])),
                Arc::new(UInt16Array::from_iter_values(vec![1, 1, 1, 1])),
                Arc::new(StringArray::from_iter_values(vec!["a", "a", "b", "b"])),
            ],
        )
        .unwrap();

        let links_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true),
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter(vec![
                    Some(1),
                    Some(1),
                    None,
                    Some(1),
                ])),
                Arc::new(UInt16Array::from_iter_values(vec![1, 1, 1, 1])),
                Arc::new(
                    FixedSizeBinaryArray::try_from_iter(
                        vec![
                            (0u8..16u8).collect::<Vec<u8>>(),
                            (0u8..16u8).collect::<Vec<u8>>(),
                            (16u8..32u8).collect::<Vec<u8>>(),
                            (16u8..32u8).collect::<Vec<u8>>(),
                        ]
                        .into_iter(),
                    )
                    .unwrap(),
                ),
            ],
        )
        .unwrap();

        let attrs_32_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt32, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values(vec![1, 1, 1, 1])),
                Arc::new(UInt8Array::from_iter_values(
                    (0..4).map(|_| AttributeValueType::Str as u8),
                )),
                Arc::new(StringArray::from_iter_values((0..4).map(|_| "attr1"))),
                Arc::new(StringArray::from_iter_values(vec!["a", "a", "b", "b"])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapBatch::Traces(Traces::default());
        otap_batch.set(ArrowPayloadType::Spans, spans_rb);
        otap_batch.set(ArrowPayloadType::SpanAttrs, attrs_16_rb.clone());
        otap_batch.set(ArrowPayloadType::ResourceAttrs, attrs_16_rb.clone());
        otap_batch.set(ArrowPayloadType::ScopeAttrs, attrs_16_rb.clone());
        otap_batch.set(ArrowPayloadType::SpanEvents, events_rb);
        otap_batch.set(ArrowPayloadType::SpanLinks, links_rb);
        otap_batch.set(ArrowPayloadType::SpanEventAttrs, attrs_32_rb.clone());
        otap_batch.set(ArrowPayloadType::SpanLinkAttrs, attrs_32_rb.clone());

        otap_batch.decode_transport_optimized_ids().unwrap();

        // check log.ids
        let spans_rb = otap_batch.get(ArrowPayloadType::Spans).unwrap();
        let span_ids = spans_rb
            .column_by_name(consts::ID)
            .unwrap()
            .as_any()
            .downcast_ref()
            .unwrap();
        let expected = UInt16Array::from_iter(vec![Some(1), Some(2), None]);
        assert_eq!(&expected, span_ids);

        // check the attributes parent IDs
        for payload_type in [
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            let attrs_rb = otap_batch.get(payload_type).unwrap();
            let attrs_parent_ids = attrs_rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt16Array::from_iter_values(vec![1, 2, 1, 2]);
            assert_eq!(&expected, attrs_parent_ids);
        }

        // check links & events
        for payload_type in [ArrowPayloadType::SpanEvents, ArrowPayloadType::SpanLinks] {
            let rb = otap_batch.get(payload_type).unwrap();
            let ids = rb
                .column_by_name(consts::ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected_ids = UInt32Array::from_iter(vec![Some(1), Some(2), None, Some(3)]);
            assert_eq!(&expected_ids, ids);

            let parent_ids = rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected_parent_ids = UInt16Array::from_iter_values(vec![1, 2, 1, 2]);
            assert_eq!(&expected_parent_ids, parent_ids);
        }
        // check event & link attrs parent ids
        for payload_type in [
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
        ] {
            let attrs_rb = otap_batch.get(payload_type).unwrap();
            let attrs_parent_ids = attrs_rb
                .column_by_name(consts::PARENT_ID)
                .unwrap()
                .as_any()
                .downcast_ref()
                .unwrap();
            let expected = UInt32Array::from_iter_values(vec![1, 2, 1, 2]);
            assert_eq!(&expected, attrs_parent_ids);
        }
    }
}
