// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests that the OTAP -> OTLP logs encoder emits `LogRecord` and `KeyValue`
//! fields in the order chosen for compression.
//!
//! - `KeyValue` is written value first, then key (as Go pdata does).
//! - `LogRecord` keeps forward field order, except that `observed_time_unix_nano`
//!   (11) and `time_unix_nano` (1) are written last.
//!
//! Field order does not change the decoded data, but these layouts make OTLP logs
//! compress noticeably better. Other messages are walked but their order is not
//! checked. See the "Field order" section of the module README.

use std::collections::{HashMap, HashSet};

use crate::TryIntoWithOptions;
use crate::otap::OtapArrowRecords;
use crate::otlp::OtlpProtoBytes;
use crate::proto::OtlpProtoMessage;
use crate::proto::consts::wire_types;
use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use crate::proto::opentelemetry::logs::v1::{
    LogRecord, LogRecordFlags, LogsData, ResourceLogs, ScopeLogs, SeverityNumber,
};
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::testing::round_trip::otlp_to_otap;
use crate::views::otlp::bytes::decode::{field_value_range, read_varint};

const TRACE_ID: [u8; 16] = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
const SPAN_ID: [u8; 8] = [21, 22, 23, 24, 25, 26, 27, 28];

/// Scenario: a logs batch with every encodable field set at every level, including
/// map and array attribute values and a map body, is encoded from OTAP to OTLP.
/// Guarantees: every `LogRecord` lists its fields in forward order with the two
/// timestamps last, every `KeyValue` (including nested ones) lists value before
/// key, and every expected field of those messages is observed at least once.
#[test]
fn test_logs_field_order() {
    let msg = OtlpProtoMessage::Logs(full_logs());
    let bytes = encode_to_otlp(&msg);
    let seen = check_message_tree(Msg::LogsData, &bytes);
    assert_full_coverage(&seen, &[Msg::LogRecord, Msg::KeyValue]);
}

fn encode_to_otlp(msg: &OtlpProtoMessage) -> Vec<u8> {
    let otap: OtapArrowRecords = otlp_to_otap(msg);
    let otlp: OtlpProtoBytes = otap.try_into_with_default().expect("encode to OTLP");
    otlp.as_bytes().to_vec()
}

/// OTLP message kinds reachable from the encoder output.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
enum Msg {
    LogsData,
    ResourceLogs,
    ScopeLogs,
    LogRecord,
    Resource,
    Scope,
    KeyValue,
    AnyValue,
    ArrayValue,
    KeyValueList,
}

impl Msg {
    /// Expected wire order of field numbers, or `None` if the order of this message
    /// is not checked.
    fn expected_order(self) -> Option<&'static [u64]> {
        match self {
            // Forward order, but observed_time_unix_nano (11) and time_unix_nano (1)
            // come last.
            Msg::LogRecord => Some(&[2, 3, 5, 6, 7, 8, 9, 10, 12, 11, 1]),
            Msg::KeyValue => Some(&[2, 1]),
            _ => None,
        }
    }

    /// Message kind of a length-delimited sub-message field, if any.
    fn child(self, field: u64) -> Option<Msg> {
        let child = match (self, field) {
            (Msg::LogsData, 1) => Msg::ResourceLogs,
            (Msg::ResourceLogs, 1) => Msg::Resource,
            (Msg::ResourceLogs, 2) => Msg::ScopeLogs,
            (Msg::ScopeLogs, 1) => Msg::Scope,
            (Msg::ScopeLogs, 2) => Msg::LogRecord,
            (Msg::LogRecord, 5) => Msg::AnyValue,
            (Msg::LogRecord, 6) => Msg::KeyValue,
            (Msg::Resource, 1) => Msg::KeyValue,
            (Msg::Scope, 3) => Msg::KeyValue,
            (Msg::KeyValue, 2) => Msg::AnyValue,
            (Msg::AnyValue, 5) => Msg::ArrayValue,
            (Msg::AnyValue, 6) => Msg::KeyValueList,
            (Msg::ArrayValue, 1) => Msg::AnyValue,
            (Msg::KeyValueList, 1) => Msg::KeyValue,
            _ => return None,
        };
        Some(child)
    }
}

/// Field numbers seen per message kind across the whole tree.
type SeenFields = HashMap<Msg, HashSet<u64>>;

/// Walks the encoded message tree rooted at `bytes`, asserting that the fields of
/// each checked message appear in the expected order. Returns the fields observed for each message kind.
fn check_message_tree(root: Msg, bytes: &[u8]) -> SeenFields {
    let mut seen = SeenFields::new();
    check_message(root, bytes, &mut seen);
    seen
}

fn check_message(msg: Msg, bytes: &[u8], seen: &mut SeenFields) {
    // Field numbers in wire order, with consecutive repeats of a field collapsed.
    let mut observed: Vec<u64> = Vec::new();
    let mut pos = 0;
    while pos < bytes.len() {
        let (tag, after_tag) = read_varint(bytes, pos).expect("valid tag");
        let field = tag >> 3;
        let wire_type = tag & 7;
        let (start, end) =
            field_value_range(bytes, wire_type, after_tag).expect("valid field value");
        pos = end;

        if observed.last() != Some(&field) {
            observed.push(field);
        }
        let _ = seen.entry(msg).or_default().insert(field);

        if wire_type == wire_types::LEN
            && let Some(child) = msg.child(field)
        {
            check_message(child, &bytes[start..end], seen);
        }
    }

    let Some(order) = msg.expected_order() else {
        return;
    };
    let present: HashSet<u64> = observed.iter().copied().collect();
    let expected: Vec<u64> = order
        .iter()
        .copied()
        .filter(|f| present.contains(f))
        .collect();
    assert_eq!(
        observed, expected,
        "{msg:?}: fields out of expected order (or a repeated field is not contiguous)"
    );
}

/// Asserts that every field of every listed message kind was observed at least once,
/// so the order checks above actually exercised the full layout.
fn assert_full_coverage(seen: &SeenFields, kinds: &[Msg]) {
    for kind in kinds {
        let fields = seen.get(kind).cloned().unwrap_or_default();
        let missing: Vec<u64> = kind
            .expected_order()
            .expect("coverage is only checked for messages with an expected order")
            .iter()
            .copied()
            .filter(|f| !fields.contains(f))
            .collect();
        assert!(
            missing.is_empty(),
            "{kind:?}: fixture does not produce fields {missing:?}"
        );
    }
}

fn kv_str(key: &str, val: &str) -> KeyValue {
    KeyValue::new(key, AnyValue::new_string(val))
}

fn full_resource() -> Resource {
    Resource::build()
        .attributes(vec![kv_str("res.a", "1"), kv_str("res.b", "2")])
        .dropped_attributes_count(3u32)
        .finish()
}

fn full_scope() -> InstrumentationScope {
    InstrumentationScope::build()
        .name("scope")
        .version("1.2.3")
        .attributes(vec![kv_str("scope.a", "x")])
        .dropped_attributes_count(4u32)
        .finish()
}

fn full_logs() -> LogsData {
    let map_value = AnyValue::new_kvlist(vec![
        kv_str("inner.str", "v"),
        KeyValue::new(
            "inner.arr",
            AnyValue::new_array(vec![AnyValue::new_int(1), AnyValue::new_string("two")]),
        ),
    ]);
    let record = LogRecord::build()
        .time_unix_nano(1_000u64)
        .observed_time_unix_nano(2_000u64)
        .severity_number(SeverityNumber::Info as i32)
        .severity_text("INFO")
        .body(map_value.clone())
        .attributes(vec![
            kv_str("attr.str", "s"),
            KeyValue::new("attr.int", AnyValue::new_int(7)),
            KeyValue::new("attr.map", map_value),
        ])
        .dropped_attributes_count(5u32)
        .flags(LogRecordFlags::TraceFlagsMask)
        .trace_id(TRACE_ID.to_vec())
        .span_id(SPAN_ID.to_vec())
        .event_name("event")
        .finish();
    let second = LogRecord::build()
        .time_unix_nano(3_000u64)
        .body(AnyValue::new_string("plain body"))
        .finish();

    LogsData::new(vec![
        ResourceLogs::new(
            full_resource(),
            vec![
                ScopeLogs::new(full_scope(), vec![record, second])
                    .set_schema_url("https://scope.schema"),
            ],
        )
        .set_schema_url("https://resource.schema"),
    ])
}
