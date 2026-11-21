// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::engine::Engine;
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use serde_json::{Value, json};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Deserialize)]
struct Migration {
    replacement: String,
    diagnostic: String,
}

pub async fn run(engine: &Engine) -> Result<()> {
    let fixtures = Path::new("crates/config/tests/fixtures/releases");
    let manifest = Path::new("docs/guarantees/release-migrations.json");
    let migrations: BTreeMap<String, Migration> = if manifest.try_exists()? {
        serde_json::from_slice(&std::fs::read(manifest)?)?
    } else {
        BTreeMap::new()
    };
    let mut inputs = BTreeMap::new();
    for release in std::fs::read_dir(fixtures)? {
        for entry in std::fs::read_dir(release?.path())? {
            let path = entry?.path();
            ensure!(path.is_file(), "unexpected fixture {}", path.display());
            let key = path
                .strip_prefix(fixtures)?
                .to_string_lossy()
                .replace('\\', "/");
            let _ = inputs.insert(key, path);
        }
    }
    ensure!(!inputs.is_empty(), "empty release fixture directory");
    for (key, migration) in &migrations {
        ensure!(
            inputs.contains_key(key)
                && inputs.contains_key(&migration.replacement)
                && !migrations.contains_key(&migration.replacement)
                && !migration.diagnostic.trim().is_empty(),
            "invalid replacement or diagnostic for {key}"
        );
    }
    for (key, path) in &inputs {
        engine
            .validate(path, migrations.get(key).map(|m| m.diagnostic.as_str()))
            .await?;
    }
    let mut invalid: Value = serde_json::from_slice(&std::fs::read(
        inputs
            .get("v0.60.0/syslog-console.json")
            .context("missing pinned JSON fixture")?,
    )?)?;
    invalid["groups"]["default"]["pipelines"]["main"]["connections"][0]["to"] =
        json!("missing-node");
    let temp = tempfile::tempdir()?;
    let path = temp.path().join("invalid.json");
    std::fs::write(&path, serde_json::to_vec(&invalid)?)?;
    engine.validate(&path, Some("missing-node")).await
}
