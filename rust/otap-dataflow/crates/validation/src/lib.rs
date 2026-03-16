// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

/// validate the encode_decoding of otlp messages
pub mod encode_decode;
/// error definitions for the validation test
pub mod error;
/// temp fanout processor to use use for validation test
pub mod fanout_processor;
/// metric definition to serialize json result from metric admin endpoint
pub mod metrics_types;
/// module for validating pipelines, runs and monitors pipelines
pub mod pipeline;
/// scenario builder that orchestrates full validation runs
pub mod scenario;
/// internal pipeline simulation utilities
mod simulate;
/// define structs to describe the traffic being created and captured for validation
pub mod traffic;
/// validation exporter to receive messages and assert their equivalence
pub mod validation_exporter;
/// invariants/checks helpers (attribute diff, filtering detection, etc.)
pub mod validation_types;

pub use validation_types::ValidationInstructions;

#[cfg(test)]
mod tests {
    use crate::ValidationInstructions;
    use crate::pipeline::Pipeline;
    use crate::scenario::Scenario;
    use crate::traffic::{Capture, Generator};
    #[cfg(target_os = "linux")]
    use crate::validation_types::attributes::{AnyValue, AttributeDomain, KeyValue};
    use std::time::Duration;

    #[test]
    #[ignore = "flaky test, https://github.com/open-telemetry/otel-arrow/issues/2227"]
    fn no_processor() {
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
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("validation scenario failed");
    }

    #[test]
    #[ignore] // flaky, see https://github.com/open-telemetry/otel-arrow/issues/2227
    fn debug_processor() {
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
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("validation scenario failed");
    }

    // Pipeline validation tests are end-to-end integration tests that spin up real
    // gRPC servers and are inherently slow (~60s+). They validate data correctness
    // through platform-independent code paths, so running on Linux alone is sufficient.
    #[test]
    #[cfg(target_os = "linux")]
    fn attribute_processor_pipeline() {
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
            .expect_within(Duration::from_secs(500))
            .run()
            .expect("attribute processor validation failed");
    }

    #[test]
    #[cfg(target_os = "linux")]
    fn filter_processor_pipeline() {
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
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("filter processor validation failed");
    }

    #[test]
    #[ignore = "flaky test, see https://github.com/open-telemetry/otel-arrow/issues/2227"]
    fn multiple_input_output() {
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
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("validation scenario failed");
    }
}

#[cfg(test)]
#[cfg(feature = "experimental-tls")]
mod tls_tests {
    use crate::pipeline::Pipeline;
    use crate::scenario::Scenario;
    use crate::traffic::{Capture, Generator, TlsConfig};
    use otap_test_tls_certs::{ExtendedKeyUsage, write_ca_and_leaf_to_dir};
    use std::time::Duration;

    /// End-to-end validation: traffic flows through a TLS-enabled OTLP gRPC
    /// receiver in the SUV pipeline.
    #[test]
    fn tls_no_processor() {
        otap_df_otap::crypto::ensure_crypto_provider();

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
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("TLS validation scenario failed");
    }

    /// End-to-end validation: traffic flows through an mTLS-enabled OTLP gRPC
    /// receiver in the SUV pipeline, requiring client certificate authentication.
    #[test]
    fn mtls_no_processor() {
        otap_df_otap::crypto::ensure_crypto_provider();

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
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("mTLS validation scenario failed");
    }
}
