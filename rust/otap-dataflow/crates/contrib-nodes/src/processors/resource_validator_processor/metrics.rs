// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Resource Validator Processor

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics collected by the Resource Validator Processor.
///
/// Tracks both batch-level and item-level counts. Validation is pass/fail for
/// the entire batch — if any resource fails, the whole batch is NACKed. Item
/// counts capture the magnitude of data loss on rejection.
#[metric_set(name = "resource_validator.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct ResourceValidatorMetrics {
    /// Number of batches that passed validation
    #[metric(unit = "{batch}")]
    pub batches_accepted: Counter<u64>,

    /// Number of batches rejected due to missing required attribute
    #[metric(unit = "{batch}")]
    pub batches_rejected_missing: Counter<u64>,

    /// Number of batches rejected due to value not in allowed list
    #[metric(unit = "{batch}")]
    pub batches_rejected_not_allowed: Counter<u64>,

    /// Number of batches rejected due to invalid attribute type (not a string)
    #[metric(unit = "{batch}")]
    pub batches_rejected_invalid_type: Counter<u64>,

    /// Number of batches rejected due to internal conversion error
    #[metric(unit = "{batch}")]
    pub batches_rejected_conversion_error: Counter<u64>,

    /// Number of telemetry items accepted
    #[metric(unit = "{item}")]
    pub items_accepted: Counter<u64>,

    /// Number of telemetry items rejected
    #[metric(unit = "{item}")]
    pub items_rejected: Counter<u64>,
}
