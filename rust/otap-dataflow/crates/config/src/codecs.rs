// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! User-facing codec settings and their pipeline-local resolution.
//!
//! Declarations inherit from pipeline, group, then top-level configuration.
//! Only explicit values override a parent; empty nested sections inherit.
//! Resolution is independent of the codec registry and creates no codec state.
//! The engine maps the resolved settings to codec runtime types at startup.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Validation required when a decoder materializes encoded data into OTAP.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DecodeValidation {
    /// Permit relaxed decoding, which may omit malformed nested content.
    #[default]
    BestEffort,
    /// Require validation of the complete encoded batch before returning records.
    Strict,
}

/// Settings shared by all codecs in a pipeline; this does not register codecs.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CodecsConfig {
    /// Defaults for every linked codec used in this pipeline.
    #[serde(default)]
    pub defaults: CodecDefaults,
}

/// Common codec defaults. Per-encoding overrides are not currently supported.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CodecDefaults {
    /// Settings applied when encoded data must be decoded.
    #[serde(default)]
    pub decode: DecodeConfig,
}

/// Decode settings declared at one configuration scope.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DecodeConfig {
    /// Omission inherits the parent setting, ultimately defaulting to best effort.
    /// Forwarding and directly borrowed encoded views do not invoke decoding.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub validation: Option<DecodeValidation>,
}

/// Immutable effective codec configuration for one pipeline runtime.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ResolvedCodecsConfig {
    /// Validation supplied to each lazily created decoder.
    pub decode_validation: DecodeValidation,
}

impl CodecsConfig {
    /// Resolves scopes in descending precedence, applying defaults only at the end.
    #[must_use]
    pub fn resolve<'a>(scopes: impl IntoIterator<Item = &'a Self>) -> ResolvedCodecsConfig {
        ResolvedCodecsConfig {
            decode_validation: scopes
                .into_iter()
                .find_map(|scope| scope.defaults.decode.validation)
                .unwrap_or_default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: Codec settings are absent, empty, inherited, or explicitly overridden.
    /// Guarantees: Empty sections preserve parent settings and explicit best effort overrides strict.
    #[test]
    fn defaults_and_explicit_overrides() {
        let strict: CodecsConfig =
            serde_yaml::from_str("defaults:\n  decode:\n    validation: strict").unwrap();
        let best_effort: CodecsConfig =
            serde_yaml::from_str("defaults:\n  decode:\n    validation: best_effort").unwrap();
        for yaml in ["{}", "defaults: {}", "defaults: { decode: {} }"] {
            let empty: CodecsConfig = serde_yaml::from_str(yaml).unwrap();
            assert_eq!(
                CodecsConfig::resolve([&empty]).decode_validation,
                DecodeValidation::BestEffort
            );
            assert_eq!(
                CodecsConfig::resolve([&empty, &strict]).decode_validation,
                DecodeValidation::Strict
            );
            assert_eq!(
                CodecsConfig::resolve([&best_effort, &strict]).decode_validation,
                DecodeValidation::BestEffort
            );
        }
    }

    /// Scenario: A codec declaration contains a typo, unsupported override, or unknown mode.
    /// Guarantees: Invalid declarations are rejected rather than silently using relaxed decoding.
    #[test]
    fn unknown_settings_are_rejected() {
        for yaml in [
            "encodings: {}",
            "defaults: { encoding: otlp-bytes }",
            "defaults: { decode: { strict: true } }",
            "defaults: { decode: { validation: strictt } }",
            "defaults: { decode: { validation: true } }",
        ] {
            assert!(
                serde_yaml::from_str::<CodecsConfig>(yaml).is_err(),
                "{yaml}"
            );
        }
    }
}
