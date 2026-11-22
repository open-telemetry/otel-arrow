// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Candidate engine scenarios used by cargo xtask conformance.
#![allow(clippy::print_stderr)]
mod dashboard;
#[path = "../../tests/common/factory_validators.rs"]
mod factory_validators;
mod fixtures;
mod graph;
mod live;
mod plan;
mod released;
mod tls;
mod ui_readers;
#[cfg(feature = "offline-fixtures")]
use otel_arrow_dfe_wasm_host as _;

use anyhow::{Result, ensure};
use otel_arrow_dfe_contract_model::catalog as contracts;
use otel_arrow_dfe_validation::deployment::Engine;
use plan::ExecutionPlan;
use std::collections::BTreeSet;
use std::path::PathBuf;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [mode, target, output, revision, options @ ..] = args.as_slice() else {
        anyhow::bail!(
            "usage: guarantee_scenarios (--binary PATH | --image IMAGE) OUTPUT.json REVISION [--component FULL_URN] [--with PREREQUISITE]... [--ui-image IMAGE]"
        );
    };
    let engine = match mode.as_str() {
        "--binary" => Engine::Binary(PathBuf::from(target).canonicalize()?),
        "--image" => Engine::Image(target.clone()),
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
            ui_engine = Some(Engine::Image(name.clone()));
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
    ensure!(
        selected.is_none() || ui_engine.is_none(),
        "--ui-image requires the full suite"
    );
    let plan = ExecutionPlan::new(selected, &contracts, &catalog, &enabled)?;
    let observed = plan.run(&engine, ui_engine.as_ref(), revision).await;
    if let Some(urn) = selected {
        let report = plan.component_report(urn, observed);
        std::fs::write(output, serde_json::to_vec_pretty(&report)?)?;
        ensure!(
            report.finding.is_none() && report.behavior_checks.values().all(|p| *p),
            "component observations incomplete for {urn}"
        );
    } else {
        std::fs::write(output, serde_json::to_vec_pretty(&observed)?)?;
        ensure!(
            observed.checks.values().all(|&passed| passed),
            "one or more guarantee scenarios failed"
        );
    }
    Ok(())
}
