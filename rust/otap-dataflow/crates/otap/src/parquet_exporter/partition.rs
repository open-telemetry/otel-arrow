// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

use otel_arrow_rust::{
    otap::OtapArrowRecords, proto::opentelemetry::arrow::v1::ArrowPayloadType,
    schema::get_schema_metadata,
};

use super::config::PartitioningStrategy;

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
    pub otap_batch: OtapArrowRecords,
    pub attributes: Option<Vec<PartitionAttribute>>,
}

pub fn partition(
    otap_batch: &OtapArrowRecords,
    strategies: &[PartitioningStrategy],
) -> Vec<Partition> {
    let mut attributes = vec![];

    // This is a fairly simply implementation for now. This will be refactored a lot when
    // we add different partitioning strategies (e.g. partitioning by date bucket)
    for strategy in strategies {
        match strategy {
            PartitioningStrategy::SchemaMetadata(metadata_keys) => attributes.append(
                &mut static_partitions_from_schema_metadata(otap_batch, metadata_keys),
            ),
        }
    }

    vec![Partition {
        otap_batch: otap_batch.clone(),
        attributes: Some(attributes),
    }]
}

fn static_partitions_from_schema_metadata(
    otap_batch: &OtapArrowRecords,
    metadata_keys: &[String],
) -> Vec<PartitionAttribute> {
    let main_record_batch = match otap_batch {
        OtapArrowRecords::Logs(_) => otap_batch.get(ArrowPayloadType::Logs),
        OtapArrowRecords::Metrics(_) => match otap_batch.get(ArrowPayloadType::UnivariateMetrics) {
            Some(rb) => Some(rb),
            None => otap_batch.get(ArrowPayloadType::MultivariateMetrics),
        },
        OtapArrowRecords::Traces(_) => otap_batch.get(ArrowPayloadType::Spans),
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

    use arrow::array::{ArrayRef, RecordBatch, StringArray};
    use arrow::datatypes::{DataType, Field, Schema};
    use otel_arrow_rust::otap::Logs;
    use otel_arrow_rust::{
        otap::OtapArrowRecords, proto::opentelemetry::arrow::v1::ArrowPayloadType,
    };
    use std::sync::Arc;

    use crate::parquet_exporter::partition::PartitioningStrategy;

    // Helper to create a dummy OtapBatch with schema metadata
    fn make_otap_batch_with_metadata(key: &str, value: &str) -> OtapArrowRecords {
        let schema = Schema::new(vec![Field::new("foo", DataType::Utf8, false)])
            .with_metadata([(key.to_string(), value.to_string())].into_iter().collect());
        let array: ArrayRef = Arc::new(StringArray::from(vec!["bar"]));
        let batch = RecordBatch::try_new(Arc::new(schema), vec![array]).unwrap();
        let mut otap_batch = OtapArrowRecords::Logs(Logs::default());
        otap_batch.set(ArrowPayloadType::Logs, batch);

        otap_batch
    }

    #[test]
    fn test_partition_with_schema_metadata_strategy() {
        let key = "test_key";
        let value = "test_value";
        let otap_batch = make_otap_batch_with_metadata(key, value);

        let strategies = vec![PartitioningStrategy::SchemaMetadata(vec![key.to_string()])];

        let partitions = partition(&otap_batch, &strategies);
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
        let partitions = partition(&otap_batch, &strategies);
        assert_eq!(partitions.len(), 1);
        let attrs = partitions[0].attributes.as_ref().unwrap();
        assert!(attrs.is_empty());
    }

    #[test]
    fn test_partition_display_trait() {
        let attr_val = PartitionAttributeValue::String("hello".to_string());
        assert_eq!(format!("{attr_val}"), "hello");
    }
}
