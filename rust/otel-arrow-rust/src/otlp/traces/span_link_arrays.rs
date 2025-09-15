// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{RecordBatch, UInt16Array, UInt32Array};

use crate::{
    arrays::{FixedSizeBinaryArrayAccessor, StringArrayAccessor, get_u16_array, get_u32_array_opt},
    error::{Error, Result},
    proto::opentelemetry::trace::v1::span,
    schema::consts,
};

pub struct SpanLink {
    pub parent_id: u16,
    pub link: span::Link,
}

pub struct SpanLinkArrays<'a> {
    pub id: Option<&'a UInt32Array>,
    pub parent_id: &'a UInt16Array,
    pub span_id: Option<FixedSizeBinaryArrayAccessor<'a>>,
    pub trace_id: Option<FixedSizeBinaryArrayAccessor<'a>>,
    pub trace_state: Option<StringArrayAccessor<'a>>,
    pub dropped_attributes_count: Option<&'a UInt32Array>,
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
        })
    }
}
