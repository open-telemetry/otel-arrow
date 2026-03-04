// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains a [`PipelineStage`] implementation that can apply transformation pipeline
//! to attributes [`RecordBatch`].
//!
//! This allows us to treat attributes individually as members of a stream, as opposed to members
//! properties on a stream of logs/traces/metrics.

use std::sync::Arc;

use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::prelude::SessionContext;
use otap_df_pdata::OtapArrowRecords;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;

use crate::error::Result;
use crate::pipeline::PipelineStage;
use crate::pipeline::planner::AttributesIdentifier;
use crate::pipeline::state::ExecutionState;

/// Implementation of [`PipelineStage`] that performs transformations directly on a stream of
/// attribute record batches. It contains a set of inner pipeline stages that have the capability
/// to transform attributes record batches directly by calling `execute_on_attributes` method.
pub struct ApplyToAttributesPipelineStage {
    /// Identifier of which attributes record batch to apply the inner pipeline
    attributes_id: AttributesIdentifier,

    /// Pipeline stages that will be applied to each attributes record batch
    pipeline_stages: Vec<Box<dyn PipelineStage>>,
}

impl ApplyToAttributesPipelineStage {
    pub fn new(
        attributes_id: AttributesIdentifier,
        pipeline_stages: Vec<Box<dyn PipelineStage>>,
    ) -> Self {
        Self {
            attributes_id,
            pipeline_stages,
        }
    }
}

#[async_trait(?Send)]
impl PipelineStage for ApplyToAttributesPipelineStage {
    async fn execute(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        session_context: &SessionContext,
        config_options: &ConfigOptions,
        task_context: Arc<TaskContext>,
        exec_options: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        let attrs_payload_type = match &self.attributes_id {
            AttributesIdentifier::Root => match otap_batch.root_payload_type() {
                ArrowPayloadType::Logs => ArrowPayloadType::LogAttrs,
                ArrowPayloadType::Spans => ArrowPayloadType::SpanAttrs,
                _ => ArrowPayloadType::MetricAttrs,
            },
            AttributesIdentifier::NonRoot(payload_type) => *payload_type,
        };

        let Some(mut curr_batch) = otap_batch.get(attrs_payload_type).cloned() else {
            // nothing to do - just return the original batch
            return Ok(otap_batch);
        };

        for pipeline_stage in &mut self.pipeline_stages {
            curr_batch = pipeline_stage
                .execute_on_attributes(
                    curr_batch,
                    session_context,
                    config_options,
                    Arc::clone(&task_context),
                    exec_options,
                )
                .await?;
        }

        // replace record batch with pipeline result
        otap_batch.set(attrs_payload_type, curr_batch);

        Ok(otap_batch)
    }
}

#[cfg(test)]
mod test {
    use data_engine_kql_parser::Parser;
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::{
        OtapArrowRecords,
        otap::Logs,
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                resource::v1::Resource,
            },
        },
        testing::{
            equiv::assert_equivalent,
            round_trip::{otlp_to_otap, to_logs_data},
        },
    };

    use crate::pipeline::{Pipeline, planner::PipelinePlanner, test::exec_logs_pipeline};

    fn gen_logs_records_with_string_attrs() -> Vec<LogRecord> {
        vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("va")),
                    KeyValue::new("k2", AnyValue::new_string("vb")),
                    KeyValue::new("k3", AnyValue::new_string("vb")),
                    KeyValue::new("k4", AnyValue::new_string("vb")),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("va")),
                    KeyValue::new("k2", AnyValue::new_string("vb")),
                    KeyValue::new("k5", AnyValue::new_string("vc")),
                    KeyValue::new("k6", AnyValue::new_string("vg")),
                    KeyValue::new("k7", AnyValue::new_string("vg")),
                ])
                .finish(),
        ]
    }

    #[tokio::test]
    async fn test_removing_attributes_filter_on_values() {
        let logs_data = to_logs_data(gen_logs_records_with_string_attrs());
        let query = r#"
            logs | apply attributes {
                where not(matches(value, ".*b"))
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, logs_data).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("k1", AnyValue::new_string("va"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("va")),
                    KeyValue::new("k5", AnyValue::new_string("vc")),
                    KeyValue::new("k6", AnyValue::new_string("vg")),
                    KeyValue::new("k7", AnyValue::new_string("vg")),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        )
    }

    #[tokio::test]
    async fn test_removing_attributes_by_key_using_multiple_filter_stages() {
        let logs_data = to_logs_data(gen_logs_records_with_string_attrs());

        let query = r#"
            logs | apply attributes {
                where key != "k3" |
                where not(matches(key, ".*2"))
            }"#;
        let result = exec_logs_pipeline::<OplParser>(query, logs_data).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("va")),
                    KeyValue::new("k4", AnyValue::new_string("vb")),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("va")),
                    KeyValue::new("k5", AnyValue::new_string("vc")),
                    KeyValue::new("k6", AnyValue::new_string("vg")),
                    KeyValue::new("k7", AnyValue::new_string("vg")),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        )
    }

    #[tokio::test]
    async fn test_removing_resource_attributes() {
        let logs_data = LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![
                    KeyValue::new("ka", AnyValue::new_string("a")),
                    KeyValue::new("kb", AnyValue::new_string("b")),
                ])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build().finish(),
                vec![LogRecord::build().finish()],
            )],
        )]);

        let query = r#"
            logs | apply resource.attributes {
                where matches(value, ".*a")
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, logs_data).await;
        let expected = LogsData::new(vec![ResourceLogs::new(
            Resource::build()
                .attributes(vec![KeyValue::new("ka", AnyValue::new_string("a"))])
                .finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build().finish(),
                vec![LogRecord::build().finish()],
            )],
        )]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        )
    }

    #[tokio::test]
    async fn test_removing_scope_attributes() {
        let logs_data = LogsData::new(vec![ResourceLogs::new(
            Resource::build().finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .attributes(vec![
                        KeyValue::new("ka", AnyValue::new_string("a")),
                        KeyValue::new("kb", AnyValue::new_string("b")),
                    ])
                    .finish(),
                vec![LogRecord::build().finish()],
            )],
        )]);

        let query = r#"
            logs | apply instrumentation_scope.attributes {
                where matches(value, ".*a")
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, logs_data).await;
        let expected = LogsData::new(vec![ResourceLogs::new(
            Resource::build().finish(),
            vec![ScopeLogs::new(
                InstrumentationScope::build()
                    .attributes(vec![KeyValue::new("ka", AnyValue::new_string("a"))])
                    .finish(),
                vec![LogRecord::build().finish()],
            )],
        )]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        )
    }

    #[test]
    fn test_pipeline_stages_that_dont_support_attribute_exec_is_planning_error() {
        let query = r#"
            logs | apply attributes {
                project-rename attributes["x"] = attributes["y"]
            }"#;
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let planner = PipelinePlanner::new();

        let session_ctx = Pipeline::create_session_context();
        let otap_batch = OtapArrowRecords::Logs(Logs::default());
        let result = planner.plan_stages(&pipeline_expr, &session_ctx, &otap_batch);

        match result {
            Err(err) => {
                let err_msg = err.to_string();

                assert!(
                    err_msg.contains("Data expression not supported on attributes stream: Transform(RenameMapKeys(RenameMapKeysTransformExpression"),
                    "unexpected error: {}",
                    err_msg
                );
            }
            Ok(_) => {
                panic!("expected OK")
            }
        }
    }

    #[tokio::test]
    async fn test_pipeline_works_correctly_on_empty_batch() {
        let query = r#"
            logs | apply attributes {
                where value > 5
            }"#;
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        let input = OtapArrowRecords::Logs(Logs::default());
        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input)
    }

    #[tokio::test]
    async fn test_pipeline_works_correctly_on_non_empty_batch_with_no_attributes() {
        let input = to_logs_data(vec![LogRecord::build().finish()]);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let query = r#"
            logs | apply attributes {
                where value > 5
            }"#;
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        let result = pipeline.execute(input.clone()).await.unwrap();
        assert_eq!(result, input)
    }

    #[tokio::test]
    async fn test_pipeline_filter_by_bool_values() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_bool(true)),
                    KeyValue::new("k2", AnyValue::new_bool(false)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                where value == true
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("k1", AnyValue::new_bool(true))])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );

        // assert filter also works when bool literal on the left
        let query = r#"
            logs | apply attributes {
                where true == value
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input).await;
        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }

    #[tokio::test]
    async fn test_pipeline_filter_by_int_values() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(5)),
                    KeyValue::new("k2", AnyValue::new_int(14)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                where value > 10
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("k2", AnyValue::new_int(14))])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );

        // assert filter also works when literal on the left
        let query = r#"
            logs | apply attributes {
                where 10 < value
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input).await;
        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }

    #[tokio::test]
    async fn test_pipeline_filter_by_float_values() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(5.0)),
                    KeyValue::new("k2", AnyValue::new_double(14.0)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                where value > 10.0
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("k2", AnyValue::new_double(14.0))])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );

        // assert filter also works when literal on the left
        let query = r#"
            logs | apply attributes {
                where 10.0 < value
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input).await;
        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }
}
