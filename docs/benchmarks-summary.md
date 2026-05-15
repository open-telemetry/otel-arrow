# OpenTelemetry Arrow Performance Summary

## Overview

The OpenTelemetry Arrow (OTel Arrow) project is currently in **Phase 2**,
building an end-to-end Arrow-based telemetry pipeline in Rust. Phase 1 focused
on collector-to-collector traffic compression using the OTAP protocol, achieving
significant network bandwidth savings. Phase 2 expands this foundation by
implementing the entire in-process pipeline using Apache Arrow's columnar
format, targeting substantial improvements in data processing efficiency while
maintaining the network efficiency gains from Phase 1.

The OTel Arrow dataflow engine, implemented in Rust, provides predictable
performance characteristics and efficient resource utilization across varying
load conditions. The engine uses a [thread-per-core
architecture](#thread-per-core-design) where resource consumption scales with
the number of configured cores.

This document presents a curated set of key performance metrics across different
load scenarios and test configurations. For the complete set of automated
performance tests (continuous, nightly, saturation, idle state, and binary size
benchmarks), see [Detailed Benchmark
Results](benchmarks.md#current-performance-results).

### Test Environment

All performance tests are executed on bare-metal compute instance with the
following specifications:

- **CPU**: 64 physical cores / 128 logical cores (x86-64 architecture)
- **Memory**: 512 GB RAM
- **Platform**: Oracle Bare Metal Instance
- **OS**: Oracle Linux 8

This consistent, high-performance environment ensures reproducible results and
allows for comprehensive testing across various CPU core configurations (1, 4,
and 8 cores etc.) by constraining the OTel Arrow dataflow engine to specific
core allocations.

### Performance Metrics

#### Idle State Performance

Baseline resource consumption with no active telemetry traffic, measured after
startup stabilization over a 60-second period. These metrics represent
post-initialization idle state and validate minimal resource footprint. Note
that longer-duration soak testing for memory leak detection is outside the scope
of this benchmark summary.

| Configuration   | CPU Usage | Memory Usage |
| --------------- | --------- | ------------ |
| Single Core     | 0.1%      | 15 MB        |
| All Cores (128) | 2.5%      | 600 MB       |

*Note: CPU usage is normalized (percentage of total system capacity). Memory
usage scales with core count due to the [thread-per-core
architecture](#thread-per-core-design).*

These baseline metrics validate that the engine maintains minimal resource
footprint when idle, ensuring efficient operation in environments with variable
telemetry loads.

#### Standard Load Performance (Single Core)

Resource utilization at 100,000 log records per second (100K logs/sec) on a
single CPU core. Tests are conducted with different batch sizes to demonstrate
the impact of batching on performance.

**Test Parameters:**

- Total input load: 100,000 log records/second
- Average log record size: 1 KB
- Batch sizes tested: 10, 100, 512, 1024, 4096, and 8192 records per request
- Test duration: 60 seconds

This wide range of batch sizes evaluates performance across diverse deployment
scenarios. Small batches (10-100) represent edge collectors or real-time
streaming requirements, while large batches (1024-8192) represent gateway
collectors and high-throughput aggregation points. This approach ensures a fair
assessment, highlighting both the overhead for small batches and the significant
efficiency gains inherent to Arrow's columnar format at larger batch sizes.

##### Standard Load - OTAP -> OTAP (Native Protocol)

| Batch Size | CPU Usage | Memory Usage | Network In | Network Out |
|------------|-----------|--------------|------------|-------------|
| 512/batch | 50% | 16 MB | 767 KB/s | 833 KB/s |

*Note: Only 512/batch has been tested for OTAP at standard load so far. OTAP
batch size variants for small batches are blocked by a known drain-timeout issue
and will be added once resolved.*

This represents the optimal scenario where the dataflow engine operates with its
native protocol end-to-end, eliminating protocol conversion overhead.

##### Standard Load - OTLP -> OTLP (Standard Protocol)

| Batch Size | CPU Usage | Memory Usage | Network In | Network Out |
|------------|-----------|--------------|------------|-------------|
| 10/batch | 71%* | 19 MB | 4.1 MB/s | 4.4 MB/s |
| 100/batch | 67% | 17 MB | 4.8 MB/s | 5.0 MB/s |
| 512/batch | 65% | 17 MB | 2.8 MB/s | 3.0 MB/s |
| 1024/batch | 65% | 17 MB | 2.6 MB/s | 2.7 MB/s |
| 4096/batch | 57% | 22 MB | 2.4 MB/s | 2.5 MB/s |
| 8192/batch | 55% | 30 MB | 2.4 MB/s | 2.5 MB/s |

*\* At batch size 10, the engine saturates at ~33K logs/sec and cannot reach the
100K target. The per-request overhead at very small batch sizes dominates.*

This scenario processes OTLP end-to-end using the standard OpenTelemetry
protocol, providing a baseline for comparison with traditional OTLP-based
pipelines.

#### Saturation Performance (Single Core)

Maximum throughput achievable on a single CPU core at full utilization. This
establishes the baseline "unit of capacity" for capacity planning.

**Test Parameters:**

- Batch size: 512 records per request
- Load: Continuously increased until the CPU core is fully saturated
- Test duration: 60 seconds at maximum load

##### Pass-through Mode

Forwarding without data transformation. Represents the minimum engine overhead
for load balancing and routing use cases.

| Protocol | Max Throughput | CPU Usage | Memory Usage |
|----------|----------------|-----------|--------------|
| OTLP -> OTLP (Standard) | ~542K logs/sec | ~96% | ~33 MB |

##### With Processing

Includes an attribute processor to force data materialization. Represents
typical production workloads where collectors perform transformations such as
filtering, attribute enrichment, renaming, or aggregation.

| Protocol | Max Throughput | CPU Usage | Memory Usage |
|----------|----------------|-----------|--------------|
| OTLP -> OTLP (Standard) | ~260K logs/sec | ~91% | ~41 MB |

*Note: OTAP -> OTAP saturation tests for passthrough and processing modes are
not yet available and will be added in a future update.*

#### Scalability

How throughput scales when adding CPU cores. The [thread-per-core
architecture](#thread-per-core-design) enables near-linear scaling by
eliminating shared-state synchronization overhead.

**Test Parameters:**

- Batch size: 512 records per request
- Protocol: OTLP -> OTLP with attribute processing
- Load: Maximum sustained throughput at each core count

| CPU Cores | Max Throughput | CPU Usage | Scaling Efficiency | Memory Usage |
|-----------|----------------|-----------|-------------------|--------------|
| 1 Core    | ~238K logs/sec | ~99%      | 100% (baseline)   | ~38 MB       |
| 2 Cores   | ~414K logs/sec | ~91%      | 87%               | ~55 MB       |
| 4 Cores   | ~653K logs/sec | ~70%*     | 69%               | ~80 MB       |
| 8 Cores   | ~1.47M logs/sec | ~82%*    | 78%               | ~178 MB      |
| 16 Cores  | ~2.37M logs/sec | ~78%*    | 62%               | ~288 MB      |
| 24 Cores  | ~3.67M logs/sec | ~77%*    | 64%               | ~461 MB      |

*\* At higher core counts, CPU utilization drops below 100% because the
load-generator cannot fully saturate all engine cores. Adding more load-generator
cores would increase both throughput and CPU utilization.*

Scaling Efficiency = (Throughput at N cores) / (N × Single-core throughput)

### Architecture

The OTel Arrow dataflow engine is built in Rust, to achieve high throughput and
low latency. The columnar data representation and zero-copy processing
capabilities enable efficient handling of telemetry data at scale.

#### Thread-Per-Core Design

The dataflow engine supports a configurable runtime execution model, using a
**thread-per-core architecture** that eliminates traditional concurrency
overhead. This design allows:

- **CPU Affinity Control**: Pipelines can be pinned to specific CPU cores or
  groups through configuration
- **NUMA Optimization**: Memory and CPU assignments can be coordinated for
  Non-Uniform Memory Access (NUMA) architectures
- **Workload Isolation**: Different telemetry signals or tenants can be assigned
  to dedicated CPU resources, preventing resource contention
- **Reduced Synchronization**: Thread-per-core design minimizes lock contention
  and context switching overhead

For detailed technical documentation, see the [OTAP Dataflow Engine
Documentation](../rust/otap-dataflow/README.md) and [Phase 2
Design](phase2-design.md).

---

## Additional Resources

- [Detailed Benchmark Results from phase2](benchmarks.md)
- [Phase 1 Benchmark Results](benchmarks-phase1.md)
- [OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md)
- [Project Phases Overview](project-phases.md)
