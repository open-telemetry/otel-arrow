// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::{Catalog, Checks, DFE, DOCS, coverage, git, read_base, repository_file};
use anyhow::{Context, Result, ensure};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Write;
use std::fs;
use std::path::Path;

const MAP: &str = "rust/otap-dataflow/docs/guarantees/review-map.json";
const FIXTURES: &str = "rust/otap-dataflow/crates/config/tests/fixtures/releases";
mod assessment;
pub(super) use assessment::approved;
use assessment::{Assessment, EXCEPTIONS, Evidence, decision_doc, parse_assessment, source};

#[derive(Deserialize)]
struct Rule {
    path: String,
    lines: Option<String>,
    evidence: String,
}
fn changed(base: &str, head: Option<&str>) -> Result<(BTreeSet<String>, BTreeSet<String>)> {
    let paths = |data: Vec<u8>| -> Result<BTreeSet<String>> {
        Ok(String::from_utf8(data)?
            .split('\0')
            .filter(|s| !s.is_empty())
            .map(str::to_owned)
            .collect())
    };
    let mut args = vec!["diff", "--no-renames", "--name-only", "-z", base];
    args.extend(head);
    args.push("--");
    let mut all = paths(git(&args)?)?;
    if head.is_some() {
        return Ok((all, BTreeSet::new()));
    }
    let new = paths(git(&["ls-files", "--others", "--exclude-standard", "-z"])?)?;
    all.extend(new.iter().cloned());
    Ok((all, new))
}

fn changed_lines(base: &str, head: Option<&str>, path: &str, new: bool) -> Result<String> {
    if new {
        return Ok(String::from_utf8_lossy(&fs::read(path)?).into_owned());
    }
    let mut args = vec!["diff", "--no-ext-diff", "--no-renames", "--unified=0", base];
    args.extend(head);
    args.extend(["--", path]);
    let bytes = git(&args)?;
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
pub(super) fn candidate_run() -> Option<String> {
    let repo = std::env::var("GITHUB_REPOSITORY").ok()?;
    let run = std::env::var("GITHUB_RUN_ID").ok()?;
    Some(format!("https://github.com/{repo}/actions/runs/{run}"))
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

// The workflow resolves these revisions from GitHub metadata and verifies queue membership.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Member {
    url: String,
    base: String,
    head: String,
    yaml: String,
}

type Impacts = BTreeMap<String, BTreeSet<String>>;

fn impacts(
    base: &str,
    head: Option<&str>,
    matcher: &Matcher<'_>,
    problems: &mut Vec<String>,
) -> Result<(BTreeSet<String>, Impacts)> {
    let (paths, untracked) = changed(base, head)?;
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
        let lines = changed_lines(base, head, name, untracked.contains(name))?;
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

fn new_components(base: &str, head: Option<&str>) -> Result<BTreeSet<String>> {
    let old =
        coverage::ids(&read_base(base, coverage::INVENTORY)?.context("base inventory missing")?)?;
    let data = match head {
        Some(head) => {
            read_base(head, coverage::INVENTORY)?.context("candidate inventory missing")?
        }
        None => fs::read(coverage::INVENTORY)?,
    };
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
    fn read(
        base: &str,
        head: Option<&str>,
        matcher: &Matcher<'_>,
        problems: &mut Vec<String>,
    ) -> Result<Self> {
        let (paths, impacts) = impacts(base, head, matcher, problems)?;
        Ok(Self {
            paths,
            impacts,
            additions: new_components(base, head)?,
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
    manifest: Option<&Path>,
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
    let mut combined = Change::read(base, None, &matcher, problems)?;
    if let Err(e) = fixture_migrations(base) {
        problems.push(format!("Release migrations: {e}"));
    }
    let mut rows = Vec::new();
    if let Some(path) = manifest {
        let members: Vec<Member> =
            serde_json::from_slice(&fs::read(path).context("missing PR membership manifest")?)
                .context("incomplete or invalid PR membership manifest")?;
        ensure!(!members.is_empty(), "merge membership is missing");
        for member in members {
            ensure!(
                [&member.base, &member.head]
                    .iter()
                    .all(|oid| oid.len() == 40 && oid.bytes().all(|c| c.is_ascii_hexdigit())),
                "invalid PR revision"
            );
            source(&member.url)?;
            ensure!(
                member.url.starts_with("https://github.com/"),
                "expected a PR URL"
            );
            let mut own = Change::read(&member.base, Some(&member.head), &matcher, problems)?;
            let reviewed = policy.evaluate(
                parse_assessment(&member.yaml)?,
                &member.url,
                &mut own,
                problems,
            );
            for claim in &reviewed.claims {
                combined.credit(claim);
            }
            combined
                .additions
                .retain(|id| !reviewed.benchmarks.contains(id));
            missing(&member.url, &own, &rules, &mut rows, problems);
            rows.extend(reviewed.rows);
        }
    } else {
        let (source, data) = match local {
            Some(path) => (
                path.display().to_string(),
                parse_assessment(&fs::read_to_string(path)?)?,
            ),
            None => (String::new(), Assessment::default()),
        };
        let reviewed = policy.evaluate(data, &source, &mut combined, problems);
        rows.extend(reviewed.rows);
    }
    missing("candidate", &combined, &rules, &mut rows, problems);
    let affected: BTreeSet<_> = combined
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

struct Reviewed {
    rows: Vec<ReviewRow>,
    claims: Vec<Evidence>,
    benchmarks: BTreeSet<String>,
}

impl Policy<'_> {
    fn evaluate(
        &self,
        data: Assessment,
        record: &str,
        change: &mut Change,
        problems: &mut Vec<String>,
    ) -> Reviewed {
        let mut benchmarks = BTreeSet::new();
        for (id, benchmark) in data.benchmarks {
            if !change.additions.contains(&id) {
                problems.push(format!("{record}: {id} was not introduced by this change"));
                continue;
            }
            match benchmark.check() {
                Ok(()) => {
                    change.additions.remove(&id);
                    benchmarks.insert(id);
                }
                Err(e) => problems.push(format!("{record}: {id}: {e}")),
            }
        }
        let mut claims = Vec::new();
        let mut rows = Vec::new();
        for item in data.evidence {
            if let Err(e) = item.check(self.base, &change.paths, self.checks, self.catalog) {
                problems.push(format!("{record}: {e}"));
                continue;
            }
            if !self.rules.contains_key(&item.area) {
                problems.push(format!("Unknown guarantee {}", item.area));
                continue;
            }
            change.credit(&item);
            let evidence = if item.status == "tested" {
                candidate_run().unwrap_or_else(|| record.to_owned())
            } else {
                link(&item.source)
            };
            rows.push(ReviewRow {
                area: item.area.clone(),
                outcome: item.status.clone(),
                reason: item.reason.clone(),
                evidence: Some(evidence),
                assessment: Some(record.to_owned()),
            });
            claims.push(item);
        }
        Reviewed {
            rows,
            claims,
            benchmarks,
        }
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
                    &Checks::from([("live-rollout".into(), passed)]),
                    &super::super::catalog().unwrap()
                )
                .is_err()
            );
        }
    }

    /// Scenario: CI supplies a passing unit-test identifier or a preflight result.
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

    /// Scenario: two queued PRs touch the same file and the first also claims a sibling-only path.
    /// Guarantees: each change needs its own valid evidence even when the combined candidate passed.
    #[test]
    fn assessments_are_scoped_to_each_member() {
        let catalog = super::super::catalog().unwrap();
        let checks = Checks::from([("finite-id-drain".to_owned(), true)]);
        let rules =
            serde_json::from_str(include_str!("../../../docs/guarantees/review-map.json")).unwrap();
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
        let mut first = change("shared.rs");
        let mut second = change("shared.rs");
        let mut findings = Vec::new();
        policy.evaluate(claim("other.rs"), "PR 1", &mut first, &mut findings);
        assert!(!findings.is_empty());
        findings.clear();
        policy.evaluate(claim("shared.rs"), "PR 1", &mut first, &mut findings);
        policy.evaluate(Assessment::default(), "PR 2", &mut second, &mut findings);
        let mut rows = Vec::new();
        missing("PR 1", &first, &rules, &mut rows, &mut findings);
        assert!(findings.is_empty());
        missing("PR 2", &second, &rules, &mut rows, &mut findings);
        assert_eq!(rows.len(), 1);
        assert!(rows[0].reason.contains("PR 2: shared.rs"));
    }
}
