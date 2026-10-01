// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Public configuration parsing, validation, and exact-path normalization.

use otel_arrow_dfe_config::error::Error;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::time::Duration;

const MAX_COUNTER_PATH_LEN: usize = 2_047;
const MIN_SCALE_POWER10: i32 = -18;
const MAX_SCALE_POWER10: i32 = 18;
const MIN_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);
const MAX_COLLECTION_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_INITIAL_DELAY: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_METRICS: usize = u16::MAX as usize + 1;
const MAX_COUNTERS: usize = 256;
const RECEIVER_ATTRIBUTE_PREFIX: &str = "windows.perf_counter.";

/// OTel metric kind used to project a performance counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MetricKind {
    /// A point-in-time Gauge.
    Gauge,
    /// A cumulative, non-monotonic Sum representing an UpDownCounter.
    UpDownCounter,
}

/// One normalized exact counter path and its metric mapping.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CounterConfig {
    /// Exact configured PDH path.
    pub path: String,
    /// OTel metric name.
    pub name: String,
    /// OTel metric unit.
    pub unit: String,
    /// OTel metric description.
    pub description: String,
    /// OTel metric kind.
    pub kind: MetricKind,
    /// Static attributes added to every point from this counter.
    pub attributes: BTreeMap<String, String>,
    /// Base-10 scaling applied after PDH calculates the native value.
    pub scale_power10: i32,
}

/// Configuration normalized for exact Windows performance-counter collection.
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    /// Exact counters to collect.
    pub counters: Vec<CounterConfig>,
    /// Time between collections; defaults to 30 seconds.
    pub collection_interval: Duration,
    /// Delay before the first collection request; defaults to one second.
    pub initial_delay: Duration,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    metrics: BTreeMap<String, MetricConfig>,
    perfcounters: Vec<ObjectConfig>,
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
    fn len(&self) -> usize {
        match self {
            Self::One(_) => 1,
            Self::Many(values) => values.len(),
        }
    }

    fn into_vec(self) -> Vec<String> {
        match self {
            Self::One(value) => vec![value],
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
    if value.contains(['\\', '(', ')', '*', '?']) {
        return Err(invalid(format!(
            "{field} contains a reserved performance-counter path character"
        )));
    }
    Ok(value)
}

fn validate_counter_name<'a>(field: &str, value: &'a str) -> Result<&'a str, Error> {
    let value = value.trim();
    require_name(field, value)?;
    if value.contains(['\\', '*', '?']) {
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
        let instances = object.instances.as_ref().map_or(1, OneOrMany::len);
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

fn validate_metric_count(count: usize) -> Result<(), Error> {
    if count > MAX_METRICS {
        return Err(invalid(format!(
            "metrics must contain at most {MAX_METRICS} entries"
        )));
    }
    Ok(())
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
    for key in counter.attributes.keys() {
        require_name(
            &format!("counter path {:?} attribute key", counter.path),
            key,
        )?;
        if key.starts_with(RECEIVER_ATTRIBUTE_PREFIX) {
            return Err(invalid(format!(
                "counter path {:?} attribute {key:?} conflicts with receiver-generated attributes",
                counter.path
            )));
        }
    }
    Ok(())
}

impl RuntimeConfig {
    /// Parse, validate, and normalize the public configuration contract.
    pub fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        let user: Config =
            serde_json::from_value(value.clone()).map_err(|err| invalid(err.to_string()))?;
        validate_metric_count(user.metrics.len())?;
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
        if user.perfcounters.is_empty() {
            return Err(invalid("perfcounters must contain at least one entry"));
        }

        for (name, metric) in &user.metrics {
            require_name("metric name", name)?;
            require_name(&format!("metrics.{name}.description"), &metric.description)?;
            require_name(&format!("metrics.{name}.unit"), &metric.unit)?;
            let _ = metric.kind(name)?;
        }

        let expanded_count = expanded_counter_count(&user.perfcounters)?;
        let mut counters = Vec::with_capacity(expanded_count);
        let mut referenced_metrics = HashSet::new();
        for (object_index, object) in user.perfcounters.into_iter().enumerate() {
            let object_field = format!("perfcounters[{object_index}]");
            let object_name =
                validate_object_or_instance(&format!("{object_field}.object"), &object.object)?;
            if object.counters.is_empty() {
                return Err(invalid(format!(
                    "{object_field}.counters must contain at least one entry"
                )));
            }
            let instances = match object.instances {
                None => vec![None],
                Some(instances) => {
                    let instances = instances.into_vec();
                    if instances.is_empty() {
                        return Err(invalid(format!(
                            "{object_field}.instances must not be empty"
                        )));
                    }
                    let mut unique = HashSet::new();
                    instances
                        .into_iter()
                        .map(|instance| {
                            let instance = validate_object_or_instance(
                                &format!("{object_field}.instances"),
                                &instance,
                            )?;
                            if !unique.insert(instance.to_lowercase()) {
                                return Err(invalid(format!(
                                    "{object_field}.instances contains duplicate {instance:?}"
                                )));
                            }
                            Ok(Some(instance.to_owned()))
                        })
                        .collect::<Result<Vec<_>, Error>>()?
                }
            };
            for (counter_index, mapping) in object.counters.into_iter().enumerate() {
                let counter_field = format!("{object_field}.counters[{counter_index}]");
                let counter_name =
                    validate_counter_name(&format!("{counter_field}.name"), &mapping.name)?;
                let metric = user.metrics.get(&mapping.metric).ok_or_else(|| {
                    invalid(format!(
                        "{counter_field}.metric references undefined metric {:?}",
                        mapping.metric
                    ))
                })?;
                let _ = referenced_metrics.insert(mapping.metric.clone());
                for instance in &instances {
                    let path = match instance {
                        Some(instance) => {
                            format!(r"\{object_name}({instance})\{counter_name}")
                        }
                        None => format!(r"\{object_name}\{counter_name}"),
                    };
                    counters.push(CounterConfig {
                        path,
                        name: mapping.metric.clone(),
                        unit: metric.unit.clone(),
                        description: metric.description.clone(),
                        kind: metric.kind(&mapping.metric)?,
                        attributes: mapping.attributes.clone(),
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
        let unused = user
            .metrics
            .keys()
            .filter(|name| !referenced_metrics.contains(*name))
            .cloned()
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
            "counter path \"\\\\Memory\\\\Available Bytes\" attribute",
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

    /// Scenario: The number of metric definitions exceeds the OTAP metric identifier domain.
    /// Guarantees: Validation rejects the count before a receiver can fail on every scrape.
    #[test]
    fn rejects_too_many_metric_definitions() {
        assert!(validate_metric_count(MAX_METRICS).is_ok());
        assert!(validate_metric_count(MAX_METRICS + 1).is_err());
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
    /// Guarantees: Every structural or wildcard character is rejected in those segments.
    #[test]
    fn rejects_each_reserved_object_and_instance_character() {
        for reserved in ['\\', '(', ')', '*', '?'] {
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
        for reserved in ['\\', '?'] {
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
