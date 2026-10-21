// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated retained-memory qualification for compiled Filelog path policy.

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::config::{Config, PathPolicy};
use std::path::PathBuf;

#[path = "support/allocation.rs"]
mod allocation;

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    allocation::run("path_policy_retained_memory", path_policy_retained_memory);
}

/// Scenario: Literal, wildcard, alternative, and mapped-exclusion programs retain cold and grown scratch.
/// Guarantees: Live program/scratch allocations remain within the configuration-owned allowance.
fn path_policy_retained_memory() {
    let base = PathBuf::from(if cfg!(windows) { "C:/work" } else { "/work" });
    let candidate = base.join(format!("logs/{}/tail.log", "ab".repeat(8192)));
    for (count, isolated) in [(1, true), (1, false), (32, false), (128, false)] {
        let config = Config {
            include: if isolated {
                vec!["logs/*.log".into()]
            } else {
                (0..count)
                    .map(|i| format!("logs/{{prefix{i},[ab]*}}/**/*.log"))
                    .collect()
            },
            exclude: if isolated {
                Vec::new()
            } else {
                (0..count)
                    .map(|i| format!("private/{i}/**/*.secret"))
                    .collect()
            },
            ..Config::default()
        };
        let session = format!("path_policy_retained_memory-{count}-{isolated}");
        let profiler = allocation::profiler("filelog_path_memory", &session);
        let compiled = config.compile("g", "p", "n", &base).expect("configuration");
        // Parsed source strings predate the profiler. The additional retained
        // configuration allocations here are the selected ID and hex component.
        let bound = compiled.paths().retained_bytes().expect("memory sum") as usize
            + 2 * (compiled.inputs().checkpoint_id().len()
                + compiled.inputs().namespace_component().len());
        // Test metadata before search growth without the lazy-cache allowance.
        let cold_bound = compiled.paths().program_bytes().expect("program sum") as usize
            - size_of::<PathPolicy>() // The profiler measures heap, not the policy's inline storage.
            + compiled.paths().observed_scratch_bytes().expect("initial scratch") as usize
            + 2 * (compiled.inputs().checkpoint_id().len() + compiled.inputs().namespace_component().len());
        let cold_live = dhat::HeapStats::get().curr_bytes;
        dhat::assert!(
            cold_live <= cold_bound,
            "{count} patterns: cold retained {cold_live} exceeds {cold_bound}"
        );
        for _ in 0..4 {
            let _ = compiled.paths().includes(&candidate);
            let _ = compiled
                .paths()
                .may_descend(candidate.parent().expect("candidate parent"));
            let _ = compiled.paths().excludes(&candidate);
            let _ = compiled.paths().excludes_subtree(&candidate);
            for root in 0..count {
                let _ = compiled
                    .paths()
                    .excludes_mapped(root, std::path::Path::new("sub/one.secret"));
            }
            let live = dhat::HeapStats::get().curr_bytes;
            dhat::assert!(
                live <= bound,
                "{count} patterns: retained {live} exceeds {bound}"
            );
        }
        drop(compiled);
        drop(profiler);
    }
    #[cfg(unix)]
    warmed_unix_matching_reuses_allocations();
}

/// Scenario: A fixed working set exercises all Unix path-policy query methods after cache warmup.
/// Guarantees: Repeated queries reuse owned scratch without transient path-copy or search allocations.
#[cfg(unix)]
fn warmed_unix_matching_reuses_allocations() {
    use std::path::Path;
    let config = Config {
        include: vec![
            "logs/{app,worker}/**/*.log".into(),
            "logs/{a/b,c}/*.txt".into(),
        ],
        exclude: vec!["logs/private/**".into(), "secret/*.log".into()],
        ..Config::default()
    }
    .compile("g", "p", "n", Path::new("/work"))
    .expect("configuration");
    let policy = config.paths();
    let profiler = allocation::profiler("filelog_path_memory", "warmed_queries");
    let query = || {
        assert!(policy.includes(Path::new("/work/logs/app/sub/event.log")));
        assert!(policy.may_descend(Path::new("/work/logs/app/sub")));
        assert!(policy.may_descend(Path::new("/work/logs/a/b")));
        assert!(policy.excludes(Path::new("/work/logs/private/sub/event.log")));
        assert!(policy.excludes_subtree(Path::new("/work/logs/private")));
        assert!(policy.excludes_mapped(1, Path::new("hidden.log")));
    };
    for _ in 0..4 {
        query();
    }
    let before = dhat::HeapStats::get();
    for _ in 0..128 {
        query();
    }
    let after = dhat::HeapStats::get();
    dhat::assert_eq!(after.total_blocks, before.total_blocks);
    dhat::assert_eq!(after.total_bytes, before.total_bytes);
    drop(profiler);
}
