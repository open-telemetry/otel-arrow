// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    ByteArrayAccessor, FixedSizeBinaryArrayAccessor, NullableArrayAccessor, get_f64_array_opt,
    get_i64_array_opt, get_timestamp_nanosecond_array, get_timestamp_nanosecond_array_opt,
    get_u32_array, get_u32_array_opt,
};
use crate::error::{self, Error, Result};
use crate::otlp::ProtoBuffer;
use crate::otlp::attributes::store::Attribute32Store;
use crate::otlp::attributes::{Attribute32Arrays, encode_key_value};
use crate::otlp::common::{ChildIndexIter, SortedBatchCursor};
use crate::otlp::metrics::AppendAndGet;
use crate::proto::consts::field_num::metrics::{
    EXEMPLAR_AS_DOUBLE, EXEMPLAR_AS_INT, EXEMPLAR_FILTERED_ATTRIBUTES, EXEMPLAR_SPAN_ID,
    EXEMPLAR_TIME_UNIX_NANO, EXEMPLAR_TRACE_ID,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::metrics::v1::Exemplar;
use crate::proto::opentelemetry::metrics::v1::exemplar::Value;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{Float64Array, Int64Array, RecordBatch, TimestampNanosecondArray, UInt32Array};
use num_enum::TryFromPrimitive;
use snafu::ensure;
use std::collections::HashMap;

pub struct ExemplarArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt32Array,
    pub time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub int_value: Option<&'a Int64Array>,
    pub double_value: Option<&'a Float64Array>,
    pub span_id: Option<FixedSizeBinaryArrayAccessor<'a>>,
    pub trace_id: Option<FixedSizeBinaryArrayAccessor<'a>>,
}

impl<'a> TryFrom<&'a RecordBatch> for ExemplarArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let id = get_u32_array_opt(rb, consts::ID)?;
        let parent_id = get_u32_array(rb, consts::PARENT_ID)?;
        let time_unix_nano = get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?;
        let int_value = get_i64_array_opt(rb, consts::INT_VALUE)?;
        let double_value = get_f64_array_opt(rb, consts::DOUBLE_VALUE)?;
        let span_id = rb
            .column_by_name(consts::SPAN_ID)
            .map(|arr| FixedSizeBinaryArrayAccessor::try_new(arr, 8))
            .transpose()?;
        let trace_id = rb
            .column_by_name(consts::TRACE_ID)
            .map(|arr| FixedSizeBinaryArrayAccessor::try_new(arr, 16))
            .transpose()?;

        Ok(Self {
            id,
            parent_id,
            time_unix_nano,
            int_value,
            double_value,
            span_id,
            trace_id,
        })
    }
}

#[derive(Default)]
pub struct ExemplarsStore {
    // This field is also not used anywhere in otel-arrow: https://github.com/open-telemetry/otel-arrow/blob/985aa1500a012859cec44855e187eacf46eda7c8/pkg/otel/metrics/otlp/exemplar.go#L49
    #[allow(unused)]
    next_id: u32,
    exemplars_by_ids: HashMap<u32, Vec<Exemplar>>,
}

impl ExemplarsStore {
    /// Gets or creates the exemplar of given id and creates a new one if not yet created.
    pub fn get_or_create_exemplar_by_id(&mut self, id: u32) -> &mut Vec<Exemplar> {
        self.exemplars_by_ids.entry(id).or_default()
    }
}

impl ExemplarsStore {
    pub fn try_from(rb: &RecordBatch, attr_store: &mut Attribute32Store) -> Result<Self> {
        let mut exemplars_store = Self::default();
        let mut parent_id_decoder =
            ExemplarParentIdDecoder::new(ParentIdEncoding::ParentIdDeltaGroupEncoding);

        let id_arr_opt = get_u32_array_opt(rb, consts::ID)?;
        let int_value_arr = get_i64_array_opt(rb, consts::INT_VALUE)?;
        let double_value_arr = get_f64_array_opt(rb, consts::DOUBLE_VALUE)?;
        let parent_id_arr = get_u32_array(rb, consts::PARENT_ID)?;
        let time_unix_nano_arr = get_timestamp_nanosecond_array(rb, consts::TIME_UNIX_NANO)?;
        let span_id_arr = ByteArrayAccessor::try_new_for_column(rb, consts::SPAN_ID)?;
        let trace_id_arr = ByteArrayAccessor::try_new_for_column(rb, consts::TRACE_ID)?;

        for idx in 0..rb.num_rows() {
            let int_value = int_value_arr.value_at(idx);
            let double_value = double_value_arr.value_at(idx);
            let parent_id = parent_id_decoder.decode(
                parent_id_arr.value_at_or_default(idx),
                int_value,
                double_value,
            );
            let existing_exemplars = exemplars_store
                .exemplars_by_ids
                .entry(parent_id)
                .or_default();
            let current_exemplar = existing_exemplars.append_and_get();

            let id_opt = id_arr_opt.value_at(idx);

            let time_unix_nano = time_unix_nano_arr.value_at_or_default(idx);
            current_exemplar.time_unix_nano = time_unix_nano as u64;

            let span_id_bytes = span_id_arr.value_at_or_default(idx);
            ensure!(
                span_id_bytes.len() == 8,
                error::InvalidSpanIdSnafu {
                    message: format!("rb: {rb:?}"),
                }
            );
            current_exemplar.span_id = span_id_bytes;

            let trace_id_bytes = trace_id_arr.value_at_or_default(idx);
            ensure!(
                trace_id_bytes.len() == 16,
                error::InvalidTraceIdSnafu {
                    message: format!("rb: {rb:?}"),
                }
            );
            current_exemplar.trace_id = trace_id_bytes;

            match (int_value, double_value) {
                (Some(int_value), None) => {
                    current_exemplar.value = Some(Value::AsInt(int_value));
                }

                (None, Some(double_value)) => {
                    current_exemplar.value = Some(Value::AsDouble(double_value))
                }
                _ => {
                    return error::InvalidExemplarDataSnafu {
                        message: format!("record batch: {rb:?}"),
                    }
                    .fail();
                }
            }

            if let Some(id) = id_opt {
                if let Some(attrs) = attr_store.attribute_by_delta_id(id) {
                    current_exemplar.filtered_attributes = attrs.to_vec();
                }
            }
        }

        Ok(exemplars_store)
    }
}

//todo: maybe merge with [attribute_decoder::ParentIdEncoding]
#[allow(clippy::enum_variant_names)]
#[derive(Eq, PartialEq, Debug, TryFromPrimitive)]
#[repr(u8)]
enum ParentIdEncoding {
    /// ParentIdNoEncoding stores the parent ID as is.
    ParentIdNoEncoding = 0,
    /// ParentIdDeltaEncoding stores the parent ID as a delta from the previous
    /// parent ID.
    ParentIdDeltaEncoding = 1,
    /// ParentIdDeltaGroupEncoding stores the parent ID as a delta from the
    /// previous parent ID in the same group. A group is defined by the
    /// combination Key and Value.
    ParentIdDeltaGroupEncoding = 2,
}

#[derive(Eq, PartialEq, Debug)]
enum ExemplarValueType {
    Undefined = 0,
    Int = 1,
    Double = 2,
}

struct ExemplarParentIdDecoder {
    encoding: ParentIdEncoding,
    prev_parent_id: u32,
    prev_type: ExemplarValueType,
    prev_int_value: Option<i64>,
    prev_double_value: Option<f64>,
}

impl ExemplarParentIdDecoder {
    fn new(encoding: ParentIdEncoding) -> ExemplarParentIdDecoder {
        Self {
            encoding,
            prev_parent_id: 0,
            prev_type: ExemplarValueType::Undefined,
            prev_int_value: None,
            prev_double_value: None,
        }
    }

    fn decode(
        &mut self,
        parent_id_or_delta: u32,
        int_value: Option<i64>,
        double_value: Option<f64>,
    ) -> u32 {
        match self.encoding {
            ParentIdEncoding::ParentIdNoEncoding => parent_id_or_delta,
            ParentIdEncoding::ParentIdDeltaEncoding => {
                self.prev_parent_id += parent_id_or_delta;
                self.prev_parent_id
            }
            ParentIdEncoding::ParentIdDeltaGroupEncoding => {
                if let Some(int_value) = int_value {
                    return if self.prev_type == ExemplarValueType::Int
                        && self.prev_int_value == Some(int_value)
                    {
                        self.prev_parent_id += parent_id_or_delta;
                        self.prev_parent_id
                    } else {
                        self.prev_type = ExemplarValueType::Int;
                        self.prev_int_value = Some(int_value);
                        self.prev_double_value = None;
                        self.prev_parent_id = parent_id_or_delta;
                        self.prev_parent_id
                    };
                }
                if let Some(double_value) = double_value {
                    return if self.prev_type == ExemplarValueType::Double
                        && self.prev_double_value == Some(double_value)
                    {
                        self.prev_parent_id += parent_id_or_delta;
                        self.prev_parent_id
                    } else {
                        self.prev_type = ExemplarValueType::Double;
                        self.prev_double_value = Some(double_value);
                        self.prev_int_value = None;
                        self.prev_parent_id = parent_id_or_delta;
                        self.prev_parent_id
                    };
                }

                self.prev_parent_id += parent_id_or_delta;
                self.prev_parent_id
            }
        }
    }
}

pub(crate) fn proto_encode_exemplar(
    index: usize,
    exemplar_arrays: &ExemplarArrays<'_>,
    attr_arrays: Option<&Attribute32Arrays<'_>>,
    attrs_cursor: &mut SortedBatchCursor,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(attrs) = attr_arrays {
        if let Some(id) = exemplar_arrays.id.value_at(index) {
            let attr_index_iter = ChildIndexIter::new(id, &attrs.parent_id, attrs_cursor);
            for attrs_index in attr_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    EXEMPLAR_FILTERED_ATTRIBUTES,
                    encode_key_value(attrs, attrs_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = exemplar_arrays.time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(EXEMPLAR_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    let mut value_is_double = false;
    if let Some(col) = exemplar_arrays.double_value {
        if let Some(val) = col.value_at(index) {
            value_is_double = true;
            result_buf.encode_field_tag(EXEMPLAR_AS_DOUBLE, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if !value_is_double {
        if let Some(col) = exemplar_arrays.int_value {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(EXEMPLAR_AS_INT, wire_types::FIXED64);
                result_buf.extend_from_slice(&val.to_le_bytes());
            }
        }
    }

    if let Some(col) = &exemplar_arrays.span_id {
        if let Some(val) = col.slice_at(index) {
            result_buf.encode_bytes(EXEMPLAR_SPAN_ID, val);
        }
    }

    if let Some(col) = &exemplar_arrays.trace_id {
        if let Some(val) = col.slice_at(index) {
            result_buf.encode_bytes(EXEMPLAR_TRACE_ID, val);
        }
    }

    Ok(())
}
