# Core Nodes

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

Most nodes in this catalog are available in the default engine build. If the
`Feature` column names a non-default feature, the node is available only in
builds that enable that feature. A node documented as `experimental` has no
stable compatibility guarantee yet, and its behavior or configuration can
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

| Type                                                                                 | Feature     | Stability    | Description                                                          |
| ------------------------------------------------------------------------------------ | ----------- | ------------ | -------------------------------------------------------------------- |
| [`receiver:host_metrics`](src/receivers/host_metrics_receiver/README.md)             | default     | experimental | Emits Linux `system.*` host metrics from procfs and sysfs.           |
| [`receiver:internal_telemetry`](src/receivers/internal_telemetry_receiver/README.md) | default     | experimental | Consumes internal engine log events for observability pipelines.     |
| [`receiver:journald`](src/receivers/journald_receiver/README.md)                     | default     | experimental | Reads local `systemd-journald` records with journald source filters. |
| [`receiver:otap`](src/receivers/otap_receiver/README.md)                             | default     | experimental | Accepts OTAP Arrow streams over gRPC.                                |
| [`receiver:otlp`](src/receivers/otlp_receiver/README.md)                             | default     | experimental | Accepts OTLP/gRPC, OTLP/HTTP, or both.                               |
| [`receiver:syslog_cef`](src/receivers/syslog_cef_receiver/README.md)                 | default     | experimental | Ingests syslog RFC 3164, syslog RFC 5424, and CEF messages.          |
| [`receiver:topic`](src/receivers/topic_receiver/README.md)                           | default     | experimental | Subscribes to a named in-process topic.                              |
| [`receiver:traffic_generator`](src/receivers/traffic_generator/README.md)            | `dev-tools` | experimental | Emits synthetic or semantic-convention-derived test traffic.         |

<!-- markdownlint-enable MD013 -->

## Processors

Processors transform, route, buffer, or otherwise handle data already moving
through a pipeline.

<!-- markdownlint-disable MD013 -->

| Type                                                                                            | Feature | Stability    | Description                                                      |
| ----------------------------------------------------------------------------------------------- | ------- | ------------ | ---------------------------------------------------------------- |
| [`processor:attribute`](src/processors/attributes_processor/README.md)                          | default | experimental | Mutates OpenTelemetry attributes in OTAP batches.                |
| [`processor:batch`](src/processors/batch_processor/README.md)                                   | default | experimental | Combines OTAP and OTLP payloads before forwarding.               |
| [`processor:content_router`](src/processors/content_router/README.md)                           | default | experimental | Routes telemetry to named output ports based on content.         |
| [`processor:debug`](src/processors/debug_processor/README.md)                                   | default | experimental | Observes passing data and emits diagnostic output.               |
| [`processor:delay`](src/processors/delay_processor/README.md)                                   | default | experimental | Sleeps for a configured duration before forwarding each message. |
| [`processor:durable_buffer`](src/processors/durable_buffer_processor/README.md)                 | default | experimental | Adds crash-resilient buffering through a local durable queue.    |
| [`processor:fanout`](src/processors/fanout_processor/README.md)                                 | default | experimental | Clones incoming data to multiple downstream destinations.        |
| [`processor:filter`](src/processors/filter_processor/README.md)                                 | default | experimental | Drops logs or traces according to include and exclude rules.     |
| [`processor:log_sampling`](src/processors/log_sampling_processor/README.md)                     | default | experimental | Reduces log volume by discarding selected log records.           |
| [`processor:retry`](src/processors/retry_processor/README.md)                                   | default | experimental | Retries downstream delivery when it receives a NACK.             |
| [`processor:type_router`](src/processors/signal_type_router/README.md)                          | default | experimental | Routes OTAP payloads to output ports by signal type.             |
| [`processor:temporal_reaggregation`](src/processors/temporal_reaggregation_processor/README.md) | default | experimental | Reaggregates high-frequency metrics into lower-frequency output. |
| [`processor:transform`](src/processors/transform_processor/README.md)                           | default | experimental | Applies query-language transformations to OTAP batches.          |

<!-- markdownlint-enable MD013 -->

For a behavioral processor taxonomy, see
[`docs/processors.md`](../../docs/processors.md).

## Exporters

Exporters send data out of a pipeline.

<!-- markdownlint-disable MD013 -->

| Type                                                               | Feature | Stability    | Description                                                 |
| ------------------------------------------------------------------ | ------- | ------------ | ----------------------------------------------------------- |
| [`exporter:console`](src/exporters/console_exporter/README.md)     | default | experimental | Prints OTLP logs, metrics, and traces for local inspection. |
| [`exporter:error`](src/exporters/error_exporter/README.md)         | default | experimental | Rejects every received message with a configured NACK.      |
| [`exporter:noop`](src/exporters/noop_exporter/README.md)           | default | experimental | Acknowledges and discards every received message.           |
| [`exporter:otap`](src/exporters/otap_exporter/README.md)           | default | experimental | Sends OTAP Arrow payloads over gRPC streams.                |
| [`exporter:otlp_grpc`](src/exporters/otlp_grpc_exporter/README.md) | default | experimental | Sends telemetry as unary OTLP/gRPC export requests.         |
| [`exporter:otlp_http`](src/exporters/otlp_http_exporter/README.md) | default | experimental | Sends telemetry to OTLP/HTTP endpoints.                     |
| [`exporter:parquet`](src/exporters/parquet_exporter/README.md)     | default | experimental | Writes OTAP batches as Parquet files.                       |
| [`exporter:perf`](src/exporters/perf_exporter/README.md)           | default | experimental | Reports pipeline throughput and optional resource usage.    |
| [`exporter:topic`](src/exporters/topic_exporter/README.md)         | default | experimental | Publishes data to a named in-process topic.                 |

<!-- markdownlint-enable MD013 -->

## Maintenance Notes

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
