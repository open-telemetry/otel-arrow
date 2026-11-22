// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Compare the catalog actually served to the UI with the schema and wire samples.
use crate::UiSignal;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
/// Telemetry dependencies read from the JavaScript module shipped with the dashboard.
pub struct Inputs {
    #[serde(skip)]
    platform: UiPlatform,
    sets: BTreeMap<String, Set>,
    dimensions: BTreeMap<String, String>,
    /// Lifecycle and log-stream inputs.
    pub events: Events,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
/// Explicit dependencies of the status overlay and live log panel.
pub struct Events {
    /// Public status event categories keyed by UI role.
    pub kinds: BTreeMap<String, String>,
    /// Condition names the status selector reads.
    pub conditions: BTreeMap<String, String>,
    /// Fields the live log panel renders.
    pub log_fields: BTreeMap<String, String>,
    /// Explicit event names read by the UI, if any.
    pub named_logs: Vec<String>,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Set {
    name: String,
    consumer: String,
    metrics: BTreeMap<String, String>,
    #[serde(default)]
    dimensions: Vec<String>,
    #[serde(default)]
    fallback_dimensions: BTreeMap<String, String>,
    #[serde(default)]
    platform: BTreeMap<String, String>,
    #[serde(default)]
    legacy: BTreeMap<String, String>,
    #[serde(default)]
    series: BTreeMap<String, Series>,
}

#[derive(Deserialize)]
struct Series {
    metric: String,
    attributes: BTreeMap<String, String>,
    #[serde(default)]
    traffic: bool,
}

/// Metrics routes available to the deployed dashboard.
pub const METRICS: [&str; 2] = ["/api/v1/telemetry/metrics", "/api/v1/metrics"];
const JOINS: [(&str, &str, &[&str], &str); 2] = [
    ("sender", "receiver", &["channel"], "Pipeline DAG"),
    (
        "publisher",
        "subscriber",
        &["process", "topic"],
        "Topic topology",
    ),
];

/// The candidate includes this same module in its embedded UI.
pub const SHIPPED_CATALOG: &str = include_str!("../../admin/ui/js/chart-inputs.js");

/// Pipeline that owns the fixture action for each status-event category.
#[must_use]
pub fn event_pipeline(key: &str) -> Option<&'static str> {
    match key {
        "request" => Some("default:observer"),
        "error" => Some("default:broken"),
        "success" => Some("default:main"),
        _ => None,
    }
}

/// Read the dependency object from the served JavaScript without executing the module.
pub fn catalog_object(source: &str) -> Result<Value> {
    let (_, object) = source
        .split_once("export const telemetryInputs =")
        .context("missing UI telemetry catalog")?;
    Ok(Value::deserialize(
        &mut serde_json::Deserializer::from_str(object),
    )?)
}

impl Inputs {
    /// Load the dependency object and validate its internal references.
    pub fn parse(source: &str) -> Result<Self> {
        let inputs: Self = serde_json::from_value(catalog_object(source)?)?;
        ensure!(!inputs.sets.is_empty(), "empty UI catalog");
        for set in inputs.sets.values() {
            ensure!(
                set.dimensions
                    .iter()
                    .chain(set.fallback_dimensions.keys())
                    .all(|d| inputs.dimensions.contains_key(d))
                    && set
                        .series
                        .values()
                        .all(|s| set.metrics.values().any(|name| name == &s.metric))
                    && set.platform.keys().all(|k| set.metrics.contains_key(k)),
                "unknown UI dependency in {}",
                set.name
            );
        }
        for (left, right, keys, _) in JOINS {
            if inputs.sets.contains_key(left) && inputs.sets.contains_key(right) {
                ensure!(
                    keys.iter().all(|key| inputs.dimensions.contains_key(*key)),
                    "unknown topology join dimension"
                );
            }
        }
        Ok(inputs)
    }

    /// Compare schema declarations and samples, optionally against a pre-traffic baseline.
    pub fn measure(
        &self,
        schema: &Value,
        data: &Value,
        path: &str,
        before: Option<&BTreeMap<String, f64>>,
    ) -> Result<Measurements> {
        let descriptors = schema["metric_sets"]
            .as_array()
            .context("missing live-schema metric_sets")?;
        let samples = data["metric_sets"]
            .as_array()
            .context("missing metrics response metric_sets")?;
        let mut measured = Measurements::default();
        for (key, set) in &self.sets {
            let declarations: Vec<_> = descriptors
                .iter()
                .filter(|d| d["name"] == set.name)
                .collect();
            let sets: Vec<_> = samples.iter().filter(|s| s["name"] == set.name).collect();
            let scoped: Vec<_> = sets
                .iter()
                .copied()
                .filter(|s| self.in_scope(key, set, &s["attributes"]))
                .collect();
            let declared_dimensions = set.dimensions.iter().all(|key| {
                schema["attributes"]
                    .as_array()
                    .is_some_and(|ds| ds.iter().any(|d| d["key"] == self.dimensions[key]))
            });
            let set_result = if declarations.is_empty() || !declared_dimensions {
                "missing from schema"
            } else if sets.is_empty() {
                "declared but absent from metrics"
            } else if scoped.is_empty() {
                "present with unusable dimensions"
            } else {
                "declared and observed"
            };
            let mut note = |name: &str, result: &str| {
                let signal = format!("{}{} ({path})", set.name, name);
                measured.rows.push(UiSignal {
                    consumer: set.consumer.clone(),
                    signal,
                    result: result.into(),
                });
            };
            if set.metrics.is_empty() {
                note("", set_result);
            }
            for name in set.legacy.values() {
                note(&format!(".{name}"), "legacy UI fallback");
            }
            for (field, name) in &set.metrics {
                if let Some(platform) = set.platform.get(field)
                    && !self.platform.available(platform)?
                {
                    note(
                        &format!(".{name}"),
                        &format!("platform-inapplicable ({platform})"),
                    );
                    continue;
                }
                let declared = declarations.iter().any(|d| {
                    d["metrics"]
                        .as_array()
                        .is_some_and(|ms| ms.iter().any(|m| m["name"] == *name))
                });
                let points: Vec<_> = scoped
                    .iter()
                    .filter_map(|s| s["metrics"].as_array())
                    .flatten()
                    .filter(|m| m["name"] == *name)
                    .collect();
                let status = if !declared {
                    "missing from schema"
                } else if set_result != "declared and observed" {
                    set_result
                } else if points.is_empty() {
                    if sets
                        .iter()
                        .filter_map(|s| s["metrics"].as_array())
                        .flatten()
                        .any(|m| m["name"] == *name)
                    {
                        "present with unusable dimensions"
                    } else {
                        "declared but absent from metrics"
                    }
                } else if points.iter().all(|p| p["value"].as_f64().is_none()) {
                    "unusable numeric sample"
                } else {
                    "declared and observed"
                };
                note(&format!(".{name}"), status);
                for (series_name, series) in set.series.iter().filter(|(_, s)| s.metric == *name) {
                    let signal = format!("{}.{name}/{series_name}", set.name);
                    let values: Vec<_> = points
                        .iter()
                        .filter(|m| {
                            series
                                .attributes
                                .iter()
                                .all(|(k, v)| attr(&m["attributes"], k) == Some(v))
                        })
                        .filter_map(|m| m["value"].as_f64())
                        .collect();
                    let total: f64 = values.iter().sum();
                    let result = if status != "declared and observed" {
                        status
                    } else if values.is_empty() {
                        "present with unusable dimensions"
                    } else if series.traffic
                        && before.is_some_and(|b| b.get(&signal).is_none_or(|old| total <= *old))
                    {
                        "no increase after marked traffic"
                    } else {
                        "declared and observed"
                    };
                    if series.traffic && !values.is_empty() {
                        let _ = measured.traffic.insert(signal, total);
                    }
                    let labels = series
                        .attributes
                        .iter()
                        .map(|(k, v)| format!("{k}={v}"))
                        .collect::<Vec<_>>()
                        .join(", ");
                    let suffix = if labels.is_empty() {
                        String::new()
                    } else {
                        format!(" [{labels}]")
                    };
                    note(&format!(".{name}/{series_name}{suffix}"), result);
                }
            }
        }
        for (from, to, keys, consumer) in JOINS {
            let (Some(left), Some(right)) = (self.sets.get(from), self.sets.get(to)) else {
                continue;
            };
            let endpoints = |role, set: &Set| -> BTreeSet<Vec<String>> {
                samples
                    .iter()
                    .filter(|s| s["name"] == set.name && self.in_scope(role, set, &s["attributes"]))
                    .filter_map(|s| {
                        keys.iter()
                            .map(|key| {
                                attr(&s["attributes"], &self.dimensions[*key])
                                    .map(str::to_owned)
                                    .or_else(|| set.fallback_dimensions.get(*key).cloned())
                            })
                            .collect()
                    })
                    .collect()
            };
            let linked = !endpoints(from, left).is_disjoint(&endpoints(to, right));
            measured.rows.push(UiSignal {
                consumer: consumer.into(),
                signal: format!("{} -> {} ({path})", left.name, right.name),
                result: if linked {
                    "declared and observed"
                } else {
                    "present with unusable dimensions"
                }
                .into(),
            });
        }
        Ok(measured)
    }

    fn in_scope(&self, key: &str, set: &Set, attrs: &Value) -> bool {
        let get = |key: &str| self.dimensions.get(key).and_then(|name| attr(attrs, name));
        if !set.dimensions.iter().all(|key| {
            let v = &attrs[&self.dimensions[key]];
            v.as_str().is_some_and(|s| !s.is_empty())
                || v.is_number()
                || v.as_object().is_some_and(|o| {
                    o.values()
                        .any(|v| v.is_number() || v.as_str().is_some_and(|s| !s.is_empty()))
                })
        }) {
            return false;
        }
        if set.dimensions.iter().any(|d| d == "pipeline")
            && (get("group") != Some("default")
                || get("pipeline")
                    != Some(if key == "subscriber" {
                        "reader"
                    } else {
                        "main"
                    }))
        {
            return false;
        }
        match key {
            "sender" | "receiver" => {
                get("kind") == Some("pdata")
                    && get("node") == Some(if key == "sender" { "input" } else { "out" })
            }
            "publisher" | "subscriber" => {
                get("topic") == Some("group::default::ui")
                    && get("node") == Some(if key == "publisher" { "out" } else { "topic" })
            }
            _ => true,
        }
    }
}

/// Target capabilities of the running dashboard deployment.
#[derive(Deserialize, Serialize, Clone)]
pub struct UiPlatform {
    os: String,
    atomic64: bool,
}

impl Default for UiPlatform {
    fn default() -> Self {
        Self {
            os: std::env::consts::OS.to_owned(),
            atomic64: cfg!(target_has_atomic = "64"),
        }
    }
}

impl UiPlatform {
    fn available(&self, platform: &str) -> Result<bool> {
        Ok(match platform {
            "windows" => self.os == "windows",
            "non-windows" => self.os != "windows",
            "unix-rusage" => matches!(self.os.as_str(), "linux" | "freebsd" | "openbsd"),
            "atomic64" => self.atomic64,
            _ => anyhow::bail!("unknown signal platform {platform}"),
        })
    }
}

#[derive(Default)]
/// Per-endpoint observations from a deployed metrics snapshot.
pub struct Measurements {
    /// One diagnostic per measured input.
    pub rows: Vec<UiSignal>,
    /// Cumulative counters in the selected workload scope.
    pub traffic: BTreeMap<String, f64>,
}

impl Measurements {
    /// Whether every applicable signal supplied its expected evidence.
    #[must_use]
    pub fn passed(&self) -> bool {
        self.rows.iter().all(|r| {
            r.result == "declared and observed"
                || r.result == "legacy UI fallback"
                || r.result.starts_with("platform-inapplicable")
        })
    }
}

fn attr<'a>(attrs: &'a Value, key: &str) -> Option<&'a str> {
    attrs[key]
        .as_str()
        .or_else(|| attrs[key]["String"].as_str())
}

impl Inputs {
    /// Reject a passing headline if CI lost any required observation or reported a failure.
    pub fn verify_success(mut self, platform: &UiPlatform, rows: &[UiSignal]) -> Result<()> {
        self.platform = platform.clone();
        let mut actual = BTreeMap::new();
        for row in rows {
            ensure!(
                actual
                    .insert(
                        (row.consumer.clone(), row.signal.clone()),
                        row.result.as_str()
                    )
                    .is_none(),
                "duplicate UI observation: {} / {}",
                row.consumer,
                row.signal
            );
        }
        let mut require = |consumer: &str, signal: &str, expected: &str| -> Result<()> {
            let result = actual.remove(&(consumer.to_owned(), signal.to_owned()));
            ensure!(
                result == Some(expected),
                "incomplete UI observation: {consumer} / {signal}: expected {expected}, got {result:?}"
            );
            Ok(())
        };
        let empty = serde_json::json!({"metric_sets":[]});
        for endpoint in METRICS {
            for row in self.measure(&empty, &empty, endpoint, None)?.rows {
                let expected = if row.result == "legacy UI fallback"
                    || row.result.starts_with("platform-inapplicable")
                {
                    row.result.as_str()
                } else {
                    "declared and observed"
                };
                require(&row.consumer, &row.signal, expected)?;
            }
        }
        require(
            "Live logs",
            "log stream and retained event fields",
            "observed",
        )?;
        require(
            "Status overlay",
            "Ready and Accepted conditions, Success.Ready",
            "observed",
        )?;
        for (key, kind) in &self.events.kinds {
            let pipeline = event_pipeline(key).context("missing lifecycle trigger")?;
            require(
                "Status overlay",
                &format!("{pipeline} {kind}"),
                "declared and observed",
            )?;
        }
        for name in &self.events.named_logs {
            require("Live logs", name, "declared and observed")?;
        }
        ensure!(
            actual.is_empty(),
            "unknown UI observations: {:?}",
            actual.keys()
        );
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: a CI artifact claims the UI passed with no signal observations.
    /// Guarantees: the headline alone cannot substitute for the shipped catalog.
    #[test]
    fn passing_ui_requires_each_observation() {
        let inputs = Inputs::parse(SHIPPED_CATALOG).unwrap();
        assert!(inputs.verify_success(&UiPlatform::default(), &[]).is_err());
    }

    /// Scenario: a channel metric is undeclared, unpublished, mislabelled, legacy-only, or stationary.
    /// Guarantees: each failure is distinct and only measured traffic earns a passing observation.
    #[test]
    fn deployment_signal_failures() {
        let inputs: Inputs = serde_json::from_value(json!({"sets":{"probe":{"name":"edge","consumer":"DAG","metrics":{"messages":"messages"},"legacy":{"old":"send.count"},"series":{"sent":{"metric":"messages","attributes":{"outcome":"success"},"traffic":true}}}},"dimensions":{},"events":{"kinds":{},"conditions":{},"logFields":{},"namedLogs":[]}})).unwrap();
        let schema = json!({"metric_sets":[{"name":"edge","metrics":[{"name":"messages"}]}]});
        let data = |name, label, value| json!({"metric_sets":[{"name":"edge","metrics":[{"name":name,"attributes":{"outcome":label},"value":value}]}]});
        let before = BTreeMap::from([("edge.messages/sent".to_owned(), 1.0)]);
        for (declarations, snapshot, reason) in [
            (
                json!({"metric_sets":[]}),
                data("messages", "success", 2),
                "missing from schema",
            ),
            (
                schema.clone(),
                json!({"metric_sets":[]}),
                "declared but absent from metrics",
            ),
            (
                schema.clone(),
                data("messages", "failure", 2),
                "present with unusable dimensions",
            ),
            (
                schema.clone(),
                data("send.count", "success", 2),
                "declared but absent from metrics",
            ),
            (
                schema.clone(),
                data("messages", "success", 1),
                "no increase after marked traffic",
            ),
        ] {
            let evidence = inputs
                .measure(&declarations, &snapshot, "metrics", Some(&before))
                .unwrap();
            assert!(!evidence.passed());
            assert!(
                evidence
                    .rows
                    .iter()
                    .any(|row| row.consumer == "DAG" && row.result == reason)
            );
        }
        assert!(
            inputs
                .measure(
                    &schema,
                    &data("messages", "success", 2),
                    "metrics",
                    Some(&before)
                )
                .unwrap()
                .passed()
        );
    }
}
