# OpenTelemetry Arrow Performance Summary

## Overview

The OpenTelemetry Arrow (OTel Arrow) project is building an end-to-end,
Arrow-native telemetry pipeline in Rust. Phase 1 reduced collector-to-collector
network traffic using the OTAP wire protocol. Phase 2 — currently in progress —
extends Arrow's columnar representation through the entire in-process pipeline,
delivering large gains in CPU efficiency on top of the existing network
savings.

The OTel Arrow dataflow engine is designed for predictable performance and
efficient resource use across the full load spectrum, from near-idle edge
agents to saturated gateway collectors. It uses a **thread-per-core,
share-nothing architecture**: each configured core runs an independent pipeline
instance with its own memory, eliminating cross-core locks and context-switch
overhead. This design enables throughput to scale near-linearly with cores,
supports CPU affinity and NUMA-aware memory placement, and allows workload
isolation across tenants or signals. For detailed technical documentation, see
the [OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md) and
[Phase 2 Design](phase2-design.md).

### OTLP vs OTAP: what the comparisons below mean

Two wire protocols are referenced throughout this document:

- **OTLP** — the standard OpenTelemetry Protocol. Row-oriented Protobuf over
  gRPC. This is what every OpenTelemetry SDK and the OpenTelemetry Collector
  speak today, and is the baseline most existing deployments use.
- **OTAP** — the OpenTelemetry Arrow Protocol. Columnar Apache Arrow records
  over gRPC, with delta-dictionary encoding. The native protocol of this
  engine. OTAP is a strict superset of OTLP semantics; the same data, on the
  wire in a far more efficient shape.

OTel Arrow speaks both. The engine can be deployed as a drop-in OTLP-only
component (no client changes required), as an OTLP-in / OTAP-out gateway, or
fully end-to-end OTAP for maximum efficiency. The performance numbers below
quantify what each of those choices costs and saves.

### At a glance

All numbers below are for a **single CPU core** — not a node, not a server,
one core:

- **Idle footprint:** ~27 MB of memory and 0.1% of one core.
- **100K logs/sec** (typical production load) sustained on one core at
  **~23% CPU on OTAP** end-to-end, or **~65% CPU on OTLP** end-to-end.
- **Peak throughput on one core, pass-through (forwarding only):**
  **~2.64 million logs/sec on OTAP** vs ~607K logs/sec on OTLP — **~4.3x**.
- **Peak throughput on one core, with attribute processing:**
  **~2.58 million logs/sec on OTAP** vs ~360K logs/sec on OTLP — **~7.2x**.
  Adding the processor costs OTAP less than 3% of its throughput, but costs
  OTLP roughly 40%, because Arrow's columnar layout enables in-place
  processing without deserialization.
- **Scaling:** ~97% average scaling efficiency from 1 to 16 cores, with
  memory growth of roughly ~30 MB per added core. Capacity planning reduces
  to multiplication.

This document presents a curated set of key performance metrics across the
load scenarios that matter for capacity planning. For the complete set of
automated performance tests (continuous, nightly, saturation, idle state, and
binary size benchmarks), see [Detailed Benchmark
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
supports comprehensive testing across CPU-core configurations (1, 4, 8, and 16
cores) by constraining the engine to specific core allocations.

### Performance Metrics

#### Idle State Performance (Single Core)

Baseline resource consumption with no active telemetry traffic, measured after
startup stabilization over a 60-second period. The engine's internal telemetry
(self-monitoring metrics, health checks) remains active, so this measures the
overhead of a fully operational but unloaded engine.

| CPU Utilization | Memory Usage |
| --------------- | ------------ |
| 0.1%            | 27 MB        |

*Note: CPU utilization is normalized to total system capacity (128 logical
CPUs), so 0.1% corresponds to roughly 0.13 of one core — effectively idle.
Memory usage is the Docker container's cgroup memory
(`container.memory.usage` from `docker stats`). The engine also exposes a
process-level `memory_rss` metric via its Prometheus endpoint for direct
comparison with `ps` / `htop`.*

This validates that the engine maintains a minimal resource footprint when
idle, ensuring efficient operation in environments with highly variable
telemetry loads — including edge agents that spend most of their time near
zero load.

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

This represents the optimal scenario where the dataflow engine operates with its
native protocol end-to-end, eliminating protocol conversion overhead. At
512/batch, OTAP delivers a **~3x reduction in CPU utilization** and a **~3.6x
reduction in network egress** compared to OTLP at the same batch size. Per-log
network egress drops from 31 bytes (OTLP) to 8.7 bytes (OTAP) — a direct result
of Arrow's columnar layout with delta-dictionary encoding.

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

Maximum throughput a single CPU core can sustain at full utilization. This
establishes the baseline "unit of capacity" for capacity planning and exposes
the raw efficiency of the engine and the wire protocol.

**Test Parameters:**

- Payload: `semantic_conventions` (~300 byte logs) — identical to the standard
  load tests for direct comparability
- Load: Continuously increased until the engine core is fully saturated
- Test duration: 60 seconds at maximum load
- NUMA-aware core pinning: the engine runs on one NUMA node while the load
  generator and backend run on a separate NUMA node, eliminating L3 cache
  contention and producing isolated, repeatable measurements

##### Pass-through Mode

Forwarding without data transformation. Represents the minimum engine overhead
for load-balancing and routing use cases, where the engine does not need to
materialize the in-memory representation of each record.

| Protocol                | Max Throughput      | CPU Utilization | Memory Usage |
| ----------------------- | ------------------- | --------------- | ------------ |
| OTLP -> OTLP (Standard) | ~607K logs/sec      | ~100%           | ~30 MB       |
| OTAP -> OTAP (Native)   | **~2.64M logs/sec** | ~100%           | ~45 MB       |

OTAP pass-through is **~4.3x faster than OTLP** on the same core. A single core
sustains over **2.6 million log records per second** end-to-end in the native
protocol.

##### With Processing

Includes an attribute processor that renames an attribute, forcing the engine
to fully materialize each record. Represents typical production workloads where
collectors perform transformations such as filtering, attribute enrichment,
renaming, or aggregation.

| Protocol                | Max Throughput      | CPU Utilization | Memory Usage |
| ----------------------- | ------------------- | --------------- | ------------ |
| OTLP -> OTLP (Standard) | ~360K logs/sec      | ~100%           | ~23 MB       |
| OTAP -> OTAP (Native)   | **~2.58M logs/sec** | ~100%           | ~48 MB       |

OTAP with attribute processing is **~7.2x faster than OTLP** on the same core.
Equally important is how little the processing step costs in OTAP: adding the
attribute processor reduces OTAP throughput by less than 3% (2.64M -> 2.58M
logs/sec), because Arrow's columnar layout enables in-place processing without
deserialization. The equivalent OTLP path loses ~40% of its throughput (607K ->
360K logs/sec) to the deserialize / mutate / reserialize cycle that row-oriented
Protobuf requires.

The practical implication for capacity planning: **one OTel Arrow core
processing OTAP traffic replaces roughly seven OTLP cores** doing the same
work.

#### Scalability

How throughput scales as CPU cores are added. The thread-per-core, share-nothing
architecture enables near-linear scaling because each core runs an independent
pipeline with no cross-core locks or shared mutable state.

**Test Parameters:**

- Batch size: 512 records per request
- Protocol: OTLP -> OTLP with attribute processing
- Payload: static 1 KB log bodies drawn from a pool of 512 unique templates
  (realistic ~3:1 compression ratio, exercising the serialization and network
  path more heavily than the smaller `semantic_conventions` payload used in
  the single-core saturation table above)
- Load: Maximum sustained throughput at each core count
- NUMA-aware core pinning isolates the engine cores from load-generator and
  backend cores

| CPU Cores | Max Throughput  | CPU Utilization | Scaling Efficiency | Memory Usage |
| --------- | --------------- | --------------- | ------------------ | ------------ |
| 1 Core    | ~125K logs/sec  | ~91%            | 100% (baseline)    | ~29 MB       |
| 2 Cores   | ~274K logs/sec  | ~99%            | 110%               | ~51 MB       |
| 4 Cores   | ~495K logs/sec  | ~93%            | 99%                | ~99 MB       |
| 8 Cores   | ~936K logs/sec  | ~91%            | 94%                | ~211 MB      |
| 16 Cores  | ~1.74M logs/sec | ~90%            | 87%                | ~480 MB      |

Scaling Efficiency = (Throughput at N cores) / (N × Single-core throughput).
Values above 100% reflect cache-warming and measurement noise; values at 16
cores are bounded in part by the load generator's ability to push traffic
rather than by the engine itself.

The average scaling efficiency across all multi-core tests is **97%**,
confirming the near-linear scalability of the thread-per-core architecture.
Memory grows linearly at roughly **~30 MB per additional core**, making
capacity planning a straightforward multiplication.

---

## Additional Resources

- [Detailed Benchmark Results from phase2](benchmarks.md)
- [Phase 1 Benchmark Results](benchmarks-phase1.md)
- [OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md)
- [Project Phases Overview](project-phases.md)
