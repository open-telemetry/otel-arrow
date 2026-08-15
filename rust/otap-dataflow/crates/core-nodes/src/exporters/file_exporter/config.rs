// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Typed configuration and deterministic path rendering for the file exporter.
//!
//! Validation rejects ambiguous file ownership before filesystem access. Runtime rendering then
//! substitutes the closed signal name and numeric core and deployment-generation identifiers.

use otap_df_config::SignalType;
use otap_df_config::error::Error as ConfigError;
use serde::Deserialize;
use serde_json::Value;
use std::path::{Component, Path, PathBuf};

const DEFAULT_MAX_FRAME_BYTES: usize = 64 * 1024 * 1024;
const MAX_MAX_FRAME_BYTES: usize = 256 * 1024 * 1024;
const TOKENS: [&str; 3] = ["{signal}", "{core_id}", "{generation}"];

/// File format supported by the exporter.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum FileFormat {
    /// Compact OTLP ProtoJSON with one pdata batch per line.
    #[default]
    OtlpJson,
}

/// Behavior when a signal file is first opened.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OpenMode {
    /// Retain complete existing frames and append new frames.
    #[default]
    Append,
    /// Discard existing contents on first open.
    Truncate,
    /// Refuse to open a path that already exists.
    CreateNew,
}

/// Durability point that must complete before an ACK is routed.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Durability {
    /// ACK after the operating system accepts and flushes the write operation.
    #[default]
    Write,
    /// ACK after `sync_data` completes for the signal file.
    SyncData,
}

/// Recovery policy for an incomplete append-mode file tail.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TailRecovery {
    /// Remove a bounded incomplete final frame.
    #[default]
    TruncatePartial,
    /// Refuse a file whose final byte is not a newline.
    Fail,
}

/// Typed file exporter configuration.
#[derive(Debug, Clone, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileExporterConfig {
    /// Absolute path template containing all required runtime tokens.
    pub path: String,
    /// Create missing parent directories.
    #[serde(default)]
    pub create_directories: bool,
    /// Output format.
    #[serde(default)]
    pub format: FileFormat,
    /// First-open behavior.
    #[serde(default)]
    pub open_mode: OpenMode,
    /// ACK durability point.
    #[serde(default)]
    pub durability: Durability,
    /// Maximum encoded frame size, including the newline.
    #[serde(default = "default_max_frame_bytes")]
    pub max_frame_bytes: usize,
    /// Explicit append-mode crash-tail policy.
    #[serde(default)]
    pub tail_recovery: Option<TailRecovery>,
}

const fn default_max_frame_bytes() -> usize {
    DEFAULT_MAX_FRAME_BYTES
}

impl FileExporterConfig {
    /// Parses and validates syntax and field relationships without filesystem access.
    pub fn parse(value: &Value) -> Result<Self, ConfigError> {
        let config: Self = serde_json::from_value(value.clone()).map_err(|error| {
            ConfigError::InvalidUserConfig {
                error: format!("Failed to parse file exporter config: {error}"),
            }
        })?;
        config.validate()?;
        Ok(config)
    }

    /// Returns the effective append-mode tail policy.
    #[must_use]
    pub const fn effective_tail_recovery(&self) -> Option<TailRecovery> {
        match self.open_mode {
            OpenMode::Append => Some(match self.tail_recovery {
                Some(value) => value,
                None => TailRecovery::TruncatePartial,
            }),
            OpenMode::Truncate | OpenMode::CreateNew => None,
        }
    }

    /// Resolves all signal paths for one pipeline runtime and rejects collisions.
    pub fn render_paths(
        &self,
        core_id: usize,
        deployment_generation: u64,
    ) -> Result<RenderedPaths, ConfigError> {
        let render = |signal: &str| {
            PathBuf::from(
                self.path
                    .replace("{signal}", signal)
                    .replace("{core_id}", &core_id.to_string())
                    .replace("{generation}", &deployment_generation.to_string()),
            )
        };
        let paths = RenderedPaths {
            logs: render("logs"),
            metrics: render("metrics"),
            traces: render("traces"),
        };
        let normalized = [
            normalize_lexically(&paths.logs),
            normalize_lexically(&paths.metrics),
            normalize_lexically(&paths.traces),
        ];
        if normalized[0] == normalized[1]
            || normalized[0] == normalized[2]
            || normalized[1] == normalized[2]
        {
            return Err(invalid(
                "file.path resolves multiple signals to the same path",
            ));
        }
        Ok(paths)
    }

    fn validate(&self) -> Result<(), ConfigError> {
        if !Path::new(&self.path).is_absolute() {
            return Err(invalid("file.path must be absolute"));
        }
        for token in TOKENS {
            if self.path.matches(token).count() != 1 {
                return Err(invalid(format!(
                    "file.path must contain {token} exactly once"
                )));
            }
        }
        let without_known_tokens = TOKENS
            .iter()
            .fold(self.path.clone(), |path, token| path.replace(token, ""));
        if without_known_tokens.contains('{') || without_known_tokens.contains('}') {
            return Err(invalid("file.path contains an unknown or malformed token"));
        }
        if !(1..=MAX_MAX_FRAME_BYTES).contains(&self.max_frame_bytes) {
            return Err(invalid(format!(
                "file.max_frame_bytes must be in the range 1..={MAX_MAX_FRAME_BYTES}"
            )));
        }
        if self.open_mode != OpenMode::Append && self.tail_recovery.is_some() {
            return Err(invalid(
                "file.tail_recovery is only valid with open_mode=append",
            ));
        }
        Ok(())
    }
}

/// Signal-specific paths resolved for one core and deployment generation.
#[derive(Debug, Clone)]
pub struct RenderedPaths {
    logs: PathBuf,
    metrics: PathBuf,
    traces: PathBuf,
}

impl RenderedPaths {
    /// Returns the resolved path for a signal.
    pub fn get(&self, signal: SignalType) -> &Path {
        match signal {
            SignalType::Logs => &self.logs,
            SignalType::Metrics => &self.metrics,
            SignalType::Traces => &self.traces,
        }
    }
}

pub(crate) fn normalize_lexically(path: &Path) -> PathBuf {
    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::CurDir => {}
            Component::ParentDir => {
                let _ = normalized.pop();
            }
            Component::Prefix(_) | Component::RootDir | Component::Normal(_) => {
                normalized.push(component.as_os_str());
            }
        }
    }
    normalized
}

fn invalid(error: impl Into<String>) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn absolute_template() -> String {
        std::env::temp_dir()
            .join("otel-{signal}-{core_id}-{generation}.jsonl")
            .to_string_lossy()
            .into_owned()
    }

    /// Scenario: Only the required path is supplied for an append-mode exporter.
    /// Guarantees: All documented defaults are selected, including partial-tail recovery.
    #[test]
    fn minimal_config_uses_documented_defaults() {
        let config = FileExporterConfig::parse(&json!({"path": absolute_template()})).unwrap();
        assert_eq!(config.format, FileFormat::OtlpJson);
        assert_eq!(config.open_mode, OpenMode::Append);
        assert_eq!(config.durability, Durability::Write);
        assert_eq!(config.max_frame_bytes, DEFAULT_MAX_FRAME_BYTES);
        assert_eq!(
            config.effective_tail_recovery(),
            Some(TailRecovery::TruncatePartial)
        );
    }

    /// Scenario: A path omits, repeats, or misspells a required runtime token.
    /// Guarantees: Configuration validation rejects every ambiguous ownership template.
    #[test]
    fn path_requires_each_known_token_exactly_once() {
        for path in [
            "/tmp/{core_id}-{generation}.jsonl",
            "/tmp/{signal}-{signal}-{core_id}-{generation}.jsonl",
            "/tmp/{signal}-{core_id}-{generation}-{unknown}.jsonl",
        ] {
            assert!(FileExporterConfig::parse(&json!({"path": path})).is_err());
        }
    }

    /// Scenario: Configuration contains an unknown field, invalid enum, path, or size bound.
    /// Guarantees: Schema and range violations fail before runtime path rendering or file access.
    #[test]
    fn rejects_invalid_schema_and_range_values() {
        for value in [
            json!({"path": absolute_template(), "unknown": true}),
            json!({"path": absolute_template(), "format": "protobuf"}),
            json!({"path": "relative-{signal}-{core_id}-{generation}.jsonl"}),
            json!({"path": absolute_template(), "max_frame_bytes": 0}),
            json!({
                "path": absolute_template(),
                "max_frame_bytes": MAX_MAX_FRAME_BYTES + 1
            }),
        ] {
            assert!(FileExporterConfig::parse(&value).is_err());
        }
    }

    /// Scenario: Tail recovery is explicitly configured for a destructive first-open mode.
    /// Guarantees: A meaningless cross-field combination is rejected before filesystem access.
    #[test]
    fn tail_recovery_requires_append_mode() {
        let error = FileExporterConfig::parse(&json!({
            "path": absolute_template(),
            "open_mode": "truncate",
            "tail_recovery": "fail"
        }));
        assert!(error.is_err());
    }

    /// Scenario: Runtime identifiers and each signal are substituted into a valid path template.
    /// Guarantees: Logs, metrics, and traces resolve deterministically to distinct absolute paths.
    #[test]
    fn renders_signal_core_and_generation_paths() {
        let config = FileExporterConfig::parse(&json!({"path": absolute_template()})).unwrap();
        let paths = config.render_paths(3, 7).unwrap();
        assert!(paths.get(SignalType::Logs).ends_with("otel-logs-3-7.jsonl"));
        assert!(
            paths
                .get(SignalType::Metrics)
                .ends_with("otel-metrics-3-7.jsonl")
        );
        assert!(
            paths
                .get(SignalType::Traces)
                .ends_with("otel-traces-3-7.jsonl")
        );
    }

    /// Scenario: Lexical parent components make three rendered signal paths equivalent.
    /// Guarantees: Configuration-time path ownership validation rejects cross-signal collisions.
    #[test]
    fn rejects_normalized_cross_signal_collision() {
        let root = std::env::temp_dir().to_string_lossy().into_owned();
        let path = format!("{root}/{{signal}}/../same-{{core_id}}-{{generation}}.jsonl");
        let config = FileExporterConfig::parse(&json!({"path": path})).unwrap();
        assert!(config.render_paths(1, 2).is_err());
    }
}
