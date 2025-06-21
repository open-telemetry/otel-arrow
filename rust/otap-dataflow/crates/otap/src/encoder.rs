// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otap_df_pdata_views::views::{
    common::{AnyValueView, AttributeView, InstrumentationScopeView, ValueType},
    logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView},
    resource::ResourceView,
};
use otel_arrow_rust::{
    encode::record::{
        attributes::{AttributesRecordBatchBuilder, AttributesRecordBatchBuilderConstructorHelper},
        logs::LogsRecordBatchBuilder,
    },
    otap::{Logs, OtapBatch},
    otlp::attributes::parent_id::ParentId,
    proto::opentelemetry::arrow::v1::ArrowPayloadType,
};

use crate::encoder::error::Result;

mod cbor;
mod error;

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
        if let Some(resource) = resource_logs.resource() {
            for kv in resource.attributes() {
                resource_attrs.append_parent_id(&curr_resource_id);
                append_attribute_value(&mut resource_attrs, &kv)?;
            }
        }

        for scope_logs in resource_logs.scopes() {
            if let Some(scope) = scope_logs.scope() {
                for kv in scope.attributes() {
                    scope_attrs.append_parent_id(&curr_scope_id);
                    append_attribute_value(&mut scope_attrs, &kv)?;
                }
            }

            for log_record in scope_logs.log_records() {
                // set the resource
                logs.resource.append_id(Some(curr_resource_id));
                logs.resource
                    .append_schema_url(resource_logs.schema_url().as_deref());
                logs.resource.append_dropped_attributes_count(
                    resource_logs
                        .resource()
                        .map(|r| r.dropped_attributes_count())
                        .unwrap_or(0),
                );

                // set the scope
                logs.scope.append_id(Some(curr_scope_id));
                if let Some(scope) = scope_logs.scope() {
                    logs.scope.append_name(scope.name().as_deref());
                    logs.scope.append_version(scope.version().as_deref());
                    logs.scope
                        .append_dropped_attributes_count(scope.dropped_attributes_count());
                } else {
                    logs.scope.append_name(None);
                    logs.scope.append_version(None);
                    logs.scope.append_dropped_attributes_count(0);
                }

                logs.append_time_unix_nano(log_record.time_unix_nano().map(|v| v as i64));
                logs.append_observed_time_unix_nano(
                    log_record.observed_time_unix_nano().map(|v| v as i64),
                );
                logs.append_schema_url(scope_logs.schema_url().as_deref());
                logs.append_severity_number(log_record.severity_number());
                logs.append_severity_text(log_record.severity_text().as_deref());
                logs.append_dropped_attributes_count(log_record.dropped_attributes_count());
                logs.append_flags(log_record.flags());
                logs.append_trace_id(log_record.trace_id())?;
                logs.append_span_id(log_record.span_id())?;

                if let Some(body) = log_record.body() {
                    match body.value_type() {
                        ValueType::String => {
                            logs.body
                                .append_str(&body.as_string().expect("body to be string"));
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
                            logs.body.appned_slice(&serialized_value);
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

                let mut log_attrs_count = 0;
                for kv in log_record.attributes() {
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
            curr_scope_id += 1;
        }
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
    attribute_rb_builder.append_key(&key);

    if let Some(val) = kv.value() {
        match val.value_type() {
            ValueType::String => {
                attribute_rb_builder.append_str(&val.as_string().expect("value to be string"));
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
        ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray, Float64Array,
        Int32Array, Int64Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray,
        UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::buffer::NullBuffer;
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit, UInt8Type, UInt16Type};
    use otel_arrow_rust::otlp::attributes::store::AttributeValueType;
    use otel_arrow_rust::proto::opentelemetry::common::v1::{
        AnyValue, InstrumentationScope, KeyValue,
    };
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{
        LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
    };
    use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
    use otel_arrow_rust::schema::consts;

    #[test]
    fn test_encode_logs_verify_all_columns() {
        // verify that every column for each record batch gets encoded as the correct type
        let logs_data = LogsData::new(vec![
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
        ]);

        let result = encode_logs_otap_batch(&logs_data);
        assert!(result.is_ok());
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
    fn test_encode_logs_verify_nullability() {
        // check that every column handles nulls correctly through the correct strategy which for
        // various columns consists of one of the following:
        // - not being present in the record batch
        // - having nulls in the columns
        // - using default values

        // logs data with all empty/default fields
        let logs_data = LogsData::new(vec![ResourceLogs {
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
        }]);
        let result = encode_logs_otap_batch(&logs_data);
        assert!(result.is_ok());
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
    fn test_encode_logs_body_all_field_types() {
        // check that all the field types allowed for a body are able to be encoded
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
        let logs_data = LogsData::new(vec![ResourceLogs {
            resource: None,
            schema_url: "".to_string(),
            scope_logs: vec![ScopeLogs {
                scope: None,
                schema_url: "".to_string(),
                log_records,
            }],
        }]);

        let result = encode_logs_otap_batch(&logs_data);
        assert!(result.is_ok());

        let otap_batch = result.unwrap();
        let logs_rb = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let body_column = logs_rb
            .column_by_name(consts::BODY)
            .unwrap()
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap();

        let mut expected_serialized_array = vec![];
        ciborium::into_writer(
            &ciborium::Value::Array(vec![ciborium::Value::Bool(true)]),
            &mut expected_serialized_array,
        )
        .unwrap();

        let mut expected_serialized_kvs = vec![];
        ciborium::into_writer(
            &ciborium::Value::Map(vec![(
                ciborium::Value::Text("key1".to_string()),
                ciborium::Value::Bool(true),
            )]),
            &mut expected_serialized_kvs,
        )
        .unwrap();

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

        assert_eq!(body_column, &expected_body)
    }

    #[test]
    fn test_attributes_all_field_types() {
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
                key: format!("{:?}", i),
                value: Some(val),
            })
            .collect::<Vec<_>>();

        // test none value
        attributes.push(KeyValue {
            key: "noneval".to_string(),
            value: None,
        });

        let logs_data = LogsData::new(vec![ResourceLogs {
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
        }]);

        let result = encode_logs_otap_batch(&logs_data);
        assert!(result.is_ok());

        let otap_batch = result.unwrap();
        let logs_attrs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();

        let mut expected_serialized_array = vec![];
        ciborium::into_writer(
            &ciborium::Value::Array(vec![ciborium::Value::Bool(true)]),
            &mut expected_serialized_array,
        )
        .unwrap();

        let mut expected_serialized_kvs = vec![];
        ciborium::into_writer(
            &ciborium::Value::Map(vec![(
                ciborium::Value::Text("key1".to_string()),
                ciborium::Value::Bool(true),
            )]),
            &mut expected_serialized_kvs,
        )
        .unwrap();

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
    }
}
