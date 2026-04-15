// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains code for planning pipeline execution

use std::collections::HashSet;

use data_engine_expressions::{
    BooleanScalarExpression, BooleanValue, DataExpression, DateTimeValue, DoubleValue, Expression,
    IntegerValue, LogicalExpression, MapSelector, MoveTransformExpression, MutableValueExpression,
    OutputExpression, PipelineExpression, PipelineFunction, PipelineFunctionExpression,
    PipelineFunctionImplementation, ReduceMapTransformExpression, RenameMapKeysTransformExpression,
    ScalarExpression, SetTransformExpression, SourceScalarExpression, StaticScalarExpression,
    StringValue, TransformExpression, ValueAccessor,
};
use datafusion::common::tree_node::{TreeNode, TreeNodeRecursion};
use datafusion::logical_expr::{BinaryExpr, Expr, Operator, col, lit};
use datafusion::prelude::{SessionContext, lit_timestamp_nano};
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::transform::{AttributesTransform, DeleteTransform, RenameTransform};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::schema::consts;

use crate::consts::{ATTRIBUTES_FIELD_NAME, RESOURCES_FIELD_NAME, SCOPE_FIELD_NAME};
use crate::error::{Error, Result};
use crate::pipeline::apply_attrs::ApplyToAttributesPipelineStage;
use crate::pipeline::assign::{AssignPipelineStage, Assignment};
use crate::pipeline::attributes::AttributeTransformPipelineStage;
use crate::pipeline::conditional::{ConditionalPipelineStage, ConditionalPipelineStageBranch};
use crate::pipeline::expr::{
    DataScope, ExprLogicalPlanner, LogicalExprDataSource, ScopedLogicalExpr,
};
use crate::pipeline::filter::optimize::{
    AttrValueColumnSelectionOptimizer, AttrsFilterCombineOptimizerRule, CompositeToBaseFilterPlan,
};
use crate::pipeline::filter::{Composite, FilterExec, FilterPipelineStage, FilterPlan};
use crate::pipeline::routing::RouteToPipelineStage;
use crate::pipeline::{BoxedPipelineStage, PipelineStage};

/// Converts an pipeline expression (AST) into a series of executable pipeline stages.
///
/// The planner analyzes the pipeline definition and decides:
/// - Which operations can be handled by DataFusion stages
/// - Which operations need custom stages (e.g., cross-table filters)
/// - Optimizing by group operations into efficient stages
pub struct PipelinePlanner {
    plan_for_attributes: bool,

    /// Whether to consider  attribute keys case sensitive in filtering pipeline stages
    filter_attribute_keys_case_sensitive: bool,
}

impl PipelinePlanner {
    /// creates a new instance of `PipelinePlanner`
    pub const fn new() -> Self {
        Self {
            plan_for_attributes: false,
            filter_attribute_keys_case_sensitive: true,
        }
    }

    pub const fn new_for_attributes() -> Self {
        Self {
            plan_for_attributes: true,
            filter_attribute_keys_case_sensitive: true,
        }
    }

    pub const fn with_filter_attribute_keys_case_sensitive(mut self, val: bool) -> Self {
        self.filter_attribute_keys_case_sensitive = val;
        self
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
        &self,
        pipeline: &PipelineExpression,
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        self.plan_data_exprs(
            pipeline.get_expressions(),
            pipeline.get_functions(),
            session_ctx,
            otap_batch,
        )
    }

    fn plan_data_exprs(
        &self,
        data_exprs: &[DataExpression],
        functions: &[PipelineFunction],
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Vec<BoxedPipelineStage>> {
        let mut results = Vec::new();
        let mut i = 0;

        while i < data_exprs.len() {
            let data_expr = &data_exprs[i];

            // coalesce consecutive SET expressions
            let mut expr_results = if let Some(first_set) = Self::as_set_exp(data_expr) {
                let set_exprs = Self::collect_consecutive_sets(&data_exprs[i..], first_set);
                let count = set_exprs.len();
                i += count;
                self.plan_sets(&set_exprs, functions, session_ctx, otap_batch)?
            } else {
                i += 1;
                self.plan_data_expr(data_expr, functions, session_ctx, otap_batch)?
            };

            // validate the pipeline stages are valid for attributes if planning pipeline to apply
            // to attrs batches only
            if self.plan_for_attributes {
                for stage in &expr_results {
                    if !stage.supports_exec_on_attributes() {
                        return Err(Error::InvalidPipelineError {
                            cause: format!(
                                "Data expression not supported on attributes stream: {data_expr:?}"
                            ),
                            query_location: Some(data_expr.get_query_location().clone()),
                        });
                    }
                }
            }
            results.append(&mut expr_results);
        }

        Ok(results)
    }

    fn as_set_exp(data_expr: &DataExpression) -> Option<&SetTransformExpression> {
        match data_expr {
            DataExpression::Transform(TransformExpression::Set(s)) => Some(s),
            _ => None,
        }
    }

    fn collect_consecutive_sets<'a>(
        data_exprs: &'a [DataExpression],
        first_set: &'a SetTransformExpression,
    ) -> Vec<&'a SetTransformExpression> {
        let mut set_exprs = vec![first_set];

        for expr in &data_exprs[1..] {
            match Self::as_set_exp(expr) {
                Some(set) => set_exprs.push(set),
                None => break,
            }
        }

        set_exprs
    }

    fn plan_data_expr(
        &self,
        data_expr: &DataExpression,
        functions: &[PipelineFunction],
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
                Some(LogicalExpression::Not(not_expr)) => self.plan_filter(
                    not_expr.get_inner_expression(),
                    functions,
                    session_ctx,
                    otap_batch,
                ),

                // the discard expression's `not` statement may get folded into a static constant
                // filter in which case we don't produce logical expression as `not(true)`, instead
                // it just gets folded into `false`. In this case, we just invert the static bool
                Some(LogicalExpression::Scalar(scalar_expr)) => match scalar_expr {
                    ScalarExpression::Static(StaticScalarExpression::Boolean(bool_expr)) => {
                        let keep_plan = LogicalExpression::Scalar(ScalarExpression::Static(
                            StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                                bool_expr.get_query_location().clone(),
                                !bool_expr.get_value(),
                            )),
                        ));
                        self.plan_filter(&keep_plan, functions, session_ctx, otap_batch)
                    }
                    invalid => Err(Error::InvalidPipelineError {
                        cause: format!(
                            "unsupported Static for discard expression. Expected boolean, found {invalid:?}"
                        ),
                        query_location: Some(discard_expr.get_query_location().clone()),
                    }),
                },
                // discard expression with `None` predicate indicates the default behaviour which
                // discards everything. In this case, what we want is a static filter that rejects
                // all rows
                None => {
                    let predicate = LogicalExpression::Scalar(ScalarExpression::Static(
                        StaticScalarExpression::Boolean(BooleanScalarExpression::new(
                            discard_expr.get_query_location().clone(),
                            false,
                        )),
                    ));
                    self.plan_filter(&predicate, functions, session_ctx, otap_batch)
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
                TransformExpression::Set(set_expr) => {
                    self.plan_sets(&[set_expr], functions, session_ctx, otap_batch)
                }
                other => Err(Error::NotYetSupportedError {
                    message: format!(
                        "transform expression not yet supported {}",
                        other.get_name()
                    ),
                }),
            },

            DataExpression::Conditional(conditional_expr) => {
                let mut pipeline_branches = vec![];
                for branch in conditional_expr.get_branches() {
                    let predicate = self.plan_filter_exec(
                        branch.get_condition(),
                        functions,
                        session_ctx,
                        otap_batch,
                    )?;

                    let pipeline_stages = self.plan_data_exprs(
                        branch.get_expressions(),
                        functions,
                        session_ctx,
                        otap_batch,
                    )?;

                    pipeline_branches.push(ConditionalPipelineStageBranch::new(
                        predicate,
                        pipeline_stages,
                    ));
                }

                let default_branch = conditional_expr
                    .get_default_branch()
                    .map(|data_exprs| {
                        self.plan_data_exprs(data_exprs, functions, session_ctx, otap_batch)
                    })
                    .transpose()?;

                let pipeline_stage =
                    ConditionalPipelineStage::new(pipeline_branches, default_branch);
                Ok(vec![Box::new(pipeline_stage)])
            }

            DataExpression::Output(output_expr) => match output_expr.get_output() {
                OutputExpression::NamedSink(name) => {
                    Ok(vec![Box::new(RouteToPipelineStage::new(name.get_value()))])
                }
            },

            // TODO support other DataExpressions
            other => Err(Error::NotYetSupportedError {
                message: format!("data expression not yet supported {}", other.get_name()),
            }),
        }
    }

    fn plan_filter(
        &self,
        logical_expr: &LogicalExpression,
        functions: &[PipelineFunction],
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        let filter_exec =
            self.plan_filter_exec(logical_expr, functions, session_ctx, otap_batch)?;
        let filter_stage = FilterPipelineStage::new(filter_exec);

        Ok(vec![Box::new(filter_stage)])
    }

    /// plan a [`FilterExec`] from a [`LogicalExpression`]
    fn plan_filter_exec(
        &self,
        logical_expr: &LogicalExpression,
        functions: &[PipelineFunction],
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Composite<FilterExec>> {
        let filter_plan = Composite::<FilterPlan>::try_from(
            logical_expr,
            self.filter_attribute_keys_case_sensitive,
            functions,
        )?;

        // optimize the to the plan
        let filter_plan = if self.plan_for_attributes {
            // Currently using a two step transformation of the FilterPlan to turn this into
            // something that can be applied directly to the attributes record batch.
            // First, we combine all the filter expressions in some Composite<FilterPlan> into
            // a single Composite::Base containing a single expression.
            // Next, we look for any places we are doing a binary expression like `value == "x"`,
            // and determining the _actual_ values column (str, int, bool, etc.) to use in this
            // expression, as "value" is just being treated as a logical column to make it easier
            // to write the expressions
            CompositeToBaseFilterPlan::optimize(filter_plan)
                .and_then(AttrValueColumnSelectionOptimizer::optimize)?
        } else {
            AttrsFilterCombineOptimizerRule::optimize(filter_plan)
        };

        // transform logical plan into executable plan
        filter_plan.to_exec(session_ctx, otap_batch)
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

    fn plan_reduce_map(
        reduce_map_expr: &ReduceMapTransformExpression,
    ) -> Result<Vec<Box<dyn PipelineStage>>> {
        let mut root_attrs_deletes = vec![];
        let mut scope_attrs_deletes = vec![];
        let mut resource_attrs_deletes = vec![];

        match reduce_map_expr {
            ReduceMapTransformExpression::Remove(remove_expr) => {
                for map_selector in remove_expr.get_selectors() {
                    match map_selector {
                        MapSelector::ValueAccessor(val) => match ColumnAccessor::try_from(val)? {
                            // currently the only kind of remove operation we support is on attributes
                            ColumnAccessor::Attributes(attrs_ident, attrs_key) => match attrs_ident
                            {
                                AttributesIdentifier::Root => root_attrs_deletes.push(attrs_key),
                                AttributesIdentifier::NonRoot(payload_type) => match payload_type {
                                    ArrowPayloadType::ResourceAttrs => {
                                        resource_attrs_deletes.push(attrs_key)
                                    }
                                    ArrowPayloadType::ScopeAttrs => {
                                        scope_attrs_deletes.push(attrs_key)
                                    }
                                    payload_type => {
                                        return Err(Error::NotYetSupportedError {
                                            message: format!(
                                                "removing map keys from payload type {payload_type:?} not yet supported"
                                            ),
                                        });
                                    }
                                },
                            },
                            column => {
                                return Err(Error::InvalidPipelineError {
                                    cause: format!(
                                        "reduce map remove specified non map column. found {column:?}"
                                    ),
                                    query_location: Some(remove_expr.get_query_location().clone()),
                                });
                            }
                        },
                        MapSelector::KeyOrKeyPattern(_) => {
                            return Err(Error::NotYetSupportedError {
                                message:
                                    "specifying map removes by key or key pattern not yet supported"
                                        .into(),
                            });
                        }
                    }
                }
            }
            ReduceMapTransformExpression::Retain(_) => {
                return Err(Error::NotYetSupportedError {
                    message: "reducing map using by specifying retain keys not yet supported"
                        .into(),
                });
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
                        cause: format!("invalid attribute delete transform {e}"),
                        query_location: Some(reduce_map_expr.get_query_location().clone()),
                    })?;

                let pipeline_stage = AttributeTransformPipelineStage::new(attrs_id, transform);
                pipeline_stages.push(Box::new(pipeline_stage));
            }
        }

        Ok(pipeline_stages)
    }

    /// Create a set of [`PipelineStage`]s that will execute the "Set" expressions.
    ///
    /// This function may combine multiple assignments into a single pipeline stage in the case
    /// where there may be performance benefits for doing so. Generally this means materializing
    /// an arrow [`RecordBatch`] for some OTAP payload type multiple times.
    fn plan_sets(
        &self,
        set_exprs: &[&SetTransformExpression],
        functions: &[PipelineFunction],
        session_ctx: &SessionContext,
        otap_batch: &OtapArrowRecords,
    ) -> Result<Vec<BoxedPipelineStage>> {
        let mut results: Vec<BoxedPipelineStage> = Vec::new();

        // list of combined assignments for the next assignment pipeline stage.
        let mut assignments = Vec::new();
        let logical_planner = ExprLogicalPlanner::default();

        // TODO - currently the logic for coalescing multiple assignments isn't as intelligent
        // as it could be. The strategy currently employed is just to look at adjacent set
        // expressions and combine them if they have the same destination & unambiguous keys/cols
        // (see note below about ambiguity). we could be more intelligent, for example:
        // - `attributes["x"] = "5"`
        // - `severity_number = 10`
        // - `attributes["y"] = "14"`
        // the first and third statement are currently are not combined, although they could be
        // for best performance.

        // When doing multiple assignments in bulk, the underlying Assignment pipeline stage makes
        // no guarantees about the logical order in which the assignments are executed. This can
        // lead to ambiguity about the final result if the same column or attribute key is used
        // in multiple expressions. For example, if we combine two expressions like:
        // `attributes["x"] = "y"` and `attributes["x"] = "z"` the end result would be different
        // depending on which executed "first". Similar situation exist for cases like:
        // `attributes["x"] = "A"` and `attributes["y"] = attributes["x"]`.
        //
        // To avoid the ambiguity, we keep track of which keys or columns used in the combined
        // expressions in this set..
        let mut cols_or_keys_referenced = HashSet::new();

        // update the referenced column/attr key tracking set
        fn set_dest_attr_key(dest_accessor: &ColumnAccessor, referenced: &mut HashSet<String>) {
            if let ColumnAccessor::ColumnName(col_name) = dest_accessor {
                _ = referenced.insert(col_name.into());
            }
            if let ColumnAccessor::Attributes(_, key) = dest_accessor {
                _ = referenced.insert(key.into());
            }
        }

        // checks if the next attribute can be combined with the previous attribute by validating
        // they have the same destination. The assign pipeline stage will only do multiple
        // assignments for the same destination.
        //
        // This also validates that the next doesn't reference keys or columns that would make the
        // combined assignment ambiguous (see comments above about ambiguity)
        fn check_combine(
            prev: &Assignment<'_>,
            next: &Assignment<'_>,
            cols_or_keys_referenced: &HashSet<String>,
        ) -> bool {
            let can_combine_dests = match (&prev.dest_column, &next.dest_column) {
                (ColumnAccessor::ColumnName(_), ColumnAccessor::ColumnName(next_key)) => {
                    !cols_or_keys_referenced.contains(next_key)
                }
                (
                    ColumnAccessor::Attributes(prev_attrs_id, _),
                    ColumnAccessor::Attributes(next_attrs_id, next_key),
                ) => prev_attrs_id == next_attrs_id && !cols_or_keys_referenced.contains(next_key),

                // in all other cases, the destinations don't match so we can't combine into
                // a single assignment
                _ => false,
            };

            if !can_combine_dests {
                return false;
            }

            // ensure that if the source is an attribute
            if source_references_col_or_key(&next.source, cols_or_keys_referenced) {
                return false;
            }

            true
        }

        // recursively descends the source plan to find any cases where the source references
        // an attribute key or column whose value is in the `referenced` set (which is the set
        // of destinations that may have been reassigned a new value).
        fn source_references_col_or_key(
            source_expr: &ScopedLogicalExpr,
            referenced: &HashSet<String>,
        ) -> bool {
            match &source_expr.source {
                LogicalExprDataSource::DataSource(data_scope) => match &data_scope {
                    DataScope::Attributes(_, key) => referenced.contains(key),
                    DataScope::StaticScalar => false,

                    // visit the expression applied to the root and search for any column exprs
                    DataScope::Root => {
                        let mut source_contains_refed_column = false;
                        _ = source_expr.logical_expr.apply(|expr| {
                            if let Expr::Column(column) = expr {
                                source_contains_refed_column = referenced.contains(column.name());
                            }

                            Ok(if source_contains_refed_column {
                                TreeNodeRecursion::Stop
                            } else {
                                TreeNodeRecursion::Continue
                            })
                        });

                        source_contains_refed_column
                    }
                },
                LogicalExprDataSource::Join(left, right) => {
                    let left = source_references_col_or_key(left.as_ref(), referenced);
                    let right = source_references_col_or_key(right.as_ref(), referenced);
                    left | right
                }
                LogicalExprDataSource::MultiJoin(children) => children
                    .iter()
                    .any(|child| source_references_col_or_key(child, referenced)),
            }
        }

        for set_expr in set_exprs {
            let MutableValueExpression::Source(dest) = set_expr.get_destination() else {
                return Err(Error::NotYetSupportedError {
                    message: "set expression only supports source destinations".to_string(),
                });
            };

            // handle if this "set" expr is a nested pipeline that should be applied to attributes.
            // In our AST expression tree, these are modeled as function invocations. These are
            // never combined into a single "set" expression because it would add complexity
            // without any performance benefit.
            //
            // TODO - in the future we may want some way to identify that this is the special type
            // of "function" that represents a nested pipeline applied to attributes, either by
            // its name or some additional metadata. For now, we just know this is the only type
            // of function invocation supported.
            if let ScalarExpression::InvokeFunction(func) = set_expr.get_source() {
                let function_id = func.get_function_id();
                let function =
                    functions
                        .get(function_id)
                        .ok_or_else(|| Error::InvalidPipelineError {
                            cause: format!("did not find function with id {}", function_id),
                            query_location: Some(func.get_query_location().clone()),
                        })?;

                if let PipelineFunctionImplementation::Expressions(function_exprs) =
                    function.get_implementation()
                {
                    // create a pipeline stage to execute any previous assignments before executing
                    // this nested pipeline
                    if !assignments.is_empty() {
                        let pipeline_stage = AssignPipelineStage::try_new(&mut assignments)?;
                        results.push(Box::new(pipeline_stage));
                        assignments.clear();
                        cols_or_keys_referenced.clear();
                    }

                    let mut inner_pipeline_data_exprs = Vec::with_capacity(function_exprs.len());
                    for func_expr in function_exprs {
                        let data_expr = match func_expr {
                            PipelineFunctionExpression::Conditional(c) => {
                                DataExpression::Conditional(c.clone())
                            }
                            PipelineFunctionExpression::Discard(d) => {
                                DataExpression::Discard(d.clone())
                            }
                            PipelineFunctionExpression::Transform(t) => {
                                DataExpression::Transform(t.clone())
                            }
                            PipelineFunctionExpression::Return(_r) => {
                                return Err(Error::NotYetSupportedError {
                                    message: "return statement in function not yet supported"
                                        .into(),
                                });
                            }
                        };
                        inner_pipeline_data_exprs.push(data_expr);
                    }

                    let planner = Self::new_for_attributes();
                    let child_pipeline = planner.plan_data_exprs(
                        &inner_pipeline_data_exprs,
                        functions,
                        session_ctx,
                        otap_batch,
                    )?;

                    let attributes_id = Self::source_to_apply_attrs_id(dest).ok_or_else(|| {
                        Error::InvalidPipelineError {
                            cause: format!(
                                "Invalid source for nested apply pipeline to attributes {:?}",
                                dest,
                            ),
                            query_location: Some(dest.get_query_location().clone()),
                        }
                    })?;

                    results.push(Box::new(ApplyToAttributesPipelineStage::new(
                        attributes_id,
                        child_pipeline,
                    )));

                    continue;
                }
            }

            // create new assignment argument
            let assignment = Assignment {
                dest_column: ColumnAccessor::try_from(dest.get_value_accessor())?,
                source: logical_planner.plan_scalar_expr(set_expr.get_source(), functions)?,
                dest_query_location: Some(dest.get_query_location()),
            };

            // check if can combine with previous set of assignments
            let combine = assignments
                .last()
                .map(|prev| check_combine(prev, &assignment, &cols_or_keys_referenced))
                .unwrap_or(true);

            // if cannot combine with other assignments, create new pipeline stage and clear
            // list of current assignments
            if !combine {
                let pipeline_stage = AssignPipelineStage::try_new(&mut assignments)?;
                results.push(Box::new(pipeline_stage));
                assignments.clear();
                cols_or_keys_referenced.clear();
            }

            // assignment will be combined with previous assignments
            set_dest_attr_key(&assignment.dest_column, &mut cols_or_keys_referenced);
            assignments.push(assignment);
        }

        if !assignments.is_empty() {
            let pipeline_stage = AssignPipelineStage::try_new(&mut assignments)?;
            results.push(Box::new(pipeline_stage));
        }

        Ok(results)
    }

    /// when we receive an expression representing a nested pipeline, we currently assume it is
    /// being applied to attributes. This attempts to determine to which set of attributes the
    /// pipeline should be applied. Returns an error if the source does not identify a set of
    /// attributes.
    ///
    /// Example valid inputs would include: attributes, resource/scope.attributes
    ///
    fn source_to_apply_attrs_id(
        source_expr: &SourceScalarExpression,
    ) -> Option<AttributesIdentifier> {
        let values_accessor = source_expr.get_value_accessor();
        let selectors = values_accessor.get_selectors();
        match selectors.len() {
            1 => match &selectors[0] {
                ScalarExpression::Static(StaticScalarExpression::String(column)) => {
                    (column.get_value() == ATTRIBUTES_FIELD_NAME)
                        .then_some(AttributesIdentifier::Root)
                }
                _ => None,
            },
            2 => match (&selectors[0], &selectors[1]) {
                (
                    ScalarExpression::Static(StaticScalarExpression::String(column0)),
                    ScalarExpression::Static(StaticScalarExpression::String(column1)),
                ) => match (column0.get_value(), column1.get_value()) {
                    (RESOURCES_FIELD_NAME, ATTRIBUTES_FIELD_NAME) => Some(
                        AttributesIdentifier::NonRoot(ArrowPayloadType::ResourceAttrs),
                    ),
                    (SCOPE_FIELD_NAME, ATTRIBUTES_FIELD_NAME) => {
                        Some(AttributesIdentifier::NonRoot(ArrowPayloadType::ScopeAttrs))
                    }
                    _ => None,
                },
                _ => None,
            },
            _ => None,
        }
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

#[cfg(test)]
mod test {
    use data_engine_kql_parser::Parser;
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::{OtapArrowRecords, otap::Logs};

    use crate::pipeline::{Pipeline, planner::PipelinePlanner};

    #[test]
    fn test_combines_set_expressions_for_root() {
        let pipeline_expr =
            OplParser::parse("logs | set severity_number = 5 | set severity_text = \"INFO\"")
                .unwrap()
                .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 1)
    }

    #[test]
    fn test_combines_set_expressions_for_attributes() {
        let pipeline_expr =
            OplParser::parse("logs | set attributes[\"x\"] = 5 | set attributes[\"y\"] = 6")
                .unwrap()
                .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 1)
    }

    #[test]
    fn test_does_not_combine_set_expressions_for_same_attribute_destination() {
        let pipeline_expr =
            OplParser::parse("logs | set attributes[\"x\"] = 5 | set attributes[\"x\"] = 6")
                .unwrap()
                .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 2)
    }

    #[test]
    fn test_does_not_combine_set_expressions_for_same_root() {
        let pipeline_expr =
            OplParser::parse("logs | set severity_text=\"INFO\" | set severity_text=\"ERROR\"")
                .unwrap()
                .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 2)
    }

    #[test]
    fn test_does_not_combine_set_expressions_for_root_when_source_reassigned() {
        let pipeline_expr =
            OplParser::parse("logs | set severity_text=\"INFO\" | set event_name=severity_text")
                .unwrap()
                .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 2)
    }

    #[test]
    fn test_combines_set_expressions_for_root_when_source_is_not_reassigned_self() {
        let pipeline_expr =
            OplParser::parse("logs | set severity_text=\"INFO\" | set event_name=event_name")
                .unwrap()
                .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 1)
    }

    #[test]
    fn test_does_not_combine_set_expressions_for_attributes_when_source_reassigned() {
        let pipeline_expr = OplParser::parse(
            "logs | set attributes[\"x\"] = 5 | set attributes[\"y\"] = attributes[\"x\"]",
        )
        .unwrap()
        .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 2)
    }

    #[test]
    fn test_does_not_combine_set_expressions_for_attributes_when_source_reassigned_in_nested_expr()
    {
        let pipeline_expr = OplParser::parse(
            "logs | set attributes[\"x\"] = 5 | set attributes[\"y\"] = attributes[\"z\"] * attributes[\"x\"]",
        )
        .unwrap()
        .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 2)
    }

    #[test]
    fn test_combine_key_state_reset_between_combinations() {
        let pipeline_expr = OplParser::parse(
            "logs | set attributes[\"x\"] = 5 | set attributes[\"y\"] = 5 | set attributes[\"x\"] = 6 | set attributes[\"y\"] = 7",
        )
        .unwrap()
        .pipeline;
        let planner = PipelinePlanner::new();
        let stages = planner
            .plan_stages(
                &pipeline_expr,
                &Pipeline::create_session_context(),
                &OtapArrowRecords::Logs(Logs::default()), // empty placeholder
            )
            .unwrap();
        assert_eq!(stages.len(), 2)
    }
}
