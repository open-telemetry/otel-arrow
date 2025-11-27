// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Benchmarks for functions involved with filtering pdata

use std::collections::HashSet;
use std::hint::black_box;
use std::sync::Arc;

use arrow::array::Array;
use criterion::{Criterion, criterion_group, criterion_main};
use otap_df_pdata::otap::filter::build_uint16_id_filter;
use otap_df_pdata::proto::OtlpProtoMessage;
use otap_df_pdata::proto::opentelemetry::arrow::v1::ArrowPayloadType;
use otap_df_pdata::proto::opentelemetry::common::v1::{AnyValue, KeyValue};
use otap_df_pdata::proto::opentelemetry::logs::v1::{LogRecord, LogsData, ResourceLogs, ScopeLogs};
use otap_df_pdata::testing::round_trip::otlp_to_otap;

/// Benchmark for [`build_uint16_id_filter`]
///
/// # Motivation:
///
/// After filtering we call this function to create a selection vector for some batch related
/// to what has been filtered. This is used to synchronize which rows are kept after applying
/// filtering to the related batch. Performance testing has revealed this can be a performance
/// bottleneck when filtering, so this benchmark is here to measure its performance.
///
fn bench_build_uint16_id_filter(c: &mut Criterion) {
    let batch_sizes = [32, 1024, 8192];
    let attrs_per_parents = [2, 5, 10, 20];
    let proportions_selected = [0.01, 0.10, 0.5, 0.75, 0.90, 0.99];

    let mut group = c.benchmark_group("build_uint16_id_filter");

    for transport_sorted in [false, true] {
        for batch_size in batch_sizes {
            for attrs_per_parent in attrs_per_parents {
                for proportion_ids_selected in proportions_selected {
                    let params = IdFilterGenParameters {
                        num_parents: batch_size,
                        attrs_per_parent,
                        proportion_ids_selected,
                        transport_sorted,
                    };
                    let args = gen_build_uint16_id_filter_args(params);
                    let id = format!(
                        "{batch_size};{attrs_per_parent};{proportion_ids_selected};{transport_sorted}"
                    );
                    _ = group.bench_with_input(id, &args, |b, args| {
                        b.iter(|| {
                            let (id_column, id_set) = &args;
                            let result = build_uint16_id_filter(id_column, id_set.clone()).unwrap();
                            _ = black_box(result);
                        })
                    });
                }
            }
        }
    }

    group.finish();
}

/// Parameters for generating the arguments for [`build_uint16_id_filter`] as if we've filtered
/// the parent record batch and are now using the function to determine which attributes to keep
#[derive(Debug)]
struct IdFilterGenParameters {
    /// How many parent IDs there are
    num_parents: usize,

    /// How many attributes each parent has
    attrs_per_parent: usize,

    /// The proportion of the parent_ids selected by the id_set. Should be a float between 0 and 1
    proportion_ids_selected: f64,

    /// The sort order of the attributes record batch.
    ///
    /// There are two ways we usually OTAP record batches: converting from OTLP, and via the
    /// OTAP receiver. When we convert from OTAP, the attributes record batches are generally
    /// sorted by parent ID. Conversely, when we receive batches from the OTAP receiver, the
    /// attributes may have a transport optimized encoding where they're sorted by a combination of
    /// key, value & parent_id. We expect the caller to have removed the delta encoding of the IDs
    /// before calling this function, but not the sort order, and so we must ensure it is
    /// performant for both sort orders.
    transport_sorted: bool,
}

/// generate arguments for [`build_uint16_id_filter`]
fn gen_build_uint16_id_filter_args(
    params: IdFilterGenParameters,
) -> (Arc<dyn Array>, HashSet<u16>) {
    let mut id_set = vec![];
    let mut log_records = vec![];

    // will be used to choose which IDs to select so we get the proper proportion of IDs
    let select_id_mod = (1.0 / params.proportion_ids_selected) as usize;

    for i in 0..params.num_parents {
        let mut attrs = vec![];
        for j in 0..params.attrs_per_parent {
            attrs.push(KeyValue::new(
                format!("key{j}"),
                AnyValue::new_string("val"),
            ));
        }

        log_records.push(LogRecord::build().attributes(attrs).finish());

        if i % select_id_mod == 0 {
            id_set.push(i as u16);
        }
    }

    let logs_data = LogsData {
        resource_logs: vec![ResourceLogs {
            scope_logs: vec![ScopeLogs {
                log_records,
                ..Default::default()
            }],
            ..Default::default()
        }],
    };

    let mut otap_batch = otlp_to_otap(&OtlpProtoMessage::Logs(logs_data));
    if params.transport_sorted {
        otap_batch.encode_transport_optimized().unwrap();
        otap_batch.decode_transport_optimized_ids().unwrap();
    }

    let log_attrs = otap_batch.get(ArrowPayloadType::LogAttrs).unwrap().clone();

    return (
        log_attrs.column_by_name("parent_id").unwrap().clone(),
        id_set.into_iter().collect(),
    );
}

#[allow(missing_docs)]
mod benches {
    use super::*;

    criterion_group!(
        name = benches;
        config = Criterion::default();
        targets = bench_build_uint16_id_filter
    );
}

criterion_main!(benches::benches);
