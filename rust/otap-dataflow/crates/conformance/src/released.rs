// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validate pinned release inputs with the candidate engine before running a pipeline.
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_conformance_model::releases::{self, FIXTURES, Outcome, Results};
use otel_arrow_dfe_validation::deployment::Engine;
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::{fs, path::Path};

/// Validate each historical pipeline containing the selected URN, or every pipeline for a full run.
/// Keep other fixtures' results if one fails. Return false when this component has no fixture.
pub async fn run(
    engine: &Engine,
    component: Option<&str>,
    observations: &mut Results,
) -> Result<bool> {
    let root = Path::new(FIXTURES);
    let corpus = releases::corpus(Path::new("."))?;
    let migrations = releases::migrations(Path::new("."), &corpus)?;
    let selected: BTreeSet<_> = corpus
        .iter()
        .filter(|(_, urns)| component.is_none_or(|urn| urns.contains(urn)))
        .map(|(path, _)| path.as_str())
        .collect();
    let mut required = selected.clone();
    for key in &selected {
        if let Some(m) = migrations.get(*key) {
            let _ = required.insert(&m.replacement);
        }
    }
    let mut failures = BTreeMap::new();
    for key in required {
        let diagnostic = migrations.get(key).map(|m| m.diagnostic.as_str());
        if let Err(e) = engine.validate(&root.join(key), diagnostic).await {
            let _ = failures.insert(key, format!("{e:#}"));
        }
    }
    for key in selected {
        let migration = migrations.get(key);
        let outcome = match failures
            .get(key)
            .or_else(|| migration.and_then(|m| failures.get(m.replacement.as_str())))
        {
            Some(reason) => Outcome::Failed {
                reason: reason.clone(),
            },
            None => match migration {
                Some(m) => Outcome::Migrated {
                    replacement: m.replacement.clone(),
                },
                None => Outcome::Accepted,
            },
        };
        let _ = observations.insert(key.to_owned(), outcome);
    }
    ensure!(
        failures.is_empty(),
        "released configuration failed:\n{}",
        failures
            .iter()
            .map(|(path, reason)| format!("{path}: {reason}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    if component.is_none() {
        // Generate the JSON once, from an accepted YAML input. Follow its replacement if retired.
        let key = "v0.60.0/syslog-console.yaml";
        let source = migrations.get(key).map_or(key, |m| &m.replacement);
        json_smoke(engine, &root.join(source)).await?;
    }
    Ok(!observations.is_empty())
}

/// Exercise the JSON entry point, then require a diagnostic for a deliberately broken edge.
async fn json_smoke(engine: &Engine, source: &Path) -> Result<()> {
    let mut graph: Value = serde_yaml::from_slice(&fs::read(source)?)?;
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("pipeline.json");
    fs::write(&path, serde_json::to_vec(&graph)?)?;
    engine
        .validate(&path, None)
        .await
        .context("generated JSON configuration")?;
    let edge = graph["groups"]
        .as_object_mut()
        .into_iter()
        .flat_map(|g| g.values_mut())
        .filter_map(|g| g["pipelines"].as_object_mut())
        .flat_map(|p| p.values_mut())
        .filter_map(|p| p["connections"].as_array_mut())
        .find_map(|c| c.first_mut())
        .context("the JSON smoke fixture needs a pipeline connection")?;
    edge["to"] = json!("missing-node");
    fs::write(&path, serde_json::to_vec(&graph)?)?;
    engine
        .validate(&path, Some("missing-node"))
        .await
        .context("invalid JSON graph")
}
