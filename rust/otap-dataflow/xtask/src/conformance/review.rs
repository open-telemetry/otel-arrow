// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{DFE, DOCS, coverage, git, read_base, repository_file};
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_conformance_model::{Catalog, Checks};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::Path;

const MAP: &str = "rust/otap-dataflow/docs/conformance/review-map.json";
const FIXTURES: &str = "rust/otap-dataflow/crates/conformance/fixtures/releases";
const LEGACY_FIXTURES: &str = "rust/otap-dataflow/crates/config/tests/fixtures/releases";
mod assessment;
pub(super) use assessment::approved;
use assessment::{Assessment, EXCEPTIONS, Evidence, decision_doc, parse_assessment};

#[derive(Deserialize)]
struct Rule {
    path: String,
    lines: Option<String>,
    evidence: String,
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

#[derive(Serialize)]
pub(super) struct Review {
    pub affected: Vec<String>,
    unaffected: Vec<String>,
    rows: Vec<ReviewRow>,
}

#[derive(Serialize)]
struct ReviewRow {
    area: String,
    outcome: String,
    reason: String,
    evidence: Option<String>,
    assessment: Option<String>,
}

impl Review {
    pub fn write_summary(&self, summary: &mut String) -> std::fmt::Result {
        writeln!(
            summary,
            "\n| Area | Outcome | Evidence |\n| --- | --- | --- |"
        )?;
        for row in &self.rows {
            write!(
                summary,
                "| {} | {} | {}",
                row.area,
                row.outcome,
                escape(&row.reason)
            )?;
            if let (Some(evidence), Some(assessment)) = (&row.evidence, &row.assessment) {
                write!(
                    summary,
                    " ([evidence](<{evidence}>), [assessment](<{assessment}>))"
                )?;
            }
            writeln!(summary, " |")?;
        }
        writeln!(
            summary,
            "\nUnaffected by the diff rules: {}",
            self.unaffected.join(", ")
        )
    }
}

type Impacts = BTreeMap<String, BTreeSet<String>>;

fn impacts(
    base: &str,
    matcher: &Matcher<'_>,
    problems: &mut Vec<String>,
) -> Result<(BTreeSet<String>, Impacts)> {
    let (paths, untracked) = changed(base)?;
    let mut impacts = Impacts::new();
    for name in &paths {
        let path = Path::new(name);
        if path.parent() == Some(Path::new(EXCEPTIONS)) {
            if path.is_file()
                && let Err(e) = decision_doc(&fs::read_to_string(path)?)
            {
                problems.push(format!("{name}: {e}"));
            }
            continue;
        }
        // A review spanning the move must still compare the historical bytes. The one
        // retired JSON copy is now regenerated from its pinned YAML equivalent.
        if let Ok(key) = path.strip_prefix(LEGACY_FIXTURES)
            && let Some(original) = read_base(base, name)?
        {
            let destination = Path::new(FIXTURES).join(key);
            let intact = if key == Path::new("v0.60.0/syslog-console.json") {
                let yaml = fs::read(destination.with_extension("yaml"))?;
                serde_json::from_slice::<serde_json::Value>(&original)?
                    == serde_yaml::from_slice::<serde_json::Value>(&yaml)?
            } else {
                fs::read(destination).is_ok_and(|bytes| bytes == original)
            };
            if path.exists() || !intact {
                problems.push(format!(
                    "keep released fixture {name} intact when relocating it"
                ));
            }
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
        // Attribute a catalog move to its new path. Coverage separately compares
        // the candidate fixtures to the catalog in the base revision.
        if coverage::CATALOG_PATHS[1..].contains(&name.as_str())
            && !path.exists()
            && paths.contains(coverage::CATALOG)
        {
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
    Ok((paths, impacts))
}

fn new_components(base: &str) -> Result<BTreeSet<String>> {
    let old =
        coverage::ids(&read_base(base, coverage::INVENTORY)?.context("base inventory missing")?)?;
    let data = fs::read(coverage::INVENTORY)?;
    Ok(coverage::ids(&data)?
        .into_keys()
        .filter(|id| !old.contains_key(id))
        .collect())
}

struct Change {
    paths: BTreeSet<String>,
    impacts: Impacts,
    additions: BTreeSet<String>,
}

impl Change {
    fn read(base: &str, matcher: &Matcher<'_>, problems: &mut Vec<String>) -> Result<Self> {
        let (paths, impacts) = impacts(base, matcher, problems)?;
        Ok(Self {
            paths,
            impacts,
            additions: new_components(base)?,
        })
    }

    fn credit(&mut self, item: &Evidence) {
        self.impacts
            .entry(item.area.clone())
            .or_default()
            .retain(|p| !item.paths.contains(p));
    }
}

struct Policy<'a> {
    base: &'a str,
    checks: &'a Checks,
    catalog: &'a Catalog,
    rules: &'a BTreeMap<String, Rule>,
}

pub fn report(
    base: &str,
    checks: &Checks,
    catalog: &Catalog,
    local: Option<&Path>,
    problems: &mut Vec<String>,
) -> Result<Review> {
    let rules: BTreeMap<String, Rule> = serde_json::from_slice(&fs::read(MAP)?)?;
    let matcher = Matcher::new(&rules)?;
    let policy = Policy {
        base,
        checks,
        catalog,
        rules: &rules,
    };
    let mut change = Change::read(base, &matcher, problems)?;
    if let Err(e) = fixture_migrations(base) {
        problems.push(format!("Release migrations: {e}"));
    }
    let (source, assessment) = match local {
        Some(path) => (
            path.display().to_string(),
            parse_assessment(&fs::read_to_string(path)?)?,
        ),
        None => (String::new(), Assessment::default()),
    };
    let mut rows = policy.evaluate(assessment, &source, &mut change, problems);
    missing("candidate", &change, &rules, &mut rows, problems);
    let affected: BTreeSet<_> = change
        .impacts
        .keys()
        .cloned()
        .chain(rows.iter().map(|r| r.area.clone()))
        .collect();
    Ok(Review {
        unaffected: rules
            .into_keys()
            .filter(|id| !affected.contains(id))
            .collect(),
        affected: affected.into_iter().collect(),
        rows,
    })
}

impl Policy<'_> {
    fn evaluate(
        &self,
        data: Assessment,
        record: &str,
        change: &mut Change,
        problems: &mut Vec<String>,
    ) -> Vec<ReviewRow> {
        for (id, benchmark) in data.benchmarks {
            if !change.additions.contains(&id) {
                problems.push(format!("{record}: {id} was not introduced by this change"));
                continue;
            }
            match benchmark.check() {
                Ok(()) => {
                    change.additions.remove(&id);
                }
                Err(e) => problems.push(format!("{record}: {id}: {e}")),
            }
        }
        let mut rows = Vec::new();
        for item in data.evidence {
            if let Err(e) = item.check(self.base, &change.paths, self.checks, self.catalog) {
                problems.push(format!("{record}: {e}"));
                continue;
            }
            if !self.rules.contains_key(&item.area) {
                problems.push(format!("Unknown review area {}", item.area));
                continue;
            }
            change.credit(&item);
            let evidence = if item.status == "tested" {
                record.to_owned()
            } else {
                item.source
            };
            rows.push(ReviewRow {
                area: item.area,
                outcome: item.status,
                reason: item.reason,
                evidence: Some(evidence),
                assessment: Some(record.to_owned()),
            });
        }
        rows
    }
}

fn missing(
    source: &str,
    change: &Change,
    rules: &BTreeMap<String, Rule>,
    rows: &mut Vec<ReviewRow>,
    problems: &mut Vec<String>,
) {
    for id in &change.additions {
        problems.push(format!(
            "{source}: new component {id} needs a continuous workload or obstacle and measurement"
        ));
    }
    for (area, remaining) in &change.impacts {
        if !remaining.is_empty() {
            let mut paths = remaining
                .iter()
                .take(5)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            if remaining.len() > 5 {
                paths.push_str(&format!(" (and {} more)", remaining.len() - 5));
            }
            rows.push(ReviewRow {
                area: area.clone(),
                outcome: "evidence missing".into(),
                reason: format!("{source}: {paths}"),
                evidence: None,
                assessment: None,
            });
            problems.push(format!("{source}: {area}: {}", rules[area].evidence));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: a receiver starts declaring arrival context without a controller edit.
    /// Guarantees: the live guarantee is selected from the cross-component change.
    #[test]
    fn declaration_adds_live_review() {
        let rules = serde_json::from_str(include_str!("../../../docs/conformance/review-map.json"))
            .unwrap();
        let matcher = Matcher::new(&rules).unwrap();
        let hits = matcher
            .affected(
                "rust/otap-dataflow/crates/custom/src/lib.rs",
                "impl ContextDeclarationProvider for Input",
            )
            .unwrap();
        assert!(hits.contains(&"live"));
    }

    /// Scenario: a change touches only a relocated rollout implementation or its evidence model.
    /// Guarantees: the path alone still selects the corresponding live or design review.
    #[test]
    fn relocated_rollout_sources_keep_their_review_areas() {
        let rules = serde_json::from_str(include_str!("../../../docs/conformance/review-map.json"))
            .unwrap();
        let matcher = Matcher::new(&rules).unwrap();
        for (path, area) in [
            ("crates/conformance/src/reconfiguration.rs", "live"),
            ("crates/conformance/src/fixture_session.rs", "live"),
            ("crates/conformance/model/src/rollout.rs", "design"),
        ] {
            let path = format!("{DFE}/{path}");
            assert!(
                matcher.affected(&path, "").unwrap().contains(&area),
                "{path}"
            );
        }
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
                    &Checks::from([("live-rollout".into(), passed)]),
                    &super::super::catalog().unwrap()
                )
                .is_err()
            );
        }
    }

    /// Scenario: an assessment cites a passing unit-test identifier or a preflight result.
    /// Guarantees: only a matching deployment observation supports a tested PR claim.
    #[test]
    fn unit_and_preflight_results_cannot_supply_deployed_evidence() {
        let paths = BTreeSet::from(["processor.rs".to_owned()]);
        let catalog = super::super::catalog().unwrap();
        for source in [
            "junit:crate/test",
            "backpressure-fanout",
            "released-inputs",
            "factory-validation",
        ] {
            let item = Evidence {
                area: catalog
                    .get(source)
                    .map_or("behavior", |c| c.area.as_str())
                    .into(),
                paths: paths.clone(),
                status: "tested".into(),
                source: source.into(),
                reason: "passed in ordinary CI".into(),
            };
            assert!(
                item.check(
                    "HEAD",
                    &paths,
                    &Checks::from([(source.into(), true)]),
                    &catalog
                )
                .is_err()
            );
        }
    }

    /// Scenario: a local assessment initially claims a file outside the selected change.
    /// Guarantees: a passing observation earns credit only for paths in the working-tree diff.
    #[test]
    fn assessment_is_scoped_to_the_local_diff() {
        let catalog = super::super::catalog().unwrap();
        let checks = Checks::from([("finite-id-drain".to_owned(), true)]);
        let rules = serde_json::from_str(include_str!("../../../docs/conformance/review-map.json"))
            .unwrap();
        let policy = Policy {
            base: "HEAD",
            checks: &checks,
            catalog: &catalog,
            rules: &rules,
        };
        let change = |path: &str| {
            let paths = BTreeSet::from([path.to_owned()]);
            Change {
                impacts: Impacts::from([("behavior".to_owned(), paths.clone())]),
                paths,
                additions: BTreeSet::new(),
            }
        };
        let claim = |path: &str| {
            parse_assessment(&format!(
            "evidence:\n- area: behavior\n  paths: [{path}]\n  status: tested\n  source: finite-id-drain\n  reason: delivery resumed"
        )).unwrap()
        };
        let mut candidate = change("shared.rs");
        let mut findings = Vec::new();
        policy.evaluate(
            claim("other.rs"),
            "assessment.yaml",
            &mut candidate,
            &mut findings,
        );
        assert!(!findings.is_empty());
        assert!(candidate.impacts["behavior"].contains("shared.rs"));
        findings.clear();
        policy.evaluate(
            claim("shared.rs"),
            "assessment.yaml",
            &mut candidate,
            &mut findings,
        );
        assert!(findings.is_empty());
        assert!(candidate.impacts["behavior"].is_empty());
    }
}
