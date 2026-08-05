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
//! Each tier is measured twice. A histogram starting at the default geometry --
//! the finest scale and one-bit counters -- pays to widen its counters and
//! reduce its scale as the population reveals itself. One given the geometry
//! that population settles on pays neither. The gap between the two is what
//! presetting a known shape is worth.
//!
//! Run with:
//!
//! ```text
//! cargo bench -p otap-df-expohisto --bench hot
//! ```

use otap_df_expohisto::{HistogramNN, Scale, Width, table_scale};
use std::hint::black_box;
use std::time::Instant;

const OBSERVATIONS: usize = 1_024;
const TRIALS: usize = 400;

/// A spread wide enough that the default geometry cannot hold it, so recording
/// pays for both counter widening and scale reduction.
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
        "{label:<34} {:>6.2} ns/observation",
        best * 1e9 / OBSERVATIONS as f64
    );
}

/// Returns the scale and width `values` settle on in an `N`-word pool.
fn settled_geometry<const N: usize>(values: &[f64]) -> (i32, Width) {
    let mut probe: HistogramNN<N> = HistogramNN::new();
    for &value in values {
        probe.update(value).expect("recordable");
    }
    (probe.view().scale(), probe.view().positive().width())
}

/// Times one tier from the default geometry and from its settled geometry.
fn measure_tier<const N: usize>(tier: &str, values: &[f64]) {
    let (scale, width) = settled_geometry::<N>(values);
    println!("{tier} ({N} words) settles at scale {scale}, {width:?}");

    measure(&format!("  {tier} from B1/max scale"), || {
        let mut histogram: HistogramNN<N> = HistogramNN::new();
        for &value in values {
            histogram.update(black_box(value)).expect("recordable");
        }
        let _ = black_box(&histogram);
    });

    let preset: HistogramNN<N> = HistogramNN::new()
        .with_min_width(width)
        .expect("the settled width fits the default scale")
        .with_max_scale(scale)
        .expect("the settled scale covers the settled width");

    measure(&format!("  {tier} preset to settled"), || {
        let mut histogram = preset.clone();
        for &value in values {
            histogram.update(black_box(value)).expect("recordable");
        }
        let _ = black_box(&histogram);
    });
}

fn main() {
    let values = wide_values();

    let mapping = Scale::new(table_scale()).expect("table scale is valid");
    measure("map_to_index alone", || {
        let mut total = 0i64;
        for &value in &values {
            total += i64::from(mapping.map_to_index(black_box(value)));
        }
        let _ = black_box(total);
    });

    measure_tier::<10>("normal", &values);
    measure_tier::<26>("detailed", &values);
}
