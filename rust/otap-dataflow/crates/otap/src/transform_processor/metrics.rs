// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the TransformProcessor node.

use otap_df_telemetry::instrument::DeltaCounter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the TransformProcessor node.
#[metric_set(name = "transform.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    /// PData messages consumed by this processor.
    #[metric(unit = "{msg}")]
    pub msgs_consumed: DeltaCounter<u64>,

    /// PData messages forwarded by this processor.
    #[metric(unit = "{msg}")]
    pub msgs_forwarded: DeltaCounter<u64>,

    /// Number of messages successfully transformed.
    #[metric(unit = "{msg}")]
    pub msgs_transformed: DeltaCounter<u64>,

    /// Number of failed transform attempts.
    #[metric(unit = "{msg}")]
    pub msgs_transform_failed: DeltaCounter<u64>,
}
