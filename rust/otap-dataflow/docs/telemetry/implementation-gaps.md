# Implementation gaps

Status: Draft

This document consolidates known gaps between the telemetry documentation and
what is currently implemented or enforced.

Goal:

- Keep all "not yet implemented" notes and process gaps in one place
- Avoid sprinkling implementation status across multiple guides

## How to use this document

- Treat the other guides as the intended design and policy.
- Treat this document as the current status tracker.
- When a gap is closed, remove it here and update any affected guide text if
  needed.

## Gaps and open work

### Signals and data model

| Area                 | Gap                                                     | Impact                                                  | 
|----------------------|---------------------------------------------------------|---------------------------------------------------------|
| Metrics              | Histograms not supported yet                            | Limits latency and size distributions                   |
| Metrics              | Bounded signal-specific metric attributes not supported | Limits modeling of small enum dimensions on core metrics |
| Multivariate metrics | OTLP and OTAP lack first-class multivariate metric sets | Limits protocol efficiency; some semantics may be lossy |
| Tracing              | Traces not implemented (draft only)                     | Limits end-to-end causality and latency debugging       |

### Resource identity and entity attributes

| Area             | Gap                                                               | 
|------------------|-------------------------------------------------------------------|
| Service identity | `service.name` not set everywhere                                 |
| Service identity | `process.instance.id` used instead of `service.instance.id`       |
| Execution engine | `thread.id` not set                                               |
| Execution engine | `core.id` used instead of `cpu.logical_number`                    |
| Execution engine | `numa.node.id` used instead of `otelcol.numa_node.logical_number` |
| Channels         | `channel.sender.out.port` not set                                 |
| Channels         | Channel id format not enforced                                    |

### Tooling and process

| Area           | Gap                                                    | Impact                                          |
|----------------|--------------------------------------------------------|-------------------------------------------------|
| Validation     | Registry compliance checks and live checks not covered | Drift between schema and emitted telemetry      |
| Stability      | Stability level not declared for all signals           | Hard to apply compatibility discipline          |
| Deprecation    | Migration windows and dual emission not implemented    | Breaking changes may slip into stable telemetry |
| SDK generation | Automated client SDK generation not implemented yet    | Manual duplication between schema and code      |

### Open questions

| Topic                      | Question                                                              |
|----------------------------|-----------------------------------------------------------------------|
| Bounded dynamic attributes | How do we implement them?                                             |
| Metrics endpoint           | What is the default deployment posture (off by default vs protected)? |
| Schema endpoint            | What is the default deployment posture (off by default vs protected)? |
