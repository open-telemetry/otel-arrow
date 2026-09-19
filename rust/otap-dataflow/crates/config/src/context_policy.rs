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
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct ContextPolicy {
    /// Named grouping entries.
    #[serde(default, deserialize_with = "deserialize_context_entries")]
    pub entries: BTreeMap<ContextEntryName, ContextEntryDefinition>,
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

        if self.0.is_empty() {
            errors.push(format!("{path_prefix} must contain at least one member"));
            return errors;
        }

        for (index, part) in self.0.iter().enumerate() {
            let (entry, name) = part.entry_ref_and_name();

            if !output_names.insert(name) {
                errors.push(format!(
                    "{path_prefix}[{index}] produces duplicate member name `{name}`"
                ));
            }
            if !value_references.insert(entry) {
                errors.push(format!(
                    "{path_prefix}[{index}] repeats reference `{}`",
                    entry
                ));
            }
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
        /// Exact source context entry reference.
        entry: ContextEntryRef,
        /// Optional member name within the composite entry.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        r#as: Option<ContextEntryName>,
    },
    /// Includes values from a verified authorized-identity entry.
    AuthorizedIdentity {
        /// Exact source context entry reference.
        entry: ContextEntryRef,
        /// Optional member name within the composite entry.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        r#as: Option<ContextEntryName>,
    },
}

impl ContextEntryPart {
    fn entry_ref_and_name(&self) -> (&'_ ContextEntryRef, &'_ ContextEntryName) {
        let (entry, r#as) = match self {
            Self::TransportHeader { entry, r#as } => (entry, r#as.as_ref()),
            Self::AuthorizedIdentity { entry, r#as } => (entry, r#as.as_ref()),
        };
        let name = r#as.unwrap_or_else(|| entry.name());
        (entry, name)
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
                        "authorized_identity"
                    ]
                },
                "entry": generator.subschema_for::<ContextEntryRef>(),
                "as": generator.subschema_for::<ContextEntryName>()
            },
            "required": ["type", "entry"],
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

    /// Scenario: one grouping entry mixes authorized-identity and transport-header members.
    /// Guarantees: both supported variants, aliases, scoped names, and order are preserved.
    #[test]
    fn parses_grouping_entry_in_order() {
        let policy: ContextPolicy = serde_yaml::from_str(
            r#"
entries:
  product_user:
    - type: authorized_identity
      entry: customer
      as: customer_id
    - type: transport_header
      entry: captured:workspace
"#,
        )
        .expect("valid context policy");

        let name = ContextEntryName::try_from("product_user").expect("valid name");
        let parts = &policy.entries.get(&name).expect("entry is present").0;
        assert!(matches!(
            &parts[0],
            ContextEntryPart::AuthorizedIdentity {
                r#as: Some(alias), ..
            } if alias.as_str() == "customer_id"
        ));
        assert!(matches!(
            &parts[1],
            ContextEntryPart::TransportHeader { entry, .. }
                if entry.scope().map(ContextEntryName::as_str) == Some("captured")
                    && entry.name().as_str() == "workspace"
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
    - {type: transport_header, entry: first}
  tenant:
    - {type: transport_header, entry: second}
"#,
        )
        .expect_err("duplicate map key must fail");

        assert!(error.to_string().contains("duplicate context entry name"));
    }

    /// Scenario: a grouping entry has no members.
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
    /// Guarantees: grouping entries cannot expose ambiguous qualified member names.
    #[test]
    fn rejects_duplicate_output_member_names() {
        for yaml in [
            "entries: {tenant: [{type: transport_header, entry: first:id}, {type: authorized_identity, entry: second:id}]}",
            "entries: {tenant: [{type: transport_header, entry: first, as: id}, {type: authorized_identity, entry: second, as: id}]}",
        ] {
            let policy = serde_yaml::from_str::<ContextPolicy>(yaml).expect("valid syntax");
            assert!(!policy.validation_errors("context").is_empty(), "{yaml}");
        }
    }

    /// Scenario: a value source reference is repeated with a different member name.
    /// Guarantees: one source value cannot create redundant grouping dimensions.
    #[test]
    fn rejects_duplicate_value_references() {
        let yaml = "entries: {tenant: [{type: transport_header, entry: id, as: first}, {type: transport_header, entry: id, as: second}]}";
        let policy = serde_yaml::from_str::<ContextPolicy>(yaml).expect("valid syntax");

        assert!(!policy.validation_errors("context").is_empty());
    }

    /// Scenario: a part uses an unsupported variant or property.
    /// Guarantees: strict serde contracts reject unknown variants and fields.
    #[test]
    fn rejects_unsupported_variants_and_fields() {
        for yaml in [
            "entries: {tenant: [{type: transport_header, entry: id, value: prod}]}",
            "entries: {tenant: [{type: transport_header, entry: id, name: alias}]}",
            "entries: {tenant: [{type: unsupported, entry: id}]}",
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
                "entries: {'product_user:customer_id': [{type: transport_header, entry: id}]}"
            )
            .is_err()
        );
    }

    /// Scenario: schema is generated for context entry parts.
    /// Guarantees: both variants and the strict common field set appear deterministically.
    #[test]
    fn schema_exposes_supported_parts() {
        let schema = serde_json::to_value(schemars::schema_for!(ContextEntryPart))
            .expect("schema serializes");
        let rendered = schema.to_string();

        for variant in ["transport_header", "authorized_identity"] {
            assert!(rendered.contains(variant));
        }
        assert_eq!(schema["required"], serde_json::json!(["type", "entry"]));
        assert!(schema["properties"].get("entry").is_some());
        assert!(schema["properties"].get("ctx_ref").is_none());
        assert!(schema["properties"].get("as").is_some());
        assert!(schema["properties"].get("name").is_none());
        assert!(rendered.contains("additionalProperties"));
    }
}
