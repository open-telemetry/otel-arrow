// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the columnar policy evaluation engine.
//!
//! This module evaluates compiled telemetry policies against OTAP Arrow batches.
//!
//! This work is still experimental and there are various TODOs that should be completed.
//!
//! # Functionality (things that aren't yet supported)
//!
//! - Metics/Traces
//! - Sampling
//! - Volume Tracking
//!
//! # Performance (possible optimizations)
//!
//! The following optimizations should be evaluated and implemented if they seem worthwhile:
//!
//! - **Short-circuit evaluation**: if all rows are discarded at any point during keep resolution
//!   skip remaining policy transformation and return an empty batch immediately (note: hit/miss
//!   statistics must still be updated)
//!
//! - **Predicate caching**: keep any planned expression evaluation instances from the query-engine
//!   crate instances between OTAP batch policy engine evaluations instead of re-planning from
//!   the expression eval on every call.  The `Predicate` contains a compiled `ScopedExpr` and a
//!   `SessionContext` that can be reused across batches.
//!
//! - **Buffer pooling**: reuse `BooleanBuffer` heap allocations across batches and across policies
//!   within a batch rather than allocating and discarding them each time. The `PolicyMatch`
//!   bitmaps are prime candidates for pooling since they have predictable sizes.
//!
//! # Error handling (needs improvement)
//!
//! TODO: validate that errors returned by this module conform to the policy spec's error handling
//! requirements:
//! - Parse errors: malformed policies must be skipped; other policies continue.
//! - Compilation errors: invalid policies must be inert (never match/transform).
//! - Runtime evaluation errors: telemetry must pass through unmodified.
//! - All categories must be fail-open: no telemetry loss on error.
//! - Errors should be reported via `PolicySyncStatus` for observability.
//!
//! Currently this just returns CompileError everywhere, alongside very simple error messages,
//! which is maybe not correct according to what the spec suggests.
//!
//! See the "Error Handling" section of the policy specification.

mod evaluate;
mod policy_match;
mod transform;

use arrow::array::BooleanArray;
use arrow::compute::not;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use policy_rs::engine::{CompiledKeep, CompiledMatchers, CompiledPolicy, LogSignal, Signal};
use policy_rs::{PolicyError, PolicySnapshot};

use evaluate::{evaluate_existence_check, evaluate_typed_check, evaluate_vectorscan_databases};
use policy_match::PolicyMatch;
use transform::{apply_policy_transforms, concat_log_batches};

/// Columnar policy evaluation engine.
///
/// Evaluates all policies in a [`PolicySnapshot`] against an OTAP Arrow batch.
pub struct PolicyEngine {}

impl PolicyEngine {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate the policies in the snapshot on all the rows within the OTAP batch, applying keep
    /// and transform modifications.
    pub fn evaluate_and_transform(
        &self,
        snapshot: &PolicySnapshot,
        otap_batch: OtapArrowRecords,
    ) -> Result<OtapArrowRecords, PolicyError> {
        // TODO:
        // - volume tracking
        // - handling of metrics & traces

        match otap_batch.signal_type() {
            SignalType::Logs => {
                let Some(compiled_matchers) = LogSignal::compiled_matchers(snapshot) else {
                    // No log policies compiled -- pass through unmodified.
                    return Ok(otap_batch);
                };

                // keep track of how many rows we had in the incoming batch before any
                // sampling/discards were applied
                let input_num_rows = otap_batch.num_items();

                if input_num_rows == 0 {
                    // nothing to do!
                    return Ok(otap_batch);
                }

                let policy_matches = find_matching_policies(compiled_matchers, &otap_batch)?;

                let keep_mask = resolve_keep_and_record_stats(
                    &compiled_matchers.policies,
                    &policy_matches,
                    input_num_rows,
                );

                // TODO keep id_bitmap_pool for reuse between invocations
                let mut id_bitmap_pool = IdBitmapPool::new();

                let mut filtered_batch = match &keep_mask {
                    PolicyMatch::None => {
                        // short-circuit: everything discarded, just return empty
                        return Ok(OtapArrowRecords::Logs(
                            otel_arrow_dfe_pdata::otap::Logs::default(),
                        ));
                    }
                    PolicyMatch::All => {
                        // nothing filtered out
                        otap_batch
                    }
                    PolicyMatch::Some(keep_selection) => {
                        filter_otap_batch(
                            &BooleanArray::new(keep_selection.clone(), None),
                            &otap_batch,
                            &mut id_bitmap_pool,
                        )
                        // TODO - not a compile error
                        .map_err(|e| PolicyError::CompileError {
                            reason: format!("filter: {e}"),
                        })?
                    }
                };

                // Build an indices vector mapping current row position to original row position.
                // This is needed because split/concat during transforms changes the row order, but
                // policy match bitmaps are computed against the original row order, so for each
                // policy apply below we need to re-order these match selection vecs based on row
                // order after the previous transforms were applied.
                let mut indices: Vec<usize> = match &keep_mask {
                    PolicyMatch::All => (0..input_num_rows).collect(),
                    PolicyMatch::None => Vec::new(), // unreachable: we returned early above
                    PolicyMatch::Some(buf) => {
                        (0..input_num_rows).filter(|&i| buf.value(i)).collect()
                    }
                };

                // Apply transforms from matching keep:"all" policies
                //
                // Note: the spec states that the policies must be applied in a "consistent order
                // (e.g., alphabetically by ID)", and we're assuming that the compiled matcher we
                // received from the snapshot has already applied this sort order.
                for (idx, policy) in compiled_matchers.policies.iter().enumerate() {
                    // TODO this should be policy matches != ALL (so we don't accidentally evaluate
                    // unsupported "sampling" policies)?
                    // TODO Do the transforms run on the rows that are not dropped in case of None?
                    if matches!(policy.keep, CompiledKeep::None) {
                        // Transforms never run on dropped rows.
                        continue;
                    }

                    let Some(transform) = &policy.transform else {
                        continue;
                    };
                    if transform.is_empty() {
                        continue;
                    }

                    // Remap the policy's match bitmap from original row order to current row
                    // order using the indices vector.
                    let current_match = policy_matches[idx].take(&indices);

                    match current_match {
                        PolicyMatch::None => continue,
                        PolicyMatch::All => {
                            // Transform all surviving rows in place.
                            // Indices unchanged since no split/concat.
                            apply_policy_transforms(&mut filtered_batch, transform).map_err(
                                |e| PolicyError::CompileError {
                                    reason: format!("transform: {e}"),
                                },
                            )?;
                        }
                        PolicyMatch::Some(ref bitmap) => {
                            let map_err = |e| PolicyError::CompileError {
                                reason: format!("transform filter: {e}"),
                            };

                            // Split indices into matched and unmatched...
                            // TODO - preallocate, optimize bitmap scan, reuse allocations, etc.
                            let mut matched_indices = Vec::new();
                            let mut unmatched_indices = Vec::new();
                            for (i, &orig) in indices.iter().enumerate() {
                                if bitmap.value(i) {
                                    matched_indices.push(orig);
                                } else {
                                    unmatched_indices.push(orig);
                                }
                            }

                            // Split the batch by the current-order mask.
                            let selection =
                                current_match.to_boolean_array(filtered_batch.num_items());
                            let mut matched_batch =
                                filter_otap_batch(&selection, &filtered_batch, &mut id_bitmap_pool)
                                    .map_err(map_err)?;

                            let not_selection =
                                not(&selection).map_err(|e| PolicyError::CompileError {
                                    reason: format!("transform: {e}"),
                                })?;
                            let unmatched_batch = filter_otap_batch(
                                &not_selection,
                                &filtered_batch,
                                &mut id_bitmap_pool,
                            )
                            .map_err(map_err)?;

                            // Apply transforms to matched subset.
                            apply_policy_transforms(&mut matched_batch, transform).map_err(
                                |e| PolicyError::CompileError {
                                    reason: format!("transform: {e}"),
                                },
                            )?;

                            // Concat back together: matched first, then unmatched.
                            filtered_batch = concat_log_batches(matched_batch, unmatched_batch)
                                .map_err(|e| PolicyError::CompileError {
                                    reason: format!("transform concat: {e}"),
                                })?;

                            // Update indices to match the new row order.
                            matched_indices.extend(unmatched_indices);
                            indices = matched_indices;
                        }
                    }
                }

                return Ok(filtered_batch);
            }
            _ => {
                // TODO: support other signal types
            }
        }

        Ok(otap_batch)
    }
}

/// Resolve the keep decisions for all policies and record match statistics.
///
/// The policy spec says that the decision about whether to keep or discard some row must be made
/// by the most restrictive policy (see the spec for the rules about how restrictiveness is
/// determined).
///
/// For each row, this will determine the most restrictive policy that matches it and then use the
/// keep decision from this policy. This returns the [`PolicyMatch`] indicating which rows to keep.
///
/// This function also updates the policies' hit/miss statistics.
fn resolve_keep_and_record_stats(
    policies: &[CompiledPolicy<LogSignal>],
    policy_matches: &[PolicyMatch],
    num_rows: usize,
) -> PolicyMatch {
    // Accumulates all rows that have been discarded. This will be updated as we iterate through
    // policies in order of restrictiveness.
    let mut already_discarded = PolicyMatch::None;

    // Process keep:"none" policies first (most restrictive).
    //
    // Note that the engine does not currently support any "keep" policy other than "all" (least
    // restrictive) and "none" (most restrictive). We first need to determine the "most
    // restrictive" that matches some row so given what is currently supported, this effectively
    // means just look at all the "none" policies first.
    //
    // "tie-breaking": The spec also says that policies must be processed in a "consistent order",
    // so we're being consistent here for all these policies with equal restrictiveness via the fact
    // that Policies are already in alphabetical ID order from compilation.
    //
    for (idx, policy) in policies.iter().enumerate() {
        // skip the less restrictive policies ...
        if !matches!(policy.keep, CompiledKeep::None) {
            continue;
        }

        let matched = &policy_matches[idx];

        // Record Stats ...

        // hits: these are the rows where this policy is the first drop policy to match. E.g. this
        // is the policy which made the decision to drop the row.
        let wins = matched.and_not(&already_discarded);
        let hits = wins.count_true(num_rows);

        // misses: rows this policy matched but a previous policy already matched
        // already discarded.
        let total_matched = matched.count_true(num_rows);
        let misses = total_matched - hits;

        // TODO - we should get policy-rs to expose a method to increment these by some amount
        // instead of iterating and incrementing by one each time.
        for _ in 0..hits {
            policy.stats.record_hit();
        }
        for _ in 0..misses {
            policy.stats.record_miss();
        }

        // Update the bitmask of discarded rows
        already_discarded = already_discarded.or(matched);
    }

    // TODO: process keep: <sampling> policies

    // Process keep:"all" policies (least restrictive). Since these don't contribute to the
    // drop decision, this is just updating statistics.
    //
    // TODO - determine what should happen when no policies match. Assumption now is that we keep
    // the telemetry. What does the go-collector policy processor do?
    //
    for (idx, policy) in policies.iter().enumerate() {
        if !matches!(policy.keep, CompiledKeep::All) {
            continue;
        }

        let matched = &policy_matches[idx];

        // Record stats ...

        // Total rows this policy matched.
        let total_matched = matched.count_true(num_rows);
        if total_matched == 0 {
            continue;
        }

        // Rows this policy matched that were discarded by drop policies.
        let matches_dropped = matched.and(&already_discarded);
        let misses = matches_dropped.count_true(num_rows);

        // Rows this policy matched that survived -- these are hits.
        let hits = total_matched - misses;

        // TODO - we should get policy-rs to expose a method to increment these by some amount
        // instead of iterating and incrementing by one each time.
        for _ in 0..hits {
            policy.stats.record_hit();
        }
        for _ in 0..misses {
            policy.stats.record_miss();
        }
    }

    // return the mask of which rows to keep
    PolicyMatch::not(&already_discarded)
}

/// Returns a [`Vec`] of [`PolicyMatch`] indicating which rows matched each policy at the
/// corresponding index.
///
// TODO: generalize to other signal types (MetricSignal, TraceSignal)
fn find_matching_policies(
    compiled: &CompiledMatchers<LogSignal>,
    otap_batch: &OtapArrowRecords,
) -> Result<Vec<PolicyMatch>, PolicyError> {
    let policy_count = compiled.policies.len();
    if policy_count == 0 {
        return Ok(Vec::new());
    }

    // Per-policy bitmap: initialized to All, but will be updated as rows are eliminated
    // as we evaluate each policy's matchers.
    let mut matched = vec![PolicyMatch::All; policy_count];

    // Evaluate Vectorscan databases. Each database groups patterns by (field, negated) and scans
    // string column values through the compiled Vectorscan DFA.
    evaluate_vectorscan_databases(compiled, otap_batch, &mut matched)?;

    // Evaluate existence checks.
    for check in &compiled.existence_checks {
        // skip evaluating the check if we've already determined there are no matching rows
        if !matches!(matched[check.policy_index], PolicyMatch::None) {
            let policy_match = evaluate_existence_check(otap_batch, check);
            matched[check.policy_index] = matched[check.policy_index].and(&policy_match);
        }
    }

    // Evaluate typed checks (equals, gt, gte, lt, lte).
    for check in &compiled.typed_checks {
        // skip evaluating the check if we've already determined there are no matching rows
        if !matches!(matched[check.policy_index], PolicyMatch::None) {
            let policy_match = evaluate_typed_check(otap_batch, check);
            matched[check.policy_index] = matched[check.policy_index].and(&policy_match);
        }
    }

    Ok(matched)
}

#[cfg(test)]
mod tests {
    use super::*;
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
    use otel_arrow_dfe_pdata::proto::opentelemetry::logs::v1::LogRecord;
    use otel_arrow_dfe_pdata::testing::round_trip::to_otap_logs;
    use policy_rs::proto::tero::policy::v1::{
        AttributePath, LogMatcher, LogRename, LogTarget, LogTransform, Policy as ProtoPolicy,
        log_matcher, log_rename, policy,
    };
    use policy_rs::{Policy, PolicyRegistry};

    fn attr_path(s: &str) -> AttributePath {
        AttributePath {
            path: vec![s.to_string()],
        }
    }

    fn attr_exists_matcher(key: &str) -> LogMatcher {
        LogMatcher {
            field: Some(log_matcher::Field::LogAttribute(attr_path(key))),
            r#match: Some(log_matcher::Match::Exists(true)),
            negate: false,
            case_insensitive: false,
        }
    }

    fn create_log_policy(
        id: &str,
        matchers: Vec<LogMatcher>,
        keep: &str,
        transform: Option<LogTransform>,
    ) -> Policy {
        Policy::new(ProtoPolicy {
            id: id.to_string(),
            name: id.to_string(),
            enabled: true,
            target: Some(policy::Target::Log(LogTarget {
                r#match: matchers,
                keep: keep.to_string(),
                transform,
                sample_key: None,
            })),
            ..Default::default()
        })
    }

    /// Scenario: two transform policies match different subsets of rows.
    /// The first policy's split/concat reorders the batch; the second
    /// policy must still apply its transforms to the correct rows.
    ///
    /// Guarantees: transforms are applied to the rows that matched each
    /// policy's matchers, regardless of how previous transforms reordered
    /// the batch.
    #[test]
    fn transforms_applied_to_correct_rows_after_reorder() {
        let registry = PolicyRegistry::new();
        let handle = registry.register_provider();

        // Policy A (id="a-rename-marker-a"): matches rows with "marker_a" attr,
        // renames "old_a" -> "new_a"
        let policy_a = create_log_policy(
            "a-rename-marker-a",
            vec![attr_exists_matcher("marker_a")],
            "all",
            Some(LogTransform {
                rename: vec![LogRename {
                    from: Some(log_rename::From::FromLogAttribute(attr_path("old_a"))),
                    to: "new_a".to_string(),
                    upsert: false,
                }],
                ..Default::default()
            }),
        );

        // Policy B (id="b-rename-marker-b"): matches rows with "marker_b" attr,
        // renames "old_b" -> "new_b"
        let policy_b = create_log_policy(
            "b-rename-marker-b",
            vec![attr_exists_matcher("marker_b")],
            "all",
            Some(LogTransform {
                rename: vec![LogRename {
                    from: Some(log_rename::From::FromLogAttribute(attr_path("old_b"))),
                    to: "new_b".to_string(),
                    upsert: false,
                }],
                ..Default::default()
            }),
        );

        handle.update(vec![policy_a, policy_b]);
        let snapshot = registry.snapshot();

        // Create 4 log records:
        //   Row 0: has marker_a + old_a (should be renamed by policy A)
        //   Row 1: has marker_b + old_b (should be renamed by policy B)
        //   Row 2: has both markers + old_a + old_b (both renames)
        //   Row 3: no markers (no transforms)
        let otap_batch = to_otap_logs(vec![
            LogRecord::build()
                .severity_text("INFO")
                .body(AnyValue::new_string("row 0"))
                .attributes(vec![
                    KeyValue::new("marker_a", AnyValue::new_bool(true)),
                    KeyValue::new("old_a", AnyValue::new_string("val_a_0")),
                ])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .body(AnyValue::new_string("row 1"))
                .attributes(vec![
                    KeyValue::new("marker_b", AnyValue::new_bool(true)),
                    KeyValue::new("old_b", AnyValue::new_string("val_b_1")),
                ])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .body(AnyValue::new_string("row 2"))
                .attributes(vec![
                    KeyValue::new("marker_a", AnyValue::new_bool(true)),
                    KeyValue::new("marker_b", AnyValue::new_bool(true)),
                    KeyValue::new("old_a", AnyValue::new_string("val_a_2")),
                    KeyValue::new("old_b", AnyValue::new_string("val_b_2")),
                ])
                .finish(),
            LogRecord::build()
                .severity_text("INFO")
                .body(AnyValue::new_string("row 3"))
                .attributes(vec![KeyValue::new(
                    "unrelated",
                    AnyValue::new_string("val_3"),
                )])
                .finish(),
        ]);

        let engine = PolicyEngine::new();
        let result = engine
            .evaluate_and_transform(&snapshot, otap_batch)
            .expect("evaluation should succeed");

        // All 4 rows should survive (no drop policies).
        let output_rows = result
            .root_record_batch()
            .map(|rb| rb.num_rows())
            .unwrap_or(0);
        assert_eq!(output_rows, 4, "all 4 rows should survive");

        // Check that the attribute renames were applied correctly.
        // After transforms: "old_a" should become "new_a" on rows 0 and 2,
        // "old_b" should become "new_b" on rows 1 and 2.
        let attrs_batch = result
            .get(otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType::LogAttrs)
            .expect("should have log attributes");

        let key_col = attrs_batch
            .column_by_name("key")
            .expect("should have key column");

        // Collect all attribute keys as strings.
        let mut all_keys = Vec::new();
        for i in 0..attrs_batch.num_rows() {
            if let Some(key) = get_string_at(key_col, i) {
                all_keys.push(key);
            }
        }

        // "old_a" should NOT appear (it was renamed to "new_a")
        assert!(
            !all_keys.contains(&"old_a".to_string()),
            "old_a should have been renamed to new_a, found keys: {:?}",
            all_keys
        );
        // "new_a" SHOULD appear
        assert!(
            all_keys.contains(&"new_a".to_string()),
            "new_a should exist after rename, found keys: {:?}",
            all_keys
        );
        // "old_b" should NOT appear (it was renamed to "new_b")
        assert!(
            !all_keys.contains(&"old_b".to_string()),
            "old_b should have been renamed to new_b, found keys: {:?}",
            all_keys
        );
        // "new_b" SHOULD appear
        assert!(
            all_keys.contains(&"new_b".to_string()),
            "new_b should exist after rename, found keys: {:?}",
            all_keys
        );

        // Verify policy stats
        let entries: Vec<_> = snapshot.iter().collect();
        // Policy A matched rows 0, 2 (2 hits)
        let a_stats = entries
            .iter()
            .find(|e| e.policy.id() == "a-rename-marker-a")
            .unwrap();
        assert_eq!(a_stats.stats.hits(), 2, "policy A should have 2 hits");
        // Policy B matched rows 1, 2 (2 hits)
        let b_stats = entries
            .iter()
            .find(|e| e.policy.id() == "b-rename-marker-b")
            .unwrap();
        assert_eq!(b_stats.stats.hits(), 2, "policy B should have 2 hits");
    }

    /// Helper: get a string value from a possibly dict-encoded column.
    fn get_string_at(col: &arrow::array::ArrayRef, idx: usize) -> Option<String> {
        use arrow::array::{Array, DictionaryArray, StringArray};
        use arrow::datatypes::{UInt8Type, UInt16Type};

        if col.is_null(idx) {
            return None;
        }
        if let Some(arr) = col.as_any().downcast_ref::<StringArray>() {
            return Some(arr.value(idx).to_string());
        }
        if let Some(dict) = col.as_any().downcast_ref::<DictionaryArray<UInt8Type>>() {
            let values = dict.values().as_any().downcast_ref::<StringArray>()?;
            let key = dict.keys().value(idx) as usize;
            return Some(values.value(key).to_string());
        }
        if let Some(dict) = col.as_any().downcast_ref::<DictionaryArray<UInt16Type>>() {
            let values = dict.values().as_any().downcast_ref::<StringArray>()?;
            let key = dict.keys().value(idx) as usize;
            return Some(values.value(key).to_string());
        }
        None
    }
}
