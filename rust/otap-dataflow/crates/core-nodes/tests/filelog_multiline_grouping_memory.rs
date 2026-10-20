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

#[path = "support/allocation.rs"]
mod allocation;

fn main() {
    allocation::run(
        "grouping_payload_and_discard_are_bounded",
        grouping_payload_and_discard_are_bounded,
    );
}

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn feed(framer: &mut MultilineFramer, matcher: &mut BoundaryMatcher, mut input: &[u8]) -> usize {
    let mut frames = 0;
    loop {
        let step = framer
            .next(framer.next_expected_input_offset(), input, matcher)
            .expect("framing");
        input = &input[step.consumed..];
        frames += usize::from(step.frame.is_some());
        let _ = black_box(step.frame);
        if !step.advanced {
            assert!(input.is_empty());
            return frames;
        }
    }
}

/// Scenario: UTF-8/UTF-16 lines fit or overflow start/end groups under every decode policy, followed by an oversized tail.
/// Guarantees: Measured payload peaks fit the configured estimate and tail scans allocate nothing.
fn grouping_payload_and_discard_are_bounded() {
    for encoding in [Encoding::Utf8, Encoding::Utf16Le] {
        // Build source data outside profiling, including twice-decoded-size UTF-16.
        let encode = |text: &str| -> Vec<u8> {
            if encoding == Encoding::Utf16Le {
                text.encode_utf16().flat_map(u16::to_le_bytes).collect()
            } else {
                text.as_bytes().to_vec()
            }
        };
        let input = encode(&"a".repeat(4096));
        let prefix = encode("prefix\n");
        let lf = encode("\n");
        let extra = encode("a");
        for mode in [GroupingMode::Start, GroupingMode::End] {
            for max_record_bytes in [1024, 32 * 1024] {
                for on_decode_error in [
                    OnDecodeError::PreserveRaw,
                    OnDecodeError::Replace,
                    OnDecodeError::Fail,
                ] {
                    for policy in [OversizeBehavior::Split, OversizeBehavior::Truncate] {
                        let config = MultilineConfig {
                            line: LineConfig {
                                encoding,
                                on_decode_error,
                                max_line_bytes: 4096,
                                max_record_bytes,
                                oversize: policy,
                            },
                            max_lines: 100,
                            mode,
                        };
                        let pattern = BoundaryPattern::compile_with_limits(
                            if mode == GroupingMode::Start {
                                "^prefix$"
                            } else {
                                "^END$"
                            },
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
                        // Name each configuration before measurement to exclude its allocation.
                        let session = format!(
                            "grouping_payload_and_discard_are_bounded-{encoding:?}-{mode:?}-{max_record_bytes}-{on_decode_error:?}-{policy:?}"
                        );
                        let profiler =
                            allocation::profiler("filelog_multiline_grouping_memory", &session);
                        let mut framer =
                            MultilineFramer::new(config, pattern, RecordStart::NewStream)
                                .expect("framer");
                        let _ = feed(&mut framer, &mut matcher, &prefix);
                        if mode == GroupingMode::Start {
                            // Force a real start boundary with an earlier record retained.
                            assert_eq!(feed(&mut framer, &mut matcher, &prefix), 1);
                        }
                        let _ = feed(&mut framer, &mut matcher, &input);
                        if encoding == Encoding::Utf16Le
                            && on_decode_error != OnDecodeError::PreserveRaw
                        {
                            // The full line is still pending: 8192 source bytes and
                            // 4096 decoded bytes must coexist before its LF arrives.
                            assert!(framer.retained_capacity() >= 3 * config.line.max_line_bytes);
                        }
                        let _ = feed(&mut framer, &mut matcher, &lf);
                        let _ = feed(&mut framer, &mut matcher, &input);
                        let _ = feed(&mut framer, &mut matcher, &extra);
                        let before = dhat::HeapStats::get();
                        let retained = framer.retained_capacity();
                        for _ in 0..64 {
                            let _ = feed(&mut framer, &mut matcher, &input);
                        }
                        let after = dhat::HeapStats::get();
                        if policy == OversizeBehavior::Truncate {
                            assert_eq!(before.total_blocks, after.total_blocks);
                            assert_eq!(retained, framer.retained_capacity());
                        }
                        let _ = feed(&mut framer, &mut matcher, &lf);
                        let peak = dhat::HeapStats::get().max_bytes;
                        assert!(
                            framer.retained_capacity()
                                <= config.payload_peak_bytes().expect("valid payload bound")
                        );
                        drop(framer);
                        drop(profiler);
                        assert!(
                            peak <= config.payload_peak_bytes().expect("valid payload bound"),
                            "peak {peak} exceeds {:?}",
                            config.payload_peak_bytes()
                        );
                    }
                }
            }
        }
    }
}
