// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Receiver and endpoint configuration types.

#![allow(dead_code)]

use super::Error;
use crate::receivers::microsoft_json_receiver::ikey_filter::IkeyFilterSource;
use otel_arrow_dfe_engine::context::PipelineContext;
use otel_arrow_dfe_otap::metrics::ReceiverMetrics;
use otel_arrow_dfe_telemetry::error::Error as TelemetryError;
use otel_arrow_dfe_telemetry::instrument::Counter;
use otel_arrow_dfe_telemetry::metrics::{MetricSet, MetricSetRegistrar, MetricSetSnapshot};
use otel_arrow_dfe_telemetry::reporter::MetricsReporter;
use otel_arrow_dfe_telemetry_macros::metric_set;
use serde::{Deserialize, Deserializer};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::time::Duration;

/// Reference to an external endpoint configuration YAML file.
///
/// Endpoint configurations are always loaded from external files.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EndpointFileRef {
    /// Path to a YAML file containing a full `EndpointConfig`.
    /// Relative paths are resolved against the pipeline config directory.
    pub config_file: String,
}

// ---------------------------------------------------------------------------
// Enrichment attribute target
// ---------------------------------------------------------------------------

/// Controls whether a group of enrichment attributes is emitted as
/// **resource** attributes or **log** attributes.
///
/// # Example
///
/// ```yaml
/// user_agent_attr_target: log_attribute  # emit as log attributes instead
/// http_attr_target: resource
/// ```
#[derive(Debug, Default, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EnrichmentTarget {
    /// Emit as OTel resource attributes (default).
    #[default]
    Resource,
    /// Emit as OTel log record attributes.
    LogAttribute,
}

// ---------------------------------------------------------------------------
// Constants
// ---------------------------------------------------------------------------

/// Default maximum request body size (3 MiB).
///
/// This acts as a hard cap across all endpoints.  Per-endpoint limits
/// (set in the endpoint YAML) may be lower but never higher.
pub(crate) const DEFAULT_MAX_REQUEST_BODY_SIZE: u32 = 3 * 1024 * 1024;

/// Default maximum concurrent requests.
pub(crate) const DEFAULT_MAX_CONCURRENT_REQUESTS: usize = 256;

// ---------------------------------------------------------------------------
// Configuration
// ---------------------------------------------------------------------------

/// Configuration for the Microsoft JSON receiver.
///
/// # Example
///
/// ```yaml
/// config:
///   listening_addr: "0.0.0.0:4319"
///   allowed_origins:
///     - "https://example.com"
///   max_request_body_size: 3145728
/// ```
#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Address on which the receiver listens for HTTP requests.
    pub(crate) listening_addr: SocketAddr,

    /// Allowed CORS origins. Use `["*"]` to allow any origin.
    /// Defaults to `["*"]`.
    #[serde(default = "default_allowed_origins")]
    pub(crate) allowed_origins: Vec<String>,

    /// Maximum request body size in bytes.
    #[serde(default = "default_max_request_body_size")]
    pub(crate) max_request_body_size: u32,

    /// Maximum concurrent in-flight requests.
    #[serde(default = "default_max_concurrent_requests")]
    pub(crate) max_concurrent_requests: usize,

    /// Request timeout.
    #[serde(default = "default_timeout", with = "humantime_serde")]
    pub(crate) timeout: Option<Duration>,

    /// Maximum time allowed to receive HTTP headers.
    #[serde(default = "default_header_read_timeout", with = "humantime_serde")]
    pub(crate) header_read_timeout: Option<Duration>,

    /// Per-endpoint configurations.
    ///
    /// Keys are URL path prefixes (e.g. `"/OneCollector/1.0"`,
    /// `"/v2/track"`).  Each value must contain a `config_file` field
    /// pointing to an external YAML file with the full endpoint
    /// configuration.
    #[serde(default)]
    pub(crate) endpoints: HashMap<String, EndpointFileRef>,

    /// Where to emit user-agent attributes (`user_agent.original`,
    /// `browser.*`, `os.*`, `device.*`).  Default: `resource`.
    #[serde(default)]
    pub(crate) user_agent_attr_target: EnrichmentTarget,

    /// Where to emit HTTP semantic convention attributes
    /// (`http.request.method`).  Default: `resource`.
    #[serde(default)]
    pub(crate) http_attr_target: EnrichmentTarget,

    /// When set, a UUID v4 is generated once at receiver startup and
    /// emitted as a resource attribute with this key on every event.
    /// Useful in horizontally-scaled deployments (e.g. Azure Container
    /// Apps) to distinguish which replica produced a given log record.
    ///
    /// Example: `instance_id_attr: "ms.container_app.instance_id"`
    #[serde(default)]
    pub(crate) instance_id_attr: Option<String>,

    /// When set, the request origin domain (from the `Origin` header,
    /// or the authority portion of the `Referer` header) is emitted as
    /// a resource attribute with this key on every event.
    ///
    /// The stored value is the full origin URL (e.g.
    /// `https://myapp.example.com`).  If no `Origin` or `Referer`
    /// header is present the attribute is omitted.
    ///
    /// Example: `origin_attr: "http.origin"`
    #[serde(default)]
    pub(crate) origin_attr: Option<String>,
}

impl Config {
    /// Validates receiver-level limits and endpoint references.
    pub fn validate(&self) -> Result<(), Error> {
        if self.max_request_body_size == 0 {
            return Err(Error::Config(
                "max_request_body_size must be greater than zero".to_owned(),
            ));
        }
        if self.max_concurrent_requests == 0 {
            return Err(Error::Config(
                "max_concurrent_requests must be greater than zero".to_owned(),
            ));
        }
        validate_optional_duration("timeout", self.timeout)?;
        validate_optional_duration("header_read_timeout", self.header_read_timeout)?;

        if self.allowed_origins.is_empty()
            || self
                .allowed_origins
                .iter()
                .any(|origin| origin.trim().is_empty())
        {
            return Err(Error::Config(
                "allowed_origins must contain at least one non-empty origin".to_owned(),
            ));
        }

        for (path, endpoint) in &self.endpoints {
            if !path.starts_with('/') {
                return Err(Error::Config(format!(
                    "endpoint path `{path}` must start with '/'"
                )));
            }
            if endpoint.config_file.trim().is_empty() {
                return Err(Error::Config(format!(
                    "endpoint path `{path}` requires a non-empty config_file"
                )));
            }
        }

        validate_optional_attribute("instance_id_attr", self.instance_id_attr.as_deref())?;
        validate_optional_attribute("origin_attr", self.origin_attr.as_deref())
    }
}

fn validate_optional_duration(name: &str, duration: Option<Duration>) -> Result<(), Error> {
    if duration.is_some_and(|value| value.is_zero()) {
        return Err(Error::Config(format!("{name} must be greater than zero")));
    }
    Ok(())
}

fn validate_optional_attribute(name: &str, attribute: Option<&str>) -> Result<(), Error> {
    if attribute.is_some_and(|value| value.trim().is_empty()) {
        return Err(Error::Config(format!("{name} must not be empty")));
    }
    Ok(())
}

fn default_allowed_origins() -> Vec<String> {
    vec!["*".to_string()]
}

const fn default_max_request_body_size() -> u32 {
    DEFAULT_MAX_REQUEST_BODY_SIZE
}

const fn default_max_concurrent_requests() -> usize {
    DEFAULT_MAX_CONCURRENT_REQUESTS
}

const fn default_timeout() -> Option<Duration> {
    Some(Duration::from_secs(30))
}

const fn default_header_read_timeout() -> Option<Duration> {
    Some(Duration::from_secs(5))
}

// ---------------------------------------------------------------------------
// Per-endpoint configuration types (deserialized from YAML)
// ---------------------------------------------------------------------------

/// Full per-endpoint configuration:  HTTP behaviour + field mapping.
///
/// Loaded from an external YAML file referenced by `config_file` in the
/// receiver's `endpoints` map.
#[derive(Debug, Clone, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub(crate) struct EndpointConfig {
    /// Human-readable label (used in diagnostics / logging).
    #[serde(default)]
    pub name: String,

    // -- Size limits -------------------------------------------------------
    /// Maximum request body size in bytes for this endpoint.
    ///
    /// Overrides the receiver-level `max_request_body_size` for requests
    /// hitting this endpoint.  The effective limit is
    /// `min(endpoint, receiver)` -- the receiver-level setting always
    /// acts as a hard cap.
    ///
    /// When omitted, the receiver-level default is used.
    ///
    /// ```yaml
    /// max_request_body_size: 3145728   # 3 MiB (OneCollector)
    /// max_request_body_size: 102400    # 100 KiB (Application Insights)
    /// ```
    #[serde(default)]
    pub max_request_body_size: Option<u32>,

    /// Maximum size of a single event (JSON line or array element) in
    /// bytes.  Events exceeding this limit are rejected individually
    /// (the rest of the request continues processing).
    ///
    /// When omitted, no per-event size check is performed (the request
    /// body limit is the only guard).
    ///
    /// ```yaml
    /// max_event_size: 2097152   # 2 MiB (OneCollector)
    /// max_event_size: 102400    # 100 KiB (Application Insights)
    /// ```
    #[serde(default)]
    pub max_event_size: Option<usize>,

    /// Whether to emit the compact JSON byte size of the retained source fields.
    #[serde(default)]
    pub filtered_item_size: bool,

    // -- HTTP behaviour ----------------------------------------------------
    /// Allowed HTTP methods for this endpoint (e.g. `["POST"]`,
    /// `["POST", "GET"]`).  If empty, defaults to `["POST"]`.
    #[serde(default)]
    pub allowed_methods: Vec<String>,

    /// Accepted `Content-Type` values for POST requests.  If empty, any
    /// content type is accepted.
    #[serde(default)]
    pub accepted_content_types: Vec<String>,

    /// Accepted POST request body formats.
    #[serde(default)]
    pub body_format: BodyFormats,

    /// How GET query-string parameters are interpreted.
    #[serde(default)]
    pub get_format: GetFormat,

    // -- CORS --------------------------------------------------------------
    /// CORS response header configuration.
    #[serde(default)]
    pub cors: CorsConfig,

    // -- Header / query-string extraction ----------------------------------
    /// Map HTTP request headers to log fields.  Each entry maps a header
    /// name (case-insensitive at runtime) to a mapping target (attribute,
    /// resource, etc.).  The header value is treated as a JSON string.
    #[serde(default)]
    pub header_mappings: Vec<HeaderMapping>,

    /// Query-string parameters to promote to HTTP headers.
    ///
    /// Keys are query-string parameter names, values are the HTTP header
    /// name to promote them to.  Promoted parameters are consumed from
    /// the query string before event processing, so they will **not**
    /// appear as event fields in GET requests.
    ///
    /// This is needed for the `sendBeacon` API which cannot set custom
    /// HTTP headers and instead promotes protocol parameters (apikey,
    /// content-type, etc.) to query-string parameters.
    #[serde(default)]
    pub query_string_headers: HashMap<String, String>,

    // -- Field mapping (unchanged from before) -----------------------------
    /// Ordered list of field mapping rules.  Rules are evaluated in order
    /// during a single pass over each JSON object.  The first matching rule
    /// wins for a given field path.
    #[serde(default)]
    pub rules: Vec<FieldRule>,

    /// How unmapped fields are handled.
    ///
    /// - `body` (default): preserve unmapped fields in log body, including
    ///   unhandled children of mapped containers with their original nesting.
    /// - `attribute`: write one log attribute containing a JSON array
    ///   of resolved unmapped field paths.
    /// - `drop`: silently discard unmapped fields.
    #[serde(default)]
    pub unmapped: UnmappedConfig,

    /// Per-event-name overrides.  Keys are event name patterns:
    ///
    /// - An exact string (e.g. `"Ms.Web.ClientPerf"`) matches that event
    ///   name literally.
    /// - A glob ending in `*` (e.g. `"Ms.Web.*"`) matches any event whose
    ///   name starts with the prefix.
    ///
    /// Override rules are **merged** with the base `rules`: paths that
    /// appear in the override replace the corresponding base rule; paths
    /// not present in the override keep the base rule.  The override can
    /// also add entirely new paths.  Rules with only metadata (`required`,
    /// `allowed_keys`) but no `target` are overlaid onto the base rule
    /// without replacing its target/validate.
    ///
    /// Fields marked `required` in the merged rule set are checked *after*
    /// the mapping pass.  If a required field was not present in the event,
    /// the configured `on_missing` action is taken (reject the event or
    /// inject a default value).
    #[serde(default)]
    pub event_overrides: HashMap<String, EventOverride>,

    // -- Event name validation -----------------------------------------------
    /// Validation level applied to the *final* (post-transform) event name.
    ///
    /// - `none` (default) -- no validation.
    /// - `printable_ascii` -- every character must be printable Basic Latin
    ///   (U+0021 .. U+007E).
    /// - `otel_semconv` -- strict OTel Semantic Conventions tooling rules:
    ///   lowercase Latin + numeric + underscore + dot, starts with a letter,
    ///   ends with an alphanumeric, no consecutive delimiters.
    #[serde(default)]
    pub event_name_validation: EventNameValidation,

    /// Event name patterns that cause the *entire event* to be silently
    /// dropped (not forwarded, not counted as rejected).
    ///
    /// Patterns follow the same syntax as `event_overrides` keys:
    /// - An exact string (e.g. `"InternalDiagnostics"`) matches literally.
    /// - A glob ending in `*` (e.g. `"Microsoft.Internal.*"`) matches any
    ///   event whose (post-transform) name starts with the prefix.
    #[serde(default)]
    pub drop_events: Vec<String>,

    // -- Response format ---------------------------------------------------
    /// Shape of the HTTP success response body.  Different protocols
    /// expect different JSON layouts.
    #[serde(default)]
    pub response_format: ResponseFormat,

    // -- iKey filtering ----------------------------------------------------
    /// Instrumentation key (iKey) filtering configuration.
    ///
    /// When set, each event's iKey is checked against the filter rules.
    /// Keys can be accepted, rejected, or silently ignored, with
    /// optional per-key event-name allow-lists.
    ///
    /// Supports inline rules or an external YAML file reference:
    ///
    /// ```yaml
    /// # Inline:
    /// ikey_filter:
    ///   keys:
    ///     "*": { action: accept }
    ///
    /// # External file:
    /// ikey_filter:
    ///   filter_file: "ikey-rules.yaml"
    /// ```
    ///
    /// See [`crate::receivers::microsoft_json_receiver::ikey_filter`]
    /// for the full configuration schema and matching rules.
    #[serde(default)]
    pub ikey_filter: Option<IkeyFilterSource>,

    // -- Environment variable mapping ----------------------------------------
    /// Maps environment variable names to OTel resource attribute keys.
    ///
    /// Values are resolved once at endpoint compile time.  Environment
    /// variables that are unset or empty are silently skipped.
    ///
    /// ```yaml
    /// env_resource_attrs:
    ///   CONTAINER_APP_NAME: "ms.container_app.name"
    ///   CONTAINER_APP_REVISION: "ms.container_app.revision"
    /// ```
    #[serde(default)]
    pub env_resource_attrs: HashMap<String, String>,
}

/// Validation level for the final event name.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EventNameValidation {
    /// No validation (accept anything).
    #[default]
    None,
    /// Every character must be in the printable ASCII range U+0021 .. U+007E.
    /// This matches the OTel recommendation for application developers.
    PrintableAscii,
    /// Strict OTel Semantic Conventions tooling rules:
    /// `[a-z0-9._]`, must start with `[a-z]`, end with `[a-z0-9]`,
    /// no consecutive delimiters (`..`, `__`, `._`, `_.`).
    OtelSemconv,
}

/// How the POST request body is parsed into individual events.
#[derive(Debug, Clone, Copy, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum BodyFormat {
    /// Newline-delimited JSON: each `\n`-separated line is a JSON object
    /// representing one event.
    #[default]
    LineSeparatedJson,
    /// A JSON array of event objects (or a single object).
    JsonArray,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct BodyFormats(pub Vec<BodyFormat>);

impl Default for BodyFormats {
    fn default() -> Self {
        Self(vec![BodyFormat::default()])
    }
}

impl<'de> Deserialize<'de> for BodyFormats {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum OneOrMany {
            One(BodyFormat),
            Many(Vec<BodyFormat>),
        }

        Ok(Self(match OneOrMany::deserialize(deserializer)? {
            OneOrMany::One(format) => vec![format],
            OneOrMany::Many(formats) => formats,
        }))
    }
}

/// How fields that do not match any mapping rule are handled.
#[derive(Debug, Clone, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub(crate) struct UnmappedConfig {
    /// Strategy for unmapped fields.
    #[serde(default)]
    pub mode: UnmappedMode,

    /// Attribute key to use when `mode=attribute`.
    #[serde(default = "default_unmapped_attribute_key")]
    pub attribute_key: String,
}

impl Default for UnmappedConfig {
    fn default() -> Self {
        Self {
            mode: UnmappedMode::default(),
            attribute_key: default_unmapped_attribute_key(),
        }
    }
}

/// Unmapped-field strategy.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum UnmappedMode {
    /// Preserve unmapped fields in log body.
    #[default]
    Body,
    /// Emit the paths of unmapped fields in a log attribute.
    Attribute,
    /// Discard unmapped fields.
    Drop,
}

fn default_unmapped_attribute_key() -> String {
    "unmapped.fields".to_string()
}

/// How GET query-string parameters are interpreted.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum GetFormat {
    /// Each key=value pair in the query string becomes a field in a single
    /// JSON event object.  Values that parse as JSON are used as-is,
    /// otherwise they become strings.
    #[default]
    EventFields,
    /// GET is not supported for this endpoint (returns 405).
    Disabled,
}

/// Shape of the HTTP success response body.
#[derive(Debug, Clone, Deserialize, Default, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ResponseFormat {
    /// Minimal JSON: `{}` (default).
    #[default]
    Minimal,
    /// OneCollector / 1DS response:
    /// `{"acc":<accepted>,"webResult":{"msfpc":"","mc1":"","anid":"","authError":"","authLoginUrl":""}}`
    /// If there are rejected events, includes `"rej":<rejected>`.
    OneCollector,
    /// Azure Monitor / Application Insights response:
    /// `{"itemsReceived":<total>,"itemsAccepted":<accepted>,"appId":"","errors":[]}`
    AzureMonitor,
}

/// Configuration for core CORS response headers.
///
/// `Access-Control-Max-Age` and `Cache-Control` share a common
/// `max_age_seconds` value.  The `cache_control` string supports a
/// `<max-age>` token that is replaced with the resolved
/// `max_age_seconds` value at compile time.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct CorsConfig {
    /// Whether to include `Access-Control-Allow-Credentials: true`.
    /// Defaults to `true`.
    #[serde(default = "default_cors_true")]
    pub allow_credentials: bool,

    /// Value for `Access-Control-Max-Age` (in seconds).
    /// Also used to resolve the `<max-age>` token in `cache_control`.
    /// Defaults to `3600`.
    #[serde(default = "default_cors_max_age")]
    pub max_age_seconds: u64,

    /// `Cache-Control` header value.  The literal `<max-age>` is
    /// replaced with the resolved `max_age_seconds` value.
    /// Defaults to `"no-cache, no-store, max-age=<max-age>"`.
    #[serde(default = "default_cors_cache_control")]
    pub cache_control: String,

    /// Value of `Access-Control-Allow-Headers`.  Comma-separated list
    /// that is returned verbatim.
    #[serde(default)]
    pub allow_headers: Option<String>,

    /// Value of `Access-Control-Expose-Headers`.
    #[serde(default)]
    pub expose_headers: Option<String>,

    /// Extra static headers to include in every response (including
    /// OPTIONS).  Keys are header names, values are header values.
    #[serde(default)]
    pub extra_response_headers: HashMap<String, String>,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allow_credentials: true,
            max_age_seconds: 3600,
            cache_control: default_cors_cache_control(),
            allow_headers: None,
            expose_headers: None,
            extra_response_headers: HashMap::new(),
        }
    }
}

fn default_cors_true() -> bool {
    true
}

fn default_cors_max_age() -> u64 {
    3600
}

pub(crate) fn default_cors_cache_control() -> String {
    "no-cache, no-store, max-age=<max-age>".to_owned()
}

/// Maps an HTTP header (or promoted query parameter) to a log field.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct HeaderMapping {
    /// Header name (matched case-insensitively at runtime).
    pub header: String,
    /// Where to place the header value.
    pub target: MappingTarget,
}

// ---------------------------------------------------------------------------
// Per-event-name overrides
// ---------------------------------------------------------------------------

/// Override configuration for events matching a specific name (or pattern).
///
/// By default, rules listed here are **merged** with the base endpoint rules:
/// - A rule whose `path` matches an existing base rule *replaces* it.
/// - A rule with `path` and only metadata (`required`, `allowed_keys`) but
///   no `target` is a *metadata overlay* -- the base rule's target/validate
///   are kept and the metadata fields are applied on top.
/// - A rule whose `path` does not exist in the base is *added*.
/// - Base rules whose `path` does not appear here are kept unchanged.
///
/// Set `inherit_rules` to `false` to compile only the rules listed here.
///
/// Fields marked `required` in the merged rules are checked after the
/// mapping pass completes.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct EventOverride {
    /// Action to take when the event name matches this override.
    /// Defaults to `accept` (process normally with the merged rules).
    #[serde(default)]
    pub action: EventAction,
    /// Whether base endpoint rules are inherited before applying this
    /// override's rules. Defaults to `true`.
    #[serde(default = "default_inherit_rules")]
    pub inherit_rules: bool,
    /// Additional or replacement field mapping rules.
    #[serde(default)]
    pub rules: Vec<FieldRule>,
}

impl Default for EventOverride {
    fn default() -> Self {
        Self {
            action: EventAction::default(),
            inherit_rules: true,
            rules: Vec::new(),
        }
    }
}

const fn default_inherit_rules() -> bool {
    true
}

/// Action to take when an event matches an override pattern.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum EventAction {
    /// Process the event normally using the (possibly overridden) rules.
    #[default]
    Accept,
    /// Silently drop the event (not counted as rejected).
    Drop,
    /// Reject the entire HTTP request payload.
    Reject,
}

/// Action when a required field is missing.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum OnMissing {
    /// Reject (drop) the entire event.
    #[default]
    Reject,
    /// Inject a default value and continue processing.
    Default,
}

/// Marks a field as required within an event override.  Checked after the
/// mapping pass; if the field was not present in the JSON event, the
/// configured `on_missing` action is taken.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Required {
    /// What to do when the field is missing.
    #[serde(default)]
    pub on_missing: OnMissing,

    /// Default value to inject when `on_missing` is `Default`.
    /// Ignored when `on_missing` is `Reject`.
    #[serde(default)]
    pub default_value: Option<serde_json::Value>,
}

/// Legacy type kept only for backward-compatibility. New configs should use
/// `EndpointConfig` directly.
///
/// Top-level mapping configuration for a single endpoint.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct MappingConfig {
    /// Human-readable label for this mapping (used in diagnostics).
    #[serde(default)]
    #[allow(dead_code)]
    pub name: String,

    /// Ordered list of field mapping rules.  Rules are evaluated in order
    /// during a single pass over each JSON object.  The first matching rule
    /// wins for a given field path.
    #[serde(default)]
    pub rules: Vec<FieldRule>,
}

/// A single field-mapping rule.
///
/// Each rule binds a *source* JSON path to a *target* location in the OTel log
/// record, with optional validation constraints.
///
/// Rules may be nested: a rule with `children` defines sub-rules whose paths
/// are relative to this rule's path.  During compilation, the tree is
/// flattened into absolute paths.  For example:
///
/// ```yaml
/// - path: "ext"
///   children:
///     - path: "dt"
///       children:
///         - path: "traceId"
///           target: { kind: trace_id }
/// ```
///
/// is equivalent to the flat form `path: "ext.dt.traceId"`.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct FieldRule {
    /// Dot-separated JSON path to match (e.g. `"data.baseType"`,
    /// `"tags.*"`, `"ext.dt.traceId"`).
    ///
    /// Two wildcard forms are supported:
    /// - `"tags.*"` -- matches every immediate child of `tags` and applies
    ///   the rule once per child.
    /// - `"ext.*.*"` -- matches two levels of nesting (group + field).
    ///
    /// When this rule is a child of another rule, the path is relative
    /// to the parent (e.g. `"traceId"` under a parent with path `"ext.dt"`
    /// becomes `"ext.dt.traceId"` after flattening).
    pub path: String,

    /// Where to place the resolved value.  If omitted (only valid when
    /// `children` is set), defaults to `Drop`.
    #[serde(default)]
    pub target: Option<MappingTarget>,

    /// Optional validation to apply *before* the value is written to the
    /// target. The validation's failure policy determines whether to skip
    /// the field or reject the entire event.
    #[serde(default)]
    pub validate: Option<Validation>,

    /// If set, restricts the children of this object path to only the
    /// listed keys.  Any child key not in the list causes validation
    /// failure for the entire event.
    #[serde(default)]
    pub allowed_keys: Option<Vec<String>>,

    /// Marks this field as required within an event override.  When present,
    /// the field is checked after the mapping pass and the configured
    /// `on_missing` action is taken if the field was absent.
    #[serde(default)]
    pub required: Option<Required>,

    /// Additional targets that receive the same (already-validated) value.
    /// Each entry may optionally transform the value before writing.
    /// See [`AlsoTarget`] for details.
    #[serde(default)]
    pub also: Vec<AlsoTarget>,

    /// Nested child rules whose paths are relative to this rule's path.
    /// The tree is flattened into absolute paths during compilation.
    #[serde(default)]
    pub children: Option<Vec<FieldRule>>,
}

/// Configuration for stripping a dot-separated segment (typically an
/// instrumentation key) from a dotted event name.
///
/// Supports two YAML forms:
///
/// - **Simple** -- `strip_segment: 2` -- strips the segment at index 2
///   only when it looks like an instrumentation key (GUID / hex string
///   of 8+ hex digits).
/// - **Detailed** -- strips the segment only when its value equals the
///   value of the given JSON field in the event. GUID keys compare without
///   hyphens and ignoring hex case; non-GUID keys compare exactly:
///   ```yaml
///   strip_segment:
///     index: 2
///     match_field: "iKey"
///   ```
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub(crate) enum StripSegment {
    /// Simple form: just the 0-indexed segment position.
    Index(usize),
    /// Detailed form: index + optional match field.
    Config {
        /// 0-indexed dot-separated segment to strip.
        index: usize,
        /// JSON field path in the event whose value must equal the
        /// segment for it to be stripped (GUID representations are normalized).
        /// If omitted, falls back to the GUID / hex-string heuristic.
        #[serde(default)]
        match_field: Option<String>,
    },
}

impl StripSegment {
    /// The 0-indexed segment position to consider stripping.
    pub fn index(&self) -> usize {
        match self {
            Self::Index(i) => *i,
            Self::Config { index, .. } => *index,
        }
    }

    /// Optional JSON field whose runtime value must match the segment.
    pub fn match_field(&self) -> Option<&str> {
        match self {
            Self::Index(_) => None,
            Self::Config { match_field, .. } => match_field.as_deref(),
        }
    }

    /// Checks the candidate segment against the caller-resolved original field.
    /// A configured match field must be present; non-GUID keys remain case-sensitive.
    /// The future mapping engine must use this comparison before stripping.
    pub fn matches_segment(&self, segment: &str, field_value: Option<&str>) -> bool {
        if self.match_field().is_some() {
            let Some(field_value) = field_value else {
                return false;
            };
            if segment == field_value {
                return true;
            }
            return match (normalized_guid(segment), normalized_guid(field_value)) {
                (Some(segment), Some(field)) => segment == field,
                _ => false,
            };
        }
        normalized_guid(segment).is_some()
            || (segment.len() >= 8 && segment.bytes().all(|byte| byte.is_ascii_hexdigit()))
    }
}

fn normalized_guid(value: &str) -> Option<[u8; 32]> {
    if value.len() != 32 && value.len() != 36 {
        return None;
    }
    let mut normalized = [0; 32];
    let mut offset = 0;
    for (index, byte) in value.bytes().enumerate() {
        if value.len() == 36 && matches!(index, 8 | 13 | 18 | 23) {
            if byte != b'-' {
                return None;
            }
        } else {
            if !byte.is_ascii_hexdigit() {
                return None;
            }
            normalized[offset] = byte.to_ascii_lowercase();
            offset += 1;
        }
    }
    Some(normalized)
}

/// Destination for a mapped value.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum MappingTarget {
    /// Map to the OTel log `event_name` column.
    EventName {
        /// Optional static prefix prepended to the value.
        #[serde(default)]
        prefix: Option<String>,
        /// If set, extract only the substring after the last occurrence
        /// of this delimiter (e.g. `"."` to turn
        /// `"Microsoft.ApplicationInsights.Event"` into `"Event"`).
        #[serde(default)]
        split_last: Option<String>,
        /// If set, remove the Nth dot-separated segment (0-indexed)
        /// from the value before any other transforms.  For example,
        /// `strip_segment: 2` turns
        /// `"Microsoft.ApplicationInsights.{iKey}.Request"`
        /// into `"Microsoft.ApplicationInsights.Request"`.
        ///
        /// See [`StripSegment`] for the simple and detailed forms.
        #[serde(default)]
        strip_segment: Option<StripSegment>,
    },

    /// Map to `time_unix_nano` (expects an RFC 3339 string).
    Timestamp,

    /// Map to `observed_time_unix_nano` (expects an RFC 3339 string).
    ObservedTimestamp,

    /// Map to `trace_id` (expects a 32-hex-char string).
    TraceId,

    /// Map to `span_id` (expects a 16-hex-char string).
    SpanId,

    /// Map to `flags` (expects an integer or numeric string).
    TraceFlags,

    /// Map to `severity_number` / `severity_text`.
    Severity {
        /// Static mapping from scalar field value -> `(severity_number, severity_text)`.
        /// Numeric and boolean values use their JSON string representation as the key.
        /// If the runtime value is not found in this map, it falls back to
        /// `UNSPECIFIED (0)`.
        #[serde(default)]
        map: HashMap<String, SeverityEntry>,
    },

    /// Map to a log attribute.
    Attribute {
        /// Attribute key name.  If omitted, the source path is used
        /// (with dots replaced by the `separator`, defaulting to `.`).
        #[serde(default)]
        key: Option<String>,
        /// Prefix prepended to the computed key.
        #[serde(default)]
        prefix: Option<String>,
    },

    /// Map to a **resource** attribute.  Events with different resource
    /// attribute values will be separated into distinct resource groups.
    Resource {
        /// Attribute key name on the resource.  If omitted, the source
        /// path is used (with optional `prefix`).
        #[serde(default)]
        key: Option<String>,
        /// Prefix prepended to the computed key.
        #[serde(default)]
        prefix: Option<String>,
    },

    /// Map to **instrumentation scope** name or version.
    Scope {
        /// Which scope field this maps to.
        field: ScopeField,
        /// Optional split strategy to extract both name and version
        /// from a single value (only used with `name_version`).
        #[serde(default)]
        split: Option<ScopeSplit>,
    },

    /// Drop this field entirely (do not include in body).
    Drop,
}

/// Which part of the instrumentation scope to set.
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScopeField {
    Name,
    Version,
    /// Extract **both** name and version from a single value using the
    /// `split` strategy on the parent `Scope` target.
    NameVersion,
}

/// Strategy for splitting a combined string into (name, version).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ScopeSplit {
    /// Split on the **last** occurrence of the given character.
    /// E.g. `"javascript:3.4.1"` with `last_char: ":"` ->
    /// name = `"javascript"`, version = `"3.4.1"`.
    LastChar(String),
    /// Split on the **first** `"-"` (or given char) that is immediately
    /// followed by an ASCII digit.  Handles SDK strings like
    /// `"1DS-Web-JS-4.4.1"` -> name = `"1DS-Web-JS"`, version = `"4.4.1"`.
    DashSemver,
}

/// A `(severity_number, severity_text)` pair for the severity lookup map.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct SeverityEntry {
    pub number: i32,
    pub text: String,
}

// ---------------------------------------------------------------------------
// Multi-target support (also)
// ---------------------------------------------------------------------------

/// An additional mapping target applied to the same (already-validated) value.
///
/// Used when a single source field must be written to multiple destinations
/// during a migration -- e.g. the same value as both the `event_name` and an
/// attribute, possibly with a transform on one of the copies.
///
/// ```yaml
/// - path: "name"
///   target: { kind: event_name }
///   also:
///     - target: { kind: attribute, key: "event.name" }
///     - target: { kind: attribute, key: "event.shortName" }
///       transform: { kind: split_take, delimiter: ".", index: -1 }
/// ```
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct AlsoTarget {
    /// The additional destination for the value.
    pub target: MappingTarget,

    /// Optional transformation applied to the validated value before
    /// writing it to this target.  If omitted, the value is written as-is.
    #[serde(default)]
    pub transform: Option<ValueTransform>,
}

/// A transformation applied to a value before it is written to an
/// [`AlsoTarget`].
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
pub(crate) enum ValueTransform {
    /// Split the string value by a delimiter and take one part.
    ///
    /// ```yaml
    /// transform: { kind: split_take, delimiter: ".", index: -1 }
    /// ```
    SplitTake {
        /// The delimiter string to split on.
        delimiter: String,
        /// 0-based index of the part to take.  Negative indices count
        /// from the end (`-1` = last element).
        #[serde(default)]
        index: i32,
    },

    /// Format the value using a template string.
    ///
    /// Supports the `{value}` placeholder which is replaced with the
    /// string representation of the original value.
    ///
    /// ```yaml
    /// transform: { kind: format, template: "legacy.{value}" }
    /// ```
    Format {
        /// Template string containing `{value}` placeholders.
        template: String,
    },

    /// Extract the **name** portion from a dash-semver SDK string.
    ///
    /// Splits at the first `-` followed by an ASCII digit.
    /// E.g. `"1DS-Web-JS-4.4.1"` -> `"1DS-Web-JS"`.
    ///
    /// ```yaml
    /// transform: { kind: dash_semver_name }
    /// ```
    DashSemverName,

    /// Extract the **version** portion from a dash-semver SDK string.
    ///
    /// Splits at the first `-` followed by an ASCII digit.
    /// E.g. `"1DS-Web-JS-4.4.1"` -> `"4.4.1"`.
    ///
    /// ```yaml
    /// transform: { kind: dash_semver_version }
    /// ```
    DashSemverVersion,
}

impl ValueTransform {
    /// Apply the transform to a JSON value, returning the transformed
    /// value or `None` if the transform cannot be applied (e.g. the
    /// value is not a string for `SplitTake`).
    pub fn apply(&self, val: &serde_json::Value) -> Option<serde_json::Value> {
        use serde_json::Value;
        match self {
            Self::SplitTake { delimiter, index } => {
                let s = val.as_str()?;
                let parts: Vec<&str> = s.split(delimiter.as_str()).collect();
                let idx = if *index < 0 {
                    let pos = parts.len() as i32 + *index;
                    if pos < 0 {
                        return None;
                    }
                    pos as usize
                } else {
                    *index as usize
                };
                parts.get(idx).map(|p| Value::String((*p).to_owned()))
            }
            Self::Format { template } => {
                let s = match val {
                    Value::String(s) => s.clone(),
                    Value::Number(n) => n.to_string(),
                    Value::Bool(b) => b.to_string(),
                    _ => return None,
                };
                Some(Value::String(template.replace("{value}", &s)))
            }
            Self::DashSemverName => {
                let s = val.as_str()?;
                let (name, _) = split_dash_semver(s);
                Some(Value::String(name.to_owned()))
            }
            Self::DashSemverVersion => {
                let s = val.as_str()?;
                let (_, version) = split_dash_semver(s);
                if version.is_empty() {
                    None
                } else {
                    Some(Value::String(version.to_owned()))
                }
            }
        }
    }
}

/// Find the first `-` that is immediately followed by an ASCII digit and
/// split there. E.g. `"1DS-Web-JS-4.4.1"` -> `("1DS-Web-JS", "4.4.1")`.
fn split_dash_semver(raw: &str) -> (&str, &str) {
    let bytes = raw.as_bytes();
    for i in 0..bytes.len().saturating_sub(1) {
        if bytes[i] == b'-' && bytes[i + 1].is_ascii_digit() {
            return (&raw[..i], &raw[i + 1..]);
        }
    }
    (raw, "")
}

// ---------------------------------------------------------------------------
// Validation
// ---------------------------------------------------------------------------

/// Validation constraints for a single field.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub(crate) struct Validation {
    /// Expected JSON value type.  If the runtime value does not match and
    /// no conversion is possible, the behaviour depends on
    /// `on_validation_failure`.
    #[serde(default)]
    pub expected_type: Option<ExpectedType>,

    /// Maximum string length (in bytes).  Applies *after* any type
    /// conversion.  Behaviour when exceeded is controlled by
    /// `on_length_exceeded`.
    #[serde(default)]
    pub max_length: Option<usize>,

    /// What to do when `max_length` is exceeded.
    #[serde(default)]
    pub on_length_exceeded: LengthExceeded,

    /// What to do when type or other validation fails.
    /// Default is `skip` (omit the field and keep the event).
    #[serde(default)]
    pub on_validation_failure: ValidationFailure,
}

/// Expected JSON value type (with optional coercion).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ExpectedType {
    String,
    Integer,
    Float,
    Boolean,
    /// Accept any type but coerce to string before writing.
    StringCoerce,
}

/// Behaviour when a string value exceeds `max_length`.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum LengthExceeded {
    /// Truncate the value to `max_length` bytes (on a char boundary).
    #[default]
    Truncate,
    /// Reject the entire event.
    Reject,
}

/// Behaviour when type or other validation fails.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ValidationFailure {
    /// Reject the entire event.
    Reject,
    /// Skip this field (drop it from the output) but keep the event.
    /// This is the default -- a telemetry receiver should be resilient
    /// to unexpected field types.
    #[default]
    Skip,
}

// ---------------------------------------------------------------------------
// Metrics
// ---------------------------------------------------------------------------

/// Microsoft JSON diagnostics not represented by shared receiver metrics.
///
/// Request outcomes, encoded payload size, and receiver-local duration use
/// [`ReceiverMetrics`]. These counters cover protocol-specific events and
/// startup behavior that cannot be derived from the shared boundary.
#[metric_set(name = "receiver.microsoft_json")]
#[derive(Debug, Default, Clone)]
pub struct MicrosoftJsonReceiverDiagnosticMetrics {
    /// Number of JSON events that failed parsing or endpoint validation.
    #[metric(unit = "{item}")]
    pub received_logs_invalid: Counter<u64>,
    /// Number of CORS preflight (`OPTIONS`) requests handled.
    #[metric(unit = "{request}")]
    pub cors_preflight_requests: Counter<u64>,
    /// Number of HTTP requests received with an encoded body.
    #[metric(unit = "{request}")]
    pub compressed_requests: Counter<u64>,
    /// Number of events rejected by the iKey filter.
    #[metric(unit = "{item}")]
    pub ikey_rejected: Counter<u64>,
    /// Number of events silently dropped by the iKey filter.
    #[metric(unit = "{item}")]
    pub ikey_ignored: Counter<u64>,
    /// Number of events rejected because their origin is not allowed.
    #[metric(unit = "{item}")]
    pub ikey_unauthorized: Counter<u64>,
    /// Number of events rejected for exceeding the endpoint event-size limit.
    #[metric(unit = "{item}")]
    pub events_oversized: Counter<u64>,
    /// Number of endpoint configuration files loaded at startup.
    #[metric(unit = "{item}")]
    pub config_endpoints_loaded: Counter<u64>,
    /// Number of endpoint configuration files that failed startup loading.
    #[metric(unit = "{error}")]
    pub config_endpoints_failed: Counter<u64>,
}

/// Shared boundary and Microsoft JSON-specific diagnostic metrics.
pub struct MicrosoftJsonReceiverMetrics {
    /// Shared receiver boundary metrics.
    pub boundary: ReceiverMetrics,
    /// Protocol-specific diagnostics not represented by the boundary.
    pub diagnostics: MetricSet<MicrosoftJsonReceiverDiagnosticMetrics>,
}

impl MicrosoftJsonReceiverMetrics {
    /// Registers all Microsoft JSON receiver metric sets.
    #[must_use]
    pub fn register(pipeline_ctx: &PipelineContext) -> Self {
        Self {
            boundary: ReceiverMetrics::register(pipeline_ctx),
            diagnostics: pipeline_ctx.register_metric_set(),
        }
    }

    /// Reports shared and receiver-specific metric sets.
    pub fn report(&mut self, reporter: &mut MetricsReporter) -> Result<(), TelemetryError> {
        self.boundary.report(reporter)?;
        reporter.report(&mut self.diagnostics)
    }

    /// Takes shared and receiver-specific terminal snapshots.
    #[must_use]
    pub fn terminal_snapshots(&mut self) -> Vec<MetricSetSnapshot> {
        let mut snapshots = self.boundary.terminal_snapshots();
        if !self.diagnostics.is_empty() {
            snapshots.extend(self.diagnostics.terminal_snapshots());
        }
        snapshots
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BodyFormat, Config, EndpointConfig, EnrichmentTarget, EventOverride, ResponseFormat,
        StripSegment, UnmappedConfig, UnmappedMode, ValueTransform,
    };
    use serde_json::json;
    use std::time::Duration;

    fn flattened_rules<'a>(
        prefix: &str,
        rules: &'a [super::FieldRule],
    ) -> Vec<(String, &'a super::FieldRule)> {
        let mut result = Vec::new();
        for rule in rules {
            let path = if prefix.is_empty() {
                rule.path.clone()
            } else {
                format!("{prefix}.{}", rule.path)
            };
            if let Some(children) = &rule.children {
                result.extend(flattened_rules(&path, children));
            }
            result.push((path, rule));
        }
        result
    }

    /// Scenario: browser messages and exceptions supply any of the five SDK severity levels.
    /// Guarantees: one inherited rule maps each level to OTel severity and retains the original value without a base-type fallback.
    #[test]
    fn browser_severity_mapping_preserves_explicit_levels() {
        let endpoint: EndpointConfig =
            serde_yaml::from_str(include_str!("azure-monitor-browser-events.yaml"))
                .expect("browser endpoint");
        let rules = flattened_rules("", &endpoint.rules);
        let severity_rules: Vec<_> = rules
            .iter()
            .filter(|(_, rule)| matches!(rule.target, Some(super::MappingTarget::Severity { .. })))
            .collect();
        assert_eq!(severity_rules.len(), 1);
        let (path, rule) = severity_rules[0];
        assert_eq!(path, "data.baseData.severityLevel");
        assert!(matches!(
            rule.validate
                .as_ref()
                .expect("type validation")
                .expected_type,
            Some(super::ExpectedType::Integer)
        ));
        let Some(super::MappingTarget::Severity { map }) = &rule.target else {
            panic!("severity target");
        };
        assert_eq!(map.len(), 5);
        for (level, number, text) in [
            ("0", 1, "TRACE"),
            ("1", 9, "INFO"),
            ("2", 13, "WARN"),
            ("3", 17, "ERROR"),
            ("4", 21, "FATAL"),
        ] {
            let entry = &map[level];
            assert_eq!((entry.number, entry.text.as_str()), (number, text));
        }
        assert_eq!(rule.also.len(), 1);
        assert!(matches!(
            &rule.also[0].target,
            super::MappingTarget::Attribute { key: Some(key), .. }
                if key == "ai.baseData.severityLevel"
        ));
        assert!(
            rule.required.is_none(),
            "missing severity must remain unspecified"
        );
        for event in [
            "Microsoft.ApplicationInsights.Exception",
            "Microsoft.ApplicationInsights.Message",
        ] {
            let event_override = &endpoint.event_overrides[event];
            assert!(event_override.inherit_rules);
            assert!(
                flattened_rules("", &event_override.rules)
                    .iter()
                    .all(|(path, _)| path != "data.baseData.severityLevel")
            );
        }
        assert!(rules.iter().any(|(path, rule)| path == "data.baseType"
            && matches!(&rule.target, Some(super::MappingTarget::Attribute { key: Some(key), .. }) if key == "ai.baseType")));
    }

    /// Scenario: exception arrays occur in either expanded endpoint's inherited or event-specific rules.
    /// Guarantees: body preservation is enabled and no ancestor, exact rule, or wildcard coerces, truncates, or drops exception details.
    #[test]
    fn expanded_examples_leave_exception_details_in_body() {
        for yaml in [
            include_str!("azure-monitor-browser-events.yaml"),
            include_str!("onecollector-1ds-endpoint.yaml"),
        ] {
            let endpoint: EndpointConfig = serde_yaml::from_str(yaml).expect("expanded endpoint");
            assert_eq!(endpoint.unmapped.mode, UnmappedMode::Body);
            for rules in std::iter::once(&endpoint.rules)
                .chain(endpoint.event_overrides.values().map(|entry| &entry.rules))
            {
                for (path, rule) in flattened_rules("", rules) {
                    assert_ne!(path, "data.baseData.*");
                    assert_ne!(path, "data.baseData.exceptions");
                    if path == "data" || path == "data.baseData" {
                        assert!(
                            rule.target.is_none(),
                            "{path} must not consume the container"
                        );
                        assert!(
                            rule.validate.is_none(),
                            "{path} must not coerce or truncate"
                        );
                        assert!(rule.allowed_keys.is_none(), "{path} must allow exceptions");
                    }
                }
            }
        }
    }

    /// Scenario: nested target, segment, or transform settings contain misspelled or unsupported options.
    /// Guarantees: YAML and JSON reject unknown fields instead of silently switching to defaults.
    #[test]
    fn nested_mapping_options_reject_unknown_fields() {
        for yaml in [
            "path: name\ntarget: { kind: event_name, strip_segment: { index: 2, match_filed: iKey } }",
            "path: name\ntarget: { kind: event_name, split_lats: '.' }",
            "path: name\ntarget: { kind: attribute, ky: name }",
            "path: name\ntarget: { kind: resource, prefex: name }",
            "path: name\ntarget: { kind: scope, field: name, splt: dash_semver }",
            "path: name\ntarget: { kind: severity, mpa: {} }",
            "path: name\nalso:\n  - target: { kind: attribute }\n    transform: { kind: split_take, delimiter: '.', indez: 1 }",
            "path: name\nalso:\n  - target: { kind: attribute }\n    transform: { kind: format, template: '{value}', unknown: true }",
        ] {
            assert!(
                serde_yaml::from_str::<super::FieldRule>(yaml).is_err(),
                "{yaml}"
            );
            let json: serde_json::Value = serde_yaml::from_str(yaml).expect("valid YAML syntax");
            assert!(
                serde_json::from_value::<super::FieldRule>(json).is_err(),
                "{yaml}"
            );
        }
    }

    /// Scenario: SDK envelope names contain dashless GUID keys while iKey retains hyphens.
    /// Guarantees: equivalent GUIDs match without normalizing missing, malformed, different, or non-GUID keys.
    #[test]
    fn strip_segment_matches_sdk_guid_representations() {
        let strip: StripSegment =
            serde_yaml::from_str("index: 2\nmatch_field: iKey").expect("strip config");
        let key = "abcdef01-2345-6789-abcd-ef0123456789";
        let name = "Microsoft.ApplicationInsights.ABCDEF0123456789ABCDEF0123456789.Pageview";
        let segment = name.split('.').nth(strip.index()).expect("key segment");
        assert!(strip.matches_segment(segment, Some(key)));
        assert!(strip.matches_segment(key, Some(segment)));
        assert!(!strip.matches_segment(segment, None));
        assert!(!strip.matches_segment(segment, Some("abcdef01-2345-6789-abcd-ef0123456780")));
        assert!(!strip.matches_segment(segment, Some("abcdef0-12345-6789-abcd-ef0123456789")));
        assert!(!strip.matches_segment(segment, Some("abcdef01-2345-6789-abcd-ef012345678z")));
        assert!(strip.matches_segment("example-key", Some("example-key")));
        assert!(!strip.matches_segment("example-key", Some("EXAMPLE-KEY")));
        assert!(!strip.matches_segment("example-key", Some("examplekey")));

        let heuristic = StripSegment::Index(2);
        assert!(heuristic.matches_segment(segment, None));
        assert!(heuristic.matches_segment(key, None));
        assert!(heuristic.matches_segment("1234abcd", None));
        assert!(!heuristic.matches_segment("Pageview", None));
    }

    /// Scenario: the browser page-view override deserializes with the canonical SDK name.
    /// Guarantees: page-view fields remain optional and retain validation when supplied.
    #[test]
    fn browser_pageview_overrides_match_sdk_contract() {
        let endpoint: EndpointConfig =
            serde_yaml::from_str(include_str!("azure-monitor-browser-events.yaml"))
                .expect("browser endpoint");
        assert!(
            !endpoint
                .event_overrides
                .contains_key("Microsoft.ApplicationInsights.PageView")
        );
        let pageview = &endpoint.event_overrides["Microsoft.ApplicationInsights.Pageview"];
        let fields = pageview.rules[0].children.as_ref().expect("data children")[0]
            .children
            .as_ref()
            .expect("baseData children");
        for path in ["name", "url", "duration", "id"] {
            let field = fields
                .iter()
                .find(|field| field.path == path)
                .expect("field");
            assert!(field.required.is_none(), "{path} must remain optional");
            assert!(
                field.validate.is_some(),
                "{path} must validate when supplied"
            );
        }
    }

    /// Scenario: field validation omits its failure policy in YAML or JSON.
    /// Guarantees: both formats default to skipping the field, while explicit rejection is retained.
    #[test]
    fn validation_failure_defaults_to_skip() {
        for validation in [
            serde_yaml::from_str::<super::Validation>("{}").expect("YAML validation"),
            serde_json::from_value::<super::Validation>(json!({})).expect("JSON validation"),
        ] {
            assert!(matches!(
                validation.on_validation_failure,
                super::ValidationFailure::Skip
            ));
        }
        let explicit: super::Validation =
            serde_yaml::from_str("on_validation_failure: reject").expect("explicit rejection");
        assert!(matches!(
            explicit.on_validation_failure,
            super::ValidationFailure::Reject
        ));
    }

    /// Scenario: receiver-local endpoint, filter, and pipeline examples are deserialized.
    /// Guarantees: shipped examples match the current schema and the local pipeline validates its receiver.
    #[test]
    fn local_examples_match_configuration_schema() {
        for yaml in [
            include_str!("onecollector-1ds-endpoint.yaml"),
            include_str!("azure-monitor-browser-events.yaml"),
            include_str!("azure-monitor-envelope.yaml"),
        ] {
            let endpoint: EndpointConfig = serde_yaml::from_str(yaml).expect("endpoint example");
            assert!(!endpoint.rules.is_empty());
        }
        let filter: super::super::ikey_filter::IkeyFilterFileConfig =
            serde_yaml::from_str(include_str!("ikey-rules.yaml")).expect("external rules example");
        assert!(!filter.keys.is_empty());
        let pipeline: serde_json::Value =
            serde_yaml::from_str(include_str!("pipeline-local-console.yaml"))
                .expect("pipeline example");
        let config: Config = serde_json::from_value(
            pipeline["groups"]["default"]["pipelines"]["main"]["nodes"]["receiver"]["config"]
                .clone(),
        )
        .expect("receiver config");
        config.validate().expect("valid receiver settings");
        assert_eq!(config.endpoints.len(), 3);

        let yaml = include_str!("azure-monitor-exporter.yaml");
        let _: otel_arrow_dfe_config::engine::OtelDataflowSpec =
            serde_yaml::from_str(yaml).expect("typed Azure Monitor pipeline");
        let pipeline: serde_json::Value =
            serde_yaml::from_str(yaml).expect("Azure Monitor pipeline example");
        let main = &pipeline["groups"]["default"]["pipelines"]["main"];
        let config: Config = serde_json::from_value(main["nodes"]["receiver"]["config"].clone())
            .expect("Azure Monitor receiver config");
        config.validate().expect("valid Azure Monitor receiver");
        assert_eq!(config.endpoints.len(), 2);
        for path in ["/v2/track", "/v2.1/track"] {
            assert_eq!(
                main["nodes"]["receiver"]["config"]["endpoints"][path]["config_file"],
                "azure-monitor-envelope.yaml"
            );
        }
        assert_eq!(
            main["nodes"]["azure-monitor-exporter"]["capabilities"]["bearer_token_provider"],
            "azure_identity"
        );
        assert_eq!(
            main["extensions"]["azure_identity"]["type"],
            "urn:microsoft:extension:azure_identity_auth"
        );
        assert_eq!(
            main["connections"],
            json!([{"from": "receiver", "to": "azure-monitor-exporter"}])
        );
        #[cfg(feature = "azure-monitor")]
        {
            let exporter: crate::exporters::azure_monitor_exporter::Config =
                serde_json::from_value(main["nodes"]["azure-monitor-exporter"]["config"].clone())
                    .expect("existing Azure Monitor exporter schema");
            exporter.validate().expect("valid existing exporter config");
        }
    }

    /// Scenario: endpoint response formats are read from YAML and JSON configuration.
    /// Guarantees: azure_monitor selects the Azure Monitor format and application_insights is rejected.
    #[test]
    fn response_format_uses_azure_monitor() {
        let yaml: EndpointConfig =
            serde_yaml::from_str("response_format: azure_monitor").expect("Azure Monitor YAML");
        let json: EndpointConfig =
            serde_json::from_value(json!({"response_format": "azure_monitor"}))
                .expect("Azure Monitor JSON");
        assert_eq!(yaml.response_format, ResponseFormat::AzureMonitor);
        assert_eq!(json.response_format, ResponseFormat::AzureMonitor);
        assert!(
            serde_yaml::from_str::<EndpointConfig>("response_format: application_insights")
                .is_err()
        );
        assert!(
            serde_json::from_value::<EndpointConfig>(
                json!({"response_format": "application_insights"})
            )
            .is_err()
        );
    }

    /// Scenario: unmapped configuration is constructed directly, omitted, or explicitly empty.
    /// Guarantees: every default path uses body mode and the same non-empty unmapped.fields attribute key.
    #[test]
    fn unmapped_defaults_match_deserialization() {
        let expected = UnmappedConfig {
            mode: UnmappedMode::Body,
            attribute_key: "unmapped.fields".to_owned(),
        };
        let yaml: UnmappedConfig = serde_yaml::from_str("{}").expect("empty YAML config");
        let json: UnmappedConfig = serde_json::from_value(json!({})).expect("empty JSON config");
        let omitted: EndpointConfig =
            serde_json::from_value(json!({})).expect("omitted unmapped config");
        let explicit: EndpointConfig =
            serde_json::from_value(json!({"unmapped": {}})).expect("explicit unmapped config");

        for config in [
            UnmappedConfig::default(),
            yaml,
            json,
            EndpointConfig::default().unmapped,
            omitted.unmapped,
            explicit.unmapped,
        ] {
            assert_eq!(config, expected);
        }
        let custom: UnmappedConfig = serde_json::from_value(json!({
            "mode": "attribute",
            "attribute_key": "custom.unmapped"
        }))
        .expect("custom unmapped config");
        assert_eq!(custom.mode, UnmappedMode::Attribute);
        assert_eq!(custom.attribute_key, "custom.unmapped");
    }

    /// Scenario: A Microsoft JSON receiver config omits both timeout settings.
    /// Guarantees: Header reads default to 5 seconds while request processing defaults to 30 seconds.
    #[test]
    fn config_uses_distinct_timeout_defaults() {
        let config: Config = serde_json::from_value(json!({
            "listening_addr": "127.0.0.1:4319"
        }))
        .unwrap();

        assert_eq!(config.header_read_timeout, Some(Duration::from_secs(5)));
        assert_eq!(config.timeout, Some(Duration::from_secs(30)));
        assert_eq!(config.max_request_body_size, 3 * 1024 * 1024);
        assert_eq!(config.max_concurrent_requests, 256);
        assert_eq!(config.allowed_origins, ["*"]);
        assert_eq!(config.user_agent_attr_target, EnrichmentTarget::Resource);
        assert_eq!(config.http_attr_target, EnrichmentTarget::Resource);
        config.validate().expect("default config is valid");
    }

    /// Scenario: A Microsoft JSON receiver config supplies a custom direct header read timeout.
    /// Guarantees: Humantime values override the 5-second default.
    #[test]
    fn config_deserializes_header_read_timeout() {
        let config: Config = serde_json::from_value(json!({
            "listening_addr": "127.0.0.1:4319",
            "header_read_timeout": "2s"
        }))
        .unwrap();

        assert_eq!(config.header_read_timeout, Some(Duration::from_secs(2)));
    }

    /// Scenario: A Microsoft JSON receiver config still uses the removed request_host_attr field.
    /// Guarantees: Deserialization rejects the legacy field instead of silently ignoring it.
    #[test]
    fn config_rejects_removed_request_host_attr() {
        let result = serde_json::from_value::<Config>(json!({
            "listening_addr": "127.0.0.1:4319",
            "request_host_attr": "http.host"
        }));

        assert!(result.is_err());
    }

    /// Scenario: An endpoint uses the legacy scalar body_format syntax.
    /// Guarantees: The scalar remains valid and produces one configured body format.
    #[test]
    fn endpoint_config_deserializes_single_body_format() {
        let config: EndpointConfig = serde_json::from_value(json!({
            "body_format": "json_array"
        }))
        .unwrap();

        assert_eq!(config.body_format.0, vec![BodyFormat::JsonArray]);
    }

    /// Scenario: An endpoint configures multiple accepted POST body formats.
    /// Guarantees: Every listed format is retained in configuration order.
    #[test]
    fn endpoint_config_deserializes_multiple_body_formats() {
        let config: EndpointConfig = serde_json::from_value(json!({
            "body_format": ["line_separated_json", "json_array"]
        }))
        .unwrap();

        assert_eq!(
            config.body_format.0,
            vec![BodyFormat::LineSeparatedJson, BodyFormat::JsonArray]
        );
    }

    /// Scenario: An event override omits inherit_rules.
    /// Guarantees: Existing configurations continue to inherit endpoint rules by default.
    #[test]
    fn event_override_inherits_rules_by_default() {
        let event_override: EventOverride = serde_json::from_value(json!({})).unwrap();

        assert!(event_override.inherit_rules);
        assert!(EventOverride::default().inherit_rules);
    }

    /// Scenario: An event override explicitly disables rule inheritance.
    /// Guarantees: The opt-out value is retained during configuration parsing.
    #[test]
    fn event_override_can_disable_rule_inheritance() {
        let event_override: EventOverride = serde_json::from_value(json!({
            "inherit_rules": false
        }))
        .unwrap();

        assert!(!event_override.inherit_rules);
    }

    /// Scenario: Receiver-level settings deserialize but contain unsafe zero values or malformed endpoint references.
    /// Guarantees: Semantic validation rejects each invalid configuration before runtime construction.
    #[test]
    fn validation_rejects_invalid_receiver_settings() {
        for (config, expected) in [
            (
                json!({
                    "listening_addr": "127.0.0.1:4319",
                    "max_request_body_size": 0
                }),
                "max_request_body_size",
            ),
            (
                json!({
                    "listening_addr": "127.0.0.1:4319",
                    "timeout": "0s"
                }),
                "timeout",
            ),
            (
                json!({
                    "listening_addr": "127.0.0.1:4319",
                    "endpoints": {
                        "OneCollector/1.0": {"config_file": "endpoint.yaml"}
                    }
                }),
                "must start with '/'",
            ),
        ] {
            let config: Config = serde_json::from_value(config).expect("config deserializes");
            let error = config.validate().expect_err("config must be rejected");
            assert!(error.to_string().contains(expected), "{error}");
        }
    }

    /// Scenario: A receiver supplies every retained receiver-level configuration setting.
    /// Guarantees: Listener, limits, endpoints, enrichment targets, and attributes deserialize and validate together.
    #[test]
    fn complete_receiver_config_deserializes() {
        let config: Config = serde_json::from_value(json!({
            "listening_addr": "0.0.0.0:4319",
            "allowed_origins": ["https://example.com"],
            "max_request_body_size": 1024,
            "max_concurrent_requests": 8,
            "timeout": "10s",
            "header_read_timeout": "2s",
            "endpoints": {
                "/v2/track": {"config_file": "application-insights.yaml"}
            },
            "user_agent_attr_target": "log_attribute",
            "http_attr_target": "resource",
            "instance_id_attr": "service.instance.id",
            "origin_attr": "http.request.header.origin"
        }))
        .expect("complete receiver config");

        config.validate().expect("complete config is valid");
        assert_eq!(
            config.instance_id_attr.as_deref(),
            Some("service.instance.id")
        );
    }

    /// Scenario: An endpoint uses the complete HTTP, mapping, validation, override, CORS, and iKey configuration.
    /// Guarantees: Every retained endpoint configuration surface deserializes without runtime implementation.
    #[test]
    fn complete_endpoint_config_deserializes() {
        let config: EndpointConfig = serde_json::from_value(json!({
            "name": "application-insights",
            "max_request_body_size": 102400,
            "max_event_size": 65536,
            "filtered_item_size": true,
            "allowed_methods": ["POST", "GET"],
            "accepted_content_types": ["application/json"],
            "body_format": ["line_separated_json", "json_array"],
            "get_format": "event_fields",
            "cors": {
                "allow_credentials": false,
                "max_age_seconds": 600,
                "cache_control": "max-age=<max-age>",
                "allow_headers": "Content-Type",
                "expose_headers": "X-Test",
                "extra_response_headers": {"X-Test": "value"}
            },
            "header_mappings": [{
                "header": "apikey",
                "target": {"kind": "resource", "key": "ai.ikey"}
            }],
            "query_string_headers": {"apikey": "apikey"},
            "rules": [{
                "path": "name",
                "target": {"kind": "event_name", "strip_segment": 2},
                "validate": {
                    "expected_type": "string",
                    "max_length": 1024,
                    "on_length_exceeded": "reject",
                    "on_validation_failure": "reject"
                },
                "also": [{
                    "target": {"kind": "attribute", "key": "event.name"},
                    "transform": {"kind": "dash_semver_name"}
                }]
            }],
            "unmapped": {"mode": "attribute", "attribute_key": "unmapped"},
            "event_overrides": {
                "Microsoft.ApplicationInsights.*": {
                    "action": "accept",
                    "inherit_rules": false,
                    "rules": [{
                        "path": "time",
                        "target": {"kind": "timestamp"},
                        "required": {
                            "on_missing": "default",
                            "default_value": "1970-01-01T00:00:00Z"
                        }
                    }]
                }
            },
            "event_name_validation": "printable_ascii",
            "drop_events": ["Microsoft.Internal.*"],
            "response_format": "azure_monitor",
            "ikey_filter": {
                "keys": {
                    "o:accepted": {
                        "action": "accept",
                        "allowed_events": ["Microsoft.*"],
                        "require_origin": true,
                        "allowed_origins": ["https://example.com"],
                        "remap_ikey": "o:replacement"
                    },
                    "*": {"action": "reject"}
                }
            },
            "env_resource_attrs": {"HOSTNAME": "host.name"}
        }))
        .expect("complete endpoint config");

        assert_eq!(config.body_format.0.len(), 2);
        assert_eq!(config.response_format, ResponseFormat::AzureMonitor);
        assert_eq!(config.ikey_filter.expect("iKey filter").keys.len(), 2);
    }

    /// Scenario: Event-name segment removal uses scalar and detailed configuration forms.
    /// Guarantees: Both forms expose their index and only the detailed form exposes a match field.
    #[test]
    fn strip_segment_exposes_complete_configuration() {
        let scalar: StripSegment = serde_json::from_value(json!(2)).expect("scalar form");
        let detailed: StripSegment = serde_json::from_value(json!({
            "index": 3,
            "match_field": "iKey"
        }))
        .expect("detailed form");

        assert_eq!(scalar.index(), 2);
        assert_eq!(scalar.match_field(), None);
        assert_eq!(detailed.index(), 3);
        assert_eq!(detailed.match_field(), Some("iKey"));
    }

    /// Scenario: Each additional-target transform is applied to a compatible JSON scalar.
    /// Guarantees: Split, format, SDK name, and SDK version transforms preserve their documented behavior.
    #[test]
    fn value_transforms_apply_complete_configuration() {
        let split = ValueTransform::SplitTake {
            delimiter: ".".to_owned(),
            index: -1,
        };
        let format = ValueTransform::Format {
            template: "legacy.{value}".to_owned(),
        };

        assert_eq!(split.apply(&json!("a.b")), Some(json!("b")));
        assert_eq!(format.apply(&json!(42)), Some(json!("legacy.42")));
        assert_eq!(
            ValueTransform::DashSemverName.apply(&json!("1DS-Web-JS-4.4.1")),
            Some(json!("1DS-Web-JS"))
        );
        assert_eq!(
            ValueTransform::DashSemverVersion.apply(&json!("1DS-Web-JS-4.4.1")),
            Some(json!("4.4.1"))
        );
    }
}
