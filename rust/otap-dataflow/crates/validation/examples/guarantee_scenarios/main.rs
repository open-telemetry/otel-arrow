// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Candidate engine scenarios used by cargo xtask guarantees.
#![allow(clippy::print_stderr)]
mod engine;
#[path = "../../tests/common/factory_validators.rs"]
mod factory_validators;
mod fixtures;
mod graph;
mod live;
mod released;
mod tls;
#[cfg(feature = "offline-fixtures")]
use otel_arrow_dfe_wasm_host as _;

use anyhow::{Context, Result, ensure};
use futures_util::{StreamExt, stream};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [mode, target, output, revision, options @ ..] = args.as_slice() else {
        anyhow::bail!(
            "usage: guarantee_scenarios (--binary PATH | --image IMAGE) OUTPUT.json REVISION [--with PREREQUISITE]..."
        );
    };
    let engine = match mode.as_str() {
        "--binary" => engine::Engine::Binary(PathBuf::from(target).canonicalize()?),
        "--image" => engine::Engine::Image(target.clone()),
        _ => anyhow::bail!("unknown engine target {mode}"),
    };
    let catalog = fixtures::catalog()?;
    let mut enabled = BTreeSet::new();
    if cfg!(target_os = "linux") {
        let _ = enabled.insert("linux".to_owned());
    }
    for pair in options.chunks(2) {
        let [flag, name] = pair else {
            anyhow::bail!("expected --with PREREQUISITE");
        };
        ensure!(
            flag == "--with" && catalog.values().any(|f| f.requires.contains(name)),
            "unknown fixture prerequisite {pair:?}"
        );
        let _ = enabled.insert(name.clone());
    }
    let linked = factory_validators::observations();
    let mut checks = BTreeMap::new();
    let validators = linked
        .iter()
        .filter_map(|(id, result)| (!result.rejected).then_some(*id))
        .collect::<Vec<_>>();
    let required = [
        "receiver",
        "processor",
        "exporter",
        "extension",
        "controller-extension",
    ];
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
                    linked.values().any(|result| result.role == category),
                    "empty {category} registry"
                );
            }
            Ok(())
        })(),
    );
    record(&mut checks, "released-inputs", released::run(&engine).await);
    let mut rollouts = BTreeMap::new();
    let mut groups = BTreeMap::from([("live-rollout", true), ("extension-rollout", true)]);
    let eligible = linked.iter().filter_map(|(urn, registration)| {
        let fixture = catalog.get(*urn)?;
        fixture
            .requires
            .iter()
            .all(|name| enabled.contains(name))
            .then_some((*urn, registration, fixture))
    });
    let engine_ref = &engine;
    let mut runs = stream::iter(eligible)
        .map(|(urn, registration, fixture)| async move {
            let group = if registration.role == "extension" {
                "extension-rollout"
            } else {
                "live-rollout"
            };
            let mut operations = live::pending();
            let result = async {
                ensure!(
                    registration.role == fixture.probe.role(),
                    "fixture probe incompatible with {} registry",
                    registration.role
                );
                live::exercise(engine_ref, fixture, &mut operations).await
            }
            .await;
            let error = result.err().map(|e| format!("{e:#}"));
            (urn, group, operations, error)
        })
        .buffer_unordered(3);
    while let Some((urn, group, operations, error)) = runs.next().await {
        if let Some(reason) = &error {
            let _ = groups.insert(group, false);
            eprintln!("FAIL {urn}: {reason}");
        }
        let _ = rollouts.insert(urn, json!({"operations": operations, "error": error}));
    }
    // The default suite promises both historical scenarios in every build it runs against.
    for (urn, group) in [
        ("urn:otel:receiver:syslog_cef", "live-rollout"),
        (
            "urn:otel:extension:flat_file_api_key_auth",
            "extension-rollout",
        ),
    ] {
        if !rollouts.contains_key(urn) {
            eprintln!("Missing required fixture or linked factory: {urn}");
            let _ = groups.insert(group, false);
        }
    }
    let rejection = timeout(Duration::from_secs(60), live::rejected_update(&engine))
        .await
        .context("rejected-update scenario timed out")
        .and_then(|r| r);
    record(&mut checks, "rejected-update", rejection);
    for (group, passed) in groups {
        record(
            &mut checks,
            group,
            if passed {
                Ok(())
            } else {
                Err(anyhow::anyhow!("see component operation results"))
            },
        );
    }
    let validators: BTreeMap<_, _> = linked
        .iter()
        .map(|(urn, result)| (urn, result.rejected))
        .collect();
    std::fs::write(
        output,
        serde_json::to_vec_pretty(&json!({
            "commit": revision, "checks": checks, "linked_validators": validators,
            "rollouts": rollouts, "prerequisites": enabled,
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
