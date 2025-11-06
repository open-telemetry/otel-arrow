// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::collections::HashSet;
use std::sync::Arc;

use arrow::array::{RecordBatch, UInt16Array};
use arrow::compute::{concat_batches, filter_record_batch};
use arrow::datatypes::DataType;
use data_engine_expressions::{
    ConditionalDataExpression, DataExpression, LogicalExpression, MutableValueExpression,
    PipelineExpression, ScalarExpression, SetTransformExpression, StaticScalarExpression,
    StringValue, TransformExpression, ValueAccessor,
};
use datafusion::common::JoinType;
use datafusion::execution::TaskContext;
use datafusion::functions_window::expr_fn::row_number;
use datafusion::logical_expr::select_expr::SelectExpr;
use datafusion::logical_expr::{Expr, LogicalPlanBuilder, col};
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::common::collect;
use datafusion::physical_plan::{ExecutionPlan, execute_stream};
use datafusion::prelude::{SessionConfig, SessionContext};

use otel_arrow_rust::arrays::{
    get_required_array, get_required_array_from_struct_array_from_record_batch,
};
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::otap::filter::build_uint16_id_filter;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts;

use crate::common::{AttributesIdentifier, ColumnAccessor, try_static_scalar_to_literal};
use crate::consts::ROW_NUMBER_COL;
use crate::consts::{ATTRIBUTES_FIELD_NAME, RESOURCES_FIELD_NAME, SCOPE_FIELD_NAME};
use crate::datasource::exec::UpdateDataSourceOptimizer;
use crate::datasource::table_provider::OtapBatchTable;
use crate::error::{Error, Result};
use crate::filter::Filter;

/// This is used to build a datafusion `LogicalPlan` from a [`PipelineExpression`]
// TODO could add more descriptive comments
#[derive(Clone)]
pub struct PipelinePlanBuilder {
    pub session_ctx: SessionContext,
    pub logical_plan: LogicalPlanBuilder,
    batch: OtapArrowRecords,
}

impl PipelinePlanBuilder {
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

        let session_config = SessionConfig::new()
            // since we're always executing in a single threaded runtime, it doesn't make sense
            // to spawn repartition tasks and run things like join and filtering in parallel
            .with_target_partitions(1)
            .with_repartition_joins(false)
            .with_repartition_file_scans(false)
            .with_repartition_windows(false)
            .with_repartition_aggregations(false)
            .with_repartition_sorts(false);
        let session_ctx = SessionContext::new_with_config(session_config);

        let table_name = format!("{:?}", root_batch_payload_type).to_lowercase();
        let table = OtapBatchTable::new(root_batch_payload_type, root_rb.clone());
        session_ctx.register_table(&table_name, Arc::new(table))?;

        let table_df = session_ctx.table(table_name).await?;
        let logical_plan = LogicalPlanBuilder::from(table_df.logical_plan().clone());

        // TODO need a more efficient version of this. Adding row numbers is slow and sometimes
        // unnecessary depending on the plan
        let logical_plan = logical_plan.window(vec![row_number().alias(ROW_NUMBER_COL)])?;

        Ok(Self {
            session_ctx,
            batch,
            logical_plan,
        })
    }

    pub async fn plan(&mut self, pipeline: &PipelineExpression) -> Result<()> {
        for data_expr in pipeline.get_expressions() {
            self.plan_data_expr(data_expr).await?;
        }

        Ok(())
    }

    async fn plan_data_expr(&mut self, data_expr: &DataExpression) -> Result<()> {
        match data_expr {
            DataExpression::Discard(discard) => {
                if let Some(predicate) = discard.get_predicate() {
                    match predicate {
                        // we do opposite of the discard predicate. e.g. keep what would be discarded
                        // note: this is effectively where we're handling the "where" clause of OPL
                        LogicalExpression::Not(not_expr) => {
                            self.plan_filter(not_expr.get_inner_expression()).await?;
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
                    TransformExpression::Set(set_expr) => self.plan_set_field(set_expr).await?,

                    // TODO handle other types of transforms like map reduction, map rename, etc.
                    expr => {
                        return Err(Error::NotYetSupportedError {
                            message: format!("transform operation not yet supported {:?}", expr),
                        });
                    }
                }
            }
            DataExpression::Conditional(conditional_expr) => {
                self.plan_conditional(conditional_expr).await?
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

    async fn plan_filter(&mut self, predicate: &LogicalExpression) -> Result<()> {
        let filter = Filter::try_from_predicate(self, predicate).await?;
        let mut plan = self.logical_plan.clone();
        if let Some(expr) = filter.filter_expr {
            plan = plan.filter(expr)?;
        }

        if let Some(join) = filter.join {
            plan = join.join_to_plan(plan)?;
        }

        // update the current plan now that filters are applied
        self.logical_plan = plan;

        Ok(())
    }

    async fn plan_set_field(&mut self, set: &SetTransformExpression) -> Result<()> {
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

        let new_col = source_expr.alias(&column_name);
        let mut col_exists = false;

        // select all current columns, replacing the column we're setting by name
        let mut selection: Vec<Expr> = self
            .logical_plan
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

        self.logical_plan = self.logical_plan.clone().project(select_exprs)?;

        Ok(())
    }

    async fn plan_conditional(
        &mut self,
        conditional_expr: &ConditionalDataExpression,
    ) -> Result<()> {
        let branches = conditional_expr.get_branches();

        // handle if branch
        let (if_cond, if_data_exprs) = &branches[0];
        let mut if_plan_builder = self.clone();
        if_plan_builder.plan_filter(if_cond).await?;

        // save the filtered_plan
        let filtered_plan = if_plan_builder.logical_plan.clone();

        // apply the data expressions inside the if plan
        for data_expr in if_data_exprs {
            // note: Box::pin here is required for recursion in async
            Box::pin(if_plan_builder.plan_data_expr(data_expr)).await?;
        }

        let mut result_plan = if_plan_builder.logical_plan;

        // build the plan for everything not selected by the if statement
        let root_plan = self.logical_plan.clone();

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
            let mut else_if_plan_builder = self.clone();
            else_if_plan_builder.logical_plan = next_branch_plan.clone();

            // apply the filter steps to everything not selected in the if
            else_if_plan_builder.plan_filter(else_if_cond).await?;

            // save the filter plan
            let filtered_plan = else_if_plan_builder.logical_plan.clone();

            // apply the data expressions to rows that matches the else if condition
            for data_expr in else_if_data_exprs {
                Box::pin(else_if_plan_builder.plan_data_expr(data_expr)).await?;
            }

            // update the result to union the results of the `if` branch with the results of this
            // `else if` branch
            result_plan = result_plan.union(else_if_plan_builder.logical_plan.build()?)?;

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
                let mut else_plan_builder = self.clone();
                else_plan_builder.logical_plan = next_branch_plan;
                for data_expr in else_data_exprs {
                    Box::pin(else_plan_builder.plan_data_expr(data_expr)).await?;
                }
                else_plan_builder.logical_plan
            }

            None => {
                // if there's no else branch, we treat is a noop and just return the leftover data
                next_branch_plan
            }
        };

        self.logical_plan = result_plan.union(else_plan.build()?)?;

        Ok(())
    }

    pub fn root_batch_payload_type(&self) -> Result<ArrowPayloadType> {
        match self.batch {
            OtapArrowRecords::Logs(_) => Ok(ArrowPayloadType::Logs),
            _ => Err(Error::NotYetSupportedError {
                message: "Only logs signal type is currently supported".into(),
            }),
        }
    }

    pub async fn scan_batch_plan(
        &self,
        payload_type: ArrowPayloadType,
    ) -> Result<LogicalPlanBuilder> {
        if let Some(rb) = self.batch.get(payload_type) {
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
                if let Some(rb) = self.batch.get(self.root_batch_payload_type()?) {
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

pub struct ExecutablePipeline {
    _session_ctx: SessionContext,
    session_config: SessionConfig,
    task_context: Arc<TaskContext>,
    physical_plan: Arc<dyn ExecutionPlan>,
    pub curr_batch: OtapArrowRecords,
}

impl ExecutablePipeline {
    pub async fn try_new(batch: OtapArrowRecords, pipeline: PipelineExpression) -> Result<Self> {
        // populate expressions on logical plan builder
        let mut pipeline_plan_builder = PipelinePlanBuilder::try_new(batch.clone()).await?;
        pipeline_plan_builder.plan(&pipeline).await?;

        // build logical plan:
        let session_ctx = pipeline_plan_builder.session_ctx;
        let state = session_ctx.state();
        let mut logical_plan = pipeline_plan_builder.logical_plan.build()?;
        logical_plan = state.optimize(&logical_plan)?;

        // build physical plan new Self
        let physical_plan = state.create_physical_plan(&logical_plan).await?;
        let session_config = session_ctx.copied_config();
        let task_context = Arc::new(TaskContext::from(&state));

        Ok(Self {
            _session_ctx: session_ctx,
            session_config,
            task_context,
            physical_plan,
            curr_batch: batch,
        })
    }

    pub fn update_batch(&mut self, next_batch: OtapArrowRecords) -> Result<()> {
        let batch_updater = UpdateDataSourceOptimizer::new(next_batch);
        let updated_plan = batch_updater.optimize(
            Arc::clone(&self.physical_plan),
            self.session_config.options(),
        )?;
        self.physical_plan = updated_plan;
        self.curr_batch = batch_updater.take_batch();

        Ok(())
    }

    pub async fn execute(&mut self) -> Result<()> {
        let stream = execute_stream(self.physical_plan.clone(), self.task_context.clone())?;
        let schema = stream.schema();
        let batches = collect(stream).await?;

        let root_payload_type = match self.curr_batch {
            OtapArrowRecords::Logs(_) => ArrowPayloadType::Logs,
            _ => {
                return Err(Error::NotYetSupportedError {
                    message: "only logs are currently supported".into(),
                });
            }
        };

        let mut root_batch = match batches.len() {
            0 => {
                // empty
                RecordBatch::new_empty(schema.clone())
            }

            1 => {
                // single batch
                batches[0].clone()
            }

            _ => {
                // TODO this branch might not even be necessary because we do disable all
                // partitioning so there should only be one batch? need to double check this
                // assumption ...

                // safety: this shouldn't fail because all the batches should have same schema
                concat_batches(batches[0].schema_ref(), &batches).expect("can concat batches")
            }
        };

        // remove the row_number column if it's there ...
        if let Ok(row_num_col_index) = schema.index_of(ROW_NUMBER_COL) {
            _ = root_batch.remove_column(row_num_col_index);
        }

        // remove any placeholder columns that were inserted by the `UpdateDataSourceOptimizer`
        for field_id in (0..schema.fields().len()).rev() {
            if matches!(
                schema.field(field_id).data_type(),
                DataType::RunEndEncoded(_, _)
            ) {
                _ = root_batch.remove_column(field_id)
            }
        }

        self.curr_batch.set(root_payload_type, root_batch);

        self.update_child_batch(ArrowPayloadType::LogAttrs)?;
        self.update_child_batch(ArrowPayloadType::ResourceAttrs)?;
        self.update_child_batch(ArrowPayloadType::ScopeAttrs)?;

        Ok(())
    }

    fn update_child_batch(&mut self, payload_type: ArrowPayloadType) -> Result<()> {
        let child_rb = match self.curr_batch.get(payload_type) {
            Some(rb) => rb,
            None => return Ok(()),
        };

        let root_payload_type = match self.curr_batch {
            OtapArrowRecords::Logs(_) => ArrowPayloadType::Logs,
            _ => {
                return Err(Error::NotYetSupportedError {
                    message: "only logs currently supported".into(),
                });
            }
        };
        let root_rb = self
            .curr_batch
            .get(root_payload_type)
            .ok_or(Error::InvalidBatchError {
                reason: "missing root record batch".into(),
            })?;

        let source_ids = match payload_type {
            ArrowPayloadType::LogAttrs => get_required_array(root_rb, consts::ID),
            ArrowPayloadType::ResourceAttrs => {
                get_required_array_from_struct_array_from_record_batch(
                    root_rb,
                    consts::RESOURCE,
                    consts::ID,
                )
            }
            ArrowPayloadType::ScopeAttrs => get_required_array_from_struct_array_from_record_batch(
                root_rb,
                consts::SCOPE,
                consts::ID,
            ),
            _ => {
                return Err(Error::NotYetSupportedError {
                    message: format!(
                        "updating child payload type of {:?} not yet supported",
                        payload_type
                    ),
                });
            }
        }
        .map_err(|e| Error::InvalidBatchError {
            reason: format!("invalid batch: {e}"),
        })?;

        // TODO not have this hard-coded to u16 IDs
        let ids_as_u16 =
            source_ids
                .as_any()
                .downcast_ref::<UInt16Array>()
                .ok_or(Error::InvalidBatchError {
                    reason: format!("expected u16 array, found type {}", source_ids.data_type()),
                })?;

        let ids_set: HashSet<u16> = ids_as_u16.iter().flatten().collect();
        let target_ids = get_required_array(child_rb, consts::PARENT_ID).map_err(|e| {
            Error::InvalidBatchError {
                reason: format!("invalid batch: {e}"),
            }
        })?;

        // TODO don't unwrap here - convert the error
        // TODO profile these calls and see if there's a faster way to do this. initial profiling
        // shows this is somewhat of a performance bottleneck
        let child_mask = build_uint16_id_filter(target_ids, ids_set).unwrap();
        let result_children = filter_record_batch(child_rb, &child_mask).unwrap();
        self.curr_batch.set(payload_type, result_children);

        Ok(())
    }
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
    use std::sync::Arc;

    use arrow::array::{DictionaryArray, RecordBatch, StringArray, UInt8Array, UInt32Array};
    use arrow::compute::kernels::cast;
    use arrow::compute::take_record_batch;
    use arrow::datatypes::{DataType, Field, Schema, UInt8Type};
    use arrow::util::pretty::pretty_format_batches;
    use data_engine_expressions::{
        ConditionalDataExpressionBuilder, DataExpression, LogicalExpression,
        PipelineExpressionBuilder,
    };
    use data_engine_kql_parser::{KqlParser, Parser, ParserOptions};
    use otel_arrow_rust::proto::opentelemetry::{arrow::v1::ArrowPayloadType, logs::v1::LogRecord};
    use otel_arrow_rust::schema::consts;
    use pretty_assertions::assert_eq;

    use crate::datagen::generate_logs_batch;
    use crate::engine::ExecutablePipeline;
    use crate::test::{apply_to_logs, logs_to_export_req};

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
    async fn test_reuse_plans_simple_filter() {
        let query = "logs | where severity_text == \"WARN\"";
        let pipeline_expr = KqlParser::parse(query).unwrap();

        let batch1 = generate_logs_batch(32, 100);

        let mut exec_pipeline = ExecutablePipeline::try_new(batch1.clone(), pipeline_expr)
            .await
            .unwrap();
        exec_pipeline.execute().await.unwrap();
        let result1 = exec_pipeline.curr_batch.clone();

        let expected_result1_indices = [3, 7, 11, 15, 19, 23, 27, 31];
        let expected_logs_result = take_record_batch(
            batch1.get(ArrowPayloadType::Logs).unwrap(),
            &UInt8Array::from_iter_values(expected_result1_indices),
        )
        .unwrap();
        assert_eq!(
            result1.get(ArrowPayloadType::Logs).unwrap(),
            &expected_logs_result
        );

        let batch2 = generate_logs_batch(64, 200);
        exec_pipeline.update_batch(batch2.clone()).unwrap();

        exec_pipeline.execute().await.unwrap();
        let result2 = exec_pipeline.curr_batch.clone();

        let expected_result2_indices =
            [3, 7, 11, 15, 19, 23, 27, 31, 35, 39, 43, 47, 51, 55, 59, 63];
        let expected_logs_result = take_record_batch(
            batch2.get(ArrowPayloadType::Logs).unwrap(),
            &UInt8Array::from_iter_values(expected_result2_indices),
        )
        .unwrap();
        assert_eq!(
            result2.get(ArrowPayloadType::Logs).unwrap(),
            &expected_logs_result
        );
    }

    #[tokio::test]
    async fn test_reuse_plans_attr_filter() {
        let query = "logs | where attributes[\"k8s.ns\"] == \"prod\"";
        let pipeline_expr = KqlParser::parse(query).unwrap();

        let batch1 = generate_logs_batch(32, 100);

        let mut exec_pipeline = ExecutablePipeline::try_new(batch1.clone(), pipeline_expr)
            .await
            .unwrap();
        exec_pipeline.execute().await.unwrap();
        let result1 = exec_pipeline.curr_batch.clone();

        let expected_result1_indices = [1, 4, 7, 10, 13, 16, 19, 22, 25, 28, 31];
        let expected_result1_attr_indices = expected_result1_indices
            .iter()
            .map(|i| i * 3)
            .flat_map(|i| [i, i + 1, i + 2]);
        let expected_logs_result = take_record_batch(
            batch1.get(ArrowPayloadType::Logs).unwrap(),
            &UInt8Array::from_iter_values(expected_result1_indices),
        )
        .unwrap();
        assert_eq!(
            result1.get(ArrowPayloadType::Logs).unwrap(),
            &expected_logs_result
        );

        let expected_log_attr_result = take_record_batch(
            batch1.get(ArrowPayloadType::LogAttrs).unwrap(),
            &UInt8Array::from_iter_values(expected_result1_attr_indices),
        )
        .unwrap();
        assert_eq!(
            result1.get(ArrowPayloadType::LogAttrs).unwrap(),
            &expected_log_attr_result
        );

        let batch2 = generate_logs_batch(64, 200);
        exec_pipeline.update_batch(batch2.clone()).unwrap();

        exec_pipeline.execute().await.unwrap();
        let result2 = exec_pipeline.curr_batch.clone();

        let expected_result2_indices = [
            0, 3, 6, 9, 12, 15, 18, 21, 24, 27, 30, 33, 36, 39, 42, 45, 48, 51, 54, 57, 60, 63,
        ];
        let expected_result2_attr_indices = expected_result2_indices
            .iter()
            .map(|i| i * 3)
            .flat_map(|i| [i, i + 1, i + 2]);
        let expected_logs_result = take_record_batch(
            batch2.get(ArrowPayloadType::Logs).unwrap(),
            &UInt8Array::from_iter_values(expected_result2_indices),
        )
        .unwrap();
        assert_eq!(
            result2.get(ArrowPayloadType::Logs).unwrap(),
            &expected_logs_result
        );
        let expected_log_attr_result = take_record_batch(
            batch2.get(ArrowPayloadType::LogAttrs).unwrap(),
            &UInt8Array::from_iter_values(expected_result2_attr_indices),
        )
        .unwrap();
        assert_eq!(
            result2.get(ArrowPayloadType::LogAttrs).unwrap(),
            &expected_log_attr_result
        );
    }

    #[tokio::test]
    async fn test_reuse_plans_schema_column_changes_simple() {
        let batch = generate_logs_batch(32, 100);
        let query = "logs | where attributes[\"k8s.ns\"] == \"prod\"";
        let pipeline = KqlParser::parse(query).unwrap();

        let mut exec_pipeline = ExecutablePipeline::try_new(batch, pipeline).await.unwrap();

        exec_pipeline.execute().await.unwrap();
        let result = exec_pipeline.curr_batch.clone();
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap().clone();
        let table_fmt = pretty_format_batches(&[logs_rb]).unwrap();
        let table_str = format!("\n{}", table_fmt);
        assert_eq!(
            table_str,
            r#"
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+
| id | resource | scope   | time_unix_nano                | observed_time_unix_nano | severity_number | severity_text | event_name |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+
| 1  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000101 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 101  |
| 4  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000104 | 1970-01-01T00:00:00     | 1               | TRACE         | event 104  |
| 7  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000107 | 1970-01-01T00:00:00     | 13              | WARN          | event 107  |
| 10 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000110 | 1970-01-01T00:00:00     | 9               | INFO          | event 110  |
| 13 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000113 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 113  |
| 16 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000116 | 1970-01-01T00:00:00     | 1               | TRACE         | event 116  |
| 19 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000119 | 1970-01-01T00:00:00     | 13              | WARN          | event 119  |
| 22 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000122 | 1970-01-01T00:00:00     | 9               | INFO          | event 122  |
| 25 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000125 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 125  |
| 28 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000128 | 1970-01-01T00:00:00     | 1               | TRACE         | event 128  |
| 31 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000131 | 1970-01-01T00:00:00     | 13              | WARN          | event 131  |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+"#
        );

        // add a column that did not exist beforehand ...
        let mut batch = generate_logs_batch(32, 200);
        let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
        let new_column = UInt32Array::from_iter_values((0..logs_rb.num_rows()).map(|_| 1u32));
        let new_field = Arc::new(Field::new(
            consts::DROPPED_ATTRIBUTES_COUNT,
            DataType::UInt32,
            true,
        ));
        let mut new_columns = logs_rb.columns().to_vec();
        new_columns.push(Arc::new(new_column));
        let mut new_fields = logs_rb.schema().fields().to_vec();
        new_fields.push(new_field);
        let new_rb = RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).unwrap();
        batch.set(ArrowPayloadType::Logs, new_rb);

        exec_pipeline.update_batch(batch).unwrap();
        exec_pipeline.execute().await.unwrap();
        let result = exec_pipeline.curr_batch.clone();
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap().clone();
        let table_fmt = pretty_format_batches(&[logs_rb]).unwrap();
        let table_str = format!("\n{}", table_fmt);
        assert_eq!(
            table_str,
            r#"
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+--------------------------+
| id | resource | scope   | time_unix_nano                | observed_time_unix_nano | severity_number | severity_text | event_name | dropped_attributes_count |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+--------------------------+
| 0  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000200 | 1970-01-01T00:00:00     | 1               | TRACE         | event 200  | 1                        |
| 3  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000203 | 1970-01-01T00:00:00     | 13              | WARN          | event 203  | 1                        |
| 6  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000206 | 1970-01-01T00:00:00     | 9               | INFO          | event 206  | 1                        |
| 9  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000209 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 209  | 1                        |
| 12 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000212 | 1970-01-01T00:00:00     | 1               | TRACE         | event 212  | 1                        |
| 15 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000215 | 1970-01-01T00:00:00     | 13              | WARN          | event 215  | 1                        |
| 18 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000218 | 1970-01-01T00:00:00     | 9               | INFO          | event 218  | 1                        |
| 21 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000221 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 221  | 1                        |
| 24 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000224 | 1970-01-01T00:00:00     | 1               | TRACE         | event 224  | 1                        |
| 27 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000227 | 1970-01-01T00:00:00     | 13              | WARN          | event 227  | 1                        |
| 30 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000230 | 1970-01-01T00:00:00     | 9               | INFO          | event 230  | 1                        |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+--------------------------+"#
        );

        // remove a column
        let mut batch = generate_logs_batch(32, 300);
        let mut new_rb = batch.get(ArrowPayloadType::Logs).unwrap().clone();
        _ = new_rb.remove_column(5);
        batch.set(ArrowPayloadType::Logs, new_rb);

        exec_pipeline.update_batch(batch).unwrap();
        exec_pipeline.execute().await.unwrap();
        let result = exec_pipeline.curr_batch.clone();
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap().clone();
        let table_fmt = pretty_format_batches(&[logs_rb]).unwrap();
        let table_str = format!("\n{}", table_fmt);
        assert_eq!(
            table_str,
            r#"
+----+----------+---------+-------------------------------+-------------------------+---------------+------------+
| id | resource | scope   | time_unix_nano                | observed_time_unix_nano | severity_text | event_name |
+----+----------+---------+-------------------------------+-------------------------+---------------+------------+
| 2  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000302 | 1970-01-01T00:00:00     | INFO          | event 302  |
| 5  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000305 | 1970-01-01T00:00:00     | DEBUG         | event 305  |
| 8  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000308 | 1970-01-01T00:00:00     | TRACE         | event 308  |
| 11 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000311 | 1970-01-01T00:00:00     | WARN          | event 311  |
| 14 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000314 | 1970-01-01T00:00:00     | INFO          | event 314  |
| 17 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000317 | 1970-01-01T00:00:00     | DEBUG         | event 317  |
| 20 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000320 | 1970-01-01T00:00:00     | TRACE         | event 320  |
| 23 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000323 | 1970-01-01T00:00:00     | WARN          | event 323  |
| 26 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000326 | 1970-01-01T00:00:00     | INFO          | event 326  |
| 29 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000329 | 1970-01-01T00:00:00     | DEBUG         | event 329  |
+----+----------+---------+-------------------------------+-------------------------+---------------+------------+"#
        );

        // change column orders
        let mut batch = generate_logs_batch(32, 400);
        let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
        let mut new_columns = logs_rb.columns().to_vec();
        let mut new_fields = logs_rb.schema().fields().to_vec();
        new_columns.swap(4, 5);
        new_fields.swap(4, 5);
        let new_rb = RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).unwrap();
        batch.set(ArrowPayloadType::Logs, new_rb);
        exec_pipeline.update_batch(batch).unwrap();
        exec_pipeline.execute().await.unwrap();
        let result = exec_pipeline.curr_batch.clone();
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap().clone();
        let table_fmt = pretty_format_batches(&[logs_rb]).unwrap();
        let table_str = format!("\n{}", table_fmt);
        // in this case, the column order shouldn't matter so we reuse the  original column order
        assert_eq!(
            table_str,
            r#"
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+
| id | resource | scope   | time_unix_nano                | observed_time_unix_nano | severity_number | severity_text | event_name |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+
| 1  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000401 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 401  |
| 4  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000404 | 1970-01-01T00:00:00     | 1               | TRACE         | event 404  |
| 7  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000407 | 1970-01-01T00:00:00     | 13              | WARN          | event 407  |
| 10 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000410 | 1970-01-01T00:00:00     | 9               | INFO          | event 410  |
| 13 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000413 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 413  |
| 16 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000416 | 1970-01-01T00:00:00     | 1               | TRACE         | event 416  |
| 19 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000419 | 1970-01-01T00:00:00     | 13              | WARN          | event 419  |
| 22 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000422 | 1970-01-01T00:00:00     | 9               | INFO          | event 422  |
| 25 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000425 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 425  |
| 28 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000428 | 1970-01-01T00:00:00     | 1               | TRACE         | event 428  |
| 31 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000431 | 1970-01-01T00:00:00     | 13              | WARN          | event 431  |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+"#
        );
    }

    #[tokio::test]
    async fn test_reuse_plans_schema_type_change() {
        let batch = generate_logs_batch(32, 100);
        let query = "logs | where attributes[\"k8s.ns\"] == \"prod\"";
        let pipeline = KqlParser::parse(query).unwrap();

        let mut exec_pipeline = ExecutablePipeline::try_new(batch, pipeline).await.unwrap();

        exec_pipeline.execute().await.unwrap();
        let result = exec_pipeline.curr_batch.clone();
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap().clone();
        let table_fmt = pretty_format_batches(&[logs_rb]).unwrap();
        let table_str = format!("\n{}", table_fmt);
        assert_eq!(
            table_str,
            r#"
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+
| id | resource | scope   | time_unix_nano                | observed_time_unix_nano | severity_number | severity_text | event_name |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+
| 1  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000101 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 101  |
| 4  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000104 | 1970-01-01T00:00:00     | 1               | TRACE         | event 104  |
| 7  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000107 | 1970-01-01T00:00:00     | 13              | WARN          | event 107  |
| 10 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000110 | 1970-01-01T00:00:00     | 9               | INFO          | event 110  |
| 13 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000113 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 113  |
| 16 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000116 | 1970-01-01T00:00:00     | 1               | TRACE         | event 116  |
| 19 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000119 | 1970-01-01T00:00:00     | 13              | WARN          | event 119  |
| 22 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000122 | 1970-01-01T00:00:00     | 9               | INFO          | event 122  |
| 25 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000125 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 125  |
| 28 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000128 | 1970-01-01T00:00:00     | 1               | TRACE         | event 128  |
| 31 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000131 | 1970-01-01T00:00:00     | 13              | WARN          | event 131  |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+"#
        );

        // sequence through a few different datatypes for the same column and ensure we handle it correctly
        let target_data_types = [
            DataType::Utf8,
            DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
            DataType::Utf8,
            DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
        ];

        for target_data_type in target_data_types {
            let mut batch = generate_logs_batch(32, 200);
            let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
            let column = logs_rb.column_by_name(consts::SEVERITY_TEXT).unwrap();
            let new_column = cast(&column, &target_data_type).unwrap();
            let field_id = logs_rb.schema().index_of(consts::SEVERITY_TEXT).unwrap();
            let mut new_columns = logs_rb.columns().to_vec();
            let mut new_fields = logs_rb.schema().fields().to_vec();
            let curr_field = new_fields[field_id].as_ref().clone();
            new_fields[field_id] =
                Arc::new(curr_field.with_data_type(new_column.data_type().clone()));
            new_columns[field_id] = new_column;
            let new_rb =
                RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).unwrap();
            batch.set(ArrowPayloadType::Logs, new_rb);

            exec_pipeline.update_batch(batch).unwrap();
            exec_pipeline.execute().await.unwrap();
            let result = exec_pipeline.curr_batch.clone();
            let logs_rb = result.get(ArrowPayloadType::Logs).unwrap().clone();
            let table_fmt = pretty_format_batches(std::slice::from_ref(&logs_rb)).unwrap();
            let table_str = format!("\n{}", table_fmt);
            assert_eq!(
                table_str,
                r#"
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+
| id | resource | scope   | time_unix_nano                | observed_time_unix_nano | severity_number | severity_text | event_name |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+
| 0  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000200 | 1970-01-01T00:00:00     | 1               | TRACE         | event 200  |
| 3  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000203 | 1970-01-01T00:00:00     | 13              | WARN          | event 203  |
| 6  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000206 | 1970-01-01T00:00:00     | 9               | INFO          | event 206  |
| 9  | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000209 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 209  |
| 12 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000212 | 1970-01-01T00:00:00     | 1               | TRACE         | event 212  |
| 15 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000215 | 1970-01-01T00:00:00     | 13              | WARN          | event 215  |
| 18 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000218 | 1970-01-01T00:00:00     | 9               | INFO          | event 218  |
| 21 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000221 | 1970-01-01T00:00:00     | 5               | DEBUG         | event 221  |
| 24 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000224 | 1970-01-01T00:00:00     | 1               | TRACE         | event 224  |
| 27 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000227 | 1970-01-01T00:00:00     | 13              | WARN          | event 227  |
| 30 | {id: 0}  | {id: 0} | 1970-01-01T00:00:00.000000230 | 1970-01-01T00:00:00     | 9               | INFO          | event 230  |
+----+----------+---------+-------------------------------+-------------------------+-----------------+---------------+------------+"#
            );

            let column = logs_rb.column_by_name(consts::SEVERITY_TEXT).unwrap();
            assert_eq!(column.data_type(), &target_data_type);
        }
    }
}
