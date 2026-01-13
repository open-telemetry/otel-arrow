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
**URL:** https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/filter/

Tests a filter scenario where a filter processor drops 95% of logs. Processes
approximately 100k logs/sec input with ~5k logs/sec output. The benchmark page
includes a direct comparison with the equivalent OTel Collector performing the
same filtering operation.

#### 2. Backpressure
**URL:** https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/backpressure/

Measures backpressure impact with `wait_for_result` set to true on the dataflow
engine receivers. Processes approximately 100k logs/sec input and output. The
pipeline includes an attribute processor configured to rename an attribute,
which forces the dataflow engine to perform in-memory representation and
conversion rather than operating in pass-through mode.

#### 3. Syslog
**URL:** https://open-telemetry.github.io/otel-arrow/benchmarks/nightly/syslog/

Tests syslog ingestion via UDP with two variations:
- Basic syslog message format
- CEF (Common Event Format) formatted messages

Processes approximately 5k logs/sec input and output.

#### 4. Normal Load
**URL:** https://open-telemetry.github.io/otel-arrow/benchmarks/continuous/

Standard load test processing 100k records/sec input and output on a single CPU
core. This test runs with each commit to main.

#### 5. Saturation and Scaling
**URL:** https://open-telemetry.github.io/otel-arrow/benchmarks/continuous-saturation/

Tests performance at saturation across different CPU configurations: 1, 2, 4, 8,
and 16 cores.

*TODO: Update test output to include scalability ratios in addition to raw throughput numbers.*

#### 6. Idle State
**URL:** https://open-telemetry.github.io/otel-arrow/benchmarks/continuous-idle-state/

Measures resource consumption in idle state with two variations:
- Single core configuration
- All-cores configuration (128 cores)

#### 7. Binary Size
**URL:** https://open-telemetry.github.io/otel-arrow/benchmarks/binary-size/

Tracks the binary size of the dataflow engine for Linux ARM64 and AMD64
architectures over time.

### Metrics Collected

All benchmark tests measure the following metrics:
- **Logs/sec input** - Input throughput
- **Logs/sec output** - Output throughput
- **RAM** - Average and maximum memory usage
- **Normalized CPU** - Average and maximum CPU usage normalized to a 0-100% range relative to the available cores in the test configuration (e.g., 80% in a 4-core test means 80% of 4 cores)
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
