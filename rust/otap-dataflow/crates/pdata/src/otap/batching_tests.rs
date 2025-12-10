// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! This module tests batching.rs logic.

use crate::otap::batching::make_output_batches;
use crate::proto::OtlpProtoMessage;
use crate::testing::equiv::assert_equivalent;
use crate::testing::fixtures::{DataGenerator, MetricsConfig};
use crate::testing::round_trip::otap_to_otlp;
use crate::testing::round_trip::otlp_to_otap;
use std::num::NonZeroU64;

/// A test case for batching metrics with specific configurations
#[derive(Debug, Clone)]
struct MetricsBatchingTestCase {
    name: String,
    config: MetricsConfig,
    max_output_batch: u64,
    input_count: usize,
}

/// Generate deterministic test cases
fn generate_metrics_batching_test_cases() -> Vec<MetricsBatchingTestCase> {
    let mut cases = Vec::new();

    let mut add_case = |name: &str, config: MetricsConfig, max_batch: u64, inputs: usize| {
        cases.push(MetricsBatchingTestCase {
            name: format!("{:04}_{}", cases.len(), name),
            config,
            max_output_batch: max_batch,
            input_count: inputs,
        });
    };

    // Basic single-type metrics with varying point counts
    for limit in [5, 10, 20, 50] {
        for point_count in [1, 2, 3, 5, 8, 10, 15, 20] {
            if point_count <= limit as usize {
                add_case(
                    &format!("single_gauge_{}_pts_limit_{}", point_count, limit),
                    MetricsConfig::new().with_gauges(vec![point_count]),
                    limit as u64,
                    5,
                );
            }
        }
    }

    // Boundary conditions: at, under, and over limit
    for limit in [5, 10, 20] {
        add_case(
            &format!("at_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![limit as usize]),
            limit as u64,
            3,
        );
        add_case(
            &format!("under_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![limit as usize - 1]),
            limit as u64,
            3,
        );
        add_case(
            &format!("over_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![limit as usize + 1]),
            limit as u64,
            3,
        );
    }

    // Oversized metrics (should appear as singletons)
    for limit in [5, 10, 20] {
        add_case(
            &format!("oversized_2x_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![limit as usize * 2]),
            limit as u64,
            2,
        );
        add_case(
            &format!("oversized_5x_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![limit as usize * 5]),
            limit as u64,
            1,
        );
    }

    // Multiple metrics of same type with varying sizes
    for limit in [10, 20, 50] {
        add_case(
            &format!("multi_gauges_mixed_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![1, 2, 3, 5, 8]),
            limit as u64,
            4,
        );
        add_case(
            &format!("multi_sums_mixed_limit_{}", limit),
            MetricsConfig::new().with_sums(vec![2, 4, 6, 8]),
            limit as u64,
            3,
        );
    }

    // All four metric types present - temporarily disabled due to schema compatibility issues
    // TODO: Investigate and fix schema merging for mixed metric types
    // for limit in [10, 20, 50] {
    //     add_case(
    //         &format!("all_types_small_limit_{}", limit),
    //         MetricsConfig::new()
    //             .with_gauges(vec![2, 3])
    //             .with_sums(vec![2])
    //             .with_histograms(vec![1, 2])
    //             .with_summaries(vec![1]),
    //         limit as u64,
    //         3,
    //     );
    // }

    // Mixed sizes: some fit, some oversized
    for limit in [10, 15, 20] {
        add_case(
            &format!("mixed_small_and_oversized_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![2, limit as usize * 2, 3, limit as usize + 5]),
            limit as u64,
            2,
        );
    }

    // Many small metrics that should pack efficiently
    for limit in [20, 50, 100] {
        add_case(
            &format!("many_small_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![1; 15]),
            limit as u64,
            5,
        );
        add_case(
            &format!("many_medium_limit_{}", limit),
            MetricsConfig::new().with_sums(vec![3; 10]),
            limit as u64,
            4,
        );
    }

    // Complex multi-type scenarios - temporarily disabled
    // TODO: Investigate and fix schema merging for mixed metric types
    // for limit in [25, 50, 100] {
    //     add_case(
    //         &format!("complex_all_types_limit_{}", limit),
    //         MetricsConfig::new()
    //             .with_gauges(vec![3, 5, 7, 2])
    //             .with_sums(vec![4, 6, 3])
    //             .with_histograms(vec![2, 8])
    //             .with_summaries(vec![5, 3, 2]),
    //         limit as u64,
    //         6,
    //     );
    // }

    // Stress test: many metrics of varying sizes - single type only
    for limit in [50, 100, 200] {
        let gauge_counts: Vec<usize> = (1..=10).collect();
        add_case(
            &format!("stress_gauges_limit_{}", limit),
            MetricsConfig::new().with_gauges(gauge_counts.clone()),
            limit,
            10,
        );
        let sum_counts: Vec<usize> = (2..=8).step_by(2).collect();
        add_case(
            &format!("stress_sums_limit_{}", limit),
            MetricsConfig::new().with_sums(sum_counts),
            limit,
            10,
        );
        add_case(
            &format!("stress_histograms_limit_{}", limit),
            MetricsConfig::new().with_histograms(vec![3, 7, 12, 5, 9]),
            limit,
            8,
        );
    }

    // Precise boundary testing
    for limit in [10, 20] {
        // Metrics that sum to exactly the limit
        add_case(
            &format!("sum_to_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![3, 3, 4]),
            limit,
            3,
        );
        // Metrics that sum to limit + 1
        add_case(
            &format!("sum_to_limit_plus_1_{}", limit),
            MetricsConfig::new().with_gauges(vec![3, 3, 5]),
            limit,
            2,
        );
    }

    // Single input with multiple metrics
    for limit in [15, 30] {
        add_case(
            &format!("single_input_multi_metrics_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![2, 4, 6, 3, 7]),
            limit,
            1,
        );
    }

    // Multiple inputs with single metric each
    for limit in [10, 25] {
        add_case(
            &format!("multi_inputs_single_metric_limit_{}", limit),
            MetricsConfig::new().with_gauges(vec![5]),
            limit,
            8,
        );
    }

    // Test with varying attributes enabled - temporarily disabled
    // TODO: Investigate parent_id column type issue with varying attributes
    // for limit in [20, 50] {
    //     add_case(
    //         &format!("with_attrs_gauges_limit_{}", limit),
    //         MetricsConfig::new()
    //             .with_gauges(vec![3, 5])
    //             .with_varying_attributes(true),
    //         limit as u64,
    //         4,
    //     );
    //     add_case(
    //         &format!("with_attrs_sums_limit_{}", limit),
    //         MetricsConfig::new()
    //             .with_sums(vec![4, 2])
    //             .with_varying_attributes(true),
    //         limit as u64,
    //         4,
    //     );
    // }

    //
    // A bunch of gauges!
    for i in 0..1000 {
        let limit = 10 + (i % 10) as u64 * 10;
        let points = vec![(i % 7) + 1, (i % 5) + 2];
        add_case(
            &format!("{:04}_filler", i),
            MetricsConfig::new().with_gauges(points),
            limit,
            3usize + (i % 11),
        );
    }

    cases
}

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

            assert_ne!(batch_len, 0usize);

            // For metrics: validate the that oversized metrics are singletons
            if let OtlpProtoMessage::Metrics(metrics_data) = output
                && batch_len > max_batch.get() as usize
            {
                let metric_count = count_metrics_in_batch(metrics_data);
                assert_eq!(metric_count, 1);
            } else {
                assert!(
                    batch_len <= max_batch.get() as usize,
                    "batch {} length {} exceeds limit {}",
                    i,
                    batch_len,
                    max_batch
                );
            }
        }
    }

    // Check OTLP equivalence
    assert_equivalent(&inputs_otlp, &outputs_otlp);
}

/// Count the total number of metrics in a MetricsData batch
fn count_metrics_in_batch(
    metrics_data: &crate::proto::opentelemetry::metrics::v1::MetricsData,
) -> usize {
    metrics_data
        .resource_metrics
        .iter()
        .flat_map(|rm| &rm.scope_metrics)
        .map(|sm| sm.metrics.len())
        .sum()
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

#[test]
fn test_simple_batch_metrics() {
    for input_count in 1..=20 {
        for max_output_batch in 3..=15 {
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

#[test]
fn test_comprehensive_batch_metrics() {
    let test_cases = generate_metrics_batching_test_cases();

    for (idx, test_case) in test_cases.iter().enumerate() {
        let mut datagen = DataGenerator::with_metrics_config(test_case.config.clone());

        let inputs: Vec<_> = (0..test_case.input_count)
            .map(|_| datagen.generate_metrics_from_config().into())
            .collect();

        // Run the test, capturing any panics with better error messages
        let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            test_batching(
                inputs.into_iter(),
                Some(NonZeroU64::new(test_case.max_output_batch).unwrap()),
            );
        }));

        if let Err(e) = result {
            eprintln!(
                "Test case {} failed: {}\n  Config: {:?}\n  Max batch: {}\n  Inputs: {}\n  Total points: {}",
                idx,
                test_case.name,
                test_case.config,
                test_case.max_output_batch,
                test_case.input_count,
                test_case.config.total_points()
            );
            std::panic::resume_unwind(e);
        }
    }
}
