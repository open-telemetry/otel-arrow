// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::super::{
    config::Config,
    framer::{LineContinuation, LineFragment, OversizeBehavior},
};
use super::*;
use otel_arrow_dfe_pdata::{
    otlp::{ProtoBuffer, ProtoBytesEncoder, logs::LogsProtoBytesEncoder},
    proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest,
};
use prost::Message;
use std::{path::Path, time::Duration};

fn config(records: u64, bytes: u64) -> Config {
    let mut config = Config {
        include: vec![
            if cfg!(windows) {
                "C:\\logs\\*.log"
            } else {
                "/logs/*.log"
            }
            .into(),
        ],
        ..Config::default()
    };
    config.framing.max_line_bytes = 64;
    config.framing.max_record_bytes = 64;
    config.framing.max_log_size_behavior = OversizeBehavior::Truncate;
    config.batch.max_records = records;
    config.batch.max_bytes = bytes;
    config
}

fn inputs(records: u64, bytes: u64) -> ValidatedInputs {
    config(records, bytes)
        .compile(
            "g",
            "p",
            "filelog",
            Path::new(if cfg!(windows) { "C:\\" } else { "/" }),
        )
        .unwrap()
        .into_parts()
        .0
}

fn record(file: u128, start: u64, body: &str) -> Record {
    let end = start + body.len() as u64 + 1;
    Record {
        file_id: cp::FileId::from_bytes(file.to_be_bytes()),
        epoch: 7,
        frame: Frame::Line(LineFrame {
            body: LineBody::Text(body.into()),
            source_body: None,
            body_range: SourceRange {
                start,
                end: end - 1,
            },
            frame_range: SourceRange { start, end },
            ending: LineEnding::Lf,
            fragment: None,
            continuation: None,
            truncated: false,
            discarded_source_bytes: 0,
            malformed_units: 0,
        }),
        guard: cp::CommittedFrontierGuard::compute(end, &vec![0; end.min(64) as usize]).unwrap(),
        observed_time_unix_nano: 123,
        metadata: Rc::new(FileMetadataAttributes::default()),
    }
}

fn decode(batch: &Batch) -> ExportLogsServiceRequest {
    let mut records = batch.records.clone();
    let mut bytes = ProtoBuffer::default();
    LogsProtoBytesEncoder::new()
        .encode(&mut records, &mut bytes)
        .unwrap();
    ExportLogsServiceRequest::decode(bytes.as_ref()).unwrap()
}

/// Scenario: Interleaved files emit text, exact byte bodies and an empty line.
/// Guarantees: OTAP preserves types and observed times, leaves semantic fields unset, and coalesces only each file's contiguous source progress.
#[test]
fn projection_and_internal_progress_are_independent() {
    let mut builder = BatchBuilder::new(&inputs(10, 1024));
    let now = Instant::now();
    let first = record(1, 0, "abc");
    let first_id = first.file_id;
    assert_eq!(builder.offer(first, now).unwrap(), Flow::Continue);
    let mut binary = record(2, 0, "x");
    let Frame::Line(frame) = &mut binary.frame else {
        unreachable!()
    };
    frame.body = LineBody::Bytes(vec![0xff]);
    binary.observed_time_unix_nano = 456;
    assert_eq!(builder.offer(binary, now).unwrap(), Flow::Continue);
    let last = record(1, 4, "");
    let guard = last.guard;
    let _ = builder.offer(last, now).unwrap();
    assert!(builder.unresolved(first_id, 7));
    builder.seal().unwrap();
    let batch = builder.retained().unwrap();
    assert_eq!(batch.progress.len(), 2);
    assert_eq!(batch.provenance.len(), 3);
    assert_eq!(batch.provenance[2].file_id, first_id);
    assert_eq!(
        batch.provenance[2].frame_range,
        SourceRange { start: 4, end: 5 }
    );
    let p = &batch.progress[0];
    assert_eq!(p.expected_committed_offset, 0);
    assert_eq!(p.new_committed_offset, 5);
    assert_eq!(p.new_committed_frontier_guard, guard);
    assert_eq!(p.new_framing_resume, cp::FramingResume::Clean);
    let request = decode(batch);
    let logs = &request.resource_logs[0].scope_logs[0].log_records;
    assert_eq!(logs.len(), 3);
    assert_eq!(
        logs[0].body.as_ref().unwrap().value,
        Some(Value::StringValue("abc".into()))
    );
    assert_eq!(
        logs[1].body.as_ref().unwrap().value,
        Some(Value::BytesValue(vec![0xff]))
    );
    assert_eq!(
        logs[2].body.as_ref().unwrap().value,
        Some(Value::StringValue(String::new()))
    );
    assert_eq!(logs[0].observed_time_unix_nano, 123);
    assert_eq!(logs[1].observed_time_unix_nano, 456);
    for log in logs {
        assert_eq!(log.time_unix_nano, 0);
        assert_eq!(log.severity_number, 0);
        assert!(log.severity_text.is_empty() && log.trace_id.is_empty() && log.span_id.is_empty());
        assert!(log.attributes.is_empty());
    }
}

/// Scenario: A variable-size record cannot fit after a smaller record.
/// Guarantees: Exactly one owned carry-over survives the wait, a mismatched release changes nothing, and promotion preserves its original body and observed time.
#[test]
fn carry_over_waits_for_matching_application_and_seeds_first() {
    let mut builder = BatchBuilder::new(&inputs(10, 300));
    let now = Instant::now();
    let _ = builder.offer(record(1, 0, "0123456789"), now).unwrap();
    let carry = record(1, 11, &"x".repeat(60));
    let id = carry.file_id;
    assert_eq!(
        builder.preflight(id, 7, RECORD_OVERHEAD, now).unwrap(),
        Flow::Continue
    );
    assert_eq!(builder.offer(carry, now).unwrap(), Flow::Stop);
    assert!(builder.carry.is_some());
    assert!(builder.unresolved(id, 7));
    assert_eq!(builder.poll(now).unwrap(), Flow::Stop);
    let applied = builder.retained().unwrap().progress.clone();
    let mut wrong = applied.clone();
    wrong[0].new_committed_offset += 1;
    assert_eq!(
        builder.release_applied(&wrong, now),
        Err(BatchError::Release)
    );
    assert_eq!(builder.retained().unwrap().record_count, 1);
    builder
        .release_applied(&applied, now + Duration::from_secs(10))
        .unwrap();
    assert!(builder.retained().is_none() && builder.carry.is_none());
    assert_eq!(builder.count, 1);
    assert_eq!(builder.progress[0].expected_committed_offset, 11);
    builder.seal().unwrap();
    let request = decode(builder.retained().unwrap());
    let log = &request.resource_logs[0].scope_logs[0].log_records[0];
    assert_eq!(
        log.body.as_ref().unwrap().value,
        Some(Value::StringValue("x".repeat(60)))
    );
    assert_eq!(log.observed_time_unix_nano, 123);
}

/// Scenario: A sparse batch receives another record just before its original deadline.
/// Guarantees: Later records do not extend the deadline and an idle poll seals exactly at expiry.
#[test]
fn first_record_deadline_is_not_extended() {
    let mut builder = BatchBuilder::new(&inputs(10, 1024));
    let now = Instant::now();
    let _ = builder.offer(record(1, 0, "a"), now).unwrap();
    let at = builder.next_deadline().unwrap();
    let _ = builder
        .offer(record(1, 2, "b"), at - Duration::from_nanos(1))
        .unwrap();
    assert_eq!(builder.next_deadline(), Some(at));
    assert_eq!(
        builder.poll(at - Duration::from_nanos(1)).unwrap(),
        Flow::Continue
    );
    assert_eq!(builder.poll(at).unwrap(), Flow::Stop);
    assert_eq!(builder.next_deadline(), None);
    assert_eq!(builder.retained().unwrap().record_count, 2);
}

/// Scenario: A source attempts an offset gap, overlap, regression or epoch change.
/// Guarantees: Rejection returns the exact frame, leaves prior progress untouched and makes admission terminal.
#[test]
fn invalid_progress_is_terminal_without_silent_merge() {
    for (offset, epoch) in [(0, 7), (1, 7), (3, 7), (2, 8)] {
        let mut builder = BatchBuilder::new(&inputs(10, 1024));
        let now = Instant::now();
        let _ = builder.offer(record(1, 0, "a"), now).unwrap();
        let mut bad = record(1, offset, "b");
        bad.epoch = epoch;
        let rejected = builder.offer(bad, now).unwrap_err();
        assert_eq!(rejected.error, BatchError::Progress);
        assert_eq!(rejected.record.frame.range().start, offset);
        assert_eq!(builder.count, 1);
        assert_eq!(builder.progress[0].new_committed_offset, 2);
        assert_eq!(builder.poll(now), Err(BatchError::Progress));
    }
}

/// Scenario: An oversized record or a second offered carry-over violates admission.
/// Guarantees: Neither output is discarded or accepted into unbounded storage.
#[test]
fn impossible_record_and_second_carry_over_are_rejected() {
    let now = Instant::now();
    let mut builder = BatchBuilder::new(&inputs(10, 300));
    let bad = builder
        .offer(record(1, 0, &"x".repeat(301)), now)
        .unwrap_err();
    assert_eq!(bad.error, BatchError::RecordSize);
    assert_eq!(builder.count, 0);
    let mut builder = BatchBuilder::new(&inputs(10, 300));
    let _ = builder.offer(record(1, 0, "0123456789"), now).unwrap();
    let _ = builder.offer(record(1, 11, &"x".repeat(60)), now).unwrap();
    let rejected = builder.offer(record(1, 72, "later"), now).unwrap_err();
    assert_eq!(rejected.error, BatchError::Stopped);
    assert_eq!(builder.carry.as_ref().unwrap().frame.range().start, 11);
    assert_eq!(builder.retained().unwrap().record_count, 1);
}

/// Scenario: A configuration allows more records than one WAL transaction's distinct-file limit.
/// Guarantees: Preflight seals at 4,096 files before any record from the next file is needed.
#[test]
fn distinct_file_limit_preflights_before_the_next_source_turn() {
    let mut builder = BatchBuilder::new(&inputs(5000, 1024 * 1024));
    let now = Instant::now();
    for file in 1..=MAX_DISTINCT_FILES {
        let _ = builder.offer(record(file as u128, 0, ""), now).unwrap();
    }
    let next = cp::FileId::from_bytes((MAX_DISTINCT_FILES as u128 + 1).to_be_bytes());
    assert_eq!(
        builder.preflight(next, 7, RECORD_OVERHEAD, now).unwrap(),
        Flow::Stop
    );
    let batch = builder.retained().unwrap();
    assert_eq!(batch.progress.len(), MAX_DISTINCT_FILES);
    assert_eq!(batch.record_count, MAX_DISTINCT_FILES);
    assert!(builder.carry.is_none());
}

/// Scenario: A configured batch reaches the maximum 65,535 records from one file.
/// Guarantees: The final ID is 65,534, the batch seals, and IDs never wrap.
#[test]
fn maximum_record_count_seals_without_id_wrap() {
    use arrow::array::UInt16Array;
    let mut builder = BatchBuilder::new(&inputs(65535, 16 * 1024 * 1024));
    let now = Instant::now();
    for offset in 0..65535 {
        let flow = builder.offer(record(1, offset, ""), now).unwrap();
        assert_eq!(
            flow,
            if offset == 65534 {
                Flow::Stop
            } else {
                Flow::Continue
            }
        );
    }
    let batch = builder.retained().unwrap();
    let logs = batch.records.get(ArrowPayloadType::Logs).unwrap();
    let ids = logs
        .column_by_name("id")
        .unwrap()
        .as_any()
        .downcast_ref::<UInt16Array>()
        .unwrap();
    assert_eq!(ids.value(65534), 65534);
    assert_eq!(batch.progress.len(), 1);
    assert_eq!(batch.progress[0].new_committed_offset, 65535);
}

/// Scenario: A nonfinal fragment is followed by its final terminal-unterminated fragment.
/// Guarantees: Fragment identity is stable, continuation follows the last included frame, and terminal evidence is exported only on the terminal frame.
#[test]
fn fragment_projection_and_resume_follow_owned_frames() {
    let mut config = config(10, 1024);
    config.framing.max_log_size_behavior = OversizeBehavior::Split;
    let inputs = config
        .compile(
            "g",
            "p",
            "filelog",
            Path::new(if cfg!(windows) { "C:\\" } else { "/" }),
        )
        .unwrap()
        .into_parts()
        .0;
    let mut builder = BatchBuilder::new(&inputs);
    let now = Instant::now();
    let mut first = record(1, 0, "abcd");
    let Frame::Line(f) = &mut first.frame else {
        unreachable!()
    };
    f.frame_range.end = 4;
    f.fragment = Some(LineFragment {
        record_start_offset: 0,
        index: 0,
        is_last: false,
    });
    f.continuation = Some(LineContinuation {
        record_start_offset: 0,
        next_fragment_index: 1,
    });
    f.ending = LineEnding::Split;
    let _ = builder.offer(first, now).unwrap();
    assert!(matches!(
        builder.progress[0].new_framing_resume,
        cp::FramingResume::Continuation {
            record_start_offset: 0,
            record_end_offset: 0,
            next_fragment_index: 1,
        }
    ));
    let mut last = record(1, 4, "ef");
    let Frame::Line(f) = &mut last.frame else {
        unreachable!()
    };
    f.fragment = Some(LineFragment {
        record_start_offset: 0,
        index: 1,
        is_last: true,
    });
    f.ending = LineEnding::PermanentEof;
    let _ = builder.offer(last, now).unwrap();
    builder.seal().unwrap();
    let batch = builder.retained().unwrap();
    assert_eq!(
        batch.progress[0].new_framing_resume,
        cp::FramingResume::Clean
    );
    let request = decode(batch);
    let logs = &request.resource_logs[0].scope_logs[0].log_records;
    let attr = |index: usize, key: &str| {
        logs[index]
            .attributes
            .iter()
            .find(|kv| kv.key == key)
            .and_then(|kv| kv.value.as_ref())
            .and_then(|v| v.value.as_ref())
    };
    assert_eq!(attr(0, FRAGMENT_KEYS[0]), attr(1, FRAGMENT_KEYS[0]));
    assert_eq!(
        attr(0, FRAGMENT_KEYS[0]),
        Some(&Value::StringValue(
            "0426593a0a8e0e63bb7f6a8259836c5e240eeb3205fa9a8daefe644bfa37b402".into()
        ))
    );
    assert_eq!(attr(0, TERMINAL_KEY), None);
    assert_eq!(attr(1, TERMINAL_KEY), Some(&Value::BoolValue(true)));
}

/// Scenario: Shutdown abandons a retained batch and its one unsent carry-over.
/// Guarantees: The explicit report accounts for both records without releasing or inventing applied progress.
#[test]
fn shutdown_reports_uncommitted_carry_over() {
    let mut builder = BatchBuilder::new(&inputs(10, 300));
    let now = Instant::now();
    let _ = builder.offer(record(1, 0, "0123456789"), now).unwrap();
    let _ = builder.offer(record(1, 11, &"x".repeat(60)), now).unwrap();
    assert_eq!(
        builder.abort(),
        PendingCapture {
            records: 2,
            logical_bytes: 138 + 188
        }
    );
}

/// Scenario: A batch reaches its exact logical byte limit, or the next known minimum cannot fit.
/// Guarantees: Exact fits seal without a carry-over, and known overflow seals before source work.
#[test]
fn logical_byte_bound_uses_shared_sizing_at_both_admission_points() {
    let now = Instant::now();
    let mut builder = BatchBuilder::new(&inputs(10, 276));
    let a = record(1, 0, "0123456789");
    assert_eq!(a.logical_size().unwrap(), logical_size(10, 0).unwrap());
    assert_eq!(builder.offer(a, now).unwrap(), Flow::Continue);
    assert_eq!(
        builder.offer(record(1, 11, "0123456789"), now).unwrap(),
        Flow::Stop
    );
    assert!(builder.carry.is_none());
    assert_eq!(builder.retained().unwrap().logical_bytes, 276);
    let mut builder = BatchBuilder::new(&inputs(10, 276));
    let _ = builder.offer(record(1, 0, &"x".repeat(60)), now).unwrap();
    let next = cp::FileId::from_bytes(1u128.to_be_bytes());
    assert_eq!(
        builder.preflight(next, 7, RECORD_OVERHEAD, now).unwrap(),
        Flow::Stop
    );
    assert!(builder.carry.is_none());
}

/// Scenario: A multiline known-end continuation proposes an early final frame.
/// Guarantees: Internal fallback/decoding evidence survives projection, and a clean resume cannot be coalesced before the stored record end.
#[test]
fn multiline_known_end_rejects_early_clean_resume() {
    use super::super::multiline::{RecordContinuation, RecordFragment};
    let mut config = config(10, 1024);
    config.framing.max_log_size_behavior = OversizeBehavior::Split;
    let inputs = config
        .compile(
            "g",
            "p",
            "filelog",
            Path::new(if cfg!(windows) { "C:\\" } else { "/" }),
        )
        .unwrap()
        .into_parts()
        .0;
    let mut builder = BatchBuilder::new(&inputs);
    let now = Instant::now();
    let mut first = record(1, 0, "abcd");
    first.frame = Frame::Multiline(RecordFrame {
        body: LineBody::Text("abcd".into()),
        body_range: SourceRange { start: 0, end: 4 },
        frame_range: SourceRange { start: 0, end: 4 },
        ending: RecordEnding::ByteLimit,
        pattern_not_matched: true,
        fragment: Some(RecordFragment {
            record_start_offset: 0,
            index: 0,
            is_last: false,
        }),
        continuation: Some(RecordContinuation {
            record_start_offset: 0,
            record_end_offset: 10,
            next_fragment_index: 1,
        }),
        truncated: false,
        discarded_source_bytes: 0,
        malformed_units: 2,
    });
    let _ = builder.offer(first, now).unwrap();
    assert_eq!(builder.provenance[0].malformed_units, 2);
    assert!(builder.provenance[0].pattern_not_matched);
    let mut last = record(1, 4, "ef");
    let Frame::Line(f) = &mut last.frame else {
        unreachable!()
    };
    f.fragment = Some(LineFragment {
        record_start_offset: 0,
        index: 1,
        is_last: true,
    });
    assert_eq!(
        builder.offer(last, now).unwrap_err().error,
        BatchError::Progress
    );
    assert!(matches!(
        builder.progress[0].new_framing_resume,
        cp::FramingResume::Continuation {
            record_end_offset: 10,
            ..
        }
    ));
}
