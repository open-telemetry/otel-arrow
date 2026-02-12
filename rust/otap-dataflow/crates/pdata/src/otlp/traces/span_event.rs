// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{RecordBatch, TimestampNanosecondArray, UInt16Array, UInt32Array};

use crate::{
    arrays::{
        NullableArrayAccessor, StringArrayAccessor, get_timestamp_nanosecond_array_opt,
        get_u16_array, get_u32_array_opt,
    },
    error::{Error, Result},
    otlp::{
        ProtoBuffer,
        attributes::{Attribute32Arrays, encode_key_value},
        common::{ChildIndexIter, SortedBatchCursor},
    },
    proto::consts::{
        field_num::traces::{
            SPAN_EVENT_ATTRIBUTES, SPAN_EVENT_DROPPED_ATTRIBUTES_COUNTS, SPAN_EVENT_NAME,
            SPAN_EVENT_TIME_UNIX_NANO,
        },
        wire_types,
    },
    proto_encode_len_delimited_unknown_size,
    schema::consts,
};

pub struct SpanEventArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt16Array,
    pub time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub name: Option<StringArrayAccessor<'a>>,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
}

impl<'a> TryFrom<&'a RecordBatch> for SpanEventArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        Ok(Self {
            id: get_u32_array_opt(rb, consts::ID)?,
            parent_id: get_u16_array(rb, consts::PARENT_ID)?,
            time_unix_nano: get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?,
            name: rb
                .column_by_name(consts::NAME)
                .map(StringArrayAccessor::try_new)
                .transpose()?,
            dropped_attributes_count: get_u32_array_opt(rb, consts::DROPPED_ATTRIBUTES_COUNT)?,
        })
    }
}

pub fn encode_span_event(
    index: usize,
    event_arrays: &SpanEventArrays<'_>,
    attrs_cursor: &mut SortedBatchCursor,
    attrs_arrays: Option<&Attribute32Arrays<'_>>,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(col) = &event_arrays.time_unix_nano {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SPAN_EVENT_TIME_UNIX_NANO, wire_types::FIXED64);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    if let Some(col) = &event_arrays.name {
        if let Some(val) = col.str_at(index) {
            result_buf.encode_string(SPAN_EVENT_NAME, val);
        }
    }

    if let Some(attrs) = attrs_arrays {
        if let Some(id) = event_arrays.id.value_at(index) {
            let attrs_index_iter = ChildIndexIter::new(id, &attrs.parent_id, attrs_cursor);
            for attrs_index in attrs_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    SPAN_EVENT_ATTRIBUTES,
                    encode_key_value(attrs, attrs_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = &event_arrays.dropped_attributes_count {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SPAN_EVENT_DROPPED_ATTRIBUTES_COUNTS, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    Ok(())
}
