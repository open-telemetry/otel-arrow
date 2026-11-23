// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Tests that every registered component's `validate_config` function correctly
//! rejects invalid configuration input.
//!
//! These tests iterate over all factories registered in `OTAP_PIPELINE_FACTORY`
//! and call `validate_config` with clearly invalid JSON values to ensure:
//! 1. Every component's validator is wired up and callable.
//! 2. Invalid configs produce `Err`, not silent acceptance.
//!
//! Valid-config paths are already covered by the CI `validate-configs.sh` script
//! which runs `--validate-and-exit` against every example YAML in the repo.

use otel_arrow_dfe_contrib_extensions as _;
use otel_arrow_dfe_contrib_nodes as _;
use otel_arrow_dfe_validation::factory_validators;

fn assert_registered_validators(kind: &str) {
    let results = factory_validators::observations(|_| true);
    let mut selected = results
        .iter()
        .filter(|(_, result)| result.role == kind)
        .peekable();
    assert!(selected.peek().is_some(), "empty {kind} registry");
    for (urn, result) in selected {
        assert!(result.rejected, "{urn} accepted an invalid scalar");
    }
}

/// Scenario: every registered receiver validator receives an invalid scalar config.
/// Guarantees: no receiver silently accepts the invalid configuration.
#[test]
fn all_receiver_validators_reject_invalid_config() {
    assert_registered_validators("receiver");
}

/// Scenario: every registered processor validator receives an invalid scalar config.
/// Guarantees: no processor silently accepts the invalid configuration.
#[test]
fn all_processor_validators_reject_invalid_config() {
    assert_registered_validators("processor");
}

/// Scenario: every registered exporter validator receives an invalid scalar config.
/// Guarantees: no exporter silently accepts the invalid configuration.
#[test]
fn all_exporter_validators_reject_invalid_config() {
    assert_registered_validators("exporter");
}

/// Scenario: linked pipeline and controller extension factories receive an invalid scalar.
/// Guarantees: both extension registries are present and reject invalid configuration.
#[test]
fn all_extension_validators_reject_invalid_config() {
    let results = factory_validators::observations(|_| true);
    assert!(results.contains_key("urn:otel:extension:flat_file_api_key_auth"));
    assert!(results.contains_key("urn:otel:extension:controller_monitor"));
    assert_registered_validators("extension");
    assert_registered_validators("controller-extension");
}
