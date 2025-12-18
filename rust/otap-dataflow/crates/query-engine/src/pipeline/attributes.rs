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

/// This pipeline stage can be used to rename and delete attributes according to the transformation
/// specified by the [`AttributesTransform`]
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
    use data_engine_kql_parser::{KqlParser, Parser};
    use otap_df_pdata::{
        OtapArrowRecords,
        otap::Logs,
        proto::opentelemetry::{
            common::v1::{AnyValue, InstrumentationScope, KeyValue},
            logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
            resource::v1::Resource,
        },
    };

    use crate::pipeline::{
        Pipeline,
        test::{exec_logs_pipeline, to_logs_data},
    };

    fn generate_logs_test_data() -> LogsData {
        LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("xr1", AnyValue::new_string("a")),
                    KeyValue::new("xr2", AnyValue::new_string("a")),
                ])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .attributes(vec![
                        KeyValue::new("xs1", AnyValue::new_string("a")),
                        KeyValue::new("xs2", AnyValue::new_string("a")),
                    ])
                    .finish(),
                vec![
                    LogRecord::build()
                        .attributes(vec![KeyValue::new("x", AnyValue::new_string("a"))])
                        .finish(),
                    LogRecord::build()
                        .attributes(vec![KeyValue::new("x2", AnyValue::new_string("b"))])
                        .finish(),
                ],
            )],
        )])
    }

    #[tokio::test]
    async fn test_rename_single_attributes() {
        let result = exec_logs_pipeline(
            "logs | project-rename attributes[\"y\"] = attributes[\"x\"]",
            generate_logs_test_data(),
        )
        .await;
        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records,
            vec![
                LogRecord::build()
                    .attributes(vec![KeyValue::new("y", AnyValue::new_string("a"))])
                    .finish(),
                LogRecord::build()
                    .attributes(vec![KeyValue::new("x2", AnyValue::new_string("b"))])
                    .finish(),
            ]
        );

        // test renaming resource attributes:
        let result = exec_logs_pipeline(
            "logs | project-rename resource.attributes[\"yr1\"] = resource.attributes[\"xr1\"]",
            generate_logs_test_data(),
        )
        .await;
        assert_eq!(
            result.resource_logs[0]
                .resource
                .as_ref()
                .unwrap()
                .attributes,
            &[
                KeyValue::new("yr1", AnyValue::new_string("a")),
                KeyValue::new("xr2", AnyValue::new_string("a")),
            ]
        );

        // test renaming scope attributes:
        let result = exec_logs_pipeline(
            "logs | project-rename instrumentation_scope.attributes[\"ys1\"] = instrumentation_scope.attributes[\"xs1\"]",
            generate_logs_test_data(),
        )
        .await;
        assert_eq!(
            result.resource_logs[0].scope_logs[0]
                .scope
                .as_ref()
                .unwrap()
                .attributes,
            &[
                KeyValue::new("ys1", AnyValue::new_string("a")),
                KeyValue::new("xs2", AnyValue::new_string("a")),
            ]
        );
    }

    #[tokio::test]
    async fn test_rename_multiple_attributes() {
        // test renaming multiple attributes from same batch
        let result = exec_logs_pipeline(
            "logs | 
                project-rename 
                    attributes[\"y\"] = attributes[\"x\"], 
                    attributes[\"y2\"] = attributes[\"x2\"]",
            generate_logs_test_data(),
        )
        .await;

        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records,
            vec![
                LogRecord::build()
                    .attributes(vec![KeyValue::new("y", AnyValue::new_string("a"))])
                    .finish(),
                LogRecord::build()
                    .attributes(vec![KeyValue::new("y2", AnyValue::new_string("b"))])
                    .finish(),
            ]
        );

        // test renaming multiple attributes from many batches
        let result = exec_logs_pipeline(
            "logs | 
                project-rename 
                    attributes[\"y\"] = attributes[\"x\"], 
                    resource.attributes[\"yr1\"] = resource.attributes[\"xr1\"],
                    instrumentation_scope.attributes[\"ys1\"] = instrumentation_scope.attributes[\"xs1\"]",
            generate_logs_test_data(),
        )
        .await;

        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records,
            vec![
                LogRecord::build()
                    .attributes(vec![KeyValue::new("y", AnyValue::new_string("a"))])
                    .finish(),
                LogRecord::build()
                    .attributes(vec![KeyValue::new("x2", AnyValue::new_string("b"))])
                    .finish(),
            ]
        );

        assert_eq!(
            result.resource_logs[0]
                .resource
                .as_ref()
                .unwrap()
                .attributes,
            &[
                KeyValue::new("yr1", AnyValue::new_string("a")),
                KeyValue::new("xr2", AnyValue::new_string("a")),
            ]
        );

        assert_eq!(
            result.resource_logs[0].scope_logs[0]
                .scope
                .as_ref()
                .unwrap()
                .attributes,
            &[
                KeyValue::new("ys1", AnyValue::new_string("a")),
                KeyValue::new("xs2", AnyValue::new_string("a")),
            ]
        );
    }

    #[tokio::test]
    async fn test_rename_when_no_attrs_batch_present() {
        let input = vec![LogRecord::build().event_name("test").finish()];
        let result = exec_logs_pipeline(
            "logs | 
                project-rename 
                    attributes[\"y\"] = attributes[\"x\"], 
                    attributes[\"y2\"] = attributes[\"x2\"]",
            to_logs_data(input.clone()),
        )
        .await;

        assert_eq!(result.resource_logs[0].scope_logs[0].log_records, input);
    }

    #[tokio::test]
    async fn test_invalid_renames_are_errors() {
        let invalid_renames = [
            "logs | project-rename attributes[\"y\"] = attributes[\"y\"]",
            "logs | project-rename attributes[\"y\"] = attributes[\"x\"], attributes[\"y\"] = attributes[\"z\"]",
        ];

        for query in invalid_renames {
            let mut pipeline = Pipeline::new(KqlParser::parse(query).unwrap().pipeline);
            let result = pipeline
                .execute(OtapArrowRecords::Logs(Logs::default()))
                .await;
            let err = result.unwrap_err();
            assert!(
                err.to_string()
                    .contains("Invalid attribute transform: Duplicate key in rename target")
            )
        }
    }

    #[tokio::test]
    async fn test_delete_attributes() {
        let result = exec_logs_pipeline(
            "logs | project-away attributes[\"x\"]",
            generate_logs_test_data(),
        )
        .await;
        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records,
            vec![
                LogRecord::build().finish(),
                LogRecord::build()
                    .attributes(vec![KeyValue::new("x2", AnyValue::new_string("b"))])
                    .finish(),
            ]
        );

        // test moving multiple attributes simultaneously from different payloads
        let result = exec_logs_pipeline(
            "logs | 
                project-away 
                    attributes[\"x\"], 
                    resource.attributes[\"xr1\"], 
                    instrumentation_scope.attributes[\"xs1\"]",
            generate_logs_test_data(),
        )
        .await;
        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records,
            vec![
                LogRecord::build().finish(),
                LogRecord::build()
                    .attributes(vec![KeyValue::new("x2", AnyValue::new_string("b"))])
                    .finish(),
            ]
        );
        assert_eq!(
            result.resource_logs[0]
                .resource
                .as_ref()
                .unwrap()
                .attributes,
            &[KeyValue::new("xr2", AnyValue::new_string("a")),]
        );
        assert_eq!(
            result.resource_logs[0].scope_logs[0]
                .scope
                .as_ref()
                .unwrap()
                .attributes,
            &[KeyValue::new("xs2", AnyValue::new_string("a")),]
        );
    }

    #[tokio::test]
    async fn test_delete_when_no_attrs_batch_present() {
        let input = LogsData::new(vec![ResourceLogs::new(
            Resource::default(),
            vec![ScopeLogs::new(
                InstrumentationScope::default(),
                vec![LogRecord::build().event_name("test").finish()],
            )],
        )]);

        let result = exec_logs_pipeline(
            "logs | 
                project-away attributes[\"y\"],
                resource.attributes[\"xr1\"], 
                instrumentation_scope.attributes[\"xs1\"]",
            input.clone(),
        )
        .await;

        assert_eq!(
            result.resource_logs[0].resource,
            input.resource_logs[0].resource,
        );
        assert_eq!(
            result.resource_logs[0].scope_logs[0].scope,
            input.resource_logs[0].scope_logs[0].scope
        );
        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records,
            input.resource_logs[0].scope_logs[0].log_records
        );
    }
}
