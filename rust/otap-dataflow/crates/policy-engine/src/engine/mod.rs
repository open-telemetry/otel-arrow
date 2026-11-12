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
use otel_arrow_dfe_query_engine::pipeline::pipeline_expr::{Predicate, PredicateResult};
use otel_arrow_dfe_query_engine::pipeline::{RecordType, SignalContext, SignalKind};
use policy_rs::engine::{CompiledMatchers, LogSignal, Signal};
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
                let _policy_matches =
                    find_matching_policies(compiled_matchers, &otap_batch)?;

                // TODO: apply keep resolution and transforms using the bitmaps
            }
            _ => {
                // TODO: support other signal types
            }
        }

        Ok(otap_batch)
    }
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
/// For each matcher, the result is folded into the per-policy bitmap:
/// - Non-negated matchers: AND into the "matched" bitmap (all must match).
/// - Negated matchers: OR into the "disqualified" bitmap (any hit disqualifies).
///
/// The final match for each policy is `matched AND NOT disqualified`.
// TODO: generalize to other signal types (MetricSignal, TraceSignal)
fn find_matching_policies(
    compiled: &CompiledMatchers<LogSignal>,
    otap_batch: &OtapArrowRecords,
) -> Result<Vec<PolicyMatch>, PolicyError> {
    let policy_count = compiled.policies.len();
    if policy_count == 0 {
        return Ok(Vec::new());
    }

    // Two bitmaps per policy:
    //   matched:      initialized to All, AND'd with each non-negated matcher result
    //   disqualified: initialized to None, OR'd with each negated matcher result
    let mut matched = vec![PolicyMatch::All; policy_count];
    let mut disqualified = vec![PolicyMatch::None; policy_count];

    // TODO: evaluate Vectorscan databases (string pattern matchers)

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

        // Negation is already baked into the LogicalExpression by
        // existence_check_to_logical, so we always AND here.
        // The check's is_negated flag was handled during translation.
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

        // Negation is already baked into the LogicalExpression by
        // typed_check_to_logical, so we always AND here.
        matched[check.policy_index] = matched[check.policy_index].and(&policy_match);
    }

    // Final result per policy: matched AND NOT disqualified
    let mut results = Vec::with_capacity(policy_count);
    for (m, d) in matched.into_iter().zip(disqualified.into_iter()) {
        let result = m.and_not(&d);
        results.push(result);
    }

    Ok(results)
}
