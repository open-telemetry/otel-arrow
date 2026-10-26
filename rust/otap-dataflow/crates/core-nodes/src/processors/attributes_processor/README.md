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
- a dynamic object nested under `value`, containing either `from_headers` (a
  nonempty ordered list of captured/stored header names) or `from_context` (a
  materialized context entry name or qualified `entry:member` reference).
  A bare entry name requires exactly one member. Only existing snapshots are
  read; the processor does not create context entries or change pdata storage.

Context values are text or binary in the shared context model. Scalar text
defaults to a string attribute, including empty strings and numeric-looking text.
Set `transform: string`, `bool`, `int`, or `double` alongside the source to
explicitly choose the attribute type. Boolean conversion accepts `true`/`false`;
integer conversion checks the signed 64-bit range. Invalid conversions, binary
values, multi-valued members, and ambiguous multi-member entries permanently
Nack the message. Missing entries or members cause no mutation.

Only headers already captured by receiver policy are accessible. This processor
does not change capture policy. Selection follows configured source-name order,
not request arrival order. For each name, the first usable captured text value
wins. Missing, binary, empty, invalid UTF-8, or malformed transformed values fall
through to later candidates. An unavailable source causes no attribute mutation.

Both header and context sources accept a nested `transform: string` (default,
preserving the text), `bool`, `int`, `double`, or `authority_host`.
The scalar conversions share the same implementation for both sources.
Invalid header conversions fall through to later captured candidates; invalid
context conversions permanently Nack the message. `authority_host` takes the
selected comma-separated value, trims it, parses an HTTP authority, and removes its port.
DNS, IPv4, and bracketed IPv6 hosts are supported; IPv6 brackets are retained.
Userinfo and invalid ports are rejected. Transforms on literals are invalid.

#### Authority list selection

With `transform: authority_host`, set `selection: first` (default) or
`selection: last` beneath `value`. This applies to both header and context sources.
For `first.example:80, last.example:443`, the results are `first.example` and
`last.example`, respectively. A single authority produces the same result for
either selection. Unknown or null selections, and selections with any other
transform, are rejected.

Selection applies **within one captured value**, not across header names or
repeated header fields. The configured header-name order and first-usable
captured-value behavior remain unchanged. An empty or invalid selected element
does not fall back to another element in that comma-separated list. For headers,
the processor tries the next captured value or source name instead; for context,
it permanently Nacks the message.

Choose based on your trusted proxy's behavior, not a universal proxy ordering:

- Azure Front Door [overwrites `X-Forwarded-Host`](https://learn.microsoft.com/en-us/azure/frontdoor/front-door-http-headers-protocol)
  with the original request host, so it normally supplies a single value.
- Azure Application Gateway [adds `x-original-host`](https://learn.microsoft.com/en-us/azure/application-gateway/how-application-gateway-works#modifications-to-the-request).
  Capture and select that name explicitly if needed; it is distinct from
  `x-azure-originalhost`.
- If a trusted proxy appends the authoritative host, use `last`. If it prepends
  it, use `first`. Neither choice establishes trust: prevent clients from
  bypassing that proxy and configure it to sanitize forwarded headers.
  Cloudflare's documented append behavior for `X-Forwarded-For` concerns IP
  addresses, not a general guarantee about `X-Forwarded-Host`.

For a trusted proxy configured to append the authoritative host:

```yaml
value:
  from_headers: [x-forwarded-host, host]
  transform: authority_host
  selection: last
```

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
      from_headers: [x-forwarded-host, x-azure-originalhost, host]
      transform: authority_host
  - action: insert
    key: request.sequence
    value:
      from_context: request_sequence
      transform: int
```

Dynamic values are resolved once per pdata message and reused across its records.
Integer conversions outside the signed 64-bit range permanently Nack the
message rather than wrapping, clamping, or silently skipping the value. Context sources are
read-only: entries must already exist in the pdata context. No fallback values
are generated, and log event and observed timestamps are never modified.

Live configuration updates replace the complete validated plan atomically.
Invalid updates are logged and retain the previous configuration. Processor
metrics and compute timing survive updates. Conversion and enrichment failures
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
        from_headers: [x-forwarded-host, x-azure-originalhost, host]
        transform: authority_host
    - action: insert
      key: http.host
      value:
        from_headers: [x-forwarded-host, x-azure-originalhost, host]
        transform: authority_host
    - action: insert
      key: request.sequence
      value:
        from_context: request_sequence
        transform: int
      signals: [logs]
```

The `request.sequence` action has no effect unless an upstream producer has
materialized a single-member `request_sequence` entry in the shared context.
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
