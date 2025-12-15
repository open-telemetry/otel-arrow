// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the AttributesProcessor node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the QueryEngineProcessor node.
#[metric_set(name = "queryengine.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    /// PData messages consumed by this processor.
    #[metric(unit = "{msg}")]
    pub msgs_consumed: Counter<u64>,

    /// PData messages forwarded by this processor.
    #[metric(unit = "{msg}")]
    pub msgs_forwarded: Counter<u64>,

    /// Number of failed transform attempts.
    #[metric(unit = "{op}")]
    pub transform_failed: Counter<u64>,
}
