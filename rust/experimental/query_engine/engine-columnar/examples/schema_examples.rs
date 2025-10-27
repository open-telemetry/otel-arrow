use arrow::array::{RecordBatch, StringArray, cast};
use arrow::datatypes::{DataType, Schema};
use arrow::util::pretty::print_batches;
use data_engine_columnar::optimize::datasource::UpdateDataSourceOptimizer;
use data_engine_columnar::table::OtapBatchTable;
use data_engine_kql_parser::{KqlParser, Parser};
use datafusion::catalog::MemTable;
use datafusion::common::JoinType;
use datafusion::error::DataFusionError;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::LogicalPlanBuilder;
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::common::collect;
use datafusion::physical_plan::displayable;
use datafusion::prelude::SessionContext;
use datafusion::prelude::{SessionConfig, col, lit};
use otap_df_otap::encoder::encode_logs_otap_batch;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otel_arrow_rust::otap::OtapArrowRecords;
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otel_arrow_rust::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otel_arrow_rust::proto::opentelemetry::logs::v1::{
    LogRecord, ResourceLogs, ScopeLogs, SeverityNumber,
};
use otel_arrow_rust::proto::opentelemetry::resource::v1::Resource;
use prost::Message;
use std::hint::black_box;
use std::sync::Arc;

use data_engine_columnar::engine::{ExecutionContext, OtapBatchEngine};

#[tokio::main]
async fn main() -> Result<(), DataFusionError> {
    let batch1 = generate_logs_batch(20, 50);

    let ctx = SessionContext::new();

    let logs_table = OtapBatchTable::new(
        ArrowPayloadType::Logs,
        batch1.get(ArrowPayloadType::Logs).unwrap().clone(),
    );

    let schema = batch1.get(ArrowPayloadType::Logs).unwrap().schema();
    let dt1 = schema.field_with_name("event_name").unwrap().data_type();
    println!("DT1 = {:?}", dt1);

    _ = ctx.register_table("logs", Arc::new(logs_table))?;

    let df1 = ctx
        .table("logs")
        .await?
        // .filter(col("event_name").like(lit("event %2")))?
        .filter(col("severity_text").like(lit("T%")))?
        .limit(1, Some(2))?;

    let state = ctx.state();
    let logical_plan = state.optimize(df1.logical_plan())?;
    let physical_plan = state.create_physical_plan(&logical_plan).await?;

    let dp = displayable(physical_plan.as_ref());
    println!("phy plan:\n{}", dp.set_show_schema(true).indent(true));

    let task_ctx = Arc::new(TaskContext::from(&state));

    let stream1 = physical_plan.execute(0, task_ctx.clone())?;
    let results1 = collect(stream1).await?;
    println!("result 1:");
    print_batches(&results1).unwrap();

    let batch2 = generate_logs_batch(512, 80);

    let schema = batch2.get(ArrowPayloadType::Logs).unwrap().schema();
    let dt2 = schema.field_with_name("event_name").unwrap().data_type();
    println!("DT2 = {:?}", dt2);

    let data_source_updater = UpdateDataSourceOptimizer::new(batch2);
    let session_cfg = ctx.copied_config();
    let physical_plan =
        data_source_updater.optimize(physical_plan, session_cfg.options().as_ref())?;

    let stream2 = physical_plan.execute(0, task_ctx.clone())?;
    let results2 = collect(stream2).await?;
    println!("result 2:");
    print_batches(&results2).unwrap();

    let mut batch3 = generate_logs_batch(512, 9000);
    let logs_rb = batch3.get(ArrowPayloadType::Logs).unwrap();
    let schema = logs_rb.schema();
    let field_id = schema.index_of("event_name").unwrap();
    let column = logs_rb.column_by_name("event_name").unwrap();
    let new_column = arrow::compute::cast(column, &DataType::Utf8);
    let mut new_columns = logs_rb.columns().to_vec();
    new_columns[field_id] = new_column.unwrap();
    let mut new_fields = schema.fields.clone().to_vec();
    new_fields[field_id] = Arc::new(
        schema
            .field(field_id)
            .clone()
            .with_data_type(DataType::Utf8),
    );
    let new_rb = RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).unwrap();
    batch3.set(ArrowPayloadType::Logs, new_rb);

    let schema = batch3.get(ArrowPayloadType::Logs).unwrap().schema();
    let dt3 = schema.field_with_name("event_name").unwrap().data_type();
    println!("DT3 = {:?}", dt3);

    let data_source_updater = UpdateDataSourceOptimizer::new(batch3);
    let physical_plan =
        data_source_updater.optimize(physical_plan, session_cfg.options().as_ref())?;

    let stream3 = physical_plan.execute(0, task_ctx.clone())?;
    let results3 = collect(stream3).await?;
    println!("result 3:");
    print_batches(&results3).unwrap();

    let dp = displayable(physical_plan.as_ref());
    println!("phy plan 3:\n{}", dp.set_show_schema(false).indent(true));

    let result_dt = results3[0]
        .schema()
        .field_with_name("event_name")
        .unwrap()
        .data_type()
        .clone();
    println!("result3 dt = {:?}", result_dt);

    let mut batch4 = generate_logs_batch(512, 9000);
    let mut logs_rb = batch4.get(ArrowPayloadType::Logs).unwrap().clone();
    let _ = logs_rb.remove_column(field_id);
    batch4.set(ArrowPayloadType::Logs, logs_rb);

    let data_source_updater = UpdateDataSourceOptimizer::new(batch4);
    let physical_plan = data_source_updater
        .optimize(physical_plan, session_cfg.options().as_ref())
        .unwrap();

    let dp = displayable(physical_plan.as_ref());
    println!("phy plan 4: {}", dp.set_show_schema(false).indent(true));

    let stream4 = physical_plan.execute(0, task_ctx.clone())?;
    let results4 = collect(stream4).await?;
    println!("result 4:");
    print_batches(&results4).unwrap();

    Ok(())
}

fn generate_logs_batch(batch_size: usize, offset: usize) -> OtapArrowRecords {
    let logs = ((0 + offset)..(batch_size + offset))
        .map(|i| {
            let severity_number = SeverityNumber::try_from(((i % 4) * 4 + 1) as i32).unwrap();
            let severity_text = severity_number
                .as_str_name()
                .split("_")
                .skip(2)
                .next()
                .unwrap();
            let event_name = format!("event {}", i);
            // let event_name = format!("{} happen", severity_text.to_lowercase());

            let attrs = vec![
                KeyValue::new("k8s.pod", AnyValue::new_string(format!("my-app-{}", i % 4))),
                KeyValue::new(
                    "k8s.ns",
                    AnyValue::new_string(format!(
                        "{}",
                        match i % 3 {
                            0 => "dev",
                            1 => "staging",
                            _ => "prod",
                        }
                    )),
                ),
                KeyValue::new(
                    "region",
                    AnyValue::new_string(if i > batch_size / 2 {
                        "us-east-1"
                    } else {
                        "us-west-1"
                    }),
                ),
            ];

            LogRecord::build(i as u64, severity_number, event_name)
                .severity_text(severity_text)
                .attributes(attrs)
                .finish()
        })
        .collect::<Vec<_>>();

    let log_req = ExportLogsServiceRequest::new(vec![
        ResourceLogs::build(Resource::default())
            .scope_logs(vec![
                ScopeLogs::build(InstrumentationScope::default())
                    .log_records(logs)
                    .finish(),
            ])
            .finish(),
    ]);

    let mut bytes = vec![];
    log_req.encode(&mut bytes).expect("can encode to vec");
    let logs_view = RawLogsData::new(&bytes);
    let otap_batch = encode_logs_otap_batch(&logs_view).expect("can convert to OTAP");

    otap_batch
}
