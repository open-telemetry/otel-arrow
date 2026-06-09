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
use otap_df_admin_api::config::pipeline::PipelineConfig;
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
        PipelineConfig::from_json(
            pipeline_group_id.to_string().into(),
            pipeline_id.to_string().into(),
            content,
        )
    } else {
        PipelineConfig::from_yaml(
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
    use otap_df_admin_api::config::pipeline::{PipelineConfigBuilder, PipelineType};

    fn pipeline_config() -> PipelineConfig {
        PipelineConfigBuilder::new()
            .add_receiver("ingress", "receiver:otlp", None)
            .add_exporter("egress", "exporter:debug", None)
            .to("ingress", "egress")
            .build(PipelineType::Otap, "tenant-a", "ingest")
            .expect("pipeline config")
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
}
