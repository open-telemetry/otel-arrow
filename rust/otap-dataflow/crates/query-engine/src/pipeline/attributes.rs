// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementations of [`PipelineStage`] for processing attributes

use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::otap::transform::{AttributesTransform, transform_attributes};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::pipeline::PipelineStage;
use crate::pipeline::planner::AttributesIdentifier;

pub struct AttributeTransformPipelineStage {
    attrs_id: AttributesIdentifier,
    transform: AttributesTransform,
}

impl AttributeTransformPipelineStage {
    pub fn new(attrs_id: AttributesIdentifier, transform: AttributesTransform) -> Self {
        Self {
            attrs_id,
            transform,
        }
    }
}

#[async_trait]
impl PipelineStage for AttributeTransformPipelineStage {
    async fn execute(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
    ) -> Result<OtapArrowRecords> {
        let attrs_payload_type = match self.attrs_id {
            AttributesIdentifier::Root => match otap_batch {
                OtapArrowRecords::Logs(_) => ArrowPayloadType::LogAttrs,
                OtapArrowRecords::Traces(_) => ArrowPayloadType::SpanAttrs,
                _ => ArrowPayloadType::MetricAttrs,
            },
            AttributesIdentifier::NonRoot(payload_type) => payload_type,
        };

        let attrs_record_batch = match otap_batch.get(attrs_payload_type) {
            Some(rb) => rb,
            None => {
                // nothing to do, there are no attributes
                return Ok(otap_batch);
            }
        };

        // transform attributes
        let attrs_transformed =
            transform_attributes(attrs_record_batch, &self.transform).map_err(|e| {
                Error::ExecutionError {
                    cause: format!("error transforming attributes {e}"),
                }
            })?;

        // replace attributes batch with transformed attributes
        otap_batch.set(attrs_payload_type, attrs_transformed);

        Ok(otap_batch)
    }
}

#[cfg(test)]
mod test {
    use otap_df_pdata::proto::opentelemetry::{
        common::v1::{AnyValue, KeyValue},
        logs::v1::LogRecord,
    };

    use crate::pipeline::test::{exec_logs_pipeline, to_logs_data};

    #[tokio::test]
    async fn test_rename_single_attributes() {
        let input = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x2", AnyValue::new_string("b"))])
                .finish(),
        ];

        let result = exec_logs_pipeline(
            "logs | project-rename attributes[\"y\"] = attributes[\"x\"]",
            to_logs_data(input),
        )
        .await;

        let expected = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x2", AnyValue::new_string("b"))])
                .finish(),
        ];

        assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }

    #[tokio::test]
    async fn test_rename_multiple_attributes() {
        let input = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("x2", AnyValue::new_string("b"))])
                .finish(),
        ];

        let result = exec_logs_pipeline(
            "logs | 
                project-rename 
                    attributes[\"y\"] = attributes[\"x\"], 
                    attributes[\"y2\"] = attributes[\"x2\"]",
            to_logs_data(input),
        )
        .await;

        let expected = vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("y", AnyValue::new_string("a"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("y2", AnyValue::new_string("b"))])
                .finish(),
        ];

        assert_eq!(result.resource_logs[0].scope_logs[0].log_records, expected)
    }

}
