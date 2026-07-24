# Partition Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:partition` (`urn:otel:processor:partition`)
- Feature gate: Default
- Stability: experimental

## Overview

The partition processor can split a single batch into multiple partitions, each
consisting of the records that have the same value for some evaluated
partitioning expression. The value will be also be added as a header.

## Getting started

Write the expression to use when partitioning telemetry batches, and choose the
name of the header to which you'd like to have the partition value inserted.

```yaml
type: processor.partition
config:
  partition_by:
    opl_expression: resource.attributes["k8s.namespace.name"]
  partition_header_name: k8s-ns
```

The `opl_expression` can be any expression supported by the
[OPL](../../../../query-engine-languages/docs/opl-user-guide/).

## Telemetry

These tables list the telemetry directly emitted by this component. Common
engine runtime metric sets may also be attached by the pipeline telemetry
policy.

### Metric Sets

### `processor.partition`

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.partition.partition_operations_succeeded` | "{batch}" | Number of incoming batches that were successfully partitioned |
| `processor.partition.partition_operations_failed` | "{batch}" | Number of incoming batches that failed to be partitioned |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- OPL expression evaluation capabilities are still evolving.
- The default header serialization strategy (`to_bytes_lossy`) does not
  preserve type information of the partition value. If this is important,
  for your downstream nodes, the `json` strategy may be used instead.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Query engine](../../../../query-engine/README.md)
- [Core node catalog](../../../README.md)
