use arrow::array::{
    Array, Int16Array, RecordBatch, RunArray, StringArray, UInt8Array, UInt32Array,
};
use arrow::datatypes::{Field, Schema};
use arrow::util::pretty::print_batches;
use data_engine_columnar::datasource::exec::UpdateDataSourceOptimizer;
use data_engine_columnar::datasource::table_provider::OtapBatchTable;
use datafusion::error::DataFusionError;
use datafusion::execution::TaskContext;
use datafusion::physical_optimizer::PhysicalOptimizerRule;
use datafusion::physical_plan::common::collect;
use datafusion::prelude::*;

use datafusion::prelude::SessionContext;
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
use otel_arrow_rust::schema::consts;
use prost::Message;
use std::hint::black_box;
use std::sync::Arc;

use data_engine_columnar::engine::{ExecutionContext, OtapBatchEngine};
#[tokio::main]
async fn main() -> Result<(), DataFusionError> {
    let ctx = SessionContext::new();

    let batch1 = generate_logs_batch(20, 100);

    let logs_table = OtapBatchTable::new(
        ArrowPayloadType::Logs,
        batch1.get(ArrowPayloadType::Logs).unwrap().clone(),
    );

    _ = ctx.register_table("logs", Arc::new(logs_table))?;

    // 3 scenarios
    // - columns change order
    // - optional column missing
    // - optional column added

    let df1 = ctx
        .table("logs")
        .await?
        // .filter(col("event_name").like(lit("event %2")))?
        .filter(col("severity_text").like(lit("T%")))?
        .limit(1, Some(2))?;

    let state = ctx.state();
    let logical_plan = state.optimize(df1.logical_plan())?;
    let physical_plan = state.create_physical_plan(&logical_plan).await?;
    let task_ctx = Arc::new(TaskContext::from(&state));
    let session_config = ctx.copied_config();

    let stream = physical_plan.execute(0, task_ctx.clone())?;
    let results = collect(stream).await?;
    println!("original result:");
    print_batches(&results).unwrap();

    // // remove a column and replace it with a placeholder - manually
    // let mut batch = generate_logs_batch(20, 200);
    // let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
    // let new_column = RunArray::try_new(
    //     &Int16Array::from_iter_values([logs_rb.num_rows() as i16]),
    //     &StringArray::new_null(1),
    // )
    // .unwrap();
    // let field_id = logs_rb.schema_ref().index_of("event_name").unwrap();
    // let mut new_fields = logs_rb.schema_ref().fields().clone().to_vec();
    // new_fields[field_id] = Arc::new(
    //     logs_rb
    //         .schema_ref()
    //         .field(field_id)
    //         .clone()
    //         .with_data_type(new_column.data_type().clone()),
    // );
    // let mut new_columns = logs_rb.columns().to_vec();
    // new_columns[field_id] = Arc::new(new_column);
    // let new_logs_rb = RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).unwrap();
    // batch.set(ArrowPayloadType::Logs, new_logs_rb);

    // let ds_updater = UpdateDataSourceOptimizer::new(batch);
    // let physical_plan = ds_updater.optimize(physical_plan, session_config.options())?;

    // let stream = physical_plan.execute(0, task_ctx.clone())?;
    // let results = collect(stream).await?;
    // println!("result w/ placeholder");
    // print_batches(&results).unwrap();


    // change the column order
    let mut batch = generate_logs_batch(20, 200);
    let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();
    let mut new_columns = logs_rb.columns().to_vec();
    new_columns.swap(5, 6);
    let mut new_fields = logs_rb.schema().fields().to_vec();
    new_fields.swap(5, 6);
    let new_logs_rb = RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).unwrap();
    batch.set(ArrowPayloadType::Logs, new_logs_rb);

    let ds_updater = UpdateDataSourceOptimizer::new(batch);
    let physical_plan = ds_updater.optimize(physical_plan, session_config.options())?;

    let stream = physical_plan.execute(0, task_ctx.clone())?;
    let results = collect(stream).await?;
    println!("result w/ swapped columns");
    print_batches(&results).unwrap();

    // add an extra column
    let mut batch = generate_logs_batch(20, 300);
    let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap();

    let mut new_columns = logs_rb.columns().to_vec();
    let mut new_fields = logs_rb.schema().fields().to_vec();
    let new_column = UInt32Array::from_iter_values((0..logs_rb.num_rows()).map(|i| i as u32));
    new_fields.push(Arc::new(Field::new(
        consts::DROPPED_ATTRIBUTES_COUNT,
        new_column.data_type().clone(),
        true,
    )));
    new_columns.push(Arc::new(new_column));
    let new_logs_rb = RecordBatch::try_new(Arc::new(Schema::new(new_fields)), new_columns).unwrap();
    batch.set(ArrowPayloadType::Logs, new_logs_rb);

    let ds_updater = UpdateDataSourceOptimizer::new(batch);
    let physical_plan = ds_updater.optimize(physical_plan, session_config.options())?;

    let stream = physical_plan.execute(0, task_ctx.clone())?;
    let results = collect(stream).await?;
    println!("result w/ extra");
    print_batches(&results).unwrap();

    // remove a column
    let mut batch = generate_logs_batch(20, 400);
    let mut logs_rb = batch.get(ArrowPayloadType::Logs).unwrap().clone();
    let _ = logs_rb.remove_column(4);
    batch.set(ArrowPayloadType::Logs, logs_rb);
    let ds_updater = UpdateDataSourceOptimizer::new(batch);
    let physical_plan = ds_updater.optimize(physical_plan, session_config.options())?;

    let stream = physical_plan.execute(0, task_ctx.clone())?;
    let results = collect(stream).await?;
    println!("result w/ removed columns");
    print_batches(&results).unwrap();

    // re-add the column
    let batch = generate_logs_batch(20, 500);
    let ds_updater = UpdateDataSourceOptimizer::new(batch);
    let physical_plan = ds_updater.optimize(physical_plan, session_config.options())?;
    let stream = physical_plan.execute(0, task_ctx.clone())?;
    let results = collect(stream).await?;
    println!("result w/ original columns again");
    print_batches(&results).unwrap();

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
