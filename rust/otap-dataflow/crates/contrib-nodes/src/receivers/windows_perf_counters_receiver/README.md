# Windows Performance Counter Receiver

## Metadata

- Type: `receiver:windows_perf_counters` (`urn:otel:receiver:windows_perf_counters`)
- Feature gate: `windows-perf-counters` (also enabled by `contrib-receivers`)
- Platform: Windows only
- Execution: One-core source pipeline only
- Stability: Experimental

## Overview

This receiver collects Windows performance counters from the local machine
using [Performance Data Helper (PDH) functions][using-pdh] and emits them as
OpenTelemetry metrics.

Each counter is identified by a path built from a performance object, an
optional instance, and a counter name, such as
`\Processor(_Total)\% Processor Time`. An instance is one item of the object,
such as the first processor in `\Processor(0)\% Processor Time` or the
`_Total` aggregate.

To find the object, instance, and counter names available on a host, use
PowerShell. For example:

```powershell
Get-Counter -ListSet Processor | Select-Object -ExpandProperty Counter
Get-Counter '\Processor(_Total)\% Processor Time'
```

| Supported | Not supported |
| --- | --- |
| Live counters | Counter log files (`.blg`) |
| Counters on the local machine | Remote machines (`\\host\...` paths); rejected at load |
| English object and counter names on any Windows display language | Localized (non-English) object and counter names |
| Exact instances and instance patterns (`*`, `?`), such as `*` for all instances | Wildcards in object or counter names, including a whole-counter `"*"`; rejected at load |
| Gauge and Sum metrics (cumulative or delta, monotonic or not) | Histogram, exponential histogram, and summary metrics |
| Static data point attributes per counter | Resource attributes such as `host.name`; add them with a processor. See [Data Point Attributes](#data-point-attributes). |
| One `collection_interval` per receiver | Per-counter intervals; use a separate receiver for each interval, as in [Collecting at Different Intervals](#collecting-at-different-intervals) |

The receiver picks up instances as they come and go. Some counters exist only
on some hosts, so a counter missing at startup produces a warning, not a
failure, and is retried at every collection. See
[Runtime Behavior](#runtime-behavior).

## Getting Started

A minimal receiver configuration:

```yaml
perf_counters:
  type: receiver:windows_perf_counters
  config:
    metrics:
      windows.memory.available:
        unit: By
        description: Physical memory immediately available for allocation.
        gauge: {}
    perfcounters:
      - object: Memory
        counters:
          - name: Available Bytes
            metric: windows.memory.available
    collection_interval: 30s
```

Run the receiver in a pipeline allocated exactly one core; see
[Limits](#limits).

The `metrics` and `perfcounters` sections are linked by metric name:

- **`metrics`** defines each metric. The entry's key is the emitted metric
  name, and the entry sets its `unit`, `description`, and type (`gauge` or
  `sum`).
- **`perfcounters`** lists the Windows counters to read. Each counter's
  `metric` field names the **`metrics`** entry it sends values to.

With this configuration, every 30 seconds the receiver reads
`\Memory\Available Bytes` and emits it as `windows.memory.available`, using
that entry's `unit`, `description`, and type:

```text
resource:
  attributes:
    os.type: windows
metric:
  name: windows.memory.available
  unit: By
  description: Physical memory immediately available for allocation.
  type: gauge
  data_point:
    value: <value of \Memory\Available Bytes>
```

## Configuration

A configuration has three parts: receiver options, `metrics`, and
`perfcounters`.

```yaml
config:
  collection_interval: 1m             # Receiver Options
  initial_delay: 1s
  timeout: 1m
  metrics:
    windows.processor.time:           # Metric Options
      unit: "%"
      description: Processor time by state.
      gauge: {}                       # Gauge or Sum
  perfcounters:
    - object: Processor               # Object Options
      instances: ["*"]                # Selecting Instances
      aggregation_name: _Total        # Aggregation Instances
      counters:
        - name: "% Processor Time"    # Counter Options
          metric: windows.processor.time
          attributes:                 # Data Point Attributes
            state: active
          scale_power10: 0            # Scaling Values
          recreate_query: false       # Recreating the Query
```

Values outside the limits in the option tables, and the cases in
[Configuration Requirements](#configuration-requirements), are rejected at
load. To check a configuration before running it, see
[Validating a Configuration](#validating-a-configuration).

### Receiver Options

`collection_interval`, `initial_delay`, and `timeout` take a time with a unit,
such as `500ms`, `30s`, `1m30s`, or `2h`. Numbers without a time unit, negative
values, and values longer than 24 hours are rejected. For all accepted formats,
see [humantime][humantime].

<!-- markdownlint-disable MD013 -->

| Option | Default | Description |
| --- | --- | --- |
| `collection_interval` | `1m` | How often to collect. Must be greater than zero. |
| `initial_delay` | `1s` | How long to wait before the first collection. Leave empty or set to `0s` for no delay. |
| `timeout` | `collection_interval` | How long a collection can run before the receiver warns. The collection is not cancelled. Leave empty or set to `0s` to use the default. |
| `metrics` | | Metric definitions, keyed by metric name. Set a counter's `metric` field to the key of a definition to use it. Unused definitions emit nothing but are still validated. |
| `perfcounters` | Required | Performance objects to collect. Must contain at least one entry. |

<!-- markdownlint-enable MD013 -->

### Metric Options

Each `metrics.<name>` entry defines the metric emitted as `<name>`. Set `gauge`
or a `sum` mapping, not both. An entry without `gauge` or a `sum` mapping
defaults to a Gauge.

<!-- markdownlint-disable MD013 -->

| Option | Default | Description |
| --- | --- | --- |
| `unit` | `"1"` | Metric unit, such as `By`, `%`, or `{operation}`. |
| `description` | `""` | What the metric measures. Backends can show it alongside the name, so set it when the name alone is not clear. |
| `gauge` | | Emit a Gauge. Write `gauge:` or `gauge: {}`. |
| `sum` | | Emit a Sum. Write `sum:` with the settings below. |
| `sum.aggregation` | Required | `cumulative` or `delta`. |
| `sum.monotonic` | `false` | `true` or `false`. |

<!-- markdownlint-enable MD013 -->

The `unit` and `description` should describe what the counter actually
measures. For example, Windows `Private Bytes` is committed memory, which may
be in RAM or the page file, so do not label it as physical memory usage.

#### Gauge or Sum

Both types emit the counter's value as read at each collection. The type tells
downstream tools how to combine values from different instances.

**Gauge.** The value is a reading, such as a percentage, rate, or ratio.
Readings can be averaged across instances but not added. Most counters are
Gauges. For example:

- Counters: `% Processor Time`, `Pages/sec`, `Avg. Disk Queue Length`.
- On a machine with two processors, if processor 0 is 80% busy and
  processor 1 is 20% busy, the machine is 50% busy, not 100%.

**Sum.** The value is an amount, such as bytes, handles, or threads.
Amounts can be added across instances to get a total. For example:

- Counters: `Private Bytes`, `Handle Count`, `Thread Count`.
- If one process has committed 400 MB and another 100 MB, together they
  have committed 500 MB.

The `sum` settings describe the values to downstream tools; the receiver emits
each value unchanged:

- `aggregation`: `cumulative` means each value is the full current amount.
  `delta` means each value is the change since the previous collection.
  Amount counters such as `Private Bytes` report the full current amount,
  and the receiver does not convert it, so use `cumulative` for them.
- `monotonic`: `true` means the amount only increases
  (e.g. `\TCPv4\Connection Failures`, a count from when the system started).
  Downstream tools can then calculate a rate from it and treat a decrease as a
  reset. `false` means it can also decrease (e.g. `Private Bytes`,
  which tracks the memory a process currently holds).

For a complete Sum, see [Collecting Every Process](#collecting-every-process).

### Object Options

Fields of each `perfcounters[]` entry:

<!-- markdownlint-disable MD013 -->

| Option | Default | Description |
| --- | --- | --- |
| `object` | Required | English name of the performance object, such as `Processor` or `Process`. |
| `instances` | | Instance names or patterns to collect. See [Selecting Instances](#selecting-instances). |
| `aggregation_name` | `_Total` | Name of the object's aggregate instance. See [Aggregation Instances](#aggregation-instances). |
| `counters` | Required | Counters to read from this object. Must contain at least one entry. |

<!-- markdownlint-enable MD013 -->

#### Selecting Instances

Set `perfcounters[].instances` to choose what to collect:

<!-- markdownlint-disable MD013 -->

| `instances` example | What it collects |
| --- | --- |
| Omitted, `""`, or `[]` | An object without instances, such as `Memory` |
| `["C:", "D:"]` | The named instances, such as drives `C:` and `D:` of `LogicalDisk` |
| `["_Total"]` | Only the aggregate (`_Total`) |
| `["w3wp*"]` or `["disk?"]` | Instances matching the pattern at each collection. `*` matches any run of characters; `?` matches one character. |
| `["*"]` | All instances present at each collection, excluding the `aggregation_name` unless it is the only instance with a value |
| `["*", "_Total"]` | All instances present at each collection, including the listed aggregate (`_Total`) |

<!-- markdownlint-enable MD013 -->

`instances` is a YAML list (`["0", "_Total"]`) or a comma-separated string
(`"0,_Total"`). In a string, do not add spaces around commas; they become part
of the names. Empty entries, such as `["0", ""]` or `"0,"`, are rejected.

Use the instance names Windows reports. For example, a `Paging File` instance
may be `C:\pagefile.sys` or `\??\C:\pagefile.sys`. To find the paths on a host,
run `(Get-Counter -ListSet 'Paging File').PathsWithInstances`, replacing
`Paging File` with the object name. A `?` in a name is still a wildcard:
`\??\C:\pagefile.sys` also matches `\ab\C:\pagefile.sys`.

#### Aggregation Instances

The aggregate instance, `_Total` by default, combines the values of the other
instances. A pattern that matches the aggregate and other instances skips the
aggregate, so a downstream sum does not count it twice. For example, if two
processes hold 100 MB and 50 MB, `_Total` is 150 MB; adding all three would give
300 MB. To keep the aggregate anyway, list it beside `*`, as in
`["*", "_Total"]`, and prefer deriving totals downstream.

Whether the aggregate has an `instance` attribute depends on how it is
selected:

- Opted in beside `*`, as in `["*", "_Total"]`: always `instance=_Total`.
- Listed by name, as in `["_Total"]` or `["0", "_Total"]`: no `instance`
  attribute.
- Matched by a pattern, as in `["*"]`: skipped, unless it is the only
  instance with a value; then it is reported with no `instance` attribute.

For example, under `object: Processor` on a host with two processors:

```text
instances: ["*"]            ->  value=12.5 instance=0
                                value=37.5 instance=1
instances: ["*", "_Total"]  ->  value=12.5 instance=0
                                value=37.5 instance=1
                                value=25.0 instance=_Total
instances: ["0", "_Total"]  ->  value=12.5 instance=0
                                value=25.0               (no instance attribute)
instances: ["_Total"]       ->  value=25.0               (no instance attribute)
```

If an object names its aggregate differently, set `aggregation_name`. For
example, `Thread` names its aggregate `_Total/_Total`:

```yaml
- object: Thread
  instances: ["*"]
  aggregation_name: _Total/_Total
  counters:
    - name: Context Switches/sec
```

### Counter Options

Fields of each `perfcounters[].counters[]` entry:

<!-- markdownlint-disable MD013 -->

| Option | Default | Description |
| --- | --- | --- |
| `name` | Required | English counter name, such as `% Processor Time`. |
| `metric` | Counter path | The `metrics` key to send values to. If omitted, see [Naming Metrics by Counter Path](#naming-metrics-by-counter-path). |
| `attributes` | | String key-value pairs added to every point, such as `state: idle`. See [Data Point Attributes](#data-point-attributes). |
| `scale_power10` | `0` | Multiply each value by 10 to this power, from `-18` to `18`. See [Scaling Values](#scaling-values). |
| `recreate_query` | `false` | Rebuild the counter's query after each collection. Use only for a counter that reports wrong values. See [Recreating the Query](#recreating-the-query). |

<!-- markdownlint-enable MD013 -->

Counters can send values to the same metric only if their points differ by a
static attribute or by instance name. See
[Sharing a Metric Between Counters](#sharing-a-metric-between-counters).

### Configuration Requirements

Besides the limits in the above option tables, a configuration must meet the
following requirements, or the receiver rejects it at load.

#### Fields and Names

- Unknown fields are rejected.
- Object, instance, and counter names must not be empty or contain a NUL
  character.
- Object and counter names must not contain `\`, `*`, or `?`. Remote paths
  (`\\host\...`) are not supported, and wildcards work only in `instances`.
- Object names must not contain parentheses. Set instances in `instances`: use
  `object: Processor` with `instances: ["0"]`, not `object: Processor(0)`.
  Parentheses in instance names must be balanced.

#### Metrics and Attributes

- A counter's `metric` must match a key under `metrics`.
- A metric must not set both `gauge` and a `sum` mapping, and a `sum` mapping
  must set `aggregation`.
- A counter without `metric` emits a Gauge named by its counter path. If another
  counter sends values to a `metrics` entry with that name, the entry must be a
  Gauge with unit `1` and no description.
- Static attribute keys must not be empty. A static `instance` attribute must
  not be combined with a pattern, because every matched instance would get the
  same label.
- Every point of a metric must be unique. Points on the same metric must differ
  by static attributes or by instance name (ignoring case):
  - Objects without instances, and an aggregate listed by name, have no
    instance name, so they must differ by static attributes.
  - A pattern counts as matching every instance, so it must differ from every
    other selection by static attributes, even a pattern that never matches the
    same instances, such as `w3wp*` and `sql*`. See
    [Sharing a Metric Between Counters](#sharing-a-metric-between-counters).

### Validating a Configuration

Use `--validate-and-exit` to check configuration without starting the receiver:

```powershell
.\df_engine.exe --validate-and-exit -c .\windows-perf-counters-console.yaml
```

This does not check counter availability. A misspelled or unavailable counter
does not prevent startup: the receiver warns with `counter_add.fail` and retries
it. See [Missing Counters](#missing-counters).

## Examples

Each example shows only the receiver node. Some link to a runnable file with
the complete pipeline. To run one, from `rust\otap-dataflow`:

```powershell
cargo run --features windows-perf-counters --bin df_engine -- -c configs\windows-perf-counters-console.yaml
```

### Collecting Multiple Counters

Collect total processor utilization alongside available memory:

```yaml
perf_counters:
  type: receiver:windows_perf_counters
  config:
    metrics:
      windows.memory.available:
        unit: By
        description: Physical memory immediately available for allocation.
        gauge: {}
      windows.processor.time:
        unit: "%"
        description: Average processor utilization across all logical processors.
        gauge: {}
    perfcounters:
      - object: Memory
        counters:
          - name: Available Bytes
            metric: windows.memory.available
      - object: Processor
        instances: ["_Total"]
        counters:
          - name: "% Processor Time"
            metric: windows.processor.time
    collection_interval: 30s
```

| Windows counter | Emitted metric | Type |
| --- | --- | --- |
| `\Memory\Available Bytes` | `windows.memory.available` | Gauge |
| `\Processor(_Total)\% Processor Time` | `windows.processor.time` | Gauge |

Runnable file:
[`windows-perf-counters-console.yaml`](../../../../../configs/windows-perf-counters-console.yaml).

### Collecting Every Process

Collect private memory for every running process alongside available memory.
`instances: ["*"]` picks up processes as they start and stop. Private memory can
be added up across processes, so the metric is a Sum:

```yaml
perf_counters:
  type: receiver:windows_perf_counters
  config:
    metrics:
      windows.memory.available:
        unit: By
        description: Physical memory immediately available for allocation.
        gauge: {}
      windows.process.private:
        unit: By
        description: Committed private memory for each process instance.
        sum:
          aggregation: cumulative
          monotonic: false
    perfcounters:
      - object: Memory
        counters:
          - name: Available Bytes
            metric: windows.memory.available
      - object: Process
        instances: ["*"]
        counters:
          - name: Private Bytes
            metric: windows.process.private
    collection_interval: 5s
```

Each process becomes its own series, identified by its `instance` attribute,
such as `svchost` or `svchost#1`. See
[Process Instances](#process-instances).

Runnable file:
[`windows-perf-counters-wildcard-console.yaml`](../../../../../configs/windows-perf-counters-wildcard-console.yaml).

### Collecting at Different Intervals

Each receiver has one `collection_interval`. To collect some counters more
often than others, configure one receiver per interval in the same pipeline.
These receivers collect available memory every 30 seconds and processor time
for each processor every minute:

```yaml
memory:
  type: receiver:windows_perf_counters
  config:
    metrics:
      windows.memory.available:
        unit: By
        description: Physical memory immediately available for allocation.
        gauge: {}
    perfcounters:
      - object: Memory
        counters:
          - name: Available Bytes
            metric: windows.memory.available
    collection_interval: 30s
processor:
  type: receiver:windows_perf_counters
  config:
    metrics:
      windows.processor.time:
        unit: "%"
        description: Processor utilization for each logical processor.
        gauge: {}
    perfcounters:
      - object: Processor
        instances: ["*"]
        counters:
          - name: "% Processor Time"
            metric: windows.processor.time
    collection_interval: 1m
```

Receivers collect independently, so a counter configured in two receivers is
collected twice. Give each counter to one receiver.

### Sharing a Metric Between Counters

Report active and idle processor time under one metric, told apart by a
static `state` attribute:

```yaml
perf_counters:
  type: receiver:windows_perf_counters
  config:
    metrics:
      windows.processor.time:
        unit: "%"
        description: Processor time by state.
        gauge: {}
    perfcounters:
      - object: Processor
        instances: ["_Total"]
        counters:
          - name: "% Processor Time"
            metric: windows.processor.time
            attributes:
              state: active
          - name: "% Idle Time"
            metric: windows.processor.time
            attributes:
              state: idle
    collection_interval: 30s
```

Each collection emits two points of `windows.processor.time`, one with
`state: active` and one with `state: idle`.

To put two patterns under one metric, such as `w3wp*` and `sql*`, use a
separate `perfcounters` entry for each, with a different static attribute.
Listing both in one `instances` list is rejected, even though they never match
the same process:

```yaml
perfcounters:
  - object: Process
    instances: ["w3wp*"]
    counters:
      - name: Private Bytes
        metric: windows.process.private
        attributes:
          workload: iis
  - object: Process
    instances: ["sql*"]
    counters:
      - name: Private Bytes
        metric: windows.process.private
        attributes:
          workload: sql
```

Each point then has both its `instance` and `workload` attributes.

### Scaling Values

Report available memory in decimal megabytes instead of bytes:

```yaml
perf_counters:
  type: receiver:windows_perf_counters
  config:
    metrics:
      windows.memory.available:
        unit: MBy
        description: Physical memory immediately available for allocation.
        gauge: {}
    perfcounters:
      - object: Memory
        counters:
          - name: Available Bytes
            metric: windows.memory.available
            scale_power10: -6
    collection_interval: 30s
```

`scale_power10: -6` multiplies each value by 10^-6, so `8,589,934,592` bytes is
emitted as `8589.934592`. The `unit` is `MBy` to match the scaled value.

### Naming Metrics by Counter Path

Omit a counter's `metric` field to emit a Gauge named by its counter path, with
unit `1` and no description:

```yaml
perf_counters:
  type: receiver:windows_perf_counters
  config:
    perfcounters:
      - object: Memory
        counters:
          - name: Available Bytes
            # metric: omitted
    collection_interval: 30s
```

This emits a Gauge named `\Memory\Available Bytes`.

With exact instances, each instance gets a metric named by its own path, such as
`\Processor(0)\% Processor Time`. With a pattern, all matching instances share
one metric named with the pattern, such as `\Process(*)\Private Bytes`. To set a
name, `unit`, `description`, or type, define a metric under `metrics` instead.

## Output

### Data Point Attributes

When the object has instances, each data point has an `instance` attribute
with the instance name, such as `0` or `C:`, except the aggregate in some
cases (see [Aggregation Instances](#aggregation-instances)). Processes that
share a name are labeled `svchost`, `svchost#1`, and so on. See
[Process Instances](#process-instances).

Points also have the counter's static `attributes`, alongside `instance`.
Prefer [OpenTelemetry semantic convention][semconv] names where they fit. A
static `instance` attribute replaces the instance name: under
`object: Processor` with `instances: ["0"]`, `attributes: {instance: cpu0}`
emits `instance: cpu0` instead of `0`.

Every batch has resource attribute `os.type: windows` and instrumentation
scope `otel-arrow-dfe-contrib-nodes/windows_perf_counters`.

The receiver sets no `host.*` attributes. To add host identity, place an
attribute processor between the receiver and the exporter. For example:

```yaml
nodes:
  perf_counters:
    type: receiver:windows_perf_counters
    config:
      # Receiver configuration, as in Getting Started.
  host_identity:
    type: processor:attribute
    config:
      apply_to: ["resource"]
      actions:
        - action: upsert
          key: host.name
          value: "${env:COMPUTERNAME}"
  console:
    type: exporter:console
    config: {}
connections:
  - from: perf_counters
    to: host_identity
  - from: host_identity
    to: console
```

### Process Instances

Processes that share a name are labeled `name`, `name#1`, `name#2`, and so on,
in the order Windows returns them. The numbers follow position, not process: as
those processes start and stop, a numbered label may move to a different
process, so one series may briefly mix values from both. See
[Handling Duplicate Instance Names][duplicate-instances].

Use `Process V2` instead where the host has it. Its instances include the
process ID (`name:PID`, such as `svchost:1234` rather than `svchost#1`), so
each process keeps its own label. To check whether a host has it:

```powershell
Get-Counter -ListSet 'Process V2'
Get-Counter '\Process V2(*)\Private Bytes' -SampleInterval 1 -MaxSamples 1
```

### Values

- Windows calculates each value from the counter's type, so values match what
  `Get-Counter` shows. Raw counter data, such as running totals for calculating
  rates downstream, is not available. Configure the visible counter, not its
  hidden base counter.
- All values are emitted as doubles.
- `scale_power10` is applied after the calculation. The unit should describe
  the scaled value.
- Percentages are not capped at 100. For example,
  `\Process(*)\% Processor Time` can exceed 100 on multi-core hosts. See
  [Troubleshooting](#troubleshooting).
- A cumulative Sum's start time is when the receiver finished starting, not
  when the underlying process or provider started. It stays the same while the
  receiver runs, unless the system clock moves back before it, in which case it
  moves back too.

## Runtime Behavior

### Startup and First Values

The first collection runs `initial_delay` after the receiver starts, or when
startup finishes if that takes longer, and then every `collection_interval`.

Windows calculates rate, percentage, and average counters from two samples. The
receiver takes the first sample of these counters at startup without
emitting it, so they report from the first collection. A new instance, or a
counter added later by a retry, takes its first sample the same way and appears
one collection later. These counters need about `1s` or more between
samples to produce meaningful values.

### When a Counter Has No Value

One counter's problem never stops the others. Each problem logs one warning
until the counter recovers, not one per attempt.

#### Missing Counters

- A counter that cannot be added, for example because it is misspelled or
  not installed, logs `counter_add.fail` and is retried at the start of every
  collection until it is added.
- A counter that cannot be read logs `counter_read.fail` and is skipped for
  that collection. An instance array larger than 64 MiB cannot be read.
- Each collection keeps at most 100,000 instances across all counters, in
  configuration order. A counter whose instances would go over that is
  skipped whole for that collection and logs `counter_read.fail` with status
  `0xc0000bc2` (`PDH_INSUFFICIENT_BUFFER`). Split counters across receivers
  to collect more.
- If a counter set is installed while the receiver runs, the retries may keep
  failing until the engine (`df_engine`) restarts.

#### No Value This Collection

Some collections legitimately produce no value for a counter. The point is
left out rather than reported as zero, and no event is logged:

- A rate, percentage, or average counter that is still taking its first sample.
- Windows reports no data, invalid data, or a negative denominator when reading
  the counter's values, or any error status for an instance.

If an entire collection produces no points, no batch is sent.

#### Collection Failures

If a counter cannot be collected, it misses that collection. The receiver logs
`collect.fail` with the counter's path and counts the collection in
`scrape.failures`. No restart is needed: the next collection sets the counter up
again, and its values return on the collection after that. If Windows reports a
negative denominator while collecting the counter's query, the receiver waits
one second and tries once more before treating it as a failure. This delays the
rest of that collection.

### Slow Collections

Only one collection runs at a time. If a collection is still running when the
next one is due, the next one is skipped, not queued, and counted in
`scrape.overruns`. A collection still running after `timeout` (by default,
`collection_interval`) logs `scrape.overrun`, which repeats only after a
collection finishes on time. Late results are still sent with their original
timestamps.

### Recreating the Query

On some versions of Windows, a counter's query can become corrupted and keep
reporting wrong values after the first collection (reported on Windows Server
2012 with wildcard `Process` counters). If that happens, set the counter's
`recreate_query` to `true` to create a fresh query after each successful
collection.

If the new query cannot be built, the receiver keeps the old one and warns with
`recreate_query.fail`. Rebuilding the query each time makes collections take
longer and can lead to `scrape.overrun` warnings, so set `recreate_query` only
on counters that need it.

### Dropped Batches

If downstream components cannot accept a batch, the batch is dropped, counted
in `batches.dropped`, and logged as `batch.dropped`. The warning repeats only
after a batch is accepted.

### Shutdown

On shutdown the receiver waits up to one second for collection to stop, or less
if the pipeline's shutdown deadline is closer. A collection in progress skips
the counters it has not reached yet. Windows performance counter calls cannot be
interrupted, so if a call is blocked, the receiver logs `shutdown.timeout` and
releases resources when the call returns. When the pipeline drains, a
collection still in progress is not sent.

## Troubleshooting

<!-- markdownlint-disable MD013 -->

| Symptom | Likely cause and fix |
| --- | --- |
| A counter never reports | Look for `counter_add.fail` or `counter_read.fail` with its `path` and `status`. Check the path with `Get-Counter` and use English names. |
| Receiver fails with "requires a one-core source pipeline" | Set the pipeline's `core_allocation` to `{type: core_count, count: 1}`. See [Limits](#limits). |
| A counter misses collections | Look for `collect.fail` with that counter's `path`. It is retried automatically, and values return within two collections. |
| A counter reports wrong values after the first collection | Set `recreate_query: true` on that counter. See [Recreating the Query](#recreating-the-query). |
| A new instance or recovered rate, percentage, or average counter skips one collection | Expected; it is taking its first sample. |
| An instance appears under two metric names | Two `instances` entries of a counter without `metric` match it, such as `instances: ["0", "?"]` under `object: Processor`, which both match `\Processor(0)\% Processor Time`. Remove one. |
| `% Processor Time` above 100 | Expected for `Process` values on multi-core hosts. A process's time is summed across logical processors, so one using two cores fully reads 200%. Divide by the number of logical processors for a 0-100% value. |
| `scrape.overrun` warnings | Collection takes longer than `timeout`. Reduce the number of counters, narrow patterns, or increase `collection_interval`. Collections are also slower when counters set `recreate_query: true`; see [Recreating the Query](#recreating-the-query). |
| `batch.dropped` warnings | Downstream components are not keeping up. Increase `collection_interval` or check the downstream pipeline. |
| A counter set installed after startup is not collected | Restart the engine (`df_engine`). |

<!-- markdownlint-enable MD013 -->

## Telemetry

Self-metrics and events use *scrape* for a collection.

### Metric Set

#### `receiver.windows_perf_counters`

<!-- markdownlint-disable MD013 -->

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.windows_perf_counters.scrapes` | `{scrape}` | Collections in which at least one counter was collected successfully. |
| `receiver.windows_perf_counters.scrape.failures` | `{scrape}` | Collections in which at least one counter could not be collected, plus startup if a counter's query could not be opened or sampled. A collection where some counters succeed and others fail counts once in both `scrapes` and `scrape.failures`. |
| `receiver.windows_perf_counters.scrape.overruns` | `{tick}` | Collections skipped because the previous collection was still running. |
| `receiver.windows_perf_counters.scrape.duration` | `s` | Time from requesting a collection to receiving its results. |
| `receiver.windows_perf_counters.counter.add.failures` | `{failure}` | Failures to add a counter, including retries. |
| `receiver.windows_perf_counters.active.counters` | `{counter}` | Counters currently added, whether or not they produce values. |
| `receiver.windows_perf_counters.batches.dropped` | `{batch}` | Batches dropped because downstream was full or closed. |

<!-- markdownlint-enable MD013 -->

### Events

Event names start with `windows_perf_counters_receiver.`. Each is logged
once per problem and again only after it recovers. `status` is the Windows
[PDH error code][pdh-errors] in hexadecimal. Every event also includes
`node`, the receiver's node name. Counter events also include
`counter_index`, the counter's position among all configured counter paths, to
distinguish counters that share the same `path`.

<!-- markdownlint-disable MD013 -->

| Event | Severity | Description |
| --- | --- | --- |
| `counter_add.fail` | warn | A counter could not be added. Fields: `counter_index`, `metric`, `path`, `status`. |
| `counter_read.fail` | warn | A counter could not be read. Fields: `counter_index`, `metric`, `path`, `status`. |
| `collect.fail` | warn | A counter's query could not be opened or collected. Fields: `counter_index`, `metric`, `path`, `status`. |
| `recreate_query.fail` | warn | A counter's query could not be rebuilt; the old one is kept. Fields: `counter_index`, `metric`, `path`, `status`. |
| `scrape.overrun` | warn | A collection ran past `timeout`. Fields: `collection_interval_seconds`, `timeout_seconds` (`0` when `timeout` is not set). |
| `batch.dropped` | warn | A batch was dropped because downstream was full or closed. |
| `shutdown.timeout` | warn | Shutdown did not finish within its time limit. |

<!-- markdownlint-enable MD013 -->

## Limits

- Windows only.
- The pipeline must be allocated exactly one core. The engine defaults to all
  cores, so set it explicitly in the pipeline's `policies`:

  ```yaml
  groups:
    default:
      pipelines:
        main:
          policies:
            resources:
              core_allocation: {type: core_count, count: 1}
  ```

  Otherwise the receiver fails at startup, even though `--validate-and-exit`
  passes. Use a topic exporter if downstream processing needs more cores.
- Missing or misspelled counters are reported at runtime, not at load; see
  [Missing Counters](#missing-counters).
- English object and counter names only.
- A counter set installed while the receiver runs may need an engine
  (`df_engine`) restart; see [Missing Counters](#missing-counters).
- Each collection keeps at most 100,000 instances; see
  [Missing Counters](#missing-counters). Prefer narrow patterns, such as
  `w3wp*` rather than `*`, on busy hosts.
- `Process` instance labels for processes that share a name can move between
  processes. Use `Process V2` where available; see
  [Process Instances](#process-instances).

## Related Docs

- [Windows Performance Counters][performance-counters]
- [Using the PDH functions to consume counter data][using-pdh]
- [Calculating counter values][counter-types]
- [PDH error codes][pdh-errors]

[counter-types]: https://learn.microsoft.com/windows/win32/perfctrs/calculating-counter-values
[duplicate-instances]: https://learn.microsoft.com/windows/win32/perfctrs/handling-duplicate-instance-names
[humantime]: https://docs.rs/humantime/latest/humantime/fn.parse_duration.html
[pdh-errors]: https://learn.microsoft.com/windows/win32/perfctrs/pdh-error-codes
[performance-counters]: https://learn.microsoft.com/windows/win32/perfctrs/performance-counters-portal
[semconv]: https://opentelemetry.io/docs/specs/semconv/
[using-pdh]: https://learn.microsoft.com/windows/win32/perfctrs/using-the-pdh-functions-to-consume-counter-data
