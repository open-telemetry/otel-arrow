// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Declarative context entry policies.

use crate::context::{ContextEntryName, ContextEntryRef};
use crate::{PipelineGroupId, PipelineId};
use schemars::JsonSchema;
use serde::de::{self, MapAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

/// User-defined grouping context entries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextPolicy {
    /// Named grouping entries.
    #[serde(default)]
    pub entries: BTreeMap<ContextEntryName, ContextEntryDefinition>,
}

impl<'de> Deserialize<'de> for ContextPolicy {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        #[serde(deny_unknown_fields)]
        struct RawContextPolicy {
            #[serde(default, deserialize_with = "deserialize_context_entries")]
            entries: BTreeMap<ContextEntryName, ContextEntryDefinition>,
        }

        let policy = Self {
            entries: RawContextPolicy::deserialize(deserializer)?.entries,
        };
        if let Some(error) = policy.validation_errors("context").into_iter().next() {
            return Err(de::Error::custom(error));
        }
        Ok(policy)
    }
}

fn deserialize_context_entries<'de, D>(
    deserializer: D,
) -> Result<BTreeMap<ContextEntryName, ContextEntryDefinition>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ContextEntriesVisitor;

    impl<'de> Visitor<'de> for ContextEntriesVisitor {
        type Value = BTreeMap<ContextEntryName, ContextEntryDefinition>;

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map of distinct context entry names")
        }

        fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
        where
            A: MapAccess<'de>,
        {
            let mut entries = BTreeMap::new();
            while let Some((name, definition)) =
                map.next_entry::<ContextEntryName, ContextEntryDefinition>()?
            {
                if entries.insert(name.clone(), definition).is_some() {
                    return Err(de::Error::custom(format!(
                        "duplicate context entry name `{name}`"
                    )));
                }
            }
            Ok(entries)
        }
    }

    deserializer.deserialize_map(ContextEntriesVisitor)
}

impl ContextPolicy {
    /// Returns configuration-local declaration errors.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        self.entries
            .iter()
            .flat_map(|(name, definition)| {
                definition.validation_errors(&format!("{path_prefix}.entries.{name}"))
            })
            .collect()
    }
}

/// An ordered grouping entry definition.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(transparent)]
pub struct ContextEntryDefinition(pub Vec<ContextEntryPart>);

impl ContextEntryDefinition {
    /// Returns configuration-local definition errors.
    #[must_use]
    pub fn validation_errors(&self, path_prefix: &str) -> Vec<String> {
        let mut errors = Vec::new();
        let mut output_names = BTreeSet::new();
        let mut value_references = BTreeSet::new();
        let mut value_members = 0;

        if self.0.is_empty() {
            errors.push(format!("{path_prefix} must contain at least one member"));
            return errors;
        }

        for (index, part) in self.0.iter().enumerate() {
            let Some(member) = part.as_value_member() else {
                continue;
            };
            value_members += 1;

            let output_name = member
                .name
                .or_else(|| member.ctx_ref.field())
                .unwrap_or_else(|| member.ctx_ref.entry());
            if !output_names.insert(output_name) {
                errors.push(format!(
                    "{path_prefix}[{index}] produces duplicate member name `{output_name}`"
                ));
            }
            if !value_references.insert((member.source_kind, member.ctx_ref)) {
                errors.push(format!(
                    "{path_prefix}[{index}] repeats {} reference `{}`",
                    member.source_kind, member.ctx_ref
                ));
            }
        }

        if value_members == 0 {
            errors.push(format!(
                "{path_prefix} must contain at least one value-bearing member"
            ));
        }
        errors
    }
}

/// One value-bearing member or conjunctive presence condition.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContextEntryPart {
    /// Includes values from a transport-header entry.
    TransportHeader {
        /// Exact source context entry or qualified member reference.
        ctx_ref: ContextEntryRef,
        /// Optional member name within the grouping entry.
        ///
        /// When omitted, the name is derived from the referenced member or entry.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<ContextEntryName>,
    },
    /// Includes values from a verified authorized-identity entry.
    AuthorizedIdentity {
        /// Exact source context entry or qualified member reference.
        ctx_ref: ContextEntryRef,
        /// Optional member name within the grouping entry.
        ///
        /// When omitted, the name is derived from the referenced member or entry.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        name: Option<ContextEntryName>,
    },
    /// Requires a transport-header entry to satisfy an explicit repeated-value match.
    TransportHeaderMatch {
        /// Exact source context entry or qualified member reference.
        ctx_ref: ContextEntryRef,
        /// Exact, case-sensitive value to compare.
        value: String,
        /// Required repeated-value quantifier.
        #[serde(rename = "match")]
        multiplicity: ContextMatchMultiplicity,
    },
}

struct ValueMemberRef<'a> {
    source_kind: &'static str,
    ctx_ref: &'a ContextEntryRef,
    name: Option<&'a ContextEntryName>,
}

impl ContextEntryPart {
    fn as_value_member(&self) -> Option<ValueMemberRef<'_>> {
        match self {
            Self::TransportHeader { ctx_ref, name } => Some(ValueMemberRef {
                source_kind: "transport_header",
                ctx_ref,
                name: name.as_ref(),
            }),
            Self::AuthorizedIdentity { ctx_ref, name } => Some(ValueMemberRef {
                source_kind: "authorized_identity",
                ctx_ref,
                name: name.as_ref(),
            }),
            Self::TransportHeaderMatch { .. } => None,
        }
    }
}

// Most config types derive JsonSchema. This enum is manual only because the
// config crate's test-only kube CRD generation requires a structural schema:
// kube rejects the derived internally tagged enum when each variant gives the
// shared `type` property a different singleton value. Keep this schema aligned
// with serde and the CRD compatibility tests in engine.rs.
impl JsonSchema for ContextEntryPart {
    fn schema_name() -> std::borrow::Cow<'static, str> {
        "ContextEntryPart".into()
    }

    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        schemars::json_schema!({
            "type": "object",
            "properties": {
                "type": {
                    "type": "string",
                    "enum": [
                        "transport_header",
                        "authorized_identity",
                        "transport_header_match"
                    ]
                },
                "ctx_ref": generator.subschema_for::<ContextEntryRef>(),
                "name": generator.subschema_for::<ContextEntryName>(),
                "value": {"type": "string"},
                "match": generator.subschema_for::<ContextMatchMultiplicity>()
            },
            "required": ["type", "ctx_ref"],
            "additionalProperties": false,
            "x-kubernetes-validations": [
                {
                    "rule": "self.type == 'transport_header_match' ? (has(self.value) && has(self.match) && !has(self.name)) : (!has(self.value) && !has(self.match))",
                    "message": "transport_header_match requires value and match and does not accept name; value-bearing members accept only optional name"
                }
            ]
        })
    }
}

/// Quantification for equality conditions over repeated transport values.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ContextMatchMultiplicity {
    /// At least one present value equals the configured value.
    Any,
    /// At least one value is present and every present value equals the configured value.
    All,
}

/// Scope that declares a context entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum ContextScope {
    /// Visible to every regular pipeline.
    Engine,
    /// Visible only to pipelines in this group.
    Group(PipelineGroupId),
    /// Visible only within this pipeline.
    Pipeline(PipelineGroupId, PipelineId),
}

/// A resolved entry definition with its declaring scope.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ContextEntryDeclaration {
    /// Declaring scope.
    pub scope: ContextScope,
    /// Entry name, unique within the pipeline's visibility chain.
    pub name: ContextEntryName,
    /// Complete ordered definition.
    pub definition: ContextEntryDefinition,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Scenario: one grouping entry mixes authorized and transport members plus two conditions.
    /// Guarantees: every supported variant parses and declaration order is preserved.
    #[test]
    fn parses_mixed_grouping_entry_in_order() {
        let policy: ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  product_user:
    - type: authorized_identity
      ctx_ref: customer
      name: customer_id
    - type: transport_header
      ctx_ref: captured:workspace
    - type: transport_header_match
      ctx_ref: environment
      value: production
      match: any
    - type: transport_header_match
      ctx_ref: region
      value: us-east
      match: all
"#,
        )
        .expect("valid context policy");

        let name = ContextEntryName::try_from("product_user").expect("valid name");
        let parts = &policy.entries.get(&name).expect("entry is present").0;
        assert!(matches!(
            &parts[0],
            ContextEntryPart::AuthorizedIdentity {
                name: Some(name), ..
            } if name.as_str() == "customer_id"
        ));
        assert!(matches!(
            &parts[1],
            ContextEntryPart::TransportHeader { ctx_ref, .. }
                if ctx_ref.to_string() == "captured:workspace"
        ));
        assert!(matches!(
            &parts[2],
            ContextEntryPart::TransportHeaderMatch {
                multiplicity: ContextMatchMultiplicity::Any,
                ..
            }
        ));
        assert!(matches!(
            &parts[3],
            ContextEntryPart::TransportHeaderMatch {
                multiplicity: ContextMatchMultiplicity::All,
                ..
            }
        ));
    }

    /// Scenario: a context entry map repeats the exact same YAML key.
    /// Guarantees: deserialization rejects the duplicate instead of retaining the last definition.
    #[test]
    fn rejects_duplicate_entry_map_keys() {
        let error = serde_yaml::from_str::<ContextPolicy>(
            r#"
entries:
  tenant:
    - {type: transport_header, ctx_ref: first}
  tenant:
    - {type: transport_header, ctx_ref: second}
"#,
        )
        .expect_err("duplicate map key must fail");

        assert!(error.to_string().contains("duplicate context entry name"));
    }

    /// Scenario: a grouping entry is empty or contains only a match condition.
    /// Guarantees: every declaration has at least one value-bearing member.
    #[test]
    fn rejects_definitions_without_value_members() {
        for yaml in [
            "entries: {tenant: []}",
            "entries: {tenant: [{type: transport_header_match, ctx_ref: env, value: prod, match: any}]}",
        ] {
            assert!(
                serde_yaml::from_str::<ContextPolicy>(yaml).is_err(),
                "{yaml}"
            );
        }
    }

    /// Scenario: two value members derive or specify the same output name.
    /// Guarantees: grouping entries cannot expose ambiguous qualified member names.
    #[test]
    fn rejects_duplicate_output_member_names() {
        for yaml in [
            "entries: {tenant: [{type: transport_header, ctx_ref: first:id}, {type: authorized_identity, ctx_ref: second:id}]}",
            "entries: {tenant: [{type: transport_header, ctx_ref: first, name: id}, {type: authorized_identity, ctx_ref: second, name: id}]}",
        ] {
            assert!(
                serde_yaml::from_str::<ContextPolicy>(yaml).is_err(),
                "{yaml}"
            );
        }
    }

    /// Scenario: a value source reference is repeated with a different member name.
    /// Guarantees: one source value cannot create redundant grouping dimensions.
    #[test]
    fn rejects_duplicate_value_references() {
        let yaml = "entries: {tenant: [{type: transport_header, ctx_ref: id, name: first}, {type: transport_header, ctx_ref: id, name: second}]}";

        assert!(serde_yaml::from_str::<ContextPolicy>(yaml).is_err());
    }

    /// Scenario: a match omits multiplicity or a part contains variant-inappropriate fields.
    /// Guarantees: strict serde contracts reject ambiguous conditions and unknown fields.
    #[test]
    fn rejects_missing_or_inapplicable_fields() {
        for yaml in [
            "entries: {tenant: [{type: transport_header_match, ctx_ref: env, value: prod}]}",
            "entries: {tenant: [{type: transport_header, ctx_ref: id, value: prod}]}",
            "entries: {tenant: [{type: transport_header_match, ctx_ref: env, value: prod, match: any, name: alias}]}",
            "entries: {tenant: [{type: unsupported, ctx_ref: id}]}",
        ] {
            assert!(
                serde_yaml::from_str::<ContextPolicy>(yaml).is_err(),
                "{yaml}"
            );
        }
    }

    /// Scenario: a top-level entry key uses qualification syntax.
    /// Guarantees: declaration names remain unqualified logical entry names.
    #[test]
    fn rejects_qualified_top_level_names() {
        assert!(
            serde_yaml::from_str::<ContextPolicy>(
                "entries: {'product_user:customer_id': [{type: transport_header, ctx_ref: id}]}"
            )
            .is_err()
        );
    }

    /// Scenario: schema is generated for context entry parts.
    /// Guarantees: all three variants and strict conditional fields appear deterministically.
    #[test]
    fn schema_exposes_supported_parts_and_condition_contract() {
        let schema = serde_json::to_value(schemars::schema_for!(ContextEntryPart))
            .expect("schema serializes");
        let rendered = schema.to_string();

        for variant in [
            "transport_header",
            "authorized_identity",
            "transport_header_match",
        ] {
            assert!(rendered.contains(variant));
        }
        assert_eq!(schema["required"], serde_json::json!(["type", "ctx_ref"]));
        assert!(schema["properties"].get("ctx_ref").is_some());
        assert!(schema["properties"].get("name").is_some());
        assert!(schema["properties"].get("as").is_none());
        assert!(rendered.contains("additionalProperties"));
        assert!(rendered.contains("x-kubernetes-validations"));
    }
}
