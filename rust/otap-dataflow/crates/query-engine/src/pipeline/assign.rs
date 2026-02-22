// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of expression evaluation for OTAP (OpenTelemetry Arrow Protocol) batches.
//!
//! # Architecture Overview
//!
//! This module converts expressions from the AST (defined in data_engine_expressions) into
//! executable code that operates on OTAP batches. The conversion happens in three phases:
//!
//! ## Phase 1: Logical Planning (AST → LogicalDomainExpr)
//! `AssignmentLogicalPlanner::plan_scalar_expr` converts a `ScalarExpression` (from the AST)
//! into a `LogicalDomainExpr` which combines:
//! - A `LogicalDataDomain` - identifies where the data comes from (Root/Attributes/StaticScalar)
//! - A DataFusion `Expr` - the logical expression to evaluate
//! - Optional child expression - for cross-domain operations that can't be combined
//!
//! ## Phase 2: Physical Planning (LogicalDomainExpr → PhysicalDomainExpr)
//! `LogicalDomainExpr::into_physical()` converts to a `PhysicalDomainExpr` which adds:
//! - A `FilterProjection` - identifies which columns are needed from input batches
//! - Lazy `PhysicalExprRef` - created on first execution using the actual batch schema
//!
//! ## Phase 3: Execution (PhysicalDomainExpr → ArrayRef)
//! `PhysicalDomainExpr::execute()` evaluates the expression on an OTAP batch:
//! 1. Gets the appropriate input RecordBatch based on the data domain
//! 2. Projects to only needed columns
//! 3. Lazily creates PhysicalExprRef from logical Expr using the batch schema (cached)
//! 4. Recursively executes child expressions if present
//! 5. Evaluates the physical expression and returns an Arrow ArrayRef
//!
//! # Data Domains
//!
//! OTAP batches contain multiple RecordBatches representing different parts of telemetry:
//! - **Root**: Main batch (e.g., Logs with severity_number, severity_text, etc.)
//! - **Attributes**: Child batches (LogAttrs, ResourceAttrs, ScopeAttrs) with key-value structure
//! - **StaticScalar**: Constant values that don't come from any batch
//!
//! Expressions within the same domain can be combined (e.g., log.severity_number + 1).
//! Cross-domain expressions use a parent-child structure where the child result is joined
//! via a special "child" column reference.
//!
//! # Example Flow
//!
//! For the expression `42 + 10`:
//! 1. Logical Planning: `lit(42) + lit(10)` with StaticScalar domain
//! 2. Physical Planning: Creates FilterProjection, stores logical expr
//! 3. Execution: Creates empty schema batch → converts to PhysicalExprRef → evaluates → ArrayRef
//!
//! # Next Steps for Implementation
//!
//! The main TODO is implementing the `execute()` method match arms (lines ~460-500):
//!
//! ## Case 1: `(Some(input), None)` - Single domain, no child
//! This is the most common case. Simply call:
//! ```ignore
//! Ok(Some(self.physical_expr.as_ref().unwrap().evaluate(&input)?))
//! ```
//! DataFusion's `evaluate()` returns `ColumnarValue` directly, handling scalar expansion automatically.
//!
//! ## Case 2: `(Some(input), Some(child))` - Cross-domain with parent and child
//! 1. Unwrap `child: Option<ColumnarValue>` or error if None
//! 2. Convert child `ColumnarValue` to an array matching `input.num_rows()`:
//!    - If `ColumnarValue::Array`: verify row count matches
//!    - If `ColumnarValue::Scalar`: use `into_array(input.num_rows())`
//! 3. Add child array as column named "child" to input batch
//! 4. Evaluate `self.physical_expr` on the extended batch
//! 5. Return `Ok(Some(result))`
//!
//! ## Case 3: `(None, Some(child))` - Only child has data
//! Already implemented: Returns `Ok(child)` directly.
//!
//! ## Case 4: `(None, None)` - No data
//! Already implemented: Returns `Ok(None)`.
//!
//! ## Also TODO:
//! - Implement `try_project_attrs()` to detect type column and project correct value column
//! - Implement `plan_scalar_expr()` for SourceScalarExpression (column references, attribute access)
//! - Add other math operations (Subtract, Multiply, Divide, etc.)
//!
//! TODO: This should probably be renamed to something like "expressions" module

use std::borrow::Cow;
use std::ops::Deref;
use std::sync::Arc;

use arrow::array::{Array, ArrayRef, NullArray, RecordBatch, StringArray};
use arrow::compute::filter_record_batch;
use arrow::compute::kernels::cmp::eq;
use arrow::datatypes::{Field, Schema};
use data_engine_expressions::{
    BinaryMathematicalScalarExpression, MathScalarExpression, ScalarExpression,
    SetTransformExpression, SourceScalarExpression,
};
use datafusion::common::DFSchema;
use datafusion::execution::context::SessionState;
use datafusion::logical_expr::{BinaryExpr, ColumnarValue, Expr, Operator, col, lit};
use datafusion::physical_expr::{PhysicalExprRef, create_physical_expr};
use datafusion::physical_plan::PhysicalExpr;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::get_required_array;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::project::FilterProjection;
use crate::pipeline::planner::AttributesIdentifier;

/// Column name used when referencing child expression results in cross-domain operations.
/// For example, if Root domain needs to add a value from Attributes domain, the parent
/// expression references the child result via col("child").
const CHILD_COLUMN_NAME: &str = "child";

/// Pipeline stage for expression evaluation and assignment.
/// TODO: Implement PipelineStage trait
pub struct AssignPipelineStage {}

/// Identifies which OTAP RecordBatch domain an expression operates on.
///
/// OTAP batches contain multiple RecordBatches. This enum identifies which batch
/// provides the data for an expression:
/// - Root: The main telemetry batch (Logs/Spans/Metrics)
/// - Attributes: Attribute batches (LogAttrs, ResourceAttrs, etc.) filtered by key
/// - StaticScalar: Constant values that don't come from any batch
#[derive(Debug, PartialEq)]
enum DataDomainId {
    /// Main telemetry batch (e.g., Logs with columns like severity_number, severity_text)
    Root,
    /// Attribute batch identified by AttributesIdentifier and filtered by the String key.
    /// For example, (AttributesIdentifier::Root, "http.method") refers to log attributes
    /// with key="http.method"
    Attributes(AttributesIdentifier, String),
    /// Constant scalar value that doesn't require any input batch
    StaticScalar,
}

/// Wrapper around DataDomainId used during logical planning phase.
/// Provides helper methods for determining if domains can be combined.
struct LogicalDataDomain {
    domain_id: DataDomainId,
}

impl LogicalDataDomain {
    /// Determines if two domains can be combined in a single expression.
    ///
    /// Rules:
    /// - Any domain can combine with StaticScalar (constants)
    /// - Same domains can combine (e.g., Root + Root)
    /// - Different non-scalar domains cannot combine (require parent-child structure)
    fn can_combine(&self, other: &LogicalDataDomain) -> bool {
        if self.is_scalar() || other.is_scalar() {
            return true;
        }

        if self.domain_id == other.domain_id {
            return true;
        }

        return false;
    }

    /// Returns true if this domain represents a static scalar value.
    fn is_scalar(&self) -> bool {
        self.domain_id == DataDomainId::StaticScalar
    }
}

/// Represents an expression during the logical planning phase.
///
/// This combines a DataFusion logical expression with domain information.
/// When expressions span multiple domains (e.g., Root + Attributes), a parent-child
/// structure is used where the parent references the child via col("child").
struct LogicalDomainExpr {
    /// The data domain this expression operates on
    data_domain: LogicalDataDomain,
    /// DataFusion logical expression (e.g., col("severity_number") + lit(1))
    logical_expr: Expr,
    /// Optional child expression for cross-domain operations
    child: Option<Box<LogicalDomainExpr>>,
}

impl LogicalDomainExpr {
    /// Convert this logical domain expression into a physical domain expression.
    ///
    /// This performs Phase 2 of the planning process:
    /// - Creates a FilterProjection to identify required columns
    /// - Recursively converts child expressions
    /// - Returns a PhysicalDomainExpr ready for execution
    ///
    /// The actual PhysicalExprRef is created lazily during execution when the
    /// schema is available.
    fn into_physical(self) -> Result<PhysicalDomainExpr> {
        let projection = FilterProjection::try_new(&self.logical_expr)?;

        let child = match self.child {
            Some(child_expr) => Some(Box::new(child_expr.into_physical()?)),
            None => None,
        };

        Ok(PhysicalDomainExpr {
            data_domain: self.data_domain.domain_id,
            logical_expr: self.logical_expr,
            physical_expr: None,
            projection,
            child,
        })
    }
}

/// Logical planner that converts AST expressions into LogicalDomainExpr.
///
/// This is Phase 1 of the planning process. It walks the expression AST and
/// determines the data domain for each sub-expression, then builds a
/// DataFusion logical expression that can operate on that domain.
struct AssignmentLogicalPlanner {}

impl AssignmentLogicalPlanner {
    /// Plans a set transformation expression.
    /// TODO: Implement this for set operations
    fn plan_set_expr(&mut self, set_expr: &SetTransformExpression) -> Result<()> {
        todo!()
    }

    /// Converts a ScalarExpression (from AST) into a LogicalDomainExpr.
    ///
    /// This is the main entry point for logical planning. It handles:
    /// - Static scalars (constants): Integer, Double, Boolean, String, Null
    /// - Source scalars (data access): TODO - column references, attribute access
    /// - Math operations: Add (implemented), others TODO
    /// - Other expression types: TODO
    ///
    /// For operations that span multiple domains (e.g., Root + Attributes),
    /// this creates a parent-child structure where the parent references the
    /// child result via col("child").
    fn plan_scalar_expr(
        &mut self,
        scalar_expression: &ScalarExpression,
    ) -> Result<LogicalDomainExpr> {
        match scalar_expression {
            ScalarExpression::Source(source_scalar_expr) => {
                // TODO: Implement source scalar planning
                // This will handle column references like log.severity_number,
                // attribute access like attributes["http.method"], etc.
                todo!()
            }
            ScalarExpression::Static(static_scalar_expr) => {
                // Convert static scalar constants to DataFusion literals.
                // All static scalars belong to the StaticScalar domain.
                use data_engine_expressions::StaticScalarExpression as SSE;
                
                let logical_expr = match static_scalar_expr {
                    SSE::Integer(int_expr) => {
                        lit(int_expr.get_value())
                    }
                    SSE::Double(double_expr) => {
                        lit(double_expr.get_value())
                    }
                    SSE::Boolean(bool_expr) => {
                        lit(bool_expr.get_value())
                    }
                    SSE::String(string_expr) => {
                        lit(string_expr.get_value())
                    }
                    SSE::Null(_) => {
                        // Create a null literal of unknown type
                        lit(datafusion::scalar::ScalarValue::Null)
                    }
                    _ => {
                        return Err(Error::ExecutionError {
                            cause: format!(
                                "Unsupported static scalar expression type: {:?}",
                                static_scalar_expr
                            ),
                        });
                    }
                };

                Ok(LogicalDomainExpr {
                    data_domain: LogicalDataDomain {
                        domain_id: DataDomainId::StaticScalar,
                    },
                    logical_expr,
                    child: None,
                })
            }
            ScalarExpression::Math(math_scalar_expr) => match math_scalar_expr {
                MathScalarExpression::Add(binary_math_expr) => {
                    // Recursively plan left and right sub-expressions
                    let left = self.plan_scalar_expr(binary_math_expr.get_left_expression())?;
                    let right = self.plan_scalar_expr(binary_math_expr.get_right_expression())?;

                    // Check if both sides operate on compatible domains
                    if left.data_domain.can_combine(&right.data_domain) {
                        // Same domain or one is scalar - can combine into single expression
                        let data_domain = if left.data_domain.is_scalar() {
                            left.data_domain
                        } else {
                            right.data_domain
                        };
                        Ok(LogicalDomainExpr {
                            data_domain,
                            logical_expr: Expr::BinaryExpr(BinaryExpr::new(
                                Box::new(left.logical_expr),
                                Operator::Plus,
                                Box::new(right.logical_expr),
                            )),
                            child: None,
                        })
                    } else {
                        // Different domains - use parent-child structure.
                        // Parent operates on left domain and references child via col("child")
                        Ok(LogicalDomainExpr {
                            data_domain: left.data_domain,
                            logical_expr: Expr::BinaryExpr(BinaryExpr::new(
                                Box::new(left.logical_expr),
                                Operator::Plus,
                                Box::new(col(CHILD_COLUMN_NAME)),
                            )),
                            child: Some(Box::new(right)),
                        })
                    }
                }
                _ => {
                    todo!("other math")
                }
            },
            _ => {
                todo!("handle other scalar expressions or return error")
            }
        }
    }
}

/// Physical planner that converts LogicalDomainExpr into PhysicalDomainExpr.
///
/// This is a thin wrapper that delegates to LogicalDomainExpr::into_physical().
/// Could potentially be removed, but provides a clear separation of concerns.
struct AssignmentPhysicalPlanner {}

impl AssignmentPhysicalPlanner {
    /// Converts a LogicalDomainExpr into an executable PhysicalDomainExpr.
    fn plan(&self, logical_expr: LogicalDomainExpr) -> Result<PhysicalDomainExpr> {
        logical_expr.into_physical()
    }
}

/// Represents an expression ready for execution on OTAP batches.
///
/// This is the final phase before execution. It contains:
/// - The logical expression (DataFusion Expr)
/// - A lazily-initialized physical expression (created using actual batch schema)
/// - A projection defining which columns are needed
/// - Optional child expression for cross-domain operations
///
/// The PhysicalExprRef is created lazily on first execution because we need the
/// actual RecordBatch schema to convert the logical Expr into a physical expression.
/// Once created, it's cached for subsequent batches with the same schema.
struct PhysicalDomainExpr {
    /// The data domain this expression operates on
    data_domain: DataDomainId,
    /// DataFusion logical expression (e.g., col("severity_number") + lit(1))
    logical_expr: Expr,

    /// Lazily initialized physical expression, cached after first execution.
    /// Created using create_physical_expr() with the actual batch schema.
    physical_expr: Option<PhysicalExprRef>,

    /// Projection that identifies which columns are needed from the input batch.
    /// TODO - we should rename this type to just "Projection"
    /// TODO - since this is only needed for root, should be in domain?
    projection: FilterProjection,

    /// Optional child expression for cross-domain operations.
    /// The parent expression references this via col("child").
    child: Option<Box<PhysicalDomainExpr>>,
}

impl PhysicalDomainExpr {
    /// Executes the expression on an OTAP batch and returns the result as a ColumnarValue.
    ///
    /// # Why Option<ColumnarValue>?
    ///
    /// Returns `Option<ColumnarValue>` to handle both missing data and scalar/array results:
    /// - `None` - When required input data is missing (e.g., optional attribute batch not present)
    /// - `Some(ColumnarValue::Array(ArrayRef))` - for array results (e.g., column references)
    /// - `Some(ColumnarValue::Scalar(ScalarValue, usize))` - for scalar results with row count
    ///
    /// This is crucial for scalar expansion. When evaluating `scalar + array`:
    /// 1. Scalar side returns `Some(ColumnarValue::Scalar(value, 0))` (row count unknown yet)
    /// 2. Array side returns `Some(ColumnarValue::Array(array))` with N rows
    /// 3. DataFusion's PhysicalExpr automatically expands the scalar to N rows
    /// 4. The operation is performed efficiently without pre-materializing the scalar array
    ///
    /// For cross-domain operations (parent-child), we need to match row counts:
    /// - If parent is array (N rows) and child is scalar: expand child to N rows
    /// - If parent is scalar and child is array (M rows): expand parent to M rows
    /// - If both are arrays: they must have same row count (or use join logic)
    ///
    /// # Execution Steps:
    /// 1. Get the appropriate input RecordBatch based on data_domain:
    ///    - Root: Main telemetry batch (e.g., Logs)
    ///    - Attributes: Attribute batch filtered by key
    ///    - StaticScalar: Empty batch (constants don't need input)
    /// 2. Project the batch to only needed columns (via FilterProjection)
    /// 3. Recursively execute child expression if present
    /// 4. Lazily create PhysicalExprRef from logical_expr using batch schema (if not cached)
    /// 5. Evaluate the physical expression and return the result
    ///
    /// # Arguments
    /// * `otap_batch` - The OTAP batch containing multiple RecordBatches
    /// * `session_state` - DataFusion session state needed for physical planning
    ///
    /// # Returns
    /// * `Ok(Some(ColumnarValue))` - Successful evaluation as Array or Scalar
    /// * `Ok(None)` - No input data available (e.g., optional attribute batch missing)
    /// * `Err(...)` - Evaluation error
    fn execute(&mut self, otap_batch: &OtapArrowRecords, session_state: &SessionState) -> Result<Option<ColumnarValue>> {
        // Step 1: Get the input RecordBatch based on the data domain
        let input_rb = match &self.data_domain {
            DataDomainId::Root => otap_batch.root_record_batch().map(|rb| {
                // Project to only the columns needed by this expression
                // TODO: Handle case where projection fails (missing columns)
                self.projection.project(rb).unwrap()
            }),
            DataDomainId::Attributes(attrs_id, key) => {
                // Get the appropriate attributes batch based on AttributesIdentifier
                let attrs_payload_type = match *attrs_id {
                    AttributesIdentifier::Root => match otap_batch.root_payload_type() {
                        ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
                        ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
                        _ => ArrowPayloadType::MetricAttrs,
                    },
                    AttributesIdentifier::NonRoot(paylod_type) => paylod_type,
                };

                match otap_batch.get(attrs_payload_type) {
                    Some(rb) => self.try_project_attrs(rb, key.as_str())?,
                    None => None,
                }
            }
            DataDomainId::StaticScalar => {
                // Static scalars don't need input data, so provide an empty batch
                // TODO: This could be a lazy static to avoid repeated allocation
                Some(RecordBatch::new_empty(Arc::new(Schema::new(
                    Vec::<Field>::new(),
                ))))
            }
        };

        // Step 2: Recursively execute child expression if present
        let child_arr = match &mut self.child {
            Some(child) => Some(child.execute(otap_batch, session_state)?),
            None => None,
        };

        // Step 3: Lazily create the physical expression if not already cached.
        // We need the actual batch schema to convert logical Expr -> PhysicalExpr.
        // Once created, it's cached and reused for subsequent batches.
        if self.physical_expr.is_none() {
            if let Some(ref rb) = input_rb {
                let df_schema = DFSchema::try_from(rb.schema_ref().as_ref().clone())?;
                let physical_expr = create_physical_expr(&self.logical_expr, &df_schema, session_state.execution_props())?;
                self.physical_expr = Some(physical_expr);
            } else {
                // If there's no input batch, return None (missing data)
                return Ok(None);
            }
        }

        // Step 4: Evaluate the physical expression and combine with child if needed
        // TODO: Implement the actual evaluation logic for each case:
        match (input_rb, child_arr) {
            (Some(input), Some(child)) => {
                // Both parent and child domains have data
                // TODO: Add child result as a column named "child" to input batch.
                // Steps:
                // 1. Convert child Option<ColumnarValue> to array matching input.num_rows():
                //    - If child is None: return error (child expression failed)
                //    - If child is Some(Array): verify row count matches or error
                //    - If child is Some(Scalar): expand to input.num_rows() using into_array()
                // 2. Add child array as new column to input batch with name "child"
                // 3. Evaluate self.physical_expr on the extended batch
                // 4. Return Some(resulting ColumnarValue)
                //
                // Example: log.severity_number + attributes["http.status"]
                // - input batch has severity_number column (N rows)
                // - child is Some(Array) of http.status values (should be N rows after join)
                // - parent expr is: col("severity_number") + col("child")
                todo!()
            },
            (Some(input), None) => {
                // Only parent domain has data (most common case)
                // TODO: Evaluate self.physical_expr.evaluate(&input) and wrap in Some().
                // PhysicalExpr::evaluate() returns ColumnarValue directly.
                //
                // Examples:
                // - lit(42): Returns Some(Scalar(42, N)) where N = input.num_rows()
                // - col("severity_number"): Returns Some(Array(severity_number_array))
                // - col("severity_number") + lit(1): Returns Some(Array) (DataFusion expands scalar)
                //
                // The returned ColumnarValue preserves whether the result is scalar or array,
                // allowing efficient composition in parent expressions.
                todo!()
            },
            (None, Some(child)) => {
                // Parent domain has no data but child does
                // Could happen if root batch is missing but attributes exist (unlikely)
                // Return the child result directly
                Ok(child)
            },
            (None, None) => {
                // Neither parent nor child has data - return None
                Ok(None)
            }
        }
    }

    /// Projects an attributes RecordBatch to only rows matching the specified key.
    ///
    /// Attributes batches have a key-value structure with columns:
    /// - key: String column for attribute key
    /// - type: Enum (0-8) identifying which value column contains the data
    /// - str/int/float/bytes/bool/ser: Type-specific value columns
    ///
    /// This method:
    /// 1. Filters the batch to rows where key matches the specified key
    /// 2. TODO: Inspects the type column to determine which value column to use
    /// 3. TODO: Projects the appropriate value column as "value"
    ///
    /// # Arguments
    /// * `record_batch` - The attributes RecordBatch to filter
    /// * `key` - The attribute key to filter by (e.g., "http.method")
    ///
    /// # Returns
    /// * `Ok(Some(RecordBatch))` - Filtered batch with the attribute values
    /// * `Ok(None)` - No rows matched the key
    /// * `Err(...)` - Error during filtering

    fn try_project_attrs(
        &self,
        record_batch: &RecordBatch,
        key: &str,
    ) -> Result<Option<RecordBatch>> {
        // Get the key column and create a mask for rows matching the specified key
        let key_col =
            get_required_array(record_batch, consts::ATTRIBUTE_KEY).map_err(|e| Error::ExecutionError {
                cause: e.to_string(),
            })?;
        let key_mask = eq(key_col, &StringArray::new_scalar(key))?;
        let filtered_batch = filter_record_batch(record_batch, &key_mask)?;
        
        // TODO: Implement type detection and value projection
        // Steps needed:
        // 1. Look at the 'type' column to find the first non-null row
        // 2. Based on the type value (0-8), determine which value column to use:
        //    - str, int, float, bytes, bool, or ser
        // 3. Project that column and rename it to "value"
        // 4. If filtered_batch is empty (no matching key), return Ok(None)
        // 5. If other side of operation has different type, return error
        todo!()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::Int64Array;
    use arrow::datatypes::DataType;
    use datafusion::logical_expr::lit;
    use datafusion::physical_expr::expressions::Literal;
    use datafusion::scalar::ScalarValue;
    use otap_df_pdata::otap::Logs;
    use crate::pipeline::project::FilterProjection;

    #[test]
    fn test_physical_domain_expr_static_scalar() {
        // Create an empty OtapArrowRecords for logs
        let otap_batch = OtapArrowRecords::Logs(Logs::default());

        // Create a logical constant expression (literal 42)
        let logical_expr = lit(42i64);

        // Create a FilterProjection for the static scalar (empty schema since no columns needed)
        let projection = FilterProjection::try_new(&logical_expr).unwrap();

        // Create a PhysicalDomainExpr with StaticScalar domain
        let mut physical_expr = PhysicalDomainExpr {
            data_domain: DataDomainId::StaticScalar,
            logical_expr,
            physical_expr: None,
            projection,
            child: None,
        };

        // Create a session state for execution
        let session_ctx = datafusion::execution::context::SessionContext::new();
        let session_state = session_ctx.state();

        // Execute the expression - will hit todo!() for now
        let result = physical_expr.execute(&otap_batch, &session_state);
        
        // Once execute() is implemented for StaticScalar, this should return:
        // - Ok(ColumnarValue::Scalar(...)) for constant values
        // For now, we just verify the structure is set up correctly
        assert!(result.is_err());
    }

    // TODO - this test can be thrown away later once we actually invoke the physical expr
    #[test]
    fn test_logical_to_physical_conversion() {
        // Create a LogicalDomainExpr with a simple constant
        let logical_expr = LogicalDomainExpr {
            data_domain: LogicalDataDomain {
                domain_id: DataDomainId::StaticScalar,
            },
            logical_expr: lit(100i64),
            child: None,
        };

        // Convert to physical
        let physical_expr = logical_expr.into_physical();
        assert!(physical_expr.is_ok());

        let physical_expr = physical_expr.unwrap();
        assert_eq!(physical_expr.data_domain, DataDomainId::StaticScalar);
        assert!(physical_expr.physical_expr.is_none()); // Not yet evaluated
        assert!(physical_expr.child.is_none());
    }

    // TODO - this test can be thrown away later once we actually invoke the physical expr
    #[test]
    fn test_logical_to_physical_with_child() {
        // Create a LogicalDomainExpr with a child
        let child_logical = LogicalDomainExpr {
            data_domain: LogicalDataDomain {
                domain_id: DataDomainId::StaticScalar,
            },
            logical_expr: lit(50i64),
            child: None,
        };

        let parent_logical = LogicalDomainExpr {
            data_domain: LogicalDataDomain {
                domain_id: DataDomainId::Root,
            },
            logical_expr: col("severity_number"),
            child: Some(Box::new(child_logical)),
        };

        // Convert to physical
        let physical_expr = parent_logical.into_physical();
        assert!(physical_expr.is_ok());

        let physical_expr = physical_expr.unwrap();
        assert_eq!(physical_expr.data_domain, DataDomainId::Root);
        assert!(physical_expr.child.is_some());

        // Verify child was also converted
        let child = physical_expr.child.unwrap();
        assert_eq!(child.data_domain, DataDomainId::StaticScalar);
    }

    #[test]
    fn test_planner_static_integer() {
        use data_engine_expressions::{IntegerScalarExpression, QueryLocation, StaticScalarExpression};
        
        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 42)
        ));
        
        let result = planner.plan_scalar_expr(&static_expr);
        assert!(result.is_ok());
        
        let logical_expr = result.unwrap();
        assert_eq!(logical_expr.data_domain.domain_id, DataDomainId::StaticScalar);
        assert!(logical_expr.child.is_none());
    }

    #[test]
    fn test_planner_static_string() {
        use data_engine_expressions::{QueryLocation, StaticScalarExpression, StringScalarExpression};
        
        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::String(
            StringScalarExpression::new(QueryLocation::new_fake(), "hello")
        ));
        
        let result = planner.plan_scalar_expr(&static_expr);
        assert!(result.is_ok());
        
        let logical_expr = result.unwrap();
        assert_eq!(logical_expr.data_domain.domain_id, DataDomainId::StaticScalar);
    }

    #[test]
    fn test_planner_static_boolean() {
        use data_engine_expressions::{BooleanScalarExpression, QueryLocation, StaticScalarExpression};
        
        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::Boolean(
            BooleanScalarExpression::new(QueryLocation::new_fake(), true)
        ));
        
        let result = planner.plan_scalar_expr(&static_expr);
        assert!(result.is_ok());
        
        let logical_expr = result.unwrap();
        assert_eq!(logical_expr.data_domain.domain_id, DataDomainId::StaticScalar);
    }

    #[test]
    fn test_planner_static_to_physical_execute() {
        use data_engine_expressions::{IntegerScalarExpression, QueryLocation, StaticScalarExpression};
        
        // Plan the scalar expression
        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 99)
        ));
        
        let logical_expr = planner.plan_scalar_expr(&static_expr).unwrap();
        
        // Convert to physical
        let mut physical_expr = logical_expr.into_physical().unwrap();
        
        // Execute
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let session_ctx = datafusion::execution::context::SessionContext::new();
        let session_state = session_ctx.state();
        
        let result = physical_expr.execute(&otap_batch, &session_state);
        
        // For now, will hit todo!() but structure is correct
        assert!(result.is_err());
    }
}
