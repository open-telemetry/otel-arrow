// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Extraction of routable OTLP endpoints from an AMCS configuration payload.
//!
//! Port of `AMCSParser.ExtractConfiguration` from the .NET `AMCSConfiguration` project.
//!
//! The algorithm, per Data Collection Rule:
//!
//! 1. Map `dataSource.kind` to an [`OtlpEventName`]; skip unrecognised kinds such as
//!    `perfCounter`.
//! 2. Read the optional `resourceAttributeRouting`. When absent, telemetry is broadcast to every
//!    endpoint instead of filtered, and a warning is emitted.
//! 3. Collect the data source's stream names.
//! 4. Resolve `sendToChannels` against the rule's channels; only `gig` channels carry OTLP
//!    endpoint templates.
//! 5. Build the identifier `{configurationId}.{channelId}` and substitute each stream name into
//!    the endpoint template in place of the `<STREAM>` token.
//! 6. Emit one [`OtlpEventInfo`] per (channel x listener) pair.

use crate::amcs::listener::{AgentSettings, OtlpEventListenerInfo, discover_listeners};
use crate::amcs::schema::{Channel, Configurations, DataSource};
use otel_arrow_dfe_telemetry::otel_warn;
use std::collections::BTreeSet;

/// The literal placeholder substituted with a stream name inside endpoint templates.
pub const URL_STREAM_REPLACEMENT_VAL: &str = "<STREAM>";

/// The channel protocol that carries OTLP endpoint templates. `ods` channels are legacy and
/// carry no OTLP endpoints, so they are ignored.
pub const GIG_PROTOCOL: &str = "gig";

/// The AMCS data source kind for OTLP logs.
pub const AMCS_KIND_OTEL_LOGS: &str = "otelLogs";

/// The AMCS data source kind for OTLP traces.
pub const AMCS_KIND_OTEL_TRACES: &str = "otelTraces";

/// The OTLP signal a data source carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum OtlpEventName {
    /// OTLP logs, from an `otelLogs` data source.
    Log,
    /// OTLP spans, from an `otelTraces` data source.
    Span,
}

impl OtlpEventName {
    /// Map an AMCS `kind` to a signal, case-insensitively.
    ///
    /// Returns `None` for every other kind (`perfCounter`, `syslog`, ...), which is how
    /// non-OTLP data sources are skipped.
    #[must_use]
    pub fn from_amcs_kind(kind: &str) -> Option<Self> {
        if kind.eq_ignore_ascii_case(AMCS_KIND_OTEL_LOGS) {
            Some(Self::Log)
        } else if kind.eq_ignore_ascii_case(AMCS_KIND_OTEL_TRACES) {
            Some(Self::Span)
        } else {
            None
        }
    }

    /// A stable lowercase name, used in diagnostics and generated node names.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Log => "logs",
            Self::Span => "traces",
        }
    }
}

/// The resource attribute a branch routes on.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtlpAttributeRouting {
    /// The resource attribute key, e.g. `service.name`.
    pub name: String,
    /// The value the key must equal, e.g. `amcs`.
    pub value: String,
}

/// One routable OTLP endpoint binding: which listener receives the data, which rule and channel
/// it belongs to, where it is forwarded, and how it is selected.
///
/// Equivalent to `OtlpEventInfo` in the .NET implementation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OtlpEventInfo {
    /// The listener that accepts this telemetry.
    pub listener: OtlpEventListenerInfo,

    /// `{configurationId}.{channelId}` -- unique per rule and destination.
    pub identifier: String,

    /// Fully-resolved destination URLs, with `<STREAM>` substituted.
    pub endpoint_urls: Vec<String>,

    /// The signal this binding carries.
    pub event_name: OtlpEventName,

    /// The routing filter, or `None` when telemetry should be broadcast to every endpoint.
    pub routing_info: Option<OtlpAttributeRouting>,
}

/// Extract every routable OTLP endpoint from an AMCS payload.
///
/// A payload carries all Data Collection Rules that apply to the host, so this iterates every
/// rule, every data source, and every channel in a single pass. Listener ports are resolved from
/// the environment and from the payload's agent settings rule, in that order of precedence.
///
/// Returns an empty vector when OTLP ingestion is disabled, when no rule declares an OTLP data
/// source, or when no data source resolves to a usable endpoint. Per
/// `Telemetry-Collection-Spec/AMACoreAgent/otel-port-configuration.md`, an agent settings rule on
/// its own is **not** sufficient to open a port: an OTel data-source rule must also be present,
/// which an empty result correctly expresses.
#[must_use]
pub fn extract_configuration<E: crate::amcs::listener::EnvironmentProvider + ?Sized>(
    env: &E,
    config: &Configurations,
) -> Vec<OtlpEventInfo> {
    let settings = AgentSettings::from_configurations(config);
    let listeners = discover_listeners(env, &settings);
    if listeners.is_empty() {
        otel_warn!(
            "amcs.extract.no_listeners",
            message = "no OTLP listeners are enabled; the generated pipeline will be empty"
        );
        return Vec::new();
    }

    let mut result = Vec::new();

    for configuration in &config.configurations {
        let Some(content) = configuration.content.as_ref() else {
            continue;
        };

        // An agent settings rule carries listener configuration, not telemetry routing. It has
        // already been consumed above, and must never become a pipeline branch.
        if content.is_agent_settings() {
            continue;
        }

        for data_source in &content.data_sources {
            // Non-OTLP kinds (perfCounter, etc.) are skipped outright.
            let Some(event_name) = OtlpEventName::from_amcs_kind(&data_source.kind) else {
                continue;
            };

            let routing_info =
                resolve_routing(data_source, &configuration.configuration_id, event_name);

            // A BTreeSet both de-duplicates (as the .NET HashSet does) and gives a stable
            // ordering, so generated output is deterministic across runs.
            let streams: BTreeSet<&str> = data_source
                .streams
                .iter()
                .map(|s| s.stream.as_str())
                .filter(|s| !s.is_empty())
                .collect();

            if streams.is_empty() {
                otel_warn!(
                    "amcs.extract.no_streams",
                    configuration_id = configuration.configuration_id.as_str(),
                    kind = data_source.kind.as_str(),
                    message = "data source declares no streams; no endpoint can be built for it"
                );
                continue;
            }

            for channel_id in &data_source.send_to_channels {
                let Some(channel) = content.channels.iter().find(|c| c.id == *channel_id) else {
                    otel_warn!(
                        "amcs.extract.channel_not_found",
                        configuration_id = configuration.configuration_id.as_str(),
                        channel_id = channel_id.as_str(),
                        message = "data source references a channel that the rule does not declare"
                    );
                    continue;
                };

                // Only `gig` channels carry OTLP endpoint templates; `ods` channels are legacy.
                if !is_gig_channel(channel) {
                    continue;
                }

                let Some(template) = endpoint_template(channel, event_name) else {
                    // Entirely expected: a customer may configure logs only, or traces only.
                    continue;
                };

                let endpoint_urls: Vec<String> = streams
                    .iter()
                    .map(|stream| template.replace(URL_STREAM_REPLACEMENT_VAL, stream))
                    .collect();

                let identifier = format!("{}.{}", configuration.configuration_id, channel_id);

                for listener in &listeners {
                    result.push(OtlpEventInfo {
                        listener: listener.clone(),
                        identifier: identifier.clone(),
                        endpoint_urls: endpoint_urls.clone(),
                        event_name,
                        routing_info: routing_info.clone(),
                    });
                }
            }
        }
    }

    result
}

/// Read the optional routing filter, warning when it is absent.
///
/// A missing filter is legal but unusual: it means telemetry is broadcast to every endpoint
/// rather than routed to this one, so it is worth surfacing.
fn resolve_routing(
    data_source: &DataSource,
    configuration_id: &str,
    event_name: OtlpEventName,
) -> Option<OtlpAttributeRouting> {
    let routing = data_source
        .configuration
        .as_ref()
        .and_then(|c| c.resource_attribute_routing.as_ref());

    let name = routing
        .and_then(|r| r.attribute_name.as_deref())
        .filter(|v| !v.is_empty());
    let value = routing
        .and_then(|r| r.attribute_value.as_deref())
        .filter(|v| !v.is_empty());

    match (name, value) {
        (Some(name), Some(value)) => Some(OtlpAttributeRouting {
            name: name.to_string(),
            value: value.to_string(),
        }),
        _ => {
            otel_warn!(
                "amcs.extract.missing_attribute_routing",
                configuration_id = configuration_id,
                signal = event_name.as_str(),
                message = "data collection rule has no resource attribute routing; telemetry will be broadcast to all endpoints"
            );
            None
        }
    }
}

/// Whether a channel is a `gig` channel, and therefore carries OTLP endpoint templates.
fn is_gig_channel(channel: &Channel) -> bool {
    channel
        .protocol
        .as_deref()
        .is_some_and(|p| p.eq_ignore_ascii_case(GIG_PROTOCOL))
}

/// Select the endpoint template matching the signal.
///
/// This deliberately diverges from the .NET implementation. `AMCSParser.cs` uses:
///
/// ```csharp
/// if (eventName == OtlpEventName.Log && otelLogsEndpointUriTemplate != null) { ...logs... }
/// else if (otelTracesEndpointUriTemplate != null)                            { ...traces... }
/// ```
///
/// When the signal is `Log` but `otelLogsEndpointUriTemplate` is null, control falls into the
/// `else if` and a **traces** URL is emitted for a **logs** data source. Because both templates
/// are independently optional, that path is reachable with real customer configuration and would
/// silently misroute logs into the traces endpoint.
///
/// Here each signal only ever selects its own template, and a data source whose template is
/// absent produces no endpoint at all.
fn endpoint_template(channel: &Channel, event_name: OtlpEventName) -> Option<&str> {
    match event_name {
        OtlpEventName::Log => channel.otel_logs_endpoint_uri_template.as_deref(),
        OtlpEventName::Span => channel.otel_traces_endpoint_uri_template.as_deref(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::amcs::listener::{
        ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, OtlpProtocol, StaticEnvironment,
    };

    /// An environment with a single gRPC listener, so each binding appears exactly once.
    fn single_listener_env() -> StaticEnvironment {
        StaticEnvironment::new().with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1")
    }

    fn parse(json: &str) -> Configurations {
        Configurations::from_json(json).expect("fixture should parse")
    }

    /// The canonical single-rule payload: one logs and one traces data source, both routed on
    /// `service.name = amcs`, plus a `perfCounter` data source that must be ignored.
    fn canonical_payload() -> &'static str {
        r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [
                {
                  "id": "perf",
                  "kind": "perfCounter",
                  "streams": [{ "stream": "GENERIC_PERF_BLOB" }],
                  "sendToChannels": ["ods-1"]
                },
                {
                  "configuration": {
                    "resourceAttributeRouting": {
                      "attributeName": "service.name",
                      "attributeValue": "amcs"
                    }
                  },
                  "id": "logs",
                  "kind": "otelLogs",
                  "streams": [{ "stream": "OPENTELEMETRY_LOGS_AGENT" }],
                  "sendToChannels": ["gig-1"]
                },
                {
                  "configuration": {
                    "resourceAttributeRouting": {
                      "attributeName": "service.name",
                      "attributeValue": "amcs"
                    }
                  },
                  "id": "traces",
                  "kind": "otelTraces",
                  "streams": [{ "stream": "OPENTELEMETRY_TRACES_AGENT" }],
                  "sendToChannels": ["gig-1"]
                }
              ],
              "channels": [
                { "id": "ods-1", "protocol": "ods" },
                {
                  "id": "gig-1",
                  "protocol": "gig",
                  "otelLogsEndpointUriTemplate": "https://dce.example.com/dataCollectionRules/dcr-1/streams/<STREAM>/otlp/v1/logs?api-version=2021-11-01-preview",
                  "otelTracesEndpointUriTemplate": "https://dce.example.com/dataCollectionRules/dcr-1/streams/<STREAM>/otlp/v1/traces?api-version=2021-11-01-preview"
                }
              ]
            }
          }]
        }"#
    }

    #[test]
    fn extracts_logs_and_traces_and_skips_perf_counter() {
        let infos = extract_configuration(&single_listener_env(), &parse(canonical_payload()));

        assert_eq!(infos.len(), 2, "expected one logs and one traces binding");

        let logs = infos
            .iter()
            .find(|i| i.event_name == OtlpEventName::Log)
            .expect("logs binding present");
        assert_eq!(logs.identifier, "dcr-1.gig-1");
        assert_eq!(
            logs.endpoint_urls,
            vec![
                "https://dce.example.com/dataCollectionRules/dcr-1/streams/OPENTELEMETRY_LOGS_AGENT/otlp/v1/logs?api-version=2021-11-01-preview"
            ]
        );
        assert_eq!(
            logs.routing_info,
            Some(OtlpAttributeRouting {
                name: "service.name".to_string(),
                value: "amcs".to_string(),
            })
        );
        assert_eq!(logs.listener.protocol, OtlpProtocol::Grpc);

        let traces = infos
            .iter()
            .find(|i| i.event_name == OtlpEventName::Span)
            .expect("traces binding present");
        assert_eq!(
            traces.endpoint_urls,
            vec![
                "https://dce.example.com/dataCollectionRules/dcr-1/streams/OPENTELEMETRY_TRACES_AGENT/otlp/v1/traces?api-version=2021-11-01-preview"
            ]
        );
    }

    /// No `<STREAM>` token may survive into a resolved endpoint URL.
    #[test]
    fn stream_placeholder_is_always_substituted() {
        let infos = extract_configuration(&single_listener_env(), &parse(canonical_payload()));

        assert!(!infos.is_empty());
        for info in &infos {
            for url in &info.endpoint_urls {
                assert!(
                    !url.contains(URL_STREAM_REPLACEMENT_VAL),
                    "unsubstituted placeholder in {url}"
                );
            }
        }
    }

    /// Each enabled listener produces its own binding for the same channel.
    #[test]
    fn each_listener_produces_a_binding() {
        // Default environment enables both gRPC and HTTP.
        let infos = extract_configuration(&StaticEnvironment::new(), &parse(canonical_payload()));

        assert_eq!(infos.len(), 4, "2 signals x 2 listeners");
        assert_eq!(
            infos
                .iter()
                .filter(|i| i.listener.protocol == OtlpProtocol::Grpc)
                .count(),
            2
        );
        assert_eq!(
            infos
                .iter()
                .filter(|i| i.listener.protocol == OtlpProtocol::HttpProtobuf)
                .count(),
            2
        );
    }

    /// Ragu's multi-DCR case: several rules arrive in one payload and all must be processed.
    #[test]
    fn handles_multiple_data_collection_rules() {
        let json = r#"{
          "configurations": [
            {
              "configurationId": "dcr-a",
              "content": {
                "dataSources": [{
                  "configuration": { "resourceAttributeRouting": { "attributeName": "service.name", "attributeValue": "a" } },
                  "id": "logs", "kind": "otelLogs",
                  "streams": [{ "stream": "S_A" }],
                  "sendToChannels": ["gig-a"]
                }],
                "channels": [{ "id": "gig-a", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://a/<STREAM>/logs" }]
              }
            },
            {
              "configurationId": "dcr-b",
              "content": {
                "dataSources": [{
                  "configuration": { "resourceAttributeRouting": { "attributeName": "service.name", "attributeValue": "b" } },
                  "id": "logs", "kind": "otelLogs",
                  "streams": [{ "stream": "S_B" }],
                  "sendToChannels": ["gig-b"]
                }],
                "channels": [{ "id": "gig-b", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://b/<STREAM>/logs" }]
              }
            }
          ]
        }"#;

        let infos = extract_configuration(&single_listener_env(), &parse(json));

        assert_eq!(infos.len(), 2);
        let mut identifiers: Vec<&str> = infos.iter().map(|i| i.identifier.as_str()).collect();
        identifiers.sort_unstable();
        assert_eq!(identifiers, vec!["dcr-a.gig-a", "dcr-b.gig-b"]);
    }

    /// A rule with only a traces endpoint yields only a traces binding.
    #[test]
    fn traces_only_configuration() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "configuration": { "resourceAttributeRouting": { "attributeName": "service.name", "attributeValue": "amcs" } },
                "id": "traces", "kind": "otelTraces",
                "streams": [{ "stream": "T" }],
                "sendToChannels": ["gig-1"]
              }],
              "channels": [{ "id": "gig-1", "protocol": "gig", "otelTracesEndpointUriTemplate": "https://x/<STREAM>/traces" }]
            }
          }]
        }"#;

        let infos = extract_configuration(&single_listener_env(), &parse(json));

        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].event_name, OtlpEventName::Span);
        assert_eq!(infos[0].endpoint_urls, vec!["https://x/T/traces"]);
    }

    /// Regression guard for the .NET fall-through bug: a logs data source whose channel has only
    /// a **traces** template must produce nothing, never a traces URL labelled as logs.
    #[test]
    fn logs_source_without_logs_template_emits_nothing() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "configuration": { "resourceAttributeRouting": { "attributeName": "service.name", "attributeValue": "amcs" } },
                "id": "logs", "kind": "otelLogs",
                "streams": [{ "stream": "S" }],
                "sendToChannels": ["gig-1"]
              }],
              "channels": [{ "id": "gig-1", "protocol": "gig", "otelTracesEndpointUriTemplate": "https://x/<STREAM>/traces" }]
            }
          }]
        }"#;

        let infos = extract_configuration(&single_listener_env(), &parse(json));

        assert!(
            infos.is_empty(),
            "a logs data source must never fall through to the traces endpoint, got {infos:?}"
        );
    }

    /// A channel with neither template yields nothing.
    #[test]
    fn channel_without_any_template_emits_nothing() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "id": "logs", "kind": "otelLogs",
                "streams": [{ "stream": "S" }],
                "sendToChannels": ["gig-1"]
              }],
              "channels": [{ "id": "gig-1", "protocol": "gig" }]
            }
          }]
        }"#;

        assert!(extract_configuration(&single_listener_env(), &parse(json)).is_empty());
    }

    /// Missing routing is legal: the binding is produced with `routing_info: None`, meaning
    /// broadcast.
    #[test]
    fn missing_attribute_routing_yields_broadcast_binding() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "id": "logs", "kind": "otelLogs",
                "streams": [{ "stream": "S" }],
                "sendToChannels": ["gig-1"]
              }],
              "channels": [{ "id": "gig-1", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://x/<STREAM>/logs" }]
            }
          }]
        }"#;

        let infos = extract_configuration(&single_listener_env(), &parse(json));

        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].routing_info, None);
    }

    /// A partially-specified routing block (value missing) is treated as absent.
    #[test]
    fn partial_attribute_routing_is_treated_as_absent() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "configuration": { "resourceAttributeRouting": { "attributeName": "service.name" } },
                "id": "logs", "kind": "otelLogs",
                "streams": [{ "stream": "S" }],
                "sendToChannels": ["gig-1"]
              }],
              "channels": [{ "id": "gig-1", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://x/<STREAM>/logs" }]
            }
          }]
        }"#;

        let infos = extract_configuration(&single_listener_env(), &parse(json));

        assert_eq!(infos.len(), 1);
        assert_eq!(infos[0].routing_info, None);
    }

    /// `ods` channels carry no OTLP endpoints and must be ignored.
    #[test]
    fn ods_channels_are_ignored() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "id": "logs", "kind": "otelLogs",
                "streams": [{ "stream": "S" }],
                "sendToChannels": ["ods-1"]
              }],
              "channels": [{ "id": "ods-1", "protocol": "ods", "otelLogsEndpointUriTemplate": "https://x/<STREAM>/logs" }]
            }
          }]
        }"#;

        assert!(extract_configuration(&single_listener_env(), &parse(json)).is_empty());
    }

    /// An unresolvable `sendToChannels` reference is skipped rather than fatal.
    #[test]
    fn unresolvable_channel_reference_is_skipped() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "id": "logs", "kind": "otelLogs",
                "streams": [{ "stream": "S" }],
                "sendToChannels": ["does-not-exist"]
              }],
              "channels": [{ "id": "gig-1", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://x/<STREAM>/logs" }]
            }
          }]
        }"#;

        assert!(extract_configuration(&single_listener_env(), &parse(json)).is_empty());
    }

    /// A data source fanning out to several channels yields one binding per channel.
    #[test]
    fn multiple_channels_per_data_source() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "configuration": { "resourceAttributeRouting": { "attributeName": "service.name", "attributeValue": "amcs" } },
                "id": "logs", "kind": "otelLogs",
                "streams": [{ "stream": "S" }],
                "sendToChannels": ["gig-1", "gig-2"]
              }],
              "channels": [
                { "id": "gig-1", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://one/<STREAM>/logs" },
                { "id": "gig-2", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://two/<STREAM>/logs" }
              ]
            }
          }]
        }"#;

        let infos = extract_configuration(&single_listener_env(), &parse(json));

        assert_eq!(infos.len(), 2);
        let mut identifiers: Vec<&str> = infos.iter().map(|i| i.identifier.as_str()).collect();
        identifiers.sort_unstable();
        assert_eq!(identifiers, vec!["dcr-1.gig-1", "dcr-1.gig-2"]);
    }

    /// Several streams on one data source produce several endpoint URLs, in stable order.
    #[test]
    fn multiple_streams_produce_multiple_urls() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "id": "logs", "kind": "otelLogs",
                "streams": [{ "stream": "B_STREAM" }, { "stream": "A_STREAM" }],
                "sendToChannels": ["gig-1"]
              }],
              "channels": [{ "id": "gig-1", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://x/<STREAM>/logs" }]
            }
          }]
        }"#;

        let infos = extract_configuration(&single_listener_env(), &parse(json));

        assert_eq!(infos.len(), 1);
        assert_eq!(
            infos[0].endpoint_urls,
            vec!["https://x/A_STREAM/logs", "https://x/B_STREAM/logs"],
            "stream order must be deterministic"
        );
    }

    /// The `kind` match is case-insensitive, as in the .NET frozen dictionary.
    #[test]
    fn kind_matching_is_case_insensitive() {
        assert_eq!(
            OtlpEventName::from_amcs_kind("OTELLOGS"),
            Some(OtlpEventName::Log)
        );
        assert_eq!(
            OtlpEventName::from_amcs_kind("otelTRACES"),
            Some(OtlpEventName::Span)
        );
        assert_eq!(OtlpEventName::from_amcs_kind("perfCounter"), None);
        assert_eq!(OtlpEventName::from_amcs_kind(""), None);
    }

    /// With every listener disabled there is nothing to build.
    #[test]
    fn no_listeners_yields_no_bindings() {
        let env = StaticEnvironment::new()
            .with(crate::amcs::listener::ENV_OTLP_GRPC_LOGS_TRACES_PORT, "-1")
            .with(ENV_OTLP_HTTP_PROTOBUF_LOGS_TRACES_PORT, "-1");

        assert!(extract_configuration(&env, &parse(canonical_payload())).is_empty());
    }

    /// A data source with no streams cannot produce an endpoint.
    #[test]
    fn data_source_without_streams_emits_nothing() {
        let json = r#"{
          "configurations": [{
            "configurationId": "dcr-1",
            "content": {
              "dataSources": [{
                "id": "logs", "kind": "otelLogs",
                "streams": [],
                "sendToChannels": ["gig-1"]
              }],
              "channels": [{ "id": "gig-1", "protocol": "gig", "otelLogsEndpointUriTemplate": "https://x/<STREAM>/logs" }]
            }
          }]
        }"#;

        assert!(extract_configuration(&single_listener_env(), &parse(json)).is_empty());
    }
}
