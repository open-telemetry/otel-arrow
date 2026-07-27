# otap-df-expohisto

An openTelemetry exponential histogram.

## Overview

This crate implements the [OpenTelemetry exponential
histogram](https://opentelemetry.io/docs/specs/otel/metrics/data-model/#exponentialhistogram)
using a fixed-size generic of `N` 64-bit words per instance
`HistogramNN<N>`. This structure does not allocate memory.

Bucket index mapping is accelerated by a compile-time lookup table
that is checked in as generated data , so the crate builds without any
code-generation step.

## Types

- `HistogramNN<N>`: positive-only histogram; negative values are
  rejected. Suited to non-negative measurements such as latencies,
  sizes, and counts.

## Usage

```rust
use otap_df_expohisto::HistogramNN;

let mut hist: HistogramNN<16> = HistogramNN::new();
hist.update(1.5).unwrap();
hist.update(2.7).unwrap();
hist.update(100.0).unwrap();

let view = hist.view();
let stats = view.stats();
assert_eq!(stats.count, 3);
```

## Features

- `std` (default): enables `std::error::Error` impls. Disable for `no_std`.
- `quantile` (default): quantile estimation over the bucket distribution.

## Dependencies

Intentionally none.
