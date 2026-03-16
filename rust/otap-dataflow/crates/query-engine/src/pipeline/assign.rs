// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains the implementation of a [`PipelineStage`] for assigning the result of
//! the evaluation of an expression to a column in an OTAP record batch.
//!
//! It evaluates the "set" stage in queries such as:
//! ```text
//! logs | set severity_text = "INFO"
//! ```
//!
//! Note: implementation is currently a work in progress, and not all destinations are supported
//!

use std::borrow::Cow;
use std::ops::Deref;
use std::rc::Rc;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, BooleanArray, DictionaryArray, Float64Array, Int64Array, NullArray,
    RecordBatch, StringArray, UInt8Array,
};
use arrow::compute::{cast, kernels::cmp::neq, take};
use arrow::datatypes::{DataType, Field, Schema, UInt16Type};
use async_trait::async_trait;
use data_engine_expressions::{
    Expression, QueryLocation, ScalarExpression, SourceScalarExpression,
};
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::ColumnarValue;
use datafusion::prelude::SessionContext;
use datafusion::scalar::ScalarValue;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::error::Error as PdataError;
use otap_df_pdata::otap::Logs;
use otap_df_pdata::otap::transform::concatenate::{Cardinality, FieldInfo, estimate_cardinality};
use otap_df_pdata::otlp::attributes::AttributeValueType;
use otap_df_pdata::schema::consts;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::expr::join::{JoinExec, RootToAttributesJoin};
use crate::pipeline::expr::types::{
    ExprLogicalType, root_field_supports_dict_encoding, root_field_type,
};
use crate::pipeline::expr::{
    DataScope, ExprLogicalPlanner, ExprPhysicalPlanner, PhysicalExprEvalResult,
    SCALAR_RECORD_BATCH_INPUT, ScopedLogicalExpr, ScopedPhysicalExpr, VALUE_COLUMN_NAME,
};
use crate::pipeline::planner::ColumnAccessor;
use crate::pipeline::project::{ProjectedSchemaColumn, Projection};
use crate::pipeline::state::ExecutionState;

/// Pipeline stage for assigning the result of an expression evaluation to an OTAP column
pub(crate) struct AssignPipelineStage {
    /// Identifier of the destination column
    dest_column: ColumnAccessor,

    /// Data Scope of the destination column.
    ///
    /// This is used at execution time to join the results which may have been computed using data
    /// that has a different row order from the destination column. Although this type can be
    /// computed from dest_column, we create it up-front to avoid cloning data during evaluation
    dest_scope: Rc<DataScope>,

    /// Expression that will produce the data to be assigned to the destination
    source: ScopedPhysicalExpr,

    /// When this pipeline stage is used in a nested pipeline that processes attributes, it may be
    /// applying an expression that references the virtual "value" column. This flag will be set if
    /// the expression references this column.
    projection_contains_value_column: bool,
}

impl AssignPipelineStage {
    /// Create a new instance of [`AssignPipelineStage`]
    pub fn try_new(dest: &SourceScalarExpression, source: &ScalarExpression) -> Result<Self> {
        let logical_planner = ExprLogicalPlanner::default();
        let source_logical_plan = logical_planner.plan_scalar_expr(source)?;

        let dest_column = ColumnAccessor::try_from(dest.get_value_accessor())?;
        validate_assign(
            &dest_column,
            dest.get_query_location(),
            &source_logical_plan,
        )?;

        let physical_planner = ExprPhysicalPlanner::default();
        let physical_expr = physical_planner.plan(source_logical_plan)?;

        let projection_contains_value_column =
            physical_expr
                .projection
                .schema
                .iter()
                .any(|projected_col| match projected_col {
                    ProjectedSchemaColumn::Root(col_name) => col_name == VALUE_COLUMN_NAME,
                    _ => false,
                });

        Ok(Self {
            dest_scope: Rc::new(DataScope::from(&dest_column)),
            dest_column,
            source: physical_expr,
            projection_contains_value_column,
        })
    }

    /// Assign the result of the expression evaluation to a column on the root record batch.
    fn assign_to_root(
        &self,
        mut otap_batch: OtapArrowRecords,
        mut eval_result: PhysicalExprEvalResult,
        dest_column_name: &str,
    ) -> Result<OtapArrowRecords> {
        let root_batch = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => {
                // nothing to do
                return Ok(otap_batch);
            }
        };

        let expected_column_logical_type = root_field_type(dest_column_name)
            // safety: this will only return None if the destination column does not exist in OTAP
            // data model, but this has been validated in the constructor of this type, so it's
            // safe to expect here
            .expect("dest column found");

        let expected_column_data_type = expected_column_logical_type
            .datatype()
            // safety: this will only return None if the logical data type for the field is
            // ambiguous, which is the case for attributes/AnyValues, but all the fields on the
            // root batch are known/un-ambiguous, so this will return Some and is safe to expect
            .expect("dest column data type");

        // coerce static scalar int" if the result was a static scalar integer, it will have been
        // produced as an int64 by default, however the expression tree doesn't actually specify
        // the type, so we assume the type should have matched the expected type here and cast it
        let mut eval_result_column_type = eval_result.values.data_type();
        if eval_result.data_scope.as_ref() == &DataScope::StaticScalar
            && eval_result_column_type.is_integer()
            && expected_column_data_type.is_integer()
        {
            eval_result.values = eval_result
                .values
                .cast_to(&expected_column_data_type, None)?;
            eval_result_column_type = expected_column_data_type.clone();
        }

        // check that the result type of the expr eval can be assigned to this field
        let mut type_compatible = expected_column_data_type == eval_result_column_type;

        // if it's dict encoded, check if the dict values match the expected type
        let column_supports_dict_encoding = root_field_supports_dict_encoding(dest_column_name);
        if !type_compatible && column_supports_dict_encoding {
            if let DataType::Dictionary(_, dict_val_type) = &eval_result_column_type {
                if dict_val_type.as_ref() == &expected_column_data_type {
                    type_compatible = true
                }
            }
        }

        // if result is not type compatible, return error
        if !type_compatible {
            return Err(Error::ExecutionError {
                cause: format!(
                    "cannot assign expression result of type {:?} to column expecting type {:?}",
                    eval_result_column_type, expected_column_data_type
                ),
            });
        }

        // convert the expression evaluation result to an array, with the correct dict encoding if
        // the destination column supports it
        let mut values = eval_result_to_array(
            &eval_result.values,
            column_supports_dict_encoding,
            root_batch.num_rows(),
        )?;

        // align the rows in the new values with the rows in the root batch, if not already aligned
        let already_aligned = eval_result.data_scope.is_scalar()
            || eval_result.data_scope.as_ref() == self.dest_scope.as_ref();

        if !already_aligned {
            // if we're here, it means we have received a column value that has the row order
            // of something other than the root attribute batch, basically meaning the result was
            // computed from attributes. We'll need to join the result's values column to the root
            // column to get the values in the correct order ...

            let DataScope::Attributes(attrs_id, _) = eval_result.data_scope.as_ref() else {
                // safety: if the data_scope were anything other than attributes, we'd have taken
                // the if branch (not the else branch) above when we checked if the data was
                // already aligned
                unreachable!("unexpected data_scope")
            };

            // create a JoinExec implementation that computes joined indices of values to root on
            // `root.id == attrs.parent_id` and use this to take rows from the result in order
            let join_exec = RootToAttributesJoin::new(*attrs_id);
            let vals_take_indices = join_exec.rows_to_take(
                &PhysicalExprEvalResult::new(
                    ColumnarValue::Scalar(ScalarValue::Null), // empty placeholder,
                    self.dest_scope.clone(),
                    root_batch,
                ),
                &eval_result,
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )?;

            values = take(&values, &vals_take_indices, None)?;
        };

        // replace the root record batch with the new one
        let root_payload_type = otap_batch.root_payload_type();
        otap_batch.set(
            root_payload_type,
            try_upsert_column(dest_column_name, values, root_batch)?,
        );

        Ok(otap_batch)
    }

    /// try to assign an all-null column to the root record batch. In practice, this just means
    /// removing the column from the record batch. This will return an error if it turns out the
    /// column is not nullable.
    fn assign_null_root_column(
        &self,
        mut otap_batch: OtapArrowRecords,
        dest_column_name: &str,
    ) -> Result<OtapArrowRecords> {
        let root_batch = match otap_batch.root_record_batch() {
            Some(rb) => rb,
            None => {
                // nothing to do
                return Ok(otap_batch);
            }
        };

        // remove the column if it exists because it's all null result
        // Note: once again we're assuming that if the field is nullable that it is also optional
        let schema = root_batch.schema_ref();
        let maybe_found_column = schema.fields().find(dest_column_name);
        if let Some((column_index, field)) = maybe_found_column {
            if field.is_nullable() {
                let mut new_root_batch = root_batch.clone();
                _ = new_root_batch.remove_column(column_index);
                otap_batch.set(otap_batch.root_payload_type(), new_root_batch);
            } else {
                return Err(Error::ExecutionError {
                    cause: format!(
                        "cannot assign null result to non-nullable column {dest_column_name}"
                    ),
                });
            }
        }

        Ok(otap_batch)
    }
}

#[async_trait(?Send)]
impl PipelineStage for AssignPipelineStage {
    async fn execute(
        &mut self,
        otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_options: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        let eval_result = self.source.execute(&otap_batch, session_context)?;

        match eval_result {
            Some(eval_result) => match &self.dest_column {
                ColumnAccessor::ColumnName(col_name) => {
                    self.assign_to_root(otap_batch, eval_result, col_name)
                }
                other_dest => {
                    return Err(Error::NotYetSupportedError {
                        message: format!(
                            "assignment to column destination {:?} not yet supported",
                            other_dest
                        ),
                    });
                }
            },
            None => match &self.dest_column {
                ColumnAccessor::ColumnName(col_name) => {
                    self.assign_null_root_column(otap_batch, col_name)
                }
                other_dest => {
                    return Err(Error::NotYetSupportedError {
                        message: format!(
                            "assignment to column destination {:?} not yet supported",
                            other_dest
                        ),
                    });
                }
            },
        }
    }

    /// Assigns the result of this pipeline stage's source expression to a column on the attributes
    /// record batch.
    ///
    /// This will be called to evaluate the `set` operator call in the context of a nested pipeline
    /// applied to attributes. For example:
    /// ```text
    /// logs | apply attributes { set value = "hello" }
    /// ```
    ///
    /// ## Limitations:
    ///
    /// Currently there are some limitations on the types of expressions which can be evaluated:
    ///
    /// 1. assignment destination can only be the attribute "value". Updating attribute key/type
    /// using this pipeline expression is not yet supported. This means it will not evaluate
    /// expressions such as:
    /// ```text
    /// logs | apply attributes { set key = "hello" } // not yet supported!
    /// logs | apply attributes { set type = 1 } // not yet supported!
    /// ```
    ///
    /// 2. both source/destination cannot reference specific values columns. Although attributes
    /// `RecordBatch`s may contain additional columns such as `type`, `int`, `float`, `str`, etc.
    /// it is not yet supported use these columns as the assignment destination, not is it yet
    /// supported to reference these columns in the source expression. This means it will not
    /// evaluate expressions such as:
    /// ```text
    /// logs | apply attributes { set str = "hello" } // not yet supported!
    /// logs | apply attributes { set value = int * 2 } // not yet supported!
    /// ```
    ///
    /// Effectively what this means is the only types of expressions that will be evaluated are
    /// those which reference the virtual "value" column and/or static literals.
    async fn execute_on_attributes(
        &mut self,
        attrs_record_batch: RecordBatch,
        session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_options: &mut ExecutionState,
    ) -> Result<RecordBatch> {
        if attrs_record_batch.num_rows() == 0 {
            // nothing to do
            return Ok(attrs_record_batch);
        }

        let input_schema = attrs_record_batch.schema_ref();

        // determine the input attribute type
        let (type_column_index, _) = input_schema
            .fields()
            .find(consts::ATTRIBUTE_TYPE)
            .ok_or_else(|| Error::ExecutionError {
                cause: PdataError::ColumnNotFound {
                    name: consts::ATTRIBUTE_TYPE.into(),
                }
                .to_string(),
            })?;
        let type_column = attrs_record_batch.column(type_column_index);
        let type_column = type_column
            .as_any()
            .downcast_ref::<UInt8Array>()
            .ok_or_else(|| Error::ExecutionError {
                cause: PdataError::ColumnDataTypeMismatch {
                    name: consts::ATTRIBUTE_TYPE.into(),
                    expect: DataType::UInt8,
                    actual: type_column.data_type().clone(),
                }
                .to_string(),
            })?;

        if type_column.null_count() != 0 {
            // even though we only look at the first non-null value to determine the input type,
            // we'll be strict here validate that there aren't any nulls
            return Err(Error::ExecutionError {
                cause: "attribute record batch type column should not contain nulls".into(),
            });
        }

        // safety: we've already checked the batch is not empty, and that there aren't any nulls
        // in this column, which means we should be safe to expect at least one non-null type
        let input_attr_type = type_column
            .iter()
            .flatten()
            .next()
            .expect("non-empty batch");

        let input_attr_type =
            AttributeValueType::try_from(input_attr_type).map_err(|e| Error::ExecutionError {
                cause: format!("invalid attribute type {input_attr_type}: {e}"),
            })?;

        // check if every value is the same type - if not, we may have problems evaluating the
        // expression (if the value is used in the expression).
        let all_rows_same_attr_type =
            neq(type_column, &UInt8Array::new_scalar(input_attr_type as u8))?.true_count() == 0;

        // create the record batch that will be the input to the datafusion physical expression..
        // if the expression involves the attribute value (e.g. `value + 2`), we produce a record
        // batch with a single column which is the "value", otherwise, the input is an empty record
        // batch. We do this because we are currently assuming the only types of expressions we
        // support are those involving the attribute values (referenced as the virtual "value")
        // column, or expressions involving static constants which don't need input columns.
        let projected_rb = if self.projection_contains_value_column {
            if !all_rows_same_attr_type {
                // if not all the attribute types are the same, we can't determine a single value
                // column to use in the projection, so return an error. In practice, the batch
                // should be split apart before this pipeline stage using other operators to ensure
                // we only have one value type.
                return Err(Error::ExecutionError {
                    cause: "All input rows for attribute assignment must have the same type \
                        if value used in expression"
                        .into(),
                });
            }

            // try to access the values column
            let values_column_name = match input_attr_type {
                AttributeValueType::Bool => Some(consts::ATTRIBUTE_BOOL),
                AttributeValueType::Double => Some(consts::ATTRIBUTE_DOUBLE),
                AttributeValueType::Int => Some(consts::ATTRIBUTE_INT),
                AttributeValueType::Str => Some(consts::ATTRIBUTE_STR),
                AttributeValueType::Empty => None,
                other => {
                    return Err(Error::NotYetSupportedError {
                        message: format!(
                            "Setting attributes of type {:?} in nested pipeline not yet supported",
                            other
                        ),
                    });
                }
            };

            let values_column =
                values_column_name.and_then(|col| attrs_record_batch.column_by_name(col));

            let values_column: ArrayRef = match values_column {
                Some(col) => Arc::clone(col),
                None => {
                    // here the values column is missing, which basically means the attributes
                    // were all null. We'll create an all null array as a placeholder column.
                    let len = attrs_record_batch.num_rows();
                    match input_attr_type {
                        AttributeValueType::Bool => Arc::new(BooleanArray::new_null(len)),
                        AttributeValueType::Double => Arc::new(Float64Array::new_null(len)),
                        AttributeValueType::Int => Arc::new(Int64Array::new_null(len)),
                        AttributeValueType::Str => Arc::new(StringArray::new_null(len)),
                        AttributeValueType::Empty => Arc::new(NullArray::new(len)),
                        other => {
                            return Err(Error::NotYetSupportedError {
                                message: format!(
                                    "Setting attributes of type {:?} in nested pipeline not yet supported",
                                    other
                                ),
                            });
                        }
                    }
                }
            };

            // create the input record batch
            let mut fields = vec![Arc::new(Field::new(
                VALUE_COLUMN_NAME,
                values_column.data_type().clone(),
                true,
            ))];
            let mut columns = vec![values_column];

            // remove dict encoding if necessary. This would be needed for certain expressions such
            // as arithmetic
            if self.source.projection_opts.downcast_dicts {
                Projection::try_downcast_dicts(&mut fields, &mut columns)?
            }

            Cow::Owned(RecordBatch::try_new(
                Arc::new(Schema::new(fields)),
                columns,
            )?)
        } else {
            // since the expression does not require the "value" column, we assume that it is an
            // expression involving only static literals, in which case the input can just be an
            // empty record batch
            Cow::Borrowed(SCALAR_RECORD_BATCH_INPUT.deref())
        };

        // evaluate the expression
        let mut result = self
            .source
            .evaluate_on_batch(session_context, &projected_rb)?
            .to_array(attrs_record_batch.num_rows())?;

        // determine the "logical" type of the result (e.g. the array type, or the values if the
        // result happens to be dictionary encoded.
        let mut result_logical_type = result.data_type();
        if let DataType::Dictionary(_, v) = result_logical_type {
            result_logical_type = v.as_ref();
        }

        // prepare insert the result into the record batch by determining the column name and
        // tye result attribute type (e.g. value in "type" column) and whether to support dict
        // encoding for the result column
        let (field_name, supports_dict, result_attr_type) = match result_logical_type {
            DataType::Utf8 => (Some(consts::ATTRIBUTE_STR), true, AttributeValueType::Str),
            DataType::Int64 => (Some(consts::ATTRIBUTE_INT), true, AttributeValueType::Int),
            DataType::Float64 => (
                Some(consts::ATTRIBUTE_DOUBLE),
                false,
                AttributeValueType::Double,
            ),
            DataType::Boolean => (
                Some(consts::ATTRIBUTE_BOOL),
                false,
                AttributeValueType::Bool,
            ),
            DataType::Null => (None, false, AttributeValueType::Empty),
            other => {
                return Err(Error::NotYetSupportedError {
                    message: format!(
                        "Setting attributes of from arrow type {:?} in nested pipeline not yet supported",
                        other
                    ),
                });
            }
        };

        // possibly cast the result into a dict if the type column supports it and if it will fit
        if supports_dict {
            let needs_to_dict = match result.data_type() {
                DataType::Dictionary(k, _) => **k != DataType::UInt16,
                _ => {
                    let field_info = FieldInfo::new_from_array(&result);
                    let cardinality = estimate_cardinality(&field_info);
                    cardinality != Cardinality::GreaterThanU16
                }
            };

            if needs_to_dict {
                result = cast(
                    &result,
                    &DataType::Dictionary(
                        Box::new(DataType::UInt16),
                        Box::new(result.data_type().clone()),
                    ),
                )?
            }
        }

        // create a new record batch including the result column ...

        let field_index = field_name.and_then(|field_name| {
            attrs_record_batch
                .schema()
                .fields()
                .find(field_name)
                .map(|(i, _)| i)
        });

        let mut fields = attrs_record_batch.schema_ref().fields.to_vec();
        let mut columns = attrs_record_batch.columns().to_vec();

        // In OTAP if a column is all null, we don't include it in the batch, so this flag will
        // be used to determine whether the result column is included in the result batch
        let all_nulls = result.null_count() == attrs_record_batch.num_rows();

        if let Some(field_index) = field_index {
            if all_nulls {
                // remove the existing column
                _ = fields.remove(field_index);
                _ = columns.remove(field_index);
            } else {
                // replace the existing column
                fields[field_index] = Arc::new(
                    fields[field_index]
                        .as_ref()
                        .clone()
                        .with_data_type(result.data_type().clone()),
                );
                columns[field_index] = result;
            }
        } else {
            // insert new column
            if !all_nulls && let Some(field_name) = field_name {
                fields.push(Arc::new(Field::new(
                    field_name,
                    result.data_type().clone(),
                    true,
                )));
                columns.push(result);
            }
        }

        // replace the type column if the result may have changed the type for some row
        if result_attr_type != input_attr_type || !all_rows_same_attr_type {
            let new_type_column = UInt8Array::from_iter_values(std::iter::repeat_n(
                result_attr_type as u8,
                attrs_record_batch.num_rows(),
            ));
            columns[type_column_index] = Arc::new(new_type_column)
        }

        Ok(RecordBatch::try_new(
            Arc::new(Schema::new(fields)),
            columns,
        )?)
    }

    fn supports_exec_on_attributes(&self) -> bool {
        true
    }
}

/// Validate that the results of the passed expression can be assigned to the destination.
/// There are multiple validations performed:
///
/// It validates that the destination exists - e.g. that it is a known column in OTAP
///
/// It also validate the types. Specifically it will check that the type could possibly be
/// assigned to the destination. Note: it does not guarantee that the expression will produce
/// a valid type for the assignment. For example, in an expression like:
/// ```text
/// severity_text = attributes["x"]
/// ```
/// This would pass validation because `attributes["x"]` could be a string, which is what the
/// destination `severity_text` expects. However, when this is evaluated we may find that
/// `attributes["x"]` is not a string in which case this would fail at runtime.
///
/// This also validates that there is not ambiguity in the assignment based on the cardinality of
/// the relationship between source and destination. Specifically, if the dest:source relationship
/// is one:many, then we cannot do the assignment because it's unclear which of the many source
/// values should be assigned to the destination row.
///
/// Here is an example of this type of invalid assignment:
/// ```text
/// logs | set resource.attributes["x"] = severity_text
/// ```
/// Because there are many logs with possibly different severities for any given resource, we
/// consider this assignment invalid.
///
fn validate_assign(
    dest_column: &ColumnAccessor,
    dest_query_location: &QueryLocation,
    source_logical_plan: &ScopedLogicalExpr,
) -> Result<()> {
    match dest_column {
        ColumnAccessor::ColumnName(col_name) => {
            // No relationship cardinality validation needs to happen for these columns which
            // are on the root record because they are not one:many with anything else in that
            // could be assigned. Validation in this case only checks the types.

            let dest_type =
                root_field_type(col_name).ok_or_else(|| Error::InvalidPipelineError {
                    cause: format!("cannot assign to non-existent column '{col_name}'"),
                    query_location: Some(dest_query_location.clone()),
                })?;

            let source_type = &source_logical_plan.expr_type;
            if !can_assign_type(&dest_type, source_type) {
                return Err(Error::InvalidPipelineError {
                    cause: format!(
                        "cannot assign expression of type {source_type:?} to type {dest_type:?}"
                    ),
                    query_location: Some(dest_query_location.clone()),
                });
            }
        }
        other_dest => {
            // TODO other assignment destinations will be supported soon
            return Err(Error::NotYetSupportedError {
                message: format!(
                    "assignment to column destination {:?} not yet supported",
                    other_dest
                ),
            });
        }
    }

    Ok(())
}

/// Determine if the source type can be assigned to the destination.
///
/// See comments on [`validate_assign`] for more details about what types are considered compatible
fn can_assign_type(dest_type: &ExprLogicalType, source_type: &ExprLogicalType) -> bool {
    if dest_type == source_type {
        return true;
    }

    // scalar int type can be converted to any integer type
    if dest_type.is_integer() && source_type == &ExprLogicalType::ScalarInt {
        return true;
    }

    match dest_type {
        ExprLogicalType::Boolean
        | ExprLogicalType::String
        | ExprLogicalType::Int64
        | ExprLogicalType::Float64 => source_type == &ExprLogicalType::AnyValue,

        ExprLogicalType::AnyValue => matches!(
            source_type,
            ExprLogicalType::Boolean
                | ExprLogicalType::String
                | ExprLogicalType::Int64
                | ExprLogicalType::Float64
                | ExprLogicalType::AnyValueNumeric
                | ExprLogicalType::ScalarInt
        ),

        // TODO - handle other cases as we support a greater variety of destinations
        _ => false,
    }
}

/// Convert result of expression evaluation into an arrow array with the appropriate dict encoding
/// for the destination column.
fn eval_result_to_array(
    expr_eval_result: &ColumnarValue,
    accept_dict_encoding: bool,
    dest_num_rows: usize,
) -> Result<ArrayRef> {
    match expr_eval_result {
        ColumnarValue::Scalar(scalar_val) => {
            if accept_dict_encoding {
                // create a dictionary with a single value, and all keys selecting this value
                let dict_values = scalar_val.to_array()?;
                let dict_keys = UInt8Array::from_iter_values(std::iter::repeat_n(0, dest_num_rows));
                Ok(Arc::new(DictionaryArray::new(dict_keys, dict_values)))
            } else {
                Ok(scalar_val.to_array_of_size(dest_num_rows)?)
            }
        }
        ColumnarValue::Array(array_vals) => {
            if accept_dict_encoding {
                // here we're going to try to select the smallest dictionary key that could contain
                // all the unique values
                match array_vals.data_type() {
                    DataType::Dictionary(k, v) => match k.as_ref() {
                        DataType::UInt8 => {
                            // already smallest dict size
                            Ok(Arc::clone(array_vals))
                        }
                        DataType::UInt16 => {
                            // check if we can use a smaller dictionary key
                            let values_as_dict = array_vals
                                .as_any()
                                .downcast_ref::<DictionaryArray<UInt16Type>>()
                                .expect("can downcast to dict");
                            if values_as_dict.values().len() <= 256 {
                                // values can fit in a smaller dict
                                Ok(cast(
                                    &array_vals,
                                    &DataType::Dictionary(
                                        Box::new(DataType::UInt8),
                                        Box::new(v.as_ref().clone()),
                                    ),
                                )?)
                            } else {
                                // values already a dict, but won't fit in a smaller dict
                                Ok(Arc::clone(array_vals))
                            }
                        }
                        other_key_type => Err(Error::ExecutionError {
                            cause: format!(
                                "invalid dictionary key in evaluation result {other_key_type:?}"
                            ),
                        }),
                    },
                    _ => {
                        // array is not dictionary encoded -- determine if we should convert it
                        let field_info = FieldInfo::new_from_array(array_vals);
                        let cardinality = estimate_cardinality(&field_info);
                        let key_type = match cardinality {
                            Cardinality::WithinU8 => Some(DataType::UInt8),
                            Cardinality::WithinU16 => Some(DataType::UInt16),
                            _ => None,
                        };

                        if let Some(key_type) = key_type {
                            // convert to smallest dictionary key allowed by cardinality
                            Ok(cast(
                                &array_vals,
                                &DataType::Dictionary(
                                    Box::new(key_type),
                                    Box::new(array_vals.data_type().clone()),
                                ),
                            )?)
                        } else {
                            Ok(Arc::clone(array_vals))
                        }
                    }
                }
            } else {
                // TODO - eventually we may have to remove the dictionary encoding here.
                // however currently the only destinations we support all either support dict
                // encoding, or it's not possible to produce an expression for the column that
                // results in dictionary encoding. If eventually we support int type coercion,
                // we'll need to remove dict encoding here for expressions like:
                // dropped_attributes_count = attributes["x"] // e.g. uint32 <- dict<u16, int64>

                Ok(Arc::clone(array_vals))
            }
        }
    }
}

/// Inserts the column into the record batch if the column does not exist, otherwise replaces the
/// existing column with the new one.
///
/// Note that if the column exists, and is not nullable, but the new column contains nulls, this
/// will return an error
fn try_upsert_column(
    column_name: &str,
    new_column: ArrayRef,
    record_batch: &RecordBatch,
) -> Result<RecordBatch> {
    let mut columns = record_batch.columns().to_vec();
    let schema = record_batch.schema();
    let fields = schema.fields();
    let maybe_found_column = fields.find(column_name);
    let mut fields = fields.to_vec();

    if let Some((target_col_index, current_field)) = maybe_found_column {
        // check that we're not assigning a column with nulls to a non-nullable column
        if !current_field.is_nullable() && new_column.null_count() != 0 {
            return Err(Error::ExecutionError {
                cause: format!("cannot assign null result to non-nullable column {column_name}"),
            });
        }

        // replace field if the datatype has changed. Note, we wont have changed the logical
        // type of the field, but the dictionary encoding may be what has changed
        let needs_field_update = fields[target_col_index].data_type() != new_column.data_type();
        if needs_field_update {
            fields
                .iter_mut()
                .enumerate()
                .for_each(|(curr_index, field)| {
                    if target_col_index == curr_index {
                        let new_field = field
                            .as_ref()
                            .clone()
                            .with_data_type(new_column.data_type().clone());
                        *field = Arc::new(new_field)
                    }
                });
        }

        // replace column
        columns
            .iter_mut()
            .enumerate()
            .for_each(|(curr_index, col)| {
                if target_col_index == curr_index {
                    *col = Arc::clone(&new_column)
                }
            });
    } else {
        // just insert the new column at the end
        fields.push(Arc::new(Field::new(
            column_name,
            new_column.data_type().clone(),
            // Note: here we're assuming that since the column was missing that it was an
            // optional column which means that it is nullable
            true,
        )));
        columns.push(new_column)
    }

    Ok(RecordBatch::try_new(
        Arc::new(Schema::new(fields)),
        columns,
    )?)
}
#[cfg(test)]
mod test {
    use arrow::{compute::kernels::cast, datatypes::DataType};
    use data_engine_kql_parser::{KqlParser, Parser};
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::{
        OtapArrowRecords,
        otap::Logs,
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                arrow::v1::ArrowPayloadType,
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                resource::v1::Resource,
                trace::v1::Span,
            },
        },
        schema::consts,
        testing::round_trip::{otlp_to_otap, to_logs_data, to_traces_data},
    };

    use crate::pipeline::{Pipeline, planner::PipelinePlanner, test::exec_logs_pipeline};

    async fn test_insert_root_column_from_scalar<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().finish(),
            LogRecord::build().finish(),
        ]);
        let result =
            exec_logs_pipeline::<P>("logs | extend severity_text = \"ERROR\"", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 2);
        for logs_record in logs_records {
            assert_eq!(logs_record.severity_text, "ERROR");
        }
    }

    #[tokio::test]
    async fn test_insert_root_column_from_scalar_opl_parser() {
        test_insert_root_column_from_scalar::<OplParser>().await
    }

    #[tokio::test]
    async fn test_insert_root_column_from_scalar_kql_parser() {
        test_insert_root_column_from_scalar::<KqlParser>().await
    }

    async fn test_upsert_root_column_from_scalar<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().finish(),
        ]);
        let result =
            exec_logs_pipeline::<P>("logs | extend severity_text = \"ERROR\"", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 2);
        for logs_record in logs_records {
            assert_eq!(logs_record.severity_text, "ERROR");
        }
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_scalar_opl_parser() {
        test_upsert_root_column_from_scalar::<OplParser>().await
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_scalar_kql_parser() {
        test_upsert_root_column_from_scalar::<KqlParser>().await
    }

    async fn test_insert_root_column_from_int_scalar<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().finish(),
            LogRecord::build().finish(),
        ]);
        let result = exec_logs_pipeline::<P>("logs | extend severity_number = 1", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 2);
        for logs_record in logs_records {
            assert_eq!(logs_record.severity_number, 1);
        }
    }

    #[tokio::test]
    async fn test_insert_root_column_from_int_scalar_opl_parser() {
        test_insert_root_column_from_int_scalar::<OplParser>().await
    }

    #[tokio::test]
    async fn test_insert_root_column_from_int_scalar_kql_parser() {
        test_insert_root_column_from_int_scalar::<KqlParser>().await
    }

    async fn test_insert_root_column_from_other_column<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);

        let result =
            exec_logs_pipeline::<P>("logs | extend event_name = severity_text", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 2);
        assert_eq!(logs_records[0].event_name, "INFO");
        assert_eq!(logs_records[1].event_name, "DEBUG");
    }

    #[tokio::test]
    async fn test_insert_root_column_from_other_column_opl_parser() {
        test_insert_root_column_from_other_column::<OplParser>().await
    }

    #[tokio::test]
    async fn test_insert_root_column_from_other_column_kql_parser() {
        test_insert_root_column_from_other_column::<KqlParser>().await
    }

    async fn test_upsert_root_column_from_other_column<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .severity_text("INFO")
                .event_name("event1")
                .finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);

        let result =
            exec_logs_pipeline::<P>("logs | extend event_name = severity_text", logs_data).await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 2);
        assert_eq!(logs_records[0].event_name, "INFO");
        assert_eq!(logs_records[1].event_name, "DEBUG");
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_other_column_opl_parser() {
        test_upsert_root_column_from_other_column::<OplParser>().await
    }

    #[tokio::test]
    async fn test_upsert_root_column_from_other_column_kql_parser() {
        test_upsert_root_column_from_other_column::<KqlParser>().await
    }

    async fn test_set_root_column_from_attribute<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("event", AnyValue::new_string("hello"))])
                .finish(),
            LogRecord::build()
                .event_name("replaceme")
                .attributes(vec![KeyValue::new("event", AnyValue::new_string("world"))])
                .finish(),
            // no event attribute, result should be ""..
            LogRecord::build().finish(),
            LogRecord::build().event_name("replaceme").finish(),
        ]);

        let result = exec_logs_pipeline::<P>(
            "logs | extend event_name = attributes[\"event\"]",
            logs_data,
        )
        .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 4);
        assert_eq!(logs_records[0].event_name, "hello");
        assert_eq!(logs_records[1].event_name, "world");
        assert_eq!(logs_records[2].event_name, "");
        assert_eq!(logs_records[3].event_name, "");
    }

    #[tokio::test]
    async fn test_set_root_column_from_attribute_opl_parser() {
        test_set_root_column_from_attribute::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_from_attribute_kql_parser() {
        test_set_root_column_from_attribute::<KqlParser>().await
    }

    async fn test_set_root_column_from_arithmetic_expression<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .severity_number(2)
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(1))])
                .finish(),
            LogRecord::build()
                .severity_number(3)
                .attributes(vec![KeyValue::new("x", AnyValue::new_int(2))])
                .finish(),
            LogRecord::build().finish(),
            LogRecord::build().event_name("replaceme").finish(),
        ]);

        // kind of a weird expression in practice, but this is just checking if the expr evaluates
        let result = exec_logs_pipeline::<P>(
            "logs | extend severity_number = 5 + severity_number * 10",
            logs_data,
        )
        .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();

        assert_eq!(logs_records.len(), 4);
        assert_eq!(logs_records[0].severity_number, 25);
        assert_eq!(logs_records[1].severity_number, 35);
        assert_eq!(logs_records[2].severity_number, 0);
        assert_eq!(logs_records[3].severity_number, 0);
    }

    #[tokio::test]
    async fn test_set_root_column_from_arithmetic_expression_opl_parser() {
        test_set_root_column_from_arithmetic_expression::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_from_arithmetic_expression_kql_parser() {
        test_set_root_column_from_arithmetic_expression::<KqlParser>().await
    }

    async fn test_set_root_column_from_non_root_attribute<P: Parser>() {
        let logs_data = LogsData::new(vec![ResourceLogs::new(
            Resource::build().finish(),
            vec![
                ScopeLogs::new(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("a"))])
                        .finish(),
                    vec![LogRecord::build().finish()],
                ),
                ScopeLogs::new(
                    InstrumentationScope::build()
                        .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("b"))])
                        .finish(),
                    vec![LogRecord::build().finish()],
                ),
            ],
        )]);

        let result = exec_logs_pipeline::<P>(
            "logs | extend event_name = instrumentation_scope.attributes[\"attr1\"]",
            logs_data,
        )
        .await;

        let logs_records = result.resource_logs[0].scope_logs[0].log_records.clone();
        assert_eq!(logs_records.len(), 1);
        assert_eq!(logs_records[0].event_name, "a");
        let logs_records = result.resource_logs[0].scope_logs[1].log_records.clone();
        assert_eq!(logs_records.len(), 1);
        assert_eq!(logs_records[0].event_name, "b");
    }

    #[tokio::test]
    async fn test_set_root_column_from_non_root_attribute_opl_parser() {
        test_set_root_column_from_non_root_attribute::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_from_non_root_attribute_kql_parser() {
        test_set_root_column_from_non_root_attribute::<KqlParser>().await
    }

    async fn test_set_root_column_rejects_invalid_type_during_planning<P: Parser>() {
        let pipeline = P::parse("logs | extend event_name = 1").unwrap().pipeline;
        let session_ctx = Pipeline::create_session_context();
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let planner = PipelinePlanner::new();
        let result = planner.plan_stages(&pipeline, &session_ctx, &otap_batch);
        match result {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("cannot assign expression of type ScalarInt to type String"),
                    "unexpected error message: {err_msg:?}"
                )
            }
            Ok(_) => {
                panic!("expected error")
            }
        };
    }

    #[tokio::test]
    async fn test_set_root_column_rejects_invalid_type_during_planning_opl_parser() {
        test_set_root_column_rejects_invalid_type_during_planning::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_rejects_invalid_type_during_planning_kql_parser() {
        test_set_root_column_rejects_invalid_type_during_planning::<KqlParser>().await
    }

    async fn test_set_root_column_rejects_invalid_column_during_planning<P: Parser>() {
        let pipeline = P::parse("logs | extend bad_column = 1").unwrap().pipeline;
        let session_ctx = Pipeline::create_session_context();
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let planner = PipelinePlanner::new();
        let result = planner.plan_stages(&pipeline, &session_ctx, &otap_batch);
        match result {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("cannot assign to non-existent column 'bad_column'"),
                    "unexpected error message: {err_msg:?}"
                )
            }
            Ok(_) => {
                panic!("expected error")
            }
        };
    }

    #[tokio::test]
    async fn test_set_root_column_rejects_invalid_column_during_planning_opl_parser() {
        test_set_root_column_rejects_invalid_column_during_planning::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_column_rejects_invalid_column_during_planning_kql_parser() {
        test_set_root_column_rejects_invalid_column_during_planning::<KqlParser>().await
    }

    async fn test_set_root_invalid_expr_result_type_rejected_at_runtime<P: Parser>() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("attr", AnyValue::new_int(1))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("attr", AnyValue::new_int(1))])
                .finish(),
        ]);

        let pipeline_expr = P::parse("logs | extend event_name = attributes[\"attr\"]")
            .unwrap()
            .pipeline;
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await;

        match result {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains(
                        "Pipeline execution error: cannot assign expression result of type Dictionary(UInt16, Int64) to column expecting type Utf8"
                    ),
                    "unexpected error message: {err_msg:?}"
                )
            }
            Ok(_) => {
                panic!("expected error")
            }
        }
    }

    #[tokio::test]
    async fn test_set_root_invalid_expr_result_type_rejected_at_runtime_opl_parser() {
        test_set_root_invalid_expr_result_type_rejected_at_runtime::<OplParser>().await
    }

    #[tokio::test]
    async fn test_set_root_invalid_expr_result_type_rejected_at_runtime_kql_parser() {
        test_set_root_invalid_expr_result_type_rejected_at_runtime::<KqlParser>().await
    }

    #[tokio::test]
    async fn test_assign_empty_batch() {
        let pipeline_expr = OplParser::parse("logs | set severity_number = 1")
            .unwrap()
            .pipeline;
        let input = OtapArrowRecords::Logs(Logs::default());
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input)
    }

    #[tokio::test]
    async fn test_assign_scalar_to_dict_column_produces_correct_type() {
        let logs_data = to_logs_data(vec![LogRecord::build().finish()]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        let pipeline_expr = OplParser::parse("logs | extend event_name = \"event\"")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();
        let logs = result.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::EVENT_NAME).unwrap().data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        )
    }

    #[tokio::test]
    async fn test_assign_scalar_to_non_dict_column_produces_correct_type() {
        let logs_data = to_logs_data(vec![LogRecord::build().finish()]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        let pipeline_expr = OplParser::parse("logs | extend dropped_attributes_count = 1")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();
        let logs = result.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::DROPPED_ATTRIBUTES_COUNT)
                .unwrap()
                .data_type(),
            &DataType::UInt32
        )
    }

    #[tokio::test]
    async fn test_assign_dict_u8_to_dict_column_produces_correct_type() {
        let logs_data = to_logs_data(vec![LogRecord::build().event_name("hello").finish()]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        // double check the input column has the expected type
        let logs = otap_batch.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::EVENT_NAME).unwrap().data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        );
        let pipeline_expr = OplParser::parse("logs | extend severity_text = event_name")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();
        let logs = result.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::SEVERITY_TEXT)
                .unwrap()
                .data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        )
    }

    #[tokio::test]
    async fn test_assign_dict_u16_to_dict_column_reduces_to_dict_u8_when_possible() {
        let logs_data = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("attr1", AnyValue::new_string("hello"))])
                .finish(),
        ]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        // double check the input column has the expected type
        let logs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::ATTRIBUTE_STR)
                .unwrap()
                .data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8))
        );
        let pipeline_expr = OplParser::parse("logs | extend severity_text = attributes[\"attr1\"]")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();
        let logs = result.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::SEVERITY_TEXT)
                .unwrap()
                .data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        )
    }

    #[tokio::test]
    async fn test_assign_dict_u16_to_dict_column_keeps_dict_u16_when_reduction_not_possible() {
        let mut log_records = vec![];
        for i in 0..300 {
            log_records.push(
                LogRecord::build()
                    .attributes(vec![KeyValue::new(
                        "attr1",
                        AnyValue::new_string(format!("{i}")),
                    )])
                    .finish(),
            )
        }

        let logs_data = to_logs_data(log_records);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        // double check the input column has the expected type
        let log_attrs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        assert_eq!(
            log_attrs
                .column_by_name(consts::ATTRIBUTE_STR)
                .unwrap()
                .data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8))
        );
        let pipeline_expr = OplParser::parse("logs | extend severity_text = attributes[\"attr1\"]")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();
        let logs = result.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::SEVERITY_TEXT)
                .unwrap()
                .data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8))
        )
    }

    #[tokio::test]
    async fn test_assign_non_dict_to_dict_casts_to_dict_u8_when_possible() {
        let mut log_records = vec![];
        for i in 0..128 {
            log_records.push(
                LogRecord::build()
                    .attributes(vec![KeyValue::new(
                        "attr1",
                        AnyValue::new_string(format!("{i}")),
                    )])
                    .finish(),
            )
        }

        let logs_data = to_logs_data(log_records);
        let mut otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        // double check the input column has the expected type
        let log_attrs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        let str_val = log_attrs.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        let log_attrs = super::try_upsert_column(
            consts::ATTRIBUTE_STR,
            cast(&str_val, &DataType::Utf8).unwrap(),
            log_attrs,
        )
        .unwrap();
        otap_batch.set(ArrowPayloadType::LogAttrs, log_attrs);

        let pipeline_expr = OplParser::parse("logs | extend severity_text = attributes[\"attr1\"]")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();
        let logs = result.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::SEVERITY_TEXT)
                .unwrap()
                .data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8))
        )
    }

    #[tokio::test]
    async fn test_assign_non_dict_to_dict_casts_to_dict_u16_when_possible() {
        let mut log_records = vec![];
        for i in 0..300 {
            log_records.push(
                LogRecord::build()
                    .attributes(vec![KeyValue::new(
                        "attr1",
                        AnyValue::new_string(format!("{i}")),
                    )])
                    .finish(),
            )
        }

        let logs_data = to_logs_data(log_records);
        let mut otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
        // double check the input column has the expected type
        let log_attrs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap();
        let str_val = log_attrs.column_by_name(consts::ATTRIBUTE_STR).unwrap();
        let log_attrs = super::try_upsert_column(
            consts::ATTRIBUTE_STR,
            cast(&str_val, &DataType::Utf8).unwrap(),
            log_attrs,
        )
        .unwrap();
        otap_batch.set(ArrowPayloadType::LogAttrs, log_attrs);

        let pipeline_expr = OplParser::parse("logs | extend severity_text = attributes[\"attr1\"]")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();
        let logs = result.get(ArrowPayloadType::Logs).unwrap();
        assert_eq!(
            logs.column_by_name(consts::SEVERITY_TEXT)
                .unwrap()
                .data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8))
        )
    }

    #[tokio::test]
    async fn test_insert_root_column_handles_null_coercion_by_removing_column() {
        let logs_data = to_logs_data(vec![
            LogRecord::build().severity_text("INFO").finish(),
            LogRecord::build().severity_text("DEBUG").finish(),
        ]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));

        let pipeline_expr = OplParser::parse("logs | set severity_text = attributes[\"x\"]")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(otap_batch).await.unwrap();

        let logs = result.get(ArrowPayloadType::Logs).unwrap();
        assert!(
            logs.column_by_name(consts::SEVERITY_TEXT).is_none(),
            "expected severity_text column to have been removed"
        )
    }

    #[tokio::test]
    async fn test_insert_root_column_wont_assign_null_to_non_nullable_column() {
        let traces_data = to_traces_data(vec![
            Span::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("event"))])
                .name("hello")
                .finish(),
            // this one doesn't have the attribute, so it will evaluate to null and the assignment
            // should fail
            Span::build().name("world").finish(),
        ]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Traces(traces_data));

        let pipeline_expr = OplParser::parse("traces | set name = attributes[\"x\"]")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        match pipeline.execute(otap_batch).await {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("cannot assign null result to non-nullable column name"),
                    "unexpected error message {:?}",
                    err_msg
                );
            }
            Ok(_) => {
                panic!("expected error, received Ok")
            }
        }
    }

    #[tokio::test]
    async fn test_insert_root_column_handles_null_coercion_to_non_null_col_with_error() {
        let traces_data = to_traces_data(vec![Span::build().finish()]);
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Traces(traces_data));

        let pipeline_expr = OplParser::parse("traces | set name = attributes[\"x\"]")
            .unwrap()
            .pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        match pipeline.execute(otap_batch).await {
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains("cannot assign null result to non-nullable column name"),
                    "unexpected error message {:?}",
                    err_msg
                );
            }
            Ok(_) => {
                panic!("expected error, received Ok")
            }
        }
    }
}
