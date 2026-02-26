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
    use crate::validation_types::attributes::{AnyValue, AttributeDomain, KeyValue};
    use std::time::Duration;

    #[test]
    fn no_processor() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/no-processor.yaml")
                    .expect("failed to read in pipeline yaml"),
            )
            .add_generator(
                "input",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "output",
                Capture::default().otlp_grpc("exporter").core_range(2, 2),
            )
            .connect("input", "output")
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("validation scenario failed");
    }

    #[test]
    fn debug_processor() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/debug-processor.yaml")
                    .expect("failed to read in pipeline yaml"),
            )
            .add_generator(
                "input",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "output",
                Capture::default().otap_grpc("exporter").core_range(2, 2),
            )
            .connect("input", "output")
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("validation scenario failed");
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
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/attribute-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "input",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .static_signals()
                    .core_range(1, 1),
            )
            .add_capture(
                "output",
                Capture::default()
                    .otlp_grpc("exporter")
                    .validate(vec![deny, require])
                    .core_range(2, 2),
            )
            .expect_within(Duration::from_secs(500))
            .run()
            .expect("attribute processor validation failed");
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

        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/filter-processor.yaml")
                    .expect("failed to read pipeline yaml"),
            )
            .add_generator(
                "input",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver")
                    .core_range(1, 1)
                    .static_signals(),
            )
            .add_capture(
                "output",
                Capture::default()
                    .otap_grpc("exporter")
                    .validate(vec![
                        ValidationInstructions::SignalDrop {
                            min_drop_ratio: None,
                            max_drop_ratio: None,
                        },
                        attr_check,
                    ])
                    .core_range(2, 2),
            )
            .connect("input", "output")
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("filter processor validation failed");
    }

    #[test]
    fn multiple_input_output() {
        Scenario::new()
            .pipeline(
                Pipeline::from_file("./validation_pipelines/multiple-input-output.yaml")
                    .expect("failed to read in pipeline yaml"),
            )
            .add_generator(
                "input1",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver1")
                    .static_signals()
                    .core_range(1, 1),
            )
            .add_generator(
                "input2",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver2")
                    .static_signals()
                    .core_range(2, 2),
            )
            .add_generator(
                "input3",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver3")
                    .static_signals()
                    .core_range(3, 3),
            )
            .add_generator(
                "input4",
                Generator::logs()
                    .fixed_count(500)
                    .otlp_grpc("receiver4")
                    .static_signals()
                    .core_range(4, 4),
            )
            .add_capture(
                "output1",
                Capture::default().otlp_grpc("exporter1").core_range(5, 5),
            )
            .add_capture(
                "output2",
                Capture::default().otlp_grpc("exporter2").core_range(6, 6),
            )
            .connect("input1", "output1")
            .connect("input2", "output1")
            .connect("input3", "output1")
            .connect("input4", "output2")
            .expect_within(Duration::from_secs(140))
            .run()
            .expect("validation scenario failed");
    }
}
