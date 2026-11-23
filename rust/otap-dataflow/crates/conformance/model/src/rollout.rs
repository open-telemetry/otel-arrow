// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Ordered lifecycle stages and their serialized deployment observations.
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A stage of the component fixture lifecycle, in execution order.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Operation {
    /// Install the subject in the running pipeline.
    Add,
    /// Replace its configuration.
    Change,
    /// Restore the baseline pipeline.
    Remove,
}

impl Operation {
    /// A failure stops execution of the remaining stages.
    pub const ALL: [Self; 3] = [Self::Add, Self::Change, Self::Remove];

    /// Stable key used in reports, saved observations and review evidence.
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Add => "add",
            Self::Change => "change",
            Self::Remove => "remove",
        }
    }

    /// The operation keys in the order used by the runner and reports.
    #[must_use]
    pub fn names() -> [&'static str; 3] {
        Self::ALL.map(Self::name)
    }
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

impl Rollout {
    /// Start with no evidence, including when fixture setup fails before add.
    #[must_use]
    pub fn pending() -> Self {
        Self {
            operations: Operation::names()
                .into_iter()
                .map(|name| (name.to_owned(), OperationOutcome::Unexercised))
                .collect(),
            error: None,
        }
    }
}
