// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! OTLP exemplar selection for Geneva metrics.

use super::super::encoder::MetricExemplar;
use super::super::encoder::exemplar::{MAX_EXEMPLAR_PAYLOAD_SIZE, encoded_exemplar_size};

const MAX_SAMPLING_BUCKET_COUNT: usize = 12;
const SAMPLING_BUCKET_COUNTS: [usize; 6] = [MAX_SAMPLING_BUCKET_COUNT, 10, 8, 6, 4, 2];

pub(super) fn retain_exemplars_within_limits(exemplars: &mut Vec<MetricExemplar>) {
    exemplars.retain(|exemplar| encoded_exemplar_size(exemplar).is_ok());
    let mut payload_size = exemplar_payload_size(exemplars);
    if payload_size <= MAX_EXEMPLAR_PAYLOAD_SIZE {
        return;
    }

    let extrema = ExemplarExtrema::from_exemplars(exemplars);
    for bucket_count in SAMPLING_BUCKET_COUNTS {
        retain_distribution_sample(exemplars, bucket_count, extrema);
        payload_size = exemplar_payload_size(exemplars);
        if payload_size <= MAX_EXEMPLAR_PAYLOAD_SIZE || exemplars.len() <= 2 {
            break;
        }
    }
    if payload_size <= MAX_EXEMPLAR_PAYLOAD_SIZE {
        return;
    }

    let excess = payload_size - MAX_EXEMPLAR_PAYLOAD_SIZE;
    let excess = discard_exemplar_bytes(exemplars, excess, |exemplar| {
        !extrema.contains(exemplar.value)
    });
    if excess > 0 {
        let excess = discard_exemplar_bytes(exemplars, excess, |_| true);
        debug_assert_eq!(excess, 0);
    }
}

#[derive(Clone, Copy, Default)]
struct ExemplarExtrema {
    negative: Option<(f64, f64)>,
    positive: Option<(f64, f64)>,
}

impl ExemplarExtrema {
    fn from_exemplars(exemplars: &[MetricExemplar]) -> Self {
        let mut extrema = Self::default();
        for exemplar in exemplars {
            let value = exemplar.value;
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

fn exemplar_payload_size(exemplars: &[MetricExemplar]) -> usize {
    exemplars.iter().map(validated_exemplar_size).sum()
}

fn discard_exemplar_bytes(
    exemplars: &mut Vec<MetricExemplar>,
    mut excess: usize,
    mut should_discard: impl FnMut(&MetricExemplar) -> bool,
) -> usize {
    exemplars.retain(|exemplar| {
        if excess == 0 || !should_discard(exemplar) {
            return true;
        }
        excess = excess.saturating_sub(validated_exemplar_size(exemplar));
        false
    });
    excess
}

fn validated_exemplar_size(exemplar: &MetricExemplar) -> usize {
    encoded_exemplar_size(exemplar).expect("exemplars were size-validated before sampling")
}

fn retain_distribution_sample(
    exemplars: &mut Vec<MetricExemplar>,
    bucket_count: usize,
    extrema: ExemplarExtrema,
) {
    if extrema.negative.is_none() && extrema.positive.is_none() {
        return;
    }
    let mut positive_buckets = [false; MAX_SAMPLING_BUCKET_COUNT + 1];
    let mut negative_buckets = [false; MAX_SAMPLING_BUCKET_COUNT + 1];
    let mut zero_seen = false;
    let mut nan_seen = false;

    exemplars.retain(|exemplar| {
        let value = exemplar.value;
        if value == 0.0 {
            if zero_seen {
                return false;
            }
            zero_seen = true;
            return true;
        }
        let selection = if value > 0.0 {
            exemplar_bucket(value, extrema.positive, bucket_count)
                .map(|index| (&mut positive_buckets, index))
        } else if value < 0.0 {
            exemplar_bucket(value, extrema.negative, bucket_count)
                .map(|index| (&mut negative_buckets, index))
        } else {
            if nan_seen {
                return false;
            }
            nan_seen = true;
            return true;
        };
        let Some((buckets, index)) = selection else {
            return false;
        };
        if buckets[index] {
            return false;
        }
        buckets[index] = true;
        true
    });
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

#[cfg(test)]
mod tests {
    use super::*;

    const DEFAULT_UNIX_SECONDS: u64 = 1_388_577_600;

    fn sampling_exemplar(value: f64) -> MetricExemplar {
        MetricExemplar {
            value,
            time_unix_nano: Some(DEFAULT_UNIX_SECONDS * 1_000_000_000),
            trace_id: Some([1; 16]),
            span_id: Some([2; 8]),
            sample_count: None,
            filtered_attributes: vec![("key".to_string(), "value".to_string())],
        }
    }

    /// Scenario: A large exemplar set spans a logarithmic positive range in forward and reverse order.
    /// Guarantees: Distribution sampling bounds the payload and retains both range extrema independently of input order.
    #[test]
    fn samples_exemplars_across_the_value_distribution() {
        let values = (0..32).map(|power| 2_f64.powi(power)).collect::<Vec<_>>();

        for reverse in [false, true] {
            let mut exemplars: Vec<_> = if reverse {
                values
                    .iter()
                    .rev()
                    .copied()
                    .map(sampling_exemplar)
                    .collect()
            } else {
                values.iter().copied().map(sampling_exemplar).collect()
            };
            assert!(exemplar_payload_size(&exemplars) > MAX_EXEMPLAR_PAYLOAD_SIZE);

            retain_exemplars_within_limits(&mut exemplars);

            assert!(exemplar_payload_size(&exemplars) <= MAX_EXEMPLAR_PAYLOAD_SIZE);
            assert!(exemplars.iter().any(|exemplar| exemplar.value == values[0]));
            assert!(
                exemplars
                    .iter()
                    .any(|exemplar| exemplar.value == values[values.len() - 1])
            );
            assert!(exemplars.len() < values.len());
        }
    }

    /// Scenario: An oversized exemplar set contains only duplicate positive values.
    /// Guarantees: ME-style bucket sampling retains one representative instead of an arbitrary payload prefix.
    #[test]
    fn samples_duplicate_exemplar_values_once() {
        let mut exemplars = vec![sampling_exemplar(68.0); 32];

        retain_exemplars_within_limits(&mut exemplars);

        assert_eq!(exemplars.len(), 1);
        assert_eq!(exemplars[0].value, 68.0);
    }

    /// Scenario: A zero-only exemplar set exceeds the payload limit by one minimal exemplar.
    /// Guarantees: Sampling skips logarithmic buckets and fallback trimming retains the maximum 102 zero exemplars.
    #[test]
    fn retains_zero_exemplars_up_to_payload_limit() {
        let exemplar = MetricExemplar {
            value: 0.0,
            time_unix_nano: None,
            trace_id: None,
            span_id: None,
            sample_count: None,
            filtered_attributes: Vec::new(),
        };
        let mut exemplars = vec![exemplar; 103];

        retain_exemplars_within_limits(&mut exemplars);

        assert_eq!(exemplars.len(), 102);
        assert_eq!(exemplar_payload_size(&exemplars), 510);
    }
}
