// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Data model for `opentelemetry.proto.policies.v1development` (`open-telemetry/opentelemetry-proto#865`).

use std::cmp::Ordering;
use thiserror::Error;

/// Errors returned during policy compilation or evaluation.
#[derive(Debug, Error)]
pub enum PolicyError {
    /// Invalid regex or policy definition.
    #[error("policy compilation error: {0}")]
    Compile(String),
    /// Error reading or mutating Arrow / OTAP records.
    #[error("otap evaluation error: {0}")]
    Evaluation(String),
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
    /// preserving exact 64-bit integer precision above `2^53` per `policy.md:313-317`.
    #[inline]
    #[must_use]
    pub fn cmp_candidate_i64(&self, candidate: i64) -> Option<Ordering> {
        match *self {
            Self::Int(thresh) => Some(candidate.cmp(&thresh)),
            Self::Double(thresh) => cmp_i64_with_f64(candidate, thresh),
        }
    }

    /// Compares a candidate `f64` field value against this `NumericValue` threshold (`candidate ? threshold`),
    /// preserving exact 64-bit integer precision above `2^53` per `policy.md:313-317`.
    #[inline]
    #[must_use]
    pub fn cmp_candidate_f64(&self, candidate: f64) -> Option<Ordering> {
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
pub fn cmp_i64_with_f64(int_val: i64, float_val: f64) -> Option<Ordering> {
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

/// Exact cross-type equality between `i64` and `f64` (`policy.md:258-262`).
#[inline]
#[must_use]
pub fn eq_i64_f64(int_val: i64, float_val: f64) -> bool {
    cmp_i64_with_f64(int_val, float_val) == Some(Ordering::Equal)
}

/// Unified 8-variant predicate (`common.proto:127-193`).
#[derive(Debug, Clone, PartialEq)]
pub enum Predicate {
    /// Field exists (`google.protobuf.Empty`).
    Exists,
    /// Exact scalar equality (`ScalarValue`).
    Equals(ScalarValue),
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
    pub fn matches_ordering(&self, ord: Option<Ordering>) -> bool {
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

/// Top-level log record fields (`logs.proto:127-166`).
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

/// Field selector for `LogMatcher` (`logs.proto:144-166`).
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

/// Target selector for `LogTransformPolicy` operations (`logs.proto:350-396`).
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LogTargetSelector {
    /// Top-level log record field (`Body`, `SeverityText`, `SeverityNumber`, `EventName`, `TraceId`, `SpanId`).
    RecordField(LogRecordField),
    /// Log record attribute (`LogAttrs`).
    LogAttribute(AttributePath),
    /// Resource attribute (`ResourceAttrs`).
    ResourceAttribute(AttributePath),
    /// Instrumentation scope attribute (`ScopeAttrs`).
    ScopeAttribute(AttributePath),
}

/// Single matcher within a `LogFilterPolicy` or `LogTransformPolicy` (`logs.proto:105-125`).
#[derive(Debug, Clone, PartialEq)]
pub struct LogMatcher {
    /// Field to inspect.
    pub field: LogFieldSelector,
    /// Predicate to evaluate.
    pub predicate: Predicate,
    /// When true, inverts the boolean result of `predicate`.
    pub negate: bool,
}

/// Atomic log filter policy (`logs.proto:45-68`).
#[derive(Debug, Clone, PartialEq)]
pub struct LogFilterPolicy {
    /// Conjunction (`AND`) of matchers. Must be non-empty.
    pub matches: Vec<LogMatcher>,
    /// Keep or drop action (`ACTION_KEEP` overrides `ACTION_DROP`).
    pub action: Action,
}

/// Stage 1 transform: rename a field/attribute (`logs.proto:195-210`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRename {
    /// Source target selector.
    pub from: LogTargetSelector,
    /// Destination target selector.
    pub to: LogTargetSelector,
    /// When false, does not overwrite an existing `to` field (and leaves `from` intact).
    pub upsert: bool,
}

/// Stage 2 transform: add/set a field/attribute (`logs.proto:212-234`).
#[derive(Debug, Clone, PartialEq)]
pub struct LogAdd {
    /// Target field/attribute to set.
    pub target: LogTargetSelector,
    /// Scalar value to write.
    pub value: ScalarValue,
    /// When false, only writes if `target` does not already exist.
    pub upsert: bool,
}

/// Stage 3 transform: remove a field/attribute (`logs.proto:184-193`).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogRemove {
    /// Target field/attribute to remove.
    pub target: LogTargetSelector,
}

/// Stage 4 transform: redact a field/attribute (`logs.proto:236-283`).
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

/// Atomic log transform policy (`logs.proto:70-103`).
#[derive(Debug, Clone, PartialEq)]
pub struct LogTransformPolicy {
    /// Conjunction (`AND`) of matchers. Empty `matches` matches all log records (`policy.md:424-428`).
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

/// Trace sampling policy placeholder (`traces.proto` in `#865`).
#[derive(Debug, Clone, PartialEq)]
pub struct TraceSamplingPolicy {
    /// Sampling probability (`0.0..=1.0`).
    pub probability: f64,
}

/// Policy definition oneof (`policies.proto:70-85`).
#[derive(Debug, Clone, PartialEq)]
pub enum PolicyDefinition {
    /// Log filter policy.
    LogFilter(LogFilterPolicy),
    /// Log transform policy.
    LogTransform(LogTransformPolicy),
    /// Metric filter policy.
    MetricFilter(MetricFilterPolicy),
    /// Trace sampling policy.
    TraceSampling(TraceSamplingPolicy),
}

/// Top-level `opentelemetry.proto.policies.v1development.Policy` (`policies.proto:35-87`).
#[derive(Debug, Clone, PartialEq)]
pub struct Policy {
    /// Unique identifier used for deterministic stage ordering (`policy.md:431-440`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    ///Whether the policy is enabled.
    pub enabled: bool,
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
            enabled: true,
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
            enabled: true,
            definition: PolicyDefinition::LogTransform(transform),
        }
    }
}

/// Expands RE2 / Rust regex replacement template syntax (`$0`, `$1`..`$99`, `${1}`..`${99}`, `${name}`, `$$`)
/// against a `regex::Captures` match per `policy.md:525-545`.
#[must_use]
pub fn apply_regex_redaction(re: &regex::Regex, haystack: &str, replacement_template: &str) -> Option<String> {
    if !re.is_match(haystack) {
        return None;
    }
    let mut out = String::with_capacity(haystack.len());
    let mut last_end = 0;
    for caps in re.captures_iter(haystack) {
        let Some(m0) = caps.get(0) else { continue };
        out.push_str(&haystack[last_end..m0.start()]);
        expand_capture_template(replacement_template, &caps, &mut out);
        last_end = m0.end();
    }
    out.push_str(&haystack[last_end..]);
    Some(out)
}

fn expand_capture_template(template: &str, caps: &regex::Captures<'_>, dst: &mut String) {
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
                    if let Ok(idx) = key.parse::<usize>() {
                        if let Some(m) = caps.get(idx) {
                            dst.push_str(m.as_str());
                        }
                    } else if let Some(m) = caps.name(key) {
                        dst.push_str(m.as_str());
                    }
                    i = close + 1;
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
                if let Some(m) = caps.get(idx) {
                    dst.push_str(m.as_str());
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
