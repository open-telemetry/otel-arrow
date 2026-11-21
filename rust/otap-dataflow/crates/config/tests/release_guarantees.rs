// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Observable resolution of a released operator configuration.

use otel_arrow_dfe_config::engine::OtelDataflowSpec;
use otel_arrow_dfe_config::policy::{CoreAllocation, CoreAllocationStrategy};
use serde_json::{Value, json};

const RELEASE: &str = include_str!("fixtures/releases/v0.60.0/syslog-console.json");

/// Scenario: resolve the released core-count alias, omit it, then override it at narrower scopes.
/// Guarantees: omission selects all cores and a pipeline override takes precedence over its group.
#[test]
fn released_core_allocation_precedence() {
    let yaml = include_str!("fixtures/releases/v0.60.0/syslog-console.yaml");
    assert_eq!(
        OtelDataflowSpec::from_yaml(yaml).unwrap(),
        OtelDataflowSpec::from_json(RELEASE).unwrap()
    );
    let mut fixture: Value = serde_json::from_str(RELEASE).unwrap();
    let allocation = |fixture: &Value| {
        let config = OtelDataflowSpec::from_json(&fixture.to_string()).unwrap();
        let regular = config.resolve().into_parts().1;
        assert_eq!(regular.len(), 1);
        regular[0].policies.resources.core_allocation.clone()
    };
    assert_eq!(allocation(&fixture), CoreAllocation::core_count(1));
    assert!(
        fixture
            .as_object_mut()
            .unwrap()
            .remove("policies")
            .is_some()
    );
    assert_eq!(
        allocation(&fixture).strategy,
        CoreAllocationStrategy::AllCores
    );
    fixture["groups"]["default"]["policies"] =
        json!({"resources": {"core_allocation": {"type": "core_count", "count": 3}}});
    assert_eq!(allocation(&fixture), CoreAllocation::core_count(3));
    fixture["groups"]["default"]["pipelines"]["main"]["policies"] =
        json!({"resources": {"core_allocation": {"type": "core_count", "count": 2}}});
    assert_eq!(allocation(&fixture), CoreAllocation::core_count(2));
}
