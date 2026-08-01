// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics for the PartitionProcessor node.

use otap_df_telemetry::instrument::Counter;
use otap_df_telemetry_macros::metric_set;

#[metric_set(name = "processor.partition")]
#[derive(Debug, Default, Clone)]
pub struct Metrics {
    /// Number of incoming batches that were successfully partitioned
    #[metrics(unit = "{batch}")]
    pub partition_operations_succeeded: Counter<u64>,

    /// Number of incoming batches that failed to be partitioned
    #[metrics(unit = "{batch}")]
    pub partition_operations_failed: Counter<u64>,
}
