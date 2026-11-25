// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Development-only protocol shared by the conformance driver and cargo xtask.
//! [`Catalog`] describes the promises to check. The driver records what happened
//! in [`Observations`] or a focused [`ComponentReport`], which xtask reads for reporting.
pub mod releases;
mod rollout;
pub mod ui;

pub use rollout::{Operation, OperationOutcome, Rollout};

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
    /// Inherited gap in deployed coverage.
    pub gap: Option<DeploymentGap>,
    /// Probe dispatched by the conformance driver.
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

/// Accountability for an operator promise awaiting a deployed scenario.
#[derive(Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct DeploymentGap {
    /// Maintainers responsible for the follow-up.
    pub owner: String,
    /// Deployment experiment needed to close the gap.
    pub follow_up: String,
}

/// Evidence for one input consumed by the embedded UI.
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct UiSignal {
    /// Dashboard view or reader.
    pub consumer: String,
    /// Qualified metric, event or stream.
    pub signal: String,
    /// What the probe saw in this deployment.
    pub result: String,
}

/// Native probes implemented by the conformance driver.
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
    /// Exercise pipeline node fixtures.
    LiveRollout,
    /// Exercise pipeline extension fixtures.
    ExtensionRollout,
    /// Verify committed service after rejection.
    RejectedUpdate,
    /// Reapply the active component configuration.
    ReconfigurationNoop,
}

/// Historical regression fixtures required in every full deployment run.
pub const REQUIRED_ROLLOUTS: [(&str, Scenario); 2] = [
    ("urn:otel:receiver:syslog_cef", Scenario::LiveRollout),
    (
        "urn:otel:extension:flat_file_api_key_auth",
        Scenario::ExtensionRollout,
    ),
];

impl Scenario {
    /// Preflight checks run before assessing deployed behavior.
    #[must_use]
    pub fn preflight(self) -> bool {
        matches!(self, Self::FactoryValidation | Self::ReleasedInputs)
    }
}

/// Load the compiled-in catalog and reject ambiguous execution sources.
pub fn catalog() -> Result<Catalog> {
    let entries: Catalog =
        serde_yaml::from_str(include_str!("../../../../docs/conformance/checks.yaml"))?;
    let mut scenarios = BTreeSet::new();
    for (id, check) in &entries {
        match (&check.gap, check.scenario) {
            (Some(gap), None) => ensure!(
                !gap.owner.trim().is_empty() && !gap.follow_up.trim().is_empty(),
                "{id}: missing gap owner or follow-up"
            ),
            (None, Some(scenario)) => ensure!(
                scenarios.insert(scenario),
                "{id}: duplicate deployment scenario"
            ),
            _ => anyhow::bail!("{id}: specify a scenario or an inherited deployment gap"),
        }
    }
    Ok(entries)
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
    /// Capabilities used to classify platform-specific UI inputs.
    pub ui_platform: ui::UiPlatform,
    /// Outcomes for the historical pipelines actually checked on this candidate.
    #[serde(default)]
    pub released_inputs: releases::Results,
    /// Per-signal observations from the dashboard deployment.
    pub ui_signals: Vec<UiSignal>,
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
    /// Historical pipelines containing the selected component, with their validation outcomes.
    #[serde(default)]
    pub released_inputs: releases::Results,
    /// Applicable catalog checks exercised for this component.
    pub behavior_checks: Checks,
    /// Diagnostics keyed by failed check ID.
    pub check_errors: BTreeMap<String, String>,
    /// Inherited gaps applicable to this component.
    pub deployment_gaps: BTreeMap<String, DeploymentGap>,
    /// Applicable checks for which no observation was available.
    pub unexercised_checks: Vec<String>,
}

impl ComponentReport {
    /// The findings shown to the component author and used for the driver's exit status.
    #[must_use]
    pub fn blocking_findings(&self) -> Vec<String> {
        let mut findings: Vec<_> = self.finding.iter().cloned().collect();
        for (name, passed) in &self.behavior_checks {
            if !passed {
                let reason = self.check_errors.get(name).map_or("failed", String::as_str);
                findings.push(format!("{name}: {reason}"));
            }
        }
        if let Some(error) = self.check_errors.get("released-inputs") {
            findings.push(format!("released-inputs: {error}"));
        }
        findings
    }
}
