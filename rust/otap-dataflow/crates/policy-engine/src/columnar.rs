// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Arrow SIMD compute kernel (`arrow::compute::kernels::cmp`) and validity-bitmap helpers
//! for columnar OTAP policy evaluation.

use ahash::AHashMap;
use arrow::array::{
    Array, ArrayRef, BinaryArray, BooleanArray, DictionaryArray, FixedSizeBinaryArray,
    Float64Array, Int32Array, Int64Array, StringArray, UInt8Array,
};
use arrow::buffer::NullBuffer;
use arrow::compute::kernels::cmp;
use arrow::datatypes::{DataType, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::arrays::{ByteArrayAccessor, MaybeDictArrayAccessor};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::otlp::common::AnyValueArrays;

use crate::bitset::{BitsetBufferPool, DenseBitset};
use crate::model::{NumericValue, Predicate, ScalarValue, eq_i64_f64};
use crate::otap_ops::extract_cbor_path_for_eval;

/// Maximum number of exact string-equality predicates on a plain `StringArray` where
/// per-predicate `arrow::compute::kernels::cmp::eq` beats `AHashMap` lookup.
pub(crate) const SMALL_PLAIN_STR_EQ_THRESHOLD: usize = 2;

/// Maximum number of exact string-equality predicates on a dictionary-encoded string array where
/// `arrow::compute::kernels::cmp::eq` beats `DictStrEvalCache`.
pub(crate) const SMALL_DICT_STR_EQ_THRESHOLD: usize = 1;

/// Maximum number of exact `TraceId`/`SpanId` equality predicates evaluated via `cmp::eq`
/// before falling back to `AHashMap` lookup over non-zero rows.
pub(crate) const SMALL_ID_EQ_THRESHOLD: usize = 4;

/// Minimum attribute table row count before enabling full-column SIMD value kernels under
/// low key selectivity.
pub(crate) const MIN_COLUMNAR_ATTR_ROWS: usize = 64;

/// Denominator for DataFusion's 20% selectivity threshold (`key_matches * 5 > num_attr_rows`):
/// when more than 20% of attribute rows match the watched key, full-column SIMD kernels + bitwise
/// AND beat sparse row-by-row value scanning.
pub(crate) const ATTR_SELECTIVITY_DENOMINATOR: usize = 5;

/// Exact integer comparison equivalent to evaluating a [`Predicate`] against an `i64` column,
/// preserving exact 64-bit precision above `2^53` and IEEE-754 `NaN` / `-0.0` semantics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Int64CmpOp {
    AllFalse,
    AllTrue,
    Eq(i64),
    Gt(i64),
    GtEq(i64),
    Lt(i64),
    LtEq(i64),
}

impl Int64CmpOp {
    /// Reduces a scalar equality or numeric range [`Predicate`] to an exact `i64` comparison.
    #[must_use]
    pub(crate) fn from_predicate(pred: &Predicate) -> Self {
        // 2^63 is exact in f64 and strictly greater than i64::MAX (2^63 - 1).
        const TWO_POW_63: f64 = 9_223_372_036_854_775_808.0_f64;
        // -2^63 is exact in f64 and equals i64::MIN.
        const NEG_TWO_POW_63: f64 = -9_223_372_036_854_775_808.0_f64;

        match *pred {
            Predicate::Equals(ScalarValue::Int(t)) => Self::Eq(t),
            Predicate::Equals(ScalarValue::Double(d)) => {
                if d.is_nan() || !d.is_finite() || !(NEG_TWO_POW_63..TWO_POW_63).contains(&d) {
                    return Self::AllFalse;
                }
                let trunc = d.trunc();
                if d == trunc {
                    Self::Eq(trunc as i64)
                } else {
                    Self::AllFalse
                }
            }
            Predicate::Gt(NumericValue::Int(t)) => Self::Gt(t),
            Predicate::Gte(NumericValue::Int(t)) => Self::GtEq(t),
            Predicate::Lt(NumericValue::Int(t)) => Self::Lt(t),
            Predicate::Lte(NumericValue::Int(t)) => Self::LtEq(t),
            Predicate::Gt(NumericValue::Double(d)) => {
                if d.is_nan() || d >= TWO_POW_63 {
                    Self::AllFalse
                } else if d < NEG_TWO_POW_63 {
                    Self::AllTrue
                } else {
                    Self::Gt(d.floor() as i64)
                }
            }
            Predicate::Gte(NumericValue::Double(d)) => {
                if d.is_nan() || d >= TWO_POW_63 {
                    Self::AllFalse
                } else if d < NEG_TWO_POW_63 {
                    Self::AllTrue
                } else {
                    Self::GtEq(d.ceil() as i64)
                }
            }
            Predicate::Lt(NumericValue::Double(d)) => {
                if d.is_nan() || d < NEG_TWO_POW_63 {
                    Self::AllFalse
                } else if d >= TWO_POW_63 {
                    Self::AllTrue
                } else {
                    Self::Lt(d.ceil() as i64)
                }
            }
            Predicate::Lte(NumericValue::Double(d)) => {
                if d.is_nan() || d < NEG_TWO_POW_63 {
                    Self::AllFalse
                } else if d >= TWO_POW_63 {
                    Self::AllTrue
                } else {
                    Self::LtEq(d.floor() as i64)
                }
            }
            _ => Self::AllFalse,
        }
    }

    /// Returns `true` if `0i64` satisfies this comparison.
    #[inline]
    #[must_use]
    pub(crate) const fn matches_zero(self) -> bool {
        match self {
            Self::AllFalse => false,
            Self::AllTrue => true,
            Self::Eq(t) => 0 == t,
            Self::Gt(t) => 0 > t,
            Self::GtEq(t) => 0 >= t,
            Self::Lt(t) => 0 < t,
            Self::LtEq(t) => 0 <= t,
        }
    }

    /// Evaluates this integer comparison on a single `i64` value.
    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) const fn eval_scalar(self, v: i64) -> bool {
        match self {
            Self::AllFalse => false,
            Self::AllTrue => true,
            Self::Eq(t) => v == t,
            Self::Gt(t) => v > t,
            Self::GtEq(t) => v >= t,
            Self::Lt(t) => v < t,
            Self::LtEq(t) => v <= t,
        }
    }

    /// Narrows this `i64` comparison into an exact `i32` comparison.
    #[must_use]
    pub(crate) const fn narrow_to_i32(self) -> Int32CmpOp {
        const MIN: i64 = i32::MIN as i64;
        const MAX: i64 = i32::MAX as i64;

        match self {
            Self::AllFalse => Int32CmpOp::AllFalse,
            Self::AllTrue => Int32CmpOp::AllTrue,
            Self::Eq(t) => {
                if t < MIN || t > MAX {
                    Int32CmpOp::AllFalse
                } else {
                    Int32CmpOp::Eq(t as i32)
                }
            }
            Self::Gt(t) => {
                if t < MIN {
                    Int32CmpOp::AllTrue
                } else if t >= MAX {
                    Int32CmpOp::AllFalse
                } else {
                    Int32CmpOp::Gt(t as i32)
                }
            }
            Self::GtEq(t) => {
                if t <= MIN {
                    Int32CmpOp::AllTrue
                } else if t > MAX {
                    Int32CmpOp::AllFalse
                } else {
                    Int32CmpOp::GtEq(t as i32)
                }
            }
            Self::Lt(t) => {
                if t <= MIN {
                    Int32CmpOp::AllFalse
                } else if t > MAX {
                    Int32CmpOp::AllTrue
                } else {
                    Int32CmpOp::Lt(t as i32)
                }
            }
            Self::LtEq(t) => {
                if t < MIN {
                    Int32CmpOp::AllFalse
                } else if t >= MAX {
                    Int32CmpOp::AllTrue
                } else {
                    Int32CmpOp::LtEq(t as i32)
                }
            }
        }
    }
}

/// Exact integer comparison equivalent to evaluating a [`Predicate`] against an `i32` column.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Int32CmpOp {
    AllFalse,
    AllTrue,
    Eq(i32),
    Gt(i32),
    GtEq(i32),
    Lt(i32),
    LtEq(i32),
}

impl Int32CmpOp {
    /// Returns `true` if `0i32` satisfies this comparison.
    #[inline]
    #[must_use]
    pub(crate) const fn matches_zero(self) -> bool {
        match self {
            Self::AllFalse => false,
            Self::AllTrue => true,
            Self::Eq(t) => 0 == t,
            Self::Gt(t) => 0 > t,
            Self::GtEq(t) => 0 >= t,
            Self::Lt(t) => 0 < t,
            Self::LtEq(t) => 0 <= t,
        }
    }

    #[cfg(test)]
    #[inline]
    #[must_use]
    pub(crate) const fn eval_scalar(self, v: i32) -> bool {
        match self {
            Self::AllFalse => false,
            Self::AllTrue => true,
            Self::Eq(t) => v == t,
            Self::Gt(t) => v > t,
            Self::GtEq(t) => v >= t,
            Self::Lt(t) => v < t,
            Self::LtEq(t) => v <= t,
        }
    }
}

/// Executes an `Int64CmpOp` against an `Int64` (or `Dict<U8|U16, Int64>`) Arrow array via SIMD
/// `arrow::compute::kernels::cmp`.
fn run_i64_cmp_kernel(col: &dyn Array, op: Int64CmpOp) -> Option<BooleanArray> {
    match op {
        Int64CmpOp::Eq(t) => cmp::eq(&col, &Int64Array::new_scalar(t)).ok(),
        Int64CmpOp::Gt(t) => cmp::gt(&col, &Int64Array::new_scalar(t)).ok(),
        Int64CmpOp::GtEq(t) => cmp::gt_eq(&col, &Int64Array::new_scalar(t)).ok(),
        Int64CmpOp::Lt(t) => cmp::lt(&col, &Int64Array::new_scalar(t)).ok(),
        Int64CmpOp::LtEq(t) => cmp::lt_eq(&col, &Int64Array::new_scalar(t)).ok(),
        Int64CmpOp::AllFalse | Int64CmpOp::AllTrue => None,
    }
}

/// Executes an `Int32CmpOp` against an `Int32` (or `Dict<U8|U16, Int32>`) Arrow array via SIMD
/// `arrow::compute::kernels::cmp`.
fn run_i32_cmp_kernel(col: &dyn Array, op: Int32CmpOp) -> Option<BooleanArray> {
    match op {
        Int32CmpOp::Eq(t) => cmp::eq(&col, &Int32Array::new_scalar(t)).ok(),
        Int32CmpOp::Gt(t) => cmp::gt(&col, &Int32Array::new_scalar(t)).ok(),
        Int32CmpOp::GtEq(t) => cmp::gt_eq(&col, &Int32Array::new_scalar(t)).ok(),
        Int32CmpOp::Lt(t) => cmp::lt(&col, &Int32Array::new_scalar(t)).ok(),
        Int32CmpOp::LtEq(t) => cmp::lt_eq(&col, &Int32Array::new_scalar(t)).ok(),
        Int32CmpOp::AllFalse | Int32CmpOp::AllTrue => None,
    }
}

/// Borrows the underlying Arrow [`Array`] from a `pdata` [`MaybeDictArrayAccessor`].
#[inline]
#[must_use]
pub(crate) fn maybe_dict_as_array<'a, V: Array + 'static>(
    acc: &'a MaybeDictArrayAccessor<'a, V>,
) -> &'a dyn Array {
    match acc {
        MaybeDictArrayAccessor::Native(a) => *a,
        MaybeDictArrayAccessor::Dictionary8(d) => d.inner(),
        MaybeDictArrayAccessor::Dictionary16(d) => d.inner(),
    }
}

/// Borrows the underlying Arrow [`Array`] from a `pdata` [`ByteArrayAccessor`].
#[inline]
#[must_use]
pub(crate) fn byte_accessor_as_array<'a>(acc: &'a ByteArrayAccessor<'a>) -> &'a dyn Array {
    match acc {
        ByteArrayAccessor::Binary(m) => maybe_dict_as_array(m),
        ByteArrayAccessor::FixedSizeBinary(m) => maybe_dict_as_array(m),
    }
}

/// Evaluates an IEEE-754 numeric equality or range predicate against a single `f64` value
/// (`NaN` always returns `false`, `-0.0 == +0.0` returns `true`, `> 2^53` `i64` precision exact).
#[inline]
#[must_use]
pub(crate) fn eval_f64_pred(pred: &Predicate, v: f64) -> bool {
    if v.is_nan() {
        return false;
    }
    match pred {
        Predicate::Equals(ScalarValue::Double(d)) => !d.is_nan() && v == *d,
        Predicate::Equals(ScalarValue::Int(i)) => eq_i64_f64(*i, v),
        Predicate::Gt(nv) | Predicate::Gte(nv) | Predicate::Lt(nv) | Predicate::Lte(nv) => {
            pred.matches_ordering(nv.cmp_candidate_f64(v))
        }
        _ => false,
    }
}

/// Returns `true` if `pred` can match an integer or floating-point value.
#[inline]
#[must_use]
pub(crate) fn is_numeric_pred(pred: &Predicate) -> bool {
    matches!(
        pred,
        Predicate::Equals(ScalarValue::Int(_) | ScalarValue::Double(_))
            | Predicate::Gt(_)
            | Predicate::Gte(_)
            | Predicate::Lt(_)
            | Predicate::Lte(_)
    )
}

/// Evaluates an `Int64` `AnyValue` predicate over rows selected by `type_mask`, respecting OTAP
/// `default_values_optional` (omitted or null slots encode `0`), and ORs results into `out`.
pub(crate) fn eval_anyval_int_pred_into(
    int_col: Option<&dyn Array>,
    pred: &Predicate,
    type_mask: &DenseBitset,
    pool: &mut BitsetBufferPool,
    out: &mut DenseBitset,
) {
    if type_mask.is_empty() {
        return;
    }
    let op = Int64CmpOp::from_predicate(pred);
    match op {
        Int64CmpOp::AllFalse => {}
        Int64CmpOp::AllTrue => out.or_assign_with_pool(type_mask, pool),
        _ => {
            let default_matches = op.matches_zero();
            let Some(col) = int_col else {
                if default_matches {
                    out.or_assign_with_pool(type_mask, pool);
                }
                return;
            };
            if let Some(cmp_arr) = run_i64_cmp_kernel(col, op) {
                out.or_boolean_array_with_pool(&cmp_arr, default_matches, Some(type_mask), pool);
            }
        }
    }
}

/// Evaluates a `Float64Array` `AnyValue` predicate in 64-row chunks (skipping zero `type_mask`
/// words in O(1)), enforcing exact IEEE-754 `NaN`/`-0.0` and `> 2^53` `i64` semantics (unlike
/// `arrow::compute::kernels::cmp`'s `f64::total_cmp`), and respecting `default_values_optional`
/// (`0.0`).
pub(crate) fn eval_anyval_double_pred_into(
    double_col: Option<&Float64Array>,
    pred: &Predicate,
    type_mask: &DenseBitset,
    pool: &mut BitsetBufferPool,
    out: &mut DenseBitset,
) {
    if type_mask.is_empty() || !is_numeric_pred(pred) {
        return;
    }
    let default_matches = eval_f64_pred(pred, 0.0);
    let Some(da) = double_col else {
        if default_matches {
            out.or_assign_with_pool(type_mask, pool);
        }
        return;
    };
    let len = da.len();
    if len == 0 {
        return;
    }
    let vals = da.values();
    let mask_words = type_mask.words();
    let num_words = len.div_ceil(64);
    let max_words = num_words.min(mask_words.len());
    let null_chunks = da.nulls().map(|n| n.inner().bit_chunks());
    let mut null_iter = null_chunks.as_ref().map(|c| c.iter_padded());
    let rem = len & 63;
    let last_word_idx = num_words - 1;
    let last_mask = if rem == 0 {
        u64::MAX
    } else {
        (1u64 << rem) - 1
    };

    for (wi, &m) in mask_words.iter().enumerate().take(max_words) {
        let n = null_iter
            .as_mut()
            .map_or(u64::MAX, |it| it.next().unwrap_or(0));
        if m == 0 {
            continue;
        }
        let base = wi * 64;
        let end = (base + 64).min(len);
        let mut v_word = 0u64;
        for (bi, &v) in vals[base..end].iter().enumerate() {
            v_word |= u64::from(eval_f64_pred(pred, v)) << bi;
        }
        let tail = if wi == last_word_idx {
            last_mask
        } else {
            u64::MAX
        };
        let w = if default_matches {
            (v_word | !n) & tail & m
        } else {
            v_word & n & tail & m
        };
        if w != 0 {
            out.or_word_with_pool(wi, w, num_words, pool);
        }
    }
}

/// Evaluates a `Bool` equality predicate directly from the Arrow [`BooleanArray`] bitmap.
/// Null boolean slots do not match (no default value per `extract_anyval`).
pub(crate) fn eval_anyval_bool_pred_into(
    bool_col: Option<&BooleanArray>,
    target: bool,
    type_mask: &DenseBitset,
    pool: &mut BitsetBufferPool,
    out: &mut DenseBitset,
) {
    if type_mask.is_empty() {
        return;
    }
    let Some(ba) = bool_col else {
        return;
    };
    if target {
        out.or_boolean_array_with_pool(ba, false, Some(type_mask), pool);
    } else {
        out.or_negated_boolean_array_with_pool(ba, Some(type_mask), pool);
    }
}

/// Evaluates a `Bytes` equality predicate via `arrow::compute::kernels::cmp::eq`, respecting
/// `default_values_optional` (`&[]`) and both `Binary` and `FixedSizeBinary` (native or dict).
pub(crate) fn eval_anyval_bytes_eq_into(
    bytes_col: Option<&dyn Array>,
    target: &[u8],
    type_mask: &DenseBitset,
    pool: &mut BitsetBufferPool,
    out: &mut DenseBitset,
) {
    if type_mask.is_empty() {
        return;
    }
    let default_matches = target.is_empty();
    let Some(col) = bytes_col else {
        if default_matches {
            out.or_assign_with_pool(type_mask, pool);
        }
        return;
    };
    let leaf_dt = match col.data_type() {
        DataType::Dictionary(_, v) => v.as_ref(),
        other => other,
    };
    match leaf_dt {
        DataType::Binary => {
            if let Ok(cmp_arr) = cmp::eq(&col, &BinaryArray::new_scalar(target)) {
                out.or_boolean_array_with_pool(&cmp_arr, default_matches, Some(type_mask), pool);
            }
        }
        DataType::FixedSizeBinary(dims) => {
            let dims_usize = usize::try_from(*dims).ok();
            if dims_usize == Some(target.len()) {
                if let Ok(cmp_arr) = cmp::eq(&col, &FixedSizeBinaryArray::new_scalar(target)) {
                    out.or_boolean_array_with_pool(
                        &cmp_arr,
                        default_matches,
                        Some(type_mask),
                        pool,
                    );
                }
            } else if default_matches && let Some(logical_nulls) = col.logical_nulls() {
                // Only null slots encode the default `&[]` when dims != 0.
                let valid = DenseBitset::from_validity_buffers(
                    col.len(),
                    Some(&logical_nulls),
                    None,
                    Some(type_mask),
                    pool,
                );
                let mut nulls_in_mask = DenseBitset::new();
                nulls_in_mask.or_assign_with_pool(type_mask, pool);
                nulls_in_mask.sub_assign(&valid);
                pool.recycle(valid);
                out.or_assign_with_pool(&nulls_in_mask, pool);
                pool.recycle(nulls_in_mask);
            }
        }
        _ => {}
    }
}

/// Evaluates an exact `Str` equality predicate on an `AnyValue` `str` column via
/// `arrow::compute::kernels::cmp::eq`, respecting `default_values_optional` (`""`).
pub(crate) fn eval_anyval_str_eq_into(
    str_col: Option<&dyn Array>,
    target: &str,
    type_mask: &DenseBitset,
    pool: &mut BitsetBufferPool,
    out: &mut DenseBitset,
) {
    if type_mask.is_empty() {
        return;
    }
    let default_matches = target.is_empty();
    let Some(col) = str_col else {
        if default_matches {
            out.or_assign_with_pool(type_mask, pool);
        }
        return;
    };
    if let Ok(cmp_arr) = cmp::eq(&col, &StringArray::new_scalar(target)) {
        out.or_boolean_array_with_pool(&cmp_arr, default_matches, Some(type_mask), pool);
    }
}

/// Computes a row mask where `attr_type == vtype` intersected with `base_mask` using SIMD `cmp::eq`.
#[must_use]
pub(crate) fn eval_anyval_type_mask(
    attr_type: &UInt8Array,
    vtype: AttributeValueType,
    base_mask: Option<&DenseBitset>,
    pool: &mut BitsetBufferPool,
) -> DenseBitset {
    if attr_type.is_empty() || base_mask.is_some_and(DenseBitset::is_empty) {
        return DenseBitset::new();
    }
    let Ok(cmp_arr) = cmp::eq(attr_type, &UInt8Array::new_scalar(vtype as u8)) else {
        return DenseBitset::new();
    };
    DenseBitset::from_boolean_array(&cmp_arr, false, base_mask, pool)
}

/// Computes the exact `Predicate::Exists` bitmask for a `logs.body` `StructArray` (`AnyValue`).
///
/// Mirrors `extract_anyval`: `Str`/`Int`/`Double`/`Bytes` always exist (via `default_values_optional`),
/// `Bool` exists iff `attr_bool` is valid, `Map`/`Slice` exist iff `attr_ser` decodes as valid CBOR,
/// and `Empty` / invalid discriminants / null structs do not exist.
#[must_use]
pub(crate) fn eval_anyval_body_exists_mask(
    arrays: &AnyValueArrays<'_>,
    base_mask: Option<&DenseBitset>,
    pool: &mut BitsetBufferPool,
) -> DenseBitset {
    let len = arrays.attr_type.len();
    if len == 0 || base_mask.is_some_and(DenseBitset::is_empty) {
        return DenseBitset::new();
    }
    let row_words = len.div_ceil(64).max(1);
    let mut out = DenseBitset::new();

    // Str (1), Int (2), Double (3) always produce a value (omitted/null => type default).
    if let (Ok(gt0), Ok(le3)) = (
        cmp::gt(arrays.attr_type, &UInt8Array::new_scalar(0)),
        cmp::lt_eq(
            arrays.attr_type,
            &UInt8Array::new_scalar(AttributeValueType::Double as u8),
        ),
    ) {
        let mut s123 = DenseBitset::from_boolean_array(&gt0, false, base_mask, pool);
        let le3_bs = DenseBitset::from_boolean_array(&le3, false, base_mask, pool);
        s123.and_assign(&le3_bs);
        pool.recycle(le3_bs);
        out.or_assign_with_pool(&s123, pool);
        pool.recycle(s123);
    }

    // Bytes (7) always produces a value (omitted/null => `&[]`).
    let bytes_mask =
        eval_anyval_type_mask(arrays.attr_type, AttributeValueType::Bytes, base_mask, pool);
    out.or_assign_with_pool(&bytes_mask, pool);
    pool.recycle(bytes_mask);

    // Bool (4) exists iff `attr_bool` is present and valid at the row.
    if let Some(ba) = arrays.attr_bool {
        let bool_type =
            eval_anyval_type_mask(arrays.attr_type, AttributeValueType::Bool, base_mask, pool);
        if !bool_type.is_empty() {
            let valid_bool =
                DenseBitset::from_validity_buffers(len, ba.nulls(), None, Some(&bool_type), pool);
            out.or_assign_with_pool(&valid_bool, pool);
            pool.recycle(valid_bool);
        }
        pool.recycle(bool_type);
    }

    // Map (5) and Slice (6) exist iff `attr_ser` is present and decodes as valid CBOR.
    if let Some(ser) = &arrays.attr_ser {
        let map_type =
            eval_anyval_type_mask(arrays.attr_type, AttributeValueType::Map, base_mask, pool);
        let slice_type =
            eval_anyval_type_mask(arrays.attr_type, AttributeValueType::Slice, base_mask, pool);
        let mut cbor_rows = map_type;
        cbor_rows.or_assign_with_pool(&slice_type, pool);
        pool.recycle(slice_type);
        for row_u32 in cbor_rows.iter() {
            let row = row_u32 as usize;
            if let Some(raw) = ser.slice_at(row)
                && extract_cbor_path_for_eval(raw, &[], false).is_some()
            {
                out.insert_with_pool(row_u32, row_words, pool);
            }
        }
        pool.recycle(cbor_rows);
    }

    out
}

/// Computes the exact `Predicate::Exists` bitmask for a root string column (`StringArray` or
/// `DictionaryArray<UInt8|UInt16, StringArray>`), using O(N/64) validity buffers when possible
/// and falling back to SIMD `cmp::neq("", ...)` when `skip_empty` is `true` and empty strings may
/// be present.
#[must_use]
pub(crate) fn eval_str_exists_bitmap(
    col: &ArrayRef,
    parent_validity: Option<&NullBuffer>,
    num_rows: usize,
    active_rows: Option<&DenseBitset>,
    skip_empty: bool,
    pool: &mut BitsetBufferPool,
) -> DenseBitset {
    if num_rows == 0 || active_rows.is_some_and(DenseBitset::is_empty) {
        return DenseBitset::new();
    }
    if col.as_any().downcast_ref::<StringArray>().is_some() {
        if !skip_empty {
            return DenseBitset::from_validity_buffers(
                num_rows,
                col.nulls(),
                parent_validity,
                active_rows,
                pool,
            );
        }
        let Ok(non_empty) = cmp::neq(col, &StringArray::new_scalar("")) else {
            return DenseBitset::new();
        };
        if let Some(pnulls) = parent_validity {
            let parent_bs =
                DenseBitset::from_validity_buffers(num_rows, None, Some(pnulls), active_rows, pool);
            let out = DenseBitset::from_boolean_array(&non_empty, false, Some(&parent_bs), pool);
            pool.recycle(parent_bs);
            return out;
        }
        return DenseBitset::from_boolean_array(&non_empty, false, active_rows, pool);
    }

    let dict_values: Option<&StringArray> =
        if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
            da.values().as_any().downcast_ref::<StringArray>()
        } else if let Some(da) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
            da.values().as_any().downcast_ref::<StringArray>()
        } else {
            None
        };

    let Some(values) = dict_values else {
        return DenseBitset::new();
    };

    if !skip_empty {
        if values.null_count() == 0 {
            return DenseBitset::from_validity_buffers(
                num_rows,
                col.nulls(),
                parent_validity,
                active_rows,
                pool,
            );
        }
        let logical = col.logical_nulls();
        return DenseBitset::from_validity_buffers(
            num_rows,
            logical.as_ref(),
            parent_validity,
            active_rows,
            pool,
        );
    }

    // When `skip_empty` is true, check whether any dictionary entry is null or `""`.
    // Under OTAP `default_values_optional`, `""` is omitted/null rather than stored in the dict,
    // so this O(dict_len) check almost always unlocks an O(N/64) validity-bitmap fast path.
    let has_null_or_empty =
        values.null_count() > 0 || (0..values.len()).any(|i| values.value(i).is_empty());
    if !has_null_or_empty {
        return DenseBitset::from_validity_buffers(
            num_rows,
            col.nulls(),
            parent_validity,
            active_rows,
            pool,
        );
    }

    let Ok(non_empty) = cmp::neq(col, &StringArray::new_scalar("")) else {
        return DenseBitset::new();
    };
    if let Some(pnulls) = parent_validity {
        let parent_bs =
            DenseBitset::from_validity_buffers(num_rows, None, Some(pnulls), active_rows, pool);
        let out = DenseBitset::from_boolean_array(&non_empty, false, Some(&parent_bs), pool);
        pool.recycle(parent_bs);
        return out;
    }
    DenseBitset::from_boolean_array(&non_empty, false, active_rows, pool)
}

/// Evaluates `LogRecordField::SeverityNumber` predicates (`Exists`, `Equals`, `Gt/Gte/Lt/Lte`)
/// columnarly via `arrow::compute::kernels::cmp`, treating `0` (`SEVERITY_NUMBER_UNSPECIFIED`)
/// as absent per OTel policy spec.
pub(crate) fn eval_severity_number_columnar(
    col: &ArrayRef,
    exists_preds: &[usize],
    scalar_and_range_preds: &[(usize, Predicate)],
    active_rows: Option<&DenseBitset>,
    pool: &mut BitsetBufferPool,
    pred_bitmaps: &mut [DenseBitset],
) {
    if col.is_empty() || active_rows.is_some_and(DenseBitset::is_empty) {
        return;
    }
    let leaf_dt = match col.data_type() {
        DataType::Dictionary(k, v) if matches!(k.as_ref(), DataType::UInt8 | DataType::UInt16) => {
            v.as_ref()
        }
        other => other,
    };

    match leaf_dt {
        DataType::Int32 => {
            let needs_non_zero = !exists_preds.is_empty()
                || scalar_and_range_preds
                    .iter()
                    .any(|(_, p)| Int64CmpOp::from_predicate(p).narrow_to_i32().matches_zero());
            let non_zero = if needs_non_zero {
                let Ok(nz_arr) = cmp::neq(col, &Int32Array::new_scalar(0)) else {
                    return;
                };
                let nz = DenseBitset::from_boolean_array(&nz_arr, false, active_rows, pool);
                if nz.is_empty() {
                    pool.recycle(nz);
                    return;
                }
                for &pid in exists_preds {
                    pred_bitmaps[pid].or_assign_with_pool(&nz, pool);
                }
                Some(nz)
            } else {
                None
            };

            for (pid, pred) in scalar_and_range_preds {
                let op = Int64CmpOp::from_predicate(pred).narrow_to_i32();
                match op {
                    Int32CmpOp::AllFalse | Int32CmpOp::Eq(0) => {}
                    Int32CmpOp::AllTrue => {
                        if let Some(nz) = &non_zero {
                            pred_bitmaps[*pid].or_assign_with_pool(nz, pool);
                        }
                    }
                    _ => {
                        let mask = if op.matches_zero() {
                            non_zero.as_ref()
                        } else {
                            active_rows
                        };
                        if let Some(cmp_arr) = run_i32_cmp_kernel(col.as_ref(), op) {
                            pred_bitmaps[*pid]
                                .or_boolean_array_with_pool(&cmp_arr, false, mask, pool);
                        }
                    }
                }
            }
            if let Some(nz) = non_zero {
                pool.recycle(nz);
            }
        }
        DataType::Int64 => {
            let needs_non_zero = !exists_preds.is_empty()
                || scalar_and_range_preds
                    .iter()
                    .any(|(_, p)| Int64CmpOp::from_predicate(p).matches_zero());
            let non_zero = if needs_non_zero {
                let Ok(nz_arr) = cmp::neq(col, &Int64Array::new_scalar(0)) else {
                    return;
                };
                let nz = DenseBitset::from_boolean_array(&nz_arr, false, active_rows, pool);
                if nz.is_empty() {
                    pool.recycle(nz);
                    return;
                }
                for &pid in exists_preds {
                    pred_bitmaps[pid].or_assign_with_pool(&nz, pool);
                }
                Some(nz)
            } else {
                None
            };

            for (pid, pred) in scalar_and_range_preds {
                let op = Int64CmpOp::from_predicate(pred);
                match op {
                    Int64CmpOp::AllFalse | Int64CmpOp::Eq(0) => {}
                    Int64CmpOp::AllTrue => {
                        if let Some(nz) = &non_zero {
                            pred_bitmaps[*pid].or_assign_with_pool(nz, pool);
                        }
                    }
                    _ => {
                        let mask = if op.matches_zero() {
                            non_zero.as_ref()
                        } else {
                            active_rows
                        };
                        if let Some(cmp_arr) = run_i64_cmp_kernel(col.as_ref(), op) {
                            pred_bitmaps[*pid]
                                .or_boolean_array_with_pool(&cmp_arr, false, mask, pool);
                        }
                    }
                }
            }
            if let Some(nz) = non_zero {
                pool.recycle(nz);
            }
        }
        _ => {}
    }
}

/// Pre-resolved accessor for `FixedSizeBinaryArray(N)` or `DictionaryArray<UInt8|UInt16, FixedSizeBinaryArray(N)>`.
enum FixedBinaryCol<'a> {
    Native(&'a FixedSizeBinaryArray),
    Dict8 {
        arr: &'a DictionaryArray<UInt8Type>,
        values: &'a FixedSizeBinaryArray,
    },
    Dict16 {
        arr: &'a DictionaryArray<UInt16Type>,
        values: &'a FixedSizeBinaryArray,
    },
}

impl<'a> FixedBinaryCol<'a> {
    fn try_from_array(col: &'a ArrayRef, expected_len: i32) -> Option<Self> {
        if let Some(fsb) = col.as_any().downcast_ref::<FixedSizeBinaryArray>() {
            return (fsb.value_length() == expected_len).then_some(Self::Native(fsb));
        }
        if let Some(arr) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>()
            && let Some(values) = arr.values().as_any().downcast_ref::<FixedSizeBinaryArray>()
        {
            return (values.value_length() == expected_len).then_some(Self::Dict8 { arr, values });
        }
        if let Some(arr) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>()
            && let Some(values) = arr.values().as_any().downcast_ref::<FixedSizeBinaryArray>()
        {
            return (values.value_length() == expected_len).then_some(Self::Dict16 { arr, values });
        }
        None
    }

    #[inline]
    fn value_at(&self, row: usize) -> Option<&'a [u8]> {
        match self {
            Self::Native(fsb) => fsb.is_valid(row).then(|| fsb.value(row)),
            Self::Dict8 { arr, values } => {
                if !arr.is_valid(row) {
                    return None;
                }
                let k = arr.keys().value(row) as usize;
                values.is_valid(k).then(|| values.value(k))
            }
            Self::Dict16 { arr, values } => {
                if !arr.is_valid(row) {
                    return None;
                }
                let k = arr.keys().value(row) as usize;
                values.is_valid(k).then(|| values.value(k))
            }
        }
    }
}

/// Evaluates `TraceId` (`N = 16`) or `SpanId` (`N = 8`) predicates using SIMD `cmp::neq([0; N])`
/// for presence filtering, SIMD `cmp::eq` for small exact-match sets, and pre-resolved
/// `FixedBinaryCol` scanning for large exact-match sets or hex regex/contains predicates.
#[allow(clippy::too_many_arguments)]
pub(crate) fn eval_fixed_id_columnar<const N: usize>(
    col: &ArrayRef,
    exists_preds: &[usize],
    equals_map: &AHashMap<[u8; N], Vec<usize>>,
    has_string_preds: bool,
    active_rows: Option<&DenseBitset>,
    row_words: usize,
    pool: &mut BitsetBufferPool,
    pred_bitmaps: &mut [DenseBitset],
    mut eval_hex: impl FnMut(&str, &mut dyn FnMut(usize)),
) {
    if col.is_empty() || active_rows.is_some_and(DenseBitset::is_empty) {
        return;
    }
    let Some(fixed_col) = FixedBinaryCol::try_from_array(col, N as i32) else {
        return;
    };
    let zero_id = [0u8; N];
    let Ok(nz_arr) = cmp::neq(col, &FixedSizeBinaryArray::new_scalar(zero_id)) else {
        return;
    };
    let non_zero = DenseBitset::from_boolean_array(&nz_arr, false, active_rows, pool);
    if non_zero.is_empty() {
        pool.recycle(non_zero);
        return;
    }

    for &pid in exists_preds {
        pred_bitmaps[pid].or_assign_with_pool(&non_zero, pool);
    }

    if !has_string_preds && equals_map.len() <= SMALL_ID_EQ_THRESHOLD {
        for (id_bytes, pids) in equals_map {
            if *id_bytes == zero_id {
                continue;
            }
            if let Ok(eq_arr) = cmp::eq(col, &FixedSizeBinaryArray::new_scalar(*id_bytes)) {
                for &pid in pids {
                    pred_bitmaps[pid].or_boolean_array_with_pool(
                        &eq_arr,
                        false,
                        Some(&non_zero),
                        pool,
                    );
                }
            }
        }
        pool.recycle(non_zero);
        return;
    }

    let mut hex_buf = [0u8; 32];
    let hex_slice = &mut hex_buf[..N * 2];
    for row_u32 in non_zero.iter() {
        let row = row_u32 as usize;
        let Some(bytes) = fixed_col.value_at(row) else {
            continue;
        };
        let Ok(id_bytes) = <[u8; N]>::try_from(bytes) else {
            continue;
        };
        if let Some(pids) = equals_map.get(&id_bytes) {
            for &pid in pids {
                pred_bitmaps[pid].insert_with_pool(row_u32, row_words, pool);
            }
        }
        if has_string_preds
            && hex::encode_to_slice(id_bytes, hex_slice).is_ok()
            && let Ok(hex_str) = std::str::from_utf8(hex_slice)
        {
            eval_hex(hex_str, &mut |pid| {
                pred_bitmaps[pid].insert_with_pool(row_u32, row_words, pool);
            });
        }
    }
    pool.recycle(non_zero);
}
