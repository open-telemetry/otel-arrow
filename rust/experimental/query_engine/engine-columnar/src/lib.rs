// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub mod engine;

// TODO this might just be public for testing...
pub mod datasource;
// TODO this might also only need to be exposed for testing
pub mod datagen;

pub mod error;

mod common;
mod consts;
mod filter;

#[cfg(test)]
mod test {
    use std::sync::Arc;

    use arrow::array::{DictionaryArray, RecordBatch, StringArray, UInt32Array};
    use arrow::datatypes::{DataType, Field, Schema, UInt8Type};
    use arrow::util::pretty::print_batches;
    use data_engine_expressions::PipelineExpression;
    use data_engine_kql_parser::{KqlParser, Parser, ParserOptions};
    use datafusion::execution::TaskContext;
    use datafusion::physical_optimizer::PhysicalOptimizerRule;
    use datafusion::physical_plan::common::collect;
    use datafusion::physical_plan::{displayable, execute_stream};
    use otap_df_otap::encoder::encode_logs_otap_batch;
    use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
    use otel_arrow_rust::otap::OtapArrowRecords;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otel_arrow_rust::schema::consts;
    use prost::Message;

    use crate::datagen::generate_logs_batch;
    use crate::datasource::exec::UpdateDataSourceOptimizer;
    use crate::engine::{ExecutablePipeline, PipelinePlanBuilder};

    pub(crate) async fn apply_to_logs(
        record: ExportLogsServiceRequest,
        pipeline_expr: PipelineExpression,
    ) -> OtapArrowRecords {
        let mut bytes = vec![];
        record.encode(&mut bytes).unwrap();
        let logs_view = RawLogsData::new(&bytes);
        let otap_batch = encode_logs_otap_batch(&logs_view).unwrap();
        let mut exec_pipeline = ExecutablePipeline::try_new(otap_batch, pipeline_expr)
            .await
            .unwrap();
        exec_pipeline.execute().await.unwrap();
        return exec_pipeline.curr_batch;
    }

    // TODO this might not be the right abstraction, it only works for filtering
    pub(crate) async fn run_logs_test(
        record: ExportLogsServiceRequest,
        kql_expr: &str,
        expected_event_name: Vec<String>,
    ) {
        let parser_options = ParserOptions::new();
        let pipeline_expr = KqlParser::parse_with_options(kql_expr, parser_options).unwrap();
        let result = apply_to_logs(record, pipeline_expr).await;
        let logs_rb = result.get(ArrowPayloadType::Logs).unwrap();

        let event_name_col = logs_rb.column_by_name(consts::EVENT_NAME).unwrap();

        let tmp = event_name_col
            .as_any()
            .downcast_ref::<DictionaryArray<UInt8Type>>()
            .unwrap();
        let tmp2 = tmp.downcast_dict::<StringArray>().unwrap();
        let mut event_names = tmp2
            .into_iter()
            .map(|v| v.unwrap().to_string())
            .collect::<Vec<_>>();
        event_names.sort();

        // TODO we should probably check the attributes and other child batches are correct

        assert_eq!(expected_event_name, event_names)
    }

    pub(crate) fn logs_to_export_req(log_records: Vec<LogRecord>) -> ExportLogsServiceRequest {
        ExportLogsServiceRequest {
            resource_logs: vec![ResourceLogs {
                scope_logs: vec![ScopeLogs {
                    log_records,
                    ..Default::default()
                }],
                ..Default::default()
            }],
        }
    }

    #[tokio::test]
    async fn test_schema_modifications() {
        // TODO should rewrite this test to use only the public facing APIs on ExecPipeline
        // and should probably also move it back into the engine module

        let batch = generate_logs_batch(32, 100);
        let query = "logs | where attributes[\"k8s.ns\"] == \"prod\"";
        let pipeline = KqlParser::parse(query).unwrap();
        let mut pipeline_planner = PipelinePlanBuilder::try_new(batch).await.unwrap();
        pipeline_planner.plan(&pipeline).await.unwrap();

        let plan = pipeline_planner.logical_plan.clone();
        let ctx = pipeline_planner.session_ctx.clone();
        let logical_plan = plan.build().unwrap();
        println!("original logical plan:\n{}\n", logical_plan);

        let results_from_lp = ctx
            .execute_logical_plan(logical_plan.clone())
            .await
            .unwrap()
            .collect()
            .await
            .unwrap();
        print_batches(&results_from_lp).unwrap();

        let state = ctx.state();
        let disp_show_schema = true;
        let logical_plan = state.optimize(&logical_plan).unwrap();
        println!("optimized logical plan:\n----\n{}\n", logical_plan);
        let physical_plan = state.create_physical_plan(&logical_plan).await.unwrap();
        let dp = displayable(physical_plan.as_ref());

        println!(
            "physical plan:\n----\n{}",
            dp.set_show_schema(disp_show_schema).indent(true)
        );

        let task_context = Arc::new(TaskContext::from(&state));
        let stream = execute_stream(physical_plan.clone(), task_context.clone()).unwrap();
        let result = collect(stream).await.unwrap();
        print_batches(&result).unwrap();

        // test what happens when a column is added
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

        let data_source_updater = UpdateDataSourceOptimizer::new(batch);
        let session_cfg = ctx.copied_config();
        let config_options = session_cfg.options();
        let physical_plan = data_source_updater
            .optimize(physical_plan, config_options.as_ref())
            .unwrap();
        let dp = displayable(physical_plan.as_ref());
        println!(
            "updated physical plan:\n----\n{}",
            dp.set_show_schema(disp_show_schema).indent(true)
        );
        let stream = execute_stream(physical_plan.clone(), task_context).unwrap();
        let result = collect(stream).await.unwrap();
        print_batches(&result).unwrap();
    }
}
