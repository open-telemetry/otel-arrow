// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Encoder unit tests and shared native-log fixtures.

use super::super::SyslogBatchBuilder;
use super::cef::{append_escaped, cef_value};
use super::fields::FieldValue;
use super::*;
use crate::EncodedPdataBatch;
use crate::{CodecService, CodecServiceBuilder, EncodingPlan, OtapPayload, PdataEncoding};
use arrow::array::{RecordBatch, UInt16Array, UInt32Array};
use chrono::{DateTime, Local, TimeZone, Utc};
use otel_arrow_dfe_config::codecs::CodecsConfig;
use otel_arrow_dfe_pdata::arrays::{MaybeDictArrayAccessor, NullableArrayAccessor};
use otel_arrow_dfe_pdata::encode::encode_logs_otap_batch;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    common::v1::{AnyValue, KeyValue},
    logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
    resource::v1::Resource,
};
use otel_arrow_dfe_pdata::schema::consts;
use otel_arrow_dfe_pdata::{OtlpProtoBytes, TryIntoWithOptions};

fn records(logs: Vec<LogRecord>, attributes: Vec<KeyValue>) -> OtapArrowRecords {
    encode_logs_otap_batch(&LogsData {
        resource_logs: vec![ResourceLogs {
            resource: Some(Resource {
                attributes,
                ..Default::default()
            }),
            scope_logs: vec![ScopeLogs {
                log_records: logs,
                ..Default::default()
            }],
            ..Default::default()
        }],
    })
    .unwrap()
}

fn log(body: &str) -> LogRecord {
    LogRecord {
        body: Some(AnyValue::new_string(body)),
        ..Default::default()
    }
}

fn text(key: &str, value: &str) -> KeyValue {
    KeyValue::new(key, AnyValue::new_string(value))
}

fn integer(key: &str, value: i64) -> KeyValue {
    KeyValue::new(key, AnyValue::new_int(value))
}

fn messages(bytes: &[u8]) -> Vec<&[u8]> {
    assert_eq!(&bytes[..4], b"SLG1");
    let count = u32::from_be_bytes(bytes[4..8].try_into().unwrap()) as usize;
    let mut cursor = BATCH_HEADER_LEN;
    let mut messages = Vec::new();
    while messages.len() < count {
        let run_count = u32::from_be_bytes(bytes[cursor + 8..cursor + 12].try_into().unwrap());
        assert_ne!(run_count, 0);
        cursor += RUN_HEADER_LEN;
        for _ in 0..run_count {
            let len = u32::from_be_bytes(bytes[cursor..cursor + 4].try_into().unwrap()) as usize;
            cursor += FRAME_HEADER_LEN;
            messages.push(&bytes[cursor..cursor + len]);
            cursor += len;
        }
    }
    assert_eq!(messages.len(), count);
    assert_eq!(cursor, bytes.len());
    messages
}

fn configured_service(options: &str) -> Result<CodecService, CodecError> {
    let config: CodecsConfig = serde_yaml::from_str(&format!(
        "bindings:\n  syslog-batch:\n    options: {options}"
    ))
    .unwrap();
    CodecServiceBuilder::from_global_registry()
        .unwrap()
        .with_configuration(CodecsConfig::resolve([&config]))
        .build()
}

fn cef_attributes() -> Vec<KeyValue> {
    vec![
        text("cef.device_vendor", "vendor"),
        text("cef.device_product", "product"),
        text("cef.device_version", "1.0"),
        text("cef.device_event_class_id", "100"),
        text("cef.name", "event"),
        integer("cef.severity", 6),
        text("src", "192.0.2.1"),
    ]
}

fn decode(bytes: Bytes) -> Vec<LogRecord> {
    use prost::Message;
    let service = CodecService::new().unwrap();
    let codec = super::super::resolve_syslog().unwrap();
    let records = OtapPayload::from(codec.admit(SignalType::Logs, bytes).unwrap())
        .try_into_otap(&service)
        .unwrap();
    let otlp: OtlpProtoBytes = records.try_into_with_default().unwrap();
    let OtlpProtoBytes::ExportLogsRequest(bytes) = otlp else {
        panic!("expected logs");
    };
    LogsData::decode(bytes)
        .unwrap()
        .resource_logs
        .into_iter()
        .flat_map(|resource| resource.scope_logs)
        .flat_map(|scope| scope.log_records)
        .collect()
}

/// Scenario: CEF fields contain plain spans, adjacent escapes, UTF-8 bytes, or input/output beyond the remaining message capacity.
/// Guarantees: Escaping preserves bytes and output limits; raw oversized values fail without changing the existing message.
#[test]
fn syslog_encoder_escaping_spans_preserves_bytes_and_limits() {
    for (header, input, expected) in [
        (true, b"".as_slice(), b"".as_slice()),
        (false, b"plain text".as_slice(), b"plain text".as_slice()),
        (true, br"|a\=b|".as_slice(), br"\|a\\=b\|".as_slice()),
        (
            false,
            b"\\a=b\n\rc|".as_slice(),
            br"\\a\=b\n\rc|".as_slice(),
        ),
        (false, b"\xc3\xa9".as_slice(), b"\xc3\xa9".as_slice()),
    ] {
        let mut message = b"prefix:".to_vec();
        append_escaped(&mut message, input, header).unwrap();
        assert_eq!(&message[..7], b"prefix:");
        assert_eq!(&message[7..], expected);
    }
    for header in [false, true] {
        let prefix = b"prefix:";
        let mut message = prefix.to_vec();
        let mut oversized = vec![b'x'; MAX_MESSAGE_SIZE - prefix.len() + 1];
        oversized[0] = b'\\';
        assert!(append_escaped(&mut message, &oversized, header).is_err());
        assert_eq!(message, prefix);
        let mut message = vec![b'x'; MAX_MESSAGE_SIZE - 2];
        append_escaped(&mut message, b"\\", header).unwrap();
        assert_eq!(message.len(), MAX_MESSAGE_SIZE);
        assert_eq!(&message[MAX_MESSAGE_SIZE - 2..], b"\\\\");
        let mut message = vec![b'x'; MAX_MESSAGE_SIZE - 1];
        assert!(append_escaped(&mut message, b"\\", header).is_err());
        assert!(message.len() <= MAX_MESSAGE_SIZE);
        let mut message = Vec::new();
        let plain = vec![b'x'; MAX_MESSAGE_SIZE];
        append_escaped(&mut message, &plain, header).unwrap();
        assert_eq!(message, plain);
        assert!(append_escaped(&mut message, b"x", header).is_err());
    }
}

/// Scenario: Physical logs and attributes are interleaved across resources and scopes.
/// Guarantees: The codec associates headers and messages with row IDs, preserving hierarchy order and each parent's duplicate precedence.
#[test]
fn syslog_encoder_index_matches_reordered_hierarchy() {
    let group = |host, messages: &[&[&str]]| ResourceLogs {
        resource: Some(Resource {
            attributes: vec![text("host.name", host)],
            ..Default::default()
        }),
        scope_logs: messages
            .iter()
            .map(|messages| ScopeLogs {
                log_records: messages
                    .iter()
                    .map(|message| {
                        let mut log = log("");
                        log.attributes = vec![
                            text("syslog.message", "overridden"),
                            text("syslog.message", message),
                        ];
                        log
                    })
                    .collect(),
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let mut input = encode_logs_otap_batch(&LogsData {
        resource_logs: vec![
            group("first", &[&["one", "two"], &["three"]]),
            group("second", &[&["four"]]),
            group("third", &[&["five"]]),
        ],
    })
    .unwrap();
    let mut encoder = SyslogEncoder::new(EncodePolicy::default());
    let expected = encoder.encode(input.clone()).unwrap();
    for payload in [
        ArrowPayloadType::Logs,
        ArrowPayloadType::LogAttrs,
        ArrowPayloadType::ResourceAttrs,
    ] {
        let batch = input.get(payload).unwrap();
        let order = if payload == ArrowPayloadType::Logs {
            vec![3, 2, 0, 4, 1]
        } else {
            let parents = MaybeDictArrayAccessor::<UInt16Array>::try_new(
                batch.column_by_name(consts::PARENT_ID).unwrap(),
            )
            .unwrap();
            let mut ranks = std::collections::BTreeMap::new();
            let mut order: Vec<_> = (0..batch.num_rows())
                .map(|row| {
                    let rank = ranks.entry(parents.value_at(row)).or_insert(0);
                    let result = (*rank, row as u32);
                    *rank += 1;
                    result
                })
                .collect();
            order.sort_unstable();
            order.into_iter().map(|(_, row)| row).collect()
        };
        let order = UInt32Array::from(order);
        let columns = batch
            .columns()
            .iter()
            .map(|column| arrow::compute::take(column.as_ref(), &order, None).unwrap())
            .collect();
        let reordered = RecordBatch::try_new(batch.schema(), columns).unwrap();
        input.set(payload, reordered).unwrap();
    }
    assert_eq!(encoder.encode(input).unwrap(), expected);
}

/// Scenario: Logs override cached resource defaults across scopes and subsequent resource groups.
/// Guarantees: Per-log overrides never change later defaults, and each resource supplies only its own header fields.
#[test]
fn syslog_encoder_resource_defaults_are_isolated() {
    let group = |attributes, logs: Vec<Vec<LogRecord>>| ResourceLogs {
        resource: Some(Resource {
            attributes,
            ..Default::default()
        }),
        scope_logs: logs
            .into_iter()
            .map(|log_records| ScopeLogs {
                log_records,
                ..Default::default()
            })
            .collect(),
        ..Default::default()
    };
    let mut overridden = log("override");
    overridden.attributes = vec![
        text("syslog.host_name", "override-host"),
        text("syslog.app_name", "override-app"),
        text("syslog.process_id_str", r"opaque\=pid"),
    ];
    let input = encode_logs_otap_batch(&LogsData {
        resource_logs: vec![
            group(
                vec![
                    text("host.name", "first"),
                    text("service.name", "first-app"),
                    integer("process.pid", 11),
                ],
                vec![vec![overridden, log("same-scope")], vec![log("next-scope")]],
            ),
            group(
                vec![
                    text("host.name", "second"),
                    text("service.name", "second-app"),
                    integer("process.pid", 22),
                ],
                vec![vec![log("next-resource")]],
            ),
            group(vec![], vec![vec![log("no-defaults")]]),
        ],
    })
    .unwrap();
    let mut encoder = SyslogEncoder::new(EncodePolicy::default());
    let bytes = encoder.encode(input).unwrap();
    assert_eq!(
        messages(&bytes),
        [
            br"<14>1 - override-host override-app opaque\=pid - - override".as_slice(),
            b"<14>1 - first first-app 11 - - same-scope",
            b"<14>1 - first first-app 11 - - next-scope",
            b"<14>1 - second second-app 22 - - next-resource",
            b"<14>1 - - - - - - no-defaults",
        ]
    );
}

/// Scenario: Invalid resource headers are overridden by valid per-log fields in each syslog profile.
/// Guarantees: Unused resource values cannot reject logs, while invalid fallbacks still fail when selected.
#[test]
fn syslog_encoder_validates_only_resolved_defaults() {
    for format in [
        Format::Rfc3164,
        Format::Rfc5424,
        Format::CefRfc3164,
        Format::CefRfc5424,
    ] {
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
        for (override_key, fallback) in [
            ("syslog.host_name", integer("host.name", 1)),
            ("syslog.host_name", text("host.name", &"x".repeat(256))),
            ("syslog.app_name", text("service.name", &"x".repeat(49))),
            ("syslog.process_id", integer("process.pid", -1)),
        ] {
            let mut input = log("body");
            input.time_unix_nano = 1_704_164_645_000_000_000;
            input.attributes = cef_attributes();
            input.attributes.extend([
                text("syslog.host_name", "host"),
                text("syslog.app_name", "app"),
                integer("syslog.process_id", 42),
            ]);
            let bytes = encoder
                .encode(records(vec![input.clone()], vec![fallback.clone()]))
                .unwrap();
            let decoded = decode(bytes);
            for expected in [
                text("syslog.host_name", "host"),
                text("syslog.app_name", "app"),
                integer("syslog.process_id", 42),
            ] {
                assert!(decoded[0].attributes.contains(&expected));
            }
            input
                .attributes
                .retain(|attribute| attribute.key != override_key);
            assert!(
                encoder
                    .encode(records(vec![input], vec![fallback]))
                    .is_err()
            );
        }
    }
}

/// Scenario: RFC 5424 PROCIDs have leading zeros, a minus sign, or opaque text alongside numeric attributes in either order.
/// Guarantees: The selected string survives parser re-encoding unchanged, unused numeric values are ignored, and invalid selected strings fail.
#[test]
fn syslog_encoder_preserves_opaque_procids() {
    for format in [Format::Rfc5424, Format::CefRfc5424] {
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
        for (process, number) in [("00123", 123), ("-42", -42), ("opaque", -1)] {
            for reverse in [false, true] {
                let mut input = log("body");
                input.attributes = cef_attributes();
                input.attributes.extend([
                    text("syslog.process_id_str", process),
                    integer("syslog.process_id", number),
                ]);
                if reverse {
                    input.attributes.reverse();
                }
                let bytes = encoder.encode(records(vec![input], vec![])).unwrap();
                let prefix = format!("<14>1 - - - {process} - - ");
                assert!(messages(&bytes)[0].starts_with(prefix.as_bytes()));
                let decoded = decode(bytes.clone());
                assert!(
                    decoded[0]
                        .attributes
                        .contains(&text("syslog.process_id_str", process))
                );
                let rewritten = encoder.encode(records(decoded, vec![])).unwrap();
                assert_eq!(messages(&rewritten), messages(&bytes));
            }
        }
        for invalid in [
            text("syslog.process_id_str", "bad value"),
            text("syslog.process_id_str", &"x".repeat(129)),
            integer("syslog.process_id_str", 42),
        ] {
            let mut input = log("body");
            input.attributes = cef_attributes();
            input
                .attributes
                .extend([integer("syslog.process_id", 42), invalid]);
            assert!(encoder.encode(records(vec![input], vec![])).is_err());
        }
    }
}

/// Scenario: Borrowed CEF fields contain signed integer extremes and repeated custom keys in unsorted order.
/// Guarantees: Text fields borrow native storage, decimal formatting is exact, and every occurrence counts toward the extension budget.
#[test]
fn syslog_encoder_borrows_fields_and_formats_integers() {
    let mut input = log("");
    input.attributes = cef_attributes();
    input.attributes.extend([
        integer("z", i64::MAX),
        text("a", "borrowed"),
        integer("m", i64::MIN),
        integer("spt", 0),
    ]);
    let batch = records(vec![input], vec![]);
    let view = OtapLogsView::try_from(&batch).unwrap();
    let resource = view.resources().next().unwrap();
    let scope = resource.scopes().next().unwrap();
    let log = scope.log_records().next().unwrap();
    let attributes = log.attributes().collect::<Vec<_>>();
    let mut fields = CefFields::default();
    for attribute in &attributes {
        let key = attribute.key();
        let value = attribute.value().unwrap();
        fields.apply(key, value).unwrap();
        if key == b"cef.device_vendor" || key == b"a" {
            let OtapAnyValueView::Str(source) = value else {
                panic!("expected native string");
            };
            let borrowed = if key == b"cef.device_vendor" {
                fields.headers[0].unwrap()
            } else {
                let (stored_key, value) = fields.extensions.last().unwrap();
                assert_eq!(stored_key.as_ptr(), key.as_ptr());
                cef_value(*value).unwrap()
            };
            let FieldValue::Text(borrowed) = borrowed else {
                panic!("expected borrowed CEF text");
            };
            assert_eq!(borrowed.as_ptr(), source.as_ptr());
            assert_eq!(borrowed.len(), source.len());
        }
    }
    fields.apply(b"a", OtapAnyValueView::Int(0)).unwrap();
    let mut bytes = Vec::new();
    fields.append(&mut bytes, None).unwrap();
    let extensions: Vec<_> = bytes
        .strip_prefix(b"CEF:0|vendor|product|1.0|100|event|6|")
        .unwrap()
        .split(|byte| *byte == b' ')
        .collect();
    assert_eq!(extensions.len(), 6);
    for expected in [
        b"src=192.0.2.1".as_slice(),
        b"spt=0",
        b"a=borrowed",
        b"a=0".as_slice(),
        b"m=-9223372036854775808",
        b"z=9223372036854775807",
    ] {
        assert!(extensions.contains(&expected));
    }
    let mut fields = CefFields::default();
    fields.apply(b"a", OtapAnyValueView::Int(i64::MIN)).unwrap();
    let text = vec![b'x'; MAX_MESSAGE_SIZE - 1];
    assert!(fields.apply(b"a", OtapAnyValueView::Str(&text)).is_err());
    let text = vec![b'x'; MAX_MESSAGE_SIZE - fields.extension_size - 3];
    fields.apply(b"a", OtapAnyValueView::Str(&text)).unwrap();
    assert!(fields.apply(b"b", OtapAnyValueView::Int(0)).is_err());
}

/// Scenario: Bare CEF records contain invalid resource and syslog header fields that wrapped CEF would use.
/// Guarantees: Lazy resource validation and the shared attribute pass preserve bare CEF's independence from syslog metadata.
#[test]
fn syslog_encoder_bare_cef_ignores_syslog_headers() {
    let mut input = log("body");
    input.attributes = cef_attributes();
    input.attributes.push(integer("syslog.host_name", 42));
    let batch = records(vec![input], vec![integer("host.name", 42)]);
    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Cef);
    let bytes = encoder.encode(batch.clone()).unwrap();
    assert_eq!(
        messages(&bytes),
        [b"CEF:0|vendor|product|1.0|100|event|6|src=192.0.2.1 msg=body".as_slice()]
    );
    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::CefRfc5424);
    assert!(encoder.encode(batch).is_err());
}

/// Scenario: Valid CEF records have OTel severity numbers outside the syslog mapping range.
/// Guarantees: Bare CEF preserves its own severity, while every syslog-wrapped profile rejects the invalid OTel severity.
#[test]
fn syslog_encoder_bare_cef_ignores_otel_severity() {
    for level in [-1, 25] {
        let mut input = log("body");
        input.severity_number = level;
        input.attributes = cef_attributes();
        let batch = records(vec![input], vec![]);
        for format in ALL_FORMATS {
            let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
            let result = encoder.encode(batch.clone());
            if format == Format::Cef {
                assert_eq!(
                    messages(&result.unwrap()),
                    [b"CEF:0|vendor|product|1.0|100|event|6|src=192.0.2.1 msg=body".as_slice()]
                );
            } else {
                assert!(
                    result
                        .unwrap_err()
                        .to_string()
                        .contains("severity_number must be in 0..=24")
                );
            }
        }
    }
}

/// Scenario: Every fixed profile prepares repeated batches and detaches output before another encode.
/// Guarantees: Borrowed output reuses message and batch allocations, framing matches the shared builder, and owned bytes remain independent.
#[test]
fn syslog_encoder_reuses_buffers_and_preserves_owned_output() {
    for format in ALL_FORMATS {
        let mut input = log("body");
        input.time_unix_nano = 1_704_164_645_000_000_000;
        input.observed_time_unix_nano = input.time_unix_nano + 7;
        input.attributes = cef_attributes();
        let mut later = input.clone();
        later.time_unix_nano = 1_735_787_045_000_000_000;
        let mut batch = records(vec![input.clone(), later, input], vec![]);
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
        let output = encoder.prepare_encode(&mut batch).unwrap();
        let pointer = output.as_ref().as_ptr();
        let expected = Bytes::copy_from_slice(output.as_ref());
        let years = match format {
            Format::Rfc3164 | Format::CefRfc3164 => [2024, 2025, 2024],
            _ => [1970; 3],
        };
        let mut builder = SyslogBatchBuilder::new();
        for (message, year) in messages(&expected).into_iter().zip(years) {
            builder.append_with_year(message, year).unwrap();
        }
        assert_eq!(
            builder
                .finish(1_704_164_645_000_000_007)
                .unwrap()
                .into_parts()
                .0,
            expected
        );
        drop(output);
        let message_pointer = encoder.message.as_ptr();
        let output = encoder.prepare_encode(&mut batch).unwrap();
        assert_eq!(output.as_ref().as_ptr(), pointer);
        assert_eq!(output.as_ref(), expected.as_ref());
        let owned = output.into_bytes();
        assert_eq!(encoder.message.as_ptr(), message_pointer);
        let mut different = log("different");
        different.time_unix_nano = 1_704_164_645_000_000_000;
        different.attributes = cef_attributes();
        let output = encoder.encode(records(vec![different], vec![])).unwrap();
        assert_ne!(output, owned);
        assert_eq!(owned, expected);
    }
}

/// Scenario: Large batches succeed or fail after filling reusable storage, followed by a small valid batch.
/// Guarantees: Success and failure bound retained capacity, failed output is cleared, and subsequent output contains no stale frames.
#[test]
fn syslog_encoder_bounds_retained_buffers_and_recovers() {
    let mut encoder = SyslogEncoder::new(EncodePolicy::default());
    let large = records(vec![log(&"x".repeat(8192)); 64], vec![]);
    let output = encoder.prepare_encode(&mut large.clone()).unwrap();
    assert!(output.as_ref().len() > MAX_RETAINED_BUFFER_CAPACITY);
    drop(output);
    assert!(encoder.buffer.capacity() <= MAX_RETAINED_BUFFER_CAPACITY);
    let owned = encoder.encode(large.clone()).unwrap();
    assert!(owned.len() > MAX_RETAINED_BUFFER_CAPACITY);
    assert!(encoder.buffer.capacity() <= MAX_RETAINED_BUFFER_CAPACITY);
    let mut invalid = log("invalid");
    invalid.body = Some(AnyValue::new_int(1));
    let mut inputs = vec![log(&"x".repeat(8192)); 64];
    inputs.push(invalid);
    assert!(
        encoder
            .prepare_encode(&mut records(inputs, vec![]))
            .is_err()
    );
    assert!(encoder.buffer.is_empty());
    assert!(encoder.buffer.capacity() <= MAX_RETAINED_BUFFER_CAPACITY);
    assert!(encoder.message.is_empty());
    let output = encoder.encode(records(vec![log("ok")], vec![])).unwrap();
    assert_eq!(messages(&output), vec![b"<14>1 - - - - - - ok".as_slice()]);
    assert_eq!(messages(&owned).len(), 64);
}

/// Scenario: Pipeline codec bindings select each profile for the same native log fields.
/// Guarantees: Production factories emit each expected wire format, escape CEF values, and decode the chosen format and message.
#[test]
fn syslog_encoder_profiles_use_codec_bindings() {
    let time = 1_704_164_645_123_456_789;
    for profile in ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"] {
        let service = configured_service(&format!("{{profile: {profile}}}")).unwrap();
        let mut input = log("event=1 \\ path\nnext\rline");
        input.time_unix_nano = time;
        input.observed_time_unix_nano = time + 1;
        input.attributes = cef_attributes();
        input.attributes.extend([
            text("syslog.app_name", "app"),
            text("syslog.msg_id", "ID47"),
            integer("syslog.process_id", 42),
            integer("syslog.facility", 16),
            integer("syslog.severity", 6),
            // Input metadata must not override the output profile.
            text("input.format", "unrelated"),
        ]);
        let mut payload = OtapPayload::from(records(vec![input], vec![text("host.name", "host")]));
        let plan = EncodingPlan::resolve(
            service.registry(),
            &PdataEncoding::new("syslog-batch"),
            EncodePolicy::default(),
        )
        .unwrap();
        let bytes = payload.encode_bytes(&service, &plan).unwrap();
        let body = if profile.starts_with("cef") {
            r"CEF:0|vendor|product|1.0|100|event|6|src=192.0.2.1 msg=event\=1 \\ path\nnext\rline"
        } else {
            "event=1 \\ path\nnext\rline"
        };
        let expected = match profile {
            "cef" => body.to_owned(),
            "rfc3164" | "cef_rfc3164" => {
                let local = DateTime::from_timestamp_nanos(time as i64).with_timezone(&Local);
                format!("<134>{} host app[42]: {body}", local.format("%b %e %T"))
            }
            _ => format!("<134>1 2024-01-02T03:04:05.123456Z host app 42 ID47 - {body}"),
        };
        assert_eq!(messages(&bytes), vec![expected.as_bytes()], "{profile}");
        let decoded = decode(bytes);
        assert_eq!(decoded.len(), 1);
        assert_eq!(decoded[0].observed_time_unix_nano, time + 1);
        assert!(
            decoded[0]
                .attributes
                .contains(&text("input.format", profile))
        );
        if profile.starts_with("cef") {
            assert!(
                decoded[0]
                    .attributes
                    .contains(&text("msg", "event=1 \\ path\nnext\rline"))
            );
        }
        match profile {
            "cef" => assert_eq!(decoded[0].time_unix_nano, 0),
            "rfc3164" | "cef_rfc3164" => {
                assert_eq!(
                    decoded[0].time_unix_nano,
                    time / 1_000_000_000 * 1_000_000_000
                );
            }
            _ => assert_eq!(decoded[0].time_unix_nano, time / 1000 * 1000),
        }
    }
}

/// Scenario: Prepared RFC 3164 timezone rules cover winter, summer, pre-epoch dates, and both sides of a DST fold.
/// Guarantees: Cached rules choose the event-date offset and reject repeated local clock times in plain and wrapped CEF output.
#[test]
fn syslog_encoder_cached_timezone_preserves_dst_and_rejects_folds() {
    let timezone = chrono_tz::America::New_York;
    let codec = super::super::resolve_syslog().unwrap();
    for format in [Format::Rfc3164, Format::CefRfc3164] {
        let factory = SyslogFactory {
            profile: Profile::Fixed(format),
            explicit_profile: true,
            timezone: Some(timezone),
        };
        let mut encoder = factory
            .create_encoder(codec, EncodePolicy::default())
            .unwrap();
        for (instant, prefix) in [
            ("2024-01-15T12:00:00Z", "<14>Jan 15 07:00:00"),
            ("2024-07-15T12:00:00Z", "<14>Jul 15 08:00:00"),
            ("1969-07-15T12:00:00Z", "<14>Jul 15 08:00:00"),
        ] {
            let mut input = log("body");
            input.time_unix_nano = DateTime::parse_from_rfc3339(instant)
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap() as u64;
            input.attributes = cef_attributes();
            let bytes = encoder.encode(records(vec![input], vec![])).unwrap();
            let body = if format.is_cef() {
                "CEF:0|vendor|product|1.0|100|event|6|src=192.0.2.1 msg=body"
            } else {
                "body"
            };
            let expected = format!("{prefix} - -: {body}");
            assert_eq!(messages(&bytes), [expected.as_bytes()]);
        }
        for instant in ["2024-11-03T05:30:00Z", "2024-11-03T06:30:00Z"] {
            let mut input = log("body");
            input.time_unix_nano = DateTime::parse_from_rfc3339(instant)
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap() as u64;
            input.attributes = cef_attributes();
            let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
            assert!(
                error
                    .to_string()
                    .contains("RFC 3164 cannot represent an ambiguous local timestamp")
            );
        }
    }
}

/// Scenario: Timezone selection receives IANA names, a named TZ override, custom POSIX rules, file paths, or unknown names.
/// Guarantees: Supported IANA zones resolve explicitly; unsupported timezone configurations never silently fall back.
#[test]
fn syslog_encoder_requires_iana_timezone() {
    for name in ["America/New_York", ":America/New_York"] {
        assert_eq!(
            parse_local_timezone(name).unwrap(),
            chrono_tz::America::New_York
        );
    }
    for name in ["EST5EDT,M3.2.0,M11.1.0", "/etc/localtime", "Unknown/Zone"] {
        let error = parse_local_timezone(name).unwrap_err();
        assert!(
            error
                .to_string()
                .contains("requires a supported IANA timezone")
        );
    }
}

/// Scenario: Parser-style message attributes coexist with absent, empty, or transformed bodies in every output format.
/// Guarantees: Empty bodies preserve attributes, format-specific fields win regardless of order, and nonempty bodies remain authoritative through re-encoding.
#[test]
fn syslog_encoder_uses_parser_message_attributes() {
    let time = 1_704_164_645_000_000_000;
    for format in ALL_FORMATS {
        for (body, replacement) in [
            (None, None),
            (Some(AnyValue::default()), None),
            (Some(AnyValue::new_string("")), None),
            (
                Some(AnyValue::new_string("transformed message")),
                Some("transformed message"),
            ),
        ] {
            for reverse in [false, true] {
                let mut attributes = cef_attributes();
                attributes.extend([
                    text("syslog.content", "RFC 3164 message"),
                    text("syslog.message", "RFC 5424 message"),
                    text("msg", "CEF message"),
                ]);
                if reverse {
                    attributes.reverse();
                }
                let input = LogRecord {
                    body: body.clone(),
                    time_unix_nano: time,
                    attributes,
                    ..Default::default()
                };
                let (key, expected) = match format {
                    Format::Rfc3164 => ("syslog.content", "RFC 3164 message"),
                    Format::Rfc5424 => ("syslog.message", "RFC 5424 message"),
                    _ => ("msg", "CEF message"),
                };
                let expected = replacement.unwrap_or(expected);
                let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
                let decoded = decode(encoder.encode(records(vec![input], vec![])).unwrap());
                assert!(decoded[0].body.is_none());
                assert!(decoded[0].attributes.contains(&text(key, expected)));
                let decoded = decode(encoder.encode(records(decoded, vec![])).unwrap());
                assert!(decoded[0].body.is_none());
                assert!(decoded[0].attributes.contains(&text(key, expected)));
            }
        }
    }
}

/// Scenario: Message fallbacks have an invalid type or size while the body is present, empty, absent, or invalid.
/// Guarantees: Only nonempty string bodies override invalid fallbacks in every profile; selected invalid values still fail.
#[test]
fn syslog_encoder_validates_only_selected_messages() {
    let oversized = "x".repeat(MAX_MESSAGE_SIZE + 1);
    for format in ALL_FORMATS {
        let keys: &[&str] = if format.is_cef() {
            &["msg"]
        } else {
            &["syslog.message", "syslog.content"]
        };
        for key in keys {
            for invalid in [AnyValue::new_bool(true), AnyValue::new_string(&oversized)] {
                for body in [
                    None,
                    Some(AnyValue::default()),
                    Some(AnyValue::new_string("")),
                    Some(AnyValue::new_string("ok")),
                    Some(AnyValue::new_int(42)),
                ] {
                    let mut attributes = cef_attributes();
                    attributes.push(KeyValue::new(*key, invalid.clone()));
                    let input = LogRecord {
                        body: body.clone(),
                        time_unix_nano: 1_704_164_645_000_000_000,
                        attributes,
                        ..Default::default()
                    };
                    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
                    let result = encoder.encode(records(vec![input], vec![]));
                    if body == Some(AnyValue::new_string("ok")) {
                        let decoded = decode(result.unwrap());
                        let output_key = match format {
                            Format::Rfc3164 => "syslog.content",
                            Format::Rfc5424 => "syslog.message",
                            _ => "msg",
                        };
                        assert!(decoded[0].attributes.contains(&text(output_key, "ok")));
                    } else {
                        assert!(result.is_err(), "{key} with body {body:?}");
                    }
                }
            }
        }
    }
}

/// Scenario: Invalid RFC 5424-only attributes accompany valid logs under each syslog wrapper.
/// Guarantees: RFC 3164 ignores unused message IDs and versions, while RFC 5424 rejects invalid selected fields.
#[test]
fn syslog_encoder_ignores_unused_rfc5424_headers() {
    for format in [
        Format::Rfc3164,
        Format::CefRfc3164,
        Format::Rfc5424,
        Format::CefRfc5424,
    ] {
        for invalid in [
            integer("syslog.msg_id", 42),
            text("syslog.msg_id", ""),
            text("syslog.version", "invalid"),
            integer("syslog.version", 256),
        ] {
            let mut input = log("ok");
            input.time_unix_nano = 1_704_164_645_000_000_000;
            input.attributes = cef_attributes();
            input.attributes.push(invalid.clone());
            let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
            let result = encoder.encode(records(vec![input], vec![]));
            if matches!(format, Format::Rfc3164 | Format::CefRfc3164) {
                let decoded = decode(result.unwrap());
                let key = if format.is_cef() {
                    "msg"
                } else {
                    "syslog.content"
                };
                assert!(decoded[0].attributes.contains(&text(key, "ok")));
            } else {
                assert!(result.is_err(), "invalid selected header: {invalid:?}");
            }
        }
    }
}

/// Scenario: Repeated CEF msg attributes accompany a large extension and an optional body override.
/// Guarantees: Discarded msg occurrences cannot exhaust the output budget; selected oversized extensions still fail.
#[test]
fn syslog_encoder_ignores_overridden_cef_msg_budget() {
    let large = "x".repeat(16_000);
    for format in [Format::Cef, Format::CefRfc3164, Format::CefRfc5424] {
        for body in [None, Some(""), Some("ok")] {
            let mut input = log("");
            input.body = body.map(AnyValue::new_string);
            input.time_unix_nano = 1_704_164_645_000_000_000;
            input.attributes = cef_attributes();
            input.attributes.push(text("vendorZone", &large));
            input.attributes.extend(vec![text("msg", "discarded"); 80]);
            let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
            let result = encoder.encode(records(vec![input], vec![]));
            if body == Some("ok") {
                let encoded = result.unwrap();
                assert!(messages(&encoded)[0].len() <= MAX_MESSAGE_SIZE);
                let decoded = decode(encoded);
                let msg: Vec<_> = decoded[0]
                    .attributes
                    .iter()
                    .filter(|attribute| attribute.key == "msg")
                    .collect();
                assert_eq!(msg, vec![&text("msg", "ok")]);
                assert!(decoded[0].attributes.contains(&text("vendorZone", &large)));
            } else {
                assert!(
                    result.is_err(),
                    "selected msg attributes with body {body:?}"
                );
            }
        }
    }
}

/// Scenario: A valid format-specific syslog message coexists with an invalid lower-priority fallback.
/// Guarantees: Fallback precedence is order-independent, and only the selected attribute is validated.
#[test]
fn syslog_encoder_validates_only_preferred_message_attribute() {
    for (format, preferred, fallback) in [
        (Format::Rfc3164, "syslog.content", "syslog.message"),
        (Format::Rfc5424, "syslog.message", "syslog.content"),
    ] {
        for reverse in [false, true] {
            for body in [None, Some(AnyValue::new_string(""))] {
                for invalid_preferred in [false, true] {
                    let mut attributes = vec![
                        text(
                            if invalid_preferred {
                                fallback
                            } else {
                                preferred
                            },
                            "ok",
                        ),
                        KeyValue::new(
                            if invalid_preferred {
                                preferred
                            } else {
                                fallback
                            },
                            AnyValue::new_bool(true),
                        ),
                    ];
                    if reverse {
                        attributes.reverse();
                    }
                    let input = LogRecord {
                        body: body.clone(),
                        time_unix_nano: 1_704_164_645_000_000_000,
                        attributes,
                        ..Default::default()
                    };
                    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
                    let result = encoder.encode(records(vec![input], vec![]));
                    if invalid_preferred {
                        assert!(result.is_err());
                    } else {
                        let decoded = decode(result.unwrap());
                        assert!(decoded[0].attributes.contains(&text(preferred, "ok")));
                    }
                }
            }
        }
    }
}

/// Scenario: An explicitly selected profile receives already encoded syslog in another format.
/// Guarantees: Explicit output settings rewrite compatible envelopes instead of forwarding the old wire format.
#[test]
fn syslog_encoder_profile_rewrites_matching_encoding() {
    let service = configured_service("{profile: rfc5424}").unwrap();
    let codec = super::super::resolve_syslog().unwrap();
    let mut builder = SyslogBatchBuilder::new();
    builder
        .append_with_year(b"<134>Jun 15 12:00:00 host app[42]: original", 2024)
        .unwrap();
    let mut payload: OtapPayload = codec.admit_built(builder.finish(123).unwrap()).unwrap();
    let plan = EncodingPlan::resolve(
        service.registry(),
        &PdataEncoding::new("syslog-batch"),
        EncodePolicy::default(),
    )
    .unwrap();
    let rewritten = payload.encode_bytes(&service, &plan).unwrap();
    assert!(messages(&rewritten)[0].starts_with(b"<134>1 "));
    let decoded = decode(rewritten);
    assert!(
        decoded[0]
            .attributes
            .contains(&text("syslog.message", "original"))
    );
}

/// Scenario: RFC 5424 and wrapped CEF encode pre-epoch event timestamps, including fractional seconds.
/// Guarantees: Negative native timestamps retain their date and round down to wire microsecond precision.
#[test]
fn syslog_encoder_rfc5424_preserves_pre_epoch_timestamps() {
    for (nanos, timestamp) in [
        (-1_000_000_000_i64, "1969-12-31T23:59:59.000000Z"),
        (-1_500_000_001, "1969-12-31T23:59:58.499999Z"),
    ] {
        for format in [Format::Rfc5424, Format::CefRfc5424] {
            let mut input = log("body");
            input.time_unix_nano = nanos as u64;
            input.attributes = cef_attributes();
            let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
            let bytes = encoder.encode(records(vec![input], vec![])).unwrap();
            let expected_prefix = format!("<14>1 {timestamp} - - - - - ");
            assert!(messages(&bytes)[0].starts_with(expected_prefix.as_bytes()));
            assert_eq!(
                decode(bytes)[0].time_unix_nano as i64,
                nanos.div_euclid(1000) * 1000
            );
        }
    }
}

/// Scenario: RFC 3164 logs cross years, including pre-epoch dates, and output budgets include each receipt-year run.
/// Guarantees: Signed timestamps and encoded years survive deferred decoding, second precision is explicit, and the complete framed-byte limit is enforced.
#[test]
fn syslog_encoder_rfc3164_preserves_year_runs_and_limits() {
    let times: Vec<_> = [1969, 2024, 2025, 1969]
        .map(|year| {
            Utc.with_ymd_and_hms(year, 6, 15, 12, 0, 0)
                .unwrap()
                .timestamp_nanos_opt()
                .unwrap() as u64
        })
        .into();
    let inputs: Vec<_> = times
        .iter()
        .map(|time| LogRecord {
            time_unix_nano: *time,
            ..log("message")
        })
        .collect();
    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Rfc3164);
    let bytes = encoder.encode(records(inputs.clone(), vec![])).unwrap();
    assert_eq!(messages(&bytes).len(), times.len());
    let decoded = decode(bytes.clone());
    assert_eq!(
        decoded
            .iter()
            .map(|log| log.time_unix_nano)
            .collect::<Vec<_>>(),
        times
    );
    encoder.max_bytes = bytes.len() - 1;
    assert!(matches!(
        encoder.encode(records(inputs.clone(), vec![])),
        Err(CodecError::EncodedSizeLimitExceeded { .. })
    ));
    encoder.max_bytes += 1;
    assert_eq!(encoder.encode(records(inputs, vec![])).unwrap(), bytes);
}

/// Scenario: Observed timestamps repeat, become negative, or are absent within the same year, including explicit profile rewrites.
/// Guarantees: Every profile preserves signed per-record observed time, coalesces only adjacent equal contexts, and budgets all run headers exactly.
#[test]
fn syslog_encoder_preserves_observed_runs() {
    let observed_times = [100_i64, 100, -7, -7, 0, 100];
    for profile in ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"] {
        let inputs: Vec<_> = observed_times
            .iter()
            .map(|observed| LogRecord {
                time_unix_nano: 1_704_164_645_000_000_000,
                observed_time_unix_nano: *observed as u64,
                attributes: cef_attributes(),
                ..log("body")
            })
            .collect();
        let mut encoder =
            SyslogEncoder::for_format(EncodePolicy::default(), Format::parse(profile).unwrap());
        let bytes = encoder.encode(records(inputs.clone(), vec![])).unwrap();
        let decoded = decode(bytes.clone());
        assert_eq!(
            decoded
                .iter()
                .map(|log| log.observed_time_unix_nano as i64)
                .collect::<Vec<_>>(),
            observed_times,
            "{profile}"
        );
        let expected_size = BATCH_HEADER_LEN
            + 4 * RUN_HEADER_LEN
            + messages(&bytes)
                .iter()
                .map(|message| FRAME_HEADER_LEN + message.len())
                .sum::<usize>();
        assert_eq!(bytes.len(), expected_size);
        encoder.max_bytes = expected_size - 1;
        assert!(matches!(
            encoder.encode(records(inputs.clone(), vec![])),
            Err(CodecError::EncodedSizeLimitExceeded { actual, limit, .. })
                if actual == expected_size && limit == expected_size - 1
        ));
        encoder.max_bytes = expected_size;
        assert_eq!(encoder.encode(records(inputs, vec![])).unwrap(), bytes);

        let service = configured_service(&format!("{{profile: {profile}}}")).unwrap();
        let plan = EncodingPlan::resolve(
            service.registry(),
            &PdataEncoding::new("syslog-batch"),
            EncodePolicy::default(),
        )
        .unwrap();
        let mut payload = OtapPayload::from(plan.codec().admit(SignalType::Logs, bytes).unwrap());
        let rewritten = payload.encode_bytes(&service, &plan).unwrap();
        assert_eq!(
            decode(rewritten)
                .iter()
                .map(|log| log.observed_time_unix_nano as i64)
                .collect::<Vec<_>>(),
            observed_times,
            "{profile}"
        );
    }
}

/// Scenario: Profile options or mapped CEF/RFC 3164 fields are invalid, missing, or oversized after escaping.
/// Guarantees: Invalid requests fail explicitly, CEF headers escape delimiters, and errors do not poison encoder reuse.
#[test]
fn syslog_encoder_profiles_reject_invalid_fields_and_options() {
    for options in ["{profile: PRIVATE}", "{profile: 1}", "{unknown: true}"] {
        let error = configured_service(options).err().unwrap().to_string();
        assert!(!error.contains("PRIVATE"));
    }
    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Cef);
    assert!(
        encoder
            .encode(records(vec![log("message")], vec![]))
            .is_err()
    );
    for attribute in [
        integer("cef.version", 2),
        text("cef.severity", "11"),
        text("cef.name", "PRIVATE\ninjected"),
        text("cef.name", ""),
    ] {
        let mut input = log("PRIVATE");
        input.attributes = cef_attributes();
        input
            .attributes
            .retain(|existing| existing.key != attribute.key);
        input.attributes.push(attribute);
        let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
        assert!(!error.to_string().contains("PRIVATE"));
    }
    let mut input = log(&"\\".repeat(MAX_MESSAGE_SIZE / 2));
    input.attributes = cef_attributes();
    assert!(encoder.encode(records(vec![input], vec![])).is_err());
    let mut input = log("ok");
    input.attributes = cef_attributes();
    input.attributes.retain(|attr| attr.key != "cef.name");
    input.attributes.push(text("cef.name", "pipe|slash\\"));
    let bytes = encoder.encode(records(vec![input], vec![])).unwrap();
    assert_eq!(
        messages(&bytes),
        vec![br"CEF:0|vendor|product|1.0|100|pipe\|slash\\|6|src=192.0.2.1 msg=ok".as_slice()]
    );
    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Rfc3164);
    assert!(
        encoder
            .encode(records(vec![log("missing timestamp")], vec![]))
            .is_err()
    );
    let mut input = log("message");
    input.time_unix_nano = 1_704_164_645_000_000_000;
    input.attributes = vec![text("syslog.app_name", "injected:tag")];
    assert!(encoder.encode(records(vec![input], vec![])).is_err());
}

/// Scenario: Vendor CEF extensions contain delimiters, invalid keys, or excessive escaped output.
/// Guarantees: Bare receiver-style fields round-trip with escaping, unrelated namespaced attributes stay private, and invalid or oversized extensions fail without poisoning reuse.
#[test]
fn syslog_encoder_vendor_extensions_use_receiver_names_and_are_bounded() {
    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Cef);
    for key in ["", "bad key", "bad=key", "cef.extension.msg"] {
        let mut input = log("body");
        input.attributes = cef_attributes();
        input.attributes.push(text(key, "PRIVATE"));
        let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
        assert!(!error.to_string().contains("PRIVATE"));
    }
    for value in [
        "x".repeat(MAX_MESSAGE_SIZE + 1),
        "x".repeat(MAX_MESSAGE_SIZE),
        "\\".repeat(MAX_MESSAGE_SIZE / 2),
    ] {
        let mut input = log("body");
        input.attributes = cef_attributes();
        input.attributes.push(text("vendorField", &value));
        let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
        assert!(
            error
                .to_string()
                .contains(&format!("{MAX_MESSAGE_SIZE} bytes"))
        );
    }
    let mut input = log("body");
    input.attributes = cef_attributes();
    input.attributes.extend([
        text("cs1Label", "PolicyName"),
        text("cs1", "rule=web\\allow\nnext\rline"),
        integer("cn1", 42),
        text("vendorZone", "trusted"),
        text("private.attribute", "PRIVATE"),
    ]);
    let bytes = encoder.encode(records(vec![input], vec![])).unwrap();
    let wire_messages = messages(&bytes);
    assert_eq!(wire_messages.len(), 1);
    let extensions: Vec<_> = wire_messages[0]
        .strip_prefix(b"CEF:0|vendor|product|1.0|100|event|6|")
        .unwrap()
        .split(|byte| *byte == b' ')
        .collect();
    assert_eq!(extensions.len(), 6);
    for expected in [
        b"src=192.0.2.1".as_slice(),
        b"msg=body",
        b"cn1=42".as_slice(),
        br"cs1=rule\=web\\allow\nnext\rline",
        b"cs1Label=PolicyName",
        b"vendorZone=trusted",
    ] {
        assert!(extensions.contains(&expected));
    }
    let mut decoded = decode(bytes);
    for _ in 0..3 {
        assert!(
            decoded[0]
                .attributes
                .contains(&text("cs1", "rule=web\\allow\nnext\rline"))
        );
        assert!(decoded[0].attributes.contains(&text("cn1", "42")));
        assert!(
            decoded[0]
                .attributes
                .contains(&text("vendorZone", "trusted"))
        );
        assert!(
            decoded[0]
                .attributes
                .contains(&text("cs1Label", "PolicyName"))
        );
        let bytes = encoder.encode(records(decoded, vec![])).unwrap();
        decoded = decode(bytes);
    }
}

/// Scenario: The all-profile codec consumes uneven batches, encounters a mid-batch failure, and is instantiated in another service.
/// Guarantees: Records rotate across all five formats across successful batches, failed batches leave the cursor unchanged, and instances rotate independently.
#[test]
fn syslog_encoder_all_profiles_round_robin_across_batches() {
    let input = || {
        let mut input = log("body");
        input.time_unix_nano = 1_704_164_645_000_000_000;
        input.attributes = cef_attributes();
        input.attributes.extend([
            text("syslog.host_name", "host"),
            text("syslog.app_name", "app"),
            integer("syslog.process_id", 42),
        ]);
        input
    };
    let service = configured_service("{profile: all}").unwrap();
    let plan = EncodingPlan::resolve(
        service.registry(),
        &PdataEncoding::new("syslog-batch"),
        EncodePolicy::default(),
    )
    .unwrap();
    let mut actual = Vec::new();
    for size in [2, 1, 4, 3] {
        if actual.len() == 2 {
            let mut invalid = input();
            invalid.attributes.clear();
            let mut payload = OtapPayload::from(records(vec![input(), invalid], vec![]));
            assert!(payload.encode_bytes(&service, &plan).is_err());
        }
        let mut payload = OtapPayload::from(records((0..size).map(|_| input()).collect(), vec![]));
        let bytes = payload.encode_bytes(&service, &plan).unwrap();
        for record in decode(bytes) {
            let attribute = record
                .attributes
                .iter()
                .find(|attribute| attribute.key == "input.format")
                .unwrap();
            actual.push(attribute.value.clone().unwrap());
        }
    }
    let expected = ["rfc3164", "rfc5424", "cef", "cef_rfc3164", "cef_rfc5424"];
    assert_eq!(
        actual,
        expected
            .into_iter()
            .cycle()
            .take(10)
            .map(AnyValue::new_string)
            .collect::<Vec<_>>()
    );
    let independent = configured_service("{profile: all}").unwrap();
    let independent_plan = EncodingPlan::resolve(
        independent.registry(),
        &PdataEncoding::new("syslog-batch"),
        EncodePolicy::default(),
    )
    .unwrap();
    let mut payload = OtapPayload::from(records(vec![input()], vec![]));
    let decoded = decode(
        payload
            .encode_bytes(&independent, &independent_plan)
            .unwrap(),
    );
    assert!(
        decoded[0]
            .attributes
            .contains(&text("input.format", "rfc3164"))
    );
}

/// Scenario: A native log has resource identity, explicit syslog overrides, and nanosecond time.
/// Guarantees: The registered encoder emits the expected RFC 5424 bytes and a decodable envelope.
#[test]
fn syslog_encoder_maps_native_fields_and_decodes() {
    let time = 1_704_164_645_123_456_789;
    let mut input = log("request completed");
    input.time_unix_nano = time;
    input.observed_time_unix_nano = time + 1;
    input.severity_number = 17;
    input.attributes = vec![
        text("syslog.host_name", "edge"),
        text("syslog.msg_id", "event"),
        integer("syslog.facility", 20),
        integer("syslog.severity", 6),
    ];
    let mut records = records(
        vec![input],
        vec![
            text("host.name", "fallback"),
            text("service.name", "app"),
            integer("process.pid", 42),
        ],
    );
    records.encode_transport_optimized().unwrap();
    let codecs = CodecService::new().unwrap();
    let plan = EncodingPlan::resolve(
        codecs.registry(),
        &PdataEncoding::new("syslog-batch"),
        EncodePolicy::default(),
    )
    .unwrap();
    let bytes = codecs.encode_bytes(&mut records, &plan).unwrap();
    assert_eq!(
        messages(&bytes),
        vec![
            b"<166>1 2024-01-02T03:04:05.123456Z edge app 42 event - request completed".as_slice()
        ]
    );
    let mut payload: OtapPayload = plan.codec().admit(SignalType::Logs, bytes).unwrap().into();
    assert_eq!(payload.num_items(), 1);
    let mut decoded = payload.try_into_otap(&codecs).unwrap();
    decoded.decode_transport_optimized_ids().unwrap();
    let view = OtapLogsView::try_from(&decoded).unwrap();
    let resource = view.resources().next().unwrap();
    let scope = resource.scopes().next().unwrap();
    let log = scope.log_records().next().unwrap();
    assert_eq!(log.time_unix_nano(), Some(time / 1000 * 1000));
    assert_eq!(log.observed_time_unix_nano(), Some(time + 1));
    assert!(log.attributes().any(|attribute| {
        attribute.key() == b"syslog.message"
            && attribute.value().unwrap().as_string() == Some(b"request completed")
    }));
}

/// Scenario: Logs omit headers and bodies or use standard OTel severity levels.
/// Guarantees: Missing values become NILVALUE, syslog message attributes remain usable, and levels map explicitly.
#[test]
fn syslog_encoder_handles_missing_fields_and_severity() {
    let mut encoder = SyslogEncoder::new(EncodePolicy::default());
    let mut inputs = vec![LogRecord::default()];
    for level in [1, 5, 9, 13, 17, 21] {
        inputs.push(LogRecord {
            severity_number: level,
            body: Some(AnyValue::new_string("message")),
            ..Default::default()
        });
    }
    inputs.push(LogRecord {
        attributes: vec![text("syslog.message", "forwarded")],
        ..Default::default()
    });
    let bytes = encoder.encode(records(inputs, vec![])).unwrap();
    assert_eq!(
        messages(&bytes),
        vec![
            b"<14>1 - - - - - -".as_slice(),
            b"<15>1 - - - - - - message",
            b"<15>1 - - - - - - message",
            b"<14>1 - - - - - - message",
            b"<12>1 - - - - - - message",
            b"<11>1 - - - - - - message",
            b"<10>1 - - - - - - message",
            b"<14>1 - - - - - - forwarded",
        ]
    );
}

/// Scenario: Inputs exceed message, item, batch-byte, or output-policy limits.
/// Guarantees: No oversized batch succeeds, diagnostics report the enforced limits, exact message boundaries work, and failures do not poison reuse.
#[test]
fn syslog_encoder_enforces_bounds_and_recovers() {
    let mut encoder = SyslogEncoder::new(EncodePolicy::default());
    let prefix = b"<14>1 - - - - - - ";
    let body = "x".repeat(MAX_MESSAGE_SIZE - prefix.len());
    let bytes = encoder.encode(records(vec![log(&body)], vec![])).unwrap();
    assert_eq!(messages(&bytes)[0].len(), MAX_MESSAGE_SIZE);
    let error = encoder
        .encode(records(vec![log(&(body.clone() + "x"))], vec![]))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains(&format!("{MAX_MESSAGE_SIZE} bytes"))
    );
    assert!(encoder.encode(records(vec![], vec![])).is_err());
    let error = encoder
        .encode(records(vec![log("x"); MAX_BATCH_ITEMS + 1], vec![]))
        .unwrap_err();
    assert!(
        error
            .to_string()
            .contains(&format!("1..={MAX_BATCH_ITEMS} logs"))
    );
    assert!(
        encoder
            .encode(records(vec![log(&body); 128], vec![]))
            .is_err()
    );
    let mut limited = SyslogEncoder::new(EncodePolicy {
        max_encoded_size: std::num::NonZeroUsize::new(32),
    });
    assert!(matches!(
        limited.encode(records(vec![log("x")], vec![])),
        Err(CodecError::EncodedSizeLimitExceeded { limit: 32, .. })
    ));
    let bytes = encoder.encode(records(vec![log("ok")], vec![])).unwrap();
    assert_eq!(messages(&bytes), vec![b"<14>1 - - - - - - ok".as_slice()]);
}

/// Scenario: Mapped headers contain invalid types, ranges, separators, or unsupported structured data.
/// Guarantees: Invalid inputs fail explicitly without truncation, content injection, or telemetry in diagnostics.
#[test]
fn syslog_encoder_rejects_unrepresentable_fields() {
    let mut encoder = SyslogEncoder::new(EncodePolicy::default());
    for attribute in [
        text("syslog.host_name", "PRIVATE\ninjected"),
        text("syslog.app_name", &"x".repeat(49)),
        integer("syslog.facility", 24),
        integer("syslog.severity", -1),
        text("syslog.severity", "PRIVATE"),
        text("syslog.structured_data", "[PRIVATE] message"),
    ] {
        let mut input = log("PRIVATE");
        input.attributes.push(attribute);
        let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
        assert!(!error.to_string().contains("PRIVATE"));
    }
    let mut input = log("unused");
    input.body = Some(AnyValue::new_int(42));
    assert!(encoder.encode(records(vec![input], vec![])).is_err());
    let mut input = log("message");
    input.severity_number = 25;
    assert!(encoder.encode(records(vec![input], vec![])).is_err());
}

mod round_trip;
