// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Presentation of evaluated evidence. Renderers only borrow the result.
use super::{DOCS, coverage, html, review};
use anyhow::{Result, ensure};
use otel_arrow_dfe_conformance_model::{
    Catalog, ComponentReport, Observations, Operation, OperationOutcome, Rollout,
};
use serde::Serialize;
use std::collections::BTreeMap;
use std::{fmt::Write, fs, path::Path};

#[derive(Serialize)]
pub(super) struct OperationGroup<'a> {
    title: String,
    rows: Vec<(&'a str, &'a Rollout)>,
}

pub(super) fn operation_groups(rollouts: &BTreeMap<String, Rollout>) -> Vec<OperationGroup<'_>> {
    let mut groups = BTreeMap::new();
    for (urn, rollout) in rollouts {
        let namespace = crate::component_inventory::component_namespace(urn);
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

pub(super) struct Report<'a> {
    pub commit: &'a str,
    pub observed: Observations,
    pub coverage: Option<coverage::CoverageReport>,
    pub review: Option<review::Review>,
    pub blocking_findings: Vec<String>,
    pub elapsed_seconds: f64,
}

#[derive(Serialize)]
struct Check<'a> {
    name: &'a str,
    passed: Option<bool>,
    status: &'static str,
    gap: Option<&'a otel_arrow_dfe_conformance_model::DeploymentGap>,
    error: Option<&'a String>,
}

pub(super) fn print_component(report: &ComponentReport) {
    for problem in report.blocking_findings() {
        println!("FAIL {problem}");
    }
    println!(
        "Component {}\nFactory configuration preflight: {}",
        report.component,
        match report.factory_validation {
            Some(true) => "passed",
            Some(false) => "failed",
            None => "unexercised",
        }
    );
    if report.released_inputs.is_empty() {
        println!("No released configuration checked");
    }
    for (path, result) in &report.released_inputs {
        println!("Released configuration {path}: {}", result.label());
    }
    if let Some(error) = report.check_errors.get("released-inputs") {
        println!("{error}");
    }
    if let Some(ops) = &report.operations {
        for op in Operation::names() {
            println!(
                "{op}: {}",
                ops.get(op).map_or("missing", OperationOutcome::label)
            );
        }
    } else {
        println!("Controller extension: startup validation only");
    }
    for (id, passed) in &report.behavior_checks {
        println!("{id}: {}", if *passed { "passed" } else { "failed" });
        if let Some(reason) = report.check_errors.get(id) {
            println!("{reason}");
        }
    }
    for id in &report.unexercised_checks {
        println!("{id}: unexercised");
    }
    for (id, gap) in &report.deployment_gaps {
        println!(
            "{id}: not yet validated in deployment ({}: {})",
            gap.owner, gap.follow_up
        );
    }
}

impl Report<'_> {
    pub fn output(
        &self,
        root: &Path,
        catalog: &Catalog,
        base: Option<&str>,
        metrics: Option<&Path>,
        page: Option<&Path>,
        open: bool,
    ) -> Result<()> {
        let checks: Vec<_> = catalog
            .iter()
            .filter(|(_, c)| c.selected(None) || c.gap.is_some())
            .map(|(id, contract)| Check {
                gap: contract.gap.as_ref(),
                status: if contract.gap.is_some() {
                    "not yet validated in deployment"
                } else if self.observed.checks.get(id) == Some(&false) {
                    "failed"
                } else if !self.observed.checks.contains_key(id) {
                    "evidence missing"
                } else if contract.scenario.is_some_and(|s| s.preflight()) {
                    "preflight passed"
                } else {
                    "tested"
                },
                name: id,
                passed: self.observed.checks.get(id).copied(),
                error: self.observed.check_errors.get(id),
            })
            .collect();
        let groups = operation_groups(&self.observed.rollouts);
        let support = format!("{DOCS}/live-support.md");
        let passed = self.blocking_findings.is_empty();
        let mut summary = String::new();
        for problem in &self.blocking_findings {
            writeln!(summary, "FAIL {problem}")?;
        }
        summary.push_str("\n## Conformance checks\n\n");
        for check in &checks {
            writeln!(summary, "- **{}** {}", check.status, check.name)?;
            if let Some(gap) = check.gap {
                writeln!(summary, "  - {}: {}", gap.owner, gap.follow_up)?;
            }
        }
        for signal in &self.observed.ui_signals {
            writeln!(
                summary,
                "- {} / {}: {}",
                signal.consumer, signal.signal, signal.result
            )?;
        }
        for group in &groups {
            writeln!(
                summary,
                "\n### {}\n\n| Component | Add | Change | Remove |\n| --- | --- | --- | --- |",
                group.title
            )?;
            for (urn, row) in &group.rows {
                write!(summary, "| `{urn}` |")?;
                for op in Operation::names() {
                    write!(
                        summary,
                        " {} |",
                        row.operations
                            .get(op)
                            .map_or("missing", OperationOutcome::label)
                    )?;
                }
                writeln!(summary)?;
            }
        }
        if let Some(c) = &self.coverage {
            writeln!(
                summary,
                "\n{} components in source, {} linked in the conformance driver. {} of {} required operations have executable coverage, {} exercised here.",
                c.source_components.len(),
                c.linked_validators.len(),
                c.scenario_linked_operations,
                c.registered_operations,
                c.exercised_operations
            )?;
            writeln!(
                summary,
                "\n### Released configuration preflight by component\n"
            )?;
            for (urn, inputs) in &c.released_configurations {
                if inputs.is_empty() {
                    writeln!(summary, "- `{urn}`: no released configuration checked")?;
                } else {
                    for (path, result) in inputs {
                        writeln!(
                            summary,
                            "- `{urn}` / {path}: {}",
                            result.as_ref().map_or("unexercised", |r| r.label())
                        )?;
                    }
                }
            }
            for urn in &c.unlinked_components {
                writeln!(summary, "- `{urn}`: not linked in this build")?;
            }
            for urn in &c.startup_only {
                writeln!(summary, "- `{urn}`: controller extension, startup only")?;
            }
            for (urn, reason) in &c.deferred_fixtures {
                writeln!(summary, "- `{urn}` unexercised ({reason})")?;
            }
            if !c.components_without_live_fixture.is_empty() {
                writeln!(
                    summary,
                    "\n### {} ({} components)\n",
                    coverage::NO_LIVE_FIXTURE,
                    c.components_without_live_fixture.len()
                )?;
                for urn in &c.components_without_live_fixture {
                    writeln!(
                        summary,
                        "- `{urn}`: {}",
                        c.fixture_gap_reasons
                            .get(urn)
                            .map_or("no fixture", String::as_str)
                    )?;
                }
            }
            writeln!(
                summary,
                "\n[Coverage by component and open gaps](<{support}>). Owner: {}.",
                c.gap_owner
            )?;
        }
        if let Some(r) = &self.review {
            r.write_summary(&mut summary)?;
        }
        let elapsed = format!("{:.2}", self.elapsed_seconds);
        println!("{summary}\nExecution after build: {elapsed}s");
        if let Some(path) = metrics {
            fs::write(
                path,
                serde_json::to_vec_pretty(&serde_json::json!({
                    "commit": self.commit, "seed_results": self.observed.checks, "checks": checks, "ui_signals": self.observed.ui_signals, "coverage": self.coverage,
                    "rollouts": self.observed.rollouts, "review": self.review,
                    "affected_areas": self.review.as_ref().map_or(&[][..], |r| r.affected.as_slice()),
                    "blocking_findings": self.blocking_findings, "elapsed_seconds": self.elapsed_seconds,
                }))?,
            )?;
        }
        if let Some(path) = page {
            let follow_up = self
                .coverage
                .as_ref()
                .map(|c| format!("{DOCS}/{}", c.follow_up));
            let issues = html::FindingGroups::new(
                &self.blocking_findings,
                &self.observed.rollouts,
                self.coverage
                    .as_ref()
                    .map_or(&[][..], |c| c.missing_evidence.as_slice()),
            );
            html::export(
                path,
                open,
                root,
                minijinja::context!(commit => self.commit, passed, issues, finding_count => self.blocking_findings.len(), dashboard => html::DashboardRows::from(self.observed.ui_signals.as_slice()), checks, coverage => self.coverage, groups, elapsed, review => self.review, base, follow_up, support),
            )?;
        }
        ensure!(
            passed,
            "{} blocking conformance finding(s)",
            self.blocking_findings.len()
        );
        Ok(())
    }
}
