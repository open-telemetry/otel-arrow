// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::arrays::{
    ByteArrayAccessor, DurationMillisArrayAccessor, StringArrayAccessor,
    get_timestamp_nanosecond_array_opt, get_u16_array, get_u32_array_opt,
};
use crate::error;
use crate::otlp::traces::spans_status_arrays::SpanStatusArrays;
use crate::schema::consts;
use arrow::array::{
    Int32Array, RecordBatch, StructArray, TimestampNanosecondArray, UInt16Array, UInt32Array,
};
use arrow::datatypes::{DataType, Fields};
use snafu::OptionExt;

pub(crate) struct SpansArrays<'a> {
    pub(crate) id: &'a UInt16Array,
    pub(crate) schema_url: Option<StringArrayAccessor<'a>>,
    pub(crate) trace_id: Option<ByteArrayAccessor<'a>>,
    pub(crate) span_id: Option<ByteArrayAccessor<'a>>,
    pub(crate) parent_span_id: Option<ByteArrayAccessor<'a>>,
    pub(crate) name: Option<StringArrayAccessor<'a>>,
    pub(crate) kind: Option<&'a Int32Array>,
    pub(crate) start_time_unix_nano: Option<&'a TimestampNanosecondArray>,
    pub(crate) duration_time_unix_nano: Option<DurationMillisArrayAccessor<'a>>,
    pub(crate) dropped_attributes_count: Option<&'a UInt32Array>,
    pub(crate) dropped_events_count: Option<&'a UInt32Array>,
    pub(crate) dropped_links_count: Option<&'a UInt32Array>,
    pub(crate) status: Option<SpanStatusArrays<'a>>,
}

impl<'a> TryFrom<&'a RecordBatch> for SpansArrays<'a> {
    type Error = error::Error;

    fn try_from(rb: &'a RecordBatch) -> error::Result<Self> {
        let id = get_u16_array(rb, consts::ID)?;
        let schema_url = rb
            .column_by_name(consts::SCHEMA_URL)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let trace_id = rb
            .column_by_name(consts::TRACE_ID)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;
        let span_id = rb
            .column_by_name(consts::SPAN_ID)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;
        let parent_span_id = rb
            .column_by_name(consts::PARENT_SPAN_ID)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;
        let name = rb
            .column_by_name(consts::NAME)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let kind = rb
            .column_by_name(consts::KIND)
            .map(|arr| {
                arr.as_any().downcast_ref::<Int32Array>().context(
                    error::ColumnDataTypeMismatchSnafu {
                        name: consts::KIND,
                        actual: arr.data_type().clone(),
                        expect: DataType::Int32,
                    },
                )
            })
            .transpose()?;

        let start_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::START_TIME_UNIX_NANO)?;
        // fixme(v0y4g3r): this would be a mistake in go implementation when it encodes nano seconds into DurationMillisecondsArray.
        let duration_time_unix_nano = rb
            .column_by_name(consts::DURATION_TIME_UNIX_NANO)
            .map(|arr| DurationMillisArrayAccessor::try_new(arr))
            .transpose()?;
        let dropped_attributes_count = get_u32_array_opt(rb, consts::DROPPED_ATTRIBUTES_COUNT)?;
        let dropped_events_count = get_u32_array_opt(rb, consts::DROPPED_EVENTS_COUNT)?;
        let dropped_links_count = get_u32_array_opt(rb, consts::DROPPED_LINKS_COUNT)?;

        let status = rb
            .column_by_name(consts::STATUS)
            .map(|arr| {
                let status_struct = arr.as_any().downcast_ref::<StructArray>().context(
                    error::ColumnDataTypeMismatchSnafu {
                        name: consts::STATUS,
                        actual: arr.data_type().clone(),
                        expect: DataType::Struct(Fields::default()),
                    },
                )?;

                SpanStatusArrays::try_from(status_struct)
            })
            .transpose()?;

        Ok(Self {
            id,
            schema_url,
            trace_id,
            span_id,
            parent_span_id,
            name,
            kind,
            start_time_unix_nano,
            duration_time_unix_nano,
            dropped_attributes_count,
            dropped_events_count,
            dropped_links_count,
            status,
        })
    }
}
