// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

//! Allocation traffic for constructing/dropping views over prepared Arrow batches.
//! Run separately from the uninstrumented Criterion timing benchmark. The input,
//! reporting and thread initialization are outside each measurement. View creation
//! is synchronous on the calling thread; unrelated threads are not counted.

#[path = "../tests/common/metrics_view.rs"]
mod fixture;

use fixture::allocations::{self, CountingAllocator};
use otel_arrow_dfe_pdata::views::otap::OtapMetricsView;
use std::{
    hint::black_box,
    io::{self, Write},
};

#[global_allocator]
static ALLOCATOR: CountingAllocator = CountingAllocator;

fn main() {
    const ITERATIONS: u64 = 30;
    let mut output = io::stdout().lock();
    writeln!(
        output,
        "points,layout,dictionary,iterations,allocation_calls_per_view,requested_bytes_per_view"
    )
    .expect("write allocation report header");
    for points in [16, 8192, 10000, 20000] {
        for layout in [
            fixture::Layout::Grouped,
            fixture::Layout::Scattered,
            fixture::Layout::Mixed,
        ] {
            for dictionary in [false, true] {
                let records = fixture::records(points, layout, dictionary);
                // Warm library initialization, but construct and drop a NEW view
                // on every measured iteration. No view/index capacity is reused.
                drop(black_box(
                    OtapMetricsView::try_from(&records).expect("valid synthetic metrics fixture"),
                ));
                let (_, calls, bytes) = allocations::measure(|| {
                    for _ in 0..ITERATIONS {
                        drop(black_box(
                            OtapMetricsView::try_from(black_box(&records))
                                .expect("valid synthetic metrics fixture"),
                        ));
                    }
                });
                writeln!(
                    output,
                    "{points},{layout:?},{dictionary},{ITERATIONS},{},{}",
                    calls as f64 / ITERATIONS as f64,
                    bytes as f64 / ITERATIONS as f64
                )
                .expect("write allocation report row");
                let _ = black_box(&records);
            }
        }
    }
}
