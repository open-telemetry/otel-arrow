// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Instrumentation key (iKey) validation and per-key event filtering.
//!
//! This module provides the [`IkeyValidator`] entry point for checking whether
//! an event should be accepted, rejected, or silently dropped based on its
//! instrumentation key, event name, and request origin.
//!
//! ## Configuration
//!
//! Rules can be inline in an endpoint configuration:
//!
//! ```yaml
//! ikey_filter:
//!   keys:
//!     "o:bad-key-guid":
//!       action: reject
//!     "o:limited-key":
//!       action: accept
//!       allowed_events:
//!         - "Ms.Web.PageView"
//!         - "Ms.Device.*"
//!     "*":
//!       action: accept
//! ```
//!
//! Or loaded from an external YAML file containing the same root `keys` map:
//!
//! ```yaml
//! ikey_filter:
//!   filter_file: "ikey-rules.yaml"
//! ```
//!
//! An external file takes precedence over inline rules. Relative file paths
//! are confined to the pipeline configuration directory when it is supplied.
//!
//! ## Matching rules
//!
//! - Key patterns are evaluated in declaration order; the first match wins.
//!   Duplicate key patterns are configuration errors.
//! - `"*"` matches any key, including a missing key. Other key patterns are
//!   exact, case-sensitive strings.
//! - Accept rules check required origins first, then event-name allow-lists,
//!   then return an optional remapping. A remapped key is not re-evaluated.
//! - Origins match case-insensitively; `"*"` allows any supplied origin but
//!   does not allow a missing origin when `require_origin` is true.
//! - Event patterns support exact names, a trailing `*` prefix glob, or `"*"`.
//!   A missing event name skips this check; an empty allow-list allows all.
//! - A failed origin check returns [`FilterVerdict::RejectUnauthorized`].
//!   A failed event-name check returns [`FilterVerdict::Reject`].
//! - Unmatched keys are accepted (open policy). End with `"*": { action: reject }`
//!   for a closed allow-list.
//!
//! The receiver skeleton does not yet invoke this module during construction
//! or request handling.

#![allow(dead_code)]

use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};
use std::collections::HashSet;
use std::fmt;
use std::path::{Component, Path, PathBuf};

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Result of checking an event through the validator.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum FilterVerdict {
    /// Event is allowed.
    Accept,
    /// Event is allowed but the iKey should be remapped to this value.
    AcceptRemap(String),
    /// Event should be rejected (error returned to sender).
    Reject,
    /// Event should be rejected with HTTP 401 (origin not allowed).
    RejectUnauthorized,
    /// Event should be silently dropped.
    Ignore,
}

/// Instrumentation key validator.
///
/// Wraps the concrete backend so additional validation strategies can be
/// introduced without changing call sites.
#[derive(Debug, Clone)]
pub(crate) struct IkeyValidator {
    kind: ValidatorKind,
}

/// The concrete validation backend.
#[derive(Debug, Clone)]
enum ValidatorKind {
    /// Rules loaded from YAML (inline config or external file).
    Rules(CompiledRulesFilter),
}

impl IkeyValidator {
    /// Create a validator from compiled rules.
    pub fn from_rules(filter: CompiledRulesFilter) -> Self {
        Self {
            kind: ValidatorKind::Rules(filter),
        }
    }

    /// Check an event's instrumentation key, resolved name, and request origin.
    ///
    /// `ikey` is the event's `"iKey"` or the `apikey` header, if present.
    /// A missing `event_name` skips event-name filtering. `origin` is the
    /// caller-resolved request origin; this module does not parse HTTP headers.
    pub fn check(
        &self,
        ikey: Option<&str>,
        event_name: Option<&str>,
        origin: Option<&str>,
    ) -> FilterVerdict {
        match &self.kind {
            ValidatorKind::Rules(rules) => rules.check(ikey, event_name, origin),
        }
    }
}

// ---------------------------------------------------------------------------
// Config types (deserialized from YAML)
// ---------------------------------------------------------------------------

/// Configuration source for the iKey filter on an endpoint.
///
/// Supports inline rules or a reference to an external YAML file.
/// When both `keys` and `filter_file` are present, `filter_file`
/// takes precedence.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IkeyFilterSource {
    /// Path to an external YAML file containing the `keys` mapping.
    /// Relative paths are resolved against the pipeline config directory.
    #[serde(default)]
    pub filter_file: Option<String>,

    /// Inline rules - same schema as the external file's root mapping.
    #[serde(default, deserialize_with = "deserialize_ordered_map")]
    pub keys: Vec<(String, IkeyRule)>,
}

/// Root schema of an external iKey filter file.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IkeyFilterFileConfig {
    /// Ordered map of iKey patterns to rules.
    #[serde(default, deserialize_with = "deserialize_ordered_map")]
    pub keys: Vec<(String, IkeyRule)>,
}

/// Rule for a single iKey pattern.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct IkeyRule {
    /// What to do when the key matches.
    pub action: IkeyAction,

    /// Optional event-name allow-list. Only meaningful when `action`
    /// is `accept`. Patterns support a trailing `*` glob.
    #[serde(default)]
    pub allowed_events: Vec<String>,

    /// When `true`, the HTTP `Origin` header must be present and must
    /// match one of `allowed_origins`. Only meaningful when `action`
    /// is `accept`.
    #[serde(default)]
    pub require_origin: bool,

    /// Allowed `Origin` header values. Use `"*"` to allow any origin.
    /// Defaults to `["*"]`. Only checked when `require_origin` is
    /// `true`.
    #[serde(default = "default_rule_allowed_origins")]
    pub allowed_origins: Vec<String>,

    /// Remap the iKey to a different value before further processing.
    /// The remapped key is not re-evaluated against the filter rules.
    /// Only meaningful when `action` is `accept`.
    #[serde(default)]
    pub remap_ikey: Option<String>,
}

/// Action to take for a matched iKey.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum IkeyAction {
    /// Accept the event (subject to `allowed_events` if configured).
    Accept,
    /// Reject the event - the sender receives an error response.
    Reject,
    /// Silently drop the event - no error is returned.
    Ignore,
}

// ---------------------------------------------------------------------------
// Compiled rules filter
// ---------------------------------------------------------------------------

/// Compiled rules-based iKey filter ready for per-event lookups.
#[derive(Debug, Clone)]
pub(crate) struct CompiledRulesFilter {
    rules: Vec<CompiledRule>,
}

#[derive(Debug, Clone)]
struct CompiledRule {
    pattern: IkeyPattern,
    action: IkeyAction,
    allowed_events: Vec<EventPattern>,
    require_origin: bool,
    allowed_origins: Vec<OriginPattern>,
    remap_ikey: Option<String>,
}

#[derive(Debug, Clone)]
enum IkeyPattern {
    Wildcard,
    Exact(String),
}

#[derive(Debug, Clone)]
enum EventPattern {
    Wildcard,
    Prefix(String),
    Exact(String),
}

#[derive(Debug, Clone)]
enum OriginPattern {
    /// Matches any supplied origin.
    Wildcard,
    /// Case-insensitive exact match.
    Exact(String),
}

impl OriginPattern {
    fn compile(pattern: &str) -> Self {
        if pattern == "*" {
            Self::Wildcard
        } else {
            Self::Exact(pattern.to_owned())
        }
    }

    fn matches(&self, origin: &str) -> bool {
        match self {
            Self::Wildcard => true,
            Self::Exact(expected) => origin.eq_ignore_ascii_case(expected),
        }
    }
}

impl EventPattern {
    fn compile(pattern: &str) -> Self {
        if pattern == "*" {
            Self::Wildcard
        } else if let Some(prefix) = pattern.strip_suffix('*') {
            Self::Prefix(prefix.to_owned())
        } else {
            Self::Exact(pattern.to_owned())
        }
    }

    fn matches(&self, event_name: &str) -> bool {
        match self {
            Self::Wildcard => true,
            Self::Prefix(prefix) => event_name.starts_with(prefix.as_str()),
            Self::Exact(expected) => event_name == expected.as_str(),
        }
    }
}

impl CompiledRulesFilter {
    /// Compile from an ordered list of `(pattern, rule)` pairs.
    pub fn compile(keys: &[(String, IkeyRule)]) -> Self {
        let rules = keys
            .iter()
            .map(|(pattern, rule)| {
                let pattern = if pattern == "*" {
                    IkeyPattern::Wildcard
                } else {
                    IkeyPattern::Exact(pattern.clone())
                };
                let allowed_events = rule
                    .allowed_events
                    .iter()
                    .map(|pattern| EventPattern::compile(pattern))
                    .collect();
                let allowed_origins = rule
                    .allowed_origins
                    .iter()
                    .map(|pattern| OriginPattern::compile(pattern))
                    .collect();
                CompiledRule {
                    pattern,
                    action: rule.action,
                    allowed_events,
                    require_origin: rule.require_origin,
                    allowed_origins,
                    remap_ikey: rule.remap_ikey.clone(),
                }
            })
            .collect();
        Self { rules }
    }

    fn check(
        &self,
        ikey: Option<&str>,
        event_name: Option<&str>,
        origin: Option<&str>,
    ) -> FilterVerdict {
        for rule in &self.rules {
            let key_matches = match &rule.pattern {
                IkeyPattern::Wildcard => true,
                IkeyPattern::Exact(expected) => ikey == Some(expected.as_str()),
            };
            if !key_matches {
                continue;
            }

            return match rule.action {
                IkeyAction::Reject => FilterVerdict::Reject,
                IkeyAction::Ignore => FilterVerdict::Ignore,
                IkeyAction::Accept => {
                    if rule.require_origin {
                        match origin {
                            None => return FilterVerdict::RejectUnauthorized,
                            Some(origin) => {
                                if !rule.allowed_origins.iter().any(|p| p.matches(origin)) {
                                    return FilterVerdict::RejectUnauthorized;
                                }
                            }
                        }
                    }

                    if !rule.allowed_events.is_empty()
                        && let Some(name) = event_name
                        && !rule.allowed_events.iter().any(|p| p.matches(name))
                    {
                        return FilterVerdict::Reject;
                    }

                    if let Some(ref new_key) = rule.remap_ikey {
                        FilterVerdict::AcceptRemap(new_key.clone())
                    } else {
                        FilterVerdict::Accept
                    }
                }
            };
        }

        // No rule matched - accept by default (open policy).
        FilterVerdict::Accept
    }
}

// ---------------------------------------------------------------------------
// Compilation from config sources
// ---------------------------------------------------------------------------

/// Compile an [`IkeyFilterSource`] into an [`IkeyValidator`].
///
/// An external file takes precedence over inline rules. When `config_dir` is
/// supplied, file paths must be relative and resolve within that directory.
/// Without it, paths are used as supplied, relative to the working directory.
///
/// Returns `(None, None)` when neither rules nor a file are configured.
/// Otherwise returns a validator and the resolved external file path, if any.
/// File resolution, read, and parse errors are returned to the caller.
pub(crate) fn compile_source(
    source: &IkeyFilterSource,
    config_dir: Option<&Path>,
) -> Result<(Option<IkeyValidator>, Option<PathBuf>), String> {
    if let Some(ref file_path) = source.filter_file {
        let resolved = match config_dir {
            Some(dir) => resolve_filter_path(file_path, dir)?,
            None => PathBuf::from(file_path),
        };
        let config = load_filter_file(&resolved)?;
        let filter = CompiledRulesFilter::compile(&config.keys);
        return Ok((Some(IkeyValidator::from_rules(filter)), Some(resolved)));
    }

    if !source.keys.is_empty() {
        let filter = CompiledRulesFilter::compile(&source.keys);
        return Ok((Some(IkeyValidator::from_rules(filter)), None));
    }

    Ok((None, None))
}

// The source receiver uses an engine-level resolver that is not yet ported.
// Keep the same confinement at this file-loading boundary.
fn resolve_filter_path(file_path: &str, config_dir: &Path) -> Result<PathBuf, String> {
    let requested = Path::new(file_path);
    if requested.components().any(|component| {
        matches!(
            component,
            Component::Prefix(_) | Component::RootDir | Component::ParentDir
        )
    }) {
        return Err(format!(
            "iKey filter path '{file_path}' must be relative to the config directory without '..' components"
        ));
    }

    let base = config_dir.canonicalize().map_err(|error| {
        format!(
            "failed to resolve config directory {}: {error}",
            config_dir.display()
        )
    })?;
    let resolved = base
        .join(requested)
        .canonicalize()
        .map_err(|error| format!("failed to resolve iKey filter file '{file_path}': {error}"))?;
    if !resolved.starts_with(&base) {
        return Err(format!(
            "iKey filter path '{file_path}' escapes config directory {}",
            config_dir.display()
        ));
    }
    Ok(resolved)
}

/// Load and parse an external iKey filter YAML file.
pub(crate) fn load_filter_file(path: &Path) -> Result<IkeyFilterFileConfig, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|error| format!("failed to read {}: {error}", path.display()))?;
    serde_yaml::from_str::<IkeyFilterFileConfig>(&content)
        .map_err(|error| format!("failed to parse {}: {error}", path.display()))
}

// ---------------------------------------------------------------------------
// Serde helpers
// ---------------------------------------------------------------------------

fn default_rule_allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

fn deserialize_ordered_map<'de, D>(deserializer: D) -> Result<Vec<(String, IkeyRule)>, D::Error>
where
    D: Deserializer<'de>,
{
    struct OrderedMapVisitor;

    impl<'de> Visitor<'de> for OrderedMapVisitor {
        type Value = Vec<(String, IkeyRule)>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a mapping of iKey patterns to filter rules")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut result = Vec::with_capacity(map.size_hint().unwrap_or(0));
            let mut seen = HashSet::new();
            while let Some((pattern, rule)) = map.next_entry::<serde_yaml::Value, IkeyRule>()? {
                let pattern = pattern
                    .as_str()
                    .ok_or_else(|| serde::de::Error::custom("iKey filter key must be a string"))?
                    .to_owned();
                if !seen.insert(pattern.clone()) {
                    return Err(serde::de::Error::custom(format!(
                        "duplicate iKey filter key pattern '{pattern}'"
                    )));
                }
                result.push((pattern, rule));
            }
            Ok(result)
        }
    }

    deserializer.deserialize_map(OrderedMapVisitor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compile_yaml(yaml: &str) -> IkeyValidator {
        let source: IkeyFilterSource =
            serde_yaml::from_str(yaml).expect("failed to parse test YAML");
        let (validator, path) = compile_source(&source, None).expect("compile failed");
        assert!(path.is_none());
        validator.expect("expected a validator")
    }

    /// Scenario: inline iKey rules contain specific and catch-all patterns.
    /// Guarantees: deserialization preserves declaration order and every rule setting.
    #[test]
    fn inline_rules_preserve_complete_configuration() {
        let source: IkeyFilterSource = serde_yaml::from_str(
            r#"
keys:
  "o:accepted":
    action: accept
    allowed_events:
      - "Microsoft.*"
    require_origin: true
    allowed_origins:
      - "https://example.com"
    remap_ikey: "o:replacement"
  "*":
    action: reject
"#,
        )
        .expect("iKey filter configuration");

        assert_eq!(source.keys.len(), 2);
        assert_eq!(source.keys[0].0, "o:accepted");
        assert_eq!(source.keys[0].1.action, IkeyAction::Accept);
        assert_eq!(source.keys[0].1.allowed_events, ["Microsoft.*"]);
        assert!(source.keys[0].1.require_origin);
        assert_eq!(source.keys[0].1.allowed_origins, ["https://example.com"]);
        assert_eq!(
            source.keys[0].1.remap_ikey.as_deref(),
            Some("o:replacement")
        );
        assert_eq!(source.keys[1].0, "*");
        assert_eq!(source.keys[1].1.action, IkeyAction::Reject);
    }

    /// Scenario: catch-all rules accept, reject, or ignore events with and without keys.
    /// Guarantees: each configured action produces its distinct verdict even for a missing key.
    #[test]
    fn wildcard_actions_include_missing_keys() {
        for (action, expected) in [
            ("accept", FilterVerdict::Accept),
            ("reject", FilterVerdict::Reject),
            ("ignore", FilterVerdict::Ignore),
        ] {
            let validator = compile_yaml(&format!("keys:\n  '*': {{ action: {action} }}"));
            for key in [Some("o:any"), None] {
                assert_eq!(validator.check(key, Some("Ev"), None), expected);
            }
        }
    }

    /// Scenario: exact reject and ignore rules precede a catch-all accept rule.
    /// Guarantees: keys match case-sensitively and a missing key only matches the catch-all.
    #[test]
    fn exact_keys_are_case_sensitive() {
        let validator = compile_yaml(
            r#"
keys:
  "o:bad": { action: reject }
  "o:silent": { action: ignore }
  "o:prefix*": { action: reject }
  "*": { action: accept }
"#,
        );
        for (key, expected) in [
            (Some("o:bad"), FilterVerdict::Reject),
            (Some("o:silent"), FilterVerdict::Ignore),
            (Some("O:BAD"), FilterVerdict::Accept),
            (Some("o:prefix-more"), FilterVerdict::Accept),
            (Some("o:prefix*"), FilterVerdict::Reject),
            (None, FilterVerdict::Accept),
        ] {
            assert_eq!(validator.check(key, Some("Ev"), None), expected);
        }
    }

    /// Scenario: an exact accept rule with event restrictions precedes a catch-all reject.
    /// Guarantees: YAML declaration order, not lexical key order, determines the first match.
    #[test]
    fn first_match_wins() {
        let validator = compile_yaml(
            r#"
keys:
  "o:specific":
    action: accept
    allowed_events: ["Allowed"]
  "*": { action: reject }
"#,
        );
        assert_eq!(
            validator.check(Some("o:specific"), Some("Allowed"), None),
            FilterVerdict::Accept
        );
        assert_eq!(
            validator.check(Some("o:specific"), Some("Other"), None),
            FilterVerdict::Reject
        );
        assert_eq!(
            validator.check(Some("o:other"), Some("Allowed"), None),
            FilterVerdict::Reject
        );
        let catch_all_first = compile_yaml(
            r#"
keys:
  "*": { action: ignore }
  "o:specific": { action: reject }
"#,
        );
        assert_eq!(
            catch_all_first.check(Some("o:specific"), None, None),
            FilterVerdict::Ignore
        );
    }

    /// Scenario: no configured rule matches the supplied or missing key.
    /// Guarantees: unmatched events retain the source receiver's default open policy.
    #[test]
    fn unmatched_keys_are_accepted() {
        let validator = compile_yaml("keys:\n  'o:blocked': { action: reject }");
        for key in [Some("o:other"), None] {
            assert_eq!(validator.check(key, None, None), FilterVerdict::Accept);
        }
    }

    /// Scenario: no file or inline rules are configured.
    /// Guarantees: the default and an explicitly empty rules map produce no validator or path.
    #[test]
    fn no_filter_returns_no_validator() {
        for source in [
            IkeyFilterSource::default(),
            serde_yaml::from_str("keys: {}").expect("empty rules"),
        ] {
            let (validator, path) = compile_source(&source, None).expect("empty filter");
            assert!(validator.is_none());
            assert!(path.is_none());
        }
    }

    /// Scenario: an accept rule allows exact and prefix event names.
    /// Guarantees: nonmatching names are rejected, matching is case-sensitive, and absent names skip checking.
    #[test]
    fn allowed_events_exact_and_prefix() {
        let validator = compile_yaml(
            r#"
keys:
  "o:restricted":
    action: accept
    allowed_events: ["Ms.Web.PageView", "Ms.Web.ClientError", "Ms.Device.*"]
"#,
        );
        for (name, expected) in [
            (Some("Ms.Web.PageView"), FilterVerdict::Accept),
            (Some("Ms.Web.ClientError"), FilterVerdict::Accept),
            (Some("Ms.Device.Info"), FilterVerdict::Accept),
            (Some("Ms.Device.Network.Status"), FilterVerdict::Accept),
            (Some("Ms.Device."), FilterVerdict::Accept),
            (Some("Ms.Web.Other"), FilterVerdict::Reject),
            (Some("ms.web.pageview"), FilterVerdict::Reject),
            (Some("Ms.Web.PageViewExtra"), FilterVerdict::Reject),
            (None, FilterVerdict::Accept),
        ] {
            assert_eq!(validator.check(Some("o:restricted"), name, None), expected);
        }
    }

    /// Scenario: event allow-lists are empty or contain a wildcard.
    /// Guarantees: both forms allow any event name, including an empty name.
    #[test]
    fn unrestricted_event_names() {
        for events in ["[]", "['*']"] {
            let validator = compile_yaml(&format!(
                "keys:\n  '*':\n    action: accept\n    allowed_events: {events}"
            ));
            for name in [Some("Anything"), Some(""), None] {
                assert_eq!(validator.check(None, name, None), FilterVerdict::Accept);
            }
        }
    }

    /// Scenario: event patterns contain exact names, trailing stars, or an interior star.
    /// Guarantees: only a trailing star is a glob and exact names never match longer or shorter names.
    #[test]
    fn event_pattern_matching() {
        for (pattern, name, expected) in [
            ("*", "", true),
            ("*", "Anything", true),
            ("Ms.Device.*", "Ms.Device.Info", true),
            ("Ms.Device.*", "Ms.Device.", true),
            ("Ms.Device.*", "Ms.Web.PageView", false),
            ("Ms.Web.PageView", "Ms.Web.PageView", true),
            ("Ms.Web.PageView", "Ms.Web.PageViewExtra", false),
            ("Ms.Web.PageView", "Ms.Web.PageVie", false),
            ("Ms.*.Info", "Ms.Device.Info", false),
            ("Ms.*.Info", "Ms.*.Info", true),
        ] {
            assert_eq!(EventPattern::compile(pattern).matches(name), expected);
        }
    }

    /// Scenario: an accept rule requires one of several configured origins.
    /// Guarantees: matching is case-insensitive, and absent, unrelated, or prefix-only origins are unauthorized.
    #[test]
    fn required_origins_match_case_insensitively() {
        let validator = compile_yaml(
            r#"
keys:
  "o:key":
    action: accept
    require_origin: true
    allowed_origins: ["https://Example.COM", "https://b.com", "https://c.com"]
"#,
        );
        for (origin, expected) in [
            (Some("HTTPS://EXAMPLE.COM"), FilterVerdict::Accept),
            (Some("https://b.com"), FilterVerdict::Accept),
            (Some("https://d.com"), FilterVerdict::RejectUnauthorized),
            (
                Some("https://example.com.evil"),
                FilterVerdict::RejectUnauthorized,
            ),
            (None, FilterVerdict::RejectUnauthorized),
        ] {
            assert_eq!(validator.check(Some("o:key"), Some("Ev"), origin), expected);
        }
    }

    /// Scenario: an accept rule requires an origin but omits the allowed-origins setting.
    /// Guarantees: the default wildcard allows supplied origins but still rejects an absent origin.
    #[test]
    fn default_origin_wildcard_requires_presence() {
        let validator = compile_yaml("keys:\n  '*':\n    action: accept\n    require_origin: true");
        assert_eq!(
            validator.check(None, None, Some("https://anything.com")),
            FilterVerdict::Accept
        );
        assert_eq!(
            validator.check(None, None, None),
            FilterVerdict::RejectUnauthorized
        );
    }

    /// Scenario: an origin-required rule has an explicitly empty allowed-origins list.
    /// Guarantees: no supplied or missing origin is authorized.
    #[test]
    fn empty_allowed_origins_rejects_all() {
        let validator = compile_yaml(
            "keys:\n  '*':\n    action: accept\n    require_origin: true\n    allowed_origins: []",
        );
        for origin in [None, Some("https://example.com")] {
            assert_eq!(
                validator.check(None, None, origin),
                FilterVerdict::RejectUnauthorized
            );
        }
    }

    /// Scenario: allowed origins are configured without requiring an origin.
    /// Guarantees: the origin list is ignored unless require_origin is true.
    #[test]
    fn optional_origin_is_not_checked() {
        let validator = compile_yaml("keys:\n  '*':\n    action: accept\n    allowed_origins: []");
        for origin in [None, Some("https://example.com")] {
            assert_eq!(validator.check(None, None, origin), FilterVerdict::Accept);
        }
    }

    /// Scenario: a remapping accept rule restricts both origins and event names.
    /// Guarantees: origin failure wins over event failure, and remapping occurs only after both checks pass.
    #[test]
    fn origin_and_event_checks_precede_remapping() {
        let validator = compile_yaml(
            r#"
keys:
  "o:old":
    action: accept
    require_origin: true
    allowed_origins: ["https://app.com"]
    allowed_events: ["Ms.Web.*"]
    remap_ikey: "o:redirected"
  "o:redirected": { action: reject }
"#,
        );
        for (name, origin, expected) in [
            (
                Some("Ms.Web.PageView"),
                Some("https://app.com"),
                FilterVerdict::AcceptRemap("o:redirected".to_owned()),
            ),
            (
                Some("Other"),
                Some("https://app.com"),
                FilterVerdict::Reject,
            ),
            (
                Some("Other"),
                Some("https://evil.com"),
                FilterVerdict::RejectUnauthorized,
            ),
            (
                Some("Ms.Web.PageView"),
                None,
                FilterVerdict::RejectUnauthorized,
            ),
        ] {
            assert_eq!(validator.check(Some("o:old"), name, origin), expected);
        }
    }

    /// Scenario: an accept rule remaps a key that would itself match a reject rule.
    /// Guarantees: remapped keys are returned to the caller without re-evaluating the rules.
    #[test]
    fn remapped_keys_are_not_rechecked() {
        let validator = compile_yaml(
            r#"
keys:
  "o:old": { action: accept, remap_ikey: "o:new" }
  "o:new": { action: reject }
"#,
        );
        assert_eq!(
            validator.check(Some("o:old"), None, None),
            FilterVerdict::AcceptRemap("o:new".to_owned())
        );
        assert_eq!(
            validator.check(Some("o:new"), None, None),
            FilterVerdict::Reject
        );
    }

    /// Scenario: reject and ignore rules also configure origins, event names, and a remapping.
    /// Guarantees: accept-only settings do not alter reject or ignore verdicts.
    #[test]
    fn non_accept_actions_ignore_accept_settings() {
        for (action, expected) in [
            ("reject", FilterVerdict::Reject),
            ("ignore", FilterVerdict::Ignore),
        ] {
            let validator = compile_yaml(&format!(
                "keys:\n  '*':\n    action: {action}\n    require_origin: true\n    allowed_events: ['Allowed']\n    remap_ikey: 'o:new'"
            ));
            assert_eq!(validator.check(None, Some("Other"), None), expected);
        }
    }

    /// Scenario: inline and external rules are both present and a config directory is supplied.
    /// Guarantees: the external rules win, retain YAML order, and return the canonical file path.
    #[test]
    fn external_file_takes_precedence() {
        let dir = tempfile::tempdir().expect("config directory");
        let path = dir.path().join("rules.yaml");
        std::fs::write(
            &path,
            "keys:\n  'o:specific': { action: accept }\n  '*': { action: reject }",
        )
        .expect("write filter");
        let source: IkeyFilterSource =
            serde_yaml::from_str("filter_file: rules.yaml\nkeys:\n  '*': { action: ignore }")
                .expect("source");
        let (validator, resolved) =
            compile_source(&source, Some(dir.path())).expect("compile file");
        assert_eq!(resolved, Some(path.canonicalize().expect("canonical path")));
        let validator = validator.expect("external validator");
        assert_eq!(
            validator.check(Some("o:specific"), None, None),
            FilterVerdict::Accept
        );
        assert_eq!(validator.check(None, None, None), FilterVerdict::Reject);
    }

    /// Scenario: an explicit external file path is supplied without a config directory.
    /// Guarantees: the caller-supplied path is loaded and an empty file rule map still yields a validator.
    #[test]
    fn external_file_without_config_directory() {
        let dir = tempfile::tempdir().expect("config directory");
        let path = dir.path().join("rules.yaml");
        std::fs::write(&path, "keys: {}").expect("write filter");
        let source = IkeyFilterSource {
            filter_file: Some(path.to_str().expect("UTF-8 temp path").to_owned()),
            ..Default::default()
        };
        let (validator, resolved) = compile_source(&source, None).expect("compile file");
        assert_eq!(resolved, Some(path));
        assert_eq!(
            validator.expect("validator").check(None, None, None),
            FilterVerdict::Accept
        );
    }

    /// Scenario: a configured external filter is missing or malformed alongside valid inline rules.
    /// Guarantees: file errors contain the path and never fall back to the inline rules.
    #[test]
    fn external_file_errors_are_propagated() {
        let dir = tempfile::tempdir().expect("config directory");
        let path = dir.path().join("rules.yaml");
        let mut source: IkeyFilterSource =
            serde_yaml::from_str("keys:\n  '*': { action: accept }").expect("inline rules");
        source.filter_file = Some(path.to_str().expect("UTF-8 temp path").to_owned());
        let error = compile_source(&source, None).expect_err("missing file");
        assert!(error.contains("failed to read"), "{error}");
        assert!(error.contains("rules.yaml"), "{error}");
        for content in [
            "keys: [",
            "unknown: true",
            "keys:\n  '*': { action: invalid }",
            "keys:\n  '*': { action: accept, unknown: true }",
        ] {
            std::fs::write(&path, content).expect("write invalid filter");
            let error = compile_source(&source, None).expect_err("invalid file");
            assert!(error.contains("failed to parse"), "{error}");
            assert!(error.contains("rules.yaml"), "{error}");
        }
    }

    /// Scenario: an external path is absolute, contains parent traversal, or is missing.
    /// Guarantees: configuration-relative loading rejects invalid paths with explicit errors.
    #[test]
    fn external_paths_are_confined() {
        let dir = tempfile::tempdir().expect("config directory");
        let absolute = dir.path().join("rules.yaml");
        std::fs::write(&absolute, "keys: {}").expect("write filter");
        for path in [absolute.to_str().expect("UTF-8 temp path"), "../rules.yaml"] {
            let error = resolve_filter_path(path, dir.path()).expect_err("invalid path");
            assert!(error.contains("must be relative"), "{error}");
        }
        let error = resolve_filter_path("missing.yaml", dir.path()).expect_err("missing file");
        assert!(error.contains("failed to resolve"), "{error}");
    }

    /// Scenario: a relative path passes through a directory link outside the config directory.
    /// Guarantees: canonicalization prevents a symlink or junction from bypassing confinement.
    #[test]
    fn linked_external_paths_are_confined() {
        let config = tempfile::tempdir().expect("config directory");
        let outside = tempfile::tempdir().expect("outside directory");
        std::fs::write(outside.path().join("rules.yaml"), "keys: {}").expect("write filter");
        let link = config.path().join("linked");
        #[cfg(unix)]
        std::os::unix::fs::symlink(outside.path(), &link).expect("directory symlink");
        #[cfg(windows)]
        {
            let output = std::process::Command::new("cmd")
                .args(["/C", "mklink", "/J"])
                .arg(&link)
                .arg(outside.path())
                .output()
                .expect("create junction");
            assert!(output.status.success(), "{output:?}");
        }
        let relative = Path::new("linked").join("rules.yaml");
        let error = resolve_filter_path(
            relative.to_str().expect("UTF-8 relative path"),
            config.path(),
        )
        .expect_err("link escapes config directory");
        assert!(error.contains("escapes config directory"), "{error}");
        #[cfg(windows)]
        std::fs::remove_dir(link).expect("remove junction");
    }

    /// Scenario: inline rules repeat an exact key or catch-all with conflicting actions.
    /// Guarantees: duplicate patterns fail parsing with the offending key instead of hiding later rules.
    #[test]
    fn duplicate_inline_patterns_are_rejected() {
        for pattern in ["o:key", "*"] {
            let yaml = format!(
                "keys:\n  '{pattern}': {{ action: accept }}\n  '{pattern}': {{ action: reject }}"
            );
            let error = serde_yaml::from_str::<IkeyFilterSource>(&yaml)
                .expect_err("duplicate pattern must be rejected")
                .to_string();
            assert!(
                error.contains(&format!("duplicate iKey filter key pattern '{pattern}'")),
                "{error}"
            );
        }
    }

    /// Scenario: an external filter repeats a key while the source also supplies valid inline rules.
    /// Guarantees: compilation returns a path-qualified duplicate error without falling back to inline rules.
    #[test]
    fn duplicate_external_patterns_are_rejected() {
        let dir = tempfile::tempdir().expect("config directory");
        let path = dir.path().join("rules.yaml");
        std::fs::write(
            &path,
            "keys:\n  'o:key': { action: accept }\n  'o:key': { action: reject }",
        )
        .expect("write duplicate rules");
        let source: IkeyFilterSource =
            serde_yaml::from_str("filter_file: rules.yaml\nkeys:\n  '*': { action: accept }")
                .expect("source config");
        let error =
            compile_source(&source, Some(dir.path())).expect_err("duplicate external pattern");
        assert!(error.contains("failed to parse"), "{error}");
        assert!(error.contains("rules.yaml"), "{error}");
        assert!(
            error.contains("duplicate iKey filter key pattern 'o:key'"),
            "{error}"
        );
    }

    /// Scenario: exact key patterns differ only by case.
    /// Guarantees: duplicate detection preserves case-sensitive matching and declaration order.
    #[test]
    fn differently_cased_patterns_remain_distinct() {
        let validator =
            compile_yaml("keys:\n  'o:key': { action: accept }\n  'O:KEY': { action: reject }");
        assert_eq!(
            validator.check(Some("o:key"), None, None),
            FilterVerdict::Accept
        );
        assert_eq!(
            validator.check(Some("O:KEY"), None, None),
            FilterVerdict::Reject
        );
    }

    /// Scenario: inline configuration contains unknown fields, invalid actions, or non-string keys.
    /// Guarantees: schema errors are rejected rather than silently changing the filter policy.
    #[test]
    fn invalid_inline_rules_are_rejected() {
        for yaml in [
            "unknown: true",
            "keys:\n  '*': { action: invalid }",
            "keys:\n  '*': { action: accept, unknown: true }",
            "keys:\n  123: { action: accept }",
        ] {
            assert!(
                serde_yaml::from_str::<IkeyFilterSource>(yaml).is_err(),
                "{yaml}"
            );
        }
    }
}
