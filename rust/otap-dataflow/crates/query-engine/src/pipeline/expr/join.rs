// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code used for joining different expression data scopes.
//!
//! As the expression evaluates, we may encounter points that need to join data from different
//! record batches. For example, in an expression like `severity_number + attributes["x"]`, we
//! need to join the root record batch (which contains the severity_number column) with the
//! attributes record batch on root.id == attributes.parent_id, before doing addition.
//!
//! This module contains a helper function called `join` which takes the results of two expression
//! evaluations (left and right), and produces a resulting record batch by joining on the
//! id/parent_id relationship.
//!
//! The schema of the produced record batch will have the following columns:
//! - "left" - the result of the left input expression
//! - "right" - the result of the right input expression
//!
//! As well as optional columns "id", "parent_id", "scope.id" and/or "resource.id", depending on
//! whether the input expression evaluation result contained these columns.
//!
//! Note that in most cases, the joins will produce a result preserving the input order/ID columns
//!  of the left input expression. However, if the left->right relationship is one->many, we
//! produce a result preserving the input order of the right side input, to avoid losing any rows
//! and to also avoid having ambiguity about the result.
//!
//! TODO
//! - currently assumption is made that all IDs are u16, because we don't yet support evaluation on
//!   any OTAP batches that uses u32 IDs. Eventually we'll need to support this, when the engine
//!   behaviour becomes more sophisticated.
//!
use std::rc::Rc;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, Int32Array, RecordBatch, StructArray, UInt16Array};
use arrow::compute::take;
use arrow::datatypes::{DataType, Field, Fields, Schema};
use datafusion::logical_expr::ColumnarValue;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::get_required_struct_array;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::expr::{
    DataScope, LEFT_COLUMN_NAME, PhysicalExprEvalResult, RIGHT_COLUMN_NAME, arg_column_name,
};
use crate::pipeline::planner::AttributesIdentifier;

/// Join the results of two expression evaluations.
///
/// Returns a RecordBatch containing "left" and "right" from the values of the left/right
/// expression evaluation results.
///
/// It preserves the IDs/row order from one of the sides, which will be indicated by the returned
// DataScope. Normally this will be the left side, except in cases where left:right is one:many.
pub fn join<'a>(
    left: &'a PhysicalExprEvalResult,
    right: &'a PhysicalExprEvalResult,
    otap_batch: &'a OtapArrowRecords,
) -> Result<(RecordBatch, Rc<DataScope>)> {
    // handle special case where both sides have same source/row order
    if left.data_scope == right.data_scope {
        let join_result = EqualScopeJoin::default().join(left, right, otap_batch)?;
        return Ok((join_result, left.data_scope.clone()));
    }

    // determine the join strategy from the source of the data
    match (left.data_scope.as_ref(), right.data_scope.as_ref()) {
        (DataScope::Attributes(left_attrs_id, _), DataScope::Attributes(right_attrs_id, _)) => {
            if left_attrs_id == right_attrs_id {
                let join_exec = AttributeToSameAttributeJoin::new();
                let join_result = join_exec.join(left, right, otap_batch)?;
                Ok((join_result, left.data_scope.clone()))
            } else if is_one_to_many(left_attrs_id, right_attrs_id) {
                let join_exec =
                    AttributeToDifferentAttributeReverseJoin::new(*left_attrs_id, *right_attrs_id);
                let join_result = join_exec.join(left, right, otap_batch)?;
                Ok((join_result, right.data_scope.clone()))
            } else {
                let join_exec =
                    AttributeToDifferentAttributeJoin::new(*left_attrs_id, *right_attrs_id);
                let join_result = join_exec.join(left, right, otap_batch)?;
                Ok((join_result, left.data_scope.clone()))
            }
        }
        (DataScope::Root, DataScope::Attributes(attr_id, _)) => {
            let join_exec = RootToAttributesJoin::new(*attr_id);
            let join_result = join_exec.join(left, right, otap_batch)?;
            Ok((join_result, left.data_scope.clone()))
        }
        (DataScope::Attributes(attr_id, _), DataScope::Root) => match attr_id {
            AttributesIdentifier::Root => {
                let join_exec = RootAttrsToRootJoin::new();
                let join_result = join_exec.join(left, right, otap_batch)?;
                Ok((join_result, left.data_scope.clone()))
            }
            AttributesIdentifier::NonRoot(payload_type) => {
                let join_exec = NonRootAttrsToRootReverseJoin::new(*payload_type);
                let join_result = join_exec.join(left, right, otap_batch)?;
                Ok((join_result, right.data_scope.clone()))
            }
        },
        (left, right) => {
            // Note: with expression trees created by our logical expression planner, we shouldn't
            // end up in this error case, non-handled combinations of data scopes don't end up
            // added to expressions in ways that require joins
            Err(Error::ExecutionError {
                cause: format!("Invalid data scopes for join: left {left:?} right {right:?}"),
            })
        }
    }
}

// helper function for determining join order. In most of our join implementations, we build a
// lookup table for the right side, then scan the left. However, if the left:right
// relationship is one:many, we need do the join backwards to avoid ambiguity about which row on
// in the lookup corresponds with the row from the side we're scanning.
//
// one:many relationships include:
// - Resource -> Scope
// - Resource -> Log/Trace/Metric,
// - Scope -> Log/Trace/Metric
//
pub fn is_one_to_many(
    left_attrs_id: &AttributesIdentifier,
    right_attrs_id: &AttributesIdentifier,
) -> bool {
    match (left_attrs_id, right_attrs_id) {
        (AttributesIdentifier::Root, _) => false,
        (AttributesIdentifier::NonRoot(_), AttributesIdentifier::Root) => true,
        (AttributesIdentifier::NonRoot(left), AttributesIdentifier::NonRoot(right)) => {
            *left == ArrowPayloadType::ResourceAttrs && *right == ArrowPayloadType::ScopeAttrs
        }
    }
}

/// Describes how two sides of a join are aligned after computing the join.
enum JoinAlignment {
    /// Both sides have the same data scope and row order; no reordering needed.
    EqualScope,
    /// The left side's row order is preserved; the right side is reordered by the given indices.
    LeftPreserved(Int32Array),
    /// The right side's row order is preserved; the left side is reordered by the given indices.
    RightPreserved(Int32Array),
}

/// Determines the alignment strategy for joining two expression results, returning which side's
/// row order is preserved and the take-indices for reordering the other side.
fn compute_join_alignment(
    left: &PhysicalExprEvalResult,
    right: &PhysicalExprEvalResult,
    otap_batch: &OtapArrowRecords,
) -> Result<(JoinAlignment, Rc<DataScope>)> {
    if left.data_scope == right.data_scope {
        return Ok((JoinAlignment::EqualScope, left.data_scope.clone()));
    }

    // Static scalars can broadcast to any row count without reordering.
    if left.data_scope.is_scalar() {
        return Ok((JoinAlignment::EqualScope, right.data_scope.clone()));
    }
    if right.data_scope.is_scalar() {
        return Ok((JoinAlignment::EqualScope, left.data_scope.clone()));
    }

    match (left.data_scope.as_ref(), right.data_scope.as_ref()) {
        (DataScope::Attributes(left_attrs_id, _), DataScope::Attributes(right_attrs_id, _)) => {
            if left_attrs_id == right_attrs_id {
                let exec = AttributeToSameAttributeJoin::new();
                let indices = exec.rows_to_take(left, right, otap_batch)?;
                Ok((
                    JoinAlignment::LeftPreserved(indices),
                    left.data_scope.clone(),
                ))
            } else if is_one_to_many(left_attrs_id, right_attrs_id) {
                let exec =
                    AttributeToDifferentAttributeReverseJoin::new(*left_attrs_id, *right_attrs_id);
                let indices = exec.rows_to_take(left, right, otap_batch)?;
                Ok((
                    JoinAlignment::RightPreserved(indices),
                    right.data_scope.clone(),
                ))
            } else {
                let exec = AttributeToDifferentAttributeJoin::new(*left_attrs_id, *right_attrs_id);
                let indices = exec.rows_to_take(left, right, otap_batch)?;
                Ok((
                    JoinAlignment::LeftPreserved(indices),
                    left.data_scope.clone(),
                ))
            }
        }
        (DataScope::Root, DataScope::Attributes(attr_id, _)) => {
            let exec = RootToAttributesJoin::new(*attr_id);
            let indices = exec.rows_to_take(left, right, otap_batch)?;
            Ok((
                JoinAlignment::LeftPreserved(indices),
                left.data_scope.clone(),
            ))
        }
        (DataScope::Attributes(attr_id, _), DataScope::Root) => match attr_id {
            AttributesIdentifier::Root => {
                let exec = RootAttrsToRootJoin::new();
                let indices = exec.rows_to_take(left, right, otap_batch)?;
                Ok((
                    JoinAlignment::LeftPreserved(indices),
                    left.data_scope.clone(),
                ))
            }
            AttributesIdentifier::NonRoot(payload_type) => {
                let exec = NonRootAttrsToRootReverseJoin::new(*payload_type);
                let indices = exec.rows_to_take(left, right, otap_batch)?;
                Ok((
                    JoinAlignment::RightPreserved(indices),
                    right.data_scope.clone(),
                ))
            }
        },
        (left, right) => Err(Error::ExecutionError {
            cause: format!("invalid data scopes for join: left {left:?} right {right:?}"),
        }),
    }
}

/// Joins multiple expression evaluation results into a single record batch.
///
/// The resulting record batch contains columns named "arg_0", "arg_1", ..., "arg_{N-1}"
/// (one per input result), plus ID columns (id, parent_id, resource, scope) from whichever
/// side's row order is preserved.
///
/// The implementation works by iteratively joining each new result against the accumulated
/// row order. When a join preserves the left (accumulated) side, the accumulated value arrays
/// keep their order and only the new argument needs reordering. When a reverse join preserves
/// the right (new) side, all previously accumulated value arrays are reordered.
pub fn multi_join(
    results: &[PhysicalExprEvalResult],
    otap_batch: &OtapArrowRecords,
) -> Result<(RecordBatch, Rc<DataScope>)> {
    if results.is_empty() {
        return Err(Error::ExecutionError {
            cause: "multi_join called with no results".into(),
        });
    }

    // If there's only one result, produce a single-column record batch directly.
    // This shouldn't happen in practice (the planner only creates MultiJoin when there
    // are incompatible scopes across 2+ args), but we handle it defensively.
    if results.len() == 1 {
        let result = &results[0];
        let values = result.values.to_array(
            result
                .parent_ids
                .as_ref()
                .map(|a| a.len())
                .or_else(|| result.ids.as_ref().map(|a| a.len()))
                .unwrap_or(1),
        )?;
        let schema = Schema::new(vec![Field::new(
            arg_column_name(0),
            values.data_type().clone(),
            true,
        )]);
        let rb = RecordBatch::try_new(Arc::new(schema), vec![values])?;
        return Ok((rb, result.data_scope.clone()));
    }

    // Start with the first result as the accumulator.
    let first = &results[0];
    let first_values = first.values.to_array(
        first
            .parent_ids
            .as_ref()
            .map(|a| a.len())
            .or_else(|| first.ids.as_ref().map(|a| a.len()))
            .unwrap_or(1),
    )?;
    let mut accumulated_values: Vec<ArrayRef> = vec![first_values];
    let mut accum_scope = first.data_scope.clone();
    let mut accum_ids = first.ids.clone();
    let mut accum_parent_ids = first.parent_ids.clone();
    let mut accum_resource_ids = first.resource_ids.clone();
    let mut accum_scope_ids = first.scope_ids.clone();

    for result in results.iter().skip(1) {
        // Build a temporary PhysicalExprEvalResult for the accumulator to pass to
        // compute_join_alignment. We use the first accumulated value as the "values" field
        // (the alignment only depends on the data scope and ID columns, not the actual values).
        let accum_result = PhysicalExprEvalResult {
            values: ColumnarValue::Array(accumulated_values[0].clone()),
            data_scope: accum_scope.clone(),
            ids: accum_ids.clone(),
            parent_ids: accum_parent_ids.clone(),
            scope_ids: accum_scope_ids.clone(),
            resource_ids: accum_resource_ids.clone(),
        };

        let (alignment, new_scope) = compute_join_alignment(&accum_result, result, otap_batch)?;

        match alignment {
            JoinAlignment::EqualScope => {
                if accum_scope.is_scalar() && !result.data_scope.is_scalar() {
                    // Transitioning from scalar to non-scalar. Adopt the new result's
                    // IDs and re-broadcast any previously accumulated scalar values to
                    // match the new result's row count.
                    let new_len = result
                        .parent_ids
                        .as_ref()
                        .map(|a| a.len())
                        .or_else(|| result.ids.as_ref().map(|a| a.len()))
                        .unwrap_or(1);
                    for arr in accumulated_values.iter_mut() {
                        *arr = ColumnarValue::Scalar(ScalarValue::try_from_array(arr, 0)?)
                            .to_array(new_len)?;
                    }
                    let new_values = result.values.to_array(new_len)?;
                    accumulated_values.push(new_values);
                    accum_ids = result.ids.clone();
                    accum_parent_ids = result.parent_ids.clone();
                    accum_resource_ids = result.resource_ids.clone();
                    accum_scope_ids = result.scope_ids.clone();
                } else {
                    // Same row order - just add the new arg's values directly.
                    let new_values = result.values.to_array(accumulated_values[0].len())?;
                    accumulated_values.push(new_values);
                }
            }
            JoinAlignment::LeftPreserved(right_take_indices) => {
                // Left (accumulator) order preserved. Reorder the new arg's values.
                let new_arr_len = result
                    .parent_ids
                    .as_ref()
                    .map(|a| a.len())
                    .or_else(|| result.ids.as_ref().map(|a| a.len()))
                    .unwrap_or(1);
                let new_values = result.values.to_array(new_arr_len)?;
                let aligned_new = take(&new_values, &right_take_indices, None)?;
                accumulated_values.push(aligned_new);
                // Scope and IDs stay with the accumulator (left side preserved).
            }
            JoinAlignment::RightPreserved(left_take_indices) => {
                // Right (new arg) order preserved. Reorder ALL accumulated values.
                for arr in accumulated_values.iter_mut() {
                    *arr = take(arr, &left_take_indices, None)?;
                }
                // Add the new arg's values directly (right side order is preserved).
                let new_arr_len = result
                    .parent_ids
                    .as_ref()
                    .map(|a| a.len())
                    .or_else(|| result.ids.as_ref().map(|a| a.len()))
                    .unwrap_or(1);
                let new_values = result.values.to_array(new_arr_len)?;
                accumulated_values.push(new_values);
                // Update IDs to the right (new) side since its row order is preserved.
                accum_ids = result.ids.clone();
                accum_parent_ids = result.parent_ids.clone();
                accum_resource_ids = result.resource_ids.clone();
                accum_scope_ids = result.scope_ids.clone();
            }
        }
        accum_scope = new_scope;
    }

    // Build the final RecordBatch with "arg_0", "arg_1", ..., "arg_{N-1}" columns + ID columns.
    let num_args = accumulated_values.len();
    let mut fields = Vec::with_capacity(num_args + 4);
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(num_args + 4);

    for (i, values) in accumulated_values.into_iter().enumerate() {
        fields.push(Field::new(
            arg_column_name(i),
            values.data_type().clone(),
            true,
        ));
        columns.push(values);
    }

    if let Some(ids) = &accum_ids {
        fields.push(Field::new(consts::ID, ids.data_type().clone(), true));
        columns.push(ids.clone());
    }

    if let Some(parent_ids) = &accum_parent_ids {
        fields.push(Field::new(
            consts::PARENT_ID,
            parent_ids.data_type().clone(),
            false,
        ));
        columns.push(parent_ids.clone());
    }

    if let Some(col) = &accum_resource_ids {
        let struct_arr = StructArray::new(
            Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
            vec![col.clone()],
            None,
        );
        fields.push(Field::new(
            consts::RESOURCE,
            struct_arr.data_type().clone(),
            true,
        ));
        columns.push(Arc::new(struct_arr));
    }

    if let Some(col) = &accum_scope_ids {
        let struct_arr = StructArray::new(
            Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
            vec![col.clone()],
            None,
        );
        fields.push(Field::new(
            consts::SCOPE,
            struct_arr.data_type().clone(),
            true,
        ));
        columns.push(Arc::new(struct_arr));
    }

    let record_batch = RecordBatch::try_new(Arc::new(Schema::new(fields)), columns)?;
    Ok((record_batch, accum_scope))
}

// helper functions for producing errors from results. Normally, we wouldn't produce these errors
// unless we received invalid input record batches with missing ID/parent_id columns or batches
// where these records are an unexpected type

fn missing_column_err(column_name: &str) -> Error {
    Error::ExecutionError {
        cause: format!("Invalid record batch: missing required column {column_name}"),
    }
}

fn invalid_column_type_error(data_type: &DataType) -> Error {
    Error::ExecutionError {
        cause: format!("Invalid record batch. Expected u16 ID column, found {data_type:?}"),
    }
}

/// Helper function to extract and downcast a UInt16Array from an optional ArrayRef
fn extract_u16_array<'a>(
    array_ref: Option<&'a ArrayRef>,
    column_name: &str,
) -> Result<&'a UInt16Array> {
    let array = array_ref.ok_or_else(|| missing_column_err(column_name))?;
    array
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or_else(|| invalid_column_type_error(array.data_type()))
}

/// Helper function to perform a simple left-to-right join using parent IDs
/// Builds a lookup from right_ids, scans left_ids, and creates a take array
fn build_simple_join_indices(left_ids: &UInt16Array, right_lookup: &IdJoinLookup) -> Int32Array {
    let mut to_take = Int32Array::builder(left_ids.len());

    left_ids.iter().for_each(|id| {
        if let Some(left_id) = id {
            let right_index = right_lookup.lookup(left_id).map(|i| i as i32);
            to_take.append_option(right_index);
        } else {
            to_take.append_null();
        }
    });

    to_take.finish()
}

/// Helper function to build join indices for two-hop joins through an intermediate lookup
/// Used when joining attributes from different scopes via the root record batch as an
/// intermediary (e.g., resource attrs + scope attrs)
fn build_two_hop_join_indices(
    left_ids: &UInt16Array,
    intermediate_lookup: &IdJoinLookup,
    right_root_ids: &UInt16Array,
    right_lookup: &IdJoinLookup,
) -> Int32Array {
    let mut to_take = Int32Array::builder(left_ids.len());

    left_ids.iter().for_each(|left_id_opt| {
        let index = left_id_opt
            .and_then(|id| intermediate_lookup.lookup(id))
            .filter(|&i| right_root_ids.is_valid(i))
            .map(|i| right_root_ids.value(i))
            .and_then(|id| right_lookup.lookup(id))
            .map(|i| i as i32);

        match index {
            Some(i) => to_take.append_value(i),
            None => to_take.append_null(),
        }
    });

    to_take.finish()
}

/// Helper function to extract id column from root batch based on AttributesIdentifier.
fn get_attrs_id_values<'a>(
    root_batch: &'a RecordBatch,
    attrs_id: &'a AttributesIdentifier,
) -> Result<&'a UInt16Array> {
    match attrs_id {
        AttributesIdentifier::Root => {
            let id_col = root_batch
                .column_by_name(consts::ID)
                .ok_or_else(|| missing_column_err(consts::ID))?;
            Ok(id_col
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or_else(|| invalid_column_type_error(id_col.data_type()))?)
        }
        AttributesIdentifier::NonRoot(payload_type) => {
            match payload_type {
                ArrowPayloadType::ResourceAttrs => {
                    let resource_struct = get_required_struct_array(root_batch, consts::RESOURCE)
                        .map_err(|e| Error::ExecutionError {
                        cause: format!("Failed to get resource struct: {e}"),
                    })?;
                    let id_col = resource_struct
                        .column_by_name(consts::ID)
                        .ok_or_else(|| missing_column_err(consts::ID))?;
                    Ok(id_col
                        .as_any()
                        .downcast_ref::<UInt16Array>()
                        .ok_or_else(|| invalid_column_type_error(id_col.data_type()))?)
                }
                ArrowPayloadType::ScopeAttrs => {
                    let scope_struct = get_required_struct_array(root_batch, consts::SCOPE)
                        .map_err(|e| Error::ExecutionError {
                            cause: format!("Failed to get scope struct: {e}"),
                        })?;
                    let id_col = scope_struct
                        .column_by_name(consts::ID)
                        .ok_or_else(|| missing_column_err(consts::ID))?;
                    Ok(id_col
                        .as_any()
                        .downcast_ref::<UInt16Array>()
                        .ok_or_else(|| invalid_column_type_error(id_col.data_type()))?)
                }
                _ => Err(Error::ExecutionError {
                    cause: "Unsupported attribute type".to_string(),
                }),
            }
        }
    }
}

/// Produce a record batch from the result of a join
fn to_join_result(left: &PhysicalExprEvalResult, right_col: ArrayRef) -> Result<RecordBatch> {
    // pre-allocate with enough capacity for left/right plus Id columns
    let mut columns = Vec::with_capacity(5);
    let mut fields = Vec::with_capacity(5);

    let left_values = left.values.to_array(right_col.len())?;
    fields.push(Field::new(
        LEFT_COLUMN_NAME,
        left_values.data_type().clone(),
        true,
    ));
    columns.push(left_values.clone());

    fields.push(Field::new(
        RIGHT_COLUMN_NAME,
        right_col.data_type().clone(),
        true,
    ));
    columns.push(right_col);

    if let Some(ids) = &left.ids {
        fields.push(Field::new(consts::ID, ids.data_type().clone(), true));
        columns.push(ids.clone());
    }

    if let Some(parent_ids) = &left.parent_ids {
        fields.push(Field::new(
            consts::PARENT_ID,
            parent_ids.data_type().clone(),
            false,
        ));
        columns.push(parent_ids.clone());
    }

    if let Some(col) = &left.resource_ids {
        let struct_arr = StructArray::new(
            Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
            vec![col.clone()],
            None,
        );
        fields.push(Field::new(
            consts::RESOURCE,
            struct_arr.data_type().clone(),
            true,
        ));
        columns.push(Arc::new(struct_arr));
    }

    if let Some(col) = &left.scope_ids {
        let struct_arr = StructArray::new(
            Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
            vec![col.clone()],
            None,
        );
        fields.push(Field::new(
            consts::SCOPE,
            struct_arr.data_type().clone(),
            true,
        ));
        columns.push(Arc::new(struct_arr));
    }

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}

pub trait JoinExec {
    /// produce the rows that should be taken
    fn rows_to_take(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Int32Array>;

    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch>;
}

// Join strategies ...

/// When both sides of the join come from the same "data scope", meaning that they select
/// the same rows from the same record batch in the same order, we use this specialized "join"
/// which just selects the values columns from both sides with no reordering
#[derive(Default)]
struct EqualScopeJoin {}

impl JoinExec for EqualScopeJoin {
    fn rows_to_take(
        &self,
        _left: &PhysicalExprEvalResult,
        _right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<Int32Array> {
        // Not implemented - if joining two results with the same row order, can just append
        // the valued directly into a vec of columns to form a joined record batch.
        Err(Error::ExecutionError {
            cause: "rows_to_take not implemented for EqualScopeJoin".into(),
        })
    }

    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let right_values = match &right.values {
            ColumnarValue::Array(arr) => Arc::clone(arr),
            ColumnarValue::Scalar(_) => {
                // Note: given how the current expression planner works, this shouldn't happen
                // because scalars exprs should always be included directly in the expression of
                // one side without any need for join
                return Err(Error::ExecutionError {
                    cause: "JoinExec expected right-side values to be Array but found Scalar"
                        .into(),
                });
            }
        };

        to_join_result(left, right_values)
    }
}

/// Joins root record batch (logs/metrics/traces) to a child attributes record batch
/// on root.id == attributes.parent_id
pub struct RootToAttributesJoin {
    attrs_id: AttributesIdentifier,
}

impl RootToAttributesJoin {
    pub fn new(attrs_id: AttributesIdentifier) -> Self {
        Self { attrs_id }
    }
}

impl JoinExec for RootToAttributesJoin {
    fn rows_to_take(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<Int32Array> {
        // build the lookup for the right side of the join by parent ID
        let right_parent_ids = extract_u16_array(right.parent_ids.as_ref(), consts::PARENT_ID)?;
        let right_lookup = IdJoinLookup::new(right_parent_ids);

        // get the ID column for which we should scan for join
        let left_id_col = match self.attrs_id {
            AttributesIdentifier::Root => &left.ids,
            AttributesIdentifier::NonRoot(payload_type) => match payload_type {
                ArrowPayloadType::ResourceAttrs => &left.resource_ids,
                ArrowPayloadType::ScopeAttrs => &left.scope_ids,
                other => {
                    return Err(Error::ExecutionError {
                        cause: format!(
                            "RootToAttributesJoin received invalid attrs payload type {other:?}"
                        ),
                    });
                }
            },
        };
        let left_parent_ids = extract_u16_array(left_id_col.as_ref(), consts::ID)?;

        Ok(build_simple_join_indices(left_parent_ids, &right_lookup))
    }

    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let right_parent_ids = extract_u16_array(right.parent_ids.as_ref(), consts::PARENT_ID)?;
        let to_take = self.rows_to_take(left, right, otap_batch)?;
        let right_values = right.values.to_array(right_parent_ids.len())?;
        let joined_arr = take(&right_values, &to_take, None)?;

        to_join_result(left, joined_arr)
    }
}

/// Joins root attributes (e.g. log.attributes, span.attributes, or metric.attributes) to the root
/// record batch on root.id == attributes.parent_id
pub(crate) struct RootAttrsToRootJoin {}

impl RootAttrsToRootJoin {
    pub fn new() -> Self {
        Self {}
    }
}

impl JoinExec for RootAttrsToRootJoin {
    fn rows_to_take(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<Int32Array> {
        // build the lookup for the right side of the join by ID column
        let right_ids = extract_u16_array(right.ids.as_ref(), consts::ID)?;
        let right_lookup = IdJoinLookup::new(right_ids);

        // scan the parent_ID column from the attributes to determine which rows from the
        // right values should be taken
        let left_parent_ids = extract_u16_array(left.parent_ids.as_ref(), consts::PARENT_ID)?;

        Ok(build_simple_join_indices(left_parent_ids, &right_lookup))
    }

    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let right_ids = extract_u16_array(right.ids.as_ref(), consts::ID)?;
        let to_take = self.rows_to_take(left, right, otap_batch)?;
        let right_values = right.values.to_array(right_ids.len())?;
        let joined_arr = take(&right_values, &to_take, None)?;

        to_join_result(left, joined_arr)
    }
}

/// Joins non-root attributes (e.g. scope.attributes or resource.attributes) to the root.
///
/// This join uses the left side of the join as the lookup, and produces a result by scanning the
/// right side over the lookup, creating a result where the row order/ID columns of the right side
/// (in this case, the root batch) are preserved.
///
/// We do this because resource/scope -> log/span/metric is a one -> many relationship which means,
/// if we had done the join the non-reversed way, scanning the left would result in ambiguity about
/// which row to take from the right side, and we'd also lose rows from the right side as a result.
struct NonRootAttrsToRootReverseJoin {
    attrs_payload_type: ArrowPayloadType,
}

impl NonRootAttrsToRootReverseJoin {
    fn new(attrs_payload_type: ArrowPayloadType) -> Self {
        Self { attrs_payload_type }
    }
}

impl JoinExec for NonRootAttrsToRootReverseJoin {
    /// produce the rows that should be taken
    fn rows_to_take(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<Int32Array> {
        // build a lookup of ID to index for the left side
        let left_parent_ids = extract_u16_array(left.parent_ids.as_ref(), consts::PARENT_ID)?;
        let left_lookup = IdJoinLookup::new(left_parent_ids);

        let right_ids = match self.attrs_payload_type {
            ArrowPayloadType::ResourceAttrs => right.resource_ids.as_ref(),
            ArrowPayloadType::ScopeAttrs => right.scope_ids.as_ref(),
            other => {
                return Err(Error::ExecutionError {
                    cause: format!(
                        "NonRootAttrsToRootReverseJoin received invalid attrs id payload type {other:?}"
                    ),
                });
            }
        };

        let right_ids = right_ids.ok_or_else(|| missing_column_err(consts::ID))?;
        let right_ids = right_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(right_ids.data_type()))?;

        // map right-side IDs to indices to take from left side values
        Ok(Int32Array::from_iter(right_ids.iter().map(|id| {
            id.and_then(|id| left_lookup.lookup(id).map(|i| i as i32))
        })))
    }

    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        // pre-allocate with enough space for right/left + ID columns
        let mut fields = Vec::with_capacity(5);
        let mut columns = Vec::with_capacity(5);

        if let Some(col) = &right.ids {
            fields.push(Field::new(consts::ID, col.data_type().clone(), true));
            columns.push(col.clone());
        }

        if let Some(col) = &right.resource_ids {
            let struct_arr = StructArray::new(
                Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
                vec![col.clone()],
                None,
            );
            fields.push(Field::new(
                consts::RESOURCE,
                struct_arr.data_type().clone(),
                true,
            ));
            columns.push(Arc::new(struct_arr));
        }

        if let Some(col) = &right.scope_ids {
            let struct_arr = StructArray::new(
                Fields::from(vec![Field::new(consts::ID, col.data_type().clone(), true)]),
                vec![col.clone()],
                None,
            );
            fields.push(Field::new(
                consts::SCOPE,
                struct_arr.data_type().clone(),
                true,
            ));
            columns.push(Arc::new(struct_arr));
        }

        let left_parent_ids = extract_u16_array(left.parent_ids.as_ref(), consts::PARENT_ID)?;
        let to_take = self.rows_to_take(left, right, otap_batch)?;
        let left_values = left.values.to_array(left_parent_ids.len())?;
        let joined_vals = take(&left_values, &to_take, None)?;
        fields.push(Field::new(
            LEFT_COLUMN_NAME,
            joined_vals.data_type().clone(),
            true,
        ));
        columns.push(joined_vals);

        let right_ids = match self.attrs_payload_type {
            ArrowPayloadType::ResourceAttrs => right.resource_ids.as_ref(),
            ArrowPayloadType::ScopeAttrs => right.scope_ids.as_ref(),
            other => {
                return Err(Error::ExecutionError {
                    cause: format!(
                        "NonRootAttrsToRootReverseJoin received invalid attrs id payload type {other:?}"
                    ),
                });
            }
        };

        let right_ids = right_ids.ok_or_else(|| missing_column_err(consts::ID))?;
        let right_ids = right_ids
            .as_any()
            .downcast_ref::<UInt16Array>()
            .ok_or_else(|| invalid_column_type_error(right_ids.data_type()))?;
        let child_col = right.values.to_array(right_ids.len())?;
        fields.push(Field::new(
            RIGHT_COLUMN_NAME,
            child_col.data_type().clone(),
            true,
        ));
        columns.push(child_col);

        Ok(RecordBatch::try_new(
            Arc::new(Schema::new(fields)),
            columns,
        )?)
    }
}

/// Join attributes that come from the same payload type on attrs.parent_id = attrs.parent_id.
///
/// Note, the reason we need to join these is because we may have a different set of rows selected
/// on either side. This would happen for expressions like `attributes["x"] + attributes["y"]`.
pub(crate) struct AttributeToSameAttributeJoin {}

impl AttributeToSameAttributeJoin {
    pub fn new() -> Self {
        Self {}
    }
}

impl JoinExec for AttributeToSameAttributeJoin {
    fn rows_to_take(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        _otap_batch: &OtapArrowRecords,
    ) -> Result<Int32Array> {
        // build a mapping of right-side parent_ids to right-side indices
        let right_parent_ids = extract_u16_array(right.parent_ids.as_ref(), consts::PARENT_ID)?;
        let right_lookup = IdJoinLookup::new(right_parent_ids);

        // determine which rows to take from the right side values
        let left_parent_ids = extract_u16_array(left.parent_ids.as_ref(), consts::PARENT_ID)?;

        Ok(build_simple_join_indices(left_parent_ids, &right_lookup))
    }

    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let to_take = self.rows_to_take(left, right, otap_batch)?;

        // take right side values and produce result
        let right_parent_ids = extract_u16_array(right.parent_ids.as_ref(), consts::PARENT_ID)?;
        let right_values = right.values.to_array(right_parent_ids.len())?;
        let joined_arr = take(&right_values, &to_take, None)?;
        to_join_result(left, joined_arr)
    }
}

/// Join rows from one set of attributes to another set that came from a different attributes
/// record batch. For example, left side may be root attributes and right side may be resource
/// attributes.
///
/// In this case, the parent_ids on both sides do not reference the same column on the root, so
/// we need to do the join using another intermediate lookup built using ID columns from the root.
///
/// i.e. join on left.parent_id == root.id and right.parent_id = root.<scope/resource>.id
///
pub(crate) struct AttributeToDifferentAttributeJoin {
    left: AttributesIdentifier,
    right: AttributesIdentifier,
}

impl AttributeToDifferentAttributeJoin {
    pub fn new(left: AttributesIdentifier, right: AttributesIdentifier) -> Self {
        Self { left, right }
    }
}

impl JoinExec for AttributeToDifferentAttributeJoin {
    fn rows_to_take(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Int32Array> {
        // build mapping of the right side parent_id to right side index
        let right_parent_ids = extract_u16_array(right.parent_ids.as_ref(), consts::PARENT_ID)?;
        let right_lookup = IdJoinLookup::new(right_parent_ids);

        // get root batch and extract the id columns we need
        let root_batch = otap_batch
            .root_record_batch()
            .ok_or_else(|| Error::ExecutionError {
                cause: "Missing root record batch".to_string(),
            })?;

        let left_root_ids = get_attrs_id_values(root_batch, &self.left)?;
        let right_root_ids = get_attrs_id_values(root_batch, &self.right)?;

        // build mapping from left root id -> root index to use as bridge
        let inter_join_lookup = IdJoinLookup::new(left_root_ids);

        // determine indices of right side values to take
        let left_parent_ids = extract_u16_array(left.parent_ids.as_ref(), consts::PARENT_ID)?;

        Ok(build_two_hop_join_indices(
            left_parent_ids,
            &inter_join_lookup,
            right_root_ids,
            &right_lookup,
        ))
    }

    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let right_parent_ids = extract_u16_array(right.parent_ids.as_ref(), consts::PARENT_ID)?;
        let to_take = self.rows_to_take(left, right, otap_batch)?;

        let right_values = right.values.to_array(right_parent_ids.len())?;
        let joined_arr = take(&right_values, &to_take, None)?;

        to_join_result(left, joined_arr)
    }
}

/// This performs the same kind of join as [`AttributeToDifferentAttributeJoin`] where we must use
/// root ID columns as an intermediary bridge.
///
/// However, the difference with this join strategy is that the order & ID columns of the right
/// side are what is preserved. This would be used if the relationship between left->right is
/// one->many, for example if left side was resource attributes and right side was log attributes.
///
struct AttributeToDifferentAttributeReverseJoin {
    left: AttributesIdentifier,
    right: AttributesIdentifier,
}

impl AttributeToDifferentAttributeReverseJoin {
    fn new(left: AttributesIdentifier, right: AttributesIdentifier) -> Self {
        Self { left, right }
    }
}

impl JoinExec for AttributeToDifferentAttributeReverseJoin {
    fn rows_to_take(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Int32Array> {
        // build mapping of the left side parent_id to left side index
        let left_parent_ids = extract_u16_array(left.parent_ids.as_ref(), consts::PARENT_ID)?;
        let left_lookup = IdJoinLookup::new(left_parent_ids);

        // get root batch and extract the id columns we need
        let root_batch = otap_batch
            .root_record_batch()
            .ok_or_else(|| Error::ExecutionError {
                cause: "Missing root record batch".to_string(),
            })?;

        let left_root_ids = get_attrs_id_values(root_batch, &self.left)?;
        let right_root_ids = get_attrs_id_values(root_batch, &self.right)?;

        // build mapping from right root id -> root index to use as bridge
        let inter_join_lookup = IdJoinLookup::new(right_root_ids);

        // determine indices of left side values to take
        let right_parent_ids = extract_u16_array(right.parent_ids.as_ref(), consts::PARENT_ID)?;

        Ok(build_two_hop_join_indices(
            right_parent_ids,
            &inter_join_lookup,
            left_root_ids,
            &left_lookup,
        ))
    }

    fn join(
        &self,
        left: &PhysicalExprEvalResult,
        right: &PhysicalExprEvalResult,
        otap_batch: &OtapArrowRecords,
    ) -> Result<RecordBatch> {
        let mut fields = Vec::with_capacity(3);
        let mut columns = Vec::with_capacity(3);

        fields.push(Field::new(consts::PARENT_ID, DataType::UInt16, false));
        columns.push(
            right
                .parent_ids
                .as_ref()
                .ok_or_else(|| missing_column_err(consts::PARENT_ID))?
                .clone(),
        );

        let left_parent_ids = extract_u16_array(left.parent_ids.as_ref(), consts::PARENT_ID)?;
        let left_values = left.values.to_array(left_parent_ids.len())?;
        let to_take = self.rows_to_take(left, right, otap_batch)?;
        let joined_vals = take(&left_values, &to_take, None)?;
        fields.push(Field::new(
            LEFT_COLUMN_NAME,
            joined_vals.data_type().clone(),
            true,
        ));
        columns.push(joined_vals);

        let right_parent_ids = extract_u16_array(right.parent_ids.as_ref(), consts::PARENT_ID)?;
        let child_col = right.values.to_array(right_parent_ids.len())?;
        fields.push(Field::new(
            RIGHT_COLUMN_NAME,
            child_col.data_type().clone(),
            true,
        ));
        columns.push(child_col);

        Ok(RecordBatch::try_new(
            Arc::new(Schema::new(fields)),
            columns,
        )?)
    }
}

/// Lookup structure for joining two record batches by ID.
///
/// The intention is for this to be a fast lookup of ID -> row.
///
/// The idea here is that we have something like a vector that can be indexed by the ID, however we
/// don't allocate the entire vector. Instead we have allocate it in pages only when there is an ID
/// in some given range.
///
/// This structure provides O(1) inserts and lookups while being memory-efficient for dense ID
/// ranges, while still attempting to avoid allocating a lookup table for the entire ID range if
/// the entire range isn't used.
///
/// The u16 space (0-65535) is divided into 64 pages of 1024 entries each.
///
/// # Memory Layout
/// - Outer array: 64 entries (top 6 bits of u16)
/// - Each page: 1024 entries (bottom 10 bits of u16)
/// - Each page is ~8KB (Option<usize> is 8 bytes on 64-bit systems)
///
/// # Example
/// For ID 5120 (binary: 00010100_00000000):
/// - Outer index: 5120 >> 10 = 5
/// - Inner index: 5120 & 0x3FF = 0
///
// TODO - benchmark performance against HashMap... HashMap could have been used for this as well
// however, this might be slightly faster because we don't need to hash the IDs for every insert
// and lookup. Also, if the join sides are mostly sorted by the ID columns, we'll get good page
// CPU locality as we scan over the IDs.
//
// TODO - eventually this will need to support u32 IDs
struct IdJoinLookup {
    /// Two-level lookup: outer array indexed by top 6 bits, inner pages indexed by bottom 10 bits.
    /// Each page maps parent_id -> row index in the right-side batch.
    lookup: Vec<Option<Box<[Option<usize>; PAGE_SIZE]>>>,
}

const PAGE_SIZE: usize = 1024;
const PAGE_BITS: u16 = 10;
const PAGE_MASK: u16 = 0x3FF; // Bottom 10 bits
const NUM_PAGES: usize = 64; // 2^16 / 2^10 = 2^6 = 64

impl IdJoinLookup {
    /// Creates a new IdJoin from a UInt16Array of parent IDs.
    ///
    /// # Arguments
    /// * `parent_ids` - The parent_id array from the right side of the join
    ///
    /// # Returns
    /// A lookup structure mapping parent_id -> row index. Null values in the array are skipped.
    fn new(ids: &UInt16Array) -> Self {
        let mut lookup: Vec<Option<Box<[Option<usize>; PAGE_SIZE]>>> = vec![None; NUM_PAGES];

        // TODO bench this. There are probably some optimizations that we can make:
        // 1 - have a loop that does no null check if there are no nulls
        // 2 - if there are nulls, we might be able to avoid checking if each row is null by
        //     using BitSliceIter on the null buffer to get ranges of non-null indices.

        for row_idx in 0..ids.len() {
            // Skip null values
            if ids.is_null(row_idx) {
                continue;
            }

            let parent_id = ids.value(row_idx);
            let outer = (parent_id >> PAGE_BITS) as usize;
            let inner = (parent_id & PAGE_MASK) as usize;

            // Allocate page if needed
            if lookup[outer].is_none() {
                lookup[outer] = Some(Box::new([None; PAGE_SIZE]));
            }

            // Store the mapping
            // safety: we've initialized lookup[outer] in the block above, so safe to expect
            lookup[outer].as_mut().expect("allocated")[inner] = Some(row_idx);
        }

        Self { lookup }
    }

    /// Looks up a left-side ID and returns the corresponding right-side row index.
    ///
    /// # Arguments
    /// * `left_id` - The ID value from the left side to look up
    ///
    /// # Returns
    /// * `Some(row_idx)` - The row index in the right batch if a match exists
    /// * `None` - No matching parent_id found
    #[inline]
    fn lookup(&self, left_id: u16) -> Option<usize> {
        let outer = (left_id >> PAGE_BITS) as usize;
        let inner = (left_id & PAGE_MASK) as usize;

        self.lookup[outer].as_ref().and_then(|page| page[inner])
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::Int64Array;
    use otap_df_pdata::otap::Logs;

    fn empty_otap_batch() -> OtapArrowRecords {
        OtapArrowRecords::Logs(Logs::default())
    }

    /// Helper to build a minimal PhysicalExprEvalResult with the given values and scope.
    fn make_result(values: ArrayRef, scope: DataScope) -> PhysicalExprEvalResult {
        PhysicalExprEvalResult {
            values: ColumnarValue::Array(values),
            data_scope: Rc::new(scope),
            ids: None,
            parent_ids: None,
            scope_ids: None,
            resource_ids: None,
        }
    }

    #[test]
    fn test_multi_join_empty_results_returns_error() {
        let otap_batch = empty_otap_batch();
        let err = multi_join(&[], &otap_batch);
        assert!(err.is_err(), "expected error for empty results");
    }

    #[test]
    fn test_multi_join_single_result_returns_single_column() {
        let otap_batch = empty_otap_batch();
        let values: ArrayRef = Arc::new(Int64Array::from(vec![10, 20, 30]));
        let result = make_result(values, DataScope::Root);

        let (rb, scope) = multi_join(&[result], &otap_batch).unwrap();

        assert_eq!(*scope, DataScope::Root);
        assert_eq!(rb.num_columns(), 1);
        assert_eq!(rb.num_rows(), 3);
        assert_eq!(rb.schema().field(0).name(), "arg_0");

        let col = rb.column(0).as_any().downcast_ref::<Int64Array>().unwrap();
        assert_eq!(col.values(), &[10, 20, 30]);
    }
}
