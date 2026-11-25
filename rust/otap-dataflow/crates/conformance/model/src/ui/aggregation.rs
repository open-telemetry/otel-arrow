// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Check the inputs to the dashboard's all-core sum and mean on the two reader instances.
use super::{Inputs, Measurements, attr};
use crate::UiSignal;
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

/// Expected per-core reading and combined value for one metric on the two-core reader.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(super) struct AllCores {
    set: String,
    metric: String,
    per_core: u64,
    expected: u64,
}

// Normalize the tagged attribute values just as the selector does.
fn label(value: &Value) -> Option<String> {
    let value = if let Some(map) = value.as_object() {
        if map.len() != 1 {
            return None;
        }
        map.values().next()?
    } else {
        value
    };
    if let Some(s) = value.as_str() {
        Some(s.to_owned())
    } else if value.is_number() || value.is_boolean() {
        Some(value.to_string())
    } else {
        None
    }
}

impl Inputs {
    pub(super) fn validate_aggregation(&self) -> Result<()> {
        for rule in &self.all_cores {
            ensure!(
                self.sets
                    .get(&rule.set)
                    .is_some_and(|s| s.metrics.contains_key(&rule.metric))
                    && ["group", "pipeline", "core", "numa", "thread"]
                        .iter()
                        .all(|d| self.dimensions.contains_key(*d)),
                "unknown allCores input: {} / {}",
                rule.set,
                rule.metric
            );
        }
        Ok(())
    }

    /// Require one reading per core in a single UI grouping. Compare the configured
    /// aggregate using a mean for gauges without temporality and a sum otherwise.
    pub(super) fn measure_aggregation(
        &self,
        samples: &[Value],
        path: &str,
        measured: &mut Measurements,
    ) {
        for rule in &self.all_cores {
            let set = &self.sets[&rule.set];
            let metric = &set.metrics[&rule.metric];
            let result = (|| -> Result<()> {
                let mut cores = BTreeSet::new();
                let mut groups = BTreeSet::new();
                let mut total = 0u64;
                for sample in samples.iter().filter(|s| s["name"] == set.name) {
                    let attrs = &sample["attributes"];
                    let dimension = |key: &str| &attrs[&self.dimensions[key]];
                    if attr(attrs, &self.dimensions["pipeline"]) != Some("reader")
                        || attr(attrs, &self.dimensions["group"]) != Some("default")
                        || set.channel.as_ref().is_some_and(|c| {
                            attr(attrs, &self.dimensions["kind"]) != Some(&c.kind)
                                || attr(attrs, &self.dimensions["node"]) != Some(&c.node)
                        })
                    {
                        continue;
                    }
                    for point in sample["metrics"]
                        .as_array()
                        .into_iter()
                        .flatten()
                        .filter(|p| p["name"] == metric.name)
                    {
                        if let Some(error) = metric.metadata_error(point, "sample") {
                            anyhow::bail!("{error}");
                        }
                        let value = point["value"]
                            .as_u64()
                            .context("missing numeric per-core value")?;
                        ensure!(
                            value == rule.per_core,
                            "expected {} per core, got {value}",
                            rule.per_core
                        );
                        let core = label(dimension("core"))
                            .context("missing core ID")?
                            .parse::<u64>()
                            .context("invalid core ID")?;
                        ensure!(cores.insert(core), "duplicate core sample");
                        let mut group = BTreeMap::new();
                        for (key, value) in
                            attrs.as_object().context("missing metric set attributes")?
                        {
                            if ["core", "numa", "thread"]
                                .iter()
                                .any(|d| self.dimensions[*d] == *key)
                            {
                                continue;
                            }
                            let _ = group.insert(
                                key,
                                label(value).ok_or_else(|| {
                                    anyhow::anyhow!("unusable grouping dimension {key}")
                                })?,
                            );
                        }
                        let _ = groups.insert(group);
                        total = total
                            .checked_add(value)
                            .ok_or_else(|| anyhow::anyhow!("aggregate overflow"))?;
                    }
                }
                ensure!(
                    cores.len() == 2 && groups.len() == 1,
                    "expected two distinct reader cores in one group, got {} cores and {} groups",
                    cores.len(),
                    groups.len()
                );
                let expected_total = if metric.instrument == "gauge" && metric.temporality.is_none()
                {
                    rule.expected.checked_mul(cores.len() as u64)
                } else {
                    Some(rule.expected)
                };
                ensure!(
                    Some(total) == expected_total,
                    "unexpected all-core aggregate"
                );
                Ok(())
            })();
            measured.rows.push(UiSignal {
                consumer: "All-core selector".into(),
                signal: format!(
                    "reader {}.{} across two cores ({path})",
                    set.name, metric.name
                ),
                result: result.map_or_else(|e| e.to_string(), |()| "declared and observed".into()),
            });
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: similarly named runtime samples belong to distinct reader cores, duplicate cores, or distinct generations.
    /// Guarantees: only two distinct cores in the same UI aggregation group with the expected worker values supply evidence.
    #[test]
    fn all_cores_requires_two_compatible_instances() {
        let inputs = Inputs::parse(super::super::SIGNAL_CATALOG).unwrap();
        let sample = |core, generation, workers| {
            json!({
            "name":"tokio.runtime", "attributes":{"pipeline.group.id":{"String":"default"},
            "pipeline.id":{"String":"reader"},"core.id":{"UInt":core},"thread.id":{"UInt":core},
            "numa.node.id":{"UInt":core}, "deployment.generation":generation},
            "metrics":[{"name":"worker.count","brief":"Workers","instrument":"up_down_counter",
            "temporality":"cumulative","unit":"{thread}","value":workers}]})
        };
        let first = sample(2, 1, 1);
        let evidence = |samples: &[Value]| {
            let mut measured = Measurements::default();
            inputs.measure_aggregation(samples, "metrics", &mut measured);
            measured
                .rows
                .into_iter()
                .find(|r| r.signal.contains("worker.count"))
                .unwrap()
                .result
        };
        assert_eq!(
            evidence(&[first.clone(), sample(3, 1, 1)]),
            "declared and observed"
        );
        for other in [sample(2, 1, 1), sample(3, 2, 1), sample(3, 1, 2)] {
            assert_ne!(evidence(&[first.clone(), other]), "declared and observed");
        }
    }
}
