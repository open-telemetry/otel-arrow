// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Select fixtures and contracts once, then collect the same observations in either scope.
use super::{dashboard, drain, fixtures::Fixture, live, reconfiguration, released};
use anyhow::{Context, Result, bail, ensure};
use futures_util::{StreamExt, stream};
use otel_arrow_dfe_conformance_model::{
    Catalog, ComponentReport, Contract, Observations, REQUIRED_ROLLOUTS, Rollout, Scenario,
};
use otel_arrow_dfe_validation::{
    deployment::Engine,
    factory_validators::{self, Observation},
};
use std::collections::{BTreeMap, BTreeSet};
use std::time::Duration;
use tokio::time::timeout;

type Registrations = BTreeMap<&'static str, Observation>;

/// Contracts selected from the review catalog and component fixtures eligible to run.
/// The linked factories supply a separate configuration preflight for this build.
pub(super) struct ExecutionPlan<'a> {
    component: Option<&'a str>,
    contracts: Vec<(&'a str, &'a Contract)>,
    fixtures: Vec<&'a Fixture>,
    linked: Registrations,
    prerequisites: &'a BTreeSet<String>,
    unavailable: Option<String>,
}

impl<'a> ExecutionPlan<'a> {
    /// Select only the dashboard deployment for a standalone UI check.
    pub fn embedded_ui(catalog: &'a Catalog, prerequisites: &'a BTreeSet<String>) -> Result<Self> {
        let (id, check) = catalog
            .get_key_value("embedded-ui")
            .context("embedded-ui missing from catalog")?;
        ensure!(
            check.scenario == Some(Scenario::EmbeddedUi),
            "embedded-ui requires its deployment scenario"
        );
        Ok(Self {
            component: None,
            contracts: vec![(id, check)],
            fixtures: Vec::new(),
            linked: BTreeMap::new(),
            prerequisites,
            unavailable: None,
        })
    }

    /// Match catalog contracts by URN or role, then schedule fixtures whose prerequisites are enabled.
    /// A focused plan retains the reason its component cannot run. In a full run, xtask
    /// accounts for missing coverage against the source inventory.
    pub fn new(
        component: Option<&'a str>,
        contracts: &'a Catalog,
        fixtures: &'a BTreeMap<String, Fixture>,
        prerequisites: &'a BTreeSet<String>,
    ) -> Result<Self> {
        let linked = factory_validators::observations(|urn| {
            component.is_none_or(|selected| selected == urn)
        });
        let unlinked = component.is_some_and(|urn| !linked.contains_key(urn));
        let scope = component.map(|urn| {
            let role = linked.get(urn).map_or_else(
                || urn.split(':').nth(2).unwrap_or_default(),
                |registration| registration.role,
            );
            (urn, role)
        });
        let contracts: Vec<_> = contracts
            .iter()
            .filter(|(_, contract)| {
                contract.selected(scope) && (!unlinked || contract.gap.is_some())
            })
            .map(|(id, contract)| (id.as_str(), contract))
            .collect();
        let mut scheduled = Vec::new();
        let mut unavailable = unlinked.then(|| "not linked in this build".to_owned());
        for (&urn, registration) in &linked {
            if registration.role == "controller-extension" {
                continue;
            }
            if component.is_some() && !registration.rejected {
                // A focused run stops this fixture at its failed preflight. In a full run
                // keep the independent deployment observation alongside the failed preflight.
                unavailable = Some("factory accepted invalid configuration".to_owned());
                continue;
            }
            let eligible = fixtures
                .get(urn)
                .context("No live reconfiguration fixture")
                .and_then(|fixture| {
                    for prerequisite in &fixture.requires {
                        ensure!(
                            prerequisites.contains(prerequisite),
                            "unexercised: requires {prerequisite}"
                        );
                    }
                    Ok(fixture)
                });
            match eligible {
                Ok(fixture) => {
                    let covered = contracts.iter().any(|(_, c)| {
                        matches!(
                            c.scenario,
                            Some(Scenario::LiveRollout | Scenario::ExtensionRollout)
                        ) && c.roles.iter().any(|r| r == registration.role)
                    });
                    ensure!(covered, "no catalog rollout check for {urn}");
                    scheduled.push(fixture);
                }
                Err(error) if component.is_some() => unavailable = Some(error.to_string()),
                Err(_) => (), // Full-suite coverage accounts for deferred and missing fixtures in xtask.
            }
        }
        Ok(Self {
            component,
            contracts,
            fixtures: scheduled,
            linked,
            prerequisites,
            unavailable,
        })
    }

    /// Exercise each selected fixture once. Reuse its stage results for the applicable
    /// rollout contracts and run the remaining scenarios against the candidate.
    pub async fn run(&self, engine: &Engine, revision: &str) -> Observations {
        let mut observed = Observations {
            commit: revision.to_owned(),
            linked_validators: self
                .linked
                .iter()
                .map(|(&urn, v)| (urn.to_owned(), v.rejected))
                .collect(),
            prerequisites: self.prerequisites.clone(),
            ..Default::default()
        };
        let mut selected_noop = None;
        let mut runs = stream::iter(self.fixtures.iter().copied())
            .map(|fixture| async move {
                let registration = &self.linked[fixture.urn.as_str()];
                let (rollout, noop) = live::exercise(engine, fixture, registration.role).await;
                (&fixture.urn, rollout, noop)
            })
            .buffer_unordered(3);
        while let Some((urn, rollout, noop)) = runs.next().await {
            if let Some(reason) = &rollout.error {
                eprintln!("FAIL {urn}: {reason}");
            }
            if self.component == Some(urn.as_str()) {
                selected_noop = noop;
            }
            let _ = observed.rollouts.insert(urn.clone(), rollout);
        }

        for &(id, contract) in &self.contracts {
            let Some(scenario) = contract.scenario else {
                continue;
            };
            let result = match scenario {
                Scenario::FactoryValidation => self.check_factories(),
                Scenario::LiveRollout | Scenario::ExtensionRollout => {
                    self.check_rollouts(scenario, contract, &observed)
                }
                Scenario::ReconfigurationNoop => {
                    if let Some(passed) = selected_noop {
                        let _ = observed.checks.insert(id.to_owned(), passed);
                    }
                    continue; // A failed add leaves no observation for the change-stage no-op.
                }
                Scenario::ReleasedInputs => {
                    match released::run(engine, self.component, &mut observed.released_inputs).await
                    {
                        Ok(false) => continue,
                        result => result.map(|_| ()),
                    }
                }
                Scenario::FiniteIdDrain => drain::finite_id_drain(engine).await,
                Scenario::EmbeddedUi => timeout(
                    Duration::from_secs(35),
                    dashboard::run(engine, &mut observed.ui_signals),
                )
                .await
                .context("embedded UI scenario timed out")
                .and_then(|r| r),
                Scenario::ReconfigurationConsistency => timeout(
                    Duration::from_secs(45),
                    reconfiguration::reconfiguration_consistency(engine),
                )
                .await
                .context("reconfiguration consistency timed out")
                .and_then(|r| r),
                Scenario::RejectedUpdate => timeout(
                    Duration::from_secs(60),
                    reconfiguration::rejected_update(engine),
                )
                .await
                .context("rejected-update scenario timed out")
                .and_then(|r| r),
            };
            let _ = observed.checks.insert(id.to_owned(), result.is_ok());
            match result {
                Ok(()) => eprintln!("PASS {id}"),
                Err(error) => {
                    eprintln!("FAIL {id}: {error:#}");
                    let _ = observed
                        .check_errors
                        .insert(id.to_owned(), format!("{error:#}"));
                }
            }
        }
        observed
    }

    /// Require each selected factory to reject invalid input. A full run must also cover every role.
    fn check_factories(&self) -> Result<()> {
        let invalid: Vec<_> = self
            .linked
            .iter()
            .filter(|(_, v)| !v.rejected)
            .map(|(&urn, _)| urn)
            .collect();
        ensure!(
            invalid.is_empty(),
            "validators accepted invalid input: {invalid:?}"
        );
        if self.component.is_none() {
            for role in [
                "receiver",
                "processor",
                "exporter",
                "extension",
                "controller-extension",
            ] {
                ensure!(
                    self.linked.values().any(|v| v.role == role),
                    "empty {role} registry"
                );
            }
        }
        Ok(())
    }

    /// Check failures for the contract's roles. A full run also needs observations for
    /// the historical regression fixtures even if their factories disappeared.
    fn check_rollouts(
        &self,
        scenario: Scenario,
        contract: &Contract,
        observed: &Observations,
    ) -> Result<()> {
        if self.component.is_none() {
            for (urn, group) in REQUIRED_ROLLOUTS {
                if group == scenario {
                    ensure!(
                        observed.rollouts.contains_key(urn),
                        "missing required fixture or linked factory: {urn}"
                    );
                }
            }
        } else if let Some(reason) = &self.unavailable {
            bail!("{reason}");
        }
        for (urn, rollout) in &observed.rollouts {
            if rollout.error.is_some()
                && contract
                    .roles
                    .iter()
                    .any(|r| r == self.linked[urn.as_str()].role)
            {
                bail!("see component operation results");
            }
        }
        Ok(())
    }

    /// Project the shared measurements into the focused artifact consumed by xtask.
    pub fn component_report(&self, urn: &str, mut observed: Observations) -> ComponentReport {
        let registration = self.linked.get(urn);
        let startup_only = registration.is_some_and(|v| v.role == "controller-extension");
        let factory_validation = observed.linked_validators.get(urn).copied();
        let mut rollout = observed
            .rollouts
            .remove(urn)
            .unwrap_or_else(Rollout::pending);
        let finding = if factory_validation == Some(false) {
            Some("factory accepted invalid configuration".to_owned())
        } else {
            self.unavailable.clone().or_else(|| rollout.error.take())
        };
        let mut report = ComponentReport {
            component: urn.to_owned(),
            commit: observed.commit,
            factory_validation,
            released_inputs: observed.released_inputs,
            operations: (!startup_only).then_some(rollout.operations),
            finding,
            behavior_checks: Default::default(),
            check_errors: Default::default(),
            deployment_gaps: Default::default(),
            unexercised_checks: Vec::new(),
        };
        for &(id, contract) in &self.contracts {
            if let Some(gap) = &contract.gap {
                let _ = report.deployment_gaps.insert(id.to_owned(), gap.clone());
                continue;
            }
            if matches!(
                contract.scenario,
                Some(
                    Scenario::FactoryValidation
                        | Scenario::LiveRollout
                        | Scenario::ExtensionRollout
                )
            ) {
                continue; // Already represented by the factory and per-operation fields.
            }
            if let Some(error) = observed.check_errors.remove(id) {
                let _ = report.check_errors.insert(id.to_owned(), error);
            }
            if contract.scenario == Some(Scenario::ReleasedInputs) {
                continue; // Per-file acceptance is a preflight, including on focused runs.
            }
            if let Some(passed) = observed.checks.remove(id) {
                let _ = report.behavior_checks.insert(id.to_owned(), passed);
            } else {
                report.unexercised_checks.push(id.to_owned());
            }
        }
        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_validation::{
        fanout_processor::FANOUT_PROCESSOR_URN, validation_exporter::VALIDATION_EXPORTER_URN,
    };

    /// Scenario: an author narrows a full suite to the durable buffer processor.
    /// Guarantees: both plans select the same fixture and drain probe; the focused plan
    /// excludes unrelated deployments and the full inventory excludes validation-only nodes.
    #[test]
    fn focused_plan_selects_the_same_fixture_and_drain_probe() {
        let contracts = otel_arrow_dfe_conformance_model::catalog().unwrap();
        let fixtures = super::super::fixtures::catalog().unwrap();
        let enabled = BTreeSet::new();
        let urn = "urn:otel:processor:durable_buffer";
        let full = ExecutionPlan::new(None, &contracts, &fixtures, &enabled).unwrap();
        let focused = ExecutionPlan::new(Some(urn), &contracts, &fixtures, &enabled).unwrap();
        assert!(!full.linked.contains_key(FANOUT_PROCESSOR_URN));
        assert!(!full.linked.contains_key(VALIDATION_EXPORTER_URN));
        assert_eq!(focused.linked.keys().copied().collect::<Vec<_>>(), [urn]);
        assert_eq!(focused.fixtures.len(), 1);
        assert_eq!(focused.fixtures[0].urn, urn);
        assert!(full.fixtures.iter().any(|f| f.urn == urn));
        let selects = |plan: &ExecutionPlan<'_>, id: &str| {
            plan.contracts.iter().any(|&(check, _)| check == id)
        };
        assert!(selects(&full, "finite-id-drain") && selects(&focused, "finite-id-drain"));
        assert!(selects(&full, "embedded-ui") && !selects(&focused, "embedded-ui"));
        assert!(selects(&focused, "reconfiguration-noop"));
    }
}
