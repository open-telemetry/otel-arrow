// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the AttributesProcessor node.

use otap_df_telemetry::instrument::DeltaCounter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the AttributesProcessor node.
#[metric_set(name = "attributes.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct AttributesProcessorMetrics {
    /// PData messages consumed by this processor.
    #[metric(unit = "{msg}")]
    pub msgs_consumed: DeltaCounter<u64>,

    /// PData messages forwarded by this processor.
    #[metric(unit = "{msg}")]
    pub msgs_forwarded: DeltaCounter<u64>,

    /// Number of failed transform attempts.
    #[metric(unit = "{op}")]
    pub transform_failed: DeltaCounter<u64>,

    /// Total number of attribute entries actually renamed.
    #[metric(unit = "{attr}")]
    pub renamed_entries: DeltaCounter<u64>,

    /// Total number of attribute entries actually deleted.
    #[metric(unit = "{attr}")]
    pub deleted_entries: DeltaCounter<u64>,

    /// Number of times transforms were applied to signal-level payloads.
    #[metric(unit = "{apply}")]
    pub domains_signal: DeltaCounter<u64>,

    /// Number of times transforms were applied to resource-level payloads.
    #[metric(unit = "{apply}")]
    pub domains_resource: DeltaCounter<u64>,

    /// Number of times transforms were applied to scope-level payloads.
    #[metric(unit = "{apply}")]
    pub domains_scope: DeltaCounter<u64>,
}
