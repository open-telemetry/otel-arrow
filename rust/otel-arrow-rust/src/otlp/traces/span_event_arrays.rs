// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{RecordBatch, TimestampNanosecondArray, UInt16Array, UInt32Array};

use crate::{
    arrays::{
        StringArrayAccessor, get_timestamp_nanosecond_array_opt, get_u16_array, get_u16_array_opt,
        get_u32_array_opt,
    },
    error::{Error, Result},
    proto::opentelemetry::trace::v1::span,
    schema::consts,
};

pub struct SpanEvent {
    pub parent_id: u16,
    pub event: span::Event,
}

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
