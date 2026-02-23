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
use datafusion::functions::core::expr_ext::FieldAccessor;
use datafusion::logical_expr::{BinaryExpr, ColumnarValue, Expr, Operator, col, lit};
use datafusion::physical_expr::{PhysicalExprRef, create_physical_expr};
use datafusion::physical_plan::PhysicalExpr;
use datafusion::prelude::SessionContext;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::get_required_array;
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::expr::join::join;
use crate::pipeline::planner::{AttributesIdentifier, ColumnAccessor};
use crate::pipeline::project::{Projection, ProjectionOptions};

mod join;

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

    // TODO comments
    requires_dict_downcast: bool,
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
        let projection = Projection::try_new(&self.logical_expr)?;

        let child = match self.child {
            Some(child_expr) => Some(Box::new(child_expr.into_physical()?)),
            None => None,
        };

        Ok(PhysicalDomainExpr {
            data_domain: self.data_domain.domain_id,
            logical_expr: self.logical_expr,
            physical_expr: None,
            projection,
            projection_opts: ProjectionOptions {
                downcast_dicts: self.requires_dict_downcast,
            },
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
                let value_accessor = source_scalar_expr.get_value_accessor();
                let column_accessor = ColumnAccessor::try_from(value_accessor)?;

                match column_accessor {
                    ColumnAccessor::ColumnName(column_name) => Ok(LogicalDomainExpr {
                        data_domain: LogicalDataDomain {
                            domain_id: DataDomainId::Root,
                        },
                        logical_expr: col(column_name),
                        child: None,
                        requires_dict_downcast: false,
                    }),
                    ColumnAccessor::StructCol(column_name, struct_field_name) => {
                        Ok(LogicalDomainExpr {
                            data_domain: LogicalDataDomain {
                                domain_id: DataDomainId::Root,
                            },
                            logical_expr: col(column_name).field(struct_field_name),
                            child: None,
                            requires_dict_downcast: false,
                        })
                    }
                    ColumnAccessor::Attributes(attrs_id, key) => {
                        // Attribute access like attributes["http.status"] creates a logical expression
                        // that operates on the Attributes domain. The key is stored in the domain_id
                        // so that during physical execution, we can:
                        // 1. Filter the attribute batch to rows matching this key
                        // 2. Detect the attribute's type from the 'type' column
                        // 3. Project the appropriate value column (str/int/double/bool/bytes)
                        // 4. Rename it to "value"
                        //
                        // The logical expression is simply col("value") because after projection,
                        // the attribute batch will have a column named "value" containing the
                        // attribute values of the correct type.
                        Ok(LogicalDomainExpr {
                            data_domain: LogicalDataDomain {
                                domain_id: DataDomainId::Attributes(attrs_id, key),
                            },
                            // TODO could have a const for this column name
                            logical_expr: col("value"),
                            child: None,
                            requires_dict_downcast: false,
                        })
                    }
                }
            }
            ScalarExpression::Static(static_scalar_expr) => {
                // Convert static scalar constants to DataFusion literals.
                // All static scalars belong to the StaticScalar domain.
                // TODO - don't like how this is imported
                use data_engine_expressions::StaticScalarExpression as SSE;

                let logical_expr = match static_scalar_expr {
                    SSE::Integer(int_expr) => lit(int_expr.get_value()),
                    SSE::Double(double_expr) => lit(double_expr.get_value()),
                    SSE::Boolean(bool_expr) => lit(bool_expr.get_value()),
                    SSE::String(string_expr) => lit(string_expr.get_value()),
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
                    requires_dict_downcast: false,
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
                            requires_dict_downcast: true,
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
                            requires_dict_downcast: true,
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
    projection: Projection,

    // TODO comments
    projection_opts: ProjectionOptions,

    /// Optional child expression for cross-domain operations.
    /// The parent expression references this via col("child").
    child: Option<Box<PhysicalDomainExpr>>,
}

pub(crate) struct PhysicalExprEvalResult {
    values: ColumnarValue,
    ids: Option<ArrayRef>,
    parent_ids: Option<ArrayRef>,
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
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_context: &SessionContext,
        include_ids: bool,
    ) -> Result<Option<PhysicalExprEvalResult>> {
        // Step 1: Get the input RecordBatch based on the data domain
        let mut input_rb = match &self.data_domain {
            // TODO - how we're projecting here doesn't seem right ...
            DataDomainId::Root => otap_batch.root_record_batch().cloned(), // TODO Cow not clone
            // .map(|rb| {
            //     // Project to only the columns needed by this expression
            //     // TODO: Handle case where projection fails (missing columns)
            //     let rb = self.projection.project_with_options(rb, &self.projection_opts);

            //     // TODO need to stick the ID column back onto this thing if there's
            //     // a child to execute b/c we'll need to join it

            //     rb
            // })
            // .flatten(),
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


        let mut input_rb = match input_rb {
            Some(input_rb) => input_rb,
            None => {
                todo!()
            }
        };


        // TODO remove all this debug stuff
        println!("data domain = {:?}", self.data_domain);
        println!("projection = {:?}", self.projection);
        println!("expr = {:?}", self.logical_expr);
        println!("physical_expr = {:?}", self.physical_expr);
        arrow::util::pretty::print_batches(&[input_rb.clone()]).unwrap();
        // println!("input rb = {:?}", input_rb);


        // TODO there's somewhere else we need to apply projection here ....

        // Step 2: Recursively execute child expression if present & join to parent
        if let Some(child) = &mut self.child {
            let child_exec_result = child.execute(otap_batch, session_context, true)?;
            match child_exec_result {
                Some(child_exec_result) => {
                    input_rb = join(
                        &input_rb,
                        &self.data_domain,
                        &child_exec_result,
                        &child.data_domain,
                    )?;

                    // TODO no unwrap
                    input_rb = self
                        .projection
                        .project_with_options(&input_rb, &self.projection_opts)
                        .unwrap();

                    println!(
                        "implementing Some(input), Some(child): {:?}",
                        self.physical_expr
                    );
                }
                None => {
                    todo!()
                }
            }
        } else {
            if self.data_domain == DataDomainId::Root {
                // TODO no unwrap
                input_rb = self.projection.project(&input_rb).unwrap()
            }
        }

        // Step 3: Lazily create the physical expression if not already cached.
        // We need the actual batch schema to convert logical Expr -> PhysicalExpr.
        // Once created, it's cached and reused for subsequent batches.
        //
        // TODO - there might be a pattern where we can do this up front without data, since we
        // are always projecting to a schema with columns in the correct order, we _should_ be able
        // to produce a representative schema if we know the logical types for each column
        if self.physical_expr.is_none() {
            let session_state = session_context.state();
            let df_schema = DFSchema::try_from(input_rb.schema_ref().as_ref().clone())?;
            let physical_expr = create_physical_expr(
                &self.logical_expr,
                &df_schema,
                session_state.execution_props(),
            )?;
            self.physical_expr = Some(physical_expr);
        }

        // TODO - should we cast back to a dict here if the originals were dicts or
        // if the source allows it ...

        let mut result = PhysicalExprEvalResult {
            values: self.physical_expr.as_ref().unwrap().evaluate(&input_rb)?,
            ids: None,
            parent_ids: None,
        };

        // TODO test this is returned
        if include_ids {
            result.ids = input_rb.column_by_name(consts::ID).cloned();
            result.parent_ids = input_rb.column_by_name(consts::PARENT_ID).cloned();
        }
        Ok(Some(result))
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

    /// Projects an attribute batch to extract values for a specific key.
    ///
    /// # Attribute Batch Structure
    /// Attribute batches have a columnar structure where each row represents one attribute:
    /// - `key`: The attribute key (e.g., "http.status", "service.name")
    /// - `type`: A u8 indicating the value type (1=Str, 2=Int, 3=Double, 4=Bool, 7=Bytes, etc.)
    /// - `str`, `int`, `double`, `bool`, `bytes`: Value columns (only one contains actual data per row)
    /// - `parent_id`: Links back to the parent record (e.g., log record, span)
    ///
    /// # Projection Strategy
    /// Since we don't know the type at planning time, we:
    /// 1. Filter rows to only those matching the specified key
    /// 2. Look at the 'type' column to determine which value column to use
    /// 3. Project just the parent_id and the appropriate value column
    /// 4. Rename the value column to "value" so the logical expression can reference it uniformly
    ///
    /// This allows expressions like `attributes["http.status"] + 200` to work without knowing
    /// at planning time whether http.status is an int, string, etc.
    fn try_project_attrs(
        &self,
        record_batch: &RecordBatch,
        key: &str,
    ) -> Result<Option<RecordBatch>> {
        // Get the key column and create a mask for rows matching the specified key
        let key_col = get_required_array(record_batch, consts::ATTRIBUTE_KEY).map_err(|e| {
            Error::ExecutionError {
                cause: e.to_string(),
            }
        })?;
        let key_mask = eq(key_col, &StringArray::new_scalar(key))?;
        let filtered_batch = filter_record_batch(record_batch, &key_mask)?;

        // If no rows match the key, handle empty case
        if filtered_batch.num_rows() == 0 {
            // TODO: Decide if this should be Ok(None) or an error
            todo!("Handle empty filtered batch - no matching attribute key")
        }

        // Get the type column to determine which value column to use
        let type_arr =
            get_required_array(&filtered_batch, consts::ATTRIBUTE_TYPE).map_err(|e| {
                Error::ExecutionError {
                    cause: e.to_string(),
                }
            })?;

        let type_col = type_arr
            .as_any()
            .downcast_ref::<arrow::array::UInt8Array>()
            .ok_or_else(|| Error::ExecutionError {
                cause: format!(
                    "Expected UInt8 for type column, got {:?}",
                    type_arr.data_type()
                ),
            })?;

        // TODO - we're making two big (and potentially invalid) assumptions here:
        // 1. if a type is present for some key, then all attributes for this key have the same
        // type. Normally this would be the case and this is definitely best practice, but some
        // users might just choose to do something bizarre so we'll need to handle that
        // 2. we're assuming that if the type column indicates some value type, that the values
        // column is supposed to be present. This isn't necessarily the case, because we might
        // have a case where all the attribute values are either null or default value. This is
        // actually a problem because when we relax this assumption, we still won't know whether
        // it's null or default value, and won't be able to just guess either way w/out sometimes
        // guessing wrong. For now, just punting the problem ...

        // Find the first non-null type value
        let type_value = type_col
            .iter()
            .find_map(|v| v)
            .ok_or_else(|| Error::ExecutionError {
                cause: "No non-null type value found in filtered attributes".to_string(),
            })?;

        // TODO no unwrap
        let type_value = AttributeValueType::try_from(type_value).unwrap();

        // Based on type value, select the appropriate value column

        // TODO - we could use helper functions to cut down on all this repeated error handling
        // code -- although, see the TODO above about whethere we _actually_ want this error
        // handling code or whether we just return None ...
        // (it's LLM generated so needs cleaned up)
        let value_array = match type_value {
            AttributeValueType::Str => {
                // Str type
                let arr = filtered_batch
                    .column_by_name(consts::ATTRIBUTE_STR)
                    .ok_or_else(|| Error::ExecutionError {
                        cause: format!("Missing {} column for str type", consts::ATTRIBUTE_STR),
                    })?;
                arr.clone()
            }
            AttributeValueType::Int => {
                // Int type
                let arr = filtered_batch
                    .column_by_name(consts::ATTRIBUTE_INT)
                    .ok_or_else(|| Error::ExecutionError {
                        cause: format!("Missing {} column for int type", consts::ATTRIBUTE_INT),
                    })?;
                arr.clone()
            }
            AttributeValueType::Double => {
                // Double type
                let arr = filtered_batch
                    .column_by_name(consts::ATTRIBUTE_DOUBLE)
                    .ok_or_else(|| Error::ExecutionError {
                        cause: format!(
                            "Missing {} column for double type",
                            consts::ATTRIBUTE_DOUBLE
                        ),
                    })?;
                arr.clone()
            }
            AttributeValueType::Bool => {
                // Bool type
                let arr = filtered_batch
                    .column_by_name(consts::ATTRIBUTE_BOOL)
                    .ok_or_else(|| Error::ExecutionError {
                        cause: format!("Missing {} column for bool type", consts::ATTRIBUTE_BOOL),
                    })?;
                arr.clone()
            }
            AttributeValueType::Bytes => {
                // Bytes type
                let arr = filtered_batch
                    .column_by_name(consts::ATTRIBUTE_BYTES)
                    .ok_or_else(|| Error::ExecutionError {
                        cause: format!("Missing {} column for bytes type", consts::ATTRIBUTE_BYTES),
                    })?;
                arr.clone()
            }
            AttributeValueType::Empty => {
                // Empty type
                todo!("Handle Empty attribute type")
            }
            AttributeValueType::Map => {
                // Map type
                todo!("Handle Map attribute type")
            }
            AttributeValueType::Slice => {
                // Slice type
                todo!("Handle Slice attribute type")
            }
        };

        // Build new schema with parent_id (if present) and value column renamed to "value"
        let mut fields = Vec::new();
        let mut columns = Vec::new();

        // Keep parent_id column if it exists
        // TODO - the LLM Agent added this "Exists" check, but the column should pretty much
        // always be there so this check should go away.
        if let Some(parent_id_col) = filtered_batch.column_by_name(consts::PARENT_ID) {
            fields.push(Arc::new(Field::new(
                consts::PARENT_ID,
                parent_id_col.data_type().clone(),
                false,
            )));
            columns.push(parent_id_col.clone());
        }

        // Add the value column renamed to "value"
        fields.push(Arc::new(Field::new(
            "value",
            value_array.data_type().clone(),
            true,
        )));
        columns.push(value_array);

        if self.projection_opts.downcast_dicts {
            Projection::downcast_dicts(&mut fields, &mut columns);
        }

        let schema = Arc::new(Schema::new(fields));
        let projected_batch =
            RecordBatch::try_new(schema, columns).map_err(|e| Error::ExecutionError {
                cause: format!("Failed to create projected batch: {}", e),
            })?;

        Ok(Some(projected_batch))
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{Int32Array, Int64Array, StructArray};
    use arrow::compute::take;
    use arrow::datatypes::DataType;
    use data_engine_expressions::{
        IntegerScalarExpression, QueryLocation, StaticScalarExpression, StringScalarExpression,
        ValueAccessor,
    };
    use datafusion::logical_expr::lit;
    use datafusion::physical_expr::expressions::Literal;
    use datafusion::scalar::ScalarValue;
    // TODO ugly import
    use crate::consts::{ATTRIBUTES_FIELD_NAME, SCOPE_FIELD_NAME};
    use crate::pipeline::{Pipeline, project::Projection};
    use otap_df_pdata::{
        otap::Logs,
        proto::{
            OtlpProtoMessage,
            opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue},
            opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
        },
        testing::round_trip::{otlp_to_otap, to_logs_data},
    };

    // #[test]
    // fn test_physical_domain_expr_static_scalar() {
    //     // Create an empty OtapArrowRecords for logs
    //     let otap_batch = OtapArrowRecords::Logs(Logs::default());

    //     // Create a logical constant expression (literal 42)
    //     let logical_expr = lit(42i64);

    //     // Create a FilterProjection for the static scalar (empty schema since no columns needed)
    //     let projection = Projection::try_new(&logical_expr).unwrap();

    //     // Create a PhysicalDomainExpr with StaticScalar domain
    //     let mut physical_expr = PhysicalDomainExpr {
    //         data_domain: DataDomainId::StaticScalar,
    //         logical_expr,
    //         physical_expr: None,
    //         projection,
    //         child: None,
    //     };

    //     // Create a session state for execution
    //     let session_ctx = Pipeline::create_session_context();

    //     // Execute the expression
    //     let result = physical_expr.execute(&otap_batch, &session_ctx, false);

    //     // Should successfully evaluate the static scalar
    //     assert!(result.is_ok());
    //     let columnar_value = result.unwrap();
    //     assert!(columnar_value.is_some());

    //     // Verify it's a scalar value
    //     match columnar_value.unwrap().values {
    //         ColumnarValue::Scalar(scalar) => {
    //             // Should be the literal value 42
    //             assert_eq!(scalar, datafusion::scalar::ScalarValue::Int64(Some(42)));
    //         }
    //         ColumnarValue::Array(_) => {
    //             panic!("Expected scalar, got array");
    //         }
    //     }
    // }

    // // TODO - this test can be thrown away later once we actually invoke the physical expr
    // #[test]
    // fn test_logical_to_physical_conversion() {
    //     // Create a LogicalDomainExpr with a simple constant
    //     let logical_expr = LogicalDomainExpr {
    //         data_domain: LogicalDataDomain {
    //             domain_id: DataDomainId::StaticScalar,
    //         },
    //         logical_expr: lit(100i64),
    //         child: None,

    //     };

    //     // Convert to physical
    //     let physical_expr = logical_expr.into_physical();
    //     assert!(physical_expr.is_ok());

    //     let physical_expr = physical_expr.unwrap();
    //     assert_eq!(physical_expr.data_domain, DataDomainId::StaticScalar);
    //     assert!(physical_expr.physical_expr.is_none()); // Not yet evaluated
    //     assert!(physical_expr.child.is_none());
    // }

    // // TODO - this test can be thrown away later once we actually invoke the physical expr
    // #[test]
    // fn test_logical_to_physical_with_child() {
    //     // Create a LogicalDomainExpr with a child
    //     let child_logical = LogicalDomainExpr {
    //         data_domain: LogicalDataDomain {
    //             domain_id: DataDomainId::StaticScalar,
    //         },
    //         logical_expr: lit(50i64),
    //         child: None,
    //     };

    //     let parent_logical = LogicalDomainExpr {
    //         data_domain: LogicalDataDomain {
    //             domain_id: DataDomainId::Root,
    //         },
    //         logical_expr: col("severity_number"),
    //         child: Some(Box::new(child_logical)),
    //     };

    //     // Convert to physical
    //     let physical_expr = parent_logical.into_physical();
    //     assert!(physical_expr.is_ok());

    //     let physical_expr = physical_expr.unwrap();
    //     assert_eq!(physical_expr.data_domain, DataDomainId::Root);
    //     assert!(physical_expr.child.is_some());

    //     // Verify child was also converted
    //     let child = physical_expr.child.unwrap();
    //     assert_eq!(child.data_domain, DataDomainId::StaticScalar);
    // }

    #[test]
    fn test_planner_static_integer() {
        use data_engine_expressions::{
            IntegerScalarExpression, QueryLocation, StaticScalarExpression,
        };

        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 42),
        ));

        let result = planner.plan_scalar_expr(&static_expr);
        assert!(result.is_ok());

        let logical_expr = result.unwrap();
        assert_eq!(
            logical_expr.data_domain.domain_id,
            DataDomainId::StaticScalar
        );
        assert!(logical_expr.child.is_none());
    }

    #[test]
    fn test_planner_static_string() {
        use data_engine_expressions::{
            QueryLocation, StaticScalarExpression, StringScalarExpression,
        };

        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::String(
            StringScalarExpression::new(QueryLocation::new_fake(), "hello"),
        ));

        let result = planner.plan_scalar_expr(&static_expr);
        assert!(result.is_ok());

        let logical_expr = result.unwrap();
        assert_eq!(
            logical_expr.data_domain.domain_id,
            DataDomainId::StaticScalar
        );
    }

    #[test]
    fn test_planner_static_boolean() {
        // TODO - don't like these local imports
        use data_engine_expressions::{
            BooleanScalarExpression, QueryLocation, StaticScalarExpression,
        };

        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::Boolean(
            BooleanScalarExpression::new(QueryLocation::new_fake(), true),
        ));

        let result = planner.plan_scalar_expr(&static_expr);
        assert!(result.is_ok());

        let logical_expr = result.unwrap();
        assert_eq!(
            logical_expr.data_domain.domain_id,
            DataDomainId::StaticScalar
        );
    }

    // TODO the name the LLM generated for these tests is a bit hokey ...
    // "_to_physical_expr" ugh
    // I think we can

    #[test]
    fn test_planner_static_to_physical_execute() {
        // Plan the scalar expression
        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 99),
        ));

        let logical_expr = planner.plan_scalar_expr(&static_expr).unwrap();

        // Convert to physical
        let mut physical_expr = logical_expr.into_physical().unwrap();

        // Execute
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr.execute(&otap_batch, &session_ctx, false);

        // Should successfully evaluate
        assert!(result.is_ok());
        let columnar_value = result.unwrap();
        assert!(columnar_value.is_some());

        // Verify it's a scalar value of 99
        match columnar_value.unwrap().values {
            ColumnarValue::Scalar(scalar) => {
                assert_eq!(scalar, datafusion::scalar::ScalarValue::Int64(Some(99)));
            }
            ColumnarValue::Array(_) => {
                panic!("Expected scalar, got array");
            }
        }
    }

    #[test]
    fn test_planner_root_source_to_physical_execute() {
        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_TEXT,
                )),
            )]),
        ));

        let logical_expr = planner.plan_scalar_expr(&static_expr).unwrap();

        // Convert to physical
        let mut physical_expr = logical_expr.into_physical().unwrap();

        let logs = to_logs_data(vec![
            LogRecord::build().severity_text("ERROR").finish(),
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr.execute(&otap_batch, &session_ctx, false);

        // get the expected column
        let logs = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let input_col = logs.column_by_name(consts::SEVERITY_TEXT).unwrap();

        // Should successfully evaluate
        assert!(result.is_ok());
        let columnar_value = result.unwrap();
        assert!(columnar_value.is_some());

        // Verify it's a scalar value of 99
        match columnar_value.unwrap().values {
            ColumnarValue::Scalar(_) => {
                panic!("Expected scalar, got array");
            }
            ColumnarValue::Array(arr) => {
                assert_eq!(arr.as_ref(), input_col.as_ref())
            }
        }
    }

    #[test]
    fn test_planner_root_struct_to_physical_execute() {
        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), SCOPE_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), consts::NAME),
                )),
            ]),
        ));

        let logical_expr = planner.plan_scalar_expr(&static_expr).unwrap();

        // Convert to physical
        let mut physical_expr = logical_expr.into_physical().unwrap();

        let logs = LogsData::new(vec![ResourceLogs {
            scope_logs: vec![
                ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![LogRecord::build().severity_text("INFO").finish()],
                ),
                ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope2".into(),
                        ..Default::default()
                    },
                    vec![LogRecord::build().severity_text("INFO").finish()],
                ),
            ],
            ..Default::default()
        }]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr.execute(&otap_batch, &session_ctx, false);

        // get the expected column
        let logs = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let scope_col = logs.column_by_name(consts::SCOPE).unwrap();
        let input_col = scope_col
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap()
            .column_by_name(consts::NAME)
            .unwrap();

        // Should successfully evaluate
        assert!(result.is_ok());
        let columnar_value = result.unwrap();
        assert!(columnar_value.is_some());

        // Verify it's a scalar value of 99
        match columnar_value.unwrap().values {
            ColumnarValue::Scalar(_) => {
                panic!("Expected scalar, got array");
            }
            ColumnarValue::Array(arr) => {
                assert_eq!(arr.as_ref(), input_col.as_ref())
            }
        }
    }

    #[test]
    fn test_planner_attribute_source_to_physical_execute() {
        let mut planner = AssignmentLogicalPlanner {};
        let static_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "k2"),
                )),
            ]),
        ));

        let logical_expr = planner.plan_scalar_expr(&static_expr).unwrap();

        // Convert to physical
        let mut physical_expr = logical_expr.into_physical().unwrap();

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("x")),
                    KeyValue::new("k2", AnyValue::new_string("y")),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k2", AnyValue::new_string("x")),
                    KeyValue::new("k3", AnyValue::new_string("y")),
                ])
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("k2", AnyValue::new_string("x"))])
                .severity_text("DEBUG")
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr.execute(&otap_batch, &session_ctx, false);

        // get the expected column
        let logs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        let input_col = logs.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        let expected_col = take(input_col, &Int32Array::from(vec![1, 2, 4]), None).unwrap();

        // Should successfully evaluate
        assert!(result.is_ok());
        let columnar_value = result.unwrap();
        assert!(columnar_value.is_some());

        // Verify it's a scalar value of 99
        match columnar_value.unwrap().values {
            ColumnarValue::Scalar(_) => {
                panic!("Expected scalar, got array");
            }
            ColumnarValue::Array(arr) => {
                assert_eq!(arr.as_ref(), expected_col.as_ref())
            }
        }
    }

    #[test]
    fn test_planner_binary_expr_same_attributes() {
        let mut planner = AssignmentLogicalPlanner {};
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "k2"),
                )),
            ]),
        ));

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "k1"),
                )),
            ]),
        ));

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        let logical_expr = planner.plan_scalar_expr(&input_expr).unwrap();

        // Convert to physical
        let mut physical_expr = logical_expr.into_physical().unwrap();

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_int(1)),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k2", AnyValue::new_int(3)),
                    KeyValue::new("k1", AnyValue::new_int(9)),
                ])
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k2", AnyValue::new_int(7)),
                    KeyValue::new("k1", AnyValue::new_int(2)),
                ])
                .severity_text("DEBUG")
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr.execute(&otap_batch, &session_ctx, false);

        // get the expected column
        // let logs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        // let input_col = logs.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        let expected_col = Arc::new(Int64Array::from(vec![4, 12, 9]));

        // Should successfully evaluate
        // assert!(result.is_ok());
        let columnar_value = result.unwrap();
        assert!(columnar_value.is_some());

        // Verify it's a scalar value of 99
        match columnar_value.unwrap().values {
            ColumnarValue::Scalar(_) => {
                panic!("Expected scalar, got array");
            }
            ColumnarValue::Array(arr) => {
                assert_eq!(arr.as_ref(), expected_col.as_ref())
            }
        }
    }

    #[test]
    fn test_planner_binary_expr_root_to_attribute() {
        let mut planner = AssignmentLogicalPlanner {};
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), consts::SEVERITY_NUMBER),
                )),
            ]),
        ));

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "k1"),
                )),
            ]),
        ));

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        let logical_expr = planner.plan_scalar_expr(&input_expr).unwrap();

        // Convert to physical
        let mut physical_expr = logical_expr.into_physical().unwrap();

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_int(1)),
                ])
                .severity_number(10)
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k2", AnyValue::new_int(3)),
                    KeyValue::new("k1", AnyValue::new_int(9)),
                ])
                .severity_number(20)
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k2", AnyValue::new_int(7)),
                    KeyValue::new("k1", AnyValue::new_int(2)),
                ])
                .severity_text("DEBUG")
                .severity_number(30)
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr.execute(&otap_batch, &session_ctx, false);

        // get the expected column
        // let logs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        // let input_col = logs.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        let expected_col = Arc::new(Int64Array::from(vec![13, 29, 32]));

        // Should successfully evaluate
        // assert!(result.is_ok());
        let columnar_value = result.unwrap();
        assert!(columnar_value.is_some());

        // Verify it's a scalar value of 99
        match columnar_value.unwrap().values {
            ColumnarValue::Scalar(_) => {
                panic!("Expected scalar, got array");
            }
            ColumnarValue::Array(arr) => {
                assert_eq!(arr.as_ref(), expected_col.as_ref())
            }
        }
    }
}
