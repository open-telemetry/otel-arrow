# Development Nodes

Development nodes support testing, failure injection, and benchmarking. They
are intentionally kept outside `core-nodes` so published runtime crates do not
depend on the Git-only Weaver crates.

The default `df_engine` build includes these nodes through the `dev-tools`
feature. Builds using `--no-default-features` must enable `dev-tools`
explicitly to register them.

For production-oriented built-in components, see the
[core-node catalog](../core-nodes/README.md).

## Receivers

| Type | Description |
| --- | --- |
| [`receiver:traffic_generator`](src/receivers/traffic_generator/README.md) | Emits synthetic or semantic-convention-derived test traffic. |

## Processors

| Type | Description |
| --- | --- |
| [`processor:delay`](src/processors/delay_processor/README.md) | Sleeps for a configured duration before forwarding each message. |

## Exporters

| Type | Description |
| --- | --- |
| [`exporter:error`](src/exporters/error_exporter/README.md) | Rejects every received message with a configured NACK. |
| [`exporter:perf`](src/exporters/perf_exporter/README.md) | Reports pipeline throughput and optional resource usage. |
