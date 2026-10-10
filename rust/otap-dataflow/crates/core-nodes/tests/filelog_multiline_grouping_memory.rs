// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation checks for grouping payloads, excluding worker regex scratch.

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    decoder::{Encoding, OnDecodeError},
    framer::{LineConfig, OversizeBehavior},
    multiline::{GroupingMode, MultilineConfig, MultilineFramer, RecordStart},
    multiline_pattern::{
        BoundaryMatcher, BoundaryPattern, PatternInput, PatternLimits, PatternMode,
    },
};
use std::hint::black_box;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn feed(framer: &mut MultilineFramer, matcher: &mut BoundaryMatcher, mut input: &[u8]) {
    loop {
        let step = framer
            .next(framer.next_expected_input_offset(), input, matcher)
            .expect("framing");
        input = &input[step.consumed..];
        let _ = black_box(step.frame);
        if !step.advanced {
            assert!(input.is_empty());
            break;
        }
    }
}

/// Scenario: Lines fit or overflow records under preserve/replace, followed by a long oversized tail.
/// Guarantees: Measured payload peaks fit the configured estimate and tail scans allocate nothing.
#[test]
fn grouping_payload_and_discard_are_bounded() {
    let input = [b'a'; 4096];
    for max_record_bytes in [1024, 32 * 1024] {
        for on_decode_error in [OnDecodeError::PreserveRaw, OnDecodeError::Replace] {
            for policy in [OversizeBehavior::Split, OversizeBehavior::Truncate] {
                let config = MultilineConfig {
                    line: LineConfig {
                        encoding: Encoding::Utf8,
                        on_decode_error,
                        max_line_bytes: 4096,
                        max_record_bytes,
                        oversize: policy,
                    },
                    max_lines: 100,
                    mode: GroupingMode::End,
                };
                let pattern = BoundaryPattern::compile_with_limits(
                    "^END$",
                    PatternMode::Text,
                    PatternLimits {
                        lazy_cache_bytes: 0,
                        ..PatternLimits::default()
                    },
                )
                .expect("pattern");
                let mut matcher = pattern.matcher();
                let _ = matcher
                    .is_match(PatternInput::Text("warm the worker scratch"))
                    .expect("warm cache");
                let profiler = dhat::Profiler::builder().testing().build();
                let mut framer =
                    MultilineFramer::new(config, pattern, RecordStart::NewStream).expect("framer");
                feed(&mut framer, &mut matcher, b"prefix\n");
                feed(&mut framer, &mut matcher, &input);
                feed(&mut framer, &mut matcher, b"\n");
                feed(&mut framer, &mut matcher, &input);
                feed(&mut framer, &mut matcher, b"a");
                let before = dhat::HeapStats::get();
                let retained = framer.retained_capacity();
                for _ in 0..64 {
                    feed(&mut framer, &mut matcher, &input);
                }
                let after = dhat::HeapStats::get();
                if policy == OversizeBehavior::Truncate {
                    assert_eq!(before.total_blocks, after.total_blocks);
                    assert_eq!(retained, framer.retained_capacity());
                }
                feed(&mut framer, &mut matcher, b"\n");
                let peak = dhat::HeapStats::get().max_bytes;
                assert!(framer.retained_capacity() <= config.payload_peak_bytes().unwrap());
                drop(framer);
                drop(profiler);
                assert!(
                    peak <= config.payload_peak_bytes().unwrap(),
                    "peak {peak} exceeds {:?}",
                    config.payload_peak_bytes()
                );
            }
        }
    }
}
