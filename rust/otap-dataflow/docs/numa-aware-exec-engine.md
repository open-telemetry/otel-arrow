# NUMA-Aware Execution Engine: Design Proposal

> Status: Proposed. This is a design proposal for review. It describes a
> long-term execution-engine direction and intended behavior; it does not
> assume any of it is built yet.

## Motivation

The df-engine is designed around explicit pipeline topology, bounded resources,
and a thread-per-core execution model. That model gives the controller enough
information to reason about where pipeline instances run, but today the
placement model is still mostly expressed as fixed core assignment.

On multi-socket hosts, fixed placement can leave performance on the table. A
pipeline instance may allocate memory on one NUMA node while a receiver,
processor, exporter, topic hop, or network interrupt executes on another. That
cross-node traffic increases latency and memory bandwidth pressure. The same
problem appears during scaling and live reconfiguration: if configuration names
specific cores, the operator has to understand the host topology and the engine
has less room to avoid overlap with pipelines that are already running.

This proposal introduces NUMA topology discovery and a controller-owned
placement model. Configuration should express the requested execution shape,
such as the number of cores for a pipeline, while the engine resolves actual
core placement dynamically from the host topology, current pipeline layout, and
runtime constraints.

## Review Focus

This proposal is primarily asking for feedback on:

1. Whether NUMA discovery should be modeled as an engine/controller capability
   rather than as part of any single receiver or load-balancing feature.
2. Whether the long-term placement model should move toward dynamic
   engine-resolved core assignment from per-pipeline core counts.
3. Whether the proposed abstraction is sufficient for Linux first while leaving
   room for Windows and other operating systems.
4. Whether the placement metadata contract is sufficient for engine subsystems
   that need core and NUMA placement information.
5. Whether the phased plan aligns with the engine configuration model and live
   reconfiguration direction.

## Goals

- Preserve current behavior when NUMA discovery or dynamic placement is not
  enabled.
- Discover CPU-to-NUMA-node topology as an engine capability, not as a
  receiver-specific or eBPF-specific concern.
- Start with a Linux implementation while keeping the API OS-abstracted so
  Windows and other operating systems can add backends later.
- Evaluate
  [`numaperf`](https://github.com/Skelf-Research/numaperf)
  as a candidate multi-OS discovery layer before committing to a custom backend
  on every supported platform.
- Allow configuration to specify the number of cores per pipeline while the
  engine resolves actual placement dynamically, following the direction in
  [#2155](https://github.com/open-telemetry/otel-arrow/issues/2155) and
  [#1837](https://github.com/open-telemetry/otel-arrow/issues/1837).
- Minimize cross-NUMA traffic and core overlap when placing new or
  reconfigured pipelines.
- Support live reconfiguration as a long-term controller capability so scale
  changes can recalculate placement without requiring operators to name
  concrete cores.
- Support multiple pipeline groups and pipelines in one engine process, each
  with its own placement requirements.
- Expose placement metadata through a strategy-agnostic engine contract so
  receivers, topic scheduling, admission, observability, and optional
  load-balancing mechanisms can build on the same placement snapshot.

## Non-Goals

- Implementing automatic NIC RSS or IRQ-affinity tuning.
- Moving established TCP connections or already-running tasks between cores.
- Replacing the engine's explicit group, pipeline, node, connection, topic, and
  policy configuration model.
- Guaranteeing NUMA locality when the host, container runtime, or scheduler
  does not provide stable CPU placement.
- Defining socket-specific load balancing, eBPF selector behavior, kernel
  attachment, or Linux reuseport grouping semantics.

## Background

NUMA systems divide CPU cores and memory into nodes. Access to local memory is
faster than access to memory attached to another node. The engine already has
the high-level concepts needed to use this information:

- a root configuration with groups and pipelines,
- pipeline-level policies that can express resource requirements,
- a controller that resolves configuration into executable pipeline instances,
- per-core execution that can keep hot data structures local when placement is
  stable,
- live reconfiguration that can replace, resize, or no-op a logical pipeline.

The missing piece is an engine-owned placement layer that understands host
topology. That layer should let the configuration describe intent and let the
controller choose the concrete cores.

## Proposed Design

### Overview

The work is structured as phases that can land and be reviewed independently:

1. NUMA topology discovery: expose CPU-to-NUMA-node information through an
   engine-owned provider abstraction.
2. Engine placement planning: let the controller allocate core sets for
   pipelines from requested core counts, current pipeline layout, and topology.
3. Placement metadata contracts: publish resolved core and NUMA metadata for
   engine subsystems that need placement information.
4. Live reconfiguration: recalculate placement for scale changes and future
   rollout operations.

The first two phases are useful on their own. They allow dynamic core-count
configuration, engine-resolved placement, and reduced core overlap between
pipelines. Later phases can consume the same placement snapshot without owning
topology discovery or placement decisions.

### Topology Provider

The controller should depend on a small abstraction rather than Linux sysfs
directly:

```rust
trait NumaTopologyProvider {
    fn discover(&self) -> NumaTopology;
}
```

The discovered topology should include:

- CPU id to NUMA node id mapping,
- the set of visible CPUs,
- the set of visible NUMA nodes,
- whether the topology is complete, partial, or unknown.

The initial Linux backend can parse
`/sys/devices/system/node/node*/cpulist`. That path is available on typical
Linux hosts and does not require `libnuma` or `hwloc`. The backend should be
container-safe: when the process runs in a container or under a cgroup CPU
limit, the visible placement set should intersect host topology with the
process's allowed CPU set rather than planning against CPUs the engine cannot
use.

Discovery must degrade safely:

- Non-Linux hosts return an unknown topology until an OS backend is available.
- Unreadable or partially readable system files produce partial or unknown
  topology instead of failing startup.
- Unknown CPU ids remain unknown; paths that need a concrete telemetry value
  may continue to fall back to node `0` to preserve current behavior.
- A single-NUMA host remains valid and simply has no cross-node placement
  choices.

Windows support is an explicit goal for the abstraction. A Windows backend
should use the operating system's processor-group and NUMA APIs rather than
Linux-shaped sysfs concepts. The project should also evaluate `numaperf` as a
possible shared discovery layer if it can represent the engine's placement
needs without forcing a Linux-only model.

### Placement Model

The configuration should move toward declaring core counts and placement intent,
not exact core ids. For example, a pipeline policy can say that a pipeline needs
`N` cores. The controller then resolves `N` to concrete cores based on:

- the topology provider output,
- cores already assigned to other running pipelines,
- group and pipeline policy scope,
- node and connection topology,
- live reconfiguration rollout constraints,
- optional placement preferences such as same-node, spread-across-nodes, or
  automatic.

The default policy should be automatic and conservative. For a single pipeline,
the controller can prefer compact placement within one NUMA node when that
reduces cross-node traffic. For multiple independent pipelines or groups, it can
spread placements across NUMA nodes to avoid hot spots. For pipeline graphs that
communicate through topics, the placement planner can keep strongly coupled
ingest and processing stages on the same node when that is more efficient than
spreading them.

This follows the direction in
[#2155](https://github.com/open-telemetry/otel-arrow/issues/2155) and
[#1837](https://github.com/open-telemetry/otel-arrow/issues/1837): operators
should be able to configure how many cores a pipeline needs, while the engine
optimizes actual placement from the pipelines already running.

### Controller Integration

The controller should own the resolved placement snapshot. Each pipeline
instance should receive the core id and NUMA node id that the controller chose
for that instance. That keeps placement decisions visible in one place and lets
future live reconfiguration compare old and new snapshots before changing
runtime state.

The placement snapshot should be observable. Useful attributes include pipeline
group id, pipeline id, core id, NUMA node id, and placement policy. Metrics
should follow the engine's existing naming style and use low-cardinality
attributes rather than embedding policy or node ids in metric names.

Live reconfiguration should eventually use the same planner. A scale-only update
can change the requested core count for one pipeline, then ask the controller to
produce a new placement snapshot. The rollout logic can decide whether the
change is a no-op, an in-place resize, or a replacement. The first
implementation does not need to support arbitrary migration of running tasks;
it only needs a design path that does not hard-code static core lists forever.

### Placement Metadata Contract

Engine subsystems should consume resolved placement through a controller-owned
snapshot rather than rediscover topology independently. The snapshot should be
stable for the lifetime of a running placement generation and should include:

- pipeline group id,
- pipeline id,
- receiver node id,
- placement generation id,
- assigned core id,
- NUMA node id,
- placement policy,
- whether the topology is complete, partial, or unknown.

The contract should be strategy-agnostic. Receivers, topic scheduling, admission
control, internal telemetry, and optional load-balancing mechanisms can consume
the same placement snapshot, but they should not own topology discovery or
pipeline placement. Socket-specific fields such as bind addresses, protocols,
file descriptors, or kernel grouping identities belong in the consumer design
that needs them, not in the engine placement contract.

## Operational Requirements

NUMA-aware placement only helps when deployment gives the engine stable CPU
placement:

- On bare metal or VMs, operators should align CPU pinning and memory locality
  expectations with the host topology.
- In Kubernetes, strong NUMA locality generally requires Guaranteed pods with
  integer CPU requests/limits, static CPU manager policy, topology manager
  alignment, and node pools where CPU placement is stable.
- Network-locality benefits additionally require NIC RSS and IRQ affinity to
  deliver packets on CPUs near the selected receiver cores.
- On single-NUMA hosts, the same machinery can still reduce core overlap and
  support dynamic core-count configuration, but it cannot provide a locality
  benefit across nodes.

If the runtime cannot discover topology or cannot trust CPU placement, it should
fall back to today's behavior or to topology-agnostic placement rather than
failing startup.

## Relationship to Socket Load Balancing

Socket load balancing is one consumer of the placement metadata contract. That
consumer may add socket-specific fields and platform-specific behavior, but the
execution engine remains the source of truth for topology discovery, placement
planning, and resolved core-to-NUMA metadata. If no socket-level load balancer
is enabled, the engine placement model is still valuable for per-core
execution, topic placement, observability, and future live reconfiguration. For
one socket-specific consumer of this contract, see
[NUMA-Local Reuseport Load Balancing](reuseport-ebpf-numa.md).

## Alternatives Considered

- Keep static core lists in configuration: simple, but it forces operators to
  encode host topology manually and makes live reconfiguration harder.
- Let each receiver discover NUMA topology independently: avoids a controller
  change, but duplicates platform-specific logic and cannot optimize placement
  across pipelines.
- Make a socket load balancer own NUMA discovery: useful for one consumer, but
  it prevents the rest of the engine from using placement information and
  conflicts with the goal of Windows and non-socket consumers.
- Depend immediately on `libnuma` or `hwloc`: these libraries are mature, but
  they add platform and packaging decisions before the engine abstraction is
  settled. They can be reconsidered as backend implementations later.

## Future Work

- Add Windows topology discovery and validate processor-group behavior.
- Evaluate `numaperf` against the provider abstraction and the engine's
  topology needs.
- Add placement preferences to resource policies after the core-count model is
  established.
- Extend live reconfiguration so scale changes can recalculate placement and
  minimize overlap with already-running pipelines.
- Feed placement snapshots into admission, topic scheduling, and autoscaling
  decisions.
- Benchmark representative multi-NUMA ingestion workloads with and without
  topology-aware placement.
