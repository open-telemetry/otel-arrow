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

// TODO: non-exhaustive list of ways to make this code better:
//
// - I'm convinced there is a way for attributes that we can cache the attribute we select with
//   some given key. Currently every time an attribute is accessed, we scan the key column and then
//   possibly filter some other columns with the same key.
//
//

use arrow::array::{
    Array, ArrayRef, BooleanArray, Datum, Float64Array, Int64Array, RecordBatch, StringArray,
    UInt16Array, UInt8Array,
};
use arrow::compute::kernels::cmp;
use arrow::datatypes::DataType;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmap, build_native_selection_vec};
use otel_arrow_dfe_pdata::otlp::attributes::AttributeValueType;
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use policy_rs::engine::{
    CompiledNumericValue, CompiledTypedMatcher, CompiledValue, ExistenceCheck, LogSignal,
    TypedCheck,
};

use crate::engine::policy_match::PolicyMatch;
use crate::engine::transform::{FieldTarget, classify_field};

/// Evaluate a [`TypedCheck`] against an OTAP batch, returning a [`PolicyMatch`] indicating which
/// rows satisfy the check.
///
/// Handles negation internally: if `check.is_negated`, the result is inverted before returning.
pub(crate) fn evaluate_typed_check(
    otap_batch: &OtapArrowRecords,
    check: &TypedCheck<LogSignal>,
) -> PolicyMatch {
    let Some(target) = classify_field(&check.field) else {
        // Unsupported field (e.g., nested attribute path) -- non-match.
        return PolicyMatch::None;
    };

    let result = match target {
        FieldTarget::Attribute(key, payload_type) => {
            evaluate_typed_check_on_attribute(otap_batch, key, payload_type, &check.matcher)
        }
        FieldTarget::SimpleField(column_name) => {
            evaluate_typed_check_on_root(otap_batch, column_name, &check.matcher)
        }
    };

    // TODO - we could potentially avoid allocating a buffer if internally we were to swap the
    // predicates instead of just evaluating the operation specified by the matcher and then
    // inverting the result.
    if check.is_negated {
        PolicyMatch::not(&result)
    } else {
        result
    }
}

/// Evaluate an [`ExistenceCheck`] against an OTAP batch, returning a
/// [`PolicyMatch`] indicating which rows satisfy the check.
///
/// Handles negation internally.
pub(crate) fn evaluate_existence_check(
    otap_batch: &OtapArrowRecords,
    check: &ExistenceCheck<LogSignal>,
) -> PolicyMatch {
    let Some(target) = classify_field(&check.field) else {
        return PolicyMatch::None;
    };

    let result = match target {
        FieldTarget::Attribute(key, payload_type) => {
            evaluate_existence_on_attribute(otap_batch, key, payload_type, check.should_exist)
        }
        FieldTarget::SimpleField(column_name) => {
            evaluate_existence_on_root(otap_batch, column_name, check.should_exist)
        }
    };

    // TODO - we could potentially avoid allocating a buffer if internally we were to swap the
    // predicates instead of just evaluating the operation specified by the matcher and then
    // inverting the result.
    if check.is_negated {
        PolicyMatch::not(&result)
    } else {
        result
    }
}

/// Evaluate a typed comparison on a root record batch column.
///
/// Per spec: type mismatches are non-matches (not errors). Integer and
/// floating-point values are compared in a single numeric domain.
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
        CompiledTypedMatcher::Gt(nv) => evaluate_numberic_cmp(column, nv, cmp::gt),
        CompiledTypedMatcher::Gte(nv) => evaluate_numberic_cmp(column, nv, cmp::gt_eq),
        CompiledTypedMatcher::Lt(nv) => evaluate_numberic_cmp(column, nv, cmp::lt),
        CompiledTypedMatcher::Lte(nv) => evaluate_numberic_cmp(column, nv, cmp::lt_eq),
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
            evaluate_numberic_cmp(column, &CompiledNumericValue::Int(*i), cmp::eq)
        }
        CompiledValue::Double(d) => {
            evaluate_numberic_cmp(column, &CompiledNumericValue::Double(*d), cmp::eq)
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

fn evaluate_numberic_cmp<F>(
    column: &ArrayRef,
    numberic_value: &CompiledNumericValue,
    cmp_fn: F,
) -> Option<BooleanArray>
where
    F: Fn(&dyn Datum, &dyn Datum) -> Result<BooleanArray, arrow::error::ArrowError> + Copy,
{
    // TODO the logic below is NOT correct -- we need to have better logic around data type
    // coercion and not just panicking when an invalid type encountered
    match numberic_value {
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
                    // TODO bunch of other numberic data types
                    todo!("numberic coercion")
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
                    // TODO bunch of other numberic data types
                    todo!("numberic coercion")
                }
            }
        }
    }
}

fn effective_data_type(column: &ArrayRef) -> DataType {
    // TODO try to avoid cloning anything not primitive here
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
    let Some(root_batch) = otap_batch.root_record_batch() else {
        return if should_exist {
            PolicyMatch::None
        } else {
            PolicyMatch::All
        };
    };

    let num_rows = root_batch.num_rows();

    let Some(column) = root_batch.column_by_name(column_name) else {
        // Column missing entirely from the batch.
        return if should_exist {
            PolicyMatch::None
        } else {
            PolicyMatch::All
        };
    };

    // TODO - the code below is quite verbose, it could be shortened and it needs to take into
    // account that if the column is an "AnyValue" struct, we may need to go into the type
    // column to determine if, for example, log "body" exists.

    let null_count = column.null_count();
    if should_exist {
        // "field exists" = "field is not null"
        if null_count == 0 {
            PolicyMatch::All
        } else if null_count == num_rows {
            PolicyMatch::None
        } else {
            // The validity bitmap has true where the value is present.
            match column.nulls() {
                Some(nulls) => {
                    PolicyMatch::from_boolean_array(&BooleanArray::new(nulls.inner().clone(), None))
                }
                None => PolicyMatch::All,
            }
        }
    } else {
        // "field does not exist" = "field is null"
        if null_count == num_rows {
            PolicyMatch::All
        } else if null_count == 0 {
            PolicyMatch::None
        } else {
            match column.nulls() {
                Some(nulls) => {
                    let inverted = !nulls.inner();
                    PolicyMatch::from_boolean_array(&BooleanArray::new(inverted, None))
                }
                None => PolicyMatch::None,
            }
        }
    }
}

/// Evaluate a field existence check on an attribute child batch.
///
/// For `should_exist: true`: returns true for root rows that have an attribute
/// with the given key. For `should_exist: false`: returns true for root rows
/// that do NOT have the attribute.
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

    // Get the attribute batch. If it doesn't exist, no rows have any
    // attributes of this type.
    let Some(attrs_batch) = otap_batch.get(payload_type) else {
        return no_attrs_result;
    };

    // Get the key column and find rows matching the target key.
    let Some(key_col) = attrs_batch.column_by_name(consts::ATTRIBUTE_KEY) else {
        return no_attrs_result;
    };

    let key_scalar = StringArray::new_scalar(attr_key);
    let Ok(key_mask) = cmp::eq(key_col, &key_scalar) else {
        // Comparison failed (unexpected type) -- non-match.
        return PolicyMatch::None;
    };

    if key_mask.true_count() == 0 {
        // No rows match the key.
        return no_attrs_result;
    }

    // Get the parent_id column from the attribute batch.
    let Some(parent_id_col) = attrs_batch.column_by_name(consts::PARENT_ID) else {
        return PolicyMatch::None;
    };
    let Some(parent_id_u16) = parent_id_col.as_any().downcast_ref::<UInt16Array>() else {
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

    // TODO - we could change the impl of build_native_selection_vec to possibly avoid
    // a heap allocation/drop for the Arc containing the BoolArray values.
    let selection = build_native_selection_vec(root_id_col, &id_bitmap);
    let result = PolicyMatch::from_boolean_array(&selection);

    // TODO - build the inverted bitmap above instead of inverting this here
    // For should_exist: false, invert the result.
    if should_exist {
        result
    } else {
        PolicyMatch::not(&result)
    }
}

/// Evaluate a typed comparison on an attribute child batch.
///
/// Filters the attribute batch by key, checks the type column for compatible
/// types, applies the comparison on the typed value column, and joins the
/// result back to root rows via parent_id/id.
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
        return PolicyMatch::None;
    };
    let key_scalar = StringArray::new_scalar(attr_key);
    let Ok(key_mask) = cmp::eq(key_col, &key_scalar) else {
        return PolicyMatch::None;
    };
    if key_mask.true_count() == 0 {
        return PolicyMatch::None;
    }

    // Get the parent_id column.
    let Some(parent_id_col) = attrs_batch.column_by_name(consts::PARENT_ID) else {
        return PolicyMatch::None;
    };
    let Some(parent_id_u16) = parent_id_col.as_any().downcast_ref::<UInt16Array>() else {
        return PolicyMatch::None;
    };

    // Get the type column to check value type compatibility.
    let Some(type_col) = attrs_batch.column_by_name(consts::ATTRIBUTE_TYPE) else {
        return PolicyMatch::None;
    };
    let Some(type_u8) = type_col.as_any().downcast_ref::<UInt8Array>() else {
        return PolicyMatch::None;
    };

    // Determine which attribute value types and columns are relevant for
    // this matcher, then attempt the comparison on each.
    // TODO: use an IdBitmapPool for reuse between evaluations.
    let mut id_bitmap = IdBitmap::new();

    match matcher {
        CompiledTypedMatcher::Equals(CompiledValue::Bool(val)) => {
            collect_matching_attr_rows(
                attrs_batch, &key_mask, type_u8, parent_id_u16,
                consts::ATTRIBUTE_BOOL, AttributeValueType::Bool as u8,
                &BooleanArray::new_scalar(*val), cmp::eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Equals(CompiledValue::Int(val)) => {
            collect_matching_attr_rows_numeric(
                attrs_batch, &key_mask, type_u8, parent_id_u16,
                &CompiledNumericValue::Int(*val), cmp::eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Equals(CompiledValue::Double(val)) => {
            collect_matching_attr_rows_numeric(
                attrs_batch, &key_mask, type_u8, parent_id_u16,
                &CompiledNumericValue::Double(*val), cmp::eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Equals(CompiledValue::Bytes(_)) => {
            todo!("Equals(Bytes) comparison on attribute values")
        }
        CompiledTypedMatcher::Gt(nv) => {
            collect_matching_attr_rows_numeric(
                attrs_batch, &key_mask, type_u8, parent_id_u16, nv, cmp::gt,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Gte(nv) => {
            collect_matching_attr_rows_numeric(
                attrs_batch, &key_mask, type_u8, parent_id_u16, nv, cmp::gt_eq,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Lt(nv) => {
            collect_matching_attr_rows_numeric(
                attrs_batch, &key_mask, type_u8, parent_id_u16, nv, cmp::lt,
                &mut id_bitmap,
            );
        }
        CompiledTypedMatcher::Lte(nv) => {
            collect_matching_attr_rows_numeric(
                attrs_batch, &key_mask, type_u8, parent_id_u16, nv, cmp::lt_eq,
                &mut id_bitmap,
            );
        }
    };

    if id_bitmap.is_empty() {
        return PolicyMatch::None;
    }

    let selection = build_native_selection_vec(root_id_col, &id_bitmap);
    PolicyMatch::from_boolean_array(&selection)
}

/// Compare attribute values in a single typed column against a scalar, and
/// insert the parent_ids of matching rows into the `id_bitmap`.
///
/// Only considers rows where `key_mask` is true and `type_col` matches
/// `expected_type`.
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

    // Compare the entire value column against the scalar.
    // The kernel handles dictionary-encoded columns transparently.
    //
    // TODO - this will actually return an error for data types that were'd coerced properly
    // which means comparisons that should pass, but just aren't supported correctly, will silently
    // reject the rows. Consider proper error handling.
    let Ok(cmp_result) = cmp_fn(value_col, scalar) else {
        return;
    };

    // Scan: row must match key AND have the right type AND pass the comparison.
    for i in 0..key_mask.len() {
        if key_mask.value(i)
            && type_col.value(i) == expected_type
            && cmp_result.value(i)
        {
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
    // TODO - There are other coercions that probably need to be supported here...

    match numeric_value {
        CompiledNumericValue::Int(v) => {
            // Check the int column directly.
            collect_matching_attr_rows(
                attrs_batch, key_mask, type_col, parent_id_col,
                consts::ATTRIBUTE_INT, AttributeValueType::Int as u8,
                &Int64Array::new_scalar(*v), cmp_fn, id_bitmap,
            );
            // Cross-numeric: also check the double column with promoted value.
            collect_matching_attr_rows(
                attrs_batch, key_mask, type_col, parent_id_col,
                consts::ATTRIBUTE_DOUBLE, AttributeValueType::Double as u8,
                &Float64Array::new_scalar(*v as f64), cmp_fn, id_bitmap,
            );
        }
        CompiledNumericValue::Double(v) => {
            // Check the double column directly.
            collect_matching_attr_rows(
                attrs_batch, key_mask, type_col, parent_id_col,
                consts::ATTRIBUTE_DOUBLE, AttributeValueType::Double as u8,
                &Float64Array::new_scalar(*v), cmp_fn, id_bitmap,
            );
            // Cross-numeric: also check the int column if the double is exact.
            if v.fract() == 0.0 && *v >= i64::MIN as f64 && *v <= i64::MAX as f64 {
                collect_matching_attr_rows(
                    attrs_batch, key_mask, type_col, parent_id_col,
                    consts::ATTRIBUTE_INT, AttributeValueType::Int as u8,
                    &Int64Array::new_scalar(*v as i64), cmp_fn, id_bitmap,
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
