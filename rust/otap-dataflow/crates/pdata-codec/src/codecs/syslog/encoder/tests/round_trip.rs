// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Receiver-materialized round-trip corpus and semantic parity regressions.

use super::*;

fn received(message: &[u8]) -> OtapArrowRecords {
    let mut builder = SyslogBatchBuilder::new();
    builder.append_with_year(message, 2024).unwrap();
    let codec = super::super::super::resolve_syslog().unwrap();
    codec
        .admit_built(builder.finish(123).unwrap())
        .unwrap()
        .try_into_otap(&CodecService::new().unwrap())
        .unwrap()
}

fn semantic_attributes(log: &LogRecord, cef_only: bool) -> Vec<KeyValue> {
    use prost::Message;
    let mut attributes: Vec<_> = log
        .attributes
        .iter()
        .filter(|attribute| {
            attribute.key != "input.format"
                && attribute.key != "syslog.message"
                && attribute.key != "syslog.content"
                && (!cef_only || !attribute.key.starts_with("syslog."))
        })
        .cloned()
        .collect();
    attributes.sort_by(|left, right| {
        left.key.cmp(&right.key).then_with(|| {
            left.value
                .as_ref()
                .map(Message::encode_to_vec)
                .cmp(&right.value.as_ref().map(Message::encode_to_vec))
        })
    });
    attributes
}

fn attribute_occurrences<'a>(log: &'a LogRecord, key: &str) -> Vec<&'a KeyValue> {
    log.attributes
        .iter()
        .filter(|attribute| attribute.key == key)
        .collect()
}

/// Scenario: Real bare and syslog-wrapped CEF inputs pass through receiver framing, native OTAP, and each output profile repeatedly.
/// Guarantees: Header values and all extension names and values survive; default syslog output retains CEF, and yearless bare CEF fails explicitly for RFC 3164.
#[test]
fn syslog_encoder_received_cef_round_trips() {
    for cef in [
        br"CEF:1|v\|end\\|p\\path|1\|0|i\\d|event\|path\\end|High|src=192.0.2.1 dpt=443 cs1=rule\=web\\allow\nnext\rline cs1Label=Policy Name cn1=42 vendor-zone=trusted 7_slot=value".as_slice(),
        b"CEF:0|vendor|product|1.0|100|event|6|".as_slice(),
        b"CEF:0||||||unclassified|vendorField=value".as_slice(),
        br"CEF:0|vendor|product|1.0|100|event|6|src=first cs1=one msg=alpha src=second cs1=two cs1=two msg=beta src= cs1= msg=".as_slice(),
        b"CEF:0|vendor|product|1.0|100|event|6|z=first msg=last  ".as_slice(),
        b"CEF:0|vendor|product|1.0|100|event|6|z=first src=last  ".as_slice(),
        b"CEF:0|vendor|product|1.0|100|event|6|z=first cs1=last  ".as_slice(),
        b"CEF:0|vendor|product|1.0|100|event|6|z=first cs1=  ".as_slice(),
        b"CEF:0|vendor|product|1.0|100|event|6|cs1=early z=first cs1=last  ".as_slice(),
        b"CEF:0|vendor|product|1.0|100|event|6|cs1=tab\t cs2=vertical\x0b cs3=form\x0c msg=key\\=value slash\\\\\\nnext\\rline".as_slice(),
    ] {
        for wrapper in [
            "",
            "<134>Jun 15 12:00:00 host app[0042]: ",
            "<134>1 2024-06-15T12:00:00Z host app 0042 ID47 - ",
        ] {
            let wire = [wrapper.as_bytes(), cef].concat();
            let native = received(&wire);
            let original = decode({
                let mut builder = SyslogBatchBuilder::new();
                builder.append_with_year(&wire, 2024).unwrap();
                builder.finish(123).unwrap().into_parts().0
            });
            let expected = semantic_attributes(&original[0], true);
            if cef.starts_with(b"CEF:1|") {
                assert!(expected.contains(&text("cef.device_vendor", "v|end\\")));
                assert!(expected.contains(&text("cef.name", "event|path\\end")));
                assert!(expected.contains(&text("cs1", "rule=web\\allow\nnext\rline")));
            }
            if cef.windows(b"src=first".len()).any(|part| part == b"src=first") {
                for (key, values) in [
                    ("src", ["first", "second", ""].as_slice()),
                    ("cs1", ["one", "two", "two", ""].as_slice()),
                    ("msg", ["alpha", "beta", ""].as_slice()),
                ] {
                    let expected_values = values.iter().map(|value| text(key, value)).collect::<Vec<_>>();
                    assert_eq!(
                        attribute_occurrences(&original[0], key),
                        expected_values.iter().collect::<Vec<_>>()
                    );
                }
            }
            for key in ["src", "cs1", "msg"] {
                if cef.ends_with(format!("{key}=last  ").as_bytes()) {
                    assert!(expected.contains(&text(key, "last  ")));
                }
            }
            if cef.ends_with(b"cs1=  ") {
                assert!(expected.contains(&text("cs1", "  ")));
            }
            if cef.windows(4).any(|part| part == b"tab\t") {
                for (key, value) in [
                    ("cs1", "tab\t"),
                    ("cs2", "vertical\x0b"),
                    ("cs3", "form\x0c"),
                    ("msg", "key=value slash\\\nnext\rline"),
                ] {
                    assert!(expected.contains(&text(key, value)));
                }
            }
            for profile in [
                None,
                Some("rfc5424"),
                Some("cef"),
                Some("cef_rfc5424"),
                Some("rfc3164"),
                Some("cef_rfc3164"),
            ] {
                let options = profile.map_or_else(|| "{}".to_owned(), |profile| format!("{{profile: {profile}}}"));
                let service = configured_service(&options).unwrap();
                let plan = EncodingPlan::resolve(
                    service.registry(),
                    &PdataEncoding::new("syslog-batch"),
                    EncodePolicy::default(),
                ).unwrap();
                let mut input = native.clone();
                if wrapper.is_empty() && matches!(profile, Some("rfc3164" | "cef_rfc3164")) {
                    assert!(
                        service.encode_bytes(&mut input, &plan)
                            .unwrap_err()
                            .to_string()
                            .contains("requires an event timestamp")
                    );
                    continue;
                }
                for _ in 0..3 {
                    let bytes = service.encode_bytes(&mut input, &plan).unwrap();
                    let logs = decode(bytes.clone());
                    assert_eq!(semantic_attributes(&logs[0], true), expected, "{profile:?}: {wrapper}");
                    for attribute in &expected {
                        assert_eq!(
                            attribute_occurrences(&logs[0], &attribute.key),
                            attribute_occurrences(&original[0], &attribute.key)
                        );
                    }
                    let same_wrapper = (wrapper.starts_with("<134>1") && matches!(profile, None | Some("rfc5424" | "cef_rfc5424")))
                        || (wrapper.starts_with("<134>Jun") && matches!(profile, Some("rfc3164" | "cef_rfc3164")));
                    if same_wrapper {
                        assert_eq!(
                            semantic_attributes(&logs[0], false),
                            semantic_attributes(&original[0], false),
                            "{profile:?}: {wrapper}"
                        );
                        assert_eq!(logs[0].time_unix_nano, original[0].time_unix_nano);
                        assert_eq!(logs[0].severity_number, original[0].severity_number);
                        assert_eq!(logs[0].severity_text, original[0].severity_text);
                    }
                    assert_eq!(logs[0].observed_time_unix_nano, 123);
                    assert!(logs[0].body.is_none());
                    let payload = OtapPayload::from(plan.codec().admit(SignalType::Logs, bytes).unwrap());
                    input = payload.try_into_otap(&service).unwrap();
                }
            }
        }
    }
}

/// Scenario: Received CEF is transformed or contains an extension key that cannot be serialized without changing its meaning.
/// Guarantees: Body edits replace only msg, invalid extension keys fail without leaking telemetry, and encoder reuse remains valid.
#[test]
fn syslog_encoder_received_cef_transforms_and_errors() {
    let service = configured_service("{}").unwrap();
    let plan = EncodingPlan::resolve(
        service.registry(),
        &PdataEncoding::new("syslog-batch"),
        EncodePolicy::default(),
    )
    .unwrap();
    let original =
        b"CEF:0|vendor|product|1.0|100|event|6|cs1=original msg=old cs1=second msg=older";
    let mut builder = SyslogBatchBuilder::new();
    builder.append_with_year(original, 2024).unwrap();
    let mut input = decode(builder.finish(123).unwrap().into_parts().0);
    input[0].body = Some(AnyValue::new_string("edited=message"));
    let output = service
        .encode_bytes(&mut records(input, vec![]), &plan)
        .unwrap();
    let logs = decode(output);
    assert_eq!(
        attribute_occurrences(&logs[0], "msg"),
        vec![&text("msg", "edited=message")]
    );
    assert_eq!(
        attribute_occurrences(&logs[0], "cs1"),
        vec![&text("cs1", "original"), &text("cs1", "second")]
    );
    for key in ["bad key", "syslog.unknown", "cef.unknown"] {
        let wire = format!("CEF:0|vendor|product|1.0|100|event|6|{key}=PRIVATE");
        let mut invalid = received(wire.as_bytes());
        let error = service.encode_bytes(&mut invalid, &plan).unwrap_err();
        assert!(!error.to_string().contains("PRIVATE"));
    }
    let output = service
        .encode_bytes(&mut received(original), &plan)
        .unwrap();
    assert!(
        decode(output)[0]
            .attributes
            .contains(&text("cs1", "original"))
    );
}

/// Scenario: Generated CEF repeats standard, custom, and msg attributes with absent, empty, or overriding bodies.
/// Guarantees: Every occurrence survives receiver materialization unless a nonempty body intentionally replaces only msg.
#[test]
fn syslog_encoder_generated_cef_preserves_occurrences() {
    for body in [None, Some(""), Some("edited  ")] {
        let mut input = LogRecord {
            body: body.map(AnyValue::new_string),
            attributes: cef_attributes(),
            ..Default::default()
        };
        input.attributes.extend([
            text("src", "second"),
            text("cs1", "one"),
            text("cs1", "two"),
            text("msg", "first"),
            text("msg", "second"),
        ]);
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Cef);
        let mut decoded = decode(encoder.encode(records(vec![input], vec![])).unwrap());
        for _ in 0..3 {
            assert_eq!(
                attribute_occurrences(&decoded[0], "src"),
                vec![&text("src", "192.0.2.1"), &text("src", "second")]
            );
            assert_eq!(
                attribute_occurrences(&decoded[0], "cs1"),
                vec![&text("cs1", "one"), &text("cs1", "two")]
            );
            let expected_msg = match body.filter(|body| !body.is_empty()) {
                Some(body) => vec![text("msg", body)],
                None => vec![text("msg", "first"), text("msg", "second")],
            };
            assert_eq!(
                attribute_occurrences(&decoded[0], "msg"),
                expected_msg.iter().collect::<Vec<_>>()
            );
            decoded = decode(encoder.encode(records(decoded, vec![])).unwrap());
        }
    }
}

/// Scenario: Generated CEF has multiple space-ending values or a space-ending value before another occurrence of the same key.
/// Guarantees: Unrepresentable whitespace and duplicate order fail explicitly without exposing values or poisoning encoder reuse.
#[test]
fn syslog_encoder_cef_rejects_unrepresentable_trailing_spaces() {
    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Cef);
    for extensions in [
        vec![text("src", "PRIVATE  "), text("cs1", "other ")],
        vec![text("cs1", "PRIVATE  "), text("cs2", "  ")],
        vec![text("msg", "PRIVATE  "), text("cs1", "other ")],
        vec![text("cs1", "PRIVATE  "), text("cs1", "later")],
    ] {
        let mut input = LogRecord {
            attributes: cef_attributes(),
            ..Default::default()
        };
        input.attributes.extend(extensions);
        let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
        assert!(error.to_string().contains("CEF cannot preserve"));
        assert!(!error.to_string().contains("PRIVATE"));
    }
    let mut input = log("");
    input.attributes = cef_attributes();
    input.attributes.push(text("cs1", "last  "));
    let output = decode(encoder.encode(records(vec![input], vec![])).unwrap());
    assert!(output[0].attributes.contains(&text("cs1", "last  ")));
}

/// Scenario: RFC 5424 and wrapped CEF carry structured data and a nondefault version, while RFC 3164 carries an opaque or zero-padded tag.
/// Guarantees: Same-format rewrites preserve receiver fields, and conversions reject structured data they cannot represent.
#[test]
fn syslog_encoder_received_syslog_headers_round_trip() {
    let sd = r#"[first k="escaped \"quote\]slash\\"][second k="value"]"#;
    for body in ["message", "CEF:0|vendor|product|1.0|100|event|6|cs1=value"] {
        let wire = format!("<134>2 2024-06-15T12:00:00Z host app 0042 ID47 {sd} {body}");
        let service = configured_service("{profile: rfc5424}").unwrap();
        let plan = EncodingPlan::resolve(
            service.registry(),
            &PdataEncoding::new("syslog-batch"),
            EncodePolicy::default(),
        )
        .unwrap();
        let mut input = received(wire.as_bytes());
        for _ in 0..2 {
            let bytes = service.encode_bytes(&mut input, &plan).unwrap();
            let logs = decode(bytes.clone());
            for expected in [
                integer("syslog.version", 2),
                text("syslog.structured_data", sd),
                text("syslog.process_id_str", "0042"),
            ] {
                assert!(logs[0].attributes.contains(&expected));
            }

            input = OtapPayload::from(plan.codec().admit(SignalType::Logs, bytes).unwrap())
                .try_into_otap(&service)
                .unwrap();
        }
        for format in [Format::Rfc3164, Format::CefRfc3164, Format::Cef] {
            let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), format);
            assert!(
                encoder
                    .encode(input.clone())
                    .unwrap_err()
                    .to_string()
                    .contains("cannot represent RFC 5424 structured data")
            );
        }
    }
    for tag in ["app[0042]", "app[worker-1]", "app[]"] {
        let wire = format!("<134>Jun 15 12:00:00 host {tag}: message");
        let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Rfc3164);
        let bytes = encoder.encode(received(wire.as_bytes())).unwrap();
        assert_eq!(messages(&bytes), [wire.as_bytes()]);
        assert!(
            decode(bytes)[0]
                .attributes
                .contains(&text("syslog.tag", tag))
        );
    }
}

/// Scenario: RFC 5424 messages begin with brackets, spaces before brackets, or a literal BOM after structured data or NILVALUE.
/// Guarantees: Receiver-materialized and generated message text stays separate from structured data through repeated encoding, including literal leading BOMs and empty values.
#[test]
fn syslog_encoder_rfc5424_message_boundary_round_trips() {
    let service = configured_service("{profile: rfc5424}").unwrap();
    let plan = EncodingPlan::resolve(
        service.registry(),
        &PdataEncoding::new("syslog-batch"),
        EncodePolicy::default(),
    )
    .unwrap();
    for sd in ["-", "[id]", "[id] [second]"] {
        for body in [
            "[INFO] hello",
            "  [INFO] hello",
            "\u{feff}[INFO] hello",
            "\u{feff}",
            "normal",
            "",
        ] {
            let wire =
                format!("<134>1 2024-06-15T12:00:00Z host app 0042 ID47 {sd} \u{feff}{body}");
            let mut builder = SyslogBatchBuilder::new();
            builder.append_with_year(wire.as_bytes(), 2024).unwrap();
            let original = decode(builder.finish(123).unwrap().into_parts().0);
            assert!(
                original[0]
                    .attributes
                    .contains(&text("syslog.message", body))
            );
            let mut input = received(wire.as_bytes());
            for _ in 0..3 {
                let bytes = service.encode_bytes(&mut input, &plan).unwrap();
                let logs = decode(bytes.clone());
                assert_eq!(
                    semantic_attributes(&logs[0], false),
                    semantic_attributes(&original[0], false),
                    "{sd}: {body:?}"
                );
                assert!(logs[0].attributes.contains(&text("syslog.message", body)));
                assert_eq!(logs[0].observed_time_unix_nano, 123);
                input = OtapPayload::from(plan.codec().admit(SignalType::Logs, bytes).unwrap())
                    .try_into_otap(&service)
                    .unwrap();
            }
            let mut generated = log(body);
            generated.attributes.push(text("syslog.message", body));
            generated
                .attributes
                .push(text("syslog.structured_data", sd));
            let bytes = service
                .encode_bytes(&mut records(vec![generated], vec![]), &plan)
                .unwrap();
            let logs = decode(bytes);
            assert!(logs[0].attributes.contains(&text("syslog.message", body)));
            if sd != "-" {
                assert!(
                    logs[0]
                        .attributes
                        .contains(&text("syslog.structured_data", sd))
                );
            }
        }
    }
}

/// Scenario: Receiver-accepted unclosed RFC 5424 structured data is emitted alone or followed by plain or CEF payloads.
/// Guarantees: Header-only data round-trips exactly, while any appended message fails explicitly without leaking values or poisoning encoder reuse.
#[test]
fn syslog_encoder_rfc5424_rejects_unclosed_data_with_message() {
    let wire = b"<134>1 2024-06-15T12:00:00.000000Z host app 0042 ID47 [unclosed";
    let mut encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::Rfc5424);
    let bytes = encoder.encode(received(wire)).unwrap();
    assert_eq!(messages(&bytes), [wire.as_slice()]);
    let mut cef_encoder = SyslogEncoder::for_format(EncodePolicy::default(), Format::CefRfc5424);
    for body in ["PRIVATE", "[PRIVATE]", ""] {
        for encoder in [&mut encoder, &mut cef_encoder] {
            let mut input = log(body);
            input.attributes = cef_attributes();
            input.attributes.push(text("syslog.message", body));
            input
                .attributes
                .push(text("syslog.structured_data", "[unclosed"));
            let error = encoder.encode(records(vec![input], vec![])).unwrap_err();
            assert!(error.to_string().contains("without changing their meaning"));
            assert!(!error.to_string().contains("PRIVATE"));
        }
    }
    assert_eq!(
        messages(&encoder.encode(received(wire)).unwrap()),
        [wire.as_slice()]
    );
    let mut input = log("ok");
    input.attributes = cef_attributes();
    let logs = decode(cef_encoder.encode(records(vec![input], vec![])).unwrap());
    assert!(logs[0].attributes.contains(&text("msg", "ok")));
}
