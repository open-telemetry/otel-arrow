# Attributes Processor

<!-- markdownlint-disable MD013 -->

## Metadata

- Full URN: `urn:otel:processor:attribute`
- Type shortcut: `processor:attribute`
- Feature gate: Default
- Stability: Experimental

## Overview

The attributes processor mutates OpenTelemetry attributes in OTAP batches. It
supports Collector-style action lists for deleting, inserting, upserting,
updating, renaming, and hashing attributes.

The processor can apply actions to signal attributes, resource attributes,
scope attributes, or any combination of those domains.

## Configuration

| Field | Type | Default | Description |
| --- | --- | --- | --- |
| `actions` | array | `[]` | Ordered list of attribute actions. |
| `apply_to` | array | `["signal"]` | Attribute domains to mutate. |

Supported `apply_to` values are `signal`, `resource`, and `scope`.

Supported actions:

| Action | Required fields | Description |
| --- | --- | --- |
| `delete` | `key` | Deletes an attribute. |
| `insert` | `key`, `value` | Inserts a value only when the key is absent. |
| `upsert` | `key`, `value` | Inserts or replaces a value. |
| `update` | `key`, `value` | Replaces a value only when the key exists. |
| `rename` | `source_key`, `destination_key` | Renames an attribute key. |
| `hash` | `key` | Replaces a scalar value with a salted hash. |

`hash.algorithm` defaults to `sha256`; `hash.salt` defaults to an empty string.
Unsupported action variants are accepted for forward compatibility and ignored.

## Examples

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
- Unsupported actions deserialize but are ignored.

## Related Docs

- [Configuration model](../../../../../docs/configuration-model.md)
- [Processor taxonomy](../../../../../docs/processors.md)
- [Core node catalog](../../../README.md)
