// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Shared component-inventory admin models.
//!
//! Wire types for the `GET /api/v1/components` admin endpoint, which reports the
//! running engine's component inventory (RFC 0001). The [`ComponentEntry`] shape
//! mirrors `components-baseline.json` (id, PascalCase category, description,
//! attributes map) so operators can diff endpoint output against the committed
//! baseline. Source `file`/`line` are intentionally omitted (they are not part
//! of the baseline and change frequently).

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Response for `GET /api/v1/components`: the engine's component inventory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ComponentsResponse {
    /// RFC 3339 timestamp when the inventory snapshot was generated.
    pub generated_at: String,
    /// Inventory entries, one per linked component.
    pub components: Vec<ComponentEntry>,
}

/// One security-relevant component in the engine's inventory.
///
/// Mirrors an entry in `components-baseline.json`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ComponentEntry {
    /// Component URN, e.g. `urn:otel:receiver:otlp`.
    pub id: String,
    /// Component category in PascalCase, e.g. `Receiver` (matches the baseline).
    pub category: String,
    /// Short human-readable description, if any.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Free-form key/value attributes (e.g. `listen_port`, `protocol`, `auth`).
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub attributes: BTreeMap<String, String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde::de::DeserializeOwned;
    use serde_json::{Value, json};

    fn assert_roundtrip<T>(value: Value)
    where
        T: DeserializeOwned + Serialize,
    {
        let parsed: T = serde_json::from_value(value.clone()).expect("fixture should deserialize");
        let serialized = serde_json::to_value(parsed).expect("model should serialize");
        assert_eq!(serialized, value);
    }

    /// Scenario: a full components response with a factory entry (attributes +
    /// description) and a non-factory entry (empty attributes, no description)
    /// is deserialized and re-serialized.
    /// Guarantees: the wire shape round-trips exactly, including `generatedAt`
    /// camelCase, PascalCase `category`, and omission of empty
    /// `attributes`/absent `description`.
    #[test]
    fn components_response_roundtrips_current_wire_shape() {
        assert_roundtrip::<ComponentsResponse>(json!({
            "generatedAt": "2026-01-01T00:00:00Z",
            "components": [
                {
                    "id": "urn:otel:receiver:otlp",
                    "category": "Receiver",
                    "description": "OTLP receiver",
                    "attributes": { "listen_port": "4317", "protocol": "gRPC" }
                },
                {
                    "id": "urn:otel:controller:main",
                    "category": "Controller"
                }
            ]
        }));
    }

    /// Scenario: an entry with no attributes and no description is serialized.
    /// Guarantees: empty `attributes` and absent `description` are omitted from
    /// the wire form (so the output matches the baseline's minimal entries).
    #[test]
    fn empty_attributes_and_description_are_omitted() {
        let entry = ComponentEntry {
            id: "urn:otel:safety:memory_limiter".to_string(),
            category: "Safety".to_string(),
            description: None,
            attributes: BTreeMap::new(),
        };
        let value = serde_json::to_value(&entry).expect("serialize");
        assert_eq!(
            value,
            json!({ "id": "urn:otel:safety:memory_limiter", "category": "Safety" })
        );
    }
}
