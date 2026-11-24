// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::cmp::Ordering;

use crate::model::{
    LogRecordField, LogRedact, LogTargetSelector, NumericValue, PolicyError, Predicate,
    ScalarValue, apply_regex_redaction, cmp_i64_with_f64, compile_matcher_regex,
    compile_redact_regex, eq_i64_f64, matches_glob,
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

    // PolicyError Display formatting
    let compile_err = PolicyError::Compile("bad".into());
    assert!(compile_err.to_string().contains("bad"));
    let eval_err = PolicyError::Evaluation("oops".into());
    assert!(eval_err.to_string().contains("oops"));
}

/// Scenario: Evaluate `matches_glob` across `*`, `?`, multi-byte UTF-8 characters, newlines, regex metacharacters, case sensitivity, and pathological repeated-star backtracking patterns.
/// Guarantees: `matches_glob` enforces exact full-string case-sensitive `policy.md` glob semantics in linear time without treating regex metacharacters specially.
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

    // Linear-time behavior on adversarial repeated-wildcard mismatch
    let evil_pat = "*a".repeat(20) + "b";
    let evil_txt = "a".repeat(60);
    assert!(!matches_glob(&evil_pat, &evil_txt));
}

/// Scenario: Compile matcher and redact regexes using disallowed non-RE2 features (lookarounds, backreferences, `(?x)`/`(?u)`/`(?R)` flags, character-class set operators, nested bracketed classes, non-RE2 assertions, `\u`/`\U` hex escapes, `\p{k=v}` property ops, and non-ASCII/punctuated capture names) alongside valid RE2 equivalents and deeply nested valid RE2 expressions.
/// Guarantees: All non-RE2 syntax is rejected at compile time while valid RE2 `\x` escapes, `\p{Name}` classes, `[0-9A-Za-z_]` capture names, and depth-60 nested RE2 expressions compile cleanly without double-parsing overhead.
#[test]
fn test_re2_validation_rejects_non_re2_and_accepts_deep_valid_re2() {
    let disallowed = [
        r"(?=foo)bar",                  // positive lookahead
        r"(?!foo)bar",                  // negative lookahead
        r"(?<=foo)bar",                 // lookbehind
        r"(a)\1",                       // backreference
        r"(?x)foo bar",                 // ignore-whitespace flag
        r"(?u)\w+",                     // Rust-only unicode flag
        r"(?-u)\w+",                    // Rust-only negated unicode flag
        r"(?R)^foo$",                   // Rust-only CRLF flag
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
        r"\pL+\p{Greek}+[\p{Latin}]",
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
}
