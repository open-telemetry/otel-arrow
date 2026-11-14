// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Synchronous thread-scoped measurements; allocation headers attribute frees to the operation.

use std::{
    alloc::{GlobalAlloc, Layout, System},
    cell::Cell,
    hint::black_box,
    path::Path,
    rc::Rc,
    sync::atomic::{AtomicUsize, Ordering},
    time::Instant,
};

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    batch::{BatchBuilder, Flow, Frame, Record},
    config::{Config, FileMetadata},
    decoder::SourceRange,
    framer::{LineBody, LineEnding, LineFrame},
    metadata::FileMetadataAttributes,
};
use otel_arrow_dfe_filelog_checkpoint::{AdvisoryPath, CommittedFrontierGuard, FileId};

#[global_allocator]
static ALLOC: ScopedAlloc = ScopedAlloc;
static EVENTS: AtomicUsize = AtomicUsize::new(0);
static LIVE_BYTES: AtomicUsize = AtomicUsize::new(0);
thread_local! {
    static MEASURING: Cell<bool> = const { Cell::new(false) };
}

struct ScopedAlloc;

// Only this test's synchronous scopes enable tracking. Headers exclude frees of
// fixtures allocated before the scope and retain attribution after it ends.
#[allow(unsafe_code)]
unsafe impl GlobalAlloc for ScopedAlloc {
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        let Ok((allocation, offset)) = Layout::new::<bool>().extend(layout) else {
            return std::ptr::null_mut();
        };
        let ptr = unsafe { System.alloc(allocation.pad_to_align()) };
        if ptr.is_null() {
            return ptr;
        }
        let tracked = MEASURING.try_with(Cell::get).unwrap_or(false);
        unsafe { ptr.cast::<bool>().write(tracked) };
        if tracked {
            let _ = EVENTS.fetch_add(1, Ordering::Relaxed);
            let _ = LIVE_BYTES.fetch_add(layout.size(), Ordering::Relaxed);
        }
        unsafe { ptr.add(offset) }
    }

    unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
        let (allocation, offset) = Layout::new::<bool>()
            .extend(layout)
            .expect("allocated layout");
        let base = unsafe { ptr.sub(offset) };
        if unsafe { base.cast::<bool>().read() } {
            let _ = LIVE_BYTES.fetch_sub(layout.size(), Ordering::Relaxed);
        }
        unsafe { System.dealloc(base, allocation.pad_to_align()) };
    }
}

struct Measurement;

impl Drop for Measurement {
    fn drop(&mut self) {
        MEASURING.with(|active| active.set(false));
    }
}

fn measure<T>(operation: impl FnOnce() -> T) -> (T, usize) {
    MEASURING.with(|active| {
        assert!(!active.replace(true), "measurement must not nest");
    });
    let scope = Measurement;
    let before = EVENTS.load(Ordering::Relaxed);
    let result = operation();
    let events = EVENTS.load(Ordering::Relaxed) - before;
    drop(scope);
    (result, events)
}

/// Scenario: A worker appends and seals records, then repeatedly seals and releases same-typed batches.
/// Guarantees: Append and seal allocation budgets hold, and omitted-column prefixes do not accumulate retained memory.
#[test]
fn append_and_seal_have_no_per_record_staging_allocations() {
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
    let make_record = |row: u64| {
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
    };
    let records: Vec<_> = (0..128).map(make_record).collect();
    let mut records = records.into_iter();
    // Exclude initial Arrow dictionary/buffer creation, not the measured append/seal work.
    for record in records.by_ref().take(64) {
        assert_eq!(builder.offer(record, now).unwrap(), Flow::Continue);
    }
    let (_, appended) = measure(|| {
        for record in records {
            let _ = black_box(builder.offer(black_box(record), now).unwrap());
        }
    });
    assert!(appended < 32, "{appended} append allocations");
    let (_, sealed) = measure(|| builder.seal().unwrap());
    assert!(sealed <= 256, "{sealed} seal allocations");
    assert_eq!(builder.retained().unwrap().record_count, 128);
    assert_eq!(builder.retained().unwrap().progress.len(), 1);
    let (canary, counted) = measure(|| black_box(Box::new(black_box([0; 16]))));
    assert_eq!(counted, 1);
    drop(canary);
    drop(builder);
    assert_eq!(LIVE_BYTES.load(Ordering::Relaxed), 0);

    let (mut builder, _) = measure(|| BatchBuilder::new(&inputs));
    let mut baseline = 0;
    let mut maximum = 0;
    for batch in 0..128 {
        let _ = measure(|| {
            for row in 0..128 {
                let _ = black_box(builder.offer(make_record(row), now).unwrap());
            }
            builder.seal().unwrap();
            let applied = builder.retained().unwrap().progress.clone();
            builder.release_applied(&applied, now).unwrap();
        });
        let live = LIVE_BYTES.load(Ordering::Relaxed);
        if batch == 0 {
            baseline = live;
        }
        maximum = maximum.max(live);
    }
    assert!(
        maximum <= baseline + 1024,
        "released batch memory grew from {baseline} to {maximum} bytes"
    );
    drop(builder);
    assert_eq!(LIVE_BYTES.load(Ordering::Relaxed), 0);
}
