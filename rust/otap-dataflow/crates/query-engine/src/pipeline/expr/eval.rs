// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of `execute_as_value` for each `ScopedExpr` variant.
//!
//! This module contains the logic for evaluating a `ScopedExpr` tree and producing a
//! `ScopedValue` result. Each variant has its own evaluation strategy.
//!
//! The core evaluation functions (`eval_df_expr_value` and `join_and_eval_value`) are also
//! used by the `bitmap` module when it needs to evaluate a node as a value first and then
//! convert the result to an `IdMask`.

use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, AsArray, BooleanArray, BooleanBufferBuilder, RecordBatch, StringArray,
    StructArray, UInt16Array,
};
use arrow::buffer::BooleanBuffer;
use arrow::compute::filter_record_batch;
use arrow::compute::kernels::cmp::eq;
use arrow::datatypes::{Field, Schema};
use datafusion::common::DFSchema;
use datafusion::logical_expr::{ColumnarValue, Expr};
use datafusion::physical_expr::{PhysicalExprRef, create_physical_expr};
use datafusion::prelude::SessionContext;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::{
    get_optional_array_from_struct_array_from_record_batch, get_required_array,
};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::expr::join::{JoinInput, join, multi_join};
use crate::pipeline::expr::{
    DataScope, LeafEval, SCALAR_RECORD_BATCH_INPUT, ScopedExpr, ScopedValue, VALUE_COLUMN_NAME,
};
use crate::pipeline::id_mask::IdMask;
use crate::pipeline::planner::AttributesIdentifier;
use crate::pipeline::project::anyval::{
    find_any_value_columns, project_any_value_columns, stitch_partitioned_results,
};
use crate::pipeline::project::{Projection, ProjectionOptions};
use otap_df_pdata::otap::filter::IdBitmapPool;

impl ScopedExpr {
    /// Produce a full `ScopedValue` (array + scope + IDs).
    ///
    /// Returns `None` when the expression's input data is absent (e.g., missing attributes or
    /// optional columns) propagating nulls.
    ///
    /// Primarily used when the consumer needs actual values — e.g., for assignment to a column,
    /// as input to arithmetic, or as an argument to a function call.
    pub(crate) fn execute_as_value(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
    ) -> Result<Option<ScopedValue>> {
        match self {
            Self::Eval { scope, eval } => {
                eval_datafusion_expr_value(scope, eval, otap_batch, session_ctx)
            }
            Self::JoinAndEval { children, eval } => {
                join_and_eval_value(children.as_mut_slice(), eval, otap_batch, session_ctx)
            }
            Self::BitmapAnd(left, right) => {
                execute_bitmap_and_as_value(left, right, otap_batch, session_ctx)
            }
            Self::BitmapOr(left, right) => {
                execute_bitmap_or_as_value(left, right, otap_batch, session_ctx)
            }
            Self::BitmapNot(child) => execute_bitmap_not_as_value(child, otap_batch, session_ctx),
        }
    }

    /// Evaluate this node directly on the provided `RecordBatch`, ignoring scope resolution.
    ///
    /// Used for nested attribute pipelines where the "root" is the attributes batch itself
    /// (e.g., `logs | apply attributes { set value = value + 2 }`).
    ///
    /// Supports `Eval(DatafusionExpr)` nodes and boolean combination nodes (`BitmapAnd`,
    /// `BitmapOr`, `BitmapNot`) which recursively evaluate their children on the same batch.
    pub(crate) fn evaluate_on_batch(
        &mut self,
        session_ctx: &SessionContext,
        record_batch: &RecordBatch,
    ) -> Result<ColumnarValue> {
        match self {
            Self::Eval {
                eval:
                    LeafEval::DatafusionExpr {
                        logical_expr,
                        physical_expr,
                        ..
                    },
                ..
            } => evaluate_df_expr(logical_expr, physical_expr, session_ctx, record_batch),
            _ => Err(Error::InvalidPipelineError {
                cause: "only Eval(DatafusionExpr) can be evaluated on a provided batch".into(),
                query_location: None,
            }),
        }
    }
}

/// Execute an `Eval` node as a value.
pub(super) fn eval_datafusion_expr_value(
    scope: &DataScope,
    eval: &mut LeafEval,
    otap_batch: &OtapArrowRecords,
    session_ctx: &SessionContext,
) -> Result<Option<ScopedValue>> {
    match eval {
        LeafEval::DatafusionExpr {
            logical_expr,
            physical_expr,
            projection,
            projection_opts,
            eval_anyval_as_struct,
            attr_key_case_sensitive,
            missing_data_passes,
        } => {
            // resolve the source RecordBatch for this scope
            let source_rb = match scope {
                DataScope::Root | DataScope::RootParent(_) => {
                    otap_batch.root_record_batch().map(Cow::Borrowed)
                }
                DataScope::Attribute(attrs_id, key) => {
                    let attrs_payload_type = resolve_attrs_payload_type(attrs_id, otap_batch);
                    otap_batch
                        .get(attrs_payload_type)
                        .map(|rb| project_attrs(rb, key.as_str(), *attr_key_case_sensitive))
                        .transpose()?
                        .flatten()
                        .map(Cow::Owned)
                }
                DataScope::AttributesAll(attrs_id) => {
                    let attrs_payload_type = resolve_attrs_payload_type(attrs_id, otap_batch);
                    otap_batch.get(attrs_payload_type).map(Cow::Borrowed)
                }
                DataScope::StaticScalar => Some(Cow::Borrowed(SCALAR_RECORD_BATCH_INPUT.deref())),
            };

            let source_rb = match source_rb {
                Some(rb) => rb,
                None => {
                    // source data absent. For is_null expressions, missing data means
                    // the field IS null to all rows pass.
                    if *missing_data_passes {
                        return Ok(Some(ScopedValue {
                            values: ColumnarValue::Scalar(ScalarValue::Boolean(Some(true))),
                            scope: scope.clone(), // TODO scope clone here clones key string - should be Rc?
                            ids: None,
                            parent_ids: None,
                        }));
                    }
                    return Ok(None);
                }
            };

            // project the source RecordBatch to match the physical expression's expected schema
            let projected_rb = if *scope != DataScope::StaticScalar {
                match projection.project_with_options(&source_rb, projection_opts)? {
                    Some(projected) => projected,
                    None => {
                        // required columns missing
                        if *missing_data_passes {
                            return Ok(Some(ScopedValue {
                                values: ColumnarValue::Scalar(ScalarValue::Boolean(Some(true))),
                                scope: scope.clone(),
                                ids: None,
                                parent_ids: None,
                            }));
                        }
                        return Ok(None);
                    }
                }
            } else {
                source_rb.as_ref().clone()
            };

            // check for AnyValue struct columns that need resolution
            let any_value_indices = find_any_value_columns(projected_rb.schema_ref());

            let result_vals = if any_value_indices.is_empty() || *eval_anyval_as_struct {
                evaluate_df_expr(logical_expr, physical_expr, session_ctx, &projected_rb)?
            } else {
                evaluate_with_anyval_partitions(
                    logical_expr,
                    physical_expr,
                    projection_opts,
                    session_ctx,
                    &projected_rb,
                    &any_value_indices,
                )?
            };

            Ok(Some(ScopedValue::new(
                coerce_nulls_for_predicate(result_vals, *missing_data_passes),
                scope.clone(),
                &source_rb,
            )))
        }

        LeafEval::BatchPredicate(predicate) => {
            let result = predicate.evaluate(otap_batch);
            Ok(Some(ScopedValue {
                values: ColumnarValue::Scalar(ScalarValue::Boolean(Some(result))),
                // TODO - should this be scalar scope?
                scope: DataScope::Root,
                ids: None,
                parent_ids: None,
            }))
        }
    }
}

/// Execute a `JoinAndEval` node as a value.
///
/// Evaluates all children, joins their results into a single RecordBatch, then evaluates
/// the DataFusion expression on the joined result.
pub(super) fn join_and_eval_value(
    children: &mut [ScopedExpr],
    eval: &mut LeafEval,
    otap_batch: &OtapArrowRecords,
    session_ctx: &SessionContext,
) -> Result<Option<ScopedValue>> {
    // evaluate all children
    let mut child_results = Vec::with_capacity(children.len());
    for child in children.iter_mut() {
        match child.execute_as_value(otap_batch, session_ctx)? {
            Some(result) => child_results.push(result),
            None => return Ok(None), // if any child is absent, the whole expression is null
        }
    }

    // convert ScopedValues to JoinInputs for the join boundary
    let eval_results: Vec<JoinInput> = child_results
        .into_iter()
        .map(|sv| scoped_value_to_join_input(sv, otap_batch))
        .collect::<Result<Vec<_>>>()?;

    // perform the join
    // TODO - is there any reason not to always use multi_join here? It could simplify the call
    let (joined_rb, result_scope) = if eval_results.len() == 2 {
        join(&eval_results[0], &eval_results[1], otap_batch)?
    } else {
        multi_join(&eval_results, otap_batch)?
    };

    // evaluate the expression on the joined RecordBatch
    let LeafEval::DatafusionExpr {
        logical_expr,
        physical_expr,
        projection,
        projection_opts,
        eval_anyval_as_struct,
        attr_key_case_sensitive: _,
        missing_data_passes,
    } = eval
    else {
        // TODO - technically we could evaluate the batch predicate w/out joining. It would be
        // correct, although it would also be a weird planner error if we ended up here w/out this
        // variant of LeafEval
        return Err(Error::ExecutionError {
            cause: "JoinAndEval requires a DatafusionExpr leaf evaluation".into(),
        });
    };

    // project the joined batch
    let projected_rb = match projection.project_with_options(&joined_rb, projection_opts)? {
        Some(projected) => projected,
        None => return Ok(None),
    };

    // handle AnyValue columns
    let any_value_indices = find_any_value_columns(projected_rb.schema_ref());
    let result_vals = if any_value_indices.is_empty() || *eval_anyval_as_struct {
        evaluate_df_expr(logical_expr, physical_expr, session_ctx, &projected_rb)?
    } else {
        evaluate_with_anyval_partitions(
            logical_expr,
            physical_expr,
            projection_opts,
            session_ctx,
            &projected_rb,
            &any_value_indices,
        )?
    };

    Ok(Some(ScopedValue::new(
        coerce_nulls_for_predicate(result_vals, *missing_data_passes),
        DataScope::clone(&result_scope),
        &joined_rb,
    )))
}

fn coerce_nulls_for_predicate(
    result_vals: ColumnarValue,
    missing_data_passes: bool,
) -> ColumnarValue {
    match &result_vals {
        ColumnarValue::Array(arr) => {
            if let Some(boolean_arr) = arr.as_boolean_opt() {
                if let Some(nulls) = boolean_arr.nulls() {
                    let combined = if missing_data_passes {
                        boolean_arr.values().clone()
                    } else {
                        boolean_arr.values() & nulls.inner()
                    };

                    return ColumnarValue::Array(Arc::new(BooleanArray::new(combined, None)));
                }
            }
        }
        ColumnarValue::Scalar(ScalarValue::Boolean(None)) => {
            return ColumnarValue::Scalar(ScalarValue::Boolean(Some(missing_data_passes)));
        }
        _ => {}
    }

    result_vals
}

/// Execute a `BitmapAnd` node as a value.
///
/// Evaluates both children as IdMasks (staying in bitmap space), intersects them, then
/// materializes the combined result to a root-aligned BooleanArray once. This avoids
/// aligning each child independently which would do duplicate parent_id → root mappings.
fn execute_bitmap_and_as_value(
    left: &mut Box<ScopedExpr>,
    right: &mut Box<ScopedExpr>,
    otap_batch: &OtapArrowRecords,
    session_ctx: &SessionContext,
) -> Result<Option<ScopedValue>> {
    let mut pool = IdBitmapPool::new();
    let left_mask = left.execute_as_id_mask(otap_batch, session_ctx, &mut pool)?;

    // short-circuit: if left is all-false, skip right
    if left_mask == IdMask::None {
        return materialize_id_mask_to_value(IdMask::None, otap_batch);
    }

    let right_mask = right.execute_as_id_mask(otap_batch, session_ctx, &mut pool)?;
    let combined = left_mask.combine_and(right_mask, &mut pool);
    materialize_id_mask_to_value(combined, otap_batch)
}

/// Execute a `BitmapOr` node as a value.
///
/// Same strategy as `BitmapAnd` — stays in IdMask space and materializes once at the end.
fn execute_bitmap_or_as_value(
    left: &mut Box<ScopedExpr>,
    right: &mut Box<ScopedExpr>,
    otap_batch: &OtapArrowRecords,
    session_ctx: &SessionContext,
) -> Result<Option<ScopedValue>> {
    let mut pool = IdBitmapPool::new();
    let left_mask = left.execute_as_id_mask(otap_batch, session_ctx, &mut pool)?;

    // short-circuit: if left is all-true, skip right
    if left_mask == IdMask::All {
        return materialize_id_mask_to_value(IdMask::All, otap_batch);
    }

    let right_mask = right.execute_as_id_mask(otap_batch, session_ctx, &mut pool)?;
    let combined = left_mask.combine_or(right_mask, &mut pool);
    materialize_id_mask_to_value(combined, otap_batch)
}

/// Execute a `BitmapNot` node as a value.
fn execute_bitmap_not_as_value(
    child: &mut Box<ScopedExpr>,
    otap_batch: &OtapArrowRecords,
    session_ctx: &SessionContext,
) -> Result<Option<ScopedValue>> {
    let mut pool = IdBitmapPool::new();
    let child_mask = child.execute_as_id_mask(otap_batch, session_ctx, &mut pool)?;
    let inverted = invert_id_mask(child_mask);
    materialize_id_mask_to_value(inverted, otap_batch)
}

/// Evaluate a DataFusion expression on a RecordBatch.
fn evaluate_df_expr(
    logical_expr: &Expr,
    physical_expr: &mut Option<PhysicalExprRef>,
    session_ctx: &SessionContext,
    record_batch: &RecordBatch,
) -> Result<ColumnarValue> {
    if physical_expr.is_none() {
        // lazily plan the physical expression
        let session_state = session_ctx.state();
        let df_schema = DFSchema::try_from(record_batch.schema_ref().as_ref().clone())?;
        let expr = create_physical_expr(logical_expr, &df_schema, session_state.execution_props())?;
        *physical_expr = Some(expr);
    }

    // SAFETY: physical_expr is initialized above if it was None
    let result = physical_expr
        .as_ref()
        .expect("physical expr initialized above")
        .evaluate(record_batch)?;

    Ok(result)
}

/// Evaluate a DataFusion expression on a RecordBatch that has AnyValue struct columns,
/// using the split-evaluate-stitch pattern where each value type is evaluated as a separate
/// partition and the results are then stitched back together.
fn evaluate_with_anyval_partitions(
    logical_expr: &Expr,
    physical_expr: &mut Option<PhysicalExprRef>,
    projection_opts: &ProjectionOptions,
    session_ctx: &SessionContext,
    projected_rb: &RecordBatch,
    any_value_indices: &[usize],
) -> Result<ColumnarValue> {
    let partitions = project_any_value_columns(projected_rb, any_value_indices)?;

    if partitions.len() == 1 {
        let partition = partitions.into_iter().next().expect("non-empty");
        let batch = maybe_downcast_dicts(partition.batch, projection_opts)?;
        evaluate_df_expr(logical_expr, physical_expr, session_ctx, &batch)
    } else {
        let total_rows = projected_rb.num_rows();
        let mut partition_results = Vec::with_capacity(partitions.len());

        for partition in partitions {
            let batch = maybe_downcast_dicts(partition.batch, projection_opts)?;
            let result = evaluate_df_expr(logical_expr, physical_expr, session_ctx, &batch)?;
            let result_arr = result.into_array(batch.num_rows())?;
            partition_results.push((result_arr, partition.original_row_ranges));
        }

        let stitched = stitch_partitioned_results(partition_results, total_rows)?;
        Ok(ColumnarValue::Array(stitched))
    }
}

/// Resolve the `ArrowPayloadType` for an attribute scope identifier.
pub(crate) fn resolve_attrs_payload_type(
    attrs_id: &AttributesIdentifier,
    otap_batch: &OtapArrowRecords,
) -> ArrowPayloadType {
    match *attrs_id {
        AttributesIdentifier::Root => match otap_batch.root_payload_type() {
            ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
            ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
            _ => ArrowPayloadType::MetricAttrs,
        },
        AttributesIdentifier::NonRoot(payload_type) => payload_type,
    }
}

/// Convert a `ScopedValue` into a `JoinInput` for use with the join module.
///
/// For root-scoped values, this looks up `scope_ids` and `resource_ids` from the root batch.
/// For other scopes, these fields are left as `None`.
pub(crate) fn scoped_value_to_join_input(
    sv: ScopedValue,
    otap_batch: &OtapArrowRecords,
) -> Result<JoinInput> {
    let is_root = sv.scope == DataScope::Root;

    let mut result = JoinInput {
        values: sv.values,
        data_scope: Rc::new(sv.scope),
        ids: sv.ids,
        parent_ids: sv.parent_ids,
        scope_ids: None,
        resource_ids: None,
    };

    // for root-scoped results, extract scope_ids and resource_ids from the root batch
    if is_root {
        if let Some(root_rb) = otap_batch.root_record_batch() {
            if let Ok(Some(resource_ids)) = get_optional_array_from_struct_array_from_record_batch(
                root_rb,
                consts::RESOURCE,
                consts::ID,
            ) {
                result.resource_ids = Some(Arc::clone(resource_ids));
            }

            if let Ok(Some(scope_ids)) = get_optional_array_from_struct_array_from_record_batch(
                root_rb,
                consts::SCOPE,
                consts::ID,
            ) {
                result.scope_ids = Some(Arc::clone(scope_ids));
            }
        }
    }

    Ok(result)
}

/// Invert an `IdMask`: `All` <-> `None`, `Some` <-> `NotSome`.
pub(super) fn invert_id_mask(mask: IdMask) -> IdMask {
    match mask {
        IdMask::All => IdMask::None,
        IdMask::None => IdMask::All,
        IdMask::Some(bm) => IdMask::NotSome(bm),
        IdMask::NotSome(bm) => IdMask::Some(bm),
    }
}

/// Materialize an `IdMask` to a root-aligned `BooleanArray` wrapped in a `ScopedValue`.
fn materialize_id_mask_to_value(
    mask: IdMask,
    otap_batch: &OtapArrowRecords,
) -> Result<Option<ScopedValue>> {
    let root_rb = match otap_batch.root_record_batch() {
        Some(rb) => rb,
        None => return Ok(None),
    };

    let num_rows = root_rb.num_rows();

    let boolean_arr = match &mask {
        IdMask::All => BooleanArray::new(BooleanBuffer::new_set(num_rows), None),
        IdMask::None => BooleanArray::new(BooleanBuffer::new_unset(num_rows), None),
        IdMask::Some(_) | IdMask::NotSome(_) => {
            let id_col = root_rb
                .column_by_name(consts::ID)
                .and_then(|c| c.as_any().downcast_ref::<UInt16Array>());

            match id_col {
                Some(id_col) => {
                    // For NotSome masks, a null ID means "no attributes exist for this row",
                    // which means the row is NOT in the bitmap — so it passes.
                    // For Some masks, null ID means no match.
                    let null_id_passes = matches!(mask, IdMask::NotSome(_));

                    let mut builder = BooleanBufferBuilder::new(num_rows);
                    let mut segment_val = false;
                    let mut segment_len = 0usize;

                    for idx in 0..id_col.len() {
                        let row_val = if id_col.is_valid(idx) {
                            mask.contains(id_col.value(idx) as u32)
                        } else {
                            null_id_passes
                        };

                        if segment_val != row_val {
                            if segment_len > 0 {
                                builder.append_n(segment_len, segment_val);
                            }
                            segment_val = row_val;
                            segment_len = 0;
                        }
                        segment_len += 1;
                    }
                    if segment_len > 0 {
                        builder.append_n(segment_len, segment_val);
                    }

                    BooleanArray::new(builder.finish(), None)
                }
                None => {
                    // No explicit id column on root — use array index as implicit ID.
                    // TODO - need to re-validate why this works?
                    let mut builder = BooleanBufferBuilder::new(num_rows);
                    let mut segment_val = false;
                    let mut segment_len = 0usize;

                    for idx in 0..num_rows {
                        let row_val = mask.contains(idx as u32);

                        if segment_val != row_val {
                            if segment_len > 0 {
                                builder.append_n(segment_len, segment_val);
                            }
                            segment_val = row_val;
                            segment_len = 0;
                        }
                        segment_len += 1;
                    }
                    if segment_len > 0 {
                        builder.append_n(segment_len, segment_val);
                    }

                    BooleanArray::new(builder.finish(), None)
                }
            }
        }
    };

    Ok(Some(ScopedValue::new(
        ColumnarValue::Array(Arc::new(boolean_arr)),
        DataScope::Root,
        root_rb,
    )))
}

/// Filter an attributes RecordBatch by key and project it into the canonical
/// `[parent_id, value]` shape expected by downstream evaluation.
///
/// For example, if we had an input batch like:
/// key:        ["a", "a", "b", "b"]
/// type:       [1, 1, 1, 1] // type 1 = str
/// str:        ["x", "x", "y", "z"]
/// parent_id:  [0, 1, 0, 1]
///
/// If the "key" argument to this function was "b", the result would be:
/// parent_id: [0, 1]
/// value:     Struct { type: [1, 1], str: ["y", "z"], ... }  (tagged as AnyValue)
///
/// When `case_sensitive` is false, the key comparison is case-insensitive.
fn project_attrs(
    record_batch: &RecordBatch,
    key: &str,
    case_sensitive: bool,
) -> Result<Option<RecordBatch>> {
    // Get the key column and create a mask for rows matching the specified key
    let key_col = get_required_array(record_batch, consts::ATTRIBUTE_KEY).map_err(|e| {
        Error::ExecutionError {
            cause: e.to_string(),
        }
    })?;

    let key_mask = if case_sensitive {
        eq(key_col, &StringArray::new_scalar(key))?
    } else {
        build_case_insensitive_mask(key_col, &key.to_lowercase())?
    };

    let filtered_batch = filter_record_batch(record_batch, &key_mask)?;

    if filtered_batch.num_rows() == 0 {
        return Ok(None);
    }

    // Build the parent_id column
    let parent_id_col = filtered_batch
        .column_by_name(consts::PARENT_ID)
        .cloned()
        .ok_or_else(|| Error::ExecutionError {
            cause: "invalid attributes record batch: missing parent_id column".into(),
        })?;

    // Build the AnyValue struct from the type + value sub-columns
    let any_value_struct = build_any_value_struct(&filtered_batch)?;

    let mut fields: Vec<Arc<Field>> = Vec::with_capacity(2);
    let mut columns: Vec<ArrayRef> = Vec::with_capacity(2);

    fields.push(Arc::new(Field::new(
        consts::PARENT_ID,
        parent_id_col.data_type().clone(),
        false,
    )));
    columns.push(parent_id_col);

    fields.push(Arc::new(Field::new(
        VALUE_COLUMN_NAME,
        any_value_struct.data_type().clone(),
        true,
    )));
    columns.push(Arc::new(any_value_struct));

    let schema = Arc::new(Schema::new(fields));
    let projected_batch = RecordBatch::try_new(schema, columns)?;

    Ok(Some(projected_batch))
}

/// Build a `BooleanArray` mask for case-insensitive string matching against a key column.
/// Supports both plain `StringArray` and `DictionaryArray<UInt8, Utf8>`.
fn build_case_insensitive_mask(key_col: &dyn Array, key_lower: &str) -> Result<BooleanArray> {
    use arrow::array::{AsArray, StringArray};
    use arrow::datatypes::DataType;

    match key_col.data_type() {
        DataType::Utf8 => {
            let key_arr = key_col
                .as_any()
                .downcast_ref::<StringArray>()
                .expect("data type is Utf8, downcast to StringArray must succeed");
            Ok(BooleanArray::from_iter(key_arr.iter().map(|opt_val| {
                opt_val.map(|v| v.to_lowercase() == *key_lower)
            })))
        }
        DataType::Dictionary(_, value_type) if matches!(value_type.as_ref(), DataType::Utf8) => {
            let dict_arr = key_col
                .as_any()
                .downcast_ref::<arrow::array::DictionaryArray<arrow::datatypes::UInt8Type>>()
                .ok_or_else(|| Error::ExecutionError {
                    cause: format!(
                        "expected Dict<UInt8, Utf8> for attribute key column, found {:?}",
                        key_col.data_type()
                    ),
                })?;

            let values = dict_arr.values().as_string::<i32>();
            let keys = dict_arr.keys();

            Ok(BooleanArray::from_iter((0..dict_arr.len()).map(|i| {
                if dict_arr.is_valid(i) {
                    let key_idx = keys.value(i) as usize;
                    Some(values.value(key_idx).to_lowercase() == *key_lower)
                } else {
                    Some(false)
                }
            })))
        }
        other => Err(Error::ExecutionError {
            cause: format!("expected string-like type for attribute key column, found {other:?}"),
        }),
    }
}

/// Collect the `type` discriminant and all present value sub-columns from an attributes
/// record batch into a single [`StructArray`].
///
/// The columns included are: `type` (required), plus whichever of `str`, `int`, `double`,
/// `bool`, `bytes`, `ser` are present in the batch.
fn build_any_value_struct(filtered_batch: &RecordBatch) -> Result<StructArray> {
    let mut struct_fields: Vec<Arc<Field>> = Vec::new();
    let mut struct_columns: Vec<ArrayRef> = Vec::new();

    // The type column is required
    let type_col = get_required_array(filtered_batch, consts::ATTRIBUTE_TYPE).map_err(|e| {
        Error::ExecutionError {
            cause: e.to_string(),
        }
    })?;
    struct_fields.push(Arc::new(Field::new(
        consts::ATTRIBUTE_TYPE,
        type_col.data_type().clone(),
        false,
    )));
    struct_columns.push(type_col.clone());

    // Collect whichever value sub-columns are present
    let value_col_names = [
        consts::ATTRIBUTE_STR,
        consts::ATTRIBUTE_INT,
        consts::ATTRIBUTE_DOUBLE,
        consts::ATTRIBUTE_BOOL,
        consts::ATTRIBUTE_BYTES,
        consts::ATTRIBUTE_SER,
    ];

    for col_name in value_col_names {
        if let Some(col) = filtered_batch.column_by_name(col_name) {
            struct_fields.push(Arc::new(Field::new(
                col_name,
                col.data_type().clone(),
                true,
            )));
            struct_columns.push(col.clone());
        }
    }

    StructArray::try_new(struct_fields.into(), struct_columns, None).map_err(|e| {
        Error::ExecutionError {
            cause: format!("failed to build AnyValue struct: {e}"),
        }
    })
}

/// Apply dictionary downcasting to a RecordBatch if `downcast_dicts` option is enabled.
fn maybe_downcast_dicts(batch: RecordBatch, opts: &ProjectionOptions) -> Result<RecordBatch> {
    if !opts.downcast_dicts {
        return Ok(batch);
    }

    let schema = batch.schema();
    let mut fields: Vec<Arc<Field>> = schema.fields().iter().cloned().collect();
    let mut columns: Vec<ArrayRef> = batch.columns().to_vec();

    Projection::try_downcast_dicts(&mut fields, &mut columns)?;

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}
