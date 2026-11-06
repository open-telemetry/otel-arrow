// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated allocation measurement for synchronous reconciliation storage reuse.
#![cfg(target_os = "linux")]

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{
    discovery::{Observation, ScanCompletion, ScanReason, reconcile::*},
    source::FileLocator,
};
use std::time::Duration;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Scenario: Warm tracked aliases alternate policy/full strength, equal-path upgrades and rekeying while pending/overflow metadata grows across 100 scans.
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
    let mut weak_only: Vec<_> = observations
        .iter()
        .map(|o| Observation {
            path: o.path.clone(),
            locator: o.locator,
            size: o.size,
            modified: o.modified,
            fingerprint: o.fingerprint.clone(),
            reason: o.reason,
            errno: o.errno,
        })
        .collect();
    for inode in 1..=64 {
        let weak_path = format!(
            "/logs/{inode:04}.{}",
            if inode % 2 == 0 { "aa" } else { "zz" }
        );
        let weak = || Observation {
            path: weak_path.clone().into(),
            locator: Some(FileLocator { device: 1, inode }),
            size: None,
            modified: None,
            fingerprint: Vec::new(),
            reason: Some(if inode % 2 == 0 {
                ScanReason::Content
            } else {
                ScanReason::PresenceOnly
            }),
            errno: None,
        };
        observations.push(weak());
        weak_only.push(weak());
        observations.push(Observation {
            path: format!("/logs/{inode:04}.zz").into(),
            locator: Some(FileLocator { device: 1, inode }),
            size: Some(64),
            modified: None,
            fingerprint: vec![b'x'; 16],
            reason: None,
            errno: None,
        });
        observations.push(weak()); // Never downgrade the fully validated minimum.
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
    // Warm both reusable buffers across weak/full rekeying and same-path upgrades.
    // Alternating weak-only aliases also exercises cross-pass hint reuse.
    for generation in 0..4 {
        run(
            &mut r,
            generation,
            if generation % 2 == 0 {
                &observations
            } else {
                &weak_only
            },
        );
    }
    let retained = r.retained_bytes();
    let before = dhat::HeapStats::get();
    for generation in 4..104 {
        let input = if generation % 2 == 0 {
            &mut observations
        } else {
            &mut weak_only
        };
        for observation in input.iter_mut().filter(|o| o.reason.is_none()) {
            observation.size = Some(64 + generation);
        }
        run(&mut r, generation, input);
    }
    let after = dhat::HeapStats::get();
    dhat::assert_eq!(before.total_blocks, after.total_blocks);
    dhat::assert_eq!(before.curr_bytes, after.curr_bytes);
    assert_eq!(r.retained_bytes(), retained);
    assert_eq!(r.stats(Duration::from_secs(208)).pending, 64);
}
