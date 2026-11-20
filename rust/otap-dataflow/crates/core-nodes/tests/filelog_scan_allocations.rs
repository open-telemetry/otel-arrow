// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated allocator measurement for scanner storage reuse.
#![cfg(target_os = "linux")]

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{discovery::*, source::*};
use std::{collections::HashMap, ffi::OsString, fs, os::unix::ffi::OsStringExt, path::Path};

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

/// Scenario: Native paths up to 1024 bytes and dot-containing exclusion prefixes alternate full and presence-only probing.
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
    let mut long_parent = dir.path().to_owned();
    for len in [255, 256, 257, 1024] {
        let mut parent = dir.path().join("boundary");
        assert!(
            parent.as_os_str().len() + 8 < len,
            "use a shorter TMPDIR for exact path-length cases"
        );
        while len - parent.as_os_str().len() - 1 > 240 {
            parent.push("d".repeat(120));
        }
        fs::create_dir_all(&parent).unwrap();
        let mut name = vec![b'x'; len - parent.as_os_str().len() - 1];
        name[0] = 0xff;
        let suffix = name.len() - 4;
        name[suffix..].copy_from_slice(b".log");
        let path = parent.join(OsString::from_vec(name));
        assert_eq!(path.as_os_str().len(), len);
        fs::write(&path, b"long native path").unwrap();
        let locator = FileLocator::from_metadata(&fs::metadata(&path).unwrap());
        let alias = path.with_extension("alias");
        std::os::unix::fs::symlink(path.file_name().unwrap(), &alias).unwrap();
        fs::write(path.with_extension("secret"), b"excluded").unwrap();
        let _ = known.insert(path, locator);
        let _ = known.insert(alias, locator);
        long_parent = parent;
    }
    // Exercise long include roots and missing exclusion prefixes too.
    let expected = known.len()
        + known
            .keys()
            .filter(|path| path.starts_with(&long_parent))
            .count();
    let missing_exclusion = long_parent.join("not-created/./child/.");
    let _profiler = dhat::Profiler::builder().testing().build();
    let mut scanner = Scanner::new(
        ScanConfig {
            include_roots: vec![dir.path().to_owned(), long_parent.clone()],
            exclusion_roots: vec![dir.path().to_owned(), missing_exclusion.clone()],
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
    let mut observed = 0;
    let mut consume = |batch: &[Observation]| {
        for observation in batch {
            assert!(matches!(
                observation.reason,
                None | Some(
                    ScanReason::PresenceOnly
                        | ScanReason::Directory
                        | ScanReason::Excluded
                        | ScanReason::UnqualifiedResumption
                )
            ));
            observed += usize::from(matches!(
                observation.reason,
                None | Some(ScanReason::PresenceOnly)
            ));
        }
        true
    };
    assert!(!scanner.scan(0, &budget, || false, &mut consume).cancelled);
    assert_eq!(observed, expected);
    let before = dhat::HeapStats::get();
    dhat::assert!(before.curr_bytes > 0);
    for pass in 1..=6 {
        let mut observed = 0;
        let end = scanner.scan_with_requests(
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
            |batch| {
                for observation in batch {
                    assert!(matches!(
                        observation.reason,
                        None | Some(
                            ScanReason::PresenceOnly
                                | ScanReason::Directory
                                | ScanReason::Excluded
                                | ScanReason::UnqualifiedResumption
                        )
                    ));
                    observed += usize::from(matches!(
                        observation.reason,
                        None | Some(ScanReason::PresenceOnly)
                    ));
                }
                true
            },
        );
        assert!(!end.cancelled);
        assert_eq!(observed, expected);
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
