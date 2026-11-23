// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Match the status conditions consumed by the embedded pipeline selector.
use serde::Deserialize;
use serde_json::Value;
use std::collections::BTreeMap;

/// Pending and unhealthy observations required from the deployed engine.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PipelineStates {
    /// A new pipeline with no observed runtime.
    pub pending: PipelineState,
    /// A pipeline whose receiver failed to bind.
    pub unhealthy: PipelineState,
}

/// Status fields displayed together in a single dashboard snapshot.
#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct PipelineState {
    running_cores: u64,
    total_cores: u64,
    accepted: Condition,
    ready: Condition,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Condition {
    status: String,
    reason: String,
}

impl PipelineState {
    /// Check the core counts and both renderable conditions on the same pipeline.
    #[must_use]
    pub fn matches(&self, pipeline: &Value, kinds: &BTreeMap<String, String>) -> bool {
        pipeline["runningCores"] == self.running_cores
            && pipeline["totalCores"] == self.total_cores
            && pipeline["conditions"].as_array().is_some_and(|conditions| {
                [("accepted", &self.accepted), ("ready", &self.ready)]
                    .iter()
                    .all(|(role, expected)| {
                        let Some(kind) = kinds.get(*role) else {
                            return false;
                        };
                        conditions
                            .iter()
                            .find(|c| c["type"] == *kind)
                            .is_some_and(|c| {
                                c["status"] == expected.status
                                    && c["reason"] == expected.reason
                                    && c["message"].as_str().is_some_and(|s| !s.trim().is_empty())
                            })
                    })
            })
    }
}

/// Status row for a pipeline whose runtime has yet to appear.
pub const PENDING: &str = "pending before runtime admission";
/// Status row for the receiver whose startup fails on the occupied port.
pub const UNHEALTHY: &str = "unhealthy after receiver bind failure";

#[cfg(test)]
mod tests {
    use super::super::{Inputs, SIGNAL_CATALOG};
    use serde_json::{Value, json};

    /// Scenario: accepted, ready or core fields cease to describe the observed lifecycle state.
    /// Guarantees: a missing pipeline, missing reason, or an admitted runtime cannot stand in for pending.
    #[test]
    fn status_requires_matching_counts_and_renderable_conditions() {
        let events = Inputs::parse(SIGNAL_CATALOG).unwrap().events;
        let pending = &events.pipeline_states.pending;
        let mut p = json!({"runningCores":0,"totalCores":0,"conditions":[
            {"type":"Accepted","status":"Unknown","reason":"NoPipelineRuntime","message":"No runtime"},
            {"type":"Ready","status":"Unknown","reason":"NoPipelineRuntime","message":"No runtime"}
        ]});
        assert!(pending.matches(&p, &events.conditions));
        assert!(
            !events
                .pipeline_states
                .unhealthy
                .matches(&p, &events.conditions)
        );
        assert!(!pending.matches(&Value::Null, &events.conditions));
        p["totalCores"] = json!(1);
        assert!(!pending.matches(&p, &events.conditions));
        p["totalCores"] = json!(0);
        p["conditions"][0]["reason"] = Value::Null;
        assert!(!pending.matches(&p, &events.conditions));
    }
}
