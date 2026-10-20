// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated allocator measurement for scanner storage reuse.
#![cfg(target_os = "linux")]

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{discovery::*, source::*};
use std::{collections::HashMap, fs, path::Path};

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
    fn excludes(&self, path: &Path) -> bool {
        path.extension()
            .is_some_and(|extension| extension == "secret")
    }
    fn excludes_subtree(&self, _: &Path) -> bool {
        false
    }
    fn excludes_mapped(&self, _: usize, _: &Path) -> bool {
        false
    }
}

/// Scenario: Flat and six-level scans with exclusions and aliases alternate full and presence-only probing.
/// Guarantees: Retained heap does not grow across passes and transient allocations stay below one per file.
#[test]
fn steady_state_storage_is_reused() {
    let dir = tempfile::tempdir().unwrap();
    let mut known = HashMap::new();
    for index in 0..256 {
        let parent = if index % 2 == 0 {
            dir.path().to_owned()
        } else {
            dir.path().join(format!(
                "pod-{index}/namespace/container/instance/current/logs"
            ))
        };
        fs::create_dir_all(&parent).unwrap();
        let path = parent.join(format!("{index}.log"));
        fs::write(&path, [b'x'; 1000]).unwrap();
        let locator = FileLocator::from_metadata(&fs::metadata(&path).unwrap());
        let _ = known.insert(path, locator);
        fs::write(parent.join(format!("{index}.secret")), b"excluded").unwrap();
        let alias = parent.join(format!("{index}.alias"));
        std::os::unix::fs::symlink(format!("{index}.log"), &alias).unwrap();
        let _ = known.insert(alias, locator);
    }
    let _profiler = dhat::Profiler::builder().testing().build();
    let mut scanner = Scanner::new(
        ScanConfig {
            include_roots: vec![dir.path().to_owned()],
            exclusion_roots: vec![dir.path().to_owned()],
            symlinks: SymlinkPolicy::Follow,
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
    assert!(scanner.scan(0, &budget, || false, |_| true).complete);
    let before = dhat::HeapStats::get();
    dhat::assert!(before.curr_bytes > 0);
    for pass in 1..=6 {
        assert!(
            scanner
                .scan_with_requests(
                    pass,
                    &budget,
                    |path| {
                        if pass > 3
                            && let Some(&expected) = known.get(path)
                        {
                            ProbeRequest::PresenceOnly { expected }
                        } else {
                            ProbeRequest::Full
                        }
                    },
                    || false,
                    |_| true
                )
                .complete
        );
    }
    let after = dhat::HeapStats::get();
    dhat::assert_eq!(after.curr_bytes, before.curr_bytes);
    dhat::assert!(after.total_blocks - before.total_blocks < 32);
    dhat::assert!(after.max_bytes < 1024 * 1024);
    use std::io::Write;
    writeln!(std::io::stdout().lock(),
        "scanner_retained_bytes={}, measured_live_bytes={}, six_pass_allocations={}, six_pass_allocated_bytes={}",
        scanner.retained_bytes(), after.curr_bytes, after.total_blocks - before.total_blocks,
        after.total_bytes - before.total_bytes).expect("write allocation results");
}
