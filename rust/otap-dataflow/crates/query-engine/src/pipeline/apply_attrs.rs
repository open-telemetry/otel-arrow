// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module contains a [`PipelineStage`] implementation that can apply transformation pipeline
//! to attributes [`RecordBatch`].
//!
//! This allows us to treat attributes individually as members of a stream, as opposed to members
//! properties on a stream of logs/traces/metrics.

use std::sync::Arc;

use arrow::array::RecordBatch;
use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::logical_expr::ColumnarValue;
use datafusion::prelude::SessionContext;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::{OtapArrowRecords, OtapPayloadHelpers};

use crate::error::Result;
use crate::pipeline::PipelineStage;
use crate::pipeline::planner::AttributesIdentifier;
use crate::pipeline::state::ExecutionState;

/// TODO API docs
pub struct ApplyToAttributesPipelineStage {
    attributes_id: AttributesIdentifier,

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

        // TODO - can we avoid unwrapping and cloning here?
        let mut curr_batch = otap_batch.get(attrs_payload_type).unwrap().clone();

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

        // replace record batch
        otap_batch.set(attrs_payload_type, curr_batch);

        Ok(otap_batch)
    }
}

#[cfg(test)]
mod test {
    use data_engine_expressions::{PipelineExpression, PipelineExpressionBuilder};
    use data_engine_kql_parser::Parser;
    use otap_df_opl::parser::OplParser;
    use otap_df_pdata::{
        proto::{
            OtlpProtoMessage,
            opentelemetry::{
                arrow::v1::ArrowPayloadType,
                common::v1::{AnyValue, KeyValue},
                logs::v1::LogRecord,
            },
        },
        testing::{
            equiv::assert_equivalent,
            round_trip::{otlp_to_otap, to_logs_data},
        },
    };

    use crate::pipeline::{
        Pipeline, PlannedPipeline,
        apply_attrs::ApplyToAttributesPipelineStage,
        planner::{AttributesIdentifier, PipelinePlanner},
        test::exec_logs_pipeline,
    };

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
}
