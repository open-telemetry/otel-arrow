# Transform Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:processor:transform`
- Type shortcut: `processor:transform`
- Feature gate: Default
- Stability: Experimental

## Overview

The transform processor applies query-language transformations to OTAP batches.
It currently accepts KQL, OPL, or OTTL log statements and may emit zero, one, or
multiple output batches depending on the transformation.

This processor and its query engine integration are under active development.

## Configuration

Exactly one query form is selected by the flattened query enum.

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `kql_query` | string | unset | KQL transformation query. |
| `opl_query` | string | unset | OPL transformation query. |
| `ottl.log_statements` | array | unset | OTTL statements for logs. |
| `inbound_request_limit` | non-zero integer | `1024` | Pending inbound request tracking limit. |
| `outbound_request_limit` | non-zero integer | `512` | Pending outbound request tracking limit. |
| `skip_sanitize_result` | bool | `false` | Skips result sanitization when `true`. |
| `filter_attribute_keys_case_sensitive` | bool | `true` | Controls filter attribute key matching. |

## Examples

```yaml
type: processor:transform
config:
  kql_query: "logs | where body != ''"
```

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
- OTTL support currently documents log statements only.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Query engine](../../../../query-engine/README.md)
- [Core node catalog](../../../README.md)
