# Changelog

All notable changes to the Rust components of this project (under
`rust/otap-dataflow/`) are documented in this file.

Entries are generated from per-PR YAML fragments in
`rust/otap-dataflow/.chloggen/` using
[chloggen](https://github.com/open-telemetry/opentelemetry-go-build-tools/tree/main/chloggen).

This project is pre-1.0; minor-version releases may include breaking
changes. See [`RELEASING.md`](../../RELEASING.md) for the versioning policy.

<!-- next version -->

## v0.48.0

### 🚀 New components 🚀

- `pipeline`: Add the journald receiver configuration and factory skeleton. ([#2858](https://github.com/open-telemetry/otel-arrow/issues/2858))
  Adds Linux-only receiver registration, validated source/checkpoint/batch configuration,
  lifecycle metrics, and a process-local source lease. Worker ingestion and checkpoint
  persistence will land in follow-up PRs.
  

### 💡 Enhancements 💡

- `all`: Adopt the chloggen tool for managing changelog entries. ([#1423](https://github.com/open-telemetry/otel-arrow/issues/1423))
- `dependencies`: Upgrade various Rust dependencies. ([#2548](https://github.com/open-telemetry/otel-arrow/issues/2548), [#2637](https://github.com/open-telemetry/otel-arrow/issues/2637), [#2639](https://github.com/open-telemetry/otel-arrow/issues/2639), [#2640](https://github.com/open-telemetry/otel-arrow/issues/2640), [#2707](https://github.com/open-telemetry/otel-arrow/issues/2707), [#2760](https://github.com/open-telemetry/otel-arrow/issues/2760), [#2762](https://github.com/open-telemetry/otel-arrow/issues/2762), [#2800](https://github.com/open-telemetry/otel-arrow/issues/2800), [#2831](https://github.com/open-telemetry/otel-arrow/issues/2831), [#2915](https://github.com/open-telemetry/otel-arrow/issues/2915), [#2921](https://github.com/open-telemetry/otel-arrow/issues/2921), [#2965](https://github.com/open-telemetry/otel-arrow/issues/2965), [#2968](https://github.com/open-telemetry/otel-arrow/issues/2968), [#2976](https://github.com/open-telemetry/otel-arrow/issues/2976), [#2979](https://github.com/open-telemetry/otel-arrow/issues/2979), [#2998](https://github.com/open-telemetry/otel-arrow/issues/2998), [#3081](https://github.com/open-telemetry/otel-arrow/issues/3081), [#3091](https://github.com/open-telemetry/otel-arrow/issues/3091), [#3092](https://github.com/open-telemetry/otel-arrow/issues/3092), [#3113](https://github.com/open-telemetry/otel-arrow/issues/3113), [#3114](https://github.com/open-telemetry/otel-arrow/issues/3114), [#3130](https://github.com/open-telemetry/otel-arrow/issues/3130), [#3133](https://github.com/open-telemetry/otel-arrow/issues/3133), [#3147](https://github.com/open-telemetry/otel-arrow/issues/3147), [#3148](https://github.com/open-telemetry/otel-arrow/issues/3148), [#3150](https://github.com/open-telemetry/otel-arrow/issues/3150))
- `pipeline`: Add optional `user_agent` config field to the Azure Monitor exporter for injecting a custom User-Agent HTTP header on all outgoing requests including heartbeat. ([#3137](https://github.com/open-telemetry/otel-arrow/issues/3137))
- `pipeline`: Add opt-in Linux load average metrics to the host metrics receiver. ([#3067](https://github.com/open-telemetry/otel-arrow/issues/3067))
- `pipeline`: Improve OTLP performance by avoiding converting to OTAP before counting items. ([#2993](https://github.com/open-telemetry/otel-arrow/issues/2993))
  The perf exporter previously converted every incoming payload into an
  OtapArrowRecords batch solely to call num_items(). For OTLP-bytes payloads
  this Arrow encoding dominated the exporter's CPU (~50%) and could bottleneck
  single-core throughput. It now uses OtapPayload::num_items().
  

<!-- previous-version -->
