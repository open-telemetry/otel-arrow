# Host Metrics Receiver Design

<!-- markdownlint-disable MD013 -->

Status: draft

Issue: <https://github.com/open-telemetry/otel-arrow/issues/2741>

Receiver URN: `urn:otel:receiver:host_metrics`

Target crate: `crates/core-nodes`

Target module: `crates/core-nodes/src/receivers/host_metrics`

The issue explicitly asks for `core-nodes`. If maintainers prefer to stage this
receiver in `contrib-nodes` while the implementation and system semantic
conventions are still settling, the module boundary below should remain the
same and only the crate root should change.

## Goals

Implement a Linux-first host metrics receiver for the OTAP dataflow engine.
The receiver emits host-level metrics as `OtapArrowRecords::Metrics` and avoids
the scraper-per-family shape used by the Collector hostmetrics receiver.

The v1 receiver must:

- run as a local receiver;
- collect only host-wide metrics, not per-process time series;
- support one receiver instance with per-family intervals;
- apply `root_path` consistently for container host filesystem collection;
- emit metrics aligned with current OpenTelemetry system metric semantic
  conventions by default;
- fail fast when configured in a pipeline that can run on more than one core;
- guard duplicate host collection in the same engine process;
- report receiver self-observability through `MetricSet`.

## Non-Goals

The v1 receiver does not include:

- per-PID process metrics;
- top-k process metrics;
- grouped process aggregates;
- eBPF;
- io_uring;
- NFS-specific handling;
- network connection counts;
- macOS support;
- Windows support;
- entity-event logs.

## Existing Code To Reuse

Receiver registration and lifecycle should follow the existing local receiver
pattern used by receivers under `crates/core-nodes/src/receivers`.

Metric emission should use the existing OTAP metric batch infrastructure:

- `OtapArrowRecords::Metrics`
- `otap_df_pdata::otap::Metrics`
- `MetricsRecordBatchBuilder`
- `NumberDataPointsRecordBatchBuilder`
- metric, resource, scope, and datapoint attribute record batches

Use a small host-specific OTAP metrics builder wrapper around the record batch
builders. This keeps the steady path direct and avoids constructing an
intermediate `MetricsView` object model for every scrape. The temporal
reaggregation processor's use of `MetricsRecordBatchBuilder` is the closest
existing write-side prior art.

Do not build steady-path OTLP protobuf metric objects just to encode them back
to OTAP Arrow records.

## Collector Reference

The Go Collector `hostmetricsreceiver` is reference material for source edge
cases, configuration ergonomics, and compatibility tests. It is not the target
architecture. The OTAP receiver deliberately diverges in these areas:

- Do not require multiple receiver instances to get different collection
  intervals. This receiver schedules per-family intervals inside one singleton
  receiver.
- Do not use independent scrapers that reread shared Linux sources in the same
  tick. This receiver builds one shared snapshot for the due family set.
- Do not preserve legacy metric names just because the Collector emits them.
  For example, the Collector load scraper emits `system.cpu.load_average.*`,
  but those names are not listed in Semantic Conventions 1.41.0 system metrics.
- Do not include per-process metrics in v1. The Collector's `process` scraper
  is useful future reference, but v1 only emits aggregate process metrics.
- Do not copy entity-event logs. They are explicitly out of scope for v1.

Collector behavior worth preserving at the boundary: root-path consistency,
host-perspective filesystem mount points, disk/filesystem/network filters, and
partial scrape behavior that emits successfully collected metrics.

## Module Layout

Use a narrow module layout and keep the boundaries explicit:

```text
crates/core-nodes/src/receivers/host_metrics/
  mod.rs
  config.rs
  metrics.rs
  scheduler.rs
  lease.rs
  host_view.rs
  error.rs
  linux/
    mod.rs
    snapshot.rs
    filesystem.rs
    parsers/
      cpu_stat.rs
      diskstats.rs
      loadavg.rs
      meminfo.rs
      net_dev.rs
      stat.rs
  project/
    mod.rs
    metrics_builder.rs
    semconv.rs
```

Keep collection, normalization, and projection separate:

- `linux/*`: read Linux sources and parse them into raw typed structs.
- `linux/snapshot.rs`: assemble the raw data required by the due families.
- `project/*`: convert snapshots and previous samples into semantic OTAP
  metrics.
- `scheduler.rs`: decide which families are due and drive scrape ticks.
- `host_view.rs`: own `root_path` resolution and validation.
- `lease.rs`: prevent duplicate collection for the same normalized host view.

## Runtime Shape

The receiver is a singleton source node. It should run in one pipeline instance
on one core. The receiver owns the family scheduler, previous samples,
duplicate lease, and Linux reads from `/proc`, `/sys`, and `statfs`.

The simplest valid topology is a one-core pipeline containing
`receiver:host_metrics` followed by the required processors and exporters. If
downstream processing needs more than one core, use topic fanout as the
parallelism boundary: a one-core source pipeline with
`receiver:host_metrics -> exporter:topic`, and a separate multicore downstream
pipeline with `receiver:topic -> processors/exporters`.

Within the receiver, one scrape tick should build one shared snapshot for the
families due at that time. Collection, previous-sample updates for derived
gauges, semantic projection, and OTAP record emission are separate steps.

This design does not require NUMA-aware collection in v1. Host metrics are
host-wide counters and gauges, and the important correctness rule is singleton
collection. NUMA placement may matter later for very high-rate local collection
or per-NUMA metrics, but that is not part of the v1 scope. For v1 procfs/sysfs
scraping, the useful optimization is reading each required source once per
scrape tick, not trying to localize file reads to the core running the receiver.
Future eBPF providers may use per-CPU maps and NUMA-aware aggregation behind
the same host observation boundary.

## Configuration

The config is native to this receiver. It should not try to mirror Collector
hostmetrics receiver config.

```yaml
nodes:
  host_metrics:
    type: receiver:host_metrics
    config:
      collection_interval: 60s
      initial_delay: 1s
      host_view:
        root_path: /
        validation: fail_selected
      families:
        cpu:
          interval: 30s
          per_cpu: false
        memory: {}
        paging: {}
        system:
          interval: 60s
        disk:
          interval: 60s
          exclude:
            devices: ["loop*"]
            match_type: glob
        filesystem:
          interval: 5m
          include_virtual_filesystems: false
          exclude_fs_types:
            fs_types: ["tmpfs", "proc", "sysfs"]
            match_type: strict
        network:
          interval: 60s
          include_connection_count: false
          exclude:
            interfaces: ["lo"]
            match_type: strict
        processes:
          interval: 60s
          mode: summary
```

Rules:

- `collection_interval` is the default family interval.
- Each family may override `interval`.
- `interval` must be greater than zero.
- `initial_delay` defaults to zero if omitted.
- Unknown fields must be rejected with `serde(deny_unknown_fields)`.
- `include_connection_count: true` is invalid in v1.
- `processes.mode` only accepts `summary` in v1.
- `processes.mode: summary` emits `system.process.count` and
  `system.process.created`; `system.process.count` is limited to `running` and
  `blocked` states from `/proc/stat`. `blocked` is a documented custom
  `process.state` value because the current registry has no well-known value
  for `procs_blocked`. It must not emit per-PID series or PID attributes.
- The load family is not shown in the default example because Semantic
  Conventions 1.41.0 does not register a load metric. If maintainers choose an
  experimental Linux load metric, add it as an explicit opt-in.
- `filesystem.include_virtual_filesystems: false` excludes at least: `autofs`,
  `bpf`, `binfmt_misc`, `cgroup`, `cgroup2`, `debugfs`, `devtmpfs`, `fusectl`,
  `mqueue`, `nsfs`, `overlay`, `proc`, `pstore`, `squashfs`, `sysfs`, `tmpfs`,
  and `tracefs`.
- Disk filters may include or exclude devices.
- Filesystem filters may include or exclude devices, filesystem types, and
  mount points. Mount point filters are evaluated from the host perspective.
- Network filters may include or exclude interfaces.
- Filter `match_type` values are `strict`, `glob`, or `regexp`. Prefer
  `strict` for low overhead; `regexp` is useful for compatibility with existing
  Collector-style configurations. Use Rust's `regex` crate or another
  linear-time engine for `regexp`.
- At least one family must be enabled.

Use `humantime_serde` or the repo's existing duration parsing convention if one
is already established for node configs.

This is still one receiver instance. Per-family intervals mean the receiver's
internal scheduler decides which enabled families are due on each tick. For the
example above, CPU is collected every 30 seconds, disk and network every 60
seconds, and filesystem every 5 minutes. Users should not deploy separate
`host_metrics` receivers just to collect different families at different rates.

## Host View And Root Path

The receiver should not concatenate root paths ad hoc. All host-relative paths
must go through a `HostView`.

```text
HostView
  root_path: normalized absolute path
  proc_path("stat") -> <root_path>/proc/stat
  sys_path("block") -> <root_path>/sys/block
  etc_path("machine-id") -> <root_path>/etc/machine-id
```

`HostView` is responsible for:

- rejecting relative `root_path`;
- normalizing repeated separators and trailing slashes;
- preventing path escape when resolving known relative paths;
- validating only the paths required by enabled families;
- producing a stable lease key.

Validation modes:

| Mode | Behavior |
| ---- | -------- |
| `fail_selected` | Fail startup if a source required by an enabled family is missing or unreadable. Default. |
| `warn_selected` | Start, disable unavailable selected sources, and report errors through metrics. |
| `none` | Skip startup validation. Runtime read errors still count. |

Do not validate sources for disabled families.

## Linux Sources

Read only the sources required by the due family set for a scrape tick.

| Family | Primary sources |
| ------ | --------------- |
| CPU | `/proc/stat`, `/sys/devices/system/cpu`, `/proc/cpuinfo` |
| Memory | `/proc/meminfo` |
| Load | `/proc/loadavg` if an experimental load metric is explicitly enabled |
| Paging/swap | `/proc/vmstat`, `/proc/meminfo`, `/proc/swaps` |
| System uptime | `/proc/uptime`; `/proc/stat` `btime` for counter start timestamps |
| Disk | `/proc/diskstats`, `/sys/block` |
| Filesystem | mountinfo, `statfs` |
| Network | `/proc/net/dev` or host-view `/proc/1/net/dev` |
| Processes | `/proc/stat` |

If several due families need the same source, read it once and share the parsed
result through the snapshot. For example, `/proc/stat` can feed CPU, process
created count, boot time, and aggregate process counters.

Use a small shared filter implementation for disk devices, filesystem devices,
filesystem types, mount points, and network interfaces. Compile filters during
configuration validation or receiver creation, not on every scrape tick.

CPU time values from `/proc/stat` are USER_HZ ticks. Cache
`sysconf(_SC_CLK_TCK)` at startup and divide tick counters by that value before
projecting `system.cpu.time` in seconds. For `cpu.mode`, map `user`, `nice`,
`system`, `idle`, `iowait`, and `steal` directly; fold both `irq` and
`softirq` into `interrupt`; do not emit `guest` or `guest_nice` because Linux
already includes them in `user` and `nice`. The parser must tolerate shorter
CPU rows on older kernels where later fields such as `steal`, `guest`, or
`guest_nice` are absent.

For CPU frequency, `/proc/cpuinfo` MHz multiplied by 1,000,000 is the reliable
path on VMs and containers. Per-CPU cpufreq data under
`/sys/devices/system/cpu/cpu*/cpufreq` may be used when present and readable,
because it can reflect current P-state frequency, but it is absent on many
hypervisors.

For system uptime, `/proc/uptime` is the file source. An implementation may use
the `CLOCK_BOOTTIME` syscall instead when it is available at low overhead.
`/proc/stat` `btime` is different: it is the Unix epoch timestamp of boot and
is used for cumulative sum start timestamps.

For paging, tolerate an empty `/proc/swaps` data set. Systems without swap may
expose only the header line.

Disk time fields from `/proc/diskstats` are milliseconds. Convert read/write
operation time and device I/O time to seconds before projection. Parse the base
kernel diskstats columns required for reads, writes, merged operations,
sectors, read/write milliseconds, and device I/O milliseconds; tolerate older
or newer kernels by ignoring optional discard and flush columns.

Linux-specific modules must be gated with `#[cfg(target_os = "linux")]`. Builds
on other operating systems should compile and return an unsupported-platform
configuration/startup error for this receiver.

For filesystem collection, prefer the host view's mountinfo. When running in a
container with `root_path` pointing at the host filesystem, that normally means
`<root_path>/proc/1/mountinfo`, not `/proc/self/mountinfo`, because
`/proc/self/mountinfo` reflects the receiver process mount namespace. If the
host proc filesystem is not visible, disable the filesystem family or fail
startup according to the validation mode.

For network collection, prefer the host view's network device file. When
running in a container with `root_path` pointing at the host filesystem, that
normally means `<root_path>/proc/1/net/dev`, not `/proc/net/dev`, because
`/proc/net/dev` reflects the receiver process network namespace. If the host
proc filesystem is not visible, disable the network family or fail startup
according to the validation mode. Reading `<root_path>/proc/1/net/dev` may
require root or `CAP_SYS_PTRACE` on hardened systems; containerized deployments
must grant enough privilege for the selected host view.

`statfs` can block on unhealthy remote, FUSE, or dying local mounts. Because
this receiver runs as a singleton local source, filesystem usage collection
must not call blocking `statfs` directly on the receiver task. Use a bounded
blocking worker path with per-mount timeout/cancellation behavior, and skip
remote filesystems plus known virtual filesystem types by default.

For process summary, use `/proc/stat` fields first: `processes`,
`procs_running`, and `procs_blocked`. Do not walk `/proc/<pid>/stat` in the v1
default path. A per-PID walk is reserved for future richer process modes and
must tolerate PIDs disappearing between directory read and file read.

## Scheduler

The receiver owns scheduling. Do not depend on one engine schedule per metric
family.

Maintain one schedule entry per enabled family:

```text
family
  interval
  next_due
```

Loop behavior:

1. Wait for `initial_delay`.
2. Compute the next due time across all families.
3. Wait for either that time or a control message.
4. On tick, collect all families whose `next_due <= now`.
5. Build one Linux snapshot for that due family set.
6. Project all due families into one metrics batch.
7. Send the batch downstream.
8. Advance each due family's `next_due` by whole intervals until it is in the
   future.

Use skipped-tick behavior instead of trying to catch up with multiple scrapes.
If the receiver is behind, emit one scrape for the current due set and count the
lag in receiver metrics.

Control messages must stay biased ahead of scrape work so drain and shutdown do
not wait behind slow host IO.

## Deployment Guard

Host-wide collection must run as a singleton source pipeline.

The default startup behavior must fail if the pipeline is allocated to more
than one core. The error message should say that host-wide collection must run
in a one-core source pipeline and should recommend topic fanout.

Recommended topology:

```text
one-core pipeline:
  receiver:host_metrics -> exporter:topic

multicore pipeline:
  receiver:topic -> processors/exporters
```

Reject `pipeline_ctx.num_cores() > 1` from the receiver factory `create`
closure with an invalid user configuration error. No engine validation hook is
required for v1 because `PipelineContext` already exposes the number of cores.

## Duplicate Lease

Add a process-local startup lease keyed by normalized host view:

```text
host_metrics:<normalized-root-path>
```

Behavior:

- acquire lease during receiver creation or startup;
- fail startup if another receiver already owns the same key;
- hold the lease in an RAII guard and release it from `Drop`;
- do not try to coordinate across processes.

The RAII guard is required so the lease is released on early return and during
panic unwinding, not only on the normal shutdown path.

The lease is a guardrail until an engine-scoped host observation extension
exists. It is not a replacement for the one-core deployment rule. It also
protects against accidental runtime duplication, for example if live
reconfiguration introduces a second host metrics receiver for the same host
root.

## Projection And Semantic Conventions

Projection must be table-driven enough that tests can verify metric shape.

Each emitted metric needs:

- name;
- unit;
- instrument kind as represented in OTAP metric records;
- temporality where applicable;
- monotonic flag where applicable;
- required and optional attributes;
- value source;
- start time behavior.

Use current OpenTelemetry system metric semantic conventions by default:

<https://opentelemetry.io/docs/specs/semconv/system/system-metrics/>

This design is aligned with Semantic Conventions 1.41.0 for the `system.*`
namespace. The receiver should emit host metrics under `system.*` only when
they are collected from inside the target system, and should use `host.*`
resource attributes for host identity. Metrics collected from technology
specific APIs, such as kubelet or container runtimes, belong under their own
namespaces and are outside this receiver.

The system metric semantic conventions are still marked development. The
receiver should validate metric names, units, instruments, attributes, and
resource attributes against the OpenTelemetry semantic-conventions registry
during implementation. Keep those constants centralized in `project/semconv.rs`.

Because the system metric semantic conventions are development status, future
registry changes may still require migration. This receiver is new, so it should
default to the current registry rather than preserving older Collector or
pre-registry names. Keep names and attributes centralized so future migrations
are contained.

Resource attributes should include stable host identity when available:

- `host.id` from `/etc/machine-id`, then `/var/lib/dbus/machine-id`; omit it
  if neither exists, and do not synthesize an ID;
- `host.name` from `/proc/sys/kernel/hostname`;
- `host.arch` from `uname` or an equivalent host-view source when available;
- `os.type = linux`.

`os.description` and `os.version` may be added later from kernel release data,
but they are not required for v1.

If `root_path` points at a mounted host filesystem from inside a container,
identity files must be read through `HostView`.

## V1 Metric Families

This table is the implementation checklist. It is intentionally limited to v1.

| Family | Metrics |
| ------ | ------- |
| CPU | `system.cpu.time`, `system.cpu.physical.count`, `system.cpu.logical.count`, `system.cpu.frequency`; opt-in: `system.cpu.utilization` |
| Memory | `system.memory.usage`, `system.memory.utilization`, `system.memory.linux.available`, `system.memory.linux.slab.usage`; opt-in: `system.memory.limit`, `system.memory.linux.shared`, Linux hugepage metrics |
| Load | Linux-specific load metrics, pending semconv registry decision. Do not emit `system.cpu.load_average.*`; those names are not in the current semconv registry. |
| Paging/swap | `system.paging.usage`, `system.paging.utilization`, `system.paging.operations`, `system.paging.faults` |
| System | `system.uptime` |
| Disk | `system.disk.io`, `system.disk.operations`, `system.disk.io_time`, `system.disk.operation_time`, `system.disk.merged`; opt-in: `system.disk.limit` |
| Filesystem | `system.filesystem.usage`, `system.filesystem.utilization`; opt-in: `system.filesystem.limit` |
| Network | `system.network.io`, `system.network.packet.count`, `system.network.packet.dropped`, `system.network.errors` |
| Processes | `system.process.count`, `system.process.created` |

The exact final list must be confirmed against the OpenTelemetry
semantic-conventions registry during implementation. If a source value cannot
be mapped cleanly, do not invent a generic `system.*` metric name in the hot
path. Add an explicit projection decision and test.

## Projection Shape

The projection layer should make instrument kind, unit, and the non-obvious
attribute choices explicit. Counter instruments below are OTAP cumulative sums
and monotonic unless noted otherwise. Use `/proc/stat` `btime` as
`start_time_unix_nano` only for cumulative sums. Gauges do not carry this start
timestamp.

| Metric | Default | Instrument | Unit | Key attributes and notes |
| ------ | ------- | ---------- | ---- | ------------------------ |
| `system.cpu.time` | Yes | Counter | `s` | `cpu.mode`; aggregate by summing all logical CPUs for each mode because `cpu.logical_number` is opt-in. Convert USER_HZ ticks to seconds. Detect per-series counter reset and re-baseline start timestamp. |
| `system.cpu.physical.count` | Yes | UpDownCounter | `{cpu}` | Host-level count. |
| `system.cpu.logical.count` | Yes | UpDownCounter | `{cpu}` | Host-level count. |
| `system.cpu.frequency` | Yes | Gauge | `Hz` | `cpu.logical_number`; use `/proc/cpuinfo` MHz times 1,000,000 as the reliable VM/container path, with readable sysfs cpufreq as an optional current-frequency source. |
| `system.cpu.utilization` | No | Gauge | `1` | `cpu.mode`; aggregate as the host-level ratio computed from summed CPU time deltas per mode. Skip first scrape silently because it requires a previous sample. |
| `system.memory.usage` | Yes | UpDownCounter | `By` | `system.memory.state` values `used`, `free`, `cached`, `buffers`; Linux `used` is `MemTotal - MemAvailable`, with legacy fallback only if `MemAvailable` is absent. |
| `system.memory.utilization` | Yes | Gauge | `1` | `system.memory.state`; same state calculation as usage. Do not assume state values sum exactly to `MemTotal` when `used` uses `MemAvailable`. |
| `system.memory.linux.available` | Yes | UpDownCounter | `By` | Linux `MemAvailable`. |
| `system.memory.linux.slab.usage` | Yes | UpDownCounter | `By` | `system.memory.linux.slab.state` values `reclaimable`, `unreclaimable`. |
| `system.memory.limit` | No | UpDownCounter | `By` | Total virtual memory. |
| `system.memory.linux.shared` | No | UpDownCounter | `By` | Linux `Shmem`. |
| Linux hugepage metrics | No | Mixed | Mixed | Use current `system.memory.linux.hugepages.*` registry definitions. |
| `system.paging.usage` | Yes | UpDownCounter | `By` | `system.paging.state`, `system.device`; use `/proc/swaps` for swap device identity. |
| `system.paging.utilization` | Yes | Gauge | `1` | `system.paging.state`, `system.device`; use `/proc/swaps` for swap device identity. |
| `system.paging.operations` | Yes | Counter | `{operation}` | `system.paging.direction` from `pswpin` and `pswpout`; intentionally omit `system.paging.fault.type` because Linux swap-in/out counters are not broken down by fault type. |
| `system.paging.faults` | Yes | Counter | `{fault}` | `system.paging.fault.type`; use `pgmajfault` for `major` and `pgfault - pgmajfault` for `minor` when both are available. |
| `system.uptime` | Yes | Gauge | `s` | Prefer `CLOCK_BOOTTIME`; fall back to `/proc/uptime`. Emit double seconds. |
| `system.disk.io` | Yes | Counter | `By` | `system.device`, `disk.io.direction`. |
| `system.disk.operations` | Yes | Counter | `{operation}` | `system.device`, `disk.io.direction`. |
| `system.disk.io_time` | Yes | Counter | `s` | `system.device`; convert diskstats device I/O milliseconds to seconds. |
| `system.disk.operation_time` | Yes | Counter | `s` | `system.device`, `disk.io.direction`; convert read/write milliseconds to seconds. |
| `system.disk.merged` | Yes | Counter | `{operation}` | `system.device`, `disk.io.direction`. |
| `system.disk.limit` | No | UpDownCounter | `By` | `system.device`. |
| `system.filesystem.usage` | Yes | UpDownCounter | `By` | `system.device`, `system.filesystem.state`, `system.filesystem.type`, `system.filesystem.mode`, `system.filesystem.mountpoint`. |
| `system.filesystem.utilization` | Yes | Gauge | `1` | Same filesystem identity attributes as usage, including `system.filesystem.state`. |
| `system.filesystem.limit` | No | UpDownCounter | `By` | Filesystem identity attributes. |
| `system.network.io` | Yes | Counter | `By` | `network.interface.name`, `network.io.direction`. |
| `system.network.packet.count` | Yes | Counter | `{packet}` | `system.device`, `network.io.direction`. |
| `system.network.packet.dropped` | Yes | Counter | `{packet}` | `network.interface.name`, `network.io.direction`. |
| `system.network.errors` | Yes | Counter | `{error}` | `network.interface.name`, `network.io.direction`. |
| `system.process.count` | Yes | UpDownCounter | `{process}` | `process.state`; v1 summary emits `running` and custom `blocked` from `/proc/stat`. |
| `system.process.created` | Yes | Counter | `{process}` | Cumulative process creations from `/proc/stat`. |

CPU time and utilization aggregate across logical CPUs by default because
`cpu.logical_number` is an opt-in attribute in the registry. `per_cpu: true`
can emit one series per `(cpu.mode, cpu.logical_number)` pair. This cardinality
is bounded by hardware topology, but it is still an explicit opt-in. CPU
frequency remains per logical CPU because `cpu.logical_number` is part of that
metric's registered shape.

V1 memory does not emit every Linux memory state named by the Collector. States
such as `inactive` and slab sub-states stay out of `system.memory.usage` unless
there is a clear registry-aligned projection; slab is reported through
`system.memory.linux.slab.usage`.

Counter start timestamps should default to host boot time for cumulative sums
whose Linux source is known to be since boot. Keep previous values by series,
detect counter resets or hot-plug replacement when a value decreases, and
re-baseline that series with a new start timestamp. Derived utilization gauges
must skip the next sample for a reset series.

Network projection has a registry inconsistency that the implementation must
preserve: `system.network.packet.count` uses `system.device` for the interface
name, while `system.network.io`, `system.network.errors`, and
`system.network.packet.dropped` use `network.interface.name`. Filters should
still operate on the Linux interface name from `/proc/net/dev`, but projection
must stamp the attribute key required by each metric.

Load average needs special handling. The semconv docs explain that UNIX load
average is not well standardized and give `system.linux.cpu.load_1m` as an
example OS-specific name, but Semantic Conventions 1.41.0 does not list a
registered load metric. Do not ship `system.cpu.load_average.1m`,
`system.cpu.load_average.5m`, or `system.cpu.load_average.15m`. If maintainers
choose an experimental Linux load metric for v1, emit raw `/proc/loadavg`
values as-is; CPU-normalized load can be a future option.

Do not emit `system.filesystem.inodes.usage`; the Collector has that metric, but
it is not present in Semantic Conventions 1.41.0. If network connection counts
are enabled after v1, use `system.network.connection.count`.

## Error Handling

Every scrape error should include:

- family;
- source path or source kind;
- low-cardinality error class.

Error classes:

- `not_found`
- `permission_denied`
- `parse`
- `unsupported`
- `timeout`
- `io`

Expected partial errors should increment metrics and be rate-limited in logs.
Examples:

- a mount disappearing between mountinfo read and `statfs`;
- a network interface disappearing between reads.

For partial scrape failures, emit the metrics that were collected successfully
and count the failed family/source with `partial_errors`. Do not reuse stale
values to hide a failed read or parse in the current tick. A family should fail
the whole tick only when no useful metric can be emitted for that due family.
The receiver should treat a scrape as fatal only when it cannot build or send
any valid metrics batch, or when the receiver runtime/control path fails.

Unexpected errors during startup validation should fail startup in
`fail_selected` mode.

Use the repo telemetry macros for logs. Do not use `println!` or raw tracing
macros. Use `otel_info!`, `otel_warn!`, and `otel_debug!` from
`otap_df_telemetry`.

## Receiver Metrics

Use `MetricSet` for receiver self-observability.

Use `Mmsc` with `ns` units for duration distribution fields to match existing
repo MetricSet duration metrics such as the traffic generator and OTAP
exporter. Use `Gauge` only if the implementation intentionally keeps a
last-value metric instead of a distribution.

Initial metric set:

| Metric | Type | Unit | Notes |
| ------ | ---- | ---- | ----- |
| `scrapes_started` | Counter | `{scrape}` | One per due tick. |
| `scrapes_completed` | Counter | `{scrape}` | Successful batch build and send. |
| `scrapes_failed` | Counter | `{scrape}` | Fatal scrape failures. |
| `families_scraped` | Counter | `{family}` | Count due families processed. |
| `scrape_duration_ns` | Mmsc | `ns` | Scrape duration distribution. |
| `scrape_lag_ns` | Mmsc | `ns` | Scheduled time to actual start. |
| `source_read_errors` | Counter | `{error}` | Attributes: `family`, `error_class`. |
| `partial_errors` | Counter | `{error}` | Attributes: `family`, `error_class`. |
| `batches_sent` | Counter | `{batch}` | Downstream sends. |
| `send_failures` | Counter | `{error}` | Attribute: `error_class`. |

Use `#[attribute_set(name = "...")]` for the low-cardinality attribute set
covering `family` and `error_class`. Do not put source paths or device names
into receiver self-observability metric attributes.

## Validation Plan

Use standard config validation tests for defaults, unknown fields, invalid
durations, and invalid v1 options. The design-specific coverage should focus on
the cases where this receiver can diverge from the intended contract:

- `HostView` normalization, root-path validation, mount-namespace behavior, and
  duplicate root-path lease conflicts.
- Disk, filesystem, and network filters, including host-perspective filesystem
  mount point matching.
- Linux parser fixtures and snapshot source sharing, especially `/proc/stat`
  reuse across CPU, system, and process metrics.
- Parser fixtures for kernel variation: missing `MemAvailable`, old and new
  `/proc/stat` CPU rows, diskstats with and without discard/flush columns, and
  `/proc/swaps`.
- Scheduler behavior for per-family intervals, skipped ticks, initial delay,
  control priority, drain, and shutdown.
- Cumulative sum temporality, `start_time_unix_nano` derived from `/proc/stat`
  `btime`, per-series reset detection, and no start timestamp on gauges.
- First-scrape behavior for derived utilization gauges: no datapoints and no
  error until a previous sample exists.
- Partial scrape behavior: emit successful families, classify the failed
  family/source, and rate-limit expected noisy failures.
- RAII lease release on normal exit, early return, and panic unwind.
- Startup rejection for `pipeline_ctx.num_cores() > 1`.
- Semantic shape tests that decode emitted OTAP metrics and verify names,
  units, instruments, temporality, monotonic flags, resource attributes, and
  data point attributes.
- Pipeline-level coverage for `receiver:host_metrics -> exporter:topic` and
  downstream `receiver:topic` fanout.

Filesystem tests should cover the blocking worker path, per-mount timeout
behavior, skipped remote and virtual filesystems, and a mount disappearing
between mountinfo read and `statfs`.

## Benchmark Plan

Add benchmarks after the first complete Linux family set is implemented.

Benchmark cases:

- all v1 families enabled at one interval;
- CPU/memory/system only;
- load only, if v1 includes an experimental Linux load metric;
- disk/filesystem/network only;
- process summary only.

Compare against the Collector hostmetrics receiver on the same host with
equivalent families. Track:

- CPU time;
- allocations;
- scrape duration;
- output series count;
- RSS over repeated scrapes.

Acceptance should be measured on the same host and configuration. The target is
at least 30 percent lower CPU time or allocation volume than the Collector
receiver for the all-families benchmark, no material regression for individual
families, and no unbounded RSS growth.

## Implementation Order

1. Add design doc, config structs, receiver registration, and empty local
   receiver lifecycle.
2. Add `HostView`, startup validation, and duplicate lease.
3. Add scheduler with tests, but use a stub collector.
4. Add OTAP metrics projection helper and semantic shape tests for simple
   gauge/sum metrics.
5. Implement CPU, memory, and system uptime.
6. Implement paging and swap.
7. Implement disk and filesystem.
8. Implement network.
9. Implement aggregate process counts.
10. Add topology docs and one-core validation.
11. Add benchmarks and compare with Collector hostmetrics receiver.

This order keeps the first functional PR small enough to review while proving
the receiver lifecycle, scheduling, root path, and OTAP emission choices before
all Linux collectors are added.

## Open Decisions

1. Whether v1 should include an experimental Linux load metric name or defer the
   load family until a registered semantic convention exists. Do not emit the
   Collector's legacy `system.cpu.load_average.*` names.
