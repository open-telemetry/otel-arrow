// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::Checks;
use anyhow::{Context, Result, ensure};
use quick_xml::{Reader, events::Event};
use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

type Case = (&'static str, &'static str, &'static str, &'static str);
type BuiltCase = (&'static str, &'static str, PathBuf);
const RETRY: Case = (
    "retry-recovery",
    "otel-arrow-dfe-validation",
    "core_node_liveness_tests",
    "test_retry_pipeline_eventually_recovers_after_transient_nacks",
);
const CASES: [Case; 2] = [
    (
        "release-precedence",
        "otel-arrow-dfe-config",
        "release_guarantees",
        "released_core_allocation_precedence",
    ),
    RETRY,
];

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

fn build_case((id, package, target, test): Case) -> Result<BuiltCase> {
    let mut args = vec!["test", "--no-run", "-p", package, "--test", target];
    if id == "retry-recovery" {
        args.extend(["--features", "integration-tests,crypto-ring"]);
    }
    Ok((id, test, executable(&args, target)?))
}

pub fn build_cases() -> Result<Vec<BuiltCase>> {
    CASES.into_iter().map(build_case).collect()
}

pub fn component_checks(urn: &str) -> Result<Checks> {
    let case = match urn {
        "urn:otel:processor:retry" => build_case(RETRY)?,
        "urn:otel:exporter:kafka" => (
            "kafka-topic",
            "exporters::kafka_exporter::exporter::test_support::tests::reconfigure_switches_topic",
            executable(
                &[
                    "test",
                    "--no-run",
                    "-p",
                    "otel-arrow-dfe-contrib-nodes",
                    "--lib",
                    "--features",
                    "kafka,otel-arrow-dfe-otap/crypto-ring",
                ],
                "otel_arrow_dfe_contrib_nodes",
            )?,
        ),
        _ => return Ok(Checks::new()),
    };
    local(vec![case])
}

pub fn local(cases: Vec<(&str, &str, PathBuf)>) -> Result<Checks> {
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
                "{}{}",
                String::from_utf8_lossy(&out.stdout),
                String::from_utf8_lossy(&out.stderr)
            );
        }
        checks.insert(id.to_owned(), passed);
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
pub fn junit(dir: &Path, sha: &str) -> Result<Checks> {
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
    for (id, package, target, test) in CASES {
        if let Some(&passed) = checks.get(&format!("junit:{package}::{target}/{test}")) {
            checks.insert(id.to_owned(), passed);
        }
    }
    Ok(checks)
}

#[cfg(test)]
mod tests {
    use super::*;
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
