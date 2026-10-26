# Host Metrics Receiver

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `receiver:host_metrics` (`urn:otel:receiver:host_metrics`)
- Feature gate: `host-metrics`
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

Operational notes:

- `<scope>.<instrument>` is used as shorthand for metric names.

### Metric Sets

#### `receiver.host_metrics.scrapes`

Scrape outcome counters dimensioned by `outcome`.

| Scope | Instrument | Unit | Attributes | Description |
| --- | --- | --- | --- | --- |
| `receiver.host_metrics.scrapes` | `scrapes` | `{scrape}` | `outcome` | Number of scrape ticks recorded by outcome. |

Attributes:

| Attribute | Values | Description |
| --- | --- | --- |
| `outcome` | `completed`, `failed`, `timeout`, `dropped`, `cancelled` | Terminal outcome of an accepted scrape. |

#### `receiver.host_metrics.errors`

Error counters dimensioned by `error.type`.

| Scope | Instrument | Unit | Attributes | Description |
| --- | --- | --- | --- | --- |
| `receiver.host_metrics.errors` | `errors` | `{error}` | `error.type` | Number of errors encountered during scrape ticks and delivery. |

Attributes:

| Attribute | Values | Description |
| --- | --- | --- |
| `error.type` | `scraper_failure`, `partial_error`, `source_read_error`, `send_failure` | Classification of the error encountered. |

#### `receiver.host_metrics`

Non-dimensioned operational and timing metrics.

| Scope | Instrument | Unit | Description |
| --- | --- | --- | --- |
| `receiver.host_metrics` | `scrapes.started` | `{scrape}` | Number of scrape ticks started (accepted by worker). |
| `receiver.host_metrics` | `scrapes.skipped` | `{scrape}` | Scheduled ticks skipped because the worker was busy. |
| `receiver.host_metrics` | `families.scraped` | `{family}` | Number of due metric families processed. |
| `receiver.host_metrics` | `batches.sent` | `{batch}` | Number of batches sent downstream. |
| `receiver.host_metrics` | `scrape.duration.ns` | `ns` | Wall-clock scrape duration. |
| `receiver.host_metrics` | `scrape.lag.ns` | `ns` | Delay between scheduled and actual scrape start. |

### Telemetry Migration (Old to New)

All legacy flat counters were emitted under the `receiver.host_metrics` scope.
They are replaced by the dimensioned metric sets above.

| Legacy Metric | New Metric | Attribute |
| --- | --- | --- |
| `receiver.host_metrics.scrapes.started` | `receiver.host_metrics` / `scrapes.started` | -- |
| `receiver.host_metrics.scrapes.completed` | `receiver.host_metrics.scrapes` / `scrapes` | `outcome=completed` |
| `receiver.host_metrics.scrapes.failed` | `receiver.host_metrics.scrapes` / `scrapes` | `outcome=failed`, plus `outcome=timeout` for the legacy failure rate |
| `receiver.host_metrics.scrapes.overrun` | `receiver.host_metrics` / `scrapes.skipped` | -- |
| `receiver.host_metrics.scrapes.overrun` | `receiver.host_metrics.scrapes` / `scrapes` | `outcome=timeout` |
| *(new - previously uncounted)* | `receiver.host_metrics.scrapes` / `scrapes` | `outcome=dropped` |
| *(new)* | `receiver.host_metrics.scrapes` / `scrapes` | `outcome=cancelled` |
| `receiver.host_metrics.families.scraped` | `receiver.host_metrics` / `families.scraped` | -- |
| `receiver.host_metrics.batches.sent` | `receiver.host_metrics` / `batches.sent` | -- |
| `receiver.host_metrics.scrape.duration.ns` | `receiver.host_metrics` / `scrape.duration.ns` | -- |
| `receiver.host_metrics.scrape.lag.ns` | `receiver.host_metrics` / `scrape.lag.ns` | -- |
| `receiver.host_metrics.scraper.failures` | `receiver.host_metrics.errors` / `errors` | `error.type=scraper_failure` |
| `receiver.host_metrics.partial.errors` | `receiver.host_metrics.errors` / `errors` | `error.type=partial_error` |
| `receiver.host_metrics.source.read.errors` | `receiver.host_metrics.errors` / `errors` | `error.type=source_read_error` |
| `receiver.host_metrics.send.failures` | `receiver.host_metrics.errors` / `errors` | `error.type=send_failure` |

Population notes:

- The legacy `receiver.host_metrics.scrapes.overrun` counter covered two cases
  that are now recorded separately: ticks skipped because the scraper worker was
  busy map to `receiver.host_metrics` / `scrapes.skipped`, while accepted
  scrapes that exceeded their timeout map to `scrapes` with `outcome=timeout`.
  Their sum reproduces the legacy `scrapes.overrun` value.
- The legacy `receiver.host_metrics.scrapes.failed` counter also counted
  accepted scrapes that timed out. A failure-rate query that must preserve the
  legacy meaning should sum `outcome=failed` and `outcome=timeout`;
  `outcome=failed` alone excludes timeouts. Worker-start failures remain
  outside this population and only appear under `error.type=scraper_failure`.
- The legacy `receiver.host_metrics.source.read.errors` counter included both
  fatal read failures and the per-tick partial error count. Fatal read failures
  now map to `error.type=source_read_error`; per-family partial read errors now
  map to `error.type=partial_error` only.
- Batches dropped to downstream backpressure previously recorded no scrape
  outcome at all; they now map to `outcome=dropped`. Worker-start failures
  previously also incremented the legacy `receiver.host_metrics.scrapes.failed`
  counter; they now appear only in the error set under
  `error.type=scraper_failure` and stay outside the accepted-scrape outcome
  population.

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `host_metrics_receiver.drain_ingress` | `info` | Receiver ingress drain started. |
| `host_metrics_receiver.shutdown` | `info` | Receiver shutdown completed. |
| `host_metrics.scraper_busy` | `warn` | A scrape tick was skipped because the previous scrape was still running. |
| `host_metrics.dropped_backpressure` | `warn` | A host metrics batch was dropped because the downstream channel was full. |
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
