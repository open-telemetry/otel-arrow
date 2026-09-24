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

/// User-defined composite context entries.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextPolicy {
    /// Named entries.
    #[serde(default, deserialize_with = "deserialize_context_entries")]
    pub entries: BTreeMap<ContextEntryName, ContextEntryDefinition>,
}

/// This ensures the names in the configuration are distinct.
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

/// Composite entry definition.
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
        let mut conditions = BTreeSet::new();
        let mut value_member_count = 0usize;

        if self.0.is_empty() {
            errors.push(format!("{path_prefix} must contain at least one member"));
            return errors;
        }

        for (index, part) in self.0.iter().enumerate() {
            match part.value_kind_ref_and_name() {
                Some((kind, entry, name)) => {
                    value_member_count += 1;
                    if !output_names.insert(name) {
                        errors.push(format!(
                            "{path_prefix}[{index}] produces duplicate member name `{name}`"
                        ));
                    }
                    if !value_references.insert((kind, entry)) {
                        errors.push(format!(
                            "{path_prefix}[{index}] repeats reference `{}`",
                            entry
                        ));
                    }
                }
                None => {
                    let ContextEntryPart::TransportHeaderMatch { name, value } = part else {
                        unreachable!("all value-bearing variants were handled")
                    };
                    if !conditions.insert((name, value)) {
                        errors.push(format!(
                            "{path_prefix}[{index}] repeats transport-header condition `{name}` = `{value}`"
                        ));
                    }
                }
            }
        }
        if value_member_count == 0 {
            errors.push(format!(
                "{path_prefix} must contain at least one value-bearing member"
            ));
        }

        errors
    }
}

/// Single member of a composite entry.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
pub enum ContextEntryPart {
    /// Includes values from a transport-header entry.
    TransportHeader {
        /// Exact source context entry reference.
        name: ContextEntryRef,
        /// Optional member name within the composite entry.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        store_as: Option<ContextEntryName>,
    },
    /// Includes values from a verified authorized-identity entry.
    AuthorizedIdentity {
        /// Exact source context entry reference.
        name: ContextEntryRef,
        /// Optional member name within the composite entry.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        store_as: Option<ContextEntryName>,
    },
    /// Requires a transport-header entry to contain an exact configured value.
    ///
    /// Header names match using ASCII case-insensitive transport semantics.
    /// Values compare as exact UTF-8 bytes, and any matching duplicate value
    /// satisfies this condition.
    TransportHeaderMatch {
        /// Exact source context entry reference.
        name: ContextEntryRef,
        /// Required text value.
        value: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum ContextEntryPartKind {
    TransportHeader,
    AuthorizedIdentity,
}

impl ContextEntryPart {
    fn value_kind_ref_and_name(
        &self,
    ) -> Option<(
        ContextEntryPartKind,
        &'_ ContextEntryRef,
        &'_ ContextEntryName,
    )> {
        let (kind, name, store_as) = match self {
            Self::TransportHeader { name, store_as } => (
                ContextEntryPartKind::TransportHeader,
                name,
                store_as.as_ref(),
            ),
            Self::AuthorizedIdentity { name, store_as } => (
                ContextEntryPartKind::AuthorizedIdentity,
                name,
                store_as.as_ref(),
            ),
            Self::TransportHeaderMatch { .. } => return None,
        };
        let output_name = store_as.unwrap_or_else(|| name.name());
        Some((kind, name, output_name))
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
                "name": generator.subschema_for::<ContextEntryRef>(),
                "store_as": generator.subschema_for::<ContextEntryName>(),
                "value": {
                    "type": "string"
                }
            },
            "required": ["type", "name"],
            "additionalProperties": false
        })
    }
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

    /// Scenario: one composite entry mixes authorized-identity and transport-header members.
    /// Guarantees: both supported variants, aliases, scoped names, and order are preserved.
    #[test]
    fn parses_composite_entry_in_order() {
        let policy: ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  product_user:
    - type: authorized_identity
      name: customer
      store_as: customer_id
    - type: transport_header
      name: captured:workspace
"#,
        )
        .expect("valid context policy");

        let name = ContextEntryName::try_from("product_user").expect("valid name");
        let parts = &policy.entries.get(&name).expect("entry is present").0;
        assert!(matches!(
            &parts[0],
            ContextEntryPart::AuthorizedIdentity {
                store_as: Some(alias),
                ..
            } if alias.as_str() == "customer_id"
        ));
        assert!(matches!(
            &parts[1],
            ContextEntryPart::TransportHeader { name, .. }
                if name.scope().map(ContextEntryName::as_str) == Some("captured")
                    && name.name().as_str() == "workspace"
        ));
    }

    /// Scenario: a composite entry includes an exact transport-header condition.
    /// Guarantees: the condition name and value are retained without becoming a value member.
    #[test]
    fn parses_transport_header_match_condition() {
        let policy: ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  product_user:
    - type: transport_header
      name: workspace
    - type: transport_header_match
      name: environment
      value: production
"#,
        )
        .expect("valid context policy");

        let name = ContextEntryName::try_from("product_user").expect("valid name");
        let parts = &policy.entries.get(&name).expect("entry is present").0;
        assert!(matches!(
            &parts[1],
            ContextEntryPart::TransportHeaderMatch { name, value }
                if name.name().as_str() == "environment" && value == "production"
        ));
        assert!(policy.validation_errors("context").is_empty());
    }

    /// Scenario: a context entry map repeats the exact same YAML key.
    /// Guarantees: deserialization rejects the duplicate instead of retaining the last definition.
    #[test]
    fn rejects_duplicate_entry_map_keys() {
        let error = serde_yaml::from_str::<ContextPolicy>(
            r#"
entries:
  tenant:
    - {type: transport_header, name: first}
  tenant:
    - {type: transport_header, name: second}
"#,
        )
        .expect_err("duplicate map key must fail");

        assert!(error.to_string().contains("duplicate context entry name"));
    }

    /// Scenario: a composite entry has no members.
    /// Guarantees: semantic validation rejects the empty declaration.
    #[test]
    fn rejects_empty_definitions() {
        let policy =
            serde_yaml::from_str::<ContextPolicy>("entries: {tenant: []}").expect("valid syntax");

        assert_eq!(
            policy.validation_errors("context"),
            ["context.entries.tenant must contain at least one member"]
        );
    }

    /// Scenario: two value members derive or specify the same output name.
    /// Guarantees: composite entries cannot expose ambiguous qualified member names.
    #[test]
    fn rejects_duplicate_output_member_names() {
        for yaml in [
            "entries: {tenant: [{type: transport_header, name: first:id}, {type: authorized_identity, name: second:id}]}",
            "entries: {tenant: [{type: transport_header, name: first, store_as: id}, {type: authorized_identity, name: second, store_as: id}]}",
        ] {
            let policy = serde_yaml::from_str::<ContextPolicy>(yaml).expect("valid syntax");
            assert!(!policy.validation_errors("context").is_empty(), "{yaml}");
        }
    }

    /// Scenario: a value source reference is repeated with a different member name.
    /// Guarantees: one source value cannot create redundant composite dimensions.
    #[test]
    fn rejects_duplicate_value_references() {
        let yaml = "entries: {tenant: [{type: transport_header, name: id, store_as: first}, {type: transport_header, name: id, store_as: second}]}";
        let policy = serde_yaml::from_str::<ContextPolicy>(yaml).expect("valid syntax");

        assert!(!policy.validation_errors("context").is_empty());
    }

    /// Scenario: a composite repeats an identical transport-header condition.
    /// Guarantees: redundant conditions are rejected while distinct values remain expressible.
    #[test]
    fn rejects_duplicate_transport_header_conditions() {
        let yaml = "entries: {tenant: [{type: transport_header, name: id}, {type: transport_header_match, name: environment, value: prod}, {type: transport_header_match, name: environment, value: prod}]}";
        let policy = serde_yaml::from_str::<ContextPolicy>(yaml).expect("valid syntax");

        assert!(!policy.validation_errors("context").is_empty());
    }

    /// Scenario: a composite contains conditions but exposes no value-bearing member.
    /// Guarantees: conditions cannot define a composite without a selectable field.
    #[test]
    fn rejects_condition_only_definitions() {
        let yaml =
            "entries: {tenant: [{type: transport_header_match, name: environment, value: prod}]}";
        let policy = serde_yaml::from_str::<ContextPolicy>(yaml).expect("valid syntax");

        assert_eq!(
            policy.validation_errors("context"),
            ["context.entries.tenant must contain at least one value-bearing member"]
        );
    }

    /// Scenario: different source types use the same entry name and one member is aliased.
    /// Guarantees: source type distinguishes references while stored member names stay unique.
    #[test]
    fn accepts_same_name_from_different_source_types_with_alias() {
        let yaml = "entries: {tenant: [{type: transport_header, name: id}, {type: authorized_identity, name: id, store_as: identity_id}]}";
        let policy = serde_yaml::from_str::<ContextPolicy>(yaml).expect("valid syntax");

        assert!(policy.validation_errors("context").is_empty());
    }

    /// Scenario: a part uses an unsupported variant or property.
    /// Guarantees: strict serde contracts reject unknown variants and fields.
    #[test]
    fn rejects_unsupported_variants_and_fields() {
        for yaml in [
            "entries: {tenant: [{type: transport_header, name: id, value: prod}]}",
            "entries: {tenant: [{type: transport_header, name: id, alias: other}]}",
            "entries: {tenant: [{type: transport_header_match, name: id, store_as: other, value: prod}]}",
            "entries: {tenant: [{type: unsupported, name: id}]}",
            "entries: {tenant: [{type: transport_header, ctx_ref: id}]}",
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
                "entries: {'product_user:customer_id': [{type: transport_header, name: id}]}"
            )
            .is_err()
        );
    }

    /// Scenario: schema is generated for context entry parts.
    /// Guarantees: all variants and the strict common field set appear deterministically.
    #[test]
    fn schema_exposes_supported_parts() {
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
        assert_eq!(schema["required"], serde_json::json!(["type", "name"]));
        assert!(schema["properties"].get("name").is_some());
        assert!(schema["properties"].get("ctx_ref").is_none());
        assert!(schema["properties"].get("store_as").is_some());
        assert!(schema["properties"].get("value").is_some());
        assert!(rendered.contains("additionalProperties"));
    }
}
