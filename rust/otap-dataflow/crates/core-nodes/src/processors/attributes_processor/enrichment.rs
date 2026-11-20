// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_config::context::policy::ContextDomain;
use otel_arrow_dfe_engine::pdata_context::{
    ContextEntrySelector, ContextEntrySelectorForm, ContextEntryTake, ContextEntryTarget,
    ContextNameMatching, ContextValueCardinality, ContextValueKind, Metadata, NodeBindings, Read,
    ReadMember, ValueRead,
};

use super::*;

/// A literal value, retaining scalar and typed-object syntax for existing configurations.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(transparent)]
pub struct AttributeLiteral(Value);

/// An optional explicit type for a literal attribute value.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LiteralType {
    /// A boolean attribute.
    Bool,
    /// A signed 64-bit integer attribute.
    Int,
    /// A floating-point attribute.
    Double,
    /// A string attribute.
    #[serde(rename = "string")]
    Str,
}

impl AttributeLiteral {
    pub(super) fn to_literal(
        &self,
        literal_type: Option<LiteralType>,
    ) -> Result<LiteralValue, ConfigError> {
        let (value, embedded_type) = match &self.0 {
            Value::Object(fields) => {
                let mut fields = fields.iter();
                let (name, value) = fields.next().ok_or_else(|| {
                    invalid_config("typed attribute literal must contain one type field")
                })?;
                if fields.next().is_some() {
                    return Err(invalid_config(
                        "typed attribute literal must contain exactly one type field",
                    ));
                }
                let embedded_type = match name.as_str() {
                    "bool" => LiteralType::Bool,
                    "int" => LiteralType::Int,
                    "double" => LiteralType::Double,
                    "string" => LiteralType::Str,
                    _ => return Err(invalid_config(format!("unknown literal type: {name}"))),
                };
                (value, Some(embedded_type))
            }
            value => (value, None),
        };
        if matches!((literal_type, embedded_type), (Some(explicit), Some(embedded)) if explicit != embedded)
        {
            return Err(invalid_config(
                "attribute type must agree with the typed literal",
            ));
        }
        let parsed = match literal_type.or(embedded_type) {
            Some(LiteralType::Bool) => {
                serde_json::from_value::<bool>(value.clone()).map(LiteralValue::Bool)
            }
            Some(LiteralType::Int) => {
                serde_json::from_value::<i64>(value.clone()).map(LiteralValue::Int)
            }
            Some(LiteralType::Double) => {
                serde_json::from_value::<f64>(value.clone()).map(LiteralValue::Double)
            }
            Some(LiteralType::Str) => {
                serde_json::from_value::<String>(value.clone()).map(LiteralValue::Str)
            }
            None => serde_json::from_value(value.clone()),
        };
        parsed.map_err(|error| invalid_config(format!("invalid attribute literal: {error}")))
    }
}

/// Configuration for selecting captured primitives or materialized context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AttributeSourceConfig {
    /// Omission selects transport headers; explicit null selects materialized composites.
    #[serde(default = "default_source_domain")]
    pub domain: Option<ContextDomain>,
    /// Ordered fallback candidates.
    pub named: Vec<otel_arrow_dfe_config::ContextEntryRef>,
}

fn default_source_domain() -> Option<ContextDomain> {
    Some(ContextDomain::TransportHeader)
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

pub(super) fn validate_source(source: &AttributeSourceConfig) -> Result<(), ConfigError> {
    if source.named.is_empty() {
        return Err(invalid_config(
            "attribute source named list must not be empty",
        ));
    }
    if source.domain.is_some() && source.named.iter().any(|name| name.scope().is_some()) {
        return Err(invalid_config(
            "primitive attribute sources do not accept qualified names",
        ));
    }
    Ok(())
}

pub(super) fn context_entries(
    source: &AttributeSourceConfig,
) -> impl Iterator<Item = ContextEntrySelector> + '_ {
    source.named.iter().map(|reference| {
        let target = match source.domain {
            Some(domain) => ContextEntryTarget::Primitive {
                domain,
                name: reference.name().clone(),
                matching: if domain == ContextDomain::TransportHeader {
                    ContextNameMatching::AsciiCaseInsensitive
                } else {
                    ContextNameMatching::Exact
                },
            },
            None => match reference.scope() {
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

/// Compiled reads for a metadata source.
pub(super) struct SourceReader {
    domain: Option<ContextDomain>,
    reads: Box<[ValueRead]>,
}

impl SourceReader {
    pub(super) fn bind(
        source: &AttributeSourceConfig,
        bindings: &NodeBindings,
    ) -> Result<Self, ConfigError> {
        Ok(Self {
            domain: source.domain,
            reads: context_entries(source)
                .map(|selector| bindings.value(&selector).cloned())
                .collect::<Result<_, _>>()?,
        })
    }

    pub(super) fn resolve(&self, metadata: &Metadata) -> Result<Option<LiteralValue>, EngineError> {
        for read in &self.reads {
            let Some(selected) = read
                .read(metadata)
                .map_err(|error| invalid_data(error.to_string()))?
            else {
                continue;
            };
            let text = match self.domain {
                Some(ContextDomain::TransportHeader) => first_usable_header(selected),
                Some(ContextDomain::AuthorizedIdentity) => scalar_text(selected, false)?,
                None => scalar_text(selected, true)?,
            };
            if let Some(text) = text {
                return Ok(Some(LiteralValue::Str(text)));
            }
        }
        Ok(None)
    }
}

fn first_usable_header(selected: Read<'_>) -> Option<String> {
    let mut found = None;
    selected.visit_members(|member| {
        if found.is_some() {
            return;
        }
        found = member
            .values()
            .find_map(|value| usable_text(value.kind, value.bytes).map(str::to_owned));
    });
    found
}

fn usable_text(kind: ContextValueKind, bytes: &[u8]) -> Option<&str> {
    if kind != ContextValueKind::Text {
        return None;
    }
    let text = std::str::from_utf8(bytes).ok()?;
    (!text.trim().is_empty()).then_some(text)
}

fn scalar_text(selected: Read<'_>, composite: bool) -> Result<Option<String>, EngineError> {
    let mut first: Option<ReadMember<'_>> = None;
    let mut members = 0usize;
    selected.visit_members(|member| {
        members += 1;
        if first.is_none() {
            first = Some(member);
        }
    });
    if composite && members > 1 {
        return Err(invalid_data(
            "named source requires entry:member for multi-member entries",
        ));
    }
    let Some(member) = first else {
        return Err(invalid_data("context source selected no members"));
    };
    if member.cardinality != ContextValueCardinality::One {
        return Err(invalid_data("context source requires a scalar value"));
    }
    let Some(value) = member.values().next() else {
        return Err(invalid_data("context source requires a scalar value"));
    };
    Ok(usable_text(value.kind, value.bytes).map(str::to_owned))
}
