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
load conditions. The engine uses a **thread-per-core architecture** where each
configured core runs an independent pipeline instance with dedicated memory,
eliminating lock contention and context switching overhead. This share-nothing
design enables throughput to scale linearly with the number of configured cores,
supports CPU affinity and NUMA-aware memory placement, and allows workload
isolation across tenants or signals. For detailed technical documentation, see
the [OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md) and
[Phase 2 Design](phase2-design.md).

This document presents a curated set of key performance metrics across different
load scenarios and test configurations. For the complete set of automated
performance tests (continuous, nightly, saturation, idle state, and binary size
benchmarks), see [Detailed Benchmark
Results](benchmarks.md#current-performance-results).

### Test Environment

All performance tests are executed on a dedicated bare-metal compute instance
with the following specifications:

- **CPU**: 64 physical cores / 128 logical cores
  (x86-64, 2 [NUMA][numa] nodes)
- **Memory**: 512 GB RAM
- **Platform**: Oracle Bare Metal Instance
- **OS**: Ubuntu 24.04

[numa]: https://en.wikipedia.org/wiki/Non-uniform_memory_access
[otlp-grpc]: https://opentelemetry.io/docs/specs/otlp/#otlpgrpc

This consistent, high-performance environment ensures reproducible results and
allows for comprehensive testing across various CPU core configurations (1, 4,
and 8 cores etc.) by constraining the OTel Arrow dataflow engine to specific
core allocations.

### Performance Metrics

#### Idle State Performance

Baseline resource consumption with no active telemetry traffic, measured after
startup stabilization over a 60-second period. The engine's internal telemetry
(self-monitoring metrics, health checks) remains active — this measures the
overhead of a fully operational but unloaded engine.

| Configuration | CPU Utilization | Memory Usage |
| ------------- | --------------- | ------------ |
| Single Core   | 0.1%            | 27 MB        |

*Note: CPU utilization is normalized to total system capacity. Memory usage
is the Docker container's cgroup memory (`container.memory.usage` from
`docker stats`). The engine also exposes a `memory_rss` metric via its
Prometheus endpoint for process-level RSS tracking.*

This validates that the engine maintains a minimal resource footprint when idle,
ensuring efficient operation in environments with variable telemetry loads.

#### Standard Load Performance (Single Core)

Resource utilization at 100,000 log records per second (100K logs/sec) on a
single CPU core. Tests are conducted with different batch sizes to demonstrate
the impact of batching on performance.

**Test Parameters:**

- Ingress: ~100 MB/s (100,000 log records/second x ~1 KB average record size)
- Egress: varies by batch size (see Network Out column)
- Batch sizes tested: 10, 100, 512, 1024, 4096, and 8192 records per request
- Test duration: 60 seconds

This wide range of batch sizes evaluates performance across diverse deployment
scenarios. Small batches (10-100) represent edge collectors or real-time
streaming requirements, while large batches (1024-8192) represent gateway
collectors and high-throughput aggregation points. This approach ensures a fair
assessment, highlighting both the overhead for small batches and the significant
efficiency gains inherent to Arrow's columnar format at larger batch sizes.

##### Standard Load - OTAP -> OTAP (Native Protocol)

| Batch Size | CPU Utilization | Memory Usage | Network In | Network Out | Egress Bytes/Log |
| ---------- | --------------- | ------------ | ---------- | ----------- | ---------------- |
| 512/batch  | 23%             | 20 MB        | 727 KB/s   | 790 KB/s    | 8.7 bytes/log    |

*Note: Only 512/batch has been tested for OTAP at standard load so far. OTAP
batch size variants for small batches are blocked by a known drain-timeout issue
and will be added once resolved.*

This represents the optimal scenario where the dataflow engine operates with its
native protocol end-to-end, eliminating protocol conversion overhead. The ~3x
reduction in CPU utilization and ~3.6x reduction in network egress compared to
OTLP demonstrates the efficiency of Arrow's columnar format with delta
dictionary encoding.

##### Standard Load - OTLP -> OTLP (Standard Protocol)

*OTLP is [gRPC][otlp-grpc] with Protobuf encoding, no TLS.*

| Batch Size | CPU Utilization | Memory Usage | Network In | Network Out | Egress Bytes/Log |
| ---------- | --------------- | ------------ | ---------- | ----------- | ---------------- |
| 10/batch   | 68%*            | 18 MB        | 4.0 MB/s   | 4.2 MB/s    | 133 bytes/log    |
| 100/batch  | 67%             | 16 MB        | 4.6 MB/s   | 4.7 MB/s    | 51 bytes/log     |
| 512/batch  | 65%             | 17 MB        | 2.7 MB/s   | 2.9 MB/s    | 31 bytes/log     |
| 1024/batch | 65%             | 19 MB        | 2.5 MB/s   | 2.6 MB/s    | 28 bytes/log     |
| 4096/batch | 57%             | 22 MB        | 2.3 MB/s   | 2.4 MB/s    | 26 bytes/log     |
| 8192/batch | 54%             | 30 MB        | 2.2 MB/s   | 2.4 MB/s    | 26 bytes/log     |

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

| Protocol | Max Throughput | CPU Utilization | Memory Usage |
| --- | --- | --- | --- |
| OTLP -> OTLP (Standard) | ~546K logs/sec | ~95% | ~35 MB |

##### With Processing

Includes an attribute processor to force data materialization. Represents
typical production workloads where collectors perform transformations such as
filtering, attribute enrichment, renaming, or aggregation.

| Protocol | Max Throughput | CPU Utilization | Memory Usage |
| --- | --- | --- | --- |
| OTLP -> OTLP (Standard) | ~125K logs/sec | ~91% | ~29 MB |

*Note: OTAP -> OTAP saturation tests for passthrough and processing modes are
not yet available and will be added in a future update. These results use
NUMA-aware core pinning to isolate the engine from load-generator cache
effects.*

#### Scalability

How throughput scales when adding CPU cores. The thread-per-core
architecture enables near-linear scaling by
eliminating shared-state synchronization overhead.

**Test Parameters:**

- Batch size: 512 records per request
- Protocol: OTLP -> OTLP with attribute processing
- Load: Maximum sustained throughput at each core count

| CPU Cores | Max Throughput  | CPU Utilization | Scaling Efficiency | Memory Usage |
| --------- | --------------- | --------------- | ------------------ | ------------ |
| 1 Core    | ~125K logs/sec  | ~91%            | 100% (baseline)    | ~29 MB       |
| 2 Cores   | ~274K logs/sec  | ~99%            | 110%               | ~51 MB       |
| 4 Cores   | ~495K logs/sec  | ~93%            | 99%                | ~99 MB       |
| 8 Cores   | ~936K logs/sec  | ~91%            | 94%                | ~211 MB      |
| 16 Cores  | ~1.74M logs/sec | ~90%            | 87%                | ~480 MB      |

*\* These results use NUMA-aware core pinning (PR #2997) which provides more
realistic isolated measurements than the previous non-NUMA runs. Scaling
efficiency is significantly better with NUMA-aware pinning because the
load-generator no longer contends with the engine for L3 cache.*

Scaling Efficiency = (Throughput at N cores) / (N × Single-core throughput)

The average scaling efficiency across all multi-core tests is **97%**, confirming
near-linear scalability of the thread-per-core architecture.

---

## Additional Resources

- [Detailed Benchmark Results from phase2](benchmarks.md)
- [Phase 1 Benchmark Results](benchmarks-phase1.md)
- [OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md)
- [Project Phases Overview](project-phases.md)
