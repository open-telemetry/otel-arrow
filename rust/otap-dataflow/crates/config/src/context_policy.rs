// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Scoped composite entry definitions. References are resolved by the context compiler.

use crate::context::{ContextEntryName, ContextEntryRef};
use crate::{PipelineGroupId, PipelineId};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize, de};
use std::collections::BTreeMap;
use std::fmt;

/// User-defined entries augment the primitive entries declared by their source domains.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextPolicy {
    /// Named, conditional groups of primitive fields.
    #[serde(default, deserialize_with = "deserialize_context_entries")]
    pub entries: BTreeMap<ContextEntryName, ContextEntryDefinition>,
}

/// Deserializes context-keyed maps without silently overwriting normalized duplicate names.
pub fn deserialize_context_entries<'de, D, V, M>(deserializer: D) -> Result<M, D::Error>
where
    D: Deserializer<'de>,
    V: Deserialize<'de>,
    M: FromIterator<(ContextEntryName, V)>,
{
    struct EntriesVisitor<V>(std::marker::PhantomData<V>);

    impl<'de, V: Deserialize<'de>> de::Visitor<'de> for EntriesVisitor<V> {
        type Value = BTreeMap<ContextEntryName, V>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map of distinct normalized context entry names")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: de::MapAccess<'de>,
        {
            let mut entries = BTreeMap::new();
            while let Some((name, definition)) = map.next_entry::<ContextEntryName, V>()? {
                if name.contains(':') {
                    return Err(de::Error::custom(format!(
                        "context entry name `{name}` must not contain the reference separator `:`"
                    )));
                }
                if entries.insert(name.clone(), definition).is_some() {
                    return Err(de::Error::custom(format!(
                        "duplicate normalized context entry name `{name}`"
                    )));
                }
            }
            Ok(entries)
        }
    }

    deserializer
        .deserialize_map(EntriesVisitor(std::marker::PhantomData))
        .map(|entries| entries.into_iter().collect())
}

/// An ordered product of fields, guarded by zero or more conjunctions.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct ContextEntryDefinition(pub Vec<ContextEntryPart>);

/// A member contributes a dimension; a match contributes only a presence condition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContextEntryPart {
    /// Includes every value of an explicitly referenced primitive transport field.
    TransportHeader {
        /// A standalone field or a qualified member of a capture group.
        name: ContextEntryRef,
        /// Optional member name within the new composite.
        #[serde(rename = "as", default, skip_serializing_if = "Option::is_none")]
        alias: Option<ContextEntryName>,
    },
    /// Requires a text field to satisfy an explicit multi-value matching rule.
    TransportHeaderMatch {
        /// A standalone field or a qualified member of a capture group.
        name: ContextEntryRef,
        /// Exact, case-sensitive text value to compare.
        value: String,
        /// Explicitly specifies how repeated values participate in the match.
        #[serde(rename = "match")]
        multiplicity: ContextMatchMultiplicity,
    },
}

impl JsonSchema for ContextEntryPart {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "ContextEntryPart".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        // Kubernetes structural schemas require shared properties outside enum
        // branches. Serde still enforces each variant's exact field contract.
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "type": {"type": "string", "enum": ["transport_header", "transport_header_match"]},
                "name": generator.subschema_for::<ContextEntryRef>(),
                "as": generator.subschema_for::<ContextEntryName>(),
                "value": {"type": "string"},
                "match": generator.subschema_for::<ContextMatchMultiplicity>()
            },
            "required": ["type", "name"],
            "additionalProperties": false,
            "x-kubernetes-validations": [
                {
                    "rule": "self.type == 'transport_header' ? (!has(self.value) && !has(self.match)) : (has(self.value) && has(self.match))",
                    "message": "transport_header_match requires value and match; transport_header accepts neither"
                }
            ]
        })
    }
}

/// Quantification for an equality condition. Missing fields never match.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ContextMatchMultiplicity {
    /// At least one value equals the configured value.
    Any,
    /// Every value equals the configured value, with at least one value present.
    All,
}

/// Scope is part of entry identity, not a runtime name-search instruction.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextScope {
    /// Visible to every regular pipeline.
    Engine,
    /// Visible only to pipelines in this group.
    Group(PipelineGroupId),
    /// Visible only within this pipeline.
    Pipeline(PipelineGroupId, PipelineId),
}

impl ContextScope {
    /// Whether a node in the supplied pipeline can bind entries in this scope.
    #[must_use]
    pub fn visible_from(&self, group: &PipelineGroupId, pipeline: &PipelineId) -> bool {
        match self {
            Self::Engine => true,
            Self::Group(owner) => owner == group,
            Self::Pipeline(owner_group, owner_pipeline) => {
                owner_group == group && owner_pipeline == pipeline
            }
        }
    }
}

/// A resolved definition retains its declaring scope across policy inheritance.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextEntryDeclaration {
    /// Declaring scope.
    pub scope: ContextScope,
    /// Entry name, unique throughout its visibility chain.
    pub name: ContextEntryName,
    /// Complete construction specification.
    pub definition: ContextEntryDefinition,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: references select a whole entry or a qualified member.
    /// Guarantees: normalization is deterministic and malformed qualification is rejected.
    #[test]
    fn strict_context_references() {
        for (input, expected) in [("Standalone", "standalone"), ("Entry:FIELD", "entry:field")] {
            let reference = ContextEntryRef::try_from(input).expect("valid reference");
            assert_eq!(reference.to_string(), expected);
        }
        for input in [
            "",
            ":field",
            "entry:",
            "entry:field:other",
            "entry:two words",
        ] {
            assert!(ContextEntryRef::try_from(input).is_err(), "{input}");
        }
    }

    /// Scenario: entry definitions contain case-only duplicate names or reserved separators.
    /// Guarantees: deserialization rejects ambiguous identities rather than overwriting definitions.
    #[test]
    fn duplicate_and_qualified_entry_names_are_rejected() {
        for input in ["entries: {Tenant: [], tenant: []}", "entries: {'a:b': []}"] {
            assert!(serde_yaml::from_str::<ContextPolicy>(input).is_err());
        }
    }

    /// Scenario: a composite condition compares a potentially repeated header.
    /// Guarantees: the configuration must explicitly choose any-value or all-values matching.
    #[test]
    fn conditions_require_explicit_multiplicity() {
        let missing =
            "entries: {p: [{type: transport_header_match, name: env, value: production}]}";
        assert!(serde_yaml::from_str::<ContextPolicy>(missing).is_err());
        let explicit = "entries: {p: [{type: transport_header_match, name: env, value: production, match: all}]}";
        assert!(serde_yaml::from_str::<ContextPolicy>(explicit).is_ok());
    }
}
