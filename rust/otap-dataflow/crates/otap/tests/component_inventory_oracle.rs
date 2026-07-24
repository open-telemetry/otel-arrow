// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Reliability oracle for the component inventory (RFC 0001).
//!
//! # What this guards
//!
//! The `cargo xtask component-inventory` scanner builds the inventory baseline
//! by parsing source with `syn`. That gives **completeness** (it sees every
//! component regardless of `#[cfg]`/feature/target gating) but it must resolve
//! factory URNs itself (following `name: SOME_URN_CONST` to the const's value,
//! including cross-crate `use` re-exports). This test is the **reliability**
//! half: it links a large set of real components (`core-nodes` + `contrib-nodes`
//! via the `otap` dev-dependency graph) and reads the **compiler-resolved**
//! `COMPONENT_INVENTORY` distributed slice, then asserts that every linked
//! component matches the committed `components-baseline.json`.
//!
//! Because the compiler has already resolved every `name:` URN const, this
//! catches any scanner URN-resolution error (e.g. a cross-crate `use`d const
//! that resolved to the wrong value, or a stale `urn:derived:unknown:*`) for
//! the subset of components that are actually linked here.
//!
//! # Why "linked subset", not "all components"
//!
//! No single build links every component: `contrib-nodes` are behind Cargo
//! features (`aws`, `azure`, `etw-receiver`, ...), some receivers are
//! platform-gated, and `wasm-host` is a separate crate. Completeness is owned
//! by the source scanner; this oracle validates correctness for whatever is
//! linked into this test binary. It therefore only asserts the "forward"
//! direction (linked => present-and-consistent in baseline) and never fails
//! for baseline entries that simply were not linked here.

use std::collections::BTreeMap;
use std::path::PathBuf;

use otap_df_engine::inventory::components;

// Pull the component-bearing crates into this test binary so their
// `#[component_inventory]` link-time entries are present in COMPONENT_INVENTORY.
// `use ... as _` keeps the dependency linked without importing any names.
use otap_df_contrib_nodes as _;
use otap_df_core_nodes as _;
use otap_df_otap as _;

/// Minimal view of a `components-baseline.json` entry (id + category are what
/// the oracle cross-checks; description/attributes are validated by the
/// scanner's own `--check`).
#[derive(serde::Deserialize)]
struct BaselineEntry {
    id: String,
    category: String,
}

/// Load `components-baseline.json` from the `otap-dataflow` workspace root.
///
/// `CARGO_MANIFEST_DIR` for this crate is `.../crates/otap`; the baseline lives
/// two levels up at the workspace root.
fn load_baseline() -> Vec<BaselineEntry> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let baseline_path = manifest_dir
        .join("..")
        .join("..")
        .join("components-baseline.json");
    let content = std::fs::read_to_string(&baseline_path).unwrap_or_else(|e| {
        panic!(
            "failed to read baseline at {}: {e}",
            baseline_path.display()
        )
    });
    serde_json::from_str(&content).expect("baseline is valid JSON")
}

/// Scenario: components linked into this test binary are read from the
/// compiler-resolved `COMPONENT_INVENTORY` slice and compared to the committed
/// baseline.
/// Guarantees: every linked component's compiler-resolved `id` exists in the
/// baseline with a matching `category`. This proves the scanner's URN
/// resolution (including cross-crate `use`d URN consts) agrees with what the
/// compiler actually linked -- the baseline cannot silently disagree with
/// reality for any linked component.
#[test]
fn linked_components_match_baseline() {
    let baseline = load_baseline();
    let baseline_by_id: BTreeMap<&str, &str> = baseline
        .iter()
        .map(|e| (e.id.as_str(), e.category.as_str()))
        .collect();

    let linked = components();
    assert!(
        !linked.is_empty(),
        "COMPONENT_INVENTORY is empty; expected the linked node crates to register components"
    );

    let mut mismatches = Vec::new();
    for meta in linked {
        // A compiler-resolved id must never look like a scanner "unresolved"
        // marker -- if it does, the macro/engine emitted something wrong.
        assert!(
            !meta.id.starts_with("urn:UNRESOLVED:") && !meta.id.starts_with("urn:derived:unknown:"),
            "linked component has an unresolved-looking id: {}",
            meta.id
        );

        match baseline_by_id.get(meta.id) {
            None => mismatches.push(format!(
                "linked component `{}` ({}) is NOT in components-baseline.json \
                 (run `cargo xtask component-inventory --update-baseline`)",
                meta.id, meta.category
            )),
            Some(&base_cat) => {
                let linked_cat = meta.category.to_string();
                // Baseline stores the Category enum name (e.g. "Receiver");
                // the linked Category `Display`s as its URN segment
                // (e.g. "receiver"). Compare case-insensitively so the two
                // representations line up without coupling to either spelling.
                if !base_cat.eq_ignore_ascii_case(&linked_cat) {
                    mismatches.push(format!(
                        "category mismatch for `{}`: linked `{}`, baseline `{}`",
                        meta.id, linked_cat, base_cat
                    ));
                }
            }
        }
    }

    assert!(
        mismatches.is_empty(),
        "component inventory oracle found {} mismatch(es):\n  - {}",
        mismatches.len(),
        mismatches.join("\n  - ")
    );
}
