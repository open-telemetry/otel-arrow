// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Candidate engine scenarios used by cargo xtask check-contracts.
#![allow(clippy::print_stderr)]
mod dashboard;
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
            "usage: guarantee_scenarios (--binary PATH | --image IMAGE) OUTPUT.json REVISION [--component FULL_URN] [--with PREREQUISITE]..."
        );
    };
    let engine = match mode.as_str() {
        "--binary" => engine::Engine::Binary(PathBuf::from(target).canonicalize()?),
        "--image" => engine::Engine::Image(target.clone()),
        _ => anyhow::bail!("unknown engine target {mode}"),
    };
    let catalog = fixtures::catalog()?;
    let mut selected = None;
    let mut enabled = BTreeSet::new();
    if cfg!(target_os = "linux") {
        let _ = enabled.insert("linux".to_owned());
    }
    for pair in options.chunks(2) {
        let [flag, name] = pair else {
            anyhow::bail!("expected --with PREREQUISITE");
        };
        if flag == "--component" {
            ensure!(
                selected.replace(name.as_str()).is_none(),
                "select one component"
            );
            continue;
        }
        ensure!(
            flag == "--with" && catalog.values().any(|f| f.requires.contains(name)),
            "unknown fixture prerequisite {pair:?}"
        );
        let _ = enabled.insert(name.clone());
    }
    let linked = factory_validators::observations(selected);
    if let Some(urn) = selected {
        return component(
            &engine,
            urn,
            linked.get(urn),
            catalog.get(urn),
            &enabled,
            output,
            revision,
        )
        .await;
    }
    let mut checks = BTreeMap::new();
    let mut check_errors = BTreeMap::new();
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
        &mut check_errors,
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
    record(
        &mut checks,
        &mut check_errors,
        "released-inputs",
        released::run(&engine).await,
    );
    record(
        &mut checks,
        &mut check_errors,
        "embedded-ui",
        timeout(Duration::from_secs(25), live::embedded_ui(&engine))
            .await
            .context("embedded UI scenario timed out")
            .and_then(|r| r),
    );
    record(
        &mut checks,
        &mut check_errors,
        "reconfiguration-consistency",
        timeout(
            Duration::from_secs(45),
            live::reconfiguration_consistency(&engine),
        )
        .await
        .context("reconfiguration consistency timed out")
        .and_then(|r| r),
    );
    record(
        &mut checks,
        &mut check_errors,
        "finite-id-drain",
        live::finite_id_drain(&engine).await,
    );
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
                live::exercise(engine_ref, fixture, &mut operations, &mut None).await
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
    record(&mut checks, &mut check_errors, "rejected-update", rejection);
    for (group, passed) in groups {
        record(
            &mut checks,
            &mut check_errors,
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
            "commit": revision, "checks": checks, "check_errors":check_errors, "linked_validators": validators,
            "rollouts": rollouts, "prerequisites": enabled,
        }))?,
    )?;
    ensure!(
        checks.values().all(|&passed| passed),
        "one or more guarantee scenarios failed"
    );
    Ok(())
}

fn record(
    checks: &mut BTreeMap<&str, bool>,
    errors: &mut BTreeMap<&str, String>,
    id: &'static str,
    result: Result<()>,
) {
    let _ = checks.insert(id, result.is_ok());
    match result {
        Ok(()) => eprintln!("PASS {id}"),
        Err(error) => {
            let _ = errors.insert(id, format!("{error:#}"));
            eprintln!("FAIL {id}: {error:#}");
        }
    }
}

#[allow(clippy::print_stdout)]
async fn component(
    engine: &engine::Engine,
    urn: &str,
    validator: Option<&factory_validators::Observation>,
    fixture: Option<&fixtures::Fixture>,
    enabled: &BTreeSet<String>,
    output: &str,
    revision: &str,
) -> Result<()> {
    let mut operations = live::pending();
    let mut noop = None;
    let result = async {
        let registration = validator.context("not linked in this build")?;
        ensure!(
            registration.rejected,
            "factory accepted invalid configuration"
        );
        if registration.role == "controller-extension" {
            return Ok(());
        }
        let fixture = fixture.context("No live reconfiguration fixture")?;
        ensure!(
            registration.role == fixture.probe.role(),
            "fixture role mismatch"
        );
        for name in &fixture.requires {
            ensure!(enabled.contains(name), "unexercised: requires {name}");
        }
        live::exercise(engine, fixture, &mut operations, &mut noop).await
    }
    .await;
    let error = result.as_ref().err().map(|e| format!("{e:#}"));
    let validator_result = validator.map(|v| v.rejected);
    let startup_only = validator.is_some_and(|v| v.role == "controller-extension");
    let mut checks = BTreeMap::new();
    let mut errors = BTreeMap::new();
    let mut unexercised = Vec::new();
    if let Some(passed) = noop {
        let _ = checks.insert("reconfiguration-noop", passed);
    } else if validator.is_some() && !startup_only {
        unexercised.push("reconfiguration-noop");
        println!("reconfiguration-noop: unexercised");
    }
    if validator.is_some()
        && matches!(
            urn,
            "urn:otel:processor:batch" | "urn:otel:processor:durable_buffer"
        )
    {
        record(
            &mut checks,
            &mut errors,
            "finite-id-drain",
            live::finite_id_drain(engine).await,
        );
    }
    let report = json!({"unexercised_checks":unexercised, "behavior_checks":checks,"check_errors":errors,"component":urn, "commit":revision, "factory_validation":validator_result,
            "operations":(!startup_only).then_some(&operations), "finding":error});
    std::fs::write(output, serde_json::to_vec_pretty(&report)?)?;
    println!("Component {urn}");
    println!(
        "Factory validation: {}",
        match validator_result {
            Some(true) => "passed",
            Some(false) => "failed",
            None => "unexercised",
        }
    );
    if startup_only {
        println!("Controller extension: startup validation only");
    } else {
        for op in ["add", "change", "remove"] {
            println!(
                "{op}: {}",
                report["operations"][op]["status"]
                    .as_str()
                    .context("missing operation")?
            );
        }
    }
    result?;
    ensure!(
        checks.values().all(|pass| *pass),
        "component behavior checks failed: {errors:?}"
    );
    Ok(())
}
