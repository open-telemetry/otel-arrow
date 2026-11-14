// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Construct/drop and construct/traverse native metric views over prepared Arrow batches.

#[path = "../tests/common/metrics_view.rs"]
mod fixture;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use fixture::Layout;
use otel_arrow_dfe_pdata::views::otap::OtapMetricsView;
use otel_arrow_dfe_pdata_views::views::{
    common::{AnyValueView, AttributeView},
    metrics::*,
};
use std::hint::black_box;

fn metrics_view(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("metrics_view");
    for points in [16, 8192, 10000, 20000] {
        let _ = group.throughput(Throughput::Elements(points as u64));
        for layout in [Layout::Grouped, Layout::Scattered, Layout::Mixed] {
            for dictionary in [false, true] {
                let records = fixture::records(points, layout, dictionary);
                let case = format!("{layout:?}/dictionary={dictionary}/{points}");
                let _ = group.bench_with_input(
                    BenchmarkId::new("construct", &case),
                    &records,
                    |bench, records| {
                        bench.iter(|| {
                            black_box(
                                OtapMetricsView::try_from(black_box(records))
                                    .expect("valid synthetic metrics fixture"),
                            )
                        });
                    },
                );
                let _ = group.bench_with_input(
                    BenchmarkId::new("traverse", &case),
                    &records,
                    |bench, records| {
                        bench.iter(|| {
                            let view = OtapMetricsView::try_from(black_box(records))
                                .expect("valid synthetic metrics fixture");
                            let mut sum = 0i64;
                            for resource in view.resources() {
                                for scope in resource.scopes() {
                                    for metric in scope.metrics() {
                                        let data =
                                            metric.data().expect("valid synthetic metrics fixture");
                                        if let Some(gauge) = data.as_gauge() {
                                            for point in gauge.data_points() {
                                                sum += checksum(&point);
                                            }
                                        } else if let Some(counter) = data.as_sum() {
                                            for point in counter.data_points() {
                                                sum += checksum(&point);
                                            }
                                        }
                                    }
                                }
                            }
                            black_box(sum)
                        });
                    },
                );
            }
        }
    }
    group.finish();
}

fn checksum(point: &impl NumberDataPointView) -> i64 {
    point
        .attributes()
        .map(|attr| {
            attr.key().len() as i64
                + attr
                    .value()
                    .expect("valid synthetic metrics fixture")
                    .as_int64()
                    .expect("valid synthetic metrics fixture")
        })
        .sum()
}

criterion_group!(benches, metrics_view);
criterion_main!(benches);
