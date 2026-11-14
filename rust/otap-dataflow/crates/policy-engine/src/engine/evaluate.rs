// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Arrow-based evaluation of policy matchers against OTAP batches.
//!
//! This module evaluates `TypedCheck` and `ExistenceCheck` on arrow data within the OTAP batches.
//!
//! # Spec conformance
//!
//! - Type mismatches and missing fields/columns are non-matches (fail open)
//! - Integer and floating-point values are compared in a single numeric domain: an int matcher may
//!   match a double column and vice versa.
//!

// TODO: (non-exhaustive list)
// - Currently every time an attribute is accessed, we scan the key column. Potential for caching.
// - Fields the log "body" resource/scope schema url, field are not supported correctly
// - Nested attribute access (multi part keys) are not supported correctly.
// - Proper error handling (needed for error statistics).

use std::collections::HashMap;

use arrow::array::{
    Array, ArrayRef, BooleanArray, BooleanBufferBuilder, Datum, DictionaryArray, Float64Array,
    Int64Array, RecordBatch, StringArray, UInt8Array, UInt16Array,
};
use arrow::compute::kernels::cmp;
use arrow::datatypes::{DataType, UInt8Type, UInt16Type};
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmap, build_native_selection_vec};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use policy_rs::PolicyError;
use policy_rs::engine::{
    CompiledDatabase, CompiledMatchers, CompiledNumericValue, CompiledTypedMatcher, CompiledValue,
    ExistenceCheck, LogSignal, TypedCheck,
};

use crate::engine::policy_match::PolicyMatch;
use crate::engine::transform::{FieldTarget, field_selector_to_otap_target};

/// Evaluate a [`TypedCheck`] against an OTAP batch, returning a [`PolicyMatch`] indicating which
/// rows satisfy the check.
///
/// Handles negation internally: if `check.is_negated`, the result is inverted before returning.
pub(crate) fn evaluate_typed_check(
    otap_batch: &OtapArrowRecords,
    check: &TypedCheck<LogSignal>,
) -> PolicyMatch {
    let Some(target) = field_selector_to_otap_target(&check.field) else {
        // Unsupported field (e.g., nested attribute path) -- non-match.
        return PolicyMatch::None;
    };

    let result = match target {
        FieldTarget::Attribute(key, payload_type) => {
            evaluate_typed_check_on_attribute(otap_batch, key, payload_type, &check.matcher)
        }
        FieldTarget::SimpleField(column_name) => {
            // TODO return an error here if column_name is "body" because this function
            // doesn't handle it correctly
            evaluate_typed_check_on_root(otap_batch, column_name, &check.matcher)
        }
    };

    // TODO - we could potentially avoid allocating a buffer if internally we were to swap the
    // predicates instead (e.g eq -> ne, gt -> lte, etc.) of just evaluating the operation specified
    // by the matcher and then inverting the result afterward.
    if check.is_negated {
        PolicyMatch::not(&result)
    } else {
        result
    }
}

/// Evaluate an [`ExistenceCheck`] against an OTAP batch, returning a [`PolicyMatch`] indicating
/// which rows satisfy the check.
///
/// Handles negation internally.
pub(crate) fn evaluate_existence_check(
    otap_batch: &OtapArrowRecords,
    check: &ExistenceCheck<LogSignal>,
) -> PolicyMatch {
    let Some(target) = field_selector_to_otap_target(&check.field) else {
        return PolicyMatch::None;
    };

    let result = match target {
        FieldTarget::Attribute(key, payload_type) => {
            evaluate_existence_on_attribute(otap_batch, key, payload_type, check.should_exist)
        }
        FieldTarget::SimpleField(column_name) => {
            // TODO return an error here if column_name is "body" because this function
            // doesn't handle it correctly
            evaluate_existence_on_root(otap_batch, column_name, check.should_exist)
        }
    };

    // TODO - we could avoid allocating & copying a buffer if we were to push the negation
    // down into the existence check above (maybe by inverting should_exist).
    if check.is_negated {
        PolicyMatch::not(&result)
    } else {
        result
    }
}

/// Evaluate a typed comparison on a root record batch column.
fn evaluate_typed_check_on_root(
    otap_batch: &OtapArrowRecords,
    column_name: &str,
    matcher: &CompiledTypedMatcher,
) -> PolicyMatch {
    let Some(root_batch) = otap_batch.root_record_batch() else {
        return PolicyMatch::None;
    };

    let Some(column) = root_batch.column_by_name(column_name) else {
        // Column not present -- non-match per spec.
        return PolicyMatch::None;
    };

    let result = match matcher {
        CompiledTypedMatcher::Equals(value) => evaluate_equals(column, value),
        CompiledTypedMatcher::Gt(nv) => evaluate_numeric_cmp(column, nv, cmp::gt),
        CompiledTypedMatcher::Gte(nv) => evaluate_numeric_cmp(column, nv, cmp::gt_eq),
        CompiledTypedMatcher::Lt(nv) => evaluate_numeric_cmp(column, nv, cmp::lt),
        CompiledTypedMatcher::Lte(nv) => evaluate_numeric_cmp(column, nv, cmp::lt_eq),
    };

    match result {
        Some(bool_arr) => PolicyMatch::from_boolean_array(&bool_arr),
        // Type mismatch or unsupported type -- non-match per spec.
        None => PolicyMatch::None,
    }
}

/// Evaluate an `Equals` matcher against a column.
///
/// Returns `None` for type mismatches or unsupported value types.
fn evaluate_equals(column: &ArrayRef, value: &CompiledValue) -> Option<BooleanArray> {
    match value {
        CompiledValue::Int(i) => {
            evaluate_numeric_cmp(column, &CompiledNumericValue::Int(*i), cmp::eq)
        }
        CompiledValue::Double(d) => {
            evaluate_numeric_cmp(column, &CompiledNumericValue::Double(*d), cmp::eq)
        }

        CompiledValue::Bool(v) => {
            let scalar = BooleanArray::new_scalar(*v);
            cmp::eq(column, &scalar).ok()
        }
        CompiledValue::Bytes(_) => {
            todo!("Equals(Bytes) comparison for byte-typed fields")
        }
    }
}

fn evaluate_numeric_cmp<F>(
    column: &ArrayRef,
    numeric_value: &CompiledNumericValue,
    cmp_fn: F,
) -> Option<BooleanArray>
where
    F: Fn(&dyn Datum, &dyn Datum) -> Result<BooleanArray, arrow::error::ArrowError> + Copy,
{
    // Note: the spec has some rules around comparing floats to int that we'll need to abide by.
    // TODO the logic below is NOT correct vis-a-vis the type coercion and currently panics in
    // some cases of type mismatches.
    match numeric_value {
        CompiledNumericValue::Int(int64_val) => {
            match effective_data_type(column) {
                DataType::Int64 => {
                    let scalar = Int64Array::new_scalar(*int64_val);
                    Some(cmp_fn(column, &scalar).unwrap())
                }
                DataType::Float64 => {
                    let scalar = Float64Array::new_scalar(*int64_val as f64);
                    Some(cmp_fn(column, &scalar).unwrap())
                }

                _ => {
                    // TODO handle other numeric data types
                    todo!("numeric coercion")
                }
            }
        }
        CompiledNumericValue::Double(double_val) => {
            match effective_data_type(column) {
                DataType::Float64 => {
                    let scalar = Float64Array::new_scalar(*double_val);
                    Some(cmp_fn(column, &scalar).unwrap())
                }

                _ => {
                    // TODO handle other numeric data types
                    todo!("numeric coercion")
                }
            }
        }
    }
}

fn effective_data_type(column: &ArrayRef) -> DataType {
    // TODO try to avoid cloning the datatype here, which is not "free" for non-primitive types
    // (e.g. Dictionaries and Structs) due to pointers/vecs contained therein
    let dt = column.data_type();
    match dt {
        DataType::Dictionary(_, v) => v.as_ref().clone(),
        _ => dt.clone(),
    }
}

/// Evaluate a field existence check on the root record batch.
///
/// - `should_exist: true` -- returns true for rows where the column is not null.
///   If the column does not exist in the batch, all rows are non-matches.
/// - `should_exist: false` -- returns true for rows where the column is null.
///   If the column does not exist in the batch, all rows are matches.
fn evaluate_existence_on_root(
    otap_batch: &OtapArrowRecords,
    column_name: &str,
    should_exist: bool,
) -> PolicyMatch {
    let no_field_result = if should_exist {
        PolicyMatch::None
    } else {
        PolicyMatch::All
    };
    let Some(root_batch) = otap_batch.root_record_batch() else {
        return no_field_result;
    };

    let num_rows = root_batch.num_rows();

    // TODO - handle getting the special resource/scope schema URL from struct columns on the root

    // TODO - handle "AnyValue" which we need to look inside of and account for the struct null
    // buffer, the type being Empty, and the null buffer within the various values columns. This
    // would be needed for log body.

    let Some(column) = root_batch.column_by_name(column_name) else {
        // Column missing entirely from the batch.
        return no_field_result;
    };

    let null_count = column.null_count();

    if null_count == num_rows {
        return no_field_result;
    }

    let Some(validity) = column.nulls() else {
        // all rows are valid
        return PolicyMatch::not(&no_field_result);
    };

    PolicyMatch::from_boolean_buffer(if should_exist {
        validity.inner().clone()
    } else {
        !validity.inner()
    })
}

/// Evaluate a field existence check on an attribute child batch.
///
/// For `should_exist: true`: returns true for root rows that have an attribute with the given key.
/// For `should_exist: false`: returns true for root rows that do NOT have the attribute.
fn evaluate_existence_on_attribute(
    otap_batch: &OtapArrowRecords,
    attr_key: &str,
    payload_type: ArrowPayloadType,
    should_exist: bool,
) -> PolicyMatch {
    // this is what we'll return if there is no matching attrs at all
    let no_attrs_result = if should_exist {
        PolicyMatch::None
    } else {
        PolicyMatch::All
    };
    let root_batch = match otap_batch.root_record_batch() {
        Some(rb) => rb,
        None => return no_attrs_result,
    };

    // Get the root id column and build a root-aligned selection vec.
    let Some(root_id_col) = get_root_id_column(root_batch, payload_type) else {
        // No ID column means no attributes
        return no_attrs_result;
    };

    // Get the attribute batch. If it doesn't exist, no rows have any attributes of this type.
    let Some(attrs_batch) = otap_batch.get(payload_type) else {
        return no_attrs_result;
    };

    // Get the key column and find rows matching the target key.
    let Some(key_col) = attrs_batch.column_by_name(consts::ATTRIBUTE_KEY) else {
        // TODO - consider returning error here? This would be an invalid batch.
        // Need to confirm what the spec says to do in this case.
        return no_attrs_result;
    };

    let key_scalar = StringArray::new_scalar(attr_key);
    let Ok(key_mask) = cmp::eq(key_col, &key_scalar) else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };

    if key_mask.true_count() == 0 {
        // No rows match the key.
        return no_attrs_result;
    }

    // Get the parent_id column from the attribute batch.
    let Some(parent_id_col) = attrs_batch.column_by_name(consts::PARENT_ID) else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };
    let Some(parent_id_u16) = parent_id_col.as_any().downcast_ref::<UInt16Array>() else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };

    // Populate an IdBitmap with the parent_ids of rows that have the key.
    // TODO: use an IdBitmapPool for reuse between evaluations.
    let mut id_bitmap = IdBitmap::new();
    for i in 0..parent_id_u16.len() {
        if key_mask.value(i) {
            id_bitmap.insert(u32::from(parent_id_u16.value(i)));
        }
    }

    // TODO - We could change the impl of build_native_selection_vec to possibly avoid
    // a heap allocation/drop for the Arc containing the BooleanArray values and have it
    // simply return an owned BooleanBuffer. Once this is done, we should grep this file for
    // all usages and resolve this issue (maybe removing from_boolean_array for PolicyMatch)
    let selection = build_native_selection_vec(root_id_col, &id_bitmap);
    let result = PolicyMatch::from_boolean_array(&selection);

    // For should_exist: false, invert the result
    //
    // TODO build the inverted bitmap above where we populate the IdBitmap instead of inverting here
    // which would save us a copy to negate the buffer in the case only some rows are matched.
    if should_exist {
        result
    } else {
        PolicyMatch::not(&result)
    }
}

/// Evaluate a typed comparison on an attribute child batch.
fn evaluate_typed_check_on_attribute(
    otap_batch: &OtapArrowRecords,
    attr_key: &str,
    payload_type: ArrowPayloadType,
    matcher: &CompiledTypedMatcher,
) -> PolicyMatch {
    let root_batch = match otap_batch.root_record_batch() {
        Some(rb) => rb,
        None => return PolicyMatch::None,
    };

    let Some(root_id_col) = get_root_id_column(root_batch, payload_type) else {
        return PolicyMatch::None;
    };

    let Some(attrs_batch) = otap_batch.get(payload_type) else {
        return PolicyMatch::None;
    };

    // Filter by key.
    let Some(key_col) = attrs_batch.column_by_name(consts::ATTRIBUTE_KEY) else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };
    let key_scalar = StringArray::new_scalar(attr_key);
    let Ok(key_mask) = cmp::eq(key_col, &key_scalar) else {
        // If this failed, it probably is because of a type mismatch type mismatch.
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };
    if key_mask.true_count() == 0 {
        return PolicyMatch::None;
    }

    // Get the parent_id column.
    let Some(parent_id_col) = attrs_batch.column_by_name(consts::PARENT_ID) else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };
    let Some(parent_id_u16) = parent_id_col.as_any().downcast_ref::<UInt16Array>() else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };

    // Get the type column to check value type compatibility.
    let Some(type_col) = attrs_batch.column_by_name(consts::ATTRIBUTE_TYPE) else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };
    let Some(type_u8) = type_col.as_any().downcast_ref::<UInt8Array>() else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        return PolicyMatch::None;
    };

    // TODO: use an IdBitmapPool for reuse between evaluations.
    let mut id_bitmap = IdBitmap::new();

    // Determine which attribute value types and columns are relevant for this matcher,
    // then attempt the comparison on each.
    match matcher {
        CompiledTypedMatcher::Equals(CompiledValue::Bool(val)) => {
            collect_matching_attr_rows(
                attrs_batch,
                &key_mask,
                type_u8,
                parent_id_u16,
                consts::ATTRIBUTE_BOOL,
                AttributeValueType::Bool as u8,
                &BooleanArray::new_scalar(*val),
                cmp::eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Equals(CompiledValue::Int(val)) => {
            collect_matching_attr_rows_numeric(
                attrs_batch,
                &key_mask,
                type_u8,
                parent_id_u16,
                &CompiledNumericValue::Int(*val),
                cmp::eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Equals(CompiledValue::Double(val)) => {
            collect_matching_attr_rows_numeric(
                attrs_batch,
                &key_mask,
                type_u8,
                parent_id_u16,
                &CompiledNumericValue::Double(*val),
                cmp::eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Gt(nv) => {
            collect_matching_attr_rows_numeric(
                attrs_batch,
                &key_mask,
                type_u8,
                parent_id_u16,
                nv,
                cmp::gt,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Gte(nv) => {
            collect_matching_attr_rows_numeric(
                attrs_batch,
                &key_mask,
                type_u8,
                parent_id_u16,
                nv,
                cmp::gt_eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Lt(nv) => {
            collect_matching_attr_rows_numeric(
                attrs_batch,
                &key_mask,
                type_u8,
                parent_id_u16,
                nv,
                cmp::lt,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Lte(nv) => {
            collect_matching_attr_rows_numeric(
                attrs_batch,
                &key_mask,
                type_u8,
                parent_id_u16,
                nv,
                cmp::lt_eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Equals(CompiledValue::Bytes(_)) => {
            todo!("Equals(Bytes) comparison on attribute values")
        }
    };

    if id_bitmap.is_empty() {
        return PolicyMatch::None;
    }

    let selection = build_native_selection_vec(root_id_col, &id_bitmap);
    PolicyMatch::from_boolean_array(&selection)
}

/// Compare attribute values in a single typed column against a scalar, and insert the parent_ids
/// of matching rows into the `id_bitmap`.
///
/// Only considers rows where `key_mask` is true and `type_col` matches `expected_type`.
fn collect_matching_attr_rows<S: Datum, F>(
    attrs_batch: &RecordBatch,
    key_mask: &BooleanArray,
    type_col: &UInt8Array,
    parent_id_col: &UInt16Array,
    value_col_name: &str,
    expected_type: u8,
    scalar: &S,
    cmp_fn: F,
    id_bitmap: &mut IdBitmap,
) where
    F: Fn(&dyn Datum, &dyn Datum) -> Result<BooleanArray, arrow::error::ArrowError>,
{
    let Some(value_col) = attrs_batch.column_by_name(value_col_name) else {
        return;
    };

    // TODO the logic here probably needs to be reworked/optimized..
    //
    // The `eq` kernel to check the `type` column below might be faster than checking it inline
    // while iterating the rows. Although, if there are many other matchers that need to check
    // attrs of the same type, it might be more efficient to do this once and cache the result.
    //
    // Also, we check the cmp_fn for every row, but we actually only need the result for rows
    // where there is a key match. We could consider pre-filtering the ID column. Whether this is
    // actually a better thing to do probably depends on how selective is the key mask.

    // Compare the entire value column against the scalar.
    //
    // TODO - This will actually return an error for data types that weren't coerced properly which
    // means comparisons that should pass, but just aren't coerced correctly, will silently not
    // match the rows. Consider proper error handling.
    let Ok(cmp_result) = cmp_fn(value_col, scalar) else {
        return;
    };

    // Scan: row must match key AND have the right type AND pass the comparison.
    for i in 0..key_mask.len() {
        if key_mask.value(i) && type_col.value(i) == expected_type && cmp_result.value(i) {
            id_bitmap.insert(u32::from(parent_id_col.value(i)));
        }
    }
}

/// Compare numeric attribute values against a numeric threshold, checking
/// both `int` and `double` columns for cross-numeric domain matching per spec.
fn collect_matching_attr_rows_numeric<F>(
    attrs_batch: &RecordBatch,
    key_mask: &BooleanArray,
    type_col: &UInt8Array,
    parent_id_col: &UInt16Array,
    numeric_value: &CompiledNumericValue,
    cmp_fn: F,
    id_bitmap: &mut IdBitmap,
) where
    F: Fn(&dyn Datum, &dyn Datum) -> Result<BooleanArray, arrow::error::ArrowError> + Copy,
{
    // TODO - this doesn't properly handle  all numeric coercions

    match numeric_value {
        CompiledNumericValue::Int(v) => {
            // Check the int column directly.
            collect_matching_attr_rows(
                attrs_batch,
                key_mask,
                type_col,
                parent_id_col,
                consts::ATTRIBUTE_INT,
                AttributeValueType::Int as u8,
                &Int64Array::new_scalar(*v),
                cmp_fn,
                id_bitmap,
            );
            // Cross-numeric: also check the double column with promoted value.
            collect_matching_attr_rows(
                attrs_batch,
                key_mask,
                type_col,
                parent_id_col,
                consts::ATTRIBUTE_DOUBLE,
                AttributeValueType::Double as u8,
                &Float64Array::new_scalar(*v as f64),
                cmp_fn,
                id_bitmap,
            );
        }
        CompiledNumericValue::Double(v) => {
            // Check the double column directly.
            collect_matching_attr_rows(
                attrs_batch,
                key_mask,
                type_col,
                parent_id_col,
                consts::ATTRIBUTE_DOUBLE,
                AttributeValueType::Double as u8,
                &Float64Array::new_scalar(*v),
                cmp_fn,
                id_bitmap,
            );
            // Cross-numeric: also check the int column if the double is exact.
            if v.fract() == 0.0 && *v >= i64::MIN as f64 && *v <= i64::MAX as f64 {
                collect_matching_attr_rows(
                    attrs_batch,
                    key_mask,
                    type_col,
                    parent_id_col,
                    consts::ATTRIBUTE_INT,
                    AttributeValueType::Int as u8,
                    &Int64Array::new_scalar(*v as i64),
                    cmp_fn,
                    id_bitmap,
                );
            }
        }
    }
}

/// Get the root id column that corresponds to a given attribute payload type.
///
/// For log/span/metric attributes, the id is a plain column on the root batch.
/// For resource/scope attributes, the id is nested inside a struct column.
fn get_root_id_column<'a>(
    root_batch: &'a RecordBatch,
    attrs_payload_type: ArrowPayloadType,
) -> Option<&'a UInt16Array> {
    match attrs_payload_type {
        ArrowPayloadType::ResourceAttrs => root_batch
            .column_by_name(consts::RESOURCE)
            .and_then(|col| col.as_any().downcast_ref::<arrow::array::StructArray>())
            .and_then(|s| s.column_by_name(consts::ID))
            .and_then(|col| col.as_any().downcast_ref::<UInt16Array>()),
        ArrowPayloadType::ScopeAttrs => root_batch
            .column_by_name(consts::SCOPE)
            .and_then(|col| col.as_any().downcast_ref::<arrow::array::StructArray>())
            .and_then(|s| s.column_by_name(consts::ID))
            .and_then(|col| col.as_any().downcast_ref::<UInt16Array>()),
        _ => root_batch
            .column_by_name(consts::ID)
            .and_then(|col| col.as_any().downcast_ref::<UInt16Array>()),
    }
}

// ---------------------------------------------------------------------------
// Vectorscan database evaluation
// ---------------------------------------------------------------------------

/// Evaluate all Vectorscan databases against the OTAP batch, updating the per-policy `matched`
/// bitmaps.
pub(crate) fn evaluate_vectorscan_databases(
    compiled: &CompiledMatchers<LogSignal>,
    otap_batch: &OtapArrowRecords,
    matched: &mut [PolicyMatch],
) -> Result<(), PolicyError> {
    let root_batch = match otap_batch.root_record_batch() {
        Some(rb) => rb,
        None => return Ok(()),
    };
    let num_root_rows = root_batch.num_rows();

    for (key, db) in &compiled.databases {
        let target = match field_selector_to_otap_target(&key.field) {
            Some(t) => t,
            None => {
                // Unsupported field -- mark all policies in this db as non-matching.
                // TODO - double check this is the right thing to do here (e.g. should this have
                // been a policy compile error due to an invalid key).
                all_policies_no_match(db, matched);

                continue;
            }
        };

        match target {
            FieldTarget::SimpleField(col_name) => {
                evaluate_vectorscan_on_root_column(
                    root_batch,
                    col_name,
                    db,
                    key.negated,
                    num_root_rows,
                    matched,
                )?;
            }
            FieldTarget::Attribute(attr_key, payload_type) => {
                evaluate_vectorscan_on_attribute(
                    otap_batch,
                    root_batch,
                    attr_key,
                    payload_type,
                    db,
                    key.negated,
                    num_root_rows,
                    matched,
                )?;
            }
        }
    }

    Ok(())
}

/// Evaluate a Vectorscan database against a root record batch column.
fn evaluate_vectorscan_on_root_column(
    root_batch: &RecordBatch,
    col_name: &str,
    db: &CompiledDatabase,
    negated: bool,
    num_rows: usize,
    matched: &mut [PolicyMatch],
) -> Result<(), PolicyError> {
    if col_name == consts::BODY {
        // TODO: "body" field not yet supported. We'll need special logic for this to pull out
        // the "str" column from the AnyValue struct, check the "type" column is string, etc.
        return Err(PolicyError::CompileError {
            reason: "'body' field not yet supported".into(),
        });
    }

    let Some(column) = root_batch.column_by_name(col_name) else {
        // Column missing -- no patterns can match.
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };

    // Build per-policy hit bitmaps by scanning the column's string values.
    //
    // TODO: no unwrapping below. Should use expect w/ safety comments or arrow cast helpers.
    let policy_hits = if let Some(str_arr) = column.as_any().downcast_ref::<StringArray>() {
        scan_plain_string_column(str_arr, db, num_rows)?
    } else if let Some(dict) = column.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
        let values = dict
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        scan_dict_string_column(dict.keys(), values, db, num_rows)?
    } else if let Some(dict) = column
        .as_any()
        .downcast_ref::<DictionaryArray<UInt16Type>>()
    {
        let values = dict
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        scan_dict_string_column(dict.keys(), values, db, num_rows)?
    } else {
        // Non-string column (or unsupported OTAP arrow dict key).
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };

    // Fold hits into the matched bitmaps.
    apply_policy_hits(&policy_hits, db, negated, matched);

    Ok(())
}

/// Evaluate a Vectorscan database against an attribute string column.
fn evaluate_vectorscan_on_attribute(
    otap_batch: &OtapArrowRecords,
    root_batch: &RecordBatch,
    attr_key: &str,
    payload_type: ArrowPayloadType,
    db: &CompiledDatabase,
    negated: bool,
    _num_root_rows: usize,
    matched: &mut [PolicyMatch],
) -> Result<(), PolicyError> {
    let Some(root_id_col) = get_root_id_column(root_batch, payload_type) else {
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };

    let Some(attrs_batch) = otap_batch.get(payload_type) else {
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };

    // Filter by attribute key.
    let Some(key_col) = attrs_batch.column_by_name(consts::ATTRIBUTE_KEY) else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };
    let key_scalar = StringArray::new_scalar(attr_key);
    let key_mask = cmp::eq(key_col, &key_scalar).map_err(|e| PolicyError::CompileError {
        reason: format!("attribute key comparison: {e}"),
    })?;
    if key_mask.true_count() == 0 {
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    }

    // Get the type column and check for string type.
    let Some(type_col) = attrs_batch.column_by_name(consts::ATTRIBUTE_TYPE) else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };
    let Some(type_u8) = type_col.as_any().downcast_ref::<UInt8Array>() else {
        // TODO - consider returning error here? This would be an invalid batch
        // Need to confirm what the spec says to do in this case.
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };

    // Get the "str" value column.
    let Some(str_col) = attrs_batch.column_by_name(consts::ATTRIBUTE_STR) else {
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };

    // Get parent_id column.
    let Some(parent_id_col) = attrs_batch.column_by_name(consts::PARENT_ID) else {
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };
    let Some(parent_id_u16) = parent_id_col.as_any().downcast_ref::<UInt16Array>() else {
        if !negated {
            all_policies_no_match(db, matched);
        }
        return Ok(());
    };

    // Scan attr rows that match the key and have type=Str.
    // Build per-policy IdBitmaps of matching parent_ids.
    let str_type = AttributeValueType::Str as u8;
    let num_attr_rows = attrs_batch.num_rows();
    let policy_id_bitmaps = scan_attr_string_column(
        str_col,
        &key_mask,
        type_u8,
        parent_id_u16,
        str_type,
        num_attr_rows,
        db,
    )?;

    // Convert per-policy IdBitmaps to root-aligned PolicyMatches and fold in.
    let mut seen_policies = std::collections::HashSet::new();
    for pattern_ref in &db.pattern_index {
        let policy_idx = pattern_ref.policy_index;
        if !seen_policies.insert(policy_idx) {
            continue;
        }

        let policy_hit = if let Some(id_bitmap) = policy_id_bitmaps.get(&policy_idx) {
            let selection = build_native_selection_vec(root_id_col, id_bitmap);
            PolicyMatch::from_boolean_array(&selection)
        } else {
            PolicyMatch::None
        };

        if negated {
            matched[policy_idx] = matched[policy_idx].and_not(&policy_hit);
        } else {
            matched[policy_idx] = matched[policy_idx].and(&policy_hit);
        }
    }

    Ok(())
}

/// Apply per-policy hit bitmaps from a Vectorscan scan to the `matched` array.
fn apply_policy_hits(
    policy_hits: &HashMap<usize, BooleanBufferBuilder>,
    db: &CompiledDatabase,
    negated: bool,
    matched: &mut [PolicyMatch],
) {
    let mut seen_policies = std::collections::HashSet::new();
    for pattern_ref in &db.pattern_index {
        let policy_idx = pattern_ref.policy_index;
        if !seen_policies.insert(policy_idx) {
            continue;
        }

        let policy_hit = if let Some(builder) = policy_hits.get(&policy_idx) {
            // TODO: find a way to avoid this finish_cloned?
            PolicyMatch::from_boolean_buffer(builder.finish_cloned())
        } else {
            PolicyMatch::None
        };

        if negated {
            matched[policy_idx] = matched[policy_idx].and_not(&policy_hit);
        } else {
            matched[policy_idx] = matched[policy_idx].and(&policy_hit);
        }
    }
}

/// Mark all policies in a database as non-matching.
fn all_policies_no_match(db: &CompiledDatabase, matched: &mut [PolicyMatch]) {
    for pattern_ref in &db.pattern_index {
        matched[pattern_ref.policy_index] = PolicyMatch::None;
    }
}

// ---------------------------------------------------------------------------
// Vectorscan string scanning helpers
// ---------------------------------------------------------------------------

/// Scan a plain (non-dictionary) `StringArray` through a Vectorscan database.
/// Returns per-policy hit bitmaps.
fn scan_plain_string_column(
    arr: &StringArray,
    db: &CompiledDatabase,
    num_rows: usize,
) -> Result<HashMap<usize, BooleanBufferBuilder>, PolicyError> {
    let mut policy_hits: HashMap<usize, BooleanBufferBuilder> = HashMap::new();

    // TODO: there are some ideas in the loop below for how we could optimize this, but before
    // jumping into those it might be worth it to look more deeply into VectorscanDatabase::scan
    // and understand if there's opportunity to call this in a more efficient manner. Currently we
    // are calling it for every individual string.

    for i in 0..num_rows {
        if arr.is_null(i) {
            // TODO: we should benchmark it but it might be helpful to switch this loop it iterate
            // non-null rows instead of calling is_null for each row (similar comment for the loop
            // below in phase 2)
            continue;
        }
        let value = arr.value(i);
        let scan_result = db.database.scan(value.as_bytes())?;

        // TODO we should benchmark the loop below to figure out how inefficient it is ...
        //
        // This might thrash the CPU cache by loading the match buffer builder for each policy for
        // each row. It'd be nice if we could have just kept each buffer in memory while we fill it
        // in for better cache locality.
        //
        // Also, it's typically faster with arrow's boolean if we set contiguous ranges of bits
        // in a single call instead of setting them all individually.
        for pattern_id in scan_result {
            if let Some(pattern_ref) = db.pattern_index.get(pattern_id as usize) {
                let builder = policy_hits
                    .entry(pattern_ref.policy_index)
                    .or_insert_with(|| {
                        let mut b = BooleanBufferBuilder::new(num_rows);
                        b.append_n(num_rows, false);
                        b
                    });
                builder.set_bit(i, true);
            }
        }
    }

    Ok(policy_hits)
}

/// Scan a dictionary-encoded string column through a Vectorscan database.
fn scan_dict_string_column<K>(
    keys: &arrow::array::PrimitiveArray<K>,
    values: &StringArray,
    db: &CompiledDatabase,
    num_rows: usize,
) -> Result<HashMap<usize, BooleanBufferBuilder>, PolicyError>
where
    K: arrow::datatypes::ArrowPrimitiveType,
    K::Native: Into<usize> + Copy,
{
    // Phase 1: scan each unique dictionary value once and cache which policies it matched by
    let mut scan_cache: HashMap<usize, Vec<u32>> = HashMap::new();

    for i in 0..num_rows {
        // TODO: we should benchmark it but it might be helpful to switch this loop it iterate
        // non-null rows instead of calling is_null for each row (similar comment for the loop
        // below in phase 2)
        if keys.is_null(i) {
            continue;
        }
        let dict_idx: usize = keys.value(i).into();
        if scan_cache.contains_key(&dict_idx) {
            continue;
        }
        let result = if values.is_null(dict_idx) {
            Vec::new()
        } else {
            db.database.scan(values.value(dict_idx).as_bytes())?
        };
        let _ = scan_cache.insert(dict_idx, result);
    }

    // Phase 2: map cached results back to rows.
    let mut policy_hits: HashMap<usize, BooleanBufferBuilder> = HashMap::new();
    for i in 0..num_rows {
        // TODO re: null check, see comment above in "phase 1" loop
        if keys.is_null(i) {
            continue;
        }
        let dict_idx: usize = keys.value(i).into();
        let Some(matching_patterns) = scan_cache.get(&dict_idx) else {
            continue;
        };

        // TODO - Not sure how inefficient what we're doing below is.. need to benchmark this and
        // figure out if it can be optimized.
        //
        // This might thrash the CPU cache by loading the match buffer builder for each policy for
        // each row. It'd be nice if we could have just kept each buffer in memory while we fill it
        // in for better cache locality.
        //
        // Also, it's typically faster with arrow's boolean if we set contiguous ranges of bits
        // in a single call instead of setting them all individually.
        for &pattern_id in matching_patterns {
            if let Some(pattern_ref) = db.pattern_index.get(pattern_id as usize) {
                let builder = policy_hits
                    .entry(pattern_ref.policy_index)
                    .or_insert_with(|| {
                        let mut b = BooleanBufferBuilder::new(num_rows);
                        b.append_n(num_rows, false);
                        b
                    });
                builder.set_bit(i, true);
            }
        }
    }

    Ok(policy_hits)
}

/// Scan attribute string values through a Vectorscan database, respecting key and type filters.
/// Returns per-policy `IdBitmap`s of matching parent_ids.
fn scan_attr_string_column(
    str_col: &ArrayRef,
    key_mask: &BooleanArray,
    type_col: &UInt8Array,
    parent_id_col: &UInt16Array,
    str_type: u8,
    num_rows: usize,
    db: &CompiledDatabase,
) -> Result<HashMap<usize, IdBitmap>, PolicyError> {
    // TODO no unwrap for the type coercions below. Use arrow tye cast utilities.
    if let Some(str_arr) = str_col.as_any().downcast_ref::<StringArray>() {
        scan_attr_plain_string(
            str_arr,
            key_mask,
            type_col,
            parent_id_col,
            str_type,
            num_rows,
            db,
        )
    } else if let Some(dict) = str_col
        .as_any()
        .downcast_ref::<DictionaryArray<UInt8Type>>()
    {
        let values = dict
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        scan_attr_dict_string(
            dict.keys(),
            values,
            key_mask,
            type_col,
            parent_id_col,
            str_type,
            num_rows,
            db,
        )
    } else if let Some(dict) = str_col
        .as_any()
        .downcast_ref::<DictionaryArray<UInt16Type>>()
    {
        let values = dict
            .values()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        scan_attr_dict_string(
            dict.keys(),
            values,
            key_mask,
            type_col,
            parent_id_col,
            str_type,
            num_rows,
            db,
        )
    } else {
        // Not a string column -- no matches.
        Ok(HashMap::new())
    }
}

/// Scan a plain attribute string column. Returns a map, keyed by policy index, of the rows it
/// matched via this vectorscan db.
fn scan_attr_plain_string(
    str_arr: &StringArray,
    key_mask: &BooleanArray,
    type_col: &UInt8Array,
    parent_id_col: &UInt16Array,
    str_type: u8,
    num_rows: usize,
    db: &CompiledDatabase,
) -> Result<HashMap<usize, IdBitmap>, PolicyError> {
    let mut policy_id_bitmaps: HashMap<usize, IdBitmap> = HashMap::new();

    // TODO: there are some ideas in the loop below for how we could write this better, but before
    // jumping into those it might be worth it to look more deeply into VectorscanDatabase::scan
    // and understand if there's opportunity to call this in a more efficient manner. Currently we
    // are calling it for every individual string.

    for i in 0..num_rows {
        // TODO - maybe make the loop iteration faster (should benchmark if this is a bottleneck).
        //
        // It might be faster to iterate over non-null segments of the array, identified something
        // like BitSliceIter instead of checking whether each row is null. It might also be faster
        // to have pre-computed where the type_col is a string and have AND'd this with they key
        // mask.If there are many attributes that need to be checked, it may be more efficient to
        // have pre-computed eq(type_col, Str) and cached it somewhere. Anyway, if we did all this,
        // we could just do iterate over valid ranges from (key & eq(type_col, str) & nulls) and
        // again avoid this per-row check.
        if !key_mask.value(i) || type_col.value(i) != str_type || str_arr.is_null(i) {
            continue;
        }

        let value = str_arr.value(i);
        let scan_result = db.database.scan(value.as_bytes())?;

        // TODO we load all the Id Bitmaps for every row here, which might thrash the CPU cache.
        // It might be nice if there was a way we could fill in individual bitmaps for all rows
        // at the same time instead.
        for pattern_id in scan_result {
            if let Some(pattern_ref) = db.pattern_index.get(pattern_id as usize) {
                policy_id_bitmaps
                    .entry(pattern_ref.policy_index)
                    .or_insert_with(IdBitmap::new)
                    .insert(u32::from(parent_id_col.value(i)));
            }
        }
    }

    Ok(policy_id_bitmaps)
}

/// Scan a dictionary-encoded attribute string column: cache unique value scans,
/// then map back to rows respecting key/type filters.
fn scan_attr_dict_string<K>(
    keys: &arrow::array::PrimitiveArray<K>,
    values: &StringArray,
    key_mask: &BooleanArray,
    type_col: &UInt8Array,
    parent_id_col: &UInt16Array,
    str_type: u8,
    num_rows: usize,
    db: &CompiledDatabase,
) -> Result<HashMap<usize, IdBitmap>, PolicyError>
where
    K: arrow::datatypes::ArrowPrimitiveType,
    K::Native: Into<usize> + Copy,
{
    // TODO: all the same commentary from scan_attr_plain_string (above) related to potential
    // future optimizations also apply to this function.

    // Phase 1: scan unique dict values that appear in qualifying rows.
    let mut scan_cache: HashMap<usize, Vec<u32>> = HashMap::new();
    for i in 0..num_rows {
        if !key_mask.value(i) || type_col.value(i) != str_type || keys.is_null(i) {
            continue;
        }
        let dict_idx: usize = keys.value(i).into();
        if scan_cache.contains_key(&dict_idx) {
            continue;
        }
        let result = if values.is_null(dict_idx) {
            Vec::new()
        } else {
            db.database.scan(values.value(dict_idx).as_bytes())?
        };
        let _ = scan_cache.insert(dict_idx, result);
    }

    // Phase 2: map cached results to parent_ids.
    let mut policy_id_bitmaps: HashMap<usize, IdBitmap> = HashMap::new();
    for i in 0..num_rows {
        if !key_mask.value(i) || type_col.value(i) != str_type || keys.is_null(i) {
            continue;
        }
        let dict_idx: usize = keys.value(i).into();
        let Some(matching_patterns) = scan_cache.get(&dict_idx) else {
            continue;
        };
        for &pattern_id in matching_patterns {
            if let Some(pattern_ref) = db.pattern_index.get(pattern_id as usize) {
                policy_id_bitmaps
                    .entry(pattern_ref.policy_index)
                    .or_insert_with(IdBitmap::new)
                    .insert(u32::from(parent_id_col.value(i)));
            }
        }
    }

    Ok(policy_id_bitmaps)
}
