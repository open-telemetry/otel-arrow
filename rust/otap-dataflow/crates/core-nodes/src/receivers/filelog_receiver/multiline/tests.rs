// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn config(mode: GroupingMode) -> MultilineConfig {
    MultilineConfig {
        line: LineConfig {
            encoding: Encoding::Utf8,
            on_decode_error: OnDecodeError::PreserveRaw,
            max_line_bytes: 64,
            max_record_bytes: 128,
            oversize: OversizeBehavior::Split,
        },
        max_lines: 10,
        mode,
    }
}
fn make(
    config: MultilineConfig,
    source: &str,
    start: RecordStart,
) -> (MultilineFramer, BoundaryMatcher) {
    let mode = if config.line.encoding == Encoding::Raw {
        PatternMode::Raw
    } else {
        PatternMode::Text
    };
    let pattern = BoundaryPattern::compile(source, mode).unwrap();
    let matcher = pattern.matcher();
    (
        MultilineFramer::new(config, pattern, start).unwrap(),
        matcher,
    )
}
fn feed(
    framer: &mut MultilineFramer,
    matcher: &mut BoundaryMatcher,
    input: &[u8],
    output: &mut Vec<RecordFrame>,
) -> Result<(), MultilineFailure> {
    let mut pos = 0;
    for _ in 0..100_000 {
        let step = framer.next(framer.next_expected_input_offset(), &input[pos..], matcher)?;
        pos += step.consumed;
        if let Some(frame) = step.frame {
            if framer.next_expected_input_offset() > frame.frame_range.end {
                assert_eq!(framer.pending_source_start(), Some(frame.frame_range.end));
            }
            output.push(frame);
        }
        if !step.advanced {
            assert_eq!(pos, input.len());
            return Ok(());
        }
    }
    panic!("step loop did not stop");
}
fn complete(
    framer: &mut MultilineFramer,
    matcher: &mut BoundaryMatcher,
    reason: PartialCompletion,
    output: &mut Vec<RecordFrame>,
) -> Result<(), MultilineFailure> {
    for _ in 0..100_000 {
        let step = framer.complete_partial(reason, matcher)?;
        if let Some(frame) = step.frame {
            if framer.next_expected_input_offset() > frame.frame_range.end {
                assert_eq!(framer.pending_source_start(), Some(frame.frame_range.end));
            }
            output.push(frame);
        }
        if step.complete {
            return Ok(());
        }
    }
    panic!("completion loop did not stop");
}
fn bytes(frame: &RecordFrame) -> &[u8] {
    match &frame.body {
        LineBody::Text(s) => s.as_bytes(),
        LineBody::Bytes(b) => b,
    }
}

/// Scenario: Start mode encounters unmatched input, a new start, and an authorized idle boundary.
/// Guarantees: Fallback, internal LF, output ordering, and exact source ranges are preserved.
#[test]
fn start_boundaries() {
    let (mut f, mut m) = make(
        config(GroupingMode::Start),
        "^START",
        RecordStart::NewStream,
    );
    let mut out = vec![];
    feed(&mut f, &mut m, b"noise\nSTART\na\nSTART\nb\n", &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(bytes(&out[0]), b"noise");
    assert_eq!(out[0].ending, RecordEnding::PatternNotMatched);
    assert_eq!(bytes(&out[1]), b"START\na");
    assert_eq!(out[1].frame_range, SourceRange { start: 6, end: 14 });
    assert_eq!(out[1].body_range, SourceRange { start: 6, end: 13 });
    complete(&mut f, &mut m, PartialCompletion::Idle, &mut out).unwrap();
    assert_eq!(bytes(&out[2]), b"START\nb");
    assert_eq!(out[2].ending, RecordEnding::Idle);
}

/// Scenario: End mode groups empty lines, CR, and NUL before the matching final line.
/// Guarantees: Only the final LF is omitted and a match includes the entire matching line.
#[test]
fn end_boundaries() {
    let (mut f, mut m) = make(config(GroupingMode::End), "^END$", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, b"\nA\r\n\0\nEND\n", &mut out).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(bytes(&out[0]), b"\nA\r\n\0\nEND");
    assert_eq!(out[0].frame_range, SourceRange { start: 0, end: 10 });
    assert_eq!(out[0].ending, RecordEnding::EndPattern);
}

/// Scenario: A line within its own limit crosses the combined record limit.
/// Guarantees: Split fragments reconstruct the trigger-line-bounded record and carry its known end.
#[test]
fn known_end_split() {
    let mut c = config(GroupingMode::End);
    c.line.max_record_bytes = 5;
    let (mut f, mut m) = make(c, "^END$", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, b"abc\ndef\n", &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(bytes(&out[0]), b"abc\nd");
    assert_eq!(bytes(&out[1]), b"ef");
    assert!(
        out.iter()
            .all(|frame| matches!(frame.body, LineBody::Bytes(_)))
    );
    assert_eq!(
        out[0].continuation,
        Some(RecordContinuation {
            record_start_offset: 0,
            record_end_offset: 8,
            next_fragment_index: 1
        })
    );
    assert_eq!(out[0].frame_range.end, 5);
    assert!(out[1].fragment.unwrap().is_last);
    assert_eq!(out[1].frame_range, SourceRange { start: 5, end: 8 });
}

/// Scenario: A physical line exceeds its limit after an unfinished multiline group.
/// Guarantees: The earlier group emits first and the oversized line is isolated and bounded.
#[test]
fn oversized_line_order() {
    let mut c = config(GroupingMode::End);
    c.line.max_line_bytes = 4;
    let (mut f, mut m) = make(c, "END", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, b"a\n123456\n", &mut out).unwrap();
    assert_eq!(out.len(), 3);
    assert_eq!(bytes(&out[0]), b"a");
    assert_eq!(out[0].ending, RecordEnding::OversizeLineBoundary);
    assert_eq!(bytes(&out[1]), b"1234");
    assert_eq!(out[1].continuation.unwrap().record_end_offset, 0);
    assert_eq!(bytes(&out[2]), b"56");
}

/// Scenario: A source resumes inside a known-end split sequence that contains another LF.
/// Guarantees: Resume suppresses pattern/line-count decisions and stops at the stored end.
#[test]
fn known_end_recovery() {
    let mut c = config(GroupingMode::Start);
    c.line.max_record_bytes = 4;
    c.max_lines = 1;
    let start = RecordStart::Continuation {
        offset: 2,
        continuation: RecordContinuation {
            record_start_offset: 0,
            record_end_offset: 9,
            next_fragment_index: 1,
        },
    };
    let (mut f, mut m) = make(c, "^", start);
    let mut out = vec![];
    feed(&mut f, &mut m, b"c\ndefg\nz\n", &mut out).unwrap();
    assert_eq!(bytes(&out[0]), b"c\nde");
    assert_eq!(out[0].frame_range, SourceRange { start: 2, end: 6 });
    assert_eq!(bytes(&out[1]), b"fg");
    assert_eq!(out[1].frame_range.end, 9);
    assert_eq!(out[1].fragment.unwrap().index, 2);
    assert_eq!(bytes(&out[2]), b"z");
    assert!(out[2].fragment.is_none());
}

fn encoded(text: &str, encoding: Encoding) -> Vec<u8> {
    match encoding {
        Encoding::Utf16Le => text.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        Encoding::Utf16Be => text.encode_utf16().flat_map(u16::to_be_bytes).collect(),
        _ => text.as_bytes().to_vec(),
    }
}

/// Scenario: The same short source arrives in every possible nonempty chunk partition.
/// Guarantees: Group bodies, source ranges, and completion evidence are independent of read boundaries.
#[test]
fn all_small_chunk_partitions() {
    for encoding in [
        Encoding::Utf8,
        Encoding::Ascii,
        Encoding::Raw,
        Encoding::Utf16Le,
        Encoding::Utf16Be,
    ] {
        for mode in [GroupingMode::Start, GroupingMode::End] {
            let mut c = config(mode);
            c.line.encoding = encoding;
            let source = encoded("S\nx\nE\n", encoding);
            let pattern = BoundaryPattern::compile(
                if mode == GroupingMode::Start {
                    "^S$"
                } else {
                    "^E$"
                },
                if encoding == Encoding::Raw {
                    PatternMode::Raw
                } else {
                    PatternMode::Text
                },
            )
            .unwrap();
            let mut baseline = vec![];
            let mut f = MultilineFramer::new(c, pattern.clone(), RecordStart::NewStream).unwrap();
            let mut m = pattern.matcher();
            feed(&mut f, &mut m, &source, &mut baseline).unwrap();
            complete(&mut f, &mut m, PartialCompletion::Idle, &mut baseline).unwrap();
            assert_eq!(baseline.len(), 1);
            for mask in 0..(1usize << (source.len() - 1)) {
                let mut f =
                    MultilineFramer::new(c, pattern.clone(), RecordStart::NewStream).unwrap();
                let mut m = pattern.matcher();
                let mut out = vec![];
                let mut start = 0;
                for end in 1..=source.len() {
                    if end == source.len() || mask & (1 << (end - 1)) != 0 {
                        feed(&mut f, &mut m, &source[start..end], &mut out).unwrap();
                        start = end;
                    }
                }
                complete(&mut f, &mut m, PartialCompletion::Idle, &mut out).unwrap();
                assert_eq!(out, baseline, "{encoding:?} {mode:?} partition {mask}");
            }
        }
    }
}

/// Scenario: A later malformed UTF-16 unit changes an unsplit group's representation.
/// Guarantees: Text pattern matching still works and exact original body bytes preserve internal LFs and omit the BOM.
#[test]
fn malformed_shadow_and_bom() {
    for encoding in [Encoding::Utf16Le, Encoding::Utf16Be] {
        let mut c = config(GroupingMode::End);
        c.line.encoding = encoding;
        let mut source = encoded("\u{feff}S\n", encoding);
        source.extend(encoded("\u{fffd}", encoding));
        // Replace the valid unit with an unpaired low surrogate.
        let n = source.len();
        source[n - 2..].copy_from_slice(&if encoding == Encoding::Utf16Le {
            0xdc00u16.to_le_bytes()
        } else {
            0xdc00u16.to_be_bytes()
        });
        source.extend(encoded("\nE\n", encoding));
        let (mut f, mut m) = make(c, "^E$", RecordStart::NewStream);
        let mut out = vec![];
        for b in &source {
            feed(&mut f, &mut m, &[*b], &mut out).unwrap();
        }
        assert_eq!(out.len(), 1);
        assert_eq!(bytes(&out[0]), &source[2..source.len() - 2]);
        assert!(matches!(out[0].body, LineBody::Bytes(_)));
        assert_eq!(out[0].malformed_units, 1);
        assert_eq!(
            out[0].body_range,
            SourceRange {
                start: 2,
                end: source.len() as u64 - 2
            }
        );
    }
}

/// Scenario: Decoded UTF-16 text overflows a smaller record bound while its physical lines fit.
/// Guarantees: Split positions use source-unit offsets rather than decoded UTF-8 byte counts.
#[test]
fn replacement_mode_utf16_split_ranges() {
    let mut c = config(GroupingMode::End);
    c.line.encoding = Encoding::Utf16Le;
    c.line.on_decode_error = OnDecodeError::Replace;
    c.line.max_record_bytes = 4;
    let source = encoded("ab\ncd\n", Encoding::Utf16Le);
    let (mut f, mut m) = make(c, "END", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, &source, &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(bytes(&out[0]), b"ab\nc");
    assert_eq!(out[0].frame_range, SourceRange { start: 0, end: 8 });
    assert_eq!(out[0].continuation.unwrap().record_end_offset, 12);
    assert_eq!(bytes(&out[1]), b"d");
    assert_eq!(out[1].body_range, SourceRange { start: 8, end: 10 });
    assert_eq!(out[1].frame_range.end, 12);
}

/// Scenario: Restart occurs after each nonfinal fragment of known-end and scan-to-LF sequences.
/// Guarantees: Recovery reproduces remaining bodies, ranges, indices, and continuation coordinates.
#[test]
fn every_fragment_restart() {
    for (max_line, source) in [(64, &b"a\n123456789\n"[..]), (4, &b"a\n123456789\n"[..])] {
        let mut c = config(GroupingMode::End);
        c.line.max_record_bytes = 4;
        c.line.max_line_bytes = max_line;
        let (mut f, mut m) = make(c, "END", RecordStart::NewStream);
        let mut expected = vec![];
        feed(&mut f, &mut m, source, &mut expected).unwrap();
        for (index, frame) in expected.iter().enumerate() {
            let Some(continuation) = frame.continuation else {
                continue;
            };
            let offset = frame.frame_range.end;
            let (mut f, mut m) = make(
                c,
                "END",
                RecordStart::Continuation {
                    offset,
                    continuation,
                },
            );
            let mut out = vec![];
            feed(&mut f, &mut m, &source[offset as usize..], &mut out).unwrap();
            assert_eq!(out.len(), expected.len() - index - 1);
            for (actual, wanted) in out.iter().zip(&expected[index + 1..]) {
                assert_eq!(actual.body, wanted.body);
                assert_eq!(actual.body_range, wanted.body_range);
                assert_eq!(actual.frame_range, wanted.frame_range);
                assert_eq!(actual.fragment, wanted.fragment);
                assert_eq!(actual.continuation, wanted.continuation);
            }
        }
    }
}

/// Scenario: An oversized line's discarded tail is malformed under decode-fail.
/// Guarantees: Earlier grouped content emits first, but no prefix of the failing line escapes.
#[test]
fn truncate_failure_order() {
    let mut c = config(GroupingMode::End);
    c.line.max_line_bytes = 4;
    c.line.oversize = OversizeBehavior::Truncate;
    c.line.on_decode_error = OnDecodeError::Fail;
    let (mut f, mut m) = make(c, "END", RecordStart::NewStream);
    let mut out = vec![];
    let error = feed(&mut f, &mut m, b"a\n12345\xff\n", &mut out).unwrap_err();
    assert!(matches!(
        error.error,
        MultilineError::Decode(DecodeError::FatalMalformed { .. })
    ));
    assert_eq!(out.len(), 1);
    assert_eq!(bytes(&out[0]), b"a");
    assert_eq!(out[0].ending, RecordEnding::OversizeLineBoundary);
    assert!(f.terminal_error().is_some());
    assert_eq!(
        f.next(f.next_expected_input_offset(), b"", &mut m)
            .unwrap_err()
            .consumed,
        0
    );
}

/// Scenario: A malformed unit occurs only in a truncated multiline tail.
/// Guarantees: The clean prefix remains text, discarded bytes count, and progress ends at the trigger-line LF.
#[test]
fn truncate_discard_evidence() {
    let mut c = config(GroupingMode::End);
    c.line.max_record_bytes = 4;
    c.line.oversize = OversizeBehavior::Truncate;
    let (mut f, mut m) = make(c, "END", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, b"ab\nc\xffd\n", &mut out).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].body, LineBody::Text("ab\nc".into()));
    assert!(out[0].truncated);
    assert_eq!(out[0].discarded_source_bytes, 2);
    assert_eq!(out[0].malformed_units, 1);
    assert_eq!(out[0].frame_range.end, 7);
    assert_eq!(out[0].body_range.end, 4);
}

/// Scenario: A line-count bound completes start-mode grouping before an unmatched line.
/// Guarantees: Completion resets seeking and unmatched input is emitted independently.
#[test]
fn line_count_resets_seeking() {
    let mut c = config(GroupingMode::Start);
    c.max_lines = 2;
    let (mut f, mut m) = make(c, "^S", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, b"S\nx\ny\n", &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(bytes(&out[0]), b"S\nx");
    assert_eq!(out[0].ending, RecordEnding::LineLimit);
    assert_eq!(out[1].ending, RecordEnding::PatternNotMatched);
}

/// Scenario: A grouped body exactly fills the record limit before later malformed input.
/// Guarantees: The clean byte boundary emits before decoding later input and does not create fragments.
#[test]
fn exact_bound_before_later_failure() {
    let mut c = config(GroupingMode::End);
    c.line.max_record_bytes = 4;
    c.line.on_decode_error = OnDecodeError::Fail;
    let (mut f, mut m) = make(c, "END", RecordStart::NewStream);
    let mut out = vec![];
    assert!(feed(&mut f, &mut m, b"abcd\n\xff", &mut out).is_err());
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].ending, RecordEnding::ByteLimit);
    assert_eq!(out[0].frame_range.end, 5);
    assert!(out[0].fragment.is_none());
}

/// Scenario: Idle completion ends an oversized partial line, then new bytes arrive.
/// Guarantees: The new line starts at the completed source boundary without stale replay offsets.
#[test]
fn completion_resets_stream_offsets() {
    let mut c = config(GroupingMode::End);
    c.line.max_line_bytes = 4;
    let (mut f, mut m) = make(c, "^E$", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, b"123456", &mut out).unwrap();
    complete(&mut f, &mut m, PartialCompletion::Idle, &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(out[1].ending, RecordEnding::Idle);
    assert_eq!(f.pending_source_start(), None);
    feed(&mut f, &mut m, b"E\n", &mut out).unwrap();
    assert_eq!(out[2].frame_range, SourceRange { start: 6, end: 8 });
}

/// Scenario: A known-end continuation reaches live EOF with no new bytes.
/// Guarantees: Neither idle nor permanent completion invents output or shortens the stored end.
#[test]
fn empty_recovery_stays_pending() {
    let c = config(GroupingMode::End);
    let start = RecordStart::Continuation {
        offset: 4,
        continuation: RecordContinuation {
            record_start_offset: 0,
            record_end_offset: 20,
            next_fragment_index: 1,
        },
    };
    let (mut f, mut m) = make(c, "END", start);
    let mut out = vec![];
    for reason in [PartialCompletion::Idle, PartialCompletion::PermanentEof] {
        complete(&mut f, &mut m, reason, &mut out).unwrap();
        assert!(out.is_empty());
        assert_eq!(f.next_expected_input_offset(), 4);
    }
}

/// Scenario: Caller supplies another pattern cache, an incorrect offset, or changes completion authority.
/// Guarantees: Contract errors consume nothing and leave the original operation usable.
#[test]
fn recoverable_caller_errors() {
    let (mut f, mut m) = make(config(GroupingMode::End), "END", RecordStart::NewStream);
    let mut other = BoundaryPattern::compile("OTHER", PatternMode::Text)
        .unwrap()
        .matcher();
    assert!(matches!(
        f.next(0, b"a", &mut other).unwrap_err().error,
        MultilineError::WrongMatcher
    ));
    assert!(matches!(
        f.next(1, b"a", &mut m).unwrap_err().error,
        MultilineError::Decode(DecodeError::OffsetDiscontinuity { .. })
    ));
    let mut out = vec![];
    feed(&mut f, &mut m, b"a", &mut out).unwrap();
    assert!(
        !f.complete_partial(PartialCompletion::Idle, &mut m)
            .unwrap()
            .complete
    );
    assert!(matches!(
        f.complete_partial(PartialCompletion::PermanentEof, &mut m)
            .unwrap_err()
            .error,
        MultilineError::CompletionInProgress
    ));
    assert!(matches!(
        f.next(f.next_expected_input_offset(), b"x", &mut m)
            .unwrap_err()
            .error,
        MultilineError::CompletionInProgress
    ));
    complete(&mut f, &mut m, PartialCompletion::Idle, &mut out).unwrap();
    assert_eq!(bytes(&out[0]), b"a");
}

/// Scenario: A matching start line would overflow the currently buffered record.
/// Guarantees: The byte-bound rule includes the trigger line and suppresses its pattern boundary.
#[test]
fn byte_bound_precedes_start_match() {
    let mut c = config(GroupingMode::Start);
    c.line.max_record_bytes = 5;
    let (mut f, mut m) = make(c, "^S", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, b"Sabc\nSdef\n", &mut out).unwrap();
    assert_eq!(out.len(), 2);
    assert_eq!(bytes(&out[0]), b"Sabc\n");
    assert_eq!(bytes(&out[1]), b"Sdef");
    assert_eq!(out[0].continuation.unwrap().record_end_offset, 10);
    assert_eq!(out[1].ending, RecordEnding::ByteLimit);
}

/// Scenario: Permanent EOF makes a grouped incomplete UTF-8 unit eligible under each decode policy.
/// Guarantees: Preserve/replace retain the terminal range; fail suppresses the incomplete record after earlier output.
#[test]
fn permanent_eof_decode_policies() {
    for policy in [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ] {
        let mut c = config(GroupingMode::End);
        c.line.on_decode_error = policy;
        let (mut f, mut m) = make(c, "^E$", RecordStart::NewStream);
        let mut out = vec![];
        feed(&mut f, &mut m, b"E\na\n\xe2", &mut out).unwrap();
        assert_eq!(out.len(), 1);
        let result = complete(&mut f, &mut m, PartialCompletion::PermanentEof, &mut out);
        if policy == OnDecodeError::Fail {
            assert!(result.is_err());
            assert_eq!(out.len(), 1);
        } else {
            result.unwrap();
            assert_eq!(out.len(), 2);
            let wanted = if policy == OnDecodeError::PreserveRaw {
                LineBody::Bytes(b"a\n\xe2".to_vec())
            } else {
                LineBody::Text("a\n\u{fffd}".into())
            };
            assert_eq!(out[1].body, wanted);
            assert_eq!(out[1].frame_range, SourceRange { start: 2, end: 5 });
            assert_eq!(out[1].ending, RecordEnding::PermanentEof);
            assert_eq!(out[1].malformed_units, 1);
        }
    }
}

/// Scenario: A new stream begins with a conflicting BOM and later matches an end line.
/// Guarantees: Bounded replay preserves the decoder's original malformed-unit grouping and replacement text.
#[test]
fn conflicting_bom_replay() {
    let mut c = config(GroupingMode::End);
    c.line.on_decode_error = OnDecodeError::Replace;
    let (mut f, mut m) = make(c, "^E$", RecordStart::NewStream);
    let mut out = vec![];
    feed(&mut f, &mut m, b"\xff\xfe\nx\nE\n", &mut out).unwrap();
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].body, LineBody::Text("\u{fffd}\nx\nE".into()));
    assert_eq!(out[0].malformed_units, 1);
}

/// Scenario: Split recovery starts at the maximum fragment index.
/// Guarantees: A final maximum-index fragment is allowed, but no nonfinal fragment escapes on overflow.
#[test]
fn fragment_index_overflow() {
    let mut c = config(GroupingMode::End);
    c.line.max_record_bytes = 4;
    for source in [&b"abc\n"[..], &b"abcde\n"[..]] {
        let start = RecordStart::Continuation {
            offset: 1,
            continuation: RecordContinuation {
                record_start_offset: 0,
                record_end_offset: 0,
                next_fragment_index: u32::MAX,
            },
        };
        let (mut f, mut m) = make(c, "END", start);
        let mut out = vec![];
        let result = feed(&mut f, &mut m, source, &mut out);
        if source.len() == 4 {
            result.unwrap();
            assert_eq!(out[0].fragment.unwrap().index, u32::MAX);
            assert!(out[0].fragment.unwrap().is_last);
        } else {
            assert!(matches!(
                result.unwrap_err().error,
                MultilineError::FragmentIndexOverflow
            ));
            assert!(out.is_empty());
        }
    }
}

/// Scenario: UTF-16 and malformed UTF-8 overflow records at different chunk boundaries and restart points.
/// Guarantees: Partitions and recovery never cut encoded units or alter output representations.
#[test]
fn split_partitions_and_encoded_restart() {
    for encoding in [Encoding::Utf8, Encoding::Utf16Le, Encoding::Utf16Be] {
        for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
            let mut c = config(GroupingMode::End);
            c.line.encoding = encoding;
            c.line.on_decode_error = policy;
            c.line.max_record_bytes = 5;
            let mut source = encoded("ab\ncdef\n", encoding);
            if encoding == Encoding::Utf8 {
                source[4] = 0xff;
            }
            let (mut f, mut m) = make(c, "END", RecordStart::NewStream);
            let mut baseline = vec![];
            feed(&mut f, &mut m, &source, &mut baseline).unwrap();
            complete(&mut f, &mut m, PartialCompletion::Idle, &mut baseline).unwrap();
            for cut in 0..=source.len() {
                let (mut f, mut m) = make(c, "END", RecordStart::NewStream);
                let mut out = vec![];
                feed(&mut f, &mut m, &source[..cut], &mut out).unwrap();
                feed(&mut f, &mut m, &source[cut..], &mut out).unwrap();
                complete(&mut f, &mut m, PartialCompletion::Idle, &mut out).unwrap();
                assert_eq!(out, baseline, "{encoding:?} {policy:?} cut {cut}");
            }
            for (index, frame) in baseline.iter().enumerate() {
                let Some(continuation) = frame.continuation else {
                    continue;
                };
                let offset = frame.frame_range.end;
                let (mut f, mut m) = make(
                    c,
                    "END",
                    RecordStart::Continuation {
                        offset,
                        continuation,
                    },
                );
                let mut out = vec![];
                feed(&mut f, &mut m, &source[offset as usize..], &mut out).unwrap();
                complete(&mut f, &mut m, PartialCompletion::Idle, &mut out).unwrap();
                for (actual, expected) in out.iter().zip(&baseline[index + 1..]) {
                    assert_eq!(actual.body, expected.body);
                    assert_eq!(actual.body_range, expected.body_range);
                    assert_eq!(actual.frame_range, expected.frame_range);
                    assert_eq!(actual.fragment, expected.fragment);
                    assert_eq!(actual.malformed_units, expected.malformed_units);
                }
                assert_eq!(out.len(), baseline.len() - index - 1);
            }
        }
    }
}

/// Scenario: Invalid bounds, a wrong pattern representation, or malformed resume coordinates are supplied.
/// Guarantees: Construction rejects them before retaining any source bytes.
#[test]
fn construction_validation() {
    let c = config(GroupingMode::Start);
    let pattern = BoundaryPattern::compile("^", PatternMode::Text).unwrap();
    for invalid in [
        MultilineConfig { max_lines: 0, ..c },
        MultilineConfig {
            line: LineConfig {
                max_record_bytes: 1,
                ..c.line
            },
            ..c
        },
        MultilineConfig {
            line: LineConfig {
                max_line_bytes: usize::MAX,
                ..c.line
            },
            ..c
        },
    ] {
        assert!(MultilineFramer::new(invalid, pattern.clone(), RecordStart::NewStream).is_err());
    }
    assert!(
        MultilineFramer::new(
            c,
            BoundaryPattern::compile("^", PatternMode::Raw).unwrap(),
            RecordStart::NewStream
        )
        .is_err()
    );
    for (offset, start, end, index) in [(0, 0, 0, 1), (3, 4, 0, 1), (3, 0, 3, 1), (3, 0, 4, 0)] {
        assert!(
            MultilineFramer::new(
                c,
                pattern.clone(),
                RecordStart::Continuation {
                    offset,
                    continuation: RecordContinuation {
                        record_start_offset: start,
                        record_end_offset: end,
                        next_fragment_index: index
                    }
                }
            )
            .is_err()
        );
    }
}

/// Scenario: Raw input uses all combinations of tiny physical-line and record limits.
/// Guarantees: Output ranges cover the source contiguously and every body is the exact bounded source slice.
#[test]
fn tiny_raw_bounds_cover_source() {
    let source = b"\nA\nBBBBBB\nCC\nD";
    for line_limit in 1..=4 {
        for record_limit in 1..=4 {
            for mode in [GroupingMode::Start, GroupingMode::End] {
                for pattern in ["^", "^Z$"] {
                    let mut c = config(mode);
                    c.line.encoding = Encoding::Raw;
                    c.line.max_line_bytes = line_limit;
                    c.line.max_record_bytes = record_limit;
                    c.max_lines = 2;
                    let (mut f, mut m) = make(c, pattern, RecordStart::NewStream);
                    let mut out = vec![];
                    for b in source {
                        feed(&mut f, &mut m, &[*b], &mut out).unwrap();
                    }
                    complete(&mut f, &mut m, PartialCompletion::Idle, &mut out).unwrap();
                    let mut offset = 0;
                    for frame in &out {
                        assert_eq!(frame.frame_range.start, offset);
                        assert!(frame.frame_range.end > offset);
                        assert!(bytes(frame).len() <= record_limit);
                        assert_eq!(
                            bytes(frame),
                            &source[frame.body_range.start as usize..frame.body_range.end as usize]
                        );
                        if frame.continuation.is_some() {
                            assert_eq!(frame.body_range.end, frame.frame_range.end);
                        }
                        offset = frame.frame_range.end;
                    }
                    assert_eq!(offset, source.len() as u64);
                    assert_eq!(f.pending_source_start(), None);
                }
            }
        }
    }
}
