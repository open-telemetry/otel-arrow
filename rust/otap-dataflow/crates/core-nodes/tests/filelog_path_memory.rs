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
            + 2 * (compiled.checkpoint_id().len() + compiled.namespace_component().len());
        // Test metadata before search growth without the lazy-cache allowance.
        let cold_bound = compiled.paths().program_bytes().expect("program sum") as usize
            - size_of::<PathPolicy>() // The profiler measures heap, not the policy's inline storage.
            + compiled.paths().observed_scratch_bytes().expect("initial scratch") as usize
            + 2 * (compiled.checkpoint_id().len() + compiled.namespace_component().len());
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
}
