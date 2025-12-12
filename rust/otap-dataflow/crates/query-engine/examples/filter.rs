// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Example of using the query engine to filter telemetry data

#![allow(clippy::print_stdout)]

use arrow::util::pretty::print_batches;

use data_engine_kql_parser::{KqlParser, Parser};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, InstrumentationScope, KeyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
use otap_df_pdata::proto::opentelemetry::resource::v1::Resource;
use otap_df_pdata::testing::round_trip::otlp_to_otap;
use otap_df_query_engine::error::Result;
use otap_df_query_engine::pipeline::Pipeline;

#[tokio::main]
async fn main() -> Result<()> {
    let logs = LogsData::new(vec![ResourceLogs::new(
        Resource::default(),
        vec![ScopeLogs::new(
            InstrumentationScope::default(),
            vec![
                LogRecord::build()
                    .severity_text("ERROR")
                    .attributes(vec![KeyValue::new(
                        "service.name",
                        AnyValue::new_string("my-app-1"),
                    )])
                    .finish(),
                LogRecord::build()
                    .severity_text("DEBUG")
                    .attributes(vec![KeyValue::new(
                        "service.name",
                        AnyValue::new_string("my-app-2"),
                    )])
                    .finish(),
            ],
        )],
    )]);

    // simple example filtering logs by some property:
    let query = "logs | where severity_text == \"ERROR\"";
    let pipeline_expr = KqlParser::parse(query).expect("parses").pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    let otap_batch = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Logs(logs.clone()));
    let result = pipeline.execute(otap_batch).await?;

    let otap_logs = result.get(ArrowPayloadType::Logs).expect("logs in result");
    println!("\nresult of '{query}':");
    print_batches(std::slice::from_ref(otap_logs))?;

    // simple example filtering logs by attributes
    let query = "logs | where attributes[\"service.name\"] == \"my-app-2\"";
    let pipeline_expr = KqlParser::parse(query).expect("parses").pipeline;
    let mut pipeline = Pipeline::new(pipeline_expr);

    let otap_batch = otlp_to_otap(&otap_df_pdata::proto::OtlpProtoMessage::Logs(logs.clone()));
    let result = pipeline.execute(otap_batch).await?;

    let otap_logs = result.get(ArrowPayloadType::Logs).expect("logs in result");
    println!("\nresult of '{query}':");
    print_batches(std::slice::from_ref(otap_logs))?;

    Ok(())
}
