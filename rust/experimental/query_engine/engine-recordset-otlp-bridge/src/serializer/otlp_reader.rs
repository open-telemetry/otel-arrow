use std::{collections::HashMap, str::FromStr};

use chrono::{TimeZone, Utc};
use data_engine_expressions::AsValue;
use data_engine_recordset::*;

use crate::{serializer::protobuf_reader::ProtobufReader, *};

pub fn read_export_logs_service_request(
    protobuf_data: &[u8],
) -> Result<ExportLogsServiceRequest, SerializerError> {
    let mut request = ExportLogsServiceRequest::new();

    let mut reader = ProtobufReader::new(protobuf_data);

    while reader.len() > 0 {
        let tag = reader.read_tag()?;

        match tag.field_number {
            1 => {
                reader.read_message(|mut reader| {
                    request.resource_logs.push(read_resource_logs(&mut reader)?);
                    Ok(())
                })?;
            }
            _ => {
                return Err(SerializerError::UnexpectedTag {
                    while_parsing: "export_logs_service_request",
                    field_number: tag.field_number,
                    wire_type: tag.wire_type,
                });
            }
        }
    }

    Ok(request)
}

fn read_resource_logs(reader: &mut ProtobufReader) -> Result<ResourceLogs, SerializerError> {
    let mut resource_logs = ResourceLogs::new();

    while reader.len() > 0 {
        let tag = reader.read_tag()?;

        match tag.field_number {
            1 => {
                reader.read_message(|mut reader| {
                    resource_logs.resource = Some(read_resource(&mut reader)?);
                    Ok(())
                })?;
            }
            2 => {
                reader.read_message(|mut reader| {
                    resource_logs.scope_logs.push(read_scope_logs(&mut reader)?);
                    Ok(())
                })?;
            }
            _ => {
                let field = reader.read_field(tag)?;
                resource_logs.extra_fields.push(field);
            }
        }
    }

    Ok(resource_logs)
}

fn read_scope_logs(reader: &mut ProtobufReader) -> Result<ScopeLogs, SerializerError> {
    let mut scope_logs = ScopeLogs::new();

    while reader.len() > 0 {
        let tag = reader.read_tag()?;

        match tag.field_number {
            1 => {
                reader.read_message(|mut reader| {
                    scope_logs.instrumentation_scope =
                        Some(read_instrumentation_scope(&mut reader)?);
                    Ok(())
                })?;
            }
            2 => {
                reader.read_message(|mut reader| {
                    scope_logs.log_records.push(read_log_record(&mut reader)?);
                    Ok(())
                })?;
            }
            _ => {
                let field = reader.read_field(tag)?;
                scope_logs.extra_fields.push(field);
            }
        }
    }

    Ok(scope_logs)
}

fn read_log_record(reader: &mut ProtobufReader) -> Result<LogRecord, SerializerError> {
    let mut log_record = LogRecord::new();

    while reader.len() > 0 {
        let tag = reader.read_tag()?;

        match tag.field_number {
            1 => {
                log_record.timestamp = Some(DateTimeValueStorage::new(
                    Utc.timestamp_nanos(reader.read_fixed64()? as i64).into(),
                ));
            }
            2 => {
                log_record.severity_number =
                    Some(IntegerValueStorage::new(reader.read_varint64()? as i32));
            }
            3 => {
                log_record.severity_text = Some(StringValueStorage::new(reader.read_string()?));
            }
            5 => {
                if !reader.read_message(|mut reader| {
                    log_record.body = Some(read_value(&mut reader)?);
                    Ok(())
                })? {
                    log_record.body = Some(AnyValue::Null);
                }
            }
            6 => {
                reader.read_message(|mut reader| {
                    if let Some((key, value)) = read_key_value(&mut reader)? {
                        if key.as_ref() == "query_engine.diagnostic_level" {
                            let v = value.to_value();
                            if let Some(i) = v.convert_to_integer() {
                                log_record.diagnostic_level =
                                    RecordSetEngineDiagnosticLevel::from_usize(i as usize);
                            } else {
                                v.convert_to_string(&mut |s| {
                                    if let Ok(v) = RecordSetEngineDiagnosticLevel::from_str(s) {
                                        log_record.diagnostic_level = Some(v);
                                    }
                                });
                            }
                        } else {
                            log_record.attributes.get_values_mut().insert(key, value);
                        }
                    }
                    Ok(())
                })?;
            }
            8 => {
                log_record.flags = Some(IntegerValueStorage::new(reader.read_fixed32()?));
            }
            9 => {
                log_record.trace_id = Some(ByteArrayValueStorage::new(
                    reader
                        .read_bytes()?
                        .iter()
                        .map(|v| IntegerValueStorage::new(*v))
                        .collect(),
                ));
            }
            10 => {
                log_record.span_id = Some(ByteArrayValueStorage::new(
                    reader
                        .read_bytes()?
                        .iter()
                        .map(|v| IntegerValueStorage::new(*v))
                        .collect(),
                ));
            }
            11 => {
                log_record.observed_timestamp = Some(DateTimeValueStorage::new(
                    Utc.timestamp_nanos(reader.read_fixed64()? as i64).into(),
                ));
            }
            12 => {
                log_record.event_name = Some(StringValueStorage::new(reader.read_string()?));
            }
            _ => {
                let field = reader.read_field(tag)?;
                log_record.extra_fields.push(field);
            }
        }
    }

    Ok(log_record)
}

fn read_resource(reader: &mut ProtobufReader) -> Result<Resource, SerializerError> {
    let mut resource = Resource::new();

    while reader.len() > 0 {
        let tag = reader.read_tag()?;

        match tag.field_number {
            1 => {
                reader.read_message(|mut reader| {
                    if let Some((key, value)) = read_key_value(&mut reader)? {
                        resource.attributes.get_values_mut().insert(key, value);
                    }
                    Ok(())
                })?;
            }
            _ => {
                let field = reader.read_field(tag)?;
                resource.extra_fields.push(field);
            }
        }
    }

    Ok(resource)
}

fn read_instrumentation_scope(
    reader: &mut ProtobufReader,
) -> Result<InstrumentationScope, SerializerError> {
    let mut scope = InstrumentationScope::new();

    while reader.len() > 0 {
        let tag = reader.read_tag()?;

        match tag.field_number {
            1 => {
                scope.name = Some(StringValueStorage::new(reader.read_string()?));
            }
            2 => {
                scope.version = Some(StringValueStorage::new(reader.read_string()?));
            }
            3 => {
                reader.read_message(|mut reader| {
                    if let Some((key, value)) = read_key_value(&mut reader)? {
                        scope.attributes.get_values_mut().insert(key, value);
                    }
                    Ok(())
                })?;
            }
            _ => {
                let field = reader.read_field(tag)?;
                scope.extra_fields.push(field);
            }
        }
    }

    Ok(scope)
}

fn read_value(reader: &mut ProtobufReader) -> Result<AnyValue, SerializerError> {
    let tag = reader.read_tag()?;

    Ok(AnyValue::Native(match tag.field_number {
        1 => OtlpAnyValue::StringValue(StringValueStorage::new(reader.read_string()?)),
        2 => OtlpAnyValue::BoolValue(BooleanValueStorage::new(reader.read_bool()?)),
        3 => OtlpAnyValue::IntValue(IntegerValueStorage::new(reader.read_int64()?)),
        4 => OtlpAnyValue::DoubleValue(DoubleValueStorage::new(reader.read_double()?)),
        5 => {
            let mut values = Vec::new();
            reader.read_message(|mut reader| {
                while reader.len() > 0 {
                    let tag = reader.read_tag()?;
                    match tag.field_number {
                        1 => {
                            if !reader.read_message(|mut reader| {
                                values.push(read_value(&mut reader)?);
                                Ok(())
                            })? {
                                values.push(AnyValue::Null);
                            }
                        }
                        _ => {
                            return Err(SerializerError::UnexpectedTag {
                                while_parsing: "array",
                                field_number: tag.field_number,
                                wire_type: tag.wire_type,
                            });
                        }
                    }
                }

                Ok(())
            })?;
            OtlpAnyValue::ArrayValue(ArrayValueStorage::new(values))
        }
        6 => {
            let mut values = HashMap::new();
            reader.read_message(|mut reader| {
                while reader.len() > 0 {
                    let tag = reader.read_tag()?;
                    match tag.field_number {
                        1 => {
                            reader.read_message(|mut reader| {
                                if let Some((key, value)) = read_key_value(&mut reader)? {
                                    values.insert(key, value);
                                }
                                Ok(())
                            })?;
                        }
                        _ => {
                            return Err(SerializerError::UnexpectedTag {
                                while_parsing: "kvlist",
                                field_number: tag.field_number,
                                wire_type: tag.wire_type,
                            });
                        }
                    }
                }

                Ok(())
            })?;
            OtlpAnyValue::KvlistValue(MapValueStorage::new(values))
        }
        7 => {
            let bytes = reader.read_bytes()?;

            OtlpAnyValue::BytesValue(ByteArrayValueStorage::new(
                bytes.iter().map(|v| IntegerValueStorage::new(*v)).collect(),
            ))
        }
        _ => {
            return Err(SerializerError::UnexpectedTag {
                while_parsing: "value",
                field_number: tag.field_number,
                wire_type: tag.wire_type,
            });
        }
    }))
}

fn read_key_value(
    reader: &mut ProtobufReader,
) -> Result<Option<(Box<str>, AnyValue)>, SerializerError> {
    let mut key = None;
    let mut value = AnyValue::Null;

    while reader.len() > 0 {
        let tag = reader.read_tag()?;
        match tag.field_number {
            1 => {
                let k = reader.read_string()?;
                if !k.is_empty() {
                    key = Some(k.into());
                }
            }
            2 => {
                reader.read_message(|mut reader| {
                    value = read_value(&mut reader)?;
                    Ok(())
                })?;
            }
            _ => {
                return Err(SerializerError::UnexpectedTag {
                    while_parsing: "key_value",
                    field_number: tag.field_number,
                    wire_type: tag.wire_type,
                });
            }
        }
    }

    if let Some(k) = key {
        Ok(Some((k, value)))
    } else {
        Ok(None)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use bytes::BytesMut;
    use prost::Message;

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
    fn test_read_value() {
        let run_test = |value: OtlpAnyValue, expected: AnyValue| {
            let mut buffer = BytesMut::new();

            value.encode(&mut buffer).unwrap();

            let bytes = buffer.freeze();

            let mut reader = ProtobufReader::new(&bytes);

            let actual = read_value(&mut reader).unwrap();

            assert_eq!(
                expected.to_value().to_string(),
                actual.to_value().to_string()
            );
        };

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::StringValue("hello world".into())),
            },
            AnyValue::Native(crate::OtlpAnyValue::StringValue(StringValueStorage::new(
                "hello world".into(),
            ))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::BoolValue(true)),
            },
            AnyValue::Native(crate::OtlpAnyValue::BoolValue(BooleanValueStorage::new(
                true,
            ))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::BoolValue(false)),
            },
            AnyValue::Native(crate::OtlpAnyValue::BoolValue(BooleanValueStorage::new(
                false,
            ))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::IntValue(18)),
            },
            AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::IntValue(-18)),
            },
            AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(-18))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::DoubleValue(-18.0)),
            },
            AnyValue::Native(crate::OtlpAnyValue::DoubleValue(DoubleValueStorage::new(
                -18.0,
            ))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::ArrayValue(OtlpArrayValue { values: Vec::new() })),
            },
            AnyValue::Native(crate::OtlpAnyValue::ArrayValue(ArrayValueStorage::new(
                Vec::new(),
            ))),
        );

        run_test(
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
            AnyValue::Native(crate::OtlpAnyValue::ArrayValue(ArrayValueStorage::new(
                vec![
                    AnyValue::Null,
                    AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
                ],
            ))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::KvlistValue(OtlpKeyValueList {
                    values: Vec::new(),
                })),
            },
            AnyValue::Native(crate::OtlpAnyValue::KvlistValue(MapValueStorage::new(
                HashMap::new(),
            ))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::KvlistValue(OtlpKeyValueList {
                    values: vec![
                        OtlpKeyValue {
                            key: "key1".into(),
                            value: None,
                        },
                        OtlpKeyValue {
                            key: "key2".into(),
                            value: Some(OtlpAnyValue { value: None }),
                        },
                        OtlpKeyValue {
                            key: "key3".into(),
                            value: Some(OtlpAnyValue {
                                value: Some(OtlpValue::IntValue(18)),
                            }),
                        },
                    ],
                })),
            },
            AnyValue::Native(crate::OtlpAnyValue::KvlistValue(MapValueStorage::new(
                HashMap::from([
                    ("key1".into(), AnyValue::Null),
                    ("key2".into(), AnyValue::Null),
                    (
                        "key3".into(),
                        AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(
                            18,
                        ))),
                    ),
                ]),
            ))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::BytesValue(Vec::new())),
            },
            AnyValue::Native(crate::OtlpAnyValue::BytesValue(ByteArrayValueStorage::new(
                Vec::new(),
            ))),
        );

        run_test(
            OtlpAnyValue {
                value: Some(OtlpValue::BytesValue(vec![0x00, 0xFF])),
            },
            AnyValue::Native(crate::OtlpAnyValue::BytesValue(ByteArrayValueStorage::new(
                vec![
                    IntegerValueStorage::new(0x00),
                    IntegerValueStorage::new(0xFF),
                ],
            ))),
        );
    }

    #[test]
    fn test_read_key_value() {
        let run_test = |value: OtlpKeyValue, expected: Option<(Box<str>, AnyValue)>| {
            let mut buffer = BytesMut::new();

            value.encode(&mut buffer).unwrap();

            let bytes = buffer.freeze();

            let mut reader = ProtobufReader::new(&bytes);

            let actual = read_key_value(&mut reader).unwrap();

            if expected.is_none() {
                assert!(actual.is_none());
            } else {
                assert_eq!(expected.as_ref().unwrap().0, actual.as_ref().unwrap().0,);

                assert_eq!(
                    expected.as_ref().unwrap().1.to_value().to_string(),
                    actual.as_ref().unwrap().1.to_value().to_string()
                );
            }
        };

        run_test(
            OtlpKeyValue {
                key: "".into(),
                value: None,
            },
            None,
        );

        run_test(
            OtlpKeyValue {
                key: "".into(),
                value: Some(OtlpAnyValue { value: None }),
            },
            None,
        );

        run_test(
            OtlpKeyValue {
                key: "key1".into(),
                value: None,
            },
            Some(("key1".into(), AnyValue::Null)),
        );

        run_test(
            OtlpKeyValue {
                key: "key1".into(),
                value: Some(OtlpAnyValue { value: None }),
            },
            Some(("key1".into(), AnyValue::Null)),
        );

        run_test(
            OtlpKeyValue {
                key: "key1".into(),
                value: Some(OtlpAnyValue {
                    value: Some(OtlpValue::IntValue(18)),
                }),
            },
            Some((
                "key1".into(),
                AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
            )),
        );
    }

    #[test]
    fn test_read_resource() {
        let run_test = |value: OtlpResource, expected: Resource| {
            let mut buffer = BytesMut::new();

            value.encode(&mut buffer).unwrap();

            let bytes = buffer.freeze();

            let mut reader = ProtobufReader::new(&bytes);

            let actual = read_resource(&mut reader).unwrap();

            assert_eq!(
                expected.to_value().to_string(),
                actual.to_value().to_string()
            );
        };

        run_test(
            OtlpResource {
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                entity_refs: Vec::new(),
            },
            Resource::new(),
        );

        run_test(
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
            Resource::new()
                .with_attribute("key1", AnyValue::Null)
                .with_attribute(
                    "key2",
                    AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
                ),
        );

        let mut resource = Resource::new();
        resource
            .extra_fields
            .push(serializer::ProtobufField::Varint {
                field_number: 2,
                value: 18,
            });
        run_test(
            OtlpResource {
                attributes: Vec::new(),
                dropped_attributes_count: 18,
                entity_refs: Vec::new(),
            },
            resource,
        );
    }

    #[test]
    fn test_read_instrumentation_scope() {
        let run_test = |value: OtlpInstrumentationScope, expected: InstrumentationScope| {
            let mut buffer = BytesMut::new();

            value.encode(&mut buffer).unwrap();

            let bytes = buffer.freeze();

            let mut reader = ProtobufReader::new(&bytes);

            let actual = read_instrumentation_scope(&mut reader).unwrap();

            assert_eq!(
                expected.to_value().to_string(),
                actual.to_value().to_string()
            );
        };

        run_test(
            OtlpInstrumentationScope {
                name: "".into(),
                version: "".into(),
                attributes: Vec::new(),
                dropped_attributes_count: 0,
            },
            InstrumentationScope::new(),
        );

        run_test(
            OtlpInstrumentationScope {
                name: "name".into(),
                version: "version".into(),
                attributes: Vec::new(),
                dropped_attributes_count: 0,
            },
            InstrumentationScope::new()
                .with_name("name".into())
                .with_version("version".into()),
        );

        run_test(
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
            InstrumentationScope::new()
                .with_attribute("key1", AnyValue::Null)
                .with_attribute(
                    "key2",
                    AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
                ),
        );

        let mut scope = InstrumentationScope::new();
        scope.extra_fields.push(serializer::ProtobufField::Varint {
            field_number: 4,
            value: 18,
        });
        run_test(
            OtlpInstrumentationScope {
                name: "".into(),
                version: "".into(),
                attributes: Vec::new(),
                dropped_attributes_count: 18,
            },
            scope,
        );
    }

    #[test]
    fn test_read_log_record() {
        let run_test = |value: OtlpLogRecord, expected: LogRecord| {
            let mut buffer = BytesMut::new();

            value.encode(&mut buffer).unwrap();

            let bytes = buffer.freeze();

            let mut reader = ProtobufReader::new(&bytes);

            let actual = read_log_record(&mut reader).unwrap();

            assert_eq!(
                expected.to_value().to_string(),
                actual.to_value().to_string()
            );
        };

        run_test(
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
            LogRecord::new(),
        );

        run_test(
            OtlpLogRecord {
                time_unix_nano: 1,
                observed_time_unix_nano: 1,
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
            LogRecord::new()
                .with_timestamp_unix_nanos(1)
                .with_observed_timestamp_unix_nanos(1),
        );

        run_test(
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 1,
                severity_text: "Info".into(),
                body: None,
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                flags: 0,
                trace_id: Vec::new(),
                span_id: Vec::new(),
                event_name: "".into(),
            },
            LogRecord::new()
                .with_severity_number(1)
                .with_severity_text("Info".into()),
        );

        run_test(
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
            LogRecord::new().with_body(AnyValue::Null),
        );

        run_test(
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 0,
                severity_text: "".into(),
                body: Some(OtlpAnyValue {
                    value: Some(OtlpValue::IntValue(18)),
                }),
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                flags: 0,
                trace_id: Vec::new(),
                span_id: Vec::new(),
                event_name: "".into(),
            },
            LogRecord::new().with_body(AnyValue::Native(crate::OtlpAnyValue::IntValue(
                IntegerValueStorage::new(18),
            ))),
        );

        run_test(
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 0,
                severity_text: "".into(),
                body: None,
                attributes: vec![
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
                        value: Some(OtlpAnyValue { value: None }),
                    },
                    OtlpKeyValue {
                        key: "key3".into(),
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
            LogRecord::new()
                .with_attribute("key1", AnyValue::Null)
                .with_attribute("key2", AnyValue::Null)
                .with_attribute(
                    "key3",
                    AnyValue::Native(crate::OtlpAnyValue::IntValue(IntegerValueStorage::new(18))),
                ),
        );

        run_test(
            OtlpLogRecord {
                time_unix_nano: 0,
                observed_time_unix_nano: 0,
                severity_number: 0,
                severity_text: "".into(),
                body: None,
                attributes: Vec::new(),
                dropped_attributes_count: 0,
                flags: 1,
                trace_id: vec![0x00, 0xFF],
                span_id: vec![0x00, 0xFF],
                event_name: "".into(),
            },
            LogRecord::new()
                .with_flags(1)
                .with_trace_id(vec![0x00, 0xFF])
                .with_span_id(vec![0x00, 0xFF]),
        );

        run_test(
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
                event_name: "event_name".into(),
            },
            LogRecord::new().with_event_name("event_name".into()),
        );

        let mut log_record = LogRecord::new();
        log_record
            .extra_fields
            .push(serializer::ProtobufField::Varint {
                field_number: 7,
                value: 18,
            });
        run_test(
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
            log_record,
        );
    }

    #[test]
    fn test_read_scope_logs() {
        let run_test = |value: OtlpScopeLogs, expected: ScopeLogs| {
            let mut buffer = BytesMut::new();

            value.encode(&mut buffer).unwrap();

            let bytes = buffer.freeze();

            let mut reader = ProtobufReader::new(&bytes);

            let actual = read_scope_logs(&mut reader).unwrap();

            assert_eq!(
                expected
                    .instrumentation_scope
                    .map(|v| v.to_value().to_string()),
                actual
                    .instrumentation_scope
                    .map(|v| v.to_value().to_string()),
            );

            assert_eq!(
                expected
                    .log_records
                    .iter()
                    .map(|v| v.to_value().to_string())
                    .collect::<Vec<_>>(),
                actual
                    .log_records
                    .iter()
                    .map(|v| v.to_value().to_string())
                    .collect::<Vec<_>>(),
            );
        };

        run_test(
            OtlpScopeLogs {
                scope: None,
                schema_url: "".into(),
                log_records: Vec::new(),
            },
            ScopeLogs::new(),
        );

        run_test(
            OtlpScopeLogs {
                scope: Some(OtlpInstrumentationScope {
                    name: "name".into(),
                    version: "".into(),
                    attributes: Vec::new(),
                    dropped_attributes_count: 0,
                }),
                schema_url: "".into(),
                log_records: Vec::new(),
            },
            ScopeLogs::new()
                .with_instrumentation_scope(InstrumentationScope::new().with_name("name".into())),
        );

        run_test(
            OtlpScopeLogs {
                scope: None,
                schema_url: "".into(),
                log_records: vec![
                    OtlpLogRecord {
                        time_unix_nano: 1,
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
                        time_unix_nano: 1,
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
            },
            ScopeLogs::new()
                .with_log_record(LogRecord::new().with_timestamp_unix_nanos(1))
                .with_log_record(LogRecord::new().with_timestamp_unix_nanos(1)),
        );

        let mut scope_logs = ScopeLogs::new();
        scope_logs
            .extra_fields
            .push(serializer::ProtobufField::LengthDelimited {
                field_number: 3,
                value: vec![0x61],
            });
        run_test(
            OtlpScopeLogs {
                scope: None,
                schema_url: "a".into(),
                log_records: Vec::new(),
            },
            scope_logs,
        );
    }

    #[test]
    fn test_read_resource_logs() {
        let run_test = |value: OtlpResourceLogs, expected: ResourceLogs| {
            let mut buffer = BytesMut::new();

            value.encode(&mut buffer).unwrap();

            let bytes = buffer.freeze();

            let mut reader = ProtobufReader::new(&bytes);

            let actual = read_resource_logs(&mut reader).unwrap();

            assert_eq!(
                expected.resource.map(|v| v.to_value().to_string()),
                actual.resource.map(|v| v.to_value().to_string()),
            );

            assert_eq!(expected.scope_logs.len(), actual.scope_logs.len(),);
        };

        run_test(
            OtlpResourceLogs {
                resource: None,
                scope_logs: Vec::new(),
                schema_url: "".into(),
            },
            ResourceLogs::new(),
        );

        run_test(
            OtlpResourceLogs {
                resource: None,
                scope_logs: vec![
                    OtlpScopeLogs {
                        scope: None,
                        log_records: Vec::new(),
                        schema_url: "a".into(),
                    },
                    OtlpScopeLogs {
                        scope: None,
                        log_records: Vec::new(),
                        schema_url: "a".into(),
                    },
                ],
                schema_url: "".into(),
            },
            ResourceLogs::new()
                .with_scope_logs(ScopeLogs::new())
                .with_scope_logs(ScopeLogs::new()),
        );

        let mut resource_logs = ResourceLogs::new();
        resource_logs
            .extra_fields
            .push(serializer::ProtobufField::LengthDelimited {
                field_number: 3,
                value: vec![0x61],
            });

        run_test(
            OtlpResourceLogs {
                resource: None,
                scope_logs: Vec::new(),
                schema_url: "a".into(),
            },
            resource_logs,
        );
    }

    #[test]
    fn test_read_export_logs_service_request() {
        let run_test = |value: OtlpExportLogsServiceRequest, expected: ExportLogsServiceRequest| {
            let mut buffer = BytesMut::new();

            value.encode(&mut buffer).unwrap();

            let bytes = buffer.freeze();

            let actual = read_export_logs_service_request(&bytes).unwrap();

            assert_eq!(expected.resource_logs.len(), actual.resource_logs.len(),);
        };

        run_test(
            OtlpExportLogsServiceRequest {
                resource_logs: Vec::new(),
            },
            ExportLogsServiceRequest::new(),
        );

        run_test(
            OtlpExportLogsServiceRequest {
                resource_logs: vec![
                    OtlpResourceLogs {
                        resource: None,
                        scope_logs: Vec::new(),
                        schema_url: "a".into(),
                    },
                    OtlpResourceLogs {
                        resource: None,
                        scope_logs: Vec::new(),
                        schema_url: "a".into(),
                    },
                ],
            },
            ExportLogsServiceRequest::new()
                .with_resource_logs(ResourceLogs::new())
                .with_resource_logs(ResourceLogs::new()),
        );
    }
}
