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

#[cfg(test)]
mod tests {
    use crate::pipeline::Pipeline;
    use crate::scenario::Scenario;
    use crate::traffic::{Capture, Generator};
    use std::time::Duration;

    #[test]
    fn no_processor() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/no-processor.yaml")
                    .expect("failed to read in pipeline yaml")
                    .wire_otlp_grpc_receiver("receiver")
                    .wire_otlp_grpc_exporter("exporter"),
            )
            .input(Generator::logs().fixed_count(500).otlp_grpc())
            .observe(Capture::default().otlp_grpc())
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("validation scenario failed");
    }

    #[test]
    fn debug_processor() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/debug-processor.yaml")
                    .expect("failed to read in pipeline yaml")
                    .wire_otlp_grpc_receiver("receiver")
                    .wire_otap_grpc_exporter("exporter"),
            )
            .input(Generator::logs().fixed_count(500).otlp_grpc())
            .observe(Capture::default().otap_grpc())
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
        let _ = rustls::crypto::ring::default_provider().install_default();

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
                .expect("failed to read in pipeline yaml")
                .wire_otlp_grpc_receiver("receiver")
                .wire_otlp_grpc_exporter("exporter"),
            )
            .input(
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc()
                    .with_tls(TlsConfig::tls_only(&ca_cert_path)),
            )
            .observe(Capture::default().otlp_grpc())
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("TLS validation scenario failed");
    }

    /// End-to-end validation: traffic flows through an mTLS-enabled OTLP gRPC
    /// receiver in the SUV pipeline, requiring client certificate authentication.
    #[test]
    fn mtls_no_processor() {
        let _ = rustls::crypto::ring::default_provider().install_default();

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
                .expect("failed to read in pipeline yaml")
                .wire_otlp_grpc_receiver("receiver")
                .wire_otlp_grpc_exporter("exporter"),
            )
            .input(
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc()
                    .with_tls(TlsConfig::mtls(
                        &ca_cert_path,
                        &client_cert_path,
                        &client_key_path,
                    )),
            )
            .observe(Capture::default().otlp_grpc())
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("mTLS validation scenario failed");
    }
}
