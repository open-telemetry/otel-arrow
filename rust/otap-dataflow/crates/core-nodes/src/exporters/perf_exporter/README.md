# Perf Exporter

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `exporter:perf` (`urn:otel:exporter:perf`)
- Feature gate: Default
- Stability: experimental

## Overview

The perf exporter reports pipeline throughput and optional process or host usage
statistics. It is mainly intended for local benchmarks and performance
experiments.

## Getting Started

Use the perf exporter at the end of a benchmark pipeline:

```yaml
type: exporter:perf
config:
  frequency: 1000
  smoothing_factor: 0.3
  self_usage: true
  cpu_usage: true
  mem_usage: true
  disk_usage: true
  io_usage: true
```

## Configuration

```yaml
type: exporter:perf
config:
  # Report interval in milliseconds (default: 1000).
  frequency: 1000

  # Exponential moving average smoothing (default: 0.3).
  smoothing_factor: 0.3

  # Report process usage (default: true).
  self_usage: true

  # Report CPU usage (default: true).
  cpu_usage: true

  # Report memory usage (default: true).
  mem_usage: true

  # Report disk usage (default: true).
  disk_usage: true

  # Report network and I/O usage (default: true).
  io_usage: true
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

Input PData message volume is reported by the engine through
`channel.receiver.recv.count` on the PData input channel and is not duplicated
by the exporter.

#### `exporter.pdata.exports`

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `exporter.pdata.exports.messages` | `{message}` | `signal`, `outcome` | Number of PData messages whose export reached a terminal outcome. |

#### `exporter.perf.pdata`

| Metric | Unit | Description |
| --- | --- | --- |
| `exporter.perf.pdata.invalid_batches` | `{msg}` | Number of invalid pdata batches received. |
| `exporter.perf.pdata.logs` | `{log}` | Number of logs received. |
| `exporter.perf.pdata.spans` | `{span}` | Number of spans received. |
| `exporter.perf.pdata.metrics` | `{metric}` | Number of metrics received. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `perf_exporter.start` | `info` | Exporter startup with the configured report interval. |

## Limits

- Output is designed for benchmark inspection, not as a stable telemetry
  export format.
- Host usage counters depend on platform support exposed to the process.

## Example Output

```text
====================Pipeline Report====================
    - arrow records throughput          : 0.00 arrow-records/s
    - average pipeline latency          : 0.00 s
    - total arrow records received      : 0
    - otlp signal throughput            : 0.00 otlp/s
    - total otlp signal received        : 0
    - pdata batch throughput            : 0.00 pdata/s
    - total pdata batch received        : 0
=====================Memory Usage======================
    - memory rss                        : 28.74 MB
    - memory virtual                    : 422.19 GB
=======================Cpu Usage=======================
    - global cpu usage                  : 0% (100% is all cores)
    - process cpu usage                 : 14.2799% (100% is a single core)
======================Disk Usage=======================
    - read bytes                        : 0 B/s
    - total read bytes                  : 0 B
    - written bytes                     : 0 B/s
    - total written bytes               : 0 B
=====================Network Usage=====================
Network Interface: lo0
    - bytes read                        : 0 B/s
    - total bytes received              : 4.07 GB
    - bytes transmitted                 : 0 B/s
    - total bytes transmitted           : 4.07 GB
    - packets received                  : 0 B/s
    - total packets received            : 13.44 MB
    - packets transmitted               : 0 B/s
    - total packets transmitted         : 13.44 MB
    - errors on received                : 0 B/s
    - total errors on received          : 0 B
    - errors on transmitted             : 0 B/s
    - total errors on transmitted       : 0 B
Network Interface: utun3
    - bytes read                        : 0 B/s
    - total bytes received              : 0 B
    - bytes transmitted                 : 0 B/s
    - total bytes transmitted           : 4.40 KB
    - packets received                  : 0 B/s
    - total packets received            : 0 B
    - packets transmitted               : 0 B/s
    - total packets transmitted         : 27 B
    - errors on received                : 0 B/s
    - total errors on received          : 0 B
    - errors on transmitted             : 0 B/s
    - total errors on transmitted       : 0 B
Network Interface: anpi0
    - bytes read                        : 0 B/s
    - total bytes received              : 0 B
    - bytes transmitted                 : 0 B/s
    - total bytes transmitted           : 0 B
    - packets received                  : 0 B/s
    - total packets received            : 0 B
    - packets transmitted               : 0 B/s
    - total packets transmitted         : 0 B
    - errors on received                : 0 B/s
    - total errors on received          : 0 B
    - errors on transmitted             : 0 B/s
    - total errors on transmitted       : 0 B
```

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Core node catalog](../../../README.md)
