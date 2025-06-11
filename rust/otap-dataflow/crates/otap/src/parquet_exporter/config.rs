// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

/// Configuration of parquet exporter
pub struct Config {
    /// The base URI for where the parquet files should be written
    pub base_uri: String,

    /// Configuration for how to compute partitions from the dataset
    pub partitioning_strategies: Option<Vec<PartitioningStrategy>>,

    /// Options for the writer
    pub writer_options: Option<WriterOptions>,
}

#[derive(Clone)]
pub struct WriterOptions {
    /// Target number of rows in one parquet file. The writer will flush automatically any files
    /// that attain greater than this number of rows. If this is None, the writer won't flush
    /// automatically and some external component will need to periodically and/or eventually call
    /// the `WriterManager.flush_all` method.
    ///
    /// This is currently approximate. The writer does not currently split batches across multiple
    /// files if the cutoff for the target rows happens to be in the middle of a batch.
    pub target_rows_per_file: Option<usize>,
}

impl Default for WriterOptions {
    fn default() -> Self {
        Self {
            target_rows_per_file: Some(100_000_000),
        }
    }
}

/// Configuration options for how the parquet files should be partitioned
pub enum PartitioningStrategy {
    /// compute partition values from schema metadata keys
    SchemaMetadata(Vec<String>),
}
