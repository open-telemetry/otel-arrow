// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Implementation of the columnar policy evaluation engine.
//!
//! This module evaluates compiled telemetry policies against OTAP Arrow
//! record batches. Instead of the row-by-row evaluation in `policy-rs`,
//! it operates on entire columns, using per-policy bitmaps to track which
//! rows each policy matches.
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
//! See the "Error Handling" section of the policy specification.

mod policy_match;
mod translate;

use otel_arrow_dfe_config::SignalType;
use otel_arrow_dfe_pdata::{OtapArrowRecords, OtapPayloadHelpers};
use otel_arrow_dfe_query_engine::pipeline::pipeline_expr::Predicate;
use otel_arrow_dfe_query_engine::pipeline::{RecordType, SignalContext, SignalKind};
use policy_rs::engine::{CompiledKeep, CompiledMatchers, CompiledPolicy, LogSignal, Signal};
use policy_rs::{PolicyError, PolicySnapshot};

use policy_match::PolicyMatch;
use translate::{existence_check_to_logical, typed_check_to_logical};

/// Columnar policy evaluation engine.
///
/// Evaluates all policies in a [`PolicySnapshot`] against an OTAP Arrow
/// batch, producing per-policy match bitmaps that drive the keep/transform
/// stages.
pub struct PolicyEngine {}

impl PolicyEngine {
    /// Create a new instance.
    pub fn new() -> Self {
        Self {}
    }

    /// Evaluate the policies in the snapshot on all the rows within the
    /// OTAP batch, applying keep and transform actions.
    pub fn evaluate_and_transform(
        &self,
        snapshot: &PolicySnapshot,
        otap_batch: OtapArrowRecords,
    ) -> Result<OtapArrowRecords, PolicyError> {
        // TODO
        // - volume tracking
        // - other signal type handling

        match otap_batch.signal_type() {
            SignalType::Logs => {
                let Some(compiled_matchers) = LogSignal::compiled_matchers(snapshot) else {
                    // No log policies compiled -- pass through unmodified.
                    return Ok(otap_batch);
                };

                let num_rows = otap_batch
                    .root_record_batch()
                    .map(|rb| rb.num_rows())
                    .unwrap_or(0);

                let policy_matches =
                    find_matching_policies(compiled_matchers, &otap_batch)?;

                let _discard_bitmap = resolve_keep(
                    &compiled_matchers.policies,
                    &policy_matches,
                    num_rows,
                );

                // TODO: filter the otap_batch using discard_bitmap
                // TODO: apply transforms from matching keep:"all" policies
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
/// Processes policies in restrictiveness order (most restrictive first):
/// 1. `keep: "none"` policies -- each one's newly-discarded rows are its hits;
///    rows it matched that were already discarded by a higher-priority drop
///    policy are its misses.
/// 2. (Future: `keep: "N%"` sampling policies would be inserted here.)
/// 3. `keep: "all"` policies -- rows matched and not discarded are hits;
///    rows matched but discarded are misses.
///
/// Returns the accumulated discard bitmap (true = drop the row).
///
/// # Match tracking
///
/// Per the spec: if a record is kept, all matching policies record a hit.
/// If a record is dropped, only the most restrictive policy (the "winner")
/// records a hit; all other matching policies record a miss.
///
/// Since policies are processed in restrictiveness order, the first
/// `keep: "none"` policy to claim a row is the winner for that row.
/// Subsequent `keep: "none"` policies that also match record a miss.
fn resolve_keep(
    policies: &[CompiledPolicy<LogSignal>],
    policy_matches: &[PolicyMatch],
    num_rows: usize,
) -> PolicyMatch {
    // Accumulates all rows that have been discarded so far.
    let mut already_discarded = PolicyMatch::None;

    // Process keep:"none" policies first (most restrictive).
    // Policies are already in alphabetical ID order from compilation,
    // and all keep:"none" policies have equal restrictiveness, so
    // iteration order is the tie-breaker.
    for (idx, policy) in policies.iter().enumerate() {
        if !matches!(policy.keep, CompiledKeep::None) {
            continue;
        }

        let matched = &policy_matches[idx];

        // Rows this policy wins: matched AND NOT already_discarded.
        // These are the rows where this policy is the first drop policy to match.
        let wins = matched.and_not(&already_discarded);
        let hits = wins.count_true(num_rows);

        // Rows this policy matched but a higher-priority drop policy already
        // claimed: matched AND already_discarded.
        let total_matched = matched.count_true(num_rows);
        let misses = total_matched - hits;

        // Record stats.
        for _ in 0..hits {
            policy.stats.record_hit();
        }
        for _ in 0..misses {
            policy.stats.record_miss();
        }

        // Add this policy's matches to the discard pile.
        already_discarded = already_discarded.or(matched);
    }

    // TODO: process keep:"N%" sampling policies here (in restrictiveness
    // order). For each sampling policy:
    //   1. effective = matched AND NOT already_discarded
    //   2. make per-row sampling decision on effective rows
    //   3. sampled_out rows: hit for this policy, added to already_discarded
    //   4. surviving rows: no action (will be counted in keep:"all" pass)

    // Process keep:"all" policies (least restrictive).
    for (idx, policy) in policies.iter().enumerate() {
        if !matches!(policy.keep, CompiledKeep::All) {
            continue;
        }

        let matched = &policy_matches[idx];

        // Total rows this policy matched.
        let total_matched = matched.count_true(num_rows);
        if total_matched == 0 {
            continue;
        }

        // Rows this policy matched that were discarded by drop policies.
        let overridden = matched.and(&already_discarded);
        let misses = overridden.count_true(num_rows);

        // Rows this policy matched that survived -- these are hits.
        let hits = total_matched - misses;

        // Record stats.
        for _ in 0..hits {
            policy.stats.record_hit();
        }
        for _ in 0..misses {
            policy.stats.record_miss();
        }
    }

    already_discarded
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

        let mut predicate = Predicate::try_new(&logical_expr, record_type.clone())
            .map_err(|e| PolicyError::CompileError {
                reason: format!("existence check: {e}"),
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

        let mut predicate = Predicate::try_new(&logical_expr, record_type.clone())
            .map_err(|e| PolicyError::CompileError {
                reason: format!("typed check: {e}"),
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
