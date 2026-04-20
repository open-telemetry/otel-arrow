# Content Router Processor

The content router routes telemetry to named output ports based on a configured
resource attribute. It is an exclusive-routing processor: each inbound message
selects at most one downstream output route.

This processor is useful when one pipeline needs to split traffic by a content
dimension such as namespace, service, environment, or any other resource
attribute.

## Configuration

```yaml
processor:
  urn: urn:otel:processor:content_router
  config:
    routing_key:
      resource_attribute: service.namespace
    case_sensitive: false
    routes:
      frontend: frontend_pipeline
      backend: backend_pipeline
    default_output: fallback
    admission_policy:
      on_full: reject_immediately # or "backpressure"
```

Configuration fields:

- `routing_key.resource_attribute`: resource attribute used as the route key.
- `routes`: map from route-key values to output port names.
- `default_output`: optional output port for messages that do not match a
  configured route.
- `case_sensitive`: whether route-key value matching is case-sensitive.
  Defaults to `true`.
- `admission_policy.on_full`: handling for a selected route whose output
  channel is full. Defaults to `reject_immediately`.

Every configured route destination and `default_output`, when present, must
refer to an output port declared by the node configuration.

## Route Selection

For each message, the content router reads the configured resource attribute
from every resource in the batch:

- if every resource resolves to the same configured route, the message is sent
  to that route's output port
- if no configured route matches, `default_output` is used when present
- if the routing key is missing or no route matches and there is no
  `default_output`, the message is rejected
- if resources in one batch resolve to different destinations, the whole batch
  is rejected as mixed
- if the router cannot convert or inspect the payload format, the message is
  rejected as a conversion error

Mixed-batch, missing-key, unmatched, and conversion-error behavior is separate
from selected-route admission. Those cases are decided before an output route is
selected.

## Selected-Route Admission

Once an output route has been selected, the router never awaits the downstream
send in the main router task. This avoids head-of-line blocking: a full or
closed selected route cannot freeze the router and prevent unrelated routes
from making progress.

The selected route can produce three admission outcomes:

- `Accepted`: the message is forwarded to the selected output.
- `Full`: the output channel is temporarily unable to admit the message.
- `Closed`: the output route is unavailable.

`Closed` always produces an immediate route-local retryable NACK with
`cause = RouteClosed`.

`Full` follows `admission_policy.on_full`:

- `reject_immediately`: emit an immediate route-local retryable NACK with
  `cause = RouteFull`; unrelated healthy routes continue to flow.
- `backpressure`: park at most one message per blocked output port, keep
  admitting pdata while at least one selectable route is still making progress,
  close pdata admission only when every selectable route currently has a parked
  full message, and reopen admission once at least one parked route forwards.

Later messages for an already parked route are retryable-NACKed with
`cause = RouteFull`.

## Shutdown Behavior

If the content router owns locally parked messages when
`NodeControlMsg::Shutdown` starts:

- every parked message is retryable-NACKed locally
- those NACKs use `cause = NodeShutdown`
- the router clears its parked state instead of leaving messages in a local
  wait path

This applies only to work still owned by the router. Work already admitted to a
downstream channel is outside the router's scope.

## Ack/Nack Propagation Across Topics

Router-generated NACKs are local to the processor unless a topic hop is
configured to bridge them upstream.

- with `ack_propagation.mode: disabled`, a content-router NACK remains local to
  the downstream side of the topic hop
- with `ack_propagation.mode: auto`, that same NACK is bridged upstream across
  the topic hop

## Observability

The content router exposes counters for:

- received messages
- messages routed to matched routes
- messages routed to the default output
- NACKed messages
- missing routing keys
- conversion errors
- selected-route full rejections
- selected-route closed rejections

Selected-route NACKs include a machine-readable `NackCause`:

- `RouteFull`
- `RouteClosed`
- `NodeShutdown`

## Non-Goals and Future Direction

The content router does not currently guarantee:

- draining or NACKing work that has already been admitted to downstream
  channels when a route later closes
- a generic engine-wide admission policy shared by all multi-output processors
  such as `fanout_processor` or `transform_processor`

Future extensions may add richer blocked-route scheduling, bounded per-route
queueing, downstream lifecycle semantics for already-admitted work, or more
selective pause conditions when some selected routes are unavailable.
