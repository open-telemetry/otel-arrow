// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::*;

fn matcher(program: &BoundaryPattern) -> BoundaryMatcher {
    program.matcher()
}

fn matches(pattern: &str, text: &str) -> bool {
    let program = BoundaryPattern::compile(pattern, PatternMode::Text).expect("pattern");
    matcher(&program)
        .is_match(PatternInput::Text(text))
        .expect("search")
}

/// Scenario: Start/end patterns search decoded physical-line bodies.
/// Guarantees: Anchors, empty bodies, CR, NUL, and non-ASCII literals retain their meaning.
#[test]
fn physical_line_patterns() {
    assert!(matches(r"^20\d\d-", "2026-10-09 message"));
    assert!(!matches(r"^20\d\d-", " 2026-10-09 message"));
    assert!(matches(r"END$", "value END"));
    assert!(!matches(r"END$", "value END\r"));
    assert!(matches(r"^$", ""));
    assert!(matches(r"a\x00b", "a\0b"));
    assert!(matches("^\u{e9}$", "\u{e9}"));
    assert!(matches(r"^.$", "\u{1f600}"));
}

/// Scenario: Perl classes encounter ASCII, Unicode, and vertical-tab input.
/// Guarantees: Classes and their negations have RE2 ASCII semantics inside and outside brackets.
#[test]
fn ascii_perl_classes() {
    for (class, yes, no) in [
        (r"\d", "5", "\u{0665}"),
        (r"\D", "\u{0665}", "5"),
        (r"\w", "_", "\u{e9}"),
        (r"\W", "\u{e9}", "_"),
        (r"\s", "\t", "\u{00a0}"),
        (r"\S", "\u{00a0}", "\t"),
    ] {
        for pattern in [format!("^{class}$"), format!("^[{class}]$")] {
            assert!(matches(&pattern, yes), "{pattern}");
            assert!(!matches(&pattern, no), "{pattern}");
        }
    }
    assert!(!matches(r"\s", "\x0b"));
    assert!(matches(r"\S", "\x0b"));
    assert!(matches(r"[[:space:]]", "\x0b"));
    assert!(matches(r"^[^\D]$", "3"));
    assert!(!matches(r"^[^\D]$", "\u{0663}"));
}

/// Scenario: Word boundaries appear beside non-ASCII text and punctuation.
/// Guarantees: Only ASCII word characters affect boundary assertions.
#[test]
fn ascii_word_boundaries() {
    assert!(matches(r"\bword\b", "\u{e9}word\u{e9}"));
    assert!(!matches(r"\bword\b", "sword"));
    assert!(matches(r"\B", "\u{e9}"));
    assert!(!matches(r"\b", "\u{e9}"));
}

/// Scenario: Accepted inline flags are global, scoped, or disabled in nested groups.
/// Guarantees: i/m/s/U work without accepting Rust-only flags.
#[test]
fn supported_flags() {
    assert!(matches("(?i)^start$", "START"));
    assert!(!matches("(?i:start)(?-i:END)", "STARTend"));
    assert!(matches("(?m)^end$", "end"));
    assert!(!matches("(?m)^end$", "end\r"));
    assert!(matches("(?s)^a.b$", "a\0b"));
    assert!(matches("(?U)^a.*b$", "a123b"));
    assert!(matches("(?im-sU:start)", "START"));
    // RE2 applies case folding before complementing a class.
    assert!(matches(r"(?i)^\w$", "\u{212a}"));
    assert!(!matches(r"(?i)^\W$", "\u{212a}"));
}

/// Scenario: Raw patterns inspect arbitrary bytes including invalid UTF-8.
/// Guarantees: Exact byte escapes, byte-sized dot, ASCII classes, and boundaries work.
#[test]
fn raw_patterns() {
    for (pattern, bytes, expected) in [
        (r"^\xFF$", &b"\xff"[..], true),
        (r"^.$", &b"\xff"[..], true),
        (r"^.$", "\u{e9}".as_bytes(), false),
        (r"\bA\b", &b"\xffA\xff"[..], true),
        (r"^\D$", &b"\xff"[..], true),
        (r"^\s$", &b"\x0b"[..], false),
    ] {
        let compiled = BoundaryPattern::compile(pattern, PatternMode::Raw)
            .expect("valid pattern test operation");
        assert_eq!(
            matcher(&compiled)
                .is_match(PatternInput::Raw(bytes))
                .expect("valid pattern test operation"),
            expected
        );
    }
}

/// Scenario: A source configuration or caller supplies syntax/representation outside the profile.
/// Guarantees: Unsupported constructs and text/raw mismatches fail explicitly.
#[test]
fn rejects_unsupported_constructs() {
    for pattern in [
        r"(a)\1",
        r"(?=a)",
        r"(?<=a)",
        r"(?!a)",
        r"(?<!a)",
        r"(?u)a",
        r"(?-u)a",
        r"(?R:a)",
        r"(?x)a",
        r"(?i-u:a)",
        r"\p{Greek}",
        r"\P{L}",
        r"[\pL]",
        r"[a-z&&b]",
        r"[a-z--b]",
        r"[a~~b]",
        r"[a[b]]",
        r"\b{start}",
        r"\<",
        r"\u0061",
        r"[\U00000061]",
        r"a**",
        r"a++",
        r"a{1}{2}",
        r"a{1001}",
        r"a{1001,}",
        r"a{1,1001}",
        r"(",
    ] {
        for mode in [PatternMode::Text, PatternMode::Raw] {
            assert!(
                BoundaryPattern::compile(pattern, mode).is_err(),
                "{pattern}"
            );
        }
    }
    let text =
        BoundaryPattern::compile(".", PatternMode::Text).expect("valid pattern test operation");
    let raw =
        BoundaryPattern::compile(".", PatternMode::Raw).expect("valid pattern test operation");
    assert!(matches!(
        matcher(&text).is_match(PatternInput::Raw(b"a")),
        Err(PatternError::InputMode)
    ));
    assert!(matches!(
        matcher(&raw).is_match(PatternInput::Text("a")),
        Err(PatternError::InputMode)
    ));
}

/// Scenario: Escapes resemble forbidden syntax but denote ordinary literal characters.
/// Guarantees: AST validation does not reject literal backslashes, brackets, or flag-like text.
#[test]
fn escaped_literals_are_not_syntax() {
    assert!(matches(r"^\\p\{L\}$", r"\p{L}"));
    assert!(matches(r"^\(\?u\)$", "(?u)"));
    assert!(matches(r"^[\[\]]$", "["));
    assert!(matches(r"^[\[\]]$", "]"));
}

/// Scenario: Patterns reach source, repetition, nesting, and compiled-program limits.
/// Guarantees: Limits are enforced during compilation, before any source record is read.
#[test]
fn compilation_bounds() {
    let at_limit = "a|".repeat(MAX_PATTERN_BYTES / 2);
    assert!(BoundaryPattern::compile(&at_limit, PatternMode::Raw).is_ok());
    assert!(matches!(
        BoundaryPattern::compile(&(at_limit + "a"), PatternMode::Raw),
        Err(PatternError::TooLong)
    ));
    assert!(BoundaryPattern::compile("a{1000}", PatternMode::Raw).is_ok());
    let nested = format!("{}a{}", "(".repeat(100), ")".repeat(100));
    assert!(BoundaryPattern::compile(&nested, PatternMode::Text).is_err());
    assert!(matches!(
        BoundaryPattern::compile("(?:a{1000}){1000}", PatternMode::Raw),
        Err(PatternError::Unsupported(_))
    ));
}

/// Scenario: Supported patterns would exceed the former eager DFA limit.
/// Guarantees: Text and raw matchers admit these patterns and produce correct results.
#[test]
fn lazy_dfa_admits_supported_patterns() {
    for (source, yes, no) in [
        (
            "ERROR.{0,50}timeout",
            "ERROR waiting for timeout",
            "ERROR waiting for success",
        ),
        (
            "[a-z]+ .{0,200} foo",
            "prefix payload foo",
            "prefix payload bar",
        ),
        ("[ab]*a[ab]{14}", "abbbbbbbbbbbbbb", "bbbbbbbbbbbbbbb"),
        (
            "[ab]*a[ab]{24}",
            "abbbbbbbbbbbbbbbbbbbbbbbb",
            "bbbbbbbbbbbbbbbbbbbbbbbbb",
        ),
    ] {
        for mode in [PatternMode::Text, PatternMode::Raw] {
            let program = BoundaryPattern::compile(source, mode).expect("supported pattern");
            let mut matcher = matcher(&program);
            for (body, expected) in [(yes, true), (no, false)] {
                let input = match mode {
                    PatternMode::Text => PatternInput::Text(body),
                    PatternMode::Raw => PatternInput::Raw(body.as_bytes()),
                };
                assert_eq!(
                    matcher.is_match(input).expect("successful search"),
                    expected,
                    "{source}"
                );
            }
            assert!(program.program_memory_usage() > 0);
            assert!(matcher.cache_memory_usage() <= program.memory_estimate().worker_heap_bound);
        }
    }
}

/// Scenario: A raw pattern contains a non-ASCII literal or an exact byte escape.
/// Guarantees: Literals match UTF-8 source bytes; hex escapes can match a single non-UTF-8 byte.
#[test]
fn raw_literals_use_utf8_bytes() {
    for (source, utf8, latin1) in [("^\u{e9}$", true, false), (r"^\xE9$", false, true)] {
        let program = BoundaryPattern::compile(source, PatternMode::Raw).expect("raw pattern");
        let mut matcher = matcher(&program);
        assert_eq!(
            matcher
                .is_match(PatternInput::Raw(b"\xc3\xa9"))
                .expect("search"),
            utf8
        );
        assert_eq!(
            matcher
                .is_match(PatternInput::Raw(b"\xe9"))
                .expect("search"),
            latin1
        );
    }
}

/// Scenario: Translation fails after Perl classes and word boundaries have been normalized.
/// Guarantees: Diagnostics contain an error reason without displaying rewritten pattern text.
#[test]
fn translation_errors_do_not_expose_generated_syntax() {
    let error = BoundaryPattern::compile("\\d\\b[\u{e9}]", PatternMode::Raw)
        .expect_err("non-ASCII literal is unsupported in a raw character class");
    let PatternError::Syntax(reason) = error else {
        panic!("expected syntax error")
    };
    assert!(!reason.is_empty());
    assert!(!reason.contains("[0-9]"));
    assert!(!reason.contains("(?-u:"));
    assert!(!reason.contains("regex parse error"));
}

/// Scenario: Nested counted repetitions reach or exceed RE2's cumulative limit.
/// Guarantees: A product of 1000 fits, and a product of 1010 is rejected.
#[test]
fn nested_repetition_limit() {
    assert!(BoundaryPattern::compile("(a{10}){100}", PatternMode::Text).is_ok());
    assert!(matches!(
        BoundaryPattern::compile("(a{10}){101}", PatternMode::Text),
        Err(PatternError::Unsupported(_))
    ));
}

fn qualified(source: &str, mode: PatternMode, capacity: usize) -> BoundaryPattern {
    qualification::compile(
        source,
        mode,
        PatternLimits {
            lazy_cache_bytes: capacity,
            ..PatternLimits::default()
        },
        false,
        false,
    )
    .expect("qualified pattern")
}

/// Scenario: Fixed and braced raw escapes denote byte values, including inside classes and ranges.
/// Guarantees: Both execution paths preserve bytes rather than encoding escaped values as UTF-8.
#[test]
fn raw_braced_hex_is_a_byte() {
    for value in [0u8, b'A', 0x7f, 0x80, 0xff] {
        for source in [
            format!(r"^\x{value:02X}$"),
            format!(r"^\x{{{value:X}}}$"),
            format!(r"^[\x{{{value:X}}}]$"),
        ] {
            for capacity in [0, MAX_LAZY_CACHE_BYTES] {
                let program = qualified(&source, PatternMode::Raw, capacity);
                let mut search = program.matcher();
                assert!(
                    search
                        .is_match(PatternInput::Raw(&[value]))
                        .expect("byte match")
                );
                if value > 127 {
                    let mut utf8 = [0; 4];
                    let text = char::from(value).encode_utf8(&mut utf8);
                    assert!(
                        !search
                            .is_match(PatternInput::Raw(text.as_bytes()))
                            .expect("not UTF-8")
                    );
                }
            }
        }
    }
    for capacity in [0, MAX_LAZY_CACHE_BYTES] {
        let program = qualified(r"^[\x{80}-\x{FF}]$", PatternMode::Raw, capacity);
        assert!(
            program
                .matcher()
                .is_match(PatternInput::Raw(b"\xfe"))
                .expect("range")
        );
        assert!(
            !program
                .matcher()
                .is_match(PatternInput::Raw(b"\x7f"))
                .expect("outside range")
        );
    }
    for source in [r"\x{100}", r"[\x{100}]", r"[\x{FF}-\x{100}]"] {
        assert!(matches!(
            BoundaryPattern::compile(source, PatternMode::Raw),
            Err(PatternError::Unsupported(_))
        ));
    }
    assert!(BoundaryPattern::compile("[\u{e9}]", PatternMode::Raw).is_err());
    // Text mode keeps Unicode semantics; ordinary raw literals still use UTF-8.
    assert!(matches(r"^\x{FF}$", "\u{ff}"));
    assert!(matches(r"^[\x{100}]$", "\u{100}"));
}

/// Scenario: Normalized patterns are evaluated with lazy acceleration and forced NFA execution.
/// Guarantees: Anchors, empty matches, ASCII classes/boundaries, flags, and raw bytes agree.
#[test]
fn engine_paths_preserve_semantics() {
    for (source, body, expected) in [
        (r"^(a)$", "a", true),
        (r"^$", "", true),
        (r"^$", "x", false),
        (r"\bword\b", "\u{e9}word\u{e9}", true),
        (r"\bword\b", "sword", false),
        (r"\B", "\u{e9}", true),
        (r"(?i)^\w$", "K", true),
        (r"^\s$", "\x0b", false),
        (r"^END$", "END\r", false),
        (r"(?m)^END$", "END", true),
        (r"(?s)^a.b$", "a\0b", true),
        (r"(?U)^a.*b$", "a123b", true),
    ] {
        for mode in [PatternMode::Text, PatternMode::Raw] {
            for capacity in [0, MAX_LAZY_CACHE_BYTES] {
                let program = qualified(source, mode, capacity);
                let input = match mode {
                    PatternMode::Text => PatternInput::Text(body),
                    PatternMode::Raw => PatternInput::Raw(body.as_bytes()),
                };
                assert_eq!(
                    program.matcher().is_match(input).expect("search"),
                    expected,
                    "{source}"
                );
                if capacity == 0 {
                    assert_eq!(
                        program.execution_policy(),
                        ExecutionPolicy::NoLazyDfa {
                            requested_cache_bytes: 0,
                            reason: AccelerationDisabled::Requested
                        }
                    );
                }
            }
        }
    }
}

/// Scenario: A directly configured lazy DFA gives up on the same normalized pattern/input used by meta.
/// Guarantees: Meta's equivalent fallback completes and preserves both nonmatch and match results.
#[test]
fn cache_churn_uses_equivalent_fallback() {
    use regex_automata::{MatchErrorKind, hybrid::dfa};
    let source = r"^[ab]*a[ab]{20}[ab]$";
    let program = qualified(source, PatternMode::Raw, MAX_LAZY_CACHE_BYTES);
    let hir = parse(source, PatternMode::Raw).expect("HIR");
    let nfa = forward_nfa(&hir, PatternMode::Raw, MAX_PROGRAM_BYTES).expect("NFA");
    // Public API controls mirror pinned meta/wrappers.rs. Start anchoring and
    // disabled prefilters select Core::search_half, so a gave-up lazy search
    // can only complete through the enabled PikeVM fallback.
    let dfa = dfa::Builder::new()
        .configure(
            dfa::Config::new()
                .cache_capacity(MAX_LAZY_CACHE_BYTES)
                .starts_for_each_pattern(true)
                .unicode_word_boundary(true)
                .minimum_cache_clear_count(Some(3))
                .minimum_bytes_per_state(Some(10)),
        )
        .build_from_nfa(nfa)
        .expect("lazy engine");
    let mut state = 1u32;
    let mut body = Vec::with_capacity(200_000);
    for _ in 0..200_000 {
        state ^= state << 13;
        state ^= state >> 17;
        state ^= state << 5;
        body.push(if state & 1 == 0 { b'a' } else { b'b' });
    }
    let at = body.len() - 22;
    body[at] = b'b';
    let error = dfa
        .try_search_fwd(&mut dfa.create_cache(), &Input::new(&body).earliest(true))
        .expect_err("fixture must force give-up");
    assert!(matches!(error.kind(), MatchErrorKind::GaveUp { .. }));
    let mut search = program.matcher();
    assert!(
        !search
            .is_match(PatternInput::Raw(&body))
            .expect("meta fallback")
    );
    body[at] = b'a';
    assert!(
        search
            .is_match(PatternInput::Raw(&body))
            .expect("matching suffix")
    );
    assert!(search.cache_memory_usage() <= program.memory_estimate().worker_heap_bound);
}

/// Scenario: The caller drops all external program handles while storing a matcher in its worker.
/// Guarantees: Worker-owned matching remains valid without self-references or borrowed configuration.
#[test]
fn worker_owns_its_program() {
    struct Worker {
        matcher: BoundaryMatcher,
    }
    let program = BoundaryPattern::compile("^ERROR$", PatternMode::Text).expect("program");
    let mut worker = Worker {
        matcher: program.matcher(),
    };
    drop(program);
    assert!(
        worker
            .matcher
            .is_match(PatternInput::Text("ERROR"))
            .expect("owned program")
    );
}

/// Scenario: Limits explicitly disable acceleration, cannot accommodate its minimum cache, or exceed ceilings.
/// Guarantees: Policy decisions are observable and oversized requests fail before activation.
#[test]
fn execution_policy_and_limits() {
    let source = r"^[ab]*a[ab]{20}[ab]$";
    let requested = qualified(source, PatternMode::Raw, 0);
    assert_eq!(
        requested.execution_policy(),
        ExecutionPolicy::NoLazyDfa {
            requested_cache_bytes: 0,
            reason: AccelerationDisabled::Requested
        }
    );
    let unavailable = qualified(source, PatternMode::Raw, 1);
    assert_eq!(
        unavailable.execution_policy(),
        ExecutionPolicy::NoLazyDfa {
            requested_cache_bytes: 1,
            reason: AccelerationDisabled::CacheCapacity
        }
    );
    assert_eq!(unavailable.memory_estimate().growing_lazy_caches, 0);
    for limits in [
        PatternLimits {
            program_bytes: MAX_PROGRAM_BYTES + 1,
            ..PatternLimits::default()
        },
        PatternLimits {
            lazy_cache_bytes: MAX_LAZY_CACHE_BYTES + 1,
            ..PatternLimits::default()
        },
    ] {
        assert!(matches!(
            BoundaryPattern::compile_with_limits(source, PatternMode::Raw, limits),
            Err(PatternError::Limit { .. })
        ));
    }
    assert!(
        BoundaryPattern::compile_with_limits(
            source,
            PatternMode::Raw,
            PatternLimits {
                program_bytes: 0,
                ..PatternLimits::default()
            }
        )
        .is_err()
    );
}

/// Scenario: The accelerated representation exceeds the aggregate program ceiling but forward-only execution fits.
/// Guarantees: Compilation retries from the same HIR and reports the policy downgrade.
#[test]
fn aggregate_program_fallback() {
    let source = r"^[ab]*a[ab]{20}[ab]$";
    let hir = parse(source, PatternMode::Raw).expect("HIR");
    let unaccelerated = qualified(source, PatternMode::Raw, 0);
    let limit = unaccelerated.program_memory_usage();
    let program = build_with_payload_limit(
        &hir,
        PatternMode::Raw,
        PatternLimits::default(),
        false,
        false,
        limit,
    )
    .expect("forward-only payload fits");
    assert_eq!(
        program.execution_policy(),
        ExecutionPolicy::NoLazyDfa {
            requested_cache_bytes: MAX_LAZY_CACHE_BYTES,
            reason: AccelerationDisabled::ProgramLimit,
        }
    );
    assert!(program.program_memory_usage() <= limit);
    assert!(
        program
            .matcher()
            .is_match(PatternInput::Raw(&[b"a".as_slice(), &[b'b'; 21]].concat()))
            .expect("matching suffix")
    );
}

/// Scenario: Instrumentation detects a scratch-model underestimate during a search.
/// Guarantees: The violation is an error, latches terminal state, and later calls do not reuse the cache.
#[test]
fn memory_model_violation_is_terminal() {
    let mut program = qualified(
        r"^[ab]*a[ab]{20}[ab]$",
        PatternMode::Raw,
        MAX_LAZY_CACHE_BYTES,
    );
    let initial = program.program.regex.create_cache().memory_usage();
    Arc::get_mut(&mut program.program)
        .expect("unique test handle")
        .memory
        .worker_heap_bound = initial;
    let mut search = program.matcher();
    assert!(matches!(
        search.is_match(PatternInput::Raw(&[b'a'; 128])),
        Err(PatternError::MemoryModel { .. })
    ));
    let cache_after_failure = search.cache_memory_usage();
    assert!(matches!(
        search.is_match(PatternInput::Raw(b"")),
        Err(PatternError::MemoryModel { .. })
    ));
    assert_eq!(search.cache_memory_usage(), cache_after_failure);
}

fn audited_versions_match(lock: &str, name: &str, expected: &str) -> bool {
    let family = expected.rsplit_once('.').expect("audited semver").0;
    let prefix = format!("{family}.");
    let name_line = format!("name = \"{name}\"");
    let versions: Vec<_> = lock
        .split("[[package]]")
        .filter(|package| package.lines().any(|line| line.trim() == name_line))
        .filter_map(|package| {
            package
                .lines()
                .find_map(|line| line.trim().strip_prefix("version = \"")?.strip_suffix('"'))
        })
        .filter(|version| version.starts_with(&prefix))
        .collect();
    !versions.is_empty() && versions.iter().all(|version| *version == expected)
}

/// Scenario: An audited dependency or toolchain changes, or unrelated older regex versions coexist.
/// Guarantees: Upgrades require an audit, while LF/CRLF and unrelated release families remain accepted.
#[test]
fn memory_model_audit_versions() {
    let lock = include_str!("../../../../../../Cargo.lock");
    let manifest = include_str!("../../../../../../Cargo.toml");
    for (name, version) in [("regex-automata", "0.4.18"), ("regex-syntax", "0.8.11")] {
        let family = version.rsplit_once('.').unwrap().0;
        assert!(
            manifest
                .lines()
                .any(|line| line.starts_with(&format!("{name} = \"{family}."))),
            "re-audit multiline_pattern/memory.rs when changing dependency families"
        );
        let with_older = format!("{lock}\n[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\n");
        for contents in [lock.to_owned(), with_older] {
            let lf = contents.replace("\r\n", "\n");
            for contents in [lf.clone(), lf.replace('\n', "\r\n")] {
                assert!(
                    audited_versions_match(&contents, name, version),
                    "re-audit multiline_pattern/memory.rs after upgrading {name}"
                );
            }
        }
        let changed = format!("[[package]]\nname = \"{name}\"\nversion = \"{family}.999\"\n");
        assert!(!audited_versions_match(&changed, name, version));
        let older_only = format!("[[package]]\nname = \"{name}\"\nversion = \"0.1.0\"\n");
        assert!(!audited_versions_match(&older_only, name, version));
    }
    assert!(
        include_str!("../../../../../../rust-toolchain.toml").contains("channel = \"1.98.1\""),
        "re-audit collection growth on toolchain upgrade"
    );
}

/// Scenario: Parsed group nesting reaches and then exceeds the profile's depth limit.
/// Guarantees: Both text and raw compilation apply the documented boundary consistently.
#[test]
fn nesting_boundary() {
    for mode in [PatternMode::Text, PatternMode::Raw] {
        let accepted = format!("{}a{}", "(".repeat(64), ")".repeat(64));
        let rejected = format!("{}a{}", "(".repeat(65), ")".repeat(65));
        assert!(BoundaryPattern::compile(&accepted, mode).is_ok());
        assert!(BoundaryPattern::compile(&rejected, mode).is_err());
    }
}

/// Scenario: Nonliteral patterns exercise both accelerated and forced fallback execution.
/// Guarantees: The comparison builds a lazy DFA and agrees with NFA execution in both modes.
#[test]
fn confirmed_engine_paths_agree() {
    for mode in [PatternMode::Text, PatternMode::Raw] {
        for source in [
            r"^[ab]+$",
            r"\b[ab]+\b",
            r"(?i)^[a-c]+$",
            r"^[a-c]*$",
            r"^[\x{80}-\x{FF}]+$",
        ] {
            let fast = qualified(source, mode, MAX_LAZY_CACHE_BYTES);
            assert!(
                matches!(fast.execution_policy(), ExecutionPolicy::LazyDfa { .. }),
                "{source}"
            );
            let slow = qualified(source, mode, 0);
            for body in ["", "ab", "CAB", "zabz", "\u{e9}"] {
                let input = match mode {
                    PatternMode::Text => PatternInput::Text(body),
                    PatternMode::Raw => PatternInput::Raw(body.as_bytes()),
                };
                assert_eq!(
                    fast.matcher().is_match(input).unwrap(),
                    slow.matcher().is_match(input).unwrap(),
                    "{source}: {body}"
                );
            }
        }
    }
}
