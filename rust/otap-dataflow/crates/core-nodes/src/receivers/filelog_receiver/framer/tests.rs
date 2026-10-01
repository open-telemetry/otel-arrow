// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn config(encoding: Encoding, policy: OnDecodeError, limit: usize) -> LineConfig {
    LineConfig {
        encoding,
        on_decode_error: policy,
        max_line_bytes: limit,
        max_record_bytes: limit,
        oversize: OversizeBehavior::Split,
    }
}

fn new(config: LineConfig) -> LineFramer {
    LineFramer::new(config, LineStart::NewStream).expect("valid test operation")
}

fn feed(
    framer: &mut LineFramer,
    mut input: &[u8],
    frames: &mut Vec<LineFrame>,
) -> Result<(), LineFailure> {
    loop {
        let step = framer.next(framer.next_expected_input_offset(), input)?;
        assert!(step.consumed <= StreamDecoder::MAX_INPUT_BYTES_PER_CALL);
        input = &input[step.consumed..];
        if let Some(frame) = step.frame {
            frames.push(frame);
        }
        if !step.advanced {
            assert!(input.is_empty());
            return Ok(());
        }
    }
}

fn finish(
    framer: &mut LineFramer,
    reason: PartialCompletion,
) -> Result<Vec<LineFrame>, LineFailure> {
    let mut frames = Vec::new();
    for _ in 0..16 {
        let step = framer.complete_partial(reason)?;
        if let Some(frame) = step.frame {
            frames.push(frame);
        }
        if step.complete {
            return Ok(frames);
        }
    }
    panic!("completion did not drain bounded state");
}

fn range(start: u64, end: u64) -> SourceRange {
    SourceRange { start, end }
}
fn text(body: &str) -> LineBody {
    LineBody::Text(body.to_owned())
}
fn bytes(body: &[u8]) -> LineBody {
    LineBody::Bytes(body.to_vec())
}

fn expected(body: LineBody, body_range: SourceRange, frame_range: SourceRange) -> LineFrame {
    LineFrame {
        body,
        source_body: None,
        body_range,
        frame_range,
        ending: LineEnding::Lf,
        fragment: None,
        continuation: None,
        truncated: false,
        discarded_source_bytes: 0,
        malformed_units: 0,
    }
}

fn split(mut frame: LineFrame, origin: u64, index: u32, last: bool) -> LineFrame {
    frame.fragment = Some(LineFragment {
        record_start_offset: origin,
        index,
        is_last: last,
    });
    if !last {
        frame.ending = LineEnding::Split;
        frame.continuation = Some(LineContinuation {
            record_start_offset: origin,
            next_fragment_index: index + 1,
        });
    }
    frame
}

fn partitioned(config: LineConfig, source: &[u8], expected: &[LineFrame]) {
    // Exhaust all cut sets, including cuts inside BOMs, scalars and LF units.
    assert!(source.len() < 20);
    for cuts in 0..(1usize << source.len().saturating_sub(1)) {
        let mut framer = new(config);
        let mut output = Vec::new();
        let mut start = 0;
        for end in 1..=source.len() {
            if end == source.len() || cuts & (1 << (end - 1)) != 0 {
                feed(&mut framer, &source[start..end], &mut output).expect("valid test operation");
                start = end;
            }
        }
        for frame in &mut output {
            let shadow = if config.on_decode_error == OnDecodeError::PreserveRaw
                && matches!(frame.body, LineBody::Text(_))
            {
                Some(
                    source[frame.body_range.start as usize..frame.body_range.end as usize].to_vec(),
                )
            } else {
                None
            };
            assert_eq!(frame.source_body.take(), shadow);
        }
        assert_eq!(output, expected, "cut mask {cuts:x}");
        assert_eq!(framer.pending_source_start(), None);
    }
}

/// Scenario: LF, CR, NUL, BOM and empty lines arrive at every source-byte partition.
/// Guarantees: All encodings produce independently specified bodies and exact delimiter/BOM ownership.
#[test]
fn encoding_bom_and_delimiter_partitions() {
    for encoding in [Encoding::Utf8, Encoding::Ascii] {
        partitioned(
            config(encoding, OnDecodeError::PreserveRaw, 32),
            b"a\r\n\n\0\n",
            &[
                expected(text("a\r"), range(0, 2), range(0, 3)),
                expected(text(""), range(3, 3), range(3, 4)),
                expected(text("\0"), range(4, 5), range(4, 6)),
            ],
        );
    }
    partitioned(
        config(Encoding::Utf8, OnDecodeError::PreserveRaw, 32),
        b"\xef\xbb\xbfx\n",
        &[expected(text("x"), range(3, 4), range(0, 5))],
    );
    for (encoding, source) in [
        (
            Encoding::Utf16Le,
            &[0xff, 0xfe, b'a', 0, 13, 0, 10, 0, 10, 0][..],
        ),
        (
            Encoding::Utf16Be,
            &[0xfe, 0xff, 0, b'a', 0, 13, 0, 10, 0, 10][..],
        ),
    ] {
        partitioned(
            config(encoding, OnDecodeError::PreserveRaw, 32),
            source,
            &[
                expected(text("a\r"), range(2, 6), range(0, 8)),
                expected(text(""), range(8, 8), range(8, 10)),
            ],
        );
    }
    partitioned(
        config(Encoding::Raw, OnDecodeError::Fail, 32),
        b"\xef\xbb\xbf\xff\r\n\n",
        &[
            expected(bytes(b"\xef\xbb\xbf\xff\r"), range(0, 5), range(0, 6)),
            expected(bytes(b""), range(6, 6), range(6, 7)),
        ],
    );
}

/// Scenario: Malformed units, conflicting BOMs and later BOM-shaped content cross input chunks.
/// Guarantees: Preserve/replace retain the complete prefix or replacement text with source-byte ranges.
#[test]
fn malformed_and_conflicting_bom_partitions() {
    for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
        let raw = policy == OnDecodeError::PreserveRaw;
        for (encoding, source, rendered, body_start, count) in [
            (Encoding::Utf8, &b"a\xe2\x82X\n"[..], "a\u{fffd}X", 0, 1),
            (Encoding::Ascii, &b"a\xff\n"[..], "a\u{fffd}", 0, 1),
            (
                Encoding::Utf16Le,
                &[b'a', 0, 0, 0xd8, b'X', 0, 10, 0][..],
                "a\u{fffd}X",
                0,
                1,
            ),
            (
                Encoding::Utf16Be,
                &[0, b'a', 0xd8, 0, 0, b'X', 0, 10][..],
                "a\u{fffd}X",
                0,
                1,
            ),
            (
                Encoding::Utf16Le,
                &[0xef, 0xbb, 0xbf, b'A', 0, 10, 0][..],
                "\u{fffd}A",
                0,
                1,
            ),
            (
                Encoding::Utf8,
                &b"\xef\xbb\xbfa\xff\n"[..],
                "a\u{fffd}",
                3,
                1,
            ),
            (Encoding::Utf8, &b"x\xef\xbb\xbf\n"[..], "x\u{feff}", 0, 0),
        ] {
            let lf_len = if matches!(encoding, Encoding::Utf16Le | Encoding::Utf16Be) {
                2
            } else {
                1
            };
            let end = source.len() - lf_len;
            let body = if raw && count != 0 {
                bytes(&source[body_start..end])
            } else {
                text(rendered)
            };
            let mut frame = expected(
                body,
                range(body_start as u64, end as u64),
                range(0, source.len() as u64),
            );
            frame.malformed_units = count;
            partitioned(config(encoding, policy, 32), source, &[frame]);
        }
    }
}

/// Scenario: Lines exactly fit or exceed either byte bound with multibyte source units.
/// Guarantees: Equality remains unsplit and split boundaries never cut scalars, pairs or replacements.
#[test]
fn exact_limits_and_safe_split_units() {
    for policy in [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ] {
        partitioned(
            config(Encoding::Utf8, policy, 4),
            b"abcd\n",
            &[expected(text("abcd"), range(0, 4), range(0, 5))],
        );
    }
    for (line, record) in [(4, 8), (8, 4), (4, 4)] {
        let mut cfg = config(Encoding::Utf8, OnDecodeError::Replace, 4);
        cfg.max_line_bytes = line;
        cfg.max_record_bytes = record;
        partitioned(
            cfg,
            "a\u{20ac}b\n".as_bytes(),
            &[
                split(
                    expected(text("a\u{20ac}"), range(0, 4), range(0, 4)),
                    0,
                    0,
                    false,
                ),
                split(expected(text("b"), range(4, 5), range(4, 6)), 0, 1, true),
            ],
        );
    }
    partitioned(
        config(Encoding::Utf16Le, OnDecodeError::Replace, 4),
        &[b'a', 0, 0x3d, 0xd8, 0, 0xde, 10, 0],
        &[
            split(expected(text("a"), range(0, 2), range(0, 2)), 0, 0, false),
            split(
                expected(text("\u{1f600}"), range(2, 6), range(2, 8)),
                0,
                1,
                true,
            ),
        ],
    );
    let mut replacement = split(
        expected(text("\u{fffd}x"), range(3, 6), range(3, 7)),
        0,
        1,
        true,
    );
    replacement.malformed_units = 1;
    partitioned(
        config(Encoding::Utf8, OnDecodeError::Replace, 4),
        b"abc\xe2\x82x\n",
        &[
            split(expected(text("abc"), range(0, 3), range(0, 3)), 0, 0, false),
            replacement,
        ],
    );
}

/// Scenario: Preserve-raw UTF-16 splits before later malformed input is observable.
/// Guarantees: Prospective sizing uses max(text, source); every fragment is exact bytes, excluding BOM/LF.
#[test]
fn preserve_raw_split_representation_and_sizing() {
    let mut last = split(
        expected(bytes(&[0, 0xd8, b'C', 0]), range(6, 10), range(6, 12)),
        0,
        1,
        true,
    );
    last.malformed_units = 1;
    // Replacement plus C is four UTF-8 bytes; its source is also four bytes.
    partitioned(
        config(Encoding::Utf16Le, OnDecodeError::PreserveRaw, 4),
        &[0xff, 0xfe, b'A', 0, b'B', 0, 0, 0xd8, b'C', 0, 10, 0],
        &[
            split(
                expected(bytes(&[b'A', 0, b'B', 0]), range(2, 6), range(0, 6)),
                0,
                0,
                false,
            ),
            last,
        ],
    );
    let mut last = split(
        expected(bytes(b"\xffz"), range(4, 6), range(4, 7)),
        0,
        1,
        true,
    );
    last.malformed_units = 1;
    partitioned(
        config(Encoding::Utf8, OnDecodeError::PreserveRaw, 4),
        b"abcd\xffz\n",
        &[
            split(
                expected(bytes(b"abcd"), range(0, 4), range(0, 4)),
                0,
                0,
                false,
            ),
            last,
        ],
    );
}

/// Scenario: Truncate discards valid and malformed units after a bounded prefix.
/// Guarantees: The largest safe prefix owns the entire frame; tail errors count without changing a clean prefix's type.
#[test]
fn truncate_tail_validation_and_ranges() {
    for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
        let mut cfg = config(Encoding::Utf16Le, policy, 4);
        cfg.oversize = OversizeBehavior::Truncate;
        let (body, body_end, discarded) = if policy == OnDecodeError::PreserveRaw {
            (text("AB"), 4, 4)
        } else {
            (text("ABC"), 6, 2)
        };
        let mut frame = expected(body, range(0, body_end), range(0, 10));
        frame.truncated = true;
        frame.discarded_source_bytes = discarded;
        frame.malformed_units = 1;
        partitioned(cfg, &[b'A', 0, b'B', 0, b'C', 0, 0, 0xd8, 10, 0], &[frame]);
    }
    let mut cfg = config(Encoding::Utf8, OnDecodeError::PreserveRaw, 4);
    cfg.oversize = OversizeBehavior::Truncate;
    let mut frame = expected(bytes(b"a\xff"), range(0, 2), range(0, 5));
    frame.truncated = true;
    frame.discarded_source_bytes = 2;
    frame.malformed_units = 1;
    partitioned(cfg, b"a\xffbc\n", &[frame]);
}

/// Scenario: A later line fails decoding, including in a discarded truncate tail.
/// Guarantees: Earlier LF-completed output survives; no prefix or progress of the failed record is returned.
#[test]
fn earlier_output_precedes_sticky_failure() {
    for behavior in [OversizeBehavior::Split, OversizeBehavior::Truncate] {
        let mut cfg = config(Encoding::Utf8, OnDecodeError::Fail, 4);
        cfg.oversize = behavior;
        let mut framer = new(cfg);
        let mut output = Vec::new();
        let failure =
            feed(&mut framer, b"ok\nabcd\xff\n", &mut output).expect_err("expected rejection");
        assert_eq!(output, [expected(text("ok"), range(0, 2), range(0, 3))]);
        assert!(matches!(
            failure.error,
            LineError::Decode(DecodeError::FatalMalformed {
                range: SourceRange { start: 7, end: 8 },
                ..
            })
        ));
        assert_eq!(framer.pending_source_start(), Some(3));
        let terminal = framer
            .next(999, b"anything")
            .expect_err("expected rejection");
        assert_eq!(
            terminal,
            LineFailure {
                consumed: 0,
                error: failure.error
            }
        );
        assert_eq!(
            finish(&mut framer, PartialCompletion::Idle).expect_err("expected rejection"),
            terminal
        );
    }
    let mut cfg = config(Encoding::Utf8, OnDecodeError::Fail, 4);
    cfg.oversize = OversizeBehavior::Truncate;
    let mut framer = new(cfg);
    let mut output = Vec::new();
    let failure =
        feed(&mut framer, b"ok\nabcde\xe2\n", &mut output).expect_err("expected rejection");
    assert_eq!(output.len(), 1);
    assert!(matches!(
        failure.error,
        LineError::Decode(DecodeError::FatalMalformed {
            range: SourceRange { start: 8, end: 9 },
            ..
        })
    ));
    assert_eq!(framer.pending_source_start(), Some(3));
}

/// Scenario: UTF-16 lookahead consumes a later unit before delivering the overflow unit.
/// Guarantees: Output pauses retain consumed bytes; resuming neither duplicates nor loses units.
#[test]
fn consumed_delivered_and_framed_boundaries_differ() {
    let cfg = config(Encoding::Utf16Le, OnDecodeError::Replace, 4);
    let source = [b'A', 0, b'B', 0, 0, 0xd8, b'C', 0, 10, 0];
    let mut framer = new(cfg);
    let mut offset = 0;
    let first = loop {
        let step = framer
            .next(offset as u64, &source[offset..])
            .expect("valid test operation");
        offset += step.consumed;
        if let Some(frame) = step.frame {
            break frame;
        }
    };
    assert_eq!(
        first,
        split(expected(text("AB"), range(0, 4), range(0, 4)), 0, 0, false)
    );
    assert_eq!(offset, 8);
    assert_eq!(framer.decoder.highest_delivered_source_boundary(), 6);
    assert_eq!(framer.decoder.pending_source_start(), Some(6));
    assert_eq!(framer.pending_source_start(), Some(4));
    // Correctable offset errors must not remove the retained overflow unit.
    assert!(framer.next(4, &source[4..]).is_err());
    assert_eq!(framer.terminal_error(), None);
    let mut output = Vec::new();
    feed(&mut framer, &source[offset..], &mut output).expect("valid test operation");
    let mut last = split(
        expected(text("\u{fffd}C"), range(4, 8), range(4, 10)),
        0,
        1,
        true,
    );
    last.malformed_units = 1;
    assert_eq!(output, [last]);
    assert_eq!(framer.pending_source_start(), None);
}

/// Scenario: Temporary EOF interrupts incomplete UTF-8, UTF-16, surrogate pairs and BOM probes.
/// Guarantees: Empty input never flushes; authorized completion owns the exact tail under each decode policy.
#[test]
fn incomplete_units_require_explicit_completion() {
    for (encoding, source, rendered, malformed) in [
        (Encoding::Utf8, &b"a\xe2\x82"[..], "a\u{fffd}", true),
        (
            Encoding::Utf16Le,
            &[b'A', 0, 0, 0xd8, 1][..],
            "A\u{fffd}",
            true,
        ),
        (
            Encoding::Utf16Be,
            &[0, b'A', 0xd8, 0][..],
            "A\u{fffd}",
            true,
        ),
        (Encoding::Utf16Le, &b"A"[..], "\u{fffd}", true),
        (Encoding::Utf8, &[0xef, 0xbb][..], "\u{fffd}", true),
        (Encoding::Ascii, &[0xef][..], "\u{fffd}", true),
        (Encoding::Utf8, &b"abc"[..], "abc", false),
    ] {
        for policy in [
            OnDecodeError::PreserveRaw,
            OnDecodeError::Replace,
            OnDecodeError::Fail,
        ] {
            for reason in [PartialCompletion::Idle, PartialCompletion::PermanentEof] {
                let mut framer = new(config(encoding, policy, 32));
                let mut output = Vec::new();
                for byte in source {
                    feed(&mut framer, &[*byte], &mut output).expect("valid test operation");
                }
                for _ in 0..3 {
                    feed(&mut framer, &[], &mut output).expect("valid test operation");
                }
                assert!(output.is_empty());
                assert_eq!(framer.pending_source_start(), Some(0));
                if policy == OnDecodeError::Fail && malformed {
                    assert!(matches!(
                        finish(&mut framer, reason)
                            .expect_err("expected rejection")
                            .error,
                        LineError::Decode(DecodeError::FatalMalformed { .. })
                    ));
                    assert_eq!(framer.pending_source_start(), Some(0));
                } else {
                    let mut frame = expected(
                        if policy == OnDecodeError::PreserveRaw && malformed {
                            bytes(source)
                        } else {
                            text(rendered)
                        },
                        range(0, source.len() as u64),
                        range(0, source.len() as u64),
                    );
                    frame.ending = match reason {
                        PartialCompletion::Idle => LineEnding::Idle,
                        PartialCompletion::PermanentEof => LineEnding::PermanentEof,
                    };
                    frame.malformed_units = u64::from(malformed);
                    if policy == OnDecodeError::PreserveRaw && !malformed {
                        frame.source_body = Some(source.to_vec());
                    }
                    assert_eq!(
                        finish(&mut framer, reason).expect("valid test operation"),
                        [frame]
                    );
                    assert!(
                        finish(&mut framer, reason)
                            .expect("valid test operation")
                            .is_empty()
                    );
                    assert_eq!(framer.pending_source_start(), None);
                }
            }
        }
    }
}

/// Scenario: A partial scalar is paused or explicitly completed before more input arrives.
/// Guarantees: Pauses preserve units; authorized completion starts fresh units and never probes a later BOM.
#[test]
fn append_after_pause_and_after_completion() {
    let cfg = config(Encoding::Utf8, OnDecodeError::Replace, 32);
    let mut paused = new(cfg);
    let mut output = Vec::new();
    feed(&mut paused, b"\xe2\x82", &mut output).expect("valid test operation");
    feed(&mut paused, b"\xac\n", &mut output).expect("valid test operation");
    assert_eq!(
        output,
        [expected(text("\u{20ac}"), range(0, 3), range(0, 4))]
    );
    let mut completed = new(cfg);
    output.clear();
    feed(&mut completed, b"\xe2\x82", &mut output).expect("valid test operation");
    assert_eq!(
        finish(&mut completed, PartialCompletion::Idle)
            .expect("valid test operation")
            .len(),
        1
    );
    feed(&mut completed, b"\xac\xef\xbb\xbf\n", &mut output).expect("valid test operation");
    let mut frame = expected(text("\u{fffd}\u{feff}"), range(2, 6), range(2, 7));
    frame.malformed_units = 1;
    assert_eq!(output, [frame]);
}

/// Scenario: Completion encounters an overflow replacement and buffered decoder lookahead.
/// Guarantees: Completion can yield repeatedly; fresh input and reason changes are rejected while active.
#[test]
fn completion_drains_pending_work_before_finishing() {
    let mut framer = new(config(Encoding::Utf8, OnDecodeError::Replace, 4));
    let mut output = Vec::new();
    feed(&mut framer, b"abc\xe2", &mut output).expect("valid test operation");
    let step = framer
        .complete_partial(PartialCompletion::PermanentEof)
        .expect("valid test operation");
    assert!(!step.complete);
    assert_eq!(
        step.frame,
        Some(split(
            expected(text("abc"), range(0, 3), range(0, 3)),
            0,
            0,
            false
        ))
    );
    assert_eq!(
        framer.next(4, b"x").expect_err("expected rejection").error,
        LineError::CompletionInProgress
    );
    assert_eq!(
        framer
            .complete_partial(PartialCompletion::Idle)
            .expect_err("expected rejection")
            .error,
        LineError::CompletionInProgress
    );
    let mut last = split(
        expected(text("\u{fffd}"), range(3, 4), range(3, 4)),
        0,
        1,
        true,
    );
    last.ending = LineEnding::PermanentEof;
    last.malformed_units = 1;
    assert_eq!(
        finish(&mut framer, PartialCompletion::PermanentEof).expect("valid test operation"),
        [last]
    );
}

/// Scenario: A fresh framer resumes at each nonfinal split boundary without prior lookahead.
/// Guarantees: Replay preserves remaining bodies, ranges and indices, even at odd UTF-16 offsets.
#[test]
fn continuation_restart_reproduces_remaining_fragments() {
    for (encoding, source) in [
        (Encoding::Utf8, &b"abcdefghij\n"[..]),
        (
            Encoding::Utf16Le,
            &[0xef, 0xbb, 0xbf, b'A', 0, b'B', 0, b'C', 0, b'D', 0, 10, 0][..],
        ),
        (Encoding::Raw, &b"abcdefghij\n"[..]),
    ] {
        let cfg = config(encoding, OnDecodeError::PreserveRaw, 4);
        let mut live = new(cfg);
        let mut frames = Vec::new();
        feed(&mut live, source, &mut frames).expect("valid test operation");
        assert!(frames.len() > 1);
        for (index, frame) in frames.iter().enumerate() {
            if let Some(continuation) = frame.continuation {
                let offset = frame.frame_range.end;
                let mut resumed = LineFramer::new(
                    cfg,
                    LineStart::Continuation {
                        offset,
                        continuation,
                    },
                )
                .expect("valid test operation");
                assert!(
                    finish(&mut resumed, PartialCompletion::Idle)
                        .expect("valid test operation")
                        .is_empty()
                );
                assert!(
                    finish(&mut resumed, PartialCompletion::PermanentEof)
                        .expect("valid test operation")
                        .is_empty()
                );
                let mut remainder = Vec::new();
                feed(&mut resumed, &source[offset as usize..], &mut remainder)
                    .expect("valid test operation");
                assert_eq!(remainder, frames[index + 1..]);
            }
        }
    }
}

/// Scenario: Recovery starts at zero or at a continuation followed by LF or maximum fragment index.
/// Guarantees: Resume keeps BOMs; LF can end an empty fragment; index overflow fails before emission.
#[test]
fn resume_validation_and_fragment_overflow() {
    let cfg = config(Encoding::Utf8, OnDecodeError::PreserveRaw, 4);
    let mut framer = LineFramer::new(cfg, LineStart::ResumeAt(0)).expect("valid test operation");
    let mut output = Vec::new();
    feed(&mut framer, b"\xef\xbb\xbf\n", &mut output).expect("valid test operation");
    let mut frame = expected(text("\u{feff}"), range(0, 3), range(0, 4));
    frame.source_body = Some(b"\xef\xbb\xbf".to_vec());
    assert_eq!(output, [frame]);
    let continuation = LineContinuation {
        record_start_offset: 0,
        next_fragment_index: u32::MAX,
    };
    for source in [&b"\n"[..], &b"abcd\n"[..]] {
        let mut framer = LineFramer::new(
            cfg,
            LineStart::Continuation {
                offset: 4,
                continuation,
            },
        )
        .expect("valid test operation");
        output.clear();
        feed(&mut framer, source, &mut output).expect("valid test operation");
        assert_eq!(output.len(), 1);
        assert_eq!(
            output[0].fragment.expect("valid test operation").index,
            u32::MAX
        );
        assert!(output[0].fragment.expect("valid test operation").is_last);
    }
    let mut framer = LineFramer::new(
        cfg,
        LineStart::Continuation {
            offset: 4,
            continuation,
        },
    )
    .expect("valid test operation");
    output.clear();
    assert_eq!(
        feed(&mut framer, b"abcde", &mut output)
            .expect_err("expected rejection")
            .error,
        LineError::FragmentIndexOverflow
    );
    assert!(output.is_empty());
    assert_eq!(framer.pending_source_start(), Some(4));
    for (offset, index, origin) in [(0, 1, 0), (4, 0, 0), (4, 1, 4), (4, 1, 5)] {
        assert_eq!(
            LineFramer::new(
                cfg,
                LineStart::Continuation {
                    offset,
                    continuation: LineContinuation {
                        record_start_offset: origin,
                        next_fragment_index: index
                    }
                }
            )
            .expect_err("expected rejection"),
            LineError::InvalidContinuation
        );
    }
}

/// Scenario: Invalid bounds or a near-u64 source offset is supplied.
/// Guarantees: Validation rejects unrepresentable settings and offset overflow grants no failed-frame progress.
#[test]
fn bounds_and_offset_overflow() {
    for (encoding, policy, minimum) in [
        (Encoding::Raw, OnDecodeError::Fail, 1),
        (Encoding::Ascii, OnDecodeError::Fail, 1),
        (Encoding::Ascii, OnDecodeError::Replace, 3),
        (Encoding::Utf8, OnDecodeError::PreserveRaw, 4),
        (Encoding::Utf16Le, OnDecodeError::Fail, 4),
    ] {
        assert!(LineFramer::new(config(encoding, policy, minimum), LineStart::NewStream).is_ok());
        for limit in [minimum - 1, usize::MAX] {
            assert_eq!(
                new_error(config(encoding, policy, limit)),
                LineError::InvalidBounds
            );
        }
    }
    let mut framer = LineFramer::new(
        config(Encoding::Raw, OnDecodeError::Fail, 4),
        LineStart::ResumeAt(u64::MAX - 2),
    )
    .expect("valid test operation");
    let mut output = Vec::new();
    let error = feed(&mut framer, b"\naX", &mut output).expect_err("expected rejection");
    assert_eq!(
        output,
        [expected(
            bytes(b""),
            range(u64::MAX - 2, u64::MAX - 2),
            range(u64::MAX - 2, u64::MAX - 1)
        )]
    );
    assert_eq!(
        error.error,
        LineError::Decode(DecodeError::SourceOffsetOverflow)
    );
    assert_eq!(framer.pending_source_start(), Some(u64::MAX - 1));
}

fn new_error(cfg: LineConfig) -> LineError {
    LineFramer::new(cfg, LineStart::NewStream).expect_err("expected rejection")
}

/// Scenario: Millions of unterminated source bytes exceed split/truncate bounds.
/// Guarantees: Capacity stays bounded, truncate tails do not grow buffers, and final output requires completion.
#[test]
fn large_unterminated_input_has_bounded_retention() {
    for behavior in [OversizeBehavior::Split, OversizeBehavior::Truncate] {
        let mut cfg = config(Encoding::Utf8, OnDecodeError::PreserveRaw, 64);
        cfg.oversize = behavior;
        let mut framer = new(cfg);
        let chunk = [b'x'; 4096];
        let mut count = 0;
        for _ in 0..512 {
            let mut input = &chunk[..];
            loop {
                let step = framer
                    .next(framer.next_expected_input_offset(), input)
                    .expect("valid test operation");
                input = &input[step.consumed..];
                assert!(framer.retained_capacity() <= 128);
                if let Some(frame) = step.frame {
                    assert_eq!(frame.ending, LineEnding::Split);
                    assert_eq!(frame.body, bytes(&[b'x'; 64]));
                    count += 1;
                }
                if !step.advanced {
                    break;
                }
            }
        }
        if behavior == OversizeBehavior::Truncate {
            assert_eq!(count, 0);
            let frame = finish(&mut framer, PartialCompletion::Idle)
                .expect("valid test operation")
                .remove(0);
            assert_eq!(frame.body, text(&"x".repeat(64)));
            assert_eq!(frame.discarded_source_bytes, 2097152 - 64);
            assert_eq!(frame.frame_range, range(0, 2097152));
        } else {
            assert_eq!(count, 32767);
            assert!(framer.pending_source_start().is_some());
        }
    }
}

/// Scenario: A valid line precedes malformed input under every partition and encoding.
/// Guarantees: Output precedes failure; consumption counts all accepted bytes without authorizing the failed line.
#[test]
fn failure_order_and_consumption_across_partitions() {
    for (encoding, source, first_end, bad_start) in [
        (Encoding::Utf8, &b"a\nx\xe2\n"[..], 2, 3),
        (Encoding::Ascii, &b"a\nx\xff\n"[..], 2, 3),
        (
            Encoding::Utf16Le,
            &[b'a', 0, 10, 0, b'x', 0, 0, 0xd8, 10, 0][..],
            4,
            6,
        ),
        (
            Encoding::Utf16Be,
            &[0, b'a', 0, 10, 0, b'x', 0xd8, 0, 0, 10][..],
            4,
            6,
        ),
    ] {
        for cuts in 0..(1usize << (source.len() - 1)) {
            let mut framer = new(config(encoding, OnDecodeError::Fail, 4));
            let mut output = Vec::new();
            let mut start = 0;
            let mut failure = None;
            'chunks: for end in 1..=source.len() {
                if end != source.len() && cuts & (1 << (end - 1)) == 0 {
                    continue;
                }
                let mut used = start;
                loop {
                    match framer.next(used as u64, &source[used..end]) {
                        Ok(step) => {
                            used += step.consumed;
                            if let Some(frame) = step.frame {
                                output.push(frame);
                            }
                            if !step.advanced {
                                break;
                            }
                        }
                        Err(error) => {
                            used += error.consumed;
                            assert_eq!(framer.next_expected_input_offset(), used as u64);
                            failure = Some(error.error);
                            break 'chunks;
                        }
                    }
                }
                start = end;
            }
            assert_eq!(
                output,
                [expected(
                    text("a"),
                    range(0, first_end / 2),
                    range(0, first_end)
                )]
            );
            let Some(LineError::Decode(DecodeError::FatalMalformed { range: bad, .. })) = failure
            else {
                panic!("malformed input must fail");
            };
            assert_eq!(bad.start, bad_start);
            assert_eq!(framer.pending_source_start(), Some(first_end));
        }
    }
}

/// Scenario: A truncate scan ends in an incomplete unit, or the source contains only a matching BOM.
/// Guarantees: Only authorized completion emits source ownership; fail policy still suppresses truncated prefixes.
#[test]
fn truncate_incomplete_tail_and_bom_only_completion() {
    for policy in [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ] {
        let mut cfg = config(Encoding::Utf8, policy, 4);
        cfg.oversize = OversizeBehavior::Truncate;
        let mut framer = new(cfg);
        let mut frames = Vec::new();
        feed(&mut framer, b"abcde\xe2", &mut frames).expect("incomplete tail waits");
        assert!(frames.is_empty());
        if policy == OnDecodeError::Fail {
            assert!(finish(&mut framer, PartialCompletion::PermanentEof).is_err());
            assert_eq!(framer.pending_source_start(), Some(0));
        } else {
            let mut frame = expected(text("abcd"), range(0, 4), range(0, 6));
            frame.truncated = true;
            frame.discarded_source_bytes = 2;
            frame.malformed_units = 1;
            frame.ending = LineEnding::PermanentEof;
            if policy == OnDecodeError::PreserveRaw {
                frame.source_body = Some(b"abcd".to_vec());
            }
            assert_eq!(
                finish(&mut framer, PartialCompletion::PermanentEof).expect("complete tail"),
                [frame]
            );
        }
    }
    let mut framer = new(config(Encoding::Utf8, OnDecodeError::PreserveRaw, 4));
    let mut frames = Vec::new();
    feed(&mut framer, b"\xef\xbb\xbf", &mut frames).expect("BOM waits");
    assert!(frames.is_empty());
    let mut frame = expected(text(""), range(3, 3), range(0, 3));
    frame.ending = LineEnding::Idle;
    frame.source_body = Some(Vec::new());
    assert_eq!(
        finish(&mut framer, PartialCompletion::Idle).expect("BOM ownership"),
        [frame]
    );
}
