# Core Nodes

This crate is currently pre-1.0. Its public API may evolve between minor
releases.

Core nodes are the built-in OTAP Dataflow Engine receivers, processors, and
exporters. Use this catalog to find the node `type` to put in runtime YAML and
to open the node-specific documentation for configuration examples, telemetry,
limits, and stability notes.

For help writing runtime YAML, start at
[`docs/configuration.md`](../../docs/configuration.md). For exact runtime
configuration semantics, see
[`docs/configuration-model.md`](../../docs/configuration-model.md).

## How To Read Node Documentation

Each node page follows the same general shape:

- `Metadata`: node type, full URN, feature gate, and stability.
- `Overview`: what the node does and where it fits in a pipeline.
- `Configuration`: the node-level `config` payload and related node options.
- `Examples`: small YAML snippets for common use cases.
- `Telemetry`: node-specific metric sets and events.
- `Limits`: important operational or compatibility limits.
- `Related Docs`: adjacent references and examples.

The default engine build enables the `core-nodes` umbrella feature and includes
every node in this catalog. Minimal builds can disable default features and
enable only the features listed below. A node documented as `experimental` has
no stable compatibility guarantee yet, and its behavior or configuration can
change between releases.

## Node Type Syntax

Use the `Type` value from the tables below in a node definition:

```yaml
type: receiver:otlp
```

The full URN form is also accepted:

```yaml
type: urn:otel:receiver:otlp
```

For the canonical node URN format, see [`docs/urns.md`](../../docs/urns.md).

## Receivers

Receivers ingest data into a pipeline.

<!-- markdownlint-disable MD013 -->

| Type | Feature | Stability | Description |
| --- | --- | --- | --- |
| [`receiver:host_metrics`](src/receivers/host_metrics_receiver/README.md) | `host-metrics` | experimental | Emits Linux `system.*` host metrics from procfs and sysfs. |
| [`receiver:internal_telemetry`](src/receivers/internal_telemetry_receiver/README.md) | Always enabled | experimental | Consumes internal engine log events for observability pipelines. |
| [`receiver:journald`](src/receivers/journald_receiver/README.md) | `journald` | experimental | Reads local `systemd-journald` records with journald source filters. |
| [`receiver:otap`](src/receivers/otap_receiver/README.md) | `otap` | experimental | Accepts OTAP Arrow streams over gRPC. |
| [`receiver:otlp`](src/receivers/otlp_receiver/README.md) | `otlp` | experimental | Accepts OTLP/gRPC, OTLP/HTTP, or both. |
| [`receiver:syslog_cef`](src/receivers/syslog_cef_receiver/README.md) | `syslog-cef` | experimental | Ingests syslog RFC 3164, syslog RFC 5424, and CEF messages. |
| [`receiver:topic`](src/receivers/topic_receiver/README.md) | `topic` | experimental | Subscribes to a named in-process topic. |

<!-- markdownlint-enable MD013 -->

## Processors

Processors transform, route, buffer, or otherwise handle data already moving
through a pipeline.

<!-- markdownlint-disable MD013 -->

| Type | Feature | Stability | Description |
| --- | --- | --- | --- |
| [`processor:attribute`](src/processors/attributes_processor/README.md) | Always enabled | experimental | Mutates OpenTelemetry attributes in OTAP batches. |
| [`processor:batch`](src/processors/batch_processor/README.md) | Always enabled | experimental | Combines OTAP and OTLP payloads before forwarding. |
| [`processor:content_router`](src/processors/content_router/README.md) | Always enabled | experimental | Routes telemetry to named output ports based on content. |
| [`processor:debug`](src/processors/debug_processor/README.md) | Always enabled | experimental | Observes passing data and emits diagnostic output. |
| [`processor:durable_buffer`](src/processors/durable_buffer_processor/README.md) | `durable-buffer` | experimental | Adds crash-resilient buffering through a local durable queue. |
| [`processor:fanout`](src/processors/fanout_processor/README.md) | Always enabled | experimental | Clones incoming data to multiple downstream destinations. |
| [`processor:filter`](src/processors/filter_processor/README.md) | Always enabled | experimental | Drops logs or traces according to include and exclude rules. |
| [`processor:log_sampling`](src/processors/log_sampling_processor/README.md) | Always enabled | experimental | Reduces log volume by discarding selected log records. |
| [`processor:partition`](src/processors/partition_processor/) | `partition` | experimental | Splits by expression and adds a partition transport header. |
| [`processor:retry`](src/processors/retry_processor/README.md) | Always enabled | experimental | Retries downstream delivery when it receives a NACK. |
| [`processor:type_router`](src/processors/signal_type_router/README.md) | Always enabled | experimental | Routes OTAP payloads to output ports by signal type. |
| [`processor:temporal_reaggregation`](src/processors/temporal_reaggregation_processor/README.md) | Always enabled | experimental | Reaggregates high-frequency metrics into lower-frequency output. |
| [`processor:transform`](src/processors/transform_processor/README.md) | `transform` | experimental | Applies query-language transformations to OTAP batches. |

<!-- markdownlint-enable MD013 -->

For a behavioral processor taxonomy, see
[`docs/processors.md`](../../docs/processors.md).

## Exporters

Exporters send data out of a pipeline.

| Type | Feature | Stability | Description |
| --- | --- | --- | --- |
| [`exporter:console`](src/exporters/console_exporter/README.md) | Always enabled | experimental | Prints logs and pretty metrics; record JSON is logs-only. |
| [`exporter:file`](src/exporters/file_exporter/README.md) | `file` | experimental | Writes signal-exclusive OTLP JSON Lines files. |
| [`exporter:noop`](src/exporters/noop_exporter/README.md) | Always enabled | experimental | Acknowledges and discards every received message. |
| [`exporter:otap`](src/exporters/otap_exporter/README.md) | `otap` | experimental | Sends OTAP Arrow payloads over gRPC streams. |
| [`exporter:otlp_grpc`](src/exporters/otlp_grpc_exporter/README.md) | `otlp` | experimental | Sends telemetry as unary OTLP/gRPC export requests. |
| [`exporter:otlp_http`](src/exporters/otlp_http_exporter/README.md) | `otlp` | experimental | Sends telemetry to OTLP/HTTP endpoints. |
| [`exporter:parquet`](src/exporters/parquet_exporter/README.md) | `parquet` | experimental | Writes OTAP batches as Parquet files. |
| [`exporter:topic`](src/exporters/topic_exporter/README.md) | `topic` | experimental | Publishes data to a named in-process topic. |

## Feature Aggregates

- `core-nodes`: enables all core receivers, processors, and exporters.
- `core-receivers`: enables all core receivers.
- `core-processors`: enables processors with optional dependency subtrees.
- `core-exporters`: enables all core exporters.
- `otap`: enables the OTAP receiver and exporter.
- `otlp`: enables the OTLP receiver plus the OTLP gRPC and HTTP exporters.
- `topic`: enables the in-process topic receiver and exporter.

Features describe what the build includes rather than implementation
directions. Each uses an unsuffixed name so future source or destination nodes
can join it without changing the public feature contract.

## Maintenance Notes

- Add or reuse an unsuffixed feature when excluding a node removes a meaningful
  dependency or external integration surface.
- Keep lightweight built-in routing, batching, filtering, debugging, and
  observability nodes always enabled.
- Keep `receiver:internal_telemetry` unconditional because every valid engine
  configuration requires it for the engine observability pipeline.
- When an integration has both source and destination nodes, gate all of them
  with the same feature and list it in both relevant categories.
- The `core-nodes` compatibility feature includes all three category umbrellas.
- Gate the module declaration in the category's `mod.rs` with
  `#[cfg(feature = "<feature>")]`. This gates both compilation and the
  `linkme` factory registration inside the module.
- Make large or isolated node-specific dependencies optional and activate them
  from the node feature with `dep:<dependency>`. Shared runtime dependencies can
  remain unconditional when splitting them would make the feature matrix harder
  to maintain than the dependency reduction justifies.
- Forward every node and aggregate feature from the top-level `df_engine`
  package.
- Do not expose direction- or transport-specific features when the nodes share
  the same integration dependency surface.
- Add the component inventory annotation and baseline entry described in the
  [Component Inventory Guide](../../docs/component-inventory.md).
- Any crate that depends on `otel-arrow-dfe-core-nodes` must make its selection
  intentional. Use `default-features = false` with exact features for a minimal
  consumer. Inventory and compatibility tests that need every core node should
  explicitly enable `core-nodes`.
- Keep node READMEs beside the node source files.
- Update this catalog when adding, removing, renaming, or feature-gating a
  node.
- Keep per-node README headings predictable: `Metadata`, `Overview`,
  `Configuration`, `Examples`, `Telemetry`, `Limits`, and `Related Docs`.
- Document node stability as `Experimental` when there is no explicit
  compatibility guarantee.
- Render `Metadata` as a list with one field per list item instead of a table.
- Omit implementation file names from metadata because the README already sits
  next to the source.
- Keep `Configuration` focused on the node's `config` shape and node-local
  options such as `outputs`.
- Prefer the smallest useful node-level YAML snippet. Include `groups`,
  `pipelines`, `topics`, or full engine structure only when the surrounding
  structure is required to explain the node behavior.
- Put `Telemetry` before `Limits`.
- In `Telemetry`, document each metric set with a `Metric`, `Unit`, and
  `Description` table, and document events with an `Event`, `Severity`, and
  `Description` table.
- State explicitly when a node has no node-specific metric set or no
  node-specific events.
- Keep examples in the native `otel_dataflow/v1` runtime format.
- Prefer linking to shared policy docs instead of duplicating long
  descriptions.
