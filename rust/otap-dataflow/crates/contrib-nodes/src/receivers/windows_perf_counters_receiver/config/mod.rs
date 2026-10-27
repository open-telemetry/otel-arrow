// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_config::error::Error;
use serde::Deserialize;
use std::collections::{BTreeMap, HashMap, HashSet};
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
    /// The aggregate instance name, which is unnamed when it is the only point and otherwise
    /// dropped, or `None` when `*` opted it in.
    pub(super) aggregate: Option<Arc<str>>,
    /// Whether a replacement query is attempted after each scrape in which this counter's query
    /// collected; a failed replacement keeps the old query.
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
    /// A null definition is a Gauge.
    #[serde(default)]
    metrics: Option<BTreeMap<String, Option<MetricConfig>>>,
    perfcounters: Vec<ObjectConfig>,
    #[serde(default = "default_interval", with = "humantime_serde")]
    collection_interval: Duration,
    /// Omitted means 1s; null means zero.
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

/// An explicit null (`key:` with no value) is the zero value.
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

#[derive(Clone, PartialEq)]
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
        let mut emitted = HashMap::new();
        for (object_index, object) in user.perfcounters.into_iter().enumerate() {
            let object_field = format!("perfcounters[{object_index}]");
            if object.counters.is_empty() {
                return Err(invalid(format!(
                    "{object_field}.counters must not be empty"
                )));
            }
            check_component(
                &format!("{object_field}.object"),
                &object.object,
                Component::Object,
            )?;
            let aggregate: Arc<str> = match object.aggregation_name.as_str() {
                "" => Arc::from(DEFAULT_AGGREGATION_NAME),
                name => Arc::from(name),
            };
            let names = match object.instances {
                None => Vec::new(),
                // A scalar is a comma-separated list, without trimming, and an empty scalar
                // is an empty list.
                Some(OneOrMany::One(name)) if name.is_empty() => Vec::new(),
                Some(OneOrMany::One(name)) => name.split(',').map(str::to_owned).collect(),
                Some(OneOrMany::Many(names)) => names,
            };
            // Validate every configured name before `*` replaces the others.
            for (index, name) in names.iter().enumerate() {
                check_component(
                    &format!("{object_field}.instances[{index}]"),
                    name,
                    Component::Instance,
                )?;
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
                    check_points(
                        &mut emitted,
                        &field,
                        &path,
                        &metric,
                        &counter.attributes,
                        instance.as_deref(),
                        aggregate.as_deref(),
                    )?;
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
/// `*` replaces any other names, and listing the aggregate beside `*` opts it in.
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

/// Names the items of one array read, given in returned order with whether each is kept.
///
/// The aggregate is the last returned item with the aggregate name, by position before status
/// filtering; every other item is a real instance. Among kept items, a lone aggregate is unnamed and
/// any other aggregate is dropped; the second and later occurrences of a name get the lowest free
/// "#1", "#2", ... suffix that is not also a returned name. Yields each emitted item's position and
/// its instance, `None` when unnamed.
pub(super) fn emitted_instances<'a>(
    items: impl Iterator<Item = (&'a str, bool)> + Clone + 'a,
    aggregate: Option<&'a str>,
) -> impl Iterator<Item = (usize, Option<String>)> + 'a {
    let aggregate = aggregate.and_then(|aggregate| {
        items
            .clone()
            .enumerate()
            .filter(|(_, (name, _))| *name == aggregate)
            .last()
            .map(|(position, _)| position)
    });
    let kept = items.enumerate().filter(|(_, (_, kept))| *kept);
    let lone = kept.clone().count() == 1;
    let returned: HashSet<&str> = kept.clone().map(|(_, (name, _))| name).collect();
    let mut next = HashMap::<&str, u32>::new();
    kept.filter_map(move |(position, (name, _))| {
        if Some(position) == aggregate {
            return lone.then_some((position, None));
        }
        let suffix = next.entry(name).or_default();
        let mut instance = name.to_owned();
        while *suffix > 0 && returned.contains(instance.as_str()) {
            instance = format!("{name}#{suffix}");
            *suffix += 1;
        }
        *suffix = (*suffix).max(1);
        Some((position, (!instance.is_empty()).then_some(instance)))
    })
}

/// PDH reads an instance index as a number: `name#0` and `name#00` are `name`, and `name#01` is
/// `name#1`.
fn canonical_index(name: &str) -> std::borrow::Cow<'_, str> {
    match name.rsplit_once('#') {
        Some((base, index)) if !index.is_empty() && index.bytes().all(|b| b.is_ascii_digit()) => {
            match index.trim_start_matches('0') {
                "" => base.into(),
                index => format!("{base}#{index}").into(),
            }
        }
        _ => name.into(),
    }
}

/// The `instance` an entry's points carry: any for a wildcard, else a static value compared
/// exactly or an emitted name compared ignoring case, as PDH resolves it; `None` is no instance.
enum Identity {
    Any,
    Static(String),
    Emitted(Option<String>),
}

impl Identity {
    fn collides(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Any, _) | (_, Self::Any) => true,
            (Self::Static(a), Self::Static(b)) => a == b,
            (
                Self::Static(a) | Self::Emitted(Some(a)),
                Self::Static(b) | Self::Emitted(Some(b)),
            ) => a.to_lowercase() == b.to_lowercase(),
            (Self::Emitted(a), Self::Emitted(b)) => a.is_none() && b.is_none(),
            _ => false,
        }
    }
}

/// The earlier entries of one metric name.
struct Emitted {
    /// The field of the first definition.
    field: String,
    /// The first definition, which every later entry must match.
    metric: Metric,
    entries: Vec<Entry>,
}

/// One emitted series of a metric: its field and path, static attributes and instance.
struct Entry {
    name: String,
    attributes: BTreeMap<String, String>,
    identity: Identity,
}

/// A metric name has one definition, and points sharing a metric, static attributes and instance
/// cannot be told apart, so rejects an entry that conflicts with an earlier one.
fn check_points(
    emitted: &mut HashMap<Arc<str>, Emitted>,
    field: &str,
    path: &str,
    metric: &Metric,
    attributes: &BTreeMap<String, String>,
    instance: Option<&str>,
    aggregate: Option<&str>,
) -> Result<(), Error> {
    let Emitted {
        field: other,
        metric: first,
        entries,
    } = emitted
        .entry(Arc::clone(&metric.name))
        .or_insert_with(|| Emitted {
            field: field.to_owned(),
            metric: metric.clone(),
            entries: Vec::new(),
        });
    if first != metric {
        let differs = match (first.kind, metric.kind) {
            (MetricKind::Sum { temporality: a, .. }, MetricKind::Sum { temporality: b, .. })
                if a != b =>
            {
                "sum.aggregation"
            }
            (MetricKind::Sum { .. }, MetricKind::Sum { .. }) if first.kind != metric.kind => {
                "sum.monotonic"
            }
            _ if first.kind != metric.kind => "gauge or sum",
            _ if first.unit != metric.unit => "unit",
            _ => "description",
        };
        return Err(invalid(format!(
            "{other} and {field} use metric {:?} with a different {differs}; give one of them a different metric name",
            metric.name
        )));
    }
    let mut attributes = attributes.clone();
    let wildcard = instance.is_some_and(|name| name.contains(['*', '?']));
    let identity = match (attributes.remove("instance"), instance) {
        (Some(_), _) if wildcard => {
            return Err(invalid(format!(
                "{field}.attributes.instance: a static instance on wildcard instances collapses every instance into one series; remove the static instance or select a specific instance"
            )));
        }
        (Some(value), _) => Identity::Static(value),
        (None, _) if wildcard => Identity::Any,
        (None, Some(name)) => Identity::Emitted(
            emitted_instances(
                std::iter::once((canonical_index(name).as_ref(), true)),
                aggregate,
            )
            .next()
            .and_then(|(_, instance)| instance),
        ),
        (None, None) => Identity::Emitted(None),
    };
    let name = format!("{field} ({path})");
    if let Some(other) = entries
        .iter()
        .find(|seen| seen.attributes == attributes && seen.identity.collides(&identity))
    {
        return Err(invalid(format!(
            "{} and {name} would emit indistinguishable points for metric {:?}; give one of them a static attribute with a distinct value",
            other.name, metric.name
        )));
    }
    entries.push(Entry {
        name,
        attributes,
        identity,
    });
    Ok(())
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
    check_component(&format!("{field}.name"), &mapping.name, Component::Counter)?;
    if mapping.attributes.contains_key("") {
        return Err(invalid(format!(
            "{field}.attributes: attribute keys must not be empty"
        )));
    }
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

/// The part of a counter path a configured name fills.
#[derive(PartialEq)]
enum Component {
    Object,
    Instance,
    Counter,
}

/// Accepts a name only in its own path role: no empty part; no `\` or NUL in any part, since paths
/// are local and NUL-terminated; wildcards only in instances; no whole-counter `*`; and no
/// parentheses in the object, since `instances` selects instances.
fn check_component(field: &str, value: &str, component: Component) -> Result<(), Error> {
    if value.is_empty() {
        return Err(invalid(format!("{field} must not be empty")));
    }
    let message = if value.contains(['\\', '\0']) {
        "remote or malformed counter paths are not supported"
    } else if component == Component::Counter && value == "*" {
        "whole-counter wildcard \"*\" is not supported yet; list counter names explicitly"
    } else if component != Component::Instance && value.contains(['*', '?']) {
        "wildcards are supported only in instance names"
    } else if component == Component::Object && value.contains(['(', ')']) {
        "select instances with `instances`, not parentheses in the object name"
    } else {
        return Ok(());
    };
    Err(invalid(format!("{field}: {message}")))
}

#[cfg(test)]
mod tests;
