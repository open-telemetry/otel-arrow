// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Development-only protocol shared by the validation driver and cargo xtask.
use anyhow::{Result, ensure};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

/// Candidate check IDs and their measured outcomes.
pub type Checks = BTreeMap<String, bool>;
/// Contracts keyed by the stable ID used in reviews and reports.
pub type Catalog = BTreeMap<String, Contract>;

/// One operator promise and the source of its evidence.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    /// Review area this observation can satisfy.
    pub area: String,
    /// Operator-visible promise.
    pub contract: String,
    /// How the implementation establishes the promise.
    pub method: String,
    /// Workspace-relative implementation path.
    pub source: String,
    /// Entry points for a native probe.
    #[serde(default)]
    pub functions: Vec<String>,
    /// Full component URNs eligible for this focused check.
    #[serde(default)]
    pub components: Vec<String>,
    /// Registry roles covered by a component-wide probe.
    #[serde(default)]
    pub roles: Vec<String>,
    /// Exclude this check from the full suite.
    #[serde(default)]
    pub focused_only: bool,
    /// Owning-crate evidence, mutually exclusive with a scenario.
    pub cargo: Option<CargoCase>,
    /// Probe dispatched by the validation driver.
    pub scenario: Option<Scenario>,
}

impl Contract {
    /// Select a required suite case or an applicable component case.
    #[must_use]
    pub fn selected(&self, component: Option<(&str, &str)>) -> bool {
        component.map_or(!self.focused_only, |(urn, role)| {
            self.components.iter().any(|c| c == urn) || self.roles.iter().any(|r| r == role)
        })
    }
}

/// Exact tests that must all pass for one contract.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct CargoCase {
    /// Owning Cargo package.
    pub package: String,
    /// Integration test target, or the library when omitted.
    pub target: Option<String>,
    /// Features needed to build this test target.
    #[serde(default)]
    pub features: String,
    /// Fully qualified test names.
    pub tests: Vec<String>,
}

/// Native probes implemented by the validation driver.
#[derive(Deserialize, Serialize, Copy, Clone, Debug, PartialEq, Eq, Ord, PartialOrd)]
#[serde(rename_all = "kebab-case")]
pub enum Scenario {
    /// Reject invalid input at every linked factory.
    FactoryValidation,
    /// Validate release-pinned inputs.
    ReleasedInputs,
    /// Exercise dashboard data sources.
    EmbeddedUi,
    /// Check no-op and failed replacement behavior.
    ReconfigurationConsistency,
    /// Observe admitted IDs across graceful shutdown.
    FiniteIdDrain,
    /// Exercise pipeline node fixtures.
    LiveRollout,
    /// Exercise pipeline extension fixtures.
    ExtensionRollout,
    /// Verify committed service after rejection.
    RejectedUpdate,
    /// Reapply the active component configuration.
    ReconfigurationNoop,
}

/// Load the compiled-in catalog and reject ambiguous execution sources.
pub fn catalog() -> Result<Catalog> {
    let entries: Catalog =
        serde_yaml::from_str(include_str!("../../../docs/guarantees/checks.yaml"))?;
    let mut scenarios = BTreeSet::new();
    for (id, check) in &entries {
        if let Some(cargo) = &check.cargo {
            ensure!(
                check.scenario.is_none() && !cargo.tests.is_empty(),
                "{id}: expected exact Cargo tests"
            );
        } else {
            ensure!(
                check.scenario.is_some_and(|s| scenarios.insert(s)),
                "{id}: missing or duplicate scenario"
            );
        }
    }
    Ok(entries)
}

/// Evidence for one stage of a live fixture.
#[derive(Deserialize, Serialize, PartialEq, Debug)]
#[serde(tag = "status", rename_all = "lowercase", deny_unknown_fields)]
pub enum OperationOutcome {
    /// The stage completed with its expected observation.
    Passed,
    /// The stage ran and its assertion failed.
    Failed {
        /// Diagnostic from the failed probe.
        reason: String,
    },
    /// No observation was made for this stage.
    Unexercised,
}

impl OperationOutcome {
    /// Human-readable operation status.
    #[must_use]
    pub fn label(&self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed { .. } => "failed",
            Self::Unexercised => "unexercised",
        }
    }
}

/// Ordered lifecycle evidence for one component.
#[derive(Deserialize, Serialize)]
pub struct Rollout {
    /// Results keyed by lifecycle operation.
    pub operations: BTreeMap<String, OperationOutcome>,
    /// First failure that stopped the fixture.
    pub error: Option<String>,
}

/// Full-suite observations for one engine build and candidate revision.
#[derive(Deserialize, Serialize, Default)]
pub struct Observations {
    /// Candidate revision supplied to the driver.
    pub commit: String,
    /// Outcomes keyed by catalog ID.
    pub checks: Checks,
    /// Diagnostics keyed by failed check ID.
    #[serde(default)]
    pub check_errors: BTreeMap<String, String>,
    /// Factory validation outcomes by linked URN.
    pub linked_validators: BTreeMap<String, bool>,
    /// Executed fixtures keyed by URN.
    pub rollouts: BTreeMap<String, Rollout>,
    /// Environments enabled for this run.
    pub prerequisites: BTreeSet<String>,
}

/// A focused observation that cannot substitute for full-suite evidence.
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentReport {
    /// The selected full component URN.
    pub component: String,
    /// Candidate revision supplied to the driver.
    pub commit: String,
    /// Factory result, absent for unlinked components.
    pub factory_validation: Option<bool>,
    /// Results keyed by lifecycle operation.
    pub operations: Option<BTreeMap<String, OperationOutcome>>,
    /// Reason this component could not complete its fixture.
    pub finding: Option<String>,
    /// Applicable catalog checks exercised for this component.
    pub behavior_checks: Checks,
    /// Diagnostics keyed by failed check ID.
    pub check_errors: BTreeMap<String, String>,
    /// Applicable checks for which no observation was available.
    pub unexercised_checks: Vec<String>,
}
