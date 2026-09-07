// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::{hint::black_box, time::Duration};

use criterion::{BenchmarkId, Criterion, Throughput};

use super::{
    Encoding, OnDecodeError,
    decoder::{DecodeEvent, DecodedValue, StreamDecoder},
};

#[derive(Debug, Default, Eq, PartialEq)]
struct Scan {
    consumed: u64,
    events: u64,
    malformed: u64,
    checksum: u64,
    failed: bool,
}

impl Scan {
    fn event(&mut self, event: DecodeEvent) {
        self.events += 1;
        match event {
            DecodeEvent::Unit {
                range,
                source,
                value,
                malformed,
            } => {
                self.malformed += u64::from(malformed);
                let scalar = match value {
                    DecodedValue::Scalar(value) => u64::from(u32::from(value)),
                    DecodedValue::RawByte(value) => u64::from(value),
                };
                self.checksum = self.checksum.wrapping_add(
                    range.start
                        ^ range.end
                        ^ scalar
                        ^ u64::from(source.as_slice()[0])
                        ^ source.as_slice().len() as u64,
                );
            }
            DecodeEvent::StrippedBom { range, .. } => {
                self.checksum ^= range.end;
            }
        }
    }
}

fn scan(data: &[u8], encoding: Encoding, policy: OnDecodeError, chunk: usize) -> Scan {
    let mut decoder = StreamDecoder::new(encoding, policy, 0, true);
    let mut result = Scan::default();
    for input in data.chunks(chunk) {
        let mut used = 0;
        loop {
            let step = match decoder.next(decoder.next_expected_input_offset(), &input[used..]) {
                Ok(step) => step,
                Err(error) => {
                    let _ = black_box(error);
                    result.failed = true;
                    result.consumed = decoder.next_expected_input_offset();
                    return result;
                }
            };
            used += step.consumed;
            if let Some(event) = black_box(step.event) {
                result.event(event);
            } else if used == input.len() {
                break;
            } else {
                assert_ne!(step.consumed, 0);
            }
        }
    }
    assert!(decoder.pending_source_start().is_none());
    result.consumed = decoder.next_expected_input_offset();
    result
}

// Experimental caller batching, not a decoder API. Its consumer elects to stop
// only after `capacity` units; a framer needing an earlier stop cannot use it.
fn buffered_scan(data: &[u8], capacity: usize) -> Scan {
    let mut decoder = StreamDecoder::new(Encoding::Utf8, OnDecodeError::Fail, 0, true);
    let mut result = Scan::default();
    let mut output = [None; 64];
    let mut used = 0;
    assert!((1..=output.len()).contains(&capacity));
    loop {
        let mut filled = 0;
        while filled < capacity {
            let step = decoder
                .next(used as u64, &data[used..])
                .expect("clean batch experiment");
            used += step.consumed;
            if let Some(event) = step.event {
                output[filled] = Some(event);
                filled += 1;
            } else {
                break;
            }
        }
        for event in &output[..filled] {
            result.event(black_box(event.expect("filled output slot")));
        }
        if filled < capacity {
            assert_eq!(used, data.len());
            result.consumed = used as u64;
            return result;
        }
    }
}

fn repeated(pattern: &[u8]) -> Vec<u8> {
    pattern.repeat((64 * 1024 / pattern.len()).max(1))
}

fn corpora() -> Vec<(&'static str, Encoding, Vec<u8>)> {
    let ascii = b"2026-09-05T12:34:56Z INFO request completed status=200 bytes=1234\r\n";
    let unicode = "path=/\u{6771}\u{4eac} value=\u{20ac} user=\u{1f600}\n";
    let bmp = "INFO \u{6771}\u{4eac} \u{20ac}\r\n";
    let pairs = "\u{1f600}\u{1f680}\u{10000}\u{10ffff}\n";
    let utf16 = |text: &str, le: bool| {
        text.encode_utf16()
            .flat_map(|unit| {
                if le {
                    unit.to_le_bytes()
                } else {
                    unit.to_be_bytes()
                }
            })
            .collect::<Vec<_>>()
    };
    let mut bom = vec![0xef, 0xbb, 0xbf];
    bom.extend_from_slice(&repeated(unicode.as_bytes()));
    vec![
        ("utf8_ascii", Encoding::Utf8, repeated(ascii)),
        ("ascii", Encoding::Ascii, repeated(ascii)),
        (
            "utf8_multibyte",
            Encoding::Utf8,
            repeated(unicode.as_bytes()),
        ),
        ("utf8_bom", Encoding::Utf8, bom),
        (
            "utf16le_bmp",
            Encoding::Utf16Le,
            repeated(&utf16(bmp, true)),
        ),
        (
            "utf16be_bmp",
            Encoding::Utf16Be,
            repeated(&utf16(bmp, false)),
        ),
        (
            "utf16le_pairs",
            Encoding::Utf16Le,
            repeated(&utf16(pairs, true)),
        ),
        (
            "utf16be_pairs",
            Encoding::Utf16Be,
            repeated(&utf16(pairs, false)),
        ),
        (
            "raw",
            Encoding::Raw,
            repeated(&(0..=255).collect::<Vec<u8>>()),
        ),
    ]
}

pub(super) fn bench_decoder(c: &mut Criterion) {
    let mut group = c.benchmark_group("filelog_decode");
    let _ = group.sample_size(10);
    let _ = group.warm_up_time(Duration::from_millis(100));
    let _ = group.measurement_time(Duration::from_millis(300));
    for (name, encoding, data) in corpora() {
        for chunk in [1, 3, 17, 65536] {
            let _ = group.throughput(Throughput::Bytes(data.len() as u64));
            let _ = group.bench_with_input(BenchmarkId::new(name, chunk), &chunk, |b, chunk| {
                b.iter(|| {
                    black_box(scan(
                        black_box(&data),
                        encoding,
                        OnDecodeError::Fail,
                        *chunk,
                    ))
                });
            });
        }
    }
    for (name, encoding, pattern) in [
        (
            "bad_utf8",
            Encoding::Utf8,
            &b"ok\n\xe2(\xa1\xed\xa0\x80\xf4\x90\x80\x80\n"[..],
        ),
        ("bad_ascii", Encoding::Ascii, &b"ok\n\x80\xff\xfe\xef\n"[..]),
        (
            "bad_utf16le",
            Encoding::Utf16Le,
            &b"o\0k\0\n\0\x00\xd8A\0\x00\xdc\n\0"[..],
        ),
        (
            "bad_utf16be",
            Encoding::Utf16Be,
            &b"\0o\0k\0\n\xd8\x00\0A\xdc\x00\0\n"[..],
        ),
    ] {
        let data = repeated(pattern);
        for policy in [
            OnDecodeError::PreserveRaw,
            OnDecodeError::Replace,
            OnDecodeError::Fail,
        ] {
            for chunk in [1, 3, 17, 65536] {
                let actual = scan(&data, encoding, policy, chunk);
                let _ = group.throughput(Throughput::Bytes(actual.consumed));
                let _ = group.bench_with_input(
                    BenchmarkId::new(format!("{name}_{policy:?}"), chunk),
                    &chunk,
                    |b, chunk| {
                        b.iter(|| black_box(scan(black_box(&data), encoding, policy, *chunk)));
                    },
                );
            }
        }
    }
    let ascii = repeated(b"INFO request completed status=200\r\n");
    let expected = scan(&ascii, Encoding::Utf8, OnDecodeError::Fail, ascii.len());
    for capacity in [0, 1, 4, 16, 64] {
        if capacity != 0 {
            assert_eq!(buffered_scan(&ascii, capacity), expected);
        }
        let _ = group.throughput(Throughput::Bytes(ascii.len() as u64));
        let _ = group.bench_with_input(
            BenchmarkId::new("buffered_ascii_experiment", capacity),
            &capacity,
            |b, capacity| {
                b.iter(|| {
                    black_box(if *capacity == 0 {
                        scan(
                            black_box(&ascii),
                            Encoding::Utf8,
                            OnDecodeError::Fail,
                            ascii.len(),
                        )
                    } else {
                        buffered_scan(black_box(&ascii), *capacity)
                    })
                });
            },
        );
    }
    group.finish();
}
