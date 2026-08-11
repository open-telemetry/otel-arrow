// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Component inventory endpoint (RFC 0001).
//!
//! - GET `/api/v1/components` - list the components linked into this engine
//!   binary, from the `COMPONENT_INVENTORY` link-time slice.
//!
//! The admin server runs inside the `df_engine` process, so this reports the
//! *running engine's* inventory (exactly what was compiled/linked, including
//! feature- and platform-gated components), not a source-level scan.

use crate::AppState;
use axum::routing::get;
use axum::{Json, Router};
use chrono::Utc;
use otap_df_admin_types::components::{ComponentEntry, ComponentsResponse};
use otap_df_engine::inventory::{self, ComponentMeta};

/// Routes for the component inventory endpoint.
pub(crate) fn routes() -> Router<AppState> {
    Router::new().route("/components", get(list_components))
}

/// GET `/api/v1/components`: the engine's component inventory.
pub async fn list_components() -> Json<ComponentsResponse> {
    let components = inventory::components().iter().map(to_entry).collect();
    Json(ComponentsResponse {
        generated_at: Utc::now().to_rfc3339(),
        components,
    })
}

/// Convert a link-time [`ComponentMeta`] into the owned wire entry.
///
/// Uses `Category::ident_str()` for the PascalCase category (matching the
/// `components-baseline.json` shape) and drops the source `file`/`line`.
fn to_entry(meta: &ComponentMeta) -> ComponentEntry {
    ComponentEntry {
        id: meta.id.to_string(),
        category: meta.category.ident_str().to_string(),
        description: meta.description.map(str::to_string),
        attributes: meta
            .attributes
            .iter()
            .map(|(k, v)| ((*k).to_string(), (*v).to_string()))
            .collect(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    /// Scenario: the live engine inventory is serialized through the endpoint's
    /// conversion for every linked component.
    /// Guarantees: every entry carries a non-empty `id` and a category string
    /// that is one of the known PascalCase category identifiers (so the wire
    /// output stays consistent with the baseline vocabulary).
    #[test]
    fn every_linked_component_converts_to_a_valid_entry() {
        const KNOWN: &[&str] = &[
            "Receiver",
            "Exporter",
            "Processor",
            "Extension",
            "Admin",
            "Controller",
            "Cli",
            "Subsystem",
            "Safety",
        ];
        for meta in inventory::components() {
            let entry = to_entry(meta);
            assert!(!entry.id.is_empty(), "component id must not be empty");
            assert!(
                KNOWN.contains(&entry.category.as_str()),
                "unexpected category `{}` for {}",
                entry.category,
                entry.id
            );
        }
    }

    /// Scenario: a `ComponentMeta` with attributes and a description is converted.
    /// Guarantees: `id`, PascalCase `category`, `description`, and the full
    /// attribute map are carried into the owned `ComponentEntry` unchanged.
    #[test]
    fn to_entry_preserves_fields_and_attributes() {
        use otap_df_engine::inventory::Category;
        let meta = ComponentMeta {
            id: "urn:otel:receiver:otlp",
            category: Category::Receiver,
            description: Some("OTLP receiver"),
            file: "file.rs",
            line: 1,
            attributes: &[("listen_port", "4317"), ("protocol", "gRPC")],
        };
        let entry = to_entry(&meta);
        assert_eq!(entry.id, "urn:otel:receiver:otlp");
        assert_eq!(entry.category, "Receiver");
        assert_eq!(entry.description.as_deref(), Some("OTLP receiver"));
        let expected: BTreeMap<String, String> = [
            ("listen_port".to_string(), "4317".to_string()),
            ("protocol".to_string(), "gRPC".to_string()),
        ]
        .into_iter()
        .collect();
        assert_eq!(entry.attributes, expected);
    }
}
