// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Policy validation, predicate deduplication, and compilation into per-stage
//! [`StagePlan`] evaluators.

use ahash::AHashMap;
use regex_automata::meta::Regex as MetaRegex;
use regex_syntax::hir::Hir;

use crate::field_eval::{FieldEvaluator, StagePlan};
use crate::model::{
    Action, AttributePath, LogAdd, LogAttributeSelector, LogFieldSelector, LogMatcher,
    LogRecordField, LogRedact, LogTargetSelector, LogTransformPolicy, NumericValue, Policy,
    PolicyCompileError, PolicyDefinition, PolicyError, Predicate, ScalarValue, ScopeField,
    compile_matcher_regex, compile_redact_regex, matches_glob,
};
use crate::otap_ops::{CompiledLogRedact, CompiledTransformPolicy};
use crate::string_index::FieldStringIndexBuilder;

/// Compiled matcher reference (`pred_id` + `negate`).
#[derive(Debug, Clone, Copy)]
pub(crate) struct CompiledMatcherRef {
    pub(crate) pred_id: usize,
    pub(crate) negate: bool,
}

/// Rule matchers partitioned by evaluation domain so resource and scope attribute conjunctions
/// combine in compact ID space before checking log rows.
#[derive(Debug, Clone, Default)]
pub(crate) struct CompiledRuleMatchers {
    pub(crate) row_matchers: Vec<CompiledMatcherRef>,
    pub(crate) resource_matchers: Vec<CompiledMatcherRef>,
    pub(crate) scope_matchers: Vec<CompiledMatcherRef>,
}

/// Compiled filter policy rule.
#[derive(Debug, Clone)]
pub(crate) struct CompiledFilterRule {
    pub(crate) policy_idx: usize,
    pub(crate) matchers: CompiledRuleMatchers,
}

/// Compiled transform policy matcher conjunction aligned with `CompiledTransformPolicy`.
#[derive(Debug, Clone)]
pub(crate) struct CompiledTransformRule {
    pub(crate) policy_idx: usize,
    pub(crate) matchers: CompiledRuleMatchers,
}

/// All compiled artifacts assembled by [`compile_policies`] for [`crate::PolicyEngine`].
pub(crate) struct CompiledPolicies {
    pub(crate) policy_ids: Vec<String>,
    pub(crate) keep_plan: StagePlan,
    pub(crate) drop_plan: StagePlan,
    pub(crate) transform_plan: StagePlan,
    pub(crate) keep_rules: Vec<CompiledFilterRule>,
    pub(crate) drop_rules: Vec<CompiledFilterRule>,
    pub(crate) transform_rules: Vec<CompiledTransformRule>,
    pub(crate) compiled_transforms: Vec<CompiledTransformPolicy>,
    pub(crate) errors: Vec<PolicyCompileError>,
}

/// Returns `true` when `from` and `to` share the same root (`LogAttribute` or `BodyField`) and
/// one path is a strict prefix (ancestor) of the other (`policy.md` "Rename").
#[must_use]
pub(crate) fn attr_selectors_overlap(
    from: &LogAttributeSelector,
    to: &LogAttributeSelector,
) -> bool {
    let (fp, tp) = match (from, to) {
        (LogAttributeSelector::LogAttribute(a), LogAttributeSelector::LogAttribute(b))
        | (LogAttributeSelector::BodyField(a), LogAttributeSelector::BodyField(b)) => {
            (&a.path, &b.path)
        }
        _ => return false,
    };
    !fp.is_empty()
        && !tp.is_empty()
        && fp.len() != tp.len()
        && (fp.starts_with(tp) || tp.starts_with(fp))
}

/// Hashable canonical representation of [`ScalarValue`] for `O(1)` predicate deduplication.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CanonicalScalarKey {
    String(String),
    Int(i64),
    Bool(bool),
    DoubleBits(u64),
    Bytes(Vec<u8>),
}

impl From<&ScalarValue> for CanonicalScalarKey {
    fn from(value: &ScalarValue) -> Self {
        match value {
            ScalarValue::String(s) => Self::String(s.clone()),
            ScalarValue::Int(i) => Self::Int(*i),
            ScalarValue::Bool(b) => Self::Bool(*b),
            ScalarValue::Double(d) => Self::DoubleBits(canonical_f64_bits(*d)),
            ScalarValue::Bytes(b) => Self::Bytes(b.clone()),
        }
    }
}

/// Hashable canonical representation of [`NumericValue`] for `O(1)` predicate deduplication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum CanonicalNumericKey {
    Int(i64),
    DoubleBits(u64),
}

impl From<&NumericValue> for CanonicalNumericKey {
    fn from(value: &NumericValue) -> Self {
        match value {
            NumericValue::Int(i) => Self::Int(*i),
            NumericValue::Double(d) => Self::DoubleBits(canonical_f64_bits(*d)),
        }
    }
}

/// Hashable canonical representation of [`Predicate`] for `O(1)` predicate deduplication.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
enum CanonicalPredicateKey {
    Exists,
    Equals(CanonicalScalarKey),
    Glob(String),
    Regex(String),
    Gt(CanonicalNumericKey),
    Gte(CanonicalNumericKey),
    Lt(CanonicalNumericKey),
    Lte(CanonicalNumericKey),
    Contains(CanonicalScalarKey),
}

impl From<&Predicate> for CanonicalPredicateKey {
    fn from(pred: &Predicate) -> Self {
        match pred {
            Predicate::Exists => Self::Exists,
            Predicate::Equals(s) => Self::Equals(CanonicalScalarKey::from(s)),
            // A wildcard-free glob is semantically identical to exact string equality across
            // every OTLP field type, so canonicalizing it into `Equals(String)` deduplicates
            // both into a single `pred_id` and preserves 1:1 exact-match maps.
            Predicate::Glob(g) if !g.contains(['*', '?']) => {
                Self::Equals(CanonicalScalarKey::String(g.clone()))
            }
            Predicate::Glob(g) => Self::Glob(g.clone()),
            Predicate::Regex(r) => Self::Regex(r.clone()),
            Predicate::Gt(n) => Self::Gt(CanonicalNumericKey::from(n)),
            Predicate::Gte(n) => Self::Gte(CanonicalNumericKey::from(n)),
            Predicate::Lt(n) => Self::Lt(CanonicalNumericKey::from(n)),
            Predicate::Lte(n) => Self::Lte(CanonicalNumericKey::from(n)),
            Predicate::Contains(s) => Self::Contains(CanonicalScalarKey::from(s)),
        }
    }
}

#[inline]
fn decode_canonical_lower_hex<const N: usize>(s: &str) -> Option<[u8; N]> {
    if s.len() != N * 2
        || !s
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return None;
    }
    let mut out = [0u8; N];
    hex::decode_to_slice(s, &mut out).ok()?;
    if out.iter().all(|&b| b == 0) {
        return None;
    }
    Some(out)
}

#[inline]
fn canonical_f64_bits(v: f64) -> u64 {
    if v == 0.0 {
        0.0_f64.to_bits()
    } else {
        v.to_bits()
    }
}

/// Per-field builder used during policy compilation.
struct FieldBuilder {
    field: LogFieldSelector,
    pred_dedup: AHashMap<CanonicalPredicateKey, usize>,
    exists_pred: Option<usize>,
    str_equals_map: AHashMap<String, usize>,
    trace_id_equals_map: AHashMap<[u8; 16], usize>,
    span_id_equals_map: AHashMap<[u8; 8], usize>,
    empty_id_preds: Vec<usize>,
    string_index_builder: FieldStringIndexBuilder,
    contains_preds: Vec<(usize, ScalarValue)>,
    scalar_and_range_preds: Vec<(usize, Predicate)>,
}

impl FieldBuilder {
    fn new(field: LogFieldSelector) -> Self {
        Self {
            field,
            pred_dedup: AHashMap::new(),
            exists_pred: None,
            str_equals_map: AHashMap::new(),
            trace_id_equals_map: AHashMap::new(),
            span_id_equals_map: AHashMap::new(),
            empty_id_preds: Vec::new(),
            string_index_builder: FieldStringIndexBuilder::default(),
            contains_preds: Vec::new(),
            scalar_and_range_preds: Vec::new(),
        }
    }

    fn is_id_field(&self) -> bool {
        matches!(
            self.field,
            LogFieldSelector::RecordField(LogRecordField::TraceId | LogRecordField::SpanId)
        )
    }

    fn register_str_equals(&mut self, pred_id: usize, s: &str) {
        match &self.field {
            LogFieldSelector::RecordField(LogRecordField::TraceId) => {
                if s.is_empty() {
                    self.empty_id_preds.push(pred_id);
                } else if let Some(bytes) = decode_canonical_lower_hex::<16>(s) {
                    let _ = self.trace_id_equals_map.insert(bytes, pred_id);
                }
            }
            LogFieldSelector::RecordField(LogRecordField::SpanId) => {
                if s.is_empty() {
                    self.empty_id_preds.push(pred_id);
                } else if let Some(bytes) = decode_canonical_lower_hex::<8>(s) {
                    let _ = self.span_id_equals_map.insert(bytes, pred_id);
                }
            }
            _ => {
                let _ = self.str_equals_map.insert(s.to_owned(), pred_id);
            }
        }
    }

    fn add_predicate(
        &mut self,
        predicate: &Predicate,
        regex_cache: &AHashMap<String, Result<(Hir, MetaRegex), String>>,
        next_pred_id: &mut usize,
    ) -> Result<usize, PolicyError> {
        let key = CanonicalPredicateKey::from(predicate);
        if let Some(&existing_id) = self.pred_dedup.get(&key) {
            return Ok(existing_id);
        }

        let pred_id = *next_pred_id;
        *next_pred_id += 1;
        let _ = self.pred_dedup.insert(key, pred_id);

        match predicate {
            Predicate::Exists => {
                self.exists_pred = Some(pred_id);
            }
            Predicate::Equals(ScalarValue::String(s)) => {
                self.register_str_equals(pred_id, s);
            }
            Predicate::Glob(pat) => {
                if !pat.contains(['*', '?']) {
                    self.register_str_equals(pred_id, pat);
                } else {
                    if self.is_id_field() && matches_glob(pat, "") {
                        self.empty_id_preds.push(pred_id);
                    }
                    self.string_index_builder.add_glob(pred_id, pat);
                }
            }
            Predicate::Equals(_)
            | Predicate::Gt(_)
            | Predicate::Gte(_)
            | Predicate::Lt(_)
            | Predicate::Lte(_) => {
                self.scalar_and_range_preds
                    .push((pred_id, predicate.clone()));
            }
            Predicate::Contains(scalar) => {
                if let ScalarValue::String(s) = scalar {
                    if self.is_id_field() && s.is_empty() {
                        self.empty_id_preds.push(pred_id);
                    }
                    self.string_index_builder.add_contains(pred_id, s);
                }
                self.contains_preds.push((pred_id, scalar.clone()));
            }
            Predicate::Regex(pat) => {
                let (hir, re) = match regex_cache.get(pat) {
                    Some(Ok((h, r))) => (h.clone(), r.clone()),
                    _ => compile_matcher_regex(pat)?,
                };
                if self.is_id_field() && re.is_match("") {
                    self.empty_id_preds.push(pred_id);
                }
                self.string_index_builder
                    .add_compiled_regex(pred_id, &hir, re);
            }
        }

        Ok(pred_id)
    }

    fn build(self) -> Result<FieldEvaluator, PolicyError> {
        Ok(FieldEvaluator {
            field: self.field,
            exists_pred: self.exists_pred,
            str_equals_map: self.str_equals_map,
            trace_id_equals_map: self.trace_id_equals_map,
            span_id_equals_map: self.span_id_equals_map,
            empty_id_preds: self.empty_id_preds,
            string_index: self.string_index_builder.build()?,
            contains_preds: self.contains_preds,
            scalar_and_range_preds: self.scalar_and_range_preds,
        })
    }
}

#[derive(Default)]
struct StagePlanBuilder {
    field_builders: Vec<FieldBuilder>,
    field_index_map: AHashMap<LogFieldSelector, usize>,
    next_pred_id: usize,
}

impl StagePlanBuilder {
    fn register_matcher(
        &mut self,
        m: &LogMatcher,
        regex_cache: &AHashMap<String, Result<(Hir, MetaRegex), String>>,
    ) -> Result<CompiledMatcherRef, PolicyError> {
        let fb_idx = if let Some(&idx) = self.field_index_map.get(&m.field) {
            idx
        } else {
            let idx = self.field_builders.len();
            self.field_builders.push(FieldBuilder::new(m.field.clone()));
            let _ = self.field_index_map.insert(m.field.clone(), idx);
            idx
        };
        let pred_id = self.field_builders[fb_idx].add_predicate(
            &m.predicate,
            regex_cache,
            &mut self.next_pred_id,
        )?;
        Ok(CompiledMatcherRef {
            pred_id,
            negate: m.negate,
        })
    }

    fn build(self) -> Result<StagePlan, PolicyError> {
        let mut field_evaluators = Vec::with_capacity(self.field_builders.len());
        let mut root_evaluator_indices = Vec::new();
        let mut log_attr_evaluators: AHashMap<String, Vec<usize>> = AHashMap::new();
        let mut res_attr_evaluators: AHashMap<String, Vec<usize>> = AHashMap::new();
        let mut scope_attr_evaluators: AHashMap<String, Vec<usize>> = AHashMap::new();

        for fb in self.field_builders {
            let idx = field_evaluators.len();
            let ev = fb.build()?;
            match &ev.field {
                LogFieldSelector::RecordField(_)
                | LogFieldSelector::ScopeField(_)
                | LogFieldSelector::BodyField(_) => {
                    root_evaluator_indices.push(idx);
                }
                LogFieldSelector::LogAttribute(path) => {
                    if let Some(first) = path.path.first() {
                        log_attr_evaluators
                            .entry(first.clone())
                            .or_default()
                            .push(idx);
                    }
                }
                LogFieldSelector::ResourceAttribute(path) => {
                    if let Some(first) = path.path.first() {
                        res_attr_evaluators
                            .entry(first.clone())
                            .or_default()
                            .push(idx);
                    }
                }
                LogFieldSelector::ScopeAttribute(path) => {
                    if let Some(first) = path.path.first() {
                        scope_attr_evaluators
                            .entry(first.clone())
                            .or_default()
                            .push(idx);
                    }
                }
            }
            field_evaluators.push(ev);
        }

        Ok(StagePlan {
            num_predicates: self.next_pred_id,
            field_evaluators,
            root_evaluator_indices,
            log_attr_evaluators,
            res_attr_evaluators,
            scope_attr_evaluators,
        })
    }
}

fn validate_attr_path(
    loc: &str,
    label: &str,
    path: &AttributePath,
    policy_errors: &mut Vec<String>,
) {
    if path.path.is_empty() {
        policy_errors.push(format!("{loc}: {label} path is empty"));
    } else if path.path.iter().any(String::is_empty) {
        policy_errors.push(format!("{loc}: {label} path contains an empty segment"));
    }
}

fn validate_finite_scalar(loc: &str, scalar: &ScalarValue, policy_errors: &mut Vec<String>) {
    if let ScalarValue::Double(d) = scalar
        && !d.is_finite()
    {
        policy_errors.push(format!("{loc}: double_value must be finite"));
    }
}

fn validate_finite_numeric(loc: &str, num: &NumericValue, policy_errors: &mut Vec<String>) {
    if let NumericValue::Double(d) = num
        && !d.is_finite()
    {
        policy_errors.push(format!("{loc}: double_value must be finite"));
    }
}

fn validate_log_matcher(
    idx: usize,
    m: &LogMatcher,
    regex_cache: &mut AHashMap<String, Result<(Hir, MetaRegex), String>>,
    policy_errors: &mut Vec<String>,
) {
    let loc = format!("matches[{idx}]");
    match &m.field {
        LogFieldSelector::RecordField(LogRecordField::Unspecified) => {
            policy_errors.push(format!("{loc}: record_field is unspecified"));
        }
        LogFieldSelector::ScopeField(ScopeField::Unspecified) => {
            policy_errors.push(format!("{loc}: scope_field is unspecified"));
        }
        LogFieldSelector::LogAttribute(p) => {
            validate_attr_path(&loc, "log_attribute", p, policy_errors);
        }
        LogFieldSelector::ResourceAttribute(p) => {
            validate_attr_path(&loc, "resource_attribute", p, policy_errors);
        }
        LogFieldSelector::ScopeAttribute(p) => {
            validate_attr_path(&loc, "scope_attribute", p, policy_errors);
        }
        LogFieldSelector::BodyField(p) => {
            validate_attr_path(&loc, "body_field", p, policy_errors);
        }
        _ => {}
    }

    match &m.predicate {
        Predicate::Equals(s) | Predicate::Contains(s) => {
            validate_finite_scalar(&loc, s, policy_errors);
        }
        Predicate::Gt(n) | Predicate::Gte(n) | Predicate::Lt(n) | Predicate::Lte(n) => {
            validate_finite_numeric(&loc, n, policy_errors);
        }
        Predicate::Regex(pat) => {
            let entry = regex_cache
                .entry(pat.clone())
                .or_insert_with(|| compile_matcher_regex(pat).map_err(|e| e.to_string()));
            if let Err(err_msg) = entry {
                policy_errors.push(format!("{loc}: {err_msg}"));
            }
        }
        Predicate::Exists | Predicate::Glob(_) => {}
    }
}

fn validate_attr_selector(loc: &str, sel: &LogAttributeSelector, policy_errors: &mut Vec<String>) {
    match sel {
        LogAttributeSelector::LogAttribute(p) => {
            validate_attr_path(loc, "log_attribute", p, policy_errors);
        }
        LogAttributeSelector::BodyField(p) => {
            validate_attr_path(loc, "body_field", p, policy_errors);
        }
    }
}

fn validate_log_add(idx: usize, op: &LogAdd, policy_errors: &mut Vec<String>) {
    let loc = format!("add[{idx}]");
    validate_finite_scalar(&format!("{loc}.value"), &op.value, policy_errors);

    match &op.target {
        LogTargetSelector::RecordField(LogRecordField::Unspecified) => {
            policy_errors.push(format!("{loc}.target: record_field is unspecified"));
        }
        LogTargetSelector::RecordField(LogRecordField::TraceId | LogRecordField::SpanId) => {
            policy_errors.push(format!(
                "{loc}.target: read-only record_field cannot be modified by add"
            ));
        }
        LogTargetSelector::RecordField(
            LogRecordField::SeverityText | LogRecordField::EventName,
        ) => {
            if !matches!(op.value, ScalarValue::String(_)) {
                policy_errors.push(format!(
                    "{loc}.value: string record_field requires string_value"
                ));
            }
        }
        LogTargetSelector::RecordField(LogRecordField::SeverityNumber) => {
            if !matches!(op.value, ScalarValue::Int(i) if (1..=24).contains(&i)) {
                policy_errors.push(format!(
                    "{loc}.value: severity_number requires int_value in 1..=24"
                ));
            }
        }
        LogTargetSelector::RecordField(LogRecordField::Body) => {}
        LogTargetSelector::LogAttribute(p) => {
            validate_attr_path(&format!("{loc}.target"), "log_attribute", p, policy_errors);
        }
        LogTargetSelector::BodyField(p) => {
            validate_attr_path(&format!("{loc}.target"), "body_field", p, policy_errors);
        }
    }
}

fn validate_and_compile_redact(
    idx: usize,
    op: &LogRedact,
    policy_errors: &mut Vec<String>,
) -> Option<CompiledLogRedact> {
    let loc = format!("redact[{idx}]");
    match &op.target {
        LogTargetSelector::RecordField(LogRecordField::Unspecified) => {
            policy_errors.push(format!("{loc}.target: record_field is unspecified"));
        }
        LogTargetSelector::RecordField(
            LogRecordField::SeverityNumber | LogRecordField::TraceId | LogRecordField::SpanId,
        ) => {
            policy_errors.push(format!(
                "{loc}.target: non-string or read-only record_field cannot be redacted"
            ));
        }
        LogTargetSelector::RecordField(
            LogRecordField::Body | LogRecordField::SeverityText | LogRecordField::EventName,
        ) => {}
        LogTargetSelector::LogAttribute(p) => {
            validate_attr_path(&format!("{loc}.target"), "log_attribute", p, policy_errors);
        }
        LogTargetSelector::BodyField(p) => {
            validate_attr_path(&format!("{loc}.target"), "body_field", p, policy_errors);
        }
    }

    let replacement = op.effective_replacement();
    let compiled_re = match &op.regex {
        Some(pat) => match compile_redact_regex(pat, replacement.contains('$')) {
            Ok(re) => Some(re),
            Err(e) => {
                policy_errors.push(format!("{loc}.regex: {e}"));
                None
            }
        },
        None => None,
    };

    if policy_errors.is_empty() {
        Some(CompiledLogRedact {
            target: op.target.clone(),
            replacement: replacement.to_owned(),
            regex: compiled_re,
        })
    } else {
        None
    }
}

fn validate_and_compile_transform(
    tx: &LogTransformPolicy,
    regex_cache: &mut AHashMap<String, Result<(Hir, MetaRegex), String>>,
    policy_errors: &mut Vec<String>,
) -> Vec<CompiledLogRedact> {
    if tx.rename.is_empty() && tx.add.is_empty() && tx.remove.is_empty() && tx.redact.is_empty() {
        policy_errors.push("transform policy must contain at least one operation".into());
    }

    for (idx, m) in tx.matches.iter().enumerate() {
        validate_log_matcher(idx, m, regex_cache, policy_errors);
    }

    for (idx, op) in tx.rename.iter().enumerate() {
        validate_attr_selector(&format!("rename[{idx}].from"), &op.from, policy_errors);
        validate_attr_selector(&format!("rename[{idx}].to"), &op.to, policy_errors);
        if attr_selectors_overlap(&op.from, &op.to) {
            policy_errors.push(format!(
                "rename[{idx}]: from and to paths overlap as ancestor/descendant"
            ));
        }
    }
    for (idx, op) in tx.add.iter().enumerate() {
        validate_log_add(idx, op, policy_errors);
    }
    for (idx, op) in tx.remove.iter().enumerate() {
        validate_attr_selector(&format!("remove[{idx}].target"), &op.target, policy_errors);
    }

    let mut compiled_redacts = Vec::with_capacity(tx.redact.len());
    for (idx, op) in tx.redact.iter().enumerate() {
        if let Some(compiled) = validate_and_compile_redact(idx, op, policy_errors) {
            compiled_redacts.push(compiled);
        }
    }

    compiled_redacts
}

fn register_rule_matchers(
    stage_builder: &mut StagePlanBuilder,
    matchers_in: &[LogMatcher],
    regex_cache: &AHashMap<String, Result<(Hir, MetaRegex), String>>,
) -> Result<CompiledRuleMatchers, PolicyError> {
    let mut matchers = CompiledRuleMatchers::default();
    for m in matchers_in {
        let mref = stage_builder.register_matcher(m, regex_cache)?;
        match &m.field {
            LogFieldSelector::ResourceAttribute(_) => matchers.resource_matchers.push(mref),
            LogFieldSelector::ScopeAttribute(_) => matchers.scope_matchers.push(mref),
            _ => matchers.row_matchers.push(mref),
        }
    }
    Ok(matchers)
}

pub(crate) fn compile_policies(policies: &[Policy]) -> Result<CompiledPolicies, PolicyError> {
    let mut keep_builder = StagePlanBuilder::default();
    let mut drop_builder = StagePlanBuilder::default();
    let mut transform_builder = StagePlanBuilder::default();
    let mut regex_cache: AHashMap<String, Result<(Hir, MetaRegex), String>> = AHashMap::new();
    let mut errors: Vec<PolicyCompileError> = Vec::new();

    // Count enabled policy IDs up front so duplicate ID rejection is commutative (order-independent).
    let mut id_counts: AHashMap<&str, usize> = AHashMap::with_capacity(policies.len());
    for policy in policies {
        if policy.enabled && !policy.id.is_empty() {
            *id_counts.entry(policy.id.as_str()).or_insert(0) += 1;
        }
    }

    let mut policy_ids: Vec<String> = Vec::new();
    let mut keep_rules = Vec::new();
    let mut drop_rules = Vec::new();
    let mut validated_transforms: Vec<(String, &LogTransformPolicy, CompiledTransformPolicy)> =
        Vec::new();

    for policy in policies {
        if !policy.enabled {
            continue;
        }
        let mut policy_errors: Vec<String> = Vec::new();
        if policy.id.is_empty() {
            policy_errors.push("policy id is empty".into());
        } else if id_counts.get(policy.id.as_str()).copied().unwrap_or(0) > 1 {
            policy_errors.push("duplicate policy id in batch".into());
        }

        match &policy.definition {
            PolicyDefinition::LogFilter(lf) => {
                if lf.action == Action::Unspecified {
                    policy_errors.push("action is unspecified".into());
                }
                if lf.matches.is_empty() {
                    policy_errors.push("matches must be non-empty".into());
                }
                for (idx, m) in lf.matches.iter().enumerate() {
                    validate_log_matcher(idx, m, &mut regex_cache, &mut policy_errors);
                }
                if policy_errors.is_empty() {
                    let stage_builder = match lf.action {
                        Action::Keep => &mut keep_builder,
                        Action::Drop | Action::Unspecified => &mut drop_builder,
                    };
                    let matchers =
                        register_rule_matchers(stage_builder, &lf.matches, &regex_cache)?;
                    let policy_idx = policy_ids.len();
                    policy_ids.push(policy.id.clone());
                    let rule = CompiledFilterRule {
                        policy_idx,
                        matchers,
                    };
                    match lf.action {
                        Action::Keep => keep_rules.push(rule),
                        Action::Drop => drop_rules.push(rule),
                        Action::Unspecified => {}
                    }
                }
            }
            PolicyDefinition::LogTransform(tx) => {
                let compiled_redacts =
                    validate_and_compile_transform(tx, &mut regex_cache, &mut policy_errors);
                if policy_errors.is_empty() {
                    let compiled_tx = CompiledTransformPolicy {
                        rename: tx.rename.clone(),
                        add: tx.add.clone(),
                        remove: tx.remove.clone(),
                        redact: compiled_redacts,
                    };
                    validated_transforms.push((policy.id.clone(), tx, compiled_tx));
                }
            }
            PolicyDefinition::MetricFilter(_) => {
                policy_errors.push("metric filter policies are not yet supported".into());
            }
            PolicyDefinition::SpanSampling(_) => {
                policy_errors.push("span sampling policies are not yet supported".into());
            }
            PolicyDefinition::CustomPolicy(_) => {
                policy_errors.push("custom policies are not supported".into());
            }
        }

        for message in policy_errors {
            errors.push(PolicyCompileError {
                policy_id: policy.id.clone(),
                message,
            });
        }
    }

    // Deterministic transform stage ordering by ascending `Policy.id` (`policy.md` "Execution Order").
    validated_transforms.sort_by(|a, b| a.0.cmp(&b.0));

    let mut transform_rules = Vec::with_capacity(validated_transforms.len());
    let mut compiled_transforms = Vec::with_capacity(validated_transforms.len());
    for (policy_id, tx, compiled_tx) in validated_transforms {
        let matchers = register_rule_matchers(&mut transform_builder, &tx.matches, &regex_cache)?;
        let policy_idx = policy_ids.len();
        policy_ids.push(policy_id);
        transform_rules.push(CompiledTransformRule {
            policy_idx,
            matchers,
        });
        compiled_transforms.push(compiled_tx);
    }

    let keep_plan = keep_builder.build()?;
    let drop_plan = drop_builder.build()?;
    let transform_plan = transform_builder.build()?;

    Ok(CompiledPolicies {
        policy_ids,
        keep_plan,
        drop_plan,
        transform_plan,
        keep_rules,
        drop_rules,
        transform_rules,
        compiled_transforms,
        errors,
    })
}
