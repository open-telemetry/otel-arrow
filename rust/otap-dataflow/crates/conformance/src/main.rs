// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Deployment driver for `cargo xtask conformance`.
//!
//! [`plan::ExecutionPlan`] selects contracts for the whole engine or one component URN.
//! For a component rollout, follow [`fixtures::Fixture`] from the YAML catalog into
//! [`live::exercise`], which changes a running pipeline and observes each operation.
//! Engine-wide scenarios such as [`dashboard`] and [`drain`] run their own deployments.
//! The driver writes observations as JSON. The xtask uses them for coverage and reporting.
#![allow(clippy::print_stderr)]
mod dashboard;
mod drain;
mod fixture_session;
mod fixtures;
mod graph;
mod live;
mod plan;
mod reconfiguration;
mod released;
mod tls;
// Link the production component registrations exercised by this build.
use otel_arrow_dfe_contrib_extensions as _;
use otel_arrow_dfe_contrib_nodes as _;
#[cfg(feature = "offline-fixtures")]
use otel_arrow_dfe_wasm_host as _;

use anyhow::{Result, ensure};
use otel_arrow_dfe_conformance_model::catalog as contracts;
use otel_arrow_dfe_validation::deployment::Engine;
use plan::ExecutionPlan;
use std::collections::BTreeSet;
use std::path::PathBuf;

/// Run the selected plan and write its observations before returning a failing exit status.
#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<()> {
    otel_arrow_dfe_otap::crypto::ensure_crypto_provider();
    let args: Vec<_> = std::env::args().skip(1).collect();
    let [mode, target, output, revision, options @ ..] = args.as_slice() else {
        anyhow::bail!(
            "usage: conformance (--binary PATH | --image IMAGE) OUTPUT.json REVISION [--component FULL_URN | --check embedded-ui] [--with PREREQUISITE]..."
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
    let mut ui_only = false;
    let mut enabled = BTreeSet::new();
    if cfg!(target_os = "linux") {
        let _ = enabled.insert("linux".to_owned());
    }
    for pair in options.chunks(2) {
        let [flag, name] = pair else {
            anyhow::bail!("expected --with PREREQUISITE");
        };
        if flag == "--check" {
            ensure!(
                !ui_only && name == "embedded-ui",
                "expected --check embedded-ui"
            );
            ui_only = true;
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
    let plan = if ui_only {
        ensure!(
            selected.is_none(),
            "--check selects a standalone deployment"
        );
        ExecutionPlan::embedded_ui(&contracts, &enabled)?
    } else {
        ExecutionPlan::new(selected, &contracts, &catalog, &enabled)?
    };
    let observed = plan.run(&engine, revision).await;
    if let Some(urn) = selected {
        let report = plan.component_report(urn, observed);
        std::fs::write(output, serde_json::to_vec_pretty(&report)?)?;
        ensure!(
            report.blocking_findings().is_empty(),
            "component observations incomplete for {urn}"
        );
    } else {
        std::fs::write(output, serde_json::to_vec_pretty(&observed)?)?;
        ensure!(
            observed.checks.values().all(|&passed| passed),
            "one or more conformance checks failed"
        );
    }
    Ok(())
}
