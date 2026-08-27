// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! End-to-end translation tests against the real AMCS payloads used by the .NET agent's own
//! test suite.
//!
//! The fixtures in `tests/fixtures/` are verbatim copies of
//! `PipelineAgentTests/TestData/ConfigPackage/AMCSConfig*` from the `PipelineAgent` repository,
//! renamed with a `.json` extension. They are the same inputs `AmcsConfigProviderTests` exercises,
//! so behaviour here can be compared directly against the .NET reference implementation.
//!
//! Fixture coverage:
//!
//! | Fixture | Shape |
//! |---|---|
//! | `AMCSConfig` | one rule; logs + traces + `perfCounter`; `gig` + `ods` channels |
//! | `AMCSConfig2` | as above, different identifiers |
//! | `AMCSConfig3` | **two rules** sharing one channel id, with different routing values |
//! | `AMCSConfig4` | no data sources at all |
//! | `AMCSConfig5` | **logs only** -- the `gig` channel has no traces template |

use otel_arrow_dfe_config::engine::OtelDataflowSpec;
use otel_arrow_dfe_contrib_config_translators::ConfigTranslator;
use otel_arrow_dfe_contrib_config_translators::amcs::AmcsTranslator;
use otel_arrow_dfe_contrib_config_translators::amcs::listener::{
    ENV_OTLP_GRPC_LOGS_TRACES_PORT, ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, StaticEnvironment,
};
use otel_arrow_dfe_contrib_config_translators::error::Error;

/// Load a fixture by name.
fn fixture(name: &str) -> String {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    std::fs::read_to_string(&path).unwrap_or_else(|e| panic!("cannot read fixture {path}: {e}"))
}

/// A translator with both listeners enabled on their default ports.
fn default_translator() -> AmcsTranslator<StaticEnvironment> {
    AmcsTranslator::with_environment(StaticEnvironment::new())
}

/// A translator with only the gRPC listener enabled, so each binding appears once.
fn grpc_only_translator() -> AmcsTranslator<StaticEnvironment> {
    AmcsTranslator::with_environment(
        StaticEnvironment::new().with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1"),
    )
}

/// Translate a fixture, asserting success.
fn translate(translator: &AmcsTranslator<StaticEnvironment>, name: &str) -> String {
    translator
        .translate_to_yaml(&fixture(name))
        .unwrap_or_else(|e| panic!("translation of {name} failed: {e}"))
}

/// Every fixture that yields a pipeline must produce YAML the engine's own loader accepts.
/// This is the guard against emitting a specification the engine would refuse to start.
#[test]
fn all_usable_fixtures_round_trip_through_the_engine_loader() {
    for name in ["AMCSConfig", "AMCSConfig2", "AMCSConfig3", "AMCSConfig5"] {
        let yaml = translate(&default_translator(), name);
        let spec = OtelDataflowSpec::from_yaml(&yaml);
        assert!(
            spec.is_ok(),
            "engine rejected the configuration generated from {name}: {spec:?}\n{yaml}"
        );
    }
}

/// No `<STREAM>` placeholder may survive into a generated configuration.
#[test]
fn no_fixture_leaves_an_unsubstituted_stream_placeholder() {
    for name in ["AMCSConfig", "AMCSConfig2", "AMCSConfig3", "AMCSConfig5"] {
        let yaml = translate(&default_translator(), name);
        assert!(
            !yaml.contains("<STREAM>"),
            "{name} produced an unsubstituted <STREAM> placeholder:\n{yaml}"
        );
    }
}

/// The canonical single-rule fixture produces one receiver and one exporter carrying both the
/// logs and the traces endpoint.
#[test]
fn canonical_fixture_produces_one_receiver_and_one_exporter() {
    let yaml = translate(&grpc_only_translator(), "AMCSConfig");

    assert_eq!(
        yaml.matches("urn:otel:receiver:otlp").count(),
        1,
        "expected exactly one receiver:\n{yaml}"
    );
    assert_eq!(
        yaml.matches("urn:otel:exporter:otlp_http").count(),
        1,
        "logs and traces for one identifier share an exporter:\n{yaml}"
    );

    assert!(yaml.contains("OPENTELEMETRY_LOGS_AGENT"));
    assert!(yaml.contains("OPENTELEMETRY_TRACES_AGENT"));
    assert!(yaml.contains("otlp/v1/logs"));
    assert!(yaml.contains("otlp/v1/traces"));
    assert!(yaml.contains("service.name"));
}

/// `perfCounter` data sources and `ods` channels carry no OTLP endpoints and must not appear.
#[test]
fn non_otlp_data_sources_and_ods_channels_are_excluded() {
    let yaml = translate(&default_translator(), "AMCSConfig");

    assert!(
        !yaml.contains("GENERIC_PERF_BLOB") && !yaml.contains("LINUX_PERF_BLOB"),
        "a perfCounter stream leaked into the configuration:\n{yaml}"
    );
    assert!(
        !yaml.contains("opinsights"),
        "an ods channel endpoint leaked into the configuration:\n{yaml}"
    );
}

/// Ragu's multi-DCR scenario: `AMCSConfig3` carries two rules in one payload. Both must appear,
/// each with its own routing value, and they must not collide even though they send to the same
/// channel id.
#[test]
fn multiple_rules_in_one_payload_produce_separate_branches() {
    let yaml = translate(&grpc_only_translator(), "AMCSConfig3");

    assert_eq!(
        yaml.matches("urn:otel:exporter:otlp_http").count(),
        2,
        "expected one exporter per rule:\n{yaml}"
    );
    assert_eq!(yaml.matches("urn:otel:receiver:otlp").count(), 1);

    // Distinct routing values prove the two rules stayed separate.
    assert!(
        yaml.contains("amcs_1"),
        "missing routing for the first rule"
    );
    assert!(
        yaml.contains("amcs_2"),
        "missing routing for the second rule"
    );

    // Distinct streams prove each rule kept its own endpoints.
    assert!(yaml.contains("OPENTELEMETRY_LOGS_AGENT_1"));
    assert!(yaml.contains("OPENTELEMETRY_LOGS_AGENT_2"));
    assert!(yaml.contains("OPENTELEMETRY_TRACES_AGENT_1"));
    assert!(yaml.contains("OPENTELEMETRY_TRACES_AGENT_2"));
}

/// `AMCSConfig5` has a `gig` channel with a logs template but **no** traces template. The
/// generated configuration must carry a logs endpoint, no traces endpoint, and must actively drop
/// traces -- otherwise the exporter would fall back to `endpoint + "/v1/traces"` and misroute them.
#[test]
fn logs_only_fixture_emits_no_traces_endpoint_and_drops_traces() {
    let yaml = translate(&grpc_only_translator(), "AMCSConfig5");

    assert!(
        yaml.contains("logs_endpoint"),
        "expected a logs endpoint:\n{yaml}"
    );
    assert!(
        !yaml.contains("traces_endpoint"),
        "a traces endpoint was emitted for a channel that has no traces template:\n{yaml}"
    );
    assert!(
        yaml.contains("urn:otel:processor:filter"),
        "a logs-only branch needs a filter to drop traces:\n{yaml}"
    );
    assert!(
        yaml.contains("span_names"),
        "expected a span_names constraint that drops traces:\n{yaml}"
    );

    // The `extension` data sources in this fixture are not OTLP and must be ignored.
    assert!(!yaml.contains("RETINA_NETWORK_FLOW_LOGS"));
    assert!(!yaml.contains("CONTAINER_LOG_BLOB"));
    assert!(yaml.contains("OPENTELEMETRY_LOGS_AGENT"));
}

/// A payload with no OTLP data sources cannot produce a pipeline, and must say so rather than
/// emitting an empty or invalid configuration.
#[test]
fn fixture_without_otlp_data_sources_is_an_empty_pipeline_error() {
    let err = default_translator()
        .translate_to_yaml(&fixture("AMCSConfig4"))
        .expect_err("AMCSConfig4 declares no OTLP data sources");

    assert!(
        matches!(err, Error::EmptyPipeline { .. }),
        "expected EmptyPipeline, got {err:?}"
    );
}

/// Both listeners enabled means both protocols appear on the single shared receiver, on their
/// default ports, resolved to literal addresses.
#[test]
fn both_listeners_appear_on_the_shared_receiver() {
    let yaml = translate(&default_translator(), "AMCSConfig");

    assert!(
        yaml.contains(":4319"),
        "missing the default gRPC port:\n{yaml}"
    );
    assert!(
        yaml.contains(":4320"),
        "missing the default HTTP port:\n{yaml}"
    );
    assert!(
        !yaml.contains("localhost:"),
        "listening_addr must be a literal socket address, not a hostname:\n{yaml}"
    );
}

/// Disabling both listeners leaves nothing to build.
#[test]
fn disabling_every_listener_yields_an_empty_pipeline_error() {
    let translator = AmcsTranslator::with_environment(
        StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "-1")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1"),
    );

    let err = translator
        .translate_to_yaml(&fixture("AMCSConfig"))
        .expect_err("no listeners means no pipeline");

    assert!(matches!(err, Error::EmptyPipeline { .. }));
}

/// Adding the second listener must not duplicate exporters: listeners configure the receiver,
/// identifiers configure the branches.
#[test]
fn listener_count_does_not_change_the_number_of_exporters() {
    let one = translate(&grpc_only_translator(), "AMCSConfig");
    let two = translate(&default_translator(), "AMCSConfig");

    assert_eq!(
        one.matches("urn:otel:exporter:otlp_http").count(),
        two.matches("urn:otel:exporter:otlp_http").count(),
        "the number of exporters must depend on identifiers, not listeners"
    );
}

/// Translation must be stable across runs so generated configurations can be cached and compared.
///
/// The engine stores nodes in a `HashMap`, so the serialized YAML text is not byte-stable. The
/// specification it represents must be.
#[test]
fn translation_is_stable_across_runs() {
    for name in ["AMCSConfig", "AMCSConfig3", "AMCSConfig5"] {
        let raw = fixture(name);
        let first = default_translator()
            .translate(&raw)
            .unwrap_or_else(|e| panic!("first translation of {name} failed: {e}"));
        let second = default_translator()
            .translate(&raw)
            .unwrap_or_else(|e| panic!("second translation of {name} failed: {e}"));
        assert_eq!(first, second, "{name} translated differently across runs");
    }
}

/// Scenario: a payload is translated through the library API without any file being written.
/// Guarantees: the engine configuration is produced entirely in memory, so the embedding host can
/// hand the specification straight to the engine and treat any YAML dump as a debug artifact
/// rather than the interface. Removing the file write cannot change what the engine runs.
#[test]
fn translation_produces_a_spec_without_touching_the_filesystem() {
    let raw = fixture("AMCSConfig");

    // `translate` returns the specification itself; no path, handle or temporary file is involved.
    let spec = default_translator()
        .translate(&raw)
        .expect("translation should succeed");

    // The same specification is what a YAML dump would serialize, so the dump carries no
    // information the in-memory value lacks.
    let yaml = serde_yaml::to_string(&spec).expect("spec should serialize");
    let reparsed = OtelDataflowSpec::from_yaml(&yaml).expect("dump should parse back");
    assert_eq!(
        spec, reparsed,
        "a YAML round-trip must not alter the specification"
    );
}

/// Custom listener host and port settings must reach the generated receiver.
#[test]
fn custom_listener_settings_reach_the_receiver() {
    let translator = AmcsTranslator::with_environment(
        StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "15317")
            .with(
                otel_arrow_dfe_contrib_config_translators::amcs::listener::ENV_OTLP_GRPC_LOGS_TRACES_HOST,
                "0.0.0.0",
            )
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1"),
    );

    let yaml = translator
        .translate_to_yaml(&fixture("AMCSConfig"))
        .expect("translation should succeed");

    assert!(
        yaml.contains("0.0.0.0:15317"),
        "custom listener settings did not reach the receiver:\n{yaml}"
    );
    assert!(!yaml.contains(":4320"), "the HTTP listener was disabled");
}

// -------------------------------------------------------------------------------------------
// Agent Settings DCR, per Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md.
// -------------------------------------------------------------------------------------------

/// Scenario 8: no environment variables, an agent settings rule supplying both ports, and OTel
/// data sources present -- the listeners use the agent settings values.
#[test]
fn agent_settings_ports_reach_the_receiver() {
    let yaml = translate(&default_translator(), "AMCSConfigAgentSettings");

    assert!(
        yaml.contains("127.0.0.1:4329"),
        "gRPC listener should use the agent settings port 4329:\n{yaml}"
    );
    assert!(
        yaml.contains("127.0.0.1:4330"),
        "HTTP listener should use the agent settings port 4330:\n{yaml}"
    );
    assert!(
        !yaml.contains(":4319") && !yaml.contains(":4320"),
        "default ports must not appear when agent settings supply values:\n{yaml}"
    );
}

/// Scenario 4: environment variables always win over the agent settings rule.
#[test]
fn environment_overrides_agent_settings_end_to_end() {
    let translator = AmcsTranslator::with_environment(
        StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "4319")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "4320"),
    );

    let yaml = translator
        .translate_to_yaml(&fixture("AMCSConfigAgentSettings"))
        .expect("translation should succeed");

    assert!(yaml.contains("127.0.0.1:4319"));
    assert!(yaml.contains("127.0.0.1:4320"));
    assert!(
        !yaml.contains(":4329") && !yaml.contains(":4330"),
        "agent settings must be ignored when the environment supplies ports:\n{yaml}"
    );
}

/// Scenarios 2, 6 and 10: an agent settings rule on its own is **not** sufficient to open a port.
/// Without an OTel data-source rule there is nothing to build.
#[test]
fn agent_settings_alone_opens_no_ports() {
    let err = default_translator()
        .translate_to_yaml(&fixture("AMCSConfigAgentSettingsOnly"))
        .expect_err("an agent settings rule alone must not produce a pipeline");

    assert!(
        matches!(err, Error::EmptyPipeline { .. }),
        "expected EmptyPipeline, got {err:?}"
    );
}

/// Scenario 14: `-1` in the environment disables the listeners even when agent settings supply
/// ports and OTel data sources are present.
#[test]
fn environment_disable_beats_agent_settings_end_to_end() {
    let translator = AmcsTranslator::with_environment(
        StaticEnvironment::new()
            .with(ENV_OTLP_GRPC_LOGS_TRACES_PORT, "-1")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1"),
    );

    let err = translator
        .translate_to_yaml(&fixture("AMCSConfigAgentSettings"))
        .expect_err("both listeners disabled means no pipeline");

    assert!(matches!(err, Error::EmptyPipeline { .. }));
}

/// The agent settings rule must never become a pipeline branch of its own: the payload has one
/// data-source rule, so exactly one exporter is expected.
#[test]
fn agent_settings_rule_is_not_treated_as_a_branch() {
    let yaml = translate(&grpc_only_translator(), "AMCSConfigAgentSettings");

    assert_eq!(
        yaml.matches("urn:otel:exporter:otlp_http").count(),
        1,
        "the agent settings rule must not produce a branch:\n{yaml}"
    );
    assert!(
        !yaml.contains("MaxDiskQuotaInMB") && !yaml.contains("d7e9af8dbad14f40900dbf304e999ae2"),
        "agent settings content leaked into the pipeline:\n{yaml}"
    );

    // The single branch still comes from the data-source rule.
    assert!(yaml.contains("OPENTELEMETRY_LOGS_AGENT"));
    assert!(yaml.contains("OPENTELEMETRY_TRACES_AGENT"));
}

/// A payload carrying an agent settings rule must still produce a configuration the engine
/// accepts.
#[test]
fn agent_settings_payload_round_trips_through_the_engine_loader() {
    let yaml = translate(&default_translator(), "AMCSConfigAgentSettings");
    let spec = OtelDataflowSpec::from_yaml(&yaml);
    assert!(
        spec.is_ok(),
        "engine rejected the configuration: {spec:?}\n{yaml}"
    );
}

/// Scenario: a generated YAML document is inspected for the engine-wide `engine:` block.
/// Guarantees: the block is omitted while it holds nothing but engine defaults. It is
/// `#[serde(default)]`, so an absent block parses back to exactly `EngineConfig::default()` and
/// the engine behaves identically -- but emitting it would freeze roughly sixty lines of internal
/// observability pipeline into every generated config, dwarfing the pipeline we actually generate
/// and silently pinning defaults that would otherwise track the engine. Deliberate engine
/// settings differ from the default and so are still written out.
#[test]
fn default_engine_configuration_is_not_pinned_into_the_output() {
    let yaml = default_translator()
        .translate_to_yaml(&fixture("AMCSConfig"))
        .expect("translation should succeed");

    assert!(
        !yaml.contains("\nengine:"),
        "a defaults-only engine block must not be written out:\n{yaml}"
    );
    // The observability pipeline is the bulky part of those defaults; make its absence explicit
    // so this test fails loudly if the block ever creeps back in.
    assert!(
        !yaml.contains("internal_telemetry"),
        "default observability pipeline leaked into the output:\n{yaml}"
    );

    // Omitting the block must not change what the engine actually runs.
    let spec = OtelDataflowSpec::from_yaml(&yaml).expect("output should parse back");
    assert_eq!(
        spec.engine,
        otel_arrow_dfe_config::engine::EngineConfig::default(),
        "an absent engine block must deserialize to the engine defaults"
    );
}
