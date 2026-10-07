// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared fixtures and streaming consumer for timing and heap measurements.

use std::hint::black_box;

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    decoder::{Encoding, OnDecodeError},
    framer::{LineBody, LineConfig, LineFramer, LineStart, OversizeBehavior},
};

pub const READ_CHUNK: usize = 128 * 1024;

pub struct Case {
    pub name: &'static str,
    pub config: LineConfig,
    pub source: Vec<u8>,
    pub expected_frames: usize,
    pub expected_body_bytes: usize,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Scan {
    pub frames: usize,
    pub body_bytes: usize,
}

pub fn cases() -> Vec<Case> {
    let short = vec![b'x'; 128];
    let multibyte = "\u{e9}\u{4e2d}\u{1f600}".repeat(16).into_bytes();
    let utf16_text = "A\u{3bb}\u{1f600}".repeat(16);
    let utf16: Vec<u8> = utf16_text
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect();
    let mut malformed = short.clone();
    malformed[64] = 0xff;
    let long = vec![b'x'; 16 * 1024];
    vec![
        case(
            "utf8_short_preserve",
            Encoding::Utf8,
            OnDecodeError::PreserveRaw,
            &short,
            2048,
            1024 * 1024,
            OversizeBehavior::Split,
            1,
            128,
        ),
        case(
            "utf8_short_replace",
            Encoding::Utf8,
            OnDecodeError::Replace,
            &short,
            2048,
            1024 * 1024,
            OversizeBehavior::Split,
            1,
            128,
        ),
        case(
            "utf8_multibyte_preserve",
            Encoding::Utf8,
            OnDecodeError::PreserveRaw,
            &multibyte,
            2048,
            1024 * 1024,
            OversizeBehavior::Split,
            1,
            multibyte.len(),
        ),
        case(
            "utf16le_preserve",
            Encoding::Utf16Le,
            OnDecodeError::PreserveRaw,
            &utf16,
            2048,
            1024 * 1024,
            OversizeBehavior::Split,
            1,
            utf16_text.len(),
        ),
        case(
            "utf8_malformed_preserve",
            Encoding::Utf8,
            OnDecodeError::PreserveRaw,
            &malformed,
            2048,
            1024 * 1024,
            OversizeBehavior::Split,
            1,
            128,
        ),
        case(
            "utf8_oversize_split",
            Encoding::Utf8,
            OnDecodeError::PreserveRaw,
            &long,
            16,
            1024,
            OversizeBehavior::Split,
            16,
            long.len(),
        ),
        case(
            "utf8_oversize_truncate",
            Encoding::Utf8,
            OnDecodeError::PreserveRaw,
            &long,
            16,
            1024,
            OversizeBehavior::Truncate,
            1,
            1024,
        ),
    ]
}

fn case(
    name: &'static str,
    encoding: Encoding,
    policy: OnDecodeError,
    body: &[u8],
    lines: usize,
    limit: usize,
    oversize: OversizeBehavior,
    frames_per_line: usize,
    body_bytes_per_line: usize,
) -> Case {
    let delimiter: &[u8] = if encoding == Encoding::Utf16Le {
        &[10, 0]
    } else {
        &[10]
    };
    let mut source = Vec::with_capacity((body.len() + delimiter.len()) * lines);
    for _ in 0..lines {
        source.extend_from_slice(body);
        source.extend_from_slice(delimiter);
    }
    Case {
        name,
        config: LineConfig {
            encoding,
            on_decode_error: policy,
            max_line_bytes: limit,
            max_record_bytes: limit,
            oversize,
        },
        source,
        expected_frames: lines * frames_per_line,
        expected_body_bytes: lines * body_bytes_per_line,
    }
}

// Drop output immediately; advance by consumed bytes, not frame ends, to preserve lookahead.
pub fn scan(case: &Case, chunk_size: usize) -> Scan {
    let mut framer = LineFramer::new(case.config, LineStart::NewStream).expect("valid fixture");
    let mut result = Scan::default();
    for chunk in case.source.chunks(chunk_size) {
        let mut input = chunk;
        loop {
            let step = framer
                .next(framer.next_expected_input_offset(), input)
                .expect("fixture must frame successfully");
            input = &input[step.consumed..];
            if let Some(frame) = step.frame {
                result.frames += 1;
                result.body_bytes += match &frame.body {
                    LineBody::Text(body) => body.len(),
                    LineBody::Bytes(body) => body.len(),
                };
                // Observe without hashing/copying; drop the frame and shadow before the next step.
                let _ = black_box(&frame);
            }
            if !step.advanced {
                assert!(input.is_empty());
                break;
            }
        }
    }
    assert_eq!(
        framer.next_expected_input_offset(),
        case.source.len() as u64
    );
    assert_eq!(framer.pending_source_start(), None);
    result
}

pub fn validate(case: &Case, result: &Scan) {
    assert_eq!(
        result.frames, case.expected_frames,
        "{} frame count",
        case.name
    );
    assert_eq!(
        result.body_bytes, case.expected_body_bytes,
        "{} body size",
        case.name
    );
}
