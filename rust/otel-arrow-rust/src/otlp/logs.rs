// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::array::{
    Array, RecordBatch, StructArray, TimestampNanosecondArray, UInt16Array, UInt32Array,
};
use arrow::datatypes::{DataType, Fields};
use snafu::OptionExt;

use crate::arrays::{
    ByteArrayAccessor, Int32ArrayAccessor, NullableArrayAccessor, StringArrayAccessor,
    StructColumnAccessor, get_timestamp_nanosecond_array_opt, get_u16_array, get_u32_array_opt,
};
use crate::error::{self, Error, Result};
use crate::otap::OtapArrowRecords;
use crate::otlp::attributes::{AttributeArrays, encode_any_value, encode_key_value};
use crate::otlp::common::{
    AnyValueArrays, BatchSorter, ChildIndexIter, ProtoBuffer, ResourceArrays, ScopeArrays,
    SortedBatchCursor, proto_encode_instrumentation_scope, proto_encode_resource,
};
use crate::proto::consts::field_num::logs::{
    LOG_RECORD_ATTRIBUTES, LOG_RECORD_BODY, LOG_RECORD_DROPPED_ATTRIBUTES_COUNT,
    LOG_RECORD_EVENT_NAME, LOG_RECORD_FLAGS, LOG_RECORD_OBSERVED_TIME_UNIX_NANO,
    LOG_RECORD_SEVERITY_NUMBER, LOG_RECORD_SEVERITY_TEXT, LOG_RECORD_SPAN_ID,
    LOG_RECORD_TIME_UNIX_NANO, LOG_RECORD_TRACE_ID, LOGS_DATA_RESOURCE, RESOURCE_LOGS_SCHEMA_URL,
    RESOURCE_LOGS_SCOPE_LOGS, SCOPE_LOG_SCOPE, SCOPE_LOGS_LOG_RECORDS, SCOPE_LOGS_SCHEMA_URL,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::proto::opentelemetry::common::v1::AnyValue;
use crate::proto_encode_len_delimited_unknown_size;
use crate::schema::consts;

use super::attributes::store::AttributeValueType;

struct LogsArrays<'a> {
    id: &'a UInt16Array,
    schema_url: Option<StringArrayAccessor<'a>>,
    time_unix_nano: Option<&'a TimestampNanosecondArray>,
    observed_time_unix_nano: Option<&'a TimestampNanosecondArray>,
    trace_id: Option<ByteArrayAccessor<'a>>,
    span_id: Option<ByteArrayAccessor<'a>>,
    severity_number: Option<Int32ArrayAccessor<'a>>,
    severity_text: Option<StringArrayAccessor<'a>>,
    body: Option<LogBodyArrays<'a>>,
    dropped_attributes_count: Option<&'a UInt32Array>,
    flags: Option<&'a UInt32Array>,
    event_name: Option<StringArrayAccessor<'a>>,
}

impl<'a> TryFrom<&'a RecordBatch> for LogsArrays<'a> {
    type Error = Error;

    fn try_from(rb: &'a RecordBatch) -> Result<Self> {
        let id = get_u16_array(rb, consts::ID)?;
        let schema_url = rb
            .column_by_name(consts::SCHEMA_URL)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let time_unix_nano = get_timestamp_nanosecond_array_opt(rb, consts::TIME_UNIX_NANO)?;
        let observed_time_unix_nano =
            get_timestamp_nanosecond_array_opt(rb, consts::OBSERVED_TIME_UNIX_NANO)?;
        let trace_id = rb
            .column_by_name(consts::TRACE_ID)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;
        let span_id = rb
            .column_by_name(consts::SPAN_ID)
            .map(ByteArrayAccessor::try_new)
            .transpose()?;
        let severity_number = rb
            .column_by_name(consts::SEVERITY_NUMBER)
            .map(Int32ArrayAccessor::try_new)
            .transpose()?;
        let severity_text = rb
            .column_by_name(consts::SEVERITY_TEXT)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let dropped_attributes_count = get_u32_array_opt(rb, consts::DROPPED_ATTRIBUTES_COUNT)?;
        let flags = get_u32_array_opt(rb, consts::FLAGS)?;
        let event_name = rb
            .column_by_name(consts::EVENT_NAME)
            .map(StringArrayAccessor::try_new)
            .transpose()?;

        let body = rb
            .column_by_name(consts::BODY)
            .map(|arr| {
                let logs_body = arr.as_any().downcast_ref::<StructArray>().context(
                    error::ColumnDataTypeMismatchSnafu {
                        name: consts::BODY,
                        actual: arr.data_type().clone(),
                        expect: DataType::Struct(Fields::default()),
                    },
                )?;

                LogBodyArrays::try_from(logs_body)
            })
            .transpose()?;

        Ok(Self {
            id,
            schema_url,
            time_unix_nano,
            observed_time_unix_nano,
            span_id,
            trace_id,
            severity_number,
            severity_text,
            body,
            dropped_attributes_count,
            flags,
            event_name,
        })
    }
}

struct LogBodyArrays<'a> {
    body: &'a StructArray,
    anyval_arrays: AnyValueArrays<'a>,
}

impl<'a> LogBodyArrays<'a> {
    fn is_valid(&self, idx: usize) -> bool {
        self.body.is_valid(idx)
    }
}

impl NullableArrayAccessor for LogBodyArrays<'_> {
    type Native = Result<AnyValue>;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if !self.is_valid(idx) {
            return None;
        }

        self.anyval_arrays.value_at(idx)
    }
}

impl<'a> TryFrom<&'a StructArray> for LogBodyArrays<'a> {
    type Error = Error;

    fn try_from(body: &'a StructArray) -> Result<Self> {
        let column_accessor = StructColumnAccessor::new(body);

        Ok(Self {
            body,
            anyval_arrays: AnyValueArrays {
                attr_type: column_accessor.primitive_column(consts::ATTRIBUTE_TYPE)?,
                attr_str: column_accessor.string_column_op(consts::ATTRIBUTE_STR)?,
                attr_double: column_accessor.primitive_column_op(consts::ATTRIBUTE_DOUBLE)?,
                attr_bool: column_accessor.bool_column_op(consts::ATTRIBUTE_BOOL)?,
                attr_int: column_accessor.int64_column_op(consts::ATTRIBUTE_INT)?,
                attr_bytes: column_accessor.byte_array_column_op(consts::ATTRIBUTE_BYTES)?,
                attr_ser: column_accessor.byte_array_column_op(consts::ATTRIBUTE_SER)?,
            },
        })
    }
}

pub struct LogsDataArrays<'a> {
    log_arrays: LogsArrays<'a>,
    scope_arrays: ScopeArrays<'a>,
    resource_arrays: ResourceArrays<'a>,
    log_attrs: Option<AttributeArrays<'a>>,
    resource_attrs: Option<AttributeArrays<'a>>,
    scope_attrs: Option<AttributeArrays<'a>>,
}

pub struct LogsProtoBytesEncoder {
    batch_sorter: BatchSorter,
    root_cursor: SortedBatchCursor,
    resource_attrs_cursor: SortedBatchCursor,
    scope_attrs_cursor: SortedBatchCursor,
    log_attrs_cursor: SortedBatchCursor,
}

impl Default for LogsProtoBytesEncoder {
    fn default() -> Self {
        Self::new()
    }
}

impl LogsProtoBytesEncoder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            batch_sorter: BatchSorter::new(),
            root_cursor: SortedBatchCursor::new(),
            resource_attrs_cursor: SortedBatchCursor::new(),
            scope_attrs_cursor: SortedBatchCursor::new(),
            log_attrs_cursor: SortedBatchCursor::new(),
        }
    }

    fn reset(&mut self) {
        self.root_cursor.reset();
        self.resource_attrs_cursor.reset();
        self.scope_attrs_cursor.reset();
        self.log_attrs_cursor.reset();
    }

    /// encode the OTAP batch into a proto serialized `LogData`/`ExportLogsServiceRequest` message
    pub fn encode(
        &mut self,
        otap_batch: &mut OtapArrowRecords,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        otap_batch.decode_transport_optimized_ids()?;

        let logs_rb = otap_batch
            .get(ArrowPayloadType::Logs)
            .context(error::LogRecordNotFoundSnafu)?;

        let logs_data_arrays = LogsDataArrays {
            log_arrays: LogsArrays::try_from(logs_rb)?,
            scope_arrays: ScopeArrays::try_from(logs_rb)?,
            resource_arrays: ResourceArrays::try_from(logs_rb)?,
            log_attrs: otap_batch
                .get(ArrowPayloadType::LogAttrs)
                .map(AttributeArrays::try_from)
                .transpose()?,
            scope_attrs: otap_batch
                .get(ArrowPayloadType::ScopeAttrs)
                .map(AttributeArrays::try_from)
                .transpose()?,
            resource_attrs: otap_batch
                .get(ArrowPayloadType::ResourceAttrs)
                .map(AttributeArrays::try_from)
                .transpose()?,
        };

        self.reset();

        // get the list of indices in the root record to visit in order
        self.batch_sorter
            .init_cursor_for_root_batch(logs_rb, &mut self.root_cursor)?;

        // get the lists of child indices for attributes to visit in oder:
        if let Some(res_attrs) = logs_data_arrays.resource_attrs.as_ref() {
            self.batch_sorter.init_cursor_for_u16_id_column(
                res_attrs.parent_id,
                &mut self.resource_attrs_cursor,
            );
        }
        if let Some(scope_attrs) = logs_data_arrays.scope_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(scope_attrs.parent_id, &mut self.scope_attrs_cursor);
        }
        if let Some(log_attrs) = logs_data_arrays.log_attrs.as_ref() {
            self.batch_sorter
                .init_cursor_for_u16_id_column(log_attrs.parent_id, &mut self.log_attrs_cursor);
        }

        // encode all `ResourceLog`s for this `LogsData`
        loop {
            proto_encode_len_delimited_unknown_size!(
                LOGS_DATA_RESOURCE,
                self.encode_resource_log(&logs_data_arrays, result_buf)?,
                result_buf
            );

            if self.root_cursor.finished() {
                break;
            }
        }

        Ok(())
    }

    fn encode_resource_log(
        &mut self,
        logs_data_arrays: &LogsDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()), // no more rows to visit
        };

        // encode the `Resource`
        proto_encode_len_delimited_unknown_size!(
            LOGS_DATA_RESOURCE,
            proto_encode_resource(
                index,
                &logs_data_arrays.resource_arrays,
                logs_data_arrays.resource_attrs.as_ref(),
                &mut self.resource_attrs_cursor,
                result_buf
            )?,
            result_buf
        );

        // encode all `ScopeLog`s for this `ResourceLog`
        let resource_id = logs_data_arrays.resource_arrays.id.value_at(index);

        loop {
            proto_encode_len_delimited_unknown_size!(
                RESOURCE_LOGS_SCOPE_LOGS,
                self.encode_scope_logs(logs_data_arrays, result_buf)?,
                result_buf
            );

            // break when we've reached the end of the record batch
            if self.root_cursor.finished() {
                break;
            }

            // check if we've found a new scope ID. If so, break
            let next_index = self.root_cursor.curr_index().expect("cursor not finished");
            if resource_id != logs_data_arrays.resource_arrays.id.value_at(next_index) {
                break;
            }
        }

        // encode schema url
        if let Some(col) = &logs_data_arrays.resource_arrays.schema_url {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(RESOURCE_LOGS_SCHEMA_URL, wire_types::LEN);
                result_buf.encode_varint(val.len() as u64);
                result_buf.extend_from_slice(val.as_bytes());
            }
        }

        Ok(())
    }

    fn encode_scope_logs(
        &mut self,
        logs_data_arrays: &LogsDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()), // no more rows to visit
        };
        // encode the `InstrumentationScope`
        proto_encode_len_delimited_unknown_size!(
            SCOPE_LOG_SCOPE,
            proto_encode_instrumentation_scope(
                index,
                &logs_data_arrays.scope_arrays,
                logs_data_arrays.scope_attrs.as_ref(),
                &mut self.scope_attrs_cursor,
                result_buf
            )?,
            result_buf
        );

        // encode all `LogRecord`s for this `ScopeLog``
        let scope_id = logs_data_arrays.scope_arrays.id.value_at(index);

        loop {
            proto_encode_len_delimited_unknown_size!(
                SCOPE_LOGS_LOG_RECORDS,
                self.encode_log_record(logs_data_arrays, result_buf)?,
                result_buf
            );

            // break if we've reached the end of the record batch
            if self.root_cursor.finished() {
                break;
            }

            // check if we've found a new scope ID. If so, break
            // Safety: we've just checked above that cursor isn't finished
            let next_index = self.root_cursor.curr_index().expect("cursor not finished");
            if scope_id != logs_data_arrays.scope_arrays.id.value_at(next_index) {
                break;
            }
        }

        // encode schema url
        if let Some(col) = &logs_data_arrays.log_arrays.schema_url {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(SCOPE_LOGS_SCHEMA_URL, wire_types::LEN);
                result_buf.encode_varint(val.len() as u64);
                result_buf.extend_from_slice(val.as_bytes());
            }
        }

        Ok(())
    }

    fn encode_log_record(
        &mut self,
        logs_data_arrays: &LogsDataArrays<'_>,
        result_buf: &mut ProtoBuffer,
    ) -> Result<()> {
        let index = match self.root_cursor.curr_index() {
            Some(index) => index,
            None => return Ok(()), // no more rows to visit
        };

        let log_arrays = &logs_data_arrays.log_arrays;

        if let Some(col) = log_arrays.time_unix_nano {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64);
                result_buf.extend_from_slice(&val.to_le_bytes());
            }
        }

        if let Some(col) = &log_arrays.severity_number {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(LOG_RECORD_SEVERITY_NUMBER, wire_types::VARINT);
                result_buf.encode_varint(val as u64);
            }
        }

        if let Some(col) = &log_arrays.severity_text {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_field_tag(LOG_RECORD_SEVERITY_TEXT, wire_types::LEN);
                result_buf.encode_varint(val.len() as u64);
                result_buf.extend_from_slice(val.as_bytes());
            }
        }

        if let Some(log_body_arrays) = &logs_data_arrays.log_arrays.body {
            if log_body_arrays.is_valid(index) {
                let anyval_arrays = &log_body_arrays.anyval_arrays;
                if let Some(value_type) = anyval_arrays.attr_type.value_at(index) {
                    if let Ok(value_type) = AttributeValueType::try_from(value_type) {
                        proto_encode_len_delimited_unknown_size!(
                            LOG_RECORD_BODY,
                            encode_any_value(anyval_arrays, index, value_type, result_buf)?,
                            result_buf
                        );
                    }
                }
            }
        }

        if let Some(log_attrs) = logs_data_arrays.log_attrs.as_ref() {
            if let Some(id) = log_arrays.id.value_at(index) {
                let attrs_index_iter =
                    ChildIndexIter::new(id, log_attrs.parent_id, &mut self.log_attrs_cursor);
                for attr_index in attrs_index_iter {
                    proto_encode_len_delimited_unknown_size!(
                        LOG_RECORD_ATTRIBUTES,
                        encode_key_value(log_attrs, attr_index, result_buf)?,
                        result_buf
                    );
                }
            }
        }

        if let Some(col) = log_arrays.dropped_attributes_count {
            if let Some(val) = col.value_at(index) {
                result_buf
                    .encode_field_tag(LOG_RECORD_DROPPED_ATTRIBUTES_COUNT, wire_types::VARINT);
                result_buf.encode_varint(val as u64);
            }
        }

        if let Some(col) = &log_arrays.flags {
            if let Some(val) = col.value_at(index) {
                result_buf.encode_field_tag(LOG_RECORD_FLAGS, wire_types::FIXED32);
                result_buf.extend_from_slice(&val.to_le_bytes());
            }
        }

        if let Some(col) = &log_arrays.trace_id {
            if let Some(val) = col.slice_at(index) {
                result_buf.encode_field_tag(LOG_RECORD_TRACE_ID, wire_types::LEN);
                result_buf.encode_varint(val.len() as u64);
                result_buf.extend_from_slice(val);
            }
        }

        if let Some(col) = &log_arrays.span_id {
            if let Some(val) = col.slice_at(index) {
                result_buf.encode_field_tag(LOG_RECORD_SPAN_ID, wire_types::LEN);
                result_buf.encode_varint(val.len() as u64);
                result_buf.extend_from_slice(val);
            }
        }

        if let Some(col) = log_arrays.observed_time_unix_nano {
            if let Some(val) = col.value_at(index) {
                result_buf
                    .encode_field_tag(LOG_RECORD_OBSERVED_TIME_UNIX_NANO, wire_types::FIXED64);
                result_buf.extend_from_slice(&val.to_le_bytes());
            }
        }

        if let Some(col) = &log_arrays.event_name {
            if let Some(val) = col.str_at(index) {
                result_buf.encode_field_tag(LOG_RECORD_EVENT_NAME, wire_types::LEN);
                result_buf.encode_varint(val.len() as u64);
                result_buf.extend_from_slice(val.as_bytes());
            }
        }

        self.root_cursor.advance();
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use arrow::array::{
        RecordBatch, StringArray, StructArray, TimestampNanosecondArray, UInt8Array, UInt16Array,
    };
    use arrow::buffer::NullBuffer;
    use arrow::datatypes::{DataType, Field, Fields, Schema, TimeUnit};
    use pretty_assertions::assert_eq;
    use prost::Message;
    use std::sync::Arc;

    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::proto::opentelemetry::common::v1::{InstrumentationScope, KeyValue};
    use crate::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use crate::proto::opentelemetry::resource::v1::Resource;
    use crate::schema::{FieldExt, consts};
    use crate::{otap::Logs, proto::opentelemetry::logs::v1::LogsData};
    use crate::{otap::OtapArrowRecords, otlp::logs::LogsProtoBytesEncoder};

    #[test]
    fn test_proto_encode() {
        // simple smoke test for proto encoding. This doesn't test every field, but those are
        // tested in other test suites in this project that encode/decode OTAP -> OTLP

        let res_struct_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
        ]);
        let scope_struct_fields = Fields::from(vec![
            Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
            Field::new(consts::NAME, DataType::Utf8, true),
        ]);
        let body_struct_fields = Fields::from(vec![
            Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
            Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        ]);

        let logs_record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(res_struct_fields.clone()),
                    true,
                ),
                Field::new(
                    consts::SCOPE,
                    DataType::Struct(scope_struct_fields.clone()),
                    true,
                ),
                Field::new(consts::ID, DataType::UInt16, true).with_plain_encoding(),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
                Field::new(
                    consts::BODY,
                    DataType::Struct(body_struct_fields.clone()),
                    true,
                ),
            ])),
            vec![
                Arc::new(StructArray::new(
                    res_struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values([0, 1, 1]))],
                    None,
                )),
                Arc::new(StructArray::new(
                    scope_struct_fields.clone(),
                    vec![
                        Arc::new(UInt16Array::from_iter_values([0, 1, 2])),
                        Arc::new(StringArray::from_iter_values(vec![
                            "scope0", "scope1", "scope2",
                        ])),
                    ],
                    None,
                )),
                Arc::new(UInt16Array::from_iter_values(vec![0, 1, 2])),
                Arc::new(TimestampNanosecondArray::from_iter_values([1, 2, 3])),
                Arc::new(StringArray::from_iter_values(vec![
                    "ERROR", "INFO", "DEBUG",
                ])),
                Arc::new(StructArray::new(
                    body_struct_fields.clone(),
                    vec![
                        Arc::new(UInt8Array::from_iter_values(std::iter::repeat_n(
                            AttributeValueType::Str as u8,
                            3,
                        ))),
                        Arc::new(StringArray::from_iter_values(vec!["a", "b", ""])),
                    ],
                    Some(NullBuffer::from_iter(vec![true, true, false])),
                )),
            ],
        )
        .unwrap();

        let attrs_record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
                Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![0, 1, 1])),
                Arc::new(UInt8Array::from_iter_values(std::iter::repeat_n(
                    AttributeValueType::Str as u8,
                    3,
                ))),
                Arc::new(StringArray::from_iter_values(vec!["ka", "ka", "kb"])),
                Arc::new(StringArray::from_iter_values(vec!["va", "va", "vb"])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch.set(ArrowPayloadType::Logs, logs_record_batch);
        otap_batch.set(ArrowPayloadType::LogAttrs, attrs_record_batch.clone());
        otap_batch.set(ArrowPayloadType::ResourceAttrs, attrs_record_batch.clone());
        otap_batch.set(ArrowPayloadType::ScopeAttrs, attrs_record_batch.clone());
        let mut result_buf = ProtoBuffer::new();
        let mut encoder = LogsProtoBytesEncoder::new();
        encoder.encode(&mut otap_batch, &mut result_buf).unwrap();

        let result = LogsData::decode(result_buf.as_ref()).unwrap();

        let id_0_attrs = vec![KeyValue::new("ka", AnyValue::new_string("va"))];
        let id_1_attrs = vec![
            KeyValue::new("ka", AnyValue::new_string("va")),
            KeyValue::new("kb", AnyValue::new_string("vb")),
        ];

        let expected = LogsData::new(vec![
            ResourceLogs::build(Resource {
                attributes: id_0_attrs.clone(),
                ..Default::default()
            })
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope {
                    name: "scope0".to_string(),
                    attributes: id_0_attrs.clone(),
                    ..Default::default()
                })
                .log_records(vec![LogRecord {
                    time_unix_nano: 1,
                    severity_text: "ERROR".to_string(),
                    body: Some(AnyValue::new_string("a")),
                    attributes: id_0_attrs.clone(),
                    ..Default::default()
                }])
                .finish(),
            ])
            .finish(),
            ResourceLogs::build(Resource {
                attributes: id_1_attrs.clone(),
                ..Default::default()
            })
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope {
                    name: "scope1".to_string(),
                    attributes: id_1_attrs.clone(),
                    ..Default::default()
                })
                .log_records(vec![LogRecord {
                    time_unix_nano: 2,
                    severity_text: "INFO".to_string(),
                    body: Some(AnyValue::new_string("b")),
                    attributes: id_1_attrs,
                    ..Default::default()
                }])
                .finish(),
                ScopeLogs::build(InstrumentationScope {
                    name: "scope2".to_string(),
                    ..Default::default()
                })
                .log_records(vec![LogRecord {
                    time_unix_nano: 3,
                    severity_text: "DEBUG".to_string(),
                    ..Default::default()
                }])
                .finish(),
            ])
            .finish(),
        ]);

        assert_eq!(result, expected);
    }
}
