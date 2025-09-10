// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::thread::Scope;

use arrow::array::{
    Array, BooleanArray, Float64Array, Int64Array, RecordBatch, StringArray, StructArray,
    TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array,
};
use arrow::datatypes::{DataType, Fields};
use related_data::RelatedData;
use snafu::{OptionExt, ResultExt, ensure};

use crate::arrays::{
    ByteArrayAccessor, Int32ArrayAccessor, NullableArrayAccessor, StringArrayAccessor,
    StructColumnAccessor, get_timestamp_nanosecond_array_opt, get_u16_array, get_u32_array_opt,
};
use crate::decode::proto_bytes::resource::ResourceProtoBytesEncoder;
use crate::decode::proto_bytes::{
    IdColumnSorter, encode_field_tag, encode_fixed64, encode_len_placeholder, encode_varint,
    patch_len_placeholder,
};
use crate::encode_len_delimited_mystery_size;
use crate::error::{self, Error, Result};
use crate::otap::OtapArrowRecords;
use crate::otlp::attributes::AttributeArrays;
use crate::otlp::common::{ResourceArrays, ScopeArrays};
use crate::otlp::metrics::AppendAndGet;
use crate::proto::consts::field_num::logs::{
    LOG_RECORD_SEVERITY_TEXT, LOG_RECORD_TIME_UNIX_NANO, LOGS_DATA_RESOURCE,
    RESOURCE_LOGS_SCOPE_LOGS, SCOPE_LOGS_LOG_RECORDS,
};
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use crate::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use crate::proto::opentelemetry::common::v1::AnyValue;
use crate::proto::opentelemetry::common::v1::any_value::Value;
use crate::schema::{consts, is_id_plain_encoded};

use super::attributes::{cbor, store::AttributeValueType};

mod related_data;

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
    value_type: &'a UInt8Array,
    str: Option<StringArrayAccessor<'a>>,
    int: Option<&'a Int64Array>,
    double: Option<&'a Float64Array>,
    bool: Option<&'a BooleanArray>,
    bytes: Option<ByteArrayAccessor<'a>>,

    // ser is for serialized value type of AnyValue including "kvlist" and "array"
    ser: Option<ByteArrayAccessor<'a>>,
}

impl NullableArrayAccessor for LogBodyArrays<'_> {
    type Native = Result<AnyValue>;

    fn value_at(&self, idx: usize) -> Option<Self::Native> {
        if !self.body.is_valid(idx) {
            return None;
        }

        let value_type = AttributeValueType::try_from(self.value_type.value_at_or_default(idx))
            .context(error::UnrecognizedAttributeValueTypeSnafu);
        let value_type = match value_type {
            Ok(v) => v,
            Err(err) => {
                return Some(Err(err));
            }
        };

        if value_type == AttributeValueType::Slice || value_type == AttributeValueType::Map {
            let bytes = self.ser.value_at(idx)?;
            let decode_result = cbor::decode_pcommon_val(&bytes).transpose()?;
            return Some(decode_result.map(|val| AnyValue { value: Some(val) }));
        }

        let value = match value_type {
            AttributeValueType::Str => Value::StringValue(self.str.value_at_or_default(idx)),
            AttributeValueType::Int => Value::IntValue(self.int.value_at_or_default(idx)),
            AttributeValueType::Double => Value::DoubleValue(self.double.value_at_or_default(idx)),
            AttributeValueType::Bool => Value::BoolValue(self.bool.value_at_or_default(idx)),
            AttributeValueType::Bytes => Value::BytesValue(self.bytes.value_at_or_default(idx)),
            _ => {
                // silently ignore unknown types to avoid DOS attacks
                return None;
            }
        };

        Some(Ok(AnyValue { value: Some(value) }))
    }
}

impl<'a> TryFrom<&'a StructArray> for LogBodyArrays<'a> {
    type Error = Error;

    fn try_from(body: &'a StructArray) -> Result<Self> {
        let column_accessor = StructColumnAccessor::new(body);
        Ok(Self {
            body,
            value_type: column_accessor.primitive_column(consts::ATTRIBUTE_TYPE)?,
            str: column_accessor.string_column_op(consts::ATTRIBUTE_STR)?,
            int: column_accessor.primitive_column_op(consts::ATTRIBUTE_INT)?,
            double: column_accessor.primitive_column_op(consts::ATTRIBUTE_DOUBLE)?,
            bool: column_accessor.bool_column_op(consts::ATTRIBUTE_BOOL)?,
            bytes: column_accessor.byte_array_column_op(consts::ATTRIBUTE_BYTES)?,
            ser: column_accessor.byte_array_column_op(consts::ATTRIBUTE_SER)?,
        })
    }
}

pub fn logs_from(logs_otap_batch: OtapArrowRecords) -> Result<ExportLogsServiceRequest> {
    let mut logs = ExportLogsServiceRequest::default();
    let mut prev_res_id: Option<u16> = None;
    let mut prev_scope_id: Option<u16> = None;

    let mut res_id = 0;
    let mut scope_id = 0;

    let rb = logs_otap_batch
        .get(ArrowPayloadType::Logs)
        .context(error::LogRecordNotFoundSnafu)?;

    let mut related_data = RelatedData::try_from(&logs_otap_batch)?;

    let resource_arrays = ResourceArrays::try_from(rb)?;
    let scope_arrays = ScopeArrays::try_from(rb)?;
    let logs_arrays = LogsArrays::try_from(rb)?;

    let ids_plain_encoded = is_id_plain_encoded(rb);

    let resource_ids_plain_encoded = rb
        .column_by_name(consts::RESOURCE)
        .and_then(|col| col.as_any().downcast_ref::<StructArray>())
        .and_then(|col_struct| col_struct.fields().find(consts::ID))
        .and_then(|(_, field)| field.metadata().get(consts::metadata::COLUMN_ENCODING))
        .map(|encoding| encoding.as_str() == consts::metadata::encodings::PLAIN)
        .unwrap_or(false);

    let scope_ids_plain_encoded = rb
        .column_by_name(consts::SCOPE)
        .and_then(|col| col.as_any().downcast_ref::<StructArray>())
        .and_then(|col_struct| col_struct.fields().find(consts::ID))
        .and_then(|(_, field)| field.metadata().get(consts::metadata::COLUMN_ENCODING))
        .map(|encoding| encoding.as_str() == consts::metadata::encodings::PLAIN)
        .unwrap_or(false);

    for idx in 0..rb.num_rows() {
        let res_maybe_delta_id = resource_arrays.id.value_at(idx).unwrap_or_default();
        if resource_ids_plain_encoded {
            res_id = res_maybe_delta_id;
        } else {
            res_id += res_maybe_delta_id;
        }

        if prev_res_id != Some(res_id) {
            // new resource id
            prev_res_id = Some(res_id);
            let resource_logs = logs.resource_logs.append_and_get();
            prev_scope_id = None;

            // Update the resource field of the current resource logs
            let resource = resource_logs.resource.get_or_insert_default();
            if let Some(dropped_attributes_count) =
                resource_arrays.dropped_attributes_count.value_at(idx)
            {
                resource.dropped_attributes_count = dropped_attributes_count;
            }

            if let Some(res_id) = resource_arrays.id.value_at(idx) {
                if let Some(attrs) = related_data.res_attr_map_store.as_mut().and_then(|store| {
                    if resource_ids_plain_encoded {
                        store.attribute_by_id(res_id)
                    } else {
                        store.attribute_by_delta_id(res_id)
                    }
                }) {
                    resource.attributes = attrs.to_vec();
                }
            }

            resource_logs.schema_url = resource_arrays.schema_url.value_at(idx).unwrap_or_default();
        }

        let scope_maybe_delta_id_opt = scope_arrays.id.value_at(idx);
        if scope_ids_plain_encoded {
            scope_id = scope_maybe_delta_id_opt.unwrap_or_default();
        } else {
            scope_id += scope_maybe_delta_id_opt.unwrap_or_default();
        }

        if prev_scope_id != Some(scope_id) {
            prev_scope_id = Some(scope_id);
            let mut scope = scope_arrays.create_instrumentation_scope(idx);
            if let Some(scope_id) = scope_maybe_delta_id_opt {
                if let Some(attrs) = related_data
                    .scope_attr_map_store
                    .as_mut()
                    .and_then(|store| {
                        if scope_ids_plain_encoded {
                            store.attribute_by_id(scope_id)
                        } else {
                            store.attribute_by_delta_id(scope_id)
                        }
                    })
                {
                    scope.attributes = attrs.to_vec();
                }
            }

            // safety: we must have appended at least one resource logs when reach here
            let current_scope_logs_slice = &mut logs
                .resource_logs
                .last_mut()
                .expect("At this stage, we should have at least one resource log.")
                .scope_logs;
            let scope_logs = current_scope_logs_slice.append_and_get();
            scope_logs.scope = Some(scope);
            scope_logs.schema_url = logs_arrays.schema_url.value_at(idx).unwrap_or_default();
        }

        // safety: we've appended at least one value at each slice when reach here.
        let current_scope_logs = &mut logs
            .resource_logs
            .last_mut()
            .expect("At this stage, we should have at least one resource log.")
            .scope_logs
            .last_mut()
            .expect("At this stage, we should have added at least one scope log.");

        let current_log_record = current_scope_logs.log_records.append_and_get();
        let maybe_delta_id = logs_arrays.id.value_at_or_default(idx);
        let log_id = if ids_plain_encoded {
            maybe_delta_id
        } else {
            related_data.log_record_id_from_delta(maybe_delta_id)
        };

        current_log_record.time_unix_nano =
            logs_arrays.time_unix_nano.value_at_or_default(idx) as u64;
        current_log_record.observed_time_unix_nano =
            logs_arrays.observed_time_unix_nano.value_at_or_default(idx) as u64;

        if let Some(trace_id_bytes) = logs_arrays.trace_id.value_at(idx) {
            ensure!(
                trace_id_bytes.len() == 16,
                error::InvalidTraceIdSnafu {
                    message: format!(
                        "log_id = {log_id}, index = {idx}, trace_id = {trace_id_bytes:?}"
                    ),
                }
            );
            current_log_record.trace_id = trace_id_bytes
        }

        if let Some(span_id_bytes) = logs_arrays.span_id.value_at(idx) {
            ensure!(
                span_id_bytes.len() == 8,
                error::InvalidSpanIdSnafu {
                    message: format!(
                        "log_id = {log_id}, index = {idx}, span_id = {span_id_bytes:?}"
                    ),
                }
            );
            current_log_record.span_id = span_id_bytes;
        }

        current_log_record.severity_number = logs_arrays.severity_number.value_at_or_default(idx);
        current_log_record.severity_text = logs_arrays.severity_text.value_at_or_default(idx);
        current_log_record.dropped_attributes_count = logs_arrays
            .dropped_attributes_count
            .value_at_or_default(idx);
        current_log_record.flags = logs_arrays.flags.value_at_or_default(idx);
        current_log_record.event_name = logs_arrays.event_name.value_at_or_default(idx);

        if let Some(body_val) = logs_arrays.body.value_at(idx) {
            current_log_record.body = Some(body_val?)
        }

        if let Some(attrs) = related_data
            .log_record_attr_map_store
            .as_mut()
            .and_then(|store| {
                if ids_plain_encoded {
                    store.attribute_by_id(maybe_delta_id)
                } else {
                    store.attribute_by_delta_id(maybe_delta_id)
                }
            })
        {
            current_log_record.attributes = attrs.to_vec()
        }
    }

    Ok(logs)
}

pub struct LogsProtoBytesEncoder {
    // TODO is name "column" here too explicit
    id_column_sorter: IdColumnSorter,
    root_columns_indices_sorted: Vec<usize>,
    root_column_sorted_index: usize,

    resource_attrs_sorted_indices: Vec<usize>,
    resource_attrs_sorted_index: usize,

    scope_attrs_sorted_indices: Vec<usize>,
    scope_attrs_sorted_index: usize,

    log_attrs_sorted_indices: Vec<usize>,
    log_attrs_sorted_index: usize,


    resource_encoder: ResourceProtoBytesEncoder,
}

impl LogsProtoBytesEncoder {
    pub fn new() -> Self {
        // TODO -- is there any way to estimate capacity here?
        Self {
            id_column_sorter: IdColumnSorter::new(),

            // TODO -- should abstract this into a struct?
            root_columns_indices_sorted: Vec::new(),
            root_column_sorted_index: 0,

            resource_attrs_sorted_indices: Vec::new(),
            resource_attrs_sorted_index: 0,

            scope_attrs_sorted_indices: Vec::new(),
            scope_attrs_sorted_index: 0,

            log_attrs_sorted_indices: Vec::new(),
            log_attrs_sorted_index: 0,

            resource_encoder: ResourceProtoBytesEncoder::new(),
        }
    }

    fn reset(&mut self) {
        self.root_columns_indices_sorted.clear();
        self.root_column_sorted_index = 0;
        self.resource_attrs_sorted_indices.clear();
        self.resource_attrs_sorted_index = 0;
        self.scope_attrs_sorted_indices.clear();
        self.scope_attrs_sorted_index = 0;
        self.log_attrs_sorted_indices.clear();
        self.log_attrs_sorted_index = 0;
    }

    /// TODO comments
    pub fn encode(
        &mut self,
        otap_batch: &OtapArrowRecords,
        result_buf: &mut Vec<u8>,
    ) -> Result<()> {
        // TODO -- do we need to ensure the buf is empty?

        // TODO nounwrap
        // TODO the otap_batch needs to have the IDs materialized
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let logs_arrays = LogsArrays::try_from(logs_rb).unwrap();
        let scope_arrays = ScopeArrays::try_from(logs_rb).unwrap();
        let resource_arrays = ResourceArrays::try_from(logs_rb).unwrap();

        let resource_attrs = otap_batch.get(ArrowPayloadType::ResourceAttrs)
            .map(AttributeArrays::try_from)
            .transpose()?;
        let scope_attrs = otap_batch.get(ArrowPayloadType::ScopeAttrs)
            .map(AttributeArrays::try_from)
            .transpose()?;
        let log_attrs = otap_batch.get(ArrowPayloadType::LogAttrs)
            .map(AttributeArrays::try_from)
            .transpose()?;

        self.reset();

        // get the list of indices in the root record to visit in order
        self.id_column_sorter
            .root_indices_sorted(logs_rb, &mut self.root_columns_indices_sorted);
        self.root_column_sorted_index = 0;

        if let Some(resource_attrs) = resource_attrs.as_ref() {
            self.id_column_sorter
                .u16_ids_sorted(resource_attrs.parent_id, &mut self.resource_attrs_sorted_indices);
        }
        if let Some(scope_attrs) = scope_attrs.as_ref() {
            self.id_column_sorter
                .u16_ids_sorted(scope_attrs.parent_id, &mut self.scope_attrs_sorted_indices);
        }
        if let Some(log_attrs) = log_attrs.as_ref() {
            self.id_column_sorter
                .u16_ids_sorted(log_attrs.parent_id, &mut self.log_attrs_sorted_indices);
        }

        // encode all `ResourceLog`s for this `LogsData`
        loop {
            let num_bytes = 5;
            encode_len_delimited_mystery_size!(
                LOGS_DATA_RESOURCE,
                num_bytes,
                self.encode_resource_log(
                    &logs_arrays,
                    &scope_arrays,
                    &resource_arrays,
                    resource_attrs.as_ref(),
                    scope_attrs.as_ref(),
                    log_attrs.as_ref(),
                    result_buf,
                ),
                result_buf
            );

            if self.root_column_sorted_index >= logs_rb.num_rows() {
                break;
            }
        }

        Ok(())
    }

    fn encode_resource_log(
        &mut self,
        logs_arrays: &LogsArrays<'_>,
        scope_arrays: &ScopeArrays<'_>,
        resource_arrays: &ResourceArrays<'_>,
        resource_attrs: Option<&AttributeArrays<'_>>,
        scope_attrs: Option<&AttributeArrays<'_>>,
        log_attrs: Option<&AttributeArrays<'_>>,
        result_buf: &mut Vec<u8>,
    ) {
        let resource_id = resource_arrays.id.value_at(self.curr_root_rb_index());

        // encode all `ScopeLog`s for this `ResourceLog`
        loop {
            let num_bytes = 5;
            encode_len_delimited_mystery_size!(
                RESOURCE_LOGS_SCOPE_LOGS,
                num_bytes,
                self.encode_scope_logs(
                    logs_arrays,
                    scope_arrays,
                    scope_attrs,
                    log_attrs,
                    result_buf,
                ),
                result_buf
            );

            // break when we've reached the end of the record batch
            if self.root_column_sorted_index >= self.root_columns_indices_sorted.len() {
                break;
            }

            // check if we've found a new scope ID. If so, break
            if resource_id != resource_arrays.id.value_at(self.curr_root_rb_index()) {
                break;
            }
        }
    }

    fn encode_scope_logs(
        &mut self,
        log_arrays: &LogsArrays<'_>,
        scope_arrays: &ScopeArrays<'_>,
        scope_attrs: Option<&AttributeArrays<'_>>,
        log_attrs: Option<&AttributeArrays<'_>>,
        result_buf: &mut Vec<u8>,
    ) {
        let scope_id = scope_arrays.id.value_at(self.curr_root_rb_index());

        // encode all `LogRecord`s for this `ScopeLog``
        loop {
            let num_bytes = 5;
            encode_len_delimited_mystery_size!(
                SCOPE_LOGS_LOG_RECORDS,
                num_bytes,
                self.encode_log_record(log_arrays, log_attrs, result_buf),
                result_buf
            );

            // break if we've reached the end of the record batch
            if self.root_column_sorted_index >= self.root_columns_indices_sorted.len() {
                break;
            }

            // check if we've found a new scope ID. If so, break
            if scope_id != scope_arrays.id.value_at(self.curr_root_rb_index()) {
                break;
            }
        }
    }

    fn encode_log_record(&mut self,
        log_arrays: &LogsArrays<'_>, 
        log_attrs: Option<&AttributeArrays<'_>>,
        result_buf: &mut Vec<u8>
    ) {
        let index = self.curr_root_rb_index();

        if let Some(col) = log_arrays.time_unix_nano {
            if let Some(val) = col.value_at(index) {
                encode_field_tag(LOG_RECORD_TIME_UNIX_NANO, wire_types::FIXED64, result_buf);
                encode_fixed64(val as u64, result_buf);
            }
        }

        if let Some(col) = &log_arrays.severity_text {
            if let Some(val) = col.value_at(index) {
                encode_field_tag(LOG_RECORD_SEVERITY_TEXT, wire_types::LEN, result_buf);
                encode_varint(val.len() as u64, result_buf);
                result_buf.extend_from_slice(val.as_bytes());
            }
        }

        self.advance_root_rb_index();
    }

    #[inline]
    fn curr_root_rb_index(&self) -> usize {
        self.root_columns_indices_sorted[self.root_column_sorted_index]
    }

    fn advance_root_rb_index(&mut self) {
        self.root_column_sorted_index += 1;
    }
}

#[cfg(test)]
mod test {
    use arrow::array::{
        RecordBatch, StringArray, StructArray, TimestampNanosecondArray, UInt16Array,
    };

    use crate::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use crate::schema::consts;
    use crate::{otap::OtapArrowRecords, otlp::logs::LogsProtoBytesEncoder};
    use arrow::datatypes::{DataType, Field, Fields, Schema, TimeUnit};
    use prost::Message;
    use std::sync::Arc;

    use crate::{otap::Logs, proto::opentelemetry::logs::v1::LogsData};

    use super::*;

    #[test]
    fn albert_smoke_test() {
        let struct_fields = Fields::from(vec![Field::new(consts::ID, DataType::UInt16, true)]);

        let record_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(struct_fields.clone()),
                    true,
                ),
                Field::new(consts::SCOPE, DataType::Struct(struct_fields.clone()), true),
                Field::new(consts::ID, DataType::UInt16, true),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(consts::SEVERITY_TEXT, DataType::Utf8, true),
            ])),
            vec![
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values([0, 1, 1]))],
                    None,
                )),
                Arc::new(StructArray::new(
                    struct_fields.clone(),
                    vec![Arc::new(UInt16Array::from_iter_values([0, 1, 2]))],
                    None,
                )),
                Arc::new(UInt16Array::new_null(3)),
                Arc::new(TimestampNanosecondArray::from_iter_values([1, 2, 3])),
                Arc::new(StringArray::from_iter_values(vec![
                    "ERROR", "INFO", "DEBUG",
                ])),
            ],
        )
        .unwrap();

        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch.set(ArrowPayloadType::Logs, record_batch);
        let mut result_buf = vec![];
        let mut encoder = LogsProtoBytesEncoder::new();
        encoder.encode(&otap_batch, &mut result_buf).unwrap();

        println!("{:?}", result_buf);
        let result = LogsData::decode(result_buf.as_ref()).unwrap();

        println!("{:#?}", result);
    }
}
