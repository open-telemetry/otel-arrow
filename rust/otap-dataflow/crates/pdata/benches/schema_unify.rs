// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks that isolate the schema-unification stages of OTAP concatenation.
//!
//! `concatenate` performs three logically distinct pieces of work per payload
//! table: indexing every field across the input batches (`index_records`),
//! selecting a unified output schema (`select_schema`), and casting/reordering
//! every input batch to that schema (`convert`). The end-to-end `concatenate`
//! benchmark is dominated by the `BatchCoalescer` row copy, which masks the cost
//! of these stages. These benchmarks call the stages directly (via the
//! `bench` feature exports) so their cost can be attributed and tracked across
//! optimization work.
//!
//! Scenarios deliberately exercise paths the generator-driven `concatenate`
//! benchmark does not: differing optional fields, permuted field order, plain
//! vs. dictionary columns, and dictionary key-width transitions.

use std::hint::black_box;
use std::sync::Arc;

use arrow::array::{
    Array, ArrayRef, DictionaryArray, RecordBatch, StringArray, UInt8Array, UInt16Array,
};
use arrow::datatypes::{DataType, Field, Schema, UInt8Type, UInt16Type};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};

use otel_arrow_dfe_pdata::otap::transform::concatenate::bench_exports::{
    bench_convert_all, bench_index_records, bench_select_schema,
};
use otel_arrow_dfe_pdata::otap::{Logs, OtapArrowRecords, OtapBatchStore, Traces};
use otel_arrow_dfe_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otel_arrow_dfe_pdata::schema::consts;
use otel_arrow_dfe_pdata::testing::fixtures::{DataGenerator, LogsConfig, TracesConfig};
use otel_arrow_dfe_pdata::testing::round_trip::otlp_to_otap;

const BATCH_COUNTS: &[usize] = &[2, 8, 32, 128];
const ROWS_PER_BATCH: usize = 256;

criterion_group!(benches, bench_all);
criterion_main!(benches);

fn bench_all(c: &mut Criterion) {
    // Scenarios 1 & 2 & 7(structs): real OTAP data via the generator. These use
    // the actual payload layouts (spans is the widest table, attributes_16 a
    // narrow one) with identical schemas across batches.
    bench_generated_traces(c);
    bench_generated_logs(c);

    // Scenarios 3-7: synthetic single-payload-slot batches that let us control
    // schema variation precisely. These run against the Logs LogAttrs slot,
    // which maps to attributes_16, so the payload spec is real.
    bench_synthetic(c);
}

// ---------------------------------------------------------------------------
// Generated-data scenarios (identical schemas)
// ---------------------------------------------------------------------------

fn bench_generated_traces(c: &mut Criterion) {
    // Spans is the widest payload table (17 top-level fields + resource/scope/
    // status struct children), so the Spans slot stresses per-field overhead.
    let spans_idx = payload_idx::<Traces>(ArrowPayloadType::Spans);

    let mut group = c.benchmark_group("schema_unify/identical/spans");
    for &n in BATCH_COUNTS {
        let batches = generate_traces(n, ROWS_PER_BATCH);
        bench_stages(
            &mut group,
            n,
            || bench_index_records::<Traces, { Traces::COUNT }>(&batches, spans_idx).expect("schema unify stage failed"),
            || bench_select_schema::<Traces, { Traces::COUNT }>(&batches, spans_idx).expect("schema unify stage failed"),
            || bench_convert_all::<Traces, { Traces::COUNT }>(&batches, spans_idx).expect("schema unify stage failed"),
        );
    }
    group.finish();
}

fn bench_generated_logs(c: &mut Criterion) {
    // LogAttrs maps to attributes_16 (9 fields, no structs): a narrow table.
    let log_attrs_idx = payload_idx::<Logs>(ArrowPayloadType::LogAttrs);

    let mut group = c.benchmark_group("schema_unify/identical/log_attrs");
    for &n in BATCH_COUNTS {
        let batches = generate_logs(n, ROWS_PER_BATCH);
        bench_stages(
            &mut group,
            n,
            || bench_index_records::<Logs, { Logs::COUNT }>(&batches, log_attrs_idx).expect("schema unify stage failed"),
            || bench_select_schema::<Logs, { Logs::COUNT }>(&batches, log_attrs_idx).expect("schema unify stage failed"),
            || bench_convert_all::<Logs, { Logs::COUNT }>(&batches, log_attrs_idx).expect("schema unify stage failed"),
        );
    }
    group.finish();
}

// ---------------------------------------------------------------------------
// Synthetic scenarios (controlled schema variation), on the LogAttrs slot.
// ---------------------------------------------------------------------------

fn bench_synthetic(c: &mut Criterion) {
    let log_attrs_idx = payload_idx::<Logs>(ArrowPayloadType::LogAttrs);

    let scenarios: &[(&str, fn(usize, usize) -> Vec<LogsBatches>)] = &[
        ("optional_subset", make_optional_subset),
        ("permuted_order", make_permuted_order),
        ("dict_cross_u8", make_dict_cross_u8),
        ("dict_cross_u16", make_dict_cross_u16),
        ("plain_and_dict", make_plain_and_dict),
    ];

    for (label, make) in scenarios {
        let mut group = c.benchmark_group(format!("schema_unify/{label}/log_attrs"));
        for &n in BATCH_COUNTS {
            let batches = make(n, ROWS_PER_BATCH);
            bench_stages(
                &mut group,
                n,
                || bench_index_records::<Logs, { Logs::COUNT }>(&batches, log_attrs_idx).expect("schema unify stage failed"),
                || bench_select_schema::<Logs, { Logs::COUNT }>(&batches, log_attrs_idx).expect("schema unify stage failed"),
                || bench_convert_all::<Logs, { Logs::COUNT }>(&batches, log_attrs_idx).expect("schema unify stage failed"),
            );
        }
        group.finish();
    }
}

// ---------------------------------------------------------------------------
// Shared bench harness: time the three stages under one parameter.
// ---------------------------------------------------------------------------

fn bench_stages<R1, R2, R3>(
    group: &mut criterion::BenchmarkGroup<'_, criterion::measurement::WallTime>,
    n: usize,
    index: impl Fn() -> R1,
    select: impl Fn() -> R2,
    convert: impl Fn() -> R3,
) {
    let _ = group.bench_with_input(BenchmarkId::new("index_records", n), &n, |b, _| {
        b.iter(|| black_box(index()));
    });
    let _ = group.bench_with_input(BenchmarkId::new("select_schema", n), &n, |b, _| {
        b.iter(|| black_box(select()));
    });
    let _ = group.bench_with_input(BenchmarkId::new("convert", n), &n, |b, _| {
        b.iter(|| black_box(convert()));
    });
}

// ---------------------------------------------------------------------------
// Generated data helpers
// ---------------------------------------------------------------------------

fn generate_traces(
    num_batches: usize,
    spans_per_scope: usize,
) -> Vec<[Option<RecordBatch>; Traces::COUNT]> {
    let mut datagen = DataGenerator::with_traces_config(
        TracesConfig::new(spans_per_scope)
            .with_resources(1)
            .with_scopes_per_resource(1)
            .with_resource_attrs(10)
            .with_scope_attrs(5)
            .with_span_attrs(3),
    );
    (0..num_batches)
        .map(
            |_| match otlp_to_otap(&datagen.generate_traces_from_config().into()) {
                OtapArrowRecords::Traces(t) => t.into_batches(),
                _ => unreachable!(),
            },
        )
        .collect()
}

fn generate_logs(
    num_batches: usize,
    logs_per_scope: usize,
) -> Vec<[Option<RecordBatch>; Logs::COUNT]> {
    let mut datagen = DataGenerator::with_logs_config(
        LogsConfig::new(logs_per_scope)
            .with_resources(1)
            .with_scopes_per_resource(1)
            .with_resource_attrs(10)
            .with_scope_attrs(5)
            .with_log_attrs(3),
    );
    (0..num_batches)
        .map(
            |_| match otlp_to_otap(&datagen.generate_logs_from_config().into()) {
                OtapArrowRecords::Logs(l) => l.into_batches(),
                _ => unreachable!(),
            },
        )
        .collect()
}

// ---------------------------------------------------------------------------
// Synthetic LogAttrs (attributes_16) batches
// ---------------------------------------------------------------------------

type LogsBatches = [Option<RecordBatch>; Logs::COUNT];

/// Wrap a single LogAttrs RecordBatch into a full Logs batch array in the
/// correct slot, leaving all other slots empty.
fn wrap_log_attrs(rb: RecordBatch) -> LogsBatches {
    let idx = payload_idx::<Logs>(ArrowPayloadType::LogAttrs);
    let mut arr: LogsBatches = std::array::from_fn(|_| None);
    arr[idx] = Some(rb);
    arr
}

/// Base required columns for an attributes_16 batch: parent_id (u16),
/// key (dict<u8,utf8>), type (u8). Callers append optional value columns.
fn base_attrs_columns(rows: usize, key_cardinality: usize) -> (Vec<Field>, Vec<ArrayRef>) {
    let parent_id: ArrayRef = Arc::new(UInt16Array::from(
        (0..rows).map(|i| (i % 64) as u16).collect::<Vec<_>>(),
    ));
    let key = dict_u8_utf8(rows, key_cardinality, "key");
    let atype: ArrayRef = Arc::new(UInt8Array::from(vec![1u8; rows]));

    let fields = vec![
        plain_field(consts::PARENT_ID, DataType::UInt16, false),
        Field::new(consts::ATTRIBUTE_KEY, key.data_type().clone(), false),
        Field::new(consts::ATTRIBUTE_TYPE, DataType::UInt8, false),
    ];
    (fields, vec![parent_id, key, atype])
}

fn plain_field(name: &str, dt: DataType, nullable: bool) -> Field {
    Field::new(name, dt, nullable)
}

/// Build a Dictionary<u8, Utf8> array of `rows` entries drawn from
/// `cardinality` distinct values.
fn dict_u8_utf8(rows: usize, cardinality: usize, prefix: &str) -> ArrayRef {
    let card = cardinality.clamp(1, 256);
    let values: Vec<String> = (0..card).map(|i| format!("{prefix}_{i}")).collect();
    let keys: Vec<u8> = (0..rows).map(|i| (i % card) as u8).collect();
    let dict = DictionaryArray::<UInt8Type>::new(
        UInt8Array::from(keys),
        Arc::new(StringArray::from(values)),
    );
    Arc::new(dict)
}

/// Build a Dictionary<u16, Utf8> array of `rows` entries drawn from
/// `cardinality` distinct values.
fn dict_u16_utf8(rows: usize, cardinality: usize, prefix: &str) -> ArrayRef {
    let card = cardinality.max(1);
    let values: Vec<String> = (0..card).map(|i| format!("{prefix}_{i}")).collect();
    let keys: Vec<u16> = (0..rows).map(|i| (i % card) as u16).collect();
    let dict = DictionaryArray::<UInt16Type>::new(
        UInt16Array::from(keys),
        Arc::new(StringArray::from(values)),
    );
    Arc::new(dict)
}

fn attrs_batch(fields: Vec<Field>, columns: Vec<ArrayRef>) -> RecordBatch {
    let schema = Arc::new(Schema::new(fields));
    RecordBatch::try_new(schema, columns).expect("valid attrs batch")
}

/// Scenario 3: optional value columns present only in some batches, forcing
/// nullability determination and null padding on convert.
fn make_optional_subset(num_batches: usize, rows: usize) -> Vec<LogsBatches> {
    (0..num_batches)
        .map(|b| {
            let (mut fields, mut cols) = base_attrs_columns(rows, 16);
            // Even batches carry attribute_str; odd batches carry attribute_int.
            if b % 2 == 0 {
                fields.push(Field::new(
                    consts::ATTRIBUTE_STR,
                    DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                    true,
                ));
                cols.push(dict_u16_utf8(rows, 32, "str"));
            } else {
                fields.push(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true));
                cols.push(Arc::new(arrow::array::Int64Array::from(
                    (0..rows as i64).collect::<Vec<_>>(),
                )));
            }
            wrap_log_attrs(attrs_batch(fields, cols))
        })
        .collect()
}

/// Scenario 4: same columns everywhere, but field order permuted so the
/// convert stage cannot rely on positional identity.
fn make_permuted_order(num_batches: usize, rows: usize) -> Vec<LogsBatches> {
    (0..num_batches)
        .map(|b| {
            let (fields, cols) = base_attrs_columns(rows, 16);
            // Rotate the three base columns by the batch index.
            let rot = b % fields.len();
            let mut f = fields;
            let mut c = cols;
            f.rotate_left(rot);
            c.rotate_left(rot);
            wrap_log_attrs(attrs_batch(f, c))
        })
        .collect()
}

/// Scenario 5: a dict<u8> column whose summed physical cardinality crosses 255
/// as batches accumulate, forcing a u8 -> u16 key widening in select_schema.
fn make_dict_cross_u8(num_batches: usize, rows: usize) -> Vec<LogsBatches> {
    (0..num_batches)
        .map(|_| {
            let (mut fields, mut cols) = base_attrs_columns(rows, 16);
            // 200 distinct values per batch: one batch stays u8, two overflow it.
            fields.push(Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt8), Box::new(DataType::Utf8)),
                true,
            ));
            cols.push(dict_u8_utf8(rows, 200, "str"));
            wrap_log_attrs(attrs_batch(fields, cols))
        })
        .collect()
}

/// Scenario 6: a dict<u16> column whose summed physical cardinality crosses
/// 65535, forcing demotion to a plain column in select_schema + convert.
fn make_dict_cross_u16(num_batches: usize, rows: usize) -> Vec<LogsBatches> {
    (0..num_batches)
        .map(|b| {
            let (mut fields, mut cols) = base_attrs_columns(rows, 16);
            fields.push(Field::new(
                consts::ATTRIBUTE_STR,
                DataType::Dictionary(Box::new(DataType::UInt16), Box::new(DataType::Utf8)),
                true,
            ));
            // Each batch contributes a disjoint block of distinct values so the
            // physical total grows without bound across batches.
            let card = 1000usize;
            let values: Vec<String> = (0..card).map(|i| format!("s_{b}_{i}")).collect();
            let keys: Vec<u16> = (0..rows).map(|i| (i % card) as u16).collect();
            let dict = DictionaryArray::<UInt16Type>::new(
                UInt16Array::from(keys),
                Arc::new(StringArray::from(values)),
            );
            cols.push(Arc::new(dict));
            wrap_log_attrs(attrs_batch(fields, cols))
        })
        .collect()
}

/// Scenario 7: the same optional column is plain in some batches and
/// dictionary-encoded in others, exercising the plain->dict upgrade arm.
fn make_plain_and_dict(num_batches: usize, rows: usize) -> Vec<LogsBatches> {
    (0..num_batches)
        .map(|b| {
            let (mut fields, mut cols) = base_attrs_columns(rows, 16);
            if b % 2 == 0 {
                // plain int
                fields.push(Field::new(consts::ATTRIBUTE_INT, DataType::Int64, true));
                cols.push(Arc::new(arrow::array::Int64Array::from(
                    (0..rows as i64).collect::<Vec<_>>(),
                )));
            } else {
                // dict<u8, int64> of the same value type
                let card = 32usize;
                let values: Vec<i64> = (0..card as i64).collect();
                let keys: Vec<u8> = (0..rows).map(|i| (i % card) as u8).collect();
                let dict = DictionaryArray::<UInt8Type>::new(
                    UInt8Array::from(keys),
                    Arc::new(arrow::array::Int64Array::from(values)),
                );
                fields.push(Field::new(
                    consts::ATTRIBUTE_INT,
                    dict.data_type().clone(),
                    true,
                ));
                cols.push(Arc::new(dict));
            }
            wrap_log_attrs(attrs_batch(fields, cols))
        })
        .collect()
}

// ---------------------------------------------------------------------------
// misc
// ---------------------------------------------------------------------------

/// Locate the array-index of a payload type within a signal's batch array.
fn payload_idx<S: OtapBatchStore>(pt: ArrowPayloadType) -> usize {
    (0..S::COUNT)
        .find(|&i| S::payload_type_at_idx(i) == pt)
        .expect("payload type present in signal")
}
