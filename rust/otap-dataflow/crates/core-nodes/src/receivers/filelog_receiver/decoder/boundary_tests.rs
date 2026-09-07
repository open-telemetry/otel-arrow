// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    DecodeError, DecodeEvent, DecodeFailure, DecodedValue, Encoding, OnDecodeError, SourceBytes,
    SourceRange, StreamDecoder, decode_all_partitions, decode_chunks, drain_events, encoded_a,
    unit,
};

const POLICIES: [OnDecodeError; 3] = [
    OnDecodeError::PreserveRaw,
    OnDecodeError::Replace,
    OnDecodeError::Fail,
];
const TEXT_ENCODINGS: [Encoding; 4] = [
    Encoding::Utf8,
    Encoding::Ascii,
    Encoding::Utf16Le,
    Encoding::Utf16Be,
];

fn assert_terminal(decoder: &mut StreamDecoder, error: DecodeError) {
    let next = decoder.next_expected_input_offset();
    let delivered = decoder.highest_delivered_source_boundary();
    let pending = decoder.pending_source_start();
    let repeated = DecodeFailure { consumed: 0, error };
    assert_eq!(decoder.terminal_error(), Some(error));
    for _ in 0..3 {
        assert_eq!(decoder.next(next, &[]), Err(repeated));
        assert_eq!(decoder.next(next, b"ignored"), Err(repeated));
        assert_eq!(decoder.next(0, b"wrong offset"), Err(repeated));
        assert_eq!(decoder.finish_incomplete_unit(), Err(repeated));
    }
    assert_eq!(decoder.next_expected_input_offset(), next);
    assert_eq!(decoder.highest_delivered_source_boundary(), delivered);
    assert_eq!(decoder.pending_source_start(), pending);
}

/// Scenario: No bytes arrive, including repeated caller-authorized boundaries on an empty stream.
/// Guarantees: No output or failure is invented, and the first actual matching BOM is still recognized.
#[test]
fn empty_stream_does_not_manufacture_output_or_reset_bom_state() {
    for encoding in TEXT_ENCODINGS.into_iter().chain([Encoding::Raw]) {
        for policy in POLICIES {
            let mut decoder = StreamDecoder::new(encoding, policy, 0, true);
            for _ in 0..3 {
                let step = decoder.next(0, &[]).expect("empty input");
                assert_eq!(step.consumed, 0);
                assert!(step.event.is_none());
                assert_eq!(decoder.finish_incomplete_unit(), Ok(None));
                assert_eq!(decoder.pending_source_start(), None);
            }
            if encoding == Encoding::Utf8 {
                let event = decoder
                    .next(0, &[0xef, 0xbb, 0xbf])
                    .expect("first actual source bytes")
                    .event
                    .expect("BOM event");
                assert!(matches!(event, DecodeEvent::StrippedBom { .. }));
                assert_eq!(event.source().as_slice(), &[0xef, 0xbb, 0xbf]);
            }
        }
    }
}

/// Scenario: Every textual encoding stops at a proper prefix of any recognized initial BOM.
/// Guarantees: Temporary EOF preserves the probe; explicit completion applies one policy unit to all probe bytes.
#[test]
fn eligible_partial_bom_is_atomic_in_every_text_encoding() {
    for encoding in TEXT_ENCODINGS {
        for policy in POLICIES {
            for prefix in [&[0xef][..], &[0xef, 0xbb], &[0xff], &[0xfe]] {
                let (events, mut decoder) = decode_chunks(encoding, policy, 0, true, &[prefix]);
                assert!(events.is_empty());
                for _ in 0..3 {
                    let step = decoder
                        .next(prefix.len() as u64, &[])
                        .expect("temporary EOF");
                    assert!(step.event.is_none());
                    assert_eq!(decoder.pending_source_start(), Some(0));
                    assert_eq!(decoder.highest_delivered_source_boundary(), 0);
                }
                let range = SourceRange {
                    start: 0,
                    end: prefix.len() as u64,
                };
                if policy == OnDecodeError::Fail {
                    let error = DecodeError::FatalMalformed {
                        range,
                        source_bytes: SourceBytes::from_slice(prefix),
                    };
                    assert_eq!(
                        decoder.finish_incomplete_unit(),
                        Err(DecodeFailure { consumed: 0, error })
                    );
                    assert_terminal(&mut decoder, error);
                } else {
                    assert_eq!(
                        decoder.finish_incomplete_unit(),
                        Ok(Some(unit(
                            0,
                            prefix,
                            DecodedValue::Scalar('\u{fffd}'),
                            true
                        )))
                    );
                    assert_eq!(decoder.pending_source_start(), None);
                    assert_eq!(decoder.highest_delivered_source_boundary(), range.end);
                    assert_eq!(decoder.finish_incomplete_unit(), Ok(None));
                    let step = decoder
                        .next(range.end, encoded_a(encoding))
                        .expect("continuation after an eligible boundary");
                    assert_eq!(
                        step.event,
                        Some(unit(
                            range.end,
                            encoded_a(encoding),
                            DecodedValue::Scalar('A'),
                            false
                        ))
                    );
                }
            }
        }
    }
}

/// Scenario: An incomplete scalar, code unit, or surrogate pair remains at a caller-established boundary.
/// Guarantees: The entire tail owns one policy outcome; ordinary empty reads do not resolve it.
#[test]
fn eligible_encoding_tails_cover_their_exact_source_range() {
    let cases: &[(Encoding, &[u8])] = &[
        (Encoding::Utf8, &[0xc2]),
        (Encoding::Utf8, &[0xe2]),
        (Encoding::Utf8, &[0xe2, 0x82]),
        (Encoding::Utf8, &[0xf0, 0x90, 0x80]),
        (Encoding::Utf16Le, &[0xff]),
        (Encoding::Utf16Be, &[0xff]),
        (Encoding::Utf16Le, &[0x00, 0xd8]),
        (Encoding::Utf16Be, &[0xd8, 0x00]),
        (Encoding::Utf16Le, &[0x00, 0xd8, 0xff]),
        (Encoding::Utf16Be, &[0xd8, 0x00, 0xff]),
    ];
    for &(encoding, tail) in cases {
        for policy in POLICIES {
            let (events, mut decoder) = decode_chunks(encoding, policy, 17, false, &[tail]);
            assert!(events.is_empty());
            assert_eq!(decoder.pending_source_start(), Some(17));
            assert_eq!(decoder.highest_delivered_source_boundary(), 17);
            let end = 17 + tail.len() as u64;
            assert!(
                decoder
                    .next(end, &[])
                    .expect("temporary EOF")
                    .event
                    .is_none()
            );
            if policy == OnDecodeError::Fail {
                let error = DecodeError::FatalMalformed {
                    range: SourceRange { start: 17, end },
                    source_bytes: SourceBytes::from_slice(tail),
                };
                assert_eq!(
                    decoder.finish_incomplete_unit(),
                    Err(DecodeFailure { consumed: 0, error })
                );
                assert_terminal(&mut decoder, error);
            } else {
                assert_eq!(
                    decoder.finish_incomplete_unit(),
                    Ok(Some(unit(17, tail, DecodedValue::Scalar('\u{fffd}'), true)))
                );
                assert_eq!(decoder.highest_delivered_source_boundary(), end);
                assert_eq!(decoder.pending_source_start(), None);
                assert_eq!(decoder.finish_incomplete_unit(), Ok(None));
            }
        }
    }
}

/// Scenario: An odd UTF-16 byte or high-surrogate-plus-odd tail is resolved before new input arrives.
/// Guarantees: Both byte orders and nonfailing policies start the next clean unit at the resolved frontier without reusing tail bytes.
#[test]
fn resolved_utf16_tails_allow_clean_continuation() {
    let assert_state =
        |decoder: &StreamDecoder, next: u64, delivered: u64, pending: Option<u64>| {
            assert_eq!(decoder.next_expected_input_offset(), next);
            assert_eq!(decoder.highest_delivered_source_boundary(), delivered);
            assert_eq!(decoder.pending_source_start(), pending);
            assert_eq!(decoder.terminal_error(), None);
        };

    for (encoding, high_tail) in [
        (Encoding::Utf16Le, [0x00, 0xd8, 0xff]),
        (Encoding::Utf16Be, [0xd8, 0x00, 0xff]),
    ] {
        for (tail, tail_end, a_end) in [(&[0xff][..], 18, 20), (high_tail.as_slice(), 20, 22)] {
            for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
                let mut decoder = StreamDecoder::new(encoding, policy, 17, false);
                assert_state(&decoder, 17, 17, None);

                let step = decoder.next(17, tail).expect("incomplete tail");
                assert_eq!(step.consumed, tail.len());
                assert_eq!(step.event, None);
                assert_state(&decoder, tail_end, 17, Some(17));

                let step = decoder.next(tail_end, &[]).expect("temporary EOF");
                assert_eq!(step.consumed, 0);
                assert_eq!(step.event, None);
                assert_state(&decoder, tail_end, 17, Some(17));

                let event = decoder
                    .finish_incomplete_unit()
                    .expect("caller-authorized completion")
                    .expect("one malformed tail event");
                assert_eq!(
                    event,
                    unit(17, tail, DecodedValue::Scalar('\u{fffd}'), true)
                );
                assert_eq!(
                    event.range(),
                    SourceRange {
                        start: 17,
                        end: tail_end
                    }
                );
                assert_state(&decoder, tail_end, tail_end, None);

                assert_eq!(decoder.finish_incomplete_unit(), Ok(None));
                assert_state(&decoder, tail_end, tail_end, None);

                let step = decoder
                    .next(tail_end, encoded_a(encoding))
                    .expect("fresh unit after the resolved tail");
                assert_eq!(step.consumed, 2);
                assert_eq!(
                    step.event,
                    Some(unit(
                        tail_end,
                        encoded_a(encoding),
                        DecodedValue::Scalar('A'),
                        false
                    ))
                );
                assert_eq!(
                    step.event.expect("clean A event").range(),
                    SourceRange {
                        start: tail_end,
                        end: a_end
                    }
                );
                assert_state(&decoder, a_end, a_end, None);

                assert_eq!(decoder.finish_incomplete_unit(), Ok(None));
                assert_state(&decoder, a_end, a_end, None);
            }
        }
    }
}

/// Scenario: A high surrogate has already consumed a complete nonsurrogate or another high surrogate.
/// Guarantees: Premature completion is rejected without losing the queued unit, which can be drained normally.
#[test]
fn queued_utf16_units_must_be_drained_before_completion() {
    for (encoding, high, valid) in [
        (Encoding::Utf16Le, [0x00, 0xd8], [b'A', 0]),
        (Encoding::Utf16Be, [0xd8, 0x00], [0, b'A']),
    ] {
        for next_unit in [valid, high] {
            for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
                let data = [high[0], high[1], next_unit[0], next_unit[1]];
                let mut decoder = StreamDecoder::new(encoding, policy, 0, false);
                let first = decoder.next(0, &data).expect("malformed first surrogate");
                assert_eq!(first.consumed, 4);
                assert_eq!(
                    first.event,
                    Some(unit(0, &high, DecodedValue::Scalar('\u{fffd}'), true))
                );
                assert_eq!(
                    decoder.finish_incomplete_unit(),
                    Err(DecodeFailure {
                        consumed: 0,
                        error: DecodeError::DrainRequired,
                    })
                );
                assert_eq!(decoder.next_expected_input_offset(), 4);
                assert_eq!(decoder.highest_delivered_source_boundary(), 2);
                assert!(decoder.terminal_error().is_none());
                let step = decoder.next(4, &[]).expect("drain queued unit");
                assert_eq!(step.consumed, 0);
                if next_unit == valid {
                    assert_eq!(
                        step.event,
                        Some(unit(2, &valid, DecodedValue::Scalar('A'), false))
                    );
                    assert_eq!(decoder.finish_incomplete_unit(), Ok(None));
                } else {
                    assert!(step.event.is_none());
                    assert_eq!(
                        decoder.finish_incomplete_unit(),
                        Ok(Some(unit(2, &high, DecodedValue::Scalar('\u{fffd}'), true)))
                    );
                }
                assert_eq!(decoder.highest_delivered_source_boundary(), 4);
                assert_eq!(decoder.pending_source_start(), None);
            }
        }
    }
}

/// Scenario: A divergent initial BOM probe leaves replay bytes after its first delivered event.
/// Guarantees: Completion cannot skip either a complete replay event or a replay byte that becomes an odd UTF-16 tail.
#[test]
fn divergent_bom_replay_must_be_drained_before_completion() {
    let data = [0xef, 0xbb, b'X'];
    for encoding in [Encoding::Utf8, Encoding::Ascii, Encoding::Utf16Le] {
        let mut decoder = StreamDecoder::new(encoding, OnDecodeError::Replace, 0, true);
        let first = decoder.next(0, &data).expect("divergent BOM");
        assert_eq!(first.consumed, 3);
        let mut events = vec![first.event.expect("first unit")];
        assert_eq!(
            decoder.finish_incomplete_unit(),
            Err(DecodeFailure {
                consumed: 0,
                error: DecodeError::DrainRequired,
            })
        );
        drain_events(&mut decoder, &mut events);
        if let Some(event) = decoder.finish_incomplete_unit().expect("drained boundary") {
            events.push(event);
        }
        let expected = match encoding {
            Encoding::Utf8 => vec![
                unit(0, &data[..2], DecodedValue::Scalar('\u{fffd}'), true),
                unit(2, b"X", DecodedValue::Scalar('X'), false),
            ],
            Encoding::Ascii => vec![
                unit(0, &data[..1], DecodedValue::Scalar('\u{fffd}'), true),
                unit(1, &data[1..2], DecodedValue::Scalar('\u{fffd}'), true),
                unit(2, b"X", DecodedValue::Scalar('X'), false),
            ],
            Encoding::Utf16Le => vec![
                unit(0, &data[..2], DecodedValue::Scalar('\u{bbef}'), false),
                unit(2, b"X", DecodedValue::Scalar('\u{fffd}'), true),
            ],
            _ => unreachable!("table covers three replay cases"),
        };
        assert_eq!(events, expected);
        assert_eq!(decoder.highest_delivered_source_boundary(), 3);
        assert_eq!(decoder.pending_source_start(), None);
    }
}

/// Scenario: Fatal input occurs in each decoding path, including BOM replay and consumed UTF-16 lookahead.
/// Guarantees: Failures report exact consumption and evidence, never advance delivery, and cannot be retried past.
#[test]
fn fatal_decode_paths_latch_error_and_consumption() {
    let cases: &[(Encoding, bool, &[u8], usize, usize)] = &[
        (Encoding::Utf8, false, &[0xff], 1, 1),
        (Encoding::Ascii, false, &[0xff], 1, 1),
        (Encoding::Utf8, false, &[0xe2, 0x82, b'A'], 2, 2),
        (Encoding::Utf16Le, false, &[0x00, 0xdc], 2, 2),
        (Encoding::Utf16Be, false, &[0xdc, 0x00], 2, 2),
        (Encoding::Utf16Le, false, &[0x00, 0xd8, b'A', 0], 4, 2),
        (Encoding::Utf16Be, false, &[0xd8, 0x00, 0, b'A'], 4, 2),
        (Encoding::Ascii, true, &[0xef, 0xbb, b'X'], 3, 1),
        (Encoding::Utf16Be, true, &[0xef, 0xbb, 0xbf], 3, 3),
        (Encoding::Utf8, true, &[0xff, 0xfe], 2, 2),
    ];
    for &(encoding, new_stream, data, consumed, malformed_len) in cases {
        let start = if new_stream { 0 } else { 10 };
        let mut decoder = StreamDecoder::new(encoding, OnDecodeError::Fail, start, new_stream);
        let failure = decoder.next(start, data).expect_err("malformed unit");
        let error = DecodeError::FatalMalformed {
            range: SourceRange {
                start,
                end: start + malformed_len as u64,
            },
            source_bytes: SourceBytes::from_slice(&data[..malformed_len]),
        };
        assert_eq!(failure, DecodeFailure { consumed, error });
        assert_eq!(
            decoder.next_expected_input_offset(),
            start + consumed as u64
        );
        assert_eq!(decoder.highest_delivered_source_boundary(), start);
        assert_terminal(&mut decoder, error);
    }
}

/// Scenario: A malformed prefix starts in earlier input, with an incompatible valid byte in the current call.
/// Guarantees: Zero current-call consumption does not lose the prior range or authorize skipping the valid successor.
#[test]
fn fatal_incomplete_prefix_does_not_consume_valid_successor() {
    let mut decoder = StreamDecoder::new(Encoding::Utf8, OnDecodeError::Fail, 100, false);
    assert!(
        decoder
            .next(100, &[0xe2, 0x82])
            .expect("prefix")
            .event
            .is_none()
    );
    let failure = decoder.next(102, b"A").expect_err("incompatible successor");
    assert_eq!(failure.consumed, 0);
    assert_eq!(
        failure.error,
        DecodeError::FatalMalformed {
            range: SourceRange {
                start: 100,
                end: 102
            },
            source_bytes: SourceBytes::from_slice(&[0xe2, 0x82]),
        }
    );
    assert_eq!(decoder.next_expected_input_offset(), 102);
    assert_terminal(&mut decoder, failure.error);
}

/// Scenario: Input reaches the largest representable half-open source boundary, possibly mid-unit.
/// Guarantees: Earlier events remain delivered, overflow consumption is exact, and overflow cannot become a terminal replacement.
#[test]
fn offset_overflow_is_immediate_and_sticky_without_wrapping() {
    for encoding in [Encoding::Utf8, Encoding::Ascii, Encoding::Raw] {
        let mut decoder = StreamDecoder::new(encoding, OnDecodeError::Replace, u64::MAX - 1, false);
        let step = decoder
            .next(u64::MAX - 1, b"AB")
            .expect("last representable byte");
        assert_eq!(step.consumed, 1);
        assert_eq!(step.event.expect("A").range().end, u64::MAX);
        let failure = decoder.next(u64::MAX, b"B").expect_err("overflow");
        assert_eq!(failure.consumed, 0);
        assert_terminal(&mut decoder, DecodeError::SourceOffsetOverflow);
    }
    for policy in POLICIES {
        for (encoding, data) in [
            (Encoding::Utf8, &[0xf0, 0x90, 0x80, 0x80][..]),
            (Encoding::Utf16Le, &[0x00, 0xd8, 0x00, 0xdc][..]),
            (Encoding::Utf16Be, &[0xd8, 0x00, 0xdc, 0x00][..]),
        ] {
            for available in 1..4 {
                let start = u64::MAX - available;
                let mut decoder = StreamDecoder::new(encoding, policy, start, false);
                let failure = decoder
                    .next(start, data)
                    .expect_err("unit crosses offset ceiling");
                assert_eq!(failure.consumed as u64, available);
                assert_eq!(failure.error, DecodeError::SourceOffsetOverflow);
                assert_eq!(decoder.next_expected_input_offset(), u64::MAX);
                assert_eq!(decoder.highest_delivered_source_boundary(), start);
                assert_terminal(&mut decoder, failure.error);
            }
        }
    }
}

/// Scenario: BOM-shaped content follows a pause or an explicit new-stream reconstruction.
/// Guarantees: Only reconstruction at offset zero re-enables stripping; resume never rewinds or aligns the supplied offset.
#[test]
fn new_stream_reset_is_distinct_from_pause_and_resume() {
    let bom = [0xef, 0xbb, 0xbf];
    let mut decoder = StreamDecoder::new(Encoding::Utf8, OnDecodeError::Fail, 0, true);
    assert!(matches!(
        decoder.next(0, &bom).expect("initial BOM").event,
        Some(DecodeEvent::StrippedBom { .. })
    ));
    assert!(decoder.next(3, &[]).expect("pause").event.is_none());
    assert_eq!(
        decoder.next(3, &bom).expect("ordinary content").event,
        Some(unit(3, &bom, DecodedValue::Scalar('\u{feff}'), false))
    );
    decoder = StreamDecoder::new(Encoding::Utf8, OnDecodeError::Fail, 0, true);
    assert!(matches!(
        decoder.next(0, &bom).expect("explicit new stream").event,
        Some(DecodeEvent::StrippedBom { .. })
    ));
    for start in [0, 3, 91] {
        let mut resumed = StreamDecoder::new(Encoding::Utf16Le, OnDecodeError::Fail, start, false);
        assert_eq!(
            resumed
                .next(start, &[0xff, 0xfe])
                .expect("exact resume")
                .event,
            Some(unit(
                start,
                &[0xff, 0xfe],
                DecodedValue::Scalar('\u{feff}'),
                false
            ))
        );
    }
}

/// Scenario: Input resembles an unsupported UTF-32 signature or a reversed BOM away from stream start.
/// Guarantees: Recognition is limited to the three declared signatures; ordinary noncharacters are valid content.
#[test]
fn bom_recognizer_is_explicit_and_does_not_reject_noncharacters() {
    let events = decode_all_partitions(
        Encoding::Utf16Le,
        OnDecodeError::Fail,
        true,
        &[0xff, 0xfe, 0, 0],
    );
    assert_eq!(
        events,
        vec![
            DecodeEvent::StrippedBom {
                range: SourceRange { start: 0, end: 2 },
                source: SourceBytes::from_slice(&[0xff, 0xfe]),
            },
            unit(2, &[0, 0], DecodedValue::Scalar('\0'), false),
        ]
    );
    for (encoding, source) in [
        (Encoding::Utf16Le, [0xfe, 0xff]),
        (Encoding::Utf16Be, [0xff, 0xfe]),
    ] {
        let (events, _) = decode_chunks(encoding, OnDecodeError::Fail, 7, true, &[&source]);
        assert_eq!(
            events,
            vec![unit(7, &source, DecodedValue::Scalar('\u{fffe}'), false)]
        );
    }
}
