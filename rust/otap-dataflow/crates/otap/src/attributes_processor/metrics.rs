// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the AttributesProcessor node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

/// Metrics for the AttributesProcessor node.
#[metric_set(name = "attributes.processor.metrics")]
#[derive(Debug, Default, Clone)]
pub struct AttributesProcessorMetrics {
    /// PData messages consumed by this processor.
    #[metric(unit = "{msg}")]
    pub msgs_consumed: Counter<u64>,

    /// PData messages forwarded by this processor.
    #[metric(unit = "{msg}")]
    pub msgs_forwarded: Counter<u64>,

    /// Number of failed transform attempts.
    #[metric(unit = "{op}")]
    pub transform_failed: Counter<u64>,

    /// Total number of attribute entries actually renamed.
    #[metric(unit = "{attr}")]
    pub renamed_entries: Counter<u64>,

    /// Total number of attribute entries actually deleted.
    #[metric(unit = "{attr}")]
    pub deleted_entries: Counter<u64>,

    /// Number of times transforms were applied to signal-level payloads.
    #[metric(unit = "{apply}")]
    pub domains_signal: Counter<u64>,

    /// Number of times transforms were applied to resource-level payloads.
    #[metric(unit = "{apply}")]
    pub domains_resource: Counter<u64>,

    /// Number of times transforms were applied to scope-level payloads.
    #[metric(unit = "{apply}")]
    pub domains_scope: Counter<u64>,
}
