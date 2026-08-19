// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Geneva Exporter for OTAP logs and traces
//!
//! This exporter sends OTAP log and trace data to Microsoft Geneva telemetry backend.
//! It is designed for Microsoft products and implements the `Exporter<OtapPdata>` trait
//! for integration with the OTAP dataflow engine.
//!
//! ## Usage
//!
//! This exporter is automatically discovered by the `df_engine` binary via `linkme`.
//! Users configure it in YAML:
//!
//! ```yaml
//! nodes:
//!   - id: geneva-exporter
//!     urn: "urn:microsoft:exporter:geneva"
//!     config:
//!       endpoint: "https://geneva.microsoft.com"
//!       environment: "production"
//!       account: "my-account"
//!       namespace: "my-namespace"
//!       account_routing:
//!         default_group: "my-account-group"
//!       # ... additional config
//! ```

otap_df_telemetry::otel_component_scope!(
    urn = GENEVA_EXPORTER_URN,
    target = "microsoft.exporter.geneva",
);

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use otap_df_config::node::NodeUserConfig;
use otap_df_engine::ConsumerEffectHandlerExtension;
use otap_df_engine::ExporterFactory;
use otap_df_engine::config::ExporterConfig;
use otap_df_engine::context::PipelineContext;
use otap_df_engine::control::NodeControlMsg;
use otap_df_engine::control::{AckMsg, NackMsg};
use otap_df_engine::error::Error;
use otap_df_engine::exporter::ExporterWrapper;
use otap_df_engine::local::exporter::{EffectHandler, Exporter};
use otap_df_engine::message::{ExporterInbox, Message};
use otap_df_engine::node::NodeId;
use otap_df_engine::terminal_state::TerminalState;
use otap_df_pdata::TryIntoWithOptions;
use otap_df_pdata::otlp::OtlpProtoBytes;
use otap_df_pdata::views::otap::OtapLogsView;
use otap_df_pdata::views::otlp::bytes::logs::RawLogsData;
use otap_df_pdata::{OtapArrowRecords, OtapPayload};
use otap_df_telemetry::instrument::{Counter, Mmsc};
use otap_df_telemetry::metrics::{MeasurementMetricSet, MetricSet};
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

// Geneva uploader dependencies
use futures::StreamExt;
use geneva_uploader::AuthMethod;
use geneva_uploader::client::{
    AccountRouting, EncodedBatch, GenevaClient, GenevaClientConfig, OboEventConfig, OboEventMap,
};
use geneva_uploader::{
    LogsEventNameMapping, LogsEventNameRoutingKey, SpanEventNameMapping, SpanEventNameRoutingKey,
};
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message as ProstMessage;

// Use crate-relative paths since we're now a module within otap
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::metrics::ExporterExportMetrics;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};

mod agent_fed_source;

use agent_fed_source::AgentFedGenevaSource;
use otap_df_engine::capability::ExtensionCapability;
use otap_df_engine::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider as AgentFedCredentialProviderCap;
use otap_df_engine::capability::registry::Capabilities;

/// The URN for the Geneva exporter
pub const GENEVA_EXPORTER_URN: &str = "urn:microsoft:exporter:geneva";

/// Deserializable wrapper for LogsEventNameRoutingKey
#[derive(Debug, Clone)]
pub enum LogsEventNameRoutingKeyConfig {
    /// Route by event name
    EventName,
    /// Route by resource attribute
    ResourceAttribute {
        /// The resource attribute key to route on
        resource_attribute: String,
    },
    /// Route by scope attribute
    ScopeAttribute {
        /// The scope attribute key to route on
        scope_attribute: String,
    },
    /// Route by log record attribute
    LogRecordAttribute {
        /// The log record attribute key to route on
        log_record_attribute: String,
    },
}

impl<'de> Deserialize<'de> for LogsEventNameRoutingKeyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        use serde::de::{self, MapAccess, Visitor};

        struct RoutingKeyVisitor;

        impl<'de> Visitor<'de> for RoutingKeyVisitor {
            type Value = LogsEventNameRoutingKeyConfig;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str(
                    "either the string 'event_name' or a map with exactly one of: \
                     'resource_attribute', 'scope_attribute', or \
                     'log_record_attribute'",
                )
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                match value {
                    "event_name" => Ok(LogsEventNameRoutingKeyConfig::EventName),
                    _ => Err(de::Error::unknown_variant(
                        value,
                        &[
                            "event_name",
                            "resource_attribute",
                            "scope_attribute",
                            "log_record_attribute",
                        ],
                    )),
                }
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut found_key: Option<String> = None;
                let mut routing_value: Option<String> = None;

                while let Some(key) = map.next_key::<String>()? {
                    if found_key.is_some() {
                        return Err(de::Error::custom(
                            "routing_key must have exactly one field, found multiple fields",
                        ));
                    }

                    match key.as_str() {
                        "event_name" => {
                            // `event_name` carries no routing value: the value
                            // comes from the log record's built-in event name.
                            // Reject the map form (e.g. `event_name: MyTable`)
                            // so a supplied value can't be silently ignored;
                            // the string form `routing_key: event_name` is the
                            // only supported way to select this variant.
                            return Err(de::Error::custom(
                                "'event_name' does not take a value; use the string form \
                                 `routing_key: event_name` instead of a map",
                            ));
                        }
                        "resource_attribute" => {
                            routing_value = Some(map.next_value::<String>()?);
                            found_key = Some("resource_attribute".to_string());
                        }
                        "scope_attribute" => {
                            routing_value = Some(map.next_value::<String>()?);
                            found_key = Some("scope_attribute".to_string());
                        }
                        "log_record_attribute" => {
                            routing_value = Some(map.next_value::<String>()?);
                            found_key = Some("log_record_attribute".to_string());
                        }
                        other => {
                            return Err(de::Error::unknown_field(
                                other,
                                &[
                                    "event_name",
                                    "resource_attribute",
                                    "scope_attribute",
                                    "log_record_attribute",
                                ],
                            ));
                        }
                    }
                }

                let non_empty = |field: &str, value: Option<String>| -> Result<String, M::Error> {
                    let value = value.unwrap_or_default();
                    if value.trim().is_empty() {
                        return Err(de::Error::custom(format!(
                            "'{field}' must be a non-empty attribute name"
                        )));
                    }
                    Ok(value)
                };

                match found_key.as_deref() {
                    Some("resource_attribute") => {
                        Ok(LogsEventNameRoutingKeyConfig::ResourceAttribute {
                            resource_attribute: non_empty("resource_attribute", routing_value)?,
                        })
                    }
                    Some("scope_attribute") => Ok(LogsEventNameRoutingKeyConfig::ScopeAttribute {
                        scope_attribute: non_empty("scope_attribute", routing_value)?,
                    }),
                    Some("log_record_attribute") => {
                        Ok(LogsEventNameRoutingKeyConfig::LogRecordAttribute {
                            log_record_attribute: non_empty("log_record_attribute", routing_value)?,
                        })
                    }
                    _ => Err(de::Error::custom(
                        "routing_key map must have one of: 'resource_attribute', 'scope_attribute', or 'log_record_attribute' (use the string form `routing_key: event_name` to route by event name)",
                    )),
                }
            }
        }

        deserializer.deserialize_any(RoutingKeyVisitor)
    }
}

impl From<LogsEventNameRoutingKeyConfig> for LogsEventNameRoutingKey {
    fn from(config: LogsEventNameRoutingKeyConfig) -> Self {
        match config {
            LogsEventNameRoutingKeyConfig::EventName => LogsEventNameRoutingKey::EventName,
            LogsEventNameRoutingKeyConfig::ResourceAttribute { resource_attribute } => {
                LogsEventNameRoutingKey::ResourceAttribute(resource_attribute)
            }
            LogsEventNameRoutingKeyConfig::ScopeAttribute { scope_attribute } => {
                LogsEventNameRoutingKey::ScopeAttribute(scope_attribute)
            }
            LogsEventNameRoutingKeyConfig::LogRecordAttribute {
                log_record_attribute,
            } => LogsEventNameRoutingKey::LogRecordAttribute(log_record_attribute),
        }
    }
}

/// Deserialize an optional event/table name, rejecting blank or whitespace-only
/// values. A missing field yields `None`; an explicit empty/whitespace string is
/// an error so it cannot silently override the uploader's default table name.
fn deserialize_optional_non_empty_event_name<'de, D>(
    deserializer: D,
) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value: Option<String> = Option::deserialize(deserializer)?;
    if let Some(ref name) = value {
        if name.trim().is_empty() {
            return Err(serde::de::Error::custom(
                "'default_event_name' must be a non-empty table name",
            ));
        }
    }
    Ok(value)
}

/// Shared validation for logs/spans routing `events` maps. Rejects mappings
/// that can never route or that carry silently-ignored values: an empty
/// `events` map, blank/whitespace source keys, or empty/whitespace destination
/// values. A `null` destination is valid and means "route to the source value
/// unchanged". Running this during deserialization keeps `--validate` and
/// pipeline startup in agreement so a config accepted by `--validate` cannot
/// fail later inside `GenevaClient::new()`.
fn validate_events_map(
    signal: &str,
    events: &std::collections::HashMap<String, Option<String>>,
) -> Result<(), String> {
    if events.is_empty() {
        return Err(format!(
            "{signal}.event_name_mapping.events must be non-empty when routing is configured"
        ));
    }
    for (source, destination) in events {
        if source.trim().is_empty() {
            return Err(format!(
                "{signal}.event_name_mapping.events source keys must not be blank"
            ));
        }
        if let Some(dest) = destination {
            if dest.trim().is_empty() {
                return Err(format!(
                    "{signal}.event_name_mapping.events destination for source '{source}' must \
                     not be empty or whitespace; omit the value (use null) to route to the \
                     source value unchanged"
                ));
            }
        }
    }
    Ok(())
}

/// Collect the destination event/table names a signal's routing can statically
/// produce into `known`: every non-null mapping value, every source key whose
/// mapping value is null (passthrough routes to the source value unchanged), and
/// the explicit `default_event_name` when set. Used to flag OBO entries that can
/// never match a reachable destination.
fn collect_known_destinations(
    default_event_name: Option<&str>,
    events: Option<&std::collections::HashMap<String, Option<String>>>,
    known: &mut std::collections::HashSet<String>,
) {
    if let Some(default_event_name) = default_event_name {
        let _ = known.insert(default_event_name.to_owned());
    }
    if let Some(events) = events {
        for (source, destination) in events {
            let _ = match destination {
                Some(destination) => known.insert(destination.clone()),
                None => known.insert(source.clone()),
            };
        }
    }
}

/// Deserializable wrapper for LogsEventNameMapping
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "LogsEventNameMappingConfigRaw")]
pub struct LogsEventNameMappingConfig {
    /// The routing key configuration (determines which attribute to route on)
    pub routing_key: LogsEventNameRoutingKeyConfig,
    /// Map of source values to destination table names. A `null` value means
    /// the source value is used unchanged as the destination; empty or
    /// whitespace-only destination strings are rejected during validation.
    pub events: std::collections::HashMap<String, Option<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LogsEventNameMappingConfigRaw {
    routing_key: LogsEventNameRoutingKeyConfig,
    events: std::collections::HashMap<String, Option<String>>,
}

impl TryFrom<LogsEventNameMappingConfigRaw> for LogsEventNameMappingConfig {
    type Error = String;

    fn try_from(raw: LogsEventNameMappingConfigRaw) -> Result<Self, Self::Error> {
        validate_events_map("logs", &raw.events)?;
        Ok(Self {
            routing_key: raw.routing_key,
            events: raw.events,
        })
    }
}

impl From<LogsEventNameMappingConfig> for LogsEventNameMapping {
    fn from(config: LogsEventNameMappingConfig) -> Self {
        LogsEventNameMapping {
            routing_key: LogsEventNameRoutingKey::from(config.routing_key),
            events: config.events,
        }
    }
}

/// Log table configuration (wrapper for YAML deserialization)
/// Deserializes to Geneva uploader's LogsConfig
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct LogsConfig {
    /// Default event name (table name) for logs sent to Geneva
    #[serde(
        default,
        deserialize_with = "deserialize_optional_non_empty_event_name"
    )]
    pub default_event_name: Option<String>,
    /// Optional logs routing configuration for mapping records to different tables
    #[serde(default)]
    pub event_name_mapping: Option<LogsEventNameMappingConfig>,
}

/// Span routing key configuration (custom deserializer for validation)
#[derive(Debug, Clone)]
pub enum SpansEventNameRoutingKeyConfig {
    /// Use resource attribute value as routing key
    ResourceAttribute {
        /// Name of the resource attribute
        resource_attribute: String,
    },
    /// Use scope attribute value as routing key
    ScopeAttribute {
        /// Name of the scope attribute
        scope_attribute: String,
    },
    /// Use span attribute value as routing key
    SpanAttribute {
        /// Name of the span attribute
        span_attribute: String,
    },
}

impl From<SpansEventNameRoutingKeyConfig> for SpanEventNameRoutingKey {
    fn from(config: SpansEventNameRoutingKeyConfig) -> Self {
        match config {
            SpansEventNameRoutingKeyConfig::ResourceAttribute { resource_attribute } => {
                SpanEventNameRoutingKey::ResourceAttribute(resource_attribute)
            }
            SpansEventNameRoutingKeyConfig::ScopeAttribute { scope_attribute } => {
                SpanEventNameRoutingKey::ScopeAttribute(scope_attribute)
            }
            SpansEventNameRoutingKeyConfig::SpanAttribute { span_attribute } => {
                SpanEventNameRoutingKey::SpanAttribute(span_attribute)
            }
        }
    }
}

impl<'de> Deserialize<'de> for SpansEventNameRoutingKeyConfig {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let map: std::collections::BTreeMap<String, String> =
            Deserialize::deserialize(deserializer)?;

        let mut found_key: Option<(String, String)> = None;

        for (key, value) in &map {
            match key.as_str() {
                "resource_attribute" | "scope_attribute" | "span_attribute" => {
                    if found_key.is_some() {
                        return Err(serde::de::Error::custom(
                            "only one of: resource_attribute, scope_attribute, span_attribute should be specified",
                        ));
                    }
                    found_key = Some((key.clone(), value.clone()));
                }
                _ => {
                    return Err(serde::de::Error::unknown_field(
                        key,
                        &["resource_attribute", "scope_attribute", "span_attribute"],
                    ));
                }
            }
        }

        match found_key {
            Some((key, value)) => {
                if value.trim().is_empty() {
                    return Err(serde::de::Error::custom(format!(
                        "'{key}' must be a non-empty attribute name"
                    )));
                }
                match key.as_str() {
                    "resource_attribute" => Ok(SpansEventNameRoutingKeyConfig::ResourceAttribute {
                        resource_attribute: value,
                    }),
                    "scope_attribute" => Ok(SpansEventNameRoutingKeyConfig::ScopeAttribute {
                        scope_attribute: value,
                    }),
                    "span_attribute" => Ok(SpansEventNameRoutingKeyConfig::SpanAttribute {
                        span_attribute: value,
                    }),
                    _ => unreachable!(),
                }
            }
            None => Err(serde::de::Error::custom(
                "one of: resource_attribute, scope_attribute, span_attribute must be specified",
            )),
        }
    }
}

/// Deserializable wrapper for SpanEventNameMapping
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "SpansEventNameMappingConfigRaw")]
pub struct SpansEventNameMappingConfig {
    /// The routing key configuration (determines which attribute to route on)
    pub routing_key: SpansEventNameRoutingKeyConfig,
    /// Map of source values to destination table names. A `null` value means
    /// the source value is used unchanged as the destination; empty or
    /// whitespace-only destination strings are rejected during validation.
    pub events: std::collections::HashMap<String, Option<String>>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SpansEventNameMappingConfigRaw {
    routing_key: SpansEventNameRoutingKeyConfig,
    events: std::collections::HashMap<String, Option<String>>,
}

impl TryFrom<SpansEventNameMappingConfigRaw> for SpansEventNameMappingConfig {
    type Error = String;

    fn try_from(raw: SpansEventNameMappingConfigRaw) -> Result<Self, Self::Error> {
        validate_events_map("spans", &raw.events)?;
        Ok(Self {
            routing_key: raw.routing_key,
            events: raw.events,
        })
    }
}

impl From<SpansEventNameMappingConfig> for SpanEventNameMapping {
    fn from(config: SpansEventNameMappingConfig) -> Self {
        SpanEventNameMapping {
            routing_key: SpanEventNameRoutingKey::from(config.routing_key),
            events: config.events,
        }
    }
}

/// Span table configuration (wrapper for YAML deserialization)
/// Deserializes to Geneva uploader's TracesConfig
#[derive(Debug, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct TracesConfig {
    /// Default event name (table name) for spans sent to Geneva
    #[serde(
        default,
        deserialize_with = "deserialize_optional_non_empty_event_name"
    )]
    pub default_event_name: Option<String>,
    /// Optional spans routing configuration for mapping records to different tables
    #[serde(default)]
    pub event_name_mapping: Option<SpansEventNameMappingConfig>,
}

/// OBO (On-Behalf-Of) configuration for the Geneva exporter.
///
/// OBO lets a single agent upload telemetry on behalf of multiple customer
/// identities. The map is keyed by Geneva event/table name; a batch whose
/// event name matches an entry is uploaded with that entry's customer identity
/// and optional annotations recipe carried as GIG query parameters. Events not
/// present in the map are uploaded without OBO. This mirrors the uploader's
/// flat, per-event `OboEventMap` data model (a single map shared across logs
/// and spans, keyed by event/table name).
///
/// Keys are the *destination* event/table name -- the name after
/// `logs`/`spans` `event_name_mapping` has resolved it, not the pre-mapping
/// source value. The uploader looks up OBO by the resolved destination name.
/// For example, if `event_name_mapping` routes source `audit` to table
/// `AuditLogs`, the OBO entry must be keyed `AuditLogs`.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "OboConfigRaw")]
pub struct OboConfig {
    /// Map of Geneva event/table name -> per-event OBO entry. Keys are the
    /// destination table name (after `event_name_mapping` resolves it), not the
    /// pre-mapping source value.
    pub events: std::collections::HashMap<String, OboEventEntryConfig>,
}

/// A single OBO entry: the resolved customer identity and an optional
/// annotations recipe.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct OboEventEntryConfig {
    /// Resolved OBO identity (the GIG `onbehalfid`). Must be non-empty.
    pub identity: String,
    /// Optional OBO annotations recipe (the GIG `onbehalfannotations`), for
    /// example `<Config onBehalfFields="resourceId" />`. When present it must
    /// be non-empty.
    #[serde(default)]
    pub annotations: Option<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct OboConfigRaw {
    events: std::collections::HashMap<String, OboEventEntryConfig>,
}

impl TryFrom<OboConfigRaw> for OboConfig {
    type Error = String;

    fn try_from(raw: OboConfigRaw) -> Result<Self, Self::Error> {
        if raw.events.is_empty() {
            return Err("obo.events must be non-empty when OBO is configured".to_owned());
        }
        for (event_name, entry) in &raw.events {
            if event_name.trim().is_empty() {
                return Err("obo.events keys (event/table names) must not be blank".to_owned());
            }
            if entry.identity.trim().is_empty() {
                return Err(format!(
                    "obo.events entry for '{event_name}' must have a non-empty identity"
                ));
            }
            if let Some(annotations) = &entry.annotations {
                if annotations.trim().is_empty() {
                    return Err(format!(
                        "obo.events entry for '{event_name}' has an empty annotations value; \
                         omit the field (or use null) instead of an empty string"
                    ));
                }
            }
        }
        Ok(Self { events: raw.events })
    }
}

impl From<OboConfig> for OboEventMap {
    fn from(config: OboConfig) -> Self {
        config
            .events
            .into_iter()
            .map(|(event_name, entry)| {
                (
                    event_name,
                    OboEventConfig {
                        identity: entry.identity,
                        annotations: entry.annotations,
                    },
                )
            })
            .collect()
    }
}

/// Routes final Geneva event/table names to logical GCS account groups.
///
/// Event overrides are keyed by the destination event/table name after
/// `event_name_mapping` has run. Events without an override use
/// `default_group`.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "AccountRoutingConfigRaw")]
pub struct AccountRoutingConfig {
    /// Logical account group used when no event-specific override matches.
    pub default_group: String,
    /// Optional destination event/table name -> logical account group map.
    #[serde(default)]
    pub events: std::collections::HashMap<String, String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct AccountRoutingConfigRaw {
    default_group: String,
    #[serde(default)]
    events: std::collections::HashMap<String, String>,
}

impl TryFrom<AccountRoutingConfigRaw> for AccountRoutingConfig {
    type Error = String;

    fn try_from(raw: AccountRoutingConfigRaw) -> Result<Self, Self::Error> {
        if raw.default_group.trim().is_empty() {
            return Err("account_routing.default_group must not be empty".to_owned());
        }
        if raw.default_group.trim() != raw.default_group {
            return Err(
                "account_routing.default_group must not have surrounding whitespace".to_owned(),
            );
        }
        for (event_name, account_group) in &raw.events {
            if event_name.trim().is_empty() || account_group.trim().is_empty() {
                return Err(
                    "account_routing event/table names and account groups must not be empty"
                        .to_owned(),
                );
            }
            if event_name.trim() != event_name || account_group.trim() != account_group {
                return Err(
                    "account_routing event/table names and account groups must not have surrounding whitespace"
                        .to_owned(),
                );
            }
        }
        Ok(Self {
            default_group: raw.default_group,
            events: raw.events,
        })
    }
}

impl From<AccountRoutingConfig> for AccountRouting {
    fn from(config: AccountRoutingConfig) -> Self {
        Self::new(config.default_group).with_event_groups(config.events)
    }
}

/// Configuration for the Geneva Exporter
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Geneva config-service endpoint URL (required except for agent-fed auth)
    #[serde(default)]
    pub endpoint: String,
    /// Environment (e.g., "production", "staging")
    pub environment: String,
    /// Geneva account name
    pub account: String,
    /// Geneva namespace
    pub namespace: String,
    /// Logical account-group routing used to select the physical moniker for
    /// each final event/table name.
    pub account_routing: AccountRoutingConfig,
    /// Azure region (required except for agent-fed auth)
    #[serde(default)]
    pub region: String,
    /// Config major version (required)
    pub config_major_version: u32,
    /// Tenant name
    pub tenant: String,
    /// Role name
    pub role_name: String,
    /// Role instance identifier
    pub role_instance: String,
    /// Authentication configuration
    pub auth: AuthConfig,
    /// Log table configuration
    #[serde(default)]
    pub logs: Option<LogsConfig>,
    /// Span table configuration
    #[serde(default)]
    pub spans: Option<TracesConfig>,
    /// Optional OBO (On-Behalf-Of) configuration for uploading telemetry on
    /// behalf of multiple customer identities, keyed by the destination
    /// event/table name (after `event_name_mapping` resolves it).
    #[serde(default)]
    pub obo: Option<OboConfig>,
    /// Maximum buffer size before forcing flush (default: 1000)
    /// Note: This field is currently reserved for future use and does not affect runtime behavior.
    #[serde(default = "default_buffer_size")]
    pub max_buffer_size: usize,
    /// Maximum concurrent uploads (default: 4)
    #[serde(default = "default_max_concurrent")]
    pub max_concurrent_uploads: usize,
}

const fn default_buffer_size() -> usize {
    1000
}

const fn default_max_concurrent() -> usize {
    4
}

impl Config {
    /// Deserializes and validates one exporter configuration.
    ///
    /// Validation lives here rather than inside `Deserialize` so `Config` stays
    /// a plain deserializable struct with a single field list. Every path that
    /// turns user JSON into a `Config` must go through this function.
    fn parse(config: &serde_json::Value) -> Result<Self, ConfigError> {
        let parsed: Self = serde_json::from_value(config.clone()).map_err(|error| {
            ConfigError::InvalidUserConfig {
                error: error.to_string(),
            }
        })?;
        parsed
            .validate()
            .map_err(|error| ConfigError::InvalidUserConfig { error })?;
        Ok(parsed)
    }

    fn validate(&self) -> Result<(), String> {
        if matches!(self.auth, AuthConfig::Certificate { .. })
            && !cfg!(feature = "geneva-certificate-auth")
        {
            return Err(
                "certificate authentication requires the 'geneva-certificate-auth' build feature"
                    .to_owned(),
            );
        }
        let is_agent_fed = matches!(self.auth, AuthConfig::AgentFed);
        if !is_agent_fed {
            if self.endpoint.trim().is_empty() {
                return Err("endpoint is required unless auth.type is agentfed".to_owned());
            }
            if self.region.trim().is_empty() {
                return Err("region is required unless auth.type is agentfed".to_owned());
            }
        }
        self.warn_unmatched_obo_events();
        Ok(())
    }

    /// Warn (non-fatally) about OBO entries that key on an event/table name no
    /// statically-known destination can produce -- the common symptom of keying
    /// OBO on the pre-mapping *source* value instead of the resolved
    /// destination table name.
    ///
    /// The uploader looks up OBO by the destination event name (after
    /// `event_name_mapping` resolves it), so an entry keyed on a source value or
    /// a typo silently never applies. This is a warning rather than an error
    /// because the set of reachable destinations is not fully static: a batch
    /// can fall back to the uploader's literal default table name when
    /// `default_event_name` is unset, and attribute-based routing derives
    /// destinations from runtime values. We only warn when a key matches none of
    /// the names we *can* enumerate.
    fn warn_unmatched_obo_events(&self) {
        for event_name in self.unmatched_obo_events() {
            otel_warn!(
                "geneva_exporter.obo.unmatched_event",
                event_name = event_name.clone(),
                message = "OBO is configured for an event/table name that no configured \
                           routing destination or default_event_name produces; OBO keys must \
                           be the destination table name (after event_name_mapping), not the \
                           source value. This OBO entry may never apply."
            );
        }
    }

    /// Return the OBO event keys that cannot match any statically reachable
    /// destination table name. A key is considered reachable if it is a
    /// configured mapping destination, a passthrough source, an explicit
    /// `default_event_name`, or, when the corresponding `default_event_name` is
    /// unset, the uploader's literal default table name ("Log"/"Span").
    /// Attribute-based routing can still produce destinations not listed here,
    /// so this is best-effort typo detection.
    fn unmatched_obo_events(&self) -> Vec<String> {
        let Some(obo) = &self.obo else {
            return Vec::new();
        };

        let mut known = std::collections::HashSet::new();

        let logs_default = self
            .logs
            .as_ref()
            .and_then(|l| l.default_event_name.as_deref());
        let spans_default = self
            .spans
            .as_ref()
            .and_then(|s| s.default_event_name.as_deref());

        // When `default_event_name` is unset, the uploader falls back to its
        // literal default table names ("Log" for logs, "Span" for spans), so
        // OBO entries keyed on those are valid and must not warn. When the user
        // overrides the default, that literal is never a reachable destination,
        // so it must not be seeded (an OBO key on it is likely a typo).
        if logs_default.is_none() {
            let _ = known.insert("Log".to_owned());
        }
        if spans_default.is_none() {
            let _ = known.insert("Span".to_owned());
        }
        collect_known_destinations(
            logs_default,
            self.logs
                .as_ref()
                .and_then(|l| l.event_name_mapping.as_ref().map(|m| &m.events)),
            &mut known,
        );
        collect_known_destinations(
            spans_default,
            self.spans
                .as_ref()
                .and_then(|s| s.event_name_mapping.as_ref().map(|m| &m.events)),
            &mut known,
        );

        obo.events
            .keys()
            .filter(|event_name| !known.contains(event_name.as_str()))
            .cloned()
            .collect()
    }

    /// Build the `geneva-uploader` `GenevaClientConfig` from this exporter
    /// configuration.
    ///
    /// This is the adapter owned by this crate: it maps the local, serde-facing
    /// config types onto the uploader's client config, including the auth
    /// method, MSI resource, and the per-signal logs/spans default table names
    /// and routing mappings. It is factored out of
    /// [`GenevaExporter::from_config`] so the pure `Config` ->
    /// `GenevaClientConfig` conversion can be unit-tested without initializing
    /// a live `GenevaClient`.
    fn to_geneva_client_config(&self) -> GenevaClientConfig {
        // Create LogsConfig and TracesConfig from the configuration
        let logs_config = self
            .logs
            .as_ref()
            .map(|logs| geneva_uploader::client::LogsConfig {
                default_event_name: logs.default_event_name.clone(),
                event_name_mapping: logs.event_name_mapping.clone().map(|m| m.into()),
            });
        let traces_config =
            self.spans
                .as_ref()
                .map(|spans| geneva_uploader::client::TracesConfig {
                    default_event_name: spans.default_event_name.clone(),
                    event_name_mapping: spans.event_name_mapping.clone().map(|m| m.into()),
                });

        GenevaClientConfig {
            endpoint: self.endpoint.clone(),
            environment: self.environment.clone(),
            account: self.account.clone(),
            namespace: self.namespace.clone(),
            account_routing: self.account_routing.clone().into(),
            region: self.region.clone(),
            config_major_version: self.config_major_version,
            auth_method: self.auth.uploader_auth_method(),
            tenant: self.tenant.clone(),
            role_name: self.role_name.clone(),
            role_instance: self.role_instance.clone(),
            msi_resource: self.auth.msi_resource(),
            logs: logs_config,
            spans: traces_config,
            obo_event_map: self.obo.clone().map(Into::into),
        }
    }
}

/// Authentication configuration
/// TODO - see if we directly use AuthMethod from geneva-uploader crate
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum AuthConfig {
    /// Certificate-based authentication (PKCS#12 format)
    Certificate {
        /// Path to PKCS#12 (.p12) certificate file
        path: String,
        /// Password to decrypt the PKCS#12 file
        password: String,
    },
    /// System-assigned managed identity
    SystemManagedIdentity {
        /// MSI resource identifier
        msi_resource: String,
    },
    /// User-assigned managed identity (by client ID)
    UserManagedIdentity {
        /// Client ID of the managed identity
        client_id: String,
        /// MSI resource identifier
        msi_resource: String,
    },
    /// User-assigned managed identity (by ARM resource ID)
    /// Identifies the managed identity by its Azure Resource Manager resource ID.
    UserManagedIdentityByArmResourceId {
        /// ARM resource ID of the extension identity
        resource_id: String,
        /// MSI resource identifier
        msi_resource: String,
    },
    /// Workload identity (Kubernetes)
    WorkloadIdentity {
        /// MSI resource identifier
        msi_resource: String,
    },
    /// Agent-fed credentials: the host agent supplies one atomic GIG token and
    /// routing snapshot through `agent_fed_credential_provider`, so the
    /// uploader skips the GCS config-service handshake. Carries no config
    /// fields; the capability is selected through the node's `capabilities`
    /// block.
    AgentFed,
}

impl AuthConfig {
    fn uploader_auth_method(&self) -> AuthMethod {
        match self {
            Self::Certificate { path, password } => AuthMethod::Certificate {
                path: PathBuf::from(path),
                password: password.clone(),
            },
            Self::SystemManagedIdentity { .. } => AuthMethod::SystemManagedIdentity,
            Self::UserManagedIdentity { client_id, .. } => AuthMethod::UserManagedIdentity {
                client_id: client_id.clone(),
            },
            Self::UserManagedIdentityByArmResourceId { resource_id, .. } => {
                AuthMethod::UserManagedIdentityByResourceId {
                    resource_id: resource_id.clone(),
                }
            }
            Self::WorkloadIdentity { msi_resource } => AuthMethod::WorkloadIdentity {
                resource: msi_resource.clone(),
            },
            // `with_agent_fed_source` ignores this field. The placeholder only
            // satisfies the shared uploader configuration type.
            Self::AgentFed => AuthMethod::SystemManagedIdentity,
        }
    }

    fn msi_resource(&self) -> Option<String> {
        match self {
            Self::SystemManagedIdentity { msi_resource }
            | Self::UserManagedIdentity { msi_resource, .. }
            | Self::UserManagedIdentityByArmResourceId { msi_resource, .. }
            | Self::WorkloadIdentity { msi_resource } => Some(msi_resource.clone()),
            Self::Certificate { .. } | Self::AgentFed => None,
        }
    }
}

/// Geneva exporter metrics.
/// Grouped under `otap.exporter.geneva`.
///
/// Upload, failure, and latency counters are split per signal type (logs vs
/// traces) so operators can identify which signal is failing or slow.
#[metric_set(name = "otap.exporter.geneva")]
#[derive(Debug, Default, Clone)]
struct ExporterMetrics {
    // -- Log-signal counters ------------------------------------------------
    /// Compressed log batches produced by the encoder.
    #[metric(unit = "{batch}")]
    pub log_batches_encoded: Counter<u64>,

    /// Log batches successfully uploaded to Geneva.
    #[metric(unit = "{batch}")]
    pub log_batches_uploaded: Counter<u64>,

    /// Log batches that failed to upload.
    #[metric(unit = "{batch}")]
    pub log_batches_failed: Counter<u64>,

    /// Individual log records successfully uploaded.
    #[metric(unit = "{record}")]
    pub log_records_uploaded: Counter<u64>,

    /// Individual log records that failed to upload.
    #[metric(unit = "{record}")]
    pub log_records_failed: Counter<u64>,

    /// Log bytes uploaded to Geneva (compressed payload size).
    #[metric(unit = "By")]
    pub log_bytes_uploaded: Counter<u64>,

    /// Per-upload latency for successful log batches in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub log_upload_success_duration: Mmsc,

    /// Per-upload latency for failed log batches in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub log_upload_failed_duration: Mmsc,

    /// Encode + compress latency for logs in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub log_encode_duration: Mmsc,

    // -- Trace-signal counters ------------------------------------------------
    /// Compressed trace batches produced by the encoder.
    #[metric(unit = "{batch}")]
    pub trace_batches_encoded: Counter<u64>,

    /// Trace batches successfully uploaded to Geneva.
    #[metric(unit = "{batch}")]
    pub trace_batches_uploaded: Counter<u64>,

    /// Trace batches that failed to upload.
    #[metric(unit = "{batch}")]
    pub trace_batches_failed: Counter<u64>,

    /// Individual trace records (spans) successfully uploaded.
    #[metric(unit = "{record}")]
    pub trace_records_uploaded: Counter<u64>,

    /// Individual trace records (spans) that failed to upload.
    #[metric(unit = "{record}")]
    pub trace_records_failed: Counter<u64>,

    /// Trace bytes uploaded to Geneva (compressed payload size).
    #[metric(unit = "By")]
    pub trace_bytes_uploaded: Counter<u64>,

    /// Per-upload latency for successful trace batches in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub trace_upload_success_duration: Mmsc,

    /// Per-upload latency for failed trace batches in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub trace_upload_failed_duration: Mmsc,

    /// Encode + compress latency for traces in milliseconds (min/max/sum/count).
    #[metric(unit = "ms")]
    pub trace_encode_duration: Mmsc,

    // -- Signal-agnostic counters ---------------------------------------------
    /// Number of empty payloads skipped (no-op ack).
    #[metric(unit = "{msg}")]
    pub empty_payloads_skipped: Counter<u64>,

    /// Number of OTAP-to-OTLP conversion errors.
    #[metric(unit = "{error}")]
    pub conversion_errors: Counter<u64>,

    /// Number of metrics payloads dropped (unsupported signal).
    #[metric(unit = "{msg}")]
    pub metrics_payloads_dropped: Counter<u64>,
}

/// Geneva exporter that sends OTAP data to Geneva backend
pub struct GenevaExporter {
    config: Config,
    pdata_metrics: MeasurementMetricSet<ExporterExportMetrics>,
    metrics: MetricSet<ExporterMetrics>,
    geneva_client: GenevaClient,
}

/// Validates the node-level bindings that the config-only factory validation
/// hook cannot inspect. Called during exporter creation before the capability
/// is consumed.
fn validate_agent_fed_capability_binding(node_config: &NodeUserConfig) -> Result<(), ConfigError> {
    let capability_name = AgentFedCredentialProviderCap::NAME;
    node_config
        .capabilities
        .get(capability_name)
        .map(|_| ())
        .ok_or_else(|| ConfigError::InvalidUserConfig {
            error: format!(
                "agent-fed Geneva auth requires an '{capability_name}' capability binding"
            ),
        })
}

fn ensure_crypto_provider() -> Result<(), ConfigError> {
    if otap_df_otap::crypto::is_crypto_provider_installed() {
        return Ok(());
    }

    Err(ConfigError::InvalidUserConfig {
        error: "Geneva exporter requires a rustls CryptoProvider, but none is installed. \
                Build with exactly one of the crypto-* features \
                (crypto-ring, crypto-aws-lc, crypto-openssl, crypto-symcrypt) and ensure \
                otap_df_otap::crypto::install_crypto_provider() runs at startup."
            .to_string(),
    })
}

fn validate_geneva_client_prerequisites(
    config: &Config,
    node_config: &NodeUserConfig,
    crypto_provider_check: impl FnOnce() -> Result<(), ConfigError>,
) -> Result<(), ConfigError> {
    if matches!(&config.auth, AuthConfig::AgentFed) {
        validate_agent_fed_capability_binding(node_config)?;
    }

    // Both GCS and agent-fed uploads use the rustls-backed HTTP client.
    crypto_provider_check()
}

fn resolve_agent_fed_source(
    capabilities: &Capabilities,
) -> Result<AgentFedGenevaSource, ConfigError> {
    let credential_provider = capabilities
        .require_shared::<AgentFedCredentialProviderCap>()
        .map_err(|error| ConfigError::InvalidUserConfig {
            error: format!(
                "agent-fed Geneva auth requires the bound agent_fed_credential_provider \
                 capability to provide a shared implementation; local-only registrations \
                 are unsupported: {error}"
            ),
        })?;

    Ok(AgentFedGenevaSource::new(credential_provider))
}

fn create_geneva_client(
    config: &Config,
    node_config: &NodeUserConfig,
    capabilities: &Capabilities,
) -> Result<GenevaClient, ConfigError> {
    validate_geneva_client_prerequisites(config, node_config, ensure_crypto_provider)?;
    let client_config = config.to_geneva_client_config();

    match &config.auth {
        AuthConfig::AgentFed => {
            let source = resolve_agent_fed_source(capabilities)?;
            GenevaClient::with_agent_fed_source(client_config, Arc::new(source)).map_err(|error| {
                ConfigError::InvalidUserConfig {
                    error: format!("Failed to initialize agent-fed Geneva client: {error}"),
                }
            })
        }
        AuthConfig::Certificate { .. }
        | AuthConfig::SystemManagedIdentity { .. }
        | AuthConfig::UserManagedIdentity { .. }
        | AuthConfig::UserManagedIdentityByArmResourceId { .. }
        | AuthConfig::WorkloadIdentity { .. } => {
            GenevaClient::new(client_config).map_err(|error| ConfigError::InvalidUserConfig {
                error: format!("Failed to initialize Geneva client: {error}"),
            })
        }
    }
}

impl GenevaExporter {
    /// Creates a Geneva exporter from configuration for legacy authentication modes.
    ///
    /// Agent-fed authentication must be constructed through the registered
    /// factory so its capability binding can be resolved.
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, ConfigError> {
        let config = Config::parse(config)?;
        let node_config = NodeUserConfig::new_exporter_config(GENEVA_EXPORTER_URN);
        Self::from_parsed_config(pipeline_ctx, config, &node_config, &Capabilities::empty())
    }

    fn from_node_config(
        pipeline_ctx: PipelineContext,
        node_config: &NodeUserConfig,
        capabilities: &Capabilities,
    ) -> Result<Self, ConfigError> {
        let config = Config::parse(&node_config.config)?;
        Self::from_parsed_config(pipeline_ctx, config, node_config, capabilities)
    }

    fn from_parsed_config(
        pipeline_ctx: PipelineContext,
        config: Config,
        node_config: &NodeUserConfig,
        capabilities: &Capabilities,
    ) -> Result<Self, ConfigError> {
        let geneva_client = create_geneva_client(&config, node_config, capabilities)?;
        let pdata_metrics = ExporterExportMetrics::register(&pipeline_ctx);
        let metrics = pipeline_ctx.register_metrics::<ExporterMetrics>();

        Ok(Self {
            config,
            pdata_metrics,
            metrics,
            geneva_client,
        })
    }

    /// Get exporter configuration
    #[must_use]
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Upload batches concurrently.
    ///
    /// All batches are attempted regardless of individual failures (no
    /// short-circuit). Per-batch upload latency and per-signal success/failure
    /// counters are recorded accurately using `batch.row_count`.
    ///
    /// # Partial-success limitation
    ///
    /// TODO: When some batches succeed and others fail, this method returns
    /// `Err` and the caller NACKs the entire payload. The retry processor then
    /// resends the whole payload, re-uploading already-successful batches and
    /// causing duplicates (Geneva assigns a fresh UUID per upload, so there is
    /// no server-side dedup). Unlike the Azure Monitor exporter, cross-message
    /// batch dedup doesn't apply here because Geneva maps one `OtapPdata` to N
    /// batches with no sharing. The real fix requires engine-level support for
    /// per-batch retry tracking (partial ACK/NACK or exporter-attached retry
    /// context on `OtapPdata`).
    async fn upload_batches_concurrent(
        &mut self,
        batches: &[EncodedBatch],
        signal_type: SignalType,
    ) -> Result<usize, String> {
        let batches_encoded = batches.len();
        match signal_type {
            SignalType::Logs => self.metrics.log_batches_encoded.add(batches_encoded as u64),
            SignalType::Traces => self
                .metrics
                .trace_batches_encoded
                .add(batches_encoded as u64),
            _ => {}
        }

        let max_concurrent = self.config.max_concurrent_uploads.max(1);
        let client = &self.geneva_client;

        // Run all uploads concurrently, processing results inline via streaming
        // to avoid an intermediate Vec allocation.
        let mut stream = futures::stream::iter(batches.iter())
            .map(|batch| {
                // TODO(https://github.com/open-telemetry/opentelemetry-rust-contrib/issues/605):
                // restore compressed byte accounting after geneva-uploader exposes a public
                // accessor such as EncodedBatch::compressed_len() returning the post-compression
                // payload size uploaded to Geneva.
                let batch_size: Option<u64> = None;
                let row_count = batch.row_count as u64;
                async move {
                    let start = Instant::now();
                    let result = client
                        .upload_batch(batch)
                        .await
                        .map_err(|e| format!("Failed to upload {:?} batch: {e}", signal_type));
                    let duration_ms = start.elapsed().as_secs_f64() * 1000.0;
                    (result, duration_ms, batch_size, row_count)
                }
            })
            .buffer_unordered(max_concurrent);

        // Aggregate results and update per-signal metrics.
        let mut first_error: Option<String> = None;
        let mut succeeded: u64 = 0;
        let mut failed: u64 = 0;
        let mut records_ok: u64 = 0;
        let mut records_err: u64 = 0;
        let mut bytes_ok: Option<u64> = None;

        while let Some((result, duration_ms, batch_size, row_count)) = stream.next().await {
            match result {
                Ok(()) => {
                    match signal_type {
                        SignalType::Logs => {
                            self.metrics.log_upload_success_duration.record(duration_ms);
                        }
                        SignalType::Traces => {
                            self.metrics
                                .trace_upload_success_duration
                                .record(duration_ms);
                        }
                        _ => {}
                    }
                    succeeded += 1;
                    records_ok += row_count;
                    if let Some(batch_size) = batch_size {
                        bytes_ok = Some(bytes_ok.unwrap_or_default() + batch_size);
                    }
                }
                Err(e) => {
                    match signal_type {
                        SignalType::Logs => {
                            self.metrics.log_upload_failed_duration.record(duration_ms);
                        }
                        SignalType::Traces => {
                            self.metrics
                                .trace_upload_failed_duration
                                .record(duration_ms);
                        }
                        _ => {}
                    }
                    failed += 1;
                    records_err += row_count;
                    if first_error.is_none() {
                        first_error = Some(e);
                    }
                }
            }
        }

        match signal_type {
            SignalType::Logs => {
                self.metrics.log_batches_uploaded.add(succeeded);
                self.metrics.log_records_uploaded.add(records_ok);
                if let Some(bytes_ok) = bytes_ok {
                    self.metrics.log_bytes_uploaded.add(bytes_ok);
                }
                self.metrics.log_batches_failed.add(failed);
                self.metrics.log_records_failed.add(records_err);
            }
            SignalType::Traces => {
                self.metrics.trace_batches_uploaded.add(succeeded);
                self.metrics.trace_records_uploaded.add(records_ok);
                if let Some(bytes_ok) = bytes_ok {
                    self.metrics.trace_bytes_uploaded.add(bytes_ok);
                }
                self.metrics.trace_batches_failed.add(failed);
                self.metrics.trace_records_failed.add(records_err);
            }
            _ => {}
        }

        if let Some(e) = first_error {
            Err(e)
        } else {
            Ok(batches_encoded)
        }
    }

    /// Handle PData message with dual-path log encoding.
    ///
    /// Supports two log data paths for Geneva encoding:
    /// - **OTAP Arrow view path**: OTAP Arrow RecordBatch -> Geneva (via LogsDataView)
    ///   Avoids protobuf deserialization by iterating directly over Arrow columns.
    ///   Used when data flows through a batch processor or syslog receiver.
    /// - **OTLP raw-byte view path**: OTLP bytes -> Geneva (via RawLogsData)
    ///   Used when an OTLP receiver connects directly to the Geneva exporter.
    ///   Validates top-level protobuf framing without materializing OTLP protobuf structs.
    ///
    /// Trace export still uses the existing fallback/prost path.
    async fn export_payload(
        &mut self,
        payload: OtapPayload,
        _effect_handler: &EffectHandler<OtapPdata>,
    ) -> Result<usize, String> {
        if payload.is_empty() {
            self.metrics.empty_payloads_skipped.inc();
            otel_info!(
                "geneva_exporter.skip",
                message = "Geneva exporter skipping empty payload"
            );
            return Ok(0);
        }

        // Handle based on payload type
        match payload {
            // OTAP Arrow path: encode logs through LogsDataView without converting back to OTLP.
            OtapPayload::OtapArrowRecords(otap_records) => {
                match otap_records {
                    mut otap_records @ OtapArrowRecords::Logs(_) => {
                        otel_info!(
                            "geneva_exporter.upload",
                            message = "Uploading log batches to Geneva using OTAP record views"
                        );

                        otap_records.decode_transport_optimized_ids().map_err(|e| {
                            self.metrics.conversion_errors.inc();
                            format!("Failed to decode OTAP transport-optimized log IDs: {}", e)
                        })?;

                        let logs_view = OtapLogsView::try_from(&otap_records).map_err(|e| {
                            self.metrics.conversion_errors.inc();
                            format!("Failed to build OTAP logs view: {}", e)
                        })?;

                        let encode_start = Instant::now();
                        let batches = self
                            .geneva_client
                            .encode_and_compress_logs(&logs_view)
                            .map_err(|e| format!("Failed to encode logs: {}", e))?;
                        let encode_ms = encode_start.elapsed().as_secs_f64() * 1000.0;
                        self.metrics.log_encode_duration.record(encode_ms);

                        let batches_uploaded = self
                            .upload_batches_concurrent(&batches, SignalType::Logs)
                            .await?;

                        otel_info!(
                            "geneva_exporter.upload",
                            count = batches_uploaded,
                            message = "Successfully uploaded log batches to Geneva using OTAP record views"
                        );

                        Ok(batches_uploaded)
                    }
                    OtapArrowRecords::Traces(otap_records) => {
                        // TODO: Zero-copy view path for future optimization (when TracesView is ready)

                        // Fallback path: Convert OTAP Arrow -> OTLP bytes
                        otel_info!(
                            "geneva_exporter.convert",
                            message = "Converting OTAP traces to OTLP bytes (fallback path)"
                        );

                        let otlp_bytes: OtlpProtoBytes =
                            OtapPayload::OtapArrowRecords(OtapArrowRecords::Traces(otap_records))
                                .try_into_with_default()
                                .map_err(|e| {
                                    self.metrics.conversion_errors.inc();
                                    format!("Failed to convert OTAP to OTLP: {:?}", e)
                                })?;

                        let OtlpProtoBytes::ExportTracesRequest(bytes) = otlp_bytes else {
                            self.metrics.conversion_errors.inc();
                            return Err("Expected traces but got different signal type".to_string());
                        };

                        // Decode OTLP bytes to ResourceSpans
                        let traces_request = ExportTraceServiceRequest::decode(&bytes[..])
                            .map_err(|e| {
                                self.metrics.conversion_errors.inc();
                                format!("Failed to decode traces request: {}", e)
                            })?;

                        // Encode and compress using Geneva client
                        let encode_start = Instant::now();
                        let batches = self
                            .geneva_client
                            .encode_and_compress_spans(&traces_request.resource_spans[..])
                            .map_err(|e| format!("Failed to encode spans: {}", e))?;
                        let encode_ms = encode_start.elapsed().as_secs_f64() * 1000.0;
                        self.metrics.trace_encode_duration.record(encode_ms);

                        let batches_uploaded = self
                            .upload_batches_concurrent(&batches, SignalType::Traces)
                            .await?;

                        otel_info!(
                            "geneva_exporter.upload",
                            count = batches_uploaded,
                            message =
                                "Successfully uploaded trace batches to Geneva (OTAP fallback)"
                        );

                        Ok(batches_uploaded)
                    }
                    OtapArrowRecords::Metrics(_) => {
                        self.metrics.metrics_payloads_dropped.inc();
                        Err("Geneva exporter does not support metrics signal".to_string())
                    }
                }
            }

            // OTLP path: Direct OTLP bytes from receivers without OTAP conversion (e.g., OTLP receiver -> Geneva exporter without batch processor)
            OtapPayload::OtlpBytes(otlp_bytes) => {
                match otlp_bytes {
                    OtlpProtoBytes::ExportLogsRequest(bytes) => {
                        otel_info!(
                            "geneva_exporter.upload",
                            message = "Uploading log batches to Geneva using OTLP raw-byte view"
                        );

                        let logs_view = RawLogsData::try_new(bytes.as_ref()).map_err(|e| {
                            self.metrics.conversion_errors.inc();
                            format!("Failed to decode logs request: {}", e)
                        })?;

                        // Encode and compress using Geneva client
                        let encode_start = Instant::now();
                        let batches = self
                            .geneva_client
                            .encode_and_compress_logs(&logs_view)
                            .map_err(|e| format!("Failed to encode logs: {}", e))?;
                        let encode_ms = encode_start.elapsed().as_secs_f64() * 1000.0;
                        self.metrics.log_encode_duration.record(encode_ms);

                        let batches_uploaded = self
                            .upload_batches_concurrent(&batches, SignalType::Logs)
                            .await?;

                        otel_info!(
                            "geneva_exporter.upload",
                            count = batches_uploaded,
                            message = "Successfully uploaded log batches to Geneva using OTLP raw-byte view"
                        );

                        Ok(batches_uploaded)
                    }
                    OtlpProtoBytes::ExportTracesRequest(bytes) => {
                        otel_info!(
                            "geneva_exporter.upload",
                            message = "Uploading traces to Geneva using OTLP path"
                        );

                        // Decode OTLP bytes to ResourceSpans
                        let traces_request = ExportTraceServiceRequest::decode(&bytes[..])
                            .map_err(|e| {
                                self.metrics.conversion_errors.inc();
                                format!("Failed to decode traces request: {}", e)
                            })?;

                        // Encode and compress using Geneva client
                        let encode_start = Instant::now();
                        let batches = self
                            .geneva_client
                            .encode_and_compress_spans(&traces_request.resource_spans[..])
                            .map_err(|e| format!("Failed to encode spans: {}", e))?;
                        let encode_ms = encode_start.elapsed().as_secs_f64() * 1000.0;
                        self.metrics.trace_encode_duration.record(encode_ms);

                        let batches_uploaded = self
                            .upload_batches_concurrent(&batches, SignalType::Traces)
                            .await?;

                        otel_info!(
                            "geneva_exporter.upload",
                            count = batches_uploaded,
                            message = "Successfully uploaded trace batches to Geneva (OTLP path)"
                        );

                        Ok(batches_uploaded)
                    }
                    OtlpProtoBytes::ExportMetricsRequest(_) => {
                        self.metrics.metrics_payloads_dropped.inc();
                        Err("Geneva exporter does not support metrics signal".to_string())
                    }
                }
            }
        }
    }
}

/// Validates the exporter configuration for the factory's config-only hook.
///
/// Routes through [`Config::parse`] so the hook applies exactly the validation
/// the exporter applies at creation time.
fn validate_geneva_config(config: &serde_json::Value) -> Result<(), ConfigError> {
    Config::parse(config).map(|_| ())
}

/// Register Geneva exporter with the OTAP exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[otap_df_engine::component_inventory(category = Exporter)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static GENEVA_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: GENEVA_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             capabilities: &Capabilities| {
        Ok(ExporterWrapper::local(
            GenevaExporter::from_node_config(pipeline, &node_config, capabilities)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: validate_geneva_config,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for GenevaExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        match &self.config.auth {
            AuthConfig::AgentFed => {
                otel_info!(
                    "geneva_exporter.start",
                    endpoint_source = "agent_fed_credential_provider",
                    namespace = self.config.namespace,
                    account = self.config.account,
                    role_name = self.config.role_name,
                    role_instance = self.config.role_instance,
                    message = "Geneva exporter starting"
                );
            }
            _ => {
                otel_info!(
                    "geneva_exporter.start",
                    endpoint = self.config.endpoint,
                    namespace = self.config.namespace,
                    account = self.config.account,
                    role_name = self.config.role_name,
                    role_instance = self.config.role_instance,
                    message = "Geneva exporter starting"
                );
            }
        }

        // Message loop
        loop {
            match msg_chan.recv().await? {
                Message::Control(NodeControlMsg::Shutdown { deadline, .. }) => {
                    otel_info!(
                        "geneva_exporter.shutdown",
                        message = "Geneva exporter shutting down"
                    );

                    return Ok(TerminalState::new(deadline, {
                        let mut snapshots = self.pdata_metrics.terminal_snapshots();
                        snapshots.push(self.metrics.snapshot());
                        snapshots
                    }));
                }
                Message::Control(NodeControlMsg::CollectTelemetry {
                    mut metrics_reporter,
                }) => {
                    _ = metrics_reporter.report_measurement(&mut self.pdata_metrics);
                    _ = metrics_reporter.report(&mut self.metrics);
                }
                Message::PData(pdata) => {
                    let export_start = Instant::now();
                    let (context, payload) = pdata.into_parts();
                    let signal_type = payload.signal_type();

                    let saved_payload = if context.may_return_payload() {
                        payload.clone()
                    } else {
                        OtapPayload::empty(signal_type)
                    };

                    match self.export_payload(payload, &effect_handler).await {
                        Ok(_batches_uploaded) => {
                            self.pdata_metrics
                                .with(SignalOutcomeAttributes {
                                    signal: signal_type,
                                    outcome: Outcome::Success,
                                })
                                .record(export_start.elapsed());
                            effect_handler
                                .notify_ack(AckMsg::new(OtapPdata::new(context, saved_payload)))
                                .await?;
                        }
                        Err(e) => {
                            self.pdata_metrics
                                .with(SignalOutcomeAttributes {
                                    signal: signal_type,
                                    outcome: Outcome::Failure,
                                })
                                .record(export_start.elapsed());
                            otel_info!(
                                "geneva_exporter.error",
                                error = e,
                                message = "Failed to export to Geneva"
                            );
                            effect_handler
                                .notify_nack(NackMsg::new(
                                    &e,
                                    OtapPdata::new(context, saved_payload),
                                ))
                                .await?;
                        }
                    }
                }
                _ => {
                    // Ignore other messages
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use serde_json;

    use arrow::array::{
        ArrayRef, Int32Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray,
        UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
    use std::sync::{Arc, RwLock};

    use bytes::Bytes;
    use geneva_uploader::client::AgentFedCredentialSource;
    use otap_df_engine::Interests;
    use otap_df_engine::capability::auth::BearerToken;
    use otap_df_engine::capability::auth::agent_fed_credential_provider::AgentFedCredentialSnapshot;
    use otap_df_engine::capability::registry::CapabilityRegistry;
    use otap_df_engine::capability::{
        CapabilityError, ExtensionCapability, LocalInstanceFactory, SharedInstanceFactory,
    };
    use otap_df_engine::control::PipelineCompletionMsg;
    use otap_df_engine::extension_capabilities;
    use otap_df_engine::local::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider as LocalAgentFedCredentialProvider;
    use otap_df_engine::shared::capability::auth::agent_fed_credential_provider::AgentFedCredentialProvider as SharedAgentFedCredentialProvider;
    use otap_df_engine::testing::capability::resolve_bindings_for_test;
    use otap_df_engine::testing::exporter::{
        TestRuntime, create_exporter_from_factory, create_test_pipeline_context,
    };
    use otap_df_engine::testing::test_node;
    use otap_df_otap::testing::{TestCallData, next_ack, next_nack};
    use otap_df_pdata::otap::OtapArrowRecords;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::schema::{FieldExt, consts};
    use otap_df_pdata::views::otap::OtapLogsView;
    use otap_df_pdata_views::views::logs::{LogsDataView, ResourceLogsView, ScopeLogsView};
    use std::any::Any;
    use std::collections::HashSet;
    use std::time::{Duration, Instant};

    fn plain_field(name: &str, data_type: DataType, nullable: bool) -> Field {
        Field::new(name, data_type, nullable).with_encoding(consts::metadata::encodings::PLAIN)
    }

    /// Helper to create a simple OTAP logs RecordBatch for testing Geneva exporter
    fn create_test_logs_batch() -> RecordBatch {
        // Define schema matching OTAP logs structure
        let resource_field = Field::new(
            "resource",
            DataType::Struct(vec![plain_field("id", DataType::UInt16, false)].into()),
            false,
        );

        let scope_field = Field::new(
            "scope",
            DataType::Struct(vec![plain_field("id", DataType::UInt16, false)].into()),
            false,
        );

        let schema = Arc::new(Schema::new(vec![
            plain_field("id", DataType::UInt16, false),
            resource_field,
            scope_field,
            Field::new(
                "time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new(
                "observed_time_unix_nano",
                DataType::Timestamp(TimeUnit::Nanosecond, None),
                true,
            ),
            Field::new("severity_number", DataType::Int32, true),
            Field::new("severity_text", DataType::Utf8, true),
            Field::new(
                "body",
                DataType::Struct(
                    vec![
                        Field::new("type", DataType::UInt8, false),
                        Field::new("str", DataType::Utf8, true),
                    ]
                    .into(),
                ),
                true,
            ),
            Field::new("flags", DataType::UInt32, true),
            Field::new("event_name", DataType::Utf8, true),
        ]));

        // Create test data (3 log records)
        let id_array = UInt16Array::from(vec![1, 2, 3]);

        // Resource structs (all from resource_id=1)
        let resource_id_array = UInt16Array::from(vec![1, 1, 1]);
        let resource_struct = StructArray::from(vec![(
            Arc::new(plain_field("id", DataType::UInt16, false)),
            Arc::new(resource_id_array) as ArrayRef,
        )]);

        // Scope structs (logs 1-2 from scope_id=10, log 3 from scope_id=11)
        let scope_id_array = UInt16Array::from(vec![10, 10, 11]);
        let scope_struct = StructArray::from(vec![(
            Arc::new(plain_field("id", DataType::UInt16, false)),
            Arc::new(scope_id_array) as ArrayRef,
        )]);

        let time_array = TimestampNanosecondArray::from(vec![
            Some(1000000000),
            Some(2000000000),
            Some(3000000000),
        ]);

        let observed_time_array = TimestampNanosecondArray::from(vec![
            Some(1000000100),
            Some(2000000100),
            Some(3000000100),
        ]);

        let severity_array = Int32Array::from(vec![Some(9), Some(17), Some(13)]); // INFO, ERROR, WARN
        let severity_text_array =
            StringArray::from(vec![Some("INFO"), Some("ERROR"), Some("WARN")]);

        let body_type_array = UInt8Array::from(vec![1, 1, 1]);
        let body_str_array = StringArray::from(vec![
            Some("Log message 1"),
            Some("Error occurred"),
            Some("Warning message"),
        ]);
        let body_struct = StructArray::from(vec![
            (
                Arc::new(Field::new("type", DataType::UInt8, false)),
                Arc::new(body_type_array) as ArrayRef,
            ),
            (
                Arc::new(Field::new("str", DataType::Utf8, true)),
                Arc::new(body_str_array) as ArrayRef,
            ),
        ]);

        let flags_array = UInt32Array::from(vec![Some(1), Some(1), Some(0)]);
        let event_name_array =
            StringArray::from(vec![Some("event1"), Some("event2"), Some("event3")]);

        RecordBatch::try_new(
            schema,
            vec![
                Arc::new(id_array),
                Arc::new(resource_struct),
                Arc::new(scope_struct),
                Arc::new(time_array),
                Arc::new(observed_time_array),
                Arc::new(severity_array),
                Arc::new(severity_text_array),
                Arc::new(body_struct),
                Arc::new(flags_array),
                Arc::new(event_name_array),
            ],
        )
        .expect("Failed to create test logs batch")
    }

    fn test_config() -> serde_json::Value {
        serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "OtlpLogs"
            },
            "spans": {
                "default_event_name": "OtlpSpans"
            },
            "auth": {
                "type": "systemmanagedidentity",
                "msi_resource": "https://example.com"
            },
            "max_buffer_size": 1000,
            "max_concurrent_uploads": 2
        })
    }

    fn agent_fed_test_config() -> serde_json::Value {
        serde_json::json!({
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "auth": {
                "type": "agentfed"
            },
            "max_buffer_size": 1000,
            "max_concurrent_uploads": 2
        })
    }

    fn agent_fed_node_config(extension: Option<&str>) -> NodeUserConfig {
        let mut node_config = NodeUserConfig::new_exporter_config(GENEVA_EXPORTER_URN);
        node_config.config = agent_fed_test_config();
        if let Some(extension) = extension {
            let _ = node_config.capabilities.insert(
                AgentFedCredentialProviderCap::NAME.into(),
                extension.to_owned().into(),
            );
        }
        node_config
    }

    #[derive(Clone)]
    struct MockAgentExtension {
        snapshot: Arc<RwLock<MockAgentSnapshot>>,
    }

    struct MockAgentSnapshot {
        token: BearerToken,
        attributes: Arc<serde_json::Map<String, serde_json::Value>>,
    }

    #[async_trait]
    impl SharedAgentFedCredentialProvider for MockAgentExtension {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            let snapshot = self.snapshot.read().expect("snapshot read lock");
            Ok(Arc::new(AgentFedCredentialSnapshot::new(
                snapshot.token.clone(),
                Arc::clone(&snapshot.attributes),
            )))
        }
    }

    #[derive(Clone)]
    struct MockLocalAgentExtension;

    #[async_trait(?Send)]
    impl LocalAgentFedCredentialProvider for MockLocalAgentExtension {
        async fn get_credential(&self) -> Result<Arc<AgentFedCredentialSnapshot>, CapabilityError> {
            Ok(Arc::new(AgentFedCredentialSnapshot::new(
                BearerToken::without_expiry("unused-token".to_owned()),
                Arc::new(serde_json::Map::new()),
            )))
        }
    }

    fn resolved_agent_fed_capabilities(
        node_config: &NodeUserConfig,
    ) -> (Capabilities, Arc<RwLock<MockAgentSnapshot>>) {
        let attributes = serde_json::json!({
            "endpoint": "https://ep",
            "moniker_map": { "test-account": "test-moniker" },
        })
        .as_object()
        .cloned()
        .expect("object");
        let snapshot = Arc::new(RwLock::new(MockAgentSnapshot {
            token: BearerToken::without_expiry("test-token".to_owned()),
            attributes: Arc::new(attributes),
        }));
        let extension = MockAgentExtension {
            snapshot: Arc::clone(&snapshot),
        };
        let extension_capabilities = extension_capabilities!(
            shared: MockAgentExtension => [AgentFedCredentialProviderCap]
        );
        let instance_factory =
            SharedInstanceFactory::new(move || Box::new(extension.clone()) as Box<dyn Any + Send>);
        let mut registry = CapabilityRegistry::new();
        (extension_capabilities.register_shared)("agent".into(), instance_factory, &mut registry)
            .expect("register capabilities");
        let known_extensions = HashSet::<otap_df_config::ExtensionId>::from(["agent".into()]);
        let capabilities =
            resolve_bindings_for_test(&node_config.capabilities, &registry, &known_extensions)
                .expect("resolve capabilities");
        (capabilities, snapshot)
    }

    fn resolved_local_only_agent_fed_capabilities(node_config: &NodeUserConfig) -> Capabilities {
        let extension_capabilities = extension_capabilities!(
            local: MockLocalAgentExtension => [AgentFedCredentialProviderCap]
        );
        let instance_factory =
            LocalInstanceFactory::new(|| Box::new(MockLocalAgentExtension) as Box<dyn Any>);
        let mut registry = CapabilityRegistry::new();
        (extension_capabilities.register_local)("agent".into(), instance_factory, &mut registry)
            .expect("register local capabilities");
        let known_extensions = HashSet::<otap_df_config::ExtensionId>::from(["agent".into()]);
        resolve_bindings_for_test(&node_config.capabilities, &registry, &known_extensions)
            .expect("resolve local-only capabilities")
    }

    /// Scenario: The exporter receives an empty OTLP log payload with an ACK subscriber.
    /// Guarantees: The exporter skips upload and returns an empty successful ACK.
    #[test]
    fn geneva_exporter_emits_ack_for_empty_payload() {
        // The Geneva uploader uses rustls (tls-rustls); reqwest needs a
        // process-wide crypto provider, which production installs at startup.
        otap_df_otap::crypto::ensure_crypto_provider();
        let test_runtime = TestRuntime::new();
        let exporter = create_exporter_from_factory(&GENEVA_EXPORTER, test_config()).unwrap();

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                let payload: OtapPayload = OtlpProtoBytes::ExportLogsRequest(Bytes::new()).into();
                let pdata = OtapPdata::new_default(payload).test_subscribe_to(
                    Interests::ACKS,
                    TestCallData::default().into(),
                    4242,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test shutdown")
                    .await
                    .unwrap();
            })
            .run_validation(|mut ctx, result| async move {
                result.expect("success");

                let mut pipeline_rx = ctx.take_pipeline_completion_receiver().unwrap();
                loop {
                    match pipeline_rx.recv().await.unwrap() {
                        PipelineCompletionMsg::DeliverAck { ack } => {
                            let (node_id, ack) = next_ack(ack).expect("expected ack subscriber");
                            assert_eq!(node_id, 4242);
                            let got: TestCallData = ack.unwind.route.calldata.try_into().unwrap();
                            assert_eq!(TestCallData::default(), got);
                            assert_eq!(ack.accepted.num_items(), 0);
                            break;
                        }
                        _ => continue, // Skip non-Ack messages (e.g. StartTelemetryTimer)
                    }
                }
            });
    }

    /// Scenario: The exporter receives malformed non-empty OTLP log bytes.
    /// Guarantees: Decode failure returns a NACK with the original subscriber route.
    #[test]
    fn geneva_exporter_emits_nack_for_decode_failure() {
        // The Geneva uploader uses rustls (tls-rustls); reqwest needs a
        // process-wide crypto provider, which production installs at startup.
        otap_df_otap::crypto::ensure_crypto_provider();
        let test_runtime = TestRuntime::new();
        let exporter = create_exporter_from_factory(&GENEVA_EXPORTER, test_config()).unwrap();

        test_runtime
            .set_exporter(exporter)
            .run_test(|ctx| async move {
                // Non-empty but invalid protobuf bytes to trigger decode error (no network).
                let payload: OtapPayload =
                    OtlpProtoBytes::ExportLogsRequest(Bytes::from_static(b"\xff")).into();
                let pdata = OtapPdata::new_default(payload).test_subscribe_to(
                    Interests::NACKS,
                    TestCallData::default().into(),
                    777,
                );
                ctx.send_pdata(pdata).await.unwrap();
                ctx.send_shutdown(Instant::now() + Duration::from_secs(1), "test shutdown")
                    .await
                    .unwrap();
            })
            .run_validation(|mut ctx, result| async move {
                result.expect("success");

                let mut pipeline_rx = ctx.take_pipeline_completion_receiver().unwrap();
                loop {
                    match pipeline_rx.recv().await.unwrap() {
                        PipelineCompletionMsg::DeliverNack { nack } => {
                            let (node_id, nack) =
                                next_nack(nack).expect("expected nack subscriber");
                            assert_eq!(node_id, 777);
                            let got: TestCallData = nack.unwind.route.calldata.try_into().unwrap();
                            assert_eq!(TestCallData::default(), got);
                            assert!(
                                nack.reason.contains("Failed to decode logs request"),
                                "unexpected nack reason: {}",
                                nack.reason
                            );
                            assert_eq!(nack.refused.num_items(), 0);
                            break;
                        }
                        _ => continue, // Skip non-Nack messages (e.g. StartTelemetryTimer)
                    }
                }
            });
    }

    /// Scenario: The exporter receives a representative OTAP logs record batch.
    /// Guarantees: The logs view exposes all records after transport ID decoding.
    #[test]
    fn test_geneva_exporter_creates_view_from_otap_records() {
        // This test verifies that the Geneva exporter can successfully create
        // an OtapLogsView from OtapArrowRecords using the TryFrom implementation.

        let logs_batch = create_test_logs_batch();

        // Create OtapArrowRecords (simulating what batch processor would send)
        let mut otap_records = OtapArrowRecords::Logs(Default::default());
        otap_records
            .set(ArrowPayloadType::Logs, logs_batch)
            .expect("set logs batch");

        // This is what the Geneva exporter does internally after transport decode.
        otap_records
            .decode_transport_optimized_ids()
            .expect("decode transport IDs");
        let logs_view = OtapLogsView::try_from(&otap_records)
            .expect("Geneva exporter should create view from OTAP records");

        // Verify the view can be used (basic sanity check)
        let mut log_count = 0;
        for resource_logs in logs_view.resources() {
            for scope_logs in resource_logs.scopes() {
                for _log_record in scope_logs.log_records() {
                    log_count += 1;
                }
            }
        }

        assert_eq!(log_count, 3, "Expected 3 logs");
    }

    // Configuration tests
    /// Scenario: A complete certificate-auth configuration is deserialized.
    /// Guarantees: Required fields, defaults, and auth data retain their values.
    #[test]
    fn test_config_deserialization() {
        let json = serde_json::json!({
            "endpoint": "https://geneva.example.com",
            "environment": "production",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "westus2",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "OtlpLogs"
            },
            "spans": {
                "default_event_name": "OtlpSpans"
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let config: Config = serde_json::from_value(json).unwrap();

        // Assert all config fields
        assert_eq!(config.endpoint, "https://geneva.example.com");
        assert_eq!(config.environment, "production");
        assert_eq!(config.account, "test-account");
        assert_eq!(config.namespace, "test-namespace");
        assert_eq!(config.region, "westus2");
        assert_eq!(config.config_major_version, 1);
        assert_eq!(config.tenant, "test-tenant");
        assert_eq!(config.role_name, "test-role");
        assert_eq!(config.role_instance, "test-instance");
        assert_eq!(
            config
                .logs
                .as_ref()
                .and_then(|l| l.default_event_name.as_deref()),
            Some("OtlpLogs")
        );
        assert_eq!(
            config
                .spans
                .as_ref()
                .and_then(|s| s.default_event_name.as_deref()),
            Some("OtlpSpans")
        );
        assert_eq!(config.max_buffer_size, 1000); // default
        assert_eq!(config.max_concurrent_uploads, 4); // default

        // Assert auth config
        match config.auth {
            AuthConfig::Certificate { path, password } => {
                assert_eq!(path, "/path/to/cert.p12");
                assert_eq!(password, "secret");
            }
            _ => panic!("Expected Certificate auth variant"),
        }
    }

    /// Scenario: Certificate authentication is configured without its opt-in build feature.
    /// Guarantees: Configuration validation rejects the unsupported authentication mode early.
    #[cfg(not(feature = "geneva-certificate-auth"))]
    #[test]
    fn certificate_auth_requires_opt_in_feature() {
        let config = serde_json::json!({
            "endpoint": "https://geneva.example.com",
            "environment": "production",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "westus2",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let error = Config::parse(&config).expect_err("certificate auth should be disabled");
        assert!(
            error
                .to_string()
                .contains("requires the 'geneva-certificate-auth' build feature")
        );
    }

    /// Scenario: Certificate authentication is configured with its opt-in build feature.
    /// Guarantees: Configuration validation accepts the certificate authentication mode.
    #[cfg(feature = "geneva-certificate-auth")]
    #[test]
    fn certificate_auth_is_accepted_when_feature_is_enabled() {
        let config = serde_json::json!({
            "endpoint": "https://geneva.example.com",
            "environment": "production",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "westus2",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed = Config::parse(&config).expect("certificate auth should be enabled");
        assert!(matches!(parsed.auth, AuthConfig::Certificate { .. }));
    }

    /// Scenario: Agent-fed configuration omits GCS-only endpoint and region fields.
    /// Guarantees: The configuration remains valid and preserves the agent-fed mode.
    #[test]
    fn agent_fed_config_can_omit_gcs_endpoint_and_region() {
        let config = Config::parse(&agent_fed_test_config()).expect("valid agent-fed config");
        assert!(config.endpoint.is_empty());
        assert!(config.region.is_empty());
        assert!(matches!(config.auth, AuthConfig::AgentFed));
    }

    /// Scenario: Agent-fed authentication is configured with log and span table routing.
    /// Guarantees: The uploader adapter preserves both routing sections in agent-fed mode.
    #[test]
    fn agent_fed_config_preserves_logs_and_spans_routing() {
        let mut config = agent_fed_test_config();
        config["logs"] = serde_json::json!({
            "default_event_name": "AgentLogs",
            "event_name_mapping": {
                "routing_key": { "log_record_attribute": "log.kind" },
                "events": { "audit": "AuditLogs" }
            }
        });
        config["spans"] = serde_json::json!({
            "default_event_name": "AgentSpans",
            "event_name_mapping": {
                "routing_key": { "span_attribute": "span.kind" },
                "events": { "SERVER": "ServerSpans" }
            }
        });

        let parsed = Config::parse(&config).expect("valid combined agent-fed config");
        assert!(matches!(parsed.auth, AuthConfig::AgentFed));

        let client_config = parsed.to_geneva_client_config();
        assert!(client_config.endpoint.is_empty());
        assert!(client_config.region.is_empty());
        assert!(client_config.msi_resource.is_none());

        let logs = client_config.logs.expect("logs config should be present");
        assert_eq!(logs.default_event_name.as_deref(), Some("AgentLogs"));
        let logs_mapping = logs
            .event_name_mapping
            .expect("logs mapping should be present");
        assert!(matches!(
            logs_mapping.routing_key,
            LogsEventNameRoutingKey::LogRecordAttribute(ref key) if key == "log.kind"
        ));
        assert_eq!(
            logs_mapping.events.get("audit"),
            Some(&Some("AuditLogs".to_owned()))
        );

        let spans = client_config.spans.expect("spans config should be present");
        assert_eq!(spans.default_event_name.as_deref(), Some("AgentSpans"));
        let spans_mapping = spans
            .event_name_mapping
            .expect("spans mapping should be present");
        assert!(matches!(
            spans_mapping.routing_key,
            SpanEventNameRoutingKey::SpanAttribute(ref key) if key == "span.kind"
        ));
        assert_eq!(
            spans_mapping.events.get("SERVER"),
            Some(&Some("ServerSpans".to_owned()))
        );
    }

    /// Scenario: A non-agent-fed configuration omits endpoint or region.
    /// Guarantees: Existing authentication modes continue to require both fields.
    #[test]
    fn non_agent_fed_config_requires_endpoint_and_region() {
        let mut missing_endpoint = test_config();
        let _ = missing_endpoint
            .as_object_mut()
            .expect("object")
            .remove("endpoint");
        let error = Config::parse(&missing_endpoint).expect_err("endpoint must be required");
        assert!(error.to_string().contains("endpoint is required"));

        let mut missing_region = test_config();
        let _ = missing_region
            .as_object_mut()
            .expect("object")
            .remove("region");
        let error = Config::parse(&missing_region).expect_err("region must be required");
        assert!(error.to_string().contains("region is required"));
    }

    /// Scenario: A configuration carries a field the exporter does not define.
    /// Guarantees: Unknown configuration fields stay rejected after validation
    /// moved out of the `Deserialize` implementation.
    #[test]
    fn unknown_config_fields_are_rejected() {
        let mut config = test_config();
        config["unexpected_field"] = serde_json::Value::Bool(true);

        let error = Config::parse(&config).expect_err("unknown fields must be rejected");
        assert!(error.to_string().contains("unexpected_field"));
    }

    /// Scenario: The required agent-fed credential-provider binding is absent.
    /// Guarantees: Startup reports the missing combined capability.
    #[test]
    fn agent_fed_credential_provider_binding_is_required() {
        let node_config = agent_fed_node_config(None);
        let error = validate_agent_fed_capability_binding(&node_config)
            .expect_err("credential-provider binding must be required");
        assert!(error.to_string().contains("agent_fed_credential_provider"));

        let node_config = agent_fed_node_config(Some("agent"));
        validate_agent_fed_capability_binding(&node_config)
            .expect("configured credential-provider binding must pass");
    }

    /// Scenario: The agent-fed capability binding is missing before TLS setup is checked.
    /// Guarantees: Binding validation short-circuits without invoking the crypto check.
    #[test]
    fn agent_fed_binding_validation_precedes_crypto_check() {
        let node_config = agent_fed_node_config(None);
        let config = Config::parse(&node_config.config).expect("valid agent-fed config");
        let error = validate_geneva_client_prerequisites(&config, &node_config, || {
            panic!("crypto provider check must not run before binding validation")
        })
        .expect_err("missing credential-provider binding must fail");

        assert!(error.to_string().contains("agent_fed_credential_provider"));
    }

    /// Scenario: The factory receives no agent-fed credential-provider binding.
    /// Guarantees: Exporter creation fails with the missing capability error.
    #[test]
    fn agent_fed_factory_fails_closed_without_credential_provider() {
        let exporter_config = ExporterConfig::new("test-exporter");

        let missing_bindings = (GENEVA_EXPORTER.create)(
            create_test_pipeline_context(),
            test_node("test-exporter".to_owned()),
            Arc::new(agent_fed_node_config(None)),
            &exporter_config,
            &Capabilities::empty(),
        );
        let error = match missing_bindings {
            Ok(_) => panic!("missing bindings must fail exporter creation"),
            Err(error) => error,
        };
        assert!(error.to_string().contains("agent_fed_credential_provider"));
    }

    /// Scenario: The config-only constructor receives agent-fed authentication.
    /// Guarantees: Construction fails closed because no capability binding can be resolved.
    #[test]
    fn agent_fed_config_only_constructor_requires_factory_binding() {
        let result =
            GenevaExporter::from_config(create_test_pipeline_context(), &agent_fed_test_config());
        let error = match result {
            Ok(_) => panic!("config-only agent-fed construction must fail"),
            Err(error) => error,
        };

        assert!(error.to_string().contains("agent_fed_credential_provider"));
    }

    /// Scenario: A node binds agent-fed credentials through a local-only extension.
    /// Guarantees: Source creation rejects the binding and identifies the shared requirement.
    #[test]
    fn agent_fed_source_requires_shared_capability_registration() {
        let node_config = agent_fed_node_config(Some("agent"));
        validate_agent_fed_capability_binding(&node_config).expect("binding should be present");
        let capabilities = resolved_local_only_agent_fed_capabilities(&node_config);
        let error = resolve_agent_fed_source(&capabilities)
            .expect_err("shared capability implementation must be required");

        assert!(
            error
                .to_string()
                .contains("local-only registrations are unsupported")
        );
    }

    /// Scenario: Registry-created credential snapshots follow a rotating host snapshot.
    /// Guarantees: Every result contains a coherent token and routing pair from one generation.
    #[tokio::test]
    async fn agent_fed_credential_provider_rotates_coherently() {
        let node_config = agent_fed_node_config(Some("agent"));
        let (capabilities, snapshot) = resolved_agent_fed_capabilities(&node_config);
        let credential_provider = capabilities
            .require_shared::<AgentFedCredentialProviderCap>()
            .expect("agent-fed credential provider");
        let source = AgentFedGenevaSource::new(credential_provider);

        let initial = source.current().await.expect("initial credential");
        assert_eq!(initial.expose_token(), "test-token");
        assert_eq!(initial.endpoint, "https://ep/");
        assert_eq!(
            initial
                .primary_monikers
                .get("test-account")
                .map(String::as_str),
            Some("test-moniker")
        );

        let rotated_attributes = serde_json::json!({
            "endpoint": "https://rotated-ep",
            "moniker_map": { "test-account": "rotated-moniker" },
        })
        .as_object()
        .cloned()
        .expect("object");
        {
            let mut snapshot = snapshot.write().expect("snapshot write lock");
            snapshot.token = BearerToken::without_expiry("rotated-token".to_owned());
            snapshot.attributes = Arc::new(rotated_attributes);
        }

        let rotated = source.current().await.expect("rotated credential");
        assert_eq!(rotated.expose_token(), "rotated-token");
        assert_eq!(rotated.endpoint, "https://rotated-ep/");
        assert_eq!(
            rotated
                .primary_monikers
                .get("test-account")
                .map(String::as_str),
            Some("rotated-moniker")
        );
    }

    /// Scenario: A valid binding resolves the combined capability from a shared extension.
    /// Guarantees: The factory constructs an agent-fed exporter successfully.
    #[test]
    fn creates_agent_fed_exporter_with_bound_capabilities() {
        otap_df_otap::crypto::ensure_crypto_provider();
        let node_config = agent_fed_node_config(Some("agent"));
        let (capabilities, _snapshot) = resolved_agent_fed_capabilities(&node_config);
        let exporter_config = ExporterConfig::new("test-exporter");
        let result = (GENEVA_EXPORTER.create)(
            create_test_pipeline_context(),
            test_node("test-exporter".to_owned()),
            Arc::new(node_config),
            &exporter_config,
            &capabilities,
        );
        assert!(result.is_ok(), "agent-fed exporter should initialize");
    }

    /// Scenario: Every supported authentication tag is deserialized.
    /// Guarantees: Serde maps each public tag to the correct auth variant.
    #[test]
    fn test_auth_config_variants() {
        let cert_json = serde_json::json!({
            "type": "certificate",
            "path": "/path/to/cert.p12",
            "password": "secret"
        });
        let cert_auth: AuthConfig = serde_json::from_value(cert_json).unwrap();
        assert!(matches!(cert_auth, AuthConfig::Certificate { .. }));

        let system_mi_json = serde_json::json!({
            "type": "systemmanagedidentity",
            "msi_resource": "https://resource"
        });
        let system_mi: AuthConfig = serde_json::from_value(system_mi_json).unwrap();
        assert!(matches!(
            system_mi,
            AuthConfig::SystemManagedIdentity { .. }
        ));

        let user_mi_json = serde_json::json!({
            "type": "usermanagedidentity",
            "client_id": "my-client-id",
            "msi_resource": "https://resource"
        });
        let user_mi: AuthConfig = serde_json::from_value(user_mi_json).unwrap();
        assert!(matches!(user_mi, AuthConfig::UserManagedIdentity { .. }));

        let user_mi_resid_json = serde_json::json!({
            "type": "usermanagedidentitybyarmresourceid",
            "resource_id": "/subscriptions/sub1/resourceGroups/rg1/providers/Microsoft.Kubernetes/extensions/ext1",
            "msi_resource": "https://monitor.core.windows.net"
        });
        let user_mi_resid: AuthConfig = serde_json::from_value(user_mi_resid_json).unwrap();
        match user_mi_resid {
            AuthConfig::UserManagedIdentityByArmResourceId {
                resource_id,
                msi_resource,
            } => {
                assert!(resource_id.contains("sub1"));
                assert_eq!(msi_resource, "https://monitor.core.windows.net");
            }
            _ => panic!("Expected UserManagedIdentityByArmResourceId auth variant"),
        }

        let workload_json = serde_json::json!({
            "type": "workloadidentity",
            "msi_resource": "https://resource"
        });
        let workload: AuthConfig = serde_json::from_value(workload_json).unwrap();
        assert!(matches!(workload, AuthConfig::WorkloadIdentity { .. }));

        let agent_fed_json = serde_json::json!({
            "type": "agentfed"
        });
        let agent_fed: AuthConfig = serde_json::from_value(agent_fed_json).unwrap();
        assert!(matches!(agent_fed, AuthConfig::AgentFed));
    }

    /// Scenario: Registration code reads the Geneva exporter URN constant.
    /// Guarantees: The public component identifier remains stable.
    #[test]
    fn test_urn_constant() {
        assert_eq!(GENEVA_EXPORTER_URN, "urn:microsoft:exporter:geneva");
    }

    /// Scenario: A legacy caller uses the config-only constructor with ARM resource-ID auth.
    /// Guarantees: The original public constructor remains source-compatible and succeeds.
    #[test]
    fn create_exporter_with_user_managed_identity_by_arm_resource_id() {
        // The Geneva uploader uses rustls (tls-rustls); reqwest needs a
        // process-wide crypto provider, which production installs at startup.
        otap_df_otap::crypto::ensure_crypto_provider();
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "OtlpLogs"
            },
            "spans": {
                "default_event_name": "OtlpSpans"
            },
            "auth": {
                "type": "usermanagedidentitybyarmresourceid",
                "resource_id": "/subscriptions/sub1/resourceGroups/rg1/providers/Microsoft.Kubernetes/extensions/ext1",
                "msi_resource": "https://monitor.core.windows.net"
            },
            "max_buffer_size": 1000,
            "max_concurrent_uploads": 2
        });
        let exporter = GenevaExporter::from_config(create_test_pipeline_context(), &config);
        assert!(
            exporter.is_ok(),
            "Exporter should initialise with UserManagedIdentityByArmResourceId auth"
        );
    }

    /// Scenario: a `logs.event_name_mapping` uses the string short-form
    /// `routing_key: "event_name"`.
    /// Guarantees: it deserializes to the `EventName` routing-key variant and
    /// the mapping is present on the parsed logs config.
    #[test]
    fn test_routing_key_event_name_variant() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": "event_name",
                    "events": {
                        "test1": "Test1"
                    }
                }
            },
            "spans": {
                "default_event_name": "Span"
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed_config: Config = serde_json::from_value(config).unwrap();
        let logs = parsed_config
            .logs
            .as_ref()
            .expect("logs should be configured");
        assert!(logs.event_name_mapping.is_some());

        let mapping = logs.event_name_mapping.as_ref().unwrap();
        match &mapping.routing_key {
            LogsEventNameRoutingKeyConfig::EventName => {
                // Expected
            }
            _ => panic!("Expected EventName variant"),
        }
    }

    /// Scenario: a `logs.event_name_mapping.routing_key` is the map form
    /// `{ scope_attribute: "scope.name" }`.
    /// Guarantees: it deserializes to the `ScopeAttribute` variant carrying the
    /// exact attribute name supplied.
    #[test]
    fn test_routing_key_scope_attribute_variant() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": {
                        "scope_attribute": "scope.name"
                    },
                    "events": {
                        "test1": "Test1",
                        "test2": "Test2"
                    }
                }
            },
            "spans": {
                "default_event_name": "Span"
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed_config: Config = serde_json::from_value(config).unwrap();
        let logs = parsed_config
            .logs
            .as_ref()
            .expect("logs should be configured");
        assert!(logs.event_name_mapping.is_some());

        let mapping = logs.event_name_mapping.as_ref().unwrap();
        match &mapping.routing_key {
            LogsEventNameRoutingKeyConfig::ScopeAttribute { scope_attribute } => {
                assert_eq!(scope_attribute, "scope.name");
            }
            _ => panic!("Expected ScopeAttribute variant"),
        }
    }

    /// Scenario: a logs `routing_key` map specifies more than one attribute
    /// field at once.
    /// Guarantees: deserialization fails with an error naming the "exactly one
    /// field" requirement, so an ambiguous routing key is never accepted.
    #[test]
    fn test_routing_key_validation_rejects_multiple_fields() {
        // Test that providing multiple routing_key fields is rejected for logs
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": {
                        "scope_attribute": "scope.name",
                        "resource_attribute": "resource.id"
                    },
                    "events": {
                        "test1": "Test1"
                    }
                }
            },
            "spans": {
                "default_event_name": "Span"
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "Should reject multiple routing_key fields");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("exactly one field"),
            "Error should mention exactly one field requirement: {}",
            error_msg
        );
    }

    /// Scenario: a `spans.event_name_mapping.routing_key` is
    /// `{ resource_attribute: ... }` with both a mapped and a null (passthrough)
    /// destination in `events`.
    /// Guarantees: it deserializes to the `ResourceAttribute` variant and the
    /// events map preserves both the explicit destination and the null value.
    #[test]
    fn test_span_routing_key_resource_attribute_variant() {
        // Test that resource_attribute variant deserializes correctly
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "event_name_mapping": {
                    "routing_key": {
                        "resource_attribute": "resource.cluster"
                    },
                    "events": {
                        "clusterA": "PremiumSpan",
                        "clusterB": null
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_ok(),
            "Should deserialize valid span resource_attribute config"
        );

        let config = result.unwrap();
        assert!(config.spans.is_some(), "Config should have spans section");

        let spans = config.spans.unwrap();
        assert_eq!(spans.default_event_name, Some("Span".to_string()));

        let mapping = spans.event_name_mapping.as_ref().unwrap();
        match &mapping.routing_key {
            SpansEventNameRoutingKeyConfig::ResourceAttribute { resource_attribute } => {
                assert_eq!(resource_attribute, "resource.cluster");
            }
            _ => panic!("Expected ResourceAttribute routing key"),
        }

        // Verify events map
        assert_eq!(
            mapping.events.get("clusterA"),
            Some(&Some("PremiumSpan".to_string()))
        );
        assert_eq!(mapping.events.get("clusterB"), Some(&None));
    }

    /// Scenario: a `spans.event_name_mapping.routing_key` is
    /// `{ scope_attribute: ... }`.
    /// Guarantees: it deserializes to the `ScopeAttribute` variant carrying the
    /// exact attribute name supplied.
    #[test]
    fn test_span_routing_key_scope_attribute_variant() {
        // Test that scope_attribute variant deserializes correctly
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "event_name_mapping": {
                    "routing_key": {
                        "scope_attribute": "instrumentation.name"
                    },
                    "events": {
                        "otel-sdk": "SDKSpan",
                        "custom": null
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_ok(),
            "Should deserialize valid span scope_attribute config"
        );

        let config = result.unwrap();
        let spans = config.spans.unwrap();
        let mapping = spans.event_name_mapping.as_ref().unwrap();

        match &mapping.routing_key {
            SpansEventNameRoutingKeyConfig::ScopeAttribute { scope_attribute } => {
                assert_eq!(scope_attribute, "instrumentation.name");
            }
            _ => panic!("Expected ScopeAttribute routing key"),
        }
    }

    /// Scenario: a `spans.event_name_mapping.routing_key` is
    /// `{ span_attribute: ... }`.
    /// Guarantees: it deserializes to the `SpanAttribute` variant and the events
    /// map preserves each destination table name.
    #[test]
    fn test_span_routing_key_span_attribute_variant() {
        // Test that span_attribute variant deserializes correctly
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "event_name_mapping": {
                    "routing_key": {
                        "span_attribute": "span.kind"
                    },
                    "events": {
                        "SERVER": "ServerSpan",
                        "CLIENT": "ClientSpan"
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_ok(),
            "Should deserialize valid span_attribute config"
        );

        let config = result.unwrap();
        let spans = config.spans.unwrap();
        let mapping = spans.event_name_mapping.as_ref().unwrap();

        match &mapping.routing_key {
            SpansEventNameRoutingKeyConfig::SpanAttribute { span_attribute } => {
                assert_eq!(span_attribute, "span.kind");
            }
            _ => panic!("Expected SpanAttribute routing key"),
        }

        assert_eq!(
            mapping.events.get("SERVER"),
            Some(&Some("ServerSpan".to_string()))
        );
        assert_eq!(
            mapping.events.get("CLIENT"),
            Some(&Some("ClientSpan".to_string()))
        );
    }

    /// Scenario: a spans `routing_key` map specifies more than one attribute
    /// field at once.
    /// Guarantees: deserialization fails with an error naming the "only one of"
    /// requirement, so an ambiguous span routing key is never accepted.
    #[test]
    fn test_span_routing_key_validation_rejects_multiple_fields() {
        // Test that providing multiple routing_key fields is rejected
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "event_name_mapping": {
                    "routing_key": {
                        "resource_attribute": "resource.id",
                        "span_attribute": "span.kind"
                    },
                    "events": {
                        "test1": "Test1"
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject multiple span routing_key fields"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("only one of"),
            "Error should mention exactly one field requirement: {}",
            error_msg
        );
    }

    /// Scenario: a logs `routing_key` attribute name is the empty string.
    /// Guarantees: deserialization fails with an error requiring a non-empty
    /// attribute name, so a blank routing attribute is never accepted.
    #[test]
    fn test_logs_routing_key_rejects_empty_attribute() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": {
                        "resource_attribute": ""
                    },
                    "events": {
                        "test1": "Test1"
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "Should reject empty logs attribute name");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("non-empty attribute name"),
            "Error should mention non-empty attribute requirement: {}",
            error_msg
        );
    }

    /// Scenario: a spans `routing_key` attribute name is whitespace-only.
    /// Guarantees: deserialization fails with an error requiring a non-empty
    /// attribute name, so a blank/whitespace routing attribute is never accepted.
    #[test]
    fn test_span_routing_key_rejects_empty_attribute() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "event_name_mapping": {
                    "routing_key": {
                        "span_attribute": "   "
                    },
                    "events": {
                        "test1": "Test1"
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "Should reject empty span attribute name");
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("non-empty attribute name"),
            "Error should mention non-empty attribute requirement: {}",
            error_msg
        );
    }

    /// Scenario: a `logs.event_name_mapping.routing_key` is
    /// `{ resource_attribute: ... }`.
    /// Guarantees: it deserializes to the `ResourceAttribute` variant carrying
    /// the exact attribute name supplied.
    #[test]
    fn test_logs_routing_key_resource_attribute_variant() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": {
                        "resource_attribute": "cluster"
                    },
                    "events": {
                        "clusterA": "PremiumLog"
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).unwrap();
        let mapping = parsed
            .logs
            .as_ref()
            .and_then(|l| l.event_name_mapping.as_ref())
            .expect("logs mapping should be configured");
        match &mapping.routing_key {
            LogsEventNameRoutingKeyConfig::ResourceAttribute { resource_attribute } => {
                assert_eq!(resource_attribute, "cluster");
            }
            _ => panic!("Expected ResourceAttribute variant"),
        }
    }

    /// Scenario: a `logs.event_name_mapping.routing_key` is
    /// `{ log_record_attribute: ... }`.
    /// Guarantees: it deserializes to the `LogRecordAttribute` variant carrying
    /// the exact attribute name supplied.
    #[test]
    fn test_logs_routing_key_log_record_attribute_variant() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": {
                        "log_record_attribute": "custom_event_name"
                    },
                    "events": {
                        "test1": null
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).unwrap();
        let mapping = parsed
            .logs
            .as_ref()
            .and_then(|l| l.event_name_mapping.as_ref())
            .expect("logs mapping should be configured");
        match &mapping.routing_key {
            LogsEventNameRoutingKeyConfig::LogRecordAttribute {
                log_record_attribute,
            } => {
                assert_eq!(log_record_attribute, "custom_event_name");
            }
            _ => panic!("Expected LogRecordAttribute variant"),
        }
    }

    /// Scenario: a parsed logs mapping (scope.name key, with a mapped and a null
    /// destination) is converted via `Into` to the uploader `LogsEventNameMapping`.
    /// Guarantees: the conversion preserves the routing key and both the explicit
    /// destination and the null (passthrough) value in the events map.
    #[test]
    fn test_logs_mapping_converts_to_uploader_type() {
        // Verify the config wrapper converts into the geneva-uploader
        // LogsEventNameMapping, preserving the reserved `scope.name` routing key
        // and null (passthrough) destination values.
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": {
                        "scope_attribute": "scope.name"
                    },
                    "events": {
                        "mapped": "DestTable",
                        "passthrough": null
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).unwrap();
        let mapping_config = parsed
            .logs
            .and_then(|l| l.event_name_mapping)
            .expect("logs mapping should be configured");

        let uploader_mapping: LogsEventNameMapping = mapping_config.into();
        match &uploader_mapping.routing_key {
            LogsEventNameRoutingKey::ScopeAttribute(key) => assert_eq!(key, "scope.name"),
            other => panic!("Expected ScopeAttribute routing key, got {other:?}"),
        }
        assert_eq!(
            uploader_mapping.events.get("mapped"),
            Some(&Some("DestTable".to_string()))
        );
        assert_eq!(uploader_mapping.events.get("passthrough"), Some(&None));
    }

    /// Scenario: a parsed spans mapping (span.kind key, with a mapped and a null
    /// destination) is converted via `Into` to the uploader `SpanEventNameMapping`.
    /// Guarantees: the conversion preserves the routing key and both the explicit
    /// destination and the null (passthrough) value in the events map.
    #[test]
    fn test_spans_mapping_converts_to_uploader_type() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "event_name_mapping": {
                    "routing_key": {
                        "span_attribute": "span.kind"
                    },
                    "events": {
                        "SERVER": "ServerSpan",
                        "CLIENT": null
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).unwrap();
        let mapping_config = parsed
            .spans
            .and_then(|s| s.event_name_mapping)
            .expect("spans mapping should be configured");

        let uploader_mapping: SpanEventNameMapping = mapping_config.into();
        match &uploader_mapping.routing_key {
            SpanEventNameRoutingKey::SpanAttribute(key) => assert_eq!(key, "span.kind"),
            other => panic!("Expected SpanAttribute routing key, got {other:?}"),
        }
        assert_eq!(
            uploader_mapping.events.get("SERVER"),
            Some(&Some("ServerSpan".to_string()))
        );
        assert_eq!(uploader_mapping.events.get("CLIENT"), Some(&None));
    }

    /// Scenario: every `LogsEventNameRoutingKeyConfig` variant (EventName,
    /// ResourceAttribute, ScopeAttribute, LogRecordAttribute) is passed through
    /// the `From` conversion to the uploader routing-key type.
    /// Guarantees: each config variant maps to its corresponding uploader variant
    /// with the attribute name carried through unchanged.
    #[test]
    fn test_logs_routing_key_from_conversion_all_variants() {
        assert!(matches!(
            LogsEventNameRoutingKey::from(LogsEventNameRoutingKeyConfig::EventName),
            LogsEventNameRoutingKey::EventName
        ));
        assert!(matches!(
            LogsEventNameRoutingKey::from(LogsEventNameRoutingKeyConfig::ResourceAttribute {
                resource_attribute: "r".to_string()
            }),
            LogsEventNameRoutingKey::ResourceAttribute(k) if k == "r"
        ));
        assert!(matches!(
            LogsEventNameRoutingKey::from(LogsEventNameRoutingKeyConfig::ScopeAttribute {
                scope_attribute: "s".to_string()
            }),
            LogsEventNameRoutingKey::ScopeAttribute(k) if k == "s"
        ));
        assert!(matches!(
            LogsEventNameRoutingKey::from(LogsEventNameRoutingKeyConfig::LogRecordAttribute {
                log_record_attribute: "l".to_string()
            }),
            LogsEventNameRoutingKey::LogRecordAttribute(k) if k == "l"
        ));
    }

    /// Scenario: every `SpansEventNameRoutingKeyConfig` variant
    /// (ResourceAttribute, ScopeAttribute, SpanAttribute) is passed through the
    /// `From` conversion to the uploader routing-key type.
    /// Guarantees: each config variant maps to its corresponding uploader variant
    /// with the attribute name carried through unchanged.
    #[test]
    fn test_span_routing_key_from_conversion_all_variants() {
        assert!(matches!(
            SpanEventNameRoutingKey::from(SpansEventNameRoutingKeyConfig::ResourceAttribute {
                resource_attribute: "r".to_string()
            }),
            SpanEventNameRoutingKey::ResourceAttribute(k) if k == "r"
        ));
        assert!(matches!(
            SpanEventNameRoutingKey::from(SpansEventNameRoutingKeyConfig::ScopeAttribute {
                scope_attribute: "s".to_string()
            }),
            SpanEventNameRoutingKey::ScopeAttribute(k) if k == "s"
        ));
        assert!(matches!(
            SpanEventNameRoutingKey::from(SpansEventNameRoutingKeyConfig::SpanAttribute {
                span_attribute: "sp".to_string()
            }),
            SpanEventNameRoutingKey::SpanAttribute(k) if k == "sp"
        ));
    }

    /// Scenario: a config omits both the `logs` and `spans` sections entirely.
    /// Guarantees: both parse to `None`, letting the uploader fall back to the
    /// default "Log"/"Span" tables.
    #[test]
    fn test_logs_and_spans_optional_default_to_none() {
        // Omitting the logs/spans sections yields None, which the uploader maps
        // to the default "Log"/"Span" tables.
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).unwrap();
        assert!(parsed.logs.is_none(), "logs should default to None");
        assert!(parsed.spans.is_none(), "spans should default to None");
    }

    /// Scenario: A `logs` block sets `default_event_name` to an empty string.
    /// Guarantees: Deserialization fails so a blank table name can never silently
    /// override the uploader's default table.
    #[test]
    fn test_logs_default_event_name_rejects_empty() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": ""
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject empty logs default_event_name"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("non-empty table name"),
            "Error should mention non-empty table name requirement: {}",
            error_msg
        );
    }

    /// Scenario: A `spans` block sets `default_event_name` to a whitespace-only
    /// string.
    /// Guarantees: Deserialization fails, so whitespace is treated as blank and
    /// cannot override the uploader's default table.
    #[test]
    fn test_spans_default_event_name_rejects_whitespace() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "   "
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject whitespace-only spans default_event_name"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("non-empty table name"),
            "Error should mention non-empty table name requirement: {}",
            error_msg
        );
    }

    /// Scenario: A `logs` block contains a misspelled field
    /// (`event_name_mappings`, plural) alongside valid fields.
    /// Guarantees: `deny_unknown_fields` rejects the typo instead of silently
    /// dropping the routing configuration.
    #[test]
    fn test_logs_config_rejects_unknown_field() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mappings": {
                    "routing_key": "event_name",
                    "events": { "test1": "Test1" }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject unknown field in logs config"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("event_name_mappings") || error_msg.contains("unknown field"),
            "Error should mention the unknown field: {}",
            error_msg
        );
    }

    /// Scenario: A logs `event_name_mapping` block contains an unknown field
    /// alongside the required `routing_key` and `events`.
    /// Guarantees: `deny_unknown_fields` on the mapping wrapper rejects the typo.
    #[test]
    fn test_logs_event_name_mapping_rejects_unknown_field() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": "event_name",
                    "events": { "test1": "Test1" },
                    "unexpected": "value"
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject unknown field in logs event_name_mapping"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("unexpected") || error_msg.contains("unknown field"),
            "Error should mention the unknown field: {}",
            error_msg
        );
    }

    /// Scenario: A spans `event_name_mapping` block contains an unknown field
    /// alongside the required `routing_key` and `events`.
    /// Guarantees: `deny_unknown_fields` on the span mapping wrapper rejects the
    /// typo.
    #[test]
    fn test_spans_event_name_mapping_rejects_unknown_field() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "event_name_mapping": {
                    "routing_key": { "span_attribute": "span.kind" },
                    "events": { "SERVER": "ServerSpan" },
                    "unexpected": "value"
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject unknown field in spans event_name_mapping"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("unexpected") || error_msg.contains("unknown field"),
            "Error should mention the unknown field: {}",
            error_msg
        );
    }

    /// Scenario: A logs `routing_key` map uses an unrecognized attribute-kind
    /// name (`log_attribute` instead of `log_record_attribute`).
    /// Guarantees: The custom deserializer reports it as an unknown field rather
    /// than accepting it.
    #[test]
    fn test_logs_routing_key_rejects_unknown_kind() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "default_event_name": "Log",
                "event_name_mapping": {
                    "routing_key": { "log_attribute": "custom" },
                    "events": { "test1": "Test1" }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject unknown logs routing_key kind"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("unknown field") && error_msg.contains("log_attribute"),
            "Error should name the unknown routing_key field: {}",
            error_msg
        );
    }

    /// Scenario: A spans `routing_key` map uses an unrecognized attribute-kind
    /// name (`log_record_attribute`, which is logs-only).
    /// Guarantees: The custom deserializer reports it as an unknown field rather
    /// than accepting it.
    #[test]
    fn test_spans_routing_key_rejects_unknown_kind() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "event_name_mapping": {
                    "routing_key": { "log_record_attribute": "custom" },
                    "events": { "test1": "Test1" }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject unknown spans routing_key kind"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("unknown field") && error_msg.contains("log_record_attribute"),
            "Error should name the unknown routing_key field: {}",
            error_msg
        );
    }

    /// Scenario: A logs `routing_key` uses the map form `{ event_name: <v> }`
    /// with either a null or a string value.
    /// Guarantees: Both forms are rejected during deserialization, since
    /// `event_name` takes no routing value; the error directs the user to the
    /// string form `routing_key: event_name` so a supplied value can never be
    /// silently ignored.
    #[test]
    fn test_logs_routing_key_event_name_map_form_is_rejected() {
        for value in [serde_json::Value::Null, serde_json::json!("anything")] {
            let config = serde_json::json!({
                "endpoint": "https://localhost",
                "environment": "test",
                "account": "test-account",
                "account_routing": { "default_group": "test-group" },
                "namespace": "test-namespace",
                "region": "test-region",
                "config_major_version": 1,
                "tenant": "test-tenant",
                "role_name": "test-role",
                "role_instance": "test-instance",
                "logs": {
                    "default_event_name": "Log",
                    "event_name_mapping": {
                        "routing_key": { "event_name": value },
                        "events": { "test1": "Test1" }
                    }
                },
                "auth": {
                    "type": "certificate",
                    "path": "/path/to/cert.p12",
                    "password": "secret"
                }
            });

            let result: Result<Config, _> = serde_json::from_value(config);
            assert!(
                result.is_err(),
                "map-form event_name should be rejected regardless of value"
            );
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("event_name") && error_msg.contains("does not take a value"),
                "Error should explain event_name takes no value, got: {error_msg}"
            );
        }
    }

    /// Scenario: `logs` and `spans` sections are present but empty (`{}`).
    /// Guarantees: They deserialize to `Some(..)` with both `default_event_name`
    /// and `event_name_mapping` defaulting to `None`, so an empty block is valid
    /// and carries no routing configuration.
    #[test]
    fn test_logs_and_spans_empty_blocks_default_inner_fields_to_none() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {},
            "spans": {},
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).unwrap();
        let logs = parsed.logs.as_ref().expect("logs block should be present");
        assert!(logs.default_event_name.is_none());
        assert!(logs.event_name_mapping.is_none());
        let spans = parsed
            .spans
            .as_ref()
            .expect("spans block should be present");
        assert!(spans.default_event_name.is_none());
        assert!(spans.event_name_mapping.is_none());
    }

    /// Scenario: `logs`/`spans` set `default_event_name` explicitly to null.
    /// Guarantees: The non-empty-name validator treats explicit null as absent,
    /// yielding `None` (not an error), matching an omitted field.
    #[test]
    fn test_default_event_name_explicit_null_is_none() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": { "default_event_name": null },
            "spans": { "default_event_name": null },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).unwrap();
        assert!(parsed.logs.as_ref().unwrap().default_event_name.is_none());
        assert!(parsed.spans.as_ref().unwrap().default_event_name.is_none());
    }

    /// Scenario: A `spans` block carries an unknown field directly (not inside
    /// `event_name_mapping`).
    /// Guarantees: `deny_unknown_fields` on `TracesConfig` rejects the typo
    /// instead of silently discarding it.
    #[test]
    fn test_spans_config_rejects_unknown_field() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "default_event_name": "Span",
                "default_event_names": "Span"
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "Should reject unknown field in spans config"
        );
        let error_msg = result.unwrap_err().to_string();
        assert!(
            error_msg.contains("default_event_names") || error_msg.contains("unknown field"),
            "Error should mention the unknown field: {}",
            error_msg
        );
    }

    /// Scenario: A full `Config` with both `logs` and `spans` sections - each
    /// with a `default_event_name` and an `event_name_mapping` using different
    /// routing-key variants and event maps - is converted through the
    /// `Config::to_geneva_client_config` adapter into a `GenevaClientConfig`.
    /// Guarantees: The adapter carries every field to the correct signal
    /// without a logs/spans mix-up: logs keep their default table name, routing
    /// key (`log_record_attribute`) and events map; spans keep their default
    /// table name, routing key (`span_attribute`) and events map; and scalar
    /// fields (endpoint, account, ...) are propagated unchanged.
    #[test]
    fn test_config_to_geneva_client_config_full_adapter() {
        let config = serde_json::json!({
            "endpoint": "https://geneva.example",
            "environment": "prod-env",
            "account": "acct-1",
            "account_routing": {
                "default_group": "default-group",
                "events": { "AuditLogs": "audit-group" }
            },
            "namespace": "ns-1",
            "region": "westus2",
            "config_major_version": 3,
            "tenant": "tenant-1",
            "role_name": "role-1",
            "role_instance": "instance-1",
            "logs": {
                "default_event_name": "MyLogs",
                "event_name_mapping": {
                    "routing_key": { "log_record_attribute": "log.kind" },
                    "events": { "audit": "AuditLogs", "raw": null }
                }
            },
            "spans": {
                "default_event_name": "MySpans",
                "event_name_mapping": {
                    "routing_key": { "span_attribute": "span.kind" },
                    "events": { "SERVER": "ServerSpans", "CLIENT": null }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).expect("config should parse");
        assert_eq!(parsed.account_routing.default_group, "default-group");
        assert_eq!(
            parsed
                .account_routing
                .events
                .get("AuditLogs")
                .map(String::as_str),
            Some("audit-group")
        );
        let client_config = parsed.to_geneva_client_config();

        // Scalar fields propagate unchanged.
        assert_eq!(client_config.endpoint, "https://geneva.example");
        assert_eq!(client_config.environment, "prod-env");
        assert_eq!(client_config.account, "acct-1");
        assert_eq!(client_config.namespace, "ns-1");
        assert_eq!(client_config.region, "westus2");
        assert_eq!(client_config.config_major_version, 3);
        assert_eq!(client_config.tenant, "tenant-1");
        assert_eq!(client_config.role_name, "role-1");
        assert_eq!(client_config.role_instance, "instance-1");

        // Certificate auth maps to AuthMethod::Certificate with no MSI resource.
        match &client_config.auth_method {
            AuthMethod::Certificate { path, password } => {
                assert_eq!(path, &PathBuf::from("/path/to/cert.p12"));
                assert_eq!(password, "secret");
            }
            other => panic!("expected Certificate auth method, got {other:?}"),
        }
        assert_eq!(client_config.msi_resource, None);
        assert!(client_config.obo_event_map.is_none());

        // Logs mapped to the logs slot with the correct routing key + events.
        let logs = client_config.logs.expect("logs config should be present");
        assert_eq!(logs.default_event_name.as_deref(), Some("MyLogs"));
        let logs_mapping = logs
            .event_name_mapping
            .expect("logs mapping should be present");
        match &logs_mapping.routing_key {
            LogsEventNameRoutingKey::LogRecordAttribute(k) => assert_eq!(k, "log.kind"),
            other => panic!("expected LogRecordAttribute routing key, got {other:?}"),
        }
        assert_eq!(
            logs_mapping.events.get("audit"),
            Some(&Some("AuditLogs".to_string()))
        );
        assert_eq!(logs_mapping.events.get("raw"), Some(&None));

        // Spans mapped to the spans slot with the correct routing key + events.
        let spans = client_config.spans.expect("spans config should be present");
        assert_eq!(spans.default_event_name.as_deref(), Some("MySpans"));
        let spans_mapping = spans
            .event_name_mapping
            .expect("spans mapping should be present");
        match &spans_mapping.routing_key {
            SpanEventNameRoutingKey::SpanAttribute(k) => assert_eq!(k, "span.kind"),
            other => panic!("expected SpanAttribute routing key, got {other:?}"),
        }
        assert_eq!(
            spans_mapping.events.get("SERVER"),
            Some(&Some("ServerSpans".to_string()))
        );
        assert_eq!(spans_mapping.events.get("CLIENT"), Some(&None));
    }

    /// Scenario: Account routing has a blank default group, event name, or mapped group.
    /// Guarantees: Invalid logical routing is rejected while parsing user configuration.
    #[test]
    fn test_account_routing_rejects_blank_names() {
        let mut blank_default = test_config();
        blank_default["account_routing"]["default_group"] =
            serde_json::Value::String("   ".to_owned());
        let error = Config::parse(&blank_default).expect_err("blank default group must fail");
        assert!(
            error
                .to_string()
                .contains("default_group must not be empty")
        );

        for (event_name, account_group) in [("", "group"), ("Event", " ")] {
            let mut config = test_config();
            config["account_routing"]["events"] = serde_json::json!({ event_name: account_group });
            let error = Config::parse(&config).expect_err("blank routing name must fail");
            assert!(error.to_string().contains("must not be empty"));
        }
    }

    /// Scenario: Account routing identifiers contain leading or trailing whitespace.
    /// Guarantees: Exact-match routing cannot accept identifiers that will miss valid groups.
    #[test]
    fn test_account_routing_rejects_surrounding_whitespace() {
        for default_group in [" default-group", "default-group "] {
            let mut config = test_config();
            config["account_routing"]["default_group"] =
                serde_json::Value::String(default_group.to_owned());
            let error = Config::parse(&config)
                .expect_err("default group with surrounding whitespace must fail");
            assert!(error.to_string().contains("surrounding whitespace"));
        }

        for (event_name, account_group) in [
            (" Event", "group"),
            ("Event ", "group"),
            ("Event", " group"),
            ("Event", "group "),
        ] {
            let mut config = test_config();
            config["account_routing"]["events"] = serde_json::json!({
                event_name: account_group
            });
            let error = Config::parse(&config)
                .expect_err("routing identifier with surrounding whitespace must fail");
            assert!(error.to_string().contains("surrounding whitespace"));
        }
    }

    /// Scenario: A `Config` with an `obo` block mapping two event/table names to
    /// customer identities - one with an annotations recipe, one without - is
    /// converted through `Config::to_geneva_client_config`.
    /// Guarantees: The adapter populates `obo_event_map` with one entry per
    /// configured event, preserving each identity and its optional annotations,
    /// so OBO uploads are enabled for exactly the configured events and left off
    /// (None) for everything else.
    #[test]
    fn test_config_to_geneva_client_config_with_obo() {
        let config = serde_json::json!({
            "endpoint": "https://geneva.example",
            "environment": "prod-env",
            "account": "acct-1",
            "account_routing": { "default_group": "default-group" },
            "namespace": "ns-1",
            "region": "westus2",
            "config_major_version": 3,
            "tenant": "tenant-1",
            "role_name": "role-1",
            "role_instance": "instance-1",
            "obo": {
                "events": {
                    "AuditLogs": {
                        "identity": "Microsoft.AuditService",
                        "annotations": "<Config onBehalfFields=\"resourceId\" />"
                    },
                    "RawLogs": {
                        "identity": "Microsoft.RawService"
                    }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).expect("config should parse");
        let client_config = parsed.to_geneva_client_config();

        let obo = client_config
            .obo_event_map
            .expect("obo_event_map should be present");
        assert_eq!(obo.len(), 2);

        let audit = obo.get("AuditLogs").expect("AuditLogs entry present");
        assert_eq!(audit.identity, "Microsoft.AuditService");
        assert_eq!(
            audit.annotations.as_deref(),
            Some("<Config onBehalfFields=\"resourceId\" />")
        );

        let raw = obo.get("RawLogs").expect("RawLogs entry present");
        assert_eq!(raw.identity, "Microsoft.RawService");
        assert_eq!(raw.annotations, None);
    }

    /// Scenario: A `Config` with no `obo` block is converted through
    /// `Config::to_geneva_client_config`.
    /// Guarantees: `obo_event_map` stays `None`, so OBO remains opt-in and the
    /// default configuration uploads without any customer identity.
    #[test]
    fn test_config_to_geneva_client_config_without_obo_is_none() {
        let parsed: Config = serde_json::from_value(test_config()).expect("config should parse");
        let client_config = parsed.to_geneva_client_config();
        assert!(client_config.obo_event_map.is_none());
    }

    /// Scenario: An `obo` entry supplies an empty (whitespace-only) identity.
    /// Guarantees: Deserialization fails up-front so `--validate` rejects an OBO
    /// entry that could not identify a customer, instead of silently uploading
    /// without OBO at pipeline startup.
    #[test]
    fn test_obo_rejects_empty_identity() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "obo": {
                "events": {
                    "AuditLogs": { "identity": "   " }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "empty identity should be rejected");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("non-empty identity"),
            "error should explain identity must be non-empty"
        );
    }

    /// Scenario: An `obo` block supplies an empty `events` map.
    /// Guarantees: Deserialization fails rather than accepting an OBO config that
    /// enables OBO for no events, keeping `--validate` in agreement with runtime.
    #[test]
    fn test_obo_rejects_empty_events() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "obo": { "events": {} },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "empty events map should be rejected");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be non-empty"),
            "error should explain events must be non-empty"
        );
    }

    /// Scenario: An `obo` entry maps an identity to an empty (whitespace-only)
    /// annotations string.
    /// Guarantees: Deserialization fails rather than silently treating the empty
    /// annotations as "no recipe"; only omitting the field (or null) selects the
    /// no-annotations form, so an ineffective config cannot look valid.
    #[test]
    fn test_obo_rejects_empty_annotations() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "obo": {
                "events": {
                    "AuditLogs": { "identity": "Microsoft.AuditService", "annotations": "   " }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "empty annotations should be rejected");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("empty annotations value"),
            "error should explain empty annotations are rejected"
        );
    }

    /// Scenario: OBO is keyed on the uploader's literal default table names
    /// ("Log"/"Span") while `logs`/`spans` (and their `default_event_name`) are
    /// omitted, so the destinations exist only via the uploader's fallback.
    /// Guarantees: These keys are treated as reachable and produce no unmatched
    /// warning, so valid configs relying on the literal defaults stay quiet.
    #[test]
    fn test_obo_literal_default_names_are_matched() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "obo": {
                "events": {
                    "Log": { "identity": "Microsoft.LogService" },
                    "Span": { "identity": "Microsoft.SpanService" }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let config: Config = serde_json::from_value(config).expect("config should deserialize");
        assert!(
            config.unmatched_obo_events().is_empty(),
            "OBO keyed on the literal default table names must not be flagged as unmatched"
        );
    }

    /// Scenario: OBO is keyed on a name that neither a mapping destination nor a
    /// `default_event_name` nor the literal defaults can produce.
    /// Guarantees: The typo-catching path reports that key as unmatched so the
    /// warning still fires for genuinely unreachable OBO entries.
    #[test]
    fn test_obo_unreachable_name_is_unmatched() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "obo": {
                "events": {
                    "TypoTable": { "identity": "Microsoft.SomeService" }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let config: Config = serde_json::from_value(config).expect("config should deserialize");
        assert_eq!(
            config.unmatched_obo_events(),
            vec!["TypoTable".to_string()],
            "an OBO key matching no reachable destination must be reported as unmatched"
        );
    }

    /// Scenario: The user overrides logs `default_event_name` (e.g. "MyLog") and
    /// keys OBO on the literal "Log", which the uploader would only produce when
    /// no default is set.
    /// Guarantees: The literal "Log" is not treated as reachable once the
    /// default is overridden, so the typo-prone key is reported as unmatched
    /// while the overridden default itself is accepted.
    #[test]
    fn test_obo_literal_default_not_matched_when_overridden() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": { "default_event_name": "MyLog" },
            "obo": {
                "events": {
                    "Log": { "identity": "Microsoft.LogService" },
                    "MyLog": { "identity": "Microsoft.MyLogService" }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let config: Config = serde_json::from_value(config).expect("config should deserialize");
        assert_eq!(
            config.unmatched_obo_events(),
            vec!["Log".to_string()],
            "the literal default must not be reachable once default_event_name is overridden"
        );
    }

    /// Scenario: A logs `event_name_mapping` supplies an empty `events` map.
    /// Guarantees: Deserialization (the same path `--validate` exercises) fails
    /// up-front instead of being accepted here and later rejected inside
    /// `GenevaClient::new()` at pipeline startup.
    #[test]
    fn test_logs_events_map_rejects_empty_events() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "event_name_mapping": {
                    "routing_key": "event_name",
                    "events": {}
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "empty events map should be rejected");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be non-empty"),
            "error should explain events must be non-empty"
        );
    }

    /// Scenario: A logs `event_name_mapping` uses a blank (whitespace-only)
    /// source key.
    /// Guarantees: Deserialization fails so `--validate` rejects a mapping that
    /// could never route, matching the uploader's own validation.
    #[test]
    fn test_logs_events_map_rejects_blank_source_key() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "event_name_mapping": {
                    "routing_key": "event_name",
                    "events": { "   ": "TableA" }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "blank source key should be rejected");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("source keys must not be blank"),
            "error should explain source keys must not be blank"
        );
    }

    /// Scenario: A logs `event_name_mapping` maps a source value to an empty
    /// (or whitespace-only) destination string, e.g. `Foo: ""`.
    /// Guarantees: Deserialization fails rather than silently treating the empty
    /// destination as passthrough; only the documented `null` form selects
    /// passthrough. This prevents an ineffective config from looking valid.
    #[test]
    fn test_logs_events_map_rejects_empty_destination() {
        for dest in ["", "   "] {
            let config = serde_json::json!({
                "endpoint": "https://localhost",
                "environment": "test",
                "account": "test-account",
                "account_routing": { "default_group": "test-group" },
                "namespace": "test-namespace",
                "region": "test-region",
                "config_major_version": 1,
                "tenant": "test-tenant",
                "role_name": "test-role",
                "role_instance": "test-instance",
                "logs": {
                    "event_name_mapping": {
                        "routing_key": "event_name",
                        "events": { "Foo": dest }
                    }
                },
                "auth": {
                    "type": "certificate",
                    "path": "/path/to/cert.p12",
                    "password": "secret"
                }
            });

            let result: Result<Config, _> = serde_json::from_value(config);
            assert!(
                result.is_err(),
                "empty destination '{dest}' should be rejected"
            );
            let error_msg = result.unwrap_err().to_string();
            assert!(
                error_msg.contains("must") && error_msg.contains("empty or whitespace"),
                "error should explain empty destinations are rejected, got: {error_msg}"
            );
        }
    }

    /// Scenario: A `null` destination (the documented passthrough form) is used
    /// in a logs `event_name_mapping`.
    /// Guarantees: The `null` form remains valid and deserializes to a `None`
    /// destination, so the destination-validation only rejects empty strings and
    /// not the intended passthrough form.
    #[test]
    fn test_logs_events_map_accepts_null_passthrough_destination() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "logs": {
                "event_name_mapping": {
                    "routing_key": "event_name",
                    "events": { "Foo": null }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let parsed: Config = serde_json::from_value(config).expect("null passthrough should parse");
        let mapping = parsed
            .logs
            .and_then(|l| l.event_name_mapping)
            .expect("logs mapping should be configured");
        assert_eq!(mapping.events.get("Foo"), Some(&None));
    }

    /// Scenario: A spans `event_name_mapping` supplies an empty `events` map.
    /// Guarantees: Deserialization fails up-front, mirroring the logs behavior
    /// so `--validate` and pipeline startup agree for spans too.
    #[test]
    fn test_spans_events_map_rejects_empty_events() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "event_name_mapping": {
                    "routing_key": { "span_attribute": "span.kind" },
                    "events": {}
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(result.is_err(), "empty spans events map should be rejected");
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("must be non-empty"),
            "error should explain events must be non-empty"
        );
    }

    /// Scenario: A spans `event_name_mapping` maps a source value to an empty
    /// destination string.
    /// Guarantees: Deserialization fails, matching the logs behavior so empty
    /// destinations are consistently rejected across both signals.
    #[test]
    fn test_spans_events_map_rejects_empty_destination() {
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
            "account_routing": { "default_group": "test-group" },
            "namespace": "test-namespace",
            "region": "test-region",
            "config_major_version": 1,
            "tenant": "test-tenant",
            "role_name": "test-role",
            "role_instance": "test-instance",
            "spans": {
                "event_name_mapping": {
                    "routing_key": { "span_attribute": "span.kind" },
                    "events": { "SERVER": "" }
                }
            },
            "auth": {
                "type": "certificate",
                "path": "/path/to/cert.p12",
                "password": "secret"
            }
        });

        let result: Result<Config, _> = serde_json::from_value(config);
        assert!(
            result.is_err(),
            "empty spans destination should be rejected"
        );
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("empty or whitespace"),
            "error should explain empty destinations are rejected"
        );
    }

    // TODO: Add integration tests when we can mock GenevaClient:
    // - test_geneva_exporter_encodes_and_uploads_logs_view()
    // - test_geneva_exporter_handles_upload_failure()
    // - test_geneva_exporter_fallback_to_otlp_bytes()
    // - test_geneva_exporter_metrics_tracking()
}
