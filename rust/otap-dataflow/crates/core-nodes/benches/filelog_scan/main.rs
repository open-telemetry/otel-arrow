// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Synchronous scanner measurements, not end-to-end receiver latency.

#[cfg(target_os = "linux")]
fn main() -> std::io::Result<()> {
    use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::{discovery::*, source::*};
    use std::{
        collections::HashMap,
        fs,
        io::Write,
        path::{Path, PathBuf},
        time::Instant,
    };
    struct Policy {
        excluded: PathBuf,
    }
    impl MatchPolicy for Policy {
        fn includes(&self, path: &Path) -> bool {
            path.extension()
                .is_some_and(|e| e == "log" || e == "secret")
        }
        fn may_descend(&self, _: &Path) -> bool {
            true
        }
        fn excludes(&self, path: &Path) -> bool {
            path.starts_with(&self.excluded) && path.extension().is_some_and(|e| e == "secret")
        }
        fn excludes_subtree(&self, _: &Path) -> bool {
            false
        }
        fn excludes_mapped(&self, _: usize, path: &Path) -> bool {
            path.extension().is_some_and(|e| e == "secret")
        }
    }
    let root = std::env::var_os("FILELOG_BENCH_ROOT").unwrap_or_else(|| "/tmp".into());
    let files = std::env::var("FILELOG_BENCH_FILES")
        .ok()
        .map(|v| v.parse::<usize>().expect("integer FILELOG_BENCH_FILES"))
        .unwrap_or(256);
    assert!(
        (128..=100_000).contains(&files),
        "use 128..=100000 regular files"
    );
    let mut output = std::io::stdout().lock();
    writeln!(
        output,
        "layout,mode,regular_files,passes,complete_passes,observed_per_pass,p50_us,p99_us"
    )?;
    for layout in [
        "flat",
        "nested",
        "links",
        "churn_direct",
        "churn_links",
        "continuous_churn",
    ] {
        let dir = tempfile::tempdir_in(&root)?;
        let storage = dir.path().join("logs");
        let links = dir.path().join("containers");
        let namespace = dir.path().join("state");
        fs::create_dir(&storage)?;
        fs::create_dir(&links)?;
        fs::create_dir(&namespace)?;
        let linked = layout == "links" || layout == "churn_links";
        let watched = if linked { &links } else { &storage };
        for index in 0..files {
            let parent = if layout == "flat" {
                storage.clone()
            } else {
                storage.join(format!(
                    "pod-{}/namespace/container/instance/current/logs",
                    index / 16
                ))
            };
            fs::create_dir_all(&parent)?;
            for extension in ["log", "secret"] {
                let path = parent.join(format!("{index}.{extension}"));
                fs::write(&path, [b'x'; 1000])?;
                if linked {
                    std::os::unix::fs::symlink(
                        &path,
                        links.join(format!("{index}.{extension}.log")),
                    )?;
                }
            }
        }
        for mode in ["full", "presence"] {
            let mut scanner = Scanner::new(
                ScanConfig {
                    include_roots: vec![watched.clone()],
                    exclusion_roots: vec![storage.clone()],
                    symlinks: SymlinkPolicy::Follow,
                    limits: ScanLimits::default(),
                    checkpoint_namespace: FileLocator::from_metadata(&fs::metadata(&namespace)?),
                },
                Policy {
                    excluded: storage.clone(),
                },
            )
            .expect("valid benchmark limits");
            // Models the caller's already-retained locator population; built outside timing.
            let mut known = HashMap::new();
            let budget = ProbeBudget::default();
            let warm = scanner.scan(
                0,
                &budget,
                || false,
                |batch| {
                    for o in batch {
                        if o.reason.is_none() {
                            let _ =
                                known.insert(o.path.clone(), o.locator.expect("candidate locator"));
                        }
                    }
                    true
                },
            );
            assert!(warm.complete);
            assert_eq!(known.len(), files);
            let mut samples = Vec::with_capacity(20);
            let mut complete = 0;
            let mut total_observed = 0;
            for pass in 1..=20 {
                let mutate =
                    layout == "continuous_churn" || (layout.starts_with("churn_") && pass % 4 == 0);
                let mut changed = false;
                let mut observed = 0;
                let start = Instant::now();
                let end = scanner.scan_with_requests(
                    pass,
                    &budget,
                    |path| {
                        if mode == "presence"
                            && let Some(&expected) = known.get(path)
                        {
                            ProbeRequest::PresenceOnly { expected }
                        } else {
                            ProbeRequest::Full
                        }
                    },
                    || false,
                    |batch| {
                        observed += batch
                            .iter()
                            .filter(|o| {
                                o.reason.is_none() || o.reason == Some(ScanReason::PresenceOnly)
                            })
                            .count();
                        if mutate && !changed {
                            let path = watched.join("churn-pod");
                            if path.exists() {
                                fs::remove_dir(&path).expect("remove churn fixture");
                            } else {
                                fs::create_dir(&path).expect("create churn fixture");
                            }
                            changed = true;
                        }
                        true
                    },
                );
                samples.push(start.elapsed().as_micros());
                assert_eq!(end.complete, !mutate);
                assert_eq!(observed, files);
                complete += usize::from(end.complete);
                total_observed += observed;
            }
            samples.sort_unstable();
            writeln!(
                output,
                "{layout},{mode},{files},20,{complete},{},{},{}",
                total_observed / 20,
                samples[10],
                samples[19]
            )?;
        }
    }
    Ok(())
}

#[cfg(not(target_os = "linux"))]
fn main() {}
