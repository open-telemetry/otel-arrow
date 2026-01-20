// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Test bundle creation for e2e stress testing.
//!
//! Generates realistic telemetry bundles with configurable row counts and sizes.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::SystemTime;

use arrow_array::RecordBatch;
use arrow_array::builder::{Float64Builder, Int64Builder, StringBuilder};
use arrow_schema::{DataType, Field, Schema};
use quiver::record_bundle::{
    BundleDescriptor, PayloadRef, RecordBundle, SchemaFingerprint, SlotDescriptor, SlotId,
};
use rand::Rng;

/// Configuration for test bundle generation.
#[derive(Debug, Clone)]
pub struct BundleConfig {
    /// Number of rows per bundle (for main metrics slot).
    pub rows_per_bundle: usize,
    /// Average size of string values in bytes.
    pub string_size: usize,
}

impl Default for BundleConfig {
    fn default() -> Self {
        Self {
            rows_per_bundle: 100,
            string_size: 32,
        }
    }
}

/// A test bundle with sample telemetry data.
pub struct TestBundle {
    descriptor: BundleDescriptor,
    ingestion_time: SystemTime,
    payloads: HashMap<SlotId, (SchemaFingerprint, RecordBatch)>,
}

impl TestBundle {
    /// Creates a new test bundle with the given index and configuration.
    pub fn new(index: usize, config: &BundleConfig) -> Self {
        let slots = vec![
            SlotDescriptor::new(SlotId::new(0), "Metrics"),
            SlotDescriptor::new(SlotId::new(1), "Resource"),
            SlotDescriptor::new(SlotId::new(2), "Scope"),
            SlotDescriptor::new(SlotId::new(3), "Attributes"),
        ];

        let mut payloads = HashMap::new();

        // Create metrics payload (main data)
        let metrics_batch = create_metrics_batch(index, config.rows_per_bundle, config.string_size);
        let _ = payloads.insert(SlotId::new(0), ([0u8; 32], metrics_batch));

        // Create resource payload
        let resource_batch = create_resource_batch(index, config.string_size);
        let _ = payloads.insert(SlotId::new(1), ([1u8; 32], resource_batch));

        // Create scope payload
        let scope_batch = create_scope_batch(index, config.string_size);
        let _ = payloads.insert(SlotId::new(2), ([2u8; 32], scope_batch));

        // Create attributes payload (1/10th the rows of metrics)
        let attrs_batch =
            create_attributes_batch(index, config.rows_per_bundle / 10, config.string_size);
        let _ = payloads.insert(SlotId::new(3), ([3u8; 32], attrs_batch));

        Self {
            descriptor: BundleDescriptor::new(slots),
            ingestion_time: SystemTime::now(),
            payloads,
        }
    }
}

impl RecordBundle for TestBundle {
    fn descriptor(&self) -> &BundleDescriptor {
        &self.descriptor
    }

    fn ingestion_time(&self) -> SystemTime {
        self.ingestion_time
    }

    fn payload(&self, slot: SlotId) -> Option<PayloadRef<'_>> {
        self.payloads.get(&slot).map(|(fp, batch)| PayloadRef {
            schema_fingerprint: *fp,
            batch,
        })
    }
}

/// Generates a vector of test bundles with default string size.
pub fn generate_test_bundles(
    count: usize,
    rows_per_bundle: usize,
    string_size: usize,
) -> Vec<TestBundle> {
    let config = BundleConfig {
        rows_per_bundle,
        string_size,
    };
    (0..count).map(|i| TestBundle::new(i, &config)).collect()
}

/// Generates a string of the specified size with a base value repeated/truncated.
fn sized_string(base: &str, target_size: usize) -> String {
    if target_size == 0 {
        return String::new();
    }
    let mut result = String::with_capacity(target_size);
    while result.len() < target_size {
        result.push_str(base);
    }
    result.truncate(target_size);
    result
}

/// Creates a sample metrics record batch with realistic metric data.
fn create_metrics_batch(index: usize, num_rows: usize, string_size: usize) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("metric_name", DataType::Utf8, false),
        Field::new("value", DataType::Float64, false),
        Field::new("timestamp_ns", DataType::Int64, false),
        Field::new("resource_id", DataType::Int64, false),
        Field::new("scope_id", DataType::Int64, false),
    ]));

    let mut rng = rand::rng();
    let mut name_builder = StringBuilder::new();
    let mut value_builder = Float64Builder::new();
    let mut ts_builder = Int64Builder::new();
    let mut resource_builder = Int64Builder::new();
    let mut scope_builder = Int64Builder::new();

    // Realistic metric names (will be sized)
    let metric_bases = [
        "cpu.utilization",
        "memory.usage",
        "disk.io.read",
        "disk.io.write",
        "network.bytes.in",
        "network.bytes.out",
        "http.request.duration",
        "http.request.count",
        "gc.pause.duration",
        "thread.count",
    ];

    let base_ts = 1_700_000_000_000_000_000_i64 + (index as i64 * 1_000_000_000);
    let resource_id = (index % 10) as i64;
    let scope_id = (index % 5) as i64;

    for i in 0..num_rows {
        let base_name = metric_bases[i % metric_bases.len()];
        let metric_name = sized_string(base_name, string_size);
        name_builder.append_value(metric_name);
        value_builder.append_value(rng.random_range(0.0..100.0));
        ts_builder.append_value(base_ts + (i as i64 * 1_000_000)); // 1ms apart
        resource_builder.append_value(resource_id);
        scope_builder.append_value(scope_id);
    }

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(name_builder.finish()),
            Arc::new(value_builder.finish()),
            Arc::new(ts_builder.finish()),
            Arc::new(resource_builder.finish()),
            Arc::new(scope_builder.finish()),
        ],
    )
    .expect("failed to create metrics batch")
}

/// Creates a sample resource record batch.
fn create_resource_batch(index: usize, string_size: usize) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("service.name", DataType::Utf8, false),
        Field::new("service.version", DataType::Utf8, false),
        Field::new("host.name", DataType::Utf8, false),
        Field::new("instance_id", DataType::Int64, false),
    ]));

    let services = [
        "api-gateway",
        "order-service",
        "payment-service",
        "inventory",
        "shipping",
    ];
    let hosts = [
        "prod-node-01",
        "prod-node-02",
        "prod-node-03",
        "prod-node-04",
    ];

    let mut service_builder = StringBuilder::new();
    let mut version_builder = StringBuilder::new();
    let mut host_builder = StringBuilder::new();
    let mut id_builder = Int64Builder::new();

    service_builder.append_value(sized_string(services[index % services.len()], string_size));
    version_builder.append_value(sized_string(
        &format!("1.{}.0", index % 10),
        string_size.min(16),
    ));
    host_builder.append_value(sized_string(hosts[index % hosts.len()], string_size));
    id_builder.append_value(index as i64);

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(service_builder.finish()),
            Arc::new(version_builder.finish()),
            Arc::new(host_builder.finish()),
            Arc::new(id_builder.finish()),
        ],
    )
    .expect("failed to create resource batch")
}

/// Creates a sample scope record batch.
fn create_scope_batch(index: usize, string_size: usize) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("scope.name", DataType::Utf8, false),
        Field::new("scope.version", DataType::Utf8, false),
    ]));

    let scopes = [
        "io.opentelemetry.metrics",
        "io.opentelemetry.traces",
        "com.example.custom",
    ];

    let mut name_builder = StringBuilder::new();
    let mut version_builder = StringBuilder::new();

    name_builder.append_value(sized_string(scopes[index % scopes.len()], string_size));
    version_builder.append_value(sized_string("1.0.0", string_size.min(16)));

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(name_builder.finish()),
            Arc::new(version_builder.finish()),
        ],
    )
    .expect("failed to create scope batch")
}

/// Creates a sample attributes record batch.
fn create_attributes_batch(index: usize, num_rows: usize, string_size: usize) -> RecordBatch {
    let schema = Arc::new(Schema::new(vec![
        Field::new("key", DataType::Utf8, false),
        Field::new("value", DataType::Utf8, false),
        Field::new("parent_id", DataType::Int64, false),
    ]));

    let attr_keys = [
        "environment",
        "region",
        "cluster",
        "pod",
        "container",
        "deployment",
        "namespace",
    ];
    let attr_values = [
        "production",
        "us-west-2",
        "cluster-01",
        "pod-abc123",
        "container-main",
        "v2-canary",
        "default",
    ];

    let mut key_builder = StringBuilder::new();
    let mut value_builder = StringBuilder::new();
    let mut parent_builder = Int64Builder::new();

    let num = num_rows.max(1);
    for i in 0..num {
        key_builder.append_value(sized_string(attr_keys[i % attr_keys.len()], string_size));
        value_builder.append_value(sized_string(
            attr_values[i % attr_values.len()],
            string_size,
        ));
        parent_builder.append_value(index as i64);
    }

    RecordBatch::try_new(
        schema,
        vec![
            Arc::new(key_builder.finish()),
            Arc::new(value_builder.finish()),
            Arc::new(parent_builder.finish()),
        ],
    )
    .expect("failed to create attributes batch")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_creation() {
        let config = BundleConfig {
            rows_per_bundle: 100,
            string_size: 32,
        };
        let bundle = TestBundle::new(42, &config);

        // Check descriptor has 4 slots
        assert!(bundle.payload(SlotId::new(0)).is_some());
        assert!(bundle.payload(SlotId::new(1)).is_some());
        assert!(bundle.payload(SlotId::new(2)).is_some());
        assert!(bundle.payload(SlotId::new(3)).is_some());

        // Check metrics payload
        let metrics = bundle.payload(SlotId::new(0)).expect("metrics payload");
        assert_eq!(metrics.batch.num_rows(), 100);
        assert_eq!(metrics.batch.num_columns(), 5);

        // Check resource payload
        let resource = bundle.payload(SlotId::new(1)).expect("resource payload");
        assert_eq!(resource.batch.num_rows(), 1);
        assert_eq!(resource.batch.num_columns(), 4);

        // Check scope payload
        let scope = bundle.payload(SlotId::new(2)).expect("scope payload");
        assert_eq!(scope.batch.num_rows(), 1);

        // Check attributes payload
        let attrs = bundle.payload(SlotId::new(3)).expect("attrs payload");
        assert_eq!(attrs.batch.num_rows(), 10); // 100 / 10
    }

    #[test]
    fn test_generate_bundles() {
        let bundles = generate_test_bundles(10, 50, 32);
        assert_eq!(bundles.len(), 10);

        for bundle in &bundles {
            let metrics = bundle.payload(SlotId::new(0)).expect("metrics");
            assert_eq!(metrics.batch.num_rows(), 50);
        }
    }

    #[test]
    fn test_sized_string() {
        assert_eq!(sized_string("abc", 10), "abcabcabca");
        assert_eq!(sized_string("hello", 3), "hel");
        assert_eq!(sized_string("test", 4), "test");
        assert_eq!(sized_string("x", 0), "");
    }
}
