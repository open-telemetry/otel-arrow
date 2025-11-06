// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use arrow::util::pretty::print_batches;
use data_engine_columnar::engine::ExecutablePipeline;
use data_engine_expressions::{
    ConditionalDataExpressionBuilder, DataExpression, LogicalExpression, PipelineExpressionBuilder,
};
use data_engine_kql_parser::{KqlParser, Parser};
use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use tokio::sync::mpsc::Receiver;

use data_engine_columnar::datagen::generate_logs_batch;
use data_engine_columnar::error::Result;
use otel_arrow_rust::otap::OtapArrowRecords;

#[tokio::main]
async fn main() -> Result<()> {
    // let pipeline_expr = {
    //     KqlParser::parse("logs | where severity_text == \"WARN\"")
    // };

    // let pipeline_expr = {
    //     KqlParser::parse("logs | where attributes[\"k8s.ns\"] == \"prod\"")
    // };

    // let pipeline_expr = {
    //     KqlParser::parse("logs | where (attributes[\"k8s.ns\"] == \"prod\" or attributes[\"k8s.ns\"] == \"staging\") and not(severity_text == \"TRACE\")")
    // };

    // let pipeline_expr = {
    //     KqlParser::parse("logs | extend severity_text = \"ERROR\" | extend severity_number = 17")
    // };
    let pipeline_expr = {
        fn logical_expr(expr: &str) -> LogicalExpression {
            let pipeline_expr = KqlParser::parse(&format!("logs | where {}", expr)).unwrap();
            let data_expr = pipeline_expr.get_expressions()[0].clone();
            if let DataExpression::Discard(discard_expr) = data_expr {
                if let LogicalExpression::Not(not_expr) = discard_expr.get_predicate().unwrap() {
                    return not_expr.get_inner_expression().clone();
                }
            }
            panic!("bad parse")
        }

        fn data_exprs(exprs: &str) -> Vec<DataExpression> {
            let pipeline_expr = KqlParser::parse(&format!("logs | {}", exprs)).unwrap();
            pipeline_expr.get_expressions().to_vec()
        }

        // if severity_text == "DEBUG"
        //    extend event_name = "debug happened"
        // else if severity_text == "TRACE"
        //   extend event_name = "trace happened"
        // else if severity_text == "INFO"
        //   extend event_name = "something happened"
        // else
        //   extend event_name = "something important happened" | extend severity_text = "ERROR" | extend severity_number = 17

        let cond_expr = ConditionalDataExpressionBuilder::from_if(
            logical_expr("severity_text == \"DEBUG\""),
            data_exprs("extend event_name = \"debug happened\""),
        )
        .with_else_if(
            logical_expr("severity_text == \"TRACE\""),
            data_exprs("extend event_name = \"trace happened\""),
        )
        .with_else_if(
            logical_expr("severity_text == \"INFO\""),
            data_exprs("extend event_name = \"something happened\""),
        )
        .with_else(data_exprs(
            "extend event_name =\"something important happened\" | extend severity_text=\"ERROR\" | extend severity_number=\"17\"",
        ))
        .build();

        PipelineExpressionBuilder::new("")
            .with_expressions(vec![DataExpression::Conditional(cond_expr)])
            .build()
    };

    let num_batches = 5;
    let batch_size = 10;
    let mut batches_stream = batches_stream(num_batches, batch_size);

    let first_batch = batches_stream.recv().await.unwrap();

    println!(">>> INPUT:");
    print_otap_batch(&first_batch);

    let mut pipeline_exec =
        ExecutablePipeline::try_new(first_batch, pipeline_expr.unwrap()).await?;
    pipeline_exec.execute().await?;

    println!(">>> RESULT:");
    print_otap_batch(&pipeline_exec.curr_batch);

    while let Some(next_batch) = batches_stream.recv().await {
        println!("------\n");
        println!(">>> INPUT:");
        print_otap_batch(&next_batch);

        pipeline_exec.update_batch(next_batch)?;
        pipeline_exec.execute().await?;

        println!(">>> RESULT:");
        print_otap_batch(&pipeline_exec.curr_batch);
    }

    Ok(())
}

fn print_otap_batch(batch: &OtapArrowRecords) {
    let logs_rb = batch.get(ArrowPayloadType::Logs).unwrap().clone();
    println!("LOGS:");
    print_batches(&[logs_rb]).unwrap();

    let logs_attrs = batch.get(ArrowPayloadType::LogAttrs).unwrap().clone();
    println!("LOG ATTRS:");
    print_batches(&[logs_attrs]).unwrap()
}

fn batches_stream(num_batches: usize, batch_size: usize) -> Receiver<OtapArrowRecords> {
    let (sender, receiver) = tokio::sync::mpsc::channel(10);

    _ = tokio::task::spawn(async move {
        for batch_num in 0..num_batches {
            let batch = generate_logs_batch(batch_size, batch_num * batch_size);
            sender.send(batch).await.unwrap();
        }
    });

    receiver
}
