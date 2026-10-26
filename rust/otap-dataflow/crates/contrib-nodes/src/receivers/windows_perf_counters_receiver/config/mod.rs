// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_config::error::Error;
use serde::Deserialize;
use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::time::Duration;

pub(super) const MIN_SCALE_POWER10: i32 = -18;
pub(super) const MAX_SCALE_POWER10: i32 = 18;

const DEFAULT_AGGREGATION_NAME: &str = "_Total";

/// Configuration normalized for Windows performance-counter collection.
#[derive(Debug, Clone)]
pub(super) struct Config {
    /// One entry per PDH path to add.
    pub(super) counters: Vec<CounterConfig>,
    /// Time between collections; defaults to one minute.
    pub(super) collection_interval: Duration,
    /// Delay before the first collection; defaults to one second.
    pub(super) initial_delay: Duration,
    /// How long a scrape may run before the overrun warning; `None` warns at the next tick.
    pub(super) timeout: Option<Duration>,
}

/// One PDH path and the metric stream its values feed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct CounterConfig {
    /// English PDH path, optionally with a wildcard instance.
    pub(super) path: String,
    /// The aggregate instance name to clear or drop, or `None` when `*` opted it in.
    pub(super) aggregate: Option<Arc<str>>,
    /// Whether this path gets its own query, replaced after every scrape.
    pub(super) recreate_query: bool,
    /// OTel metric name; counters may share one definition.
    pub(super) name: Arc<str>,
    pub(super) unit: Arc<str>,
    pub(super) description: Arc<str>,
    pub(super) metric_kind: MetricKind,
    /// Static attributes from the counter mapping.
    pub(super) attributes: Arc<BTreeMap<String, String>>,
    /// Base-10 scaling applied to the formatted value.
    pub(super) scale_power10: i32,
}

/// Supported OTel metric stream kinds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum MetricKind {
    Gauge,
    Sum {
        temporality: Temporality,
        monotonic: bool,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub(super) enum Temporality {
    Cumulative,
    Delta,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct UserConfig {
    /// A null definition is the zero value, a Gauge, as in Go.
    #[serde(default)]
    metrics: Option<BTreeMap<String, Option<MetricConfig>>>,
    perfcounters: Vec<ObjectConfig>,
    #[serde(default = "default_interval", with = "humantime_serde")]
    collection_interval: Duration,
    /// Omitted means 1s; null is the zero value, as in Go.
    #[serde(default = "default_initial_delay", with = "humantime_serde::option")]
    initial_delay: Option<Duration>,
    #[serde(default, with = "humantime_serde")]
    timeout: Option<Duration>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct MetricConfig {
    #[serde(default, deserialize_with = "null_default")]
    description: String,
    #[serde(default, deserialize_with = "null_default")]
    unit: String,
    #[serde(default, deserialize_with = "present_marker")]
    gauge: Option<EmptyConfig>,
    #[serde(default)]
    sum: Option<SumConfig>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct EmptyConfig {}

/// A present Sum mapping must set `aggregation`; a null `sum` counts as absent.
#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct SumConfig {
    #[serde(default)]
    aggregation: Option<Temporality>,
    #[serde(default, deserialize_with = "null_default")]
    monotonic: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ObjectConfig {
    object: String,
    #[serde(default)]
    instances: Option<OneOrMany>,
    #[serde(default, deserialize_with = "null_default")]
    aggregation_name: String,
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

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct CounterMapping {
    name: String,
    #[serde(default)]
    metric: Option<String>,
    #[serde(default, deserialize_with = "null_default")]
    recreate_query: bool,
    #[serde(default, deserialize_with = "null_default")]
    attributes: BTreeMap<String, String>,
    #[serde(default)]
    scale_power10: i32,
}

fn present_marker<'de, D>(deserializer: D) -> Result<Option<EmptyConfig>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Some(
        Option::<EmptyConfig>::deserialize(deserializer)?.unwrap_or_default(),
    ))
}

fn default_interval() -> Duration {
    Duration::from_secs(60)
}

/// As the Collector's confmap does, an explicit null (`key:` with no value) is the zero value.
fn null_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::<T>::deserialize(deserializer)?.unwrap_or_default())
}

fn default_initial_delay() -> Option<Duration> {
    Some(Duration::from_secs(1))
}

fn invalid(error: impl Into<String>) -> Error {
    Error::InvalidUserConfig {
        error: error.into(),
    }
}

#[derive(Clone)]
struct Metric {
    name: Arc<str>,
    unit: Arc<str>,
    description: Arc<str>,
    kind: MetricKind,
}

impl Config {
    /// Parse, validate, and normalize the public configuration.
    pub(super) fn from_json(value: &serde_json::Value) -> Result<Self, Error> {
        let user = UserConfig::deserialize(value).map_err(|err| invalid(err.to_string()))?;
        if user.collection_interval.is_zero() {
            return Err(invalid("collection_interval must be greater than zero"));
        }
        if user.perfcounters.is_empty() {
            return Err(invalid("perfcounters must not be empty"));
        }
        let metrics = user
            .metrics
            .unwrap_or_default()
            .into_iter()
            .map(|(name, metric)| {
                let metric = metric.unwrap_or_default();
                let kind = match (&metric.gauge, &metric.sum) {
                    (Some(_), Some(_)) => {
                        return Err(invalid(format!(
                            "metrics.{name} must not contain both gauge and sum"
                        )));
                    }
                    (_, Some(sum)) => match sum.aggregation {
                        Some(temporality) => MetricKind::Sum {
                            temporality,
                            monotonic: sum.monotonic,
                        },
                        None => {
                            return Err(invalid(format!(
                                "metrics.{name}.sum requires aggregation"
                            )));
                        }
                    },
                    (_, None) => MetricKind::Gauge,
                };
                let unit = if metric.unit.is_empty() {
                    "1"
                } else {
                    &metric.unit
                };
                let metric = Metric {
                    name: Arc::from(name.as_str()),
                    unit: Arc::from(unit),
                    description: Arc::from(metric.description),
                    kind,
                };
                Ok((name, metric))
            })
            .collect::<Result<BTreeMap<_, _>, Error>>()?;

        let mut counters = Vec::new();
        let mut seen = HashSet::new();
        for (object_index, object) in user.perfcounters.into_iter().enumerate() {
            let object_field = format!("perfcounters[{object_index}]");
            if object.object.is_empty() {
                return Err(invalid(format!("{object_field}.object must not be empty")));
            }
            if object.counters.is_empty() {
                return Err(invalid(format!(
                    "{object_field}.counters must not be empty"
                )));
            }
            reject_wildcard(&format!("{object_field}.object"), &object.object)?;
            reject_backslash(&format!("{object_field}.object"), &object.object)?;
            let aggregate: Arc<str> = match object.aggregation_name.as_str() {
                "" => Arc::from(DEFAULT_AGGREGATION_NAME),
                name => Arc::from(name),
            };
            let names = match object.instances {
                None => Vec::new(),
                // As Go's confmap does, a scalar is a comma-separated list, without trimming,
                // and an empty scalar is an empty list.
                Some(OneOrMany::One(name)) if name.is_empty() => Vec::new(),
                Some(OneOrMany::One(name)) => name.split(',').map(str::to_owned).collect(),
                Some(OneOrMany::Many(names)) => names,
            };
            // Validate every configured name before `*` replaces the others.
            if names.iter().any(String::is_empty) {
                return Err(invalid(format!(
                    "{object_field}.instances must not contain an empty name"
                )));
            }
            for name in &names {
                reject_backslash(&format!("{object_field}.instances"), name)?;
            }
            let (instances, aggregate) = instances(names, aggregate);
            for (counter_index, mapping) in object.counters.into_iter().enumerate() {
                let field = format!("{object_field}.counters[{counter_index}]");
                let counter = normalize_counter(&field, &metrics, mapping)?;
                for instance in &instances {
                    let path = match instance {
                        Some(instance) => {
                            format!(r"\{}({instance})\{}", object.object, counter.name)
                        }
                        None => format!(r"\{}\{}", object.object, counter.name),
                    };
                    let metric = counter.metric.clone().unwrap_or_else(|| Metric {
                        name: Arc::from(path.as_str()),
                        unit: Arc::from("1"),
                        description: Arc::from(""),
                        kind: MetricKind::Gauge,
                    });
                    // Identical entries would only emit duplicate points.
                    if !seen.insert((
                        path.to_lowercase(),
                        Arc::clone(&metric.name),
                        Arc::clone(&counter.attributes),
                    )) {
                        return Err(invalid(format!(
                            "{field}: duplicate counter path {path} with the same metric and attributes"
                        )));
                    }
                    counters.push(CounterConfig {
                        path,
                        aggregate: aggregate.clone(),
                        recreate_query: counter.recreate_query,
                        name: metric.name,
                        unit: metric.unit,
                        description: metric.description,
                        metric_kind: metric.kind,
                        attributes: Arc::clone(&counter.attributes),
                        scale_power10: counter.scale_power10,
                    });
                }
            }
        }
        Ok(Self {
            counters,
            collection_interval: user.collection_interval,
            initial_delay: user.initial_delay.unwrap_or_default(),
            timeout: user.timeout.filter(|timeout| !timeout.is_zero()),
        })
    }
}

/// Configured instances, one per path, and the aggregate name the reads clear or drop.
///
/// As in Go, `*` replaces any other names, and listing the aggregate beside `*` opts it in.
fn instances(names: Vec<String>, aggregate: Arc<str>) -> (Vec<Option<String>>, Option<Arc<str>>) {
    if names.is_empty() {
        return (vec![None], Some(aggregate));
    }
    if names.iter().any(|name| name == "*") {
        let opted_in = names.iter().any(|name| **name == *aggregate);
        return (vec![Some("*".to_owned())], (!opted_in).then_some(aggregate));
    }
    (names.into_iter().map(Some).collect(), Some(aggregate))
}

/// A validated counter mapping, shared by the paths of each configured instance.
struct Mapping {
    name: String,
    metric: Option<Metric>,
    recreate_query: bool,
    attributes: Arc<BTreeMap<String, String>>,
    scale_power10: i32,
}

fn normalize_counter(
    field: &str,
    metrics: &BTreeMap<String, Metric>,
    mapping: CounterMapping,
) -> Result<Mapping, Error> {
    if mapping.name == "*" {
        return Err(invalid(format!(
            "{field}.name: whole-counter wildcard \"*\" is not supported yet; list counter names explicitly"
        )));
    }
    reject_wildcard(&format!("{field}.name"), &mapping.name)?;
    reject_backslash(&format!("{field}.name"), &mapping.name)?;
    if !(MIN_SCALE_POWER10..=MAX_SCALE_POWER10).contains(&mapping.scale_power10) {
        return Err(invalid(format!(
            "{field}.scale_power10 must be between {MIN_SCALE_POWER10} and {MAX_SCALE_POWER10}"
        )));
    }
    let metric = match mapping.metric.as_deref() {
        None | Some("") => None,
        Some(name) => Some(metrics.get(name).cloned().ok_or_else(|| {
            invalid(format!(
                "{field}.metric references undefined metric {name:?}"
            ))
        })?),
    };
    Ok(Mapping {
        name: mapping.name,
        metric,
        recreate_query: mapping.recreate_query,
        attributes: Arc::new(mapping.attributes),
        scale_power10: mapping.scale_power10,
    })
}

fn reject_wildcard(field: &str, value: &str) -> Result<(), Error> {
    if value.contains(['*', '?']) {
        return Err(invalid(format!(
            "{field}: wildcards are supported only in instance names"
        )));
    }
    Ok(())
}

fn reject_backslash(field: &str, value: &str) -> Result<(), Error> {
    if value.contains('\\') {
        return Err(invalid(format!(
            "{field}: remote or malformed counter paths are not supported"
        )));
    }
    Ok(())
}

#[cfg(test)]
mod tests;
