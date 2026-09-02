// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Safe compilation of operator allowlist regex patterns for dynamic topic
//! routing.
//!
//! The dynamic-routing allowlist ([`SignalConfig::allowed_topics_regex`]) is an
//! authorization boundary for a client-controlled destination. Patterns are
//! matched against a whole topic, a guarantee that comes solely from anchoring
//! each pattern as `\A(?:<pattern>)\z`. [`compile_anchor_and_validate`]
//! centralizes that anchoring together with the standalone-validation step that
//! prevents a crafted pattern from escaping the anchors, so config-time
//! validation ([`super::config`]) and runtime compilation ([`super::exporter`])
//! stay in lock-step.
//!
//! [`SignalConfig::allowed_topics_regex`]: super::config::SignalConfig::allowed_topics_regex

use regex::Regex;

/// Validates a single operator allowlist pattern and compiles it into a
/// whole-topic-anchored [`Regex`], rejecting a pattern that could escape the
/// anchoring wrapper.
///
/// The dynamic-routing allowlist is an authorization boundary for a
/// client-controlled destination, and the whole-topic guarantee comes solely
/// from anchoring the pattern as `\A(?:<pattern>)\z`. A pattern whose
/// parentheses balance against that wrapper (for example `tenant_.)\z|(?:evil.`
/// wraps to `\A(?:tenant_.)\z|(?:evil.)\z`) can close the `(?:` group early and
/// introduce a top-level alternation that loses the `\A` anchor, silently
/// permitting unintended topics -- a privilege escalation on this boundary.
///
/// To close that gap, the operator pattern is first compiled **on its own**:
/// any wrapper-escaping pattern requires an unbalanced parenthesis, which makes
/// it an invalid standalone regex, so this step rejects the escape. Only a
/// pattern that is a valid, self-contained regex (and therefore stays inside
/// the `(?:...)` group) is then compiled in the anchored form that the router
/// matches against.
///
/// This is a dependency-light utility: it returns the regex-compilation message
/// as a `String` on failure, leaving each caller to attach its own context
/// (signal, offending pattern) and error type.
///
/// # Errors
///
/// Returns `Err` with the regex-compilation message if `pattern` is not a valid
/// standalone regular expression, or if the anchored form fails to compile.
pub(crate) fn compile_anchor_and_validate(pattern: &str) -> Result<Regex, String> {
    // Compile the bare pattern first. A pattern crafted to break out of the
    // anchoring wrapper needs an unbalanced parenthesis and so fails here,
    // before it can be given a chance to defeat the whole-topic anchors.
    let _ = Regex::new(pattern).map_err(|e| e.to_string())?;
    // Anchor to a whole-topic match: with a self-contained pattern the anchors
    // always apply, so a header-supplied topic cannot slip past a prefix or
    // suffix pattern via a substring match.
    Regex::new(&format!(r"\A(?:{pattern})\z")).map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a benign operator allowlist pattern (including a balanced
    /// top-level alternation group) is validated and compiled.
    /// Guarantees: `compile_anchor_and_validate` accepts valid standalone
    /// patterns and anchors them, so `tenant_.*` permits the whole topic
    /// `tenant_a` but rejects a topic that merely contains it, and
    /// `(tenant_a|tenant_b).*` matches only topics that start with an allowed
    /// tenant prefix.
    #[test]
    fn compile_anchor_and_validate_accepts_and_anchors_valid_patterns() {
        let re = compile_anchor_and_validate("tenant_.*").expect("valid pattern compiles");
        assert!(re.is_match("tenant_a"), "whole-topic match permitted");
        assert!(
            !re.is_match("evil-tenant_a-x"),
            "substring match must NOT be permitted (authorization boundary)"
        );

        let grouped =
            compile_anchor_and_validate("(tenant_a|tenant_b).*").expect("balanced group compiles");
        assert!(
            grouped.is_match("tenant_a_logs"),
            "allowed tenant permitted"
        );
        assert!(
            !grouped.is_match("x-tenant_a"),
            "a leading-prefixed topic must NOT be permitted"
        );
    }

    /// Scenario: an operator pattern whose parentheses balance against the
    /// `\A(?:<pattern>)\z` anchoring wrapper (e.g. `tenant_.)\z|(?:evil.`) is
    /// validated. Wrapped, it would become
    /// `\A(?:tenant_.)\z|(?:evil.)\z`, whose second alternative drops the `\A`
    /// anchor and would permit any topic ending in `evil` + one character.
    /// Guarantees: `compile_anchor_and_validate` rejects the pattern (returning a
    /// non-empty error message) because it is not a valid standalone regex,
    /// closing the anchor-breakout authorization bypass before the pattern can
    /// ever match a topic.
    #[test]
    fn compile_anchor_and_validate_rejects_anchor_breakout() {
        let breakout = r"tenant_.)\z|(?:evil.";
        // Sanity: the naive wrapped-only compile (the old behavior) would have
        // accepted this and matched an unintended topic.
        let anchored = format!(r"\A(?:{breakout})\z");
        let naive = Regex::new(&anchored).expect("wrapped breakout compiles under naive anchoring");
        assert!(
            naive.is_match("x-evilY"),
            "precondition: the wrapped breakout under-anchors and permits an unintended topic"
        );

        // The validating helper rejects it up front.
        let err = compile_anchor_and_validate(breakout)
            .expect_err("anchor-breakout pattern must be rejected");
        assert!(!err.is_empty(), "an error message must be reported");
    }

    /// Scenario: an ordinary malformed operator pattern (unclosed character
    /// class) is validated.
    /// Guarantees: `compile_anchor_and_validate` returns an error message rather
    /// than a regex, so a caller can surface an actionable diagnostic.
    #[test]
    fn compile_anchor_and_validate_reports_error_message_on_invalid_pattern() {
        let err = compile_anchor_and_validate("[").expect_err("invalid regex must be rejected");
        assert!(!err.is_empty(), "an error message must be reported");
    }
}
