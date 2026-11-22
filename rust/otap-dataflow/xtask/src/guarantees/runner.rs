// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Collect evidence locally or replay candidate-bound CI observations.
use super::{
    Catalog, ComponentReport, DFE, Observations, OperationOutcome, Options, coverage, git,
};
use anyhow::{Context, Result, ensure};
use serde::Deserialize;
use std::process::Stdio;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

// A failed or incomplete observation stays failed when another CI environment also ran it.
pub(super) fn merge(observed: &mut Observations, other: Observations, catalog: &Catalog) {
    for (id, _) in catalog
        .iter()
        .filter(|(_, c)| c.scenario.is_some() && !c.focused_only)
    {
        *observed.checks.entry(id.into()).or_insert(true) &= other.checks.get(id) == Some(&true);
    }
    for (urn, passed) in other.linked_validators {
        *observed.linked_validators.entry(urn).or_insert(true) &= passed;
    }
    observed.ui_signals.extend(other.ui_signals);
    observed.check_errors.extend(other.check_errors);
    observed.prerequisites.extend(other.prerequisites);
    for (urn, mut result) in other.rollouts {
        match observed.rollouts.entry(urn) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                let _ = slot.insert(result);
            }
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                let existing = slot.get_mut();
                if existing.error.is_none() {
                    existing.error = result.error;
                }
                for op in coverage::OPERATIONS {
                    let state = existing
                        .operations
                        .entry(op.into())
                        .or_insert(OperationOutcome::Unexercised);
                    let incoming = result
                        .operations
                        .remove(op)
                        .unwrap_or(OperationOutcome::Unexercised);
                    if matches!(state, OperationOutcome::Passed)
                        || matches!(incoming, OperationOutcome::Failed { .. })
                    {
                        *state = incoming;
                    }
                }
            }
        }
    }
}

fn executable(args: &[&str], name: &str) -> Result<PathBuf> {
    #[derive(Deserialize)]
    struct Artifact {
        executable: Option<PathBuf>,
        target: Option<Target>,
    }
    #[derive(Deserialize)]
    struct Target {
        name: String,
    }
    let output = Command::new(env!("CARGO"))
        .args(args)
        .args(["--locked", "--message-format=json-render-diagnostics"])
        .stderr(Stdio::inherit())
        .output()?;
    ensure!(output.status.success(), "cargo {} failed", args.join(" "));
    for line in output.stdout.split(|&c| c == b'\n') {
        if let Ok(Artifact {
            executable: Some(path),
            target: Some(target),
        }) = serde_json::from_slice(line)
            && target.name == name
        {
            return Ok(path);
        }
    }
    anyhow::bail!("Cargo produced no executable for {name}")
}

fn native_tools() -> Result<(PathBuf, PathBuf)> {
    let engine = executable(
        &[
            "build",
            "-p",
            "otel-arrow-dfe",
            "--bin",
            "df_engine",
            "--features",
            "offline-fixtures",
        ],
        "df_engine",
    )?;
    let driver = executable(
        &[
            "build",
            "-p",
            "otel-arrow-dfe-validation",
            "--example",
            "guarantee_scenarios",
            "--features",
            "crypto-ring,offline-fixtures",
        ],
        "guarantee_scenarios",
    )?;
    Ok((engine, driver))
}

pub(super) fn component(
    root: &Path,
    sha: &str,
    urn: &str,
    opt: &Options,
) -> Result<(ComponentReport, bool)> {
    let (components, _) = crate::component_inventory::guarantee_inventory(&root.join(DFE))?;
    let registered = coverage::registered(components)?;
    ensure!(registered.contains_key(urn), "unknown component URN: {urn}");
    let (engine, driver) = native_tools()?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("component.json");
    let mut cmd = Command::new(driver);
    let _ = cmd
        .arg("--binary")
        .arg(engine)
        .arg(&path)
        .arg(sha)
        .args(["--component", urn]);
    for name in &opt.prerequisites {
        let _ = cmd.args(["--with", name]);
    }
    let status = cmd.status()?;
    let report: ComponentReport = serde_json::from_slice(
        &fs::read(&path).context("component scenario did not produce a report")?,
    )?;
    let passed = status.success();
    Ok((report, passed))
}

pub(super) fn collect(
    root: &Path,
    sha: &str,
    opt: &Options,
    catalog: &Catalog,
    observed: &mut Observations,
    problems: &mut Vec<String>,
    now: &mut Instant,
) -> Result<()> {
    if !opt.observations.is_empty() {
        ensure!(
            git(&["status", "--porcelain", "--untracked-files=normal"])?.is_empty(),
            "saved CI observations require a clean checkout"
        );
        for file in &opt.observations {
            let replay: Observations = serde_json::from_slice(
                &fs::read(file).with_context(|| format!("missing {}", file.display()))?,
            )?;
            ensure!(
                replay.commit == sha,
                "observations came from a different commit"
            );
            coverage::verify_artifact(root, &replay)?;
            merge(observed, replay, catalog);
        }
    } else {
        let (engine, driver) = native_tools()?;
        *now = Instant::now();
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("observations.json");
        let mut command = Command::new(driver);
        let _ = command.arg("--binary").arg(engine).arg(&path).arg(sha);
        for name in &opt.prerequisites {
            let _ = command.args(["--with", name]);
        }
        let status = command.status()?;
        if path.try_exists()? {
            let native: Observations = serde_json::from_slice(&fs::read(path)?)?;
            coverage::verify_artifact(root, &native)?;
            *observed = native;
        }
        if !status.success() && observed.checks.values().all(|passed| *passed) {
            problems.push(format!(
                "scenario runner exited with {status} before reporting a failed observation"
            ));
        }
    }
    Ok(())
}
