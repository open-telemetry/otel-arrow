// Copyright The OpenTelemetry Authors
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::print_stdout)]

//! Minimum-of-trials timing for the recording hot path.
//!
//! Criterion reports a median, which on a shared machine drifts by several
//! percent between runs and hides changes of that size. The minimum over many
//! trials is far more stable, because interference can only ever make a trial
//! slower.
//!
//! Run with:
//!
//! ```text
//! cargo bench -p otap-df-expohisto --bench hot
//! ```

use otap_df_expohisto::{HistogramNN, Scale, table_scale};
use std::hint::black_box;
use std::time::Instant;

const OBSERVATIONS: usize = 1_024;
const TRIALS: usize = 400;

fn wide_values() -> [f64; OBSERVATIONS] {
    std::array::from_fn(|index| {
        let magnitude = (index % 64) as i32 - 16;
        let mantissa = 1.0 + ((index * 17) % 100) as f64 / 100.0;
        mantissa * 2.0_f64.powi(magnitude)
    })
}

/// Nanoseconds per observation, taken as the minimum over `TRIALS` runs.
fn measure(label: &str, mut trial: impl FnMut()) {
    // Warm the caches and let the clock settle before measuring.
    for _ in 0..32 {
        trial();
    }

    let mut best = f64::INFINITY;
    for _ in 0..TRIALS {
        let start = Instant::now();
        trial();
        let elapsed = start.elapsed().as_secs_f64();
        best = best.min(elapsed);
    }
    println!(
        "{label:<24} {:>7.2} ns/observation",
        best * 1e9 / OBSERVATIONS as f64
    );
}

fn main() {
    let values = wide_values();

    let mut probe: HistogramNN<10> = HistogramNN::new();
    for value in &values {
        probe.update(*value).expect("recordable");
    }
    let settled_scale = probe.view().scale();
    let settled_width = probe.view().positive().width();
    println!("settled geometry: scale {settled_scale}, {settled_width:?}");

    let configured: HistogramNN<10> = HistogramNN::new()
        .with_min_width(settled_width)
        .expect("the settled width fits the default scale")
        .with_max_scale(settled_scale)
        .expect("the settled scale covers the settled width");

    let mapping = Scale::new(table_scale()).expect("table scale is valid");
    measure("map_to_index", || {
        let mut total = 0i64;
        for value in &values {
            total += i64::from(mapping.map_to_index(black_box(*value)));
        }
        let _ = black_box(total);
    });

    measure("record_configured", || {
        let mut histogram = configured.clone();
        for value in &values {
            histogram.update(black_box(*value)).expect("recordable");
        }
        let _ = black_box(&histogram);
    });

    measure("record_from_empty", || {
        let mut histogram: HistogramNN<10> = HistogramNN::new();
        for value in &values {
            histogram.update(black_box(*value)).expect("recordable");
        }
        let _ = black_box(&histogram);
    });
}
