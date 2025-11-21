// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module tests batching.rs logic.

use crate::otap::batching::make_output_batches;
use crate::proto::OtlpProtoMessage;
use crate::testing::equiv::assert_equivalent;
use crate::testing::fixtures::DataGenerator;
use crate::testing::round_trip::otap_to_otlp;
use crate::testing::round_trip::otlp_to_otap;
use std::num::NonZeroU64;

/// Generic test function for batching across all signal types.
fn test_batching(
    inputs_otlp: impl Iterator<Item = OtlpProtoMessage>,
    max_output_batch: Option<NonZeroU64>,
) {
    // Clone the inputs for later equivalence checking.
    let inputs_otlp: Vec<_> = inputs_otlp.collect();
    let inputs_otap: Vec<_> = inputs_otlp.iter().map(otlp_to_otap).collect();
    let signal_type = inputs_otap
        .first()
        .expect("at least one input")
        .signal_type();

    let outputs_otlp: Vec<_> = make_output_batches(signal_type, max_output_batch, inputs_otap)
        .expect("batching should succeed")
        .iter()
        .map(otap_to_otlp)
        .collect();

    // Assert batch_length <= max_output_batch
    if let Some(max_batch) = max_output_batch {
        for (i, output) in outputs_otlp.iter().enumerate() {
            let batch_len = output.batch_length();

            // Not empty.
            assert_ne!(batch_len, 0usize);

            // Note that the following assertion would fail if any
            // individual Metric contains more than max_batch.  The
            // current test fixtures used here do not have no more
            // than three data points, hence we test with limit 3..5.
            //
            // TODO: add testing for oversize-metric data. relax this
            // test helper to identify individual metric point counts
            // and refine the assertion. If metrics, presently: if
            // batch_len() exceeds the limit, there must be exactly
            // one metric (i.e., oversize metrics must be singletons).
            assert!(
                batch_len <= max_batch.get() as usize,
                "batch {} length {} exceeds limit {}",
                i,
                batch_len,
                max_batch
            );
        }
    }

    // Check OTLP equivalence
    assert_equivalent(&inputs_otlp, &outputs_otlp);
}

// Note: the tests below are quite simple, all they do is repeat
// similar data N times, but with a very fixed subset of the OTLP
// model: no attributes, no scope, only repetition with timestam
// variation.
//
// We envision extending this to more synthetic and corner-case data
// in a future PR.

#[test]
fn test_simple_batch_logs() {
    for input_count in 1..=20 {
        for max_output_batch in 3..=5 {
            // TODO: This 1 (limit) is not used for logs, fix.
            let mut datagen = DataGenerator::new(1);
            test_batching(
                (0..input_count).map(|_| datagen.generate_logs().into()),
                Some(NonZeroU64::new(max_output_batch).unwrap()),
            );
        }
    }
}

#[test]
fn test_simple_batch_traces() {
    for input_count in 1..=20 {
        for max_output_batch in 3..=5 {
            // TODO: This 1 (limit) is not used for metrics, fix.
            let mut datagen = DataGenerator::new(1);
            test_batching(
                (0..input_count).map(|_| datagen.generate_traces().into()),
                Some(NonZeroU64::new(max_output_batch).unwrap()),
            );
        }
    }
}

/// TODO(#1334): This test is currently broken. Unify() has some issues.
#[test]
#[ignore]
fn test_simple_batch_metrics() {
    for input_count in 1..=20 {
        // TODO: Note that changing 3..=5 to 3..=15 breaks the test
        for max_output_batch in 3..=5 {
            eprintln!("in/max {input_count}, {max_output_batch}");
            for point_count in 1..=10 {
                let mut datagen = DataGenerator::new(point_count);
                test_batching(
                    (0..input_count).map(|_| datagen.generate_metrics().into()),
                    Some(NonZeroU64::new(max_output_batch).unwrap()),
                );
            }
        }
    }
}
