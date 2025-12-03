// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for OtapLogsView (zero-copy) vs OTLP conversion
//!
//! ## Benchmark Results
//!
//! System: Apple M4 Pro, 24 GB RAM, 12 cores
//!
//! | Logs | Zero-Copy View | OTAP→OTLP→Process | Direct Arrow |
//! |------|----------------|-------------------|--------------|
//! |   10 |      1.48 µs   |      6.43 µs      |    24.41 ns  |
//! |  100 |      9.86 µs   |     40.40 µs      |   233.67 ns  |
//! | 1000 |    121.42 µs   |    383.11 µs      |     2.35 µs  |
//!
//! OTAP→OTLP→Process includes: OTAP→OTLP conversion + OTLP decode + iterate.
//! Direct Arrow is fastest but unrealistic (baseline only, no hierarchical access).

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use otap_df_pdata::otap::OtapArrowRecords;
use otap_df_pdata::otlp::{ProtoBuffer, ProtoBytesEncoder, logs::LogsProtoBytesEncoder};
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::proto::opentelemetry::collector::logs::v1::ExportLogsServiceRequest;
use otap_df_pdata::views::common::AttributeView;
use otap_df_pdata::views::logs::{LogRecordView, LogsDataView, ResourceLogsView, ScopeLogsView};
use otap_df_pdata::views::otap::OtapLogsView;
use prost::Message;

use arrow::array::Array;
use arrow::array::DictionaryArray;
use arrow::datatypes::UInt8Type;
use arrow::error::ArrowError;
use otap_df_pdata::encode::record::attributes::StrKeysAttributesRecordBatchBuilder;
use otap_df_pdata::encode::record::logs::LogsRecordBatchBuilder;

// Helper to create a simple OTAP logs RecordBatch for testing
fn create_test_logs_batch(count: usize) -> Result<OtapArrowRecords, ArrowError> {
    let mut builder = LogsRecordBatchBuilder::new();

    for i in 0..count {
        // Log Record fields
        builder.append_time_unix_nano(100);
        builder.append_observed_time_unix_nano(110);
        builder.append_severity_number(Some(1));
        builder.append_severity_text(Some(b"INFO"));
        builder.body.append_str(b"test log body");
        builder.append_dropped_attributes_count(0);
        builder.append_flags(Some(1));
        // Append unique ID for attribute linking (1-based to avoid 0 which is sometimes treated as null/default)
        builder.append_id(Some((i + 1) as u16));

        // Resource fields (repeated for simplicity in this test)
        builder.resource.append_id(Some(1));
        builder.resource.append_schema_url(None);
        builder.resource.append_dropped_attributes_count(0);

        // Scope fields (repeated for simplicity in this test)
        builder.scope.append_id(Some(10));
        builder.scope.append_name(Some(b"test_scope"));
        builder.scope.append_version(Some(b"1.0"));
        builder.scope.append_dropped_attributes_count(0);
    }

    let mut otap_records = OtapArrowRecords::Logs(Default::default());
    otap_records.set(ArrowPayloadType::Logs, builder.finish()?);

    // Create Resource Attributes (Parent ID = 1)
    let mut res_attrs = StrKeysAttributesRecordBatchBuilder::<u16>::new();
    res_attrs.append_parent_id(&1);
    res_attrs.append_key("service.name");
    res_attrs.any_values_builder.append_str(b"test_service");
    res_attrs.append_parent_id(&1);
    res_attrs.append_key("host.name");
    res_attrs.any_values_builder.append_str(b"localhost");
    otap_records.set(ArrowPayloadType::ResourceAttrs, res_attrs.finish()?);

    // Create Scope Attributes (Parent ID = 10)
    let mut scope_attrs = StrKeysAttributesRecordBatchBuilder::<u16>::new();
    scope_attrs.append_parent_id(&10);
    scope_attrs.append_key("scope.attr");
    scope_attrs.any_values_builder.append_str(b"scope_value");
    otap_records.set(ArrowPayloadType::ScopeAttrs, scope_attrs.finish()?);

    // Create Log Attributes (Parent ID = 1..count)
    let mut log_attrs = StrKeysAttributesRecordBatchBuilder::<u16>::new();
    for i in 0..count {
        let log_id = (i + 1) as u16;
        // Add 3 attributes per log
        log_attrs.append_parent_id(&log_id);
        log_attrs.append_key("log.attr.1");
        log_attrs.any_values_builder.append_str(b"value1");

        log_attrs.append_parent_id(&log_id);
        log_attrs.append_key("log.attr.2");
        log_attrs.any_values_builder.append_int(42);

        log_attrs.append_parent_id(&log_id);
        log_attrs.append_key("log.attr.3");
        log_attrs.any_values_builder.append_bool(true);
    }
    otap_records.set(ArrowPayloadType::LogAttrs, log_attrs.finish()?);

    Ok(otap_records)
}

/// Benchmark entry point
fn bench_otap_logs_view(c: &mut Criterion) {
    let mut group = c.benchmark_group("otap_logs_view");

    for size in [10, 100, 1000].iter() {
        let input = create_test_logs_batch(*size).expect("Failed to create test logs batch");

        let _ = group.bench_with_input(
            BenchmarkId::new("zero_copy_view", size),
            &input,
            |b, input| {
                b.iter(|| {
                    let view =
                        OtapLogsView::try_from(input).expect("Failed to create OtapLogsView");
                    let mut count = 0;
                    for resource in view.resources() {
                        for scope in resource.scopes() {
                            for log in scope.log_records() {
                                // Access body
                                let _ = std::hint::black_box(log.body());
                                // Access attributes
                                for attr in log.attributes() {
                                    let _ = std::hint::black_box(attr.value());
                                }
                                count += 1;
                            }
                        }
                    }
                    assert_eq!(count, *size);
                })
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("otlp_bytes_processing", size),
            &input,
            |b, input: &OtapArrowRecords| {
                b.iter(|| {
                    // 1. Convert OTAP -> OTLP bytes
                    let mut logs_encoder = LogsProtoBytesEncoder::new();
                    let mut buffer = ProtoBuffer::new();
                    let mut input_clone = input.clone();
                    logs_encoder
                        .encode(&mut input_clone, &mut buffer)
                        .expect("Failed to encode logs to OTLP");
                    let otlp_bytes = buffer.as_ref();

                    // 2. Decode OTLP bytes -> OTLP Structs
                    let request = ExportLogsServiceRequest::decode(otlp_bytes)
                        .expect("Failed to decode OTLP logs");

                    // 3. Iterate
                    let mut count = 0;
                    for resource_logs in request.resource_logs {
                        for scope_logs in resource_logs.scope_logs {
                            for _log in scope_logs.log_records {
                                count += 1;
                            }
                        }
                    }
                    assert_eq!(count, *size);
                })
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("direct_arrow_iteration", size),
            &input,
            |b, input: &OtapArrowRecords| {
                let logs_batch = input
                    .get(ArrowPayloadType::Logs)
                    .expect("Logs batch not found");
                let resource_attrs = input
                    .get(ArrowPayloadType::ResourceAttrs)
                    .expect("Resource attrs not found");
                let scope_attrs = input
                    .get(ArrowPayloadType::ScopeAttrs)
                    .expect("Scope attrs not found");
                let log_attrs = input
                    .get(ArrowPayloadType::LogAttrs)
                    .expect("Log attrs not found");

                let time_col = logs_batch
                    .column_by_name("time_unix_nano")
                    .expect("time_unix_nano column not found")
                    .as_any()
                    .downcast_ref::<arrow::array::TimestampNanosecondArray>()
                    .expect("Failed to downcast time_unix_nano");
                let resource_col = logs_batch
                    .column_by_name("resource")
                    .expect("resource column not found")
                    .as_any()
                    .downcast_ref::<arrow::array::StructArray>()
                    .expect("Failed to downcast resource");
                let resource_id_col = resource_col
                    .column_by_name("id")
                    .expect("resource.id column not found")
                    .as_any()
                    .downcast_ref::<arrow::array::UInt16Array>()
                    .expect("Failed to downcast resource.id");
                let scope_col = logs_batch
                    .column_by_name("scope")
                    .expect("scope column not found")
                    .as_any()
                    .downcast_ref::<arrow::array::StructArray>()
                    .expect("Failed to downcast scope");
                let scope_id_col = scope_col
                    .column_by_name("id")
                    .expect("scope.id column not found")
                    .as_any()
                    .downcast_ref::<arrow::array::UInt16Array>()
                    .expect("Failed to downcast scope.id");

                // Attribute columns (Dictionary encoded)
                let res_attr_key_col = resource_attrs
                    .column_by_name("key")
                    .expect("resource attrs key column not found")
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("Failed to downcast resource attrs key");
                let scope_attr_key_col = scope_attrs
                    .column_by_name("key")
                    .expect("scope attrs key column not found")
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("Failed to downcast scope attrs key");
                let log_attr_key_col = log_attrs
                    .column_by_name("key")
                    .expect("log attrs key column not found")
                    .as_any()
                    .downcast_ref::<DictionaryArray<UInt8Type>>()
                    .expect("Failed to downcast log attrs key");

                // Get underlying string values for length calculation
                let res_keys = res_attr_key_col
                    .values()
                    .as_any()
                    .downcast_ref::<arrow::array::StringArray>()
                    .expect("Failed to downcast resource keys to StringArray");
                let scope_keys = scope_attr_key_col
                    .values()
                    .as_any()
                    .downcast_ref::<arrow::array::StringArray>()
                    .expect("Failed to downcast scope keys to StringArray");
                let log_keys = log_attr_key_col
                    .values()
                    .as_any()
                    .downcast_ref::<arrow::array::StringArray>()
                    .expect("Failed to downcast log keys to StringArray");

                b.iter(|| {
                    let mut sum = 0;
                    // Iterate logs
                    for i in 0..logs_batch.num_rows() {
                        let time = time_col.value(i);
                        let res_id = resource_id_col.value(i);
                        let scope_id = scope_id_col.value(i);
                        sum += time as u64 + res_id as u64 + scope_id as u64;
                    }

                    // Iterate attributes (simulating flat access)
                    // For dictionary array, we should look up the value using the key index
                    // But for simplicity/speed in this "flat" benchmark, we can just iterate the keys array
                    // or just sum the lengths of the referenced strings.
                    // Actually, `value(i)` on DictionaryArray returns the value? No, it returns the key index?
                    // DictionaryArray doesn't have `value(i)` returning the string directly in a simple way without casting?
                    // `res_attr_key_col.keys().value(i)` gives the index.
                    // `res_keys.value(index)` gives the string.

                    let res_indices = res_attr_key_col.keys();
                    for i in 0..resource_attrs.num_rows() {
                        if res_indices.is_valid(i) {
                            let idx = res_indices.value(i) as usize;
                            sum += res_keys.value(idx).len() as u64;
                        }
                    }

                    let scope_indices = scope_attr_key_col.keys();
                    for i in 0..scope_attrs.num_rows() {
                        if scope_indices.is_valid(i) {
                            let idx = scope_indices.value(i) as usize;
                            sum += scope_keys.value(idx).len() as u64;
                        }
                    }

                    let log_indices = log_attr_key_col.keys();
                    for i in 0..log_attrs.num_rows() {
                        if log_indices.is_valid(i) {
                            let idx = log_indices.value(i) as usize;
                            sum += log_keys.value(idx).len() as u64;
                        }
                    }

                    let _ = std::hint::black_box(sum);
                })
            },
        );
    }

    group.finish();
}

#[allow(missing_docs)]
mod bench_entry {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_otap_logs_view
    );
}

criterion_main!(bench_entry::benches);
