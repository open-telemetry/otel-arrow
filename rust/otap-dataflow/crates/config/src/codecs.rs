// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Optional codec bindings, reusable configurations, and scope resolution.
//!
//! Linked implementations provide defaults. Declarations replace individual
//! bindings or named definitions from pipeline, group, then root scope; they
//! never restrict discovery to the configured encodings. Codec-specific options
//! are validated by the selected implementation at pipeline startup.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Validation requested when decoding encoded data into OTAP.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecodeValidation {
    /// Permit relaxed decoding, which may omit malformed nested content.
    #[default]
    BestEffort,
    /// Require validation of the complete batch before returning records.
    Strict,
}

/// Optional codec overrides. Omission and empty maps preserve inherited bindings.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CodecsConfig {
    /// Reusable settings; declaring a name does not select it for an encoding.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub configurations: BTreeMap<String, CodecConfiguration>,
    /// One selected configuration per encoding in a pipeline, not an allowlist.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub bindings: BTreeMap<String, CodecBinding>,
}

/// Either inline settings or a named reference, never a mixture of both.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(untagged)]
pub enum CodecBinding {
    /// Select a reusable configuration visible at this pipeline's scope.
    Named(CodecConfigurationRef),
    /// Override an encoding directly, using its default implementation if omitted.
    Inline(CodecConfiguration),
}

/// Reference to a named configuration. Additional override fields are rejected.
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CodecConfigurationRef {
    /// Name in `codecs.configurations`.
    pub configuration: String,
}

/// Immutable settings supplied to one implementation's startup preparation hook.
///
/// Native and future hosted implementations use the same configuration shape.
/// Options do not contain module-loading instructions unless an implementation
/// explicitly supports them. Debug output intentionally omits option values.
#[derive(Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CodecConfiguration {
    /// Registered implementation ID. Required for named definitions; optional inline.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub implementation: Option<String>,
    /// Common decoding contract. Omission uses the implementation's default.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub decode: Option<DecodeConfig>,
    /// Implementation-owned options, validated at startup, never interpreted per record.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub options: BTreeMap<String, serde_json::Value>,
}

impl std::fmt::Debug for CodecConfiguration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CodecConfiguration")
            .field("implementation", &self.implementation)
            .field("decode", &self.decode)
            .field("options", &"<redacted>")
            .finish()
    }
}

/// Common decoding settings; these do not validate pass-through or borrowed bytes.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DecodeConfig {
    /// Requested malformed-input detection, applied only when decoding is needed.
    #[serde(default)]
    pub validation: DecodeValidation,
}

/// Scope-resolved declarations for one runtime, without coupling to codec factories.
#[derive(Debug, Clone, Default)]
pub struct ResolvedCodecsConfig {
    /// Visible named definitions. Names not selected do not affect runtime equality.
    pub configurations: BTreeMap<String, CodecConfiguration>,
    /// Nearest binding declaration for each encoding.
    pub bindings: BTreeMap<String, CodecBinding>,
}

impl ResolvedCodecsConfig {
    /// Expands references at startup. Errors contain names, never option values.
    pub fn effective_bindings(&self) -> Result<BTreeMap<String, CodecConfiguration>, String> {
        self.bindings
            .iter()
            .map(|(encoding, binding)| {
                let config = match binding {
                    CodecBinding::Inline(config) => config,
                    CodecBinding::Named(reference) => self
                        .configurations
                        .get(&reference.configuration)
                        .ok_or_else(|| {
                            format!(
                                "codecs.bindings.{encoding}: unknown configuration `{}`",
                                reference.configuration
                            )
                        })?,
                };
                Ok((encoding.clone(), config.clone()))
            })
            .collect()
    }
}

impl PartialEq for ResolvedCodecsConfig {
    fn eq(&self, other: &Self) -> bool {
        match (self.effective_bindings(), other.effective_bindings()) {
            (Ok(left), Ok(right)) => left == right,
            _ => self.configurations == other.configurations && self.bindings == other.bindings,
        }
    }
}
impl Eq for ResolvedCodecsConfig {}

impl CodecsConfig {
    /// Resolves scopes in descending precedence; each map entry replaces as a unit.
    #[must_use]
    pub fn resolve<'a>(scopes: impl IntoIterator<Item = &'a Self>) -> ResolvedCodecsConfig {
        let mut result = ResolvedCodecsConfig::default();
        for scope in scopes {
            for (name, config) in &scope.configurations {
                _ = result
                    .configurations
                    .entry(name.clone())
                    .or_insert_with(|| config.clone());
            }
            for (encoding, binding) in &scope.bindings {
                _ = result
                    .bindings
                    .entry(encoding.clone())
                    .or_insert_with(|| binding.clone());
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Empty scopes inherit maps and a nearer binding replaces an entire parent entry.
    /// Guarantees: Defaults stay implicit, options never deep-merge, and named settings resolve at pipeline scope.
    #[test]
    fn inheritance_and_named_resolution() {
        let parent: CodecsConfig = serde_yaml::from_str("configurations:\n  checked:\n    implementation: otel.otlp\n    decode: {validation: strict}\nbindings:\n  otlp-bytes: {configuration: checked}\n  other: {options: {limit: 4}}").unwrap();
        let empty = CodecsConfig::default();
        assert_eq!(
            CodecsConfig::resolve([&empty, &parent]),
            CodecsConfig::resolve([&parent])
        );
        let child: CodecsConfig =
            serde_yaml::from_str("bindings:\n  otlp-bytes: {}\n  other: {options: {size: 2}}")
                .unwrap();
        let resolved = CodecsConfig::resolve([&child, &parent])
            .effective_bindings()
            .unwrap();
        assert_eq!(resolved["otlp-bytes"], CodecConfiguration::default());
        assert!(!resolved["other"].options.contains_key("limit"));
        assert_eq!(resolved["other"].options["size"], 2);
        let shadow: CodecsConfig =
            serde_yaml::from_str("configurations: {checked: {implementation: otel.otlp}}").unwrap();
        let shadowed = CodecsConfig::resolve([&shadow, &parent])
            .effective_bindings()
            .unwrap();
        assert!(shadowed["otlp-bytes"].decode.is_none());
        let named = CodecsConfig::resolve([&parent])
            .effective_bindings()
            .unwrap();
        assert_eq!(
            named["otlp-bytes"].decode.as_ref().unwrap().validation,
            DecodeValidation::Strict
        );
    }

    /// Scenario: References, unsupported fields, or mixed inline/reference settings are supplied.
    /// Guarantees: Typos and obsolete defaults fail instead of silently selecting relaxed decoding.
    #[test]
    fn invalid_declarations_are_rejected() {
        for yaml in [
            "defaults: {}",
            "encodings: {}",
            "bindings: {otlp-bytes: {configuration: checked, decode: {}}}",
            "bindings: {otlp-bytes: {decode: {validation: strictt}}}",
            "bindings: {otlp-bytes: {module: codec.wasm}}",
        ] {
            assert!(
                serde_yaml::from_str::<CodecsConfig>(yaml).is_err(),
                "{yaml}"
            );
        }
        let missing: CodecsConfig =
            serde_yaml::from_str("bindings: {otlp-bytes: {configuration: absent}}").unwrap();
        assert!(
            CodecsConfig::resolve([&missing])
                .effective_bindings()
                .unwrap_err()
                .contains("absent")
        );
    }

    /// Scenario: A binding switches between equivalent inline/named declarations while unused names change.
    /// Guarantees: Runtime equality tracks selected settings, and Debug never exposes option values.
    #[test]
    fn equality_and_diagnostics_follow_effective_settings() {
        let inline: CodecsConfig = serde_yaml::from_str(
            "bindings: {otlp-bytes: {implementation: otel.otlp, options: {token: secret}}}",
        )
        .unwrap();
        let named: CodecsConfig = serde_yaml::from_str("configurations: {checked: {implementation: otel.otlp, options: {token: secret}}, unused: {implementation: unused}}\nbindings: {otlp-bytes: {configuration: checked}}").unwrap();
        assert_eq!(
            CodecsConfig::resolve([&inline]),
            CodecsConfig::resolve([&named])
        );
        assert!(!format!("{named:?}").contains("secret"));
    }
}
