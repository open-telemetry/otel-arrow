// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the Resource Validator Processor

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics collected by the Resource Validator Processor
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
}
