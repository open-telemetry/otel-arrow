<!-- markdownlint-disable MD013 -->

# Contrib Nodes

Contrib nodes are optional receivers, processors, and exporters that extend the
default OTel Arrow Dataflow Engine build. Use this catalog to find the node
`type` to put in runtime YAML and to open the node-specific documentation for
configuration examples, telemetry, limits, and stability notes.

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

Contrib nodes are enabled through individual feature gates or aggregate feature
gates such as `contrib-receivers`, `contrib-processors`, and
`contrib-exporters`. The `Stability` column uses the levels defined in
[`docs/node-stability.md`](../../docs/node-stability.md). A node documented as
`experimental` or `alpha` has no stable compatibility guarantee yet, and its
behavior or configuration can change between releases.

## Node Type Syntax

Use the `Type` value from the tables below in a node definition:

```yaml
type: receiver:user_events
```

The full URN form is also accepted for OTel nodes, and some vendor-specific
contrib nodes currently document only their full URN:

```yaml
type: urn:microsoft:exporter:geneva
```

For the canonical node URN format, see [`docs/urns.md`](../../docs/urns.md).

## Receivers

Receivers ingest data into a pipeline.

| Type                                                                   | Feature                | Stability    | Description                                      |
| ---------------------------------------------------------------------- | ---------------------- | ------------ | ------------------------------------------------ |
| [`receiver:kafka`](src/receivers/kafka_receiver/README.md)             | `kafka-receiver`       | experimental | Consumes traces, metrics, and logs from Kafka.   |
| [`receiver:user_events`](src/receivers/user_events_receiver/README.md) | `user_events-receiver` | experimental | Ingests Linux `user_events` tracepoints as logs. |

## Processors

Processors transform or validate data already moving through a pipeline.

| Type                                                                                        | Feature                         | Stability    | Description                                                  |
| ------------------------------------------------------------------------------------------- | ------------------------------- | ------------ | ------------------------------------------------------------ |
| [`processor:condense_attributes`](src/processors/condense_attributes_processor/README.md)   | `condense-attributes-processor` | experimental | Condenses multiple log attributes into one string attribute. |
| [`urn:microsoft:processor:recordset_kql`](src/processors/recordset_kql_processor/README.md) | `recordset-kql-processor`       | experimental | Runs KQL expressions over OTAP data in an opinionated shape. |
| [`processor:resource_validator`](src/processors/resource_validator_processor/README.md)     | `resource-validator-processor`  | experimental | NACKs data missing required resource attribute values.       |

## Exporters

Exporters send data out of a pipeline.

| Type                                                                                     | Feature                  | Stability    | Description                                             |
| ---------------------------------------------------------------------------------------- | ------------------------ | ------------ | ------------------------------------------------------- |
| [`urn:microsoft:exporter:azure_monitor`](src/exporters/azure_monitor_exporter/README.md) | `azure-monitor-exporter` | alpha        | Sends OpenTelemetry logs to Azure Monitor.              |
| [`exporter:clickhouse`](src/exporters/clickhouse_exporter/README.md)                     | `clickhouse-exporter`    | experimental | Inserts OTAP logs and traces into ClickHouse over HTTP. |
| [`urn:microsoft:exporter:geneva`](src/exporters/geneva_exporter/README.md)               | `geneva-exporter`        | alpha        | Sends telemetry to Microsoft's Geneva backend.          |
| [`exporter:kafka`](src/exporters/kafka_exporter/README.md)                               | `kafka-exporter`         | experimental | Produces traces, metrics, and logs to Kafka.            |

## Feature Aggregates

- `contrib-receivers`: enables all contrib receivers.
- `contrib-processors`: enables all contrib processors.
- `contrib-exporters`: enables all contrib exporters.

When these features are enabled in the top-level binary, their factories are
registered into the OTAP pipeline factory maps.
