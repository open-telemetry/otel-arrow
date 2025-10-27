// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

pub mod engine;
pub mod error;
pub mod table;

mod common;
mod consts;
mod datasource;
mod filter;
mod optimize;

/// helpers for testing
#[cfg(test)]
mod test {
    use arrow::array::{DictionaryArray, StringArray};
    use arrow::datatypes::UInt8Type;
    use data_engine_expressions::PipelineExpression;
    use data_engine_kql_parser::{KqlParser, Parser, ParserOptions};
    use otap_df_otap::encoder::encode_logs_otap_batch;
    use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
    use otel_arrow_rust::otap::OtapArrowRecords;
    use otel_arrow_rust::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_rust::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
    use otel_arrow_rust::proto::opentelemetry::logs::v1::{LogRecord, ResourceLogs, ScopeLogs};
    use otel_arrow_rust::schema::consts;
    use prost::Message;

    use crate::engine::OtapBatchEngine;

    pub(crate) async fn apply_to_logs(
        record: ExportLogsServiceRequest,
        pipeline_expr: PipelineExpression,
    ) -> OtapArrowRecords {
        let mut bytes = vec![];
        record.encode(&mut bytes).unwrap();
        let logs_view = RawLogsData::new(&bytes);
        let otap_batch = encode_logs_otap_batch(&logs_view).unwrap();
        let mut engine = OtapBatchEngine::new();
        engine.execute(&pipeline_expr, &otap_batch).await.unwrap()
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
}
