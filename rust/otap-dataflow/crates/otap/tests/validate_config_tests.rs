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

use otap_df_config::pipeline::PipelineConfig;
use otap_df_config::{PipelineGroupId, PipelineId};
use otap_df_otap::OTAP_PIPELINE_FACTORY;
use serde_json::json;

// Keep this side-effect import so the crate is linked and its `linkme`
// distributed-slice registrations (contrib nodes) are visible
// in `OTAP_PIPELINE_FACTORY` at runtime.
use otap_df_contrib_nodes as _;

// Keep this side-effect import so the crate is linked and its `linkme`
// distributed-slice registrations (core nodes) are visible
// in `OTAP_PIPELINE_FACTORY` at runtime.
use otap_df_core_nodes as _;

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

// -- processor_chain validation via startup::validate_pipeline_components ------

fn chain_pipeline(chain_config_yaml: &str) -> PipelineConfig {
    let yaml = format!(
        r#"
nodes:
  receiver:
    type: receiver:traffic_generator
    config:
      traffic_config:
        max_batch_size: 1
        signals_per_second: 1
        log_weight: 100
      registry_path: https://github.com/open-telemetry/semantic-conventions.git[model]
  chain:
    type: processor_chain:composite
    config:
{chain_config_yaml}
  exporter:
    type: exporter:noop
    config:
connections:
  - from: receiver
    to: chain
  - from: chain
    to: exporter
"#
    );
    let gid: PipelineGroupId = "test_group".into();
    let pid: PipelineId = "test_pipeline".into();
    PipelineConfig::from_yaml(gid, pid, &yaml).expect("pipeline YAML should parse")
}

#[test]
fn validate_processor_chain_accepted() {
    let cfg = chain_pipeline(
        r#"      processors:
        dbg:
          type: processor:debug
          config:
            verbosity: basic"#,
    );
    let gid: PipelineGroupId = "test_group".into();
    let pid: PipelineId = "test_pipeline".into();
    otap_df_controller::startup::validate_pipeline_components(
        &gid,
        &pid,
        &cfg,
        &OTAP_PIPELINE_FACTORY,
    )
    .expect("processor chain with valid sub-processors should pass validation");
}

#[test]
fn validate_processor_chain_unknown_sub_processor_rejected() {
    let cfg = chain_pipeline(
        r#"      processors:
        bad:
          type: "processor:nonexistent_processor""#,
    );
    let gid: PipelineGroupId = "test_group".into();
    let pid: PipelineId = "test_pipeline".into();
    let err = otap_df_controller::startup::validate_pipeline_components(
        &gid,
        &pid,
        &cfg,
        &OTAP_PIPELINE_FACTORY,
    )
    .expect_err("unknown sub-processor should fail validation");
    assert!(
        err.to_string().contains("Unknown processor component"),
        "error should mention unknown processor: {err}"
    );
}

#[test]
fn validate_processor_chain_invalid_config_rejected() {
    let cfg = chain_pipeline(r#"      not_processors: []"#);
    let gid: PipelineGroupId = "test_group".into();
    let pid: PipelineId = "test_pipeline".into();
    let err = otap_df_controller::startup::validate_pipeline_components(
        &gid,
        &pid,
        &cfg,
        &OTAP_PIPELINE_FACTORY,
    )
    .expect_err("invalid chain config should fail validation");
    assert!(
        err.to_string().contains("Invalid processor_chain config"),
        "error should mention invalid config: {err}"
    );
}
