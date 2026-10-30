// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Configuration of the filter processor.
//!
//! The types in this module mirror the user-facing JSON/YAML config shape and
//! are `Deserialize`-able. They are compiled into the precompiled filter types
//! in [`otel_arrow_dfe_pdata::otap::filter`] via [`Config::compile`], which
//! compiles every regex pattern once (rejecting invalid patterns) instead of
//! recompiling on each batch.

use otel_arrow_dfe_config::error::Error as ConfigError;
use otel_arrow_dfe_pdata::otap::filter::{
    AttrMatcher, Pattern, ValueMatcher,
    logs::{LogFilter, LogMatchProperties, LogSeverityNumberMatchProperties},
    metrics::{MetricFilter, MetricMatchProperties},
    traces::{TraceFilter, TraceMatchProperties},
};
use regex::Regex;
use serde::Deserialize;

/// Top-level filter processor configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub metrics: MetricFilterConfig,
    #[serde(default)]
    pub logs: LogFilterConfig,
    #[serde(default)]
    pub traces: TraceFilterConfig,
}

impl Config {
    /// Compile the config into the precompiled filter types, compiling every
    /// regex pattern once. Returns a [`ConfigError`] if any pattern is invalid.
    pub fn compile(&self) -> Result<Filters, ConfigError> {
        Ok(Filters {
            metrics: self.metrics.compile()?,
            logs: self.logs.compile()?,
            traces: self.traces.compile()?,
        })
    }
}

/// The compiled filters for each signal.
#[derive(Debug, Clone)]
pub struct Filters {
    /// Compiled metrics filter.
    pub metrics: MetricFilter,
    /// Compiled logs filter.
    pub logs: LogFilter,
    /// Compiled traces filter.
    pub traces: TraceFilter,
}

/// How to match string values.
#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MatchType {
    /// Match the string values exactly.
    #[default]
    Strict,
    /// Apply the string values as regular expressions.
    Regexp,
}

impl MatchType {
    /// Compile a single pattern string against this match type.
    fn compile(&self, value: &str) -> Result<Pattern, ConfigError> {
        match self {
            MatchType::Strict => Ok(Pattern::Exact(value.to_string())),
            MatchType::Regexp => {
                let regex = Regex::new(value).map_err(|e| ConfigError::InvalidUserConfig {
                    error: format!("invalid regex '{value}': {e}"),
                })?;
                Ok(Pattern::Regex(regex))
            }
        }
    }
}

/// A value that may be any supported scalar type.
#[derive(Debug, Clone, PartialEq, Deserialize)]
#[serde(untagged)]
pub enum AnyValueConfig {
    /// String value.
    String(String),
    /// Int value.
    Int(i64),
    /// Double value.
    Double(f64),
    /// Boolean value.
    Boolean(bool),
}

impl AnyValueConfig {
    /// Compile this value into a [`ValueMatcher`]. Array and nested key-value
    /// attributes are not supported.
    fn compile(&self, match_type: &MatchType) -> Result<ValueMatcher, ConfigError> {
        Ok(match self {
            AnyValueConfig::String(value) => ValueMatcher::String(match_type.compile(value)?),
            AnyValueConfig::Int(value) => ValueMatcher::Int(*value),
            AnyValueConfig::Double(value) => ValueMatcher::Double(*value),
            AnyValueConfig::Boolean(value) => ValueMatcher::Bool(*value),
        })
    }
}

/// An attribute key paired with a value to match.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct KeyValueConfig {
    /// Attribute key.
    pub key: String,
    /// Attribute value.
    pub value: AnyValueConfig,
}

impl KeyValueConfig {
    /// Create a new key value pair.
    #[must_use]
    pub const fn new(key: String, value: AnyValueConfig) -> Self {
        Self { key, value }
    }

    fn compile(&self, match_type: &MatchType) -> Result<AttrMatcher, ConfigError> {
        Ok(AttrMatcher {
            key: self.key.clone(),
            value: self.value.compile(match_type)?,
        })
    }
}

fn compile_attrs(
    attrs: &[KeyValueConfig],
    match_type: &MatchType,
) -> Result<Vec<AttrMatcher>, ConfigError> {
    attrs.iter().map(|kv| kv.compile(match_type)).collect()
}

fn compile_patterns(
    values: &[String],
    match_type: &MatchType,
) -> Result<Vec<Pattern>, ConfigError> {
    values.iter().map(|v| match_type.compile(v)).collect()
}

fn compile_values(
    values: &[AnyValueConfig],
    match_type: &MatchType,
) -> Result<Vec<ValueMatcher>, ConfigError> {
    values.iter().map(|v| v.compile(match_type)).collect()
}

// --- Logs ---------------------------------------------------------------------

/// Log filter configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct LogFilterConfig {
    include: Option<LogMatchPropertiesConfig>,
    exclude: Option<LogMatchPropertiesConfig>,
    // ToDo: Add OTTL Support and accept OTTL expressions
}

impl LogFilterConfig {
    /// Create a new log filter config.
    #[must_use]
    pub const fn new(
        include: Option<LogMatchPropertiesConfig>,
        exclude: Option<LogMatchPropertiesConfig>,
    ) -> Self {
        Self { include, exclude }
    }

    fn compile(&self) -> Result<LogFilter, ConfigError> {
        Ok(LogFilter::new(
            self.include.as_ref().map(|p| p.compile()).transpose()?,
            self.exclude.as_ref().map(|p| p.compile()).transpose()?,
        ))
    }
}

/// Log match properties configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct LogMatchPropertiesConfig {
    #[serde(default)]
    match_type: MatchType,
    #[serde(default)]
    resource_attributes: Vec<KeyValueConfig>,
    #[serde(default)]
    record_attributes: Vec<KeyValueConfig>,
    #[serde(default)]
    severity_texts: Vec<String>,
    severity_number: Option<LogSeverityNumberConfig>,
    #[serde(default)]
    bodies: Vec<AnyValueConfig>,
}

impl LogMatchPropertiesConfig {
    /// Create a new log match properties config.
    #[must_use]
    pub const fn new(
        match_type: MatchType,
        resource_attributes: Vec<KeyValueConfig>,
        record_attributes: Vec<KeyValueConfig>,
        severity_texts: Vec<String>,
        severity_number: Option<LogSeverityNumberConfig>,
        bodies: Vec<AnyValueConfig>,
    ) -> Self {
        Self {
            match_type,
            resource_attributes,
            record_attributes,
            severity_texts,
            severity_number,
            bodies,
        }
    }

    fn compile(&self) -> Result<LogMatchProperties, ConfigError> {
        Ok(LogMatchProperties::new(
            compile_attrs(&self.resource_attributes, &self.match_type)?,
            compile_attrs(&self.record_attributes, &self.match_type)?,
            compile_patterns(&self.severity_texts, &self.match_type)?,
            self.severity_number.as_ref().map(|s| s.compile()),
            compile_values(&self.bodies, &self.match_type)?,
        ))
    }
}

/// Log severity number match properties configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct LogSeverityNumberConfig {
    min: i32,
    match_undefined: bool,
}

impl LogSeverityNumberConfig {
    /// Create a new log severity number config.
    #[must_use]
    pub const fn new(min: i32, match_undefined: bool) -> Self {
        Self {
            min,
            match_undefined,
        }
    }

    fn compile(&self) -> LogSeverityNumberMatchProperties {
        LogSeverityNumberMatchProperties::new(self.min, self.match_undefined)
    }
}

// --- Metrics ------------------------------------------------------------------

/// Metric filter configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MetricFilterConfig {
    include: Option<MetricMatchPropertiesConfig>,
    exclude: Option<MetricMatchPropertiesConfig>,
}

impl MetricFilterConfig {
    /// Create a new metric filter config.
    #[must_use]
    pub const fn new(
        include: Option<MetricMatchPropertiesConfig>,
        exclude: Option<MetricMatchPropertiesConfig>,
    ) -> Self {
        Self { include, exclude }
    }

    fn compile(&self) -> Result<MetricFilter, ConfigError> {
        Ok(MetricFilter::new(
            self.include.as_ref().map(|p| p.compile()).transpose()?,
            self.exclude.as_ref().map(|p| p.compile()).transpose()?,
        ))
    }
}

/// Metric match properties configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct MetricMatchPropertiesConfig {
    #[serde(default)]
    match_type: MatchType,
    #[serde(default)]
    metric_names: Vec<String>,
}

impl MetricMatchPropertiesConfig {
    /// Create a new metric match properties config.
    #[must_use]
    pub const fn new(match_type: MatchType, metric_names: Vec<String>) -> Self {
        Self {
            match_type,
            metric_names,
        }
    }

    fn compile(&self) -> Result<MetricMatchProperties, ConfigError> {
        Ok(MetricMatchProperties::new(compile_patterns(
            &self.metric_names,
            &self.match_type,
        )?))
    }
}

// --- Traces -------------------------------------------------------------------

/// Trace filter configuration.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct TraceFilterConfig {
    include: Option<TraceMatchPropertiesConfig>,
    exclude: Option<TraceMatchPropertiesConfig>,
}

impl TraceFilterConfig {
    /// Create a new trace filter config.
    #[must_use]
    pub const fn new(
        include: Option<TraceMatchPropertiesConfig>,
        exclude: Option<TraceMatchPropertiesConfig>,
    ) -> Self {
        Self { include, exclude }
    }

    fn compile(&self) -> Result<TraceFilter, ConfigError> {
        Ok(TraceFilter::new(
            self.include.as_ref().map(|p| p.compile()).transpose()?,
            self.exclude.as_ref().map(|p| p.compile()).transpose()?,
        ))
    }
}

/// Trace match properties configuration.
#[derive(Debug, Clone, Deserialize)]
pub struct TraceMatchPropertiesConfig {
    #[serde(default)]
    match_type: MatchType,
    #[serde(default)]
    resource_attributes: Vec<KeyValueConfig>,
    #[serde(default)]
    span_attributes: Vec<KeyValueConfig>,
    #[serde(default)]
    span_names: Vec<String>,
    #[serde(default)]
    event_names: Vec<String>,
    #[serde(default)]
    event_attributes: Vec<KeyValueConfig>,
    #[serde(default)]
    link_attributes: Vec<KeyValueConfig>,
}

impl TraceMatchPropertiesConfig {
    /// Create a new trace match properties config.
    #[must_use]
    pub const fn new(
        match_type: MatchType,
        resource_attributes: Vec<KeyValueConfig>,
        span_attributes: Vec<KeyValueConfig>,
        span_names: Vec<String>,
        event_names: Vec<String>,
        event_attributes: Vec<KeyValueConfig>,
        link_attributes: Vec<KeyValueConfig>,
    ) -> Self {
        Self {
            match_type,
            resource_attributes,
            span_attributes,
            span_names,
            event_names,
            event_attributes,
            link_attributes,
        }
    }

    fn compile(&self) -> Result<TraceMatchProperties, ConfigError> {
        Ok(TraceMatchProperties::new(
            compile_attrs(&self.resource_attributes, &self.match_type)?,
            compile_attrs(&self.span_attributes, &self.match_type)?,
            compile_patterns(&self.span_names, &self.match_type)?,
            compile_patterns(&self.event_names, &self.match_type)?,
            compile_attrs(&self.event_attributes, &self.match_type)?,
            compile_attrs(&self.link_attributes, &self.match_type)?,
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Scenario: a config with a syntactically invalid regex pattern is compiled.
    /// Guarantees: compilation fails with InvalidUserConfig rather than deferring
    /// the error to the first batch (where it would previously panic).
    #[test]
    fn test_compile_rejects_invalid_regex() {
        let config = Config {
            metrics: MetricFilterConfig::new(
                Some(MetricMatchPropertiesConfig::new(
                    MatchType::Regexp,
                    vec!["(".to_string()],
                )),
                None,
            ),
            ..Default::default()
        };
        let err = config
            .compile()
            .expect_err("invalid regex should be rejected");
        assert!(matches!(err, ConfigError::InvalidUserConfig { .. }));
    }

    /// Scenario: a strict (non-regex) pattern that is not valid regex syntax is
    /// compiled.
    /// Guarantees: strict patterns are treated as literals and never fail to
    /// compile, even when they contain regex metacharacters.
    #[test]
    fn test_compile_strict_pattern_is_literal() {
        let config = Config {
            metrics: MetricFilterConfig::new(
                Some(MetricMatchPropertiesConfig::new(
                    MatchType::Strict,
                    vec!["(".to_string()],
                )),
                None,
            ),
            ..Default::default()
        };
        let _ = config.compile().expect("strict patterns always compile");
    }

    /// Scenario: the documented filter-processor config shape is deserialized and
    /// compiled.
    /// Guarantees: a realistic regexp config round-trips from JSON through compile
    /// without error.
    #[test]
    fn test_deserialize_and_compile_roundtrip() {
        let value = json!({
            "metrics": {
                "exclude": {
                    "match_type": "regexp",
                    "metric_names": ["^debug_.*$"]
                }
            },
            "logs": {
                "include": {
                    "match_type": "regexp",
                    "severity_texts": ["^(WARN|ERROR)$"]
                }
            }
        });
        let config: Config = serde_json::from_value(value).expect("valid config");
        let _ = config.compile().expect("compiles");
    }
}
