# Attributes Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Type: `processor:attribute` (`urn:otel:processor:attribute`)
- Feature gate: Default
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
- `insert` requires `key` and `value`, and inserts a value only when the key is
  absent.
- `upsert` requires `key` and `value`, and inserts or replaces a value.
- `update` requires `key` and `value`, and replaces a value only when the key
  exists.
- `rename` requires `source_key` and `destination_key`, and renames an
  attribute key.
- `hash` requires `key` and replaces a scalar value with a salted hash.

`hash.algorithm` defaults to `sha256`; `hash.salt` defaults to an empty string.
Unsupported action variants are accepted for forward compatibility and ignored.

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

## Telemetry

These tables list telemetry emitted directly by this node. Common engine
runtime metric sets may also be attached by the pipeline telemetry policy.

### Metric Sets

#### `processor.attributes`

| Metric | Unit | Description |
| --- | --- | --- |
| `processor.attributes.transform_failed` | `{op}` | Number of failed transform attempts. |
| `processor.attributes.renamed_entries` | `{attr}` | Total number of attribute entries actually renamed. |
| `processor.attributes.deleted_entries` | `{attr}` | Total number of attribute entries actually deleted. |
| `processor.attributes.inserted_entries` | `{attr}` | Total number of attribute entries actually inserted. |
| `processor.attributes.upserted_entries` | `{attr}` | Total number of attribute entries actually upserted. |
| `processor.attributes.updated_entries` | `{attr}` | Total number of attribute entries actually updated. |
| `processor.attributes.hashed_entries` | `{attr}` | Total number of attribute entries actually hashed. |
| `processor.attributes.domains_signal` | `{apply}` | Number of times transforms were applied to signal-level payloads. |
| `processor.attributes.domains_resource` | `{apply}` | Number of times transforms were applied to resource-level payloads. |
| `processor.attributes.domains_scope` | `{apply}` | Number of times transforms were applied to scope-level payloads. |

### Events

| Event | Severity | Description |
| --- | --- | --- |
| *None* | N/A | No node-specific events are emitted. |

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
