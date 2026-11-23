// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Run candidate scenarios locally or reuse observations from the same revision.
mod cli;
mod coverage;
mod evidence;
mod html;
mod report;
mod review;
mod runner;

use anyhow::{Context, Result, ensure};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use otel_arrow_dfe_conformance_model::{Observations, Operation, OperationOutcome, catalog};
const DFE: &str = "rust/otap-dataflow";
const DOCS: &str = "rust/otap-dataflow/docs/conformance";
fn git(args: &[&str]) -> Result<Vec<u8>> {
    let output = Command::new("git")
        .arg("--literal-pathspecs")
        .args(args)
        .output()?;
    ensure!(
        output.status.success(),
        "git {}: {}",
        args.join(" "),
        String::from_utf8_lossy(&output.stderr)
    );
    Ok(output.stdout)
}

fn read_base(base: &str, path: &str) -> Result<Option<Vec<u8>>> {
    // A failing git show can be a missing object as well as a missing path.
    if git(&["ls-tree", "-z", base, "--", path])?.is_empty() {
        return Ok(None);
    }
    Ok(Some(git(&["show", &format!("{base}:{path}")])?))
}

fn repository_file(path: &Path) -> Result<PathBuf> {
    ensure!(
        path.is_relative(),
        "expected a repository-relative file: {}",
        path.display()
    );
    let absolute = path
        .canonicalize()
        .with_context(|| format!("missing {}", path.display()))?;
    ensure!(absolute.is_file(), "expected file at {}", path.display());
    Ok(absolute
        .strip_prefix(std::env::current_dir()?.canonicalize()?)?
        .to_owned())
}

pub fn run(args: &[String]) -> Result<()> {
    if args == ["--help"] || args == ["-h"] {
        println!("{}", cli::HELP);
        return Ok(());
    }
    let opt = cli::parse(args)?;
    let mut catalog = catalog()?;
    if opt.ui_only {
        catalog.retain(|_, contract| {
            contract.scenario == Some(otel_arrow_dfe_conformance_model::Scenario::EmbeddedUi)
        });
    }
    let root = String::from_utf8(git(&["rev-parse", "--show-toplevel"])?)?;
    let root = Path::new(root.trim());
    std::env::set_current_dir(root.join(DFE))?;
    let sha = String::from_utf8(git(&["rev-parse", "HEAD"])?)?
        .trim()
        .to_owned();
    if let Some(urn) = opt.component.as_deref() {
        let (report, passed) = runner::component(root, &sha, urn, &opt)?;
        report::print_component(&report);
        if let Some(path) = opt.metrics {
            fs::write(path, serde_json::to_vec_pretty(&report)?)?;
        }
        if let Some(path) = opt.html {
            html::export(
                &path,
                opt.open,
                root,
                html::component_context(&report, passed),
            )?;
        }
        ensure!(passed, "component conformance checks incomplete for {urn}");
        return Ok(());
    }
    let base = opt
        .base
        .as_ref()
        .map(|b| -> Result<String> {
            Ok(String::from_utf8(git(&[
                "rev-parse",
                "--verify",
                "--end-of-options",
                &format!("{b}^{{commit}}"),
            ])?)?
            .trim()
            .to_owned())
        })
        .transpose()?;
    let mut problems = Vec::new();
    let mut now = Instant::now();
    let mut observed = Observations::default();
    if !opt.write {
        let result = if opt.observations.is_empty() {
            runner::collect(root, &sha, &opt, &mut observed, &mut problems, &mut now)
        } else {
            evidence::import(root, &sha, &opt.observations, &catalog, &mut observed)
        };
        if let Err(error) = result {
            problems.push(format!("{error:#}"));
        }
    }

    std::env::set_current_dir(root)?;
    if !opt.write {
        for (id, _) in catalog
            .iter()
            .filter(|(_, c)| c.selected(None) && c.scenario.is_some())
        {
            if observed.checks.get(id) != Some(&true) {
                problems.push(match observed.check_errors.get(id) {
                    Some(detail) => format!("{id}: {detail}"),
                    None => format!("No passing candidate observation for {id}"),
                });
            }
        }
        for (urn, rollout) in &observed.rollouts {
            for op in Operation::names() {
                let passed = rollout.operations.get(op) == Some(&OperationOutcome::Passed);
                let _ = observed
                    .checks
                    .insert(format!("rollout:{urn}/{op}"), passed);
            }
            if let Some(error) = &rollout.error {
                problems.push(format!("{urn}: {error}"));
            }
        }
    }
    let coverage = if opt.ui_only {
        None
    } else {
        super::component_inventory::conformance_inventory(&root.join(DFE))
            .and_then(|(entries, drift)| {
                if let Err(e) = drift {
                    problems.push(format!("source inventory: {e}"));
                }
                coverage::registered(entries)
            })
            .and_then(|current| {
                coverage::check(
                    current,
                    base.as_deref(),
                    opt.write,
                    &observed,
                    &mut problems,
                )
            })
            .map_err(|e| problems.push(format!("Coverage: {e:#}")))
            .ok()
    };
    if opt.write {
        ensure!(problems.is_empty(), "{}", problems.join("\n"));
        println!("Generated {DOCS}/live-support.md");
        return Ok(());
    }
    let review = base.as_deref().and_then(|reference| {
        review::report(
            reference,
            &observed.checks,
            &catalog,
            opt.assessment.as_deref(),
            &mut problems,
        )
        .map_err(|e| problems.push(format!("Evidence missing for PR review: {e:#}")))
        .ok()
    });
    let report = report::Report {
        commit: &sha,
        observed,
        coverage,
        review,
        blocking_findings: problems,
        elapsed_seconds: now.elapsed().as_secs_f64(),
    };
    report.output(
        root,
        &catalog,
        base.as_deref(),
        opt.metrics.as_deref(),
        opt.html.as_deref(),
        opt.open,
    )?;
    Ok(())
}
