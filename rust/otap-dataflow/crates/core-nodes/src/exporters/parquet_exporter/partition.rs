// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use chrono::{DateTime, Utc};
use otel_arrow_dfe_pdata::{
    proto::opentelemetry::arrow::v1::ArrowPayloadType, schema::get_schema_metadata,
};

use super::config::{PartitioningStrategy, TimeBucketConfig, TimeBucketGranularity};
use super::records::OtapParquetRecords;

pub enum PartitionAttributeValue {
    String(String),
}

pub struct PartitionAttribute {
    pub key: String,
    pub value: PartitionAttributeValue,
}

impl std::fmt::Display for PartitionAttributeValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PartitionAttributeValue::String(value) => write!(f, "{value}"),
        }
    }
}

pub struct Partition {
    pub otap_batch: OtapParquetRecords,
    pub attributes: Option<Vec<PartitionAttribute>>,
}

#[must_use]
pub fn partition(
    otap_batch: OtapParquetRecords,
    strategies: &[PartitioningStrategy],
) -> Vec<Partition> {
    let mut attributes = vec![];

    // This is a fairly simply implementation for now. This will be refactored a lot when
    // we add different partitioning strategies (e.g. partitioning by date bucket)
    for strategy in strategies {
        match strategy {
            PartitioningStrategy::SchemaMetadata(metadata_keys) => attributes.append(
                &mut static_partitions_from_schema_metadata(&otap_batch, metadata_keys),
            ),
            PartitioningStrategy::TimeBucket(config) => {
                attributes.append(&mut time_bucket_partitions(config, Utc::now()))
            }
        }
    }

    vec![Partition {
        otap_batch,
        attributes: Some(attributes),
    }]
}

fn time_bucket_partitions(
    config: &TimeBucketConfig,
    now: DateTime<Utc>,
) -> Vec<PartitionAttribute> {
    let mut attributes = vec![PartitionAttribute {
        key: "date".to_string(),
        value: PartitionAttributeValue::String(now.format("%Y-%m-%d").to_string()),
    }];
    if config.granularity == TimeBucketGranularity::Hour {
        attributes.push(PartitionAttribute {
            key: "hour".to_string(),
            value: PartitionAttributeValue::String(now.format("%H").to_string()),
        });
    }

    attributes
}

fn static_partitions_from_schema_metadata(
    otap_batch: &OtapParquetRecords,
    metadata_keys: &[String],
) -> Vec<PartitionAttribute> {
    let main_record_batch = match otap_batch {
        OtapParquetRecords::Logs(_) => otap_batch.get(ArrowPayloadType::Logs),
        OtapParquetRecords::Metrics(_) => {
            match otap_batch.get(ArrowPayloadType::UnivariateMetrics) {
                Some(rb) => Some(rb),
                None => otap_batch.get(ArrowPayloadType::MultivariateMetrics),
            }
        }
        OtapParquetRecords::Traces(_) => otap_batch.get(ArrowPayloadType::Spans),
    };
    match main_record_batch {
        None => vec![],
        Some(record_batch) => {
            let mut attributes = Vec::with_capacity(metadata_keys.len());
            let schema = record_batch.schema_ref();
            for key in metadata_keys {
                if let Some(value) = get_schema_metadata(schema, key) {
                    attributes.push(PartitionAttribute {
                        key: key.to_string(),
                        value: PartitionAttributeValue::String(value.to_string()),
                    })
                }
            }

            attributes
        }
    }
}

#[cfg(test)]
pub mod test {
    use super::*;

    use arrow::array::{ArrayRef, RecordBatch, UInt16Array};
    use arrow::datatypes::{DataType, Field, Schema};
    use otel_arrow_dfe_pdata::otap::Logs;
    use otel_arrow_dfe_pdata::otap::OtapArrowRecords;
    use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
    use otel_arrow_dfe_pdata::schema::consts;
    use std::sync::Arc;

    use crate::exporters::parquet_exporter::config::PartitioningStrategy;

    // Helper to create a dummy OtapParquetRecords with schema metadata
    fn make_otap_batch_with_metadata(key: &str, value: &str) -> OtapParquetRecords {
        let schema = Schema::new(vec![Field::new(consts::ID, DataType::UInt16, true)])
            .with_metadata([(key.to_string(), value.to_string())].into_iter().collect());
        let array: ArrayRef = Arc::new(UInt16Array::from(vec![0u16]));
        let batch = RecordBatch::try_new(Arc::new(schema), vec![array]).unwrap();
        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch.set(ArrowPayloadType::Logs, batch).unwrap();
        otap_batch.into()
    }

    #[test]
    fn test_partition_with_schema_metadata_strategy() {
        let key = "test_key";
        let value = "test_value";
        let otap_batch = make_otap_batch_with_metadata(key, value);

        let strategies = vec![PartitioningStrategy::SchemaMetadata(vec![key.to_string()])];

        let partitions = partition(otap_batch, &strategies);
        assert_eq!(partitions.len(), 1);

        let attrs = partitions[0].attributes.as_ref().unwrap();
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].key, key);
        match &attrs[0].value {
            PartitionAttributeValue::String(s) => assert_eq!(s, value),
        }
    }

    #[test]
    fn test_partition_with_missing_metadata_key() {
        let otap_batch = make_otap_batch_with_metadata("present", "yes");
        let strategies = vec![PartitioningStrategy::SchemaMetadata(vec![
            "absent".to_string(),
        ])];
        let partitions = partition(otap_batch, &strategies);
        assert_eq!(partitions.len(), 1);
        let attrs = partitions[0].attributes.as_ref().unwrap();
        assert!(attrs.is_empty());
    }

    #[test]
    fn test_time_bucket_partitions_day() {
        let now = DateTime::parse_from_rfc3339("2026-08-11T23:59:31Z")
            .unwrap()
            .to_utc();
        let config = TimeBucketConfig {
            granularity: TimeBucketGranularity::Day,
        };
        let attrs = time_bucket_partitions(&config, now);
        assert_eq!(attrs.len(), 1);
        assert_eq!(attrs[0].key, "date");
        assert_eq!(format!("{}", attrs[0].value), "2026-08-11");
    }

    #[test]
    fn test_time_bucket_partitions_hour() {
        let now = DateTime::parse_from_rfc3339("2026-08-11T05:00:00Z")
            .unwrap()
            .to_utc();
        let config = TimeBucketConfig {
            granularity: TimeBucketGranularity::Hour,
        };
        let attrs = time_bucket_partitions(&config, now);
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key, "date");
        assert_eq!(format!("{}", attrs[0].value), "2026-08-11");
        assert_eq!(attrs[1].key, "hour");
        assert_eq!(format!("{}", attrs[1].value), "05");
    }

    #[test]
    fn test_partition_with_time_bucket_and_schema_metadata_strategies() {
        let otap_batch = make_otap_batch_with_metadata("_part_id", "abc-123");
        let strategies = vec![
            PartitioningStrategy::TimeBucket(TimeBucketConfig {
                granularity: TimeBucketGranularity::Day,
            }),
            PartitioningStrategy::SchemaMetadata(vec!["_part_id".to_string()]),
        ];

        let partitions = partition(otap_batch, &strategies);
        assert_eq!(partitions.len(), 1);

        // strategies emit attributes in config order: date first, then _part_id
        let attrs = partitions[0].attributes.as_ref().unwrap();
        assert_eq!(attrs.len(), 2);
        assert_eq!(attrs[0].key, "date");
        assert_eq!(attrs[1].key, "_part_id");
        assert_eq!(format!("{}", attrs[1].value), "abc-123");
    }

    #[test]
    fn test_partition_display_trait() {
        let attr_val = PartitionAttributeValue::String("hello".to_string());
        assert_eq!(format!("{attr_val}"), "hello");
    }
}
