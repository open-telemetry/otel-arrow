// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::array::RecordBatch;
use arrow::compute::concat_batches;
use data_engine_expressions::{
    ConditionalDataExpression, DataExpression, LogicalExpression, MutableValueExpression,
    PipelineExpression, ScalarExpression, SetTransformExpression, StaticScalarExpression,
    StringValue, TransformExpression, ValueAccessor,
};
use datafusion::catalog::MemTable;
use datafusion::common::JoinType;
use datafusion::execution::TaskContext;
use datafusion::functions::core::expr_ext::FieldAccessor;
use datafusion::functions_window::expr_fn::row_number;
use datafusion::logical_expr::select_expr::SelectExpr;
use datafusion::logical_expr::{Expr, LogicalPlanBuilder, col};
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::common::collect;
use datafusion::physical_plan::{displayable, execute_stream, ExecutionPlan};
use datafusion::prelude::{SessionConfig, SessionContext};

use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts;

use crate::common::{AttributesIdentifier, ColumnAccessor, try_static_scalar_to_literal};
use crate::consts::ROW_NUMBER_COL;
use crate::consts::{ATTRIBUTES_FIELD_NAME, RESOURCES_FIELD_NAME, SCOPE_FIELD_NAME};
use crate::datasource::exec::UpdateDataSourceOptimizer;
use crate::datasource::table_provider::OtapBatchTable;
use crate::error::{Error, Result};
use crate::filter::Filter;

#[derive(Default)]
pub struct OtapBatchEngine {}

impl OtapBatchEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn execute(
        &self,
        pipeline: &PipelineExpression,
        mut exec_ctx: &mut ExecutionContext,
    ) -> Result<()> {
        if exec_ctx.planned_execution.is_none() {
            self.plan(&mut exec_ctx, pipeline).await?;

            // do physical planning (TODO this could be a method on exec_ctx)
            let state = exec_ctx.session_ctx.state();
            let mut logical_plan = exec_ctx.curr_plan.clone().build()?;
            logical_plan = state.optimize(&logical_plan)?;
            let physical_plan = state.create_physical_plan(&logical_plan).await?;

            exec_ctx.planned_execution = Some(PlannedExecution {
                task_context: Arc::new(TaskContext::from(&state)),
                root_physical_plan: physical_plan,
                // TODO I can't decide when to plan these, so just deferring it until we need them
                // which may or may not be correct
                root_attrs_filter: None,
                scope_attrs_filter: None,
                resource_attrs_filter: None,
            })
        }

        let planned_exec = exec_ctx
            .planned_execution
            .as_ref()
            .expect("should be initialized");

        // TODO this could be a method on planned_exec
        let stream = execute_stream(
            Arc::clone(&planned_exec.root_physical_plan),
            Arc::clone(&planned_exec.task_context),
        )?;
        let batches = collect(stream).await?;

        let result = if !batches.is_empty() {
            // TODO not sure concat_batches is necessary here as there should only be one batch.
            // need to double check if any repartitioning happens that could cause multiple batches
            // safety: this shouldn't fail because all the batches should have same schema
            // (datafusion enforces this)
            let mut result =
                concat_batches(batches[0].schema_ref(), &batches).expect("can concat batches");

            // remove the ROW_ID col
            if let Ok(col_idx) = result.schema_ref().index_of(ROW_NUMBER_COL) {
                _ = result.remove_column(col_idx);
            }

            result
        } else {
            let root_batch = exec_ctx
                .curr_batch
                .get(exec_ctx.root_batch_payload_type()?)
                .ok_or(Error::InvalidBatchError {
                    reason: "received OTAP batch missing root RecordBatch".into(),
                })?;
            RecordBatch::new_empty(root_batch.schema())
        };

        exec_ctx
            .curr_batch
            .set(exec_ctx.root_batch_payload_type()?, result);

        // TODO this is a hack to get the correct batch in the right place before updating the children
        exec_ctx.update_batch(exec_ctx.curr_batch.clone())?;

        // // update the attributes
        // // TODO -- we only need to do this if there was filtering applied to the root batch?
        self.filter_attrs_for_root(&mut exec_ctx, ArrowPayloadType::LogAttrs)
            .await?;
        self.filter_attrs_for_root(&mut exec_ctx, ArrowPayloadType::ResourceAttrs)
            .await?;
        self.filter_attrs_for_root(&mut exec_ctx, ArrowPayloadType::ScopeAttrs)
            .await?;

        Ok(())
    }

    pub async fn plan(
        &self,
        exec_ctx: &mut ExecutionContext,
        pipeline: &PipelineExpression,
    ) -> Result<()> {
        for data_expr in pipeline.get_expressions() {
            self.plan_data_expr(exec_ctx, data_expr).await?;
        }

        Ok(())
    }

    async fn plan_data_expr(
        &self,
        exec_ctx: &mut ExecutionContext,
        data_expr: &DataExpression,
    ) -> Result<()> {
        match data_expr {
            DataExpression::Discard(discard) => {
                if let Some(predicate) = discard.get_predicate() {
                    match predicate {
                        // we do opposite of the discard predicate. e.g. keep what would be discarded
                        // note: this is effectively where we're handling the "where" clause of OPL
                        LogicalExpression::Not(not_expr) => {
                            self.plan_filter(exec_ctx, not_expr.get_inner_expression())
                                .await?;
                        }
                        _ => {
                            return Err(Error::InvalidPipelineError {
                                reason: format!(
                                    "expected Discard data expression to contain a Not predicate as root of logical expression tree. Received: {:?}",
                                    predicate
                                ),
                            });
                        }
                    }
                }
            }

            DataExpression::Transform(transform_expr) => {
                match transform_expr {
                    TransformExpression::Set(set_expr) => {
                        self.plan_set_field(exec_ctx, set_expr).await?
                    }

                    // TODO handle other types of transforms like map reduction, map rename, etc.
                    expr => {
                        return Err(Error::NotYetSupportedError {
                            message: format!("transform operation not yet supported {:?}", expr),
                        });
                    }
                }
            }
            DataExpression::Conditional(conditional_expr) => {
                self.plan_conditional(exec_ctx, conditional_expr).await?
            }
            DataExpression::Summary(_) => {
                return Err(Error::InvalidPipelineError {
                    reason: "Summary type data expressions are not supported by columnar engine"
                        .into(),
                });
            }
        }
        Ok(())
    }

    async fn plan_filter(
        &self,
        exec_ctx: &mut ExecutionContext,
        predicate: &LogicalExpression,
    ) -> Result<()> {
        let filter = Filter::try_from_predicate(exec_ctx, predicate).await?;
        let mut root_plan = exec_ctx.root_batch_plan()?;
        if let Some(expr) = filter.filter_expr {
            root_plan = root_plan.filter(expr)?;
        }

        if let Some(join) = filter.join {
            root_plan = join.join_to_plan(root_plan)?;
        }

        // update the current plan now that filters are applied
        exec_ctx.curr_plan = root_plan;

        Ok(())
    }

    async fn plan_set_field(
        &self,
        exec_ctx: &mut ExecutionContext,
        set: &SetTransformExpression,
    ) -> Result<()> {
        // TODO here we're setting the column from a literal, which is not quite correct.
        // ideally, we'd want to figure out if this column can be dictionary encoded, and if so
        // what is the minimum key size, and use the dict builder to compute the column value.
        //
        // TODO also handle the case where we're setting an optional column to null, in which case
        // it's possible we could drop the column.
        let source_expr = match set.get_source() {
            ScalarExpression::Static(static_scalar) => try_static_scalar_to_literal(static_scalar)?,
            source => {
                return Err(Error::NotYetSupportedError {
                    message: format!(
                        "only setting value from static literal source is currently supported. received {:?}",
                        source
                    ),
                });
            }
        };

        let column_name = match set.get_destination() {
            MutableValueExpression::Source(source) => {
                let column_accessor = ColumnAccessor::try_from(source.get_value_accessor())?;
                match column_accessor {
                    ColumnAccessor::ColumnName(column_name) => column_name,
                    column_accessor => {
                        return Err(Error::NotYetSupportedError {
                            message: format!(
                                "only setting non-nested column on root batch is current supported. received {:?}",
                                column_accessor
                            ),
                        });
                    }
                }
            }
            MutableValueExpression::Variable(var) => {
                return Err(Error::NotYetSupportedError {
                    message: format!(
                        "only setting fields are supported. received variable {:?}",
                        var
                    ),
                });
            }
        };

        let root_plan = exec_ctx.root_batch_plan()?;
        let new_col = source_expr.alias(&column_name);
        let mut col_exists = false;

        // select all current columns, replacing the column we're setting by name
        let mut selection: Vec<Expr> = root_plan
            .schema()
            .fields()
            .iter()
            .map(|field| {
                if field.name() == &column_name {
                    col_exists = true;
                    new_col.clone()
                } else {
                    col(field.name())
                }
            })
            .collect();

        // if the column replacement wasn't made (because col doesn't currently exist) add the new
        // column to the batch
        if !col_exists {
            selection.push(new_col);
        }

        let select_exprs = selection.into_iter().map(SelectExpr::Expression);

        exec_ctx.curr_plan = root_plan.project(select_exprs)?;

        Ok(())
    }

    async fn plan_conditional(
        &self,
        exec_ctx: &mut ExecutionContext,
        conditional_expr: &ConditionalDataExpression,
    ) -> Result<()> {
        let branches = conditional_expr.get_branches();

        // handle if branch
        let (if_cond, if_data_exprs) = &branches[0];
        let mut if_exec_ctx = exec_ctx.clone();
        self.plan_filter(&mut if_exec_ctx, if_cond).await?;

        // save the filtered_plan
        let filtered_plan = if_exec_ctx.root_batch_plan()?;

        // apply the data expressions inside the if plan
        for data_expr in if_data_exprs {
            // note: Box::pin here is required for recursion in async
            Box::pin(self.plan_data_expr(&mut if_exec_ctx, data_expr)).await?;
        }

        let mut result_plan = if_exec_ctx.curr_plan;

        // build the plan for everything not selected by the if statement
        let root_plan = exec_ctx.root_batch_plan()?;

        // TODO -- if the filter condition didn't have joins, we could probably
        // actually just use a `not(filter_cond)` here and avoid the join
        let mut next_branch_plan = root_plan.join(
            filtered_plan.build()?,
            JoinType::LeftAnti,
            (vec![ROW_NUMBER_COL], vec![ROW_NUMBER_COL]),
            None,
        )?;

        // handle all the `else if`s
        for branch in branches.iter().skip(1) {
            let (else_if_cond, else_if_data_exprs) = &branch;
            let mut else_if_exec_ctx = exec_ctx.clone();
            else_if_exec_ctx.curr_plan = next_branch_plan.clone();

            // apply the filter steps to everything not selected in the if
            self.plan_filter(&mut else_if_exec_ctx, else_if_cond)
                .await?;

            // save the filter plan
            let filtered_plan = else_if_exec_ctx.root_batch_plan()?;

            // apply the data expressions to rows that matches the else if condition
            for data_expr in else_if_data_exprs {
                Box::pin(self.plan_data_expr(&mut else_if_exec_ctx, data_expr)).await?;
            }

            // update the result to union the results of the `if` branch with the results of this
            // `else if` branch
            result_plan = result_plan.union(else_if_exec_ctx.curr_plan.build()?)?;

            // the next branch will receive everything that didn't match the previous branches
            // and also didn't match this branch's conditions
            // TODO -- if the filter condition didn't have joins, we could probably
            // actually just use a `not(filter_cond)` here and avoid the join
            next_branch_plan = next_branch_plan.join(
                filtered_plan.build()?,
                JoinType::LeftAnti,
                (vec![ROW_NUMBER_COL], vec![ROW_NUMBER_COL]),
                None,
            )?;
        }

        // handle the else branch
        let else_plan = match conditional_expr.get_default_branch() {
            Some(else_data_exprs) => {
                // apply the pipeline to the leftover data
                let mut else_exec_ctx = exec_ctx.clone();
                else_exec_ctx.curr_plan = next_branch_plan;
                for data_expr in else_data_exprs {
                    Box::pin(self.plan_data_expr(&mut else_exec_ctx, data_expr)).await?;
                }
                else_exec_ctx.curr_plan
            }

            None => {
                // if there's no else branch, we treat is a noop and just return the leftover data
                next_branch_plan
            }
        };

        exec_ctx.curr_plan = result_plan.union(else_plan.build()?)?;

        Ok(())
    }

    async fn filter_attrs_for_root(
        &self,
        exec_ctx: &mut ExecutionContext,
        payload_type: ArrowPayloadType,
    ) -> Result<()> {
        if let Some(attrs_rb) = exec_ctx.curr_batch.get(payload_type) {
            // TODO tons of unwraps below

            let stream = match payload_type {
                ArrowPayloadType::ResourceAttrs => {
                    if exec_ctx
                        .planned_execution
                        .as_ref()
                        .unwrap()
                        .resource_attrs_filter
                        .is_none()
                    {
                        let attrs_table_scan = exec_ctx
                            .scan_batch_plan(payload_type)
                            .await?
                            .join_on(
                                exec_ctx.root_batch_plan()?.build()?,
                                JoinType::LeftSemi,
                                [col(consts::RESOURCE)
                                    .field(consts::ID)
                                    .eq(col(consts::PARENT_ID))],
                            )?
                            .build()?;
                        let state = exec_ctx.session_ctx.state();
                        let logical_plan = state.optimize(&attrs_table_scan)?;
                        let physical_plan = state.create_physical_plan(&logical_plan).await?;
                        let planned_exec = exec_ctx.planned_execution.as_mut().unwrap();
                        planned_exec.resource_attrs_filter = Some(physical_plan);
                    }
                    let planned_exec = exec_ctx.planned_execution.as_ref().unwrap();
                    let phy_plan = planned_exec
                        .resource_attrs_filter
                        .as_ref()
                        .expect("root attrs filter plan initialized");
                    execute_stream(Arc::clone(phy_plan), Arc::clone(&planned_exec.task_context))?
                }
                ArrowPayloadType::ScopeAttrs => {
                    if exec_ctx
                        .planned_execution
                        .as_ref()
                        .unwrap()
                        .scope_attrs_filter
                        .is_none()
                    {
                        let attrs_table_scan = exec_ctx
                            .scan_batch_plan(payload_type)
                            .await?
                            .join_on(
                                exec_ctx.root_batch_plan()?.build()?,
                                JoinType::LeftSemi,
                                [col(consts::SCOPE)
                                    .field(consts::ID)
                                    .eq(col(consts::PARENT_ID))],
                            )?
                            .build()?;
                        let state = exec_ctx.session_ctx.state();
                        let logical_plan = state.optimize(&attrs_table_scan)?;
                        let physical_plan = state.create_physical_plan(&logical_plan).await?;
                        let planned_exec = exec_ctx.planned_execution.as_mut().unwrap();
                        planned_exec.scope_attrs_filter = Some(physical_plan);
                    }
                    let planned_exec = exec_ctx.planned_execution.as_ref().unwrap();
                    let phy_plan = planned_exec
                        .scope_attrs_filter
                        .as_ref()
                        .expect("root attrs filter plan initialized");
                    execute_stream(Arc::clone(phy_plan), Arc::clone(&planned_exec.task_context))?
                }

                // root attributes
                _ => {
                    if exec_ctx
                        .planned_execution
                        .as_ref()
                        .unwrap()
                        .root_attrs_filter
                        .is_none()
                    {
                        let attrs_table_scan = exec_ctx
                            .scan_batch_plan(payload_type)
                            .await?
                            .join(
                                // exec_ctx.root_batch_plan()?.build()?,
                                exec_ctx.scan_batch_plan(ArrowPayloadType::Logs).await?.build()?,
                                JoinType::LeftSemi,
                                (vec![consts::PARENT_ID], vec![consts::ID]),
                                None,
                            )?
                            .build()?;
                        let state = exec_ctx.session_ctx.state();
                        let logical_plan = state.optimize(&attrs_table_scan)?;
                        let physical_plan = state.create_physical_plan(&logical_plan).await?;
                        let planned_exec = exec_ctx.planned_execution.as_mut().unwrap();
                        planned_exec.root_attrs_filter = Some(physical_plan);
                    }
                    let planned_exec = exec_ctx.planned_execution.as_ref().unwrap();
                    let phy_plan = planned_exec
                        .root_attrs_filter
                        .as_ref()
                        .expect("root attrs filter plan initialized");
                    execute_stream(Arc::clone(phy_plan), Arc::clone(&planned_exec.task_context))?
                }
            };

            let batches = collect(stream).await?;
            let result = if !batches.is_empty() {
                // safety: this shouldn't fail unless batches don't have matching schemas, but they
                // will b/c datafusion enforces this
                concat_batches(batches[0].schema_ref(), &batches).expect("can concat batches")
            } else {
                RecordBatch::new_empty(attrs_rb.schema())
            };

            exec_ctx.curr_batch.set(payload_type, result);
        }

        Ok(())
    }
}

// TODO rethink how the internal state in this thing is structured.
// Currently it weirdly contains planning stuff for both logical and
// physical plans and only one can be used at a time
#[derive(Clone)]
pub struct ExecutionContext {
    curr_plan: LogicalPlanBuilder,

    // TODO doesn't maybe need to be pub these next 2 members?
    pub curr_batch: OtapArrowRecords,
    pub session_ctx: SessionContext,

    planned_execution: Option<PlannedExecution>,
}

// TODO there is so much duplicated code in this struct mama mia
impl ExecutionContext {
    pub fn print_phy_plans(&self) {
        let planned_ex = self.planned_execution.as_ref().unwrap();
        let plan = planned_ex.root_physical_plan.as_ref();
        
        println!("\nroot plan");
        let dp = displayable(plan);
        println!("{}", dp.indent(true));

        println!("\nroot attrs plan:");
        if let Some(plan) = planned_ex.root_attrs_filter.as_ref() {
            let dp = displayable(plan.as_ref());
            println!("{}", dp.indent(true));
        } else {
            println!("None")
        }

        println!("\nscope attrs plan:");
        if let Some(plan) = planned_ex.scope_attrs_filter.as_ref() {
            let dp = displayable(plan.as_ref());
            println!("{}", dp.indent(true));
        } else {
            println!("None")
        }

        println!("\nres attrs plan:");
        if let Some(plan) = planned_ex.scope_attrs_filter.as_ref() {
            let dp = displayable(plan.as_ref());
            println!("{}", dp.indent(true));
        } else {
            println!("None")
        }
        
    }

    pub fn update_batch(&mut self, otap_batch: OtapArrowRecords) -> Result<()> {
        let plan_batch_updater = UpdateDataSourceOptimizer::new(otap_batch);
        // TODO avoid copying the config
        let session_config = self.session_ctx.copied_config();
        let config_options = session_config.options();

        if let Some(planned_execution) = &mut self.planned_execution {
            let updated_root_plan = plan_batch_updater.optimize(
                planned_execution.root_physical_plan.clone(),
                config_options.as_ref(),
            )?;
            planned_execution.root_physical_plan = updated_root_plan;

            if let Some(attrs_plan) = planned_execution.root_attrs_filter.take() {
                let updated_plan =
                    plan_batch_updater.optimize(attrs_plan, config_options.as_ref())?;
                planned_execution.root_attrs_filter = Some(updated_plan);
            }

            if let Some(res_attrs_plan) = planned_execution.resource_attrs_filter.take() {
                let updated_plan =
                    plan_batch_updater.optimize(res_attrs_plan, config_options.as_ref())?;
                planned_execution.resource_attrs_filter = Some(updated_plan);
            }

            if let Some(scope_attrs_plan) = planned_execution.resource_attrs_filter.take() {
                let updated_plan =
                    plan_batch_updater.optimize(scope_attrs_plan, config_options.as_ref())?;
                planned_execution.resource_attrs_filter = Some(updated_plan);
            }
        }

        Ok(())
    }

    pub async fn try_new(batch: OtapArrowRecords) -> Result<Self> {
        // TODO this logic is also duplicated below (should this just be a method on OtapArrowRecords?)
        let root_batch_payload_type = match batch {
            OtapArrowRecords::Logs(_) => ArrowPayloadType::Logs,
            _ => {
                return Err(Error::NotYetSupportedError {
                    message: "Only logs signal type is currently supported".into(),
                });
            }
        };
        let root_rb = batch
            .get(root_batch_payload_type)
            .ok_or(Error::InvalidBatchError {
                reason: "received OTAP batch missing root RecordBatch".into(),
            })?;

        // TODO this logic is temporarily duplicated from scan_batch until figure out whether it
        // makes more sense to just register everything in session ctx

        let session_config = SessionConfig::new()
            // since we're executing always in single threaded runtime it doesn't really make
            // sense to spawn repartition tasks to repartition to do things in parallel.
            .with_target_partitions(1)
            .with_repartition_joins(false)
            .with_repartition_file_scans(false)
            .with_repartition_windows(false)
            .with_repartition_aggregations(false);

        let session_ctx = SessionContext::new_with_config(session_config);

        let table_name = format!("{:?}", root_batch_payload_type).to_lowercase();
        let table = OtapBatchTable::new(root_batch_payload_type, root_rb.clone());
        session_ctx.register_table(&table_name, Arc::new(table))?;

        let table_df = session_ctx.table(table_name).await?;
        let plan = LogicalPlanBuilder::from(table_df.logical_plan().clone());

        // add a row number column
        // TODO comment on why we're doing this
        // TODO this is a performance issue
        // let plan = plan.window(vec![row_number().alias(ROW_NUMBER_COL)])?;

        Ok(Self {
            curr_batch: batch,
            curr_plan: plan,
            session_ctx,
            planned_execution: None,
        })
    }

    pub fn root_batch_payload_type(&self) -> Result<ArrowPayloadType> {
        match self.curr_batch {
            OtapArrowRecords::Logs(_) => Ok(ArrowPayloadType::Logs),
            _ => Err(Error::NotYetSupportedError {
                message: "Only logs signal type is currently supported".into(),
            }),
        }
    }

    pub fn root_batch_plan(&self) -> Result<LogicalPlanBuilder> {
        Ok(self.curr_plan.clone())
    }

    pub async fn scan_batch_plan(
        &self,
        payload_type: ArrowPayloadType,
    ) -> Result<LogicalPlanBuilder> {
        if let Some(rb) = self.curr_batch.get(payload_type) {
            // TODO make this a method
            let table_name = format!("{:?}", payload_type).to_ascii_lowercase();
            if !self.session_ctx.table_exist(&table_name)? {
                let table_name = format!("{:?}", payload_type).to_lowercase();
                // let table = MemTable::try_new(rb.schema(), vec![vec![rb.clone()]])?;
                let table = OtapBatchTable::new(payload_type, rb.clone());
                self.session_ctx
                    .register_table(&table_name, Arc::new(table))?;
            }

            let table_df = self.session_ctx.table(table_name).await?;
            let plan = LogicalPlanBuilder::from(table_df.logical_plan().clone());

            Ok(plan)
        } else {
            Err(Error::InvalidBatchError {
                reason: format!(
                    "cannot plan to scan batch {:?}. it is not present in OTAP batch",
                    payload_type
                ),
            })
        }
    }

    pub fn column_exists(&self, accessor: &ColumnAccessor) -> Result<bool> {
        Ok(match accessor {
            ColumnAccessor::ColumnName(column_name) => {
                // TODO - eventually we might need to loosen the assumption that this is
                // a column on the root batch
                if let Some(rb) = self.curr_batch.get(self.root_batch_payload_type()?) {
                    rb.column_by_name(column_name).is_some()
                } else {
                    // it'd be unusual if the root batch didn't exit
                    false
                }
            }
            _ => {
                // TODO handle checking if attributes exist and if nested struct columns exist.
                // For attributes, we might need to change the signature because this might need
                // to get called with the value column of any AnyValue (e.g. attributes str column)
                true
            }
        })
    }
}

// TODO comments on what this is about
#[derive(Clone)]
pub struct PlannedExecution {
    task_context: Arc<TaskContext>,
    root_physical_plan: Arc<dyn ExecutionPlan>,

    root_attrs_filter: Option<Arc<dyn ExecutionPlan>>,
    scope_attrs_filter: Option<Arc<dyn ExecutionPlan>>,
    resource_attrs_filter: Option<Arc<dyn ExecutionPlan>>,
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
                reason: format!(
                    "unsupported nested struct column definition for struct {}. received {:?}",
                    struct_column_name, expr
                ),
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
                reason: format!("unsupported column definition. received {:?}", expr),
            }),
        }
    }
}

#[cfg(test)]
mod test {
    use arrow::array::{DictionaryArray, StringArray};
    use arrow::datatypes::UInt8Type;
    use data_engine_expressions::{
        ConditionalDataExpressionBuilder, DataExpression, LogicalExpression,
        PipelineExpressionBuilder,
    };
    use data_engine_kql_parser::{KqlParser, Parser, ParserOptions};
    use otel_arrow_rust::proto::opentelemetry::{arrow::v1::ArrowPayloadType, logs::v1::LogRecord};
    use otel_arrow_rust::schema::consts;

    use crate::engine::{ExecutionContext, OtapBatchEngine};
    use crate::test::{apply_to_logs, generate_logs_batch, logs_to_export_req};

    #[tokio::test]
    async fn test_simple_extend_new_column() {
        let log_records = logs_to_export_req(vec![LogRecord {
            ..Default::default()
        }]);

        let kql = "logs | extend severity_text = \"WARN\"";
        let pipeline = KqlParser::parse_with_options(kql, ParserOptions::default()).unwrap();

        let result = apply_to_logs(log_records, pipeline).await;
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();

        let severity_text = logs_rb
            .column_by_name(consts::SEVERITY_TEXT)
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        for t in severity_text.iter() {
            assert_eq!(t, Some("WARN"))
        }
    }

    #[tokio::test]
    async fn test_simple_extend_replace_column() {
        let log_records = logs_to_export_req(vec![LogRecord {
            severity_text: "INFO".into(),
            ..Default::default()
        }]);

        let kql = "logs | extend severity_text = \"WARN\"";
        let pipeline = KqlParser::parse_with_options(kql, ParserOptions::default()).unwrap();

        let result = apply_to_logs(log_records, pipeline).await;
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();

        let severity_text = logs_rb
            .column_by_name(consts::SEVERITY_TEXT)
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();
        for t in severity_text.iter() {
            assert_eq!(t, Some("WARN"))
        }
    }

    #[tokio::test]
    async fn test_simple_extend_multiple_columns() {
        let log_records = logs_to_export_req(vec![LogRecord {
            severity_text: "INFO".into(),
            ..Default::default()
        }]);

        let kql = "logs | extend severity_text = \"WARN\" | extend event_name = \"test\"";
        let pipeline = KqlParser::parse_with_options(kql, ParserOptions::default()).unwrap();

        let result = apply_to_logs(log_records, pipeline).await;
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();

        let severity_text = logs_rb
            .column_by_name(consts::SEVERITY_TEXT)
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();

        for t in severity_text.iter() {
            assert_eq!(t, Some("WARN"))
        }

        let event_name = logs_rb
            .column_by_name(consts::EVENT_NAME)
            .unwrap()
            .as_any()
            .downcast_ref::<StringArray>()
            .unwrap();

        for t in event_name.iter() {
            assert_eq!(t, Some("test"))
        }
    }

    // TODO the KQL parser doesn't yet support our if/else syntax, so to build a pipeline with
    // this type of expression in it, we need to cheese it
    struct IfElseExpressions {
        if_condition: &'static str,
        if_branch: &'static str,

        // tuples here are (condition, branch data exprs)
        else_ifs: Vec<(&'static str, &'static str)>,

        else_branch: Option<&'static str>,
    }

    impl IfElseExpressions {
        fn into_data_expr(self) -> DataExpression {
            let if_condition = parse_to_condition(self.if_condition);
            let if_branch_data_exprs = parse_to_data_exprs(self.if_branch);
            let mut if_expr_builder =
                ConditionalDataExpressionBuilder::from_if(if_condition, if_branch_data_exprs);

            for (condition, branch) in self.else_ifs {
                if_expr_builder = if_expr_builder
                    .with_else_if(parse_to_condition(condition), parse_to_data_exprs(branch));
            }

            if_expr_builder = match self.else_branch {
                Some(branch) => if_expr_builder.with_else(parse_to_data_exprs(branch)),
                None => if_expr_builder,
            };
            DataExpression::Conditional(if_expr_builder.build())
        }
    }

    fn parse_to_condition(condition: &str) -> LogicalExpression {
        let pipeline_expr = KqlParser::parse(&format!("i | where {}", condition)).unwrap();
        let pipeline_exprs = pipeline_expr.get_expressions();
        if let DataExpression::Discard(discard_expr) = &pipeline_exprs[0] {
            if let LogicalExpression::Not(not_expr) = discard_expr.get_predicate().unwrap() {
                return not_expr.get_inner_expression().clone();
            }
        }

        panic!("invalid pipeline {}", pipeline_expr);
    }

    fn parse_to_data_exprs(pipeline_exprs: &str) -> Vec<DataExpression> {
        let pipeline_expr = KqlParser::parse(&format!("i |{}", pipeline_exprs)).unwrap();
        let pipeline_exprs = pipeline_expr.get_expressions();
        pipeline_exprs.to_vec()
    }

    #[tokio::test]
    async fn test_simple_if() {
        let if_expr = IfElseExpressions {
            if_condition: "severity_text == \"INFO\"",
            if_branch: "extend severity_text = \"DEBUG\"",
            else_ifs: vec![],
            else_branch: None,
        };

        let pipeline_expr = PipelineExpressionBuilder::new("")
            .with_expressions(vec![if_expr.into_data_expr()])
            .build()
            .unwrap();

        let log_records = logs_to_export_req(vec![
            LogRecord {
                severity_text: "INFO".into(),
                ..Default::default()
            },
            LogRecord {
                severity_text: "WARN".into(),
                ..Default::default()
            },
        ]);

        let result = apply_to_logs(log_records, pipeline_expr).await;
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();

        let severity_column = logs_rb
            .column_by_name(consts::SEVERITY_TEXT)
            .unwrap()
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap()
            .downcast_dict::<StringArray>()
            .unwrap()
            .into_iter()
            .filter_map(|s| s.map(|s| s.to_string()))
            .collect::<Vec<_>>();

        let expected = vec!["DEBUG".to_string(), "WARN".to_string()];
        assert_eq!(severity_column, expected);
    }

    #[tokio::test]
    async fn test_simple_if_else() {
        let if_expr = IfElseExpressions {
            if_condition: "severity_text == \"INFO\"",
            if_branch: "extend severity_text = \"DEBUG\"",
            else_ifs: vec![],
            else_branch: Some("extend severity_text = \"ERROR\""),
        };

        let pipeline_expr = PipelineExpressionBuilder::new("")
            .with_expressions(vec![if_expr.into_data_expr()])
            .build()
            .unwrap();

        let log_records = logs_to_export_req(vec![
            LogRecord {
                severity_text: "INFO".into(),
                ..Default::default()
            },
            LogRecord {
                severity_text: "WARN".into(),
                ..Default::default()
            },
        ]);

        let result = apply_to_logs(log_records, pipeline_expr).await;
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();

        let severity_column = logs_rb
            .column_by_name(consts::SEVERITY_TEXT)
            .unwrap()
            .as_any()
            // TODO need to fix the issue where the dict column gets replaced by a String column
            // .downcast_ref::<DictionaryArray<UInt8Type>>()
            // .unwrap()
            // .downcast_dict::<StringArray>()
            .downcast_ref::<StringArray>()
            .unwrap()
            .into_iter()
            .filter_map(|s| s.map(|s| s.to_string()))
            .collect::<Vec<_>>();

        let expected = vec!["DEBUG".to_string(), "ERROR".to_string()];
        assert_eq!(severity_column, expected);
    }

    #[tokio::test]
    async fn test_simple_if_else_if() {
        let if_expr = IfElseExpressions {
            if_condition: "severity_text == \"INFO\"",
            if_branch: "extend severity_text = \"DEBUG\"",
            else_ifs: vec![(
                "severity_text == \"WARN\"",
                "extend severity_text = \"ERROR\"",
            )],
            else_branch: Some("extend severity_text = \"TRACE\""),
        };

        let pipeline_expr = PipelineExpressionBuilder::new("")
            .with_expressions(vec![if_expr.into_data_expr()])
            .build()
            .unwrap();

        let log_records = logs_to_export_req(vec![
            LogRecord {
                severity_text: "INFO".into(), // -> DEBUG
                ..Default::default()
            },
            LogRecord {
                severity_text: "WARN".into(), // -> ERROR
                ..Default::default()
            },
            LogRecord {
                severity_text: "DEBUG".into(), // -> TRACE
                ..Default::default()
            },
        ]);

        let result = apply_to_logs(log_records, pipeline_expr).await;
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();
        let severity_column = logs_rb
            .column_by_name(consts::SEVERITY_TEXT)
            .unwrap()
            .as_any()
            // TODO need to fix the issue where the dict column gets replaced by a String column
            // .downcast_ref::<DictionaryArray<UInt8Type>>()
            // .unwrap()
            // .downcast_dict::<StringArray>()
            .downcast_ref::<StringArray>()
            .unwrap()
            .into_iter()
            .filter_map(|s| s.map(|s| s.to_string()))
            .collect::<Vec<_>>();

        let expected = vec![
            "DEBUG".to_string(),
            "ERROR".to_string(),
            "TRACE".to_string(),
        ];
        assert_eq!(severity_column, expected);
    }

    #[tokio::test]
    async fn test_filter_after_update() {
        let if_expr = IfElseExpressions {
            if_condition: "severity_text == \"INFO\"",
            if_branch: "extend severity_text = \"DEBUG\"",
            else_ifs: vec![],
            else_branch: None,
        };

        let filter_expr = parse_to_data_exprs("where severity_text == \"DEBUG\"")[0].clone();

        let pipeline_expr = PipelineExpressionBuilder::new("")
            .with_expressions(vec![if_expr.into_data_expr(), filter_expr])
            .build()
            .unwrap();

        let log_records = logs_to_export_req(vec![
            LogRecord {
                severity_text: "INFO".into(), // -> DEBUG
                event_name: "1".into(),
                ..Default::default()
            },
            LogRecord {
                severity_text: "WARN".into(), // no change
                event_name: "2".into(),
                ..Default::default()
            },
            LogRecord {
                severity_text: "DEBUG".into(), // no change
                event_name: "3".into(),
                ..Default::default()
            },
        ]);

        let result = apply_to_logs(log_records, pipeline_expr).await;
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();
        let event_name_column = logs_rb
            .column_by_name(consts::EVENT_NAME)
            .unwrap()
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap()
            .downcast_dict::<StringArray>()
            .unwrap()
            .into_iter()
            .filter_map(|s| s.map(|s| s.to_string()))
            .collect::<Vec<_>>();

        let expected = vec!["1".to_string(), "3".to_string()];
        assert_eq!(event_name_column, expected);
    }

    #[tokio::test]
    async fn test_reuse_plans_simple() {
        let query = "logs | where severity_text == \"WARN\"";
        let pipeline_expr = KqlParser::parse(query).unwrap();

        let batch1 = generate_logs_batch(32, 100);
        let mut exec_ctx = ExecutionContext::try_new(batch1).await.unwrap();

        let engine = OtapBatchEngine::new();

        engine.execute(&pipeline_expr, &mut exec_ctx).await.unwrap();
        let result1 = exec_ctx.curr_batch.clone();

        let batch2 = generate_logs_batch(64, 200);
        exec_ctx.update_batch(batch2).unwrap();

        engine.execute(&pipeline_expr, &mut exec_ctx).await.unwrap();
        let result2 = exec_ctx.curr_batch.clone();

        // TODO need to add some real asserts here

        println!("result1 logs:");
        arrow::util::pretty::print_batches(&[result1.get(ArrowPayloadType::Logs).unwrap().clone()])
            .unwrap();
        println!("result1 log_attrs:");
        arrow::util::pretty::print_batches(&[result1
            .get(ArrowPayloadType::LogAttrs)
            .unwrap()
            .clone()])
        .unwrap();

        println!("result2 logs:");
        arrow::util::pretty::print_batches(&[result2.get(ArrowPayloadType::Logs).unwrap().clone()])
            .unwrap();
        println!("result2 log_attrs:");
        arrow::util::pretty::print_batches(&[result2
            .get(ArrowPayloadType::LogAttrs)
            .unwrap()
            .clone()])
        .unwrap();
    }
}
