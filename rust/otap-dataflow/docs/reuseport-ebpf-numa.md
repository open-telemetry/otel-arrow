# NUMA-Local Reuseport Load Balancing: Design Proposal

> Status: Proposed. This is a design proposal for review. It describes a
> plan and intended behavior; it does not assume any of it is built yet.

## Motivation

The df-engine runs receiver pipelines on dedicated cores, and those receivers
can bind the same address with `SO_REUSEPORT`. Today the Linux kernel decides
which listener receives each new TCP connection or UDP datagram using its
default reuseport hash. That behavior is simple and correct, but it has two
shortcomings for ingestion workloads:

- With a small number of long-lived TCP connections, the default hash can place
  connections unevenly, leaving some receiver cores idle while others are hot.
- The default hash is not NUMA-aware. On a multi-socket host, a connection
  whose RX interrupt landed on one NUMA node may be handed to a receiver pinned
  to another node, paying cross-node memory traffic for the life of the
  connection.

This document proposes an optional Linux `sk_reuseport` eBPF selector that
chooses sockets for new connections and datagrams using listener placement
metadata resolved by the engine. It is the socket-level companion to
[NUMA-Aware Execution Engine](numa-aware-exec-engine.md), which covers topology
discovery, controller-owned placement, dynamic core-count configuration, and the
listener-group contract consumed here.

## Review Focus

This proposal is primarily asking for feedback on:

1. Whether the eBPF selector should remain an optional consumer of the
   engine's NUMA-aware listener-group contract.
2. Whether the first selector policy should be NUMA-local round-robin with a
   global fallback.
3. Whether the proposed failure behavior is safe: selector attach failure logs
   and continues unless strict mode is enabled.
4. Whether the Linux, Docker, Kubernetes, and AKS operational constraints are
   acceptable for this feature.
5. Whether the build and packaging approach is acceptable for an off-by-default
   Linux feature.

## Goals

- Preserve current behavior exactly when the feature is disabled.
- Improve per-core distribution of new TCP connections across df-engine
  receiver cores.
- Keep new connections NUMA-local when possible on multi-socket Linux hosts.
- Support both TCP and UDP receivers, for example OTLP/gRPC, OTLP/HTTP, OTAP,
  and syslog/CEF.
- Integrate with engine-controller listener-group planning rather than
  discovering placement independently.
- Support multiple pipeline groups and pipelines, each producing its own
  listener-group plans and, therefore, multiple `SO_REUSEPORT` socket groups.
- Leave room for live reconfiguration by treating selector attachment as a
  consumer of the controller's current placement snapshot.
- Stay safe to enable on any build: where eBPF cannot run, the engine continues
  without the selector, using coordinated plain `SO_REUSEPORT` where a
  listener plan exists and the existing independent bind path otherwise.

## Non-Goals

- NUMA topology discovery, core-count configuration, or engine placement
  planning. Those belong to
  [NUMA-Aware Execution Engine](numa-aware-exec-engine.md).
- Rebalancing work already multiplexed above the socket layer, such as HTTP/2
  streams inside one long-lived gRPC connection. The mechanism chooses a socket
  for a *new* connection or datagram only.
- Per-request, per-record, or per-byte balancing.
- Client- or exporter-side connection fan-out.
- Automated NIC/RSS or IRQ-affinity tuning.
- Coordination across separate engine processes.
- Graceful listener migration during rolling restarts. It is sketched as
  future work, not part of this proposal.

## Background

BPF is a Linux facility for loading small, verified programs into the kernel and
attaching them to specific kernel hooks. eBPF is the modern form of that
facility. The program still comes from userspace, but once loaded it runs in the
kernel at the hook point, subject to kernel verification and capability checks.

`SO_REUSEPORT` lets several sockets bind the same address; the kernel collects
them into one reuseport group and load-balances incoming connections across the
group. Linux additionally allows a BPF program of type
`BPF_PROG_TYPE_SK_REUSEPORT` to be attached to the group to pick the destination
socket for each new connection or datagram, overriding the default hash. Such a
program selects a socket from a `BPF_MAP_TYPE_REUSEPORT_SOCKARRAY` and may
consult the NUMA node of the CPU currently handling the packet.

This proposal uses those primitives to express a NUMA-local,
round-robin-within-node policy that the kernel's built-in hash cannot.

In this design, the selector is not a sidecar or separate userspace process. It
is a small eBPF program loaded into the kernel by `df_engine` at startup and
attached to each eligible `SO_REUSEPORT` socket group. Linux invokes it whenever
it needs to choose which socket in that group should receive a new TCP
connection or UDP datagram.

```mermaid
flowchart TD
    Clients[Clients] --> Kernel[Linux kernel]
    Kernel --> Group["SO_REUSEPORT group<br/>same IP:port"]
    Group --> Selector["eBPF sk_reuseport selector<br/>loaded by df_engine"]
    Selector --> R0["Receiver socket<br/>core 0 / NUMA 0"]
    Selector --> R1["Receiver socket<br/>core 1 / NUMA 0"]
    Selector --> R8["Receiver socket<br/>core 8 / NUMA 1"]
    Selector --> R9["Receiver socket<br/>core 9 / NUMA 1"]
```

In Docker or Kubernetes, `df_engine` still loads the selector into the host
kernel from inside the container; the container only needs the required
capabilities and seccomp permissions.

## Prerequisites

The selector consumes the placement metadata contract described in
[NUMA-Aware Execution Engine](numa-aware-exec-engine.md) and extends it with
socket-specific listener-group fields. Before attaching the selector, the
engine must already know:

- which pipeline group, pipeline, and receiver node owns the listener group;
- the effective bind address and protocol;
- any future logical bind-device identity;
- the listener id, file descriptor, core id, and NUMA node for each socket in
  the group;
- whether the listener group was created through the coordinated listener path.

The selector does not perform topology discovery or choose pipeline placement.
It only receives the already-resolved listener metadata and converts it into
BPF maps.

The logical listener-group identity includes
`(pipeline_group_id, pipeline_id, receiver_node_id, bind_address, protocol,
optional bind_device)`. Keying on the complete tuple avoids collisions when two
unrelated receivers in different pipelines happen to share a bind address;
receiver node names are unique only within a pipeline. `bind_device` is included
only to prevent future logical identity collisions if receiver configs expose
device binding. This proposal does not apply `SO_BINDTODEVICE` to bind a
specific network device, so effective kernel reuseport grouping would still be
determined only by `(address, protocol)`.

Because Linux groups all `SO_REUSEPORT` sockets with the same effective
`(address, protocol)` into one kernel reuseport group, the engine must refuse to
coordinate two logically distinct plans that map to the same effective bind
identity rather than attach a partial selector to a shared kernel group. Such
receivers fall back to independent binding.

## Proposed Design

### eBPF NUMA selector

When enabled and supported, the manager attaches a
`BPF_PROG_TYPE_SK_REUSEPORT` selector to each eligible group exactly once,
after all sockets are bound and before any member starts accepting. The
userspace side populates per-group BPF maps describing the socket array and the
per-NUMA contiguous ranges, then attaches the program to the group with
`setsockopt(SO_ATTACH_REUSEPORT_EBPF)`.

The selector is intentionally tiny and never drops traffic: any helper failure
leaves the kernel free to apply its default hash.

### Selection policy

For each new connection or datagram, the selector:

1. Reads the NUMA node of the CPU currently handling the packet.
2. If that node owns at least one listener, picks one socket from the node's
   contiguous sub-range using a per-NUMA round-robin counter. The counter is a
   shared array counter advanced atomically, modulo the range length, so
   selection is globally fair across RX CPUs rather than per-CPU.
3. Otherwise falls back to a global round-robin counter modulo the total socket
   count.

```mermaid
flowchart TD
    C["New connections/datagrams"] --> K["Kernel reuseport group"]
    K --> SEL{"eBPF selector: which NUMA node is the RX CPU on?"}
    SEL -->|node 0| RR0["round-robin among node-0 listeners"]
    SEL -->|node 1| RR1["round-robin among node-1 listeners"]
    SEL -->|unknown or empty| RRG["global round-robin"]
    RR0 --> P0["receiver core on NUMA node 0"]
    RR1 --> P1["receiver core on NUMA node 1"]
    RRG --> PA["any receiver core"]
```

Round-robin is chosen over a hash because hash collisions can leave some
listeners idle when the working set of connections is small. The policy
balances new connections across listeners within the local NUMA node; it does
not rebalance requests, streams inside an existing connection, or bytes.
Long-lived connections stay on the socket first chosen. The atomic counter
requires BPF ISA v3 and therefore Linux 5.12 or newer.

### Engine-level policy alternative

The eBPF selector is one consumer of listener-group placement metadata, not the
only possible balancing strategy. An engine-level policy can also consume the
same controller-owned placement snapshot and remain aligned with the engine
configuration model.

For example, a future policy could select among:

```yaml
load_balancing:
  strategy: kernel # kernel | ebpf_numa | engine
```

The `kernel` strategy leaves selection to the default Linux reuseport hash. The
`ebpf_numa` strategy attaches the selector described here. The `engine` strategy
is reserved for a controller-owned policy that can coordinate placement,
listener-group creation, and live reconfiguration without requiring the socket
selector to own the configuration model.

This proposal does not define arbitrary third-party strategy loading. The near
term goal is to keep the eBPF loader isolated and keep the listener-group
contract strategy-agnostic so the backend remains swappable.

### Proposed configuration

A single user-facing switch is proposed:

- `OTAP_DF_REUSEPORT_EBPF=1` activates coordinated listener planning and
  acquisition end-to-end and, where supported, installs the eBPF selector. On
  builds or platforms without selector support it logs once and continues with
  coordinated plain `SO_REUSEPORT`.
- `OTAP_DF_REUSEPORT_EBPF_STRICT=1` makes selector attach failure abort startup
  instead of the default log-and-continue. It is meaningful only alongside the
  switch above.

The eBPF loader itself is proposed to sit behind a `reuseport-ebpf` Cargo
feature so the default build carries no BPF toolchain or runtime dependency.
With the switch unset, behavior is identical to today: every receiver binds
independently and no coordination, planning, or attach occurs.

### OTLP and OTAP / gRPC connection fan-out

Because the policy balances new TCP connections, OTLP and OTAP over gRPC
benefit only when upstream collectors open multiple connections, for example
via connection max-age or client-side pool fan-out. A single long-lived HTTP/2
connection pins all its RPCs to one receiver core regardless of the selector.
Driving upstream fan-out is a separate benchmarking follow-up.

Upstream spec follow-up: OpenTelemetry should specify exporter sub-channel and
fan-out behavior in SDK and collector OTLP/gRPC exporters so server-side
reuseport balancing has enough connection entropy to work predictably.

## Build and Packaging (proposed)

The Linux eBPF path is feature-gated and inert by default. When built on Linux
with the `reuseport-ebpf` feature, the crate build script compiles the selector
source into a BPF object and makes it available to the loader. The loader:

- opens and loads the BPF object;
- sets the program and attach types for `sk_reuseport` selection;
- populates the socket-array, per-NUMA range, and total-count maps;
- attaches the program to the group with
  `setsockopt(SO_ATTACH_REUSEPORT_EBPF)`.

To keep the feature usable in slim, multi-stage container images, the proposal
is to embed the compiled BPF object into the engine binary and load it from
memory rather than read a build-time path at runtime. The build script resolves
`vmlinux.h` from an explicit override path, a checked-in header for controlled
builds, or optional `bpftool` generation from kernel BTF. A Linux CI job is
proposed to compile the feature so the BPF build and loader stay green.

## Operational Requirements

### Kernel and capabilities

- Linux with `BPF_PROG_TYPE_SK_REUSEPORT` and
  `BPF_MAP_TYPE_REUSEPORT_SOCKARRAY`.
- Kernel 5.12 or newer for the atomic round-robin counter (BPF ISA v3).
- Loading and attaching the selector require `CAP_BPF` plus `CAP_NET_ADMIN` on
  newer kernels, or `CAP_SYS_ADMIN` on older ones. The coordinated plain
  `SO_REUSEPORT` path needs no special privilege.

At a high level, this feature does not require making the whole engine run as
root or giving the whole container broad privileges. The coordinated plain
`SO_REUSEPORT` fallback needs no special permission, and the optional eBPF
selector only needs enough privilege to load and attach that selector. On
kernels and runtimes that support fine-grained BPF capabilities, prefer a
non-root `df_engine` process with only the required capabilities.

For a systemd-managed Linux service, that usually means setting `User=...`,
`AmbientCapabilities=CAP_BPF CAP_NET_ADMIN`, and
`CapabilityBoundingSet=CAP_BPF CAP_NET_ADMIN`, while keeping the rest of the
service hardening profile as narrow as possible. Keep `NoNewPrivileges=yes`
when the capabilities are granted by the service manager before exec; relax it
only if the chosen deployment relies on file capabilities or another post-exec
privilege transition.

Use `CAP_SYS_ADMIN`, `--privileged`, or an all-capabilities container only as a
compatibility fallback for older kernels or container runtimes that cannot grant
`CAP_BPF` / `CAP_NET_ADMIN` cleanly. In non-strict mode, missing permissions
should be treated as an expected fallback to plain `SO_REUSEPORT`, not as a
reason to broaden privileges by default.

Build prerequisites for the feature: clang 12+ with the BPF backend, libbpf
headers, and a `vmlinux.h` for the target, or the opt-in `bpftool` generation
path.

### NUMA locality prerequisites

NUMA locality only holds if packets are received on NUMA-local CPUs. Operators
must align NIC RSS and IRQ affinity so RX queues interrupt on the intended node,
and pin df-engine workers to the same CPU set. On a single-NUMA host the
mechanism still provides per-core distribution but no locality benefit. Useful
checks include `lscpu -e=CPU,NODE`, `/sys/class/net/<iface>/device/numa_node`,
`ethtool -l` / `ethtool -x`, `/proc/interrupts`, and
`/proc/irq/<irq>/smp_affinity_list`.

### Kubernetes / AKS / Docker

Two deployment tiers are expected:

- Coordinated plain `SO_REUSEPORT` works in ordinary containers with no extra
  privileges; the selector hook becomes a logged no-op or a non-strict
  fallback.
- The eBPF selector additionally requires a Linux build with the feature, a
  supported kernel, and permission to call `bpf()` and attach the program.

The anticipated container shape grants `CAP_BPF` and `CAP_NET_ADMIN`, or
`CAP_SYS_ADMIN` on older kernels, and permits the `bpf()` syscall under seccomp:

```yaml
securityContext:
  capabilities:
    add: ["BPF", "NET_ADMIN"]
  seccompProfile:
    type: Unconfined
```

Clusters enforcing the `restricted` Pod Security Standard normally reject these
capabilities and unconfined seccomp, so the workload is intended for an
explicitly trusted namespace or dedicated node pool. Sandboxed runtimes that do
not pass `bpf()` through to the host kernel are out of scope. Attaching the
selector yields per-core placement inside the pod's reuseport group; the
NUMA-locality benefit additionally needs a multi-NUMA node, stable CPU
placement, and node-level RSS / IRQ alignment. Ordinary multi-tenant pods should
expect the fallback or per-core tier, not guaranteed NUMA-local placement.

## Limitations

- Linux-only and experimental; off by default.
- Balances new connections/datagrams, not requests, streams, or bytes.
- OTLP and OTAP over gRPC need upstream connection fan-out to benefit.
- UDP selection runs per datagram, so high packet-rate UDP receivers should be
  benchmark-gated before enabling the selector in production.
- The eBPF path needs a supported kernel and attach-time capabilities.
- Single-NUMA hosts get per-core distribution only, not locality.

## Alternatives Considered

- Keep the default `SO_REUSEPORT` hash: correct, but uneven for small numbers
  of long-lived connections and not NUMA-aware.
- One engine process per NUMA node behind an external load balancer: a valid
  operational pattern, but it pushes routing complexity onto operators and
  abandons the single-process model.
- Coordinated `SO_REUSEPORT` without eBPF: this is the proposed foundation and
  fallback. It controls listener creation but still leaves placement to the
  kernel hash, so it is not NUMA-aware on its own.
- Classic BPF (`SO_ATTACH_REUSEPORT_CBPF`): a tiny RX-CPU-index selector gives
  per-CPU fairness on a single-NUMA host with no BTF, no `vmlinux.h`, no
  `CAP_BPF`, and no 5.12 floor. It cannot express NUMA-local range selection
  with round-robin and a global fallback.
- `SO_INCOMING_CPU`: not a safe alternative on most production kernels; broken
  between Linux 4.1 and 6.1 and only fixed from 6.2.
- XDP / AF_XDP: a much heavier packet-processing architecture that runs before
  the normal socket listener path. It could be explored as a separate future
  ingestion architecture, but it does not fit this design's goal of preserving
  the standard `TcpListener` / `UdpSocket` receiver model.
- Aya: Aya is a viable future alternative because it exposes `SkReuseport` and
  `ReusePortSockArray`, and it would provide a more Rust-native eBPF stack. This
  proposal keeps libbpf-rs for the initial design because the selector is a
  small, stable UAPI-bound C program, the prototype already validates the
  libbpf-rs path, and the build/runtime cost is contained behind an optional
  Linux-only feature. The loader should stay isolated so the backend can be
  revisited if the BPF surface grows or the project prefers a pure-Rust eBPF
  toolchain later.

The eBPF selector is justified when the host is multi-NUMA, the workload
benefits from NUMA-local range selection plus per-NUMA round-robin plus a global
fallback, and a future migration phase is desired. Realistic gains from
locality-aware reuseport are in the single-digit-percent range on representative
ingestion workloads; this design does not change that ceiling.

## Future Work -- listener migration

Linux 5.14+ adds `net.ipv4.tcp_migrate_req` and a select-or-migrate reuseport
program type, which together let a selector pick a live replacement listener
when a peer in the group closes. That would enable hot-standby / zero-downtime
worker upgrades without dropping in-flight handshakes -- a property no simpler
alternative offers. It is out of scope here: migration has its own design
considerations, and should be a separate phase after the selector is exercised
on real multi-NUMA hardware.
