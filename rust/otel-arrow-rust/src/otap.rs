// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains various types and methods for interacting with and manipulating
//! OTAP data / record batches

use std::{
    iter::{once, repeat, repeat_n},
    num::NonZeroU64,
    ops::{Add, RangeFrom, RangeInclusive},
    sync::Arc,
};

use arrow::{
    array::{
        ArrowPrimitiveType, DictionaryArray, PrimitiveArray, RecordBatch, UInt16Array, UInt32Array,
    },
    datatypes::{
        ArrowNativeTypeOp, DataType, Schema, SchemaBuilder, UInt8Type, UInt16Type, UInt32Type,
    },
};
use snafu::ResultExt;

use transform::{
    materialize_parent_id_for_attributes, materialize_parent_id_for_exemplars,
    materialize_parent_ids_by_columns, remove_delta_encoding,
};

use crate::{
    decode::record_message::RecordMessage,
    error::{self, Result},
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::consts,
};

pub mod batching;
pub mod schema;
#[allow(missing_docs)]
pub mod transform;

/// The OtapBatch enum is used to represent a batch of OTAP data.
#[derive(Clone, Debug, PartialEq)]
#[allow(clippy::large_enum_variant)]
pub enum OtapArrowRecords {
    /// Represents a batch of logs data.
    Logs(Logs),
    /// Represents a batch of metrics data.
    Metrics(Metrics),
    /// Represents a batch of spans data.
    Traces(Traces),
}

#[derive(Clone, Debug, PartialEq, Eq)]
enum OtapArrowRecordTag {
    /// Logs data
    Logs,
    /// Metrics data
    Metrics,
    /// Spans data
    Traces,
}

impl OtapArrowRecords {
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

    /// Fetch the number of items as defined by the batching system
    #[must_use]
    pub fn batch_length(&self) -> usize {
        match self {
            Self::Logs(logs) => logs.batch_length(),
            Self::Metrics(metrics) => metrics.batch_length(),
            Self::Traces(traces) => traces.batch_length(),
        }
    }

    #[must_use]
    fn tag(&self) -> OtapArrowRecordTag {
        match self {
            Self::Logs(_) => OtapArrowRecordTag::Logs,
            Self::Metrics(_) => OtapArrowRecordTag::Metrics,
            Self::Traces(_) => OtapArrowRecordTag::Traces,
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

    /// The number of `RecordBatch`es needed for this kind of telemetry.
    const COUNT: usize;

    /// Mutable access to the batch array. The array the implementer returns
    /// should be the size of the number of types it supports, and it should expect
    /// that types to be positioned in the array according to the POSITION_LOOKUP array.
    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>];
    fn batches(&self) -> &[Option<RecordBatch>];
    fn into_batches(self) -> [Option<RecordBatch>; Metrics::COUNT];

    /// Return a list of the allowed payload types associated with this type of batch
    fn allowed_payload_types() -> &'static [ArrowPayloadType];

    /// Decode the delta-encoded and quasi-delta encoded IDs & parent IDs on each Arrow
    /// Arrow Record Batch contained in this Otap Batch. Internally, implementers should
    /// know which payloads use which types (u16 or u32) for ID/Parent ID fields as well
    /// as how the IDs are encoded.
    fn decode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()>;

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

    fn batch_length(&self) -> usize;
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
const UNUSED_INDEX: usize = 99;

/// Store of record batches for a batch of OTAP logs data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Logs {
    batches: [Option<RecordBatch>; Logs::COUNT],
}

impl OtapBatchStore for Logs {
    const TYPE_MASK: u64 = (1 << ArrowPayloadType::ResourceAttrs as u64)
        + (1 << ArrowPayloadType::ScopeAttrs as u64)
        + (1 << ArrowPayloadType::Logs as u64)
        + (1 << ArrowPayloadType::LogAttrs as u64);

    const COUNT: usize = 4;

    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>] {
        &mut self.batches
    }

    fn batches(&self) -> &[Option<RecordBatch>] {
        &self.batches
    }

    fn into_batches(self) -> [Option<RecordBatch>; Metrics::COUNT] {
        let mut iter = self.batches.into_iter();
        std::array::from_fn(|_| iter.next().unwrap_or_default())
    }

    fn allowed_payload_types() -> &'static [ArrowPayloadType] {
        &[
            ArrowPayloadType::Logs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::LogAttrs,
        ]
    }

    fn decode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()> {
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

    fn batch_length(&self) -> usize {
        batch_length(&self.batches)
    }
}

/// Store of record batches for a batch of OTAP metrics data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metrics {
    batches: [Option<RecordBatch>; Metrics::COUNT],
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
        + (1 << ArrowPayloadType::MultivariateMetrics as u64)
        + (1 << ArrowPayloadType::MetricAttrs as u64);

    const COUNT: usize = 19;

    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>] {
        &mut self.batches
    }

    fn batches(&self) -> &[Option<RecordBatch>] {
        &self.batches
    }

    fn into_batches(self) -> [Option<RecordBatch>; Metrics::COUNT] {
        self.batches
    }

    fn allowed_payload_types() -> &'static [ArrowPayloadType] {
        &[
            ArrowPayloadType::UnivariateMetrics,
            ArrowPayloadType::MultivariateMetrics,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
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
            ArrowPayloadType::MetricAttrs,
        ]
    }

    fn decode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()> {
        if let Some(metrics_rb) = otap_batch.get(ArrowPayloadType::UnivariateMetrics) {
            let rb = remove_delta_encoding::<UInt16Type>(metrics_rb, consts::ID)?;
            otap_batch.set(ArrowPayloadType::UnivariateMetrics, rb);
        }

        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::MetricAttrs,
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

    fn batch_length(&self) -> usize {
        batch_length(&self.batches)
    }
}

/// Store of record batches for a batch of OTAP traces data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Traces {
    batches: [Option<RecordBatch>; Traces::COUNT],
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

    const COUNT: usize = 8;

    fn batches_mut(&mut self) -> &mut [Option<RecordBatch>] {
        &mut self.batches
    }

    fn batches(&self) -> &[Option<RecordBatch>] {
        &self.batches
    }

    fn into_batches(self) -> [Option<RecordBatch>; Metrics::COUNT] {
        let mut iter = self.batches.into_iter();
        std::array::from_fn(|_| iter.next().unwrap_or_default())
    }

    fn allowed_payload_types() -> &'static [ArrowPayloadType] {
        &[
            ArrowPayloadType::Spans,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::SpanEvents,
            ArrowPayloadType::SpanLinks,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
        ]
    }

    fn decode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()> {
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

    fn batch_length(&self) -> usize {
        batch_length(&self.batches)
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
        ArrowPayloadType::NumberDpExemplars => &[ArrowPayloadType::NumberDpExemplarAttrs],
        ArrowPayloadType::SummaryDataPoints => &[ArrowPayloadType::SummaryDpAttrs],
        ArrowPayloadType::HistogramDataPoints => &[
            ArrowPayloadType::HistogramDpAttrs,
            ArrowPayloadType::HistogramDpExemplars,
        ],
        ArrowPayloadType::HistogramDpExemplars => &[ArrowPayloadType::HistogramDpExemplarAttrs],
        ArrowPayloadType::ExpHistogramDataPoints => &[
            ArrowPayloadType::ExpHistogramDpAttrs,
            ArrowPayloadType::ExpHistogramDpExemplars,
        ],

        ArrowPayloadType::ExpHistogramDpExemplars => {
            &[ArrowPayloadType::ExpHistogramDpExemplarAttrs]
        }
        ArrowPayloadType::MultivariateMetrics => {
            child_payload_types(ArrowPayloadType::UnivariateMetrics)
        }
        _ => &[],
    }
}

/// I'm a convenient wrapper for dealing with ID and PARENT_ID columns in generic code.
enum IDColumn<'rb> {
    U16(&'rb UInt16Array),
    U32(&'rb UInt32Array),
}

impl<'rb> IDColumn<'rb> {
    fn extract(input: &'rb RecordBatch, column_name: &'static str) -> Result<IDColumn<'rb>> {
        use snafu::OptionExt;
        let id = input
            .column_by_name(column_name)
            .context(error::ColumnNotFoundSnafu { name: column_name })?;

        match (
            id.as_any().downcast_ref::<UInt16Array>(),
            id.as_any().downcast_ref::<UInt32Array>(),
        ) {
            (Some(array), None) => Ok(Self::U16(array)),
            (None, Some(array)) => Ok(Self::U32(array)),
            (Some(_), Some(_)) => unreachable!(),
            (None, None) => {
                error::ColumnDataTypeMismatchSnafu {
                    name: column_name,
                    expect: DataType::UInt16, // Or UInt32, but we can only provide one
                    actual: id.data_type().clone(),
                }
                .fail()
            }
        }
    }
}

/// I describe the indices and values in an ID column for reindexing
struct IDRange<T> {
    indices: RangeInclusive<usize>,
    ids: RangeInclusive<T>,
    is_gap_free: bool,
}

/// I describe the indices and values in an ID column for reindexing; if the column is all nulls, my
/// interior values will be None.
struct MaybeIDRange<T>(Option<IDRange<T>>);

impl<T> MaybeIDRange<T> {
    /// Make a new `MaybeIDRange` from an array reference
    pub fn from_generic_array<Native, ArrowPrimitive>(
        array: &PrimitiveArray<ArrowPrimitive>,
    ) -> MaybeIDRange<Native>
    where
        Native: Add<Output = Native> + Copy + PartialEq + PartialOrd + ArrowNativeTypeOp,
        ArrowPrimitive: ArrowPrimitiveType<Native = Native>,
    {
        // Null-handling
        //
        // We're going to rely heavily on the fact that we sorted with nulls first. Why first and not last?
        // Because the more efficient PrimitiveArray.nulls().unwrap().valid_indices() iterator is not double ended
        // unlike PrimitiveArray.nulls().unwrap().iter() which is slower.

        let all_values = array.values();
        let len = array.len();
        use arrow::array::Array;

        let (non_null_slice, indices) = match array.nulls() {
            // There are no nulls at all, so everything is valid!
            None => (&all_values[..], Some(0..=len - 1)),

            // Everything is null, so nothing is valid
            Some(nulls) if nulls.null_count() == len => (&all_values[0..0], None),

            // There are some nulls but also some non-null values, so somethings are valid
            Some(nulls) => {
                // We rely on the fact that we've sorted so that nulls come at the front.

                // SAFETY: unwrap is safe here because we've already verified that the entire array
                // isn't null, which means there has to be at least one valid index which means
                // .next() will return Some the first time we call it.
                let first_valid_index = nulls
                    .valid_indices()
                    .next()
                    .expect("a non-null must be here");
                (
                    &all_values[first_valid_index..],
                    Some(first_valid_index..=len - 1),
                )
            }
        };
        let is_gap_free = non_null_slice
            .windows(2)
            .all(|pair| pair[0] == pair[1] || pair[1] == pair[0] + Native::ONE);

        MaybeIDRange(indices.map(|indices| {
            let ids = non_null_slice[0]..=non_null_slice[len - 1];
            IDRange {
                ids,
                indices,
                is_gap_free,
            }
        }))
    }

    fn reindex(
        rb: RecordBatch,
        column_name: &'static str,
        mut next_starting_id: u32,
    ) -> Result<(RecordBatch, u32)> {
        let id = IDColumn::extract(&rb, column_name)?;

        let maybe_new_ids = match id {
            IDColumn::U16(array) => Self::generic_reindex_column(array, next_starting_id)?,
            IDColumn::U32(array) => Self::generic_reindex_column(array, next_starting_id)?,
        };

        // Sigh. There doesn't seemt to be a way to mutate the values of a single column of a
        // `RecordBatch` without taking it apart entirely and then putting it back together.
        let (schema, mut columns, _len) = rb.into_parts();

        if let Some((new_id_array, new_next_starting_id)) = maybe_new_ids {
            let column_index = schema
                .fields
                .find(column_name)
                .expect("we already extracted this column")
                .0;
            columns[column_index] = new_id_array;
            next_starting_id = new_next_starting_id;
        }

        Ok((
            RecordBatch::try_new(schema, columns).context(error::BatchingSnafu)?,
            next_starting_id,
        ))
    }

    fn generic_reindex_column<Native, ArrowPrimitive>(
        array: &PrimitiveArray<ArrowPrimitive>,
        next_starting_id: u32,
    ) -> Result<Option<(Arc<dyn arrow::array::Array>, u32)>>
    where
        Native: Add<Output = Native>
            + Copy
            + PartialEq
            + PartialOrd
            + TryFrom<u32>
            + TryFrom<i64>
            + Into<i64>
            + Into<u32>
            + Clone
            + ArrowNativeTypeOp,
        RangeFrom<Native>: Iterator<Item = Native>,
        <Native as TryFrom<u32>>::Error: std::fmt::Debug,
        <Native as TryFrom<i64>>::Error: std::fmt::Debug,
        ArrowPrimitive: ArrowPrimitiveType<Native = Native>,
    {
        // FIxME: resort as needed!
        if let MaybeIDRange(Some(id_range)) = Self::from_generic_array(array) {
            // We do our bounds checking in `i64`-land because the only types we care about are `u32`
            // and `u16` and `i64` can repersent all those values and all offsets between them.
            let start: i64 = (*id_range.ids.start()).into();
            let end: i64 = (*id_range.ids.end()).into();
            let offset = (next_starting_id as i64) - start;
            let do_sub_offset = offset.signum() == -1;
            let offset = offset.abs();

            // If this statement works, then we know that all additions/subtractions will work
            // because we rely on the fact that the slice is sorted, so `start` is the smallest
            // possible value and `end` is the largest possible value.
            let _ = Native::try_from(if do_sub_offset {
                start - offset
            } else {
                end + offset
            })
            .expect("overflow occurred");

            let offset = Native::try_from(offset).expect("this should never happen");

            let array = if id_range.is_gap_free {
                // Whee! We can just do vectorized addition/subtraction!
                let scalar = PrimitiveArray::<ArrowPrimitive>::new_scalar(offset);

                // The normal add/sub kernels check for overflow which we don't want since we've already
                // verified that overflow can't happen, so we use the wrapping variants even though we
                // know no wrapping will occur to avoid the cost of overflow checks.
                let array = if do_sub_offset {
                    arrow::compute::kernels::numeric::sub_wrapping(array, &scalar)
                } else {
                    arrow::compute::kernels::numeric::add_wrapping(array, &scalar)
                };

                // FIXME: downcast_array will panic if the types aren't right; all it is doing is
                // PrimitiveArray<ArrowType>::from(input.data()); maybe try that with try_from instead?
                let array: PrimitiveArray<ArrowPrimitive> = arrow::array::downcast_array(
                    &array.expect("this array is of the expected type"),
                );
                array
            } else {
                // Ugh, there are gaps, so we need to do something slower and more complicated to
                // replace the sequence with a gap free version. This is complicated by the presence of
                // duplciates.
                use itertools::Itertools;

                let null_count = *(id_range.indices.start());
                let valid_ids = &array.values()[id_range.indices];
                let next_starting_id = Native::try_from(next_starting_id)
                    .expect("we can convert next_starting_id to our element type");

                let values = valid_ids
                    .iter()
                    // we convert the original values into a form of run-length encoding
                    .dedup_with_count()
                    // and combine it with a gap-free sequence of new integer values
                    .zip(next_starting_id..)
                    .flat_map(|((count, _old_id), new_id)| {
                        // swapping out old for new values, repeating items as needed
                        repeat_n(new_id, count)
                    });
                if null_count == 0 {
                    PrimitiveArray::from_iter_values(values)
                } else {
                    // First, we start with as many nulls as the original...
                    let nulls = repeat_n(None, null_count);
                    // ...then we add the non-null values
                    nulls.chain(values.map(Some)).collect()
                }
            };

            let last_id: u32 = array.values()[array.len() - 1].into();
            let next_starting_id = last_id.checked_add(1).expect("no overflow");
            Ok(Some((Arc::new(array), next_starting_id)))
        } else {
            Ok(None)
        }
    }
}

/// I describe ID column values for splitting
enum IDSeqs {
    RangeU16(Vec<RangeInclusive<u16>>),
    RangeU32(Vec<RangeInclusive<u32>>),
}

impl From<Vec<RangeInclusive<u16>>> for IDSeqs {
    fn from(r: Vec<RangeInclusive<u16>>) -> Self {
        Self::RangeU16(r)
    }
}

impl From<Vec<RangeInclusive<u32>>> for IDSeqs {
    fn from(r: Vec<RangeInclusive<u32>>) -> Self {
        Self::RangeU32(r)
    }
}

impl IDSeqs {
    fn from_col<'rb>(ids: IDColumn<'rb>, lengths: &[usize]) -> Self {
        match ids {
            IDColumn::U16(array) => Self::from_generic_array(array.values(), lengths),
            IDColumn::U32(array) => Self::from_generic_array(array.values(), lengths),
        }
    }

    fn from_generic_array<T>(slice: &[T], lengths: &[usize]) -> Self
    where
        T: Copy,
        Self: From<Vec<RangeInclusive<T>>>,
    {
        let mut ranges = Vec::with_capacity(lengths.len());
        let mut start = 0;
        for length in lengths {
            // subslice[0] and subslice[len-1] expressions at the end of this block rely on the fact
            // that subslice is not empty!
            assert!(*length > 0);

            let end = start + length;
            let subslice = &slice[start..end];

            let ids = subslice[0]..=subslice[subslice.len() - 1];
            ranges.push(ids);

            start = end;
        }

        ranges.into()
    }

    fn from_split_cols(inputs: &[RecordBatch]) -> Result<Self> {
        let column_name = consts::ID;
        let lengths = inputs
            .iter()
            .map(|input| input.num_rows())
            .collect::<Vec<_>>();
        let total_length = lengths.iter().sum();
        let first = IDColumn::extract(
            inputs.first().expect("there should be at least one input"),
            column_name,
        )?;
        Ok(match first {
            IDColumn::U16(_) => {
                let mut ids: Vec<u16> = Vec::with_capacity(total_length);
                for input in inputs {
                    if let IDColumn::U16(next_array) = IDColumn::extract(input, column_name)? {
                        ids.extend(next_array.values());
                    } else {
                        panic!();
                    }
                }
                Self::from_generic_array(&ids, &lengths)
            }
            IDColumn::U32(_) => {
                let mut ids: Vec<u32> = Vec::with_capacity(total_length);
                for input in inputs {
                    if let IDColumn::U32(next_array) = IDColumn::extract(input, column_name)? {
                        ids.extend(next_array.values());
                    } else {
                        panic!();
                    }
                }
                Self::from_generic_array(&ids, &lengths)
            }
        })
    }

    fn split_child_record_batch(&self, input: &RecordBatch) -> Result<Vec<RecordBatch>> {
        let id = IDColumn::extract(input, consts::PARENT_ID)?;
        Ok(match (self, id) {
            (IDSeqs::RangeU16(id_ranges), IDColumn::U16(parent_ids)) => {
                Self::generic_split_child_record_batch(id_ranges, parent_ids, input)
            }
            (IDSeqs::RangeU32(id_ranges), IDColumn::U32(parent_ids)) => {
                Self::generic_split_child_record_batch(id_ranges, parent_ids, input)
            }
            _ => {
                panic!();
            }
        })
    }

    fn generic_split_child_record_batch<T>(
        id_ranges: &[RangeInclusive<T::Native>],
        parent_ids: &PrimitiveArray<T>,
        input: &RecordBatch,
    ) -> Vec<RecordBatch>
    where
        T: ArrowPrimitiveType,
        T::Native: ArrowNativeTypeOp + From<u16> + Copy + PartialEq + Eq + Add<Output = T::Native>,
    {
        let one = T::Native::from(1u16);

        let slice = parent_ids.values();
        debug_assert!(slice.is_sorted());

        let mut result = Vec::with_capacity(id_ranges.len());
        for range in id_ranges {
            // We're using `partition_point` instead of `.binary_search` because it returns a
            // deterministic result in cases where there are multiple results found.

            // the first index where id >= start
            let first_index = slice.partition_point(|id| id < range.start());
            // the last index where id <= end
            let last_index = slice
                .partition_point(|&id| id < *range.end() + one)
                .checked_sub(1)
                .unwrap_or(first_index);
            result.push(
                if range.contains(&slice[first_index]) && range.contains(&slice[last_index]) {
                    input.slice(first_index, 1 + (last_index - first_index))
                } else {
                    // it is possible that this id range doesn't show up in this table at all!
                    RecordBatch::new_empty(input.schema())
                    // FIXME: this should be None and method should return Option
                },
            );
        }

        result
    }
}

/// Return a `RecordBatch` lexically sorted by either the `parent_id` column and secondarily by the
/// `id` column or just by the `id` column.
fn sort_recordbatch_by_ids(rb: RecordBatch) -> arrow::error::Result<RecordBatch> {
    let (schema, columns, _num_rows) = rb.into_parts();
    let id_column_index = schema.column_with_name(consts::ID).map(|pair| pair.0);
    let parent_id_column_index = schema
        .column_with_name(consts::PARENT_ID)
        .map(|pair| pair.0);

    use arrow::compute::{SortColumn, SortOptions, lexsort_to_indices, take};
    let options = Some(SortOptions {
        descending: false,
        nulls_first: true, // We rely on this heavily later on!
    });
    let sort_columns = match (parent_id_column_index, id_column_index) {
        (Some(parent_id), Some(id)) => {
            // FIXME: use row format for faster sorts right here
            let parent_id_values = columns[parent_id].clone();
            let id_values = columns[id].clone();
            vec![
                SortColumn {
                    values: parent_id_values,
                    options,
                },
                SortColumn {
                    values: id_values,
                    options,
                },
            ]
        }
        (None, Some(id)) => {
            let id_values = columns[id].clone();
            vec![SortColumn {
                values: id_values,
                options,
            }]
        }
        _ => {
            panic!("no ID column found")
        }
    };

    // FIXME: we don't need a Vec here...
    let indices = lexsort_to_indices(&sort_columns, None)?;
    let input_was_already_sorted = indices.values().is_sorted();
    let columns = if input_was_already_sorted {
        // Don't bother with take if the input was already sorted as we need.
        columns
    } else {
        columns
            .iter()
            .map(|c| take(c.as_ref(), &indices, None))
            .collect::<arrow::error::Result<Vec<_>>>()?
    };

    RecordBatch::try_new(schema, columns)
}

fn split_record_batch(
    first_batch_size: usize,
    rest_batch_size: usize,
    input: &RecordBatch,
) -> Result<(Vec<RecordBatch>, IDSeqs)> {
    let input_len = input.num_rows();
    let mut result =
        Vec::with_capacity(1 + (input_len - first_batch_size).div_ceil(rest_batch_size));
    let batch_sizes = once(first_batch_size).chain(repeat(rest_batch_size));
    let mut offset = 0;
    for batch_size in batch_sizes {
        let batch_size = batch_size.min(input_len - offset);
        result.push(input.slice(offset, batch_size));
        offset += batch_size;
        if offset == input_len {
            break;
        }
    }

    assert_eq!(
        input_len,
        result.iter().map(|rb| rb.num_rows()).sum::<usize>()
    );

    let lengths = result.iter().map(|rb| rb.num_rows()).collect::<Vec<_>>();
    let ids = IDSeqs::from_col(IDColumn::extract(input, consts::ID)?, &lengths);
    Ok((result, ids))
}

/// Fetch the number of items as defined by the batching system
#[must_use]
fn batch_length<const N: usize>(batches: &[Option<RecordBatch>; N]) -> usize {
    match N {
        Logs::COUNT => batches[POSITION_LOOKUP[ArrowPayloadType::Logs as usize]]
            .as_ref()
            .map(|batch| batch.num_rows())
            .unwrap_or(0),
        Metrics::COUNT => {
            const DATA_POINTS_TYPES: [ArrowPayloadType; 4] = [
                ArrowPayloadType::NumberDataPoints,
                ArrowPayloadType::SummaryDataPoints,
                ArrowPayloadType::HistogramDataPoints,
                ArrowPayloadType::ExpHistogramDataPoints,
            ];
            DATA_POINTS_TYPES
                .iter()
                .flat_map(|dpt| batches[POSITION_LOOKUP[*dpt as usize]].as_ref())
                .map(|batch| batch.num_rows())
                .sum()
        }
        Traces::COUNT => batches[POSITION_LOOKUP[ArrowPayloadType::Spans as usize]]
            .as_ref()
            .map(|batch| batch.num_rows())
            .unwrap_or(0),
        _ => {
            unreachable!()
        }
    }
}

/// I logically represent a sequence of OtapArrowRecords that all share exactly the same tag.
pub enum RecordsGroup {
    /// A sequence of batches representing log data
    Logs(Vec<[Option<RecordBatch>; Logs::COUNT]>),
    /// A sequence of batches representing metric data
    Metrics(Vec<[Option<RecordBatch>; Metrics::COUNT]>),
    /// A sequence of batches representing span data
    Traces(Vec<[Option<RecordBatch>; Traces::COUNT]>),
}

// FIXME!!!!! We need to ensure that by the time the batching machinery sees a batch, batches are
// non empty and at least have their primary type with nonzero length.

fn tag_count(records: &[OtapArrowRecords], tag: OtapArrowRecordTag) -> usize {
    records
        .iter()
        .map(|records| (records.tag() == tag) as usize)
        .sum()
}

impl RecordsGroup {
    /// Convert a sequence of `OtapArrowRecords` into three `RecordsGroup` objects
    #[must_use]
    pub fn split_by_type(records: Vec<OtapArrowRecords>) -> [Self; 3] {
        let log_count = tag_count(&records, OtapArrowRecordTag::Logs);
        let mut log_records = Vec::with_capacity(log_count);

        let metric_count = tag_count(&records, OtapArrowRecordTag::Metrics);
        let mut metric_records = Vec::with_capacity(metric_count);

        let trace_count = tag_count(&records, OtapArrowRecordTag::Traces);
        let mut trace_records = Vec::with_capacity(trace_count);

        for records in records {
            match records {
                OtapArrowRecords::Logs(logs) => {
                    log_records.push(shrink(logs.into_batches()));
                }
                OtapArrowRecords::Metrics(metrics) => {
                    metric_records.push(metrics.into_batches());
                }
                OtapArrowRecords::Traces(traces) => {
                    trace_records.push(shrink(traces.into_batches()));
                }
            }
        }

        [
            RecordsGroup::Logs(log_records),
            RecordsGroup::Metrics(metric_records),
            RecordsGroup::Traces(trace_records),
        ]
    }

    /// Split `RecordBatch`es as need when they're larger than our threshold or when we need them in
    /// smaller pieces to concatenate together into our target size.
    pub fn split(self, max_output_batch: NonZeroU64) -> Result<Self> {
        Ok(match self {
            RecordsGroup::Logs(items) => RecordsGroup::Logs(generic_split(
                items,
                max_output_batch,
                Logs::allowed_payload_types(),
                ArrowPayloadType::Logs,
            )?),
            RecordsGroup::Metrics(items) => RecordsGroup::Metrics(generic_split(
                items,
                max_output_batch,
                Metrics::allowed_payload_types(),
                ArrowPayloadType::UnivariateMetrics,
            )?),
            RecordsGroup::Traces(items) => RecordsGroup::Traces(generic_split(
                items,
                max_output_batch,
                Traces::allowed_payload_types(),
                ArrowPayloadType::Spans,
            )?),
        })
    }

    /// Merge `RecordBatch`es together so that they're no bigger than `max_output_batch`.
    pub fn concatenate(self, max_output_batch: Option<NonZeroU64>) -> Result<Self> {
        Ok(match self {
            RecordsGroup::Logs(items) => RecordsGroup::Logs(generic_concatenate(
                items,
                Logs::allowed_payload_types(),
                max_output_batch,
            )?),
            RecordsGroup::Metrics(items) => RecordsGroup::Metrics(generic_concatenate(
                items,
                Metrics::allowed_payload_types(),
                max_output_batch,
            )?),
            RecordsGroup::Traces(items) => RecordsGroup::Traces(generic_concatenate(
                items,
                Traces::allowed_payload_types(),
                max_output_batch,
            )?),
        })
    }

    // FIXME: replace this with an Extend impl to avoid unnecessary allocations
    /// Convert into a sequence of `OtapArrowRecords`
    #[must_use]
    pub fn into_otap_arrow_records(self) -> Vec<OtapArrowRecords> {
        match self {
            RecordsGroup::Logs(items) => items
                .into_iter()
                .map(|batches| OtapArrowRecords::Logs(Logs { batches }))
                .collect(),
            RecordsGroup::Metrics(items) => items
                .into_iter()
                .map(|batches| OtapArrowRecords::Metrics(Metrics { batches }))
                .collect(),
            RecordsGroup::Traces(items) => items
                .into_iter()
                .map(|batches| OtapArrowRecords::Traces(Traces { batches }))
                .collect(),
        }
    }
}

fn generic_concatenate<const N: usize>(
    batches: Vec<[Option<RecordBatch>; N]>,
    allowed_payloads: &[ArrowPayloadType],
    max_output_batch: Option<NonZeroU64>,
) -> Result<Vec<[Option<RecordBatch>; N]>> {
    let mut result = Vec::new();

    let mut current = Vec::new();
    let mut current_batch_length = 0;
    for batches in batches {
        let emit_new_batch = max_output_batch
            .map(|max_output_batch| {
                (current_batch_length + batch_length(&batches)) as u64 >= max_output_batch.get()
            })
            .unwrap_or(true);
        if emit_new_batch {
            reindex(&mut current, allowed_payloads)?;
            result.push(generic_schemaless_concatenate(&mut current)?);
            current_batch_length = 0;
            current.clear();
        } else {
            current_batch_length += batch_length(&batches);
            current.push(batches);
        }
    }

    Ok(result)
}

/// This is basically a transpose view thats lets us look at a sequence of the `i`-th table given a
/// sequence of `RecordBatch` arrays.
fn select<const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
    i: usize,
) -> impl Iterator<Item = &RecordBatch> {
    batches.iter().flat_map(move |batches| batches[i].as_ref())
}

fn reindex<const N: usize>(
    batches: &mut [[Option<RecordBatch>; N]],
    allowed_payloads: &[ArrowPayloadType],
) -> Result<()> {
    let mut starting_ids: [u32; N] = [0; N];
    for payload in allowed_payloads {
        let child_payloads = child_payload_types(*payload);
        if !child_payloads.is_empty() {
            for batches in batches.iter_mut() {
                let parent_offset = POSITION_LOOKUP[*payload as usize];
                let parent = batches[parent_offset].take();
                if let Some(parent) = parent {
                    let parent_starting_offset = starting_ids[parent_offset];
                    let (parent, next_starting_id) =
                        MaybeIDRange::<u16>::reindex(parent, consts::ID, parent_starting_offset)?;
                    starting_ids[parent_offset] += next_starting_id;
                    // return parent to batches since we took it!
                    let _ = batches[parent_offset].replace(parent);
                    for child in child_payloads {
                        let child_offset = POSITION_LOOKUP[*child as usize];
                        if let Some(child) = batches[child_offset].take() {
                            let (child, next_starting_id) = MaybeIDRange::<u16>::reindex(
                                child,
                                consts::PARENT_ID,
                                parent_starting_offset,
                            )?;
                            starting_ids[child_offset] += next_starting_id;
                            // return child to batches since we took it!
                            let _ = batches[child_offset].replace(child);
                            // We don't have to reindex child's id column since we'll get to it in a
                            // later iteration of the loop if it exists.
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

/// A generic helper for `unify`
fn flatten_dictionary_array<K: arrow::datatypes::ArrowDictionaryKeyType>(
    array: DictionaryArray<K>,
) -> arrow::error::Result<Arc<dyn arrow::array::Array>> {
    let (keys, values) = array.into_parts();
    let options = Some(arrow::compute::TakeOptions {
        check_bounds: false,
    });
    arrow::compute::take(&values, &keys, options)
}

// There are two problems we need to solve when concatenating `RecordBatch`es:
//
// * The `AdaptiveArrayBuilder` code in `encode` will generate columns with different types: either
//   a flat array, a Dict8 array or a Dict16 array. When any dictionary arrays are being used here,
//   we need to convert them to flat arrays.
//
// * Optional columns will be missing entirely; if they're missing from only some batches, we
//   need to add all null columns from the batches where they're not present.

fn unify<const N: usize>(batches: &mut [[Option<RecordBatch>; N]]) -> Result<()> {
    let mut schemas = Vec::with_capacity(batches.len());

    use std::collections::{HashMap, HashSet};
    // FIXME: perhaps this whole function should coalesce operations against the same
    // `RecordBatch`es together? At least investigate the performance cost of calling
    // RecordBatch::into_parts/try_new repeatedly.

    let mut cols_to_dict_fields: HashMap<usize, Vec<usize>> = HashMap::new();

    // `field_name_to_col_indices` maps column name to vector of col-indices
    let mut field_name_to_col_indices: HashMap<String, HashSet<usize>> = HashMap::new();
    // FIXME: replace sets with a real bitset type, ideally one that stores small sets inline
    // without allocations.

    let mut all_cols: HashSet<usize> = HashSet::new();

    // I need a dummy object to swap into column Vecs while I mutate and replace the column I
    // actually care about.
    let mut dummy = arrow::array::new_null_array(&DataType::UInt8, 0);

    for row in 0..N {
        schemas.clear(); // We're going to reuse this allocation accross loop iterations
        schemas.extend(select(batches, row).map(|batch| batch.schema()));
        if schemas.is_empty() {
            return Ok(());
        }
        let len = batches.len();

        cols_to_dict_fields.clear();
        field_name_to_col_indices.clear();
        all_cols.clear();

        for (col, schema) in schemas.iter().enumerate() {
            for (field_index, field) in schema.fields.iter().enumerate() {
                if matches!(field.data_type(), DataType::Dictionary(_, _)) {
                    // FIXME: we're assuming that field indices correspond to column indices...is
                    // that true?
                    cols_to_dict_fields
                        .entry(col)
                        .or_default()
                        .push(field_index);
                }
                let _ = field_name_to_col_indices
                    .entry(field.name().clone())
                    .or_default()
                    .insert(col);
            }
        }
        (0..len).for_each(|col| {
            let _ = all_cols.insert(col);
        });

        // Let's flatten out dictionary columns

        // FIXME: this is so bad; replace this with a smarter strategy that tries to consolidate
        // dict columns together preserving the space savings when possible!
        for (col, dict_field_indices) in cols_to_dict_fields.iter() {
            if let Some(batch) = batches[*col][row].take() {
                let (schema, mut columns, _num_rows) = batch.into_parts();
                let schema = Arc::into_inner(schema)
                    .expect("there should only be one outstanding reference");
                let mut builder = SchemaBuilder::from(&schema);

                for field_index in dict_field_indices {
                    // SAFETY: expect is fine because there's only one reference to the field outstanding.
                    let field = Arc::get_mut(builder.field_mut(*field_index))
                        .expect("there should only be one outstanding reference");
                    let (key, value) = match field.data_type() {
                        DataType::Dictionary(key, value) => (*key.clone(), *value.clone()),
                        _ => unreachable!(),
                    };
                    field.set_data_type(value.clone());

                    std::mem::swap(&mut columns[*field_index], &mut dummy);
                    // Now, our dummy value is stored in `columns` and `dummy` refers to the column we care about.
                    let mut column = match key {
                        DataType::UInt8 => flatten_dictionary_array(
                            DictionaryArray::<UInt8Type>::from(dummy.to_data()),
                        ),
                        DataType::UInt16 => flatten_dictionary_array(
                            DictionaryArray::<UInt16Type>::from(dummy.to_data()),
                        ),
                        _ => unreachable!(),
                    }
                    .context(error::BatchingSnafu)?;

                    std::mem::swap(&mut columns[*field_index], &mut column);
                    dummy = column;
                    assert_eq!(dummy.len(), 0);
                }

                let schema = Arc::new(builder.finish());
                let batch = RecordBatch::try_new(schema, columns).context(error::BatchingSnafu)?;

                let _ = batches[*col][row].replace(batch);
            }
        }

        // Let's find missing optional columns; note that this must happen after we deal with the
        // dict columns since we rely on the assumption that all fields with the same name will have
        // the same type.
        for (missing_field_name, present_cols) in field_name_to_col_indices
            .iter()
            .filter(|(_, cols)| cols.len() != len)
        {
            // All the present columns should have the same Field definition, so just pick the first
            // one arbitrarily; we know there has to be at least one because if there were none, we
            // wouldn't have a mismatch to begin with.
            let field = Arc::new(
                schemas[*present_cols
                    .iter()
                    .next()
                    .expect("there should be at least one schema")]
                .field_with_name(missing_field_name)
                .context(error::BatchingSnafu)?
                .clone(),
            );
            assert!(field.is_nullable());
            for missing_col in all_cols.difference(present_cols).copied() {
                if let Some(batch) = batches[missing_col][row].take() {
                    let (schema, mut columns, num_rows) = batch.into_parts();
                    let schema = Arc::into_inner(schema)
                        .expect("there should only be one outstanding reference");
                    let mut builder = SchemaBuilder::from(&schema);
                    builder.push(field.clone());
                    let schema = Arc::new(builder.finish());

                    columns.push(arrow::array::new_null_array(field.data_type(), num_rows));
                    let batch =
                        RecordBatch::try_new(schema, columns).context(error::BatchingSnafu)?;

                    let _ = batches[missing_col][row].replace(batch);
                }
            }
        }
    }
    Ok(())
}

fn generic_schemaless_concatenate<const N: usize>(
    batches: &mut [[Option<RecordBatch>; N]],
) -> Result<[Option<RecordBatch>; N]> {
    unify(batches)?;
    let mut result = [const { None }; N];
    for i in 0..N {
        // ignore all the rows where every item is None
        if select(batches, i).next().is_some() {
            let schema = Arc::new(
                Schema::try_merge(select(batches, i).map(|rb| Arc::unwrap_or_clone(rb.schema())))
                    .context(error::BatchingSnafu)?,
            );

            let num_rows = select(batches, i).map(RecordBatch::num_rows).sum();
            let mut batcher = arrow::compute::BatchCoalescer::new(schema.clone(), num_rows);
            for row in batches.iter_mut() {
                if let Some(rb) = row[i].take() {
                    batcher
                        .push_batch(
                            rb.with_schema(schema.clone())
                                .context(error::BatchingSnafu)?,
                        )
                        .context(error::BatchingSnafu)?;
                }
            }
            batcher
                .finish_buffered_batch()
                .context(error::BatchingSnafu)?;
            let concatenated = batcher
                .next_completed_batch()
                .expect("by construction this should never be empty");
            result[i] = Some(concatenated);
        }
    }

    for batches in batches {
        assert_eq!(batches, &[const { None }; N]);
    }
    Ok(result)
}

fn generic_split<const N: usize>(
    mut batches: Vec<[Option<RecordBatch>; N]>,
    max_output_batch: NonZeroU64,
    allowed_payloads: &[ArrowPayloadType],
    primary_payload: ArrowPayloadType,
) -> Result<Vec<[Option<RecordBatch>; N]>> {
    assert_eq!(N, allowed_payloads.len());
    assert!(allowed_payloads.contains(&primary_payload));
    assert!(!batches.is_empty());

    // First, ensure that all RecordBatches are sorted by parent_id & id so that we can efficiently
    // pluck ranges from them.
    for batches in batches.iter_mut() {
        #[allow(clippy::needless_range_loop)]
        for i in 0..N {
            if let Some(rb) = std::mem::take(&mut batches[i]) {
                let rb = sort_recordbatch_by_ids(rb).context(error::BatchingSnafu)?;
                batches[i] = Some(rb);
            }
        }
    }

    // Next, split the primary table
    let primary_offset = POSITION_LOOKUP[primary_payload as usize];
    let mut result = Vec::with_capacity(
        batches
            .iter()
            .map(batch_length)
            .map(|l| l as u64)
            .sum::<u64>()
            .div_ceil(max_output_batch.get()) as usize,
    );
    // SAFETY: on 32-bit archs, `as` conversion from u64 to usize can be wrong for values >=
    // u32::MAX, but we don't care about those cases because if they happen we'll only fail to avoid
    // a reallocation.

    let mut total_records_seen: u64 = 0; // think of this like iter::single(0).chain(batch_sizes.iter()).cumsum()
    for batches in batches.iter_mut() {
        let num_records = batch_length(batches) as u64;
        // SAFETY: % panics if the second arg is 0, but we're relying on NonZeroU64 to ensure
        // that can't happen.
        let prev_batch_size = total_records_seen % max_output_batch.get();
        let first_batch_size = max_output_batch.get() - prev_batch_size;

        if num_records > first_batch_size {
            // We're going to split `batches`!
            if let Some(rb) = batches[primary_offset].as_ref() {
                let (split_primary_parts, ids) = split_record_batch(
                    first_batch_size as usize,
                    max_output_batch.get() as usize,
                    rb,
                )?;

                // use ids to split the child tables: call split_child_record_batch
                let new_batch_count = split_primary_parts.len();
                result.extend(repeat_n([const { None }; N], new_batch_count));
                let result_len = result.len(); // only here to avoid immutable borrowing overlapping mutable borrowing
                // this is where we're going to be writing the rest of this split batch into!
                let new_batch = &mut result[result_len - new_batch_count..];

                // Store the newly split primary tables into this function's result
                for (i, split_primary) in split_primary_parts.into_iter().enumerate() {
                    new_batch[i][primary_offset] = Some(split_primary);
                }
                for payload in allowed_payloads.iter().copied() {
                    generic_split_helper(batches, payload, primary_payload, &ids, new_batch)?;
                }
            } else {
                panic!("expected to have primary for every group");
            }
        } else {
            // Don't bother splitting `batches` since it is smaller than our threshold....
            result.push(std::mem::replace(batches, [const { None }; N]));
        }

        // When we're done, the input should be an empty husk; if there's anything still left there,
        // that means we screwed up!
        assert_eq!(batches, &[const { None }; N]);
        total_records_seen += num_records;
    }

    Ok(result)
}

// This is a recursive helper function; the depth of recursion is bounded by parent-child
// relationships described in `child_payload_types` so we won't blow the stack.
fn generic_split_helper<const N: usize>(
    input: &[Option<RecordBatch>; N],
    payload: ArrowPayloadType,
    primary_payload: ArrowPayloadType,
    parent_ids: &IDSeqs,
    output: &mut [[Option<RecordBatch>; N]],
) -> Result<()> {
    // First, do the splitting, but only for non-primary payloads since those have to be done
    // specially.
    assert_ne!(payload, primary_payload);

    let payload_offset = POSITION_LOOKUP[payload as usize];
    if let Some(table) = input[payload_offset].as_ref() {
        // `table` is a parent table with some children!
        let child_payloads = child_payload_types(payload);

        // Let's split it...
        let split_table_parts = parent_ids.split_child_record_batch(table)?;
        assert_eq!(split_table_parts.len(), output.len());

        // ...and then recursively call ourself to do the same with our children
        if !child_payloads.is_empty()
            && child_payloads
                .iter()
                .any(|&payload| input[POSITION_LOOKUP[payload as usize]].is_some())
        {
            // We don't want to construct `id` unless there are children. Note that leaf nodes
            // won't have children and consequently won't have an `id` column so we shouldn't
            // construct `id` blindly!
            let id = IDSeqs::from_split_cols(&split_table_parts)?;

            for child_payload in child_payloads {
                generic_split_helper(input, *child_payload, primary_payload, &id, output)?;
            }
        }

        // ...and stash the result in `output`
        for (i, split_table) in split_table_parts.into_iter().enumerate() {
            output[i][payload_offset] = Some(split_table);
        }
    }

    Ok(())
}

/// In order to make `into_batches()` work, it has to return the maximally sized array of
/// `Option<RecordBatch>` padding out the end with `None`s. This function undoes that for
/// reintegrating the data into a generic context where we know exactly how big the array should be.
fn shrink<T, const BIGGER: usize, const SMALLER: usize>(
    array: [Option<T>; BIGGER],
) -> [Option<T>; SMALLER] {
    // Because the T we actually care about doesn't impl Copy, I think this is the simplest way to
    // verify that the tail is all None.
    for none in array[SMALLER..].iter() {
        assert!(none.is_none());
    }

    assert!(SMALLER < BIGGER);
    let mut iter = array.into_iter();
    // SAFETY: we've already verified that the iterator won't run out with the assert above.
    std::array::from_fn(|_| {
        iter.next()
            .expect("we will have the right number of elements")
    })
}

#[cfg(test)]
mod test {
    use arrow::{
        array::{
            FixedSizeBinaryArray, Int64Array, RecordBatch, StringArray, UInt8Array, UInt16Array,
            UInt32Array,
        },
        datatypes::Field,
    };

    use crate::otlp::attributes::store::AttributeValueType;

    use super::*;

    #[test]
    fn test_multivariate_metrics_child_types() {
        let multivariate_types = child_payload_types(ArrowPayloadType::MultivariateMetrics);
        let univariate_types = child_payload_types(ArrowPayloadType::UnivariateMetrics);
        assert_eq!(multivariate_types, univariate_types)
    }

    #[test]
    fn test_log_getset() {
        let mut otap_batch = OtapArrowRecords::Logs(Logs::new());

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
        let mut otap_batch = OtapArrowRecords::Metrics(Metrics::new());

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
                "Failed for type: {metric_type:?}"
            );
        }
    }

    #[test]
    fn test_traces_getset() {
        let mut otap_batch = OtapArrowRecords::Traces(Traces::new());

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
                "Failed for type: {trace_type:?}"
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

        let mut batch = OtapArrowRecords::Logs(Logs::default());
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

        let mut otap_batch = OtapArrowRecords::Metrics(Metrics::default());
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

        let mut otap_batch = OtapArrowRecords::Traces(Traces::default());
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
