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
load conditions. This columnar approach is expected to offer substantial
advantages over traditional row-oriented telemetry pipelines in terms of CPU
efficiency, memory usage, and throughput.

The dataflow engine uses a **thread-per-core architecture**, where each
available CPU core runs an independent runtime instance. This design eliminates
traditional concurrency overhead (lock contention, context switching) but means
that resource consumption scales with the number of configured cores. See the
[Architecture](#architecture) section for more details.

This document presents key performance metrics across different load scenarios
and test configurations.

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

| Configuration | CPU Usage | Memory Usage |
|---------------|-----------|--------------|
| Single Core   | 0.06%     | 27 MB        |
| All Cores (128) | 2.5%    | 600 MB       |

*Note: CPU usage is normalized (percentage of total system capacity). Memory
usage scales with core count due to the thread-per-core architecture (see
[Overview](#overview)).*

These baseline metrics validate that the engine maintains minimal resource
footprint when idle, ensuring efficient operation in environments with variable
telemetry loads.

#### Standard Load Performance

Resource utilization at 100,000 log records per second (100K logs/sec). Tests
are conducted with four different batch sizes to demonstrate the impact of
batching on performance.

**Test Parameters:**

- Total input load: 100,000 log records/second
- Average log record size: 1 KB
- Batch sizes tested: 10, 100, 1000, and 10000 records per request
- Test duration: 60 seconds

This wide range of batch sizes evaluates performance across diverse deployment
scenarios. Small batches (10-100) represent edge collectors or real-time
streaming requirements, while large batches (1000-10000) represent gateway
collectors and high-throughput aggregation points. This approach ensures a fair
assessment, highlighting both the overhead for small batches and the significant
efficiency gains inherent to Arrow's columnar format at larger batch sizes.

##### Standard Load - OTAP -> OTAP (Native Protocol)

| Batch Size | CPU Usage | Memory Usage | Network In | Network Out |
|------------|-----------|--------------|------------|-------------|
| 10/batch | TBD | TBD | TBD | TBD |
| 100/batch | TBD | TBD | TBD | TBD |
| 1000/batch | 17% | 47 MB | 718 KB/s | 748 KB/s |
| 10000/batch | TBD | TBD | TBD | TBD |

This represents the optimal scenario where the dataflow engine operates with its
native protocol end-to-end, eliminating protocol conversion overhead. Results
are shown for a single CPU core to demonstrate baseline efficiency and the
impact of batch size on resource utilization. For hardware scaling
characteristics, refer to the Saturation Performance section.

##### Standard Load - OTLP -> OTLP (Standard Protocol)

| Batch Size | CPU Usage | Memory Usage | Network In | Network Out |
|------------|-----------|--------------|------------|-------------|
| 10/batch | TBD | TBD | TBD | TBD |
| 100/batch | TBD | TBD | TBD | TBD |
| 1000/batch | 43% | 53 MB | 2.1 MB/s | 2.2 MB/s |
| 10000/batch | TBD | TBD | TBD | TBD |

This scenario processes OTLP end-to-end using the standard OpenTelemetry
protocol, providing a baseline for comparison with traditional OTLP-based
pipelines and demonstrating the performance of the columnar architecture even
without OTAP protocol benefits. Results are shown for a single CPU core. For
hardware scaling characteristics, refer to the Saturation Performance section.

#### Saturation Performance (Single Core)

Maximum throughput achievable on a single CPU core at full utilization. This
establishes the baseline "unit of capacity" for capacity planning.

**Test Parameters:**

- Batch size: 500 records per request
- Load: Continuously increased until the CPU core is fully saturated
- Test duration: 60 seconds at maximum load

| Protocol | Max Throughput | Memory Usage |
|----------|----------------|--------------|
| OTAP → OTAP (Native) | TBD | TBD |
| OTLP → OTLP (Standard) | TBD | TBD |

#### Scalability

How throughput scales when adding CPU cores. The thread-per-core architecture
enables near-linear scaling by eliminating shared-state synchronization overhead
(see [Overview](#overview)).

**Test Parameters:**

- Batch size: 500 records per request
- Protocol: OTAP → OTAP (native protocol)
- Load: Maximum sustained throughput at each core count

| CPU Cores | Max Throughput | Scaling Efficiency | Memory Usage |
|-----------|----------------|-------------------|--------------|
| 1 Core    | TBD            | 100% (baseline)   | TBD          |
| 2 Cores   | TBD            | TBD               | TBD          |
| 4 Cores   | TBD            | TBD               | TBD          |
| 8 Cores   | TBD            | TBD               | TBD          |
| 16 Cores  | TBD            | TBD               | TBD          |

*Scaling Efficiency = (Throughput at N cores) / (N × Single-core throughput)*

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

## Comparative Analysis: OTel Arrow vs OpenTelemetry Collector

### Methodology

To provide a fair and meaningful comparison between the OTel Arrow dataflow
engine and the OpenTelemetry Collector, we use **Syslog (UDP/TCP)** as the
ingress protocol for both systems.

#### Rationale for Syslog-Based Comparison

Syslog was specifically chosen as the input protocol because:

1. Neutral Ground: Syslog is neither OTLP (OpenTelemetry Protocol) nor OTAP
   (OpenTelemetry Arrow Protocol), ensuring neither system has a native protocol
   advantage
2. Real-World Relevance: Syslog is widely deployed in production environments,
   particularly for log aggregation from network devices, legacy systems, and
   infrastructure components
3. Conversion Overhead: Both systems must perform meaningful work to convert
   incoming Syslog messages into their internal representations:
   - **OTel Collector**: Converts to Go-based `pdata` (protocol data) structures
   - **OTel Arrow**: Converts to Arrow-based columnar memory format
4. Complete Pipeline Test: This approach validates the full pipeline efficiency,
   including parsing, transformation, and serialization stages

The output protocols are set to each system's native format: OTLP for the
OpenTelemetry Collector and OTAP for the OTel Arrow engine, ensuring optimal
egress performance for each.

### Performance Comparison

#### Baseline (Idle State)

| Metric | OTel Collector | OTel Arrow | Improvement |
|--------|---------------|------------|-------------|
| CPU Usage | TBD | TBD | TBD |
| Memory Usage | TBD | TBD | TBD |

#### Standard Load (100K Syslog Messages/sec)

| Metric | OTel Collector | OTel Arrow | Improvement |
|--------|---------------|------------|-------------|
| CPU Usage | TBD | TBD | TBD |
| Memory Usage | TBD | TBD | TBD |
| Network Egress | TBD | TBD | TBD |
| Throughput (messages/sec) | TBD | TBD | TBD |

#### Saturation

| Metric | OTel Collector | OTel Arrow | Improvement |
|--------|---------------|------------|-------------|
| Maximum Sustained Throughput | TBD | TBD | TBD |
| Throughput / Core | TBD | TBD | TBD |
| CPU at Saturation | TBD | TBD | TBD |
| Memory at Saturation | TBD | TBD | TBD |
| Behavior Under Overload | TBD | TBD | TBD |

### Key Findings

To be populated with analysis once benchmark data is available.

The comparative analysis will demonstrate:

- Relative efficiency of Arrow-based columnar processing vs traditional
  row-oriented data structures
- Memory allocation patterns and garbage collection impact (Rust vs Go)
- Throughput characteristics under varying load conditions

---

## Additional Resources

- [Detailed Benchmark Results from phase2](benchmarks.md)
- [Phase 1 Benchmark Results](benchmarks-phase1.md)
- [OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md)
- [Project Phases Overview](project-phases.md)
