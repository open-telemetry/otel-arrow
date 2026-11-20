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
- `insert` requires `key` and exactly one of `value` or `context_value`, and
  inserts a value only when the key is absent.
- `upsert` requires `key` and exactly one of `value` or `context_value`, and
  inserts or replaces a value.
- `update` requires `key` and exactly one of `value` or `context_value`, and
  replaces a value only when the key exists.
- `rename` requires `source_key` and `destination_key`, and renames an
  attribute key.
- `hash` requires `key` and replaces a scalar value with a salted hash.

`hash.algorithm` defaults to `sha256`; `hash.salt` defaults to an empty string.
Unsupported action variants are accepted for forward compatibility and ignored.

### Value sources and signal selection

`insert`, `upsert`, and `update` require exactly one of `value` or
`context_value`. Supplying both fields or neither is a configuration error.

#### Literal values

Use `value` for literal attributes. Typed objects support `string`, `bool`,
`int` (signed 64-bit), and `double`:

```yaml
actions:
  - action: insert
    key: deployment.environment
    value:
      string: prod
  - action: insert
    key: feature.enabled
    value:
      bool: true
  - action: insert
    key: request.sequence
    value:
      int: 1234
  - action: insert
    key: sample.ratio
    value:
      double: 0.5
```

Scalar shorthand is also supported and produces the same attribute values.
The extended configuration below includes both literal forms and context sources.

An optional sibling `type` selects the literal's attribute type: `string`,
`bool`, `int` (signed 64-bit), or `double`. Without `type`, the type is inferred
from the scalar or typed-object value. Explicit types must match typed-object
keys. Strings are not converted to numbers or booleans. A numeric scalar can
use `type: double` to produce a floating-point attribute. `type` is only valid
with a literal `value`, not with `context_value`.

#### Context values

Use the sibling `context_value` field to read metadata instead of a literal.

A `context_value` requires a nonempty ordered `named` list. The `domain`
defaults to `transport_header` for stored header names. Set it to
`authorized_identity` for captured verified identity claims, or explicitly
set `domain: null` for materialized composite names or qualified `entry:member`
references. A bare composite name requires exactly one member. Primitive
selectors do not accept qualified references.

Referenced composites and members must exist in the pipeline's context policy.
Unknown composites or members are configuration errors. The engine materializes
only composites referenced by effective actions. Sources do not change receiver
capture or authorization policies or create context entries.
Only actions that survive last-action-wins precedence on at least one signal
introduce context requirements. Fully overridden sources require neither
composite definitions nor materialization.

Context values are text or binary in the shared context model. Usable scalar
text is projected unchanged as a string attribute, including numeric-looking
text. For every domain, missing, binary, invalid UTF-8, empty, or whitespace-only
values fall through to later `named` candidates in configured order. An
unavailable source causes no attribute mutation. Usable text is not trimmed.
Identity claims and composite members must be scalar. Multi-valued members
and ambiguous multi-member composites permanently Nack the message instead of
falling through. Literal empty strings remain supported.

Only headers captured by receiver policy are accessible. Selection follows
configured source-name order, not request arrival order. For each name, the
first usable captured text value wins; unusable repeated values are skipped
before trying the next name.

Every action accepts an optional `signals: [logs, metrics, traces]` filter.
Omission selects all signals; empty lists and unknown signals are rejected.
This is independent of `apply_to`, which selects attribute domains.

For each signal, repeated keys within the same action kind use the last
configured action, whether static or dynamic. If that final dynamic source is
missing, the earlier action is not used as a fallback. Across action kinds,
keys must be disjoint, including rename sources and destinations. Collisions
are rejected at configuration time even if a source might be missing.
Actions with nonintersecting signal filters may reuse keys.

This complete configuration retains the existing actions and adds scalar
literals, context sources, and a signal filter. Comments identify each addition:

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

    # Scalar shorthand: string, boolean, integer, and double literals.
    # These use value without a typed-object wrapper.
    - action: insert
      key: deployment.region
      value: us-east-1
    - action: insert
      key: feature.enabled
      value: true
    - action: insert
      key: retry.limit
      value: 1234
    - action: insert
      key: sample.ratio
      value: 0.5

    # Optional literal type: validate a string or select a numeric field type.
    - action: insert
      key: deployment.stage
      value: prod
      type: string
    - action: insert
      key: retry.backoff
      value: 1
      type: double

    # Context value: read the first usable captured header.
    # Omitted domain defaults to transport_header; do not also set value.
    - action: insert
      key: server.address
      context_value:
        named: [x-forwarded-host, x-azure-originalhost, host]

    # Context value: read a captured verified identity claim.
    - action: insert
      key: auth.user.id
      context_value:
        domain: authorized_identity
        named: [UserId]

    # Context value: read an existing materialized context entry.
    # Explicit null selects composites rather than transport headers.
    - action: insert
      key: request.sequence
      context_value:
        domain: null
        named: [request_sequence]
      # Signal filter: apply only to logs, independently of apply_to.
      signals: [logs]
```

Dynamic values are resolved once per pdata message and reused across its records.
Context sources are read-only: entries must already exist in the pdata context.
No fallback values are generated, and log event and observed timestamps are
never modified.

Valid live updates atomically replace the configuration. Invalid updates are
logged and leave the current configuration active. Processor
updates cannot introduce context reads outside the installed bindings; changes
to context requirements must go through pipeline reconfiguration. Reordering
fallback candidates or regrouping the same reads across actions preserves
bindings and takes effect without pipeline reconfiguration. Updates do not reset
metrics or compute timing. Materialization and enrichment failures
permanently Nack the message with its context; successful outputs retain normal
downstream Ack/Nack routing.
Compute timing records dynamic source resolution and attribute mutation as
separate observations, each with its own success or failure outcome. Literal-only
plans record only mutation time. Decoding and downstream waits are excluded.
Sources are resolved before decoding, so missing-only plans forward the original
encoded message unchanged but still record resolution time. Decoding failures
retain any completed resolution observation. Empty plans record no observations.

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
      context_value:
        named: [x-forwarded-host, x-azure-originalhost, host]
    - action: insert
      key: http.host
      context_value:
        named: [x-forwarded-host, x-azure-originalhost, host]
    - action: insert
      key: request.sequence
      context_value:
        domain: null
        named: [request_sequence]
      signals: [logs]
```

Headers are projected verbatim, including ports and comma-separated lists.
This processor does not parse hosts or select elements within a header value.
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
