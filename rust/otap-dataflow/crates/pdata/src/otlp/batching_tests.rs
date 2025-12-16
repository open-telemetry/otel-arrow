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

    let inputs_bytes: Vec<OtlpProtoBytes> = inputs_otlp.iter().map(otlp_message_to_bytes).collect();

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

/// Test that the batcher handles corrupted protobuf data
#[test]
fn test_corrupted_protobuf_handling() {
    use otap_df_config::SignalType;

    let mut datagen = DataGenerator::new(1);
    let logs1 = datagen.generate_logs();
    let logs2 = datagen.generate_logs();

    // Convert both to bytes
    let good_bytes1 = otlp_message_to_bytes(&logs1.clone().into());
    let good_bytes2 = otlp_message_to_bytes(&logs2.clone().into());
    let good_size = good_bytes1.byte_size() + good_bytes2.byte_size();

    // Create a third input that's corrupted
    let mut corrupted_bytes = Vec::new();
    // Create a malformed field: valid tag (field 1, wire type 2=LEN_DELIM)
    let garbage = vec![
        0x0A, // field 1, wire type 2 (LEN_DELIM)
        0xFF, 0xFF, 0xFF, 0xFF, 0x0F, // varint: huge length that will fail
    ];
    corrupted_bytes.extend_from_slice(&garbage);

    let corrupted_input = OtlpProtoBytes::new_from_bytes(SignalType::Logs, corrupted_bytes);
    let corrupted_size = corrupted_input.byte_size();

    let total_size = good_size + corrupted_size;

    // Batch with max_size between good_size and total_size
    // This should produce 2 outputs: good content batched together, then corrupt content
    let max_size = good_size + 2; // > good_size but < total_size

    let outputs = make_bytes_batches(
        SignalType::Logs,
        NonZeroU64::new(max_size as u64),
        vec![good_bytes1, good_bytes2, corrupted_input.clone()],
    )
    .expect("batching should succeed");

    // Should get 2 batches: good data together, then corrupt data
    assert_eq!(outputs.len(), 2);

    // First batch should contain the good data
    let first_size = outputs[0].byte_size();
    assert_eq!(first_size, good_size);

    // Second batch should contain the garbage
    let second_size = outputs[1].byte_size();
    assert_eq!(second_size, corrupted_size);
    assert_eq!(outputs[1].as_bytes(), garbage);

    // Total size should be preserved
    let total_output = first_size + second_size;
    assert_eq!(total_output, total_size);

    // First batch should decode successfully
    let first_decoded = otlp_bytes_to_message(outputs[0].clone());
    assert_eq!(first_decoded.num_items(), 6);

    // Verify first batch is equivalent to original good data
    let expected: Vec<OtlpProtoMessage> = vec![logs1.into(), logs2.into()];
    assert_equivalent(&expected, &[first_decoded]);
}
