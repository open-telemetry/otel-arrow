// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use std::time::Duration;

use serde::Deserialize;

/// Configuration of parquet exporter
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct Config {
    /// The base URI for where the parquet files should be written
    pub storage: otel_arrow_dfe_otap::object_store::StorageType,

    /// Optional object_store retry settings for cloud-backed storage.
    pub retry: Option<otel_arrow_dfe_otap::object_store::RetryOptions>,

    /// Configuration for how to compute partitions from the dataset
    pub partitioning_strategies: Option<Vec<PartitioningStrategy>>,

    /// Options for the writer
    pub writer_options: Option<WriterOptions>,

    /// How IDs are (re)generated before writing (see [`IdGenerationStrategy`])
    #[serde(default)]
    pub id_generation: IdGenerationStrategy,
}

/// Strategy for converting per-batch OTAP ids into ids that are meaningful in storage
#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum IdGenerationStrategy {
    /// sequential ids, unique only within a `_part_id` partition (the historical behavior)
    #[default]
    PartitionSequence,
    /// content-hash ids: attribute-set/series hashes that are globally stable and give the
    /// attrs tables dimension-table semantics (deduplicated per UTC day). Widens id columns
    /// to u64.
    ContentHash,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct WriterOptions {
    /// Target number of rows in one parquet file. The writer will flush automatically any files
    /// that attain greater than this number of rows. If this is `None`, the writer won't flush
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
    /// Note that files may actually be buffered for slightly longer than this value. For more
    /// details see [`Self::flush_age_check_interval`]
    #[serde(default)]
    #[serde(with = "humantime_serde")]
    pub flush_when_older_than: Option<Duration>,
}

impl Default for WriterOptions {
    fn default() -> Self {
        Self {
            flush_when_older_than: None,
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

    /// compute partition values from the wall-clock time at which the batch is written
    /// (ingest time). Produces hive-style `date=YYYY-MM-DD` path segments (plus `hour=HH`
    /// for hourly granularity), which enables time-based partition pruning in query engines.
    ///
    /// Note this buckets by ingest time, not by the timestamps within the data. Data that
    /// arrives late will land in the partition for the time it was received, so readers
    /// filtering on event time should include a margin of adjacent partitions.
    #[serde(alias = "time_bucket")]
    TimeBucket(TimeBucketConfig),
}

/// Configuration for the [`PartitioningStrategy::TimeBucket`] partitioning strategy
#[derive(Debug, Deserialize, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct TimeBucketConfig {
    /// the granularity of the time buckets
    pub granularity: TimeBucketGranularity,
}

/// Granularity of the time bucket partitions
#[derive(Clone, Copy, Debug, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum TimeBucketGranularity {
    /// partition by day: `date=YYYY-MM-DD`
    Day,
    /// partition by hour: `date=YYYY-MM-DD/hour=HH`
    Hour,
}

#[cfg(test)]
mod test {
    use serde_json::json;

    use otel_arrow_dfe_otap::object_store::StorageType;

    use super::*;

    #[test]
    fn test_deserialize() {
        let json_cfg = json!({
            "storage": {
                "file": {
                  "base_uri": "s3://albert-bucket/parquet-files"
                }
            },
            "partitioning_strategies": [
                {
                    "schema_metadata": [ "_part_id" ]
                }
            ],
            "writer_options": {
            "flush_when_older_than": "300s",
            "target_rows_per_file": 1000000000
            }
        })
        .to_string();

        let config: Config = serde_json::from_str(&json_cfg).unwrap();
        let expected = Config {
            storage: StorageType::File {
                base_uri: "s3://albert-bucket/parquet-files".to_string(),
            },
            retry: None,
            partitioning_strategies: Some(vec![PartitioningStrategy::SchemaMetadata(vec![
                "_part_id".to_string(),
            ])]),
            id_generation: Default::default(),
            writer_options: Some(WriterOptions {
                flush_when_older_than: Some(Duration::from_secs(300)),
                target_rows_per_file: Some(1000000000),
            }),
        };
        assert_eq!(config, expected)
    }

    #[test]
    fn test_deserialize_time_bucket() {
        let json_cfg = json!({
            "storage": {
                "file": {
                  "base_uri": "s3://albert-bucket/parquet-files"
                }
            },
            "partitioning_strategies": [
                {
                    "time_bucket": { "granularity": "day" }
                },
                {
                    "schema_metadata": [ "_part_id" ]
                }
            ]
        })
        .to_string();

        let config: Config = serde_json::from_str(&json_cfg).unwrap();
        assert_eq!(
            config.partitioning_strategies,
            Some(vec![
                PartitioningStrategy::TimeBucket(TimeBucketConfig {
                    granularity: TimeBucketGranularity::Day,
                }),
                PartitioningStrategy::SchemaMetadata(vec!["_part_id".to_string()]),
            ])
        );

        // hour granularity also deserializes
        let json_cfg = json!({
            "storage": { "file": { "base_uri": "/tmp/parquet-files" } },
            "partitioning_strategies": [
                { "time_bucket": { "granularity": "hour" } }
            ]
        })
        .to_string();
        let config: Config = serde_json::from_str(&json_cfg).unwrap();
        assert_eq!(
            config.partitioning_strategies,
            Some(vec![PartitioningStrategy::TimeBucket(TimeBucketConfig {
                granularity: TimeBucketGranularity::Hour,
            })])
        );
    }

    #[test]
    fn test_deserialize_explicit_retry() {
        let json_cfg = json!({
            "storage": {
                "file": {
                  "base_uri": "/tmp/parquet-files"
                }
            },
            "retry": {
                "max_retries": 10,
                "init_backoff": "200ms",
                "max_backoff": "30s",
                "backoff_base": 2.0,
                "retry_timeout": "2min"
            }
        })
        .to_string();

        let config: Config = serde_json::from_str(&json_cfg).unwrap();
        let retry = config.retry.unwrap();

        assert_eq!(retry.max_retries, 10);
        assert_eq!(retry.init_backoff, Duration::from_millis(200));
        assert_eq!(retry.max_backoff, Duration::from_secs(30));
        assert_eq!(retry.backoff_base, 2.0);
        assert_eq!(retry.retry_timeout, Duration::from_secs(120));
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
