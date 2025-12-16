// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module tests batching.rs logic.

use crate::otlp::OtlpProtoBytes;
use crate::otlp::batching::make_bytes_batches;
use crate::proto::OtlpProtoMessage;
use crate::testing::equiv::assert_equivalent;
use crate::testing::fixtures::DataGenerator;
use crate::testing::round_trip::otlp_bytes_to_message;
use crate::testing::round_trip::otlp_message_to_bytes;
use std::num::NonZeroU64;

/// Test bytes-based batching with various size limits
fn test_batching(inputs_otlp: impl Iterator<Item = OtlpProtoMessage>) {
    // Clone the inputs for later equivalence checking.
    let inputs_otlp: Vec<_> = inputs_otlp.collect();
    let signal_type = inputs_otlp.get(0).expect("ok").signal_type();

    let inputs_bytes: Vec<OtlpProtoBytes> = inputs_otlp
        .iter()
        .map(|m| otlp_message_to_bytes(m.clone()))
        .collect();

    let total_input_bytes: usize = inputs_bytes.iter().map(|b| b.byte_size()).sum();

    // Run a single equivalence test
    let test_config = |limit: Option<NonZeroU64>, label: &str| {
        let outputs = make_bytes_batches(signal_type, limit, inputs_bytes.clone()).expect("ok");
        let total: usize = outputs.iter().map(|b| b.byte_size()).sum();
        assert_eq!(total_input_bytes, total, "{}: byte count mismatch", label);

        // Convert outputs back to OtlpProtoMessage and verify equivalence
        let outputs_msgs: Vec<OtlpProtoMessage> =
            outputs.into_iter().map(otlp_bytes_to_message).collect();
        assert_equivalent(&inputs_otlp, &outputs_msgs);
    };

    // Run with no limit (worst case)
    test_config(None, "no limit");

    // Run with limit == actual size
    if total_input_bytes > 0 {
        test_config(
            Some(NonZeroU64::new(total_input_bytes as u64).unwrap()),
            "actual size",
        );
    }

    // Run with limit == actual_size * 0.1
    if total_input_bytes >= 10 {
        let limit_10pct = (total_input_bytes / 10).max(1);
        test_config(
            Some(NonZeroU64::new(limit_10pct as u64).unwrap()),
            "10% limit",
        );
    }

    // Run with limit == actual_size * 0.5
    if total_input_bytes >= 2 {
        let limit_50pct = (total_input_bytes / 2).max(1);
        test_config(
            Some(NonZeroU64::new(limit_50pct as u64).unwrap()),
            "50% limit",
        );
    }

    // Run with limit == 1 (worst case: should produce single-field batches)
    test_config(Some(NonZeroU64::new(1).unwrap()), "limit 1");
}

// Note: this test is similar to ../otap/batching_tests. We should
// consider a consolidation.

#[test]
fn test_simple_batch_logs() {
    for input_count in 1..=20 {
        let mut datagen = DataGenerator::new(1);
        test_batching((0..input_count).map(|_| datagen.generate_logs().into()));
    }
}

#[test]
fn test_simple_batch_traces() {
    for input_count in 1..=20 {
        let mut datagen = DataGenerator::new(1);
        test_batching((0..input_count).map(|_| datagen.generate_traces().into()));
    }
}

#[test]
fn test_simple_batch_metrics() {
    for input_count in 1..=20 {
        for point_count in 1..=10 {
            let mut datagen = DataGenerator::new(point_count);
            test_batching((0..input_count).map(|_| datagen.generate_metrics().into()));
        }
    }
}
