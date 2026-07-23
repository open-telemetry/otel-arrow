# otap-df-expohisto

Allocation-free OpenTelemetry exponential histogram.

## Overview

This crate implements the [OpenTelemetry exponential
histogram](https://opentelemetry.io/docs/specs/otel/metrics/data-model/#exponentialhistogram)
using a fixed-size, const-generic data pool (`Histogram<N>`). It performs no
heap allocation, contains no `unsafe` code, and has no runtime dependencies.

Bucket index mapping is accelerated by a compile-time lookup table that is
checked in as generated data (`src/lookup_tables.rs` and
`src/inverse_factors.rs`), so the crate builds without any code-generation
step.

## Types

- `HistogramNN<N>` (aliased as `Histogram<N>`): positive-only histogram;
  negative values are rejected. Suited to non-negative measurements such as
  latencies, sizes, and counts.
- `HistogramPN<K, L>`: independent positive and negative bucket ranges with
  synchronized scales, for values of any sign.

## Usage

```rust
use otap_df_expohisto::Histogram;

let mut hist: Histogram<16> = Histogram::new();
hist.update(1.5).unwrap();
hist.update(2.7).unwrap();
hist.update(100.0).unwrap();

let view = hist.view();
let stats = view.stats();
assert_eq!(stats.count, 3);
```

## Features

- `std` (default): enables `std::error::Error` impls. Disable for `no_std`.
- `boundary` (default): bucket lower-boundary computation.
- `quantile` (default): quantile estimation over the bucket distribution.

## Dependencies

Intentionally none.
