// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of expression evaluation for OTAP (OpenTelemetry Arrow Protocol) batches.
//!
//! # Expression Tree
//!
//! The expressions are planned an executed as a tree of various expression types. The input
//! for planning is an AST of expressions from the [`data_engine_expressions`]. The planning stage
//! converts this into a tree containing datafusion logical plans ([`Expr`]s). At runtime, these
//! logical plans are converted to datafusion physical expressions
//! ([`PhysicalExpr`s](datafusion::physical_expr::PhysicalExprRef)) during evaluation.
//!
//! There is an additional layer of abstraction in the expression tree containing these datafusion
//! logical/physical expressions. This is necessary because typically with typical datafusion
//! expression evaluation, there would be a single [`RecordBatch`] as input and a single expression
//! tree which produces the resulting [`ColumnValue`]. However, in the OTAP data-model, not all
//! data is in one [`RecordBatch`].
//!
//! For this reason, sections of the expression tree are grouped in higher level tree nodes,
//! each containing only the portion of the overall expression tree that can be executed on a
//! single "data scope". Each scope-specific node will evaluate its expression on the source data
//! and the results of these evaluations will be joined together as the expression evaluates.
//!
//! ## Data Scopes
//!
//! The "data scope" represents the source of the data for a given section of the expression tree.
//! It indicates both the source record batch, and the rows that will be selected.
//!
//! For example, consider evaluating an expression like `severity_number + attributes["x"]`. The
//! data scope of the left side of is the root record batch, and for the right side it's the
//! attributes record batch, filtered where `key=="x"`.
//!
//! Note, the data scope is _not_ simply an indicator of the arrow payload type for the record
//! batch. A given payload type can have multiple data scopes, for example `attributes["x"]` and
//! `attributes["y"]` would produce a different set of filtered rows from the same record batch.
//!
//! When evaluating a binary expression with inputs from different data scopes, the execution
//! will join the two inputs before executing the datafusion expression on the join result.
//!
//! *Current status* - for now this only supports a small set of binary arithmetic operations.

use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::{Arc, LazyLock};

use arrow::array::{Array, ArrayRef, RecordBatch, StringArray};
use arrow::compute::filter_record_batch;
use arrow::compute::kernels::cmp::eq;
use arrow::datatypes::{Field, Schema};
use data_engine_expressions::{
    BinaryMathematicalScalarExpression, BooleanValue, DoubleValue, Expression, IntegerValue,
    MathScalarExpression, ScalarExpression, StaticScalarExpression, StringValue,
};
use datafusion::common::DFSchema;
use datafusion::functions::core::expr_ext::FieldAccessor;
use datafusion::logical_expr::{BinaryExpr, ColumnarValue, Expr, Operator, col, lit};
use datafusion::physical_expr::{PhysicalExprRef, create_physical_expr};
use datafusion::prelude::SessionContext;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::arrays::{
    get_optional_array_from_struct_array_from_record_batch, get_required_array,
};
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::expr::join::join;
use crate::pipeline::expr::types::{
    ExprLogicalType, coerce_arithmetic, nested_struct_field_type, root_field_type,
};
use crate::pipeline::planner::{AttributesIdentifier, ColumnAccessor};
use crate::pipeline::project::{Projection, ProjectionOptions};

pub(crate) mod join;
pub(crate) mod types;

pub(crate) const VALUE_COLUMN_NAME: &str = "value";
pub(crate) const LEFT_COLUMN_NAME: &str = "left";
pub(crate) const RIGHT_COLUMN_NAME: &str = "right";

/// Identifies OTAP data either consumed or produced by some expression.
///
/// OTAP batches contain multiple [`RecordBatch`]s, and within a given record batch, some expression
/// may indicate a different set of rows. This type is used to identify both the payload type of
/// the record batch, and which rows may have been selected from it.
///
#[derive(Clone, Debug, PartialEq)]
pub(crate) enum DataScope {
    /// Main telemetry batch (e.g., Logs with columns like severity_number, severity_text)
    Root,

    /// Attribute batch identified by [`AttributesIdentifier`] and filtered by some key.
    /// For example, (AttributesIdentifier::Root, "http.method") may refer to log attributes
    /// with key="http.method"
    Attributes(AttributesIdentifier, String),

    /// A special data scope indicating the data is produced from a static scalar value defined
    /// in the input expression tree, rather than data from the OTAP batch.
    StaticScalar,
}

impl DataScope {
    /// Determines if expressions for two scopes can be combined into a single expression without
    /// performing a join.
    ///
    /// Rules:
    /// - Any scope can combine with StaticScalar (constants)
    /// - Same scopes can combine (e.g., Root + Root), because the row order is the same.
    fn can_combine(&self, other: &Self) -> bool {
        self.is_scalar() || other.is_scalar() || (self == other)
    }

    /// Returns true if this scope represents a static scalar value.
    pub fn is_scalar(&self) -> bool {
        *self == Self::StaticScalar
    }
}

impl From<&ColumnAccessor> for DataScope {
    fn from(value: &ColumnAccessor) -> Self {
        match value {
            ColumnAccessor::ColumnName(_) | ColumnAccessor::StructCol(_, _) => Self::Root,
            ColumnAccessor::Attributes(attrs_id, attrs_key) => {
                Self::Attributes(*attrs_id, attrs_key.clone())
            }
        }
    }
}

/// Identifier of the incoming source data for some scoped expression.
#[derive(Debug)]
enum LogicalExprDataSource {
    /// This indicates the input to the expression data from the incoming OTAP batch
    DataSource(DataScope),

    /// The input to the expression is the result of joining two child expressions
    Join(Box<ScopedLogicalExpr>, Box<ScopedLogicalExpr>),
}

/// Represents an expression during the logical planning phase.
///
/// This combines a DataFusion logical expression with data source, result type and input type
/// coercion information
#[derive(Debug)]
pub struct ScopedLogicalExpr {
    /// the definition of the datafusion that should be applied to the input data
    logical_expr: Expr,

    /// the type that the expression will produce.
    ///
    /// this is used during planning to check for cases where certain operations/expressions may be
    /// invalid for a given input and to ensure any input types that require coercion are correctly
    /// casted.
    ///
    /// note: type checking during planning is best-effort and there are some expressions where the
    /// expression's type validity cannot be guaranteed before we see the data. this is especially
    /// true for expressions involving AnyValues (attributes/logs body).
    pub expr_type: ExprLogicalType,

    /// identifies the source for the incoming data
    source: LogicalExprDataSource,

    /// flag for whether the type of expression requires that dictionary encoding is removed from
    /// the input columns.
    ///
    /// For example, arrow's numeric compute kernels do not work on dictionary encoded primitive
    /// arrays, so arithmetic expressions require converting the columns to the non-dict encoded
    /// arrow type.
    //
    // TODO: it would be cleaner to just have custom expression impl we could add to the plan to
    // remove dictionary encoding from some column, instead of passing this flag down and doing it
    // during projection.
    requires_dict_downcast: bool,
}

impl ScopedLogicalExpr {
    /// Convert the logical expression (used during planning) into the physical expression
    /// which is used during evaluation.
    ///
    /// Note that for now the actual conversion of the underlying datafusion [`Expr`] to the
    /// `ScopedPhysicalExpr` happens lazily when we actually receive the incoming batch.
    fn into_physical(self) -> Result<ScopedPhysicalExpr> {
        let source = match self.source {
            LogicalExprDataSource::DataSource(scope) => {
                PhysicalExprDataSource::DataSource(Rc::new(scope))
            }
            LogicalExprDataSource::Join(left, right) => PhysicalExprDataSource::Join(
                Box::new(left.into_physical()?),
                Box::new(right.into_physical()?),
            ),
        };
        let projection = Projection::try_new(&self.logical_expr)?;

        Ok(ScopedPhysicalExpr {
            source,
            logical_expr: self.logical_expr,
            physical_expr: None, // computed when data received
            projection,
            projection_opts: ProjectionOptions {
                downcast_dicts: self.requires_dict_downcast,
            },
        })
    }
}

/// Logical planner that converts AST expressions into ScopedLogicalExpr.
#[derive(Default)]
pub(crate) struct ExprLogicalPlanner {}

impl ExprLogicalPlanner {
    pub fn plan_scalar_expr(
        &self,
        scalar_expression: &ScalarExpression,
    ) -> Result<ScopedLogicalExpr> {
        match scalar_expression {
            ScalarExpression::Source(source_scalar_expr) => {
                let value_accessor = source_scalar_expr.get_value_accessor();
                let column_accessor = ColumnAccessor::try_from(value_accessor)?;

                match column_accessor {
                    ColumnAccessor::ColumnName(column_name) => {
                        let field_type = root_field_type(&column_name).ok_or_else(|| {
                            Error::InvalidPipelineError {
                                cause: format!("unknown field {column_name} on root record batch"),
                                query_location: Some(
                                    source_scalar_expr.get_query_location().clone(),
                                ),
                            }
                        })?;
                        Ok(ScopedLogicalExpr {
                            logical_expr: col(column_name),
                            requires_dict_downcast: false,
                            source: LogicalExprDataSource::DataSource(DataScope::Root),
                            expr_type: field_type,
                        })
                    }
                    ColumnAccessor::StructCol(column_name, struct_field_name) => {
                        let field_type =
                            nested_struct_field_type(&struct_field_name).ok_or_else(|| Error::InvalidPipelineError {
                                cause: format!("unknown field {struct_field_name} on {column_name} struct column"),
                                query_location: Some(
                                    source_scalar_expr.get_query_location().clone(),
                                ),
                            })?;
                        Ok(ScopedLogicalExpr {
                            logical_expr: col(column_name).field(struct_field_name),
                            requires_dict_downcast: false,
                            source: LogicalExprDataSource::DataSource(DataScope::Root),
                            expr_type: field_type,
                        })
                    }
                    ColumnAccessor::Attributes(attrs_id, key) => Ok(ScopedLogicalExpr {
                        logical_expr: col(VALUE_COLUMN_NAME),
                        requires_dict_downcast: false,
                        source: LogicalExprDataSource::DataSource(DataScope::Attributes(
                            attrs_id, key,
                        )),
                        expr_type: ExprLogicalType::AnyValue,
                    }),
                }
            }
            ScalarExpression::Static(static_scalar_expr) => {
                let (logical_expr, expr_type) = match static_scalar_expr {
                    StaticScalarExpression::Integer(int_expr) => {
                        (lit(int_expr.get_value()), ExprLogicalType::ScalarInt)
                    }
                    StaticScalarExpression::Double(double_expr) => {
                        (lit(double_expr.get_value()), ExprLogicalType::Float64)
                    }
                    StaticScalarExpression::Boolean(bool_expr) => {
                        (lit(bool_expr.get_value()), ExprLogicalType::Boolean)
                    }
                    StaticScalarExpression::String(string_expr) => {
                        (lit(string_expr.get_value()), ExprLogicalType::String)
                    }
                    _ => {
                        return Err(Error::NotYetSupportedError {
                            message: format!(
                                "static scalar expression type not yet supported: {:?}",
                                static_scalar_expr
                            ),
                        });
                    }
                };

                Ok(ScopedLogicalExpr {
                    logical_expr,
                    expr_type,
                    source: LogicalExprDataSource::DataSource(DataScope::StaticScalar),
                    requires_dict_downcast: false,
                })
            }
            ScalarExpression::Math(math_scalar_expr) => match math_scalar_expr {
                MathScalarExpression::Add(binary_math_expr) => {
                    self.plan_binary_math_expr(binary_math_expr, Operator::Plus)
                }
                MathScalarExpression::Subtract(binary_math_expr) => {
                    self.plan_binary_math_expr(binary_math_expr, Operator::Minus)
                }
                MathScalarExpression::Multiply(binary_math_expr) => {
                    self.plan_binary_math_expr(binary_math_expr, Operator::Multiply)
                }
                MathScalarExpression::Divide(binary_math_expr) => {
                    self.plan_binary_math_expr(binary_math_expr, Operator::Divide)
                }
                MathScalarExpression::Modulus(binary_math_expr) => {
                    self.plan_binary_math_expr(binary_math_expr, Operator::Modulo)
                }
                other_math_expr => Err(Error::NotYetSupportedError {
                    message: format!("math expression not yet supported {other_math_expr:?}"),
                }),
            },
            other_expr => Err(Error::NotYetSupportedError {
                message: format!("expression not yet supported {other_expr:?}"),
            }),
        }
    }

    fn plan_binary_math_expr(
        &self,
        binary_math_expr: &BinaryMathematicalScalarExpression,
        operator: Operator,
    ) -> Result<ScopedLogicalExpr> {
        // Recursively plan left and right sub-expressions
        let mut left = self.plan_scalar_expr(binary_math_expr.get_left_expression())?;
        let mut right = self.plan_scalar_expr(binary_math_expr.get_right_expression())?;

        let expr_type = coerce_arithmetic(&mut left, &mut right).ok_or_else(|| {
            Error::InvalidPipelineError {
                cause: format!(
                    "could not coerce types for arithmetic: left type {:?}, right type {:?}",
                    left.expr_type, right.expr_type
                ),
                query_location: Some(binary_math_expr.get_query_location().clone()),
            }
        })?;

        // determine if we can execute the binary expression without joining data from a different
        // data scope. We'd be able to do this when the left/right side either have the same input
        // RecordBatch & row order or if one/both sides are a scalar.
        //
        // for example, we had an expression like:
        // `attributes["x"] * 2` or `observed_timestamp_unix_nano - timestamp_unix_nano`.
        let possible_combined_expr_scope = match (&left.source, &right.source) {
            (
                LogicalExprDataSource::DataSource(left_scope),
                LogicalExprDataSource::DataSource(right_scope),
            ) => left_scope
                .can_combine(right_scope)
                .then_some(if !left_scope.is_scalar() {
                    left_scope
                } else {
                    right_scope
                }),
            _ => None,
        };

        if let Some(combined_scope) = possible_combined_expr_scope {
            Ok(ScopedLogicalExpr {
                logical_expr: Expr::BinaryExpr(BinaryExpr::new(
                    Box::new(left.logical_expr),
                    operator,
                    Box::new(right.logical_expr),
                )),
                source: LogicalExprDataSource::DataSource(combined_scope.clone()),
                expr_type,
                requires_dict_downcast: true,
            })
        } else {
            Ok(ScopedLogicalExpr {
                logical_expr: Expr::BinaryExpr(BinaryExpr::new(
                    Box::new(col(LEFT_COLUMN_NAME)),
                    operator,
                    Box::new(col(RIGHT_COLUMN_NAME)),
                )),
                source: LogicalExprDataSource::Join(Box::new(left), Box::new(right)),
                expr_type,
                requires_dict_downcast: true,
            })
        }
    }
}

/// Physical planner that converts ScopedLogicalExpr into ScopedPhysicalExpr.
///
/// This is just a thin wrapper that delegates to ScopedLogicalExpr::into_physical().
/// Could potentially be removed, but provides a clear separation of concerns.
#[derive(Default)]
pub(crate) struct ExprPhysicalPlanner {}

impl ExprPhysicalPlanner {
    /// Converts a ScopedLogicalExpr into an executable ScopedPhysicalExpr.
    pub fn plan(&self, logical_expr: ScopedLogicalExpr) -> Result<ScopedPhysicalExpr> {
        logical_expr.into_physical()
    }
}

/// A node in the expression tree used for expression evaluation.
///
/// This encapsulates a datafusion PhysicalExpr that evaluates some section of the overall
/// expression tree (the section delineation being expressions where a single, scoped `RecordBatch`
/// can be used as a source without doing any joins).
///
/// This type is responsible for organizing source data into this single input `RecordBatch`, which
/// it does in one of three ways:
/// - select the appropriate data from the incoming OTAP batch
/// - recursively evaluate left/right child expressions and join them
/// - create a dummy empty record batch (special case for scalar-only expressions)
///
pub(crate) struct ScopedPhysicalExpr {
    /// Identifier of the data source from which the input to the PhysicalExpr will be crafted
    source: PhysicalExprDataSource,

    /// The datafusion PhysicalExpr that computes this segment of the expression tree. This is
    /// planned lazily from the logical expression when we receive data (because an actual Arrow
    /// is needed to do the planning)
    physical_expr: Option<PhysicalExprRef>,

    /// The logical representation of this segment of the expression tree. Used to lazily plan the
    /// physical expression
    logical_expr: Expr,

    /// This projection will attempt to select the required columns from the input record batch in
    /// the correct order before evaluating the physical expression. This is necessary because OTAP
    /// record batches are not guaranteed to always have the same set of columns in the same order
    /// across subsequent batches, but this consistent schema is expected by the physical expr.
    pub(crate) projection: Projection,

    /// Options for projection, including whether to remove dictionary encoding (which is required
    /// for arrow numeric compute kernels).
    pub(crate) projection_opts: ProjectionOptions,
}

/// Identifies the source for the input to the physical expression
enum PhysicalExprDataSource {
    /// Source the data from the incoming OTAP record batch
    DataSource(Rc<DataScope>),

    /// Source the data by evaluating left/right child expressions and joining the results
    Join(Box<ScopedPhysicalExpr>, Box<ScopedPhysicalExpr>),
}

/// To evaluate expressions that only produce scalar values, we need to pass some RecordBatch into
/// the call to PhysicalExpr::evaluate. We just pass a static empty record batch.
pub(crate) static SCALAR_RECORD_BATCH_INPUT: LazyLock<RecordBatch> =
    LazyLock::new(|| RecordBatch::new_empty(Arc::new(Schema::new(Vec::<Field>::new()))));

impl ScopedPhysicalExpr {
    pub fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_context: &SessionContext,
    ) -> Result<Option<PhysicalExprEvalResult>> {
        let (source_rb, result_data_scope) = match &mut self.source {
            PhysicalExprDataSource::DataSource(data_scope_id) => {
                let input_rb = match data_scope_id.as_ref() {
                    DataScope::Root => otap_batch.root_record_batch().map(Cow::Borrowed),
                    DataScope::Attributes(attrs_id, key) => {
                        let attrs_payload_type = match *attrs_id {
                            AttributesIdentifier::Root => match otap_batch.root_payload_type() {
                                ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
                                ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
                                _ => ArrowPayloadType::MetricAttrs,
                            },
                            AttributesIdentifier::NonRoot(payload_type) => payload_type,
                        };

                        otap_batch
                            .get(attrs_payload_type)
                            .map(|rb| {
                                Self::try_project_attrs(
                                    rb,
                                    key.as_str(),
                                    self.projection_opts.downcast_dicts,
                                )
                            })
                            .transpose()?
                            .flatten()
                            .map(Cow::Owned)
                    }
                    DataScope::StaticScalar => {
                        Some(Cow::Borrowed(SCALAR_RECORD_BATCH_INPUT.deref()))
                    }
                };

                (input_rb, Rc::clone(data_scope_id))
            }
            PhysicalExprDataSource::Join(left, right) => {
                let left_result = left.execute(otap_batch, session_context)?;
                let right_result = right.execute(otap_batch, session_context)?;
                match (left_result, right_result) {
                    (Some(left_result), Some(right_result)) => {
                        let (joined_rb, result_data_scope) =
                            join(&left_result, &right_result, otap_batch)?;
                        (Some(Cow::Owned(joined_rb)), result_data_scope)
                    }
                    _ => return Ok(None),
                }
            }
        };

        let (source_rb, projected_rb) = match source_rb {
            Some(rb) => {
                // project the source record batch into the schema expected by the physical expr
                let projected = if *result_data_scope != DataScope::StaticScalar {
                    match self
                        .projection
                        .project_with_options(&rb, &self.projection_opts)?
                    {
                        Some(projected) => projected,
                        None => return Ok(None),
                    }
                } else {
                    // don't project for scalar record batch, as it's just a placeholder with no columns
                    rb.as_ref().clone()
                };
                (rb, projected)
            }
            None => {
                // the source was not present, return None indicating the expression is evaluated
                // as null for the entire input
                return Ok(None);
            }
        };

        // evaluate the expression
        let result_vals = self.evaluate_on_batch(session_context, &projected_rb)?;

        Ok(Some(PhysicalExprEvalResult::new(
            result_vals,
            result_data_scope,
            &source_rb,
        )))
    }

    pub(crate) fn evaluate_on_batch(
        &mut self,
        session_context: &SessionContext,
        record_batch: &RecordBatch,
    ) -> Result<ColumnarValue> {
        // plan the physical expressions from logical expression. This happens lazily the first
        // time we receive a non-null batch
        if self.physical_expr.is_none() {
            let session_state = session_context.state();
            let df_schema = DFSchema::try_from(record_batch.schema_ref().as_ref().clone())?;
            let physical_expr = create_physical_expr(
                &self.logical_expr,
                &df_schema,
                session_state.execution_props(),
            )?;
            self.physical_expr = Some(physical_expr);
        }

        // evaluate the expression
        // safety: we've just initialized physical_expr, so it's safe to expect here
        let result_vals = self
            .physical_expr
            .as_ref()
            .expect("physical expr initialized")
            .evaluate(record_batch)?;

        Ok(result_vals)
    }

    /// Filters the record batch by key, and then projects the column containing values of the
    /// type for this attribute to a column called "values".
    ///
    /// For example, if we had an input batch like:
    /// key:        ["a", "a", "b", "b"]
    /// type:       [1, 1, 1, 1] // type 1 = str
    /// str:        ["x", "x", y", "z"]
    /// parent_id:  [0, 1, 0, 1]
    ///
    /// If the "key" argument to this function was "b", the result would be:
    /// value:     ["y", "z"]
    /// parent_id: [0, 1]
    ///
    // TODO - we're making an assumptions here that will need to be later revisited. We assume
    // if a type is present for some key, then all attributes for this key have the same type
    // Normally this would be the case and this is definitely best practice, eventually we'll
    // need to relax this assumption for the sake of correctness.
    fn try_project_attrs(
        record_batch: &RecordBatch,
        key: &str,
        downcast_dicts: bool,
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
            return Ok(None);
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

        // Find the first non-null type value
        let type_value = type_col
            .iter()
            .find_map(|v| v)
            .ok_or_else(|| Error::ExecutionError {
                cause: "No non-null type value found in filtered attributes".to_string(),
            })?;

        let type_value = AttributeValueType::try_from(type_value).map_err(|_e| Error::ExecutionError {
            cause:  format!("invalid record batch. Found invalid value in attributes type column: {type_value}")
        })?;

        // Based on type value, select the appropriate value column
        let value_array = match type_value {
            AttributeValueType::Str => filtered_batch.column_by_name(consts::ATTRIBUTE_STR),
            AttributeValueType::Int => filtered_batch.column_by_name(consts::ATTRIBUTE_INT),
            AttributeValueType::Double => filtered_batch.column_by_name(consts::ATTRIBUTE_DOUBLE),
            AttributeValueType::Bool => filtered_batch.column_by_name(consts::ATTRIBUTE_BOOL),
            AttributeValueType::Bytes => filtered_batch.column_by_name(consts::ATTRIBUTE_BYTES),
            AttributeValueType::Empty => return Ok(None),
            AttributeValueType::Map | AttributeValueType::Slice => {
                return Err(Error::NotYetSupportedError {
                    message:
                        "expression evaluation on non-scalar type attributes (Map/Slice) not yet supported".into()
                    ,
                });
            }
        };

        let value_array = value_array.cloned().ok_or_else(|| Error::ExecutionError {
            cause: format!("Missing values column for type {type_value:?}",),
        })?;

        // Build new schema with parent_id (if present) and value column renamed to "value"
        let mut fields = Vec::new();
        let mut columns = Vec::new();

        let parent_id_col = filtered_batch
            .column_by_name(consts::PARENT_ID)
            .cloned()
            .ok_or_else(|| Error::ExecutionError {
                cause: "Invalid attributes record batch: missing values parent_id column".into(),
            })?;
        fields.push(Arc::new(Field::new(
            consts::PARENT_ID,
            parent_id_col.data_type().clone(),
            false,
        )));
        columns.push(parent_id_col.clone());

        // Add the value column renamed to "value"
        fields.push(Arc::new(Field::new(
            VALUE_COLUMN_NAME,
            value_array.data_type().clone(),
            true,
        )));
        columns.push(value_array);

        if downcast_dicts {
            Projection::try_downcast_dicts(&mut fields, &mut columns)?;
        }

        let schema = Arc::new(Schema::new(fields));
        let projected_batch = RecordBatch::try_new(schema, columns)?;

        Ok(Some(projected_batch))
    }
}

/// Result of evaluating some physical expression scoped to a given data scope.
///
/// This structure contains the resulting array of values, plus identifiers such as data scope and
/// a set of IDs to help identify to which row the resulting values correspond.
///
/// For example, if we had
/// - values: ["a", "b" ... ]
/// - data_domain: DataDomain::Attributes,
/// - parent_ids: Some([0, 1 ...])
///
/// This would indicate that log/trace/metric row with ID 0 corresponds to value "a", and the row
/// with ID 1 corresponds to value "b", and so on.
#[derive(Debug)]
pub(crate) struct PhysicalExprEvalResult {
    /// expression evaluation result values
    pub values: ColumnarValue,

    /// identifies with which arrow record batch should be associated, as well as which rows were
    /// selected (in the case of attributes)
    pub data_scope: Rc<DataScope>,

    // ID columns populated from the source data
    ids: Option<ArrayRef>,
    parent_ids: Option<ArrayRef>,
    scope_ids: Option<ArrayRef>,
    resource_ids: Option<ArrayRef>,
}

impl PhysicalExprEvalResult {
    pub fn new(values: ColumnarValue, data_scope: Rc<DataScope>, source: &RecordBatch) -> Self {
        let is_root = *data_scope == DataScope::Root;

        let mut result = Self {
            values,
            data_scope,
            ids: source.column_by_name(consts::ID).cloned(),
            parent_ids: source.column_by_name(consts::PARENT_ID).cloned(),
            scope_ids: None,
            resource_ids: None,
        };

        if is_root {
            if let Ok(Some(resource_ids)) = get_optional_array_from_struct_array_from_record_batch(
                source,
                consts::RESOURCE,
                consts::ID,
            ) {
                result.resource_ids = Some(Arc::clone(resource_ids))
            }

            if let Ok(Some(scope_ids)) = get_optional_array_from_struct_array_from_record_batch(
                source,
                consts::SCOPE,
                consts::ID,
            ) {
                result.scope_ids = Some(Arc::clone(scope_ids))
            }
        }

        result
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use arrow::array::{Float64Array, Int32Array, Int64Array, StructArray, UInt8Array};
    use arrow::compute::take;
    use data_engine_expressions::{
        BinaryMathematicalScalarExpression, IntegerScalarExpression, QueryLocation,
        SourceScalarExpression, StaticScalarExpression, StringScalarExpression, ValueAccessor,
    };
    use otap_df_pdata::{
        otap::Logs,
        proto::{
            OtlpProtoMessage,
            opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue},
            opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
            opentelemetry::resource::v1::Resource,
        },
        testing::round_trip::{otlp_to_otap, to_logs_data},
    };

    use crate::consts::{ATTRIBUTES_FIELD_NAME, RESOURCES_FIELD_NAME, SCOPE_FIELD_NAME};
    use crate::pipeline::Pipeline;

    fn run_scalar_expr_test(
        input_expr: ScalarExpression,
        input_data: &OtapArrowRecords,
    ) -> Option<ColumnarValue> {
        let planner = ExprLogicalPlanner {};
        let logical_expr = planner.plan_scalar_expr(&input_expr).unwrap();
        let mut physical_expr = logical_expr.into_physical().unwrap();
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr.execute(input_data, &session_ctx).unwrap();
        result.map(|result| result.values)
    }

    fn run_scalar_expr_failure_test(
        input_expr: ScalarExpression,
        input_data: &OtapArrowRecords,
    ) -> Error {
        let planner = ExprLogicalPlanner {};
        let logical_expr = planner.plan_scalar_expr(&input_expr).unwrap();
        let mut physical_expr = logical_expr.into_physical().unwrap();
        let session_ctx = Pipeline::create_session_context();
        physical_expr.execute(input_data, &session_ctx).unwrap_err()
    }

    fn run_scalar_expr_success_test(
        input_expr: ScalarExpression,
        input_data: &OtapArrowRecords,
        expected_result: ArrayRef,
    ) {
        let result = run_scalar_expr_test(input_expr, input_data);
        match &result {
            Some(ColumnarValue::Array(arr)) => {
                assert_eq!(arr.as_ref(), expected_result.as_ref())
            }
            otherwise => {
                panic!("expected Some(ColumnarValue({expected_result:?})), got {otherwise:?}")
            }
        }
    }

    #[test]
    fn test_expr_eval_static_scalar() {
        // Plan the scalar expression
        let planner = ExprLogicalPlanner {};
        let static_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 99),
        ));

        let logical_expr = planner.plan_scalar_expr(&static_expr).unwrap();

        // Convert to physical
        let mut physical_expr = logical_expr.into_physical().unwrap();

        // Execute
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr.execute(&otap_batch, &session_ctx);

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
    fn test_expr_eval_source_column() {
        let input_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_TEXT,
                )),
            )]),
        ));

        let logs = to_logs_data(vec![
            LogRecord::build().severity_text("ERROR").finish(),
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        // get the expected column, which is the column we're accessing
        let logs = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let input_col = logs.column_by_name(consts::SEVERITY_TEXT).unwrap();
        run_scalar_expr_success_test(input_expr, &otap_batch, input_col.clone());
    }

    #[test]
    fn test_expr_eval_struct_source_column() {
        let input_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        // get the expected column
        let logs = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        let scope_col = logs.column_by_name(consts::SCOPE).unwrap();
        let input_col = scope_col
            .as_any()
            .downcast_ref::<StructArray>()
            .unwrap()
            .column_by_name(consts::NAME)
            .unwrap();

        run_scalar_expr_success_test(input_expr, &otap_batch, input_col.clone());
    }

    #[test]
    fn test_expr_eval_attr_value() {
        let input_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        // get the expected column
        let logs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        let input_col = logs.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        let expected_col = take(input_col, &Int32Array::from(vec![1, 2, 4]), None).unwrap();

        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_column_scalar() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let right_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
        ));

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        let logs = to_logs_data(vec![
            LogRecord::build().severity_number(10).finish(),
            LogRecord::build()
                .severity_number(30)
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .severity_number(20)
                .severity_text("DEBUG")
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        let expected_col = Arc::new(Int32Array::from_iter_values(vec![12, 32, 22]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_scalar_root_column() {
        let left_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 2),
        ));

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        let logs = to_logs_data(vec![
            LogRecord::build().severity_number(10).finish(),
            LogRecord::build()
                .severity_number(30)
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .severity_number(20)
                .severity_text("DEBUG")
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let expected_col = Arc::new(Int32Array::from_iter_values(vec![12, 32, 22]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_root_attributes() {
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
        let expected_col = Arc::new(Int64Array::from(vec![4, 12, 9]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_root_with_attribute() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
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
        let expected_col = Arc::new(Int64Array::from(vec![13, 29, 32]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_attribute_with_root() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

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
        let expected_col = Arc::new(Int64Array::from(vec![13, 29, 32]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_binary_arithmetic_expr_additional_operators_int_values() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let binary_expr = BinaryMathematicalScalarExpression::new(
            QueryLocation::new_fake(),
            left_expr,
            right_expr,
        );

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
                    KeyValue::new("k1", AnyValue::new_int(9)),
                    KeyValue::new("k2", AnyValue::new_int(3)),
                ])
                .severity_number(20)
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(2)),
                    KeyValue::new("k2", AnyValue::new_int(7)),
                ])
                .severity_text("DEBUG")
                .severity_number(30)
                .finish(),
        ]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        let test_cases = vec![
            (
                MathScalarExpression::Subtract(binary_expr.clone()),
                vec![2, 6, -5],
            ),
            (
                MathScalarExpression::Multiply(binary_expr.clone()),
                vec![3, 27, 14],
            ),
            (
                MathScalarExpression::Divide(binary_expr.clone()),
                vec![3, 3, 0],
            ),
            (
                MathScalarExpression::Modulus(binary_expr.clone()),
                vec![0, 0, 2],
            ),
        ];
        for (math_expr, expected) in test_cases {
            let input_expr = ScalarExpression::Math(math_expr);
            let expected_col = Arc::new(Int64Array::from(expected));
            run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
        }
    }

    #[test]
    fn test_binary_arithmetic_expr_additional_operators_float_values() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let binary_expr = BinaryMathematicalScalarExpression::new(
            QueryLocation::new_fake(),
            left_expr,
            right_expr,
        );

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(3)),
                    KeyValue::new("k2", AnyValue::new_double(1)),
                ])
                .severity_number(10)
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(9)),
                    KeyValue::new("k2", AnyValue::new_double(3)),
                ])
                .severity_number(20)
                .severity_text("INFO")
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(2)),
                    KeyValue::new("k2", AnyValue::new_double(7)),
                ])
                .severity_text("DEBUG")
                .severity_number(30)
                .finish(),
        ]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        let test_cases = vec![
            (
                MathScalarExpression::Subtract(binary_expr.clone()),
                vec![2.0, 6.0, -5.0],
            ),
            (
                MathScalarExpression::Multiply(binary_expr.clone()),
                vec![3.0, 27.0, 14.0],
            ),
            (
                MathScalarExpression::Divide(binary_expr.clone()),
                vec![3.0, 3.0, 2.0 / 7.0],
            ),
            (
                MathScalarExpression::Modulus(binary_expr.clone()),
                vec![0.0, 0.0, 2.0],
            ),
        ];
        for (math_expr, expected) in test_cases {
            let input_expr = ScalarExpression::Math(math_expr);
            let expected_col = Arc::new(Float64Array::from(expected));
            run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
        }
    }

    #[test]
    fn test_expr_eval_arithmetic_root_to_nonroot_attrs() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), RESOURCES_FIELD_NAME),
                )),
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

        let logs = LogsData::new(vec![
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(10))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build().severity_number(3).finish(),
                        LogRecord::build().severity_number(5).finish(),
                    ],
                )],
                ..Default::default()
            },
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(20))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![LogRecord::build().severity_number(7).finish()],
                )],
                ..Default::default()
            },
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let expected_col = Arc::new(Int64Array::from(vec![13, 15, 27]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_nonroot_attrs_to_root() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), RESOURCES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "k1"),
                )),
            ]),
        ));

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        let logs = LogsData::new(vec![
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(10))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build().severity_number(3).finish(),
                        LogRecord::build().severity_number(5).finish(),
                    ],
                )],
                ..Default::default()
            },
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(20))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![LogRecord::build().severity_number(7).finish()],
                )],
                ..Default::default()
            },
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let expected_col = Arc::new(Int64Array::from(vec![13, 15, 27]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_attrs_to_nonroot_attrs() {
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
                    StringScalarExpression::new(QueryLocation::new_fake(), RESOURCES_FIELD_NAME),
                )),
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

        let logs = LogsData::new(vec![
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(10))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(1))])
                            .severity_number(3)
                            .finish(),
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(2))])
                            .severity_number(5)
                            .finish(),
                    ],
                )],
                ..Default::default()
            },
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(20))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(7))])
                            .severity_number(7)
                            .finish(),
                    ],
                )],
                ..Default::default()
            },
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let expected_col = Arc::new(Int64Array::from(vec![11, 12, 27]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_nonroot_attrs_to_root_attrs() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), RESOURCES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "k1"),
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
                    StringScalarExpression::new(QueryLocation::new_fake(), "k2"),
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

        let logs = LogsData::new(vec![
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(10))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(1))])
                            .severity_number(3)
                            .finish(),
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(2))])
                            .severity_number(5)
                            .finish(),
                    ],
                )],
                ..Default::default()
            },
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(20))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(7))])
                            .severity_number(7)
                            .finish(),
                    ],
                )],
                ..Default::default()
            },
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let expected_col = Arc::new(Int64Array::from(vec![11, 12, 27]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_expr_eval_arithmetic_deeply_nested_expr() {
        let resource_attrs_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), RESOURCES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "k1"),
                )),
            ]),
        ));

        let attrs_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let root_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Math(MathScalarExpression::Add(
                    BinaryMathematicalScalarExpression::new(
                        QueryLocation::new_fake(),
                        resource_attrs_expr,
                        attrs_expr,
                    ),
                )),
                root_expr,
            ),
        ));

        let logs = LogsData::new(vec![
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(10))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(1))])
                            .severity_number(3)
                            .finish(),
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(2))])
                            .severity_number(5)
                            .finish(),
                    ],
                )],
                ..Default::default()
            },
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(20))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(7))])
                            .severity_number(7)
                            .finish(),
                    ],
                )],
                ..Default::default()
            },
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        // get the expected column
        let expected_col = Arc::new(Int64Array::from(vec![14, 17, 34]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_arithmetic_expr_with_changing_column_orders() {
        // basically this test is ensuring that we correctly project the input batches
        // before evaluating the expression

        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::DROPPED_ATTRIBUTES_COUNT,
                )),
            )]),
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

        let logs = to_logs_data(vec![
            LogRecord::build()
                .dropped_attributes_count(10u32)
                .attributes(vec![KeyValue::new("k1", AnyValue::new_int(3))])
                .finish(),
            LogRecord::build()
                .dropped_attributes_count(20u32)
                .attributes(vec![KeyValue::new("k1", AnyValue::new_int(7))])
                .finish(),
        ]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        let logs_input_1 = otap_batch.get(ArrowPayloadType::Logs).unwrap();

        run_scalar_expr_success_test(
            input_expr.clone(),
            &otap_batch,
            Arc::new(Int64Array::from(vec![13, 27])),
        );

        // send a second batch where the column order will have changed
        let logs = to_logs_data(vec![
            LogRecord::build()
                .severity_text("info")
                .dropped_attributes_count(30u32)
                .attributes(vec![KeyValue::new("k1", AnyValue::new_int(3))])
                .finish(),
            LogRecord::build()
                .severity_text("debug")
                .dropped_attributes_count(40u32)
                .attributes(vec![KeyValue::new("k1", AnyValue::new_int(7))])
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let logs_input_2 = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        // ensure the column isn't in the same location
        assert_ne!(
            logs_input_1
                .schema()
                .index_of(consts::DROPPED_ATTRIBUTES_COUNT)
                .unwrap(),
            logs_input_2
                .schema()
                .index_of(consts::DROPPED_ATTRIBUTES_COUNT)
                .unwrap()
        );

        // ensure we succeed to evaluate the expression despite the column order changing
        run_scalar_expr_success_test(
            input_expr,
            &otap_batch,
            Arc::new(Int64Array::from(vec![33, 47])),
        );
    }

    #[test]
    fn test_two_subsequent_batches_with_attributes_same_name_different_types() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        // batch 1 - attributes are ints
        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_int(3)),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(7)),
                    KeyValue::new("k2", AnyValue::new_int(7)),
                ])
                .finish(),
        ]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        run_scalar_expr_success_test(
            input_expr.clone(),
            &otap_batch,
            Arc::new(Int64Array::from(vec![6, 14])),
        );

        // batch 2 - attributes are floats
        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(4.0)),
                    KeyValue::new("k2", AnyValue::new_double(4.0)),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(5.0)),
                    KeyValue::new("k2", AnyValue::new_double(7.0)),
                ])
                .finish(),
        ]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        run_scalar_expr_success_test(
            input_expr.clone(),
            &otap_batch,
            Arc::new(Float64Array::from(vec![8.0, 12.0])),
        );
    }

    #[test]
    fn test_deeply_nested_arithmetic_expr_that_forces_root_to_root_join() {
        // in this expression, root+resource.attrs should evaluate first, then we
        // which should produce an intermediate result with the same row order as
        // the input root batch, which means we can do a special join that just concats
        // the vec of columns together from both input sides

        let resource_attrs_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), RESOURCES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "k1"),
                )),
            ]),
        ));

        let root_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                ScalarExpression::Math(MathScalarExpression::Add(
                    BinaryMathematicalScalarExpression::new(
                        QueryLocation::new_fake(),
                        resource_attrs_expr,
                        root_expr.clone(),
                    ),
                )),
                root_expr,
            ),
        ));

        let logs = LogsData::new(vec![
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(10))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(1))])
                            .severity_number(3)
                            .finish(),
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(2))])
                            .severity_number(5)
                            .finish(),
                    ],
                )],
                ..Default::default()
            },
            ResourceLogs {
                resource: Some(
                    Resource::build()
                        .attributes(vec![KeyValue::new("k1", AnyValue::new_int(20))])
                        .finish(),
                ),

                scope_logs: vec![ScopeLogs::new(
                    InstrumentationScope {
                        name: "scope1".into(),
                        ..Default::default()
                    },
                    vec![
                        LogRecord::build()
                            .attributes(vec![KeyValue::new("k2", AnyValue::new_int(7))])
                            .severity_number(7)
                            .finish(),
                    ],
                )],
                ..Default::default()
            },
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        // get the expected column
        let expected_col = Arc::new(Int64Array::from(vec![16, 20, 34]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_arithmetic_null_propagation_null_values_no_join() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let right_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
        ));
        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        let logs = to_logs_data(vec![
            LogRecord::build().severity_number(1).finish(),
            LogRecord::build().finish(),
            LogRecord::build()
                .severity_number(6)
                .severity_text("DEBUG")
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let expected_col = Arc::new(Int32Array::from_iter([Some(4), None, Some(9)]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_arithmetic_null_propagation_null_column_no_join() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let right_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
        ));
        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        // no severity number column
        let logs = to_logs_data(vec![
            LogRecord::build().finish(),
            LogRecord::build().finish(),
            LogRecord::build().finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        let result = run_scalar_expr_test(input_expr, &otap_batch);
        assert!(result.is_none(), "expected result to be None")
    }

    #[test]
    fn test_arithmetic_null_propagation_null_batch_no_join() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "x"),
                )),
            ]),
        ));

        let right_expr = ScalarExpression::Static(StaticScalarExpression::Integer(
            IntegerScalarExpression::new(QueryLocation::new_fake(), 3),
        ));
        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        // no attributes record batch column
        let logs = to_logs_data(vec![
            LogRecord::build().finish(),
            LogRecord::build().finish(),
            LogRecord::build().finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));
        assert!(otap_batch.get(ArrowPayloadType::LogAttrs).is_none());
        let result = run_scalar_expr_test(input_expr, &otap_batch);
        assert!(result.is_none(), "expected result to be None")
    }

    #[test]
    fn test_arithmetic_null_propagation_null_values_on_right_of_join() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_int(1)),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("k1", AnyValue::new_int(9))])
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
        let expected_col = Arc::new(Int64Array::from_iter([Some(4), None, Some(9)]));
        run_scalar_expr_success_test(input_expr, &otap_batch, expected_col.clone());
    }

    #[test]
    fn test_arithmetic_null_propagation_null_result_on_join() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_NUMBER,
                )),
            )]),
        ));

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        // severity number not present
        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_int(1)),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("k1", AnyValue::new_int(9))])
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
        let result = run_scalar_expr_test(input_expr, &otap_batch);
        assert!(result.is_none());
    }

    #[test]
    fn test_null_propagation_no_attributes_existing() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), ATTRIBUTES_FIELD_NAME),
                )),
                ScalarExpression::Static(StaticScalarExpression::String(
                    StringScalarExpression::new(QueryLocation::new_fake(), "nonexist"),
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
                    StringScalarExpression::new(QueryLocation::new_fake(), "k2"),
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

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_int(1)),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("k1", AnyValue::new_int(9))])
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
        let result = run_scalar_expr_test(input_expr, &otap_batch);
        assert!(result.is_none());
    }

    #[test]
    fn test_null_propagation_empty_attributes() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        let input_expr = ScalarExpression::Math(MathScalarExpression::Add(
            BinaryMathematicalScalarExpression::new(
                QueryLocation::new_fake(),
                left_expr,
                right_expr,
            ),
        ));

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue { value: None }),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k2", AnyValue::new_int(7)),
                    KeyValue::new("k2", AnyValue { value: None }),
                ])
                .severity_text("DEBUG")
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        // ensure the attribute values are what we expect
        let log_attrs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        let type_col = log_attrs
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(type_col.value(1), AttributeValueType::Empty as u8);
        assert_eq!(type_col.value(3), AttributeValueType::Empty as u8);

        let result = run_scalar_expr_test(input_expr, &otap_batch);
        assert!(result.is_none());
    }

    #[test]
    fn test_arithmetic_type_mismatch_caught_planning() {
        let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
            QueryLocation::new_fake(),
            ValueAccessor::new_with_selectors(vec![ScalarExpression::Static(
                StaticScalarExpression::String(StringScalarExpression::new(
                    QueryLocation::new_fake(),
                    consts::SEVERITY_TEXT,
                )),
            )]),
        ));

        let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
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

        // Check it returns an error when it detects at planning time that it won't be able to add
        // these two fields.
        let planner = ExprLogicalPlanner {};
        let err = planner
            .plan_scalar_expr(&ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    left_expr.clone(),
                    right_expr.clone(),
                ),
            )))
            .unwrap_err();

        let err_msg = err.to_string();
        assert!(
            err_msg.contains(
                "could not coerce types for arithmetic: left type String, right type AnyValue"
            ),
            "Unexpected error message: {:?}",
            err_msg
        );

        // check it with swapped left/right arguments (for good measure):
        let planner = ExprLogicalPlanner {};
        let err = planner
            .plan_scalar_expr(&ScalarExpression::Math(MathScalarExpression::Add(
                BinaryMathematicalScalarExpression::new(
                    QueryLocation::new_fake(),
                    right_expr,
                    left_expr,
                ),
            )))
            .unwrap_err();

        let err_msg = err.to_string();
        assert!(
            err_msg.contains(
                "could not coerce types for arithmetic: left type AnyValue, right type String"
            ),
            "Unexpected error message: {:?}",
            err_msg
        )
    }

    #[test]
    fn test_arithmetic_runtime_any_value_type_mismatches() {
        // check that adding types that cannot be added fails ar runtime as a fallback for when
        // we can't detect at compile time that the types are invalid. In this case, we're doing
        // something like attributes["x"] + attributes["y"], where we don't know what type are
        // these attribute values.

        let logs = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_double(4.0)),
                    KeyValue::new("k3", AnyValue::new_string("a")),
                    KeyValue::new("k4", AnyValue::new_bool(false)),
                    KeyValue::new("k5", AnyValue::new_bytes(b"a")),
                ])
                .finish(),
        ]);

        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs));

        let test_cases = vec![
            ("k1", "k2"),
            ("k1", "k3"),
            ("k1", "k4"),
            ("k1", "k5"),
            ("k2", "k3"),
            ("k2", "k4"),
            ("k2", "k5"),
            ("k3", "k4"),
            ("k3", "k5"),
            ("k4", "k5"),
        ];

        fn check_arithmetic_fails(
            left: &'static str,
            right: &'static str,
            otap_batch: &OtapArrowRecords,
        ) {
            let left_expr = ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            ATTRIBUTES_FIELD_NAME,
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), left),
                    )),
                ]),
            ));

            let right_expr = ScalarExpression::Source(SourceScalarExpression::new(
                QueryLocation::new_fake(),
                ValueAccessor::new_with_selectors(vec![
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(
                            QueryLocation::new_fake(),
                            ATTRIBUTES_FIELD_NAME,
                        ),
                    )),
                    ScalarExpression::Static(StaticScalarExpression::String(
                        StringScalarExpression::new(QueryLocation::new_fake(), right),
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

            let err = run_scalar_expr_failure_test(input_expr, otap_batch);
            let err_msg = err.to_string();
            assert!(
                err_msg.contains("Invalid arithmetic operation"),
                "unexpected error. left key = {left}, right key = {right}, error_msg = {err_msg:?}"
            );
        }

        for (left, right) in test_cases {
            check_arithmetic_fails(left, right, &otap_batch);
            check_arithmetic_fails(right, left, &otap_batch);
        }
    }
}
