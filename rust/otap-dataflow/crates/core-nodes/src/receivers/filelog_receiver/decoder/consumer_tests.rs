// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    DecodeError, DecodeEvent, DecodeStart, DecodedValue, Encoding, OnDecodeError, SourceRange,
    StreamDecoder, decode_chunks, unit,
};

/// Scenario: A consumer stops at LF while later malformed input was offered in the same source turn.
/// Guarantees: The earlier complete record is available before failure, and no source beyond its LF is consumed.
#[test]
fn complete_record_precedes_later_error() {
    let input = b"ok\r\n\xff";
    let mut decoder =
        StreamDecoder::new(Encoding::Utf8, OnDecodeError::Fail, DecodeStart::NewStream);
    let mut body = String::new();
    let mut used = 0;
    loop {
        let step = decoder
            .next(used as u64, &input[used..])
            .expect("earlier record");
        used += step.consumed;
        if let Some(DecodeEvent::Unit {
            value: DecodedValue::Scalar(value),
            ..
        }) = step.event
        {
            if value == '\n' {
                break;
            }
            body.push(value);
        }
    }
    assert_eq!(body, "ok\r");
    assert_eq!(used, 4);
    assert_eq!(decoder.highest_delivered_source_boundary(), 4);
    assert!(decoder.terminal_error().is_none());
    let failure = decoder.next(4, &input[used..]).expect_err("later failure");
    assert!(matches!(
        failure.error,
        DecodeError::FatalMalformed {
            range: SourceRange { start: 4, end: 5 },
            ..
        }
    ));
    assert_eq!(body, "ok\r");
    assert_eq!(decoder.highest_delivered_source_boundary(), 4);
}

/// Scenario: A consumer's four-byte UTF-8 output bound fits exactly one UTF-16 surrogate pair.
/// Guarantees: The consumer can stop after the pair, with later malformed units untouched and its pair evidence intact.
#[test]
fn consumer_can_stop_at_an_exact_safe_source_boundary() {
    let data = [0x3d, 0xd8, 0x00, 0xde, 0x00, 0xdc];
    let mut decoder = StreamDecoder::new(
        Encoding::Utf16Le,
        OnDecodeError::Fail,
        DecodeStart::NewStream,
    );
    let step = decoder.next(0, &data).expect("complete pair");
    let event = step.event.expect("pair event");
    assert_eq!(
        event,
        unit(0, &data[..4], DecodedValue::Scalar('\u{1f600}'), false)
    );
    assert_eq!(step.consumed, 4);
    assert_eq!(decoder.highest_delivered_source_boundary(), 4);
    assert_eq!(decoder.pending_source_start(), None);
    assert!(decoder.terminal_error().is_none());
    assert!(decoder.next(4, &data[4..]).is_err());
}

/// Scenario: A bounded consumer retains only a prefix but validates the discarded tail before emitting it.
/// Guarantees: A malformed discarded unit remains fatal and the same-record prefix never becomes emit-ready.
#[test]
fn discard_scan_does_not_hide_later_malformed_units() {
    let input = b"abcde\xe2\n";
    for policy in [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ] {
        let mut decoder = StreamDecoder::new(Encoding::Utf8, policy, DecodeStart::NewStream);
        let mut kept = [0; 4];
        let mut kept_len = 0;
        let mut used = 0;
        let mut malformed = 0;
        let mut ready = false;
        loop {
            match decoder.next(used as u64, &input[used..]) {
                Ok(step) => {
                    used += step.consumed;
                    if let Some(DecodeEvent::Unit {
                        value: DecodedValue::Scalar(value),
                        malformed: bad,
                        ..
                    }) = step.event
                    {
                        malformed += usize::from(bad);
                        if value == '\n' {
                            ready = true;
                            break;
                        }
                        if kept_len < kept.len() {
                            kept[kept_len] = u8::try_from(value).expect("ASCII prefix");
                            kept_len += 1;
                        }
                    }
                }
                Err(failure) => {
                    assert_eq!(policy, OnDecodeError::Fail);
                    assert!(matches!(
                        failure.error,
                        DecodeError::FatalMalformed {
                            range: SourceRange { start: 5, end: 6 },
                            ..
                        }
                    ));
                    break;
                }
            }
        }
        assert_eq!(&kept, b"abcd");
        assert_eq!(ready, policy != OnDecodeError::Fail);
        assert_eq!(malformed, usize::from(policy != OnDecodeError::Fail));
    }
}

/// Scenario: A preserve-raw consumer releases a clean UTF-16 fragment before encountering a later malformed unit.
/// Guarantees: Original bytes of every clean unit are available from the first fragment, without decoder record retention.
#[test]
fn clean_first_fragment_has_exact_raw_evidence() {
    let input = [b'A', 0, b'B', 0, 0x00, 0xdc, b'\n', 0];
    let mut decoder = StreamDecoder::new(
        Encoding::Utf16Le,
        OnDecodeError::PreserveRaw,
        DecodeStart::ResumeAt(0),
    );
    let mut first_fragment = [0; 4];
    for start in [0, 2] {
        let step = decoder
            .next(start as u64, &input[start..])
            .expect("clean unit");
        assert_eq!(step.consumed, 2);
        let event = step.event.expect("unit");
        assert!(matches!(
            event,
            DecodeEvent::Unit {
                malformed: false,
                ..
            }
        ));
        first_fragment[start..start + 2].copy_from_slice(event.source().as_slice());
    }
    assert_eq!(first_fragment, [b'A', 0, b'B', 0]);
    let later = decoder
        .next(4, &input[4..])
        .expect("preserved error")
        .event
        .expect("unit");
    assert_eq!(
        later,
        unit(4, &input[4..6], DecodedValue::Scalar('\u{fffd}'), true)
    );
    assert_eq!(first_fragment, [b'A', 0, b'B', 0]);
}

/// Scenario: The input scratch buffer is overwritten while decoder state is parked as if its descriptor were closed.
/// Guarantees: BOM probes, scalar prefixes and surrogate lookahead own their pending bytes and resume without redecoding.
#[test]
fn pause_and_input_buffer_reuse_preserve_pending_ownership() {
    let cases: &[(Encoding, DecodeStart, &[u8], usize, char)] = &[
        (
            Encoding::Utf8,
            DecodeStart::ResumeAt(0),
            &[0xe2, 0x82, 0xac],
            1,
            '\u{20ac}',
        ),
        (
            Encoding::Utf16Le,
            DecodeStart::ResumeAt(0),
            &[0x3d, 0xd8, 0x00, 0xde],
            3,
            '\u{1f600}',
        ),
        (
            Encoding::Utf16Be,
            DecodeStart::ResumeAt(0),
            &[0xd8, 0x3d, 0xde, 0x00],
            2,
            '\u{1f600}',
        ),
    ];
    for &(encoding, decode_start, bytes, cut, value) in cases {
        let mut scratch = [0; 4];
        scratch[..cut].copy_from_slice(&bytes[..cut]);
        let mut decoder = StreamDecoder::new(encoding, OnDecodeError::Fail, decode_start);
        let first = decoder.next(0, &scratch[..cut]).expect("partial read");
        assert_eq!(first.consumed, cut);
        assert!(first.event.is_none());
        scratch.fill(0xff);
        for _ in 0..5 {
            assert!(
                decoder
                    .next(cut as u64, &[])
                    .expect("parked EOF")
                    .event
                    .is_none()
            );
        }
        scratch[..bytes.len() - cut].copy_from_slice(&bytes[cut..]);
        let step = decoder
            .next(cut as u64, &scratch[..bytes.len() - cut])
            .expect("reopened continuation");
        assert_eq!(
            step.event,
            Some(unit(0, bytes, DecodedValue::Scalar(value), false))
        );
        scratch.fill(0);
        assert_eq!(step.event.expect("owned result").source().as_slice(), bytes);
    }
    let mut scratch = [0xef, 0xbb, 0];
    let mut decoder =
        StreamDecoder::new(Encoding::Utf8, OnDecodeError::Fail, DecodeStart::NewStream);
    assert!(
        decoder
            .next(0, &scratch[..2])
            .expect("partial BOM")
            .event
            .is_none()
    );
    scratch.fill(0xbf);
    let bom = decoder
        .next(2, &scratch[..1])
        .expect("complete BOM")
        .event
        .expect("event");
    assert_eq!(bom.source().as_slice(), &[0xef, 0xbb, 0xbf]);
}

/// Scenario: Text contains NUL and CR, and a UTF-16 unit contains byte 0x0a without being LF.
/// Guarantees: The decoder preserves all content and only decoded U+000A is a textual LF.
#[test]
fn text_controls_and_embedded_lf_byte_are_not_normalized() {
    let bytes = [0x0a, 0x01, 0x00, 0x00, 0x0d, 0x00, 0x0a, 0x00];
    let (events, _) = decode_chunks(
        Encoding::Utf16Le,
        OnDecodeError::Fail,
        DecodeStart::NewStream,
        &[&bytes],
    );
    assert_eq!(
        events,
        vec![
            unit(0, &bytes[..2], DecodedValue::Scalar('\u{10a}'), false),
            unit(2, &bytes[2..4], DecodedValue::Scalar('\0'), false),
            unit(4, &bytes[4..6], DecodedValue::Scalar('\r'), false),
            unit(6, &bytes[6..], DecodedValue::Scalar('\n'), false),
        ]
    );
}

/// Scenario: Each encoding and policy resumes at clean LF boundaries, with BOMs and malformed lookahead.
/// Guarantees: Reconstruction preserves events and source frontiers; both paths stop at the same fatal error.
#[test]
fn clean_line_feed_reconstruction_matches_continuous_decoding() {
    let mut cases = vec![
        (Encoding::Utf8, b"\xef\xbb\xbfa\n\xef\xbb\xbfb\n".to_vec()),
        (Encoding::Utf8, b"a\n\xff\nb\n".to_vec()),
        (Encoding::Utf8, b"\xef\na\n".to_vec()),
        (Encoding::Utf8, b"a\n\xe2\x82\nb\n".to_vec()),
        (Encoding::Ascii, b"a\nb\n".to_vec()),
        (Encoding::Ascii, b"a\n\xff\nb\n".to_vec()),
        (Encoding::Raw, b"\xef\xbb\xbfa\n\xff\nb\n".to_vec()),
    ];
    for encoding in [Encoding::Utf16Le, Encoding::Utf16Be] {
        for units in [
            vec![0xfeff, 0x61, 0x0a, 0xfeff, 0x62, 0x0a],
            // A lone high surrogate consumes LF as lookahead before reporting the error.
            vec![0xfeff, 0x61, 0x0a, 0xd800, 0x0a, 0x62, 0x0a],
        ] {
            let bytes = units
                .into_iter()
                .flat_map(|unit: u16| {
                    if encoding == Encoding::Utf16Le {
                        unit.to_le_bytes()
                    } else {
                        unit.to_be_bytes()
                    }
                })
                .collect();
            cases.push((encoding, bytes));
        }
    }

    for (encoding, data) in cases {
        for policy in [
            OnDecodeError::PreserveRaw,
            OnDecodeError::Replace,
            OnDecodeError::Fail,
        ] {
            for chunk in [1, 2, 3, data.len()] {
                let run = |reconstruct: bool| {
                    let mut decoder = StreamDecoder::new(encoding, policy, DecodeStart::NewStream);
                    let mut events = Vec::new();
                    let mut used = 0;
                    let mut yields = 0;
                    let mut failure = None;
                    // Every call consumes input, emits an event, or terminates this loop.
                    for _ in 0..data.len() * 3 + 8 {
                        let end = (used + chunk).min(data.len());
                        match decoder.next(used as u64, &data[used..end]) {
                            Ok(step) => {
                                used += step.consumed;
                                if let Some(event) = step.event {
                                    let is_lf = matches!(
                                        event,
                                        DecodeEvent::Unit {
                                            value: DecodedValue::Scalar('\n')
                                                | DecodedValue::RawByte(b'\n'),
                                            ..
                                        }
                                    );
                                    events.push(event);
                                    if is_lf {
                                        let boundary = decoder.highest_delivered_source_boundary();
                                        assert_eq!(decoder.pending_source_start(), None);
                                        assert_eq!(decoder.terminal_error(), None);
                                        assert_eq!(boundary, used as u64);
                                        yields += 1;
                                        if reconstruct {
                                            decoder = StreamDecoder::new(
                                                encoding,
                                                policy,
                                                DecodeStart::ResumeAt(boundary),
                                            );
                                        }
                                    }
                                } else if step.consumed == 0 {
                                    assert_eq!(used, data.len());
                                    break;
                                }
                            }
                            Err(error) => {
                                used += error.consumed;
                                failure = Some(error);
                                // Fatal state is observed, never reset to get past malformed input.
                                assert_eq!(decoder.terminal_error(), Some(error.error));
                                break;
                            }
                        }
                    }
                    if encoding == Encoding::Utf8
                        && policy == OnDecodeError::Fail
                        && data == b"\xef\na\n"
                    {
                        assert_eq!(yields, 0);
                        assert!(matches!(
                            failure,
                            Some(super::DecodeFailure {
                                error: DecodeError::FatalMalformed {
                                    range: SourceRange { start: 0, end: 1 },
                                    ..
                                },
                                ..
                            })
                        ));
                    } else {
                        assert!(yields > 0);
                    }
                    assert!(failure.is_some() || used == data.len());
                    (
                        events,
                        failure,
                        used,
                        decoder.next_expected_input_offset(),
                        decoder.highest_delivered_source_boundary(),
                        decoder.pending_source_start(),
                    )
                };
                assert_eq!(
                    run(false),
                    run(true),
                    "{encoding:?} {policy:?} chunk={chunk}"
                );
            }
        }
    }
}
