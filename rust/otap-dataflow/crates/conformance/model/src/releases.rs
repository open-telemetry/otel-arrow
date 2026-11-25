// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Index pinned inputs by the components they declare and exchange per-file preflight results.
use anyhow::{Context, Result, ensure};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::{fs, path::Path};

/// Workspace-relative root of the immutable release inputs.
pub const FIXTURES: &str = "crates/conformance/fixtures/releases";

/// Outcome of running the candidate against one whole configuration.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "status", rename_all = "kebab-case")]
pub enum Outcome {
    /// The engine accepted the released configuration.
    Accepted,
    /// The engine returned the migration diagnostic and accepted the replacement.
    Migrated {
        /// Release-relative path to the accepted input.
        replacement: String,
    },
    /// Validation did not produce the expected outcome.
    Failed {
        /// Diagnostic from the deployed candidate or the probe.
        reason: String,
    },
}

impl Outcome {
    /// Short label used by the terminal report.
    #[must_use]
    pub const fn label(&self) -> &'static str {
        match self {
            Self::Accepted => "accepted",
            Self::Migrated { .. } => "migrated",
            Self::Failed { .. } => "failed",
        }
    }
}

/// Observations use release-relative paths, including when a fixture contains several nodes.
pub type Results = BTreeMap<String, Outcome>;
/// Components explicitly declared in each historical input, independent of candidate acceptance.
pub type Corpus = BTreeMap<String, BTreeSet<String>>;

fn values(map: &Value) -> impl Iterator<Item = &Value> {
    map.as_object().into_iter().flat_map(|m| m.values())
}

// Read only the graph structure. A change to a node's configuration schema must still
// let the driver identify its old fixture and report the candidate's actual diagnostic.
fn components(doc: &Value) -> Result<BTreeSet<String>> {
    let pipelines = values(&doc["groups"])
        .flat_map(|g| values(&g["pipelines"]))
        .chain(doc.pointer("/engine/observability/pipeline"));
    let declarations = pipelines
        .flat_map(|p| values(&p["nodes"]).chain(values(&p["extensions"])))
        .chain(values(&doc["engine"]["controller"]["extensions"]));
    let mut urns = BTreeSet::new();
    for entry in declarations {
        let name = entry["type"]
            .as_str()
            .context("component declaration missing type")?
            .trim();
        let urn = if name.split(':').count() == 2 {
            format!("urn:otel:{name}")
        } else {
            name.to_owned()
        };
        ensure!(
            urn.to_ascii_lowercase().starts_with("urn:") && urn.split(':').count() == 4,
            "invalid component type {name}"
        );
        let _ = urns.insert(urn.to_ascii_lowercase());
    }
    Ok(urns)
}

/// Discover fixture files without consulting Git or contacting a remote release.
pub fn corpus(workspace: &Path) -> Result<Corpus> {
    let root = workspace.join(FIXTURES);
    let mut corpus = Corpus::new();
    for release in fs::read_dir(&root)? {
        for file in fs::read_dir(release?.path())? {
            let path = file?.path();
            ensure!(
                matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("yaml" | "yml" | "json")
                ),
                "unexpected release fixture {}",
                path.display()
            );
            let key = path
                .strip_prefix(&root)?
                .to_str()
                .context("non-UTF8 fixture name")?
                .replace('\\', "/");
            let document: Value = serde_yaml::from_slice(&fs::read(&path)?)
                .with_context(|| format!("reading {key}"))?;
            let _ = corpus.insert(
                key.clone(),
                components(&document).with_context(|| format!("indexing {key}"))?,
            );
        }
    }
    ensure!(!corpus.is_empty(), "empty release corpus");
    Ok(corpus)
}

/// A retired input must yield this diagnostic and have an accepted, non-retired replacement.
#[derive(Deserialize)]
pub struct Migration {
    /// Key of the replacement in the same corpus.
    pub replacement: String,
    /// Text the engine must include when it rejects the retired configuration.
    pub diagnostic: String,
}

/// Check the manifest against real fixture names before scheduling or replaying observations.
pub fn migrations(workspace: &Path, corpus: &Corpus) -> Result<BTreeMap<String, Migration>> {
    let path = workspace.join("docs/conformance/release-migrations.json");
    if !path.try_exists()? {
        return Ok(BTreeMap::new());
    }
    let entries: BTreeMap<String, Migration> = serde_json::from_slice(&fs::read(path)?)?;
    for (key, m) in &entries {
        ensure!(
            corpus.contains_key(key)
                && corpus.contains_key(&m.replacement)
                && !entries.contains_key(&m.replacement)
                && !m.diagnostic.trim().is_empty(),
            "invalid replacement or diagnostic for {key}"
        );
    }
    Ok(entries)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: a downstream deployment upgrades with configurations taken from supported release examples.
    /// Guarantees: each requested component, including both OTLP exporter transports, occurs in a pinned graph.
    #[test]
    fn downstream_nodes_have_released_inputs() {
        let workspace = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let corpus = corpus(&workspace).unwrap();
        let nodes: BTreeSet<_> = corpus.values().flatten().cloned().collect();
        for node in [
            "receiver:otlp",
            "receiver:syslog_cef",
            "processor:batch",
            "processor:temporal_reaggregation",
            "processor:type_router",
            "processor:content_router",
            "processor:retry",
            "processor:attribute",
            "processor:filter",
            "processor:transform",
            "exporter:otlp_grpc",
            "exporter:otlp_http",
            "receiver:otap",
            "exporter:otap",
            "receiver:kafka",
            "exporter:kafka",
        ] {
            assert!(
                nodes.contains(&format!("urn:otel:{node}")),
                "missing released configuration for {node}"
            );
        }
    }

    /// Scenario: a node configuration contains an unrelated type string alongside actual graph declarations.
    /// Guarantees: only the explicit node and extension URNs receive credit, including non-OTel namespaces.
    #[test]
    fn index_uses_declarations_only() {
        let graph = json!({"groups":{"g":{"pipelines":{"p":{"nodes":{
            "n":{"type":"receiver:otlp","config":{"type":"exporter:kafka"}}},
            "extensions":{"e":{"type":"urn:vendor:extension:auth"}}}}}},
            "engine":{"controller":{"extensions":{"o":{"type":"extension:opamp"}}}}});
        assert_eq!(
            components(&graph).unwrap(),
            [
                "urn:otel:receiver:otlp",
                "urn:vendor:extension:auth",
                "urn:otel:extension:opamp"
            ]
            .map(str::to_owned)
            .into()
        );
    }
}
