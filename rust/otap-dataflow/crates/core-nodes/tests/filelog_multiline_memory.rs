// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Isolated allocation regression coverage for boundary matcher scratch.

use otel_arrow_dfe_core_nodes::receivers::filelog_receiver::multiline_pattern::{
    BoundaryPattern, ExecutionPolicy, PatternInput, PatternLimits, PatternMode,
};

#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

/// Scenario: Fallback, anchored, unanchored, and cache-churning matchers process repeated nonmatches.
/// Guarantees: Cold allocation and retained cache growth stay within each worker heap bound.
#[test]
fn worker_scratch_stays_within_model() {
    // One test owns DHAT so allocation intervals cannot overlap within this binary.
    let mut state = 1u32;
    let mut churn = Vec::with_capacity(32 * 1024);
    for _ in 0..32 * 1024 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        churn.push(if state & 1 == 0 { b'a' } else { b'b' });
    }
    let suffix = churn.len() - 22;
    churn[suffix] = b'b';
    verify_churn(&churn);
    let body = vec![b'a'; 8192];
    for (source, capacity, caches) in [
        (r"^[ab]*a[ab]{20}[ab]$", 0, 0),
        (r"^[ab]+[cd]$", 64 * 1024, 1),
        (r"[ab]+[cd]", 64 * 1024, 3),
        (r"^[ab]*a[ab]{20}[ab]$", 64 * 1024, 1),
    ] {
        let program = BoundaryPattern::compile_with_limits(
            source,
            PatternMode::Raw,
            PatternLimits {
                lazy_cache_bytes: capacity,
                ..PatternLimits::default()
            },
        )
        .expect("pattern");
        assert_eq!(
            matches!(program.execution_policy(), ExecutionPolicy::LazyDfa { .. }),
            capacity > 0
        );
        assert_eq!(program.memory_estimate().growing_lazy_caches, caches);
        // The fallback fixture fails its required suffix, avoiding early success.
        let mut input = if source.contains("{20}") && capacity > 0 {
            churn.clone()
        } else {
            body.clone()
        };
        if capacity == 0 {
            let suffix = input.len() - 22;
            input[suffix] = b'b';
        }
        let profiler = dhat::Profiler::builder().testing().build();
        let mut matcher = program.matcher();
        for _ in 0..3 {
            assert!(!matcher.is_match(PatternInput::Raw(&input)).expect("search"));
        }
        let peak = dhat::HeapStats::get().max_bytes;
        drop(matcher);
        drop(profiler);
        assert!(
            peak <= program.memory_estimate().worker_heap_bound,
            "{source}: peak {peak} exceeds {:?}",
            program.memory_estimate()
        );
    }
}

// This pattern is already normalized ASCII. Mirror meta's lazy-DFA controls and
// establish churn outside the allocation interval without inspecting private caches.
fn verify_churn(body: &[u8]) {
    use regex_automata::{Input, MatchErrorKind, hybrid::dfa, nfa::thompson, util::syntax};
    // Build the NFA explicitly: DFA::Builder::build would discard captures,
    // whereas production meta retains the implicit whole-match capture.
    let nfa = thompson::NFA::compiler()
        .syntax(syntax::Config::new().unicode(false).utf8(false))
        .configure(
            thompson::Config::new()
                .utf8(false)
                .shrink(false)
                .which_captures(thompson::WhichCaptures::Implicit),
        )
        .build(r"^[ab]*a[ab]{20}[ab]$")
        .expect("production-equivalent NFA");
    assert_eq!(nfa.group_info().slot_len(), 2);
    // These cache controls must match the audited meta engine's defaults.
    let dfa = dfa::Builder::new()
        .configure(
            dfa::Config::new()
                .cache_capacity(64 * 1024)
                .starts_for_each_pattern(true)
                .unicode_word_boundary(true)
                .minimum_cache_clear_count(Some(3))
                .minimum_bytes_per_state(Some(10)),
        )
        .build_from_nfa(nfa)
        .expect("lazy DFA");
    let mut cache = dfa.create_cache();
    let error = dfa
        .try_search_fwd(&mut cache, &Input::new(body).earliest(true))
        .expect_err("fixture must exhaust its lazy cache");
    assert!(
        cache.clear_count() >= 3,
        "fixture must repeatedly clear the cache"
    );
    assert!(matches!(error.kind(), MatchErrorKind::GaveUp { .. }));
}
