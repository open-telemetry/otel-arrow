// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for planning pipeline execution

use data_engine_expressions::{
    BooleanValue, DataExpression, DateTimeValue, DoubleValue, Expression, IntegerValue, LogicalExpression, MapSelector, MoveTransformExpression, MutableValueExpression, PipelineExpression, ReduceMapTransformExpression, RemoveTransformExpression, RenameMapKeysTransformExpression, ScalarExpression, StaticScalarExpression, StringValue, TransformExpression, ValueAccessor
};
use datafusion::logical_expr::{BinaryExpr, Expr, Operator, col, lit};
use datafusion::prelude::{SessionContext, lit_timestamp_nano};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::transform::{AttributesTransform, DeleteTransform, RenameTransform};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::consts::{ATTRIBUTES_FIELD_NAME, RESOURCES_FIELD_NAME, SCOPE_FIELD_NAME};
use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::attributes::AttributeTransformPipelineStage;
use crate::pipeline::filter::optimize::AttrsFilterCombineOptimizerRule;
use crate::pipeline::filter::{Composite, FilterPipelineStage, FilterPlan};

/// Converts an pipeline expression (AST) into a series of executable pipeline stages.
///
/// The planner analyzes the pipeline definition and decides:
/// - Which operations can be handled by DataFusion stages
/// - Which operations need custom stages (e.g., cross-table filters)
/// - Optimizing by group operations into efficient stages
pub struct PipelinePlanner {}

impl PipelinePlanner {
    /// creates a new instance of `PipelinePlanner`
    pub fn new() -> Self {
        Self {}
    }

    /// Create pipeline stages from the pipeline definition.
    ///
    /// # Parameters
    /// - `session_context`: For creating DataFusion logical/physical plans
    /// - `pipeline_def`: The OPL expression tree to compile
    /// - `otap_batch`: The first batch, used to extract schemas for planning
    ///
    /// # Returns
    /// A vector of compiled stages ready for execution
    pub fn plan_stages(
        &mut self,
        pipeline: &PipelineExpression,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        let mut results = Vec::new();
        for data_expr in pipeline.get_expressions() {
            let mut expr_results = self.plan_data_expr(data_expr, session_ctx, otap_batch)?;
            results.append(&mut expr_results);
        }

        Ok(results)
    }

    fn plan_data_expr(
        &mut self,
        data_expr: &DataExpression,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        match data_expr {
            DataExpression::Discard(discard_expr) => match discard_expr.get_predicate() {
                // with discard expressions, we expect the expression actually specifies a filter
                // which is inverting the predicate by wrapping it in a "not". for example:
                // `logs | where severity_text == "ERROR"` would be a discard expr discarding
                // everything where not(severity_text == "ERROR"). we use the inner predicate to
                // build the filter.
                Some(LogicalExpression::Not(not_expr)) => {
                    self.plan_filter(not_expr.get_inner_expression(), session_ctx, otap_batch)
                }
                invalid => Err(Error::InvalidPipelineError {
                    cause: format!(
                        "expected DiscardExpression to contain Not predicate, found {invalid:?}"
                    ),
                    query_location: Some(discard_expr.get_query_location().clone()),
                }),
            },

            DataExpression::Transform(transform_expr) => match transform_expr {
                TransformExpression::Move(move_expr) => self.plan_move(move_expr),
                TransformExpression::RenameMapKeys(rename_map_keys_expr) => {
                    self.plan_rename(rename_map_keys_expr)
                }
                TransformExpression::ReduceMap(reduce_map_exr) => {
                    Self::plan_reduce_map(reduce_map_exr)
                }
                other => Err(Error::NotYetSupportedError {
                    message: format!(
                        "transform expression not yet supported {}",
                        other.get_name()
                    ),
                }),
            },

            // TODO support other DataExpressions
            other => Err(Error::NotYetSupportedError {
                message: format!("data expression not yet supported {}", other.get_name()),
            }),
        }
    }

    fn plan_filter(
        &mut self,
        logical_expr: &LogicalExpression,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        let filter_plan = Composite::<FilterPlan>::try_from(logical_expr)?;

        // optimize the to the plan
        let filter_plan = AttrsFilterCombineOptimizerRule::optimize(filter_plan);

        // transform logical plan into executable plan
        let filter_exec = filter_plan.to_exec(session_ctx, otap_batch)?;
        let filter_stage = FilterPipelineStage::new(filter_exec);

        Ok(vec![Box::new(filter_stage)])
    }

    fn plan_move(
        &self,
        move_expr: &MoveTransformExpression,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        match (move_expr.get_source(), move_expr.get_destination()) {
            (MutableValueExpression::Source(source), MutableValueExpression::Source(dest)) => {
                let source = ColumnAccessor::try_from(source.get_value_accessor())?;
                let dest = ColumnAccessor::try_from(dest.get_value_accessor())?;

                match (source, dest) {
                    // currently the only type of move transform supported is renaming attributes
                    (
                        ColumnAccessor::Attributes(src_attrs_id, src_key),
                        ColumnAccessor::Attributes(dest_attrs_id, dest_key),
                    ) => {
                        // the attributes being renamed must be in the same map. for example doing
                        // `attributes["x"] = resource.attributes["y"]` is not yet supported
                        if src_attrs_id != dest_attrs_id {
                            Err(Error::NotYetSupportedError {
                                message: format!(
                                    "attribute key rename currently only supports renaming within the same attributes map; found {:?} to {:?}",
                                    src_attrs_id, dest_attrs_id,
                                ),
                            })
                        } else {
                            let transform = AttributesTransform::default().with_rename(
                                RenameTransform::new([(src_key, dest_key)].into_iter().collect()),
                            );
                            transform
                                .validate()
                                .map_err(|e| Error::InvalidPipelineError {
                                    cause: format!("invalid attribute rename transform {e}"),
                                    query_location: Some(move_expr.get_query_location().clone()),
                                })?;
                            Ok(vec![Box::new(AttributeTransformPipelineStage::new(
                                src_attrs_id,
                                transform,
                            ))])
                        }
                    }

                    (source, dest) => Err(Error::NotYetSupportedError {
                        message: format!(
                            "move expression for column source = {source:?}, dest = {dest:?}"
                        ),
                    }),
                }
            }
            (source, dest) => Err(Error::NotYetSupportedError {
                message: format!("move from {source:?} to {dest:?}"),
            }),
        }
    }

    fn plan_rename(
        &self,
        rename_map_keys_expr: &RenameMapKeysTransformExpression,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        let mut root_attrs_renames = vec![];
        let mut scope_attrs_renames = vec![];
        let mut resource_attrs_renames = vec![];

        for key_rename in rename_map_keys_expr.get_keys() {
            let source = ColumnAccessor::try_from(key_rename.get_source())?;
            let dest = ColumnAccessor::try_from(key_rename.get_destination())?;

            match (source, dest) {
                // currently the only type of move transform supported is renaming attributes
                (
                    ColumnAccessor::Attributes(src_attrs_id, src_key),
                    ColumnAccessor::Attributes(dest_attrs_id, dest_key),
                ) => {
                    // the attributes being renamed must be in the same map. for example doing
                    // `attributes["x"] = resource.attributes["y"]` is not yet supported
                    if src_attrs_id != dest_attrs_id {
                        return Err(Error::NotYetSupportedError {
                            message: format!(
                                "attribute key rename currently only supports renaming within the same attributes map; found {:?} to {:?}",
                                src_attrs_id, dest_attrs_id,
                            ),
                        });
                    }

                    let rename = (src_key, dest_key);

                    match src_attrs_id {
                        AttributesIdentifier::Root => root_attrs_renames.push(rename),
                        AttributesIdentifier::NonRoot(payload_type) => match payload_type {
                            ArrowPayloadType::ResourceAttrs => resource_attrs_renames.push(rename),
                            ArrowPayloadType::ScopeAttrs => scope_attrs_renames.push(rename),
                            other => {
                                return Err(Error::NotYetSupportedError {
                                    message: format!("renaming attributes for payload {other:?}"),
                                });
                            }
                        },
                    }
                }

                (source, dest) => {
                    return Err(Error::NotYetSupportedError {
                        message: format!(
                            "move expression for column source = {source:?}, dest = {dest:?}"
                        ),
                    });
                }
            }
        }

        let mut pipeline_stages: Vec<Box<dyn PipelineStage>> = vec![];

        // build up a pipeline stage for each type set of attributes in the expression
        for (renames, attrs_id) in [
            (root_attrs_renames, AttributesIdentifier::Root),
            (
                scope_attrs_renames,
                AttributesIdentifier::NonRoot(ArrowPayloadType::ScopeAttrs),
            ),
            (
                resource_attrs_renames,
                AttributesIdentifier::NonRoot(ArrowPayloadType::ResourceAttrs),
            ),
        ] {
            if !renames.is_empty() {
                let rename_transform = RenameTransform::new(renames.into_iter().collect());
                let transform = AttributesTransform::default().with_rename(rename_transform);
                transform
                    .validate()
                    .map_err(|e| Error::InvalidPipelineError {
                        cause: format!("invalid attribute rename transform {e}"),
                        query_location: Some(rename_map_keys_expr.get_query_location().clone()),
                    })?;

                let pipeline_stage = AttributeTransformPipelineStage::new(attrs_id, transform);
                pipeline_stages.push(Box::new(pipeline_stage));
            }
        }

        Ok(pipeline_stages)
    }

    fn plan_reduce_map(reduce_map_expr: &ReduceMapTransformExpression) -> Result<Vec<Box<dyn PipelineStage>>> {
        let mut root_attrs_deletes = vec![];
        let mut scope_attrs_deletes = vec![];
        let mut resource_attrs_deletes = vec![];

        match reduce_map_expr {
            ReduceMapTransformExpression::Remove(remove_expr) => {
                for map_selector  in remove_expr.get_selectors() {
                    match map_selector {
                        MapSelector::ValueAccessor(val) => match ColumnAccessor::try_from(val)? {
                            ColumnAccessor::Attributes(attrs_ident, attrs_key) => match attrs_ident {
                                AttributesIdentifier::Root => root_attrs_deletes.push(attrs_key),
                                AttributesIdentifier::NonRoot(payload_type) => match payload_type {
                                    ArrowPayloadType::ResourceAttrs => resource_attrs_deletes.push(attrs_key),
                                    ArrowPayloadType::ScopeAttrs => scope_attrs_deletes.push(attrs_key),
                                    _ => {
                                        // invalid attributes payload type
                                        todo!()
                                    }
                                }
                            },
                            _=> {
                                // invalid columna ccessor
                                todo!()
                            }
                        }
                        MapSelector::KeyOrKeyPattern(_) => {
                            // TODO error about how remove using map key pattern not supported
                            todo!()
                        }
                    }
                }
            },
            ReduceMapTransformExpression::Retain(retain_expr) => {
                // return an error here
                // write a test case that we return an error here
                todo!()
            }
        }

        let mut pipeline_stages: Vec<Box<dyn PipelineStage>> = vec![];

        // build up a pipeline stage for each type set of attributes in the expression
        for (deletes, attrs_id) in [
            (root_attrs_deletes, AttributesIdentifier::Root),
            (
                scope_attrs_deletes,
                AttributesIdentifier::NonRoot(ArrowPayloadType::ScopeAttrs),
            ),
            (
                resource_attrs_deletes,
                AttributesIdentifier::NonRoot(ArrowPayloadType::ResourceAttrs),
            ),
        ] {
            if !deletes.is_empty() {
                let delete_transform = DeleteTransform::new(deletes.into_iter().collect());
                let transform = AttributesTransform::default().with_delete(delete_transform);
                transform
                    .validate()
                    .map_err(|e| Error::InvalidPipelineError {
                        cause: format!("invalid attribute rename transform {e}"),
                        query_location: Some(reduce_map_expr.get_query_location().clone()),
                    })?;

                let pipeline_stage = AttributeTransformPipelineStage::new(attrs_id, transform);
                pipeline_stages.push(Box::new(pipeline_stage));
            }
        }

        Ok(pipeline_stages)
    }
}

#[derive(Debug)]
pub enum BinaryArg {
    Column(ColumnAccessor),
    Literal(StaticScalarExpression),
    Null,
}

impl TryFrom<&ScalarExpression> for BinaryArg {
    type Error = Error;

    fn try_from(scalar_expr: &ScalarExpression) -> Result<Self> {
        let binary_arg = match scalar_expr {
            ScalarExpression::Source(source) => {
                BinaryArg::Column(ColumnAccessor::try_from(source.get_value_accessor())?)
            }
            ScalarExpression::Static(static_expr) => match static_expr {
                StaticScalarExpression::Null(_) => BinaryArg::Null,
                static_expr => BinaryArg::Literal(static_expr.clone()),
            },
            expr => {
                return Err(Error::NotYetSupportedError {
                    message: format!(
                        "expression type not yet supported as argument to binary operation. received {:?}",
                        expr,
                    ),
                });
            }
        };

        Ok(binary_arg)
    }
}

#[derive(Debug)]
pub enum ColumnAccessor {
    ColumnName(String),

    /// column in a nested struct. for example resource.schema_url or instrumentation_scope.name
    StructCol(&'static str, String),

    /// payload type identifies which attributes are being joined
    /// and the string identifies the attribute key
    Attributes(AttributesIdentifier, String),
}

impl ColumnAccessor {
    fn try_from_attrs_key(
        attrs_identifier: AttributesIdentifier,
        scalar_expr: &ScalarExpression,
    ) -> Result<Self> {
        match scalar_expr {
            ScalarExpression::Static(StaticScalarExpression::String(attr_key)) => Ok(
                Self::Attributes(attrs_identifier, attr_key.get_value().to_string()),
            ),

            // TODO: handle users accessing attributes in a different way, like for example from a variable,
            // function result, etc.
            expr => Err(Error::NotYetSupportedError {
                message: format!(
                    "unsupported attributes key. currently only static string key name is supported. received {:?}",
                    expr
                ),
            }),
        }
    }

    fn try_from_struct_field(
        struct_column_name: &'static str,
        attrs_payload_type: ArrowPayloadType,
        selectors: &[ScalarExpression],
    ) -> Result<Self> {
        match &selectors[1] {
            ScalarExpression::Static(StaticScalarExpression::String(struct_field)) => {
                match struct_field.get_value() {
                    ATTRIBUTES_FIELD_NAME => Self::try_from_attrs_key(
                        AttributesIdentifier::NonRoot(attrs_payload_type),
                        &selectors[2],
                    ),
                    struct_field => Ok(Self::StructCol(
                        struct_column_name,
                        struct_field.to_string(),
                    )),
                }
            }
            expr => Err(Error::InvalidPipelineError {
                cause: format!(
                    "unsupported nested struct column definition for struct {}. received {:?}",
                    struct_column_name, expr
                ),
                query_location: Some(selectors[1].get_query_location().clone()),
            }),
        }
    }
}

impl TryFrom<&ValueAccessor> for ColumnAccessor {
    type Error = Error;

    fn try_from(accessor: &ValueAccessor) -> Result<Self> {
        let selectors = accessor.get_selectors();

        match &selectors[0] {
            ScalarExpression::Static(StaticScalarExpression::String(column)) => {
                let column_name = column.get_value();
                match column_name {
                    ATTRIBUTES_FIELD_NAME => {
                        Self::try_from_attrs_key(AttributesIdentifier::Root, &selectors[1])
                    }
                    RESOURCES_FIELD_NAME => Self::try_from_struct_field(
                        consts::RESOURCE,
                        ArrowPayloadType::ResourceAttrs,
                        selectors,
                    ),
                    SCOPE_FIELD_NAME => Self::try_from_struct_field(
                        consts::SCOPE,
                        ArrowPayloadType::ScopeAttrs,
                        selectors,
                    ),
                    value => Ok(Self::ColumnName(value.to_string())),
                }
            }
            expr => Err(Error::InvalidPipelineError {
                cause: format!("unsupported column definition. received {:?}", expr),
                query_location: Some(selectors[0].get_query_location().clone()),
            }),
        }
    }
}

/// Identifier of a batch of attributes
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum AttributesIdentifier {
    /// Attributes for the root record type. E.g. LogAttrs for a batch of log records
    Root,

    /// Attributes for something that isn't the root record type. E.g. ScopeAttrs, ResourceAttrs
    NonRoot(ArrowPayloadType),
}

pub fn try_static_scalar_to_literal_for_column(
    column_name: &str,
    static_scalar: &StaticScalarExpression,
) -> Result<Expr> {
    Ok(match static_scalar {
        // for integers, we need to choose the correct type of literal for the field
        // note: this currently contains fields only on the root record batches as we don't
        // yet support traversing into the OTAP batch to filter by nested fields like span
        // events/links, metrics data points, etc.
        StaticScalarExpression::Integer(int_val) => {
            let val = int_val.get_value();
            match column_name {
                consts::AGGREGATION_TEMPORALITY
                | consts::SEVERITY_NUMBER
                | consts::KIND
                | consts::EXP_HISTOGRAM_OFFSET => lit(val as i32),

                consts::METRIC_TYPE => lit(val as u8),

                consts::DROPPED_ATTRIBUTES_COUNT
                | consts::FLAGS
                | consts::DROPPED_EVENTS_COUNT
                | consts::DROPPED_LINKS_COUNT => lit(val as u32),

                // other columns for which filtering is currently supported are i64
                _ => lit(val),
            }
        }
        StaticScalarExpression::String(str_val) => lit(str_val.get_value()),
        StaticScalarExpression::Boolean(bool_val) => lit(bool_val.get_value()),
        StaticScalarExpression::Double(float_val) => lit(float_val.get_value()),
        StaticScalarExpression::DateTime(dt_val) => {
            let val =
                dt_val
                    .get_value()
                    .timestamp_nanos_opt()
                    .ok_or_else(|| Error::ExecutionError {
                        cause: format!("failed to convert {dt_val:?} to nanosecond timestamp"),
                    })?;
            lit_timestamp_nano(val)
        }
        _ => {
            return Err(Error::NotYetSupportedError {
                message: format!(
                    "literal from scalar expression. received {:?}",
                    static_scalar
                ),
            });
        }
    })
}

pub fn try_static_scalar_to_attr_literal(static_scalar: &StaticScalarExpression) -> Result<Expr> {
    let lit_expr = match static_scalar {
        StaticScalarExpression::String(str_val) => lit(str_val.get_value()),
        StaticScalarExpression::Boolean(bool_val) => lit(bool_val.get_value()),
        StaticScalarExpression::Integer(int_val) => lit(int_val.get_value()),
        StaticScalarExpression::Double(float_val) => lit(float_val.get_value()),
        _ => {
            return Err(Error::NotYetSupportedError {
                message: format!(
                    "literal from scalar expression. received {:?}",
                    static_scalar
                ),
            });
        }
    };

    Ok(lit_expr)
}

/// try to get the column from an OTAP batch containing an AnyValue based on the value of some
/// defined static scalar.
pub fn try_static_scalar_to_any_val_column(
    static_scalar: &StaticScalarExpression,
) -> Result<&'static str> {
    let col_name = match static_scalar {
        StaticScalarExpression::Boolean(_) => consts::ATTRIBUTE_BOOL,
        StaticScalarExpression::Double(_) => consts::ATTRIBUTE_DOUBLE,
        StaticScalarExpression::Integer(_) => consts::ATTRIBUTE_INT,
        StaticScalarExpression::String(_) => consts::ATTRIBUTE_STR,
        _ => {
            return Err(Error::NotYetSupportedError {
                message: format!(
                    "AnyValues values column from scalar literal. received {:?}",
                    static_scalar
                ),
            });
        }
    };

    Ok(col_name)
}

/// Create the BinaryExpr that would be used to filter for the value of an attribute in an OTAP
/// attributes record batch. This considers the type of the scalar literal to select the correct
/// column e.g. string literals should filter by the "str" column and also creates a datafusion
/// literal expr with the correct type to compare against.
pub fn try_attrs_value_filter_from_literal(
    scalar_lit: &StaticScalarExpression,
    binary_op: Operator,
) -> Result<BinaryExpr> {
    Ok(BinaryExpr::new(
        Box::new(col(try_static_scalar_to_any_val_column(scalar_lit)?)),
        binary_op,
        Box::new(try_static_scalar_to_attr_literal(scalar_lit)?),
    ))
}
