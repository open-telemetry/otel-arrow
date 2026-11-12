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

use arrow::array::{
    Array, ArrayRef, BooleanArray, Datum, Float64Array, Int64Array, StringArray, UInt16Array,
};
use arrow::compute::kernels::cmp;
use arrow::datatypes::DataType;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmap, build_native_selection_vec};
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
        FieldTarget::Attribute(_, _) => {
            todo!("attribute field typed check evaluation with parent_id join-back via IdBitmap")
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

/// Get the root id column that corresponds to a given attribute payload type.
///
/// For log/span/metric attributes, the id is a plain column on the root batch.
/// For resource/scope attributes, the id is nested inside a struct column.
fn get_root_id_column<'a>(
    root_batch: &'a arrow::array::RecordBatch,
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
