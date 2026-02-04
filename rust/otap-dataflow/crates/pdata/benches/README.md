# OTAP Concatenation Benchmarks

This directory contains benchmarks for comparing the performance of different OTAP record concatenation implementations.

## Available Benchmarks

### `concatenate.rs`

Compares two implementations for concatenating OTAP record batches:

1. **New implementation**: `concatenate()` from `transform/concatenate.rs`
2. **Old implementation**: `generic_schemaless_concatenate()` from `groups.rs`

**Important**: Both implementations are benchmarked on the exact same data to ensure fair comparison. Data is generated once upfront for each configuration, then cloned for each benchmark iteration.

The benchmark tests both implementations with:
- **Single metric type**: Varying numbers of inputs (10, 100) with 100 points per metric
- **Mixed metric types**: Gauges, sums, and histograms with varying numbers of inputs (10, 100)

## Running the Benchmarks

### Run all concatenation benchmarks
```bash
cargo bench --bench concatenate
```

### Run only single-metric-type comparison
```bash
cargo bench --bench concatenate concatenate_comparison
```

### Run only mixed-metric-type comparison
```bash
cargo bench --bench concatenate mixed_comparison
```

### Filter for just new or old implementation results
```bash
cargo bench --bench concatenate -- new
cargo bench --bench concatenate -- old
```

### Test that benchmarks work (without full benchmark run)
```bash
cargo bench --bench concatenate -- --test
```

## Interpreting Results

Criterion will output:
- **Time per iteration**: How long each concatenation operation takes
- **Throughput**: Operations per second
- **Comparison**: Performance difference between new and old implementations

Results are saved to `target/criterion/` and can be viewed as HTML reports.

Since both implementations run on identical data within each benchmark group, the performance comparison is directly meaningful and fair.

## Implementation Details

The benchmarks:
1. Generate OTAP metrics data using `DataGenerator` from the testing module
2. Convert OTLP protocol messages to OTAP format using `otlp_to_otap`
3. Extract the internal batch arrays from `OtapArrowRecords::Metrics`
4. **Store the generated data for reuse across both implementations**
5. For each iteration, clone the pre-generated data (cloning time is measured but negligible compared to concatenation)
6. Measure the time to concatenate the batches using each implementation

This approach ensures:
- Both implementations operate on identical data
- Only the concatenation logic is meaningfully benchmarked
- Results are directly comparable
- The clone overhead is consistent across both implementations

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
