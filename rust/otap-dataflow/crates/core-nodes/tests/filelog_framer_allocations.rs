// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated allocation evidence for bounded physical-line framing.

use std::hint::black_box;

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    decoder::{Encoding, OnDecodeError},
    framer::{LineConfig, LineFramer, LineStart, OversizeBehavior, PartialCompletion},
};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn discard(framer: &mut LineFramer, mut input: &[u8]) {
    loop {
        let step = framer
            .next(framer.next_expected_input_offset(), input)
            .expect("valid source");
        input = &input[step.consumed..];
        let _ = black_box(step.frame);
        if !step.advanced {
            break;
        }
    }
}

/// Scenario: A preserve-raw truncation scans four MiB after filling its bounded prefix.
/// Guarantees: Prefix growth uses at most 10 allocation events; tail scans and transfer allocate nothing.
#[test]
fn truncate_tail_allocates_nothing() {
    let _profiler = dhat::Profiler::builder().testing().build();
    let limit = 1024;
    let mut framer = LineFramer::new(
        LineConfig {
            encoding: Encoding::Utf8,
            on_decode_error: OnDecodeError::PreserveRaw,
            max_line_bytes: limit,
            max_record_bytes: limit,
            oversize: OversizeBehavior::Truncate,
        },
        LineStart::NewStream,
    )
    .expect("valid config");
    let scratch = [b'x'; 4096];
    let before = dhat::HeapStats::get();
    discard(&mut framer, &scratch);
    let filled = dhat::HeapStats::get();
    dhat::assert!(filled.total_blocks - before.total_blocks <= 10);
    dhat::assert!(framer.retained_capacity() <= 2 * limit);
    for _ in 0..1024 {
        discard(&mut framer, black_box(&scratch));
    }
    loop {
        let step = framer
            .complete_partial(PartialCompletion::Idle)
            .expect("authorized completion");
        let _ = black_box(step.frame);
        if step.complete {
            break;
        }
    }
    let after = dhat::HeapStats::get();
    dhat::assert_eq!(after.total_blocks, filled.total_blocks);
    dhat::assert_eq!(after.total_bytes, filled.total_bytes);
    // Prove the allocator instrumentation is live.
    let canary = black_box(Box::new(black_box([0u8; 16])));
    let counted = dhat::HeapStats::get();
    dhat::assert_eq!(counted.total_blocks - after.total_blocks, 1);
    drop(canary);
}
