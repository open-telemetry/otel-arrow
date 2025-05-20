// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(missing_docs)]

use std::sync::Arc;

use arrow::array::{
    BooleanArray, Float64Array, Int64Array, RecordBatch, StringBuilder, UInt8Array, UInt16Array,
};
use arrow::datatypes::{DataType, Field, Schema};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use otel_arrow_rust::otlp::attributes::{
    decoder::materialize_parent_id, store::AttributeValueType,
};
use otel_arrow_rust::schema::consts;

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
            str_values.append_value(&format!("str{}", (i as f64 / 10.0) as usize));
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
    .unwrap()
}

fn bench_materialize_parent_ids(c: &mut Criterion) {
    let mut group = c.benchmark_group("materialize_parent_ids");

    for size in [128, 1536, 8092] {
        let input = create_bench_batch(size);
        let _ = group.bench_with_input(
            BenchmarkId::new("materialize_parent_ids", size),
            &input,
            |b, input| {
                b.iter(|| {
                    let _ = materialize_parent_id::<u16>(input).unwrap();
                });
            },
        );
    }

    group.finish()
}

criterion_group!(benches, bench_materialize_parent_ids);

criterion_main!(benches);

/*
Before refactoring key attr iID

   Compiling otel-arrow-rust v0.1.0 (/Users/a.lockett/Development/otel-arrow/rust/otel-arrow-rust)
    Finished `bench` profile [optimized] target(s) in 2.91s
     Running benches/materialize_parent_id.rs (target/release/deps/materialize_parent_id-a7d317d401e450db)
Gnuplot not found, using plotters backend
materialize_parent_ids/materialize_parent_ids/128
                        time:   [3.9560 µs 3.9672 µs 3.9787 µs]
                        change: [-0.8860% -0.4680% -0.0247%] (p = 0.03 < 0.05)
                        Change within noise threshold.

materialize_parent_ids/materialize_parent_ids/128
                        time:   [3.5858 µs 3.5949 µs 3.6041 µs]
                        change: [-9.5912% -9.2245% -8.8545%] (p = 0.00 < 0.05)
                        Performance has improved.


Before setting targetcpu = native

Gnuplot not found, using plotters backend
materialize_parent_ids/materialize_parent_ids/128
                        time:   [3.5086 µs 3.5255 µs 3.5432 µs]
                        change: [-2.2071% -1.7504% -1.3228%] (p = 0.00 < 0.05)
                        Performance has improved.
Found 7 outliers among 100 measurements (7.00%)
  5 (5.00%) low mild
  2 (2.00%) high mild
materialize_parent_ids/materialize_parent_ids/1536
                        time:   [22.737 µs 22.829 µs 22.929 µs]
Found 4 outliers among 100 measurements (4.00%)
  1 (1.00%) low mild
  3 (3.00%) high mild
materialize_parent_ids/materialize_parent_ids/8092
                        time:   [115.25 µs 115.57 µs 115.88 µs]


*/
