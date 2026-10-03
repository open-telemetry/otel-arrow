---
Proposal Name: validation-framework-harness-plugins
Start Date: 2026-08-25
RFC PR: open-telemetry/otel-arrow#0000
Tracking Issue: open-telemetry/otel-arrow#3510
---

# RFC 0007: Validation Framework Harness Plugins

## Summary

`otap-dataflow`'s **validation framework** is a builder-driven, end-to-end test
harness. A test author describes a scenario -- synthetic traffic, a
system-under-validation (SUV) pipeline, and the checks that must pass -- and the
framework renders a real engine configuration, runs it in-process through the
real engine controller, and reports pass/fail. The framework supports two
backend strategies: an in-process capture pipeline that asserts on the SUV
output stream, and Docker containers for real external dependencies.

A user-implementable **plugin** mechanism adds a third strategy so a scenario
can (a) provision a test backend (an in-process mock, a networked server, or a
filesystem double), (b) wire it into the SUV pipeline via config overrides, and
(c) assert **backend-side state** after traffic has flowed (messages on a
broker, rows in parquet, requests at an HTTP endpoint). Containers are retained;
plugins are a broader, Docker-optional alternative that can also make assertions
the container path cannot.

A single `HarnessPlugin` trait with an optional `validate` hook is the point at
which users plug in. Plugins are a **test-time construct owned by the
scenario**, driven on a dedicated per-plugin thread so a `!Send` backend double
(for example, a mock message broker holding a raw pointer into a C library)
never crosses a thread boundary. The design in this document was exercised by a
prototype in which three reference plugins -- a Kafka mock broker, a Parquet
temp-directory, and an Azure Monitor HTTP mock -- ran real end-to-end scenarios
through the engine, gated behind the manual validation test suite.

## Motivation

The framework renders a three-pipeline engine group -- a traffic generator, the
SUV pipeline, and a capture pipeline -- from templates, runs it in-process
through the real engine controller, and polls the admin metrics API for
pass/fail. Docker backends are described by a container configuration, started
and stopped around the run.

The framework validates a pipeline's **stream behavior** well, but falls short
in four ways when the thing under test terminates in an external system:

1. **Container-only backends.** The container path assumes Docker. Several nodes
   have first-class in-process mocks that need no Docker -- a Kafka exporter, for
   example, can be tested against an in-process mock broker. There is no way to
   express "use this in-process mock" as a scenario backend.
2. **No backend-side assertions.** The validation checks compare the
   control stream against the SUV output stream. They cannot assert on the
   backend itself (broker state, rows in a file, requests at an endpoint).
3. **No node-local ownership.** All scenarios and backends live in the harness
   crate. A node author cannot ship a backend + assertions bundle next to their
   node, nor run node-local validation tests.
4. **Not generic.** There is no trait a user can implement to teach the harness
   about a new backend or a new class of backend assertion.

### Which nodes benefit

Not every node needs a plugin. Exporters and receivers with an external
dependency or a useful test double benefit (message brokers, HTTP-ingestion
exporters, columnar-file exporters, durable buffers, host and journald
receivers). Pure or internal processors, and internal receivers and exporters,
do not. The plugin mechanism is therefore **opt-in per scenario**, never
mandatory, and the design must not assume every node wires a plugin.

## Guide-level explanation

### The scenario model

The framework is organized around a **scenario**: a builder that assembles three
logical pipelines and runs them together as one engine group.

- **Generator** -- produces synthetic telemetry at a configured rate and count,
  and sends it into the SUV pipeline over OTLP or OTAP. It also emits a
  duplicate **control stream** used as ground truth for equivalence checks.
- **System under validation (SUV)** -- the pipeline actually being tested. It is
  supplied by the author as ordinary pipeline YAML.
- **Capture** -- receives the SUV output and runs stream-level assertions,
  reporting results through engine admin metrics as `finished` and `valid`
  gauges (the harness's metrics contract).

A minimal scenario looks like this:

```rust
Scenario::new()
    .pipeline(Pipeline::from_yaml(SUV_PIPELINE)?)
    .add_generator("gen", Generator::logs().fixed_count(1_000).otlp_grpc("receiver"))
    .add_capture("out", Capture::equivalence().otlp_grpc("exporter"))
    .expect_within(30)
    .run()?;
```

The framework renders these three pieces into a single engine group, starts the
real engine controller in-process, waits for readiness, drives the generator to
its limit, and then reads the capture's `finished`/`valid` gauges to decide
pass or fail. `expect_within` bounds the whole run.

### Backends: containers and plugins

Some SUV pipelines terminate in an external system rather than in a capture. The
framework supports two ways to stand one up for a test:

- **Containers** -- start a real dependency in Docker, wait for it to be
  healthy, and rewrite the SUV pipeline's endpoint to point at the mapped host
  port. This is the right tool when only the real server will do.
- **Plugins** -- a Docker-optional mechanism (described next) that can provision
  in-process mocks, filesystem doubles, or networked servers, wire them into the
  SUV pipeline, and -- crucially -- assert on backend state afterwards.

### The mental model of a harness plugin

A **harness plugin** is a scenario participant with a three-call lifecycle:

1. `setup` -- provision the backend and return the wiring the harness applies to
   the SUV pipeline (config-key overrides, named endpoints, env vars).
2. `validate` -- assert backend-specific invariants after traffic completes
   (optional; the default is a no-op).
3. `teardown` -- release resources (the default is a no-op for backends cleaned
   up by RAII).

You register one with `Scenario::add_plugin(label, factory)`, mirroring the way
containers are added. The `factory` closure builds the plugin on its own host
thread, so a `!Send` backend is constructed where it will live.

### Example: Kafka exporter against an in-process mock broker

```rust
Scenario::new()
    .pipeline(Pipeline::from_yaml(KAFKA_SUV_PIPELINE)?)
    .add_generator("gen", Generator::logs().fixed_count(200).otlp_grpc("receiver"))
    .add_plugin("kafka", || {
        MockKafkaPlugin::new("exporter", "otlp-logs").min_messages(1)
    })?
    // No capture: the assertion lives in the plugin. Let async Kafka
    // delivery settle after the generator finishes.
    .backend_settle(5)
    .expect_within(60)
    .run()?;
```

`MockKafkaPlugin::new` takes the SUV exporter node id (`"exporter"`) and the
topic (`"otlp-logs"`); `min_messages(1)` is the backend assertion. Its `setup`
starts an in-process mock broker, pre-creates the topic, and returns config
overrides that point the exporter's `brokers`, `client_id`, and `logs.*`
settings at the mock. Its `validate` reads the broker's high and low watermarks
and asserts that at least `min_messages` landed on the topic. There is no Docker
and no external broker: the mock broker is `!Send` and lives entirely on the
plugin's host thread.

### Example: Parquet exporter against a temp directory

```rust
Scenario::new()
    .pipeline(Pipeline::from_yaml(PARQUET_SUV_PIPELINE)?)
    .add_generator("gen", Generator::logs().fixed_count(300).otap_grpc("receiver"))
    .add_plugin("parquet", || TempDirParquetPlugin::new("exporter", "logs").min_rows(1))?
    .backend_settle(6)
    .expect_within(90)
    .run()?;
```

Here there is no server at all. `TempDirParquetPlugin::new` takes the exporter
node id and a signal name; `min_rows(1)` is the assertion. `setup` creates a
temporary directory and rewrites the exporter's **nested** config field
`storage.file.base_uri` to point at it (plus a short flush interval so files
land inside the settle window). `validate` opens the written parquet files off
disk and counts rows, asserting the total is at least `min_rows`. The temp
directory is removed when the plugin is torn down.

### Example: Azure Monitor exporter against an HTTP mock (with a bound credential)

Some exporters need more than a config rewrite. The Azure Monitor exporter
requires a `bearer_token_provider` capability bound to its node. The plugin
still handles the backend (a mock ingestion server plus a `dcr_endpoint`
rewrite), while the **capability binding is expressed in the SUV pipeline YAML**
and satisfied by a registered test-double credential provider:

```yaml
nodes:
  exporter:
    type: "urn:microsoft:exporter:azure_monitor"
    capabilities:
      bearer_token_provider: "auth"
    config:
      api:
        dcr_endpoint: "http://127.0.0.1:1"   # rewritten by the plugin at setup
        stream_name: "Custom-Logs"
        dcr: "dcr-abc123"
extensions:
  auth:
    type: "urn:otel:extension:static_bearer_auth_test"
    config: { token: "validation-static-token" }
```

```rust
Scenario::new()
    .pipeline(Pipeline::from_yaml(&suv_yaml)?)
    .add_generator("gen", Generator::logs().fixed_count(200).otlp_grpc("receiver"))
    .add_plugin("azure", || MockAzureMonitorPlugin::new("exporter").min_requests(1))?
    .backend_settle(6)
    .run()?;
```

`MockAzureMonitorPlugin::new` takes the exporter node id; `min_requests(1)` is
the assertion. `setup` binds a mock ingestion server, rewrites the exporter's
`api.dcr_endpoint` to the bound address, and keeps the server serving for the
whole run. `validate` reads a server-side request counter and asserts the
exporter delivered at least `min_requests` ingestion requests. The credential
the exporter needs is provided by the `static_bearer_auth_test` provider named
in the YAML, not by the plugin.

### How to think about it as a node author

Ship the plugin and its end-to-end scenario test **next to your node**, gated to
the manual validation suite. Because engine node factories are discovered
through a link-time registration mechanism, a validation scenario that uses your
node as the SUV must run inside a test binary that links your crate -- which is
naturally your own crate's test tree. That is where the plugin and its
validation test live.

## Reference-level explanation

### Overview of the moving parts

The framework provides, in one crate:

- the **scenario builder** and the generator, capture, and SUV pipeline
  abstractions;
- the **template renderer** that turns a scenario into a single engine group;
- the **run loop** that starts the engine controller in-process, waits for
  readiness, drives traffic, reads the metrics contract, and shuts down;
- the **container** integration for Docker backends.

The **plugin** mechanism comprises the `HarnessPlugin` trait, the setup/validate
contexts, the `PluginWiring` output type, and the dedicated-thread `PluginHost`
that drives a plugin. The following subsections specify how these pieces slot
into the run loop.

Node crates that ship plugins take the framework crate as a test-time dependency
only. There is no dependency cycle: the framework depends on the engine and node
libraries, none of which depend on the crates that ship plugins.

### The trait

```rust
#[async_trait(?Send)]
pub trait HarnessPlugin {
    async fn setup(&mut self, ctx: &mut PluginSetupContext)
        -> Result<PluginWiring, ValidationError>;

    async fn validate(&mut self, ctx: &PluginValidateContext)
        -> Result<(), ValidationError> { Ok(()) }

    async fn teardown(&mut self) -> Result<(), ValidationError> { Ok(()) }
}
```

One trait, optional `validate`. A single type plays both the backend/middleware
role (`setup`/`teardown`) and the validation-provider role (`validate`) without
marker sub-traits. The trait is `?Send` because a backend double may be `!Send`;
it is always driven on a dedicated current-thread runtime (see below), so no
`Send` bound is required of the plugin or its backend.

`PluginSetupContext` and `PluginValidateContext` are deliberately thin
(currently carrying the scenario label). They exist as structs so future wiring
inputs -- a port-allocation callback, sibling-plugin endpoints, an observed run
summary -- can be added without changing the trait signature.

### Wiring outputs: `PluginWiring`

```rust
PluginWiring::new()
    .with_config_override("exporter", "storage.file.base_uri", &base_uri)
    .with_config_override("exporter", "writer_options.flush_when_older_than", "1s")
    .with_endpoint("output_dir", &base_uri)
```

`PluginWiring` generalizes what a container mapping produces (host/port
mappings, templated env vars) into a backend-agnostic bundle of:

- **config overrides**: dot-path rewrites under a node's `config`, applied
  through the same config-path machinery the container and endpoint wiring
  already use;
- **named endpoints**: informational connection strings;
- **env vars**: process-environment overrides (parity with the container path).

The dot-path walker creates intermediate mappings on the way to the leaf, so a
**nested-enum** config target works with no special support: a `storage` field
that is a serde-tagged enum whose `File` variant serializes as
`storage: { file: { base_uri: ... } }` is rewritten directly by
`storage.file.base_uri`. This was the primary uncertainty going into the
prototype; the existing walker handled it unchanged.

### Threading: the `!Send` problem and the dedicated-thread host

The engine runs on a multi-threaded runtime, and the scenario run builds that
runtime and blocks on the pipeline group. But a mock broker double may be
`!Send`: it can hold a raw pointer into a C library and must live on its
creation thread for the broker's entire lifetime.

The design resolves this with a `PluginHost`: a dedicated OS thread that owns a
current-thread runtime and drives one plugin over channels.

```text
harness thread                     plugin host thread
--------------                     ------------------
add_plugin(factory) -------------> spawn(); build plugin via factory()
                                   (plugin + !Send backend live here)
run():
  setup()   --- Setup cmd ------>  plugin.setup(ctx).await
            <-- PluginWiring (Send) ---
  apply wiring to SUV pipeline
  block_on(engine run) ...         (backend keeps serving on this thread)
  validate() - Validate cmd ---->  plugin.validate(ctx).await
            <-- Result (Send) ----
  teardown() - Teardown cmd ---->  plugin.teardown().await; thread joins
```

The plugin value, its backend, and every future it drives stay on that one
thread. Only `Send` results (wiring, `()`, `ValidationError`) cross the channel.
`setup`, `validate`, and `teardown` are called from the non-async harness thread
(outside the engine block-on), so a blocking channel receive there is safe; the
plugin code never runs on the engine runtime.

A refinement the prototype surfaced: a **networked-server** backend must keep
being polled during the traffic phase, when the host runtime is idle between
commands. A mock HTTP server therefore runs on its own OS thread with an
independent runtime that stays active for the whole run; `setup` binds the port
on that thread and sends the address back. The `PluginHost` model accommodates
this -- a plugin is free to spawn additional threads it owns.

### Scenario run integration

The run loop gains four steps, around the container block:

1. **Setup before launch.** After the scenario's configuration is assembled, for
   each plugin call `setup()` and apply the returned `PluginWiring` to the SUV
   pipeline before templating, so the rewrites appear in the rendered group.
2. **Completion mode.** If the scenario has no capture pipeline, the SUV
   terminates in a plugin backend. A `CompletionMode::LoadgenThenSettle` waits
   for generators to reach their limit, then a fixed settle window (so async
   delivery drains), then shuts down. With a capture, the existing
   validation-exporter contract (`finished`/`valid` gauges) is used unchanged.
   The capture-required check is relaxed only when plugins are present.
3. **Validate after traffic.** On a successful pipeline run, call `validate()`
   for each plugin; the first backend failure is returned as a validation error.
4. **Teardown always.** Tear down every plugin (joining its host thread) on the
   way out, even on a stream-level failure, so backends do not leak.

### Coexistence with containers

Plugins and containers coexist. A plugin that wants a container can drive the
container tooling itself, or (future) delegate to the harness's container
configuration. Both wiring paths feed the same pipeline config-path machinery,
so nothing about the container path changes.

### Reporting backend results

Stream-level assertions still surface pass/fail through engine admin metrics
(the validation-exporter contract). Backend assertions run in the harness run
loop and surface as the `Result` of the run. Both paths report into a single
scenario result. Whether backend results should also be funneled through the
metrics contract for uniformity is left as an open question (below); the
prototype took the direct-`Result` path and it was sufficient for the three
reference backends.

### Reference plugins, mapped to design axes

<!-- markdownlint-disable MD013 -->
| Axis | Kafka | Parquet | Azure Monitor |
| --- | --- | --- | --- |
| Backend double | in-process mock broker (`!Send`, no port) | filesystem temp dir (no server) | mock HTTP server (`Send`, real port) |
| Provisioning | construct broker handle on host thread | temp-dir RAII guard | bind port on a dedicated server thread |
| Config rewrite | flat `brokers` | nested-enum `storage.file.base_uri` | nested `api.dcr_endpoint` |
| Extra wiring | none | force short flush interval | bound `bearer_token_provider` capability |
| Assertion | broker watermark delta | parquet rows read off disk | server-side request counter |
<!-- markdownlint-enable MD013 -->

## Drawbacks

- **A second lifecycle to reason about.** Plugins add setup/validate/teardown
  around the container and metrics flows. Mitigated by making them
  opt-in and by keeping the trait tiny (one required method).
- **Thread-per-plugin cost.** Each plugin gets an OS thread. For a test-time
  harness this is negligible, and it is the price of hosting `!Send` backends
  without infecting the whole harness with a single-threaded model.
- **Node-local tests need a test-time dependency on the harness.** Node crates
  that ship plugins take the harness crate as a test-only dependency. This is a
  one-line addition and does not affect production builds.
- **Backends that need capabilities need pipeline plumbing.** As the Azure case
  shows, a `bearer_token_provider`-style dependency is expressed in the SUV
  pipeline YAML, not by the plugin trait. That is arguably correct (it is a
  pipeline concern) but it means "add a plugin" is not always the whole story.

## Rationale and alternatives

- **Why a single trait with optional `validate` over split marker sub-traits?**
  All three reference plugins play both roles, and a single type covering both
  keeps call sites and the host loop simple. Marker sub-traits add surface area
  with no demonstrated need; they can be introduced later if a real plugin needs
  to be one-role-only and the type system should enforce it.
- **Why a dedicated thread instead of running the harness on a single-threaded
  runtime?** Converting the whole harness to a current-thread model would
  conflict with the engine controller's own threading and force every plugin
  into a single-threaded world. The dedicated-thread host confines the `!Send`
  constraint to exactly the plugins that need it and leaves the engine path
  untouched.
- **Impact of not doing this:** backend-side validation stays impossible,
  in-process mocks stay unusable as scenario backends, and node authors cannot
  own their validation next to their node.

## Prior art

- **A mature engine-level extension system** exists for long-lived capability
  providers hosted inside a running pipeline. This design borrows the
  "user-implemented trait as the extension point" and "kinds/roles" ideas, but
  keeps harness plugins a separate, test-time construct. The two are unrelated
  despite the overlapping vocabulary.
- **In-process mock-broker test harnesses** demonstrate the pattern of hosting a
  `!Send` mock on a dedicated single-threaded runtime; the `PluginHost`
  generalizes it.
- **testcontainers** -- the existing container path, retained as one backend
  strategy among several.
- The [Rust RFC process](https://github.com/rust-lang/rfcs) and
  [OpenDAL RFCs](https://github.com/apache/opendal/tree/main/core/core/src/docs/rfcs)
  for the document shape.

## Unresolved questions

- **Custom stream-level assertions.** Should the stream-level validation
  vocabulary gain a `Custom` variant so stream-level and backend-level
  assertions share one vocabulary, or should plugin-owned assertions stay a
  parallel path? The stream-level assertion set is currently closed and
  serialized into the validation-exporter config, so a `Custom` variant needs a
  serialization story. The prototype did not need it; backend assertions live
  entirely in the plugin.
- **Backend results vs. the metrics contract.** Backend assertions currently
  surface as the `Result` of the scenario run, while stream assertions surface
  via admin metrics. Is one scenario result from two paths acceptable, or should
  backend results also be funneled through the metrics contract for uniformity?
- **Settle window vs. an explicit drain signal.** `LoadgenThenSettle` uses a
  fixed settle duration for no-capture runs. A more precise signal (for example,
  the SUV exporter reporting zero in-flight deliveries via metrics) would remove
  the timing guess. Out of scope for the initial mechanism.
- **Capability-dependent backends.** The Azure case needs a bound
  `bearer_token_provider`. Should the harness offer a blessed, registered set of
  test-double capability providers (static bearer token, etc.) so scenarios do
  not each ship their own, and should `add_plugin` help declare the binding?
- **Exposing node test doubles across crates.** A node's mock harness is often
  crate-private. The prototype kept a plugin in-crate to reuse it, but a
  test-helpers-gated export would let plugins live in a shared crate. Decide
  whether that is worth the surface.

## Future possibilities

- **Reusable test-double capability providers** (static bearer token, static
  vendor bundle) shipped by the harness.
- **A ClickHouse plugin** exercising the "spin up a container *and* assert rows
  landed" dual role against a real backend, closing the loop between the
  container path and backend assertions.
- **Fault-injection plugins** (broker down, HTTP 429/503, disk full) building on
  the reference backends to validate exporter retry and backpressure behavior.