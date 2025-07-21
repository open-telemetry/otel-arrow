// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otap_df_pdata_views::views::{
    common::{AnyValueView, AttributeView, InstrumentationScopeView, ValueType},
    logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView},
    resource::ResourceView,
    trace::{
        EventView, LinkView, ResourceSpansView, ScopeSpansView, SpanView, StatusView, TracesView,
    },
};
use otel_arrow_rust::{
    encode::record::{
        attributes::{AttributesRecordBatchBuilder, AttributesRecordBatchBuilderConstructorHelper},
        logs::LogsRecordBatchBuilder,
        spans::{EventsBuilder, LinksBuilder, SpansRecordBatchBuilder},
    },
    otap::{Logs, OtapBatch, Traces},
    otlp::attributes::parent_id::ParentId,
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
};

use crate::encoder::error::{Error, Result};

mod cbor;
mod error;

/// Traverse the trace structure within the TracesView and produces an `OtapBatch' for the span
/// data.
pub fn encode_spans_otap_batch<T>(traces_view: &T) -> Result<OtapBatch>
where
    T: TracesView,
{
    let mut resource_attrs = AttributesRecordBatchBuilder::<u16>::new();
    let mut scope_attrs = AttributesRecordBatchBuilder::<u16>::new();
    let mut span_attrs = AttributesRecordBatchBuilder::<u16>::new();
    let mut event_attrs = AttributesRecordBatchBuilder::<u32>::new();
    let mut link_attrs = AttributesRecordBatchBuilder::<u32>::new();

    let mut curr_resource_id: u16 = 0;
    let mut curr_scope_id: u16 = 0;
    let mut curr_span_id: u16 = 0;
    let mut curr_event_id: u32 = 0;
    let mut curr_link_id: u32 = 0;

    let mut spans = SpansRecordBatchBuilder::new();
    let mut events = EventsBuilder::new();
    let mut links = LinksBuilder::new();

    // First, we traverse the view collecting the trace data into our RecordBatch builders.

    #[allow(clippy::explicit_counter_loop)]
    for resource_spans in traces_view.resources() {
        if let Some(resource) = resource_spans.resource() {
            for kv in resource.attributes() {
                resource_attrs.append_parent_id(&curr_resource_id);
                append_attribute_value(&mut resource_attrs, &kv)?;
            }
        }

        // Hoist Resource id, schema_url and dropped_attributes_count handling out of the loop over
        // scope_spans.
        {
            let span_count: usize = resource_spans
                .scopes()
                .map(|scope| scope.spans().count())
                .sum();
            let resource_schema_url = resource_spans.schema_url();
            let resource_dropped_attributes_count = resource_spans
                .resource()
                .map(|r| r.dropped_attributes_count())
                .unwrap_or(0);
            let resource = &mut spans.resource;
            // FIXME: Arrow's array builders support a `append_value_n` method that adds repeats, so
            // consider adding that functionality to `encode::record::array`. At the very least we
            // should dispatch to `append_value_n` or `append_nulls` after matching against an
            // `Option` only once.
            (0..span_count).for_each(|_| resource.append_id(Some(curr_resource_id)));
            (0..span_count).for_each(|_| resource.append_schema_url(resource_schema_url));
            (0..span_count).for_each(|_| {
                resource.append_dropped_attributes_count(resource_dropped_attributes_count)
            });
        }

        for scope_spans in resource_spans.scopes() {
            if let Some(scope) = scope_spans.scope() {
                for kv in scope.attributes() {
                    scope_attrs.append_parent_id(&curr_scope_id);
                    append_attribute_value(&mut scope_attrs, &kv)?;
                }
            }

            for span in scope_spans.spans() {
                // set the scope
                spans.scope.append_id(Some(curr_scope_id));
                if let Some(scope) = scope_spans.scope() {
                    spans.scope.append_name(scope.name());
                    spans.scope.append_version(scope.version());
                    spans
                        .scope
                        .append_dropped_attributes_count(scope.dropped_attributes_count());
                } else {
                    spans.scope.append_name(None);
                    spans.scope.append_version(None);
                    spans.scope.append_dropped_attributes_count(0);
                }

                spans.append_id(Some(curr_span_id));

                for kv in span.attributes() {
                    span_attrs.append_parent_id(&curr_span_id);
                    append_attribute_value(&mut span_attrs, &kv)?;
                }

                spans.append_start_time_unix_nano(span.start_time_unix_nano().map(|v| v as i64));
                let duration = match (span.start_time_unix_nano(), span.end_time_unix_nano()) {
                    (Some(start), Some(end)) => Some((end as i64) - (start as i64)),
                    _ => None,
                };
                spans.append_duration_time_unix_nano(duration);

                spans.append_trace_id(span.trace_id())?;
                spans.append_span_id(span.span_id())?;
                spans.append_trace_state(span.trace_state());
                spans.append_parent_span_id(span.parent_span_id())?;
                spans.append_name(span.name());
                spans.append_kind(Some(span.kind()));
                spans.append_dropped_attributes_count(span.dropped_attributes_count());
                spans.append_dropped_events_count(span.dropped_events_count());
                spans.append_dropped_links_count(span.dropped_links_count());

                if let Some(status) = span.status() {
                    spans.append_status_code(Some(status.status_code()));
                    spans.append_status_status_message(status.message());
                } else {
                    spans.append_status_code(None);
                    spans.append_status_status_message(None);
                }

                for event in span.events() {
                    events.append_id(Some(curr_event_id));
                    events.append_parent_id(curr_span_id);
                    events.append_time_unix_nano(event.time_unix_nano().map(|v| v as i64));
                    events.append_name(event.name());
                    events.append_dropped_attributes_count(event.dropped_attributes_count());

                    for kv in event.attributes() {
                        event_attrs.append_parent_id(&curr_event_id);
                        append_attribute_value(&mut event_attrs, &kv)?;
                    }

                    curr_event_id = curr_event_id
                        .checked_add(1)
                        .ok_or(Error::U32OverflowError)?;
                }

                for link in span.links() {
                    links.append_id(Some(curr_link_id));
                    links.append_parent_id(curr_span_id);
                    links.append_trace_id(link.trace_id())?;
                    links.append_span_id(link.span_id())?;
                    links.append_trace_state(link.trace_state());
                    links.append_dropped_attributes_count(link.dropped_attributes_count());

                    for kv in link.attributes() {
                        link_attrs.append_parent_id(&curr_link_id);
                        append_attribute_value(&mut link_attrs, &kv)?;
                    }

                    curr_link_id = curr_link_id.checked_add(1).ok_or(Error::U32OverflowError)?;
                }

                curr_span_id = curr_span_id.checked_add(1).ok_or(Error::U16OverflowError)?;
            }

            curr_scope_id = curr_scope_id
                .checked_add(1)
                .ok_or(Error::U16OverflowError)?;
        }

        curr_resource_id = curr_resource_id
            .checked_add(1)
            .ok_or(Error::U16OverflowError)?;
    }

    // Then we build up an OTAP Batch from the RecordBatch builders....

    let mut otap_batch = OtapBatch::Traces(Traces::default());

    // Append spans records along with events and links!
    otap_batch.set(ArrowPayloadType::Spans, spans.finish()?);
    otap_batch.set(ArrowPayloadType::SpanEvents, events.finish()?);
    otap_batch.set(ArrowPayloadType::SpanLinks, links.finish()?);

    // Append attrs for spans, scopes, resources, events and links!
    let span_attrs_rb = span_attrs.finish()?;
    if span_attrs_rb.num_rows() > 0 {
        otap_batch.set(ArrowPayloadType::SpanAttrs, span_attrs_rb);
    }

    let resource_attrs_rb = resource_attrs.finish()?;
    if resource_attrs_rb.num_rows() > 0 {
        otap_batch.set(ArrowPayloadType::ResourceAttrs, resource_attrs_rb);
    }

    let scope_attrs_rb = scope_attrs.finish()?;
    if scope_attrs_rb.num_rows() > 0 {
        otap_batch.set(ArrowPayloadType::ScopeAttrs, scope_attrs_rb);
    }

    let event_attrs_rb = event_attrs.finish()?;
    if event_attrs_rb.num_rows() > 0 {
        otap_batch.set(ArrowPayloadType::SpanEventAttrs, event_attrs_rb);
    }

    let link_attrs_rb = link_attrs.finish()?;
    if link_attrs_rb.num_rows() > 0 {
        otap_batch.set(ArrowPayloadType::SpanLinkAttrs, link_attrs_rb);
    }

    Ok(otap_batch)
}

/// traverse the log structure within the LogDataView and produces an `OtapBatch' for the log data
pub fn encode_logs_otap_batch<T>(logs_view: &T) -> Result<OtapBatch>
where
    T: LogsDataView,
{
    let mut resource_attrs = AttributesRecordBatchBuilder::<u16>::new();

    let mut curr_scope_id = 0;
    let mut scope_attrs = AttributesRecordBatchBuilder::<u16>::new();

    let mut curr_log_id = 0;
    let mut logs = LogsRecordBatchBuilder::new();
    let mut log_attrs = AttributesRecordBatchBuilder::<u16>::new();

    for (curr_resource_id, resource_logs) in logs_view.resources().enumerate() {
        let curr_resource_id = curr_resource_id as u16;

        // keep reference to resource dropped attributes, which will be appended to log later
        let resource_dropped_attrs_count = if let Some(resource) = resource_logs.resource() {
            // append resource attributes
            for kv in resource.attributes() {
                resource_attrs.append_parent_id(&curr_resource_id);
                append_attribute_value(&mut resource_attrs, &kv)?;
            }
            resource.dropped_attributes_count()
        } else {
            0
        };

        let resource_schema_url = resource_logs.schema_url();

        let mut resource_log_count = 0;

        for scope_logs in resource_logs.scopes() {
            let scope = scope_logs.scope();

            let (scope_name, scope_version, scope_dropped_attributes_count) =
                if let Some(scope) = scope.as_ref() {
                    // since there is an instrumentations scope present, append the attributes
                    for kv in scope.attributes() {
                        scope_attrs.append_parent_id(&curr_scope_id);
                        append_attribute_value(&mut scope_attrs, &kv)?;
                    }

                    // keep track of scope fields, which will be appended to log later on
                    (
                        scope.name(),
                        scope.version(),
                        scope.dropped_attributes_count(),
                    )
                } else {
                    (None, None, 0)
                };

            let scope_schema_url = scope_logs.schema_url();

            let mut logs_record_iter = scope_logs.log_records();

            let mut scope_log_count = 0;

            const CHUNK_SIZE: usize = 64;
            loop {
                let mut logs_count = 0;
                let log_records_chunk: [_; CHUNK_SIZE] = std::array::from_fn(|_| {
                    if let Some(log_record) = logs_record_iter.next() {
                        logs_count += 1;
                        Some(log_record)
                    } else {
                        // if there are no more log records, return a default value
                        None
                    }
                });

                if logs_count == 0 {
                    break;
                }
                scope_log_count += logs_count;

                let log_records_slice = &log_records_chunk[..logs_count];

                // Set the log record fields for all logs in this scope
                for log_record in log_records_slice {
                    logs.append_time_unix_nano(
                        log_record
                            .as_ref()
                            .expect("LogRecord should not be None")
                            .time_unix_nano()
                            .map(|v| v as i64),
                    );
                }
                for log_record in log_records_slice {
                    logs.append_observed_time_unix_nano(
                        log_record
                            .as_ref()
                            .expect("LogRecord should not be None")
                            .observed_time_unix_nano()
                            .map(|v| v as i64),
                    );
                }
                logs.append_schema_url_n(scope_schema_url, logs_count);
                for log_record in log_records_slice {
                    logs.append_severity_number(
                        log_record
                            .as_ref()
                            .expect("LogRecord should not be None")
                            .severity_number(),
                    );
                }
                for log_record in log_records_slice {
                    logs.append_severity_text(
                        log_record
                            .as_ref()
                            .expect("LogRecord should not be None")
                            .severity_text(),
                    );
                }
                for log_record in log_records_slice {
                    logs.append_dropped_attributes_count(
                        log_record
                            .as_ref()
                            .expect("LogRecord should not be None")
                            .dropped_attributes_count(),
                    );
                }
                for log_record in log_records_slice {
                    logs.append_flags(
                        log_record
                            .as_ref()
                            .expect("LogRecord should not be None")
                            .flags(),
                    );
                }
                for log_record in log_records_slice {
                    logs.append_trace_id(
                        log_record
                            .as_ref()
                            .expect("LogRecord should not be None")
                            .trace_id(),
                    )?;
                }
                for log_record in log_records_slice {
                    logs.append_span_id(
                        log_record
                            .as_ref()
                            .expect("LogRecord should not be None")
                            .span_id(),
                    )?;
                }

                for log_record in log_records_slice {
                    if let Some(body) = log_record
                        .as_ref()
                        .expect("LogRecord should not be None")
                        .body()
                    {
                        match body.value_type() {
                            ValueType::String => {
                                logs.body
                                    .append_str(body.as_string().expect("body to be string"));
                            }
                            ValueType::Double => {
                                logs.body
                                    .append_double(body.as_double().expect("body to be double"));
                            }
                            ValueType::Int64 => {
                                logs.body
                                    .append_int(body.as_int64().expect("body to be int64"));
                            }
                            ValueType::Bool => {
                                logs.body
                                    .append_bool(body.as_bool().expect("body to be bool"));
                            }
                            ValueType::Bytes => {
                                logs.body
                                    .append_bytes(body.as_bytes().expect("body to be bytes"));
                            }
                            ValueType::Array => {
                                let mut serialized_value = vec![];
                                cbor::serialize_any_values(
                                    body.as_array().expect("body to be array"),
                                    &mut serialized_value,
                                )?;
                                logs.body.append_slice(&serialized_value);
                            }

                            ValueType::KeyValueList => {
                                let mut serialized_value = vec![];
                                cbor::serialize_kv_list(
                                    body.as_kvlist().expect("body to be kvlist"),
                                    &mut serialized_value,
                                )?;
                                logs.body.append_map(&serialized_value);
                            }
                            ValueType::Empty => {
                                logs.body.append_null();
                            }
                        }
                    } else {
                        logs.body.append_null();
                    }
                }

                for log_record in log_records_slice {
                    let mut log_attrs_count = 0;
                    for kv in log_record
                        .as_ref()
                        .expect("LogRecord should not be None")
                        .attributes()
                    {
                        log_attrs.append_parent_id(&curr_log_id);
                        log_attrs_count += 1;
                        append_attribute_value(&mut log_attrs, &kv)?;
                    }

                    if log_attrs_count > 0 {
                        logs.append_id(Some(curr_log_id));
                        curr_log_id += 1;
                    } else {
                        logs.append_id(None);
                    }
                }

                // If we didn't fill the entire array, this was the last chunk
                if logs_count < CHUNK_SIZE {
                    break;
                }
            }

            logs.scope.append_id_n(curr_scope_id, scope_log_count);
            logs.scope.append_name_n(scope_name, scope_log_count);
            logs.scope.append_version_n(scope_version, scope_log_count);
            logs.scope
                .append_dropped_attributes_count_n(scope_dropped_attributes_count, scope_log_count);

            resource_log_count += scope_log_count;
            curr_scope_id = curr_scope_id
                .checked_add(1)
                .ok_or(Error::U16OverflowError)?;
        }

        logs.resource
            .append_id_n(curr_resource_id, resource_log_count);
        logs.resource
            .append_schema_url_n(resource_schema_url, resource_log_count);
        logs.resource
            .append_dropped_attributes_count_n(resource_dropped_attrs_count, resource_log_count);
    }

    let mut otap_batch = OtapBatch::Logs(Logs::default());

    // append logs record
    otap_batch.set(ArrowPayloadType::Logs, logs.finish()?);

    // append log attrs record batch if there is one
    let log_attrs_rb = log_attrs.finish()?;
    if log_attrs_rb.num_rows() > 0 {
        otap_batch.set(ArrowPayloadType::LogAttrs, log_attrs_rb);
    }

    let resource_attrs_rb = resource_attrs.finish()?;
    if resource_attrs_rb.num_rows() > 0 {
        otap_batch.set(ArrowPayloadType::ResourceAttrs, resource_attrs_rb);
    }

    let scope_attrs_rb = scope_attrs.finish()?;
    if scope_attrs_rb.num_rows() > 0 {
        otap_batch.set(ArrowPayloadType::ScopeAttrs, scope_attrs_rb);
    }

    Ok(otap_batch)
}

fn append_attribute_value<T, KV>(
    attribute_rb_builder: &mut AttributesRecordBatchBuilder<T>,
    kv: &KV,
) -> Result<()>
where
    T: ParentId + AttributesRecordBatchBuilderConstructorHelper,
    KV: AttributeView,
{
    let key = kv.key();
    attribute_rb_builder.append_key(key);

    if let Some(val) = kv.value() {
        match val.value_type() {
            ValueType::String => {
                attribute_rb_builder.append_str(val.as_string().expect("value to be string"));
            }
            ValueType::Int64 => {
                attribute_rb_builder.append_int(val.as_int64().expect("value to be int64"))
            }
            ValueType::Double => {
                attribute_rb_builder.append_double(val.as_double().expect("value to be double"));
            }
            ValueType::Bool => {
                attribute_rb_builder.append_bool(val.as_bool().expect("value to be bool"));
            }
            ValueType::Bytes => {
                attribute_rb_builder.append_bytes(val.as_bytes().expect("value to be bytes"))
            }
            ValueType::Array => {
                let mut serialized_values = vec![];
                cbor::serialize_any_values(
                    val.as_array().expect("value to be array"),
                    &mut serialized_values,
                )?;
                attribute_rb_builder.append_slice(&serialized_values)
            }
            ValueType::KeyValueList => {
                let mut serialized_value = vec![];
                cbor::serialize_kv_list(
                    val.as_kvlist().expect("value is kvlist"),
                    &mut serialized_value,
                )?;
                attribute_rb_builder.append_map(&serialized_value);
            }
            ValueType::Empty => {
                attribute_rb_builder.append_empty();
            }
        }
    } else {
        attribute_rb_builder.append_empty();
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use super::*;

    use arrow::array::{
        ArrayRef, BinaryArray, BooleanArray, DictionaryArray, DurationNanosecondArray,
        FixedSizeBinaryArray, Float64Array, Int32Array, Int64Array, RecordBatch, StringArray,
        StructArray, TimestampNanosecondArray, UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::buffer::NullBuffer;
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit, UInt8Type, UInt16Type};
    use otap_df_pdata_views::otlp::bytes::logs::RawLogsData;
    use otel_arrow_rust::otlp::attributes::cbor::decode_pcommon_val;
    use otel_arrow_rust::otlp::attributes::store::AttributeValueType;
    use otel_arrow_rust::proto::opentelemetry::common::v1::{
        AnyValue, ArrayValue, InstrumentationScope, KeyValue, KeyValueList, any_value,
    };
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{
        LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_rust::schema::consts;
    use prost::Message;

    fn _generate_logs_for_verify_all_columns() -> LogsData {
        LogsData::new(vec![
            ResourceLogs::build(
                Resource::build(vec![KeyValue::new(
                    "resource_attr1",
                    AnyValue::new_string("resource_value"),
                )])
                .dropped_attributes_count(1u32),
            )
            .schema_url("https://schema.opentelemetry.io/resource_schema")
            .scope_logs(vec![
                ScopeLogs::build(
                    InstrumentationScope::build("library")
                        .version("scopev1")
                        .attributes(vec![KeyValue::new(
                            "scope_attr1",
                            AnyValue::new_string("scope_val1"),
                        )])
                        .dropped_attributes_count(2u32)
                        .finish(),
                )
                .schema_url("https://schema.opentelemetry.io/scope_schema")
                .log_records(vec![
                    LogRecord::build(2_000_000_000u64, SeverityNumber::Info, "event1")
                        .observed_time_unix_nano(3_000_000_000u64)
                        .trace_id(vec![0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3])
                        .span_id(vec![0, 0, 0, 0, 1, 1, 1, 1])
                        .severity_text("Info")
                        .attributes(vec![KeyValue::new(
                            "log_attr1",
                            AnyValue::new_string("log_val_1"),
                        )])
                        .dropped_attributes_count(3u32)
                        .flags(LogRecordFlags::TraceFlagsMask)
                        .body(AnyValue::new_string("log_body"))
                        .finish(),
                ])
                .finish(),
            ])
            .finish(),
        ])
    }

    fn _test_encode_logs_verify_all_columns_generic<T>(logs_data: T)
    where
        T: LogsDataView,
    {
        // verify that every column for each record batch gets encoded as the correct type
        let result = encode_logs_otap_batch(&logs_data);
        let otap_batch = result.unwrap();

        // check that the logs record batch is what we expect
        let expected_log_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::UInt16, true),
                Field::new(
                    "resource",
                    DataType::Struct(
                        vec![
                            Field::new("id", DataType::UInt16, true),
                            Field::new(
                                "schema_url",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                            Field::new("dropped_attributes_count", DataType::UInt32, true),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "scope",
                    DataType::Struct(
                        vec![
                            Field::new("id", DataType::UInt16, true),
                            Field::new(
                                "name",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                            Field::new(
                                "version",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                            Field::new("dropped_attributes_count", DataType::UInt32, true),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "schema_url",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    "time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "observed_time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "trace_id",
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(16)),
                    ),
                    true,
                ),
                Field::new(
                    "span_id",
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(8)),
                    ),
                    true,
                ),
                Field::new(
                    "severity_number",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int32)),
                    true,
                ),
                Field::new(
                    "severity_text",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    "body",
                    DataType::Struct(
                        vec![
                            Field::new("type", DataType::UInt8, true),
                            Field::new(
                                "str",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt16),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new("dropped_attributes_count", DataType::UInt32, false),
                Field::new("flags", DataType::UInt32, true),
            ])),
            vec![
                // id
                Arc::new(UInt16Array::from_iter(vec![Some(0)])),
                // resource
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // resource.id
                        Arc::new(UInt16Array::from(vec![0])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "schema_url",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // resource.schema_url
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0]),
                            Arc::new(StringArray::from_iter_values(vec![
                                "https://schema.opentelemetry.io/resource_schema",
                            ])),
                        )) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "dropped_attributes_count",
                            DataType::UInt32,
                            true,
                        )),
                        // resource.dropped_attributes.count
                        Arc::new(UInt32Array::from(vec![1])) as ArrayRef,
                    ),
                ])),
                // scope
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // scope.id
                        Arc::new(UInt16Array::from(vec![0])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "name",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // scope.name
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0]),
                            Arc::new(StringArray::from(vec!["library"])),
                        )) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "version",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // scope.version
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0]),
                            Arc::new(StringArray::from(vec!["scopev1"])),
                        )) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "dropped_attributes_count",
                            DataType::UInt32,
                            true,
                        )),
                        // scope.dropped_attributes.count
                        Arc::new(UInt32Array::from(vec![2])) as ArrayRef,
                    ),
                ])) as ArrayRef,
                // schema_url
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec![
                        "https://schema.opentelemetry.io/scope_schema",
                    ])),
                )),
                // timestamps
                Arc::new(TimestampNanosecondArray::from(vec![2_000_000_000])),
                // observed_time_unix_nano
                Arc::new(TimestampNanosecondArray::from(vec![3_000_000_000i64])) as ArrayRef,
                // trace_id
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(
                        FixedSizeBinaryArray::try_from_iter(
                            vec![vec![0, 0, 0, 0, 1, 1, 1, 1, 2, 2, 2, 2, 3, 3, 3, 3]].into_iter(),
                        )
                        .unwrap(),
                    ),
                )) as ArrayRef,
                // span_id
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(
                        FixedSizeBinaryArray::try_from_iter(
                            vec![vec![0, 0, 0, 0, 1, 1, 1, 1]].into_iter(),
                        )
                        .unwrap(),
                    ),
                )) as ArrayRef,
                // severity_number
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(Int32Array::from(vec![9])),
                )) as ArrayRef,
                // severity_text
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(StringArray::from(vec!["Info"])),
                )) as ArrayRef,
                // body
                Arc::new(StructArray::new(
                    vec![
                        Field::new("type", DataType::UInt8, true),
                        Field::new(
                            "str",
                            DataType::Dictionary(
                                Box::new(DataType::UInt16),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        ),
                    ]
                    .into(),
                    vec![
                        Arc::new(UInt8Array::from(vec![AttributeValueType::Str as u8])),
                        Arc::new(DictionaryArray::<UInt16Type>::new(
                            UInt16Array::from(vec![0]),
                            Arc::new(StringArray::from(vec!["log_body"])),
                        )) as ArrayRef,
                    ],
                    Some(NullBuffer::from_iter(vec![true])),
                )) as ArrayRef,
                // dropped_attributes_count
                Arc::new(UInt32Array::from(vec![3])) as ArrayRef,
                // flags
                Arc::new(UInt32Array::from(vec![
                    LogRecordFlags::TraceFlagsMask as u32,
                ])) as ArrayRef,
            ],
        )
        .unwrap();
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(logs_rb, &expected_log_batch);

        // check that the resource_attrs record batch is what we expect
        let expected_resource_attrs_log_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("parent_id", DataType::UInt16, false),
                Field::new(
                    "key",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new("type", DataType::UInt8, false),
                Field::new(
                    "str",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![0])),
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["resource_attr1"])),
                )),
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["resource_value"])),
                )),
            ],
        )
        .unwrap();
        let resource_attrs_batch = otap_batch.get(ArrowPayloadType::ResourceAttrs).unwrap();
        assert_eq!(resource_attrs_batch, &expected_resource_attrs_log_batch);

        // check that the scope_attrs record batch is what we expect
        let expected_scope_attrs_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("parent_id", DataType::UInt16, false),
                Field::new(
                    "key",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new("type", DataType::UInt8, false),
                Field::new(
                    "str",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![0])),
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["scope_attr1"])),
                )),
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["scope_val1"])),
                )),
            ],
        )
        .unwrap();
        let scope_attrs_batch = otap_batch.get(ArrowPayloadType::ScopeAttrs).unwrap();
        assert_eq!(scope_attrs_batch, &expected_scope_attrs_batch);

        // check that the log_attrs record batch is what we expect
        let expected_log_attrs_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("parent_id", DataType::UInt16, false),
                Field::new(
                    "key",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new("type", DataType::UInt8, false),
                Field::new(
                    "str",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![0])),
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["log_attr1"])),
                )),
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["log_val_1"])),
                )),
            ],
        )
        .unwrap();
        let log_attrs_batch = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        assert_eq!(log_attrs_batch, &expected_log_attrs_batch);
    }

    #[test]
    fn test_encode_logs_verify_all_columns_proto_struct() {
        let logs_data = _generate_logs_for_verify_all_columns();
        _test_encode_logs_verify_all_columns_generic(logs_data);
    }

    #[test]
    fn test_encode_logs_verify_all_columns_proto_bytes() {
        let logs_data = _generate_logs_for_verify_all_columns();
        let mut logs_data_bytes = vec![];
        logs_data.encode(&mut logs_data_bytes).unwrap();
        _test_encode_logs_verify_all_columns_generic(RawLogsData::new(&logs_data_bytes));
    }

    fn _generate_logs_for_verify_nullability() -> LogsData {
        // logs data with all empty/default fields
        LogsData::new(vec![ResourceLogs {
            resource: None,
            schema_url: "".to_string(),
            scope_logs: vec![ScopeLogs {
                scope: None,
                schema_url: "".to_string(),
                log_records: vec![LogRecord {
                    time_unix_nano: 0,
                    observed_time_unix_nano: 0,
                    severity_number: 0,
                    severity_text: "".to_string(),
                    body: None,
                    attributes: vec![],
                    dropped_attributes_count: 0,
                    flags: 0,
                    trace_id: vec![],
                    span_id: vec![],
                    event_name: "".to_string(),
                }],
            }],
        }])
    }

    fn _test_encode_logs_verify_nullability_generic<T>(logs_data: &T)
    where
        T: LogsDataView,
    {
        // check that every column handles nulls correctly through the correct strategy which for
        // various columns consists of one of the following:
        // - not being present in the record batch
        // - having nulls in the columns
        // - using default values

        let result = encode_logs_otap_batch(logs_data);
        let otap_batch = result.unwrap();

        let expected_logs_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(vec![Field::new(consts::ID, DataType::UInt16, true)].into()),
                    true,
                ),
                Field::new(
                    consts::SCOPE,
                    DataType::Struct(vec![Field::new(consts::ID, DataType::UInt16, true)].into()),
                    true,
                ),
                Field::new(
                    consts::TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    consts::OBSERVED_TIME_UNIX_NANO,
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
            ])),
            vec![
                Arc::new(StructArray::new(
                    vec![Field::new(consts::ID, DataType::UInt16, true)].into(),
                    vec![Arc::new(UInt16Array::from(vec![0]))],
                    None,
                )),
                Arc::new(StructArray::new(
                    vec![Field::new(consts::ID, DataType::UInt16, true)].into(),
                    vec![Arc::new(UInt16Array::from(vec![0]))],
                    None,
                )),
                Arc::new(TimestampNanosecondArray::from_iter_values(vec![0])),
                Arc::new(TimestampNanosecondArray::from_iter_values(vec![0])),
            ],
        )
        .unwrap();

        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(logs_rb, &expected_logs_batch);
    }

    #[test]
    fn test_encode_logs_verify_nullability_proto_struct() {
        let logs_data = _generate_logs_for_verify_nullability();
        _test_encode_logs_verify_nullability_generic(&logs_data);
    }

    #[test]
    fn test_encode_logs_verify_nullability_proto_bytes() {
        let logs_data = _generate_logs_for_verify_nullability();
        let mut logs_data_bytes = vec![];
        logs_data.encode(&mut logs_data_bytes).unwrap();
        _test_encode_logs_verify_nullability_generic(&RawLogsData::new(&logs_data_bytes));
    }

    fn _generate_logs_no_attributes() -> LogsData {
        LogsData::new(vec![
            ResourceLogs::build(Resource::new(vec![]))
                .schema_url("https://schema.opentelemetry.io/resource_schema")
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::new("scope"))
                        .log_records(vec![
                            LogRecord::build(2_000_000_000u64, SeverityNumber::Debug, "event")
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ])
    }

    fn _test_logs_logs_no_attributes<T>(logs_view: &T)
    where
        T: LogsDataView,
    {
        let otap_batch = encode_logs_otap_batch(logs_view).unwrap();

        let logs_batch = otap_batch.get(ArrowPayloadType::Logs).unwrap();

        let expected_logs_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(
                        vec![
                            Field::new(consts::ID, DataType::UInt16, true),
                            Field::new(
                                "schema_url",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "scope",
                    DataType::Struct(
                        vec![
                            Field::new("id", DataType::UInt16, true),
                            Field::new(
                                "name",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "observed_time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "severity_number",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int32)),
                    true,
                ),
            ])),
            vec![
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // resource.id
                        Arc::new(UInt16Array::from(vec![0])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "schema_url",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // resource.schema_url
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0]),
                            Arc::new(StringArray::from_iter_values(vec![
                                "https://schema.opentelemetry.io/resource_schema",
                            ])),
                        )) as ArrayRef,
                    ),
                ])),
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // scope.id
                        Arc::new(UInt16Array::from(vec![0])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "name",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // scope.name
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0]),
                            Arc::new(StringArray::from(vec!["scope"])),
                        )) as ArrayRef,
                    ),
                ])),
                // timestamps
                Arc::new(TimestampNanosecondArray::from(vec![2_000_000_000])),
                // observed_time_unix_nano
                Arc::new(TimestampNanosecondArray::from(vec![0i64])) as ArrayRef,
                // severity_number
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(Int32Array::from(vec![5])),
                )) as ArrayRef,
            ],
        )
        .unwrap();

        assert_eq!(logs_batch, &expected_logs_batch);

        assert!(otap_batch.get(ArrowPayloadType::ResourceAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::ScopeAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());
    }

    #[test]
    fn test_logs_no_attributes_proto_struct() {
        let logs_data = _generate_logs_no_attributes();
        _test_logs_logs_no_attributes(&logs_data);
    }

    #[test]
    fn test_logs_no_attributes_proto_bytes() {
        let logs_data = _generate_logs_no_attributes();
        let mut logs_data_bytes = vec![];
        logs_data.encode(&mut logs_data_bytes).unwrap();
        _test_logs_logs_no_attributes(&RawLogsData::new(&logs_data_bytes));
    }

    fn _generate_logs_multiple_logs_and_attrs() -> LogsData {
        LogsData::new(vec![
            ResourceLogs::build(Resource::new(vec![]))
                .schema_url("https://schema.opentelemetry.io/resource_schema")
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::new("scope"))
                        .log_records(vec![
                            LogRecord::build(0u64, SeverityNumber::Debug, "event")
                                .attributes(vec![
                                    KeyValue::new("key1", AnyValue::new_string("val1")),
                                    KeyValue::new("key2", AnyValue::new_string("val2")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                    ScopeLogs::build(InstrumentationScope::new("scope2"))
                        .log_records(vec![
                            LogRecord::build(0u64, SeverityNumber::Info, "event").finish(),
                        ])
                        .finish(),
                ])
                .finish(),
            ResourceLogs::build(Resource::new(vec![]))
                .schema_url("https://schema.opentelemetry.io/resource_schema")
                .scope_logs(vec![
                    ScopeLogs::build(InstrumentationScope::new("scope"))
                        .log_records(vec![
                            LogRecord::build(0u64, SeverityNumber::Debug, "event")
                                .attributes(vec![
                                    KeyValue::new("key1", AnyValue::new_string("val1")),
                                    KeyValue::new("key2", AnyValue::new_string("val2.b")),
                                ])
                                .finish(),
                        ])
                        .finish(),
                ])
                .finish(),
        ])
    }

    fn _test_logs_multiple_logs_and_attrs_generic<T>(logs_view: &T)
    where
        T: LogsDataView,
    {
        let otap_batch = encode_logs_otap_batch(logs_view).unwrap();

        let logs_batch = otap_batch.get(ArrowPayloadType::Logs).unwrap();

        let expected_logs_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::ID, DataType::UInt16, true),
                Field::new(
                    consts::RESOURCE,
                    DataType::Struct(
                        vec![
                            Field::new(consts::ID, DataType::UInt16, true),
                            Field::new(
                                "schema_url",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "scope",
                    DataType::Struct(
                        vec![
                            Field::new("id", DataType::UInt16, true),
                            Field::new(
                                "name",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "observed_time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "severity_number",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int32)),
                    true,
                ),
            ])),
            vec![
                // id
                Arc::new(UInt16Array::from_iter(vec![Some(0), None, Some(1)])),
                // resource
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // resource.id
                        Arc::new(UInt16Array::from(vec![0, 0, 1])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "schema_url",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // resource.schema_url
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0, 0, 0]),
                            Arc::new(StringArray::from_iter_values(vec![
                                "https://schema.opentelemetry.io/resource_schema",
                            ])),
                        )) as ArrayRef,
                    ),
                ])),
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // scope.id
                        Arc::new(UInt16Array::from(vec![0, 1, 2])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "name",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // scope.name
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0, 1, 0]),
                            Arc::new(StringArray::from(vec!["scope", "scope2"])),
                        )) as ArrayRef,
                    ),
                ])),
                // timestamps
                Arc::new(TimestampNanosecondArray::from(vec![0, 0, 0])),
                // observed_time_unix_nano
                Arc::new(TimestampNanosecondArray::from(vec![0i64, 0, 0])) as ArrayRef,
                // severity_number
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0, 1, 0]),
                    Arc::new(Int32Array::from(vec![5, 9, 5])),
                )) as ArrayRef,
            ],
        )
        .unwrap();

        assert_eq!(logs_batch, &expected_logs_batch);

        let log_attrs_batch = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();

        // check that the log_attrs record batch is what we expect
        let expected_log_attrs_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("parent_id", DataType::UInt16, false),
                Field::new(
                    "key",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new("type", DataType::UInt8, false),
                Field::new(
                    "str",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![0, 0, 1, 1])),
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(vec![0, 1, 0, 1]),
                    Arc::new(StringArray::from_iter_values(vec!["key1", "key2"])),
                )),
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter_values(vec![0, 1, 0, 2]),
                    Arc::new(StringArray::from_iter_values(vec![
                        "val1", "val2", "val2.b",
                    ])),
                )),
            ],
        )
        .unwrap();

        assert_eq!(log_attrs_batch, &expected_log_attrs_batch);

        assert!(otap_batch.get(ArrowPayloadType::ResourceAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::ScopeAttrs).is_none());
    }

    #[test]
    fn test_logs_multiple_logs_and_attrs_prost_structs() {
        let logs_data = _generate_logs_multiple_logs_and_attrs();
        _test_logs_multiple_logs_and_attrs_generic(&logs_data);
    }

    #[test]
    fn test_logs_multiple_logs_and_attrs_proto_bytes() {
        let logs_data = _generate_logs_multiple_logs_and_attrs();
        let mut logs_data_bytes = vec![];
        logs_data.encode(&mut logs_data_bytes).unwrap();
        _test_logs_multiple_logs_and_attrs_generic(&RawLogsData::new(&logs_data_bytes));
    }

    fn _generate_log_body_all_field_types_data() -> LogsData {
        let log_bodies = vec![
            AnyValue::new_string("terry"),
            AnyValue::new_bool(true),
            AnyValue::new_int(5),
            AnyValue::new_double(2.0),
            AnyValue::new_bytes(b"hi"),
            AnyValue {
                // test the empty value
                value: None,
            },
            AnyValue::new_array(vec![AnyValue::new_bool(true)]),
            AnyValue::new_kvlist(vec![KeyValue::new("key1", AnyValue::new_bool(true))]),
        ];

        let mut log_records = vec![
            // log with empty body
            LogRecord::build(5u64, SeverityNumber::Info, "event").finish(),
        ];
        log_records.append(
            &mut log_bodies
                .clone()
                .into_iter()
                .map(|body| {
                    LogRecord::build(5u64, SeverityNumber::Info, "event")
                        .body(body)
                        .finish()
                })
                .collect::<Vec<_>>(),
        );
        LogsData::new(vec![ResourceLogs {
            resource: None,
            schema_url: "".to_string(),
            scope_logs: vec![ScopeLogs {
                scope: None,
                schema_url: "".to_string(),
                log_records,
            }],
        }])
    }

    fn _test_encode_logs_body_all_field_types_generic<T>(logs_data: &T)
    where
        T: LogsDataView,
    {
        // check that all the field types allowed for a body are able to be encoded
        let result = encode_logs_otap_batch(logs_data);
        assert!(result.is_ok());

        let otap_batch = result.unwrap();
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let body_column = logs_rb
            .column_by_name(consts::BODY)
            .unwrap()
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();

        // if the generated test data changes, the values can be found by rerunning the test
        // and inspecting the test failures
        let expected_serialized_array = vec![159, 245, 255];
        let expected_serialized_kvs = vec![191, 100, 107, 101, 121, 49, 245, 255];

        let expected_body = StructArray::try_new(
            vec![
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, true),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_INT,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(
                    consts::ATTRIBUTE_BYTES,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_SER,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
            ]
            .into(),
            vec![
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Str as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Map as u8,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter(vec![
                        None,
                        Some(0),
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from(vec![Some("terry")])),
                )),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter(vec![
                        None,
                        None,
                        None,
                        Some(0),
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(Int64Array::from(vec![Some(5)])),
                )),
                Arc::new(Float64Array::from(vec![
                    None,
                    None,
                    None,
                    None,
                    Some(2.0),
                    None,
                    None,
                    None,
                    None,
                ])),
                Arc::new(BooleanArray::from(vec![
                    None,
                    None,
                    Some(true),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter(vec![
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(BinaryArray::from(vec![Some(b"hi".to_vec().as_slice())])),
                )),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter(vec![
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(0),
                        Some(1),
                    ]),
                    Arc::new(BinaryArray::from(vec![
                        Some(expected_serialized_array.as_slice()),
                        Some(expected_serialized_kvs.as_slice()),
                    ])),
                )),
            ],
            Some(NullBuffer::from(vec![
                false, true, true, true, true, true, false, true, true,
            ])),
        )
        .unwrap();

        assert_eq!(body_column, &expected_body);

        assert!(otap_batch.get(ArrowPayloadType::ResourceAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::ScopeAttrs).is_none());
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());

        // check the serialized values are what is expected
        let deserialized_array = decode_pcommon_val(&expected_serialized_array).unwrap();
        assert_eq!(
            deserialized_array,
            Some(any_value::Value::ArrayValue(ArrayValue {
                values: vec![AnyValue::new_bool(true)]
            }))
        );

        let deserialized_kvs = decode_pcommon_val(&expected_serialized_kvs).unwrap();
        assert_eq!(
            deserialized_kvs,
            Some(any_value::Value::KvlistValue(KeyValueList {
                values: vec![KeyValue::new("key1", AnyValue::new_bool(true))]
            }))
        );
    }

    #[test]
    fn test_encode_logs_body_all_field_types_proto_struct() {
        let logs_data = _generate_log_body_all_field_types_data();
        _test_encode_logs_body_all_field_types_generic(&logs_data);
    }

    #[test]
    fn test_encode_logs_body_all_fields_proto_bytes() {
        let logs_data = _generate_log_body_all_field_types_data();
        let mut logs_data_bytes = vec![];
        logs_data.encode(&mut logs_data_bytes).unwrap();
        _test_encode_logs_body_all_field_types_generic(&RawLogsData::new(&logs_data_bytes));
    }

    fn _generate_test_data_all_field_types() -> LogsData {
        let attr_values = vec![
            AnyValue::new_string("terry"),
            AnyValue::new_bool(true),
            AnyValue::new_int(5),
            AnyValue::new_double(2.0),
            AnyValue::new_bytes(b"hi"),
            AnyValue { value: None },
            AnyValue::new_array(vec![AnyValue::new_bool(true)]),
            AnyValue::new_kvlist(vec![KeyValue::new("key1", AnyValue::new_bool(true))]),
        ];
        let mut attributes = attr_values
            .into_iter()
            .enumerate()
            .map(|(i, val)| KeyValue {
                key: format!("{i:?}"),
                value: Some(val),
            })
            .collect::<Vec<_>>();

        // test none value
        attributes.push(KeyValue {
            key: "noneval".to_string(),
            value: None,
        });

        LogsData::new(vec![ResourceLogs {
            resource: None,
            schema_url: "".to_string(),
            scope_logs: vec![ScopeLogs {
                scope: None,
                schema_url: "".to_string(),
                log_records: vec![
                    LogRecord::build(0u64, SeverityNumber::Info, "")
                        .attributes(attributes)
                        .finish(),
                ],
            }],
        }])
    }

    fn _test_attributes_all_field_types_generic<T>(logs_data: T)
    where
        T: LogsDataView,
    {
        let result = encode_logs_otap_batch(&logs_data);
        assert!(result.is_ok());

        let otap_batch = result.unwrap();
        let logs_attrs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();

        // if the generated test data changes, the values can be found by rerunning the test
        // and inspecting the test failures
        let expected_serialized_array = vec![159, 245, 255];
        let expected_serialized_kvs = vec![191, 100, 107, 101, 121, 49, 245, 255];

        let expected_attrs = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(consts::PARENT_ID, DataType::UInt16, false),
                Field::new(
                    consts::ATTRIBUTE_KEY,
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
                Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_INT,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64)),
                    true,
                ),
                Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
                Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
                Field::new(
                    consts::ATTRIBUTE_BYTES,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
                Field::new(
                    consts::ATTRIBUTE_SER,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Binary)),
                    true,
                ),
            ])),
            vec![
                Arc::new(UInt16Array::from_iter_values(vec![
                    0, 0, 0, 0, 0, 0, 0, 0, 0,
                ])),
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(0..9),
                    Arc::new(StringArray::from(vec![
                        Some("0"),
                        Some("1"),
                        Some("2"),
                        Some("3"),
                        Some("4"),
                        Some("5"),
                        Some("6"),
                        Some("7"),
                        Some("noneval"),
                    ])),
                )),
                Arc::new(UInt8Array::from_iter_values([
                    AttributeValueType::Str as u8,
                    AttributeValueType::Bool as u8,
                    AttributeValueType::Int as u8,
                    AttributeValueType::Double as u8,
                    AttributeValueType::Bytes as u8,
                    AttributeValueType::Empty as u8,
                    AttributeValueType::Slice as u8,
                    AttributeValueType::Map as u8,
                    AttributeValueType::Empty as u8,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter([
                        Some(0), // "terry"
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(StringArray::from(vec![Some("terry")])),
                )),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        Some(0), // 5
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(Int64Array::from(vec![Some(5)])),
                )),
                Arc::new(Float64Array::from(vec![
                    None,
                    None,
                    None,
                    Some(2.0),
                    None,
                    None,
                    None,
                    None,
                    None,
                ])),
                Arc::new(BooleanArray::from(vec![
                    None,
                    Some(true),
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                    None,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter([
                        None,
                        None,
                        None,
                        None,
                        Some(0), // b"hi"
                        None,
                        None,
                        None,
                        None,
                    ]),
                    Arc::new(BinaryArray::from(vec![Some(b"hi".to_vec().as_slice())])),
                )),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter(vec![
                        None,
                        None,
                        None,
                        None,
                        None,
                        None,
                        Some(0), // expected slice
                        Some(1), // expected map
                        None,
                    ]),
                    Arc::new(BinaryArray::from(vec![
                        Some(expected_serialized_array.as_slice()),
                        Some(expected_serialized_kvs.as_slice()),
                    ])),
                )),
            ],
        )
        .unwrap();

        assert_eq!(logs_attrs, &expected_attrs);

        // check the serialized values are what is expected
        let deserialized_array = decode_pcommon_val(&expected_serialized_array).unwrap();
        assert_eq!(
            deserialized_array,
            Some(any_value::Value::ArrayValue(ArrayValue {
                values: vec![AnyValue::new_bool(true)]
            }))
        );

        let deserialized_kvs = decode_pcommon_val(&expected_serialized_kvs).unwrap();
        assert_eq!(
            deserialized_kvs,
            Some(any_value::Value::KvlistValue(KeyValueList {
                values: vec![KeyValue::new("key1", AnyValue::new_bool(true))]
            }))
        );
    }

    #[test]
    fn test_attributes_all_field_types_proto_struct() {
        let logs_data = _generate_test_data_all_field_types();
        _test_attributes_all_field_types_generic(logs_data)
    }

    #[test]
    fn test_attributes_all_field_types_proto_bytes() {
        let logs_data = _generate_test_data_all_field_types();
        let mut logs_data_bytes = vec![];
        logs_data.encode(&mut logs_data_bytes).unwrap();
        _test_attributes_all_field_types_generic(RawLogsData::new(&logs_data_bytes));
    }

    #[test]
    fn test_spans_proto_struct() {
        use otel_arrow_rust::encode::record::spans::{SpanId, TraceId};
        use otel_arrow_rust::proto::opentelemetry::trace::v1::*;

        let a_trace_id: TraceId = [0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15];
        let a_span_id: SpanId = [17, 18, 19, 20, 21, 22, 23, 24];
        let a_parent_span_id: SpanId = [27, 28, 19, 20, 21, 22, 23, 24];
        let traces_data = TracesData::new(vec![
            ResourceSpans::build(
                Resource::build(vec![KeyValue::new(
                    "attr1",
                    AnyValue::new_string("some_value"),
                )])
                .dropped_attributes_count(123u32),
            )
            .schema_url("https://schema.opentelemetry.io/resource_schema")
            .scope_spans(vec![
                ScopeSpans::build(
                    InstrumentationScope::build("library")
                        .version("scopev1")
                        .attributes(vec![KeyValue::new(
                            "scope_attr1",
                            AnyValue::new_string("scope_val1"),
                        )])
                        .dropped_attributes_count(17u32)
                        .finish(),
                )
                .schema_url("https://schema.opentelemetry.io/scope_schema")
                .spans(vec![
                    Span::build(
                        a_trace_id.to_vec(),
                        a_span_id.to_vec(),
                        "span_name_1",
                        999u64,
                    )
                    .trace_state("some_state")
                    .end_time_unix_nano(1999u64)
                    .parent_span_id(a_parent_span_id.to_vec())
                    .dropped_attributes_count(7u32)
                    .dropped_events_count(11u32)
                    .dropped_links_count(29u32)
                    .kind(span::SpanKind::Consumer)
                    .status(Status::new("something happened", status::StatusCode::Error))
                    .events(vec![
                        span::Event::build("an_event", 456u64)
                            .attributes(vec![KeyValue::new(
                                "event_attr1",
                                AnyValue::new_string("hi"),
                            )])
                            .dropped_attributes_count(12345u32)
                            .finish(),
                    ])
                    .links(vec![
                        span::Link::build(a_trace_id.to_vec(), a_span_id.to_vec())
                            .trace_state("some link state")
                            .dropped_attributes_count(567u32)
                            .flags(7u32)
                            .attributes(vec![KeyValue::new(
                                "link_attr1",
                                AnyValue::new_string("hello"),
                            )])
                            .finish(),
                    ])
                    .finish(),
                ])
                .finish(),
            ])
            .finish(),
        ]);

        let otap_batch = encode_spans_otap_batch(&traces_data).unwrap();

        let expected_span_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::UInt16, true),
                Field::new(
                    "resource",
                    DataType::Struct(
                        vec![
                            Field::new("id", DataType::UInt16, true),
                            Field::new(
                                "schema_url",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                            Field::new("dropped_attributes_count", DataType::UInt32, true),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "scope",
                    DataType::Struct(
                        vec![
                            Field::new("id", DataType::UInt16, true),
                            Field::new(
                                "name",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                            Field::new(
                                "version",
                                DataType::Dictionary(
                                    Box::new(DataType::UInt8),
                                    Box::new(DataType::Utf8),
                                ),
                                true,
                            ),
                            Field::new("dropped_attributes_count", DataType::UInt32, true),
                        ]
                        .into(),
                    ),
                    true,
                ),
                Field::new(
                    "start_time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    false,
                ),
                Field::new(
                    "duration_time_unix_nano",
                    DataType::Duration(TimeUnit::Nanosecond),
                    false,
                ),
                Field::new(
                    "trace_id",
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(16)),
                    ),
                    false,
                ),
                Field::new(
                    "span_id",
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(8)),
                    ),
                    false,
                ),
                Field::new(
                    "trace_state",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new(
                    "parent_span_id",
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(8)),
                    ),
                    true,
                ),
                Field::new(
                    "name",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new(
                    "kind",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int32)),
                    true,
                ),
                Field::new("dropped_attributes_count", DataType::UInt32, true),
                Field::new("dropped_events_count", DataType::UInt32, true),
                Field::new("dropped_links_count", DataType::UInt32, true),
                Field::new(
                    "code",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Int32)),
                    true,
                ),
                Field::new(
                    "status_message",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                // id
                Arc::new(UInt16Array::from_iter(vec![Some(0)])),
                // resource
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // resource.id
                        Arc::new(UInt16Array::from(vec![0])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "schema_url",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // resource.schema_url
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0]),
                            Arc::new(StringArray::from_iter_values(vec![
                                "https://schema.opentelemetry.io/resource_schema",
                            ])),
                        )) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "dropped_attributes_count",
                            DataType::UInt32,
                            true,
                        )),
                        // resource.dropped_attributes.count
                        Arc::new(UInt32Array::from(vec![123])) as ArrayRef,
                    ),
                ])),
                // scope
                Arc::new(StructArray::from(vec![
                    (
                        Arc::new(Field::new("id", DataType::UInt16, true)),
                        // scope.id
                        Arc::new(UInt16Array::from(vec![0])) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "name",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // scope.name
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0]),
                            Arc::new(StringArray::from(vec!["library"])),
                        )) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "version",
                            DataType::Dictionary(
                                Box::new(DataType::UInt8),
                                Box::new(DataType::Utf8),
                            ),
                            true,
                        )),
                        // scope.version
                        Arc::new(DictionaryArray::<UInt8Type>::new(
                            UInt8Array::from(vec![0]),
                            Arc::new(StringArray::from(vec!["scopev1"])),
                        )) as ArrayRef,
                    ),
                    (
                        Arc::new(Field::new(
                            "dropped_attributes_count",
                            DataType::UInt32,
                            true,
                        )),
                        // scope.dropped_attributes.count
                        Arc::new(UInt32Array::from(vec![17])) as ArrayRef,
                    ),
                ])) as ArrayRef,
                // timestamps
                Arc::new(TimestampNanosecondArray::from(vec![999])),
                Arc::new(DurationNanosecondArray::from(vec![1000])),
                // trace_id
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(
                        FixedSizeBinaryArray::try_from_iter(vec![a_trace_id.to_vec()].into_iter())
                            .unwrap(),
                    ),
                )) as ArrayRef,
                // span_id
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(
                        FixedSizeBinaryArray::try_from_iter(vec![a_span_id.to_vec()].into_iter())
                            .unwrap(),
                    ),
                )) as ArrayRef,
                // trace_state
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(StringArray::from(vec!["some_state"])),
                )) as ArrayRef,
                // parent_span_id
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(
                        FixedSizeBinaryArray::try_from_iter(
                            vec![a_parent_span_id.to_vec()].into_iter(),
                        )
                        .unwrap(),
                    ),
                )) as ArrayRef,
                // name
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(StringArray::from(vec!["span_name_1"])),
                )) as ArrayRef,
                // kind
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(Int32Array::from(vec![span::SpanKind::Consumer as i32])),
                )),
                // dropped_attributes_count
                Arc::new(UInt32Array::from(vec![7])) as ArrayRef,
                // dropped_events_count
                Arc::new(UInt32Array::from(vec![11])) as ArrayRef,
                // dropped_links_count
                Arc::new(UInt32Array::from(vec![29])) as ArrayRef,
                // status code
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(Int32Array::from(vec![status::StatusCode::Error as i32])),
                )),
                // status message
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(StringArray::from(vec!["something happened"])),
                )),
            ],
        )
        .unwrap();
        let spans_rb = otap_batch.get(ArrowPayloadType::Spans).unwrap();
        compare_record_batches(spans_rb, &expected_span_batch);
        assert_eq!(spans_rb, &expected_span_batch);

        let expected_events_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::UInt32, true),
                Field::new("parent_id", DataType::UInt16, false),
                Field::new(
                    "time_unix_nano",
                    DataType::Timestamp(TimeUnit::Nanosecond, None),
                    true,
                ),
                Field::new(
                    "name",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new("dropped_attributes_count", DataType::UInt32, true),
            ])),
            vec![
                // id
                Arc::new(UInt32Array::from_iter(vec![Some(0)])),
                // parent_id
                Arc::new(UInt16Array::from_iter(vec![0])),
                // time_unix_nano
                Arc::new(TimestampNanosecondArray::from(vec![456])),
                // name
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(StringArray::from(vec!["an_event"])),
                )) as ArrayRef,
                // dropped_attributes_count
                Arc::new(UInt32Array::from(vec![12345])) as ArrayRef,
            ],
        )
        .unwrap();
        let events_rb = otap_batch.get(ArrowPayloadType::SpanEvents).unwrap();
        compare_record_batches(events_rb, &expected_events_batch);
        assert_eq!(events_rb, &expected_events_batch);

        let expected_links_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new("id", DataType::UInt32, true),
                Field::new("parent_id", DataType::UInt16, false),
                Field::new(
                    "trace_id",
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(16)),
                    ),
                    true,
                ),
                Field::new(
                    "span_id",
                    DataType::Dictionary(
                        Box::new(DataType::UInt8),
                        Box::new(DataType::FixedSizeBinary(8)),
                    ),
                    true,
                ),
                Field::new(
                    "trace_state",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    true,
                ),
                Field::new("dropped_attributes_count", DataType::UInt32, true),
            ])),
            vec![
                // id
                Arc::new(UInt32Array::from_iter(vec![Some(0)])),
                // parent_id
                Arc::new(UInt16Array::from_iter(vec![0])),
                // trace_id
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(
                        FixedSizeBinaryArray::try_from_iter(vec![a_trace_id.to_vec()].into_iter())
                            .unwrap(),
                    ),
                )) as ArrayRef,
                // span_id
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(
                        FixedSizeBinaryArray::try_from_iter(vec![a_span_id.to_vec()].into_iter())
                            .unwrap(),
                    ),
                )) as ArrayRef,
                // trace_state
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(vec![0]),
                    Arc::new(StringArray::from(vec!["some link state"])),
                )) as ArrayRef,
                // dropped_attributes_count
                Arc::new(UInt32Array::from(vec![567])) as ArrayRef,
            ],
        )
        .unwrap();
        let links_rb = otap_batch.get(ArrowPayloadType::SpanLinks).unwrap();
        compare_record_batches(links_rb, &expected_links_batch);
        assert_eq!(links_rb, &expected_links_batch);

        let expected_event_attrs_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    "parent_id",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                    false,
                ),
                Field::new(
                    "key",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new("type", DataType::UInt8, false),
                Field::new(
                    "str",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(UInt32Array::from_iter_values(vec![0])),
                )),
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["event_attr1"])),
                )),
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["hi"])),
                )),
            ],
        )
        .unwrap();
        let event_attrs_rb = otap_batch.get(ArrowPayloadType::SpanEventAttrs).unwrap();
        compare_record_batches(event_attrs_rb, &expected_event_attrs_batch);
        assert_eq!(event_attrs_rb, &expected_event_attrs_batch);

        let expected_link_attrs_batch = RecordBatch::try_new(
            Arc::new(Schema::new(vec![
                Field::new(
                    "parent_id",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::UInt32)),
                    false,
                ),
                Field::new(
                    "key",
                    DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                    false,
                ),
                Field::new("type", DataType::UInt8, false),
                Field::new(
                    "str",
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ),
            ])),
            vec![
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(UInt32Array::from_iter_values(vec![0])),
                )),
                Arc::new(DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["link_attr1"])),
                )),
                Arc::new(UInt8Array::from_iter_values(vec![
                    AttributeValueType::Str as u8,
                ])),
                Arc::new(DictionaryArray::<UInt16Type>::new(
                    UInt16Array::from_iter_values(vec![0]),
                    Arc::new(StringArray::from_iter_values(vec!["hello"])),
                )),
            ],
        )
        .unwrap();
        let link_attrs_rb = otap_batch.get(ArrowPayloadType::SpanLinkAttrs).unwrap();
        compare_record_batches(link_attrs_rb, &expected_link_attrs_batch);
        assert_eq!(link_attrs_rb, &expected_link_attrs_batch);
    }

    /// I'm a small helper function for examining differences between expected and under-test
    /// `RecordBatch`es. For large `RecordBatch`es, I produce debug output that's much simpler to
    /// understand than the results of an `assert_eq!` failure.
    fn compare_record_batches(a: &RecordBatch, b: &RecordBatch) {
        //  Ideally we could use something like
        //  https://docs.rs/datafusion/48.0.1/datafusion/macro.assert_batches_eq.html but right now
        //  it doesn't support the nested types we rely on, so this hack will have to suffice.
        if a == b {
            return;
        }

        let a_schema = a.schema();
        let b_schema = b.schema();
        let a_names: Vec<_> = a_schema
            .fields()
            .into_iter()
            .map(|field| field.name())
            .collect();
        let b_names: Vec<_> = b_schema
            .fields()
            .into_iter()
            .map(|field| field.name())
            .collect();
        assert_eq!(a_names, b_names);

        for field in a.schema().fields() {
            let col_name = field.name();
            let a_field = a_schema.field_with_name(col_name).unwrap();
            let b_field = b_schema.field_with_name(col_name).unwrap();
            if a_field != b_field {
                dbg!(col_name, a_field, b_field);
            }

            let a_col = format!("{:?}", a.column_by_name(col_name).unwrap());
            let b_col = format!("{:?}", b.column_by_name(col_name).unwrap());
            if a_col != b_col {
                dbg!(col_name, a_col, b_col);
            }
        }
    }
}
