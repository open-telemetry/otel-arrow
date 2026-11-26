// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validate native deployment observations before reporting them.
use super::DFE;
use super::coverage;
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_conformance_model::releases::{self, Outcome};
use otel_arrow_dfe_conformance_model::{Observations, Operation, OperationOutcome};
use std::path::Path;

/// Validate each job separately before combining jobs with different linked features or prerequisites.
pub(super) fn verify_artifact(root: &Path, report: &Observations) -> Result<()> {
    if report.checks.get("factory-validation") == Some(&true) {
        ensure!(
            !report.linked_validators.is_empty()
                && report.linked_validators.values().all(|passed| *passed),
            "passing inventory preflight requires linked validator observations"
        );
    }
    if report.checks.contains_key("released-inputs") || !report.released_inputs.is_empty() {
        let workspace = root.join(DFE);
        let expected = releases::corpus(&workspace)?;
        let migrations = releases::migrations(&workspace, &expected)?;
        for (path, result) in &report.released_inputs {
            ensure!(expected.contains_key(path), "unknown released input {path}");
            let migration = migrations.get(path);
            let consistent = match result {
                Outcome::Accepted => migration.is_none(),
                Outcome::Migrated { replacement } => {
                    migration.is_some_and(|m| &m.replacement == replacement)
                        && report.released_inputs.get(replacement) == Some(&Outcome::Accepted)
                }
                Outcome::Failed { reason } => !reason.trim().is_empty(),
            };
            ensure!(consistent, "inconsistent release evidence for {path}");
        }
        if report.checks.get("released-inputs") == Some(&true) {
            ensure!(
                expected.keys().eq(report.released_inputs.keys())
                    && report
                        .released_inputs
                        .values()
                        .all(|result| !matches!(result, Outcome::Failed { .. })),
                "passing release preflight requires a successful observation for every pinned input"
            );
        }
    }
    let catalog = otel_arrow_dfe_conformance_model::catalog()?;
    for (urn, scenario) in otel_arrow_dfe_conformance_model::REQUIRED_ROLLOUTS {
        if catalog
            .iter()
            .any(|(id, c)| c.scenario == Some(scenario) && report.checks.get(id) == Some(&true))
        {
            ensure!(
                report.rollouts.get(urn).is_some_and(|r| r.error.is_none()),
                "missing required passing deployment for {urn}"
            );
        }
    }
    if report.checks.get("embedded-ui") == Some(&true) {
        use otel_arrow_dfe_conformance_model::ui::{Inputs, SIGNAL_CATALOG};
        Inputs::parse(SIGNAL_CATALOG)?.verify_success(&report.ui_platform, &report.ui_signals)?;
    }
    let scheduled = coverage::scheduled_fixtures(root, report)?;
    ensure!(
        scheduled.iter().eq(report.rollouts.keys()),
        "live results differ from the fixtures scheduled for this build"
    );
    for (urn, row) in &report.rollouts {
        let mut stopped = false;
        ensure!(
            row.operations.len() == Operation::ALL.len(),
            "{urn}: incomplete operation set"
        );
        for op in Operation::names() {
            let state = row
                .operations
                .get(op)
                .with_context(|| format!("{urn}: missing {op}"))?;
            ensure!(
                !stopped || *state == OperationOutcome::Unexercised,
                "{urn}: {op} reported after a failed or unexercised operation"
            );
            stopped = *state != OperationOutcome::Passed;
        }
        ensure!(
            row.error.is_some() == stopped,
            "{urn}: inconsistent fixture outcome"
        );
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::coverage::CATALOG;
    use super::*;
    use std::fs;

    /// Scenario: two runs link a broker fixture, but only one has the broker available.
    /// Guarantees: only the eligible run owes results, and an omitted row or a pass after a failed stage is rejected.
    #[test]
    fn verify_scheduled_operations_per_run() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join(CATALOG);
        fs::create_dir_all(path.parent().unwrap()).unwrap();
        fs::write(
            path,
            r#"[{"urn":"test","config":{},"probe":{},"requires":["broker"]}]"#,
        )
        .unwrap();
        let mut report = Observations::default();
        let _ = report.linked_validators.insert("test".into(), true);
        verify_artifact(dir.path(), &report).unwrap();
        let _ = report.prerequisites.insert("broker".into());
        assert!(verify_artifact(dir.path(), &report).is_err());
        report.rollouts = serde_json::from_value(serde_json::json!({"test":{"operations":{
            "add":{"status":"failed","reason":"refused"},"change":{"status":"passed"},"remove":{"status":"unexercised"}
        },"error":"refused"}})).unwrap();
        assert!(verify_artifact(dir.path(), &report).is_err());
        let _ = report
            .rollouts
            .get_mut("test")
            .unwrap()
            .operations
            .insert("change".into(), OperationOutcome::Unexercised);
        verify_artifact(dir.path(), &report).unwrap();
    }
}
