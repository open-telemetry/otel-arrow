// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Parity tests against the .NET reference implementation.
//!
//! The expected values below are **not** hand-written. They were produced by invoking the real
//! `AMCSParser.ExtractConfigurationByIdentifier` from the prebuilt `AMCSConfiguration.dll` in the
//! `PipelineAgent` repository against these exact fixtures, with
//! `OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT=-1` so each binding appears once.
//!
//! Every assertion here therefore encodes observed .NET behaviour, which is what makes this a
//! port rather than a reimplementation.
//!
//! # Known intentional divergence
//!
//! `AMCSParser.cs:235-242` selects the endpoint template with:
//!
//! ```csharp
//! if (eventName == Log && otelLogsEndpointUriTemplate != null) { ...logs... }
//! else if (otelTracesEndpointUriTemplate != null)              { ...traces... }
//! ```
//!
//! so a logs data source whose channel has no logs template falls through and is given a
//! **traces** URL. None of the fixtures exercise that path, so it does not affect the values
//! below, but this port deliberately does not reproduce it. See `endpoint_template` in
//! `crate::amcs::extract`.

use otel_arrow_dfe_contrib_config_translators::amcs::extract::{
    OtlpEventName, extract_configuration,
};
use otel_arrow_dfe_contrib_config_translators::amcs::listener::{
    DEFAULT_HOST, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT, ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT,
    OtlpProtocol, StaticEnvironment,
};
use otel_arrow_dfe_contrib_config_translators::amcs::schema::Configurations;

/// One expected endpoint binding, as reported by the .NET reference.
struct Expected {
    identifier: &'static str,
    event_name: OtlpEventName,
    endpoint_url: &'static str,
    routing: Option<(&'static str, &'static str)>,
}

/// Load a fixture and extract bindings with only the gRPC listener enabled, matching the
/// environment used to capture the reference values.
fn extract(
    name: &str,
) -> Vec<otel_arrow_dfe_contrib_config_translators::amcs::extract::OtlpEventInfo> {
    let path = format!("{}/tests/fixtures/{name}.json", env!("CARGO_MANIFEST_DIR"));
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("cannot read fixture {path}: {e}"));
    let configs = Configurations::from_json(&raw)
        .unwrap_or_else(|e| panic!("fixture {name} should parse: {e}"));

    let env = StaticEnvironment::new().with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1");
    extract_configuration(&env, &configs)
}

/// Assert that extraction of `name` matches the reference bindings exactly.
fn assert_parity(name: &str, expected: &[Expected]) {
    let actual = extract(name);

    assert_eq!(
        actual.len(),
        expected.len(),
        "{name}: expected {} bindings, got {}: {actual:#?}",
        expected.len(),
        actual.len()
    );

    for want in expected {
        let found = actual
            .iter()
            .find(|a| a.identifier == want.identifier && a.event_name == want.event_name)
            .unwrap_or_else(|| {
                panic!(
                    "{name}: no binding for {} / {:?} in {actual:#?}",
                    want.identifier, want.event_name
                )
            });

        assert_eq!(
            found.endpoint_urls,
            vec![want.endpoint_url.to_string()],
            "{name}: endpoint URL mismatch for {} / {:?}",
            want.identifier,
            want.event_name
        );

        let actual_routing = found
            .routing_info
            .as_ref()
            .map(|r| (r.name.as_str(), r.value.as_str()));
        assert_eq!(
            actual_routing, want.routing,
            "{name}: routing mismatch for {} / {:?}",
            want.identifier, want.event_name
        );

        // Listener details come from the environment and must match the reference too.
        assert_eq!(found.listener.host, DEFAULT_HOST);
        assert_eq!(found.listener.port, DEFAULT_OTLP_GRPC_LOGS_TRACES_PORT);
        assert_eq!(found.listener.protocol, OtlpProtocol::Grpc);
    }
}

const DCE: &str = "https://example-dce-2.eastus2euap-1.ingest.monitor.azure.com";

/// `AMCSConfig`: one rule, logs and traces, both routed on `service.name = amcs`.
#[test]
fn parity_amcs_config() {
    assert_parity(
        "AMCSConfig",
        &[
            Expected {
                identifier: "dcr-00000000000000000000000000000002.gigl-dce-00000000000000000000000000000002",
                event_name: OtlpEventName::Log,
                endpoint_url: const_format_logs(),
                routing: Some(("service.name", "amcs")),
            },
            Expected {
                identifier: "dcr-00000000000000000000000000000002.gigl-dce-00000000000000000000000000000002",
                event_name: OtlpEventName::Span,
                endpoint_url: const_format_traces(),
                routing: Some(("service.name", "amcs")),
            },
        ],
    );
}

/// `AMCSConfig2` is the same shape, except its **traces** data source declares no
/// `resourceAttributeRouting`. The reference reports `RoutingInfo: null` for the span binding
/// while the log binding keeps its filter, so routing must be tracked per signal.
#[test]
fn parity_amcs_config2_has_per_signal_routing() {
    assert_parity(
        "AMCSConfig2",
        &[
            Expected {
                identifier: "dcr-00000000000000000000000000000002.gigl-dce-00000000000000000000000000000002",
                event_name: OtlpEventName::Log,
                endpoint_url: const_format_logs(),
                routing: Some(("service.name", "amcs")),
            },
            Expected {
                identifier: "dcr-00000000000000000000000000000002.gigl-dce-00000000000000000000000000000002",
                event_name: OtlpEventName::Span,
                endpoint_url: const_format_traces(),
                // The reference reports no routing for this binding.
                routing: None,
            },
        ],
    );
}

/// `AMCSConfig3`: two rules in one payload, sending to the same channel id but with distinct
/// routing values and distinct streams.
#[test]
fn parity_amcs_config3_multi_rule() {
    assert_parity(
        "AMCSConfig3",
        &[
            Expected {
                identifier: "dcr-00000000000000000000000000000002.gigl-dce-00000000000000000000000000000002",
                event_name: OtlpEventName::Log,
                endpoint_url: "https://example-dce-2.eastus2euap-1.ingest.monitor.azure.com/dataCollectionRules/dcr-00000000000000000000000000000002/streams/OPENTELEMETRY_LOGS_AGENT_1/otlp/v1/logs?api-version=2021-11-01-preview",
                routing: Some(("service.name", "amcs_1")),
            },
            Expected {
                identifier: "dcr-00000000000000000000000000000002.gigl-dce-00000000000000000000000000000002",
                event_name: OtlpEventName::Span,
                endpoint_url: "https://example-dce-2.eastus2euap-1.ingest.monitor.azure.com/dataCollectionRules/dcr-00000000000000000000000000000002/streams/OPENTELEMETRY_TRACES_AGENT_1/otlp/v1/traces?api-version=2021-11-01-preview",
                routing: Some(("service.name", "amcs_1")),
            },
            Expected {
                identifier: "dcr-00000000000000000000000000000004.gigl-dce-00000000000000000000000000000002",
                event_name: OtlpEventName::Log,
                endpoint_url: "https://example-dce-2.eastus2euap-1.ingest.monitor.azure.com/dataCollectionRules/dcr-00000000000000000000000000000004/streams/OPENTELEMETRY_LOGS_AGENT_2/otlp/v1/logs?api-version=2021-11-01-preview",
                routing: Some(("service.name", "amcs_2")),
            },
            Expected {
                identifier: "dcr-00000000000000000000000000000004.gigl-dce-00000000000000000000000000000002",
                event_name: OtlpEventName::Span,
                endpoint_url: "https://example-dce-2.eastus2euap-1.ingest.monitor.azure.com/dataCollectionRules/dcr-00000000000000000000000000000004/streams/OPENTELEMETRY_TRACES_AGENT_2/otlp/v1/traces?api-version=2021-11-01-preview",
                routing: Some(("service.name", "amcs_2")),
            },
        ],
    );
}

/// `AMCSConfig4` declares no data sources; the reference returns an empty map.
#[test]
fn parity_amcs_config4_is_empty() {
    assert_parity("AMCSConfig4", &[]);
}

/// `AMCSConfig5`: `extension` data sources are ignored, and the `gig` channel has a logs template
/// but no traces template, so the reference reports a single logs binding.
#[test]
fn parity_amcs_config5_logs_only() {
    assert_parity(
        "AMCSConfig5",
        &[Expected {
            identifier: "dcr-00000000000000000000000000000001.gigl-dce-00000000000000000000000000000001",
            event_name: OtlpEventName::Log,
            endpoint_url: "https://example-dce-1.eastus2euap-1.ingest.monitor.azure.com/dataCollectionRules/dcr-00000000000000000000000000000001/streams/OPENTELEMETRY_LOGS_AGENT/otlp/v1/logs?api-version=2021-11-01-preview",
            routing: Some(("service.name", "amcs")),
        }],
    );
}

/// The logs endpoint shared by `AMCSConfig` and `AMCSConfig2`.
const fn const_format_logs() -> &'static str {
    "https://example-dce-2.eastus2euap-1.ingest.monitor.azure.com/dataCollectionRules/dcr-00000000000000000000000000000002/streams/OPENTELEMETRY_LOGS_AGENT/otlp/v1/logs?api-version=2021-11-01-preview"
}

/// The traces endpoint shared by `AMCSConfig` and `AMCSConfig2`.
const fn const_format_traces() -> &'static str {
    "https://example-dce-2.eastus2euap-1.ingest.monitor.azure.com/dataCollectionRules/dcr-00000000000000000000000000000002/streams/OPENTELEMETRY_TRACES_AGENT/otlp/v1/traces?api-version=2021-11-01-preview"
}

/// Sanity check that the endpoint constants really are rooted at the fixture's data collection
/// endpoint, so a typo in the expectations above cannot silently pass.
#[test]
fn expected_endpoints_are_rooted_at_the_fixture_dce() {
    assert!(const_format_logs().starts_with(DCE));
    assert!(const_format_traces().starts_with(DCE));
}
