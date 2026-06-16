# Host Metrics Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:host_metrics` (`urn:otel:receiver:host_metrics`)
- Feature gate: Default
- Stability: Experimental

## Overview

Linux host metrics receiver backed by procfs and sysfs. It emits OpenTelemetry
`system.*` metrics for CPU, memory, paging, system uptime, disk, filesystem,
network, and aggregate process counts.

## Getting Started

Start with the default Linux host metric families and a scrape interval:

```yaml
type: receiver:host_metrics
config:
  collection_interval: 10s
```

## Configuration

```yaml
type: receiver:host_metrics
config:
  # Default scrape interval (default: 10s).
  collection_interval: 10s

  # Delay before the first scrape (default: 0s).
  initial_delay: 0s

  # Host filesystem view. Use root_path: /host when reading from a mounted host
  # root inside a container.
  host_view:
    root_path: /
    validation: fail_selected # "fail_selected", "warn_selected", or "none".

  # Metric family controls. All families default to enabled except "load",
  # which defaults to disabled.
  families:
    cpu:
      enabled: true
      interval: 10s
      utilization: false
    memory:
      limit: false
      shared: false
      hugepages: false
    disk:
      limit: false
    filesystem:
      limit: false
      include_virtual_filesystems: false
      include_remote_filesystems: false
    load:
      enabled: false
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
- Process metrics are aggregate host summaries by default. Linux per-process
  metrics are available only with `processes.mode: summary_and_per_process`.
- Linux per-process metrics require `processes.process.labels.pid: true` so
  exported per-process metric series include a process identity.
- `system.process.count` emits the registered `process.state=running` summary.
  Linux `procs_blocked` is parsed but not emitted because `blocked` is not a
  registered `process.state` value.
- Filesystem collection can time out individual `statvfs` calls; avoid enabling
  remote filesystems unless the host environment is known to be healthy.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
