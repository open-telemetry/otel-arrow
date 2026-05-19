# Benchmark Results

## Phase 2 Benchmarks

The OpenTelemetry Arrow project is currently in **Phase 2**, where we are
building an end-to-end dataflow engine in Rust. This architecture is expected to
have substantially lower overhead than traditional row-oriented pipelines.

### Current Performance Results

We run two types of automated benchmark tests for Phase 2:

- **[Continuous
  Benchmarks](https://open-telemetry.github.io/otel-arrow/benchmarks/continuous/)**
  - Run with each commit to main
- **[Nightly
  Benchmarks](https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/)**
  - Comprehensive test suites run nightly

Both provide performance metrics for the OTAP dataflow engine for various
scenarios. Unless otherwise specified, all tests run on a single CPU core.

#### 1. Filter + OTel Collector Comparison

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/filter/>

Tests a filter scenario where a filter processor drops 95% of logs. Processes
approximately 100k logs/sec input with ~5k logs/sec output. The benchmark page
includes a direct comparison with the equivalent OTel Collector performing the
same filtering operation.

#### 2. Backpressure

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/backpressure/>

Measures backpressure impact with `wait_for_result` set to true on the dataflow
engine receivers. Processes approximately 100k logs/sec input and output. The
pipeline includes an attribute processor configured to rename an attribute,
which forces the dataflow engine to perform in-memory representation and
conversion rather than operating in pass-through mode.

#### 3. Syslog

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/syslog/>

Tests syslog ingestion via UDP with two variations:

- Basic syslog message format
- CEF (Common Event Format) formatted messages

Processes approximately 5k logs/sec input and output.

#### 4. Normal Load

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/continuous/>

Standard load test processing 100k records/sec input and output on a single CPU
core. This test runs with each commit to main.

#### 5. Normal Load with Batch Size Variations

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/standardload-batch-size/>

Standard load test (100k logs/sec) with varying input batch sizes: 10, 100, 512,
1024, 4096, and 8192 records per request. Uses power-of-2 values that align with
OTel SDK defaults (512 is the standard SDK batch size). Tests both OTAP->OTAP
(native protocol) and OTLP->OTLP (standard protocol) configurations to evaluate
the impact of batch size on CPU, memory, and network efficiency.

#### 6. Saturation and Scaling

##### 6a. Max Throughput (Single Core)

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/saturation/>

Measures the absolute maximum throughput a single core can sustain, for both
OTLP and OTAP protocols. Uses `semantic_conventions` (~300 byte logs) -- the
same payload as all other benchmarks -- for direct comparability. Loadgen is
unleashed (no rate cap) with enough cores to fully saturate the single SUT core.

OTAP uses 8 loadgen + 4 backend cores (vs 4+2 for OTLP) because the Arrow
protocol is significantly more efficient and requires more load to saturate.

##### 6b. Scaling Efficiency (Multi-Core)

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/saturation/>

Validates the shared-nothing, thread-per-core architecture by measuring how
throughput scales as more CPU cores are added. Each test pushes load until the
engine is fully saturated (CPU-bound) at 1, 2, 4, 8, and 16 cores, then computes
scaling efficiency:

```txt
Scaling Efficiency = Actual Throughput / (Baseline 1-core Throughput x Core Count)
```

A value of 1.0 means perfect linear scaling -- doubling cores doubles throughput.
Values below 1.0 indicate contention, shared resources, or measurement
limitations (e.g., loadgen unable to push enough traffic). The architecture
targets >0.90 efficiency at all core counts.

Uses static 1KB log bodies with realistic entropy (512 unique bodies) -- unlike
other tests which use `semantic_conventions` (~300 byte logs) -- to better
exercise the serialization/compression/network path at saturation.

Scaling efficiency ratios are tracked over time:
<https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/scaling-efficiency/>

#### 7. Pass-through Mode

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/continuous-passthrough/>

Tests maximum throughput in pass-through mode where the engine forwards data
without transformation. This scenario represents the minimum engine overhead for
load balancing and routing use cases. Unlike the saturation tests which include
an attribute processor, pass-through mode allows the engine to forward data
without materializing the internal representation, achieving significantly
higher throughput.

#### 8. Idle State

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/continuous-idle-state/>

Measures resource consumption in idle state across multiple core configurations
(1, 2, 4, 8, 16, 32 cores) to validate the linear memory scaling model:

```txt
Memory (MiB) = C + N * R
```

Where:

- **C** = Constant overhead (shared infrastructure)
- **N** = Number of cores
- **R** = Per-core memory overhead

This validates the share-nothing, thread-per-core architecture where each
additional core adds a predictable amount of memory overhead.

#### 9. Binary Size

**URL:** <https://open-telemetry.github.io/otel-arrow/benchmarks/binary-size/>

Tracks the binary size of the dataflow engine for Linux ARM64 and AMD64
architectures over time.

### Metrics Collected

All benchmark tests measure the following metrics:

- **Logs/sec input** - Input throughput
- **Logs/sec output** - Output throughput
- **RAM** - Average and maximum memory usage
- **Normalized CPU** - Average and maximum CPU usage, normalized to 0-100% where
  100% represents full utilization of all available cores. For example, in a
  4-core test, 80% means 3.2 cores are being used (0.8 X 4 cores)
- **Network bytes/sec** - Input and output network bandwidth

### Learn More About Phase 2

- [Phase 2 OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md)
- [Project Phases Overview](project-phases.md)
- [Phase 2 Design Document](phase2-design.md)

## Phase 1 Benchmarks (Historical)

For historical benchmark results from Phase 1 (the collector-to-collector
traffic reduction implementation), please see [Phase 1 Benchmark Results](benchmarks-phase1.md).

Phase 1 focused on facilitating traffic reduction between OpenTelemetry
Collectors and is now complete. These components are available in the
OpenTelemetry Collector-Contrib distribution.
