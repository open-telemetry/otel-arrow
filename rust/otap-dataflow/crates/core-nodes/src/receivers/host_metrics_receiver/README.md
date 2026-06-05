# Host Metrics Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:receiver:host_metrics`
- Type shortcut: `receiver:host_metrics`
- Feature gate: Default
- Stability: Experimental

## Overview

Linux host metrics receiver backed by procfs and sysfs. It emits OpenTelemetry
`system.*` metrics for CPU, memory, paging, system uptime, disk, filesystem,
network, and aggregate process counts.

## Configuration

Minimal configuration:

```yaml
type: receiver:host_metrics
config:
  collection_interval: 10s
```

Collect from a host root mounted into a container:

```yaml
type: receiver:host_metrics
config:
  collection_interval: 10s
  host_view:
    root_path: /host
    validation: fail_selected
```

Enable selected opt-in metrics:

```yaml
type: receiver:host_metrics
config:
  families:
    cpu:
      utilization: true
    memory:
      limit: true
      hugepages: true
    disk:
      limit: true
    filesystem:
      limit: true
```

## Configuration Options

| Field | Type | Default | Description |
| ----- | ---- | ------- | ----------- |
| `collection_interval` | duration | `10s` | Default scrape interval. |
| `initial_delay` | duration | `0s` | Delay before the first scrape. |
| `host_view.root_path` | path | `/` | Host filesystem root to read procfs/sysfs from. |
| `host_view.validation` | enum | `fail_selected` | One of `fail_selected`, `warn_selected`, or `none`. |
| `families.<name>.enabled` | bool | varies | Enables or disables a metric family. All families default to `true` except `load`, which defaults to `false`. |
| `families.<name>.interval` | duration | unset | Per-family interval; falls back to `collection_interval`. |
| `families.cpu.utilization` | bool | `false` | Emits derived CPU utilization gauges. |
| `families.load.enabled` | bool | `false` | Emits development-stability Linux load averages from `/proc/loadavg`. |
| `families.memory.limit` | bool | `false` | Emits `system.memory.limit`. |
| `families.memory.shared` | bool | `false` | Emits Linux shared memory. |
| `families.memory.hugepages` | bool | `false` | Emits Linux hugepage metrics. |
| `families.disk.limit` | bool | `false` | Emits disk capacity from sysfs. |
| `families.filesystem.limit` | bool | `false` | Emits filesystem capacity. |
| `families.filesystem.include_virtual_filesystems` | bool | `false` | Includes virtual filesystems such as tmpfs. |
| `families.filesystem.include_remote_filesystems` | bool | `false` | Includes remote and userspace filesystems such as NFS, CIFS, 9p, and FUSE. |

Families are `cpu`, `memory`, `paging`, `system`, `disk`, `filesystem`,
`network`, `processes`, and `load`.

Host-wide collection must run in a one-core source pipeline. Configure that at
the surrounding pipeline `policies.resources.core_allocation` level. Use a
topic exporter to fan out to multicore downstream processing when needed.

## Filters

Disk, filesystem, and network families support include and exclude filters.
Filter `match_type` values are `strict`, `glob`, and `regexp`.

```yaml
type: receiver:host_metrics
config:
  families:
    disk:
      exclude:
        match_type: glob
        devices: ["loop*", "ram*"]
    network:
      exclude:
        match_type: strict
        interfaces: ["lo"]
    filesystem:
      exclude_fs_types:
        match_type: strict
        fs_types: ["tmpfs", "proc", "sysfs"]
```

## Examples

See the minimal, mounted-host, opt-in family, and filter examples above.

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `receiver.host_metrics`

| Metric | Unit | Description |
| --- | --- | --- |
| `receiver.host_metrics.scrapes_started` | `{scrape}` | Number of scrape ticks started. |
| `receiver.host_metrics.scrapes_completed` | `{scrape}` | Number of scrape ticks that built and sent a metrics batch. |
| `receiver.host_metrics.scrapes_failed` | `{scrape}` | Number of fatal scrape failures. |
| `receiver.host_metrics.scrapes_overrun` | `{scrape}` | Number of scrape ticks skipped or timed out before completion. |
| `receiver.host_metrics.scraper_failures` | `{error}` | Number of scraper worker failures. |
| `receiver.host_metrics.partial_errors` | `{error}` | Number of source read errors skipped because other families succeeded. |
| `receiver.host_metrics.source_read_errors` | `{error}` | Number of source read errors seen during scrapes. |
| `receiver.host_metrics.families_scraped` | `{family}` | Number of due metric families processed. |
| `receiver.host_metrics.scrape_duration_ns` | `ns` | Wall-clock scrape duration. |
| `receiver.host_metrics.scrape_lag_ns` | `ns` | Delay between scheduled and actual scrape start. |
| `receiver.host_metrics.batches_sent` | `{batch}` | Number of batches sent downstream. |
| `receiver.host_metrics.send_failures` | `{error}` | Number of downstream send failures. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `host_metrics_receiver.drain_ingress` | `info` | Receiver ingress drain started. |
| `host_metrics_receiver.shutdown` | `info` | Receiver shutdown completed. |
| `host_metrics.scraper_busy` | `warn` | A scrape tick was skipped because the previous scrape was still running. |
| `host_metrics.dropped_backpressure` | `warn` | A host metrics batch was dropped because downstream was unavailable or full. |
| `host_metrics.scrape_failed` | `warn` | A scrape failed. |
| `host_metrics.scraper_stopped` | `warn` | The scraper worker stopped unexpectedly. |
| `host_metrics.scrape_timeout` | `warn` | A scrape exceeded its configured timeout. |

## Limits

- Linux only.
- `families.load.enabled` is disabled by default. When enabled, it emits
  development-stability Collector-compatible gauges:
  `system.cpu.load_average.1m`, `system.cpu.load_average.5m`, and
  `system.cpu.load_average.15m` with unit `{thread}`. Semantic Conventions
  1.41.0 does not register these names.
- `families.cpu.per_cpu` is rejected in v1.
- `families.network.include_connection_count` is rejected in v1.
- Process metrics are aggregate host summaries, not per-process scrapes.
- `system.process.count` emits the registered `process.state=running` summary.
  Linux `procs_blocked` is parsed but not emitted because `blocked` is not a
  registered `process.state` value.
- Filesystem collection can time out individual `statvfs` calls; avoid enabling
  remote filesystems unless the host environment is known to be healthy.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
