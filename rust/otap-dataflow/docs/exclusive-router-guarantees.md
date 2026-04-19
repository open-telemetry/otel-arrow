<!-- markdownlint-disable-next-line MD041 -->
# Exclusive Router Guarantees

`content_router` and `signal_type_router` are exclusive-routing processors:
each inbound message selects at most one downstream output route.

This document defines the runtime guarantees they provide once a route has been
selected.

## Shared contract

For both routers:

- selected-route admission never awaits the downstream send in the main router
  task
- `Closed` on the selected route always produces an immediate route-local
  retryable NACK
- `default_output` uses the same admission policy as matched or named routing
  once the default route has been selected
- the router returns success after route-local rejection, so the node stays
  live and control traffic continues to flow

The routers choose among two policies for selected-route `Full`:

- `reject_immediately`
  - default policy
  - emit an immediate route-local retryable NACK with `cause = RouteFull`
  - unrelated healthy routes continue to flow
- `backpressure`
  - keep at most one parked message per blocked output port
  - keep admitting pdata while at least one selectable route is still making
    progress
  - close pdata admission for the router only when every selectable route
    currently has a parked full message
  - keep control traffic and wakeups flowing so the router can still react to
    shutdown and route recovery
  - reopen pdata admission once at least one parked route forwards
  - later messages for an already parked route are retryable-NACKed with
    `cause = RouteFull`

These policies are all implemented with explicit router-local state and
processor-local wakeups. They do not reintroduce the old head-of-line blocking
bug caused by awaiting the selected route send inside the main router task.

## Shutdown behavior

If a router has locally parked messages when `NodeControlMsg::Shutdown` starts:

- every parked message is retryable-NACKed locally
- those NACKs use `cause = NodeShutdown`
- the router clears its parked state instead of leaving messages stranded in a
  local wait path

This applies only to work still owned by the router. Work already admitted to a
downstream channel is outside the router's scope.

## Router-specific behavior

These guarantees apply only after an output route has been selected. Route
selection itself remains processor-specific.

### `content_router`

- routes by configured resource attribute values
- if no configured route matches, `default_output` is used when present
- if the routing key is missing or no route matches and there is no default
  output, the message is rejected as before
- mixed-batch rejection and conversion-error rejection are unchanged

### `signal_type_router`

- prefers the signal-type-specific named output (`logs`, `metrics`, `traces`)
  when that port is connected
- falls back to the node default output only when the type-specific named port
  is not connected
- existing drop behavior when no type-specific port is connected and no default
  output exists is unchanged

## Ack/Nack propagation across topic hops

These router guarantees are local to the processor. Whether a router-generated
NACK is bridged farther upstream depends on the topic's
`ack_propagation.mode`.

- with `ack_propagation.mode: disabled`, a router-generated NACK remains local
  to the downstream side of the topic hop
- with `ack_propagation.mode: auto`, that same NACK is bridged upstream across
  the topic hop

This means the admission policy is the same in both modes, but the visibility
of its NACKs to upstream publishers differs. See
[Configuration Model](./configuration-model.md#topics).

## Observability

Both routers expose separate telemetry for route rejection caused by:

- selected route full
- selected route closed

`signal_type_router` reports those counters per signal type. These counters are
distinct from unmatched-route, missing-key, conversion, and drop telemetry.

Routers also stamp a machine-readable `NackCause` alongside the existing
human-readable `reason` string:

- `RouteFull`
- `RouteClosed`
- `NodeShutdown`

## Non-goals of the current contract

These routers do not currently guarantee any of the following:

- draining or NACKing work that has already been admitted to downstream
  channels when a route later closes
- a generic engine-wide admission policy shared by all multi-output processors
  such as `fanout_processor` or `transform_processor`

## Future direction

The current policy surface is intentionally local to exclusive routers.

The next useful extensions would be:

- richer blocked-route scheduling only if a new exclusive router needs the same
  contract
- bounded per-route queueing beyond one parked message per blocked output port
- downstream lifecycle semantics for flushing already-admitted work when a
  route closes or a pipeline shuts down
- more selective pause conditions when some selected routes are unavailable
  rather than merely full

Those extensions should keep the same `NackCause` meanings instead of
reinterpreting `RouteFull`, `RouteClosed`, or `NodeShutdown`.
