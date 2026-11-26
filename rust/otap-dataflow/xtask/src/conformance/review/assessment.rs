// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validate the change-specific claims and the protected decisions they cite.
use super::{Catalog, Checks, read_base, repository_file};
use anyhow::{Context, Result, bail, ensure};
use regex::Regex;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs,
    path::Path,
};

pub(super) const EXCEPTIONS: &str = "rust/otap-dataflow/docs/conformance/exceptions";

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Assessment {
    #[serde(default)]
    pub(super) evidence: Vec<Evidence>,
    #[serde(default)]
    pub(super) benchmarks: BTreeMap<String, Benchmark>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Evidence {
    pub(super) area: String,
    pub(super) paths: BTreeSet<String>,
    pub(super) status: String,
    pub(super) source: String,
    pub(super) reason: String,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub(super) struct Benchmark {
    workload: Option<String>,
    obstacle: Option<String>,
    measurement: String,
}

fn section<'a>(doc: &'a str, heading: &str) -> Option<&'a str> {
    let (_, body) = doc.split_once(&format!("\n## {heading}\n"))?;
    Some(body.split("\n## ").next()?.trim()).filter(|text| !text.is_empty())
}
pub(super) fn decision_doc(doc: &str) -> Result<()> {
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
    pub(super) fn check(&self) -> Result<()> {
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
    pub(super) fn check(
        &self,
        base: &str,
        paths: &BTreeSet<String>,
        checks: &Checks,
        catalog: &Catalog,
    ) -> Result<()> {
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
                let area = catalog
                    .get(&self.source)
                    .filter(|c| c.scenario.is_some_and(|s| !s.preflight()))
                    .map(|c| c.area.as_str())
                    .or_else(|| self.source.starts_with("rollout:").then_some("live"))
                    .context("tested evidence must identify a deployed scenario")?;
                ensure!(area == self.area, "{} covers {area}", self.source);
            }
            "reviewed with rationale" => source(&self.source)?,
            "approved exception" => approved(base, Path::new(&self.source), &self.area)?,
            _ => bail!("unexpected evidence status {}", self.status),
        }
        Ok(())
    }
}

pub(super) fn parse_assessment(yaml: &str) -> Result<Assessment> {
    if yaml.trim().is_empty() {
        Ok(Assessment::default())
    } else {
        Ok(serde_yaml::from_str(yaml).context("invalid contract assessment YAML")?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
