# Resource Validator Processor

The Resource Validator Processor validates that telemetry data contains a
required resource attribute with a value from an allowed list. Requests that
fail validation are permanently NACKed, enabling clients to detect
misconfiguration immediately rather than having data silently dropped.

## Example Use Case

In multi-tenant Azure environments, telemetry includes a `microsoft.resourceId`
resource attribute containing the Azure Resource Manager (ARM) resource ID.
The collector must:

1. Validate the attribute exists on the Resource
2. Check value is in an allowed list
3. Reject with error (HTTP 400 / gRPC INVALID_ARGUMENT) on failure

This enables clients to detect misconfiguration immediately rather than having
data silently dropped.

## Why Existing Processors Don't Work

| Processor            | Behavior                         | Gap                |
| -------------------- | -------------------------------- | ------------------ |
| Filter Processor     | Silently drops non-matching data | No error to client |
| Transform Processor  | Modifies/transforms data         | Cannot reject      |
| Attributes Processor | Adds/updates/deletes attributes  | Cannot reject      |

None validate attribute values against an allowlist with error propagation
to client.

## Configuration

```yaml
processors:
  resource_validator:
    # The resource attribute key that must be present (default: "microsoft.resourceId")
    required_attribute: "microsoft.resourceId"

    # List of allowed values (required - empty list rejects all values)
    allowed_values:
      - "/subscriptions/xxx/resourceGroups/yyy/..."
      - "/subscriptions/aaa/resourceGroups/bbb/..."

    # Case-insensitive comparison (default: false)
    case_insensitive: true
```

## Behavior

| Condition                                    | Result                    |
| -------------------------------------------- | ------------------------- |
| Attribute present with value in allowed list | Pass through              |
| Attribute present, empty allowed_values list | Permanent NACK - HTTP 400 |
| Attribute missing                            | Permanent NACK - HTTP 400 |
| Attribute wrong type (not string)            | Permanent NACK - HTTP 400 |
| Attribute value not in allowed list          | Permanent NACK - HTTP 400 |

## Metrics

- `resource_validator_batches_accepted` - Batches that passed validation
- `resource_validator_batches_rejected_missing` - Rejected: missing attribute
- `resource_validator_batches_rejected_not_allowed` - Rejected: invalid value

## Feature Flag

This processor is experimental and requires the `resource-validator-processor`
feature flag:

```toml
[dependencies]
otap-df-otap = { version = "...", features = ["resource-validator-processor"] }
```

## Extensibility for Dynamic Auth Context

The processor is designed to be extensible for future dynamic validation via
auth context:

### Current Implementation (Static)

Uses `allowed_values` from configuration:

```yaml
processors:
  resource_validator:
    allowed_values:
      - "/subscriptions/xxx/..."  # Static list from config
```

### Future Implementation (Dynamic)

When SAT auth extension is ready, allowed values can come from request auth
context:

```text
+-----------+   +------------------+   +--------------------+   +----------+
| Client    |-->| OTLP Receiver    |-->| Resource Validator |-->| Exporter |
| + token   |   | + Auth Extension |   | Processor          |   |          |
+-----------+   +------------------+   +--------------------+   +----------+
                        |                       |
                        v                       v
                 Auth Extension:         Processor reads from
                 1. Validates token      context instead of config
                 2. Extracts claims      via get_allowed_values()
                 3. Sets ctx.auth
```

### Extension Points

The processor provides these extension points for dynamic auth:

1. **`AllowedValuesSource` enum**: Supports `Static` (current) and `Dynamic`
   (future) modes
2. **`get_allowed_values()` method**: Returns allowed values for a request;
   can be extended to check `pdata.context().auth()` first
3. **Fallback support**: Dynamic mode includes config fallback for requests
   without auth context

When the SAT auth extension is implemented, the `get_allowed_values()` method
can be updated to:

1. Check if the request context contains auth information
2. Extract allowed resource IDs from auth claims
3. Fall back to static config if auth context is unavailable
