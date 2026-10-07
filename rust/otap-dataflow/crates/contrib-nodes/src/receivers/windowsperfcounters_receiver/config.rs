// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration parsing, validation, and exact-path normalization.

use otel_arrow_dfe_config::error::Error;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

const MAX_COUNTER_PATH_LEN: usize = 2_047;
const MIN_SCALE_POWER10: i32 = -18;
const MAX_SCALE_POWER10: i32 = 18;
const MIN_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);
const MAX_COLLECTION_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_INITIAL_DELAY: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_METRIC_NAME_LEN: usize = 255;
const MAX_METRIC_UNIT_LEN: usize = 63;
const MAX_COUNTERS: usize = 256;
const RECEIVER_ATTRIBUTE_PREFIX: &str = "windows.perf_counter.";

/// OTel metric kind used to project a performance counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum MetricKind {
    /// A point-in-time Gauge.
    Gauge,
    /// A cumulative, non-monotonic Sum representing an UpDownCounter.
    UpDownCounter,
}

/// One normalized exact counter path and its metric mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CounterConfig {
    /// Exact configured PDH path.
    pub(super) path: String,
    /// OTel metric name.
    pub(super) name: String,
    /// OTel metric unit.
    pub(super) unit: String,
    /// OTel metric description, shared by every counter mapped to this metric.
    pub(super) description: Arc<str>,
    /// OTel metric kind.
    pub(super) kind: MetricKind,
    /// Static attributes added to every point, shared by every path expanded
    /// from one counter mapping.
    pub(super) attributes: Arc<BTreeMap<String, String>>,
    /// Base-10 scaling applied after PDH calculates the native value.
    pub(super) scale_power10: i32,
}

/// Configuration normalized for exact Windows performance-counter collection.
#[derive(Debug, Clone)]
pub(super) struct RuntimeConfig {
    /// Exact counters to collect.
    pub(super) counters: Vec<CounterConfig>,
    /// Time between collections; defaults to 30 seconds.
    pub(super) collection_interval: Duration,
    /// Delay before the first collection request; defaults to one second.
    pub(super) initial_delay: Duration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    metrics: BTreeMap<String, MetricConfig>,
    #[serde(rename = "perfcounters")]
    perf_counters: Vec<ObjectConfig>,
    #[serde(default = "default_interval", with = "humantime_serde")]
    collection_interval: Duration,
    #[serde(default = "default_initial_delay", with = "humantime_serde")]
    initial_delay: Duration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetricConfig {
    description: String,
    unit: String,
    #[serde(default, deserialize_with = "present_marker")]
    gauge: Option<EmptyConfig>,
    #[serde(default, deserialize_with = "present_marker")]
    up_down_counter: Option<EmptyConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyConfig {}

/// Treats a present key as selected even when its value is null (e.g. YAML `gauge:`).
fn present_marker<'de, D>(deserializer: D) -> Result<Option<EmptyConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(
        Option::<EmptyConfig>::deserialize(deserializer)?.unwrap_or_default(),
    ))
}

impl MetricConfig {
    fn kind(&self, name: &str) -> Result<MetricKind, Error> {
        match (self.gauge.is_some(), self.up_down_counter.is_some()) {
            (true, false) => Ok(MetricKind::Gauge),
            (false, true) => Ok(MetricKind::UpDownCounter),
            _ => Err(invalid(format!(
                "metrics.{name} must contain exactly one of gauge or up_down_counter"
            ))),
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectConfig {
    object: String,
    #[serde(default)]
    instances: Option<OneOrMany>,
    counters: Vec<CounterMapping>,
}

#[derive(Debug, Deserialize)]
#[serde(
    untagged,
    expecting = "instances must be a string or a list of strings"
)]
enum OneOrMany {
    One(String),
    Many(Vec<String>),
}

impl OneOrMany {
    fn as_slice(&self) -> &[String] {
        match self {
            Self::One(value) => std::slice::from_ref(value),
            Self::Many(values) => values,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CounterMapping {
    name: String,
    metric: String,
    #[serde(default)]
    attributes: BTreeMap<String, String>,
    #[serde(default)]
    scale_power10: i32,
}

fn default_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_initial_delay() -> Duration {
    Duration::from_secs(1)
}

fn invalid(error: impl Into<String>) -> Error {
    Error::InvalidUserConfig {
        error: error.into(),
    }
}

fn require_name(field: &str, value: &str) -> Result<(), Error> {
    if value.trim().is_empty() {
        return Err(invalid(format!("{field} must not be empty")));
    }
    Ok(())
}

fn validate_object_or_instance<'a>(field: &str, value: &'a str) -> Result<&'a str, Error> {
    let value = value.trim();
    require_name(field, value)?;
    if value.contains(['\\', '(', ')', '*', '?', '\0']) {
        return Err(invalid(format!(
            "{field} contains a reserved performance-counter path character"
        )));
    }
    Ok(value)
}

fn validate_counter_name<'a>(field: &str, value: &'a str) -> Result<&'a str, Error> {
    let value = value.trim();
    require_name(field, value)?;
    if value.contains(['\\', '*', '?', '\0']) {
        return Err(invalid(format!(
            "{field} contains a reserved performance-counter path character"
        )));
    }
    Ok(value)
}

/// Counts expanded counter paths before allocating them, so instance-by-counter
/// products are bounded before materialization.
fn expanded_counter_count(objects: &[ObjectConfig]) -> Result<usize, Error> {
    let too_many = || {
        invalid(format!(
            "perfcounters must expand to at most {MAX_COUNTERS} counter paths"
        ))
    };
    let mut total: usize = 0;
    for object in objects {
        let instances = object.instances.as_ref().map_or(1, |i| i.as_slice().len());
        let paths = instances
            .checked_mul(object.counters.len())
            .ok_or_else(too_many)?;
        total = total.checked_add(paths).ok_or_else(too_many)?;
        if total > MAX_COUNTERS {
            return Err(too_many());
        }
    }
    Ok(total)
}

/// Enforces the OpenTelemetry instrument-name syntax on the trimmed name.
fn validate_metric_name(name: &str) -> Result<&str, Error> {
    let name = name.trim();
    require_name("metric name", name)?;
    let mut chars = name.chars();
    let valid = name.len() <= MAX_METRIC_NAME_LEN
        && chars.next().is_some_and(|c| c.is_ascii_alphabetic())
        && chars.all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '.' | '-' | '/'));
    if !valid {
        return Err(invalid(format!(
            "metric name {name:?} must start with an ASCII letter, contain only ASCII \
             letters, digits, '_', '.', '-', or '/', and be at most {MAX_METRIC_NAME_LEN} characters"
        )));
    }
    Ok(name)
}

/// Enforces the OpenTelemetry instrument-unit syntax on the trimmed unit.
fn validate_metric_unit<'a>(name: &str, unit: &'a str) -> Result<&'a str, Error> {
    let unit = unit.trim();
    let field = format!("metrics.{name}.unit");
    require_name(&field, unit)?;
    if unit.len() > MAX_METRIC_UNIT_LEN || !unit.bytes().all(|b| b.is_ascii_graphic() || b == b' ')
    {
        return Err(invalid(format!(
            "{field} must be printable ASCII and at most {MAX_METRIC_UNIT_LEN} characters"
        )));
    }
    Ok(unit)
}

/// Trims attribute keys, rejecting blank, reserved, or post-trim duplicate keys.
fn normalize_attributes(
    field: &str,
    attributes: &BTreeMap<String, String>,
) -> Result<BTreeMap<String, String>, Error> {
    let mut normalized = BTreeMap::new();
    for (key, value) in attributes {
        let key = key.trim();
        require_name(&format!("{field} attribute key"), key)?;
        if key.starts_with(RECEIVER_ATTRIBUTE_PREFIX) {
            return Err(invalid(format!(
                "{field} attribute {key:?} conflicts with receiver-generated attributes"
            )));
        }
        if normalized.insert(key.to_owned(), value.clone()).is_some() {
            return Err(invalid(format!(
                "{field} contains duplicate attribute key {key:?}"
            )));
        }
    }
    Ok(normalized)
}

/// A metric definition after trimming and validation.
struct NormalizedMetric<'a> {
    description: Arc<str>,
    unit: &'a str,
    kind: MetricKind,
}

fn validate_counter(counter: &CounterConfig) -> Result<(), Error> {
    if counter.path.encode_utf16().count() > MAX_COUNTER_PATH_LEN {
        return Err(invalid(format!(
            "counter path {:?} must contain at most {MAX_COUNTER_PATH_LEN} UTF-16 code units",
            counter.path
        )));
    }
    if !(MIN_SCALE_POWER10..=MAX_SCALE_POWER10).contains(&counter.scale_power10) {
        return Err(invalid(format!(
            "counter path {:?} scale_power10 must be between {MIN_SCALE_POWER10} and {MAX_SCALE_POWER10}",
            counter.path
        )));
    }
    Ok(())
}

impl RuntimeConfig {
    /// Parse, validate, and normalize the public configuration contract.
    pub(super) fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        let user: Config =
            serde_json::from_value(value.clone()).map_err(|err| invalid(err.to_string()))?;
        // Every metric must be referenced, so the counter cap also bounds metric definitions.
        if user.metrics.len() > MAX_COUNTERS {
            return Err(invalid(format!(
                "metrics must contain at most {MAX_COUNTERS} entries"
            )));
        }
        if !(MIN_COLLECTION_INTERVAL..=MAX_COLLECTION_INTERVAL).contains(&user.collection_interval)
        {
            return Err(invalid(format!(
                "collection_interval must be between {} and {}",
                humantime::format_duration(MIN_COLLECTION_INTERVAL),
                humantime::format_duration(MAX_COLLECTION_INTERVAL)
            )));
        }
        if user.initial_delay > MAX_INITIAL_DELAY {
            return Err(invalid(format!(
                "initial_delay must be between {} and {}",
                humantime::format_duration(Duration::ZERO),
                humantime::format_duration(MAX_INITIAL_DELAY)
            )));
        }
        if user.metrics.is_empty() {
            return Err(invalid("metrics must contain at least one entry"));
        }
        if user.perf_counters.is_empty() {
            return Err(invalid("perfcounters must contain at least one entry"));
        }

        let mut metrics = BTreeMap::new();
        let mut metric_identities = HashSet::new();
        for (name, metric) in &user.metrics {
            let name = validate_metric_name(name)?;
            // OTel instrument names are case-insensitive.
            if !metric_identities.insert(name.to_ascii_lowercase()) {
                return Err(invalid(format!("metrics contains duplicate {name:?}")));
            }
            let description = metric.description.trim();
            require_name(&format!("metrics.{name}.description"), description)?;
            let unit = validate_metric_unit(name, &metric.unit)?;
            let kind = metric.kind(name)?;
            let _ = metrics.insert(
                name,
                NormalizedMetric {
                    description: Arc::from(description),
                    unit,
                    kind,
                },
            );
        }

        let expanded_count = expanded_counter_count(&user.perf_counters)?;
        let mut counters = Vec::with_capacity(expanded_count);
        let mut referenced_metrics = HashSet::new();
        for (object_index, object) in user.perf_counters.iter().enumerate() {
            let object_field = format!("perfcounters[{object_index}]");
            let object_name =
                validate_object_or_instance(&format!("{object_field}.object"), &object.object)?;
            if object.counters.is_empty() {
                return Err(invalid(format!(
                    "{object_field}.counters must contain at least one entry"
                )));
            }
            let instances = match &object.instances {
                None => vec![None],
                Some(instances) => {
                    let instances = instances.as_slice();
                    if instances.is_empty() {
                        return Err(invalid(format!(
                            "{object_field}.instances must not be empty"
                        )));
                    }
                    let mut unique = HashSet::new();
                    instances
                        .iter()
                        .map(|instance| {
                            let instance = validate_object_or_instance(
                                &format!("{object_field}.instances"),
                                instance,
                            )?;
                            if !unique.insert(instance.to_lowercase()) {
                                return Err(invalid(format!(
                                    "{object_field}.instances contains duplicate {instance:?}"
                                )));
                            }
                            Ok(Some(instance))
                        })
                        .collect::<Result<Vec<_>, Error>>()?
                }
            };
            for (counter_index, mapping) in object.counters.iter().enumerate() {
                let counter_field = format!("{object_field}.counters[{counter_index}]");
                let counter_name =
                    validate_counter_name(&format!("{counter_field}.name"), &mapping.name)?;
                let metric_name = mapping.metric.trim();
                let metric = metrics.get(metric_name).ok_or_else(|| {
                    invalid(format!(
                        "{counter_field}.metric references undefined metric {metric_name:?}"
                    ))
                })?;
                let _ = referenced_metrics.insert(metric_name);
                let attributes =
                    Arc::new(normalize_attributes(&counter_field, &mapping.attributes)?);
                for instance in &instances {
                    let path = match instance {
                        Some(instance) => {
                            format!(r"\{object_name}({instance})\{counter_name}")
                        }
                        None => format!(r"\{object_name}\{counter_name}"),
                    };
                    counters.push(CounterConfig {
                        path,
                        name: metric_name.to_owned(),
                        unit: metric.unit.to_owned(),
                        description: Arc::clone(&metric.description),
                        kind: metric.kind,
                        attributes: Arc::clone(&attributes),
                        scale_power10: mapping.scale_power10,
                    });
                }
            }
        }
        let mut paths = HashSet::with_capacity(counters.len());
        for counter in &counters {
            validate_counter(counter)?;
            if !paths.insert(counter.path.to_lowercase()) {
                return Err(invalid(format!("duplicate counter path: {}", counter.path)));
            }
        }
        let unused = metrics
            .keys()
            .filter(|name| !referenced_metrics.contains(*name))
            .copied()
            .collect::<Vec<_>>();
        if !unused.is_empty() {
            return Err(invalid(format!(
                "unreferenced metric definitions: {}",
                unused.join(", ")
            )));
        }
        Ok(Self {
            counters,
            collection_interval: user.collection_interval,
            initial_delay: user.initial_delay,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn metrics() -> serde_json::Value {
        json!({
            "available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            },
            "private": {
                "description": "Committed private memory.",
                "unit": "By",
                "up_down_counter": {}
            }
        })
    }

    fn config_with_timing(collection_interval: &str, initial_delay: &str) -> serde_json::Value {
        json!({
            "metrics": {
                "available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }
            },
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "available"}]
            }],
            "collection_interval": collection_interval,
            "initial_delay": initial_delay
        })
    }

    fn assert_config_error(value: serde_json::Value, expected: &str) {
        let error = RuntimeConfig::from_json(&value).unwrap_err().to_string();
        assert!(error.contains(expected), "unexpected error: {error}");
    }

    fn gauge_config(perfcounter: serde_json::Value) -> serde_json::Value {
        json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [perfcounter]
        })
    }

    /// Scenario: Exact object counters use no instance or explicitly named instances.
    /// Guarantees: Structured configuration normalizes paths and retains both metric kinds.
    #[test]
    fn normalizes_exact_configuration() {
        let config = RuntimeConfig::from_json(&json!({
            "metrics": metrics(),
            "perfcounters": [
                {
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "available"}]
                },
                {
                    "object": "Process",
                    "instances": ["app", "worker"],
                    "counters": [{"name": "Private Bytes", "metric": "private"}]
                }
            ]
        }))
        .unwrap();
        assert_eq!(config.counters.len(), 3);
        assert_eq!(config.counters[0].path, r"\Memory\Available Bytes");
        assert_eq!(config.counters[0].kind, MetricKind::Gauge);
        assert_eq!(config.counters[1].path, r"\Process(app)\Private Bytes");
        assert_eq!(config.counters[1].kind, MetricKind::UpDownCounter);
        assert_eq!(config.collection_interval, Duration::from_secs(30));
        assert_eq!(config.initial_delay, Duration::from_secs(1));
    }

    /// Scenario: An object specifies one exact instance as a scalar string.
    /// Guarantees: The shorthand normalizes to the same exact instance path as a one-item list.
    #[test]
    fn normalizes_single_instance_string() {
        let config = RuntimeConfig::from_json(&json!({
            "metrics": {"private": {
                "description": "Committed private memory.",
                "unit": "By",
                "up_down_counter": {}
            }},
            "perfcounters": [{
                "object": "Process",
                "instances": "app",
                "counters": [{"name": "Private Bytes", "metric": "private"}]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters[0].path, r"\Process(app)\Private Bytes");
    }

    /// Scenario: Collection timing is configured at, below, and above its supported boundaries.
    /// Guarantees: Exact boundary values succeed while out-of-range values identify their field.
    #[test]
    fn enforces_collection_timing_bounds() {
        let _config = RuntimeConfig::from_json(&config_with_timing("24h", "24h")).unwrap();

        assert_config_error(config_with_timing("500ms", "0s"), "collection_interval");
        assert_config_error(
            config_with_timing("86401s", "0s"),
            "collection_interval must be between",
        );
        assert_config_error(
            config_with_timing("1s", "86401s"),
            "initial_delay must be between",
        );
    }

    /// Scenario: A configured instance uses a wildcard in this exact-path receiver.
    /// Guarantees: Validation rejects wildcard expansion until that behavior is supported.
    #[test]
    fn rejects_wildcard_instances() {
        assert_config_error(
            json!({
                "metrics": metrics(),
                "perfcounters": [{
                    "object": "Process",
                    "instances": "*",
                    "counters": [{"name": "Private Bytes", "metric": "private"}]
                }]
            }),
            "contains a reserved performance-counter path character",
        );
    }

    /// Scenario: A metric definition selects both supported metric kinds.
    /// Guarantees: Validation rejects ambiguous projection semantics.
    #[test]
    fn rejects_ambiguous_metric_kind() {
        assert_config_error(
            json!({
                "metrics": {"value": {
                    "description": "Value.",
                    "unit": "1",
                    "gauge": {},
                    "up_down_counter": {}
                }},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "value"}]
                }]
            }),
            "metrics.value must contain exactly one of gauge or up_down_counter",
        );
    }

    /// Scenario: A metric definition is not referenced by any configured counter.
    /// Guarantees: Validation rejects metadata that cannot produce telemetry.
    #[test]
    fn rejects_unreferenced_metric_definition() {
        assert_config_error(
            json!({
                "metrics": metrics(),
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "available"}]
                }]
            }),
            "unreferenced metric definitions: private",
        );
    }

    /// Scenario: A counter mapping names a metric that is not defined.
    /// Guarantees: Validation identifies the public YAML field that contains the bad reference.
    #[test]
    fn rejects_undefined_metric_reference() {
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "missing"}]
                }]
            }),
            "perfcounters[0].counters[0].metric references undefined metric \"missing\"",
        );
    }

    /// Scenario: A configured data-point attribute uses the receiver-owned namespace.
    /// Guarantees: Validation preserves ownership of generated performance-counter attributes.
    #[test]
    fn rejects_reserved_attribute_prefix() {
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{
                        "name": "Available Bytes",
                        "metric": "available",
                        "attributes": {"windows.perf_counter.custom": "value"}
                    }]
                }]
            }),
            "perfcounters[0].counters[0] attribute \"windows.perf_counter.custom\" conflicts",
        );
    }

    /// Scenario: Decimal scaling exceeds the receiver's exact integer-power range.
    /// Guarantees: Validation rejects both lower and upper out-of-range values with the path.
    #[test]
    fn rejects_out_of_range_scaling() {
        for scale_power10 in [-19, 19] {
            assert_config_error(
                json!({
                    "metrics": {"available": {
                        "description": "Available physical memory.",
                        "unit": "By",
                        "gauge": {}
                    }},
                    "perfcounters": [{
                        "object": "Memory",
                        "counters": [{
                            "name": "Available Bytes",
                            "metric": "available",
                            "scale_power10": scale_power10
                        }]
                    }]
                }),
                "counter path \"\\\\Memory\\\\Available Bytes\" scale_power10",
            );
        }
    }

    /// Scenario: Required metric and performance-counter collections are empty.
    /// Guarantees: Validation rejects configurations that cannot emit any telemetry.
    #[test]
    fn rejects_empty_required_collections() {
        assert_config_error(
            json!({"metrics": {}, "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available Bytes", "metric": "available"}]
            }]}),
            "metrics must contain at least one entry",
        );
        assert_config_error(
            json!({"metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }}, "perfcounters": []}),
            "perfcounters must contain at least one entry",
        );
    }

    /// Scenario: Object and counter fields contain path delimiters or wildcard characters.
    /// Guarantees: Structural characters are rejected while parentheses remain valid in counter names.
    #[test]
    fn validates_reserved_path_characters_by_field_role() {
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [{
                    "object": "Memory(test)",
                    "counters": [{"name": "Available Bytes", "metric": "available"}]
                }]
            }),
            "perfcounters[0].object contains a reserved",
        );
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available *", "metric": "available"}]
                }]
            }),
            "perfcounters[0].counters[0].name contains a reserved",
        );
        let config = RuntimeConfig::from_json(&json!({
            "metrics": {"available": {
                "description": "Available physical memory.",
                "unit": "By",
                "gauge": {}
            }},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{"name": "Available (Bytes)", "metric": "available"}]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters[0].path, r"\Memory\Available (Bytes)");
    }

    /// Scenario: Configuration contains a field not defined by the receiver contract.
    /// Guarantees: Deserialization rejects misspelled or unsupported fields instead of ignoring them.
    #[test]
    fn rejects_unknown_fields() {
        let mut value = config_with_timing("30s", "1s");
        value["unknown"] = json!(true);
        assert_config_error(value, "unknown field `unknown`");
    }

    /// Scenario: More metric definitions are configured than counters can reference.
    /// Guarantees: Validation rejects the count up front instead of listing every unreferenced metric.
    #[test]
    fn rejects_too_many_metric_definitions() {
        let metrics = (0..=MAX_COUNTERS)
            .map(|index| {
                (
                    format!("m{index}"),
                    json!({"description": "Value.", "unit": "By", "gauge": {}}),
                )
            })
            .collect::<serde_json::Map<_, _>>();
        assert_config_error(
            json!({
                "metrics": metrics,
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "m0"}]
                }]
            }),
            "metrics must contain at most 256 entries",
        );
    }

    /// Scenario: Metric names violate the OpenTelemetry instrument-name syntax.
    /// Guarantees: Validation rejects them instead of emitting invalid metric names.
    #[test]
    fn rejects_invalid_metric_names() {
        let too_long = format!("a{}", "b".repeat(MAX_METRIC_NAME_LEN));
        for name in ["1available", "avail able", "é", too_long.as_str()] {
            assert_config_error(
                json!({
                    "metrics": {name: {"description": "Value.", "unit": "By", "gauge": {}}},
                    "perfcounters": [{
                        "object": "Memory",
                        "counters": [{"name": "Available Bytes", "metric": name}]
                    }]
                }),
                "must start with an ASCII letter",
            );
        }
        let at_limit = format!("a{}", "b".repeat(MAX_METRIC_NAME_LEN - 1));
        for name in ["system.memory_usage-1/s", at_limit.as_str()] {
            assert!(
                RuntimeConfig::from_json(&json!({
                    "metrics": {name: {"description": "Value.", "unit": "By", "gauge": {}}},
                    "perfcounters": [{
                        "object": "Memory",
                        "counters": [{"name": "Available Bytes", "metric": name}]
                    }]
                }))
                .is_ok()
            );
        }
    }

    /// Scenario: Metric units violate the OpenTelemetry instrument-unit syntax.
    /// Guarantees: Validation rejects non-ASCII, control-character, or overlong units.
    #[test]
    fn rejects_invalid_metric_units() {
        let unit_config = |unit: &str| {
            json!({
                "metrics": {"m": {"description": "Value.", "unit": unit, "gauge": {}}},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "m"}]
                }]
            })
        };
        let too_long = "a".repeat(MAX_METRIC_UNIT_LEN + 1);
        for unit in ["µs", "B\ty", too_long.as_str()] {
            assert_config_error(unit_config(unit), "metrics.m.unit must be printable ASCII");
        }
        let at_limit = "a".repeat(MAX_METRIC_UNIT_LEN);
        for unit in ["By", "{packets}/s", "1", at_limit.as_str()] {
            assert!(RuntimeConfig::from_json(&unit_config(unit)).is_ok());
        }
    }

    /// Scenario: Metric names, references, descriptions, units, and attribute keys are padded.
    /// Guarantees: Values are trimmed consistently and padded references still resolve.
    #[test]
    fn trims_metric_definitions_and_attribute_keys() {
        let config = RuntimeConfig::from_json(&json!({
            "metrics": {" available ": {
                "description": " Available memory. ",
                "unit": " By ",
                "gauge": {}
            }},
            "perfcounters": [{
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "available ",
                    "attributes": {" state ": " free "}
                }]
            }]
        }))
        .unwrap();
        let counter = &config.counters[0];
        assert_eq!(counter.name, "available");
        assert_eq!(&*counter.description, "Available memory.");
        assert_eq!(counter.unit, "By");
        assert_eq!(
            *counter.attributes,
            BTreeMap::from([("state".to_owned(), " free ".to_owned())])
        );
    }

    /// Scenario: One counter mapping expands across several instances.
    /// Guarantees: Expanded counters share metadata instead of cloning it per path.
    #[test]
    fn shares_metadata_across_expanded_counters() {
        let config = RuntimeConfig::from_json(&gauge_config(json!({
            "object": "Process",
            "instances": ["a", "b"],
            "counters": [{
                "name": "Private Bytes",
                "metric": "available",
                "attributes": {"state": "used"}
            }]
        })))
        .unwrap();
        let [first, second] = config.counters.as_slice() else {
            panic!("expected two expanded counters");
        };
        assert!(Arc::ptr_eq(&first.attributes, &second.attributes));
        assert!(Arc::ptr_eq(&first.description, &second.description));
    }

    /// Scenario: Metric names or attribute keys collide after trimming or case folding.
    /// Guarantees: Validation rejects them instead of silently merging definitions.
    #[test]
    fn rejects_duplicate_metric_names_and_attribute_keys() {
        let metric = json!({"description": "Value.", "unit": "By", "gauge": {}});
        for (first, second) in [("cpu", " cpu"), ("cpu", "CPU")] {
            assert_config_error(
                json!({
                    "metrics": {first: metric.clone(), second: metric.clone()},
                    "perfcounters": [{
                        "object": "Processor",
                        "counters": [
                            {"name": "% Processor Time", "metric": first},
                            {"name": "% User Time", "metric": second}
                        ]
                    }]
                }),
                "metrics contains duplicate",
            );
        }
        assert_config_error(
            gauge_config(json!({
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "available",
                    "attributes": {"state": "a", " state": "b"}
                }]
            })),
            "contains duplicate attribute key \"state\"",
        );
    }

    /// Scenario: Explicit instance names differ only by letter casing.
    /// Guarantees: Validation rejects duplicate instance paths before opening a PDH query.
    #[test]
    fn rejects_duplicate_instances_case_insensitively() {
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [{
                    "object": "Process",
                    "instances": ["app", "APP"],
                    "counters": [{"name": "Private Bytes", "metric": "available"}]
                }]
            }),
            "perfcounters[0].instances contains duplicate \"APP\"",
        );
    }

    /// Scenario: Object and counter names normalize to the same path with different casing.
    /// Guarantees: Validation rejects duplicate native counter handles.
    #[test]
    fn rejects_duplicate_counter_paths_case_insensitively() {
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [
                    {
                        "object": "Memory",
                        "counters": [{"name": "Available Bytes", "metric": "available"}]
                    },
                    {
                        "object": "memory",
                        "counters": [{"name": "available bytes", "metric": "available"}]
                    }
                ]
            }),
            "duplicate counter path: \\memory\\available bytes",
        );
    }

    /// Scenario: A metric kind is written as a bare YAML key (`gauge:`), which parses as null.
    /// Guarantees: A present-but-null kind selects that kind, and still conflicts with the other.
    #[test]
    fn accepts_null_metric_kind_shorthand() {
        let config = RuntimeConfig::from_json(&json!({
            "metrics": {
                "available": {"description": "Available.", "unit": "By", "gauge": null},
                "private": {"description": "Private.", "unit": "By", "up_down_counter": null}
            },
            "perfcounters": [{
                "object": "Memory",
                "counters": [
                    {"name": "Available Bytes", "metric": "available"},
                    {"name": "Committed Bytes", "metric": "private"}
                ]
            }]
        }))
        .unwrap();
        assert_eq!(config.counters[0].kind, MetricKind::Gauge);
        assert_eq!(config.counters[1].kind, MetricKind::UpDownCounter);

        assert_config_error(
            json!({
                "metrics": {"value": {
                    "description": "Value.",
                    "unit": "1",
                    "gauge": {},
                    "up_down_counter": null
                }},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "value"}]
                }]
            }),
            "metrics.value must contain exactly one of gauge or up_down_counter",
        );
    }

    /// Scenario: A metric definition selects no metric kind.
    /// Guarantees: Validation rejects a metric whose projection is unspecified.
    #[test]
    fn rejects_missing_metric_kind() {
        assert_config_error(
            json!({
                "metrics": {"value": {"description": "Value.", "unit": "1"}},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": "value"}]
                }]
            }),
            "metrics.value must contain exactly one of gauge or up_down_counter",
        );
    }

    /// Scenario: Counter paths are at and beyond the PDH path limit, including non-BMP text.
    /// Guarantees: The limit is enforced in UTF-16 code units, not bytes or chars.
    #[test]
    fn enforces_counter_path_length_in_utf16_units() {
        // `\Memory\` is 8 code units.
        let at_limit = "a".repeat(MAX_COUNTER_PATH_LEN - 8);
        let _config = RuntimeConfig::from_json(&gauge_config(json!({
            "object": "Memory",
            "counters": [{"name": at_limit, "metric": "available"}]
        })))
        .unwrap();

        let over_limit = "a".repeat(MAX_COUNTER_PATH_LEN - 7);
        assert_config_error(
            gauge_config(json!({
                "object": "Memory",
                "counters": [{"name": over_limit, "metric": "available"}]
            })),
            "must contain at most 2047 UTF-16 code units",
        );

        // `\Process(` + `)\X` is 12 code units; each U+1F600 is 2 code units, 1 char, 4 bytes.
        let at_limit_non_bmp = "\u{1F600}".repeat((MAX_COUNTER_PATH_LEN - 12) / 2);
        let _config = RuntimeConfig::from_json(&gauge_config(json!({
            "object": "Process",
            "instances": at_limit_non_bmp,
            "counters": [{"name": "X", "metric": "available"}]
        })))
        .unwrap();

        let non_bmp = "\u{1F600}".repeat((MAX_COUNTER_PATH_LEN - 12) / 2 + 1);
        assert!(12 + non_bmp.chars().count() <= MAX_COUNTER_PATH_LEN);
        assert_config_error(
            gauge_config(json!({
                "object": "Process",
                "instances": non_bmp,
                "counters": [{"name": "X", "metric": "available"}]
            })),
            "must contain at most 2047 UTF-16 code units",
        );
    }

    /// Scenario: Decimal scaling is configured at its lower and upper bounds.
    /// Guarantees: Inclusive boundary values are accepted and retained.
    #[test]
    fn accepts_scaling_bounds() {
        for scale_power10 in [MIN_SCALE_POWER10, MAX_SCALE_POWER10] {
            let config = RuntimeConfig::from_json(&gauge_config(json!({
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "available",
                    "scale_power10": scale_power10
                }]
            })))
            .unwrap();
            assert_eq!(config.counters[0].scale_power10, scale_power10);
        }
    }

    /// Scenario: An object's counter list or explicit instance list is empty.
    /// Guarantees: Validation rejects objects that would produce no counter paths.
    #[test]
    fn rejects_empty_object_collections() {
        assert_config_error(
            gauge_config(json!({"object": "Memory", "counters": []})),
            "perfcounters[0].counters must contain at least one entry",
        );
        assert_config_error(
            gauge_config(json!({
                "object": "Process",
                "instances": [],
                "counters": [{"name": "Private Bytes", "metric": "available"}]
            })),
            "perfcounters[0].instances must not be empty",
        );
    }

    /// Scenario: Instances are configured with a type other than a string or list of strings.
    /// Guarantees: The deserialization error names the field and its accepted shapes.
    #[test]
    fn rejects_invalid_instances_type() {
        assert_config_error(
            gauge_config(json!({
                "object": "Process",
                "instances": 5,
                "counters": [{"name": "Private Bytes", "metric": "available"}]
            })),
            "instances must be a string or a list of strings",
        );
    }

    /// Scenario: Required text values are empty or whitespace-only.
    /// Guarantees: Validation identifies each blank field instead of emitting unnamed telemetry.
    #[test]
    fn rejects_blank_required_values() {
        let counter = json!([{"name": "Available Bytes", "metric": "m"}]);
        for (metric, expected) in [
            (
                json!({"description": " ", "unit": "By", "gauge": {}}),
                "metrics.m.description must not be empty",
            ),
            (
                json!({"description": "Value.", "unit": "", "gauge": {}}),
                "metrics.m.unit must not be empty",
            ),
        ] {
            assert_config_error(
                json!({
                    "metrics": {"m": metric},
                    "perfcounters": [{"object": "Memory", "counters": counter}]
                }),
                expected,
            );
        }
        assert_config_error(
            json!({
                "metrics": {" ": {"description": "Value.", "unit": "By", "gauge": {}}},
                "perfcounters": [{
                    "object": "Memory",
                    "counters": [{"name": "Available Bytes", "metric": " "}]
                }]
            }),
            "metric name must not be empty",
        );
        assert_config_error(
            gauge_config(json!({
                "object": "Memory",
                "counters": [{
                    "name": "Available Bytes",
                    "metric": "available",
                    "attributes": {" ": "value"}
                }]
            })),
            "attribute key must not be empty",
        );
    }

    /// Scenario: Object and instance names contain each reserved path character.
    /// Guarantees: Every structural, wildcard, or NUL character is rejected in those segments.
    #[test]
    fn rejects_each_reserved_object_and_instance_character() {
        for reserved in ['\\', '(', ')', '*', '?', '\0'] {
            assert_config_error(
                gauge_config(json!({
                    "object": format!("Mem{reserved}ory"),
                    "counters": [{"name": "Available Bytes", "metric": "available"}]
                })),
                "perfcounters[0].object contains a reserved",
            );
            assert_config_error(
                gauge_config(json!({
                    "object": "Process",
                    "instances": format!("app{reserved}"),
                    "counters": [{"name": "Private Bytes", "metric": "available"}]
                })),
                "perfcounters[0].instances contains a reserved",
            );
        }
        for reserved in ['\\', '?', '\0'] {
            assert_config_error(
                gauge_config(json!({
                    "object": "Memory",
                    "counters": [{"name": format!("Available{reserved}"), "metric": "available"}]
                })),
                "perfcounters[0].counters[0].name contains a reserved",
            );
        }
    }

    /// Scenario: Instance and counter lists multiply into many expanded counter paths.
    /// Guarantees: The expanded total is bounded before any paths are materialized.
    #[test]
    fn bounds_expanded_counter_paths() {
        let object = |instances: usize, counters: usize| {
            json!({
                "object": "Process",
                "instances": (0..instances).map(|i| format!("i{i}")).collect::<Vec<_>>(),
                "counters": (0..counters)
                    .map(|c| json!({"name": format!("C{c}"), "metric": "available"}))
                    .collect::<Vec<_>>()
            })
        };

        let config = RuntimeConfig::from_json(&gauge_config(object(16, 16))).unwrap();
        assert_eq!(config.counters.len(), MAX_COUNTERS);

        assert_config_error(
            gauge_config(object(17, 16)),
            "perfcounters must expand to at most 256 counter paths",
        );
        assert_config_error(
            gauge_config(object(10_000, 10_000)),
            "perfcounters must expand to at most 256 counter paths",
        );
    }

    /// Scenario: Several object entries each stay under the cap but exceed it together.
    /// Guarantees: The expanded-path bound applies to the total across all entries.
    #[test]
    fn bounds_expanded_counter_paths_across_objects() {
        let objects = |count: usize| {
            json!({
                "metrics": {"available": {"description": "Value.", "unit": "By", "gauge": {}}},
                "perfcounters": (0..2)
                    .map(|o| json!({
                        "object": format!("Object{o}"),
                        "counters": (0..count)
                            .map(|c| json!({"name": format!("C{c}"), "metric": "available"}))
                            .collect::<Vec<_>>()
                    }))
                    .collect::<Vec<_>>()
            })
        };
        let config = RuntimeConfig::from_json(&objects(MAX_COUNTERS / 2)).unwrap();
        assert_eq!(config.counters.len(), MAX_COUNTERS);
        assert_config_error(
            objects(MAX_COUNTERS / 2 + 1),
            "perfcounters must expand to at most 256 counter paths",
        );
    }

    /// Scenario: Path segments have leading or trailing whitespace.
    /// Guarantees: Segments are trimmed before path construction and duplicate detection.
    #[test]
    fn trims_surrounding_whitespace_in_path_segments() {
        let config = RuntimeConfig::from_json(&gauge_config(json!({
            "object": " Process ",
            "instances": [" app "],
            "counters": [{"name": " Private Bytes ", "metric": "available"}]
        })))
        .unwrap();
        assert_eq!(config.counters[0].path, r"\Process(app)\Private Bytes");

        assert_config_error(
            gauge_config(json!({
                "object": "Process",
                "instances": ["app", "app "],
                "counters": [{"name": "Private Bytes", "metric": "available"}]
            })),
            "perfcounters[0].instances contains duplicate \"app\"",
        );
        assert_config_error(
            json!({
                "metrics": {"available": {
                    "description": "Available physical memory.",
                    "unit": "By",
                    "gauge": {}
                }},
                "perfcounters": [
                    {
                        "object": "Memory",
                        "counters": [{"name": "Available Bytes", "metric": "available"}]
                    },
                    {
                        "object": "Memory ",
                        "counters": [{"name": "Available Bytes", "metric": "available"}]
                    }
                ]
            }),
            "duplicate counter path: \\Memory\\Available Bytes",
        );
    }
}
