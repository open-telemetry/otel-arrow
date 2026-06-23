# Changelog

All notable changes to the Rust components of this project (under
`rust/otap-dataflow/`) are documented in this file.

Entries are generated from per-PR YAML fragments in
`rust/otap-dataflow/.chloggen/` using
[chloggen](https://github.com/open-telemetry/opentelemetry-go-build-tools/tree/main/chloggen).

This project is pre-1.0; minor-version releases may include breaking
changes. See [`RELEASING.md`](../../RELEASING.md) for the versioning policy.

<!-- next version -->

## v0.49.0

### :stop_sign: Breaking changes :stop_sign:

- `engine`: Pass `&ExtensionContext` to `ExtensionFactory.create` so extensions can register custom metric sets and extension entities at construction time. ([#3285](https://github.com/open-telemetry/otel-arrow/issues/3285))
  After PipelineContext was decoupled from extensions, factories had no way to
  build a `MetricSet<T>` (no public constructor) or otherwise register telemetry
  before the extension started. The new first argument re-opens that path with a
  narrower, extension-scoped context. All in-tree factories and tests have been
  updated.

- `engine`: Add scope-agnostic extension lifecycle monitor with per-variant telemetry. ([#3143](https://github.com/open-telemetry/otel-arrow/issues/3143))
  Decouples the extension subsystem from PipelineContext via a new
  ExtensionContext carrying an opaque (scope.kind, scope.id) attribute pair,
  so extensions can be hosted at pipeline scope today and engine/group scope
  in the future without changing the attribute shape. Local and shared
  extension variants are each registered as distinct telemetry entities and
  driven by a dedicated ExtensionMetricsMonitor that owns its own collection
  interval and shutdown fan-out per scope.

  Breaking changes to the public API of `otap-df-engine`:
  - `ExtensionFactory::create` no longer receives `PipelineContext`. Its
    signature is now `(ExtensionId, Arc<ExtensionUserConfig>, &ExtensionConfig)
    -> Result<ExtensionBundle, Error>`. Out-of-tree extensions need to be
    updated to drop the `PipelineContext` parameter.
  - `PipelineContext::register_channel_entity` and `channel_attribute_set`
    are renamed to `register_node_channel_entity` and
    `node_channel_attribute_set`. The `ChannelAttributeSet` struct is split
    into `NodeChannelAttributeSet` and `ExtensionChannelAttributeSet`.

- `observability`: Self-telemetry metric attributes are now split across the correct OpenTelemetry layers instead of all being emitted as data-point attributes ([#3161](https://github.com/open-telemetry/otel-arrow/issues/3161))
  Metric entity identity moves to the instrumentation scope (`otel_scope_*`
  labels); logs already emitted entity identity on scope. Process/host identity
  (`host.id`, `container.id`, `service.instance.id`) is now applied
  consistently to the Resource layer for both metrics and logs (SDK `Resource`
  / Prometheus `target_info`). Resource attributes are no longer per-metric
  labels (query `target_info`), and metric entity attributes are now
  `otel_scope_`-prefixed on the admin Prometheus output.

- `query-engine`: changes keyword "exclude" to "remove", and changes "date_time" literal tag to "timestamp" to align with nascent spec ([#3051](https://github.com/open-telemetry/otel-arrow/issues/3051))
- `query-engine`: OPL `rename` syntax changed from `rename attributes["new"] = attributes["old"]` to `rename attributes "old" as "new"`. The `project-rename` alias has been removed from OPL. ([#3224](https://github.com/open-telemetry/otel-arrow/issues/3224))

### :bulb: Enhancements :bulb:

- `dependencies`: Upgrade various Rust dependencies. ([#3216](https://github.com/open-telemetry/otel-arrow/issues/3216), [#3217](https://github.com/open-telemetry/otel-arrow/issues/3217), [#3293](https://github.com/open-telemetry/otel-arrow/issues/3293), [#3301](https://github.com/open-telemetry/otel-arrow/issues/3301), [#3308](https://github.com/open-telemetry/otel-arrow/issues/3308), [#3326](https://github.com/open-telemetry/otel-arrow/issues/3326))
- `engine`: Add an optional `purpose` field to flow_metric config, emitted as the `flow.purpose` scope attribute so OTel View selectors can target distinct flavors of flow work. ([#2859](https://github.com/open-telemetry/otel-arrow/issues/2859))
  The `flow.purpose` attribute is always emitted on flow metrics; when `purpose` is omitted it carries an empty value, so existing single-`flow`-scope selectors keep matching.
- `engine`: Add controller extension hooks, static validation, fail-fast runtime error handling, and an opt-in read-only controller monitor extension. ([#3263](https://github.com/open-telemetry/otel-arrow/issues/3263))
- `engine`: Add full-engine live reconciliation primitives plus Admin HTTP endpoints for config snapshots, config reconciliation, pipeline group creation, and pipeline/group deletion. ([#3266](https://github.com/open-telemetry/otel-arrow/issues/3266))
- `otap`: Propagate the receiver-observed peer socket address on `OtapPdata` so processors can read request-scoped transport facts via `OtapPdata::peer_addr()`. ([#3220](https://github.com/open-telemetry/otel-arrow/issues/3220))
  Adds an optional `Option<SocketAddr>` peer-address slot to the `Context`
  carried by every `OtapPdata`, populated by the OTLP gRPC, OTLP HTTP, OTAP
  gRPC, and syslog/CEF TCP receivers from the accepting socket. Receivers
  without a real socket (file-based, journald, UDS) leave it `None`. The
  value is preserved across transport boundaries by `clone_without_context`
  and is exposed via `OtapPdata::peer_addr()` so processors that care
  (notably the k8sattributes processor's `pod_association: connection` and
  `passthrough` modes) can opt in to reading it.

- `pipeline`: Add optional `headers` config field to `HttpClientSettings` and `GrpcClientSettings` for sending static request headers (e.g. `Authorization`, tenant routing) on every outbound OTLP/HTTP and OTLP/gRPC request ([#3303](https://github.com/open-telemetry/otel-arrow/issues/3303))
  Headers are validated at config load: the OTLP/HTTP exporter rejects
  protocol-reserved names (Content-Type/Content-Encoding/Content-Length/Host)
  and response-negotiation names (Accept/Accept-Encoding), and the OTLP/gRPC
  exporter rejects transport-reserved metadata (Content-Type/TE/User-Agent and
  the `grpc-` prefix); protocol headers always take precedence. The gRPC static
  metadata is built
  once at startup so the per-message hot path keeps its zero-allocation fast
  path. The OTAP (Arrow) exporter shares `GrpcClientSettings` but does not apply
  static headers to its streams yet, so it rejects a non-empty `headers` map at
  config load rather than silently dropping it. This is the static-header
  complement to header propagation (#2563).

- `pipeline`: Add TDH (Trace Data Helper) decoder for ETW tracelogging events in the ETW receiver ([#2783](https://github.com/open-telemetry/otel-arrow/issues/2783))
- `pipeline`: Add static gRPC endpoint syntax validation and optional startup_check (none/dns/connect) to OTLP gRPC and OTAP exporters ([#3211](https://github.com/open-telemetry/otel-arrow/issues/3211))
- `pipeline`: Add opt-in Linux per-process metrics to the host metrics receiver. ([#3066](https://github.com/open-telemetry/otel-arrow/issues/3066))
  Per-process metric series include `process.pid` by default and require it when enabled to avoid indistinguishable same-command process series.

- `pipeline`: Implement journald receiver ingestion, encoding, and cursor checkpointing. ([#2858](https://github.com/open-telemetry/otel-arrow/issues/2858))
  Adds the Linux sd-journal worker, OTAP log projection, downstream Ack/Nack
  handling, durable cursor commits, and source metrics for the journald receiver.

- `pipeline`: Improve engine configuration and node documentation for users discovering dataflow pipelines. ([#3212](https://github.com/open-telemetry/otel-arrow/issues/3212))
- `pipeline`: Add grouped session and per-subscription pending limits to the Linux user_events receiver. ([#3071](https://github.com/open-telemetry/otel-arrow/issues/3071))
- `pipeline`: Add optional `user_agent` config field to `HttpClientSettings` and `GrpcClientSettings` for custom User-Agent headers ([#3138](https://github.com/open-telemetry/otel-arrow/issues/3138))
- `pipeline`: Add `exporter.azure_monitor.in_flight_log_records` gauge to the Azure Monitor exporter for retry-backpressure visibility. ([#3304](https://github.com/open-telemetry/otel-arrow/issues/3304))
  The gauge reports the number of log records currently in-flight at the
  exporter (enqueued export requests awaiting completion, including records
  being retried).

- `pipeline`: Add end-to-end integration test for the ETW receiver on Windows ([#2783](https://github.com/open-telemetry/otel-arrow/issues/2783))
  Drives the ETW receiver from a `TestRuntime`, emits TraceLogging events via
  both the dynamic (`tracelogging_dynamic`) and static (`tracelogging`)
  producer crates, and asserts the decoded events arrive in an OTAP Arrow
  batch with the expected scalar, string, and nested-struct field values.
  Gated on `--ignored` and Administrator privileges; runs in CI on
  `windows-latest`.

- `pipeline`: Preserve `OtapPdata::peer_addr` across merging processors (`batch_processor`, `temporal_reaggregation_processor`) by merging contributing inputs' peer addresses via the new `Context::merge_peer_addr` helper. ([#3228](https://github.com/open-telemetry/otel-arrow/issues/3228))
  Adds `Context::merge_peer_addr`, which returns `Some(addr)` only when every
  contributing input observed the same peer address and `None` otherwise.
  `batch_processor` and `temporal_reaggregation_processor` - the two processors
  that rebuild `OtapPdata` with a fresh `Context` when merging multiple inputs
  - now apply this helper so single-peer streams retain the receiver-observed
  peer address end-to-end, while multi-peer merges keep `peer_addr` as `None`
  to avoid misattribution.

- `pipeline`: Align the ETW receiver with the OpenTelemetry logs data model appendix ETW mapping - preserve the raw ETW level in the `etw.level` attribute so the SeverityNumber mapping stays reversible, and emit `SeverityText` using the standardized ETW level names (`LOG_ALWAYS`, `CRITICAL`, `ERROR`, `WARNING`, `INFO`, `VERBOSE`) ([#2783](https://github.com/open-telemetry/otel-arrow/issues/2783))
- `pipeline`: Expose object-store retry settings on the Parquet exporter and add flush outcome metrics. ([#3052](https://github.com/open-telemetry/otel-arrow/issues/3052))
- `pipeline`: OTAP exporter now supports static request `headers` natively: configured headers are attached once as the initial metadata of each Arrow log/metric/trace stream. The previous config-load rejection is removed. ([#3314](https://github.com/open-telemetry/otel-arrow/issues/3314))
  Headers are sent once per stream establishment, not per `BatchArrowRecords`, so the per-message hot path remains allocation-free. Header validation (ASCII-only, no gRPC-reserved metadata) is shared with the OTLP/gRPC exporter via `GrpcClientSettings`.
- `query-engine`: Add user guide for OPL ([#3183](https://github.com/open-telemetry/otel-arrow/issues/3183))
- `query-engine`: Add `fork` operator call to OPL language and support in query-engine ([#3188](https://github.com/open-telemetry/otel-arrow/issues/3188))
- `query-engine`: Adds small application that serves an interactive playground for crafting OPL programs ([#3214](https://github.com/open-telemetry/otel-arrow/issues/3214))
- `query-engine`: Adds `drop` operator to OPL for unconditionally discarding telemetry data ([#3253](https://github.com/open-telemetry/otel-arrow/issues/3253))
- `query-engine`: Adds support for comments in OPL programs ([#3151](https://github.com/open-telemetry/otel-arrow/issues/3151))
- `query-engine`: Add `Bytes` type keyword support for OPL type-checking expressions (e.g. `attributes["x"] is Bytes`). ([#3223](https://github.com/open-telemetry/otel-arrow/issues/3223))

### :toolbox: Bug fixes :toolbox:

- `otap`: Fix gRPC startup DNS checks to skip local DNS only when the target endpoint actually uses a configured proxy. ([#3222](https://github.com/open-telemetry/otel-arrow/issues/3222))
- `otap`: Fix build failure (E0433) when a dependent crate is compiled without a crypto-* feature, by gating the rustls ring fallback in ensure_crypto_provider() behind the test-utils feature. ([#3280](https://github.com/open-telemetry/otel-arrow/issues/3280))
  rustls is pinned with default-features off, so rustls::crypto::ring only exists
  when the ring feature is enabled (via a crypto-* feature or test-utils). The
  unconditional fallback referenced that module even when it was configured out,
  breaking lib-only checks of crates that depend on otap-df-otap without selecting
  a crypto provider.

- `pdata`: Handle Duration and Timestamp cardinality estimation across all Arrow time units in concatenate (previously panicked). ([#3181](https://github.com/open-telemetry/otel-arrow/issues/3181))
- `pipeline`: Add the optional `event_name` field to the Parquet exporter logs schema so it is no longer silently dropped. ([#3270](https://github.com/open-telemetry/otel-arrow/issues/3270))
  The `event_name` column already exists in the canonical OTAP logs schema but was
  missing from the hand-written Parquet exporter logs template. Logs that carry
  `event_name` are now written with the value preserved; logs without it produce a
  NULL column.

- `pipeline`: Transform processor no longer returns a config error when the query has leading whitespace. ([#3209](https://github.com/open-telemetry/otel-arrow/issues/3209))
- `query-engine`: Fix `coalesce` to preserve spans missing the first attribute by using an outer join when aligning attribute batches. ([#3078](https://github.com/open-telemetry/otel-arrow/issues/3078))
- `query-engine`: Resolves handful of correctness issues in the otap query-engine related to logical expression evaluation ([#3003](https://github.com/open-telemetry/otel-arrow/issues/3003))

<!-- previous-version -->

## v0.48.0

### :rocket: New components :rocket:

- `pipeline`: Add the journald receiver configuration and factory skeleton. ([#2858](https://github.com/open-telemetry/otel-arrow/issues/2858))
  Adds Linux-only receiver registration, validated source/checkpoint/batch configuration,
  lifecycle metrics, and a process-local source lease. Worker ingestion and checkpoint
  persistence will land in follow-up PRs.

### :bulb: Enhancements :bulb:

- `all`: Adopt the chloggen tool for managing changelog entries. ([#1423](https://github.com/open-telemetry/otel-arrow/issues/1423))
- `dependencies`: Upgrade various Rust dependencies. ([#2548](https://github.com/open-telemetry/otel-arrow/issues/2548), [#2637](https://github.com/open-telemetry/otel-arrow/issues/2637), [#2639](https://github.com/open-telemetry/otel-arrow/issues/2639), [#2640](https://github.com/open-telemetry/otel-arrow/issues/2640), [#2707](https://github.com/open-telemetry/otel-arrow/issues/2707), [#2760](https://github.com/open-telemetry/otel-arrow/issues/2760), [#2762](https://github.com/open-telemetry/otel-arrow/issues/2762), [#2800](https://github.com/open-telemetry/otel-arrow/issues/2800), [#2831](https://github.com/open-telemetry/otel-arrow/issues/2831), [#2915](https://github.com/open-telemetry/otel-arrow/issues/2915), [#2921](https://github.com/open-telemetry/otel-arrow/issues/2921), [#2965](https://github.com/open-telemetry/otel-arrow/issues/2965), [#2968](https://github.com/open-telemetry/otel-arrow/issues/2968), [#2976](https://github.com/open-telemetry/otel-arrow/issues/2976), [#2979](https://github.com/open-telemetry/otel-arrow/issues/2979), [#2998](https://github.com/open-telemetry/otel-arrow/issues/2998), [#3081](https://github.com/open-telemetry/otel-arrow/issues/3081), [#3091](https://github.com/open-telemetry/otel-arrow/issues/3091), [#3092](https://github.com/open-telemetry/otel-arrow/issues/3092), [#3113](https://github.com/open-telemetry/otel-arrow/issues/3113), [#3114](https://github.com/open-telemetry/otel-arrow/issues/3114), [#3130](https://github.com/open-telemetry/otel-arrow/issues/3130), [#3133](https://github.com/open-telemetry/otel-arrow/issues/3133), [#3147](https://github.com/open-telemetry/otel-arrow/issues/3147), [#3148](https://github.com/open-telemetry/otel-arrow/issues/3148), [#3150](https://github.com/open-telemetry/otel-arrow/issues/3150))
- `pipeline`: Add optional `user_agent` config field to the Azure Monitor exporter for injecting a custom User-Agent HTTP header on all outgoing requests including heartbeat. ([#3137](https://github.com/open-telemetry/otel-arrow/issues/3137))
- `pipeline`: Add metric-name filtering (include/exclude with `strict` or `regexp` match) to the filter processor. ([#2777](https://github.com/open-telemetry/otel-arrow/issues/2777))
- `pipeline`: Hardens flow metric config to throw an error if the user attempts to configure a flow metric with an unreachable end node. ([#3009](https://github.com/open-telemetry/otel-arrow/issues/3009))
- `pipeline`: Add opt-in Linux load average metrics to the host metrics receiver. ([#3067](https://github.com/open-telemetry/otel-arrow/issues/3067))
- `pipeline`: Improve OTLP performance by avoiding converting to OTAP before counting items. ([#2993](https://github.com/open-telemetry/otel-arrow/issues/2993))
  The perf exporter previously converted every incoming payload into an
  OtapArrowRecords batch solely to call num_items(). For OTLP-bytes payloads
  this Arrow encoding dominated the exporter's CPU (~50%) and could bottleneck
  single-core throughput. It now uses OtapPayload::num_items().

<!-- previous-version -->
