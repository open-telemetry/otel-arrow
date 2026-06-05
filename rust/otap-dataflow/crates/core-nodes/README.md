# Core Nodes

This crate contains the built-in OTAP Dataflow Engine receivers, processors,
and exporters. These nodes are registered into the OTAP pipeline factory maps
when the crate is linked by the engine binary.

For help writing runtime YAML, start at
[`docs/configuration.md`](../../docs/configuration.md). For exact runtime
configuration semantics, see
[`docs/configuration-model.md`](../../docs/configuration-model.md). For the
canonical node URN format, see [`docs/urns.md`](../../docs/urns.md).

## Documentation Contract

Each registered node should have a README next to its implementation directory.
Per-node READMEs use predictable headings so the same content can be read by
humans and discovered by agents:

- `Metadata`
- `Overview`
- `Configuration`
- `Examples`
- `Telemetry`
- `Limits`
- `Related Docs`

When a node does not have an explicit compatibility guarantee, its stability is
documented as `experimental`.

The `Metadata` section should be a rendered list, one field per list item,
instead of a table:

```markdown
- Full URN: `urn:otel:<role>:<type>`
- Type shortcut: `<role>:<type>`
- Feature gate: Default
- Stability: Experimental
```

The node role is already implied by the URN and shortcut, and implementation
file names are intentionally omitted because the README sits next to that code.

The `Configuration` section should describe only the node's `config` shape and
node-local options such as `outputs` when they are required to understand the
node. Examples should use the smallest useful node-level snippet:

```yaml
type: processor:example
config:
  option: value
```

Include `groups`, `pipelines`, `topics`, or full engine structure only when
that surrounding structure is required to explain the node behavior.

The `Telemetry` section should appear before `Limits` and describe:

- `Metric Sets`: for each node-specific metric set, a table with `Metric`,
  `Unit`, and `Description` columns. State explicitly when a node has no
  node-specific metric set.
- `Events`: a table with `Event`, `Severity`, and `Description` columns. State
  explicitly when a node does not emit node-specific events.

## Layout

```text
src/
  exporters/
    <node_name>/
  processors/
    <node_name>/
  receivers/
    <node_name>/
```

The directory name is the implementation module. The configured node `type`
uses the OTel shortcut form, such as `receiver:otlp`, or the full URN, such as
`urn:otel:receiver:otlp`.

## Exporters

<!-- markdownlint-disable MD013 -->
| Node | Type | Feature | Stability | Documentation |
| --- | --- | --- | --- | --- |
| `console_exporter` | `exporter:console` | default | experimental | [`src/exporters/console_exporter/README.md`](src/exporters/console_exporter/README.md) |
| `error_exporter` | `exporter:error` | default | experimental | [`src/exporters/error_exporter/README.md`](src/exporters/error_exporter/README.md) |
| `noop_exporter` | `exporter:noop` | default | experimental | [`src/exporters/noop_exporter/README.md`](src/exporters/noop_exporter/README.md) |
| `otap_exporter` | `exporter:otap` | default | experimental | [`src/exporters/otap_exporter/README.md`](src/exporters/otap_exporter/README.md) |
| `otlp_grpc_exporter` | `exporter:otlp_grpc` | default | experimental | [`src/exporters/otlp_grpc_exporter/README.md`](src/exporters/otlp_grpc_exporter/README.md) |
| `otlp_http_exporter` | `exporter:otlp_http` | default | experimental | [`src/exporters/otlp_http_exporter/README.md`](src/exporters/otlp_http_exporter/README.md) |
| `parquet_exporter` | `exporter:parquet` | default | experimental | [`src/exporters/parquet_exporter/README.md`](src/exporters/parquet_exporter/README.md) |
| `perf_exporter` | `exporter:perf` | default | experimental | [`src/exporters/perf_exporter/README.md`](src/exporters/perf_exporter/README.md) |
| `topic_exporter` | `exporter:topic` | default | experimental | [`src/exporters/topic_exporter/README.md`](src/exporters/topic_exporter/README.md) |
<!-- markdownlint-enable MD013 -->

## Processors

<!-- markdownlint-disable MD013 -->
| Node | Type | Feature | Stability | Documentation |
| --- | --- | --- | --- | --- |
| `attributes_processor` | `processor:attribute` | default | experimental | [`src/processors/attributes_processor/README.md`](src/processors/attributes_processor/README.md) |
| `batch_processor` | `processor:batch` | default | experimental | [`src/processors/batch_processor/README.md`](src/processors/batch_processor/README.md) |
| `content_router` | `processor:content_router` | default | experimental | [`src/processors/content_router/README.md`](src/processors/content_router/README.md) |
| `debug_processor` | `processor:debug` | default | experimental | [`src/processors/debug_processor/README.md`](src/processors/debug_processor/README.md) |
| `delay_processor` | `processor:delay` | default | experimental | [`src/processors/delay_processor/README.md`](src/processors/delay_processor/README.md) |
| `durable_buffer_processor` | `processor:durable_buffer` | default | experimental | [`src/processors/durable_buffer_processor/README.md`](src/processors/durable_buffer_processor/README.md) |
| `fanout_processor` | `processor:fanout` | default | experimental | [`src/processors/fanout_processor/README.md`](src/processors/fanout_processor/README.md) |
| `filter_processor` | `processor:filter` | default | experimental | [`src/processors/filter_processor/README.md`](src/processors/filter_processor/README.md) |
| `log_sampling_processor` | `processor:log_sampling` | default | experimental | [`src/processors/log_sampling_processor/README.md`](src/processors/log_sampling_processor/README.md) |
| `retry_processor` | `processor:retry` | default | experimental | [`src/processors/retry_processor/README.md`](src/processors/retry_processor/README.md) |
| `signal_type_router` | `processor:type_router` | default | experimental | [`src/processors/signal_type_router/README.md`](src/processors/signal_type_router/README.md) |
| `temporal_reaggregation_processor` | `processor:temporal_reaggregation` | default | experimental | [`src/processors/temporal_reaggregation_processor/README.md`](src/processors/temporal_reaggregation_processor/README.md) |
| `transform_processor` | `processor:transform` | default | experimental | [`src/processors/transform_processor/README.md`](src/processors/transform_processor/README.md) |
<!-- markdownlint-enable MD013 -->

For a behavioral processor taxonomy, see
[`docs/processors.md`](../../docs/processors.md).

## Receivers

<!-- markdownlint-disable MD013 -->
| Node | Type | Feature | Stability | Documentation |
| --- | --- | --- | --- | --- |
| `host_metrics_receiver` | `receiver:host_metrics` | default | experimental | [`src/receivers/host_metrics_receiver/README.md`](src/receivers/host_metrics_receiver/README.md) |
| `internal_telemetry_receiver` | `receiver:internal_telemetry` | default | experimental | [`src/receivers/internal_telemetry_receiver/README.md`](src/receivers/internal_telemetry_receiver/README.md) |
| `journald_receiver` | `receiver:journald` | default | experimental | [`src/receivers/journald_receiver/README.md`](src/receivers/journald_receiver/README.md) |
| `otap_receiver` | `receiver:otap` | default | experimental | [`src/receivers/otap_receiver/README.md`](src/receivers/otap_receiver/README.md) |
| `otlp_receiver` | `receiver:otlp` | default | experimental | [`src/receivers/otlp_receiver/README.md`](src/receivers/otlp_receiver/README.md) |
| `syslog_cef_receiver` | `receiver:syslog_cef` | default | experimental | [`src/receivers/syslog_cef_receiver/README.md`](src/receivers/syslog_cef_receiver/README.md) |
| `topic_receiver` | `receiver:topic` | default | experimental | [`src/receivers/topic_receiver/README.md`](src/receivers/topic_receiver/README.md) |
| `traffic_generator` | `receiver:traffic_generator` | `dev-tools` | experimental | [`src/receivers/traffic_generator/README.md`](src/receivers/traffic_generator/README.md) |
<!-- markdownlint-enable MD013 -->

## Maintenance Notes

- Keep node READMEs in the same directory as their `mod.rs` or `config.rs`.
- Update this index when adding, removing, renaming, or feature-gating a node.
- Keep examples in the native `otel_dataflow/v1` runtime format.
- Prefer linking to shared policy docs instead of duplicating long descriptions.
