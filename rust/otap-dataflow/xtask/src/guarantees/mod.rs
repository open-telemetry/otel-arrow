// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Run candidate scenarios locally or consume observations from this CI run.
mod coverage;
mod html;
mod results;
mod review;

use anyhow::{Context, Result, bail, ensure};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

type Checks = BTreeMap<String, bool>;
const DFE: &str = "rust/otap-dataflow";
const DOCS: &str = "rust/otap-dataflow/docs/guarantees";
const NATIVE: [&str; 8] = [
    "factory-validation",
    "embedded-ui",
    "released-inputs",
    "reconfiguration-consistency",
    "finite-id-drain",
    "live-rollout",
    "extension-rollout",
    "rejected-update",
];

fn area(id: &str) -> Option<&'static str> {
    match id {
        "factory-validation" | "release-precedence" | "released-inputs" => Some("configuration"),
        "embedded-ui" => Some("telemetry"),
        "retry-recovery" | "wire-fidelity" | "finite-id-drain" => Some("behavior"),
        id if id.starts_with("backpressure-") => Some("architecture"),
        "reconfiguration-planning" | "reconfiguration-consistency" => Some("live"),
        "live-rollout" | "extension-rollout" | "rejected-update" => Some("live"),
        id if id.starts_with("rollout:") => Some("live"),
        _ => None,
    }
}
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
#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(tag = "status", rename_all = "lowercase", deny_unknown_fields)]
enum OperationOutcome {
    Passed,
    Failed { reason: String },
    Unexercised,
}

impl OperationOutcome {
    fn label(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed { .. } => "failed",
            Self::Unexercised => "unexercised",
        }
    }
}

#[derive(Deserialize, Serialize)]
struct Rollout {
    operations: BTreeMap<String, OperationOutcome>,
    error: Option<String>,
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

#[derive(Deserialize, Default)]
struct Observations {
    commit: String,
    checks: Checks,
    #[serde(default)]
    check_errors: BTreeMap<String, String>,
    linked_validators: BTreeMap<String, bool>,
    rollouts: BTreeMap<String, Rollout>,
    prerequisites: BTreeSet<String>,
}

impl Observations {
    // A failed or incomplete observation stays failed when another CI environment also ran it.
    fn merge(&mut self, other: Self) {
        for id in NATIVE {
            *self.checks.entry(id.into()).or_insert(true) &= other.checks.get(id) == Some(&true);
        }
        for (urn, passed) in other.linked_validators {
            *self.linked_validators.entry(urn).or_insert(true) &= passed;
        }
        self.check_errors.extend(other.check_errors);
        self.prerequisites.extend(other.prerequisites);
        for (urn, mut result) in other.rollouts {
            match self.rollouts.entry(urn) {
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
}

const HELP: &str = "cargo xtask check-contracts [--component FULL_URN | --base REF] [--metrics PATH] [--html PATH] [--open]\n  Validate the current working tree and print the operator summary.\n  --component FULL_URN checks one factory, its lifecycle fixture and known behavior tests.\n  --base REF also reviews staged, unstaged and untracked changes against REF.\n  --with NAME also runs fixtures requiring this prerequisite (repeatable).\n  --html PATH exports a self-contained HTML report, including findings on an unsuccessful run.\n  --open opens the HTML report in the default browser (default path: target/guarantees.html).\n  --write-support regenerates the inventory table only.\n  CI: --junit DIR with one or more --observations FILE options consumes results from the checked-out candidate.";
#[derive(Default)]
struct Options {
    component: Option<String>,
    base: Option<String>,
    metrics: Option<PathBuf>,
    html: Option<PathBuf>,
    open: bool,
    observations: Vec<PathBuf>,
    prerequisites: BTreeSet<String>,
    junit: Option<PathBuf>,
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
            "--base" => opt.base = Some(value.clone()),
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
            "--junit" => opt.junit = Some(cwd.join(value)),
            _ => bail!("unknown check-contracts option {flag}"),
        }
    }
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
        opt.observations.is_empty() || opt.junit.is_some(),
        "--observations requires --junit"
    );
    ensure!(
        !opt.write
            || (opt.base.is_none()
                && opt.metrics.is_none()
                && opt.html.is_none()
                && opt.junit.is_none()
                && opt.prerequisites.is_empty()),
        "--write-support is a standalone operation"
    );
    ensure!(
        opt.observations.is_empty() || opt.prerequisites.is_empty(),
        "--with selects native fixtures; imported artifacts record their own prerequisites"
    );
    ensure!(
        opt.component.is_none() || (opt.base.is_none() && opt.junit.is_none() && !opt.write),
        "--component runs a local component check; --base, --junit and --write-support require the full inventory"
    );
    Ok(opt)
}

fn native_tools() -> Result<(PathBuf, PathBuf)> {
    let engine = results::executable(
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
    let driver = results::executable(
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

pub fn run(args: &[String]) -> Result<()> {
    if args == ["--help"] || args == ["-h"] {
        println!("{HELP}");
        return Ok(());
    }
    let opt = options(args)?;
    let root = String::from_utf8(git(&["rev-parse", "--show-toplevel"])?)?;
    let root = Path::new(root.trim());
    std::env::set_current_dir(root.join(DFE))?;
    let sha = String::from_utf8(git(&["rev-parse", "HEAD"])?)?
        .trim()
        .to_owned();
    if let Some(urn) = opt.component {
        let (components, _) = super::component_inventory::guarantee_inventory(&root.join(DFE))?;
        ensure!(
            coverage::registered(components)?.contains_key(&urn),
            "unknown component URN: {urn}"
        );
        let (engine, driver) = native_tools()?;
        let dir = tempfile::tempdir()?;
        let path = dir.path().join("component.json");
        let mut cmd = Command::new(driver);
        let _ = cmd
            .arg("--binary")
            .arg(engine)
            .arg(&path)
            .arg(&sha)
            .args(["--component", &urn]);
        for name in opt.prerequisites {
            let _ = cmd.args(["--with", &name]);
        }
        let status = cmd.status()?;
        let mut report: serde_json::Value = serde_json::from_slice(
            &fs::read(&path).context("component scenario did not produce a report")?,
        )?;
        let mut checks: Checks = serde_json::from_value(report["behavior_checks"].take())?;
        if report["factory_validation"].is_boolean() {
            checks.extend(results::local(results::build_cases(Some(&urn))?)?);
        }
        for (id, passed) in &checks {
            println!("{id}: {}", if *passed { "passed" } else { "failed" });
        }
        report["behavior_checks"] = json!(checks);
        let passed = status.success() && checks.values().all(|passed| *passed);
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
        let outcome = (|| -> Result<()> {
            if let Some(dir) = opt.junit {
                ensure!(
                    git(&["status", "--porcelain", "--untracked-files=normal"])?.is_empty(),
                    "saved CI observations require a clean checkout"
                );
                match results::junit(&dir, &sha) {
                    Ok(cases) => observed.checks = cases,
                    Err(e) => problems.push(format!("JUnit: {e}")),
                }
            }
            if !opt.observations.is_empty() {
                for file in opt.observations {
                    let replay: Observations = serde_json::from_slice(
                        &fs::read(&file).with_context(|| format!("missing {}", file.display()))?,
                    )?;
                    ensure!(
                        replay.commit == sha,
                        "observations came from a different commit"
                    );
                    coverage::verify_artifact(root, &replay)?;
                    observed.merge(replay);
                }
            } else {
                let (engine, driver) = native_tools()?;
                let cases = results::build_cases(None)?;
                now = Instant::now();
                let dir = tempfile::tempdir()?;
                let path = dir.path().join("observations.json");
                let mut command = Command::new(driver);
                let _ = command.arg("--binary").arg(engine).arg(&path).arg(&sha);
                for name in opt.prerequisites {
                    let _ = command.args(["--with", &name]);
                }
                let status = command.status()?;
                if path.try_exists()? {
                    let mut native: Observations = serde_json::from_slice(&fs::read(path)?)?;
                    coverage::verify_artifact(root, &native)?;
                    native.checks.extend(std::mem::take(&mut observed.checks));
                    observed = native;
                }
                if !status.success() && observed.checks.values().all(|passed| *passed) {
                    problems.push(format!(
                        "scenario runner exited with {status} before reporting a failed observation"
                    ));
                }
                for (id, passed) in results::local(cases)? {
                    *observed.checks.entry(id).or_insert(true) &= passed;
                }
            }
            Ok(())
        })();
        if let Err(e) = outcome {
            problems.push(format!("{e:#}"));
        }
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
    let mut summary = String::from("## Operator guarantees\n\n");
    let candidate_run = review::candidate_run();
    if !opt.write {
        if let Some(url) = &candidate_run {
            writeln!(summary, "[Candidate test run]({url})\n")?;
        }
        for id in NATIVE.into_iter().chain(results::required()) {
            let passed = observed.checks.get(id) == Some(&true);
            writeln!(
                summary,
                "- **{}** {id}",
                if passed { "tested" } else { "evidence missing" }
            )?;
            if !passed {
                problems.push(match observed.check_errors.get(id) {
                    Some(detail) => format!("{id}: {detail}"),
                    None => format!("No passing candidate observation for {id}"),
                });
            }
        }
    }
    let groups = operation_groups(&observed.rollouts);
    for group in &groups {
        writeln!(summary, "\n### {}", group.title)?;
        writeln!(
            summary,
            "\n| Component | Add | Change | Remove |\n| --- | --- | --- | --- |"
        )?;
        for (urn, rollout) in &group.rows {
            write!(summary, "| `{urn}` |")?;
            for op in coverage::OPERATIONS {
                let state = rollout.operations.get(op);
                let label = state.map_or("missing", OperationOutcome::label);
                write!(summary, " {label} |")?;
                let _ = observed.checks.insert(
                    format!("rollout:{urn}/{op}"),
                    state == Some(&OperationOutcome::Passed),
                );
            }
            writeln!(summary)?;
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
    if !opt.write
        && let Some(c) = &coverage
    {
        writeln!(
            summary,
            "\n{} components in source, {} linked in the validation runner. {} of {} required operations have executable coverage, {} exercised here.",
            c["source_components"].as_array().map_or(0, Vec::len),
            observed.linked_validators.len(),
            c["scenario_linked_operations"],
            c["registered_operations"],
            c["exercised_operations"]
        )?;
    }
    if let Some(c) = &coverage {
        if let Some(deferred) = c["deferred_fixtures"].as_object() {
            for (urn, reason) in deferred {
                writeln!(
                    summary,
                    "- `{urn}` unexercised ({})",
                    reason.as_str().unwrap_or_default()
                )?;
            }
        }
        let table = review::link(&format!("{DOCS}/live-support.md"));
        if opt.write {
            writeln!(summary, "Generated {DOCS}/live-support.md")?;
        } else {
            if let Some(components) = c["components_without_live_fixture"].as_array()
                && !components.is_empty()
            {
                writeln!(
                    summary,
                    "\n### {} ({} components)\n",
                    coverage::NO_LIVE_FIXTURE,
                    components.len()
                )?;
                for urn in components.iter().filter_map(serde_json::Value::as_str) {
                    writeln!(summary, "- `{urn}`")?;
                }
            }
            writeln!(
                summary,
                "\n[Coverage by component and open gaps](<{table}>). Owner: {}.",
                c["gap_owner"].as_str().unwrap_or_default()
            )?;
        }
    }
    let mut review = None;
    if let Some(base) = &base {
        match review::report(base, &observed.checks, &mut problems) {
            Ok(result) => {
                result.write_summary(&mut summary)?;
                review = Some(result);
            }
            Err(e) => problems.push(format!("PR review: {e:#}")),
        }
    }
    for problem in &problems {
        writeln!(summary, "\n- {problem}")?;
    }
    let elapsed = now.elapsed().as_secs_f64();
    writeln!(summary, "\nExecution after build: {elapsed:.2}s")?;
    if let Some(path) = opt.metrics {
        fs::write(
            path,
            serde_json::to_vec_pretty(&json!({
                "commit":sha,"seed_results":observed.checks,"coverage":coverage,"rollouts":observed.rollouts,
                "affected_areas":review.as_ref().map_or(&[][..], |r| r.affected.as_slice()),"blocking_findings":problems,"elapsed_seconds":elapsed
            }))?,
        )?;
    }
    print!("{summary}");
    if let Some(path) = opt.html {
        let checks = NATIVE.into_iter().chain(results::required()).map(|id| {
            json!({"name": id, "passed": observed.checks.get(id), "error": observed.check_errors.get(id)})
        }).collect::<Vec<_>>();
        let follow_up = coverage
            .as_ref()
            .and_then(|c| c["follow_up"].as_str())
            .map(|path| review::link(&format!("{DOCS}/{path}")));
        html::export(
            &path,
            opt.open,
            root,
            minijinja::context!(
                commit => sha, passed => problems.is_empty(), findings => problems,
                checks, coverage, groups, elapsed => format!("{elapsed:.2}"),
                review, base, candidate_run, follow_up, support => review::link(&format!("{DOCS}/live-support.md")),
            ),
        )?;
    }
    ensure!(
        problems.is_empty(),
        "{} guarantee finding(s)",
        problems.len()
    );
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
                let report = json!({"commit":"candidate", "checks":{"live-rollout":operations == &complete}, "linked_validators":{"urn":true},
                    "prerequisites":[], "rollouts":{"urn":{"operations":operations,"error":null}}});
                merged.merge(serde_json::from_value(report).unwrap());
            }
            let states = &merged.rollouts["urn"].operations;
            assert_eq!(states["add"], OperationOutcome::Passed);
            assert!(matches!(states["change"], OperationOutcome::Failed { .. }));
            assert_eq!(states["remove"], OperationOutcome::Unexercised);
            assert!(!merged.checks["live-rollout"]);
        }
    }
}
