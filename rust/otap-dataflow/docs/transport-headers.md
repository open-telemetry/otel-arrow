# Transport Header Policy

## Overview

The transport header policy controls end-to-end forwarding of
request-scoped transport metadata (gRPC metadata, HTTP headers)
through the pipeline. It operates in two phases:

1. **Capture** - Receiver nodes extract selected headers from inbound
   requests and attach them to the pipeline message context.
2. **Propagation** - Exporter nodes filter the captured headers and
   attach the approved subset to outbound requests.

The feature is entirely **opt-in**. When no transport header policy is
configured, no headers are captured or forwarded and there is zero
runtime overhead.

## Configuration Scope

The transport header policy can be set at multiple levels of the
configuration hierarchy. Each level inherits from its parent and can
override it.

### Policy Inheritance

```text
engine policies  (broadest scope)
  -> group policies
    -> pipeline policies  (narrowest scope before node overrides)
```

At each level, the `transport_headers` field is placed inside
`policies`:

```yaml
version: otel_dataflow/v1
policies:
  transport_headers:
    header_capture:
      # ...
    header_propagation:
      # ...
groups:
  my_group:
    policies:
      transport_headers:
        # overrides engine-level policy for this group
    pipelines:
      my_pipeline:
        policies:
          transport_headers:
            # overrides group-level policy for this pipeline
```

The most specific (narrowest) scope wins. If a pipeline defines its
own `transport_headers`, that policy is used for all nodes in that
pipeline regardless of what the group or engine level specifies.

### Node-Level Overrides

Individual receiver and exporter nodes can override the capture or
propagation policy independently. A node-level override **fully
replaces** the pipeline-level policy for that node.

```yaml
nodes:
  otlp/ingest:
    type: receiver:otlp
    config:
      protocols:
        grpc:
          listening_addr: "0.0.0.0:4317"
    # Override pipeline-level capture for this receiver.
    header_capture:
      headers:
        - match_names: ["x-tenant-id"]
          store_as: tenant_id

  otlp/export:
    type: exporter:otlp_grpc
    config:
      grpc_endpoint: "http://backend:4317"
    # Override pipeline-level propagation for this exporter.
    header_propagation:
      default:
        selector: all_captured
      overrides:
        - match:
            stored_names: ["authorization"]
          action: drop
```

- `header_capture` is only valid on **receiver** nodes. Setting it
  on a processor or exporter is a configuration error.
- `header_propagation` is only valid on **exporter** nodes. Setting
  it on a processor or receiver is a configuration error.

## Header Capture

The capture policy controls which inbound transport headers a
receiver extracts from each request. Only headers that match at
least one capture rule are extracted.

```yaml
header_capture:
  defaults:
    max_entries: 32       # default: 32
    max_name_bytes: 128   # default: 128
    max_value_bytes: 4096 # default: 4096
    on_error: drop        # default: drop
  headers:
    - match_names: ["x-tenant-id"]
      store_as: tenant_id
    - match_names: ["x-request-id"]
    - match_names: ["authorization"]
      sensitive: true
    - match_names: ["x-trace-context-bin"]
      value_kind: binary
```

### Capture Rules

Each entry in `headers` is a capture rule. A header is captured
when its wire name matches any entry in `match_names`
(case-insensitive).

- `match_names` (required): wire header names to match
  (case-insensitive)
- `store_as` (optional): normalized name used for policy matching
  and storage. Default: first matched name lowercased.
- `sensitive` (optional): marks the header as containing sensitive
  data (e.g., auth tokens). Default: `false`.
- `value_kind` (optional): override auto-detected value kind
  (`text` or `binary`). When omitted, headers ending in `-bin`
  are treated as binary; all others as text.

### Defaults

The `defaults` block sets limits applied to all captured headers
in a single request.

- `max_entries` (default `32`): maximum headers captured per
  request. Matching headers beyond this limit are skipped.
- `max_name_bytes` (default `128`): maximum byte length of a
  header name. Headers with longer names are skipped.
- `max_value_bytes` (default `4096`): maximum byte length of a
  header value. Headers with longer values are skipped.
- `on_error` (default `drop`): action when a limit is violated.
  Currently only `drop` is supported.

When any matching header is skipped due to a limit, the runtime
reports statistics indicating how many headers were skipped and
why (max entries reached, name too long, or value too long).

## Header Propagation

The propagation policy controls which captured headers an exporter
includes on outbound requests. It operates in two stages:

1. **Default selector** - Determines the baseline set of headers
   eligible for propagation.
2. **Overrides** - Per-header rules that can force-propagate or
   force-drop specific headers, regardless of the default.

```yaml
header_propagation:
  default:
    selector: all_captured  # default: none
    action: propagate       # default: propagate
    name: preserve          # default: preserve
    on_error: drop          # default: drop
  overrides:
    - match:
        stored_names: ["authorization"]
      action: drop
    - match:
        stored_names: ["x-internal-trace"]
      action: propagate
      name: stored_name
```

### Default Behavior

- `selector` (default `none`): which captured headers are
  candidates for propagation. See selector values below.
- `action` (default `propagate`): action applied to selected
  headers (`propagate` or `drop`).
- `name` (default `preserve`): how the outbound wire name is
  determined. See name strategy below.
- `on_error` (default `drop`): action on error. Currently only
  `drop` is supported.

### Selector Values

| Value | Behavior |
| --- | --- |
| `all_captured` | Propagate all captured headers. |
| `none` | Propagate nothing by default (default). |
| `!named [list]` | Propagate only listed stored names. |

When `none` is used, only headers explicitly matched by an override
with `action: propagate` are included on egress.

Example of the `named` selector:

```yaml
header_propagation:
  default:
    selector: !named
      - tenant_id
      - x-request-id
```

### Name Strategy

| Value | Behavior |
| --- | --- |
| `preserve` | Use original wire name (default). |
| `stored_name` | Use the normalized stored name. |

For example, if a header was captured from `X-Tenant-Id` and stored
as `tenant_id`, then `preserve` emits `X-Tenant-Id` on egress while
`stored_name` emits `tenant_id`.

### Overrides

Each override targets specific headers by their stored (normalized)
name and can force a different action or name strategy than the
default.

- `match.stored_names` (required): match headers whose stored name
  appears in this list (case-insensitive).
- `action` (optional): action for matched headers. Default:
  `propagate`.
- `name` (optional): override name strategy for matched headers.
  Inherits from `default.name` when omitted.
- `on_error` (optional): override error action for matched headers.
  Inherits from `default.on_error` when omitted.

Overrides are evaluated before the default selector. If a header
matches an override, the override's action is used and the default
selector is not consulted.

## Examples

### Capture and Forward Tenant ID

Capture a single header at the receiver and forward it through to
all exporters with the original wire name preserved.

```yaml
version: otel_dataflow/v1
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: ["x-tenant-id"]
          store_as: tenant_id
    header_propagation:
      default:
        selector: all_captured
groups:
  default:
    pipelines:
      main:
        nodes:
          otlp/ingest:
            type: receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "0.0.0.0:4317"
          batch:
            type: processor:batch
            config: {}
          otlp/export:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://backend:4317"
        connections:
          - from: otlp/ingest
            to: batch
          - from: batch
            to: otlp/export
```

### Drop Sensitive Headers on Egress

Capture multiple headers including authorization, but strip the
authorization header before forwarding to downstream services.

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: ["x-tenant-id"]
          store_as: tenant_id
        - match_names: ["x-request-id"]
        - match_names: ["authorization"]
          sensitive: true
    header_propagation:
      default:
        selector: all_captured
      overrides:
        - match:
            stored_names: ["authorization"]
          action: drop
```

### Allowlist: Propagate Only Named Headers

Instead of forwarding all captured headers, propagate only an
explicit list. Any captured header not in the list is dropped on
egress.

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: ["x-tenant-id"]
          store_as: tenant_id
        - match_names: ["x-request-id"]
        - match_names: ["authorization"]
          sensitive: true
        - match_names: ["x-debug-flags"]
    header_propagation:
      default:
        selector: !named
          - tenant_id
          - x-request-id
```

In this example, `authorization` and `x-debug-flags` are captured
(available for internal processing or logging) but are not forwarded
to downstream services.

### Per-Node Override

A pipeline captures tenant ID globally, but one specific exporter
needs a different propagation policy that renames the wire name to
the stored name.

```yaml
policies:
  transport_headers:
    header_capture:
      headers:
        - match_names: ["x-tenant-id"]
          store_as: tenant_id
    header_propagation:
      default:
        selector: all_captured

groups:
  default:
    pipelines:
      main:
        nodes:
          otlp/ingest:
            type: receiver:otlp
            config:
              protocols:
                grpc:
                  listening_addr: "0.0.0.0:4317"
          batch:
            type: processor:batch
            config: {}
          primary/export:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://primary:4317"
            # Pipeline-level propagation (preserve wire name).
          secondary/export:
            type: exporter:otlp_grpc
            config:
              grpc_endpoint: "http://secondary:4317"
            # Override: use stored name as wire name.
            header_propagation:
              default:
                selector: all_captured
                name: stored_name
        connections:
          - from: otlp/ingest
            to: batch
          - from: batch
            to: [primary/export, secondary/export]
```

In this example, `primary/export` sends the header as `X-Tenant-Id`
(the original wire name), while `secondary/export` sends it as
`tenant_id` (the stored name).

## Limitations

1. **`on_error` only supports `drop`** - No `log` or `reject`
   actions are available. Headers that violate limits are silently
   dropped.
2. **Exact match only** - `match_names` requires exact header name
   matches (case-insensitive). Regex and glob patterns are not
   supported.
