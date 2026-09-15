// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{
    DecodeError, DecodeEvent, DecodeFailure, DecodedValue, Encoding, OnDecodeError, SourceBytes,
    SourceRange, StreamDecoder,
};

#[path = "boundary_tests.rs"]
mod boundaries;
#[path = "consumer_tests.rs"]
mod consumers;
#[path = "property_tests.rs"]
mod properties;

fn unit(start: u64, source: &[u8], value: DecodedValue, malformed: bool) -> DecodeEvent {
    DecodeEvent::Unit {
        range: SourceRange {
            start,
            end: start + source.len() as u64,
        },
        source: SourceBytes::from_slice(source),
        value,
        malformed,
    }
}

fn encoded_a(encoding: Encoding) -> &'static [u8] {
    match encoding {
        Encoding::Utf8 | Encoding::Ascii => b"A",
        Encoding::Utf16Le => &[b'A', 0],
        Encoding::Utf16Be => &[0, b'A'],
        Encoding::Raw => unreachable!("raw mode does not classify BOMs"),
    }
}

fn decode_chunks(
    encoding: Encoding,
    policy: OnDecodeError,
    source_offset: u64,
    new_stream_start: bool,
    chunks: &[&[u8]],
) -> (Vec<DecodeEvent>, StreamDecoder) {
    let mut decoder = StreamDecoder::new(encoding, policy, source_offset, new_stream_start);
    let mut events = Vec::new();

    for chunk in chunks {
        let mut cursor = 0;
        while cursor < chunk.len() {
            let offset = decoder.next_expected_input_offset();
            let step = decoder
                .next(offset, &chunk[cursor..])
                .expect("test input should decode");
            assert!(
                step.consumed != 0 || step.event.is_some(),
                "nonempty input must make observable progress"
            );
            cursor += step.consumed;
            if let Some(event) = step.event {
                events.push(event);
            }
        }
        drain_events(&mut decoder, &mut events);
    }

    drain_events(&mut decoder, &mut events);
    (events, decoder)
}

fn drain_events(decoder: &mut StreamDecoder, events: &mut Vec<DecodeEvent>) {
    loop {
        let offset = decoder.next_expected_input_offset();
        let step = decoder
            .next(offset, &[])
            .expect("draining test input should decode");
        assert_eq!(step.consumed, 0);
        let Some(event) = step.event else {
            break;
        };
        events.push(event);
    }
}

fn decode_all_partitions(
    encoding: Encoding,
    policy: OnDecodeError,
    new_stream_start: bool,
    data: &[u8],
) -> Vec<DecodeEvent> {
    let baseline = decode_chunks(encoding, policy, 0, new_stream_start, &[data]).0;
    if data.len() <= 1 {
        return baseline;
    }

    let partition_count = 1_usize << (data.len() - 1);
    for mask in 0..partition_count {
        let mut chunks = Vec::new();
        let mut start = 0;
        for boundary in 0..data.len() - 1 {
            if mask & (1 << boundary) != 0 {
                chunks.push(&data[start..=boundary]);
                start = boundary + 1;
            }
        }
        chunks.push(&data[start..]);
        let actual = decode_chunks(encoding, policy, 0, new_stream_start, &chunks).0;
        assert_eq!(actual, baseline, "partition mask {mask:#x}");
    }
    baseline
}

fn fail_for_chunks(
    encoding: Encoding,
    source_offset: u64,
    new_stream_start: bool,
    chunks: &[&[u8]],
) -> DecodeError {
    let mut decoder = StreamDecoder::new(
        encoding,
        OnDecodeError::Fail,
        source_offset,
        new_stream_start,
    );
    for chunk in chunks {
        let mut cursor = 0;
        loop {
            let offset = decoder.next_expected_input_offset();
            match decoder.next(offset, &chunk[cursor..]) {
                Ok(step) => {
                    cursor += step.consumed;
                    if cursor == chunk.len() && step.event.is_none() {
                        break;
                    }
                }
                Err(failure) => return failure.error,
            }
        }
    }
    panic!("test input did not produce a fatal decode error");
}

/// Scenario: Matching UTF-8 and UTF-16 BOMs arrive under every byte partition and policy.
/// Guarantees: Exactly the initial matching BOM is stripped and following text keeps source offsets.
#[test]
fn matching_boms_are_stripped_across_all_partitions() {
    let cases: &[(Encoding, &[u8], &[u8], char)] = &[
        (Encoding::Utf8, &[0xef, 0xbb, 0xbf], b"A", 'A'),
        (Encoding::Utf16Le, &[0xff, 0xfe], &[b'A', 0], 'A'),
        (Encoding::Utf16Be, &[0xfe, 0xff], &[0, b'A'], 'A'),
    ];
    let policies = [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ];

    for &(encoding, bom, payload, scalar) in cases {
        let mut data = bom.to_vec();
        data.extend_from_slice(payload);
        for policy in policies {
            let actual = decode_all_partitions(encoding, policy, true, &data);
            assert_eq!(
                actual,
                vec![
                    DecodeEvent::StrippedBom {
                        range: SourceRange {
                            start: 0,
                            end: bom.len() as u64,
                        },
                        source: SourceBytes::from_slice(bom),
                    },
                    unit(
                        bom.len() as u64,
                        payload,
                        DecodedValue::Scalar(scalar),
                        false,
                    ),
                ]
            );
        }
    }
}

/// Scenario: Every recognized BOM conflicts with each nonmatching configured text encoding.
/// Guarantees: Each policy reports the exact BOM and the decoder retains the configured encoding.
#[test]
fn conflicting_boms_follow_each_decode_error_policy() {
    let boms: &[(Encoding, &[u8])] = &[
        (Encoding::Utf8, &[0xef, 0xbb, 0xbf]),
        (Encoding::Utf16Le, &[0xff, 0xfe]),
        (Encoding::Utf16Be, &[0xfe, 0xff]),
    ];
    let encodings = [
        Encoding::Utf8,
        Encoding::Ascii,
        Encoding::Utf16Le,
        Encoding::Utf16Be,
    ];

    for encoding in encodings {
        for &(matching_encoding, bom) in boms {
            if encoding == matching_encoding {
                continue;
            }
            for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
                let payload = encoded_a(encoding);
                let mut data = bom.to_vec();
                data.extend_from_slice(payload);
                let actual = decode_all_partitions(encoding, policy, true, &data);
                assert_eq!(
                    actual,
                    vec![
                        unit(0, bom, DecodedValue::Scalar('\u{fffd}'), true,),
                        unit(bom.len() as u64, payload, DecodedValue::Scalar('A'), false,),
                    ],
                    "{encoding:?} with {bom:?} under {policy:?}"
                );
            }

            let chunks: Vec<&[u8]> = bom.chunks(1).collect();
            assert_eq!(
                fail_for_chunks(encoding, 0, true, &chunks),
                DecodeError::FatalMalformed {
                    range: SourceRange {
                        start: 0,
                        end: bom.len() as u64,
                    },
                    source_bytes: SourceBytes::from_slice(bom),
                }
            );
        }
    }
}

/// Scenario: A live stream ends a chunk at each proper prefix of a recognized BOM.
/// Guarantees: Partial BOM evidence remains pending under every policy and is never discarded or failed.
#[test]
fn partial_boms_remain_pending_under_every_policy() {
    let prefixes: &[&[u8]] = &[&[0xef], &[0xef, 0xbb], &[0xff], &[0xfe]];
    let encodings = [
        Encoding::Utf8,
        Encoding::Ascii,
        Encoding::Utf16Le,
        Encoding::Utf16Be,
    ];
    let policies = [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ];

    for prefix in prefixes {
        for encoding in encodings {
            for policy in policies {
                let (events, decoder) = decode_chunks(encoding, policy, 0, true, &[prefix]);
                assert!(events.is_empty());
                assert_eq!(decoder.pending_source_start(), Some(0));
                assert_eq!(decoder.highest_delivered_source_boundary(), 0);
                assert_eq!(decoder.next_expected_input_offset(), prefix.len() as u64);
            }
        }
    }
}

/// Scenario: A BOM prefix diverges after one or two bytes in a configured UTF-8 stream.
/// Guarantees: Probe bytes are replayed exactly, malformed prefixes follow policy, and the current byte is reprocessed.
#[test]
fn divergent_bom_prefixes_are_replayed_without_loss() {
    let cases: &[(&[u8], usize)] = &[
        (&[0xef, b'X'], 1),
        (&[0xef, 0xbb, b'X'], 2),
        (&[0xff, b'X'], 1),
        (&[0xfe, b'X'], 1),
    ];
    for &(data, malformed_len) in cases {
        for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
            let actual = decode_all_partitions(Encoding::Utf8, policy, true, data);
            assert_eq!(
                actual,
                vec![
                    unit(
                        0,
                        &data[..malformed_len],
                        DecodedValue::Scalar('\u{fffd}'),
                        true
                    ),
                    unit(
                        malformed_len as u64,
                        &data[malformed_len..],
                        DecodedValue::Scalar('X'),
                        false,
                    ),
                ]
            );
        }
        assert_eq!(
            fail_for_chunks(
                Encoding::Utf8,
                0,
                true,
                &[&data[..malformed_len], &data[malformed_len..]]
            ),
            DecodeError::FatalMalformed {
                range: SourceRange {
                    start: 0,
                    end: malformed_len as u64,
                },
                source_bytes: SourceBytes::from_slice(&data[..malformed_len]),
            }
        );
    }
}

/// Scenario: A BOM probe diverges into valid configured UTF-8 and UTF-16 units.
/// Guarantees: Replayed probe bytes can complete ordinary units under every chunk partition.
#[test]
fn divergent_bom_prefixes_can_form_valid_configured_units() {
    let cases: &[(Encoding, &[u8], char)] = &[
        (Encoding::Utf8, &[0xef, 0x80, 0x80], '\u{f000}'),
        (Encoding::Utf16Le, &[0xff, 0x00], '\u{ff}'),
        (Encoding::Utf16Be, &[0xfe, 0x00], '\u{fe00}'),
    ];

    for &(encoding, data, scalar) in cases {
        for policy in [
            OnDecodeError::PreserveRaw,
            OnDecodeError::Replace,
            OnDecodeError::Fail,
        ] {
            assert_eq!(
                decode_all_partitions(encoding, policy, true, data),
                vec![unit(0, data, DecodedValue::Scalar(scalar), false)]
            );
        }
    }
}

/// Scenario: BOM byte sequences occur after ordinary content or when restart is not a new stream.
/// Guarantees: Midstream BOMs decode as ordinary content and are not stripped under any policy.
#[test]
fn midstream_boms_are_ordinary_content() {
    for policy in [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ] {
        let utf8 = [b'A', 0xef, 0xbb, 0xbf];
        assert_eq!(
            decode_all_partitions(Encoding::Utf8, policy, true, &utf8),
            vec![
                unit(0, b"A", DecodedValue::Scalar('A'), false),
                unit(
                    1,
                    &[0xef, 0xbb, 0xbf],
                    DecodedValue::Scalar('\u{feff}'),
                    false,
                ),
            ]
        );

        let utf16 = [b'A', 0, 0xff, 0xfe];
        assert_eq!(
            decode_all_partitions(Encoding::Utf16Le, policy, false, &utf16),
            vec![
                unit(0, &[b'A', 0], DecodedValue::Scalar('A'), false),
                unit(2, &[0xff, 0xfe], DecodedValue::Scalar('\u{feff}'), false,),
            ]
        );
    }

    let restarted = decode_chunks(
        Encoding::Utf8,
        OnDecodeError::Fail,
        5,
        true,
        &[&[0xef, 0xbb, 0xbf]],
    )
    .0;
    assert_eq!(
        restarted,
        vec![unit(
            5,
            &[0xef, 0xbb, 0xbf],
            DecodedValue::Scalar('\u{feff}'),
            false,
        )]
    );

    let ascii = [b'A', 0xef, 0xbb, 0xbf];
    for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
        assert_eq!(
            decode_all_partitions(Encoding::Ascii, policy, true, &ascii),
            vec![
                unit(0, b"A", DecodedValue::Scalar('A'), false),
                unit(1, &[0xef], DecodedValue::Scalar('\u{fffd}'), true),
                unit(2, &[0xbb], DecodedValue::Scalar('\u{fffd}'), true),
                unit(3, &[0xbf], DecodedValue::Scalar('\u{fffd}'), true),
            ]
        );
    }
    assert_eq!(
        fail_for_chunks(Encoding::Ascii, 0, true, &[&ascii]),
        DecodeError::FatalMalformed {
            range: SourceRange { start: 1, end: 2 },
            source_bytes: SourceBytes::from_slice(&[0xef]),
        }
    );
}

/// Scenario: Valid UTF-8 scalars of every width, NUL, and newline cross every chunk partition.
/// Guarantees: Each scalar has its exact source range and chunking cannot change the event stream.
#[test]
fn utf8_valid_scalars_decode_at_every_partition() {
    let data = [
        0x00, b'A', 0xc2, 0xa2, 0xe2, 0x82, 0xac, 0xf0, 0x9f, 0x98, 0x80, b'\n',
    ];
    let actual = decode_all_partitions(Encoding::Utf8, OnDecodeError::PreserveRaw, false, &data);
    assert_eq!(
        actual,
        vec![
            unit(0, &[0], DecodedValue::Scalar('\0'), false),
            unit(1, b"A", DecodedValue::Scalar('A'), false),
            unit(2, &[0xc2, 0xa2], DecodedValue::Scalar('\u{a2}'), false),
            unit(
                4,
                &[0xe2, 0x82, 0xac],
                DecodedValue::Scalar('\u{20ac}'),
                false,
            ),
            unit(
                7,
                &[0xf0, 0x9f, 0x98, 0x80],
                DecodedValue::Scalar('\u{1f600}'),
                false,
            ),
            unit(11, b"\n", DecodedValue::Scalar('\n'), false),
        ]
    );
}

/// Scenario: UTF-8 contains invalid leads, overlongs, a surrogate, an out-of-range scalar, and bad continuations.
/// Guarantees: Standard UTF-8 error-prefix boundaries are exact and invariant across every chunk partition.
#[test]
fn utf8_malformed_prefixes_are_exact_at_every_partition() {
    let cases: &[(&[u8], &[&[u8]])] = &[
        (&[0x80], &[&[0x80]]),
        (&[0xc0, 0xaf], &[&[0xc0], &[0xaf]]),
        (&[0xe0, 0x80, 0x80], &[&[0xe0], &[0x80], &[0x80]]),
        (&[0xed, 0xa0, 0x80], &[&[0xed], &[0xa0], &[0x80]]),
        (
            &[0xf4, 0x90, 0x80, 0x80],
            &[&[0xf4], &[0x90], &[0x80], &[0x80]],
        ),
        (
            &[0xf5, 0x80, 0x80, 0x80],
            &[&[0xf5], &[0x80], &[0x80], &[0x80]],
        ),
        (&[0xe2, b'(', 0xa1], &[&[0xe2], b"(", &[0xa1]]),
        (&[0xf0, 0x90, b'(', 0x80], &[&[0xf0, 0x90], b"(", &[0x80]]),
    ];

    for &(data, expected_sources) in cases {
        for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
            let actual = decode_all_partitions(Encoding::Utf8, policy, false, data);
            let mut offset = 0;
            let expected: Vec<_> = expected_sources
                .iter()
                .map(|source| {
                    let malformed = !source.iter().all(u8::is_ascii);
                    let value = if malformed {
                        DecodedValue::Scalar('\u{fffd}')
                    } else {
                        DecodedValue::Scalar(char::from(source[0]))
                    };
                    let event = unit(offset, source, value, malformed);
                    offset += source.len() as u64;
                    event
                })
                .collect();
            assert_eq!(actual, expected, "{data:?} under {policy:?}");
        }
    }
}

/// Scenario: A chunk ends after each possible incomplete UTF-8 scalar prefix.
/// Guarantees: Live incomplete prefixes remain pending and the delivered boundary never advances into them.
#[test]
fn utf8_incomplete_trailing_prefixes_remain_pending() {
    for suffix in [&[0xc2][..], &[0xe2, 0x82][..], &[0xf0, 0x90, 0x80][..]] {
        let mut data = vec![b'A'];
        data.extend_from_slice(suffix);
        for policy in [
            OnDecodeError::PreserveRaw,
            OnDecodeError::Replace,
            OnDecodeError::Fail,
        ] {
            let (events, decoder) = decode_chunks(Encoding::Utf8, policy, 0, false, &[&data]);
            assert_eq!(
                events,
                vec![unit(0, b"A", DecodedValue::Scalar('A'), false)]
            );
            assert_eq!(decoder.highest_delivered_source_boundary(), 1);
            assert_eq!(decoder.pending_source_start(), Some(1));
            assert_eq!(decoder.next_expected_input_offset(), data.len() as u64);
        }
    }
}

/// Scenario: ASCII input contains NUL, DEL, and bytes on both sides of the invalid boundary.
/// Guarantees: ASCII preserves control scalars and marks each byte from 0x80 through 0xff separately.
#[test]
fn ascii_nul_and_non_ascii_bytes_are_handled_per_byte() {
    let data = [0, 0x7f, 0x80, 0xff];
    for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
        assert_eq!(
            decode_all_partitions(Encoding::Ascii, policy, false, &data),
            vec![
                unit(0, &[0], DecodedValue::Scalar('\0'), false),
                unit(1, &[0x7f], DecodedValue::Scalar('\u{7f}'), false),
                unit(2, &[0x80], DecodedValue::Scalar('\u{fffd}'), true),
                unit(3, &[0xff], DecodedValue::Scalar('\u{fffd}'), true),
            ]
        );
    }
    assert_eq!(
        fail_for_chunks(Encoding::Ascii, 0, false, &[&data]),
        DecodeError::FatalMalformed {
            range: SourceRange { start: 2, end: 3 },
            source_bytes: SourceBytes::from_slice(&[0x80]),
        }
    );
}

/// Scenario: Little- and big-endian UTF-16 contain BMP text, NUL, newline, and a surrogate pair.
/// Guarantees: Fixed endianness and exact source widths produce the same intended Unicode scalars.
#[test]
fn utf16_endianness_bmp_controls_and_surrogate_pairs_decode() {
    let cases: &[(Encoding, &[u8])] = &[
        (
            Encoding::Utf16Le,
            &[b'A', 0, 0, 0, b'\n', 0, 0x3d, 0xd8, 0x00, 0xde],
        ),
        (
            Encoding::Utf16Be,
            &[0, b'A', 0, 0, 0, b'\n', 0xd8, 0x3d, 0xde, 0x00],
        ),
    ];
    for &(encoding, data) in cases {
        let actual = decode_all_partitions(encoding, OnDecodeError::PreserveRaw, false, data);
        assert_eq!(actual.len(), 4);
        assert_eq!(
            actual.iter().map(event_value).collect::<Vec<_>>(),
            vec!['A', '\0', '\n', '\u{1f600}']
        );
        assert_eq!(
            actual
                .iter()
                .map(|event| event_range(*event))
                .collect::<Vec<_>>(),
            vec![
                SourceRange { start: 0, end: 2 },
                SourceRange { start: 2, end: 4 },
                SourceRange { start: 4, end: 6 },
                SourceRange { start: 6, end: 10 },
            ]
        );
    }
}

fn event_value(event: &DecodeEvent) -> char {
    match event {
        DecodeEvent::Unit {
            value: DecodedValue::Scalar(value),
            ..
        } => *value,
        other => panic!("expected scalar event, got {other:?}"),
    }
}

fn event_range(event: DecodeEvent) -> SourceRange {
    match event {
        DecodeEvent::Unit { range, .. } | DecodeEvent::StrippedBom { range, .. } => range,
    }
}

/// Scenario: UTF-16 ends with an odd byte or a complete high surrogate on a live stream.
/// Guarantees: Neither incomplete form becomes an error and the exposed complete boundary excludes it.
#[test]
fn utf16_live_incomplete_units_remain_pending() {
    let cases: &[(&[u8], u64)] = &[
        (&[b'A', 0, b'B'], 2),
        (&[b'A', 0, 0x3d, 0xd8], 2),
        (&[b'A', 0, 0x3d, 0xd8, 0x00], 2),
    ];
    for &(data, boundary) in cases {
        for policy in [
            OnDecodeError::PreserveRaw,
            OnDecodeError::Replace,
            OnDecodeError::Fail,
        ] {
            let (events, decoder) = decode_chunks(Encoding::Utf16Le, policy, 0, false, &[data]);
            assert_eq!(
                events,
                vec![unit(0, &[b'A', 0], DecodedValue::Scalar('A'), false)]
            );
            assert_eq!(decoder.highest_delivered_source_boundary(), boundary);
            assert_eq!(decoder.pending_source_start(), Some(boundary));
        }
    }
}

/// Scenario: UTF-16 has an isolated low surrogate and a high surrogate followed by a BMP unit.
/// Guarantees: Exact malformed units are emitted once and the following complete unit is reprocessed without loss.
#[test]
fn utf16_lone_and_reprocessed_surrogates_are_exact() {
    let low_then_a = [0x00, 0xdc, b'A', 0];
    let high_then_a = [0x3d, 0xd8, b'A', 0];
    for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
        assert_eq!(
            decode_all_partitions(Encoding::Utf16Le, policy, false, &low_then_a),
            vec![
                unit(0, &[0x00, 0xdc], DecodedValue::Scalar('\u{fffd}'), true,),
                unit(2, &[b'A', 0], DecodedValue::Scalar('A'), false),
            ]
        );
        assert_eq!(
            decode_all_partitions(Encoding::Utf16Le, policy, false, &high_then_a),
            vec![
                unit(0, &[0x3d, 0xd8], DecodedValue::Scalar('\u{fffd}'), true,),
                unit(2, &[b'A', 0], DecodedValue::Scalar('A'), false),
            ]
        );
    }

    let mut decoder = StreamDecoder::new(Encoding::Utf16Le, OnDecodeError::Replace, 0, false);
    let first = decoder.next(0, &high_then_a).expect("first event");
    assert_eq!(first.consumed, 4);
    assert!(matches!(
        first.event,
        Some(DecodeEvent::Unit {
            malformed: true,
            ..
        })
    ));
    assert_eq!(decoder.highest_delivered_source_boundary(), 2);
    assert_eq!(decoder.pending_source_start(), Some(2));
    let replayed = decoder.next(4, &[]).expect("reprocessed event");
    assert_eq!(replayed.consumed, 0);
    assert_eq!(
        replayed.event,
        Some(unit(2, &[b'A', 0], DecodedValue::Scalar('A'), false))
    );
    assert_eq!(decoder.highest_delivered_source_boundary(), 4);
    assert_eq!(decoder.pending_source_start(), None);
}

/// Scenario: One UTF-16 high surrogate is followed by another high surrogate and then a low surrogate.
/// Guarantees: The first unit is malformed while the queued second high surrogate pairs with the low surrogate.
#[test]
fn utf16_queued_high_surrogate_can_start_a_valid_pair() {
    let data = [0x3d, 0xd8, 0x3e, 0xd8, 0x00, 0xdc];
    for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
        assert_eq!(
            decode_all_partitions(Encoding::Utf16Le, policy, false, &data),
            vec![
                unit(0, &[0x3d, 0xd8], DecodedValue::Scalar('\u{fffd}'), true,),
                unit(
                    2,
                    &[0x3e, 0xd8, 0x00, 0xdc],
                    DecodedValue::Scalar('\u{1f800}'),
                    false,
                ),
            ]
        );
    }

    assert_eq!(
        fail_for_chunks(
            Encoding::Utf16Le,
            0,
            false,
            &[&data[..2], &data[2..4], &data[4..]]
        ),
        DecodeError::FatalMalformed {
            range: SourceRange { start: 0, end: 2 },
            source_bytes: SourceBytes::from_slice(&data[..2]),
        }
    );
}

/// Scenario: Fail policy encounters malformed UTF-8, UTF-16, and a conflicting BOM after split input.
/// Guarantees: Each fatal error carries only the exact malformed source range and bytes.
#[test]
fn fail_policy_reports_exact_malformed_evidence() {
    assert_eq!(
        fail_for_chunks(Encoding::Utf8, 10, false, &[&[0xf0], &[0x90], b"("]),
        DecodeError::FatalMalformed {
            range: SourceRange { start: 10, end: 12 },
            source_bytes: SourceBytes::from_slice(&[0xf0, 0x90]),
        }
    );
    assert_eq!(
        fail_for_chunks(
            Encoding::Utf16Le,
            20,
            false,
            &[&[0x3d], &[0xd8, b'A'], &[0]]
        ),
        DecodeError::FatalMalformed {
            range: SourceRange { start: 20, end: 22 },
            source_bytes: SourceBytes::from_slice(&[0x3d, 0xd8]),
        }
    );
    assert_eq!(
        fail_for_chunks(Encoding::Utf16Be, 0, true, &[&[0xef], &[0xbb], &[0xbf]]),
        DecodeError::FatalMalformed {
            range: SourceRange { start: 0, end: 3 },
            source_bytes: SourceBytes::from_slice(&[0xef, 0xbb, 0xbf]),
        }
    );
}

/// Scenario: Raw mode receives BOM-looking bytes, NUL, newline, and an arbitrary high byte.
/// Guarantees: Every source byte is emitted unchanged and raw mode performs no BOM handling.
#[test]
fn raw_mode_emits_every_byte_without_bom_handling() {
    let data = [0xef, 0xbb, 0xbf, 0, b'\n', 0xff];
    for policy in [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ] {
        let actual = decode_all_partitions(Encoding::Raw, policy, true, &data);
        let expected: Vec<_> = data
            .iter()
            .enumerate()
            .map(|(offset, byte)| {
                unit(offset as u64, &[*byte], DecodedValue::RawByte(*byte), false)
            })
            .collect();
        assert_eq!(actual, expected);
    }
}

/// Scenario: The caller supplies a skipped offset and source bytes at the end of the u64 offset domain.
/// Guarantees: Discontinuity is retryable; overflow fails immediately with exact per-call consumption.
#[test]
fn offset_discontinuity_and_overflow_are_reported() {
    let mut decoder = StreamDecoder::new(Encoding::Raw, OnDecodeError::PreserveRaw, 7, false);
    assert_eq!(
        decoder.next(8, b"x"),
        Err(DecodeFailure {
            consumed: 0,
            error: DecodeError::OffsetDiscontinuity {
                expected: 7,
                actual: 8,
            },
        })
    );
    assert!(decoder.terminal_error().is_none());
    assert_eq!(decoder.next(7, b"x").expect("corrected offset").consumed, 1);

    let mut decoder =
        StreamDecoder::new(Encoding::Raw, OnDecodeError::PreserveRaw, u64::MAX, false);
    assert_eq!(
        decoder.next(u64::MAX, b"x"),
        Err(DecodeFailure {
            consumed: 0,
            error: DecodeError::SourceOffsetOverflow,
        })
    );

    let mut decoder = StreamDecoder::new(
        Encoding::Utf16Le,
        OnDecodeError::PreserveRaw,
        u64::MAX - 1,
        false,
    );
    assert_eq!(
        decoder.next(u64::MAX - 1, &[b'A', 0]),
        Err(DecodeFailure {
            consumed: 1,
            error: DecodeError::SourceOffsetOverflow,
        })
    );
    assert_eq!(decoder.pending_source_start(), Some(u64::MAX - 1));
    assert_eq!(
        decoder.next(u64::MAX, &[0]),
        Err(DecodeFailure {
            consumed: 0,
            error: DecodeError::SourceOffsetOverflow,
        })
    );
}

/// Scenario: Mixed valid and malformed streams are decoded one-shot, bytewise, and under every partition.
/// Guarantees: Event order, ranges, source evidence, and malformed markers are chunk-partition independent.
#[test]
fn event_streams_are_deterministic_across_all_chunk_partitions() {
    let cases: &[(Encoding, bool, &[u8])] = &[
        (
            Encoding::Utf8,
            true,
            &[0xef, 0xbb, 0xbf, b'A', 0xe2, b'(', 0xa1, 0],
        ),
        (Encoding::Ascii, true, &[b'A', 0x80, 0, b'\n', 0xff]),
        (
            Encoding::Utf16Le,
            true,
            &[
                0xff, 0xfe, b'A', 0, 0x3d, 0xd8, b'B', 0, 0x00, 0xdc, b'\n', 0,
            ],
        ),
        (
            Encoding::Utf16Be,
            false,
            &[0, b'A', 0xd8, 0x3d, 0xde, 0x00, 0, b'\n'],
        ),
        (Encoding::Raw, true, &[0xef, 0xbb, 0xbf, 0, b'\n', 0xff]),
    ];

    for &(encoding, new_stream_start, data) in cases {
        let baseline =
            decode_all_partitions(encoding, OnDecodeError::Replace, new_stream_start, data);
        let byte_chunks: Vec<&[u8]> = data.chunks(1).collect();
        let bytewise = decode_chunks(
            encoding,
            OnDecodeError::Replace,
            0,
            new_stream_start,
            &byte_chunks,
        )
        .0;
        assert_eq!(bytewise, baseline);
    }
}
