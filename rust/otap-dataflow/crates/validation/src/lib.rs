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
    /// 10) across a range of input sizes. The sampler should drop approximately
    /// 90% of log records regardless of how many signals are sent.
    #[test]
    fn validation_log_sampling_ratio_pipeline() {
        for signal_count in [10, 100, 500] {
            let min_drop = if signal_count <= 10 { 0.70 } else { 0.80 };

            Scenario::new()
                .pipeline(
                    Pipeline::from_file("./validation_pipelines/log-sampling-processor.yaml")
                        .expect("failed to read pipeline yaml"),
                )
                .add_generator(
                    "traffic_gen",
                    Generator::logs()
                        .fixed_count(signal_count)
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
                                min_drop_ratio: Some(min_drop),
                                max_drop_ratio: Some(0.99),
                            },
                            ValidationInstructions::AttributeNoDuplicate,
                        ])
                        .control_streams(["traffic_gen"])
                        .core_range(2, 2),
                )
                .run()
                .unwrap_or_else(|e| {
                    panic!(
                        "log sampling ratio validation failed for signal_count={signal_count}: {e}"
                    )
                });
        }
    }

    /// Validates the transform processor with a KQL filter query that keeps
    /// only ERROR-severity logs across a range of input sizes. Static log
    /// signals have ~5% ERROR, ~15% WARN, and ~80% INFO. Severity assignment
    /// cycles every 20 records, so inputs below 20 may contain zero ERROR
    /// records.
    #[test]
    fn validation_transform_processor_filter_pipeline() {
        for signal_count in [10, 100, 500] {
            // With < 20 signals the severity cycle may yield zero ERROR logs,
            // resulting in 100% drop.
            let max_drop = if signal_count < 20 { 1.0 } else { 0.99 };

            Scenario::new()
                .pipeline(
                    Pipeline::from_file("./validation_pipelines/transform-processor.yaml")
                        .expect("failed to read pipeline yaml"),
                )
                .add_generator(
                    "traffic_gen",
                    Generator::logs()
                        .fixed_count(signal_count)
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
                                max_drop_ratio: Some(max_drop),
                            },
                            ValidationInstructions::AttributeNoDuplicate,
                        ])
                        .control_streams(["traffic_gen"])
                        .core_range(2, 2),
                )
                .run()
                .unwrap_or_else(|e| {
                    panic!("transform processor filter validation failed for signal_count={signal_count}: {e}")
                });
        }
    }

    /// Validates the log sampling processor with a zip policy (max 50 items per
    /// 60-second window) across a range of input sizes. When input is within
    /// the budget (≤50), all signals pass through. When input exceeds the
    /// budget, only 50 are kept.
    #[test]
    fn validation_log_sampling_zip_pipeline() {
        for signal_count in [10, 100, 500] {
            // Zip budget is 50 items. Signals within budget pass through unchanged.
            let validations = if signal_count <= 50 {
                vec![ValidationInstructions::Equivalence]
            } else {
                vec![
                    ValidationInstructions::SignalDrop {
                        min_drop_ratio: Some(0.30),
                        max_drop_ratio: Some(0.99),
                    },
                    ValidationInstructions::AttributeNoDuplicate,
                ]
            };

            Scenario::new()
                .pipeline(
                    Pipeline::from_file("./validation_pipelines/log-sampling-zip-processor.yaml")
                        .expect("failed to read pipeline yaml"),
                )
                .add_generator(
                    "traffic_gen",
                    Generator::logs()
                        .fixed_count(signal_count)
                        .otlp_grpc("receiver")
                        .core_range(1, 1)
                        .static_signals(),
                )
                .add_capture(
                    "validate",
                    Capture::default()
                        .otap_grpc("exporter")
                        .validate(validations)
                        .control_streams(["traffic_gen"])
                        .core_range(2, 2),
                )
                .run()
                .unwrap_or_else(|e| {
                    panic!(
                        "log sampling zip validation failed for signal_count={signal_count}: {e}"
                    )
                });
        }
    }

    /// Validates that the log sampling processor passes non-log signals through
    /// unchanged. Sends trace signals through a ratio sampler and asserts
    /// semantic equivalence -- the sampler should not alter traces at all.
    #[test]
    fn validation_log_sampling_passthrough_traces() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-processor.yaml")
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
            .expect("log sampling passthrough traces validation failed");
    }

    /// Validates that the log sampling processor passes metric signals through
    /// unchanged. The sampler only operates on logs; metrics should be forwarded
    /// with no modifications.
    #[test]
    fn validation_log_sampling_passthrough_metrics() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/log-sampling-processor.yaml")
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
            .expect("log sampling passthrough metrics validation failed");
    }

    /// Validates the log sampling processor with a full passthrough ratio policy
    /// (emit 1 out of 1 = 100% sampling). All log signals should pass through
    /// unchanged.
    #[test]
    fn validation_log_sampling_full_passthrough() {
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
            .expect("log sampling full passthrough validation failed");
    }

    /// Validates the transform processor with a negated KQL filter that drops
    /// INFO-severity logs and keeps WARN + ERROR across a range of input sizes.
    /// Static log signals have ~80% INFO. Severity assignment cycles every 20
    /// records, so inputs below 20 may contain only INFO records.
    #[test]
    fn validation_transform_processor_not_filter_pipeline() {
        for signal_count in [10, 100, 500] {
            // With < 20 signals the severity cycle may yield only INFO logs,
            // resulting in 100% drop.
            let max_drop = if signal_count < 20 { 1.0 } else { 0.90 };

            Scenario::new()
                .pipeline(
                    Pipeline::from_file("./validation_pipelines/transform-not-filter-processor.yaml")
                        .expect("failed to read pipeline yaml"),
                )
                .add_generator(
                    "traffic_gen",
                    Generator::logs()
                        .fixed_count(signal_count)
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
                                max_drop_ratio: Some(max_drop),
                            },
                            ValidationInstructions::AttributeNoDuplicate,
                        ])
                        .control_streams(["traffic_gen"])
                        .core_range(2, 2),
                )
                .run()
                .unwrap_or_else(|e| {
                    panic!("transform processor not-filter validation failed for signal_count={signal_count}: {e}")
                });
        }
    }

    /// Validates the transform processor with an OPL set query that adds a new
    /// attribute to every log record. All logs should pass through (no
    /// filtering) and the new attribute key must be present on every record.
    #[test]
    fn validation_transform_processor_attribute_set_pipeline() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/transform-set-processor.yaml")
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
            .expect("transform processor attribute set validation failed");
    }

    /// Validates that the transform processor passes non-matching signal types
    /// through unchanged. The query is scoped to `logs |`, so trace signals
    /// should be forwarded with no modifications.
    #[test]
    fn validation_transform_processor_passthrough_traces() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/transform-processor.yaml")
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
            .expect("transform processor passthrough traces validation failed");
    }

    /// Validates the transform processor with an OPL exclude query that removes
    /// the `thread.id` attribute from every log record. The `thread.name`
    /// attribute should still be present after the transform.
    #[test]
    fn validation_transform_processor_attribute_exclude() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/transform-exclude-processor.yaml")
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
            .expect("transform processor attribute exclude validation failed");
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

    /// Validates the temporal reaggregation processor with metric signals across
    /// a range of input sizes. Static metrics (cumulative sums and gauges) share
    /// the same stream identity, so all data points collapse to ~2 outputs (one
    /// per metric name) regardless of input size. Drop ratio scales with input.
    #[test]
    fn validation_temporal_reaggregation_metrics() {
        for signal_count in [10, 100, 500] {
            // Output is ~2 data points (one per metric name). Drop ratio = 1 - 2/n.
            // With 10 signals: 80% drop. With 100: 98%. With 500: 99.6%.
            // Use a formula with tolerance: min_drop = 1 - 4/n (allowing 2x output).
            let min_drop = (1.0 - 4.0 / signal_count as f64).max(0.30);

            Scenario::new()
                .pipeline(
                    Pipeline::from_file(
                        "./validation_pipelines/temporal-reaggregation-processor.yaml",
                    )
                    .expect("failed to read pipeline yaml"),
                )
                .add_generator(
                    "traffic_gen",
                    Generator::metrics()
                        .fixed_count(signal_count)
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
                                min_drop_ratio: Some(min_drop),
                                max_drop_ratio: Some(1.0),
                            },
                            ValidationInstructions::AttributeNoDuplicate,
                        ])
                        .control_streams(["traffic_gen"])
                        .core_range(2, 2),
                )
                .run()
                .unwrap_or_else(|e| {
                    panic!("temporal reaggregation metrics validation failed for signal_count={signal_count}: {e}")
                });
        }
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
