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

| Metric | Unit | Attributes | Description |
| --- | --- | --- | --- |
| `processor.transform.operations` | `{operation}` | `language`, `signal`, `outcome` | Number of matching input messages whose local transform operation terminated. |
| `processor.transform.failures` | `{operation}` | `language`, `signal`, `error.type` | Failed transform operations grouped by actionable error category. |

The bounded `language` attribute is fixed for a processor instance and is one
of `kql`, `opl`, or `ottl`. The `signal` attribute is one of `traces`,
`metrics`, or `logs`.

An operation covers all configured transforms that match one input message.
Messages with no matching transform are passed through without recording an
operation. The `outcome` is `success` only after the transformed default and
routed outputs have been accepted by their immediate sends; otherwise it is
`failure`. Downstream acknowledgements do not change this local outcome.

The bounded `error.type` values are:

- `payload_conversion`: the input could not be converted to OTAP Arrow records.
- `id_decode`: transport-optimized identifiers could not be decoded.
- `query_execution`: the configured query pipeline failed while executing.
- `route_not_configured`: the query referenced an unconfigured output route.
- `inbound_capacity`: `inbound_request_limit` was exhausted.
- `outbound_capacity`: `outbound_request_limit` was exhausted.
- `output_send`: an immediate default or routed output send failed.
- `internal`: an internal transform processor invariant failed.

Common engine telemetry provides total consumed and produced traffic, dropped
items, channel send failures, and downstream acknowledgement outcomes.

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
