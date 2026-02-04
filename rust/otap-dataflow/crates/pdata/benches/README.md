# OTAP Concatenation Benchmarks

This directory contains benchmarks for comparing the performance of different OTAP record concatenation implementations.

## Running the Benchmarks

### Concatenation
```bash
cargo bench --bench concatenate
```

## Benchmark Results

The following results compare the performance of the new `concatenate` implementation versus the old `generic_schemaless_concatenate` implementation.

### Single Metric Type (Gauges)

| Configuration | New Implementation | Old Implementation | Speedup |
|---------------|-------------------|-------------------|---------|
| 10 inputs, 5 points | 28.18 µs | 101.03 µs | **3.58x** |
| 10 inputs, 50 points | 29.82 µs | 110.47 µs | **3.70x** |
| 100 inputs, 5 points | 246.37 µs | 951.29 µs | **3.86x** |
| 100 inputs, 50 points | 267.27 µs | 1,020.0 µs | **3.82x** |
| 1000 inputs, 5 points | 4.47 ms | 16.62 ms | **3.72x** |
| 1000 inputs, 50 points | 4.98 ms | 17.44 ms | **3.50x** |

### Mixed Metric Types (Gauges, Sums, Histograms)

| Configuration | New Implementation | Old Implementation | Speedup |
|---------------|-------------------|-------------------|---------|
| 10 inputs | 51.87 µs | 183.74 µs | **3.54x** |
| 100 inputs | 497.11 µs | 1,736.3 µs | **3.49x** |
| 1000 inputs | 11.60 ms | 37.88 ms | **3.27x** |

### Summary

The new `concatenate` implementation consistently outperforms the old `generic_schemaless_concatenate` implementation by approximately **3.5x across all tested scenarios**. The performance improvement is consistent regardless of:
- Number of input batches (10 to 1000)
- Points per metric (5 to 50)
- Metric type complexity (single vs mixed types)

This demonstrates that the new implementation provides significant performance benefits for all common concatenation workloads.
