// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Serde model for the AMCS (Azure Monitor Configuration Service) configuration payload.
//!
//! This is a direct port of `Configurations.cs` from the .NET `AMCSConfiguration` project.
//!
//! The payload delivered to the agent contains substantially more than we consume -- `eTag`,
//! `op`, `settings`, `tokenEndpointUri`, `endpoint`, `endpointUriTemplate`, per-stream
//! `solution`, and entire data source kinds such as `perfCounter`. These structs therefore
//! deliberately **do not** use `deny_unknown_fields`: unrecognised fields must be ignored rather
//! than rejected, exactly as the .NET parser does.
//!
//! Field names use AMCS's lowerCamelCase spelling verbatim so the mapping stays obvious when
//! comparing against a raw payload.

use serde::Deserialize;

/// Root of an AMCS configuration payload.
///
/// A single payload carries **every** Data Collection Rule (DCR) that applies to the host, so
/// `configurations` commonly holds more than one entry.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Configurations {
    /// One entry per DCR.
    #[serde(default)]
    pub configurations: Vec<Configuration>,
}

/// A single Data Collection Rule.
#[derive(Debug, Clone, Deserialize)]
pub struct Configuration {
    /// The DCR identifier, e.g. `dcr-00000000000000000000000000000002`.
    ///
    /// Forms the first half of the per-branch identifier (see
    /// [`OtlpEventInfo::identifier`](crate::amcs::extract::OtlpEventInfo::identifier)).
    #[serde(rename = "configurationId")]
    pub configuration_id: String,

    /// The rule body: data sources and channels.
    #[serde(default)]
    pub content: Option<Content>,
}

/// The body of a Data Collection Rule.
///
/// A rule is either a **data-source rule** (carrying `dataSources` and `channels`) or an
/// **agent settings rule** (`kind: "AgentSettings"`, carrying `settings`). The two never mix, and
/// per the OTel port configuration specification a host has at most one agent settings rule.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Content {
    /// The rule kind. `AgentSettings` marks an agent settings rule; data-source rules leave this
    /// unset.
    ///
    /// Unknown kinds must be skipped rather than rejected, so that a newer control plane can add
    /// rule kinds without breaking existing agents.
    #[serde(default)]
    pub kind: Option<String>,

    /// Agent settings key/value pairs, present only on an agent settings rule.
    ///
    /// The live AMCS payload nests these under `content.settings`. `agentSettings` is accepted as
    /// an alias because the specification writes it that way.
    #[serde(default, alias = "agentSettings")]
    pub settings: Vec<AmcsSetting>,

    /// Declared inputs. Only `otelLogs` and `otelTraces` kinds are relevant here.
    #[serde(rename = "dataSources", default)]
    pub data_sources: Vec<DataSource>,

    /// Declared destinations, referenced by [`DataSource::send_to_channels`].
    #[serde(default)]
    pub channels: Vec<Channel>,
}

/// A single agent settings key/value pair.
///
/// Values arrive as strings even when they denote numbers, matching the AMCS wire format.
#[derive(Debug, Clone, Deserialize)]
pub struct AmcsSetting {
    /// The setting name, for example `OtlpGrpcLogsTracesPort`.
    #[serde(default)]
    pub name: String,

    /// The setting value, always a string on the wire.
    #[serde(default)]
    pub value: Option<String>,
}

/// A telemetry input declared by a Data Collection Rule.
#[derive(Debug, Clone, Deserialize)]
pub struct DataSource {
    /// The data source identifier within the rule.
    #[serde(default)]
    pub id: String,

    /// The data source kind, e.g. `otelLogs`, `otelTraces`, `perfCounter`.
    ///
    /// Matched case-insensitively against
    /// [`OtlpEventName::from_amcs_kind`](crate::amcs::extract::OtlpEventName::from_amcs_kind);
    /// anything unrecognised is skipped.
    #[serde(default)]
    pub kind: String,

    /// Ids of the channels this data source sends to, resolved against [`Content::channels`].
    #[serde(rename = "sendToChannels", default)]
    pub send_to_channels: Vec<String>,

    /// The streams (schemas) this data source produces. Each stream name is substituted into the
    /// channel's endpoint template in place of the `<STREAM>` token.
    #[serde(default)]
    pub streams: Vec<AmcsStream>,

    /// Optional kind-specific configuration. Only `resourceAttributeRouting` is consumed.
    #[serde(default)]
    pub configuration: Option<DataSourceConfiguration>,
}

/// A destination declared by a Data Collection Rule.
#[derive(Debug, Clone, Deserialize)]
pub struct Channel {
    /// The channel identifier, referenced by [`DataSource::send_to_channels`].
    ///
    /// Forms the second half of the per-branch identifier.
    #[serde(default)]
    pub id: String,

    /// The channel protocol. Only `gig` channels carry OTLP endpoint templates; `ods` channels
    /// are legacy and ignored.
    #[serde(default)]
    pub protocol: Option<String>,

    /// Endpoint template for OTLP logs, containing the literal `<STREAM>` token.
    ///
    /// Optional: a customer may configure traces only, logs only, both, or neither.
    #[serde(rename = "otelLogsEndpointUriTemplate", default)]
    pub otel_logs_endpoint_uri_template: Option<String>,

    /// Endpoint template for OTLP traces, containing the literal `<STREAM>` token.
    ///
    /// Optional, for the same reason as [`Channel::otel_logs_endpoint_uri_template`].
    #[serde(rename = "otelTracesEndpointUriTemplate", default)]
    pub otel_traces_endpoint_uri_template: Option<String>,
}

/// A named stream (schema) produced by a data source.
#[derive(Debug, Clone, Deserialize)]
pub struct AmcsStream {
    /// The stream name, e.g. `OPENTELEMETRY_LOGS_AGENT`.
    #[serde(default)]
    pub stream: String,
}

/// Kind-specific data source configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct DataSourceConfiguration {
    /// How telemetry is routed to this data source.
    ///
    /// When absent, telemetry is broadcast to every endpoint rather than filtered.
    #[serde(rename = "resourceAttributeRouting", default)]
    pub resource_attribute_routing: Option<ResourceAttributeRouting>,
}

/// The resource attribute used to route telemetry to a specific data source.
#[derive(Debug, Clone, Deserialize)]
pub struct ResourceAttributeRouting {
    /// The resource attribute key, e.g. `service.name`.
    #[serde(rename = "attributeName", default)]
    pub attribute_name: Option<String>,

    /// The resource attribute value the key must equal, e.g. `amcs`.
    #[serde(rename = "attributeValue", default)]
    pub attribute_value: Option<String>,
}

/// The `content.kind` value marking an agent settings rule.
pub const AGENT_SETTINGS_KIND: &str = "AgentSettings";

impl Content {
    /// Whether this rule is an agent settings rule rather than a data-source rule.
    #[must_use]
    pub fn is_agent_settings(&self) -> bool {
        self.kind
            .as_deref()
            .is_some_and(|k| k.eq_ignore_ascii_case(AGENT_SETTINGS_KIND))
    }
}

impl Configurations {
    /// Parse an AMCS configuration payload from JSON.
    ///
    /// # Errors
    ///
    /// Returns [`Error::Deserialization`](crate::Error::Deserialization) if the payload is not
    /// valid JSON or does not match the expected shape.
    pub fn from_json(json: &str) -> Result<Self, crate::Error> {
        serde_json::from_str(json).map_err(|e| crate::Error::Deserialization {
            format: "JSON",
            details: e.to_string(),
        })
    }

    /// Look up an agent setting by name, matched case-insensitively.
    ///
    /// Returns `None` when no agent settings rule is present, or when the named setting is absent
    /// or has no value. A host has at most one agent settings rule, but if several were somehow
    /// delivered the first match wins.
    #[must_use]
    pub fn agent_setting(&self, name: &str) -> Option<&str> {
        self.configurations
            .iter()
            .filter_map(|c| c.content.as_ref())
            .filter(|content| content.is_agent_settings())
            .flat_map(|content| content.settings.iter())
            .find(|setting| setting.name.eq_ignore_ascii_case(name))
            .and_then(|setting| setting.value.as_deref())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Unknown fields such as `eTag`, `op` and `perfCounter` configuration blocks must be
    /// ignored, not rejected.
    #[test]
    fn ignores_unknown_fields() {
        let json = r#"{
            "configurations": [{
                "configurationId": "dcr-1",
                "eTag": "dcr-1/%22abc%22",
                "op": "added",
                "content": {
                    "dataSources": [{
                        "configuration": {
                            "scheduledTransferPeriod": "PT1M",
                            "counters": [{ "samplingFrequencyInSeconds": 30 }]
                        },
                        "id": "perf",
                        "kind": "perfCounter",
                        "streams": [{ "stream": "GENERIC_PERF_BLOB", "solution": "LogManagement" }],
                        "sendToChannels": ["ods-1"]
                    }],
                    "channels": [{
                        "endpoint": "https://example.ods.opinsights.azure.com",
                        "tokenEndpointUri": "https://example/issueIngestionToken",
                        "id": "ods-1",
                        "protocol": "ods"
                    }]
                }
            }]
        }"#;

        let parsed = Configurations::from_json(json).expect("payload should parse");
        assert_eq!(parsed.configurations.len(), 1);
        assert_eq!(parsed.configurations[0].configuration_id, "dcr-1");

        let content = parsed.configurations[0]
            .content
            .as_ref()
            .expect("content present");
        assert_eq!(content.data_sources.len(), 1);
        assert_eq!(content.data_sources[0].kind, "perfCounter");
        assert_eq!(content.channels[0].protocol.as_deref(), Some("ods"));
    }

    /// Both endpoint templates are independently optional.
    #[test]
    fn endpoint_templates_are_optional() {
        let json = r#"{
            "configurations": [{
                "configurationId": "dcr-1",
                "content": { "channels": [{ "id": "gig-1", "protocol": "gig" }] }
            }]
        }"#;

        let parsed = Configurations::from_json(json).expect("payload should parse");
        let channel = &parsed.configurations[0]
            .content
            .as_ref()
            .expect("content present")
            .channels[0];
        assert!(channel.otel_logs_endpoint_uri_template.is_none());
        assert!(channel.otel_traces_endpoint_uri_template.is_none());
    }

    /// An entirely empty payload is valid and yields no configurations.
    #[test]
    fn empty_payload_parses() {
        let parsed = Configurations::from_json("{}").expect("empty payload should parse");
        assert!(parsed.configurations.is_empty());
    }

    /// Malformed JSON must surface as a deserialization error rather than a panic.
    #[test]
    fn malformed_json_is_an_error() {
        let err = Configurations::from_json("{ not json").expect_err("should fail");
        assert!(matches!(err, crate::Error::Deserialization { .. }));
    }

    /// The live agent settings shape: `content.kind` plus `content.settings`.
    const AGENT_SETTINGS_PAYLOAD: &str = r#"{
        "configurations": [{
            "configurationId": "dcr-00000000000000000000000000000003",
            "eTag": "dcr-00000000000000000000000000000003/%22e3e3e3e3%22",
            "op": "added",
            "content": {
                "kind": "AgentSettings",
                "settings": [
                    { "name": "MaxDiskQuotaInMB", "value": "10240" },
                    { "name": "OtlpGrpcLogsTracesPort", "value": "4319" },
                    { "name": "OtlpHttpProtobufLogsTracesPort", "value": "4320" }
                ]
            }
        }]
    }"#;

    #[test]
    fn parses_an_agent_settings_rule() {
        let parsed = Configurations::from_json(AGENT_SETTINGS_PAYLOAD).expect("should parse");
        let content = parsed.configurations[0]
            .content
            .as_ref()
            .expect("content present");

        assert!(content.is_agent_settings());
        assert_eq!(content.settings.len(), 3);
        // An agent settings rule carries no data sources or channels.
        assert!(content.data_sources.is_empty());
        assert!(content.channels.is_empty());
    }

    #[test]
    fn looks_up_agent_settings_by_name() {
        let parsed = Configurations::from_json(AGENT_SETTINGS_PAYLOAD).expect("should parse");

        assert_eq!(parsed.agent_setting("OtlpGrpcLogsTracesPort"), Some("4319"));
        assert_eq!(
            parsed.agent_setting("OtlpHttpProtobufLogsTracesPort"),
            Some("4320")
        );
        assert_eq!(parsed.agent_setting("MaxDiskQuotaInMB"), Some("10240"));
        assert_eq!(parsed.agent_setting("NoSuchSetting"), None);
    }

    /// Setting names are matched case-insensitively, as elsewhere in the payload.
    #[test]
    fn agent_setting_lookup_is_case_insensitive() {
        let parsed = Configurations::from_json(AGENT_SETTINGS_PAYLOAD).expect("should parse");
        assert_eq!(parsed.agent_setting("otlpgrpclogstracesport"), Some("4319"));
    }

    /// A data-source rule is not an agent settings rule, and exposes no settings.
    #[test]
    fn data_source_rule_is_not_agent_settings() {
        let json = r#"{
            "configurations": [{
                "configurationId": "dcr-1",
                "content": {
                    "dataSources": [{ "id": "logs", "kind": "otelLogs" }],
                    "channels": [{ "id": "gig-1", "protocol": "gig" }]
                }
            }]
        }"#;

        let parsed = Configurations::from_json(json).expect("should parse");
        let content = parsed.configurations[0]
            .content
            .as_ref()
            .expect("content present");

        assert!(!content.is_agent_settings());
        assert_eq!(parsed.agent_setting("OtlpGrpcLogsTracesPort"), None);
    }

    /// The specification writes the settings array as `agentSettings`; accept it as an alias.
    #[test]
    fn accepts_the_agent_settings_alias() {
        let json = r#"{
            "configurations": [{
                "configurationId": "dcr-1",
                "content": {
                    "kind": "AgentSettings",
                    "agentSettings": [
                        { "name": "OtlpGrpcLogsTracesPort", "value": "4329" }
                    ]
                }
            }]
        }"#;

        let parsed = Configurations::from_json(json).expect("should parse");
        assert_eq!(parsed.agent_setting("OtlpGrpcLogsTracesPort"), Some("4329"));
    }

    /// An unrecognised rule kind must be skipped, not rejected, so a newer control plane can add
    /// kinds without breaking existing agents.
    #[test]
    fn unknown_rule_kind_is_ignored() {
        let json = r#"{
            "configurations": [{
                "configurationId": "dcr-1",
                "content": { "kind": "SomeFutureKind", "settings": [{ "name": "X", "value": "1" }] }
            }]
        }"#;

        let parsed = Configurations::from_json(json).expect("should parse");
        let content = parsed.configurations[0]
            .content
            .as_ref()
            .expect("content present");

        assert!(!content.is_agent_settings());
        assert_eq!(parsed.agent_setting("X"), None);
    }

    /// A mixed payload carries both an agent settings rule and data-source rules.
    #[test]
    fn mixed_payload_exposes_settings_and_data_sources() {
        let json = r#"{
            "configurations": [
                {
                    "configurationId": "dcr-settings",
                    "content": {
                        "kind": "AgentSettings",
                        "settings": [{ "name": "OtlpGrpcLogsTracesPort", "value": "4329" }]
                    }
                },
                {
                    "configurationId": "dcr-data",
                    "content": {
                        "dataSources": [{ "id": "logs", "kind": "otelLogs" }],
                        "channels": [{ "id": "gig-1", "protocol": "gig" }]
                    }
                }
            ]
        }"#;

        let parsed = Configurations::from_json(json).expect("should parse");
        assert_eq!(parsed.agent_setting("OtlpGrpcLogsTracesPort"), Some("4329"));

        let data_rule = parsed.configurations[1]
            .content
            .as_ref()
            .expect("content present");
        assert!(!data_rule.is_agent_settings());
        assert_eq!(data_rule.data_sources.len(), 1);
    }
}
