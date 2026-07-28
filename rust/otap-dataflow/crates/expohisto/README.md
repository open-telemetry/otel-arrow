# otap-df-expohisto

An OpenTelemetry exponential histogram aggregator.

## Overview

This crate implements the [OpenTelemetry exponential
histogram](https://opentelemetry.io/docs/specs/otel/metrics/data-model/#exponentialhistogram)
using a fixed-size generic of `N` 64-bit words per instance
`HistogramNN<N>`. This structure does not allocate memory.

This data structure incorporates an exact lookup table.

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
