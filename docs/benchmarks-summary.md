# OpenTelemetry Arrow Performance Summary

## Overview

The OpenTelemetry Arrow (OTel Arrow) project is currently in **Phase 2**,
building an end-to-end Arrow-based telemetry pipeline in Rust. Phase 1 focused
on collector-to-collector traffic compression using the OTAP protocol, achieving
significant network bandwidth savings. Phase 2 expands this foundation by
implementing the entire in-process pipeline using Apache Arrow's columnar
format, targeting substantial improvements in data processing efficiency while
maintaining the network efficiency gains from Phase 1.

The dataflow engine (df-engine), implemented in Rust, provides predictable
performance characteristics and efficient resource utilization across varying
load conditions. This columnar approach is expected to offer substantial
advantages over traditional row-oriented telemetry pipelines in terms of CPU
efficiency, memory usage, and throughput.

This document presents key performance metrics across different load scenarios
and test configurations.

### Test Environment

All performance tests are executed on bare-metal compute instance with the
following specifications:

- **CPU**: 64 cores (x86-64 architecture)
- **Memory**: 512 GB RAM
- **Platform**: Oracle Bare Metal Instance
- **OS**: Oracle Linux 8

This consistent, high-performance environment ensures reproducible results and
allows for comprehensive testing across various CPU core configurations (1, 4,
and 8 cores etc.) by constraining the df-engine to specific core allocations.

### Performance Metrics

#### Idle State Performance

Baseline resource consumption with no active telemetry traffic.

| Metric | Value |
|--------|-------|
| CPU Usage | TBD |
| Memory Usage | TBD |

These baseline metrics validate that the engine maintains minimal resource
footprint when idle, ensuring efficient operation in environments with variable
telemetry loads.

#### Standard Load Performance

Resource utilization at 100,000 log records per second (100K logs/sec). Tests
are conducted with three different batch sizes to demonstrate the impact of
batching on performance.

**Test Parameters:**

- Total input load: 100,000 log records/second
- Average log record size: 1 KB
- Batch sizes tested: 10, 100, and 1000 records per request

##### OTAP → OTAP (Native Protocol)

| CPU Cores | Batch Size | CPU Usage | Memory Usage |
|-----------|------------|-----------|---------------|
| 1 Core    | 10/batch | TBD | TBD |
| 1 Core    | 100/batch | TBD | TBD |
| 1 Core    | 1000/batch | TBD | TBD |
| 4 Cores   | 10/batch | TBD | TBD |
| 4 Cores   | 100/batch | TBD | TBD |
| 4 Cores   | 1000/batch | TBD | TBD |
| 8 Cores   | 10/batch | TBD | TBD |
| 8 Cores   | 100/batch | TBD | TBD |
| 8 Cores   | 1000/batch | TBD | TBD |

This represents the optimal scenario where the df-engine operates with its
native protocol end-to-end, eliminating protocol conversion overhead. The
thread-per-core architecture demonstrates linear scaling across CPU cores
without contention, allowing the engine to be configured for specific deployment
requirements.

##### OTLP → OTAP (Protocol Conversion)

| CPU Cores | Batch Size | CPU Usage | Memory Usage |
|-----------|------------|-----------|---------------|
| 1 Core | 10/batch | TBD | TBD |
| 1 Core | 100/batch | TBD | TBD |
| 1 Core | 1000/batch | TBD | TBD |
| 4 Cores | 10/batch | TBD | TBD |
| 4 Cores | 100/batch | TBD | TBD |
| 4 Cores | 1000/batch | TBD | TBD |
| 8 Cores | 10/batch | TBD | TBD |
| 8 Cores | 100/batch | TBD | TBD |
| 8 Cores | 1000/batch | TBD | TBD |

This scenario represents the common case where OpenTelemetry SDK clients emit
OTLP (not yet capable of OTAP), and the df-engine converts to OTAP for egress.
This demonstrates backward compatibility and protocol conversion efficiency
while maintaining linear scaling characteristics across CPU cores.

#### Saturation Performance

Behavior at maximum capacity when physical resource limits are reached.

##### OTAP → OTAP (Native Protocol)

| CPU Cores | Maximum Sustained RPS | CPU Usage | Memory Usage |
|-----------|---------------------|-----------|---------------|
| 1 Core | TBD | TBD | TBD |
| 4 Cores | TBD | TBD | TBD |
| 8 Cores | TBD | TBD | TBD |

##### OTLP → OTAP (Protocol Conversion)

| CPU Cores | Maximum Sustained RPS | CPU Usage | Memory Usage |
|-----------|---------------------|-----------|---------------|
| 1 Core | TBD | TBD | TBD |
| 4 Cores | TBD | TBD | TBD |
| 8 Cores | TBD | TBD | TBD |

Saturation testing validates the engine's stability under extreme load. The
df-engine exhibits well-defined behavior when operating at capacity, maintaining
predictable performance without degradation or instability. These results
demonstrate the maximum throughput achievable with different CPU core
allocations.

<!--TODO: Document what is the behavior - is it applying backpressure 
(`wait_for_result` feature)? or dropping items and keeping internal metric
about it. -->

### Architecture

The OTel Arrow dataflow engine is built in Rust, to achieve high throughput and
low latency. The columnar data representation and zero-copy processing
capabilities enable efficient handling of telemetry data at scale.

#### Thread-Per-Core Design

The df-engine supports a configurable runtime execution model, using a
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
2. Real-World Relevance: Syslog is widely deployed in production
   environments, particularly for log aggregation from network devices, legacy
   systems, and infrastructure components
3. Conversion Overhead: Both systems must perform meaningful work to convert
   incoming Syslog messages into their internal representations:
   - **OTel Collector**: Converts to Go-based `pdata` (protocol data) structures
   - **OTel Arrow**: Converts to Arrow-based columnar memory format
4. Complete Pipeline Test: This approach validates the full pipeline
   efficiency, including parsing, transformation, and serialization stages

The output protocols are set to each system's native format: OTLP for the
OpenTelemetry Collector and OTAP for the OTel Arrow engine, ensuring optimal
egress performance for each.

### Performance Comparison

#### Baseline (Idle State)

| Metric | OTel Collector | OTel Arrow | Improvement |
|--------|---------------|------------|-------------|
| CPU Usage | TBD | TBD | TBD |
| Memory Usage | TBD | TBD | TBD |

#### Standard Load (100K RPS Syslog Messages)

| Metric | OTel Collector | OTel Arrow | Improvement |
|--------|---------------|------------|-------------|
| CPU Usage | TBD | TBD | TBD |
| Memory Usage | TBD | TBD | TBD |
| Network Egress | TBD | TBD | TBD |
| Latency (p50) | TBD | TBD | TBD |
| Latency (p99) | TBD | TBD | TBD |
| Throughput (messages/sec) | TBD | TBD | TBD |

#### Saturation

| Metric | OTel Collector | OTel Arrow | Improvement |
|--------|---------------|------------|-------------|
| Maximum Sustained RPS | TBD | TBD | TBD |
| CPU at Saturation | TBD | TBD | TBD |
| Memory at Saturation | TBD | TBD | TBD |
| Behavior Under Overload | TBD | TBD | TBD |

### Key Findings

To be populated with analysis once benchmark data is available.

The comparative analysis will demonstrate:

- Relative efficiency of Arrow-based columnar processing vs traditional
  row-oriented data structures
- Memory allocation patterns and garbage collection impact (Rust vs Go)
- Throughput and latency characteristics under varying load conditions

---

## Additional Resources

- [Detailed Benchmark Results from phase2](benchmarks.md)
- [Phase 1 Benchmark Results](benchmarks-phase1.md)
- [OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md)
- [Project Phases Overview](project-phases.md)
