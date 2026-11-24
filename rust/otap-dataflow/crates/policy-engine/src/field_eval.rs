// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-field columnar evaluation for root log fields, structured `logs.body`,
//! and OTAP attribute tables (`LogAttrs`, `ResourceAttrs`, `ScopeAttrs`).

use std::borrow::Cow;

use ahash::AHashMap;
use arrow::array::{Array, ArrayRef, RecordBatch, StringArray, StructArray};
use arrow::buffer::NullBuffer;
use arrow::compute::kernels::cmp;
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::otlp::common::AnyValueArrays;
use otel_arrow_dfe_pdata::schema::consts;

use crate::bitset::{BitsetBufferPool, DenseBitset};
use crate::columnar::{
    ATTR_SELECTIVITY_DENOMINATOR, MIN_COLUMNAR_ATTR_ROWS, SMALL_DICT_STR_EQ_THRESHOLD,
    SMALL_PLAIN_STR_EQ_THRESHOLD, byte_accessor_as_array, eval_anyval_body_exists_mask,
    eval_anyval_bool_pred_into, eval_anyval_bytes_eq_into, eval_anyval_double_pred_into,
    eval_anyval_int_pred_into, eval_anyval_str_eq_into, eval_anyval_type_mask,
    eval_fixed_id_columnar, eval_severity_number_columnar, eval_str_exists_bitmap, is_numeric_pred,
    maybe_dict_as_array,
};
use crate::model::{LogFieldSelector, LogRecordField, Predicate, ScalarValue, ScopeField};
use crate::otap_ops::{
    CborPathResult, ExtractedValue, LogsBatchView, ParentIdColumn, StrColAccessor, byte_slice_at,
    extract_anyval, extract_cbor_path_for_eval, extract_cbor_path_status,
};
use crate::string_index::{FieldStringIndex, StringScanScratch};

/// Controls how empty strings (`""`) are treated on a root string column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum EmptyStrMode {
    /// Empty string is treated as absent for `Exists` only (plain-string `LogRecord.body`).
    AbsentForExistsOnly,
    /// Empty string is treated as absent for all predicates (`SeverityText`, `EventName`, `ScopeField`).
    AbsentAll,
}

/// Single compiled field evaluator covering all predicates inspecting a specific `LogFieldSelector`.
#[derive(Debug, Clone)]
pub(crate) struct FieldEvaluator {
    pub(crate) field: LogFieldSelector,
    /// Deduplicated predicate ID for `Predicate::Exists`.
    pub(crate) exists_pred: Option<usize>,
    /// O(1) hash map for `Predicate::Equals(ScalarValue::String(s))` (and wildcard-free `Glob`) -> predicate ID.
    pub(crate) str_equals_map: AHashMap<String, usize>,
    /// Pre-decoded 16-byte raw `TraceId` exact-match literals (`policy.md` "Identifier Fields").
    pub(crate) trace_id_equals_map: AHashMap<[u8; 16], usize>,
    /// Pre-decoded 8-byte raw `SpanId` exact-match literals (`policy.md` "Identifier Fields").
    pub(crate) span_id_equals_map: AHashMap<[u8; 8], usize>,
    /// Predicate IDs on `TraceId` / `SpanId` that match `""` when the identifier is absent/null/all-zero.
    pub(crate) empty_id_preds: Vec<usize>,
    /// N-gram inverted index + verifiers for string `Regex`, `Glob`, and `Contains(String)`.
    pub(crate) string_index: FieldStringIndex,
    /// `(pred_id, ScalarValue)` for `Predicate::Contains` (needed on `Array` / `Bytes` values).
    pub(crate) contains_preds: Vec<(usize, ScalarValue)>,
    /// Non-string `Equals` and numeric range (`Gt`, `Gte`, `Lt`, `Lte`) predicates.
    pub(crate) scalar_and_range_preds: Vec<(usize, Predicate)>,
}

impl FieldEvaluator {
    #[inline]
    fn eval_str_only<F>(&self, s: &str, scratch: &mut StringScanScratch, mut on_match: F)
    where
        F: FnMut(usize),
    {
        if let Some(pid) = self.exists_pred {
            on_match(pid);
        }
        self.eval_str_value_preds(s, scratch, on_match);
    }

    #[inline]
    fn eval_str_value_preds<F>(&self, s: &str, scratch: &mut StringScanScratch, mut on_match: F)
    where
        F: FnMut(usize),
    {
        if let Some(&pid) = self.str_equals_map.get(s) {
            on_match(pid);
        }
        if !self.string_index.is_empty() {
            self.string_index.evaluate_str(s, scratch, on_match);
        }
    }

    #[inline]
    fn needs_array_elements(&self) -> bool {
        !self.contains_preds.is_empty()
    }

    #[inline]
    fn has_str_value_preds(&self) -> bool {
        !self.str_equals_map.is_empty() || !self.string_index.is_empty()
    }

    #[inline]
    fn has_numeric_preds(&self) -> bool {
        self.scalar_and_range_preds
            .iter()
            .any(|(_, p)| is_numeric_pred(p))
    }

    #[inline]
    fn has_bool_preds(&self) -> bool {
        self.scalar_and_range_preds
            .iter()
            .any(|(_, p)| matches!(p, Predicate::Equals(ScalarValue::Bool(_))))
    }

    #[inline]
    fn has_bytes_eq_preds(&self) -> bool {
        self.scalar_and_range_preds
            .iter()
            .any(|(_, p)| matches!(p, Predicate::Equals(ScalarValue::Bytes(_))))
    }

    #[inline]
    fn has_bytes_contains_preds(&self) -> bool {
        self.contains_preds
            .iter()
            .any(|(_, s)| matches!(s, ScalarValue::Bytes(_)))
    }

    #[inline]
    fn is_exists_only(&self) -> bool {
        self.str_equals_map.is_empty()
            && self.trace_id_equals_map.is_empty()
            && self.span_id_equals_map.is_empty()
            && self.empty_id_preds.is_empty()
            && self.string_index.is_empty()
            && self.scalar_and_range_preds.is_empty()
            && self.contains_preds.is_empty()
    }

    #[inline]
    fn eval_cbor_path_result<F>(
        &self,
        res: CborPathResult<'_>,
        scratch: &mut StringScanScratch,
        mut on_match: F,
    ) where
        F: FnMut(usize),
    {
        match res {
            CborPathResult::Present(val) => self.eval_extracted_value(&val, scratch, on_match),
            CborPathResult::PresentEmpty => {
                if let Some(pid) = self.exists_pred {
                    on_match(pid);
                }
            }
            CborPathResult::Absent => {}
        }
    }

    #[inline]
    fn eval_extracted_value<F>(
        &self,
        val: &ExtractedValue<'_>,
        scratch: &mut StringScanScratch,
        mut on_match: F,
    ) where
        F: FnMut(usize),
    {
        if let Some(pid) = self.exists_pred {
            on_match(pid);
        }
        self.eval_extracted_value_without_exists(val, scratch, on_match);
    }

    #[inline]
    fn eval_extracted_value_without_exists<F>(
        &self,
        val: &ExtractedValue<'_>,
        scratch: &mut StringScanScratch,
        mut on_match: F,
    ) where
        F: FnMut(usize),
    {
        match val {
            ExtractedValue::Str(s) => {
                if let Some(&pid) = self.str_equals_map.get(s.as_ref()) {
                    on_match(pid);
                }
                if !self.string_index.is_empty() {
                    self.string_index
                        .evaluate_str(s.as_ref(), scratch, &mut on_match);
                }
            }
            ExtractedValue::Int(v) => {
                for (pid, pred) in &self.scalar_and_range_preds {
                    let matched = match pred {
                        Predicate::Equals(target) => val.equals_scalar(target),
                        Predicate::Gt(nv)
                        | Predicate::Gte(nv)
                        | Predicate::Lt(nv)
                        | Predicate::Lte(nv) => pred.matches_ordering(nv.cmp_candidate_i64(*v)),
                        _ => false,
                    };
                    if matched {
                        on_match(*pid);
                    }
                }
            }
            ExtractedValue::Double(v) => {
                for (pid, pred) in &self.scalar_and_range_preds {
                    let matched = match pred {
                        Predicate::Equals(target) => val.equals_scalar(target),
                        Predicate::Gt(nv)
                        | Predicate::Gte(nv)
                        | Predicate::Lt(nv)
                        | Predicate::Lte(nv) => pred.matches_ordering(nv.cmp_candidate_f64(*v)),
                        _ => false,
                    };
                    if matched {
                        on_match(*pid);
                    }
                }
            }
            ExtractedValue::Bool(_) => {
                for (pid, pred) in &self.scalar_and_range_preds {
                    if let Predicate::Equals(target) = pred
                        && val.equals_scalar(target)
                    {
                        on_match(*pid);
                    }
                }
            }
            ExtractedValue::Bytes(_) => {
                for (pid, pred) in &self.scalar_and_range_preds {
                    if let Predicate::Equals(target) = pred
                        && val.equals_scalar(target)
                    {
                        on_match(*pid);
                    }
                }
                for (pid, target) in &self.contains_preds {
                    if val.contains_scalar(target) {
                        on_match(*pid);
                    }
                }
            }
            ExtractedValue::Array(_) => {
                for (pid, target) in &self.contains_preds {
                    if val.contains_scalar(target) {
                        on_match(*pid);
                    }
                }
            }
            ExtractedValue::Map(_) => {}
        }
    }
}

/// Compiled predicate evaluation plan for a single pipeline stage (keep, drop, or transform).
#[derive(Debug, Clone)]
pub(crate) struct StagePlan {
    pub(crate) num_predicates: usize,
    pub(crate) field_evaluators: Vec<FieldEvaluator>,
    /// Indices into `field_evaluators` for top-level / `BodyField` selectors.
    pub(crate) root_evaluator_indices: Vec<usize>,
    /// Keyed indices into `field_evaluators` for `LogAttribute(path)` keyed by `path[0]`.
    pub(crate) log_attr_evaluators: AHashMap<String, Vec<usize>>,
    /// Keyed indices into `field_evaluators` for `ResourceAttribute(path)` keyed by `path[0]`.
    pub(crate) res_attr_evaluators: AHashMap<String, Vec<usize>>,
    /// Keyed indices into `field_evaluators` for `ScopeAttribute(path)` keyed by `path[0]`.
    pub(crate) scope_attr_evaluators: AHashMap<String, Vec<usize>>,
}

impl StagePlan {
    /// Returns `true` if this stage inspects any OTAP attribute table (`LogAttrs`, `ResourceAttrs`,
    /// or `ScopeAttrs`), which requires decoded parent/entity IDs.
    #[inline]
    pub(crate) fn uses_attributes(&self) -> bool {
        !self.log_attr_evaluators.is_empty()
            || !self.res_attr_evaluators.is_empty()
            || !self.scope_attr_evaluators.is_empty()
    }
}

/// Pre-resolved dictionary-encoded string view backed by [`StrColAccessor`].
#[derive(Clone, Copy)]
struct DictStrView<'a> {
    acc: StrColAccessor<'a>,
    values: &'a StringArray,
}

impl<'a> DictStrView<'a> {
    fn from_accessor(acc: StrColAccessor<'a>) -> Option<Self> {
        let values = acc.dict_values()?;
        Some(Self { acc, values })
    }

    fn try_from_array(col: &'a ArrayRef) -> Option<Self> {
        Self::from_accessor(StrColAccessor::try_from_dict_array(col)?)
    }

    #[inline]
    fn dict_len(&self) -> usize {
        self.values.len()
    }

    #[inline]
    fn valid_dict_idx(&self, row: usize) -> Option<usize> {
        self.acc.valid_dict_idx(row)
    }
}

/// Pre-resolved OTAP `AnyValue` columns for a `logs.body` `StructArray` or attribute `RecordBatch`.
struct AnyValColumns<'a> {
    struct_validity: Option<&'a NullBuffer>,
    arrays: AnyValueArrays<'a>,
}

impl<'a> AnyValColumns<'a> {
    fn from_struct_array(sa: &'a StructArray) -> Option<Self> {
        let arrays = AnyValueArrays::try_from(sa).ok()?;
        Some(Self {
            struct_validity: sa.nulls(),
            arrays,
        })
    }

    fn from_attr_batch(rb: &'a RecordBatch) -> Option<Self> {
        let arrays = AnyValueArrays::try_from(rb).ok()?;
        Some(Self {
            struct_validity: None,
            arrays,
        })
    }

    fn dict_str_view(&self) -> Option<DictStrView<'a>> {
        let acc = StrColAccessor::from_maybe_dict(self.arrays.attr_str.as_ref()?);
        DictStrView::from_accessor(acc)
    }

    #[inline]
    fn is_str_row(&self, row: usize) -> bool {
        if let Some(nulls) = self.struct_validity
            && !nulls.is_valid(row)
        {
            return false;
        }
        self.arrays.attr_type.is_valid(row)
            && self.arrays.attr_type.value(row) == AttributeValueType::Str as u8
    }

    #[inline]
    fn cbor_row_kind(&self, row: usize) -> Option<(AttributeValueType, Option<&'a [u8]>)> {
        if let Some(nulls) = self.struct_validity
            && !nulls.is_valid(row)
        {
            return None;
        }
        if !self.arrays.attr_type.is_valid(row) {
            return None;
        }
        let vtype = match AttributeValueType::try_from(self.arrays.attr_type.value(row)) {
            Ok(vt @ (AttributeValueType::Map | AttributeValueType::Slice)) => vt,
            _ => return None,
        };
        let raw = self
            .arrays
            .attr_ser
            .as_ref()
            .and_then(|c| byte_slice_at(c, row));
        Some((vtype, raw))
    }

    #[inline]
    fn extract_at_row(&self, row: usize) -> Option<ExtractedValue<'a>> {
        if let Some(nulls) = self.struct_validity
            && !nulls.is_valid(row)
        {
            return None;
        }
        extract_anyval(&self.arrays, row)
    }
}

const UNSEEN_DICT_SLOT: u32 = u32::MAX;
const DIRECT_DICT_CACHE_LIMIT: usize = 1024;

enum DictEvalStorage {
    /// Direct-indexed slots for small dictionaries (`len <= DIRECT_DICT_CACHE_LIMIT`).
    /// Uncached slots store `(UNSEEN_DICT_SLOT, 0)`; cached slots store `(start, len)` into `matched_preds`.
    Direct(Vec<(u32, u32)>),
    /// Sparse cache for large `UInt16` dictionaries so per-field caches scale with distinct visited values.
    Sparse(AHashMap<usize, (u32, u32)>),
}

/// Lazy per-dictionary-value memoization cache for evaluating a single [`FieldEvaluator`]
/// across a dictionary-encoded string column.
struct DictStrEvalCache {
    storage: DictEvalStorage,
    matched_preds: Vec<usize>,
}

impl DictStrEvalCache {
    fn new(dict_len: usize) -> Self {
        let storage = if dict_len <= DIRECT_DICT_CACHE_LIMIT {
            DictEvalStorage::Direct(vec![(UNSEEN_DICT_SLOT, 0); dict_len])
        } else {
            DictEvalStorage::Sparse(AHashMap::new())
        };
        Self {
            storage,
            matched_preds: Vec::new(),
        }
    }

    /// Evaluates dictionary entry `dict_idx` at most once and returns its matched predicate IDs.
    #[inline]
    fn eval_or_get(
        &mut self,
        dict_idx: usize,
        dict_values: &StringArray,
        skip_empty: bool,
        include_exists: bool,
        ev: &FieldEvaluator,
        scratch: &mut StringScanScratch,
    ) -> &[usize] {
        let cached = match &self.storage {
            DictEvalStorage::Direct(slots) => {
                let slot = slots[dict_idx];
                (slot.0 != UNSEEN_DICT_SLOT).then_some(slot)
            }
            DictEvalStorage::Sparse(map) => map.get(&dict_idx).copied(),
        };
        if let Some((start, len)) = cached {
            let s = start as usize;
            return &self.matched_preds[s..s + len as usize];
        }

        let start = self.matched_preds.len();
        let val = dict_values.value(dict_idx);
        if !skip_empty || !val.is_empty() {
            if include_exists {
                ev.eval_str_only(val, scratch, |pid| {
                    self.matched_preds.push(pid);
                });
            } else {
                ev.eval_str_value_preds(val, scratch, |pid| {
                    self.matched_preds.push(pid);
                });
            }
        }
        let end = self.matched_preds.len();
        let slot = (start as u32, (end - start) as u32);
        match &mut self.storage {
            DictEvalStorage::Direct(slots) => slots[dict_idx] = slot,
            DictEvalStorage::Sparse(map) => {
                let _ = map.insert(dict_idx, slot);
            }
        }
        &self.matched_preds[start..end]
    }
}

/// Lazy sparse per-dictionary-key cache for huge `UInt16` attribute key dictionaries
/// (`dict_len > DIRECT_DICT_CACHE_LIMIT`).
struct DictKeyLookupCache<'a> {
    map: AHashMap<usize, Option<&'a [usize]>>,
}

impl<'a> DictKeyLookupCache<'a> {
    fn new() -> Self {
        Self {
            map: AHashMap::new(),
        }
    }

    #[inline]
    fn get_or_lookup(
        &mut self,
        dict_idx: usize,
        dict_values: &StringArray,
        key_map: &'a AHashMap<String, Vec<usize>>,
    ) -> Option<&'a [usize]> {
        *self
            .map
            .entry(dict_idx)
            .or_insert_with(|| key_map.get(dict_values.value(dict_idx)).map(Vec::as_slice))
    }
}

#[inline]
fn attr_path_segments(field: &LogFieldSelector) -> Option<&[String]> {
    match field {
        LogFieldSelector::LogAttribute(p)
        | LogFieldSelector::ResourceAttribute(p)
        | LogFieldSelector::ScopeAttribute(p) => Some(&p.path),
        _ => None,
    }
}

#[inline]
pub(crate) fn for_each_active_row(
    num_rows: usize,
    active_rows: Option<&DenseBitset>,
    mut f: impl FnMut(usize),
) {
    if let Some(mask) = active_rows {
        for row_u32 in mask.iter() {
            f(row_u32 as usize);
        }
    } else {
        for row in 0..num_rows {
            f(row);
        }
    }
}

fn emit_empty_id_matches(
    empty_id_preds: &[usize],
    num_rows: usize,
    active_rows: Option<&DenseBitset>,
    bitset_pool: &mut BitsetBufferPool,
    pred_bitmaps: &mut [DenseBitset],
) {
    if empty_id_preds.is_empty() || num_rows == 0 || active_rows.is_some_and(DenseBitset::is_empty)
    {
        return;
    }
    let absent = match active_rows {
        Some(act) => act.clone_with_pool(bitset_pool),
        None => DenseBitset::full_with_pool(num_rows, bitset_pool),
    };
    for &pid in empty_id_preds {
        pred_bitmaps[pid].or_assign_with_pool(&absent, bitset_pool);
    }
    bitset_pool.recycle(absent);
}

#[allow(clippy::too_many_arguments)]
fn eval_str_column(
    ev: &FieldEvaluator,
    col: &ArrayRef,
    parent_validity: Option<&NullBuffer>,
    num_rows: usize,
    row_words: usize,
    active_rows: Option<&DenseBitset>,
    empty_mode: EmptyStrMode,
    scratch: &mut StringScanScratch,
    bitset_pool: &mut BitsetBufferPool,
    pred_bitmaps: &mut [DenseBitset],
) {
    let skip_empty_exists = matches!(
        empty_mode,
        EmptyStrMode::AbsentForExistsOnly | EmptyStrMode::AbsentAll
    );
    let skip_empty_values = matches!(empty_mode, EmptyStrMode::AbsentAll);

    // 1. Validity-bitmap / SIMD `Exists` fast path (avoids per-row scanning for `Exists`).
    if let Some(pid) = ev.exists_pred {
        let exists_mask = eval_str_exists_bitmap(
            col,
            parent_validity,
            num_rows,
            active_rows,
            skip_empty_exists,
            bitset_pool,
        );
        pred_bitmaps[pid].or_assign_with_pool(&exists_mask, bitset_pool);
        bitset_pool.recycle(exists_mask);
        if !ev.has_str_value_preds() {
            return;
        }
    }

    let dict_opt = DictStrView::try_from_array(col);
    let plain_opt = col.as_any().downcast_ref::<StringArray>();

    // 2. Small exact-match sets run faster via Arrow SIMD `cmp::eq` than per-row hashing / caching.
    let eq_threshold = if dict_opt.is_some() {
        SMALL_DICT_STR_EQ_THRESHOLD
    } else {
        SMALL_PLAIN_STR_EQ_THRESHOLD
    };
    if ev.string_index.is_empty()
        && ev.str_equals_map.len() <= eq_threshold
        && (dict_opt.is_some() || plain_opt.is_some())
    {
        let parent_mask = parent_validity.map(|pnulls| {
            DenseBitset::from_validity_buffers(
                num_rows,
                None,
                Some(pnulls),
                active_rows,
                bitset_pool,
            )
        });
        let eff_mask = parent_mask.as_ref().or(active_rows);
        for (target, &pid) in &ev.str_equals_map {
            if skip_empty_values && target.is_empty() {
                continue;
            }
            if let Ok(eq_arr) = cmp::eq(col, &StringArray::new_scalar(target.as_str())) {
                pred_bitmaps[pid].or_boolean_array_with_pool(&eq_arr, false, eff_mask, bitset_pool);
            }
        }
        if let Some(pm) = parent_mask {
            bitset_pool.recycle(pm);
        }
        return;
    }

    // 3. Dictionary-memoized or plain single-pass evaluation for regex / glob / contains / large `Equals`.
    if let Some(dict) = dict_opt {
        let mut cache = DictStrEvalCache::new(dict.dict_len());
        for_each_active_row(num_rows, active_rows, |row| {
            if parent_validity.is_some_and(|n| !n.is_valid(row)) {
                return;
            }
            if let Some(dict_idx) = dict.valid_dict_idx(row) {
                let matched =
                    cache.eval_or_get(dict_idx, dict.values, skip_empty_values, false, ev, scratch);
                if !matched.is_empty() {
                    let row_u32 = row as u32;
                    for &pid in matched {
                        pred_bitmaps[pid].insert_with_pool(row_u32, row_words, bitset_pool);
                    }
                }
            }
        });
        return;
    }

    if let Some(sa) = plain_opt {
        for_each_active_row(num_rows, active_rows, |row| {
            if parent_validity.is_some_and(|n| !n.is_valid(row)) || !sa.is_valid(row) {
                return;
            }
            let s = sa.value(row);
            if !skip_empty_values || !s.is_empty() {
                let row_u32 = row as u32;
                ev.eval_str_value_preds(s, scratch, |pid| {
                    pred_bitmaps[pid].insert_with_pool(row_u32, row_words, bitset_pool);
                });
            }
        });
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_root_field(
    ev: &FieldEvaluator,
    view: &LogsBatchView<'_>,
    row_words: usize,
    active_rows: Option<&DenseBitset>,
    scratch: &mut StringScanScratch,
    bitset_pool: &mut BitsetBufferPool,
    pred_bitmaps: &mut [DenseBitset],
) {
    let num_rows = view.num_rows;

    match &ev.field {
        LogFieldSelector::RecordField(rf) => match rf {
            LogRecordField::Unspecified => {}
            LogRecordField::Body => {
                let Some(body_col) = view.root.column_by_name(consts::BODY) else {
                    return;
                };
                if let Some(body_struct) = body_col.as_any().downcast_ref::<StructArray>() {
                    evaluate_body_struct(
                        ev,
                        body_struct,
                        num_rows,
                        row_words,
                        active_rows,
                        scratch,
                        bitset_pool,
                        pred_bitmaps,
                    );
                } else {
                    eval_str_column(
                        ev,
                        body_col,
                        None,
                        num_rows,
                        row_words,
                        active_rows,
                        EmptyStrMode::AbsentForExistsOnly,
                        scratch,
                        bitset_pool,
                        pred_bitmaps,
                    );
                }
            }
            LogRecordField::SeverityText => {
                if let Some(col) = view.root.column_by_name(consts::SEVERITY_TEXT) {
                    eval_str_column(
                        ev,
                        col,
                        None,
                        num_rows,
                        row_words,
                        active_rows,
                        EmptyStrMode::AbsentAll,
                        scratch,
                        bitset_pool,
                        pred_bitmaps,
                    );
                }
            }
            LogRecordField::EventName => {
                if let Some(col) = view.root.column_by_name(consts::EVENT_NAME) {
                    eval_str_column(
                        ev,
                        col,
                        None,
                        num_rows,
                        row_words,
                        active_rows,
                        EmptyStrMode::AbsentAll,
                        scratch,
                        bitset_pool,
                        pred_bitmaps,
                    );
                }
            }
            LogRecordField::SeverityNumber => {
                if let Some(col) = view.root.column_by_name(consts::SEVERITY_NUMBER) {
                    eval_severity_number_columnar(
                        col,
                        ev.exists_pred,
                        &ev.scalar_and_range_preds,
                        active_rows,
                        bitset_pool,
                        pred_bitmaps,
                    );
                }
            }
            LogRecordField::TraceId => {
                if let Some(col) = view.root.column_by_name(consts::TRACE_ID) {
                    eval_fixed_id_columnar::<16>(
                        col,
                        ev.exists_pred,
                        &ev.trace_id_equals_map,
                        &ev.empty_id_preds,
                        !ev.string_index.is_empty(),
                        active_rows,
                        row_words,
                        bitset_pool,
                        pred_bitmaps,
                        |hex_str, on_match| {
                            ev.string_index.evaluate_str(hex_str, scratch, on_match);
                        },
                    );
                } else {
                    emit_empty_id_matches(
                        &ev.empty_id_preds,
                        num_rows,
                        active_rows,
                        bitset_pool,
                        pred_bitmaps,
                    );
                }
            }
            LogRecordField::SpanId => {
                if let Some(col) = view.root.column_by_name(consts::SPAN_ID) {
                    eval_fixed_id_columnar::<8>(
                        col,
                        ev.exists_pred,
                        &ev.span_id_equals_map,
                        &ev.empty_id_preds,
                        !ev.string_index.is_empty(),
                        active_rows,
                        row_words,
                        bitset_pool,
                        pred_bitmaps,
                        |hex_str, on_match| {
                            ev.string_index.evaluate_str(hex_str, scratch, on_match);
                        },
                    );
                } else {
                    emit_empty_id_matches(
                        &ev.empty_id_preds,
                        num_rows,
                        active_rows,
                        bitset_pool,
                        pred_bitmaps,
                    );
                }
            }
        },
        LogFieldSelector::ScopeField(sf) => {
            let col_and_nulls = match sf {
                ScopeField::Unspecified => None,
                ScopeField::SchemaUrl => view
                    .root
                    .column_by_name(consts::SCHEMA_URL)
                    .map(|c| (c, None)),
                ScopeField::Name => view
                    .root
                    .column_by_name(consts::SCOPE)
                    .and_then(|c| c.as_any().downcast_ref::<StructArray>())
                    .and_then(|s| s.column_by_name(consts::NAME).map(|c| (c, s.nulls()))),
                ScopeField::Version => view
                    .root
                    .column_by_name(consts::SCOPE)
                    .and_then(|c| c.as_any().downcast_ref::<StructArray>())
                    .and_then(|s| s.column_by_name(consts::VERSION).map(|c| (c, s.nulls()))),
            };
            if let Some((col, parent_nulls)) = col_and_nulls {
                eval_str_column(
                    ev,
                    col,
                    parent_nulls,
                    num_rows,
                    row_words,
                    active_rows,
                    EmptyStrMode::AbsentAll,
                    scratch,
                    bitset_pool,
                    pred_bitmaps,
                );
            }
        }
        LogFieldSelector::BodyField(path) => {
            if path.path.is_empty() {
                return;
            }
            let Some(body_col) = view.root.column_by_name(consts::BODY) else {
                return;
            };
            let Some(body_struct) = body_col.as_any().downcast_ref::<StructArray>() else {
                return;
            };
            let Some(cols) = AnyValColumns::from_struct_array(body_struct) else {
                return;
            };
            let Some(ser) = &cols.arrays.attr_ser else {
                return;
            };
            let base_mask = body_struct.nulls().map(|nulls| {
                DenseBitset::from_validity_buffers(
                    num_rows,
                    None,
                    Some(nulls),
                    active_rows,
                    bitset_pool,
                )
            });
            let eff_mask = base_mask.as_ref().or(active_rows);
            // Pre-filter via SIMD `type == Map` so non-Map body rows skip CBOR decoding.
            let is_map = eval_anyval_type_mask(
                cols.arrays.attr_type,
                AttributeValueType::Map,
                eff_mask,
                bitset_pool,
            );
            let decode_arrays = ev.needs_array_elements();
            for row_u32 in is_map.iter() {
                if let Some(raw) = byte_slice_at(ser, row_u32 as usize) {
                    let res = extract_cbor_path_status(raw, &path.path, decode_arrays);
                    ev.eval_cbor_path_result(res, scratch, |pid| {
                        pred_bitmaps[pid].insert_with_pool(row_u32, row_words, bitset_pool);
                    });
                }
            }
            bitset_pool.recycle(is_map);
            if let Some(bm) = base_mask {
                bitset_pool.recycle(bm);
            }
        }
        LogFieldSelector::LogAttribute(_)
        | LogFieldSelector::ResourceAttribute(_)
        | LogFieldSelector::ScopeAttribute(_) => {}
    }
}

#[allow(clippy::too_many_arguments)]
fn evaluate_body_struct(
    ev: &FieldEvaluator,
    body_struct: &StructArray,
    num_rows: usize,
    row_words: usize,
    active_rows: Option<&DenseBitset>,
    scratch: &mut StringScanScratch,
    bitset_pool: &mut BitsetBufferPool,
    pred_bitmaps: &mut [DenseBitset],
) {
    let Some(cols) = AnyValColumns::from_struct_array(body_struct) else {
        return;
    };
    let base_mask = body_struct.nulls().map(|nulls| {
        DenseBitset::from_validity_buffers(num_rows, None, Some(nulls), active_rows, bitset_pool)
    });
    if base_mask.as_ref().is_some_and(DenseBitset::is_empty) {
        if let Some(bm) = base_mask {
            bitset_pool.recycle(bm);
        }
        return;
    }
    let eff_mask = base_mask.as_ref().or(active_rows);

    // 1. Columnar `Exists` fast path across all OTAP AnyValue discriminants.
    if let Some(pid) = ev.exists_pred {
        let exists_mask = eval_anyval_body_exists_mask(&cols.arrays, eff_mask, bitset_pool);
        pred_bitmaps[pid].or_assign_with_pool(&exists_mask, bitset_pool);
        bitset_pool.recycle(exists_mask);
        if ev.is_exists_only() {
            if let Some(bm) = base_mask {
                bitset_pool.recycle(bm);
            }
            return;
        }
    }

    // 2. String predicates (`Str`).
    if ev.has_str_value_preds() {
        let is_str = eval_anyval_type_mask(
            cols.arrays.attr_type,
            AttributeValueType::Str,
            eff_mask,
            bitset_pool,
        );
        if !is_str.is_empty() {
            let str_dict = cols.dict_str_view();
            let eq_threshold = if str_dict.is_some() {
                SMALL_DICT_STR_EQ_THRESHOLD
            } else {
                SMALL_PLAIN_STR_EQ_THRESHOLD
            };
            if ev.string_index.is_empty() && ev.str_equals_map.len() <= eq_threshold {
                let str_arr = cols.arrays.attr_str.as_ref().map(maybe_dict_as_array);
                for (target, &pid) in &ev.str_equals_map {
                    eval_anyval_str_eq_into(
                        str_arr,
                        target,
                        &is_str,
                        bitset_pool,
                        &mut pred_bitmaps[pid],
                    );
                }
            } else if let Some(dict) = &str_dict {
                let mut str_cache = DictStrEvalCache::new(dict.dict_len());
                for row_u32 in is_str.iter() {
                    let row = row_u32 as usize;
                    if let Some(dict_idx) = dict.valid_dict_idx(row) {
                        let matched =
                            str_cache.eval_or_get(dict_idx, dict.values, false, false, ev, scratch);
                        for &pid in matched {
                            pred_bitmaps[pid].insert_with_pool(row_u32, row_words, bitset_pool);
                        }
                    } else {
                        // Null slot in an AttributeValueType::Str body encodes `""`.
                        ev.eval_str_value_preds("", scratch, |pid| {
                            pred_bitmaps[pid].insert_with_pool(row_u32, row_words, bitset_pool);
                        });
                    }
                }
            } else {
                let str_acc = cols
                    .arrays
                    .attr_str
                    .as_ref()
                    .map(StrColAccessor::from_maybe_dict);
                for row_u32 in is_str.iter() {
                    let s = str_acc
                        .as_ref()
                        .and_then(|c| c.str_at(row_u32 as usize))
                        .unwrap_or("");
                    ev.eval_str_value_preds(s, scratch, |pid| {
                        pred_bitmaps[pid].insert_with_pool(row_u32, row_words, bitset_pool);
                    });
                }
            }
        }
        bitset_pool.recycle(is_str);
    }

    // 3. Integer & Double numeric predicates (`Int`, `Double`).
    if ev.has_numeric_preds() {
        let is_int = eval_anyval_type_mask(
            cols.arrays.attr_type,
            AttributeValueType::Int,
            eff_mask,
            bitset_pool,
        );
        if !is_int.is_empty() {
            let int_arr = cols.arrays.attr_int.as_ref().map(maybe_dict_as_array);
            for (pid, pred) in &ev.scalar_and_range_preds {
                if is_numeric_pred(pred) {
                    eval_anyval_int_pred_into(
                        int_arr,
                        pred,
                        &is_int,
                        bitset_pool,
                        &mut pred_bitmaps[*pid],
                    );
                }
            }
        }
        bitset_pool.recycle(is_int);

        let is_double = eval_anyval_type_mask(
            cols.arrays.attr_type,
            AttributeValueType::Double,
            eff_mask,
            bitset_pool,
        );
        if !is_double.is_empty() {
            for (pid, pred) in &ev.scalar_and_range_preds {
                if is_numeric_pred(pred) {
                    eval_anyval_double_pred_into(
                        cols.arrays.attr_double,
                        pred,
                        &is_double,
                        bitset_pool,
                        &mut pred_bitmaps[*pid],
                    );
                }
            }
        }
        bitset_pool.recycle(is_double);
    }

    // 4. Boolean equality predicates (`Bool`).
    if ev.has_bool_preds() && cols.arrays.attr_bool.is_some() {
        let is_bool = eval_anyval_type_mask(
            cols.arrays.attr_type,
            AttributeValueType::Bool,
            eff_mask,
            bitset_pool,
        );
        if !is_bool.is_empty() {
            for (pid, pred) in &ev.scalar_and_range_preds {
                if let Predicate::Equals(ScalarValue::Bool(target)) = pred {
                    eval_anyval_bool_pred_into(
                        cols.arrays.attr_bool,
                        *target,
                        &is_bool,
                        bitset_pool,
                        &mut pred_bitmaps[*pid],
                    );
                }
            }
        }
        bitset_pool.recycle(is_bool);
    }

    // 5. Bytes equality & subsequence predicates (`Bytes`).
    if ev.has_bytes_eq_preds() || ev.has_bytes_contains_preds() {
        let is_bytes = eval_anyval_type_mask(
            cols.arrays.attr_type,
            AttributeValueType::Bytes,
            eff_mask,
            bitset_pool,
        );
        if !is_bytes.is_empty() {
            let bytes_arr = cols.arrays.attr_bytes.as_ref().map(byte_accessor_as_array);
            for (pid, pred) in &ev.scalar_and_range_preds {
                if let Predicate::Equals(ScalarValue::Bytes(target)) = pred {
                    eval_anyval_bytes_eq_into(
                        bytes_arr,
                        target,
                        &is_bytes,
                        bitset_pool,
                        &mut pred_bitmaps[*pid],
                    );
                }
            }
            if ev.has_bytes_contains_preds() {
                for row_u32 in is_bytes.iter() {
                    let raw = cols
                        .arrays
                        .attr_bytes
                        .as_ref()
                        .and_then(|c| byte_slice_at(c, row_u32 as usize))
                        .unwrap_or(&[]);
                    let val = ExtractedValue::Bytes(Cow::Borrowed(raw));
                    for (pid, target) in &ev.contains_preds {
                        if val.contains_scalar(target) {
                            pred_bitmaps[*pid].insert_with_pool(row_u32, row_words, bitset_pool);
                        }
                    }
                }
            }
        }
        bitset_pool.recycle(is_bytes);
    }

    // 6. CBOR array `Contains` predicates (`Slice`).
    if !ev.contains_preds.is_empty()
        && let Some(ser) = &cols.arrays.attr_ser
    {
        let is_slice = eval_anyval_type_mask(
            cols.arrays.attr_type,
            AttributeValueType::Slice,
            eff_mask,
            bitset_pool,
        );
        for row_u32 in is_slice.iter() {
            if let Some(raw) = byte_slice_at(ser, row_u32 as usize)
                && let Some(val) = extract_cbor_path_for_eval(raw, &[], true)
            {
                for (pid, target) in &ev.contains_preds {
                    if val.contains_scalar(target) {
                        pred_bitmaps[*pid].insert_with_pool(row_u32, row_words, bitset_pool);
                    }
                }
            }
        }
        bitset_pool.recycle(is_slice);
    }

    if let Some(bm) = base_mask {
        bitset_pool.recycle(bm);
    }
}

/// Shared execution context for evaluating one OTAP attribute table (`LogAttrs`, `ResourceAttrs`,
/// or `ScopeAttrs`) without threading long parameter lists through helper calls.
struct AttrEvalCtx<'a, 'b, R> {
    plan: &'a StagePlan,
    key_col: &'a ArrayRef,
    parent_ids: ParentIdColumn<'a>,
    cols: AnyValColumns<'a>,
    str_dict: Option<DictStrView<'a>>,
    str_caches: Vec<Option<DictStrEvalCache>>,
    num_attr_rows: usize,
    min_words: usize,
    string_scratches: &'b mut [StringScanScratch],
    bitset_pool: &'b mut BitsetBufferPool,
    resolve_target: R,
    pred_bitmaps: &'b mut [DenseBitset],
}

impl<'a, 'b, R> AttrEvalCtx<'a, 'b, R>
where
    R: FnMut(u16) -> Option<u32>,
{
    fn map_attr_mask_to_target(&mut self, attr_mask: &DenseBitset, pid: usize) {
        for r_u32 in attr_mask.iter() {
            let Some(parent_id) = self.parent_ids.value_at(r_u32 as usize) else {
                continue;
            };
            let Some(target_id) = (self.resolve_target)(parent_id) else {
                continue;
            };
            self.pred_bitmaps[pid].insert_with_pool(target_id, self.min_words, self.bitset_pool);
        }
    }

    /// Maps surviving attribute rows into `pred_bitmaps[pid]` and recycles `matched_attr_rows`.
    fn map_and_recycle_attr_mask(&mut self, matched_attr_rows: DenseBitset, pid: usize) {
        self.map_attr_mask_to_target(&matched_attr_rows, pid);
        self.bitset_pool.recycle(matched_attr_rows);
    }

    fn evaluate_attr_row(
        &mut self,
        attr_row: usize,
        target_id: u32,
        ev_indices: &[usize],
        include_top_exists: bool,
    ) {
        if let Some(dict) = self.str_dict
            && self.cols.is_str_row(attr_row)
        {
            for &ev_idx in ev_indices {
                let ev = &self.plan.field_evaluators[ev_idx];
                let Some(path_segments) = attr_path_segments(&ev.field) else {
                    continue;
                };
                if path_segments.len() != 1 {
                    continue;
                }
                if !ev.has_str_value_preds() {
                    if include_top_exists && let Some(pid) = ev.exists_pred {
                        self.pred_bitmaps[pid].insert_with_pool(
                            target_id,
                            self.min_words,
                            self.bitset_pool,
                        );
                    }
                    continue;
                }
                let scratch = &mut self.string_scratches[ev_idx];
                ev.string_index.prepare_scratch(scratch);
                if let Some(dict_idx) = dict.valid_dict_idx(attr_row) {
                    if self.str_caches.is_empty() {
                        self.str_caches
                            .resize_with(self.plan.field_evaluators.len(), || None);
                    }
                    let cache = self.str_caches[ev_idx]
                        .get_or_insert_with(|| DictStrEvalCache::new(dict.dict_len()));
                    let matched = cache.eval_or_get(
                        dict_idx,
                        dict.values,
                        false,
                        include_top_exists,
                        ev,
                        scratch,
                    );
                    for &pid in matched {
                        self.pred_bitmaps[pid].insert_with_pool(
                            target_id,
                            self.min_words,
                            self.bitset_pool,
                        );
                    }
                } else if include_top_exists {
                    // Null slot in an AttributeValueType::Str attribute encodes `""`.
                    ev.eval_str_only("", scratch, |pid| {
                        self.pred_bitmaps[pid].insert_with_pool(
                            target_id,
                            self.min_words,
                            self.bitset_pool,
                        );
                    });
                } else {
                    ev.eval_str_value_preds("", scratch, |pid| {
                        self.pred_bitmaps[pid].insert_with_pool(
                            target_id,
                            self.min_words,
                            self.bitset_pool,
                        );
                    });
                }
            }
            return;
        }

        if let Some((vtype, raw_opt)) = self.cols.cbor_row_kind(attr_row) {
            for &ev_idx in ev_indices {
                let ev = &self.plan.field_evaluators[ev_idx];
                let Some(path_segments) = attr_path_segments(&ev.field) else {
                    continue;
                };

                if path_segments.len() == 1 {
                    if include_top_exists && let Some(pid) = ev.exists_pred {
                        // Top-level attribute key is present regardless of CBOR contents.
                        self.pred_bitmaps[pid].insert_with_pool(
                            target_id,
                            self.min_words,
                            self.bitset_pool,
                        );
                    }
                    if vtype == AttributeValueType::Slice
                        && !ev.contains_preds.is_empty()
                        && let Some(raw) = raw_opt
                        && let Some(val) = extract_cbor_path_for_eval(raw, &[], true)
                    {
                        for (pid, target) in &ev.contains_preds {
                            if val.contains_scalar(target) {
                                self.pred_bitmaps[*pid].insert_with_pool(
                                    target_id,
                                    self.min_words,
                                    self.bitset_pool,
                                );
                            }
                        }
                    }
                } else if vtype == AttributeValueType::Map
                    && let Some(raw) = raw_opt
                {
                    let res = extract_cbor_path_status(
                        raw,
                        &path_segments[1..],
                        ev.needs_array_elements(),
                    );
                    let scratch = &mut self.string_scratches[ev_idx];
                    ev.string_index.prepare_scratch(scratch);
                    ev.eval_cbor_path_result(res, scratch, |pid| {
                        self.pred_bitmaps[pid].insert_with_pool(
                            target_id,
                            self.min_words,
                            self.bitset_pool,
                        );
                    });
                }
            }
            return;
        }

        let val_opt = self.cols.extract_at_row(attr_row);

        for &ev_idx in ev_indices {
            let ev = &self.plan.field_evaluators[ev_idx];
            let Some(path_segments) = attr_path_segments(&ev.field) else {
                continue;
            };

            if path_segments.len() == 1 {
                match &val_opt {
                    Some(val) => {
                        let scratch = &mut self.string_scratches[ev_idx];
                        ev.string_index.prepare_scratch(scratch);
                        if include_top_exists {
                            ev.eval_extracted_value(val, scratch, |pid| {
                                self.pred_bitmaps[pid].insert_with_pool(
                                    target_id,
                                    self.min_words,
                                    self.bitset_pool,
                                );
                            });
                        } else {
                            ev.eval_extracted_value_without_exists(val, scratch, |pid| {
                                self.pred_bitmaps[pid].insert_with_pool(
                                    target_id,
                                    self.min_words,
                                    self.bitset_pool,
                                );
                            });
                        }
                    }
                    None => {
                        if include_top_exists && let Some(pid) = ev.exists_pred {
                            // Top-level key is present even when its AnyValue is Empty.
                            self.pred_bitmaps[pid].insert_with_pool(
                                target_id,
                                self.min_words,
                                self.bitset_pool,
                            );
                        }
                    }
                }
            }
        }
    }

    /// Evaluates all predicates sharing a single watched attribute `key` using Arrow SIMD
    /// `cmp::eq(key)` plus DataFusion's 20% selectivity threshold.
    fn evaluate_attr_key_columnar(&mut self, key: &str, ev_indices: &[usize]) {
        let Ok(key_cmp) = cmp::eq(self.key_col, &StringArray::new_scalar(key)) else {
            return;
        };
        let key_mask = DenseBitset::from_boolean_array(&key_cmp, false, None, self.bitset_pool);
        if key_mask.is_empty() {
            self.bitset_pool.recycle(key_mask);
            return;
        }

        // 1. Identify the unique single-segment evaluator (if any) and resolve its top-level
        //    `Exists` predicate directly from `key_mask` without reading `attr_type` or value
        //    columns (`StagePlanBuilder` deduplicates `LogFieldSelector`).
        let mut top_ev_idx: Option<usize> = None;
        let mut has_nested = false;
        let mut has_non_exists_work = false;
        for &ev_idx in ev_indices {
            let ev = &self.plan.field_evaluators[ev_idx];
            let Some(path_segments) = attr_path_segments(&ev.field) else {
                continue;
            };
            if path_segments.len() == 1 {
                debug_assert!(top_ev_idx.is_none() || top_ev_idx == Some(ev_idx));
                top_ev_idx = Some(ev_idx);
                if !ev.is_exists_only() {
                    has_non_exists_work = true;
                }
            } else {
                has_nested = true;
                has_non_exists_work = true;
            }
        }
        if let Some(pid) = top_ev_idx.and_then(|idx| self.plan.field_evaluators[idx].exists_pred) {
            self.map_attr_mask_to_target(&key_mask, pid);
        }
        if !has_non_exists_work {
            self.bitset_pool.recycle(key_mask);
            return;
        }

        // 2. Selectivity gate (DataFusion's 20% threshold):
        //    - Low selectivity (> 20% of rows match `key`): SIMD value kernels + bitwise AND.
        //    - High selectivity (<= 20%): sparse `key_mask` iteration with early target gating.
        let key_matches = key_mask.len() as usize;
        let low_selectivity = self.num_attr_rows >= MIN_COLUMNAR_ATTR_ROWS
            && key_matches * ATTR_SELECTIVITY_DENOMINATOR > self.num_attr_rows;

        if !low_selectivity {
            for attr_row_u32 in key_mask.iter() {
                let attr_row = attr_row_u32 as usize;
                let Some(parent_id) = self.parent_ids.value_at(attr_row) else {
                    continue;
                };
                let Some(target_id) = (self.resolve_target)(parent_id) else {
                    continue;
                };
                self.evaluate_attr_row(attr_row, target_id, ev_indices, false);
            }
            self.bitset_pool.recycle(key_mask);
            return;
        }

        // Low-selectivity columnar path (> 20% of rows have this key).
        if let Some(ev_idx) = top_ev_idx {
            let ev = &self.plan.field_evaluators[ev_idx];

            if ev.has_numeric_preds() {
                let is_int = eval_anyval_type_mask(
                    self.cols.arrays.attr_type,
                    AttributeValueType::Int,
                    Some(&key_mask),
                    self.bitset_pool,
                );
                if !is_int.is_empty() {
                    let int_arr = self.cols.arrays.attr_int.as_ref().map(maybe_dict_as_array);
                    for (pid, pred) in &ev.scalar_and_range_preds {
                        if is_numeric_pred(pred) {
                            let mut matched = DenseBitset::new();
                            eval_anyval_int_pred_into(
                                int_arr,
                                pred,
                                &is_int,
                                self.bitset_pool,
                                &mut matched,
                            );
                            self.map_and_recycle_attr_mask(matched, *pid);
                        }
                    }
                }
                self.bitset_pool.recycle(is_int);

                let is_double = eval_anyval_type_mask(
                    self.cols.arrays.attr_type,
                    AttributeValueType::Double,
                    Some(&key_mask),
                    self.bitset_pool,
                );
                if !is_double.is_empty() {
                    for (pid, pred) in &ev.scalar_and_range_preds {
                        if is_numeric_pred(pred) {
                            let mut matched = DenseBitset::new();
                            eval_anyval_double_pred_into(
                                self.cols.arrays.attr_double,
                                pred,
                                &is_double,
                                self.bitset_pool,
                                &mut matched,
                            );
                            self.map_and_recycle_attr_mask(matched, *pid);
                        }
                    }
                }
                self.bitset_pool.recycle(is_double);
            }

            if ev.has_bool_preds() && self.cols.arrays.attr_bool.is_some() {
                let is_bool = eval_anyval_type_mask(
                    self.cols.arrays.attr_type,
                    AttributeValueType::Bool,
                    Some(&key_mask),
                    self.bitset_pool,
                );
                if !is_bool.is_empty() {
                    for (pid, pred) in &ev.scalar_and_range_preds {
                        if let Predicate::Equals(ScalarValue::Bool(target)) = pred {
                            let mut matched = DenseBitset::new();
                            eval_anyval_bool_pred_into(
                                self.cols.arrays.attr_bool,
                                *target,
                                &is_bool,
                                self.bitset_pool,
                                &mut matched,
                            );
                            self.map_and_recycle_attr_mask(matched, *pid);
                        }
                    }
                }
                self.bitset_pool.recycle(is_bool);
            }

            if ev.has_bytes_eq_preds() || ev.has_bytes_contains_preds() {
                let is_bytes = eval_anyval_type_mask(
                    self.cols.arrays.attr_type,
                    AttributeValueType::Bytes,
                    Some(&key_mask),
                    self.bitset_pool,
                );
                if !is_bytes.is_empty() {
                    let bytes_arr = self
                        .cols
                        .arrays
                        .attr_bytes
                        .as_ref()
                        .map(byte_accessor_as_array);
                    for (pid, pred) in &ev.scalar_and_range_preds {
                        if let Predicate::Equals(ScalarValue::Bytes(target)) = pred {
                            let mut matched = DenseBitset::new();
                            eval_anyval_bytes_eq_into(
                                bytes_arr,
                                target,
                                &is_bytes,
                                self.bitset_pool,
                                &mut matched,
                            );
                            self.map_and_recycle_attr_mask(matched, *pid);
                        }
                    }
                    if ev.has_bytes_contains_preds() {
                        for r_u32 in is_bytes.iter() {
                            let r = r_u32 as usize;
                            let Some(parent_id) = self.parent_ids.value_at(r) else {
                                continue;
                            };
                            let Some(target_id) = (self.resolve_target)(parent_id) else {
                                continue;
                            };
                            let raw = self
                                .cols
                                .arrays
                                .attr_bytes
                                .as_ref()
                                .and_then(|c| byte_slice_at(c, r))
                                .unwrap_or(&[]);
                            let val = ExtractedValue::Bytes(Cow::Borrowed(raw));
                            for (pid, target) in &ev.contains_preds {
                                if val.contains_scalar(target) {
                                    self.pred_bitmaps[*pid].insert_with_pool(
                                        target_id,
                                        self.min_words,
                                        self.bitset_pool,
                                    );
                                }
                            }
                        }
                    }
                }
                self.bitset_pool.recycle(is_bytes);
            }

            if ev.has_str_value_preds() {
                let is_str = eval_anyval_type_mask(
                    self.cols.arrays.attr_type,
                    AttributeValueType::Str,
                    Some(&key_mask),
                    self.bitset_pool,
                );
                if !is_str.is_empty() {
                    let eq_threshold = if self.str_dict.is_some() {
                        SMALL_DICT_STR_EQ_THRESHOLD
                    } else {
                        SMALL_PLAIN_STR_EQ_THRESHOLD
                    };
                    if ev.string_index.is_empty() && ev.str_equals_map.len() <= eq_threshold {
                        let str_arr = self.cols.arrays.attr_str.as_ref().map(maybe_dict_as_array);
                        for (target, &pid) in &ev.str_equals_map {
                            let mut matched = DenseBitset::new();
                            eval_anyval_str_eq_into(
                                str_arr,
                                target,
                                &is_str,
                                self.bitset_pool,
                                &mut matched,
                            );
                            self.map_and_recycle_attr_mask(matched, pid);
                        }
                    } else if let Some(dict) = self.str_dict {
                        if self.str_caches.is_empty() {
                            self.str_caches
                                .resize_with(self.plan.field_evaluators.len(), || None);
                        }
                        let scratch = &mut self.string_scratches[ev_idx];
                        ev.string_index.prepare_scratch(scratch);
                        let cache = self.str_caches[ev_idx]
                            .get_or_insert_with(|| DictStrEvalCache::new(dict.dict_len()));
                        for r_u32 in is_str.iter() {
                            let r = r_u32 as usize;
                            let Some(parent_id) = self.parent_ids.value_at(r) else {
                                continue;
                            };
                            let Some(target_id) = (self.resolve_target)(parent_id) else {
                                continue;
                            };
                            if let Some(dict_idx) = dict.valid_dict_idx(r) {
                                let matched = cache.eval_or_get(
                                    dict_idx,
                                    dict.values,
                                    false,
                                    false,
                                    ev,
                                    scratch,
                                );
                                for &pid in matched {
                                    self.pred_bitmaps[pid].insert_with_pool(
                                        target_id,
                                        self.min_words,
                                        self.bitset_pool,
                                    );
                                }
                            } else {
                                ev.eval_str_value_preds("", scratch, |pid| {
                                    self.pred_bitmaps[pid].insert_with_pool(
                                        target_id,
                                        self.min_words,
                                        self.bitset_pool,
                                    );
                                });
                            }
                        }
                    } else {
                        let scratch = &mut self.string_scratches[ev_idx];
                        ev.string_index.prepare_scratch(scratch);
                        let str_acc = self
                            .cols
                            .arrays
                            .attr_str
                            .as_ref()
                            .map(StrColAccessor::from_maybe_dict);
                        for r_u32 in is_str.iter() {
                            let r = r_u32 as usize;
                            let Some(parent_id) = self.parent_ids.value_at(r) else {
                                continue;
                            };
                            let Some(target_id) = (self.resolve_target)(parent_id) else {
                                continue;
                            };
                            let s = str_acc.as_ref().and_then(|c| c.str_at(r)).unwrap_or("");
                            ev.eval_str_value_preds(s, scratch, |pid| {
                                self.pred_bitmaps[pid].insert_with_pool(
                                    target_id,
                                    self.min_words,
                                    self.bitset_pool,
                                );
                            });
                        }
                    }
                }
                self.bitset_pool.recycle(is_str);
            }

            if !ev.contains_preds.is_empty()
                && let Some(ser) = &self.cols.arrays.attr_ser
            {
                let is_slice = eval_anyval_type_mask(
                    self.cols.arrays.attr_type,
                    AttributeValueType::Slice,
                    Some(&key_mask),
                    self.bitset_pool,
                );
                for r_u32 in is_slice.iter() {
                    let r = r_u32 as usize;
                    let Some(parent_id) = self.parent_ids.value_at(r) else {
                        continue;
                    };
                    let Some(target_id) = (self.resolve_target)(parent_id) else {
                        continue;
                    };
                    let Some(raw) = byte_slice_at(ser, r) else {
                        continue;
                    };
                    let Some(val) = extract_cbor_path_for_eval(raw, &[], true) else {
                        continue;
                    };
                    for (pid, target) in &ev.contains_preds {
                        if val.contains_scalar(target) {
                            self.pred_bitmaps[*pid].insert_with_pool(
                                target_id,
                                self.min_words,
                                self.bitset_pool,
                            );
                        }
                    }
                }
                self.bitset_pool.recycle(is_slice);
            }
        }

        if has_nested && let Some(ser) = &self.cols.arrays.attr_ser {
            // SIMD pre-filter `key == path[0] && type == Map` before CBOR path decoding.
            let is_map = eval_anyval_type_mask(
                self.cols.arrays.attr_type,
                AttributeValueType::Map,
                Some(&key_mask),
                self.bitset_pool,
            );
            for r_u32 in is_map.iter() {
                let r = r_u32 as usize;
                let Some(parent_id) = self.parent_ids.value_at(r) else {
                    continue;
                };
                let Some(target_id) = (self.resolve_target)(parent_id) else {
                    continue;
                };
                let Some(raw) = byte_slice_at(ser, r) else {
                    continue;
                };
                for &ev_idx in ev_indices {
                    let ev = &self.plan.field_evaluators[ev_idx];
                    let Some(path_segments) = attr_path_segments(&ev.field) else {
                        continue;
                    };
                    if path_segments.len() <= 1 {
                        continue;
                    }
                    let res = extract_cbor_path_status(
                        raw,
                        &path_segments[1..],
                        ev.needs_array_elements(),
                    );
                    let scratch = &mut self.string_scratches[ev_idx];
                    ev.string_index.prepare_scratch(scratch);
                    ev.eval_cbor_path_result(res, scratch, |pid| {
                        self.pred_bitmaps[pid].insert_with_pool(
                            target_id,
                            self.min_words,
                            self.bitset_pool,
                        );
                    });
                }
            }
            self.bitset_pool.recycle(is_map);
        }

        self.bitset_pool.recycle(key_mask);
    }
}

#[allow(clippy::too_many_arguments)]
pub(crate) fn evaluate_attr_table<R>(
    plan: &StagePlan,
    rb: &RecordBatch,
    key_map: &AHashMap<String, Vec<usize>>,
    min_words: usize,
    string_scratches: &mut [StringScanScratch],
    bitset_pool: &mut BitsetBufferPool,
    resolve_target: R,
    pred_bitmaps: &mut [DenseBitset],
) where
    R: FnMut(u16) -> Option<u32>,
{
    let num_attr_rows = rb.num_rows();
    if num_attr_rows == 0 {
        return;
    }
    let Some(key_col) = rb.column_by_name(consts::ATTRIBUTE_KEY) else {
        return;
    };
    let Some(parent_ids) = ParentIdColumn::from_batch(rb) else {
        return;
    };
    let Some(cols) = AnyValColumns::from_attr_batch(rb) else {
        return;
    };
    let key_dict = DictStrView::try_from_array(key_col);
    let str_dict = cols.dict_str_view();
    let str_caches: Vec<Option<DictStrEvalCache>> = Vec::new();

    let mut ctx = AttrEvalCtx {
        plan,
        key_col,
        parent_ids,
        cols,
        str_dict,
        str_caches,
        num_attr_rows,
        min_words,
        string_scratches,
        bitset_pool,
        resolve_target,
        pred_bitmaps,
    };

    // Pre-filter dictionary-encoded key columns (<= 1024 entries):
    // - 0 watched keys in dictionary -> skip the entire table in O(dict_len).
    // - 1 watched key in dictionary -> selectivity-gated Arrow SIMD path.
    // - 2+ watched keys -> single-pass scan using the pre-built direct dict lookup table.
    if let Some(kd) = &key_dict {
        if kd.dict_len() <= DIRECT_DICT_CACHE_LIMIT {
            let kvals = kd.values;
            let mut present_keys = 0usize;
            let mut first_hit: Option<(usize, &str, &[usize])> = None;
            let mut dict_ev_map: Vec<Option<&[usize]>> = Vec::new();
            for di in 0..kvals.len() {
                if kvals.is_valid(di) {
                    let k = kvals.value(di);
                    if let Some(indices) = key_map.get(k) {
                        let slice = indices.as_slice();
                        present_keys += 1;
                        if present_keys == 1 {
                            first_hit = Some((di, k, slice));
                        } else {
                            if dict_ev_map.is_empty() {
                                dict_ev_map.resize(kvals.len(), None);
                                if let Some((first_di, _, first_slice)) = first_hit {
                                    dict_ev_map[first_di] = Some(first_slice);
                                }
                            }
                            dict_ev_map[di] = Some(slice);
                        }
                    }
                }
            }
            if present_keys == 0 {
                return;
            }
            if present_keys == 1
                && let Some((_, key, ev_indices)) = first_hit
            {
                ctx.evaluate_attr_key_columnar(key, ev_indices);
                return;
            }

            for attr_row in 0..num_attr_rows {
                let Some(key_dict_idx) = kd.valid_dict_idx(attr_row) else {
                    continue;
                };
                let Some(ev_indices) = dict_ev_map[key_dict_idx] else {
                    continue;
                };
                let Some(parent_id) = ctx.parent_ids.value_at(attr_row) else {
                    continue;
                };
                let Some(target_id) = (ctx.resolve_target)(parent_id) else {
                    continue;
                };
                ctx.evaluate_attr_row(attr_row, target_id, ev_indices, true);
            }
            return;
        }

        if key_map.len() == 1
            && let Some((key, indices)) = key_map.iter().next()
        {
            ctx.evaluate_attr_key_columnar(key, indices.as_slice());
            return;
        }

        let mut key_cache = DictKeyLookupCache::new();
        for attr_row in 0..num_attr_rows {
            let Some(key_dict_idx) = kd.valid_dict_idx(attr_row) else {
                continue;
            };
            let Some(ev_indices) = key_cache.get_or_lookup(key_dict_idx, kd.values, key_map) else {
                continue;
            };
            let Some(parent_id) = ctx.parent_ids.value_at(attr_row) else {
                continue;
            };
            let Some(target_id) = (ctx.resolve_target)(parent_id) else {
                continue;
            };
            ctx.evaluate_attr_row(attr_row, target_id, ev_indices, true);
        }
        return;
    }

    let Some(plain_keys) = key_col.as_any().downcast_ref::<StringArray>() else {
        return;
    };

    // Plain `StringArray` key column: use selectivity-gated SIMD `cmp::eq` for small watched
    // key sets (`<= 2`), otherwise single-pass `AHashMap` lookup.
    if key_map.len() <= SMALL_PLAIN_STR_EQ_THRESHOLD {
        for (key, indices) in key_map {
            ctx.evaluate_attr_key_columnar(key, indices.as_slice());
        }
        return;
    }

    for attr_row in 0..num_attr_rows {
        if !plain_keys.is_valid(attr_row) {
            continue;
        }
        let Some(indices) = key_map.get(plain_keys.value(attr_row)) else {
            continue;
        };
        let Some(parent_id) = ctx.parent_ids.value_at(attr_row) else {
            continue;
        };
        let Some(target_id) = (ctx.resolve_target)(parent_id) else {
            continue;
        };
        ctx.evaluate_attr_row(attr_row, target_id, indices.as_slice(), true);
    }
}
