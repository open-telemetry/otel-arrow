// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated worst-case path-storage measurement; no filesystem or background work.
#![cfg(target_os = "linux")]

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    discovery::{Observation, ScanCompletion, reconcile::*},
    source::FileLocator,
};
use std::{path::PathBuf, time::Duration};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn path(inode: u64, prefix: char, bytes: usize) -> PathBuf {
    let mut value = format!("/{prefix}/{inode:05}/");
    while value.len() + 201 < bytes - 2 {
        value.push_str(&"x".repeat(200));
        value.push('/');
    }
    value.push_str(&"x".repeat(bytes - 2 - value.len()));
    value.push_str("/f");
    value.into()
}

/// Scenario: Ten thousand long-path records grow alias buffers; short-path records churn through tracked indexes.
/// Guarantees: Both populations fit retained accounting and the 3L+512 plus fixed and temporary storage admission charge per record.
#[test]
fn worst_case_tracked_path_storage_is_charged() {
    const N: usize = 10_000;
    const L: usize = 4096;
    // All fixture allocations and harness work precede the profiler. Only the
    // synchronous reconciler owns allocations inside this measurement boundary.
    let fixtures: Vec<_> = (1..=N as u64)
        .map(|inode| {
            let make = |prefix, bytes| Observation {
                path: path(inode, prefix, bytes),
                locator: Some(FileLocator { device: 1, inode }),
                size: Some(16),
                modified: None,
                fingerprint: vec![0; 16],
                reason: None,
                errno: None,
            };
            (make('m', L - 1), make('z', L - 2), make('a', L - 1))
        })
        .collect();
    let short_paths: Vec<PathBuf> = (1..=5 * N as u64)
        .map(|id| format!("/d{id:08}/f").into())
        .collect();
    let profiler = dhat::Profiler::builder().testing().build();
    let mut r = Reconciler::new(ReconcileLimits {
        tracked: N,
        pending: 1,
        offers: 1,
        path_bytes: L,
        fingerprint_bytes: 16,
    })
    .unwrap();
    for (binding, _, _) in &fixtures {
        let locator = binding.locator.unwrap();
        r.track(TrackedFile {
            id: TrackedId(locator.inode),
            locator,
            binding: Some(binding.path.clone()),
            role: BindingRole::Live,
            active: true,
            resident: true,
        })
        .unwrap();
    }
    // Each pass grows minimum from L-2 to L-1, exercising geometric capacity;
    // begin swaps the first grown buffer into the retained earlier-pass alias.
    for generation in 1..=2 {
        let now = Duration::from_secs(generation);
        r.begin(generation, now, generation).unwrap();
        for (_, small, large) in &fixtures {
            r.observe(generation, small, now, |_| {}).unwrap();
            r.observe(generation, large, now, |_| {}).unwrap();
        }
        let _ = r
            .finish(
                ScanCompletion {
                    generation,
                    complete: true,
                    cancelled: false,
                    observations: (2 * N) as u64,
                },
                now,
                |_| {},
            )
            .unwrap();
    }
    let heap = dhat::HeapStats::get();
    let retained = r.retained_bytes();
    let charge = 4096 + L + 16 + N * (3 * L + 512) + 2 * (2 * (L + 16) + 512);
    drop(r);
    let released = dhat::HeapStats::get();
    // Short-path limits expose inline/index overhead that long paths can hide.
    // Keep occupancy constant while replacing IDs to exercise deletion tombstones.
    const CHURN: usize = 10_000;
    let mut churn = Reconciler::new(ReconcileLimits {
        tracked: CHURN,
        pending: 1,
        offers: 1,
        path_bytes: 3,
        fingerprint_bytes: 16,
    })
    .unwrap();
    let file = |id| TrackedFile {
        id: TrackedId(id),
        locator: FileLocator {
            device: 1,
            inode: id,
        },
        binding: None,
        role: BindingRole::Live,
        active: true,
        resident: false,
    };
    for id in 1..=CHURN as u64 {
        churn.track(file(id)).unwrap();
    }
    for id in 1..=4 * CHURN as u64 {
        drop(churn.untrack(TrackedId(id)).unwrap());
        churn.track(file(id + CHURN as u64)).unwrap();
    }
    let churn_heap = dhat::HeapStats::get();
    let churn_estimate = churn.retained_bytes();
    let churn_charge = 4096 + 3 + 16 + CHURN * (3 * 3 + 512) + 2 * (2 * (3 + 16) + 512);
    drop(churn);
    let churn_released = dhat::HeapStats::get();
    let mut populated = Reconciler::new(ReconcileLimits {
        tracked: N,
        pending: 1,
        offers: 1,
        path_bytes: 32,
        fingerprint_bytes: 16,
    })
    .unwrap();
    let short_file = |id: u64| TrackedFile {
        id: TrackedId(id),
        locator: FileLocator {
            device: 1,
            inode: id,
        },
        binding: Some(short_paths[id as usize - 1].clone()),
        role: BindingRole::Live,
        active: true,
        resident: false,
    };
    for id in 1..=N as u64 {
        populated.track(short_file(id)).unwrap();
    }
    let populated_before = populated.retained_bytes();
    for id in 1..=4 * N as u64 {
        drop(populated.untrack(TrackedId(id)).unwrap());
        populated.track(short_file(id + N as u64)).unwrap();
    }
    let populated_heap = dhat::HeapStats::get();
    let populated_estimate = populated.retained_bytes();
    let populated_charge = 4096 + 32 + 16 + N * (3 * 32 + 512) + 2 * (2 * (32 + 16) + 512);
    drop(populated);
    let populated_released = dhat::HeapStats::get();
    drop(profiler);
    use std::io::Write;
    writeln!(
        std::io::stdout().lock(),
        "tracked={N} path_limit={L} live={} peak={} estimate={retained} charge={charge}",
        heap.curr_bytes,
        heap.max_bytes
    )
    .expect("write memory measurement");
    writeln!(std::io::stdout().lock(), "churn_records={CHURN} path_limit=3 live={} estimate={churn_estimate} charge={churn_charge}", churn_heap.curr_bytes).expect("write churn measurement");
    writeln!(std::io::stdout().lock(), "populated_records={N} path_limit=32 live={} estimate={populated_estimate} charge={populated_charge}", populated_heap.curr_bytes).expect("write populated measurement");
    assert!(populated_heap.curr_bytes <= populated_estimate);
    assert!(populated_estimate <= populated_charge);
    assert_eq!(populated_before, populated_estimate);
    assert_eq!(populated_released.curr_bytes, 0);
    assert!(churn_heap.curr_bytes <= churn_estimate);
    assert!(
        churn_estimate <= churn_charge,
        "short-path index churn exceeds charge"
    );
    assert_eq!(churn_released.curr_bytes, 0);
    assert!(heap.curr_bytes <= retained, "live heap exceeds accounting");
    assert!(retained <= charge, "accounting exceeds admission charge");
    assert!(
        heap.max_bytes <= charge,
        "buffer replacement peak exceeds admission charge"
    );
    assert_eq!(
        released.curr_bytes, 0,
        "all measured storage is owned by reconciliation"
    );
}
