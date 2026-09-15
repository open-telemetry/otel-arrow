// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{hint::black_box, mem};

use rand::{RngExt, SeedableRng, rngs::StdRng};

use super::super::DecoderState;
use super::{
    DecodeError, DecodeEvent, DecodedValue, Encoding, OnDecodeError, StreamDecoder,
    decode_all_partitions, unit,
};

const ENCODINGS: [Encoding; 5] = [
    Encoding::Utf8,
    Encoding::Ascii,
    Encoding::Utf16Le,
    Encoding::Utf16Be,
    Encoding::Raw,
];
const POLICIES: [OnDecodeError; 3] = [
    OnDecodeError::PreserveRaw,
    OnDecodeError::Replace,
    OnDecodeError::Fail,
];

#[derive(Debug, Eq, PartialEq)]
struct Observation {
    events: Vec<DecodeEvent>,
    error: Option<DecodeError>,
    next: u64,
    delivered: u64,
    pending: Vec<u8>,
    probing: bool,
}

fn pending_bytes(decoder: &StreamDecoder) -> Vec<u8> {
    assert_eq!(decoder.replay.pos, decoder.replay.len, "replay was drained");
    let mut pending = Vec::new();
    if let Some(probe) = &decoder.bom_probe {
        pending.extend_from_slice(probe.as_slice());
    }
    match &decoder.state {
        DecoderState::Utf8(state) => {
            pending.extend_from_slice(&state.bytes[..usize::from(state.len)]);
        }
        DecoderState::Utf16(state) => {
            assert!(state.queued.is_none(), "lookahead was drained");
            if let Some(high) = state.high {
                pending.extend_from_slice(&high.bytes);
            }
            if let Some((_, byte)) = state.odd {
                pending.push(byte);
            }
        }
        DecoderState::Ascii | DecoderState::Raw => {}
    }
    pending
}

fn observe(
    data: &[u8],
    encoding: Encoding,
    policy: OnDecodeError,
    start: u64,
    chunks: &[usize],
    finish: bool,
) -> Observation {
    let mut decoder = StreamDecoder::new(encoding, policy, start, true);
    let mut events = Vec::new();
    let mut error = None;
    let mut used = 0;
    let mut turn = 0;
    let mut delivered = start;
    'input: while used < data.len() {
        let end = (used + chunks[turn % chunks.len()]).min(data.len());
        turn += 1;
        loop {
            let before = decoder.next_expected_input_offset();
            match decoder.next(before, &data[used..end]) {
                Ok(step) => {
                    assert!(step.consumed <= StreamDecoder::MAX_INPUT_BYTES_PER_CALL);
                    assert!(step.consumed <= end - used);
                    assert!(used == end || step.consumed != 0 || step.event.is_some());
                    used += step.consumed;
                    assert_eq!(
                        decoder.next_expected_input_offset(),
                        before
                            .checked_add(step.consumed as u64)
                            .expect("representable consumption")
                    );
                    if let Some(event) = step.event {
                        assert_eq!(event.range().start, delivered);
                        delivered = event.range().end;
                        events.push(event);
                    }
                    assert_eq!(decoder.highest_delivered_source_boundary(), delivered);
                    assert!(decoder.next_expected_input_offset() - delivered <= 3);
                    if used == end && step.event.is_none() {
                        break;
                    }
                }
                Err(failure) => {
                    assert!(failure.consumed <= StreamDecoder::MAX_INPUT_BYTES_PER_CALL);
                    assert!(failure.consumed <= end - used);
                    used += failure.consumed;
                    assert_eq!(
                        decoder.next_expected_input_offset(),
                        before
                            .checked_add(failure.consumed as u64)
                            .expect("failure consumption")
                    );
                    assert!(decoder.next_expected_input_offset() - delivered <= 4);
                    assert_eq!(decoder.highest_delivered_source_boundary(), delivered);
                    assert_eq!(decoder.terminal_error(), Some(failure.error));
                    assert_eq!(
                        decoder
                            .next(before, b"not retried")
                            .expect_err("sticky")
                            .consumed,
                        0
                    );
                    error = Some(failure.error);
                    break 'input;
                }
            }
        }
    }
    if finish && error.is_none() {
        match decoder.finish_incomplete_unit() {
            Ok(Some(event)) => {
                assert_eq!(event.range().start, delivered);
                events.push(event);
            }
            Ok(None) => {}
            Err(failure) => {
                assert_eq!(failure.consumed, 0);
                error = Some(failure.error);
            }
        }
    }
    assert_eq!(decoder.next_expected_input_offset() - start, used as u64);
    for event in &events {
        let range = event.range();
        let first = usize::try_from(range.start - start).expect("small test range");
        let last = usize::try_from(range.end - start).expect("small test range");
        assert_eq!(event.source().as_slice(), &data[first..last]);
        assert!((1..=4).contains(&event.source().as_slice().len()));
    }
    let pending = if error.is_none() {
        let pending = pending_bytes(&decoder);
        let first = usize::try_from(decoder.highest_delivered_source_boundary() - start)
            .expect("small delivered range");
        assert_eq!(pending, &data[first..used]);
        assert_eq!(
            decoder.pending_source_start(),
            (!pending.is_empty()).then_some(decoder.highest_delivered_source_boundary())
        );
        pending
    } else {
        Vec::new()
    };
    Observation {
        events,
        error,
        next: decoder.next_expected_input_offset(),
        delivered: decoder.highest_delivered_source_boundary(),
        pending,
        probing: decoder.bom_probe.is_some(),
    }
}

/// Scenario: Every Unicode scalar is independently encoded as UTF-8 and both UTF-16 byte orders.
/// Guarantees: All scalar widths, supplementary pairs, controls and noncharacters decode with exact source evidence.
#[test]
fn every_unicode_scalar_decodes_with_its_original_width() {
    for code in 0..=0x10ffff {
        let Some(value) = char::from_u32(code) else {
            continue;
        };
        let mut utf8 = [0; 4];
        let bytes = value.encode_utf8(&mut utf8).as_bytes();
        let mut decoder = StreamDecoder::new(Encoding::Utf8, OnDecodeError::Fail, 11, false);
        let step = decoder.next(11, bytes).expect("encoded scalar is valid");
        assert_eq!(step.consumed, bytes.len());
        assert_eq!(
            step.event,
            Some(unit(11, bytes, DecodedValue::Scalar(value), false))
        );

        let mut units = [0; 2];
        let units = value.encode_utf16(&mut units);
        for encoding in [Encoding::Utf16Le, Encoding::Utf16Be] {
            let mut bytes = [0; 4];
            for (index, code_unit) in units.iter().enumerate() {
                let encoded = if encoding == Encoding::Utf16Le {
                    code_unit.to_le_bytes()
                } else {
                    code_unit.to_be_bytes()
                };
                bytes[index * 2..index * 2 + 2].copy_from_slice(&encoded);
            }
            let bytes = &bytes[..units.len() * 2];
            let mut decoder = StreamDecoder::new(encoding, OnDecodeError::Fail, 11, false);
            let step = decoder
                .next(11, bytes)
                .expect("encoded UTF-16 scalar is valid");
            assert_eq!(step.consumed, bytes.len());
            assert_eq!(
                step.event,
                Some(unit(11, bytes, DecodedValue::Scalar(value), false))
            );
        }
    }
}

/// Scenario: Independently specified UTF-8 maximal-subpart vectors include invalid leads, forbidden second bytes and interrupted prefixes.
/// Guarantees: Adjacent errors stay separate and valid successors are never absorbed, under every small input partition.
#[test]
fn additional_unicode_maximal_subpart_vectors_are_independent() {
    let cases: &[(&[u8], &[&[u8]])] = &[
        (&[0x80, 0x80], &[&[0x80], &[0x80]]),
        (&[0xc1, 0xbf], &[&[0xc1], &[0xbf]]),
        (&[0xfe, 0xff], &[&[0xfe], &[0xff]]),
        (&[0xe2, 0x82, b'A'], &[&[0xe2, 0x82], b"A"]),
        (&[0xf1, 0x80, 0x80, b'A'], &[&[0xf1, 0x80, 0x80], b"A"]),
        (
            &[0xf0, 0x8f, 0xbf, 0xbf],
            &[&[0xf0], &[0x8f], &[0xbf], &[0xbf]],
        ),
        (&[0xed, 0xbf, 0xbf], &[&[0xed], &[0xbf], &[0xbf]]),
        (&[0xe0, 0xa0, b'\n'], &[&[0xe0, 0xa0], b"\n"]),
    ];
    for &(data, sources) in cases {
        for policy in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
            let mut start = 0;
            let expected: Vec<_> = sources
                .iter()
                .map(|source| {
                    let clean = source == &b"A".as_slice() || source == &b"\n".as_slice();
                    let value = if clean {
                        char::from(source[0])
                    } else {
                        '\u{fffd}'
                    };
                    let event = unit(start, source, DecodedValue::Scalar(value), !clean);
                    start += source.len() as u64;
                    event
                })
                .collect();
            assert_eq!(
                decode_all_partitions(Encoding::Utf8, policy, false, data),
                expected
            );
        }
    }
}

/// Scenario: All byte values are supplied individually to ASCII decoding at a noninitial offset.
/// Guarantees: 0x00 through 0x7f are unchanged scalars and every higher byte owns one malformed outcome.
#[test]
fn all_ascii_bytes_have_independent_expected_outcomes() {
    for byte in 0..=255 {
        for policy in POLICIES {
            let mut decoder = StreamDecoder::new(Encoding::Ascii, policy, 29, false);
            let result = decoder.next(29, &[byte]);
            if byte > 0x7f && policy == OnDecodeError::Fail {
                let failure = result.expect_err("non-ASCII byte");
                assert_eq!(failure.consumed, 1);
                assert!(matches!(failure.error, DecodeError::FatalMalformed { .. }));
            } else {
                let value = if byte.is_ascii() {
                    char::from(byte)
                } else {
                    '\u{fffd}'
                };
                assert_eq!(
                    result.expect("ASCII policy").event,
                    Some(unit(
                        29,
                        &[byte],
                        DecodedValue::Scalar(value),
                        !byte.is_ascii()
                    ))
                );
            }
        }
    }
}

/// Scenario: Small streams containing incomplete probes, replay and surrogate tails arrive in every possible partition.
/// Guarantees: Events, terminal errors, source frontiers and the actual retained incomplete bytes are partition-independent.
#[test]
fn exhaustive_partitions_preserve_terminal_and_pending_state() {
    let cases: &[(Encoding, &[u8])] = &[
        (Encoding::Utf8, &[0xef, 0xbb]),
        (Encoding::Utf8, &[0xef, 0xbb, b'X', 0xf0, 0x90]),
        (Encoding::Ascii, &[0xef, 0xbb, b'X']),
        (Encoding::Utf16Le, &[0xef, 0xbb, b'X']),
        (Encoding::Utf16Le, &[0x00, 0xd8, 0x01, 0xd8, 0xff]),
        (Encoding::Utf16Be, &[0xd8, 0x00, 0xd8, 0x01, 0xff]),
        (Encoding::Raw, &[0, 0xef, 0xbb, 0xbf, 0xff, b'\n']),
    ];
    for &(encoding, data) in cases {
        for policy in POLICIES {
            for finish in [false, true] {
                let baseline = observe(data, encoding, policy, 0, &[data.len()], finish);
                for mask in 0..(1usize << (data.len() - 1)) {
                    let mut chunks = Vec::new();
                    let mut previous = 0;
                    for boundary in 1..data.len() {
                        if mask & (1 << (boundary - 1)) != 0 {
                            chunks.push(boundary - previous);
                            previous = boundary;
                        }
                    }
                    chunks.push(data.len() - previous);
                    assert_eq!(
                        observe(data, encoding, policy, 0, &chunks, finish),
                        baseline,
                        "{encoding:?} {policy:?} mask={mask} finish={finish}"
                    );
                }
            }
        }
    }
}

/// Scenario: Seeded arbitrary bytes use adversarial chunk schedules and offsets including the u64 ceiling.
/// Guarantees: Bounded consumption, source ownership, malformed grouping, errors and retained state are chunk-invariant.
#[test]
fn seeded_arbitrary_streams_preserve_source_ownership() {
    let mut rng = StdRng::seed_from_u64(0x4649_4c45_4c4f_4701);
    for case in 0..128 {
        let mut data = vec![0; rng.random_range(0..=128)];
        rng.fill(data.as_mut_slice());
        let random_chunks: Vec<_> = (0..17).map(|_| rng.random_range(1..=23)).collect();
        let start = match case % 4 {
            0 => 0,
            1 => rng.random_range(1..=1024),
            2 => u64::MAX - data.len() as u64 - 16,
            _ => u64::MAX - rng.random_range(0..=8),
        };
        for encoding in ENCODINGS {
            for policy in POLICIES {
                let finish = case % 2 == 0;
                let baseline = observe(&data, encoding, policy, start, &[129], finish);
                for chunks in [&[1][..], &[3], &[17], random_chunks.as_slice()] {
                    assert_eq!(
                        observe(&data, encoding, policy, start, chunks, finish),
                        baseline,
                        "case={case}, encoding={encoding:?}, policy={policy:?}, chunks={chunks:?}"
                    );
                }
            }
        }
    }
}

/// Scenario: Arbitrary complete UTF-16 units are decoded using an independent standard-library iterator oracle.
/// Guarantees: Lone surrogates and valid successor/pair decisions agree without sharing the decoder's lookahead implementation.
#[test]
fn utf16_units_match_the_standard_library_oracle() {
    let mut rng = StdRng::seed_from_u64(0x5554_4631_3600_0001);
    for _ in 0..128 {
        let units: Vec<u16> = (0..64).map(|_| rng.random()).collect();
        let expected: Vec<_> = char::decode_utf16(units.iter().copied())
            .map(|result| result.unwrap_or('\u{fffd}'))
            .collect();
        for encoding in [Encoding::Utf16Le, Encoding::Utf16Be] {
            let bytes: Vec<_> = units
                .iter()
                .flat_map(|value| {
                    if encoding == Encoding::Utf16Le {
                        value.to_le_bytes()
                    } else {
                        value.to_be_bytes()
                    }
                })
                .collect();
            let result = observe(
                &bytes,
                encoding,
                OnDecodeError::Replace,
                5,
                &[1, 7, 3],
                true,
            );
            let actual: Vec<_> = result
                .events
                .iter()
                .map(|event| match event {
                    DecodeEvent::Unit {
                        value: DecodedValue::Scalar(value),
                        ..
                    } => *value,
                    other => panic!("expected scalar, got {other:?}"),
                })
                .collect();
            assert_eq!(actual, expected);
            assert!(result.pending.is_empty());
            assert!(result.error.is_none());
        }
    }
}

/// Scenario: A consumer repeatedly discards a long stream while keeping only the decoder and reused scratch.
/// Guarantees: Each call has bounded progress, pending state never grows, and the decoder requires no drop-managed allocation.
#[test]
fn long_discard_scan_retains_constant_state() {
    assert!(!mem::needs_drop::<StreamDecoder>());
    assert!(size_of::<StreamDecoder>() <= 192);
    let scratch = [b'x'; 4096];
    let mut decoder = StreamDecoder::new(Encoding::Utf8, OnDecodeError::Fail, 0, true);
    for _ in 0..1024 {
        let mut used = 0;
        while used < scratch.len() {
            let step = decoder
                .next(
                    decoder.next_expected_input_offset(),
                    black_box(&scratch[used..]),
                )
                .expect("clean discard scan");
            assert_eq!(step.consumed, 1);
            assert_eq!(decoder.pending_source_start(), None);
            used += step.consumed;
            let _ = black_box(step.event);
        }
    }
    assert_eq!(decoder.next_expected_input_offset(), 4 * 1024 * 1024);
    assert_eq!(decoder.highest_delivered_source_boundary(), 4 * 1024 * 1024);
}
