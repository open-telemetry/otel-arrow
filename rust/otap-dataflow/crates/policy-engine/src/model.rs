// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Data model for `opentelemetry.proto.policies.v1development` (`open-telemetry/opentelemetry-proto#865`).

// Crate-internal helpers are exercised by unit tests until the evaluation engine is implemented.
#![allow(dead_code)]

use std::cmp::Ordering;

use regex_automata::meta::{Config as MetaConfig, Regex as MetaRegex};
use regex_automata::nfa::thompson::WhichCaptures;
use regex_automata::util::iter::Searcher;
use regex_automata::{Input, PatternID};
use regex_syntax::ast::{
    self, AssertionKind, Ast, ClassBracketed, ClassPerl, ClassPerlKind, ClassSet, ClassSetItem,
    ClassSetRange, ClassSetUnion, ClassUnicode, ClassUnicodeKind, Flag, Flags, FlagsItem,
    FlagsItemKind, Group, GroupKind, HexLiteralKind, Literal, LiteralKind, Span,
    SpecialLiteralKind,
};
use regex_syntax::hir::Hir;
use regex_syntax::hir::translate::TranslatorBuilder;
use thiserror::Error;

/// Maximum RE2 regex source length accepted per policy pattern (bytes).
pub(crate) const MAX_REGEX_PATTERN_BYTES: usize = 4096;
/// Maximum AST nesting depth accepted per policy pattern.
pub(crate) const MAX_REGEX_NESTING: u32 = 64;
/// Maximum Thompson NFA size per compiled pattern (256 KiB).
pub(crate) const MAX_REGEX_NFA_BYTES: usize = 256 * 1024;
/// Maximum lazy-DFA hybrid cache capacity per compiled pattern (64 KiB).
pub(crate) const MAX_REGEX_HYBRID_CACHE_BYTES: usize = 64 * 1024;

/// Errors returned during policy compilation or evaluation.
#[derive(Debug, Error)]
pub enum PolicyError {
    /// Fatal engine compilation error.
    #[error("policy compilation error: {0}")]
    Compile(String),
    /// Error reading or mutating Arrow / OTAP records.
    #[error("otap evaluation error: {0}")]
    Evaluation(String),
}

/// Per-policy validation or unsupported-feature error collected during policy compilation.
///
/// Per `policy.md` ("Compilation Errors"), offending policies are kept inert while valid
/// policies in the same batch continue to compile and run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyCompileError {
    /// Identifier (`Policy.id`) of the offending policy.
    pub policy_id: String,
    /// Human-readable error message with the location within the policy (for example `matches[0]: ...`).
    pub message: String,
}

impl std::fmt::Display for PolicyCompileError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "policy '{}': {}", self.policy_id, self.message)
    }
}

/// Action taken by a filter or sampling policy (`common.proto`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Action {
    /// Unspecified action (`ACTION_UNSPECIFIED`).
    Unspecified,
    /// Explicitly keep matching telemetry (`ACTION_KEEP`). Overrides `Drop`.
    Keep,
    /// Drop matching telemetry unless exempted by a `Keep` policy (`ACTION_DROP`).
    Drop,
}

/// Scalar value matched by `equals`/`contains` or set by `LogAdd` (`common.proto`).
#[derive(Debug, Clone, PartialEq)]
pub enum ScalarValue {
    /// UTF-8 string value.
    String(String),
    /// 64-bit signed integer value.
    Int(i64),
    /// Boolean value.
    Bool(bool),
    /// 64-bit IEEE-754 floating-point value.
    Double(f64),
    /// Raw byte sequence.
    Bytes(Vec<u8>),
}

/// Numeric value for range predicates (`gt`, `gte`, `lt`, `lte` in `common.proto`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum NumericValue {
    /// 64-bit signed integer threshold.
    Int(i64),
    /// 64-bit floating-point threshold.
    Double(f64),
}

impl NumericValue {
    /// Compares a candidate `i64` field value against this `NumericValue` threshold (`candidate ? threshold`),
    /// preserving exact 64-bit integer precision above `2^53` per `policy.md` ("Typed Values").
    #[inline]
    #[must_use]
    pub(crate) fn cmp_candidate_i64(&self, candidate: i64) -> Option<Ordering> {
        match *self {
            Self::Int(thresh) => Some(candidate.cmp(&thresh)),
            Self::Double(thresh) => cmp_i64_with_f64(candidate, thresh),
        }
    }

    /// Compares a candidate `f64` field value against this `NumericValue` threshold (`candidate ? threshold`),
    /// preserving exact 64-bit integer precision above `2^53` per `policy.md` ("Typed Values").
    #[inline]
    #[must_use]
    pub(crate) fn cmp_candidate_f64(&self, candidate: f64) -> Option<Ordering> {
        if candidate.is_nan() {
            return None;
        }
        match *self {
            Self::Int(thresh) => cmp_i64_with_f64(thresh, candidate).map(Ordering::reverse),
            Self::Double(thresh) => candidate.partial_cmp(&thresh),
        }
    }
}

/// Exact comparison between an `i64` and an `f64` without losing precision on integers > 2^53.
#[inline]
#[must_use]
pub(crate) fn cmp_i64_with_f64(int_val: i64, float_val: f64) -> Option<Ordering> {
    if float_val.is_nan() {
        return None;
    }
    if float_val == f64::INFINITY {
        return Some(Ordering::Less);
    }
    if float_val == f64::NEG_INFINITY {
        return Some(Ordering::Greater);
    }
    // 9223372036854775808.0 == 2^63, which is strictly greater than i64::MAX (2^63 - 1).
    if float_val >= 9_223_372_036_854_775_808.0_f64 {
        return Some(Ordering::Less);
    }
    // -9223372036854775808.0 == -2^63 == i64::MIN.
    if float_val < -9_223_372_036_854_775_808.0_f64 {
        return Some(Ordering::Greater);
    }
    // Within [-2^63, 2^63), truncating float_val to i128 is exact for its integral part.
    let float_trunc = float_val.trunc();
    let float_int = float_trunc as i128;
    let lhs = int_val as i128;
    match lhs.cmp(&float_int) {
        Ordering::Less => Some(Ordering::Less),
        Ordering::Greater => Some(Ordering::Greater),
        Ordering::Equal => {
            let frac = float_val - float_trunc;
            if frac > 0.0 {
                Some(Ordering::Less)
            } else if frac < 0.0 {
                Some(Ordering::Greater)
            } else {
                Some(Ordering::Equal)
            }
        }
    }
}

/// Exact cross-type equality between `i64` and `f64` (`policy.md` "Typed Values").
#[inline]
#[must_use]
pub(crate) fn eq_i64_f64(int_val: i64, float_val: f64) -> bool {
    cmp_i64_with_f64(int_val, float_val) == Some(Ordering::Equal)
}

/// Case-sensitive full-string glob matcher (`*` matches any sequence of characters including none,
/// `?` matches any single Unicode scalar value, and every other character matches only itself)
/// per `policy.md` ("glob") and `common.proto` (`Predicate.glob`).
#[must_use]
pub(crate) fn matches_glob(pattern: &str, text: &str) -> bool {
    let mut p_idx = 0usize;
    let mut t_idx = 0usize;
    let mut star_p: Option<usize> = None;
    let mut star_t = 0usize;

    while let Some(tc) = text[t_idx..].chars().next() {
        if let Some(pc) = pattern[p_idx..].chars().next() {
            if pc == '*' {
                p_idx += 1;
                star_p = Some(p_idx);
                star_t = t_idx;
                continue;
            }
            if pc == '?' || pc == tc {
                p_idx += pc.len_utf8();
                t_idx += tc.len_utf8();
                continue;
            }
        }
        if let Some(sp) = star_p
            && let Some(stc) = text[star_t..].chars().next()
        {
            star_t += stc.len_utf8();
            t_idx = star_t;
            p_idx = sp;
            continue;
        }
        return false;
    }

    while pattern[p_idx..].starts_with('*') {
        p_idx += 1;
    }
    p_idx == pattern.len()
}

/// Unified predicate (`common.proto` `Predicate`).
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    /// Field exists (`google.protobuf.Empty`).
    Exists,
    /// Exact scalar equality (`ScalarValue`).
    Equals(ScalarValue),
    /// Case-sensitive full-string wildcard pattern (`*` and `?`) (`common.proto` `Predicate.glob`).
    Glob(String),
    /// RE2 regular expression (`string`). Case-insensitive matching uses inline `(?i)`.
    Regex(String),
    /// Strictly greater than (`NumericValue`).
    Gt(NumericValue),
    /// Greater than or equal to (`NumericValue`).
    Gte(NumericValue),
    /// Strictly less than (`NumericValue`).
    Lt(NumericValue),
    /// Less than or equal to (`NumericValue`).
    Lte(NumericValue),
    /// Substring on `String`, byte subsequence on `Bytes`, or element membership on `Array`.
    Contains(ScalarValue),
}

impl Predicate {
    /// Evaluates a numeric ordering result (`candidate.cmp(threshold)`).
    #[inline]
    #[must_use]
    pub(crate) fn matches_ordering(&self, ord: Option<Ordering>) -> bool {
        let Some(ord) = ord else {
            return false;
        };
        match self {
            Self::Gt(_) => ord == Ordering::Greater,
            Self::Gte(_) => ord == Ordering::Greater || ord == Ordering::Equal,
            Self::Lt(_) => ord == Ordering::Less,
            Self::Lte(_) => ord == Ordering::Less || ord == Ordering::Equal,
            _ => false,
        }
    }
}

/// Path into flat or nested map attributes (`common.proto`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AttributePath {
    /// Path segments (`path[0]` is the top-level attribute key).
    pub path: Vec<String>,
}

impl AttributePath {
    /// Creates a single-segment attribute path.
    #[must_use]
    pub fn single(key: impl Into<String>) -> Self {
        Self {
            path: vec![key.into()],
        }
    }

    /// Creates a multi-segment nested attribute path.
    #[must_use]
    pub fn nested<I, S>(segments: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            path: segments.into_iter().map(Into::into).collect(),
        }
    }
}

/// Top-level log record fields (`logs.proto` `LogRecordField`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LogRecordField {
    /// Unspecified field.
    Unspecified,
    /// Log body (`logs.body` `StructArray`).
    Body,
    /// Severity text (`logs.severity_text`).
    SeverityText,
    /// Severity number (`logs.severity_number`).
    SeverityNumber,
    /// 16-byte trace ID (`logs.trace_id`, evaluated as 32-char lowercase hex).
    TraceId,
    /// 8-byte span ID (`logs.span_id`, evaluated as 16-char lowercase hex).
    SpanId,
    /// Event name (`logs.event_name`).
    EventName,
}

/// Instrumentation scope fields (`common.proto`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ScopeField {
    /// Unspecified scope field.
    Unspecified,
    /// Scope name (`logs.scope.name`).
    Name,
    /// Scope version (`logs.scope.version`).
    Version,
    /// Scope schema URL (`logs.schema_url`).
    SchemaUrl,
}

/// Field selector for `LogMatcher` (`logs.proto` `LogFieldSelector`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogFieldSelector {
    /// Top-level log record field.
    RecordField(LogRecordField),
    /// Instrumentation scope field.
    ScopeField(ScopeField),
    /// Log record attribute (`LogAttrs`).
    LogAttribute(AttributePath),
    /// Resource attribute (`ResourceAttrs`).
    ResourceAttribute(AttributePath),
    /// Instrumentation scope attribute (`ScopeAttrs`).
    ScopeAttribute(AttributePath),
    /// Nested field inside a map-valued `logs.body` (`body_field`).
    BodyField(AttributePath),
}

/// Target selector for `LogAdd` and `LogRedact` (`logs.proto` `LogTargetSelector`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogTargetSelector {
    /// Writable first-class log record field: `Body` (any `ScalarValue`), `SeverityText` /
    /// `EventName` (`String`), or `SeverityNumber` (`Int` in `1..=24`) for `LogAdd`;
    /// `Body`, `SeverityText`, or `EventName` for `LogRedact`. `TraceId` and `SpanId` are
    /// read-only and rejected at compile time per `logs.proto` and `policy.md`.
    RecordField(LogRecordField),
    /// Log record attribute (`LogAttrs`).
    LogAttribute(AttributePath),
    /// Nested field inside a structured (map) log record body (`body_field`).
    BodyField(AttributePath),
}

/// Attribute or structured body selector for `LogRename` and `LogRemove` (`logs.proto` `LogAttributeSelector`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogAttributeSelector {
    /// Log record attribute (`LogAttrs`).
    LogAttribute(AttributePath),
    /// Nested field inside a structured (map) log record body (`body_field`).
    BodyField(AttributePath),
}

/// Single matcher within a `LogFilterPolicy` or `LogTransformPolicy` (`logs.proto` `LogMatcher`).
#[derive(Debug, Clone, PartialEq)]
pub struct LogMatcher {
    /// Field to inspect.
    pub field: LogFieldSelector,
    /// Predicate to evaluate.
    pub predicate: Predicate,
    /// When true, inverts the boolean result of `predicate`.
    pub negate: bool,
}

/// Atomic log filter policy (`logs.proto` `LogFilterPolicy`).
#[derive(Debug, Clone, PartialEq)]
pub struct LogFilterPolicy {
    /// Conjunction (`AND`) of matchers. Must be non-empty.
    pub matches: Vec<LogMatcher>,
    /// Keep or drop action (`ACTION_KEEP` overrides `ACTION_DROP`).
    pub action: Action,
}

/// Stage 1 transform: rename a log attribute or structured body field (`logs.proto` `LogRename`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRename {
    /// Source attribute or structured body selector.
    pub from: LogAttributeSelector,
    /// Destination attribute or structured body selector.
    pub to: LogAttributeSelector,
    /// When false, does not overwrite an existing `to` field (and leaves `from` intact).
    pub upsert: bool,
}

/// Stage 2 transform: add/set a field/attribute (`logs.proto` `LogAdd`).
#[derive(Debug, Clone, PartialEq)]
pub struct LogAdd {
    /// Target field/attribute to set.
    pub target: LogTargetSelector,
    /// Scalar value to write.
    pub value: ScalarValue,
    /// When false, only writes if `target` does not already exist.
    pub upsert: bool,
}

/// Stage 3 transform: remove a log attribute or structured body field (`logs.proto` `LogRemove`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRemove {
    /// Attribute or structured body field to remove.
    pub target: LogAttributeSelector,
}

/// Stage 4 transform: redact a field/attribute (`logs.proto` `LogRedact`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRedact {
    /// Target field/attribute to redact.
    pub target: LogTargetSelector,
    /// Replacement mask string; defaults to `"[REDACTED]"` when `None`.
    pub replacement: Option<String>,
    /// Optional RE2 regex pattern. When `None`, masks the entire value; when `Some`, replaces
    /// non-overlapping regex matches and expands `$0`, `$1`..`$99`, `${1}`..`${99}`, `${name}`, `$$`.
    pub regex: Option<String>,
}

impl LogRedact {
    /// Returns the effective replacement template (`"[REDACTED]"` when `None`).
    #[inline]
    #[must_use]
    pub fn effective_replacement(&self) -> &str {
        self.replacement.as_deref().unwrap_or("[REDACTED]")
    }
}

/// Atomic log transform policy (`logs.proto` `LogTransformPolicy`).
#[derive(Debug, Clone, PartialEq)]
pub struct LogTransformPolicy {
    /// Conjunction (`AND`) of matchers. Empty `matches` matches all log records (`policy.md` "Log Transform Policy").
    pub matches: Vec<LogMatcher>,
    /// Stage 1: Rename operations.
    pub rename: Vec<LogRename>,
    /// Stage 2: Add operations.
    pub add: Vec<LogAdd>,
    /// Stage 3: Remove operations.
    pub remove: Vec<LogRemove>,
    /// Stage 4: Redact operations.
    pub redact: Vec<LogRedact>,
}

/// Metric filter policy placeholder (`metrics.proto` in `#865`).
#[derive(Debug, Clone, PartialEq)]
pub struct MetricFilterPolicy {
    /// Action (`Keep` or `Drop`).
    pub action: Action,
}

/// Span sampling mode placeholder (`traces.proto` `SamplingMode` in `#865`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SamplingMode {
    /// Unspecified mode.
    Unspecified,
    /// Proportional sampling mode.
    Proportional,
    /// Equalizing sampling mode (spec default).
    Equalizing,
}

/// Span sampling configuration placeholder (`traces.proto` `SamplingConfig` in `#865`).
#[derive(Debug, Clone, PartialEq)]
pub struct SamplingConfig {
    /// Share of matching spans to keep (`0.0..=100.0`).
    pub percentage: Option<f64>,
    /// Sampling mode (defaults to `Equalizing`).
    pub mode: Option<SamplingMode>,
    /// Number of hex digits encoding threshold in tracestate (`1..=14`, defaults to `4`).
    pub sampling_precision: Option<u32>,
}

/// Span sampling policy placeholder (`traces.proto` `SpanSamplingPolicy` in `#865`).
#[derive(Debug, Clone, PartialEq)]
pub struct SpanSamplingPolicy {
    /// Sampling configuration.
    pub sampling: Option<SamplingConfig>,
}

/// Custom or experimental policy payload placeholder (`policies.proto` `Policy.custom_policy` in `#865`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomPolicy {
    /// Protobuf `google.protobuf.Any.type_url` identifying the custom policy schema.
    pub type_url: String,
    /// Raw serialized payload bytes.
    pub value: Vec<u8>,
}

/// Policy definition oneof (`policies.proto` `Policy.definition`).
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDefinition {
    /// Log filter policy.
    LogFilter(LogFilterPolicy),
    /// Log transform policy (boxed to keep `PolicyDefinition` enum size compact).
    LogTransform(Box<LogTransformPolicy>),
    /// Metric filter policy (unsupported by this log-only engine; kept inert at compile time).
    MetricFilter(MetricFilterPolicy),
    /// Span sampling policy (unsupported by this log-only engine; kept inert at compile time).
    SpanSampling(SpanSamplingPolicy),
    /// Custom policy payload (unsupported by this engine; kept inert at compile time).
    CustomPolicy(CustomPolicy),
}

/// Top-level `opentelemetry.proto.policies.v1development.Policy` (`policies.proto` `Policy`).
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    /// Unique identifier used for deterministic stage ordering (`policy.md` "Execution Order").
    pub id: String,
    /// Human-readable short name.
    pub name: String,
    /// Explanation of the policy's purpose (`policies.proto` `Policy.description`).
    pub description: String,
    /// Whether the policy is enabled (`policies.proto` `Policy.enabled`, defaults to `true`).
    pub enabled: bool,
    /// Operator routing/organization labels (`policies.proto` `Policy.labels`, not interpreted).
    pub labels: Vec<(String, ScalarValue)>,
    /// Policy definition variant.
    pub definition: PolicyDefinition,
}

impl Policy {
    /// Helper constructor for an enabled `LogFilterPolicy`.
    #[must_use]
    pub fn log_filter(id: impl Into<String>, matches: Vec<LogMatcher>, action: Action) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            description: String::new(),
            enabled: true,
            labels: Vec::new(),
            definition: PolicyDefinition::LogFilter(LogFilterPolicy { matches, action }),
        }
    }

    /// Helper constructor for an enabled `LogTransformPolicy`.
    #[must_use]
    pub fn log_transform(id: impl Into<String>, transform: LogTransformPolicy) -> Self {
        let id = id.into();
        Self {
            name: id.clone(),
            id,
            description: String::new(),
            enabled: true,
            labels: Vec::new(),
            definition: PolicyDefinition::LogTransform(Box::new(transform)),
        }
    }
}

/// Parses, validates RE2-only syntax, and translates an RE2 pattern directly into a
/// `regex_syntax::hir::Hir` (with ASCII-only `\d`/`\w`/`\s`/`\b`/`\B` and `multi_line(false)`)
/// without printing or double-parsing the rewritten AST.
pub(crate) fn parse_re2_hir(pat: &str) -> Result<Hir, PolicyError> {
    if pat.len() > MAX_REGEX_PATTERN_BYTES {
        return Err(PolicyError::Compile(format!(
            "regex exceeds maximum length of {MAX_REGEX_PATTERN_BYTES} bytes"
        )));
    }
    let mut ast = ast::parse::ParserBuilder::new()
        .nest_limit(MAX_REGEX_NESTING)
        .build()
        .parse(pat)
        .map_err(|e| PolicyError::Compile(format!("invalid regex '{pat}': {e}")))?;
    rewrite_re2_ast(&mut ast)?;
    TranslatorBuilder::new()
        .multi_line(false)
        .build()
        .translate(pat, &ast)
        .map_err(|e| PolicyError::Compile(format!("invalid regex '{pat}': {e}")))
}

/// Compiles a bounded `regex_automata::meta::Regex` for filter/transform matching (no captures)
/// directly from a pre-parsed `Hir`.
pub(crate) fn compile_matcher_regex(pat: &str) -> Result<(Hir, MetaRegex), PolicyError> {
    let hir = parse_re2_hir(pat)?;
    let re = MetaRegex::builder()
        .configure(
            MetaConfig::new()
                .which_captures(WhichCaptures::None)
                .nfa_size_limit(Some(MAX_REGEX_NFA_BYTES))
                .hybrid(true)
                .hybrid_cache_capacity(MAX_REGEX_HYBRID_CACHE_BYTES)
                .dfa(false),
        )
        .build_from_hir(&hir)
        .map_err(|e| PolicyError::Compile(format!("invalid regex '{pat}': {e}")))?;
    Ok((hir, re))
}

/// Compiles a bounded `regex_automata::meta::Regex` for `LogRedact` replacement directly from a
/// pre-parsed `Hir`. Rejects empty `pat` per `policy.md` ("Redact") and omits capture tracking
/// (`WhichCaptures::Implicit`) when `needs_captures` is `false`.
pub(crate) fn compile_redact_regex(
    pat: &str,
    needs_captures: bool,
) -> Result<MetaRegex, PolicyError> {
    if pat.is_empty() {
        return Err(PolicyError::Compile(
            "redact regex must not be empty (leave pattern unset for whole-field redaction)".into(),
        ));
    }
    let hir = parse_re2_hir(pat)?;
    let which = if needs_captures {
        WhichCaptures::All
    } else {
        WhichCaptures::Implicit
    };
    MetaRegex::builder()
        .configure(
            MetaConfig::new()
                .which_captures(which)
                .nfa_size_limit(Some(MAX_REGEX_NFA_BYTES))
                .hybrid(true)
                .hybrid_cache_capacity(MAX_REGEX_HYBRID_CACHE_BYTES)
                .dfa(false),
        )
        .build_from_hir(&hir)
        .map_err(|e| PolicyError::Compile(format!("invalid regex '{pat}': {e}")))
}

fn validate_re2_flags(flags: &Flags) -> Result<(), PolicyError> {
    for item in &flags.items {
        if let FlagsItemKind::Flag(flag) = item.kind
            && matches!(flag, Flag::IgnoreWhitespace | Flag::Unicode | Flag::CRLF)
        {
            return Err(PolicyError::Compile(format!(
                "unsupported non-RE2 regex flag '{flag:?}'"
            )));
        }
    }
    Ok(())
}

fn re2_lit(span: Span, c: char) -> Literal {
    let kind = match c {
        '\t' => LiteralKind::Special(SpecialLiteralKind::Tab),
        '\n' => LiteralKind::Special(SpecialLiteralKind::LineFeed),
        '\x0c' => LiteralKind::Special(SpecialLiteralKind::FormFeed),
        '\r' => LiteralKind::Special(SpecialLiteralKind::CarriageReturn),
        ' ' => LiteralKind::HexFixed(HexLiteralKind::X),
        _ => LiteralKind::Verbatim,
    };
    Literal { span, kind, c }
}

fn re2_range(span: Span, start: char, end: char) -> ClassSetItem {
    ClassSetItem::Range(ClassSetRange {
        span,
        start: re2_lit(span, start),
        end: re2_lit(span, end),
    })
}

fn re2_perl_bracketed(perl: &ClassPerl) -> ClassBracketed {
    let span = perl.span;
    let items = match perl.kind {
        ClassPerlKind::Digit => vec![re2_range(span, '0', '9')],
        ClassPerlKind::Word => vec![
            re2_range(span, '0', '9'),
            re2_range(span, 'A', 'Z'),
            re2_range(span, 'a', 'z'),
            ClassSetItem::Literal(re2_lit(span, '_')),
        ],
        // RE2 defines \s as [\t\n\f\r ] (note: excludes \v / \x0b).
        ClassPerlKind::Space => vec![
            ClassSetItem::Literal(re2_lit(span, '\t')),
            ClassSetItem::Literal(re2_lit(span, '\n')),
            ClassSetItem::Literal(re2_lit(span, '\x0c')),
            ClassSetItem::Literal(re2_lit(span, '\r')),
            ClassSetItem::Literal(re2_lit(span, ' ')),
        ],
    };
    ClassBracketed {
        span,
        negated: perl.negated,
        kind: ClassSet::Item(ClassSetItem::Union(ClassSetUnion { span, items })),
    }
}

fn wrap_negated_flag(span: Span, flag: Flag, inner: Ast) -> Ast {
    Ast::group(Group {
        span,
        kind: GroupKind::NonCapturing(Flags {
            span,
            items: vec![
                FlagsItem {
                    span,
                    kind: FlagsItemKind::Negation,
                },
                FlagsItem {
                    span,
                    kind: FlagsItemKind::Flag(flag),
                },
            ],
        }),
        ast: Box::new(inner),
    })
}

fn validate_re2_literal(lit: &Literal) -> Result<(), PolicyError> {
    if matches!(
        lit.kind,
        LiteralKind::HexFixed(HexLiteralKind::UnicodeShort | HexLiteralKind::UnicodeLong)
            | LiteralKind::HexBrace(HexLiteralKind::UnicodeShort | HexLiteralKind::UnicodeLong)
    ) {
        return Err(PolicyError::Compile(
            "unsupported non-RE2 Unicode hex escape (\\u/\\U); use \\xHH or \\x{H+}".into(),
        ));
    }
    Ok(())
}

/// Sorted allowlist of case-sensitive Unicode group names accepted by RE2 (`\p{Name}` / `\P{Name}`):
/// `"Any"`, 1- and 2-letter General Categories (excluding UTF-16 surrogates `Cs`, which cannot
/// appear in UTF-8 and are not defined in `regex-syntax`), and canonical Unicode Script names.
/// Rejects Rust-only Unicode binary properties (`White_Space`, `Emoji`, `Alphabetic`, etc.),
/// long category names (`Letter`), non-RE2 shorthands (`ASCII`, `Assigned`, `Latn`), and non-canonical case.
pub(crate) const RE2_UNICODE_GROUPS: &[&str] = &[
    "Adlam",
    "Ahom",
    "Anatolian_Hieroglyphs",
    "Any",
    "Arabic",
    "Armenian",
    "Avestan",
    "Balinese",
    "Bamum",
    "Bassa_Vah",
    "Batak",
    "Bengali",
    "Bhaiksuki",
    "Bopomofo",
    "Brahmi",
    "Braille",
    "Buginese",
    "Buhid",
    "C",
    "Canadian_Aboriginal",
    "Carian",
    "Caucasian_Albanian",
    "Cc",
    "Cf",
    "Chakma",
    "Cham",
    "Cherokee",
    "Chorasmian",
    "Co",
    "Common",
    "Coptic",
    "Cuneiform",
    "Cypriot",
    "Cypro_Minoan",
    "Cyrillic",
    "Deseret",
    "Devanagari",
    "Dives_Akuru",
    "Dogra",
    "Duployan",
    "Egyptian_Hieroglyphs",
    "Elbasan",
    "Elymaic",
    "Ethiopic",
    "Garay",
    "Georgian",
    "Glagolitic",
    "Gothic",
    "Grantha",
    "Greek",
    "Gujarati",
    "Gunjala_Gondi",
    "Gurmukhi",
    "Gurung_Khema",
    "Han",
    "Hangul",
    "Hanifi_Rohingya",
    "Hanunoo",
    "Hatran",
    "Hebrew",
    "Hiragana",
    "Imperial_Aramaic",
    "Inherited",
    "Inscriptional_Pahlavi",
    "Inscriptional_Parthian",
    "Javanese",
    "Kaithi",
    "Kannada",
    "Katakana",
    "Kawi",
    "Kayah_Li",
    "Kharoshthi",
    "Khitan_Small_Script",
    "Khmer",
    "Khojki",
    "Khudawadi",
    "Kirat_Rai",
    "L",
    "Lao",
    "Latin",
    "Lepcha",
    "Limbu",
    "Linear_A",
    "Linear_B",
    "Lisu",
    "Ll",
    "Lm",
    "Lo",
    "Lt",
    "Lu",
    "Lycian",
    "Lydian",
    "M",
    "Mahajani",
    "Makasar",
    "Malayalam",
    "Mandaic",
    "Manichaean",
    "Marchen",
    "Masaram_Gondi",
    "Mc",
    "Me",
    "Medefaidrin",
    "Meetei_Mayek",
    "Mende_Kikakui",
    "Meroitic_Cursive",
    "Meroitic_Hieroglyphs",
    "Miao",
    "Mn",
    "Modi",
    "Mongolian",
    "Mro",
    "Multani",
    "Myanmar",
    "N",
    "Nabataean",
    "Nag_Mundari",
    "Nandinagari",
    "Nd",
    "New_Tai_Lue",
    "Newa",
    "Nko",
    "Nl",
    "No",
    "Nushu",
    "Nyiakeng_Puachue_Hmong",
    "Ogham",
    "Ol_Chiki",
    "Ol_Onal",
    "Old_Hungarian",
    "Old_Italic",
    "Old_North_Arabian",
    "Old_Permic",
    "Old_Persian",
    "Old_Sogdian",
    "Old_South_Arabian",
    "Old_Turkic",
    "Old_Uyghur",
    "Oriya",
    "Osage",
    "Osmanya",
    "P",
    "Pahawh_Hmong",
    "Palmyrene",
    "Pau_Cin_Hau",
    "Pc",
    "Pd",
    "Pe",
    "Pf",
    "Phags_Pa",
    "Phoenician",
    "Pi",
    "Po",
    "Ps",
    "Psalter_Pahlavi",
    "Rejang",
    "Runic",
    "S",
    "Samaritan",
    "Saurashtra",
    "Sc",
    "Sharada",
    "Shavian",
    "Siddham",
    "SignWriting",
    "Sinhala",
    "Sk",
    "Sm",
    "So",
    "Sogdian",
    "Sora_Sompeng",
    "Soyombo",
    "Sundanese",
    "Sunuwar",
    "Syloti_Nagri",
    "Syriac",
    "Tagalog",
    "Tagbanwa",
    "Tai_Le",
    "Tai_Tham",
    "Tai_Viet",
    "Takri",
    "Tamil",
    "Tangsa",
    "Tangut",
    "Telugu",
    "Thaana",
    "Thai",
    "Tibetan",
    "Tifinagh",
    "Tirhuta",
    "Todhri",
    "Toto",
    "Tulu_Tigalari",
    "Ugaritic",
    "Vai",
    "Vithkuqi",
    "Wancho",
    "Warang_Citi",
    "Yezidi",
    "Yi",
    "Z",
    "Zanabazar_Square",
    "Zl",
    "Zp",
    "Zs",
];

fn rewrite_re2_unicode_class(class: &mut ClassUnicode) -> Result<(), PolicyError> {
    match &mut class.kind {
        ClassUnicodeKind::OneLetter(c) => {
            if matches!(c, 'C' | 'L' | 'M' | 'N' | 'P' | 'S' | 'Z') {
                Ok(())
            } else {
                Err(PolicyError::Compile(format!(
                    "unsupported non-RE2 Unicode character class '\\p{c}'"
                )))
            }
        }
        ClassUnicodeKind::Named(name) => {
            // RE2 supports leading `^` negation inside braced Unicode classes (`\p{^Name}` == `\P{Name}`,
            // `\P{^Name}` == `\p{Name}`), which `regex-syntax` parses as a literal `"^Name"`.
            let (lookup, flip_negated) = match name.strip_prefix('^') {
                Some(stripped) => (stripped, true),
                None => (name.as_str(), false),
            };
            if RE2_UNICODE_GROUPS.binary_search(&lookup).is_ok() {
                if flip_negated {
                    class.negated = !class.negated;
                    *name = lookup.to_owned();
                }
                Ok(())
            } else {
                Err(PolicyError::Compile(format!(
                    "unsupported non-RE2 Unicode character class '\\p{{{name}}}'"
                )))
            }
        }
        ClassUnicodeKind::NamedValue { .. } => Err(PolicyError::Compile(
            "unsupported non-RE2 Unicode property key=value syntax".into(),
        )),
    }
}

fn rewrite_re2_class_set(set: &mut ClassSet) -> Result<(), PolicyError> {
    match set {
        ClassSet::Item(item) => rewrite_re2_class_item(item),
        ClassSet::BinaryOp(_) => Err(PolicyError::Compile(
            "unsupported non-RE2 character-class set operator (&&, --, ~~)".into(),
        )),
    }
}

fn rewrite_re2_class_item(item: &mut ClassSetItem) -> Result<(), PolicyError> {
    match item {
        ClassSetItem::Perl(perl) => {
            *item = ClassSetItem::Bracketed(Box::new(re2_perl_bracketed(perl)));
            Ok(())
        }
        ClassSetItem::Bracketed(_) => Err(PolicyError::Compile(
            "unsupported non-RE2 nested bracketed character class".into(),
        )),
        ClassSetItem::Union(union) => {
            for child in &mut union.items {
                rewrite_re2_class_item(child)?;
            }
            Ok(())
        }
        ClassSetItem::Literal(lit) => validate_re2_literal(lit),
        ClassSetItem::Range(range) => {
            validate_re2_literal(&range.start)?;
            validate_re2_literal(&range.end)
        }
        ClassSetItem::Unicode(u) => rewrite_re2_unicode_class(u),
        ClassSetItem::Empty(_) | ClassSetItem::Ascii(_) => Ok(()),
    }
}

fn rewrite_re2_ast(ast: &mut Ast) -> Result<(), PolicyError> {
    match ast {
        Ast::Flags(set_flags) => validate_re2_flags(&set_flags.flags),
        Ast::ClassPerl(perl) => {
            // Rewrites \d, \w, \s (and negations) to RE2 ASCII definitions while keeping `(?i)\w`
            // consistent with `(?i)[\w]` and RE2's Unicode case folding.
            *ast = Ast::class_bracketed(re2_perl_bracketed(perl));
            Ok(())
        }
        Ast::Assertion(a) => match &a.kind {
            AssertionKind::WordBoundary | AssertionKind::NotWordBoundary => {
                let span = a.span;
                let inner = Ast::assertion((**a).clone());
                *ast = wrap_negated_flag(span, Flag::Unicode, inner);
                Ok(())
            }
            AssertionKind::StartLine
            | AssertionKind::EndLine
            | AssertionKind::StartText
            | AssertionKind::EndText => Ok(()),
            other => Err(PolicyError::Compile(format!(
                "unsupported non-RE2 regex assertion '{other:?}'"
            ))),
        },
        Ast::ClassBracketed(cb) => rewrite_re2_class_set(&mut cb.kind),
        Ast::Repetition(rep) => rewrite_re2_ast(&mut rep.ast),
        Ast::Group(group) => {
            match &group.kind {
                GroupKind::NonCapturing(flags) => validate_re2_flags(flags)?,
                GroupKind::CaptureName { name, .. } => {
                    if name.name.is_empty()
                        || !name
                            .name
                            .bytes()
                            .all(|b| b.is_ascii_alphanumeric() || b == b'_')
                    {
                        return Err(PolicyError::Compile(format!(
                            "unsupported non-RE2 capture group name '{}'",
                            name.name
                        )));
                    }
                }
                GroupKind::CaptureIndex(_) => {}
            }
            rewrite_re2_ast(&mut group.ast)
        }
        Ast::Alternation(alt) => {
            for sub in &mut alt.asts {
                rewrite_re2_ast(sub)?;
            }
            Ok(())
        }
        Ast::Concat(concat) => {
            for sub in &mut concat.asts {
                rewrite_re2_ast(sub)?;
            }
            Ok(())
        }
        Ast::Literal(lit) => validate_re2_literal(lit),
        Ast::ClassUnicode(u) => rewrite_re2_unicode_class(u),
        Ast::Empty(_) | Ast::Dot(_) => Ok(()),
    }
}

/// Expands RE2 / Rust regex replacement template syntax (`$0`, `$1`..`$99`, `${1}`..`${99}`, `${name}`, `$$`)
/// against a `regex_automata::meta::Regex` match per `policy.md` ("Redact").
///
/// Uses a single-pass `find_iter` fast path when `replacement_template` contains no `$`, and
/// reuses a caller-supplied `Captures` buffer (allocating one only if `None`) when capture
/// expansion is needed.
#[must_use]
pub(crate) fn apply_regex_redaction_with_caps(
    re: &MetaRegex,
    haystack: &str,
    replacement_template: &str,
    caps: Option<&mut regex_automata::util::captures::Captures>,
) -> Option<String> {
    if !replacement_template.contains('$') {
        let mut iter = re.find_iter(haystack);
        let first = iter.next()?;
        let mut out = String::with_capacity(haystack.len());
        out.push_str(&haystack[..first.start()]);
        out.push_str(replacement_template);
        let mut last_end = first.end();
        for m in iter {
            out.push_str(&haystack[last_end..m.start()]);
            out.push_str(replacement_template);
            last_end = m.end();
        }
        out.push_str(&haystack[last_end..]);
        return Some(out);
    }

    let group_info = re.group_info();
    let mut local_caps;
    let caps = match caps {
        Some(c) => c,
        None => {
            local_caps = re.create_captures();
            &mut local_caps
        }
    };
    let mut searcher = Searcher::new(Input::new(haystack));
    let mut out: Option<String> = None;
    let mut last_end = 0;

    while let Some(m0) = searcher.advance(|input| {
        re.search_captures(input, caps);
        Ok(caps.get_match())
    }) {
        let dst = out.get_or_insert_with(|| String::with_capacity(haystack.len()));
        dst.push_str(&haystack[last_end..m0.start()]);
        expand_capture_template(replacement_template, haystack, caps, group_info, dst);
        last_end = m0.end();
    }

    let mut out = out?;
    out.push_str(&haystack[last_end..]);
    Some(out)
}

#[cfg(test)]
#[must_use]
pub(crate) fn apply_regex_redaction(
    re: &MetaRegex,
    haystack: &str,
    replacement_template: &str,
) -> Option<String> {
    apply_regex_redaction_with_caps(re, haystack, replacement_template, None)
}

fn expand_capture_template(
    template: &str,
    haystack: &str,
    caps: &regex_automata::util::captures::Captures,
    group_info: &regex_automata::util::captures::GroupInfo,
    dst: &mut String,
) {
    let bytes = template.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] != b'$' {
            let next_dollar = memchr::memchr(b'$', &bytes[i..])
                .map(|off| i + off)
                .unwrap_or(bytes.len());
            dst.push_str(&template[i..next_dollar]);
            i = next_dollar;
            continue;
        }
        if i + 1 >= bytes.len() {
            dst.push('$');
            break;
        }
        match bytes[i + 1] {
            b'$' => {
                dst.push('$');
                i += 2;
            }
            b'{' => {
                if let Some(close_rel) = memchr::memchr(b'}', &bytes[i + 2..]) {
                    let close = i + 2 + close_rel;
                    let key = &template[i + 2..close];
                    let is_valid_key = !key.is_empty()
                        && key.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_');
                    if is_valid_key {
                        if key.bytes().all(|b| b.is_ascii_digit()) {
                            if let Ok(idx) = key.parse::<usize>()
                                && let Some(span) = caps.get_group(idx)
                            {
                                dst.push_str(&haystack[span.range()]);
                            }
                        } else if let Some(idx) = group_info.to_index(PatternID::ZERO, key)
                            && let Some(span) = caps.get_group(idx)
                        {
                            dst.push_str(&haystack[span.range()]);
                        }
                        i = close + 1;
                    } else {
                        dst.push('$');
                        i += 1;
                    }
                } else {
                    dst.push('$');
                    i += 1;
                }
            }
            b'0'..=b'9' => {
                let d1 = (bytes[i + 1] - b'0') as usize;
                let (idx, consumed) = if i + 2 < bytes.len() && bytes[i + 2].is_ascii_digit() {
                    let d2 = (bytes[i + 2] - b'0') as usize;
                    (d1 * 10 + d2, 3)
                } else {
                    (d1, 2)
                };
                if let Some(span) = caps.get_group(idx) {
                    dst.push_str(&haystack[span.range()]);
                }
                i += consumed;
            }
            _ => {
                dst.push('$');
                i += 1;
            }
        }
    }
}
