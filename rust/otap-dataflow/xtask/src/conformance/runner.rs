// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Build and invoke the native engine and conformance driver.
use super::{DFE, cli::Options, coverage, evidence};
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_conformance_model::{ComponentReport, Observations};
use serde::Deserialize;
use std::process::Stdio;
use std::{
    fs,
    path::{Path, PathBuf},
    process::Command,
    time::Instant,
};

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
            "otel-arrow-dfe-conformance",
            "--bin",
            "conformance",
            "--features",
            "crypto-ring,offline-fixtures",
        ],
        "conformance",
    )?;
    Ok((engine, driver))
}

fn native_command(
    driver: &Path,
    engine: &Path,
    output: &Path,
    sha: &str,
    opt: &Options,
) -> Command {
    let mut cmd = Command::new(driver);
    let _ = cmd.arg("--binary").arg(engine).arg(output).arg(sha);
    if let Some(urn) = &opt.component {
        let _ = cmd.args(["--component", urn]);
    }
    for name in &opt.prerequisites {
        let _ = cmd.args(["--with", name]);
    }
    if opt.ui_only {
        let _ = cmd.args(["--check", "embedded-ui"]);
    }
    cmd
}

pub(super) fn component(
    root: &Path,
    sha: &str,
    urn: &str,
    opt: &Options,
) -> Result<(ComponentReport, bool)> {
    let (components, _) = crate::component_inventory::conformance_inventory(&root.join(DFE))?;
    let registered = coverage::registered(components)?;
    ensure!(registered.contains_key(urn), "unknown component URN: {urn}");
    let (engine, driver) = native_tools()?;
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("component.json");
    let status = native_command(&driver, &engine, &path, sha, opt).status()?;
    let mut report: ComponentReport = serde_json::from_slice(
        &fs::read(&path).context("component scenario did not produce a report")?,
    )?;
    if let Some(finding) = &mut report.finding
        && let Some(prerequisite) = coverage::fixture_prerequisite(root, urn)?
    {
        finding.push_str(": ");
        finding.push_str(&prerequisite);
    }
    let passed = status.success();
    Ok((report, passed))
}

pub(super) fn collect(
    root: &Path,
    sha: &str,
    opt: &Options,
    observed: &mut Observations,
    problems: &mut Vec<String>,
    now: &mut Instant,
) -> Result<()> {
    let (engine, driver) = native_tools()?;
    *now = Instant::now();
    let dir = tempfile::tempdir()?;
    let path = dir.path().join("observations.json");
    let status = native_command(&driver, &engine, &path, sha, opt).status()?;
    if path.try_exists()? {
        let native: Observations = serde_json::from_slice(&fs::read(path)?)?;
        evidence::verify_artifact(root, &native)?;
        *observed = native;
    }
    if !status.success() && observed.checks.values().all(|passed| *passed) {
        problems.push(format!(
            "scenario runner exited with {status} before reporting a failed observation"
        ));
    }
    Ok(())
}
