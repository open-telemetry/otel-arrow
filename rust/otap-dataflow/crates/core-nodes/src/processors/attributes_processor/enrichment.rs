// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_config::context::transport_headers::ValueKind;
use otel_arrow_dfe_engine::pdata_context::{
    ContextDomain, ContextEntrySelector, ContextEntrySelectorForm, ContextEntryTake,
    ContextEntryTarget, ContextNameMatching, ContextValueCardinality, ContextValueKind,
};

use super::*;

/// An insert, upsert, or update value: either the existing literal or a nested source.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum ConfiguredValue {
    /// Existing typed literal, with unchanged configuration syntax.
    Literal(LiteralValue),
    /// Explicit typed literal, as documented in existing configuration examples.
    Typed(#[serde(with = "TypedLiteralSerde")] LiteralValue),
    /// Dynamic request metadata source, configured under the `value` key.
    Dynamic {
        /// Ordered primitive names or composite references.
        source: AttributeSource,
    },
}

/// Processor-local selection of captured primitives or materialized context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeSource {
    /// Context domain to read without changing receiver capture policy.
    #[serde(rename = "type")]
    pub source_type: AttributeSourceType,
    /// Ordered fallback candidates.
    pub named: Vec<otel_arrow_dfe_config::ContextEntryRef>,
}

/// Context domains supported by attribute value projection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttributeSourceType {
    /// Captured transport-header values.
    TransportHeader,
    /// Captured verified identity claims.
    AuthorizedIdentity,
    /// Materialized composites or qualified members.
    Named,
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

pub(super) fn validate_source(source: &AttributeSource) -> Result<(), ConfigError> {
    if source.named.is_empty() {
        return Err(invalid_config(
            "attribute source named list must not be empty",
        ));
    }
    if source.source_type != AttributeSourceType::Named
        && source.named.iter().any(|name| name.scope().is_some())
    {
        return Err(invalid_config(
            "primitive attribute sources do not accept qualified names",
        ));
    }
    Ok(())
}

pub(super) fn context_entries(
    source: &AttributeSource,
) -> impl Iterator<Item = ContextEntrySelector> + '_ {
    source.named.iter().map(|reference| {
        let target = match source.source_type {
            AttributeSourceType::TransportHeader | AttributeSourceType::AuthorizedIdentity => {
                ContextEntryTarget::Primitive {
                    domain: if source.source_type == AttributeSourceType::TransportHeader {
                        ContextDomain::TransportHeader
                    } else {
                        ContextDomain::AuthorizedIdentity
                    },
                    name: reference.name().clone(),
                    matching: if source.source_type == AttributeSourceType::TransportHeader {
                        ContextNameMatching::AsciiCaseInsensitive
                    } else {
                        ContextNameMatching::Exact
                    },
                }
            }
            _ => match reference.scope() {
                Some(composite) => ContextEntryTarget::CompositeMember {
                    composite: composite.clone(),
                    member: reference.name().clone(),
                },
                None => ContextEntryTarget::Composite {
                    name: reference.name().clone(),
                },
            },
        };
        ContextEntrySelector {
            target,
            form: ContextEntrySelectorForm::Value,
            take: ContextEntryTake::All,
        }
    })
}

pub(super) fn resolve_source(
    source: &AttributeSource,
    context: &Context,
) -> Result<Option<LiteralValue>, EngineError> {
    let names = &source.named;
    if source.source_type == AttributeSourceType::TransportHeader {
        return Ok(context.transport_headers().and_then(|headers| {
            names.iter().find_map(|name| {
                headers
                    .find_by_name(name.name().as_str())
                    .find_map(|header| {
                        if header.value.value_kind != ValueKind::Text {
                            return None;
                        }
                        let text = header.value_as_str()?;
                        if text.trim().is_empty() {
                            return None;
                        }
                        Some(LiteralValue::Str(text.to_owned()))
                    })
            })
        }));
    }
    for reference in names {
        let value = if source.source_type == AttributeSourceType::AuthorizedIdentity {
            let Some(entry) = context
                .authorized_identity_entries()
                .and_then(|entries| entries.get(reference.name().as_str()))
            else {
                continue;
            };
            let text = entry.value().as_str().ok_or_else(|| {
                invalid_data("authorized_identity requires a scalar context entry")
            })?;
            Some(LiteralValue::Str(text.to_owned()))
        } else {
            resolve_composite(context, reference)?
        };
        if value.is_some() {
            return Ok(value);
        }
    }
    Ok(None)
}

fn resolve_composite(
    context: &Context,
    reference: &otel_arrow_dfe_config::ContextEntryRef,
) -> Result<Option<LiteralValue>, EngineError> {
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
                        "named source requires entry:member for multi-member entries",
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
            "named source requires a scalar context member",
        ));
    }
    let value = &values.values()[0];
    if value.kind() != ContextValueKind::Text {
        return Err(invalid_data(
            "named source requires text for scalar attributes",
        ));
    }
    let text = value
        .as_str()
        .ok_or_else(|| invalid_data("named source contains invalid UTF-8"))?;
    Ok(Some(LiteralValue::Str(text.to_owned())))
}
impl ConfiguredValue {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        match self {
            Self::Literal(_) | Self::Typed(_) => Ok(()),
            Self::Dynamic { source } => validate_source(source),
        }
    }
}
