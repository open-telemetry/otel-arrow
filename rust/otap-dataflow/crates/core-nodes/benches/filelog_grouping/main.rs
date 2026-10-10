// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compare decoder-plus-newline framing with start/end multiline grouping.
#![allow(missing_docs)]

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    decoder::{Encoding, OnDecodeError},
    framer::{LineBody, LineConfig, LineFramer, LineStart, OversizeBehavior, PartialCompletion},
    multiline::{GroupingMode, MultilineConfig, MultilineFramer, RecordStart},
    multiline_pattern::{BoundaryMatcher, BoundaryPattern, PatternMode},
};
use std::{
    hint::black_box,
    io::{self, Write},
    time::Instant,
};

const CHUNK: usize = 128 * 1024;

struct Case {
    name: &'static str,
    encoding: Encoding,
    source: Vec<u8>,
    lines: usize,
    records: usize,
    body_bytes: usize,
}

fn case(
    name: &'static str,
    encoding: Encoding,
    body_len: usize,
    lines: usize,
    group_lines: usize,
) -> Case {
    let mut text = String::new();
    for index in 0..lines {
        let start = if index % group_lines == 0 {
            "START "
        } else {
            "      "
        };
        let end = if index % group_lines == group_lines - 1 {
            "END"
        } else {
            "   "
        };
        text.push_str(start);
        text.push_str(&"x".repeat(body_len - start.len() - end.len()));
        text.push_str(end);
        text.push('\n');
    }
    let source = match encoding {
        Encoding::Utf16Le => text.encode_utf16().flat_map(u16::to_le_bytes).collect(),
        _ => text.into_bytes(),
    };
    Case {
        name,
        encoding,
        source,
        lines,
        records: lines / group_lines,
        body_bytes: lines * body_len,
    }
}

#[derive(Clone, Copy)]
enum Strategy {
    Newline,
    Start,
    End,
}
impl Strategy {
    fn name(self) -> &'static str {
        match self {
            Self::Newline => "newline",
            Self::Start => "start",
            Self::End => "end",
        }
    }
}

#[derive(Debug, Default)]
struct Stats {
    frames: usize,
    body_bytes: usize,
    source_end: u64,
}
impl Stats {
    fn add(&mut self, body: &LineBody, end: u64) {
        self.frames += 1;
        self.body_bytes += match body {
            LineBody::Text(s) => s.len(),
            LineBody::Bytes(b) => b.len(),
        };
        self.source_end = end;
    }
    fn validate(&self, case: &Case, strategy: Strategy) {
        let records = if matches!(strategy, Strategy::Newline) {
            case.lines
        } else {
            case.records
        };
        assert_eq!(self.frames, records);
        assert_eq!(self.body_bytes, case.body_bytes + case.lines - records);
        assert_eq!(self.source_end, case.source.len() as u64);
    }
}

fn scan(
    case: &Case,
    strategy: Strategy,
    pattern: &BoundaryPattern,
    matcher: &mut BoundaryMatcher,
) -> Stats {
    let config = LineConfig {
        encoding: case.encoding,
        on_decode_error: OnDecodeError::PreserveRaw,
        max_line_bytes: 64 * 1024,
        max_record_bytes: 1024 * 1024,
        oversize: OversizeBehavior::Split,
    };
    let mut stats = Stats::default();
    match strategy {
        Strategy::Newline => {
            let mut framer = LineFramer::new(config, LineStart::NewStream).expect("newline config");
            for chunk in case.source.chunks(CHUNK) {
                let mut input = chunk;
                loop {
                    let step = framer
                        .next(framer.next_expected_input_offset(), input)
                        .expect("newline step");
                    input = &input[step.consumed..];
                    if let Some(frame) = step.frame {
                        stats.add(&frame.body, frame.frame_range.end);
                        let _ = black_box(frame);
                    }
                    if !step.advanced {
                        assert!(input.is_empty());
                        break;
                    }
                }
            }
        }
        Strategy::Start | Strategy::End => {
            let mode = if matches!(strategy, Strategy::Start) {
                GroupingMode::Start
            } else {
                GroupingMode::End
            };
            let mut framer = MultilineFramer::new(
                MultilineConfig {
                    line: config,
                    max_lines: 32,
                    mode,
                },
                pattern.clone(),
                RecordStart::NewStream,
            )
            .expect("grouping config");
            for chunk in case.source.chunks(CHUNK) {
                let mut input = chunk;
                loop {
                    let step = framer
                        .next(framer.next_expected_input_offset(), input, matcher)
                        .expect("grouping step");
                    input = &input[step.consumed..];
                    if let Some(frame) = step.frame {
                        stats.add(&frame.body, frame.frame_range.end);
                        let _ = black_box(frame);
                    }
                    if !step.advanced {
                        assert!(input.is_empty());
                        break;
                    }
                }
            }
            loop {
                let step = framer
                    .complete_partial(PartialCompletion::Idle, matcher)
                    .expect("final pending group");
                if let Some(frame) = step.frame {
                    stats.add(&frame.body, frame.frame_range.end);
                    let _ = black_box(frame);
                }
                if step.complete {
                    break;
                }
            }
        }
    }
    stats
}

fn main() -> io::Result<()> {
    let samples: usize =
        std::env::var("FILELOG_GROUPING_SAMPLES").map_or(7, |v| v.parse().expect("sample count"));
    let repeats: usize =
        std::env::var("FILELOG_GROUPING_REPEATS").map_or(8, |v| v.parse().expect("repeat count"));
    assert!(samples > 0 && repeats > 0);
    let mut out = io::stdout().lock();
    writeln!(
        out,
        "case,framer,source_bytes,records_per_scan,samples,median_mib_per_s"
    )?;
    for case in [
        case("short_one_line", Encoding::Utf8, 128, 2048, 1),
        case("short_four_lines", Encoding::Utf8, 128, 2048, 4),
        case("long_four_lines", Encoding::Utf8, 16 * 1024, 32, 4),
        case("utf16_four_lines", Encoding::Utf16Le, 128, 2048, 4),
    ] {
        for strategy in [Strategy::Newline, Strategy::Start, Strategy::End] {
            let pattern = BoundaryPattern::compile(
                if matches!(strategy, Strategy::End) {
                    "END$"
                } else {
                    "^START"
                },
                PatternMode::Text,
            )
            .expect("pattern");
            let mut matcher = pattern.matcher();
            scan(&case, strategy, &pattern, &mut matcher).validate(&case, strategy);
            let mut rates = Vec::with_capacity(samples);
            for _ in 0..samples {
                let begin = Instant::now();
                let mut last = Stats::default();
                for _ in 0..repeats {
                    last = black_box(scan(black_box(&case), strategy, &pattern, &mut matcher));
                }
                let seconds = begin.elapsed().as_secs_f64();
                last.validate(&case, strategy);
                rates.push(case.source.len() as f64 * repeats as f64 / (1024.0 * 1024.0) / seconds);
            }
            rates.sort_by(f64::total_cmp);
            let records = if matches!(strategy, Strategy::Newline) {
                case.lines
            } else {
                case.records
            };
            writeln!(
                out,
                "{},{},{},{},{},{:.2}",
                case.name,
                strategy.name(),
                case.source.len(),
                records,
                samples,
                rates[samples / 2]
            )?;
        }
    }
    Ok(())
}
