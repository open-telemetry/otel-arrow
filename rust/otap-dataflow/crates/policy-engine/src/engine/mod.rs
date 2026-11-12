// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the columnar policy evaluation engine.
//!
//! This module evaluates compiled telemetry policies against OTAP Arrow batches.
//!
//! ---
//!
//! TODO - This work is still experimental and there are various TODOs that should be completed,
//! some of which are listed below.
//!
//! # Functionality (things that aren't yet supported)
//!
//! - Metics/Traces
//! - Sampling
//! - Volume Tracking
//!
//! # Performance
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
//!   within a batch rather than allocating and discarding them each time. The `PolicyMatch` bitmaps are prime
//!   candidates for pooling since they have predictable sizes.
//!
//! # Error handling
//!
//! TODO: validate that errors returned by this module conform to the policy
//! spec's error handling requirements:
//! - Parse errors: malformed policies must be skipped; other policies continue.
//! - Compilation errors: invalid policies must be inert (never match/transform).
//! - Runtime evaluation errors: telemetry must pass through unmodified.
//! - All categories must be fail-open: no telemetry loss on error.
//! - Errors should be reported via `PolicySyncStatus` for observability.
//!
//! Currently this just returns CompileError everywhere, alongside very simple (some might say
//! uninformative) error messages, which I doubt is correct according to what the spec suggests.
//!
//! See the "Error Handling" section of the policy specification.

mod policy_match;
mod transform;
mod translate;

use arrow::array::BooleanArray;
use arrow::compute::not;
use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::otap::filter::{IdBitmapPool, filter_otap_batch};
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use otel_arrow_dfe_query_engine::pipeline::pipeline_expr::Predicate;
use otel_arrow_dfe_query_engine::pipeline::{RecordType, SignalContext, SignalKind};
use policy_rs::engine::{CompiledKeep, CompiledMatchers, CompiledPolicy, LogSignal, Signal};
use policy_rs::{PolicyError, PolicySnapshot};

use policy_match::PolicyMatch;
use transform::{apply_policy_transforms, concat_log_batches};
use translate::{existence_check_to_logical, typed_check_to_logical};

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
                            // TODO Arc Arc Arc ??
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

                // Apply transforms from matching keep:"all" policies.
                // Policies are already in alphabetical ID order from compilation.
                // Filtered match bitmaps are computed on the fly to avoid
                // materializing a Vec of all filtered bitmaps up front.
                let surviving_rows = filtered_batch.num_items();

                for (idx, policy) in compiled_matchers.policies.iter().enumerate() {
                    // TODO this should be policy matches != ALL (so we don't accidentally
                    // evaluate unsupported "sampling" policies)?
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

                    // Get the mask of non-dropped rows that are matched by this policy
                    let filtered_match = policy_matches[idx].filter_by(&keep_mask);

                    match filtered_match {
                        PolicyMatch::None => continue,
                        PolicyMatch::All => {
                            // Transform all surviving rows in place.
                            apply_policy_transforms(&mut filtered_batch, transform).map_err(
                                |e| PolicyError::CompileError {
                                    reason: format!("transform: {e}"),
                                },
                            )?;
                        }
                        PolicyMatch::Some(_) => {
                            let map_transform_filter = |e| PolicyError::CompileError {
                                reason: format!("transform filter: {e}"),
                            };

                            // Apply transforms to matched subset.
                            let selection = filtered_match.to_boolean_array(surviving_rows);
                            let mut matched_batch =
                                filter_otap_batch(&selection, &filtered_batch, &mut id_bitmap_pool)
                                    .map_err(map_transform_filter)?;
                            apply_policy_transforms(&mut matched_batch, transform).map_err(
                                |e| PolicyError::CompileError {
                                    reason: format!("transform: {e}"),
                                },
                            )?;

                            // Get the non-matched rows
                            let not_selection =
                                not(&selection).map_err(|e| PolicyError::CompileError {
                                    reason: format!("transform: {e}"),
                                })?;
                            let unmatched_batch = filter_otap_batch(
                                &not_selection,
                                &filtered_batch,
                                &mut id_bitmap_pool,
                            )
                            .map_err(map_transform_filter)?;

                            // Concat back together.
                            filtered_batch = concat_log_batches(matched_batch, unmatched_batch)
                                .map_err(|e| PolicyError::CompileError {
                                    reason: format!("transform concat: {e}"),
                                })?;
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
    // TODO - DOUBLE CHECK the assumption that "keep" doesn't contribute to the drop decision.
    // E.g. if no policy matched for some row, do we keep it? What does the go-collector policy
    // processor do?
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

/// Returns a [`Vec`] of [`PolicyMatch`] indicating which rows matched each
/// policy at the corresponding index.
///
/// The evaluation iterates through three categories of matchers from the
/// compiled policy snapshot:
///
/// 1. **Vectorscan databases** -- string pattern matchers (exact, regex,
///    starts_with, ends_with, contains) grouped by (field, negated).
/// 2. **Existence checks** -- field presence/absence tests.
/// 3. **Typed checks** -- typed equality and numeric comparisons.
///
/// For existence and typed checks, negation is handled during expression
/// translation (the `NOT` is baked into the `LogicalExpression`), so all
/// results are AND'd into the per-policy `matched` bitmap.
///
/// The final match for each policy is the `matched` bitmap.
// TODO: generalize to other signal types (MetricSignal, TraceSignal)
fn find_matching_policies(
    compiled: &CompiledMatchers<LogSignal>,
    otap_batch: &OtapArrowRecords,
) -> Result<Vec<PolicyMatch>, PolicyError> {
    let policy_count = compiled.policies.len();
    if policy_count == 0 {
        return Ok(Vec::new());
    }

    // Per-policy bitmap: initialized to All, AND'd with each matcher result.
    // Negation for existence and typed checks is baked into the translated
    // LogicalExpression, so we always AND.
    let mut matched = vec![PolicyMatch::All; policy_count];

    // TODO: evaluate Vectorscan databases (string pattern matchers).
    //
    // Vectorscan scans all patterns for a given (field, negated) group in one
    // pass and returns which patterns matched. Unlike existence/typed checks,
    // negation cannot be baked into the scan -- it must be tracked separately.
    // When adding Vectorscan support:
    //   - For non-negated databases: patterns that matched AND into `matched`.
    //     Patterns in the database that did NOT match must also AND false into
    //     `matched` for their respective policies.
    //   - For negated databases: patterns that matched must set the
    //     corresponding rows to not-matched for their policies. This may
    //     require a separate `disqualified` bitmap (OR'd with negated hits)
    //     and a final `matched AND NOT disqualified` step, or the negation
    //     can be applied inline by AND'ing `NOT(hit)` into `matched`.

    // Evaluate existence checks.
    //
    // TODO: consider grouping checks that target the same field into a single
    // Predicate evaluation to avoid redundant column access.
    let record_type = RecordType::Signal(SignalContext::Single(SignalKind::Logs));

    for check in &compiled.existence_checks {
        let logical_expr = existence_check_to_logical(check);

        let mut predicate =
            Predicate::try_new(&logical_expr, record_type.clone()).map_err(|e| {
                PolicyError::CompileError {
                    reason: format!("existence check: {e}"),
                }
            })?;

        let result = predicate
            .evaluate(otap_batch)
            .map_err(|e| PolicyError::CompileError {
                reason: format!("existence check eval: {e}"),
            })?;

        let policy_match = PolicyMatch::from_predicate_result(result);
        matched[check.policy_index] = matched[check.policy_index].and(&policy_match);
    }

    // Evaluate typed checks (equals, gt, gte, lt, lte).
    //
    // TODO: consider grouping checks that target the same field into a single
    // compound expression to avoid redundant column access.
    for check in &compiled.typed_checks {
        let Some(logical_expr) = typed_check_to_logical(check) else {
            // Unsupported matcher (e.g., Equals(Bytes)) -- skip.
            // Since we use a bitmap approach (not counts), we explicitly
            // mark this policy as not matching for this check.
            matched[check.policy_index] = PolicyMatch::None;
            continue;
        };

        let mut predicate =
            Predicate::try_new(&logical_expr, record_type.clone()).map_err(|e| {
                PolicyError::CompileError {
                    reason: format!("typed check: {e}"),
                }
            })?;

        let result = predicate
            .evaluate(otap_batch)
            .map_err(|e| PolicyError::CompileError {
                reason: format!("typed check eval: {e}"),
            })?;

        let policy_match = PolicyMatch::from_predicate_result(result);
        matched[check.policy_index] = matched[check.policy_index].and(&policy_match);
    }

    Ok(matched)
}
