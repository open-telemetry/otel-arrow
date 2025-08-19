// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use data_engine_expressions::*;

use crate::{serializer::protobuf_writer::ProtobufWriter, *};

pub fn write_export_logs_service_request(
    export_logs_service_request: &ExportLogsServiceRequest,
    initial_capacity: usize,
) -> Result<Vec<u8>, SerializerError> {
    let mut writer = ProtobufWriter::new(initial_capacity);

    for resource_logs in &export_logs_service_request.resource_logs {
        writer.write_message_field(1, |writer| write_resource_logs(writer, resource_logs))?;
    }

    Ok(writer.freeze().to_vec())
}

fn write_resource_logs(
    writer: &mut ProtobufWriter,
    resource_logs: &ResourceLogs,
) -> Result<(), SerializerError> {
    if let Some(resource) = &resource_logs.resource {
        writer.write_message_field(1, |writer| write_resource(writer, resource))?;
    }

    for scope_logs in &resource_logs.scope_logs {
        writer.write_message_field(2, |writer| write_scope_logs(writer, scope_logs))?;
    }

    for field in &resource_logs.extra_fields {
        writer.write_field(field);
    }

    Ok(())
}

fn write_scope_logs(
    writer: &mut ProtobufWriter,
    scope_logs: &ScopeLogs,
) -> Result<(), SerializerError> {
    if let Some(scope) = &scope_logs.instrumentation_scope {
        writer.write_message_field(1, |writer| write_instrumentation_scope(writer, scope))?;
    }

    for log_record in &scope_logs.log_records {
        writer.write_message_field(2, |writer| write_log_record(writer, log_record))?;
    }

    for field in &scope_logs.extra_fields {
        writer.write_field(field);
    }

    Ok(())
}

fn write_resource(writer: &mut ProtobufWriter, resource: &Resource) -> Result<(), SerializerError> {
    let attributes = resource.attributes.get_values();
    if !attributes.is_empty() {
        if cfg!(test) {
            // Note: When building tests we sort the key list so that it is
            // deterministice.
            let mut values: Vec<(&Box<str>, &AnyValue)> = attributes.iter().collect();
            values.sort_by(|left, right| left.0.cmp(right.0));
            for (key, value) in values {
                writer.write_message_field(1, |writer| write_key_value(writer, key, value))?;
            }
        } else {
            for (key, value) in attributes {
                writer.write_message_field(1, |writer| write_key_value(writer, key, value))?;
            }
        }
    }

    for field in &resource.extra_fields {
        writer.write_field(field);
    }

    Ok(())
}

fn write_instrumentation_scope(
    writer: &mut ProtobufWriter,
    instrumentation_scope: &InstrumentationScope,
) -> Result<(), SerializerError> {
    if let Some(name) = &instrumentation_scope.name {
        writer.write_string_field(1, name.get_raw_value());
    }

    if let Some(version) = &instrumentation_scope.version {
        writer.write_string_field(2, version.get_raw_value());
    }

    let attributes = instrumentation_scope.attributes.get_values();
    if !attributes.is_empty() {
        if cfg!(test) {
            // Note: When building tests we sort the key list so that it is
            // deterministice.
            let mut values: Vec<(&Box<str>, &AnyValue)> = attributes.iter().collect();
            values.sort_by(|left, right| left.0.cmp(right.0));
            for (key, value) in values {
                writer.write_message_field(3, |writer| write_key_value(writer, key, value))?;
            }
        } else {
            for (key, value) in attributes {
                writer.write_message_field(3, |writer| write_key_value(writer, key, value))?;
            }
        }
    }

    for field in &instrumentation_scope.extra_fields {
        writer.write_field(field);
    }

    Ok(())
}

fn write_log_record(
    writer: &mut ProtobufWriter,
    log_record: &LogRecord,
) -> Result<(), SerializerError> {
    if let Some(Some(unix_nano)) = log_record
        .timestamp
        .as_ref()
        .map(|v| v.get_raw_value().timestamp_nanos_opt())
    {
        writer.write_fixed64_field(1, unix_nano as u64);
    }

    if let Some(severity_number) = &log_record.severity_number {
        writer.write_int32_field(2, *severity_number.get_raw_value());
    }

    if let Some(severity_text) = &log_record.severity_text {
        writer.write_string_field(3, severity_text.get_raw_value());
    }

    if let Some(body) = &log_record.body {
        writer.write_message_field(5, |writer| write_value(writer, body))?;
    }

    let attributes = log_record.attributes.get_values();
    if !attributes.is_empty() {
        if cfg!(test) {
            // Note: When building tests we sort the key list so that it is
            // deterministice.
            let mut values: Vec<(&Box<str>, &AnyValue)> = attributes.iter().collect();
            values.sort_by(|left, right| left.0.cmp(right.0));
            for (key, value) in values {
                writer.write_message_field(6, |writer| write_key_value(writer, key, value))?;
            }
        } else {
            for (key, value) in attributes {
                writer.write_message_field(6, |writer| write_key_value(writer, key, value))?;
            }
        }
    }

    if let Some(flags) = &log_record.flags {
        writer.write_fixed32_field(8, *flags.get_raw_value());
    }

    if let Some(trace_id) = &log_record.trace_id {
        writer.write_byte_array_field(9, trace_id);
    }

    if let Some(span_id) = &log_record.span_id {
        writer.write_byte_array_field(10, span_id);
    }

    if let Some(Some(unix_nano)) = log_record
        .observed_timestamp
        .as_ref()
        .map(|v| v.get_raw_value().timestamp_nanos_opt())
    {
        writer.write_fixed64_field(11, unix_nano as u64);
    }

    if let Some(event_name) = &log_record.event_name {
        writer.write_string_field(12, event_name.get_raw_value());
    }

    for field in &log_record.extra_fields {
        writer.write_field(field);
    }

    Ok(())
}

fn write_key_value(
    writer: &mut ProtobufWriter,
    key: &str,
    value: &AnyValue,
) -> Result<(), SerializerError> {
    writer.write_string_field(1, key);

    if matches!(value, AnyValue::Null) {
        Ok(())
    } else {
        writer.write_message_field(2, |writer| write_value(writer, value))
    }
}

fn write_value(writer: &mut ProtobufWriter, value: &AnyValue) -> Result<(), SerializerError> {
    match value {
        AnyValue::Null => {}
        AnyValue::Native(a) => write_native_value(writer, a)?,
        AnyValue::Extended(_) => {
            let value = value.to_value();
            value.convert_to_string(&mut |s| {
                writer.write_string_field(1, s);
            });
        }
    }

    Ok(())
}

fn write_native_value(
    writer: &mut ProtobufWriter,
    value: &OtlpAnyValue,
) -> Result<(), SerializerError> {
    match value {
        OtlpAnyValue::StringValue(s) => {
            writer.write_string_field(1, s.get_value());
        }
        OtlpAnyValue::BoolValue(b) => {
            writer.write_bool_field(2, b.get_value());
        }
        OtlpAnyValue::IntValue(i) => {
            writer.write_int64_field(3, i.get_value());
        }
        OtlpAnyValue::DoubleValue(d) => {
            writer.write_double_field(4, d.get_value());
        }
        OtlpAnyValue::ArrayValue(a) => {
            writer.write_message_field(5, |writer| {
                for item in a.get_values() {
                    writer.write_message_field(1, |writer| write_value(writer, item))?;
                }

                Ok(())
            })?;
        }
        OtlpAnyValue::KvlistValue(k) => {
            writer.write_message_field(6, |writer| {
                if cfg!(test) {
                    // Note: When building tests we sort the key list so that it is
                    // deterministice.
                    let mut values: Vec<(&Box<str>, &AnyValue)> = k.get_values().iter().collect();
                    values.sort_by(|left, right| left.0.cmp(right.0));
                    for (key, value) in values {
                        writer
                            .write_message_field(1, |writer| write_key_value(writer, key, value))?;
                    }
                } else {
                    for (key, value) in k.get_values() {
                        writer
                            .write_message_field(1, |writer| write_key_value(writer, key, value))?;
                    }
                }

                Ok(())
            })?;
        }
        OtlpAnyValue::BytesValue(b) => {
            writer.write_byte_array_field(7, b);
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use chrono::{SecondsFormat, Utc};
    use data_engine_recordset::*;
    use prost::Message;
    use regex::Regex;

    use crate::serializer::ProtobufField;

    use super::*;

    type OtlpExportLogsServiceRequest =
        opentelemetry_proto::tonic::collector::logs::v1::ExportLogsServiceRequest;
    type OtlpResourceLogs = opentelemetry_proto::tonic::logs::v1::ResourceLogs;
    type OtlpScopeLogs = opentelemetry_proto::tonic::logs::v1::ScopeLogs;
    type OtlpLogRecord = opentelemetry_proto::tonic::logs::v1::LogRecord;
    type OtlpResource = opentelemetry_proto::tonic::resource::v1::Resource;
    type OtlpInstrumentationScope = opentelemetry_proto::tonic::common::v1::InstrumentationScope;
    type OtlpKeyValue = opentelemetry_proto::tonic::common::v1::KeyValue;
    type OtlpAnyValue = opentelemetry_proto::tonic::common::v1::AnyValue;
    type OtlpValue = opentelemetry_proto::tonic::common::v1::any_value::Value;
    type OtlpArrayValue = opentelemetry_proto::tonic::common::v1::ArrayValue;
    type OtlpKeyValueList = opentelemetry_proto::tonic::common::v1::KeyValueList;

    #[test]
    fn test_write_key_value() {
        let run_test = |key_value: (&str, AnyValue), expected: OtlpKeyValue| {
            let mut writer = ProtobufWriter::new(0);

            write_key_value(&mut writer, key_value.0, &key_value.1).unwrap();

            let actual = OtlpKeyValue::decode(writer.freeze()).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            ("", AnyValue::Null),
            OtlpKeyValue {
                key: "".into(),
                value: None,
            },
        );

        run_test(
            ("key1", AnyValue::Null),
            OtlpKeyValue {
                key: "key1".into(),
                value: None,
            },
        );

        run_test(
            (
                "key1",
                AnyValue::Native(crate::OtlpAnyValue::StringValue(StringValueStorage::new(
                    "value1".into(),
                ))),
            ),
            OtlpKeyValue {
                key: "key1".into(),
                value: Some(OtlpAnyValue {
                    value: Some(OtlpValue::StringValue("value1".into())),
                }),
            },
        );
    }

    #[test]
    fn test_write_value() {
        let run_test = |value: AnyValue, expected: OtlpAnyValue| {
            let mut writer = ProtobufWriter::new(0);

            write_value(&mut writer, &value).unwrap();

            let actual = OtlpAnyValue::decode(writer.freeze()).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(AnyValue::Null, OtlpAnyValue { value: None });

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::StringValue(StringValueStorage::new(
                "".into(),
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::StringValue("".into())),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::StringValue(StringValueStorage::new(
                "hello world".into(),
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::StringValue("hello world".into())),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::BoolValue(BooleanValueStorage::new(
                true,
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::BoolValue(true)),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::BoolValue(BooleanValueStorage::new(
                false,
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::BoolValue(false)),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
            OtlpAnyValue {
                value: Some(OtlpValue::IntValue(18)),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(-18))),
            OtlpAnyValue {
                value: Some(OtlpValue::IntValue(-18)),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::DoubleValue(DoubleValueStorage::new(
                18.0,
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::DoubleValue(18.0)),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::DoubleValue(DoubleValueStorage::new(
                -18.0,
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::DoubleValue(-18.0)),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::ArrayValue(ArrayValueStorage::new(
                Vec::new(),
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::ArrayValue(OtlpArrayValue { values: Vec::new() })),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::ArrayValue(ArrayValueStorage::new(
                vec![
                    AnyValue::Null,
                    AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
                ],
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::ArrayValue(OtlpArrayValue {
                    values: vec![
                        OtlpAnyValue { value: None },
                        OtlpAnyValue {
                            value: Some(OtlpValue::IntValue(18)),
                        },
                    ],
                })),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::KvlistValue(MapValueStorage::new(
                HashMap::new(),
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::KvlistValue(OtlpKeyValueList {
                    values: Vec::new(),
                })),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::KvlistValue(MapValueStorage::new(
                HashMap::from([
                    ("".into(), AnyValue::Null),
                    ("key1".into(), AnyValue::Null),
                    (
                        "key2".into(),
                        AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(
                            18,
                        ))),
                    ),
                ]),
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::KvlistValue(OtlpKeyValueList {
                    values: vec![
                        OtlpKeyValue {
                            key: "".into(),
                            value: None,
                        },
                        OtlpKeyValue {
                            key: "key1".into(),
                            value: None,
                        },
                        OtlpKeyValue {
                            key: "key2".into(),
                            value: Some(OtlpAnyValue {
                                value: Some(OtlpValue::IntValue(18)),
                            }),
                        },
                    ],
                })),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::BytesValue(ByteArrayValueStorage::new(
                Vec::new(),
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::BytesValue(Vec::new())),
            },
        );

        run_test(
            AnyValue::Native(crate::OtlpAnyValue::BytesValue(ByteArrayValueStorage::new(
                vec![
                    IntegerValueStorage::new(0x00),
                    IntegerValueStorage::new(0xFF),
                ],
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::BytesValue(vec![0x00, 0xFF])),
            },
        );

        // Extended
        let now = Utc::now();
        run_test(
            AnyValue::Extended(ExtendedValue::DateTime(DateTimeValueStorage::new(
                now.into(),
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::StringValue(
                    now.to_rfc3339_opts(SecondsFormat::AutoSi, true),
                )),
            },
        );

        run_test(
            AnyValue::Extended(ExtendedValue::Regex(RegexValueStorage::new(
                Regex::new(".*").unwrap(),
            ))),
            OtlpAnyValue {
                value: Some(OtlpValue::StringValue(".*".into())),
            },
        );
    }

    #[test]
    fn test_write_resource() {
        let run_test = |value: Resource, expected: OtlpResource| {
            let mut writer = ProtobufWriter::new(0);

            write_resource(&mut writer, &value).unwrap();

            let actual = OtlpResource::decode(writer.freeze()).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            Resource::new(),
            OtlpResource {
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            },
        );

        run_test(
            Resource {
                attributes: MapValueStorage::new(HashMap::from([
                    ("key1".into(), AnyValue::Null),
                    (
                        "key2".into(),
                        AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(
                            18,
                        ))),
                    ),
                ])),
                extra_fields: Vec::new(),
            },
            OtlpResource {
                attributes: vec![
                    OtlpKeyValue {
                        key: "key1".into(),
                        value: None,
                    },
                    OtlpKeyValue {
                        key: "key2".into(),
                        value: Some(OtlpAnyValue {
                            value: Some(OtlpValue::IntValue(18)),
                        }),
                    },
                ],
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            },
        );

        run_test(
            Resource {
                attributes: MapValueStorage::new(HashMap::new()),
                extra_fields: vec![ProtobufField::Varint {
                    field_number: 2,
                    value: 18,
                }],
            },
            OtlpResource {
                attributes: Vec::new(),
                dropped_attributes_count: 18,
                entity_refs: Vec::new(),
            },
        );
    }

    #[test]
    fn test_write_instrumentation_scope() {
        let run_test = |value: InstrumentationScope, expected: OtlpInstrumentationScope| {
            let mut writer = ProtobufWriter::new(0);

            write_instrumentation_scope(&mut writer, &value).unwrap();

            let actual = OtlpInstrumentationScope::decode(writer.freeze()).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            InstrumentationScope {
                name: None,
                version: None,
                attributes: MapValueStorage::new(HashMap::new()),
                extra_fields: vec![ProtobufField::Varint {
                    field_number: 4,
                    value: 18,
                }],
            },
            OtlpInstrumentationScope {
                name: "".into(),
                version: "".into(),
                attributes: Vec::new(),
                dropped_attributes_count: 18,
            },
        );

        run_test(
            InstrumentationScope {
                name: Some(StringValueStorage::new("name".into())),
                version: Some(StringValueStorage::new("version".into())),
                attributes: MapValueStorage::new(HashMap::new()),
                extra_fields: Vec::new(),
            },
            OtlpInstrumentationScope {
                name: "name".into(),
                version: "version".into(),
                attributes: Vec::new(),
                dropped_attributes_count: 0,
            },
        );

        run_test(
            InstrumentationScope {
                name: None,
                version: None,
                attributes: MapValueStorage::new(HashMap::from([
                    ("key1".into(), AnyValue::Null),
                    (
                        "key2".into(),
                        AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(
                            18,
                        ))),
                    ),
                ])),
                extra_fields: Vec::new(),
            },
            OtlpInstrumentationScope {
                name: "".into(),
                version: "".into(),
                attributes: vec![
                    OtlpKeyValue {
                        key: "key1".into(),
                        value: None,
                    },
                    OtlpKeyValue {
                        key: "key2".into(),
                        value: Some(OtlpAnyValue {
                            value: Some(OtlpValue::IntValue(18)),
                        }),
                    },
                ],
                dropped_attributes_count: 0,
            },
        );
    }

    #[test]
    fn test_write_log_record() {
        let run_test = |value: LogRecord, expected: OtlpLogRecord| {
            let mut writer = ProtobufWriter::new(0);

            write_log_record(&mut writer, &value).unwrap();

            let actual = OtlpLogRecord::decode(writer.freeze()).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            LogRecord::new(),
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 0,
                severity_text: "".into(),
                body: None,
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                flags: 0,
                trace_id: Vec::new(),
                span_id: Vec::new(),
                event_name: "".into(),
            },
        );

        run_test(
            LogRecord::new()
                .with_timestamp_unix_nanos(1)
                .with_observed_timestamp_unix_nanos(2),
            OtlpLogRecord {
                time_unix_nano: 1,
                observed_time_unix_nano: 2,
                severity_number: 0,
                severity_text: "".into(),
                body: None,
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                flags: 0,
                trace_id: Vec::new(),
                span_id: Vec::new(),
                event_name: "".into(),
            },
        );

        run_test(
            LogRecord::new()
                .with_severity_number(10)
                .with_severity_text("Info".into()),
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 10,
                severity_text: "Info".into(),
                body: None,
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                flags: 0,
                trace_id: Vec::new(),
                span_id: Vec::new(),
                event_name: "".into(),
            },
        );

        run_test(
            LogRecord::new().with_body(AnyValue::Null),
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 0,
                severity_text: "".into(),
                body: Some(OtlpAnyValue { value: None }),
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                flags: 0,
                trace_id: Vec::new(),
                span_id: Vec::new(),
                event_name: "".into(),
            },
        );

        run_test(
            LogRecord::new()
                .with_attribute("key1", AnyValue::Null)
                .with_attribute(
                    "key2",
                    AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
                ),
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 0,
                severity_text: "".into(),
                body: None,
                attributes: vec![
                    OtlpKeyValue {
                        key: "key1".into(),
                        value: None,
                    },
                    OtlpKeyValue {
                        key: "key2".into(),
                        value: Some(OtlpAnyValue {
                            value: Some(OtlpValue::IntValue(18)),
                        }),
                    },
                ],
                dropped_attributes_count: 0,
                flags: 0,
                trace_id: Vec::new(),
                span_id: Vec::new(),
                event_name: "".into(),
            },
        );

        run_test(
            LogRecord::new()
                .with_flags(1)
                .with_trace_id(vec![0xFF])
                .with_span_id(vec![0x80])
                .with_event_name("event_name".into()),
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 0,
                severity_text: "".into(),
                body: None,
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                flags: 1,
                trace_id: vec![0xFF],
                span_id: vec![0x80],
                event_name: "event_name".into(),
            },
        );

        let mut log_record = LogRecord::new();
        log_record.extra_fields.push(ProtobufField::Varint {
            field_number: 7,
            value: 18,
        });
        run_test(
            log_record,
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 0,
                severity_text: "".into(),
                body: None,
                attributes: Vec::new(),
                dropped_attributes_count: 18,
                flags: 0,
                trace_id: Vec::new(),
                span_id: Vec::new(),
                event_name: "".into(),
            },
        );
    }

    #[test]
    fn test_write_scope_logs() {
        let run_test = |value: ScopeLogs, expected: OtlpScopeLogs| {
            let mut writer = ProtobufWriter::new(0);

            write_scope_logs(&mut writer, &value).unwrap();

            let actual = OtlpScopeLogs::decode(writer.freeze()).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            ScopeLogs::new(),
            OtlpScopeLogs {
                scope: None,
                log_records: Vec::new(),
                schema_url: "".into(),
            },
        );

        run_test(
            ScopeLogs::new()
                .with_instrumentation_scope(InstrumentationScope::new().with_name("name".into())),
            OtlpScopeLogs {
                scope: Some(OtlpInstrumentationScope {
                    name: "name".into(),
                    version: "".into(),
                    attributes: Vec::new(),
                    dropped_attributes_count: 0,
                }),
                log_records: Vec::new(),
                schema_url: "".into(),
            },
        );

        run_test(
            ScopeLogs::new()
                .with_log_record(LogRecord::new())
                .with_log_record(LogRecord::new()),
            OtlpScopeLogs {
                scope: None,
                log_records: vec![
                    OtlpLogRecord {
                        time_unix_nano: 0,
                        observed_time_unix_nano: 0,
                        severity_number: 0,
                        severity_text: "".into(),
                        body: None,
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                        flags: 0,
                        trace_id: Vec::new(),
                        span_id: Vec::new(),
                        event_name: "".into(),
                    },
                    OtlpLogRecord {
                        time_unix_nano: 0,
                        observed_time_unix_nano: 0,
                        severity_number: 0,
                        severity_text: "".into(),
                        body: None,
                        attributes: Vec::new(),
                        dropped_attributes_count: 0,
                        flags: 0,
                        trace_id: Vec::new(),
                        span_id: Vec::new(),
                        event_name: "".into(),
                    },
                ],
                schema_url: "".into(),
            },
        );

        let mut scope_logs = ScopeLogs::new();
        scope_logs
            .extra_fields
            .push(ProtobufField::LengthDelimited {
                field_number: 3,
                value: vec![0x61],
            });

        run_test(
            scope_logs,
            OtlpScopeLogs {
                scope: None,
                log_records: Vec::new(),
                schema_url: "a".into(),
            },
        );
    }

    #[test]
    fn test_write_resource_logs() {
        let run_test = |value: ResourceLogs, expected: OtlpResourceLogs| {
            let mut writer = ProtobufWriter::new(0);

            write_resource_logs(&mut writer, &value).unwrap();

            let actual = OtlpResourceLogs::decode(writer.freeze()).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            ResourceLogs::new(),
            OtlpResourceLogs {
                resource: None,
                scope_logs: Vec::new(),
                schema_url: "".into(),
            },
        );

        run_test(
            ResourceLogs::new().with_resource(Resource::new()),
            OtlpResourceLogs {
                resource: Some(OtlpResource {
                    attributes: Vec::new(),
                    dropped_attributes_count: 0,
                    entity_refs: Vec::new(),
                }),
                scope_logs: Vec::new(),
                schema_url: "".into(),
            },
        );

        run_test(
            ResourceLogs::new()
                .with_scope_logs(ScopeLogs::new())
                .with_scope_logs(ScopeLogs::new()),
            OtlpResourceLogs {
                resource: None,
                scope_logs: vec![
                    OtlpScopeLogs {
                        scope: None,
                        log_records: Vec::new(),
                        schema_url: "".into(),
                    },
                    OtlpScopeLogs {
                        scope: None,
                        log_records: Vec::new(),
                        schema_url: "".into(),
                    },
                ],
                schema_url: "".into(),
            },
        );

        let mut resource_logs = ResourceLogs::new();
        resource_logs
            .extra_fields
            .push(ProtobufField::LengthDelimited {
                field_number: 3,
                value: vec![0x61],
            });

        run_test(
            resource_logs,
            OtlpResourceLogs {
                resource: None,
                scope_logs: Vec::new(),
                schema_url: "a".into(),
            },
        );
    }

    #[test]
    fn test_write_export_logs_service_request() {
        let run_test = |value: ExportLogsServiceRequest, expected: OtlpExportLogsServiceRequest| {
            let bytes = write_export_logs_service_request(&value, 2048).unwrap();

            let actual = OtlpExportLogsServiceRequest::decode(&bytes[..]).unwrap();

            assert_eq!(expected, actual);
        };

        run_test(
            ExportLogsServiceRequest {
                resource_logs: Vec::new(),
            },
            OtlpExportLogsServiceRequest {
                resource_logs: Vec::new(),
            },
        );

        run_test(
            ExportLogsServiceRequest {
                resource_logs: vec![ResourceLogs::new(), ResourceLogs::new()],
            },
            OtlpExportLogsServiceRequest {
                resource_logs: vec![
                    OtlpResourceLogs {
                        resource: None,
                        scope_logs: Vec::new(),
                        schema_url: "".into(),
                    },
                    OtlpResourceLogs {
                        resource: None,
                        scope_logs: Vec::new(),
                        schema_url: "".into(),
                    },
                ],
            },
        );
    }
}
