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
use crate::pipeline::state::ExecutionState;

/// This pipeline stage can be used to rename and delete attributes according to the transformation
/// specified by the [`AttributesTransform`]
pub struct AttributeTransformPipelineStage {
    attrs_id: AttributesIdentifier,
    transform: AttributesTransform,
}

impl AttributeTransformPipelineStage {
    pub const fn new(attrs_id: AttributesIdentifier, transform: AttributesTransform) -> Self {
        Self {
            attrs_id,
            transform,
        }
    }
}

#[async_trait(?Send)]
impl PipelineStage for AttributeTransformPipelineStage {
    async fn execute(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        _session_context: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_state: &mut ExecutionState,
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

        if attrs_transformed.num_rows() == 0 {
            // all attributes deleted. remove as it's now empty
            otap_batch.remove(attrs_payload_type);
        } else {
            // replace attributes batch with transformed attributes
            otap_batch.set(attrs_payload_type, attrs_transformed);
        }

        Ok(otap_batch)
    }
}

#[cfg(test)]
mod test {
    use data_engine_kql_parser::{KqlParser, Parser};
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::{
        OtapArrowRecords,
        otap::Logs,
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                arrow::v1::ArrowPayloadType,
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                resource::v1::Resource,
            },
        },
        testing::round_trip::{otlp_to_otap, to_logs_data},
    };

    use crate::pipeline::{Pipeline, test::exec_logs_pipeline};

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

    async fn test_rename_single_attributes<P: Parser>() {
        let result = exec_logs_pipeline::<P>(
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
        let result = exec_logs_pipeline::<P>(
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
        let result = exec_logs_pipeline::<P>(
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
    async fn test_rename_single_attributes_kql_parser() {
        test_rename_single_attributes::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_rename_single_attributes_opl_parser() {
        test_rename_single_attributes::<OplParser>().await;
    }

    async fn test_rename_multiple_attributes<P: Parser>() {
        // test renaming multiple attributes from same batch
        let result = exec_logs_pipeline::<P>(
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
        let result = exec_logs_pipeline::<P>(
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
    async fn test_rename_multiple_attributes_kql_parser() {
        test_rename_multiple_attributes::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_rename_multiple_attributes_opl_parser() {
        test_rename_multiple_attributes::<OplParser>().await;
    }

    async fn test_rename_when_no_attrs_batch_present<P: Parser>() {
        let input = vec![LogRecord::build().event_name("test").finish()];
        let result = exec_logs_pipeline::<P>(
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
    async fn test_rename_when_no_attrs_batch_present_kql_parser() {
        test_rename_when_no_attrs_batch_present::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_rename_when_no_attrs_batch_present_opl_parser() {
        test_rename_when_no_attrs_batch_present::<OplParser>().await;
    }

    async fn test_invalid_renames_are_errors<P: Parser>() {
        let invalid_renames = [
            "logs | project-rename attributes[\"y\"] = attributes[\"y\"]",
            "logs | project-rename attributes[\"y\"] = attributes[\"x\"], attributes[\"y\"] = attributes[\"z\"]",
        ];

        for query in invalid_renames {
            let mut pipeline = Pipeline::new(P::parse(query).unwrap().pipeline);
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
    async fn test_invalid_renames_are_errors_kql_parser() {
        test_invalid_renames_are_errors::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_invalid_renames_are_errors_opl_parser() {
        test_invalid_renames_are_errors::<OplParser>().await;
    }

    async fn test_delete_attributes<P: Parser>() {
        let result = exec_logs_pipeline::<P>(
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
        let result = exec_logs_pipeline::<P>(
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
    async fn test_delete_attributes_kql_parser() {
        test_delete_attributes::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_delete_attributes_opl_parser() {
        test_delete_attributes::<OplParser>().await;
    }

    async fn test_delete_when_no_attrs_batch_present<P: Parser>() {
        let input = LogsData::new(vec![ResourceLogs::new(
            Resource::default(),
            vec![ScopeLogs::new(
                InstrumentationScope::default(),
                vec![LogRecord::build().event_name("test").finish()],
            )],
        )]);

        let result = exec_logs_pipeline::<P>(
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

    #[tokio::test]
    async fn test_delete_when_no_attrs_batch_present_kql_parser() {
        test_delete_when_no_attrs_batch_present::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_delete_when_no_attrs_batch_present_opl_parser() {
        test_delete_when_no_attrs_batch_present::<OplParser>().await;
    }

    async fn test_delete_all_attributes<P: Parser>() {
        let input = generate_logs_test_data();
        let otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let query = "logs |
            project-away attributes[\"x\"], attributes[\"x2\"]
        ";
        let parser_result = P::parse(query).unwrap();
        let mut pipeline = Pipeline::new(parser_result.pipeline);
        let result = pipeline.execute(otap_batch).await.unwrap();

        assert!(
            result.get(ArrowPayloadType::LogAttrs).is_none(),
            "expected LogAttrs RecordBatch removed"
        )
    }

    #[tokio::test]
    async fn test_delete_all_attributes_kql_parser() {
        test_delete_all_attributes::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_delete_all_attributes_opl_parser() {
        test_delete_all_attributes::<OplParser>().await;
    }

    async fn test_insert_attributes<P: Parser>() {
        let result = exec_logs_pipeline::<P>(
            "logs | extend attributes[\"new_attr\"] = \"new_value\"",
            generate_logs_test_data(),
        )
        .await;

        assert_eq!(
            result.resource_logs[0].scope_logs[0].log_records,
            vec![
                LogRecord::build()
                    .attributes(vec![
                        KeyValue::new("x", AnyValue::new_string("a")),
                        KeyValue::new("new_attr", AnyValue::new_string("new_value"))
                    ])
                    .finish(),
                LogRecord::build()
                    .attributes(vec![
                        KeyValue::new("x2", AnyValue::new_string("b")),
                        KeyValue::new("new_attr", AnyValue::new_string("new_value"))
                    ])
                    .finish(),
            ]
        );
    }

    #[tokio::test]
    async fn test_insert_attributes_kql_parser() {
        test_insert_attributes::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_insert_attributes_opl_parser() {
        test_insert_attributes::<OplParser>().await;
    }

    async fn test_insert_attributes_types<P: Parser>() {
        let result = exec_logs_pipeline::<P>(
            "logs |
                extend
                    attributes[\"int_attr\"] = 1,
                    attributes[\"float_attr\"] = 1.0,
                    attributes[\"bool_attr\"] = true",
            generate_logs_test_data(),
        )
        .await;

        let attrs = &result.resource_logs[0].scope_logs[0].log_records[0].attributes;
        assert!(attrs.contains(&KeyValue::new("int_attr", AnyValue::new_int(1))));
        assert!(attrs.contains(&KeyValue::new("float_attr", AnyValue::new_double(1.0))));
        assert!(attrs.contains(&KeyValue::new("bool_attr", AnyValue::new_bool(true))));
    }

    #[tokio::test]
    async fn test_insert_attributes_types_kql_parser() {
        test_insert_attributes_types::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_insert_attributes_types_opl_parser() {
        test_insert_attributes_types::<OplParser>().await;
    }

    async fn test_insert_attributes_scopes<P: Parser>() {
        let result = exec_logs_pipeline::<P>(
            "logs |
                extend
                    resource.attributes[\"res_new\"] = \"test\",
                    instrumentation_scope.attributes[\"scope_new\"] = \"test\"",
            generate_logs_test_data(),
        )
        .await;

        let res_attrs = &result.resource_logs[0]
            .resource
            .as_ref()
            .unwrap()
            .attributes;
        assert!(res_attrs.contains(&KeyValue::new("res_new", AnyValue::new_string("test"))));

        let scope_attrs = &result.resource_logs[0].scope_logs[0]
            .scope
            .as_ref()
            .unwrap()
            .attributes;
        assert!(scope_attrs.contains(&KeyValue::new("scope_new", AnyValue::new_string("test"))));
    }

    #[tokio::test]
    async fn test_insert_attributes_scopes_kql_parser() {
        test_insert_attributes_scopes::<KqlParser>().await;
    }

    #[tokio::test]
    async fn test_insert_attributes_scopes_opl_parser() {
        test_insert_attributes_scopes::<OplParser>().await;
    }
}
