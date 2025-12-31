// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validation test module to validate the encoding/decoding process for otlp messages

// ToDo: Add support to simulate a pipeline with various processors
// ToDo: Move the validation process to it's own CICD job (outside of the tests)
use otap_df_pdata::otap::{OtapArrowRecords, from_record_messages};
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::testing::round_trip::{otap_to_otlp, otlp_to_otap};
use otap_df_pdata::{Consumer, Producer};
use weaver_common::result::WResult;
use weaver_common::vdir::VirtualDirectoryPath;
use weaver_forge::registry::ResolvedRegistry;
use weaver_resolver::SchemaResolver;
use weaver_semconv::registry::SemConvRegistry;
use weaver_semconv::registry_repo::RegistryRepo;

/// struct to simulate the otel arrow protocol, uses a producer and consumer to encode and decode a otlp request
pub struct OtelProtoSimulator {
    producer: Producer,
    consumer: Consumer,
}

impl OtelProtoSimulator {
    /// Takes the Otlp request message and encodes it and decodes it via producer -> consumer
    pub fn simulate_proto(&mut self, proto_message: &OtlpProtoMessage) -> OtlpProtoMessage {
        // take otlp proto message
        // convert to otap arrow records which we can pass to the producer
        let mut otap_message = otlp_to_otap(proto_message);
        // convert to batch arrow records
        // converg batch arrow records
        // convert msg to proto bytes?
        let mut bar = self.producer.produce_bar(&mut otap_message).unwrap();
        let records = self.consumer.consume_bar(&mut bar).unwrap();
        let otap_message = match proto_message {
            OtlpProtoMessage::Logs(_) => OtapArrowRecords::Logs(from_record_messages(records)),
            OtlpProtoMessage::Metrics(_) => {
                OtapArrowRecords::Metrics(from_record_messages(records))
            }
            OtlpProtoMessage::Traces(_) => OtapArrowRecords::Traces(from_record_messages(records)),
        };
        otap_to_otlp(&otap_message)
    }

    // ToDo: add function to simulate pipeline
    // if pipeline alters the data via a processor that performs some transofmration we should expect the equivalent assert to fail
    // otherwise the assert should succeed
    // pub fn simulate_pipeline(proto_message: OtlpProtoMessage) -> OtlpProtoMessage {
    //     // todo: run a pipeline
    // }
}

impl Default for OtelProtoSimulator {
    fn default() -> Self {
        Self {
            producer: Producer::new(),
            consumer: Consumer::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::fake_data_generator::fake_signal::{
        fake_otlp_logs, fake_otlp_metrics, fake_otlp_traces,
    };
    use otap_df_pdata::testing::equiv::assert_equivalent;

    const LOG_SIGNAL_COUNT: usize = 100;
    const METRIC_SIGNAL_COUNT: usize = 100;
    const TRACE_SIGNAL_COUNT: usize = 100;
    const ITERATIONS: usize = 10;

    fn get_registry() -> ResolvedRegistry {
        let registry_repo = RegistryRepo::try_new(
            "main",
            &VirtualDirectoryPath::GitRepo {
                url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
                sub_folder: Some("model".to_owned()),
                refspec: None,
            },
        )
        .expect("all registries are definied under the model folder in semantic convention repo");

        // Load the semantic convention specs
        let semconv_specs = match SchemaResolver::load_semconv_specs(&registry_repo, true, false) {
            WResult::Ok(semconv_specs) => semconv_specs,
            WResult::OkWithNFEs(semconv_specs, _) => semconv_specs,
            WResult::FatalErr(err) => {
                panic!("Failed to load semantic convention specs");
            }
        };

        // Resolve the main registry
        let mut registry = SemConvRegistry::from_semconv_specs(&registry_repo, semconv_specs)
            .expect("Can resolve the registries defined in semantic convention repo");
        // Resolve the semantic convention specifications.
        // If there are any resolution errors, they should be captured into the ongoing list of
        // diagnostic messages and returned immediately because there is no point in continuing
        // as the resolution is a prerequisite for the next stages.
        let resolved_schema =
            match SchemaResolver::resolve_semantic_convention_registry(&mut registry, true) {
                WResult::Ok(resolved_schema) => resolved_schema,
                WResult::OkWithNFEs(resolved_schema, _) => resolved_schema,
                WResult::FatalErr(err) => {
                    panic!("Failed to resolve semantic convetion schema");
                }
            };

        let resolved_registry = ResolvedRegistry::try_from_resolved_registry(
            &resolved_schema.registry,
            resolved_schema.catalog(),
        )
        .expect("can get resolved registry from official semantic convention repo");
        resolved_registry
    }

    // validate the encoding and decoding
    #[test]
    fn validate_encode_decode() {
        let mut otel_proto_simulator = OtelProtoSimulator::default();

        let registry = get_registry();

        for _ in 0..ITERATIONS {
            // generate data and simulate the protocol and compare result
            let logs = OtlpProtoMessage::Logs(fake_otlp_logs(LOG_SIGNAL_COUNT, &registry));
            let logs_output = otel_proto_simulator.simulate_proto(&logs);
            assert_equivalent(&[logs], &[logs_output]);

            let metrics =
                OtlpProtoMessage::Metrics(fake_otlp_metrics(METRIC_SIGNAL_COUNT, &registry));
            let metrics_output = otel_proto_simulator.simulate_proto(&metrics);
            assert_equivalent(&[metrics], &[metrics_output]);

            let traces = OtlpProtoMessage::Traces(fake_otlp_traces(TRACE_SIGNAL_COUNT, &registry));
            let traces_output = otel_proto_simulator.simulate_proto(&traces);
            assert_equivalent(&[traces], &[traces_output]);
        }
    }
}
