// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Run candidate scenarios locally or consume observations from this CI run.
mod coverage;
mod results;
mod review;

use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use serde_json::json;
use std::collections::BTreeMap;
use std::fmt::Write;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Instant;

type Checks = BTreeMap<String, bool>;
const DFE: &str = "rust/otap-dataflow";
const DOCS: &str = "rust/otap-dataflow/docs/guarantees";
const REQUIRED: [&str; 6] = [
    "factory-validation",
    "released-inputs",
    "release-precedence",
    "retry-recovery",
    "live-rollout",
    "extension-rollout",
];

fn area(id: &str) -> Option<&'static str> {
    match id {
        "factory-validation" | "release-precedence" | "released-inputs" => Some("configuration"),
        "retry-recovery" => Some("behavior"),
        "live-rollout" | "extension-rollout" => Some("live"),
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
#[derive(Deserialize, Default)]
struct Observations {
    commit: String,
    checks: Checks,
    linked_validators: BTreeMap<String, bool>,
    rollout_components: BTreeMap<String, Vec<String>>,
}

const HELP: &str = "cargo xtask guarantees [--base REF] [--metrics PATH]\n  Validate the current working tree and print the operator summary.\n  --base REF also reviews staged, unstaged and untracked changes against REF.\n  --write-support regenerates the inventory table only.\n  CI: --observations FILE --junit DIR consumes results from the checked-out candidate.";
#[derive(Default)]
struct Options {
    base: Option<String>,
    metrics: Option<PathBuf>,
    observations: Option<PathBuf>,
    junit: Option<PathBuf>,
    write: bool,
}
fn options(args: &[String]) -> Result<Options> {
    let mut opt = Options::default();
    let mut args = args.iter();
    let cwd = std::env::current_dir()?;
    while let Some(flag) = args.next() {
        if flag == "--write-support" {
            opt.write = true;
            continue;
        }
        let value = args
            .next()
            .with_context(|| format!("missing value for {flag}"))?;
        match flag.as_str() {
            "--base" => opt.base = Some(value.clone()),
            "--metrics" => opt.metrics = Some(cwd.join(value)),
            "--observations" => opt.observations = Some(cwd.join(value)),
            "--junit" => opt.junit = Some(cwd.join(value)),
            _ => bail!("unknown guarantees option {flag}"),
        }
    }
    ensure!(
        opt.observations.is_none() || opt.junit.is_some(),
        "--observations requires --junit"
    );
    ensure!(
        !opt.write || (opt.base.is_none() && opt.metrics.is_none() && opt.junit.is_none()),
        "--write-support is a standalone operation"
    );
    Ok(opt)
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
            if let Some(file) = opt.observations {
                let replay: Observations = serde_json::from_slice(
                    &fs::read(&file).with_context(|| format!("missing {}", file.display()))?,
                )?;
                ensure!(
                    replay.commit == sha,
                    "observations came from a different commit"
                );
                for id in [
                    "factory-validation",
                    "released-inputs",
                    "live-rollout",
                    "extension-rollout",
                ] {
                    observed
                        .checks
                        .insert(id.to_owned(), replay.checks.get(id) == Some(&true));
                }
                observed.linked_validators = replay.linked_validators;
                observed.rollout_components = replay.rollout_components;
            } else {
                let engine = results::executable(
                    &[
                        "build",
                        "-p",
                        "otel-arrow-dfe",
                        "--bin",
                        "df_engine",
                        "--features",
                        "flat-file-api-key-auth",
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
                        "crypto-ring",
                    ],
                    "guarantee_scenarios",
                )?;
                let cases = results::build_cases()?;
                now = Instant::now();
                let dir = tempfile::tempdir()?;
                let path = dir.path().join("observations.json");
                let status = Command::new(driver)
                    .arg("--binary")
                    .arg(engine)
                    .arg(&path)
                    .arg(&sha)
                    .status()?;
                if path.try_exists()? {
                    let mut native: Observations = serde_json::from_slice(&fs::read(path)?)?;
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
    if !opt.write {
        if let (Ok(repo), Ok(run)) = (
            std::env::var("GITHUB_REPOSITORY"),
            std::env::var("GITHUB_RUN_ID"),
        ) {
            writeln!(
                summary,
                "[Candidate test run](https://github.com/{repo}/actions/runs/{run})\n"
            )?;
        }
        for id in REQUIRED {
            let passed = observed.checks.get(id) == Some(&true);
            writeln!(
                summary,
                "- **{}** {id}",
                if passed { "tested" } else { "evidence missing" }
            )?;
            if !passed {
                problems.push(format!("No passing candidate observation for {id}"));
            }
        }
    }
    let coverage = inventory
        .and_then(|current| coverage::check(current, base.as_deref(), opt.write, &observed))
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
        let table = review::link(&format!("{DOCS}/live-support.md"));
        if opt.write {
            writeln!(summary, "Generated {DOCS}/live-support.md")?;
        } else {
            writeln!(
                summary,
                "[Coverage by component and open gaps](<{table}>). Owner: {}.",
                c["gap_owner"].as_str().unwrap_or_default()
            )?;
        }
    }
    let mut areas = Vec::new();
    if let Some(base) = base {
        match review::report(&base, &observed.checks, &mut problems, &mut summary) {
            Ok(affected) => areas = affected,
            Err(e) => problems.push(format!("PR review: {e:#}")),
        }
    }
    for problem in &problems {
        writeln!(summary, "\n- {problem}")?;
    }
    writeln!(
        summary,
        "\nExecution after build: {:.2}s",
        now.elapsed().as_secs_f64()
    )?;
    if let Some(path) = opt.metrics {
        fs::write(
            path,
            serde_json::to_vec_pretty(&json!({
                "commit":sha,"seed_results":observed.checks,"coverage":coverage,
                "affected_areas":areas,"blocking_findings":problems,"elapsed_seconds":now.elapsed().as_secs_f64()
            }))?,
        )?;
    }
    print!("{summary}");
    ensure!(
        problems.is_empty(),
        "{} guarantee finding(s)",
        problems.len()
    );
    Ok(())
}
