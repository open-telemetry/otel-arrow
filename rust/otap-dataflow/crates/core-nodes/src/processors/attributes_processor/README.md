# Attributes Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:attribute` (`urn:otel:processor:attribute`)
- Feature gate: Always enabled
- Stability: Experimental

## Overview

The attributes processor mutates OpenTelemetry attributes in OTAP batches. It
supports Collector-style action lists for deleting, inserting, upserting,
updating, renaming, and hashing attributes.

The processor can apply actions to signal attributes, resource attributes,
scope attributes, or any combination of those domains.

## Getting Started

Start with an ordered action list and the attribute domains to mutate:

```yaml
type: processor:attribute
config:
  apply_to: ["resource", "signal"]
  actions:
    - action: upsert
      key: deployment.environment
      value:
        string: prod
    - action: hash
      key: user.email
      salt: "tenant-specific-salt"
```

## Configuration

```yaml
type: processor:attribute
config:
  # Attribute domains to mutate (default: ["signal"]).
  # Supported values are "signal", "resource", and "scope".
  apply_to: ["resource", "signal"]

  # Ordered list of attribute actions (default: []).
  actions:
    - action: delete
      key: temporary.attribute
    - action: insert
      key: deployment.environment
      value:
        string: prod
    - action: upsert
      key: service.namespace
      value:
        string: checkout
    - action: update
      key: service.version
      value:
        string: "1.2.3"
    - action: rename
      source_key: service
      destination_key: service.name
    - action: hash
      key: user.email
      algorithm: sha256
      salt: "tenant-specific-salt"
```

Supported `apply_to` values are `signal`, `resource`, and `scope`.

Supported actions:

- `delete` requires `key` and deletes an attribute.
- `insert` requires `key` and one value source, and inserts a value only when the key is
  absent.
- `upsert` requires `key` and one value source, and inserts or replaces a value.
- `update` requires `key` and one value source, and replaces a value only when the key
  exists.
- `rename` requires `source_key` and `destination_key`, and renames an
  attribute key.
- `hash` requires `key` and replaces a scalar value with a salted hash.

`hash.algorithm` defaults to `sha256`; `hash.salt` defaults to an empty string.
Unsupported action variants are accepted for forward compatibility and ignored.

### Value sources and signal selection

`insert`, `upsert`, and `update` require exactly one of:

- `value`: a literal (string, boolean, signed integer, or double). Scalar syntax
  remains supported; explicit typed objects use `string`, `bool`, `int`, or
  `double`, for example `value: { string: prod }`.
- a dynamic object nested under `value`, containing a shared `source` selector
  with a `type` and a nonempty ordered `named` list:
  - `transport_header`: captured/stored header names.
  - `named`: materialized composite names or qualified `entry:member`
    references. A bare composite name requires exactly one member.
  Primitive selectors do not accept qualified references. `all_captured` and
  `none` are not attribute value sources. The former `from_headers` and
  `from_context` fields are not supported.

The processor declares its selected context reads to the engine. Referenced
composites and members must exist in the pipeline's context policy; the engine
selects only the required composite snapshots for materialization. Unknown
composites or members are configuration errors. Declarations do not broaden
receiver capture or authorization policy, and the processor does not create
context entries or change pdata storage.
Only sources that survive last-action-wins precedence on at least one signal
introduce context requirements. Fully overridden sources are not materialized
and do not require composite definitions.

Context values are text or binary in the shared context model. Scalar text
is projected unchanged as a string attribute, including empty strings and
numeric-looking text. Binary values, multi-valued members, and ambiguous
multi-member entries permanently Nack the message. Missing entries or members
cause no mutation.
For multiple `named` candidates, missing values fall
through in configured order; present but invalid scalar values permanently Nack.

Only headers already captured by receiver policy are accessible. This processor
does not change capture policy. Selection follows configured source-name order,
not request arrival order. For each name, the first usable captured text value
wins. Missing, binary, empty, or invalid UTF-8 values fall
through to later candidates. An unavailable source causes no attribute mutation.

Every action accepts an optional `signals: [logs, metrics, traces]` filter.
Omission selects all signals; empty lists and unknown signals are rejected.
This is independent of `apply_to`, which selects attribute domains.

Existing action precedence and collision validation are unchanged. For a given
signal, repeated keys within the same action kind use the last
configured action, whether static or dynamic. If that final dynamic source is
missing, the earlier action is not used as a fallback. Across action kinds,
keys must be disjoint, including rename sources and destinations. Collisions
are rejected at configuration time even if a source might be missing.
Actions with nonintersecting signal filters may reuse keys.

Dynamic sources are configured beneath the existing `value` key:

```yaml
actions:
  - action: insert
    key: server.address
    value:
      source:
        type: transport_header
        named: [x-forwarded-host, x-azure-originalhost, host]
  - action: insert
    key: request.sequence
    value:
      source:
        type: named
        named: [request_sequence]
```

Dynamic values are resolved once per pdata message and reused across its records.
Context sources are
read-only: entries must already exist in the pdata context. No fallback values
are generated, and log event and observed timestamps are never modified.

Live configuration updates replace the complete validated plan atomically.
Invalid updates are logged and retain the previous configuration. Processor
updates cannot introduce context reads outside the installed bindings; changes
to context requirements must go through pipeline reconfiguration. Reordering
fallback candidates or regrouping the same reads across actions preserves
bindings and takes effect without pipeline reconfiguration. Processor
metrics and compute timing survive updates. Materialization and enrichment failures
permanently Nack the message with its context; successful outputs retain normal
downstream Ack/Nack routing.

## Examples

Rename a resource attribute:

```yaml
type: processor:attribute
config:
  apply_to: ["resource"]
  actions:
    - action: rename
      source_key: service
      destination_key: service.name
```

Project preferred captured host headers into resources and expose an existing
materialized context entry as an attribute:

```yaml
type: processor:attribute
config:
  apply_to: [resource]
  actions:
    - action: insert
      key: server.address
      value:
        source:
          type: transport_header
          named: [x-forwarded-host, x-azure-originalhost, host]
    - action: insert
      key: http.host
      value:
        source:
          type: transport_header
          named: [x-forwarded-host, x-azure-originalhost, host]
    - action: insert
      key: request.sequence
      value:
        source:
          type: named
          named: [request_sequence]
      signals: [logs]
```

Configure host normalization on the receiver before this processor; otherwise
hosts are projected verbatim, including ports and comma-separated lists.
Define `request_sequence` in `policies.context.entries` and configure capture
for any primitive inputs it uses. The engine includes this selected composite
in the receiver's materialization plan. The action has no effect when that
composite is absent from a message, for example when a required input is missing.
See the [complete HTTP enrichment pipeline](../../../../../configs/otlp-http-attributes.yaml)
and [transport header capture policy](../../../../../docs/transport-headers.md).

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.attributes`

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.attributes.transforms` | `{transform}` | Number of transform attempts, partitioned by `outcome` (`success` or `failure`). |
| `processor.attributes.modified.entries` | `{attr}` | Total number of attribute entries modified, partitioned by `action` and `domain`. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| `attributes_processor.reconfigure_error` | WARN | Invalid configuration update rejected; previous configuration retained. |

## Limits

- `apply_to` values other than `signal`, `resource`, or `scope` are rejected.
- Hashing supports scalar values and currently documents `sha256`.
- The same key cannot currently be used by more than one action in a single
  action list; see [issue #1036](https://github.com/open-telemetry/otel-arrow/issues/1036).
- Unsupported actions deserialize but are ignored.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
