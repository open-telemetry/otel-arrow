// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::pattern::{BoundaryPattern, PatternMode};

pub struct Case {
    pub name: &'static str,
    pub pattern: String,
    pub body: Vec<u8>,
    pub mode: PatternMode,
}

pub fn cases(bytes: usize) -> Vec<Case> {
    assert!(bytes >= 128);
    let mut state = 1u32;
    let mut churn = Vec::with_capacity(bytes);
    for _ in 0..bytes {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        churn.push(if state & 1 == 0 { b'a' } else { b'b' });
    }
    churn[bytes - 22] = b'b'; // Required 'a' must occur here for a suffix match.
    let candidate = b"ERROR xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx timeouX ";
    let mut repeated = candidate.repeat(bytes / candidate.len() + 1);
    repeated.truncate(bytes);
    let mut cases = vec![
        Case {
            name: "timestamp_short",
            pattern: r"^\d{4}-\d{2}-\d{2}".into(),
            body: vec![b'x'; 128],
            mode: PatternMode::Text,
        },
        Case {
            name: "literal_absent_max",
            pattern: "ERROR.{0,50}timeout".into(),
            body: vec![b'x'; bytes],
            mode: PatternMode::Text,
        },
        Case {
            name: "literal_candidates_max",
            pattern: "ERROR.{0,50}timeout".into(),
            body: repeated,
            mode: PatternMode::Text,
        },
        Case {
            name: "churn_short",
            pattern: r"^[ab]*a[ab]{20}[ab]$".into(),
            body: churn[..128]
                .iter()
                .copied()
                .enumerate()
                .map(|(i, b)| if i == 106 { b'b' } else { b })
                .collect(),
            mode: PatternMode::Raw,
        },
        Case {
            name: "churn_raw_max",
            pattern: r"^[ab]*a[ab]{20}[ab]$".into(),
            body: churn.clone(),
            mode: PatternMode::Raw,
        },
        Case {
            name: "churn_text_max",
            pattern: r"^[ab]*a[ab]{20}[ab]$".into(),
            body: churn,
            mode: PatternMode::Text,
        },
    ];
    if std::env::var_os("FILELOG_BENCH_LARGE").is_some() {
        let mut repetition = cases
            .iter()
            .find(|c| c.name == "churn_raw_max")
            .expect("fixture")
            .body
            .clone();
        if bytes >= 1001 {
            repetition[bytes - 1001] = b'b';
        }
        cases.push(Case {
            name: "repetition_cap",
            pattern: r"^[ab]*a[ab]{1000}$".into(),
            body: repetition,
            mode: PatternMode::Raw,
        });
        cases.push(Case {
            name: "large_program_search",
            pattern: format!("^[ab]*(?:{}){{1000}}Z$", "a".repeat(200)),
            body: cases
                .iter()
                .find(|c| c.name == "churn_raw_max")
                .expect("fixture")
                .body
                .clone(),
            mode: PatternMode::Raw,
        });
        cases.push(Case {
            name: "branching_nonmatch",
            pattern: r"^a*(?:a|aa){1000}Z$".into(),
            body: vec![b'a'; bytes],
            mode: PatternMode::Raw,
        });
    }
    if let Ok(selected) = std::env::var("FILELOG_BENCH_CASES") {
        let names: Vec<_> = selected.split(',').collect();
        for name in &names {
            assert!(cases.iter().any(|case| case.name == *name), "known case");
        }
        cases.retain(|case| names.contains(&case.name));
    }
    for case in &cases {
        if case.name == "large_program_search" {
            assert!(
                bytes >= 200_001,
                "large-program search must pass the minimum-length shortcut"
            );
        }
        if case.name == "repetition_cap" {
            assert!(
                bytes >= 1001,
                "repetition-cap search needs a full eligible suffix"
            );
        }
    }
    cases
}

pub const VARIANTS: [&str; 8] = [
    "meta",
    "meta_no_prefilter",
    "meta_64k",
    "meta_256k",
    "meta_backtrack",
    "pike",
    "pike_prefilter",
    "pike_backtrack",
];

pub fn compile(case: &Case, variant: &str) -> BoundaryPattern {
    use super::pattern::{MAX_LAZY_CACHE_BYTES, PatternLimits, qualification};
    let (capacity, prefilter, backtrack) = match variant {
        "meta" => (MAX_LAZY_CACHE_BYTES, true, false),
        "meta_no_prefilter" => (MAX_LAZY_CACHE_BYTES, false, false),
        "meta_64k" => (64 * 1024, true, false),
        "meta_256k" => (256 * 1024, true, false),
        "meta_backtrack" => (MAX_LAZY_CACHE_BYTES, true, true),
        "pike" => (0, false, false),
        "pike_prefilter" => (0, true, false),
        "pike_backtrack" => (0, false, true),
        _ => unreachable!("known variant"),
    };
    qualification::compile(
        &case.pattern,
        case.mode,
        PatternLimits {
            lazy_cache_bytes: capacity,
            ..PatternLimits::default()
        },
        prefilter,
        backtrack,
    )
    .expect("benchmark pattern")
}

pub fn input(case: &Case) -> super::pattern::PatternInput<'_> {
    match case.mode {
        PatternMode::Text => super::pattern::PatternInput::Text(
            std::str::from_utf8(&case.body).expect("UTF-8 fixture"),
        ),
        PatternMode::Raw => super::pattern::PatternInput::Raw(&case.body),
    }
}

pub fn variants() -> Vec<&'static str> {
    match std::env::var("FILELOG_BENCH_VARIANTS") {
        Ok(selected) => selected
            .split(',')
            .map(|name| {
                *VARIANTS
                    .iter()
                    .find(|&&v| v == name)
                    .expect("known variant")
            })
            .collect(),
        Err(_) => VARIANTS.to_vec(),
    }
}

pub fn report_cgroup() -> std::io::Result<()> {
    let Ok(destination) = std::env::var("FILELOG_BENCH_CGROUP_REPORT") else {
        return Ok(());
    };
    let membership = std::fs::read_to_string("/proc/self/cgroup")?;
    let group = membership
        .lines()
        .find_map(|line| line.strip_prefix("0::"))
        .expect("cgroup v2");
    let root = std::path::Path::new("/sys/fs/cgroup").join(group.trim_start_matches('/'));
    let mut report = String::new();
    for field in [
        "memory.max",
        "memory.peak",
        "memory.swap.max",
        "cpu.max",
        "cpu.stat",
        "cpuset.cpus.effective",
        "memory.events",
    ] {
        let value = std::fs::read_to_string(root.join(field))?;
        report.push_str(field);
        report.push_str(": ");
        report.push_str(value.trim());
        report.push('\n');
    }
    std::fs::write(destination, report)
}
