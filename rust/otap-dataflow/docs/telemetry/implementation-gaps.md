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

| Area                       | Gap                                                                                                                                  | Impact                                                                  |
|----------------------------|--------------------------------------------------------------------------------------------------------------------------------------|-------------------------------------------------------------------------|
| Metric temporality         | [Consumer-selected downstream conversion is not implemented](https://github.com/open-telemetry/otel-arrow/issues/3543)               | Consumers receive the canonical low-memory temporality                  |
| Multivariate metrics       | OTLP and OTAP lack first-class multivariate metric sets                                                                              | Limits protocol efficiency; some semantics may be lossy                 |
| Tracing                    | Traces not implemented (draft only)                                                                                                  | Limits end-to-end causality and latency debugging                       |
| Distribution configuration | Exponential histogram table size is hard-coded, not compile-time or runtime configurable, limits on scale and width not configurable | User has no control over histogram resolution, performance, space usage |

### Resource identity and entity attributes

| Area             | Gap                                                               |
|------------------|-------------------------------------------------------------------|
| Service identity | `service.name` not set everywhere                                 |
| Service identity | `process.instance.id` used instead of `service.instance.id`       |
| Execution engine | `thread.id` not set                                               |
| Execution engine | `core.id` used instead of `cpu.logical_number`                    |

### Tooling and process

| Area           | Gap                                                                                                                                                                                                                                                                                              | Impact                                                        |
|----------------|--------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|---------------------------------------------------------------|
| Validation     | Weaver codegen drift check (`cargo xtask telemetry-codegen --check`) and live-check run in CI for events and core metrics; component-specific metrics are not yet migrated                                                                                                                     | Component-specific metrics are not yet schema-validated in CI |
| Stability      | Stability level not declared for all signals                                                                                                                                                                                                                                                     | Hard to apply compatibility discipline                        |
| Deprecation    | Migration windows and dual emission not implemented                                                                                                                                                                                                                                              | Breaking changes may slip into stable telemetry               |
| SDK generation | Core metric sets (`controller`, `engine`, and shared `otap` boundaries) are generated from `semconv/` via Weaver templates; component-specific metric sets and attribute/event types remain hand-written                                                                                          | Component telemetry still defines structs manually            |
| Registry files | Core metrics and internal events are defined in `semconv/groups/`; component-specific metrics remain to be migrated from hand-written macros into `semconv/`                                                                                                                                      | Component metrics lack generated reference docs until migrated |

### Open questions

| Topic                      | Question                                                              |
|----------------------------|-----------------------------------------------------------------------|
| Metrics endpoint           | What is the default deployment posture (off by default vs protected)? |
| Schema endpoint            | What is the default deployment posture (off by default vs protected)? |
