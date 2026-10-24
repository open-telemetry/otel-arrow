---
Proposal Name: hierarchical-cpu-limits
Start Date: 2026-10-22
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#4515
---

# RFC 0007: Hierarchical CPU Limits

## Summary

CPU limits are a required form of system isolation.

This RFC proposes aggregate, hierarchical CPU limits in the OTel-Arrow
Dataflow Engine (DFE), its pipeline groups, and its pipelines, using
the best available operating-system interfaces (e.g., on Linux hosts,
we will use the container groups v2 interface).

Using these options requires special operating-system privileges for
delegation of the necessary capabilities. This may require running the
DFE in a special way, for example, via special `systemd` unit
settings. When operating-system-level features or user-level
privileges are not available, `individual` limits may still be
enforced using cooperative mechanisms.

## Background

The dataflow engine currently supports configuring a core allocation,
detailing which of the available CPUs the engine will run
on. While the engine supports CPU placement as well as rate and
memory limits, it does not currently implement CPU limits.

The engine uses three levels of policies to configure itself, and each
level is a natural place to impose an aggregate CPU limit. An
engine-wide limit applies to all pipeline groups; group-wide limits
apply to all pipelines in the group; and pipeline-wide limits apply to
all nodes in the pipeline.

## Policy configuration

There are several common ways to communicate about CPU limits. With so
many ways to express these limits, we propose an explicit syntax
that supports variety. For example, a limit of 1.5 CPUs
per wall-clock second can be expressed in several ways:

- Milli-CPUs per second: 1/1000 CPUs per second, common in Kubernetes,
  e.g., `1500` is 1500 ms/second.
- CPU percentage: 1/100 CPUs per second, common in `systemd`, e.g., `150%`.
- Units per period: Microseconds per period, as used by the cgroups interface,
  e.g., `"150000 100000"` represents 150 ms per 100 ms scheduler period.
- System percent: Percent of total CPU, e.g., `18.75%` on an 8-CPU system.
- Parent percent: Percent of the enclosing environment, e.g., `37.5%` in a
  container with a `4cpu` limit.

CPU limits are nested. Containers and job objects express limits over
the sum of their children, which may be nested containers or job
objects.

A new `policies.resources.cpu.limit` policy section will be added alongside
`policies.resources.cpu.core_allocation`. The new section will offer:

- `total`: the limit applies to the sum across cores of the engine
- `individual`: the limit applies separately to individual cores of the engine

For pipelines, `individual` limits apply directly. At the group and
engine levels, an `individual` limit applies to every enclosed
pipeline. When configuring limits for a given pipeline, the minimum
value of the engine, group, and pipeline `individual` settings applies.

### Example configuration

Here is an example combining CPU limits at multiple levels.

```yaml
# Engine-wide configuration
policies:
  resources:
    cpu:
      limit:
        # Total self-imposed engine-wide mCPU limit: 4 CPUs/sec
        total: 4000

        # Per-pipeline limit across all groups: 0.5 CPUs/sec
        individual: 500

groups:
  # External receivers are in the ingest group.
  ingest:
    policies:
      resources:
        cpu:
          limit:
            # Total ingest-group limit: 2 CPUs/sec
            total: 2000

            # Per-pipeline limit for the ingest group: 25% of one CPU
            individual: 250

    # Pipelines in the ingest group.
    pipelines:
      # Pipeline for push-based receivers in the ingest group
      push:
        policies:
          resources:
            # Runs on 2 cores, subject to the engine-wide and ingest-group
            # limits and the individual limit of 25% of one core set above.
            core_allocation:
              type: core_count
              count: 2
        nodes:
          otlp:
            type: "receiver:otlp"
            policies:
              # CPU limits cannot be set in node policies.
            config:
              ...

      # Pipeline for pull-based receivers in the ingest group
      pull:
        # ...
```

## Implementation details

The conceptual hierarchy of CPU limits is represented below.

```text
engine                         total engine limit (all cores)
  group                        total group limit (all cores)
    pipeline                   total pipeline limit (all cores)
      worker on core 0         individual limit
      worker on core 1         individual limit
```

Unlike other areas of engine policy, these levels are always additive,
adding limits in addition to those placed above in the hierarchy.

The `individual` CPU limit corresponds to one operating-system
thread per engine core. Every CPU core allocation of the dataflow
engine instantiates a number of pipelines, and thus a number of threads
per CPU core. When any of the engine-wide, group-level, or pipeline-level
`individual` settings is present, a CPU limit is implied for each
per-core pipeline thread.

### Isolation guarantees

Every level of the `cpu::limit` policy is independent of the others, and
pipelines are always subject to every ancestor's limits. Limits are a
ceiling on CPU activity, not a reservation. Operating-system-level
configuration determines how pipelines are scheduled when the CPU is
overcommitted.

When CPU limits are configured and the required capabilities are not
available to the DFE process when it starts, it will abort the
process. We will not bypass CPU limits when the operating-system
feature is determined to be unavailable.

The mechanism presented here does not protect against untrusted
code.

### Milli-CPU units preferred

General limit configuration uses integer-valued configuration fields
expressed in milli-CPUs per wall-clock second, or `mcpu`. The finest CPU
limit configuration resolution is 1/1000 s. Instead of percent-based
configuration, we recommend using `individual` limit configuration,
multiplying the percent value by ten (e.g., `1%` equals `10 mcpu`).

There is no accommodation for configuring system-percent limits in the DFE.

### Non-pipeline CPU limits

A dedicated `system` resource with separate `cpu.limit` configuration
will be established for non-engine threads.

```yaml
policies:
  resources:
    cpu:
      limit:
        # Total self-imposed engine-wide mCPU limit: 4 CPUs/sec
        total: 4000

      system_limit:
        # Total limit on administrative and support activities
        total: 200
```

Likewise, for the observability pipeline:

```yaml
engine:
  observability:
    pipeline:
      policies:
        resources:
          cpu:
            # Internal telemetry system threads get 10% of one CPU
            individual: 100
```

### Linux implementation details

#### Cgroups v2

As needed, the engine will create a dedicated "threaded-domain root"
container. This is also the level at which engine-wide memory limits
are set, and this container is the process-wide parent underneath
which we find group-total containers, then pipeline-total containers,
then individual containers, one per pipeline per core.

Optional group-level and pipeline-level `total` limit configurations
require creating new cgroups and nesting them appropriately. When an
`individual` limit is applied, every pipeline on every core receives a
dedicated container group.

Platform-specific settings will be placed under `engine.cpu_control` in
OS-specific sections. These support optional features, such as naming
the engine's parent group or configuring its enforcement interval.

```yaml
engine:
  cpu_control:
    # Engine-level Linux-specific configuration for CPU limits
    linux:
      delegated_root: /sys/fs/cgroup/otel.service
      period: 100ms
```

[Further implementation details are out of scope][cgroupthreads].

[cgroupthreads]: https://docs.kernel.org/admin-guide/cgroup-v2.html#threads

#### Systemd recommendations

For safety reasons, we recommend that users configure external total
CPU limits as a backstop for a faulty or compromised engine.

An illustrative systemd service configuration is:

```ini
[Service]
User=otel
CPUQuota=400%
Delegate=cpu
NoNewPrivileges=yes
CapabilityBoundingSet=
```

#### T.B.D. More Linux details

### Windows implementation details

The Windows Job Object interface supports process-wide CPU limits.

The Windows kernel does not support nested container limits within
a process; for that, we will use cooperative limits.

### Limit observability

Every engine-controlled thread will have an associated metric
describing its CPU usage using the OpenTelemetry semantic convention
for CPU time, `engine.cpu.time`, in CPU seconds counter, with
attributes indicating the pipeline, pipeline group, and CPU core
number responsible for the usage. From these metrics we can calculate
total and group-subtotal CPU usage.

### Cooperative CPU budgets

Because of our thread-per-core design, the engine can limit itself by
effectively stalling the per-core engine thread's async runtime.

The internal metric system delivers per-thread CPU usage as described
above. Using this information and a cooperative controller, the engine
can implement `individual` and `total` limiters.

Cooperative CPU limiters will be defined through the extension
interface. Individual limits will use local capabilities (`!Send`),
while pipeline and group `total` limits will use shared (`Send`)
capabilities.

### Limit reconfiguration

Limited live reconfiguration of CPU limits will be supported, for
example to modify the value of an existing `individual` or `total`
limit.

### Work plan

T.B.D.
