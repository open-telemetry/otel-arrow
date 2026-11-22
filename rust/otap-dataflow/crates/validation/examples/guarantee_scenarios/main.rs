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
mod ui_readers;
#[cfg(feature = "offline-fixtures")]
use otel_arrow_dfe_wasm_host as _;

use anyhow::{Context, Result, ensure};
use futures_util::{StreamExt, stream};
use otel_arrow_dfe_contract_model::{
    Checks, ComponentReport, Observations, Rollout, Scenario, catalog as contracts,
};
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::time::Duration;
use tokio::time::timeout;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [mode, target, output, revision, options @ ..] = args.as_slice() else {
        anyhow::bail!(
            "usage: guarantee_scenarios (--binary PATH | --image IMAGE) OUTPUT.json REVISION [--component FULL_URN] [--with PREREQUISITE]... [--ui-image IMAGE]"
        );
    };
    let engine = match mode.as_str() {
        "--binary" => engine::Engine::Binary(PathBuf::from(target).canonicalize()?),
        "--image" => engine::Engine::Image(target.clone()),
        _ => anyhow::bail!("unknown engine target {mode}"),
    };
    let catalog = fixtures::catalog()?;
    let contracts = contracts()?;
    let mut selected = None;
    let mut ui_engine = None;
    let mut enabled = BTreeSet::new();
    if cfg!(target_os = "linux") {
        let _ = enabled.insert("linux".to_owned());
    }
    for pair in options.chunks(2) {
        let [flag, name] = pair else {
            anyhow::bail!("expected --with PREREQUISITE");
        };
        if flag == "--ui-image" {
            ensure!(
                mode == "--binary" && ui_engine.is_none(),
                "--ui-image requires a native fixture suite"
            );
            ui_engine = Some(engine::Engine::Image(name.clone()));
            continue;
        }
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
        ensure!(ui_engine.is_none(), "--ui-image requires the full suite");
        let report = component(
            &engine,
            urn,
            linked.get(urn),
            catalog.get(urn),
            &enabled,
            revision,
            &contracts,
        )
        .await;
        std::fs::write(output, serde_json::to_vec_pretty(&report)?)?;
        ensure!(
            report.finding.is_none() && report.behavior_checks.values().all(|p| *p),
            "component observations incomplete for {urn}"
        );
        return Ok(());
    }
    let mut checks = BTreeMap::new();
    let mut check_errors = BTreeMap::new();
    let mut rollouts = BTreeMap::new();
    let mut ui_signals = Vec::new();
    let mut groups = BTreeMap::from([
        (Scenario::LiveRollout, true),
        (Scenario::ExtensionRollout, true),
    ]);
    let rollout_roles: BTreeMap<_, _> = contracts
        .values()
        .filter_map(|c| {
            let scenario = c.scenario?;
            matches!(scenario, Scenario::LiveRollout | Scenario::ExtensionRollout)
                .then_some((c, scenario))
        })
        .flat_map(|(c, scenario)| c.roles.iter().map(move |role| (role.as_str(), scenario)))
        .collect();
    let eligible = linked.iter().filter_map(|(urn, registration)| {
        let fixture = catalog.get(*urn)?;
        fixture
            .requires
            .iter()
            .all(|name| enabled.contains(name))
            .then_some((*urn, registration, fixture))
    });
    let engine_ref = &engine;
    let rollout_roles = &rollout_roles;
    let mut runs = stream::iter(eligible)
        .map(|(urn, registration, fixture)| async move {
            let group = rollout_roles.get(registration.role).copied();
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
        let group = group.context("no catalog rollout check for linked fixture role")?;
        if let Some(reason) = &error {
            let _ = groups.insert(group, false);
            eprintln!("FAIL {urn}: {reason}");
        }
        let _ = rollouts.insert(urn.to_owned(), Rollout { operations, error });
    }
    drop(runs);
    // The default suite promises both historical scenarios in every build it runs against.
    for (urn, group) in otel_arrow_dfe_contract_model::REQUIRED_ROLLOUTS {
        if !rollouts.contains_key(urn) {
            eprintln!("Missing required fixture or linked factory: {urn}");
            let _ = groups.insert(group, false);
        }
    }
    for (id, contract) in &contracts {
        let Some(scenario) = contract.scenario else {
            continue;
        };
        if !contract.selected(None) {
            continue;
        }
        let result = match scenario {
            Scenario::FactoryValidation => (|| {
                let validators: Vec<_> = linked
                    .iter()
                    .filter_map(|(id, v)| (!v.rejected).then_some(*id))
                    .collect();
                let required = [
                    "receiver",
                    "processor",
                    "exporter",
                    "extension",
                    "controller-extension",
                ];
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
            Scenario::LiveRollout | Scenario::ExtensionRollout => {
                if groups[&scenario] {
                    Ok(())
                } else {
                    Err(anyhow::anyhow!("see component operation results"))
                }
            }
            Scenario::EmbeddedUi => timeout(
                Duration::from_secs(35),
                live::embedded_ui(ui_engine.as_ref().unwrap_or(&engine), &mut ui_signals),
            )
            .await
            .context("embedded UI scenario timed out")
            .and_then(|r| r),
            other => engine_probe(other, &engine).await,
        };
        record(&mut checks, &mut check_errors, id, result);
    }
    let report = Observations {
        ui_platform: Default::default(),
        ui_signals,
        commit: revision.clone(),
        checks,
        check_errors,
        rollouts,
        prerequisites: enabled,
        linked_validators: linked
            .into_iter()
            .map(|(id, v)| (id.to_owned(), v.rejected))
            .collect(),
    };
    std::fs::write(output, serde_json::to_vec_pretty(&report)?)?;
    ensure!(
        report.checks.values().all(|&passed| passed),
        "one or more guarantee scenarios failed"
    );
    Ok(())
}

async fn engine_probe(scenario: Scenario, engine: &engine::Engine) -> Result<()> {
    match scenario {
        Scenario::ReleasedInputs => released::run(engine).await,
        Scenario::ReconfigurationConsistency => timeout(
            Duration::from_secs(45),
            live::reconfiguration_consistency(engine),
        )
        .await
        .context("reconfiguration consistency timed out")
        .and_then(|r| r),
        Scenario::FiniteIdDrain => live::finite_id_drain(engine).await,
        Scenario::RejectedUpdate => timeout(Duration::from_secs(60), live::rejected_update(engine))
            .await
            .context("rejected-update scenario timed out")
            .and_then(|r| r),
        _ => anyhow::bail!("{scenario:?} requires a component fixture or factory"),
    }
}

fn record(
    checks: &mut Checks,
    errors: &mut BTreeMap<String, String>,
    id: &str,
    result: Result<()>,
) {
    let _ = checks.insert(id.to_owned(), result.is_ok());
    match result {
        Ok(()) => eprintln!("PASS {id}"),
        Err(error) => {
            let _ = errors.insert(id.to_owned(), format!("{error:#}"));
            eprintln!("FAIL {id}: {error:#}");
        }
    }
}

async fn component(
    engine: &engine::Engine,
    urn: &str,
    validator: Option<&factory_validators::Observation>,
    fixture: Option<&fixtures::Fixture>,
    enabled: &BTreeSet<String>,
    revision: &str,
    contracts: &otel_arrow_dfe_contract_model::Catalog,
) -> ComponentReport {
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
    for (id, contract) in contracts {
        if !validator.is_some_and(|v| contract.selected(Some((urn, v.role)))) {
            continue;
        }
        match contract.scenario {
            Some(Scenario::ReconfigurationNoop) => {
                if let Some(passed) = noop {
                    let _ = checks.insert(id.clone(), passed);
                } else {
                    unexercised.push(id.clone());
                }
            }
            Some(
                Scenario::LiveRollout | Scenario::ExtensionRollout | Scenario::FactoryValidation,
            ) => (), // Reported per operation and factory.
            Some(scenario) => {
                record(
                    &mut checks,
                    &mut errors,
                    id,
                    engine_probe(scenario, engine).await,
                );
            }
            _ => (),
        }
    }
    let role = if startup_only {
        "controller-extension"
    } else {
        urn.split(':').nth(2).unwrap_or_default()
    };
    let deployment_gaps = contracts
        .iter()
        .filter(|(_, c)| c.selected(Some((urn, role))))
        .filter_map(|(id, c)| Some((id.clone(), c.gap.clone()?)))
        .collect();
    ComponentReport {
        deployment_gaps,
        component: urn.to_owned(),
        commit: revision.to_owned(),
        factory_validation: validator_result,
        operations: (!startup_only).then_some(operations),
        finding: error,
        behavior_checks: checks,
        check_errors: errors,
        unexercised_checks: unexercised,
    }
}
