# Host Metrics Receiver

<!-- markdownlint-disable MD013 -->

**URN:** `urn:otel:receiver:host_metrics`

Linux host metrics receiver backed by procfs and sysfs. It emits OpenTelemetry
`system.*` metrics for CPU, memory, paging, system uptime, disk, filesystem,
network, and aggregate process counts.

## Configuration

Minimal configuration:

```yaml
groups:
  host:
    pipelines:
      collect:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          host_metrics:
            type: receiver:host_metrics
            config:
              collection_interval: 10s
          publish:
            type: exporter:topic
            config:
              topic: host_metrics
        connections:
          - from: host_metrics
            to: publish
```

Collect from a host root mounted into a container:

```yaml
groups:
  host:
    pipelines:
      collect:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          host_metrics:
            type: receiver:host_metrics
            config:
              collection_interval: 10s
              host_view:
                root_path: /host
                validation: fail_selected
          publish:
            type: exporter:topic
            config:
              topic: host_metrics
        connections:
          - from: host_metrics
            to: publish
```

Enable selected opt-in metrics:

```yaml
groups:
  host:
    pipelines:
      collect:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          host_metrics:
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
          publish:
            type: exporter:topic
            config:
              topic: host_metrics
        connections:
          - from: host_metrics
            to: publish
```

## Configuration Options

| Field | Type | Default | Description |
| ----- | ---- | ------- | ----------- |
| `collection_interval` | duration | `10s` | Default scrape interval. |
| `initial_delay` | duration | `0s` | Delay before the first scrape. |
| `host_view.root_path` | path | `/` | Host filesystem root to read procfs/sysfs from. |
| `host_view.validation` | enum | `fail_selected` | One of `fail_selected`, `warn_selected`, or `none`. |
| `families.<name>.enabled` | bool | `true` | Enables or disables a metric family. |
| `families.<name>.interval` | duration | unset | Per-family interval; falls back to `collection_interval`. |
| `families.cpu.utilization` | bool | `false` | Emits derived CPU utilization gauges. |
| `families.memory.limit` | bool | `false` | Emits `system.memory.limit`. |
| `families.memory.shared` | bool | `false` | Emits Linux shared memory. |
| `families.memory.hugepages` | bool | `false` | Emits Linux hugepage metrics. |
| `families.disk.limit` | bool | `false` | Emits disk capacity from sysfs. |
| `families.filesystem.limit` | bool | `false` | Emits filesystem capacity. |
| `families.filesystem.include_virtual_filesystems` | bool | `false` | Includes virtual filesystems such as tmpfs. |
| `families.filesystem.include_remote_filesystems` | bool | `false` | Includes remote and userspace filesystems such as NFS, CIFS, 9p, and FUSE. |

Families are `cpu`, `memory`, `paging`, `system`, `disk`, `filesystem`,
`network`, and `processes`.

Host-wide collection must run in a one-core source pipeline. Use a topic
exporter to fan out to multicore downstream processing when needed.

## Filters

Disk, filesystem, and network families support include and exclude filters.
Filter `match_type` values are `strict`, `glob`, and `regexp`.

```yaml
groups:
  host:
    pipelines:
      collect:
        policies:
          resources:
            core_allocation:
              type: core_count
              count: 1
        nodes:
          host_metrics:
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
          publish:
            type: exporter:topic
            config:
              topic: host_metrics
        connections:
          - from: host_metrics
            to: publish
```

## Current Limits

- Linux only.
- Load metrics are not emitted in v1 because Semantic Conventions 1.41.0 does
  not register a system load metric.
- `families.cpu.per_cpu` is rejected in v1.
- `families.network.include_connection_count` is rejected in v1.
- Process metrics are aggregate host summaries, not per-process scrapes.
- `system.process.count` emits the registered `process.state=running` summary.
  Linux `procs_blocked` is parsed but not emitted because `blocked` is not a
  registered `process.state` value.
- Filesystem collection can time out individual `statvfs` calls; avoid enabling
  remote filesystems unless the host environment is known to be healthy.
