// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Validate a deployment artifact before admitting it as evidence. Replay requires a clean checkout at the recorded revision.
use super::{DFE, coverage, git};
use anyhow::{Context, Result, ensure};
use otel_arrow_dfe_conformance_model::{
    Catalog, Observations, Operation, OperationOutcome,
    releases::{self, Outcome},
};
use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn import(
    root: &Path,
    sha: &str,
    files: &[PathBuf],
    catalog: &Catalog,
    observed: &mut Observations,
) -> Result<()> {
    ensure!(
        git(&["status", "--porcelain", "--untracked-files=normal"])?.is_empty(),
        "saved observations require a clean checkout"
    );
    for file in files {
        let replay: Observations = serde_json::from_slice(
            &fs::read(file).with_context(|| format!("missing {}", file.display()))?,
        )?;
        ensure!(
            replay.commit == sha,
            "observations came from a different commit"
        );
        verify_artifact(root, &replay)?;
        merge(observed, replay, catalog);
    }
    Ok(())
}

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

// A failed or incomplete observation stays failed when another environment also ran it.
fn merge(observed: &mut Observations, other: Observations, catalog: &Catalog) {
    for (id, _) in catalog
        .iter()
        .filter(|(_, c)| c.scenario.is_some() && !c.focused_only)
    {
        *observed.checks.entry(id.into()).or_insert(true) &= other.checks.get(id) == Some(&true);
    }
    for (urn, passed) in other.linked_validators {
        *observed.linked_validators.entry(urn).or_insert(true) &= passed;
    }
    for (path, result) in other.released_inputs {
        // Retain a failing platform's result even if another candidate build accepted the file.
        let entry = observed
            .released_inputs
            .entry(path)
            .or_insert_with(|| result.clone());
        if matches!(result, Outcome::Failed { .. }) {
            *entry = result;
        }
    }
    observed.ui_signals.extend(other.ui_signals);
    observed.check_errors.extend(other.check_errors);
    observed.prerequisites.extend(other.prerequisites);
    for (urn, mut result) in other.rollouts {
        match observed.rollouts.entry(urn) {
            std::collections::btree_map::Entry::Vacant(slot) => {
                let _ = slot.insert(result);
            }
            std::collections::btree_map::Entry::Occupied(mut slot) => {
                let existing = slot.get_mut();
                if existing.error.is_none() {
                    existing.error = result.error;
                }
                for op in Operation::names() {
                    let state = existing
                        .operations
                        .entry(op.into())
                        .or_insert(OperationOutcome::Unexercised);
                    let incoming = result
                        .operations
                        .remove(op)
                        .unwrap_or(OperationOutcome::Unexercised);
                    if matches!(state, OperationOutcome::Passed)
                        || matches!(incoming, OperationOutcome::Failed { .. })
                    {
                        *state = incoming;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::coverage::CATALOG;
    use super::*;
    use otel_arrow_dfe_conformance_model::catalog;
    use serde_json::json;

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

    /// Scenario: a saved report claims released-inputs passed but omits a fixture or labels a failed fixture as covered.
    /// Guarantees: replay requires an observation matching every pinned file and keeps a failure when merging builds.
    #[test]
    fn release_headline_requires_each_fixture() {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..");
        let mut report = Observations::default();
        let _ = report.checks.insert("released-inputs".into(), true);
        assert!(verify_artifact(&root, &report).is_err());
        report.released_inputs = releases::corpus(&root.join(DFE))
            .unwrap()
            .into_keys()
            .map(|path| (path, Outcome::Accepted))
            .collect();
        verify_artifact(&root, &report).unwrap();
        let file = report.released_inputs.keys().next().unwrap().clone();
        let mut failed = Observations::default();
        let _ = failed.released_inputs.insert(
            file.clone(),
            Outcome::Failed {
                reason: "renamed setting".into(),
            },
        );
        merge(&mut failed, report, &catalog().unwrap());
        assert!(matches!(
            failed.released_inputs[&file],
            Outcome::Failed { .. }
        ));
        let _ = failed.checks.insert("released-inputs".into(), true);
        assert!(verify_artifact(&root, &failed).is_err());
    }

    /// Scenario: two environments report the same add, change and remove with a failure and a missing case in one.
    /// Guarantees: passing evidence remains per operation and another run cannot erase a failure or fill a skipped stage.
    #[test]
    fn merge_preserves_partial_rollouts_in_either_order() {
        let complete = json!({"add":{"status":"passed"},"change":{"status":"passed"},"remove":{"status":"passed"}});
        let partial =
            json!({"add":{"status":"passed"},"change":{"status":"failed","reason":"refused"}});
        for reports in [[&complete, &partial], [&partial, &complete]] {
            let mut merged = Observations::default();
            for operations in reports {
                let report = json!({"commit":"candidate", "ui_platform":{"os":"linux","atomic64":true}, "ui_signals":[], "checks":{"live-rollout":operations == &complete}, "linked_validators":{"urn":true},
                    "prerequisites":[], "rollouts":{"urn":{"operations":operations,"error":null}}});
                merge(
                    &mut merged,
                    serde_json::from_value(report).unwrap(),
                    &catalog().unwrap(),
                );
            }
            let states = &merged.rollouts["urn"].operations;
            assert_eq!(states["add"], OperationOutcome::Passed);
            assert!(matches!(states["change"], OperationOutcome::Failed { .. }));
            assert_eq!(states["remove"], OperationOutcome::Unexercised);
            assert!(!merged.checks["live-rollout"]);
        }
    }
}
