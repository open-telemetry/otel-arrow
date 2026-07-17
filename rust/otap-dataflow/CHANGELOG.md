# Changelog

All notable changes to the Rust components of this project (under
`rust/otap-dataflow/`) are documented in this file.

Entries are generated from per-PR YAML fragments in
`rust/otap-dataflow/.chloggen/` using
[chloggen](https://github.com/open-telemetry/opentelemetry-go-build-tools/tree/main/chloggen).

This project is pre-1.0; minor-version releases may include breaking
changes. See [`RELEASING.md`](../../RELEASING.md) for the versioning policy.

<!-- next version -->

## v0.50.0

### :stop_sign: Breaking changes :stop_sign:

- `engine`: Scope capability exports by capability module so every item has a single public path. Capability data types and registration handles move from the flat `capability::` namespace to `capability::<name>` (e.g. `capability::bearer_token_provider::BearerToken`), and trait variants move to `{local,shared}::capability::<name>`. `BearerToken::expose_secret` is renamed to `expose_token`. ([#3492](https://github.com/open-telemetry/otel-arrow/issues/3492))
  The `#[capability]` macro now emits its `local`/`shared` trait modules as
  `pub(crate)`; the traits are made public only through the hand-written
  `{local,shared}::capability` re-exports, so no capability item is reachable via
  two paths. Shared framework infrastructure (`CapabilityError`,
  `CapabilityErrorSource`, the instance-factory types, `ExtensionCapability`, and
  `KNOWN_CAPABILITIES`) remains flat at `capability::`.

- `pipeline`: Rename ETW receiver attribute keys to use dot notation for consistency (e.g. `etw.event_id` -> `etw.event.id`, `etw.process_id` -> `etw.process.id`, `etw.thread_id` -> `etw.thread.id`). ([#3346](https://github.com/open-telemetry/otel-arrow/issues/3346))
  Downstream queries or processors that reference the previous underscore-style
  keys must be updated to use the new dot-notation keys.

- `pipeline`: The Azure Monitor exporter now authenticates via the `bearer_token_provider` capability instead of an in-config `auth` block. ([#3356](https://github.com/open-telemetry/otel-arrow/issues/3356))
  Authentication is no longer configured on the exporter. It obtains bearer
  tokens from the `azure_identity_auth` extension bound via the
  `bearer_token_provider` capability. This is a breaking config change:
  existing configs that set an `auth:` block under the exporter now fail to
  deserialize (`unknown field `auth``). To migrate, declare an
  `azure_identity_auth` extension in the pipeline's `extensions:` section and
  bind it on the exporter node with
  `capabilities: { bearer_token_provider: <instance-name> }`. The exporter README
  and sample configs are updated accordingly.

### :rocket: New components :rocket:

- `engine`: Initial implementation of OpAMP Agent Controller Extension ([#3387](https://github.com/open-telemetry/otel-arrow/issues/3387))
- `pipeline`: Add the `azure_identity_auth` extension, a `BearerTokenProvider` backed by Azure identity flows. ([#3356](https://github.com/open-telemetry/otel-arrow/issues/3356))
  New `otap-df-contrib-extensions` crate (feature `azure-identity-auth-extension`,
  URN `urn:microsoft:extension:azure_identity_auth`) that acquires and refreshes
  Azure access tokens and exposes them to data-path nodes via the shared
  `BearerTokenProvider` capability. Supports Managed Identity, developer tooling
  (`development`), and Workload Identity Federation auth methods. Tokens are
  cached and refreshed ahead of expiry in a background task, concurrent cache
  misses are coalesced onto a single credential call, and the extension gates
  pipeline startup on the first successful token publish. Emits success/failure/
  publish counters and acquisition-latency telemetry.

- `pipeline`: Add an experimental ClickHouse exporter (`urn:otel:exporter:clickhouse`) ([#3291](https://github.com/open-telemetry/otel-arrow/issues/3291))
  Exports OTAP logs and traces to ClickHouse over HTTP using the official
  `clickhouse` client with `FORMAT ArrowStream`. OTAP Arrow batches are
  reshaped into the otel-collector-contrib ClickHouse schema (attributes as
  `Map(LowCardinality(String), String)`, events/links as `Array(...)`,
  hex-encoded trace/span ids), and the target database and tables are created
  automatically. Opt-in behind the `clickhouse-exporter` feature; the endpoint,
  database, credentials, `async_insert`, and per-table engine/TTL are
  configurable, and `password` supports `${env:VAR}` substitution.

### :bulb: Enhancements :bulb:

- `dependencies`: Upgrade various Rust dependencies. ([#3329](https://github.com/open-telemetry/otel-arrow/issues/3329), [#3341](https://github.com/open-telemetry/otel-arrow/issues/3341), [#3362](https://github.com/open-telemetry/otel-arrow/issues/3362), [#3376](https://github.com/open-telemetry/otel-arrow/issues/3376), [#3384](https://github.com/open-telemetry/otel-arrow/issues/3384), [#3390](https://github.com/open-telemetry/otel-arrow/issues/3390), [#3414](https://github.com/open-telemetry/otel-arrow/issues/3414), [#3415](https://github.com/open-telemetry/otel-arrow/issues/3415), [#3419](https://github.com/open-telemetry/otel-arrow/issues/3419), [#3458](https://github.com/open-telemetry/otel-arrow/issues/3458), [#3459](https://github.com/open-telemetry/otel-arrow/issues/3459), [#3461](https://github.com/open-telemetry/otel-arrow/issues/3461), [#3463](https://github.com/open-telemetry/otel-arrow/issues/3463), [#3474](https://github.com/open-telemetry/otel-arrow/issues/3474), [#3485](https://github.com/open-telemetry/otel-arrow/issues/3485), [#3498](https://github.com/open-telemetry/otel-arrow/issues/3498), [#3508](https://github.com/open-telemetry/otel-arrow/issues/3508))
- `engine`: Add opt-in extension readiness probe to gate pipeline startup on per-extension readiness signals. ([#3143](https://github.com/open-telemetry/otel-arrow/issues/3143))
  Active and background extensions can now opt in to startup gating via
  `active().with_readiness_probe().shared(...).local(...)` (or the
  background twin), which applies the default 5-second readiness
  timeout. The timeout can be overridden (shorter or longer) with
  `with_readiness_probe_timeout_override(t)`, which emits an `INFO`
  event (`extension.readiness.timeout_override`) when set; a zero
  timeout is rejected at build time. The engine threads a
  `ReadinessSignaller` into each variant's `EffectHandler`; the
  extension fires it via `EffectHandler::signal_ready()` once it is
  ready to serve. The controller awaits all opted-in probes (in
  parallel) after the spawn barrier and before tipping nodes into their
  runtime, failing fast with a named laggard on timeout, dropped
  signaller, or an extension that exits during the readiness window.
  Extensions that do not opt in pay zero cost: `signal_ready()` is a
  silent no-op when no probe is registered.

- `engine`: Add an opt-in "Full width" layout toggle to the admin dashboard UI. ([#3373](https://github.com/open-telemetry/otel-arrow/issues/3373))
  The dashboard defaults to the centered, capped column. The toggle lets the
  page shell use the full viewport width and persists the choice per browser
  via localStorage.

- `engine`: Add `signals.dropped` flow metric recorded per drop decision node ([#2859](https://github.com/open-telemetry/otel-arrow/issues/2859))
  Flow metrics declared in the telemetry policy can now enable `signals_dropped`.
  A processor that declares the drop capability and lies within such a flow records
  its dropped count, attributed per decision node via the new `flow.node.decision`
  scope attribute (so a flow may contain multiple decision nodes). Flow-wide kept is
  read from `signals.outgoing`. `filter_processor` and `log_sampling_processor` are
  the first consumers. Single-node flow ranges (`start == end`) are now supported.

- `engine`: `otap-df-ctl` now lets library embedders override the CLI identity used in output. A new `Branding` type (binary name + output-envelope `schemaVersion`) and a `run_with_terminal_and_diagnostics_branded` entrypoint allow a downstream binary that embeds the library to emit help, shell completions, and machine-readable JSON envelopes under its own identity instead of `dfctl`. The standalone `dfctl` binary and the existing public entrypoints are unchanged; they use the default `dfctl` / `dfctl/v1` branding. ([#3370](https://github.com/open-telemetry/otel-arrow/issues/3370))
  Initial scope covers the binary name and envelope `schemaVersion`. Schema catalog identifiers, schema `$id` URLs, `DFCTL_*` environment-variable prefixes, and the interactive TUI command hints remain `dfctl`-branded and may be addressed in follow-ups.
- `engine`: Add the `BearerTokenProvider` capability (with `BearerToken` and `CapabilityError`) so extensions can hand OAuth bearer tokens to data-path nodes. ([#3356](https://github.com/open-telemetry/otel-arrow/issues/3356))
  Defines the engine-side capability surface: the `#[capability]`-generated
  `BearerTokenProvider` trait (`get_token` and `token_stream`), the
  secret-redacting `BearerToken` type, and `CapabilityError` /
  `CapabilityErrorSource` for attributing runtime capability failures to the
  failing `(extension, capability)` pair. No provider implementation is
  included.

- `engine`: Add internal primitives for a broadcast `all` (consensus) Ack/Nack mode for topic hops ([#2252](https://github.com/open-telemetry/otel-arrow/issues/2252))
  Not yet usable: this is internal-only foundation work with no behavior change. It adds the
  `TopicBroadcastAckMode` enum, a consensus-capable publish tracker, and a reserve/commit ring split,
  all gated behind callers that still pass `First`. The `all` mode is not wired into the broadcast
  engine and is not configurable yet — those land in follow-up PRs under #2252.

- `observability`: Add item-level enum-attribute mechanism for metric sets (codegen, export path, and compile-time cardinality check). ([#3430](https://github.com/open-telemetry/otel-arrow/issues/3430), [#3454](https://github.com/open-telemetry/otel-arrow/issues/3454), [#3499](https://github.com/open-telemetry/otel-arrow/issues/3499))
  Adds bounded enum dimensions to metric items, including export support and compile-time
  cardinality and key-conflict checks. Components use
  `#[attribute_set(item, registration|measurement)]` and generated `register(...)`;
  named measurement attribute sets remain supported for compatibility.

- `pdata`: Add retained-memory byte estimates for OTAP Arrow records and payloads. ([#3442](https://github.com/open-telemetry/otel-arrow/issues/3442))
- `pipeline`: Enhance durable_buffer observability with storage utilization and per-signal data loss metrics. ([#3117](https://github.com/open-telemetry/otel-arrow/issues/3117))
- `pipeline`: Add Workload Identity Federation authentication support to the Azure Monitor exporter ([#3336](https://github.com/open-telemetry/otel-arrow/issues/3336))
  Adds a new `workload_identity` (aliases: `wif`, `workload_identity`) auth
  method that exchanges a projected federated ServiceAccount token with Entra
  ID for an access token. New optional `tenant_id` and `token_file_path`
  config fields fall back to the `AZURE_TENANT_ID` and
  `AZURE_FEDERATED_TOKEN_FILE` environment variables, and `client_id` falls
  back to `AZURE_CLIENT_ID`. This enables Azure Monitor export from Kubernetes
  workloads that do not have a managed identity (e.g. self-hosted or
  non-AKS clusters using Workload Identity Federation).

- `pipeline`: Add additional details to auth handler creation error for azure monitor exporter ([#3409](https://github.com/open-telemetry/otel-arrow/issues/3409))
- `pipeline`: Static exporter `headers` values are now wrapped in `secrecy::SecretString` and marked sensitive on the wire, so a header credential (e.g. an `Authorization` token or tenant API key) is redacted in the `Debug`/telemetry output of the typed `HttpClientSettings` / `GrpcClientSettings` and excluded from HTTP/2 HPACK indexing. The cleartext is read only via an explicit `expose_secret()` accessor (during validation and when building the request); behavior is otherwise unchanged. ([#3306](https://github.com/open-telemetry/otel-arrow/issues/3306))
  Scope: this protects the typed settings and the values placed on the wire. It does not redact the raw configuration snapshot returned by the admin config endpoints (e.g. `GET /api/v1/config`), which still serialize the original config value; that broader redaction is tracked in #3328. `user_agent` is intentionally left a plain `String` as a non-credential client identifier.
- `pipeline`: Add ETW receiver telemetry counters for produced events, slow-worker drops, and decode failures, with an explicit counter-algebra contract. ([#3361](https://github.com/open-telemetry/otel-arrow/issues/3361))
  The ETW receiver now records (emitted under the `receiver.etw` metric set;
  the names below are the exact field names you query):
  - `received_events_total`: every event the session produced, counted on the
    producer (`ProcessTrace`) side before the per-core channel send. This is
    the single ingress denominator.
  - `received_events_dropped_slow_worker`: events dropped when an internal
    per-core channel is full (previously only logged), bridged from the
    `!Send` ProcessTrace callback via a shared atomic.
  - `received_events_invalid`: TDH decode failures (the counter existed but was
    never incremented; the failing decode path now feeds it).
  Counter algebra (the exact relationships; the authoritative copy lives in the
  doc comment on `EtwReceiverMetrics` in
  `rust/otap-dataflow/crates/contrib-nodes/src/receivers/etw_receiver/mod.rs`):
  - `received_events_total = received_events_forwarded + received_events_forward_failed
    + received_events_dropped_slow_worker` plus any events still buffered in the
    per-core channel and/or in the in-flight builder at snapshot time.
  - Slow-worker drop rate = `received_events_dropped_slow_worker / received_events_total`.
  - Forward-failure rate = `received_events_forward_failed / received_events_total`.
  - `received_events_invalid` is orthogonal: a decode failure does not drop the
    event (it is still forwarded with empty fields), so it is not subtracted
    from `received_events_total`.

  The producer-side counters are session-scoped (shared across all per-core
  receivers of a `session_name`); each field is claimed via an independent
  `swap(0)`, so concurrent per-core drains never double-count and the registry
  sums the per-core snapshots into one exact session-wide series. A residual
  delta produced after the last core's terminal snapshot is acceptably dropped
  at teardown.
- `pipeline`: The journald receiver no longer double-copies each entry's `MESSAGE` payload. The decoded log body now references the `MESSAGE` value already stored in the entry's field list instead of cloning it into a second buffer, removing one allocation and copy of the message (typically the largest field) per record. Emitted OTLP output is unchanged. ([#3398](https://github.com/open-telemetry/otel-arrow/issues/3398))
- `pipeline`: Record the `signals.dropped` flow metric from the transform processor ([#2859](https://github.com/open-telemetry/otel-arrow/issues/2859))
  The transform processor now declares the drop-decision capability and records the
  number of records removed by the query engine's filter (`where`) stages, so a
  telemetry policy that enables `signals_dropped` can observe dropped counts for a
  transform node.

- `pipeline`: Record the `signals.dropped` flow metric from the recordset_kql processor ([#2859](https://github.com/open-telemetry/otel-arrow/issues/2859))
  The recordset_kql processor now declares the drop-decision capability and records
  the number of records it filters out, so a telemetry policy that enables
  `signals_dropped` can observe dropped counts for the node (e.g. a KQL `where`
  filter). Records folded into a `summarize` aggregation are excluded from the
  dropped count: the recordset engine now reports summarized records separately from
  genuinely filtered ones, so aggregation is no longer miscounted as dropping.

- `query-engine`: Add nested path assignment support for serialized Map and Slice attribute values in OPL transforms ([#3343](https://github.com/open-telemetry/otel-arrow/issues/3343))

### :toolbox: Bug fixes :toolbox:

- `engine`: Credential header values are now redacted from the admin config snapshot APIs (`GET /api/v1/config`, the per-group detail endpoint, and the per-pipeline detail endpoint). Static exporter `headers` values (e.g. an `Authorization` token or tenant API key) were previously returned in cleartext from the raw node `config`, bypassing the `secrecy::SecretString` protection on the typed settings; they are now replaced with `[REDACTED]` in the response while the header keys are preserved so operators can still see which headers are configured. ([#3328](https://github.com/open-telemetry/otel-arrow/issues/3328))
  Redaction is applied to the API response only; the controller's stored configuration retains the original values, so reconciliation and runtime behavior are unchanged.
- `engine`: Use flume's async interface in the shared MPMC channel so `SharedSender::send` and `SharedReceiver::recv` no longer block the Tokio worker thread ([#1704](https://github.com/open-telemetry/otel-arrow/issues/1704))
  The MPMC variant of the shared channel called flume's synchronous `send()`/`recv()`
  from inside `async fn`, which parked the underlying async runtime thread instead of
  yielding cooperatively. These now use `send_async().await` / `recv_async().await`,
  matching the local channel abstraction.

- `engine`: `AttributeValue` serialization now emits plain scalars/sequences, fixing YAML/JSON round-trip failures for telemetry resource attributes. ([#3358](https://github.com/open-telemetry/otel-arrow/issues/3358))
  The derived Serialize on AttributeValue and AttributeValueArray produced
  externally-tagged enum output (e.g. `!String value` in YAML) that the custom
  Deserialize implementation could not parse back. Replaced with custom Serialize
  impls that emit plain values matching the Deserialize expectations.

- `observability`: Suppress noisy per-scrape `MetricValidationFailed` warnings from the Prometheus pull exporter ([#2734](https://github.com/open-telemetry/otel-arrow/issues/2734))
  The `opentelemetry-prometheus` crate flattens OpenTelemetry
  instrumentation scopes into a single namespace keyed by metric name (scope is
  exposed only as the `otel_scope_name` label, per the OTel/Prometheus interop
  spec). When two scopes emit the same metric name with different descriptions
  (e.g. `cpu_utilization`, `pending_sends.buffered`), the exporter keeps the
  first `# HELP` and logs an "Instrument description conflict" warning on every
  scrape. No data is lost — each scope remains a distinct time series — so the
  warning was pure noise. The internal-telemetry log filter now adds a
  field-scoped directive
  (`opentelemetry-prometheus[{metric_description}]=error`) that silences only
  that benign description-conflict warning. The sibling type-conflict warning
  (which carries a `metric_type` field and *does* drop data) and every other
  diagnostic from the crate remain visible at their original level.

- `otap`: `GrpcServerSettings::default()` now binds the loopback interface (`127.0.0.1:0`) instead of all interfaces (`0.0.0.0:0`). Production is unaffected because `listening_addr` is a required config field; the default is only reached when a gRPC server is built programmatically via `..Default::default()`, where the previous all-interfaces default tripped the Windows Defender Firewall prompt during `cargo test` and exposed builder-constructed servers to the local network. ([#3400](https://github.com/open-telemetry/otel-arrow/issues/3400))
- `pdata`: Fix silent attribute loss when decoding OTAP batches whose root IDs are not monotonic in resource/scope visitation order ([#3448](https://github.com/open-telemetry/otel-arrow/issues/3448))
  The OTLP decoder joined child records (attributes, datapoints, span
  events/links) to parents with a shared forward-only cursor that assumed parent
  IDs were visited in ascending order. When root IDs are not monotonic along
  `(resource_id, scope_id, id)` visitation order, a later-visited smaller-ID
  record's child rows were skipped, silently dropping all of that record's
  attributes with no error. `ChildIndexIter::new` now binary-searches
  (`SortedBatchCursor::seek_to_parent`) to each parent's rows, making the join
  order-independent for logs, metrics, and traces.

- `pdata`: OTAP logs/metrics/traces views and OTLP encoders now accept a missing root payload and produce 0 rows / an empty result instead of erroring, matching the updated spec where a missing payload is semantically equivalent to one with 0 rows. ([#3444](https://github.com/open-telemetry/otel-arrow/issues/3444))
- `pipeline`: The journald receiver now follows newly appended entries when `start_at: end` and no checkpoint exists — including on an empty journal, or one whose filters match none of the existing entries. Previously it sought the journal tail but never anchored the read head, so the follow loop never advanced onto new entries: a fresh `start_at: end` receiver emitted no records and never committed a checkpoint. ([#3395](https://github.com/open-telemetry/otel-arrow/issues/3395))
  `sd_journal_seek_tail()` parks the read head after the most recent entry without making any entry current; from there a bare `sd_journal_next()` advances toward a following entry, finds none, and returns `0` (the EOF marker) without anchoring; it does not move backward. (It is `sd_journal_step_one()`, not `sd_journal_next()`, that libsystemd documents as behaving like `sd_journal_previous()` at the tail.) The fresh `StartAt::End` start now issues `sd_journal_seek_tail()` then a single `sd_journal_previous()` to anchor on the last existing entry, so the worker's first `next()` steps forward onto genuinely new entries. When `previous()` finds no entry (empty or fully-filtered journal) the head is repositioned with `sd_journal_seek_head()` to avoid a permanent stall at the tail. This is the accepted best-effort idiom (see systemd/systemd#17662); across rotated or multi-file journals the tail position is approximate, and `start_at: end` has no durable resume anchor until the first checkpoint commit.
- `pipeline`: Fix shutdown deadlock in OTAPExporter when stream queues are full and the downstream receiver is unreachable. ([#3411](https://github.com/open-telemetry/otel-arrow/issues/3411))
  The exporter's enqueue loop now checks for shutdown signals while waiting for
  queue capacity, and gRPC connection attempts are abortable on shutdown.

- `pipeline`: OTLP gRPC exporter now classifies gRPC status codes as retryable or permanent per the OTLP specification. The OTLP gRPC receiver maps permanent NACKs to INTERNAL and transient NACKs to UNAVAILABLE. ([#1920](https://github.com/open-telemetry/otel-arrow/issues/1920))
- `pipeline`: Report durable buffer per-signal data-loss metrics as synchronous Counters, lowers cost to export delta temporality. ([#3117](https://github.com/open-telemetry/otel-arrow/issues/3117))
  The per-signal dropped/expired metrics (`dropped_{log_records,spans,metric_datapoints}`
  and `expired_{log_records,spans,metric_datapoints}`) were `ObserveCounter`s exported as
  cumulative regardless of the configured temporality. Their source
  (`engine.drain_*_pending()`) is delta-native, so they are now plain `Counter`s: a Delta
  exporter reports per-interval loss and a cumulative exporter still reports a monotonic Sum.

<!-- previous-version -->

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
