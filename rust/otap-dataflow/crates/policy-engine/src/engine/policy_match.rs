// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Per-policy match bitmaps with lazy materialization.
//!
//! [`PolicyMatch`] represents which rows in an OTAP Arrow batch matched a
//! policy's matchers. It avoids allocating a full boolean buffer when all
//! rows or no rows matched -- a common case that lets us skip bitmap work
//! entirely.

use arrow::array::{Array, BooleanArray};
use arrow::buffer::BooleanBuffer;
use arrow::compute::filter as arrow_filter;
use otel_arrow_dfe_query_engine::pipeline::pipeline_expr::PredicateResult;

/// Indicates which rows in a batch matched a policy.
///
/// The three variants form a lattice for AND:
///   `None` < `Some(bitmap)` < `All`
///
/// This lets the common all-match and no-match cases avoid bitmap
/// allocation entirely.
#[derive(Clone, Debug)]
pub(crate) enum PolicyMatch {
    /// Every row matched (or no matcher has been applied yet).
    All,
    /// No rows matched.
    None,
    /// A subset of rows matched; the buffer has one bit per row.
    Some(BooleanBuffer),
}

impl PolicyMatch {
    /// AND this match with another: a row is in the result only if it
    /// appears in both inputs. Used to fold matcher results into the
    /// per-policy "matched" bitmap.
    #[must_use]
    pub(crate) fn and(&self, other: &PolicyMatch) -> PolicyMatch {
        match (self, other) {
            // All AND x = x
            (PolicyMatch::All, x) | (x, PolicyMatch::All) => x.clone(),
            // None AND x = None
            (PolicyMatch::None, _) | (_, PolicyMatch::None) => PolicyMatch::None,
            // Some AND Some = bitwise AND
            (PolicyMatch::Some(a), PolicyMatch::Some(b)) => {
                debug_assert_eq!(a.len(), b.len());
                let result = a & b;
                if result.count_set_bits() == 0 {
                    PolicyMatch::None
                } else if result.count_set_bits() == result.len() {
                    PolicyMatch::All
                } else {
                    PolicyMatch::Some(result)
                }
            }
        }
    }

    /// OR this match with another: a row is in the result if it appears
    /// in either input. Used to accumulate the "already discarded"
    /// bitmap during keep resolution.
    #[must_use]
    pub(crate) fn or(&self, other: &PolicyMatch) -> PolicyMatch {
        match (self, other) {
            // None OR x = x
            (PolicyMatch::None, x) | (x, PolicyMatch::None) => x.clone(),
            // All OR x = All
            (PolicyMatch::All, _) | (_, PolicyMatch::All) => PolicyMatch::All,
            // Some OR Some = bitwise OR
            (PolicyMatch::Some(a), PolicyMatch::Some(b)) => {
                debug_assert_eq!(a.len(), b.len());
                let result = a | b;
                if result.count_set_bits() == result.len() {
                    PolicyMatch::All
                } else {
                    PolicyMatch::Some(result)
                }
            }
        }
    }

    /// Compute `self AND NOT other`. Returns only the rows that are set
    /// in `self` but not set in `other`.
    #[must_use]
    pub(crate) fn and_not(&self, other: &PolicyMatch) -> PolicyMatch {
        match (self, other) {
            // x AND NOT None = x  (nothing to subtract)
            (x, PolicyMatch::None) => x.clone(),
            // x AND NOT All = None  (everything subtracted)
            (_, PolicyMatch::All) => PolicyMatch::None,
            // None AND NOT _ = None
            (PolicyMatch::None, _) => PolicyMatch::None,
            // All AND NOT Some(d) = Some(!d)
            (PolicyMatch::All, PolicyMatch::Some(d)) => {
                let result = !d;
                if result.count_set_bits() == 0 {
                    PolicyMatch::None
                } else if result.count_set_bits() == result.len() {
                    PolicyMatch::All
                } else {
                    PolicyMatch::Some(result)
                }
            }
            // Some(m) AND NOT Some(d) = Some(m & !d)
            (PolicyMatch::Some(m), PolicyMatch::Some(d)) => {
                debug_assert_eq!(m.len(), d.len());
                let not_d = !d;
                let result = m & &not_d;
                if result.count_set_bits() == 0 {
                    PolicyMatch::None
                } else {
                    PolicyMatch::Some(result)
                }
            }
        }
    }

    /// Returns the number of set bits (matched rows).
    ///
    /// For `All`, requires `num_rows` to know the total.
    /// For `None`, always 0.
    /// For `Some`, uses `BooleanBuffer::count_set_bits`.
    pub(crate) fn count_true(&self, num_rows: usize) -> usize {
        match self {
            PolicyMatch::All => num_rows,
            PolicyMatch::None => 0,
            PolicyMatch::Some(buf) => buf.count_set_bits(),
        }
    }

    /// Invert this bitmap: `true` becomes `false` and vice versa.
    ///
    /// `num_rows` is needed to materialize `All` -> all-set buffer for
    /// negation. Returns `All` if all bits become set, `None` if all
    /// become unset.
    #[must_use]
    pub(crate) fn not(this: &PolicyMatch) -> PolicyMatch {
        match this {
            PolicyMatch::All => PolicyMatch::None,
            PolicyMatch::None => PolicyMatch::All,
            PolicyMatch::Some(buf) => {
                let result = !buf;
                if result.count_set_bits() == 0 {
                    PolicyMatch::None
                } else if result.count_set_bits() == result.len() {
                    PolicyMatch::All
                } else {
                    PolicyMatch::Some(result)
                }
            }
        }
    }

    /// Filter this bitmap to keep only the rows indicated by `keep_mask`.
    #[must_use]
    pub(crate) fn filter_by(&self, keep_mask: &PolicyMatch) -> PolicyMatch {
        match (self, keep_mask) {
            // If the mask keeps everything, the bitmap is unchanged.
            (x, PolicyMatch::All) => x.clone(),
            // If the mask keeps nothing, no rows survive.
            (_, PolicyMatch::None) => PolicyMatch::None,
            // If this bitmap matches everything, the filtered result is All
            // (all surviving rows are matched).
            (PolicyMatch::All, _) => PolicyMatch::All,
            // If this bitmap matches nothing, still nothing.
            (PolicyMatch::None, _) => PolicyMatch::None,
            // Filter the bitmap: keep only bits where keep_mask is true.
            (PolicyMatch::Some(buf), PolicyMatch::Some(mask)) => {
                debug_assert_eq!(buf.len(), mask.len());
                let arr = BooleanArray::new(buf.clone(), None);
                let mask_arr = BooleanArray::new(mask.clone(), None);
                let filtered = arrow_filter(&arr, &mask_arr)
                    .expect("filter should not fail on equal-length boolean arrays");
                let filtered_bool = filtered
                    .as_any()
                    .downcast_ref::<BooleanArray>()
                    .expect("filter of BooleanArray produces BooleanArray");
                let new_buf = filtered_bool.values().clone();
                if new_buf.count_set_bits() == 0 {
                    PolicyMatch::None
                } else if new_buf.count_set_bits() == new_buf.len() {
                    PolicyMatch::All
                } else {
                    PolicyMatch::Some(new_buf)
                }
            }
        }
    }

    /// Convert to a [`BooleanArray`] for use with `filter_otap_batch`.
    ///
    /// `true` means the row should be **kept** (not discarded).
    pub(crate) fn to_boolean_array(&self, num_rows: usize) -> BooleanArray {
        match self {
            PolicyMatch::All => BooleanArray::new(BooleanBuffer::new_set(num_rows), None),
            PolicyMatch::None => BooleanArray::new(BooleanBuffer::new_unset(num_rows), None),
            PolicyMatch::Some(buf) => BooleanArray::new(buf.clone(), None),
        }
    }

    /// Create a [`PolicyMatch`] from a [`PredicateResult`].
    ///
    /// Preserves the scalar shortcuts from `PredicateResult` -- `All` and
    /// `None` map directly without bitmap allocation. For array results,
    /// delegates to [`from_boolean_array`](Self::from_boolean_array).
    pub(crate) fn from_predicate_result(result: PredicateResult) -> PolicyMatch {
        match result {
            PredicateResult::All => PolicyMatch::All,
            PredicateResult::None => PolicyMatch::None,
            PredicateResult::Array(arr) => Self::from_boolean_array(&arr),
        }
    }

    /// Create a [`PolicyMatch`] from a [`BooleanArray`].
    ///
    /// Null values in the array are treated as `false` (fail-open).
    /// If all values are true, returns [`PolicyMatch::All`]; if all
    /// are false/null, returns [`PolicyMatch::None`]; otherwise
    /// returns [`PolicyMatch::Some`] with the boolean buffer.
    fn from_boolean_array(arr: &BooleanArray) -> PolicyMatch {
        // Strip nulls: treat null as false by ANDing values with validity.
        let buffer = match arr.nulls() {
            Some(nulls) => arr.values() & nulls.inner(),
            None => arr.values().clone(),
        };

        let set_bits = buffer.count_set_bits();
        if set_bits == 0 {
            PolicyMatch::None
        } else if set_bits == buffer.len() {
            PolicyMatch::All
        } else {
            PolicyMatch::Some(buffer)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_buffer(bits: &[bool]) -> BooleanBuffer {
        BooleanBuffer::from_iter(bits.iter().copied())
    }

    /// Scenario: AND of All with any value returns that value.
    /// Guarantees: All is the identity element for AND.
    #[test]
    fn and_all_is_identity() {
        let some = PolicyMatch::Some(make_buffer(&[true, false, true]));

        // All AND Some = Some
        let result = PolicyMatch::All.and(&some);
        assert!(matches!(result, PolicyMatch::Some(_)));

        // All AND None = None
        let result = PolicyMatch::All.and(&PolicyMatch::None);
        assert!(matches!(result, PolicyMatch::None));

        // All AND All = All
        let result = PolicyMatch::All.and(&PolicyMatch::All);
        assert!(matches!(result, PolicyMatch::All));
    }

    /// Scenario: AND with None always returns None.
    /// Guarantees: None is the absorbing element for AND.
    #[test]
    fn and_none_absorbs() {
        let some = PolicyMatch::Some(make_buffer(&[true, false, true]));

        let result = some.and(&PolicyMatch::None);
        assert!(matches!(result, PolicyMatch::None));

        let result = PolicyMatch::None.and(&PolicyMatch::All);
        assert!(matches!(result, PolicyMatch::None));
    }

    /// Scenario: AND of two bitmaps produces the bitwise intersection.
    /// Guarantees: Only rows set in both inputs remain set.
    #[test]
    fn and_some_some() {
        let a = PolicyMatch::Some(make_buffer(&[true, true, false, false]));
        let b = PolicyMatch::Some(make_buffer(&[true, false, true, false]));

        let result = a.and(&b);
        match result {
            PolicyMatch::Some(buf) => {
                assert_eq!(buf.len(), 4);
                assert!(buf.value(0));
                assert!(!buf.value(1));
                assert!(!buf.value(2));
                assert!(!buf.value(3));
            }
            _ => panic!("expected Some"),
        }
    }

    /// Scenario: AND of two all-set bitmaps collapses to All.
    /// Guarantees: Full intersection is recognized as All.
    #[test]
    fn and_some_some_collapses_to_all() {
        let a = PolicyMatch::Some(make_buffer(&[true, true, true]));
        let b = PolicyMatch::Some(make_buffer(&[true, true, true]));

        let result = a.and(&b);
        assert!(matches!(result, PolicyMatch::All));
    }

    /// Scenario: AND of disjoint bitmaps collapses to None.
    /// Guarantees: Empty intersection is recognized as None.
    #[test]
    fn and_some_some_collapses_to_none() {
        let a = PolicyMatch::Some(make_buffer(&[true, false]));
        let b = PolicyMatch::Some(make_buffer(&[false, true]));

        let result = a.and(&b);
        assert!(matches!(result, PolicyMatch::None));
    }

    /// Scenario: from_predicate_result maps All directly.
    /// Guarantees: Scalar true avoids bitmap allocation.
    #[test]
    fn from_predicate_result_all() {
        let result = PolicyMatch::from_predicate_result(PredicateResult::All);
        assert!(matches!(result, PolicyMatch::All));
    }

    /// Scenario: from_predicate_result maps None directly.
    /// Guarantees: Scalar false avoids bitmap allocation.
    #[test]
    fn from_predicate_result_none() {
        let result = PolicyMatch::from_predicate_result(PredicateResult::None);
        assert!(matches!(result, PolicyMatch::None));
    }

    /// Scenario: from_predicate_result with a mixed array produces Some.
    /// Guarantees: Array results are converted to bitmaps.
    #[test]
    fn from_predicate_result_array() {
        let arr = BooleanArray::from(vec![true, false, true]);
        let result = PolicyMatch::from_predicate_result(PredicateResult::Array(arr));
        match result {
            PolicyMatch::Some(buf) => {
                assert_eq!(buf.len(), 3);
                assert!(buf.value(0));
                assert!(!buf.value(1));
                assert!(buf.value(2));
            }
            _ => panic!("expected Some"),
        }
    }

    /// Scenario: from_boolean_array with all-true values returns All.
    /// Guarantees: Full-match arrays are recognized as All.
    #[test]
    fn from_boolean_array_all_true() {
        let arr = BooleanArray::from(vec![true, true, true]);
        let result = PolicyMatch::from_boolean_array(&arr);
        assert!(matches!(result, PolicyMatch::All));
    }

    /// Scenario: from_boolean_array with all-false values returns None.
    /// Guarantees: No-match arrays are recognized as None.
    #[test]
    fn from_boolean_array_all_false() {
        let arr = BooleanArray::from(vec![false, false, false]);
        let result = PolicyMatch::from_boolean_array(&arr);
        assert!(matches!(result, PolicyMatch::None));
    }

    /// Scenario: from_boolean_array with mixed values returns Some.
    /// Guarantees: Partial matches produce a bitmap.
    #[test]
    fn from_boolean_array_mixed() {
        let arr = BooleanArray::from(vec![true, false, true]);
        let result = PolicyMatch::from_boolean_array(&arr);
        match result {
            PolicyMatch::Some(buf) => {
                assert_eq!(buf.len(), 3);
                assert!(buf.value(0));
                assert!(!buf.value(1));
                assert!(buf.value(2));
            }
            _ => panic!("expected Some"),
        }
    }

    /// Scenario: from_boolean_array treats nulls as false.
    /// Guarantees: Null predicate results do not count as matches.
    #[test]
    fn from_boolean_array_nulls_are_false() {
        let arr = BooleanArray::from(vec![Some(true), None, Some(false)]);
        let result = PolicyMatch::from_boolean_array(&arr);
        match result {
            PolicyMatch::Some(buf) => {
                assert_eq!(buf.len(), 3);
                assert!(buf.value(0));
                assert!(!buf.value(1)); // null -> false
                assert!(!buf.value(2));
            }
            _ => panic!("expected Some"),
        }
    }

    /// Scenario: from_boolean_array with all-null values returns None.
    /// Guarantees: All-null arrays are treated as no matches.
    #[test]
    fn from_boolean_array_all_null() {
        let arr = BooleanArray::from(vec![Option::<bool>::None, None, None]);
        let result = PolicyMatch::from_boolean_array(&arr);
        assert!(matches!(result, PolicyMatch::None));
    }

    // -- OR tests --

    /// Scenario: OR of None with any value returns that value.
    /// Guarantees: None is the identity element for OR.
    #[test]
    fn or_none_is_identity() {
        let some = PolicyMatch::Some(make_buffer(&[true, false, true]));

        let result = PolicyMatch::None.or(&some);
        assert!(matches!(result, PolicyMatch::Some(_)));

        let result = PolicyMatch::None.or(&PolicyMatch::All);
        assert!(matches!(result, PolicyMatch::All));

        let result = PolicyMatch::None.or(&PolicyMatch::None);
        assert!(matches!(result, PolicyMatch::None));
    }

    /// Scenario: OR with All always returns All.
    /// Guarantees: All is the absorbing element for OR.
    #[test]
    fn or_all_absorbs() {
        let some = PolicyMatch::Some(make_buffer(&[true, false, true]));
        let result = some.or(&PolicyMatch::All);
        assert!(matches!(result, PolicyMatch::All));
    }

    /// Scenario: OR of two bitmaps produces the bitwise union.
    /// Guarantees: Rows set in either input are set in the result.
    #[test]
    fn or_some_some() {
        let a = PolicyMatch::Some(make_buffer(&[true, false, false]));
        let b = PolicyMatch::Some(make_buffer(&[false, false, true]));

        let result = a.or(&b);
        match result {
            PolicyMatch::Some(buf) => {
                assert_eq!(buf.len(), 3);
                assert!(buf.value(0));
                assert!(!buf.value(1));
                assert!(buf.value(2));
            }
            _ => panic!("expected Some"),
        }
    }

    // -- AND NOT tests --

    /// Scenario: and_not with None subtracted returns the original.
    /// Guarantees: Subtracting nothing preserves all rows.
    #[test]
    fn and_not_none_preserves() {
        let some = PolicyMatch::Some(make_buffer(&[true, false, true]));
        let result = some.and_not(&PolicyMatch::None);
        assert!(matches!(result, PolicyMatch::Some(_)));

        let result = PolicyMatch::All.and_not(&PolicyMatch::None);
        assert!(matches!(result, PolicyMatch::All));
    }

    /// Scenario: and_not with All subtracted returns None.
    /// Guarantees: Subtracting everything leaves nothing.
    #[test]
    fn and_not_all_clears() {
        let some = PolicyMatch::Some(make_buffer(&[true, false, true]));
        let result = some.and_not(&PolicyMatch::All);
        assert!(matches!(result, PolicyMatch::None));
    }

    /// Scenario: and_not with partial subtraction removes the right rows.
    /// Guarantees: Only non-subtracted matched rows survive.
    #[test]
    fn and_not_some_some() {
        let m = PolicyMatch::Some(make_buffer(&[true, true, false, true]));
        let d = PolicyMatch::Some(make_buffer(&[false, true, false, false]));

        let result = m.and_not(&d);
        match result {
            PolicyMatch::Some(buf) => {
                assert_eq!(buf.len(), 4);
                assert!(buf.value(0));
                assert!(!buf.value(1));
                assert!(!buf.value(2));
                assert!(buf.value(3));
            }
            _ => panic!("expected Some"),
        }
    }

    // -- count_true tests --

    /// Scenario: count_true returns correct counts for each variant.
    /// Guarantees: Counts are consistent with the bitmap contents.
    #[test]
    fn count_true_variants() {
        assert_eq!(PolicyMatch::All.count_true(100), 100);
        assert_eq!(PolicyMatch::None.count_true(100), 0);

        let some = PolicyMatch::Some(make_buffer(&[true, false, true, false, true]));
        assert_eq!(some.count_true(5), 3);
    }
}
