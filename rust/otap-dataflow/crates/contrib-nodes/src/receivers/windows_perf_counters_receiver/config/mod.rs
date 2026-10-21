// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration parsing, validation, and exact-path normalization.

use otel_arrow_dfe_config::error::Error;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

pub(super) const MIN_SCALE_POWER10: i32 = -18;
pub(super) const MAX_SCALE_POWER10: i32 = 18;

const MAX_COUNTER_PATH_LEN: usize = 2_047;
const MIN_COLLECTION_INTERVAL: Duration = Duration::from_secs(1);
const MAX_COLLECTION_INTERVAL: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_INITIAL_DELAY: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_COUNTERS: usize = 256;
const RECEIVER_ATTRIBUTE_PREFIX: &str = "windows.perf_counter.";

const OBJECT_INSTANCE_RESERVED: &[char] = &['\\', '(', ')', '*', '?', '\0'];
const COUNTER_RESERVED: &[char] = &['\\', '*', '?', '\0'];

/// OTel metric kind used to project a performance counter.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub(super) enum MetricKind {
    /// A point-in-time Gauge.
    Gauge,
    /// A cumulative, non-monotonic Sum.
    Sum,
}

/// OTel Sum aggregation temporality.
#[derive(Debug, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
enum Aggregation {
    Cumulative,
    Delta,
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

impl RuntimeConfig {
    /// Parse, validate, and normalize the public configuration contract.
    pub(super) fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        let user: Config =
            serde_json::from_value(value.clone()).map_err(|err| invalid(err.to_string()))?;
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
        if user.perf_counters.is_empty() {
            return Err(invalid("perfcounters must contain at least one entry"));
        }

        let mut metrics = BTreeMap::new();
        for (name, metric) in &user.metrics {
            validate_metric_name(name)?;
            let unit = normalize_metric_unit(&metric.unit);
            let kind = metric.kind(name)?;
            let _ = metrics.insert(
                name.as_str(),
                NormalizedMetric {
                    description: Arc::from(metric.description.as_str()),
                    unit,
                    kind,
                },
            );
        }
        for group in case_distinct_metric_definitions(metrics.keys().copied()) {
            otel_arrow_dfe_telemetry::otel_warn!(
                "windows_perf_counters_receiver.config.metric_definition.case_collision",
                message = "metric definitions differ only by case; exact lookup \
                           keeps them distinct, but this may be a configuration typo",
                names = ?group
            );
        }

        let default_metric = NormalizedMetric {
            description: Arc::from(""),
            unit: "1",
            kind: MetricKind::Gauge,
        };
        let expanded_count = expanded_counter_count(&user.perf_counters)?;
        let mut counters = Vec::with_capacity(expanded_count);
        for (object_index, object) in user.perf_counters.iter().enumerate() {
            let object_field = format!("perfcounters[{object_index}]");
            let object_name = validate_object(&format!("{object_field}.object"), &object.object)?;
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
                            let (normalized, instance) =
                                normalize_instance(&format!("{object_field}.instances"), instance)?;
                            if !unique.insert(normalized.to_lowercase()) {
                                return Err(invalid(format!(
                                    "{object_field}.instances contains duplicate {instance:?}"
                                )));
                            }
                            Ok(Some((normalized, instance)))
                        })
                        .collect::<Result<Vec<_>, Error>>()?
                }
            };
            for (counter_index, mapping) in object.counters.iter().enumerate() {
                let counter_field = format!("{object_field}.counters[{counter_index}]");
                let counter_name =
                    validate_counter_name(&format!("{counter_field}.name"), &mapping.name)?;
                let metric_name = match &mapping.metric {
                    None => None,
                    Some(Some(name)) => Some(name.as_str()).filter(|name| !name.is_empty()),
                    Some(None) => {
                        return Err(invalid(format!(
                            "{counter_field}.metric must be a string when present; null metric is invalid"
                        )));
                    }
                };
                if let Some(metric_name) = metric_name
                    && !metrics.contains_key(metric_name)
                {
                    return Err(invalid(format!(
                        "{counter_field}.metric references undefined metric {metric_name:?}"
                    )));
                }
                let attributes = Arc::new(if metric_name.is_some() {
                    validate_attribute_keys(&counter_field, &mapping.attributes)?;
                    mapping.attributes.clone()
                } else {
                    BTreeMap::new()
                });
                for instance in &instances {
                    let path = counter_path(
                        object_name,
                        instance.as_ref().map(|(normalized, _)| normalized.as_str()),
                        counter_name,
                    );
                    let name = match metric_name {
                        Some(name) => name.to_owned(),
                        None => {
                            // Keep configured spelling for the emitted metric name and definition lookup.
                            let name = counter_path(
                                object_name,
                                instance.as_ref().map(|(_, configured)| *configured),
                                counter_name,
                            );
                            validate_path_length("counter path", &name)?;
                            name
                        }
                    };
                    let metric = metrics.get(name.as_str()).unwrap_or(&default_metric);
                    counters.push(CounterConfig {
                        path,
                        name,
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
        for name in unused_path_metric_definitions(metrics.keys().copied(), &counters) {
            otel_arrow_dfe_telemetry::otel_warn!(
                "windows_perf_counters_receiver.config.metric_definition.unused_path",
                message = "path-shaped metric definition is not selected by an explicit reference \
                           or exact configured path lookup; it emits no metric",
                definition = name
            );
        }
        Ok(Self {
            counters,
            collection_interval: user.collection_interval,
            initial_delay: user.initial_delay,
        })
    }
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Config {
    /// Metric definitions by name. Defaults to empty, including when null.
    /// Names are kept as written and must not be empty. Case-only variants
    /// stay distinct and warn once per lowercase group. Path-shaped keys not
    /// selected by a configured counter warn once; unused definitions emit nothing.
    #[serde(default, deserialize_with = "deserialize_metric_definitions")]
    metrics: BTreeMap<String, MetricConfig>,
    /// Objects to collect. At least one; must expand to at most 256 counter paths,
    /// one per mapping and instance.
    #[serde(rename = "perfcounters")]
    perf_counters: Vec<ObjectConfig>,
    /// Time between collections. Defaults to 30 seconds; 1 second to 24 hours.
    #[serde(default = "default_interval", with = "humantime_serde")]
    collection_interval: Duration,
    /// Delay before the first collection. Defaults to 1 second; up to 24 hours.
    #[serde(default = "default_initial_delay", with = "humantime_serde")]
    initial_delay: Duration,
}

fn deserialize_metric_definitions<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<String, MetricConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<BTreeMap<String, MetricConfig>>::deserialize(deserializer)?.unwrap_or_default())
}

fn default_interval() -> Duration {
    Duration::from_secs(30)
}

fn default_initial_delay() -> Duration {
    Duration::from_secs(1)
}

/// `gauge` and `sum` are mutually exclusive.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetricConfig {
    /// Metric description, kept as written. Defaults to empty.
    #[serde(default)]
    description: String,
    /// Metric unit, kept as written. Defaults to `"1"` when omitted or empty.
    #[serde(default)]
    unit: String,
    /// Selects Gauge. Defaults to Gauge when no kind is set.
    #[serde(default, deserialize_with = "present_marker")]
    gauge: Option<EmptyConfig>,
    /// Selects Sum.
    #[serde(default, deserialize_with = "present_sum")]
    sum: Option<SumConfig>,
}

impl MetricConfig {
    fn kind(&self, name: &str) -> Result<MetricKind, Error> {
        if self.gauge.is_some() && self.sum.is_some() {
            return Err(invalid(format!(
                "metrics.{name} must not contain both gauge and sum"
            )));
        }
        let Some(sum) = &self.sum else {
            return Ok(MetricKind::Gauge);
        };
        if sum.aggregation == Aggregation::Delta {
            return Err(invalid(format!(
                "metrics.{name}: delta Sum is not supported yet; use gauge or sum with aggregation: cumulative and monotonic: false"
            )));
        }
        if sum.monotonic {
            return Err(invalid(format!(
                "metrics.{name}: monotonic Sum (monotonic: true) is not supported yet; use gauge or sum with aggregation: cumulative and monotonic: false"
            )));
        }
        Ok(MetricKind::Sum)
    }
}

/// Treats a present key as selected even when its value is null (e.g. YAML `gauge:`).
fn present_marker<'de, D>(deserializer: D) -> Result<Option<EmptyConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(
        Option::<EmptyConfig>::deserialize(deserializer)?.unwrap_or_default(),
    ))
}

fn present_sum<'de, D>(deserializer: D) -> Result<Option<SumConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<SumConfig>::deserialize(deserializer)?
        .ok_or_else(|| {
            serde::de::Error::custom(
                "sum must be a mapping with explicit aggregation: cumulative; null Sum is invalid",
            )
        })
        .map(Some)
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyConfig {}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SumConfig {
    /// Sum aggregation. Required; only `cumulative` is supported.
    aggregation: Aggregation,
    /// Sum monotonicity. Defaults to `false`; only `false` is supported.
    #[serde(default)]
    monotonic: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectConfig {
    /// Native object name. Surrounding whitespace is trimmed; must be nonblank
    /// and must not contain `\`, `(`, `)`, `*`, `?` or NUL.
    object: String,
    /// Native instance name or list. Defaults to no instance.
    /// If set, the list must be nonempty; each name is trimmed, must be nonblank
    /// and must not contain `\`, `(`, `)`, `*`, `?` or NUL. Numeric suffixes
    /// omit `#0` and leading index zeroes for native paths, preserving configured
    /// spelling after trimming for metric lookup. An index must follow a nonempty
    /// instance name. Duplicate native instances are rejected regardless of case.
    #[serde(default)]
    instances: Option<OneOrMany>,
    /// Counter mappings. At least one.
    counters: Vec<CounterMapping>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CounterMapping {
    /// Native counter name. Surrounding whitespace is trimmed; must be nonblank
    /// and must not contain `\`, `*`, `?` or NUL.
    name: String,
    /// Metric definition name, kept as written. Defaults to the configured counter
    /// path when omitted or empty. Implicit paths use an exact matching definition
    /// or otherwise a Gauge. Nonempty references must be defined. Null is invalid.
    // Point null-reference errors to the specific counter that needs fixing.
    #[serde(default, deserialize_with = "present_metric")]
    metric: Option<Option<String>>,
    /// Point attributes, applied only with an explicit `metric`. Defaults to empty.
    /// Keys are kept verbatim, must not be empty, and must not start with
    /// the exact `windows.perf_counter.` prefix. Values are kept verbatim.
    #[serde(default)]
    attributes: BTreeMap<String, String>,
    /// Multiplies the value by 10 to this power. Defaults to 0; -18 to 18.
    #[serde(default)]
    scale_power10: i32,
}

fn present_metric<'de, D>(deserializer: D) -> Result<Option<Option<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Option::<String>::deserialize(deserializer).map(Some)
}

/// Validated metric metadata with the empty-unit default applied.
struct NormalizedMetric<'a> {
    description: Arc<str>,
    unit: &'a str,
    kind: MetricKind,
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

/// Splits a PDH duplicate-instance suffix, such as `app#2`, into name and index.
fn instance_index(value: &str) -> Option<(&str, &str)> {
    value
        .rsplit_once('#')
        .filter(|(_, index)| !index.is_empty() && index.bytes().all(|b| b.is_ascii_digit()))
}

/// Accepts OTLP stream names without imposing SDK instrument-name restrictions.
fn validate_metric_name(name: &str) -> Result<(), Error> {
    if name.is_empty() {
        return Err(invalid("metric name must not be empty"));
    }
    Ok(())
}

fn is_path_metric_name(name: &str) -> bool {
    name.starts_with('\\')
}

fn unused_path_metric_definitions<'a>(
    names: impl IntoIterator<Item = &'a str>,
    counters: &[CounterConfig],
) -> Vec<&'a str> {
    let selected: HashSet<_> = counters
        .iter()
        .map(|counter| counter.name.as_str())
        .collect();
    names
        .into_iter()
        .filter(|name| is_path_metric_name(name) && !selected.contains(name))
        .collect()
}

fn case_distinct_metric_definitions<'a>(
    names: impl IntoIterator<Item = &'a str>,
) -> Vec<Vec<&'a str>> {
    let mut groups = BTreeMap::<String, Vec<&str>>::new();
    for name in names {
        groups.entry(name.to_lowercase()).or_default().push(name);
    }
    groups
        .into_values()
        .filter(|group| group.len() > 1)
        .collect()
}

fn validate_instance<'a>(field: &str, value: &'a str) -> Result<&'a str, Error> {
    let value = validate_path_segment(field, value, OBJECT_INSTANCE_RESERVED)?;
    if instance_index(value).is_some_and(|(name, _)| name.is_empty()) {
        return Err(invalid(format!(
            "{field} must include an instance name before its index"
        )));
    }
    Ok(value)
}

fn validate_path_segment<'a>(
    field: &str,
    value: &'a str,
    reserved: &[char],
) -> Result<&'a str, Error> {
    let value = value.trim();
    require_name(field, value)?;
    if value.contains(reserved) {
        return Err(invalid(format!(
            "{field} contains a reserved performance-counter path character"
        )));
    }
    Ok(value)
}

fn validate_object<'a>(field: &str, value: &'a str) -> Result<&'a str, Error> {
    validate_path_segment(field, value, OBJECT_INSTANCE_RESERVED)
}

fn validate_counter_name<'a>(field: &str, value: &'a str) -> Result<&'a str, Error> {
    validate_path_segment(field, value, COUNTER_RESERVED)
}

fn validate_path_length(field: &str, path: &str) -> Result<(), Error> {
    if path.encode_utf16().count() > MAX_COUNTER_PATH_LEN {
        return Err(invalid(format!(
            "{field} {path:?} must contain at most {MAX_COUNTER_PATH_LEN} UTF-16 code units"
        )));
    }
    Ok(())
}

/// Defaults an empty unit to "1"; preserves every nonempty unit as written.
fn normalize_metric_unit(unit: &str) -> &str {
    if unit.is_empty() { "1" } else { unit }
}

fn validate_counter(counter: &CounterConfig) -> Result<(), Error> {
    validate_path_length("counter path", &counter.path)?;
    if !(MIN_SCALE_POWER10..=MAX_SCALE_POWER10).contains(&counter.scale_power10) {
        return Err(invalid(format!(
            "counter path {:?} scale_power10 must be between {MIN_SCALE_POWER10} and {MAX_SCALE_POWER10}",
            counter.path
        )));
    }
    Ok(())
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

/// Canonicalizes PDH's decimal instance index, where the first occurrence omits `#0`.
/// Returns `(canonical, configured)`.
fn normalize_instance<'a>(field: &str, value: &'a str) -> Result<(String, &'a str), Error> {
    let value = validate_instance(field, value)?;
    let canonical = match instance_index(value) {
        Some((name, index)) => match index.trim_start_matches('0') {
            "" => name.to_owned(),
            index => format!("{name}#{index}"),
        },
        None => value.to_owned(),
    };
    Ok((canonical, value))
}

/// Rejects only literal empty keys and the receiver-owned prefix.
fn validate_attribute_keys(
    field: &str,
    attributes: &BTreeMap<String, String>,
) -> Result<(), Error> {
    for key in attributes.keys() {
        if key.is_empty() {
            return Err(invalid(format!("{field} attribute key must not be empty")));
        }
        if key.starts_with(RECEIVER_ATTRIBUTE_PREFIX) {
            return Err(invalid(format!(
                "{field} attribute {key:?} conflicts with receiver-generated attributes"
            )));
        }
    }
    Ok(())
}

fn counter_path(object: &str, instance: Option<&str>, counter: &str) -> String {
    match instance {
        Some(instance) => format!(r"\{object}({instance})\{counter}"),
        None => format!(r"\{object}\{counter}"),
    }
}

#[cfg(test)]
mod tests;
