// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_dfe_engine::pdata_context::{
    ContextDomain, ContextEntrySelector, ContextEntrySelectorForm, ContextEntryTake,
    ContextEntryTarget, ContextNameMatching, ContextValueCardinality, ContextValueKind, Metadata,
    NodeBindings, Read, ReadMember, ValueRead,
};
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;

use super::*;

/// An insert, upsert, or update value: a literal or a metadata source.
#[derive(Debug, Clone)]
pub enum ConfiguredValue {
    /// Scalar literal, retained for existing configurations.
    Literal(LiteralValue),
    /// Typed shorthand, retained for existing configurations.
    Typed(LiteralValue),
    /// Explicit literal with `type` and `value` fields.
    Explicit(LiteralValue),
    /// Dynamic request metadata source, configured under the `value` key.
    Dynamic {
        /// Ordered primitive names or composite references.
        source: AttributeSource,
    },
}

#[derive(Deserialize)]
#[serde(
    tag = "type",
    content = "value",
    rename_all = "lowercase",
    deny_unknown_fields
)]
enum ExplicitLiteral {
    Bool(bool),
    #[serde(rename = "int64")]
    Int(i64),
    Double(f64),
    #[serde(rename = "string")]
    Str(String),
}

impl From<ExplicitLiteral> for LiteralValue {
    fn from(value: ExplicitLiteral) -> Self {
        match value {
            ExplicitLiteral::Bool(value) => Self::Bool(value),
            ExplicitLiteral::Int(value) => Self::Int(value),
            ExplicitLiteral::Double(value) => Self::Double(value),
            ExplicitLiteral::Str(value) => Self::Str(value),
        }
    }
}

impl Serialize for ConfiguredValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        #[derive(Serialize)]
        struct LiteralFields<'a> {
            #[serde(rename = "type")]
            literal_type: &'static str,
            value: &'a LiteralValue,
        }

        match self {
            Self::Literal(value) => value.serialize(serializer),
            Self::Dynamic { source } => {
                let mut map = serializer.serialize_map(Some(1))?;
                map.serialize_entry("source", source)?;
                map.end()
            }
            Self::Typed(value) | Self::Explicit(value) => {
                let (shorthand, literal_type) = match value {
                    LiteralValue::Bool(_) => ("bool", "bool"),
                    LiteralValue::Int(_) => ("int", "int64"),
                    LiteralValue::Double(_) => ("double", "double"),
                    LiteralValue::Str(_) => ("string", "string"),
                };
                let mut map = serializer.serialize_map(Some(1))?;
                if matches!(self, Self::Typed(_)) {
                    map.serialize_entry(shorthand, value)?;
                } else {
                    map.serialize_entry(
                        "literal",
                        &LiteralFields {
                            literal_type,
                            value,
                        },
                    )?;
                }
                map.end()
            }
        }
    }
}

impl<'de> Deserialize<'de> for ConfiguredValue {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ValueVisitor;

        impl<'de> Visitor<'de> for ValueVisitor {
            type Value = ConfiguredValue;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                formatter.write_str("a scalar literal, typed literal, or metadata source")
            }

            fn visit_bool<E: serde::de::Error>(self, value: bool) -> Result<Self::Value, E> {
                Ok(ConfiguredValue::Literal(LiteralValue::Bool(value)))
            }

            fn visit_i64<E: serde::de::Error>(self, value: i64) -> Result<Self::Value, E> {
                Ok(ConfiguredValue::Literal(LiteralValue::Int(value)))
            }

            fn visit_u64<E: serde::de::Error>(self, value: u64) -> Result<Self::Value, E> {
                LiteralValue::deserialize(serde::de::value::U64Deserializer::<E>::new(value))
                    .map(ConfiguredValue::Literal)
            }

            fn visit_f64<E: serde::de::Error>(self, value: f64) -> Result<Self::Value, E> {
                Ok(ConfiguredValue::Literal(LiteralValue::Double(value)))
            }

            fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Self::Value, E> {
                self.visit_string(value.to_owned())
            }

            fn visit_string<E: serde::de::Error>(self, value: String) -> Result<Self::Value, E> {
                Ok(ConfiguredValue::Literal(LiteralValue::Str(value)))
            }

            fn visit_map<M: MapAccess<'de>>(self, mut map: M) -> Result<Self::Value, M::Error> {
                const FIELDS: &[&str] = &["literal", "source", "string", "bool", "int", "double"];
                let Some(field) = map.next_key::<String>()? else {
                    return Err(serde::de::Error::custom("attribute value cannot be empty"));
                };
                // Keep scalar and typed shorthand forms accepted for existing configurations.
                let value = match field.as_str() {
                    "literal" => {
                        ConfiguredValue::Explicit(map.next_value::<ExplicitLiteral>()?.into())
                    }
                    "source" => ConfiguredValue::Dynamic {
                        source: map.next_value()?,
                    },
                    "string" => ConfiguredValue::Typed(LiteralValue::Str(map.next_value()?)),
                    "bool" => ConfiguredValue::Typed(LiteralValue::Bool(map.next_value()?)),
                    "int" => ConfiguredValue::Typed(LiteralValue::Int(map.next_value()?)),
                    "double" => ConfiguredValue::Typed(LiteralValue::Double(map.next_value()?)),
                    _ => return Err(serde::de::Error::unknown_field(&field, FIELDS)),
                };
                if map.next_key::<String>()?.is_some() {
                    return Err(serde::de::Error::custom(
                        "attribute value must contain exactly one literal or source",
                    ));
                }
                Ok(value)
            }
        }

        deserializer.deserialize_any(ValueVisitor)
    }
}

/// Processor-local selection of captured primitives or materialized context.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(try_from = "SourceFields")]
pub struct AttributeSource {
    /// Preparing the changes to support to forth-coming ContextSelector pattern with domain; omission selects named composites or members.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain: Option<AttributeSourceDomain>,
    /// Ordered fallback candidates.
    pub named: Vec<otel_arrow_dfe_config::ContextEntryRef>,
}

/// Context domains supported by attribute value projection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AttributeSourceDomain {
    /// Captured transport-header values.
    TransportHeader,
    /// Captured verified identity claims.
    AuthorizedIdentity,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct SourceFields {
    #[serde(default, deserialize_with = "non_null")]
    domain: Option<AttributeSourceDomain>,
    // Keep source.type accepted for existing configurations.
    #[serde(default, rename = "type", deserialize_with = "non_null")]
    source_type: Option<LegacySourceType>,
    named: Vec<otel_arrow_dfe_config::ContextEntryRef>,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
enum LegacySourceType {
    TransportHeader,
    AuthorizedIdentity,
    Named,
}

impl TryFrom<SourceFields> for AttributeSource {
    type Error = String;

    fn try_from(fields: SourceFields) -> Result<Self, Self::Error> {
        let domain = match (fields.domain, fields.source_type) {
            (Some(_), Some(_)) => {
                return Err("source accepts either domain or type, not both".into());
            }
            (Some(domain), None) => Some(domain),
            (None, Some(LegacySourceType::TransportHeader)) => {
                Some(AttributeSourceDomain::TransportHeader)
            }
            (None, Some(LegacySourceType::AuthorizedIdentity)) => {
                Some(AttributeSourceDomain::AuthorizedIdentity)
            }
            (None, Some(LegacySourceType::Named) | None) => None,
        };
        Ok(Self {
            domain,
            named: fields.named,
        })
    }
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
    if source.domain.is_some() && source.named.iter().any(|name| name.scope().is_some()) {
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
        let target = match source.domain {
            Some(domain) => ContextEntryTarget::Primitive {
                domain: match domain {
                    AttributeSourceDomain::TransportHeader => ContextDomain::TransportHeader,
                    AttributeSourceDomain::AuthorizedIdentity => ContextDomain::AuthorizedIdentity,
                },
                name: reference.name().clone(),
                matching: if domain == AttributeSourceDomain::TransportHeader {
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
    domain: Option<AttributeSourceDomain>,
    reads: Box<[ValueRead]>,
}

impl SourceReader {
    pub(super) fn bind(
        source: &AttributeSource,
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
                Some(AttributeSourceDomain::TransportHeader) => first_usable_header(selected),
                Some(AttributeSourceDomain::AuthorizedIdentity) => {
                    Some(scalar_text(selected, false)?)
                }
                None => Some(scalar_text(selected, true)?),
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
        found = member.values().find_map(|value| {
            if value.kind != ContextValueKind::Text {
                return None;
            }
            let text = std::str::from_utf8(value.bytes).ok()?;
            (!text.trim().is_empty()).then(|| text.to_owned())
        });
    });
    found
}

fn scalar_text(selected: Read<'_>, composite: bool) -> Result<String, EngineError> {
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
    if value.kind != ContextValueKind::Text {
        return Err(invalid_data(
            "context source requires text for scalar attributes",
        ));
    }
    std::str::from_utf8(value.bytes)
        .map(str::to_owned)
        .map_err(|_| invalid_data("context source contains invalid UTF-8"))
}
impl ConfiguredValue {
    pub(super) fn validate(&self) -> Result<(), ConfigError> {
        match self {
            Self::Literal(_) | Self::Typed(_) | Self::Explicit(_) => Ok(()),
            Self::Dynamic { source } => validate_source(source),
        }
    }
}
