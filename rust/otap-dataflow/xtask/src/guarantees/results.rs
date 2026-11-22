// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_contract_model::{Catalog, Checks};
use quick_xml::{Reader, events::Event};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

pub fn executable(args: &[&str], name: &str) -> Result<PathBuf> {
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

type BuiltCase<'a> = (&'a str, &'a str, PathBuf);

pub fn build_cases<'a>(
    catalog: &'a Catalog,
    component: Option<(&str, &str)>,
) -> Result<Vec<BuiltCase<'a>>> {
    let mut binaries = BTreeMap::new();
    let mut cases = Vec::new();
    for (id, contract) in catalog {
        let Some(case) = &contract.cargo else {
            continue;
        };
        if !contract.selected(component) {
            continue;
        }
        let key = (&case.package, &case.target, &case.features);
        let bin = match binaries.entry(key) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let mut args = vec!["test", "--no-run", "-p", case.package.as_str()];
                let lib = case.package.replace('-', "_");
                let target = if let Some(target) = case.target.as_deref() {
                    args.extend(["--test", target]);
                    target
                } else {
                    args.push("--lib");
                    &lib
                };
                if !case.features.is_empty() {
                    args.extend(["--features", &case.features]);
                }
                entry.insert(executable(&args, target)?)
            }
        };
        for test in &case.tests {
            cases.push((id.as_str(), test.as_str(), bin.clone()));
        }
    }
    Ok(cases)
}

pub fn local(cases: Vec<BuiltCase<'_>>) -> Result<Checks> {
    let mut checks = Checks::new();
    for (id, test, bin) in cases {
        let out = Command::new(bin)
            .args(["--exact", test, "--format", "pretty"])
            .output()?;
        let passed = out.status.success()
            && String::from_utf8_lossy(&out.stdout)
                .lines()
                .any(|line| line == format!("test {test} ... ok"));
        if !passed {
            eprintln!(
                "{id}: no passing result for {test}\n{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        *checks.entry(id.to_owned()).or_insert(true) &= passed;
    }
    Ok(checks)
}

fn parse(xml: &str, checks: &mut Checks) -> Result<()> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().expand_empty_elements = true;
    let mut case = None;
    loop {
        match reader.read_event()? {
            Event::Start(tag) if tag.name().as_ref() == "testcase" => {
                let mut class = None;
                let mut name = None;
                for attr in tag.attributes() {
                    let attr = attr?;
                    let value = quick_xml::escape::unescape(&attr.value)?.into_owned();
                    match attr.key.as_ref() {
                        "classname" => class = Some(value),
                        "name" => name = Some(value),
                        _ => (),
                    }
                }
                let key = format!(
                    "junit:{}/{}",
                    class.context("testcase classname")?,
                    name.context("testcase name")?
                );
                case = Some((key, true));
            }
            Event::Start(tag)
                if matches!(
                    tag.name().as_ref(),
                    "failure"
                        | "error"
                        | "skipped"
                        | "rerunFailure"
                        | "rerunError"
                        | "flakyFailure"
                        | "flakyError"
                ) =>
            {
                if let Some((_, passed)) = &mut case {
                    *passed = false;
                }
            }
            Event::End(tag) if tag.name().as_ref() == "testcase" => {
                let (key, passed) = case.take().context("unmatched testcase")?;
                *checks.entry(key).or_insert(true) &= passed;
            }
            Event::Eof => {
                ensure!(case.is_none(), "incomplete JUnit testcase");
                break;
            }
            _ => (),
        }
    }
    Ok(())
}
pub fn junit(dir: &Path, sha: &str, catalog: &Catalog) -> Result<Checks> {
    fn visit(dir: &Path, checks: &mut Checks, sha: &str) -> Result<()> {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.file_type()?.is_dir() {
                visit(&entry.path(), checks, sha)?;
            } else if entry.file_name() == "junit.xml" {
                let meta: serde_json::Value =
                    serde_json::from_slice(&fs::read(dir.join("metadata.json"))?)?;
                ensure!(
                    meta["commit"].as_str() == Some(sha),
                    "JUnit artifact came from a different commit"
                );
                parse(&fs::read_to_string(entry.path())?, checks)?;
            }
        }
        Ok(())
    }
    let mut checks = Checks::new();
    visit(dir, &mut checks, sha)?;
    summarize_cases(&mut checks, catalog);
    Ok(checks)
}

fn summarize_cases(checks: &mut Checks, catalog: &Catalog) {
    for (id, contract) in catalog {
        let Some(case) = &contract.cargo else {
            continue;
        };
        if contract.focused_only {
            continue;
        }
        let class = case.target.as_deref().map_or_else(
            || case.package.to_owned(),
            |target| format!("{}::{target}", case.package),
        );
        let mut results = case
            .tests
            .iter()
            .map(|test| checks.get(&format!("junit:{class}/{test}")));
        // A partial report cannot establish a multi-test guarantee. An absent
        // guarantee may still be exercised independently by the native suite.
        if results.clone().any(|r| r.is_some()) {
            let passed = results.all(|r| r == Some(&true));
            let _ = checks.insert(id.to_owned(), passed);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    /// Scenario: only one fan-out test is present, or its sibling was skipped or retried.
    /// Guarantees: CI credits backpressure only when both exact owning-library tests passed cleanly.
    #[test]
    fn multi_case_evidence_requires_every_member() {
        let catalog = &otel_arrow_dfe_contract_model::catalog().unwrap();
        let id = "backpressure-fanout";
        let case = catalog[id].cargo.as_ref().unwrap();
        let mut checks = Checks::new();
        let key = |name: &str| format!("junit:{}/{name}", case.package);
        let _ = checks.insert(key(&case.tests[0]), true);
        summarize_cases(&mut checks, catalog);
        assert!(!checks[id]);
        let _ = checks.insert(key(&case.tests[1]), false);
        summarize_cases(&mut checks, catalog);
        assert!(!checks[id]);
        let _ = checks.insert(key(&case.tests[1]), true);
        summarize_cases(&mut checks, catalog);
        assert!(checks[id]);
        assert!(!checks.contains_key("backpressure-type-router"));
    }

    /// Scenario: a report directory contains a successful test from another commit.
    /// Guarantees: candidate-specific replay rejects the artifact before crediting any case.
    #[test]
    fn rejects_wrong_revision_junit() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join("junit.xml"),
            r#"<testcase classname="p" name="t"/>"#,
        )
        .unwrap();
        fs::write(dir.path().join("metadata.json"), r#"{"commit":"other"}"#).unwrap();
        let catalog = otel_arrow_dfe_contract_model::catalog().unwrap();
        assert!(junit(dir.path(), "candidate", &catalog).is_err());
    }

    /// Scenario: a partition reports a skipped, retried or failed case and another reports success.
    /// Guarantees: no partition can turn that result or an absent case into passing evidence.
    #[test]
    fn failures_survive_aggregation() {
        for outcome in ["skipped", "error", "failure", "flakyFailure", "rerunError"] {
            let mut checks = Checks::new();
            parse(&format!(r#"<testsuite><testcase classname="p" name="t"><{outcome}/></testcase><testcase classname="p" name="t"/></testsuite>"#),&mut checks).unwrap();
            assert_eq!(checks.get("junit:p/t"), Some(&false));
            parse(r#"<testcase classname="another" name="t"/>"#, &mut checks).unwrap();
            assert_eq!(checks.get("junit:another/t"), Some(&true));
            assert_eq!(checks.get("junit:p/t"), Some(&false));
            assert!(!checks.contains_key("junit:p/missing"));
        }
    }
}
