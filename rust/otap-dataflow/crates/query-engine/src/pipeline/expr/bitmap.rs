// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of `execute_as_id_mask` for each `ScopedExpr` variant.
//!
//! This module contains the logic for evaluating a `ScopedExpr` tree and producing an
//! `IdMask` result. This is the efficient path for boolean predicates: the result stays
//! in bitmap space, avoiding unnecessary materialization of intermediate arrays.

use arrow::array::{Array, UInt16Array};
use arrow::util::bit_iterator::BitSliceIterator;
use datafusion::common::cast::as_boolean_array;
use datafusion::logical_expr::ColumnarValue;
use datafusion::prelude::SessionContext;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::filter::IdBitmapPool;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::expr::DataScope;
use crate::pipeline::id_mask::IdMask;

use super::eval::{eval_datafusion_expr_value, invert_id_mask, join_and_eval_value};
use super::{LeafEval, ScopedExpr, ScopedValue};

impl ScopedExpr {
    /// Produce an `IdMask` (bitmap of IDs that matched).
    ///
    /// Used when the consumer only needs membership information — e.g., for row filtering,
    /// or as input to a boolean combination (`BitmapAnd`/`BitmapOr`/`BitmapNot`).
    #[allow(dead_code)]
    pub(crate) fn execute_as_id_mask(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_ctx: &SessionContext,
        pool: &mut IdBitmapPool,
    ) -> Result<IdMask> {
        match self {
            Self::Eval { scope, eval } => {
                execute_eval_as_id_mask(scope, eval, otap_batch, session_ctx, pool)
            }
            Self::JoinAndEval { children, eval } => execute_join_and_eval_as_id_mask(
                children.as_mut_slice(),
                eval,
                otap_batch,
                session_ctx,
                pool,
            ),
            Self::BitmapAnd(left, right) => {
                let left_mask = left.execute_as_id_mask(otap_batch, session_ctx, pool)?;

                // short-circuit: if left is None, the AND result is None regardless of right
                if left_mask == IdMask::None {
                    return Ok(IdMask::None);
                }

                let right_mask = right.execute_as_id_mask(otap_batch, session_ctx, pool)?;
                Ok(left_mask.combine_and(right_mask, pool))
            }
            Self::BitmapOr(left, right) => {
                let left_mask = left.execute_as_id_mask(otap_batch, session_ctx, pool)?;

                // short-circuit: if left is All, the OR result is All regardless of right
                if left_mask == IdMask::All {
                    return Ok(IdMask::All);
                }

                let right_mask = right.execute_as_id_mask(otap_batch, session_ctx, pool)?;
                Ok(left_mask.combine_or(right_mask, pool))
            }
            Self::BitmapNot(child) => {
                let child_mask = child.execute_as_id_mask(otap_batch, session_ctx, pool)?;
                Ok(invert_id_mask(child_mask))
            }
        }
    }
}

/// Execute an `Eval` node as an IdMask.
fn execute_eval_as_id_mask(
    scope: &DataScope,
    eval: &mut LeafEval,
    otap_batch: &OtapArrowRecords,
    session_ctx: &SessionContext,
    pool: &mut IdBitmapPool,
) -> Result<IdMask> {
    match eval {
        LeafEval::BatchPredicate(predicate) => {
            if predicate.evaluate(otap_batch) {
                Ok(IdMask::All)
            } else {
                Ok(IdMask::None)
            }
        }
        LeafEval::DatafusionExpr { .. } => {
            // evaluate as value first, then convert to IdMask
            let value_result = eval_datafusion_expr_value(scope, eval, otap_batch, session_ctx)?;

            match value_result {
                None => Ok(IdMask::None), // missing data, nothing matches
                Some(sv) => scoped_value_to_id_mask(sv, otap_batch, pool),
            }
        }
    }
}

/// Execute a `JoinAndEval` node as an IdMask.
fn execute_join_and_eval_as_id_mask(
    children: &mut [ScopedExpr],
    eval: &mut LeafEval,
    otap_batch: &OtapArrowRecords,
    session_ctx: &SessionContext,
    pool: &mut IdBitmapPool,
) -> Result<IdMask> {
    // JoinAndEval always materializes values (the join requires actual arrays),
    // then converts the boolean result to an IdMask.
    let value_result = join_and_eval_value(children, eval, otap_batch, session_ctx)?;

    match value_result {
        None => Ok(IdMask::None),
        Some(sv) => scoped_value_to_id_mask(sv, otap_batch, pool),
    }
}

/// Convert a `ScopedValue` containing a boolean result into an `IdMask`.
fn scoped_value_to_id_mask(
    sv: ScopedValue,
    otap_batch: &OtapArrowRecords,
    pool: &mut IdBitmapPool,
) -> Result<IdMask> {
    // handle scalar results
    if let ColumnarValue::Scalar(scalar) = &sv.values {
        return match scalar {
            ScalarValue::Boolean(Some(true)) => {
                // For attribute-scoped scalars, "all true" means "all rows that matched
                // the key filter pass", not ALL rows in the batch. We need to build an
                // IdMask from the parent_ids of the matching rows.
                if matches!(sv.scope, DataScope::Attribute(_, _)) {
                    if let Some(parent_ids) = &sv.parent_ids {
                        let parent_id_col = parent_ids
                            .as_any()
                            .downcast_ref::<UInt16Array>()
                            .ok_or_else(|| Error::ExecutionError {
                                cause: format!(
                                    "expected parent_id to be UInt16, found {:?}",
                                    parent_ids.data_type()
                                ),
                            })?;
                        let mut bitmap = pool.acquire();
                        bitmap.populate(parent_id_col.values().iter().map(|pid| *pid as u32));
                        return Ok(IdMask::Some(bitmap));
                    }
                }
                Ok(IdMask::All)
            }
            ScalarValue::Boolean(Some(false)) | ScalarValue::Boolean(None) | ScalarValue::Null => {
                Ok(IdMask::None)
            }
            other => Err(Error::ExecutionError {
                cause: format!(
                    "expected boolean result for IdMask conversion, found {:?}",
                    other.data_type()
                ),
            }),
        };
    }

    // handle array results
    let arr = match &sv.values {
        ColumnarValue::Array(arr) => arr,
        _ => unreachable!("handled scalar case above"),
    };

    let boolean_arr = as_boolean_array(arr).map_err(|_| Error::ExecutionError {
        cause: format!(
            "expected boolean array for IdMask conversion, found {}",
            arr.data_type()
        ),
    })?;

    match &sv.scope {
        DataScope::Root | DataScope::RootParent(_) => {
            // root-scoped: use the root batch's id column to build the IdMask
            let root_rb = otap_batch
                .root_record_batch()
                .ok_or_else(|| Error::ExecutionError {
                    cause: "root batch not present".into(),
                })?;

            let id_col = root_rb
                .column_by_name(consts::ID)
                .and_then(|c| c.as_any().downcast_ref::<UInt16Array>());

            match id_col {
                Some(id_col) => {
                    // check if all or none pass first
                    if boolean_arr.true_count() == boolean_arr.len() {
                        return Ok(IdMask::All);
                    }
                    if boolean_arr.false_count() == boolean_arr.len() {
                        return Ok(IdMask::None);
                    }

                    // create a bitmap of the IDs that pass the filter
                    let bool_values = boolean_arr.values();
                    let mut bitmap = pool.acquire();
                    for (start, end) in
                        BitSliceIterator::new(bool_values.inner().as_slice(), 0, boolean_arr.len())
                    {
                        for idx in start..end {
                            bitmap.insert(id_col.value(idx) as u32);
                        }
                    }
                    Ok(IdMask::Some(bitmap))
                }
                None => {
                    // No explicit id column on root batch — use array index as implicit ID.
                    // This happens for simple batches without attributes or when the ID
                    // column has been omitted.
                    if boolean_arr.true_count() == boolean_arr.len() {
                        Ok(IdMask::All)
                    } else if boolean_arr.false_count() == boolean_arr.len() {
                        Ok(IdMask::None)
                    } else {
                        // This shouldn't happen, unless we somehow got an invalid batch. Basically it 
                        // means we did some filtering on a child batch that was present, but there is
                        // no ID column to join with it
                        Err(Error::InvalidPipelineError { 
                            cause: "materialize_id_mask_to_value expected id column for materializing id bitmap".into(),
                            query_location: None
                        })
                    }
                }
            }
        }
        DataScope::Attribute(_, _) | DataScope::AttributesAll(_) => {
            // attribute-scoped: use parent_ids to populate an IdBitmap
            let parent_ids = sv
                .parent_ids
                .as_ref()
                .ok_or_else(|| Error::ExecutionError {
                    cause: "attribute-scoped result missing parent_id column for IdMask conversion"
                        .into(),
                })?;
            // TODO - eventually we will handle u32 IDs.
            let parent_id_col = parent_ids
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or_else(|| Error::ExecutionError {
                    cause: format!(
                        "expected parent_id to be UInt16, found {:?}",
                        parent_ids.data_type()
                    ),
                })?;

            let bool_values = boolean_arr.values();
            let mut bitmap = pool.acquire();
            for (start, end) in
                BitSliceIterator::new(bool_values.inner().as_slice(), 0, boolean_arr.len())
            {
                for idx in start..end {
                    bitmap.insert(parent_id_col.value(idx) as u32);
                }
            }
            Ok(IdMask::Some(bitmap))
        }
        DataScope::StaticScalar => {
            // scalar-scoped: shouldn't normally happen for arrays, but handle defensively
            if boolean_arr.true_count() == boolean_arr.len() {
                Ok(IdMask::All)
            } else {
                Ok(IdMask::None)
            }
        }
    }
}
