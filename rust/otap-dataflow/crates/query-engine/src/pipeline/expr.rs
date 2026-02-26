// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

// TODO we'll use this eventually
#![allow(dead_code)]

//! Implementation of expression evaluation for OTAP (OpenTelemetry Arrow Protocol) batches.
//!

use std::borrow::Cow;
use std::ops::Deref;
use std::sync::{Arc, LazyLock};

use arrow::array::{Array, ArrayRef, NullArray, RecordBatch, StringArray, StructArray};
use arrow::compute::filter_record_batch;
use arrow::compute::kernels::cmp::eq;
use arrow::datatypes::{Field, Fields, Schema};
use data_engine_expressions::{
    BinaryMathematicalScalarExpression, Expression, MathScalarExpression, ScalarExpression,
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

mod join;
mod types;

pub(crate) const VALUE_COLUMN_NAME: &str = "value";
pub(crate) const LEFT_COLUMN_NAME: &str = "left";
pub(crate) const RIGHT_COLUMN_NAME: &str = "right";

/// Identifies which OTAP RecordBatch domain an expression operates on.
///
/// OTAP batches contain multiple RecordBatches. This enum identifies which batch
/// provides the data for an expression:
/// - Root: The main telemetry batch (Logs/Spans/Metrics)
/// - Attributes: Attribute batches (LogAttrs, ResourceAttrs, etc.) filtered by key
/// - StaticScalar: Constant values that don't come from any batch
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Debug, Clone)]
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

#[derive(Debug)]
enum LogicalExprDataSource {
    DataSource(LogicalDataDomain),
    Join(Box<LogicalDomainExpr>, Box<LogicalDomainExpr>),
}

/// Represents an expression during the logical planning phase.
///
/// This combines a DataFusion logical expression with domain information.
/// When expressions span multiple domains (e.g., Root + Attributes), a parent-child
/// structure is used where the parent references the child via col("child").
#[derive(Debug)]
struct LogicalDomainExpr {
    logical_expr: Expr,
    expr_type: ExprLogicalType,
    source: LogicalExprDataSource,
    requires_dict_downcast: bool,
}

impl LogicalDomainExpr {
    fn into_physical(self) -> Result<PhysicalDomainExpr> {
        let source = match self.source {
            LogicalExprDataSource::DataSource(domain) => {
                PhysicalExprDataSource::DataSource(domain.domain_id)
            }
            LogicalExprDataSource::Join(left, right) => PhysicalExprDataSource::Join(
                Box::new(left.into_physical()?),
                Box::new(right.into_physical()?),
            ),
        };
        let projection = Projection::try_new(&self.logical_expr)?;

        Ok(PhysicalDomainExpr {
            source,
            logical_expr: self.logical_expr,
            physical_expr: None,
            projection,
            projection_opts: ProjectionOptions {
                downcast_dicts: self.requires_dict_downcast,
            },
        })
    }
}

/// Logical planner that converts AST expressions into LogicalDomainExpr.
///
/// This is Phase 1 of the planning process. It walks the expression AST and
/// determines the data domain for each sub-expression, then builds a
/// DataFusion logical expression that can operate on that domain.
struct ExprLogicalPlanner {}

impl ExprLogicalPlanner {
    fn plan_scalar_expr(
        &mut self,
        scalar_expression: &ScalarExpression,
    ) -> Result<LogicalDomainExpr> {
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
                        Ok(LogicalDomainExpr {
                            logical_expr: col(column_name),
                            requires_dict_downcast: false,
                            source: LogicalExprDataSource::DataSource(LogicalDataDomain {
                                domain_id: DataDomainId::Root,
                            }),
                            expr_type: field_type,
                        })
                    }
                    ColumnAccessor::StructCol(column_name, struct_field_name) => {
                        let field_type =
                            root_field_type(&struct_field_name).ok_or_else(|| Error::InvalidPipelineError {
                                cause: format!("unknown field {struct_field_name} on {column_name} struct column"),
                                query_location: Some(
                                    source_scalar_expr.get_query_location().clone(),
                                ),
                            })?;
                        Ok(LogicalDomainExpr {
                            logical_expr: col(column_name).field(struct_field_name),
                            requires_dict_downcast: false,
                            source: LogicalExprDataSource::DataSource(LogicalDataDomain {
                                domain_id: DataDomainId::Root,
                            }),
                            expr_type: field_type,
                        })
                    }
                    ColumnAccessor::Attributes(attrs_id, key) => Ok(LogicalDomainExpr {
                        logical_expr: col(VALUE_COLUMN_NAME),
                        requires_dict_downcast: false,
                        source: LogicalExprDataSource::DataSource(LogicalDataDomain {
                            domain_id: DataDomainId::Attributes(attrs_id, key),
                        }),
                        expr_type: ExprLogicalType::AnyValue,
                    }),
                }
            }
            ScalarExpression::Static(static_scalar_expr) => {
                use data_engine_expressions::StaticScalarExpression as SSE;

                let (logical_expr, expr_type) = match static_scalar_expr {
                    SSE::Integer(int_expr) => {
                        (lit(int_expr.get_value()), ExprLogicalType::ScalarInt)
                    }
                    SSE::Double(double_expr) => {
                        (lit(double_expr.get_value()), ExprLogicalType::Float64)
                    }
                    SSE::Boolean(bool_expr) => {
                        (lit(bool_expr.get_value()), ExprLogicalType::Boolean)
                    }
                    SSE::String(string_expr) => {
                        (lit(string_expr.get_value()), ExprLogicalType::String)
                    }
                    // SSE::Null(_) => {
                    //     // Create a null literal of unknown type
                    //     lit(datafusion::scalar::ScalarValue::Null)
                    // }
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
                    logical_expr,
                    expr_type,
                    source: LogicalExprDataSource::DataSource(LogicalDataDomain {
                        domain_id: DataDomainId::StaticScalar,
                    }),
                    requires_dict_downcast: false,
                })
            }
            ScalarExpression::Math(math_scalar_expr) => match math_scalar_expr {
                MathScalarExpression::Add(binary_math_expr) => {
                    // Recursively plan left and right sub-expressions
                    let mut left = self.plan_scalar_expr(binary_math_expr.get_left_expression())?;
                    let mut right =
                        self.plan_scalar_expr(binary_math_expr.get_right_expression())?;

                    let expr_type = coerce_arithmetic(&mut left, &mut right).ok_or_else(|| {
                        // TODO - test for this code path
                        Error::InvalidPipelineError {
                            cause: format!(
                                "could not coerce types for arithmetic: left type {:?}, right type {:?}",
                                left.expr_type,
                                right.expr_type
                            ),
                            query_location: Some(math_scalar_expr.get_query_location().clone())
                        }
                    })?;

                    // TODO comment about what we're doing here
                    let possible_combined_expr_domain = match (&left.source, &right.source) {
                        (
                            LogicalExprDataSource::DataSource(left_domain),
                            LogicalExprDataSource::DataSource(right_domain),
                        ) => left_domain.can_combine(right_domain).then_some(
                            if !left_domain.is_scalar() {
                                left_domain
                            } else {
                                right_domain
                            },
                        ),
                        _ => None,
                    };

                    if let Some(combined_domain) = possible_combined_expr_domain {
                        Ok(LogicalDomainExpr {
                            logical_expr: Expr::BinaryExpr(BinaryExpr::new(
                                Box::new(left.logical_expr),
                                Operator::Plus,
                                Box::new(right.logical_expr),
                            )),
                            source: LogicalExprDataSource::DataSource(combined_domain.clone()),
                            expr_type,
                            requires_dict_downcast: true,
                        })
                    } else {
                        Ok(LogicalDomainExpr {
                            logical_expr: Expr::BinaryExpr(BinaryExpr::new(
                                Box::new(col(LEFT_COLUMN_NAME)),
                                Operator::Plus,
                                Box::new(col(RIGHT_COLUMN_NAME)),
                            )),
                            source: LogicalExprDataSource::Join(Box::new(left), Box::new(right)),
                            expr_type,
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

struct PhysicalDomainExpr {
    source: PhysicalExprDataSource,

    logical_expr: Expr,

    physical_expr: Option<PhysicalExprRef>,

    projection: Projection,

    projection_opts: ProjectionOptions,
}

enum PhysicalExprDataSource {
    DataSource(DataDomainId),
    Join(Box<PhysicalDomainExpr>, Box<PhysicalDomainExpr>),
}

#[derive(Debug)]
pub(crate) struct PhysicalExprEvalResult {
    values: ColumnarValue,
    ids: Option<ArrayRef>,
    data_domain: DataDomainId,
    parent_ids: Option<ArrayRef>,
    scope_ids: Option<ArrayRef>,
    resource_ids: Option<ArrayRef>,
}

/// To evaluate expressions that only produce scalar values, we need to pass some RecordBatch into
/// the call to PhysicalExpr::evaluate. We just pass a static empty record batch.
static SCALAR_RECORD_BATCH_INPUT: LazyLock<RecordBatch> =
    LazyLock::new(|| RecordBatch::new_empty(Arc::new(Schema::new(Vec::<Field>::new()))));

impl PhysicalDomainExpr {
    fn execute(
        &mut self,
        otap_batch: &OtapArrowRecords,
        session_context: &SessionContext,
        include_ids: bool,
    ) -> Result<Option<PhysicalExprEvalResult>> {
        // TODO need to avoid cloning the data domain Id here
        let (source_rb, result_data_domain) = match &mut self.source {
            PhysicalExprDataSource::DataSource(data_domain_id) => {
                let input_rb = match data_domain_id {
                    DataDomainId::Root => otap_batch.root_record_batch().map(Cow::Borrowed),
                    DataDomainId::Attributes(attrs_id, key) => {
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
                    DataDomainId::StaticScalar => {
                        Some(Cow::Borrowed(SCALAR_RECORD_BATCH_INPUT.deref()))
                    }
                };

                (input_rb, data_domain_id.clone())
            }
            PhysicalExprDataSource::Join(left, right) => {
                // TODO no unwrap - need null propagation
                let left_result = left.execute(otap_batch, session_context, true)?.unwrap();
                let right_result = right.execute(otap_batch, session_context, true)?.unwrap();

                let (joined_rb, result_data_domain) =
                    join(&left_result, &right_result, otap_batch)?;
                (Some(Cow::Owned(joined_rb)), result_data_domain)
            }
        };

        let source_rb = match source_rb {
            Some(rb) => rb,
            None => return Ok(None),
        };

        let projected_rb = if result_data_domain != DataDomainId::StaticScalar {
            match self
                .projection
                .project_with_options(&source_rb, &self.projection_opts)
            {
                Some(rb) => Cow::Owned(rb),
                None => return Ok(None),
            }
        } else {
            // TODO there must be a helper for this
            match &source_rb {
                Cow::Borrowed(rb) => Cow::Borrowed(*rb),
                Cow::Owned(rb) => Cow::Borrowed(rb),
            }
        };

        if self.physical_expr.is_none() {
            let session_state = session_context.state();
            let df_schema = DFSchema::try_from(projected_rb.schema_ref().as_ref().clone())?;
            let physical_expr = create_physical_expr(
                &self.logical_expr,
                &df_schema,
                session_state.execution_props(),
            )?;
            self.physical_expr = Some(physical_expr);
        }
        let result_vals = self
            .physical_expr
            .as_ref()
            .unwrap()
            .evaluate(&projected_rb)?;

        // TODO this'd be cleaner as a constructor
        let mut result = PhysicalExprEvalResult {
            values: result_vals,
            ids: None,
            parent_ids: None,
            scope_ids: None,
            resource_ids: None,
            // TODO - the only reason this is cloned is b/c we use it below to figure out whether
            // to tack the scope/resource IDs onto the result. There's gotta be a better way
            data_domain: result_data_domain.clone(),
        };

        // TODO -- need to coerce values back into the dict type?

        // TODO test this is returned
        // if include_ids {
        result.ids = source_rb.column_by_name(consts::ID).cloned();
        result.parent_ids = source_rb.column_by_name(consts::PARENT_ID).cloned();

        if result_data_domain == DataDomainId::Root {
            result.scope_ids = get_optional_array_from_struct_array_from_record_batch(
                &source_rb,
                consts::SCOPE,
                consts::ID,
            )
            .unwrap()
            .cloned();
            result.resource_ids = get_optional_array_from_struct_array_from_record_batch(
                &source_rb,
                consts::SCOPE,
                consts::ID,
            )
            .unwrap()
            .cloned();
        }
        // }
        Ok(Some(result))
    }

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
            VALUE_COLUMN_NAME,
            value_array.data_type().clone(),
            true,
        )));
        columns.push(value_array);

        if downcast_dicts {
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
    use data_engine_expressions::{
        IntegerScalarExpression, QueryLocation, StaticScalarExpression, StringScalarExpression,
        ValueAccessor,
    };
    // TODO ugly imports
    use crate::consts::{ATTRIBUTES_FIELD_NAME, RESOURCES_FIELD_NAME, SCOPE_FIELD_NAME};
    use crate::pipeline::Pipeline;
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

    fn run_scalar_expr_test(
        input_expr: ScalarExpression,
        input_data: &OtapArrowRecords,
    ) -> Option<ColumnarValue> {
        let mut planner = ExprLogicalPlanner {};
        let logical_expr = planner.plan_scalar_expr(&input_expr).unwrap();
        let mut physical_expr = logical_expr.into_physical().unwrap();
        let session_ctx = Pipeline::create_session_context();
        let result = physical_expr
            .execute(&input_data, &session_ctx, false)
            .unwrap();
        result.map(|result| result.values)
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

    // TODO the name the LLM generated for these tests is a bit hokey ...
    // "_to_physical_expr" ugh
    // I think we can

    #[test]
    fn test_planner_static_to_physical_execute() {
        // Plan the scalar expression
        let mut planner = ExprLogicalPlanner {};
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
    fn test_planner_root_struct_to_physical_execute() {
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
    fn test_planner_attribute_source_to_physical_execute() {
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
    fn test_planner_binary_expr_column_to_scalar() {
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
    fn test_planner_binary_expr_scalar_to_column() {
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
    fn test_planner_binary_expr_same_attributes() {
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
    fn test_planner_binary_expr_root_to_attribute() {
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
    fn test_planner_binary_expr_attribute_to_root() {
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
    fn test_planner_binary_expr_root_to_nonroot_attrs() {
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
    fn test_planner_binary_expr_nonroot_attrs_to_root() {
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
    fn test_planner_binary_expr_attrs_to_nonroot_attrs() {
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
    fn test_planner_binary_expr_nonroot_attrs_to_root_attrs() {
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

    // TODO - this is kind of just a smoke test, should we have it be more purposeful?
    #[test]
    fn test_planner_binary_expr_deeply_nested_expr() {
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
}
