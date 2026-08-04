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
//!       # ... additional config
//! ```

use async_trait::async_trait;
use linkme::distributed_slice;
use otap_df_config::SignalType;
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
use otap_df_telemetry::otel_info;
use otap_df_telemetry_macros::metric_set;
use serde::{Deserialize, Deserializer};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Instant;

// Geneva uploader dependencies
use futures::StreamExt;
use geneva_uploader::AuthMethod;
use geneva_uploader::client::{EncodedBatch, GenevaClient, GenevaClientConfig};
use geneva_uploader::{
    LogsEventNameMapping, LogsEventNameRoutingKey, SpanEventNameMapping, SpanEventNameRoutingKey,
};
use opentelemetry_proto::tonic::collector::trace::v1::ExportTraceServiceRequest;
use prost::Message as ProstMessage;

// Use crate-relative paths since we're now a module within otap
use otap_df_otap::OTAP_EXPORTER_FACTORIES;
use otap_df_otap::metrics::ExporterPDataExportMetrics;
use otap_df_otap::pdata::OtapPdata;
use otap_df_telemetry::common_attributes::{Outcome, SignalOutcomeAttributes};

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

/// Configuration for the Geneva Exporter
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Geneva endpoint URL
    pub endpoint: String,
    /// Environment (e.g., "production", "staging")
    pub environment: String,
    /// Geneva account name
    pub account: String,
    /// Geneva namespace
    pub namespace: String,
    /// Azure region
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
        // Convert AuthConfig to AuthMethod
        let auth_method = match &self.auth {
            AuthConfig::Certificate { path, password } => AuthMethod::Certificate {
                path: PathBuf::from(path),
                password: password.clone(),
            },
            AuthConfig::SystemManagedIdentity { .. } => AuthMethod::SystemManagedIdentity,
            AuthConfig::UserManagedIdentity { client_id, .. } => AuthMethod::UserManagedIdentity {
                client_id: client_id.clone(),
            },
            AuthConfig::UserManagedIdentityByArmResourceId { resource_id, .. } => {
                AuthMethod::UserManagedIdentityByResourceId {
                    resource_id: resource_id.clone(),
                }
            }
            AuthConfig::WorkloadIdentity { msi_resource } => AuthMethod::WorkloadIdentity {
                resource: msi_resource.clone(),
            },
        };

        // Get MSI resource if needed for managed identity
        let msi_resource = match &self.auth {
            AuthConfig::SystemManagedIdentity { msi_resource }
            | AuthConfig::UserManagedIdentity { msi_resource, .. }
            | AuthConfig::UserManagedIdentityByArmResourceId { msi_resource, .. }
            | AuthConfig::WorkloadIdentity { msi_resource } => Some(msi_resource.clone()),
            AuthConfig::Certificate { .. } => None,
        };

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
            region: self.region.clone(),
            config_major_version: self.config_major_version,
            auth_method,
            tenant: self.tenant.clone(),
            role_name: self.role_name.clone(),
            role_instance: self.role_instance.clone(),
            msi_resource,
            logs: logs_config,
            spans: traces_config,
            obo_event_map: None,
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
    pdata_metrics: MeasurementMetricSet<ExporterPDataExportMetrics>,
    metrics: MetricSet<ExporterMetrics>,
    geneva_client: GenevaClient,
}

impl GenevaExporter {
    /// Create a new Geneva exporter from configuration
    pub fn from_config(
        pipeline_ctx: PipelineContext,
        config: &serde_json::Value,
    ) -> Result<Self, otap_df_config::error::Error> {
        let pdata_metrics = ExporterPDataExportMetrics::register(&pipeline_ctx);
        let metrics = pipeline_ctx.register_metrics::<ExporterMetrics>();

        let config: Config = serde_json::from_value(config.clone()).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: e.to_string(),
            }
        })?;

        let client_config = config.to_geneva_client_config();

        // The Geneva exporter uses rustls for mTLS. If no process-wide crypto
        // provider was installed at startup (i.e. the binary was built without
        // any `crypto-*` feature), fail fast with an actionable error instead
        // of surfacing an opaque rustls handshake failure at export time.
        if !otap_df_otap::crypto::is_crypto_provider_installed() {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "Geneva exporter requires a rustls CryptoProvider, but none is installed. \
                        Build with exactly one of the crypto-* features \
                        (crypto-ring, crypto-aws-lc, crypto-openssl, crypto-symcrypt) and ensure \
                        otap_df_otap::crypto::install_crypto_provider() runs at startup."
                    .to_string(),
            });
        }

        // Initialize Geneva client
        let geneva_client = GenevaClient::new(client_config).map_err(|e| {
            otap_df_config::error::Error::InvalidUserConfig {
                error: format!("Failed to initialize Geneva client: {}", e),
            }
        })?;

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

/// Register Geneva exporter with the OTAP exporter factory
///
/// Unsafe code is temporarily used here to allow the use of `distributed_slice` macro
/// This macro is part of the `linkme` crate which is considered safe and well maintained.
#[allow(unsafe_code)]
#[distributed_slice(OTAP_EXPORTER_FACTORIES)]
pub static GENEVA_EXPORTER: ExporterFactory<OtapPdata> = ExporterFactory {
    name: GENEVA_EXPORTER_URN,
    create: |pipeline: PipelineContext,
             node: NodeId,
             node_config: Arc<NodeUserConfig>,
             exporter_config: &ExporterConfig,
             _capabilities: &otap_df_engine::capability::registry::Capabilities| {
        Ok(ExporterWrapper::local(
            GenevaExporter::from_config(pipeline, &node_config.config)?,
            node,
            node_config,
            exporter_config,
        ))
    },
    wiring_contract: otap_df_engine::wiring_contract::WiringContract::UNRESTRICTED,
    validate_config: otap_df_config::validation::validate_typed_config::<Config>,
};

#[async_trait(?Send)]
impl Exporter<OtapPdata> for GenevaExporter {
    async fn start(
        mut self: Box<Self>,
        mut msg_chan: ExporterInbox<OtapPdata>,
        effect_handler: EffectHandler<OtapPdata>,
    ) -> Result<TerminalState, Error> {
        otel_info!(
            "geneva_exporter.start",
            endpoint = self.config.endpoint,
            namespace = self.config.namespace,
            account = self.config.account,
            role_name = self.config.role_name,
            role_instance = self.config.role_instance,
            message = "Geneva exporter starting"
        );

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
                                .messages
                                .inc();
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
                                .messages
                                .inc();
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
    use serde_json;

    use arrow::array::{
        ArrayRef, Int32Array, RecordBatch, StringArray, StructArray, TimestampNanosecondArray,
        UInt8Array, UInt16Array, UInt32Array,
    };
    use arrow::datatypes::{DataType, Field, Schema, TimeUnit};
    use std::sync::Arc;

    use bytes::Bytes;
    use otap_df_engine::Interests;
    use otap_df_engine::control::PipelineCompletionMsg;
    use otap_df_engine::testing::exporter::{TestRuntime, create_exporter_from_factory};
    use otap_df_otap::testing::{TestCallData, next_ack, next_nack};
    use otap_df_pdata::otap::OtapArrowRecords;
    use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otap_df_pdata::schema::{FieldExt, consts};
    use otap_df_pdata::views::otap::OtapLogsView;
    use otap_df_pdata_views::views::logs::{LogsDataView, ResourceLogsView, ScopeLogsView};
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
    #[test]
    fn test_config_deserialization() {
        let json = serde_json::json!({
            "endpoint": "https://geneva.example.com",
            "environment": "production",
            "account": "test-account",
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
    }

    #[test]
    fn test_urn_constant() {
        assert_eq!(GENEVA_EXPORTER_URN, "urn:microsoft:exporter:geneva");
    }

    #[test]
    fn create_exporter_with_user_managed_identity_by_arm_resource_id() {
        // The Geneva uploader uses rustls (tls-rustls); reqwest needs a
        // process-wide crypto provider, which production installs at startup.
        otap_df_otap::crypto::ensure_crypto_provider();
        let config = serde_json::json!({
            "endpoint": "https://localhost",
            "environment": "test",
            "account": "test-account",
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
        let exporter = create_exporter_from_factory(&GENEVA_EXPORTER, config);
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
