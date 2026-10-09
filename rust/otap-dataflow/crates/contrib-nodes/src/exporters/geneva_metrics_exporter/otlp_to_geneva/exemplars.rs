// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP exemplar selection for Geneva metrics.

use std::str::{self, Utf8Error};

use otel_arrow_dfe_pdata_views::views::common::{AnyValueView, AttributeView};
use otel_arrow_dfe_pdata_views::views::metrics::{ExemplarView, Value};

use super::super::encoder::MetricExemplar;
use super::super::encoder::exemplar::{
    MAX_EXEMPLAR_PAYLOAD_SIZE, encoded_exemplar_size_from_parts,
};

const MAX_EXEMPLAR_AGE_NANOS: u64 = 60_000_000_000;
const MAX_SAMPLING_BUCKET_COUNT: usize = 12;
const SAMPLING_BUCKET_COUNTS: [usize; 6] = [MAX_SAMPLING_BUCKET_COUNT, 10, 8, 6, 4, 2];

#[derive(Clone, Debug)]
struct ExemplarCandidate<E> {
    source: E,
    value: f64,
    encoded_size: usize,
}

pub(super) fn map_exemplars<I, E>(
    exemplars: I,
    data_point_time: u64,
) -> Result<Vec<MetricExemplar>, Utf8Error>
where
    I: IntoIterator<Item = E>,
    E: ExemplarView,
{
    let mut candidates = Vec::new();
    for exemplar in exemplars {
        if exemplar
            .time_unix_nano()
            .saturating_add(MAX_EXEMPLAR_AGE_NANOS)
            < data_point_time
        {
            continue;
        }
        if let Some(candidate) = exemplar_candidate(exemplar)? {
            candidates.push(candidate);
        }
    }
    retain_candidates_within_limits(&mut candidates);
    candidates
        .into_iter()
        .map(|candidate| map_exemplar(&candidate.source))
        .collect()
}

fn exemplar_candidate<E>(exemplar: E) -> Result<Option<ExemplarCandidate<E>>, Utf8Error>
where
    E: ExemplarView,
{
    validate_exemplar_attributes(&exemplar)?;
    let value = exemplar_value(&exemplar);
    let encoded_size = encoded_exemplar_size_from_parts(
        value,
        exemplar.time_unix_nano() != 0,
        exemplar.trace_id().is_some_and(id_is_nonzero),
        exemplar.span_id().is_some_and(id_is_nonzero),
        false,
        exemplar.filtered_attributes().map(|attribute| {
            let value_length = attribute
                .value()
                .map_or(0, |value| value.as_string().map_or(0, |value| value.len()));
            (attribute.key().len(), value_length)
        }),
    )
    .ok();
    Ok(encoded_size.map(|encoded_size| ExemplarCandidate {
        source: exemplar,
        value,
        encoded_size,
    }))
}

fn validate_exemplar_attributes<E>(exemplar: &E) -> Result<(), Utf8Error>
where
    E: ExemplarView,
{
    for attribute in exemplar.filtered_attributes() {
        let _ = str::from_utf8(attribute.key())?;
        if let Some(value) = attribute.value()
            && let Some(value) = value.as_string()
        {
            let _ = str::from_utf8(value)?;
        }
    }
    Ok(())
}

fn map_exemplar<E>(exemplar: &E) -> Result<MetricExemplar, Utf8Error>
where
    E: ExemplarView,
{
    let mut filtered_attributes = Vec::new();
    for attribute in exemplar.filtered_attributes() {
        let name = str::from_utf8(attribute.key())?.to_string();
        let value = match attribute.value() {
            Some(value) => match value.as_string() {
                Some(value) => str::from_utf8(value)?.to_string(),
                None => String::new(),
            },
            None => String::new(),
        };
        filtered_attributes.push((name, value));
    }
    Ok(MetricExemplar {
        value: exemplar_value(exemplar),
        time_unix_nano: Some(exemplar.time_unix_nano()),
        trace_id: nonzero_id(exemplar.trace_id()),
        span_id: nonzero_id(exemplar.span_id()),
        sample_count: None,
        filtered_attributes,
    })
}

fn nonzero_id<const N: usize>(identifier: Option<&[u8; N]>) -> Option<[u8; N]> {
    identifier
        .filter(|identifier| id_is_nonzero(identifier))
        .copied()
}

fn id_is_nonzero<const N: usize>(identifier: &[u8; N]) -> bool {
    identifier.iter().any(|byte| *byte != 0)
}

fn exemplar_value<E>(exemplar: &E) -> f64
where
    E: ExemplarView,
{
    match exemplar.value().unwrap_or(Value::Integer(0)) {
        Value::Double(value) => value,
        Value::Integer(value) => value as f64,
    }
}

fn retain_candidates_within_limits<E>(candidates: &mut Vec<ExemplarCandidate<E>>) {
    let mut payload_size = candidate_payload_size(candidates);
    if payload_size <= MAX_EXEMPLAR_PAYLOAD_SIZE {
        return;
    }

    let extrema = ExemplarExtrema::from_candidates(candidates);
    for bucket_count in SAMPLING_BUCKET_COUNTS {
        payload_size = retain_distribution_sample(candidates, bucket_count, extrema, payload_size);
        if payload_size <= MAX_EXEMPLAR_PAYLOAD_SIZE || candidates.len() <= 2 {
            break;
        }
    }
    if payload_size <= MAX_EXEMPLAR_PAYLOAD_SIZE {
        return;
    }

    let excess = payload_size - MAX_EXEMPLAR_PAYLOAD_SIZE;
    let excess = discard_candidate_bytes(candidates, excess, |candidate| {
        !extrema.contains(candidate.value)
    });
    if excess > 0 {
        let excess = discard_candidate_bytes(candidates, excess, |_| true);
        debug_assert_eq!(excess, 0);
    }
}

#[derive(Clone, Copy, Default)]
struct ExemplarExtrema {
    negative: Option<(f64, f64)>,
    positive: Option<(f64, f64)>,
}

impl ExemplarExtrema {
    fn from_candidates<E>(candidates: &[ExemplarCandidate<E>]) -> Self {
        let mut extrema = Self::default();
        for candidate in candidates {
            let value = candidate.value;
            let range = if value < 0.0 {
                &mut extrema.negative
            } else if value > 0.0 {
                &mut extrema.positive
            } else {
                continue;
            };
            let (minimum, maximum) = range.unwrap_or((value, value));
            *range = Some((minimum.min(value), maximum.max(value)));
        }
        extrema
    }

    fn contains(self, value: f64) -> bool {
        [self.negative, self.positive]
            .into_iter()
            .flatten()
            .any(|(minimum, maximum)| value == minimum || value == maximum)
    }
}

fn candidate_payload_size<E>(candidates: &[ExemplarCandidate<E>]) -> usize {
    candidates
        .iter()
        .map(|candidate| candidate.encoded_size)
        .sum()
}

fn discard_candidate_bytes<E>(
    candidates: &mut Vec<ExemplarCandidate<E>>,
    mut excess: usize,
    mut should_discard: impl FnMut(&ExemplarCandidate<E>) -> bool,
) -> usize {
    candidates.retain(|candidate| {
        if excess == 0 || !should_discard(candidate) {
            return true;
        }
        excess = excess.saturating_sub(candidate.encoded_size);
        false
    });
    excess
}

fn retain_distribution_sample<E>(
    candidates: &mut Vec<ExemplarCandidate<E>>,
    bucket_count: usize,
    extrema: ExemplarExtrema,
    mut payload_size: usize,
) -> usize {
    if extrema.negative.is_none() && extrema.positive.is_none() {
        return payload_size;
    }
    let mut positive_buckets = [false; MAX_SAMPLING_BUCKET_COUNT + 1];
    let mut negative_buckets = [false; MAX_SAMPLING_BUCKET_COUNT + 1];
    let mut zero_seen = false;
    let has_mixed_signs = extrema.negative.is_some() && extrema.positive.is_some();
    let single_range = extrema.positive.or(extrema.negative);
    let original_len = candidates.len();
    let mut current = 0;
    let mut active_len = original_len;

    for _ in 0..original_len {
        let value = candidates[current].value;
        let is_zero = value == 0.0;
        let decision = if is_zero {
            if zero_seen {
                SamplingDecision::Discard
            } else {
                zero_seen = true;
                SamplingDecision::Keep
            }
        } else if has_mixed_signs {
            if value > 0.0 {
                bucket_decision(
                    &mut positive_buckets,
                    exemplar_bucket(value, extrema.positive, bucket_count),
                )
            } else if value < 0.0 {
                bucket_decision(
                    &mut negative_buckets,
                    mixed_negative_exemplar_bucket(value, extrema.negative, bucket_count),
                )
            } else {
                // NaN: no comparison above matches, so discard rather than stall the loop.
                SamplingDecision::Discard
            }
        } else {
            let buckets = if extrema.positive.is_some() {
                &mut positive_buckets
            } else {
                &mut negative_buckets
            };
            bucket_decision(buckets, exemplar_bucket(value, single_range, bucket_count))
        };

        match decision {
            SamplingDecision::Keep => current += 1,
            SamplingDecision::Discard => {
                payload_size = payload_size.saturating_sub(candidates[current].encoded_size);
                active_len -= 1;
                candidates.swap(current, active_len);
            }
        }
        if !is_zero && payload_size <= MAX_EXEMPLAR_PAYLOAD_SIZE {
            break;
        }
    }
    candidates.truncate(active_len);
    payload_size
}

enum SamplingDecision {
    Keep,
    Discard,
}

fn bucket_decision(
    buckets: &mut [bool; MAX_SAMPLING_BUCKET_COUNT + 1],
    index: Option<usize>,
) -> SamplingDecision {
    match index {
        Some(index) if !buckets[index] => {
            buckets[index] = true;
            SamplingDecision::Keep
        }
        _ => SamplingDecision::Discard,
    }
}

fn exemplar_bucket(value: f64, range: Option<(f64, f64)>, bucket_count: usize) -> Option<usize> {
    let (minimum, maximum) = range?;
    if value == maximum {
        return Some(bucket_count);
    }
    if minimum == maximum {
        return Some(0);
    }
    let growth = (maximum / minimum).powf(1.0 / bucket_count as f64);
    let raw_index = (value / minimum).ln() / growth.ln();
    if !raw_index.is_finite() {
        return None;
    }
    let index = raw_index.floor() as isize;
    if index < 0 || index > bucket_count as isize {
        return None;
    }
    let index = index as usize;
    if (index == 0 && value != minimum) || (index == bucket_count && value != maximum) {
        return None;
    }
    Some(index)
}

fn mixed_negative_exemplar_bucket(
    value: f64,
    range: Option<(f64, f64)>,
    bucket_count: usize,
) -> Option<usize> {
    let (minimum, maximum) = range?;
    let raw_index = if value == minimum {
        bucket_count as f64
    } else {
        let growth = (maximum / minimum).powf(1.0 / bucket_count as f64);
        ((value / minimum).ln() / growth.ln()).floor()
    };
    if !raw_index.is_finite() {
        return None;
    }
    let mut index = raw_index as isize;
    if index < 0 {
        index = bucket_count as isize;
    }
    if index > bucket_count as isize {
        return None;
    }
    let index = index as usize;
    if (index == 0 && value != minimum) || (index == bucket_count && value != maximum) {
        return None;
    }
    Some(index)
}

#[cfg(test)]
mod tests {
    use otel_arrow_dfe_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue, any_value};
    use otel_arrow_dfe_pdata::proto::opentelemetry::metrics::v1::{
        Exemplar as OtlpExemplar, exemplar,
    };
    use otel_arrow_dfe_pdata::views::otlp::proto::metrics::{ExemplarIter, ObjExemplar};
    use otel_arrow_dfe_pdata::views::otlp::proto::wrappers::Wraps;

    use super::*;

    const DEFAULT_UNIX_SECONDS: u64 = 1_388_577_600;

    fn exemplar_candidate(exemplar: &OtlpExemplar) -> Option<ExemplarCandidate<ObjExemplar<'_>>> {
        super::exemplar_candidate(ObjExemplar::new(exemplar))
            .expect("test exemplar should contain valid UTF-8")
    }

    fn map_exemplar(exemplar: &OtlpExemplar) -> MetricExemplar {
        super::map_exemplar(&ObjExemplar::new(exemplar))
            .expect("test exemplar should contain valid UTF-8")
    }

    fn sampling_candidate(value: f64, source_index: usize) -> ExemplarCandidate<usize> {
        let encoded_size = encoded_exemplar_size_from_parts(
            value,
            true,
            true,
            true,
            false,
            std::iter::once((3, 5)),
        )
        .expect("sampling candidate should fit");
        ExemplarCandidate {
            source: source_index,
            value,
            encoded_size,
        }
    }

    fn string_attribute(key: &str, value: &str) -> KeyValue {
        KeyValue {
            key: key.to_string(),
            value: Some(AnyValue {
                value: Some(any_value::Value::StringValue(value.to_string())),
            }),
        }
    }

    /// Scenario: A large exemplar set spans a logarithmic positive range in forward and reverse order.
    /// Guarantees: Distribution sampling bounds the payload and retains both range extrema independently of input order.
    #[test]
    fn samples_candidates_across_the_value_distribution() {
        let values = (0..32).map(|power| 2_f64.powi(power)).collect::<Vec<_>>();

        for reverse in [false, true] {
            let mut candidates: Vec<_> = if reverse {
                values
                    .iter()
                    .rev()
                    .copied()
                    .enumerate()
                    .map(|(index, value)| sampling_candidate(value, index))
                    .collect()
            } else {
                values
                    .iter()
                    .copied()
                    .enumerate()
                    .map(|(index, value)| sampling_candidate(value, index))
                    .collect()
            };
            assert!(candidate_payload_size(&candidates) > MAX_EXEMPLAR_PAYLOAD_SIZE);

            retain_candidates_within_limits(&mut candidates);

            assert!(candidate_payload_size(&candidates) <= MAX_EXEMPLAR_PAYLOAD_SIZE);
            assert!(
                candidates
                    .iter()
                    .any(|candidate| candidate.value == values[0])
            );
            assert!(
                candidates
                    .iter()
                    .any(|candidate| candidate.value == values[values.len() - 1])
            );
            assert!(candidates.len() < values.len());
        }
    }

    /// Scenario: An oversized exemplar set contains equal positive values with distinct trace IDs.
    /// Guarantees: Tail replacement preserves the same retained exemplar identities and order.
    #[test]
    fn retains_duplicate_exemplars_up_to_payload_limit() {
        let mut candidates = (0..32)
            .map(|index| sampling_candidate(68.0, index))
            .collect::<Vec<_>>();
        let retained_count = MAX_EXEMPLAR_PAYLOAD_SIZE / candidates[0].encoded_size;
        let mut expected_indexes = vec![0, retained_count];
        expected_indexes.extend(2..retained_count);

        retain_candidates_within_limits(&mut candidates);

        assert_eq!(candidates.len(), retained_count);
        assert!(candidates.iter().all(|candidate| candidate.value == 68.0));
        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.source)
                .collect::<Vec<_>>(),
            expected_indexes
        );
        assert!(candidate_payload_size(&candidates) <= MAX_EXEMPLAR_PAYLOAD_SIZE);
    }

    /// Scenario: Mixed-sign sampling encounters the negative minimum and a duplicate negative maximum.
    /// Guarantees: Mixed-sign buckets and tail swaps select the same exemplar identities and order.
    #[test]
    fn matches_mixed_sign_tail_replacement() {
        let values = [
            -16.0, -1.0, -1.0, 1.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 1.0,
        ];
        let mut candidates = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| sampling_candidate(value, index))
            .collect::<Vec<_>>();

        retain_candidates_within_limits(&mut candidates);

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.source)
                .collect::<Vec<_>>(),
            vec![11, 1, 10, 3, 4, 5, 6, 7, 8, 9]
        );
        assert!(candidate_payload_size(&candidates) <= MAX_EXEMPLAR_PAYLOAD_SIZE);
    }

    /// Scenario: Removing a duplicate zero first brings an exemplar payload below the size limit.
    /// Guarantees: Tail processing reprocesses and removes the swapped duplicate exemplar before stopping.
    #[test]
    fn reprocesses_tail_after_zero_removal_reaches_limit() {
        let values = [1.0, 0.0, 0.0, 2.0, 4.0, 8.0, 16.0, 32.0, 64.0, 128.0, 1.0];
        let mut candidates = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| sampling_candidate(value, index))
            .collect::<Vec<_>>();

        retain_candidates_within_limits(&mut candidates);

        assert_eq!(
            candidates
                .iter()
                .map(|candidate| candidate.source)
                .collect::<Vec<_>>(),
            vec![0, 1, 9, 3, 4, 5, 6, 7, 8]
        );
        assert!(candidate_payload_size(&candidates) <= MAX_EXEMPLAR_PAYLOAD_SIZE);
    }

    /// Scenario: A mixed-sign exemplar set includes a NaN value, which cannot be bucketed into
    /// either the positive or negative distribution.
    /// Guarantees: The NaN candidate is discarded outright rather than stalling the sampling
    /// cursor, so later candidates are still evaluated and the payload is trimmed to the limit.
    #[test]
    fn discards_nan_values_in_mixed_sign_distributions() {
        // NaN is placed first so it is guaranteed to be evaluated before any early-exit check
        // can short-circuit the sampling loop once the payload already fits under the limit.
        let values = [
            f64::NAN,
            -16.0,
            -1.0,
            -1.0,
            2.0,
            4.0,
            8.0,
            16.0,
            32.0,
            64.0,
            128.0,
            1.0,
        ];
        let mut candidates = values
            .into_iter()
            .enumerate()
            .map(|(index, value)| sampling_candidate(value, index))
            .collect::<Vec<_>>();
        assert!(candidate_payload_size(&candidates) > MAX_EXEMPLAR_PAYLOAD_SIZE);

        retain_candidates_within_limits(&mut candidates);

        assert!(candidates.iter().all(|candidate| !candidate.value.is_nan()));
        assert!(candidate_payload_size(&candidates) <= MAX_EXEMPLAR_PAYLOAD_SIZE);
    }

    /// Scenario: Borrowed OTLP exemplar sizing covers string and non-string labels plus optional identifier fields.
    /// Guarantees: Candidate preflight matches the encoder size and rejects oversized exemplars before label cloning.
    #[test]
    fn preflights_exemplar_size_before_mapping() {
        let valid = OtlpExemplar {
            filtered_attributes: vec![
                string_attribute("key", "value"),
                KeyValue {
                    key: "empty".to_string(),
                    value: Some(AnyValue {
                        value: Some(any_value::Value::IntValue(7)),
                    }),
                },
            ],
            time_unix_nano: DEFAULT_UNIX_SECONDS * 1_000_000_000,
            span_id: vec![2; 8],
            trace_id: vec![1; 16],
            value: Some(exemplar::Value::AsDouble(1.5)),
        };
        let candidate = exemplar_candidate(&valid).expect("valid exemplar should fit");
        let mapped = map_exemplar(&valid);

        assert_eq!(
            candidate.encoded_size,
            super::super::super::encoder::exemplar::encoded_exemplar_size(&mapped)
                .expect("mapped exemplar should fit")
        );

        let oversized = OtlpExemplar {
            filtered_attributes: vec![string_attribute(&"k".repeat(193), "")],
            ..valid
        };
        assert!(exemplar_candidate(&oversized).is_none());
    }

    /// Scenario: Exemplar input contains one stale entry, one oversized entry, and one valid entry.
    /// Guarantees: Single-pass candidate collection retains only the valid exemplar without changing filtering behavior.
    #[test]
    fn collects_only_valid_exemplar_candidates() {
        let data_point_time = DEFAULT_UNIX_SECONDS * 1_000_000_000;
        let stale = OtlpExemplar {
            time_unix_nano: data_point_time - MAX_EXEMPLAR_AGE_NANOS - 1,
            value: Some(exemplar::Value::AsDouble(1.0)),
            ..Default::default()
        };
        let oversized = OtlpExemplar {
            filtered_attributes: vec![string_attribute(&"k".repeat(193), "")],
            time_unix_nano: data_point_time,
            value: Some(exemplar::Value::AsDouble(2.0)),
            ..Default::default()
        };
        let valid = OtlpExemplar {
            filtered_attributes: vec![string_attribute("key", "value")],
            time_unix_nano: data_point_time,
            value: Some(exemplar::Value::AsDouble(3.0)),
            ..Default::default()
        };

        let mapped = map_exemplars(
            ExemplarIter::new([stale, oversized, valid].iter()),
            data_point_time,
        )
        .expect("test exemplars should contain valid UTF-8");

        assert_eq!(mapped.len(), 1);
        assert_eq!(mapped[0].value, 3.0);
        assert_eq!(
            mapped[0].filtered_attributes,
            vec![("key".to_string(), "value".to_string())]
        );
    }

    /// Scenario: A zero-only exemplar set exceeds the payload limit by one minimal exemplar.
    /// Guarantees: Sampling skips logarithmic buckets and fallback trimming retains the maximum 102 zero exemplars.
    #[test]
    fn retains_zero_exemplars_up_to_payload_limit() {
        let candidate = ExemplarCandidate {
            source: (),
            value: 0.0,
            encoded_size: encoded_exemplar_size_from_parts(
                0.0,
                false,
                false,
                false,
                false,
                std::iter::empty(),
            )
            .expect("minimal exemplar should fit"),
        };
        let mut candidates = vec![candidate; 103];

        retain_candidates_within_limits(&mut candidates);

        assert_eq!(candidates.len(), 102);
        assert_eq!(candidate_payload_size(&candidates), 510);
    }
}
