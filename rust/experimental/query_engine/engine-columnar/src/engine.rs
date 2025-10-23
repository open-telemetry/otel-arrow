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
use datafusion::datasource::provider_as_source;
use datafusion::functions_window::expr_fn::row_number;
use datafusion::logical_expr::select_expr::SelectExpr;
use datafusion::logical_expr::{Expr, LogicalPlanBuilder, col};
use datafusion::prelude::SessionContext;

use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts;

use crate::common::{AttributesIdentifier, ColumnAccessor, try_static_scalar_to_literal};
use crate::consts::ROW_NUMBER_COL;
use crate::consts::{ATTRIBUTES_FIELD_NAME, RESOURCES_FIELD_NAME, SCOPE_FIELD_NAME};
use crate::error::{Error, Result};
use crate::filter::Filter;

#[derive(Default)]
pub struct OtapBatchEngine {}

impl OtapBatchEngine {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn process(
        &mut self,
        pipeline: &PipelineExpression,
        otap_batch: &OtapArrowRecords,
    ) -> Result<OtapArrowRecords> {
        // TODO - revisit the args here and figure out if we need to clone batch, or if the engine
        // can just take an owned reference
        let mut exec_ctx = ExecutionContext::try_new(otap_batch.clone())?;

        for data_expr in pipeline.get_expressions() {
            self.plan_data_expr(&mut exec_ctx, data_expr).await?;
        }

        // apply the plan:
        //
        // TODO at some point we may want to think more carefully about where to apply the plan.
        // currently it's always happening at the end, buts maybe it could happen in
        // process_data_expr depending on the expression?
        //
        // TODO can we avoid the clone here by deconstructing the execution context?
        // it's needed b/c below we call root_batch_payload_type() and `build()` moves the plan
        let plan = exec_ctx.curr_plan.clone().build().unwrap();
        let batches = exec_ctx
            .session_ctx
            .execute_logical_plan(plan)
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();

        let result = if batches.len() > 0 {
            // TODO not sure concat_batches is necessary here as there should only be one batch.
            // need to double check if any repartitioning happens that could cause multiple batches
            let mut result = concat_batches(batches[0].schema_ref(), &batches).unwrap();

            // remove the ROW_ID col
            if let Ok(col_idx) = result.schema_ref().index_of(ROW_NUMBER_COL) {
                _ = result.remove_column(col_idx);
            }

            result
        } else {
            // TODO can expect here ..
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

        // TODO need re-add this
        // filter_attrs_for_root(exec_ctx, ArrowPayloadType::LogAttrs).await?;
        // filter_attrs_for_root(exec_ctx, ArrowPayloadType::ResourceAttrs).await?;
        // filter_attrs_for_root(exec_ctx, ArrowPayloadType::ScopeAttrs).await?;

        Ok(exec_ctx.curr_batch)
    }

    async fn plan_data_expr(
        &mut self,
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
        &mut self,
        exec_ctx: &mut ExecutionContext,
        predicate: &LogicalExpression,
    ) -> Result<()> {
        let filter = Filter::try_from_predicate(&exec_ctx, predicate)?;
        let mut root_plan = exec_ctx.root_batch_plan()?;
        if let Some(expr) = filter.filter_expr {
            root_plan = root_plan.filter(expr)?;
        }

        if let Some(join) = filter.join {
            root_plan = join.join_to_plan(root_plan);
        }

        // update the current plan now that filters are applied
        exec_ctx.curr_plan = root_plan;

        Ok(())
    }

    async fn plan_set_field(
        &mut self,
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
            _ => {
                todo!("handle other/invalid sources")
            }
        };

        let column_name = match set.get_destination() {
            MutableValueExpression::Source(source) => {
                let column_accessor = ColumnAccessor::try_from(source.get_value_accessor())?;
                match column_accessor {
                    ColumnAccessor::ColumnName(column_name) => column_name,
                    _ => {
                        todo!("handle unsupported set target")
                    }
                }
            }
            _ => {
                todo!("handle unsupported target definition")
            }
        };

        let root_plan = exec_ctx.root_batch_plan()?;
        let new_col = source_expr.alias(&column_name);
        let mut col_exists = false;

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

        if !col_exists {
            selection.push(new_col);
        }

        let select_exprs = selection
            .into_iter()
            .map(|expr| SelectExpr::Expression(expr));

        exec_ctx.curr_plan = root_plan.project(select_exprs)?;

        Ok(())
    }

    async fn plan_conditional(
        &mut self,
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
        let mut next_branch_plan = root_plan
            .join(
                filtered_plan.build()?,
                JoinType::LeftAnti,
                (vec![ROW_NUMBER_COL], vec![ROW_NUMBER_COL]),
                None,
            )
            .unwrap();

        // handle all the `else if`s
        for i in 1..branches.len() {
            let (else_if_cond, else_if_data_exprs) = &branches[i];
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
}

#[derive(Clone)]
pub(crate) struct ExecutionContext {
    curr_plan: LogicalPlanBuilder,
    curr_batch: OtapArrowRecords,
    session_ctx: SessionContext,
}

impl ExecutionContext {
    fn try_new(batch: OtapArrowRecords) -> Result<Self> {
        // TODO this logic is also duplicated below (should this just be a method on OtapArrowRecords?)
        let root_batch_payload_type = match batch {
            OtapArrowRecords::Logs(_) => ArrowPayloadType::Logs,
            _ => {
                return Err(Error::NotYetSupportedError {
                    message: format!("Only logs signal type is currently supported"),
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
        let table_provider = MemTable::try_new(root_rb.schema(), vec![vec![root_rb.clone()]])?;
        let table_source = provider_as_source(Arc::new(table_provider));
        let plan = LogicalPlanBuilder::scan(
            format!("{:?}", root_batch_payload_type).to_ascii_lowercase(),
            table_source,
            None,
        )?;

        // add a row number column
        // TODO comment on why we're doing this
        let plan = plan.window(vec![row_number().alias(ROW_NUMBER_COL)])?;

        Ok(Self {
            curr_batch: batch,
            curr_plan: plan,
            session_ctx: SessionContext::new(),
        })
    }

    fn root_batch_payload_type(&self) -> Result<ArrowPayloadType> {
        match self.curr_batch {
            OtapArrowRecords::Logs(_) => Ok(ArrowPayloadType::Logs),
            _ => {
                return Err(Error::NotYetSupportedError {
                    message: format!("Only logs signal type is currently supported"),
                });
            }
        }
    }

    pub fn root_batch_plan(&self) -> Result<LogicalPlanBuilder> {
        Ok(self.curr_plan.clone())
    }

    // TODO - give this a less fear-inducing name than scan?
    pub fn scan_batch(&self, payload_type: ArrowPayloadType) -> Result<LogicalPlanBuilder> {
        if let Some(rb) = self.curr_batch.get(payload_type) {
            // TODO would there be anything gained by registering this table on the execution context
            // and just reusing it? Probably save at least:
            // - the vec allocations
            // - the arc allocation
            // - cloning the record batch (more vec/arc allocations internally)

            let table_provider = MemTable::try_new(rb.schema(), vec![vec![rb.clone()]]).unwrap();
            let table_source = provider_as_source(Arc::new(table_provider));
            let logical_plan = LogicalPlanBuilder::scan(
                // TODO could avoid allocation here?
                format!("{:?}", payload_type).to_ascii_lowercase(),
                table_source,
                None,
            )
            .unwrap();

            Ok(logical_plan)
        } else {
            todo!("handle payload type missing");
        }
    }

    pub fn column_exists(&self, accessor: &ColumnAccessor) -> Result<bool> {
        Ok(match accessor {
            ColumnAccessor::ColumnName(column_name) => {
                // TODO - eventually we might need to loosen the assumption that this is
                // a column on the root batch
                if let Some(rb) = self.curr_batch.get(self.root_batch_payload_type()?) {
                    rb.column_by_name(&column_name).is_some()
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

impl ColumnAccessor {
    fn try_from_attrs_key(
        attrs_identifier: AttributesIdentifier,
        scalar_expr: &ScalarExpression,
    ) -> Result<Self> {
        match scalar_expr {
            ScalarExpression::Static(StaticScalarExpression::String(attr_key)) => Ok(
                Self::Attributes(attrs_identifier, attr_key.get_value().to_string()),
            ),
            _ => {
                todo!("handle invalid attribute key")
            }
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
            _ => {
                todo!("handle invalid struct field name")
            }
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
            _ => {
                todo!("handle invalid attr expression")
            }
        }
    }
}

// tODO implementation of this is not correct
async fn filter_attrs_for_root(
    exec_ctx: &mut ExecutionContext,
    payload_type: ArrowPayloadType,
) -> Result<()> {
    if exec_ctx.curr_batch.get(payload_type).is_some() {
        let attrs_table_scan = exec_ctx.scan_batch(payload_type)?;
        let root_table_scan = exec_ctx.root_batch_plan()?;

        let logical_plan = attrs_table_scan
            .join(
                root_table_scan.build().unwrap(),
                JoinType::LeftSemi,
                (vec![consts::PARENT_ID], vec![consts::ID]),
                None,
            )
            .unwrap()
            .build()
            .unwrap();

        let batches = exec_ctx
            .session_ctx
            .execute_logical_plan(logical_plan)
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();
        // TODO handle batches empty
        let result = concat_batches(batches[0].schema_ref(), &batches).unwrap();

        exec_ctx.curr_batch.set(payload_type, result);
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use arrow::array::{DictionaryArray, StringArray};
    use arrow::datatypes::UInt8Type;
    use arrow::util::pretty::print_batches;
    use data_engine_expressions::{
        ConditionalDataExpressionBuilder, DataExpression, LogicalExpression,
        PipelineExpressionBuilder,
    };
    use data_engine_kql_parser::{KqlParser, Parser, ParserOptions};
    use otel_arrow_rust::proto::opentelemetry::{arrow::v1::ArrowPayloadType, logs::v1::LogRecord};
    use otel_arrow_rust::schema::consts;

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
        fn to_data_expr(self) -> DataExpression {
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
        return pipeline_exprs.to_vec();
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
            .with_expressions(vec![if_expr.to_data_expr()])
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
        print_batches(&[logs_rb.clone()]).unwrap();

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
            .with_expressions(vec![if_expr.to_data_expr()])
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
        print_batches(&[logs_rb.clone()]).unwrap();

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
            .with_expressions(vec![if_expr.to_data_expr()])
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
        print_batches(&[logs_rb.clone()]).unwrap();

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
}
