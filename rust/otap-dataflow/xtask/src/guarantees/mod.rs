// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Run candidate scenarios locally or consume observations from this CI run.
mod coverage;
mod html;
mod report;
mod review;
mod runner;

use anyhow::{Context, Result, bail, ensure};
use serde::Serialize;
#[cfg(test)]
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

use otel_arrow_dfe_contract_model::{
    Catalog, Checks, ComponentReport, Observations, OperationOutcome, Rollout, catalog,
};
const DFE: &str = "rust/otap-dataflow";
const DOCS: &str = "rust/otap-dataflow/docs/guarantees";
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
#[derive(Serialize)]
struct OperationGroup<'a> {
    title: String,
    rows: Vec<(&'a str, &'a Rollout)>,
}

fn operation_groups(rollouts: &BTreeMap<String, Rollout>) -> Vec<OperationGroup<'_>> {
    let mut groups = BTreeMap::new();
    for (urn, rollout) in rollouts {
        let namespace = super::component_inventory::component_namespace(urn);
        // Present the OTel namespace first, then each external namespace in key order.
        groups
            .entry((namespace != Some("otel"), namespace))
            .or_insert_with(Vec::new)
            .push((urn.as_str(), rollout));
    }
    groups
        .into_iter()
        .map(|((_, namespace), rows)| {
            let title = match namespace {
                Some("otel") => "OpenTelemetry (urn:otel)".to_owned(),
                Some(company) => format!("Company components (urn:{company})"),
                None => "Unrecognized namespace".to_owned(),
            };
            OperationGroup { title, rows }
        })
        .collect()
}

const HELP: &str = "cargo xtask conformance [--component FULL_URN | --base REF [--assessment FILE]] [--metrics PATH] [--html PATH] [--open]\n  Validate the current working tree and print the operator summary.\n  --component FULL_URN checks one factory preflight, its deployed scenarios and coverage gaps.\n  --base REF also reviews staged, unstaged and untracked changes against REF.\n  --assessment FILE reads PR evidence from a local YAML file with --base.\n  --with NAME also runs fixtures requiring this prerequisite (repeatable).\n  --html PATH exports a self-contained HTML report, including findings on an unsuccessful run.\n  --open opens the HTML report in the default browser (default path: target/guarantees.html).\n  --write-support regenerates the inventory table only.\n  CI: --observations FILE imports deployment evidence from the clean candidate revision (repeatable).";
#[derive(Default)]
struct Options {
    component: Option<String>,
    base: Option<String>,
    assessment: Option<PathBuf>,
    review_members: Option<PathBuf>,
    metrics: Option<PathBuf>,
    html: Option<PathBuf>,
    open: bool,
    observations: Vec<PathBuf>,
    prerequisites: BTreeSet<String>,
    write: bool,
}
fn options(args: &[String]) -> Result<Options> {
    let mut opt = Options::default();
    let mut args = args.iter();
    let cwd = std::env::current_dir()?;
    while let Some(flag) = args.next() {
        if flag == "--open" {
            opt.open = true;
            continue;
        }
        if flag == "--write-support" {
            opt.write = true;
            continue;
        }
        let value = args
            .next()
            .with_context(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--component" => ensure!(
                opt.component.replace(value.clone()).is_none(),
                "select one component"
            ),
            "--review-members" => opt.review_members = Some(cwd.join(value)),
            "--base" => opt.base = Some(value.clone()),
            "--assessment" => {
                ensure!(
                    opt.assessment.replace(cwd.join(value)).is_none(),
                    "select one assessment"
                );
            }
            "--html" => {
                ensure!(
                    !value.is_empty() && !value.starts_with("--"),
                    "--html requires an output path"
                );
                opt.html = Some(cwd.join(value));
            }
            "--metrics" => opt.metrics = Some(cwd.join(value)),
            "--observations" => opt.observations.push(cwd.join(value)),
            "--with" => {
                let _ = opt.prerequisites.insert(value.clone());
            }
            _ => bail!("unknown conformance option {flag}"),
        }
    }
    ensure!(
        opt.review_members.is_none() || (opt.base.is_some() && opt.assessment.is_none()),
        "CI review members require --base and are exclusive with --assessment"
    );
    ensure!(
        opt.assessment.is_none() || opt.base.is_some(),
        "--assessment requires --base"
    );
    if opt.open && opt.html.is_none() {
        opt.html = Some(cwd.join("target/guarantees.html"));
    }
    if let (Some(html), Some(metrics)) = (&opt.html, &opt.metrics) {
        ensure!(
            html::destination(html)? != html::destination(metrics)?,
            "--html and --metrics require different output paths"
        );
    }
    ensure!(
        !opt.write
            || (opt.base.is_none()
                && opt.metrics.is_none()
                && opt.html.is_none()
                && opt.observations.is_empty()
                && opt.prerequisites.is_empty()),
        "--write-support is a standalone operation"
    );
    ensure!(
        opt.observations.is_empty() || opt.prerequisites.is_empty(),
        "--with selects native fixtures; imported artifacts record their own prerequisites"
    );
    ensure!(
        opt.component.is_none()
            || (opt.base.is_none() && opt.observations.is_empty() && !opt.write),
        "--component runs a local component check; --base, --observations and --write-support require the full inventory"
    );
    Ok(opt)
}

pub fn run(args: &[String]) -> Result<()> {
    if args == ["--help"] || args == ["-h"] {
        println!("{HELP}");
        return Ok(());
    }
    let opt = options(args)?;
    let catalog = catalog()?;
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
                minijinja::context!(
                    commit => sha, passed, component => &urn, focused => report,
                ),
            )?;
        }
        ensure!(passed, "component guarantees incomplete for {urn}");
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
    if !opt.write
        && let Err(e) = runner::collect(
            root,
            &sha,
            &opt,
            &catalog,
            &mut observed,
            &mut problems,
            &mut now,
        )
    {
        problems.push(format!("{e:#}"));
    }
    let inventory = super::component_inventory::guarantee_inventory(&root.join(DFE)).and_then(
        |(entries, drift)| {
            if let Err(e) = drift {
                problems.push(format!("source inventory: {e}"));
            }
            coverage::registered(entries)
        },
    );
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
            for op in coverage::OPERATIONS {
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
    let coverage = inventory
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
        .ok();
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
            opt.review_members.as_deref(),
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: an author passes a focused component report as full candidate evidence.
    /// Guarantees: CI rejects the scoped schema and the CLI keeps focused and PR review modes distinct.
    #[test]
    fn focused_evidence_cannot_satisfy_the_pr_gate() {
        let report = json!({"component":"urn:otel:processor:transform","commit":"head",
            "factory_validation":true,"operations":{},"finding":null});
        assert!(serde_json::from_value::<Observations>(report).is_err());
        let args = [
            "--component",
            "urn:otel:processor:transform",
            "--base",
            "main",
        ]
        .map(String::from);
        assert!(options(&args).is_err());
    }

    /// Scenario: two CI environments report the same add, change and remove with a failure and a missing case in one.
    /// Guarantees: passing evidence remains per operation and another run cannot erase a failure or fill a skipped stage.
    #[test]
    fn merge_preserves_partial_rollouts_in_either_order() {
        let complete = json!({"add":{"status":"passed"},"change":{"status":"passed"},"remove":{"status":"passed"}});
        let partial =
            json!({"add":{"status":"passed"},"change":{"status":"failed","reason":"refused"}});
        for reports in [[&complete, &partial], [&partial, &complete]] {
            let mut merged = Observations::default();
            for operations in reports {
                let report = json!({"commit":"candidate", "ui_platform":{"os":"linux","atomic64":true}, "ui_signals":[], "checks":{"live-rollout":operations == &complete}, "linked_validators":{"urn":true},
                    "prerequisites":[], "rollouts":{"urn":{"operations":operations,"error":null}}});
                runner::merge(
                    &mut merged,
                    serde_json::from_value(report).unwrap(),
                    &catalog().unwrap(),
                );
            }
            let states = &merged.rollouts["urn"].operations;
            assert_eq!(states["add"], OperationOutcome::Passed);
            assert!(matches!(states["change"], OperationOutcome::Failed { .. }));
            assert_eq!(states["remove"], OperationOutcome::Unexercised);
            assert!(!merged.checks["live-rollout"]);
        }
    }
}
