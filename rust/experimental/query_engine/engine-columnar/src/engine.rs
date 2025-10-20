// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::sync::Arc;

use arrow::compute::concat_batches;
use data_engine_expressions::{DataExpression, LogicalExpression, PipelineExpression, ScalarExpression, SourceScalarExpression, StaticScalarExpression, StringValue};
use datafusion::catalog::MemTable;
use datafusion::datasource::provider_as_source;
use datafusion::logical_expr::{self, LogicalPlan, LogicalPlanBuilder, Operator};

use datafusion::prelude::SessionContext;
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::error::Result;
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


fn scan_root_batch(
    exec_ctx: &ExecutionContext
) -> Result<LogicalPlanBuilder> {
    match exec_ctx.curr_batch {
        OtapArrowRecords::Logs(_) =>  scan_batch(exec_ctx, ArrowPayloadType::Logs),
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
            None
        ).unwrap();

        Ok(logical_plan)
    } else {
        todo!("handle payload type missing");
    }
}

async fn filter_batch(
    exec_ctx: &mut ExecutionContext,
    predicate: &LogicalExpression,
) -> Result<OtapArrowRecords> {
    let mut logical_plan = scan_root_batch(&exec_ctx)?;


    let predicate_expr = match predicate {
        LogicalExpression::EqualTo(eq_expr) => {
            println!("handling EQ expr {:#?}", eq_expr);

            let left = eq_expr.get_left();
            let left_expr = match left {
                ScalarExpression::Source(source) => {
                    let accessor = source.get_value_accessor();
                    let selectors = accessor.get_selectors();
                    match &selectors[1] {
                        ScalarExpression::Static(StaticScalarExpression::String(column)) => {
                            let column_name = column.get_value();
                            match column_name {
                                "attributes" => {
                                    todo!("handle attribute access");
                                },
                                "resource" => {
                                    todo!("handle resource access");
                                },
                                "instrumentation_scope" => {
                                    todo!("handle scope access");
                                },
                                value => {
                                    logical_expr::col(value)
                                }

                            }
                        },
                        _ => {
                            todo!("handle invalid column selector");
                        }
                    }
                },
                _ => {
                    todo!("handle invalid left expr");
                }
            };

            let right = eq_expr.get_right();
            let right_expr = match right {
                ScalarExpression::Static(source) => {
                    match source {
                        StaticScalarExpression::String(str_val) => {
                            logical_expr::lit(str_val.get_value())
                        },
                         _ => {
                            todo!("handle other literal values")
                         }                         
                    }
                },
                _ => {
                    todo!("handle invalid right expr");
                }
            };

            // eq
            logical_expr::binary_expr(left_expr, Operator::Eq, right_expr)
        }
        _ => {
            todo!("handle unsupported predicate")
        }
    };

    logical_plan = logical_plan.filter(predicate_expr).unwrap();

    let logical_plan = logical_plan.build().unwrap();
    let ctx = SessionContext::new();
    let physical_plan = ctx.state().create_physical_plan(&logical_plan).await.unwrap();
    // physical_plan.execute(partition, context)

    // ctx.execute_logical_plan(logical_plan)
    let batches = ctx.execute_logical_plan(logical_plan).await.unwrap().collect().await.unwrap();
    let result = concat_batches(batches[0].schema_ref(), &batches).unwrap();

    arrow::util::pretty::print_batches(&[result]).unwrap();
    
    todo!()

}



#[cfg(test)]
mod test {
    use data_engine_kql_parser::{KqlParser, Parser, ParserMapKeySchema, ParserMapSchema, ParserOptions};
    use otap_df_otap::encoder::encode_logs_otap_batch;
    use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
    use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
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
                            ..Default::default()
                        },
                        LogRecord {
                            severity_text: "WARN".to_string(),
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

        let kql_expr = "logs | where log.severity_text == \"INFO\"";
        let pipeline_expr = KqlParser::parse_with_options(kql_expr, parser_options).unwrap();

        engine.process(&pipeline_expr, &otap_batch).await.unwrap();
    }
}
