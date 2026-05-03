// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

/// Docker container configuration for validation scenarios
pub mod container;
/// validate the encode_decoding of otlp messages
pub mod encode_decode;
/// error definitions for the validation test
pub mod error;
/// temp fanout processor to use for validation test
pub mod fanout_processor;
/// metric definition to serialize json result from metric admin endpoint
pub mod metrics_types;
/// module for validating pipelines, runs and monitors pipelines
pub mod pipeline;
/// scenario builder that orchestrates full validation runs
pub mod scenario;
/// internal pipeline simulation utilities
mod simulate;
/// shared Jinja2 template rendering helper
mod template;
/// define structs to describe the traffic being created and captured for validation
pub mod traffic;
/// validation exporter to receive messages and assert their equivalence
pub mod validation_exporter;
/// invariants/checks helpers (attribute diff, filtering detection, etc.)
pub mod validation_types;

pub use container::ContainerConfig;
pub use error::ValidationError;
pub use validation_types::ValidationInstructions;

#[cfg(test)]
#[cfg(validation_tests)]
mod tests {
    use crate::ValidationInstructions;
    use crate::pipeline::Pipeline;
    use crate::scenario::Scenario;
    use crate::traffic::{Capture, Generator};
    use crate::validation_types::attributes::{AnyValue, AttributeDomain, KeyValue};

    #[test]
    fn validation_no_processor() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/no-processor.yaml")
                    .expect("failed to read in pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otlp_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("validation scenario failed");
    }

    #[test]
    fn validation_debug_processor() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/debug-processor.yaml")
                    .expect("failed to read in pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("validation scenario failed");
    }

    #[test]
    fn validation_attribute_processor_pipeline() {
        let deny = ValidationInstructions::AttributeDeny {
            domains: vec![AttributeDomain::Signal],
            keys: vec!["ios.app.state".into()],
        };
        let require = ValidationInstructions::AttributeRequireKey {
            domains: vec![AttributeDomain::Signal],
            keys: vec!["ios.app.state2".into()],
        };
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/attribute-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .static_signals()
                    .core_range(1, 1),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![deny, require])
                    .core_range(2, 2),
            )
            .run()
            .expect("attribute processor validation failed");
    }

    #[test]
    fn validation_filter_processor_pipeline() {
        let attr_check = ValidationInstructions::AttributeRequireKeyValue {
            domains: vec![AttributeDomain::Signal],
            pairs: vec![KeyValue::new(
                "ios.app.state".into(),
                AnyValue::String("active".into()),
            )],
        };

        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/filter-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: None,
                            max_drop_ratio: None,
                        },
                        attr_check,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("filter processor validation failed");
    }

    /// Validates the log sampling processor with a ratio policy (emit 1 out of
    /// 10) using 10 signals. With a small input the sampler should still drop
    /// approximately 90% of log records.
    #[test]
    fn validation_log_sampling_ratio_pipeline_1() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-ratio-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(10)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: Some(0.80),
                            max_drop_ratio: Some(0.99),
                        },
                        ValidationInstructions::AttributeNoDuplicate,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling ratio validation failed for signal_count=10");
    }

    /// Validates the log sampling processor with a ratio policy (emit 1 out of
    /// 10) using 100 signals, matching the default batch size boundary.
    #[test]
    fn validation_log_sampling_ratio_pipeline_2() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-ratio-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(100)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: Some(0.80),
                            max_drop_ratio: Some(0.99),
                        },
                        ValidationInstructions::AttributeNoDuplicate,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling ratio validation failed for signal_count=100");
    }

    /// Validates the log sampling processor with a ratio policy (emit 1 out of
    /// 10) using 500 signals spanning multiple batches.
    #[test]
    fn validation_log_sampling_ratio_pipeline_3() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-ratio-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: Some(0.80),
                            max_drop_ratio: Some(0.99),
                        },
                        ValidationInstructions::AttributeNoDuplicate,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling ratio validation failed for signal_count=500");
    }

    /// Tests the transform processor KQL `where ==` operation. Filters logs
    /// keeping only ERROR-severity records. Static logs are ~5% ERROR, ~15%
    /// WARN, ~80% INFO, so the vast majority should be dropped.
    /// Query: `logs | where severity_text == "ERROR"`
    #[test]
    fn validation_transform_kql_where_eq() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/transform-kql-where-eq-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: Some(0.80),
                            max_drop_ratio: Some(0.99),
                        },
                        ValidationInstructions::AttributeNoDuplicate,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform kql where-eq validation failed");
    }

    /// Validates the log sampling processor with a zip policy (max 50 items per
    /// 60-second window) using 10 signals. All signals are within the budget
    /// so everything should pass through unchanged.
    #[test]
    fn validation_log_sampling_zip_pipeline_1() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-zip-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(10)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling zip validation failed for signal_count=10");
    }

    /// Validates the log sampling processor with a zip policy (max 50 items per
    /// 60-second window) using 100 signals. The budget is exceeded so
    /// approximately 50% of signals should be dropped.
    #[test]
    fn validation_log_sampling_zip_pipeline_2() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-zip-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(100)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: Some(0.45),
                            max_drop_ratio: Some(0.55),
                        },
                        ValidationInstructions::AttributeNoDuplicate,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling zip validation failed for signal_count=100");
    }

    /// Validates the log sampling processor with a zip policy (max 50 items per
    /// 60-second window) using 500 signals spanning multiple batches. The budget
    /// is far exceeded so approximately 90% of signals should be dropped.
    #[test]
    fn validation_log_sampling_zip_pipeline_3() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-zip-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: Some(0.85),
                            max_drop_ratio: Some(0.95),
                        },
                        ValidationInstructions::AttributeNoDuplicate,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling zip validation failed for signal_count=500");
    }

    /// Validates that the log sampling processor (ratio policy) passes non-log
    /// signals through unchanged. Sends trace signals through a ratio sampler
    /// and asserts semantic equivalence -- the sampler should not alter traces
    /// at all.
    #[test]
    fn validation_log_sampling_ratio_passthrough_traces() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-ratio-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::traces()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling ratio passthrough traces validation failed");
    }

    /// Validates that the log sampling processor (ratio policy) passes metric
    /// signals through unchanged. The ratio sampler only operates on logs;
    /// metrics should be forwarded with no modifications.
    #[test]
    fn validation_log_sampling_ratio_passthrough_metrics() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-ratio-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::metrics()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling ratio passthrough metrics validation failed");
    }

    /// Validates the log sampling processor with a full passthrough ratio policy
    /// (emit 1 out of 1 = 100% sampling). All log signals should pass through
    /// unchanged.
    #[test]
    fn validation_log_sampling_ratio_full_passthrough() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file(
                    "./validation_pipelines/log-sampling-full-passthrough-processor.yaml",
                )
                .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("log sampling ratio full passthrough validation failed");
    }

    /// Tests the transform processor KQL `where !=` operation. Filters logs
    /// dropping INFO-severity records and keeping WARN + ERROR. Static logs
    /// are ~80% INFO so roughly 80% should be dropped.
    /// Query: `logs | where severity_text != "INFO"`
    #[test]
    fn validation_transform_kql_where_neq() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file(
                    "./validation_pipelines/transform-kql-where-neq-processor.yaml",
                )
                .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: Some(0.70),
                            max_drop_ratio: Some(0.90),
                        },
                        ValidationInstructions::AttributeNoDuplicate,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform kql where-neq validation failed");
    }

    /// Tests the transform processor OPL `set` operation on a signal attribute.
    /// Adds a new `processed_by` attribute to every log record.
    /// Query: `logs | set attributes["processed_by"] = "transform"`
    #[test]
    fn validation_transform_opl_set_attribute() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file(
                    "./validation_pipelines/transform-opl-set-attribute-processor.yaml",
                )
                .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::AttributeRequireKey {
                        domains: vec![AttributeDomain::Signal],
                        keys: vec!["processed_by".into()],
                    }])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform opl set-attribute validation failed");
    }

    /// Tests the transform processor KQL signal scoping passthrough. The query
    /// is scoped to `logs |`, so trace signals should be forwarded unchanged.
    /// Query: `logs | where severity_text == "ERROR"` (KQL, but traces sent)
    #[test]
    fn validation_transform_kql_passthrough_traces() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/transform-kql-where-eq-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::traces()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform kql passthrough traces validation failed");
    }

    /// Tests the transform processor OPL `exclude` operation. Removes the
    /// `thread.id` attribute from every log record while preserving `thread.name`.
    /// Query: `logs | exclude attributes["thread.id"]`
    #[test]
    fn validation_transform_opl_exclude_attribute() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file(
                    "./validation_pipelines/transform-opl-exclude-attribute-processor.yaml",
                )
                .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::AttributeDeny {
                            domains: vec![AttributeDomain::Signal],
                            keys: vec!["thread.id".into()],
                        },
                        ValidationInstructions::AttributeRequireKey {
                            domains: vec![AttributeDomain::Signal],
                            keys: vec!["thread.name".into()],
                        },
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform opl exclude-attribute validation failed");
    }

    /// Tests the transform processor OPL `rename` operation. Renames the
    /// `thread.id` attribute to `new_thread_id`. The old key must be absent
    /// and the new key must be present on every log record.
    /// Query: `logs | rename attributes["new_thread_id"] = attributes["thread.id"]`
    #[test]
    fn validation_transform_opl_rename_attribute() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file(
                    "./validation_pipelines/transform-opl-rename-attribute-processor.yaml",
                )
                .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::AttributeDeny {
                            domains: vec![AttributeDomain::Signal],
                            keys: vec!["thread.id".into()],
                        },
                        ValidationInstructions::AttributeRequireKey {
                            domains: vec![AttributeDomain::Signal],
                            keys: vec!["new_thread_id".into()],
                        },
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform opl rename-attribute validation failed");
    }

    /// Tests the transform processor OPL `if/else` conditional with `set`.
    /// Sets `is_error` to `"true"` for ERROR logs and `"false"` for all others.
    /// Every log record should have the `is_error` attribute regardless of
    /// severity.
    /// Query: `logs | if (severity_text == "ERROR") { set attributes["is_error"]
    ///   = "true" } else { set attributes["is_error"] = "false" }`
    #[test]
    fn validation_transform_opl_conditional_set() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file(
                    "./validation_pipelines/transform-opl-conditional-set-processor.yaml",
                )
                .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::AttributeRequireKey {
                        domains: vec![AttributeDomain::Signal],
                        keys: vec!["is_error".into()],
                    }])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform opl conditional-set validation failed");
    }

    /// Tests the transform processor with chained OPL `set` + `exclude`
    /// operations. First adds a `processed` attribute, then removes
    /// `thread.id`. Both transformations must be reflected in the output.
    /// Query: `logs | set attributes["processed"] = "yes" | exclude
    ///   attributes["thread.id"]`
    #[test]
    fn validation_transform_opl_chained_set_exclude() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file(
                    "./validation_pipelines/transform-opl-chained-set-exclude-processor.yaml",
                )
                .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::AttributeRequireKey {
                            domains: vec![AttributeDomain::Signal],
                            keys: vec!["processed".into()],
                        },
                        ValidationInstructions::AttributeDeny {
                            domains: vec![AttributeDomain::Signal],
                            keys: vec!["thread.id".into()],
                        },
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform opl chained set+exclude validation failed");
    }

    /// Tests the transform processor OPL `set` operation on a resource
    /// attribute. Adds `env` = `"test"` to every resource.
    /// Query: `logs | set resource.attributes["env"] = "test"`
    #[test]
    fn validation_transform_opl_set_resource_attribute() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file(
                    "./validation_pipelines/transform-opl-set-resource-attribute-processor.yaml",
                )
                .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::AttributeRequireKeyValue {
                        domains: vec![AttributeDomain::Resource],
                        pairs: vec![KeyValue::new("env".into(), AnyValue::String("test".into()))],
                    }])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transform opl set-resource-attribute validation failed");
    }

    /// Validates that the temporal reaggregation processor passes log signals
    /// through unchanged. The processor only operates on metrics; logs should
    /// be forwarded with no modifications.
    #[test]
    fn validation_temporal_reaggregation_passthrough_logs() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/temporal-reaggregation-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("temporal reaggregation passthrough logs validation failed");
    }

    /// Validates that the temporal reaggregation processor passes trace signals
    /// through unchanged. The processor only operates on metrics; traces should
    /// be forwarded with no modifications.
    #[test]
    fn validation_temporal_reaggregation_passthrough_traces() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/temporal-reaggregation-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::traces()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("temporal reaggregation passthrough traces validation failed");
    }

    /// Validates the temporal reaggregation processor with metric signals.
    #[test]
    fn validation_temporal_reaggregation_metrics() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/temporal-reaggregation-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::metrics()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: Some(0.05),
                            max_drop_ratio: Some(0.80),
                        },
                        ValidationInstructions::AttributeNoDuplicate,
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("temporal reaggregation metrics validation failed");
    }

    /// Validates that the temporal reaggregation processor preserves resource
    /// attributes when passing log signals through. The static signal generator
    /// sets `service.name` to `"load-generator"` on all resources, and this
    /// must be present in the output.
    #[test]
    fn validation_temporal_reaggregation_resource_preservation() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/temporal-reaggregation-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::AttributeRequireKeyValue {
                        domains: vec![AttributeDomain::Resource],
                        pairs: vec![KeyValue::new(
                            "service.name".into(),
                            AnyValue::String("load-generator".into()),
                        )],
                    }])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("temporal reaggregation resource preservation validation failed");
    }

    /// Validates that the temporal reaggregation processor preserves data point
    /// attributes through the aggregation and flush cycle. Static metrics have
    /// `http.method` and `http.route` on every data point, and these must be
    /// present in the flushed output after deduplication.
    #[test]
    fn validation_temporal_reaggregation_metrics_attribute_preservation() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/temporal-reaggregation-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::metrics()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::AttributeRequireKey {
                        domains: vec![AttributeDomain::Signal],
                        keys: vec!["http.method".into(), "http.route".into()],
                    }])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("temporal reaggregation metrics attribute preservation validation failed");
    }

    /// Validates that the temporal reaggregation processor preserves resource
    /// attributes through the aggregation and flush cycle for metric signals.
    /// Unlike `resource_preservation` which tests the passthrough path (logs),
    /// this test exercises the aggregation path where metrics are buffered,
    /// rebuilt by the builder, and flushed.
    #[test]
    fn validation_temporal_reaggregation_metrics_resource_attributes() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/temporal-reaggregation-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::metrics()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![ValidationInstructions::AttributeRequireKeyValue {
                        domains: vec![AttributeDomain::Resource],
                        pairs: vec![KeyValue::new(
                            "service.name".into(),
                            AnyValue::String("load-generator".into()),
                        )],
                    }])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("temporal reaggregation metrics resource attributes validation failed");
    }

    #[test]
    fn validation_multiple_input_output() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/multiple-input-output.yaml")
                    .expect("failed to read in pipeline yaml"),
            )
            .add_generator(
                "traffic_gen1",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver1")
                    .static_signals(),
            )
            .add_generator(
                "traffic_gen2",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver2")
                    .static_signals(),
            )
            .add_capture(
                "validate1",
                Capture::default()
                    .otlp_grpc("exporter1")
                    .validate(vec![ValidationInstructions::Equivalence])
                    .control_streams(["traffic_gen1", "traffic_gen2"]),
            )
            .run()
            .expect("validation scenario failed");
    }

    /// End-to-end validation: transport headers injected by the fake data
    /// generator survive the full pipeline chain (generator → SUV → capture)
    /// and can be asserted via transport header validation instructions.
    ///
    /// Only OTLP receivers and exporters support transport header
    /// capture/propagation, so every hop in the chain uses OTLP gRPC.
    #[test]
    fn validation_transport_headers() {
        use crate::validation_types::transport_headers::TransportHeaderKeyValue;

        let header_key = "x-tenant-id";
        let header_value = "test-tenant";

        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/no-processor.yaml")
                    .expect("failed to read in pipeline yaml")
                    .with_transport_headers_policy_yaml(
                        r#"
header_capture:
  headers:
    - match_names: ["x-tenant-id"]
header_propagation:
  default:
    selector:
      type: all_captured
    action: propagate
"#,
                    )
                    .expect("failed to parse transport headers policy"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals()
                    .with_transport_headers([(header_key, Some(header_value))]),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otlp_grpc("exporter")
                    .with_capture_header_keys([header_key])
                    .validate(vec![
                        ValidationInstructions::TransportHeaderRequireKey {
                            keys: vec![header_key.into()],
                        },
                        ValidationInstructions::TransportHeaderRequireKeyValue {
                            pairs: vec![TransportHeaderKeyValue::new(header_key, header_value)],
                        },
                        ValidationInstructions::TransportHeaderDeny {
                            keys: vec!["x-should-not-exist".into()],
                        },
                    ])
                    .control_streams(["traffic_gen"])
                    .core_range(2, 2),
            )
            .run()
            .expect("transport headers validation failed");
    }
}

#[cfg(test)]
#[cfg(validation_tests)]
mod tls_tests {
    use crate::pipeline::Pipeline;
    use crate::scenario::Scenario;
    use crate::traffic::{Capture, Generator, TlsConfig};
    use otap_test_tls_certs::{ExtendedKeyUsage, write_ca_and_leaf_to_dir};

    /// End-to-end validation: traffic flows through a TLS-enabled OTLP gRPC
    /// receiver in the SUV pipeline.
    #[test]
    fn validation_tls_no_processor() {
        let _ = otap_df_otap::crypto::install_crypto_provider();

        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let dir = temp_dir.path();

        let (_ca, _server_cert) = write_ca_and_leaf_to_dir(
            dir,
            "ca",
            "Test CA",
            "server",
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );

        let server_cert_path = dir.join("server.crt");
        let server_key_path = dir.join("server.key");
        let ca_cert_path = dir.join("ca.crt");

        Scenario::new()
            .pipeline(
                Pipeline::from_file_with_vars(
                    "./validation_pipelines/tls-no-processor.yaml",
                    &[
                        ("TLS_SERVER_CERT", server_cert_path.to_str().unwrap()),
                        ("TLS_SERVER_KEY", server_key_path.to_str().unwrap()),
                    ],
                )
                .expect("failed to read in pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .with_tls(TlsConfig::tls_only(&ca_cert_path)),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["traffic_gen"]),
            )
            .run()
            .expect("TLS validation scenario failed");
    }

    /// End-to-end validation: traffic flows through an mTLS-enabled OTLP gRPC
    /// receiver in the SUV pipeline, requiring client certificate authentication.
    #[test]
    fn validation_mtls_no_processor() {
        let _ = otap_df_otap::crypto::install_crypto_provider();

        let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
        let dir = temp_dir.path();

        // Generate CA + server cert (signed by CA)
        let (ca, _server_cert) = write_ca_and_leaf_to_dir(
            dir,
            "ca",
            "Test CA",
            "server",
            "localhost",
            Some("localhost"),
            Some(ExtendedKeyUsage::ServerAuth),
        );

        // Generate client cert signed by the same CA
        let client_cert = ca.issue_leaf("Test Client", None, Some(ExtendedKeyUsage::ClientAuth));
        client_cert.write_to_dir(dir, "client");

        let server_cert_path = dir.join("server.crt");
        let server_key_path = dir.join("server.key");
        let ca_cert_path = dir.join("ca.crt");
        let client_cert_path = dir.join("client.crt");
        let client_key_path = dir.join("client.key");

        Scenario::new()
            .pipeline(
                Pipeline::from_file_with_vars(
                    "./validation_pipelines/mtls-no-processor.yaml",
                    &[
                        ("TLS_SERVER_CERT", server_cert_path.to_str().unwrap()),
                        ("TLS_SERVER_KEY", server_key_path.to_str().unwrap()),
                        ("TLS_CLIENT_CA", ca_cert_path.to_str().unwrap()),
                    ],
                )
                .expect("failed to read in pipeline yaml"),
            )
            .add_generator(
                "traffic_gen",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .with_tls(TlsConfig::mtls(
                        &ca_cert_path,
                        &client_cert_path,
                        &client_key_path,
                    )),
            )
            .add_capture(
                "validate",
                Capture::default()
                    .otlp_grpc("exporter")
                    .control_streams(["traffic_gen"]),
            )
            .run()
            .expect("mTLS validation scenario failed");
    }
}
