// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmark helpers for comparing predicate evaluation strategies.
//!
//! This module is gated behind the `bench` feature and exposes internal machinery
//! so that benchmarks can measure just the boolean array production without the
//! overhead of row filtering or column assignment.

use arrow::array::{Array, BooleanArray, BooleanBufferBuilder, UInt16Array};
use arrow::buffer::BooleanBuffer;
use arrow::datatypes::UInt16Type;
use data_engine_expressions::{LogicalExpression, PipelineFunction, ScalarExpression};
use datafusion::common::cast::as_boolean_array;
use datafusion::execution::context::SessionContext;
use datafusion::logical_expr::ColumnarValue;
use otap_df_pdata::arrays::MaybeDictArrayAccessor;
use otap_df_pdata::otap::filter::{ChildBatchFilterIdHelper, IdBitmapPool};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::OtapArrowRecords;

use crate::error::{Error, Result};
use crate::pipeline::expr::{
    DataScope, ExprLogicalPlanner, ExprPhysicalPlanner, ScopedPhysicalExpr,
};
use crate::pipeline::filter::{Composite, FilterExec, FilterPlan};
use crate::pipeline::planner::AttributesIdentifier;
use crate::pipeline::Pipeline;

/// Evaluates a logical expression as a filter predicate, producing a `BooleanArray`
/// aligned to the root batch.
///
/// This uses the filter planning and execution infrastructure
/// (`Composite<FilterPlan>` -> `Composite<FilterExec>`), which includes optimized fast paths
/// for column-vs-literal and attribute comparisons.
pub struct FilterPredicateEvaluator {
    filter_exec: Composite<FilterExec>,
    session_ctx: SessionContext,
    id_bitmap_pool: IdBitmapPool,
}

impl FilterPredicateEvaluator {
    /// Plan and initialize the filter evaluator.
    ///
    /// Performs a warmup execution so that lazy physical expression planning is completed
    /// before benchmarking.
    pub fn setup(
        logical_expr: &LogicalExpression,
        functions: &[PipelineFunction],
        otap_batch: &OtapArrowRecords,
    ) -> Result<Self> {
        let filter_plan = Composite::<FilterPlan>::try_from(logical_expr, true, functions)?;
        let session_ctx = Pipeline::create_session_context();
        let mut filter_exec = filter_plan.to_exec(&session_ctx, otap_batch)?;

        // warmup to trigger lazy physical expr planning
        let mut pool = IdBitmapPool::new();
        let _ = filter_exec.execute(otap_batch, &session_ctx, &mut pool)?;

        Ok(Self {
            filter_exec,
            session_ctx,
            id_bitmap_pool: pool,
        })
    }

    /// Evaluate the predicate and return the boolean selection vector aligned to root.
    pub fn evaluate(&mut self, otap_batch: &OtapArrowRecords) -> Result<BooleanArray> {
        self.filter_exec
            .execute(otap_batch, &self.session_ctx, &mut self.id_bitmap_pool)
    }
}

/// Evaluates a logical expression via the general expression evaluation system,
/// producing a `BooleanArray` aligned to the root batch.
///
/// This uses the expression planner (`ExprLogicalPlanner` -> `ScopedPhysicalExpr`),
/// which evaluates the expression through DataFusion's physical expression framework.
/// When the result is scoped to an attribute batch, it is aligned back to the root batch
/// so that the output is comparable to the filter path.
pub struct ExprPredicateEvaluator {
    physical_expr: ScopedPhysicalExpr,
    session_ctx: SessionContext,
}

impl ExprPredicateEvaluator {
    /// Plan and initialize the expression evaluator.
    ///
    /// The `scalar_expression` should be a `ScalarExpression::Logical(...)` wrapping the
    /// logical expression to evaluate.
    ///
    /// Performs a warmup execution so that lazy physical expression planning is completed
    /// before benchmarking.
    pub fn setup(
        scalar_expression: &ScalarExpression,
        functions: &[PipelineFunction],
        otap_batch: &OtapArrowRecords,
    ) -> Result<Self> {
        let logical_planner = ExprLogicalPlanner::default();
        let logical_expr = logical_planner.plan_scalar_expr(scalar_expression, functions)?;
        let physical_planner = ExprPhysicalPlanner::default();
        let mut physical_expr = physical_planner.plan(logical_expr)?;
        let session_ctx = Pipeline::create_session_context();

        // warmup to trigger lazy physical expr planning
        let _ = physical_expr.execute(otap_batch, &session_ctx)?;

        Ok(Self {
            physical_expr,
            session_ctx,
        })
    }

    /// Evaluate the expression and return the boolean result array aligned to root.
    ///
    /// If the expression result is scoped to an attribute batch, the result is realigned
    /// to the root batch using parent_id mapping, matching how the filter path produces
    /// its output.
    pub fn evaluate(&mut self, otap_batch: &OtapArrowRecords) -> Result<Option<BooleanArray>> {
        let result = self.physical_expr.execute(otap_batch, &self.session_ctx)?;
        let eval_result = match result {
            Some(r) => r,
            None => return Ok(None),
        };

        let num_root_rows = otap_batch
            .root_record_batch()
            .map(|rb| rb.num_rows())
            .unwrap_or(0);

        // Extract the boolean array from the result
        let boolean_arr = match &eval_result.values {
            ColumnarValue::Array(arr) => as_boolean_array(arr)
                .cloned()
                .map_err(|_| Error::ExecutionError {
                    cause: format!(
                        "expected BooleanArray from logical expression, got {}",
                        arr.data_type()
                    ),
                })?,
            ColumnarValue::Scalar(scalar) => {
                let bool_val = matches!(
                    scalar,
                    datafusion::scalar::ScalarValue::Boolean(Some(true))
                );
                return Ok(Some(BooleanArray::new(
                    if bool_val {
                        BooleanBuffer::new_set(num_root_rows)
                    } else {
                        BooleanBuffer::new_unset(num_root_rows)
                    },
                    None,
                )));
            }
        };

        // Check if we need to align the result to the root batch
        match eval_result.data_scope.as_ref() {
            DataScope::Root | DataScope::StaticScalar => Ok(Some(boolean_arr)),
            DataScope::Attributes(attrs_id, _) => {
                let root_rb = otap_batch
                    .root_record_batch()
                    .ok_or_else(|| Error::ExecutionError {
                        cause: "root batch not present".into(),
                    })?;

                let aligned = align_attrs_result_to_root(
                    &boolean_arr,
                    eval_result
                        .parent_ids
                        .as_ref()
                        .ok_or_else(|| Error::ExecutionError {
                            cause: "missing parent_id column on attribute result".into(),
                        })?,
                    *attrs_id,
                    otap_batch,
                    root_rb,
                )?;
                Ok(Some(aligned))
            }
        }
    }
}

/// Aligns a boolean result from an attributes-scoped expression evaluation back to the
/// root record batch using parent_id -> root.id mapping.
fn align_attrs_result_to_root(
    boolean_arr: &BooleanArray,
    parent_ids: &arrow::array::ArrayRef,
    attrs_id: AttributesIdentifier,
    otap_batch: &OtapArrowRecords,
    root_rb: &arrow::array::RecordBatch,
) -> Result<BooleanArray> {
    let num_rows = root_rb.num_rows();

    let parent_id_col = parent_ids
        .as_any()
        .downcast_ref::<UInt16Array>()
        .ok_or_else(|| Error::ExecutionError {
            cause: format!(
                "expected parent_id to be UInt16, found {:?}",
                parent_ids.data_type()
            ),
        })?;

    let attrs_payload_type = match attrs_id {
        AttributesIdentifier::Root => match otap_batch.root_payload_type() {
            ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
            ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
            _ => ArrowPayloadType::MetricAttrs,
        },
        AttributesIdentifier::NonRoot(payload_type) => payload_type,
    };

    let id_col = match UInt16Type::get_id_col_from_parent(root_rb, attrs_payload_type)? {
        Some(MaybeDictArrayAccessor::Native(id_col)) => id_col,
        Some(_) => {
            return Err(Error::ExecutionError {
                cause: "invalid type for ID column on root batch".into(),
            });
        }
        None => {
            return Ok(BooleanArray::new(BooleanBuffer::new_unset(num_rows), None));
        }
    };

    // Build parent_id -> boolean lookup (0=not seen, 1=seen+false, 2=seen+true)
    let mut id_result: Vec<u8> = vec![0u8; 65536];
    for i in 0..parent_id_col.len() {
        if parent_id_col.is_valid(i) {
            let pid = parent_id_col.value(i) as usize;
            let passes = boolean_arr.value(i);
            if passes {
                id_result[pid] = 2;
            } else if id_result[pid] == 0 {
                id_result[pid] = 1;
            }
        }
    }

    // Map root batch rows through the lookup using segment-based building
    let mut builder = BooleanBufferBuilder::new(num_rows);
    let mut segment_validity = false;
    let mut segment_len = 0usize;

    for index in 0..id_col.len() {
        let row_passes = if id_col.is_valid(index) {
            id_result[id_col.value(index) as usize] == 2
        } else {
            false
        };

        if segment_validity != row_passes {
            if segment_len > 0 {
                builder.append_n(segment_len, segment_validity);
            }
            segment_validity = row_passes;
            segment_len = 0;
        }
        segment_len += 1;
    }
    if segment_len > 0 {
        builder.append_n(segment_len, segment_validity);
    }

    Ok(BooleanArray::new(builder.finish(), None))
}
