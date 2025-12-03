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

Both provide performance metrics for the OTAP dataflow engine, for various
scenarios.
TODO: Add details on the scenario

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
