// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{RecordBatch, UInt16Array, UInt32Array};

use crate::{
    arrays::{
        FixedSizeBinaryArrayAccessor, NullableArrayAccessor, StringArrayAccessor, get_u16_array,
        get_u32_array_opt,
    },
    error::{Error, Result},
    otlp::{
        ProtoBuffer,
        attributes::{Attribute32Arrays, encode_key_value},
        common::{ChildIndexIter, SortedBatchCursor},
    },
    proto::consts::{
        field_num::traces::{
            SPAN_DROPPED_ATTRIBUTES_COUNT, SPAN_LINK_ATTRIBUTES, SPAN_LINK_FLAGS,
            SPAN_LINK_SPAN_ID, SPAN_LINK_TRACE_ID, SPAN_LINK_TRACE_STATE,
        },
        wire_types,
    },
    proto_encode_len_delimited_unknown_size,
    schema::consts,
};

pub struct SpanLinkArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt16Array,
    pub span_id: Option<FixedSizeBinaryArrayAccessor<'a>>,
    pub trace_id: Option<FixedSizeBinaryArrayAccessor<'a>>,
    pub trace_state: Option<StringArrayAccessor<'a>>,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
    pub flags: Option<&'a UInt32Array>,
}

impl<'a> TryFrom<&'a RecordBatch> for SpanLinkArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        Ok(Self {
            id: get_u32_array_opt(rb, consts::ID)?,
            parent_id: get_u16_array(rb, consts::PARENT_ID)?,
            span_id: rb
                .column_by_name(consts::SPAN_ID)
                .map(|arr| FixedSizeBinaryArrayAccessor::try_new(arr, 8))
                .transpose()?,
            trace_id: rb
                .column_by_name(consts::TRACE_ID)
                .map(|arr| FixedSizeBinaryArrayAccessor::try_new(arr, 16))
                .transpose()?,
            trace_state: rb
                .column_by_name(consts::TRACE_STATE)
                .map(StringArrayAccessor::try_new)
                .transpose()?,
            dropped_attributes_count: get_u32_array_opt(rb, consts::DROPPED_ATTRIBUTES_COUNT)?,
            flags: get_u32_array_opt(rb, consts::FLAGS)?,
        })
    }
}

pub fn encode_span_link(
    index: usize,
    link_arrays: &SpanLinkArrays<'_>,
    attrs_cursor: &mut SortedBatchCursor,
    attrs_arrays: Option<&Attribute32Arrays<'_>>,
    result_buf: &mut ProtoBuffer,
) -> Result<()> {
    if let Some(col) = &link_arrays.trace_id {
        if let Some(val) = col.slice_at(index) {
            result_buf.encode_bytes(SPAN_LINK_TRACE_ID, val);
        }
    }

    if let Some(col) = &link_arrays.span_id {
        if let Some(val) = col.slice_at(index) {
            result_buf.encode_bytes(SPAN_LINK_SPAN_ID, val);
        }
    }

    if let Some(col) = &link_arrays.trace_state {
        if let Some(val) = col.str_at(index) {
            result_buf.encode_string(SPAN_LINK_TRACE_STATE, val);
        }
    }

    if let Some(attrs) = attrs_arrays {
        if let Some(id) = link_arrays.id.value_at(index) {
            let attrs_index_iter = ChildIndexIter::new(id, &attrs.parent_id, attrs_cursor);
            for attrs_index in attrs_index_iter {
                proto_encode_len_delimited_unknown_size!(
                    SPAN_LINK_ATTRIBUTES,
                    encode_key_value(attrs, attrs_index, result_buf)?,
                    result_buf
                );
            }
        }
    }

    if let Some(col) = &link_arrays.dropped_attributes_count {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SPAN_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT);
            result_buf.encode_varint(val as u64);
        }
    }

    if let Some(col) = &link_arrays.flags {
        if let Some(val) = col.value_at(index) {
            result_buf.encode_field_tag(SPAN_LINK_FLAGS, wire_types::FIXED32);
            result_buf.extend_from_slice(&val.to_le_bytes());
        }
    }

    Ok(())
}
