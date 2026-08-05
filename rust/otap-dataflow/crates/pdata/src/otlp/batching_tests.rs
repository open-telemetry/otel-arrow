// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module tests batching.rs logic.

use crate::otlp::OtlpProtoBytes;
use crate::otlp::batching::make_bytes_batches;
use crate::proto::OtlpProtoMessage;
use crate::proto::opentelemetry::common::v1::any_value::Value;
use crate::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope};
use crate::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
use crate::proto::opentelemetry::metrics::v1::{
    Gauge, Metric, MetricsData, NumberDataPoint, ResourceMetrics, ScopeMetrics,
};
use crate::proto::opentelemetry::resource::v1::Resource;
use crate::proto::opentelemetry::trace::v1::{ResourceSpans, ScopeSpans, Span, TracesData};
use crate::testing::equiv::assert_equivalent;
use crate::testing::fixtures::DataGenerator;
use crate::testing::round_trip::otlp_bytes_to_message;
use crate::testing::round_trip::otlp_message_to_bytes;
use otap_df_config::SignalType;
use std::num::NonZeroU64;

/// Test bytes-based batching with various size limits
fn test_batching(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>) {
    // Clone the inputs for later equivalence checking.
    let inputs_otlp: Vec<_> = inputs_otlp.collect();
    let signal_type = inputs_otlp.first().expect("ok").signal_type();

    let inputs_bytes: Vec<OtlpProtoBytes> = inputs_otlp.iter().map(otlp_message_to_bytes).collect();
    let inputs_toplevel: usize = inputs_otlp
        .iter()
        .map(|b| match b {
            OtlpProtoMessage::Logs(data) => data.resource_logs.len(),
            OtlpProtoMessage::Metrics(data) => data.resource_metrics.len(),
            OtlpProtoMessage::Traces(data) => data.resource_spans.len(),
        })
        .sum();

    let total_input_bytes: usize = inputs_bytes.iter().map(|b| b.num_bytes()).sum();

    // Run a single equivalence test
    let test_config = |limit: Option<NonZeroU64>, expect_batches: usize, label: &str| {
        let outputs = make_bytes_batches(signal_type, limit, inputs_bytes.clone()).expect("ok");
        let total: usize = outputs.iter().map(|b| b.num_bytes()).sum();

        // When a resource entry is split within itself, the resource/scope
        // wrapper headers are duplicated across fragments, so the output may be
        // larger than the input. No record is ever dropped, so it can never be
        // smaller.
        assert!(
            total >= total_input_bytes,
            "{}: output byte count {total} < input {total_input_bytes}",
            label,
        );

        // Expected number of batches is tested (coarsely, because we
        // haven't carefully controlled the number of bytes per
        // toplevel item).
        assert!(
            outputs.len() >= expect_batches,
            "{} outputs expecting at least {expect_batches}",
            outputs.len(),
        );
        // The tight upper bound only holds when the limit is large enough that
        // no single resource entry must be split (sub-resource splitting can
        // legitimately produce more batches).
        let no_subresource_split = match limit {
            None => true,
            Some(l) => (l.get() as usize) >= total_input_bytes,
        };
        if no_subresource_split {
            assert!(
                outputs.len() <= expect_batches + 1,
                "{} outputs expecting at most {}",
                outputs.len(),
                expect_batches + 1,
            );
        }

        // Convert outputs back to OtlpProtoMessage and verify equivalence
        let outputs_msgs: Vec<OtlpProtoMessage> =
            outputs.into_iter().map(otlp_bytes_to_message).collect();
        assert_equivalent(&inputs_otlp, &outputs_msgs);
    };

    // Run with no limit (worst case)
    test_config(None, 1, "no limit");

    // Run with limit == actual size
    if total_input_bytes > 0 {
        test_config(
            Some(NonZeroU64::new(total_input_bytes as u64).unwrap()),
            1,
            "actual size",
        );
    }

    // Run with limit == actual_size * 0.5
    if total_input_bytes >= 2 {
        let limit_50pct = (total_input_bytes / 2).max(1);
        test_config(
            Some(NonZeroU64::new(limit_50pct as u64).unwrap()),
            std::cmp::min(2, inputs_toplevel),
            "50% limit",
        );
    }

    // Run with limit == 1 (worst case: should produce single-field batches)
    test_config(
        Some(NonZeroU64::new(1).unwrap()),
        inputs_toplevel,
        "limit 1",
    );
}

// Note: this test is similar to ../otap/batching_tests. We should
// consider a consolidation.

#[test]
fn test_simple_batch_logs() {
    for input_count in 1..=20 {
        let mut datagen = DataGenerator::new(1);
        test_batching((0..input_count).map(|_| datagen.generate_logs().into()));
    }
}

#[test]
fn test_simple_batch_traces() {
    for input_count in 1..=20 {
        let mut datagen = DataGenerator::new(1);
        test_batching((0..input_count).map(|_| datagen.generate_traces().into()));
    }
}

#[test]
fn test_simple_batch_metrics() {
    for input_count in 1..=20 {
        for point_count in 1..=10 {
            let mut datagen = DataGenerator::new(point_count);
            test_batching((0..input_count).map(|_| datagen.generate_metrics().into()));
        }
    }
}

/// Test that the batcher handles corrupted protobuf data
#[test]
fn test_corrupted_protobuf_handling() {
    let mut datagen = DataGenerator::new(1);
    let logs1 = datagen.generate_logs();
    let logs2 = datagen.generate_logs();

    // Convert both to bytes
    let good_bytes1 = otlp_message_to_bytes(&logs1.clone().into());
    let good_bytes2 = otlp_message_to_bytes(&logs2.clone().into());
    let good_size = good_bytes1.num_bytes() + good_bytes2.num_bytes();

    // Create a third input that's corrupted
    let mut corrupted_bytes = Vec::new();
    // Create a malformed field: valid tag (field 1, wire type 2=LEN_DELIM)
    let garbage = vec![
        0x0A, // field 1, wire type 2 (LEN_DELIM)
        0xFF, 0xFF, 0xFF, 0xFF, 0x0F, // varint: huge length that will fail
    ];
    corrupted_bytes.extend_from_slice(&garbage);

    let corrupted_input = OtlpProtoBytes::new_from_bytes(SignalType::Logs, corrupted_bytes);
    let corrupted_size = corrupted_input.num_bytes();

    let total_size = good_size + corrupted_size;

    // Batch with max_size between good_size and total_size
    // This should produce 2 outputs: good content batched together, then corrupt content
    let max_size = good_size + 2; // > good_size but < total_size

    let outputs = make_bytes_batches(
        SignalType::Logs,
        NonZeroU64::new(max_size as u64),
        vec![good_bytes1, good_bytes2, corrupted_input.clone()],
    )
    .expect("batching should succeed");

    // Should get 2 batches: good data together, then corrupt data
    assert_eq!(outputs.len(), 2);

    // First batch should contain the good data
    let first_size = outputs[0].num_bytes();
    assert_eq!(first_size, good_size);

    // Second batch should contain the garbage
    let second_size = outputs[1].num_bytes();
    assert_eq!(second_size, corrupted_size);
    assert_eq!(outputs[1].as_bytes(), garbage);

    // Total size should be preserved
    let total_output = first_size + second_size;
    assert_eq!(total_output, total_size);

    // First batch should decode successfully
    let first_decoded = otlp_bytes_to_message(outputs[0].clone());
    assert_eq!(first_decoded.num_items(), 6);

    // Verify first batch is equivalent to original good data
    let expected: Vec<OtlpProtoMessage> = vec![logs1.into(), logs2.into()];
    assert_equivalent(&expected, &[first_decoded]);
}

// -- Sub-resource (intra-ResourceX) splitting tests ------------------------
//
// These exercise the case where a single top-level resource entry exceeds the
// byte limit and must be split *within* the entry (down to scopes and, if
// needed, individual records).

/// Build a single-resource, single-scope LogsData with `n` small records.
fn single_resource_logs(scope_name: &str, n: usize) -> LogsData {
    let records: Vec<LogRecord> = (0..n)
        .map(|i| {
            LogRecord::build()
                .time_unix_nano(1000u64 + i as u64)
                .observed_time_unix_nano(1100u64 + i as u64)
                .body(AnyValue::new_string(format!(
                    "log record body number {i} with some padding"
                )))
                .finish()
        })
        .collect();
    LogsData::new(vec![ResourceLogs::new(
        Resource::build().finish(),
        vec![ScopeLogs::new(
            InstrumentationScope::build()
                .name(scope_name.to_string())
                .finish(),
            records,
        )],
    )])
}

/// Flatten every log record body (as a string) across the given messages, in
/// resource/scope/record order. Unlike `assert_equivalent` (which canonicalizes
/// into a `BTreeSet` and therefore ignores duplication and ordering), an exact
/// ordered comparison of this vector catches records that are dropped,
/// duplicated or reordered by splitting.
fn log_bodies(msgs: &[OtlpProtoMessage]) -> Vec<String> {
    let mut out = Vec::new();
    for m in msgs {
        if let OtlpProtoMessage::Logs(data) = m {
            for rl in &data.resource_logs {
                for sl in &rl.scope_logs {
                    for rec in &sl.log_records {
                        let body = rec
                            .body
                            .as_ref()
                            .and_then(|b| b.value.as_ref())
                            .map(|v| match v {
                                Value::StringValue(s) => s.clone(),
                                _ => String::new(),
                            })
                            .unwrap_or_default();
                        out.push(body);
                    }
                }
            }
        }
    }
    out
}

/// Minimal protobuf varint writer for hand-built wire-format test inputs.
fn wv(buf: &mut Vec<u8>, mut v: u64) {
    while v >= 0x80 {
        buf.push(((v & 0x7F) | 0x80) as u8);
        v >>= 7;
    }
    buf.push(v as u8);
}

/// Minimal protobuf LEN-delimited field writer.
fn wlen(buf: &mut Vec<u8>, field: u64, payload: &[u8]) {
    wv(buf, (field << 3) | 2);
    wv(buf, payload.len() as u64);
    buf.extend_from_slice(payload);
}

/// Scenario: A single ExportLogsServiceRequest carrying one ResourceLogs with a
/// single scope and many small records is batched with a byte limit far smaller
/// than the whole resource entry, forcing a split *within* the one resource
/// entry.
/// Guarantees: More than one batch is produced; every batch is within the byte
/// limit (records are individually small); and the union of the batches is
/// equivalent to the input, with no record lost, duplicated or reordered.
#[test]
fn test_split_single_resource_many_records() {
    let logs = single_resource_logs("scope", 40);
    let input = otlp_message_to_bytes(&logs.clone().into());
    let total = input.num_bytes();
    let max_size = (total / 8).max(1);

    let outputs = make_bytes_batches(
        SignalType::Logs,
        NonZeroU64::new(max_size as u64),
        vec![input],
    )
    .expect("ok");

    assert!(
        outputs.len() > 1,
        "expected multiple batches, got {}",
        outputs.len(),
    );
    for out in &outputs {
        assert!(
            out.num_bytes() <= max_size,
            "batch of {} bytes exceeds limit {max_size}",
            out.num_bytes(),
        );
    }

    let out_msgs: Vec<OtlpProtoMessage> = outputs.into_iter().map(otlp_bytes_to_message).collect();
    // Independent ordered check (assert_equivalent uses a BTreeSet and cannot
    // detect duplication or reordering): every body appears exactly once, in
    // the original order.
    let expected_bodies: Vec<String> = (0..40)
        .map(|i| format!("log record body number {i} with some padding"))
        .collect();
    assert_eq!(
        log_bodies(&out_msgs),
        expected_bodies,
        "records dropped, duplicated or reordered",
    );
    assert_equivalent(&[logs.into()], &out_msgs);
}

/// Scenario: A single ResourceLogs contains two scopes, each with several
/// records, and the byte limit forces boundaries both between scopes and within
/// a scope.
/// Guarantees: Multiple batches are produced, each within the byte limit, and
/// the union is equivalent to the input (both scopes and all records preserved).
#[test]
fn test_split_single_resource_multi_scope() {
    let logs = LogsData::new(vec![ResourceLogs::new(
        Resource::build().finish(),
        vec![
            ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope-a".to_string())
                    .finish(),
                (0..12)
                    .map(|i| {
                        LogRecord::build()
                            .time_unix_nano(1000u64 + i as u64)
                            .body(AnyValue::new_string(format!("a-{i}-xxxxxxxxxx")))
                            .finish()
                    })
                    .collect::<Vec<_>>(),
            ),
            ScopeLogs::new(
                InstrumentationScope::build()
                    .name("scope-b".to_string())
                    .finish(),
                (0..12)
                    .map(|i| {
                        LogRecord::build()
                            .time_unix_nano(2000u64 + i as u64)
                            .body(AnyValue::new_string(format!("b-{i}-yyyyyyyyyy")))
                            .finish()
                    })
                    .collect::<Vec<_>>(),
            ),
        ],
    )]);
    let input = otlp_message_to_bytes(&logs.clone().into());
    let total = input.num_bytes();
    let max_size = (total / 6).max(1);

    let outputs = make_bytes_batches(
        SignalType::Logs,
        NonZeroU64::new(max_size as u64),
        vec![input],
    )
    .expect("ok");

    assert!(outputs.len() > 1);
    for out in &outputs {
        assert!(out.num_bytes() <= max_size);
    }

    let out_msgs: Vec<OtlpProtoMessage> = outputs.into_iter().map(otlp_bytes_to_message).collect();
    // Ordered check across both scopes: scope-a records precede scope-b
    // records, each exactly once (catches duplication/reordering that
    // assert_equivalent's BTreeSet cannot).
    let expected_bodies: Vec<String> = (0..12)
        .map(|i| format!("a-{i}-xxxxxxxxxx"))
        .chain((0..12).map(|i| format!("b-{i}-yyyyyyyyyy")))
        .collect();
    assert_eq!(
        log_bodies(&out_msgs),
        expected_bodies,
        "records dropped, duplicated or reordered",
    );
    assert_equivalent(&[logs.into()], &out_msgs);
}

/// Scenario: A single resource entry carrying an unknown field on the resource
/// wrapper and an unknown field on the scope wrapper is split within the entry.
/// Guarantees: Every produced batch preserves both unknown wrapper fields, and
/// the union is equivalent to the input.
#[test]
fn test_split_preserves_unknown_fields() {
    // ScopeLogs payload: scope(1) empty, unknown field 9 (varint 777),
    // then 20 empty LogRecord entries (field 2).
    let mut scope = Vec::new();
    wlen(&mut scope, 1, &[]); // InstrumentationScope (empty)
    wv(&mut scope, 9 << 3); // unknown field 9, wire type 0 (varint)
    wv(&mut scope, 777);
    for _ in 0..20 {
        wlen(&mut scope, 2, &[]); // empty LogRecord
    }

    // ResourceLogs payload: resource(1) empty, unknown field 15 (LEN "META"),
    // then the scope list entry (field 2).
    let mut entry = Vec::new();
    wlen(&mut entry, 1, &[]); // Resource (empty)
    wlen(&mut entry, 15, b"META"); // unknown field 15
    wlen(&mut entry, 2, &scope); // ScopeLogs

    // Top-level ExportLogsServiceRequest: resource_logs (field 1).
    let mut top = Vec::new();
    wlen(&mut top, 1, &entry);

    let input = OtlpProtoBytes::new_from_bytes(SignalType::Logs, top.clone());
    let expected = otlp_bytes_to_message(input.clone());
    let max_size = (top.len() / 4).max(20);

    let outputs = make_bytes_batches(
        SignalType::Logs,
        NonZeroU64::new(max_size as u64),
        vec![input],
    )
    .expect("ok");

    assert!(outputs.len() > 1, "expected a split, got {}", outputs.len());
    let scope_unknown = [0x48u8, 0x89, 0x06]; // field 9 varint, value 777
    for out in &outputs {
        let bytes = out.as_bytes();
        assert!(
            bytes.windows(4).any(|w| w == b"META"),
            "resource unknown field dropped from a batch",
        );
        assert!(
            bytes.windows(3).any(|w| w == scope_unknown),
            "scope unknown field dropped from a batch",
        );
    }

    let out_msgs: Vec<OtlpProtoMessage> = outputs.into_iter().map(otlp_bytes_to_message).collect();
    assert_equivalent(&[expected], &out_msgs);
}

/// Scenario: A single record is larger than the byte limit (with minimal
/// wrappers it still cannot fit).
/// Guarantees: It is emitted as exactly one batch (best-effort, exceeding the
/// limit) and remains equivalent to the input.
#[test]
fn test_split_lone_oversize_record() {
    let logs = LogsData::new(vec![ResourceLogs::new(
        Resource::build().finish(),
        vec![ScopeLogs::new(
            InstrumentationScope::build().name("s".to_string()).finish(),
            vec![
                LogRecord::build()
                    .time_unix_nano(1000u64)
                    .body(AnyValue::new_string("X".repeat(300)))
                    .finish(),
            ],
        )],
    )]);
    let input = otlp_message_to_bytes(&logs.clone().into());

    let outputs =
        make_bytes_batches(SignalType::Logs, NonZeroU64::new(32), vec![input]).expect("ok");

    assert_eq!(outputs.len(), 1);
    let out_msgs: Vec<OtlpProtoMessage> = outputs.into_iter().map(otlp_bytes_to_message).collect();
    assert_equivalent(&[logs.into()], &out_msgs);
}

/// Scenario: A single ResourceSpans with one scope and many spans is batched
/// with a byte limit smaller than the resource entry.
/// Guarantees: Multiple batches within the limit, equivalent to the input.
#[test]
fn test_split_single_resource_traces() {
    let spans: Vec<Span> = (0..24)
        .map(|i| {
            Span::build()
                .trace_id(vec![0u8; 16])
                .span_id(vec![i as u8; 8])
                .name(format!("span-number-{i}-with-padding"))
                .start_time_unix_nano(1000u64 + i as u64)
                .end_time_unix_nano(1100u64 + i as u64)
                .finish()
        })
        .collect();
    let traces = TracesData::new(vec![ResourceSpans::new(
        Resource::build().finish(),
        vec![ScopeSpans::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            spans,
        )],
    )]);
    let input = otlp_message_to_bytes(&traces.clone().into());
    let total = input.num_bytes();
    let max_size = (total / 8).max(1);

    let outputs = make_bytes_batches(
        SignalType::Traces,
        NonZeroU64::new(max_size as u64),
        vec![input],
    )
    .expect("ok");

    assert!(outputs.len() > 1);
    for out in &outputs {
        assert!(out.num_bytes() <= max_size);
    }
    let out_msgs: Vec<OtlpProtoMessage> = outputs.into_iter().map(otlp_bytes_to_message).collect();
    assert_equivalent(&[traces.into()], &out_msgs);
}

/// Scenario: A single ResourceMetrics with one scope and many metrics is
/// batched with a byte limit smaller than the resource entry.
/// Guarantees: Multiple batches within the limit, equivalent to the input.
#[test]
fn test_split_single_resource_metrics() {
    let metrics: Vec<Metric> = (0..24)
        .map(|i| {
            Metric::build()
                .name(format!("metric-{i}"))
                .description("a gauge metric")
                .unit("1")
                .data_gauge(Gauge::new(vec![
                    NumberDataPoint::build()
                        .value_double(i as f64 * 2.0)
                        .time_unix_nano(1000u64 + i as u64)
                        .finish(),
                ]))
                .finish()
        })
        .collect();
    let data = MetricsData::new(vec![ResourceMetrics::new(
        Resource::build().finish(),
        vec![ScopeMetrics::new(
            InstrumentationScope::build()
                .name("scope".to_string())
                .finish(),
            metrics,
        )],
    )]);
    let input = otlp_message_to_bytes(&data.clone().into());
    let total = input.num_bytes();
    let max_size = (total / 8).max(1);

    let outputs = make_bytes_batches(
        SignalType::Metrics,
        NonZeroU64::new(max_size as u64),
        vec![input],
    )
    .expect("ok");

    assert!(outputs.len() > 1);
    for out in &outputs {
        assert!(out.num_bytes() <= max_size);
    }
    let out_msgs: Vec<OtlpProtoMessage> = outputs.into_iter().map(otlp_bytes_to_message).collect();
    assert_equivalent(&[data.into()], &out_msgs);
}

/// Scenario: A single oversize resource entry contains a valid scope followed
/// by a malformed (truncated) field at the resource level, under a byte limit
/// small enough to normally force a within-entry split.
/// Guarantees: Rather than folding the corrupt tail into a duplicated header
/// (which would reorder/duplicate it ahead of every fragment), the whole
/// resource entry is emitted byte-for-byte as a single batch.
#[test]
fn test_split_malformed_resource_field_emits_entry_whole() {
    let mut scope = Vec::new();
    wlen(&mut scope, 1, &[]); // InstrumentationScope (empty)
    for i in 0..8u8 {
        wlen(&mut scope, 2, &[0x08, i]); // valid LogRecord (field 1 varint)
    }

    let mut entry = Vec::new();
    wlen(&mut entry, 1, &[]); // Resource (empty)
    wlen(&mut entry, 2, &scope); // ScopeLogs (valid)
    entry.push((3 << 3) | 2); // field 3, wire type 2 (LEN)
    entry.push(0x7F); // declares 127 payload bytes but none follow: truncated

    let mut top = Vec::new();
    wlen(&mut top, 1, &entry);

    let input = OtlpProtoBytes::new_from_bytes(SignalType::Logs, top.clone());
    let max_size = (top.len() / 4).max(4);

    let outputs = make_bytes_batches(
        SignalType::Logs,
        NonZeroU64::new(max_size as u64),
        vec![input],
    )
    .expect("ok");

    assert_eq!(outputs.len(), 1, "malformed entry must not be split");
    assert_eq!(
        outputs[0].as_bytes(),
        top.as_slice(),
        "malformed entry must be preserved byte-for-byte",
    );
}

/// Scenario: A single oversize resource entry holds one scope whose payload has
/// valid records followed by a malformed (truncated) field, under a byte limit
/// small enough that the scope must be split by records.
/// Guarantees: Instead of reordering the corrupt scope tail ahead of the
/// records, the whole entry is emitted byte-for-byte as a single batch.
#[test]
fn test_split_malformed_scope_field_emits_entry_whole() {
    let mut scope = Vec::new();
    wlen(&mut scope, 1, &[]); // InstrumentationScope (empty)
    for i in 0..8u8 {
        wlen(&mut scope, 2, &[0x08, i]); // valid LogRecord (field 1 varint)
    }
    scope.push((3 << 3) | 2); // field 3, wire type 2 (LEN)
    scope.push(0x7F); // declares 127 payload bytes but none follow: truncated

    let mut entry = Vec::new();
    wlen(&mut entry, 1, &[]); // Resource (empty)
    wlen(&mut entry, 2, &scope); // ScopeLogs wrapping the malformed payload

    let mut top = Vec::new();
    wlen(&mut top, 1, &entry);

    let input = OtlpProtoBytes::new_from_bytes(SignalType::Logs, top.clone());
    let max_size = (top.len() / 4).max(4);

    let outputs = make_bytes_batches(
        SignalType::Logs,
        NonZeroU64::new(max_size as u64),
        vec![input],
    )
    .expect("ok");

    assert_eq!(outputs.len(), 1, "malformed scope must not be split");
    assert_eq!(
        outputs[0].as_bytes(),
        top.as_slice(),
        "malformed scope must be preserved byte-for-byte",
    );
}
