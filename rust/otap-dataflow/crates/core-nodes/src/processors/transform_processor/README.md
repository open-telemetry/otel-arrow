# Transform Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:transform` (`urn:otel:processor:transform`)
- Feature gate: Default
- Stability: Experimental

## Overview

The transform processor applies query-language transformations to OTAP batches.
It currently accepts KQL, OPL, or OTTL log statements and may emit zero, one, or
multiple output batches depending on the transformation.

This processor and its query engine integration are under active development.

## Getting Started

Write the query to transform your telemetry in your preferred language:

```yaml
type: processor:transform
config:
  kql_query: "logs | where body != ''"

  # Pending request tracking limits.
  inbound_request_limit: 1024
  outbound_request_limit: 512

  # Skips result sanitization when true (default: false).
  skip_sanitize_result: false

  # Controls filter attribute key matching (default: true).
  filter_attribute_keys_case_sensitive: true
```

## Examples

OTTL log statements:

```yaml
type: processor:transform
config:
  ottl:
    log_statements:
      - set(attributes["processed"], true)
```

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.transform`

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.transform.msgs_transformed` | `{msg}` | Number of messages successfully transformed. |
| `processor.transform.msgs_transform_failed` | `{msg}` | Number of failed transform attempts. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

## Limits

- The transformation query surface is still evolving.
- `skip_sanitize_result: true` can leave removed data in unused Arrow buffers;
  keep the default when transformations redact sensitive data.
- OTTL currently supports only updating setting log fields to literal values. Additional
  operations such as filtering, function evaluation, and other expression types, as well as
  applying OTTL transforms to spans and metrics are not yet supported.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Query engine](../../../../query-engine/README.md)
- [Core node catalog](../../../README.md)
