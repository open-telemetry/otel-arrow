// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! One isolated synchronous operation; all fixtures and assertions are outside measurement.

use std::{hint::black_box, path::Path, rc::Rc, time::Instant};

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    batch::{BatchBuilder, Flow, Frame, Record},
    config::{Config, FileMetadata},
    decoder::SourceRange,
    framer::{LineBody, LineEnding, LineFrame},
    metadata::FileMetadataAttributes,
};
use otel_arrow_dfe_filelog_checkpoint::{AdvisoryPath, CommittedFrontierGuard, FileId};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Scenario: A worker appends repeated bodies from one file, then seals one OTAP batch.
/// Guarantees: Steady append has no per-record staging allocations; bounded Arrow growth and sealing stay within separately measured allocation budgets.
#[test]
fn append_and_seal_have_no_per_record_staging_allocations() {
    let _profiler = dhat::Profiler::builder().testing().build();
    let mut config = Config {
        include: vec![
            if cfg!(windows) {
                "C:\\logs\\*.log"
            } else {
                "/logs/*.log"
            }
            .into(),
        ],
        ..Config::default()
    };
    config.framing.max_line_bytes = 512;
    config.framing.max_record_bytes = 512;
    config.include_file_metadata = FileMetadata::Name;
    let inputs = config
        .compile(
            "g",
            "p",
            "filelog",
            Path::new(if cfg!(windows) { "C:\\" } else { "/" }),
        )
        .unwrap()
        .into_parts()
        .0;
    let mut builder = BatchBuilder::new(&inputs);
    let now = Instant::now();
    let metadata = Rc::new(FileMetadataAttributes::project(
        FileMetadata::Name,
        &AdvisoryPath::from_unix_bytes(b"/logs/app.log").unwrap(),
    ));
    let records: Vec<_> = (0..128)
        .map(|row| {
            let start = row * 257;
            let end = start + 257;
            Record {
                file_id: FileId::from_bytes([1; 16]),
                epoch: 1,
                frame: Frame::Line(LineFrame {
                    body: LineBody::Text("x".repeat(256)),
                    source_body: None,
                    body_range: SourceRange {
                        start,
                        end: end - 1,
                    },
                    frame_range: SourceRange { start, end },
                    ending: LineEnding::Lf,
                    fragment: None,
                    continuation: None,
                    truncated: false,
                    discarded_source_bytes: 0,
                    malformed_units: 0,
                }),
                guard: CommittedFrontierGuard::compute(end, &[0; 64]).unwrap(),
                observed_time_unix_nano: 123,
                metadata: Rc::clone(&metadata),
            }
        })
        .collect();
    let mut records = records.into_iter();
    // Exclude initial Arrow dictionary/buffer creation, not the measured append/seal work.
    for record in records.by_ref().take(64) {
        assert_eq!(builder.offer(record, now).unwrap(), Flow::Continue);
    }
    let before = dhat::HeapStats::get();
    for record in records {
        let _ = black_box(builder.offer(black_box(record), now).unwrap());
    }
    let appended = dhat::HeapStats::get();
    dhat::assert!(appended.total_blocks - before.total_blocks < 32);
    let before_seal = dhat::HeapStats::get();
    builder.seal().unwrap();
    let sealed = dhat::HeapStats::get();
    dhat::assert!(sealed.total_blocks - before_seal.total_blocks <= 256);
    assert_eq!(builder.retained().unwrap().record_count, 128);
    assert_eq!(builder.retained().unwrap().progress.len(), 1);
    let canary = black_box(Box::new(black_box([0; 16])));
    let counted = dhat::HeapStats::get();
    dhat::assert_eq!(counted.total_blocks - sealed.total_blocks, 1);
    drop(canary);
}
