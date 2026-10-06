// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementations of [`PipelineStage`] for processing attributes

use async_trait::async_trait;
use datafusion::config::ConfigOptions;
use datafusion::execution::TaskContext;
use datafusion::execution::context::SessionContext;
use otel_arrow_dfe_pdata::OtapArrowRecords;
use otel_arrow_dfe_pdata::otap::transform::{AttributesTransform, apply_attribute_transform};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use std::sync::Arc;

use crate::error::{Error, Result};
use crate::pipeline::expr::ChildRecordKind;
use crate::pipeline::expr::RecordScope;
use crate::pipeline::expr::types::MetricDataPointType;
use crate::pipeline::planner::{AttributesIdentifier, RecordType};
use crate::pipeline::state::ExecutionState;
use crate::pipeline::{ParentBehavior, PipelineStage};

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
            AttributesIdentifier::Record(RecordScope::Signal) => match otap_batch {
                OtapArrowRecords::Logs(_) => ArrowPayloadType::LogAttrs,
                OtapArrowRecords::Traces(_) => ArrowPayloadType::SpanAttrs,
                _ => ArrowPayloadType::MetricAttrs,
            },
            AttributesIdentifier::Record(RecordScope::Child(ChildRecordKind::DataPoint)) => {
                return Err(Error::InvalidPipelineError {
                    cause: "Encountered non-signal scoped attributes identifier \
                        when executing pipeline for signal"
                        .into(),
                    query_location: Default::default(),
                });
            }
            AttributesIdentifier::NonRecord(payload_type) => payload_type,
        };

        _ = apply_attribute_transform(&mut otap_batch, attrs_payload_type, &self.transform, false)?;

        Ok(otap_batch)
    }

    async fn execute_on_metric_data_points(
        &mut self,
        mut otap_batch: OtapArrowRecords,
        _session_ctx: &SessionContext,
        _config_options: &ConfigOptions,
        _task_context: Arc<TaskContext>,
        _exec_state: &mut ExecutionState,
    ) -> Result<OtapArrowRecords> {
        for dp_type in MetricDataPointType::all() {
            let dp_attrs_payload = dp_type.dp_attrs_payload_type();
            if otap_batch.get(dp_attrs_payload).is_some() {
                _ = apply_attribute_transform(
                    &mut otap_batch,
                    dp_attrs_payload,
                    &self.transform,
                    false,
                )?;
            }
        }
        Ok(otap_batch)
    }

    fn supports_exec_on(&self, record_type: &RecordType) -> bool {
        match record_type {
            RecordType::Signal => true,
            RecordType::Child(ChildRecordKind::DataPoint) => {
                matches!(self.attrs_id, AttributesIdentifier::Record(_))
            }
            RecordType::Attributes => false,
        }
    }

    // Renaming or deleting non-record attributes changes scope/resource parent metadata.
    fn parent_behavior(&self) -> ParentBehavior {
        if matches!(self.attrs_id, AttributesIdentifier::NonRecord(_)) {
            ParentBehavior::RequiresReindex
        } else {
            ParentBehavior::Preserves
        }
    }
}

#[cfg(test)]
mod test {
    use otel_arrow_contrib_data_engine_kql_parser::{KqlParser, Parser};
    use otel_arrow_dfe_pdata::{
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
    use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

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

    // KQL rename tests use KQL's `project-rename` syntax (assignment-based)
    #[tokio::test]
    async fn test_rename_single_attributes_kql_parser() {
        let result = exec_logs_pipeline::<KqlParser>(
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

        let result = exec_logs_pipeline::<KqlParser>(
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

        let result = exec_logs_pipeline::<KqlParser>(
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

    // OPL rename tests use the new `rename <target> "old" as "new"` syntax
    #[tokio::test]
    async fn test_rename_single_attributes_opl_parser() {
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs | rename attributes "x" as "y""#,
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
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs | rename resource.attributes "xr1" as "yr1""#,
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
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs | rename instrumentation_scope.attributes "xs1" as "ys1""#,
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
    async fn test_rename_multiple_attributes_kql_parser() {
        // test renaming multiple attributes from same batch
        let result = exec_logs_pipeline::<KqlParser>(
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
        let result = exec_logs_pipeline::<KqlParser>(
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
    async fn test_rename_multiple_attributes_opl_parser() {
        // test renaming multiple attributes from same batch
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs | rename attributes "x" as "y", "x2" as "y2""#,
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

        // test renaming multiple attributes from many batches using chained rename calls
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs
                | rename attributes "x" as "y"
                | rename resource.attributes "xr1" as "yr1"
                | rename instrumentation_scope.attributes "xs1" as "ys1""#,
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
    async fn test_rename_when_no_attrs_batch_present_kql_parser() {
        let input = vec![LogRecord::build().event_name("test").finish()];
        let result = exec_logs_pipeline::<KqlParser>(
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
    async fn test_rename_when_no_attrs_batch_present_opl_parser() {
        let input = vec![LogRecord::build().event_name("test").finish()];
        let result = exec_logs_pipeline::<OplParser>(
            r#"logs | rename attributes "x" as "y", "x2" as "y2""#,
            to_logs_data(input.clone()),
        )
        .await;

        assert_eq!(result.resource_logs[0].scope_logs[0].log_records, input);
    }

    #[tokio::test]
    async fn test_invalid_renames_are_errors_kql_parser() {
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
    async fn test_invalid_renames_are_errors_opl_parser() {
        let invalid_renames = [
            r#"logs | rename attributes "y" as "y""#,
            r#"logs | rename attributes "x" as "y", "z" as "y""#,
        ];

        for query in invalid_renames {
            let mut pipeline = Pipeline::new(OplParser::parse(query).unwrap().pipeline);
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

    // --- Metric data-point attribute rename/delete tests ---

    mod data_point_attrs {
        use otel_arrow_contrib_data_engine_kql_parser::Parser;
        use otel_arrow_dfe_pdata::{
            proto::{
                OtlpProtoMessage,
                opentelemetry::{
                    arrow::v1::ArrowPayloadType,
                    common::v1::{AnyValue, KeyValue},
                    metrics::v1::{
                        ExponentialHistogram, ExponentialHistogramDataPoint, Gauge, Histogram,
                        HistogramDataPoint, Metric, MetricsData, NumberDataPoint, Sum, Summary,
                        SummaryDataPoint, metric::Data,
                    },
                },
            },
            testing::round_trip::{otap_to_otlp, otlp_to_otap, to_metrics_data},
        };
        use otel_arrow_dfe_query_engine_languages::opl::parser::OplParser;

        use crate::{parser::default_parser_options, pipeline::Pipeline};

        /// Build metrics with one data point per metric type, all sharing the same attributes.
        fn all_metric_types_with_attrs(attrs: Vec<KeyValue>) -> Vec<Metric> {
            vec![
                Metric::build()
                    .name("gauge")
                    .data_gauge(Gauge {
                        data_points: vec![
                            NumberDataPoint::build().attributes(attrs.clone()).finish(),
                        ],
                    })
                    .finish(),
                Metric::build()
                    .name("sum")
                    .data_sum(Sum {
                        data_points: vec![
                            NumberDataPoint::build().attributes(attrs.clone()).finish(),
                        ],
                        ..Default::default()
                    })
                    .finish(),
                Metric::build()
                    .name("histogram")
                    .data_histogram(Histogram {
                        data_points: vec![
                            HistogramDataPoint::build()
                                .attributes(attrs.clone())
                                .finish(),
                        ],
                        ..Default::default()
                    })
                    .finish(),
                Metric::build()
                    .name("exp_histogram")
                    .data_exponential_histogram(ExponentialHistogram {
                        data_points: vec![
                            ExponentialHistogramDataPoint::build()
                                .attributes(attrs.clone())
                                .finish(),
                        ],
                        ..Default::default()
                    })
                    .finish(),
                Metric::build()
                    .name("summary")
                    .data_summary(Summary {
                        data_points: vec![SummaryDataPoint::build().attributes(attrs).finish()],
                    })
                    .finish(),
            ]
        }

        /// Execute a metrics pipeline and return the result as MetricsData.
        async fn exec(query: &str, metrics: Vec<Metric>) -> MetricsData {
            let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
                .unwrap()
                .pipeline;
            let mut pipeline = Pipeline::new(pipeline_expr);
            let input = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));
            let result = pipeline.execute(input).await.unwrap();
            let OtlpProtoMessage::Metrics(md) = otap_to_otlp(&result) else {
                panic!("expected metrics")
            };
            md
        }

        /// Extract the attribute list from each metric's first data point.
        fn dp_attrs(md: &MetricsData) -> Vec<&[KeyValue]> {
            md.resource_metrics[0].scope_metrics[0]
                .metrics
                .iter()
                .map(|m| {
                    let attrs: &[KeyValue] = match m.data.as_ref().unwrap() {
                        Data::Gauge(g) => &g.data_points[0].attributes,
                        Data::Sum(s) => &s.data_points[0].attributes,
                        Data::Histogram(h) => &h.data_points[0].attributes,
                        Data::ExponentialHistogram(h) => &h.data_points[0].attributes,
                        Data::Summary(s) => &s.data_points[0].attributes,
                    };
                    attrs
                })
                .collect()
        }

        /// Scenario: Rename a data-point attribute across all metric types
        /// Guarantees: The attribute key is renamed in every data-point type
        #[tokio::test]
        async fn test_rename_dp_attrs() {
            let metrics = all_metric_types_with_attrs(vec![
                KeyValue::new("old_key", AnyValue::new_string("v")),
                KeyValue::new("keep", AnyValue::new_string("k")),
            ]);

            let result = exec(
                r#"metrics | apply data_points { rename attributes "old_key" as "new_key" }"#,
                metrics,
            )
            .await;

            let expected = vec![
                KeyValue::new("new_key", AnyValue::new_string("v")),
                KeyValue::new("keep", AnyValue::new_string("k")),
            ];
            for attrs in dp_attrs(&result) {
                assert_eq!(attrs, &expected);
            }
        }

        /// Scenario: Delete a data-point attribute across all metric types
        /// Guarantees: The specified attribute key is removed from every data-point type
        #[tokio::test]
        async fn test_delete_dp_attrs() {
            let metrics = all_metric_types_with_attrs(vec![
                KeyValue::new("remove_me", AnyValue::new_string("x")),
                KeyValue::new("keep", AnyValue::new_string("k")),
            ]);

            let result = exec(
                r#"metrics | apply data_points { remove attributes["remove_me"] }"#,
                metrics,
            )
            .await;

            let expected = vec![KeyValue::new("keep", AnyValue::new_string("k"))];
            for attrs in dp_attrs(&result) {
                assert_eq!(attrs, &expected);
            }
        }

        /// Scenario: Rename when data points have no attributes
        /// Guarantees: No-op -- the pipeline does not error
        #[tokio::test]
        async fn test_rename_dp_attrs_when_none_present() {
            let metrics = all_metric_types_with_attrs(vec![]);

            let result = exec(
                r#"metrics | apply data_points { rename attributes "x" as "y" }"#,
                metrics,
            )
            .await;

            for attrs in dp_attrs(&result) {
                assert!(attrs.is_empty());
            }
        }

        /// Scenario: Delete all data-point attributes
        /// Guarantees: The dp attrs batch is removed when every key is deleted
        #[tokio::test]
        async fn test_delete_all_dp_attrs() {
            let metrics = all_metric_types_with_attrs(vec![
                KeyValue::new("a", AnyValue::new_int(1)),
                KeyValue::new("b", AnyValue::new_int(2)),
            ]);

            let query = r#"metrics | apply data_points { remove attributes["a"] | remove attributes["b"] }"#;

            let pipeline_expr = OplParser::parse_with_options(query, default_parser_options())
                .unwrap()
                .pipeline;
            let mut pipeline = Pipeline::new(pipeline_expr);
            let input = otlp_to_otap(&OtlpProtoMessage::Metrics(to_metrics_data(metrics)));
            let result = pipeline.execute(input).await.unwrap();

            // all dp attrs payloads should be gone
            assert!(result.get(ArrowPayloadType::NumberDpAttrs).is_none());
            assert!(result.get(ArrowPayloadType::HistogramDpAttrs).is_none());
            assert!(result.get(ArrowPayloadType::ExpHistogramDpAttrs).is_none());
            assert!(result.get(ArrowPayloadType::SummaryDpAttrs).is_none());
        }

        /// Scenario: Rename multiple data-point attributes in one expression
        /// Guarantees: All specified keys are renamed in a single pipeline pass
        #[tokio::test]
        async fn test_rename_multiple_dp_attrs() {
            let metrics = all_metric_types_with_attrs(vec![
                KeyValue::new("a", AnyValue::new_int(1)),
                KeyValue::new("b", AnyValue::new_int(2)),
                KeyValue::new("c", AnyValue::new_int(3)),
            ]);

            let result = exec(
                r#"metrics | apply data_points { rename attributes "a" as "x", "b" as "y" }"#,
                metrics,
            )
            .await;

            let expected = vec![
                KeyValue::new("x", AnyValue::new_int(1)),
                KeyValue::new("y", AnyValue::new_int(2)),
                KeyValue::new("c", AnyValue::new_int(3)),
            ];
            for attrs in dp_attrs(&result) {
                assert_eq!(attrs, &expected);
            }
        }
    }
}
