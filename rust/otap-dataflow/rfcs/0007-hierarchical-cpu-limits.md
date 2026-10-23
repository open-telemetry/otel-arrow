---
Proposal Name: hierarchical-cpu-limits
Start Date: 2026-10-22
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#0000
---

# RFC 0007: Hierarchical CPU Limits

## Summary

Introduce aggregate CPU budgets at engine, pipeline-group, and logical-pipeline
scope, using a common fractional-CPU configuration and platform-specific
enforcement. Linux maps the budget hierarchy to cgroup v2 CPU controls. Windows
maps it to Job Object CPU rate controls where execution boundaries permit.
These are nested limits, not defaults copied into every child. CPU placement
remains separate from CPU bandwidth. Linux can enforce all three levels within
the current thread-per-core process; Windows Job Objects cannot independently
limit threads within one process, so Windows initially supports an engine-wide
limit. Full Windows hierarchy support requires a separately designed
process-isolated pipeline execution mode.

This document proposes behavior; it does not describe an implemented feature.

## Motivation

Core allocation answers where workers run and how many workers a pipeline has.
It does not express a fractional execution budget or constrain aggregate CPU
consumption across a group of pipelines.

Operators need to cap an engine's total CPU consumption, partition that budget
among pipeline groups, and constrain individual pipelines without multiplying
the limit by the number of worker cores. For example, a tenant group might have
a two-CPU budget shared by several pipelines, with one expensive pipeline capped
at half a CPU.

The operating system should enforce these limits even when a worker is executing
a long synchronous computation. Enforcement must not depend on every node
cooperatively checking a timer, nor introduce shared budget accounting into the
per-record data path.

The configuration should express the same intent on Linux and Windows without
pretending that the platforms have interchangeable isolation primitives.

## Guide-level explanation

### Budgets are aggregate and hierarchical

Add `policies.resources.cpu.limit` at each existing policy declaration scope.
A limit is measured in logical-CPU equivalents: `"1.5"` requests a ceiling
equivalent to one and a half fully utilized logical CPUs. It is not a percentage
of the host, a percentage of the parent, or a throughput guarantee.

The following configuration fragment shows the proposed policy shape. Node and
connection definitions are omitted:

```yaml
policies:
  resources:
    cpu:
      limit: "4"

groups:
  ingest:
    policies:
      resources:
        cpu:
          limit: "2"
    pipelines:
      traces:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 2
            cpu:
              limit: "0.5"
      logs: {}
  egress:
    policies:
      resources:
        cpu:
          limit: "3"
    pipelines:
      export: {}
```

The engine, including its control and observability work, shares four CPUs.
All `ingest` pipelines together share two CPUs. All workers of `traces`
together share half a CPU, despite its two-core placement. `logs` has no
additional local cap, but competes within the ingest budget. Egress has a
three-CPU ceiling, but ingest and egress cannot collectively escape the
engine ceiling.

The sum of sibling limits may exceed their parent's limit. These are ceilings,
not reservations: idle siblings do not reserve capacity, and a busy child is
still constrained by its own cap and every ancestor. No equal-share or minimum
service guarantee is implied.

### This is not ordinary policy inheritance

Existing resource policies generally resolve by nearest declaration.
CPU budgets deliberately differ:

- A declaration creates a budget at the scope where it appears.
- An omitted `cpu` creates no additional cap at that scope.
- An omitted child cap does not remove any ancestor cap.
- A child declaration cannot override, disable, or increase a parent's budget.
- Removing a declaration removes only that scope's local cap.

Thus a group limit of `"2"` is one shared two-CPU budget, not a separate
two-CPU allowance for every pipeline. Configuration documentation and resolved
admin output must make this distinction explicit.

### Placement and platform settings

`core_allocation` continues to choose worker count and affinity. Configuring a
CPU budget does not implicitly resize workers, change exclusivity rules, or
allocate fractional CPU IDs. A half-CPU budget can run on one pinned worker or
be shared by several pinned workers, with different latency and locality costs.

Platform setup belongs under a proposed engine-wide `engine.cpu_control`
section, separate from the portable budgets:

```yaml
engine:
  cpu_control:
    linux:
      delegated_root: /sys/fs/cgroup/otel.service
      period: 100ms
```

On Linux, `delegated_root` identifies an operator-provisioned subtree the engine
is allowed to manage, not an arbitrary host cgroup to take over. `period` has a
proposed default of `100ms` and applies consistently to engine-managed quotas.
Its supported range and quota granularity are validated against the backend.

The Windows backend uses the operating system's scheduling interval and does
not accept a Linux quota-period setting. It creates private Job Objects rather
than requiring a filesystem root. A `windows` settings block is reserved for
future, justified Windows-specific options; raw `CpuRate` is not a second way
to specify the common limit.

Platform-specific blocks are validated syntactically everywhere and applied
only on the matching platform. They do not suppress portable budget validation.
An unsupported requested scope is an error, not permission to ignore its cap.

### Initial capability boundary

| Execution environment | Engine cap | Group and pipeline caps |
| --- | --- | --- |
| Linux with delegated cgroup v2 CPU controller | Supported | Supported |
| Linux without usable delegation | Error if requested | Error if requested |
| Windows, single-process engine | Conditional | Unsupported |
| Windows, future process-isolated pipelines | Intended | Intended |
| Other platforms | Unsupported | Unsupported |

Windows engine support is conditional on CPU rate control availability,
compatible existing job membership, and a verified capacity basis. An embedded
engine must also have explicit host authorization to constrain the entire
process. Multiple independently limited engines sharing one host process are
not supported by this initial design.

With no CPU budgets configured, existing behavior remains unchanged and no
enforcement hierarchy is created.

## Reference-level explanation

### Existing implementation boundaries

The main integration points are:

- [Resource policy declarations and resolution][policy]: add the portable
  declaration, but do not flatten its hierarchy through nearest-scope selection.
- [Linux topology discovery][topology]: already filters CPUs by affinity and
  `cpuset.cpus.effective`; CPU bandwidth is a separate resource.
- [Controller worker launch][launch]: one common launch boundary for startup,
  replacement, and recovery.
- [Pipeline runtime][runtime]: current-thread Tokio runtimes and local futures
  preserve the thread-per-core execution model.
- [Live rollout planning][planning]: currently recognizes resize-only changes
  while ignoring resource differences; CPU budget changes must be classified
  explicitly rather than lost in this comparison.
- [Memory limiter][memory]: existing read-only cgroup discovery is useful prior
  art, but is not a delegated cgroup manager.
- [Pipeline metrics][metrics]: existing worker CPU measurements remain useful
  alongside aggregate scope accounting.

### Portable representation and compilation

Use positive decimal strings with at most three fractional digits in the first
version, parsed into checked integer milli-CPU units. For example, `"0.5"`
becomes 500 milli-CPUs. Reject zero, negative values, non-finite values, excess
precision, and overflow. Do not store or compare configuration as binary
floating point. Omission is the only initial spelling for no local cap.

The common resolution phase produces a budget tree keyed by engine identity,
group identity, and logical pipeline identity. Each node retains its local
declaration and parent identity, including uncapped intermediate scopes.
Runtime workers bind to their logical pipeline's budget handle; deployment
generation and core ID do not create independent budgets.

Compile the tree before starting work. Validate platform capability and
backend representability, and report both requested and programmed limits.
Backend quantization must never silently increase a requested ceiling. Round
down where necessary and expose the result; reject a positive value that would
round to zero or violate an OS minimum.

A useful configured upper bound is the minimum of the known limits along the
ancestor chain. This is not available capacity or an entitlement: siblings,
affinity, external limits, and other workloads may further reduce execution.
Preserve the tree rather than replacing it with that scalar minimum.

### Backend comparison

| Property | Linux cgroup v2 | Windows Job Objects |
| --- | --- | --- |
| Membership unit | Threads in a threaded subtree | Whole processes |
| Hard-cap control | `cpu.max` quota and period | `ENABLE` plus `HARD_CAP` |
| Hierarchy | Nested cgroups | Nested jobs |
| Local cap units | CPU time per period | Rate in units of 1/10,000 |
| Parent interaction | Ancestor quotas apply | Parent-relative rates |
| Relative weighting | `cpu.weight` | Weight-based CPU rate mode |
| Interval | Configurable period | OS-controlled |
| Provisioning | Writable delegated subtree | Job rights and valid nesting |

Weights, minimum reservations, and explicit burst allowances are not part of
the initial common contract. In particular, Linux's independent weight and quota
controls must not be presented as universally equivalent to Windows rate modes.
The portable contract is a CPU-capacity ceiling, not identical scheduling,
instantaneous utilization, or work completed on different processors.

### Linux hierarchy and attachment

Use cgroup v2 only for initial enforcement. CPU is a threaded controller, so
multiple pipelines in one process can belong to different leaves:

```text
operator-owned delegated root
  engine instance                 domain threaded; engine cpu.max
    system                        threaded; controller and shared services
    group ingest                  threaded; group cpu.max
      pipeline traces             threaded; pipeline cpu.max
      pipeline logs               threaded; no additional cap
    group egress                  threaded; group cpu.max
      pipeline export             threaded; no additional cap
```

The engine owns descendants it creates, not its supervisor's quota. The parent
container or service retains authority over the entire subtree. Discovery must
resolve cgroup membership against mount information and namespace boundaries;
blindly concatenating `/proc/self/cgroup` with `/sys/fs/cgroup` is insufficient
for general container layouts. Inspect accessible ancestors for quota context,
and report unavailable external information as unknown.

Provisioning must respect controller availability, `cgroup.subtree_control`,
threaded-domain constraints, and delegation permissions. Domain-only controllers
such as memory must not be enabled inside the threaded subtree. Preserve
external memory controls and resolve the existing memory sampler against the
appropriate domain and visible ancestor limits after migration.

For systemd, request CPU delegation and place the threaded subtree below an
engine-owned domain under the delegated unit, leaving room for systemd's
non-threaded `.control` sibling. Do not convert the delegated service root into
a threaded subtree. Container root privileges alone are not sufficient;
read-only mounts and missing CPU delegation are actionable startup errors.

Establish process membership in the engine domain before starting pipeline
workers, then put control/shared-service threads in `system`. Whole-process
migration is a bootstrap operation, not a per-pipeline operation. Each worker
attaches its own current Linux TID to its leaf through `cgroup.threads` before
component construction, runtime creation, helper-thread creation, or readiness.
Using `cgroup.procs` to attach a worker would incorrectly migrate the process.

For a requested limit `L` in milli-CPUs and period `P` in microseconds:

```text
quota_usec = floor(L * P / 1000)
cpu.max = "<quota_usec> <P>"
```

Use checked arithmetic and kernel-supported bounds. For `"1.5"` and `100ms`,
write `150000 100000`. Read back the installed configuration. This is an
aggregate scheduling budget, not a promise of a continuously running 1.5 CPUs.
Normal fair-class scheduling is the initial supported mode; realtime or other
scheduler classes must not be assumed to obey `cpu.max`.

### Windows hierarchy and process isolation

Use `SetInformationJobObject` with
`JobObjectCpuRateControlInformation` and
`JOB_OBJECT_CPU_RATE_CONTROL_ENABLE | JOB_OBJECT_CPU_RATE_CONTROL_HARD_CAP`.
Associate processes using `AssignProcessToJobObject`. There is no corresponding
Job Object operation that assigns only one pipeline thread.

Assigning the current engine process to several jobs does not partition its
threads. At best it establishes a nested chain constraining the same process.
It cannot represent sibling pipeline or group budgets. Initial Windows support
must reject those declarations, even if all pipelines happen to use disjoint
CPU affinity masks.

Full hierarchy support would require processes that do not mix independently
budgeted pipelines. A prospective arrangement is one process per logical
pipeline, containing its pinned worker threads, with engine, group, and pipeline
jobs forming an ancestor chain for that process. The controller belongs only to
the engine chain. A process-per-group model cannot enforce pipeline caps within
that group.

This is substantial work, not an implementation detail of a CPU backend:
in-memory topics, listener sharing, control messages, telemetry registries,
configuration delivery, and shutdown need explicit cross-process transports or
ownership changes. No such process mode is assumed to exist today. Keep
thread-local futures within worker processes; do not solve IPC by making all
node futures `Send` or replacing local runtimes with a shared scheduler.

For an unparented rate-controlled job, let `C` be the verified system capacity
in logical-CPU equivalents corresponding to the Windows rate denominator:

```text
CpuRate = floor(10000 * requested_cpu_limit / C)
```

For a job beneath a rate-controlled ancestor, convert against the applicable
parent rate budget instead, not host capacity. For example, on an eight-CPU
system, an engine budget of four uses rate 5000; its two-CPU group uses 5000
relative to that engine; the group's half-CPU pipeline uses 2500. Account for
rounding at each level using the programmed ancestor budget. A child whose cap
exceeds its ancestor ceiling needs no more than a 100% relative rate.

These formulas describe the mapping, not a claim that affinity-mask size or
Rust's `available_parallelism()` gives the correct denominator. Processor
groups, CPU sets, external jobs, and Windows containers require explicit
verification. Windows describes rate control in processor cycles, whereas
Linux quotas use execution time; hardware and scheduling differences preclude
bit-for-bit equivalent accounting. Report the conversion basis and reject
configurations where it cannot be established reliably.

Build nested jobs and assign processes in valid ancestor-to-descendant order.
For future child workers, use suspended creation and complete assignment before
allowing execution. Do not enable breakaway to escape ancestor limits. Existing
job membership and restrictions must be probed; failed nesting is not a reason
to continue unbounded. Job association cannot simply be undone, so preflight,
embedding authorization, and process lifetime ownership are essential.

### Ownership, attribution, and shared services

The controller owns OS budget objects and passes lightweight binding handles
to launch paths. Backend management runs on the control path, not per record.
No cross-worker token bucket, hot-path lock, or periodic thread suspension is
needed for kernel enforcement.

Worker-created helper threads should stay within the pipeline's enforcement
boundary. Audit Tokio blocking workers, native clients, library pools, and
subprocesses rather than assuming everything runs in the main worker thread.
An existing shared pool cannot acquire accurate pipeline attribution merely by
receiving pipeline work.

The initial boundary is execution ownership: pipeline-owned workers and helpers
are charged to the pipeline, while deliberately shared services are charged to
the engine's system branch. Shared console output and observability work are
examples of engine overhead, not precisely attributed per-request CPU. This
limitation must be documented; a pipeline cap is not complete causal accounting
of all work its traffic triggers. Components must not offload private expensive
work to shared services merely to evade the cap.

System work remains subject to the engine and external ancestor ceilings. An
uncapped system branch protects it from a pipeline-local cap, not from aggregate
engine exhaustion. This proposal does not promise a reserved control-plane CPU.

### Lifecycle and live configuration

Budget identity is stable across worker restart, resize, and rolling
replacement. Old and new generations share the same logical-pipeline budget
while both exist. A group budget spans all its pipelines and generations.
Do not create a fresh full allowance for each replacement.

Prepare and validate the affected budget tree before accepting a rollout.
Attachment and limit-installation failures must prevent readiness and enter the
existing controller error reporting path. Recovery repeats attachment; it does
not launch an unrestricted replacement.

The initial implementation may require restart for any CPU-policy change, but
must reject unsupported live changes explicitly. Later live updates need a
serialized transition plan, readback, and failure recovery. OS tree updates are
not atomic. Windows parent-rate changes additionally require descendant rate
recalculation to preserve absolute requested limits. Transition ordering must
avoid transiently relaxing hard caps; conservative temporary throttling is
preferable to silently exceeding either accepted configuration. The control
plane must not report success until the new policy is installed.

Removing a local cap leaves ancestors active. Moving a pipeline to another
group is a rollout with changed budget ancestry, not a metadata rename.
Windows process membership may require process replacement rather than moving
an existing process between sibling jobs.

Delete OS objects only after all workers, helpers, and descendants have exited.
Use engine-instance ownership markers and encoded internal scope identifiers,
not raw user names as filesystem paths. Bound object count by configured scopes
and tracked draining state. Report cleanup failures and stale owned objects;
never recursively remove an operator-owned subtree. Windows job-handle lifetime
and kill-on-close behavior require explicit controller ownership, not incidental
handle drops.

### Throttling, backpressure, and observability

Kernel throttling pauses all execution in a capped worker, including timers,
control messages, and shutdown processing. A wide worker set can burn its quota
early in a period and then stall together. Receiver buffering, bounded queues,
existing admission controls, and timeout behavior remain important; CPU limits
are not a substitute for backpressure or memory limits.

Sample budget accounting from the controller so a throttled pipeline does not
have to run to report its own throttle state. Expose requested and programmed
limits, budget ancestry, backend capability, and enforcement failures through
admin state. Distinguish a known configured ceiling from unknown external
constraints and from measured usage.

Provide aggregate CPU usage per budget scope and supported backend throttle
signals. Linux can read `cpu.stat`; ancestor throttling must also be considered
when interpreting local counters. Windows job accounting and available
notifications are not identical to Linux throttle counters. Missing capability
must not be emitted as a fabricated zero. Notifications alone are not proof that
enforcement or accounting is healthy.

Follow the [metrics guide][metrics-guide] and
[item attributes guide][item-attributes]. Register metrics on stable scope
entities, not each worker replica of the same shared budget. Use observed
cumulative counters for OS cumulative accounting, with reset/lifetime handling.
Keep OS paths, TIDs, PIDs, job handles, and errors out of item attributes.
Parent accounting includes descendants; summing parent and child usage double
counts work.

### Validation and delivery

Implement in independently reviewable stages:

1. Add common declarations, exact numeric parsing, budget-tree compilation,
   capability reporting, and explicit rejection of unsupported live changes.
2. Add Linux delegated hierarchy management, attachment, and accounting.
3. Add Windows engine-wide Job Object enforcement with verified normalization.
4. Design process-isolated execution separately before promising Windows group
   or pipeline enforcement.

Unit tests cover hierarchy preservation, omitted local caps, oversubscribed
siblings, numeric boundaries, quantization, and Windows parent-relative rates.
Controller tests cover attach failure, helper ownership, recovery, overlapping
generations sharing one cap, deletion, and unsupported reloads. Document each
test's scenario and protected guarantee.

Dedicated Linux integration tests use an explicitly delegated temporary
subtree; Windows tests use isolated child processes and owned jobs. Establish
worker readiness with handshakes, read back membership and configured limits,
and verify aggregate enforcement with CPU-bound workers across several cores
and siblings. Measure cumulative CPU consumption over multiple enforcement
intervals with declared tolerances for granularity and boundary effects.
Do not assert scheduling latency on a loaded general-purpose test runner.

An enforcement test must demonstrate that aggregate sibling work is bounded by
the parent and that worker count does not multiply a pipeline cap. Include
external ancestor limits, unsupported deployment modes, and cleanup after
failure. Capability-gated developer tests may explicitly skip unavailable
backends, but dedicated platform CI must require them. No suite may silently
pass by skipping all enforcement cases.

## Drawbacks

The common policy cannot deliver all scopes on Windows without a new process
execution architecture. Linux needs deployment privileges that ordinary
containers often do not expose. Both backends add lifecycle complexity, and
strict caps can increase tail latency or delay health and shutdown processing.

Aggregate CPU budgets also introduce a deliberate exception to existing policy
inheritance. Operators need clear resolved-state presentation to avoid treating
ceilings as reservations or assuming shared services are pipeline-attributed.

## Rationale and alternatives

OS enforcement handles non-cooperative computations without changing the hot
path. A hierarchy directly represents the requested engine/group/pipeline
aggregation and naturally preserves parent authority.

Affinity alone limits placement, not fractional consumption. Admission-rate
limits bound traffic rather than CPU cost. Cooperative userspace CPU accounting
could approximate a cross-platform budget, but cannot reliably preempt a long
poll or native call; it must not be labeled equivalent hard enforcement.
Suspending arbitrary worker threads risks freezing shared locks and is rejected.

Separate engines in externally managed containers or processes already offer
coarse isolation, and are a valid Windows deployment alternative. They change
topology, operational overhead, and communication costs rather than adding
in-process per-pipeline enforcement. Adopting process isolation on both platforms
would improve symmetry but unnecessarily discard Linux's existing thread-level
fit.

Flattening every ancestor limit into a per-pipeline number is rejected: it
cannot enforce the aggregate group total when siblings run concurrently.

## Prior art

Linux documents [threaded cgroups and CPU controls][cgroup-v2], including
`cpu.max`, ancestor constraints, and the distinction between domain and threaded
controllers. [systemd delegation][delegation] describes subtree ownership and
the placement of threaded subtrees below delegated services.

Microsoft documents [Job Objects][jobs] as process groups,
[nested jobs][nested-jobs] as process-subset hierarchies, and
[CPU rate controls][job-cpu] as rates relative to a rate-controlled parent or,
without one, the system. These differences determine the support boundary,
rather than being hidden behind a nominally portable backend interface.

The existing engine memory limiter, CPU topology discovery, and per-worker
metrics supply local precedents for configuration and accounting. None currently
provides writable OS CPU-budget hierarchy management.

## Unresolved questions

- Is partial Windows support acceptable initially, or must process-isolated
  execution be a prerequisite for shipping the common policy?
- Should a future explicit system-branch budget or reservation protect control
  and observability work, and how would its semantics map across platforms?
- Which Windows processor-group, container, and external-job combinations can
  establish a trustworthy normalization basis and be supported in platform CI?
- Is milli-CPU precision sufficient, and what quantization difference should be
  considered too large to accept even when explicitly reported?
- Should the first release reject all CPU-policy reloads, or include a limited
  verified subset of in-place changes?

## Future possibilities

Future work may add quota-aware worker sizing, relative sibling weights,
explicit burst control, process-isolated pipelines, and richer external-limit
discovery. Each needs its own semantics; none should be inferred from the
presence of a CPU ceiling.

Memory and I/O budgets are not automatically portable extensions of this tree.
In particular, Linux threaded CPU cgroups do not provide independent memory
domains for threads sharing one process.

[policy]: ../crates/config/src/policy.rs
[topology]: ../crates/engine/src/topology/linux.rs
[launch]: ../crates/controller/src/live_control/runtime.rs
[runtime]: ../crates/engine/src/runtime_pipeline.rs
[planning]: ../crates/controller/src/live_control/planning.rs
[memory]: ../crates/engine/src/memory_limiter.rs
[metrics]: ../crates/engine/src/pipeline_metrics.rs
[metrics-guide]: ../docs/telemetry/metrics-guide.md
[item-attributes]: ../docs/telemetry/item-attributes.md
[cgroup-v2]: https://docs.kernel.org/admin-guide/cgroup-v2.html
[delegation]: https://systemd.io/CGROUP_DELEGATION/
[jobs]: https://learn.microsoft.com/en-us/windows/win32/procthread/job-objects
[nested-jobs]: https://learn.microsoft.com/en-us/windows/win32/procthread/nested-jobs
[job-cpu]: https://learn.microsoft.com/en-us/windows/win32/api/winnt/ns-winnt-jobobject_cpu_rate_control_information
