<!-- markdownlint-disable-next-line MD041 -->
# Exclusive Router Guarantees

`content_router` and `signal_type_router` are exclusive-routing processors:
each inbound message selects at most one downstream output route.

This document defines the runtime guarantees they now provide when the selected
route is blocked or unavailable.

## Shared contract

For both routers:

- Selected-route admission is non-blocking.
- If the selected output route is writable, the message is forwarded normally.
- If the selected output route is full, that message is rejected immediately
  with a route-local retryable NACK.
- If the selected output route is closed, that message is rejected immediately
  with a route-local retryable NACK.
- The processor returns success after emitting that NACK, so the node stays
  live and unrelated healthy routes can continue to make progress.
- Default-route forwarding uses the same admission policy as matched or named
  route forwarding once the default route has been selected.

In practical terms, these routers isolate failure at the selected route:

- traffic for a blocked route is refused explicitly
- traffic for other healthy routes is not stalled by that blocked route

## Router-specific behavior

The new guarantee applies only after the router has selected an output route.
Other routing decisions remain processor-specific.

### `content_router`

- Routes by configured resource attribute values.
- If no configured route matches, `default_output` is used when present.
- If the routing key is missing or no route matches and there is no default
  output, the message is rejected as before.
- Mixed-batch rejection and conversion-error rejection are unchanged.

### `signal_type_router`

- Prefers the signal-type-specific named output (`logs`, `metrics`, `traces`)
  when that port is connected.
- Falls back to the node default output only when the signal-type-specific
  named port is not connected.
- If the selected named or default route is full or closed, the message is
  rejected immediately instead of waiting on that route.
- Existing drop behavior when no signal-type-specific port is connected and no
  default output exists is unchanged.

## Ack/Nack propagation across topic hops

These router guarantees are local to the processor. Whether a router-generated
NACK is bridged farther upstream depends on the topic's
`ack_propagation.mode`.

- With `ack_propagation.mode: disabled`, a route-local NACK remains local to
  the downstream side of the topic hop.
- With `ack_propagation.mode: auto`, that same route-local NACK is bridged
  upstream across the topic hop.

This means the router behavior is the same in both modes, but the visibility of
that rejection to upstream publishers differs. See
[Configuration Model](./configuration-model.md#topics).

## Observability

Both routers expose separate telemetry for route rejection caused by:

- selected route full
- selected route closed

`signal_type_router` reports those counters per signal type. These counters are
distinct from existing unmatched-route, missing-key, conversion, and drop
telemetry.

## Non-goals of the current contract

These routers do not currently guarantee any of the following:

- waiting for a blocked route before rejecting
- a router-local queue or scheduler for blocked routes
- node-level backpressure when all candidate routes are blocked
- the same admission policy for other multi-output processors such as
  `fanout_processor` or `transform_processor`

## Future direction

The current behavior is intentionally a narrow v1: reject immediately when the
selected route is full or closed, and keep unrelated routes live.

If configurable admission policy is added later, the recommended direction is:

- keep the current behavior as the default, backward-compatible baseline
- add any policy configuration locally to the router rather than as a generic
  engine-wide policy
- implement richer policies with explicit per-route blocked-state tracking and
  a node-local scheduler instead of awaiting the selected route in the main
  router task

For `content_router`, one plausible future configuration shape would be:

```yaml
config:
    routing_key:
      resource_attribute: service.namespace
    routes:
      service-a: service_a
      service-b: service_b
      service-c: service_c
  default_output: unmatched
  admission_policy:
    on_blocked_route: reject_immediately
```

Candidate future policy values:

- `reject_immediately`
  - current behavior
  - use non-blocking admission and emit an immediate route-local NACK on
    `Full`
- `wait_then_reject`
  - keep at most one blocked admission pending per route
  - if the route stays blocked past a configured timeout, NACK that message
  - do not stall unrelated routes while waiting
- `block_when_all_routes_blocked`
  - only meaningful once the router tracks per-route blocked state
  - stop admitting new input only when all admissible routes are blocked
  - resume as soon as any admissible route becomes writable

Important constraints for that future work:

- do not implement configurable policy by awaiting the selected route send in
  the main router task
- any timeout-based or "all routes blocked" behavior should use explicit
  per-route state and node-level scheduling
- telemetry and NACK semantics introduced by the current contract should remain
  stable so future policies add new states instead of redefining existing ones
