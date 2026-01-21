# Condense Attributes Processor

**Status:** WIP

## Overview

The Condense Attributes Processor is an experimental OTAP processor that
condenses multiple log attributes into a single string attribute. This is a
niche operation useful when working with firmly-structured data shapes that
include a specific catch-all field for non-critical attributes.

It is possible that this implementation could be folded into
`attributes_processor` but at the moment it is an isolated component.

### Important Notes

- The condense operation may be non-deterministic for the same logical data
  (producing `x=1;y=2` or `y=2;x=1`). This is currently deemed intentional and
  acceptable if considering the condensed destination key is aligned with an
  expected catch-all field. Users working with this catch-all field will likely
  split or parse the content at later stages as needed without a dependency on
  ordering.

## Feature Flag

This processor requires the `condense-attributes-processor` feature flag to be
enabled:

```bash
cargo build --features condense-attributes-processor
```

## Configuration

The processor accepts the following configuration options:

| Field | Type | Required | Description |
| ----- | ---- | -------- | ----------- |
| `destination_key` | string | **Yes** | The name of the attribute that will contain the condensed string |
| `delimiter` | string | **Yes** | The delimiter used to separate key=value pairs in the condensed string (must be a single character) |
| `source_keys` | array of strings | No | If provided, only attributes with these keys will be condensed. Other attributes are preserved as-is. |
| `exclude_keys` | array of strings | No | If provided, attributes with these keys will NOT be condensed. They are preserved as-is while all others are condensed. |

**Important Notes:**

- `source_keys` and `exclude_keys` are mutually exclusive. If both are provided,
  the processor will return an error.
- `destination_key` cannot be included in `source_keys` or `exclude_keys`.
  Configuration validation will fail if this occurs.
- If an input attribute has the same key as `destination_key`, it will be
  automatically skipped (not condensed) to prevent circular references.

## Behavior

### Condense All Attributes (Default)

When neither `source_keys` nor `exclude_keys` is specified, all attributes are
condensed into a single string.

**Example Configuration:**

```json
{
  "destination_key": "condensed",
  "delimiter": ";"
}
```

**Input Attributes:**

- `attr1` = "value1"
- `attr2` = 42
- `attr3` = true

**Output Attributes:**

- `condensed` = "attr1=value1;attr2=42;attr3=true"

### Condense Specific Attributes (source_keys)

When `source_keys` is provided, only the specified attributes are condensed. All
other attributes are preserved in their original form.

**Example Configuration:**

```json
{
  "destination_key": "condensed",
  "delimiter": ";",
  "source_keys": ["attr1", "attr2"]
}
```

**Input Attributes:**

- `attr1` = "value1"
- `attr2` = 42
- `attr3` = true

**Output Attributes:**

- `condensed` = "attr1=value1;attr2=42"
- `attr3` = true

### Condense All Except Specific Attributes (exclude_keys)

When `exclude_keys` is provided, all attributes except the specified ones are
condensed. The excluded attributes are preserved in their original form.

**Example Configuration:**

```json
{
  "destination_key": "condensed",
  "delimiter": ";",
  "exclude_keys": ["attr1"]
}
```

**Input Attributes:**

- `attr1` = "value1"
- `attr2` = 42
- `attr3` = true

**Output Attributes:**

- `condensed` = "attr2=42;attr3=true"
- `attr1` = "value1"
