// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HIR / Glob N-Gram Inverted Index (`AhoCorasick`) + `regex_automata::meta::Regex`
//! columnar policy evaluation engine (`open-telemetry/otel-arrow#4726`).
//!
//! # Architecture
//! - Compiles policies in [`crate::compile`] into per-stage [`StagePlan`]s (keep, drop, transform)
//!   with deduplicated predicates and per-field n-gram inverted indices ([`crate::string_index`]).
//! - Evaluates predicates columnarly in [`crate::field_eval`] using Arrow SIMD kernels,
//!   dictionary memoization, selectivity-gated attribute scans, and pooled dense bitsets.
//! - Orchestrates filter short-circuiting (`ACTION_KEEP` overriding `ACTION_DROP`), exact
//!   per-policy match statistics, and global 4-stage masked transforms (`1. Rename -> 2. Add ->
//!   3. Remove -> 4. Redact`) in this module.

use std::sync::atomic::{AtomicU64, Ordering};

use arrow::array::{Array, RecordBatch, StructArray, UInt16Array};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::IdBitmapPool;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::EvaluationOutput;
use crate::bitset::{BitsetBufferPool, DenseBitset};
use crate::compile::{
    CompiledFilterRule, CompiledMatcherRef, CompiledRuleMatchers, CompiledTransformRule,
    compile_policies,
};
use crate::field_eval::{StagePlan, evaluate_attr_table, evaluate_root_field, for_each_active_row};
use crate::model::{Policy, PolicyCompileError, PolicyError};
use crate::otap_ops::{
    CompiledTransformPolicy, LogsBatchView, apply_spec_transforms_masked,
    filter_otap_logs_by_bitmap,
};
use crate::string_index::StringScanScratch;

/// Reusable per-core evaluation scratch state pooling [`IdBitmapPool`], n-gram scan counters,
/// and dense bitset word buffers across batches to eliminate hot-path allocations.
#[derive(Default)]
pub struct EvaluationScratch {
    id_bitmap_pool: IdBitmapPool,
    string_scratches: Vec<StringScanScratch>,
    bitset_pool: BitsetBufferPool,
}

impl EvaluationScratch {
    /// Creates an empty reusable `EvaluationScratch` that sizes lazily to the active engine.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    fn ensure_string_scratches(&mut self, count: usize) {
        if self.string_scratches.len() < count {
            self.string_scratches
                .resize_with(count, StringScanScratch::default);
        }
    }

    #[cfg(test)]
    pub(crate) fn pooled_bitset_buffers(&self) -> usize {
        self.bitset_pool.free_len()
    }

    #[cfg(test)]
    pub(crate) fn set_string_scratch_gen_for_test(&mut self, ev_idx: usize, gen_val: u32) {
        self.ensure_string_scratches(ev_idx + 1);
        self.string_scratches[ev_idx].set_cur_gen_for_test(gen_val);
    }
}

/// Result of evaluating a conjunction of predicates in resource-ID or scope-ID space.
enum IdFilter {
    /// Every log row satisfies the ID-space constraint (including rows with null/missing parent ID).
    PassAll,
    /// No log row can satisfy the ID-space constraint (short-circuits row evaluation).
    Empty,
    /// Log row must carry a valid parent ID present in this bitmap.
    KeepIds(DenseBitset),
    /// Log row passes unless it carries a valid parent ID present in this bitmap.
    ExcludeIds(DenseBitset),
}

impl IdFilter {
    fn recycle(self, pool: &mut BitsetBufferPool) {
        match self {
            Self::KeepIds(bm) | Self::ExcludeIds(bm) => pool.recycle(bm),
            Self::PassAll | Self::Empty => {}
        }
    }
}

/// Row match bitmap returned by rule evaluation, borrowing directly from `pred_bitmaps` or
/// `candidate_rows` on single-matcher positive row-only rules to avoid cloning.
enum MatchedRows<'a> {
    Borrowed(&'a DenseBitset),
    Owned(DenseBitset),
}

impl<'a> MatchedRows<'a> {
    #[inline]
    fn as_bitset(&self) -> &DenseBitset {
        match self {
            Self::Borrowed(bm) => bm,
            Self::Owned(bm) => bm,
        }
    }

    #[inline]
    fn into_owned(self, pool: &mut BitsetBufferPool) -> DenseBitset {
        match self {
            Self::Borrowed(bm) => bm.clone_with_pool(pool),
            Self::Owned(bm) => bm,
        }
    }

    #[inline]
    fn recycle(self, pool: &mut BitsetBufferPool) {
        if let Self::Owned(bm) = self {
            pool.recycle(bm);
        }
    }
}

/// Monotonic ID generator so each [`PolicyEngine::compile`] call binds its [`PolicyMatchStats`]
/// instances and catches stale stats reused across policy reloads.
static NEXT_ENGINE_ID: AtomicU64 = AtomicU64::new(1);

/// Cumulative `hit` and `miss` counts for a single compiled policy (`policy.md` "Match Outcome Semantics").
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct PolicyMatchCount {
    /// Records where the stage's final decision honored this matching policy's intent.
    pub hits: u64,
    /// Records where another matching policy in the same stage overrode this policy's intent.
    pub misses: u64,
}

/// Reusable per-policy match outcome accumulator aligned to a compiled [`PolicyEngine`].
///
/// Tracks the evaluated record denominator (`evaluated_rows`) and per-policy `hit` / `miss`
/// outcomes defined in `policy.md` ("Match Outcome Semantics"). Create via
/// [`PolicyEngine::new_match_stats`], pass as `Some(&mut stats)` to [`PolicyEngine::evaluate`] or
/// [`PolicyEngine::evaluate_filter`], and call [`PolicyMatchStats::reset`] between reporting
/// intervals to zero counters without reallocating.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PolicyMatchStats {
    engine_id: u64,
    evaluated_rows: u64,
    counts: Vec<PolicyMatchCount>,
}

impl PolicyMatchStats {
    /// Returns the total number of log records that entered policy evaluation (denominator).
    #[must_use]
    pub const fn evaluated_rows(&self) -> u64 {
        self.evaluated_rows
    }

    /// Returns per-policy counts aligned 1:1 with [`PolicyEngine::policy_ids`].
    #[must_use]
    pub fn counts(&self) -> &[PolicyMatchCount] {
        &self.counts
    }

    /// Iterates `(policy_id, counts)` pairs for every valid compiled policy in `engine`.
    ///
    /// Returns [`PolicyError::Evaluation`] if these stats were created for a different compiled
    /// [`PolicyEngine`].
    pub fn iter<'a>(
        &'a self,
        engine: &'a PolicyEngine,
    ) -> Result<impl Iterator<Item = (&'a str, PolicyMatchCount)> + 'a, PolicyError> {
        engine.validate_match_stats(self)?;
        Ok(engine
            .policy_ids()
            .iter()
            .map(String::as_str)
            .zip(self.counts.iter().copied()))
    }

    /// Resets `evaluated_rows` and all per-policy counts to zero without reallocating.
    pub fn reset(&mut self) {
        self.evaluated_rows = 0;
        for c in &mut self.counts {
            *c = PolicyMatchCount::default();
        }
    }
}

/// Opaque set of kept log row indices (`0..num_rows`) returned by [`PolicyEngine::evaluate_filter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowSelection {
    bitmap: DenseBitset,
}

impl RowSelection {
    /// Returns the number of selected rows.
    #[must_use]
    pub fn len(&self) -> u64 {
        self.bitmap.len()
    }

    /// Returns `true` if no rows were selected.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.bitmap.is_empty()
    }

    /// Returns `true` if row index `row` is selected.
    #[must_use]
    pub fn contains(&self, row: u32) -> bool {
        self.bitmap.contains(row)
    }

    /// Iterates over selected row indices in ascending order.
    pub fn iter(&self) -> impl Iterator<Item = u32> + '_ {
        self.bitmap.iter()
    }

    #[cfg(test)]
    pub(crate) fn into_bitmap(self) -> DenseBitset {
        self.bitmap
    }
}

/// Output of [`PolicyEngine::compile`], containing the compiled engine for all valid policies
/// alongside any per-policy validation or unsupported-feature errors.
#[derive(Debug, Clone)]
pub struct CompileOutput {
    /// Compiled policy engine containing all valid, enabled policies.
    pub engine: PolicyEngine,
    /// Validation errors for policies that were rejected and kept inert.
    pub errors: Vec<PolicyCompileError>,
}

/// Pure-Rust columnar policy evaluation engine: HIR / Glob N-Gram Inverted Index (`AhoCorasick`)
/// with `regex_automata::meta::Regex` verification and columnar OTAP execution.
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    engine_id: u64,
    policy_ids: Vec<String>,
    keep_plan: StagePlan,
    drop_plan: StagePlan,
    transform_plan: StagePlan,
    keep_rules: Vec<CompiledFilterRule>,
    drop_rules: Vec<CompiledFilterRule>,
    transform_rules: Vec<CompiledTransformRule>,
    compiled_transforms: Vec<CompiledTransformPolicy>,
}

impl PolicyEngine {
    /// Compiles a slice of `Policy` definitions into a [`CompileOutput`].
    ///
    /// Each enabled policy is validated atomically before any of its matchers or transforms are
    /// registered (`policy.md` "Compilation Errors"). Invalid or unsupported policies are kept
    /// inert and reported in [`CompileOutput::errors`], while valid policies in the same batch
    /// continue to compile and execute.
    pub fn compile(policies: &[Policy]) -> Result<CompileOutput, PolicyError> {
        let compiled = compile_policies(policies)?;
        let engine_id = NEXT_ENGINE_ID.fetch_add(1, Ordering::Relaxed);
        Ok(CompileOutput {
            engine: Self {
                engine_id,
                policy_ids: compiled.policy_ids,
                keep_plan: compiled.keep_plan,
                drop_plan: compiled.drop_plan,
                transform_plan: compiled.transform_plan,
                keep_rules: compiled.keep_rules,
                drop_rules: compiled.drop_rules,
                transform_rules: compiled.transform_rules,
                compiled_transforms: compiled.compiled_transforms,
            },
            errors: compiled.errors,
        })
    }

    /// Returns the IDs of all valid compiled policies in index order aligned with
    /// [`PolicyMatchStats::counts`]. Disabled and compile-error policies are excluded.
    #[must_use]
    pub fn policy_ids(&self) -> &[String] {
        &self.policy_ids
    }

    /// Allocates a zeroed [`PolicyMatchStats`] accumulator bound to this compiled policy set.
    #[must_use]
    pub fn new_match_stats(&self) -> PolicyMatchStats {
        PolicyMatchStats {
            engine_id: self.engine_id,
            evaluated_rows: 0,
            counts: vec![PolicyMatchCount::default(); self.policy_ids.len()],
        }
    }

    fn validate_match_stats(&self, stats: &PolicyMatchStats) -> Result<(), PolicyError> {
        if stats.engine_id != self.engine_id || stats.counts.len() != self.policy_ids.len() {
            return Err(PolicyError::Evaluation(
                "PolicyMatchStats does not match this compiled PolicyEngine".into(),
            ));
        }
        Ok(())
    }

    #[inline]
    fn has_filter_rules(&self) -> bool {
        !self.keep_rules.is_empty() || !self.drop_rules.is_empty()
    }

    #[inline]
    fn filter_uses_attributes(&self) -> bool {
        self.keep_plan.uses_attributes()
            || (!self.drop_rules.is_empty() && self.drop_plan.uses_attributes())
    }

    #[cfg(test)]
    pub(crate) fn num_keep_predicates(&self) -> usize {
        self.keep_plan.num_predicates
    }

    #[cfg(test)]
    pub(crate) fn num_drop_predicates(&self) -> usize {
        self.drop_plan.num_predicates
    }

    #[cfg(test)]
    pub(crate) fn num_filter_predicates(&self) -> usize {
        self.keep_plan.num_predicates + self.drop_plan.num_predicates
    }

    #[cfg(test)]
    pub(crate) fn num_transform_predicates(&self) -> usize {
        self.transform_plan.num_predicates
    }

    #[cfg(test)]
    pub(crate) fn num_predicates(&self) -> usize {
        self.num_filter_predicates() + self.transform_plan.num_predicates
    }

    #[cfg(test)]
    pub(crate) fn evaluate_filter_predicates_for_test(
        &self,
        view: &LogsBatchView<'_>,
        scratch: &mut EvaluationScratch,
    ) -> Vec<DenseBitset> {
        let mut out = self.evaluate_predicates(&self.keep_plan, view, scratch, None);
        out.extend(self.evaluate_predicates(&self.drop_plan, view, scratch, None));
        out
    }

    #[cfg(test)]
    pub(crate) fn evaluate_filter_view_for_test(
        &self,
        view: &LogsBatchView<'_>,
        scratch: &mut EvaluationScratch,
        stats: Option<&mut PolicyMatchStats>,
    ) -> DenseBitset {
        self.evaluate_filter_view(view, scratch, stats)
    }

    fn evaluate_predicates(
        &self,
        plan: &StagePlan,
        view: &LogsBatchView<'_>,
        scratch: &mut EvaluationScratch,
        active_rows: Option<&DenseBitset>,
    ) -> Vec<DenseBitset> {
        let mut pred_bitmaps = vec![DenseBitset::new(); plan.num_predicates];
        if plan.num_predicates == 0 {
            return pred_bitmaps;
        }

        scratch.ensure_string_scratches(plan.field_evaluators.len());
        let row_words = view.num_rows.div_ceil(64).max(1);

        // 1. Evaluate root / scope / body fields.
        for &ev_idx in &plan.root_evaluator_indices {
            let ev = &plan.field_evaluators[ev_idx];
            let str_scratch = &mut scratch.string_scratches[ev_idx];
            ev.string_index.prepare_scratch(str_scratch);
            evaluate_root_field(
                ev,
                view,
                row_words,
                active_rows,
                str_scratch,
                &mut scratch.bitset_pool,
                &mut pred_bitmaps,
            );
        }

        // 2. Evaluate LogAttrs in a single pass over the LogAttrs RecordBatch.
        if !plan.log_attr_evaluators.is_empty()
            && let Some(rb) = view.records.get(ArrowPayloadType::LogAttrs)
        {
            let log_id_to_row = view.log_id_to_row();
            if !log_id_to_row.is_empty() {
                evaluate_attr_table(
                    plan,
                    rb,
                    &plan.log_attr_evaluators,
                    row_words,
                    &mut scratch.string_scratches,
                    &mut scratch.bitset_pool,
                    |parent_id| {
                        let &row = log_id_to_row.get(parent_id as usize)?;
                        if row == u32::MAX || active_rows.is_some_and(|m| !m.contains(row)) {
                            None
                        } else {
                            Some(row)
                        }
                    },
                    &mut pred_bitmaps,
                );
            }
        }

        // 3. Evaluate ResourceAttrs in resource-ID space (bitmap stores matching resource.id values).
        if !plan.res_attr_evaluators.is_empty()
            && let Some(rb) = view.records.get(ArrowPayloadType::ResourceAttrs)
        {
            evaluate_attr_table(
                plan,
                rb,
                &plan.res_attr_evaluators,
                1,
                &mut scratch.string_scratches,
                &mut scratch.bitset_pool,
                |parent_id| Some(u32::from(parent_id)),
                &mut pred_bitmaps,
            );
        }

        // 4. Evaluate ScopeAttrs in scope-ID space (bitmap stores matching scope.id values).
        if !plan.scope_attr_evaluators.is_empty()
            && let Some(rb) = view.records.get(ArrowPayloadType::ScopeAttrs)
        {
            evaluate_attr_table(
                plan,
                rb,
                &plan.scope_attr_evaluators,
                1,
                &mut scratch.string_scratches,
                &mut scratch.bitset_pool,
                |parent_id| Some(u32::from(parent_id)),
                &mut pred_bitmaps,
            );
        }

        pred_bitmaps
    }

    fn eval_row_conjunction<'a>(
        matchers: &[CompiledMatcherRef],
        pred_bitmaps: &'a [DenseBitset],
        candidate_rows: &DenseBitset,
        candidate_is_subset: bool,
        pool: &mut BitsetBufferPool,
    ) -> MatchedRows<'a> {
        let Some((first, rest)) = matchers.split_first() else {
            return MatchedRows::Owned(candidate_rows.clone_with_pool(pool));
        };
        if rest.is_empty() && !first.negate && !candidate_is_subset {
            return MatchedRows::Borrowed(&pred_bitmaps[first.pred_id]);
        }
        let mut acc = if first.negate {
            candidate_rows.sub_with_pool(&pred_bitmaps[first.pred_id], pool)
        } else if candidate_is_subset {
            pred_bitmaps[first.pred_id].and_with_pool(candidate_rows, pool)
        } else {
            pred_bitmaps[first.pred_id].clone_with_pool(pool)
        };
        for m in rest {
            if acc.is_empty() {
                break;
            }
            if m.negate {
                acc.sub_assign(&pred_bitmaps[m.pred_id]);
            } else {
                acc.and_assign(&pred_bitmaps[m.pred_id]);
            }
        }
        MatchedRows::Owned(acc)
    }

    fn eval_id_conjunction(
        matchers: &[CompiledMatcherRef],
        pred_bitmaps: &[DenseBitset],
        pool: &mut BitsetBufferPool,
    ) -> IdFilter {
        if matchers.is_empty() {
            return IdFilter::PassAll;
        }
        if let Some(first_pos_idx) = matchers.iter().position(|m| !m.negate) {
            let mut acc = pred_bitmaps[matchers[first_pos_idx].pred_id].clone_with_pool(pool);
            if acc.is_empty() {
                pool.recycle(acc);
                return IdFilter::Empty;
            }
            for (idx, m) in matchers.iter().enumerate() {
                if idx == first_pos_idx || m.negate {
                    continue;
                }
                acc.and_assign(&pred_bitmaps[m.pred_id]);
                if acc.is_empty() {
                    pool.recycle(acc);
                    return IdFilter::Empty;
                }
            }
            for m in matchers.iter().filter(|m| m.negate) {
                acc.sub_assign(&pred_bitmaps[m.pred_id]);
                if acc.is_empty() {
                    pool.recycle(acc);
                    return IdFilter::Empty;
                }
            }
            IdFilter::KeepIds(acc)
        } else {
            // All matchers are negated: (!N1 & !N2 & ...) == !(N1 | N2 | ...).
            let mut excluded = DenseBitset::new();
            for m in matchers {
                excluded.or_assign_with_pool(&pred_bitmaps[m.pred_id], pool);
            }
            if excluded.is_empty() {
                pool.recycle(excluded);
                IdFilter::PassAll
            } else {
                IdFilter::ExcludeIds(excluded)
            }
        }
    }

    #[inline]
    fn matches_id_filter(
        filter: &IdFilter,
        col: Option<(&StructArray, &UInt16Array)>,
        row: usize,
    ) -> bool {
        match filter {
            IdFilter::PassAll => true,
            IdFilter::Empty => false,
            IdFilter::KeepIds(ids) => col.is_some_and(|(s, arr)| {
                s.is_valid(row) && arr.is_valid(row) && ids.contains(u32::from(arr.value(row)))
            }),
            IdFilter::ExcludeIds(excluded) => !col.is_some_and(|(s, arr)| {
                s.is_valid(row) && arr.is_valid(row) && excluded.contains(u32::from(arr.value(row)))
            }),
        }
    }

    fn eval_rule_matchers<'a>(
        matchers: &CompiledRuleMatchers,
        pred_bitmaps: &'a [DenseBitset],
        view: &LogsBatchView<'_>,
        candidate_rows: &DenseBitset,
        candidate_is_subset: bool,
        pool: &mut BitsetBufferPool,
    ) -> MatchedRows<'a> {
        let res_filter = Self::eval_id_conjunction(&matchers.resource_matchers, pred_bitmaps, pool);
        if matches!(res_filter, IdFilter::Empty) {
            return MatchedRows::Owned(DenseBitset::new());
        }
        let scope_filter = Self::eval_id_conjunction(&matchers.scope_matchers, pred_bitmaps, pool);
        if matches!(scope_filter, IdFilter::Empty) {
            res_filter.recycle(pool);
            return MatchedRows::Owned(DenseBitset::new());
        }

        let res_needed = !matches!(res_filter, IdFilter::PassAll);
        let scope_needed = !matches!(scope_filter, IdFilter::PassAll);
        if !res_needed && !scope_needed {
            return Self::eval_row_conjunction(
                &matchers.row_matchers,
                pred_bitmaps,
                candidate_rows,
                candidate_is_subset,
                pool,
            );
        }

        let res_col = if res_needed {
            view.resource_id_column()
        } else {
            None
        };
        if matches!(res_filter, IdFilter::KeepIds(_)) && res_col.is_none() {
            res_filter.recycle(pool);
            scope_filter.recycle(pool);
            return MatchedRows::Owned(DenseBitset::new());
        }
        let scope_col = if scope_needed {
            view.scope_id_column()
        } else {
            None
        };
        if matches!(scope_filter, IdFilter::KeepIds(_)) && scope_col.is_none() {
            res_filter.recycle(pool);
            scope_filter.recycle(pool);
            return MatchedRows::Owned(DenseBitset::new());
        }

        let row_words = view.num_rows.div_ceil(64).max(1);
        let mut out = DenseBitset::new();
        if matchers.row_matchers.is_empty() {
            let active = candidate_is_subset.then_some(candidate_rows);
            for_each_active_row(view.num_rows, active, |row| {
                if Self::matches_id_filter(&res_filter, res_col, row)
                    && Self::matches_id_filter(&scope_filter, scope_col, row)
                {
                    out.insert_with_pool(row as u32, row_words, pool);
                }
            });
        } else {
            let row_candidates = Self::eval_row_conjunction(
                &matchers.row_matchers,
                pred_bitmaps,
                candidate_rows,
                candidate_is_subset,
                pool,
            );
            for row_u32 in row_candidates.as_bitset().iter() {
                let row = row_u32 as usize;
                if Self::matches_id_filter(&res_filter, res_col, row)
                    && Self::matches_id_filter(&scope_filter, scope_col, row)
                {
                    out.insert_with_pool(row_u32, row_words, pool);
                }
            }
            row_candidates.recycle(pool);
        }
        res_filter.recycle(pool);
        scope_filter.recycle(pool);
        MatchedRows::Owned(out)
    }

    fn evaluate_filter_view(
        &self,
        view: &LogsBatchView<'_>,
        scratch: &mut EvaluationScratch,
        mut stats: Option<&mut PolicyMatchStats>,
    ) -> DenseBitset {
        let num_rows_u64 = view.num_rows as u64;
        if let Some(s) = stats.as_deref_mut() {
            s.evaluated_rows += num_rows_u64;
        }
        let mut all_rows = DenseBitset::full_with_pool(view.num_rows, &mut scratch.bitset_pool);

        let short_circuit = stats.is_none();
        if !self.has_filter_rules() || (self.drop_rules.is_empty() && short_circuit) {
            return all_rows;
        }

        // 1. Evaluate Keep rules first so `keep_exempt` is known before running Drop rules.
        let mut keep_exempt = DenseBitset::new();
        if !self.keep_rules.is_empty() {
            let mut keep_bitmaps = self.evaluate_predicates(&self.keep_plan, view, scratch, None);
            let mut keep_all = false;
            for rule in &self.keep_rules {
                let matched = Self::eval_rule_matchers(
                    &rule.matchers,
                    &keep_bitmaps,
                    view,
                    &all_rows,
                    false,
                    &mut scratch.bitset_pool,
                );
                let matched_bm = matched.as_bitset();
                if !matched_bm.is_empty() {
                    if let Some(s) = stats.as_deref_mut() {
                        s.counts[rule.policy_idx].hits += matched_bm.len();
                    }
                    keep_exempt.or_assign_with_pool(matched_bm, &mut scratch.bitset_pool);
                    if short_circuit && keep_exempt.len() == num_rows_u64 {
                        keep_all = true;
                    }
                }
                matched.recycle(&mut scratch.bitset_pool);
                if keep_all {
                    // Every row is kept; skip remaining Keep rules and the entire Drop plan.
                    break;
                }
            }
            scratch.bitset_pool.recycle_vec(&mut keep_bitmaps);
            if keep_all {
                scratch.bitset_pool.recycle(keep_exempt);
                return all_rows;
            }
        }

        if self.drop_rules.is_empty() {
            scratch.bitset_pool.recycle(keep_exempt);
            return all_rows;
        }

        // 2. Evaluate Drop rules.
        if short_circuit {
            // Only evaluate Drop predicates on rows not already exempted by Keep rules.
            let mut undecided = if keep_exempt.is_empty() {
                all_rows
            } else {
                all_rows.sub_assign(&keep_exempt);
                all_rows
            };
            let active_mask = (!keep_exempt.is_empty()).then_some(&undecided);
            let mut drop_bitmaps =
                self.evaluate_predicates(&self.drop_plan, view, scratch, active_mask);
            for rule in &self.drop_rules {
                let is_subset = undecided.len() < num_rows_u64;
                let matched = Self::eval_rule_matchers(
                    &rule.matchers,
                    &drop_bitmaps,
                    view,
                    &undecided,
                    is_subset,
                    &mut scratch.bitset_pool,
                );
                let matched_bm = matched.as_bitset();
                if !matched_bm.is_empty() {
                    undecided.sub_assign(matched_bm);
                }
                matched.recycle(&mut scratch.bitset_pool);
                if undecided.is_empty() {
                    // Every remaining undecided row is already marked to drop.
                    break;
                }
            }
            scratch.bitset_pool.recycle_vec(&mut drop_bitmaps);
            if keep_exempt.is_empty() {
                scratch.bitset_pool.recycle(keep_exempt);
                undecided
            } else {
                keep_exempt.or_assign_with_pool(&undecided, &mut scratch.bitset_pool);
                scratch.bitset_pool.recycle(undecided);
                keep_exempt
            }
        } else {
            // Exact per-policy stats mode: evaluate Drop predicates across all rows so
            // overridden Drop rules record exact `miss` counts against `keep_exempt`.
            let mut drop_bitmaps = self.evaluate_predicates(&self.drop_plan, view, scratch, None);
            let mut drop_matched = DenseBitset::new();
            for rule in &self.drop_rules {
                let matched = Self::eval_rule_matchers(
                    &rule.matchers,
                    &drop_bitmaps,
                    view,
                    &all_rows,
                    false,
                    &mut scratch.bitset_pool,
                );
                let matched_bm = matched.as_bitset();
                if !matched_bm.is_empty() {
                    if let Some(s) = stats.as_deref_mut() {
                        let misses = if keep_exempt.is_empty() {
                            0
                        } else {
                            matched_bm.intersection_len(&keep_exempt)
                        };
                        let hits = matched_bm.len() - misses;
                        s.counts[rule.policy_idx].hits += hits;
                        s.counts[rule.policy_idx].misses += misses;
                    }
                    drop_matched.or_assign_with_pool(matched_bm, &mut scratch.bitset_pool);
                }
                matched.recycle(&mut scratch.bitset_pool);
            }
            scratch.bitset_pool.recycle_vec(&mut drop_bitmaps);
            drop_matched.sub_assign(&keep_exempt);
            all_rows.sub_assign(&drop_matched);
            scratch.bitset_pool.recycle(keep_exempt);
            scratch.bitset_pool.recycle(drop_matched);
            all_rows
        }
    }

    fn compute_transform_masks(
        &self,
        pred_bitmaps: &[DenseBitset],
        view: &LogsBatchView<'_>,
        pool: &mut BitsetBufferPool,
    ) -> Vec<DenseBitset> {
        let all_rows = DenseBitset::full_with_pool(view.num_rows, pool);

        let masks = self
            .transform_rules
            .iter()
            .map(|rule| {
                Self::eval_rule_matchers(&rule.matchers, pred_bitmaps, view, &all_rows, false, pool)
                    .into_owned(pool)
            })
            .collect();
        pool.recycle(all_rows);
        masks
    }

    fn evaluate_filter_normalized(
        &self,
        batch: &OtapArrowRecords,
        scratch: &mut EvaluationScratch,
        stats: Option<&mut PolicyMatchStats>,
    ) -> Result<DenseBitset, PolicyError> {
        let Some(view) = LogsBatchView::try_new(batch)? else {
            return Ok(DenseBitset::new());
        };
        Ok(self.evaluate_filter_view(&view, scratch, stats))
    }

    fn apply_transforms_to_batch(
        &self,
        out_batch: &mut OtapArrowRecords,
        scratch: &mut EvaluationScratch,
        stats: Option<&mut PolicyMatchStats>,
    ) -> Result<(), PolicyError> {
        if self.compiled_transforms.is_empty() {
            return Ok(());
        }
        let Some(view) = LogsBatchView::try_new(out_batch)? else {
            return Ok(());
        };
        let mut pred_bitmaps = self.evaluate_predicates(&self.transform_plan, &view, scratch, None);
        let mut tx_masks =
            self.compute_transform_masks(&pred_bitmaps, &view, &mut scratch.bitset_pool);
        scratch.bitset_pool.recycle_vec(&mut pred_bitmaps);
        apply_spec_transforms_masked(out_batch, &self.compiled_transforms, &tx_masks)?;
        if let Some(s) = stats {
            for (rule, mask) in self.transform_rules.iter().zip(&tx_masks) {
                s.counts[rule.policy_idx].hits += mask.len();
            }
        }
        scratch.bitset_pool.recycle_vec(&mut tx_masks);
        Ok(())
    }

    /// Evaluates filter policies against a `Logs` batch using reusable `scratch` and returns the
    /// kept row indices (`0..num_rows`).
    ///
    /// When `stats` is `None`, enables short-circuit evaluation (skipping `Keep` plans when no
    /// `Drop` rules exist, skipping `Drop` plans when `Keep` covers all rows, gating `Drop`
    /// predicates to undecided rows, and stopping once all undecided rows are dropped). When
    /// `stats` is `Some`, evaluates all rules and accumulates `evaluated_rows` and per-policy
    /// filter `hit`/`miss` outcomes. Returns [`PolicyError::Evaluation`] if `batch` is not
    /// [`OtapArrowRecords::Logs`] or if `stats` was created for a different compiled
    /// [`PolicyEngine`].
    pub fn evaluate_filter(
        &self,
        batch: &OtapArrowRecords,
        scratch: &mut EvaluationScratch,
        stats: Option<&mut PolicyMatchStats>,
    ) -> Result<RowSelection, PolicyError> {
        if !matches!(batch, OtapArrowRecords::Logs(_)) {
            return Err(PolicyError::Evaluation(
                "PolicyEngine currently only supports Logs batches".into(),
            ));
        }
        if let Some(s) = stats.as_deref() {
            self.validate_match_stats(s)?;
        }
        let num_rows = batch
            .get(ArrowPayloadType::Logs)
            .map_or(0, RecordBatch::num_rows);
        if num_rows == 0 {
            return Ok(RowSelection {
                bitmap: DenseBitset::new(),
            });
        }
        if !self.has_filter_rules() || (self.drop_rules.is_empty() && stats.is_none()) {
            if let Some(s) = stats {
                s.evaluated_rows += num_rows as u64;
            }
            return Ok(RowSelection {
                bitmap: DenseBitset::full(num_rows),
            });
        }

        // Root-only filter plans do not read parent/entity IDs, avoiding batch clone and ID decoding.
        if !self.filter_uses_attributes() {
            let bitmap = self.evaluate_filter_normalized(batch, scratch, stats)?;
            return Ok(RowSelection { bitmap });
        }

        let mut normalized = batch.clone();
        normalized
            .decode_transport_optimized_ids()
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        let bitmap = self.evaluate_filter_normalized(&normalized, scratch, stats)?;
        Ok(RowSelection { bitmap })
    }

    /// Evaluates filter and transform policies against a `Logs` batch using reusable `scratch` and
    /// returns the filtered/transformed [`EvaluationOutput`].
    ///
    /// Takes `batch` by shared reference so callers retain ownership of the original payload to
    /// nack on error. When `stats` is `Some`, accumulates `evaluated_rows` and per-policy
    /// `hit`/`miss` outcomes across filter and transform stages. Returns
    /// [`PolicyError::Evaluation`] if `batch` is not [`OtapArrowRecords::Logs`] or if `stats` was
    /// created for a different compiled [`PolicyEngine`].
    pub fn evaluate(
        &self,
        batch: &OtapArrowRecords,
        scratch: &mut EvaluationScratch,
        mut stats: Option<&mut PolicyMatchStats>,
    ) -> Result<EvaluationOutput, PolicyError> {
        if !matches!(batch, OtapArrowRecords::Logs(_)) {
            return Err(PolicyError::Evaluation(
                "PolicyEngine currently only supports Logs batches".into(),
            ));
        }
        if let Some(s) = stats.as_deref() {
            self.validate_match_stats(s)?;
        }
        let total_rows = batch
            .get(ArrowPayloadType::Logs)
            .map_or(0, RecordBatch::num_rows);
        if total_rows == 0 {
            return Ok(EvaluationOutput {
                records: Some(batch.clone()),
                kept_rows: 0,
                dropped_rows: 0,
            });
        }

        let skip_filter =
            !self.has_filter_rules() || (self.drop_rules.is_empty() && stats.is_none());

        // Pure passthrough: no filter work and no transforms; skip transport-ID decoding entirely.
        if skip_filter && self.compiled_transforms.is_empty() {
            if let Some(s) = stats {
                s.evaluated_rows += total_rows as u64;
            }
            return Ok(EvaluationOutput {
                records: Some(batch.clone()),
                kept_rows: total_rows as u64,
                dropped_rows: 0,
            });
        }

        // Fast path when no filter work is needed, only transforms.
        if skip_filter {
            let mut out_batch = batch.clone();
            out_batch
                .decode_transport_optimized_ids()
                .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
            self.apply_transforms_to_batch(&mut out_batch, scratch, stats.as_deref_mut())?;
            if let Some(s) = stats {
                s.evaluated_rows += total_rows as u64;
            }
            return Ok(EvaluationOutput {
                records: Some(out_batch),
                kept_rows: total_rows as u64,
                dropped_rows: 0,
            });
        }

        // Filter stage: defer clone + transport-ID decoding when filter is root-only and no
        // partial row filtering or transform stage needs decoded parent IDs.
        let (keep_bitmap, normalized_opt) = if !self.filter_uses_attributes() {
            let bm = self.evaluate_filter_normalized(batch, scratch, stats.as_deref_mut())?;
            (bm, None)
        } else {
            let mut normalized = batch.clone();
            normalized
                .decode_transport_optimized_ids()
                .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
            let bm = self.evaluate_filter_normalized(&normalized, scratch, stats.as_deref_mut())?;
            (bm, Some(normalized))
        };

        let kept_rows = keep_bitmap.len();
        let dropped_rows = (total_rows as u64).saturating_sub(kept_rows);
        if kept_rows == 0 {
            scratch.bitset_pool.recycle(keep_bitmap);
            return Ok(EvaluationOutput {
                records: None,
                kept_rows: 0,
                dropped_rows: total_rows as u64,
            });
        }

        if kept_rows == total_rows as u64 && self.compiled_transforms.is_empty() {
            scratch.bitset_pool.recycle(keep_bitmap);
            let out_batch = normalized_opt.unwrap_or_else(|| batch.clone());
            return Ok(EvaluationOutput {
                records: Some(out_batch),
                kept_rows,
                dropped_rows: 0,
            });
        }

        let normalized = match normalized_opt {
            Some(n) => n,
            None => {
                let mut n = batch.clone();
                n.decode_transport_optimized_ids()
                    .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
                n
            }
        };

        let mut filtered_batch = if kept_rows == total_rows as u64 {
            normalized
        } else {
            filter_otap_logs_by_bitmap(
                &normalized,
                &keep_bitmap,
                total_rows,
                &mut scratch.id_bitmap_pool,
            )?
        };
        scratch.bitset_pool.recycle(keep_bitmap);

        self.apply_transforms_to_batch(&mut filtered_batch, scratch, stats)?;

        Ok(EvaluationOutput {
            records: Some(filtered_batch),
            kept_rows,
            dropped_rows,
        })
    }
}
