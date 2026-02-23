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
    use crate::traffic::{Capture, Generator};
    use crate::validation_types::attributes::{AnyValue, AttributeDomain, KeyValue};

    #[test]
    fn no_processor() {
        let mut pipeline = Pipeline::from_file("./validation_pipelines/no-processor.yaml")
            .expect("failed to read in pipeline yaml");
        pipeline
            .apply_endpoint(
                crate::pipeline::EndpointKind::OtlpGrpcReceiver("receiver".into()),
                50051,
            )
            .unwrap();
        pipeline
            .apply_endpoint(
                crate::pipeline::EndpointKind::OtlpGrpcExporter("exporter".into()),
                50052,
            )
            .unwrap();

        let generator = Generator::logs()
            .fixed_count(500)
            .otlp_grpc("receiver");
        assert_eq!(generator.suv_exporter_node, "receiver");

        let capture = Capture::default().otlp_grpc("exporter");
        assert_eq!(capture.suv_receiver_node, "exporter");
    }

    #[test]
    fn debug_processor() {
        let mut pipeline = Pipeline::from_file("./validation_pipelines/debug-processor.yaml")
            .expect("failed to read in pipeline yaml");
        pipeline
            .apply_endpoint(
                crate::pipeline::EndpointKind::OtlpGrpcReceiver("receiver".into()),
                50053,
            )
            .unwrap();
        pipeline
            .apply_endpoint(
                crate::pipeline::EndpointKind::OtapGrpcExporter("exporter".into()),
                50054,
            )
            .unwrap();

        let generator = Generator::logs()
            .fixed_count(500)
            .otlp_grpc("receiver");
        assert_eq!(generator.suv_exporter_node, "receiver");
        let capture = Capture::default().otap_grpc("exporter");
        assert_eq!(capture.suv_receiver_node, "exporter");
    }

    #[test]
    fn attribute_processor_pipeline() {
        let deny = ValidationInstructions::AttributeDeny {
            domains: vec![AttributeDomain::Signal],
            keys: vec!["ios.app.state".into()],
        };
        let require = ValidationInstructions::AttributeRequireKey {
            domains: vec![AttributeDomain::Signal],
            keys: vec!["ios.app.state2".into()],
        };
        let mut pipeline = Pipeline::from_file("./validation_pipelines/attribute-processor.yaml")
            .expect("failed to read pipeline yaml");
        pipeline
            .apply_endpoint(
                crate::pipeline::EndpointKind::OtlpGrpcReceiver("receiver".into()),
                50055,
            )
            .unwrap();
        pipeline
            .apply_endpoint(
                crate::pipeline::EndpointKind::OtapGrpcExporter("exporter".into()),
                50056,
            )
            .unwrap();

        let generator = Generator::logs()
            .fixed_count(500)
            .otlp_grpc("receiver");
        assert_eq!(generator.suv_exporter_node, "receiver");

        let capture = Capture::default()
            .otap_grpc("exporter")
            .validate(vec![deny, require]);
        assert_eq!(capture.suv_receiver_node, "exporter");
    }

    #[test]
    fn filter_processor_pipeline() {
        let attr_check = ValidationInstructions::AttributeRequireKeyValue {
            domains: vec![AttributeDomain::Signal],
            pairs: vec![KeyValue::new(
                "ios.app.state".into(),
                AnyValue::String("active".into()),
            )],
        };

        let mut pipeline = Pipeline::from_file("./validation_pipelines/filter-processor.yaml")
            .expect("failed to read pipeline yaml");
        pipeline
            .apply_endpoint(
                crate::pipeline::EndpointKind::OtlpGrpcReceiver("receiver".into()),
                50057,
            )
            .unwrap();
        pipeline
            .apply_endpoint(
                crate::pipeline::EndpointKind::OtapGrpcExporter("exporter".into()),
                50058,
            )
            .unwrap();

        let generator = Generator::logs()
            .fixed_count(500)
            .otlp_grpc("receiver");
        assert_eq!(generator.suv_exporter_node, "receiver");

        let capture = Capture::default().otap_grpc("exporter").validate(vec![
            ValidationInstructions::SignalDrop {
                min_drop_ratio: None,
                max_drop_ratio: None,
            },
            attr_check,
        ]);
        assert_eq!(capture.suv_receiver_node, "exporter");
    }
}
