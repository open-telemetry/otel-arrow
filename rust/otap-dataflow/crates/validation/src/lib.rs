// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

use serde::{Deserialize, Serialize};

use crate::checks::AttributeCheck;

/// invariants/checks helpers (attribute diff, filtering detection, etc.)
pub mod checks;
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

/// Supported validation kinds executed by the validation exporter.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValidationKind {
    /// Check semantic equivalence between control and suv outputs.
    Equivalence,
    /// Check that after contains fewer signals than before.
    SignalDrop,
    /// Check that each message meets a minimum batch size (applied to SUV messages).
    Batch {
        /// Minimum items required in each message.
        min_batch_size: usize,
    },
    /// Check attribute presence/absence rules (applied to SUV messages).
    Attributes {
        /// Attribute rules to enforce.
        config: AttributeCheck,
    },
}
