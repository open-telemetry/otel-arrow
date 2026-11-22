// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Presentation of evaluated evidence. Renderers only borrow the result.
use super::{ComponentReport, DOCS, Observations, coverage, html, operation_groups, review};
use anyhow::{Result, ensure};
use otel_arrow_dfe_contract_model::{Catalog, OperationOutcome};
use serde::Serialize;
use std::{fmt::Write, fs, path::Path};

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
    gap: Option<&'a otel_arrow_dfe_contract_model::DeploymentGap>,
    error: Option<&'a String>,
}

pub(super) fn print_component(report: &ComponentReport) {
    println!(
        "Component {}\nFactory configuration preflight: {}",
        report.component,
        match report.factory_validation {
            Some(true) => "passed",
            Some(false) => "failed",
            None => "unexercised",
        }
    );
    if let Some(ops) = &report.operations {
        for op in coverage::OPERATIONS {
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
    if let Some(detail) = &report.finding {
        println!("{detail}");
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
        let candidate_run = review::candidate_run();
        let support = review::link(&format!("{DOCS}/live-support.md"));
        let passed = self.blocking_findings.is_empty();
        let mut summary = String::from("## Operator guarantees\n\n");
        if let Some(url) = &candidate_run {
            writeln!(summary, "[Candidate test run]({url})\n")?;
        }
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
                for op in coverage::OPERATIONS {
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
                "\n{} components in source, {} linked in the validation runner. {} of {} required operations have executable coverage, {} exercised here.",
                c.source_components.len(),
                c.linked_validators.len(),
                c.scenario_linked_operations,
                c.registered_operations,
                c.exercised_operations
            )?;
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
                    writeln!(summary, "- `{urn}`")?;
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
        for problem in &self.blocking_findings {
            writeln!(summary, "\n- {problem}")?;
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
                .map(|c| review::link(&format!("{DOCS}/{}", c.follow_up)));
            html::export(
                path,
                open,
                root,
                minijinja::context!(commit => self.commit, passed, findings => self.blocking_findings, ui_signals => self.observed.ui_signals, checks, coverage => self.coverage, groups, elapsed, review => self.review, base, candidate_run, follow_up, support),
            )?;
        }
        ensure!(
            passed,
            "{} guarantee finding(s)",
            self.blocking_findings.len()
        );
        Ok(())
    }
}
