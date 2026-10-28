// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Portable values shared by configuration, PDH collection, and OTAP projection.

use super::config::{MAX_SCALE_POWER10, MIN_SCALE_POWER10};
use std::collections::HashMap;
use std::sync::Arc;

// Measurements put the metadata-index crossover above 32,768 comparisons.
const METADATA_INDEX_WORK_THRESHOLD: usize = 32_768;

#[derive(Clone, Copy, Eq, PartialEq, Hash)]
struct MetadataKey {
    counter_index: usize,
    reason: &'static str,
}

impl MetadataKey {
    fn new(counter_index: usize, reason: &'static str) -> Self {
        Self {
            counter_index,
            reason,
        }
    }
}

/// A numeric performance-counter value.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum Number {
    /// An exact signed integer.
    Integer(i64),
    /// A finite calculated or scaled value, including subnormals and signed zero.
    Double(f64),
}

/// A bounded reason why a numeric value cannot be scaled.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub(super) enum ScaleError {
    /// The input or rounded result is NaN or infinite.
    #[error("scaling input or result is not finite")]
    NonFinite,
    /// The decimal exponent is outside the supported range.
    #[error(
        "scale_power10 must be between {} and {}",
        MIN_SCALE_POWER10,
        MAX_SCALE_POWER10
    )]
    InvalidScale,
}

/// One configured counter's state for a collection.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(super) enum SampleValue {
    /// A ready numeric value.
    Value(Number),
    /// A two-sample counter is in its bounded first-scrape warm-up.
    Warming,
    /// A valid base did not advance because no operations were observed.
    NoObservation,
}

/// Identity parsed from one expanded wildcard instance path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct InstanceIdentity {
    /// Instance name without the duplicate index suffix.
    pub(super) name: String,
    /// Optional parent instance name.
    pub(super) parent: Option<String>,
    /// Duplicate instance index used by PDH to distinguish equal names.
    pub(super) index: u32,
}

/// One exact or expanded counter point in a collection.
#[derive(Debug, Clone, PartialEq)]
pub(super) struct SamplePoint {
    /// Index of the configured counter that defines metric metadata.
    pub(super) counter_index: usize,
    /// Exact configured path or concrete expanded wildcard path.
    pub(super) path: Arc<str>,
    /// Parsed identity for an expanded wildcard instance.
    pub(super) instance: Option<Arc<InstanceIdentity>>,
    /// Whether to emit the Go-compatible `instance` attribute for this point.
    pub(super) emit_instance_attribute: bool,
    /// Time of this point's observation.
    pub(super) timestamp_unix_nano: i64,
    /// Start of this point's cumulative sequence, when applicable.
    pub(super) start_time_unix_nano: Option<i64>,
    /// Ready value or expected omission.
    pub(super) value: SampleValue,
}

/// One admitted native source eligible to own an output series.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SampleMember {
    /// Index of the configured counter that defines metric metadata.
    pub(super) counter_index: usize,
    /// Exact configured path or concrete expanded wildcard path.
    pub(super) path: Arc<str>,
    /// Parsed identity for an expanded or exact instance path.
    pub(super) instance: Option<Arc<InstanceIdentity>>,
    /// Whether to emit the Go-compatible `instance` attribute for this member.
    pub(super) emit_instance_attribute: bool,
}

/// One counter-local failure omitted from an otherwise successful collection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct SampleFailure {
    /// Index of the configured exact or wildcard path.
    pub(super) counter_index: usize,
    /// Concrete native path, or the configured template for query-wide failures.
    pub(super) path: Arc<str>,
    /// Low-cardinality failure category.
    pub(super) reason: &'static str,
    /// Native status or calculation detail without expanded instance identity.
    pub(super) error: String,
    /// Whether this entry exists only to provide per-mapping log context.
    pub(super) diagnostic_only: bool,
}

/// One explicit wildcard expansion limit reached during discovery.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) struct ExpansionOverflow {
    /// Index of the configured wildcard path.
    pub(super) counter_index: usize,
    /// Low-cardinality limit category.
    pub(super) reason: &'static str,
    /// Number of concrete paths discovered before this limit was applied.
    pub(super) discovered: usize,
    /// Number of concrete paths retained after this limit was applied.
    pub(super) retained: usize,
    /// Number of concrete paths omitted by this limit.
    pub(super) omitted: usize,
    /// Number omitted by the per-pattern member limit.
    pub(super) per_wildcard_omitted: usize,
    /// Additional omissions caused by the receiver-wide entry limit.
    pub(super) total_omitted: usize,
    /// Up to three concrete paths omitted by this admission limit.
    pub(super) examples: Vec<Arc<str>>,
}

/// Bounded operational changes observed while producing one sample.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(super) struct SampleDiagnostics {
    /// Whether at least one scheduled query reached native collection for this scrape.
    pub(super) any_query_collected: bool,
    /// Whether the query is awaiting recovery or has no installed counters yet.
    pub(super) query_pending: bool,
    /// Number of currently active expanded wildcard counters.
    pub(super) active_expanded_counters: usize,
    /// Number of wildcard discovery refreshes attempted.
    pub(super) discovery_refreshes: u64,
    /// Number of wildcard discovery attempts that failed.
    pub(super) discovery_failures: u64,
    /// Number of newly active expanded instances.
    pub(super) instances_added: u64,
    /// Number of expanded instances removed.
    pub(super) instances_removed: u64,
    /// Number of expanded instances omitted by configured limits.
    pub(super) instances_omitted_over_limit: u64,
    /// Number of counter-add attempts that failed.
    pub(super) counter_add_failures: u64,
    /// Number of deferred counter or query-collection retry attempts.
    pub(super) retry_attempts: u64,
    /// Number of deferred counter or query-collection retries that recovered.
    pub(super) retry_recoveries: u64,
    /// Number of worker-owned query rebuilds attempted.
    pub(super) query_rebuild_attempts: u64,
    /// Number of worker-owned query rebuilds that recovered.
    pub(super) query_rebuild_recoveries: u64,
    /// Number of independently warming points omitted.
    pub(super) warmup_omissions: u64,
}

#[derive(Debug, Clone, Default)]
pub(super) struct PendingMetadata {
    pub(super) diagnostics: SampleDiagnostics,
    pub(super) failures: Vec<SampleFailure>,
    pub(super) overflows: Vec<ExpansionOverflow>,
    /// Current cleanup identities; None means this delta contains no cleanup snapshot.
    pub(super) pending_cleanup: Option<Vec<(usize, Arc<str>)>>,
}

impl PendingMetadata {
    pub(super) fn merge(&mut self, delta: Self) {
        if delta.pending_cleanup.is_some() {
            self.pending_cleanup = delta.pending_cleanup;
        }
        merge_diagnostics(&mut self.diagnostics, delta.diagnostics);
        merge_latest_by_key(&mut self.failures, delta.failures, |failure| {
            (
                failure.counter_index,
                failure.reason,
                Arc::clone(&failure.path),
            )
        });
        merge_latest_by_key(&mut self.overflows, delta.overflows, |overflow| {
            MetadataKey::new(overflow.counter_index, overflow.reason)
        });
    }

    pub(super) fn merge_older(&mut self, older: Self) {
        if self.pending_cleanup.is_none() {
            self.pending_cleanup = older.pending_cleanup;
        }
        merge_diagnostics(&mut self.diagnostics, older.diagnostics);
        merge_older_by_key(&mut self.failures, older.failures, |failure| {
            (
                failure.counter_index,
                failure.reason,
                Arc::clone(&failure.path),
            )
        });
        merge_older_by_key(&mut self.overflows, older.overflows, |overflow| {
            MetadataKey::new(overflow.counter_index, overflow.reason)
        });
    }
}

fn linear_merge_is_cheap(current: usize, incoming: usize) -> bool {
    current.saturating_add(incoming).saturating_mul(incoming) <= METADATA_INDEX_WORK_THRESHOLD
}

fn merge_latest_by_key<T, K: Eq + std::hash::Hash>(
    current: &mut Vec<T>,
    latest: Vec<T>,
    key: impl Fn(&T) -> K,
) {
    if latest.is_empty() {
        return;
    }
    if linear_merge_is_cheap(current.len(), latest.len()) {
        merge_latest_linearly(current, latest, key);
        return;
    }
    let mut indices = metadata_indices(current, &key);
    for item in latest {
        let item_key = key(&item);
        if let Some(index) = indices.get(&item_key).copied() {
            current[index] = item;
        } else {
            let index = current.len();
            current.push(item);
            let _ = indices.insert(item_key, index);
        }
    }
}

fn merge_older_by_key<T, K: Eq + std::hash::Hash>(
    current: &mut Vec<T>,
    older: Vec<T>,
    key: impl Fn(&T) -> K,
) {
    if older.is_empty() {
        return;
    }
    if linear_merge_is_cheap(current.len(), older.len()) {
        merge_older_linearly(current, older, key);
        return;
    }
    let mut indices = metadata_indices(current, &key);
    for item in older {
        let item_key = key(&item);
        if let std::collections::hash_map::Entry::Vacant(entry) = indices.entry(item_key) {
            let index = current.len();
            current.push(item);
            let _ = entry.insert(index);
        }
    }
}

fn metadata_indices<T, K: Eq + std::hash::Hash>(
    current: &[T],
    key: &impl Fn(&T) -> K,
) -> HashMap<K, usize> {
    current
        .iter()
        .enumerate()
        .map(|(index, item)| (key(item), index))
        .collect()
}

fn merge_latest_linearly<T, K: Eq>(current: &mut Vec<T>, latest: Vec<T>, key: impl Fn(&T) -> K) {
    for item in latest {
        if let Some(existing) = current
            .iter_mut()
            .find(|existing| key(existing) == key(&item))
        {
            *existing = item;
        } else {
            current.push(item);
        }
    }
}

fn merge_older_linearly<T, K: Eq>(current: &mut Vec<T>, older: Vec<T>, key: impl Fn(&T) -> K) {
    for item in older {
        if !current.iter().any(|existing| key(existing) == key(&item)) {
            current.push(item);
        }
    }
}

/// A successful point-in-time PDH reading, independent of Windows handles.
#[derive(Debug, Clone)]
pub(super) struct Sample {
    /// Collection time as nanoseconds since the Unix epoch.
    pub(super) timestamp_unix_nano: i64,
    /// Admitted sources used to resolve stable output ownership before point filtering.
    pub(super) members: Vec<SampleMember>,
    /// Exact and expanded points keyed by configured counter and concrete path.
    pub(super) points: Vec<SamplePoint>,
    /// Diagnostics and bounded counter-local evidence for this collection.
    pub(super) metadata: PendingMetadata,
}

impl Sample {
    pub(super) fn empty(timestamp_unix_nano: i64) -> Self {
        Self {
            timestamp_unix_nano,
            members: Vec::new(),
            points: Vec::new(),
            metadata: PendingMetadata::default(),
        }
    }
}

pub(super) fn merge_diagnostics(target: &mut SampleDiagnostics, delta: SampleDiagnostics) {
    target.any_query_collected |= delta.any_query_collected;
    target.discovery_refreshes += delta.discovery_refreshes;
    target.discovery_failures += delta.discovery_failures;
    target.instances_added += delta.instances_added;
    target.instances_removed += delta.instances_removed;
    target.instances_omitted_over_limit += delta.instances_omitted_over_limit;
    target.counter_add_failures += delta.counter_add_failures;
    target.retry_attempts += delta.retry_attempts;
    target.retry_recoveries += delta.retry_recoveries;
    target.query_rebuild_attempts += delta.query_rebuild_attempts;
    target.query_rebuild_recoveries += delta.query_rebuild_recoveries;
    target.warmup_omissions += delta.warmup_omissions;
}

/// Scales an integer by a power of ten.
/// Zero scale preserves the integer.
/// Nonzero scales return a double; magnitudes above 2^53 may round during conversion.
/// Powers outside the configuration's supported range return an error.
pub(super) fn scale_integer(value: i64, power10: i32) -> Result<Number, ScaleError> {
    if power10 == 0 {
        return Ok(Number::Integer(value));
    }
    scale_double(value as f64, power10)
}

/// Scales a calculated value by a power of ten without clamping.
/// Uses normal f64 rounding and accepts subnormals and signed zero.
/// Non-finite inputs or results and unsupported powers return an error.
pub(super) fn scale_double(value: f64, power10: i32) -> Result<Number, ScaleError> {
    if !value.is_finite() {
        return Err(ScaleError::NonFinite);
    }
    // The supported powers of ten are exact in f64, so this cast does not round.
    let factor = scale_factor(power10)? as f64;
    let scaled = if power10 < 0 {
        value / factor
    } else {
        value * factor
    };
    if !scaled.is_finite() {
        return Err(ScaleError::NonFinite);
    }
    Ok(Number::Double(scaled))
}

/// Returns 10 raised to the absolute value of a supported power.
fn scale_factor(power10: i32) -> Result<i64, ScaleError> {
    if !(MIN_SCALE_POWER10..=MAX_SCALE_POWER10).contains(&power10) {
        return Err(ScaleError::InvalidScale);
    }
    // Keep the overflow check in case the supported range is widened.
    10_i64
        .checked_pow(power10.unsigned_abs())
        .ok_or(ScaleError::InvalidScale)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn failure_with_reason(
        counter_index: usize,
        reason: &'static str,
        detail: &str,
    ) -> SampleFailure {
        SampleFailure {
            counter_index,
            path: Arc::from(format!("counter-{counter_index}")),
            reason,
            error: detail.to_owned(),
            diagnostic_only: false,
        }
    }

    fn failure(counter_index: usize, detail: &str) -> SampleFailure {
        failure_with_reason(counter_index, "n12", detail)
    }

    fn failure_with_path(counter_index: usize, path: &str, detail: &str) -> SampleFailure {
        SampleFailure {
            counter_index,
            path: Arc::from(path),
            reason: "counter CStatus",
            error: detail.to_owned(),
            diagnostic_only: false,
        }
    }

    fn metadata(size: usize, detail: &str) -> PendingMetadata {
        PendingMetadata {
            failures: (0..size).map(|index| failure(index, detail)).collect(),
            ..Default::default()
        }
    }

    /// Scenario: Cleanup snapshots cross metadata-only deltas, recovery handoffs, and successful removal.
    /// Guarantees: Deltas retain the outstanding episode, while a newer empty snapshot clears it without resurrection by older evidence.
    #[test]
    fn cleanup_snapshot_merge_preserves_freshness() {
        let pending = PendingMetadata {
            pending_cleanup: Some(vec![(0, Arc::from(r"\Process(worker)\Private Bytes"))]),
            ..Default::default()
        };
        let mut current = PendingMetadata::default();
        current.merge_older(pending.clone());
        current.merge(PendingMetadata::default());
        assert_eq!(current.pending_cleanup, pending.pending_cleanup);
        current.merge(PendingMetadata {
            pending_cleanup: Some(Vec::new()),
            ..Default::default()
        });
        current.merge_older(pending);
        assert_eq!(current.pending_cleanup, Some(Vec::new()));
    }

    /// Scenario: A high-cardinality merge updates every existing key, repeats one key, and appends one new key.
    /// Guarantees: Indexed merging preserves first-position ordering while the newest evidence wins, keeping diagnostic output stable and current.
    #[test]
    fn metadata_merge_preserves_order_and_latest_values() {
        let mut current = metadata(256, "current");
        let mut latest = metadata(256, "latest");
        latest.failures.push(failure(0, "final"));
        latest.failures.push(failure(256, "new"));

        current.merge(latest);

        assert_eq!(current.failures.len(), 257);
        assert_eq!(current.failures[0].counter_index, 0);
        assert_eq!(current.failures[0].error, "final");
        assert_eq!(current.failures[255].counter_index, 255);
        assert_eq!(current.failures[255].error, "latest");
        assert_eq!(current.failures[256].counter_index, 256);
        assert_eq!(current.failures[256].error, "new");
    }

    /// Scenario: Newer and older batches each repeat one key while the destination is empty.
    /// Guarantees: Initial merges retain the established last-newer and first-older rules so startup evidence stays deterministic.
    #[test]
    fn empty_metadata_merge_deduplicates_by_freshness() {
        let mut latest_target = PendingMetadata::default();
        latest_target.merge(PendingMetadata {
            failures: vec![failure(0, "first"), failure(0, "last")],
            ..Default::default()
        });
        assert_eq!(latest_target.failures.len(), 1);
        assert_eq!(latest_target.failures[0].error, "last");

        let mut older_target = PendingMetadata::default();
        older_target.merge_older(PendingMetadata {
            failures: vec![failure(0, "first"), failure(0, "last")],
            ..Default::default()
        });
        assert_eq!(older_target.failures.len(), 1);
        assert_eq!(older_target.failures[0].error, "first");
    }

    /// Scenario: An empty destination receives thousands of distinct failures and one repeated key.
    /// Guarantees: Both freshness policies preserve order and duplicate semantics with linear key work as the destination grows.
    #[test]
    fn large_initial_metadata_merges_bound_key_work() {
        for latest in [false, true] {
            let calls = std::cell::Cell::new(0);
            let mut current = Vec::new();
            let mut incoming = (0..4_096)
                .map(|index| failure(index, "first"))
                .collect::<Vec<_>>();
            incoming.push(failure(0, "last"));
            let incoming_count = incoming.len();
            let key = |item: &SampleFailure| {
                calls.set(calls.get() + 1);
                MetadataKey::new(item.counter_index, item.reason)
            };
            if latest {
                merge_latest_by_key(&mut current, incoming, key);
            } else {
                merge_older_by_key(&mut current, incoming, key);
            }
            assert_eq!(current.len(), 4_096);
            assert_eq!(current[0].error, if latest { "last" } else { "first" });
            assert_eq!(current[4_095].counter_index, 4_095);
            assert!(
                calls.get() <= 2 * incoming_count,
                "{} key evaluations for {incoming_count} incoming failures",
                calls.get()
            );
        }
    }

    /// Scenario: Two concrete wildcard members report the same failure class under one mapping.
    /// Guarantees: Metadata merging retains both failed values independently of query merge order.
    #[test]
    fn metadata_merge_keys_failures_by_concrete_path() {
        let first = failure_with_path(0, r"\Process(first)\Value", "first");
        let second = failure_with_path(0, r"\Process(second)\Value", "second");
        for failures in [
            vec![first.clone(), second.clone()],
            vec![second.clone(), first.clone()],
        ] {
            let mut metadata = PendingMetadata::default();
            metadata.merge(PendingMetadata {
                failures,
                ..Default::default()
            });
            assert_eq!(metadata.failures.len(), 2);
            assert!(metadata.failures.contains(&first));
            assert!(metadata.failures.contains(&second));
        }
    }

    /// Scenario: High-cardinality pending metadata overlaps current evidence and repeats a previously unseen older key.
    /// Guarantees: Recovery cannot replace newer diagnostics, and the first retained older detail remains deterministic for operators.
    #[test]
    fn older_metadata_never_overwrites_current_values() {
        let mut current = metadata(256, "current");
        let mut older = metadata(256, "older");
        older.failures.push(failure(256, "first older"));
        older.failures.push(failure(256, "second older"));

        current.merge_older(older);

        assert_eq!(current.failures.len(), 257);
        assert!(
            current.failures[..256]
                .iter()
                .all(|failure| failure.error == "current")
        );
        assert_eq!(current.failures[256].counter_index, 256);
        assert_eq!(current.failures[256].error, "first older");
    }

    /// Scenario: Metadata contains a sparse, pathological counter index.
    /// Guarantees: Index storage remains proportional to evidence count rather than the largest configured-counter index.
    #[test]
    fn metadata_index_bounds_sparse_storage() {
        let mut current = (0..256)
            .map(|index| failure(index, "current"))
            .collect::<Vec<_>>();
        current[0].counter_index = usize::MAX;
        let incoming = (0..256).map(|index| failure(index, "incoming")).collect();
        merge_older_by_key(&mut current, incoming, |failure| {
            MetadataKey::new(failure.counter_index, failure.reason)
        });
        assert_eq!(current.len(), 257);
        assert_eq!(current[0].counter_index, usize::MAX);
        assert_eq!(current[256].error, "incoming");
    }

    /// Scenario: Every configured index has two failure reasons while older evidence partially overlaps and partially extends current metadata.
    /// Guarantees: Indexed merge_older matches the linear freshness and ordering semantics for the full (counter_index, reason) key.
    #[test]
    fn metadata_index_matches_linear_merge_for_two_reasons_per_index() {
        let reasons = ["prime", "read"];
        let mut indexed = (0..256)
            .flat_map(|counter_index| {
                reasons.map(|reason| failure_with_reason(counter_index, reason, "current"))
            })
            .collect::<Vec<_>>();
        let older = (128..384)
            .flat_map(|counter_index| {
                reasons.map(|reason| failure_with_reason(counter_index, reason, "older"))
            })
            .collect::<Vec<_>>();
        let mut linear = indexed.clone();

        merge_older_by_key(&mut indexed, older.clone(), |failure| {
            MetadataKey::new(failure.counter_index, failure.reason)
        });
        merge_older_linearly(&mut linear, older, |failure| {
            MetadataKey::new(failure.counter_index, failure.reason)
        });

        assert_eq!(indexed, linear);
        assert_eq!(indexed.len(), 768);
    }

    /// Scenario: Integer values use zero, positive, negative and boundary decimal scales.
    /// Guarantees: Zero scale preserves signed integers; every nonzero scale returns a double.
    #[test]
    fn scales_direct_values_by_decimal_power() {
        assert_eq!(
            scale_integer(i64::MAX, 0).unwrap(),
            Number::Integer(i64::MAX)
        );
        assert_eq!(
            scale_integer(i64::MIN, 0).unwrap(),
            Number::Integer(i64::MIN)
        );
        assert_eq!(scale_integer(42, 3).unwrap(), Number::Double(42_000.0));
        assert_eq!(scale_integer(-42, 3).unwrap(), Number::Double(-42_000.0));
        assert_eq!(scale_integer(9, 18).unwrap(), Number::Double(9e18));
        assert_eq!(scale_integer(-9, 18).unwrap(), Number::Double(-9e18));
        assert_eq!(scale_integer(0, 18).unwrap(), Number::Double(0.0));
        assert_eq!(scale_integer(42, -1).unwrap(), Number::Double(4.2));
        assert_eq!(scale_integer(-42, -1).unwrap(), Number::Double(-4.2));
        assert_eq!(scale_integer(3, -1).unwrap(), Number::Double(0.3));
        assert_eq!(scale_integer(7, -1).unwrap(), Number::Double(0.7));
        assert_eq!(
            scale_integer(86_580_518_913, -6).unwrap(),
            Number::Double(86_580.518_913)
        );
        assert_eq!(
            scale_integer(1_000_000_000_000_000_000, -18).unwrap(),
            Number::Double(1.0)
        );
        assert_eq!(scale_integer(0, -18).unwrap(), Number::Double(0.0));
    }

    /// Scenario: Integer extrema are scaled at both supported exponent limits.
    /// Guarantees: Large scaled values remain finite doubles rather than overflowing or being omitted.
    #[test]
    fn scales_integer_extremes_without_overflow() {
        assert_eq!(scale_integer(10, 18).unwrap(), Number::Double(1e19));
        assert_eq!(
            scale_integer(i64::MAX, 18).unwrap(),
            Number::Double(9.223_372_036_854_776e36)
        );
        assert_eq!(
            scale_integer(i64::MIN, 18).unwrap(),
            Number::Double(-9.223_372_036_854_776e36)
        );
        assert_eq!(
            scale_integer(i64::MAX, -18).unwrap(),
            Number::Double(9.223_372_036_854_776)
        );
        assert_eq!(
            scale_integer(i64::MIN, -18).unwrap(),
            Number::Double(-9.223_372_036_854_776)
        );
    }

    /// Scenario: Calculated values include percentages above 100, signed fractions and small magnitudes.
    /// Guarantees: Scaling preserves these values without clamping, integer coercion or discarding subnormals.
    #[test]
    fn scales_calculated_values_without_clamping() {
        assert_eq!(scale_double(250.0, 0).unwrap(), Number::Double(250.0));
        assert_eq!(scale_double(250.0, -2).unwrap(), Number::Double(2.5));
        assert_eq!(scale_double(-12.5, -1).unwrap(), Number::Double(-1.25));
        assert_eq!(scale_double(1.25, 1).unwrap(), Number::Double(12.5));
        assert_eq!(scale_double(1.0, 18).unwrap(), Number::Double(1e18));
        assert_eq!(scale_double(1.0, -18).unwrap(), Number::Double(1e-18));
        assert_eq!(scale_double(0.0, -18).unwrap(), Number::Double(0.0));
        assert_eq!(scale_double(1e-300, -18).unwrap(), Number::Double(1e-318));
        assert_eq!(scale_double(-1e-300, -18).unwrap(), Number::Double(-1e-318));
    }

    /// Scenario: Small calculated values round to zero, or a signed zero is scaled in either direction.
    /// Guarantees: Zero results retain the input's sign.
    #[test]
    fn preserves_signed_zero_when_scaling_underflows() {
        for value in [f64::MIN_POSITIVE, -f64::MIN_POSITIVE] {
            let Number::Double(scaled) = scale_double(value, -18).unwrap() else {
                panic!("calculated values must remain doubles");
            };
            assert_eq!(scaled, 0.0);
            assert_eq!(scaled.is_sign_negative(), value.is_sign_negative());
        }
        for value in [0.0_f64, -0.0_f64] {
            for power10 in [-18, 0, 18] {
                let Number::Double(scaled) = scale_double(value, power10).unwrap() else {
                    panic!("calculated values must remain doubles");
                };
                assert_eq!(scaled, 0.0);
                assert_eq!(scaled.is_sign_negative(), value.is_sign_negative());
            }
        }
    }

    /// Scenario: A calculated value is non-finite or scaling overflows the finite f64 range.
    /// Guarantees: Invalid observations return a bounded error rather than clamping or emitting infinities.
    #[test]
    fn rejects_non_finite_double_scaling() {
        for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
            for power10 in [-18, 0, 18] {
                assert_eq!(scale_double(value, power10), Err(ScaleError::NonFinite));
            }
        }
        assert_eq!(scale_double(f64::MAX, 18), Err(ScaleError::NonFinite));
        assert_eq!(scale_double(-f64::MAX, 18), Err(ScaleError::NonFinite));
    }

    /// Scenario: A scaling helper receives an exponent outside the configuration's supported range.
    /// Guarantees: Invalid powers return explicit errors without overflow, panic or non-finite output.
    #[test]
    fn rejects_unsupported_scale_powers() {
        for power10 in [i32::MIN, -19, 19, i32::MAX] {
            assert_eq!(
                scale_integer(i64::MAX, power10),
                Err(ScaleError::InvalidScale)
            );
            assert_eq!(scale_double(1.0, power10), Err(ScaleError::InvalidScale));
        }
    }
}
