// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::util::pretty::print_batches;
use data_engine_columnar::optimize::datasource::UpdateDataSourceOptimizer;
use data_engine_columnar::table::OtapBatchTable;
use data_engine_kql_parser::{KqlParser, Parser};
use datafusion::catalog::MemTable;
use datafusion::common::JoinType;
use datafusion::error::DataFusionError;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::LogicalPlanBuilder;
use datafusion::physical_plan::common::collect;
use datafusion::physical_plan::displayable;
use datafusion::physical_optimizer::PhysicalOptimizerRule;
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
    let batch1 = generate_logs_batch(10, 50);

    let ctx = SessionContext::new();
    
    let logs_table = OtapBatchTable::new(
        ArrowPayloadType::Logs,
        batch1.get(ArrowPayloadType::Logs).unwrap().clone()
    );

    _ = ctx.register_table("logs", Arc::new(logs_table))?;

    let df1 = ctx.table("logs").await?
        .limit(1, Some(2))?;

    let state = ctx.state();
    let logical_plan = state.optimize(df1.logical_plan())?;
    let physical_plan = state.create_physical_plan(&logical_plan).await?;
    
    let task_ctx = Arc::new(TaskContext::from(&state));
    
    let stream1 = physical_plan.execute(0, task_ctx.clone())?;
    let results1 = collect(stream1).await?;
    println!("result 1:");
    print_batches(&results1).unwrap();

    let batch2 = generate_logs_batch(10, 80);
    let data_source_updater = UpdateDataSourceOptimizer::new(batch2);
    let session_cfg = ctx.copied_config();
    let physical_plan = data_source_updater.optimize(physical_plan, session_cfg.options().as_ref())?;

    let stream2 = physical_plan.execute(0, task_ctx.clone())?;
    let results2 = collect(stream2).await?;
    println!("result 2:");
    print_batches(&results2).unwrap();
    Ok(())    
}

/*
#[tokio::main]
async fn main() -> Result<(), DataFusionError> {
    let batch_size = 8192;
    let mut offset = 0;
    let batch1 = generate_logs_batch(batch_size, offset);
    offset += batch_size;
    let batch2 = generate_logs_batch(batch_size, offset);

    let data = vec![
        (
            batch1.get(ArrowPayloadType::Logs).unwrap(),
            batch1.get(ArrowPayloadType::LogAttrs).unwrap(),
        ),
        (
            batch2.get(ArrowPayloadType::Logs).unwrap(),
            batch2.get(ArrowPayloadType::LogAttrs).unwrap(),
        ),
    ];

    // let session_config = SessionConfig::new()
    //     .with_batch_size(8192 * 4)
    //     .with_target_partitions(1);
    // let ctx = SessionContext::new_with_config(session_config);
    let ctx = SessionContext::new();

    // register first batch
    let (logs, log_attrs) = data[0];

    let logs_table = MemTable::try_new(logs.schema(), vec![vec![logs.clone()]]).unwrap();
    let log_attrs_table = MemTable::try_new(log_attrs.schema(), vec![vec![log_attrs.clone()]]).unwrap();
    let logs_table = Arc::new(logs_table);
    let log_attrs_table = Arc::new(log_attrs_table);
    // let logs_table = Arc::new(OtapBatchTable::new(logs.schema()));
    // let log_attrs_table = Arc::new(OtapBatchTable::new(log_attrs.schema()));
    // logs_table.replace_batches(vec![logs.clone()]);
    // log_attrs_table.replace_batches(vec![log_attrs.clone()]);

    ctx.register_table("logs", logs_table.clone()).unwrap();
    ctx.register_table("logattrs", log_attrs_table.clone())
        .unwrap();

    let df = ctx
        .table("logs")
        .await
        .unwrap()
        .join(
            ctx.table("logattrs")
                .await
                .unwrap()
                .filter(col("key").eq(lit("k8s.ns")).and(col("str").eq(lit("prod"))))
                .unwrap(),
            JoinType::LeftSemi,
            &["id"],
            &["parent_id"],
            None,
        )
        .unwrap();

    let state = ctx.state();
    let task_ctx = Arc::new(TaskContext::from(&state));

    let logical_plan = state.optimize(df.logical_plan()).unwrap();
    let physical_plan = state.create_physical_plan(&logical_plan).await?;

    let df2 = ctx
        .table("logattrs")
        .await
        .unwrap()
        .join(df, JoinType::LeftSemi, &["parent_id"], &["id"], None)
        .unwrap();
    let logical_plan = state.optimize(df2.logical_plan()).unwrap();
    let mut physical_plan2 = state.create_physical_plan(&logical_plan).await.unwrap();

    let dp2 = displayable(physical_plan2.as_ref());
    println!("physucal plan 2:\n{}", dp2.indent(true));

    for i in 0..2 {
        println!("{:?}", i);
        let stream = physical_plan.execute(0, task_ctx.clone())?;
        let result = collect(stream).await?;
        black_box(result);

        physical_plan2 = physical_plan2.reset_state().unwrap();
        let stream = physical_plan2.execute(0, task_ctx.clone())?;
        let result = collect(stream).await?;
        black_box(result);

        // logs_table.replace_batches(vec![logs.clone()]);
        // log_attrs_table.replace_batches(vec![log_attrs.clone()]);
        //     let logs_table = MutableMemTable::new(logs.schema());
        //     logs_table.replace_batches(vec![logs.clone()]);
        //     let log_attrs_table = MutableMemTable::new(log_attrs.schema());
        //     log_attrs_table.replace_batches(vec![log_attrs.clone()]);
    }

    // println!("input 1");
    // arrow::util::pretty::print_batches(&[logs.clone()]).unwrap();

    // println!("results 1");

    Ok(())
}
*/

/*
#[tokio::main]
async fn main() {

    let batch = generate_logs_batch(30, 0);
    let pipeline = KqlParser::parse("logs | where attributes[\"k8s.ns\"] == \"prod\"").unwrap();
    let engine = OtapBatchEngine::new();

    let mut exec_ctx = ExecutionContext::try_new(batch).await.unwrap();
    engine.plan(&mut exec_ctx, &pipeline).await.unwrap();

    let plan = exec_ctx.root_batch_plan().unwrap();
    let session_ctx = exec_ctx.session_ctx.clone();

    let logical_plan = plan.build().unwrap();

    println!("logical plan = {}", logical_plan);
    let state = session_ctx.state();
    let logical_plan = state.optimize(&logical_plan).unwrap();
    let task_ctx = Arc::new(TaskContext::from(&state));

    let physical_plan = session_ctx.state().create_physical_plan(&logical_plan).await.unwrap();

    let displayable = displayable(physical_plan.as_ref());
    println!("\nphysicial plan:{}", displayable.indent(true));

    let stream = physical_plan.execute(0, task_ctx.clone()).unwrap();
    let batches = collect(stream).await.unwrap();

    println!("result 1");
    arrow::util::pretty::print_batches(&batches).unwrap();

    let next_batch = generate_logs_batch(30, 30);
    let next_logs = next_batch.get(ArrowPayloadType::Logs).unwrap().clone();
    let next_attrs = next_batch.get(ArrowPayloadType::LogAttrs).unwrap().clone();

    session_ctx.deregister_table("logs").unwrap();
    session_ctx.deregister_table("logattrs").unwrap();
    let logs_table = MemTable::try_new(next_logs.schema(), vec![vec![next_logs]]).unwrap();
    session_ctx.register_table("logs", Arc::new(logs_table)).unwrap();

    let log_attrs_table = MemTable::try_new(next_attrs.schema(), vec![vec![next_attrs]]).unwrap();
    session_ctx.register_table("logattrs", Arc::new(log_attrs_table)).unwrap();

    let test = session_ctx.sql("select * from logs limit 5").await.unwrap().collect().await.unwrap();
    arrow::util::pretty::print_batches(&test);

    let stream = physical_plan.execute(0, task_ctx.clone()).unwrap();
    let batches = collect(stream).await.unwrap();
    println!("result from physical exec 2");
    arrow::util::pretty::print_batches(&batches).unwrap();
}
*/

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
            let event_name = format!("{} happen", severity_text.to_lowercase());

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
