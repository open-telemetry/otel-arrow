// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Metrics specific to the Parquet exporter IO lifecycle.

use otap_df_telemetry::instrument::DeltaCounter;
use otap_df_telemetry_macros::metric_set;

/// Parquet exporter IO metrics.
/// Grouped under `otap.exporter.parquet`.
#[metric_set(name = "otap.exporter.parquet")]
#[derive(Debug, Default, Clone)]
pub struct ParquetExporterMetrics {
    /// Number of Parquet files created (across all payload types and partitions).
    #[metric(unit = "{file}")]
    pub files_created: DeltaCounter<u64>,

    /// Number of Parquet files successfully closed (flushed and visible to readers).
    #[metric(unit = "{file}")]
    pub files_closed: DeltaCounter<u64>,

    /// Total number of rows written into Parquet writers (appended, not necessarily flushed yet).
    #[metric(unit = "{row}")]
    pub rows_written: DeltaCounter<u64>,

    /// Files scheduled for flush due to reaching target rows per file.
    #[metric(unit = "{file}")]
    pub flush_scheduled_max_rows: DeltaCounter<u64>,

    /// Files scheduled for flush due to exceeding max age threshold.
    #[metric(unit = "{file}")]
    pub flush_scheduled_max_age: DeltaCounter<u64>,
}
