// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Multi-stage validation building blocks.
//!
//! A [`Stage`] captures one complete `{ SUV pipeline + generators + captures +
//! assertions }` set. A [`Scenario`](crate::scenario::Scenario) runs one or
//! more stages in order, transitioning between them with the engine's
//! live-update (reconfigure) API rather than restarting the engine.
//!
//! Each stage is independent: on every transition the framework reconfigures
//! all three pipeline families (the `suv` pipeline plus one pipeline per
//! generator label and one per capture label), so every stage runs with fresh
//! traffic and a fresh validation exporter.

use crate::pipeline::Pipeline;
use crate::traffic::{Capture, Generator};
use std::collections::HashMap;

/// Classification the engine applies to a live-update (reconfigure) request for
/// one logical pipeline.
///
/// This mirrors the `action` field returned on a rollout status
/// (`create`, `noop`, `replace`, `resize`). A stage can assert the expected
/// classification of its `suv` transition via [`Stage::expect_rollout`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RolloutAction {
    /// The logical pipeline did not exist before and was created.
    Create,
    /// The candidate pipeline is identical to the live one; nothing changed.
    NoOp,
    /// The runtime graph or a node's config changed; the whole pipeline is
    /// relaunched per core and the old instance is drained.
    Replace,
    /// Only `core_allocation` changed; instances are added/removed per core.
    Resize,
}

impl RolloutAction {
    /// Returns the wire string used by the admin API `action` field.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            RolloutAction::Create => "create",
            RolloutAction::NoOp => "noop",
            RolloutAction::Replace => "replace",
            RolloutAction::Resize => "resize",
        }
    }

    /// Parses the wire string used by the admin API `action` field.
    #[must_use]
    pub fn from_wire(action: &str) -> Option<Self> {
        match action {
            "create" => Some(RolloutAction::Create),
            "noop" => Some(RolloutAction::NoOp),
            "replace" => Some(RolloutAction::Replace),
            "resize" => Some(RolloutAction::Resize),
            _ => None,
        }
    }
}

/// One stage of a multi-stage validation scenario.
///
/// A stage bundles the system-under-validation pipeline, its traffic
/// generators, and its captures (assertions). Stages are transitioned via live
/// update; see [`crate::scenario::Scenario::add_stage`].
pub struct Stage {
    pub(crate) pipeline: Option<Pipeline>,
    pub(crate) generators: HashMap<String, Generator>,
    pub(crate) captures: HashMap<String, Capture>,
    pub(crate) expected_action: Option<RolloutAction>,
}

impl Default for Stage {
    fn default() -> Self {
        Self::new()
    }
}

impl Stage {
    /// Start a new, empty stage.
    #[must_use]
    pub fn new() -> Self {
        Self {
            pipeline: None,
            generators: HashMap::new(),
            captures: HashMap::new(),
            expected_action: None,
        }
    }

    /// Provide the system-under-validation pipeline for this stage.
    #[must_use]
    pub fn pipeline(mut self, pipeline: Pipeline) -> Self {
        self.pipeline = Some(pipeline);
        self
    }

    /// Add a traffic generator labeled for wiring.
    ///
    /// The label is also the pipeline id used when reconfiguring this
    /// generator during a stage transition.
    #[must_use]
    pub fn add_generator(mut self, label: impl Into<String>, generator: Generator) -> Self {
        let _ = self.generators.insert(label.into(), generator);
        self
    }

    /// Add a capture labeled for wiring.
    ///
    /// The label is also the pipeline id used when reconfiguring this capture
    /// during a stage transition.
    #[must_use]
    pub fn add_capture(mut self, label: impl Into<String>, capture: Capture) -> Self {
        let _ = self.captures.insert(label.into(), capture);
        self
    }

    /// Assert the classification the engine must apply when transitioning the
    /// `suv` pipeline into this stage.
    ///
    /// Only meaningful for stages after the first (the first stage starts the
    /// engine rather than reconfiguring it). If the observed rollout action
    /// differs, the scenario fails with
    /// [`ValidationError::Reconfigure`](crate::error::ValidationError::Reconfigure).
    #[must_use]
    pub fn expect_rollout(mut self, action: RolloutAction) -> Self {
        self.expected_action = Some(action);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::traffic::{Capture, Generator};

    /// Scenario: RolloutAction converts to and from its admin-API wire strings.
    /// Guarantees: the wire mapping stays aligned with the controller's
    /// `create`/`noop`/`replace`/`resize` action names in both directions.
    #[test]
    fn rollout_action_wire_roundtrip() {
        for action in [
            RolloutAction::Create,
            RolloutAction::NoOp,
            RolloutAction::Replace,
            RolloutAction::Resize,
        ] {
            assert_eq!(RolloutAction::from_wire(action.as_str()), Some(action));
        }
        assert_eq!(RolloutAction::from_wire("bogus"), None);
    }

    /// Scenario: a stage is assembled through its fluent builder.
    /// Guarantees: pipeline, generators, captures, and the expected rollout
    /// action are all recorded on the stage.
    #[test]
    fn stage_builder_records_fields() {
        let stage = Stage::new()
            .add_generator("gen", Generator::logs().otlp_grpc("receiver"))
            .add_capture("cap", Capture::default().otlp_grpc("exporter"))
            .expect_rollout(RolloutAction::Replace);

        assert!(stage.pipeline.is_none());
        assert!(stage.generators.contains_key("gen"));
        assert!(stage.captures.contains_key("cap"));
        assert_eq!(stage.expected_action, Some(RolloutAction::Replace));
    }
}
