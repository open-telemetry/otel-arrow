// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;

use crate::model::{
    Action, AttributePath, LogFieldSelector, LogMatcher, LogRecordField, LogRedact,
    LogTargetSelector, LogTransformPolicy, MAX_REGEX_NESTING, MAX_REGEX_PATTERN_BYTES,
    NumericValue, Policy, PolicyCompileError, PolicyDefinition, PolicyError, Predicate,
    RE2_UNICODE_GROUPS, ScalarValue, apply_regex_redaction, apply_regex_redaction_with_caps,
    cmp_i64_with_f64, compile_matcher_regex, compile_redact_regex, eq_i64_f64, matches_glob,
    parse_re2_hir,
};

/// Scenario: Compare `i64` against special `f64` values (`NaN`, `+/-inf`, `+/-0.0`), `2^63`/`-2^63` boundaries, fractional remainders, and integers above `2^53`.
/// Guarantees: `cmp_i64_with_f64` and `eq_i64_f64` return `None`/`false` for `NaN`, handle `+/-2^63` without overflow or precision loss, and order fractional and large integers exactly.
#[test]
fn test_cmp_i64_with_f64_and_numeric_value() {
    // Special floating-point values
    assert_eq!(cmp_i64_with_f64(0, f64::NAN), None);
    assert_eq!(
        cmp_i64_with_f64(i64::MAX, f64::INFINITY),
        Some(Ordering::Less)
    );
    assert_eq!(
        cmp_i64_with_f64(i64::MIN, f64::NEG_INFINITY),
        Some(Ordering::Greater)
    );

    // Bounds around 2^63 and -2^63
    assert_eq!(
        cmp_i64_with_f64(i64::MAX, 9_223_372_036_854_775_808.0),
        Some(Ordering::Less)
    );
    assert_eq!(cmp_i64_with_f64(42, 1e30), Some(Ordering::Less));
    assert_eq!(
        cmp_i64_with_f64(i64::MIN, -9_223_372_036_854_775_808.0),
        Some(Ordering::Equal)
    );
    assert_eq!(cmp_i64_with_f64(i64::MIN, -1e30), Some(Ordering::Greater));
    assert_eq!(
        cmp_i64_with_f64(i64::MAX, 9_223_372_036_854_774_784.0),
        Some(Ordering::Greater)
    );

    // Integral comparisons
    assert_eq!(cmp_i64_with_f64(10, 20.0), Some(Ordering::Less));
    assert_eq!(cmp_i64_with_f64(20, 10.0), Some(Ordering::Greater));
    assert_eq!(cmp_i64_with_f64(10, 10.0), Some(Ordering::Equal));
    assert_eq!(cmp_i64_with_f64(0, -0.0), Some(Ordering::Equal));

    // Fractional positive and negative remainders when truncated parts match
    assert_eq!(cmp_i64_with_f64(5, 5.25), Some(Ordering::Less));
    assert_eq!(cmp_i64_with_f64(0, 0.5), Some(Ordering::Less));
    assert_eq!(cmp_i64_with_f64(-5, -5.25), Some(Ordering::Greater));
    assert_eq!(cmp_i64_with_f64(0, -0.5), Some(Ordering::Greater));

    // Exact 64-bit precision above 2^53
    assert_eq!(
        cmp_i64_with_f64(9_007_199_254_740_991, 9_007_199_254_740_992.0),
        Some(Ordering::Less)
    );
    assert_eq!(
        cmp_i64_with_f64(9_007_199_254_740_992, 9_007_199_254_740_992.0),
        Some(Ordering::Equal)
    );
    assert_eq!(
        cmp_i64_with_f64(9_007_199_254_740_993, 9_007_199_254_740_992.0),
        Some(Ordering::Greater)
    );

    // eq_i64_f64 helper
    assert!(eq_i64_f64(42, 42.0));
    assert!(eq_i64_f64(i64::MIN, -9_223_372_036_854_775_808.0));
    assert!(!eq_i64_f64(42, 42.5));
    assert!(!eq_i64_f64(42, f64::NAN));
    assert!(!eq_i64_f64(9_007_199_254_740_993, 9_007_199_254_740_992.0));

    // NumericValue::cmp_candidate_i64
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_i64(5),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_i64(10),
        Some(Ordering::Equal)
    );
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_i64(15),
        Some(Ordering::Greater)
    );
    assert_eq!(NumericValue::Double(f64::NAN).cmp_candidate_i64(10), None);
    assert_eq!(
        NumericValue::Double(10.5).cmp_candidate_i64(10),
        Some(Ordering::Less)
    );

    // NumericValue::cmp_candidate_f64 (NaN, Int threshold, Double threshold)
    assert_eq!(NumericValue::Int(10).cmp_candidate_f64(f64::NAN), None);
    assert_eq!(NumericValue::Double(10.0).cmp_candidate_f64(f64::NAN), None);
    assert_eq!(NumericValue::Double(f64::NAN).cmp_candidate_f64(10.0), None);
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_f64(9.5),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_f64(10.0),
        Some(Ordering::Equal)
    );
    assert_eq!(
        NumericValue::Int(10).cmp_candidate_f64(10.5),
        Some(Ordering::Greater)
    );
    assert_eq!(
        NumericValue::Int(i64::MAX).cmp_candidate_f64(f64::INFINITY),
        Some(Ordering::Greater)
    );
    assert_eq!(
        NumericValue::Int(i64::MIN).cmp_candidate_f64(f64::NEG_INFINITY),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Int(9_007_199_254_740_993).cmp_candidate_f64(9_007_199_254_740_992.0),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Double(10.0).cmp_candidate_f64(9.25),
        Some(Ordering::Less)
    );
    assert_eq!(
        NumericValue::Double(10.0).cmp_candidate_f64(10.0),
        Some(Ordering::Equal)
    );
    assert_eq!(
        NumericValue::Double(10.0).cmp_candidate_f64(10.25),
        Some(Ordering::Greater)
    );
}

/// Scenario: Evaluate `Gt`, `Gte`, `Lt`, and `Lte` predicates across `Less`, `Equal`, `Greater`, and `None` orderings.
/// Guarantees: Each comparison predicate matches only its strict or inclusive ordering outcomes and returns `false` when ordering is undefined (`None`).
#[test]
fn test_predicate_matches_ordering() {
    let gt = Predicate::Gt(NumericValue::Int(0));
    let gte = Predicate::Gte(NumericValue::Int(0));
    let lt = Predicate::Lt(NumericValue::Int(0));
    let lte = Predicate::Lte(NumericValue::Int(0));

    assert!(!gt.matches_ordering(None));
    assert!(!gte.matches_ordering(None));
    assert!(!lt.matches_ordering(None));
    assert!(!lte.matches_ordering(None));

    assert!(!gt.matches_ordering(Some(Ordering::Less)));
    assert!(!gt.matches_ordering(Some(Ordering::Equal)));
    assert!(gt.matches_ordering(Some(Ordering::Greater)));

    assert!(!gte.matches_ordering(Some(Ordering::Less)));
    assert!(gte.matches_ordering(Some(Ordering::Equal)));
    assert!(gte.matches_ordering(Some(Ordering::Greater)));

    assert!(lt.matches_ordering(Some(Ordering::Less)));
    assert!(!lt.matches_ordering(Some(Ordering::Equal)));
    assert!(!lt.matches_ordering(Some(Ordering::Greater)));

    assert!(lte.matches_ordering(Some(Ordering::Less)));
    assert!(lte.matches_ordering(Some(Ordering::Equal)));
    assert!(!lte.matches_ordering(Some(Ordering::Greater)));

    for non_range in [
        Predicate::Exists,
        Predicate::Equals(ScalarValue::Int(0)),
        Predicate::Regex("a".into()),
        Predicate::Contains(ScalarValue::String("a".into())),
    ] {
        assert!(!non_range.matches_ordering(Some(Ordering::Equal)));
        assert!(!non_range.matches_ordering(Some(Ordering::Greater)));
        assert!(!non_range.matches_ordering(Some(Ordering::Less)));
    }
}

/// Scenario: Run `apply_regex_redaction` with literal replacements, `$$`, `$0`/`$1`/`$10`, `${1}`/`${name}`, malformed braced templates like `{"a": "${raw", "b": "$1"}` and `${+1}`, missing groups, and default replacement fallback.
/// Guarantees: Non-matching inputs return `None`, valid numbered and named captures expand accurately, malformed braced keys stay literal without consuming later `$1` captures, and unset replacement defaults to `"[REDACTED]"`.
#[test]
fn test_regex_redaction_and_capture_templates() {
    // No match returns None; verify both capture-enabled and capture-free (`WhichCaptures::Implicit`) paths.
    let re_user_caps = compile_redact_regex(r"user=(?P<name>[a-z]+)", true).unwrap();
    let re_user_lit = compile_redact_regex(r"user=(?P<name>[a-z]+)", false).unwrap();
    assert_eq!(
        apply_regex_redaction(&re_user_caps, "no match here", "X"),
        None
    );
    assert_eq!(
        apply_regex_redaction(&re_user_lit, "no match here", "X"),
        None
    );

    // Literal replacement across multiple matches (including empty replacement string `""`)
    assert_eq!(
        apply_regex_redaction(&re_user_lit, "a user=alice and user=bob!", "REDACTED"),
        Some("a REDACTED and REDACTED!".into())
    );
    assert_eq!(
        apply_regex_redaction(&re_user_lit, "a user=alice and user=bob!", ""),
        Some("a  and !".into())
    );

    // Escaped $$, $0, $1, ${1}, ${name}, missing captures, trailing $, unclosed ${, unknown $_
    assert_eq!(
        apply_regex_redaction(
            &re_user_caps,
            "user=alice",
            "$$0=$0,${1},${name},${99},${missing}"
        ),
        Some("$0=user=alice,alice,alice,,".into())
    );
    assert_eq!(
        apply_regex_redaction(&re_user_caps, "user=alice", "tail$"),
        Some("tail$".into())
    );
    assert_eq!(
        apply_regex_redaction(&re_user_caps, "user=alice", "open${unclosed"),
        Some("open${unclosed".into())
    );
    assert_eq!(
        apply_regex_redaction(&re_user_caps, "user=alice", "$_keep$x"),
        Some("$_keep$x".into())
    );
    // Invalid braced capture identifiers must not eat later literal text/captures or accept `${+1}`
    assert_eq!(
        apply_regex_redaction(&re_user_caps, "user=alice", r#"{"a": "${raw", "b": "$1"}"#),
        Some(r#"{"a": "${raw", "b": "alice"}"#.into())
    );
    assert_eq!(
        apply_regex_redaction(&re_user_caps, "user=alice", "${+1},${}"),
        Some("${+1},${}".into())
    );

    // Without (?m), `$` must NOT match before a trailing newline; with (?m), it matches line end.
    let re_end = compile_redact_regex(r"^ERROR$", false).unwrap();
    assert_eq!(apply_regex_redaction(&re_end, "ERROR\n", "X"), None);
    assert_eq!(
        apply_regex_redaction(&re_end, "ERROR", "X"),
        Some("X".into())
    );
    let re_end_multiline = compile_redact_regex(r"(?m)^ERROR$", false).unwrap();
    assert_eq!(
        apply_regex_redaction(&re_end_multiline, "ERROR\nINFO", "X"),
        Some("X\nINFO".into())
    );

    // Two-digit capture indexes ($10, $12) and missing capture ($20, unmatched optional)
    let re_twelve =
        compile_redact_regex(r"(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)(k)(l)(m)?", true).unwrap();
    assert_eq!(
        apply_regex_redaction(&re_twelve, "abcdefghijkl", "$10-$12-$13-$20-${11}"),
        Some("j-l---k".into())
    );

    // Empty redact regex is rejected per policy.md ("Redact")
    assert!(compile_redact_regex("", false).is_err());

    // LogRedact::effective_replacement default vs explicit
    let default_redact = LogRedact {
        target: LogTargetSelector::RecordField(LogRecordField::Body),
        replacement: None,
        regex: None,
    };
    assert_eq!(default_redact.effective_replacement(), "[REDACTED]");
    let custom_redact = LogRedact {
        target: LogTargetSelector::RecordField(LogRecordField::Body),
        replacement: Some("MASK".into()),
        regex: Some("secret".into()),
    };
    assert_eq!(custom_redact.effective_replacement(), "MASK");

    // Reusable `Captures` buffer via `apply_regex_redaction_with_caps` must not leak stale
    // optional-group captures across alternating matches or calls.
    let re_alt = compile_redact_regex(r"(a)|(b)", true).unwrap();
    let mut caps = re_alt.create_captures();
    assert_eq!(
        apply_regex_redaction_with_caps(&re_alt, "ab", "[$1:$2]", Some(&mut caps)),
        Some("[a:][:b]".into())
    );
    assert_eq!(
        apply_regex_redaction_with_caps(&re_alt, "b", "[$1:$2]", Some(&mut caps)),
        Some("[:b]".into())
    );
    assert_eq!(
        apply_regex_redaction_with_caps(&re_alt, "z", "[$1:$2]", Some(&mut caps)),
        None
    );

    // PolicyError and PolicyCompileError Display formatting
    let compile_err = PolicyError::Compile("bad".into());
    assert!(compile_err.to_string().contains("bad"));
    let eval_err = PolicyError::Evaluation("oops".into());
    assert!(eval_err.to_string().contains("oops"));
    let policy_err = PolicyCompileError {
        policy_id: "p1".into(),
        message: "invalid regex".into(),
    };
    assert_eq!(policy_err.to_string(), "policy 'p1': invalid regex");
}

/// Scenario: Evaluate `matches_glob` across `*`, `?`, multi-byte UTF-8 characters, newlines, regex metacharacters, case sensitivity, and pathological repeated-star backtracking patterns.
/// Guarantees: `matches_glob` enforces exact full-string case-sensitive `policy.md` glob semantics in polynomial time (non-exponential) without treating regex metacharacters specially.
#[test]
fn test_matches_glob_wildcards_unicode_newlines_and_metachars() {
    // Empty and wildcard-only patterns
    assert!(matches_glob("", ""));
    assert!(!matches_glob("", "a"));
    assert!(matches_glob("*", ""));
    assert!(matches_glob("*", "anything\nwith\nnewlines"));
    assert!(matches_glob("***", ""));
    assert!(!matches_glob("?", ""));
    assert!(matches_glob("?", "a"));
    assert!(matches_glob("?", "\n"));
    assert!(matches_glob("?", "\u{03b1}")); // single 2-byte UTF-8 char
    assert!(matches_glob("?", "\u{1f680}")); // single 4-byte UTF-8 char
    assert!(!matches_glob("?", "ab"));

    // Case sensitivity and full-string anchoring
    assert!(matches_glob("http.*", "http.route"));
    assert!(!matches_glob("http.*", "HTTP.route"));
    assert!(!matches_glob("http", "http.route"));
    assert!(matches_glob("a?c*e", "a\u{00e9}cxyz\ne"));
    assert!(!matches_glob("a?c*e", "acxyze"));

    // Regex metacharacters match literally (not as regex syntax)
    let meta = r".+()[]^$|\";
    assert!(matches_glob(meta, meta));
    assert!(!matches_glob("a.b", "axb"));
    assert!(matches_glob("a.b", "a.b"));
    assert!(matches_glob("[a-z]+", "[a-z]+"));
    assert!(!matches_glob("[a-z]+", "abc"));

    // Non-exponential behavior on adversarial repeated-wildcard mismatch
    let evil_pat = "*a".repeat(20) + "b";
    let evil_txt = "a".repeat(60);
    assert!(!matches_glob(&evil_pat, &evil_txt));
}

/// Scenario: Compile matcher and redact regexes using disallowed non-RE2 features (lookarounds, backreferences, `(?x)`/`(?u)`/`(?R)` flags, character-class set operators, nested bracketed classes, non-RE2 assertions, `\u`/`\U` hex escapes, `\p{k=v}` property ops, non-RE2 Unicode binary/long/lowercase properties, and non-ASCII/punctuated capture names) alongside valid RE2 equivalents, NFA/length/depth bounds, and RE2 ASCII-only Perl classes and boundaries.
/// Guarantees: All non-RE2 syntax and oversized patterns are rejected at compile time, `RE2_UNICODE_GROUPS` stays strictly sorted, valid RE2 `\x` escapes, `\p{Name}` classes, `[0-9A-Za-z_]` capture names, and depth-60 nested RE2 expressions compile cleanly, and `\d`/`\w`/`\s`/`\D`/`\W`/`\S`/`\b`/`\B` (standalone and inside `[...]`) match RE2 ASCII semantics.
#[test]
fn test_re2_validation_rejects_non_re2_and_accepts_deep_valid_re2() {
    assert!(
        RE2_UNICODE_GROUPS.windows(2).all(|w| w[0] < w[1]),
        "RE2_UNICODE_GROUPS must remain strictly sorted for binary_search"
    );
    // Every allowlisted RE2 Unicode group must translate cleanly through `regex-syntax`.
    for &group in RE2_UNICODE_GROUPS {
        let pat = format!(r"\p{{{group}}}");
        assert!(
            parse_re2_hir(&pat).is_ok(),
            "allowlisted RE2 Unicode group failed HIR translation: {pat}"
        );
    }

    let disallowed = [
        r"(?=foo)bar",                  // positive lookahead
        r"(?!foo)bar",                  // negative lookahead
        r"(?<=foo)bar",                 // lookbehind
        r"(a)\1",                       // backreference
        r"(?x)foo bar",                 // ignore-whitespace flag
        r"(?x:foo bar)",                // scoped non-RE2 ignore-whitespace flag
        r"(?u)\w+",                     // Rust-only unicode flag
        r"(?-u)\w+",                    // Rust-only negated unicode flag
        r"(?-u:\w+)",                   // scoped Rust-only negated unicode flag
        r"(?R)^foo$",                   // Rust-only CRLF flag
        r"(?R:^foo$)",                  // scoped Rust-only CRLF flag
        r"[a-z&&[^c]]",                 // class intersection
        r"[a-z--c]",                    // class difference
        r"[a-z~~c]",                    // class symmetric diff
        r"[[a-z]0-9]",                  // nested bracketed class
        r"\<word\>",                    // non-RE2 word-boundary assertions
        r"\b{start}word",               // Rust-only braced word boundary
        r"\u0041",                      // non-RE2 \u escape
        r"\U00000041",                  // non-RE2 \U escape
        r"\u{41}",                      // non-RE2 \u{...} escape
        r"\U{41}",                      // non-RE2 \U{...} escape
        r"[\u0041]",                    // non-RE2 \u inside class
        r"[\u0041-\u005a]",             // non-RE2 \u in class range
        r"[A-\u005a]",                  // non-RE2 \u as range end
        r"\p{scx=Greek}",               // non-RE2 Unicode key=value property
        r"\p{General_Category:Letter}", // non-RE2 Unicode key:value property
        r"[\p{Script!=Latin}]",         // non-RE2 Unicode key!=value inside class
        r"\p{White_Space}",             // non-RE2 Unicode binary property
        r"\p{^White_Space}",            // negated non-RE2 Unicode binary property
        r"\p{^^Greek}",                 // double caret invalid in RE2
        r"\p{Cs}",                      // UTF-16 surrogates unrepresentable in UTF-8
        r"\p{Emoji}",                   // non-RE2 Unicode binary property
        r"\p{Alphabetic}",              // non-RE2 Unicode binary property
        r"\p{Hex_Digit}",               // non-RE2 Unicode binary property
        r"\p{Lowercase}",               // non-RE2 Unicode binary property
        r"\p{Uppercase}",               // non-RE2 Unicode binary property
        r"\p{Letter}",                  // non-RE2 long General Category name
        r"\p{ASCII}",                   // non-RE2 shorthand
        r"\p{Assigned}",                // non-RE2 shorthand
        r"\p{Latn}",                    // non-RE2 4-letter ISO script alias
        r"\p{greek}",                   // non-canonical lowercase script name
        r"\p{lu}",                      // non-canonical lowercase category name
        r"\pl",                         // non-canonical lowercase 1-letter class
        r"[\p{White_Space}]",           // non-RE2 binary property inside bracketed class
        r"(?P<a.b>foo)",                // non-RE2 capture name containing '.'
        r"(?<a[b]>foo)",                // non-RE2 capture name containing '['/']'
    ];
    for pat in disallowed {
        assert!(
            compile_matcher_regex(pat).is_err(),
            "matcher regex must reject non-RE2 pattern: {pat}"
        );
        assert!(
            compile_redact_regex(pat, false).is_err(),
            "redact regex must reject non-RE2 pattern: {pat}"
        );
    }

    // Valid RE2 flags (`i`, `m`, `s`, `U`), `\x` escapes, `\p{Name}` / `\pL`, valid capture names,
    // and depth-60 nested group compile cleanly.
    for allowed in [
        r"(?imsU)^foo.*bar$",
        r"\x41\x{42}[\x41-\x5a]",
        r"\pL+\p{Lu}+\p{Greek}+[\p{Latin}\p{Nd}]",
        r"\p{Any}\p{Common}\p{Inherited}\p{SignWriting}",
        r"(?P<user_1>foo)(?<user_2>bar)",
    ] {
        assert!(
            compile_matcher_regex(allowed).is_ok(),
            "valid RE2 pattern must compile: {allowed}"
        );
        assert!(
            compile_redact_regex(allowed, true).is_ok(),
            "valid RE2 redact pattern must compile: {allowed}"
        );
    }
    let deep_valid = format!("{}ok{}", "(".repeat(60), ")".repeat(60));
    let (_, re) = compile_matcher_regex(&deep_valid).expect("depth 60 valid RE2");
    assert!(re.is_match("ok"));

    // Regex source length, nesting depth, and Thompson NFA size bounds are enforced at compile time.
    let too_long = "a".repeat(MAX_REGEX_PATTERN_BYTES + 1);
    assert!(compile_matcher_regex(&too_long).is_err());
    let too_deep = format!(
        "{}ok{}",
        "(".repeat((MAX_REGEX_NESTING + 2) as usize),
        ")".repeat((MAX_REGEX_NESTING + 2) as usize)
    );
    assert!(compile_matcher_regex(&too_deep).is_err());
    assert!(compile_matcher_regex("a{1,100000}").is_err());
    assert!(compile_redact_regex("a{1,100000}", false).is_err());

    // RE2 ASCII-only shorthands: \d, \w, \s (excluding \x0b), \D, \W, \S, \b, \B,
    // both standalone and inside bracketed character classes `[...]`.
    let (_, re_digit) = compile_matcher_regex(r"^\d+$").expect("compile \\d");
    assert!(re_digit.is_match("123"));
    assert!(!re_digit.is_match("\u{0661}")); // Arabic-Indic digit 1
    let (_, re_bracket_digit) = compile_matcher_regex(r"^[\d]+$").expect("compile [\\d]");
    assert!(re_bracket_digit.is_match("123"));
    assert!(!re_bracket_digit.is_match("\u{0661}"));
    let (_, re_neg_not_digit) = compile_matcher_regex(r"^[^\D]+$").expect("compile [^\\D]");
    assert!(re_neg_not_digit.is_match("123"));
    assert!(!re_neg_not_digit.is_match("\u{0661}"));
    let (_, re_not_digit) = compile_matcher_regex(r"^\D+$").expect("compile \\D");
    assert!(re_not_digit.is_match("\u{0661}"));

    // RE2 negated-braced Unicode classes: \p{^Name} == \P{Name}, \P{^Name} == \p{Name}
    let (_, re_not_greek) = compile_matcher_regex(r"^\p{^Greek}$").expect("compile \\p{^Greek}");
    assert!(re_not_greek.is_match("a"));
    assert!(!re_not_greek.is_match("\u{03b1}"));
    let (_, re_double_neg_greek) =
        compile_matcher_regex(r"^\P{^Greek}$").expect("compile \\P{^Greek}");
    assert!(re_double_neg_greek.is_match("\u{03b1}"));
    assert!(!re_double_neg_greek.is_match("a"));
    let (_, re_bracket_not_nd) = compile_matcher_regex(r"^[\p{^Nd}]$").expect("compile [\\p{^Nd}]");
    assert!(re_bracket_not_nd.is_match("a"));
    assert!(!re_bracket_not_nd.is_match("5"));

    let (_, re_word) = compile_matcher_regex(r"^\w+$").expect("compile \\w");
    assert!(re_word.is_match("abc_123"));
    assert!(!re_word.is_match("caf\u{00e9}"));
    let (_, re_bracket_word) = compile_matcher_regex(r"^[\w]+$").expect("compile [\\w]");
    assert!(re_bracket_word.is_match("abc_123"));
    assert!(!re_bracket_word.is_match("caf\u{00e9}"));
    let (_, re_not_word) = compile_matcher_regex(r"^\W+$").expect("compile \\W");
    assert!(re_not_word.is_match("\u{00e9}"));
    // RE2 Unicode case-folding parity under (?i): \w and [\w] fold ASCII k/s to Kelvin sign
    // (\u{212a}) and long s (\u{017f}), and (?i)\W excludes them.
    let (_, re_ci_word) = compile_matcher_regex(r"(?i)^\w+$").expect("compile (?i)\\w");
    let (_, re_ci_bracket_word) = compile_matcher_regex(r"(?i)^[\w]+$").expect("compile (?i)[\\w]");
    let (_, re_ci_not_word) = compile_matcher_regex(r"(?i)^\W+$").expect("compile (?i)\\W");
    for folded in ["\u{212a}", "\u{017f}"] {
        assert!(re_ci_word.is_match(folded));
        assert!(re_ci_bracket_word.is_match(folded));
        assert!(!re_ci_not_word.is_match(folded));
    }

    let (_, re_space) = compile_matcher_regex(r"^\s+$").expect("compile \\s");
    assert!(re_space.is_match(" \t\n\r\u{000c}"));
    assert!(!re_space.is_match("\x0b")); // RE2 \s excludes vertical tab \v (\x0b)
    assert!(!re_space.is_match("\u{00a0}")); // non-breaking space
    let (_, re_bracket_space) = compile_matcher_regex(r"^[\s]+$").expect("compile [\\s]");
    assert!(re_bracket_space.is_match(" \t\n\r\u{000c}"));
    assert!(!re_bracket_space.is_match("\x0b"));
    assert!(!re_bracket_space.is_match("\u{00a0}"));
    let (_, re_not_space) = compile_matcher_regex(r"^\S+$").expect("compile \\S");
    assert!(re_not_space.is_match("\x0b"));

    let (_, re_bound) = compile_matcher_regex(r"\bfoo\b").expect("compile \\b");
    assert!(re_bound.is_match("foo\u{00e9}")); // ASCII word boundary before non-ASCII char
    let (_, re_not_bound) = compile_matcher_regex(r"\Bfoo\B").expect("compile \\B");
    assert!(re_not_bound.is_match("xfooy"));
    assert!(!re_not_bound.is_match("\u{00e9}foo\u{00e9}"));
}

/// Scenario: Construct `AttributePath` (`single` and `nested`) and `Policy` (`log_filter` and `log_transform`) via helper constructors.
/// Guarantees: Constructors populate expected path segments, default `enabled = true`, mirror `name` from `id`, and wrap the matching `PolicyDefinition` variant.
#[test]
fn test_attribute_path_and_policy_constructors() {
    assert_eq!(
        AttributePath::single("http.method").path,
        vec!["http.method"]
    );
    assert_eq!(
        AttributePath::nested(["user", "profile", "id"]).path,
        vec!["user", "profile", "id"]
    );

    let filter = Policy::log_filter(
        "drop_debug",
        vec![LogMatcher {
            field: LogFieldSelector::RecordField(LogRecordField::SeverityText),
            predicate: Predicate::Equals(ScalarValue::String("DEBUG".into())),
            negate: false,
        }],
        Action::Drop,
    );
    assert_eq!(filter.id, "drop_debug");
    assert_eq!(filter.name, "drop_debug");
    assert!(filter.enabled);
    assert!(filter.description.is_empty());
    assert!(filter.labels.is_empty());
    assert!(matches!(filter.definition, PolicyDefinition::LogFilter(_)));

    let transform = Policy::log_transform(
        "redact_body",
        LogTransformPolicy {
            matches: Vec::new(),
            rename: Vec::new(),
            add: Vec::new(),
            remove: Vec::new(),
            redact: Vec::new(),
        },
    );
    assert_eq!(transform.id, "redact_body");
    assert_eq!(transform.name, "redact_body");
    assert!(transform.enabled);
    assert!(matches!(
        transform.definition,
        PolicyDefinition::LogTransform(_)
    ));
}
