// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use super::Checks;
use anyhow::{Context, Result, ensure};
use quick_xml::{Reader, events::Event};
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
    process::{Command, Stdio},
};

struct Case {
    id: &'static str,
    package: &'static str,
    // None selects the library test binary.
    integration: Option<&'static str>,
    features: &'static str,
    components: &'static [&'static str],
    tests: &'static [&'static str],
}

const CORE: &str = "otel-arrow-dfe-core-nodes";
const RING: &str = "otel-arrow-dfe-otap/crypto-ring";
const CASES: &[Case] = &[
    Case {
        id: "release-precedence",
        package: "otel-arrow-dfe-config",
        integration: Some("release_guarantees"),
        features: "",
        components: &[],
        tests: &["released_core_allocation_precedence"],
    },
    Case {
        id: "retry-recovery",
        package: "otel-arrow-dfe-validation",
        integration: Some("core_node_liveness_tests"),
        features: "integration-tests,crypto-ring",
        components: &["urn:otel:processor:retry"],
        tests: &["test_retry_pipeline_eventually_recovers_after_transient_nacks"],
    },
    Case {
        id: "backpressure-fanout",
        package: CORE,
        integration: None,
        features: RING,
        components: &["urn:otel:processor:fanout"],
        tests: &[
            "processors::fanout_processor::tests::max_inflight_pauses_pdata_when_limit_reached",
            "processors::fanout_processor::tests::no_nacks_at_max_inflight",
        ],
    },
    Case {
        id: "backpressure-content-router",
        package: CORE,
        integration: None,
        features: RING,
        components: &["urn:otel:processor:content_router"],
        tests: &[
            "processors::content_router::tests::admission_policy::test_backpressure_pauses_on_any_blocked_route_and_resumes_after_recovery",
        ],
    },
    Case {
        id: "backpressure-type-router",
        package: CORE,
        integration: None,
        features: RING,
        components: &["urn:otel:processor:type_router"],
        tests: &[
            "processors::signal_type_router::tests::admission_policy::test_backpressure_pauses_on_any_blocked_route_and_resumes_after_recovery",
        ],
    },
    Case {
        id: "wire-fidelity",
        package: "otel-arrow-dfe-pdata",
        integration: None,
        features: "",
        components: &["urn:otel:receiver:otap", "urn:otel:exporter:otap"],
        tests: &["views::otap::logs::tests::test_composite_bodies_survive_otap_wire_round_trip"],
    },
    Case {
        id: "reconfiguration-planning",
        package: "otel-arrow-dfe-controller",
        integration: None,
        features: RING,
        components: &[],
        tests: &[
            "live_control::tests::prepare_rollout_plan_returns_noop_for_identical_active_pipeline",
            "live_control::tests::prepare_rollout_plan_rejects_concurrent_rollout_for_same_pipeline",
            "live_control::tests::rollback_replace_rollout_restores_recovered_serving_generation",
        ],
    },
];
const KAFKA: Case = Case {
    id: "kafka-topic",
    package: "otel-arrow-dfe-contrib-nodes",
    integration: None,
    features: "kafka,otel-arrow-dfe-otap/crypto-ring",
    components: &["urn:otel:exporter:kafka"],
    tests: &[
        "exporters::kafka_exporter::exporter::test_support::tests::reconfigure_switches_topic",
    ],
};

pub(super) fn test_names() -> impl Iterator<Item = (&'static str, &'static [&'static str])> {
    CASES
        .iter()
        .chain([&KAFKA])
        .map(|case| (case.id, case.tests))
}

pub fn required() -> impl Iterator<Item = &'static str> {
    CASES.iter().map(|case| case.id)
}

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

type BuiltCase = (&'static str, &'static str, PathBuf);

pub fn build_cases(component: Option<&str>) -> Result<Vec<BuiltCase>> {
    let mut binaries = BTreeMap::new();
    let mut cases = Vec::new();
    for case in CASES.iter().chain(component.map(|_| &KAFKA)) {
        if component.is_some_and(|urn| !case.components.contains(&urn)) {
            continue;
        }
        let key = (case.package, case.integration, case.features);
        let bin = match binaries.entry(key) {
            std::collections::btree_map::Entry::Occupied(entry) => entry.into_mut(),
            std::collections::btree_map::Entry::Vacant(entry) => {
                let mut args = vec!["test", "--no-run", "-p", case.package];
                let lib = case.package.replace('-', "_");
                let target = if let Some(target) = case.integration {
                    args.extend(["--test", target]);
                    target
                } else {
                    args.push("--lib");
                    &lib
                };
                if !case.features.is_empty() {
                    args.extend(["--features", case.features]);
                }
                entry.insert(executable(&args, target)?)
            }
        };
        for &test in case.tests {
            cases.push((case.id, test, bin.clone()));
        }
    }
    Ok(cases)
}

pub fn local(cases: Vec<BuiltCase>) -> Result<Checks> {
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
    summarize_cases(&mut checks);
    Ok(checks)
}

fn summarize_cases(checks: &mut Checks) {
    for case in CASES {
        let class = case.integration.map_or_else(
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
            let _ = checks.insert(case.id.to_owned(), passed);
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
        let case = CASES
            .iter()
            .find(|case| case.id == "backpressure-fanout")
            .unwrap();
        let mut checks = Checks::new();
        let key = |name| format!("junit:{CORE}/{name}");
        let _ = checks.insert(key(case.tests[0]), true);
        summarize_cases(&mut checks);
        assert!(!checks[case.id]);
        let _ = checks.insert(key(case.tests[1]), false);
        summarize_cases(&mut checks);
        assert!(!checks[case.id]);
        let _ = checks.insert(key(case.tests[1]), true);
        summarize_cases(&mut checks);
        assert!(checks[case.id]);
        assert!(!checks.contains_key("backpressure-type-router"));
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
