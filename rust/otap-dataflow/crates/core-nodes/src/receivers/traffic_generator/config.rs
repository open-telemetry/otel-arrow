// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the traffic generator receiver configuration

use serde::de::Deserializer;
use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::num::NonZeroU32;

use weaver_common::result::WResult;
use weaver_common::vdir::VirtualDirectoryPath;
use weaver_forge::registry::ResolvedRegistry;
use weaver_resolver::SchemaResolver;
use weaver_semconv::registry_repo::RegistryRepo;

/// Source of telemetry data schema and attributes
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DataSource {
    /// Use OpenTelemetry semantic conventions registry via weaver
    /// - Fetches and parses semantic conventions from registry_path
    /// - Full attribute coverage based on spec
    /// - Requires network/file access at startup
    #[default]
    SemanticConventions,

    /// Use minimal static hardcoded signals
    /// - No external dependencies or network access
    /// - Fixed set of attributes (e.g., service.name, http.method, etc.)
    Static,
}

/// Strategy for generating telemetry batches
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum GenerationStrategy {
    /// Generate fresh signals for every batch
    /// - New objects allocated per signal
    /// - Fresh timestamps and unique IDs per signal
    /// - Highest CPU/memory cost per batch
    #[default]
    Fresh,

    /// Pre-generate complete batches at startup, cycle through them unchanged
    /// - Zero per-batch allocation at runtime
    /// - Timestamps and IDs will repeat (stale)
    /// - Lowest CPU cost, maximum throughput
    PreGenerated,
    // Future: Templates variant — pre-generate signal templates, clone and update
    // timestamps/IDs per batch for moderate CPU cost with fresh data per batch.
    // Not yet implemented.
    // Templates,
}

/// Controls how signal batches are paced within each one-second production run.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProductionMode {
    /// Spread batches evenly across the second using a fixed-interval ticker.
    #[default]
    Smooth,
    /// Produce all batches as fast as possible without pacing.
    Open,
}

/// A single resource-attribute set with an optional batch weight.
///
/// The weight controls how many batches out of every `sum(weights)` batches will
/// use this attribute set during round-robin rotation.  Defaults to 1.
///
/// ```yaml
/// resource_attributes:
///   - attrs: {"tenant.id": "prod"}
///     weight: 3
///   - attrs: {"tenant.id": "ppe"}
///     weight: 1
/// ```
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ResourceAttributeSet {
    /// Key-value pairs merged into the resource of every generated signal.
    pub attrs: HashMap<String, String>,
    /// Relative batch weight (must be ≥ 1).  Defaults to 1.
    #[serde(default = "default_resource_weight")]
    pub weight: NonZeroU32,
}

/// Configuration should take a scenario to play out
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// Configuration of the traffic to generate
    traffic_config: TrafficConfig,

    /// Source of telemetry schema and attributes
    #[serde(default)]
    data_source: DataSource,

    /// Path to the semantic conventions registry (only used when data_source = semantic_conventions)
    #[serde(default = "default_registry_path")]
    registry_path: VirtualDirectoryPath,

    /// Strategy for generating telemetry batches
    #[serde(default)]
    generation_strategy: GenerationStrategy,

    /// When enabled, generated pdata subscribes to Ack/Nack notifications.
    ///
    /// This is useful for end-to-end Ack/Nack measurements across topic hops.
    #[serde(default = "default_enable_ack_nack")]
    enable_ack_nack: bool,

    /// Resource attribute sets to rotate across generated batches.
    /// Only applies to `data_source: static`. With `pre_generated`, only the
    /// first attribute set is used.
    ///
    /// Accepted forms (all backward-compatible):
    ///
    /// Single plain map (weight 1):
    /// ```yaml
    /// resource_attributes:
    ///   "tenant.id": "prod"
    /// ```
    ///
    /// List of plain maps (each weight 1, equal rotation):
    /// ```yaml
    /// resource_attributes:
    ///   - {"tenant.id": "prod"}
    ///   - {"tenant.id": "ppe"}
    /// ```
    ///
    /// List of weighted entries (3:1 split — prod gets 3 batches per ppe batch):
    /// ```yaml
    /// resource_attributes:
    ///   - attrs: {"tenant.id": "prod"}
    ///     weight: 3
    ///   - attrs: {"tenant.id": "ppe"}
    ///     weight: 1
    /// ```
    #[serde(default, deserialize_with = "deserialize_resource_attributes")]
    resource_attributes: Vec<ResourceAttributeSet>,

    /// Optional transport headers to attach to each generated pdata message.
    ///
    /// Keys are header names. Values are optional fixed strings; when left
    /// empty, a random value is generated once at startup.
    ///
    /// ```yaml
    /// transport_headers:
    ///   x-tenant-id: "acme"
    ///   x-request-id:
    /// ```
    #[serde(default)]
    transport_headers: HashMap<String, Option<String>>,
}

/// Configuration to describe the traffic being sent
#[derive(Clone, Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TrafficConfig {
    /// How signal batches are paced within each production run.
    #[serde(default)]
    pub production_mode: ProductionMode,
    /// Target number of signals to produce per second across all signal types.
    /// When `None`, defaults to `max_batch_size` (one full batch per second).
    #[serde(default = "default_signals_per_second")]
    pub signals_per_second: Option<usize>,
    /// Maximum total signals to produce before stopping. `None` means unlimited.
    #[serde(default = "default_max_signal")]
    pub max_signal_count: Option<u64>,
    /// Maximum number of signals in a single batch.
    #[serde(default = "default_max_batch_size")]
    pub max_batch_size: usize,
    /// Relative weight for metric signal production (0 disables metrics).
    #[serde(default = "default_weight")]
    pub metric_weight: u32,
    /// Relative weight for trace signal production (0 disables traces).
    #[serde(default = "default_weight")]
    pub trace_weight: u32,
    /// Relative weight for log signal production (0 disables logs).
    #[serde(default = "default_weight")]
    pub log_weight: u32,

    /// Target size of each log record body in bytes (Static data source only).
    /// When set, pre-generates a pool of 50 distinct body strings of this size;
    /// records cycle through the pool for realistic dictionary cardinality.
    /// When 0, the body is omitted entirely.
    /// When unset, cycles through ~50 default log message templates.
    #[serde(default)]
    pub log_body_size_bytes: Option<usize>,

    /// Number of attributes to attach to each log record (Static data source only).
    /// When set, generates this many key-value string attributes.
    /// When unset, uses the default 2 attributes (thread.id, thread.name).
    #[serde(default)]
    pub num_log_attributes: Option<usize>,

    /// When true, each log record gets a unique random trace_id and span_id,
    /// matching real log-to-trace correlation and adding per-record entropy.
    #[serde(default)]
    pub use_trace_context: bool,

    /// Number of attributes to attach to each metric data point (Static data source only).
    /// When set, generates this many key-value attributes per data point.
    /// When unset, uses the default 3 attributes (http.method, http.route, http.status_code).
    #[serde(default)]
    pub num_metric_attributes: Option<usize>,

    /// Number of data points per metric (Static data source only).
    /// When set, generates this many data points per metric.
    /// When unset, uses the default of 1 data point per metric.
    #[serde(default)]
    pub num_data_points_per_metric: Option<usize>,
}

impl Config {
    /// Create a new config given a name and a vector of scenario steps
    #[must_use]
    pub fn new(traffic_config: TrafficConfig, registry_path: VirtualDirectoryPath) -> Self {
        Self {
            traffic_config,
            registry_path,
            data_source: DataSource::default(),
            generation_strategy: GenerationStrategy::default(),
            enable_ack_nack: default_enable_ack_nack(),
            resource_attributes: Vec::new(),
            transport_headers: HashMap::new(),
        }
    }

    /// Builder-style method to set data source
    #[must_use]
    pub const fn with_data_source(mut self, data_source: DataSource) -> Self {
        self.data_source = data_source;
        self
    }

    /// Builder-style method to set generation strategy
    #[must_use]
    pub const fn with_generation_strategy(
        mut self,
        generation_strategy: GenerationStrategy,
    ) -> Self {
        self.generation_strategy = generation_strategy;
        self
    }

    /// Builder-style method to set transport headers.
    #[must_use]
    pub fn with_transport_headers(
        mut self,
        transport_headers: HashMap<String, Option<String>>,
    ) -> Self {
        self.transport_headers = transport_headers;
        self
    }

    /// Get the data source
    #[must_use]
    pub const fn data_source(&self) -> &DataSource {
        &self.data_source
    }

    /// Get the generation strategy
    #[must_use]
    pub const fn generation_strategy(&self) -> &GenerationStrategy {
        &self.generation_strategy
    }

    /// Provide a reference to the traffic config
    #[must_use]
    pub const fn get_traffic_config(&self) -> &TrafficConfig {
        &self.traffic_config
    }

    /// Returns whether generated pdata should subscribe to Ack/Nack.
    #[must_use]
    pub const fn enable_ack_nack(&self) -> bool {
        self.enable_ack_nack
    }

    /// Provide a reference to the ResolvedRegistry.
    /// Returns None if data_source is Static.
    pub fn get_registry(&self) -> Result<Option<ResolvedRegistry>, String> {
        match self.data_source {
            DataSource::Static => Ok(None),
            DataSource::SemanticConventions => {
                let registry_repo = RegistryRepo::try_new("main", &self.registry_path)
                    .map_err(|err| err.to_string())?;

                // Load the semantic convention registry.
                let registry = match SchemaResolver::load_semconv_repository(registry_repo, false) {
                    WResult::Ok(registry) => registry,
                    WResult::OkWithNFEs(registry, _) => registry,
                    WResult::FatalErr(err) => return Err(err.to_string()),
                };

                let resolved_schema = match SchemaResolver::resolve(registry, true) {
                    WResult::Ok(resolved_schema) => resolved_schema,
                    WResult::OkWithNFEs(resolved_schema, _) => resolved_schema,
                    WResult::FatalErr(err) => return Err(err.to_string()),
                };

                let resolved_registry = ResolvedRegistry::try_from_resolved_registry(
                    &resolved_schema.registry,
                    resolved_schema.catalog(),
                )
                .map_err(|err| err.to_string())?;

                Ok(Some(resolved_registry))
            }
        }
    }

    /// Get the typed resource attribute sets.
    #[must_use]
    pub fn resource_attributes(&self) -> &[ResourceAttributeSet] {
        &self.resource_attributes
    }

    /// Get the transport headers configuration.
    ///
    /// Keys are header names. Entries with a value produce fixed headers;
    /// entries left empty produce a random value generated once at startup.
    #[must_use]
    pub fn transport_headers(&self) -> &HashMap<String, Option<String>> {
        &self.transport_headers
    }
}

impl TrafficConfig {
    /// create a new traffic config which describes the output traffic of the receiver
    #[must_use]
    pub const fn new(
        signals_per_second: Option<usize>,
        max_signal_count: Option<u64>,
        max_batch_size: usize,
        metric_weight: u32,
        trace_weight: u32,
        log_weight: u32,
    ) -> Self {
        Self {
            production_mode: ProductionMode::Smooth,
            signals_per_second,
            max_signal_count,
            max_batch_size,
            metric_weight,
            trace_weight,
            log_weight,
            log_body_size_bytes: None,
            num_log_attributes: None,
            use_trace_context: false,
            num_metric_attributes: None,
            num_data_points_per_metric: None,
        }
    }

    /// return the specified message rate
    #[must_use]
    pub const fn get_signal_rate(&self) -> Option<usize> {
        self.signals_per_second
    }

    /// returns the max amounts of signals that should be sent
    #[must_use]
    pub const fn get_max_signal_count(&self) -> Option<u64> {
        self.max_signal_count
    }

    /// returns the max batch size per message
    #[must_use]
    pub const fn get_max_batch_size(&self) -> usize {
        self.max_batch_size
    }

    /// Returns the configured log body size in bytes, if set.
    #[must_use]
    pub const fn log_body_size_bytes(&self) -> Option<usize> {
        self.log_body_size_bytes
    }

    /// Returns the configured number of log attributes, if set.
    #[must_use]
    pub const fn num_log_attributes(&self) -> Option<usize> {
        self.num_log_attributes
    }

    /// Returns whether log records should include trace_id and span_id.
    #[must_use]
    pub const fn use_trace_context(&self) -> bool {
        self.use_trace_context
    }

    /// Returns the configured number of metric attributes, if set.
    #[must_use]
    pub const fn num_metric_attributes(&self) -> Option<usize> {
        self.num_metric_attributes
    }

    /// Returns the configured number of data points per metric, if set.
    #[must_use]
    pub const fn num_data_points_per_metric(&self) -> Option<usize> {
        self.num_data_points_per_metric
    }

    /// Validate semantic invariants that serde cannot express.
    ///
    /// Currently checks that at least one signal weight is non-zero so the
    /// producer has something to generate.
    pub fn validate(&self) -> Result<(), otap_df_config::error::Error> {
        if self.metric_weight + self.trace_weight + self.log_weight == 0 {
            return Err(otap_df_config::error::Error::InvalidUserConfig {
                error: "at least one of metric_weight, trace_weight, or log_weight must be > 0"
                    .to_string(),
            });
        }
        Ok(())
    }
}

const fn default_signals_per_second() -> Option<usize> {
    Some(30)
}

const fn default_max_signal() -> Option<u64> {
    None
}

const fn default_max_batch_size() -> usize {
    1000
}

const fn default_weight() -> u32 {
    0
}

fn default_registry_path() -> VirtualDirectoryPath {
    VirtualDirectoryPath::GitRepo {
        url: "https://github.com/open-telemetry/semantic-conventions.git".to_owned(),
        sub_folder: Some("model".to_owned()),
        refspec: None,
    }
}

const fn default_enable_ack_nack() -> bool {
    false
}

fn default_resource_weight() -> NonZeroU32 {
    NonZeroU32::new(1).expect("1 is non-zero")
}

/// Precompute the rotation index table from a slice of weighted attribute sets.
///
/// Each entry at position `i` contributes `entry.weight` copies of `i` to the
/// table.  The hot path is then:
/// ```text
/// slot = rotation[batch_index % rotation.len()]
/// attrs = &entries[slot].attrs
/// ```
/// An empty table means no custom attributes are configured.
///
/// Two entries with weights 3 and 1 produce `[0, 0, 0, 1]`.
///
/// Future improvement: replace with smooth weighted round-robin to avoid bursty traffic shape.
#[must_use]
pub(crate) fn build_rotation_table(entries: &[ResourceAttributeSet]) -> Vec<usize> {
    entries
        .iter()
        .enumerate()
        .flat_map(|(i, e)| std::iter::repeat_n(i, e.weight.get() as usize))
        .collect()
}

/// Accepts a plain map, a list of plain maps, a list of weighted structs, or a
/// mixed list — all normalized to `Vec<ResourceAttributeSet>`.
fn deserialize_resource_attributes<'de, D>(
    deserializer: D,
) -> Result<Vec<ResourceAttributeSet>, D::Error>
where
    D: Deserializer<'de>,
{
    /// A single entry: either a weighted struct or a plain map.
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum RawEntry {
        Weighted(ResourceAttributeSet),
        Plain(HashMap<String, String>),
    }

    #[derive(Deserialize)]
    #[serde(untagged)]
    enum OneOrMany {
        One(RawEntry),
        Many(Vec<RawEntry>),
    }

    let raw = match OneOrMany::deserialize(deserializer)? {
        OneOrMany::One(e) => vec![e],
        OneOrMany::Many(list) => list,
    };

    raw.into_iter()
        .filter_map(|e| match e {
            RawEntry::Weighted(s) => {
                if s.attrs.is_empty() {
                    // Explicit weighted entry with no attrs is a misconfiguration:
                    // the user wrote `attrs: {}` (or omitted it entirely) alongside
                    // a `weight`, which has no useful effect and is almost certainly
                    // a typo.  Reject it rather than silently ignoring it.
                    Some(Err(serde::de::Error::custom(
                        "resource_attributes entry has `attrs` that is empty; \
                         either provide at least one attribute or remove the entry",
                    )))
                } else {
                    Some(Ok(s))
                }
            }
            RawEntry::Plain(map) => {
                if map.is_empty() {
                    // A bare empty map `{}` in a list is harmless noise; skip it.
                    None
                } else {
                    Some(Ok(ResourceAttributeSet {
                        attrs: map,
                        weight: default_resource_weight(),
                    }))
                }
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::{Config, DataSource, GenerationStrategy, build_rotation_table};
    use serde_json::json;

    #[test]
    fn parse_config_defaults_enable_ack_nack_to_false() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": {
                "signals_per_second": 10,
                "max_signal_count": null,
                "max_batch_size": 1000,
                "metric_weight": 0,
                "trace_weight": 0,
                "log_weight": 1
            },
            "data_source": "static",
            "generation_strategy": "pre_generated"
        }))
        .expect("config should parse");

        assert!(!cfg.enable_ack_nack());
        assert_eq!(cfg.data_source(), &DataSource::Static);
        assert_eq!(cfg.generation_strategy(), &GenerationStrategy::PreGenerated);
    }

    #[test]
    fn parse_config_accepts_enable_ack_nack_true() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": {
                "signals_per_second": 10,
                "max_signal_count": null,
                "max_batch_size": 1000,
                "metric_weight": 0,
                "trace_weight": 0,
                "log_weight": 1
            },
            "data_source": "static",
            "generation_strategy": "fresh",
            "enable_ack_nack": true
        }))
        .expect("config should parse");

        assert!(cfg.enable_ack_nack());
    }

    fn base_traffic() -> serde_json::Value {
        json!({
            "signals_per_second": 10,
            "max_signal_count": null,
            "max_batch_size": 100,
            "metric_weight": 0,
            "trace_weight": 0,
            "log_weight": 1
        })
    }

    #[test]
    fn resource_attributes_absent_yields_empty() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": base_traffic(),
            "data_source": "static"
        }))
        .expect("config should parse");
        assert!(cfg.resource_attributes().is_empty());
        assert!(build_rotation_table(cfg.resource_attributes()).is_empty());
    }

    #[test]
    fn resource_attributes_plain_single_map() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "resource_attributes": {"tenant.id": "prod"}
        }))
        .expect("config should parse");
        let attrs = cfg.resource_attributes();
        assert_eq!(attrs.len(), 1);
        assert_eq!(
            attrs[0].attrs.get("tenant.id").map(String::as_str),
            Some("prod")
        );
        assert_eq!(attrs[0].weight.get(), 1);
        assert_eq!(build_rotation_table(attrs), vec![0]);
    }

    #[test]
    fn resource_attributes_list_of_plain_maps() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "resource_attributes": [
                {"tenant.id": "prod"},
                {"tenant.id": "ppe"}
            ]
        }))
        .expect("config should parse");
        let attrs = cfg.resource_attributes();
        assert_eq!(attrs.len(), 2);
        assert_eq!(build_rotation_table(attrs), vec![0, 1]);
    }

    #[test]
    fn resource_attributes_weighted_entries() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "resource_attributes": [
                {"attrs": {"tenant.id": "prod"}, "weight": 3},
                {"attrs": {"tenant.id": "ppe"},  "weight": 1}
            ]
        }))
        .expect("config should parse");
        let attrs = cfg.resource_attributes();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].weight.get(), 3);
        assert_eq!(attrs[1].weight.get(), 1);
        // rotation table: prod 3 times, then ppe once
        assert_eq!(build_rotation_table(attrs), vec![0, 0, 0, 1]);
    }

    #[test]
    fn resource_attributes_weighted_default_weight() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "resource_attributes": [
                {"attrs": {"tenant.id": "prod"}}
            ]
        }))
        .expect("config should parse");
        assert_eq!(cfg.resource_attributes()[0].weight.get(), 1);
    }

    #[test]
    fn resource_attributes_weight_zero_is_rejected() {
        let result = serde_json::from_value::<Config>(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "resource_attributes": [
                {"attrs": {"tenant.id": "prod"}, "weight": 0}
            ]
        }));
        assert!(result.is_err(), "weight=0 should be rejected");
    }

    #[test]
    fn resource_attributes_unknown_field_is_rejected() {
        // "weights" is a common typo for "weight" — must not silently fall back
        // to weight=1 with the stray field ignored.
        let result = serde_json::from_value::<Config>(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "resource_attributes": [
                {"attrs": {"tenant.id": "prod"}, "weights": 3}
            ]
        }));
        assert!(
            result.is_err(),
            "unknown field 'weights' should be rejected"
        );
    }

    #[test]
    fn resource_attributes_weighted_empty_attrs_is_rejected() {
        let result = serde_json::from_value::<Config>(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "resource_attributes": [
                {"attrs": {}, "weight": 2}
            ]
        }));
        assert!(
            result.is_err(),
            "weighted entry with empty attrs should be rejected"
        );
    }

    #[test]
    fn build_rotation_table_correct_order() {
        use super::ResourceAttributeSet;
        use std::collections::HashMap;
        use std::num::NonZeroU32;

        let entries = vec![
            ResourceAttributeSet {
                attrs: HashMap::new(),
                weight: NonZeroU32::new(2).unwrap(),
            },
            ResourceAttributeSet {
                attrs: HashMap::new(),
                weight: NonZeroU32::new(3).unwrap(),
            },
        ];
        assert_eq!(build_rotation_table(&entries), vec![0, 0, 1, 1, 1]);
    }

    // -- transport_headers config tests ----------------------------------------

    #[test]
    fn parse_config_transport_headers_default_empty() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "generation_strategy": "fresh"
        }))
        .expect("config should parse");

        assert!(
            cfg.transport_headers().is_empty(),
            "absent transport_headers should default to empty"
        );
    }

    #[test]
    fn parse_config_transport_headers_with_values() {
        let cfg: Config = serde_json::from_value(json!({
            "traffic_config": base_traffic(),
            "data_source": "static",
            "generation_strategy": "fresh",
            "transport_headers": {
                "x-tenant-id": "acme",
                "x-request-id": null
            }
        }))
        .expect("config should parse");

        let headers = cfg.transport_headers();
        assert_eq!(headers.len(), 2);
        assert_eq!(
            headers.get("x-tenant-id"),
            Some(&Some("acme".to_string())),
            "fixed value should be preserved"
        );
        assert_eq!(
            headers.get("x-request-id"),
            Some(&None),
            "null value should parse as None"
        );
    }

    #[test]
    fn validate_rejects_all_zero_weights() {
        let cfg = super::TrafficConfig::new(Some(10), None, 100, 0, 0, 0);
        let result = cfg.validate();
        assert!(
            result.is_err(),
            "all-zero signal weights should be rejected"
        );
    }

    #[test]
    fn validate_accepts_at_least_one_nonzero_weight() {
        let cfg = super::TrafficConfig::new(Some(10), None, 100, 0, 0, 1);
        cfg.validate().expect("one non-zero weight should pass");

        let cfg = super::TrafficConfig::new(Some(10), None, 100, 1, 0, 0);
        cfg.validate().expect("one non-zero weight should pass");

        let cfg = super::TrafficConfig::new(Some(10), None, 100, 0, 1, 0);
        cfg.validate().expect("one non-zero weight should pass");

        let cfg = super::TrafficConfig::new(Some(10), None, 100, 1, 1, 1);
        cfg.validate().expect("all non-zero weights should pass");
    }
}
