// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_config::ContextEntryRef;
use otel_arrow_dfe_config::transport_headers::ValueKind;
use otel_arrow_dfe_engine::context_declaration::{ContextValueCardinality, ContextValueKind};

use super::*;

/// Transformation of captured header or materialized context text.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ValueTransform {
    /// Use the text without modification.
    #[default]
    String,
    /// Parse `true` or `false`.
    Bool,
    /// Parse a signed 64-bit integer.
    Int,
    /// Parse a floating-point value.
    Double,
    /// Parse the selected comma-separated authority and remove its port.
    AuthorityHost,
}

/// Comma-separated authority selected before host parsing.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityHostSelection {
    /// Select the first value, preserving the default behavior.
    #[default]
    First,
    /// Select the last value.
    Last,
}

/// An insert, upsert, or update value: either the existing literal or a nested source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ConfiguredValue {
    /// Existing typed literal, with unchanged configuration syntax.
    Literal(LiteralValue),
    /// Explicit typed literal, as documented in existing configuration examples.
    Typed(#[serde(with = "TypedLiteralSerde")] LiteralValue),
    /// Dynamic request metadata source, configured under the `value` key.
    Dynamic(ValueSource),
}

// Adapt the documented tagged syntax without introducing a second value representation.
#[derive(Serialize, Deserialize)]
#[serde(remote = "LiteralValue", rename_all = "lowercase", deny_unknown_fields)]
enum TypedLiteralSerde {
    Bool(bool),
    Int(i64),
    Double(f64),
    #[serde(rename = "string")]
    Str(String),
}

/// Dynamic source options nested beneath an action's `value` key.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ValueSource {
    /// Ordered captured/stored header names.
    #[serde(
        default,
        deserialize_with = "non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_headers: Option<Vec<String>>,
    /// Existing materialized context entry or qualified `entry:member` lookup.
    #[serde(
        default,
        deserialize_with = "non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub from_context: Option<ContextEntryRef>,
    /// Conversion of source text; defaults to string.
    #[serde(
        default,
        deserialize_with = "non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub transform: Option<ValueTransform>,
    /// Comma-list selection for authority_host only; defaults to first.
    #[serde(
        default,
        deserialize_with = "non_null",
        skip_serializing_if = "Option::is_none"
    )]
    pub selection: Option<AuthorityHostSelection>,
}

pub(super) fn non_null<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: Deserialize<'de>,
{
    T::deserialize(deserializer).map(Some)
}

pub(super) fn invalid_config(error: impl Into<String>) -> ConfigError {
    ConfigError::InvalidUserConfig {
        error: error.into(),
    }
}

pub(super) fn invalid_data(error: impl Into<String>) -> EngineError {
    EngineError::PdataConversionError {
        error: error.into(),
    }
}

impl ValueSource {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        if usize::from(self.from_headers.is_some()) + usize::from(self.from_context.is_some()) != 1
        {
            return Err(invalid_config(
                "dynamic value requires exactly one of from_headers or from_context",
            ));
        }
        if self.from_headers.as_ref().is_some_and(|names| {
            names.is_empty() || names.iter().any(|name| name.trim().is_empty())
        }) {
            return Err(invalid_config(
                "source names and from_headers must be nonempty",
            ));
        }
        if self.selection.is_some()
            && !matches!(self.transform, Some(ValueTransform::AuthorityHost))
        {
            return Err(invalid_config(
                "selection is only supported with transform: authority_host",
            ));
        }
        Ok(())
    }

    pub(super) fn resolve(&self, context: &Context) -> Result<Option<LiteralValue>, EngineError> {
        if let Some(names) = &self.from_headers {
            return Ok(context.transport_headers().and_then(|headers| {
                names.iter().find_map(|name| {
                    headers.find_by_name(name).find_map(|header| {
                        if header.value.value_kind != ValueKind::Text {
                            return None;
                        }
                        let text = header.value_as_str()?;
                        if text.trim().is_empty() {
                            return None;
                        }
                        self.transform
                            .unwrap_or_default()
                            .apply(text, self.selection.unwrap_or_default())
                    })
                })
            }));
        }
        let Some(reference) = &self.from_context else {
            return Ok(None);
        };
        let Some(materialized) = context.materialized_context() else {
            return Ok(None);
        };
        let member = if let Some(scope) = reference.scope() {
            materialized
                .entry(scope)
                .and_then(|entry| entry.member(reference.name()))
        } else {
            match materialized.entry(reference.name()) {
                Some(entry) => match entry.members() {
                    [member] => Some(member),
                    _ => {
                        return Err(invalid_data(
                            "from_context requires entry:member for multi-member entries",
                        ));
                    }
                },
                None => None,
            }
        };
        let Some(member) = member else {
            return Ok(None);
        };
        let values = member.values();
        if values.cardinality() != ContextValueCardinality::One {
            return Err(invalid_data(
                "from_context requires a scalar context member",
            ));
        }
        let value = &values.values()[0];
        if value.kind() != ContextValueKind::Text {
            return Err(invalid_data(
                "from_context requires text for scalar attributes",
            ));
        }
        let text = value
            .as_str()
            .ok_or_else(|| invalid_data("from_context contains invalid UTF-8"))?;
        self.transform
            .unwrap_or_default()
            .apply(text, self.selection.unwrap_or_default())
            .map(Some)
            .ok_or_else(|| {
                invalid_data("from_context value cannot be converted using the requested transform")
            })
    }
}

impl ConfiguredValue {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        match self {
            Self::Literal(_) | Self::Typed(_) => Ok(()),
            Self::Dynamic(source) => source.validate(),
        }
    }

    pub(super) fn literal(&self) -> Option<LiteralValue> {
        match self {
            Self::Literal(value) | Self::Typed(value) => Some(value.clone()),
            Self::Dynamic(_) => None,
        }
    }

    pub(super) fn resolve(&self, context: &Context) -> Result<Option<LiteralValue>, EngineError> {
        match self {
            Self::Literal(value) | Self::Typed(value) => Ok(Some(value.clone())),
            Self::Dynamic(source) => source.resolve(context),
        }
    }
}

impl ValueTransform {
    fn apply(self, value: &str, selection: AuthorityHostSelection) -> Option<LiteralValue> {
        match self {
            Self::String => Some(LiteralValue::Str(value.to_owned())),
            Self::Bool => value.parse().ok().map(LiteralValue::Bool),
            Self::Int => value.parse().ok().map(LiteralValue::Int),
            Self::Double => value.parse().ok().map(LiteralValue::Double),
            Self::AuthorityHost => {
                let value = match selection {
                    AuthorityHostSelection::First => {
                        value.split_once(',').map_or(value, |(first, _)| first)
                    }
                    AuthorityHostSelection::Last => {
                        value.rsplit_once(',').map_or(value, |(_, last)| last)
                    }
                }
                .trim();
                let authority = value.parse::<http::uri::Authority>().ok()?;
                // An authority used as a host must not contain userinfo or a malformed port.
                let host = authority.host();
                if value.contains('@') || host.is_empty() {
                    return None;
                }
                let suffix = value.strip_prefix(host)?;
                if !suffix.is_empty() {
                    let _ = suffix.strip_prefix(':')?.parse::<u16>().ok()?;
                }
                Some(LiteralValue::Str(host.to_owned()))
            }
        }
    }
}
