// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-field HIR / glob n-gram inverted index (`AhoCorasick`) and generation-stamped
//! scan scratch buffers.

use ahash::AHashMap;
use aho_corasick::{AhoCorasick, MatchKind};
use regex_automata::meta::Regex as MetaRegex;
use regex_syntax::hir::{Hir, HirKind};

use crate::model::{PolicyError, matches_glob};

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

/// Extracts all mandatory literal segments (`len >= 2` bytes) from a glob pattern by splitting
/// on `*` and `?` wildcards.
#[must_use]
pub(crate) fn extract_glob_ngrams(pat: &str) -> Vec<Vec<u8>> {
    let mut unique: Vec<Vec<u8>> = Vec::new();
    for chunk in pat.split(['*', '?']) {
        let bytes = chunk.as_bytes();
        if bytes.len() >= 2 && !unique.iter().any(|u| u.as_slice() == bytes) {
            unique.push(bytes.to_vec());
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
    /// Full-string wildcard match (`Predicate::Glob`).
    Glob(String),
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
            Self::Glob(pat) => matches_glob(pat, haystack),
            Self::Regex(re) => re.is_match(haystack),
        }
    }

    #[inline]
    fn verify_fallback(&self, haystack: &str) -> bool {
        match self {
            Self::Contains(sub) => haystack.contains(sub.as_str()),
            Self::Glob(pat) => matches_glob(pat, haystack),
            Self::Regex(re) => re.is_match(haystack),
        }
    }
}

/// A regex, glob, or `Contains` string pattern with `>= 1` required n-grams indexed in `AhoCorasick`.
#[derive(Debug, Clone)]
struct IndexedPattern {
    pred_id: usize,
    required_ngrams: u16,
    verifier: StringVerifier,
}

/// A regex, glob, or `Contains` string pattern with 0 extractable n-grams of length `>= 2`.
#[derive(Debug, Clone)]
struct FallbackPattern {
    pred_id: usize,
    verifier: StringVerifier,
}

/// Per-field n-gram inverted index over `AhoCorasick` (`MatchKind::Standard`) + pattern verifiers.
#[derive(Debug, Clone, Default)]
pub(crate) struct FieldStringIndex {
    ac: Option<AhoCorasick>,
    /// Maps `ngram_id` (`0..num_unique_ngrams`) -> list of `indexed_patterns` indices (`u32`).
    ngram_postings: Vec<Vec<u32>>,
    indexed_patterns: Vec<IndexedPattern>,
    fallback_patterns: Vec<FallbackPattern>,
}

/// Reusable generation-stamped scratch state for scanning strings against a `FieldStringIndex`
/// without per-row vector clearing or allocations.
#[derive(Default)]
pub(crate) struct StringScanScratch {
    cur_gen: u32,
    ngram_seen_gen: Vec<u32>,
    pat_seen_gen: Vec<u32>,
    pat_matched_count: Vec<u16>,
}

impl StringScanScratch {
    /// Ensures counter buffers match `(num_ngrams, num_patterns)`, reusing existing capacity on
    /// steady-state batches.
    ///
    /// Never rewinds `cur_gen`: maintaining `forall i: seen_gen[i] <= cur_gen` guarantees that
    /// partial dimension changes across stages or policy reloads (where only one of `num_ngrams`
    /// or `num_patterns` changes) never collide with stale generation stamps in the unchanged vec.
    pub(crate) fn prepare(&mut self, num_ngrams: usize, num_patterns: usize) {
        if self.ngram_seen_gen.len() != num_ngrams {
            self.ngram_seen_gen.clear();
            self.ngram_seen_gen.resize(num_ngrams, 0);
        }
        if self.pat_seen_gen.len() != num_patterns {
            self.pat_seen_gen.clear();
            self.pat_seen_gen.resize(num_patterns, 0);
            self.pat_matched_count.clear();
            self.pat_matched_count.resize(num_patterns, 0);
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

    #[cfg(test)]
    pub(crate) fn set_cur_gen_for_test(&mut self, gen_val: u32) {
        self.cur_gen = gen_val;
    }
}

impl FieldStringIndex {
    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.ac.is_none() && self.fallback_patterns.is_empty()
    }

    #[inline]
    pub(crate) fn prepare_scratch(&self, scratch: &mut StringScanScratch) {
        scratch.prepare(self.ngram_postings.len(), self.indexed_patterns.len());
    }

    #[inline]
    pub(crate) fn evaluate_str<F>(
        &self,
        haystack: &str,
        scratch: &mut StringScanScratch,
        mut on_match: F,
    ) where
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
pub(crate) struct FieldStringIndexBuilder {
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

    fn add_indexed_or_fallback(
        &mut self,
        pred_id: usize,
        ngrams: Vec<Vec<u8>>,
        verifier: StringVerifier,
    ) {
        if ngrams.is_empty() || ngrams.len() > u16::MAX as usize {
            self.fallback_patterns
                .push(FallbackPattern { pred_id, verifier });
            return;
        }
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

    pub(crate) fn add_contains(&mut self, pred_id: usize, sub: &str) {
        let verifier = StringVerifier::Contains(sub.to_owned());
        if sub.len() >= 2 {
            self.add_indexed_or_fallback(pred_id, vec![sub.as_bytes().to_vec()], verifier);
        } else {
            self.fallback_patterns
                .push(FallbackPattern { pred_id, verifier });
        }
    }

    pub(crate) fn add_glob(&mut self, pred_id: usize, pat: &str) {
        let ngrams = extract_glob_ngrams(pat);
        self.add_indexed_or_fallback(pred_id, ngrams, StringVerifier::Glob(pat.to_owned()));
    }

    pub(crate) fn add_compiled_regex(&mut self, pred_id: usize, hir: &Hir, re: MetaRegex) {
        let ngrams = extract_required_ngrams(hir);
        self.add_indexed_or_fallback(pred_id, ngrams, StringVerifier::Regex(re));
    }

    pub(crate) fn build(self) -> Result<FieldStringIndex, PolicyError> {
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
