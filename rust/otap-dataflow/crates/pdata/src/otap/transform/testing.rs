// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test helpers specific to the transform submodules (reindex, split, etc.).

use std::collections::HashSet;

use arrow::array::{Array, AsArray, RecordBatch};
use arrow::datatypes::{
    ArrowDictionaryKeyType, ArrowNativeType, DataType, UInt8Type, UInt16Type, UInt32Type,
};

use crate::otap::OtapBatchStore;
use crate::otap::transform::util::{access_column, payload_relations, payload_to_idx};

/// For each batch group, relation, and child type, records which child row
/// indices map to each parent by ordinal position (smallest parent ID = 0,
/// second smallest = 1, etc).  This captures structural relationships
/// independent of actual ID values, so it should be identical before and
/// after any ID‑rewriting transform.
#[must_use]
pub(crate) fn extract_relation_fingerprints<S: OtapBatchStore, const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
) -> Vec<Vec<Vec<usize>>> {
    let mut fingerprints = Vec::new();

    for group in batches.iter() {
        for &payload_type in S::allowed_payload_types() {
            let parent_idx = payload_to_idx(payload_type);
            let Some(parent_batch) = &group[parent_idx] else {
                continue;
            };

            for relation in payload_relations(payload_type).relations {
                let Some(parent_col) = access_column(
                    relation.key_col,
                    &parent_batch.schema(),
                    parent_batch.columns(),
                ) else {
                    continue;
                };
                let parent_ids = collect_row_ids(parent_col.as_ref());

                // Sort and deduplicate to get ordinal mapping
                let mut unique_sorted: Vec<u32> = parent_ids.clone();
                unique_sorted.sort();
                unique_sorted.dedup();

                for &child_type in relation.child_types {
                    let child_idx = payload_to_idx(child_type);
                    let Some(child_batch) = &group[child_idx] else {
                        continue;
                    };

                    let Some(child_col) =
                        access_column("parent_id", &child_batch.schema(), child_batch.columns())
                    else {
                        continue;
                    };
                    let child_parent_ids = collect_row_ids(child_col.as_ref());

                    // For each parent ordinal, record which child row indices reference it
                    let mut ordinal_to_children: Vec<Vec<usize>> =
                        vec![vec![]; unique_sorted.len()];
                    for (child_row, &child_pid) in child_parent_ids.iter().enumerate() {
                        if let Ok(ordinal) = unique_sorted.binary_search(&child_pid) {
                            ordinal_to_children[ordinal].push(child_row);
                        }
                    }

                    fingerprints.push(ordinal_to_children);
                }
            }
        }
    }

    fingerprints
}

/// Validates that no ID column has overlapping values across batch groups.
/// Uses `payload_relations` to discover all ID columns that should be unique.
///
/// For all columns: checks that unique ID sets don't overlap across batches.
/// For primary ID columns only: additionally checks that there are no duplicate
/// IDs within a single batch (primary IDs must be unique per row).
pub(crate) fn assert_no_id_overlaps<S: OtapBatchStore, const N: usize>(
    batches: &[[Option<RecordBatch>; N]],
) {
    for &payload_type in S::allowed_payload_types() {
        let idx = payload_to_idx(payload_type);
        let info = payload_relations(payload_type);
        let primary_id_name = info.primary_id.as_ref().map(|p| p.name);

        for relation in info.relations {
            let is_primary = primary_id_name == Some(relation.key_col);
            let mut seen_across_batches = HashSet::new();

            for group in batches.iter() {
                let Some(batch) = &group[idx] else {
                    continue;
                };

                let Some(col) = access_column(relation.key_col, &batch.schema(), batch.columns())
                else {
                    continue;
                };

                let ids = collect_row_ids(col.as_ref());
                let unique: HashSet<u32> = ids.iter().copied().collect();

                // Primary ID columns must have unique values within each batch.
                if is_primary {
                    assert_eq!(
                        ids.len(),
                        unique.len(),
                        "Duplicate IDs within batch for primary column '{}'",
                        relation.key_col,
                    );
                }

                // All ID columns must not overlap across batches.
                for &id in &unique {
                    assert!(
                        seen_across_batches.insert(id),
                        "Overlapping ID {id} in column '{}' across batches",
                        relation.key_col,
                    );
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------
// ID extraction helpers
// ---------------------------------------------------------------------------

/// Collects per-row logical ID values as `u32`, resolving dictionary encoding.
/// Handles plain UInt16, UInt32, and dictionary-encoded columns.
pub(crate) fn collect_row_ids(col: &dyn Array) -> Vec<u32> {
    match col.data_type() {
        DataType::UInt16 => col
            .as_primitive::<UInt16Type>()
            .values()
            .iter()
            .map(|&v| v as u32)
            .collect(),
        DataType::UInt32 => col
            .as_primitive::<UInt32Type>()
            .values()
            .iter()
            .copied()
            .collect(),
        DataType::Dictionary(key_type, _) => match key_type.as_ref() {
            DataType::UInt8 => collect_dict_row_ids::<UInt8Type>(col),
            DataType::UInt16 => collect_dict_row_ids::<UInt16Type>(col),
            _ => unreachable!("Unsupported dictionary key type"),
        },
        _ => unreachable!("Unsupported column type: {:?}", col.data_type()),
    }
}

fn collect_dict_row_ids<K>(col: &dyn Array) -> Vec<u32>
where
    K: ArrowDictionaryKeyType,
    K::Native: ArrowNativeType,
{
    let dict = col.as_dictionary::<K>();
    let values = dict.values();
    match values.data_type() {
        DataType::UInt16 => {
            let vals = values.as_primitive::<UInt16Type>();
            dict.keys()
                .values()
                .iter()
                .map(|k: &K::Native| vals.value(k.as_usize()) as u32)
                .collect()
        }
        DataType::UInt32 => {
            let vals = values.as_primitive::<UInt32Type>();
            dict.keys()
                .values()
                .iter()
                .map(|k: &K::Native| vals.value(k.as_usize()))
                .collect()
        }
        _ => unreachable!("Unsupported dictionary value type"),
    }
}

// /// Utility for pretty printing a bunch of otap batches, nice for debugging
// pub(crate) fn pretty_print_otap_batches<const N: usize>(batches: &[[Option<RecordBatch>; N]]) {
//     for (idx, b) in batches.iter().enumerate() {
//         use arrow::util::pretty;
//         println!("-----Batch #{}------", idx);
//         for rb in b.iter().flatten().cloned() {
//             println!("{}", pretty::pretty_format_batches(&[rb]).unwrap());
//         }
//         println!("-----End Batch #{}------", idx);
//     }
// }
