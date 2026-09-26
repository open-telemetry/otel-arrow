// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Pipeline-config parsing and serialization helpers shared by the CLI and TUI.
//!
//! Pipeline reconfiguration accepts input from files or stdin and supports both
//! JSON and YAML because automation commonly emits JSON while humans usually
//! edit YAML. Centralizing the detection, parsing, and YAML serialization here
//! keeps command handlers focused on admin operations and gives the TUI editor
//! path the same validation behavior as non-interactive `dfctl` commands.

use crate::error::CliError;
use otel_arrow_dfe_admin_api::config::pipeline::PipelineConfig;
use std::fs;
use std::io::Read;
use std::path::Path;

/// Load a pipeline config from a file path or stdin and parse it as JSON or YAML.
pub(crate) fn load_pipeline_config(
    path: &Path,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<PipelineConfig, CliError> {
    let mut content = String::new();
    if path == Path::new("-") {
        _ = std::io::stdin().read_to_string(&mut content)?;
    } else {
        content = fs::read_to_string(path).map_err(|err| {
            CliError::config(format!(
                "failed to read pipeline file '{}': {err}",
                path.display()
            ))
        })?;
    }

    parse_pipeline_config_content(&content, pipeline_group_id, pipeline_id)
}

/// Parse pipeline config content while preserving the current JSON/YAML auto-detection.
pub(crate) fn parse_pipeline_config_content(
    content: &str,
    pipeline_group_id: &str,
    pipeline_id: &str,
) -> Result<PipelineConfig, CliError> {
    let parse_result = if looks_like_json(content) {
        PipelineConfig::from_json_allowing_inherited_extensions(
            pipeline_group_id.to_string().into(),
            pipeline_id.to_string().into(),
            content,
        )
    } else {
        PipelineConfig::from_yaml_allowing_inherited_extensions(
            pipeline_group_id.to_string().into(),
            pipeline_id.to_string().into(),
            content,
        )
    };

    parse_result.map_err(|err| {
        CliError::config(format!(
            "failed to parse pipeline config for '{}/{}': {err}",
            pipeline_group_id, pipeline_id
        ))
    })
}

/// Serialize a pipeline config to YAML for human editing and diffing flows.
pub(crate) fn serialize_pipeline_config_yaml(
    pipeline: &PipelineConfig,
) -> Result<String, CliError> {
    serde_yaml::to_string(pipeline).map_err(|err| {
        CliError::config(format!(
            "failed to serialize pipeline config to YAML: {err}"
        ))
    })
}

fn looks_like_json(content: &str) -> bool {
    matches!(content.chars().find(|ch| !ch.is_whitespace()), Some('{'))
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_admin_api::config::pipeline::{PipelineConfigBuilder, PipelineType};

    fn pipeline_config() -> PipelineConfig {
        PipelineConfigBuilder::new()
            .add_receiver("ingress", "receiver:otlp", None)
            .add_exporter("egress", "exporter:debug", None)
            .to("ingress", "egress")
            .build(PipelineType::Otap, "tenant-a", "ingest")
            .expect("pipeline config")
    }

    fn inherited_pipeline_definition() -> serde_json::Value {
        serde_json::json!({
            "nodes": {
                "ingress": {"type": "receiver:otlp"},
                "egress": {
                    "type": "exporter:debug",
                    "capabilities": {"bearer_token_provider": "ancestor_auth"}
                }
            },
            "connections": [{"from": "ingress", "to": "egress"}]
        })
    }

    fn pipeline_documents(definition: &serde_json::Value) -> [String; 2] {
        [
            serde_yaml::to_string(definition).expect("pipeline definition serializes as YAML"),
            serde_json::to_string_pretty(definition)
                .expect("pipeline definition serializes as JSON"),
        ]
    }

    /// Scenario: the TUI serializes a committed pipeline config before handing
    /// it to an editor-driven reconfigure flow.
    /// Guarantees: the shared YAML serializer succeeds on a normal pipeline
    /// config so the editor path can round-trip the rendered document.
    #[test]
    fn serialize_pipeline_config_yaml_round_trips_pipeline() {
        let expected = pipeline_config();
        let rendered =
            serialize_pipeline_config_yaml(&expected).expect("pipeline config should serialize");
        let reparsed = parse_pipeline_config_content(&rendered, "tenant-a", "ingest")
            .expect("serialized pipeline config should parse");
        assert!(reparsed.eq_ignoring_policies(&expected));
    }

    /// Scenario: the CLI receives pipeline config content in JSON form.
    /// Guarantees: the shared parser auto-detects JSON and returns the typed
    /// pipeline config without requiring the caller to select the format.
    #[test]
    fn parse_pipeline_config_content_accepts_json() {
        let rendered = serde_json::to_string_pretty(&pipeline_config())
            .expect("pipeline config should serialize to json");
        let parsed = parse_pipeline_config_content(&rendered, "tenant-a", "ingest")
            .expect("json pipeline config should parse");
        assert!(parsed.eq_ignoring_policies(&pipeline_config()));
    }

    /// Scenario: JSON and YAML pipeline documents bind an ancestor capability and round-trip through the editor.
    /// Guarantees: parsing canonicalizes node URNs and preserves bindings without inventing local providers.
    #[test]
    fn parse_pipeline_config_content_accepts_inherited_extension_binding() {
        for content in pipeline_documents(&inherited_pipeline_definition()) {
            let parsed = parse_pipeline_config_content(&content, "tenant-a", "ingest")
                .expect("inherited binding should be deferred to server validation");
            let rendered = serde_json::to_value(&parsed).expect("parsed pipeline should serialize");
            assert_eq!(
                rendered["nodes"]["egress"]["capabilities"]["bearer_token_provider"],
                "ancestor_auth"
            );
            assert_eq!(
                rendered["nodes"]["ingress"]["type"],
                "urn:otel:receiver:otlp"
            );
            assert_eq!(
                rendered["nodes"]["egress"]["type"],
                "urn:otel:exporter:debug"
            );
            assert!(parsed.extensions().is_empty());

            let yaml = serialize_pipeline_config_yaml(&parsed)
                .expect("inherited pipeline should serialize for editing");
            let reparsed = parse_pipeline_config_content(&yaml, "tenant-a", "ingest")
                .expect("edited pipeline should retain inherited binding support");
            assert_eq!(
                serde_json::to_value(reparsed).expect("reparsed pipeline should serialize"),
                rendered
            );
        }
    }

    /// Scenario: an inherited capability binding accompanies a connection to a nonexistent node.
    /// Guarantees: deferring provider lookup does not bypass graph validation in either input format.
    #[test]
    fn parse_pipeline_config_content_rejects_broken_graph_with_inherited_binding() {
        let mut definition = inherited_pipeline_definition();
        definition["connections"][0]["to"] = serde_json::json!("missing");
        for content in pipeline_documents(&definition) {
            let error = parse_pipeline_config_content(&content, "tenant-a", "ingest")
                .expect_err("missing graph target must remain invalid");
            let message = error.to_string();
            assert!(message.contains("missing"), "{message}");
            assert!(message.contains("tenant-a/ingest"), "{message}");
        }
    }

    /// Scenario: an inherited binding accompanies a malformed node URN in JSON or YAML.
    /// Guarantees: ancestor support still rejects invalid component identifiers with pipeline context.
    #[test]
    fn parse_pipeline_config_content_rejects_invalid_urn_with_inherited_binding() {
        let mut definition = inherited_pipeline_definition();
        definition["nodes"]["egress"]["type"] = serde_json::json!("not-a-component-urn");
        for content in pipeline_documents(&definition) {
            let error = parse_pipeline_config_content(&content, "tenant-a", "ingest")
                .expect_err("malformed component identifier must be rejected");
            assert!(
                error.to_string().contains("tenant-a/ingest"),
                "pipeline context should be retained: {error}"
            );
        }
    }

    /// Scenario: CLI/TUI input is malformed JSON or malformed YAML.
    /// Guarantees: parse failures remain explicit and identify the affected pipeline.
    #[test]
    fn parse_pipeline_config_content_rejects_malformed_documents() {
        for content in ["{\"nodes\":", "nodes: [unterminated"] {
            let error = parse_pipeline_config_content(content, "tenant-a", "ingest")
                .expect_err("malformed document must not be accepted");
            assert!(
                error.to_string().contains("tenant-a/ingest"),
                "pipeline context should be retained: {error}"
            );
        }
    }
}
