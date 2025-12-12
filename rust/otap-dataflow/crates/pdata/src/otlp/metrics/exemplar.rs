// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    FixedSizeBinaryArrayAccessor, MaybeDictArrayAccessor, NullableArrayAccessor, get_f64_array_opt,
    get_i64_array_opt, get_required_array, get_timestamp_nanosecond_array_opt, get_u32_array_opt,
};
use crate::error::{Error, Result};
use crate::otlp::ProtoBuffer;
use crate::otlp::attributes::{Attribute32Arrays, encode_key_value};
use crate::otlp::common::{ChildIndexIter, SortedBatchCursor};
use crate::proto::consts::field_num::metrics::{
    EXEMPLAR_AS_DOUBLE, EXEMPLAR_AS_INT, EXEMPLAR_FILTERED_ATTRIBUTES, EXEMPLAR_SPAN_ID,
    EXEMPLAR_TIME_UNIX_NANO, EXEMPLAR_TRACE_ID,
};
use crate::proto::consts::wire_types;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{Float64Array, Int64Array, RecordBatch, TimestampNanosecondArray, UInt32Array};

pub struct ExemplarArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: MaybeDictArrayAccessor<'a, UInt32Array>,
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
        let parent_id = MaybeDictArrayAccessor::<UInt32Array>::try_new(get_required_array(
            rb,
            consts::PARENT_ID,
        )?)?;
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
