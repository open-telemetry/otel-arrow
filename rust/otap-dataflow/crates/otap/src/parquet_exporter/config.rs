// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use serde::Deserialize;

/// Configuration of parquet exporter
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The base URI for where the parquet files should be written
    pub base_uri: String,

    /// Configuration for how to compute partitions from the dataset
    pub partitioning_strategies: Option<Vec<PartitioningStrategy>>,

    /// Options for the writer
    // TODO fix the default for this?
    pub writer_options: Option<WriterOptions>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct WriterOptions {
    /// Target number of rows in one parquet file. The writer will flush automatically any files
    /// that attain greater than this number of rows. If this is `None``, the writer won't flush
    /// automatically when a given file size is reached (in this case, it is best to set
    /// [`Self::flush_when_older_than`]).
    ///
    /// This is currently approximate. The writer does not currently split batches across multiple
    /// files if the cutoff for the target rows happens to be in the middle of a batch.
    ///
    /// Default = 100 million rows.
    pub target_rows_per_file: Option<usize>,

    /// If this is set, the exporter will flush files whose first batch is older than this
    /// interval. This can be used to configure the writer to flush the file before the target rows
    /// per file has been reached, which can be useful in the case that there is a desire to have
    /// the data become visible earlier. Note, setting this to too small of an interval could
    /// result in the creation of many small files, which can negatively impact read performance.
    ///
    /// Note that files may actually be buffered for slightly longer than this value. See how this
    /// configuration value works in concert with [`Self::flush_age_check_interval`] and
    #[serde(with = "humantime_serde")]
    pub flush_when_older_than: Option<Duration>,

    /// Period at which the age of unflushed files are checked to determine if they should be
    /// flushed. Files whose age is older than [`Self::flush_when_older_than`] will be flushed on
    /// this interval (if that value is `None`, this config value is ignored).
    ///
    /// Note: setting this to a smaller value may cause unflushed files to be to be buffered for
    /// less time beyond the age at which they should be flushed. However, using an extremely short
    /// interval can cause extra overhead in the pipeline.
    ///
    /// Default = "5s" (5 seconds)
    // TODO need to set the default here somehow -- we don't want to let users configure this as null?
    #[serde(with = "humantime_serde")]
    pub flush_age_check_interval: Option<Duration>,

    /// If a parent writer is older than the configured `flush_when_older_than` threshold, but 
    /// it contains rows for a batch with child rows in a writer whose age is less than the 
    /// threshold, setting this to true will cause the writer to automatically flush the child 
    /// writers when the parent is flushed. Otherwise, the parent writer will not be flushed until
    /// the child batch is also flushed.
    /// 
    /// Setting this to `true` can cause the parent batches to flush earlier, but risks resulting 
    /// in undersized child batches (which can negatively impact read performance).
    /// 
    /// Default = false
    // TODO need to set the default
    pub force_flush_children_on_parent_timeout: bool
}

impl Default for WriterOptions {
    fn default() -> Self {
        Self {
            flush_when_older_than: None,
            flush_age_check_interval: None,
            force_flush_children_on_parent_timeout: false,
            target_rows_per_file: Some(100_000_000),
        }
    }
}

/// Configuration options for how the parquet files should be partitioned
#[derive(Debug, Deserialize, PartialEq)]
pub enum PartitioningStrategy {
    /// compute partition values from schema metadata keys
    #[serde(alias = "schema_metadata")]
    SchemaMetadata(Vec<String>),
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_deserialize() {
        let json_cfg = "{
            \"base_uri\": \"s3://albert-bucket/parquet-files\",
            \"partitioning_strategies\": [
                {
                    \"schema_metadata\": [ \"_part_id\" ]
                }
            ],
            \"writer_options\": {
                \"target_rows_per_file\": 1000000000
            }
        }";

        let config: Config = serde_json::from_str(json_cfg).unwrap();
        let expected = Config {
            base_uri: "s3://albert-bucket/parquet-files".to_string(),
            partitioning_strategies: Some(vec![PartitioningStrategy::SchemaMetadata(vec![
                "_part_id".to_string(),
            ])]),
            writer_options: Some(WriterOptions {
                flush_age_check_interval: None,
                flush_when_older_than: None,
                target_rows_per_file: Some(1000000000),
                // TODO fix this default
                force_flush_children_on_parent_timeout: false,
            }),
        };
        assert_eq!(config, expected)
    }

    #[test]
    fn test_max_flush_interval_config() {
        todo!()
    }

    #[test]
    fn test_deserialize_error_unknown_fields() {
        // this has a mistake in it where target_rows_per_file should be
        // nested w/in writer_options:
        let json_cfg = "{
            \"base_uri\": \"s3://albert-bucket/parquet-files\",
            \"partitioning_strategies\": [
                {
                    \"schema_metadata\": [ \"_part_id\" ]
                }
            ],
            \"target_rows_per_file\": 1000000000
        }";
        assert!(serde_json::from_str::<Config>(json_cfg).is_err())
    }
}
