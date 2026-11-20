// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! HIR N-Gram Inverted Index (`AhoCorasick`) + `regex_automata::meta::Regex`
//! columnar policy evaluation engine (`open-telemetry/otel-arrow#4726`).
//!
//! # Architecture
//! - Parses each `Predicate::Regex(pat)` and `Predicate::Contains(ScalarValue::String(s))` into a
//!   `regex_syntax::hir::Hir` and walks `HirKind::Concat`, `HirKind::Literal`, `HirKind::Capture`,
//!   `HirKind::Repetition` (`min >= 1`), and `HirKind::Alternation` to extract all mandatory
//!   contiguous byte literals of length `>= 2` (2-grams, 3-grams, and longer required literal runs).
//! - Indexes all extracted n-grams across all string patterns on a field into a single
//!   `aho_corasick::AhoCorasick` (`MatchKind::Standard`) automaton mapping each unique `ngram_id`
//!   to the posting list of pattern IDs requiring that n-gram.
//! - Scans each string value in a single `ac.find_overlapping_iter(haystack)` pass using
//!   generation-stamped counters (zero per-row clearing allocations) to count distinct matched
//!   required n-grams per candidate pattern, invoking the individual `regex_automata::meta::Regex`
//!   verifier only when `matched_ngrams == required_ngram_count`.

use std::borrow::Cow;
use std::sync::Arc;

use ahash::AHashMap;
use aho_corasick::{AhoCorasick, MatchKind};
use arrow::array::{
    Array, ArrayRef, BooleanArray, DictionaryArray, Float64Array, RecordBatch, StringArray,
    StructArray, UInt8Array, UInt16Array,
};
use arrow::buffer::NullBuffer;
use arrow::datatypes::{Schema, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::IdBitmapPool;
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use regex_automata::meta::Regex as MetaRegex;
use regex_syntax::hir::{Hir, HirKind};
use roaring::RoaringBitmap;

use crate::EvaluationOutput;
use crate::model::{
    Action, LogAttributeSelector, LogFieldSelector, LogMatcher, LogRecordField, LogTargetSelector,
    LogTransformPolicy, NumericValue, Policy, PolicyCompileError, PolicyDefinition, PolicyError,
    Predicate, ScalarValue, ScopeField, compile_matcher_regex, compile_redact_regex,
};
use crate::otap_ops::{
    CompiledLogRedact, CompiledTransformPolicy, ExtractedValue, LogsBatchView, ParentIdColumn,
    apply_spec_transforms_masked, decode_cbor_value, extract_bytes_from_array,
    extract_cbor_path_for_eval, extract_fixed_bytes_from_array, extract_i64_from_array,
    extract_str_from_array, filter_otap_logs_by_bitmap,
};

/// Extracts all mandatory contiguous byte literals (n-grams of length `>= 2` bytes)
/// that must appear verbatim in every match of `hir`.
///
/// Walks `HirKind::Concat` (coalescing adjacent mandatory literals), `HirKind::Literal`,
/// `HirKind::Capture`, `HirKind::Repetition` (`min >= 1`), and `HirKind::Alternation`
/// (retaining n-grams common to all branches).
#[must_use]
pub(crate) fn extract_required_ngrams(hir: &Hir) -> Vec<Vec<u8>> {
    let mut raw = Vec::new();
    if let Some(lit) = as_pure_literal(hir) {
        if lit.len() >= 2 {
            raw.push(lit);
        }
    } else {
        extract_ngrams_recursive(hir, &mut raw);
    }

    let mut unique: Vec<Vec<u8>> = Vec::with_capacity(raw.len());
    for ngram in raw {
        if ngram.len() >= 2 && !unique.iter().any(|u| u.as_slice() == ngram.as_slice()) {
            unique.push(ngram);
        }
    }
    unique
}

/// Returns `Some(bytes)` if `hir` matches only a single fixed byte sequence.
fn as_pure_literal(hir: &Hir) -> Option<Vec<u8>> {
    match hir.kind() {
        HirKind::Literal(lit) => Some(lit.0.to_vec()),
        HirKind::Capture(cap) => as_pure_literal(&cap.sub),
        HirKind::Concat(subs) => {
            let mut buf = Vec::new();
            for sub in subs {
                let part = as_pure_literal(sub)?;
                buf.extend_from_slice(&part);
            }
            Some(buf)
        }
        _ => None,
    }
}

fn extract_ngrams_recursive(hir: &Hir, out: &mut Vec<Vec<u8>>) {
    match hir.kind() {
        HirKind::Empty | HirKind::Class(_) | HirKind::Look(_) => {}
        HirKind::Literal(lit) => {
            if lit.0.len() >= 2 {
                out.push(lit.0.to_vec());
            }
        }
        HirKind::Capture(cap) => {
            if let Some(lit) = as_pure_literal(&cap.sub) {
                if lit.len() >= 2 {
                    out.push(lit);
                }
            } else {
                extract_ngrams_recursive(&cap.sub, out);
            }
        }
        HirKind::Repetition(rep) => {
            if rep.min >= 1 {
                if let Some(lit) = as_pure_literal(&rep.sub) {
                    let repeat_count = (rep.min as usize).min(16);
                    let repeated = lit.repeat(repeat_count);
                    if repeated.len() >= 2 {
                        out.push(repeated);
                    }
                } else {
                    extract_ngrams_recursive(&rep.sub, out);
                }
            }
        }
        HirKind::Concat(subs) => {
            let mut current_lit = Vec::new();
            for sub in subs {
                if let Some(lit) = as_pure_literal(sub) {
                    current_lit.extend_from_slice(&lit);
                } else {
                    if current_lit.len() >= 2 {
                        out.push(std::mem::take(&mut current_lit));
                    } else {
                        current_lit.clear();
                    }
                    extract_ngrams_recursive(sub, out);
                }
            }
            if current_lit.len() >= 2 {
                out.push(current_lit);
            }
        }
        HirKind::Alternation(branches) => {
            if let Some((first, rest)) = branches.split_first() {
                let mut common = extract_required_ngrams(first);
                for branch in rest {
                    if common.is_empty() {
                        break;
                    }
                    let branch_ngrams = extract_required_ngrams(branch);
                    common.retain(|c| branch_ngrams.iter().any(|b| b.as_slice() == c.as_slice()));
                }
                out.extend(common);
            }
        }
    }
}

/// Verification stage for a string pattern after required n-gram prefiltering.
#[derive(Debug, Clone)]
enum StringVerifier {
    /// Substring match (`Predicate::Contains(ScalarValue::String)`).
    Contains(String),
    /// Compiled `regex_automata::meta::Regex` (`multi_line(false)`).
    Regex(MetaRegex),
}

impl StringVerifier {
    #[inline]
    fn verify_after_all_ngrams_matched(&self, haystack: &str) -> bool {
        match self {
            // For `Contains(s)` indexed as a single full-substring n-gram in `AhoCorasick`,
            // matching that n-gram already proves `haystack.contains(s)`.
            Self::Contains(_) => true,
            Self::Regex(re) => re.is_match(haystack),
        }
    }

    #[inline]
    fn verify_fallback(&self, haystack: &str) -> bool {
        match self {
            Self::Contains(sub) => haystack.contains(sub.as_str()),
            Self::Regex(re) => re.is_match(haystack),
        }
    }
}

/// A regex or `Contains` string pattern with `>= 1` required n-grams indexed in `AhoCorasick`.
#[derive(Debug, Clone)]
struct IndexedPattern {
    pred_id: usize,
    required_ngrams: u16,
    verifier: StringVerifier,
}

/// A regex or `Contains` string pattern with 0 extractable n-grams of length `>= 2`.
#[derive(Debug, Clone)]
struct FallbackPattern {
    pred_id: usize,
    verifier: StringVerifier,
}

/// Per-field n-gram inverted index over `AhoCorasick` (`MatchKind::Standard`) + `MetaRegex` verifiers.
#[derive(Debug, Clone, Default)]
struct FieldStringIndex {
    ac: Option<AhoCorasick>,
    /// Maps `ngram_id` (`0..num_unique_ngrams`) -> list of `indexed_patterns` indices (`u32`).
    ngram_postings: Vec<Vec<u32>>,
    indexed_patterns: Vec<IndexedPattern>,
    fallback_patterns: Vec<FallbackPattern>,
}

/// Reusable generation-stamped scratch state for scanning strings against a `FieldStringIndex`
/// without per-row vector clearing or allocations.
#[derive(Default)]
struct StringScanScratch {
    cur_gen: u32,
    ngram_seen_gen: Vec<u32>,
    pat_seen_gen: Vec<u32>,
    pat_matched_count: Vec<u16>,
}

impl StringScanScratch {
    /// Ensures counter buffers match `(num_ngrams, num_patterns)`, reusing existing capacity on
    /// steady-state batches and resetting generations when resized across policy reloads.
    fn prepare(&mut self, num_ngrams: usize, num_patterns: usize) {
        if self.ngram_seen_gen.len() != num_ngrams {
            self.ngram_seen_gen.clear();
            self.ngram_seen_gen.resize(num_ngrams, 0);
            self.cur_gen = 0;
        }
        if self.pat_seen_gen.len() != num_patterns {
            self.pat_seen_gen.clear();
            self.pat_seen_gen.resize(num_patterns, 0);
            self.pat_matched_count.clear();
            self.pat_matched_count.resize(num_patterns, 0);
            self.cur_gen = 0;
        }
    }

    #[inline]
    fn next_generation(&mut self) -> u32 {
        self.cur_gen = self.cur_gen.wrapping_add(1);
        if self.cur_gen == 0 {
            self.ngram_seen_gen.fill(0);
            self.pat_seen_gen.fill(0);
            self.cur_gen = 1;
        }
        self.cur_gen
    }
}

/// Reusable per-core evaluation scratch state pooling [`IdBitmapPool`] and n-gram scan counters
/// across batches to eliminate hot-path allocations.
#[derive(Default)]
pub struct EvaluationScratch {
    id_bitmap_pool: IdBitmapPool,
    string_scratches: Vec<StringScanScratch>,
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
}

impl FieldStringIndex {
    fn is_empty(&self) -> bool {
        self.ac.is_none() && self.fallback_patterns.is_empty()
    }

    #[inline]
    fn prepare_scratch(&self, scratch: &mut StringScanScratch) {
        scratch.prepare(self.ngram_postings.len(), self.indexed_patterns.len());
    }

    #[inline]
    fn evaluate_str<F>(&self, haystack: &str, scratch: &mut StringScanScratch, mut on_match: F)
    where
        F: FnMut(usize),
    {
        if let Some(ac) = &self.ac {
            let generation = scratch.next_generation();
            for mat in ac.find_overlapping_iter(haystack) {
                let ngram_id = mat.pattern().as_usize();
                if scratch.ngram_seen_gen[ngram_id] == generation {
                    continue;
                }
                scratch.ngram_seen_gen[ngram_id] = generation;

                for &pat_idx_u32 in &self.ngram_postings[ngram_id] {
                    let pat_idx = pat_idx_u32 as usize;
                    let count = if scratch.pat_seen_gen[pat_idx] == generation {
                        scratch.pat_matched_count[pat_idx] + 1
                    } else {
                        scratch.pat_seen_gen[pat_idx] = generation;
                        1
                    };
                    scratch.pat_matched_count[pat_idx] = count;

                    let pat = &self.indexed_patterns[pat_idx];
                    if count == pat.required_ngrams
                        && pat.verifier.verify_after_all_ngrams_matched(haystack)
                    {
                        on_match(pat.pred_id);
                    }
                }
            }
        }

        for fb in &self.fallback_patterns {
            if fb.verifier.verify_fallback(haystack) {
                on_match(fb.pred_id);
            }
        }
    }
}

/// Builder for compiling string patterns on a single field into a `FieldStringIndex`.
#[derive(Default)]
struct FieldStringIndexBuilder {
    unique_ngrams: Vec<Vec<u8>>,
    ngram_to_id: AHashMap<Vec<u8>, usize>,
    ngram_postings: Vec<Vec<u32>>,
    indexed_patterns: Vec<IndexedPattern>,
    fallback_patterns: Vec<FallbackPattern>,
}

impl FieldStringIndexBuilder {
    fn intern_ngram(&mut self, ngram: Vec<u8>) -> usize {
        if let Some(&id) = self.ngram_to_id.get(&ngram) {
            return id;
        }
        let id = self.unique_ngrams.len();
        self.unique_ngrams.push(ngram.clone());
        let _ = self.ngram_to_id.insert(ngram, id);
        self.ngram_postings.push(Vec::new());
        id
    }

    fn add_contains(&mut self, pred_id: usize, sub: &str) {
        if sub.len() >= 2 {
            let pat_idx = self.indexed_patterns.len() as u32;
            let ngram_id = self.intern_ngram(sub.as_bytes().to_vec());
            self.ngram_postings[ngram_id].push(pat_idx);
            self.indexed_patterns.push(IndexedPattern {
                pred_id,
                required_ngrams: 1,
                verifier: StringVerifier::Contains(sub.to_owned()),
            });
        } else {
            self.fallback_patterns.push(FallbackPattern {
                pred_id,
                verifier: StringVerifier::Contains(sub.to_owned()),
            });
        }
    }

    fn add_compiled_regex(&mut self, pred_id: usize, hir: &Hir, re: MetaRegex) {
        let verifier = StringVerifier::Regex(re);

        let ngrams = extract_required_ngrams(hir);
        if ngrams.is_empty() || ngrams.len() > u16::MAX as usize {
            self.fallback_patterns
                .push(FallbackPattern { pred_id, verifier });
        } else {
            let pat_idx = self.indexed_patterns.len() as u32;
            let required_ngrams = ngrams.len() as u16;
            for ngram in ngrams {
                let ngram_id = self.intern_ngram(ngram);
                self.ngram_postings[ngram_id].push(pat_idx);
            }
            self.indexed_patterns.push(IndexedPattern {
                pred_id,
                required_ngrams,
                verifier,
            });
        }
    }

    fn build(self) -> Result<FieldStringIndex, PolicyError> {
        let ac = if self.unique_ngrams.is_empty() {
            None
        } else {
            Some(
                AhoCorasick::builder()
                    .match_kind(MatchKind::Standard)
                    .build(&self.unique_ngrams)
                    .map_err(|e| PolicyError::Compile(format!("aho-corasick build error: {e}")))?,
            )
        };
        Ok(FieldStringIndex {
            ac,
            ngram_postings: self.ngram_postings,
            indexed_patterns: self.indexed_patterns,
            fallback_patterns: self.fallback_patterns,
        })
    }
}

/// Single compiled field evaluator covering all predicates inspecting a specific `LogFieldSelector`.
#[derive(Debug, Clone)]
struct FieldEvaluator {
    field: LogFieldSelector,
    /// Predicate IDs for `Predicate::Exists`.
    exists_preds: Vec<usize>,
    /// O(1) hash map for `Predicate::Equals(ScalarValue::String(s))` -> predicate IDs.
    str_equals_map: AHashMap<String, Vec<usize>>,
    /// Pre-decoded 16-byte raw `TraceId` exact-match literals (`policy.md` "Identifier Comparison").
    trace_id_equals_map: AHashMap<[u8; 16], Vec<usize>>,
    /// Pre-decoded 8-byte raw `SpanId` exact-match literals (`policy.md` "Identifier Comparison").
    span_id_equals_map: AHashMap<[u8; 8], Vec<usize>>,
    /// N-gram inverted index + `MetaRegex` verifiers for string `Regex` and `Contains(String)`.
    string_index: FieldStringIndex,
    /// `(pred_id, ScalarValue)` for `Predicate::Contains` (needed when field value is `ExtractedValue::Array` or `ExtractedValue::Bytes`).
    contains_preds: Vec<(usize, ScalarValue)>,
    /// Non-string `Equals` and numeric range (`Gt`, `Gte`, `Lt`, `Lte`) predicates.
    scalar_and_range_preds: Vec<(usize, Predicate)>,
}

impl FieldEvaluator {
    #[inline]
    fn eval_str_only<F>(&self, s: &str, scratch: &mut StringScanScratch, mut on_match: F)
    where
        F: FnMut(usize),
    {
        for &pid in &self.exists_preds {
            on_match(pid);
        }
        if let Some(pids) = self.str_equals_map.get(s) {
            for &pid in pids {
                on_match(pid);
            }
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
    fn is_exists_only(&self) -> bool {
        self.string_index.is_empty()
            && self.scalar_and_range_preds.is_empty()
            && self.contains_preds.is_empty()
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
        for &pid in &self.exists_preds {
            on_match(pid);
        }

        match val {
            ExtractedValue::Str(s) => {
                if let Some(pids) = self.str_equals_map.get(s.as_ref()) {
                    for &pid in pids {
                        on_match(pid);
                    }
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

/// Pre-resolved column references for an OTAP `StructArray` (`logs.body`) or attribute `RecordBatch`.
struct AnyValColumns<'a> {
    struct_validity: Option<&'a NullBuffer>,
    type_arr: &'a UInt8Array,
    str_col: Option<&'a ArrayRef>,
    int_col: Option<&'a ArrayRef>,
    double_arr: Option<&'a Float64Array>,
    bool_arr: Option<&'a BooleanArray>,
    bytes_col: Option<&'a ArrayRef>,
    ser_col: Option<&'a ArrayRef>,
}

impl<'a> AnyValColumns<'a> {
    fn from_struct_array(sa: &'a StructArray) -> Option<Self> {
        let type_arr = sa
            .column_by_name(consts::ATTRIBUTE_TYPE)?
            .as_any()
            .downcast_ref::<UInt8Array>()?;
        Some(Self {
            struct_validity: sa.nulls(),
            type_arr,
            str_col: sa.column_by_name(consts::ATTRIBUTE_STR),
            int_col: sa.column_by_name(consts::ATTRIBUTE_INT),
            double_arr: sa
                .column_by_name(consts::ATTRIBUTE_DOUBLE)
                .and_then(|c| c.as_any().downcast_ref::<Float64Array>()),
            bool_arr: sa
                .column_by_name(consts::ATTRIBUTE_BOOL)
                .and_then(|c| c.as_any().downcast_ref::<BooleanArray>()),
            bytes_col: sa.column_by_name(consts::ATTRIBUTE_BYTES),
            ser_col: sa.column_by_name(consts::ATTRIBUTE_SER),
        })
    }

    fn from_attr_batch(rb: &'a RecordBatch) -> Option<Self> {
        let type_arr = rb
            .column_by_name(consts::ATTRIBUTE_TYPE)?
            .as_any()
            .downcast_ref::<UInt8Array>()?;
        Some(Self {
            struct_validity: None,
            type_arr,
            str_col: rb.column_by_name(consts::ATTRIBUTE_STR),
            int_col: rb.column_by_name(consts::ATTRIBUTE_INT),
            double_arr: rb
                .column_by_name(consts::ATTRIBUTE_DOUBLE)
                .and_then(|c| c.as_any().downcast_ref::<Float64Array>()),
            bool_arr: rb
                .column_by_name(consts::ATTRIBUTE_BOOL)
                .and_then(|c| c.as_any().downcast_ref::<BooleanArray>()),
            bytes_col: rb.column_by_name(consts::ATTRIBUTE_BYTES),
            ser_col: rb.column_by_name(consts::ATTRIBUTE_SER),
        })
    }

    #[inline]
    fn is_str_row(&self, row: usize) -> bool {
        if let Some(nulls) = self.struct_validity
            && !nulls.is_valid(row)
        {
            return false;
        }
        self.type_arr.is_valid(row) && self.type_arr.value(row) == AttributeValueType::Str as u8
    }

    #[inline]
    fn is_cbor_row(&self, row: usize) -> bool {
        if let Some(nulls) = self.struct_validity
            && !nulls.is_valid(row)
        {
            return false;
        }
        self.type_arr.is_valid(row)
            && matches!(
                AttributeValueType::try_from(self.type_arr.value(row)),
                Ok(AttributeValueType::Map | AttributeValueType::Slice)
            )
    }

    #[inline]
    fn raw_cbor_at_row(&self, row: usize) -> Option<&'a [u8]> {
        if !self.is_cbor_row(row) {
            return None;
        }
        extract_bytes_from_array(self.ser_col?, row)
    }

    #[inline]
    fn extract_at_row(&self, row: usize) -> Option<ExtractedValue<'a>> {
        if let Some(nulls) = self.struct_validity
            && !nulls.is_valid(row)
        {
            return None;
        }
        if !self.type_arr.is_valid(row) {
            return None;
        }
        let vtype = AttributeValueType::try_from(self.type_arr.value(row)).ok()?;
        match vtype {
            AttributeValueType::Empty => None,
            // Per pdata's `AnyValuesRecordsBuilder` (`default_values_optional: true`) and
            // `ResolvedAnyValue::resolve`, omitted or null str/int/double/bytes slots encode
            // their respective type defaults (`""`, `0`, `0.0`, `&[]`).
            AttributeValueType::Str => Some(ExtractedValue::Str(Cow::Borrowed(
                self.str_col
                    .and_then(|c| extract_str_from_array(c, row))
                    .unwrap_or(""),
            ))),
            AttributeValueType::Int => Some(ExtractedValue::Int(
                self.int_col
                    .and_then(|c| extract_i64_from_array(c, row))
                    .unwrap_or(0),
            )),
            AttributeValueType::Double => Some(ExtractedValue::Double(
                self.double_arr
                    .and_then(|da| da.is_valid(row).then(|| da.value(row)))
                    .unwrap_or(0.0),
            )),
            AttributeValueType::Bool => {
                let ba = self.bool_arr?;
                ba.is_valid(row)
                    .then(|| ExtractedValue::Bool(ba.value(row)))
            }
            AttributeValueType::Bytes => Some(ExtractedValue::Bytes(Cow::Borrowed(
                self.bytes_col
                    .and_then(|c| extract_bytes_from_array(c, row))
                    .unwrap_or(&[]),
            ))),
            AttributeValueType::Map | AttributeValueType::Slice => {
                let bytes = extract_bytes_from_array(self.ser_col?, row)?;
                decode_cbor_value(bytes)
            }
        }
    }
}

/// Borrowed view over a `DictionaryArray<UInt8Type | UInt16Type>` backed by a `StringArray`.
enum DictStrCol<'a> {
    UInt8 {
        arr: &'a DictionaryArray<UInt8Type>,
        values: &'a StringArray,
    },
    UInt16 {
        arr: &'a DictionaryArray<UInt16Type>,
        values: &'a StringArray,
    },
}

impl<'a> DictStrCol<'a> {
    fn try_from_array(col: &'a ArrayRef) -> Option<Self> {
        if let Some(arr) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>()
            && let Some(values) = arr.values().as_any().downcast_ref::<StringArray>()
        {
            return Some(Self::UInt8 { arr, values });
        }
        if let Some(arr) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>()
            && let Some(values) = arr.values().as_any().downcast_ref::<StringArray>()
        {
            return Some(Self::UInt16 { arr, values });
        }
        None
    }

    #[inline]
    fn dict_len(&self) -> usize {
        match self {
            Self::UInt8 { values, .. } | Self::UInt16 { values, .. } => values.len(),
        }
    }

    #[inline]
    fn values(&self) -> &'a StringArray {
        match self {
            Self::UInt8 { values, .. } | Self::UInt16 { values, .. } => values,
        }
    }

    /// Returns `Some(dict_idx)` when both the row's key and the referenced dictionary value are valid.
    #[inline]
    fn valid_dict_idx(&self, row: usize) -> Option<usize> {
        match self {
            Self::UInt8 { arr, values } => {
                if !arr.is_valid(row) {
                    return None;
                }
                let k = arr.keys().value(row) as usize;
                (k < values.len() && values.is_valid(k)).then_some(k)
            }
            Self::UInt16 { arr, values } => {
                if !arr.is_valid(row) {
                    return None;
                }
                let k = arr.keys().value(row) as usize;
                (k < values.len() && values.is_valid(k)).then_some(k)
            }
        }
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
            ev.eval_str_only(val, scratch, |pid| {
                self.matched_preds.push(pid);
            });
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

enum DictKeyStorage<'a> {
    Direct(Vec<Option<Option<&'a [usize]>>>),
    Sparse(AHashMap<usize, Option<&'a [usize]>>),
}

/// Lazy per-dictionary-key cache for resolving watched attribute keys in `evaluate_attr_table`.
struct DictKeyLookupCache<'a> {
    storage: DictKeyStorage<'a>,
}

impl<'a> DictKeyLookupCache<'a> {
    fn new(dict_len: usize) -> Self {
        let storage = if dict_len <= DIRECT_DICT_CACHE_LIMIT {
            DictKeyStorage::Direct(vec![None; dict_len])
        } else {
            DictKeyStorage::Sparse(AHashMap::new())
        };
        Self { storage }
    }

    #[inline]
    fn get_or_lookup(
        &mut self,
        dict_idx: usize,
        dict_values: &StringArray,
        key_map: &'a AHashMap<String, Vec<usize>>,
    ) -> Option<&'a [usize]> {
        match &mut self.storage {
            DictKeyStorage::Direct(slots) => {
                if let Some(hit) = slots[dict_idx] {
                    return hit;
                }
                let resolved = key_map.get(dict_values.value(dict_idx)).map(Vec::as_slice);
                slots[dict_idx] = Some(resolved);
                resolved
            }
            DictKeyStorage::Sparse(map) => *map
                .entry(dict_idx)
                .or_insert_with(|| key_map.get(dict_values.value(dict_idx)).map(Vec::as_slice)),
        }
    }
}

fn eval_str_column(
    ev: &FieldEvaluator,
    col: &ArrayRef,
    parent_validity: Option<&NullBuffer>,
    num_rows: usize,
    skip_empty: bool,
    scratch: &mut StringScanScratch,
    pred_bitmaps: &mut [RoaringBitmap],
) {
    if let Some(dict) = DictStrCol::try_from_array(col) {
        let mut cache = DictStrEvalCache::new(dict.dict_len());
        let values = dict.values();
        for row in 0..num_rows {
            if parent_validity.is_some_and(|n| !n.is_valid(row)) {
                continue;
            }
            if let Some(dict_idx) = dict.valid_dict_idx(row) {
                let matched = cache.eval_or_get(dict_idx, values, skip_empty, ev, scratch);
                if !matched.is_empty() {
                    let row_u32 = row as u32;
                    for &pid in matched {
                        let _ = pred_bitmaps[pid].insert(row_u32);
                    }
                }
            }
        }
        return;
    }

    for row in 0..num_rows {
        if parent_validity.is_some_and(|n| !n.is_valid(row)) {
            continue;
        }
        if let Some(s) = extract_str_from_array(col, row)
            && (!skip_empty || !s.is_empty())
        {
            let row_u32 = row as u32;
            ev.eval_str_only(s, scratch, |pid| {
                let _ = pred_bitmaps[pid].insert(row_u32);
            });
        }
    }
}

/// Compiled matcher reference (`pred_id` + `negate`).
#[derive(Debug, Clone, Copy)]
struct CompiledMatcherRef {
    pred_id: usize,
    negate: bool,
}

/// Rule matchers partitioned by evaluation domain so resource and scope attribute conjunctions
/// combine in compact ID space before checking log rows.
#[derive(Debug, Clone, Default)]
struct CompiledRuleMatchers {
    row_matchers: Vec<CompiledMatcherRef>,
    resource_matchers: Vec<CompiledMatcherRef>,
    scope_matchers: Vec<CompiledMatcherRef>,
}

/// Result of evaluating a conjunction of predicates in resource-ID or scope-ID space.
enum IdFilter {
    /// Every log row satisfies the ID-space constraint (including rows with null/missing parent ID).
    PassAll,
    /// No log row can satisfy the ID-space constraint (short-circuits row evaluation).
    Empty,
    /// Log row must carry a valid parent ID present in this bitmap.
    KeepIds(RoaringBitmap),
    /// Log row passes unless it carries a valid parent ID present in this bitmap.
    ExcludeIds(RoaringBitmap),
}

/// Compiled filter policy rule.
#[derive(Debug, Clone)]
struct CompiledFilterRule {
    matchers: CompiledRuleMatchers,
    action: Action,
}

/// Compiled transform policy matcher conjunction aligned with `CompiledTransformPolicy`.
#[derive(Debug, Clone)]
struct CompiledTransformRule {
    matchers: CompiledRuleMatchers,
}

/// Opaque set of kept log row indices (`0..num_rows`) returned by [`PolicyEngine::evaluate_filter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RowSelection {
    bitmap: RoaringBitmap,
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
    pub(crate) fn into_bitmap(self) -> RoaringBitmap {
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

/// Compiled predicate evaluation plan for a single pipeline stage (filter or transform).
///
/// Keeping filter and transform plans separate ensures each stage allocates predicate bitmaps and
/// evaluates string/regex indices only for predicates active in that stage.
#[derive(Debug, Clone)]
struct StagePlan {
    num_predicates: usize,
    field_evaluators: Vec<FieldEvaluator>,
    /// Indices into `field_evaluators` for top-level / `BodyField` selectors.
    root_evaluator_indices: Vec<usize>,
    /// Keyed indices into `field_evaluators` for `LogAttribute(path)` keyed by `path[0]`.
    log_attr_evaluators: AHashMap<String, Vec<usize>>,
    /// Keyed indices into `field_evaluators` for `ResourceAttribute(path)` keyed by `path[0]`.
    res_attr_evaluators: AHashMap<String, Vec<usize>>,
    /// Keyed indices into `field_evaluators` for `ScopeAttribute(path)` keyed by `path[0]`.
    scope_attr_evaluators: AHashMap<String, Vec<usize>>,
}

/// Pure-Rust columnar policy evaluation engine: HIR N-Gram Inverted Index (`AhoCorasick`)
/// with `regex_automata::meta::Regex` verification and columnar OTAP execution.
#[derive(Debug, Clone)]
pub struct PolicyEngine {
    filter_plan: StagePlan,
    transform_plan: StagePlan,
    filter_rules: Vec<CompiledFilterRule>,
    transform_rules: Vec<CompiledTransformRule>,
    compiled_transforms: Vec<CompiledTransformPolicy>,
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
    if v.is_nan() {
        f64::NAN.to_bits()
    } else if v == 0.0 {
        0.0_f64.to_bits()
    } else {
        v.to_bits()
    }
}

/// Per-field builder used during `PolicyEngine::compile`.
struct FieldBuilder {
    field: LogFieldSelector,
    pred_dedup: AHashMap<CanonicalPredicateKey, usize>,
    exists_preds: Vec<usize>,
    str_equals_map: AHashMap<String, Vec<usize>>,
    trace_id_equals_map: AHashMap<[u8; 16], Vec<usize>>,
    span_id_equals_map: AHashMap<[u8; 8], Vec<usize>>,
    string_index_builder: FieldStringIndexBuilder,
    contains_preds: Vec<(usize, ScalarValue)>,
    scalar_and_range_preds: Vec<(usize, Predicate)>,
}

impl FieldBuilder {
    fn new(field: LogFieldSelector) -> Self {
        Self {
            field,
            pred_dedup: AHashMap::new(),
            exists_preds: Vec::new(),
            str_equals_map: AHashMap::new(),
            trace_id_equals_map: AHashMap::new(),
            span_id_equals_map: AHashMap::new(),
            string_index_builder: FieldStringIndexBuilder::default(),
            contains_preds: Vec::new(),
            scalar_and_range_preds: Vec::new(),
        }
    }

    fn add_predicate(
        &mut self,
        predicate: &Predicate,
        compiled_regex: Option<(Hir, MetaRegex)>,
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
                self.exists_preds.push(pred_id);
            }
            Predicate::Equals(ScalarValue::String(s)) => match &self.field {
                LogFieldSelector::RecordField(LogRecordField::TraceId) => {
                    if let Some(bytes) = decode_canonical_lower_hex::<16>(s) {
                        self.trace_id_equals_map
                            .entry(bytes)
                            .or_default()
                            .push(pred_id);
                    }
                }
                LogFieldSelector::RecordField(LogRecordField::SpanId) => {
                    if let Some(bytes) = decode_canonical_lower_hex::<8>(s) {
                        self.span_id_equals_map
                            .entry(bytes)
                            .or_default()
                            .push(pred_id);
                    }
                }
                _ => {
                    self.str_equals_map
                        .entry(s.clone())
                        .or_default()
                        .push(pred_id);
                }
            },
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
                    self.string_index_builder.add_contains(pred_id, s);
                }
                self.contains_preds.push((pred_id, scalar.clone()));
            }
            Predicate::Regex(pat) => {
                let (hir, re) = match compiled_regex {
                    Some(compiled) => compiled,
                    None => compile_matcher_regex(pat)?,
                };
                self.string_index_builder
                    .add_compiled_regex(pred_id, &hir, re);
            }
        }

        Ok(pred_id)
    }

    fn build(self) -> Result<FieldEvaluator, PolicyError> {
        Ok(FieldEvaluator {
            field: self.field,
            exists_preds: self.exists_preds,
            str_equals_map: self.str_equals_map,
            trace_id_equals_map: self.trace_id_equals_map,
            span_id_equals_map: self.span_id_equals_map,
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
        compiled_regex: Option<(Hir, MetaRegex)>,
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
            compiled_regex,
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

fn validate_log_matcher(
    idx: usize,
    m: &LogMatcher,
    field_index_map: &AHashMap<LogFieldSelector, usize>,
    field_builders: &[FieldBuilder],
    policy_errors: &mut Vec<String>,
) -> Option<(Hir, MetaRegex)> {
    match &m.field {
        LogFieldSelector::RecordField(LogRecordField::Unspecified) => {
            policy_errors.push(format!("matches[{idx}]: record_field is unspecified"));
        }
        LogFieldSelector::ScopeField(ScopeField::Unspecified) => {
            policy_errors.push(format!("matches[{idx}]: scope_field is unspecified"));
        }
        LogFieldSelector::LogAttribute(p) if p.path.is_empty() => {
            policy_errors.push(format!("matches[{idx}]: log_attribute path is empty"));
        }
        LogFieldSelector::ResourceAttribute(p) if p.path.is_empty() => {
            policy_errors.push(format!("matches[{idx}]: resource_attribute path is empty"));
        }
        LogFieldSelector::ScopeAttribute(p) if p.path.is_empty() => {
            policy_errors.push(format!("matches[{idx}]: scope_attribute path is empty"));
        }
        LogFieldSelector::BodyField(p) if p.path.is_empty() => {
            policy_errors.push(format!("matches[{idx}]: body_field path is empty"));
        }
        _ => {}
    }

    if let Predicate::Regex(pat) = &m.predicate {
        let already_compiled = field_index_map.get(&m.field).is_some_and(|&fb_idx| {
            field_builders[fb_idx]
                .pred_dedup
                .contains_key(&CanonicalPredicateKey::Regex(pat.clone()))
        });
        if !already_compiled {
            match compile_matcher_regex(pat) {
                Ok(compiled) => return Some(compiled),
                Err(e) => policy_errors.push(format!("matches[{idx}]: {e}")),
            }
        }
    }
    None
}

fn validate_attr_selector(loc: &str, sel: &LogAttributeSelector, policy_errors: &mut Vec<String>) {
    match sel {
        LogAttributeSelector::LogAttribute(p) => {
            if p.path.is_empty() {
                policy_errors.push(format!("{loc}: log_attribute path is empty"));
            } else if p.path.len() > 1 {
                policy_errors.push(format!(
                    "{loc}: nested log_attribute transform paths are not yet supported"
                ));
            }
        }
        LogAttributeSelector::BodyField(p) => {
            if p.path.is_empty() {
                policy_errors.push(format!("{loc}: body_field path is empty"));
            } else {
                policy_errors.push(format!(
                    "{loc}: body_field transform writes are not yet supported"
                ));
            }
        }
    }
}

fn validate_target_selector(
    loc: &str,
    sel: &LogTargetSelector,
    is_redact: bool,
    policy_errors: &mut Vec<String>,
) {
    match sel {
        LogTargetSelector::RecordField(LogRecordField::Unspecified) => {
            policy_errors.push(format!("{loc}: record_field is unspecified"));
        }
        LogTargetSelector::RecordField(LogRecordField::TraceId | LogRecordField::SpanId)
            if !is_redact =>
        {
            policy_errors.push(format!(
                "{loc}: trace_id/span_id transform writes are not yet supported"
            ));
        }
        LogTargetSelector::RecordField(_) => {}
        LogTargetSelector::LogAttribute(p) => {
            if p.path.is_empty() {
                policy_errors.push(format!("{loc}: log_attribute path is empty"));
            } else if p.path.len() > 1 {
                policy_errors.push(format!(
                    "{loc}: nested log_attribute transform paths are not yet supported"
                ));
            }
        }
        LogTargetSelector::BodyField(p) => {
            if p.path.is_empty() {
                policy_errors.push(format!("{loc}: body_field path is empty"));
            } else {
                policy_errors.push(format!(
                    "{loc}: body_field transform writes are not yet supported"
                ));
            }
        }
    }
}

fn validate_and_compile_transform(
    tx: &LogTransformPolicy,
    field_index_map: &AHashMap<LogFieldSelector, usize>,
    field_builders: &[FieldBuilder],
    policy_errors: &mut Vec<String>,
) -> (Vec<Option<(Hir, MetaRegex)>>, Vec<CompiledLogRedact>) {
    if tx.rename.is_empty() && tx.add.is_empty() && tx.remove.is_empty() && tx.redact.is_empty() {
        policy_errors.push("transform policy must contain at least one operation".into());
    }

    let mut compiled_matchers = Vec::with_capacity(tx.matches.len());
    for (idx, m) in tx.matches.iter().enumerate() {
        compiled_matchers.push(validate_log_matcher(
            idx,
            m,
            field_index_map,
            field_builders,
            policy_errors,
        ));
    }

    for (idx, op) in tx.rename.iter().enumerate() {
        validate_attr_selector(&format!("rename[{idx}].from"), &op.from, policy_errors);
        validate_attr_selector(&format!("rename[{idx}].to"), &op.to, policy_errors);
    }
    for (idx, op) in tx.add.iter().enumerate() {
        validate_target_selector(
            &format!("add[{idx}].target"),
            &op.target,
            false,
            policy_errors,
        );
    }
    for (idx, op) in tx.remove.iter().enumerate() {
        validate_target_selector(
            &format!("remove[{idx}].target"),
            &op.target,
            false,
            policy_errors,
        );
    }

    let mut compiled_redacts = Vec::with_capacity(tx.redact.len());
    for (idx, op) in tx.redact.iter().enumerate() {
        validate_target_selector(
            &format!("redact[{idx}].target"),
            &op.target,
            true,
            policy_errors,
        );
        let compiled_re = match &op.regex {
            Some(pat) => match compile_redact_regex(pat) {
                Ok(re) => Some(re),
                Err(e) => {
                    policy_errors.push(format!("redact[{idx}].regex: {e}"));
                    None
                }
            },
            None => None,
        };
        if policy_errors.is_empty() {
            compiled_redacts.push(CompiledLogRedact {
                target: op.target.clone(),
                replacement: op.effective_replacement().to_owned(),
                regex: compiled_re,
            });
        }
    }

    (compiled_matchers, compiled_redacts)
}

impl PolicyEngine {
    /// Compiles a slice of `Policy` definitions into a [`CompileOutput`].
    ///
    /// Each enabled policy is validated atomically before any of its matchers or transforms are
    /// registered (`policy.md` "Compilation Errors"). Invalid or unsupported policies are kept
    /// inert and reported in [`CompileOutput::errors`], while valid policies in the same batch
    /// continue to compile and execute.
    pub fn compile(policies: &[Policy]) -> Result<CompileOutput, PolicyError> {
        let mut filter_builder = StagePlanBuilder::default();
        let mut transform_builder = StagePlanBuilder::default();
        let mut errors: Vec<PolicyCompileError> = Vec::new();

        // Count enabled policy IDs up front so duplicate ID rejection is commutative (order-independent).
        let mut id_counts: AHashMap<&str, usize> = AHashMap::with_capacity(policies.len());
        for policy in policies {
            if policy.enabled && !policy.id.is_empty() {
                *id_counts.entry(policy.id.as_str()).or_insert(0) += 1;
            }
        }

        let mut filter_rules = Vec::new();
        let mut validated_transforms: Vec<(
            String,
            &LogTransformPolicy,
            Vec<Option<(Hir, MetaRegex)>>,
            CompiledTransformPolicy,
        )> = Vec::new();

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
                    let mut compiled_matchers = Vec::with_capacity(lf.matches.len());
                    for (idx, m) in lf.matches.iter().enumerate() {
                        compiled_matchers.push(validate_log_matcher(
                            idx,
                            m,
                            &filter_builder.field_index_map,
                            &filter_builder.field_builders,
                            &mut policy_errors,
                        ));
                    }
                    if policy_errors.is_empty() {
                        let mut matchers = CompiledRuleMatchers::default();
                        for (m, compiled_re) in lf.matches.iter().zip(compiled_matchers) {
                            let mref = filter_builder.register_matcher(m, compiled_re)?;
                            match &m.field {
                                LogFieldSelector::ResourceAttribute(_) => {
                                    matchers.resource_matchers.push(mref);
                                }
                                LogFieldSelector::ScopeAttribute(_) => {
                                    matchers.scope_matchers.push(mref);
                                }
                                _ => matchers.row_matchers.push(mref),
                            }
                        }
                        filter_rules.push(CompiledFilterRule {
                            matchers,
                            action: lf.action,
                        });
                    }
                }
                PolicyDefinition::LogTransform(tx) => {
                    let (compiled_matchers, compiled_redacts) = validate_and_compile_transform(
                        tx,
                        &transform_builder.field_index_map,
                        &transform_builder.field_builders,
                        &mut policy_errors,
                    );
                    if policy_errors.is_empty() {
                        let compiled_tx = CompiledTransformPolicy {
                            rename: tx.rename.clone(),
                            add: tx.add.clone(),
                            remove: tx.remove.clone(),
                            redact: compiled_redacts,
                        };
                        validated_transforms.push((
                            policy.id.clone(),
                            tx,
                            compiled_matchers,
                            compiled_tx,
                        ));
                    }
                }
                PolicyDefinition::MetricFilter(_) => {
                    policy_errors.push("metric filter policies are not yet supported".into());
                }
                PolicyDefinition::TraceSampling(_) => {
                    policy_errors.push("trace sampling policies are not yet supported".into());
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
        for (_policy_id, tx, compiled_matchers, compiled_tx) in validated_transforms {
            let mut matchers = CompiledRuleMatchers::default();
            for (m, compiled_re) in tx.matches.iter().zip(compiled_matchers) {
                let mref = transform_builder.register_matcher(m, compiled_re)?;
                match &m.field {
                    LogFieldSelector::ResourceAttribute(_) => {
                        matchers.resource_matchers.push(mref);
                    }
                    LogFieldSelector::ScopeAttribute(_) => {
                        matchers.scope_matchers.push(mref);
                    }
                    _ => matchers.row_matchers.push(mref),
                }
            }
            transform_rules.push(CompiledTransformRule { matchers });
            compiled_transforms.push(compiled_tx);
        }

        let filter_plan = filter_builder.build()?;
        let transform_plan = transform_builder.build()?;

        Ok(CompileOutput {
            engine: Self {
                filter_plan,
                transform_plan,
                filter_rules,
                transform_rules,
                compiled_transforms,
            },
            errors,
        })
    }

    #[cfg(test)]
    pub(crate) fn num_filter_predicates(&self) -> usize {
        self.filter_plan.num_predicates
    }

    #[cfg(test)]
    pub(crate) fn num_transform_predicates(&self) -> usize {
        self.transform_plan.num_predicates
    }

    #[cfg(test)]
    pub(crate) fn num_predicates(&self) -> usize {
        self.filter_plan.num_predicates + self.transform_plan.num_predicates
    }

    #[cfg(test)]
    pub(crate) fn evaluate_filter_predicates_for_test(
        &self,
        view: &LogsBatchView<'_>,
        scratch: &mut EvaluationScratch,
    ) -> Vec<RoaringBitmap> {
        self.evaluate_predicates(&self.filter_plan, view, scratch)
    }

    fn evaluate_predicates(
        &self,
        plan: &StagePlan,
        view: &LogsBatchView<'_>,
        scratch: &mut EvaluationScratch,
    ) -> Vec<RoaringBitmap> {
        let mut pred_bitmaps = vec![RoaringBitmap::new(); plan.num_predicates];
        if plan.num_predicates == 0 {
            return pred_bitmaps;
        }

        scratch.ensure_string_scratches(plan.field_evaluators.len());

        // 1. Evaluate root / scope / body fields.
        for &ev_idx in &plan.root_evaluator_indices {
            let ev = &plan.field_evaluators[ev_idx];
            let str_scratch = &mut scratch.string_scratches[ev_idx];
            ev.string_index.prepare_scratch(str_scratch);
            self.evaluate_root_field(ev, view, str_scratch, &mut pred_bitmaps);
        }

        // 2. Evaluate LogAttrs in a single pass over the LogAttrs RecordBatch.
        if !plan.log_attr_evaluators.is_empty()
            && let Some(rb) = view.records.get(ArrowPayloadType::LogAttrs)
        {
            let log_id_to_row = view.log_id_to_row();
            if !log_id_to_row.is_empty() {
                Self::evaluate_attr_table(
                    plan,
                    rb,
                    &plan.log_attr_evaluators,
                    &mut scratch.string_scratches,
                    |parent_id, pid, bitmaps| {
                        if let Some(&row) = log_id_to_row.get(parent_id as usize)
                            && row != u32::MAX
                        {
                            let _ = bitmaps[pid].insert(row);
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
            Self::evaluate_attr_table(
                plan,
                rb,
                &plan.res_attr_evaluators,
                &mut scratch.string_scratches,
                |parent_id, pid, bitmaps| {
                    let _ = bitmaps[pid].insert(u32::from(parent_id));
                },
                &mut pred_bitmaps,
            );
        }

        // 4. Evaluate ScopeAttrs in scope-ID space (bitmap stores matching scope.id values).
        if !plan.scope_attr_evaluators.is_empty()
            && let Some(rb) = view.records.get(ArrowPayloadType::ScopeAttrs)
        {
            Self::evaluate_attr_table(
                plan,
                rb,
                &plan.scope_attr_evaluators,
                &mut scratch.string_scratches,
                |parent_id, pid, bitmaps| {
                    let _ = bitmaps[pid].insert(u32::from(parent_id));
                },
                &mut pred_bitmaps,
            );
        }

        pred_bitmaps
    }

    fn evaluate_root_field(
        &self,
        ev: &FieldEvaluator,
        view: &LogsBatchView<'_>,
        scratch: &mut StringScanScratch,
        pred_bitmaps: &mut [RoaringBitmap],
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
                        let Some(cols) = AnyValColumns::from_struct_array(body_struct) else {
                            return;
                        };
                        let str_dict = cols.str_col.and_then(DictStrCol::try_from_array);
                        let mut str_cache = str_dict
                            .as_ref()
                            .map(|d| DictStrEvalCache::new(d.dict_len()));
                        for row in 0..num_rows {
                            if let (Some(dict), Some(cache)) = (&str_dict, &mut str_cache)
                                && cols.is_str_row(row)
                            {
                                if let Some(dict_idx) = dict.valid_dict_idx(row) {
                                    let matched = cache.eval_or_get(
                                        dict_idx,
                                        dict.values(),
                                        false,
                                        ev,
                                        scratch,
                                    );
                                    if !matched.is_empty() {
                                        let row_u32 = row as u32;
                                        for &pid in matched {
                                            let _ = pred_bitmaps[pid].insert(row_u32);
                                        }
                                    }
                                } else {
                                    // Null slot in an AttributeValueType::Str body encodes `""`.
                                    let row_u32 = row as u32;
                                    ev.eval_str_only("", scratch, |pid| {
                                        let _ = pred_bitmaps[pid].insert(row_u32);
                                    });
                                }
                                continue;
                            }
                            if cols.is_cbor_row(row) {
                                if let Some(raw) = cols.raw_cbor_at_row(row)
                                    && let Some(val) = extract_cbor_path_for_eval(
                                        raw,
                                        &[],
                                        ev.needs_array_elements(),
                                    )
                                {
                                    let row_u32 = row as u32;
                                    ev.eval_extracted_value(&val, scratch, |pid| {
                                        let _ = pred_bitmaps[pid].insert(row_u32);
                                    });
                                }
                                continue;
                            }
                            if let Some(val) = cols.extract_at_row(row) {
                                let row_u32 = row as u32;
                                ev.eval_extracted_value(&val, scratch, |pid| {
                                    let _ = pred_bitmaps[pid].insert(row_u32);
                                });
                            }
                        }
                    } else {
                        eval_str_column(ev, body_col, None, num_rows, false, scratch, pred_bitmaps);
                    }
                }
                LogRecordField::SeverityText => {
                    if let Some(col) = view.root.column_by_name(consts::SEVERITY_TEXT) {
                        eval_str_column(ev, col, None, num_rows, true, scratch, pred_bitmaps);
                    }
                }
                LogRecordField::EventName => {
                    if let Some(col) = view.root.column_by_name(consts::EVENT_NAME) {
                        eval_str_column(ev, col, None, num_rows, true, scratch, pred_bitmaps);
                    }
                }
                LogRecordField::SeverityNumber => {
                    if let Some(col) = view.root.column_by_name(consts::SEVERITY_NUMBER) {
                        for row in 0..num_rows {
                            if let Some(v) = extract_i64_from_array(col, row)
                                && v != 0
                            {
                                let row_u32 = row as u32;
                                let val = ExtractedValue::Int(v);
                                ev.eval_extracted_value(&val, scratch, |pid| {
                                    let _ = pred_bitmaps[pid].insert(row_u32);
                                });
                            }
                        }
                    }
                }
                LogRecordField::TraceId => {
                    if let Some(col) = view.root.column_by_name(consts::TRACE_ID) {
                        let needs_hex = !ev.string_index.is_empty();
                        let mut hex_buf = [0u8; 32];
                        for row in 0..num_rows {
                            if let Some(bytes) = extract_fixed_bytes_from_array(col, row)
                                && let Ok(id_bytes) = <[u8; 16]>::try_from(bytes)
                                && !id_bytes.iter().all(|&b| b == 0)
                            {
                                let row_u32 = row as u32;
                                for &pid in &ev.exists_preds {
                                    let _ = pred_bitmaps[pid].insert(row_u32);
                                }
                                if let Some(pids) = ev.trace_id_equals_map.get(&id_bytes) {
                                    for &pid in pids {
                                        let _ = pred_bitmaps[pid].insert(row_u32);
                                    }
                                }
                                if needs_hex
                                    && hex::encode_to_slice(id_bytes, &mut hex_buf).is_ok()
                                    && let Ok(hex_str) = std::str::from_utf8(&hex_buf)
                                {
                                    ev.string_index.evaluate_str(hex_str, scratch, |pid| {
                                        let _ = pred_bitmaps[pid].insert(row_u32);
                                    });
                                }
                            }
                        }
                    }
                }
                LogRecordField::SpanId => {
                    if let Some(col) = view.root.column_by_name(consts::SPAN_ID) {
                        let needs_hex = !ev.string_index.is_empty();
                        let mut hex_buf = [0u8; 16];
                        for row in 0..num_rows {
                            if let Some(bytes) = extract_fixed_bytes_from_array(col, row)
                                && let Ok(id_bytes) = <[u8; 8]>::try_from(bytes)
                                && !id_bytes.iter().all(|&b| b == 0)
                            {
                                let row_u32 = row as u32;
                                for &pid in &ev.exists_preds {
                                    let _ = pred_bitmaps[pid].insert(row_u32);
                                }
                                if let Some(pids) = ev.span_id_equals_map.get(&id_bytes) {
                                    for &pid in pids {
                                        let _ = pred_bitmaps[pid].insert(row_u32);
                                    }
                                }
                                if needs_hex
                                    && hex::encode_to_slice(id_bytes, &mut hex_buf).is_ok()
                                    && let Ok(hex_str) = std::str::from_utf8(&hex_buf)
                                {
                                    ev.string_index.evaluate_str(hex_str, scratch, |pid| {
                                        let _ = pred_bitmaps[pid].insert(row_u32);
                                    });
                                }
                            }
                        }
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
                    eval_str_column(ev, col, parent_nulls, num_rows, true, scratch, pred_bitmaps);
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
                let decode_arrays = ev.needs_array_elements();
                for row in 0..num_rows {
                    if let Some(raw) = cols.raw_cbor_at_row(row)
                        && let Some(nested_val) =
                            extract_cbor_path_for_eval(raw, &path.path, decode_arrays)
                    {
                        let row_u32 = row as u32;
                        ev.eval_extracted_value(&nested_val, scratch, |pid| {
                            let _ = pred_bitmaps[pid].insert(row_u32);
                        });
                    }
                }
            }
            LogFieldSelector::LogAttribute(_)
            | LogFieldSelector::ResourceAttribute(_)
            | LogFieldSelector::ScopeAttribute(_) => {}
        }
    }

    fn evaluate_attr_table<F>(
        plan: &StagePlan,
        rb: &RecordBatch,
        key_map: &AHashMap<String, Vec<usize>>,
        string_scratches: &mut [StringScanScratch],
        mut record_hit: F,
        pred_bitmaps: &mut [RoaringBitmap],
    ) where
        F: FnMut(u16, usize, &mut [RoaringBitmap]),
    {
        let Some(key_col) = rb.column_by_name(consts::ATTRIBUTE_KEY) else {
            return;
        };
        let Some(parent_ids) = ParentIdColumn::from_batch(rb) else {
            return;
        };
        let Some(cols) = AnyValColumns::from_attr_batch(rb) else {
            return;
        };
        let key_dict = DictStrCol::try_from_array(key_col);
        let mut key_cache = key_dict
            .as_ref()
            .map(|d| DictKeyLookupCache::new(d.dict_len()));
        let str_dict = cols.str_col.and_then(DictStrCol::try_from_array);
        let mut str_caches: Vec<Option<DictStrEvalCache>> = if str_dict.is_some() {
            (0..plan.field_evaluators.len()).map(|_| None).collect()
        } else {
            Vec::new()
        };

        let num_attr_rows = rb.num_rows();
        for attr_row in 0..num_attr_rows {
            let ev_indices: &[usize] = if let (Some(kd), Some(kc)) = (&key_dict, &mut key_cache) {
                let Some(key_dict_idx) = kd.valid_dict_idx(attr_row) else {
                    continue;
                };
                let Some(indices) = kc.get_or_lookup(key_dict_idx, kd.values(), key_map) else {
                    continue;
                };
                indices
            } else {
                let Some(key) = extract_str_from_array(key_col, attr_row) else {
                    continue;
                };
                let Some(indices) = key_map.get(key) else {
                    continue;
                };
                indices.as_slice()
            };
            let Some(parent_id) = parent_ids.value_at(attr_row) else {
                continue;
            };

            if let Some(dict) = &str_dict
                && cols.is_str_row(attr_row)
            {
                let str_dict_idx = dict.valid_dict_idx(attr_row);
                for &ev_idx in ev_indices {
                    let ev = &plan.field_evaluators[ev_idx];
                    let path_segments = match &ev.field {
                        LogFieldSelector::LogAttribute(p)
                        | LogFieldSelector::ResourceAttribute(p)
                        | LogFieldSelector::ScopeAttribute(p) => &p.path,
                        _ => continue,
                    };
                    if path_segments.len() != 1 {
                        continue;
                    }
                    let scratch = &mut string_scratches[ev_idx];
                    ev.string_index.prepare_scratch(scratch);
                    if let Some(dict_idx) = str_dict_idx {
                        let cache = str_caches[ev_idx]
                            .get_or_insert_with(|| DictStrEvalCache::new(dict.dict_len()));
                        let matched =
                            cache.eval_or_get(dict_idx, dict.values(), false, ev, scratch);
                        for &pid in matched {
                            record_hit(parent_id, pid, pred_bitmaps);
                        }
                    } else {
                        // Null slot in an AttributeValueType::Str attribute encodes `""`.
                        ev.eval_str_only("", scratch, |pid| {
                            record_hit(parent_id, pid, pred_bitmaps);
                        });
                    }
                }
                continue;
            }

            if cols.is_cbor_row(attr_row) {
                let raw_opt = cols.raw_cbor_at_row(attr_row);
                for &ev_idx in ev_indices {
                    let ev = &plan.field_evaluators[ev_idx];
                    let path_segments = match &ev.field {
                        LogFieldSelector::LogAttribute(p)
                        | LogFieldSelector::ResourceAttribute(p)
                        | LogFieldSelector::ScopeAttribute(p) => &p.path,
                        _ => continue,
                    };

                    if path_segments.len() == 1 {
                        if ev.is_exists_only() {
                            // Top-level attribute key is present regardless of CBOR contents.
                            for &pid in &ev.exists_preds {
                                record_hit(parent_id, pid, pred_bitmaps);
                            }
                        } else if let Some(raw) = raw_opt
                            && let Some(val) =
                                extract_cbor_path_for_eval(raw, &[], ev.needs_array_elements())
                        {
                            let scratch = &mut string_scratches[ev_idx];
                            ev.string_index.prepare_scratch(scratch);
                            ev.eval_extracted_value(&val, scratch, |pid| {
                                record_hit(parent_id, pid, pred_bitmaps);
                            });
                        } else {
                            // Top-level key is present even when CBOR bytes are missing or undecodable.
                            for &pid in &ev.exists_preds {
                                record_hit(parent_id, pid, pred_bitmaps);
                            }
                        }
                    } else if let Some(raw) = raw_opt
                        && let Some(nested_val) = extract_cbor_path_for_eval(
                            raw,
                            &path_segments[1..],
                            ev.needs_array_elements(),
                        )
                    {
                        let scratch = &mut string_scratches[ev_idx];
                        ev.string_index.prepare_scratch(scratch);
                        ev.eval_extracted_value(&nested_val, scratch, |pid| {
                            record_hit(parent_id, pid, pred_bitmaps);
                        });
                    }
                }
                continue;
            }

            let val_opt = cols.extract_at_row(attr_row);

            for &ev_idx in ev_indices {
                let ev = &plan.field_evaluators[ev_idx];
                let path_segments = match &ev.field {
                    LogFieldSelector::LogAttribute(p)
                    | LogFieldSelector::ResourceAttribute(p)
                    | LogFieldSelector::ScopeAttribute(p) => &p.path,
                    _ => continue,
                };

                if path_segments.len() == 1 {
                    match &val_opt {
                        Some(val) => {
                            let scratch = &mut string_scratches[ev_idx];
                            ev.string_index.prepare_scratch(scratch);
                            ev.eval_extracted_value(val, scratch, |pid| {
                                record_hit(parent_id, pid, pred_bitmaps);
                            });
                        }
                        None => {
                            // Top-level key is present even when its AnyValue is Empty.
                            for &pid in &ev.exists_preds {
                                record_hit(parent_id, pid, pred_bitmaps);
                            }
                        }
                    }
                }
            }
        }
    }

    fn eval_row_conjunction(
        matchers: &[CompiledMatcherRef],
        pred_bitmaps: &[RoaringBitmap],
        all_rows: &RoaringBitmap,
    ) -> RoaringBitmap {
        let Some((first, rest)) = matchers.split_first() else {
            return all_rows.clone();
        };
        let mut acc = if first.negate {
            all_rows - &pred_bitmaps[first.pred_id]
        } else {
            pred_bitmaps[first.pred_id].clone()
        };
        for m in rest {
            if acc.is_empty() {
                break;
            }
            if m.negate {
                acc -= &pred_bitmaps[m.pred_id];
            } else {
                acc &= &pred_bitmaps[m.pred_id];
            }
        }
        acc
    }

    fn eval_id_conjunction(
        matchers: &[CompiledMatcherRef],
        pred_bitmaps: &[RoaringBitmap],
    ) -> IdFilter {
        if matchers.is_empty() {
            return IdFilter::PassAll;
        }
        if let Some(first_pos_idx) = matchers.iter().position(|m| !m.negate) {
            let mut acc = pred_bitmaps[matchers[first_pos_idx].pred_id].clone();
            if acc.is_empty() {
                return IdFilter::Empty;
            }
            for (idx, m) in matchers.iter().enumerate() {
                if idx == first_pos_idx || m.negate {
                    continue;
                }
                acc &= &pred_bitmaps[m.pred_id];
                if acc.is_empty() {
                    return IdFilter::Empty;
                }
            }
            for m in matchers.iter().filter(|m| m.negate) {
                acc -= &pred_bitmaps[m.pred_id];
                if acc.is_empty() {
                    return IdFilter::Empty;
                }
            }
            IdFilter::KeepIds(acc)
        } else {
            // All matchers are negated: (!N1 & !N2 & ...) == !(N1 | N2 | ...).
            let mut excluded = RoaringBitmap::new();
            for m in matchers {
                excluded |= &pred_bitmaps[m.pred_id];
            }
            if excluded.is_empty() {
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

    fn eval_rule_matchers(
        matchers: &CompiledRuleMatchers,
        pred_bitmaps: &[RoaringBitmap],
        view: &LogsBatchView<'_>,
        all_rows: &RoaringBitmap,
    ) -> RoaringBitmap {
        let res_filter = Self::eval_id_conjunction(&matchers.resource_matchers, pred_bitmaps);
        if matches!(res_filter, IdFilter::Empty) {
            return RoaringBitmap::new();
        }
        let scope_filter = Self::eval_id_conjunction(&matchers.scope_matchers, pred_bitmaps);
        if matches!(scope_filter, IdFilter::Empty) {
            return RoaringBitmap::new();
        }

        let res_needed = !matches!(res_filter, IdFilter::PassAll);
        let scope_needed = !matches!(scope_filter, IdFilter::PassAll);
        if !res_needed && !scope_needed {
            return Self::eval_row_conjunction(&matchers.row_matchers, pred_bitmaps, all_rows);
        }

        let res_col = if res_needed {
            view.resource_id_column()
        } else {
            None
        };
        if matches!(res_filter, IdFilter::KeepIds(_)) && res_col.is_none() {
            return RoaringBitmap::new();
        }
        let scope_col = if scope_needed {
            view.scope_id_column()
        } else {
            None
        };
        if matches!(scope_filter, IdFilter::KeepIds(_)) && scope_col.is_none() {
            return RoaringBitmap::new();
        }

        let mut out = RoaringBitmap::new();
        if matchers.row_matchers.is_empty() {
            for row in 0..view.num_rows {
                if Self::matches_id_filter(&res_filter, res_col, row)
                    && Self::matches_id_filter(&scope_filter, scope_col, row)
                {
                    let _ = out.insert(row as u32);
                }
            }
        } else {
            let row_candidates =
                Self::eval_row_conjunction(&matchers.row_matchers, pred_bitmaps, all_rows);
            for row_u32 in row_candidates {
                let row = row_u32 as usize;
                if Self::matches_id_filter(&res_filter, res_col, row)
                    && Self::matches_id_filter(&scope_filter, scope_col, row)
                {
                    let _ = out.insert(row_u32);
                }
            }
        }
        out
    }

    fn compute_keep_bitmap(
        &self,
        pred_bitmaps: &[RoaringBitmap],
        view: &LogsBatchView<'_>,
    ) -> RoaringBitmap {
        let mut all_rows = RoaringBitmap::new();
        let _ = all_rows.insert_range(0..view.num_rows as u32);
        if self.filter_rules.is_empty() {
            return all_rows;
        }

        let mut keep_exempt = RoaringBitmap::new();
        let mut drop_matched = RoaringBitmap::new();

        for rule in &self.filter_rules {
            let matched = Self::eval_rule_matchers(&rule.matchers, pred_bitmaps, view, &all_rows);
            if matched.is_empty() {
                continue;
            }
            match rule.action {
                Action::Keep => keep_exempt |= matched,
                Action::Drop => drop_matched |= matched,
                Action::Unspecified => {}
            }
        }

        drop_matched -= &keep_exempt;
        all_rows -= &drop_matched;
        all_rows
    }

    fn compute_transform_masks(
        &self,
        pred_bitmaps: &[RoaringBitmap],
        view: &LogsBatchView<'_>,
    ) -> Vec<RoaringBitmap> {
        let mut all_rows = RoaringBitmap::new();
        let _ = all_rows.insert_range(0..view.num_rows as u32);

        self.transform_rules
            .iter()
            .map(|rule| Self::eval_rule_matchers(&rule.matchers, pred_bitmaps, view, &all_rows))
            .collect()
    }
    fn evaluate_filter_normalized(
        &self,
        batch: &OtapArrowRecords,
        scratch: &mut EvaluationScratch,
    ) -> Result<RoaringBitmap, PolicyError> {
        let Some(view) = LogsBatchView::try_new(batch)? else {
            return Ok(RoaringBitmap::new());
        };
        if self.filter_rules.is_empty() {
            let mut all_rows = RoaringBitmap::new();
            let _ = all_rows.insert_range(0..view.num_rows as u32);
            return Ok(all_rows);
        }
        let pred_bitmaps = self.evaluate_predicates(&self.filter_plan, &view, scratch);
        Ok(self.compute_keep_bitmap(&pred_bitmaps, &view))
    }

    /// Evaluates filter policies against a `Logs` batch using reusable `scratch` and returns the
    /// kept row indices (`0..num_rows`).
    ///
    /// Returns [`PolicyError::Evaluation`] if `batch` is not [`OtapArrowRecords::Logs`].
    pub fn evaluate_filter(
        &self,
        batch: &OtapArrowRecords,
        scratch: &mut EvaluationScratch,
    ) -> Result<RowSelection, PolicyError> {
        if !matches!(batch, OtapArrowRecords::Logs(_)) {
            return Err(PolicyError::Evaluation(
                "PolicyEngine currently only supports Logs batches".into(),
            ));
        }
        if self.filter_rules.is_empty() {
            let Some(root) = batch.get(ArrowPayloadType::Logs) else {
                return Ok(RowSelection {
                    bitmap: RoaringBitmap::new(),
                });
            };
            let mut all_rows = RoaringBitmap::new();
            let _ = all_rows.insert_range(0..root.num_rows() as u32);
            return Ok(RowSelection { bitmap: all_rows });
        }
        let mut normalized = batch.clone();
        normalized
            .decode_transport_optimized_ids()
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        let bitmap = self.evaluate_filter_normalized(&normalized, scratch)?;
        Ok(RowSelection { bitmap })
    }

    /// Evaluates filter and transform policies against a `Logs` batch using reusable `scratch` and
    /// returns the filtered/transformed [`EvaluationOutput`].
    ///
    /// Takes `batch` by shared reference so callers retain ownership of the original payload to
    /// nack on error. Returns [`PolicyError::Evaluation`] if `batch` is not
    /// [`OtapArrowRecords::Logs`].
    pub fn evaluate(
        &self,
        batch: &OtapArrowRecords,
        scratch: &mut EvaluationScratch,
    ) -> Result<EvaluationOutput, PolicyError> {
        if !matches!(batch, OtapArrowRecords::Logs(_)) {
            return Err(PolicyError::Evaluation(
                "PolicyEngine currently only supports Logs batches".into(),
            ));
        }
        let total_rows = batch
            .get(ArrowPayloadType::Logs)
            .map_or(0, RecordBatch::num_rows);
        if total_rows == 0 {
            return Ok(EvaluationOutput {
                records: Some(batch.clone()),
                kept_rows: 0,
                dropped_rows: 0,
                transform_hits: 0,
            });
        }

        // Normalize delta / quasi-delta transport-encoded IDs on a shallow Arc-cloned working copy
        // so the caller's original batch stays untouched if evaluation returns an error.
        let mut normalized = batch.clone();
        normalized
            .decode_transport_optimized_ids()
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        // `pdata::remove_transport_optimized_encodings` currently constructs `Schema::new(fields)`
        // on `Logs`, dropping incoming schema-level metadata. Restore non-empty root metadata until
        // pdata preserves it natively.
        if let Some(orig_meta) = batch
            .get(ArrowPayloadType::Logs)
            .map(|rb| rb.schema_ref().metadata())
            .filter(|m| !m.is_empty())
            && let Some(norm_root) = normalized.get(ArrowPayloadType::Logs)
            && norm_root.schema_ref().metadata().is_empty()
        {
            let restored = RecordBatch::try_new(
                Arc::new(Schema::new_with_metadata(
                    norm_root.schema_ref().fields().clone(),
                    orig_meta.clone(),
                )),
                norm_root.columns().to_vec(),
            )
            .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
            normalized
                .set(ArrowPayloadType::Logs, restored)
                .map_err(|e| PolicyError::Evaluation(e.to_string()))?;
        }

        // Fast path when no filter rules exist: evaluate transforms directly on `normalized`.
        if self.filter_rules.is_empty() {
            let mut out_batch = normalized;
            let transform_hits = if self.compiled_transforms.is_empty() {
                0
            } else {
                let tx_masks = if let Some(view) = LogsBatchView::try_new(&out_batch)? {
                    let pred_bitmaps =
                        self.evaluate_predicates(&self.transform_plan, &view, scratch);
                    self.compute_transform_masks(&pred_bitmaps, &view)
                } else {
                    Vec::new()
                };
                apply_spec_transforms_masked(&mut out_batch, &self.compiled_transforms, &tx_masks)?
            };
            return Ok(EvaluationOutput {
                records: Some(out_batch),
                kept_rows: total_rows as u64,
                dropped_rows: 0,
                transform_hits,
            });
        }

        let keep_bitmap = self.evaluate_filter_normalized(&normalized, scratch)?;
        let kept_rows = keep_bitmap.len();
        let dropped_rows = (total_rows as u64).saturating_sub(kept_rows);
        if kept_rows == 0 {
            return Ok(EvaluationOutput {
                records: None,
                kept_rows: 0,
                dropped_rows: total_rows as u64,
                transform_hits: 0,
            });
        }

        let mut filtered_batch = filter_otap_logs_by_bitmap(
            &normalized,
            &keep_bitmap,
            total_rows,
            &mut scratch.id_bitmap_pool,
        )?;

        let transform_hits = if self.compiled_transforms.is_empty() {
            0
        } else if let Some(view) = LogsBatchView::try_new(&filtered_batch)? {
            let pred_bitmaps = self.evaluate_predicates(&self.transform_plan, &view, scratch);
            let tx_masks = self.compute_transform_masks(&pred_bitmaps, &view);
            apply_spec_transforms_masked(&mut filtered_batch, &self.compiled_transforms, &tx_masks)?
        } else {
            0
        };

        Ok(EvaluationOutput {
            records: Some(filtered_batch),
            kept_rows,
            dropped_rows,
            transform_hits,
        })
    }
}
