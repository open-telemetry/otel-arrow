// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

use std::sync::Arc;

use arrow::array::{
    BooleanArray, Float64Array, Int64Array, RecordBatch, StringBuilder, UInt8Array, UInt16Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use criterion::{BenchmarkId, Criterion, black_box, criterion_group, criterion_main};

use otel_arrow_rust::otap::transform::sort_by_parent_id;
use otel_arrow_rust::otlp::attributes::store::{Attribute16Store, AttributeStoreV2};
use otel_arrow_rust::otlp::attributes::{
    decoder::materialize_parent_id, store::AttributeValueType,
};
use otel_arrow_rust::schema::consts;

// TODO probably don't want this duplicated from the materialize bench id test
fn create_bench_batch(num_attrs: usize) -> RecordBatch {
    let mut types = UInt8Array::builder(num_attrs);
    let mut keys = StringBuilder::new();
    let mut str_values = StringBuilder::new();
    let mut int_values = Int64Array::builder(num_attrs);
    let mut double_values = Float64Array::builder(num_attrs);
    let mut bool_values = BooleanArray::builder(num_attrs);
    let mut parent_ids = UInt16Array::builder(num_attrs);

    // Distribute value types: 40% string, 30% int, 20% double, 10% bool
    let str_threshold = (num_attrs as f64 * 0.4) as usize;
    let int_threshold = (num_attrs as f64 * 0.7) as usize;
    let double_threshold = (num_attrs as f64 * 0.9) as usize;

    for i in 0..num_attrs {
        parent_ids.append_value(1);
        let attr_name = format!("attr{}", (i as f64 / 50.0) as usize);
        keys.append_value(attr_name);

        if i < str_threshold {
            types.append_value(AttributeValueType::Str as u8);
            int_values.append_null();
            bool_values.append_null();
            double_values.append_null();
            str_values.append_value(format!("str{}", (i as f64 / 10.0) as usize));
            continue;
        }

        if i < int_threshold {
            types.append_value(AttributeValueType::Int as u8);
            bool_values.append_null();
            double_values.append_null();
            str_values.append_null();
            int_values.append_value((i as f64 / 10.0) as i64);
            continue;
        }

        if i < double_threshold {
            types.append_value(AttributeValueType::Double as u8);
            bool_values.append_null();
            int_values.append_null();
            str_values.append_null();
            double_values.append_value((i as f64 / 10.0).floor());
            continue;
        }

        types.append_value(AttributeValueType::Bool as u8);
        str_values.append_null();
        int_values.append_null();
        double_values.append_null();
        bool_values.append_value((i as f64 / 10.0) as usize % 2 == 0);
    }

    let schema = Schema::new(vec![
        Field::new(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(consts::ATTRIBUTE_KEY, DataType::Utf8, false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
        Field::new(consts::ATTRIBUTE_STR, DataType::Utf8, true),
        Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true),
        Field::new(consts::ATTRIBUTE_DOUBLE, DataType::Float64, true),
        Field::new(consts::ATTRIBUTE_BOOL, DataType::Boolean, true),
    ]);

    RecordBatch::try_new(Arc::new(schema), vec![
        Arc::new(parent_ids.finish()),
        Arc::new(keys.finish()),
        Arc::new(types.finish()),
        Arc::new(str_values.finish()),
        Arc::new(int_values.finish()),
        Arc::new(double_values.finish()),
        Arc::new(bool_values.finish()),
    ])
    .expect("expect can create this record batch")
}

fn bench_attribute_store(c: &mut Criterion) {
    let mut group = c.benchmark_group("materialize_parent_ids");

    for size in [128, 1536, 8092] {
        let input = create_bench_batch(size);
        let _ = group.bench_with_input(
            BenchmarkId::new("v1_attr_store", size),
            &input,
            |b, input| {
                b.iter(|| {
                    let mut store = Attribute16Store::try_from(input).unwrap();
                    for _ in 1..=10 {
                        let attr_iter = store.attribute_by_delta_id(1).unwrap();
                        for attr in attr_iter {
                            _ = black_box(attr);
                        }
                    }
                });
            },
        );

        let _ = group.bench_with_input(
            BenchmarkId::new("v2_attr_store", size),
            &input,
            |b, input| {
                b.iter(|| {
                    let sorted_batch = sort_by_parent_id(input).unwrap();
                    let mut store = AttributeStoreV2::<u16>::try_from(&sorted_batch).unwrap();
                    for _ in 1..=10 {
                        let attr_iter = store.attribute_by_delta_id(1).unwrap();
                        for item in attr_iter {
                            _ = black_box(item);
                        }
                    }
                });
            },
        );
    }

    group.finish()
}

criterion_group! {
    name = benches;
    config = Criterion::default().measurement_time(std::time::Duration::from_secs(10));
    targets = bench_attribute_store
}
criterion_main!(benches);
