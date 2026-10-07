// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use crate::processors::recordset_kql_processor::otlp_bridge::{
    BridgePipeline, parse_kql_logs_query_into_pipeline,
    process_protobuf_otlp_export_logs_service_request_using_pipeline,
};
use crate::processors::recordset_kql_processor::processor::RecordsetKqlProcessor;
use crate::receivers::kafka_receiver::config::TextTransformationQueryType;
use otel_arrow_contrib_data_engine_recordset::RecordSetEngineDiagnosticLevel;
use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_engine::error::Error as EngineError;
use otel_arrow_dfe_pdata::proto::opentelemetry::{
    collector::logs::v1::ExportLogsServiceRequest,
    common::v1::{AnyValue, InstrumentationScope, KeyValue},
    logs::v1::{LogRecord, ResourceLogs, ScopeLogs},
    resource::v1::Resource,
};
use prost::Message;

#[derive(Debug)]
pub(crate) struct TextTransformation {
    pipeline: BridgePipeline,
}

impl TextTransformation {
    pub(super) fn new(
        query_type: TextTransformationQueryType,
        query: &str,
    ) -> Result<Self, ConfigError> {
        let pipeline = match query_type {
            TextTransformationQueryType::Kql => {
                let options = RecordsetKqlProcessor::apply_bridge_options_defaults(None);
                parse_kql_logs_query_into_pipeline(query, Some(options)).map_err(|errors| {
                    ConfigError::InvalidUserConfig {
                        error: format!(
                            "Failed to parse logs.transformation_query as KQL: {errors:?}"
                        ),
                    }
                })?
            }
        };
        Ok(Self { pipeline })
    }

    pub(super) fn transform(&self, data: &[u8]) -> Result<Vec<u8>, EngineError> {
        let request_data =
            std::str::from_utf8(data).map_err(|error| EngineError::PdataConversionError {
                error: format!("text_transformation payload is not valid UTF-8: {error}"),
            })?;
        let request = ExportLogsServiceRequest::new(vec![ResourceLogs::new(
            Resource::default(),
            vec![ScopeLogs::new(
                InstrumentationScope::default(),
                vec![
                    LogRecord::build()
                        .attributes(vec![KeyValue::new(
                            "request_data",
                            AnyValue::new_string(request_data),
                        )])
                        .finish(),
                ],
            )],
        )]);
        let input = request.encode_to_vec();
        let response = process_protobuf_otlp_export_logs_service_request_using_pipeline(
            &self.pipeline,
            RecordSetEngineDiagnosticLevel::Warn,
            &input,
        )
        .map_err(|error| EngineError::PdataConversionError {
            error: format!("text_transformation KQL execution failed: {error}"),
        })?;
        response
            .into_otlp_bytes()
            .map(|(output, _)| output)
            .map_err(|error| EngineError::PdataConversionError {
                error: format!("text_transformation OTLP serialization failed: {error}"),
            })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: A Kafka record contains JSON text and a KQL query projects JSON
    /// fields into canonical OTLP log fields.
    /// Guarantees: The transform emits one log with the requested severity and body.
    #[test]
    fn transforms_json_text_into_otlp_log() {
        let transform = TextTransformation::new(
            TextTransformationQueryType::Kql,
            "source \
             | extend json = parse_json(Attributes['request_data']) \
             | project severity_text = tostring(json['level']), body = tostring(json['message'])",
        )
        .expect("query should parse");

        let output = transform
            .transform(br#"{"message":"Hello world","level":"Info"}"#)
            .expect("payload should transform");
        let request =
            ExportLogsServiceRequest::decode(output.as_slice()).expect("output should be OTLP");
        let record = &request.resource_logs[0].scope_logs[0].log_records[0];

        assert_eq!(record.severity_text, "Info");
        assert_eq!(
            record.body.as_ref().and_then(|body| body.value.as_ref()),
            Some(
                &otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::any_value::Value::StringValue(
                    "Hello world".to_string(),
                ),
            )
        );
    }

    /// Scenario: A Kafka record configured for text transformation is not valid UTF-8.
    /// Guarantees: The record is rejected explicitly instead of being transformed
    /// from lossy text.
    #[test]
    fn rejects_non_utf8_input() {
        let transform = TextTransformation::new(
            TextTransformationQueryType::Kql,
            "source | project body = request_data",
        )
        .expect("query should parse");
        let error = transform.transform(&[0xff]).unwrap_err().to_string();

        assert!(error.contains("not valid UTF-8"));
    }

    /// Scenario: The configured text transformation query is syntactically invalid.
    /// Guarantees: Query compilation fails during receiver construction rather than
    /// deferring the error until a Kafka record arrives.
    #[test]
    fn rejects_invalid_query_at_startup() {
        let error = TextTransformation::new(TextTransformationQueryType::Kql, "source | where")
            .unwrap_err()
            .to_string();

        assert!(error.contains("Failed to parse logs.transformation_query as KQL"));
    }
}
