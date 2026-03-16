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
        if curr_batch.num_rows() > 0 {
            otap_batch.set(attrs_payload_type, curr_batch);
        } else {
            otap_batch.remove(attrs_payload_type);
        }

        Ok(otap_batch)
    }
}

#[cfg(test)]
mod test {
    use arrow::{array::UInt8Array, datatypes::DataType};
    use data_engine_kql_parser::Parser;
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::{
        OtapArrowRecords,
        otap::Logs,
        otlp::attributes::AttributeValueType,
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                arrow::v1::ArrowPayloadType,
                common::v1::{AnyValue, InstrumentationScope, KeyValue},
                logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs},
                resource::v1::Resource,
            },
        },
        schema::consts,
        testing::{
            equiv::assert_equivalent,
            round_trip::{otap_to_otlp, otlp_to_otap, to_logs_data},
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
    async fn test_filtering_attributes_using_and_in_logical_expr() {
        let logs_data = to_logs_data(gen_logs_records_with_string_attrs());

        let query = r#"
            logs | apply attributes {
                where key == "k1" and value == "va"
            }"#;
        let result = exec_logs_pipeline::<OplParser>(query, logs_data).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("k1", AnyValue::new_string("va"))])
                .finish(),
            LogRecord::build()
                .attributes(vec![KeyValue::new("k1", AnyValue::new_string("va"))])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        )
    }

    #[tokio::test]
    async fn test_filtering_attributes_using_or_in_logical_expr() {
        let logs_data = to_logs_data(gen_logs_records_with_string_attrs());

        let query = r#"
            logs | apply attributes {
                where key == "k1" or key == "k2"
            }"#;
        let result = exec_logs_pipeline::<OplParser>(query, logs_data).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("va")),
                    KeyValue::new("k2", AnyValue::new_string("vb")),
                ])
                .finish(),
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("va")),
                    KeyValue::new("k2", AnyValue::new_string("vb")),
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

    #[test]
    fn test_invalid_apply_targets_are_planning_errors() {
        let bad_targets = ["attributes.attrs.attrs", "resource.name", "severity_text"];

        for bad_target in bad_targets {
            let query = format!(
                "logs | apply {bad_target} {{
                    where value == 2
                }}"
            );
            let pipeline_expr = OplParser::parse(&query).unwrap().pipeline;
            let planner = PipelinePlanner::new();

            let session_ctx = Pipeline::create_session_context();
            let otap_batch = OtapArrowRecords::Logs(Logs::default());
            let result = planner.plan_stages(&pipeline_expr, &session_ctx, &otap_batch);

            match result {
                Err(err) => {
                    let err_msg = err.to_string();

                    assert!(
                        err_msg.contains("Invalid source for nested apply pipeline to attributes"),
                        "unexpected error: {}",
                        err_msg
                    );
                }
                Ok(_) => {
                    panic!("expected OK")
                }
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
    async fn test_pipeline_removes_attrs_record_batch_when_all_attrs_removed() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![KeyValue::new("a", AnyValue::new_int(6))])
                .finish(),
        ]);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let query = r#"
            logs | apply attributes {
                where value < 5
            }"#;
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        let result = pipeline.execute(input.clone()).await.unwrap();
        assert!(result.get(ArrowPayloadType::LogAttrs).is_none())
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

    #[tokio::test]
    async fn test_pipeline_set_string_from_static_literal() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("a")),
                    KeyValue::new("k2", AnyValue::new_string("b")),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = "a"
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("a")),
                    KeyValue::new("k2", AnyValue::new_string("a")),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_int_from_static_literal() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(1)),
                    KeyValue::new("k2", AnyValue::new_int(2)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = 3
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_int(3)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_float_from_static_literal() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(1.0)),
                    KeyValue::new("k2", AnyValue::new_double(2.0)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = 3.0
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(3.0)),
                    KeyValue::new("k2", AnyValue::new_double(3.0)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_bool_from_static_literal() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_bool(false)),
                    KeyValue::new("k2", AnyValue::new_bool(true)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = false
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_bool(false)),
                    KeyValue::new("k2", AnyValue::new_bool(false)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_int_from_arithmetic_with_static() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(1)),
                    KeyValue::new("k2", AnyValue::new_int(2)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = value + 1
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(2)),
                    KeyValue::new("k2", AnyValue::new_int(3)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_float_from_arithmetic_with_static() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(1.0)),
                    KeyValue::new("k2", AnyValue::new_double(2.0)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = value + 1.0
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(2.0)),
                    KeyValue::new("k2", AnyValue::new_double(3.0)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_int_arithmetic_involving_no_statics() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(1)),
                    KeyValue::new("k2", AnyValue::new_int(2)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = value + value
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(2)),
                    KeyValue::new("k2", AnyValue::new_int(4)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_float_arithmetic_involving_no_statics() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(2.0)),
                    KeyValue::new("k2", AnyValue::new_double(3.0)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = value * value
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_double(4.0)),
                    KeyValue::new("k2", AnyValue::new_double(9.0)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_changes_type() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(1)),
                    KeyValue::new("k2", AnyValue::new_int(2)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                set value = "hello"
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("hello")),
                    KeyValue::new("k2", AnyValue::new_string("hello")),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_downcasts_to_dict_if_type_supports_dict() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(1)),
                    KeyValue::new("k2", AnyValue::new_int(2)),
                ])
                .finish(),
        ]);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let query = r#"
            logs | apply attributes {
                set value = value + 2
            }"#;

        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input).await.unwrap();

        // verify we have the correct typ
        let logs_attrs = result.get(ArrowPayloadType::LogAttrs).unwrap();
        let int_col = logs_attrs.column_by_name(consts::ATTRIBUTE_INT).unwrap();
        assert_eq!(
            int_col.data_type(),
            &DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Int64))
        );

        // verify the assignment results are also correct
        let result_as_otlp = otap_to_otlp(&result);
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(3)),
                    KeyValue::new("k2", AnyValue::new_int(4)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[result_as_otlp],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_missing_int_column() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    // placeholder values, will replace with nulls
                    KeyValue::new("k1", AnyValue::new_int(0)),
                    KeyValue::new("k2", AnyValue::new_int(1)),
                ])
                .finish(),
        ]);
        let mut input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let mut log_attrs = input.get(ArrowPayloadType::LogAttrs).unwrap().clone();
        let (id_col_index, _) = log_attrs
            .schema()
            .fields()
            .find(consts::ATTRIBUTE_INT)
            .unwrap();
        _ = log_attrs.remove_column(id_col_index);
        input.set(ArrowPayloadType::LogAttrs, log_attrs);

        let query = r#"
            logs | apply attributes {
                set value = value + 2
            }"#;

        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input).await.unwrap();

        // null + 2 would evaluate to null, which means the whole column is null which means
        // the column should not be in the result
        let logs_attrs = result.get(ArrowPayloadType::LogAttrs).unwrap();
        assert!(logs_attrs.column_by_name(consts::ATTRIBUTE_INT).is_none());
    }

    #[tokio::test]
    async fn test_pipeline_set_missing_float_column() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    // placeholder values, will replace with nulls
                    KeyValue::new("k1", AnyValue::new_double(0.0)),
                    KeyValue::new("k2", AnyValue::new_double(1.0)),
                ])
                .finish(),
        ]);
        let mut input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let mut log_attrs = input.get(ArrowPayloadType::LogAttrs).unwrap().clone();
        let (id_col_index, _) = log_attrs
            .schema()
            .fields()
            .find(consts::ATTRIBUTE_DOUBLE)
            .unwrap();
        _ = log_attrs.remove_column(id_col_index);
        input.set(ArrowPayloadType::LogAttrs, log_attrs);

        let query = r#"
            logs | apply attributes {
                set value = value + 2.0
            }"#;

        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input).await.unwrap();

        // null + 2.0 would evaluate to null, which means the whole column is null which means
        // the column should not be in the result
        let logs_attrs = result.get(ArrowPayloadType::LogAttrs).unwrap();
        assert!(
            logs_attrs
                .column_by_name(consts::ATTRIBUTE_DOUBLE)
                .is_none()
        );
    }

    #[tokio::test]
    async fn test_pipeline_set_missing_bool_column() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    // placeholder values, will replace with nulls
                    KeyValue::new("k1", AnyValue::new_bool(false)),
                    KeyValue::new("k2", AnyValue::new_bool(true)),
                ])
                .finish(),
        ]);
        let mut input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let mut log_attrs = input.get(ArrowPayloadType::LogAttrs).unwrap().clone();
        let (id_col_index, _) = log_attrs
            .schema()
            .fields()
            .find(consts::ATTRIBUTE_BOOL)
            .unwrap();
        _ = log_attrs.remove_column(id_col_index);
        input.set(ArrowPayloadType::LogAttrs, log_attrs);

        // kind of a useless update, but just trying to test something that will read from
        // a null column and also produce a null result
        let query = r#"
            logs | apply attributes {
                set value = value
            }"#;

        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input).await.unwrap();

        let logs_attrs = result.get(ArrowPayloadType::LogAttrs).unwrap();
        assert!(logs_attrs.column_by_name(consts::ATTRIBUTE_BOOL).is_none());
    }

    #[tokio::test]
    async fn test_pipeline_set_missing_str_column() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    // placeholder values, will replace with nulls
                    KeyValue::new("k1", AnyValue::new_string("a")),
                    KeyValue::new("k2", AnyValue::new_string("")),
                ])
                .finish(),
        ]);
        let mut input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let mut log_attrs = input.get(ArrowPayloadType::LogAttrs).unwrap().clone();
        let (id_col_index, _) = log_attrs
            .schema()
            .fields()
            .find(consts::ATTRIBUTE_STR)
            .unwrap();
        _ = log_attrs.remove_column(id_col_index);
        input.set(ArrowPayloadType::LogAttrs, log_attrs);

        // kind of a useless update, but just trying to test something that will read from
        // a null column and also produce a null result
        let query = r#"
            logs | apply attributes {
                set value = value
            }"#;

        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input).await.unwrap();

        let logs_attrs = result.get(ArrowPayloadType::LogAttrs).unwrap();
        assert!(logs_attrs.column_by_name(consts::ATTRIBUTE_STR).is_none());
    }

    #[tokio::test]
    async fn test_pipeline_set_empty_values() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue { value: None }),
                    KeyValue::new("k2", AnyValue { value: None }),
                ])
                .finish(),
        ]);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let log_attrs = input.get(ArrowPayloadType::LogAttrs).unwrap().clone();
        // check that the type of attribute is indeed empty ...
        let attrs_type = log_attrs
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(attrs_type.value(0), AttributeValueType::Empty as u8);
        assert_eq!(attrs_type.value(1), AttributeValueType::Empty as u8);

        // assert there is only the key, value and type column
        assert_eq!(log_attrs.num_columns(), 3);

        // kind of a useless update, but just trying to test something that will read from
        // a empty column and also produce a result of basically the same type, just to make
        // sure nothing breaks as we try to handle this
        let query = r#"
            logs | apply attributes {
                set value = value
            }"#;

        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let result = pipeline.execute(input).await.unwrap();

        // assert we still have the empty attributes
        let logs_attrs = result.get(ArrowPayloadType::LogAttrs).unwrap();
        let attrs_type = logs_attrs
            .column_by_name(consts::ATTRIBUTE_TYPE)
            .unwrap()
            .as_any()
            .downcast_ref::<UInt8Array>()
            .unwrap();
        assert_eq!(attrs_type.value(0), AttributeValueType::Empty as u8);
        assert_eq!(attrs_type.value(1), AttributeValueType::Empty as u8);
        assert_eq!(log_attrs.num_columns(), 3);
    }

    #[tokio::test]
    async fn test_pipeline_set_empty_attrs_batch() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("hello")),
                    KeyValue::new("k1", AnyValue::new_string("world")),
                ])
                .finish(),
        ]);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(input));

        // this should filter out all the attributes before calling the set operation
        let query = r#"
            logs | apply attributes {
                where not (key == "k1") | set value = "a"
            }"#;

        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        // just make sure we don't panic/return error and that we end up with zero attrs
        let result = pipeline.execute(input).await.unwrap();
        assert!(result.get(ArrowPayloadType::LogAttrs).is_none());
    }

    #[tokio::test]
    async fn test_pipeline_set_with_attrs_input_different_types_and_values_used_in_expr() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(1)),
                    KeyValue::new("k1", AnyValue::new_double(2.0)),
                ])
                .finish(),
        ]);
        let input = otlp_to_otap(&OtlpProtoMessage::Logs(input));

        let query = r#"
            logs | apply attributes {
                set value = value * 2
            }"#;
        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);
        let err = pipeline.execute(input).await.unwrap_err();
        assert!(
            err.to_string().contains("All input rows for attribute assignment must have the same type if value used in expression"),
            "unexpected error message {}",
            err
        )
    }

    #[tokio::test]
    async fn test_pipeline_set_with_attrs_input_types_static_expression_source() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(1)),
                    KeyValue::new("k1", AnyValue::new_double(2.0)),
                ])
                .finish(),
        ]);

        let query = r#"
            logs | apply attributes {
                set value = 5
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(5)),
                    KeyValue::new("k1", AnyValue::new_int(5)),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected.clone())],
        );
    }

    #[tokio::test]
    async fn test_pipeline_conditionally_set_keys() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("x")),
                    KeyValue::new("k2", AnyValue::new_string("y")),
                    KeyValue::new("k3", AnyValue::new_string("z")),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                if (key == "k1") {
                    set value = "a"
                } else if (key == "k2") {
                    set value = "b"
                } else {
                    set value = "c"
                }
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("a")),
                    KeyValue::new("k2", AnyValue::new_string("b")),
                    KeyValue::new("k3", AnyValue::new_string("c")),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }

    #[tokio::test]
    async fn test_pipeline_condition_keeps_unmodified_attrs_if_no_else_block() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("x")),
                    KeyValue::new("k2", AnyValue::new_string("y")),
                    KeyValue::new("k3", AnyValue::new_string("z")),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                if (key == "k1" or key == "k2") {
                    set value = "a"
                }
            }"#;

        let result = exec_logs_pipeline::<OplParser>(query, input.clone()).await;
        let expected = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_string("a")),
                    KeyValue::new("k2", AnyValue::new_string("a")),
                    KeyValue::new("k3", AnyValue::new_string("z")),
                ])
                .finish(),
        ]);

        assert_equivalent(
            &[OtlpProtoMessage::Logs(result)],
            &[OtlpProtoMessage::Logs(expected)],
        );
    }

    #[tokio::test]
    async fn test_pipeline_where_conditional_receives_empty_batch_works_correctly() {
        let input = to_logs_data(vec![
            LogRecord::build()
                .attributes(vec![
                    KeyValue::new("k1", AnyValue::new_int(1)),
                    KeyValue::new("k2", AnyValue::new_int(2)),
                ])
                .finish(),
        ]);
        let query = r#"
            logs | apply attributes {
                where value == 0 |
                if (value < 10) {
                    set value = 2
                }
            }"#;

        let pipeline_expr = OplParser::parse(query).unwrap().pipeline;
        let mut pipeline = Pipeline::new(pipeline_expr);

        let input = otlp_to_otap(&OtlpProtoMessage::Logs(input));
        let result = pipeline.execute(input.clone()).await.unwrap();

        // also just assert there are no attrs remaining
        assert!(result.get(ArrowPayloadType::LogAttrs).is_none())
    }
}
