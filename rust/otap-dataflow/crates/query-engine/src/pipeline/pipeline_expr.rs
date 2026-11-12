// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public facade for evaluating logical predicates against OTAP batches.
//!
//! [`Predicate`] wraps the internal expression planning and evaluation
//! machinery behind a simple public API. It takes a [`LogicalExpression`]
//! AST node, plans it into an executable form, and evaluates it against
//! OTAP Arrow record batches to produce a [`PredicateResult`].
//!
//! This follows the same facade pattern as [`Partitioner`](super::partition::Partitioner),
//! which wraps scalar expression evaluation for partitioning.

use arrow::array::{Array, BooleanArray};
use arrow::buffer::BooleanBuffer;
use datafusion::common::cast::as_boolean_array;
use datafusion::logical_expr::ColumnarValue;
use datafusion::prelude::SessionContext;
use datafusion::scalar::ScalarValue;
use otel_arrow_contrib_data_engine_expressions::LogicalExpression;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};

use crate::error::{Error, Result};
use crate::pipeline::expr::eval::EvalContext;
use crate::pipeline::expr::planner::ExprPlanner;
use crate::pipeline::expr::{DataScope, RecordScope, ScopedExpr};
use crate::pipeline::filter::align_selection_to_root;
use crate::pipeline::planner::RecordType;
use crate::pipeline::Pipeline;

/// The result of evaluating a [`Predicate`] against an OTAP batch.
///
/// This avoids materializing a full [`BooleanArray`] when the predicate
/// evaluates to a uniform result across all rows.
#[derive(Debug)]
pub enum PredicateResult {
    /// Every row satisfies the predicate.
    All,
    /// No rows satisfy the predicate.
    None,
    /// A subset of rows satisfies the predicate. The [`BooleanArray`] has
    /// one element per row in the root record batch, with `true` indicating
    /// the row satisfies the predicate. Nulls are treated as `false`.
    Array(BooleanArray),
}

/// A compiled logical predicate that can be evaluated against OTAP batches.
///
/// Construct with [`Predicate::try_new`] by passing a [`LogicalExpression`]
/// AST node and a [`RecordType`] indicating which signal context to plan for.
/// Then call [`Predicate::evaluate`] repeatedly on different batches.
///
/// # Example
///
/// ```ignore
/// let expr = /* build a LogicalExpression */;
/// let mut predicate = Predicate::try_new(
///     &expr,
///     RecordType::Signal(SignalContext::Single(SignalKind::Logs)),
/// )?;
/// let result: PredicateResult = predicate.evaluate(&otap_batch)?;
/// ```
pub struct Predicate {
    /// The planned expression tree.
    expr: ScopedExpr,

    /// DataFusion session context for expression evaluation.
    session_ctx: SessionContext,
}

impl Predicate {
    /// Compile a [`LogicalExpression`] into an executable predicate.
    ///
    /// The `record_type` specifies which signal context the expression
    /// will be evaluated against (e.g., logs, metrics, traces).
    pub fn try_new(logical_expr: &LogicalExpression, record_type: RecordType) -> Result<Self> {
        let planner = ExprPlanner::new(true, record_type);
        let scoped_expr = planner.plan_logical(logical_expr, &[])?;

        Ok(Self {
            expr: scoped_expr,
            session_ctx: Pipeline::create_session_context(),
        })
    }

    /// Evaluate the predicate against an OTAP batch.
    ///
    /// Returns a [`PredicateResult`] indicating which rows satisfy the
    /// predicate. Returns [`PredicateResult::None`] if the batch is empty
    /// or has no root record batch.
    pub fn evaluate(&mut self, otap_batch: &OtapArrowRecords) -> Result<PredicateResult> {
        let num_rows = match otap_batch.root_record_batch() {
            Some(rb) => rb.num_rows(),
            None => return Ok(PredicateResult::None),
        };

        let eval_context = EvalContext::new(&self.session_ctx);
        let result = self.expr.execute_as_value(otap_batch, &eval_context)?;

        let columnar_value = match result {
            None => return Ok(PredicateResult::None),
            Some(scoped_value) => {
                if matches!(scoped_value.scope, DataScope::Record(RecordScope::Signal))
                    || matches!(scoped_value.scope, DataScope::RootParent(_))
                    || scoped_value.scope == DataScope::StaticScalar
                {
                    scoped_value.values
                } else {
                    // Attribute-scoped result -- align to root and return as array.
                    let aligned = align_selection_to_root(
                        Some(scoped_value),
                        otap_batch,
                        &eval_context,
                    )?;
                    return Ok(to_predicate_result_from_boolean_array(aligned, num_rows));
                }
            }
        };

        to_predicate_result(columnar_value, num_rows)
    }
}

/// Convert a `ColumnarValue` to a `PredicateResult`, preserving scalar
/// shortcuts to avoid materializing arrays.
fn to_predicate_result(value: ColumnarValue, num_rows: usize) -> Result<PredicateResult> {
    match value {
        ColumnarValue::Scalar(scalar) => match scalar {
            ScalarValue::Boolean(Some(true)) => Ok(PredicateResult::All),
            ScalarValue::Boolean(Some(false))
            | ScalarValue::Boolean(None)
            | ScalarValue::Null => Ok(PredicateResult::None),
            other => Err(Error::ExecutionError {
                cause: format!(
                    "expected boolean scalar for predicate, found {:?}",
                    other.data_type()
                ),
            }),
        },
        ColumnarValue::Array(arr) => {
            let boolean_arr = as_boolean_array(&arr).map_err(|_| Error::ExecutionError {
                cause: format!(
                    "expected boolean array for predicate, found {}",
                    arr.data_type()
                ),
            })?;
            Ok(to_predicate_result_from_boolean_array(
                boolean_arr.clone(),
                num_rows,
            ))
        }
    }
}

/// Convert a `BooleanArray` to a `PredicateResult`, collapsing to
/// `All`/`None` when possible.
fn to_predicate_result_from_boolean_array(
    arr: BooleanArray,
    num_rows: usize,
) -> PredicateResult {
    // Strip nulls: treat null predicate results as false.
    let buffer = match arr.nulls() {
        Some(nulls) => BooleanArray::new(arr.values() & nulls.inner(), None),
        None => arr,
    };

    let true_count = buffer.true_count();
    if true_count == 0 {
        PredicateResult::None
    } else if true_count == num_rows {
        PredicateResult::All
    } else {
        PredicateResult::Array(buffer)
    }
}
