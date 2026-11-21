// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Candidate engine scenarios used by cargo xtask guarantees.
#![allow(clippy::print_stderr)]
mod engine;
#[path = "../../tests/common/factory_validators.rs"]
mod factory_validators;
mod live;
mod released;

use anyhow::{Context, Result, ensure};
use serde_json::json;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [mode, target, output, revision] = args.as_slice() else {
        anyhow::bail!(
            "usage: guarantee_scenarios (--binary PATH | --image IMAGE) OUTPUT.json REVISION"
        );
    };
    let engine = match mode.as_str() {
        "--binary" => engine::Engine::Binary(PathBuf::from(target).canonicalize()?),
        "--image" => engine::Engine::Image(target.clone()),
        _ => anyhow::bail!("unknown engine target {mode}"),
    };
    let linked = factory_validators::observations();
    let mut checks = BTreeMap::new();
    let validators = linked
        .iter()
        .filter_map(|(id, &ok)| (!ok).then_some(*id))
        .collect::<Vec<_>>();
    let required = ["receiver", "processor", "exporter", "extension"];
    record(
        &mut checks,
        "factory-validation",
        (|| {
            ensure!(
                validators.is_empty(),
                "validators accepted invalid input: {validators:?}"
            );
            for category in required {
                ensure!(
                    linked
                        .keys()
                        .any(|id| id.split(':').nth(2) == Some(category)),
                    "empty {category} registry"
                );
            }
            Ok(())
        })(),
    );
    record(&mut checks, "released-inputs", released::run(&engine).await);
    for (id, extension) in [("live-rollout", false), ("extension-rollout", true)] {
        let result =
            tokio::time::timeout(Duration::from_secs(60), live::run(&engine, extension)).await;
        record(
            &mut checks,
            id,
            result.context("scenario timed out").and_then(|r| r),
        );
    }
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "commit": revision, "checks": checks, "linked_validators": linked,
            "rollout_components": {"live-rollout": live::NODE_OPERATIONS, "extension-rollout": live::EXTENSION_OPERATIONS},
        }))?,
    )?;
    ensure!(
        checks.values().all(|&passed| passed),
        "one or more guarantee scenarios failed"
    );
    Ok(())
}

fn record(checks: &mut BTreeMap<&str, bool>, id: &'static str, result: Result<()>) {
    let _ = checks.insert(id, result.is_ok());
    match result {
        Ok(()) => eprintln!("PASS {id}"),
        Err(error) => eprintln!("FAIL {id}: {error:#}"),
    }
}
