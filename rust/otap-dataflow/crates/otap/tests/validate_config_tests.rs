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

use otap_df_otap::OTAP_PIPELINE_FACTORY;
use serde_json::json;

#[test]
fn all_receiver_validators_reject_invalid_config() {
    let factory_map = OTAP_PIPELINE_FACTORY.get_receiver_factory_map();
    assert!(
        !factory_map.is_empty(),
        "No receiver factories registered — test is misconfigured"
    );

    for (urn, factory) in factory_map {
        let result = (factory.validate_config)(&json!("this is not a valid config"));
        assert!(
            result.is_err(),
            "Receiver `{urn}`: validate_config should reject a plain string"
        );
    }
}

#[test]
fn all_processor_validators_reject_invalid_config() {
    let factory_map = OTAP_PIPELINE_FACTORY.get_processor_factory_map();
    assert!(
        !factory_map.is_empty(),
        "No processor factories registered — test is misconfigured"
    );

    for (urn, factory) in factory_map {
        let result = (factory.validate_config)(&json!("this is not a valid config"));
        assert!(
            result.is_err(),
            "Processor `{urn}`: validate_config should reject a plain string"
        );
    }
}

#[test]
fn all_exporter_validators_reject_invalid_config() {
    let factory_map = OTAP_PIPELINE_FACTORY.get_exporter_factory_map();
    assert!(
        !factory_map.is_empty(),
        "No exporter factories registered — test is misconfigured"
    );

    for (urn, factory) in factory_map {
        let result = (factory.validate_config)(&json!("this is not a valid config"));
        assert!(
            result.is_err(),
            "Exporter `{urn}`: validate_config should reject a plain string"
        );
    }
}
