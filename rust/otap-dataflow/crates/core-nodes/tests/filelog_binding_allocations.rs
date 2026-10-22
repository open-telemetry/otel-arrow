// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated warm targeted-binding allocation measurement.
#![cfg(target_os = "linux")]

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::discovery::reconcile::*;
use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{discovery::*, source::*};
use std::{fs, path::Path, time::Duration};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

struct Policy;
impl MatchPolicy for Policy {
    fn includes(&self, _: &Path) -> bool {
        true
    }
    fn may_descend(&self, _: &Path) -> bool {
        true
    }
    fn excludes(&self, _: &Path) -> bool {
        false
    }
    fn excludes_subtree(&self, _: &Path) -> bool {
        false
    }
    fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
        false
    }
}

/// Scenario: Repeated excluded observations drive sixteen warm bindings through reconciliation and batched scanner checks.
/// Guarantees: Worklist driving and result application allocate nothing after warm-up and release all probe descriptors.
#[test]
fn warm_binding_batches_reuse_storage() {
    let dir = tempfile::tempdir().unwrap();
    let parent = FileLocator::from_metadata(&fs::metadata(dir.path()).unwrap());
    let paths: Vec<_> = (0..16)
        .map(|id| {
            let path = dir.path().join(format!("{id:04}.log"));
            fs::write(&path, [b'x'; 32]).unwrap();
            path
        })
        .collect();
    let mut scanner = Scanner::new(
        ScanConfig {
            include_roots: vec![dir.path().to_owned()],
            exclusion_roots: vec![],
            symlinks: SymlinkPolicy::Reject,
            limits: ScanLimits::default(),
            checkpoint_namespace: FileLocator {
                device: u64::MAX,
                inode: u64::MAX,
            },
        },
        Policy,
    )
    .unwrap();
    let budget = ProbeBudget::default();
    let mut r =
        Reconciler::new(ReconcileLimits::from_scan(&ScanLimits::default(), 16, 1, 1)).unwrap();
    let excluded: Vec<_> = paths
        .iter()
        .enumerate()
        .map(|(id, path)| {
            let locator = FileLocator::from_metadata(&fs::metadata(path).unwrap());
            r.track(TrackedFile {
                id: TrackedId(id as u64),
                locator,
                binding: Some(path.clone()),
                role: BindingRole::Live,
                active: true,
                resident: true,
            })
            .unwrap();
            Observation {
                path: path.with_extension("excluded"),
                locator: Some(locator),
                size: None,
                modified: None,
                fingerprint: Vec::new(),
                reason: Some(ScanReason::Excluded),
                errno: None,
            }
        })
        .collect();
    let directory = Observation {
        path: dir.path().to_owned(),
        locator: Some(parent),
        size: None,
        modified: None,
        fingerprint: Vec::new(),
        reason: Some(ScanReason::Directory),
        errno: None,
    };
    let run = |scanner: &mut Scanner<Policy>, r: &mut Reconciler, generation| {
        let now = Duration::from_secs(generation);
        r.begin(generation, now, generation).unwrap();
        r.observe(generation, &directory, now, |_| {}).unwrap();
        for observation in &excluded {
            r.observe(generation, observation, now, |_| {}).unwrap();
        }
        let mut batch = scanner.binding_batch(&budget, || false);
        let mut valid = 0;
        while let Some(id) = r.next_binding_check() {
            let request = r.binding_request(id).unwrap();
            let result = batch.check(request.path, request.parent).unwrap();
            valid += usize::from(result.observation.reason.is_none() && result.context_valid);
            r.apply_binding_check(generation, id, result, now, |_| {})
                .unwrap();
        }
        let _ = r
            .finish(
                ScanCompletion {
                    generation,
                    complete: true,
                    cancelled: false,
                    observations: excluded.len() as u64 + 1,
                },
                now,
                |_| {},
            )
            .unwrap();
        valid
    };
    assert_eq!(run(&mut scanner, &mut r, 0), paths.len());
    // Profiler and fixtures are outside the synchronous measured interval. This
    // binary contains one test and creates no background tasks or logging work.
    let profiler = dhat::Profiler::builder().testing().build();
    let before = dhat::HeapStats::get();
    let mut valid = 0;
    for generation in 1..=10 {
        valid += run(&mut scanner, &mut r, generation);
    }
    let after = dhat::HeapStats::get();
    drop(profiler);
    assert_eq!(valid, paths.len() * 10);
    assert_eq!(after.total_blocks, before.total_blocks);
    assert_eq!(after.curr_bytes, before.curr_bytes);
    assert!(budget.try_reserve().is_some());
}
