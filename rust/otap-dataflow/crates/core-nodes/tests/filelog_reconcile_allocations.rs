// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated allocation measurement for synchronous reconciliation storage reuse.
#![cfg(target_os = "linux")]

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    discovery::{Observation, ScanCompletion, reconcile::*},
    source::FileLocator,
};
use std::time::Duration;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Scenario: Warm tracked bindings/aliases, pending candidates and overflow samples receive appended metadata across 100 scans.
/// Guarantees: The measured synchronous passes allocate no new blocks and retain constant owned capacity.
#[test]
fn warm_reconciliation_reuses_storage() {
    // One test in this process. Fixture construction, profiler setup, output and
    // assertions are outside the measured interval; no background threads run.
    let mut observations: Vec<_> = (1..=256)
        .map(|inode| Observation {
            path: format!("/logs/{inode:04}.log").into(),
            locator: Some(FileLocator { device: 1, inode }),
            size: Some(64),
            modified: None,
            fingerprint: vec![b'x'; 16],
            reason: None,
            errno: None,
        })
        .collect();
    for inode in 1..=64 {
        observations.push(Observation {
            path: format!("/logs/{inode:04}.alias").into(),
            locator: Some(FileLocator { device: 1, inode }),
            size: Some(64),
            modified: None,
            fingerprint: vec![b'x'; 16],
            reason: None,
            errno: None,
        });
    }
    let mut r = Reconciler::new(ReconcileLimits {
        tracked: 64,
        pending: 64,
        offers: 16,
        path_bytes: 4096,
        fingerprint_bytes: 16,
    })
    .unwrap();
    for observation in &observations[..64] {
        let locator = observation.locator.unwrap();
        r.track(TrackedFile {
            id: TrackedId(locator.inode),
            locator,
            binding: Some(observation.path.clone()),
            role: BindingRole::Live,
            active: true,
            resident: true,
        })
        .unwrap();
    }
    let run = |r: &mut Reconciler, generation, observations: &[Observation]| {
        let start = Duration::from_secs(generation * 2);
        r.begin(generation, start, generation).unwrap();
        for observation in observations {
            r.observe(generation, observation, start, |_| {}).unwrap();
        }
        let _ = r
            .finish(
                ScanCompletion {
                    generation,
                    complete: true,
                    cancelled: false,
                    observations: observations.len() as u64,
                },
                start + Duration::from_secs(1),
                |_| {},
            )
            .unwrap();
        loop {
            let token = r.offers().unwrap().next().map(|offer| offer.token);
            let Some(token) = token else {
                break;
            };
            r.acknowledge(
                token,
                IdentityFeedback::Deferred,
                start + Duration::from_secs(1),
            )
            .unwrap();
        }
    };
    let _profiler = dhat::Profiler::builder().testing().build();
    // Two warm passes initialize both reusable alias buffers. Construction is
    // measured separately; this assertion covers steady-state index/buffer reuse.
    run(&mut r, 0, &observations);
    run(&mut r, 1, &observations);
    let retained = r.retained_bytes();
    let before = dhat::HeapStats::get();
    for generation in 2..=101 {
        for observation in &mut observations {
            observation.size = Some(64 + generation);
        }
        run(&mut r, generation, &observations);
    }
    let after = dhat::HeapStats::get();
    dhat::assert_eq!(before.total_blocks, after.total_blocks);
    dhat::assert_eq!(before.curr_bytes, after.curr_bytes);
    assert_eq!(r.retained_bytes(), retained);
    assert_eq!(r.stats(Duration::from_secs(202)).pending, 64);
}
