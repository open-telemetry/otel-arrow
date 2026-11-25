// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Evaluate the dashboard's YAML expectations against live schema and wire samples.
mod aggregation;
mod status;
pub use status::{PENDING, PipelineState, PipelineStates, UNHEALTHY};

use crate::UiSignal;
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Telemetry dependencies declared in the dashboard's YAML catalog.
pub struct Inputs {
    #[serde(skip)]
    platform: UiPlatform,
    sets: BTreeMap<String, Set>,
    #[serde(default)]
    all_cores: Vec<aggregation::AllCores>,
    dimensions: BTreeMap<String, String>,
    /// Lifecycle and log-stream inputs.
    pub events: Events,
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
/// Explicit dependencies of the status overlay and live log panel.
pub struct Events {
    /// Public status event categories keyed by UI role.
    pub kinds: BTreeMap<String, String>,
    /// Condition names the status selector reads.
    pub conditions: BTreeMap<String, String>,
    /// Expected condition values and core counts in each deployed status fixture.
    pub pipeline_states: PipelineStates,
    /// Error-frame field used by the log viewer to request missed events.
    pub lag_cursor: String,
    /// Fields the live log panel renders.
    pub log_fields: BTreeMap<String, String>,
    /// Explicit event names read by the UI, if any.
    pub named_logs: Vec<String>,
}

/// One consumer's expected metric set, including the dimensions and series it reads.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct Set {
    name: String,
    consumer: String,
    metrics: BTreeMap<String, Metric>,
    channel: Option<ChannelScope>,
    #[serde(default)]
    dimensions: Vec<String>,
    #[serde(default)]
    fallback_dimensions: BTreeMap<String, String>,
    #[serde(default)]
    platform: BTreeMap<String, String>,
    #[serde(default)]
    series: BTreeMap<String, Series>,
}

/// Select one fixture channel by kind and node so another channel cannot satisfy the observation.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct ChannelScope {
    kind: String,
    node: String,
}

/// Wire name and metadata expected in both the schema declaration and emitted sample.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Metric {
    name: String,
    unit: String,
    instrument: String,
    temporality: Option<String>,
}

impl Metric {
    fn metadata_error(&self, point: &Value, surface: &str) -> Option<String> {
        if point["brief"]
            .as_str()
            .is_none_or(|brief| brief.trim().is_empty())
        {
            return Some(format!("missing {surface} metric brief"));
        }
        for (field, expected) in [
            ("unit", Some(self.unit.as_str())),
            ("instrument", Some(self.instrument.as_str())),
            ("temporality", self.temporality.as_deref()),
        ] {
            let actual = &point[field];
            let matches = expected.map_or_else(|| actual.is_null(), |v| actual == v);
            if !matches {
                return Some(format!(
                    "unexpected {surface} {field}: expected {}, got {actual}",
                    expected.unwrap_or("none")
                ));
            }
        }
        None
    }
}

/// Attribute values selecting a series, with an optional requirement for growth across the workload.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Series {
    metric: String,
    attributes: BTreeMap<String, String>,
    #[serde(default)]
    traffic: bool,
}

impl Events {
    /// Report label shared by the deployment and evidence verifier.
    #[must_use]
    pub fn resync_signal(&self) -> String {
        format!("{} recovers skipped retained events", self.lag_cursor)
    }
}

/// Metrics routes available to the deployed dashboard.
pub const METRICS: [&str; 2] = ["/api/v1/telemetry/metrics", "/api/v1/metrics"];
/// Pairs of metric sets whose endpoints must share dimension values for the UI to draw an edge.
const JOINS: [(&str, &str, &[&str], &str); 3] = [
    ("sender", "receiver", &["channel"], "Pipeline DAG"),
    (
        "controlSender",
        "controlReceiver",
        &["channel"],
        "Control channel DAG",
    ),
    (
        "publisher",
        "subscriber",
        &["process", "topic"],
        "Topic topology",
    ),
];

/// Dashboard telemetry expectations used by the deployment probe and observation replay.
pub const SIGNAL_CATALOG: &str =
    include_str!("../../../../docs/conformance/telemetry-signals.yaml");

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

impl Inputs {
    /// Load the dependency object and validate its internal references.
    pub fn parse(source: &str) -> Result<Self> {
        let inputs: Self =
            serde_yaml::from_str(source).context("invalid telemetry-signals.yaml")?;
        ensure!(!inputs.sets.is_empty(), "empty UI catalog");
        ensure!(
            inputs.events.named_logs.is_empty() || inputs.events.log_fields.contains_key("name"),
            "namedLogs requires a log name field"
        );
        ensure!(
            ["accepted", "ready"]
                .iter()
                .all(|k| inputs.events.conditions.contains_key(*k)),
            "missing status condition role"
        );
        for set in inputs.sets.values() {
            ensure!(
                set.dimensions
                    .iter()
                    .chain(set.fallback_dimensions.keys())
                    .all(|d| inputs.dimensions.contains_key(d))
                    && set
                        .series
                        .values()
                        .all(|s| set.metrics.values().any(|m| m.name == s.metric))
                    && (set.channel.is_none()
                        || ["kind", "node"]
                            .iter()
                            .all(|d| set.dimensions.iter().any(|k| k == d)))
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
        inputs.validate_aggregation()?;
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
            for (field, metric) in &set.metrics {
                let name = &metric.name;
                if let Some(platform) = set.platform.get(field)
                    && !self.platform.available(platform)?
                {
                    note(
                        &format!(".{name}"),
                        &format!("platform-inapplicable ({platform})"),
                    );
                    continue;
                }
                let fields: Vec<_> = declarations
                    .iter()
                    .filter_map(|d| d["metrics"].as_array())
                    .flatten()
                    .filter(|m| m["name"] == *name)
                    .collect();
                let points: Vec<_> = scoped
                    .iter()
                    .filter_map(|s| s["metrics"].as_array())
                    .flatten()
                    .filter(|m| m["name"] == *name)
                    .collect();
                let schema_error = fields
                    .iter()
                    .find_map(|m| metric.metadata_error(m, "schema"));
                let sample_error = points
                    .iter()
                    .find_map(|p| metric.metadata_error(p, "sample"));
                let status = if fields.is_empty() {
                    "missing from schema"
                } else if let Some(error) = schema_error.as_deref() {
                    error
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
                } else if let Some(error) = sample_error.as_deref() {
                    error
                } else if points.iter().any(|p| p["value"].as_f64().is_none()) {
                    "unusable numeric sample"
                } else {
                    "declared and observed"
                };
                note(&format!(".{name}"), status);
                for (series_name, series) in set.series.iter().filter(|(_, s)| s.metric == *name) {
                    let signal = format!("{key}.{name}/{series_name}");
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
        self.measure_aggregation(samples, path, &mut measured);
        Ok(measured)
    }

    /// Match usable dimensions and the fixture pipeline, topic or channel this consumer reads.
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
        if let Some(channel) = &set.channel {
            return get("kind") == Some(channel.kind.as_str())
                && get("node") == Some(channel.node.as_str());
        }
        match key {
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
            r.result == "declared and observed" || r.result.starts_with("platform-inapplicable")
        })
    }
}

fn attr<'a>(attrs: &'a Value, key: &str) -> Option<&'a str> {
    attrs[key]
        .as_str()
        .or_else(|| attrs[key]["String"].as_str())
}

impl Inputs {
    /// Reject a passing headline if the report omits any required observation or records a failure.
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
                let expected = if row.result.starts_with("platform-inapplicable") {
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
            "running pipeline and renderable Ready event",
            "observed",
        )?;
        require("Live logs", &self.events.resync_signal(), "observed")?;
        for state in [PENDING, UNHEALTHY] {
            require("Status overlay", state, "observed")?;
        }
        for name in self.events.conditions.values() {
            require("Status overlay", &format!("condition {name}"), "observed")?;
        }
        for field in self.events.log_fields.values() {
            require("Live logs", &format!("field {field}"), "observed")?;
        }
        for (key, kind) in &self.events.kinds {
            let pipeline = event_pipeline(key).context("missing lifecycle trigger")?;
            require("Status overlay", &format!("{pipeline} {kind}"), "observed")?;
        }
        for name in &self.events.named_logs {
            require("Live logs", name, "observed")?;
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

    const CHANNEL_CATALOG: &str = "
sets:
  probe:
    name: edge
    consumer: DAG
    metrics: {count: {name: messages, unit: '{message}', instrument: counter, temporality: delta}}
    series:
      sent:
        metric: messages
        attributes: {outcome: success}
        traffic: true
dimensions: {}
events:
  kinds: {}
  conditions: {ready: Ready, accepted: Accepted}
  pipelineStates:
    pending: &state {runningCores: 0, totalCores: 0, accepted: {status: Unknown, reason: NoPipelineRuntime}, ready: {status: Unknown, reason: NoPipelineRuntime}}
    unhealthy: *state
  lagCursor: lag_before_seq
  logFields: {}
  namedLogs: []
";

    /// Scenario: a saved report claims the UI passed with no signal observations.
    /// Guarantees: the checker requires the signal observations listed in the YAML catalog.
    #[test]
    fn passing_ui_requires_each_observation() {
        let inputs = Inputs::parse(SIGNAL_CATALOG).unwrap();
        assert!(inputs.verify_success(&UiPlatform::default(), &[]).is_err());
    }

    /// Scenario: an author changes a counter's wire name in the YAML.
    /// Guarantees: the probe requires the new declaration even when the engine still emits the old name.
    #[test]
    fn edited_yaml_drives_metric_expectations() {
        let schema = json!({"metric_sets":[{"name":"edge","metrics":[{"name":"messages","unit":"{message}","brief":"Delivered messages","instrument":"counter","temporality":"delta"}]}]});
        let data = json!({"metric_sets":[{"name":"edge","metrics":[{"name":"messages","unit":"{message}","brief":"Delivered messages","instrument":"counter","temporality":"delta","attributes":{"outcome":"success"},"value":1}]}]});
        for (wire_name, expected) in [
            ("messages", "declared and observed"),
            ("sent.messages", "missing from schema"),
        ] {
            let edited = CHANNEL_CATALOG.replace("messages", wire_name);
            let rows = Inputs::parse(&edited)
                .unwrap()
                .measure(&schema, &data, METRICS[0], None)
                .unwrap()
                .rows;
            let signal = format!("edge.{wire_name} ({})", METRICS[0]);
            assert!(
                rows.iter()
                    .any(|row| row.signal == signal && row.result == expected)
            );
        }
    }

    /// Scenario: a tooltip, unit, instrument or temporality changes on only one API surface.
    /// Guarantees: a valid declaration cannot conceal an unusable emitted point, or vice versa.
    #[test]
    fn schema_and_sample_metadata_are_independent() {
        let inputs = Inputs::parse(CHANNEL_CATALOG).unwrap();
        let point = json!({"name":"messages","brief":"Messages","unit":"{message}","instrument":"counter",
            "temporality":"delta","value":1,"attributes":{"outcome":"success"}});
        let response = |point: &Value| json!({"metric_sets":[{"name":"edge","metrics":[point]}]});
        let good = response(&point);
        for (field, wrong) in [
            ("brief", " "),
            ("unit", "s"),
            ("instrument", "gauge"),
            ("temporality", "cumulative"),
        ] {
            let mut changed = point.clone();
            changed[field] = json!(wrong);
            let bad = response(&changed);
            for (schema, data, surface) in [(&bad, &good, "schema"), (&good, &bad, "sample")] {
                let observed = inputs.measure(schema, data, METRICS[0], None).unwrap();
                assert!(!observed.passed());
                assert!(observed.rows[0].result.contains(surface));
            }
        }
    }

    /// Scenario: data endpoints still join while a control receiver is absent or advertises a different channel.
    /// Guarantees: a data edge cannot satisfy the control DAG observation, and both control ends must share an ID.
    #[test]
    fn control_edge_needs_two_matching_control_endpoints() {
        let inputs = Inputs::parse(SIGNAL_CATALOG).unwrap();
        let endpoint = |role: &str, id: &str| {
            let set = &inputs.sets[role];
            let scope = set.channel.as_ref().unwrap();
            let mut attrs = json!({});
            for d in &set.dimensions {
                attrs[&inputs.dimensions[d]] = json!("fixture");
            }
            attrs["pipeline.group.id"] = json!("default");
            attrs["pipeline.id"] = json!("main");
            attrs["node.id"] = json!(scope.node);
            attrs["channel.kind"] = json!(scope.kind);
            attrs["channel.id"] = json!(id);
            json!({"name":set.name, "attributes":attrs, "metrics":[]})
        };
        let schema = json!({"metric_sets":[]});
        for (receiver_id, expected) in [
            (None, false),
            (Some("wrong"), false),
            (Some("control"), true),
        ] {
            let mut sets = vec![
                endpoint("sender", "data"),
                endpoint("receiver", "data"),
                endpoint("controlSender", "control"),
            ];
            if let Some(id) = receiver_id {
                sets.push(endpoint("controlReceiver", id));
            }
            let rows = inputs
                .measure(&schema, &json!({"metric_sets":sets}), "metrics", None)
                .unwrap()
                .rows;
            let joined = |consumer| {
                rows.iter()
                    .any(|r| r.consumer == consumer && r.result == "declared and observed")
            };
            assert!(joined("Pipeline DAG"));
            assert_eq!(joined("Control channel DAG"), expected);
        }
    }

    /// Scenario: a catalog author misspells the traffic rule.
    /// Guarantees: an unknown key cannot silently disable the required traffic observation.
    #[test]
    fn unknown_expectation_is_rejected() {
        assert!(Inputs::parse(&CHANNEL_CATALOG.replace("traffic: true", "trafic: true")).is_err());
    }

    /// Scenario: a channel metric is undeclared, unpublished, mislabelled, legacy-only, or stationary.
    /// Guarantees: each failure is distinct and only measured traffic earns a passing observation.
    #[test]
    fn deployment_signal_failures() {
        let inputs = Inputs::parse(CHANNEL_CATALOG).unwrap();
        let schema = json!({"metric_sets":[{"name":"edge","metrics":[{"name":"messages","unit":"{message}","brief":"Delivered messages","instrument":"counter","temporality":"delta"}]}]});
        let data = |name, label, value| json!({"metric_sets":[{"name":"edge","metrics":[{"name":name,"unit":"{message}","brief":"Delivered messages","instrument":"counter","temporality":"delta","attributes":{"outcome":label},"value":value}]}]});
        let before = BTreeMap::from([("probe.messages/sent".to_owned(), 1.0)]);
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
