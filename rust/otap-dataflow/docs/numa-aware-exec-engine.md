# NUMA-Aware Execution Engine: Design Proposal

## Motivation

The df-engine is designed around explicit pipeline topology, bounded resources,
and a thread-per-core execution model. That model gives the controller enough
information to reason about where pipeline instances run, but today the
placement model supports core counts and explicit core sets without
NUMA-aware placement.

On multi-socket hosts, topology-agnostic placement can leave performance on the
table. The current controller chooses cores without NUMA topology, so pipeline
instances and topic-connected pipelines can be placed across NUMA nodes
unintentionally. That cross-node traffic increases latency and memory bandwidth
pressure, especially when it interacts poorly with operator-configured NIC RSS
and IRQ affinity. The same problem appears during scaling and live
reconfiguration: if configuration names specific cores, the operator has to
understand the host topology and the engine has less room to avoid overlap with
pipelines that are already running.

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

- Preserve deterministic fallback behavior when NUMA topology cannot be
  discovered.
- Discover CPU-to-NUMA-node topology as an engine capability, not as a
  receiver-specific or eBPF-specific concern.
- Start with a Linux implementation while keeping the API OS-abstracted so
  Windows and other operating systems can add backends later.
- Evaluate
  [`numaperf`](https://github.com/Skelf-Research/numaperf)
  as a candidate multi-OS discovery layer before committing to a custom backend
  on every supported platform, if it exposes the CPU-to-NUMA-node topology the
  engine needs and not only profiling data.
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
  with its own placement requirements. The initial placement unit is still the
  pipeline; group-wide placement policies can be added later without changing
  the metadata contract.
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

Completeness is part of the contract:

- `complete`: every visible CPU has a known NUMA node and discovery did not
  encounter degraded reads.
- `partial`: at least one visible CPU has a known NUMA node, but discovery
  skipped or degraded some topology, affinity, or cgroup data.
- `unknown`: no trustworthy CPU-to-NUMA mapping is available for placement.

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
possible shared discovery layer if it exposes CPU-to-NUMA topology and can
represent the engine's placement needs without forcing a Linux-only model.

### Placement Model

The configuration should move toward declaring core counts and placement intent,
not exact core ids. For example, a pipeline policy can say that a pipeline needs
`N` cores. The controller then resolves `N` to concrete cores based on:

- the topology provider output,
- cores already assigned to other running pipelines,
- group and pipeline policy scope,
- pipeline graph, topic relationships, and inter-pipeline communication
  topology,
- live reconfiguration rollout constraints,
- optional placement preferences such as same-node, spread-across-nodes, or
  automatic.

The controller should separate placement mechanism from placement policy. The
planner owns visible-core filtering, reservation checks, conflict errors, and
startup/live rollout integration. A placement strategy owns the core-selection
heuristic. Strategy implementations must be deterministic for identical inputs,
must honor the set of reserved cores supplied by the planner, and must only
select from the available visible cores. That contract matters because the same
placement path is used for startup and live control-plane operations.

The initial strategy should be automatic and conservative: for `core_count`, it
prefers compact placement within one NUMA node when enough unreserved cores are
available there, then falls back to deterministic visible-core order. This
NUMA-packing default is chosen for intra-pipeline cache and memory locality.
Balancing across nodes, graph-aware placement, or group-level policies can be
added as additional strategies without changing planner call sites or the
metadata contract.

The initial implementation does not treat a pipeline group as an implicit
same-NUMA placement unit. Multiple pipelines in one group still receive
pipeline-level placements, while the engine-level reservation model prevents
exclusive `core_count` and `core_set` allocations from silently overlapping.
Future group-level policies can build on the same strategy interface if the
configuration model needs that behavior.

This is related to the direction in
[#2155](https://github.com/open-telemetry/otel-arrow/issues/2155) and
[#1837](https://github.com/open-telemetry/otel-arrow/issues/1837): operators
should be able to configure how many cores a pipeline needs, while the engine
optimizes actual placement from the pipelines already running.

During the transition, explicit-core configuration remains valid. When an
operator names concrete cores, the controller can preserve those assignments and
annotate them with discovered NUMA metadata rather than forcing an immediate
move to core-count-only configuration.

### Compatibility

This changes `core_count` placement semantics. A `core_count` pipeline is now
resolved by the controller as an exclusive placement: it avoids cores explicitly
claimed by `core_set` pipelines and cores already selected for other
`core_count` pipelines. `all_cores` remains shared.

When NUMA topology is known, the controller prefers compact same-node placement.
When topology is unknown, placement falls back to deterministic visible-core
selection while keeping the same reservation rules.

If the controller cannot find enough unreserved visible cores, startup or live
update fails instead of silently overlapping pipelines or shrinking the
placement.

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

Live reconfiguration should use the same planner and strategy contract as
startup. A scale-only update can change the requested core count for one
pipeline, then ask the controller to produce a new placement snapshot. The
rollout logic can decide whether the change is a no-op, an in-place resize, or
a replacement. The first implementation does not need to support arbitrary
migration of running tasks; it only needs a design path that does not hard-code
static core lists forever.

### Placement Metadata Contract

Engine subsystems should consume resolved placement through a controller-owned
snapshot rather than rediscover topology independently. The snapshot should be
stable for the lifetime of a running placement generation and should include:

- pipeline group id,
- pipeline id,
- pipeline node id or component id, when applicable,
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
- Evaluate whether `numaperf` exposes the CPU-to-NUMA topology required by the
  provider abstraction and the engine's placement needs.
- Add placement preferences to resource policies after the core-count model is
  established.
- Extend live reconfiguration so scale changes can recalculate placement and
  minimize overlap with already-running pipelines.
- Feed placement snapshots into admission, topic scheduling, and autoscaling
  decisions.
- Benchmark representative multi-NUMA ingestion workloads with and without
  topology-aware placement.
