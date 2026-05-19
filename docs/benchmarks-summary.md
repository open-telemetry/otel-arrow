# OpenTelemetry Arrow Performance Summary

## Overview

The OpenTelemetry Arrow (OTel Arrow) project is building an end-to-end,
Arrow-native telemetry pipeline in Rust. Phase 1 reduced collector-to-collector
network traffic using the OTAP wire protocol and is complete. Phase 2 — the
focus of this document — extends Arrow's columnar representation through the
entire in-process pipeline, delivering large gains in CPU efficiency on top of
the existing network savings.

**Status:** Phase 2 is actively under development. The dataflow engine is not
yet a stable, production-ready release — APIs, configuration, and component
coverage are still evolving. The performance characteristics documented here
are real and measured on every commit through automated benchmarks, but
readers evaluating OTel Arrow for production adoption should plan for
continued change until Phase 2 reaches a stable milestone.

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

The headline single-core numbers (multi-core scaling appears further below):

- **Idle footprint:** ~14 MB of memory and 0.06% of one core.
- **100K logs/sec** (typical production load) sustained on one core at
  **~23% CPU on OTAP** end-to-end, or **~65% CPU on OTLP** end-to-end.
- **Peak throughput on one core, pass-through (forwarding only):**
  **~2.64 million logs/sec on OTAP** vs ~625K logs/sec on OTLP — **~4.2x**.
- **Peak throughput on one core, with attribute processing:**
  **~2.58 million logs/sec on OTAP** vs ~360K logs/sec on OTLP — **~7.2x**.
  Adding the processor costs OTAP less than 3% of its throughput, but costs
  OTLP roughly 42%, because Arrow's columnar layout enables in-place
  processing without deserialization.
- **Multi-core scaling:** **~85–94% scaling efficiency** at 2, 4, and 8 cores
  (1-core baseline runs at fully saturated 100% CPU), with memory growth of
  roughly ~30 MB per added core. Capacity planning reduces to multiplication.

The sections that follow back each of these numbers with full test
parameters and conditions. For the complete set of automated performance
tests (continuous, nightly, saturation, idle state, and binary size
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
supports comprehensive testing across CPU-core configurations (1, 2, 4, and 8
cores) by constraining the engine to specific core allocations.

### Performance Metrics

#### Idle State Performance (Single Core)

Baseline resource consumption with no active telemetry traffic, measured after
startup stabilization over a 60-second period. The engine's internal telemetry
(self-monitoring metrics, health checks) remains active, so this measures the
overhead of a fully operational but unloaded engine.

| CPU Utilization | Memory Usage |
| --------------- | ------------ |
| 0.06%           | 14 MB        |

*Note: CPU utilization is normalized to total system capacity (128 logical
CPUs), so 0.06% corresponds to roughly 0.08 of one core — effectively idle.
Memory usage is the Docker container's cgroup memory
(`container.memory.usage` from `docker stats`); the engine also exposes a
process-level `memory_rss` metric via its Prometheus endpoint, matching what
`ps` or `htop` would report.*

This validates that the engine maintains a minimal resource footprint when
idle, ensuring efficient operation in environments with highly variable
telemetry loads — including edge agents that spend most of their time near
zero load.

#### Standard Load Performance (Single Core)

Resource utilization at 100,000 log records per second (100K logs/sec) on a
single CPU core. This is a representative production-volume load for many
edge and gateway deployments.

**Test Parameters:**

- Ingress: ~100 MB/s (100,000 log records/second × ~1 KB average record size)
- Batch size: 512 records per request (the OpenTelemetry SDK default, and the
  most common production batch size)
- Test duration: 60 seconds
- *OTLP is [gRPC][otlp-grpc] with Protobuf encoding, no TLS.*

| Protocol                | CPU Utilization | Memory Usage | Egress Bytes/Log  |
| ----------------------- | --------------- | ------------ | ----------------- |
| OTAP -> OTAP (Native)   | **23%**         | 20 MB        | **8.7 bytes/log** |
| OTLP -> OTLP (Standard) | 65%             | 17 MB        | 31 bytes/log      |

At 100K logs/sec, end-to-end OTAP delivers a **~2.8x reduction in CPU
utilization** and a **~3.6x reduction in network egress** compared to OTLP.
Per-log network egress drops from 31 bytes (OTLP) to 8.7 bytes (OTAP) — a
direct result of Arrow's columnar layout with delta-dictionary encoding.
Memory footprint stays under 25 MB in either configuration.

For batch-size sensitivity (10, 100, 512, 1024, 4096, 8192 records per
request) across the same load, see the [detailed standard-load batch-size
results](benchmarks.md#5-normal-load-with-batch-size-variations).

#### Saturation Performance (Single Core)

Maximum throughput a single CPU core can sustain at full utilization. This
establishes the baseline "unit of capacity" for capacity planning and exposes
the raw efficiency of the engine and the wire protocol.

**Test Parameters:**

- Payload: realistic ~300-byte log records (the OpenTelemetry semantic
  conventions data shape), identical to the standard-load tests above for
  direct comparability
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
| OTLP -> OTLP (Standard) | ~625K logs/sec      | ~100%           | ~27 MB       |
| OTAP -> OTAP (Native)   | **~2.64M logs/sec** | ~100%           | ~47 MB       |

OTAP pass-through is **~4.2x faster than OTLP** on the same core. A single core
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
deserialization. The equivalent OTLP path loses ~42% of its throughput (625K ->
360K logs/sec) to the deserialize / mutate / reserialize cycle that row-oriented
Protobuf requires.

The practical implication for capacity planning: **one OTel Arrow core
processing OTAP traffic replaces roughly seven OTLP cores** doing the same
work.

#### Multi-Core Scalability

How throughput scales as CPU cores are added. The thread-per-core, share-nothing
architecture enables near-linear scaling because each core runs an independent
pipeline with no cross-core locks or shared mutable state.

**Test Parameters:**

- Batch size: 512 records per request
- Protocol: OTLP -> OTLP with attribute processing
- Payload: static 1 KB log bodies drawn from a pool of 512 unique templates
  (realistic ~3:1 compression ratio, exercising the serialization and network
  path more heavily than the smaller ~300-byte payload used in the
  single-core saturation table above)
- Load: Maximum sustained throughput at each core count
- NUMA-aware core pinning isolates the engine cores from load-generator and
  backend cores

| CPU Cores | Max Throughput  | CPU Utilization | Scaling Efficiency | Memory Usage |
| --------- | --------------- | --------------- | ------------------ | ------------ |
| 1 Core    | ~144K logs/sec  | ~100%           | 100% (baseline)    | ~31 MB       |
| 2 Cores   | ~266K logs/sec  | ~91%            | 92%                | ~52 MB       |
| 4 Cores   | ~543K logs/sec  | ~95%            | 94%                | ~98 MB       |
| 8 Cores   | ~980K logs/sec  | ~91%            | 85%                | ~211 MB      |

Scaling Efficiency = (Throughput at N cores) / (N × Single-core throughput).

Across the 2-, 4-, and 8-core configurations scaling efficiency holds at
**~85–94%**, and the 1-core baseline runs at a fully saturated 100% CPU.
Memory grows linearly at roughly **~30 MB per added core**, making capacity
planning a straightforward multiplication. Validation of scaling beyond 8
cores is an active workstream — pushing those runs to engine saturation
requires removing a load-generator ceiling on the test harness, not changes
to the engine itself.

---

## Additional Resources

- [Detailed Benchmark Results from phase2](benchmarks.md)
- [Phase 1 Benchmark Results](benchmarks-phase1.md)
- [OTAP Dataflow Engine Documentation](../rust/otap-dataflow/README.md)
- [Project Phases Overview](project-phases.md)
