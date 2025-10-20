// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::compute::concat_batches;
use data_engine_expressions::{
    DataExpression, LogicalExpression, PipelineExpression, ScalarExpression,
    SourceScalarExpression, StaticScalarExpression, StringValue, ValueAccessor,
};
use datafusion::catalog::MemTable;
use datafusion::common::JoinType;
use datafusion::datasource::provider_as_source;
use datafusion::functions::unicode::right;
use datafusion::logical_expr::Expr;
use datafusion::logical_expr::{self, LogicalPlan, LogicalPlanBuilder, Operator};

use datafusion::prelude::{SessionContext, col};

use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::schema::consts;

use crate::error::{Error, Result};
use crate::out_port::{OutPort, OutPortProvider};

struct ExecutionContext {
    curr_batch: OtapArrowRecords,
}

pub struct OtapBatchEngine {}

impl OtapBatchEngine {
    pub async fn process(
        &mut self,
        pipeline: &PipelineExpression,
        otap_batch: &OtapArrowRecords,
    ) -> Result<()> {
        let mut exec_ctx = ExecutionContext {
            curr_batch: otap_batch.clone(),
        };

        for data_expr in pipeline.get_expressions() {
            self.process_data_expr(data_expr, &mut exec_ctx).await?;
        }

        Ok(())
    }

    async fn process_data_expr(
        &mut self,
        data_expr: &DataExpression,
        exec_ctx: &mut ExecutionContext,
    ) -> Result<()> {
        match data_expr {
            DataExpression::Discard(discard) => {
                if let Some(predicate) = discard.get_predicate() {
                    match predicate {
                        // we do opposite of the discard predicate. e.g. keep what would be discarded
                        LogicalExpression::Not(not_expr) => {
                            // TODO do something with the result
                            _ = filter_batch(exec_ctx, not_expr.get_inner_expression()).await;
                        }
                        _ => todo!("handle invalid discard predciate"),
                    }
                }
            }
            _ => {
                todo!()
            }
        }
        Ok(())
    }
}

fn scan_root_batch(exec_ctx: &ExecutionContext) -> Result<LogicalPlanBuilder> {
    match exec_ctx.curr_batch {
        OtapArrowRecords::Logs(_) => scan_batch(exec_ctx, ArrowPayloadType::Logs),
        _ => {
            todo!("handle other root batches");
        }
    }
}

fn scan_batch(
    exec_ctx: &ExecutionContext,
    payload_type: ArrowPayloadType,
) -> Result<LogicalPlanBuilder> {
    if let Some(rb) = exec_ctx.curr_batch.get(payload_type) {
        let table_provider = MemTable::try_new(rb.schema(), vec![vec![rb.clone()]]).unwrap();
        let table_source = provider_as_source(Arc::new(table_provider));
        let logical_plan = LogicalPlanBuilder::scan(
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

enum ColumnAccessor {
    ColumnName(String),
    Attributes(String),
}

impl TryFrom<&ValueAccessor> for ColumnAccessor {
    type Error = Error;

    fn try_from(accessor: &ValueAccessor) -> Result<Self> {
        let selectors = accessor.get_selectors();
        match &selectors[1] {
            ScalarExpression::Static(StaticScalarExpression::String(column)) => {
                let column_name = column.get_value();
                match column_name {
                    "attributes" => match &selectors[2] {
                        ScalarExpression::Static(StaticScalarExpression::String(attr_key)) => {
                            Ok(Self::Attributes(attr_key.get_value().to_string()))
                        }
                        _ => {
                            todo!("handle invalid attribute key")
                        }
                    },
                    "resource" => {
                        todo!("handle resource access");
                    }
                    "instrumentation_scope" => {
                        todo!("handle instrumentation scope");
                    }
                    value => Ok(Self::ColumnName(value.to_string())),
                }
            }
            _ => {
                todo!("handle invalid attr expression")
            }
        }
    }
}

#[derive(Default)]
struct FilterBuilder {
    root_batch_exprs: Vec<Expr>,
    attr_batch_exprs: Vec<Expr>,

    // TODO make this not a vec, it should be a left/right tuple ...
    union_distinct: Vec<FilterBuilder>,
}

fn handle_binary_filter_expr(
    filter_builder: &mut FilterBuilder,
    operator: Operator,
    left: &ScalarExpression,
    right: &ScalarExpression,
) -> Result<()> {
    let left_col = match left {
        ScalarExpression::Source(source) => {
            let accessor = source.get_value_accessor();
            ColumnAccessor::try_from(accessor)?
        }
        _ => {
            todo!("handle invalid left expr");
        }
    };

    let right_expr = match right {
        ScalarExpression::Static(source) => match source {
            StaticScalarExpression::String(str_val) => logical_expr::lit(str_val.get_value()),
            _ => {
                todo!("handle other literal values")
            }
        },
        _ => {
            todo!("handle invalid right expr");
        }
    };

    match left_col {
        ColumnAccessor::ColumnName(col_name) => {
            let col = logical_expr::col(col_name);
            let filter_expr = logical_expr::binary_expr(col, Operator::Eq, right_expr);
            filter_builder.root_batch_exprs.push(filter_expr);
        }
        ColumnAccessor::Attributes(attr_key) => {
            let filter_expr = logical_expr::and(
                logical_expr::binary_expr(
                    logical_expr::col(consts::ATTRIBUTE_KEY),
                    operator,
                    logical_expr::lit(attr_key),
                ),
                logical_expr::binary_expr(
                    logical_expr::col(consts::ATTRIBUTE_STR),
                    operator,
                    right_expr,
                ),
            );
            filter_builder.attr_batch_exprs.push(filter_expr);
        }
    };

    Ok(())
}

fn handle_filter_predicate(
    filter_builder: &mut FilterBuilder,
    predicate: &LogicalExpression,
) -> Result<()> {
    match predicate {
        LogicalExpression::Or(or_expr) => {
            let mut left_builder = FilterBuilder::default();
            let left = or_expr.get_left();
            handle_filter_predicate(&mut left_builder, left)?;

            let mut right_builder = FilterBuilder::default();
            let right = or_expr.get_right();
            handle_filter_predicate(&mut right_builder, right)?;

            // TODO need to check we're not also joining resource_attrs, scope_attrs
            let no_joins = right_builder.attr_batch_exprs.is_empty()
                && left_builder.attr_batch_exprs.is_empty();

            if no_joins {
                let mut lefts = left_builder.root_batch_exprs[0].clone();
                for next_left in left_builder.root_batch_exprs.iter().skip(1) {
                    lefts = logical_expr::and(lefts, next_left.clone());
                }

                let mut rights = right_builder.root_batch_exprs[0].clone();
                for next_right in right_builder.root_batch_exprs.iter().skip(1) {
                    rights = logical_expr::and(rights, next_right.clone());
                }
                let filter_expr = logical_expr::or(lefts, rights);
                filter_builder.root_batch_exprs.push(filter_expr);
            } else {
                filter_builder.union_distinct.push(left_builder);
                filter_builder.union_distinct.push(right_builder);
            }
        }
        LogicalExpression::EqualTo(eq_expr) => {
            println!("handling EQ expr {:#?}", eq_expr);
            let left = eq_expr.get_left();
            let right = eq_expr.get_right();
            handle_binary_filter_expr(filter_builder, Operator::Eq, left, right)?;
        }
        _ => {
            todo!("handle unsupported predicate")
        }
    };

    Ok(())
}

fn append_filter_steps(
    exec_ctx: &mut ExecutionContext,
    mut root_logical_plan: LogicalPlanBuilder,
    filter_builder: &FilterBuilder,
) -> Result<LogicalPlanBuilder> {
    for filter_expr in &filter_builder.root_batch_exprs {
        root_logical_plan = root_logical_plan.filter(filter_expr.clone()).unwrap();
    }

    if !filter_builder.attr_batch_exprs.is_empty() {
        let mut attrs_logical_plan = scan_batch(&exec_ctx, ArrowPayloadType::LogAttrs)?;
        for filter_expr in &filter_builder.attr_batch_exprs {
            attrs_logical_plan = attrs_logical_plan.filter(filter_expr.clone()).unwrap();
        }

        root_logical_plan = root_logical_plan
            .join(
                attrs_logical_plan.build().unwrap(),
                JoinType::LeftSemi,
                (vec![consts::ID], vec![consts::PARENT_ID]),
                None,
            )
            .unwrap();
    }

    // TODO this is a goofy check
    if filter_builder.union_distinct.len() == 2 {
        let left_builder = &filter_builder.union_distinct[0];
        let left = append_filter_steps(exec_ctx, root_logical_plan.clone(), left_builder)?;

        let right_builder = &filter_builder.union_distinct[1];
        let right = append_filter_steps(exec_ctx, root_logical_plan.clone(), right_builder)?;

        root_logical_plan = left.union_distinct(right.build().unwrap()).unwrap();
    }

    return Ok(root_logical_plan);
}

async fn filter_batch(
    exec_ctx: &mut ExecutionContext,
    predicate: &LogicalExpression,
) -> Result<OtapArrowRecords> {
    let mut filter_builder = FilterBuilder::default();

    // let mut root_batch_filter_exprs = vec![];
    // let mut attr_batch_filter_exprs = vec![];
    handle_filter_predicate(&mut filter_builder, predicate)?;
    let root_logical_plan =
        append_filter_steps(exec_ctx, scan_root_batch(&exec_ctx)?, &filter_builder)?;

    // let rpc = root_logical_plan.clone();
    // root_logical_plan = root_logical_plan.filter(logical_expr::binary_expr(logical_expr::col("severity_text"), Operator::Eq, logical_expr::lit("WARN"))).unwrap()
    //     // .join(
    //     //     rpc.build().unwrap(),
    //     //     JoinType::Left,
    //     //     (vec![consts::ID], vec![consts::ID]),
    //     //     None
    //     // ).unwrap();
    //     //  Doesn't work -- duplicates stuff
    //     .union_distinct(
    //         rpc.clone().join(
    //             attrs_logical_plan.build().unwrap(),
    //             JoinType::LeftSemi,
    //             (vec![consts::ID], vec![consts::PARENT_ID]),
    //             None,
    //         ).unwrap()
    //         .build()
    //         .unwrap()
    //     )
    //     .unwrap();
    // .dis;

    let logical_plan = root_logical_plan.build().unwrap();
    println!("logical plan = {}", logical_plan);
    let ctx = SessionContext::new();
    let physical_plan = ctx
        .state()
        .create_physical_plan(&logical_plan)
        .await
        .unwrap();
    // println!("physical plan = {:#?}", physical_plan);
    // physical_plan.execute(partition, context)

    // ctx.execute_logical_plan(logical_plan)
    let batches = ctx
        .execute_logical_plan(logical_plan)
        .await
        .unwrap()
        .collect()
        .await
        .unwrap();
    let result = concat_batches(batches[0].schema_ref(), &batches).unwrap();

    arrow::util::pretty::print_batches(&[result]).unwrap();

    todo!()
}

#[cfg(test)]
mod test {
    use data_engine_kql_parser::{
        KqlParser, Parser, ParserMapKeySchema, ParserMapSchema, ParserOptions,
    };
    use otap_df_otap::encoder::encode_logs_otap_batch;
    use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
    use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use prost::Message;

    use crate::out_port::MapOutPortProvider;

    use super::*;

    #[tokio::test]
    async fn smoke_test() {
        let record = ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records: vec![
                        LogRecord {
                            severity_text: "INFO".to_string(),
                            attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                            event_name: "1".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "WARN".to_string(),
                            attributes: vec![KeyValue::new("X", AnyValue::new_string("Y"))],
                            event_name: "2".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "WARN".to_string(),
                            event_name: "3".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "ERROR".to_string(),
                            event_name: "4".to_string(),
                            ..Default::default()
                        },
                        LogRecord {
                            attributes: vec![KeyValue::new(
                                "error",
                                AnyValue::new_string("error happen"),
                            )],
                            ..Default::default()
                        },
                    ],
                    ..Default::default()
                }],
                ..Default::default()
            }],
        };

        let mut bytes = vec![];
        record.encode(&mut bytes).unwrap();
        let logs_view = RawLogsData::new(&bytes);
        let otap_batch = encode_logs_otap_batch(&logs_view).unwrap();

        let mut engine = OtapBatchEngine {};

        let mut parser_options = ParserOptions::new();

        // TODO what is the point of this?
        // let mut logs_schema = ParserMapSchema::new()
        //     .with_key_definition("severity_text", ParserMapKeySchema::Integer)
        //     .with_key_definition("attributes", ParserMapKeySchema::Map(None));
        // parser_options = parser_options.with_source_map_schema(logs_schema);

        // let kql_expr = "logs | where log.severity_text == \"WARN\"";
        // let kql_expr = "logs | where log.attributes[\"X\"] == \"Y\"";
        // let kql_expr = "logs | where log.severity_text == \"INFO\" or log.severity_text == \"ERROR\"";
        let kql_expr = "logs | where log.severity_text == \"ERROR\" or log.attributes[\"error\"] == \"error happen\"";

        // TODO next:
        // - filter with an AND expression
        // - filter by attrs where type isn't a string
        // - filter by resource.name
        // - filter by resource.attributes
        // - severity_text == "WARN" or attributes["X"] == "Y"

        let pipeline_expr = KqlParser::parse_with_options(kql_expr, parser_options).unwrap();

        engine.process(&pipeline_expr, &otap_batch).await.unwrap();
    }
}
