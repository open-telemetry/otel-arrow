// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    NullableArrayAccessor, get_f64_array_opt, get_i64_array_opt,
    get_timestamp_nanosecond_array_opt, get_u16_array, get_u32_array_opt,
};
use crate::error::{Error, Result};
use crate::otlp::ProtoBuffer;
use crate::otlp::attributes::{Attribute32Arrays, encode_key_value};
use crate::otlp::common::{ChildIndexIter, SortedBatchCursor};
use crate::otlp::metrics::exemplar::{ExemplarArrays, proto_encode_exemplar};
use crate::proto::consts::field_num::metrics::{
    NUMBER_DP_AS_DOUBLE, NUMBER_DP_AS_INT, NUMBER_DP_ATTRIBUTES, NUMBER_DP_EXEMPLARS,
    NUMBER_DP_FLAGS, NUMBER_DP_START_TIME_UNIX_NANO, NUMBER_DP_TIME_UNIX_NANO,
};
use crate::proto::consts::wire_types;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;
use arrow::array::{
    Float64Array, Int64Array, RecordBatch, TimestampNanosecondArray, UInt16Array, UInt32Array,
};

pub struct NumberDpArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt16Array,
    pub start_time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub int_value: Option<&'a Int64Array>,
    pub double_value: Option<&'a Float64Array>,
    pub flags: Option<&'a UInt32Array>,
}

impl<'a> TryFrom<&'a RecordBatch> for NumberDpArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let id = get_u32_array_opt(rb, consts::ID)?;
        let parent_id = get_u16_array(rb, consts::PARENT_ID)?;
        let start_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        let time_unix_nano = get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?;
        let int_value = get_i64_array_opt(rb, consts::INT_VALUE)?;
        let double_value = get_f64_array_opt(rb, consts::DOUBLE_VALUE)?;
        let flags = get_u32_array_opt(rb, consts::FLAGS)?;

        Ok(Self {
            id,
            parent_id,
            start_time_unix_nano,
            time_unix_nano,
            int_value,
            double_value,
            flags,
        })
    }
}

pub(crate) fn proto_encode_number_data_point(
    index: usize,
    number_dp_arrays: &NumberDpArrays<'_>,
    attr_arrays: Option<&Attribute32Arrays<'_>>,
    attrs_cursor: &mut SortedBatchCursor,
    exemplar_arrays: Option<&ExemplarArrays<'_>>,
    exemplar_cursor: &mut SortedBatchCursor,
    exemplar_attr_arrays: Option<&Attribute32Arrays<'_>>,
    exemplar_attrs_cursor: &mut SortedBatchCursor,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(attrs) = attr_arrays {
        if let Some(id) = number_dp_arrays.id.value_at(index) {
            let attrs_index_iter = ChildIndexIter::new(id, &attrs.parent_id, attrs_cursor);
            for attrs_index in attrs_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    NUMBER_DP_ATTRIBUTES,
                    encode_key_value(attrs, attrs_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = number_dp_arrays.start_time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(NUMBER_DP_START_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = number_dp_arrays.time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(NUMBER_DP_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    let mut value_is_double = false;
    if let Some(col) = number_dp_arrays.double_value {
        if let Some(val) = col.value_at(index) {
            value_is_double = true;
            result_buf.encode_field_tag(NUMBER_DP_AS_DOUBLE, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if !value_is_double {
        if let Some(col) = number_dp_arrays.int_value {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(NUMBER_DP_AS_INT, wire_types::FIXED64);
                result_buf.extend_from_slice(&val.to_le_bytes());
            }
        }
    }

    if let Some(exemplar_arrays) = exemplar_arrays {
        if let Some(id) = number_dp_arrays.id.value_at(index) {
            let exemplar_index_iter =
                ChildIndexIter::new(id, &exemplar_arrays.parent_id, exemplar_cursor);
            for exemplar_index in exemplar_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    NUMBER_DP_EXEMPLARS,
                    proto_encode_exemplar(
                        exemplar_index,
                        exemplar_arrays,
                        exemplar_attr_arrays,
                        exemplar_attrs_cursor,
                        result_buf
                    )?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = number_dp_arrays.flags {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(NUMBER_DP_FLAGS, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    Ok(())
}
