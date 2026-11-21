// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{Checks, DFE, DOCS, coverage, git, read_base, repository_file};
use anyhow::{Context, Result, bail, ensure};
use regex::Regex;
use serde::Deserialize;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::Path;

const MAP: &str = "rust/otap-dataflow/docs/guarantees/review-map.json";
const FIXTURES: &str = "rust/otap-dataflow/crates/config/tests/fixtures/releases";
const REVIEW: &str = ".github/guarantee-reviews";
const EXCEPTIONS: &str = "rust/otap-dataflow/docs/guarantees/exceptions";

#[derive(Deserialize)]
struct Rule {
    path: String,
    lines: Option<String>,
    evidence: String,
}
#[derive(Deserialize)]
struct Assessment {
    evidence: Vec<Evidence>,
    #[serde(default)]
    benchmarks: BTreeMap<String, Benchmark>,
}
#[derive(Deserialize)]
struct Evidence {
    area: String,
    paths: BTreeSet<String>,
    status: String,
    source: String,
    reason: String,
}
#[derive(Deserialize)]
struct Benchmark {
    workload: Option<String>,
    obstacle: Option<String>,
    measurement: String,
}

fn section<'a>(doc: &'a str, heading: &str) -> Option<&'a str> {
    let (_, body) = doc.split_once(&format!("\n## {heading}\n"))?;
    Some(body.split("\n## ").next()?.trim()).filter(|text| !text.is_empty())
}
fn decision_doc(doc: &str) -> Result<()> {
    for heading in [
        "Scope",
        "Justification",
        "Operator procedure",
        "Migration and release",
        "Verification",
        "Decision",
    ] {
        ensure!(
            section(doc, heading).is_some(),
            "exception needs a populated '{heading}' section"
        );
    }
    ensure!(
        Regex::new(r"https://github\.com/open-telemetry/otel-arrow/(pull|issues)/[0-9]+")?
            .is_match(doc),
        "exception needs the maintainer decision link"
    );
    Ok(())
}
pub fn approved(base: &str, path: &Path, scope: &str) -> Result<()> {
    ensure!(
        path.parent() == Some(Path::new(EXCEPTIONS)) && path.extension().is_some_and(|x| x == "md"),
        "exception outside approved directory"
    );
    let path = path.to_str().context("non-UTF8 decision path")?;
    let git_path = if cfg!(windows) {
        path.replace('\\', "/")
    } else {
        path.to_owned()
    };
    let before = read_base(base, &git_path)?;
    decision(before.as_deref(), &fs::read(path)?, scope)
}

fn decision(before: Option<&[u8]>, current: &[u8], scope: &str) -> Result<()> {
    ensure!(
        before.context("decision must already be on the base")? == current,
        "decision must be unchanged from the base"
    );
    let doc = std::str::from_utf8(current)?;
    decision_doc(doc)?;
    ensure!(
        section(doc, "Scope").is_some_and(|s| s.lines().any(|l| l == format!("- {scope}"))),
        "exception scope must include {scope}"
    );
    Ok(())
}
fn source(s: &str) -> Result<()> {
    if s.starts_with("https://") {
        ensure!(
            s.len() > 8 && !s.contains(|c: char| c.is_whitespace() || "<>|".contains(c)),
            "invalid evidence URL"
        );
    } else {
        repository_file(Path::new(s.split('#').next().unwrap_or_default()))?;
    }
    Ok(())
}
impl Benchmark {
    fn check(&self) -> Result<()> {
        if let Some(workload) = &self.workload {
            let path = repository_file(Path::new(workload))?;
            ensure!(
                path.starts_with("tools/pipeline_perf_test/test_suites/integration/continuous"),
                "workload outside continuous suites"
            );
            let workflow =
                fs::read_to_string(".github/workflows/pipeline-perf-test-continuous.yml")?;
            let relative = path
                .strip_prefix("tools/pipeline_perf_test")?
                .to_str()
                .context("workload path")?;
            ensure!(
                workflow.contains(&relative.replace('\\', "/")),
                "workload must run in continuous CI"
            );
        } else {
            ensure!(
                self.obstacle.as_ref().is_some_and(|s| !s.trim().is_empty()),
                "missing benchmark workload or obstacle"
            );
        }
        source(&self.measurement)
    }
}
impl Evidence {
    fn check(&self, base: &str, paths: &BTreeSet<String>, checks: &Checks) -> Result<()> {
        ensure!(
            !self.paths.is_empty() && self.paths.is_subset(paths) && !self.reason.trim().is_empty(),
            "expected changed paths and a rationale"
        );
        match self.status.as_str() {
            "tested" => {
                ensure!(
                    checks.get(&self.source) == Some(&true),
                    "missing passing observation for {}",
                    self.source
                );
                if let Some(area) = super::area(&self.source) {
                    ensure!(area == self.area, "{} covers {area}", self.source);
                }
            }
            "reviewed with rationale" => source(&self.source)?,
            "approved exception" => approved(base, Path::new(&self.source), &self.area)?,
            _ => bail!("unexpected evidence status {}", self.status),
        }
        Ok(())
    }
}

fn changed(base: &str) -> Result<(BTreeSet<String>, BTreeSet<String>)> {
    let paths = |data: Vec<u8>| -> Result<BTreeSet<String>> {
        Ok(String::from_utf8(data)?
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect())
    };
    let mut all = paths(git(&[
        "diff",
        "--no-renames",
        "--name-only",
        "-z",
        base,
        "--",
    ])?)?;
    let new = paths(git(&["ls-files", "--others", "--exclude-standard", "-z"])?)?;
    all.extend(new.iter().cloned());
    Ok((all, new))
}

fn changed_lines(base: &str, path: &str, new: bool) -> Result<String> {
    if new {
        return Ok(String::from_utf8_lossy(&fs::read(path)?).into_owned());
    }
    let bytes = git(&[
        "diff",
        "--no-ext-diff",
        "--no-renames",
        "--unified=0",
        base,
        "--",
        path,
    ])?;
    let text = String::from_utf8_lossy(&bytes);
    Ok(text
        .lines()
        .filter(|l| l.starts_with(['+', '-']) && !l.starts_with("+++") && !l.starts_with("---"))
        .map(|l| &l[1..])
        .collect::<Vec<_>>()
        .join("\n"))
}
fn fixture_migrations(base: &str) -> Result<()> {
    #[derive(Deserialize)]
    struct Migration {
        replacement: String,
        decision: String,
        diagnostic: String,
    }
    let manifest = Path::new(DOCS).join("release-migrations.json");
    if !manifest.try_exists()? {
        return Ok(());
    }
    let migrations: BTreeMap<String, Migration> = serde_json::from_slice(&fs::read(manifest)?)?;
    for (name, m) in &migrations {
        for file in [name, &m.replacement] {
            let path = repository_file(&Path::new(FIXTURES).join(file))?;
            ensure!(
                path.starts_with(FIXTURES)
                    && path.strip_prefix(FIXTURES)?.components().count() == 2,
                "invalid release fixture path"
            );
        }
        ensure!(
            !migrations.contains_key(&m.replacement) && !m.diagnostic.trim().is_empty(),
            "invalid replacement for {name}"
        );
        approved(
            base,
            Path::new(&m.decision),
            &format!("configuration {name}"),
        )?;
    }
    Ok(())
}
fn escape(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('|', "&#124;")
        .replace(['\n', '\r'], " ")
}
pub(super) fn link(s: &str) -> String {
    let repo = std::env::var("GITHUB_REPOSITORY");
    let sha = std::env::var("GITHUB_SHA");
    if s.starts_with("https://") {
        s.to_owned()
    } else if let (Ok(repo), Ok(sha)) = (repo, sha) {
        format!("https://github.com/{repo}/blob/{sha}/{s}")
    } else {
        s.to_owned()
    }
}

struct Matcher<'a> {
    patterns: Vec<(&'a String, Regex, Option<Regex>)>,
    node_doc: Regex,
}
impl<'a> Matcher<'a> {
    fn new(rules: &'a BTreeMap<String, Rule>) -> Result<Self> {
        let patterns = rules
            .iter()
            .map(|(id, r)| {
                Ok((
                    id,
                    Regex::new(&r.path)?,
                    r.lines.as_deref().map(Regex::new).transpose()?,
                ))
            })
            .collect::<Result<Vec<_>>>()?;
        let node_doc = Regex::new(r"/(receivers|processors|exporters)/")?;
        Ok(Self { patterns, node_doc })
    }

    fn affected(&self, name: &str, lines: &str) -> Result<Vec<&str>> {
        let path = Path::new(name);
        let mut areas = Vec::new();
        if name.ends_with("/README.md") && self.node_doc.is_match(name) {
            areas.push("configuration");
            for (area, pattern) in [
                ("live", "reconfigur|in-place"),
                ("security", "credential|secret|authentication"),
                ("performance", "benchmark|throughput"),
            ] {
                if Regex::new(pattern)?.is_match(lines) {
                    areas.push(area);
                }
            }
        } else {
            for (area, pattern, content) in &self.patterns {
                let code = matches!(
                    path.extension().and_then(|s| s.to_str()),
                    Some("rs" | "toml" | "proto" | "yaml" | "yml")
                );
                if pattern.is_match(name)
                    || (path.starts_with(DFE)
                        && code
                        && content.as_ref().is_some_and(|r| r.is_match(lines)))
                {
                    areas.push(area);
                }
            }
            if areas.is_empty() && name.ends_with(".rs") {
                areas.push("design");
            }
        }
        Ok(areas)
    }
}

pub fn report(
    base: &str,
    checks: &Checks,
    problems: &mut Vec<String>,
    summary: &mut String,
) -> Result<Vec<String>> {
    let (paths, untracked) = changed(base)?;
    let rules: BTreeMap<String, Rule> = serde_json::from_slice(&fs::read(MAP)?)?;
    let mut impacts: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
    let mut records = Vec::new();
    let mut benchmarks = BTreeSet::new();
    let matcher = Matcher::new(&rules)?;
    for name in &paths {
        let path = Path::new(name);
        if path.parent() == Some(Path::new(REVIEW)) && path.extension().is_some_and(|x| x == "json")
        {
            let result = (|| -> Result<()> {
                ensure!(
                    read_base(base, name)?.is_none(),
                    "add a new assessment instead of changing a prior one"
                );
                let data: Assessment = serde_json::from_slice(&fs::read(path)?)?;
                for (id, b) in data.benchmarks {
                    match b.check() {
                        Ok(()) => {
                            benchmarks.insert(id);
                        }
                        Err(e) => problems.push(format!("{name}: {id}: {e}")),
                    }
                }
                for item in data.evidence {
                    if let Err(e) = item.check(base, &paths, checks) {
                        problems.push(format!("{name}: {e}"));
                        continue;
                    }
                    if !rules.contains_key(&item.area) {
                        problems.push(format!("Unknown guarantee {}", item.area));
                        continue;
                    }
                    impacts
                        .entry(item.area.clone())
                        .or_default()
                        .extend(item.paths.iter().cloned());
                    records.push((name, item));
                }
                Ok(())
            })();
            if let Err(e) = result {
                problems.push(format!("{name}: {e}"));
            }
            continue;
        }
        if path.parent() == Some(Path::new(EXCEPTIONS)) {
            if path.is_file()
                && let Err(e) = decision_doc(&fs::read_to_string(path)?)
            {
                problems.push(format!("{name}: {e}"));
            }
            continue;
        }
        if path.starts_with(FIXTURES) && read_base(base, name)?.is_some() {
            problems.push(format!("keep released fixture {name} intact"));
        }
        if ![
            DFE,
            "proto",
            ".github/workflows",
            "tools/pipeline_perf_test",
        ]
        .iter()
        .any(|prefix| path.starts_with(prefix))
        {
            continue;
        }
        // Attribute the format migration to the YAML catalog. The coverage check
        // still compares every operation to its declaration in the base JSON.
        if name == coverage::LEGACY_CATALOG && !path.exists() && paths.contains(coverage::CATALOG) {
            continue;
        }
        let lines = changed_lines(base, name, untracked.contains(name))?;
        for area in matcher.affected(name, &lines)? {
            impacts
                .entry(area.to_string())
                .or_default()
                .insert(name.clone());
        }
    }
    if [
        "docs/design-principles.md",
        "docs/ai/ai-assisted-pr-review.md",
    ]
    .iter()
    .any(|p| paths.contains(&format!("{DFE}/{p}")))
        && !paths.contains(MAP)
    {
        problems.push("Update the mapping when either review guide changes.".into());
    }
    if let Err(e) = fixture_migrations(base) {
        problems.push(format!("Release migrations: {e}"));
    }
    let old =
        coverage::ids(&read_base(base, coverage::INVENTORY)?.context("base inventory missing")?)?;
    for id in coverage::ids(&fs::read(coverage::INVENTORY)?)?.keys() {
        if !old.contains_key(id) && !benchmarks.contains(id) {
            problems.push(format!(
                "New component {id} needs a continuous workload or obstacle and measurement"
            ));
        }
    }
    writeln!(
        summary,
        "\n| Area | Outcome | Evidence |\n| --- | --- | --- |"
    )?;
    for (record, item) in records {
        let scope = impacts
            .get_mut(&item.area)
            .expect("inserted during validation");
        scope.retain(|p| !item.paths.contains(p));
        let evidence = if item.status == "tested" {
            std::env::var("GITHUB_RUN_ID")
                .ok()
                .zip(std::env::var("GITHUB_REPOSITORY").ok())
                .map(|(id, repo)| format!("https://github.com/{repo}/actions/runs/{id}"))
                .unwrap_or_else(|| link(record))
        } else {
            link(&item.source)
        };
        writeln!(
            summary,
            "| {} | {} | {} ([evidence](<{}>), [assessment](<{}>)) |",
            item.area,
            item.status,
            escape(&item.reason),
            evidence,
            link(record)
        )?;
    }
    for (area, remaining) in &impacts {
        if !remaining.is_empty() {
            let mut paths = remaining
                .iter()
                .take(5)
                .map(|path| escape(path))
                .collect::<Vec<_>>()
                .join(", ");
            if remaining.len() > 5 {
                write!(paths, " (and {} more)", remaining.len() - 5)?;
            }
            writeln!(summary, "| {area} | evidence missing | {paths} |")?;
            problems.push(format!("{area}: {}", rules[area].evidence));
        }
    }
    writeln!(
        summary,
        "\nUnaffected by the diff rules: {}",
        rules
            .keys()
            .filter(|a| !impacts.contains_key(*a))
            .cloned()
            .collect::<Vec<_>>()
            .join(", ")
    )?;
    Ok(impacts.into_keys().collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a receiver starts declaring arrival context without a controller edit.
    /// Guarantees: the live guarantee is selected from the cross-component change.
    #[test]
    fn declaration_adds_live_review() {
        let rules =
            serde_json::from_str(include_str!("../../../docs/guarantees/review-map.json")).unwrap();
        let matcher = Matcher::new(&rules).unwrap();
        let hits = matcher
            .affected(
                "rust/otap-dataflow/crates/custom/src/lib.rs",
                "impl ContextDeclarationProvider for Input",
            )
            .unwrap();
        assert!(hits.contains(&"live"));
    }

    /// Scenario: an assessment cites a failed observation or a successful test for another guarantee.
    /// Guarantees: neither observation can satisfy the changed behavior.
    #[test]
    fn scoped_candidate_results() {
        let paths = BTreeSet::from(["engine.rs".to_owned()]);
        let item = Evidence {
            area: "security".into(),
            source: "live-rollout".into(),
            reason: "probe".into(),
            status: "tested".into(),
            paths: paths.clone(),
        };
        for passed in [false, true] {
            assert!(
                item.check(
                    "HEAD",
                    &paths,
                    &Checks::from([("live-rollout".into(), passed)])
                )
                .is_err()
            );
        }
    }

    /// Scenario: the candidate proposes a new exception absent from the base.
    /// Guarantees: an assessment cannot approve its own exception.
    #[test]
    fn rejects_unapproved_exception() {
        let doc=b"# Decision\n## Scope\n- live\n## Justification\nMigration\n## Operator procedure\nRestart\n## Migration and release\nv2\n## Verification\nManual rollout\n## Decision\nhttps://github.com/open-telemetry/otel-arrow/issues/1";
        assert!(decision(Some(doc), doc, "live").is_ok());
        assert!(decision(None, doc, "live").is_err());
        assert!(decision(Some(b"older"), doc, "live").is_err());
        assert!(decision(Some(doc), doc, "security").is_err());
    }
}
