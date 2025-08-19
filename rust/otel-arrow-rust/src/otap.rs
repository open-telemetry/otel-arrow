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
    decode::record_message::RecordMessage,
    encode::column_encoding::{
        RESOURCE_ID_COL_PATH, SCOPE_ID_COL_PATH, apply_column_encodings, remap_parent_ids,
    },
    error::Result,
    otap,
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::consts,
};

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

    /// TODO
    pub fn encode_transport_optimized_ids(&mut self) -> Result<()> {
        match self {
            Self::Logs(_) => Logs::encode_transport_optimized_ids(self),
            Self::Metrics(_) => Metrics::encode_transport_optimized_ids(self),
            Self::Traces(_) => Traces::encode_transport_optimized_ids(self),
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
    fn decode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()>;

    /// TODO comments
    fn encode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()>;

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

    fn encode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()> {
        // apply encoding to root logs record
        if let Some(rb) = otap_batch.get(ArrowPayloadType::Logs) {
            let (rb, id_remappings) = apply_column_encodings(&ArrowPayloadType::Logs, rb)?;
            otap_batch.set(ArrowPayloadType::Logs, rb);

            // remap any of the child IDs if necessary ...
            if let Some(id_remappings) = id_remappings {
                for id_remapping in id_remappings {
                    let child_payload_type = match id_remapping.column_path {
                        RESOURCE_ID_COL_PATH => ArrowPayloadType::ResourceAttrs,
                        SCOPE_ID_COL_PATH => ArrowPayloadType::ScopeAttrs,
                        consts::ID => ArrowPayloadType::LogAttrs,

                        // TODO this would be an unusual situation? Something clearly wrong ..
                        _ => continue,
                    };
                    if let Some(child_rb) = otap_batch.get(child_payload_type) {
                        let rb = remap_parent_ids(
                            &child_payload_type,
                            child_rb,
                            &id_remapping.remapped_ids,
                        )?;
                        otap_batch.set(child_payload_type, rb);
                    }
                }
            }
        }

        // apply any encodings to the child attributes
        for payload_type in [
            ArrowPayloadType::LogAttrs,
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
        ] {
            if let Some(rb) = otap_batch.get(payload_type) {
                let (rb, _) = apply_column_encodings(&payload_type, rb)?;
                otap_batch.set(payload_type, rb);
            }
        }

        Ok(())
    }
}

/// Store of record batches for a batch of OTAP metrics data.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Metrics {
    batches: [Option<RecordBatch>; 19],
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

    fn encode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()> {
        todo!()
    }
}

/// Store of record batches for a batch of OTAP traces data.
#[derive(Clone, Debug, Default, PartialEq)]
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

    fn encode_transport_optimized_ids(otap_batch: &mut OtapArrowRecords) -> Result<()> {
        if let Some(rb) = otap_batch.get(ArrowPayloadType::Spans) {
            let (rb, id_remappings) = apply_column_encodings(&ArrowPayloadType::Spans, rb)?;
            otap_batch.set(ArrowPayloadType::Spans, rb);

            // remap any of the child IDs if necessary ...
            if let Some(id_remappings) = id_remappings {
                for id_remapping in id_remappings {
                    let child_payload_types = match id_remapping.column_path {
                        RESOURCE_ID_COL_PATH => [ArrowPayloadType::ResourceAttrs].as_slice(),
                        SCOPE_ID_COL_PATH => [ArrowPayloadType::ScopeAttrs].as_slice(),
                        consts::ID => [
                            ArrowPayloadType::SpanAttrs,
                            ArrowPayloadType::SpanEvents,
                            ArrowPayloadType::SpanLinks,
                        ]
                        .as_slice(),

                        // TODO this would be an unusual situation? Something clearly wrong ..
                        _ => continue,
                    };
                    for child_payload_type in child_payload_types.into_iter() {
                        if let Some(child_rb) = otap_batch.get(*child_payload_type) {
                            let rb = remap_parent_ids(
                                &child_payload_type,
                                child_rb,
                                &id_remapping.remapped_ids,
                            )?;
                            otap_batch.set(*child_payload_type, rb);
                        }
                    }
                }
            }
        }


        for (payload_type, child_payload_type) in [
            (ArrowPayloadType::SpanEvents, ArrowPayloadType::SpanEventAttrs),
            (ArrowPayloadType::SpanLinks, ArrowPayloadType::SpanLinkAttrs)
        ] {
            if let Some(rb) = otap_batch.get(payload_type) {
                let (rb, id_remappings) = apply_column_encodings(&payload_type, rb)?;
                otap_batch.set(payload_type, rb);

                if let Some(id_remappings) = id_remappings {
                    for id_remapping in id_remappings {
                        if id_remapping.column_path == consts::ID {
                            if let Some(child_rb) = otap_batch.get(child_payload_type) {
                                let rb = remap_parent_ids(
                                    &child_payload_type,
                                    child_rb,
                                    &id_remapping.remapped_ids,
                                )?;
                                otap_batch.set(child_payload_type, rb);
                            }
                        }
                    }
                }
            }
        }

        for payload_type in [
            ArrowPayloadType::ResourceAttrs,
            ArrowPayloadType::ScopeAttrs,
            ArrowPayloadType::SpanAttrs,
            ArrowPayloadType::SpanEventAttrs,
            ArrowPayloadType::SpanLinkAttrs,
        ] {
            if let Some(rb) = otap_batch.get(payload_type) {
                let (rb, _) = apply_column_encodings(&payload_type, rb)?;
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

#[cfg(test)]
mod test {
    use arrow::array::{
        ArrowPrimitiveType, FixedSizeBinaryArray, Int64Array, PrimitiveArray, RecordBatch,
        StringArray, StructArray, UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::datatypes::{DataType, Field, Fields, Schema};
    use std::sync::Arc;

    use crate::otlp::attributes::store::AttributeValueType;
    use crate::schema::FieldExt;

    use super::*;

    /// helper function for easily constructing a record batch of attributes. In this particular
    /// generated batch, all the types will be string with the same key. The argument is an array
    /// of `(parent_id, attr value)` tuples
    fn attrs_from_data<T: ArrowPrimitiveType>(
        data: Vec<(T::Native, &'static str)>,
        encoding: &'static str,
    ) -> RecordBatch {
        let attrs_schema = Arc::new(Schema::new(vec![
            Field::new(consts::PARENT_ID, T::DATA_TYPE, false).with_encoding(encoding.into()),
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        ]));
        let parent_ids = PrimitiveArray::<T>::from_iter_values(data.iter().map(|d| d.0));
        let attr_vals = StringArray::from_iter_values(data.iter().map(|d| d.1));
        let attr_keys = StringArray::from_iter_values(std::iter::repeat_n("a", data.len()));
        let attr_types = UInt8Array::from_iter_values(std::iter::repeat_n(
            AttributeValueType::Str as u8,
            data.len(),
        ));
        RecordBatch::try_new(
            attrs_schema,
            vec![
                Arc::new(parent_ids),
                Arc::new(attr_types),
                Arc::new(attr_keys),
                Arc::new(attr_vals),
            ],
        )
        .unwrap()
    }

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

        let batch_b4 = batch.clone();

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

        // TODO test roundtrip?
        batch.encode_transport_optimized_ids().unwrap();
        println!("{:?}", batch);

        assert_eq!(batch, batch_b4);
    }

    #[test]
    fn test_logs_encode_transport_optimized_ids() {
        let struct_fields: Fields =
            vec![Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding()].into();

        let logs_schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
            Field::new(
                consts::RESOURCE,
                DataType::Struct(struct_fields.clone()),
                true,
            ),
            Field::new(consts::SCOPE, DataType::Struct(struct_fields.clone()), true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        ]));

        // This data is meant to represent a batch of logs that will be sorted, and will need to
        // have the ID column remapped because it is no longer in an ascending order and so cannot
        // be delta encoded into an unsigned int type.
        //
        // To help make sense of the expected results below, note that it will be sorted by
        // (resource.id, scope.id, trace_id).
        // So 'logs_data', below, is effectively just a shuffled version of the following:
        // (0, 0, 0, 1),
        // (0, 0, 1, 2),
        // (0, 2, 2, 3),
        // (0, 2, 3, 7),
        // (1, 1, 4, 5),
        // (1, 1, 5, 6),
        // (1, 3, 6, 4),
        // (1, 3, 7, 0),
        //
        // We can then deduce how the IDs need to be remapped so they're increasing so as not to
        // have any negative deltas:
        //
        //  data:          log_id:    scope_id:
        // --------------|----------|----------
        // (0, 0, 0, 0),    1 -> 0    0 = same
        // (0, 0, 1, 1),    2 -> 1
        // (0, 1, 2, 2),    3 -> 2    2 -> 1
        // (0, 1, 3, 3),    7 -> 3
        // (1, 2, 4, 4),    5 -> 6    1 -> 2
        // (1, 2, 5, 5),    6 -> 5
        // (1, 3, 6, 6),    4 -> 6    3 = same
        // (1, 3, 7, 7),    0 -> 7
        //
        // Also note that there's no remapping of the resource IDs because it is the primary column
        // we sort by, so it will always be increasing after sorting.
        //
        let logs_data = vec![
            (1, 3, 6, 4),
            (0, 2, 2, 3),
            (0, 0, 1, 2),
            (0, 2, 3, 7),
            (1, 1, 5, 6),
            (1, 1, 4, 5),
            (0, 0, 0, 1),
            (1, 3, 7, 0),
        ];
        let resource_ids = UInt16Array::from_iter_values(logs_data.iter().map(|d| d.0));
        let scope_ids = UInt16Array::from_iter_values(logs_data.iter().map(|d| d.1));
        let trace_ids =
            FixedSizeBinaryArray::try_from_iter(logs_data.iter().map(|d| u128::to_be_bytes(d.2)))
                .unwrap();
        let log_ids = UInt16Array::from_iter_values(logs_data.iter().map(|d| d.3));

        let logs_rb = RecordBatch::try_new(
            logs_schema.clone(),
            vec![
                Arc::new(log_ids),
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(resource_ids)],
                    None,
                )),
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(scope_ids)],
                    None,
                )),
                Arc::new(trace_ids),
            ],
        )
        .unwrap();

        let log_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (1, "a"),
                (2, "a"),
                (1, "c"),
                (2, "c"),
                (5, "c"),
                (7, "c"),
                (2, "b"),
                (4, "b"),
            ],
            consts::metadata::encodings::PLAIN,
        );

        let scope_attrs = attrs_from_data::<UInt16Type>(
            vec![
                (1, "a"),
                (1, "b"),
                (2, "a"),
                (2, "c"),
                (0, "a"),
                (3, "a"),
                (3, "b"),
                (1, "b"),
            ],
            consts::metadata::encodings::PLAIN,
        );

        let resource_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (1, "a"),
                (0, "a"),
                (1, "c"),
                (0, "c"),
                (1, "b"),
                (1, "c"),
                (0, "b"),
            ],
            consts::metadata::encodings::PLAIN,
        );

        let mut batch = OtapArrowRecords::Logs(Logs::default());
        batch.set(ArrowPayloadType::Logs, logs_rb);
        batch.set(ArrowPayloadType::LogAttrs, log_attrs_rb);
        batch.set(ArrowPayloadType::ScopeAttrs, scope_attrs);
        batch.set(ArrowPayloadType::ResourceAttrs, resource_attrs_rb);

        batch.encode_transport_optimized_ids().unwrap();

        let expected_struct_fields: Fields = vec![
            Field::new(consts::ID, DataType::UInt16, false)
                .with_encoding(consts::metadata::encodings::DELTA),
        ]
        .into();
        let expected_logs_schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, true)
                .with_encoding(consts::metadata::encodings::DELTA),
            Field::new(
                consts::RESOURCE,
                DataType::Struct(expected_struct_fields.clone()),
                true,
            ),
            Field::new(
                consts::SCOPE,
                DataType::Struct(expected_struct_fields.clone()),
                true,
            ),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        ]));

        // TODO maybe comment about how this was computed
        let expected_logs_data = vec![
            (0, 0, 0, 0),
            (0, 0, 1, 1),
            (0, 1, 2, 1),
            (0, 0, 3, 1),
            (1, 1, 4, 1),
            (0, 0, 5, 1),
            (0, 1, 6, 1),
            (0, 0, 7, 1),
        ];

        let resource_ids = UInt16Array::from_iter_values(expected_logs_data.iter().map(|d| d.0));
        let scope_ids = UInt16Array::from_iter_values(expected_logs_data.iter().map(|d| d.1));
        let trace_ids = FixedSizeBinaryArray::try_from_iter(
            expected_logs_data.iter().map(|d| u128::to_be_bytes(d.2)),
        )
        .unwrap();
        let log_ids = UInt16Array::from_iter_values(expected_logs_data.iter().map(|d| d.3));

        let expected_logs_rb = RecordBatch::try_new(
            expected_logs_schema,
            vec![
                Arc::new(log_ids),
                Arc::new(StructArray::new(
                    expected_struct_fields.clone(),
                    vec![Arc::new(resource_ids)],
                    None,
                )),
                Arc::new(StructArray::new(
                    expected_struct_fields.clone(),
                    vec![Arc::new(scope_ids)],
                    None,
                )),
                Arc::new(trace_ids),
            ],
        )
        .unwrap();

        let result_logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(result_logs_rb, &expected_logs_rb);

        // expected logs_attr is calculated like - Note after remapping IDs, the attribute record
        // batches get sorted by:
        // (type, key, value, parent_id)
        //
        // original parent_id --> remapped parent_id, val --> quasi-delta parent ID
        // 1 --> 0, "a" --> 0
        // 2 --> 1, "a" --> 1
        // 2 --> 1, "b" --> 1
        // 4 --> 6, "b" --> 5
        // 1 --> 0, "c" --> 0
        // 2 --> 1, "c" --> 1
        // 7 --> 3, "c" --> 2
        // 5 --> 4, "c" --> 1

        let expected_log_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (0, "a"),
                (1, "a"),
                (1, "b"),
                (5, "b"),
                (0, "c"),
                (1, "c"),
                (2, "c"),
                (1, "c"),
            ],
            consts::metadata::encodings::QUASI_DELTA,
        );
        let result_log_attrs_rb = batch.get(ArrowPayloadType::LogAttrs).unwrap();
        assert_eq!(result_log_attrs_rb, &expected_log_attrs_rb);

        // expected scope attrs
        //
        // original parent_id --> remapped parent_id, val --> quasi-delta parent ID
        // 0 -> 0, "a" -> 0
        // 2 -> 1, "a" -> 1
        // 1 -> 2, "a" -> 1
        // 3 -> 3, "a" -> 1
        // 1 -> 2, "b" -> 2
        // 1 -> 2, "b" -> 0
        // 3 -> 3, "b" -> 1
        // 2 -> 1, "c" -> 1
        let expected_scope_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (0, "a"),
                (1, "a"),
                (1, "a"),
                (1, "a"),
                (2, "b"),
                (0, "b"),
                (1, "b"),
                (1, "c"),
            ],
            consts::metadata::encodings::QUASI_DELTA,
        );
        let result_scope_attrs_rb = batch.get(ArrowPayloadType::ScopeAttrs).unwrap();
        assert_eq!(result_scope_attrs_rb, &expected_scope_attrs_rb);

        // expected resource attrs
        //
        // note that because the record batch gets sorted by resource ID as the primary sort column
        // that there are no remapped IDs, so this is just the original resource attribute batch
        // sorted
        //
        // parent_id, val -> quasi-delta parent ID
        // 0, a -> 0
        // 1, a -> 1
        // 0, b -> 0
        // 1, b -> 1
        // 0, c -> 0
        // 1, c -> 1
        // 1, c -> 0
        //
        let expected_resource_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (0, "a"),
                (1, "a"),
                (0, "b"),
                (1, "b"),
                (0, "c"),
                (1, "c"),
                (0, "c"),
            ],
            consts::metadata::encodings::QUASI_DELTA,
        );
        let result_resource_attrs_rb = batch.get(ArrowPayloadType::ResourceAttrs).unwrap();
        assert_eq!(result_resource_attrs_rb, &expected_resource_attrs_rb);
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

    #[test]
    fn test_trace_encode_transport_optimized_ids() {
        let struct_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, false).with_plain_encoding(),
        ]);

        let spans_schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
            Field::new(
                consts::RESOURCE,
                DataType::Struct(struct_fields.clone()),
                true,
            ),
            Field::new(consts::SCOPE, DataType::Struct(struct_fields.clone()), true),
            Field::new(consts::NAME, DataType::Utf8, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        ]));

        // This data is meant to represent a batch of spans that will be sorted, and will need to
        // have the ID column remapped because it is no longer in an ascending order and so cannot
        // be delta encoded into an unsigned int type.
        //
        // To help make sense of the expected results below, note that it will be sorted by
        // (resource.id, scope.id, name, trace_id).
        // So 'spans_data', below, is effectively just a shuffled version of the following:
        // (0, 0, "a", 0, 9),
        // (0, 0, "a", 1, 5),
        // (0, 0, "b", 0, 1),
        // (0, 0, "b", 1, 3),
        // (0, 2, "a", 0, 0),
        // (0, 2, "a", 1, 7),
        // (0, 2, "b", 0, 4),
        // (0, 2, "b", 1, 2),
        // (1, 1, "a", 0, 6),
        // (1, 3, "a", 0, 8),
        //
        // We can deduce how the IDs need to be remapped so they're in increasing order so as not to
        // have any negative deltas:
        //
        //  data:               log_id:  scope_id:
        // -------------------|--------|-----------
        // (0, 0, "a", 0, 0),   9 -> 0    0 = same
        // (0, 0, "a", 1, 1),   5 -> 1
        // (0, 0, "b", 0, 2),   1 -> 2
        // (0, 0, "b", 1, 3),   3 -> 3
        // (0, 1, "a", 0, 4),   0 -> 4    2 -> 1
        // (0, 1, "a", 1, 5),   7 -> 5
        // (0, 1, "b", 0, 6),   4 -> 6
        // (0, 1, "b", 1, 7),   2 -> 7
        // (1, 2, "a", 0, 8),   6 -> 8    1 -> 2
        // (1, 3, "a", 0, 9),   8 -> 9    3 = same
        //
        // Also note that there's no remapping of the resource IDs because it is the primary column
        // we sort by, so it will always be increasing after sorting.
        let spans_data = vec![
            (0, 0, "a", 0, 9),
            (0, 0, "a", 1, 5),
            (0, 0, "b", 0, 1),
            (0, 0, "b", 1, 3),
            (0, 2, "a", 0, 0),
            (0, 2, "a", 1, 7),
            (0, 2, "b", 0, 4),
            (0, 2, "b", 1, 2),
            (1, 1, "a", 0, 6),
            (1, 3, "a", 0, 8),
        ];

        let resource_ids = UInt16Array::from_iter_values(spans_data.iter().map(|d| d.0));
        let scope_ids = UInt16Array::from_iter_values(spans_data.iter().map(|d| d.1));
        let span_names = StringArray::from_iter_values(spans_data.iter().map(|d| d.2));
        let trace_ids =
            FixedSizeBinaryArray::try_from_iter(spans_data.iter().map(|d| u128::to_be_bytes(d.3)))
                .unwrap();
        let ids = UInt16Array::from_iter_values(spans_data.iter().map(|d| d.4));

        let spans_rb = RecordBatch::try_new(
            spans_schema.clone(),
            vec![
                Arc::new(ids),
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(resource_ids)],
                    None,
                )),
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(scope_ids)],
                    None,
                )),
                Arc::new(span_names),
                Arc::new(trace_ids),
            ],
        )
        .unwrap();

        let span_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (1, "a"),
                (2, "a"),
                (1, "c"),
                (2, "c"),
                (5, "c"),
                (7, "c"),
                (2, "b"),
                (4, "b"),
            ],
            consts::metadata::encodings::PLAIN,
        );

        let scope_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (1, "a"),
                (1, "b"),
                (2, "a"),
                (2, "c"),
                (0, "a"),
                (3, "a"),
                (3, "b"),
                (1, "b"),
            ],
            consts::metadata::encodings::PLAIN,
        );

        let resource_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (1, "a"),
                (0, "a"),
                (1, "c"),
                (0, "c"),
                (1, "b"),
                (1, "c"),
                (0, "b"),
            ],
            consts::metadata::encodings::PLAIN,
        );

        let span_event_data = vec![
            (0, 1, "b"),
            (2, 2, "d"),
            (1, 3, "c"),
            (3, 5, "a"),
            (2, 4, "a"),
            (6, 7, "a"),
            (4, 2, "b"),
        ];

        let span_events_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true).with_plain_encoding(),
                Field::new(consts::PARENT_ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(consts::NAME, DataType::Utf8, false),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values(
                    span_event_data.iter().map(|d| d.0),
                )),
                Arc::new(UInt16Array::from_iter_values(
                    span_event_data.iter().map(|d| d.1),
                )),
                Arc::new(StringArray::from_iter_values(
                    span_event_data.iter().map(|d| d.2),
                )),
            ],
        )
        .unwrap();

        let span_events_attrs_rb = attrs_from_data::<UInt32Type>(
            vec![
                (1, "a"),
                (3, "a"),
                (4, "a"),
                (2, "b"),
                (3, "b"),
                (2, "a"),
                (0, "a"),
            ],
            consts::metadata::encodings::PLAIN,
        );

        // for span links, we'll use basically the same ordering of IDs and attributes values
        // as span events, which will make asserting on it a bit easier. The only difference is,
        // we replace the span name with a trace_id that will sort in the same order.
        let span_links_data = vec![
            (0, 1, 2),
            (2, 2, 4),
            (1, 3, 3),
            (3, 5, 1),
            (2, 4, 1),
            (6, 7, 1),
            (4, 2, 2),
        ];
        let span_links_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true).with_plain_encoding(),
                Field::new(consts::PARENT_ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), false),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values(
                    span_links_data.iter().map(|d| d.0),
                )),
                Arc::new(UInt16Array::from_iter_values(
                    span_links_data.iter().map(|d| d.1),
                )),
                Arc::new(FixedSizeBinaryArray::try_from_iter(
                    span_links_data.iter().map(|d| u128::to_be_bytes(d.2))
                ).unwrap()),
            ],
        )
        .unwrap();
        let span_links_attrs_rb = span_events_attrs_rb.clone();

        let mut batch = OtapArrowRecords::Traces(Traces::default());
        batch.set(ArrowPayloadType::Spans, spans_rb);
        batch.set(ArrowPayloadType::SpanAttrs, span_attrs_rb);
        batch.set(ArrowPayloadType::ScopeAttrs, scope_attrs_rb);
        batch.set(ArrowPayloadType::ResourceAttrs, resource_attrs_rb);
        batch.set(ArrowPayloadType::SpanEvents, span_events_rb);
        batch.set(ArrowPayloadType::SpanEventAttrs, span_events_attrs_rb);
        batch.set(ArrowPayloadType::SpanLinks, span_links_rb);
        batch.set(ArrowPayloadType::SpanLinkAttrs, span_links_attrs_rb);

        batch.encode_transport_optimized_ids().unwrap();

        let expected_struct_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, false)
                .with_encoding(consts::metadata::encodings::DELTA),
        ]);
        let expected_spans_schema = Arc::new(Schema::new(vec![
            Field::new(consts::ID, DataType::UInt16, true)
                .with_encoding(consts::metadata::encodings::DELTA),
            Field::new(
                consts::RESOURCE,
                DataType::Struct(expected_struct_fields.clone()),
                true,
            ),
            Field::new(
                consts::SCOPE,
                DataType::Struct(expected_struct_fields.clone()),
                true,
            ),
            Field::new(consts::NAME, DataType::Utf8, true),
            Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), true),
        ]));

        let expected_spans_data = vec![
            (0, 0, "a", 0, 0),
            (0, 0, "a", 1, 1),
            (0, 0, "b", 0, 1),
            (0, 0, "b", 1, 1),
            (0, 1, "a", 0, 1),
            (0, 0, "a", 1, 1),
            (0, 0, "b", 0, 1),
            (0, 0, "b", 1, 1),
            (1, 1, "a", 0, 1),
            (0, 1, "a", 0, 1),
        ];

        let resource_ids = UInt16Array::from_iter_values(expected_spans_data.iter().map(|d| d.0));
        let scope_ids = UInt16Array::from_iter_values(expected_spans_data.iter().map(|d| d.1));
        let span_names = StringArray::from_iter_values(expected_spans_data.iter().map(|d| d.2));
        let trace_ids = FixedSizeBinaryArray::try_from_iter(
            expected_spans_data.iter().map(|d| u128::to_be_bytes(d.3)),
        )
        .unwrap();
        let span_ids = UInt16Array::from_iter_values(expected_spans_data.iter().map(|d| d.4));

        let expected_spans_rb = RecordBatch::try_new(
            expected_spans_schema.clone(),
            vec![
                Arc::new(span_ids),
                Arc::new(StructArray::new(
                    expected_struct_fields.clone(),
                    vec![Arc::new(resource_ids)],
                    None,
                )),
                Arc::new(StructArray::new(
                    expected_struct_fields.clone(),
                    vec![Arc::new(scope_ids)],
                    None,
                )),
                Arc::new(span_names),
                Arc::new(trace_ids),
            ],
        )
        .unwrap();

        let result_spans_rb = batch.get(ArrowPayloadType::Spans).unwrap();
        assert_eq!(result_spans_rb, &expected_spans_rb);

        // attr record batches will get sorted by key, then have the parent IDs remapped
        //
        // original parent_id --> remapped parent_id, val --> quasi-delta parent ID
        // 1 -> 2, "a" -> 2
        // 2 -> 7, "a" -> 5
        // 4 -> 6, "b" -> 6
        // 2 -> 7, "b" -> 1
        // 5 -> 1, "c" -> 1
        // 1 -> 2, "c" -> 1
        // 7 -> 5, "c" -> 3
        // 2 -> 7, "c" -> 2
        let expected_span_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (2, "a"),
                (5, "a"),
                (6, "b"),
                (1, "b"),
                (1, "c"),
                (1, "c"),
                (3, "c"),
                (2, "c"),
            ],
            consts::metadata::encodings::QUASI_DELTA,
        );
        let result_span_attrs_rb = batch.get(ArrowPayloadType::SpanAttrs).unwrap();
        assert_eq!(result_span_attrs_rb, &expected_span_attrs_rb);

        // expected scope attrs
        //
        // original parent_id --> remapped parent_id, val --> quasi-delta parent ID
        // 0 -> 0, "a" -> 0
        // 2 -> 1, "a" -> 1
        // 1 -> 2, "a" -> 1
        // 3 -> 3, "a" -> 1
        // 1 -> 2, "b" -> 2
        // 1 -> 2, "b" -> 0
        // 3 -> 3, "b" -> 1
        // 2 -> 1, "c" -> 1
        let expected_scope_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (0, "a"),
                (1, "a"),
                (1, "a"),
                (1, "a"),
                (2, "b"),
                (0, "b"),
                (1, "b"),
                (1, "c"),
            ],
            consts::metadata::encodings::QUASI_DELTA,
        );
        let result_scope_attrs_rb = batch.get(ArrowPayloadType::ScopeAttrs).unwrap();
        assert_eq!(result_scope_attrs_rb, &expected_scope_attrs_rb);

        // expected resource attrs
        //
        // note that because the record batch gets sorted by resource ID as the primary sort column
        // that there are no remapped IDs, so this is just the original resource attribute batch
        // sorted
        //
        // parent_id, val -> quasi-delta parent ID
        // 0, a -> 0
        // 1, a -> 1
        // 0, b -> 0
        // 1, b -> 1
        // 0, c -> 0
        // 1, c -> 1
        // 1, c -> 0
        //
        let expected_resource_attrs_rb = attrs_from_data::<UInt16Type>(
            vec![
                (0, "a"),
                (1, "a"),
                (0, "b"),
                (1, "b"),
                (0, "c"),
                (1, "c"),
                (0, "c"),
            ],
            consts::metadata::encodings::QUASI_DELTA,
        );
        let result_resource_attrs_rb = batch.get(ArrowPayloadType::ResourceAttrs).unwrap();
        assert_eq!(result_resource_attrs_rb, &expected_resource_attrs_rb);

        // span events get sorted by name,parent_id and have the parent ID remapped. after sorting
        //  and ID remapping, we expect the record batch to look like this:
        //
        //  id:     parent_id:   name:   quasi-delta parent_id
        // (3 -> 0,  5 -> 1,     "a"     1
        // (6 -> 1,  7 -> 5,     "a"     4
        // (2 -> 2,  4 -> 6,     "a"     1
        // (0 -> 3,  1 -> 2,     "b"     2
        // (4 -> 4,  2 -> 7,     "b"     5
        // (1 -> 5,  3 -> 3,     "c"     3
        // (2 -> 6,  2 -> 7,     "d"     7
        //
        // then the IDs will get replaced
        //
        let expected_span_events_data = vec![
            (0, 1, "a"),
            (1, 4, "a"),
            (1, 1, "a"),
            (1, 2, "b"),
            (1, 5, "b"),
            (1, 3, "c"),
            (1, 7, "d"),
        ];
        let expected_span_events_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true)
                    .with_encoding(consts::metadata::encodings::DELTA),
                Field::new(consts::PARENT_ID, DataType::UInt16, true)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::NAME, DataType::Utf8, false),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values(
                    expected_span_events_data.iter().map(|d| d.0),
                )),
                Arc::new(UInt16Array::from_iter_values(
                    expected_span_events_data.iter().map(|d| d.1),
                )),
                Arc::new(StringArray::from_iter_values(
                    expected_span_events_data.iter().map(|d| d.2),
                )),
            ],
        )
        .unwrap();

        let result_span_events_rb = batch.get(ArrowPayloadType::SpanEvents).unwrap();
        assert_eq!(result_span_events_rb, &expected_span_events_rb);

        // because span links for this test case was constructed in a way that should sort and
        // remap the IDs in basically the same ways as span events, see the comment above that
        // expected record batch for comments about why this is data is expected
        let expected_span_links_data = vec![
            (0, 1, 1),
            (1, 4, 1),
            (1, 1, 1),
            (1, 2, 2),
            (1, 5, 2),
            (1, 3, 3),
            (1, 7, 4),
        ];
        let expected_span_links_rb = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt32, true)
                    .with_encoding(consts::metadata::encodings::DELTA),
                Field::new(consts::PARENT_ID, DataType::UInt16, true)
                    .with_encoding(consts::metadata::encodings::QUASI_DELTA),
                Field::new(consts::TRACE_ID, DataType::FixedSizeBinary(16), false),
            ])),
            vec![
                Arc::new(UInt32Array::from_iter_values(
                    expected_span_links_data.iter().map(|d| d.0),
                )),
                Arc::new(UInt16Array::from_iter_values(
                    expected_span_links_data.iter().map(|d| d.1),
                )),
                Arc::new(FixedSizeBinaryArray::try_from_iter(
                    expected_span_links_data.iter().map(|d| u128::to_be_bytes(d.2)),
                ).unwrap()),
            ],
        )
        .unwrap();

        let result_span_links_rb = batch.get(ArrowPayloadType::SpanLinks).unwrap();
        assert_eq!(result_span_links_rb, &expected_span_links_rb);


        // expected span attributes
        //
        // parent_id -> remapped parent_id, val -> quasi-delta parent ID
        // 3 -> 0, "a"  -> 0
        // 0 -> 3, "a"  -> 3
        // 4 -> 4, "a"  -> 1
        // 1 -> 5, "a"  -> 1
        // 2 -> 6, "a"  -> 1
        // 3 -> 0, "b"  -> 0
        // 2 -> 6, "b"  -> 6
        let expected_span_events_attrs_rb = attrs_from_data::<UInt32Type>(
            vec![
                (0, "a"),
                (3, "a"),
                (1, "a"),
                (1, "a"),
                (1, "a"),
                (0, "b"),
                (6, "b"),
            ],
            consts::metadata::encodings::QUASI_DELTA,
        );
        let result_span_events_attrs_rb = batch.get(ArrowPayloadType::SpanEventAttrs).unwrap();
        assert_eq!(result_span_events_attrs_rb, &expected_span_events_attrs_rb);

        // we've constructed span links and span link attributes in a very similar way to span
        // events and span event attrs, so this check should also pass:
        let expected_span_link_attrs_rb = expected_span_events_attrs_rb.clone();
        let result_span_links_attrs_rb = batch.get(ArrowPayloadType::SpanLinkAttrs).unwrap();
        assert_eq!(result_span_links_attrs_rb, &expected_span_link_attrs_rb);
    }
}
