// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use arrow::array::{
    BinaryArray, BooleanArray, Float64Array, Int64Array, PrimitiveArray, StringArray, UInt8Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::sync::Arc;

use arrow::record_batch::RecordBatch;
use otel_arrow_rust::otlp::attributes::store::{Attribute32Store, AttributeStore};
use otel_arrow_rust::otlp::attributes::store2::{Attribute32Store2, AttributeStore2};
use otel_arrow_rust::proto::opentelemetry::common::v1::KeyValue;
use otel_arrow_rust::schema::consts;

fn create_bench_batch(num_attrs: usize, num_unique_parents: usize) -> RecordBatch {
    let schema = Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt32, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
        Field::new(consts::ATTRIBUTE_BYTES, DataType::Binary, true),
    ]);

    // Create parent IDs with roughly equal distribution
    let parent_ids: Vec<u32> = (0..num_attrs)
        .map(|idx| (idx % num_unique_parents) as u32 + 1)
        .collect();

    // Create attribute keys
    let keys: Vec<String> = (0..num_attrs)
        .map(|idx| format!("attr_key_{}", idx % 5))
        .collect();

    // Distribute value types: 40% string, 30% int, 20% double, 10% bool
    let mut types = Vec::with_capacity(num_attrs);
    let mut str_values = Vec::with_capacity(num_attrs);
    let mut int_values = Vec::with_capacity(num_attrs);
    let mut double_values = Vec::with_capacity(num_attrs);
    let mut bool_values = Vec::with_capacity(num_attrs);
    let mut bytes_values = Vec::with_capacity(num_attrs);

    for i in 0..num_attrs {
        let value_type = match i % 10 {
            0..=3 => 1, // string
            4..=6 => 2, // int
            7..=8 => 3, // double
            _ => 4,     // bool
        };

        types.push(value_type);

        // Add values only for the corresponding type
        str_values.push(if value_type == 1 {
            Some(format!("value_{}", i))
        } else {
            None
        });
        int_values.push(if value_type == 2 {
            Some(i as i64)
        } else {
            None
        });
        double_values.push(if value_type == 3 {
            Some(i as f64 / 10.0)
        } else {
            None
        });
        bool_values.push(if value_type == 4 {
            Some(i % 2 == 0)
        } else {
            None
        });
        bytes_values.push(None); // Not using binary values in benchmark
    }

    let parent_id = Arc::new(PrimitiveArray::<arrow::datatypes::UInt32Type>::from(
        parent_ids,
    ));
    let keys = Arc::new(StringArray::from_iter(keys.into_iter().map(Some)));
    let types = Arc::new(UInt8Array::from(types));
    let string_values = Arc::new(StringArray::from_opt_vec(str_values));
    let int_values = Arc::new(Int64Array::from_opt_vec(int_values));
    let double_values = Arc::new(Float64Array::from_opt_vec(double_values));
    let bool_values = Arc::new(BooleanArray::from_opt_vec(bool_values));
    let bytes_values = Arc::new(BinaryArray::from_opt_vec(bytes_values));

    RecordBatch::try_new(Arc::new(schema), vec![
        parent_id,
        keys,
        types,
        string_values,
        int_values,
        double_values,
        bool_values,
        bytes_values,
    ])
    .unwrap()
}

fn bench_attribute_store_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("attribute_store_creation");

    // Test with different data sizes
    for size in [100, 1000, 10000] {
        let num_unique_parents = size / 5; // Average 5 attributes per parent
        let rb = create_bench_batch(size, num_unique_parents);

        group.bench_with_input(BenchmarkId::new("AttributeStore", size), &rb, |b, rb| {
            b.iter(|| {
                let _store = Attribute32Store::try_from(rb).unwrap();
            });
        });

        group.bench_with_input(BenchmarkId::new("AttributeStore2", size), &rb, |b, rb| {
            b.iter(|| {
                let _store = Attribute32Store2::try_from(rb).unwrap();
            });
        });
    }

    group.finish();
}

fn bench_attribute_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("attribute_lookup");

    for size in [100, 1000, 10000] {
        let num_unique_parents = size / 5; // Average 5 attributes per parent
        let rb = create_bench_batch(size, num_unique_parents);

        // Create both store types
        let store1 = Attribute32Store::try_from(&rb).unwrap();
        let store2 = Attribute32Store2::try_from(&rb).unwrap();

        // Benchmark lookups on 10 random parent IDs
        let ids: Vec<u32> = (1..=10).collect();

        group.bench_with_input(
            BenchmarkId::new("AttributeStore-lookup", size),
            &ids,
            |b, ids| {
                b.iter(|| {
                    let mut results = Vec::new();
                    for &id in ids {
                        if let Some(attrs) = store1.attributes_by_id(id) {
                            results.push(attrs.len());
                        }
                    }
                    results
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("AttributeStore2-lookup", size),
            &ids,
            |b, ids| {
                b.iter(|| {
                    let mut results = Vec::new();
                    for &id in ids {
                        if let Some(attrs) = store2.attributes_by_id(&id) {
                            results.push(attrs.count());
                        }
                    }
                    results
                });
            },
        );
    }

    group.finish();
}

fn bench_delta_id_lookup(c: &mut Criterion) {
    let mut group = c.benchmark_group("delta_id_lookup");

    for size in [100, 1000, 10000] {
        let num_unique_parents = size / 5; // Average 5 attributes per parent
        let rb = create_bench_batch(size, num_unique_parents);

        // Create both store types
        let mut store1 = Attribute32Store::try_from(&rb).unwrap();
        let mut store2 = Attribute32Store2::try_from(&rb).unwrap();

        // Benchmark delta lookups
        group.bench_with_input(
            BenchmarkId::new("AttributeStore-delta", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut results = Vec::new();
                    for i in 1..=10 {
                        let mut count = 0;
                        if let Some(attrs) = store1.attributes_by_delta_id(i) {
                            count = attrs.len();
                        }
                        results.push(count);
                    }
                    store1.last_id = 0; // Reset for next iteration
                    results
                });
            },
        );

        group.bench_with_input(
            BenchmarkId::new("AttributeStore2-delta", size),
            &size,
            |b, _| {
                b.iter(|| {
                    let mut results = Vec::new();
                    for i in 1..=10 {
                        let mut count = 0;
                        if let Some(attrs) = store2.attributes_by_delta_id(i) {
                            count = attrs.count();
                        }
                        results.push(count);
                    }
                    store2.last_id = 0; // Reset for next iteration
                    results
                });
            },
        );
    }

    group.finish();
}

fn memory_usage_comparison() {
    // This function doesn't run as part of the benchmark but can be called separately
    // to print memory usage information

    let sizes = [100, 1000, 10000, 100000];

    println!("Memory Usage Comparison:");
    println!("------------------------");
    println!("Size\tStore1 Size\tStore2 Size\tRatio");

    for size in sizes {
        let num_unique_parents = size / 5;
        let rb = create_bench_batch(size, num_unique_parents);

        let store1 = Attribute32Store::try_from(&rb).unwrap();
        let store2 = Attribute32Store2::try_from(&rb).unwrap();

        // Count number of key-values in each store to estimate memory usage
        let store1_count: usize = store1.attribute_by_ids.values().map(|v| v.len()).sum();

        // Count number of KeyValue objects in the HashMap for store1
        let store1_size = std::mem::size_of::<KeyValue>() * store1_count
            + std::mem::size_of::<(u32, Vec<KeyValue>)>() * store1.attribute_by_ids.len();

        // For store2, we just count the indices plus the RecordBatch reference
        let store2_size = std::mem::size_of::<Option<usize>>() * store2.next_indices.len()
            + std::mem::size_of::<(u32, usize)>() * store2.first_index_by_id.len()
            + std::mem::size_of::<Arc<RecordBatch>>();

        let ratio = store1_size as f64 / store2_size as f64;

        println!("{}\t{}\t{}\t{:.2}x", size, store1_size, store2_size, ratio);
    }
}

criterion_group!(
    benches,
    bench_attribute_store_creation,
    bench_attribute_lookup,
    bench_delta_id_lookup
);
criterion_main!(benches);
