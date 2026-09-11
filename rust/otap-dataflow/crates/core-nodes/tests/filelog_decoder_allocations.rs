// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated allocator measurement; one test prevents harness-thread allocation noise.

use std::{hint::black_box, mem};

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::decoder::{
    Encoding, OnDecodeError, StreamDecoder,
};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn discard(decoder: &mut StreamDecoder, data: &[u8], chunk_size: usize) -> bool {
    for input in data.chunks(chunk_size) {
        let mut used = 0;
        loop {
            match decoder.next(decoder.next_expected_input_offset(), &input[used..]) {
                Ok(step) => {
                    used += step.consumed;
                    let _ = black_box(step.event);
                    assert!(step.consumed <= StreamDecoder::MAX_INPUT_BYTES_PER_CALL);
                    if used == input.len() && step.event.is_none() {
                        break;
                    }
                    assert!(step.consumed != 0 || step.event.is_some());
                }
                Err(failure) => {
                    let _ = black_box(failure);
                    return false;
                }
            }
        }
    }
    true
}

/// Scenario: Clean and malformed streams are decoded/discarded across reused input, both bytewise and in full chunks.
/// Guarantees: Decoder construction, steady state, explicit completion and sticky errors allocate zero heap blocks or bytes.
#[test]
fn filelog_decoder_heap_allocation_is_zero() {
    let mut scratch = [0; 4096];
    let _profiler = dhat::Profiler::builder().testing().build();
    let before = dhat::HeapStats::get();
    for (encoding, pattern) in [
        (Encoding::Utf8, &b"abc\n"[..]),
        (Encoding::Ascii, &b"abc\n"[..]),
        (Encoding::Utf8, &[0xf0, 0x9f, 0x98, 0x80][..]),
        (Encoding::Utf16Le, &[b'A', 0][..]),
        (Encoding::Utf16Be, &[0, b'A'][..]),
        (Encoding::Utf16Le, &[0x3d, 0xd8, 0x00, 0xde][..]),
        (Encoding::Utf16Be, &[0xd8, 0x3d, 0xde, 0x00][..]),
        (Encoding::Raw, &[0xff, 0, b'\r', b'\n'][..]),
        (Encoding::Utf8, &[0xff][..]),
        (Encoding::Ascii, &[0xff][..]),
        (Encoding::Utf16Le, &[0x00, 0xdc][..]),
        (Encoding::Utf16Be, &[0xdc, 0x00][..]),
    ] {
        for (index, byte) in scratch.iter_mut().enumerate() {
            *byte = pattern[index % pattern.len()];
        }
        for policy in [
            OnDecodeError::PreserveRaw,
            OnDecodeError::Replace,
            OnDecodeError::Fail,
        ] {
            for chunk in [1, scratch.len()] {
                let mut decoder = StreamDecoder::new(encoding, policy, 0, true);
                for _ in 0..128 {
                    if !discard(&mut decoder, black_box(&scratch), chunk) {
                        break;
                    }
                }
                let _ = black_box(decoder.finish_incomplete_unit());
                let _ = black_box(decoder.next(decoder.next_expected_input_offset(), &[]));
                assert!(size_of_val(&decoder) <= 192);
                assert!(!mem::needs_drop::<StreamDecoder>());
            }
        }
    }
    for policy in [
        OnDecodeError::PreserveRaw,
        OnDecodeError::Replace,
        OnDecodeError::Fail,
    ] {
        let mut decoder = StreamDecoder::new(Encoding::Utf8, policy, 0, true);
        assert!(discard(&mut decoder, &[0xef, 0xbb], 1));
        let _ = black_box(decoder.finish_incomplete_unit());
        let _ = black_box(decoder.finish_incomplete_unit());
    }
    let after = dhat::HeapStats::get();
    dhat::assert_eq!(after.total_blocks - before.total_blocks, 0);
    dhat::assert_eq!(after.total_bytes - before.total_bytes, 0);
    dhat::assert_eq!(after.curr_bytes, before.curr_bytes);

    // Prove the allocator is active; otherwise a zero count would be vacuous.
    let canary = black_box(Box::new(black_box([0u8; 16])));
    let counted = dhat::HeapStats::get();
    dhat::assert_eq!(counted.total_blocks - after.total_blocks, 1);
    dhat::assert_eq!(counted.total_bytes - after.total_bytes, 16);
    drop(canary);
}
